import express from 'express';
import { body, query, param, validationResult } from 'express-validator';
import rateLimit from 'express-rate-limit';
import { authMiddleware } from '../middleware/auth.middleware';
import { kycMiddleware } from '../middleware/kyc.middleware';
import { validationMiddleware } from '../middleware/validation.middleware';
import { XPService } from '../services/xp.service';
import { AntiBot } from '../services/anti-bot.service';
import { AIQuality } from '../services/ai-quality.service';
import { logger } from '../utils/logger';
import { ApiResponse } from '../types/api.types';
import { XPActivity, XPLevel, XPReward } from '../types/xp.types';

const router = express.Router();
const xpService = new XPService();
const antiBotService = new AntiBot();
const aiQualityService = new AIQuality();

// Rate limiting for XP endpoints
const xpRateLimit = rateLimit({
  windowMs: 15 * 60 * 1000, // 15 minutes
  max: 100, // 100 requests per window
  message: 'Too many XP requests, please try again later',
  standardHeaders: true,
  legacyHeaders: false
});

const activityRateLimit = rateLimit({
  windowMs: 60 * 1000, // 1 minute
  max: 20, // 20 activities per minute
  message: 'Activity submission rate exceeded',
  keyGenerator: (req) => `${req.user?.id}-activity`
});

/**
 * @route GET /api/xp/profile/:userId?
 * @desc Get user XP profile and statistics
 * @access Private
 */
router.get('/profile/:userId?',
  authMiddleware,
  [
    param('userId').optional().isUUID().withMessage('Invalid user ID format'),
    query('includeHistory').optional().isBoolean().withMessage('Include history must be boolean'),
    query('timeRange').optional().isIn(['day', 'week', 'month', 'year', 'all']).withMessage('Invalid time range')
  ],
  validationMiddleware,
  async (req, res) => {
    try {
      const userId = req.params.userId || req.user.id;
      const includeHistory = req.query.includeHistory === 'true';
      const timeRange = req.query.timeRange as string || 'month';

      // Authorization check
      if (userId !== req.user.id && !req.user.roles?.includes('admin')) {
        return res.status(403).json({
          success: false,
          message: 'Access denied'
        });
      }

      const profile = await xpService.getUserXPProfile(userId, {
        includeHistory,
        timeRange
      });

      const response: ApiResponse<typeof profile> = {
        success: true,
        message: 'XP profile retrieved successfully',
        data: profile,
        timestamp: new Date().toISOString()
      };

      res.json(response);
    } catch (error) {
      logger.error('Get XP profile error:', error);
      res.status(500).json({
        success: false,
        message: 'Failed to retrieve XP profile',
        error: process.env.NODE_ENV === 'development' ? error.message : undefined
      });
    }
  }
);

/**
 * @route POST /api/xp/activity
 * @desc Submit activity for XP calculation
 * @access Private
 */
