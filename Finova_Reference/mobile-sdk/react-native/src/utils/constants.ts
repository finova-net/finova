/**
 * Finova Network React Native SDK - Constants
 * Version: 1.0.0
 * 
 * Enterprise-grade constants for the complete Finova ecosystem
 * Supports: Mining, XP, RP, Staking, NFT, Security, and all integrated systems
 */

// ==========================================
// CORE NETWORK CONFIGURATION
// ==========================================

export const FINOVA_NETWORK = {
  CLUSTER: {
    MAINNET: 'https://api.mainnet-beta.solana.com',
    TESTNET: 'https://api.testnet.solana.com',
    DEVNET: 'https://api.devnet.solana.com',
  },
  PROGRAM_IDS: {
    FINOVA_CORE: 'FinovaCoreProgram11111111111111111111111111',
    FINOVA_TOKEN: 'FinovaTokenProgram111111111111111111111111',
    FINOVA_NFT: 'FinovaNFTProgram1111111111111111111111111111',
    FINOVA_DEFI: 'FinovaDeFiProgram111111111111111111111111111',
    FINOVA_BRIDGE: 'FinovaBridgeProgram11111111111111111111111',
    FINOVA_ORACLE: 'FinovaOracleProgram11111111111111111111111',
  },
} as const;

// ==========================================
// TOKEN ECONOMICS
// ==========================================

export const TOKEN_CONFIG = {
  FIN: {
    SYMBOL: '$FIN',
    DECIMALS: 9,
    MAX_SUPPLY: 100_000_000_000, // 100 billion
    MINT_ADDRESS: 'FinTokenMint111111111111111111111111111111',
    INITIAL_MINING_RATE: 0.1, // per hour in Phase 1
  },
  S_FIN: {
    SYMBOL: '$sFIN',
    DECIMALS: 9,
    MINT_ADDRESS: 'sFinTokenMint11111111111111111111111111111',
    MIN_STAKE_AMOUNT: 100,
    UNBONDING_PERIOD: 7 * 24 * 60 * 60 * 1000, // 7 days in ms
  },
  USD_FIN: {
    SYMBOL: '$USDfin',
    DECIMALS: 6,
    PEG_RATE: 1.0, // 1:1 USD
    MINT_ADDRESS: 'USDFinMint1111111111111111111111111111111',
  },
  S_USD_FIN: {
    SYMBOL: '$sUSDfin',
    DECIMALS: 6,
    MINT_ADDRESS: 'sUSDFinMint111111111111111111111111111111',
    BASE_APY: 0.04, // 4% base APY
    MAX_APY: 0.08, // 8% max APY
  },
} as const;

// ==========================================
// MINING SYSTEM CONSTANTS
// ==========================================

export const MINING_CONFIG = {
  PHASES: {
    FINIZEN: {
      PHASE: 1,
      USER_RANGE: [0, 100_000],
      BASE_RATE: 0.1, // $FIN/hour
      FINIZEN_BONUS: 2.0,
      MAX_DAILY: 4.8,
      DESCRIPTION: 'Early Adopter Phase',
    },
    GROWTH: {
      PHASE: 2,
      USER_RANGE: [100_001, 1_000_000],
      BASE_RATE: 0.05,
      FINIZEN_BONUS: 1.5,
      MAX_DAILY: 1.8,
      DESCRIPTION: 'Growth Phase',
    },
    MATURITY: {
      PHASE: 3,
      USER_RANGE: [1_000_001, 10_000_000],
      BASE_RATE: 0.025,
      FINIZEN_BONUS: 1.2,
      MAX_DAILY: 0.72,
      DESCRIPTION: 'Maturity Phase',
    },
    STABILITY: {
      PHASE: 4,
      USER_RANGE: [10_000_001, Infinity],
      BASE_RATE: 0.01,
      FINIZEN_BONUS: 1.0,
      MAX_DAILY: 0.24,
      DESCRIPTION: 'Stability Phase',
    },
  },
  BONUSES: {
    REFERRAL_MULTIPLIER: 0.1, // 10% per active referral
    MAX_REFERRAL_BONUS: 3.5, // Maximum 350% bonus
    KYC_SECURITY_BONUS: 1.2,
    NON_KYC_PENALTY: 0.8,
    REGRESSION_COEFFICIENT: 0.001, // For exponential decay
  },
  ACTIVITY_BOOSTERS: {
    DAILY_POST: { BOOST: 0.2, DURATION: 24 * 60 * 60 * 1000, MAX_STACK: 3 },
    DAILY_QUEST: { BOOST: 0.5, DURATION: 12 * 60 * 60 * 1000, MAX_STACK: 1 },
    REFERRAL_KYC: { BOOST: 1.0, DURATION: 48 * 60 * 60 * 1000, MAX_STACK: 5 },
    SPECIAL_CARD: { BOOST: 2.0, DURATION: 0, MAX_STACK: 99 }, // Variable duration
    GUILD_EVENT: { BOOST: 0.3, DURATION: 0, MAX_STACK: 1 }, // Event-based
  },
} as const;

// ==========================================
// EXPERIENCE POINTS (XP) SYSTEM
// ==========================================

