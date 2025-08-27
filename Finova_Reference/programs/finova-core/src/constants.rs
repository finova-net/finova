// programs/finova-core/src/constants.rs

use anchor_lang::prelude::*;

/// Program constants for Finova Network Core
pub mod program_constants {
    use super::*;

    // ======================
    // PROGRAM VERSION & METADATA
    // ======================
    pub const PROGRAM_VERSION: u8 = 1;
    pub const PROGRAM_NAME: &str = "finova-core";
    pub const NETWORK_NAME: &str = "Finova Network";

    // ======================
    // MINING SYSTEM CONSTANTS
    // ======================
    
    /// Base mining rates for different phases (FIN tokens per hour)
    pub const PHASE_1_BASE_RATE: u64 = 100_000; // 0.1 FIN (in micro-FIN)
    pub const PHASE_2_BASE_RATE: u64 = 50_000;  // 0.05 FIN
    pub const PHASE_3_BASE_RATE: u64 = 25_000;  // 0.025 FIN
    pub const PHASE_4_BASE_RATE: u64 = 10_000;  // 0.01 FIN

    /// Mining phase thresholds (number of users)
    pub const PHASE_1_USER_THRESHOLD: u64 = 100_000;    // 0-100K users
    pub const PHASE_2_USER_THRESHOLD: u64 = 1_000_000;  // 100K-1M users
    pub const PHASE_3_USER_THRESHOLD: u64 = 10_000_000; // 1M-10M users
    pub const PHASE_4_USER_THRESHOLD: u64 = u64::MAX;   // 10M+ users

    /// Finizen bonus multipliers for each phase
    pub const PHASE_1_FINIZEN_BONUS: u64 = 200; // 2.0x (in basis points * 100)
    pub const PHASE_2_FINIZEN_BONUS: u64 = 150; // 1.5x
    pub const PHASE_3_FINIZEN_BONUS: u64 = 120; // 1.2x
    pub const PHASE_4_FINIZEN_BONUS: u64 = 100; // 1.0x

    /// Mining calculation constants
    pub const REFERRAL_BONUS_PER_ACTIVE: u64 = 10; // 0.1x per active referral (in basis points)
    pub const MAX_REFERRAL_BONUS: u64 = 300; // 3.0x maximum referral bonus
    pub const KYC_SECURITY_BONUS: u64 = 120; // 1.2x for KYC verified
    pub const NON_KYC_PENALTY: u64 = 80;     // 0.8x for non-KYC
    
    /// Exponential regression constants
    pub const REGRESSION_COEFFICIENT: u64 = 1000; // 0.001 coefficient scaled by 1M
    pub const REGRESSION_SCALE: u64 = 1_000_000;
    
    /// Daily mining caps (in micro-FIN)
    pub const PHASE_1_DAILY_CAP: u64 = 4_800_000; // 4.8 FIN
    pub const PHASE_2_DAILY_CAP: u64 = 1_800_000; // 1.8 FIN
    pub const PHASE_3_DAILY_CAP: u64 = 720_000;   // 0.72 FIN
    pub const PHASE_4_DAILY_CAP: u64 = 240_000;   // 0.24 FIN

    // ======================
    // XP SYSTEM CONSTANTS
    // ======================
    
    /// Base XP values for different activities
    pub const XP_ORIGINAL_POST: u64 = 50;
    pub const XP_PHOTO_POST: u64 = 75;
    pub const XP_VIDEO_POST: u64 = 150;
    pub const XP_STORY_STATUS: u64 = 25;
    pub const XP_MEANINGFUL_COMMENT: u64 = 25;
    pub const XP_LIKE_REACT: u64 = 5;
    pub const XP_SHARE_REPOST: u64 = 15;
    pub const XP_FOLLOW_SUBSCRIBE: u64 = 20;
    pub const XP_DAILY_LOGIN: u64 = 10;
    pub const XP_DAILY_QUEST: u64 = 100;
    pub const XP_MILESTONE: u64 = 500;
    pub const XP_VIRAL_CONTENT: u64 = 1000;

    /// Platform multipliers (in basis points, 100 = 1.0x)
    pub const PLATFORM_TIKTOK_MULTIPLIER: u64 = 130;    // 1.3x
    pub const PLATFORM_INSTAGRAM_MULTIPLIER: u64 = 120; // 1.2x
    pub const PLATFORM_YOUTUBE_MULTIPLIER: u64 = 140;   // 1.4x
    pub const PLATFORM_X_MULTIPLIER: u64 = 120;         // 1.2x
    pub const PLATFORM_FACEBOOK_MULTIPLIER: u64 = 110;  // 1.1x
    pub const PLATFORM_DEFAULT_MULTIPLIER: u64 = 100;   // 1.0x

    /// Quality score bounds (in basis points)
    pub const MIN_QUALITY_SCORE: u64 = 50;  // 0.5x minimum
    pub const MAX_QUALITY_SCORE: u64 = 200; // 2.0x maximum
    pub const DEFAULT_QUALITY_SCORE: u64 = 100; // 1.0x default

    /// Streak bonus multipliers (in basis points)
    pub const MAX_STREAK_BONUS: u64 = 300; // 3.0x maximum
    pub const STREAK_BONUS_INCREMENT: u64 = 5; // 0.05x per day

    /// Level progression constants
    pub const LEVEL_PROGRESSION_COEFFICIENT: u64 = 100; // 0.01 scaled by 10K
    pub const LEVEL_PROGRESSION_SCALE: u64 = 10_000;

    /// Daily activity limits
    pub const DAILY_PHOTO_POST_LIMIT: u32 = 20;
    pub const DAILY_VIDEO_POST_LIMIT: u32 = 10;
    pub const DAILY_STORY_LIMIT: u32 = 50;
    pub const DAILY_COMMENT_LIMIT: u32 = 100;
    pub const DAILY_LIKE_LIMIT: u32 = 200;
    pub const DAILY_SHARE_LIMIT: u32 = 50;
    pub const DAILY_FOLLOW_LIMIT: u32 = 25;
    pub const DAILY_QUEST_LIMIT: u32 = 3;

    /// XP Level tiers and mining multipliers
    pub const BRONZE_LEVEL_START: u32 = 1;
    pub const BRONZE_LEVEL_END: u32 = 10;
    pub const SILVER_LEVEL_START: u32 = 11;
    pub const SILVER_LEVEL_END: u32 = 25;
    pub const GOLD_LEVEL_START: u32 = 26;
    pub const GOLD_LEVEL_END: u32 = 50;
    pub const PLATINUM_LEVEL_START: u32 = 51;
    pub const PLATINUM_LEVEL_END: u32 = 75;
    pub const DIAMOND_LEVEL_START: u32 = 76;
    pub const DIAMOND_LEVEL_END: u32 = 100;
    pub const MYTHIC_LEVEL_START: u32 = 101;

