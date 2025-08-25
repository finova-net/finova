import express from 'express';
import cors from 'cors';
import helmet from 'helmet';
import rateLimit from 'express-rate-limit';
import compression from 'compression';
import morgan from 'morgan';
import { createServer } from 'http';
import { Server } from 'socket.io';
import swaggerUi from 'swagger-ui-express';
import { config } from './config';
import { connectDatabase } from './config/database';
import { connectRedis } from './config/redis';
import { initializeBlockchain } from './config/blockchain';
import logger from './utils/logger';
import { errorHandler } from './middleware/error.middleware';
import { authMiddleware } from './middleware/auth.middleware';
import { corsMiddleware } from './middleware/cors.middleware';
import { rateLimitMiddleware } from './middleware/rate-limit.middleware';
import { validationMiddleware } from './middleware/validation.middleware';
import { kycMiddleware } from './middleware/kyc.middleware';

// Routes
import authRoutes from './routes/auth.routes';
import userRoutes from './routes/user.routes';
import miningRoutes from './routes/mining.routes';
import xpRoutes from './routes/xp.routes';
import referralRoutes from './routes/referral.routes';
import nftRoutes from './routes/nft.routes';
import socialRoutes from './routes/social.routes';
import adminRoutes from './routes/admin.routes';

// WebSocket Handlers
import { setupMiningHandlers } from './websocket/handlers/mining.handler';
import { setupXPHandlers } from './websocket/handlers/xp.handler';
import { setupSocialHandlers } from './websocket/handlers/social.handler';
import { setupNotificationHandlers } from './websocket/handlers/notification.handler';
import { authWSMiddleware } from './websocket/middleware/auth.ws';
import { rateLimitWSMiddleware } from './websocket/middleware/rate-limit.ws';

// Services
import { MiningService } from './services/mining.service';
import { XPService } from './services/xp.service';
import { ReferralService } from './services/referral.service';
import { AntiBotService } from './services/anti-bot.service';
import { AIQualityService } from './services/ai-quality.service';
import { NotificationService } from './services/notification.service';
import { AnalyticsService } from './services/analytics.service';
import { BlockchainService } from './services/blockchain.service';

class FinovaApp {
  public app: express.Application;
  public server: any;
  public io: Server;
  
  // Core Services
  private miningService: MiningService;
  private xpService: XPService;
  private referralService: ReferralService;
  private antiBotService: AntiBotService;
  private aiQualityService: AIQualityService;
  private notificationService: NotificationService;
  private analyticsService: AnalyticsService;
  private blockchainService: BlockchainService;

  constructor() {
    this.app = express();
    this.server = createServer(this.app);
    this.io = new Server(this.server, {
      cors: {
        origin: config.CORS_ORIGINS,
        credentials: true,
      },
      transports: ['websocket', 'polling'],
      pingTimeout: 60000,
      pingInterval: 25000,
    });
    
    this.initializeServices();
    this.setupMiddleware();
    this.setupRoutes();
    this.setupWebSocket();
    this.setupErrorHandling();
  }

  private initializeServices(): void {
    try {
      this.blockchainService = new BlockchainService();
      this.miningService = new MiningService(this.blockchainService);
      this.xpService = new XPService();
      this.referralService = new ReferralService();
      this.antiBotService = new AntiBotService();
      this.aiQualityService = new AIQualityService();
      this.notificationService = new NotificationService();
      this.analyticsService = new AnalyticsService();
      
      logger.info('🚀 All services initialized successfully');
    } catch (error) {
      logger.error('❌ Service initialization failed:', error);
      process.exit(1);
    }
  }

