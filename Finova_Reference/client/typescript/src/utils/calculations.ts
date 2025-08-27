// finova-net/finova/client/typescript/src/utils/calculations.ts

/**
 * Finova Network - Core Calculations Utility
 * Enterprise-grade calculation functions for XP, RP, Mining, and Rewards
 * 
 * @version 3.0.0
 * @author Finova Network Development Team
 * @license MIT
 */

import { BN } from '@project-serum/anchor';
import { PublicKey } from '@solana/web3.js';

// ============================================================================
// TYPES & INTERFACES
// ============================================================================

export interface UserProfile {
  publicKey: PublicKey;
  level: number;
  totalXp: number;
  totalRp: number;
  totalHoldings: BN;
  stakedAmount: BN;
  referralCount: number;
  activeReferrals: number;
  networkSize: number;
  isKycVerified: boolean;
  streakDays: number;
  joinedAt: Date;
  lastActivity: Date;
}

export interface Activity {
  type: ActivityType;
  platform: SocialPlatform;
  content: string;
  engagement: number;
  timestamp: Date;
  qualityScore?: number;
}

export interface MiningSession {
  userId: PublicKey;
  startTime: Date;
  duration: number; // hours
  baseRate: number;
  multipliers: MultiplierSet;
  totalMined: BN;
}

export interface MultiplierSet {
  xp: number;
  rp: number;
  staking: number;
  quality: number;
  special: number;
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
  LINKEDIN = 'linkedin'
}

export enum MiningPhase {
  FINIZEN = 'finizen',
  GROWTH = 'growth',
  MATURITY = 'maturity',
  STABILITY = 'stability'
}

export enum RPTier {
  EXPLORER = 'explorer',
  CONNECTOR = 'connector',
  INFLUENCER = 'influencer',
  LEADER = 'leader',
  AMBASSADOR = 'ambassador'
}

// ============================================================================
// CONSTANTS & CONFIGURATION
// ============================================================================

export const MINING_CONFIG = {
  phases: {
    [MiningPhase.FINIZEN]: { userLimit: 100000, baseRate: 0.1, finizenBonus: 2.0 },
    [MiningPhase.GROWTH]: { userLimit: 1000000, baseRate: 0.05, finizenBonus: 1.5 },
    [MiningPhase.MATURITY]: { userLimit: 10000000, baseRate: 0.025, finizenBonus: 1.2 },
    [MiningPhase.STABILITY]: { userLimit: Infinity, baseRate: 0.01, finizenBonus: 1.0 }
  },
  maxDailyMultiplier: 24,
  regressionFactor: 0.001,
  securityBonus: { kyc: 1.2, nonKyc: 0.8 }
};

export const XP_CONFIG = {
  baseXp: {
    [ActivityType.POST]: 50,
    [ActivityType.COMMENT]: 25,
    [ActivityType.LIKE]: 5,
    [ActivityType.SHARE]: 15,
    [ActivityType.FOLLOW]: 20,
    [ActivityType.STORY]: 25,
    [ActivityType.VIDEO]: 150,
    [ActivityType.LIVE]: 200
  },
  platformMultipliers: {
    [SocialPlatform.TIKTOK]: 1.3,
    [SocialPlatform.YOUTUBE]: 1.4,
    [SocialPlatform.INSTAGRAM]: 1.2,
    [SocialPlatform.TWITTER]: 1.2,
    [SocialPlatform.FACEBOOK]: 1.1,
    [SocialPlatform.LINKEDIN]: 1.0
  },
  levelProgression: 0.01,
  maxStreakBonus: 3.0,
  qualityRange: { min: 0.5, max: 2.0 },
  dailyLimits: {
    [ActivityType.POST]: -1, // unlimited
    [ActivityType.COMMENT]: 100,
    [ActivityType.LIKE]: 200,
    [ActivityType.SHARE]: 50,
    [ActivityType.FOLLOW]: 25,
    [ActivityType.STORY]: 50,
    [ActivityType.VIDEO]: 10,
    [ActivityType.LIVE]: 5
  }
};

