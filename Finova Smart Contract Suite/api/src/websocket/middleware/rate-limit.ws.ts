import { WebSocket } from 'ws';
import Redis from 'ioredis';
import { logger } from '../../utils/logger';
import { config } from '../../config';

interface RateLimitConfig {
  windowMs: number;
  maxRequests: number;
  keyGenerator: (ws: WebSocket, message: any) => string;
  skipSuccessfulRequests?: boolean;
  skipFailedRequests?: boolean;
  onLimitReached?: (ws: WebSocket, key: string) => void;
}

interface UserTier {
  tier: string;
  multiplier: number;
  maxConnections: number;
}

interface RateLimitEntry {
  count: number;
  resetTime: number;
  blocked: boolean;
  warnings: number;
}

export class WebSocketRateLimiter {
  private redis: Redis;
  private configs: Map<string, RateLimitConfig>;
  private userTiers: Map<string, UserTier>;
  private suspiciousIPs: Set<string>;
  private globalMetrics: Map<string, number>;

  constructor() {
    this.redis = new Redis({
      host: config.redis.host,
      port: config.redis.port,
      password: config.redis.password,
      db: config.redis.rateLimitDb || 2,
      retryDelayOnFailover: 100,
      maxRetriesPerRequest: 3
    });

    this.configs = new Map();
    this.userTiers = new Map();
    this.suspiciousIPs = new Set();
    this.globalMetrics = new Map();
    
    this.initializeConfigurations();
    this.startCleanupInterval();
  }

  private initializeConfigurations(): void {
    // Mining activity rate limits
    this.configs.set('mining:claim', {
      windowMs: 60 * 1000, // 1 minute
      maxRequests: 1,
      keyGenerator: (ws, msg) => `mining:${this.getUserId(ws)}`,
      onLimitReached: this.handleMiningAbuse.bind(this)
    });

    // XP activity rate limits
    this.configs.set('xp:activity', {
      windowMs: 10 * 1000, // 10 seconds
      maxRequests: 5,
      keyGenerator: (ws, msg) => `xp:${this.getUserId(ws)}`,
      onLimitReached: this.handleXPSpam.bind(this)
    });

    // Social integration rate limits
    this.configs.set('social:post', {
      windowMs: 30 * 1000, // 30 seconds
      maxRequests: 3,
      keyGenerator: (ws, msg) => `social:${this.getUserId(ws)}`,
      onLimitReached: this.handleSocialSpam.bind(this)
    });

    // Referral system rate limits
    this.configs.set('referral:action', {
      windowMs: 5 * 60 * 1000, // 5 minutes
      maxRequests: 10,
      keyGenerator: (ws, msg) => `referral:${this.getUserId(ws)}`,
      onLimitReached: this.handleReferralAbuse.bind(this)
    });

    // NFT marketplace rate limits
    this.configs.set('nft:trade', {
      windowMs: 15 * 1000, // 15 seconds
      maxRequests: 2,
      keyGenerator: (ws, msg) => `nft:${this.getUserId(ws)}`,
      onLimitReached: this.handleNFTSpam.bind(this)
    });

    // Guild activities
    this.configs.set('guild:message', {
      windowMs: 2 * 1000, // 2 seconds
      maxRequests: 1,
      keyGenerator: (ws, msg) => `guild:${this.getUserId(ws)}`,
      onLimitReached: this.handleGuildSpam.bind(this)
    });

    // General message rate limit
    this.configs.set('general:message', {
      windowMs: 1000, // 1 second
      maxRequests: 10,
      keyGenerator: (ws, msg) => `general:${this.getClientIP(ws)}`,
      onLimitReached: this.handleGeneralSpam.bind(this)
    });

    // Connection rate limits per IP
    this.configs.set('connection:ip', {
      windowMs: 60 * 1000, // 1 minute
      maxRequests: 10,
      keyGenerator: (ws, msg) => `conn:${this.getClientIP(ws)}`,
      onLimitReached: this.handleConnectionSpam.bind(this)
    });
  }

  private initializeUserTiers(): void {
    this.userTiers.set('bronze', {
      tier: 'bronze',
      multiplier: 1.0,
      maxConnections: 2
    });

    this.userTiers.set('silver', {
      tier: 'silver',
      multiplier: 1.2,
      maxConnections: 3
    });

    this.userTiers.set('gold', {
      tier: 'gold',
      multiplier: 1.5,
      maxConnections: 5
    });

    this.userTiers.set('platinum', {
      tier: 'platinum',
      multiplier: 2.0,
      maxConnections: 10
    });

    this.userTiers.set('diamond', {
      tier: 'diamond',
      multiplier: 3.0,
      maxConnections: 15
    });

    this.userTiers.set('mythic', {
      tier: 'mythic',
      multiplier: 5.0,
      maxConnections: 25
    });
  }

