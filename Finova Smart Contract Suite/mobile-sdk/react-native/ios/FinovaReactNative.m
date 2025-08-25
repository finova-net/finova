//
//  FinovaReactNative.m
//  Finova Network React Native iOS Bridge
//  
//  Enterprise-grade implementation with security, mining, XP, RP systems
//  Copyright © 2025 Finova Network. All rights reserved.
//

#import "FinovaReactNative.h"
#import <React/RCTLog.h>
#import <React/RCTUtils.h>
#import <React/RCTConvert.h>
#import <CommonCrypto/CommonCrypto.h>
#import <LocalAuthentication/LocalAuthentication.h>
#import <CryptoKit/CryptoKit.h>
#import <AdSupport/AdSupport.h>
#import <AppTrackingTransparency/AppTrackingTransparency.h>

@interface FinovaReactNative()

@property (nonatomic, strong) NSTimer *miningTimer;
@property (nonatomic, strong) NSMutableDictionary *userState;
@property (nonatomic, strong) NSMutableDictionary *miningConfig;
@property (nonatomic, strong) NSMutableArray *activityQueue;
@property (nonatomic, strong) NSString *deviceFingerprint;
@property (nonatomic, strong) NSString *encryptionKey;
@property (nonatomic, assign) BOOL isInitialized;
@property (nonatomic, assign) BOOL isMiningActive;
@property (nonatomic, assign) double lastActivityTime;

@end

@implementation FinovaReactNative

RCT_EXPORT_MODULE()

#pragma mark - Initialization

- (instancetype)init {
    self = [super init];
    if (self) {
        [self initializeFinovaSDK];
    }
    return self;
}

- (void)initializeFinovaSDK {
    self.userState = [NSMutableDictionary dictionary];
    self.miningConfig = [NSMutableDictionary dictionary];
    self.activityQueue = [NSMutableArray array];
    self.deviceFingerprint = [self generateDeviceFingerprint];
    self.encryptionKey = [self generateEncryptionKey];
    self.isInitialized = NO;
    self.isMiningActive = NO;
    self.lastActivityTime = [[NSDate date] timeIntervalSince1970];
    
    // Setup default mining configuration
    [self setupDefaultMiningConfig];
    
    RCTLogInfo(@"Finova SDK initialized for iOS");
}

- (void)setupDefaultMiningConfig {
    self.miningConfig[@"baseRate"] = @0.05;
    self.miningConfig[@"pioneerBonus"] = @2.0;
    self.miningConfig[@"maxDailyLimit"] = @4.8;
    self.miningConfig[@"phase"] = @1;
    self.miningConfig[@"networkSize"] = @0;
    self.miningConfig[@"regressionFactor"] = @1.0;
}

#pragma mark - React Native Bridge Methods

RCT_EXPORT_METHOD(initialize:(NSDictionary *)config
                  resolver:(RCTPromiseResolveBlock)resolve
                  rejecter:(RCTPromiseRejectBlock)reject) {
    
    @try {
        // Validate configuration
        if (![self validateConfig:config]) {
            reject(@"INVALID_CONFIG", @"Invalid configuration provided", nil);
            return;
        }
        
        // Store user configuration
        [self.userState addEntriesFromDictionary:config];
        
        // Initialize biometric authentication
        [self initializeBiometricAuth:^(BOOL success, NSError *error) {
            if (success) {
                self.isInitialized = YES;
                resolve(@{
                    @"success": @YES,
                    @"deviceId": self.deviceFingerprint,
                    @"timestamp": @([[NSDate date] timeIntervalSince1970])
                });
            } else {
                reject(@"BIOMETRIC_FAILED", @"Biometric initialization failed", error);
            }
        }];
        
    } @catch (NSException *exception) {
        reject(@"INIT_ERROR", exception.reason, nil);
    }
}

RCT_EXPORT_METHOD(startMining:(RCTPromiseResolveBlock)resolve
                  rejecter:(RCTPromiseRejectBlock)reject) {
    
    if (!self.isInitialized) {
        reject(@"NOT_INITIALIZED", @"SDK not initialized", nil);
        return;
    }
    
    if (self.isMiningActive) {
        reject(@"MINING_ACTIVE", @"Mining already active", nil);
        return;
    }
    
    [self authenticateUser:^(BOOL success, NSError *error) {
        if (success) {
            [self initiateMining];
            resolve(@{
                @"success": @YES,
                @"miningRate": [self calculateMiningRate],
                @"timestamp": @([[NSDate date] timeIntervalSince1970])
            });
        } else {
            reject(@"AUTH_FAILED", @"User authentication failed", error);
        }
    }];
}