export const RP_CONFIG = {
  tiers: {
    [RPTier.EXPLORER]: { min: 0, max: 999, miningBonus: 0, referralBonus: 0.1, networkCap: 10 },
    [RPTier.CONNECTOR]: { min: 1000, max: 4999, miningBonus: 0.2, referralBonus: 0.15, networkCap: 25 },
    [RPTier.INFLUENCER]: { min: 5000, max: 14999, miningBonus: 0.5, referralBonus: 0.2, networkCap: 50 },
    [RPTier.LEADER]: { min: 15000, max: 49999, miningBonus: 1.0, referralBonus: 0.25, networkCap: 100 },
    [RPTier.AMBASSADOR]: { min: 50000, max: Infinity, miningBonus: 2.0, referralBonus: 0.3, networkCap: -1 }
  },
  networkRegressionFactor: 0.0001,
  activityDecayRate: 0.95, // daily decay for inactive referrals
  qualityThreshold: 0.5
};

export const STAKING_CONFIG = {
  tiers: [
    { min: 100, max: 499, apy: 0.08, miningBoost: 0.2, xpBonus: 0.1, rpBonus: 0.05 },
    { min: 500, max: 999, apy: 0.10, miningBoost: 0.35, xpBonus: 0.2, rpBonus: 0.1 },
    { min: 1000, max: 4999, apy: 0.12, miningBoost: 0.5, xpBonus: 0.3, rpBonus: 0.2 },
    { min: 5000, max: 9999, apy: 0.14, miningBoost: 0.75, xpBonus: 0.5, rpBonus: 0.35 },
    { min: 10000, max: Infinity, apy: 0.15, miningBoost: 1.0, xpBonus: 0.75, rpBonus: 0.5 }
  ],
  loyaltyBonus: 0.05, // per month
  activityBonus: 0.1 // per activity score
};

// ============================================================================
// CORE CALCULATION FUNCTIONS
// ============================================================================

/**
 * Calculates the current mining phase based on total network users
 */
export function getCurrentMiningPhase(totalUsers: number): MiningPhase {
  if (totalUsers <= MINING_CONFIG.phases[MiningPhase.FINIZEN].userLimit) return MiningPhase.FINIZEN;
  if (totalUsers <= MINING_CONFIG.phases[MiningPhase.GROWTH].userLimit) return MiningPhase.GROWTH;
  if (totalUsers <= MINING_CONFIG.phases[MiningPhase.MATURITY].userLimit) return MiningPhase.MATURITY;
  return MiningPhase.STABILITY;
}

/**
 * Calculates hourly mining rate for a user
 */
export function calculateMiningRate(
  user: UserProfile,
  totalUsers: number,
  customMultipliers: Partial<MultiplierSet> = {}
): number {
  const phase = getCurrentMiningPhase(totalUsers);
  const config = MINING_CONFIG.phases[phase];
  
  // Base components
  const baseRate = config.baseRate;
  const finizenBonus = Math.max(1.0, config.finizenBonus - (totalUsers / 1000000));
  const referralBonus = 1 + (user.activeReferrals * 0.1);
  const securityBonus = user.isKycVerified ? 
    MINING_CONFIG.securityBonus.kyc : 
    MINING_CONFIG.securityBonus.nonKyc;
  
  // Exponential regression to prevent whale dominance
  const holdingsInFin = user.totalHoldings.toNumber() / 1e9; // Convert lamports to FIN
  const regressionFactor = Math.exp(-MINING_CONFIG.regressionFactor * holdingsInFin);
  
  // XP level multiplier
  const xpMultiplier = 1 + (user.level / 100) * (customMultipliers.xp || 1);
  
  // RP tier multiplier
  const rpTier = getRPTier(user.totalRp);
  const rpMultiplier = 1 + RP_CONFIG.tiers[rpTier].miningBonus * (customMultipliers.rp || 1);
  
  // Staking multiplier
  const stakingMultiplier = calculateStakingMultiplier(user.stakedAmount) * (customMultipliers.staking || 1);
  
  // Quality multiplier (default to 1.0 if not provided)
  const qualityMultiplier = customMultipliers.quality || 1.0;
  
  // Special multiplier (cards, events, etc.)
  const specialMultiplier = customMultipliers.special || 1.0;
  
  return baseRate * 
         finizenBonus * 
         referralBonus * 
         securityBonus * 
         regressionFactor * 
         xpMultiplier * 
         rpMultiplier * 
         stakingMultiplier * 
         qualityMultiplier * 
         specialMultiplier;
}