export const XP_CONFIG = {
  ACTIVITIES: {
    // Content Creation
    ORIGINAL_POST: { BASE_XP: 50, DAILY_LIMIT: null, QUALITY_RANGE: [0.8, 1.5] },
    PHOTO_POST: { BASE_XP: 75, DAILY_LIMIT: 20, QUALITY_RANGE: [0.9, 1.8] },
    VIDEO_CONTENT: { BASE_XP: 150, DAILY_LIMIT: 10, QUALITY_RANGE: [1.0, 2.0] },
    STORY_STATUS: { BASE_XP: 25, DAILY_LIMIT: 50, QUALITY_RANGE: [0.7, 1.2] },
    
    // Engagement
    MEANINGFUL_COMMENT: { BASE_XP: 25, DAILY_LIMIT: 100, QUALITY_RANGE: [0.5, 1.5] },
    LIKE_REACT: { BASE_XP: 5, DAILY_LIMIT: 200, QUALITY_RANGE: [1.0, 1.0] },
    SHARE_REPOST: { BASE_XP: 15, DAILY_LIMIT: 50, QUALITY_RANGE: [0.8, 1.3] },
    FOLLOW_SUBSCRIBE: { BASE_XP: 20, DAILY_LIMIT: 25, QUALITY_RANGE: [1.0, 1.0] },
    
    // Special Actions
    FIRST_DAILY_LOGIN: { BASE_XP: 10, DAILY_LIMIT: 1, QUALITY_RANGE: [1.0, 1.0] },
    COMPLETE_DAILY_QUEST: { BASE_XP: 100, DAILY_LIMIT: 3, QUALITY_RANGE: [1.0, 1.0] },
    ACHIEVE_MILESTONE: { BASE_XP: 500, DAILY_LIMIT: null, QUALITY_RANGE: [1.0, 1.0] },
    VIRAL_CONTENT: { BASE_XP: 1000, DAILY_LIMIT: null, QUALITY_RANGE: [2.0, 2.0] },
  },
  PLATFORM_MULTIPLIERS: {
    TIKTOK: 1.3,
    INSTAGRAM: 1.2,
    YOUTUBE: 1.4,
    FACEBOOK: 1.1,
    TWITTER_X: 1.2,
    DEFAULT: 1.0,
  },
  LEVEL_SYSTEM: {
    BRONZE: { RANGE: [1, 10], XP_REQUIRED: [0, 999], MINING_MULTIPLIER: [1.0, 1.2], DAILY_CAP: [0.5, 2.0] },
    SILVER: { RANGE: [11, 25], XP_REQUIRED: [1000, 4999], MINING_MULTIPLIER: [1.3, 1.8], DAILY_CAP: [2.0, 4.0] },
    GOLD: { RANGE: [26, 50], XP_REQUIRED: [5000, 19999], MINING_MULTIPLIER: [1.9, 2.5], DAILY_CAP: [4.0, 6.0] },
    PLATINUM: { RANGE: [51, 75], XP_REQUIRED: [20000, 49999], MINING_MULTIPLIER: [2.6, 3.2], DAILY_CAP: [6.0, 8.0] },
    DIAMOND: { RANGE: [76, 100], XP_REQUIRED: [50000, 99999], MINING_MULTIPLIER: [3.3, 4.0], DAILY_CAP: [8.0, 10.0] },
    MYTHIC: { RANGE: [101, 999], XP_REQUIRED: [100000, Infinity], MINING_MULTIPLIER: [4.1, 5.0], DAILY_CAP: [10.0, 15.0] },
  },
  PROGRESSION_COEFFICIENT: 0.01, // For exponential level progression
} as const;

// ==========================================
// REFERRAL POINTS (RP) SYSTEM
// ==========================================

export const RP_CONFIG = {
  EARNING_ACTIONS: {
    REGISTRATION: { RP: 50, NETWORK_SIZE: 1, DURATION: 'permanent' },
    COMPLETE_KYC: { RP: 100, NETWORK_SIZE: 2, DURATION: 'permanent' },
    FIRST_FIN_EARNED: { RP: 25, NETWORK_SIZE: 0.5, DURATION: 'permanent' },
    DAILY_MINING_SHARE: { PERCENTAGE: 0.1, DURATION: 'daily' },
    XP_GAINS_SHARE: { PERCENTAGE: 0.05, DURATION: 'realtime' },
    MILESTONE_ACHIEVEMENT: { RP: 50, DURATION: 'per_milestone' },
  },
  NETWORK_BONUSES: {
    10: { RP: 500, MULTIPLIER: 0.5, REQUIREMENT: '30_day_activity' },
    25: { RP: 1500, MULTIPLIER: 1.0, REQUIREMENT: '30_day_activity' },
    50: { RP: 5000, MULTIPLIER: 1.5, REQUIREMENT: '30_day_activity' },
    100: { RP: 15000, MULTIPLIER: 2.0, REQUIREMENT: 'ambassador_status' },
  },
  TIERS: {
    EXPLORER: { 
      RP_RANGE: [0, 999], 
      MINING_BONUS: 0.0, 
      REFERRAL_BONUS: 0.1, 
      NETWORK_CAP: 10,
      FEATURES: ['basic_referral_link']
    },
    CONNECTOR: { 
      RP_RANGE: [1000, 4999], 
      MINING_BONUS: 0.2, 
      REFERRAL_BONUS: 0.15, 
      NETWORK_CAP: 25,
      FEATURES: ['custom_referral_code', 'basic_analytics']
    },
    INFLUENCER: { 
      RP_RANGE: [5000, 14999], 
      MINING_BONUS: 0.5, 
      REFERRAL_BONUS: 0.2, 
      NETWORK_CAP: 50,
      FEATURES: ['referral_analytics', 'priority_support']
    },
    LEADER: { 
      RP_RANGE: [15000, 49999], 
      MINING_BONUS: 1.0, 
      REFERRAL_BONUS: 0.25, 
      NETWORK_CAP: 100,
      FEATURES: ['exclusive_events', 'advanced_analytics']
    },
    AMBASSADOR: { 
      RP_RANGE: [50000, Infinity], 
      MINING_BONUS: 2.0, 
      REFERRAL_BONUS: 0.3, 
      NETWORK_CAP: Infinity,
      FEATURES: ['dao_governance', 'ambassador_program']
    },
  },
  NETWORK_LEVELS: {
    L1_SHARE: { DIRECT: 1.0, MINING: 0.3, XP: 0.05 },
    L2_SHARE: { INDIRECT: 0.3, MINING: 0.1, XP: 0.02 },
    L3_SHARE: { INDIRECT: 0.1, MINING: 0.05, XP: 0.01 },
  },
  REGRESSION_COEFFICIENT: 0.0001, // For network quality-based regression
} as const;

