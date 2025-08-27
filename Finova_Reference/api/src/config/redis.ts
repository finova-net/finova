import Redis from 'ioredis';
import { createHash } from 'crypto';
import { logger } from '../utils/logger';

/**
 * Redis Configuration for Finova Network
 * Handles caching, session management, real-time data, and mining calculations
 */

// Redis connection pools for different purposes
export interface RedisConfig {
  host: string;
  port: number;
  password?: string;
  username?: string;
  db: number;
  keyPrefix?: string;
  maxRetriesPerRequest: number;
  retryDelayOnFailover: number;
  connectTimeout: number;
  commandTimeout: number;
  lazyConnect: boolean;
  enableReadyCheck: boolean;
  maxLoadBalanceLatency?: number;
}

// Environment-specific configurations
const getRedisConfig = (purpose: 'cache' | 'session' | 'queue' | 'realtime'): RedisConfig => {
  const baseConfig = {
    host: process.env.REDIS_HOST || 'localhost',
    port: parseInt(process.env.REDIS_PORT || '6379'),
    password: process.env.REDIS_PASSWORD,
    username: process.env.REDIS_USERNAME,
    maxRetriesPerRequest: 3,
    retryDelayOnFailover: 100,
    connectTimeout: 10000,
    commandTimeout: 5000,
    lazyConnect: true,
    enableReadyCheck: true,
  };

  switch (purpose) {
    case 'cache':
      return {
        ...baseConfig,
        db: parseInt(process.env.REDIS_CACHE_DB || '0'),
        keyPrefix: 'finova:cache:',
      };
    case 'session':
      return {
        ...baseConfig,
        db: parseInt(process.env.REDIS_SESSION_DB || '1'),
        keyPrefix: 'finova:session:',
      };
    case 'queue':
      return {
        ...baseConfig,
        db: parseInt(process.env.REDIS_QUEUE_DB || '2'),
        keyPrefix: 'finova:queue:',
      };
    case 'realtime':
      return {
        ...baseConfig,
        db: parseInt(process.env.REDIS_REALTIME_DB || '3'),
        keyPrefix: 'finova:rt:',
      };
    default:
      return baseConfig;
  }
};

// Redis clients for different purposes
export class RedisManager {
  private static instance: RedisManager;
  
  public cache: Redis;
  public session: Redis;
  public queue: Redis;
  public realtime: Redis;
  public pubsub: Redis;

  private constructor() {
    // Initialize Redis connections with proper error handling
    this.cache = this.createConnection('cache');
    this.session = this.createConnection('session');
    this.queue = this.createConnection('queue');
    this.realtime = this.createConnection('realtime');
    
    // Separate pub/sub connection
    this.pubsub = new Redis(getRedisConfig('realtime'));
    
    this.setupEventHandlers();
    this.initializeKeyspaces();
  }

  public static getInstance(): RedisManager {
    if (!RedisManager.instance) {
      RedisManager.instance = new RedisManager();
    }
    return RedisManager.instance;
  }

  private createConnection(purpose: 'cache' | 'session' | 'queue' | 'realtime'): Redis {
    const config = getRedisConfig(purpose);
    const redis = new Redis(config);

    // Connection event handlers
    redis.on('connect', () => {
      logger.info(`Redis ${purpose} connected successfully`);
    });

    redis.on('error', (error) => {
      logger.error(`Redis ${purpose} connection error:`, error);
    });

    redis.on('close', () => {
      logger.warn(`Redis ${purpose} connection closed`);
    });

    return redis;
  }

  private setupEventHandlers(): void {
    // Mining calculation events
    this.pubsub.on('message', (channel: string, message: string) => {
      try {
        const data = JSON.parse(message);
        this.handleRealtimeEvent(channel, data);
      } catch (error) {
        logger.error('Failed to parse Redis pub/sub message:', error);
      }
    });

    // Subscribe to critical channels
    this.pubsub.subscribe([
      'mining:rates:update',
      'xp:calculation:update',
      'rp:network:update',
      'user:activity:stream',
      'nft:marketplace:events',
      'guild:activities',
      'staking:rewards:distribution'
    ]);
  }

