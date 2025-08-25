import { Request, Response } from 'express';
import { validationResult } from 'express-validator';
import { prisma } from '../config/database';
import { SocialService } from '../services/social.service';
import { XPService } from '../services/xp.service';
import { MiningService } from '../services/mining.service';
import { AIQualityService } from '../services/ai-quality.service';
import { AntiBotService } from '../services/anti-bot.service';
import { BlockchainService } from '../services/blockchain.service';
import { NotificationService } from '../services/notification.service';
import { logger } from '../utils/logger';
import { ApiResponse } from '../types/api.types';
import { 
  SocialActivity, 
  ContentType, 
  Platform, 
  QualityScore,
  XPCalculationResult,
  MiningRewardResult 
} from '../types/social.types';

export class SocialController {
  private socialService: SocialService;
  private xpService: XPService;
  private miningService: MiningService;
  private aiQualityService: AIQualityService;
  private antiBotService: AntiBotService;
  private blockchainService: BlockchainService;
  private notificationService: NotificationService;

  constructor() {
    this.socialService = new SocialService();
    this.xpService = new XPService();
    this.miningService = new MiningService();
    this.aiQualityService = new AIQualityService();
    this.antiBotService = new AntiBotService();
    this.blockchainService = new BlockchainService();
    this.notificationService = new NotificationService();
  }

  /**
   * Submit social media activity for XP and mining rewards
   * POST /api/social/activity
   */
  public submitActivity = async (req: Request, res: Response): Promise<void> => {
    try {
      const errors = validationResult(req);
      if (!errors.isEmpty()) {
        res.status(400).json({
          success: false,
          message: 'Validation failed',
          errors: errors.array()
        } as ApiResponse);
        return;
      }

      const userId = req.user?.id;
      const {
        platform,
        contentType,
        content,
        mediaUrls = [],
        postUrl,
        engagement = {}
      } = req.body;

      // Anti-bot verification
      const humanScore = await this.antiBotService.calculateHumanProbability({
        userId,
        activityType: contentType,
        content,
        timestamp: new Date(),
        deviceFingerprint: req.headers['x-device-fingerprint'] as string,
        sessionData: req.session
      });

      if (humanScore < 0.7) {
        logger.warn(`Low human score detected for user ${userId}: ${humanScore}`);
        res.status(429).json({
          success: false,
          message: 'Activity verification failed. Please try again later.',
          data: { humanScore: Math.round(humanScore * 100) }
        } as ApiResponse);
        return;
      }

      // Check daily limits
      const dailyStats = await this.socialService.getDailyActivityStats(userId);
      if (!this.socialService.isWithinDailyLimits(contentType, dailyStats)) {
        res.status(429).json({
          success: false,
          message: 'Daily activity limit reached for this content type'
        } as ApiResponse);
        return;
      }

      // AI content quality analysis
      const qualityScore = await this.aiQualityService.analyzeContentQuality({
        content,
        mediaUrls,
        platform,
        contentType
      });

      // Create social activity record
      const activity: SocialActivity = await prisma.socialActivity.create({
        data: {
          userId,
          platform,
          contentType,
          content,
          mediaUrls,
          postUrl,
          engagement,
          qualityScore: qualityScore.score,
          qualityDetails: qualityScore.details,
          humanScore,
          status: 'PENDING_VERIFICATION'
        }
      });

      // Calculate XP rewards
      const xpResult = await this.calculateXPReward(userId, activity, qualityScore);
      
      // Calculate mining boost
      const miningResult = await this.calculateMiningReward(userId, activity, qualityScore);

      // Update user XP and mining rate
      await this.updateUserRewards(userId, xpResult, miningResult);

      // Update referral network rewards
      await this.updateReferralRewards(userId, xpResult, miningResult);

      // Send blockchain transaction for significant rewards
      if (miningResult.finAmount > 0.1) {
        await this.blockchainService.distributeMiningReward(
          userId,
          miningResult.finAmount,
          activity.id
        );
      }

      // Send real-time notifications
      await this.notificationService.sendActivityReward(userId, {
        xpGained: xpResult.totalXP,
        miningBoost: miningResult.boostPercentage,
        qualityScore: qualityScore.score
      });

      res.status(201).json({
        success: true,
        message: 'Activity submitted successfully',
        data: {
          activityId: activity.id,
          xpReward: xpResult,
          miningReward: miningResult,
          qualityScore: qualityScore.score,
          humanScore: Math.round(humanScore * 100)
        }
      } as ApiResponse);

    } catch (error) {
      logger.error('Error submitting social activity:', error);
      res.status(500).json({
        success: false,
        message: 'Failed to submit activity'
      } as ApiResponse);
    }
  };

