/**
 * Finova Network API Constants
 * 
 * Comprehensive constants for the Finova Network Social-Fi Super App
 * Integrated XP, RP, and $FIN Mining System
 * 
 * @version 3.0
 * @author Finova Network Team
 * @created July 2025
 */

// ==================== TOKEN ECONOMICS ====================
export const TOKEN_CONFIG = {
  FIN: {
    SYMBOL: 'FIN',
    DECIMALS: 9,
    MAX_SUPPLY: 100_000_000_000, // 100 billion
    INITIAL_SUPPLY: 0,
    MINT_AUTHORITY_REQUIRED: true,
  },
  SFIN: {
    SYMBOL: 'sFIN',
    DECIMALS: 9,
    STAKING_APY_MIN: 8,
    STAKING_APY_MAX: 15,
  },
  USDFIN: {
    SYMBOL: 'USDfin',
    DECIMALS: 6,
    PEG_VALUE: 1.0, // 1:1 USD
  },
  SUSDFIN: {
    SYMBOL: 'sUSDfin',
    DECIMALS: 6,
    YIELD_APY_MIN: 4,
    YIELD_APY_MAX: 8,
  },
} as const;

// ==================== MINING SYSTEM ====================
export const MINING_CONFIG = {
  PHASES: {
    FINIZEN: {
      PHASE: 1,
      USER_RANGE: [0, 100_000],
      BASE_RATE: 0.1, // FIN/hour
      FINIZEN_BONUS: 2.0,
      MAX_DAILY: 4.8,
      DESCRIPTION: 'Early Pioneer Phase',
    },
    GROWTH: {
      PHASE: 2,
      USER_RANGE: [100_000, 1_000_000],
      BASE_RATE: 0.05,
      FINIZEN_BONUS: 1.5,
      MAX_DAILY: 1.8,
      DESCRIPTION: 'Network Growth Phase',
    },
    MATURITY: {
      PHASE: 3,
      USER_RANGE: [1_000_000, 10_000_000],
      BASE_RATE: 0.025,
      FINIZEN_BONUS: 1.2,
      MAX_DAILY: 0.72,
      DESCRIPTION: 'Market Maturity Phase',
    },
    STABILITY: {
      PHASE: 4,
      USER_RANGE: [10_000_000, Infinity],
      BASE_RATE: 0.01,
      FINIZEN_BONUS: 1.0,
      MAX_DAILY: 0.24,
      DESCRIPTION: 'Network Stability Phase',
    },
  },
  
  BONUSES: {
    KYC_VERIFIED: 1.2,
    KYC_UNVERIFIED: 0.8,
    REFERRAL_MULTIPLIER: 0.1, // per active referral
    MAX_REFERRAL_BONUS: 3.5,
  },
  
  REGRESSION: {
    COEFFICIENT: -0.001,
    WHALE_THRESHOLD: 100_000, // FIN
    PROGRESSIVE_TAX_RATES: [
      { min: 0, max: 1_000, rate: 0 },
      { min: 1_000, max: 10_000, rate: 0.05 },
      { min: 10_000, max: 100_000, rate: 0.15 },
      { min: 100_000, max: 1_000_000, rate: 0.35 },
      { min: 1_000_000, max: Infinity, rate: 0.50 },
    ],
  },

  ACTIVITY_BOOSTERS: [
    { type: 'DAILY_POST', boost: 0.2, duration: 24, stackable: true, max_stack: 3 },
    { type: 'DAILY_QUEST', boost: 0.5, duration: 12, stackable: false },
    { type: 'REFERRAL_KYC', boost: 1.0, duration: 48, stackable: true, max_stack: 5 },
    { type: 'SPECIAL_CARD', boost: 2.0, duration: 'variable', stackable: true },
    { type: 'GUILD_EVENT', boost: 0.3, duration: 'event', stackable: true },
  ],
} as const;

