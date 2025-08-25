// finova-net/finova/client/typescript/src/utils/index.ts

/**
 * Finova Network Client SDK - Core Utilities
 * Enterprise-grade TypeScript utilities for XP, RP, Mining calculations
 * 
 * @version 3.0.0
 * @author Finova Network Team
 * @license MIT
 */

import { BN } from '@coral-xyz/anchor';
import { PublicKey } from '@solana/web3.js';

// ============================================================================
// TYPE DEFINITIONS
// ============================================================================

export interface UserProfile {
  publicKey: PublicKey;
  level: number;
  totalXP: number;
  totalRP: number;
  totalHoldings: BN;
  referralCount: number;
  activeReferrals: number;
  isKYCVerified: boolean;
  streakDays: number;
  lastActivity: Date;
  guildId?: PublicKey;
  stakingTier: number;
}

export interface ActivityData {
  type: ActivityType;
  platform: SocialPlatform;
  content: string;
  timestamp: Date;
  quality: number;
  engagement: number;
  views?: number;
}

export interface MiningSession {
  userId: PublicKey;
  startTime: Date;
  baseRate: number;
  multipliers: MultiplierSet;
  totalEarned: BN;
  status: 'active' | 'paused' | 'completed';
}

export interface NetworkStats {
  totalUsers: number;
  activeUsers: number;
  totalSupply: BN;
  currentPhase: MiningPhase;
  networkQuality: number;
}

export enum ActivityType {
  POST = 'post',
  COMMENT = 'comment',
  LIKE = 'like',
  SHARE = 'share',
  FOLLOW = 'follow',
  STORY = 'story',
  VIDEO = 'video',
  LIVE = 'live'
}

export enum SocialPlatform {
  INSTAGRAM = 'instagram',
  TIKTOK = 'tiktok',
  YOUTUBE = 'youtube',
  FACEBOOK = 'facebook',
  TWITTER = 'twitter',
  FINOVA = 'finova'
}

export enum MiningPhase {
  FINIZEN = 'finizen',
  GROWTH = 'growth',
  MATURITY = 'maturity',
  STABILITY = 'stability'
}

export interface MultiplierSet {
  xp: number;
  rp: number;
  mining: number;
  quality: number;
  staking: number;
  loyalty: number;
}

// ============================================================================
// CONSTANTS
// ============================================================================

export const FINOVA_CONSTANTS = {
  // Mining Constants
  BASE_MINING_RATES: {
    [MiningPhase.FINIZEN]: 0.1,
    [MiningPhase.GROWTH]: 0.05,
    [MiningPhase.MATURITY]: 0.025,
    [MiningPhase.STABILITY]: 0.01
  },
  
  // XP Constants
  BASE_XP_VALUES: {
    [ActivityType.POST]: 50,
    [ActivityType.COMMENT]: 25,
    [ActivityType.LIKE]: 5,
    [ActivityType.SHARE]: 15,
    [ActivityType.FOLLOW]: 20,
    [ActivityType.STORY]: 25,
    [ActivityType.VIDEO]: 150,
    [ActivityType.LIVE]: 200
  },
  
  // Platform Multipliers
  PLATFORM_MULTIPLIERS: {
    [SocialPlatform.TIKTOK]: 1.3,
    [SocialPlatform.YOUTUBE]: 1.4,
    [SocialPlatform.INSTAGRAM]: 1.2,
    [SocialPlatform.TWITTER]: 1.2,
    [SocialPlatform.FACEBOOK]: 1.1,
    [SocialPlatform.FINOVA]: 1.0
  },
  
  // Level Thresholds
  LEVEL_THRESHOLDS: [
    0, 100, 250, 500, 1000, 2000, 4000, 8000, 15000, 25000,
    40000, 60000, 85000, 120000, 160000, 210000, 270000, 340000, 420000, 510000
  ],
  
  // RP Tier Thresholds
  RP_TIERS: {
    EXPLORER: { min: 0, max: 999, bonus: 0, referralBonus: 0.1 },
    CONNECTOR: { min: 1000, max: 4999, bonus: 0.2, referralBonus: 0.15 },
    INFLUENCER: { min: 5000, max: 14999, bonus: 0.5, referralBonus: 0.2 },
    LEADER: { min: 15000, max: 49999, bonus: 1.0, referralBonus: 0.25 },
    AMBASSADOR: { min: 50000, max: Infinity, bonus: 2.0, referralBonus: 0.3 }
  },
  
  // Regression Parameters
  REGRESSION: {
    MINING_DECAY: 0.001,
    XP_LEVEL_DECAY: 0.01,
    RP_NETWORK_DECAY: 0.0001,
    WHALE_THRESHOLD: 100000
  },
  
  // Quality Score Bounds
  QUALITY_BOUNDS: {
    MIN: 0.5,
    MAX: 2.0,
    DEFAULT: 1.0
  }
} as const;

