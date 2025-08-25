// finova-net/finova/mobile-sdk/ios/FinovaSDK/Sources/Utils/Constants.swift

//
//  Constants.swift
//  FinovaSDK
//
//  Created by Finova Network Team
//  Copyright © 2025 Finova Network. All rights reserved.
//

import Foundation
import UIKit

// MARK: - API Constants
public struct APIConstants {
    // Base URLs
    public static let baseURL = "https://api.finova.network"
    public static let testnetURL = "https://testnet-api.finova.network"
    public static let devnetURL = "https://devnet-api.finova.network"
    
    // WebSocket URLs
    public static let websocketURL = "wss://ws.finova.network"
    public static let testnetWebsocketURL = "wss://testnet-ws.finova.network"
    
    // API Versions
    public static let currentAPIVersion = "v1"
    public static let supportedAPIVersions = ["v1"]
    
    // Endpoints
    public struct Endpoints {
        public static let auth = "/auth"
        public static let user = "/user"
        public static let mining = "/mining"
        public static let xp = "/xp"
        public static let referral = "/referral"
        public static let nft = "/nft"
        public static let social = "/social"
        public static let staking = "/staking"
        public static let guild = "/guild"
        public static let marketplace = "/marketplace"
        public static let analytics = "/analytics"
        public static let kyc = "/kyc"
    }
    
    // Request timeouts (seconds)
    public static let defaultTimeout: TimeInterval = 30
    public static let uploadTimeout: TimeInterval = 120
    public static let downloadTimeout: TimeInterval = 60
    
    // Rate limiting
    public static let maxRequestsPerMinute = 100
    public static let maxConcurrentRequests = 10
}

// MARK: - Blockchain Constants
public struct BlockchainConstants {
    // Solana Network
    public static let solanaMainnetRPC = "https://api.mainnet-beta.solana.com"
    public static let solanaTestnetRPC = "https://api.testnet.solana.com"
    public static let solanaDevnetRPC = "https://api.devnet.solana.com"
    
    // Program IDs (Mainnet)
    public static let finovaCoreProgram = "FinovaCoreProgramID123456789"
    public static let finovaTokenProgram = "FinovaTokenProgramID123456789"
    public static let finovaNFTProgram = "FinovaNFTProgramID123456789"
    public static let finovaDeFiProgram = "FinovaDeFiProgramID123456789"
    public static let finovaBridgeProgram = "FinovaBridgeProgramID123456789"
    public static let finovaOracleProgram = "FinovaOracleProgramID123456789"
    
    // Token Addresses
    public static let finTokenMint = "FinTokenMintAddress123456789"
    public static let sFinTokenMint = "sFinTokenMintAddress123456789"
    public static let usdFinTokenMint = "USDFinTokenMintAddress123456789"
    public static let sUsdFinTokenMint = "sUSDFinTokenMintAddress123456789"
    
    // Transaction settings
    public static let defaultSlippage: Double = 0.005 // 0.5%
    public static let maxSlippage: Double = 0.05 // 5%
    public static let defaultPriorityFee: UInt64 = 1000 // lamports
    public static let confirmationTimeout: TimeInterval = 60
}

// MARK: - Mining Constants
public struct MiningConstants {
    // Base mining rates per phase (FIN/hour)
    public static let phase1BaseRate: Double = 0.1
    public static let phase2BaseRate: Double = 0.05
    public static let phase3BaseRate: Double = 0.025
    public static let phase4BaseRate: Double = 0.01
    
    // Phase thresholds (user count)
    public static let phase1Threshold = 100_000
    public static let phase2Threshold = 1_000_000
    public static let phase3Threshold = 10_000_000
    
    // Finizen bonuses
    public static let maxFinizenBonus: Double = 2.0
    public static let minFinizenBonus: Double = 1.0
    
    // Referral bonuses
    public static let referralBonusRate: Double = 0.1
    public static let maxReferralBonus: Double = 3.5
    
    // Security bonuses
    public static let kycVerifiedBonus: Double = 1.2
    public static let nonKycPenalty: Double = 0.8
    