RCT_EXPORT_METHOD(stopMining:(RCTPromiseResolveBlock)resolve
                  rejecter:(RCTPromiseRejectBlock)reject) {
    
    [self stopMiningProcess];
    resolve(@{
        @"success": @YES,
        @"totalMined": self.userState[@"totalMined"] ?: @0,
        @"timestamp": @([[NSDate date] timeIntervalSince1970])
    });
}

RCT_EXPORT_METHOD(addXPActivity:(NSDictionary *)activity
                  resolver:(RCTPromiseResolveBlock)resolve
                  rejecter:(RCTPromiseRejectBlock)reject) {
    
    @try {
        NSDictionary *result = [self processXPActivity:activity];
        
        // Update user state
        NSNumber *currentXP = self.userState[@"totalXP"] ?: @0;
        NSNumber *gainedXP = result[@"xpGained"];
        self.userState[@"totalXP"] = @([currentXP doubleValue] + [gainedXP doubleValue]);
        
        // Check for level up
        NSDictionary *levelInfo = [self checkLevelUp];
        
        resolve(@{
            @"success": @YES,
            @"xpGained": gainedXP,
            @"totalXP": self.userState[@"totalXP"],
            @"currentLevel": levelInfo[@"level"],
            @"leveledUp": levelInfo[@"leveledUp"],
            @"qualityScore": result[@"qualityScore"]
        });
        
    } @catch (NSException *exception) {
        reject(@"XP_ERROR", exception.reason, nil);
    }
}

RCT_EXPORT_METHOD(addReferral:(NSString *)referralCode
                  userInfo:(NSDictionary *)userInfo
                  resolver:(RCTPromiseResolveBlock)resolve
                  rejecter:(RCTPromiseRejectBlock)reject) {
    
    @try {
        // Validate referral code
        if (![self validateReferralCode:referralCode]) {
            reject(@"INVALID_REFERRAL", @"Invalid referral code", nil);
            return;
        }
        
        // Process referral
        NSDictionary *result = [self processReferral:referralCode userInfo:userInfo];
        
        // Update referral points
        NSMutableArray *referrals = self.userState[@"referrals"] ?: [NSMutableArray array];
        [referrals addObject:result[@"referralData"]];
        self.userState[@"referrals"] = referrals;
        
        NSNumber *currentRP = self.userState[@"totalRP"] ?: @0;
        NSNumber *gainedRP = result[@"rpGained"];
        self.userState[@"totalRP"] = @([currentRP doubleValue] + [gainedRP doubleValue]);
        
        resolve(@{
            @"success": @YES,
            @"rpGained": gainedRP,
            @"totalRP": self.userState[@"totalRP"],
            @"referralTier": [self calculateRPTier],
            @"networkSize": @([referrals count])
        });
        
    } @catch (NSException *exception) {
        reject(@"REFERRAL_ERROR", exception.reason, nil);
    }
}

RCT_EXPORT_METHOD(useSpecialCard:(NSString *)cardId
                  resolver:(RCTPromiseResolveBlock)resolve
                  rejecter:(RCTPromiseRejectBlock)reject) {
    
    @try {
        NSDictionary *cardInfo = [self getCardInfo:cardId];
        
        if (!cardInfo) {
            reject(@"INVALID_CARD", @"Card not found or already used", nil);
            return;
        }
        
        // Apply card effects
        NSDictionary *effects = [self applyCardEffects:cardInfo];
        
        // Mark card as used
        [self markCardAsUsed:cardId];
        
        resolve(@{
            @"success": @YES,
            @"cardName": cardInfo[@"name"],
            @"effects": effects,
            @"duration": cardInfo[@"duration"],
            @"timestamp": @([[NSDate date] timeIntervalSince1970])
        });
        
    } @catch (NSException *exception) {
        reject(@"CARD_ERROR", exception.reason, nil);
    }
}

RCT_EXPORT_METHOD(getUserStats:(RCTPromiseResolveBlock)resolve
                  rejecter:(RCTPromiseRejectBlock)reject) {
    
    @try {
        NSDictionary *stats = [self compileUserStats];
        resolve(stats);
        
    } @catch (NSException *exception) {
        reject(@"STATS_ERROR", exception.reason, nil);
    }
}

