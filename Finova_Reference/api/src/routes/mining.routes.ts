import { Router, Request, Response, NextFunction } from 'express';
import rateLimit from 'express-rate-limit';
import { body, query, param, validationResult } from 'express-validator';
import { authMiddleware } from '../middleware/auth.middleware';
import { kycMiddleware } from '../middleware/kyc.middleware';
import { antiBotMiddleware } from '../middleware/anti-bot.middleware';
import { validationMiddleware } from '../middleware/validation.middleware';
import { miningService } from '../services/mining.service';
import { userService } from '../services/user.service';
import { xpService } from '../services/xp.service';
import { referralService } from '../services/referral.service';
import { blockchainService } from '../services/blockchain.service';
import { analyticsService } from '../services/analytics.service';
import { logger } from '../utils/logger';
import { ApiResponse, MiningSession, MiningStats, MiningBoost } from '../types/mining.types';
import { RedisClient } from '../config/redis';
import { WebSocketManager } from '../websocket';

const router = Router();
const redis = RedisClient.getInstance();
const wsManager = WebSocketManager.getInstance();

// Rate limiting for mining operations
const miningRateLimit = rateLimit({
  windowMs: 60 * 1000, // 1 minute
  max: 10, // 10 requests per minute per IP
  message: { error: 'Too many mining requests, please try again later' },
  standardHeaders: true,
  legacyHeaders: false,
});

// Heavy operation rate limiting
const heavyOperationLimit = rateLimit({
  windowMs: 5 * 60 * 1000, // 5 minutes
  max: 3, // 3 requests per 5 minutes
  message: { error: 'Rate limit exceeded for heavy operations' },
});

/**
 * @route GET /api/mining/status
 * @desc Get current mining status for authenticated user
 * @access Private
 */
router.get('/status',
  authMiddleware,
  antiBotMiddleware,
  async (req: Request, res: Response, next: NextFunction) => {
    try {
      const userId = req.user?.id;
      if (!userId) {
        return res.status(401).json({ error: 'User not authenticated' });
      }

      // Get cached mining status first
      const cacheKey = `mining:status:${userId}`;
      const cachedStatus = await redis.get(cacheKey);
      
      if (cachedStatus) {
        return res.json(JSON.parse(cachedStatus));
      }

      // Calculate real-time mining status
      const [
        user,
        miningSession,
        xpData,
        rpData,
        activeBoosts
      ] = await Promise.all([
        userService.getUserById(userId),
        miningService.getCurrentSession(userId),
        xpService.getUserXP(userId),
        referralService.getUserRP(userId),
        miningService.getActiveBoosts(userId)
      ]);

      if (!user) {
        return res.status(404).json({ error: 'User not found' });
      }

      // Calculate integrated mining rate using whitepaper formula
      const miningRate = miningService.calculateIntegratedMiningRate({
        userId,
        baseRate: miningSession?.baseRate || 0.05,
        xpLevel: xpData.level,
        xpMultiplier: xpService.calculateXPMultiplier(xpData.level),
        rpTier: rpData.tier,
        rpMultiplier: referralService.calculateRPMultiplier(rpData.tier),
        qualityScore: user.qualityScore || 1.0,
        networkRegression: miningService.calculateNetworkRegression(user.totalHoldings),
        securityBonus: user.isKYCVerified ? 1.2 : 0.8,
        activeBoosts
      });

      // Calculate accumulated rewards
      const accumulatedRewards = miningService.calculateAccumulatedRewards(
        miningSession,
        miningRate,
        new Date()
      );

      const response: ApiResponse<MiningStats> = {
        success: true,
        data: {
          isActive: miningSession?.isActive || false,
          currentRate: miningRate.finalRate,
          baseRate: miningRate.baseRate,
          multipliers: {
            xp: miningRate.xpMultiplier,
            rp: miningRate.rpMultiplier,
            quality: miningRate.qualityScore,
            security: miningRate.securityBonus,
            regression: miningRate.networkRegression
          },
          accumulated: {
            total: accumulatedRewards.total,
            fromMining: accumulatedRewards.fromMining,
            fromXP: accumulatedRewards.fromXP,
            fromRP: accumulatedRewards.fromRP
          },
          session: {
            startTime: miningSession?.startTime,
            lastUpdate: miningSession?.lastUpdate,
            duration: miningSession?.duration || 0
          },
          dailyStats: {
            earned: user.dailyEarned || 0,
            cap: miningService.getDailyCap(xpData.level),
            remaining: Math.max(0, miningService.getDailyCap(xpData.level) - (user.dailyEarned || 0))
          },
          activeBoosts: activeBoosts.map(boost => ({
            type: boost.type,
            multiplier: boost.multiplier,
            duration: boost.duration,
            remaining: boost.remaining
          })),
          nextPhaseInfo: miningService.getNextPhaseInfo()
        }
      };

      // Cache for 30 seconds
      await redis.setex(cacheKey, 30, JSON.stringify(response));

      res.json(response);
    } catch (error) {
      logger.error('Error getting mining status:', error);
      next(error);
    }
  }
);

