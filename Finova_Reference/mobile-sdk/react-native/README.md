# Finova Network React Native SDK

[![npm version](https://badge.fury.io/js/finova-react-native-sdk.svg)](https://badge.fury.io/js/finova-react-native-sdk)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Platform](https://img.shields.io/badge/platform-iOS%20%7C%20Android-blue.svg)](https://reactnative.dev/)

The official React Native SDK for Finova Network - the next-generation Social-Fi Super App that integrates XP, RP, and $FIN mining systems.

## 🚀 Features

- **Triple Reward Integration**: XP, RP, and $FIN mining in one SDK
- **Social Media Mining**: Earn rewards from Instagram, TikTok, YouTube activities
- **Real-time Mining**: Live $FIN generation with exponential regression
- **NFT & Special Cards**: Complete marketplace integration
- **Anti-Bot Protection**: AI-powered security validation
- **E-wallet Integration**: IDR payment gateway support
- **Guild System**: Community building and competitions

## 📱 Installation

```bash
npm install finova-react-native-sdk
# or
yarn add finova-react-native-sdk
```

### iOS Setup
```bash
cd ios && pod install
```

### Android Setup
Add to `android/app/build.gradle`:
```gradle
dependencies {
    implementation project(':finova-react-native-sdk')
}
```

## ⚡ Quick Start

### 1. Initialize SDK

```typescript
import FinovaSDK from 'finova-react-native-sdk';

// Initialize in App.tsx
const finovaConfig = {
  apiUrl: 'https://api.finova.network',
  environment: 'production', // or 'testnet'
  apiKey: 'your-api-key'
};

FinovaSDK.initialize(finovaConfig);
```

### 2. User Authentication

```typescript
// Login with wallet
const loginResult = await FinovaSDK.auth.loginWithWallet(walletAddress);

// Login with social
const socialLogin = await FinovaSDK.auth.loginWithSocial({
  provider: 'google',
  token: 'oauth-token'
});

// KYC verification
const kycResult = await FinovaSDK.auth.submitKYC({
  idType: 'ktp',
  idNumber: '1234567890123456',
  selfieImage: 'base64-image',
  idImage: 'base64-image'
});
```

### 3. Mining Operations

```typescript
// Start mining
const miningStatus = await FinovaSDK.mining.start();

// Get mining rate
const rate = await FinovaSDK.mining.getCurrentRate();
console.log(`Mining at: ${rate.ratePerHour} $FIN/hour`);

// Claim rewards
const claimed = await FinovaSDK.mining.claimRewards();

// Get mining stats
const stats = await FinovaSDK.mining.getStats();
```

### 4. XP System Integration

```typescript
// Track social activity
const xpGain = await FinovaSDK.xp.trackActivity({
  platform: 'instagram',
  activityType: 'post',
  contentId: 'post-123',
  metadata: {
    views: 1500,
    likes: 200,
    comments: 25
  }
});

// Get XP stats
const xpStats = await FinovaSDK.xp.getStats();
console.log(`Level: ${xpStats.level}, XP: ${xpStats.totalXP}`);

// Level progression
const progression = await FinovaSDK.xp.getLevelProgression();
```

### 5. Referral System

```typescript
// Get referral code
const referralCode = await FinovaSDK.referral.getReferralCode();

// Track referral
const referralResult = await FinovaSDK.referral.trackReferral({
  referralCode: 'ABC123',
  referredUserId: 'user-456'
});

// Get referral stats
const rpStats = await FinovaSDK.referral.getStats();
console.log(`RP Tier: ${rpStats.tier}, Total RP: ${rpStats.totalRP}`);
```

## 🎮 Advanced Features

### NFT & Special Cards

```typescript
// Get user's NFTs
const nfts = await FinovaSDK.nft.getUserNFTs();

// Use special card
const cardUse = await FinovaSDK.nft.useSpecialCard({
  cardId: 'card-123',
  cardType: 'double_mining'
});

// Buy NFT from marketplace
const purchase = await FinovaSDK.nft.buyNFT({
  nftId: 'nft-456',
  price: 100, // in $FIN
  paymentMethod: 'fin_balance'
});
```

### Staking Integration

```typescript
// Stake $FIN tokens
const stakeResult = await FinovaSDK.staking.stake({
  amount: 1000,
  duration: 'flexible' // or '30_days', '90_days', '365_days'
});

// Get staking rewards
const rewards = await FinovaSDK.staking.getRewards();

// Unstake tokens
const unstake = await FinovaSDK.staking.unstake({
  stakeId: 'stake-123',
  amount: 500
});
```

### Guild System

```typescript
// Join guild
const guildJoin = await FinovaSDK.guild.join('guild-id-123');

// Get guild info
const guildInfo = await FinovaSDK.guild.getInfo('guild-id-123');

// Participate in guild competition
const compete = await FinovaSDK.guild.participateInCompetition({
  competitionId: 'comp-456',
  activityData: { /* competition specific data */ }
});
```

## 🔒 Security & Anti-Bot

### Biometric Verification

```typescript
// Initialize biometric check
const biometricInit = await FinovaSDK.security.initBiometric();

// Perform periodic verification
const verified = await FinovaSDK.security.verifyBiometric();

// Report suspicious activity
await FinovaSDK.security.reportSuspiciousActivity({
  type: 'unusual_pattern',
  description: 'Detected automated behavior'
});
```

### Human Verification

```typescript
// Get human verification challenge
const challenge = await FinovaSDK.security.getHumanChallenge();

// Submit challenge response
const verification = await FinovaSDK.security.submitChallengeResponse({
  challengeId: challenge.id,
  response: 'user-response'
});
```

## 💰 E-wallet Integration

```typescript
// Initialize e-wallet
const walletInit = await FinovaSDK.ewallet.initialize({
  provider: 'ovo', // 'gopay', 'dana', 'shopeepay'
  userId: 'user-123'
});

// Convert $FIN to IDR
const conversion = await FinovaSDK.ewallet.convertToIDR({
  finAmount: 100,
  targetWallet: 'ovo'
});

// Withdraw to e-wallet
const withdrawal = await FinovaSDK.ewallet.withdraw({
  amount: 50000, // IDR
  walletAddress: 'ovo-account-123'
});
```

## 📊 Real-time Updates

### WebSocket Integration

```typescript
// Connect to real-time updates
FinovaSDK.realtime.connect();

// Listen to mining updates
FinovaSDK.realtime.onMiningUpdate((data) => {
  console.log('Mining update:', data);
});

// Listen to XP changes
FinovaSDK.realtime.onXPUpdate((xpData) => {
  console.log('XP gained:', xpData.amount);
});

// Listen to referral activities
FinovaSDK.realtime.onReferralUpdate((referralData) => {
  console.log('Referral activity:', referralData);
});

// Disconnect
FinovaSDK.realtime.disconnect();
```

## 🛠️ Configuration

### Environment Configuration

```typescript
interface FinovaConfig {
  apiUrl: string;
  environment: 'development' | 'testnet' | 'production';
  apiKey: string;
  enableLogging?: boolean;
  enableAnalytics?: boolean;
  socketUrl?: string;
  socialIntegrations?: {
    instagram?: boolean;
    tiktok?: boolean;
    youtube?: boolean;
    facebook?: boolean;
    twitter?: boolean;
  };
}
```

### Platform-specific Settings

```typescript
// iOS specific
const iosConfig = {
  biometricType: 'touchid', // or 'faceid'
  keychainService: 'com.finova.app',
  backgroundMining: true
};

// Android specific  
const androidConfig = {
  biometricPrompt: 'Verify your identity',
  keystoreAlias: 'finova_keys',
  foregroundService: true
};

FinovaSDK.configurePlatform(Platform.OS === 'ios' ? iosConfig : androidConfig);
```

## 🧪 Testing

### Mock Mode

```typescript
// Enable mock mode for testing
FinovaSDK.enableMockMode({
  mockMiningRate: 0.1,
  mockUserLevel: 25,
  mockRPTier: 'influencer'
});

// Run integration tests
const testResult = await FinovaSDK.runDiagnostics();
console.log('SDK Status:', testResult);
```

## 🔧 Error Handling

```typescript
import { FinovaError, ErrorCodes } from 'finova-react-native-sdk';

try {
  await FinovaSDK.mining.start();
} catch (error) {
  if (error instanceof FinovaError) {
    switch (error.code) {
      case ErrorCodes.INSUFFICIENT_BALANCE:
        // Handle insufficient balance
        break;
      case ErrorCodes.RATE_LIMITED:
        // Handle rate limiting
        break;
      case ErrorCodes.NETWORK_ERROR:
        // Handle network issues
        break;
      default:
        // Handle other errors
        break;
    }
  }
}
```

## 🎯 Best Practices

### Performance Optimization

```typescript
// Batch operations
const batchOperations = await FinovaSDK.batch([
  { method: 'mining.getStats' },
  { method: 'xp.getStats' },
  { method: 'referral.getStats' }
]);

// Cache management
FinovaSDK.cache.setTTL('user_stats', 300); // 5 minutes
const cachedStats = await FinovaSDK.cache.get('user_stats');
```

### Memory Management

```typescript
// Cleanup when component unmounts
useEffect(() => {
  return () => {
    FinovaSDK.cleanup();
    FinovaSDK.realtime.disconnect();
  };
}, []);
```

## 📈 Analytics & Monitoring

```typescript
// Track custom events
FinovaSDK.analytics.track('user_action', {
  action: 'card_purchase',
  cardType: 'double_mining',
  price: 50
});

// Monitor performance
const metrics = await FinovaSDK.monitoring.getMetrics();
console.log('SDK Performance:', metrics);
```

## 🌐 Multi-language Support

```typescript
// Set language
FinovaSDK.setLanguage('id'); // Indonesian
FinovaSDK.setLanguage('en'); // English

// Get localized strings
const strings = FinovaSDK.getLocalizedStrings();
```

## 🤝 Community Integration

### Social Sharing

```typescript
// Share achievement
const shareResult = await FinovaSDK.social.share({
  type: 'achievement',
  data: {
    achievement: 'level_50',
    platform: 'instagram'
  }
});

// Generate shareable content
const content = await FinovaSDK.social.generateShareContent({
  type: 'mining_stats',
  customMessage: 'Check out my Finova mining progress!'
});
```

## 📋 API Reference

### Core Modules

- `FinovaSDK.auth` - Authentication & KYC
- `FinovaSDK.mining` - Mining operations
- `FinovaSDK.xp` - Experience points system
- `FinovaSDK.referral` - Referral program
- `FinovaSDK.nft` - NFT marketplace
- `FinovaSDK.staking` - Token staking
- `FinovaSDK.guild` - Guild system
- `FinovaSDK.ewallet` - E-wallet integration
- `FinovaSDK.security` - Security & verification
- `FinovaSDK.realtime` - WebSocket updates

## 🐛 Troubleshooting

### Common Issues

**Mining not starting:**
```typescript
// Check mining prerequisites
const prereqs = await FinovaSDK.mining.checkPrerequisites();
if (!prereqs.kycVerified) {
  // Prompt KYC verification
}
```

**Network connectivity:**
```typescript
// Test network connection
const networkStatus = await FinovaSDK.network.testConnectivity();
if (!networkStatus.connected) {
  // Handle offline mode
}
```

## 📞 Support

- **Documentation**: [https://docs.finova.network](https://docs.finova.network)
- **Discord**: [https://discord.gg/finova](https://discord.gg/finova)
- **Telegram**: [@FinovaNetwork](https://t.me/FinovaNetwork)
- **Email**: sdk-support@finova.network

## 📄 License

MIT License - see [LICENSE](LICENSE) file for details.

## 🚀 Contributing

We welcome contributions! Please see our [Contributing Guide](CONTRIBUTING.md) for details.

---

**Finova Network** - Where Every Interaction Has Value
*Start mining today. Build your network. Earn while you engage.*