  /**
   * Get user's social activity history
   * GET /api/social/activities
   */
  public getActivities = async (req: Request, res: Response): Promise<void> => {
    try {
      const userId = req.user?.id;
      const { 
        page = 1, 
        limit = 20, 
        platform, 
        contentType,
        startDate,
        endDate 
      } = req.query;

      const activities = await this.socialService.getUserActivities(userId, {
        page: Number(page),
        limit: Number(limit),
        platform: platform as Platform,
        contentType: contentType as ContentType,
        startDate: startDate ? new Date(startDate as string) : undefined,
        endDate: endDate ? new Date(endDate as string) : undefined
      });

      const stats = await this.socialService.getActivityStats(userId);

      res.json({
        success: true,
        data: {
          activities: activities.data,
          pagination: activities.pagination,
          stats
        }
      } as ApiResponse);

    } catch (error) {
      logger.error('Error fetching activities:', error);
      res.status(500).json({
        success: false,
        message: 'Failed to fetch activities'
      } as ApiResponse);
    }
  };

  /**
   * Connect social media platform
   * POST /api/social/connect
   */
  public connectPlatform = async (req: Request, res: Response): Promise<void> => {
    try {
      const userId = req.user?.id;
      const { platform, accessToken, refreshToken, expiresAt } = req.body;

      // Verify platform token
      const platformUser = await this.socialService.verifyPlatformToken(
        platform,
        accessToken
      );

      if (!platformUser) {
        res.status(400).json({
          success: false,
          message: 'Invalid platform credentials'
        } as ApiResponse);
        return;
      }

      // Store platform connection
      const connection = await prisma.socialConnection.upsert({
        where: {
          userId_platform: {
            userId,
            platform
          }
        },
        update: {
          accessToken,
          refreshToken,
          expiresAt: new Date(expiresAt),
          platformUserId: platformUser.id,
          platformUsername: platformUser.username,
          isActive: true,
          updatedAt: new Date()
        },
        create: {
          userId,
          platform,
          accessToken,
          refreshToken,
          expiresAt: new Date(expiresAt),
          platformUserId: platformUser.id,
          platformUsername: platformUser.username,
          isActive: true
        }
      });

      // Award connection bonus
      const connectionBonus = await this.xpService.awardConnectionBonus(userId, platform);

      res.json({
        success: true,
        message: 'Platform connected successfully',
        data: {
          connection: {
            platform: connection.platform,
            username: connection.platformUsername,
            connectedAt: connection.createdAt
          },
          bonus: connectionBonus
        }
      } as ApiResponse);

    } catch (error) {
      logger.error('Error connecting platform:', error);
      res.status(500).json({
        success: false,
        message: 'Failed to connect platform'
      } as ApiResponse);
    }
  };

  /**
   * Get connected platforms
   * GET /api/social/connections
   */
  public getConnections = async (req: Request, res: Response): Promise<void> => {
    try {
      const userId = req.user?.id;

      const connections = await prisma.socialConnection.findMany({
        where: { userId, isActive: true },
        select: {
          platform: true,
          platformUsername: true,
          createdAt: true,
          lastSyncAt: true
        }
      });

      res.json({
        success: true,
        data: { connections }
      } as ApiResponse);

    } catch (error) {
      logger.error('Error fetching connections:', error);
      res.status(500).json({
        success: false,
        message: 'Failed to fetch connections'
      } as ApiResponse);
    }
  };

