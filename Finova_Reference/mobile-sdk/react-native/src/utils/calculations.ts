/**
 * Finova Network - React Native SDK Calculations Engine
 * Comprehensive calculation system for Mining, XP, and RP
 * Enterprise-grade implementation with security and performance optimization
 * 
 * @version 1.0.0
 * @author Finova Network Team
 * @license MIT
 */

import { Platform } from 'react-native';

// ===== TYPES & INTERFACES =====

export interface UserProfile {
  id: string;
  level: number;
  totalXP: number;
  totalRP: number;
  totalFINHoldings: number;
  streakDays: number;
  isKYCVerified: boolean;
  registrationDate: Date;
  lastActiveDate: Date;
  stakingAmount: number;
  stakingDuration: number; // months
  humanProbabilityScore: number; // 0.0 - 1.0
}

export interface ReferralNetwork {
  directReferrals: UserProfile[];
  level2Referrals: UserProfile[];
  level3Referrals: UserProfile[];
  totalNetworkSize: number;
  activeReferrals: number;
  networkQualityScore: number; // 0.0 - 1.0
}

export interface Activity {
  type: ActivityType;
  platform: SocialPlatform;
  content: string;
  engagement: {
    views: number;
    likes: number;
    comments: number;
    shares: number;
  };
  timestamp: Date;
  qualityScore: number; // 0.5 - 2.0
}

export interface NetworkStats {
  totalUsers: number;
  activeUsers: number;
  currentPhase: MiningPhase;
  globalMiningRate: number;
}

export interface StakingTier {
  minAmount: number;
  maxAmount: number;
  apyRate: number;
  miningBoost: number;
  xpMultiplier: number;
  rpBonus: number;
}

export enum ActivityType {
  ORIGINAL_POST = 'ORIGINAL_POST',
  COMMENT = 'COMMENT',
  LIKE = 'LIKE',
  SHARE = 'SHARE',
  FOLLOW = 'FOLLOW',
  STORY = 'STORY',
  VIDEO = 'VIDEO',
  PHOTO = 'PHOTO',
  DAILY_LOGIN = 'DAILY_LOGIN',
  QUEST_COMPLETE = 'QUEST_COMPLETE'
}

export enum SocialPlatform {
  INSTAGRAM = 'INSTAGRAM',
  TIKTOK = 'TIKTOK',
  YOUTUBE = 'YOUTUBE',
  FACEBOOK = 'FACEBOOK',
  TWITTER_X = 'TWITTER_X',
  APP_INTERNAL = 'APP_INTERNAL'
}

export enum MiningPhase {
  FINIZEN = 'FINIZEN',
  GROWTH = 'GROWTH',
  MATURITY = 'MATURITY',
  STABILITY = 'STABILITY'
}

export enum RPTier {
  EXPLORER = 'EXPLORER',
  CONNECTOR = 'CONNECTOR',
  INFLUENCER = 'INFLUENCER',
  LEADER = 'LEADER',
  AMBASSADOR = 'AMBASSADOR'
}

// ===== CONSTANTS =====

const MINING_CONSTANTS = {
  PHASES: {
    [MiningPhase.FINIZEN]: {
      baseRate: 0.1,
      finazenBonus: 2.0,
      userThreshold: 100000,
      maxDaily: 4.8
    },
    [MiningPhase.GROWTH]: {
      baseRate: 0.05,
      finazenBonus: 1.5,
      userThreshold: 1000000,
      maxDaily: 1.8
    },
    [MiningPhase.MATURITY]: {
      baseRate: 0.025,
      finazenBonus: 1.2,
      userThreshold: 10000000,
      maxDaily: 0.72
    },
    [MiningPhase.STABILITY]: {
      baseRate: 0.01,
      finazenBonus: 1.0,
      userThreshold: Infinity,
      maxDaily: 0.24
    }
  },
  REGRESSION_COEFFICIENT: 0.001,
  REFERRAL_BONUS_RATE: 0.1,
  KYC_BONUS: 1.2,
  NON_KYC_PENALTY: 0.8
};

