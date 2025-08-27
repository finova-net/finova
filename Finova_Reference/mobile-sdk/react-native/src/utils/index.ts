// mobile-sdk/react-native/src/utils/index.ts
export * from './calculations';
export * from './validation';
export * from './formatting';
export * from './constants';
export * from './crypto';
export * from './storage';
export * from './network';
export * from './biometric';

// mobile-sdk/react-native/src/utils/constants.ts
export const FINOVA_CONSTANTS = {
  // Network Configuration
  SOLANA_MAINNET_RPC: 'https://api.mainnet-beta.solana.com',
  SOLANA_DEVNET_RPC: 'https://api.devnet.solana.com',
  FINOVA_API_BASE: 'https://api.finova.network/v1',
  WS_ENDPOINT: 'wss://ws.finova.network',

  // Mining Constants
  MINING: {
    BASE_RATE_PHASE_1: 0.1,
    BASE_RATE_PHASE_2: 0.05,
    BASE_RATE_PHASE_3: 0.025,
    BASE_RATE_PHASE_4: 0.01,
    REGRESSION_FACTOR: 0.001,
    MAX_DAILY_MINING: 24,
    PIONEER_BONUS_MAX: 2.0,
    REFERRAL_BONUS_RATE: 0.1,
    SECURITY_BONUS_KYC: 1.2,
    SECURITY_BONUS_NO_KYC: 0.8
  },

  // XP System Constants
  XP: {
    ACTIVITIES: {
      ORIGINAL_POST: 50,
      PHOTO_POST: 75,
      VIDEO_CONTENT: 150,
      STORY_STATUS: 25,
      MEANINGFUL_COMMENT: 25,
      LIKE_REACT: 5,
      SHARE_REPOST: 15,
      FOLLOW_SUBSCRIBE: 20,
      DAILY_LOGIN: 10,
      DAILY_QUEST: 100,
      MILESTONE: 500,
      VIRAL_CONTENT: 1000
    },
    PLATFORM_MULTIPLIERS: {
      TIKTOK: 1.3,
      INSTAGRAM: 1.2,
      YOUTUBE: 1.4,
      FACEBOOK: 1.1,
      TWITTER: 1.2,
      DEFAULT: 1.0
    },
    LEVEL_PROGRESSION_FACTOR: 0.01,
    MAX_QUALITY_SCORE: 2.0,
    MIN_QUALITY_SCORE: 0.5
  },

  // RP System Constants
  RP: {
    REGISTRATION_BONUS: 50,
    KYC_COMPLETION: 100,
    FIRST_MINING: 25,
    NETWORK_REGRESSION: 0.0001,
    TIER_MULTIPLIERS: {
      EXPLORER: 1.0,
      CONNECTOR: 1.2,
      INFLUENCER: 1.5,
      LEADER: 2.0,
      AMBASSADOR: 3.0
    }
  },

  // Staking Constants
  STAKING: {
    MIN_STAKE: 100,
    TIER_THRESHOLDS: [100, 500, 1000, 5000, 10000],
    BASE_APY: [8, 10, 12, 14, 15],
    MINING_BOOSTS: [0.2, 0.35, 0.5, 0.75, 1.0],
    LOYALTY_BONUS_RATE: 0.05
  },

  // Security & Validation
  SECURITY: {
    JWT_EXPIRY: 3600000, // 1 hour
    REFRESH_TOKEN_EXPIRY: 2592000000, // 30 days
    MAX_LOGIN_ATTEMPTS: 5,
    LOCKOUT_DURATION: 900000, // 15 minutes
    BIOMETRIC_TIMEOUT: 300000, // 5 minutes
    API_RATE_LIMIT: 100 // requests per minute
  },

  // NFT & Cards
  NFT: {
    CARD_EFFECTS: {
      DOUBLE_MINING: 2.0,
      TRIPLE_MINING: 3.0,
      MINING_FRENZY: 6.0,
      ETERNAL_MINER: 1.5,
      XP_DOUBLE: 2.0,
      LEVEL_RUSH: 500,
      XP_MAGNET: 4.0
    },
    RARITY_MULTIPLIERS: {
      COMMON: 1.0,
      UNCOMMON: 1.05,
      RARE: 1.1,
      EPIC: 1.2,
      LEGENDARY: 1.35
    }
  }
} as const;

