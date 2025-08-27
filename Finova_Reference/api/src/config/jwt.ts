import jwt from 'jsonwebtoken';
import { createHash, randomBytes } from 'crypto';
import { Redis } from 'ioredis';
import { logger } from '../utils/logger';

// JWT Configuration Interface
export interface JWTConfig {
  accessTokenSecret: string;
  refreshTokenSecret: string;
  accessTokenExpiry: string;
  refreshTokenExpiry: string;
  issuer: string;
  audience: string;
  algorithm: jwt.Algorithm;
  biometricSecret: string;
  kycSecret: string;
}

// Token Payload Interface
export interface TokenPayload {
  userId: string;
  walletAddress?: string;
  email?: string;
  role: 'user' | 'admin' | 'moderator' | 'ambassador';
  kycVerified: boolean;
  biometricVerified: boolean;
  xpLevel: number;
  rpTier: 'explorer' | 'connector' | 'influencer' | 'leader' | 'ambassador';
  stakingTier?: number;
  permissions: string[];
  sessionId: string;
  deviceFingerprint?: string;
  lastActivity: number;
  antiBot: {
    humanScore: number;
    riskLevel: 'low' | 'medium' | 'high';
    lastVerification: number;
  };
}

// Biometric Verification Payload
export interface BiometricPayload {
  userId: string;
  biometricHash: string;
  deviceId: string;
  timestamp: number;
  verificationLevel: 'face' | 'fingerprint' | 'voice';
}

// KYC Token Payload
export interface KYCPayload {
  userId: string;
  kycLevel: 'basic' | 'advanced' | 'premium';
  documentHash: string;
  verificationDate: number;
  providerRef: string;
}

class JWTManager {
  private config: JWTConfig;
  private redis: Redis;
  private blacklistedTokens: Set<string> = new Set();
  private activeSessions: Map<string, TokenPayload> = new Map();

  constructor() {
    this.config = {
      accessTokenSecret: process.env.JWT_ACCESS_SECRET || this.generateSecret(),
      refreshTokenSecret: process.env.JWT_REFRESH_SECRET || this.generateSecret(),
      accessTokenExpiry: process.env.JWT_ACCESS_EXPIRY || '15m',
      refreshTokenExpiry: process.env.JWT_REFRESH_EXPIRY || '7d',
      issuer: process.env.JWT_ISSUER || 'finova-network',
      audience: process.env.JWT_AUDIENCE || 'finova-api',
      algorithm: (process.env.JWT_ALGORITHM as jwt.Algorithm) || 'HS256',
      biometricSecret: process.env.JWT_BIOMETRIC_SECRET || this.generateSecret(),
      kycSecret: process.env.JWT_KYC_SECRET || this.generateSecret(),
    };

    this.redis = new Redis({
      host: process.env.REDIS_HOST || 'localhost',
      port: parseInt(process.env.REDIS_PORT || '6379'),
      password: process.env.REDIS_PASSWORD,
      db: parseInt(process.env.REDIS_JWT_DB || '2'),
    });

    this.setupCleanupScheduler();
  }

  /**
   * Generate secure random secret
   */
  private generateSecret(): string {
    return randomBytes(64).toString('hex');
  }

  /**
   * Create device fingerprint hash
   */
  private createDeviceFingerprint(userAgent: string, ip: string, additionalData?: any): string {
    const data = JSON.stringify({ userAgent, ip, ...additionalData });
    return createHash('sha256').update(data).digest('hex');
  }

  /**
   * Generate access token with comprehensive payload
   */
  public generateAccessToken(payload: Omit<TokenPayload, 'sessionId' | 'lastActivity'>): string {
    const sessionId = randomBytes(16).toString('hex');
    const fullPayload: TokenPayload = {
      ...payload,
      sessionId,
      lastActivity: Date.now(),
    };

    // Store session in memory and Redis
    this.activeSessions.set(sessionId, fullPayload);
    this.redis.setex(`session:${sessionId}`, 900, JSON.stringify(fullPayload)); // 15 min

    const token = jwt.sign(fullPayload, this.config.accessTokenSecret, {
      expiresIn: this.config.accessTokenExpiry,
      issuer: this.config.issuer,
      audience: this.config.audience,
      algorithm: this.config.algorithm,
      jwtid: sessionId,
    });

    logger.info('Access token generated', { 
      userId: payload.userId, 
      sessionId, 
      xpLevel: payload.xpLevel,
      rpTier: payload.rpTier 
    });

    return token;
  }

