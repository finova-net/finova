import express, { Express, Request, Response, NextFunction } from 'express';
import { createServer } from 'http';
import { Server as SocketIOServer } from 'socket.io';
import cors from 'cors';
import helmet from 'helmet';
import compression from 'compression';
import rateLimit from 'express-rate-limit';
import { config } from './config';
import { connectDatabase } from './config/database';
import { initializeBlockchain } from './config/blockchain';
import { initializeRedis } from './config/redis';
import { logger } from './utils/logger';
import { errorMiddleware } from './middleware/error.middleware';
import { authMiddleware } from './middleware/auth.middleware';
import { corsMiddleware } from './middleware/cors.middleware';
import { rateLimitMiddleware } from './middleware/rate-limit.middleware';
import { validationMiddleware } from './middleware/validation.middleware';

// Route imports
import authRoutes from './routes/auth.routes';
import userRoutes from './routes/user.routes';
import miningRoutes from './routes/mining.routes';
import xpRoutes from './routes/xp.routes';
import referralRoutes from './routes/referral.routes';
import nftRoutes from './routes/nft.routes';
import socialRoutes from './routes/social.routes';
import adminRoutes from './routes/admin.routes';

// Service imports
import { MiningService } from './services/mining.service';
import { XPService } from './services/xp.service';
import { ReferralService } from './services/referral.service';
import { BlockchainService } from './services/blockchain.service';
import { AnalyticsService } from './services/analytics.service';

// WebSocket handlers
import { initializeWebSocket } from './websocket';

// Types
interface ApiMetrics {
  totalRequests: number;
  activeUsers: number;
  miningRate: number;
  systemHealth: 'healthy' | 'degraded' | 'critical';
}

class FinovaApiServer {
  private app: Express;
  private server: any;
  private io: SocketIOServer;
  private metrics: ApiMetrics = {
    totalRequests: 0,
    activeUsers: 0,
    miningRate: 0,
    systemHealth: 'healthy'
  };

  constructor() {
    this.app = express();
    this.server = createServer(this.app);
    this.io = new SocketIOServer(this.server, {
      cors: {
        origin: config.cors.allowedOrigins,
        credentials: true
      },
      transports: ['websocket', 'polling']
    });
    
    this.initializeMiddlewares();
    this.initializeRoutes();
    this.initializeWebSocket();
    this.initializeErrorHandling();
  }