/**
 * Calculates XP gained from an activity
 */
export function calculateXPGain(
  activity: Activity,
  user: UserProfile,
  engagementMetrics?: { views?: number; likes?: number; shares?: number }
): number {
  const baseXp = XP_CONFIG.baseXp[activity.type];
  const platformMultiplier = XP_CONFIG.platformMultipliers[activity.platform] || 1.0;
  
  // Quality score (AI-evaluated or provided)
  const qualityScore = activity.qualityScore || 1.0;
  const clampedQuality = Math.max(XP_CONFIG.qualityRange.min, Math.min(XP_CONFIG.qualityRange.max, qualityScore));
  
  // Streak bonus
  const streakBonus = Math.min(XP_CONFIG.maxStreakBonus, 1 + (user.streakDays * 0.1));
  
  // Level progression (diminishing returns)
  const levelProgression = Math.exp(-XP_CONFIG.levelProgression * user.level);
  
  // Viral content bonus
  let viralBonus = 1.0;
  if (engagementMetrics?.views && engagementMetrics.views >= 1000) {
    viralBonus = 1 + Math.log10(engagementMetrics.views / 1000);
  }
  
  return Math.floor(
    baseXp * 
    platformMultiplier * 
    clampedQuality * 
    streakBonus * 
    levelProgression * 
    viralBonus
  );
}

/**
 * Calculates RP (Referral Points) value for a user's network
 */
export function calculateRPValue(
  user: UserProfile,
  referralNetwork: UserProfile[]
): number {
  // Direct referral points
  const directRP = referralNetwork.reduce((sum, referral) => {
    const activityMultiplier = calculateActivityMultiplier(referral.lastActivity);
    const levelMultiplier = 1 + (referral.level / 100);
    return sum + (100 * activityMultiplier * levelMultiplier);
  }, 0);
  
  // Network quality assessment
  const activeReferrals = referralNetwork.filter(r => isReferralActive(r)).length;
  const networkQuality = activeReferrals / Math.max(1, referralNetwork.length);
  const avgLevel = referralNetwork.reduce((sum, r) => sum + r.level, 0) / Math.max(1, referralNetwork.length);
  
  // Quality bonus
  const qualityBonus = networkQuality * (1 + avgLevel / 100) * calculateRetentionRate(referralNetwork);
  
  // Network regression to prevent abuse
  const totalNetworkSize = user.networkSize;
  const regressionFactor = Math.exp(
    -RP_CONFIG.networkRegressionFactor * totalNetworkSize * networkQuality
  );
  
  return Math.floor((directRP * qualityBonus * regressionFactor));
}

/**
 * Determines RP tier based on total RP value
 */
export function getRPTier(totalRP: number): RPTier {
  for (const [tier, config] of Object.entries(RP_CONFIG.tiers)) {
    if (totalRP >= config.min && totalRP <= config.max) {
      return tier as RPTier;
    }
  }
  return RPTier.EXPLORER;
}

/**
 * Calculates staking multiplier based on staked amount
 */
export function calculateStakingMultiplier(stakedAmount: BN): number {
  const stakedFin = stakedAmount.toNumber() / 1e9; // Convert to FIN
  
  for (const tier of STAKING_CONFIG.tiers) {
    if (stakedFin >= tier.min && stakedFin <= tier.max) {
      return 1 + tier.miningBoost;
    }
  }
  
  return 1.0; // No staking bonus
}

/**
 * Calculates staking rewards APY and benefits
 */
export function calculateStakingRewards(
  stakedAmount: BN,
  stakingDuration: number, // in months
  activityScore: number // 0-100
): {
  apy: number;
  miningBoost: number;
  xpBonus: number;
  rpBonus: number;
  totalMultiplier: number;
} {
  const stakedFin = stakedAmount.toNumber() / 1e9;
  
  const tier = STAKING_CONFIG.tiers.find(t => stakedFin >= t.min && stakedFin <= t.max) || STAKING_CONFIG.tiers[0];
  
  const loyaltyBonus = 1 + (stakingDuration * STAKING_CONFIG.loyaltyBonus);
  const activityBonus = 1 + ((activityScore / 100) * STAKING_CONFIG.activityBonus);
  
  return {
    apy: tier.apy * loyaltyBonus * activityBonus,
    miningBoost: tier.miningBoost,
    xpBonus: tier.xpBonus,
    rpBonus: tier.rpBonus,
    totalMultiplier: loyaltyBonus * activityBonus
  };
}