// ==========================================
// STAKING SYSTEM
// ==========================================

export const STAKING_CONFIG = {
  TIERS: [
    {
      MIN_STAKE: 100,
      MAX_STAKE: 499,
      APY: 0.08,
      MINING_BOOST: 0.2,
      XP_MULTIPLIER: 0.1,
      RP_BONUS: 0.05,
      FEATURES: ['basic_staking_rewards'],
    },
    {
      MIN_STAKE: 500,
      MAX_STAKE: 999,
      APY: 0.10,
      MINING_BOOST: 0.35,
      XP_MULTIPLIER: 0.2,
      RP_BONUS: 0.1,
      FEATURES: ['premium_badge', 'priority_support'],
    },
    {
      MIN_STAKE: 1000,
      MAX_STAKE: 4999,
      APY: 0.12,
      MINING_BOOST: 0.5,
      XP_MULTIPLIER: 0.3,
      RP_BONUS: 0.2,
      FEATURES: ['vip_features', 'exclusive_events'],
    },
    {
      MIN_STAKE: 5000,
      MAX_STAKE: 9999,
      APY: 0.14,
      MINING_BOOST: 0.75,
      XP_MULTIPLIER: 0.5,
      RP_BONUS: 0.35,
      FEATURES: ['guild_master_privileges'],
    },
    {
      MIN_STAKE: 10000,
      MAX_STAKE: Infinity,
      APY: 0.15,
      MINING_BOOST: 1.0,
      XP_MULTIPLIER: 0.75,
      RP_BONUS: 0.5,
      FEATURES: ['dao_governance', 'max_benefits'],
    },
  ],
  REWARD_POOL_ALLOCATION: {
    BASE_STAKING: 0.4,
    ACTIVITY_BONUS: 0.25,
    LOYALTY_REWARDS: 0.2,
    PERFORMANCE_INCENTIVES: 0.1,
    SPECIAL_EVENTS: 0.05,
  },
  LOYALTY_BONUS_PER_MONTH: 0.05, // 5% per month staked
  ACTIVITY_BONUS_COEFFICIENT: 0.1, // Based on daily activity score
} as const;

// ==========================================
// NFT & SPECIAL CARDS
// ==========================================

export const NFT_CONFIG = {
  COLLECTIONS: {
    MINING_BOOST_CARDS: 'FinovaMiningBoostCollection111111111111111',
    XP_ACCELERATOR_CARDS: 'FinovaXPAcceleratorCollection11111111111',
    REFERRAL_POWER_CARDS: 'FinovaReferralPowerCollection1111111111',
    PROFILE_BADGES: 'FinovaProfileBadgeCollection111111111111111',
    ACHIEVEMENT_NFTS: 'FinovaAchievementNFTCollection11111111111',
  },
  SPECIAL_CARDS: {
    // Mining Boost Cards
    DOUBLE_MINING: {
      EFFECT: 1.0, // +100%
      DURATION: 24 * 60 * 60 * 1000, // 24 hours
      RARITY: 'common',
      PRICE: 50,
      CATEGORY: 'mining_boost',
    },
    TRIPLE_MINING: {
      EFFECT: 2.0, // +200%
      DURATION: 12 * 60 * 60 * 1000, // 12 hours
      RARITY: 'rare',
      PRICE: 150,
      CATEGORY: 'mining_boost',
    },
    MINING_FRENZY: {
      EFFECT: 5.0, // +500%
      DURATION: 4 * 60 * 60 * 1000, // 4 hours
      RARITY: 'epic',
      PRICE: 500,
      CATEGORY: 'mining_boost',
    },
    ETERNAL_MINER: {
      EFFECT: 0.5, // +50%
      DURATION: 30 * 24 * 60 * 60 * 1000, // 30 days
      RARITY: 'legendary',
      PRICE: 2000,
      CATEGORY: 'mining_boost',
    },
    
    // XP Accelerator Cards
    XP_DOUBLE: {
      EFFECT: 1.0, // +100%
      DURATION: 24 * 60 * 60 * 1000,
      RARITY: 'common',
      PRICE: 40,
      CATEGORY: 'xp_accelerator',
    },
    STREAK_SAVER: {
      EFFECT: 'streak_protection',
      DURATION: 7 * 24 * 60 * 60 * 1000, // 7 days
      RARITY: 'uncommon',
      PRICE: 80,
      CATEGORY: 'xp_accelerator',
    },
    LEVEL_RUSH: {
      EFFECT: 500, // Instant +500 XP
      DURATION: 0, // Instant
      RARITY: 'rare',
      PRICE: 120,
      CATEGORY: 'xp_accelerator',
    },
    XP_MAGNET: {
      EFFECT: 3.0, // +300% for viral content
      DURATION: 48 * 60 * 60 * 1000, // 48 hours
      RARITY: 'epic',
      PRICE: 300,
      CATEGORY: 'xp_accelerator',
    },
    
    // Referral Power Cards
    REFERRAL_BOOST: {
      EFFECT: 0.5, // +50%
      DURATION: 7 * 24 * 60 * 60 * 1000,
      RARITY: 'common',
      PRICE: 60,
      CATEGORY: 'referral_power',
    },
    NETWORK_AMPLIFIER: {
      EFFECT: 'tier_boost_2_levels',
      DURATION: 24 * 60 * 60 * 1000,
      RARITY: 'rare',
      PRICE: 200,
      CATEGORY: 'referral_power',
    },
    AMBASSADOR_PASS: {
      EFFECT: 'ambassador_access',
      DURATION: 48 * 60 * 60 * 1000,
      RARITY: 'epic',
      PRICE: 400,
      CATEGORY: 'referral_power',
    },
    NETWORK_KING: {
      EFFECT: 1.0, // +100% from entire network
      DURATION: 12 * 60 * 60 * 1000,
      RARITY: 'legendary',
      PRICE: 1000,
      CATEGORY: 'referral_power',
    },
  },
  RARITIES: {
    COMMON: { MULTIPLIER: 1.0, COLOR: '#9CA3AF' },
    UNCOMMON: { MULTIPLIER: 1.05, COLOR: '#10B981' },
    RARE: { MULTIPLIER: 1.1, COLOR: '#3B82F6' },
    EPIC: { MULTIPLIER: 1.2, COLOR: '#8B5CF6' },
    LEGENDARY: { MULTIPLIER: 1.35, COLOR: '#F59E0B' },
  },
  SYNERGY_BONUSES: {
    SAME_CATEGORY: 0.15, // +15% when same category cards are active
    ALL_CATEGORIES: 0.3, // +30% when all three categories are active
    CARD_COUNT_MULTIPLIER: 0.1, // +10% per active card
  },
} as const;

