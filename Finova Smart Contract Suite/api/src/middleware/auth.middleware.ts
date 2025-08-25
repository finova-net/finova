import { Request, Response, NextFunction } from 'express';
import jwt from 'jsonwebtoken';
import { createHash, createHmac } from 'crypto';
import rateLimit from 'express-rate-limit';
import { User } from '../models/User.model';
import { logger } from '../utils/logger';
import { RedisService } from '../services/redis.service';
import { AntiBot } from '../services/anti-bot.service';

// Environment variables
const JWT_SECRET = process.env.JWT_SECRET || 'finova-super-secret-key';
const JWT_REFRESH_SECRET = process.env.JWT_REFRESH_SECRET || 'finova-refresh-secret';
const JWT_EXPIRES_IN = process.env.JWT_EXPIRES_IN || '15m';
const REFRESH_EXPIRES_IN = process.env.REFRESH_EXPIRES_IN || '7d';

// Extended Request interface
export interface AuthenticatedRequest extends Request {
  user?: {
    id: string;
    walletAddress: string;
    email: string;
    kycStatus: 'pending' | 'verified' | 'rejected';
    role: 'user' | 'admin' | 'moderator';
    xpLevel: number;
    rpTier: number;
    stakingTier: number;
    humanScore: number;
    isActive: boolean;
    lastActivity: Date;
  };
  deviceFingerprint?: string;
  ipAddress?: string;
  sessionId?: string;
}

// Token payload interface
interface TokenPayload {
  userId: string;
  walletAddress: string;
  role: string;
  sessionId: string;
  deviceFingerprint: string;
  iat: number;
  exp: number;
}

// Rate limiting configuration
export const authRateLimit = rateLimit({
  windowMs: 15 * 60 * 1000, // 15 minutes
  max: 100, // 100 requests per window
  message: { error: 'Too many authentication requests, please try again later' },
  standardHeaders: true,
  legacyHeaders: false,
});

// Login rate limiting (more restrictive)
export const loginRateLimit = rateLimit({
  windowMs: 15 * 60 * 1000,
  max: 5, // 5 login attempts per window
  message: { error: 'Too many login attempts, please try again in 15 minutes' },
  skipSuccessfulRequests: true,
});

// Device fingerprinting
function generateDeviceFingerprint(req: Request): string {
  const userAgent = req.get('user-agent') || '';
  const acceptLanguage = req.get('accept-language') || '';
  const acceptEncoding = req.get('accept-encoding') || '';
  const ipAddress = req.ip || req.connection.remoteAddress || '';
  
  const fingerprintData = `${userAgent}|${acceptLanguage}|${acceptEncoding}|${ipAddress}`;
  return createHash('sha256').update(fingerprintData).digest('hex');
}

// Session ID generator
function generateSessionId(): string {
  return createHash('sha256')
    .update(`${Date.now()}${Math.random()}`)
    .digest('hex')
    .substring(0, 32);
}

// JWT token generation
export function generateTokens(user: any, deviceFingerprint: string) {
  const sessionId = generateSessionId();
  
  const accessPayload = {
    userId: user.id,
    walletAddress: user.walletAddress,
    role: user.role,
    sessionId,
    deviceFingerprint,
  };

  const refreshPayload = {
    userId: user.id,
    sessionId,
    deviceFingerprint,
    type: 'refresh',
  };

  const accessToken = jwt.sign(accessPayload, JWT_SECRET, {
    expiresIn: JWT_EXPIRES_IN,
    issuer: 'finova-network',
    audience: 'finova-app',
  });

  const refreshToken = jwt.sign(refreshPayload, JWT_REFRESH_SECRET, {
    expiresIn: REFRESH_EXPIRES_IN,
    issuer: 'finova-network',
    audience: 'finova-app',
  });

  return { accessToken, refreshToken, sessionId };
}

// Verify JWT token
async function verifyToken(token: string, secret: string): Promise<TokenPayload> {
  return new Promise((resolve, reject) => {
    jwt.verify(token, secret, {
      issuer: 'finova-network',
      audience: 'finova-app',
    }, (err, decoded) => {
      if (err) {
        reject(err);
      } else {
        resolve(decoded as TokenPayload);
      }
    });
  });
}