// ==================== XP SYSTEM ====================
export const XP_CONFIG = {
  ACTIVITIES: {
    // Content Creation
    ORIGINAL_POST: { base_xp: 50, daily_limit: null, quality_range: [0.8, 1.5] },
    PHOTO_POST: { base_xp: 75, daily_limit: 20, quality_range: [0.9, 1.8] },
    VIDEO_CONTENT: { base_xp: 150, daily_limit: 10, quality_range: [1.0, 2.0] },
    STORY_STATUS: { base_xp: 25, daily_limit: 50, quality_range: [0.7, 1.2] },
    
    // Engagement
    MEANINGFUL_COMMENT: { base_xp: 25, daily_limit: 100, quality_range: [0.5, 1.5] },
    LIKE_REACT: { base_xp: 5, daily_limit: 200, quality_range: [1.0, 1.0] },
    SHARE_REPOST: { base_xp: 15, daily_limit: 50, quality_range: [0.8, 1.3] },
    FOLLOW_SUBSCRIBE: { base_xp: 20, daily_limit: 25, quality_range: [1.0, 1.0] },
    
    // Special Actions
    DAILY_LOGIN: { base_xp: 10, daily_limit: 1, quality_range: [1.0, 1.0] },
    DAILY_QUEST: { base_xp: 100, daily_limit: 3, quality_range: [1.0, 1.0] },
    MILESTONE: { base_xp: 500, daily_limit: null, quality_range: [1.0, 1.0] },
    VIRAL_CONTENT: { base_xp: 1000, daily_limit: null, quality_range: [2.0, 2.0] },
  },

  PLATFORM_MULTIPLIERS: {
    TIKTOK: 1.3,
    INSTAGRAM: 1.2,
    YOUTUBE: 1.4,
    FACEBOOK: 1.1,
    TWITTER_X: 1.2,
    DEFAULT: 1.0,
  },

  LEVEL_SYSTEM: [
    // Bronze Tier (1-10)
    ...Array.from({ length: 10 }, (_, i) => ({
      level: i + 1,
      xp_required: i * 100,
      tier: 'BRONZE',
      mining_multiplier: 1.0 + (i * 0.02),
      daily_fin_cap: 0.5 + (i * 0.15),
      badge: `BRONZE_${['I', 'II', 'III', 'IV', 'V', 'VI', 'VII', 'VIII', 'IX', 'X'][i]}`,
    })),
    
    // Silver Tier (11-25)
    ...Array.from({ length: 15 }, (_, i) => ({
      level: i + 11,
      xp_required: 1000 + (i * 250),
      tier: 'SILVER',
      mining_multiplier: 1.3 + (i * 0.033),
      daily_fin_cap: 2.0 + (i * 0.133),
      badge: `SILVER_${i + 1}`,
    })),
    
    // Gold Tier (26-50)
    ...Array.from({ length: 25 }, (_, i) => ({
      level: i + 26,
      xp_required: 5000 + (i * 600),
      tier: 'GOLD',
      mining_multiplier: 1.9 + (i * 0.024),
      daily_fin_cap: 4.0 + (i * 0.08),
      badge: `GOLD_${i + 1}`,
    })),
    
    // Platinum Tier (51-75)
    ...Array.from({ length: 25 }, (_, i) => ({
      level: i + 51,
      xp_required: 20000 + (i * 1200),
      tier: 'PLATINUM',
      mining_multiplier: 2.6 + (i * 0.024),
      daily_fin_cap: 6.0 + (i * 0.08),
      badge: `PLATINUM_${i + 1}`,
    })),
    
    // Diamond Tier (76-100)
    ...Array.from({ length: 25 }, (_, i) => ({
      level: i + 76,
      xp_required: 50000 + (i * 2000),
      tier: 'DIAMOND',
      mining_multiplier: 3.3 + (i * 0.028),
      daily_fin_cap: 8.0 + (i * 0.08),
      badge: `DIAMOND_${i + 1}`,
    })),
    
    // Mythic Tier (101+)
    {
      level: 101,
      xp_required: 100000,
      tier: 'MYTHIC',
      mining_multiplier: 4.1,
      daily_fin_cap: 10.0,
      badge: 'MYTHIC_I',
    },
  ],

  STREAK_BONUSES: [
    { days: 1, multiplier: 1.0 },
    { days: 3, multiplier: 1.1 },
    { days: 7, multiplier: 1.2 },
    { days: 14, multiplier: 1.4 },
    { days: 30, multiplier: 1.6 },
    { days: 90, multiplier: 2.0 },
    { days: 365, multiplier: 3.0 },
  ],

  LEVEL_PROGRESSION_DECAY: 0.01, // e^(-0.01 * level)
} as const;

