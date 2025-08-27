import { Request, Response } from 'express';
import { body, param, query, validationResult } from 'express-validator';
import { XPService } from '../services/xp.service';
import { MiningService } from '../services/mining.service';
import { ReferralService } from '../services/referral.service';
import { AIQualityService } from '../services/ai-quality.service';
import { AntiBotService } from '../services/anti-bot.service';
import { BlockchainService } from '../services/blockchain.service';
import { NotificationService } from '../services/notification.service';
import { AnalyticsService } from '../services/analytics.service';
import { logger } from '../utils/logger';
import { ApiResponse, XPActivity, XPLevel, XPStats, PlatformType } from '../types/api.types';
import { RateLimiter } from '../middleware/rate-limit.middleware';
import { validateApiKey, requireAuth, requireKYC } from '../middleware/auth.middleware';

/**
 * XP Controller - Handles all Experience Points related operations
 * Integrates with Mining, Referral, and Quality assessment systems
 * 
 * Features:
 * - Activity tracking across platforms (Instagram, TikTok, YouTube, etc.)
 * - AI-powered content quality assessment
 * - Real-time XP calculation with multipliers
 * - Level progression with mining bonuses
 * - Anti-bot protection and validation
 * - Blockchain integration for reward distribution
 */
export class XPController {
  private xpService: XPService;
  private miningService: MiningService;
  private referralService: ReferralService;
  private aiQualityService: AIQualityService;
  private antiBotService: AntiBotService;
  private blockchainService: BlockchainService;
  private notificationService: NotificationService;
  private analyticsService: AnalyticsService;
  private rateLimiter: RateLimiter;

  constructor() {
    this.xpService = new XPService();
    this.miningService = new MiningService();
    this.referralService = new ReferralService();
    this.aiQualityService = new AIQualityService();
    this.antiBotService = new AntiBotService();
    this.blockchainService = new BlockchainService();
    this.notificationService = new NotificationService();
    this.analyticsService = new AnalyticsService();
    this.rateLimiter = new RateLimiter();
  }

