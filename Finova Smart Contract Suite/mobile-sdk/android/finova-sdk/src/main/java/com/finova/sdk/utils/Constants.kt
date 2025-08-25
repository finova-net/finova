package com.finova.sdk.utils

/**
 * Finova Network Android SDK Constants
 * 
 * Contains all constants for the Finova Network mobile SDK including:
 * - API endpoints and configuration
 * - Mining and reward system parameters
 * - XP system constants
 * - Referral Point (RP) system values
 * - Token economics parameters
 * - NFT and special card configurations
 * - Security and validation constants
 * 
 * @version 3.0
 * @author Finova Network Team
 * @since 2025-07-25
 */
object Constants {
    
    // ================================
    // NETWORK & API CONFIGURATION
    // ================================
    
    object Network {
        const val BASE_URL_PRODUCTION = "https://api.finova.network/v1/"
        const val BASE_URL_STAGING = "https://staging-api.finova.network/v1/"
        const val BASE_URL_DEVELOPMENT = "https://dev-api.finova.network/v1/"
        
        const val WEBSOCKET_URL_PROD = "wss://ws.finova.network"
        const val WEBSOCKET_URL_STAGING = "wss://staging-ws.finova.network"
        const val WEBSOCKET_URL_DEV = "wss://dev-ws.finova.network"
        
        const val SOLANA_RPC_MAINNET = "https://api.mainnet-beta.solana.com"
        const val SOLANA_RPC_DEVNET = "https://api.devnet.solana.com"
        const val SOLANA_RPC_TESTNET = "https://api.testnet.solana.com"
        
        const val TIMEOUT_CONNECT = 30L // seconds
        const val TIMEOUT_READ = 60L // seconds
        const val TIMEOUT_WRITE = 30L // seconds
        
        const val MAX_RETRY_ATTEMPTS = 3
        const val RETRY_DELAY_MS = 1000L
        const val EXPONENTIAL_BACKOFF_MULTIPLIER = 2.0
    }
    
    object Headers {
        const val AUTHORIZATION = "Authorization"
        const val CONTENT_TYPE = "Content-Type"
        const val USER_AGENT = "User-Agent"
        const val API_VERSION = "X-API-Version"
        const val CLIENT_ID = "X-Client-ID"
        const val SESSION_ID = "X-Session-ID"
        const val DEVICE_ID = "X-Device-ID"
        const val APP_VERSION = "X-App-Version"
        const val PLATFORM = "X-Platform"
        
        const val CONTENT_TYPE_JSON = "application/json"
        const val USER_AGENT_VALUE = "FinovaSDK-Android/3.0"
        const val API_VERSION_VALUE = "v1"
        const val PLATFORM_VALUE = "android"
    }
    
    // ================================
    // MINING SYSTEM CONSTANTS
    // ================================
    
    object Mining {
        // Phase-based mining rates (per hour in FIN tokens)
        const val PHASE_1_BASE_RATE = 0.1 // Finizen Phase (0-100K users)
        const val PHASE_2_BASE_RATE = 0.05 // Growth Phase (100K-1M users)
        const val PHASE_3_BASE_RATE = 0.025 // Maturity Phase (1M-10M users)
        const val PHASE_4_BASE_RATE = 0.01 // Stability Phase (10M+ users)
        
        // Finizen bonus multipliers
        const val PHASE_1_FINIZEN_BONUS = 2.0
        const val PHASE_2_FINIZEN_BONUS = 1.5
        const val PHASE_3_FINIZEN_BONUS = 1.2
        const val PHASE_4_FINIZEN_BONUS = 1.0
        
        // User thresholds for phase transitions
        const val PHASE_1_USER_THRESHOLD = 100_000L
        const val PHASE_2_USER_THRESHOLD = 1_000_000L
        const val PHASE_3_USER_THRESHOLD = 10_000_000L
        
        // Security and KYC bonuses
        const val KYC_VERIFIED_BONUS = 1.2
        const val KYC_UNVERIFIED_PENALTY = 0.8
        
        // Referral bonus parameters
        const val REFERRAL_BONUS_PER_ACTIVE = 0.1
        const val MAX_REFERRAL_BONUS = 3.5
        
        // Regression parameters
        const val REGRESSION_COEFFICIENT = 0.001
        const val MIN_REGRESSION_FACTOR = 0.000045
        
        // Daily mining limits
        const val PHASE_1_DAILY_LIMIT = 4.8
        const val PHASE_2_DAILY_LIMIT = 1.8
        const val PHASE_3_DAILY_LIMIT = 0.72
        const val PHASE_4_DAILY_LIMIT = 0.24
        
        // Mining session parameters
        const val MINING_SESSION_DURATION_HOURS = 24
        const val MINING_SESSION_EXTENSION_HOURS = 12
        const val AUTO_MINING_STOP_HOURS = 24
        
        // Activity boosters
        const val DAILY_POST_BOOST = 0.2 // +20%
        const val DAILY_QUEST_BOOST = 0.5 // +50%
        const val REFERRAL_KYC_BOOST = 1.0 // +100%
        const val GUILD_PARTICIPATION_BOOST = 0.3 // +30%
        
        // Boost durations (in hours)
        const val DAILY_POST_BOOST_DURATION = 24
        const val DAILY_QUEST_BOOST_DURATION = 12
        const val REFERRAL_KYC_BOOST_DURATION = 48
        
        // Maximum stackable boosts
        const val MAX_DAILY_POST_STACKS = 3
        const val MAX_REFERRAL_KYC_STACKS = 5
    }
    
    // ================================
    // EXPERIENCE POINTS (XP) SYSTEM
    // ================================
    