    /// Mining multipliers for XP levels (in basis points)
    pub const BRONZE_MINING_MULTIPLIER_MIN: u64 = 100; // 1.0x
    pub const BRONZE_MINING_MULTIPLIER_MAX: u64 = 120; // 1.2x
    pub const SILVER_MINING_MULTIPLIER_MIN: u64 = 130; // 1.3x
    pub const SILVER_MINING_MULTIPLIER_MAX: u64 = 180; // 1.8x
    pub const GOLD_MINING_MULTIPLIER_MIN: u64 = 190;   // 1.9x
    pub const GOLD_MINING_MULTIPLIER_MAX: u64 = 250;   // 2.5x
    pub const PLATINUM_MINING_MULTIPLIER_MIN: u64 = 260; // 2.6x
    pub const PLATINUM_MINING_MULTIPLIER_MAX: u64 = 320; // 3.2x
    pub const DIAMOND_MINING_MULTIPLIER_MIN: u64 = 330;  // 3.3x
    pub const DIAMOND_MINING_MULTIPLIER_MAX: u64 = 400;  // 4.0x
    pub const MYTHIC_MINING_MULTIPLIER_MIN: u64 = 410;   // 4.1x
    pub const MYTHIC_MINING_MULTIPLIER_MAX: u64 = 500;   // 5.0x

    /// XP requirements for each level tier
    pub const BRONZE_XP_REQUIREMENT: u64 = 0;
    pub const SILVER_XP_REQUIREMENT: u64 = 1_000;
    pub const GOLD_XP_REQUIREMENT: u64 = 5_000;
    pub const PLATINUM_XP_REQUIREMENT: u64 = 20_000;
    pub const DIAMOND_XP_REQUIREMENT: u64 = 50_000;
    pub const MYTHIC_XP_REQUIREMENT: u64 = 100_000;

    // ======================
    // REFERRAL SYSTEM CONSTANTS
    // ======================

    /// RP earning structure
    pub const RP_SIGNUP_BONUS: u64 = 50;
    pub const RP_KYC_BONUS: u64 = 100;
    pub const RP_FIRST_MINING_BONUS: u64 = 25;
    pub const RP_DAILY_MINING_PERCENTAGE: u64 = 10; // 10% of referral's daily mining
    pub const RP_XP_PERCENTAGE: u64 = 5; // 5% of referral's XP gains
    pub const RP_ACHIEVEMENT_BONUS: u64 = 50;

    /// Network milestone bonuses
    pub const RP_10_ACTIVE_BONUS: u64 = 500;
    pub const RP_25_ACTIVE_BONUS: u64 = 1_500;
    pub const RP_50_ACTIVE_BONUS: u64 = 5_000;
    pub const RP_100_ACTIVE_BONUS: u64 = 15_000;

    /// RP Tier thresholds and benefits
    pub const EXPLORER_RP_MIN: u64 = 0;
    pub const CONNECTOR_RP_MIN: u64 = 1_000;
    pub const INFLUENCER_RP_MIN: u64 = 5_000;
    pub const LEADER_RP_MIN: u64 = 15_000;
    pub const AMBASSADOR_RP_MIN: u64 = 50_000;

    /// RP Tier mining bonuses (in basis points)
    pub const EXPLORER_MINING_BONUS: u64 = 0;    // +0%
    pub const CONNECTOR_MINING_BONUS: u64 = 20;  // +20%
    pub const INFLUENCER_MINING_BONUS: u64 = 50; // +50%
    pub const LEADER_MINING_BONUS: u64 = 100;    // +100%
    pub const AMBASSADOR_MINING_BONUS: u64 = 200; // +200%

    /// Referral network limits
    pub const EXPLORER_REFERRAL_LIMIT: u32 = 10;
    pub const CONNECTOR_REFERRAL_LIMIT: u32 = 25;
    pub const INFLUENCER_REFERRAL_LIMIT: u32 = 50;
    pub const LEADER_REFERRAL_LIMIT: u32 = 100;
    pub const AMBASSADOR_REFERRAL_LIMIT: u32 = u32::MAX;

    /// Referral commission percentages (in basis points)
    pub const EXPLORER_L1_COMMISSION: u64 = 10;  // 10%
    pub const CONNECTOR_L1_COMMISSION: u64 = 15; // 15%
    pub const INFLUENCER_L1_COMMISSION: u64 = 20; // 20%
    pub const LEADER_L1_COMMISSION: u64 = 25;    // 25%
    pub const AMBASSADOR_L1_COMMISSION: u64 = 30; // 30%

    pub const CONNECTOR_L2_COMMISSION: u64 = 5;   // 5%
    pub const INFLUENCER_L2_COMMISSION: u64 = 8;  // 8%
    pub const LEADER_L2_COMMISSION: u64 = 10;     // 10%
    pub const AMBASSADOR_L2_COMMISSION: u64 = 15; // 15%

    pub const INFLUENCER_L3_COMMISSION: u64 = 3;  // 3%
    pub const LEADER_L3_COMMISSION: u64 = 5;      // 5%
    pub const AMBASSADOR_L3_COMMISSION: u64 = 8;  // 8%

    /// Network quality regression constants
    pub const NETWORK_REGRESSION_COEFFICIENT: u64 = 100; // 0.0001 scaled by 1M
    pub const NETWORK_REGRESSION_SCALE: u64 = 1_000_000;

    // ======================
    // STAKING SYSTEM CONSTANTS
    // ======================

    /// Staking tiers (in FIN tokens, scaled by 1M for micro-FIN)
    pub const BASIC_STAKE_MIN: u64 = 100_000_000;      // 100 FIN
    pub const PREMIUM_STAKE_MIN: u64 = 500_000_000;    // 500 FIN
    pub const VIP_STAKE_MIN: u64 = 1_000_000_000;      // 1,000 FIN
    pub const ELITE_STAKE_MIN: u64 = 5_000_000_000;    // 5,000 FIN
    pub const WHALE_STAKE_MIN: u64 = 10_000_000_000;   // 10,000 FIN

    /// Staking APY rates (in basis points, annual)
    pub const BASIC_STAKE_APY: u64 = 800;    // 8%
    pub const PREMIUM_STAKE_APY: u64 = 1000;  // 10%
    pub const VIP_STAKE_APY: u64 = 1200;     // 12%
    pub const ELITE_STAKE_APY: u64 = 1400;   // 14%
    pub const WHALE_STAKE_APY: u64 = 1500;   // 15%

    /// Staking mining boosts (in basis points)
    pub const BASIC_STAKE_MINING_BOOST: u64 = 20;   // +20%
    pub const PREMIUM_STAKE_MINING_BOOST: u64 = 35; // +35%
    pub const VIP_STAKE_MINING_BOOST: u64 = 50;     // +50%
    pub const ELITE_STAKE_MINING_BOOST: u64 = 75;   // +75%
    pub const WHALE_STAKE_MINING_BOOST: u64 = 100;  // +100%

    /// Staking XP multipliers (in basis points)
    pub const BASIC_STAKE_XP_MULTIPLIER: u64 = 110;  // +10%
    pub const PREMIUM_STAKE_XP_MULTIPLIER: u64 = 120; // +20%
    pub const VIP_STAKE_XP_MULTIPLIER: u64 = 130;    // +30%
    pub const ELITE_STAKE_XP_MULTIPLIER: u64 = 150;   // +50%
    pub const WHALE_STAKE_XP_MULTIPLIER: u64 = 175;   // +75%