// ==================== REFERRAL POINTS SYSTEM ====================
export const RP_CONFIG = {
  ACTIONS: {
    SIGNUP_WITH_CODE: { rp: 50, network_size: 1, permanent: true },
    COMPLETE_KYC: { rp: 100, network_size: 2, permanent: true },
    FIRST_FIN_EARNED: { rp: 25, network_size: 0.5, permanent: true },
    DAILY_MINING: { rp_percentage: 0.1, compound: true },
    XP_GAINS: { rp_percentage: 0.05, realtime: true },
    ACHIEVEMENTS: { rp: 50, milestone_based: true },
  },

  NETWORK_BONUSES: [
    { active_referrals: 10, rp_bonus: 500, multiplier: 0.5, permanent: true },
    { active_referrals: 25, rp_bonus: 1500, multiplier: 1.0, permanent: true },
    { active_referrals: 50, rp_bonus: 5000, multiplier: 1.5, permanent: true },
    { active_referrals: 100, rp_bonus: 15000, multiplier: 2.0, ambassador: true },
  ],

  TIER_SYSTEM: [
    {
      tier: 'EXPLORER',
      rp_range: [0, 999],
      mining_bonus: 0,
      referral_l1: 0.1,
      referral_l2: 0,
      referral_l3: 0,
      network_cap: 10,
      features: ['basic_referral_link'],
    },
    {
      tier: 'CONNECTOR',
      rp_range: [1000, 4999],
      mining_bonus: 0.2,
      referral_l1: 0.15,
      referral_l2: 0.05,
      referral_l3: 0,
      network_cap: 25,
      features: ['custom_referral_code'],
    },
    {
      tier: 'INFLUENCER',
      rp_range: [5000, 14999],
      mining_bonus: 0.5,
      referral_l1: 0.2,
      referral_l2: 0.08,
      referral_l3: 0.03,
      network_cap: 50,
      features: ['referral_analytics'],
    },
    {
      tier: 'LEADER',
      rp_range: [15000, 49999],
      mining_bonus: 1.0,
      referral_l1: 0.25,
      referral_l2: 0.1,
      referral_l3: 0.05,
      network_cap: 100,
      features: ['exclusive_events'],
    },
    {
      tier: 'AMBASSADOR',
      rp_range: [50000, Infinity],
      mining_bonus: 2.0,
      referral_l1: 0.3,
      referral_l2: 0.15,
      referral_l3: 0.08,
      network_cap: null,
      features: ['dao_governance', 'ambassador_status'],
    },
  ],

  NETWORK_REGRESSION: {
    COEFFICIENT: -0.0001,
    QUALITY_WEIGHT: 1.0,
    ACTIVITY_THRESHOLD: 0.3, // 30% activity required
  },
} as const;

// ==================== NFT & SPECIAL CARDS ====================
export const NFT_CONFIG = {
  COLLECTIONS: {
    SPECIAL_CARDS: 'finova_special_cards',
    PROFILE_BADGES: 'finova_profile_badges',
    ACHIEVEMENT_NFTS: 'finova_achievements',
  },

  SPECIAL_CARDS: {
    MINING_BOOST: [
      { name: 'DOUBLE_MINING', effect: 1.0, duration: 24, rarity: 'COMMON', price: 50 },
      { name: 'TRIPLE_MINING', effect: 2.0, duration: 12, rarity: 'RARE', price: 150 },
      { name: 'MINING_FRENZY', effect: 5.0, duration: 4, rarity: 'EPIC', price: 500 },
      { name: 'ETERNAL_MINER', effect: 0.5, duration: 720, rarity: 'LEGENDARY', price: 2000 },
    ],

    XP_ACCELERATOR: [
      { name: 'XP_DOUBLE', effect: 1.0, duration: 24, rarity: 'COMMON', price: 40 },
      { name: 'STREAK_SAVER', effect: 'maintain_streak', duration: 168, rarity: 'UNCOMMON', price: 80 },
      { name: 'LEVEL_RUSH', effect: 500, duration: 0, rarity: 'RARE', price: 120 },
      { name: 'XP_MAGNET', effect: 3.0, duration: 48, rarity: 'EPIC', price: 300 },
    ],

    REFERRAL_POWER: [
      { name: 'REFERRAL_BOOST', effect: 0.5, duration: 168, rarity: 'COMMON', price: 60 },
      { name: 'NETWORK_AMPLIFIER', effect: '+2_tiers', duration: 24, rarity: 'RARE', price: 200 },
      { name: 'AMBASSADOR_PASS', effect: 'temp_ambassador', duration: 48, rarity: 'EPIC', price: 400 },
      { name: 'NETWORK_KING', effect: 1.0, duration: 12, rarity: 'LEGENDARY', price: 1000 },
    ],
  },

  RARITY_MULTIPLIERS: {
    COMMON: 0,
    UNCOMMON: 0.05,
    RARE: 0.10,
    EPIC: 0.20,
    LEGENDARY: 0.35,
  },

  SYNERGY_BONUSES: {
    SAME_CATEGORY: 0.15,
    ALL_CATEGORIES: 0.30,
    CARD_COUNT_MULTIPLIER: 0.10,
  },

  ACHIEVEMENT_NFTS: {
    FINIZEN: { requirement: 'first_1000_users', bonus: 0.25, permanent: true },
    CONTENT_KING: { requirement: 'viral_creator', bonus: 0.50, xp_bonus: true },
    AMBASSADOR: { requirement: 'network_builder', bonus: 0.30, rp_bonus: true },
    DIAMOND_HANDS: { requirement: 'whale_staker', bonus: 0.20, staking_bonus: true },
  },
} as const;

