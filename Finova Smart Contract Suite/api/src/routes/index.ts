/**
 * Finova Network API Routes - Main Index
 * Enterprise-grade routing system for Social-Fi Super App
 * 
 * Features:
 * - Integrated XP, RP, and $FIN mining system
 * - Real-time WebSocket support
 * - Comprehensive security middleware
 * - Rate limiting and DDoS protection
 * - Multi-platform social integration
 * - Indonesian e-wallet support
 */

import express, { Router, Request, Response, NextFunction } from 'express';
import cors from 'cors';
import helmet from 'helmet';
import compression from 'compression';
import rateLimit from 'express-rate-limit';
import { body, query, validationResult } from 'express-validator';

// Import route modules
import authRoutes from './auth.routes';
import userRoutes from './user.routes';
import miningRoutes from './mining.routes';
import xpRoutes from './xp.routes';
import referralRoutes from './referral.routes';
import nftRoutes from './nft.routes';
import socialRoutes from './social.routes';
import adminRoutes from './admin.routes';

// Import middleware
import { authMiddleware } from '../middleware/auth.middleware';
import { kycMiddleware } from '../middleware/kyc.middleware';
import { rateLimitMiddleware } from '../middleware/rate-limit.middleware';
import { validationMiddleware } from '../middleware/validation.middleware';
import { corsMiddleware } from '../middleware/cors.middleware';
import { errorMiddleware } from '../middleware/error.middleware';

// Import services
import { BlockchainService } from '../services/blockchain.service';
import { AnalyticsService } from '../services/analytics.service';
import { NotificationService } from '../services/notification.service';
import { logger } from '../utils/logger';

const router = Router();

// Global middleware stack
router.use(helmet({
  crossOriginEmbedderPolicy: false,
  contentSecurityPolicy: {
    directives: {
      defaultSrc: ["'self'"],
      styleSrc: ["'self'", "'unsafe-inline'"],
      scriptSrc: ["'self'"],
      imgSrc: ["'self'", "data:", "https:"],
      connectSrc: ["'self'", "wss:", "https:"],
    },
  },
}));

router.use(compression());
router.use(corsMiddleware);

// Global rate limiting
const globalRateLimit = rateLimit({
  windowMs: 15 * 60 * 1000, // 15 minutes
  max: 1000, // Limit each IP to 1000 requests per windowMs
  message: {
    error: 'Too many requests from this IP',
    retryAfter: '15 minutes',
    code: 'RATE_LIMIT_EXCEEDED'
  },
  standardHeaders: true,
  legacyHeaders: false,
});

router.use(globalRateLimit);

// Request logging middleware
router.use((req: Request, res: Response, next: NextFunction) => {
  const startTime = Date.now();
  
  res.on('finish', () => {
    const duration = Date.now() - startTime;
    logger.info('API Request', {
      method: req.method,
      path: req.path,
      statusCode: res.statusCode,
      duration: `${duration}ms`,
      userAgent: req.get('User-Agent'),
      ip: req.ip,
      userId: (req as any).user?.id || 'anonymous'
    });

    // Track analytics
    AnalyticsService.trackAPIUsage({
      endpoint: req.path,
      method: req.method,
      statusCode: res.statusCode,
      duration,
      userId: (req as any).user?.id
    }).catch(err => logger.error('Analytics tracking failed', err));
  });

  next();
});

// Health check endpoint
router.get('/health', async (req: Request, res: Response) => {
  try {
    const health = {
      status: 'healthy',
      timestamp: new Date().toISOString(),
      version: process.env.API_VERSION || '1.0.0',
      environment: process.env.NODE_ENV || 'development',
      services: {
        database: await checkDatabaseHealth(),
        blockchain: await checkBlockchainHealth(),
        redis: await checkRedisHealth(),
        websocket: await checkWebSocketHealth()
      },
      metrics: {
        uptime: process.uptime(),
        memory: process.memoryUsage(),
        cpu: process.cpuUsage()
      }
    };

    // Check if any critical service is down
    const criticalServices = ['database', 'blockchain', 'redis'];
    const unhealthyServices = criticalServices.filter(
      service => health.services[service as keyof typeof health.services] !== 'healthy'
    );

    if (unhealthyServices.length > 0) {
      health.status = 'degraded';
      res.status(503);
    }

    res.json(health);
  } catch (error) {
    logger.error('Health check failed', error);
    res.status(503).json({
      status: 'unhealthy',
      error: 'Health check failed',
      timestamp: new Date().toISOString()
    });
  }
});