// ============================================================================
// CORE CALCULATION UTILITIES
// ============================================================================

/**
 * Calculate user's current XP level based on total XP
 */
export function calculateXPLevel(totalXP: number): number {
  for (let i = FINOVA_CONSTANTS.LEVEL_THRESHOLDS.length - 1; i >= 0; i--) {
    if (totalXP >= FINOVA_CONSTANTS.LEVEL_THRESHOLDS[i]) {
      return i + 1;
    }
  }
  return 1;
}

/**
 * Calculate XP required for next level
 */
export function getXPForNextLevel(currentXP: number): number {
  const currentLevel = calculateXPLevel(currentXP);
  if (currentLevel >= FINOVA_CONSTANTS.LEVEL_THRESHOLDS.length) {
    return 0; // Max level reached
  }
  return FINOVA_CONSTANTS.LEVEL_THRESHOLDS[currentLevel] - currentXP;
}

/**
 * Calculate XP gain from activity with all multipliers
 */
export function calculateXPGain(
  activity: ActivityData,
  user: UserProfile,
  streakMultiplier: number = 1.0
): number {
  const baseXP = FINOVA_CONSTANTS.BASE_XP_VALUES[activity.type];
  const platformMultiplier = FINOVA_CONSTANTS.PLATFORM_MULTIPLIERS[activity.platform];
  const qualityScore = Math.max(
    FINOVA_CONSTANTS.QUALITY_BOUNDS.MIN,
    Math.min(FINOVA_CONSTANTS.QUALITY_BOUNDS.MAX, activity.quality)
  );
  const levelProgression = Math.exp(-FINOVA_CONSTANTS.REGRESSION.XP_LEVEL_DECAY * user.level);
  
  // Viral content bonus
  let viralBonus = 1.0;
  if (activity.views && activity.views >= 1000) {
    viralBonus = Math.min(2.0, 1.0 + Math.log10(activity.views / 1000) * 0.3);
  }
  
  return Math.floor(
    baseXP * 
    platformMultiplier * 
    qualityScore * 
    streakMultiplier * 
    levelProgression * 
    viralBonus
  );
}

/**
 * Calculate current mining phase based on total users
 */
export function getCurrentMiningPhase(totalUsers: number): MiningPhase {
  if (totalUsers < 100000) return MiningPhase.FINIZEN;
  if (totalUsers < 1000000) return MiningPhase.GROWTH;
  if (totalUsers < 10000000) return MiningPhase.MATURITY;
  return MiningPhase.STABILITY;
}

/**
 * Calculate mining rate with exponential regression
 */
export function calculateMiningRate(
  user: UserProfile,
  networkStats: NetworkStats
): number {
  const baseRate = FINOVA_CONSTANTS.BASE_MINING_RATES[networkStats.currentPhase];
  
  // Pioneer bonus (decreases as network grows)
  const pioneerBonus = Math.max(1.0, 2.0 - (networkStats.totalUsers / 1000000));
  
  // Referral bonus
  const referralBonus = 1 + (user.activeReferrals * 0.1);
  
  // Security bonus
  const securityBonus = user.isKYCVerified ? 1.2 : 0.8;
  
  // Exponential regression for whale prevention
  const holdingsNum = user.totalHoldings.toNumber();
  const regressionFactor = Math.exp(-FINOVA_CONSTANTS.REGRESSION.MINING_DECAY * holdingsNum);
  
  // XP level bonus
  const xpBonus = 1 + (user.level / 100);
  
  return baseRate * pioneerBonus * referralBonus * securityBonus * regressionFactor * xpBonus;
}