RCT_EXPORT_METHOD(stakeFIN:(NSNumber *)amount
                  duration:(NSNumber *)duration
                  resolver:(RCTPromiseResolveBlock)resolve
                  rejecter:(RCTPromiseRejectBlock)reject) {
    
    @try {
        // Validate staking parameters
        if ([amount doubleValue] < 100) {
            reject(@"INVALID_STAKE", @"Minimum stake is 100 FIN", nil);
            return;
        }
        
        NSNumber *availableFIN = self.userState[@"totalMined"] ?: @0;
        if ([availableFIN doubleValue] < [amount doubleValue]) {
            reject(@"INSUFFICIENT_BALANCE", @"Insufficient FIN balance", nil);
            return;
        }
        
        // Process staking
        NSDictionary *stakingInfo = [self processStaking:amount duration:duration];
        
        // Update user state
        self.userState[@"totalMined"] = @([availableFIN doubleValue] - [amount doubleValue]);
        
        NSMutableArray *stakes = self.userState[@"stakes"] ?: [NSMutableArray array];
        [stakes addObject:stakingInfo];
        self.userState[@"stakes"] = stakes;
        
        resolve(@{
            @"success": @YES,
            @"stakeId": stakingInfo[@"id"],
            @"amount": amount,
            @"expectedAPY": stakingInfo[@"apy"],
            @"unlockTime": stakingInfo[@"unlockTime"],
            @"multipliers": stakingInfo[@"multipliers"]
        });
        
    } @catch (NSException *exception) {
        reject(@"STAKING_ERROR", exception.reason, nil);
    }
}

#pragma mark - Mining Engine

- (void)initiateMining {
    self.isMiningActive = YES;
    
    // Start mining timer (every minute)
    self.miningTimer = [NSTimer scheduledTimerWithTimeInterval:60.0
                                                        target:self
                                                      selector:@selector(processMiningCycle)
                                                      userInfo:nil
                                                       repeats:YES];
    
    RCTLogInfo(@"Mining started with rate: %f FIN/hour", [[self calculateMiningRate] doubleValue]);
}

- (void)stopMiningProcess {
    self.isMiningActive = NO;
    
    if (self.miningTimer) {
        [self.miningTimer invalidate];
        self.miningTimer = nil;
    }
    
    RCTLogInfo(@"Mining stopped");
}

- (void)processMiningCycle {
    if (!self.isMiningActive) return;
    
    // Calculate mining reward for this cycle
    NSNumber *miningRate = [self calculateMiningRate];
    double rewardPerMinute = [miningRate doubleValue] / 60.0;
    
    // Apply activity bonus
    double activityBonus = [self calculateActivityBonus];
    double finalReward = rewardPerMinute * activityBonus;
    
    // Update total mined
    NSNumber *currentTotal = self.userState[@"totalMined"] ?: @0;
    self.userState[@"totalMined"] = @([currentTotal doubleValue] + finalReward);
    
    // Check daily limits
    [self enforceDaily Limits];
    
    // Send mining update to React Native
    [self sendMiningUpdate:@{
        @"reward": @(finalReward),
        @"totalMined": self.userState[@"totalMined"],
        @"miningRate": miningRate,
        @"activityBonus": @(activityBonus)
    }];
}

- (NSNumber *)calculateMiningRate {
    double baseRate = [self.miningConfig[@"baseRate"] doubleValue];
    double pioneerBonus = [self.miningConfig[@"pioneerBonus"] doubleValue];
    double referralBonus = [self calculateReferralBonus];
    double securityBonus = [self.userState[@"isKYCVerified"] boolValue] ? 1.2 : 0.8;
    double regressionFactor = [self calculateRegressionFactor];
    
    double finalRate = baseRate * pioneerBonus * referralBonus * securityBonus * regressionFactor;
    
    return @(finalRate);
}

- (double)calculateReferralBonus {
    NSArray *referrals = self.userState[@"referrals"] ?: @[];
    NSInteger activeReferrals = [self countActiveReferrals:referrals];
    
    return 1.0 + (activeReferrals * 0.1);
}

- (double)calculateRegressionFactor {
    NSNumber *totalHoldings = self.userState[@"totalMined"] ?: @0;
    double holdings = [totalHoldings doubleValue];
    
    // Exponential regression: e^(-0.001 * holdings)
    return exp(-0.001 * holdings);
}

- (double)calculateActivityBonus {
    double currentTime = [[NSDate date] timeIntervalSince1970];
    double timeSinceActivity = currentTime - self.lastActivityTime;
    
    // Activity bonus decreases over time (max 24 hours)
    double activityFactor = MAX(0.5, 1.0 - (timeSinceActivity / 86400.0));
    
    return activityFactor;
}

#pragma mark - XP System

- (NSDictionary *)processXPActivity:(NSDictionary *)activity {
    NSString *activityType = activity[@"type"];
    NSString *platform = activity[@"platform"];
    NSString *content = activity[@"content"];
    
    // Base XP calculation
    double baseXP = [self getBaseXPForActivity:activityType];
    
    // Platform multiplier
    double platformMultiplier = [self getPlatformMultiplier:platform];
    
    // Quality score from AI analysis
    double qualityScore = [self analyzeContentQuality:content];
    
    // Streak bonus
    double streakBonus = [self calculateStreakBonus];
    
    // Level progression factor
    NSNumber *currentLevel = self.userState[@"currentLevel"] ?: @1;
    double levelProgression = exp(-0.01 * [currentLevel doubleValue]);
    
    double finalXP = baseXP * platformMultiplier * qualityScore * streakBonus * levelProgression;
    
    // Update last activity time
    self.lastActivityTime = [[NSDate date] timeIntervalSince1970];
    
    return @{
        @"xpGained": @(finalXP),
        @"qualityScore": @(qualityScore),
        @"breakdown": @{
            @"baseXP": @(baseXP),
            @"platformMultiplier": @(platformMultiplier),
            @"qualityScore": @(qualityScore),
            @"streakBonus": @(streakBonus),
            @"levelProgression": @(levelProgression)
        }
    };
}