  /**
   * Record XP activity with comprehensive validation and quality assessment
   * POST /api/xp/activity
   */
  @validateApiKey
  @requireAuth
  @requireKYC
  async recordActivity(req: Request, res: Response): Promise<void> {
    try {
      const errors = validationResult(req);
      if (!errors.isEmpty()) {
        res.status(400).json({
          success: false,
          message: 'Validation failed',
          errors: errors.array()
        });
        return;
      }

      const userId = req.user.id;
      const {
        activityType,
        platform,
        content,
        metadata,
        socialMediaUrl,
        engagementData
      } = req.body;

      // Rate limiting check
      const rateLimitCheck = await this.rateLimiter.checkLimit(
        `xp_activity_${userId}`,
        this.getActivityLimit(activityType),
        3600 // 1 hour window
      );

      if (!rateLimitCheck.allowed) {
        res.status(429).json({
          success: false,
          message: 'Activity rate limit exceeded',
          resetTime: rateLimitCheck.resetTime
        });
        return;
      }

      // Anti-bot validation
      const humanScore = await this.antiBotService.validateActivity({
        userId,
        activityType,
        platform,
        content,
        timestamp: Date.now(),
        sessionData: req.sessionData
      });

      if (humanScore < 0.5) {
        logger.warn(`Low human score for user ${userId}: ${humanScore}`);
        res.status(403).json({
          success: false,
          message: 'Activity validation failed',
          code: 'SUSPICIOUS_ACTIVITY'
        });
        return;
      }

      // Content quality assessment using AI
      let qualityScore = 1.0;
      if (content) {
        qualityScore = await this.aiQualityService.analyzeContent({
          content,
          platform,
          activityType,
          userId,
          socialMediaUrl,
          engagementData
        });
      }

      // Calculate XP with all multipliers
      const xpCalculation = await this.calculateXPReward({
        userId,
        activityType,
        platform,
        qualityScore,
        humanScore,
        metadata
      });

      // Record activity in database
      const activity = await this.xpService.recordActivity({
        userId,
        activityType,
        platform,
        content,
        metadata,
        socialMediaUrl,
        engagementData,
        xpGained: xpCalculation.totalXP,
        qualityScore,
        humanScore,
        multipliers: xpCalculation.multipliers
      });

      // Update user XP and check for level progression
      const userXPData = await this.xpService.updateUserXP(userId, xpCalculation.totalXP);
      
      // Check for level up and mining bonus updates
      const levelUpResult = await this.checkLevelProgression(userId, userXPData);

      // Update mining multiplier if level changed
      if (levelUpResult.leveledUp) {
        await this.miningService.updateUserMultipliers(userId, {
          xpLevelMultiplier: levelUpResult.newLevel.miningMultiplier
        });

        // Send level up notification
        await this.notificationService.sendLevelUpNotification(
          userId,
          levelUpResult.newLevel
        );
      }

      // Update referral network rewards
      await this.referralService.distributeXPBonus(userId, xpCalculation.totalXP);

      // Record analytics
      await this.analyticsService.trackXPActivity({
        userId,
        activityType,
        platform,
        xpGained: xpCalculation.totalXP,
        qualityScore,
        humanScore,
        levelUp: levelUpResult.leveledUp
      });

      // Blockchain integration for significant XP gains
      if (xpCalculation.totalXP >= 100) {
        await this.blockchainService.recordXPMilestone({
          userId,
          xpGained: xpCalculation.totalXP,
          activityId: activity.id,
          qualityScore
        });
      }

      const response: ApiResponse<any> = {
        success: true,
        message: levelUpResult.leveledUp ? 'XP recorded and level up achieved!' : 'XP recorded successfully',
        data: {
          activity: {
            id: activity.id,
            xpGained: xpCalculation.totalXP,
            qualityScore,
            humanScore,
            multipliers: xpCalculation.multipliers
          },
          userStats: {
            totalXP: userXPData.totalXP,
            currentLevel: userXPData.currentLevel,
            xpToNextLevel: userXPData.xpToNextLevel,
            dailyXP: userXPData.dailyXP,
            weeklyXP: userXPData.weeklyXP
          },
          levelProgression: levelUpResult.leveledUp ? {
            newLevel: levelUpResult.newLevel.level,
            newBadge: levelUpResult.newLevel.badge,
            newMiningMultiplier: levelUpResult.newLevel.miningMultiplier,
            unlockedFeatures: levelUpResult.newLevel.unlockedFeatures
          } : null
        }
      };

      res.status(201).json(response);

    } catch (error) {
      logger.error('Error recording XP activity:', error);
      res.status(500).json({
        success: false,
        message: 'Internal server error',
        error: process.env.NODE_ENV === 'development' ? error.message : undefined
      });
    }
  }

  /**
   * Get user XP statistics and level information
   * GET /api/xp/stats/:userId
   */
  @validateApiKey
  @requireAuth
  async getUserXPStats(req: Request, res: Response): Promise<void> {
    try {
      const { userId } = req.params;
      const requestingUserId = req.user.id;

      // Privacy check - users can only view their own detailed stats
      const isOwnProfile = userId === requestingUserId;
      
      const xpStats = await this.xpService.getUserXPStats(userId, isOwnProfile);
      
      if (!xpStats) {
        res.status(404).json({
          success: false,
          message: 'User not found'
        });
        return;
      }

      // Get current level information
      const levelInfo = await this.xpService.getCurrentLevel(userId);
      
      // Get XP leaderboard position
      const leaderboardPosition = await this.xpService.getUserLeaderboardPosition(userId);

      // Get recent activities (if own profile)
      let recentActivities = null;
      if (isOwnProfile) {
        recentActivities = await this.xpService.getRecentActivities(userId, 10);
      }

      const response: ApiResponse<XPStats> = {
        success: true,
        data: {
          ...xpStats,
          levelInfo,
          leaderboardPosition,
          recentActivities
        }
      };

      res.json(response);

    } catch (error) {
      logger.error('Error fetching XP stats:', error);
      res.status(500).json({
        success: false,
        message: 'Internal server error'
      });
    }
  }