/**
 * Calculate RP tier from total RP
 */
export function getRPTier(totalRP: number): keyof typeof FINOVA_CONSTANTS.RP_TIERS {
  for (const [tierName, tier] of Object.entries(FINOVA_CONSTANTS.RP_TIERS)) {
    if (totalRP >= tier.min && totalRP <= tier.max) {
      return tierName as keyof typeof FINOVA_CONSTANTS.RP_TIERS;
    }
  }
  return 'EXPLORER';
}

/**
 * Calculate RP value from network activity
 */
export function calculateRPValue(
  user: UserProfile,
  referralNetwork: UserProfile[]
): number {
  // Direct referral points
  const directRP = referralNetwork.reduce((sum, referral) => {
    const activity = Math.min(referral.totalXP / 1000, 10); // Cap activity score
    const timeDecay = Math.exp(-0.001 * getDaysSinceLastActivity(referral.lastActivity));
    return sum + (activity * timeDecay);
  }, 0);
  
  // Network quality bonus
  const activeReferrals = referralNetwork.filter(r => 
    getDaysSinceLastActivity(r.lastActivity) <= 30
  ).length;
  const networkQuality = activeReferrals / Math.max(1, referralNetwork.length);
  const avgLevel = referralNetwork.reduce((sum, r) => sum + r.level, 0) / 
                   Math.max(1, referralNetwork.length);
  
  const qualityBonus = networkQuality * (avgLevel / 10) * 0.8;
  
  // Network size regression
  const networkSize = referralNetwork.length;
  const regressionFactor = Math.exp(
    -FINOVA_CONSTANTS.REGRESSION.RP_NETWORK_DECAY * networkSize * networkQuality
  );
  
  return Math.floor((directRP + qualityBonus) * regressionFactor);
}

/**
 * Calculate staking multipliers based on staked amount
 */
export function calculateStakingMultipliers(stakedAmount: BN): MultiplierSet {
  const amount = stakedAmount.toNumber();
  
  if (amount >= 10000) {
    return { xp: 1.75, rp: 1.5, mining: 2.0, quality: 1.0, staking: 1.15, loyalty: 1.0 };
  } else if (amount >= 5000) {
    return { xp: 1.5, rp: 1.35, mining: 1.75, quality: 1.0, staking: 1.14, loyalty: 1.0 };
  } else if (amount >= 1000) {
    return { xp: 1.3, rp: 1.2, mining: 1.5, quality: 1.0, staking: 1.12, loyalty: 1.0 };
  } else if (amount >= 500) {
    return { xp: 1.2, rp: 1.1, mining: 1.35, quality: 1.0, staking: 1.1, loyalty: 1.0 };
  } else if (amount >= 100) {
    return { xp: 1.1, rp: 1.05, mining: 1.2, quality: 1.0, staking: 1.08, loyalty: 1.0 };
  }
  
  return { xp: 1.0, rp: 1.0, mining: 1.0, quality: 1.0, staking: 1.0, loyalty: 1.0 };
}

// ============================================================================
// UTILITY FUNCTIONS
// ============================================================================

/**
 * Get days since last activity
 */
export function getDaysSinceLastActivity(lastActivity: Date): number {
  return Math.floor((Date.now() - lastActivity.getTime()) / (1000 * 60 * 60 * 24));
}

/**
 * Calculate streak bonus based on consecutive days
 */
export function calculateStreakBonus(streakDays: number): number {
  if (streakDays < 3) return 1.0;
  if (streakDays < 7) return 1.2;
  if (streakDays < 14) return 1.5;
  if (streakDays < 30) return 2.0;
  return Math.min(3.0, 2.0 + (streakDays - 30) * 0.01);
}

/**
 * Validate content quality score
 */