const XP_CONSTANTS = {
  BASE_VALUES: {
    [ActivityType.ORIGINAL_POST]: 50,
    [ActivityType.COMMENT]: 25,
    [ActivityType.LIKE]: 5,
    [ActivityType.SHARE]: 15,
    [ActivityType.FOLLOW]: 20,
    [ActivityType.STORY]: 25,
    [ActivityType.VIDEO]: 150,
    [ActivityType.PHOTO]: 75,
    [ActivityType.DAILY_LOGIN]: 10,
    [ActivityType.QUEST_COMPLETE]: 100
  },
  PLATFORM_MULTIPLIERS: {
    [SocialPlatform.TIKTOK]: 1.3,
    [SocialPlatform.YOUTUBE]: 1.4,
    [SocialPlatform.TWITTER_X]: 1.2,
    [SocialPlatform.INSTAGRAM]: 1.2,
    [SocialPlatform.FACEBOOK]: 1.1,
    [SocialPlatform.APP_INTERNAL]: 1.0
  },
  LEVEL_PROGRESSION_COEFFICIENT: 0.01,
  STREAK_MAX_MULTIPLIER: 3.0,
  VIRAL_THRESHOLD: 1000,
  VIRAL_MULTIPLIER: 2.0,
  DAILY_LIMITS: {
    [ActivityType.ORIGINAL_POST]: Infinity,
    [ActivityType.COMMENT]: 100,
    [ActivityType.LIKE]: 200,
    [ActivityType.SHARE]: 50,
    [ActivityType.FOLLOW]: 25,
    [ActivityType.STORY]: 50,
    [ActivityType.VIDEO]: 10,
    [ActivityType.PHOTO]: 20,
    [ActivityType.DAILY_LOGIN]: 1,
    [ActivityType.QUEST_COMPLETE]: 3
  }
};

const RP_CONSTANTS = {
  TIERS: {
    [RPTier.EXPLORER]: { min: 0, max: 999, miningBonus: 0, referralBonus: 0.10, networkCap: 10 },
    [RPTier.CONNECTOR]: { min: 1000, max: 4999, miningBonus: 0.20, referralBonus: 0.15, networkCap: 25 },
    [RPTier.INFLUENCER]: { min: 5000, max: 14999, miningBonus: 0.50, referralBonus: 0.20, networkCap: 50 },
    [RPTier.LEADER]: { min: 15000, max: 49999, miningBonus: 1.00, referralBonus: 0.25, networkCap: 100 },
    [RPTier.AMBASSADOR]: { min: 50000, max: Infinity, miningBonus: 2.00, referralBonus: 0.30, networkCap: Infinity }
  },
  NETWORK_REGRESSION_COEFFICIENT: 0.0001,
  L2_MULTIPLIER: 0.3,
  L3_MULTIPLIER: 0.1,
  QUALITY_WEIGHT: 0.85,
  RETENTION_WEIGHT: 0.85
};

const STAKING_TIERS: StakingTier[] = [
  { minAmount: 100, maxAmount: 499, apyRate: 0.08, miningBoost: 0.20, xpMultiplier: 0.10, rpBonus: 0.05 },
  { minAmount: 500, maxAmount: 999, apyRate: 0.10, miningBoost: 0.35, xpMultiplier: 0.20, rpBonus: 0.10 },
  { minAmount: 1000, maxAmount: 4999, apyRate: 0.12, miningBoost: 0.50, xpMultiplier: 0.30, rpBonus: 0.20 },
  { minAmount: 5000, maxAmount: 9999, apyRate: 0.14, miningBoost: 0.75, xpMultiplier: 0.50, rpBonus: 0.35 },
  { minAmount: 10000, maxAmount: Infinity, apyRate: 0.15, miningBoost: 1.00, xpMultiplier: 0.75, rpBonus: 0.50 }
];

// ===== CORE CALCULATION ENGINE =====

export class FinovaCalculationEngine {
  private static instance: FinovaCalculationEngine;
  private securityValidator: SecurityValidator;
  
  private constructor() {
    this.securityValidator = new SecurityValidator();
  }

  public static getInstance(): FinovaCalculationEngine {
    if (!FinovaCalculationEngine.instance) {
      FinovaCalculationEngine.instance = new FinovaCalculationEngine();
    }
    return FinovaCalculationEngine.instance;
  }

  // ===== MINING CALCULATIONS =====