  /**
   * Get XP leaderboard with filtering options
   * GET /api/xp/leaderboard
   */
  @validateApiKey
  async getLeaderboard(req: Request, res: Response): Promise<void> {
    try {
      const {
        period = 'all-time', // 'daily', 'weekly', 'monthly', 'all-time'
        limit = 50,
        offset = 0,
        platform,
        level
      } = req.query;

      const leaderboard = await this.xpService.getLeaderboard({
        period: period as string,
        limit: Math.min(Number(limit), 100), // Max 100 entries
        offset: Number(offset),
        platform: platform as PlatformType,
        level: level ? Number(level) : undefined
      });

      const response: ApiResponse<any> = {
        success: true,
        data: {
          leaderboard,
          pagination: {
            limit: Number(limit),
            offset: Number(offset),
            total: leaderboard.length
          },
          filters: {
            period,
            platform,
            level
          }
        }
      };

      res.json(response);

    } catch (error) {
      logger.error('Error fetching leaderboard:', error);
      res.status(500).json({
        success: false,
        message: 'Internal server error'
      });
    }
  }

  /**
   * Get user's XP activity history with filters
   * GET /api/xp/history
   */
  @validateApiKey
  @requireAuth
  async getActivityHistory(req: Request, res: Response): Promise<void> {
    try {
      const userId = req.user.id;
      const {
        startDate,
        endDate,
        platform,
        activityType,
        limit = 50,
        offset = 0
      } = req.query;

      const activities = await this.xpService.getActivityHistory({
        userId,
        startDate: startDate ? new Date(startDate as string) : undefined,
        endDate: endDate ? new Date(endDate as string) : undefined,
        platform: platform as PlatformType,
        activityType: activityType as string,
        limit: Math.min(Number(limit), 100),
        offset: Number(offset)
      });

      // Calculate summary statistics
      const summary = await this.xpService.getActivitySummary({
        userId,
        startDate: startDate ? new Date(startDate as string) : undefined,
        endDate: endDate ? new Date(endDate as string) : undefined,
        platform: platform as PlatformType,
        activityType: activityType as string
      });

      const response: ApiResponse<any> = {
        success: true,
        data: {
          activities,
          summary,
          pagination: {
            limit: Number(limit),
            offset: Number(offset),
            total: activities.length
          }
        }
      };

      res.json(response);

    } catch (error) {
      logger.error('Error fetching activity history:', error);
      res.status(500).json({
        success: false,
        message: 'Internal server error'
      });
    }
  }

  /**
   * Get available daily quests for XP
   * GET /api/xp/quests/daily
   */
  @validateApiKey
  @requireAuth
  async getDailyQuests(req: Request, res: Response): Promise<void> {
    try {
      const userId = req.user.id;
      
      const quests = await this.xpService.getDailyQuests(userId);
      
      const response: ApiResponse<any> = {
        success: true,
        data: {
          quests,
          resetTime: this.xpService.getNextQuestReset(),
          totalPossibleXP: quests.reduce((sum, quest) => sum + quest.xpReward, 0)
        }
      };

      res.json(response);

    } catch (error) {
      logger.error('Error fetching daily quests:', error);
      res.status(500).json({
        success: false,
        message: 'Internal server error'
      });
    }
  }

  /**
   * Complete a daily quest
   * POST /api/xp/quests/:questId/complete
   */
  @validateApiKey
  @requireAuth
  async completeQuest(req: Request, res: Response): Promise<void> {
    try {
      const { questId } = req.params;
      const userId = req.user.id;
      const { proof } = req.body;

      const questResult = await this.xpService.completeQuest({
        questId,
        userId,
        proof
      });

      if (!questResult.success) {
        res.status(400).json({
          success: false,
          message: questResult.message
        });
        return;
      }

      // Update user XP
      const userXPData = await this.xpService.updateUserXP(userId, questResult.xpReward);
      
      // Check for level progression
      const levelUpResult = await this.checkLevelProgression(userId, userXPData);

      // Update mining multiplier if leveled up
      if (levelUpResult.leveledUp) {
        await this.miningService.updateUserMultipliers(userId, {
          xpLevelMultiplier: levelUpResult.newLevel.miningMultiplier
        });
      }

      const response: ApiResponse<any> = {
        success: true,
        message: 'Quest completed successfully!',
        data: {
          quest: questResult.quest,
          xpGained: questResult.xpReward,
          userStats: {
            totalXP: userXPData.totalXP,
            currentLevel: userXPData.currentLevel,
            xpToNextLevel: userXPData.xpToNextLevel
          },
          levelUp: levelUpResult.leveledUp ? levelUpResult.newLevel : null
        }
      };

      res.json(response);

    } catch (error) {
      logger.error('Error completing quest:', error);
      res.status(500).json({
        success: false,
        message: 'Internal server error'
      });
    }
  }