  private setupMiddleware(): void {
    // Security & Performance
    this.app.use(helmet({
      crossOriginEmbedderPolicy: false,
      contentSecurityPolicy: {
        directives: {
          defaultSrc: ["'self'"],
          styleSrc: ["'self'", "'unsafe-inline'"],
          scriptSrc: ["'self'"],
          imgSrc: ["'self'", "data:", "https:"],
        },
      },
    }));
    
    this.app.use(compression());
    this.app.use(morgan('combined', { 
      stream: { write: (message) => logger.info(message.trim()) }
    }));

    // CORS Configuration
    this.app.use(corsMiddleware);

    // Rate Limiting
    this.app.use('/api/v1/auth', rateLimit({
      windowMs: 15 * 60 * 1000, // 15 minutes
      max: 10, // limit each IP to 10 requests per windowMs
      message: 'Too many authentication attempts, please try again later.',
      standardHeaders: true,
      legacyHeaders: false,
    }));

    this.app.use('/api/v1/mining', rateLimit({
      windowMs: 60 * 1000, // 1 minute
      max: 60, // limit each IP to 60 requests per minute
      message: 'Mining rate limit exceeded, please slow down.',
      standardHeaders: true,
      legacyHeaders: false,
    }));

    this.app.use('/api/v1', rateLimitMiddleware);

    // Body Parsing
    this.app.use(express.json({ 
      limit: '10mb',
      verify: (req: any, res, buf) => {
        req.rawBody = buf.toString('utf8');
      }
    }));
    this.app.use(express.urlencoded({ extended: true, limit: '10mb' }));

    // Request ID & Logging
    this.app.use((req: any, res, next) => {
      req.requestId = require('crypto').randomBytes(16).toString('hex');
      res.setHeader('X-Request-ID', req.requestId);
      logger.info(`📡 ${req.method} ${req.originalUrl} - ${req.ip} - ${req.requestId}`);
      next();
    });

    // Health Check
    this.app.get('/health', (req, res) => {
      res.status(200).json({
        status: 'healthy',
        timestamp: new Date().toISOString(),
        uptime: process.uptime(),
        environment: config.NODE_ENV,
        version: require('../package.json').version,
        services: {
          database: 'connected',
          redis: 'connected',
          blockchain: 'connected',
        }
      });
    });

    // API Documentation
    if (config.NODE_ENV !== 'production') {
      const swaggerDocument = require('../swagger.json');
      this.app.use('/api-docs', swaggerUi.serve, swaggerUi.setup(swaggerDocument));
    }
  }

  private setupRoutes(): void {
    const apiRouter = express.Router();

    // Public routes
    apiRouter.use('/auth', authRoutes);
    
    // Protected routes (require authentication)
    apiRouter.use('/users', authMiddleware, userRoutes);
    apiRouter.use('/mining', authMiddleware, miningRoutes);
    apiRouter.use('/xp', authMiddleware, xpRoutes);
    apiRouter.use('/referrals', authMiddleware, referralRoutes);
    apiRouter.use('/nft', authMiddleware, nftRoutes);
    apiRouter.use('/social', authMiddleware, socialRoutes);
    
    // Admin routes (require admin authentication)
    apiRouter.use('/admin', authMiddleware, kycMiddleware('admin'), adminRoutes);

    this.app.use('/api/v1', apiRouter);

    // Catch-all route
    this.app.use('*', (req, res) => {
      res.status(404).json({
        success: false,
        message: 'Endpoint not found',
        path: req.originalUrl,
        timestamp: new Date().toISOString(),
      });
    });
  }

  private setupWebSocket(): void {
    // WebSocket Authentication
    this.io.use(authWSMiddleware);
    this.io.use(rateLimitWSMiddleware);

    this.io.on('connection', (socket) => {
      const user = (socket as any).user;
      logger.info(`👤 User connected: ${user.id} - ${socket.id}`);

      // Join user to personal room
      socket.join(`user:${user.id}`);
      
      // Join user to referral network room
      if (user.referralCode) {
        socket.join(`referral:${user.referralCode}`);
      }

      // Setup event handlers
      setupMiningHandlers(socket, this.miningService, this.antiBotService);
      setupXPHandlers(socket, this.xpService, this.aiQualityService);
      setupSocialHandlers(socket, this.aiQualityService, this.analyticsService);
      setupNotificationHandlers(socket, this.notificationService);

      // Handle disconnection
      socket.on('disconnect', (reason) => {
        logger.info(`👤 User disconnected: ${user.id} - ${reason}`);
        
        // Update user status
        this.analyticsService.trackUserDisconnection(user.id, reason);
        
        // Leave all rooms
        socket.leave(`user:${user.id}`);
        if (user.referralCode) {
          socket.leave(`referral:${user.referralCode}`);
        }
      });

      // Send welcome message with current status
      socket.emit('connection:established', {
        userId: user.id,
        serverTime: new Date().toISOString(),
        miningRate: this.miningService.getCurrentMiningRate(user),
        xpLevel: user.xpLevel,
        rpTier: user.rpTier,
        notifications: this.notificationService.getUnreadCount(user.id),
      });
    });

    // Real-time mining updates
    setInterval(async () => {
      try {
        const activeMiners = await this.miningService.getActiveMiners();
        
        for (const miner of activeMiners) {
          const currentRate = this.miningService.getCurrentMiningRate(miner);
          const totalEarned = await this.miningService.getTotalEarned(miner.id);
          
          this.io.to(`user:${miner.id}`).emit('mining:update', {
            currentRate,
            totalEarned,
            lastUpdate: new Date().toISOString(),
          });
        }
      } catch (error) {
        logger.error('Mining update broadcast error:', error);
      }
    }, 30000); // Update every 30 seconds

    logger.info('🔗 WebSocket server initialized');
  }

