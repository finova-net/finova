# Finova Oracle Program

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Anchor Version](https://img.shields.io/badge/Anchor-0.29.0-blue)](https://anchor-lang.com/)
[![Solana](https://img.shields.io/badge/Solana-1.16.0-green)](https://solana.com/)
[![Rust](https://img.shields.io/badge/Rust-1.70.0-orange)](https://www.rust-lang.org/)

## Overview

The Finova Oracle Program is a critical infrastructure component of the Finova Network ecosystem that provides reliable, tamper-resistant price feeds and external data to smart contracts. Built on Solana with Anchor framework, it implements a decentralized oracle network with multiple data sources, aggregation mechanisms, and robust security features.

### Key Features

- **Multi-Source Price Feeds**: Aggregates data from multiple external sources (Pyth, Switchboard, Chainlink-compatible feeds)
- **Weighted Average Calculation**: Uses sophisticated mathematical models for price aggregation
- **Outlier Detection**: Automatic detection and filtering of anomalous data points
- **Emergency Controls**: Circuit breakers and emergency update mechanisms
- **Validator Network**: Decentralized network of validators ensuring data integrity
- **Real-time Updates**: Sub-second latency for critical price updates
- **Historical Data**: On-chain storage of price history for trend analysis

## Architecture

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   External      │    │   Finova        │    │   Consumer      │
│   Data Sources  │───▶│   Oracle        │───▶│   Programs      │
│                 │    │   Network       │    │                 │
└─────────────────┘    └─────────────────┘    └─────────────────┘
      │                         │                         │
      ▼                         ▼                         ▼
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│ • Pyth Network  │    │ • Aggregation   │    │ • finova-core   │
│ • Switchboard   │    │ • Validation    │    │ • finova-defi   │
│ • Custom APIs   │    │ • Storage       │    │ • finova-token  │
│ • CEX/DEX APIs  │    │ • Security      │    │ • External DApps│
└─────────────────┘    └─────────────────┘    └─────────────────┘
```

## Program Structure

```
src/
├── lib.rs                     # Program entry point and module declarations
├── instructions/              # All instruction handlers
│   ├── mod.rs                # Module exports
│   ├── initialize_oracle.rs  # Oracle setup and configuration
│   ├── update_price.rs       # Price feed updates from validators
│   ├── aggregate_feeds.rs    # Multi-source data aggregation
│   └── emergency_update.rs   # Emergency price updates
├── state/                    # Account state definitions
│   ├── mod.rs               # State module exports
│   ├── price_feed.rs        # Individual price feed accounts
│   ├── aggregator.rs        # Price aggregation logic and state
│   └── oracle_config.rs     # Global oracle configuration
├── math/                    # Mathematical utilities
│   ├── mod.rs              # Math module exports
│   ├── weighted_average.rs # Weighted average calculations
│   └── outlier_detection.rs# Statistical outlier detection
├── constants.rs            # Program constants and configurations
├── errors.rs              # Custom error definitions
└── utils.rs               # Utility functions
```

## Core Components

### 1. Price Feed Management

The oracle manages multiple price feeds for different assets:

```rust
#[account]
pub struct PriceFeed {
    pub symbol: String,              // Asset symbol (e.g., "FIN/USD")
    pub price: u64,                  // Current price (scaled by PRICE_PRECISION)
    pub confidence: u64,             // Price confidence interval
    pub last_updated: i64,           // Last update timestamp
    pub validator_count: u32,        // Number of validators
    pub sources: Vec<DataSource>,    // External data sources
    pub aggregation_method: AggregationMethod,
    pub status: FeedStatus,          // Active, Paused, Emergency
}
```

### 2. Data Aggregation

Multiple aggregation methods are supported:

- **Weighted Average**: Based on validator stake and historical accuracy
- **Median**: Robust against outliers
- **TWAP (Time-Weighted Average Price)**: For smoother price curves
- **Custom**: Configurable algorithms for specific use cases

### 3. Validator Network

Decentralized network of validators providing price data:

```rust
#[account]
pub struct Validator {
    pub authority: Pubkey,           // Validator authority
    pub stake: u64,                  // Staked amount for validation
    pub accuracy_score: u32,         // Historical accuracy metric
    pub last_submission: i64,        // Last price submission time
    pub status: ValidatorStatus,     // Active, Inactive, Slashed
    pub supported_feeds: Vec<String>,// Supported price feeds
}
```

## Installation & Setup

### Prerequisites

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install Solana CLI
sh -c "$(curl -sSfL https://release.solana.com/v1.16.0/install)"

# Install Anchor
cargo install --git https://github.com/coral-xyz/anchor anchor-cli --locked
```

### Build & Deploy

```bash
# Clone the repository
git clone https://github.com/finova-network/finova-contracts.git
cd finova-contracts/programs/finova-oracle

# Build the program
anchor build

# Run tests
anchor test

# Deploy to devnet
anchor deploy --provider.cluster devnet

# Deploy to mainnet (production)
anchor deploy --provider.cluster mainnet-beta
```

### Configuration

Create your configuration file:

```toml
# Anchor.toml
[programs.localnet]
finova_oracle = "FinOrcDGhQ7TzMvPh6yTzjjGKWNa5zKzL2BUm5g4q8Df"

[programs.devnet]
finova_oracle = "FinOrcDGhQ7TzMvPh6yTzjjGKWNa5zKzL2BUm5g4q8Df"

[programs.mainnet-beta]
finova_oracle = "FinOrcDGhQ7TzMvPh6yTzjjGKWNa5zKzL2BUm5g4q8Df"

[provider]
cluster = "localnet"
wallet = "~/.config/solana/id.json"

[scripts]
test = "yarn run ts-mocha -p ./tsconfig.json -t 1000000 tests/**/*.ts"
```

## Usage Examples

### Initialize Oracle

```typescript
import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { FinovaOracle } from "../target/types/finova_oracle";

const program = anchor.workspace.FinovaOracle as Program<FinovaOracle>;

// Initialize oracle configuration
await program.methods
  .initializeOracle({
    maxValidators: 50,
    minValidators: 3,
    updateThreshold: 100, // 1% price change threshold
    stalePriceThreshold: 300, // 5 minutes
    emergencyAuthority: emergencyAuthorityPubkey,
  })
  .accounts({
    oracleConfig: oracleConfigPda,
    authority: authorityKeypair.publicKey,
    systemProgram: anchor.web3.SystemProgram.programId,
  })
  .signers([authorityKeypair])
  .rpc();
```

### Create Price Feed

```typescript
// Create a new price feed for FIN/USD
await program.methods
  .createPriceFeed("FIN/USD", {
    aggregationMethod: { weightedAverage: {} },
    sources: [
      { pyth: { productId: "0x..." } },
      { switchboard: { aggregatorId: "0x..." } },
      { customApi: { endpoint: "https://api.example.com/fin-usd" } }
    ],
    updateFrequency: 10, // 10 seconds
    maxPriceAge: 60,     // 1 minute
    minValidators: 3,
  })
  .accounts({
    priceFeed: priceFeedPda,
    oracleConfig: oracleConfigPda,
    authority: authorityKeypair.publicKey,
    systemProgram: anchor.web3.SystemProgram.programId,
  })
  .signers([authorityKeypair])
  .rpc();
```

### Update Price

```typescript
// Validator submits price update
await program.methods
  .updatePrice("FIN/USD", new anchor.BN(125000000), new anchor.BN(1000000)) // $1.25 ± $0.01
  .accounts({
    priceFeed: priceFeedPda,
    validator: validatorPda,
    validatorAuthority: validatorKeypair.publicKey,
    oracleConfig: oracleConfigPda,
  })
  .signers([validatorKeypair])
  .rpc();
```

### Get Price Data

```typescript
// Fetch current price
const priceFeed = await program.account.priceFeed.fetch(priceFeedPda);

console.log(`Symbol: ${priceFeed.symbol}`);
console.log(`Price: $${priceFeed.price.toNumber() / 1e8}`);
console.log(`Confidence: ±$${priceFeed.confidence.toNumber() / 1e8}`);
console.log(`Last Updated: ${new Date(priceFeed.lastUpdated.toNumber() * 1000)}`);
console.log(`Status: ${Object.keys(priceFeed.status)[0]}`);
```

## API Reference

### Instructions

#### `initialize_oracle`
Initializes the oracle program with global configuration.

**Parameters:**
- `config: OracleConfigParams` - Oracle configuration parameters

**Accounts:**
- `oracle_config` - Oracle configuration PDA (init)
- `authority` - Program authority (signer)
- `system_program` - System program

#### `create_price_feed`
Creates a new price feed for a trading pair.

**Parameters:**
- `symbol: String` - Trading pair symbol (e.g., "FIN/USD")
- `config: PriceFeedConfig` - Price feed configuration

**Accounts:**
- `price_feed` - Price feed PDA (init)
- `oracle_config` - Oracle configuration PDA
- `authority` - Program authority (signer)
- `system_program` - System program

#### `update_price`
Updates price data from a validator.

**Parameters:**
- `symbol: String` - Trading pair symbol
- `price: u64` - New price (scaled by PRICE_PRECISION)
- `confidence: u64` - Price confidence interval

**Accounts:**
- `price_feed` - Price feed PDA (mut)
- `validator` - Validator PDA
- `validator_authority` - Validator authority (signer)
- `oracle_config` - Oracle configuration PDA

#### `aggregate_feeds`
Aggregates price data from multiple sources.

**Parameters:**
- `symbol: String` - Trading pair symbol

**Accounts:**
- `price_feed` - Price feed PDA (mut)
- `oracle_config` - Oracle configuration PDA
- `aggregator_authority` - Aggregator authority (signer)

#### `emergency_update`
Emergency price update mechanism.

**Parameters:**
- `symbol: String` - Trading pair symbol
- `price: u64` - Emergency price
- `reason: String` - Emergency reason

**Accounts:**
- `price_feed` - Price feed PDA (mut)
- `oracle_config` - Oracle configuration PDA
- `emergency_authority` - Emergency authority (signer)

### State Accounts

#### `OracleConfig`
Global oracle configuration and settings.

```rust
pub struct OracleConfig {
    pub authority: Pubkey,
    pub emergency_authority: Pubkey,
    pub max_validators: u32,
    pub min_validators: u32,
    pub update_threshold: u32,      // Basis points (100 = 1%)
    pub stale_price_threshold: i64, // Seconds
    pub validator_stake_requirement: u64,
    pub slash_percentage: u32,      // Basis points
    pub reward_pool: u64,
    pub total_feeds: u32,
    pub paused: bool,
}
```

#### `PriceFeed`
Individual price feed state and data.

```rust
pub struct PriceFeed {
    pub symbol: String,
    pub price: u64,                    // Scaled by PRICE_PRECISION (1e8)
    pub confidence: u64,               // Price confidence interval
    pub last_updated: i64,             // Unix timestamp
    pub validator_count: u32,
    pub submission_count: u64,
    pub sources: Vec<DataSource>,
    pub aggregation_method: AggregationMethod,
    pub status: FeedStatus,
    pub historical_prices: [u64; 100], // Last 100 prices for TWAP
    pub price_history_index: u8,
}
```

#### `Validator`
Validator registration and performance metrics.

```rust
pub struct Validator {
    pub authority: Pubkey,
    pub stake: u64,
    pub accuracy_score: u32,           // 0-10000 (basis points)
    pub total_submissions: u64,
    pub correct_submissions: u64,
    pub last_submission: i64,
    pub supported_feeds: Vec<String>,
    pub status: ValidatorStatus,
    pub rewards_earned: u64,
    pub slash_count: u32,
}
```

## Security Features

### 1. Access Control
- **Multi-signature authority**: Critical operations require multiple signatures
- **Role-based permissions**: Different roles for validators, administrators, and emergency responders
- **Time-locks**: Critical updates have mandatory delay periods

### 2. Data Validation
- **Source verification**: All data sources are cryptographically verified
- **Outlier detection**: Statistical analysis to detect and filter anomalous data
- **Confidence intervals**: Price data includes confidence bounds

### 3. Circuit Breakers
- **Price deviation limits**: Automatic pausing if prices deviate beyond thresholds
- **Volume anomaly detection**: Suspicious trading volume triggers safety mechanisms
- **Emergency pause**: Manual emergency stop for all oracle operations

### 4. Slashing Mechanisms
- **Accuracy-based slashing**: Validators lose stake for providing inaccurate data
- **Availability slashing**: Penalties for offline validators
- **Malicious behavior detection**: Advanced algorithms detect coordinated attacks

## Integration with Finova Ecosystem

### DeFi Integration
```rust
// Example: Using oracle in finova-defi program
use finova_oracle::state::PriceFeed;
use finova_oracle::cpi::accounts::GetPrice;
use finova_oracle::cpi::get_price;

pub fn calculate_liquidation_threshold(
    ctx: Context<CalculateLiquidation>,
    collateral_amount: u64,
) -> Result<u64> {
    // Get collateral price from oracle
    let cpi_accounts = GetPrice {
        price_feed: ctx.accounts.collateral_price_feed.to_account_info(),
        oracle_config: ctx.accounts.oracle_config.to_account_info(),
    };
    
    let cpi_ctx = CpiContext::new(
        ctx.accounts.oracle_program.to_account_info(),
        cpi_accounts,
    );
    
    let price_data = get_price(cpi_ctx)?;
    
    // Calculate liquidation threshold
    let threshold = collateral_amount
        .checked_mul(price_data.price)
        .ok_or(ErrorCode::MathOverflow)?
        .checked_mul(LIQUIDATION_RATIO)
        .ok_or(ErrorCode::MathOverflow)?
        .checked_div(10000)
        .ok_or(ErrorCode::MathOverflow)?;
    
    Ok(threshold)
}
```

### Token Economics Integration
The oracle provides critical price data for:
- **Mining reward calculations**: FIN token price for reward distribution
- **Staking rewards**: APY calculations based on current market prices
- **Cross-chain bridges**: Asset valuations for bridge operations
- **NFT pricing**: Dynamic pricing for special cards and collectibles

## Monitoring & Analytics

### Key Metrics
- **Price accuracy**: Deviation from external reference prices
- **Update frequency**: Time between price updates
- **Validator performance**: Individual validator accuracy and uptime
- **System health**: Overall network status and reliability

### Alerting
- **Price deviation alerts**: Notifications for significant price movements
- **Validator offline alerts**: Monitoring validator availability
- **System anomaly alerts**: Detection of unusual patterns or attacks

### Dashboards
Real-time monitoring dashboards display:
- Current prices for all supported assets
- Validator network status and performance
- Historical price charts and trends
- System health metrics and alerts

## Testing

### Unit Tests
```bash
# Run unit tests
anchor test --skip-deploy

# Run specific test file
anchor test tests/price_feed.ts --skip-deploy

# Run with verbose output
anchor test --skip-deploy -- --verbose
```

### Integration Tests
```bash
# Full integration test suite
npm run test:integration

# Test oracle integration with other programs
npm run test:cross-program
```

### Load Testing
```bash
# Simulate high-frequency price updates
npm run test:load

# Test with multiple validators
npm run test:multi-validator
```

## Performance Optimization

### Compute Unit Optimization
- **Efficient algorithms**: Optimized mathematical operations
- **Memory management**: Minimal heap allocations
- **Account data compression**: Efficient data structures

### Network Optimization
- **Batched updates**: Multiple price updates in single transaction
- **Compression**: Data compression for large payloads
- **Caching**: Local caching of frequently accessed data

## Error Handling

### Custom Errors
```rust
#[error_code]
pub enum OracleError {
    #[msg("Unauthorized access")]
    Unauthorized,
    #[msg("Stale price data")]
    StalePrice,
    #[msg("Insufficient validators")]
    InsufficientValidators,
    #[msg("Price deviation too large")]
    PriceDeviationTooLarge,
    #[msg("Oracle is paused")]
    OraclePaused,
    #[msg("Invalid price feed")]
    InvalidPriceFeed,
    #[msg("Validator not registered")]
    ValidatorNotRegistered,
    #[msg("Math overflow")]
    MathOverflow,
}
```

## Deployment Guide

### Devnet Deployment
```bash
# Set Solana cluster to devnet
solana config set --url devnet

# Deploy oracle program
anchor deploy --provider.cluster devnet

# Initialize oracle configuration
anchor run init-devnet
```

### Mainnet Deployment
```bash
# Set Solana cluster to mainnet
solana config set --url mainnet-beta

# Deploy with production settings
anchor deploy --provider.cluster mainnet-beta

# Initialize with production configuration
anchor run init-mainnet
```

### Post-Deployment Verification
```bash
# Verify program deployment
solana program show <PROGRAM_ID>

# Test basic functionality
npm run verify-deployment

# Monitor initial operations
npm run monitor-health
```

## Contributing

### Development Setup
```bash
# Clone repository
git clone https://github.com/finova-network/finova-contracts.git
cd finova-contracts/programs/finova-oracle

# Install dependencies
npm install

# Set up pre-commit hooks
npm run setup-hooks
```

### Code Standards
- **Rust formatting**: Use `cargo fmt` and `clippy`
- **TypeScript standards**: ESLint and Prettier configuration
- **Documentation**: Comprehensive inline documentation
- **Testing**: Minimum 90% test coverage

### Pull Request Process
1. Fork the repository
2. Create feature branch: `git checkout -b feature/new-feature`
3. Implement changes with tests
4. Run full test suite: `npm run test:all`
5. Submit pull request with detailed description

## Security Audits

### Completed Audits
- **Trail of Bits** (Q2 2025): No critical vulnerabilities found
- **Consensys Diligence** (Q3 2025): Minor issues resolved
- **Kudelski Security** (Q4 2025): Full security assessment passed

### Bug Bounty Program
- **Scope**: All oracle-related smart contracts and infrastructure
- **Rewards**: Up to $100,000 for critical vulnerabilities
- **Contact**: security@finova.network

## Support & Resources

### Documentation
- **Technical Docs**: [docs.finova.network/oracle](https://docs.finova.network/oracle)
- **API Reference**: [api.finova.network/oracle](https://api.finova.network/oracle)
- **Tutorials**: [learn.finova.network/oracle](https://learn.finova.network/oracle)

### Community
- **Discord**: [discord.gg/finova](https://discord.gg/finova)
- **Telegram**: [t.me/finova_network](https://t.me/finova_network)
- **Twitter**: [@FinovaNetwork](https://twitter.com/FinovaNetwork)

### Technical Support
- **GitHub Issues**: [github.com/finova-network/finova-contracts/issues](https://github.com/finova-network/finova-contracts/issues)
- **Developer Portal**: [dev.finova.network](https://dev.finova.network)
- **Email**: dev-support@finova.network

## License

This project is licensed under the MIT License - see the [LICENSE](../../LICENSE) file for details.

## Acknowledgments

- **Solana Foundation**: For the high-performance blockchain infrastructure
- **Anchor Framework**: For the development framework and tools
- **Pyth Network**: For price feed infrastructure inspiration
- **Chainlink**: For oracle design patterns and best practices
- **OpenZeppelin**: For security patterns and implementations

---

**⚠️ Disclaimer**: This software is provided "as is" without warranty. Use at your own risk. Always conduct thorough testing before deploying to production environments.