  private async initializeKeyspaces(): Promise<void> {
    try {
      // Initialize mining calculation cache
      await this.cache.hset('mining:config', {
        base_rate: '0.05',
        phase: '2',
        total_users: '50000',
        last_update: Date.now().toString()
      });

      // Initialize XP calculation constants
      await this.cache.hset('xp:constants', {
        level_progression_factor: '0.01',
        quality_multiplier_range: '0.5:2.0',
        streak_max_bonus: '3.0'
      });

      // Initialize RP network settings
      await this.cache.hset('rp:config', {
        network_quality_weight: '0.85',
        regression_factor: '0.0001',
        tier_multipliers: JSON.stringify({
          explorer: 1.0,
          connector: 1.2,
          influencer: 1.5,
          leader: 2.0,
          ambassador: 3.0
        })
      });

      logger.info('Redis keyspaces initialized successfully');
    } catch (error) {
      logger.error('Failed to initialize Redis keyspaces:', error);
    }
  }

  private handleRealtimeEvent(channel: string, data: any): void {
    // Handle different types of real-time events
    switch (channel) {
      case 'mining:rates:update':
        this.updateMiningRates(data);
        break;
      case 'xp:calculation:update':
        this.updateXPCalculations(data);
        break;
      case 'rp:network:update':
        this.updateRPNetwork(data);
        break;
      default:
        logger.debug(`Unhandled channel: ${channel}`, data);
    }
  }

  // Mining-specific caching methods
  public async getMiningRate(userId: string): Promise<number> {
    const key = `user:${userId}:mining_rate`;
    const cached = await this.cache.get(key);
    
    if (cached) {
      return parseFloat(cached);
    }

    // Calculate and cache for 5 minutes
    const rate = await this.calculateMiningRate(userId);
    await this.cache.setex(key, 300, rate.toString());
    return rate;
  }

  private async calculateMiningRate(userId: string): Promise<number> {
    // Get user data from cache or database
    const userData = await this.getUserMiningData(userId);
    
    // Apply Finova's exponential regression formula
    const baseRate = parseFloat(await this.cache.hget('mining:config', 'base_rate') || '0.05');
    const totalUsers = parseInt(await this.cache.hget('mining:config', 'total_users') || '50000');
    
    const pioneerBonus = Math.max(1.0, 2.0 - (totalUsers / 1000000));
    const referralBonus = 1 + (userData.activeReferrals * 0.1);
    const securityBonus = userData.kycVerified ? 1.2 : 0.8;
    const regressionFactor = Math.exp(-0.001 * userData.totalHoldings);
    
    return baseRate * pioneerBonus * referralBonus * securityBonus * regressionFactor;
  }

  // XP calculation with integrated bonuses
  public async calculateXPReward(userId: string, activity: any): Promise<number> {
    const cacheKey = `xp:calc:${userId}:${createHash('md5').update(JSON.stringify(activity)).digest('hex')}`;
    const cached = await this.cache.get(cacheKey);
    
    if (cached) {
      return parseFloat(cached);
    }

    const userData = await this.getUserXPData(userId);
    const constants = await this.cache.hgetall('xp:constants');
    
    const baseXP = this.getBaseXP(activity.type);
    const platformMultiplier = this.getPlatformMultiplier(activity.platform);
    const qualityScore = activity.qualityScore || 1.0;
    const streakBonus = Math.min(1.0 + (userData.streakDays * 0.1), 3.0);
    const levelProgression = Math.exp(-parseFloat(constants.level_progression_factor) * userData.level);
    
    const xpReward = baseXP * platformMultiplier * qualityScore * streakBonus * levelProgression;
    
    // Cache for 1 hour
    await this.cache.setex(cacheKey, 3600, xpReward.toString());
    
    return xpReward;
  }