    object XP {
        // Base XP values for different activities
        const val ORIGINAL_POST_BASE = 50
        const val PHOTO_POST_BASE = 75
        const val VIDEO_POST_BASE = 150
        const val STORY_STATUS_BASE = 25
        const val MEANINGFUL_COMMENT_BASE = 25
        const val LIKE_REACT_BASE = 5
        const val SHARE_REPOST_BASE = 15
        const val FOLLOW_SUBSCRIBE_BASE = 20
        const val DAILY_LOGIN_BASE = 10
        const val DAILY_QUEST_BASE = 100
        const val MILESTONE_ACHIEVEMENT_BASE = 500
        const val VIRAL_CONTENT_BASE = 1000
        
        // Daily activity limits
        const val PHOTO_POST_DAILY_LIMIT = 20
        const val VIDEO_POST_DAILY_LIMIT = 10
        const val STORY_STATUS_DAILY_LIMIT = 50
        const val COMMENT_DAILY_LIMIT = 100
        const val LIKE_REACT_DAILY_LIMIT = 200
        const val SHARE_REPOST_DAILY_LIMIT = 50
        const val FOLLOW_SUBSCRIBE_DAILY_LIMIT = 25
        const val DAILY_QUEST_LIMIT = 3
        
        // Platform multipliers
        const val TIKTOK_MULTIPLIER = 1.3
        const val INSTAGRAM_MULTIPLIER = 1.2
        const val YOUTUBE_MULTIPLIER = 1.4
        const val FACEBOOK_MULTIPLIER = 1.1
        const val TWITTER_X_MULTIPLIER = 1.2
        const val DEFAULT_PLATFORM_MULTIPLIER = 1.0
        
        // Quality score ranges
        const val MIN_QUALITY_SCORE = 0.5
        const val MAX_QUALITY_SCORE = 2.0
        const val DEFAULT_QUALITY_SCORE = 1.0
        
        // Streak bonuses
        const val MAX_STREAK_BONUS = 3.0
        const val STREAK_BONUS_INCREMENT = 0.1 // +10% per day
        
        // Level progression constants
        const val LEVEL_PROGRESSION_COEFFICIENT = 0.01
        
        // Viral content threshold
        const val VIRAL_CONTENT_THRESHOLD = 1000 // views/engagements
        const val VIRAL_CONTENT_MULTIPLIER = 2.0
        
        // XP level thresholds and rewards
        val LEVEL_THRESHOLDS = mapOf(
            1 to 0L, 2 to 100L, 3 to 250L, 4 to 450L, 5 to 700L,
            6 to 1000L, 7 to 1350L, 8 to 1750L, 9 to 2200L, 10 to 2700L,
            11 to 3250L, 15 to 5000L, 20 to 8000L, 25 to 12000L,
            30 to 17000L, 35 to 23000L, 40 to 30000L, 45 to 38000L,
            50 to 47000L, 60 to 68000L, 70 to 92000L, 80 to 120000L,
            90 to 152000L, 100 to 188000L
        )
        
        // Mining multipliers by level range
        val MINING_MULTIPLIERS = mapOf(
            1..10 to 1.0..1.2,
            11..25 to 1.3..1.8,
            26..50 to 1.9..2.5,
            51..75 to 2.6..3.2,
            76..100 to 3.3..4.0,
            101..Int.MAX_VALUE to 4.1..5.0
        )
        
        // Badge tiers
        enum class BadgeTier(val levelRange: IntRange, val name: String) {
            BRONZE(1..10, "Bronze"),
            SILVER(11..25, "Silver"),
            GOLD(26..50, "Gold"),
            PLATINUM(51..75, "Platinum"),
            DIAMOND(76..100, "Diamond"),
            MYTHIC(101..Int.MAX_VALUE, "Mythic")
        }
    }
    
    // ================================
    // REFERRAL POINTS (RP) SYSTEM
    // ================================
    
    object ReferralPoints {
        // Base RP rewards
        const val SIGNUP_WITH_CODE = 50
        const val COMPLETE_KYC = 100
        const val FIRST_FIN_EARNED = 25
        
        // Ongoing activity percentages
        const val REFERRAL_DAILY_MINING_PERCENT = 0.10 // 10%
        const val REFERRAL_XP_GAINS_PERCENT = 0.05 // 5%
        const val REFERRAL_ACHIEVEMENTS = 50
        
        // Network milestone bonuses
        const val MILESTONE_10_ACTIVE = 500
        const val MILESTONE_25_ACTIVE = 1500
        const val MILESTONE_50_ACTIVE = 5000
        const val MILESTONE_100_ACTIVE = 15000
        
        // Network effect multipliers
        const val LEVEL_1_MULTIPLIER = 1.0
        const val LEVEL_2_MULTIPLIER = 0.3
        const val LEVEL_3_MULTIPLIER = 0.1
        
        // RP tier thresholds and benefits
        enum class RPTier(
            val minRP: Int,
            val maxRP: Int,
            val tierName: String,
            val miningBonus: Double,
            val referralBonus: String,
            val networkCap: Int
        ) {
            EXPLORER(0, 999, "Explorer", 0.0, "10% of L1", 10),
            CONNECTOR(1000, 4999, "Connector", 0.2, "15% of L1, 5% of L2", 25),
            INFLUENCER(5000, 14999, "Influencer", 0.5, "20% of L1, 8% of L2, 3% of L3", 50),
            LEADER(15000, 49999, "Leader", 1.0, "25% of L1, 10% of L2, 5% of L3", 100),
            AMBASSADOR(50000, Int.MAX_VALUE, "Ambassador", 2.0, "30% of L1, 15% of L2, 8% of L3", Int.MAX_VALUE)
        }
        
        // Network quality calculation constants
        const val ACTIVITY_THRESHOLD_DAYS = 30
        const val MIN_NETWORK_QUALITY_SCORE = 0.1
        const val MAX_NETWORK_QUALITY_SCORE = 1.0
        
        // Regression parameters for RP
        const val NETWORK_REGRESSION_COEFFICIENT = 0.0001
    }
    
    // ================================
    // TOKEN ECONOMICS
    // ================================
    