  /**
   * Get XP multipliers and bonuses for user
   * GET /api/xp/multipliers
   */
  @validateApiKey
  @requireAuth
  async getUserMultipliers(req: Request, res: Response): Promise<void> {
    try {
      const userId = req.user.id;
      
      const multipliers = await this.xpService.getUserMultipliers(userId);
      
      const response: ApiResponse<any> = {
        success: true,
        data: multipliers
      };

      res.json(response);

    } catch (error) {
      logger.error('Error fetching multipliers:', error);
      res.status(500).json({
        success: false,
        message: 'Internal server error'
      });
    }
  }

  /**
   * Private helper methods
   */
  private async calculateXPReward(params: {
    userId: string;
    activityType: string;
    platform: PlatformType;
    qualityScore: number;
    humanScore: number;
    metadata?: any;
  }): Promise<{
    totalXP: number;
    multipliers: {
      base: number;
      platform: number;
      quality: number;
      streak: number;
      level: number;
      special: number;
    };
  }> {
    const { userId, activityType, platform, qualityScore, humanScore, metadata } = params;

    // Get base XP for activity type
    const baseXP = this.xpService.getBaseXP(activityType);
    
    // Get platform multiplier
    const platformMultiplier = this.xpService.getPlatformMultiplier(platform);
    
    // Get user's current streak bonus
    const streakBonus = await this.xpService.getStreakBonus(userId);
    
    // Get level progression factor
    const userLevel = await this.xpService.getUserCurrentLevel(userId);
    const levelProgression = Math.exp(-0.01 * userLevel);
    
    // Special bonuses (events, cards, etc.)
    const specialBonuses = await this.xpService.getSpecialBonuses(userId);
    
    const multipliers = {
      base: baseXP,
      platform: platformMultiplier,
      quality: qualityScore,
      streak: streakBonus,
      level: levelProgression,
      special: specialBonuses
    };

    const totalXP = Math.floor(
      baseXP * platformMultiplier * qualityScore * streakBonus * levelProgression * specialBonuses
    );

    return { totalXP, multipliers };
  }

  private async checkLevelProgression(userId: string, userXPData: any): Promise<{
    leveledUp: boolean;
    newLevel?: XPLevel;
    previousLevel?: number;
  }> {
    const currentLevel = await this.xpService.calculateLevel(userXPData.totalXP);
    
    if (currentLevel.level > userXPData.currentLevel) {
      const newLevelInfo = await this.xpService.getLevelInfo(currentLevel.level);
      
      await this.xpService.updateUserLevel(userId, currentLevel.level);
      
      return {
        leveledUp: true,
        newLevel: newLevelInfo,
        previousLevel: userXPData.currentLevel
      };
    }

    return { leveledUp: false };
  }

  private getActivityLimit(activityType: string): number {
    const limits: { [key: string]: number } = {
      'original_post': 20,
      'comment': 100,
      'like': 200,
      'share': 50,
      'follow': 25,
      'story': 50,
      'video': 10
    };

    return limits[activityType] || 10;
  }
}

// Validation middleware for XP endpoints
export const xpValidators = {
  recordActivity: [
    body('activityType')
      .isIn(['original_post', 'comment', 'like', 'share', 'follow', 'story', 'video', 'photo'])
      .withMessage('Invalid activity type'),
    body('platform')
      .isIn(['instagram', 'tiktok', 'youtube', 'facebook', 'twitter', 'linkedin'])
      .withMessage('Invalid platform'),
    body('content')
      .optional()
      .isString()
      .isLength({ max: 10000 })
      .withMessage('Content too long'),
    body('socialMediaUrl')
      .optional()
      .isURL()
      .withMessage('Invalid URL format'),
    body('engagementData')
      .optional()
      .isObject()
      .withMessage('Engagement data must be an object')
  ],
  
  getUserStats: [
    param('userId').isUUID().withMessage('Invalid user ID')
  ],
  
  getLeaderboard: [
    query('period').optional().isIn(['daily', 'weekly', 'monthly', 'all-time']),
    query('limit').optional().isInt({ min: 1, max: 100 }),
    query('offset').optional().isInt({ min: 0 })
  ],
  
  completeQuest: [
    param('questId').isUUID().withMessage('Invalid quest ID'),
    body('proof').optional().isObject().withMessage('Proof must be an object')
  ]
};

export default XPController;
