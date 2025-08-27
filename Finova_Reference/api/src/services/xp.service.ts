import { Injectable, Logger } from '@nestjs/common';
import { InjectRepository } from '@nestjs/typeorm';
import { Repository, EntityManager } from 'typeorm';
import { Redis } from 'ioredis';
import { InjectRedis } from '@liaoliaots/nestjs-redis';
import { User } from '../models/User.model';
import { XP } from '../models/XP.model';
import { Mining } from '../models/Mining.model';
import { AIQualityService } from './ai-quality.service';
import { AntiBotService } from './anti-bot.service';
import { NotificationService } from './notification.service';
import { AnalyticsService } from './analytics.service';
import { 
  XPActivity, 
  XPCalculationResult, 
  PlatformType, 
  ContentQuality,
  XPStreakBonus,
  XPLevelTier,
  XPMultipliers 
} from '../types/xp.types';

@Injectable()
export class XPService {
  private readonly logger = new Logger(XPService.name);
  
  // XP Configuration Constants
  private readonly XP_CONFIG = {
    // Base XP values per activity type
    BASE_XP: {
      ORIGINAL_POST: 50,
      PHOTO_POST: 75,
      VIDEO_POST: 150,
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
    
    // Platform multipliers
    PLATFORM_MULTIPLIERS: {
      TIKTOK: 1.3,
      INSTAGRAM: 1.2,
      YOUTUBE: 1.4,
      FACEBOOK: 1.1,
      X_TWITTER: 1.2,
      DEFAULT: 1.0
    },
    
    // Daily limits per activity
    DAILY_LIMITS: {
      ORIGINAL_POST: null, // No limit
      PHOTO_POST: 20,
      VIDEO_POST: 10,
      STORY_STATUS: 50,
      MEANINGFUL_COMMENT: 100,
      LIKE_REACT: 200,
      SHARE_REPOST: 50,
      FOLLOW_SUBSCRIBE: 25,
      DAILY_LOGIN: 1,
      DAILY_QUEST: 3
    },
    
    // Level progression parameters
    LEVEL_PROGRESSION: {
      REGRESSION_FACTOR: 0.01, // e^(-0.01 * level)
      MAX_LEVEL: 200,
      XP_REQUIRED_BASE: 100,
      XP_MULTIPLIER: 1.5
    },
    
    // Mining multipliers per level tier
    MINING_MULTIPLIERS: {
      BRONZE: { min: 1, max: 10, multiplier: 1.0, bonus: 0.02 },
      SILVER: { min: 11, max: 25, multiplier: 1.3, bonus: 0.05 },
      GOLD: { min: 26, max: 50, multiplier: 1.9, bonus: 0.1 },
      PLATINUM: { min: 51, max: 75, multiplier: 2.6, bonus: 0.15 },
      DIAMOND: { min: 76, max: 100, multiplier: 3.3, bonus: 0.2 },
      MYTHIC: { min: 101, max: 200, multiplier: 4.1, bonus: 0.25 }
    },
    
    // Quality score ranges
    QUALITY_RANGES: {
      MIN: 0.5,
      MAX: 2.0,
      DEFAULT: 1.0
    },
    
    // Streak bonus configuration
    STREAK_CONFIG: {
      MAX_BONUS: 3.0,
      PROGRESSION: 0.1, // +10% per day
      DECAY_HOURS: 48 // Streak breaks after 48 hours inactive
    }
  };

  constructor(
    @InjectRepository(User)
    private readonly userRepository: Repository<User>,
    @InjectRepository(XP)
    private readonly xpRepository: Repository<XP>,
    @InjectRepository(Mining)
    private readonly miningRepository: Repository<Mining>,
    @InjectRedis()
    private readonly redis: Redis,
    private readonly aiQualityService: AIQualityService,
    private readonly antiBotService: AntiBotService,
    private readonly notificationService: NotificationService,
    private readonly analyticsService: AnalyticsService,
  ) {}

  /**
   * Calculate and award XP for user activity
   */
  async calculateAndAwardXP(
    userId: string,
    activity: XPActivity,
    manager?: EntityManager
  ): Promise<XPCalculationResult> {
    const transactionManager = manager || this.userRepository.manager;
    
    try {
      // Get user with current XP data
      const user = await this.getUserWithXPData(userId, transactionManager);
      if (!user) {
        throw new Error(`User ${userId} not found`);
      }

      // Check daily limits
      await this.checkDailyLimits(user, activity);

      // Perform anti-bot validation
      const humanProbability = await this.antiBotService.validateActivity(userId, activity);
      if (humanProbability < 0.7) {
        this.logger.warn(`Low human probability ${humanProbability} for user ${userId}`);
        return { success: false, reason: 'Bot detection triggered', xpGained: 0 };
      }

      // Calculate XP components
      const calculation = await this.performXPCalculation(user, activity);
      
      // Apply final validations
      if (calculation.totalXP <= 0) {
        return { success: false, reason: 'Invalid XP calculation', xpGained: 0 };
      }

      // Award XP and update user
      const result = await this.awardXP(user, calculation, activity, transactionManager);
      
      // Send notifications for level ups
      if (result.levelUp) {
        await this.handleLevelUp(user, result);
      }

      // Track analytics
      await this.analyticsService.trackXPGain(userId, activity, result);

      return result;
      
    } catch (error) {
      this.logger.error(`Error calculating XP for user ${userId}:`, error);
      throw error;
    }
  }

  /**
   * Core XP calculation logic implementing the whitepaper formula
   */
  private async performXPCalculation(
    user: User, 
    activity: XPActivity
  ): Promise<XPMultipliers> {
    // Base XP from activity type
    const baseXP = this.getBaseXP(activity.type);

    // Platform multiplier
    const platformMultiplier = this.getPlatformMultiplier(activity.platform);

    // Quality score from AI analysis
    const qualityScore = await this.getQualityScore(activity);

    // Streak bonus
    const streakBonus = this.calculateStreakBonus(user);

    // Level progression regression (anti-whale mechanism)
    const levelProgression = this.calculateLevelProgression(user.currentLevel);

    // Special event multipliers
    const eventMultiplier = await this.getActiveEventMultipliers(user.id);

    // Calculate total XP: Base_XP × Platform_Multiplier × Quality_Score × Streak_Bonus × Level_Progression
    const totalXP = Math.floor(
      baseXP * 
      platformMultiplier * 
      qualityScore * 
      streakBonus * 
      levelProgression * 
      eventMultiplier
    );

    return {
      baseXP,
      platformMultiplier,
      qualityScore,
      streakBonus,
      levelProgression,
      eventMultiplier,
      totalXP,
      humanProbability: await this.antiBotService.getHumanProbability(user.id)
    };
  }

  /**
   * Award calculated XP and handle level progression
   */
  private async awardXP(
    user: User,
    calculation: XPMultipliers,
    activity: XPActivity,
    manager: EntityManager
  ): Promise<XPCalculationResult> {
    const oldLevel = user.currentLevel;
    const oldXP = user.totalXP;
    const newXP = oldXP + calculation.totalXP;

    // Calculate new level
    const newLevel = this.calculateLevelFromXP(newXP);
    const levelUp = newLevel > oldLevel;

    // Update user XP data
    await manager.update(User, user.id, {
      totalXP: newXP,
      currentLevel: newLevel,
      lastActivityAt: new Date(),
      streakDays: this.calculateNewStreak(user)
    });

    // Record XP transaction
    const xpRecord = manager.create(XP, {
      userId: user.id,
      activityType: activity.type,
      platform: activity.platform,
      baseXP: calculation.baseXP,
      multipliers: {
        platform: calculation.platformMultiplier,
        quality: calculation.qualityScore,
        streak: calculation.streakBonus,
        level: calculation.levelProgression,
        event: calculation.eventMultiplier
      },
      totalXPGained: calculation.totalXP,
      contentId: activity.contentId,
      metadata: activity.metadata
    });

    await manager.save(XP, xpRecord);

    // Update Redis cache
    await this.updateXPCache(user.id, newXP, newLevel);

    // Update mining multiplier if level changed
    if (levelUp) {
      await this.updateMiningMultiplier(user.id, newLevel, manager);
    }

    return {
      success: true,
      xpGained: calculation.totalXP,
      totalXP: newXP,
      oldLevel,
      newLevel,
      levelUp,
      miningMultiplier: this.getMiningMultiplierForLevel(newLevel),
      calculation
    };
  }

  /**
   * Calculate level progression regression factor
   */
  private calculateLevelProgression(currentLevel: number): number {
    return Math.exp(-this.XP_CONFIG.LEVEL_PROGRESSION.REGRESSION_FACTOR * currentLevel);
  }

  /**
   * Calculate streak bonus
   */
  private calculateStreakBonus(user: User): number {
    const streakDays = user.streakDays || 0;
    const bonus = 1 + (streakDays * this.XP_CONFIG.STREAK_CONFIG.PROGRESSION);
    return Math.min(bonus, this.XP_CONFIG.STREAK_CONFIG.MAX_BONUS);
  }

  /**
   * Get quality score from AI analysis
   */
  private async getQualityScore(activity: XPActivity): Promise<number> {
    try {
      if (!activity.content) {
        return this.XP_CONFIG.QUALITY_RANGES.DEFAULT;
      }

      const qualityAnalysis = await this.aiQualityService.analyzeContent({
        type: activity.type,
        content: activity.content,
        platform: activity.platform,
        metadata: activity.metadata
      });

      // Clamp quality score to valid range
      return Math.max(
        this.XP_CONFIG.QUALITY_RANGES.MIN,
        Math.min(this.XP_CONFIG.QUALITY_RANGES.MAX, qualityAnalysis.overallScore)
      );
    } catch (error) {
      this.logger.warn(`Quality analysis failed for activity: ${error.message}`);
      return this.XP_CONFIG.QUALITY_RANGES.DEFAULT;
    }
  }

  /**
   * Calculate level from total XP using exponential formula
   */
  private calculateLevelFromXP(totalXP: number): number {
    const base = this.XP_CONFIG.LEVEL_PROGRESSION.XP_REQUIRED_BASE;
    const multiplier = this.XP_CONFIG.LEVEL_PROGRESSION.XP_MULTIPLIER;
    
    // Formula: Level = floor(log(totalXP / base + 1) / log(multiplier))
    const level = Math.floor(Math.log(totalXP / base + 1) / Math.log(multiplier));
    return Math.min(level, this.XP_CONFIG.LEVEL_PROGRESSION.MAX_LEVEL);
  }

  /**
   * Get mining multiplier for specific level
   */
  private getMiningMultiplierForLevel(level: number): number {
    const tier = this.getLevelTier(level);
    const tierConfig = this.XP_CONFIG.MINING_MULTIPLIERS[tier];
    
    // Progressive multiplier within tier
    const tierProgress = (level - tierConfig.min) / (tierConfig.max - tierConfig.min);
    return tierConfig.multiplier + (tierProgress * tierConfig.bonus);
  }

  /**
   * Get level tier (Bronze, Silver, Gold, etc.)
   */
  private getLevelTier(level: number): XPLevelTier {
    for (const [tier, config] of Object.entries(this.XP_CONFIG.MINING_MULTIPLIERS)) {
      if (level >= config.min && level <= config.max) {
        return tier as XPLevelTier;
      }
    }
    return 'MYTHIC'; // Default to highest tier
  }

  /**
   * Check daily activity limits
   */
  private async checkDailyLimits(user: User, activity: XPActivity): Promise<void> {
    const limit = this.XP_CONFIG.DAILY_LIMITS[activity.type];
    if (!limit) return; // No limit for this activity

    const today = new Date().toISOString().split('T')[0];
    const cacheKey = `xp_daily_${user.id}_${activity.type}_${today}`;
    
    const currentCount = await this.redis.get(cacheKey);
    const count = parseInt(currentCount || '0');

    if (count >= limit) {
      throw new Error(`Daily limit reached for ${activity.type}: ${limit}`);
    }

    // Increment counter
    await this.redis.incr(cacheKey);
    await this.redis.expire(cacheKey, 86400); // 24 hours
  }

  /**
   * Get base XP for activity type
   */
  private getBaseXP(activityType: string): number {
    return this.XP_CONFIG.BASE_XP[activityType] || 0;
  }

  /**
   * Get platform multiplier
   */
  private getPlatformMultiplier(platform: PlatformType): number {
    return this.XP_CONFIG.PLATFORM_MULTIPLIERS[platform] || 
           this.XP_CONFIG.PLATFORM_MULTIPLIERS.DEFAULT;
  }

  /**
   * Get user with XP data
   */
  private async getUserWithXPData(
    userId: string, 
    manager: EntityManager
  ): Promise<User | null> {
    return manager.findOne(User, {
      where: { id: userId },
      select: [
        'id', 'totalXP', 'currentLevel', 'streakDays', 
        'lastActivityAt', 'isKYCVerified', 'createdAt'
      ]
    });
  }

  /**
   * Update XP cache in Redis
   */
  private async updateXPCache(userId: string, totalXP: number, level: number): Promise<void> {
    const cacheKey = `user_xp_${userId}`;
    await this.redis.hset(cacheKey, {
      totalXP: totalXP.toString(),
      level: level.toString(),
      updatedAt: Date.now().toString()
    });
    await this.redis.expire(cacheKey, 3600); // 1 hour cache
  }

  /**
   * Update mining multiplier when level changes
   */
  private async updateMiningMultiplier(
    userId: string, 
    newLevel: number, 
    manager: EntityManager
  ): Promise<void> {
    const multiplier = this.getMiningMultiplierForLevel(newLevel);
    
    await manager.update(Mining, 
      { userId }, 
      { 
        xpMultiplier: multiplier,
        lastXPUpdate: new Date()
      }
    );
  }

  /**
   * Handle level up notifications and rewards
   */
  private async handleLevelUp(user: User, result: XPCalculationResult): Promise<void> {
    const tier = this.getLevelTier(result.newLevel);
    const oldTier = this.getLevelTier(result.oldLevel);
    
    // Send level up notification
    await this.notificationService.sendLevelUpNotification(user.id, {
      oldLevel: result.oldLevel,
      newLevel: result.newLevel,
      oldTier,
      newTier: tier,
      miningMultiplier: result.miningMultiplier
    });

    // Award tier upgrade rewards if tier changed
    if (tier !== oldTier) {
      await this.awardTierUpgradeRewards(user.id, tier);
    }
  }

  /**
   * Award tier upgrade rewards
   */
  private async awardTierUpgradeRewards(userId: string, tier: XPLevelTier): Promise<void> {
    // Implementation for tier rewards (NFTs, special cards, etc.)
    this.logger.log(`User ${userId} upgraded to ${tier} tier`);
  }

  /**
   * Calculate new streak days
   */
  private calculateNewStreak(user: User): number {
    const now = new Date();
    const lastActivity = user.lastActivityAt;
    
    if (!lastActivity) return 1; // First activity
    
    const hoursDiff = (now.getTime() - lastActivity.getTime()) / (1000 * 60 * 60);
    
    if (hoursDiff > this.XP_CONFIG.STREAK_CONFIG.DECAY_HOURS) {
      return 1; // Streak broken
    }
    
    // Check if it's a new day
    const lastDay = lastActivity.toDateString();
    const currentDay = now.toDateString();
    
    return lastDay === currentDay ? user.streakDays : (user.streakDays || 0) + 1;
  }

  /**
   * Get active event multipliers
   */
  private async getActiveEventMultipliers(userId: string): Promise<number> {
    try {
      const cacheKey = `event_multipliers_${userId}`;
      const multipliers = await this.redis.get(cacheKey);
      return multipliers ? parseFloat(multipliers) : 1.0;
    } catch {
      return 1.0;
    }
  }

  /**
   * Get user XP statistics
   */
  async getUserXPStats(userId: string): Promise<any> {
    const user = await this.userRepository.findOne({
      where: { id: userId },
      select: ['totalXP', 'currentLevel', 'streakDays', 'lastActivityAt']
    });

    if (!user) {
      throw new Error(`User ${userId} not found`);
    }

    const tier = this.getLevelTier(user.currentLevel);
    const miningMultiplier = this.getMiningMultiplierForLevel(user.currentLevel);
    const nextLevelXP = this.calculateXPRequiredForLevel(user.currentLevel + 1);

    return {
      totalXP: user.totalXP,
      currentLevel: user.currentLevel,
      tier,
      streakDays: user.streakDays,
      miningMultiplier,
      xpToNextLevel: nextLevelXP - user.totalXP,
      lastActivity: user.lastActivityAt
    };
  }

  /**
   * Calculate XP required for specific level
   */
  private calculateXPRequiredForLevel(level: number): number {
    const base = this.XP_CONFIG.LEVEL_PROGRESSION.XP_REQUIRED_BASE;
    const multiplier = this.XP_CONFIG.LEVEL_PROGRESSION.XP_MULTIPLIER;
    
    return Math.floor(base * (Math.pow(multiplier, level) - 1));
  }

  /**
   * Get leaderboard data
   */
  async getXPLeaderboard(limit: number = 100, offset: number = 0): Promise<any[]> {
    return this.userRepository.find({
      select: ['id', 'username', 'totalXP', 'currentLevel'],
      order: { totalXP: 'DESC' },
      take: limit,
      skip: offset
    });
  }
}
