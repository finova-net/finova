//
//  FinovaReactNative.h
//  Finova Network React Native SDK
//
//  Enterprise-grade iOS header for Finova Super App integration
//  Supports: Mining, XP System, RP System, NFT, Staking, Social Integration
//
//  Copyright © 2025 Finova Network. All rights reserved.
//

#import <React/RCTBridgeModule.h>
#import <React/RCTEventEmitter.h>
#import <Foundation/Foundation.h>

NS_ASSUME_NONNULL_BEGIN

@interface FinovaReactNative : RCTEventEmitter <RCTBridgeModule>

#pragma mark - Core Authentication & User Management

/**
 * Initialize Finova SDK with API credentials
 * @param apiKey Your Finova API key
 * @param environment "development", "staging", "production"
 * @param resolver Promise resolver
 * @param rejecter Promise rejecter
 */
- (void)initialize:(NSString *)apiKey
       environment:(NSString *)environment
          resolver:(RCTPromiseResolveBlock)resolver
          rejecter:(RCTPromiseRejectBlock)rejecter;

/**
 * Authenticate user with biometric + social login
 * @param socialProvider "google", "apple", "facebook", "twitter"
 * @param biometricEnabled Enable Face/Touch ID verification
 */
- (void)authenticateUser:(NSString *)socialProvider
        biometricEnabled:(BOOL)biometricEnabled
                resolver:(RCTPromiseResolveBlock)resolver
                rejecter:(RCTPromiseRejectBlock)rejecter;

/**
 * Complete KYC verification process
 * @param documentType "passport", "ktp", "driving_license"
 * @param frontImageUri Document front image URI
 * @param backImageUri Document back image URI
 * @param selfieUri User selfie image URI
 */
- (void)submitKYC:(NSString *)documentType
     frontImageUri:(NSString *)frontImageUri
      backImageUri:(NSString *)frontImageUri
         selfieUri:(NSString *)selfieUri
          resolver:(RCTPromiseResolveBlock)resolver
          rejecter:(RCTPromiseRejectBlock)rejecter;

#pragma mark - Mining System (Pi Network-inspired)

/**
 * Start mining session with exponential regression
 */
- (void)startMining:(RCTPromiseResolveBlock)resolver
           rejecter:(RCTPromiseRejectBlock)rejecter;

/**
 * Stop mining and calculate earned FIN tokens
 */
- (void)stopMining:(RCTPromiseResolveBlock)resolver
          rejecter:(RCTPromiseRejectBlock)rejecter;

/**
 * Get current mining rate with all multipliers applied
 * Formula: Base_Rate × Finizen_Bonus × Referral_Bonus × Security_Bonus × Regression_Factor
 */
- (void)getMiningRate:(RCTPromiseResolveBlock)resolver
             rejecter:(RCTPromiseRejectBlock)rejecter;

/**
 * Get mining statistics and performance metrics
 */
- (void)getMiningStats:(RCTPromiseResolveBlock)resolver
              rejecter:(RCTPromiseRejectBlock)rejecter;

#pragma mark - XP System (Hamster Kombat-inspired)

/**
 * Record social media activity for XP calculation
 * @param platform "instagram", "tiktok", "youtube", "facebook", "twitter"
 * @param activityType "post", "comment", "like", "share", "follow"
 * @param contentData Activity metadata (text, media URLs, engagement)
 */
- (void)recordActivity:(NSString *)platform
          activityType:(NSString *)activityType
           contentData:(NSDictionary *)contentData
              resolver:(RCTPromiseResolveBlock)resolver
              rejecter:(RCTPromiseRejectBlock)rejecter;

/**
 * Get current XP level and progression
 */
- (void)getXPStatus:(RCTPromiseResolveBlock)resolver
           rejecter:(RCTPromiseRejectBlock)rejecter;

/**
 * Get XP leaderboard and user ranking
 */
- (void)getXPLeaderboard:(NSNumber *)limit
                resolver:(RCTPromiseResolveBlock)resolver
                rejecter:(RCTPromiseRejectBlock)rejecter;

#pragma mark - Referral Points (RP) System

/**
 * Generate personalized referral code
 */
- (void)generateReferralCode:(RCTPromiseResolveBlock)resolver
                    rejecter:(RCTPromiseRejectBlock)rejecter;

/**
 * Apply referral code for new user
 */
- (void)applyReferralCode:(NSString *)referralCode
                 resolver:(RCTPromiseResolveBlock)resolver
                 rejecter:(RCTPromiseRejectBlock)rejecter;

/**
 * Get referral network statistics with exponential regression analysis
 */
- (void)getReferralStats:(RCTPromiseResolveBlock)resolver
                rejecter:(RCTPromiseRejectBlock)rejecter;

/**
 * Get RP tier status and benefits
 */
- (void)getRPTierStatus:(RCTPromiseResolveBlock)resolver
               rejecter:(RCTPromiseRejectBlock)rejecter;

