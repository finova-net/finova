import { Server } from 'socket.io';
import { createServer } from 'http';
import jwt from 'jsonwebtoken';
import Redis from 'ioredis';
import { RateLimiterRedis } from 'rate-limiter-flexible';
import { EventEmitter } from 'events';
import { z } from 'zod';

// Import handlers
import { MiningHandler } from './handlers/mining.handler';
import { XPHandler } from './handlers/xp.handler';
import { SocialHandler } from './handlers/social.handler';
import { NotificationHandler } from './handlers/notification.handler';

// Import middleware
import { authMiddleware } from './middleware/auth.ws';
import { rateLimitMiddleware } from './middleware/rate-limit.ws';

// Types and interfaces
interface AuthenticatedSocket extends Socket {
  userId: string;
  userLevel: number;
  rpTier: string;
  stakingTier: string;
  isKYCVerified: boolean;
  guildId?: string;
  sessionData: {
    loginTime: Date;
    lastActivity: Date;
    deviceInfo: any;
    location?: string;
  };
}

interface WebSocketConfig {
  port: number;
  redisUrl: string;
  jwtSecret: string;
  rateLimits: {
    points: number;
    duration: number;
    blockDuration: number;
  };
  cors: {
    origin: string[];
    credentials: boolean;
  };
}

// Event schemas for validation
const miningEventSchema = z.object({
  action: z.enum(['start', 'stop', 'claim', 'boost']),
  data: z.object({
    boostCardId: z.string().optional(),
    platform: z.string().optional(),
  }).optional(),
});

const socialEventSchema = z.object({
  action: z.enum(['post_created', 'engagement', 'viral_milestone', 'quality_score']),
  data: z.object({
    platform: z.string(),
    contentId: z.string(),
    contentType: z.enum(['text', 'image', 'video', 'story']),
    engagementType: z.enum(['like', 'comment', 'share', 'follow']).optional(),
    qualityScore: z.number().min(0.5).max(2.0).optional(),
    viewCount: z.number().optional(),
  }),
});

const xpEventSchema = z.object({
  action: z.enum(['gain', 'level_up', 'milestone', 'streak_bonus']),
  data: z.object({
    amount: z.number().positive(),
    source: z.string(),
    multiplier: z.number().optional(),
    newLevel: z.number().optional(),
  }),
});

const guildEventSchema = z.object({
  action: z.enum(['join', 'leave', 'challenge', 'tournament', 'chat']),
  data: z.object({
    guildId: z.string(),
    challengeId: z.string().optional(),
    message: z.string().optional(),
    tournamentId: z.string().optional(),
  }),
});

class FinovaWebSocketServer extends EventEmitter {
  private io: Server;
  private redis: Redis;
  private rateLimiter: RateLimiterRedis;
  private config: WebSocketConfig;
  
  // Handler instances
  private miningHandler: MiningHandler;
  private xpHandler: XPHandler;
  private socialHandler: SocialHandler;
  private notificationHandler: NotificationHandler;
  
  // Active connections tracking
  private activeConnections = new Map<string, AuthenticatedSocket>();
  private userRooms = new Map<string, Set<string>>(); // userId -> Set of room names
  
  constructor(config: WebSocketConfig) {
    super();
    this.config = config;
    this.setupRedis();
    this.setupRateLimiter();
    this.setupServer();
    this.initializeHandlers();
    this.setupEventListeners();
  }

  private setupRedis(): void {
    this.redis = new Redis(this.config.redisUrl, {
      retryDelayOnFailover: 100,
      enableReadyCheck: false,
      maxRetriesPerRequest: null,
    });

    this.redis.on('error', (error) => {
      console.error('Redis WebSocket error:', error);
      this.emit('error', error);
    });
  }

  private setupRateLimiter(): void {
    this.rateLimiter = new RateLimiterRedis({
      storeClient: this.redis,
      keyPrefix: 'ws_rate_limit',
      points: this.config.rateLimits.points,
      duration: this.config.rateLimits.duration,
      blockDuration: this.config.rateLimits.blockDuration,
    });
  }