// ==================== STAKING SYSTEM ====================
export const STAKING_CONFIG = {
  TIERS: [
    {
      min_stake: 100,
      max_stake: 499,
      sfin_apy: 8,
      mining_boost: 0.20,
      xp_multiplier: 0.10,
      rp_bonus: 0.05,
      features: ['basic_staking'],
    },
    {
      min_stake: 500,
      max_stake: 999,
      sfin_apy: 10,
      mining_boost: 0.35,
      xp_multiplier: 0.20,
      rp_bonus: 0.10,
      features: ['premium_badge', 'priority_support'],
    },
    {
      min_stake: 1000,
      max_stake: 4999,
      sfin_apy: 12,
      mining_boost: 0.50,
      xp_multiplier: 0.30,
      rp_bonus: 0.20,
      features: ['vip_features', 'exclusive_events'],
    },
    {
      min_stake: 5000,
      max_stake: 9999,
      sfin_apy: 14,
      mining_boost: 0.75,
      xp_multiplier: 0.50,
      rp_bonus: 0.35,
      features: ['guild_master_privileges'],
    },
    {
      min_stake: 10000,
      max_stake: Infinity,
      sfin_apy: 15,
      mining_boost: 1.00,
      xp_multiplier: 0.75,
      rp_bonus: 0.50,
      features: ['dao_governance', 'maximum_benefits'],
    },
  ],

  REWARD_POOL_DISTRIBUTION: {
    BASE_STAKING: 0.40,
    ACTIVITY_BONUS: 0.25,
    LOYALTY_REWARDS: 0.20,
    PERFORMANCE_INCENTIVES: 0.10,
    SPECIAL_EVENTS: 0.05,
  },

  LOYALTY_MULTIPLIER: 0.05, // per month
  ACTIVITY_MULTIPLIER: 0.10, // based on daily activity score
} as const;

// ==================== GUILD SYSTEM ====================
export const GUILD_CONFIG = {
  REQUIREMENTS: {
    MIN_LEVEL: 11, // Silver tier
    MIN_MEMBERS: 10,
    MAX_MEMBERS: 50,
    LEADERSHIP_LEVEL: 26, // Gold tier for Guild Master
  },

  COMPETITIONS: {
    DAILY_CHALLENGES: { duration: 24, xp_bonus: 0.20, type: 'individual' },
    WEEKLY_WARS: { duration: 168, reward_type: 'treasury', type: 'team_vs_team' },
    MONTHLY_CHAMPIONSHIPS: { duration: 720, reward_type: 'rare_nft', type: 'tournament' },
    SEASONAL_LEAGUES: { duration: 2160, reward_type: 'massive_fin', type: 'ranking' },
  },

  ROLES: {
    GUILD_MASTER: { permissions: ['all'], voting_weight: 3 },
    OFFICER: { permissions: ['moderate', 'invite', 'kick'], voting_weight: 2 },
    MEMBER: { permissions: ['participate'], voting_weight: 1 },
  },
} as const;