    object Tokens {
        // Token symbols and decimals
        const val FIN_SYMBOL = "FIN"
        const val SFIN_SYMBOL = "sFIN"
        const val USDFIN_SYMBOL = "USDfin"
        const val SUSDFIN_SYMBOL = "sUSDfin"
        
        const val FIN_DECIMALS = 9
        const val TOKEN_DECIMALS = 9
        
        // Maximum supply
        const val MAX_SUPPLY_FIN = 100_000_000_000L // 100 billion
        
        // Distribution percentages
        const val COMMUNITY_MINING_PERCENT = 0.50 // 50%
        const val TEAM_ALLOCATION_PERCENT = 0.20 // 20%
        const val INVESTORS_ALLOCATION_PERCENT = 0.15 // 15%
        const val PUBLIC_SALE_PERCENT = 0.10 // 10%
        const val TREASURY_PERCENT = 0.05 // 5%
        
        // Staking APY ranges
        const val MIN_STAKING_APY = 0.08 // 8%
        const val MAX_STAKING_APY = 0.15 // 15%
        const val STABLE_STAKING_APY = 0.04 // 4%
        const val STABLE_STAKING_MAX_APY = 0.08 // 8%
        
        // Transaction fees
        const val TRANSACTION_FEE_PERCENT = 0.001 // 0.1%
        const val NFT_TRADING_FEE_PERCENT = 0.025 // 2.5%
        
        // Whale tax thresholds
        const val WHALE_THRESHOLD_1 = 10_000 // 10K FIN
        const val WHALE_THRESHOLD_2 = 50_000 // 50K FIN
        const val WHALE_THRESHOLD_3 = 100_000 // 100K FIN
        
        const val WHALE_TAX_RATE_1 = 0.05 // 5%
        const val WHALE_TAX_RATE_2 = 0.10 // 10%
        const val WHALE_TAX_RATE_3 = 0.15 // 15%
    }
    
    // ================================
    // STAKING SYSTEM
    // ================================
    
    object Staking {
        // Staking tier thresholds
        const val TIER_1_MIN = 100 // 100 FIN
        const val TIER_1_MAX = 499
        const val TIER_2_MIN = 500 // 500 FIN
        const val TIER_2_MAX = 999
        const val TIER_3_MIN = 1000 // 1K FIN
        const val TIER_3_MAX = 4999
        const val TIER_4_MIN = 5000 // 5K FIN
        const val TIER_4_MAX = 9999
        const val TIER_5_MIN = 10000 // 10K FIN
        
        // Staking benefits by tier
        val STAKING_BENEFITS = mapOf(
            1 to StakingTier(0.08, 0.20, 0.10, 0.05),
            2 to StakingTier(0.10, 0.35, 0.20, 0.10),
            3 to StakingTier(0.12, 0.50, 0.30, 0.20),
            4 to StakingTier(0.14, 0.75, 0.50, 0.35),
            5 to StakingTier(0.15, 1.00, 0.75, 0.50)
        )
        
        data class StakingTier(
            val apyRate: Double,
            val miningBoost: Double,
            val xpMultiplier: Double,
            val rpBonus: Double
        )
        
        // Loyalty bonus parameters
        const val LOYALTY_BONUS_PER_MONTH = 0.05 // 5% per month
        const val MAX_LOYALTY_BONUS = 2.0 // 200% max
        
        // Activity bonus parameters
        const val ACTIVITY_BONUS_PER_SCORE = 0.1 // 10% per activity score point
        const val MAX_ACTIVITY_BONUS = 1.0 // 100% max
        
        // Unstaking periods
        const val UNSTAKING_PERIOD_DAYS = 7
        const val EMERGENCY_UNSTAKING_PENALTY = 0.10 // 10%
    }
    
    // ================================
    // NFT & SPECIAL CARDS
    // ================================
    
    object NFT {
        // Card categories
        enum class CardCategory {
            MINING_BOOST, XP_ACCELERATOR, REFERRAL_POWER, PROFILE_BADGE, ACHIEVEMENT
        }
        
        enum class CardRarity(val multiplier: Double, val bonusPercent: Double) {
            COMMON(1.0, 0.0),
            UNCOMMON(1.05, 0.05),
            RARE(1.10, 0.10),
            EPIC(1.20, 0.20),
            LEGENDARY(1.35, 0.35)
        }
        
        // Mining boost cards
        object MiningBoostCards {
            const val DOUBLE_MINING_BOOST = 1.0 // +100%
            const val DOUBLE_MINING_DURATION = 24 // hours
            const val DOUBLE_MINING_PRICE = 50 // FIN
            
            const val TRIPLE_MINING_BOOST = 2.0 // +200%
            const val TRIPLE_MINING_DURATION = 12 // hours
            const val TRIPLE_MINING_PRICE = 150 // FIN
            
            const val MINING_FRENZY_BOOST = 5.0 // +500%
            const val MINING_FRENZY_DURATION = 4 // hours
            const val MINING_FRENZY_PRICE = 500 // FIN
            
            const val ETERNAL_MINER_BOOST = 0.5 // +50%
            const val ETERNAL_MINER_DURATION = 720 // hours (30 days)
            const val ETERNAL_MINER_PRICE = 2000 // FIN
        }
        
        // XP accelerator cards
        object XPAcceleratorCards {
            const val XP_DOUBLE_BOOST = 1.0 // +100%
            const val XP_DOUBLE_DURATION = 24 // hours
            const val XP_DOUBLE_PRICE = 40 // FIN
            
            const val STREAK_SAVER_DURATION = 168 // hours (7 days)
            const val STREAK_SAVER_PRICE = 80 // FIN
            
            const val LEVEL_RUSH_XP = 500 // instant XP
            const val LEVEL_RUSH_PRICE = 120 // FIN
            
            const val XP_MAGNET_BOOST = 3.0 // +300% for viral content
            const val XP_MAGNET_DURATION = 48 // hours
            const val XP_MAGNET_PRICE = 300 // FIN
        }
        
