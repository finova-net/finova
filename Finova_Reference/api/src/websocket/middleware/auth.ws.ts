import { WebSocket } from 'ws';
import jwt from 'jsonwebtoken';
import crypto from 'crypto';
import { Redis } from 'ioredis';
import { logger } from '../../utils/logger';
import { 
  AuthenticatedWebSocket, 
  WebSocketAuthPayload, 
  ConnectionSecurityConfig,
  UserSessionData,
  SecurityMetrics 
} from '../../types/websocket.types';

/**
 * Enhanced WebSocket Authentication Middleware for Finova Network
 * Implements multi-layer security for real-time XP, RP, and mining updates
 */
export class WebSocketAuthMiddleware {
  private redis: Redis;
  private securityConfig: ConnectionSecurityConfig;
  private connectionMetrics: Map<string, SecurityMetrics>;
  private rateLimitMap: Map<string, number[]>;

  constructor(redis: Redis, config?: Partial<ConnectionSecurityConfig>) {
    this.redis = redis;
    this.connectionMetrics = new Map();
    this.rateLimitMap = new Map();
    
    this.securityConfig = {
      maxConnectionsPerUser: 5,
      connectionTimeoutMs: 30000,
      rateLimitWindow: 60000,
      maxMessagesPerWindow: 100,
      requireKYC: true,
      enableFingerprinting: true,
      sessionTimeoutHours: 24,
      ...config
    };
  }

  /**
   * Primary authentication middleware for WebSocket connections
   */
  public authenticate = async (
    ws: WebSocket, 
    request: any, 
    next: Function
  ): Promise<void> => {
    const startTime = Date.now();
    let authPayload: WebSocketAuthPayload | null = null;

    try {
      // Extract authentication data
      authPayload = await this.extractAuthPayload(request);
      
      // Validate JWT token
      const tokenPayload = await this.validateJWT(authPayload.token);
      
      // Security checks
      await this.performSecurityChecks(authPayload, request);
      
      // Rate limiting
      await this.checkRateLimit(tokenPayload.userId, request.socket.remoteAddress);
      
      // User session validation
      const sessionData = await this.validateUserSession(tokenPayload.userId);
      
      // Connection limits
      await this.enforceConnectionLimits(tokenPayload.userId);
      
      // Enhanced WebSocket with user context
      const authenticatedWs = this.enhanceWebSocket(ws, tokenPayload, sessionData);
      
      // Store connection metrics
      await this.recordConnectionMetrics(tokenPayload.userId, request, startTime);
      
      // Setup connection lifecycle handlers
      this.setupConnectionHandlers(authenticatedWs, tokenPayload.userId);
      
      logger.info('WebSocket authenticated successfully', {
        userId: tokenPayload.userId,
        connectionTime: Date.now() - startTime,
        userLevel: sessionData.xpLevel,
        rpTier: sessionData.rpTier
      });

      next();
      
    } catch (error) {
      await this.handleAuthenticationError(ws, error, authPayload);
    }
  };

  /**
   * Extract authentication payload from request
   */
  private async extractAuthPayload(request: any): Promise<WebSocketAuthPayload> {
    const url = new URL(request.url, 'ws://localhost');
    const token = url.searchParams.get('token') || 
                  request.headers.authorization?.replace('Bearer ', '');
    
    if (!token) {
      throw new Error('MISSING_TOKEN');
    }

    const fingerprint = this.generateFingerprint(request);
    const clientInfo = this.extractClientInfo(request);

    return {
      token,
      fingerprint,
      clientInfo,
      timestamp: Date.now()
    };
  }

  /**
   * Validate JWT token and extract payload
   */
  private async validateJWT(token: string): Promise<any> {
    try {
      const decoded = jwt.verify(token, process.env.JWT_SECRET!) as any;
      
      // Check token expiry with buffer
      const now = Math.floor(Date.now() / 1000);
      if (decoded.exp && decoded.exp < now) {
        throw new Error('TOKEN_EXPIRED');
      }

      // Validate required claims
      if (!decoded.userId || !decoded.sessionId) {
        throw new Error('INVALID_TOKEN_CLAIMS');
      }

      return decoded;
      
    } catch (error) {
      if (error instanceof jwt.JsonWebTokenError) {
        throw new Error('INVALID_TOKEN');
      }
      throw error;
    }
  }

  /**
   * Comprehensive security checks
   */
  private async performSecurityChecks(
    authPayload: WebSocketAuthPayload, 
    request: any
  ): Promise<void> {
    // Fingerprint validation
    if (this.securityConfig.enableFingerprinting) {
      await this.validateFingerprint(authPayload);
    }

    // IP reputation check
    await this.checkIPReputation(request.socket.remoteAddress);
    
    // Device consistency validation
    await this.validateDeviceConsistency(authPayload);
    
    // Anti-bot detection
    await this.performAntiBotChecks(authPayload);
  }

