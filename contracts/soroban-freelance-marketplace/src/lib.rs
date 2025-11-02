#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, token, Address, Env, String, Vec};

// Task status enumeration
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TaskStatus {
    Open,
    Assigned,
    InProgress,
    UnderReview,
    Completed,
    Disputed,
    Cancelled,
}

// Bid structure
#[contracttype]
#[derive(Clone, Debug)]
pub struct Bid {
    pub freelancer: Address,
    pub amount: i128,
    pub proposal: String,
    pub delivery_time: u64, // in days
    pub timestamp: u64,
}

// Task structure - using separate storage for optional freelancer
#[contracttype]
#[derive(Clone, Debug)]
pub struct Task {
    pub id: u64,
    pub employer: Address,
    pub title: String,
    pub description: String,
    pub budget: i128,
    pub deadline: u64,
    pub status: TaskStatus,
    pub has_freelancer: bool, // Flag to check if freelancer is assigned
    pub escrow_amount: i128,
    pub created_at: u64,
    pub has_completed: bool,
    pub completed_at: u64,
}

// Dispute structure
#[contracttype]
#[derive(Clone, Debug)]
pub struct Dispute {
    pub task_id: u64,
    pub raised_by: Address,
    pub reason: String,
    pub timestamp: u64,
    pub resolved: bool,
}

// Storage keys
#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    TaskCounter,
    Task(u64),
    TaskFreelancer(u64), // Separate storage for assigned freelancer
    Bids(u64),
    Dispute(u64),
    TokenAddress,
    PlatformFee, // basis points (e.g., 250 = 2.5%)
    Admin,
}

#[contract]
pub struct FreelanceMarketplace;