  /**
   * Sync activities from connected platforms
   * POST /api/social/sync
   */
  public syncPlatforms = async (req: Request, res: Response): Promise<void> => {
    try {
      const userId = req.user?.id;
      const { platforms } = req.body;

      const syncResults = await Promise.allSettled(
        platforms.map((platform: Platform) => 
          this.socialService.syncPlatformActivities(userId, platform)
        )
      );

      const results = syncResults.map((result, index) => ({
        platform: platforms[index],
        success: result.status === 'fulfilled',
        data: result.status === 'fulfilled' ? result.value : null,
        error: result.status === 'rejected' ? result.reason.message : null
      }));

      res.json({
        success: true,
        message: 'Platform sync completed',
        data: { results }
      } as ApiResponse);

    } catch (error) {
      logger.error('Error syncing platforms:', error);
      res.status(500).json({
        success: false,
        message: 'Failed to sync platforms'
      } as ApiResponse);
    }
  };

  /**
   * Get social leaderboard
   * GET /api/social/leaderboard
   */
  public getLeaderboard = async (req: Request, res: Response): Promise<void> => {
    try {
      const { 
        type = 'xp', 
        period = 'weekly',
        limit = 50 
      } = req.query;

      const leaderboard = await this.socialService.getLeaderboard({
        type: type as 'xp' | 'mining' | 'referral',
        period: period as 'daily' | 'weekly' | 'monthly' | 'allTime',
        limit: Number(limit)
      });

      res.json({
        success: true,
        data: { leaderboard }
      } as ApiResponse);

    } catch (error) {
      logger.error('Error fetching leaderboard:', error);
      res.status(500).json({
        success: false,
        message: 'Failed to fetch leaderboard'
      } as ApiResponse);
    }
  };

  /**
   * Report inappropriate content
   * POST /api/social/report
   */
  public reportContent = async (req: Request, res: Response): Promise<void> => {
    try {
      const userId = req.user?.id;
      const { activityId, reason, description } = req.body;

      const report = await this.socialService.reportContent(
        userId,
        activityId,
        reason,
        description
      );

      res.json({
        success: true,
        message: 'Content reported successfully',
        data: { reportId: report.id }
      } as ApiResponse);

    } catch (error) {
      logger.error('Error reporting content:', error);
      res.status(500).json({
        success: false,
        message: 'Failed to report content'
      } as ApiResponse);
    }
  };

  /**
   * Get user's social stats and insights
   * GET /api/social/stats
   */
  public getStats = async (req: Request, res: Response): Promise<void> => {
    try {
      const userId = req.user?.id;
      const { period = '30d' } = req.query;

      const stats = await this.socialService.getUserStats(userId, period as string);
      
      res.json({
        success: true,
        data: { stats }
      } as ApiResponse);

    } catch (error) {
      logger.error('Error fetching social stats:', error);
      res.status(500).json({
        success: false,
        message: 'Failed to fetch stats'
      } as ApiResponse);
    }
  };

  // Private helper methods

  private async calculateXPReward(
    userId: string, 
    activity: SocialActivity, 
    qualityScore: QualityScore
  ): Promise<XPCalculationResult> {
    const user = await prisma.user.findUnique({
      where: { id: userId },
      include: { xpProfile: true }
    });

    const baseXP = this.getBaseXP(activity.contentType);
    const platformMultiplier = this.getPlatformMultiplier(activity.platform);
    const qualityMultiplier = qualityScore.score;
    const streakBonus = this.calculateStreakBonus(user?.xpProfile?.streakDays || 0);
    const levelProgression = Math.exp(-0.01 * (user?.xpProfile?.level || 1));

    const totalXP = Math.floor(
      baseXP * platformMultiplier * qualityMultiplier * streakBonus * levelProgression
    );

    return {
      baseXP,
      multipliers: {
        platform: platformMultiplier,
        quality: qualityMultiplier,
        streak: streakBonus,
        level: levelProgression
      },
      totalXP,
      newLevel: this.xpService.calculateLevel(
        (user?.xpProfile?.totalXP || 0) + totalXP
      )
    };
  }