// ==========================================
// SECURITY & ANTI-BOT SYSTEM
// ==========================================

export const SECURITY_CONFIG = {
  PROOF_OF_HUMANITY: {
    MIN_HUMAN_SCORE: 0.7, // Minimum score to be considered human
    BIOMETRIC_WEIGHT: 0.3,
    BEHAVIORAL_WEIGHT: 0.25,
    SOCIAL_GRAPH_WEIGHT: 0.2,
    DEVICE_WEIGHT: 0.15,
    INTERACTION_WEIGHT: 0.1,
  },
  ANTI_BOT_MEASURES: {
    DIFFICULTY_SCALING: {
      BASE_MULTIPLIER: 1.0,
      EARNINGS_COEFFICIENT: 0.001, // Per 1000 FIN earned
      SUSPICION_MULTIPLIER: 2.0,
      MAX_PENALTY: 0.9, // 90% reduction max
    },
    COOLING_PERIODS: {
      HIGH_ACTIVITY: 2 * 60 * 60 * 1000, // 2 hours after high activity
      SUSPICIOUS_PATTERN: 24 * 60 * 60 * 1000, // 24 hours for suspicious activity
      BOT_DETECTION: 7 * 24 * 60 * 60 * 1000, // 7 days for confirmed bot behavior
    },
    DAILY_CAPS: {
      XP_CAP_MULTIPLIER: 10, // 10x base XP as daily cap
      MINING_CAP_MULTIPLIER: 5, // 5x base mining as daily cap
      RP_CAP_MULTIPLIER: 3, // 3x base RP as daily cap
    },
  },
  KYC_REQUIREMENTS: {
    LEVELS: {
      BASIC: { 
        REQUIREMENTS: ['email_verification', 'phone_verification'],
        MINING_BONUS: 1.0,
        DAILY_LIMIT_MULTIPLIER: 1.0,
      },
      STANDARD: { 
        REQUIREMENTS: ['basic_kyc', 'government_id', 'selfie_verification'],
        MINING_BONUS: 1.2,
        DAILY_LIMIT_MULTIPLIER: 1.5,
      },
      PREMIUM: { 
        REQUIREMENTS: ['standard_kyc', 'address_proof', 'video_verification'],
        MINING_BONUS: 1.5,
        DAILY_LIMIT_MULTIPLIER: 2.0,
      },
    },
  },
  ENCRYPTION: {
    ALGORITHMS: {
      SYMMETRIC: 'AES-256-GCM',
      ASYMMETRIC: 'RSA-4096',
      HASHING: 'SHA-256',
      KEY_DERIVATION: 'PBKDF2',
    },
    KEY_ROTATION_INTERVAL: 30 * 24 * 60 * 60 * 1000, // 30 days
  },
} as const;

// ==========================================
// GUILD SYSTEM
// ==========================================

export const GUILD_CONFIG = {
  STRUCTURE: {
    MIN_MEMBERS: 10,
    MAX_MEMBERS: 50,
    MIN_LEVEL_REQUIREMENT: 11, // Silver level
    LEADERSHIP_POSITIONS: ['guild_master', 'officers', 'members'],
  },
  COMPETITIONS: {
    DAILY_CHALLENGES: {
      DURATION: 24 * 60 * 60 * 1000,
      XP_BONUS: 0.2, // +20% for all members
      PARTICIPATION_REQUIREMENT: 'individual',
    },
    WEEKLY_WARS: {
      DURATION: 7 * 24 * 60 * 60 * 1000,
      REWARD_TYPE: 'guild_treasury',
      PARTICIPATION_REQUIREMENT: 'team_battles',
    },
    MONTHLY_CHAMPIONSHIPS: {
      DURATION: 30 * 24 * 60 * 60 * 1000,
      REWARD_TYPE: 'rare_nft_collections',
      PARTICIPATION_REQUIREMENT: 'cross_guild_tournaments',
    },
    SEASONAL_LEAGUES: {
      DURATION: 90 * 24 * 60 * 60 * 1000,
      REWARD_TYPE: 'massive_fin_prizes',
      PARTICIPATION_REQUIREMENT: 'ranking_system',
    },
  },
  BENEFITS: {
    SHARED_MINING_BONUS: 0.1, // +10% when guild members are active
    COLLECTIVE_XP_MULTIPLIER: 0.05, // +5% XP when guild achieves milestones
    EXCLUSIVE_EVENTS_ACCESS: true,
    GUILD_TREASURY_SHARING: 0.02, // 2% of all member earnings go to guild treasury
  },
} as const;

// ==========================================
// API ENDPOINTS & CONFIGURATION
// ==========================================