router.post('/activity',
  authMiddleware,
  kycMiddleware,
  activityRateLimit,
  [
    body('type').isIn([
      'original_post', 'photo_post', 'video_content', 'story_status',
      'meaningful_comment', 'like_react', 'share_repost', 'follow_subscribe',
      'daily_login', 'complete_quest', 'achieve_milestone', 'viral_content'
    ]).withMessage('Invalid activity type'),
    body('platform').isIn([
      'instagram', 'tiktok', 'youtube', 'facebook', 'twitter', 'app'
    ]).withMessage('Invalid platform'),
    body('content').optional().isString().isLength({ max: 5000 }).withMessage('Content too long'),
    body('mediaUrl').optional().isURL().withMessage('Invalid media URL'),
    body('externalId').optional().isString().withMessage('External ID must be string'),
    body('metadata').optional().isObject().withMessage('Metadata must be object'),
    body('timestamp').optional().isISO8601().withMessage('Invalid timestamp format')
  ],
  validationMiddleware,
  async (req, res) => {
    try {
      const { type, platform, content, mediaUrl, externalId, metadata, timestamp } = req.body;
      const userId = req.user.id;

      // Anti-bot verification
      const botCheck = await antiBotService.verifyActivity(userId, {
        type,
        platform,
        timestamp: timestamp || new Date().toISOString(),
        fingerprint: req.fingerprint,
        userAgent: req.headers['user-agent']
      });

      if (botCheck.suspiciousScore > 0.8) {
        logger.warn(`High bot probability for user ${userId}:`, botCheck);
        return res.status(429).json({
          success: false,
          message: 'Activity verification failed',
          retryAfter: botCheck.cooldownSeconds
        });
      }

      // Content quality analysis (if content provided)
      let qualityScore = 1.0;
      if (content || mediaUrl) {
        const qualityAnalysis = await aiQualityService.analyzeContent({
          text: content,
          mediaUrl,
          platform,
          type
        });
        qualityScore = qualityAnalysis.score;

        // Reject low-quality content
        if (qualityScore < 0.3) {
          return res.status(400).json({
            success: false,
            message: 'Content quality below minimum threshold',
            details: qualityAnalysis.reasons
          });
        }
      }

      // Calculate XP reward
      const xpReward = await xpService.calculateXPReward(userId, {
        type,
        platform,
        content,
        mediaUrl,
        externalId,
        qualityScore,
        metadata: {
          ...metadata,
          botScore: botCheck.suspiciousScore,
          qualityFactors: qualityScore < 1.0 ? await aiQualityService.getQualityFactors() : undefined
        }
      });

      // Award XP
      const result = await xpService.awardXP(userId, xpReward);

      // Log activity for analytics
      logger.info(`XP awarded: ${result.xpGained} to user ${userId}`, {
        activity: type,
        platform,
        qualityScore,
        botScore: botCheck.suspiciousScore,
        newLevel: result.newLevel,
        levelUp: result.leveledUp
      });

      const response: ApiResponse<typeof result> = {
        success: true,
        message: result.leveledUp ? 
          `Congratulations! You leveled up to ${result.newLevel}!` : 
          'XP awarded successfully',
        data: result,
        timestamp: new Date().toISOString()
      };

      res.json(response);
    } catch (error) {
      logger.error('Submit XP activity error:', error);
      
      if (error.code === 'DUPLICATE_ACTIVITY') {
        return res.status(409).json({
          success: false,
          message: 'Activity already recorded',
          error: error.message
        });
      }

      if (error.code === 'DAILY_LIMIT_EXCEEDED') {
        return res.status(429).json({
          success: false,
          message: 'Daily activity limit exceeded',
          error: error.message,
          retryAfter: error.retryAfter
        });
      }

      res.status(500).json({
        success: false,
        message: 'Failed to process activity',
        error: process.env.NODE_ENV === 'development' ? error.message : undefined
      });
    }
  }
);

/**
 * @route GET /api/xp/leaderboard
 * @desc Get XP leaderboard
 * @access Private
 */
router.get('/leaderboard',
  authMiddleware,
  xpRateLimit,
  [
    query('timeframe').optional().isIn(['daily', 'weekly', 'monthly', 'all-time']).withMessage('Invalid timeframe'),
    query('category').optional().isIn(['level', 'xp', 'activity']).withMessage('Invalid category'),
    query('limit').optional().isInt({ min: 1, max: 100 }).withMessage('Limit must be between 1 and 100'),
    query('offset').optional().isInt({ min: 0 }).withMessage('Invalid offset'),
    query('guild').optional().isUUID().withMessage('Invalid guild ID')
  ],
  validationMiddleware,
  async (req, res) => {
    try {
      const {
        timeframe = 'weekly',
        category = 'xp',
        limit = 20,
        offset = 0,
        guild
      } = req.query;

      const leaderboard = await xpService.getLeaderboard({
        timeframe: timeframe as string,
        category: category as string,
        limit: parseInt(limit as string),
        offset: parseInt(offset as string),
        guildId: guild as string,
        requesterId: req.user.id
      });

      const response: ApiResponse<typeof leaderboard> = {
        success: true,
        message: 'Leaderboard retrieved successfully',
        data: leaderboard,
        timestamp: new Date().toISOString()
      };

      res.json(response);
    } catch (error) {
      logger.error('Get XP leaderboard error:', error);
      res.status(500).json({
        success: false,
        message: 'Failed to retrieve leaderboard',
        error: process.env.NODE_ENV === 'development' ? error.message : undefined
      });
    }
  }
);

/**
 * @route GET /api/xp/levels
 * @desc Get XP level requirements and rewards
 * @access Private
 */
router.get('/levels',
  authMiddleware,
  xpRateLimit,
  async (req, res) => {
    try {
      const levels = await xpService.getLevelRequirements();

      const response: ApiResponse<typeof levels> = {
        success: true,
        message: 'XP levels retrieved successfully',
        data: levels,
        timestamp: new Date().toISOString()
      };

      res.json(response);
    } catch (error) {
      logger.error('Get XP levels error:', error);
      res.status(500).json({
        success: false,
        message: 'Failed to retrieve XP levels',
        error: process.env.NODE_ENV === 'development' ? error.message : undefined
      });
    }
  }
);