  /**
   * Generate refresh token
   */
  public generateRefreshToken(userId: string, sessionId: string): string {
    const payload = {
      userId,
      sessionId,
      type: 'refresh',
      timestamp: Date.now(),
    };

    const token = jwt.sign(payload, this.config.refreshTokenSecret, {
      expiresIn: this.config.refreshTokenExpiry,
      issuer: this.config.issuer,
      audience: this.config.audience,
      algorithm: this.config.algorithm,
    });

    // Store refresh token in Redis with longer TTL
    this.redis.setex(`refresh:${sessionId}`, 604800, token); // 7 days

    return token;
  }

  /**
   * Generate biometric verification token
   */
  public generateBiometricToken(payload: BiometricPayload): string {
    const token = jwt.sign(payload, this.config.biometricSecret, {
      expiresIn: '5m', // Short-lived for security
      issuer: this.config.issuer,
      audience: 'finova-biometric',
      algorithm: this.config.algorithm,
    });

    // Store biometric verification in Redis
    this.redis.setex(`biometric:${payload.userId}`, 300, JSON.stringify(payload));

    return token;
  }

  /**
   * Generate KYC verification token
   */
  public generateKYCToken(payload: KYCPayload): string {
    const token = jwt.sign(payload, this.config.kycSecret, {
      expiresIn: '1h',
      issuer: this.config.issuer,
      audience: 'finova-kyc',
      algorithm: this.config.algorithm,
    });

    // Store KYC data in Redis
    this.redis.setex(`kyc:${payload.userId}`, 3600, JSON.stringify(payload));

    return token;
  }

  /**
   * Verify access token with comprehensive validation
   */
  public async verifyAccessToken(token: string): Promise<TokenPayload | null> {
    try {
      // Check if token is blacklisted
      if (this.blacklistedTokens.has(token)) {
        throw new Error('Token is blacklisted');
      }

      const decoded = jwt.verify(token, this.config.accessTokenSecret, {
        issuer: this.config.issuer,
        audience: this.config.audience,
        algorithms: [this.config.algorithm],
      }) as TokenPayload;

      // Verify session exists
      const sessionData = await this.redis.get(`session:${decoded.sessionId}`);
      if (!sessionData) {
        throw new Error('Session expired or invalid');
      }

      // Update last activity
      decoded.lastActivity = Date.now();
      await this.redis.setex(`session:${decoded.sessionId}`, 900, JSON.stringify(decoded));
      this.activeSessions.set(decoded.sessionId, decoded);

      // Anti-bot verification check
      if (decoded.antiBot.riskLevel === 'high' && 
          Date.now() - decoded.antiBot.lastVerification > 3600000) { // 1 hour
        throw new Error('Re-verification required for high-risk user');
      }

      return decoded;
    } catch (error) {
      logger.warn('Token verification failed', { error: error.message });
      return null;
    }
  }

  /**
   * Verify refresh token
   */
  public async verifyRefreshToken(token: string): Promise<any | null> {
    try {
      const decoded = jwt.verify(token, this.config.refreshTokenSecret, {
        issuer: this.config.issuer,
        audience: this.config.audience,
        algorithms: [this.config.algorithm],
      });

      // Check if refresh token exists in Redis
      const storedToken = await this.redis.get(`refresh:${decoded.sessionId}`);
      if (storedToken !== token) {
        throw new Error('Invalid refresh token');
      }

      return decoded;
    } catch (error) {
      logger.warn('Refresh token verification failed', { error: error.message });
      return null;
    }
  }

  /**
   * Verify biometric token
   */
  public async verifyBiometricToken(token: string): Promise<BiometricPayload | null> {
    try {
      const decoded = jwt.verify(token, this.config.biometricSecret, {
        issuer: this.config.issuer,
        audience: 'finova-biometric',
        algorithms: [this.config.algorithm],
      }) as BiometricPayload;

      // Verify biometric data exists
      const biometricData = await this.redis.get(`biometric:${decoded.userId}`);
      if (!biometricData) {
        throw new Error('Biometric verification expired');
      }

      return decoded;
    } catch (error) {
      logger.warn('Biometric token verification failed', { error: error.message });
      return null;
    }
  }