    // Regression parameters
    public static let regressionCoefficient: Double = 0.001
    public static let maxDailyMining: [String: Double] = [
        "phase1": 4.8,
        "phase2": 1.8,
        "phase3": 0.72,
        "phase4": 0.24
    ]
    
    // Mining session limits
    public static let minMiningInterval: TimeInterval = 3600 // 1 hour
    public static let maxMiningSessionDuration: TimeInterval = 86400 // 24 hours
    public static let miningClaimWindow: TimeInterval = 604800 // 7 days
}

// MARK: - XP System Constants
public struct XPConstants {
    // Base XP values
    public static let originalPostXP = 50
    public static let photoPostXP = 75
    public static let videoContentXP = 150
    public static let storyPostXP = 25
    public static let meaningfulCommentXP = 25
    public static let likeReactXP = 5
    public static let shareRepostXP = 15
    public static let followSubscribeXP = 20
    public static let dailyLoginXP = 10
    public static let dailyQuestXP = 100
    public static let milestoneXP = 500
    public static let viralContentXP = 1000
    
    // Daily limits
    public static let maxPostsPerDay = 20
    public static let maxVideosPerDay = 10
    public static let maxStoriesPerDay = 50
    public static let maxCommentsPerDay = 100
    public static let maxLikesPerDay = 200
    public static let maxSharesPerDay = 50
    public static let maxFollowsPerDay = 25
    public static let maxDailyQuestsPerDay = 3
    
    // Platform multipliers
    public static let platformMultipliers: [String: Double] = [
        "tiktok": 1.3,
        "instagram": 1.2,
        "youtube": 1.4,
        "facebook": 1.1,
        "twitter": 1.2,
        "default": 1.0
    ]
    
    // Quality score ranges
    public static let minQualityScore: Double = 0.5
    public static let maxQualityScore: Double = 2.0
    public static let baseQualityScore: Double = 1.0
    
    // Level progression
    public static let levelProgressionCoeff: Double = 0.01
    public static let maxXPLevel = 200
    
    // Level thresholds
    public static let levelThresholds: [Int] = [
        0, 100, 250, 500, 1000, 2000, 3500, 5500, 8000, 11000, 15000, // 1-11
        20000, 26000, 33000, 41000, 50000, 60000, 71000, 83000, 96000, // 12-20
        110000, 125000, 141000, 158000, 176000, 195000, 215000, 236000, 258000, 281000 // 21-30
    ]
    
    // Mining multipliers by level range
    public static let levelMiningMultipliers: [ClosedRange<Int>: ClosedRange<Double>] = [
        1...10: 1.0...1.2,
        11...25: 1.3...1.8,
        26...50: 1.9...2.5,
        51...75: 2.6...3.2,
        76...100: 3.3...4.0,
        101...200: 4.1...5.0
    ]
}

// MARK: - Referral System Constants
public struct ReferralConstants {
    // RP earning structure
    public static let signupRP = 50
    public static let kycCompleteRP = 100
    public static let firstMiningRP = 25
    
    // Network bonuses
    public static let networkBonusThresholds = [10, 25, 50, 100]
    public static let networkBonusRP = [500, 1500, 5000, 15000]
    public static let networkBonusMultipliers = [0.5, 1.0, 1.5, 2.0]
    
    // Referral percentages
    public static let level1Percentage: Double = 0.10 // 10%
    public static let level2Percentage: Double = 0.05 // 5%
    public static let level3Percentage: Double = 0.03 // 3%
    
    // RP tier thresholds
    public static let rpTierThresholds: [String: ClosedRange<Int>] = [
        "explorer": 0...999,
        "connector": 1000...4999,
        "influencer": 5000...14999,
        "leader": 15000...49999,
        "ambassador": 50000...Int.max
    ]
    
    // Tier benefits
    public static let tierMiningBonuses: [String: Double] = [
        "explorer": 0.0,
        "connector": 0.2,
        "influencer": 0.5,
        "leader": 1.0,
        "ambassador": 2.0
    ]
    
    // Network regression
    public static let networkRegressionCoeff: Double = 0.0001
    public static let minNetworkQuality: Double = 0.1
    public static let maxNetworkMultiplier: Double = 30.0
}