  public async checkRateLimit(
    ws: WebSocket,
    messageType: string,
    message: any
  ): Promise<{ allowed: boolean; remainingRequests?: number; resetTime?: number; reason?: string }> {
    try {
      const config = this.configs.get(messageType);
      if (!config) {
        return { allowed: true };
      }

      const key = config.keyGenerator(ws, message);
      const userTier = await this.getUserTier(ws);
      const adjustedLimit = Math.floor(config.maxRequests * userTier.multiplier);

      // Check if IP is suspicious
      const clientIP = this.getClientIP(ws);
      if (this.suspiciousIPs.has(clientIP)) {
        return {
          allowed: false,
          reason: 'Suspicious activity detected'
        };
      }

      // Get current rate limit data
      const entry = await this.getRateLimitEntry(key, config.windowMs);
      
      // Check if user is currently blocked
      if (entry.blocked && Date.now() < entry.resetTime) {
        return {
          allowed: false,
          resetTime: entry.resetTime,
          reason: 'Rate limit exceeded - temporarily blocked'
        };
      }

      // Check rate limit
      if (entry.count >= adjustedLimit) {
        await this.handleRateLimitExceeded(ws, key, config, entry);
        return {
          allowed: false,
          remainingRequests: 0,
          resetTime: entry.resetTime,
          reason: 'Rate limit exceeded'
        };
      }

      // Increment counter
      await this.incrementCounter(key, config.windowMs);
      
      // Update global metrics
      this.updateGlobalMetrics(messageType);

      // Check for suspicious patterns
      await this.detectSuspiciousActivity(ws, messageType, entry);

      return {
        allowed: true,
        remainingRequests: adjustedLimit - (entry.count + 1),
        resetTime: entry.resetTime
      };

    } catch (error) {
      logger.error('Rate limit check failed:', error);
      // Fail open for availability, but log the error
      return { allowed: true };
    }
  }

  private async getRateLimitEntry(key: string, windowMs: number): Promise<RateLimitEntry> {
    const multi = this.redis.multi();
    multi.hgetall(`rl:${key}`);
    multi.ttl(`rl:${key}`);
    
    const results = await multi.exec();
    if (!results || results.length !== 2) {
      throw new Error('Failed to get rate limit entry');
    }

    const [entryResult, ttlResult] = results;
    if (entryResult[0] || ttlResult[0]) {
      throw new Error('Redis operation failed');
    }

    const entryData = entryResult[1] as Record<string, string>;
    const ttl = ttlResult[1] as number;

    const now = Date.now();
    const resetTime = ttl > 0 ? now + (ttl * 1000) : now + windowMs;

    return {
      count: parseInt(entryData.count || '0'),
      resetTime,
      blocked: entryData.blocked === 'true',
      warnings: parseInt(entryData.warnings || '0')
    };
  }

  private async incrementCounter(key: string, windowMs: number): Promise<void> {
    const pipeline = this.redis.pipeline();
    pipeline.hincrby(`rl:${key}`, 'count', 1);
    pipeline.expire(`rl:${key}`, Math.ceil(windowMs / 1000));
    await pipeline.exec();
  }

  private async handleRateLimitExceeded(
    ws: WebSocket,
    key: string,
    config: RateLimitConfig,
    entry: RateLimitEntry
  ): Promise<void> {
    const warnings = entry.warnings + 1;
    const blockDuration = this.calculateBlockDuration(warnings);
    
    // Update entry with warning and potential block
    const pipeline = this.redis.pipeline();
    pipeline.hset(`rl:${key}`, 'warnings', warnings);
    
    if (warnings >= 3) {
      pipeline.hset(`rl:${key}`, 'blocked', 'true');
      pipeline.expire(`rl:${key}`, blockDuration);
      
      // Add IP to suspicious list if multiple violations
      const clientIP = this.getClientIP(ws);
      if (warnings >= 5) {
        this.suspiciousIPs.add(clientIP);
        await this.redis.setex(`suspicious:${clientIP}`, 3600, 'true'); // 1 hour
      }
    }
    
    await pipeline.exec();

    // Call configured handler
    if (config.onLimitReached) {
      config.onLimitReached(ws, key);
    }

    // Log rate limit violation
    logger.warn('Rate limit exceeded', {
      key,
      warnings,
      blocked: warnings >= 3,
      userAgent: (ws as any).upgradeReq?.headers['user-agent'],
      ip: this.getClientIP(ws)
    });
  }

