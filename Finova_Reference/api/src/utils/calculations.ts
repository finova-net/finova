import { Decimal } from 'decimal.js';

// ============================================================================
// TYPES & INTERFACES
// ============================================================================

export interface User {
  id: string;
  totalHoldings: number;
  xpLevel: number;
  rpTier: number;
  rpPoints: number;
  isKYCVerified: boolean;
  streakDays: number;
  currentLevel: number;
  totalNetworkSize: number;
  networkQualityScore: number;
  stakedAmount: number;
  stakingDurationMonths: number;
  dailyActivityScore: number;
  createdAt: Date;
}

export interface Activity {
  type: ActivityType;
  platform: Platform;
  content?: string;
  viewCount?: number;
  engagementRate?: number;
  qualityScore?: number;
  timestamp: Date;
}

export interface ReferralNetwork {
  directReferrals: User[];
  level2Network: User[];
  level3Network: User[];
  totalActiveReferrals: number;
  averageActivityLevel: number;
  retentionRate: number;
}

export enum ActivityType {
  ORIGINAL_POST = 'original_post',
  PHOTO_POST = 'photo_post',
  VIDEO_CONTENT = 'video_content',
  STORY_STATUS = 'story_status',
  MEANINGFUL_COMMENT = 'meaningful_comment',
  LIKE_REACT = 'like_react',
  SHARE_REPOST = 'share_repost',
  FOLLOW_SUBSCRIBE = 'follow_subscribe',
  DAILY_LOGIN = 'daily_login',
  DAILY_QUEST = 'daily_quest',
  MILESTONE = 'milestone',
  VIRAL_CONTENT = 'viral_content'
}

export enum Platform {
  TIKTOK = 'tiktok',
  INSTAGRAM = 'instagram',
  YOUTUBE = 'youtube',
  FACEBOOK = 'facebook',
  TWITTER_X = 'twitter_x',
  APP = 'app'
}

export enum MiningPhase {
  FINIZEN = 'finizen',
  GROWTH = 'growth',
  MATURITY = 'maturity',
  STABILITY = 'stability'
}

export enum RPTier {
  EXPLORER = 0,
  CONNECTOR = 1,
  INFLUENCER = 2,
  LEADER = 3,
  AMBASSADOR = 4
}

// ============================================================================
// CONSTANTS & CONFIGURATION
// ============================================================================

export const MINING_CONFIG = {
  phases: {
    [MiningPhase.FINIZEN]: {
      userRange: [0, 100000],
      baseRate: 0.1,
      finizenBonus: 2.0,
      maxDaily: 4.8
    },
    [MiningPhase.GROWTH]: {
      userRange: [100000, 1000000],
      baseRate: 0.05,
      finizenBonus: 1.5,
      maxDaily: 1.8
    },
    [MiningPhase.MATURITY]: {
      userRange: [1000000, 10000000],
      baseRate: 0.025,
      finizenBonus: 1.2,
      maxDaily: 0.72
    },
    [MiningPhase.STABILITY]: {
      userRange: [10000000, Infinity],
      baseRate: 0.01,
      finizenBonus: 1.0,
      maxDaily: 0.24
    }
  },
  regressionFactor: 0.001,
  kycBonus: 1.2,
  nonKycPenalty: 0.8,
  referralBonusRate: 0.1
};