/**
 * Calculates total integrated reward for a user session
 */
export function calculateIntegratedReward(
  user: UserProfile,
  activities: Activity[],
  totalUsers: number,
  sessionDuration: number // in hours
): {
  miningReward: number;
  xpGained: number;
  rpGained: number;
  totalValue: number;
  multipliers: MultiplierSet;
} {
  // Calculate base mining reward
  const hourlyMiningRate = calculateMiningRate(user, totalUsers);
  const baseMiningReward = hourlyMiningRate * sessionDuration;
  
  // Calculate XP from activities
  let totalXP = 0;
  for (const activity of activities) {
    totalXP += calculateXPGain(activity, user);
  }
  
  // XP bonus to mining (20% of base mining per XP level)
  const xpBonus = (user.level / 100) * baseMiningReward * 0.2;
  
  // RP bonus to mining (30% of base mining per RP tier)
  const rpTier = getRPTier(user.totalRp);
  const rpBonus = RP_CONFIG.tiers[rpTier].miningBonus * baseMiningReward * 0.3;
  
  // Quality multiplier from activities
  const avgQuality = activities.reduce((sum, a) => sum + (a.qualityScore || 1.0), 0) / Math.max(1, activities.length);
  const qualityMultiplier = Math.max(0.5, Math.min(2.0, avgQuality));
  
  const multipliers: MultiplierSet = {
    xp: 1 + (user.level / 100),
    rp: 1 + RP_CONFIG.tiers[rpTier].miningBonus,
    staking: calculateStakingMultiplier(user.stakedAmount),
    quality: qualityMultiplier,
    special: 1.0 // Can be enhanced with NFT cards
  };
  
  const finalMiningReward = (baseMiningReward + xpBonus + rpBonus) * qualityMultiplier;
  
  return {
    miningReward: finalMiningReward,
    xpGained: totalXP,
    rpGained: 0, // RP is calculated separately from network activity
    totalValue: finalMiningReward + (totalXP * 0.001), // XP has small FIN value
    multipliers
  };
}

// ============================================================================
// UTILITY FUNCTIONS
// ============================================================================

/**
 * Checks if a referral is considered active (active in last 30 days)
 */
export function isReferralActive(referral: UserProfile): boolean {
  const thirtyDaysAgo = new Date(Date.now() - 30 * 24 * 60 * 60 * 1000);
  return referral.lastActivity > thirtyDaysAgo;
}

/**
 * Calculates activity multiplier based on recency
 */
export function calculateActivityMultiplier(lastActivity: Date): number {
  const daysSinceActivity = (Date.now() - lastActivity.getTime()) / (1000 * 60 * 60 * 24);
  return Math.pow(RP_CONFIG.activityDecayRate, daysSinceActivity);
}

/**
 * Calculates network retention rate
 */
export function calculateRetentionRate(network: UserProfile[]): number {
  if (network.length === 0) return 1.0;
  
  const activeCount = network.filter(isReferralActive).length;
  return activeCount / network.length;
}

/**
 * Converts XP to user level
 */
export function xpToLevel(totalXp: number): number {
  // Level formula: level = floor(sqrt(totalXp / 100))
  return Math.floor(Math.sqrt(totalXp / 100));
}

/**
 * Calculates XP required for next level
 */
export function xpForLevel(level: number): number {
  return (level + 1) ** 2 * 100;
}

/**
 * Calculates progress to next level
 */
export function levelProgress(totalXp: number): { 
  currentLevel: number; 
  xpInLevel: number; 
  xpForNext: number; 
  progress: number; 
} {
  const currentLevel = xpToLevel(totalXp);
  const xpForCurrentLevel = currentLevel ** 2 * 100;
  const xpForNextLevel = xpForLevel(currentLevel);
  const xpInLevel = totalXp - xpForCurrentLevel;
  const xpNeededForNext = xpForNextLevel - xpForCurrentLevel;
  
  return {
    currentLevel,
    xpInLevel,
    xpForNext: xpNeededForNext - xpInLevel,
    progress: xpInLevel / xpNeededForNext
  };
}