// System status endpoint (public)
router.get('/status', (req: Request, res: Response) => {
  res.json({
    network: 'Finova Network',
    status: 'operational',
    version: '3.0.0',
    blockchain: 'Solana',
    features: {
      mining: true,
      xpSystem: true,
      referralProgram: true,
      nftMarketplace: true,
      socialIntegration: true,
      ewalletSupport: true
    },
    supportedPlatforms: [
      'Instagram', 'TikTok', 'YouTube', 'Facebook', 'Twitter/X'
    ],
    supportedEwallets: [
      'OVO', 'GoPay', 'Dana', 'ShopeePay', 'LinkAja'
    ]
  });
});

// API documentation endpoint
router.get('/docs', (req: Request, res: Response) => {
  res.json({
    title: 'Finova Network API',
    version: '3.0.0',
    description: 'The Next Generation Social-Fi Super App API',
    baseUrl: process.env.API_BASE_URL || 'https://api.finova.network',
    endpoints: {
      authentication: '/api/v1/auth/*',
      users: '/api/v1/users/*',
      mining: '/api/v1/mining/*',
      xp: '/api/v1/xp/*',
      referrals: '/api/v1/referrals/*',
      nft: '/api/v1/nft/*',
      social: '/api/v1/social/*',
      admin: '/api/v1/admin/*'
    },
    websocket: process.env.WS_URL || 'wss://ws.finova.network',
    rateLimit: '1000 requests per 15 minutes',
    authentication: 'Bearer JWT token required for protected endpoints'
  });
});

// API Statistics endpoint (requires auth)
router.get('/stats', 
  authMiddleware,
  async (req: Request, res: Response) => {
    try {
      const stats = await AnalyticsService.getGlobalStats();
      res.json({
        network: {
          totalUsers: stats.totalUsers,
          activeMiners: stats.activeMiners,
          totalFinMined: stats.totalFinMined,
          totalXPEarned: stats.totalXPEarned,
          referralNetworks: stats.referralNetworks,
          nftsMinted: stats.nftsMinted
        },
        mining: {
          currentPhase: stats.miningPhase,
          baseRate: stats.baseMiningRate,
          totalSupply: stats.totalSupply,
          circulatingSupply: stats.circulatingSupply
        },
        activity: {
          last24h: stats.activity24h,
          last7d: stats.activity7d,
          last30d: stats.activity30d
        },
        platforms: stats.platformBreakdown
      });
    } catch (error) {
      logger.error('Failed to fetch stats', error);
      res.status(500).json({ error: 'Failed to fetch statistics' });
    }
  }
);

// Main API routes with versioning
const API_VERSION = '/v1';

// Authentication routes (public)
router.use(`${API_VERSION}/auth`, authRoutes);

// Protected routes (require authentication)
router.use(`${API_VERSION}/users`, authMiddleware, userRoutes);
router.use(`${API_VERSION}/mining`, authMiddleware, miningRoutes);
router.use(`${API_VERSION}/xp`, authMiddleware, xpRoutes);
router.use(`${API_VERSION}/referrals`, authMiddleware, referralRoutes);
router.use(`${API_VERSION}/nft`, authMiddleware, nftRoutes);
router.use(`${API_VERSION}/social`, authMiddleware, socialRoutes);

// Admin routes (require admin authentication)
router.use(`${API_VERSION}/admin`, 
  authMiddleware,
  kycMiddleware,
  rateLimitMiddleware.adminRateLimit,
  adminRoutes
);