    /// Staking RP bonuses (in basis points)
    pub const BASIC_STAKE_RP_BONUS: u64 = 5;   // +5%
    pub const PREMIUM_STAKE_RP_BONUS: u64 = 10; // +10%
    pub const VIP_STAKE_RP_BONUS: u64 = 20;    // +20%
    pub const ELITE_STAKE_RP_BONUS: u64 = 35;   // +35%
    pub const WHALE_STAKE_RP_BONUS: u64 = 50;   // +50%

    /// Staking duration multipliers (in basis points)
    pub const LOYALTY_BONUS_MONTHLY_INCREMENT: u64 = 5; // +0.05x per month
    pub const MAX_LOYALTY_BONUS: u64 = 200; // +2.0x maximum after 40 months

    /// Unstaking penalties (in basis points)
    pub const EARLY_UNSTAKE_PENALTY_1_MONTH: u64 = 500;  // 5% penalty if < 1 month
    pub const EARLY_UNSTAKE_PENALTY_3_MONTHS: u64 = 300; // 3% penalty if < 3 months
    pub const EARLY_UNSTAKE_PENALTY_6_MONTHS: u64 = 100; // 1% penalty if < 6 months

    // ======================
    // ANTI-BOT & SECURITY CONSTANTS
    // ======================

    /// Human probability thresholds (in basis points)
    pub const MIN_HUMAN_PROBABILITY: u64 = 5000; // 50% minimum to participate
    pub const HIGH_HUMAN_PROBABILITY: u64 = 8000; // 80% for full benefits
    pub const PERFECT_HUMAN_PROBABILITY: u64 = 10000; // 100% maximum

    /// Bot detection factors and weights (in basis points)
    pub const BIOMETRIC_WEIGHT: u64 = 2500;    // 25%
    pub const BEHAVIORAL_WEIGHT: u64 = 2000;   // 20%
    pub const SOCIAL_GRAPH_WEIGHT: u64 = 2000; // 20%
    pub const DEVICE_WEIGHT: u64 = 1500;       // 15%
    pub const INTERACTION_WEIGHT: u64 = 2000;  // 20%

    /// Activity analysis constants
    pub const MIN_SESSION_DURATION: u64 = 30;  // 30 seconds minimum
    pub const MAX_SESSION_DURATION: u64 = 14400; // 4 hours maximum
    pub const SUSPICIOUS_CLICK_SPEED: u64 = 100; // < 100ms between clicks
    pub const NATURAL_BREAK_MIN: u64 = 300;    // 5 minutes minimum break
    
    /// Progressive difficulty scaling
    pub const DIFFICULTY_SCALING_FACTOR: u64 = 1000; // Divide total earned by this
    pub const SUSPICIOUS_SCORE_MULTIPLIER: u64 = 200; // 2x multiplier for suspicious activity
    
    /// Cooling period constants (in seconds)
    pub const INTENSIVE_SESSION_COOLDOWN: u64 = 3600;    // 1 hour cooldown
    pub const SUSPICIOUS_ACTIVITY_COOLDOWN: u64 = 86400; // 24 hour cooldown
    pub const BOT_DETECTION_COOLDOWN: u64 = 604800;      // 7 day cooldown

    // ======================
    // GUILD SYSTEM CONSTANTS
    // ======================

    /// Guild size limits
    pub const MIN_GUILD_SIZE: u32 = 10;
    pub const MAX_GUILD_SIZE: u32 = 50;
    pub const GUILD_LEADER_MIN_LEVEL: u32 = SILVER_LEVEL_START; // Must be Silver or higher

    /// Guild activity bonuses (in basis points)
    pub const DAILY_CHALLENGE_XP_BONUS: u64 = 20;  // +20% XP for all members
    pub const WEEKLY_WAR_MINING_BONUS: u64 = 15;   // +15% mining during wars
    pub const GUILD_PARTICIPATION_BONUS: u64 = 30; // +30% mining during events

    /// Guild competition rewards (in micro-FIN)
    pub const DAILY_CHALLENGE_REWARD_POOL: u64 = 1_000_000;  // 1 FIN per challenge
    pub const WEEKLY_WAR_REWARD_POOL: u64 = 10_000_000;      // 10 FIN per war
    pub const MONTHLY_CHAMPIONSHIP_POOL: u64 = 100_000_000;   // 100 FIN per championship
    pub const SEASONAL_LEAGUE_POOL: u64 = 1_000_000_000;     // 1,000 FIN per season

    /// Guild treasury limits
    pub const MAX_GUILD_TREASURY: u64 = 100_000_000_000; // 100,000 FIN maximum

    // ======================
    // GOVERNANCE CONSTANTS
    // ======================

    /// Voting power calculation weights (in basis points)
    pub const STAKED_TOKEN_WEIGHT: u64 = 4000;  // 40% weight
    pub const XP_LEVEL_WEIGHT: u64 = 2000;      // 20% weight
    pub const RP_TIER_WEIGHT: u64 = 2000;       // 20% weight
    pub const ACTIVITY_WEIGHT: u64 = 2000;      // 20% weight

    /// Proposal thresholds
    pub const MIN_PROPOSAL_THRESHOLD: u64 = 1_000_000_000; // 1,000 FIN staked minimum
    pub const QUORUM_PERCENTAGE: u64 = 2000; // 20% of total staked tokens
    pub const PASSING_THRESHOLD: u64 = 5100; // 51% approval required

    /// Voting periods (in slots, ~400ms per slot on Solana)
    pub const VOTING_PERIOD: u64 = 1_209_600; // ~5.6 days (1.2M slots)
    pub const EXECUTION_DELAY: u64 = 432_000;  // ~2 days (432K slots)
    pub const GRACE_PERIOD: u64 = 432_000;     // ~2 days (432K slots)

    // ======================
    // NFT & SPECIAL CARDS CONSTANTS
    // ======================

    /// Special card types and effects
    pub const DOUBLE_MINING_CARD_BOOST: u64 = 200;  // +100% (2x total)
    pub const TRIPLE_MINING_CARD_BOOST: u64 = 300;  // +200% (3x total)
    pub const MINING_FRENZY_CARD_BOOST: u64 = 600;  // +500% (6x total)
    pub const ETERNAL_MINER_CARD_BOOST: u64 = 150;  // +50% (1.5x total)

    /// Card durations (in seconds)
    pub const DOUBLE_MINING_DURATION: u64 = 86400;    // 24 hours
    pub const TRIPLE_MINING_DURATION: u64 = 43200;    // 12 hours
    pub const MINING_FRENZY_DURATION: u64 = 14400;    // 4 hours
    pub const ETERNAL_MINER_DURATION: u64 = 2592000;  // 30 days

    /// Card prices (in micro-FIN)
    pub const DOUBLE_MINING_CARD_PRICE: u64 = 50_000_000;    // 50 FIN
    pub const TRIPLE_MINING_CARD_PRICE: u64 = 150_000_000;   // 150 FIN
    pub const MINING_FRENZY_CARD_PRICE: u64 = 500_000_000;   // 500 FIN
    pub const ETERNAL_MINER_CARD_PRICE: u64 = 2_000_000_000; // 2,000 FIN