  /**
   * Verify KYC token
   */
  public async verifyKYCToken(token: string): Promise<KYCPayload | null> {
    try {
      const decoded = jwt.verify(token, this.config.kycSecret, {
        issuer: this.config.issuer,
        audience: 'finova-kyc',
        algorithms: [this.config.algorithm],
      }) as KYCPayload;

      // Verify KYC data exists
      const kycData = await this.redis.get(`kyc:${decoded.userId}`);
      if (!kycData) {
        throw new Error('KYC verification expired');
      }

      return decoded;
    } catch (error) {
      logger.warn('KYC token verification failed', { error: error.message });
      return null;
    }
  }

  /**
   * Refresh access token using refresh token
   */
  public async refreshAccessToken(refreshToken: string, newPayload?: Partial<TokenPayload>): Promise<{ accessToken: string; newRefreshToken: string } | null> {
    const refreshPayload = await this.verifyRefreshToken(refreshToken);
    if (!refreshPayload) {
      return null;
    }

    // Get existing session data
    const sessionData = await this.redis.get(`session:${refreshPayload.sessionId}`);
    if (!sessionData) {
      return null;
    }

    const existingPayload = JSON.parse(sessionData) as TokenPayload;
    
    // Create new session with updated data
    const updatedPayload: Omit<TokenPayload, 'sessionId' | 'lastActivity'> = {
      ...existingPayload,
      ...newPayload,
    };

    // Generate new tokens
    const newAccessToken = this.generateAccessToken(updatedPayload);
    const newRefreshToken = this.generateRefreshToken(refreshPayload.userId, refreshPayload.sessionId);

    // Remove old refresh token
    await this.redis.del(`refresh:${refreshPayload.sessionId}`);

    return {
      accessToken: newAccessToken,
      newRefreshToken,
    };
  }

  /**
   * Blacklist token (logout)
   */
  public async blacklistToken(token: string): Promise<void> {
    try {
      const decoded = jwt.decode(token) as any;
      if (decoded?.sessionId) {
        // Remove from active sessions
        this.activeSessions.delete(decoded.sessionId);
        
        // Remove from Redis
        await this.redis.del(`session:${decoded.sessionId}`);
        await this.redis.del(`refresh:${decoded.sessionId}`);
        
        // Add to blacklist with expiry matching token expiry
        const expiresAt = decoded.exp * 1000;
        const ttl = Math.max(0, expiresAt - Date.now()) / 1000;
        
        if (ttl > 0) {
          this.blacklistedTokens.add(token);
          await this.redis.setex(`blacklist:${token}`, Math.ceil(ttl), 'true');
        }
      }
      
      logger.info('Token blacklisted successfully', { sessionId: decoded?.sessionId });
    } catch (error) {
      logger.error('Failed to blacklist token', { error: error.message });
    }
  }

  /**
   * Update user session data
   */
  public async updateSession(sessionId: string, updates: Partial<TokenPayload>): Promise<void> {
    const sessionData = await this.redis.get(`session:${sessionId}`);
    if (sessionData) {
      const payload = JSON.parse(sessionData) as TokenPayload;
      const updatedPayload = { ...payload, ...updates, lastActivity: Date.now() };
      
      await this.redis.setex(`session:${sessionId}`, 900, JSON.stringify(updatedPayload));
      this.activeSessions.set(sessionId, updatedPayload);
    }
  }

  /**
   * Get active session data
   */
  public async getSession(sessionId: string): Promise<TokenPayload | null> {
    const sessionData = await this.redis.get(`session:${sessionId}`);
    return sessionData ? JSON.parse(sessionData) : null;
  }

  /**
   * Invalidate all user sessions
   */
  public async invalidateUserSessions(userId: string): Promise<void> {
    const sessions = await this.redis.keys(`session:*`);
    
    for (const sessionKey of sessions) {
      const sessionData = await this.redis.get(sessionKey);
      if (sessionData) {
        const payload = JSON.parse(sessionData) as TokenPayload;
        if (payload.userId === userId) {
          const sessionId = sessionKey.replace('session:', '');
          this.activeSessions.delete(sessionId);
          await this.redis.del(sessionKey);
          await this.redis.del(`refresh:${sessionId}`);
        }
      }
    }
    
    logger.info('All user sessions invalidated', { userId });
  }