// MARK: - NFT Constants
public struct NFTConstants {
    // Special card categories
    public enum CardCategory: String, CaseIterable {
        case miningBoost = "mining_boost"
        case xpAccelerator = "xp_accelerator"
        case referralPower = "referral_power"
        case profileBadge = "profile_badge"
        case achievement = "achievement"
    }
    
    // Card rarities
    public enum CardRarity: String, CaseIterable {
        case common = "common"
        case uncommon = "uncommon"
        case rare = "rare"
        case epic = "epic"
        case legendary = "legendary"
    }
    
    // Rarity bonuses
    public static let rarityBonuses: [CardRarity: Double] = [
        .common: 0.0,
        .uncommon: 0.05,
        .rare: 0.10,
        .epic: 0.20,
        .legendary: 0.35
    ]
    
    // Card prices (in FIN)
    public static let cardPrices: [String: Double] = [
        "double_mining": 50,
        "triple_mining": 150,
        "mining_frenzy": 500,
        "eternal_miner": 2000,
        "xp_double": 40,
        "streak_saver": 80,
        "level_rush": 120,
        "xp_magnet": 300,
        "referral_boost": 60,
        "network_amplifier": 200,
        "ambassador_pass": 400,
        "network_king": 1000
    ]
    
    // Card durations (in seconds)
    public static let cardDurations: [String: TimeInterval] = [
        "double_mining": 86400, // 24 hours
        "triple_mining": 43200, // 12 hours
        "mining_frenzy": 14400, // 4 hours
        "eternal_miner": 2592000, // 30 days
        "xp_double": 86400, // 24 hours
        "streak_saver": 604800, // 7 days
        "xp_magnet": 172800, // 48 hours
        "referral_boost": 604800, // 7 days
        "network_amplifier": 86400, // 24 hours
        "ambassador_pass": 172800, // 48 hours
        "network_king": 43200 // 12 hours
    ]
    
    // Synergy system
    public static let synergyBonus: Double = 0.1
    public static let sameCategoryBonus: Double = 0.15
    public static let allCategoriesBonus: Double = 0.30
    public static let maxActiveCards = 5
}

// MARK: - Staking Constants
public struct StakingConstants {
    // Staking tiers (in FIN)
    public static let stakingTiers: [ClosedRange<Double>] = [
        100...499,
        500...999,
        1000...4999,
        5000...9999,
        10000...Double.greatestFiniteMagnitude
    ]
    
    // APY rates
    public static let stakingAPYs: [Double] = [0.08, 0.10, 0.12, 0.14, 0.15]
    
    // Mining boosts
    public static let stakingMiningBoosts: [Double] = [0.20, 0.35, 0.50, 0.75, 1.00]
    
    // XP multipliers
    public static let stakingXPMultipliers: [Double] = [0.10, 0.20, 0.30, 0.50, 0.75]
    
    // RP bonuses
    public static let stakingRPBonuses: [Double] = [0.05, 0.10, 0.20, 0.35, 0.50]
    
    // Staking parameters
    public static let minStakingAmount: Double = 100
    public static let maxStakingAmount: Double = 1_000_000
    public static let stakingCooldownPeriod: TimeInterval = 604800 // 7 days
    public static let rewardClaimInterval: TimeInterval = 86400 // 24 hours
    
    // Loyalty bonuses (per month)
    public static let loyaltyBonusRate: Double = 0.05
    public static let maxLoyaltyBonus: Double = 1.0
    
    // Activity bonuses
    public static let activityBonusRate: Double = 0.1
    public static let maxActivityBonus: Double = 2.0
}

// MARK: - Guild Constants
public struct GuildConstants {
    // Guild sizes
    public static let minGuildSize = 10
    public static let maxGuildSize = 50
    public static let minLevelRequirement = 11 // Silver level
    
    // Competition types
    public enum CompetitionType: String, CaseIterable {
        case dailyChallenge = "daily_challenge"
        case weeklyWar = "weekly_war"
        case monthlyChampionship = "monthly_championship"
        case seasonalLeague = "seasonal_league"
    }
    