- (double)getBaseXPForActivity:(NSString *)activityType {
    NSDictionary *xpValues = @{
        @"original_post": @50,
        @"photo_post": @75,
        @"video_content": @150,
        @"story_status": @25,
        @"comment": @25,
        @"like_react": @5,
        @"share_repost": @15,
        @"follow_subscribe": @20,
        @"daily_login": @10,
        @"daily_quest": @100,
        @"viral_content": @1000
    };
    
    return [xpValues[activityType] doubleValue] ?: 10.0;
}

- (double)getPlatformMultiplier:(NSString *)platform {
    NSDictionary *multipliers = @{
        @"tiktok": @1.3,
        @"instagram": @1.2,
        @"youtube": @1.4,
        @"x": @1.2,
        @"facebook": @1.1,
        @"default": @1.0
    };
    
    return [multipliers[platform.lowercaseString] doubleValue] ?: 1.0;
}

- (double)analyzeContentQuality:(NSString *)content {
    if (!content || content.length == 0) return 0.8;
    
    // Simple quality metrics (in production, use AI service)
    double qualityScore = 1.0;
    
    // Length factor
    if (content.length < 10) qualityScore *= 0.8;
    else if (content.length > 100) qualityScore *= 1.2;
    
    // Uniqueness check (simplified)
    if ([self isContentOriginal:content]) {
        qualityScore *= 1.3;
    }
    
    // Clamp between 0.5 and 2.0
    return MAX(0.5, MIN(2.0, qualityScore));
}

- (double)calculateStreakBonus {
    NSNumber *streakDays = self.userState[@"streakDays"] ?: @0;
    return MIN(3.0, 1.0 + ([streakDays integerValue] * 0.1));
}

- (NSDictionary *)checkLevelUp {
    NSNumber *totalXP = self.userState[@"totalXP"] ?: @0;
    NSNumber *currentLevel = self.userState[@"currentLevel"] ?: @1;
    
    NSInteger requiredXP = [self getXPRequiredForLevel:[currentLevel integerValue] + 1];
    
    if ([totalXP integerValue] >= requiredXP) {
        NSInteger newLevel = [currentLevel integerValue] + 1;
        self.userState[@"currentLevel"] = @(newLevel);
        
        return @{
            @"level": @(newLevel),
            @"leveledUp": @YES,
            @"miningBonus": @([self getMiningMultiplierForLevel:newLevel])
        };
    }
    
    return @{
        @"level": currentLevel,
        @"leveledUp": @NO,
        @"miningBonus": @([self getMiningMultiplierForLevel:[currentLevel integerValue]])
    };
}

#pragma mark - RP System

- (NSDictionary *)processReferral:(NSString *)referralCode userInfo:(NSDictionary *)userInfo {
    // Generate referral data
    NSString *referralId = [self generateReferralID];
    double timestamp = [[NSDate date] timeIntervalSince1970];
    
    NSDictionary *referralData = @{
        @"id": referralId,
        @"code": referralCode,
        @"userInfo": userInfo,
        @"timestamp": @(timestamp),
        @"status": @"active",
        @"level": @1
    };
    
    // Calculate RP gained
    double baseRP = 50; // Registration RP
    if ([userInfo[@"isKYCVerified"] boolValue]) {
        baseRP += 100; // KYC bonus
    }
    
    return @{
        @"referralData": referralData,
        @"rpGained": @(baseRP)
    };
}

- (NSString *)calculateRPTier {
    NSNumber *totalRP = self.userState[@"totalRP"] ?: @0;
    double rp = [totalRP doubleValue];
    
    if (rp >= 50000) return @"Ambassador";
    if (rp >= 15000) return @"Leader";
    if (rp >= 5000) return @"Influencer";
    if (rp >= 1000) return @"Connector";
    return @"Explorer";
}

#pragma mark - Special Cards & NFTs

- (NSDictionary *)getCardInfo:(NSString *)cardId {
    // In production, this would query the blockchain/database
    NSArray *userCards = self.userState[@"specialCards"] ?: @[];
    
    for (NSDictionary *card in userCards) {
        if ([card[@"id"] isEqualToString:cardId] && ![card[@"used"] boolValue]) {
            return card;
        }
    }
    
    return nil;
}