  /**
   * Calculate hourly mining rate with all bonuses and regressions applied
   */
  public calculateMiningRate(
    user: UserProfile,
    networkStats: NetworkStats,
    referralNetwork: ReferralNetwork
  ): number {
    try {
      // Input validation
      if (!this.securityValidator.validateUser(user)) {
        throw new Error('Invalid user profile');
      }

      const baseRate = this.getPhaseBaseRate(networkStats);
      const finazenBonus = this.calculateFinazenBonus(networkStats);
      const referralBonus = this.calculateReferralBonus(referralNetwork);
      const securityBonus = user.isKYCVerified ? 
        MINING_CONSTANTS.KYC_BONUS : MINING_CONSTANTS.NON_KYC_PENALTY;
      const regressionFactor = this.calculateRegressionFactor(user.totalFINHoldings);
      const humanProbability = Math.max(0.1, user.humanProbabilityScore);

      const hourlyRate = baseRate * finazenBonus * referralBonus * 
                        securityBonus * regressionFactor * humanProbability;

      // Apply daily cap
      const phaseData = MINING_CONSTANTS.PHASES[networkStats.currentPhase];
      const dailyCap = phaseData.maxDaily;
      
      return Math.min(hourlyRate, dailyCap / 24);
    } catch (error) {
      console.error('Mining rate calculation error:', error);
      return 0;
    }
  }

  /**
   * Calculate total user reward including XP and RP bonuses
   */
  public calculateTotalReward(
    user: UserProfile,
    networkStats: NetworkStats,
    referralNetwork: ReferralNetwork,
    activities: Activity[] = []
  ): {
    miningReward: number;
    xpBonus: number;
    rpBonus: number;
    totalReward: number;
    breakdown: any;
  } {
    const baseMiningRate = this.calculateMiningRate(user, networkStats, referralNetwork);
    const xpMultiplier = this.calculateXPLevelMultiplier(user.level);
    const rpMultiplier = this.calculateRPTierMultiplier(this.getRPTier(referralNetwork));
    const stakingBonus = this.calculateStakingBonus(user);
    
    // Quality multiplier based on recent activities
    const qualityMultiplier = activities.length > 0 ? 
      this.calculateAverageQualityScore(activities) : 1.0;

    const miningReward = baseMiningRate * (1 + stakingBonus.miningBoost);
    const xpBonus = baseMiningRate * xpMultiplier * 0.2;
    const rpBonus = baseMiningRate * rpMultiplier * 0.3;
    const qualityBonus = (miningReward + xpBonus + rpBonus) * (qualityMultiplier - 1) * 0.5;

    const totalReward = miningReward + xpBonus + rpBonus + qualityBonus;

    return {
      miningReward,
      xpBonus,
      rpBonus,
      totalReward,
      breakdown: {
        baseMiningRate,
        xpMultiplier,
        rpMultiplier,
        stakingBonus: stakingBonus.miningBoost,
        qualityMultiplier,
        qualityBonus
      }
    };
  }

  // ===== XP CALCULATIONS =====

  /**
   * Calculate XP gained from a specific activity
   */
  public calculateXPGain(
    activity: Activity,
    user: UserProfile,
    dailyActivityCount: Record<ActivityType, number> = {}
  ): number {
    try {
      const baseXP = XP_CONSTANTS.BASE_VALUES[activity.type] || 0;
      
      // Check daily limits
      const dailyLimit = XP_CONSTANTS.DAILY_LIMITS[activity.type];
      const currentCount = dailyActivityCount[activity.type] || 0;
      
      if (currentCount >= dailyLimit) {
        return 0; // Daily limit reached
      }

      const platformMultiplier = XP_CONSTANTS.PLATFORM_MULTIPLIERS[activity.platform] || 1.0;
      const qualityScore = Math.max(0.5, Math.min(2.0, activity.qualityScore));
      const streakBonus = this.calculateStreakBonus(user.streakDays);
      const levelProgression = Math.exp(-XP_CONSTANTS.LEVEL_PROGRESSION_COEFFICIENT * user.level);
      
      // Viral content bonus
      let viralMultiplier = 1.0;
      if (activity.engagement.views >= XP_CONSTANTS.VIRAL_THRESHOLD) {
        viralMultiplier = XP_CONSTANTS.VIRAL_MULTIPLIER;
      }

      // Staking bonus
      const stakingBonus = this.calculateStakingBonus(user);
      const xpStakingMultiplier = 1 + stakingBonus.xpMultiplier;

      const xpGained = baseXP * platformMultiplier * qualityScore * 
                      streakBonus * levelProgression * viralMultiplier * xpStakingMultiplier;

      return Math.round(xpGained);
    } catch (error) {
      console.error('XP calculation error:', error);
      return 0;
    }
  }