  private setupServer(): void {
    const httpServer = createServer();
    
    this.io = new Server(httpServer, {
      cors: this.config.cors,
      transports: ['websocket', 'polling'],
      pingTimeout: 60000,
      pingInterval: 25000,
      allowEIO3: true,
    });

    // Apply middleware
    this.io.use(authMiddleware(this.config.jwtSecret));
    this.io.use(rateLimitMiddleware(this.rateLimiter));

    // Connection handling
    this.io.on('connection', this.handleConnection.bind(this));
    
    httpServer.listen(this.config.port, () => {
      console.log(`🚀 Finova WebSocket server running on port ${this.config.port}`);
    });
  }

  private initializeHandlers(): void {
    this.miningHandler = new MiningHandler(this.redis, this.io);
    this.xpHandler = new XPHandler(this.redis, this.io);
    this.socialHandler = new SocialHandler(this.redis, this.io);
    this.notificationHandler = new NotificationHandler(this.redis, this.io);
  }

  private setupEventListeners(): void {
    // Listen to external events from API
    this.redis.subscribe('finova:mining:update');
    this.redis.subscribe('finova:xp:gain');
    this.redis.subscribe('finova:referral:bonus');
    this.redis.subscribe('finova:social:viral');
    this.redis.subscribe('finova:guild:event');
    this.redis.subscribe('finova:notification:send');

    this.redis.on('message', this.handleRedisMessage.bind(this));
  }

  private async handleConnection(socket: AuthenticatedSocket): Promise<void> {
    try {
      console.log(`User ${socket.userId} connected to WebSocket`);
      
      // Track connection
      this.activeConnections.set(socket.userId, socket);
      
      // Join user-specific room
      await socket.join(`user:${socket.userId}`);
      
      // Join tier-based rooms
      await socket.join(`level:${socket.userLevel}`);
      await socket.join(`rp:${socket.rpTier}`);
      
      if (socket.stakingTier) {
        await socket.join(`staking:${socket.stakingTier}`);
      }
      
      if (socket.guildId) {
        await socket.join(`guild:${socket.guildId}`);
      }

      // Update user rooms tracking
      this.updateUserRooms(socket.userId, [
        `user:${socket.userId}`,
        `level:${socket.userLevel}`,
        `rp:${socket.rpTier}`,
        socket.stakingTier ? `staking:${socket.stakingTier}` : null,
        socket.guildId ? `guild:${socket.guildId}` : null,
      ].filter(Boolean) as string[]);

      // Send initial data
      await this.sendInitialData(socket);
      
      // Register event handlers
      this.registerSocketEvents(socket);
      
      // Update user status
      await this.updateUserStatus(socket.userId, 'online');
      
      // Handle disconnection
      socket.on('disconnect', () => this.handleDisconnection(socket));
      
    } catch (error) {
      console.error('Connection error:', error);
      socket.emit('error', { message: 'Connection failed' });
      socket.disconnect();
    }
  }

  private async sendInitialData(socket: AuthenticatedSocket): Promise<void> {
    try {
      const [miningData, xpData, notifications] = await Promise.all([
        this.miningHandler.getUserMiningData(socket.userId),
        this.xpHandler.getUserXPData(socket.userId),
        this.notificationHandler.getPendingNotifications(socket.userId),
      ]);

      socket.emit('initial:data', {
        mining: miningData,
        xp: xpData,
        notifications,
        serverTime: new Date().toISOString(),
      });
    } catch (error) {
      console.error('Error sending initial data:', error);
    }
  }

  private registerSocketEvents(socket: AuthenticatedSocket): void {
    // Mining events
    socket.on('mining:action', async (data) => {
      try {
        const validData = miningEventSchema.parse(data);
        await this.miningHandler.handleMiningAction(socket, validData);
      } catch (error) {
        socket.emit('error', { type: 'validation', message: error.message });
      }
    });

    // Social events
    socket.on('social:event', async (data) => {
      try {
        const validData = socialEventSchema.parse(data);
        await this.socialHandler.handleSocialEvent(socket, validData);
      } catch (error) {
        socket.emit('error', { type: 'validation', message: error.message });
      }
    });

    // XP events
    socket.on('xp:event', async (data) => {
      try {
        const validData = xpEventSchema.parse(data);
        await this.xpHandler.handleXPEvent(socket, validData);
      } catch (error) {
        socket.emit('error', { type: 'validation', message: error.message });
      }
    });

    // Guild events
    socket.on('guild:event', async (data) => {
      try {
        const validData = guildEventSchema.parse(data);
        await this.handleGuildEvent(socket, validData);
      } catch (error) {
        socket.emit('error', { type: 'validation', message: error.message });
      }
    });

    // Heartbeat for mining
    socket.on('heartbeat', async () => {
      await this.miningHandler.updateHeartbeat(socket.userId);
      socket.emit('heartbeat:ack', { timestamp: Date.now() });
    });

    // Request data refresh
    socket.on('refresh:request', async (type: string) => {
      await this.handleRefreshRequest(socket, type);
    });
  }