export const XP_CONFIG = {
  activities: {
    [ActivityType.ORIGINAL_POST]: { baseXP: 50, dailyLimit: null },
    [ActivityType.PHOTO_POST]: { baseXP: 75, dailyLimit: 20 },
    [ActivityType.VIDEO_CONTENT]: { baseXP: 150, dailyLimit: 10 },
    [ActivityType.STORY_STATUS]: { baseXP: 25, dailyLimit: 50 },
    [ActivityType.MEANINGFUL_COMMENT]: { baseXP: 25, dailyLimit: 100 },
    [ActivityType.LIKE_REACT]: { baseXP: 5, dailyLimit: 200 },
    [ActivityType.SHARE_REPOST]: { baseXP: 15, dailyLimit: 50 },
    [ActivityType.FOLLOW_SUBSCRIBE]: { baseXP: 20, dailyLimit: 25 },
    [ActivityType.DAILY_LOGIN]: { baseXP: 10, dailyLimit: 1 },
    [ActivityType.DAILY_QUEST]: { baseXP: 100, dailyLimit: 3 },
    [ActivityType.MILESTONE]: { baseXP: 500, dailyLimit: null },
    [ActivityType.VIRAL_CONTENT]: { baseXP: 1000, dailyLimit: null }
  },
  platformMultipliers: {
    [Platform.TIKTOK]: 1.3,
    [Platform.INSTAGRAM]: 1.2,
    [Platform.YOUTUBE]: 1.4,
    [Platform.FACEBOOK]: 1.1,
    [Platform.TWITTER_X]: 1.2,
    [Platform.APP]: 1.0
  },
  levelProgressionDecay: 0.01,
  maxStreakMultiplier: 3.0,
  qualityRange: [0.5, 2.0]
};

export const RP_CONFIG = {
  tiers: [
    { name: 'Explorer', range: [0, 999], miningBonus: 0, referralBonus: 0.1, networkCap: 10 },
    { name: 'Connector', range: [1000, 4999], miningBonus: 0.2, referralBonus: 0.15, networkCap: 25 },
    { name: 'Influencer', range: [5000, 14999], miningBonus: 0.5, referralBonus: 0.2, networkCap: 50 },
    { name: 'Leader', range: [15000, 49999], miningBonus: 1.0, referralBonus: 0.25, networkCap: 100 },
    { name: 'Ambassador', range: [50000, Infinity], miningBonus: 2.0, referralBonus: 0.3, networkCap: Infinity }
  ],
  networkRegressionFactor: 0.0001,
  level2Multiplier: 0.3,
  level3Multiplier: 0.1
};

export const STAKING_CONFIG = {
  tiers: [
    { range: [100, 499], apy: 0.08, miningBoost: 0.2, xpMultiplier: 0.1, rpBonus: 0.05 },
    { range: [500, 999], apy: 0.10, miningBoost: 0.35, xpMultiplier: 0.2, rpBonus: 0.1 },
    { range: [1000, 4999], apy: 0.12, miningBoost: 0.5, xpMultiplier: 0.3, rpBonus: 0.2 },
    { range: [5000, 9999], apy: 0.14, miningBoost: 0.75, xpMultiplier: 0.5, rpBonus: 0.35 },
    { range: [10000, Infinity], apy: 0.15, miningBonus: 1.0, xpMultiplier: 0.75, rpBonus: 0.5 }
  ],
  loyaltyBonusRate: 0.05,
  activityBonusRate: 0.1
};

// ============================================================================
// CORE MINING CALCULATIONS
// ============================================================================

export class MiningCalculations {
  /**
   * Calculate the current mining phase based on total users
   */
  static getCurrentPhase(totalUsers: number): MiningPhase {
    if (totalUsers < 100000) return MiningPhase.FINIZEN;
    if (totalUsers < 1000000) return MiningPhase.GROWTH;
    if (totalUsers < 10000000) return MiningPhase.MATURITY;
    return MiningPhase.STABILITY;
  }

  /**
   * Core mining rate calculation with exponential regression
   */
  static calculateMiningRate(user: User, totalUsers: number): number {
    const phase = this.getCurrentPhase(totalUsers);
    const config = MINING_CONFIG.phases[phase];
    
    // Base components
    const baseRate = new Decimal(config.baseRate);
    const finizenBonus = new Decimal(config.finizenBonus);
    const referralBonus = new Decimal(1).plus(
      new Decimal(this.getActiveReferrals(user)).times(MINING_CONFIG.referralBonusRate)
    );
    const securityBonus = new Decimal(user.isKYCVerified ? MINING_CONFIG.kycBonus : MINING_CONFIG.nonKycPenalty);
    
    // Exponential regression factor to prevent whale dominance
    const regressionFactor = Decimal.exp(
      new Decimal(-MINING_CONFIG.regressionFactor).times(user.totalHoldings)
    );
    
    // Final calculation
    const hourlyRate = baseRate
      .times(finizenBonus)
      .times(referralBonus)
      .times(securityBonus)
      .times(regressionFactor);
    
    // Apply daily cap
    const dailyRate = hourlyRate.times(24);
    const cappedDaily = Decimal.min(dailyRate, config.maxDaily);
    
    return cappedDaily.div(24).toNumber();
  }