        // Referral power cards
        object ReferralPowerCards {
            const val REFERRAL_BOOST_MULTIPLIER = 0.5 // +50%
            const val REFERRAL_BOOST_DURATION = 168 // hours (7 days)
            const val REFERRAL_BOOST_PRICE = 60 // FIN
            
            const val NETWORK_AMPLIFIER_TIER_BOOST = 2 // +2 RP tiers
            const val NETWORK_AMPLIFIER_DURATION = 24 // hours
            const val NETWORK_AMPLIFIER_PRICE = 200 // FIN
            
            const val AMBASSADOR_PASS_DURATION = 48 // hours
            const val AMBASSADOR_PASS_PRICE = 400 // FIN
            
            const val NETWORK_KING_BOOST = 1.0 // +100% from entire network
            const val NETWORK_KING_DURATION = 12 // hours
            const val NETWORK_KING_PRICE = 1000 // FIN
        }
        
        // Synergy system
        const val SYNERGY_BONUS_PER_CARD = 0.1 // 10% per additional card
        const val SAME_CATEGORY_BONUS = 0.15 // 15%
        const val ALL_CATEGORIES_BONUS = 0.30 // 30%
        
        // Special NFT bonuses
        const val FINIZEN_BADGE_MINING_BONUS = 0.25 // +25% lifetime
        const val CONTENT_KING_XP_BONUS = 0.50 // +50% XP from posts
        const val AMBASSADOR_REFERRAL_BONUS = 0.30 // +30% referral rewards
        const val DIAMOND_HANDS_STAKING_BONUS = 0.20 // +20% staking rewards
    }
    
    // ================================
    // GUILD SYSTEM
    // ================================
    
    object Guild {
        // Guild parameters
        const val MIN_GUILD_SIZE = 10
        const val MAX_GUILD_SIZE = 50
        const val MIN_LEVEL_REQUIREMENT = 11 // Silver level
        
        // Competition types and rewards
        enum class CompetitionType(val duration: Long, val rewardMultiplier: Double) {
            DAILY_CHALLENGES(24, 0.20), // 24 hours, +20% XP
            WEEKLY_WARS(168, 1.0), // 7 days, guild treasury
            MONTHLY_CHAMPIONSHIPS(720, 2.0), // 30 days, rare NFTs
            SEASONAL_LEAGUES(2160, 5.0) // 90 days, massive FIN prizes
        }
        
        // Guild roles
        enum class GuildRole(val permissions: List<String>) {
            MEMBER(listOf("participate", "vote")),
            OFFICER(listOf("participate", "vote", "invite", "moderate")),
            MASTER(listOf("participate", "vote", "invite", "moderate", "manage", "disband"))
        }
        
        // Guild bonuses
        const val GUILD_XP_BONUS = 0.15 // +15%
        const val GUILD_MINING_BONUS = 0.10 // +10%
        const val GUILD_EVENT_MULTIPLIER = 2.0 // 2x during events
    }
    
    // ================================
    // SECURITY & VALIDATION
    // ================================
    
    object Security {
        // Human verification thresholds
        const val MIN_HUMAN_PROBABILITY = 0.7
        const val SUSPICIOUS_ACTIVITY_THRESHOLD = 0.3
        const val BOT_DETECTION_THRESHOLD = 0.2
        
        // Rate limiting
        const val API_RATE_LIMIT_PER_MINUTE = 60
        const val MINING_RATE_LIMIT_PER_HOUR = 1
        const val XP_ACTION_RATE_LIMIT_PER_MINUTE = 10
        
        // Session management
        const val JWT_EXPIRY_MINUTES = 60
        const val REFRESH_TOKEN_EXPIRY_DAYS = 30
        const val SESSION_TIMEOUT_MINUTES = 30
        
        // Biometric settings
        const val BIOMETRIC_TIMEOUT_SECONDS = 30
        const val MAX_BIOMETRIC_ATTEMPTS = 3
        const val BIOMETRIC_LOCKOUT_MINUTES = 15
        
        // Encryption
        const val AES_KEY_SIZE = 256
        const val RSA_KEY_SIZE = 2048
        const val HASH_ITERATIONS = 100000
        
        // Device verification
        const val DEVICE_FINGERPRINT_EXPIRY_DAYS = 90
        const val MAX_DEVICES_PER_USER = 3
        const val DEVICE_CHANGE_COOLDOWN_HOURS = 24
        
        // Anti-bot measures
        const val PATTERN_ANALYSIS_WINDOW_HOURS = 24
        const val SUSPICIOUS_PATTERN_THRESHOLD = 5
        const val BOT_DETECTION_COOLDOWN_MINUTES = 15
        
        // Progressive difficulty scaling
        const val DIFFICULTY_BASE_MULTIPLIER = 1.0
        const val DIFFICULTY_INCREMENT_PER_1000_FIN = 1.0
        const val SUSPICIOUS_SCORE_MULTIPLIER = 2.0
        const val MAX_DIFFICULTY_MULTIPLIER = 10.0
    }
    
    // ================================
    // USER INTERFACE
    // ================================
    
    object UI {
        // Animation durations
        const val ANIMATION_DURATION_SHORT = 200L
        const val ANIMATION_DURATION_MEDIUM = 300L
        const val ANIMATION_DURATION_LONG = 500L
        
        // Refresh intervals
        const val MINING_UPDATE_INTERVAL_MS = 60000L // 1 minute
        const val XP_UPDATE_INTERVAL_MS = 30000L // 30 seconds
        const val BALANCE_UPDATE_INTERVAL_MS = 120000L // 2 minutes
        const val LEADERBOARD_UPDATE_INTERVAL_MS = 300000L // 5 minutes
        
        // Cache durations
        const val USER_DATA_CACHE_MINUTES = 15
        const val MINING_DATA_CACHE_MINUTES = 5
        const val SOCIAL_DATA_CACHE_MINUTES = 10
        const val NFT_DATA_CACHE_MINUTES = 30
        