    /// XP accelerator cards
    pub const XP_DOUBLE_CARD_BOOST: u64 = 200;      // +100% XP
    pub const XP_DOUBLE_DURATION: u64 = 86400;      // 24 hours
    pub const XP_DOUBLE_PRICE: u64 = 40_000_000;    // 40 FIN

    pub const LEVEL_RUSH_XP_BONUS: u64 = 500;       // +500 XP instant
    pub const LEVEL_RUSH_PRICE: u64 = 120_000_000;  // 120 FIN

    pub const XP_MAGNET_VIRAL_BOOST: u64 = 400;     // +300% for viral content
    pub const XP_MAGNET_DURATION: u64 = 172800;     // 48 hours
    pub const XP_MAGNET_PRICE: u64 = 300_000_000;   // 300 FIN

    /// Referral power cards
    pub const REFERRAL_BOOST_BONUS: u64 = 150;      // +50% referral rewards
    pub const REFERRAL_BOOST_DURATION: u64 = 604800; // 7 days
    pub const REFERRAL_BOOST_PRICE: u64 = 60_000_000; // 60 FIN

    pub const NETWORK_AMPLIFIER_TIER_BOOST: u64 = 2; // +2 RP tier levels
    pub const NETWORK_AMPLIFIER_DURATION: u64 = 86400; // 24 hours
    pub const NETWORK_AMPLIFIER_PRICE: u64 = 200_000_000; // 200 FIN

    /// Card synergy bonuses (in basis points)
    pub const SAME_CATEGORY_SYNERGY_BONUS: u64 = 15; // +15%
    pub const ALL_CATEGORIES_SYNERGY_BONUS: u64 = 30; // +30%
    pub const CARD_COUNT_BONUS_PER_CARD: u64 = 10;   // +10% per active card

    /// Card rarity multipliers (in basis points)
    pub const COMMON_RARITY_BONUS: u64 = 0;     // +0%
    pub const UNCOMMON_RARITY_BONUS: u64 = 5;   // +5%
    pub const RARE_RARITY_BONUS: u64 = 10;      // +10%
    pub const EPIC_RARITY_BONUS: u64 = 20;      // +20%
    pub const LEGENDARY_RARITY_BONUS: u64 = 35; // +35%

    // ======================
    // TIME AND CALCULATION CONSTANTS
    // ======================

    /// Time constants (in seconds)
    pub const SECONDS_PER_MINUTE: u64 = 60;
    pub const SECONDS_PER_HOUR: u64 = 3600;
    pub const SECONDS_PER_DAY: u64 = 86400;
    pub const SECONDS_PER_WEEK: u64 = 604800;
    pub const SECONDS_PER_MONTH: u64 = 2629746; // Average month (30.44 days)
    pub const SECONDS_PER_YEAR: u64 = 31556952; // Average year (365.25 days)

    /// Calculation precision constants
    pub const BASIS_POINTS_SCALE: u64 = 10000;     // 100% = 10,000 basis points
    pub const PERCENTAGE_SCALE: u64 = 100;         // 100% = 100
    pub const MICRO_FIN_SCALE: u64 = 1_000_000;    // 1 FIN = 1M micro-FIN
    pub const PRECISION_MULTIPLIER: u64 = 1_000_000; // For precise calculations

    /// Mathematical constants (scaled for integer arithmetic)
    pub const E_SCALED: u64 = 2718282; // e ≈ 2.718282 (scaled by 1M)
    pub const PI_SCALED: u64 = 3141593; // π ≈ 3.141593 (scaled by 1M)

    // ======================
    // ECONOMIC MODEL CONSTANTS
    // ======================

    /// Token supply constants (in micro-FIN)
    pub const MAX_TOTAL_SUPPLY: u64 = 100_000_000_000_000_000; // 100 billion FIN
    pub const COMMUNITY_MINING_ALLOCATION: u64 = 50_000_000_000_000_000; // 50% community mining
    pub const TEAM_ALLOCATION: u64 = 20_000_000_000_000_000; // 20% team
    pub const INVESTOR_ALLOCATION: u64 = 15_000_000_000_000_000; // 15% investors
    pub const PUBLIC_SALE_ALLOCATION: u64 = 10_000_000_000_000_000; // 10% public sale
    pub const TREASURY_ALLOCATION: u64 = 5_000_000_000_000_000; // 5% treasury

    /// Fee constants (in basis points)
    pub const TRANSACTION_FEE: u64 = 10;        // 0.1% transaction fee
    pub const NFT_MARKETPLACE_FEE: u64 = 250;   // 2.5% marketplace fee
    pub const DEX_SWAP_FE: u64 = 30;            // 0.3% DEX swap fee
    pub const WHALE_TAX_THRESHOLD: u64 = 100_000_000_000; // 100K FIN threshold
    pub const WHALE_TAX_RATE: u64 = 500;        // 5% progressive tax on whales

    /// Reward pool distribution (in basis points)
    pub const MINING_REWARDS_ALLOCATION: u64 = 4000;    // 40%
    pub const XP_BONUS_ALLOCATION: u64 = 2500;          // 25%
    pub const RP_NETWORK_ALLOCATION: u64 = 2000;        // 20%
    pub const SPECIAL_EVENTS_ALLOCATION: u64 = 1000;    // 10%
    pub const TREASURY_RESERVE_ALLOCATION: u64 = 500;   // 5%

    // ======================
    // PLATFORM INTEGRATION CONSTANTS
    // ======================

    /// Social platform identifiers
    pub const PLATFORM_INSTAGRAM: u8 = 1;
    pub const PLATFORM_TIKTOK: u8 = 2;
    pub const PLATFORM_YOUTUBE: u8 = 3;
    pub const PLATFORM_FACEBOOK: u8 = 4;
    pub const PLATFORM_X_TWITTER: u8 = 5;
    pub const PLATFORM_FINOVA_APP: u8 = 6;

    /// Content type identifiers
    pub const CONTENT_TEXT_POST: u8 = 1;
    pub const CONTENT_PHOTO_POST: u8 = 2;
    pub const CONTENT_VIDEO_POST: u8 = 3;
    pub const CONTENT_STORY_STATUS: u8 = 4;
    pub const CONTENT_COMMENT: u8 = 5;
    pub const CONTENT_LIKE_REACT: u8 = 6;
    pub const CONTENT_SHARE_REPOST: u8 = 7;
    pub const CONTENT_FOLLOW_SUBSCRIBE: u8 = 8;

    /// Activity type identifiers
    pub const ACTIVITY_DAILY_LOGIN: u8 = 10;
    pub const ACTIVITY_DAILY_QUEST: u8 = 11;
    pub const ACTIVITY_MILESTONE: u8 = 12;
    pub const ACTIVITY_VIRAL_CONTENT: u8 = 13;
    pub const ACTIVITY_REFERRAL_SUCCESS: u8 = 14;
    pub const ACTIVITY_KYC_COMPLETION: u8 = 15;
    pub const ACTIVITY_FIRST_MINING: u8 = 16;
    pub const ACTIVITY_GUILD_PARTICIPATION: u8 = 17;

