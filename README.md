# 🚀 Stellar Freelance Marketplace

> A decentralized freelance marketplace built on Stellar using Soroban smart contracts. Enables trustless transactions between employers and freelancers with built-in escrow, bidding system, and dispute resolution.

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Stellar](https://img.shields.io/badge/Stellar-Soroban-blue.svg)](https://stellar.org/soroban)
[![Rust](https://img.shields.io/badge/Rust-1.74+-orange.svg)](https://www.rust-lang.org/)

## 🌟 Features

### Core Functionality

- **📋 Task Posting**: Employers create tasks with XLM budgets and deadlines
- **💰 Competitive Bidding**: Freelancers submit proposals with custom rates and timelines
- **🔒 Escrow Protection**: Funds locked in smart contract until work completion
- **⚡ Fast Settlement**: Instant payouts upon approval using Stellar's speed
- **💸 Low Fees**: Minimal transaction costs with configurable platform fees (default 2.5%)
- **🌍 Global Access**: Permissionless participation from anywhere
- **⚖️ Dispute Resolution**: Built-in arbitration system for conflict management

### Security & Trust

- ✅ Funds held in auditable smart contract escrow
- ✅ Multi-stage workflow with clear state transitions
- ✅ Authorization checks on all critical operations
- ✅ Immutable task history and transparent bidding

## 📋 Table of Contents

- [Architecture](#-architecture)
- [Installation](#-installation)
- [Deployment](#-deployment)
- [Usage](#-usage)
- [Smart Contract API](#-smart-contract-api)
- [Development](#-development)
- [Testing](#-testing)
- [Contributing](#-contributing)
- [License](#-license)

## 🏗️ Architecture

### Smart Contract Structure

```
FreelanceMarketplace
├── Task Management
│   ├── post_task()
│   ├── cancel_task()
│   └── get_task()
├── Bidding System
│   ├── submit_bid()
│   ├── accept_bid()
│   └── get_bids()
├── Work Flow
│   ├── start_work()
│   ├── submit_work()
│   └── approve_work()
└── Dispute Handling
    ├── raise_dispute()
    └── resolve_dispute()
```

### Task Lifecycle

```
┌─────┐     ┌──────────┐     ┌────────────┐     ┌─────────────┐     ┌───────────┐
│Open │────▶│Assigned  │────▶│InProgress  │────▶│UnderReview  │────▶│Completed  │
└─────┘     └──────────┘     └────────────┘     └─────────────┘     └───────────┘
   │            │                   │                    │
   │            │                   │                    │
   ▼            ▼                   ▼                    ▼
┌─────────┐  ┌────────────────────────────────────────────┐
│Cancelled│  │              Disputed                      │
└─────────┘  └────────────────────────────────────────────┘
```

## 🛠️ Installation

### Prerequisites

- **Rust** 1.74.0 or higher ([Install Rust](https://www.rust-lang.org/tools/install))
- **Soroban CLI** ([Installation Guide](https://soroban.stellar.org/docs/getting-started/setup))
- **Stellar Account** with testnet XLM ([Get Testnet XLM](https://laboratory.stellar.org/#account-creator))

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install Soroban CLI
cargo install --locked soroban-cli

# Add WASM target
rustup target add wasm32-unknown-unknown
```

### Build the Contract

```bash
# Clone the repository
git clone https://github.com/yourusername/soroban-freelance-marketplace.git
cd soroban-freelance-marketplace

# Build optimized WASM
cargo build --target wasm32-unknown-unknown --release

# Optimize the contract (optional but recommended)
soroban contract optimize \
  --wasm target/wasm32-unknown-unknown/release/soroban_freelance_marketplace.wasm
```

## 🚀 Deployment

### 1. Configure Network

```bash
# Add Stellar Testnet
soroban network add testnet \
  --rpc-url https://soroban-testnet.stellar.org \
  --network-passphrase "Test SDF Network ; September 2015"

# Add Mainnet (for production)
soroban network add mainnet \
  --rpc-url https://soroban-mainnet.stellar.org \
  --network-passphrase "Public Global Stellar Network ; September 2015"
```

### 2. Deploy Contract

```bash
# Deploy to testnet
soroban contract deploy \
  --wasm target/wasm32-unknown-unknown/release/soroban_freelance_marketplace.wasm \
  --source YOUR_SECRET_KEY \
  --network testnet

# Save the contract ID returned
export CONTRACT_ID=<returned_contract_id>
```

### 3. Initialize Contract

```bash
# Initialize with parameters
# platform_fee in basis points (250 = 2.5%)
soroban contract invoke \
  --id $CONTRACT_ID \
  --source YOUR_SECRET_KEY \
  --network testnet \
  -- initialize \
  --token STELLAR_ASSET_CONTRACT_ID \
  --platform_fee 250 \
  --admin YOUR_ADMIN_ADDRESS
```

## 📖 Usage

### For Employers

#### Post a Task

```bash
soroban contract invoke \
  --id $CONTRACT_ID \
  --source EMPLOYER_SECRET \
  --network testnet \
  -- post_task \
  --employer EMPLOYER_ADDRESS \
  --title "\"Build Mobile App\"" \
  --description "\"Need React Native developer\"" \
  --budget 50000000000 \
  --deadline 1735689600

# Returns: task_id (e.g., 1)
```

#### View Bids for a Task

```bash
soroban contract invoke \
  --id $CONTRACT_ID \
  --network testnet \
  -- get_bids \
  --task_id 1
```

#### Accept a Bid

```bash
# This transfers funds from employer to contract escrow
soroban contract invoke \
  --id $CONTRACT_ID \
  --source EMPLOYER_SECRET \
  --network testnet \
  -- accept_bid \
  --task_id 1 \
  --freelancer FREELANCER_ADDRESS
```

#### Approve Completed Work

```bash
# Releases payment to freelancer (minus platform fee)
soroban contract invoke \
  --id $CONTRACT_ID \
  --source EMPLOYER_SECRET \
  --network testnet \
  -- approve_work \
  --task_id 1
```

### For Freelancers

#### Submit a Bid

```bash
soroban contract invoke \
  --id $CONTRACT_ID \
  --source FREELANCER_SECRET \
  --network testnet \
  -- submit_bid \
  --task_id 1 \
  --freelancer FREELANCER_ADDRESS \
  --amount 45000000000 \
  --proposal "\"5+ years React Native experience\"" \
  --delivery_time 90
```

#### Start Working

```bash
soroban contract invoke \
  --id $CONTRACT_ID \
  --source FREELANCER_SECRET \
  --network testnet \
  -- start_work \
  --task_id 1 \
  --freelancer FREELANCER_ADDRESS
```

#### Submit Completed Work

```bash
soroban contract invoke \
  --id $CONTRACT_ID \
  --source FREELANCER_SECRET \
  --network testnet \
  -- submit_work \
  --task_id 1 \
  --freelancer FREELANCER_ADDRESS
```

### Dispute Management

#### Raise a Dispute

```bash
# Can be called by employer or freelancer
soroban contract invoke \
  --id $CONTRACT_ID \
  --source CALLER_SECRET \
  --network testnet \
  -- raise_dispute \
  --task_id 1 \
  --caller CALLER_ADDRESS \
  --reason "\"Work incomplete per requirements\""
```

#### Resolve Dispute (Admin Only)

```bash
# employer_percentage: 0-100 (0 = all to freelancer, 100 = all to employer)
soroban contract invoke \
  --id $CONTRACT_ID \
  --source ADMIN_SECRET \
  --network testnet \
  -- resolve_dispute \
  --task_id 1 \
  --employer_percentage 50
```

### Query Functions

```bash
# Get task details
soroban contract invoke \
  --id $CONTRACT_ID \
  --network testnet \
  -- get_task \
  --task_id 1

# Get assigned freelancer
soroban contract invoke \
  --id $CONTRACT_ID \
  --network testnet \
  -- get_task_freelancer \
  --task_id 1

# Get total task count
soroban contract invoke \
  --id $CONTRACT_ID \
  --network testnet \
  -- get_task_count

# Check if dispute exists
soroban contract invoke \
  --id $CONTRACT_ID \
  --network testnet \
  -- has_dispute \
  --task_id 1

# Get platform fee
soroban contract invoke \
  --id $CONTRACT_ID \
  --network testnet \
  -- get_platform_fee
```

## 📚 Smart Contract API

### Initialization

| Function     | Parameters                                          | Description                                                   |
| ------------ | --------------------------------------------------- | ------------------------------------------------------------- |
| `initialize` | `token: Address, platform_fee: u32, admin: Address` | Initialize contract with token, fee (basis points), and admin |

### Task Management

| Function         | Parameters                                                                           | Description                      |
| ---------------- | ------------------------------------------------------------------------------------ | -------------------------------- |
| `post_task`      | `employer: Address, title: String, description: String, budget: i128, deadline: u64` | Create new task                  |
| `cancel_task`    | `task_id: u64`                                                                       | Cancel open task (employer only) |
| `get_task`       | `task_id: u64`                                                                       | Get task details                 |
| `get_task_count` | -                                                                                    | Get total number of tasks        |

### Bidding

| Function     | Parameters                                                                              | Description                         |
| ------------ | --------------------------------------------------------------------------------------- | ----------------------------------- |
| `submit_bid` | `task_id: u64, freelancer: Address, amount: i128, proposal: String, delivery_time: u64` | Submit bid on task                  |
| `accept_bid` | `task_id: u64, freelancer: Address`                                                     | Accept bid and lock funds in escrow |
| `get_bids`   | `task_id: u64`                                                                          | Get all bids for task               |

### Workflow

| Function       | Parameters                          | Description                      |
| -------------- | ----------------------------------- | -------------------------------- |
| `start_work`   | `task_id: u64, freelancer: Address` | Mark task as in progress         |
| `submit_work`  | `task_id: u64, freelancer: Address` | Submit work for review           |
| `approve_work` | `task_id: u64`                      | Approve work and release payment |

### Disputes

| Function          | Parameters                                      | Description                  |
| ----------------- | ----------------------------------------------- | ---------------------------- |
| `raise_dispute`   | `task_id: u64, caller: Address, reason: String` | Raise dispute on task        |
| `resolve_dispute` | `task_id: u64, employer_percentage: u32`        | Resolve dispute (admin only) |
| `get_dispute`     | `task_id: u64`                                  | Get dispute details          |
| `has_dispute`     | `task_id: u64`                                  | Check if dispute exists      |

### Administration

| Function              | Parameters     | Description                      |
| --------------------- | -------------- | -------------------------------- |
| `get_platform_fee`    | -              | Get current platform fee         |
| `update_platform_fee` | `new_fee: u32` | Update platform fee (admin only) |

## 🔧 Development

### Project Structure

```
soroban-freelance-marketplace/
├── src/
│   └── lib.rs              # Main contract code
├── Cargo.toml              # Rust dependencies
├── README.md               # This file
└── target/                 # Build output
```

### Run Tests

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_name
```

### Local Development

```bash
# Start local Stellar network (for development)
soroban network start local

# Deploy to local network
soroban contract deploy \
  --wasm target/wasm32-unknown-unknown/release/soroban_freelance_marketplace.wasm \
  --source DEVELOPMENT_SECRET \
  --network local
```

## 🧪 Testing

The contract includes comprehensive tests for:

- ✅ Task creation and lifecycle
- ✅ Bidding system
- ✅ Escrow fund management
- ✅ Work approval and payment release
- ✅ Dispute raising and resolution
- ✅ Authorization checks
- ✅ Edge cases and error handling

## 🗺️ Roadmap

- [ ] Multi-milestone tasks with partial payments
- [ ] Reputation system for freelancers and employers
- [ ] Automatic deadline enforcement
- [ ] Support for multiple token types
- [ ] Decentralized dispute resolution (DAO voting)
- [ ] Skill-based task categorization
- [ ] Freelancer portfolio integration

## 🤝 Contributing

We welcome contributions! Please follow these steps:

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

### Code Style

- Follow Rust standard formatting (`cargo fmt`)
- Ensure all tests pass (`cargo test`)
- Add tests for new features
- Update documentation as needed

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- [Stellar Development Foundation](https://stellar.org/) for Soroban platform
- [Rust Community](https://www.rust-lang.org/community) for excellent tooling
- All contributors and early adopters

## 📞 Contact

- **Project Link**: [https://github.com/yourusername/soroban-freelance-marketplace](https://github.com/yourusername/soroban-freelance-marketplace)
- **Issues**: [https://github.com/yourusername/soroban-freelance-marketplace/issues](https://github.com/yourusername/soroban-freelance-marketplace/issues)
- **Stellar Discord**: [https://discord.gg/stellar](https://discord.gg/stellar)

## ⭐ Support

If you find this project useful, please consider:

- Giving it a ⭐ on GitHub
- Sharing it with others
- Contributing to its development

---

**Built with ❤️ on Stellar**