// ==================== ANTI-BOT SYSTEM ====================
export const ANTI_BOT_CONFIG = {
  HUMAN_VERIFICATION: {
    BIOMETRIC_WEIGHT: 0.25,
    BEHAVIORAL_WEIGHT: 0.20,
    SOCIAL_GRAPH_WEIGHT: 0.20,
    DEVICE_WEIGHT: 0.15,
    INTERACTION_WEIGHT: 0.20,
  },

  SUSPICIOUS_THRESHOLDS: {
    CLICK_SPEED_MAX: 10, // clicks per second
    SESSION_LENGTH_MIN: 300, // seconds
    CONTENT_SIMILARITY_MAX: 0.8,
    NETWORK_CLUSTERING_MAX: 0.9,
  },

  PENALTIES: {
    MINING_REDUCTION: [0.85, 0.70, 0.50, 0.25, 0.10], // by offense level
    XP_REDUCTION: [0.90, 0.75, 0.50, 0.20, 0.05],
    RP_SUSPENSION: [false, false, true, true, true],
    ACCOUNT_REVIEW: [false, false, true, true, true],
  },

  DIFFICULTY_SCALING: {
    FIN_COEFFICIENT: 0.001, // per 1000 FIN earned
    SUSPICIOUS_MULTIPLIER: 2.0,
    MAX_DIFFICULTY: 0.9,
  },
} as const;

// ==================== API CONFIGURATION ====================
export const API_CONFIG = {
  RATE_LIMITS: {
    MINING_CHECK: { requests: 60, window: 3600 }, // 1 per minute
    XP_SUBMISSION: { requests: 1000, window: 3600 }, // High for social activities
    RP_CALCULATION: { requests: 100, window: 3600 },
    NFT_OPERATIONS: { requests: 50, window: 3600 },
    AUTH_ENDPOINTS: { requests: 10, window: 900 }, // 15 minutes
  },

  CACHE_DURATIONS: {
    USER_PROFILE: 300, // 5 minutes
    MINING_RATE: 60, // 1 minute
    XP_LEVELS: 3600, // 1 hour
    RP_TIERS: 3600, // 1 hour
    NFT_METADATA: 86400, // 24 hours
    LEADERBOARDS: 600, // 10 minutes
  },

  PAGINATION: {
    DEFAULT_LIMIT: 20,
    MAX_LIMIT: 100,
    OFFSET_MAX: 10000,
  },
} as const;

// ==================== SOCIAL PLATFORM INTEGRATION ====================
export const SOCIAL_PLATFORMS = {
  INSTAGRAM: {
    name: 'Instagram',
    api_version: 'v18.0',
    base_url: 'https://graph.instagram.com',
    supported_content: ['post', 'story', 'reel'],
    xp_multiplier: 1.2,
    rate_limits: { posts: 100, comments: 500 },
  },
  TIKTOK: {
    name: 'TikTok',
    api_version: 'v2',
    base_url: 'https://open-api.tiktok.com',
    supported_content: ['video', 'live'],
    xp_multiplier: 1.3,
    rate_limits: { posts: 50, comments: 200 },
  },
  YOUTUBE: {
    name: 'YouTube',
    api_version: 'v3',
    base_url: 'https://www.googleapis.com/youtube/v3',
    supported_content: ['video', 'short', 'community'],
    xp_multiplier: 1.4,
    rate_limits: { posts: 20, comments: 100 },
  },
  FACEBOOK: {
    name: 'Facebook',
    api_version: 'v18.0',
    base_url: 'https://graph.facebook.com',
    supported_content: ['post', 'story', 'reel'],
    xp_multiplier: 1.1,
    rate_limits: { posts: 200, comments: 1000 },
  },
  TWITTER_X: {
    name: 'X (Twitter)',
    api_version: 'v2',
    base_url: 'https://api.twitter.com/2',
    supported_content: ['tweet', 'retweet', 'reply'],
    xp_multiplier: 1.2,
    rate_limits: { posts: 300, comments: 1000 },
  },
} as const;

// ==================== ERROR CODES ====================
export const ERROR_CODES = {
  // Authentication
  AUTH_INVALID_TOKEN: 'AUTH_001',
  AUTH_EXPIRED_TOKEN: 'AUTH_002',
  AUTH_INSUFFICIENT_PERMISSIONS: 'AUTH_003',
  
  // Mining
  MINING_RATE_LIMITED: 'MINING_001',
  MINING_INSUFFICIENT_KYC: 'MINING_002',
  MINING_DAILY_LIMIT_EXCEEDED: 'MINING_003',
  MINING_SUSPICIOUS_ACTIVITY: 'MINING_004',
  
  // XP System
  XP_DAILY_LIMIT_EXCEEDED: 'XP_001',
  XP_INVALID_PLATFORM: 'XP_002',
  XP_CONTENT_QUALITY_LOW: 'XP_003',
  
  // Referral System
  RP_INVALID_CODE: 'RP_001',
  RP_SELF_REFERRAL: 'RP_002',
  RP_NETWORK_LIMIT_EXCEEDED: 'RP_003',
  
  // NFT Operations
  NFT_INSUFFICIENT_BALANCE: 'NFT_001',
  NFT_INVALID_METADATA: 'NFT_002',
  NFT_MARKETPLACE_ERROR: 'NFT_003',
  
  // Staking
  STAKING_INSUFFICIENT_AMOUNT: 'STAKE_001',
  STAKING_COOLDOWN_ACTIVE: 'STAKE_002',
  STAKING_INVALID_TIER: 'STAKE_003',
  
  // General
  VALIDATION_ERROR: 'GEN_001',
  DATABASE_ERROR: 'GEN_002',
  BLOCKCHAIN_ERROR: 'GEN_003',
  EXTERNAL_API_ERROR: 'GEN_004',
} as const;