        // Pagination
        const val DEFAULT_PAGE_SIZE = 20
        const val MAX_PAGE_SIZE = 100
        const val LEADERBOARD_PAGE_SIZE = 50
        
        // Notification settings
        const val MAX_NOTIFICATIONS_DISPLAYED = 5
        const val NOTIFICATION_AUTO_DISMISS_MS = 5000L
        const val MINING_REMINDER_INTERVAL_HOURS = 12
    }
    
    // ================================
    // SOCIAL PLATFORM INTEGRATION
    // ================================
    
    object SocialPlatforms {
        // Platform identifiers
        const val PLATFORM_INSTAGRAM = "instagram"
        const val PLATFORM_TIKTOK = "tiktok"
        const val PLATFORM_YOUTUBE = "youtube"
        const val PLATFORM_FACEBOOK = "facebook"
        const val PLATFORM_TWITTER_X = "twitter_x"
        const val PLATFORM_LINKEDIN = "linkedin"
        
        // OAuth settings
        const val OAUTH_TIMEOUT_SECONDS = 60
        const val TOKEN_REFRESH_THRESHOLD_MINUTES = 30
        const val MAX_OAUTH_RETRIES = 3
        
        // Content analysis
        const val MIN_CONTENT_LENGTH = 10 // characters
        const val MAX_CONTENT_LENGTH = 10000 // characters
        const val IMAGE_MAX_SIZE_MB = 50
        const val VIDEO_MAX_SIZE_MB = 500
        const val VIDEO_MAX_DURATION_MINUTES = 30
        
        // Platform-specific limits
        val PLATFORM_DAILY_LIMITS = mapOf(
            PLATFORM_INSTAGRAM to mapOf("posts" to 5, "stories" to 10, "comments" to 50),
            PLATFORM_TIKTOK to mapOf("posts" to 3, "comments" to 30),
            PLATFORM_YOUTUBE to mapOf("posts" to 2, "comments" to 25),
            PLATFORM_FACEBOOK to mapOf("posts" to 10, "comments" to 100),
            PLATFORM_TWITTER_X to mapOf("posts" to 50, "comments" to 200)
        )
    }
    
    // ================================
    // E-WALLET INTEGRATION (INDONESIA)
    // ================================
    
    object EWallet {
        // Supported e-wallets
        const val OVO = "ovo"
        const val GOPAY = "gopay"
        const val DANA = "dana"
        const val SHOPEEPAY = "shopeepay"
        const val LINKAJA = "linkaja"
        
        // Transaction limits (IDR)
        const val MIN_WITHDRAWAL_IDR = 10000 // 10K IDR
        const val MAX_WITHDRAWAL_IDR = 10000000 // 10M IDR
        const val DAILY_WITHDRAWAL_LIMIT_IDR = 20000000 // 20M IDR
        
        // Exchange rates cache
        const val EXCHANGE_RATE_CACHE_MINUTES = 5
        const val EXCHANGE_RATE_UPDATE_INTERVAL_MS = 300000L // 5 minutes
        
        // Transaction fees
        const val WITHDRAWAL_FEE_PERCENT = 0.02 // 2%
        const val MIN_WITHDRAWAL_FEE_IDR = 1000 // 1K IDR
        const val MAX_WITHDRAWAL_FEE_IDR = 50000 // 50K IDR
        
        // Verification requirements
        const val KYC_REQUIRED_FOR_WITHDRAWAL = true
        const val MIN_ACCOUNT_AGE_DAYS_FOR_WITHDRAWAL = 7
        const val MIN_ACTIVITY_SCORE_FOR_WITHDRAWAL = 100
    }
    
    // ================================
    // ERROR CODES & MESSAGES
    // ================================
    
    object ErrorCodes {
        // Network errors
        const val NETWORK_ERROR = 1000
        const val TIMEOUT_ERROR = 1001
        const val SERVER_ERROR = 1002
        const val UNAUTHORIZED_ERROR = 1003
        const val FORBIDDEN_ERROR = 1004
        const val NOT_FOUND_ERROR = 1005
        const val RATE_LIMIT_ERROR = 1006
        
        // Authentication errors
        const val INVALID_CREDENTIALS = 2000
        const val EXPIRED_TOKEN = 2001
        const val INVALID_BIOMETRIC = 2002
        const val ACCOUNT_LOCKED = 2003
        const val KYC_REQUIRED = 2004
        const val DEVICE_NOT_VERIFIED = 2005
        
        // Mining errors
        const val MINING_SESSION_EXPIRED = 3000
        const val MINING_LIMIT_REACHED = 3001
        const val INSUFFICIENT_BALANCE = 3002
        const val BOT_DETECTED = 3003
        const val QUALITY_SCORE_TOO_LOW = 3004
        const val REGRESSION_LIMIT_REACHED = 3005
        
        // XP system errors
        const val XP_DAILY_LIMIT_REACHED = 4000
        const val INVALID_ACTIVITY_TYPE = 4001
        const val CONTENT_QUALITY_REJECTED = 4002
        const val PLATFORM_NOT_CONNECTED = 4003
        
        // Referral system errors
        const val INVALID_REFERRAL_CODE = 5000
        const val REFERRAL_LIMIT_REACHED = 5001
        const val SELF_REFERRAL_NOT_ALLOWED = 5002
        const val REFERRAL_ALREADY_USED = 5003
        
        // NFT errors
        const val NFT_NOT_FOUND = 6000
        const val NFT_NOT_OWNED = 6001
        const val NFT_ALREADY_USED = 6002
        const val NFT_EXPIRED = 6003
        const val INSUFFICIENT_FIN_FOR_NFT = 6004
        
        // Guild errors
        const val GUILD_NOT_FOUND = 7000
        const val GUILD_FULL = 7001
        const val INSUFFICIENT_LEVEL_FOR_GUILD = 7002
        const val ALREADY_IN_GUILD = 7003
        const val NOT_GUILD_MEMBER = 7004
        const val INSUFFICIENT_PERMISSIONS = 7005
        