  /**
   * Validate user session and retrieve context
   */
  private async validateUserSession(userId: string): Promise<UserSessionData> {
    const sessionKey = `finova:session:${userId}`;
    const sessionData = await this.redis.get(sessionKey);
    
    if (!sessionData) {
      throw new Error('SESSION_NOT_FOUND');
    }

    const parsed = JSON.parse(sessionData);
    
    // Check session expiry
    if (parsed.expiresAt < Date.now()) {
      await this.redis.del(sessionKey);
      throw new Error('SESSION_EXPIRED');
    }

    // KYC requirement check
    if (this.securityConfig.requireKYC && !parsed.isKYCVerified) {
      throw new Error('KYC_REQUIRED');
    }

    return parsed;
  }

  /**
   * Rate limiting implementation
   */
  private async checkRateLimit(userId: string, ip: string): Promise<void> {
    const now = Date.now();
    const windowStart = now - this.securityConfig.rateLimitWindow;
    
    // User-based rate limiting
    const userKey = `rate_limit:user:${userId}`;
    const userRequests = this.rateLimitMap.get(userKey) || [];
    const recentUserRequests = userRequests.filter(time => time > windowStart);
    
    if (recentUserRequests.length >= this.securityConfig.maxMessagesPerWindow) {
      throw new Error('USER_RATE_LIMIT_EXCEEDED');
    }

    // IP-based rate limiting
    const ipKey = `rate_limit:ip:${ip}`;
    const ipRequests = this.rateLimitMap.get(ipKey) || [];
    const recentIpRequests = ipRequests.filter(time => time > windowStart);
    
    if (recentIpRequests.length >= this.securityConfig.maxMessagesPerWindow * 2) {
      throw new Error('IP_RATE_LIMIT_EXCEEDED');
    }

    // Update rate limit maps
    recentUserRequests.push(now);
    recentIpRequests.push(now);
    this.rateLimitMap.set(userKey, recentUserRequests);
    this.rateLimitMap.set(ipKey, recentIpRequests);
  }

  /**
   * Enforce connection limits per user
   */
  private async enforceConnectionLimits(userId: string): Promise<void> {
    const connectionKey = `connections:${userId}`;
    const activeConnections = await this.redis.scard(connectionKey);
    
    if (activeConnections >= this.securityConfig.maxConnectionsPerUser) {
      // Remove oldest connection
      const oldestConnection = await this.redis.spop(connectionKey);
      if (oldestConnection) {
        logger.warn('Removed oldest connection due to limit', { 
          userId, 
          removedConnection: oldestConnection 
        });
      }
    }
  }

  /**
   * Enhance WebSocket with Finova-specific context
   */
  private enhanceWebSocket(
    ws: WebSocket, 
    tokenPayload: any, 
    sessionData: UserSessionData
  ): AuthenticatedWebSocket {
    const authenticatedWs = ws as AuthenticatedWebSocket;
    
    // Add user context
    authenticatedWs.userId = tokenPayload.userId;
    authenticatedWs.sessionId = tokenPayload.sessionId;
    authenticatedWs.userLevel = sessionData.xpLevel;
    authenticatedWs.rpTier = sessionData.rpTier;
    authenticatedWs.stakingTier = sessionData.stakingTier;
    authenticatedWs.isKYCVerified = sessionData.isKYCVerified;
    authenticatedWs.subscriptions = new Set();
    authenticatedWs.lastActivity = Date.now();
    
    // Add Finova-specific methods
    authenticatedWs.subscribe = (channel: string) => {
      authenticatedWs.subscriptions.add(channel);
      logger.debug('User subscribed to channel', { 
        userId: authenticatedWs.userId, 
        channel 
      });
    };
    
    authenticatedWs.unsubscribe = (channel: string) => {
      authenticatedWs.subscriptions.delete(channel);
    };
    
    authenticatedWs.sendFinovaMessage = (type: string, data: any) => {
      const message = {
        type,
        data,
        timestamp: Date.now(),
        userId: authenticatedWs.userId
      };
      authenticatedWs.send(JSON.stringify(message));
    };

    return authenticatedWs;
  }

  /**
   * Setup connection lifecycle handlers
   */
  private setupConnectionHandlers(ws: AuthenticatedWebSocket, userId: string): void {
    const connectionId = crypto.randomUUID();
    const connectionKey = `connections:${userId}`;
    
    // Register connection
    this.redis.sadd(connectionKey, connectionId);
    this.redis.expire(connectionKey, this.securityConfig.sessionTimeoutHours * 3600);
    
    // Heartbeat mechanism
    const heartbeatInterval = setInterval(() => {
      if (ws.readyState === WebSocket.OPEN) {
        ws.ping();
        ws.lastActivity = Date.now();
      } else {
        clearInterval(heartbeatInterval);
      }
    }, 30000);

    // Connection cleanup
    ws.on('close', async () => {
      clearInterval(heartbeatInterval);
      await this.redis.srem(connectionKey, connectionId);
      this.connectionMetrics.delete(userId);
      
      logger.info('WebSocket connection closed', { 
        userId, 
        connectionId,
        duration: Date.now() - ws.lastActivity 
      });
    });

    // Error handling
    ws.on('error', (error) => {
      logger.error('WebSocket error', { userId, error: error.message });
      clearInterval(heartbeatInterval);
    });

    // Pong response
    ws.on('pong', () => {
      ws.lastActivity = Date.now();
    });
  }

