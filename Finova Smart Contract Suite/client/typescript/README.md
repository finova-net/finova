# Finova Network - TypeScript SDK

[![npm version](https://badge.fury.io/js/@finova/sdk.svg)](https://badge.fury.io/js/@finova/sdk)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Build Status](https://github.com/finova-network/finova-contracts/workflows/CI/badge.svg)](https://github.com/finova-network/finova-contracts/actions)

The official TypeScript SDK for Finova Network - The next-generation Social-Fi Super App that integrates XP, RP, and $FIN mining systems.

## 🚀 Quick Start

```bash
npm install @finova/sdk
# or
yarn add @finova/sdk
```

```typescript
import { FinovaClient } from '@finova/sdk';

const client = new FinovaClient({
  network: 'mainnet', // 'devnet' | 'testnet' | 'mainnet'
  rpcUrl: 'https://api.mainnet-beta.solana.com',
  commitment: 'confirmed'
});

// Initialize user mining
const userAccount = await client.mining.initializeUser({
  referralCode: 'FINOVA123',
  kycVerified: true
});

// Start earning $FIN, XP, and RP!
const miningRate = await client.mining.getCurrentMiningRate(userAccount);
console.log(`Mining ${miningRate} $FIN/hour`);
```

## 📋 Features

- **🏭 Mining System**: Pi Network-inspired exponential regression mining
- **⭐ XP System**: Hamster Kombat-style gamified progression  
- **🔗 RP Network**: Multi-level referral rewards system
- **🎯 NFT Integration**: Special cards with utility bonuses
- **💰 Staking**: Liquid staking with enhanced rewards
- **🛡️ Anti-Bot**: AI-powered fraud detection
- **🌐 Multi-Platform**: Instagram, TikTok, YouTube, Facebook, Twitter/X

## 🏗️ Architecture

```
FinovaClient
├── mining        # Core mining operations
├── xp            # Experience points system
├── referral      # Referral network management
├── nft           # NFT marketplace & special cards
├── staking       # Liquid staking operations
├── social        # Social media integrations
└── governance    # DAO voting & proposals
```

## 🛠️ Core Usage

### Mining Operations

```typescript
// Initialize mining account
const miningAccount = await client.mining.initialize({
  user: userPublicKey,
  referralCode: 'FINOVA123'
});

// Get current mining rate with all multipliers
const rate = await client.mining.getMiningRate(userPublicKey);
// Returns: { baseFin: 0.05, xpMultiplier: 1.5, rpMultiplier: 2.0, totalRate: 0.15 }

// Claim accumulated rewards
const rewards = await client.mining.claimRewards(userPublicKey);
console.log(`Claimed: ${rewards.fin} $FIN, ${rewards.xp} XP, ${rewards.rp} RP`);

// Get mining statistics
const stats = await client.mining.getStats(userPublicKey);
/*
{
  totalMined: 1250.5,
  currentPhase: 2,
  networkSize: 750000,
  regressionFactor: 0.85,
  dailyCap: 2.4,
  timeUntilNextClaim: 3600
}
*/
```

### XP System Integration

```typescript
// Record social media activity
const xpGain = await client.xp.recordActivity({
  type: 'POST',
  platform: 'TIKTOK',
  content: 'Check out my latest dance video! #FinovaNetwork',
  engagementData: {
    views: 5000,
    likes: 250,
    comments: 45,
    shares: 12
  }
});

// Get XP breakdown
const xpData = await client.xp.getUserXP(userPublicKey);
/*
{
  totalXP: 15750,
  currentLevel: 35,
  levelProgress: 0.65,
  miningMultiplier: 2.8,
  nextLevelXP: 18000,
  badges: ['Silver V', 'Content Creator', 'Viral Master']
}
*/

// Calculate XP for activity
const calculation = client.xp.calculateXP({
  baseXP: 150,
  platform: 'TIKTOK', // 1.3x multiplier
  qualityScore: 2.0,   // Viral content
  streakBonus: 1.5,    // 7-day streak
  userLevel: 35        // Level progression factor
});
// Returns: 412 XP
```

### Referral Network Management

```typescript
// Generate referral code
const referralCode = await client.referral.generateCode(userPublicKey);
console.log(`Your code: ${referralCode.code}`); // e.g., "JOHN2024FIN"

// Get referral network stats
const network = await client.referral.getNetwork(userPublicKey);
/*
{
  tier: 'INFLUENCER',
  totalRP: 25750,
  directReferrals: 32,
  activeReferrals: 28,
  networkSize: 450,
  miningBonus: 1.5,
  qualityScore: 0.87,
  earnings: {
    daily: 15.6,
    weekly: 109.2,
    total: 2847.3
  }
}
*/

// Process referral signup
const referralReward = await client.referral.processSignup({
  referrer: referrerPublicKey,
  referee: refereePublicKey,
  referralCode: 'FINOVA123'
});
```

### NFT & Special Cards

```typescript
// Get user's NFT collection
const nfts = await client.nft.getUserCollection(userPublicKey);

// Use special card
const cardEffect = await client.nft.useSpecialCard({
  cardMint: cardPublicKey,
  cardType: 'DOUBLE_MINING',
  duration: 24 * 3600 // 24 hours
});

// Buy special card from marketplace
const purchase = await client.nft.buyCard({
  cardType: 'TRIPLE_MINING',
  rarity: 'RARE',
  paymentToken: '$FIN',
  amount: 150
});

// Get marketplace listings
const marketplace = await client.nft.getMarketplace({
  category: 'MINING_BOOST',
  priceRange: { min: 50, max: 500 },
  rarity: ['COMMON', 'RARE']
});
```

### Staking Operations

```typescript
// Stake $FIN tokens
const stakeResult = await client.staking.stake({
  amount: 1000,
  tier: 'GOLD' // Automatically calculated based on amount
});

// Get staking info
const stakingInfo = await client.staking.getStakingInfo(userPublicKey);
/*
{
  stakedAmount: 1000,
  sFINBalance: 1008.5,
  tier: 'GOLD',
  apy: 12,
  miningBoost: 1.5,
  xpMultiplier: 1.3,
  rpBonus: 1.2,
  rewards: {
    pending: 45.6,
    claimed: 234.8
  }
}
*/

// Claim staking rewards
const rewards = await client.staking.claimRewards(userPublicKey);
```

## 🔌 Social Media Integration

```typescript
// Connect social platforms
await client.social.connectPlatform({
  platform: 'INSTAGRAM',
  accessToken: 'your_instagram_token',
  permissions: ['read_posts', 'read_insights']
});

// Sync social activity (automated)
const syncResult = await client.social.syncActivity({
  platform: 'TIKTOK',
  timeRange: { start: '2025-07-01', end: '2025-07-30' }
});

// Get platform analytics
const analytics = await client.social.getAnalytics(userPublicKey);
/*
{
  platforms: {
    TIKTOK: { posts: 45, totalViews: 125000, avgEngagement: 0.08 },
    INSTAGRAM: { posts: 32, totalLikes: 8500, followers: 2100 }
  },
  totalXP: 15750,
  qualityScore: 0.89,
  viralContent: 3
}
*/
```

## ⚙️ Configuration

```typescript
const client = new FinovaClient({
  // Network settings
  network: 'mainnet',
  rpcUrl: 'https://api.mainnet-beta.solana.com',
  commitment: 'confirmed',
  
  // Program addresses (auto-detected by network)
  programs: {
    core: 'FiNoVa1111111111111111111111111111111111111',
    token: 'FiNoVaT0Ken111111111111111111111111111111',
    nft: 'FiNoVaNFT111111111111111111111111111111111',
    staking: 'FiNoVaStake11111111111111111111111111111'
  },
  
  // API endpoints
  apiEndpoints: {
    main: 'https://api.finova.network/v1',
    ai: 'https://ai.finova.network/v1',
    analytics: 'https://analytics.finova.network/v1'
  },
  
  // Security settings
  security: {
    enableAntiBot: true,
    humanVerificationRequired: true,
    maxDailyTransactions: 1000
  }
});
```

## 🧪 Advanced Examples

### Complete User Journey

```typescript
async function completeUserFlow() {
  // 1. Initialize new user
  const user = await client.mining.initializeUser({
    referralCode: 'FINOVA2025',
    kycData: {
      verified: true,
      level: 'BASIC',
      country: 'ID'
    }
  });
  
  // 2. Connect social platforms
  await client.social.connectPlatform({
    platform: 'TIKTOK',
    accessToken: process.env.TIKTOK_TOKEN
  });
  
  // 3. Record daily activities
  const activities = [
    { type: 'POST', platform: 'TIKTOK', content: 'Dancing video' },
    { type: 'COMMENT', platform: 'INSTAGRAM', content: 'Great post!' },
    { type: 'LIKE', platform: 'YOUTUBE', targetId: 'video123' }
  ];
  
  for (const activity of activities) {
    await client.xp.recordActivity(activity);
  }
  
  // 4. Use special card for boost
  await client.nft.useSpecialCard({
    cardType: 'DOUBLE_MINING',
    duration: 24 * 3600
  });
  
  // 5. Stake tokens for enhanced rewards
  await client.staking.stake({ amount: 500 });
  
  // 6. Check total rewards
  const summary = await client.getSummary(user.publicKey);
  console.log('Daily earnings:', summary);
}
```

### Guild Management

```typescript
// Create guild
const guild = await client.governance.createGuild({
  name: 'Finova Warriors',
  description: 'Elite mining guild',
  maxMembers: 50,
  requirements: {
    minLevel: 25,
    minStaked: 1000
  }
});

// Join guild competition
await client.governance.joinCompetition({
  guildId: guild.id,
  competitionType: 'WEEKLY_WAR',
  strategy: 'MINING_FOCUSED'
});

// Vote on proposal
await client.governance.vote({
  proposalId: 'PROP_001',
  choice: 'YES',
  weight: 1500 // Based on staked amount + XP + RP
});
```

## 🔒 Security Best Practices

```typescript
// Enable additional security
const secureClient = new FinovaClient({
  security: {
    enableAntiBot: true,
    humanVerificationRequired: true,
    encryptSensitiveData: true,
    auditTransactions: true
  }
});

// Verify human before high-value operations
const humanProof = await client.security.generateHumanProof();
await client.security.verifyHuman(humanProof);

// Monitor suspicious activity
client.on('suspiciousActivity', (event) => {
  console.log('Security alert:', event);
  // Implement additional verification
});
```

## 📊 Analytics & Monitoring

```typescript
// Get comprehensive analytics
const analytics = await client.analytics.getUserAnalytics(userPublicKey);
/*
{
  mining: { totalMined: 5420, currentRate: 0.15, efficiency: 0.87 },
  xp: { total: 25750, level: 45, weeklyGain: 1250 },
  referral: { network: 85, activeRate: 0.82, earnings: 340.5 },
  social: { platforms: 5, totalPosts: 127, avgQuality: 1.4 },
  nfts: { owned: 12, used: 45, marketplace: 3 },
  staking: { amount: 2500, rewards: 180.4, apy: 14.2 }
}
*/

// Performance monitoring
client.on('performanceMetrics', (metrics) => {
  console.log(`API Response Time: ${metrics.responseTime}ms`);
  console.log(`Success Rate: ${metrics.successRate}%`);
});
```

## 🐛 Error Handling

```typescript
try {
  await client.mining.claimRewards(userPublicKey);
} catch (error) {
  if (error.code === 'INSUFFICIENT_BALANCE') {
    console.log('Wait for more rewards to accumulate');
  } else if (error.code === 'RATE_LIMITED') {
    console.log('Too many requests, please wait');
  } else if (error.code === 'BOT_DETECTED') {
    console.log('Please complete human verification');
    await client.security.requestHumanVerification();
  }
}

// Global error handler
client.on('error', (error) => {
  console.error('Finova SDK Error:', error);
  // Implement retry logic or user notification
});
```

## 🔧 Development Setup

```bash
# Clone the repository
git clone https://github.com/finova-network/finova-contracts.git
cd finova-contracts/client/typescript

# Install dependencies
npm install

# Set environment variables
cp .env.example .env
# Edit .env with your configuration

# Run tests
npm test

# Build the project
npm run build

# Generate documentation
npm run docs
```

## 📚 API Reference

### Core Classes

- `FinovaClient` - Main SDK client
- `MiningService` - Mining operations
- `XPService` - Experience points system
- `ReferralService` - Referral network management
- `NFTService` - NFT marketplace operations
- `StakingService` - Staking and rewards
- `SocialService` - Social media integration
- `GovernanceService` - DAO and guild operations

### Types

```typescript
interface UserAccount {
  publicKey: PublicKey;
  miningRate: number;
  xpLevel: number;
  rpTier: string;
  stakingTier: string;
  nftCount: number;
  isKYCVerified: boolean;
}

interface MiningStats {
  totalMined: number;
  currentRate: number;
  dailyCap: number;
  phaseMultiplier: number;
  regressionFactor: number;
}

interface RewardCalculation {
  baseMining: number;
  xpMultiplier: number;
  rpMultiplier: number;
  qualityScore: number;
  finalRate: number;
}
```

## 🚀 Performance Optimization

- **Connection Pooling**: Automatic RPC connection management
- **Caching**: Intelligent caching of account data and calculations
- **Batch Operations**: Combine multiple transactions
- **Retry Logic**: Automatic retry with exponential backoff
- **Rate Limiting**: Built-in protection against API limits

## 🌍 Internationalization

```typescript
import { FinovaClient } from '@finova/sdk';
import { i18n } from '@finova/sdk/i18n';

// Set language
i18n.setLanguage('id'); // Indonesian
i18n.setLanguage('en'); // English (default)

// Localized messages
console.log(i18n.t('mining.success')); // "Mining berhasil dimulai!"
```

## 📈 Migration Guide

### From v1.x to v2.x

```typescript
// Old v1.x way
const mining = new FinovaMining();
const rate = await mining.getRate(user);

// New v2.x way
const client = new FinovaClient();
const rate = await client.mining.getMiningRate(user);
```

## 🤝 Contributing

1. Fork the repository
2. Create feature branch: `git checkout -b feature/amazing-feature`
3. Commit changes: `git commit -m 'Add amazing feature'`
4. Push to branch: `git push origin feature/amazing-feature`
5. Open Pull Request

## 📄 License

MIT License - see [LICENSE](../../../LICENSE) file for details.

## 🆘 Support

- **Documentation**: [docs.finova.network](https://docs.finova.network)
- **Discord**: [discord.gg/finova](https://discord.gg/finova)
- **Telegram**: [@finova_network](https://t.me/finova_network)
- **Email**: support@finova.network

## 🎯 Roadmap

- [ ] WebSocket real-time updates
- [ ] React Native SDK bridge
- [ ] GraphQL API integration
- [ ] Advanced analytics dashboard
- [ ] Multi-chain support
- [ ] Enterprise API features

---

**Built with ❤️ by the Finova Network team**

*Engage & Earn - Transform your social presence into measurable value*