/**
 * Anti-bot detection: calculates human probability score
 */
export function calculateHumanProbability(user: UserProfile, activities: Activity[]): number {
  let score = 0.5; // Base score
  
  // KYC verification adds significant trust
  if (user.isKycVerified) score += 0.3;
  
  // Activity pattern analysis
  const activityTimes = activities.map(a => a.timestamp.getHours());
  const uniqueHours = new Set(activityTimes).size;
  const circadianNormality = uniqueHours / 24; // More spread = more human-like
  score += circadianNormality * 0.2;
  
  // Content quality variance (humans vary more than bots)
  const qualityScores = activities.map(a => a.qualityScore || 1.0);
  const qualityVariance = calculateVariance(qualityScores);
  score += Math.min(0.2, qualityVariance * 0.5);
  
  // Network authenticity (real humans have varied connection patterns)
  const networkDiversity = user.networkSize > 0 ? 
    Math.min(0.1, user.activeReferrals / user.networkSize) : 0;
  score += networkDiversity;
  
  return Math.max(0.1, Math.min(1.0, score));
}

/**
 * Calculates statistical variance
 */
export function calculateVariance(values: number[]): number {
  if (values.length === 0) return 0;
  
  const mean = values.reduce((sum, val) => sum + val, 0) / values.length;
  const squaredDiffs = values.map(val => Math.pow(val - mean, 2));
  return squaredDiffs.reduce((sum, diff) => sum + diff, 0) / values.length;
}

/**
 * Formats FIN amount for display
 */
export function formatFIN(amount: number | BN, decimals: number = 4): string {
  const value = typeof amount === 'number' ? amount : amount.toNumber() / 1e9;
  return value.toFixed(decimals) + ' FIN';
}

/**
 * Validates calculation inputs
 */
export function validateCalculationInputs(user: UserProfile): boolean {
  return !!(
    user.publicKey &&
    user.level >= 0 &&
    user.totalXp >= 0 &&
    user.totalRp >= 0 &&
    user.totalHoldings.gte(new BN(0)) &&
    user.referralCount >= 0 &&
    user.activeReferrals >= 0 &&
    user.networkSize >= 0
  );
}

/**
 * Calculates compound growth rate for projections
 */
export function calculateCompoundGrowth(
  initialValue: number,
  growthRate: number,
  periods: number
): number {
  return initialValue * Math.pow(1 + growthRate, periods);
}

/**
 * Estimates future mining potential
 */
export function estimateFutureMining(
  user: UserProfile,
  days: number,
  totalUsers: number,
  expectedGrowthRate: number = 0.1
): {
  totalMined: number;
  dailyAverage: number;
  projectedLevel: number;
  projectedRP: number;
} {
  const currentHourlyRate = calculateMiningRate(user, totalUsers);
  const dailyRate = currentHourlyRate * 24;
  
  // Account for network growth reducing mining rates
  const futureUsers = calculateCompoundGrowth(totalUsers, expectedGrowthRate / 365, days);
  const futureHourlyRate = calculateMiningRate(user, futureUsers);
  
  const averageRate = (dailyRate + futureHourlyRate * 24) / 2;
  const totalMined = averageRate * days;
  
  return {
    totalMined,
    dailyAverage: averageRate,
    projectedLevel: user.level + Math.floor(days * 10), // Estimated XP growth
    projectedRP: user.totalRp + Math.floor(days * 5) // Estimated RP growth
  };
}

// Export all calculation functions
export default {
  getCurrentMiningPhase,
  calculateMiningRate,
  calculateXPGain,
  calculateRPValue,
  getRPTier,
  calculateStakingMultiplier,
  calculateStakingRewards,
  calculateIntegratedReward,
  calculateHumanProbability,
  estimateFutureMining,
  // Utility functions
  isReferralActive,
  calculateActivityMultiplier,
  calculateRetentionRate,
  xpToLevel,
  xpForLevel,
  levelProgress,
  formatFIN,
  validateCalculationInputs,
  calculateCompoundGrowth,
  calculateVariance
};