- (NSDictionary *)applyCardEffects:(NSDictionary *)cardInfo {
    NSString *cardType = cardInfo[@"type"];
    NSDictionary *effects = cardInfo[@"effects"];
    
    NSMutableDictionary *appliedEffects = [NSMutableDictionary dictionary];
    
    if ([cardType isEqualToString:@"mining_boost"]) {
        double multiplier = [effects[@"miningMultiplier"] doubleValue];
        NSNumber *duration = effects[@"duration"];
        
        // Store active effect
        NSMutableArray *activeEffects = self.userState[@"activeEffects"] ?: [NSMutableArray array];
        [activeEffects addObject:@{
            @"type": @"mining_boost",
            @"multiplier": @(multiplier),
            @"expiresAt": @([[NSDate date] timeIntervalSince1970] + [duration doubleValue])
        }];
        self.userState[@"activeEffects"] = activeEffects;
        
        appliedEffects[@"miningBoost"] = @(multiplier);
        appliedEffects[@"duration"] = duration;
    }
    
    return appliedEffects;
}

- (void)markCardAsUsed:(NSString *)cardId {
    NSMutableArray *userCards = [self.userState[@"specialCards"] mutableCopy] ?: [NSMutableArray array];
    
    for (NSMutableDictionary *card in userCards) {
        if ([card[@"id"] isEqualToString:cardId]) {
            card[@"used"] = @YES;
            card[@"usedAt"] = @([[NSDate date] timeIntervalSince1970]);
            break;
        }
    }
    
    self.userState[@"specialCards"] = userCards;
}

#pragma mark - Staking System

- (NSDictionary *)processStaking:(NSNumber *)amount duration:(NSNumber *)duration {
    NSString *stakeId = [self generateStakeID];
    double timestamp = [[NSDate date] timeIntervalSince1970];
    double unlockTime = timestamp + ([duration doubleValue] * 86400); // Convert days to seconds
    
    // Calculate APY based on amount and duration
    double apy = [self calculateStakingAPY:amount duration:duration];
    
    // Calculate multipliers
    NSDictionary *multipliers = [self calculateStakingMultipliers:amount];
    
    return @{
        @"id": stakeId,
        @"amount": amount,
        @"duration": duration,
        @"apy": @(apy),
        @"startTime": @(timestamp),
        @"unlockTime": @(unlockTime),
        @"multipliers": multipliers,
        @"status": @"active"
    };
}

- (double)calculateStakingAPY:(NSNumber *)amount duration:(NSNumber *)duration {
    double baseAPY = 8.0; // 8% base
    
    // Amount tier bonus
    double amountValue = [amount doubleValue];
    if (amountValue >= 10000) baseAPY += 7.0;
    else if (amountValue >= 5000) baseAPY += 6.0;
    else if (amountValue >= 1000) baseAPY += 4.0;
    else if (amountValue >= 500) baseAPY += 2.0;
    
    // Duration bonus
    double durationDays = [duration doubleValue];
    if (durationDays >= 365) baseAPY += 3.0;
    else if (durationDays >= 180) baseAPY += 2.0;
    else if (durationDays >= 90) baseAPY += 1.0;
    
    return MIN(15.0, baseAPY); // Cap at 15%
}

- (NSDictionary *)calculateStakingMultipliers:(NSNumber *)amount {
    double amountValue = [amount doubleValue];
    
    double miningBoost = 0.2; // 20% base
    double xpMultiplier = 0.1; // 10% base
    double rpBonus = 0.05; // 5% base
    
    if (amountValue >= 10000) {
        miningBoost = 1.0; // 100%
        xpMultiplier = 0.75; // 75%
        rpBonus = 0.5; // 50%
    } else if (amountValue >= 5000) {
        miningBoost = 0.75; // 75%
        xpMultiplier = 0.5; // 50%
        rpBonus = 0.35; // 35%
    } else if (amountValue >= 1000) {
        miningBoost = 0.5; // 50%
        xpMultiplier = 0.3; // 30%
        rpBonus = 0.2; // 20%
    }
    
    return @{
        @"miningBoost": @(miningBoost),
        @"xpMultiplier": @(xpMultiplier),
        @"rpBonus": @(rpBonus)
    };
}

#pragma mark - Security & Anti-Bot

- (BOOL)validateConfig:(NSDictionary *)config {
    // Validate required fields
    NSArray *requiredFields = @[@"userId", @"apiKey"];
    
    for (NSString *field in requiredFields) {
        if (!config[field] || [config[field] length] == 0) {
            return NO;
        }
    }
    
    // Validate API key format
    NSString *apiKey = config[@"apiKey"];
    if (apiKey.length < 32) {
        return NO;
    }
    
    return YES;
}