export function validateQualityScore(score: number): number {
  return Math.max(
    FINOVA_CONSTANTS.QUALITY_BOUNDS.MIN,
    Math.min(FINOVA_CONSTANTS.QUALITY_BOUNDS.MAX, score)
  );
}

/**
 * Format token amount with proper decimals
 */
export function formatTokenAmount(amount: BN, decimals: number = 9): string {
  const divisor = new BN(10).pow(new BN(decimals));
  const quotient = amount.div(divisor);
  const remainder = amount.mod(divisor);
  
  if (remainder.isZero()) {
    return quotient.toString();
  }
  
  const remainderStr = remainder.toString().padStart(decimals, '0');
  const trimmedRemainder = remainderStr.replace(/0+$/, '');
  
  return trimmedRemainder.length > 0 ? 
    `${quotient.toString()}.${trimmedRemainder}` : 
    quotient.toString();
}

/**
 * Parse token amount to BN with decimals
 */
export function parseTokenAmount(amount: string, decimals: number = 9): BN {
  const [whole, fraction = ''] = amount.split('.');
  const paddedFraction = fraction.padEnd(decimals, '0').slice(0, decimals);
  const fullAmount = whole + paddedFraction;
  return new BN(fullAmount);
}

/**
 * Calculate comprehensive user rewards
 */
export function calculateTotalRewards(
  user: UserProfile,
  networkStats: NetworkStats,
  referralNetwork: UserProfile[],
  stakingMultipliers: MultiplierSet,
  hoursActive: number = 24
): {
  miningReward: number;
  xpBonus: number;
  rpBonus: number;
  totalReward: number;
  breakdown: Record<string, number>;
} {
  const baseMining = calculateMiningRate(user, networkStats);
  const rpValue = calculateRPValue(user, referralNetwork);
  const rpTier = getRPTier(rpValue);
  const rpMultiplier = FINOVA_CONSTANTS.RP_TIERS[rpTier].bonus;
  
  const miningReward = baseMining * hoursActive * stakingMultipliers.mining;
  const xpBonus = miningReward * 0.2 * (user.level / 100) * stakingMultipliers.xp;
  const rpBonus = miningReward * 0.3 * rpMultiplier * stakingMultipliers.rp;
  
  const totalReward = miningReward + xpBonus + rpBonus;
  
  return {
    miningReward,
    xpBonus,
    rpBonus,
    totalReward,
    breakdown: {
      baseMining: baseMining * hoursActive,
      stakingBonus: (miningReward - (baseMining * hoursActive)),
      xpLevelBonus: xpBonus,
      rpNetworkBonus: rpBonus,
      qualityMultiplier: 0 // To be calculated based on content quality
    }
  };
}

/**
 * Estimate mining potential for next 24 hours
 */
export function estimateDailyEarnings(
  user: UserProfile,
  networkStats: NetworkStats,
  referralNetwork: UserProfile[],
  plannedActivities: ActivityData[] = []
): {
  estimatedMining: number;
  estimatedXP: number;
  estimatedRP: number;
  projectedLevel: number;
  recommendations: string[];
} {
  const stakingMultipliers = calculateStakingMultipliers(user.totalHoldings);
  const rewards = calculateTotalRewards(user, networkStats, referralNetwork, stakingMultipliers);
  
  let estimatedXP = 0;
  const streakBonus = calculateStreakBonus(user.streakDays + 1);
  
  for (const activity of plannedActivities) {
    estimatedXP += calculateXPGain(activity, user, streakBonus);
  }
  
  const currentRP = calculateRPValue(user, referralNetwork);
  const estimatedRP = currentRP + (referralNetwork.length * 10); // Rough estimate
  
  const projectedTotalXP = user.totalXP + estimatedXP;
  const projectedLevel = calculateXPLevel(projectedTotalXP);
  
  const recommendations: string[] = [];
  
  if (user.streakDays < 7) {
    recommendations.push("Maintain daily streak for 2x bonus");
  }
  if (user.activeReferrals < 10) {
    recommendations.push("Invite more friends for referral bonuses");
  }
  if (!user.isKYCVerified) {
    recommendations.push("Complete KYC for 20% mining bonus");
  }
  if (plannedActivities.length < 5) {
    recommendations.push("Create more quality content for higher XP");
  }
  
  return {
    estimatedMining: rewards.totalReward,
    estimatedXP,
    estimatedRP,
    projectedLevel,
    recommendations
  };
}