    // Competition durations (in seconds)
    public static let competitionDurations: [CompetitionType: TimeInterval] = [
        .dailyChallenge: 86400, // 24 hours
        .weeklyWar: 604800, // 7 days
        .monthlyChampionship: 2592000, // 30 days
        .seasonalLeague: 7776000 // 90 days
    ]
    
    // Reward multipliers
    public static let guildXPBonus: Double = 0.20
    public static let guildMiningBonus: Double = 0.30
    public static let guildRPBonus: Double = 0.15
    
    // Guild roles
    public enum GuildRole: String, CaseIterable {
        case member = "member"
        case officer = "officer"
        case master = "master"
    }
}

// MARK: - Security Constants
public struct SecurityConstants {
    // Biometric settings
    public static let maxBiometricAttempts = 3
    public static let biometricCooldownPeriod: TimeInterval = 300 // 5 minutes
    
    // Session management
    public static let sessionTimeout: TimeInterval = 3600 // 1 hour
    public static let maxActiveSessions = 3
    public static let tokenRefreshThreshold: TimeInterval = 300 // 5 minutes before expiry
    
    // Anti-bot thresholds
    public static let minHumanProbability: Double = 0.7
    public static let suspiciousActivityThreshold: Double = 0.3
    public static let botPenaltyFactor: Double = 0.1
    
    // Rate limiting
    public static let maxLoginAttempts = 5
    public static let loginCooldownPeriod: TimeInterval = 900 // 15 minutes
    public static let maxAPICallsPerMinute = 60
    
    // Encryption
    public static let aesKeySize = 256
    public static let rsaKeySize = 2048
    public static let hashIterations = 100000
}

// MARK: - UI Constants
public struct UIConstants {
    // Colors (hex values)
    public static let primaryColor = "#6366F1" // Indigo
    public static let secondaryColor = "#10B981" // Emerald
    public static let accentColor = "#F59E0B" // Amber
    public static let errorColor = "#EF4444" // Red
    public static let warningColor = "#F97316" // Orange
    public static let successColor = "#10B981" // Green
    public static let backgroundColor = "#F9FAFB" // Gray 50
    public static let surfaceColor = "#FFFFFF" // White
    public static let textPrimaryColor = "#111827" // Gray 900
    public static let textSecondaryColor = "#6B7280" // Gray 500
    
    // Gradients
    public static let primaryGradient = ["#6366F1", "#8B5CF6"] // Indigo to Purple
    public static let miningGradient = ["#10B981", "#059669"] // Emerald gradient
    public static let xpGradient = ["#F59E0B", "#D97706"] // Amber gradient
    public static let rpGradient = ["#8B5CF6", "#7C3AED"] // Purple gradient
    
    // Animations
    public static let defaultAnimationDuration: TimeInterval = 0.3
    public static let quickAnimationDuration: TimeInterval = 0.15
    public static let slowAnimationDuration: TimeInterval = 0.5
    
    // Spacing
    public static let extraSmallSpacing: CGFloat = 4
    public static let smallSpacing: CGFloat = 8
    public static let mediumSpacing: CGFloat = 16
    public static let largeSpacing: CGFloat = 24
    public static let extraLargeSpacing: CGFloat = 32
    
    // Corner radius
    public static let smallRadius: CGFloat = 8
    public static let mediumRadius: CGFloat = 12
    public static let largeRadius: CGFloat = 16
    public static let extraLargeRadius: CGFloat = 24
    
    // Shadow
    public static let shadowOpacity: Float = 0.1
    public static let shadowOffset = CGSize(width: 0, height: 2)
    public static let shadowRadius: CGFloat = 4
}

// MARK: - Analytics Constants
public struct AnalyticsConstants {
    // Event categories
    public enum EventCategory: String, CaseIterable {
        case user = "user"
        case mining = "mining"
        case xp = "xp"
        case referral = "referral"
        case nft = "nft"
        case social = "social"
        case staking = "staking"
        case guild = "guild"
    }
    
