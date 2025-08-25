import { Request, Response, NextFunction } from 'express';
import rateLimit from 'express-rate-limit';
import RedisStore from 'rate-limit-redis';
import { createClient } from 'redis';
import { AuthRequest } from '../types/api.types';
import { logger } from '../utils/logger';

// Redis client for rate limiting
const redisClient = createClient({
  url: process.env.REDIS_URL || 'redis://localhost:6379',
  retry_strategy: (options) => Math.min(options.attempt * 100, 3000)
});

redisClient.on('error', (err) => logger.error('Redis Rate Limit Error:', err));
redisClient.connect().catch(console.error);

// Rate limit configurations
const RATE_LIMITS = {
  // API endpoints
  general: { windowMs: 15 * 60 * 1000, max: 100 }, // 100 requests per 15 minutes
  auth: { windowMs: 15 * 60 * 1000, max: 5 }, // 5 auth attempts per 15 minutes
  mining: { windowMs: 60 * 1000, max: 10 }, // 10 mining requests per minute
  xp: { windowMs: 60 * 1000, max: 50 }, // 50 XP activities per minute
  social: { windowMs: 60 * 1000, max: 30 }, // 30 social posts per minute
  nft: { windowMs: 5 * 60 * 1000, max: 20 }, // 20 NFT operations per 5 minutes
  referral: { windowMs: 60 * 1000, max: 15 }, // 15 referral operations per minute
  
  // User tier based limits
  bronze: { windowMs: 60 * 1000, max: 20 },
  silver: { windowMs: 60 * 1000, max: 35 },
  gold: { windowMs: 60 * 1000, max: 50 },
  platinum: { windowMs: 60 * 1000, max: 75 },
  diamond: { windowMs: 60 * 1000, max: 100 },
  mythic: { windowMs: 60 * 1000, max: 200 }
};

// Anti-bot detection thresholds
const SUSPICIOUS_THRESHOLDS = {
  requests_per_second: 10,
  identical_requests: 5,
  rapid_endpoint_switching: 20,
  unusual_patterns: 3
};

interface SuspiciousActivity {
  userId?: string;
  ip: string;
  userAgent: string;
  rapidRequests: number;
  identicalRequests: number;
  endpointSwitching: number;
  patternScore: number;
  lastActivity: Date;
}

const suspiciousActivities = new Map<string, SuspiciousActivity>();

// Create Redis store for distributed rate limiting
const redisStore = new RedisStore({
  // @ts-expect-error - RedisStore typing issue
  sendCommand: (...args: string[]) => redisClient.sendCommand(args),
});

// Dynamic rate limit based on user tier and behavior
export const dynamicRateLimit = (endpoint: keyof typeof RATE_LIMITS) => {
  return rateLimit({
    store: redisStore,
    windowMs: RATE_LIMITS[endpoint].windowMs,
    max: (req: AuthRequest) => {
      // Get user tier from authenticated request
      const userTier = req.user?.xpLevel || 1;
      const tierName = getUserTierName(userTier);
      
      // Base limit from endpoint
      let limit = RATE_LIMITS[endpoint].max;
      
      // Apply tier multiplier
      if (RATE_LIMITS[tierName as keyof typeof RATE_LIMITS]) {
        const tierLimit = RATE_LIMITS[tierName as keyof typeof RATE_LIMITS].max;
        limit = Math.max(limit, tierLimit);
      }
      
      // Apply staking bonus
      if (req.user?.stakingTier) {
        const stakingMultiplier = getStakingMultiplier(req.user.stakingTier);
        limit = Math.floor(limit * stakingMultiplier);
      }
      
      // Reduce limit for suspicious users
      const suspiciousKey = req.user?.id || req.ip;
      const suspicious = suspiciousActivities.get(suspiciousKey);
      if (suspicious && isSuspiciousUser(suspicious)) {
        limit = Math.floor(limit * 0.3); // 70% reduction for suspicious users
      }
      
      return limit;
    },
    keyGenerator: (req: AuthRequest) => {
      // Use user ID if authenticated, otherwise IP
      return req.user?.id || req.ip;
    },
    standardHeaders: true,
    legacyHeaders: false,
    handler: (req: Request, res: Response) => {
      logger.warn('Rate limit exceeded', {
        ip: req.ip,
        userId: (req as AuthRequest).user?.id,
        endpoint: req.path,
        userAgent: req.get('User-Agent')
      });
      
      res.status(429).json({
        error: 'Too Many Requests',
        message: 'Rate limit exceeded. Please try again later.',
        retryAfter: Math.ceil(RATE_LIMITS[endpoint].windowMs / 1000)
      });
    },
    skip: (req: AuthRequest) => {
      // Skip rate limiting for admin users
      return req.user?.role === 'admin';
    }
  });
};