  /**
   * Generate device fingerprint for security
   */
  private generateFingerprint(request: any): string {
    const components = [
      request.headers['user-agent'] || '',
      request.headers['accept-language'] || '',
      request.headers['accept-encoding'] || '',
      request.socket.remoteAddress || '',
      request.headers['x-forwarded-for'] || ''
    ];
    
    return crypto
      .createHash('sha256')
      .update(components.join('|'))
      .digest('hex')
      .substring(0, 16);
  }

  /**
   * Extract client information
   */
  private extractClientInfo(request: any): any {
    return {
      userAgent: request.headers['user-agent'],
      ip: request.socket.remoteAddress,
      forwardedFor: request.headers['x-forwarded-for'],
      acceptLanguage: request.headers['accept-language'],
      origin: request.headers.origin
    };
  }

  /**
   * Validate device fingerprint consistency
   */
  private async validateFingerprint(authPayload: WebSocketAuthPayload): Promise<void> {
    const fpKey = `fingerprint:${authPayload.fingerprint}`;
    const storedData = await this.redis.get(fpKey);
    
    if (storedData) {
      const parsed = JSON.parse(storedData);
      
      // Check for suspicious changes
      if (parsed.userAgent !== authPayload.clientInfo.userAgent) {
        logger.warn('Fingerprint mismatch detected', {
          fingerprint: authPayload.fingerprint,
          stored: parsed.userAgent,
          current: authPayload.clientInfo.userAgent
        });
      }
    } else {
      // Store new fingerprint
      await this.redis.setex(fpKey, 86400 * 30, JSON.stringify(authPayload.clientInfo));
    }
  }

  /**
   * Check IP reputation and geolocation
   */
  private async checkIPReputation(ip: string): Promise<void> {
    const reputationKey = `ip_reputation:${ip}`;
    const reputation = await this.redis.get(reputationKey);
    
    if (reputation && parseInt(reputation) < 0) {
      throw new Error('IP_BLOCKED');
    }
  }

  /**
   * Validate device consistency
   */
  private async validateDeviceConsistency(authPayload: WebSocketAuthPayload): Promise<void> {
    // Implementation for device consistency checks
    // This would involve checking device characteristics against stored patterns
  }

  /**
   * Anti-bot detection checks
   */
  private async performAntiBotChecks(authPayload: WebSocketAuthPayload): Promise<void> {
    // Implementation for anti-bot detection
    // This would involve behavioral pattern analysis
  }

  /**
   * Record connection metrics for monitoring
   */
  private async recordConnectionMetrics(
    userId: string, 
    request: any, 
    startTime: number
  ): Promise<void> {
    const metrics: SecurityMetrics = {
      userId,
      connectionTime: Date.now() - startTime,
      ip: request.socket.remoteAddress,
      userAgent: request.headers['user-agent'],
      timestamp: Date.now()
    };
    
    this.connectionMetrics.set(userId, metrics);
    
    // Store in Redis for analytics
    const metricsKey = `ws_metrics:${userId}:${Date.now()}`;
    await this.redis.setex(metricsKey, 3600, JSON.stringify(metrics));
  }

  /**
   * Handle authentication errors
   */
  private async handleAuthenticationError(
    ws: WebSocket, 
    error: any, 
    authPayload: WebSocketAuthPayload | null
  ): Promise<void> {
    const errorResponse = {
      type: 'auth_error',
      error: error.message,
      timestamp: Date.now()
    };

    logger.error('WebSocket authentication failed', {
      error: error.message,
      fingerprint: authPayload?.fingerprint,
      clientInfo: authPayload?.clientInfo
    });

    // Send error response before closing
    if (ws.readyState === WebSocket.OPEN) {
      ws.send(JSON.stringify(errorResponse));
    }
    
    // Close connection with appropriate code
    const closeCode = this.getCloseCodeForError(error.message);
    ws.close(closeCode, error.message);
  }

  /**
   * Get appropriate close code for error type
   */
  private getCloseCodeForError(errorMessage: string): number {
    const errorCodes: { [key: string]: number } = {
      'MISSING_TOKEN': 4001,
      'INVALID_TOKEN': 4002,
      'TOKEN_EXPIRED': 4003,
      'SESSION_EXPIRED': 4004,
      'KYC_REQUIRED': 4005,
      'RATE_LIMIT_EXCEEDED': 4006,
      'IP_BLOCKED': 4007
    };
    
    return errorCodes[errorMessage] || 4000;
  }

  /**
   * Cleanup method for graceful shutdown
   */
  public async cleanup(): Promise<void> {
    this.connectionMetrics.clear();
    this.rateLimitMap.clear();
    logger.info('WebSocket auth middleware cleaned up');
  }
}