    // Event actions
    public enum EventAction: String, CaseIterable {
        case view = "view"
        case tap = "tap"
        case create = "create"
        case update = "update"
        case delete = "delete"
        case share = "share"
        case purchase = "purchase"
        case claim = "claim"
        case stake = "stake"
        case unstake = "unstake"
    }
    
    // Batch settings
    public static let maxBatchSize = 50
    public static let batchTimeout: TimeInterval = 30
    public static let maxRetries = 3
    public static let retryDelay: TimeInterval = 1
}

// MARK: - Error Constants
public struct ErrorConstants {
    // Error domains
    public static let finovaSDKDomain = "com.finova.sdk"
    public static let networkDomain = "com.finova.network"
    public static let blockchainDomain = "com.finova.blockchain"
    public static let authDomain = "com.finova.auth"
    
    // Error codes
    public enum ErrorCode: Int, CaseIterable {
        case unknown = 0
        case networkError = 1000
        case authenticationError = 1001
        case authorizationError = 1002
        case validationError = 1003
        case blockchainError = 2000
        case transactionError = 2001
        case insufficientFunds = 2002
        case miningError = 3000
        case xpError = 3001
        case referralError = 3002
        case nftError = 3003
        case stakingError = 3004
        case guildError = 3005
    }
}

// MARK: - Configuration Constants
public struct ConfigConstants {
    // Environment
    public enum Environment: String, CaseIterable {
        case development = "development"
        case staging = "staging"
        case testnet = "testnet"
        case mainnet = "mainnet"
    }
    
    // Default configuration
    public static let defaultEnvironment: Environment = .testnet
    public static let enableLogging = true
    public static let enableAnalytics = true
    public static let enableCrashReporting = true
    
    // Cache settings
    public static let cacheSize: Int = 100 * 1024 * 1024 // 100MB
    public static let cacheExpiry: TimeInterval = 3600 // 1 hour
    public static let maxCacheAge: TimeInterval = 86400 // 24 hours
    
    // Network settings
    public static let maxRetryAttempts = 3
    public static let retryDelay: TimeInterval = 1
    public static let connectionTimeout: TimeInterval = 10
}

// MARK: - Version Constants
public struct VersionConstants {
    public static let sdkVersion = "1.0.0"
    public static let apiVersion = "1.0"
    public static let minIOSVersion = "14.0"
    public static let buildNumber = "1"
    
    // Compatibility
    public static let minBackendVersion = "1.0.0"
    public static let minSmartContractVersion = "1.0.0"
    
    // Update settings
    public static let forceUpdateRequired = false
    public static let softUpdateAvailable = false
    public static let updateCheckInterval: TimeInterval = 86400 // 24 hours
}

// MARK: - Device Constants
public struct DeviceConstants {
    // Requirements
    public static let minRAM: UInt64 = 2 * 1024 * 1024 * 1024 // 2GB
    public static let minStorage: UInt64 = 100 * 1024 * 1024 // 100MB
    
    // Capabilities
    public static let requiresBiometrics = false
    public static let requiresCamera = false
    public static let requiresMicrophone = false
    public static let requiresLocation = false
    
    // Performance
    public static let maxConcurrentOperations = 5
    public static let backgroundTaskTimeout: TimeInterval = 30
    public static let foregroundTaskTimeout: TimeInterval = 60
}

// MARK: - Notification Constants
public struct NotificationConstants {
    // Categories
    public enum Category: String, CaseIterable {
        case mining = "mining"
        case xp = "xp"
        case referral = "referral"
        case achievement = "achievement"
        case guild = "guild"
        case marketplace = "marketplace"
        case system = "system"
    }
    
    // Types
    public enum NotificationType: String, CaseIterable {
        case miningReady = "mining_ready"
        case miningComplete = "mining_complete"
        case levelUp = "level_up"
        case newReferral = "new_referral"
        case achievementUnlocked = "achievement_unlocked"
        case guildInvite = "guild_invite"
        case nftReceived = "nft_received"
        case maintenance = "maintenance"
    }
    
    // Settings
    public static let defaultEnabled = true
    public static let soundEnabled = true
    public static let vibrationEnabled = true
    public static let badgeEnabled = true
}