// Integration endpoints for external services
router.post(`${API_VERSION}/webhook/social/:platform`,
  rateLimitMiddleware.webhookRateLimit,
  validationMiddleware.validateWebhook,
  async (req: Request, res: Response) => {
    try {
      const { platform } = req.params;
      const webhookData = req.body;

      logger.info('Webhook received', { platform, data: webhookData });

      // Process social platform webhook
      await processSocialWebhook(platform, webhookData);

      res.json({ success: true, message: 'Webhook processed' });
    } catch (error) {
      logger.error('Webhook processing failed', error);
      res.status(500).json({ error: 'Webhook processing failed' });
    }
  }
);

// Real-time mining rate endpoint
router.get(`${API_VERSION}/mining/rate/current`,
  authMiddleware,
  async (req: Request, res: Response) => {
    try {
      const user = (req as any).user;
      const currentRate = await calculateCurrentMiningRate(user.id);
      
      res.json({
        userId: user.id,
        currentRate: currentRate.hourlyRate,
        dailyProjection: currentRate.dailyProjection,
        bonuses: currentRate.bonuses,
        nextUpdate: currentRate.nextUpdate,
        phase: currentRate.phase
      });
    } catch (error) {
      logger.error('Failed to get mining rate', error);
      res.status(500).json({ error: 'Failed to get current mining rate' });
    }
  }
);

// XP leaderboard endpoint
router.get(`${API_VERSION}/xp/leaderboard`,
  query('limit').optional().isInt({ min: 1, max: 100 }),
  query('period').optional().isIn(['daily', 'weekly', 'monthly', 'all']),
  validationMiddleware,
  async (req: Request, res: Response) => {
    try {
      const limit = parseInt(req.query.limit as string) || 50;
      const period = req.query.period as string || 'weekly';
      
      const leaderboard = await getXPLeaderboard(limit, period);
      
      res.json({
        period,
        limit,
        leaderboard: leaderboard.map((entry, index) => ({
          rank: index + 1,
          userId: entry.userId,
          username: entry.username,
          xp: entry.xp,
          level: entry.level,
          badge: entry.badge,
          avatar: entry.avatar
        }))
      });
    } catch (error) {
      logger.error('Failed to get leaderboard', error);
      res.status(500).json({ error: 'Failed to get leaderboard' });
    }
  }
);

// Referral network visualization endpoint
router.get(`${API_VERSION}/referrals/network/:userId`,
  authMiddleware,
  async (req: Request, res: Response) => {
    try {
      const { userId } = req.params;
      const requestingUser = (req as any).user;
      
      // Check if user can view this network (own network or admin)
      if (userId !== requestingUser.id && !requestingUser.isAdmin) {
        return res.status(403).json({ error: 'Access denied' });
      }
      
      const networkData = await getReferralNetworkData(userId);
      
      res.json({
        userId,
        network: {
          totalReferrals: networkData.totalReferrals,
          activeReferrals: networkData.activeReferrals,
          networkLevels: networkData.levels,
          totalRP: networkData.totalRP,
          tier: networkData.tier
        },
        visualization: {
          nodes: networkData.nodes,
          edges: networkData.edges,
          metrics: networkData.metrics
        }
      });
    } catch (error) {
      logger.error('Failed to get network data', error);
      res.status(500).json({ error: 'Failed to get network data' });
    }
  }
);

// Special cards marketplace endpoint
router.get(`${API_VERSION}/nft/cards/marketplace`,
  query('category').optional().isIn(['mining', 'xp', 'referral', 'special']),
  query('rarity').optional().isIn(['common', 'uncommon', 'rare', 'epic', 'legendary']),
  query('sort').optional().isIn(['price', 'rarity', 'popularity', 'newest']),
  query('limit').optional().isInt({ min: 1, max: 100 }),
  validationMiddleware,
  async (req: Request, res: Response) => {
    try {
      const filters = {
        category: req.query.category as string,
        rarity: req.query.rarity as string,
        sort: req.query.sort as string || 'popularity',
        limit: parseInt(req.query.limit as string) || 20
      };
      
      const cards = await getNFTMarketplace(filters);
      
      res.json({
        filters,
        total: cards.total,
        cards: cards.items.map(card => ({
          id: card.id,
          name: card.name,
          category: card.category,
          rarity: card.rarity,
          effect: card.effect,
          duration: card.duration,
          price: card.price,
          image: card.image,
          supply: card.supply,
          popularity: card.popularity
        }))
      });
    } catch (error) {
      logger.error('Failed to get marketplace', error);
      res.status(500).json({ error: 'Failed to get marketplace data' });
    }
  }
);