/**
 * @route POST /api/mining/start
 * @desc Start mining session
 * @access Private
 */
router.post('/start',
  miningRateLimit,
  authMiddleware,
  kycMiddleware,
  antiBotMiddleware,
  async (req: Request, res: Response, next: NextFunction) => {
    try {
      const userId = req.user?.id;
      if (!userId) {
        return res.status(401).json({ error: 'User not authenticated' });
      }

      // Check if already mining
      const existingSession = await miningService.getCurrentSession(userId);
      if (existingSession?.isActive) {
        return res.status(400).json({ 
          error: 'Mining session already active',
          data: existingSession
        });
      }

      // Anti-bot validation
      const humanProbability = await miningService.calculateHumanProbability(userId);
      if (humanProbability < 0.7) {
        logger.warn(`Low human probability for user ${userId}: ${humanProbability}`);
        return res.status(403).json({ 
          error: 'Account under review for suspicious activity' 
        });
      }

      // Check daily mining cap
      const user = await userService.getUserById(userId);
      const dailyCap = miningService.getDailyCap(user?.xpLevel || 1);
      
      if ((user?.dailyEarned || 0) >= dailyCap) {
        return res.status(400).json({ 
          error: 'Daily mining cap reached',
          data: { dailyEarned: user?.dailyEarned, dailyCap }
        });
      }

      // Start mining session
      const session = await miningService.startMiningSession(userId, {
        humanProbability,
        deviceFingerprint: req.headers['x-device-fingerprint'] as string,
        ipAddress: req.ip,
        userAgent: req.headers['user-agent']
      });

      // Real-time update via WebSocket
      wsManager.sendToUser(userId, 'mining:started', {
        sessionId: session.id,
        startTime: session.startTime,
        estimatedRate: session.estimatedRate
      });

      // Track analytics
      analyticsService.track('mining_started', {
        userId,
        sessionId: session.id,
        estimatedRate: session.estimatedRate,
        humanProbability
      });

      res.json({
        success: true,
        message: 'Mining session started successfully',
        data: session
      });
    } catch (error) {
      logger.error('Error starting mining:', error);
      next(error);
    }
  }
);

/**
 * @route POST /api/mining/claim
 * @desc Claim accumulated mining rewards
 * @access Private
 */