  private calculateBlockDuration(warnings: number): number {
    // Exponential backoff: 1min, 5min, 15min, 1hour, 6hours
    const durations = [60, 300, 900, 3600, 21600];
    const index = Math.min(warnings - 3, durations.length - 1);
    return durations[index];
  }

  private async detectSuspiciousActivity(ws: WebSocket, messageType: string, entry: RateLimitEntry): Promise<void> {
    const userId = this.getUserId(ws);
    const clientIP = this.getClientIP(ws);
    
    // Check for rapid successive requests
    if (entry.count > 5) {
      const suspicious = await this.redis.incr(`suspicious:rapid:${userId}`);
      await this.redis.expire(`suspicious:rapid:${userId}`, 300); // 5 minutes
      
      if (suspicious > 10) {
        logger.warn('Suspicious rapid activity detected', { userId, clientIP, messageType });
      }
    }

    // Check for distributed attacks from same user
    const connectionCount = await this.redis.scard(`connections:${userId}`);
    const userTier = await this.getUserTier(ws);
    
    if (connectionCount > userTier.maxConnections) {
      logger.warn('Suspicious connection count', { userId, connectionCount, maxAllowed: userTier.maxConnections });
      this.suspiciousIPs.add(clientIP);
    }
  }

  private async getUserTier(ws: WebSocket): Promise<UserTier> {
    const userId = this.getUserId(ws);
    if (!userId) {
      return this.userTiers.get('bronze')!;
    }

    try {
      const tierData = await this.redis.hget(`user:${userId}`, 'tier');
      return this.userTiers.get(tierData || 'bronze') || this.userTiers.get('bronze')!;
    } catch (error) {
      logger.error('Failed to get user tier:', error);
      return this.userTiers.get('bronze')!;
    }
  }

  private getUserId(ws: WebSocket): string | null {
    return (ws as any).userId || null;
  }

  private getClientIP(ws: WebSocket): string {
    const req = (ws as any).upgradeReq;
    return req?.headers['x-forwarded-for']?.split(',')[0] || 
           req?.headers['x-real-ip'] || 
           req?.connection?.remoteAddress || 
           'unknown';
  }

  private updateGlobalMetrics(messageType: string): void {
    const current = this.globalMetrics.get(messageType) || 0;
    this.globalMetrics.set(messageType, current + 1);
  }

  // Abuse handlers for different message types
  private handleMiningAbuse(ws: WebSocket, key: string): void {
    this.sendRateLimitMessage(ws, 'MINING_RATE_LIMIT', {
      message: 'Mining claims are limited to once per minute',
      nextAllowed: Date.now() + 60000
    });
  }

  private handleXPSpam(ws: WebSocket, key: string): void {
    this.sendRateLimitMessage(ws, 'XP_RATE_LIMIT', {
      message: 'XP activities are limited to 5 per 10 seconds',
      suggestion: 'Focus on quality over quantity for better rewards'
    });
  }

  private handleSocialSpam(ws: WebSocket, key: string): void {
    this.sendRateLimitMessage(ws, 'SOCIAL_RATE_LIMIT', {
      message: 'Social posts are limited to 3 per 30 seconds',
      suggestion: 'Create meaningful content for higher XP rewards'
    });
  }

  private handleReferralAbuse(ws: WebSocket, key: string): void {
    this.sendRateLimitMessage(ws, 'REFERRAL_RATE_LIMIT', {
      message: 'Referral actions are limited to 10 per 5 minutes',
      suggestion: 'Focus on quality referrals for better RP rewards'
    });
  }

  private handleNFTSpam(ws: WebSocket, key: string): void {
    this.sendRateLimitMessage(ws, 'NFT_RATE_LIMIT', {
      message: 'NFT trades are limited to 2 per 15 seconds',
      suggestion: 'Consider your trades carefully for optimal returns'
    });
  }

  private handleGuildSpam(ws: WebSocket, key: string): void {
    this.sendRateLimitMessage(ws, 'GUILD_RATE_LIMIT', {
      message: 'Guild messages are limited to 1 per 2 seconds',
      suggestion: 'Quality communication builds stronger guilds'
    });
  }