        // E-wallet errors
        const val EWALLET_NOT_CONNECTED = 8000
        const val WITHDRAWAL_LIMIT_EXCEEDED = 8001
        const val INSUFFICIENT_BALANCE_FOR_WITHDRAWAL = 8002
        const val WITHDRAWAL_FEE_TOO_HIGH = 8003
        const val EXCHANGE_RATE_UNAVAILABLE = 8004
        
        // Validation errors
        const val INVALID_INPUT = 9000
        const val MISSING_REQUIRED_FIELD = 9001
        const val INVALID_FORMAT = 9002
        const val VALUE_OUT_OF_RANGE = 9003
        const val DUPLICATE_ENTRY = 9004
    }
    
    object ErrorMessages {
        const val NETWORK_ERROR = "Network connection error. Please check your internet connection."
        const val TIMEOUT_ERROR = "Request timed out. Please try again."
        const val SERVER_ERROR = "Server error. Please try again later."
        const val UNAUTHORIZED_ERROR = "Authentication required. Please login again."
        const val RATE_LIMIT_ERROR = "Too many requests. Please wait before trying again."
        
        const val BOT_DETECTED = "Suspicious activity detected. Please verify you are human."
        const val MINING_LIMIT_REACHED = "Daily mining limit reached. Come back tomorrow!"
        const val INSUFFICIENT_BALANCE = "Insufficient FIN balance for this transaction."
        const val KYC_REQUIRED = "KYC verification required for this action."
        
        const val INVALID_REFERRAL_CODE = "Invalid referral code. Please check and try again."
        const val PLATFORM_NOT_CONNECTED = "Please connect your social media account first."
        const val GUILD_FULL = "This guild is full. Try joining another guild."
        
        const val WITHDRAWAL_LIMIT_EXCEEDED = "Withdrawal amount exceeds daily limit."
        const val EWALLET_NOT_CONNECTED = "Please connect your e-wallet first."
    }
    
    // ================================
    // FEATURE FLAGS
    // ================================
    
    object FeatureFlags {
        // Core features
        const val MINING_ENABLED = "mining_enabled"
        const val XP_SYSTEM_ENABLED = "xp_system_enabled"
        const val REFERRAL_SYSTEM_ENABLED = "referral_system_enabled"
        const val STAKING_ENABLED = "staking_enabled"
        
        // Social features
        const val SOCIAL_INTEGRATION_ENABLED = "social_integration_enabled"
        const val CONTENT_QUALITY_CHECK_ENABLED = "content_quality_check_enabled"
        const val VIRAL_BONUS_ENABLED = "viral_bonus_enabled"
        
        // Advanced features
        const val NFT_MARKETPLACE_ENABLED = "nft_marketplace_enabled"
        const val GUILD_SYSTEM_ENABLED = "guild_system_enabled"
        const val ADVANCED_ANALYTICS_ENABLED = "advanced_analytics_enabled"
        
        // Regional features
        const val INDONESIA_EWALLET_ENABLED = "indonesia_ewallet_enabled"
        const val MULTI_LANGUAGE_ENABLED = "multi_language_enabled"
        
        // Security features
        const val BIOMETRIC_LOGIN_ENABLED = "biometric_login_enabled"
        const val ADVANCED_BOT_DETECTION_ENABLED = "advanced_bot_detection_enabled"
        const val DEVICE_VERIFICATION_ENABLED = "device_verification_enabled"
        
        // Experimental features
        const val AI_CONTENT_ANALYSIS_ENABLED = "ai_content_analysis_enabled"
        const val PREDICTIVE_ANALYTICS_ENABLED = "predictive_analytics_enabled"
        const val CROSS_CHAIN_ENABLED = "cross_chain_enabled"
    }
    
    // ================================
    // ANALYTICS & TRACKING
    // ================================
    
    object Analytics {
        // Event categories
        const val CATEGORY_USER = "user"
        const val CATEGORY_MINING = "mining"
        const val CATEGORY_XP = "xp"
        const val CATEGORY_REFERRAL = "referral"
        const val CATEGORY_NFT = "nft"
        const val CATEGORY_SOCIAL = "social"
        const val CATEGORY_GUILD = "guild"
        const val CATEGORY_EWALLET = "ewallet"
        
        // User events
        const val EVENT_USER_REGISTRATION = "user_registration"
        const val EVENT_USER_LOGIN = "user_login"
        const val EVENT_KYC_COMPLETED = "kyc_completed"
        const val EVENT_PROFILE_UPDATED = "profile_updated"
        
        // Mining events
        const val EVENT_MINING_STARTED = "mining_started"
        const val EVENT_MINING_STOPPED = "mining_stopped"
        const val EVENT_MINING_BOOST_USED = "mining_boost_used"
        const val EVENT_PHASE_TRANSITION = "phase_transition"
        
        // XP events
        const val EVENT_XP_GAINED = "xp_gained"
        const val EVENT_LEVEL_UP = "level_up"
        const val EVENT_STREAK_ACHIEVED = "streak_achieved"
        const val EVENT_VIRAL_CONTENT = "viral_content"
        
        // Social events
        const val EVENT_PLATFORM_CONNECTED = "platform_connected"
        const val EVENT_CONTENT_POSTED = "content_posted"
        const val EVENT_ENGAGEMENT_RECEIVED = "engagement_received"
        
        // NFT events
        const val EVENT_NFT_PURCHASED = "nft_purchased"
        const val EVENT_NFT_USED = "nft_used"
        const val EVENT_CARD_ACTIVATED = "card_activated"
        
        // Referral events
        const val EVENT_REFERRAL_SENT = "referral_sent"
        const val EVENT_REFERRAL_COMPLETED = "referral_completed"
        const val EVENT_RP_TIER_ACHIEVED = "rp_tier_achieved"
        