router.post('/claim',
  heavyOperationLimit,
  authMiddleware,
  kycMiddleware,
  antiBotMiddleware,
  async (req: Request, res: Response, next: NextFunction) => {
    try {
      const userId = req.user?.id;
      if (!userId) {
        return res.status(401).json({ error: 'User not authenticated' });
      }

      const session = await miningService.getCurrentSession(userId);
      if (!session?.isActive) {
        return res.status(400).json({ error: 'No active mining session' });
      }

      // Calculate final rewards with all multipliers
      const rewards = await miningService.calculateFinalRewards(userId, session);
      
      if (rewards.total <= 0) {
        return res.status(400).json({ 
          error: 'No rewards to claim',
          data: { accumulated: rewards.total }
        });
      }

      // Blockchain transaction for token minting
      const transaction = await blockchainService.mintTokens({
        userId,
        amount: rewards.total,
        breakdown: {
          baseMining: rewards.fromMining,
          xpBonus: rewards.fromXP,
          rpBonus: rewards.fromRP,
          qualityMultiplier: rewards.qualityMultiplier
        },
        sessionId: session.id,
        metadata: {
          claimTime: new Date(),
          miningDuration: session.duration,
          averageRate: rewards.total / (session.duration / 3600) // per hour
        }
      });

      // Update user stats
      await Promise.all([
        userService.updateUserStats(userId, {
          totalEarned: rewards.total,
          dailyEarned: rewards.total,
          lastClaimTime: new Date(),
          transactionHash: transaction.hash
        }),
        miningService.endSession(session.id, rewards),
        // Update XP for mining activity
        xpService.addXP(userId, Math.floor(rewards.total * 10), 'mining_claim'),
        // Update RP for network activity
        referralService.distributeNetworkRewards(userId, rewards.total * 0.1)
      ]);

      // Clear cache
      await redis.del(`mining:status:${userId}`);

      // Real-time notifications
      wsManager.sendToUser(userId, 'mining:claimed', {
        amount: rewards.total,
        breakdown: rewards,
        transactionHash: transaction.hash,
        newBalance: await userService.getUserBalance(userId)
      });

      // Notify referral network
      const referrals = await referralService.getActiveReferrals(userId);
      for (const referral of referrals) {
        wsManager.sendToUser(referral.id, 'referral:earning', {
          referrerId: userId,
          amount: rewards.total * referral.commission,
          source: 'mining_claim'
        });
      }

      // Analytics tracking
      analyticsService.track('mining_claimed', {
        userId,
        amount: rewards.total,
        breakdown: rewards,
        sessionDuration: session.duration,
        transactionHash: transaction.hash
      });

      res.json({
        success: true,
        message: 'Rewards claimed successfully',
        data: {
          claimed: rewards.total,
          breakdown: rewards,
          transactionHash: transaction.hash,
          session: {
            id: session.id,
            duration: session.duration,
            averageRate: rewards.total / (session.duration / 3600)
          }
        }
      });
    } catch (error) {
      logger.error('Error claiming mining rewards:', error);
      next(error);
    }
  }
);

/**
 * @route GET /api/mining/history
 * @desc Get mining history for user
 * @access Private
 */
router.get('/history',
  authMiddleware,
  [
    query('page').optional().isInt({ min: 1 }),
    query('limit').optional().isInt({ min: 1, max: 100 }),
    query('startDate').optional().isISO8601(),
    query('endDate').optional().isISO8601(),
  ],
  validationMiddleware,
  async (req: Request, res: Response, next: NextFunction) => {
    try {
      const userId = req.user?.id;
      const page = parseInt(req.query.page as string) || 1;
      const limit = parseInt(req.query.limit as string) || 20;
      const startDate = req.query.startDate as string;
      const endDate = req.query.endDate as string;

      const history = await miningService.getMiningHistory(userId!, {
        page,
        limit,
        startDate: startDate ? new Date(startDate) : undefined,
        endDate: endDate ? new Date(endDate) : undefined
      });

      // Calculate summary statistics
      const summary = await miningService.getMiningStats(userId!, {
        startDate: startDate ? new Date(startDate) : undefined,
        endDate: endDate ? new Date(endDate) : undefined
      });

      res.json({
        success: true,
        data: {
          history: history.data,
          pagination: {
            page,
            limit,
            total: history.total,
            pages: Math.ceil(history.total / limit)
          },
          summary: {
            totalSessions: summary.totalSessions,
            totalEarned: summary.totalEarned,
            averageSession: summary.averageSessionDuration,
            averageRate: summary.averageHourlyRate,
            bestDay: summary.bestDay,
            streak: summary.currentStreak
          }
        }
      });
    } catch (error) {
      logger.error('Error getting mining history:', error);
      next(error);
    }
  }
);