  private async handleGuildEvent(socket: AuthenticatedSocket, data: any): Promise<void> {
    const { action, data: eventData } = data;
    
    switch (action) {
      case 'join':
        await socket.join(`guild:${eventData.guildId}`);
        socket.guildId = eventData.guildId;
        this.io.to(`guild:${eventData.guildId}`).emit('guild:member_joined', {
          userId: socket.userId,
          timestamp: new Date().toISOString(),
        });
        break;
        
      case 'leave':
        await socket.leave(`guild:${eventData.guildId}`);
        socket.guildId = undefined;
        this.io.to(`guild:${eventData.guildId}`).emit('guild:member_left', {
          userId: socket.userId,
          timestamp: new Date().toISOString(),
        });
        break;
        
      case 'chat':
        this.io.to(`guild:${eventData.guildId}`).emit('guild:message', {
          userId: socket.userId,
          message: eventData.message,
          timestamp: new Date().toISOString(),
        });
        break;
    }
  }

  private async handleRefreshRequest(socket: AuthenticatedSocket, type: string): Promise<void> {
    switch (type) {
      case 'mining':
        const miningData = await this.miningHandler.getUserMiningData(socket.userId);
        socket.emit('mining:data', miningData);
        break;
        
      case 'xp':
        const xpData = await this.xpHandler.getUserXPData(socket.userId);
        socket.emit('xp:data', xpData);
        break;
        
      case 'notifications':
        const notifications = await this.notificationHandler.getPendingNotifications(socket.userId);
        socket.emit('notifications:data', notifications);
        break;
        
      case 'all':
        await this.sendInitialData(socket);
        break;
    }
  }

  private async handleRedisMessage(channel: string, message: string): Promise<void> {
    try {
      const data = JSON.parse(message);
      
      switch (channel) {
        case 'finova:mining:update':
          await this.broadcastMiningUpdate(data);
          break;
          
        case 'finova:xp:gain':
          await this.broadcastXPGain(data);
          break;
          
        case 'finova:referral:bonus':
          await this.broadcastReferralBonus(data);
          break;
          
        case 'finova:social:viral':
          await this.broadcastViralContent(data);
          break;
          
        case 'finova:guild:event':
          await this.broadcastGuildEvent(data);
          break;
          
        case 'finova:notification:send':
          await this.sendNotification(data);
          break;
      }
    } catch (error) {
      console.error('Error handling Redis message:', error);
    }
  }

  private async broadcastMiningUpdate(data: any): Promise<void> {
    const { userId, miningRate, totalEarned, phase } = data;
    
    // Send to specific user
    this.io.to(`user:${userId}`).emit('mining:update', {
      miningRate,
      totalEarned,
      phase,
      timestamp: new Date().toISOString(),
    });
    
    // Send aggregated data to all users
    this.io.emit('mining:network_stats', {
      totalMiners: await this.redis.scard('active_miners'),
      totalMined: await this.redis.get('total_mined'),
      currentPhase: phase,
    });
  }

  private async broadcastXPGain(data: any): Promise<void> {
    const { userId, xpGained, newLevel, source } = data;
    
    this.io.to(`user:${userId}`).emit('xp:gained', {
      amount: xpGained,
      newLevel,
      source,
      timestamp: new Date().toISOString(),
    });
    
    // If level up, broadcast to relevant rooms
    if (newLevel && data.oldLevel !== newLevel) {
      this.io.to(`level:${newLevel}`).emit('level:new_member', {
        userId,
        level: newLevel,
      });
    }
  }