export const ERROR_CODES = {
  // Authentication Errors
  AUTH_INVALID_CREDENTIALS: 'AUTH_001',
  AUTH_TOKEN_EXPIRED: 'AUTH_002',
  AUTH_BIOMETRIC_FAILED: 'AUTH_003',
  AUTH_KYC_REQUIRED: 'AUTH_004',
  AUTH_ACCOUNT_LOCKED: 'AUTH_005',

  // Mining Errors
  MINING_RATE_LIMIT: 'MINING_001',
  MINING_INSUFFICIENT_BALANCE: 'MINING_002',
  MINING_BOT_DETECTED: 'MINING_003',
  MINING_NETWORK_ERROR: 'MINING_004',

  // Validation Errors
  VALIDATION_INVALID_INPUT: 'VAL_001',
  VALIDATION_MISSING_FIELD: 'VAL_002',
  VALIDATION_FORMAT_ERROR: 'VAL_003',
  VALIDATION_LENGTH_ERROR: 'VAL_004',

  // Network Errors
  NETWORK_TIMEOUT: 'NET_001',
  NETWORK_CONNECTION_FAILED: 'NET_002',
  NETWORK_SERVER_ERROR: 'NET_003',
  NETWORK_RATE_LIMITED: 'NET_004'
} as const;

// mobile-sdk/react-native/src/utils/calculations.ts
import { FINOVA_CONSTANTS } from './constants';

export interface MiningCalculationInput {
  totalUsers: number;
  userHoldings: number;
  activeReferrals: number;
  isKYCVerified: boolean;
  xpLevel: number;
  rpTier: number;
  stakingAmount?: number;
  activeCards?: string[];
}

export interface XPCalculationInput {
  activityType: string;
  platform: string;
  qualityScore: number;
  streakDays: number;
  currentLevel: number;
}

export interface RPCalculationInput {
  directReferrals: number;
  l2Network: number;
  l3Network: number;
  networkQualityScore: number;
  totalNetworkSize: number;
}

export class FinovaCalculations {
  
  // Mining Rate Calculations
  static calculateMiningRate(input: MiningCalculationInput): number {
    const { totalUsers, userHoldings, activeReferrals, isKYCVerified, xpLevel, rpTier } = input;
    
    const baseRate = this.getCurrentPhaseRate(totalUsers);
    const pioneerBonus = this.calculatePioneerBonus(totalUsers);
    const referralBonus = 1 + (activeReferrals * FINOVA_CONSTANTS.MINING.REFERRAL_BONUS_RATE);
    const securityBonus = isKYCVerified 
      ? FINOVA_CONSTANTS.MINING.SECURITY_BONUS_KYC 
      : FINOVA_CONSTANTS.MINING.SECURITY_BONUS_NO_KYC;
    const regressionFactor = Math.exp(-FINOVA_CONSTANTS.MINING.REGRESSION_FACTOR * userHoldings);
    const xpBonus = this.getXPMiningBonus(xpLevel);
    const rpBonus = this.getRPMiningBonus(rpTier);
    
    return baseRate * pioneerBonus * referralBonus * securityBonus * regressionFactor * xpBonus * rpBonus;
  }

  private static getCurrentPhaseRate(totalUsers: number): number {
    if (totalUsers < 100000) return FINOVA_CONSTANTS.MINING.BASE_RATE_PHASE_1;
    if (totalUsers < 1000000) return FINOVA_CONSTANTS.MINING.BASE_RATE_PHASE_2;
    if (totalUsers < 10000000) return FINOVA_CONSTANTS.MINING.BASE_RATE_PHASE_3;
    return FINOVA_CONSTANTS.MINING.BASE_RATE_PHASE_4;
  }

  private static calculatePioneerBonus(totalUsers: number): number {
    return Math.max(1.0, FINOVA_CONSTANTS.MINING.PIONEER_BONUS_MAX - (totalUsers / 1000000));
  }