        // E-wallet events
        const val EVENT_EWALLET_CONNECTED = "ewallet_connected"
        const val EVENT_WITHDRAWAL_INITIATED = "withdrawal_initiated"
        const val EVENT_WITHDRAWAL_COMPLETED = "withdrawal_completed"
    }
    
    // ================================
    // LOCALIZATION
    // ================================
    
    object Localization {
        // Supported languages
        const val LANGUAGE_ENGLISH = "en"
        const val LANGUAGE_INDONESIAN = "id"
        const val LANGUAGE_MALAY = "ms"
        const val LANGUAGE_VIETNAMESE = "vi"
        const val LANGUAGE_THAI = "th"
        const val LANGUAGE_TAGALOG = "tl"
        
        // Default language
        const val DEFAULT_LANGUAGE = LANGUAGE_ENGLISH
        
        // Currency codes
        const val CURRENCY_USD = "USD"
        const val CURRENCY_IDR = "IDR"
        const val CURRENCY_MYR = "MYR"
        const val CURRENCY_VND = "VND"
        const val CURRENCY_THB = "THB"
        const val CURRENCY_PHP = "PHP"
        
        // Date formats
        const val DATE_FORMAT_DEFAULT = "yyyy-MM-dd"
        const val DATE_FORMAT_DISPLAY = "dd MMM yyyy"
        const val DATETIME_FORMAT_FULL = "yyyy-MM-dd HH:mm:ss"
        const val TIME_FORMAT_12H = "hh:mm a"
        const val TIME_FORMAT_24H = "HH:mm"
    }
    
    // ================================
    // BLOCKCHAIN CONFIGURATION
    // ================================
    
    object Blockchain {
        // Solana program addresses (will be updated with actual deployed addresses)
        const val FINOVA_CORE_PROGRAM_ID = "FinovaCoreProgram11111111111111111111111111"
        const val FINOVA_TOKEN_PROGRAM_ID = "FinovaTokenProgram111111111111111111111111"
        const val FINOVA_NFT_PROGRAM_ID = "FinovaNFTProgram1111111111111111111111111"
        const val FINOVA_DEFI_PROGRAM_ID = "FinovaDeFiProgram111111111111111111111111"
        const val FINOVA_BRIDGE_PROGRAM_ID = "FinovaBridgeProgram1111111111111111111111"
        const val FINOVA_ORACLE_PROGRAM_ID = "FinovaOracleProgram1111111111111111111111"
        
        // Token mint addresses
        const val FIN_MINT_ADDRESS = "FinTokenMint11111111111111111111111111111"
        const val SFIN_MINT_ADDRESS = "SFinTokenMint1111111111111111111111111111"
        const val USDFIN_MINT_ADDRESS = "USDFinMint111111111111111111111111111111"
        const val SUSDFIN_MINT_ADDRESS = "SUSDFinMint11111111111111111111111111111"
        
        // Transaction confirmation levels
        const val CONFIRMATION_PROCESSED = "processed"
        const val CONFIRMATION_CONFIRMED = "confirmed"
        const val CONFIRMATION_FINALIZED = "finalized"
        
        // Default confirmation level
        const val DEFAULT_CONFIRMATION = CONFIRMATION_CONFIRMED
        
        // Transaction retry parameters
        const val MAX_TRANSACTION_RETRIES = 5
        const val TRANSACTION_RETRY_DELAY_MS = 2000L
        const val TRANSACTION_TIMEOUT_MS = 60000L
        
        // Gas and priority fee settings
        const val PRIORITY_FEE_LAMPORTS = 1000L // 0.000001 SOL
        const val MAX_COMPUTE_UNITS = 1_400_000
        const val COMPUTE_UNIT_PRICE = 1L // microlamports
        
        // Wallet connection settings
        const val WALLET_CONNECT_TIMEOUT_MS = 30000L
        const val WALLET_SIGN_TIMEOUT_MS = 60000L
        
        // Cross-chain bridge settings
        const val BRIDGE_MIN_AMOUNT = 1.0 // 1 FIN minimum
        const val BRIDGE_MAX_AMOUNT = 1000000.0 // 1M FIN maximum
        const val BRIDGE_FEE_PERCENT = 0.005 // 0.5%
        const val BRIDGE_CONFIRMATION_BLOCKS = 20
    }
    
    // ================================
    // ENVIRONMENT CONFIGURATION
    // ================================
    
    object Environment {
        const val PRODUCTION = "production"
        const val STAGING = "staging"
        const val DEVELOPMENT = "development"
        const val LOCAL = "local"
        
        // Default environment
        const val DEFAULT_ENVIRONMENT = DEVELOPMENT
        
        // Debug settings
        const val DEBUG_LOGGING_ENABLED = true
        const val VERBOSE_LOGGING_ENABLED = false
        const val CRASH_REPORTING_ENABLED = true
        
        // Performance monitoring
        const val PERFORMANCE_MONITORING_ENABLED = true
        const val PERFORMANCE_SAMPLE_RATE = 0.1 // 10%
        
        // Feature rollout percentages
        val FEATURE_ROLLOUT_PERCENTAGES = mapOf(
            "new_mining_algorithm" to 0.1, // 10%
            "advanced_ai_detection" to 0.05, // 5%
            "experimental_ui" to 0.01 // 1%
        )
    }
    
    // ================================
    // VERSION INFORMATION
    // ================================
    
    object Version {
        const val SDK_VERSION = "3.0.0"
        const val API_VERSION = "1.0"
        const val PROTOCOL_VERSION = "1"
        
        const val MIN_SUPPORTED_API_VERSION = "1.0"
        const val MIN_SUPPORTED_APP_VERSION = "3.0.0"
        
        // Version compatibility
        const val BACKWARD_COMPATIBILITY_VERSIONS = 3
        
        // Update requirements
        const val FORCE_UPDATE_THRESHOLD = "2.0.0"
        const val RECOMMENDED_UPDATE_THRESHOLD = "2.9.0"
    }
    
    // ================================
    // UTILITY FUNCTIONS
    // ================================
    