  /**
   * Calculate XP level from total XP
   */
  public calculateXPLevel(totalXP: number): number {
    // Exponential level progression: Level = floor(sqrt(totalXP / 100))
    return Math.floor(Math.sqrt(Math.max(0, totalXP) / 100));
  }

  /**
   * Calculate XP required for next level
   */
  public calculateXPForNextLevel(currentLevel: number): number {
    const nextLevel = currentLevel + 1;
    return (nextLevel * nextLevel) * 100;
  }

  /**
   * Calculate mining multiplier based on XP level
   */
  public calculateXPLevelMultiplier(level: number): number {
    // Level tiers from whitepaper
    if (level >= 101) return 5.0;
    if (level >= 76) return 3.3 + (level - 76) * 0.028;
    if (level >= 51) return 2.6 + (level - 51) * 0.028;
    if (level >= 26) return 1.9 + (level - 26) * 0.024;
    if (level >= 11) return 1.3 + (level - 11) * 0.033;
    return 1.0 + level * 0.02;
  }

  // ===== RP CALCULATIONS =====

  /**
   * Calculate total RP value for a user
   */
  public calculateRPValue(referralNetwork: ReferralNetwork): number {
    try {
      const directRP = this.calculateDirectReferralPoints(referralNetwork.directReferrals);
      const indirectRP = this.calculateIndirectNetworkPoints(referralNetwork);
      const qualityBonus = this.calculateNetworkQualityBonus(referralNetwork);
      const regressionFactor = this.calculateNetworkRegressionFactor(referralNetwork);

      const totalRP = (directRP + indirectRP) * qualityBonus * regressionFactor;
      return Math.round(totalRP);
    } catch (error) {
      console.error('RP calculation error:', error);
      return 0;
    }
  }

  /**
   * Get RP tier based on total RP value
   */
  public getRPTier(referralNetwork: ReferralNetwork): RPTier {
    const totalRP = this.calculateRPValue(referralNetwork);
    
    for (const [tier, config] of Object.entries(RP_CONSTANTS.TIERS)) {
      if (totalRP >= config.min && totalRP <= config.max) {
        return tier as RPTier;
      }
    }
    return RPTier.EXPLORER;
  }

  /**
   * Calculate RP tier mining multiplier
   */
  public calculateRPTierMultiplier(tier: RPTier): number {
    return 1 + RP_CONSTANTS.TIERS[tier].miningBonus;
  }

  /**
   * Calculate referral bonus for mining
   */
  public calculateReferralBonus(referralNetwork: ReferralNetwork): number {
    const activeReferrals = Math.min(referralNetwork.activeReferrals, 100); // Cap at 100
    return 1 + (activeReferrals * MINING_CONSTANTS.REFERRAL_BONUS_RATE);
  }

  // ===== STAKING CALCULATIONS =====

  /**
   * Calculate staking bonuses based on staking amount
   */
  public calculateStakingBonus(user: UserProfile): {
    apyRate: number;
    miningBoost: number;
    xpMultiplier: number;
    rpBonus: number;
  } {
    const tier = this.getStakingTier(user.stakingAmount);
    if (!tier) {
      return { apyRate: 0, miningBoost: 0, xpMultiplier: 0, rpBonus: 0 };
    }

    // Apply loyalty bonus based on staking duration
    const loyaltyBonus = Math.min(user.stakingDuration * 0.05, 1.0); // Max 100% bonus at 20 months

    return {
      apyRate: tier.apyRate + loyaltyBonus * 0.02,
      miningBoost: tier.miningBonus + loyaltyBonus * 0.1,
      xpMultiplier: tier.xpMultiplier + loyaltyBonus * 0.05,
      rpBonus: tier.rpBonus + loyaltyBonus * 0.02
    };
  }

