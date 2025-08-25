// client/typescript/src/constants.ts

import { PublicKey } from '@solana/web3.js';

/**
 * Finova Network Program IDs and Constants
 */

// Program IDs (these would be the actual deployed program addresses)
export const PROGRAM_IDS = {
  FINOVA_CORE: new PublicKey('FiNoVaCoreProgram11111111111111111111111111'),
  FINOVA_TOKEN: new PublicKey('FiNoVaTokenProgram1111111111111111111111111'),
  FINOVA_NFT: new PublicKey('FiNoVaNFTProgram111111111111111111111111111'),
  FINOVA_DEFI: new PublicKey('FiNoVaDeFiProgram11111111111111111111111111'),
  FINOVA_ORACLE: new PublicKey('FiNoVaOracleProgram111111111111111111111111'),
  FINOVA_BRIDGE: new PublicKey('FiNoVaBridgeProgram111111111111111111111111'),
} as const;

// Token Mints
export const TOKEN_MINTS = {
  FIN: new PublicKey('FiNoVaFINToken1111111111111111111111111111111'),
  sFIN: new PublicKey('FiNoVasFINToken111111111111111111111111111111'),
  USDfin: new PublicKey('FiNoVaUSDfinToken111111111111111111111111111'),
  sUSDfin: new PublicKey('FiNoVasUSDfinToken11111111111111111111111111'),
} as const;

// Network Configuration
export const NETWORK_CONFIG = {
  MAINNET: {
    RPC_URL: 'https://api.mainnet-beta.solana.com',
    CLUSTER: 'mainnet-beta',
  },
  TESTNET: {
    RPC_URL: 'https://api.testnet.solana.com',
    CLUSTER: 'testnet',
  },
  DEVNET: {
    RPC_URL: 'https://api.devnet.solana.com',
    CLUSTER: 'devnet',
  },
  LOCALNET: {
    RPC_URL: 'http://127.0.0.1:8899',
    CLUSTER: 'localnet',
  },
} as const;

// Mining Constants
export const MINING_CONSTANTS = {
  BASE_MINING_RATE: 0.05, // $FIN per hour
  PHASES: {
    FINIZEN: {
      USER_THRESHOLD: 100_000,
      BASE_RATE: 0.1,
      FINIZEN_BONUS: 2.0,
      MAX_DAILY: 4.8,
    },
    GROWTH: {
      USER_THRESHOLD: 1_000_000,
      BASE_RATE: 0.05,
      FINIZEN_BONUS: 1.5,
      MAX_DAILY: 1.8,
    },
    MATURITY: {
      USER_THRESHOLD: 10_000_000,
      BASE_RATE: 0.025,
      FINIZEN_BONUS: 1.2,
      MAX_DAILY: 0.72,
    },
    STABILITY: {
      USER_THRESHOLD: Infinity,
      BASE_RATE: 0.01,
      FINIZEN_BONUS: 1.0,
      MAX_DAILY: 0.24,
    },
  },
  REGRESSION_FACTOR: 0.001,
  SECURITY_BONUS: {
    KYC_VERIFIED: 1.2,
    NOT_VERIFIED: 0.8,
  },
  REFERRAL_BONUS_RATE: 0.1,
} as const;

// XP Constants
export const XP_CONSTANTS = {
  ACTIVITIES: {
    ORIGINAL_POST: 50,
    PHOTO_POST: 75,
    VIDEO_POST: 150,
    STORY_POST: 25,
    MEANINGFUL_COMMENT: 25,
    LIKE_REACT: 5,
    SHARE_REPOST: 15,
    FOLLOW_SUBSCRIBE: 20,
    DAILY_LOGIN: 10,
    DAILY_QUEST: 100,
    MILESTONE: 500,
    VIRAL_CONTENT: 1000,
  },
  PLATFORM_MULTIPLIERS: {
    TIKTOK: 1.3,
    INSTAGRAM: 1.2,
    YOUTUBE: 1.4,
    X: 1.2,
    FACEBOOK: 1.1,
    DEFAULT: 1.0,
  },
  LEVEL_TIERS: {
    BRONZE: { min: 1, max: 10, mining_multiplier: [1.0, 1.2] },
    SILVER: { min: 11, max: 25, mining_multiplier: [1.3, 1.8] },
    GOLD: { min: 26, max: 50, mining_multiplier: [1.9, 2.5] },
    PLATINUM: { min: 51, max: 75, mining_multiplier: [2.6, 3.2] },
    DIAMOND: { min: 76, max: 100, mining_multiplier: [3.3, 4.0] },
    MYTHIC: { min: 101, max: Infinity, mining_multiplier: [4.1, 5.0] },
  },
} as const;

// RP Constants
export const RP_CONSTANTS = {
  DIRECT_REFERRAL_POINTS: {
    SIGNUP: 50,
    KYC_COMPLETE: 100,
    FIRST_MINING: 25,
  },
  NETWORK_BONUSES: {
    TIER_10: { rp: 500, multiplier: 0.5 },
    TIER_25: { rp: 1500, multiplier: 1.0 },
    TIER_50: { rp: 5000, multiplier: 1.5 },
    TIER_100: { rp: 15000, multiplier: 2.0 },
  },
  TIERS: {
    EXPLORER: { min: 0, max: 999, bonus: 0, referral_bonus: 0.1 },
    CONNECTOR: { min: 1000, max: 4999, bonus: 0.2, referral_bonus: 0.15 },
    INFLUENCER: { min: 5000, max: 14999, bonus: 0.5, referral_bonus: 0.2 },
    LEADER: { min: 15000, max: 49999, bonus: 1.0, referral_bonus: 0.25 },
    AMBASSADOR: { min: 50000, max: Infinity, bonus: 2.0, referral_bonus: 0.3 },
  },
} as const;

// Account Seed Constants
export const SEEDS = {
  NETWORK_STATE: 'network_state',
  USER_STATE: 'user_state',
  XP_STATE: 'xp_state',
  REFERRAL_STATE: 'referral_state',
  STAKING_STATE: 'staking_state',
  ACTIVE_EFFECTS_STATE: 'active_effects_state',
  GUILD_STATE: 'guild_state',
  PROPOSAL_STATE: 'proposal_state',
  VOTE_RECORD: 'vote_record',
  NFT_COLLECTION: 'nft_collection',
  NFT_METADATA: 'nft_metadata',
  MARKETPLACE_STATE: 'marketplace_state',
} as const;

// Transaction Constants
export const TRANSACTION_CONSTANTS = {
  DEFAULT_COMMITMENT: 'confirmed' as const,
  DEFAULT_TIMEOUT: 30000, // 30 seconds
  MAX_RETRIES: 3,
  RETRY_DELAY: 1000, // 1 second
} as const;

// Error Codes (matching smart contract error codes)
export const ERROR_CODES = {
  UNAUTHORIZED: 6000,
  INVALID_CALCULATION: 6001,
  ALREADY_INITIALIZED: 6002,
  NOT_INITIALIZED: 6003,
  INSUFFICIENT_BALANCE: 6004,
  INVALID_AMOUNT: 6005,
  MINING_NOT_ACTIVE: 6006,
  COOLDOWN_ACTIVE: 6007,
  INVALID_REFERRER: 6008,
  MAX_REFERRALS_REACHED: 6009,
  INVALID_CARD_TYPE: 6010,
  CARD_ALREADY_USED: 6011,
  GUILD_FULL: 6012,
  NOT_GUILD_MEMBER: 6013,
  INVALID_PROPOSAL: 6014,
  VOTING_ENDED: 6015,
  ALREADY_VOTED: 6016,
} as const;