  /**
   * Calculate total user reward integrating XP, RP, and mining
   */
  static calculateTotalReward(user: User, activity: Activity, totalUsers: number): {
    miningReward: number;
    xpBonus: number;
    rpBonus: number;
    qualityMultiplier: number;
    totalReward: number;
  } {
    const baseMining = this.calculateMiningRate(user, totalUsers);
    const xpMultiplier = XPCalculations.getLevelMultiplier(user.xpLevel);
    const rpMultiplier = RPCalculations.getTierMultiplier(user.rpTier);
    const qualityScore = activity.qualityScore || 1.0;
    
    const miningReward = baseMining;
    const xpBonus = baseMining * xpMultiplier * 0.2;
    const rpBonus = baseMining * rpMultiplier * 0.3;
    const qualityMultiplier = qualityScore * 0.5;
    
    const totalReward = (miningReward + xpBonus + rpBonus) * (1 + qualityMultiplier);
    
    return {
      miningReward,
      xpBonus,
      rpBonus,
      qualityMultiplier,
      totalReward
    };
  }

  private static getActiveReferrals(user: User): number {
    // This would typically query the database for active referrals
    // For now, we'll use a simplified calculation
    return Math.min(user.totalNetworkSize * user.networkQualityScore, 50);
  }
}

// ============================================================================
// XP SYSTEM CALCULATIONS
// ============================================================================

export class XPCalculations {
  /**
   * Calculate XP gained from an activity
   */
  static calculateXPGain(activity: Activity, user: User): number {
    const config = XP_CONFIG.activities[activity.type];
    if (!config) return 0;
    
    const baseXP = new Decimal(config.baseXP);
    const platformMultiplier = new Decimal(XP_CONFIG.platformMultipliers[activity.platform] || 1.0);
    const qualityScore = new Decimal(Math.max(0.5, Math.min(2.0, activity.qualityScore || 1.0)));
    const streakBonus = this.calculateStreakBonus(user.streakDays);
    const levelProgression = Decimal.exp(
      new Decimal(-XP_CONFIG.levelProgressionDecay).times(user.currentLevel)
    );
    
    // Special handling for viral content
    if (activity.type === ActivityType.VIRAL_CONTENT && activity.viewCount) {
      const viralMultiplier = Math.min(2.0, 1.0 + Math.log10(activity.viewCount / 1000));
      return baseXP
        .times(platformMultiplier)
        .times(viralMultiplier)
        .times(streakBonus)
        .times(levelProgression)
        .toNumber();
    }
    
    return baseXP
      .times(platformMultiplier)
      .times(qualityScore)
      .times(streakBonus)
      .times(levelProgression)
      .toNumber();
  }

  /**
   * Calculate streak bonus multiplier
   */
  static calculateStreakBonus(streakDays: number): Decimal {
    const maxStreak = 30;
    const normalizedStreak = Math.min(streakDays, maxStreak) / maxStreak;
    const streakMultiplier = 1 + (normalizedStreak * (XP_CONFIG.maxStreakMultiplier - 1));
    return new Decimal(streakMultiplier);
  }

  /**
   * Get mining multiplier based on XP level
   */
  static getLevelMultiplier(xpLevel: number): number {
    if (xpLevel <= 10) return 1.0 + (xpLevel * 0.02); // 1.0x - 1.2x
    if (xpLevel <= 25) return 1.3 + ((xpLevel - 10) * 0.033); // 1.3x - 1.8x
    if (xpLevel <= 50) return 1.9 + ((xpLevel - 25) * 0.024); // 1.9x - 2.5x
    if (xpLevel <= 75) return 2.6 + ((xpLevel - 50) * 0.024); // 2.6x - 3.2x
    if (xpLevel <= 100) return 3.3 + ((xpLevel - 75) * 0.028); // 3.3x - 4.0x
    return 4.1 + Math.min((xpLevel - 100) * 0.018, 0.9); // 4.1x - 5.0x
  }