#[contractimpl]
impl FreelanceMarketplace {
    /// Initialize the contract with token address, platform fee, and admin
    pub fn initialize(env: Env, token: Address, platform_fee: u32, admin: Address) {
        if env.storage().instance().has(&DataKey::Admin) {
            panic!("Contract already initialized");
        }

        env.storage().instance().set(&DataKey::TokenAddress, &token);
        env.storage()
            .instance()
            .set(&DataKey::PlatformFee, &platform_fee);
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::TaskCounter, &0u64);
    }

    /// Post a new task
    pub fn post_task(
        env: Env,
        employer: Address,
        title: String,
        description: String,
        budget: i128,
        deadline: u64,
    ) -> u64 {
        employer.require_auth();

        let task_id: u64 = env
            .storage()
            .instance()
            .get(&DataKey::TaskCounter)
            .unwrap_or(0);
        let new_task_id = task_id + 1;

        let task = Task {
            id: new_task_id,
            employer: employer.clone(),
            title,
            description,
            budget,
            deadline,
            status: TaskStatus::Open,
            has_freelancer: false,
            escrow_amount: 0,
            created_at: env.ledger().timestamp(),
            has_completed: false,
            completed_at: 0,
        };

        env.storage()
            .persistent()
            .set(&DataKey::Task(new_task_id), &task);
        env.storage()
            .instance()
            .set(&DataKey::TaskCounter, &new_task_id);

        // Initialize empty bids vector
        let bids: Vec<Bid> = Vec::new(&env);
        env.storage()
            .persistent()
            .set(&DataKey::Bids(new_task_id), &bids);

        new_task_id
    }

    /// Submit a bid for a task
    pub fn submit_bid(
        env: Env,
        task_id: u64,
        freelancer: Address,
        amount: i128,
        proposal: String,
        delivery_time: u64,
    ) {
        freelancer.require_auth();

        let task: Task = env
            .storage()
            .persistent()
            .get(&DataKey::Task(task_id))
            .expect("Task not found");

        if task.status != TaskStatus::Open {
            panic!("Task is not open for bids");
        }

        let bid = Bid {
            freelancer: freelancer.clone(),
            amount,
            proposal,
            delivery_time,
            timestamp: env.ledger().timestamp(),
        };

        let mut bids: Vec<Bid> = env
            .storage()
            .persistent()
            .get(&DataKey::Bids(task_id))
            .unwrap_or(Vec::new(&env));

        bids.push_back(bid);
        env.storage()
            .persistent()
            .set(&DataKey::Bids(task_id), &bids);
    }

    /// Accept a bid and move funds to escrow
    pub fn accept_bid(env: Env, task_id: u64, freelancer: Address) {
        let mut task: Task = env
            .storage()
            .persistent()
            .get(&DataKey::Task(task_id))
            .expect("Task not found");

        task.employer.require_auth();

        if task.status != TaskStatus::Open {
            panic!("Task is not open");
        }

        let bids: Vec<Bid> = env
            .storage()
            .persistent()
            .get(&DataKey::Bids(task_id))
            .expect("No bids found");

        let selected_bid = bids
            .iter()
            .find(|b| b.freelancer == freelancer)
            .expect("Bid not found");

        // Transfer funds to contract escrow
        let token_address: Address = env
            .storage()
            .instance()
            .get(&DataKey::TokenAddress)
            .expect("Token not set");
        let token_client = token::Client::new(&env, &token_address);

        token_client.transfer(
            &task.employer,
            &env.current_contract_address(),
            &selected_bid.amount,
        );

        // Update task
        task.status = TaskStatus::Assigned;
        task.has_freelancer = true;
        task.escrow_amount = selected_bid.amount;

        // Store freelancer address separately
        env.storage()
            .persistent()
            .set(&DataKey::TaskFreelancer(task_id), &freelancer);

        env.storage()
            .persistent()
            .set(&DataKey::Task(task_id), &task);
    }

    /// Freelancer marks task as in progress
    pub fn start_work(env: Env, task_id: u64, freelancer: Address) {
        freelancer.require_auth();

        let mut task: Task = env
            .storage()
            .persistent()
            .get(&DataKey::Task(task_id))
            .expect("Task not found");

        if !task.has_freelancer {
            panic!("Task has no assigned freelancer");
        }

        let assigned_freelancer: Address = env
            .storage()
            .persistent()
            .get(&DataKey::TaskFreelancer(task_id))
            .expect("Freelancer not found");

        if assigned_freelancer != freelancer {
            panic!("Not assigned to this task");
        }

        if task.status != TaskStatus::Assigned {
            panic!("Task not in assigned state");
        }

        task.status = TaskStatus::InProgress;
        env.storage()
            .persistent()
            .set(&DataKey::Task(task_id), &task);
    }

    /// Freelancer submits completed work
    pub fn submit_work(env: Env, task_id: u64, freelancer: Address) {
        freelancer.require_auth();

        let mut task: Task = env
            .storage()
            .persistent()
            .get(&DataKey::Task(task_id))
            .expect("Task not found");

        if !task.has_freelancer {
            panic!("Task has no assigned freelancer");
        }

        let assigned_freelancer: Address = env
            .storage()
            .persistent()
            .get(&DataKey::TaskFreelancer(task_id))
            .expect("Freelancer not found");

        if assigned_freelancer != freelancer {
            panic!("Not assigned to this task");
        }

        if task.status != TaskStatus::InProgress {
            panic!("Task not in progress");
        }

        task.status = TaskStatus::UnderReview;
        env.storage()
            .persistent()
            .set(&DataKey::Task(task_id), &task);
    }

    /// Employer approves work and releases payment
    pub fn approve_work(env: Env, task_id: u64) {
        let mut task: Task = env
            .storage()
            .persistent()
            .get(&DataKey::Task(task_id))
            .expect("Task not found");

        task.employer.require_auth();

        if task.status != TaskStatus::UnderReview {
            panic!("Task not under review");
        }

        if !task.has_freelancer {
            panic!("Task has no assigned freelancer");
        }

        let freelancer: Address = env
            .storage()
            .persistent()
            .get(&DataKey::TaskFreelancer(task_id))
            .expect("Freelancer not found");

        let platform_fee: u32 = env
            .storage()
            .instance()
            .get(&DataKey::PlatformFee)
            .unwrap_or(0);

        // Calculate platform fee and freelancer payment
        let fee_amount = (task.escrow_amount * platform_fee as i128) / 10000;
        let freelancer_payment = task.escrow_amount - fee_amount;

        let token_address: Address = env
            .storage()
            .instance()
            .get(&DataKey::TokenAddress)
            .expect("Token not set");
        let token_client = token::Client::new(&env, &token_address);

        // Pay freelancer
        token_client.transfer(
            &env.current_contract_address(),
            &freelancer,
            &freelancer_payment,
        );

        // Pay platform fee to admin
        if fee_amount > 0 {
            let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
            token_client.transfer(&env.current_contract_address(), &admin, &fee_amount);
        }

        task.status = TaskStatus::Completed;
        task.has_completed = true;
        task.completed_at = env.ledger().timestamp();
        task.escrow_amount = 0;

        env.storage()
            .persistent()
            .set(&DataKey::Task(task_id), &task);
    }

    /// Raise a dispute
    pub fn raise_dispute(env: Env, task_id: u64, caller: Address, reason: String) {
        caller.require_auth();

        let task: Task = env
            .storage()
            .persistent()
            .get(&DataKey::Task(task_id))
            .expect("Task not found");

        // Check if caller is employer or assigned freelancer
        let is_employer = caller == task.employer;
        let mut is_freelancer = false;

        if task.has_freelancer {
            let assigned_freelancer: Address = env
                .storage()
                .persistent()
                .get(&DataKey::TaskFreelancer(task_id))
                .expect("Freelancer not found");
            is_freelancer = caller == assigned_freelancer;
        }

        if !is_employer && !is_freelancer {
            panic!("Not authorized to raise dispute");
        }

        if task.status != TaskStatus::InProgress && task.status != TaskStatus::UnderReview {
            panic!("Cannot dispute task in current status");
        }

        let dispute = Dispute {
            task_id,
            raised_by: caller,
            reason,
            timestamp: env.ledger().timestamp(),
            resolved: false,
        };

        env.storage()
            .persistent()
            .set(&DataKey::Dispute(task_id), &dispute);

        let mut task = task;
        task.status = TaskStatus::Disputed;
        env.storage()
            .persistent()
            .set(&DataKey::Task(task_id), &task);
    }

    /// Admin resolves dispute (split can be 0-100 for employer)
    pub fn resolve_dispute(env: Env, task_id: u64, employer_percentage: u32) {
        let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        admin.require_auth();

        if employer_percentage > 100 {
            panic!("Invalid percentage");
        }

        let mut task: Task = env
            .storage()
            .persistent()
            .get(&DataKey::Task(task_id))
            .expect("Task not found");

        if task.status != TaskStatus::Disputed {
            panic!("Task not disputed");
        }

        if !task.has_freelancer {
            panic!("Task has no assigned freelancer");
        }

        let freelancer: Address = env
            .storage()
            .persistent()
            .get(&DataKey::TaskFreelancer(task_id))
            .expect("Freelancer not found");

        let token_address: Address = env
            .storage()
            .instance()
            .get(&DataKey::TokenAddress)
            .expect("Token not set");
        let token_client = token::Client::new(&env, &token_address);

        let employer_amount = (task.escrow_amount * employer_percentage as i128) / 100;
        let freelancer_amount = task.escrow_amount - employer_amount;

        // Refund employer
        if employer_amount > 0 {
            token_client.transfer(
                &env.current_contract_address(),
                &task.employer,
                &employer_amount,
            );
        }

        // Pay freelancer
        if freelancer_amount > 0 {
            token_client.transfer(
                &env.current_contract_address(),
                &freelancer,
                &freelancer_amount,
            );
        }

        task.status = TaskStatus::Completed;
        task.has_completed = true;
        task.completed_at = env.ledger().timestamp();
        task.escrow_amount = 0;

        env.storage()
            .persistent()
            .set(&DataKey::Task(task_id), &task);

        let mut dispute: Dispute = env
            .storage()
            .persistent()
            .get(&DataKey::Dispute(task_id))
            .unwrap();
        dispute.resolved = true;
        env.storage()
            .persistent()
            .set(&DataKey::Dispute(task_id), &dispute);
    }

    /// Cancel task (only if not assigned)
    pub fn cancel_task(env: Env, task_id: u64) {
        let mut task: Task = env
            .storage()
            .persistent()
            .get(&DataKey::Task(task_id))
            .expect("Task not found");

        task.employer.require_auth();

        if task.status != TaskStatus::Open {
            panic!("Can only cancel open tasks");
        }

        task.status = TaskStatus::Cancelled;
        env.storage()
            .persistent()
            .set(&DataKey::Task(task_id), &task);
    }

    /// Get task details
    pub fn get_task(env: Env, task_id: u64) -> Task {
        env.storage()
            .persistent()
            .get(&DataKey::Task(task_id))
            .expect("Task not found")
    }

    /// Get assigned freelancer for a task
    pub fn get_task_freelancer(env: Env, task_id: u64) -> Address {
        let task: Task = env
            .storage()
            .persistent()
            .get(&DataKey::Task(task_id))
            .expect("Task not found");

        if !task.has_freelancer {
            panic!("Task has no assigned freelancer");
        }

        env.storage()
            .persistent()
            .get(&DataKey::TaskFreelancer(task_id))
            .expect("Freelancer not found")
    }

    /// Get all bids for a task
    pub fn get_bids(env: Env, task_id: u64) -> Vec<Bid> {
        env.storage()
            .persistent()
            .get(&DataKey::Bids(task_id))
            .unwrap_or(Vec::new(&env))
    }

    /// Get dispute details if exists
    pub fn get_dispute(env: Env, task_id: u64) -> Dispute {
        env.storage()
            .persistent()
            .get(&DataKey::Dispute(task_id))
            .expect("Dispute not found")
    }

    /// Check if task has dispute
    pub fn has_dispute(env: Env, task_id: u64) -> bool {
        env.storage().persistent().has(&DataKey::Dispute(task_id))
    }

    /// Get total task count
    pub fn get_task_count(env: Env) -> u64 {
        env.storage()
            .instance()
            .get(&DataKey::TaskCounter)
            .unwrap_or(0)
    }

    /// Get platform fee
    pub fn get_platform_fee(env: Env) -> u32 {
        env.storage()
            .instance()
            .get(&DataKey::PlatformFee)
            .unwrap_or(0)
    }

    /// Update platform fee (admin only)
    pub fn update_platform_fee(env: Env, new_fee: u32) {
        let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        admin.require_auth();

        if new_fee > 10000 {
            panic!("Fee cannot exceed 100%");
        }

        env.storage()
            .instance()
            .set(&DataKey::PlatformFee, &new_fee);
    }
}