// Main authentication middleware
export const authenticateToken = async (
  req: AuthenticatedRequest,
  res: Response,
  next: NextFunction
): Promise<void> => {
  try {
    // Extract token from Authorization header
    const authHeader = req.headers.authorization;
    const token = authHeader && authHeader.split(' ')[1]; // Bearer TOKEN

    if (!token) {
      res.status(401).json({
        error: 'Access token required',
        code: 'TOKEN_MISSING',
      });
      return;
    }

    // Verify and decode token
    let decoded: TokenPayload;
    try {
      decoded = await verifyToken(token, JWT_SECRET);
    } catch (error) {
      if (error instanceof jwt.TokenExpiredError) {
        res.status(401).json({
          error: 'Token expired',
          code: 'TOKEN_EXPIRED',
        });
        return;
      } else if (error instanceof jwt.JsonWebTokenError) {
        res.status(401).json({
          error: 'Invalid token',
          code: 'TOKEN_INVALID',
        });
        return;
      } else {
        throw error;
      }
    }

    // Generate current device fingerprint
    const currentFingerprint = generateDeviceFingerprint(req);
    
    // Verify device fingerprint (security measure)
    if (decoded.deviceFingerprint !== currentFingerprint) {
      logger.warn('Device fingerprint mismatch', {
        userId: decoded.userId,
        expected: decoded.deviceFingerprint,
        actual: currentFingerprint,
        ip: req.ip,
      });
      
      res.status(401).json({
        error: 'Device fingerprint mismatch',
        code: 'DEVICE_MISMATCH',
      });
      return;
    }

    // Check if session is blacklisted (Redis)
    const redis = RedisService.getInstance();
    const isBlacklisted = await redis.get(`blacklist:session:${decoded.sessionId}`);
    
    if (isBlacklisted) {
      res.status(401).json({
        error: 'Session revoked',
        code: 'SESSION_REVOKED',
      });
      return;
    }

    // Fetch user from database with caching
    const cachedUser = await redis.get(`user:${decoded.userId}`);
    let user;

    if (cachedUser) {
      user = JSON.parse(cachedUser);
    } else {
      user = await User.findById(decoded.userId).select(
        'id walletAddress email kycStatus role xpLevel rpTier stakingTier humanScore isActive lastActivity'
      );
      
      if (!user) {
        res.status(401).json({
          error: 'User not found',
          code: 'USER_NOT_FOUND',
        });
        return;
      }

      // Cache user for 5 minutes
      await redis.setex(`user:${decoded.userId}`, 300, JSON.stringify(user));
    }

    // Check if user is active
    if (!user.isActive) {
      res.status(403).json({
        error: 'Account disabled',
        code: 'ACCOUNT_DISABLED',
      });
      return;
    }

    // Anti-bot check for suspicious activity
    const antiBot = new AntiBot();
    const suspiciousActivity = await antiBot.checkUserActivity(user.id, req.ip);
    
    if (suspiciousActivity.isSuspicious) {
      logger.warn('Suspicious user activity detected', {
        userId: user.id,
        ip: req.ip,
        reason: suspiciousActivity.reason,
        score: suspiciousActivity.score,
      });

      // If highly suspicious, require re-authentication
      if (suspiciousActivity.score > 0.8) {
        res.status(429).json({
          error: 'Suspicious activity detected. Please re-authenticate.',
          code: 'SUSPICIOUS_ACTIVITY',
        });
        return;
      }
    }

    // Update last activity timestamp in Redis
    await redis.set(`activity:${user.id}`, Date.now().toString(), 'EX', 3600);

    // Attach user data to request
    req.user = user;
    req.deviceFingerprint = currentFingerprint;
    req.ipAddress = req.ip;
    req.sessionId = decoded.sessionId;

    // Log successful authentication
    logger.info('User authenticated successfully', {
      userId: user.id,
      ip: req.ip,
      endpoint: req.originalUrl,
    });

    next();
  } catch (error) {
    logger.error('Authentication middleware error', {
      error: error.message,
      stack: error.stack,
      ip: req.ip,
      endpoint: req.originalUrl,
    });

    res.status(500).json({
      error: 'Authentication service error',
      code: 'AUTH_SERVICE_ERROR',
    });
  }
};

// Role-based authorization middleware
export const requireRole = (allowedRoles: string[]) => {
  return (req: AuthenticatedRequest, res: Response, next: NextFunction): void => {
    if (!req.user) {
      res.status(401).json({
        error: 'Authentication required',
        code: 'AUTH_REQUIRED',
      });
      return;
    }

    if (!allowedRoles.includes(req.user.role)) {
      res.status(403).json({
        error: 'Insufficient permissions',
        code: 'INSUFFICIENT_PERMISSIONS',
        required: allowedRoles,
        current: req.user.role,
      });
      return;
    }

    next();
  };
};