  /**
   * Calculate staking rewards
   */
  public calculateStakingRewards(
    stakingAmount: number,
    stakingDurationDays: number,
    user: UserProfile
  ): number {
    const bonus = this.calculateStakingBonus(user);
    const dailyRate = bonus.apyRate / 365;
    return stakingAmount * dailyRate * stakingDurationDays;
  }

  // ===== PRIVATE HELPER METHODS =====

  private getPhaseBaseRate(networkStats: NetworkStats): number {
    return MINING_CONSTANTS.PHASES[networkStats.currentPhase].baseRate;
  }

  private calculateFinazenBonus(networkStats: NetworkStats): number {
    const phaseData = MINING_CONSTANTS.PHASES[networkStats.currentPhase];
    const ratio = networkStats.totalUsers / 1000000; // 1M users normalization
    return Math.max(1.0, phaseData.finazenBonus - ratio);
  }

  private calculateRegressionFactor(totalHoldings: number): number {
    return Math.exp(-MINING_CONSTANTS.REGRESSION_COEFFICIENT * totalHoldings);
  }

  private calculateStreakBonus(streakDays: number): number {
    // Exponential streak bonus with cap
    const bonus = 1 + Math.min(streakDays * 0.05, 2.0);
    return Math.min(bonus, XP_CONSTANTS.STREAK_MAX_MULTIPLIER);
  }

  private calculateDirectReferralPoints(directReferrals: UserProfile[]): number {
    return directReferrals.reduce((total, referral) => {
      const activityScore = this.calculateUserActivityScore(referral);
      const timeDecay = this.calculateTimeDecay(referral.lastActiveDate);
      return total + (activityScore * timeDecay);
    }, 0);
  }

  private calculateIndirectNetworkPoints(referralNetwork: ReferralNetwork): number {
    const l2Points = referralNetwork.level2Referrals.length * 
                    RP_CONSTANTS.L2_MULTIPLIER * 50;
    const l3Points = referralNetwork.level3Referrals.length * 
                    RP_CONSTANTS.L3_MULTIPLIER * 25;
    return l2Points + l3Points;
  }

  private calculateNetworkQualityBonus(referralNetwork: ReferralNetwork): number {
    const diversity = referralNetwork.activeReferrals / Math.max(1, referralNetwork.totalNetworkSize);
    const avgLevel = this.calculateAverageNetworkLevel(referralNetwork);
    const retentionRate = referralNetwork.networkQualityScore;
    
    return diversity * (avgLevel / 10) * retentionRate * 10;
  }

  private calculateNetworkRegressionFactor(referralNetwork: ReferralNetwork): number {
    const factor = RP_CONSTANTS.NETWORK_REGRESSION_COEFFICIENT * 
                   referralNetwork.totalNetworkSize * 
                   referralNetwork.networkQualityScore;
    return Math.exp(-factor);
  }

  private calculateUserActivityScore(user: UserProfile): number {
    const daysSinceRegistration = (Date.now() - user.registrationDate.getTime()) / (1000 * 60 * 60 * 24);
    const activityRatio = user.totalXP / Math.max(1, daysSinceRegistration);
    return Math.min(100, activityRatio);
  }

  private calculateTimeDecay(lastActiveDate: Date): number {
    const daysSinceActive = (Date.now() - lastActiveDate.getTime()) / (1000 * 60 * 60 * 24);
    return Math.exp(-daysSinceActive * 0.01); // 1% decay per day
  }

  private calculateAverageNetworkLevel(referralNetwork: ReferralNetwork): number {
    const allUsers = [
      ...referralNetwork.directReferrals,
      ...referralNetwork.level2Referrals,
      ...referralNetwork.level3Referrals
    ];
    
    if (allUsers.length === 0) return 1;
    
    const totalLevels = allUsers.reduce((sum, user) => sum + user.level, 0);
    return totalLevels / allUsers.length;
  }

  private calculateAverageQualityScore(activities: Activity[]): number {
    if (activities.length === 0) return 1.0;
    
    const totalQuality = activities.reduce((sum, activity) => sum + activity.qualityScore, 0);
    return totalQuality / activities.length;
  }

  private getStakingTier(stakingAmount: number): StakingTier | null {
    return STAKING_TIERS.find(tier => 
      stakingAmount >= tier.minAmount && stakingAmount <= tier.maxAmount
    ) || null;
  }

  // ===== NETWORK EFFECT CALCULATIONS =====