    /// Viral content thresholds
    pub const VIRAL_THRESHOLD_VIEWS: u64 = 1000;        // 1K+ views
    pub const VIRAL_THRESHOLD_LIKES: u64 = 100;         // 100+ likes
    pub const VIRAL_THRESHOLD_SHARES: u64 = 50;         // 50+ shares
    pub const VIRAL_THRESHOLD_COMMENTS: u64 = 25;       // 25+ comments

    /// E-wallet integration constants
    pub const EWALLET_OVO: u8 = 1;
    pub const EWALLET_GOPAY: u8 = 2;
    pub const EWALLET_DANA: u8 = 3;
    pub const EWALLET_SHOPEEPAY: u8 = 4;
    pub const EWALLET_LINKAJA: u8 = 5;

    /// KYC verification levels
    pub const KYC_LEVEL_NONE: u8 = 0;
    pub const KYC_LEVEL_BASIC: u8 = 1;     // Phone + Email
    pub const KYC_LEVEL_STANDARD: u8 = 2;  // + ID Document
    pub const KYC_LEVEL_PREMIUM: u8 = 3;   // + Biometric + Address
    pub const KYC_LEVEL_ENTERPRISE: u8 = 4; // + Enhanced due diligence

    // ======================
    // ERROR HANDLING CONSTANTS
    // ======================

    /// Maximum retry attempts for failed operations
    pub const MAX_RETRY_ATTEMPTS: u8 = 3;
    pub const RETRY_DELAY_BASE: u64 = 1000; // 1 second base delay (in milliseconds)

    /// Rate limiting constants
    pub const MAX_TRANSACTIONS_PER_BLOCK: u32 = 100;
    pub const MAX_XP_ACTIVITIES_PER_HOUR: u32 = 500;
    pub const MAX_MINING_CLAIMS_PER_DAY: u32 = 24;
    pub const MAX_REFERRAL_INVITES_PER_DAY: u32 = 10;

    /// Account state validation constants
    pub const MIN_ACCOUNT_AGE_FOR_MINING: u64 = 86400; // 24 hours minimum account age
    pub const MIN_ACTIVITY_FOR_REWARDS: u32 = 1;       // Minimum 1 activity per day
    pub const MAX_INACTIVE_DAYS: u32 = 30;             // 30 days max inactivity

    // ======================
    // NETWORK HEALTH CONSTANTS
    // ======================

    /// Network growth targets
    pub const HEALTHY_DAILY_NEW_USERS: u32 = 1000;     // Target new users per day
    pub const HEALTHY_RETENTION_RATE: u64 = 7000;      // 70% retention rate (in basis points)
    pub const HEALTHY_ENGAGEMENT_RATE: u64 = 2000;     // 20% daily engagement rate

    /// Network stability thresholds
    pub const MAX_WHALE_CONCENTRATION: u64 = 1000;     // 10% max whale concentration
    pub const MIN_NETWORK_DIVERSITY: u64 = 8000;       // 80% network diversity score
    pub const SUSPICIOUS_GROWTH_RATE: u64 = 10000;     // 100% daily growth rate alarm

    /// Economic health indicators
    pub const HEALTHY_MINING_DISTRIBUTION: u64 = 8000;  // 80% distribution score
    pub const MAX_INFLATION_RATE: u64 = 500;           // 5% max annual inflation
    pub const MIN_STAKING_PARTICIPATION: u64 = 3000;   // 30% min staking participation

    // ======================
    // SPECIAL EVENT CONSTANTS
    // ======================

    /// Event multipliers and bonuses
    pub const LAUNCH_WEEK_BONUS: u64 = 200;            // +100% bonus during launch week
    pub const HOLIDAY_BONUS: u64 = 150;                // +50% bonus during holidays
    pub const ANNIVERSARY_BONUS: u64 = 300;            // +200% bonus during anniversary
    pub const COMMUNITY_MILESTONE_BONUS: u64 = 250;    // +150% bonus for milestones

    /// Event durations (in seconds)
    pub const LAUNCH_WEEK_DURATION: u64 = 604800;      // 7 days
    pub const HOLIDAY_EVENT_DURATION: u64 = 259200;    // 3 days
    pub const ANNIVERSARY_DURATION: u64 = 1209600;     // 14 days
    pub const FLASH_EVENT_DURATION: u64 = 86400;       // 24 hours

    /// Competition and tournament constants
    pub const TOURNAMENT_ENTRY_FEE: u64 = 10_000_000;  // 10 FIN entry fee
    pub const MIN_TOURNAMENT_PARTICIPANTS: u32 = 100;   // Minimum 100 participants
    pub const MAX_TOURNAMENT_PARTICIPANTS: u32 = 10000; // Maximum 10K participants
    pub const TOURNAMENT_PRIZE_POOL_BASE: u64 = 1_000_000_000; // 1,000 FIN base prize

    // ======================
    // API AND INTEGRATION LIMITS
    // ======================

    /// Social media API rate limits (per hour)
    pub const INSTAGRAM_API_LIMIT: u32 = 1000;
    pub const TIKTOK_API_LIMIT: u32 = 800;
    pub const YOUTUBE_API_LIMIT: u32 = 1200;
    pub const FACEBOOK_API_LIMIT: u32 = 1500;
    pub const X_API_LIMIT: u32 = 600;

    /// Webhook and notification limits
    pub const MAX_WEBHOOK_RETRIES: u8 = 5;
    pub const WEBHOOK_TIMEOUT: u64 = 30000;           // 30 seconds timeout
    pub const MAX_NOTIFICATIONS_PER_USER_PER_DAY: u32 = 50;
    pub const NOTIFICATION_BATCH_SIZE: u32 = 1000;

    /// Data sync and backup constants
    pub const BLOCKCHAIN_SYNC_INTERVAL: u64 = 300;     // 5 minutes
    pub const DATABASE_BACKUP_INTERVAL: u64 = 86400;   // 24 hours
    pub const CACHE_REFRESH_INTERVAL: u64 = 3600;      // 1 hour
    pub const ANALYTICS_PROCESSING_INTERVAL: u64 = 1800; // 30 minutes

    // ======================
    // SECURITY AND COMPLIANCE
    // ======================

    /// Security thresholds and limits
    pub const MAX_LOGIN_ATTEMPTS: u8 = 5;
    pub const LOCKOUT_DURATION: u64 = 3600;            // 1 hour lockout
    pub const SESSION_TIMEOUT: u64 = 86400;            // 24 hour session
    pub const PASSWORD_RESET_COOLDOWN: u64 = 300;      // 5 minutes between resets

    /// AML/KYC compliance limits (in micro-FIN)
    pub const DAILY_WITHDRAWAL_LIMIT_BASIC: u64 = 1_000_000_000;    // 1,000 FIN
    pub const DAILY_WITHDRAWAL_LIMIT_STANDARD: u64 = 10_000_000_000; // 10,000 FIN
    pub const DAILY_WITHDRAWAL_LIMIT_PREMIUM: u64 = 100_000_000_000; // 100,000 FIN
    pub const MONTHLY_VOLUME_LIMIT_BASIC: u64 = 10_000_000_000;     // 10,000 FIN
    pub const MONTHLY_VOLUME_LIMIT_STANDARD: u64 = 100_000_000_000;  // 100,000 FIN