  private static getXPMiningBonus(level: number): number {
    if (level <= 10) return 1.0 + (level * 0.02);
    if (level <= 25) return 1.2 + ((level - 10) * 0.04);
    if (level <= 50) return 1.8 + ((level - 25) * 0.028);
    if (level <= 75) return 2.5 + ((level - 50) * 0.028);
    if (level <= 100) return 3.2 + ((level - 75) * 0.032);
    return 4.0 + Math.min(1.0, (level - 100) * 0.01);
  }

  private static getRPMiningBonus(tier: number): number {
    const tierMultipliers = Object.values(FINOVA_CONSTANTS.RP.TIER_MULTIPLIERS);
    return tierMultipliers[Math.min(tier, tierMultipliers.length - 1)] || 1.0;
  }

  // XP Calculations
  static calculateXPGain(input: XPCalculationInput): number {
    const { activityType, platform, qualityScore, streakDays, currentLevel } = input;
    
    const baseXP = FINOVA_CONSTANTS.XP.ACTIVITIES[activityType as keyof typeof FINOVA_CONSTANTS.XP.ACTIVITIES] || 0;
    const platformMultiplier = FINOVA_CONSTANTS.XP.PLATFORM_MULTIPLIERS[platform as keyof typeof FINOVA_CONSTANTS.XP.PLATFORM_MULTIPLIERS] || FINOVA_CONSTANTS.XP.PLATFORM_MULTIPLIERS.DEFAULT;
    const qualityMultiplier = Math.max(FINOVA_CONSTANTS.XP.MIN_QUALITY_SCORE, Math.min(FINOVA_CONSTANTS.XP.MAX_QUALITY_SCORE, qualityScore));
    const streakBonus = Math.min(3.0, 1 + (streakDays * 0.05));
    const levelProgression = Math.exp(-FINOVA_CONSTANTS.XP.LEVEL_PROGRESSION_FACTOR * currentLevel);
    
    return Math.round(baseXP * platformMultiplier * qualityMultiplier * streakBonus * levelProgression);
  }

  static calculateLevelFromXP(totalXP: number): number {
    if (totalXP < 1000) return Math.floor(totalXP / 100) + 1;
    if (totalXP < 5000) return 10 + Math.floor((totalXP - 1000) / 250);
    if (totalXP < 20000) return 25 + Math.floor((totalXP - 5000) / 600);
    if (totalXP < 50000) return 50 + Math.floor((totalXP - 20000) / 1200);
    if (totalXP < 100000) return 75 + Math.floor((totalXP - 50000) / 2000);
    return 100 + Math.floor((totalXP - 100000) / 5000);
  }

  // RP Calculations
  static calculateRPValue(input: RPCalculationInput): number {
    const { directReferrals, l2Network, l3Network, networkQualityScore, totalNetworkSize } = input;
    
    const directRP = directReferrals * 100 * networkQualityScore;
    const indirectRP = (l2Network * 50 * 0.3) + (l3Network * 25 * 0.1);
    const qualityBonus = networkQualityScore * 10;
    const regressionFactor = Math.exp(-FINOVA_CONSTANTS.RP.NETWORK_REGRESSION * totalNetworkSize * networkQualityScore);
    
    return Math.round((directRP + indirectRP + qualityBonus) * regressionFactor);
  }

  static getRPTierFromPoints(rpPoints: number): number {
    if (rpPoints < 1000) return 0; // Explorer
    if (rpPoints < 5000) return 1; // Connector
    if (rpPoints < 15000) return 2; // Influencer
    if (rpPoints < 50000) return 3; // Leader
    return 4; // Ambassador
  }

