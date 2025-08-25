# Finova Network Android SDK

[![Maven Central](https://img.shields.io/maven-central/v/com.finova/finova-sdk)](https://search.maven.org/artifact/com.finova/finova-sdk)
[![License](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![API](https://img.shields.io/badge/API-21%2B-brightgreen.svg?style=flat)](https://android-arsenal.com/api?level=21)

The official Android SDK for Finova Network - the next-generation Social-Fi Super App that integrates XP, RP, and $FIN mining rewards.

## Features

- 🚀 **Triple Reward System**: XP, RP, and $FIN mining integration
- ⛏️ **Exponential Regression Mining**: Pi Network-inspired fair distribution
- 🎮 **Gamified Experience**: Hamster Kombat-style progression
- 🔗 **Social Media Integration**: Instagram, TikTok, YouTube, Facebook, X
- 💎 **NFT & Special Cards**: Collectible utility cards
- 👥 **Referral Network**: Multi-level RP system
- 🏛️ **Staking & Governance**: Enhanced rewards and DAO participation
- 🔐 **Anti-Bot Protection**: AI-powered fraud detection
- 🌐 **Real-time Sync**: WebSocket-based live updates

## Installation

### Gradle (Recommended)

Add to your app-level `build.gradle`:

```kotlin
dependencies {
    implementation 'com.finova:finova-sdk:1.0.0'
    implementation 'org.jetbrains.kotlinx:kotlinx-coroutines-android:1.7.3'
    implementation 'com.squareup.retrofit2:retrofit:2.9.0'
    implementation 'com.squareup.retrofit2:converter-gson:2.9.0'
    implementation 'com.squareup.okhttp3:logging-interceptor:4.11.0'
}
```

### Permissions

Add to `AndroidManifest.xml`:

```xml
<uses-permission android:name="android.permission.INTERNET" />
<uses-permission android:name="android.permission.ACCESS_NETWORK_STATE" />
<uses-permission android:name="android.permission.CAMERA" />
<uses-permission android:name="android.permission.WRITE_EXTERNAL_STORAGE" />
```

## Quick Start

### 1. Initialize SDK

```kotlin
class MyApplication : Application() {
    override fun onCreate() {
        super.onCreate()
        
        FinovaSDK.initialize(
            context = this,
            config = FinovaConfig(
                apiKey = "your_api_key",
                environment = FinovaEnvironment.PRODUCTION,
                enableLogging = BuildConfig.DEBUG
            )
        )
    }
}
```

### 2. User Authentication

```kotlin
class MainActivity : AppCompatActivity() {
    private lateinit var finovaClient: FinovaClient
    
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        
        finovaClient = FinovaSDK.getClient()
        
        // Wallet-based authentication
        authenticateUser()
    }
    
    private suspend fun authenticateUser() {
        try {
            val authResult = finovaClient.authenticate(
                walletAddress = "user_wallet_address",
                signature = "signed_message"
            )
            
            when (authResult) {
                is AuthResult.Success -> {
                    // User authenticated successfully
                    startMining()
                    setupRealTimeUpdates()
                }
                is AuthResult.Error -> {
                    // Handle authentication error
                    showError(authResult.message)
                }
            }
        } catch (e: Exception) {
            Log.e("Auth", "Authentication failed", e)
        }
    }
}
```

### 3. Mining Operations

```kotlin
class MiningService {
    private val miningService = FinovaSDK.getMiningService()
    
    suspend fun startMining() {
        try {
            val miningResult = miningService.startMining()
            
            when (miningResult) {
                is MiningResult.Success -> {
                    Log.d("Mining", "Mining rate: ${miningResult.data.hourlyRate} $FIN/hour")
                    Log.d("Mining", "XP Multiplier: ${miningResult.data.xpMultiplier}x")
                    Log.d("Mining", "RP Bonus: ${miningResult.data.rpBonus}x")
                }
                is MiningResult.Error -> {
                    Log.e("Mining", "Mining failed: ${miningResult.message}")
                }
            }
        } catch (e: Exception) {
            Log.e("Mining", "Mining error", e)
        }
    }
    
    suspend fun claimMiningRewards() {
        try {
            val claimResult = miningService.claimRewards()
            
            when (claimResult) {
                is ClaimResult.Success -> {
                    val rewards = claimResult.data
                    Log.d("Claim", "Claimed ${rewards.finAmount} $FIN")
                    Log.d("Claim", "Gained ${rewards.xpGained} XP")
                }
                is ClaimResult.Error -> {
                    Log.e("Claim", "Claim failed: ${claimResult.message}")
                }
            }
        } catch (e: Exception) {
            Log.e("Claim", "Claim error", e)
        }
    }
}
```

### 4. XP System Integration

```kotlin
class XPManager {
    private val xpService = FinovaSDK.getXPService()
    
    suspend fun recordSocialActivity(activity: SocialActivity) {
        try {
            val xpResult = xpService.recordActivity(
                SocialActivityRequest(
                    platform = activity.platform,
                    activityType = activity.type,
                    contentUrl = activity.url,
                    engagement = activity.engagement,
                    qualityMetrics = activity.qualityMetrics
                )
            )
            
            when (xpResult) {
                is XPResult.Success -> {
                    val xpData = xpResult.data
                    Log.d("XP", "Gained ${xpData.xpGained} XP")
                    Log.d("XP", "Quality Score: ${xpData.qualityScore}")
                    Log.d("XP", "New Level: ${xpData.newLevel}")
                    
                    // Update UI
                    updateXPDisplay(xpData)
                }
                is XPResult.Error -> {
                    Log.e("XP", "XP recording failed: ${xpResult.message}")
                }
            }
        } catch (e: Exception) {
            Log.e("XP", "XP error", e)
        }
    }
    
    suspend fun getUserXPStats(): UserXPStats? {
        return try {
            when (val result = xpService.getUserStats()) {
                is XPStatsResult.Success -> result.data
                is XPStatsResult.Error -> {
                    Log.e("XP", "Failed to get XP stats: ${result.message}")
                    null
                }
            }
        } catch (e: Exception) {
            Log.e("XP", "XP stats error", e)
            null
        }
    }
}
```

### 5. Referral System

```kotlin
class ReferralManager {
    private val referralService = FinovaSDK.getReferralService()
    
    suspend fun generateReferralCode(): String? {
        return try {
            when (val result = referralService.generateReferralCode()) {
                is ReferralCodeResult.Success -> result.data.code
                is ReferralCodeResult.Error -> {
                    Log.e("Referral", "Failed to generate code: ${result.message}")
                    null
                }
            }
        } catch (e: Exception) {
            Log.e("Referral", "Referral code error", e)
            null
        }
    }
    
    suspend fun useReferralCode(code: String): Boolean {
        return try {
            when (val result = referralService.useReferralCode(code)) {
                is ReferralUseResult.Success -> {
                    Log.d("Referral", "Successfully used referral code")
                    Log.d("Referral", "RP Bonus: ${result.data.rpBonus}")
                    true
                }
                is ReferralUseResult.Error -> {
                    Log.e("Referral", "Failed to use code: ${result.message}")
                    false
                }
            }
        } catch (e: Exception) {
            Log.e("Referral", "Referral use error", e)
            false
        }
    }
    
    suspend fun getReferralNetwork(): ReferralNetwork? {
        return try {
            when (val result = referralService.getNetwork()) {
                is NetworkResult.Success -> result.data
                is NetworkResult.Error -> {
                    Log.e("Referral", "Failed to get network: ${result.message}")
                    null
                }
            }
        } catch (e: Exception) {
            Log.e("Referral", "Network error", e)
            null
        }
    }
}
```

### 6. NFT & Special Cards

```kotlin
class NFTManager {
    private val nftService = FinovaSDK.getNFTService()
    
    suspend fun getUserNFTs(): List<NFTCard> {
        return try {
            when (val result = nftService.getUserNFTs()) {
                is NFTResult.Success -> result.data
                is NFTResult.Error -> {
                    Log.e("NFT", "Failed to get NFTs: ${result.message}")
                    emptyList()
                }
            }
        } catch (e: Exception) {
            Log.e("NFT", "NFT error", e)
            emptyList()
        }
    }
    
    suspend fun useSpecialCard(cardId: String): Boolean {
        return try {
            when (val result = nftService.useCard(cardId)) {
                is CardUseResult.Success -> {
                    Log.d("NFT", "Card used successfully")
                    Log.d("NFT", "Effect: ${result.data.effect}")
                    Log.d("NFT", "Duration: ${result.data.duration}")
                    true
                }
                is CardUseResult.Error -> {
                    Log.e("NFT", "Failed to use card: ${result.message}")
                    false
                }
            }
        } catch (e: Exception) {
            Log.e("NFT", "Card use error", e)
            false
        }
    }
    
    suspend fun purchaseNFT(nftId: String, price: Double): Boolean {
        return try {
            when (val result = nftService.purchaseNFT(nftId, price)) {
                is PurchaseResult.Success -> {
                    Log.d("NFT", "NFT purchased successfully")
                    true
                }
                is PurchaseResult.Error -> {
                    Log.e("NFT", "Purchase failed: ${result.message}")
                    false
                }
            }
        } catch (e: Exception) {
            Log.e("NFT", "Purchase error", e)
            false
        }
    }
}
```

## Real-Time Updates

### WebSocket Integration

```kotlin
class RealtimeManager {
    private lateinit var webSocketClient: WebSocketClient
    
    fun startRealtimeUpdates() {
        webSocketClient = FinovaSDK.getWebSocketClient()
        
        webSocketClient.connect(object : WebSocketListener {
            override fun onMiningUpdate(update: MiningUpdate) {
                // Handle mining rate changes
                Log.d("Realtime", "New mining rate: ${update.newRate}")
                updateMiningUI(update)
            }
            
            override fun onXPGained(xpUpdate: XPUpdate) {
                // Handle XP gains
                Log.d("Realtime", "XP gained: ${xpUpdate.amount}")
                showXPAnimation(xpUpdate)
            }
            
            override fun onReferralActivity(referralUpdate: ReferralUpdate) {
                // Handle referral network changes
                Log.d("Realtime", "Referral activity: ${referralUpdate.type}")
                updateReferralStats(referralUpdate)
            }
            
            override fun onCardActivated(cardUpdate: CardUpdate) {
                // Handle special card effects
                Log.d("Realtime", "Card activated: ${cardUpdate.cardName}")
                showCardEffect(cardUpdate)
            }
            
            override fun onError(error: WebSocketError) {
                Log.e("Realtime", "WebSocket error: ${error.message}")
            }
        })
    }
    
    fun stopRealtimeUpdates() {
        webSocketClient.disconnect()
    }
}
```

## Social Media Integration

### Activity Recording

```kotlin
class SocialIntegrationManager {
    private val socialService = FinovaSDK.getSocialService()
    
    // Instagram Integration
    suspend fun recordInstagramPost(postUrl: String, metrics: EngagementMetrics) {
        val activity = SocialActivity(
            platform = SocialPlatform.INSTAGRAM,
            type = ActivityType.POST,
            url = postUrl,
            engagement = metrics,
            timestamp = System.currentTimeMillis()
        )
        
        recordActivity(activity)
    }
    
    // TikTok Integration
    suspend fun recordTikTokVideo(videoUrl: String, views: Int, likes: Int) {
        val activity = SocialActivity(
            platform = SocialPlatform.TIKTOK,
            type = ActivityType.VIDEO,
            url = videoUrl,
            engagement = EngagementMetrics(views, likes, 0, 0),
            timestamp = System.currentTimeMillis()
        )
        
        recordActivity(activity)
    }
    
    private suspend fun recordActivity(activity: SocialActivity) {
        try {
            when (val result = socialService.recordActivity(activity)) {
                is SocialResult.Success -> {
                    Log.d("Social", "Activity recorded: ${result.data.xpGained} XP gained")
                }
                is SocialResult.Error -> {
                    Log.e("Social", "Failed to record activity: ${result.message}")
                }
            }
        } catch (e: Exception) {
            Log.e("Social", "Social activity error", e)
        }
    }
}
```

## Staking Operations

```kotlin
class StakingManager {
    private val stakingService = FinovaSDK.getStakingService()
    
    suspend fun stakeTokens(amount: Double, duration: StakingDuration): Boolean {
        return try {
            when (val result = stakingService.stake(amount, duration)) {
                is StakingResult.Success -> {
                    val stakeInfo = result.data
                    Log.d("Staking", "Staked ${stakeInfo.amount} $FIN")
                    Log.d("Staking", "APY: ${stakeInfo.apy}%")
                    Log.d("Staking", "Mining Boost: ${stakeInfo.miningBoost}x")
                    true
                }
                is StakingResult.Error -> {
                    Log.e("Staking", "Staking failed: ${result.message}")
                    false
                }
            }
        } catch (e: Exception) {
            Log.e("Staking", "Staking error", e)
            false
        }
    }
    
    suspend fun getStakingInfo(): StakingInfo? {
        return try {
            when (val result = stakingService.getStakingInfo()) {
                is StakingInfoResult.Success -> result.data
                is StakingInfoResult.Error -> {
                    Log.e("Staking", "Failed to get staking info: ${result.message}")
                    null
                }
            }
        } catch (e: Exception) {
            Log.e("Staking", "Staking info error", e)
            null
        }
    }
}
```

## Error Handling

### Custom Exception Handling

```kotlin
sealed class FinovaException(message: String) : Exception(message) {
    class NetworkException(message: String) : FinovaException(message)
    class AuthenticationException(message: String) : FinovaException(message)
    class ValidationException(message: String) : FinovaException(message)
    class RateLimitException(message: String) : FinovaException(message)
    class InsufficientBalanceException(message: String) : FinovaException(message)
}

// Usage in your code
try {
    finovaClient.performOperation()
} catch (e: FinovaException.NetworkException) {
    showNetworkError()
} catch (e: FinovaException.AuthenticationException) {
    redirectToLogin()
} catch (e: FinovaException.RateLimitException) {
    showRateLimitDialog()
}
```

## Proguard Configuration

Add to your `proguard-rules.pro`:

```
# Finova SDK
-keep class com.finova.sdk.** { *; }
-keep interface com.finova.sdk.** { *; }

# Gson
-keepattributes Signature
-keepattributes *Annotation*
-keep class sun.misc.Unsafe { *; }
-keep class com.google.gson.stream.** { *; }

# Retrofit
-keepattributes Signature
-keepattributes Exceptions
-keepclasseswithmembers class * {
    @retrofit2.http.* <methods>;
}
```

## Testing

### Unit Tests

```kotlin
@Test
fun testMiningCalculation() = runTest {
    val mockUser = User(
        level = 25,
        totalHoldings = 1000.0,
        activeReferrals = 10,
        isKycVerified = true
    )
    
    val miningRate = MiningCalculator.calculateRate(mockUser)
    
    assertTrue("Mining rate should be positive", miningRate > 0)
    assertTrue("Mining rate should have XP bonus", miningRate > 0.05)
}

@Test
fun testXPProgression() = runTest {
    val xpCalculator = XPCalculator()
    
    val baseXP = xpCalculator.calculateXP(
        activity = ActivityType.POST,
        platform = SocialPlatform.INSTAGRAM,
        qualityScore = 1.5,
        userLevel = 10
    )
    
    assertTrue("XP should be calculated correctly", baseXP > 0)
}
```

## Performance Optimization

### Caching Strategy

```kotlin
class CacheManager {
    companion object {
        const val CACHE_DURATION_MINING = 60_000L // 1 minute
        const val CACHE_DURATION_XP = 30_000L // 30 seconds
        const val CACHE_DURATION_NFT = 300_000L // 5 minutes
    }
    
    private val miningCache = LruCache<String, MiningData>(50)
    private val xpCache = LruCache<String, XPData>(100)
    
    fun getCachedMiningData(userId: String): MiningData? {
        return miningCache.get(userId)
    }
    
    fun cacheMiningData(userId: String, data: MiningData) {
        miningCache.put(userId, data)
    }
}
```

## Security Best Practices

### API Key Management

```kotlin
// Store API keys securely
class SecurityManager {
    companion object {
        fun getApiKey(context: Context): String {
            // Use Android Keystore for production
            val keyStore = context.getSharedPreferences("finova_secure", Context.MODE_PRIVATE)
            return keyStore.getString("api_key", "") ?: ""
        }
        
        fun validateSignature(message: String, signature: String, publicKey: String): Boolean {
            // Implement signature validation
            return CryptoUtils.verifySignature(message, signature, publicKey)
        }
    }
}
```

## Migration Guide

### From v0.9.x to v1.0.0

```kotlin
// Old way (v0.9.x)
FinovaClient.init("api_key")

// New way (v1.0.0)
FinovaSDK.initialize(
    context = this,
    config = FinovaConfig(
        apiKey = "api_key",
        environment = FinovaEnvironment.PRODUCTION
    )
)
```

## Support & Documentation

- 📚 **Full Documentation**: [docs.finova.network/android](https://docs.finova.network/android)
- 🐛 **Bug Reports**: [GitHub Issues](https://github.com/finova-network/android-sdk/issues)
- 💬 **Community**: [Discord Server](https://discord.gg/finova)
- 📧 **Support**: developers@finova.network

## License

```
MIT License

Copyright (c) 2025 Finova Network

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
```

---

**Built with ❤️ by the Finova Network Team**

*Join the Social-Fi revolution and start earning from your social media activity today!*