// Anti-bot middleware with advanced pattern detection
export const antiBotMiddleware = (req: AuthRequest, res: Response, next: NextFunction) => {
  const key = req.user?.id || req.ip;
  const userAgent = req.get('User-Agent') || '';
  const now = new Date();
  
  // Get or create suspicious activity record
  let activity = suspiciousActivities.get(key) || {
    userId: req.user?.id,
    ip: req.ip,
    userAgent,
    rapidRequests: 0,
    identicalRequests: 0,
    endpointSwitching: 0,
    patternScore: 0,
    lastActivity: now
  };
  
  // Check for rapid requests (more than 10 per second)
  const timeDiff = now.getTime() - activity.lastActivity.getTime();
  if (timeDiff < 100) { // Less than 100ms between requests
    activity.rapidRequests++;
  } else if (timeDiff > 1000) { // Reset counter after 1 second
    activity.rapidRequests = Math.max(0, activity.rapidRequests - 1);
  }
  
  // Check for identical requests
  const requestSignature = `${req.method}:${req.path}:${JSON.stringify(req.body)}`;
  const lastSignature = (req as any).session?.lastSignature;
  if (requestSignature === lastSignature) {
    activity.identicalRequests++;
  } else {
    activity.identicalRequests = Math.max(0, activity.identicalRequests - 1);
    (req as any).session = { ...(req as any).session, lastSignature: requestSignature };
  }
  
  // Check for unusual endpoint switching patterns
  const endpoint = req.path;
  const lastEndpoint = (req as any).session?.lastEndpoint;
  if (endpoint !== lastEndpoint) {
    activity.endpointSwitching++;
    (req as any).session = { ...(req as any).session, lastEndpoint: endpoint };
  }
  
  // Calculate pattern score based on user agent and behavior
  activity.patternScore = calculatePatternScore(userAgent, activity);
  activity.lastActivity = now;
  
  // Update the activity record
  suspiciousActivities.set(key, activity);
  
  // Check if user is suspicious
  if (isSuspiciousUser(activity)) {
    logger.warn('Suspicious bot-like activity detected', {
      key,
      activity,
      endpoint: req.path
    });
    
    // Apply progressive penalties
    const penalty = getBotPenalty(activity);
    if (penalty.block) {
      return res.status(429).json({
        error: 'Suspicious Activity Detected',
        message: 'Your account has been temporarily restricted due to unusual activity patterns.',
        suspensionTime: penalty.suspensionMinutes
      });
    }
  }
  
  // Clean up old suspicious activity records (older than 1 hour)
  cleanupSuspiciousActivities();
  
  next();
};

// Specific rate limiters for different endpoints
export const authRateLimit = dynamicRateLimit('auth');
export const generalRateLimit = dynamicRateLimit('general');
export const miningRateLimit = dynamicRateLimit('mining');
export const xpRateLimit = dynamicRateLimit('xp');
export const socialRateLimit = dynamicRateLimit('social');
export const nftRateLimit = dynamicRateLimit('nft');
export const referralRateLimit = dynamicRateLimit('referral');

// IP-based rate limiting for unauthenticated requests
export const ipRateLimit = rateLimit({
  store: redisStore,
  windowMs: 15 * 60 * 1000, // 15 minutes
  max: 50, // 50 requests per 15 minutes per IP
  standardHeaders: true,
  legacyHeaders: false,
  keyGenerator: (req) => req.ip,
  handler: (req, res) => {
    res.status(429).json({
      error: 'Too Many Requests',
      message: 'IP rate limit exceeded. Please try again later.'
    });
  }
});

// Helper functions
function getUserTierName(xpLevel: number): string {
  if (xpLevel >= 101) return 'mythic';
  if (xpLevel >= 76) return 'diamond';
  if (xpLevel >= 51) return 'platinum';
  if (xpLevel >= 26) return 'gold';
  if (xpLevel >= 11) return 'silver';
  return 'bronze';
}