  private setupErrorHandling(): void {
    // Global error handler
    this.app.use(errorHandler);

    // Unhandled promise rejections
    process.on('unhandledRejection', (reason, promise) => {
      logger.error('🚨 Unhandled Rejection at:', promise, 'reason:', reason);
      // Don't exit process in production, just log the error
      if (config.NODE_ENV !== 'production') {
        process.exit(1);
      }
    });

    // Uncaught exceptions
    process.on('uncaughtException', (error) => {
      logger.error('🚨 Uncaught Exception thrown:', error);
      process.exit(1);
    });

    // Graceful shutdown
    process.on('SIGTERM', this.gracefulShutdown.bind(this));
    process.on('SIGINT', this.gracefulShutdown.bind(this));
  }

  private async gracefulShutdown(signal: string): Promise<void> {
    logger.info(`🛑 ${signal} received, starting graceful shutdown...`);
    
    try {
      // Stop accepting new connections
      this.server.close((err: any) => {
        if (err) {
          logger.error('Error during server shutdown:', err);
          return process.exit(1);
        }
        
        logger.info('✅ HTTP server closed');
      });

      // Close WebSocket connections
      this.io.close((err: any) => {
        if (err) {
          logger.error('Error during WebSocket shutdown:', err);
        } else {
          logger.info('✅ WebSocket server closed');
        }
      });

      // Close database connections
      // await disconnectDatabase();
      // await disconnectRedis();
      
      logger.info('✅ Graceful shutdown completed');
      process.exit(0);
    } catch (error) {
      logger.error('❌ Error during graceful shutdown:', error);
      process.exit(1);
    }
  }

  public async start(): Promise<void> {
    try {
      // Initialize external connections
      await connectDatabase();
      await connectRedis();
      await initializeBlockchain();

      // Start background services
      await this.startBackgroundServices();

      // Start server
      const port = config.PORT || 3000;
      this.server.listen(port, () => {
        logger.info(`🚀 Finova Network API server started on port ${port}`);
        logger.info(`📚 API Documentation available at: http://localhost:${port}/api-docs`);
        logger.info(`🔗 Environment: ${config.NODE_ENV}`);
        logger.info(`⚡ WebSocket ready for connections`);
      });

    } catch (error) {
      logger.error('❌ Failed to start server:', error);
      process.exit(1);
    }
  }

  private async startBackgroundServices(): Promise<void> {
    // Start mining calculation service
    this.miningService.startMiningCalculations();
    
    // Start XP decay calculations
    this.xpService.startXPDecayCalculations();
    
    // Start referral network updates
    this.referralService.startNetworkUpdates();
    
    // Start anti-bot monitoring
    this.antiBotService.startMonitoring();
    
    // Start analytics processing
    this.analyticsService.startProcessing();

    logger.info('🔄 Background services started successfully');
  }
}

// Export app instance
const finovaApp = new FinovaApp();
export default finovaApp;

// Start server if this file is run directly
if (require.main === module) {
  finovaApp.start().catch(error => {
    logger.error('Failed to start Finova Network:', error);
    process.exit(1);
  });
}