  // Staking Calculations
  static calculateStakingRewards(stakedAmount: number, stakingDurationDays: number, userLevel: number, rpTier: number): {
    baseAPY: number;
    totalMultiplier: number;
    dailyReward: number;
    projectedAnnualReward: number;
  } {
    const tierIndex = FINOVA_CONSTANTS.STAKING.TIER_THRESHOLDS.findIndex(threshold => stakedAmount < threshold);
    const actualTierIndex = tierIndex === -1 ? FINOVA_CONSTANTS.STAKING.TIER_THRESHOLDS.length - 1 : Math.max(0, tierIndex - 1);
    
    const baseAPY = FINOVA_CONSTANTS.STAKING.BASE_APY[actualTierIndex];
    const levelBonus = 1 + (userLevel / 100);
    const rpBonus = FINOVA_CONSTANTS.RP.TIER_MULTIPLIERS[Object.keys(FINOVA_CONSTANTS.RP.TIER_MULTIPLIERS)[rpTier] as keyof typeof FINOVA_CONSTANTS.RP.TIER_MULTIPLIERS];
    const loyaltyBonus = 1 + (Math.floor(stakingDurationDays / 30) * FINOVA_CONSTANTS.STAKING.LOYALTY_BONUS_RATE);
    
    const totalMultiplier = levelBonus * rpBonus * loyaltyBonus;
    const effectiveAPY = (baseAPY / 100) * totalMultiplier;
    const dailyReward = (stakedAmount * effectiveAPY) / 365;
    const projectedAnnualReward = stakedAmount * effectiveAPY;
    
    return {
      baseAPY,
      totalMultiplier,
      dailyReward,
      projectedAnnualReward
    };
  }

  // Token Economics
  static calculateTokenValue(circulatingSupply: number, totalRevenue: number, burnRate: number): {
    marketCap: number;
    tokenPrice: number;
    supplyAfterBurn: number;
    priceImpact: number;
  } {
    const baseValue = totalRevenue / circulatingSupply;
    const supplyAfterBurn = circulatingSupply * (1 - burnRate);
    const adjustedValue = totalRevenue / supplyAfterBurn;
    const priceImpact = ((adjustedValue - baseValue) / baseValue) * 100;
    
    return {
      marketCap: totalRevenue,
      tokenPrice: adjustedValue,
      supplyAfterBurn,
      priceImpact
    };
  }

  // Utility Functions
  static formatFinAmount(amount: number, decimals: number = 4): string {
    return amount.toFixed(decimals);
  }

  static calculateTimeToNextLevel(currentXP: number, targetLevel: number): number {
    const currentLevel = this.calculateLevelFromXP(currentXP);
    if (currentLevel >= targetLevel) return 0;
    
    const xpForTargetLevel = this.getXPRequiredForLevel(targetLevel);
    const xpNeeded = xpForTargetLevel - currentXP;
    
    return Math.max(0, xpNeeded);
  }

  private static getXPRequiredForLevel(level: number): number {
    if (level <= 10) return (level - 1) * 100;
    if (level <= 25) return 1000 + (level - 10) * 250;
    if (level <= 50) return 5000 + (level - 25) * 600;
    if (level <= 75) return 20000 + (level - 50) * 1200;
    if (level <= 100) return 50000 + (level - 75) * 2000;
    return 100000 + (level - 100) * 5000;
  }

  static calculateNetworkGrowthRate(currentUsers: number, previousUsers: number, timeframeDays: number): number {
    if (previousUsers === 0 || timeframeDays === 0) return 0;
    const growthRate = ((currentUsers - previousUsers) / previousUsers) * 100;
    return growthRate / timeframeDays; // Daily growth rate
  }

  static predictMiningPhaseTransition(currentUsers: number, dailyGrowthRate: number): {
    daysToNextPhase: number;
    nextPhaseUserThreshold: number;
    currentPhase: number;
  } {
    let nextThreshold: number;
    let currentPhase: number;
    
    if (currentUsers < 100000) {
      nextThreshold = 100000;
      currentPhase = 1;
    } else if (currentUsers < 1000000) {
      nextThreshold = 1000000;
      currentPhase = 2;
    } else if (currentUsers < 10000000) {
      nextThreshold = 10000000;
      currentPhase = 3;
    } else {
      nextThreshold = currentUsers;
      currentPhase = 4;
    }
    
    const usersToNextPhase = nextThreshold - currentUsers;
    const daysToNextPhase = dailyGrowthRate > 0 ? Math.ceil(usersToNextPhase / (currentUsers * (dailyGrowthRate / 100))) : Infinity;
    
    return {
      daysToNextPhase: Math.max(0, daysToNextPhase),
      nextPhaseUserThreshold: nextThreshold,
      currentPhase
    };
  }
}