#pragma mark - Token Management (Ethena-inspired)

/**
 * Get wallet balance for all Finova tokens
 * Returns: FIN, sFIN, USDfin, sUSDfin balances
 */
- (void)getWalletBalance:(RCTPromiseResolveBlock)resolver
                rejecter:(RCTPromiseRejectBlock)rejecter;

/**
 * Stake FIN tokens for enhanced rewards
 * @param amount Amount to stake in FIN tokens
 * @param duration Staking period in days
 */
- (void)stakeFIN:(NSNumber *)amount
        duration:(NSNumber *)duration
        resolver:(RCTPromiseResolveBlock)resolver
        rejecter:(RCTPromiseRejectBlock)rejecter;

/**
 * Unstake FIN tokens and claim rewards
 */
- (void)unstakeFIN:(NSNumber *)amount
          resolver:(RCTPromiseResolveBlock)resolver
          rejecter:(RCTPromiseRejectBlock)rejecter;

/**
 * Transfer tokens to another user
 * @param recipient Recipient's wallet address or username
 * @param tokenType "FIN", "sFIN", "USDfin", "sUSDfin"
 * @param amount Transfer amount
 */
- (void)transferTokens:(NSString *)recipient
             tokenType:(NSString *)tokenType
                amount:(NSNumber *)amount
              resolver:(RCTPromiseResolveBlock)resolver
              rejecter:(RCTPromiseRejectBlock)rejecter;

#pragma mark - NFT & Special Cards

/**
 * Get user's NFT collection
 */
- (void)getNFTCollection:(RCTPromiseResolveBlock)resolver
                rejecter:(RCTPromiseRejectBlock)rejecter;

/**
 * Use special card for temporary boosts
 * @param cardId NFT card identifier
 */
- (void)useSpecialCard:(NSString *)cardId
              resolver:(RCTPromiseResolveBlock)resolver
              rejecter:(RCTPromiseRejectBlock)rejecter;

/**
 * Purchase special card from marketplace
 * @param cardType "mining_boost", "xp_accelerator", "referral_power"
 * @param rarity "common", "uncommon", "rare", "epic", "legendary"
 */
- (void)purchaseSpecialCard:(NSString *)cardType
                     rarity:(NSString *)rarity
                   resolver:(RCTPromiseResolveBlock)resolver
                   rejecter:(RCTPromiseRejectBlock)rejecter;

#pragma mark - Guild System

/**
 * Create new guild
 * @param guildName Guild name
 * @param description Guild description
 * @param isPrivate Whether guild requires invitation
 */
- (void)createGuild:(NSString *)guildName
        description:(NSString *)description
          isPrivate:(BOOL)isPrivate
           resolver:(RCTPromiseResolveBlock)resolver
           rejecter:(RCTPromiseRejectBlock)rejecter;

/**
 * Join existing guild
 */
- (void)joinGuild:(NSString *)guildId
         resolver:(RCTPromiseResolveBlock)resolver
         rejecter:(RCTPromiseRejectBlock)rejecter;

/**
 * Get guild leaderboard and competitions
 */
- (void)getGuildLeaderboard:(NSString *)guildId
                   resolver:(RCTPromiseResolveBlock)resolver
                   rejecter:(RCTPromiseRejectBlock)rejecter;

#pragma mark - Social Platform Integration

/**
 * Connect social media account
 * @param platform Platform identifier
 * @param accessToken OAuth access token
 */
- (void)connectSocialAccount:(NSString *)platform
                 accessToken:(NSString *)accessToken
                    resolver:(RCTPromiseResolveBlock)resolver
                    rejecter:(RCTPromiseRejectBlock)rejecter;

/**
 * Disconnect social media account
 */
- (void)disconnectSocialAccount:(NSString *)platform
                       resolver:(RCTPromiseResolveBlock)resolver
                       rejecter:(RCTPromiseRejectBlock)rejecter;

/**
 * Get connected social accounts status
 */
- (void)getSocialAccountsStatus:(RCTPromiseResolveBlock)resolver
                       rejecter:(RCTPromiseRejectBlock)rejecter;

#pragma mark - E-Wallet Integration (Indonesian Market)

/**
 * Connect Indonesian e-wallet (OVO, GoPay, Dana, ShopeePay)
 * @param walletType "ovo", "gopay", "dana", "shopeepay"
 * @param phoneNumber User's phone number
 */
- (void)connectEWallet:(NSString *)walletType
           phoneNumber:(NSString *)phoneNumber
              resolver:(RCTPromiseResolveBlock)resolver
              rejecter:(RCTPromiseRejectBlock)rejecter;

/**
 * Convert FIN tokens to IDR via e-wallet
 * @param amount FIN token amount to convert
 * @param walletType Target e-wallet
 */
- (void)convertToIDR:(NSNumber *)amount
          walletType:(NSString *)walletType
            resolver:(RCTPromiseResolveBlock)resolver
            rejecter:(RCTPromiseRejectBlock)rejecter;