  // RP network value calculation
  public async calculateRPValue(userId: string): Promise<number> {
    const key = `rp:value:${userId}`;
    const cached = await this.cache.get(key);
    
    if (cached) {
      return parseFloat(cached);
    }

    const networkData = await this.getUserNetworkData(userId);
    const config = await this.cache.hgetall('rp:config');
    
    const directRP = networkData.directReferrals.reduce((sum: number, ref: any) => 
      sum + (ref.activity * ref.level * ref.timeDecay), 0);
    
    const indirectRP = (networkData.l2Activity * 0.3) + (networkData.l3Activity * 0.1);
    
    const networkQuality = networkData.activeUsers / Math.max(networkData.totalReferrals, 1);
    const qualityBonus = networkQuality * networkData.averageLevel * networkData.retentionRate;
    
    const regressionFactor = Math.exp(-parseFloat(config.network_quality_weight) * 
      networkData.totalNetworkSize * networkData.qualityScore);
    
    const rpValue = (directRP + indirectRP) * qualityBonus * regressionFactor;
    
    // Cache for 10 minutes
    await this.cache.setex(key, 600, rpValue.toString());
    
    return rpValue;
  }

  // Session management for user authentication
  public async setUserSession(userId: string, sessionData: any, ttl: number = 86400): Promise<void> {
    const key = `user:${userId}:session`;
    await this.session.setex(key, ttl, JSON.stringify(sessionData));
  }

  public async getUserSession(userId: string): Promise<any | null> {
    const key = `user:${userId}:session`;
    const session = await this.session.get(key);
    return session ? JSON.parse(session) : null;
  }

  public async deleteUserSession(userId: string): Promise<void> {
    const key = `user:${userId}:session`;
    await this.session.del(key);
  }

  // Rate limiting for anti-bot protection
  public async checkRateLimit(identifier: string, limit: number, window: number): Promise<{ allowed: boolean; remaining: number }> {
    const key = `rate_limit:${identifier}`;
    const current = await this.cache.incr(key);
    
    if (current === 1) {
      await this.cache.expire(key, window);
    }

    return {
      allowed: current <= limit,
      remaining: Math.max(0, limit - current)
    };
  }

  // Real-time leaderboard updates
  public async updateLeaderboard(category: string, userId: string, score: number): Promise<void> {
    const key = `leaderboard:${category}`;
    await this.realtime.zadd(key, score, userId);
    
    // Keep only top 1000 entries
    await this.realtime.zremrangebyrank(key, 0, -1001);
    
    // Publish update
    await this.pubsub.publish(`leaderboard:${category}:update`, JSON.stringify({
      userId,
      score,
      timestamp: Date.now()
    }));
  }

  public async getLeaderboard(category: string, start: number = 0, end: number = 99): Promise<Array<{ userId: string; score: number; rank: number }>> {
    const key = `leaderboard:${category}`;
    const results = await this.realtime.zrevrange(key, start, end, 'WITHSCORES');
    
    const leaderboard = [];
    for (let i = 0; i < results.length; i += 2) {
      leaderboard.push({
        userId: results[i],
        score: parseFloat(results[i + 1]),
        rank: start + (i / 2) + 1
      });
    }
    
    return leaderboard;
  }

  // NFT and marketplace caching
  public async cacheNFTMetadata(tokenId: string, metadata: any, ttl: number = 3600): Promise<void> {
    const key = `nft:metadata:${tokenId}`;
    await this.cache.setex(key, ttl, JSON.stringify(metadata));
  }

  public async getNFTMetadata(tokenId: string): Promise<any | null> {
    const key = `nft:metadata:${tokenId}`;
    const metadata = await this.cache.get(key);
    return metadata ? JSON.parse(metadata) : null;
  }

  // Guild activity tracking
  public async trackGuildActivity(guildId: string, userId: string, activity: any): Promise<void> {
    const key = `guild:${guildId}:activity`;
    const activityData = {
      userId,
      activity,
      timestamp: Date.now()
    };
    
    await this.realtime.lpush(key, JSON.stringify(activityData));
    await this.realtime.ltrim(key, 0, 999); // Keep last 1000 activities
    
    // Publish to guild members
    await this.pubsub.publish(`guild:${guildId}:activity`, JSON.stringify(activityData));
  }