  /**
   * Calculate required XP for a given level
   */
  static getRequiredXP(level: number): number {
    if (level <= 10) return level * 100;
    if (level <= 25) return 1000 + (level - 10) * 266;
    if (level <= 50) return 5000 + (level - 25) * 600;
    if (level <= 75) return 20000 + (level - 50) * 1200;
    if (level <= 100) return 50000 + (level - 75) * 2000;
    return 100000 + (level - 100) * 5000;
  }

  /**
   * Get current level from total XP
   */
  static getLevelFromXP(totalXP: number): number {
    let level = 1;
    while (this.getRequiredXP(level + 1) <= totalXP) {
      level++;
    }
    return level;
  }
}

// ============================================================================
// REFERRAL POINTS (RP) CALCULATIONS
// ============================================================================

export class RPCalculations {
  /**
   * Calculate total RP value from network
   */
  static calculateRPValue(user: User, referralNetwork: ReferralNetwork): number {
    const directRP = this.calculateDirectReferralPoints(referralNetwork);
    const indirectRP = this.calculateIndirectNetworkPoints(referralNetwork);
    const qualityBonus = this.calculateNetworkQualityBonus(referralNetwork);
    const regressionFactor = this.calculateNetworkRegression(user);
    
    return (directRP + indirectRP) * qualityBonus * regressionFactor;
  }

  /**
   * Calculate direct referral points
   */
  private static calculateDirectReferralPoints(network: ReferralNetwork): number {
    return network.directReferrals.reduce((total, referral) => {
      const activity = referral.dailyActivityScore || 50; // Default activity score
      const level = referral.currentLevel || 1;
      const timeDecay = this.calculateTimeDecay(referral.createdAt);
      
      return total + (activity * level * timeDecay);
    }, 0);
  }

  /**
   * Calculate indirect network points (L2 and L3)
   */
  private static calculateIndirectNetworkPoints(network: ReferralNetwork): number {
    const l2Points = network.level2Network.length * network.averageActivityLevel * RP_CONFIG.level2Multiplier;
    const l3Points = network.level3Network.length * network.averageActivityLevel * RP_CONFIG.level3Multiplier;
    
    return l2Points + l3Points;
  }

  /**
   * Calculate network quality bonus
   */
  private static calculateNetworkQualityBonus(network: ReferralNetwork): number {
    const networkDiversity = this.calculateNetworkDiversity(network);
    const averageLevel = this.calculateAverageLevel(network);
    const retentionRate = network.retentionRate;
    
    return networkDiversity * averageLevel * retentionRate;
  }

  /**
   * Calculate network regression factor
   */
  private static calculateNetworkRegression(user: User): number {
    const regressionExponent = -RP_CONFIG.networkRegressionFactor * 
      user.totalNetworkSize * user.networkQualityScore;
    
    return Math.exp(regressionExponent);
  }

  /**
   * Get RP tier from total RP points
   */
  static getRPTier(rpPoints: number): RPTier {
    for (let i = RP_CONFIG.tiers.length - 1; i >= 0; i--) {
      const tier = RP_CONFIG.tiers[i];
      if (rpPoints >= tier.range[0]) {
        return i as RPTier;
      }
    }
    return RPTier.EXPLORER;
  }

  /**
   * Get tier multiplier for mining
   */
  static getTierMultiplier(tier: RPTier): number {
    return 1 + RP_CONFIG.tiers[tier].miningBonus;
  }

  /**
   * Get referral bonus percentage
   */
  static getReferralBonus(tier: RPTier): number {
    return RP_CONFIG.tiers[tier].referralBonus;
  }

