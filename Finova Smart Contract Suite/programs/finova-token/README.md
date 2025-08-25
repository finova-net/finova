# Finova Token Program

## 🏆 Enterprise-Grade Multi-Token Ecosystem on Solana

The Finova Token program implements a sophisticated multi-token ecosystem that powers the entire Finova Network social-fi platform. Built with security, scalability, and economic sustainability at its core.

## 📋 Table of Contents

- [Overview](#overview)
- [Token Architecture](#token-architecture)
- [Features](#features)
- [Program Structure](#program-structure)
- [Installation](#installation)
- [Usage](#usage)
- [Instructions](#instructions)
- [State Management](#state-management)
- [Security Features](#security-features)
- [Testing](#testing)
- [Deployment](#deployment)
- [API Reference](#api-reference)
- [Economic Model](#economic-model)
- [Integration Examples](#integration-examples)
- [Troubleshooting](#troubleshooting)
- [Contributing](#contributing)

## 🎯 Overview

The Finova Token program manages four interconnected tokens that form the backbone of the Finova Network ecosystem:

- **$FIN**: Primary utility token with mining-based distribution
- **$sFIN**: Liquid staking derivative with auto-compounding rewards
- **$USDfin**: Synthetic stablecoin pegged to USD
- **$sUSDfin**: Yield-bearing stablecoin with DeFi integration

### Key Innovations

✅ **Exponential Regression Algorithm**: Pi Network-inspired fair distribution  
✅ **Integrated Staking System**: Liquid staking with enhanced rewards  
✅ **Anti-Whale Mechanisms**: Progressive taxation and caps  
✅ **Real-time Yield Generation**: Auto-compounding staking rewards  
✅ **Cross-Program Integration**: Seamless interaction with core Finova programs  

## 🏗️ Token Architecture

### Multi-Token Design Pattern

```
Finova Token Ecosystem
├── $FIN (Primary)
│   ├── Mining Rewards
│   ├── Governance Rights
│   ├── Staking Collateral
│   └── NFT Purchases
├── $sFIN (Staked FIN)
│   ├── Liquid Staking Receipt
│   ├── Auto-compounding
│   ├── Enhanced Mining Rates
│   └── Governance Weight
├── $USDfin (Stablecoin)
│   ├── USD Pegged (1:1)
│   ├── Gas Fee Payments
│   ├── E-wallet Integration
│   └── Stable Transactions
└── $sUSDfin (Staked USD)
    ├── Yield Bearing
    ├── DeFi Integration
    ├── Collateral Usage
    └── Treasury Management
```

### Economic Formula Integration

The token program implements the core Finova economic formulas:

```rust
// Core Mining Formula
Final_Reward = Base_Mining_Rate × XP_Multiplier × RP_Multiplier × Quality_Score × Network_Regression

// Exponential Regression (Anti-whale)
Regression_Factor = e^(-0.001 × User_Total_Holdings)

// Staking Rewards
Staking_Reward = (Staked_Amount / Total_Staked) × Pool_Rewards × Multiplier_Effects
```

## ✨ Features

### 🔄 Dynamic Token Management
- **Automated Minting**: Based on network activity and mining algorithms
- **Controlled Burning**: Deflationary mechanisms through usage
- **Cross-token Swaps**: Seamless conversion between token types
- **Yield Generation**: Real-time reward calculation and distribution

### 🛡️ Security Features
- **Multi-signature Controls**: Critical operations require multiple signatures
- **Time-locked Operations**: Delayed execution for sensitive changes
- **Circuit Breakers**: Emergency pause functionality
- **Audit Trail**: Complete transaction history and event logging

### 📊 Advanced Analytics
- **Real-time Metrics**: Supply, demand, and utilization tracking
- **Economic Indicators**: Inflation, deflation, and growth metrics
- **User Analytics**: Holdings distribution and activity patterns
- **Network Health**: Overall ecosystem performance monitoring

## 📁 Program Structure

```
src/
├── lib.rs                 # Program entry point and exports
├── instructions/          # All program instructions
│   ├── mod.rs            # Instruction module exports
│   ├── initialize_mint.rs # Mint initialization logic
│   ├── mint_tokens.rs    # Token minting operations
│   ├── burn_tokens.rs    # Token burning operations
│   ├── stake_tokens.rs   # Staking mechanism
│   ├── unstake_tokens.rs # Unstaking with rewards
│   └── claim_rewards.rs  # Reward distribution
├── state/                # Account state definitions
│   ├── mod.rs           # State module exports
│   ├── mint_info.rs     # Mint configuration data
│   ├── stake_account.rs # User staking information
│   └── reward_pool.rs   # Reward pool management
├── events/              # Program events
│   ├── mod.rs          # Event module exports
│   ├── mint.rs         # Minting events
│   ├── burn.rs         # Burning events
│   └── stake.rs        # Staking events
├── constants.rs         # Program constants
├── errors.rs           # Custom error definitions
└── utils.rs            # Utility functions
```

## 🚀 Installation

### Prerequisites

- **Rust**: 1.70.0 or later
- **Solana CLI**: 1.16.0 or later
- **Anchor Framework**: 0.29.0 or later
- **Node.js**: 18.0.0 or later (for client interactions)

### Build Instructions

```bash
# Clone the repository
git clone https://github.com/finova-network/finova-contracts.git
cd finova-contracts

# Install dependencies
anchor build

# Run tests
anchor test

# Deploy to devnet
anchor deploy --provider.cluster devnet
```

### Environment Setup

```bash
# Copy environment template
cp .env.example .env

# Configure your environment
SOLANA_NETWORK=devnet
PROGRAM_ID=your_program_id_here
AUTHORITY_KEYPAIR=path/to/authority.json
```

## 💻 Usage

### Basic Token Operations

```typescript
import { FinovaTokenClient } from '@finova/token-sdk';

// Initialize client
const client = new FinovaTokenClient({
  connection: connection,
  wallet: wallet,
  programId: FINOVA_TOKEN_PROGRAM_ID
});

// Mint new tokens (authority only)
await client.mintTokens({
  mint: finMintAddress,
  to: userTokenAccount,
  amount: 1000 * 10**9, // 1000 FIN
  mintType: 'mining_reward'
});

// Stake tokens
await client.stakeTokens({
  mint: finMintAddress,
  amount: 500 * 10**9, // 500 FIN
  stakingTier: 'gold'
});

// Claim staking rewards
await client.claimRewards({
  stakeAccount: userStakeAccount
});
```

### Advanced Integration

```typescript
// Get user's complete token portfolio
const portfolio = await client.getUserPortfolio(userPublicKey);
console.log(portfolio);
// {
//   fin: { balance: 1500, staked: 500, rewards: 25 },
//   sFin: { balance: 525, apy: 12.5 },
//   usdFin: { balance: 100 },
//   sUsdFin: { balance: 50, yield: 6.8 }
// }

// Calculate optimal staking strategy
const strategy = await client.calculateOptimalStaking({
  currentHoldings: portfolio.fin.balance,
  stakingGoal: 'maximize_yield',
  timeHorizon: '1_year'
});
```

## 📋 Instructions

### Initialize Mint

Creates a new token mint with Finova-specific configurations.

```rust
pub fn initialize_mint(
    ctx: Context<InitializeMint>,
    token_type: TokenType,
    decimals: u8,
    supply_cap: Option<u64>,
    mint_authority: Option<Pubkey>,
    freeze_authority: Option<Pubkey>,
) -> Result<()>
```

**Parameters:**
- `token_type`: Type of token (FIN, sFIN, USDfin, sUSDfin)
- `decimals`: Token decimal places (9 for all Finova tokens)
- `supply_cap`: Maximum supply limit (if applicable)
- `mint_authority`: Authority allowed to mint tokens
- `freeze_authority`: Authority allowed to freeze accounts

### Mint Tokens

Mints new tokens based on network activity and economic formulas.

```rust
pub fn mint_tokens(
    ctx: Context<MintTokens>,
    amount: u64,
    mint_reason: MintReason,
    recipient_data: RecipientData,
) -> Result<()>
```

**Mint Reasons:**
- `MiningReward`: From user mining activity
- `StakingReward`: From staking yield generation
- `ReferralBonus`: From referral network activity
- `LiquidityIncentive`: For providing liquidity
- `GovernanceReward`: For DAO participation

### Burn Tokens

Burns tokens for deflationary pressure and utility consumption.

```rust
pub fn burn_tokens(
    ctx: Context<BurnTokens>,
    amount: u64,
    burn_reason: BurnReason,
) -> Result<()>
```

**Burn Reasons:**
- `TransactionFee`: 0.1% of all transfers
- `SpecialCardUsage`: NFT utility consumption  
- `WhaleTax`: Progressive taxation on large holdings
- `StabilityMechanism`: Economic stability maintenance

### Stake Tokens

Converts FIN tokens to sFIN with enhanced rewards.

```rust
pub fn stake_tokens(
    ctx: Context<StakeTokens>,
    amount: u64,
    staking_tier: StakingTier,
    lock_period: Option<u64>,
) -> Result<()>
```

**Staking Tiers:**
- `Bronze`: 100-499 FIN (8% APY, +20% mining boost)
- `Silver`: 500-999 FIN (10% APY, +35% mining boost)
- `Gold`: 1,000-4,999 FIN (12% APY, +50% mining boost)
- `Platinum`: 5,000-9,999 FIN (14% APY, +75% mining boost)
- `Diamond`: 10,000+ FIN (15% APY, +100% mining boost)

### Claim Rewards

Distributes accumulated staking and mining rewards.

```rust
pub fn claim_rewards(
    ctx: Context<ClaimRewards>,
    reward_types: Vec<RewardType>,
) -> Result<()>
```

**Reward Types:**
- `StakingYield`: Base staking APY rewards
- `MiningBonus`: Enhanced mining from staking
- `LoyaltyBonus`: Long-term staking incentives
- `GovernanceReward`: DAO participation rewards

## 📊 State Management

### MintInfo Account

```rust
#[account]
pub struct MintInfo {
    pub token_type: TokenType,           // Type of token
    pub mint: Pubkey,                   // Token mint address
    pub decimals: u8,                   // Token decimals
    pub supply_cap: Option<u64>,        // Maximum supply
    pub current_supply: u64,            // Current circulating supply
    pub total_minted: u64,              // Total ever minted
    pub total_burned: u64,              // Total ever burned
    pub mint_authority: Option<Pubkey>,  // Mint authority
    pub freeze_authority: Option<Pubkey>, // Freeze authority
    pub created_at: i64,                // Creation timestamp
    pub last_updated: i64,              // Last update timestamp
    pub economic_params: EconomicParams, // Economic formula parameters
}
```

### StakeAccount

```rust
#[account]
pub struct StakeAccount {
    pub owner: Pubkey,                  // Stake account owner
    pub mint: Pubkey,                   // Staked token mint
    pub staked_amount: u64,             // Amount currently staked
    pub s_token_amount: u64,            // sFIN tokens received
    pub stake_timestamp: i64,           // When staking started
    pub last_reward_claim: i64,         // Last reward claim time
    pub accumulated_rewards: u64,       // Unclaimed rewards
    pub staking_tier: StakingTier,      // Current staking tier
    pub lock_period: Option<u64>,       // Optional lock period
    pub multiplier_effects: MultiplierEffects, // Active multipliers
}
```

### RewardPool

```rust
#[account]
pub struct RewardPool {
    pub pool_type: RewardPoolType,      // Type of reward pool
    pub total_rewards: u64,             // Total rewards allocated
    pub distributed_rewards: u64,       // Rewards already distributed
    pub participants: u32,              // Number of participants
    pub last_distribution: i64,         // Last distribution timestamp
    pub distribution_rate: u64,         // Rewards per second
    pub pool_authority: Pubkey,         // Pool management authority
}
```

## 🔒 Security Features

### Multi-Signature Requirements

Critical operations require multiple signatures:

```rust
// Authority validation for sensitive operations
pub fn validate_authority(
    authority: &Signer,
    required_authorities: &[Pubkey],
    min_signatures: u8,
) -> Result<()> {
    // Multi-sig validation logic
}
```

### Circuit Breakers

Emergency pause functionality:

```rust
#[account]
pub struct EmergencyState {
    pub is_paused: bool,
    pub pause_authority: Pubkey,
    pub pause_timestamp: Option<i64>,
    pub pause_reason: String,
}
```

### Anti-Whale Mechanisms

Progressive taxation and caps:

```rust
pub fn calculate_whale_tax(
    user_holdings: u64,
    transaction_amount: u64,
) -> u64 {
    let holdings_ratio = user_holdings as f64 / TOTAL_SUPPLY as f64;
    if holdings_ratio > 0.01 { // 1% of total supply
        let tax_rate = (holdings_ratio * 100.0).min(50.0); // Max 50% tax
        (transaction_amount as f64 * tax_rate / 100.0) as u64
    } else {
        0
    }
}
```

## 🧪 Testing

### Comprehensive Test Suite

```bash
# Run all tests
anchor test

# Run specific test categories
anchor test --skip-deploy tests/unit/
anchor test --skip-deploy tests/integration/
anchor test --skip-deploy tests/security/

# Run with coverage
anchor test --skip-deploy -- --nocapture
```

### Test Categories

**Unit Tests:**
- Individual instruction testing
- State validation
- Mathematical formula verification
- Error handling

**Integration Tests:**
- Cross-program interactions
- End-to-end token flows
- Economic model validation
- Multi-user scenarios

**Security Tests:**
- Access control validation
- Overflow/underflow protection
- Reentrancy attack prevention
- Economic exploit resistance

### Sample Test

```rust
#[tokio::test]
async fn test_staking_rewards_calculation() {
    let mut context = program_test().start_with_context().await;
    
    // Setup test environment
    let user = Keypair::new();
    let stake_amount = 1000 * 10_u64.pow(9); // 1000 FIN
    
    // Execute staking
    let result = stake_tokens(
        &mut context,
        &user,
        stake_amount,
        StakingTier::Gold,
        None
    ).await;
    
    assert!(result.is_ok());
    
    // Advance time to accumulate rewards
    context.warp_to_slot(1000).unwrap();
    
    // Verify reward calculation
    let rewards = calculate_staking_rewards(&context, &user).await.unwrap();
    let expected_rewards = calculate_expected_rewards(stake_amount, StakingTier::Gold, 1000);
    
    assert_eq!(rewards, expected_rewards);
}
```

## 🚀 Deployment

### Deployment Checklist

- [ ] Security audit completed
- [ ] All tests passing
- [ ] Economic parameters validated
- [ ] Emergency procedures tested
- [ ] Documentation updated
- [ ] Monitoring configured

### Network Deployment

```bash
# Deploy to devnet
anchor deploy --provider.cluster devnet

# Deploy to testnet
anchor deploy --provider.cluster testnet

# Deploy to mainnet (requires additional verification)
anchor deploy --provider.cluster mainnet-beta
```

### Post-Deployment Setup

```typescript
// Initialize token mints
await initializeFinMint();
await initializeSFinMint();
await initializeUSDFinMint();
await initializeSUSDFinMint();

// Setup reward pools
await createStakingRewardPool();
await createMiningRewardPool();
await createReferralRewardPool();

// Configure economic parameters
await setEconomicParameters({
  baseMininingRate: 0.05,
  regressionFactor: 0.001,
  stakingMultipliers: [1.2, 1.35, 1.5, 1.75, 2.0]
});
```

## 📚 API Reference

### Client SDK Methods

#### Token Operations

```typescript
class FinovaTokenClient {
  // Mint new tokens (authority only)
  async mintTokens(params: MintTokensParams): Promise<TransactionSignature>
  
  // Burn tokens
  async burnTokens(params: BurnTokensParams): Promise<TransactionSignature>
  
  // Transfer tokens with fees
  async transferTokens(params: TransferParams): Promise<TransactionSignature>
}
```

#### Staking Operations

```typescript
interface StakingOperations {
  // Stake FIN tokens for sFIN
  async stakeTokens(params: StakeParams): Promise<TransactionSignature>
  
  // Unstake sFIN for FIN + rewards
  async unstakeTokens(params: UnstakeParams): Promise<TransactionSignature>
  
  // Claim accumulated rewards
  async claimRewards(account: PublicKey): Promise<TransactionSignature>
  
  // Get staking information
  async getStakeInfo(account: PublicKey): Promise<StakeInfo>
}
```

#### Analytics Operations

```typescript
interface AnalyticsOperations {
  // Get user token portfolio
  async getUserPortfolio(user: PublicKey): Promise<TokenPortfolio>
  
  // Get network statistics
  async getNetworkStats(): Promise<NetworkStats>
  
  // Calculate potential rewards
  async calculateRewards(params: RewardCalculationParams): Promise<RewardProjection>
}
```

## 💰 Economic Model

### Token Distribution

```
Total $FIN Supply: 100 Billion Tokens

Distribution:
├── Community Mining: 50B (50%)
├── Team & Development: 20B (20%)
├── Investors: 15B (15%)
├── Public Sale: 10B (10%)
└── Treasury Reserve: 5B (5%)
```

### Mining Economics

```typescript
// Base mining rate calculation
function calculateMiningRate(phase: MiningPhase, users: number): number {
  const baseRates = {
    finizen: 0.1,    // 0-100K users
    growth: 0.05,    // 100K-1M users  
    maturity: 0.025, // 1M-10M users
    stability: 0.01  // 10M+ users
  };
  
  const pioneerBonus = Math.max(1.0, 2.0 - (users / 1_000_000));
  return baseRates[phase] * pioneerBonus;
}
```

### Staking Economics

```typescript
// Staking reward calculation
function calculateStakingReward(
  stakedAmount: number,
  totalStaked: number,
  poolRewards: number,
  multipliers: MultiplierEffects
): number {
  const baseReward = (stakedAmount / totalStaked) * poolRewards;
  const xpBonus = 1 + (multipliers.xpLevel / 100);
  const rpBonus = 1 + (multipliers.rpTier * 0.2); 
  const loyaltyBonus = 1 + (multipliers.stakingMonths * 0.05);
  
  return baseReward * xpBonus * rpBonus * loyaltyBonus;
}
```

## 🔧 Integration Examples

### Mining Reward Distribution

```typescript
// Integrate with Finova Core program for mining rewards
async function distributeMiningRewards(
  user: PublicKey,
  miningData: MiningCalculation
) {
  const client = new FinovaTokenClient(connection, wallet);
  
  // Calculate final reward amount
  const baseReward = miningData.baseMininingRate;
  const xpMultiplier = miningData.xpLevel * 0.1 + 1;
  const rpMultiplier = miningData.rpTier * 0.2 + 1;
  const regressionFactor = Math.exp(-0.001 * miningData.totalHoldings);
  
  const finalReward = baseReward * xpMultiplier * rpMultiplier * regressionFactor;
  
  // Mint tokens to user
  await client.mintTokens({
    mint: FIN_MINT,
    to: await getAssociatedTokenAddress(FIN_MINT, user),
    amount: finalReward * 10**9,
    mintReason: 'mining_reward',
    recipientData: {
      user,
      xpLevel: miningData.xpLevel,
      rpTier: miningData.rpTier,
      qualityScore: miningData.qualityScore
    }
  });
}
```

### Social Media Integration

```typescript
// Reward social media activity
async function rewardSocialActivity(
  user: PublicKey,
  activity: SocialActivity
) {
  const rewards = calculateSocialRewards(activity);
  
  // Mint FIN tokens for the activity
  await client.mintTokens({
    mint: FIN_MINT,
    to: userTokenAccount,
    amount: rewards.finAmount,
    mintReason: 'social_activity',
    recipientData: {
      platform: activity.platform,
      activityType: activity.type,
      engagementScore: activity.engagement
    }
  });
  
  // Update user's XP (handled by core program)
  await coreProgram.updateXP(user, rewards.xpGain);
}
```

### E-wallet Integration

```typescript
// Convert FIN to USDfin for e-wallet usage
async function convertForEWallet(
  user: PublicKey,
  finAmount: number
) {
  const currentPrice = await getFinPrice();
  const usdFinAmount = finAmount * currentPrice;
  
  // Burn FIN tokens
  await client.burnTokens({
    mint: FIN_MINT,
    from: userFinAccount,
    amount: finAmount * 10**9,
    burnReason: 'currency_conversion'
  });
  
  // Mint equivalent USDfin
  await client.mintTokens({
    mint: USDFIN_MINT,
    to: userUSDFinAccount,
    amount: usdFinAmount * 10**9,
    mintReason: 'currency_conversion'
  });
  
  // Update e-wallet balance
  await updateEWalletBalance(user, usdFinAmount);
}
```

## 🚨 Troubleshooting

### Common Issues

**Issue: Transaction Failed - Insufficient Authority**
```bash
Error: Custom program error: 0x1001
```
**Solution:** Ensure the signing wallet has the required authority for the operation.

**Issue: Staking Calculation Mismatch**
```bash
Error: Reward calculation overflow
```
**Solution:** Check that all multipliers are within expected ranges and total staked amount is valid.

**Issue: Token Account Not Found**
```bash
Error: Account not found
```
**Solution:** Ensure associated token accounts are created before attempting operations.

### Debug Mode

Enable debug logging:

```rust
// In lib.rs
#[cfg(feature = "debug")]
msg!("Debug: {}", debug_info);
```

```bash
# Build with debug features
anchor build -- --features debug
```

### Common Error Codes

| Code | Description | Solution |
|------|-------------|----------|
| 6000 | Insufficient Balance | Check user token balance |
| 6001 | Invalid Authority | Verify signing authority |
| 6002 | Staking Period Not Met | Wait for lock period completion |
| 6003 | Reward Pool Empty | Contact administrators |
| 6004 | Economic Parameters Invalid | Check parameter ranges |

## 🤝 Contributing

We welcome contributions from the community! Please follow these guidelines:

### Development Setup

```bash
# Fork and clone the repository
git clone https://github.com/your-username/finova-contracts.git
cd finova-contracts

# Create a new branch
git checkout -b feature/your-feature-name

# Install dependencies
yarn install
anchor build

# Run tests
anchor test
```

### Code Standards

- **Rust**: Follow standard Rust formatting with `rustfmt`
- **TypeScript**: Use Prettier and ESLint configurations
- **Documentation**: All public functions must have documentation
- **Testing**: Minimum 80% test coverage required

### Pull Request Process

1. **Create Feature Branch**: Use descriptive branch names
2. **Write Tests**: Ensure new functionality is tested
3. **Update Documentation**: Keep README and docs current
4. **Security Review**: Consider security implications
5. **Performance Testing**: Validate gas usage and performance

### Code Review Checklist

- [ ] All tests passing
- [ ] Security considerations addressed
- [ ] Gas optimization implemented
- [ ] Documentation updated
- [ ] Integration tested
- [ ] Error handling comprehensive

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](../../LICENSE) file for details.

## 🔗 Links

- **Website**: [https://finova.network](https://finova.network)
- **Documentation**: [https://docs.finova.network](https://docs.finova.network)
- **GitHub**: [https://github.com/finova-network/finova-contracts](https://github.com/finova-network/finova-contracts)
- **Discord**: [https://discord.gg/finova](https://discord.gg/finova)
- **Telegram**: [https://t.me/finovanetwork](https://t.me/finovanetwork)

## 📞 Support

For technical support and questions:

- **Technical Issues**: Create an issue on GitHub
- **General Questions**: Join our Discord community  
- **Security Concerns**: Email security@finova.network
- **Partnership Inquiries**: Email partnerships@finova.network

---

**⚠️ Important Notice**: This software is provided "as is" without warranty. Users are responsible for conducting their own security audits before deploying to mainnet. Always test thoroughly on devnet/testnet before production deployment.

**🔒 Security First**: Report security vulnerabilities responsibly through our bug bounty program. Do not disclose vulnerabilities publicly until they have been addressed.

---

*Built with ❤️ by the Finova Network team*