// Guild system endpoint
router.get(`${API_VERSION}/guilds`,
  authMiddleware,
  query('search').optional().isString(),
  query('category').optional().isIn(['competitive', 'casual', 'educational', 'regional']),
  validationMiddleware,
  async (req: Request, res: Response) => {
    try {
      const search = req.query.search as string;
      const category = req.query.category as string;
      
      const guilds = await getGuilds({ search, category });
      
      res.json({
        total: guilds.length,
        guilds: guilds.map(guild => ({
          id: guild.id,
          name: guild.name,
          description: guild.description,
          category: guild.category,
          memberCount: guild.memberCount,
          maxMembers: guild.maxMembers,
          level: guild.level,
          badges: guild.badges,
          requirements: guild.requirements,
          isJoinable: guild.isJoinable
        }))
      });
    } catch (error) {
      logger.error('Failed to get guilds', error);
      res.status(500).json({ error: 'Failed to get guilds' });
    }
  }
);

// Error handling for undefined routes
router.use('*', (req: Request, res: Response) => {
  res.status(404).json({
    error: 'Endpoint not found',
    message: `Route ${req.originalUrl} not found`,
    availableRoutes: {
      health: '/health',
      status: '/status',
      docs: '/docs',
      auth: '/v1/auth/*',
      users: '/v1/users/*',
      mining: '/v1/mining/*',
      xp: '/v1/xp/*',
      referrals: '/v1/referrals/*',
      nft: '/v1/nft/*',
      social: '/v1/social/*',
      admin: '/v1/admin/*'
    }
  });
});

// Global error handler
router.use(errorMiddleware);

// Helper functions
async function checkDatabaseHealth(): Promise<string> {
  try {
    // Implement database health check
    return 'healthy';
  } catch (error) {
    return 'unhealthy';
  }
}

async function checkBlockchainHealth(): Promise<string> {
  try {
    const connection = await BlockchainService.getConnection();
    const slot = await connection.getSlot();
    return slot > 0 ? 'healthy' : 'unhealthy';
  } catch (error) {
    return 'unhealthy';
  }
}

async function checkRedisHealth(): Promise<string> {
  try {
    // Implement Redis health check
    return 'healthy';
  } catch (error) {
    return 'unhealthy';
  }
}

async function checkWebSocketHealth(): Promise<string> {
  try {
    // Implement WebSocket health check
    return 'healthy';
  } catch (error) {
    return 'unhealthy';
  }
}

async function processSocialWebhook(platform: string, data: any) {
  // Implement social platform webhook processing
  logger.info(`Processing ${platform} webhook`, data);
}

async function calculateCurrentMiningRate(userId: string) {
  // Implement real-time mining rate calculation
  return {
    hourlyRate: 0.05,
    dailyProjection: 1.2,
    bonuses: {},
    nextUpdate: new Date(),
    phase: 'growth'
  };
}

async function getXPLeaderboard(limit: number, period: string) {
  // Implement XP leaderboard logic
  return [];
}

async function getReferralNetworkData(userId: string) {
  // Implement referral network visualization data
  return {
    totalReferrals: 0,
    activeReferrals: 0,
    levels: [],
    totalRP: 0,
    tier: 'Explorer',
    nodes: [],
    edges: [],
    metrics: {}
  };
}

async function getNFTMarketplace(filters: any) {
  // Implement NFT marketplace logic
  return {
    total: 0,
    items: []
  };
}

async function getGuilds(filters: any) {
  // Implement guild system logic
  return [];
}

export default router;