/**
 * @route GET /api/xp/activities/:activityId
 * @desc Get specific activity details
 * @access Private
 */
router.get('/activities/:activityId',
  authMiddleware,
  [
    param('activityId').isUUID().withMessage('Invalid activity ID format')
  ],
  validationMiddleware,
  async (req, res) => {
    try {
      const { activityId } = req.params;
      const userId = req.user.id;

      const activity = await xpService.getActivity(activityId, userId);

      if (!activity) {
        return res.status(404).json({
          success: false,
          message: 'Activity not found'
        });
      }

      const response: ApiResponse<typeof activity> = {
        success: true,
        message: 'Activity retrieved successfully',
        data: activity,
        timestamp: new Date().toISOString()
      };

      res.json(response);
    } catch (error) {
      logger.error('Get XP activity error:', error);
      res.status(500).json({
        success: false,
        message: 'Failed to retrieve activity',
        error: process.env.NODE_ENV === 'development' ? error.message : undefined
      });
    }
  }
);

/**
 * @route GET /api/xp/statistics
 * @desc Get user XP statistics and analytics
 * @access Private
 */
router.get('/statistics',
  authMiddleware,
  [
    query('period').optional().isIn(['day', 'week', 'month', 'quarter', 'year']).withMessage('Invalid period'),
    query('includeComparison').optional().isBoolean().withMessage('Include comparison must be boolean'),
    query('breakdown').optional().isIn(['platform', 'activity', 'time']).withMessage('Invalid breakdown type')
  ],
  validationMiddleware,
  async (req, res) => {
    try {
      const userId = req.user.id;
      const {
        period = 'month',
        includeComparison = false,
        breakdown = 'platform'
      } = req.query;

      const statistics = await xpService.getUserStatistics(userId, {
        period: period as string,
        includeComparison: includeComparison === 'true',
        breakdown: breakdown as string
      });

      const response: ApiResponse<typeof statistics> = {
        success: true,
        message: 'XP statistics retrieved successfully',
        data: statistics,
        timestamp: new Date().toISOString()
      };

      res.json(response);
    } catch (error) {
      logger.error('Get XP statistics error:', error);
      res.status(500).json({
        success: false,
        message: 'Failed to retrieve statistics',
        error: process.env.NODE_ENV === 'development' ? error.message : undefined
      });
    }
  }
);

/**
 * @route POST /api/xp/streak
 * @desc Claim daily streak bonus
 * @access Private
 */
router.post('/streak',
  authMiddleware,
  kycMiddleware,
  rateLimit({
    windowMs: 24 * 60 * 60 * 1000, // 24 hours
    max: 1, // Only once per day
    message: 'Daily streak already claimed',
    keyGenerator: (req) => `${req.user.id}-streak`
  }),
  async (req, res) => {
    try {
      const userId = req.user.id;

      // Anti-bot check for streak claims
      const botCheck = await antiBotService.verifyStreak(userId, {
        timestamp: new Date().toISOString(),
        fingerprint: req.fingerprint,
        userAgent: req.headers['user-agent']
      });

      if (botCheck.suspiciousScore > 0.5) {
        return res.status(429).json({
          success: false,
          message: 'Streak verification failed',
          retryAfter: botCheck.cooldownSeconds
        });
      }

      const streakResult = await xpService.claimDailyStreak(userId);

      const response: ApiResponse<typeof streakResult> = {
        success: true,
        message: `Daily streak claimed! ${streakResult.streakDays} days in a row!`,
        data: streakResult,
        timestamp: new Date().toISOString()
      };

      res.json(response);
    } catch (error) {
      logger.error('Claim streak error:', error);

      if (error.code === 'STREAK_ALREADY_CLAIMED') {
        return res.status(409).json({
          success: false,
          message: 'Daily streak already claimed today',
          nextClaimTime: error.nextClaimTime
        });
      }

      res.status(500).json({
        success: false,
        message: 'Failed to claim streak',
        error: process.env.NODE_ENV === 'development' ? error.message : undefined
      });
    }
  }
);

/**
 * @route POST /api/xp/boost
 * @desc Apply XP boost card or multiplier
 * @access Private
 */