  private static calculateTimeDecay(createdAt: Date): number {
    const daysSinceCreation = (Date.now() - createdAt.getTime()) / (1000 * 60 * 60 * 24);
    return Math.max(0.1, Math.exp(-daysSinceCreation / 365)); // Decay over year
  }

  private static calculateNetworkDiversity(network: ReferralNetwork): number {
    // Simplified diversity calculation - could be enhanced with geographic/platform diversity
    const totalUsers = network.directReferrals.length + network.level2Network.length + network.level3Network.length;
    return Math.min(2.0, 1.0 + Math.log10(totalUsers + 1));
  }

  private static calculateAverageLevel(network: ReferralNetwork): number {
    const allUsers = [...network.directReferrals, ...network.level2Network, ...network.level3Network];
    const totalLevels = allUsers.reduce((sum, user) => sum + (user.currentLevel || 1), 0);
    return totalLevels / (allUsers.length || 1);
  }
}

// ============================================================================
// STAKING CALCULATIONS
// ============================================================================

export class StakingCalculations {
  /**
   * Calculate staking rewards with integrated bonuses
   */
  static calculateStakingReward(user: User, totalStaked: number, poolRewards: number): number {
    const stakingTier = this.getStakingTier(user.stakedAmount);
    const baseReward = (user.stakedAmount / totalStaked) * poolRewards;
    
    const multiplierEffects = this.calculateMultiplierEffects(user, stakingTier);
    
    return baseReward * multiplierEffects;
  }

  /**
   * Calculate all multiplier effects
   */
  private static calculateMultiplierEffects(user: User, tier: any): number {
    const xpLevelBonus = 1.0 + (user.xpLevel / 100);
    const rpTierBonus = 1.0 + (user.rpTier * 0.2);
    const loyaltyBonus = 1.0 + (user.stakingDurationMonths * STAKING_CONFIG.loyaltyBonusRate);
    const activityBonus = 1.0 + (user.dailyActivityScore * STAKING_CONFIG.activityBonusRate);
    
    return xpLevelBonus * rpTierBonus * loyaltyBonus * activityBonus;
  }

  /**
   * Get staking tier configuration
   */
  static getStakingTier(stakedAmount: number) {
    return STAKING_CONFIG.tiers.find(tier => 
      stakedAmount >= tier.range[0] && stakedAmount <= tier.range[1]
    ) || STAKING_CONFIG.tiers[0];
  }

  /**
   * Calculate APY for staked amount
   */
  static calculateAPY(stakedAmount: number, user: User): number {
    const tier = this.getStakingTier(stakedAmount);
    const multipliers = this.calculateMultiplierEffects(user, tier);
    
    return tier.apy * multipliers;
  }
}

// ============================================================================
// ANTI-BOT & QUALITY CALCULATIONS
// ============================================================================

export class QualityCalculations {
  /**
   * Calculate human probability score
   */
  static calculateHumanProbability(userBehaviorData: any): number {
    const factors = {
      biometricConsistency: this.analyzeBiometricPatterns(userBehaviorData),
      behavioralPatterns: this.detectHumanRhythms(userBehaviorData),
      socialGraphValidity: this.validateRealConnections(userBehaviorData),
      deviceAuthenticity: this.checkDeviceFingerprint(userBehaviorData),
      interactionQuality: this.measureContentUniqueness(userBehaviorData)
    };

    const weights = {
      biometricConsistency: 0.3,
      behavioralPatterns: 0.25,
      socialGraphValidity: 0.2,
      deviceAuthenticity: 0.15,
      interactionQuality: 0.1
    };

    let weightedScore = 0;
    for (const [factor, score] of Object.entries(factors)) {
      weightedScore += score * weights[factor as keyof typeof weights];
    }

    return Math.max(0.1, Math.min(1.0, weightedScore));
  }

  /**
   * Calculate difficulty multiplier for progressive scaling
   */
  static calculateDifficultyMultiplier(totalEarnedFIN: number, suspiciousScore: number): number {
    return 1 + (totalEarnedFIN / 1000) + (suspiciousScore * 2);
  }