function getStakingMultiplier(stakingTier: number): number {
  const multipliers = [1.0, 1.2, 1.5, 1.8, 2.0, 2.5];
  return multipliers[Math.min(stakingTier, multipliers.length - 1)];
}

function calculatePatternScore(userAgent: string, activity: SuspiciousActivity): number {
  let score = 0;
  
  // Check for bot-like user agents
  const botPatterns = [
    /bot/i, /crawler/i, /spider/i, /scraper/i,
    /curl/i, /wget/i, /python/i, /java/i
  ];
  
  if (botPatterns.some(pattern => pattern.test(userAgent))) {
    score += 2;
  }
  
  // Check for missing or suspicious user agent
  if (!userAgent || userAgent.length < 10) {
    score += 1;
  }
  
  // Add activity-based scoring
  score += Math.min(activity.rapidRequests / 5, 3);
  score += Math.min(activity.identicalRequests / 3, 2);
  score += Math.min(activity.endpointSwitching / 10, 2);
  
  return score;
}

function isSuspiciousUser(activity: SuspiciousActivity): boolean {
  return (
    activity.rapidRequests > SUSPICIOUS_THRESHOLDS.requests_per_second ||
    activity.identicalRequests > SUSPICIOUS_THRESHOLDS.identical_requests ||
    activity.endpointSwitching > SUSPICIOUS_THRESHOLDS.rapid_endpoint_switching ||
    activity.patternScore > SUSPICIOUS_THRESHOLDS.unusual_patterns
  );
}

function getBotPenalty(activity: SuspiciousActivity): { block: boolean; suspensionMinutes: number } {
  const suspicionLevel = 
    (activity.rapidRequests / SUSPICIOUS_THRESHOLDS.requests_per_second) +
    (activity.identicalRequests / SUSPICIOUS_THRESHOLDS.identical_requests) +
    (activity.endpointSwitching / SUSPICIOUS_THRESHOLDS.rapid_endpoint_switching) +
    (activity.patternScore / SUSPICIOUS_THRESHOLDS.unusual_patterns);
  
  if (suspicionLevel > 4) {
    return { block: true, suspensionMinutes: 60 }; // 1 hour suspension
  } else if (suspicionLevel > 3) {
    return { block: true, suspensionMinutes: 30 }; // 30 minutes suspension
  } else if (suspicionLevel > 2) {
    return { block: true, suspensionMinutes: 15 }; // 15 minutes suspension
  }
  
  return { block: false, suspensionMinutes: 0 };
}

function cleanupSuspiciousActivities(): void {
  const oneHourAgo = new Date(Date.now() - 60 * 60 * 1000);
  
  for (const [key, activity] of suspiciousActivities.entries()) {
    if (activity.lastActivity < oneHourAgo) {
      suspiciousActivities.delete(key);
    }
  }
}

// Mining-specific rate limiting with exponential backoff
export const miningActionRateLimit = (req: AuthRequest, res: Response, next: NextFunction) => {
  const userId = req.user?.id;
  if (!userId) {
    return res.status(401).json({ error: 'Authentication required' });
  }
  
  const key = `mining_action:${userId}`;
  const now = Date.now();
  
  // Get user's last mining action from Redis
  redisClient.get(key).then((lastAction) => {
    const lastActionTime = lastAction ? parseInt(lastAction) : 0;
    const timeSinceLastAction = now - lastActionTime;
    
    // Minimum 10 seconds between mining actions
    const minInterval = 10 * 1000;
    
    if (timeSinceLastAction < minInterval) {
      const waitTime = Math.ceil((minInterval - timeSinceLastAction) / 1000);
      return res.status(429).json({
        error: 'Mining Action Rate Limit',
        message: `Please wait ${waitTime} seconds before next mining action.`,
        waitTime
      });
    }
    
    // Store current action time
    redisClient.setex(key, 300, now.toString()); // 5 minutes TTL
    next();
  }).catch((err) => {
    logger.error('Redis error in mining rate limit:', err);
    next(); // Continue on Redis error
  });
};

// Export cleanup function for graceful shutdown
export const cleanupRateLimit = async (): Promise<void> => {
  try {
    await redisClient.quit();
    suspiciousActivities.clear();
    logger.info('Rate limit cleanup completed');
  } catch (error) {
    logger.error('Error during rate limit cleanup:', error);
  }
};