router.post('/boost',
  authMiddleware,
  [
    body('boostType').isIn(['card', 'premium', 'event']).withMessage('Invalid boost type'),
    body('boostId').isString().withMessage('Boost ID required'),
    body('duration').optional().isInt({ min: 1, max: 168 }).withMessage('Duration must be 1-168 hours')
  ],
  validationMiddleware,
  async (req, res) => {
    try {
      const userId = req.user.id;
      const { boostType, boostId, duration } = req.body;

      const boostResult = await xpService.applyXPBoost(userId, {
        type: boostType,
        id: boostId,
        duration
      });

      const response: ApiResponse<typeof boostResult> = {
        success: true,
        message: `XP boost activated! ${boostResult.multiplier}x for ${boostResult.duration} hours`,
        data: boostResult,
        timestamp: new Date().toISOString()
      };

      res.json(response);
    } catch (error) {
      logger.error('Apply XP boost error:', error);

      if (error.code === 'INSUFFICIENT_CARDS') {
        return res.status(400).json({
          success: false,
          message: 'Insufficient boost cards',
          required: error.required,
          available: error.available
        });
      }

      if (error.code === 'BOOST_ALREADY_ACTIVE') {
        return res.status(409).json({
          success: false,
          message: 'XP boost already active',
          activeUntil: error.activeUntil
        });
      }

      res.status(500).json({
        success: false,
        message: 'Failed to apply XP boost',
        error: process.env.NODE_ENV === 'development' ? error.message : undefined
      });
    }
  }
);

/**
 * @route GET /api/xp/achievements
 * @desc Get user XP achievements and milestones
 * @access Private
 */
router.get('/achievements',
  authMiddleware,
  [
    query('category').optional().isIn(['level', 'activity', 'streak', 'social', 'special']).withMessage('Invalid category'),
    query('status').optional().isIn(['completed', 'in-progress', 'locked']).withMessage('Invalid status')
  ],
  validationMiddleware,
  async (req, res) => {
    try {
      const userId = req.user.id;
      const { category, status } = req.query;

      const achievements = await xpService.getUserAchievements(userId, {
        category: category as string,
        status: status as string
      });

      const response: ApiResponse<typeof achievements> = {
        success: true,
        message: 'Achievements retrieved successfully',
        data: achievements,
        timestamp: new Date().toISOString()
      };

      res.json(response);
    } catch (error) {
      logger.error('Get achievements error:', error);
      res.status(500).json({
        success: false,
        message: 'Failed to retrieve achievements',
        error: process.env.NODE_ENV === 'development' ? error.message : undefined
      });
    }
  }
);

/**
 * @route POST /api/xp/claim-achievement
 * @desc Claim completed achievement rewards
 * @access Private
 */
router.post('/claim-achievement',
  authMiddleware,
  [
    body('achievementId').isUUID().withMessage('Invalid achievement ID')
  ],
  validationMiddleware,
  async (req, res) => {
    try {
      const userId = req.user.id;
      const { achievementId } = req.body;

      const claimResult = await xpService.claimAchievement(userId, achievementId);

      const response: ApiResponse<typeof claimResult> = {
        success: true,
        message: `Achievement claimed! Earned ${claimResult.xpReward} XP`,
        data: claimResult,
        timestamp: new Date().toISOString()
      };

      res.json(response);
    } catch (error) {
      logger.error('Claim achievement error:', error);

      if (error.code === 'ACHIEVEMENT_NOT_COMPLETED') {
        return res.status(400).json({
          success: false,
          message: 'Achievement not yet completed',
          progress: error.progress
        });
      }

      if (error.code === 'ACHIEVEMENT_ALREADY_CLAIMED') {
        return res.status(409).json({
          success: false,
          message: 'Achievement already claimed',
          claimedAt: error.claimedAt
        });
      }

      res.status(500).json({
        success: false,
        message: 'Failed to claim achievement',
        error: process.env.NODE_ENV === 'development' ? error.message : undefined
      });
    }
  }
);

// Error handling middleware
router.use((error: any, req: express.Request, res: express.Response, next: express.NextFunction) => {
  logger.error('XP routes error:', error);
  
  if (error.type === 'entity.parse.failed') {
    return res.status(400).json({
      success: false,
      message: 'Invalid JSON format',
      error: 'Malformed request body'
    });
  }

  res.status(500).json({
    success: false,
    message: 'Internal server error in XP routes',
    error: process.env.NODE_ENV === 'development' ? error.message : undefined,
    timestamp: new Date().toISOString()
  });
});

export default router;