  /**
   * Calculate compound network effect for user rewards
   */
  public calculateNetworkEffect(
    user: UserProfile,
    referralNetwork: ReferralNetwork,
    networkStats: NetworkStats
  ): number {
    const networkSize = referralNetwork.totalNetworkSize;
    const networkQuality = referralNetwork.networkQualityScore;
    const globalNetworkRatio = networkSize / Math.max(1, networkStats.activeUsers);
    
    // Metcalfe's law application: value increases with n²
    const metcalfeValue = Math.sqrt(networkSize) * networkQuality;
    const globalPositioning = Math.log(1 + globalNetworkRatio * 1000) / 10;
    
    return Math.min(2.0, 1 + metcalfeValue * 0.01 + globalPositioning);
  }

  /**
   * Calculate guild participation bonus
   */
  public calculateGuildBonus(
    userGuildContribution: number,
    guildSize: number,
    guildRanking: number
  ): number {
    const contributionRatio = userGuildContribution / Math.max(1, guildSize);
    const rankingBonus = Math.max(0, (100 - guildRanking) / 100);
    
    return 1 + (contributionRatio * 0.3) + (rankingBonus * 0.2);
  }
}

// ===== SECURITY & VALIDATION =====

class SecurityValidator {
  private readonly MAX_SAFE_INTEGER = Number.MAX_SAFE_INTEGER;
  private readonly MIN_HUMAN_SCORE = 0.1;
  private readonly MAX_HOLDINGS = 1000000; // 1M FIN cap for calculations

  validateUser(user: UserProfile): boolean {
    return (
      user.id && 
      user.level >= 0 && 
      user.totalXP >= 0 &&
      user.totalRP >= 0 &&
      user.totalFINHoldings >= 0 &&
      user.totalFINHoldings <= this.MAX_HOLDINGS &&
      user.humanProbabilityScore >= this.MIN_HUMAN_SCORE &&
      user.humanProbabilityScore <= 1.0
    );
  }

  sanitizeNumber(value: number, max: number = this.MAX_SAFE_INTEGER): number {
    if (!Number.isFinite(value) || value < 0) return 0;
    return Math.min(value, max);
  }

  validateActivity(activity: Activity): boolean {
    return (
      activity.type && 
      activity.platform &&
      activity.qualityScore >= 0.5 &&
      activity.qualityScore <= 2.0 &&
      activity.engagement.views >= 0
    );
  }
}

// ===== UTILITY FUNCTIONS =====

export const FinovaUtils = {
  /**
   * Format FIN amount with proper decimals
   */
  formatFINAmount(amount: number, decimals: number = 4): string {
    return amount.toFixed(decimals);
  },

  /**
   * Calculate percentage change
   */
  calculatePercentageChange(oldValue: number, newValue: number): number {
    if (oldValue === 0) return newValue > 0 ? 100 : 0;
    return ((newValue - oldValue) / oldValue) * 100;
  },

  /**
   * Estimate time to next level
   */
  estimateTimeToNextLevel(
    currentXP: number,
    currentLevel: number,
    avgXPPerDay: number
  ): number {
    const engine = FinovaCalculationEngine.getInstance();
    const xpNeeded = engine.calculateXPForNextLevel(currentLevel) - currentXP;
    return avgXPPerDay > 0 ? Math.ceil(xpNeeded / avgXPPerDay) : Infinity;
  },

  /**
   * Get level badge name
   */
  getLevelBadge(level: number): string {
    if (level >= 101) return 'Mythic';
    if (level >= 76) return 'Diamond';
    if (level >= 51) return 'Platinum';
    if (level >= 26) return 'Gold';
    if (level >= 11) return 'Silver';
    return 'Bronze';
  },

  /**
   * Platform-specific optimization
   */
  getPlatformOptimizations(): Record<string, any> {
    const platform = Platform.OS;
    
    return {
      ios: {
        precision: 6,
        useHardwareAcceleration: true,
        cacheSize: 100
      },
      android: {
        precision: 4,
        useHardwareAcceleration: false,
        cacheSize: 50
      },
      web: {
        precision: 8,
        useWebWorkers: true,
        cacheSize: 200
      }
    }[platform] || {};
  }
};

// ===== EXPORT MAIN ENGINE =====

export default FinovaCalculationEngine;