- (BOOL)validateReferralCode:(NSString *)referralCode {
    // Basic validation
    if (!referralCode || referralCode.length < 6) {
        return NO;
    }
    
    // Check format (alphanumeric)
    NSCharacterSet *alphanumeric = [NSCharacterSet alphanumericCharacterSet];
    NSCharacterSet *inputSet = [NSCharacterSet characterSetWithCharactersInString:referralCode];
    
    return [alphanumeric isSupersetOfSet:inputSet];
}

- (void)initializeBiometricAuth:(void (^)(BOOL success, NSError *error))completion {
    LAContext *context = [[LAContext alloc] init];
    NSError *error = nil;
    
    if ([context canEvaluatePolicy:LAPolicyDeviceOwnerAuthenticationWithBiometrics error:&error]) {
        [context evaluatePolicy:LAPolicyDeviceOwnerAuthenticationWithBiometrics
                localizedReason:@"Authenticate to access Finova Network"
                          reply:^(BOOL success, NSError *error) {
            dispatch_async(dispatch_get_main_queue(), ^{
                completion(success, error);
            });
        }];
    } else {
        // Fallback to device passcode
        [context evaluatePolicy:LAPolicyDeviceOwnerAuthentication
                localizedReason:@"Authenticate to access Finova Network"
                          reply:^(BOOL success, NSError *error) {
            dispatch_async(dispatch_get_main_queue(), ^{
                completion(success, error);
            });
        }];
    }
}

- (void)authenticateUser:(void (^)(BOOL success, NSError *error))completion {
    [self initializeBiometricAuth:completion];
}

#pragma mark - Utility Methods

- (NSString *)generateDeviceFingerprint {
    // Create unique device identifier
    NSString *deviceId = [[[UIDevice currentDevice] identifierForVendor] UUIDString];
    NSString *systemVersion = [[UIDevice currentDevice] systemVersion];
    NSString *model = [[UIDevice currentDevice] model];
    
    NSString *fingerprint = [NSString stringWithFormat:@"%@_%@_%@", deviceId, systemVersion, model];
    
    return [self sha256:fingerprint];
}

- (NSString *)generateEncryptionKey {
    NSMutableData *keyData = [NSMutableData dataWithLength:32];
    int result = SecRandomCopyBytes(kSecRandomDefault, 32, keyData.mutableBytes);
    
    if (result == errSecSuccess) {
        return [keyData base64EncodedStringWithOptions:0];
    }
    
    return @"default_encryption_key_fallback";
}

- (NSString *)generateReferralID {
    NSUUID *uuid = [NSUUID UUID];
    return [uuid.UUIDString stringByReplacingOccurrencesOfString:@"-" withString:@""];
}

- (NSString *)generateStakeID {
    return [NSString stringWithFormat:@"stake_%@_%ld",
            [self generateReferralID], (long)[[NSDate date] timeIntervalSince1970]];
}

- (NSString *)sha256:(NSString *)input {
    const char *str = [input UTF8String];
    unsigned char result[CC_SHA256_DIGEST_LENGTH];
    CC_SHA256(str, (CC_LONG)strlen(str), result);
    
    NSMutableString *hash = [NSMutableString stringWithCapacity:CC_SHA256_DIGEST_LENGTH * 2];
    for (int i = 0; i < CC_SHA256_DIGEST_LENGTH; i++) {
        [hash appendFormat:@"%02x", result[i]];
    }
    
    return [hash copy];
}

- (BOOL)isContentOriginal:(NSString *)content {
    // Simple originality check (in production, use AI service)
    NSArray *cachedContent = self.userState[@"contentHistory"] ?: @[];
    
    for (NSString *previousContent in cachedContent) {
        if ([content isEqualToString:previousContent]) {
            return NO;
        }
    }
    
    // Store content for future checks
    NSMutableArray *contentHistory = [cachedContent mutableCopy];
    [contentHistory addObject:content];
    
    // Keep only last 100 entries
    if (contentHistory.count > 100) {
        [contentHistory removeObjectAtIndex:0];
    }
    
    self.userState[@"contentHistory"] = contentHistory;
    return YES;
}

- (NSInteger)countActiveReferrals:(NSArray *)referrals {
    NSInteger activeCount = 0;
    double currentTime = [[NSDate date] timeIntervalSince1970];
    double thirtyDaysAgo = currentTime - (30 * 24 * 60 * 60);
    
    for (NSDictionary *referral in referrals) {
        double lastActivity = [referral[@"lastActivity"] doubleValue];
        if (lastActivity > thirtyDaysAgo && [referral[@"status"] isEqualToString:@"active"]) {
            activeCount++;
        }
    }
    
    return activeCount;
}