/**
 * @route POST /api/mining/boost/activate
 * @desc Activate mining boost card
 * @access Private
 */
router.post('/boost/activate',
  miningRateLimit,
  authMiddleware,
  [
    body('cardId').isString().notEmpty(),
    body('boostType').isIn(['mining', 'xp', 'rp', 'quality']),
  ],
  validationMiddleware,
  async (req: Request, res: Response, next: NextFunction) => {
    try {
      const userId = req.user?.id;
      const { cardId, boostType } = req.body;

      // Verify card ownership
      const card = await miningService.verifyBoostCard(userId!, cardId);
      if (!card) {
        return res.status(404).json({ error: 'Boost card not found or not owned' });
      }

      if (card.used) {
        return res.status(400).json({ error: 'Boost card already used' });
      }

      // Check boost compatibility
      const compatibility = miningService.checkBoostCompatibility(card, boostType);
      if (!compatibility.compatible) {
        return res.status(400).json({ 
          error: 'Incompatible boost type',
          details: compatibility.reason
        });
      }

      // Activate boost
      const boost = await miningService.activateBoost(userId!, {
        cardId,
        boostType,
        multiplier: card.multiplier,
        duration: card.duration,
        metadata: card.metadata
      });

      // Update card status (single-use cards are marked as used)
      if (card.singleUse) {
        await miningService.markCardAsUsed(cardId);
      }

      // Real-time notification
      wsManager.sendToUser(userId!, 'mining:boost_activated', {
        boostId: boost.id,
        type: boostType,
        multiplier: card.multiplier,
        duration: card.duration,
        remaining: card.duration
      });

      // Track analytics
      analyticsService.track('boost_activated', {
        userId,
        cardId,
        boostType,
        multiplier: card.multiplier,
        duration: card.duration
      });

      res.json({
        success: true,
        message: 'Boost activated successfully',
        data: boost
      });
    } catch (error) {
      logger.error('Error activating boost:', error);
      next(error);
    }
  }
);

/**
 * @route GET /api/mining/leaderboard
 * @desc Get mining leaderboard
 * @access Public
 */
router.get('/leaderboard',
  rateLimit({
    windowMs: 60 * 1000,
    max: 30,
    message: { error: 'Too many leaderboard requests' }
  }),
  [
    query('period').optional().isIn(['daily', 'weekly', 'monthly', 'all-time']),
    query('category').optional().isIn(['total', 'rate', 'consistency', 'network']),
    query('limit').optional().isInt({ min: 10, max: 100 }),
  ],
  validationMiddleware,
  async (req: Request, res: Response, next: NextFunction) => {
    try {
      const period = (req.query.period as string) || 'weekly';
      const category = (req.query.category as string) || 'total';
      const limit = parseInt(req.query.limit as string) || 50;

      // Check cache first
      const cacheKey = `leaderboard:${period}:${category}:${limit}`;
      const cached = await redis.get(cacheKey);
      
      if (cached) {
        return res.json(JSON.parse(cached));
      }

      const leaderboard = await miningService.getLeaderboard({
        period,
        category,
        limit,
        includeStats: true
      });

      // Privacy protection - only show limited user info
      const sanitized = leaderboard.map((entry, index) => ({
        rank: index + 1,
        username: entry.user.username || `User${entry.user.id.slice(-4)}`,
        avatar: entry.user.avatar,
        level: entry.user.xpLevel,
        badge: entry.user.badge,
        stats: {
          [category]: entry.value,
          streak: entry.streak,
          efficiency: entry.efficiency
        },
        isVerified: entry.user.isKYCVerified,
        networkSize: entry.user.referralCount
      }));

      const response = {
        success: true,
        data: {
          leaderboard: sanitized,
          meta: {
            period,
            category,
            total: leaderboard.length,
            lastUpdated: new Date()
          }
        }
      };

      // Cache for 5 minutes
      await redis.setex(cacheKey, 300, JSON.stringify(response));

      res.json(response);
    } catch (error) {
      logger.error('Error getting leaderboard:', error);
      next(error);
    }
  }
);