  /**
   * Apply penalties based on difficulty multiplier
   */
  static applyPenalties(baseRate: number, baseXP: number, baseRP: number, difficultyMultiplier: number) {
    const miningPenalty = baseRate * (1 - difficultyMultiplier * 0.1);
    const xpPenalty = baseXP * (1 - difficultyMultiplier * 0.05);
    const rpPenalty = baseRP * (1 - difficultyMultiplier * 0.08);
    
    return {
      adjustedMining: Math.max(0, miningPenalty),
      adjustedXP: Math.max(0, xpPenalty),
      adjustedRP: Math.max(0, rpPenalty)
    };
  }

  private static analyzeBiometricPatterns(data: any): number {
    // Simplified biometric analysis - in production would use ML models
    return Math.random() * 0.4 + 0.6; // Mock score 0.6-1.0
  }

  private static detectHumanRhythms(data: any): number {
    // Analyze timing patterns for human-like behavior
    return Math.random() * 0.4 + 0.6; // Mock score 0.6-1.0
  }

  private static validateRealConnections(data: any): number {
    // Check social graph authenticity
    return Math.random() * 0.4 + 0.6; // Mock score 0.6-1.0
  }

  private static checkDeviceFingerprint(data: any): number {
    // Device authenticity verification
    return Math.random() * 0.4 + 0.6; // Mock score 0.6-1.0
  }

  private static measureContentUniqueness(data: any): number {
    // Content originality analysis
    return Math.random() * 0.4 + 0.6; // Mock score 0.6-1.0
  }
}

// ============================================================================
// UTILITY FUNCTIONS
// ============================================================================

export class CalculationUtils {
  /**
   * Safe decimal calculation wrapper
   */
  static safeCalculate(operation: () => Decimal): number {
    try {
      const result = operation();
      return result.isFinite() ? result.toNumber() : 0;
    } catch (error) {
      console.error('Calculation error:', error);
      return 0;
    }
  }

  /**
   * Clamp value between min and max
   */
  static clamp(value: number, min: number, max: number): number {
    return Math.max(min, Math.min(max, value));
  }

  /**
   * Calculate percentage change
   */
  static percentageChange(oldValue: number, newValue: number): number {
    if (oldValue === 0) return newValue > 0 ? 100 : 0;
    return ((newValue - oldValue) / oldValue) * 100;
  }

  /**
   * Round to specified decimal places
   */
  static roundTo(value: number, decimals: number): number {
    const factor = Math.pow(10, decimals);
    return Math.round(value * factor) / factor;
  }

  /**
   * Calculate compound interest
   */
  static compoundInterest(principal: number, rate: number, time: number, compoundFreq: number = 365): number {
    return principal * Math.pow(1 + rate / compoundFreq, compoundFreq * time);
  }
}

// ============================================================================
// VALIDATION FUNCTIONS
// ============================================================================

export class ValidationUtils {
  /**
   * Validate user data for calculations
   */
  static validateUser(user: User): boolean {
    return (
      user.id && 
      typeof user.totalHoldings === 'number' && user.totalHoldings >= 0 &&
      typeof user.xpLevel === 'number' && user.xpLevel >= 0 &&
      typeof user.rpTier === 'number' && user.rpTier >= 0 &&
      typeof user.isKYCVerified === 'boolean'
    );
  }

  /**
   * Validate activity data
   */
  static validateActivity(activity: Activity): boolean {
    return (
      Object.values(ActivityType).includes(activity.type) &&
      Object.values(Platform).includes(activity.platform) &&
      activity.timestamp instanceof Date
    );
  }

  /**
   * Check daily limits for activities
   */
  static checkDailyLimit(activityType: ActivityType, todayCount: number): boolean {
    const config = XP_CONFIG.activities[activityType];
    return !config.dailyLimit || todayCount < config.dailyLimit;
  }
}

export default {
  MiningCalculations,
  XPCalculations,
  RPCalculations,
  StakingCalculations,
  QualityCalculations,
  CalculationUtils,
  ValidationUtils,
  MINING_CONFIG,
  XP_CONFIG,
  RP_CONFIG,
  STAKING_CONFIG
};