// KYC verification requirement
export const requireKYC = (req: AuthenticatedRequest, res: Response, next: NextFunction): void => {
  if (!req.user) {
    res.status(401).json({
      error: 'Authentication required',
      code: 'AUTH_REQUIRED',
    });
    return;
  }

  if (req.user.kycStatus !== 'verified') {
    res.status(403).json({
      error: 'KYC verification required',
      code: 'KYC_REQUIRED',
      currentStatus: req.user.kycStatus,
    });
    return;
  }

  next();
};

// Staking tier requirement
export const requireStakingTier = (minTier: number) => {
  return (req: AuthenticatedRequest, res: Response, next: NextFunction): void => {
    if (!req.user) {
      res.status(401).json({
        error: 'Authentication required',
        code: 'AUTH_REQUIRED',
      });
      return;
    }

    if (req.user.stakingTier < minTier) {
      res.status(403).json({
        error: 'Higher staking tier required',
        code: 'STAKING_TIER_REQUIRED',
        required: minTier,
        current: req.user.stakingTier,
      });
      return;
    }

    next();
  };
};

// Human verification requirement (anti-bot)
export const requireHumanVerification = (minScore: number = 0.7) => {
  return (req: AuthenticatedRequest, res: Response, next: NextFunction): void => {
    if (!req.user) {
      res.status(401).json({
        error: 'Authentication required',
        code: 'AUTH_REQUIRED',
      });
      return;
    }

    if (req.user.humanScore < minScore) {
      res.status(403).json({
        error: 'Human verification required',
        code: 'HUMAN_VERIFICATION_REQUIRED',
        required: minScore,
        current: req.user.humanScore,
      });
      return;
    }

    next();
  };
};

// Refresh token middleware
export const refreshToken = async (
  req: Request,
  res: Response,
  next: NextFunction
): Promise<void> => {
  try {
    const { refreshToken } = req.body;

    if (!refreshToken) {
      res.status(401).json({
        error: 'Refresh token required',
        code: 'REFRESH_TOKEN_MISSING',
      });
      return;
    }

    // Verify refresh token
    const decoded = await verifyToken(refreshToken, JWT_REFRESH_SECRET);
    
    if (decoded.type !== 'refresh') {
      res.status(401).json({
        error: 'Invalid refresh token',
        code: 'INVALID_REFRESH_TOKEN',
      });
      return;
    }

    // Check device fingerprint
    const currentFingerprint = generateDeviceFingerprint(req);
    if (decoded.deviceFingerprint !== currentFingerprint) {
      res.status(401).json({
        error: 'Device fingerprint mismatch',
        code: 'DEVICE_MISMATCH',
      });
      return;
    }

    // Fetch user
    const user = await User.findById(decoded.userId);
    if (!user || !user.isActive) {
      res.status(401).json({
        error: 'User not found or inactive',
        code: 'USER_INVALID',
      });
      return;
    }

    // Generate new tokens
    const tokens = generateTokens(user, currentFingerprint);

    // Blacklist old session
    const redis = RedisService.getInstance();
    await redis.setex(
      `blacklist:session:${decoded.sessionId}`,
      604800, // 7 days
      'revoked'
    );

    res.json({
      success: true,
      tokens,
      user: {
        id: user.id,
        walletAddress: user.walletAddress,
        email: user.email,
        role: user.role,
      },
    });
  } catch (error) {
    logger.error('Refresh token error', {
      error: error.message,
      stack: error.stack,
      ip: req.ip,
    });

    res.status(401).json({
      error: 'Invalid refresh token',
      code: 'REFRESH_TOKEN_INVALID',
    });
  }
};

// Logout middleware
export const logout = async (
  req: AuthenticatedRequest,
  res: Response,
  next: NextFunction
): Promise<void> => {
  try {
    if (req.sessionId) {
      // Blacklist current session
      const redis = RedisService.getInstance();
      await redis.setex(
        `blacklist:session:${req.sessionId}`,
        604800, // 7 days
        'logged_out'
      );

      logger.info('User logged out', {
        userId: req.user?.id,
        sessionId: req.sessionId,
        ip: req.ip,
      });
    }

    res.json({
      success: true,
      message: 'Logged out successfully',
    });
  } catch (error) {
    logger.error('Logout error', {
      error: error.message,
      userId: req.user?.id,
      ip: req.ip,
    });

    res.status(500).json({
      error: 'Logout service error',
      code: 'LOGOUT_ERROR',
    });
  }
};

export default {
  authenticateToken,
  requireRole,
  requireKYC,
  requireStakingTier,
  requireHumanVerification,
  refreshToken,
  logout,
  authRateLimit,
  loginRateLimit,
  generateTokens,
};