/**
 * @route GET /api/mining/analytics
 * @desc Get mining analytics for user
 * @access Private
 */
router.get('/analytics',
  authMiddleware,
  [
    query('timeframe').optional().isIn(['7d', '30d', '90d', '1y']),
    query('granularity').optional().isIn(['hour', 'day', 'week', 'month']),
  ],
  validationMiddleware,
  async (req: Request, res: Response, next: NextFunction) => {
    try {
      const userId = req.user?.id;
      const timeframe = (req.query.timeframe as string) || '30d';
      const granularity = (req.query.granularity as string) || 'day';

      const analytics = await miningService.getUserAnalytics(userId!, {
        timeframe,
        granularity
      });

      res.json({
        success: true,
        data: {
          timeline: analytics.timeline,
          summary: {
            totalEarned: analytics.totalEarned,
            averageRate: analytics.averageRate,
            bestPeriod: analytics.bestPeriod,
            consistency: analytics.consistencyScore,
            efficiency: analytics.efficiencyScore
          },
          insights: {
            recommendations: analytics.recommendations,
            patterns: analytics.patterns,
            projections: analytics.projections
          },
          comparisons: {
            vsAverage: analytics.vsAverageUser,
            vsLevel: analytics.vsLevelPeers,
            ranking: analytics.globalRanking
          }
        }
      });
    } catch (error) {
      logger.error('Error getting mining analytics:', error);
      next(error);
    }
  }
);

/**
 * @route POST /api/mining/report
 * @desc Report suspicious mining activity
 * @access Private
 */
router.post('/report',
  heavyOperationLimit,
  authMiddleware,
  [
    body('reportedUserId').isString().notEmpty(),
    body('reason').isIn(['bot_activity', 'fake_engagement', 'network_manipulation', 'other']),
    body('description').isString().isLength({ min: 10, max: 1000 }),
    body('evidence').optional().isArray(),
  ],
  validationMiddleware,
  async (req: Request, res: Response, next: NextFunction) => {
    try {
      const reporterId = req.user?.id;
      const { reportedUserId, reason, description, evidence } = req.body;

      if (reporterId === reportedUserId) {
        return res.status(400).json({ error: 'Cannot report yourself' });
      }

      // Check if user exists
      const reportedUser = await userService.getUserById(reportedUserId);
      if (!reportedUser) {
        return res.status(404).json({ error: 'Reported user not found' });
      }

      // Create report
      const report = await miningService.createSuspiciousActivityReport({
        reporterId,
        reportedUserId,
        reason,
        description,
        evidence: evidence || [],
        metadata: {
          reporterIP: req.ip,
          timestamp: new Date()
        }
      });

      // Trigger automated analysis
      miningService.triggerSuspiciousActivityAnalysis(reportedUserId);

      logger.info(`Suspicious activity reported: ${reporterId} reported ${reportedUserId} for ${reason}`);

      res.json({
        success: true,
        message: 'Report submitted successfully',
        data: {
          reportId: report.id,
          status: 'pending_review'
        }
      });
    } catch (error) {
      logger.error('Error submitting report:', error);
      next(error);
    }
  }
);

// Error handling middleware
router.use((error: any, req: Request, res: Response, next: NextFunction) => {
  logger.error('Mining routes error:', error);
  
  res.status(error.status || 500).json({
    success: false,
    error: error.message || 'Internal server error',
    ...(process.env.NODE_ENV === 'development' && { stack: error.stack })
  });
});

export default router;