/**
 * Generate mining session summary
 */
export function generateMiningReport(
  session: MiningSession,
  user: UserProfile,
  activities: ActivityData[]
): {
  duration: number;
  totalEarned: string;
  xpGained: number;
  activitiesCompleted: number;
  efficiency: number;
  nextOptimalSession: Date;
} {
  const duration = (Date.now() - session.startTime.getTime()) / (1000 * 60 * 60);
  const xpGained = activities.reduce((sum, activity) => 
    sum + calculateXPGain(activity, user), 0
  );
  
  const efficiency = session.totalEarned.toNumber() / Math.max(1, duration);
  const nextOptimalSession = new Date(Date.now() + (8 * 60 * 60 * 1000)); // 8 hours later
  
  return {
    duration,
    totalEarned: formatTokenAmount(session.totalEarned),
    xpGained,
    activitiesCompleted: activities.length,
    efficiency,
    nextOptimalSession
  };
}

// ============================================================================
// VALIDATION UTILITIES
// ============================================================================

/**
 * Validate user profile data
 */
export function validateUserProfile(profile: Partial<UserProfile>): {
  isValid: boolean;
  errors: string[];
} {
  const errors: string[] = [];
  
  if (!profile.publicKey) {
    errors.push("Public key is required");
  }
  
  if (typeof profile.level !== 'number' || profile.level < 1) {
    errors.push("Invalid level value");
  }
  
  if (typeof profile.totalXP !== 'number' || profile.totalXP < 0) {
    errors.push("Invalid XP value");
  }
  
  if (typeof profile.totalRP !== 'number' || profile.totalRP < 0) {
    errors.push("Invalid RP value");
  }
  
  if (!BN.isBN(profile.totalHoldings) || profile.totalHoldings.isNeg()) {
    errors.push("Invalid holdings value");
  }
  
  return {
    isValid: errors.length === 0,
    errors
  };
}

/**
 * Validate activity data
 */
export function validateActivityData(activity: Partial<ActivityData>): {
  isValid: boolean;
  errors: string[];
} {
  const errors: string[] = [];
  
  if (!Object.values(ActivityType).includes(activity.type as ActivityType)) {
    errors.push("Invalid activity type");
  }
  
  if (!Object.values(SocialPlatform).includes(activity.platform as SocialPlatform)) {
    errors.push("Invalid platform");
  }
  
  if (!activity.content || activity.content.trim().length === 0) {
    errors.push("Content cannot be empty");
  }
  
  if (typeof activity.quality !== 'number' || 
      activity.quality < FINOVA_CONSTANTS.QUALITY_BOUNDS.MIN || 
      activity.quality > FINOVA_CONSTANTS.QUALITY_BOUNDS.MAX) {
    errors.push("Quality score must be between 0.5 and 2.0");
  }
  
  return {
    isValid: errors.length === 0,
    errors
  };
}

// ============================================================================
// EXPORT ALL UTILITIES
// ============================================================================

export * from './calculations';
export * from './formatting';
export * from './validation';

export default {
  // Core calculations
  calculateXPLevel,
  getXPForNextLevel,
  calculateXPGain,
  calculateMiningRate,
  calculateRPValue,
  calculateStakingMultipliers,
  calculateTotalRewards,
  
  // Utility functions
  getDaysSinceLastActivity,
  calculateStreakBonus,
  validateQualityScore,
  formatTokenAmount,
  parseTokenAmount,
  
  // Analysis functions
  estimateDailyEarnings,
  generateMiningReport,
  getCurrentMiningPhase,
  getRPTier,
  
  // Validation
  validateUserProfile,
  validateActivityData,
  
  // Constants
  FINOVA_CONSTANTS
};