#pragma mark - Anti-Bot & Security

/**
 * Perform biometric verification for high-value operations
 * @param operationType "mining", "transfer", "staking", "nft_purchase"
 */
- (void)performBiometricVerification:(NSString *)operationType
                            resolver:(RCTPromiseResolveBlock)resolver
                            rejecter:(RCTPromiseRejectBlock)rejecter;

/**
 * Submit proof-of-humanity challenge
 * @param challengeType "captcha", "behavioral", "biometric", "social_graph"
 */
- (void)submitHumanityProof:(NSString *)challengeType
               challengeData:(NSDictionary *)challengeData
                   resolver:(RCTPromiseResolveBlock)resolver
                   rejecter:(RCTPromiseRejectBlock)rejecter;

#pragma mark - Analytics & Insights

/**
 * Get user dashboard analytics
 * @param timeframe "daily", "weekly", "monthly", "all_time"
 */
- (void)getDashboardAnalytics:(NSString *)timeframe
                     resolver:(RCTPromiseResolveBlock)resolver
                     rejecter:(RCTPromiseRejectBlock)rejecter;

/**
 * Get network growth insights
 */
- (void)getNetworkInsights:(RCTPromiseResolveBlock)resolver
                  rejecter:(RCTPromiseRejectBlock)rejecter;

#pragma mark - Notifications & Real-time Updates

/**
 * Subscribe to real-time updates
 * @param eventTypes Array of event types to subscribe to
 */
- (void)subscribeToUpdates:(NSArray<NSString *> *)eventTypes
                  resolver:(RCTPromiseResolveBlock)resolver
                  rejecter:(RCTPromiseRejectBlock)rejecter;

/**
 * Unsubscribe from real-time updates
 */
- (void)unsubscribeFromUpdates:(RCTPromiseResolveBlock)resolver
                      rejecter:(RCTPromiseRejectBlock)rejecter;

#pragma mark - Utility Methods

/**
 * Get current Finova network status
 */
- (void)getNetworkStatus:(RCTPromiseResolveBlock)resolver
                rejecter:(RCTPromiseRejectBlock)rejecter;

/**
 * Calculate estimated rewards based on activity
 * @param activities Array of planned activities
 */
- (void)calculateEstimatedRewards:(NSArray *)activities
                         resolver:(RCTPromiseResolveBlock)resolver
                         rejecter:(RCTPromiseRejectBlock)rejecter;

/**
 * Get app configuration and feature flags
 */
- (void)getAppConfig:(RCTPromiseResolveBlock)resolver
            rejecter:(RCTPromiseRejectBlock)rejecter;

#pragma mark - Debug & Development (Debug builds only)

#ifdef DEBUG
/**
 * Reset user data for testing (Debug only)
 */
- (void)resetUserDataForTesting:(RCTPromiseResolveBlock)resolver
                       rejecter:(RCTPromiseRejectBlock)rejecter;

/**
 * Simulate mining for testing (Debug only)
 */
- (void)simulateMiningForTesting:(NSNumber *)hours
                        resolver:(RCTPromiseResolveBlock)resolver
                        rejecter:(RCTPromiseRejectBlock)rejecter;
#endif

@end

#pragma mark - Event Names (for RCTEventEmitter)

// Real-time event constants
FOUNDATION_EXPORT NSString *const kFinovaMiningStatusChanged;
FOUNDATION_EXPORT NSString *const kFinovaXPUpdated;
FOUNDATION_EXPORT NSString *const kFinovaRPUpdated;
FOUNDATION_EXPORT NSString *const kFinovaTokenBalanceChanged;
FOUNDATION_EXPORT NSString *const kFinovaNFTReceived;
FOUNDATION_EXPORT NSString *const kFinovaGuildEventOccurred;
FOUNDATION_EXPORT NSString *const kFinovaNetworkStatusChanged;
FOUNDATION_EXPORT NSString *const kFinovaBiometricRequired;
FOUNDATION_EXPORT NSString *const kFinovaHumanityProofRequired;

#pragma mark - Error Codes

typedef NS_ENUM(NSInteger, FinovaErrorCode) {
    FinovaErrorCodeNetworkError = 1000,
    FinovaErrorCodeAuthenticationFailed = 1001,
    FinovaErrorCodeKYCRequired = 1002,
    FinovaErrorCodeInsufficientBalance = 1003,
    FinovaErrorCodeMiningNotActive = 1004,
    FinovaErrorCodeBiometricFailed = 1005,
    FinovaErrorCodeHumanityProofFailed = 1006,
    FinovaErrorCodeRateLimitExceeded = 1007,
    FinovaErrorCodeInvalidParameters = 1008,
    FinovaErrorCodeServiceUnavailable = 1009,
    FinovaErrorCodeUnauthorized = 1010
};

NS_ASSUME_NONNULL_END