  /**
   * Enhanced security check for suspicious activity
   */
  public async performSecurityCheck(payload: TokenPayload, request: any): Promise<boolean> {
    const riskFactors = [];
    
    // Check for unusual activity patterns
    if (payload.antiBot.riskLevel === 'high') {
      riskFactors.push('high_risk_user');
    }
    
    // Check device fingerprint consistency
    if (payload.deviceFingerprint && request.deviceFingerprint) {
      if (payload.deviceFingerprint !== request.deviceFingerprint) {
        riskFactors.push('device_mismatch');
      }
    }
    
    // Check for rapid successive requests (potential bot behavior)
    const lastActivity = Date.now() - payload.lastActivity;
    if (lastActivity < 100) { // Less than 100ms between requests
      riskFactors.push('suspicious_timing');
    }
    
    // Check for unusual mining patterns (related to project's mining system)
    const hoursSinceLastActivity = lastActivity / (1000 * 60 * 60);
    if (payload.xpLevel > 50 && hoursSinceLastActivity < 0.01) { // Very frequent activity for high-level users
      riskFactors.push('potential_bot_mining');
    }
    
    // Log security events
    if (riskFactors.length > 0) {
      logger.warn('Security risk detected', {
        userId: payload.userId,
        sessionId: payload.sessionId,
        riskFactors,
        humanScore: payload.antiBot.humanScore
      });
      
      // Update risk level if multiple factors detected
      if (riskFactors.length >= 2) {
        await this.updateSession(payload.sessionId, {
          antiBot: {
            ...payload.antiBot,
            riskLevel: 'high',
            lastVerification: Date.now()
          }
        });
      }
    }
    
    // Return false if critical risk factors detected
    return !riskFactors.includes('potential_bot_mining');
  }

  /**
   * Setup automatic cleanup of expired tokens and sessions
   */
  private setupCleanupScheduler(): void {
    // Clean up every 30 minutes
    setInterval(async () => {
      try {
        // Clean expired blacklisted tokens
        const expiredTokens = [];
        for (const token of this.blacklistedTokens) {
          try {
            jwt.verify(token, this.config.accessTokenSecret);
          } catch (error) {
            if (error.name === 'TokenExpiredError') {
              expiredTokens.push(token);
            }
          }
        }
        
        expiredTokens.forEach(token => this.blacklistedTokens.delete(token));
        
        // Clean expired sessions from memory
        for (const [sessionId, payload] of this.activeSessions) {
          if (Date.now() - payload.lastActivity > 900000) { // 15 minutes
            this.activeSessions.delete(sessionId);
          }
        }
        
        logger.info('JWT cleanup completed', { 
          expiredTokens: expiredTokens.length,
          activeSessions: this.activeSessions.size 
        });
      } catch (error) {
        logger.error('JWT cleanup failed', { error: error.message });
      }
    }, 30 * 60 * 1000);
  }

  /**
   * Generate token pair (access + refresh)
   */
  public generateTokenPair(payload: Omit<TokenPayload, 'sessionId' | 'lastActivity'>): {
    accessToken: string;
    refreshToken: string;
    expiresIn: number;
  } {
    const accessToken = this.generateAccessToken(payload);
    const decoded = jwt.decode(accessToken) as any;
    const refreshToken = this.generateRefreshToken(payload.userId, decoded.jti);
    
    return {
      accessToken,
      refreshToken,
      expiresIn: decoded.exp * 1000, // Convert to milliseconds
    };
  }

  /**
   * Validate mining permissions based on user tier and anti-bot status
   */
  public validateMiningPermissions(payload: TokenPayload): boolean {
    // Check if user is verified for mining
    if (!payload.kycVerified) {
      return false;
    }
    
    // Check anti-bot status
    if (payload.antiBot.riskLevel === 'high' && payload.antiBot.humanScore < 0.6) {
      return false;
    }
    
    // Check if user has been inactive too long (potential account compromise)
    const hoursSinceLastActivity = (Date.now() - payload.lastActivity) / (1000 * 60 * 60);
    if (hoursSinceLastActivity > 72) { // 3 days
      return false;
    }
    
    return true;
  }
}

// Export singleton instance
export const jwtManager = new JWTManager();

// Export configuration for external use
export const jwtConfig = {
  secret: process.env.JWT_ACCESS_SECRET,
  expiresIn: process.env.JWT_ACCESS_EXPIRY || '15m',
  issuer: process.env.JWT_ISSUER || 'finova-network',
  audience: process.env.JWT_AUDIENCE || 'finova-api',
};

// Export types
export type {
  TokenPayload,
  BiometricPayload,
  KYCPayload,
  JWTConfig,
};

export default jwtManager;