export const API_CONFIG = {
  BASE_URLS: {
    PRODUCTION: 'https://api.finova.network/v1',
    STAGING: 'https://api-staging.finova.network/v1',
    DEVELOPMENT: 'https://api-dev.finova.network/v1',
    LOCAL: 'http://localhost:8080/v1',
  },
  ENDPOINTS: {
    AUTH: {
      LOGIN: '/auth/login',
      REGISTER: '/auth/register',
      REFRESH: '/auth/refresh',
      LOGOUT: '/auth/logout',
      KYC_SUBMIT: '/auth/kyc/submit',
      KYC_STATUS: '/auth/kyc/status',
    },
    USER: {
      PROFILE: '/user/profile',
      SETTINGS: '/user/settings',
      STATISTICS: '/user/statistics',
      ACHIEVEMENTS: '/user/achievements',
    },
    MINING: {
      STATUS: '/mining/status',
      START: '/mining/start',
      CLAIM: '/mining/claim',
      HISTORY: '/mining/history',
      BOOSTERS: '/mining/boosters',
    },
    XP: {
      CURRENT: '/xp/current',
      HISTORY: '/xp/history',
      LEADERBOARD: '/xp/leaderboard',
      ACTIVITIES: '/xp/activities',
    },
    RP: {
      NETWORK: '/rp/network',
      STATISTICS: '/rp/statistics',
      REFERRAL_CODE: '/rp/referral-code',
      LEADERBOARD: '/rp/leaderboard',
    },
    STAKING: {
      POSITIONS: '/staking/positions',
      STAKE: '/staking/stake',
      UNSTAKE: '/staking/unstake',
      REWARDS: '/staking/rewards',
    },
    NFT: {
      COLLECTION: '/nft/collection',
      MARKETPLACE: '/nft/marketplace',
      MY_NFTS: '/nft/my-collection',
      USE_CARD: '/nft/use-special-card',
    },
    SOCIAL: {
      PLATFORMS: '/social/platforms',
      CONNECT: '/social/connect',
      DISCONNECT: '/social/disconnect',
      ACTIVITIES: '/social/activities',
    },
    GUILDS: {
      LIST: '/guilds/list',
      JOIN: '/guilds/join',
      LEAVE: '/guilds/leave',
      CREATE: '/guilds/create',
      ACTIVITIES: '/guilds/activities',
    },
  },
  RATE_LIMITS: {
    GENERAL: { REQUESTS: 1000, WINDOW: 60 * 60 * 1000 }, // 1000 per hour
    AUTH: { REQUESTS: 10, WINDOW: 15 * 60 * 1000 }, // 10 per 15 minutes
    MINING: { REQUESTS: 100, WINDOW: 60 * 60 * 1000 }, // 100 per hour
    SOCIAL: { REQUESTS: 500, WINDOW: 60 * 60 * 1000 }, // 500 per hour
  },
  TIMEOUTS: {
    DEFAULT: 10000, // 10 seconds
    FILE_UPLOAD: 30000, // 30 seconds
    BLOCKCHAIN_TRANSACTION: 60000, // 60 seconds
  },
} as const;

// ==========================================
// SOCIAL MEDIA INTEGRATION
// ==========================================

export const SOCIAL_PLATFORMS = {
  TIKTOK: {
    NAME: 'TikTok',
    API_VERSION: 'v1',
    XP_MULTIPLIER: 1.3,
    SUPPORTED_ACTIVITIES: ['post', 'like', 'comment', 'share', 'follow'],
    QUALITY_THRESHOLD: 1000, // 1K views for viral content
  },
  INSTAGRAM: {
    NAME: 'Instagram',
    API_VERSION: 'v18.0',
    XP_MULTIPLIER: 1.2,
    SUPPORTED_ACTIVITIES: ['post', 'story', 'reel', 'like', 'comment', 'follow'],
    QUALITY_THRESHOLD: 500,
  },
  YOUTUBE: {
    NAME: 'YouTube',
    API_VERSION: 'v3',
    XP_MULTIPLIER: 1.4,
    SUPPORTED_ACTIVITIES: ['video', 'short', 'like', 'comment', 'subscribe'],
    QUALITY_THRESHOLD: 1000,
  },
  FACEBOOK: {
    NAME: 'Facebook',
    API_VERSION: 'v18.0',
    XP_MULTIPLIER: 1.1,
    SUPPORTED_ACTIVITIES: ['post', 'like', 'comment', 'share'],
    QUALITY_THRESHOLD: 100,
  },
  TWITTER_X: {
    NAME: 'X (Twitter)',
    API_VERSION: 'v2',
    XP_MULTIPLIER: 1.2,
    SUPPORTED_ACTIVITIES: ['tweet', 'retweet', 'like', 'reply', 'follow'],
    QUALITY_THRESHOLD: 1000,
  },
} as const;

// ==========================================
// E-WALLET INTEGRATION (INDONESIA)
// ==========================================

export const E_WALLET_CONFIG = {
  PROVIDERS: {
    OVO: {
      NAME: 'OVO',
      CURRENCY: 'IDR',
      MIN_TRANSACTION: 10000, // 10,000 IDR
      MAX_TRANSACTION: 20000000, // 20,000,000 IDR
      FEE_PERCENTAGE: 0.005, // 0.5%
      SUPPORTED_OPERATIONS: ['deposit', 'withdraw', 'transfer'],
    },
    GOPAY: {
      NAME: 'GoPay',
      CURRENCY: 'IDR',
      MIN_TRANSACTION: 1000,
      MAX_TRANSACTION: 20000000,
      FEE_PERCENTAGE: 0.007, // 0.7%
      SUPPORTED_OPERATIONS: ['deposit', 'withdraw', 'transfer'],
    },
    DANA: {
      NAME: 'DANA',
      CURRENCY: 'IDR',
      MIN_TRANSACTION: 1000,
      MAX_TRANSACTION: 20000000,
      FEE_PERCENTAGE: 0.006, // 0.6%
      SUPPORTED_OPERATIONS: ['deposit', 'withdraw', 'transfer'],
    },
    SHOPEEPAY: {
      NAME: 'ShopeePay',
      CURRENCY: 'IDR',
      MIN_TRANSACTION: 10000,
      MAX_TRANSACTION: 10000000,
      FEE_PERCENTAGE: 0.008, // 0.8%
      SUPPORTED_OPERATIONS: ['deposit', 'withdraw'],
    },
  },
  EXCHANGE_RATES: {
    FIN_TO_IDR_RATE: 'dynamic', // Fetched from oracle
    USD_FIN_TO_IDR_RATE: 'pegged', // Pegged to USD-IDR rate
    UPDATE_INTERVAL: 5 * 60 * 1000, // 5 minutes
  },
} as const;