  private async calculateMiningReward(
    userId: string,
    activity: SocialActivity,
    qualityScore: QualityScore
  ): Promise<MiningRewardResult> {
    const miningRate = await this.miningService.getCurrentMiningRate(userId);
    const activityBoost = this.getActivityMiningBoost(activity.contentType);
    const qualityBoost = qualityScore.score;
    
    const boostPercentage = (activityBoost * qualityBoost - 1) * 100;
    const boostDuration = this.getBoostDuration(activity.contentType);
    const finAmount = miningRate * activityBoost * qualityBoost * (boostDuration / 24);

    return {
      baseRate: miningRate,
      boostPercentage,
      boostDuration,
      finAmount,
      expiresAt: new Date(Date.now() + boostDuration * 60 * 60 * 1000)
    };
  }

  private async updateUserRewards(
    userId: string,
    xpResult: XPCalculationResult,
    miningResult: MiningRewardResult
  ): Promise<void> {
    await prisma.$transaction(async (tx) => {
      // Update XP
      await tx.xpProfile.upsert({
        where: { userId },
        update: {
          totalXP: { increment: xpResult.totalXP },
          level: xpResult.newLevel,
          lastActivityAt: new Date()
        },
        create: {
          userId,
          totalXP: xpResult.totalXP,
          level: xpResult.newLevel,
          streakDays: 1,
          lastActivityAt: new Date()
        }
      });

      // Update mining boost
      if (miningResult.boostPercentage > 0) {
        await tx.miningBoost.create({
          data: {
            userId,
            boostPercentage: miningResult.boostPercentage,
            duration: miningResult.boostDuration,
            expiresAt: miningResult.expiresAt,
            source: 'SOCIAL_ACTIVITY'
          }
        });
      }
    });
  }

  private async updateReferralRewards(
    userId: string,
    xpResult: XPCalculationResult,
    miningResult: MiningRewardResult
  ): Promise<void> {
    const referrer = await prisma.referral.findFirst({
      where: { referredId: userId },
      include: { referrer: true }
    });

    if (referrer) {
      const rpBonus = Math.floor(xpResult.totalXP * 0.05); // 5% of XP as RP
      
      await prisma.referralProfile.upsert({
        where: { userId: referrer.referrerId },
        update: {
          totalRP: { increment: rpBonus },
          totalEarnings: { increment: miningResult.finAmount * 0.1 }
        },
        create: {
          userId: referrer.referrerId,
          totalRP: rpBonus,
          totalEarnings: miningResult.finAmount * 0.1
        }
      });
    }
  }

  private getBaseXP(contentType: ContentType): number {
    const baseXPMap: Record<ContentType, number> = {
      'ORIGINAL_POST': 50,
      'PHOTO_POST': 75,
      'VIDEO_POST': 150,
      'STORY': 25,
      'COMMENT': 25,
      'LIKE': 5,
      'SHARE': 15,
      'FOLLOW': 20
    };
    return baseXPMap[contentType] || 10;
  }

  private getPlatformMultiplier(platform: Platform): number {
    const multiplierMap: Record<Platform, number> = {
      'TIKTOK': 1.3,
      'INSTAGRAM': 1.2,
      'YOUTUBE': 1.4,
      'FACEBOOK': 1.1,
      'TWITTER': 1.2,
      'LINKEDIN': 1.1
    };
    return multiplierMap[platform] || 1.0;
  }

  private calculateStreakBonus(streakDays: number): number {
    return Math.min(1 + (streakDays * 0.02), 3.0); // Max 3x bonus
  }

  private getActivityMiningBoost(contentType: ContentType): number {
    const boostMap: Record<ContentType, number> = {
      'ORIGINAL_POST': 1.2,
      'PHOTO_POST': 1.15,
      'VIDEO_POST': 1.3,
      'STORY': 1.1,
      'COMMENT': 1.05,
      'LIKE': 1.01,
      'SHARE': 1.08,
      'FOLLOW': 1.05
    };
    return boostMap[contentType] || 1.0;
  }

  private getBoostDuration(contentType: ContentType): number {
    const durationMap: Record<ContentType, number> = {
      'ORIGINAL_POST': 24,
      'PHOTO_POST': 12,
      'VIDEO_POST': 48,
      'STORY': 6,
      'COMMENT': 2,
      'LIKE': 0.5,
      'SHARE': 4,
      'FOLLOW': 1
    };
    return durationMap[contentType] || 1;
  }
}