    /// Suspicious activity thresholds
    pub const LARGE_TRANSACTION_THRESHOLD: u64 = 50_000_000_000;    // 50,000 FIN
    pub const RAPID_TRANSACTION_COUNT: u32 = 100;                   // 100 transactions in hour
    pub const VELOCITY_CHECK_THRESHOLD: u64 = 10_000_000_000;       // 10,000 FIN per hour

    // ======================
    // ALGORITHM PARAMETERS
    // ======================

    /// Exponential regression parameters
    pub const EXPONENTIAL_BASE: u64 = 2718282;         // e (scaled by 1M)
    pub const REGRESSION_DECAY_RATE: u64 = 1000;       // 0.001 decay rate (scaled by 1M)
    pub const MIN_REGRESSION_VALUE: u64 = 100;         // 0.0001 minimum value (scaled by 1M)

    /// Quality scoring algorithm parameters
    pub const ORIGINALITY_WEIGHT: u64 = 2500;          // 25% weight
    pub const ENGAGEMENT_PREDICTION_WEIGHT: u64 = 2000; // 20% weight
    pub const PLATFORM_RELEVANCE_WEIGHT: u64 = 2000;   // 20% weight
    pub const BRAND_SAFETY_WEIGHT: u64 = 1500;         // 15% weight
    pub const HUMAN_GENERATED_WEIGHT: u64 = 2000;      // 20% weight

    /// Network analysis parameters
    pub const MIN_NETWORK_CONNECTIONS: u32 = 3;        // Minimum connections for analysis
    pub const MAX_NETWORK_DEPTH: u32 = 6;              // Maximum degrees of separation
    pub const CLUSTERING_COEFFICIENT_THRESHOLD: u64 = 5000; // 50% clustering threshold

    /// Machine learning model parameters
    pub const TRAINING_DATA_MIN_SIZE: u32 = 10000;     // Minimum training dataset size
    pub const MODEL_UPDATE_FREQUENCY: u64 = 604800;    // Weekly model updates
    pub const PREDICTION_CONFIDENCE_THRESHOLD: u64 = 7500; // 75% confidence minimum

    // ======================
    // DEVELOPMENT AND TESTING
    // ======================

    /// Test environment constants
    pub const TEST_USER_MINING_MULTIPLIER: u64 = 1000;  // 10x mining for testing
    pub const TEST_FAST_TIME_MULTIPLIER: u64 = 100;     // 100x time acceleration
    pub const TEST_REDUCED_COOLDOWNS: u64 = 60;         // 1 minute cooldowns for testing

    /// Debug and monitoring constants
    pub const LOG_LEVEL_ERROR: u8 = 0;
    pub const LOG_LEVEL_WARN: u8 = 1;
    pub const LOG_LEVEL_INFO: u8 = 2;
    pub const LOG_LEVEL_DEBUG: u8 = 3;
    pub const LOG_LEVEL_TRACE: u8 = 4;

    /// Performance monitoring thresholds
    pub const MAX_INSTRUCTION_EXECUTION_TIME: u64 = 1000; // 1 second max execution
    pub const MAX_COMPUTE_UNITS: u64 = 1_400_000;         // Solana compute unit limit
    pub const WARNING_COMPUTE_UNITS: u64 = 1_000_000;     // Warning threshold

    // ======================
    // FEATURE FLAGS AND TOGGLES
    // ======================

    /// Feature enablement flags (bit flags)
    pub const FEATURE_MINING_ENABLED: u64 = 1 << 0;
    pub const FEATURE_XP_SYSTEM_ENABLED: u64 = 1 << 1;
    pub const FEATURE_REFERRAL_SYSTEM_ENABLED: u64 = 1 << 2;
    pub const FEATURE_STAKING_ENABLED: u64 = 1 << 3;
    pub const FEATURE_NFTS_ENABLED: u64 = 1 << 4;
    pub const FEATURE_GUILDS_ENABLED: u64 = 1 << 5;
    pub const FEATURE_GOVERNANCE_ENABLED: u64 = 1 << 6;
    pub const FEATURE_ANTI_BOT_ENABLED: u64 = 1 << 7;
    pub const FEATURE_SOCIAL_INTEGRATION_ENABLED: u64 = 1 << 8;
    pub const FEATURE_ANALYTICS_ENABLED: u64 = 1 << 9;

    /// Default feature set for mainnet
    pub const MAINNET_DEFAULT_FEATURES: u64 = FEATURE_MINING_ENABLED
        | FEATURE_XP_SYSTEM_ENABLED
        | FEATURE_REFERRAL_SYSTEM_ENABLED
        | FEATURE_STAKING_ENABLED
        | FEATURE_NFTS_ENABLED
        | FEATURE_GUILDS_ENABLED
        | FEATURE_GOVERNANCE_ENABLED
        | FEATURE_ANTI_BOT_ENABLED
        | FEATURE_SOCIAL_INTEGRATION_ENABLED
        | FEATURE_ANALYTICS_ENABLED;

    /// Gradual rollout percentages (in basis points)
    pub const BETA_USER_PERCENTAGE: u64 = 1000;        // 10% beta users
    pub const EARLY_ACCESS_PERCENTAGE: u64 = 2500;     // 25% early access
    pub const FULL_ROLLOUT_PERCENTAGE: u64 = 10000;    // 100% full rollout
}

/// Account size constants for Program Derived Addresses (PDAs)
pub mod account_sizes {
    use super::*;

    // ======================
    // CORE ACCOUNT SIZES
    // ======================

    /// User account size calculation:
    /// - Discriminator: 8 bytes
    /// - Authority (Pubkey): 32 bytes
    /// - User data: ~200 bytes
    /// - Mining state: ~100 bytes
    /// - XP state: ~50 bytes
    /// - RP state: ~50 bytes
    /// - Staking state: ~100 bytes
    /// - Social connections: ~200 bytes
    /// - Metadata and flags: ~100 bytes
    /// - Buffer for future expansion: ~200 bytes
    pub const USER_ACCOUNT_SIZE: usize = 1024; // 1KB per user

    /// Mining state account size:
    /// - Discriminator: 8 bytes
    /// - User reference: 32 bytes
    /// - Mining statistics: ~150 bytes
    /// - Phase information: ~50 bytes
    /// - Bonus calculations: ~100 bytes
    /// - Timestamps and counters: ~100 bytes
    /// - Buffer: ~200 bytes
    pub const MINING_STATE_SIZE: usize = 640;

    /// XP state account size:
    /// - Discriminator: 8 bytes
    /// - User reference: 32 bytes
    /// - XP totals and levels: ~50 bytes
    /// - Activity tracking: ~200 bytes
    /// - Platform statistics: ~100 bytes
    /// - Streak information: ~50 bytes
    /// - Buffer: ~200 bytes
    pub const XP_STATE_SIZE: usize = 640;