// ==========================================
// NOTIFICATION SYSTEM
// ==========================================

export const NOTIFICATION_CONFIG = {
  TYPES: {
    MINING_REWARD: {
      TITLE: 'Mining Reward Earned',
      PRIORITY: 'normal',
      SOUND: 'mining_success',
      VIBRATION: [100, 50, 100],
    },
    XP_MILESTONE: {
      TITLE: 'XP Milestone Achieved',
      PRIORITY: 'high',
      SOUND: 'achievement',
      VIBRATION: [200, 100, 200],
    },
    REFERRAL_SUCCESS: {
      TITLE: 'New Referral Joined',
      PRIORITY: 'high',
      SOUND: 'referral_success',
      VIBRATION: [150, 75, 150],
    },
    LEVEL_UP: {
      TITLE: 'Level Up!',
      PRIORITY: 'high',
      SOUND: 'level_up',
      VIBRATION: [300, 150, 300, 150, 300],
    },
    SPECIAL_CARD_EXPIRY: {
      TITLE: 'Special Card Expiring Soon',
      PRIORITY: 'normal',
      SOUND: 'warning',
      VIBRATION: [100],
    },
    GUILD_EVENT: {
      TITLE: 'Guild Event Started',
      PRIORITY: 'normal',
      SOUND: 'guild_event',
      VIBRATION: [100, 50, 100, 50, 100],
    },
    SECURITY_ALERT: {
      TITLE: 'Security Alert',
      PRIORITY: 'urgent',
      SOUND: 'security_alert',
      VIBRATION: [500, 200, 500, 200, 500],
    },
  },
  CHANNELS: {
    REWARDS: 'finova_rewards',
    ACHIEVEMENTS: 'finova_achievements',
    SOCIAL: 'finova_social',
    SECURITY: 'finova_security',
    GUILDS: 'finova_guilds',
  },
  DELIVERY_METHODS: ['push', 'in_app', 'email', 'sms'],
} as const;

// ==========================================
// AI QUALITY ASSESSMENT
// ==========================================

export const AI_CONFIG = {
  QUALITY_FACTORS: {
    ORIGINALITY: {
      WEIGHT: 0.3,
      THRESHOLD_EXCELLENT: 0.9,
      THRESHOLD_GOOD: 0.7,
      THRESHOLD_POOR: 0.4,
    },
    ENGAGEMENT_POTENTIAL: {
      WEIGHT: 0.25,
      FACTORS: ['trending_topics', 'hashtag_relevance', 'timing', 'audience_match'],
    },
    PLATFORM_RELEVANCE: {
      WEIGHT: 0.2,
      PLATFORM_SPECIFIC_SCORING: true,
    },
    BRAND_SAFETY: {
      WEIGHT: 0.15,
      RESTRICTED_CONTENT: ['adult', 'violence', 'hate_speech', 'spam'],
    },
    HUMAN_GENERATED: {
      WEIGHT: 0.1,
      AI_DETECTION_THRESHOLD: 0.8,
    },
  },
  MODELS: {
    CONTENT_CLASSIFIER: 'finova-content-v2.1',
    IMAGE_ANALYZER: 'finova-vision-v1.3',
    VIDEO_ANALYZER: 'finova-video-v1.1',
    TEXT_PROCESSOR: 'finova-nlp-v3.0',
  },
  PROCESSING_LIMITS: {
    MAX_TEXT_LENGTH: 10000, // characters
    MAX_IMAGE_SIZE: 50 * 1024 * 1024, // 50MB
    MAX_VIDEO_DURATION: 300, // 5 minutes
    BATCH_SIZE: 100,
  },
} as const;

// ==========================================
// PERFORMANCE & CACHING
// ==========================================

export const PERFORMANCE_CONFIG = {
  CACHE_STRATEGIES: {
    USER_PROFILE: {
      TTL: 5 * 60 * 1000, // 5 minutes
      STRATEGY: 'cache_first',
    },
    MINING_STATUS: {
      TTL: 30 * 1000, // 30 seconds
      STRATEGY: 'network_first',
    },
    XP_LEADERBOARD: {
      TTL: 60 * 1000, // 1 minute
      STRATEGY: 'cache_first',
    },
    NFT_COLLECTION: {
      TTL: 10 * 60 * 1000, // 10 minutes
      STRATEGY: 'cache_first',
    },
    EXCHANGE_RATES: {
      TTL: 5 * 60 * 1000, // 5 minutes
      STRATEGY: 'network_first',
    },
  },
  RETRY_POLICIES: {
    DEFAULT: {
      MAX_ATTEMPTS: 3,
      BACKOFF_MULTIPLIER: 2,
      INITIAL_DELAY: 1000,
    },
    BLOCKCHAIN: {
      MAX_ATTEMPTS: 5,
      BACKOFF_MULTIPLIER: 1.5,
      INITIAL_DELAY: 2000,
    },
    CRITICAL: {
      MAX_ATTEMPTS: 10,
      BACKOFF_MULTIPLIER: 1.2,
      INITIAL_DELAY: 500,
    },
  },
  BATCH_PROCESSING: {
    XP_CALCULATIONS: {
      BATCH_SIZE: 50,
      PROCESSING_INTERVAL: 5000, // 5 seconds
    },
    MINING_REWARDS: {
      BATCH_SIZE: 100,
      PROCESSING_INTERVAL: 60000, // 1 minute
    },
    NOTIFICATIONS: {
      BATCH_SIZE: 500,
      PROCESSING_INTERVAL: 10000, // 10 seconds
    },
  },
} as const;

// ==========================================
// ERROR CODES & MESSAGES
// ==========================================