  private async broadcastReferralBonus(data: any): Promise<void> {
    const { referrerId, refereeId, bonusAmount, rpGained } = data;
    
    // Send to referrer
    this.io.to(`user:${referrerId}`).emit('referral:bonus', {
      refereeId,
      bonusAmount,
      rpGained,
      timestamp: new Date().toISOString(),
    });
    
    // Send to referee
    this.io.to(`user:${refereeId}`).emit('referral:milestone', {
      referrerId,
      milestone: 'first_reward',
      timestamp: new Date().toISOString(),
    });
  }

  private async broadcastViralContent(data: any): Promise<void> {
    const { userId, contentId, platform, viewCount } = data;
    
    // Celebrate viral content
    this.io.emit('social:viral_celebration', {
      userId,
      contentId,
      platform,
      viewCount,
      timestamp: new Date().toISOString(),
    });
    
    // Send bonus to creator
    this.io.to(`user:${userId}`).emit('achievement:unlocked', {
      type: 'viral_content',
      platform,
      viewCount,
      bonus: data.bonus,
    });
  }

  private async broadcastGuildEvent(data: any): Promise<void> {
    const { guildId, eventType, eventData } = data;
    
    this.io.to(`guild:${guildId}`).emit('guild:event', {
      type: eventType,
      data: eventData,
      timestamp: new Date().toISOString(),
    });
  }

  private async sendNotification(data: any): Promise<void> {
    const { userId, notification } = data;
    
    this.io.to(`user:${userId}`).emit('notification:new', {
      ...notification,
      timestamp: new Date().toISOString(),
    });
  }

  private updateUserRooms(userId: string, rooms: string[]): void {
    this.userRooms.set(userId, new Set(rooms));
  }

  private async updateUserStatus(userId: string, status: 'online' | 'offline'): Promise<void> {
    await this.redis.setex(`user_status:${userId}`, 300, status); // 5 minute expiry
    
    // Notify friends/guild members
    const userRooms = this.userRooms.get(userId);
    if (userRooms) {
      userRooms.forEach(room => {
        if (room.startsWith('guild:')) {
          this.io.to(room).emit('user:status_change', { userId, status });
        }
      });
    }
  }

  private async handleDisconnection(socket: AuthenticatedSocket): Promise<void> {
    console.log(`User ${socket.userId} disconnected from WebSocket`);
    
    // Remove from active connections
    this.activeConnections.delete(socket.userId);
    this.userRooms.delete(socket.userId);
    
    // Update status
    await this.updateUserStatus(socket.userId, 'offline');
    
    // Stop mining if active
    await this.miningHandler.handleDisconnection(socket.userId);
    
    // Clean up any pending operations
    await this.redis.del(`ws_session:${socket.userId}`);
  }

  // Public methods for external access
  public async broadcastToUser(userId: string, event: string, data: any): Promise<void> {
    this.io.to(`user:${userId}`).emit(event, data);
  }

  public async broadcastToGuild(guildId: string, event: string, data: any): Promise<void> {
    this.io.to(`guild:${guildId}`).emit(event, data);
  }

  public async broadcastToLevel(level: number, event: string, data: any): Promise<void> {
    this.io.to(`level:${level}`).emit(event, data);
  }

  public async broadcastGlobal(event: string, data: any): Promise<void> {
    this.io.emit(event, data);
  }

  public getActiveConnections(): number {
    return this.activeConnections.size;
  }

  public async getOnlineUsers(): Promise<string[]> {
    return Array.from(this.activeConnections.keys());
  }

  public async gracefulShutdown(): Promise<void> {
    console.log('Shutting down WebSocket server...');
    
    // Notify all clients
    this.io.emit('server:shutdown', {
      message: 'Server is shutting down for maintenance',
      timestamp: new Date().toISOString(),
    });
    
    // Give clients time to save state
    await new Promise(resolve => setTimeout(resolve, 5000));
    
    // Close all connections
    this.io.close();
    await this.redis.quit();
    
    console.log('WebSocket server shutdown complete');
  }
}

// Export function to create and start server
export function createWebSocketServer(config: WebSocketConfig): FinovaWebSocketServer {
  return new FinovaWebSocketServer(config);
}

// Export types
export { FinovaWebSocketServer, AuthenticatedSocket, WebSocketConfig };

// Default export
export default FinovaWebSocketServer;