  // Utility methods
  private async getUserMiningData(userId: string): Promise<any> {
    const key = `user:${userId}:mining_data`;
    const cached = await this.cache.get(key);
    
    if (cached) {
      return JSON.parse(cached);
    }

    // Default values for new users
    return {
      totalHoldings: 0,
      activeReferrals: 0,
      kycVerified: false,
      miningStartTime: Date.now()
    };
  }

  private async getUserXPData(userId: string): Promise<any> {
    const key = `user:${userId}:xp_data`;
    const cached = await this.cache.get(key);
    
    if (cached) {
      return JSON.parse(cached);
    }

    return {
      level: 1,
      totalXP: 0,
      streakDays: 0,
      lastActivity: Date.now()
    };
  }

  private async getUserNetworkData(userId: string): Promise<any> {
    const key = `user:${userId}:network_data`;
    const cached = await this.cache.get(key);
    
    if (cached) {
      return JSON.parse(cached);
    }

    return {
      directReferrals: [],
      l2Activity: 0,
      l3Activity: 0,
      totalNetworkSize: 0,
      qualityScore: 1.0,
      activeUsers: 0,
      totalReferrals: 0,
      averageLevel: 1,
      retentionRate: 1.0
    };
  }

  private getBaseXP(activityType: string): number {
    const xpMap: Record<string, number> = {
      'original_post': 50,
      'photo_post': 75,
      'video_post': 150,
      'story_post': 25,
      'comment': 25,
      'like': 5,
      'share': 15,
      'follow': 20,
      'daily_login': 10,
      'daily_quest': 100,
      'milestone': 500,
      'viral_content': 1000
    };
    
    return xpMap[activityType] || 0;
  }

  private getPlatformMultiplier(platform: string): number {
    const multipliers: Record<string, number> = {
      'tiktok': 1.3,
      'instagram': 1.2,
      'youtube': 1.4,
      'x': 1.2,
      'facebook': 1.1,
      'default': 1.0
    };
    
    return multipliers[platform] || multipliers.default;
  }

  private async updateMiningRates(data: any): Promise<void> {
    await this.cache.hset('mining:config', data);
    logger.info('Mining rates updated via pub/sub', data);
  }

  private async updateXPCalculations(data: any): Promise<void> {
    await this.cache.hset('xp:constants', data);
    logger.info('XP calculations updated via pub/sub', data);
  }

  private async updateRPNetwork(data: any): Promise<void> {
    await this.cache.hset('rp:config', data);
    logger.info('RP network config updated via pub/sub', data);
  }

  // Health check and monitoring
  public async healthCheck(): Promise<{ status: string; connections: Record<string, string> }> {
    const connections: Record<string, string> = {};
    
    try {
      await this.cache.ping();
      connections.cache = 'healthy';
    } catch {
      connections.cache = 'unhealthy';
    }

    try {
      await this.session.ping();
      connections.session = 'healthy';
    } catch {
      connections.session = 'unhealthy';
    }

    try {
      await this.queue.ping();
      connections.queue = 'healthy';
    } catch {
      connections.queue = 'unhealthy';
    }

    try {
      await this.realtime.ping();
      connections.realtime = 'healthy';
    } catch {
      connections.realtime = 'unhealthy';
    }

    const allHealthy = Object.values(connections).every(status => status === 'healthy');
    
    return {
      status: allHealthy ? 'healthy' : 'degraded',
      connections
    };
  }

  // Graceful shutdown
  public async disconnect(): Promise<void> {
    await Promise.all([
      this.cache.quit(),
      this.session.quit(),
      this.queue.quit(),
      this.realtime.quit(),
      this.pubsub.quit()
    ]);
    
    logger.info('All Redis connections closed gracefully');
  }
}

// Export singleton instance
export const redisManager = RedisManager.getInstance();

// Export individual clients for convenience
export const {
  cache: cacheRedis,
  session: sessionRedis,
  queue: queueRedis,
  realtime: realtimeRedis,
  pubsub: pubsubRedis
} = redisManager;