export const ERROR_CODES = {
  // Authentication Errors (1000-1999)
  AUTH_INVALID_CREDENTIALS: { CODE: 1001, MESSAGE: 'Invalid credentials provided' },
  AUTH_TOKEN_EXPIRED: { CODE: 1002, MESSAGE: 'Authentication token has expired' },
  AUTH_INSUFFICIENT_PERMISSIONS: { CODE: 1003, MESSAGE: 'Insufficient permissions for this action' },
  AUTH_ACCOUNT_SUSPENDED: { CODE: 1004, MESSAGE: 'Account has been suspended' },
  AUTH_KYC_REQUIRED: { CODE: 1005, MESSAGE: 'KYC verification required' },
  AUTH_KYC_PENDING: { CODE: 1006, MESSAGE: 'KYC verification is pending' },
  AUTH_KYC_REJECTED: { CODE: 1007, MESSAGE: 'KYC verification was rejected' },

  // Mining Errors (2000-2999)
  MINING_NOT_STARTED: { CODE: 2001, MESSAGE: 'Mining session has not started' },
  MINING_ALREADY_ACTIVE: { CODE: 2002, MESSAGE: 'Mining session is already active' },
  MINING_DAILY_LIMIT_REACHED: { CODE: 2003, MESSAGE: 'Daily mining limit reached' },
  MINING_INSUFFICIENT_ENERGY: { CODE: 2004, MESSAGE: 'Insufficient energy to continue mining' },
  MINING_BOT_DETECTED: { CODE: 2005, MESSAGE: 'Suspicious bot activity detected' },
  MINING_COOLDOWN_ACTIVE: { CODE: 2006, MESSAGE: 'Mining cooldown period is active' },

  // XP System Errors (3000-3999)
  XP_DAILY_LIMIT_REACHED: { CODE: 3001, MESSAGE: 'Daily XP limit reached for this activity' },
  XP_INVALID_ACTIVITY: { CODE: 3002, MESSAGE: 'Invalid XP activity type' },
  XP_QUALITY_TOO_LOW: { CODE: 3003, MESSAGE: 'Content quality score too low for XP reward' },
  XP_PLATFORM_NOT_CONNECTED: { CODE: 3004, MESSAGE: 'Social media platform not connected' },

  // Referral Errors (4000-4999)
  RP_INVALID_CODE: { CODE: 4001, MESSAGE: 'Invalid referral code' },
  RP_SELF_REFERRAL: { CODE: 4002, MESSAGE: 'Cannot refer yourself' },
  RP_ALREADY_REFERRED: { CODE: 4003, MESSAGE: 'User already has a referrer' },
  RP_NETWORK_LIMIT_REACHED: { CODE: 4004, MESSAGE: 'Referral network size limit reached' },
  RP_INACTIVE_NETWORK: { CODE: 4005, MESSAGE: 'Referral network activity too low' },

  // Staking Errors (5000-5999)
  STAKING_INSUFFICIENT_BALANCE: { CODE: 5001, MESSAGE: 'Insufficient balance for staking' },
  STAKING_MINIMUM_NOT_MET: { CODE: 5002, MESSAGE: 'Minimum staking amount not met' },
  STAKING_UNBONDING_PERIOD: { CODE: 5003, MESSAGE: 'Tokens are in unbonding period' },
  STAKING_NO_ACTIVE_STAKE: { CODE: 5004, MESSAGE: 'No active staking position found' },

  // NFT Errors (6000-6999)
  NFT_NOT_FOUND: { CODE: 6001, MESSAGE: 'NFT not found' },
  NFT_NOT_OWNED: { CODE: 6002, MESSAGE: 'You do not own this NFT' },
  NFT_ALREADY_USED: { CODE: 6003, MESSAGE: 'This special card has already been used' },
  NFT_EXPIRED: { CODE: 6004, MESSAGE: 'This special card has expired' },
  NFT_INSUFFICIENT_FUNDS: { CODE: 6005, MESSAGE: 'Insufficient funds to purchase NFT' },

  // Blockchain Errors (7000-7999)
  BLOCKCHAIN_TRANSACTION_FAILED: { CODE: 7001, MESSAGE: 'Blockchain transaction failed' },
  BLOCKCHAIN_INSUFFICIENT_SOL: { CODE: 7002, MESSAGE: 'Insufficient SOL for transaction fees' },
  BLOCKCHAIN_NETWORK_CONGESTION: { CODE: 7003, MESSAGE: 'Network congestion, please retry' },
  BLOCKCHAIN_INVALID_SIGNATURE: { CODE: 7004, MESSAGE: 'Invalid transaction signature' },

  // Guild Errors (8000-8999)
  GUILD_NOT_FOUND: { CODE: 8001, MESSAGE: 'Guild not found' },
  GUILD_ALREADY_MEMBER: { CODE: 8002, MESSAGE: 'Already a member of this guild' },
  GUILD_FULL: { CODE: 8003, MESSAGE: 'Guild has reached maximum member capacity' },
  GUILD_INSUFFICIENT_LEVEL: { CODE: 8004, MESSAGE: 'Insufficient level to join this guild' },
  GUILD_NOT_MEMBER: { CODE: 8005, MESSAGE: 'You are not a member of this guild' },

  // System Errors (9000-9999)
  SYSTEM_MAINTENANCE: { CODE: 9001, MESSAGE: 'System is under maintenance' },
  SYSTEM_RATE_LIMIT: { CODE: 9002, MESSAGE: 'Rate limit exceeded, please slow down' },
  SYSTEM_INTERNAL_ERROR: { CODE: 9003, MESSAGE: 'Internal system error occurred' },
  SYSTEM_FEATURE_DISABLED: { CODE: 9004, MESSAGE: 'This feature is currently disabled' },
} as const;

// ==========================================
// FEATURE FLAGS
// ==========================================