- (NSInteger)getXPRequiredForLevel:(NSInteger)level {
    if (level <= 10) return level * 100;
    if (level <= 25) return 1000 + ((level - 10) * 200);
    if (level <= 50) return 4000 + ((level - 25) * 400);
    if (level <= 75) return 14000 + ((level - 50) * 800);
    if (level <= 100) return 34000 + ((level - 75) * 1600);
    return 74000 + ((level - 100) * 3200);
}

- (double)getMiningMultiplierForLevel:(NSInteger)level {
    if (level <= 10) return 1.0 + (level * 0.02);
    if (level <= 25) return 1.2 + ((level - 10) * 0.04);
    if (level <= 50) return 1.8 + ((level - 25) * 0.028);
    if (level <= 75) return 2.5 + ((level - 50) * 0.028);
    if (level <= 100) return 3.2 + ((level - 75) * 0.032);
    return 4.0 + ((level - 100) * 0.01); // Max 5.0x
}

- (void)enforceDailyLimits {
    NSNumber *dailyMined = self.userState[@"dailyMined"] ?: @0;
    NSNumber *maxDaily = self.miningConfig[@"maxDailyLimit"];
    
    if ([dailyMined doubleValue] >= [maxDaily doubleValue]) {
        [self stopMiningProcess];
        
        [self sendEventToReactNative:@"dailyLimitReached" body:@{
            @"dailyMined": dailyMined,
            @"maxDaily": maxDaily,
            @"resetTime": @([self getNextDayResetTime])
        }];
    }
}

- (double)getNextDayResetTime {
    NSCalendar *calendar = [NSCalendar currentCalendar];
    NSDate *now = [NSDate date];
    NSDate *tomorrow = [calendar dateByAddingUnit:NSCalendarUnitDay value:1 toDate:now options:0];
    NSDate *startOfTomorrow = [calendar startOfDayForDate:tomorrow];
    
    return [startOfTomorrow timeIntervalSince1970];
}

- (NSDictionary *)compileUserStats {
    NSNumber *totalXP = self.userState[@"totalXP"] ?: @0;
    NSNumber *currentLevel = self.userState[@"currentLevel"] ?: @1;
    NSNumber *totalRP = self.userState[@"totalRP"] ?: @0;
    NSNumber *totalMined = self.userState[@"totalMined"] ?: @0;
    NSArray *referrals = self.userState[@"referrals"] ?: @[];
    NSArray *stakes = self.userState[@"stakes"] ?: @[];
    NSArray *specialCards = self.userState[@"specialCards"] ?: @[];
    
    // Calculate mining rate with all bonuses
    NSNumber *currentMiningRate = [self calculateMiningRate];
    
    // Calculate staking info
    double totalStaked = 0;
    double stakingRewards = 0;
    for (NSDictionary *stake in stakes) {
        if ([stake[@"status"] isEqualToString:@"active"]) {
            totalStaked += [stake[@"amount"] doubleValue];
            // Calculate accrued rewards
            double startTime = [stake[@"startTime"] doubleValue];
            double currentTime = [[NSDate date] timeIntervalSince1970];
            double daysStaked = (currentTime - startTime) / 86400.0;
            double apy = [stake[@"apy"] doubleValue];
            double yearlyReward = [stake[@"amount"] doubleValue] * (apy / 100.0);
            stakingRewards += (yearlyReward * daysStaked / 365.0);
        }
    }
    
    return @{
        @"totalXP": totalXP,
        @"currentLevel": currentLevel,
        @"nextLevelXP": @([self getXPRequiredForLevel:[currentLevel integerValue] + 1]),
        @"totalRP": totalRP,
        @"rpTier": [self calculateRPTier],
        @"totalMined": totalMined,
        @"totalStaked": @(totalStaked),
        @"stakingRewards": @(stakingRewards),
        @"currentMiningRate": currentMiningRate,
        @"networkSize": @([referrals count]),
        @"activeReferrals": @([self countActiveReferrals:referrals]),
        @"specialCardsCount": @([specialCards count]),
        @"deviceId": self.deviceFingerprint,
        @"isMiningActive": @(self.isMiningActive),
        @"lastActivity": @(self.lastActivityTime),
        @"timestamp": @([[NSDate date] timeIntervalSince1970])
    };
}

#pragma mark - Event Bridge

- (void)sendMiningUpdate:(NSDictionary *)data {
    [self sendEventToReactNative:@"miningUpdate" body:data];
}

- (void)sendEventToReactNative:(NSString *)eventName body:(NSDictionary *)body {
    if (self.bridge) {
        [self.bridge enqueueJSCall:@"RCTDeviceEventEmitter"
                            method:@"emit"
                              args:@[eventName, body]
                        completion:NULL];
    }
}

#pragma mark - React Native Required Methods

- (dispatch_queue_t)methodQueue {
    return dispatch_get_main_queue();
}

+ (BOOL)requiresMainQueueSetup {
    return YES;
}