    /**
     * Get mining rate based on current phase and user count
     */
    fun getMiningRateForPhase(userCount: Long): Double {
        return when {
            userCount < Mining.PHASE_1_USER_THRESHOLD -> Mining.PHASE_1_BASE_RATE
            userCount < Mining.PHASE_2_USER_THRESHOLD -> Mining.PHASE_2_BASE_RATE
            userCount < Mining.PHASE_3_USER_THRESHOLD -> Mining.PHASE_3_BASE_RATE
            else -> Mining.PHASE_4_BASE_RATE
        }
    }
    
    /**
     * Get Finizen bonus based on current phase
     */
    fun getFinizenBonusForPhase(userCount: Long): Double {
        return when {
            userCount < Mining.PHASE_1_USER_THRESHOLD -> Mining.PHASE_1_FINIZEN_BONUS
            userCount < Mining.PHASE_2_USER_THRESHOLD -> Mining.PHASE_2_FINIZEN_BONUS
            userCount < Mining.PHASE_3_USER_THRESHOLD -> Mining.PHASE_3_FINIZEN_BONUS
            else -> Mining.PHASE_4_FINIZEN_BONUS
        }
    }
    
    /**
     * Get XP badge tier based on level
     */
    fun getXPBadgeTier(level: Int): XP.BadgeTier {
        return XP.BadgeTier.values().find { level in it.levelRange } ?: XP.BadgeTier.BRONZE
    }
    
    /**
     * Get RP tier based on RP amount
     */
    fun getRPTier(rpAmount: Int): ReferralPoints.RPTier {
        return ReferralPoints.RPTier.values().find { 
            rpAmount in it.minRP..it.maxRP 
        } ?: ReferralPoints.RPTier.EXPLORER
    }
    
    /**
     * Get staking tier based on staked amount
     */
    fun getStakingTier(stakedAmount: Int): Int {
        return when {
            stakedAmount < Staking.TIER_1_MIN -> 0
            stakedAmount <= Staking.TIER_1_MAX -> 1
            stakedAmount <= Staking.TIER_2_MAX -> 2
            stakedAmount <= Staking.TIER_3_MAX -> 3
            stakedAmount <= Staking.TIER_4_MAX -> 4
            else -> 5
        }
    }
    
    /**
     * Get platform multiplier for XP calculation
     */
    fun getPlatformMultiplier(platform: String): Double {
        return when (platform.lowercase()) {
            SocialPlatforms.PLATFORM_TIKTOK -> XP.TIKTOK_MULTIPLIER
            SocialPlatforms.PLATFORM_INSTAGRAM -> XP.INSTAGRAM_MULTIPLIER
            SocialPlatforms.PLATFORM_YOUTUBE -> XP.YOUTUBE_MULTIPLIER
            SocialPlatforms.PLATFORM_FACEBOOK -> XP.FACEBOOK_MULTIPLIER
            SocialPlatforms.PLATFORM_TWITTER_X -> XP.TWITTER_X_MULTIPLIER
            else -> XP.DEFAULT_PLATFORM_MULTIPLIER
        }
    }
    
    /**
     * Check if feature is enabled based on feature flag
     */
    fun isFeatureEnabled(featureFlag: String, userId: String? = null): Boolean {
        // This would typically check against a remote config service
        // For now, return true for core features
        return when (featureFlag) {
            FeatureFlags.MINING_ENABLED,
            FeatureFlags.XP_SYSTEM_ENABLED,
            FeatureFlags.REFERRAL_SYSTEM_ENABLED -> true
            else -> false
        }
    }
    
    /**
     * Format FIN amount for display
     */
    fun formatFinAmount(amount: Double, decimals: Int = 6): String {
        return String.format("%.${decimals}f FIN", amount)
    }
    
    /**
     * Format XP amount for display
     */
    fun formatXPAmount(amount: Long): String {
        return when {
            amount >= 1_000_000 -> String.format("%.1fM XP", amount / 1_000_000.0)
            amount >= 1_000 -> String.format("%.1fK XP", amount / 1_000.0)
            else -> "$amount XP"
        }
    }
    
    /**
     * Get error message by error code
     */
    fun getErrorMessage(errorCode: Int): String {
        return when (errorCode) {
            ErrorCodes.NETWORK_ERROR -> ErrorMessages.NETWORK_ERROR
            ErrorCodes.TIMEOUT_ERROR -> ErrorMessages.TIMEOUT_ERROR
            ErrorCodes.SERVER_ERROR -> ErrorMessages.SERVER_ERROR
            ErrorCodes.UNAUTHORIZED_ERROR -> ErrorMessages.UNAUTHORIZED_ERROR
            ErrorCodes.RATE_LIMIT_ERROR -> ErrorMessages.RATE_LIMIT_ERROR
            ErrorCodes.BOT_DETECTED -> ErrorMessages.BOT_DETECTED
            ErrorCodes.MINING_LIMIT_REACHED -> ErrorMessages.MINING_LIMIT_REACHED
            ErrorCodes.INSUFFICIENT_BALANCE -> ErrorMessages.INSUFFICIENT_BALANCE
            ErrorCodes.KYC_REQUIRED -> ErrorMessages.KYC_REQUIRED
            ErrorCodes.INVALID_REFERRAL_CODE -> ErrorMessages.INVALID_REFERRAL_CODE
            ErrorCodes.PLATFORM_NOT_CONNECTED -> ErrorMessages.PLATFORM_NOT_CONNECTED
            ErrorCodes.GUILD_FULL -> ErrorMessages.GUILD_FULL
            ErrorCodes.WITHDRAWAL_LIMIT_EXCEEDED -> ErrorMessages.WITHDRAWAL_LIMIT_EXCEEDED
            ErrorCodes.EWALLET_NOT_CONNECTED -> ErrorMessages.EWALLET_NOT_CONNECTED
            else -> "Unknown error occurred. Please try again."
        }
    }
}