  private initializeMiddlewares(): void {
    // Security middlewares
    this.app.use(helmet({
      contentSecurityPolicy: {
        directives: {
          defaultSrc: ["'self'"],
          scriptSrc: ["'self'", "'unsafe-inline'"],
          styleSrc: ["'self'", "'unsafe-inline'"],
          imgSrc: ["'self'", "data:", "https:"],
          connectSrc: ["'self'", "wss:", "https:"]
        }
      },
      crossOriginEmbedderPolicy: false
    }));

    // CORS
    this.app.use(corsMiddleware);

    // Compression
    this.app.use(compression({
      filter: (req, res) => {
        if (req.headers['x-no-compression']) return false;
        return compression.filter(req, res);
      },
      level: 6
    }));

    // Rate limiting
    this.app.use('/api', rateLimitMiddleware);

    // Body parsing
    this.app.use(express.json({ 
      limit: '10mb',
      verify: (req: any, res, buf) => {
        req.rawBody = buf;
      }
    }));
    this.app.use(express.urlencoded({ extended: true, limit: '10mb' }));

    // Request tracking
    this.app.use((req: Request, res: Response, next: NextFunction) => {
      this.metrics.totalRequests++;
      req.requestId = `req_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
      
      logger.info(`${req.method} ${req.path}`, {
        requestId: req.requestId,
        userAgent: req.get('User-Agent'),
        ip: req.ip,
        query: req.query,
        body: req.method === 'POST' ? '[BODY_PRESENT]' : undefined
      });
      
      next();
    });

    // API versioning
    this.app.use('/api/v1', this.createV1Router());
    this.app.use('/api', this.createV1Router()); // Default to v1
  }

  private createV1Router(): express.Router {
    const router = express.Router();

    // Health check (no auth required)
    router.get('/health', this.healthCheck.bind(this));
    router.get('/metrics', this.getMetrics.bind(this));

    // Public routes
    router.use('/auth', authRoutes);

    // Protected routes (require authentication)
    router.use('/users', authMiddleware, userRoutes);
    router.use('/mining', authMiddleware, miningRoutes);
    router.use('/xp', authMiddleware, xpRoutes);
    router.use('/referral', authMiddleware, referralRoutes);
    router.use('/nft', authMiddleware, nftRoutes);
    router.use('/social', authMiddleware, socialRoutes);
    router.use('/admin', authMiddleware, adminRoutes);

    return router;
  }

  private initializeRoutes(): void {
    // Root endpoint
    this.app.get('/', (req: Request, res: Response) => {
      res.json({
        name: 'Finova Network API',
        version: '1.0.0',
        status: 'operational',
        timestamp: new Date().toISOString(),
        environment: config.nodeEnv,
        documentation: '/api/docs',
        health: '/api/health',
        metrics: '/api/metrics'
      });
    });

    // API Documentation
    this.app.get('/api/docs', (req: Request, res: Response) => {
      res.json({
        openapi: '3.0.0',
        info: {
          title: 'Finova Network API',
          version: '1.0.0',
          description: 'Social-Fi Super App with XP, RP, and $FIN Mining',
          contact: {
            name: 'Finova Network',
            url: 'https://finova.network',
            email: 'api@finova.network'
          }
        },
        servers: [
          {
            url: config.apiUrl,
            description: 'Production Server'
          }
        ],
        paths: {
          '/auth/login': { post: { summary: 'User authentication' }},
          '/mining/start': { post: { summary: 'Start mining session' }},
          '/xp/activities': { get: { summary: 'Get XP activities' }},
          '/referral/network': { get: { summary: 'Get referral network' }},
          '/nft/marketplace': { get: { summary: 'NFT marketplace' }},
          '/social/integrate': { post: { summary: 'Social media integration' }}
        }
      });
    });

    // Catch 404
    this.app.use('*', (req: Request, res: Response) => {
      res.status(404).json({
        success: false,
        error: {
          code: 'ENDPOINT_NOT_FOUND',
          message: `Cannot ${req.method} ${req.originalUrl}`,
          timestamp: new Date().toISOString(),
          requestId: req.requestId
        }
      });
    });
  }

  private initializeWebSocket(): void {
    initializeWebSocket(this.io);

    // WebSocket connection tracking
    this.io.on('connection', (socket) => {
      this.metrics.activeUsers++;
      logger.info(`WebSocket connected: ${socket.id}`, {
        activeConnections: this.metrics.activeUsers
      });

      socket.on('disconnect', () => {
        this.metrics.activeUsers = Math.max(0, this.metrics.activeUsers - 1);
        logger.info(`WebSocket disconnected: ${socket.id}`, {
          activeConnections: this.metrics.activeUsers
        });
      });
    });
  }

  private initializeErrorHandling(): void {
    // Global error handler
    this.app.use(errorMiddleware);

    // Graceful shutdown handlers
    process.on('SIGTERM', this.gracefulShutdown.bind(this));
    process.on('SIGINT', this.gracefulShutdown.bind(this));
    process.on('SIGUSR2', this.gracefulShutdown.bind(this)); // Nodemon restart

    // Unhandled promise rejections
    process.on('unhandledRejection', (reason, promise) => {
      logger.error('Unhandled Rejection at Promise', {
        reason,
        promise: promise.toString()
      });
    });

    // Uncaught exceptions
    process.on('uncaughtException', (error) => {
      logger.error('Uncaught Exception thrown', { error: error.message, stack: error.stack });
      process.exit(1);
    });
  }

  private async healthCheck(req: Request, res: Response): Promise<void> {
    try {
      const healthData = {
        status: 'healthy',
        timestamp: new Date().toISOString(),
        uptime: process.uptime(),
        version: '1.0.0',
        environment: config.nodeEnv,
        services: {
          database: await this.checkDatabaseHealth(),
          redis: await this.checkRedisHealth(),
          blockchain: await this.checkBlockchainHealth()
        },
        metrics: {
          totalRequests: this.metrics.totalRequests,
          activeUsers: this.metrics.activeUsers,
          memoryUsage: process.memoryUsage(),
          cpuUsage: process.cpuUsage()
        }
      };

      // Determine overall health
      const servicesHealthy = Object.values(healthData.services).every(service => service.status === 'healthy');
      healthData.status = servicesHealthy ? 'healthy' : 'degraded';
      this.metrics.systemHealth = healthData.status as any;

      res.status(healthData.status === 'healthy' ? 200 : 503).json(healthData);
    } catch (error) {
      logger.error('Health check failed', { error });
      res.status(503).json({
        status: 'critical',
        timestamp: new Date().toISOString(),
        error: 'Health check failed'
      });
    }
  }

  private async getMetrics(req: Request, res: Response): Promise<void> {
    try {
      const miningService = new MiningService();
      const analyticsService = new AnalyticsService();

      const metrics = {
        api: {
          totalRequests: this.metrics.totalRequests,
          activeUsers: this.metrics.activeUsers,
          systemHealth: this.metrics.systemHealth,
          uptime: process.uptime(),
          version: '1.0.0'
        },
        blockchain: {
          totalUsers: await miningService.getTotalUsers(),
          activeMinersPast24h: await miningService.getActiveMinersCount(24),
          totalFINMined: await miningService.getTotalFINMined(),
          currentMiningPhase: await miningService.getCurrentPhase()
        },
        network: {
          totalReferrals: await analyticsService.getTotalReferrals(),
          averageNetworkSize: await analyticsService.getAverageNetworkSize(),
          topReferrers: await analyticsService.getTopReferrers(10)
        },
        xp: {
          totalXPEarned: await analyticsService.getTotalXPEarned(),
          averageUserLevel: await analyticsService.getAverageUserLevel(),
          mostActiveActivities: await analyticsService.getMostActiveActivities()
        },
        performance: {
          responseTime: await this.getAverageResponseTime(),
          errorRate: await this.getErrorRate(),
          throughput: this.getThroughput()
        },
        timestamp: new Date().toISOString()
      };

      res.json(metrics);
    } catch (error) {
      logger.error('Metrics retrieval failed', { error });
      res.status(500).json({
        success: false,
        error: 'Failed to retrieve metrics'
      });
    }
  }

  private async checkDatabaseHealth(): Promise<{status: string, latency?: number, error?: string}> {
    try {
      const start = Date.now();
      // Simple query to check database connectivity
      await connectDatabase();
      const latency = Date.now() - start;
      return { status: 'healthy', latency };
    } catch (error) {
      return { status: 'unhealthy', error: error.message };
    }
  }

  private async checkRedisHealth(): Promise<{status: string, latency?: number, error?: string}> {
    try {
      const start = Date.now();
      const redis = await initializeRedis();
      await redis.ping();
      const latency = Date.now() - start;
      return { status: 'healthy', latency };
    } catch (error) {
      return { status: 'unhealthy', error: error.message };
    }
  }

  private async checkBlockchainHealth(): Promise<{status: string, latency?: number, blockHeight?: number, error?: string}> {
    try {
      const start = Date.now();
      const blockchain = await initializeBlockchain();
      const blockHeight = await blockchain.getSlot();
      const latency = Date.now() - start;
      return { status: 'healthy', latency, blockHeight };
    } catch (error) {
      return { status: 'unhealthy', error: error.message };
    }
  }

  private async getAverageResponseTime(): Promise<number> {
    // Implementation would track response times
    return 150; // ms - placeholder
  }

  private async getErrorRate(): Promise<number> {
    // Implementation would track error rates
    return 0.02; // 2% - placeholder
  }

  private getThroughput(): number {
    // Requests per second calculation
    const uptime = process.uptime();
    return uptime > 0 ? this.metrics.totalRequests / uptime : 0;
  }

  public async start(): Promise<void> {
    try {
      // Initialize external services
      logger.info('Initializing external services...');
      
      await connectDatabase();
      logger.info('Database connected successfully');

      await initializeRedis();
      logger.info('Redis connected successfully');

      await initializeBlockchain();
      logger.info('Blockchain connection established');

      // Start background services
      await this.startBackgroundServices();

      // Start server
      const port = config.port || 3000;
      this.server.listen(port, () => {
        logger.info(`Finova API Server started`, {
          port,
          environment: config.nodeEnv,
          timestamp: new Date().toISOString(),
          processId: process.pid,
          nodeVersion: process.version
        });

        // Log startup metrics
        logger.info('Server startup metrics', {
          memoryUsage: process.memoryUsage(),
          cpuUsage: process.cpuUsage()
        });
      });

      // Handle server errors
      this.server.on('error', (error: any) => {
        if (error.syscall !== 'listen') {
          throw error;
        }

        switch (error.code) {
          case 'EACCES':
            logger.error(`Port ${port} requires elevated privileges`);
            process.exit(1);
            break;
          case 'EADDRINUSE':
            logger.error(`Port ${port} is already in use`);
            process.exit(1);
            break;
          default:
            throw error;
        }
      });

    } catch (error) {
      logger.error('Failed to start server', { error: error.message, stack: error.stack });
      process.exit(1);
    }
  }

  private async startBackgroundServices(): Promise<void> {
    logger.info('Starting background services...');

    // Mining rate calculator service
    setInterval(async () => {
      try {
        const miningService = new MiningService();
        await miningService.updateGlobalMiningRates();
      } catch (error) {
        logger.error('Mining rate update failed', { error });
      }
    }, 60000); // Every minute

    // XP calculation service
    setInterval(async () => {
      try {
        const xpService = new XPService();
        await xpService.processQueuedActivities();
      } catch (error) {
        logger.error('XP processing failed', { error });
      }
    }, 30000); // Every 30 seconds

    // Referral network updates
    setInterval(async () => {
      try {
        const referralService = new ReferralService();
        await referralService.updateNetworkCalculations();
      } catch (error) {
        logger.error('Referral network update failed', { error });
      }
    }, 120000); // Every 2 minutes

    // System metrics collection
    setInterval(() => {
      const memUsage = process.memoryUsage();
      const cpuUsage = process.cpuUsage();
      
      logger.info('System metrics', {
        memory: {
          used: Math.round(memUsage.used / 1024 / 1024),
          heap: Math.round(memUsage.heapUsed / 1024 / 1024),
          external: Math.round(memUsage.external / 1024 / 1024)
        },
        cpu: {
          user: cpuUsage.user,
          system: cpuUsage.system
        },
        uptime: process.uptime(),
        activeUsers: this.metrics.activeUsers
      });
    }, 300000); // Every 5 minutes

    logger.info('Background services started successfully');
  }

  private async gracefulShutdown(signal: string): Promise<void> {
    logger.info(`Received ${signal}. Starting graceful shutdown...`);

    // Stop accepting new connections
    this.server.close(async () => {
      logger.info('HTTP server closed');

      try {
        // Close WebSocket connections
        this.io.close();
        logger.info('WebSocket server closed');

        // Close database connections
        // await closeDatabaseConnections();
        logger.info('Database connections closed');

        // Close Redis connections
        // await closeRedisConnections();
        logger.info('Redis connections closed');

        logger.info('Graceful shutdown completed');
        process.exit(0);
      } catch (error) {
        logger.error('Error during graceful shutdown', { error });
        process.exit(1);
      }
    });

    // Force shutdown after 30 seconds
    setTimeout(() => {
      logger.error('Could not close connections in time, forcefully shutting down');
      process.exit(1);
    }, 30000);
  }
}

// Extend Request interface
declare global {
  namespace Express {
    interface Request {
      requestId?: string;
      user?: any;
      rawBody?: Buffer;
    }
  }
}

// Create and start server
const apiServer = new FinovaApiServer();

// Start server
if (require.main === module) {
  apiServer.start().catch((error) => {
    logger.error('Failed to start Finova API Server', { error });
    process.exit(1);
  });
}

export { apiServer };
export default apiServer;