// ==================== WEBHOOK EVENTS ====================
export const WEBHOOK_EVENTS = {
  MINING: {
    REWARD_EARNED: 'mining.reward_earned',
    DAILY_LIMIT_REACHED: 'mining.daily_limit_reached',
    PHASE_TRANSITION: 'mining.phase_transition',
  },
  XP: {
    LEVEL_UP: 'xp.level_up',
    TIER_PROMOTION: 'xp.tier_promotion',
    STREAK_MILESTONE: 'xp.streak_milestone',
  },
  REFERRAL: {
    NEW_REFERRAL: 'referral.new_referral',
    TIER_UPGRADE: 'referral.tier_upgrade',
    NETWORK_MILESTONE: 'referral.network_milestone',
  },
  NFT: {
    CARD_USED: 'nft.card_used',
    ACHIEVEMENT_UNLOCKED: 'nft.achievement_unlocked',
    MARKETPLACE_SALE: 'nft.marketplace_sale',
  },
} as const;

// ==================== FORMULA CONSTANTS ====================
export const FORMULAS = {
  // Mining calculation coefficients
  MINING_REGRESSION_COEFFICIENT: -0.001,
  FINIZEN_BONUS_DIVISOR: 1_000_000,
  
  // XP progression
  XP_LEVEL_DECAY: -0.01,
  XP_QUALITY_MIN: 0.5,
  XP_QUALITY_MAX: 2.0,
  
  // RP network effects
  RP_NETWORK_REGRESSION: -0.0001,
  RP_QUALITY_THRESHOLD: 0.3,
  
  // Staking calculations
  LOYALTY_BONUS_PER_MONTH: 0.05,
  ACTIVITY_BONUS_MAX: 2.0,
  
  // Economic sustainability
  REVENUE_TO_REWARDS_RATIO: 0.6,
  BURN_RATE_TRANSACTION: 0.001,
} as const;

// ==================== FEATURE FLAGS ====================
export const FEATURE_FLAGS = {
  MINING_ENABLED: true,
  XP_SYSTEM_ENABLED: true,
  REFERRAL_SYSTEM_ENABLED: true,
  NFT_MARKETPLACE_ENABLED: true,
  STAKING_ENABLED: true,
  GUILD_SYSTEM_ENABLED: true,
  
  // Beta features
  AI_CONTENT_ANALYSIS: true,
  ADVANCED_ANTI_BOT: true,
  CROSS_CHAIN_BRIDGE: false,
  DEFI_INTEGRATION: false,
  
  // Regional features
  INDONESIA_EWALLET: true,
  GLOBAL_KYC: true,
  MULTI_LANGUAGE: false,
} as const;

// ==================== VERSION INFO ====================
export const VERSION_INFO = {
  API_VERSION: '3.0.0',
  PROTOCOL_VERSION: '1.0.0',
  LAST_UPDATED: '2025-07-25',
  COMPATIBILITY: {
    MIN_CLIENT_VERSION: '3.0.0',
    MIN_MOBILE_VERSION: '3.0.0',
  },
} as const;

// Export all constants as default
export default {
  TOKEN_CONFIG,
  MINING_CONFIG,
  XP_CONFIG,
  RP_CONFIG,
  NFT_CONFIG,
  STAKING_CONFIG,
  GUILD_CONFIG,
  ANTI_BOT_CONFIG,
  API_CONFIG,
  SOCIAL_PLATFORMS,
  ERROR_CODES,
  WEBHOOK_EVENTS,
  FORMULAS,
  FEATURE_FLAGS,
  VERSION_INFO,
} as const;