    /// Referral network account size:
    /// - Discriminator: 8 bytes
    /// - User reference: 32 bytes
    /// - Referral tree data: ~300 bytes
    /// - RP calculations: ~100 bytes
    /// - Network statistics: ~150 bytes
    /// - Quality metrics: ~50 bytes
    /// - Buffer: ~200 bytes
    pub const REFERRAL_STATE_SIZE: usize = 840;

    /// Staking account size:
    /// - Discriminator: 8 bytes
    /// - User reference: 32 bytes
    /// - Staked amounts: ~100 bytes
    /// - Reward calculations: ~100 bytes
    /// - Lock periods: ~50 bytes
    /// - Tier information: ~50 bytes
    /// - Buffer: ~200 bytes
    pub const STAKING_STATE_SIZE: usize = 540;

    /// Guild account size:
    /// - Discriminator: 8 bytes
    /// - Guild metadata: ~200 bytes
    /// - Member list: ~1600 bytes (50 members × 32 bytes)
    /// - Statistics: ~100 bytes
    /// - Treasury info: ~50 bytes
    /// - Competition data: ~200 bytes
    /// - Buffer: ~300 bytes
    pub const GUILD_ACCOUNT_SIZE: usize = 2458;

    /// NFT collection account size:
    /// - Discriminator: 8 bytes
    /// - Collection metadata: ~200 bytes
    /// - Creator info: ~100 bytes
    /// - Royalty settings: ~50 bytes
    /// - Statistics: ~100 bytes
    /// - Buffer: ~200 bytes
    pub const NFT_COLLECTION_SIZE: usize = 658;

    /// Special card account size:
    /// - Discriminator: 8 bytes
    /// - Card metadata: ~150 bytes
    /// - Effect parameters: ~100 bytes
    /// - Usage tracking: ~50 bytes
    /// - Owner info: ~32 bytes
    /// - Buffer: ~100 bytes
    pub const SPECIAL_CARD_SIZE: usize = 440;

    // ======================
    // GOVERNANCE ACCOUNT SIZES
    // ======================

    /// Proposal account size:
    /// - Discriminator: 8 bytes
    /// - Proposal data: ~500 bytes
    /// - Voting statistics: ~100 bytes
    /// - Timeline info: ~50 bytes
    /// - Execution data: ~200 bytes
    /// - Buffer: ~200 bytes
    pub const PROPOSAL_ACCOUNT_SIZE: usize = 1058;

    /// Vote record account size:
    /// - Discriminator: 8 bytes
    /// - Voter pubkey: 32 bytes
    /// - Proposal reference: 32 bytes
    /// - Vote data: ~50 bytes
    /// - Timestamp: 8 bytes
    /// - Buffer: ~50 bytes
    pub const VOTE_RECORD_SIZE: usize = 180;

    // ======================
    // SYSTEM ACCOUNT SIZES
    // ======================

    /// Global state account size:
    /// - Discriminator: 8 bytes
    /// - Network statistics: ~200 bytes
    /// - Economic parameters: ~200 bytes
    /// - Feature flags: ~50 bytes
    /// - Admin settings: ~100 bytes
    /// - Emergency controls: ~50 bytes
    /// - Buffer: ~400 bytes
    pub const GLOBAL_STATE_SIZE: usize = 1008;

    /// Analytics account size:
    /// - Discriminator: 8 bytes
    /// - Metrics data: ~800 bytes
    /// - Timestamp ranges: ~50 bytes
    /// - Aggregation info: ~100 bytes
    /// - Buffer: ~200 bytes
    pub const ANALYTICS_ACCOUNT_SIZE: usize = 1158;
}

/// String constants for the program
pub mod string_constants {
    /// Program identification strings
    pub const PROGRAM_NAME_FULL: &str = "Finova Network Core Program";
    pub const PROGRAM_DESCRIPTION: &str = "Core smart contract for Finova Network social mining platform";
    pub const PROGRAM_VERSION_STRING: &str = "1.0.0";

    /// Seed strings for PDA generation
    pub const USER_SEED: &str = "user";
    pub const MINING_SEED: &str = "mining";
    pub const XP_SEED: &str = "xp";
    pub const REFERRAL_SEED: &str = "referral";
    pub const STAKING_SEED: &str = "staking";
    pub const GUILD_SEED: &str = "guild";
    pub const NFT_COLLECTION_SEED: &str = "collection";
    pub const SPECIAL_CARD_SEED: &str = "special_card";
    pub const PROPOSAL_SEED: &str = "proposal";
    pub const VOTE_SEED: &str = "vote";
    pub const GLOBAL_STATE_SEED: &str = "global_state";
    pub const ANALYTICS_SEED: &str = "analytics";

    /// Activity description strings
    pub const ACTIVITY_LOGIN_DESC: &str = "Daily login";
    pub const ACTIVITY_POST_DESC: &str = "Social media post";
    pub const ACTIVITY_COMMENT_DESC: &str = "Social media comment";
    pub const ACTIVITY_LIKE_DESC: &str = "Social media like";
    pub const ACTIVITY_SHARE_DESC: &str = "Social media share";
    pub const ACTIVITY_FOLLOW_DESC: &str = "Social media follow";
    pub const ACTIVITY_QUEST_DESC: &str = "Daily quest completion";
    pub const ACTIVITY_MILESTONE_DESC: &str = "Milestone achievement";
    pub const ACTIVITY_VIRAL_DESC: &str = "Viral content creation";
    pub const ACTIVITY_REFERRAL_DESC: &str = "Successful referral";

    /// Error message strings
    pub const ERROR_INSUFFICIENT_FUNDS: &str = "Insufficient funds for operation";
    pub const ERROR_INVALID_AUTHORITY: &str = "Invalid authority for this operation";
    pub const ERROR_ACCOUNT_NOT_INITIALIZED: &str = "Account not properly initialized";
    pub const ERROR_INVALID_CALCULATION: &str = "Invalid calculation parameters";
    pub const ERROR_RATE_LIMIT_EXCEEDED: &str = "Rate limit exceeded";
    pub const ERROR_SUSPICIOUS_ACTIVITY: &str = "Suspicious activity detected";
    pub const ERROR_NETWORK_CONGESTION: &str = "Network congestion, try again later";
    pub const ERROR_FEATURE_DISABLED: &str = "Feature currently disabled";

    /// Success message strings
    pub const SUCCESS_MINING_CLAIMED: &str = "Mining rewards successfully claimed";
    pub const SUCCESS_XP_EARNED: &str = "XP successfully earned";
    pub const SUCCESS_REFERRAL_REGISTERED: &str = "Referral successfully registered";
    pub const SUCCESS_STAKE_DEPOSITED: &str = "Tokens successfully staked";
    pub const SUCCESS_GUILD_JOINED: &str = "Successfully joined guild";
    pub const SUCCESS_NFT_MINTED: &str = "NFT successfully minted";
    pub const SUCCESS_VOTE_CAST: &str = "Vote successfully cast";
}

/// Calculation helper constants and formulas
pub mod calculation_helpers {
    use super::program_constants::*;