- (NSArray<NSString *> *)supportedEvents {
    return @[
        @"miningUpdate",
        @"levelUp",
        @"referralSuccess",
        @"cardActivated",
        @"dailyLimitReached",
        @"stakingReward",
        @"securityAlert"
    ];
}

#pragma mark - Background Processing

- (void)handleAppStateChange:(NSString *)newState {
    if ([newState isEqualToString:@"background"]) {
        [self pauseMining];
        [self saveUserState];
    } else if ([newState isEqualToString:@"active"]) {
        [self resumeMining];
        [self loadUserState];
    }
}

- (void)pauseMining {
    if (self.miningTimer) {
        [self.miningTimer invalidate];
        self.miningTimer = nil;
    }
    
    self.userState[@"lastPauseTime"] = @([[NSDate date] timeIntervalSince1970]);
}

- (void)resumeMining {
    if (self.isMiningActive) {
        // Calculate offline mining rewards (limited)
        NSNumber *lastPause = self.userState[@"lastPauseTime"];
        if (lastPause) {
            double offlineTime = [[NSDate date] timeIntervalSince1970] - [lastPause doubleValue];
            double maxOfflineHours = 24.0; // 24 hour max offline mining
            double offlineHours = MIN(offlineTime / 3600.0, maxOfflineHours);
            
            double offlineRewards = offlineHours * [[self calculateMiningRate] doubleValue] * 0.5; // 50% offline rate
            
            NSNumber *currentTotal = self.userState[@"totalMined"] ?: @0;
            self.userState[@"totalMined"] = @([currentTotal doubleValue] + offlineRewards);
            
            [self sendEventToReactNative:@"offlineMiningRewards" body:@{
                @"rewards": @(offlineRewards),
                @"offlineHours": @(offlineHours)
            }];
        }
        
        [self initiateMining];
    }
}

- (void)saveUserState {
    NSData *data = [NSJSONSerialization dataWithJSONObject:self.userState options:0 error:nil];
    if (data) {
        [[NSUserDefaults standardUserDefaults] setObject:data forKey:@"FinovaUserState"];
        [[NSUserDefaults standardUserDefaults] synchronize];
    }
}

- (void)loadUserState {
    NSData *data = [[NSUserDefaults standardUserDefaults] objectForKey:@"FinovaUserState"];
    if (data) {
        NSDictionary *savedState = [NSJSONSerialization JSONObjectWithData:data options:0 error:nil];
        if (savedState) {
            [self.userState addEntriesFromDictionary:savedState];
        }
    }
}

#pragma mark - Memory Management

- (void)dealloc {
    [self stopMiningProcess];
    [self saveUserState];
    
    [[NSNotificationCenter defaultCenter] removeObserver:self];
    
    RCTLogInfo(@"Finova SDK dealloc");
}

@end

#pragma mark - Extension Categories

@implementation FinovaReactNative (SecurityExtensions)

- (BOOL)detectJailbreak {
    // Check for common jailbreak indicators
    NSArray *jailbreakPaths = @[
        @"/Applications/Cydia.app",
        @"/Library/MobileSubstrate/MobileSubstrate.dylib",
        @"/bin/bash",
        @"/usr/sbin/sshd",
        @"/etc/apt",
        @"/private/var/lib/apt/"
    ];
    
    for (NSString *path in jailbreakPaths) {
        if ([[NSFileManager defaultManager] fileExistsAtPath:path]) {
            return YES;
        }
    }
    
    // Check if we can write to system directories
    NSString *testFile = @"/private/jailbreak_test.txt";
    NSError *error;
    [@"test" writeToFile:testFile atomically:YES encoding:NSUTF8StringEncoding error:&error];
    
    if (!error) {
        [[NSFileManager defaultManager] removeItemAtPath:testFile error:nil];
        return YES;
    }
    
    return NO;
}

- (BOOL)detectDebugger {
    // Check for debugger attachment
    int junk;
    int mib[4];
    struct kinfo_proc info;
    size_t size;
    
    mib[0] = CTL_KERN;
    mib[1] = KERN_PROC;
    mib[2] = KERN_PROC_PID;
    mib[3] = getpid();
    
    size = sizeof(info);
    junk = sysctl(mib, sizeof(mib) / sizeof(*mib), &info, &size, NULL, 0);
    
    return (info.kp_proc.p_flag & P_TRACED) != 0;
}

- (NSString *)getNetworkSecurityToken {
    // Generate network security token for API calls
    NSString *deviceId = self.deviceFingerprint;
    NSString *timestamp = [NSString stringWithFormat:@"%.0f", [[NSDate date] timeIntervalSince1970]];
    NSString *nonce = [[NSUUID UUID] UUIDString];
    
    NSString *tokenData = [NSString stringWithFormat:@"%@:%@:%@", deviceId, timestamp, nonce];
    return [self sha256:tokenData];
}

@end