export const FEATURE_FLAGS = {
  // Core Features
  MINING_ENABLED: true,
  XP_SYSTEM_ENABLED: true,
  RP_SYSTEM_ENABLED: true,
  STAKING_ENABLED: true,
  NFT_MARKETPLACE_ENABLED: true,
  GUILD_SYSTEM_ENABLED: true,

  // Social Platform Integrations
  TIKTOK_INTEGRATION_ENABLED: true,
  INSTAGRAM_INTEGRATION_ENABLED: true,
  YOUTUBE_INTEGRATION_ENABLED: false, // Coming soon
  FACEBOOK_INTEGRATION_ENABLED: true,
  TWITTER_X_INTEGRATION_ENABLED: false, // Coming soon

  // Payment & E-wallet
  E_WALLET_INTEGRATION_ENABLED: true,
  CRYPTO_PAYMENTS_ENABLED: true,
  FIAT_ONRAMP_ENABLED: false, // Coming soon

  // Advanced Features
  AI_QUALITY_ASSESSMENT_ENABLED: true,
  ADVANCED_ANTI_BOT_ENABLED: true,
  CROSS_CHAIN_BRIDGE_ENABLED: false, // Coming soon
  DAO_GOVERNANCE_ENABLED: false, // Coming soon

  // Beta Features
  BETA_FEATURES_ENABLED: false,
  ADVANCED_ANALYTICS_ENABLED: false,
  ENTERPRISE_API_ENABLED: false,
} as const;

// ==========================================
// DEVELOPMENT & DEBUG CONSTANTS
// ==========================================

export const DEBUG_CONFIG = {
  LOGGING_LEVELS: {
    ERROR: 0,
    WARN: 1,
    INFO: 2,
    DEBUG: 3,
    VERBOSE: 4,
  },
  MOCK_DATA_ENABLED: __DEV__ ? true : false,
  PERFORMANCE_MONITORING: true,
  CRASH_REPORTING: true,
  ANALYTICS_ENABLED: true,
  DEBUG_NETWORK_CALLS: __DEV__ ? true : false,
} as const;

// ==========================================
// EXPORTED UTILITY FUNCTIONS
// ==========================================

/**
 * Calculate mining rate based on user data and current phase
 */
export const calculateMiningRate = (
  userCount: number,
  userHoldings: number,
  referralCount: number,
  isKYCVerified: boolean
): number => {
  // Determine current phase
  let phase = MINING_CONFIG.PHASES.FINIZEN;
  if (userCount > 10_000_000) phase = MINING_CONFIG.PHASES.STABILITY;
  else if (userCount > 1_000_000) phase = MINING_CONFIG.PHASES.MATURITY;
  else if (userCount > 100_000) phase = MINING_CONFIG.PHASES.GROWTH;

  const baseRate = phase.BASE_RATE;
  const finenBonus = Math.max(1.0, phase.FINIZEN_BONUS - (userCount / 1_000_000));
  const referralBonus = 1 + (Math.min(referralCount, 35) * MINING_CONFIG.BONUSES.REFERRAL_MULTIPLIER);
  const securityBonus = isKYCVerified ? MINING_CONFIG.BONUSES.KYC_SECURITY_BONUS : MINING_CONFIG.BONUSES.NON_KYC_PENALTY;
  const regressionFactor = Math.exp(-MINING_CONFIG.BONUSES.REGRESSION_COEFFICIENT * userHoldings);

  return baseRate * finenBonus * referralBonus * securityBonus * regressionFactor;
};

/**
 * Calculate XP gain for an activity
 */
export const calculateXPGain = (
  activityType: keyof typeof XP_CONFIG.ACTIVITIES,
  platform: keyof typeof XP_CONFIG.PLATFORM_MULTIPLIERS,
  qualityScore: number,
  streakDays: number,
  userLevel: number
): number => {
  const activity = XP_CONFIG.ACTIVITIES[activityType];
  if (!activity) return 0;

  const baseXP = activity.BASE_XP;
  const platformMultiplier = XP_CONFIG.PLATFORM_MULTIPLIERS[platform] || XP_CONFIG.PLATFORM_MULTIPLIERS.DEFAULT;
  const clampedQuality = Math.max(activity.QUALITY_RANGE[0], Math.min(activity.QUALITY_RANGE[1], qualityScore));
  const streakBonus = 1 + Math.min(streakDays * 0.1, 2.0); // Max 3x streak bonus
  const levelProgression = Math.exp(-XP_CONFIG.PROGRESSION_COEFFICIENT * userLevel);

  return Math.floor(baseXP * platformMultiplier * clampedQuality * streakBonus * levelProgression);
};

/**
 * Get user tier based on XP level
 */
export const getUserTier = (level: number): keyof typeof XP_CONFIG.LEVEL_SYSTEM => {
  if (level >= 101) return 'MYTHIC';
  if (level >= 76) return 'DIAMOND';
  if (level >= 51) return 'PLATINUM';
  if (level >= 26) return 'GOLD';
  if (level >= 11) return 'SILVER';
  return 'BRONZE';
};

/**
 * Get RP tier based on RP amount
 */
export const getRPTier = (rpAmount: number): keyof typeof RP_CONFIG.TIERS => {
  if (rpAmount >= 50000) return 'AMBASSADOR';
  if (rpAmount >= 15000) return 'LEADER';
  if (rpAmount >= 5000) return 'INFLUENCER';
  if (rpAmount >= 1000) return 'CONNECTOR';
  return 'EXPLORER';
};

/**
 * Get staking tier based on staked amount
 */
export const getStakingTier = (stakedAmount: number): number => {
  return STAKING_CONFIG.TIERS.findIndex(tier => 
    stakedAmount >= tier.MIN_STAKE && stakedAmount <= tier.MAX_STAKE
  );
};

/**
 * Check if feature is enabled
 */
export const isFeatureEnabled = (featureName: keyof typeof FEATURE_FLAGS): boolean => {
  return FEATURE_FLAGS[featureName];
};

/**
 * Get error message by code
 */
export const getErrorMessage = (errorCode: number): string => {
  const error = Object.values(ERROR_CODES).find(err => err.CODE === errorCode);
  return error?.MESSAGE || 'Unknown error occurred';
};