    /// Helper function to calculate mining phase based on user count
    pub fn get_mining_phase(total_users: u64) -> u8 {
        if total_users < PHASE_1_USER_THRESHOLD {
            1
        } else if total_users < PHASE_2_USER_THRESHOLD {
            2
        } else if total_users < PHASE_3_USER_THRESHOLD {
            3
        } else {
            4
        }
    }

    /// Helper function to get base mining rate for current phase
    pub fn get_base_mining_rate(phase: u8) -> u64 {
        match phase {
            1 => PHASE_1_BASE_RATE,
            2 => PHASE_2_BASE_RATE,
            3 => PHASE_3_BASE_RATE,
            4 => PHASE_4_BASE_RATE,
            _ => PHASE_4_BASE_RATE, // Default to phase 4
        }
    }

    /// Helper function to get Finizen bonus multiplier
    pub fn get_finizen_bonus(phase: u8) -> u64 {
        match phase {
            1 => PHASE_1_FINIZEN_BONUS,
            2 => PHASE_2_FINIZEN_BONUS,
            3 => PHASE_3_FINIZEN_BONUS,
            4 => PHASE_4_FINIZEN_BONUS,
            _ => PHASE_4_FINIZEN_BONUS,
        }
    }

    /// Helper function to get daily mining cap
    pub fn get_daily_mining_cap(phase: u8) -> u64 {
        match phase {
            1 => PHASE_1_DAILY_CAP,
            2 => PHASE_2_DAILY_CAP,
            3 => PHASE_3_DAILY_CAP,
            4 => PHASE_4_DAILY_CAP,
            _ => PHASE_4_DAILY_CAP,
        }
    }

    /// Helper function to determine XP level tier
    pub fn get_xp_tier_from_level(level: u32) -> u8 {
        if level >= MYTHIC_LEVEL_START {
            6 // Mythic
        } else if level >= DIAMOND_LEVEL_START {
            5 // Diamond
        } else if level >= PLATINUM_LEVEL_START {
            4 // Platinum
        } else if level >= GOLD_LEVEL_START {
            3 // Gold
        } else if level >= SILVER_LEVEL_START {
            2 // Silver
        } else {
            1 // Bronze
        }
    }

    /// Helper function to get XP mining multiplier range
    pub fn get_xp_mining_multiplier_range(tier: u8) -> (u64, u64) {
        match tier {
            1 => (BRONZE_MINING_MULTIPLIER_MIN, BRONZE_MINING_MULTIPLIER_MAX),
            2 => (SILVER_MINING_MULTIPLIER_MIN, SILVER_MINING_MULTIPLIER_MAX),
            3 => (GOLD_MINING_MULTIPLIER_MIN, GOLD_MINING_MULTIPLIER_MAX),
            4 => (PLATINUM_MINING_MULTIPLIER_MIN, PLATINUM_MINING_MULTIPLIER_MAX),
            5 => (DIAMOND_MINING_MULTIPLIER_MIN, DIAMOND_MINING_MULTIPLIER_MAX),
            6 => (MYTHIC_MINING_MULTIPLIER_MIN, MYTHIC_MINING_MULTIPLIER_MAX),
            _ => (BRONZE_MINING_MULTIPLIER_MIN, BRONZE_MINING_MULTIPLIER_MAX),
        }
    }

    /// Helper function to determine RP tier from RP amount
    pub fn get_rp_tier(rp_amount: u64) -> u8 {
        if rp_amount >= AMBASSADOR_RP_MIN {
            5 // Ambassador
        } else if rp_amount >= LEADER_RP_MIN {
            4 // Leader
        } else if rp_amount >= INFLUENCER_RP_MIN {
            3 // Influencer
        } else if rp_amount >= CONNECTOR_RP_MIN {
            2 // Connector
        } else {
            1 // Explorer
        }
    }

    /// Helper function to get RP tier mining bonus
    pub fn get_rp_mining_bonus(tier: u8) -> u64 {
        match tier {
            1 => EXPLORER_MINING_BONUS,
            2 => CONNECTOR_MINING_BONUS,
            3 => INFLUENCER_MINING_BONUS,
            4 => LEADER_MINING_BONUS,
            5 => AMBASSADOR_MINING_BONUS,
            _ => EXPLORER_MINING_BONUS,
        }
    }

    /// Helper function to get referral network limits
    pub fn get_referral_limit(tier: u8) -> u32 {
        match tier {
            1 => EXPLORER_REFERRAL_LIMIT,
            2 => CONNECTOR_REFERRAL_LIMIT,
            3 => INFLUENCER_REFERRAL_LIMIT,
            4 => LEADER_REFERRAL_LIMIT,
            5 => AMBASSADOR_REFERRAL_LIMIT,
            _ => EXPLORER_REFERRAL_LIMIT,
        }
    }

    /// Helper function to determine staking tier
    pub fn get_staking_tier(staked_amount: u64) -> u8 {
        if staked_amount >= WHALE_STAKE_MIN {
            5 // Whale
        } else if staked_amount >= ELITE_STAKE_MIN {
            4 // Elite
        } else if staked_amount >= VIP_STAKE_MIN {
            3 // VIP
        } else if staked_amount >= PREMIUM_STAKE_MIN {
            2 // Premium
        } else if staked_amount >= BASIC_STAKE_MIN {
            1 // Basic
        } else {
            0 // No staking tier
        }
    }

    /// Helper function to get staking APY
    pub fn get_staking_apy(tier: u8) -> u64 {
        match tier {
            1 => BASIC_STAKE_APY,
            2 => PREMIUM_STAKE_APY,
            3 => VIP_STAKE_APY,
            4 => ELITE_STAKE_APY,
            5 => WHALE_STAKE_APY,
            _ => 0,
        }
    }

    /// Helper function to convert basis points to decimal multiplier
    pub fn basis_points_to_multiplier(basis_points: u64) -> f64 {
        basis_points as f64 / BASIS_POINTS_SCALE as f64
    }

    /// Helper function to convert decimal to basis points
    pub fn decimal_to_basis_points(decimal: f64) -> u64 {
        (decimal * BASIS_POINTS_SCALE as f64) as u64
    }

    /// Helper function to calculate exponential decay
    pub fn calculate_exponential_decay(holdings: u64, coefficient: u64) -> u64 {
        // Simplified exponential calculation for on-chain use
        // e^(-coefficient * holdings / PRECISION_MULTIPLIER)
        let exponent = (coefficient * holdings) / PRECISION_MULTIPLIER;
        
        if exponent == 0 {
            PRECISION_MULTIPLIER
        } else if exponent >= 20 * PRECISION_MULTIPLIER / 1000 {
            // Cap at very small value to prevent underflow
            1
        } else {
            // Use approximation: e^(-x) ≈ 1 / (1 + x + x²/2 + x³/6)
            let x = exponent;
            let x_squared = (x * x) / PRECISION_MULTIPLIER;
            let x_cubed = (x_squared * x) / PRECISION_MULTIPLIER;
            
            let denominator = PRECISION_MULTIPLIER + x + x_squared / 2 + x_cubed / 6;
            PRECISION_MULTIPLIER * PRECISION_MULTIPLIER / denominator
        }
    }
}