  private handleGeneralSpam(ws: WebSocket, key: string): void {
    this.sendRateLimitMessage(ws, 'GENERAL_RATE_LIMIT', {
      message: 'Too many messages sent too quickly',
      suggestion: 'Please slow down your interactions'
    });
  }

  private handleConnectionSpam(ws: WebSocket, key: string): void {
    logger.warn('Connection spam detected', { ip: this.getClientIP(ws) });
    ws.close(1008, 'Too many connections from this IP');
  }

  private sendRateLimitMessage(ws: WebSocket, type: string, data: any): void {
    if (ws.readyState === WebSocket.OPEN) {
      ws.send(JSON.stringify({
        type,
        data,
        timestamp: Date.now()
      }));
    }
  }

  public async trackConnection(ws: WebSocket): Promise<void> {
    const userId = this.getUserId(ws);
    const clientIP = this.getClientIP(ws);
    const connectionId = `${Date.now()}-${Math.random()}`;

    if (userId) {
      await this.redis.sadd(`connections:${userId}`, connectionId);
      await this.redis.expire(`connections:${userId}`, 3600); // 1 hour
    }

    await this.redis.sadd(`connections:ip:${clientIP}`, connectionId);
    await this.redis.expire(`connections:ip:${clientIP}`, 3600);

    // Store connection reference for cleanup
    (ws as any).connectionId = connectionId;
  }

  public async untrackConnection(ws: WebSocket): Promise<void> {
    const userId = this.getUserId(ws);
    const clientIP = this.getClientIP(ws);
    const connectionId = (ws as any).connectionId;

    if (connectionId) {
      if (userId) {
        await this.redis.srem(`connections:${userId}`, connectionId);
      }
      await this.redis.srem(`connections:ip:${clientIP}`, connectionId);
    }
  }

  public async getMetrics(): Promise<any> {
    const metrics = {
      globalCounts: Object.fromEntries(this.globalMetrics),
      suspiciousIPs: this.suspiciousIPs.size,
      activeConnections: await this.getActiveConnectionCount(),
      rateLimitViolations: await this.getRateLimitViolationCount()
    };

    return metrics;
  }

  private async getActiveConnectionCount(): Promise<number> {
    const keys = await this.redis.keys('connections:*');
    let total = 0;
    
    for (const key of keys) {
      const count = await this.redis.scard(key);
      total += count;
    }
    
    return total;
  }

  private async getRateLimitViolationCount(): Promise<number> {
    const keys = await this.redis.keys('rl:*');
    let violations = 0;
    
    for (const key of keys) {
      const warnings = await this.redis.hget(key, 'warnings');
      if (warnings && parseInt(warnings) > 0) {
        violations++;
      }
    }
    
    return violations;
  }

  private startCleanupInterval(): void {
    // Clean up suspicious IPs every hour
    setInterval(async () => {
      try {
        const keys = await this.redis.keys('suspicious:*');
        for (const key of keys) {
          const exists = await this.redis.exists(key);
          if (!exists) {
            const ip = key.replace('suspicious:', '');
            this.suspiciousIPs.delete(ip);
          }
        }
        
        // Reset global metrics every hour
        this.globalMetrics.clear();
        
        logger.info('Rate limiter cleanup completed');
      } catch (error) {
        logger.error('Rate limiter cleanup failed:', error);
      }
    }, 3600000); // 1 hour
  }

  public async cleanup(): Promise<void> {
    await this.redis.quit();
  }
}

// Export singleton instance
export const rateLimiter = new WebSocketRateLimiter();

// Middleware function for easy integration
export const rateLimitMiddleware = async (
  ws: WebSocket,
  messageType: string,
  message: any,
  next: () => void
): Promise<void> => {
  try {
    const result = await rateLimiter.checkRateLimit(ws, messageType, message);
    
    if (!result.allowed) {
      logger.warn('WebSocket message blocked by rate limiter', {
        messageType,
        reason: result.reason,
        userId: (ws as any).userId,
        ip: (ws as any).upgradeReq?.connection?.remoteAddress
      });
      return; // Don't call next() - block the message
    }
    
    // Rate limit check passed, continue processing
    next();
  } catch (error) {
    logger.error('Rate limit middleware error:', error);
    // On error, allow the message through but log it
    next();
  }
};
