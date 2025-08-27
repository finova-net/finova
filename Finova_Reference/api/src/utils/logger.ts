import winston from 'winston';
import DailyRotateFile from 'winston-daily-rotate-file';
import { Request } from 'express';

// Log levels for Finova Network operations
const logLevels = {
  error: 0,    // Critical errors, security incidents
  warn: 1,     // Bot detection warnings, rate limits
  info: 2,     // Mining operations, XP calculations
  http: 3,     // API requests, social integrations
  verbose: 4,  // Detailed operations
  debug: 5,    // Development debugging
  silly: 6     // Trace level logging
};

// Colors for console output
const logColors = {
  error: 'red',
  warn: 'yellow',
  info: 'green',
  http: 'magenta',
  verbose: 'grey',
  debug: 'white',
  silly: 'cyan'
};

winston.addColors(logColors);

// Custom format for Finova Network logs
const finovaFormat = winston.format.combine(
  winston.format.timestamp({
    format: 'YYYY-MM-DD HH:mm:ss.SSS'
  }),
  winston.format.errors({ stack: true }),
  winston.format.json(),
  winston.format.printf((info) => {
    const { timestamp, level, message, userId, walletAddress, operation, ...meta } = info;
    
    let logEntry = {
      timestamp,
      level: level.toUpperCase(),
      message,
      service: 'finova-api'
    };

    // Add Finova-specific context
    if (userId) logEntry = { ...logEntry, userId };
    if (walletAddress) logEntry = { ...logEntry, wallet: walletAddress };
    if (operation) logEntry = { ...logEntry, operation };
    
    // Add metadata
    if (Object.keys(meta).length > 0) {
      logEntry = { ...logEntry, meta };
    }

    return JSON.stringify(logEntry);
  })
);

// Console format for development
const consoleFormat = winston.format.combine(
  winston.format.colorize({ all: true }),
  winston.format.timestamp({
    format: 'HH:mm:ss'
  }),
  winston.format.printf((info) => {
    const { timestamp, level, message, userId, operation } = info;
    const userInfo = userId ? `[User:${userId}]` : '';
    const opInfo = operation ? `[Op:${operation}]` : '';
    return `${timestamp} ${level} ${userInfo}${opInfo} ${message}`;
  })
);

// Transport configurations
const transports: winston.transport[] = [];

// Console transport for development
if (process.env.NODE_ENV !== 'production') {
  transports.push(
    new winston.transports.Console({
      format: consoleFormat,
      level: 'debug'
    })
  );
}

// File transports for production
if (process.env.NODE_ENV === 'production') {
  // General application logs
  transports.push(
    new DailyRotateFile({
      filename: 'logs/finova-api-%DATE%.log',
      datePattern: 'YYYY-MM-DD',
      maxSize: '100m',
      maxFiles: '30d',
      format: finovaFormat,
      level: 'info'
    })
  );

  // Error logs
  transports.push(
    new DailyRotateFile({
      filename: 'logs/finova-errors-%DATE%.log',
      datePattern: 'YYYY-MM-DD',
      maxSize: '50m',
      maxFiles: '90d',
      format: finovaFormat,
      level: 'error'
    })
  );

  // Security and anti-bot logs
  transports.push(
    new DailyRotateFile({
      filename: 'logs/finova-security-%DATE%.log',
      datePattern: 'YYYY-MM-DD-HH',
      maxSize: '20m',
      maxFiles: '7d',
      format: finovaFormat,
      level: 'warn'
    })
  );

  // Mining and rewards logs
  transports.push(
    new DailyRotateFile({
      filename: 'logs/finova-mining-%DATE%.log',
      datePattern: 'YYYY-MM-DD',
      maxSize: '200m',
      maxFiles: '30d',
      format: finovaFormat,
      level: 'verbose'
    })
  );
}

// Create logger instance
const logger = winston.createLogger({
  levels: logLevels,
  transports,
  exitOnError: false
});

// Helper interfaces
interface FinovaLogContext {
  userId?: string;
  walletAddress?: string;
  operation?: string;
  platform?: string;
  amount?: number;
  xpGain?: number;
  rpGain?: number;
  requestId?: string;
  ip?: string;
  userAgent?: string;
}

interface SecurityLogData {
  suspiciousScore?: number;
  botProbability?: number;
  rateLimitHit?: boolean;
  failedAttempts?: number;
  blockedAction?: string;
  geoLocation?: string;
}

// Enhanced logger class with Finova-specific methods
class FinovaLogger {
  private baseLogger: winston.Logger;

  constructor(baseLogger: winston.Logger) {
    this.baseLogger = baseLogger;
  }

  // Standard logging methods
  error(message: string, context?: FinovaLogContext, error?: Error): void {
    this.baseLogger.error(message, {
      ...context,
      error: error ? {
        name: error.name,
        message: error.message,
        stack: error.stack
      } : undefined
    });
  }

  warn(message: string, context?: FinovaLogContext): void {
    this.baseLogger.warn(message, context);
  }

  info(message: string, context?: FinovaLogContext): void {
    this.baseLogger.info(message, context);
  }

  debug(message: string, context?: FinovaLogContext): void {
    this.baseLogger.debug(message, context);
  }

  // Finova-specific logging methods
  mining(message: string, data: {
    userId: string;
    walletAddress?: string;
    miningRate: number;
    totalEarned: number;
    phase?: string;
    regression?: number;
  }): void {
    this.baseLogger.info(message, {
      operation: 'mining',
      ...data
    });
  }

  xpActivity(message: string, data: {
    userId: string;
    platform: string;
    activityType: string;
    baseXP: number;
    finalXP: number;
    multipliers: Record<string, number>;
    level?: number;
  }): void {
    this.baseLogger.info(message, {
      operation: 'xp_activity',
      ...data
    });
  }

  referralActivity(message: string, data: {
    referrerId: string;
    refereeId?: string;
    rpGain: number;
    networkSize: number;
    tier: string;
    bonusMultiplier?: number;
  }): void {
    this.baseLogger.info(message, {
      operation: 'referral',
      ...data
    });
  }

  socialIntegration(message: string, data: {
    userId: string;
    platform: string;
    action: string;
    success: boolean;
    apiResponse?: any;
    rateLimited?: boolean;
  }): void {
    this.baseLogger.http(message, {
      operation: 'social_integration',
      ...data
    });
  }

  antiBot(message: string, data: SecurityLogData & {
    userId?: string;
    walletAddress?: string;
    action: string;
    humanScore?: number;
  }): void {
    this.baseLogger.warn(message, {
      operation: 'anti_bot',
      ...data
    });
  }

  staking(message: string, data: {
    userId: string;
    walletAddress: string;
    action: 'stake' | 'unstake' | 'claim';
    amount: number;
    tier?: string;
    apy?: number;
    duration?: number;
  }): void {
    this.baseLogger.info(message, {
      operation: 'staking',
      ...data
    });
  }

  nftActivity(message: string, data: {
    userId: string;
    walletAddress?: string;
    action: 'mint' | 'transfer' | 'use_card' | 'marketplace';
    tokenId?: string;
    cardType?: string;
    price?: number;
    rarity?: string;
  }): void {
    this.baseLogger.info(message, {
      operation: 'nft',
      ...data
    });
  }

  guildActivity(message: string, data: {
    userId: string;
    guildId: string;
    action: string;
    role?: string;
    contribution?: number;
    rewards?: number;
  }): void {
    this.baseLogger.info(message, {
      operation: 'guild',
      ...data
    });
  }

  apiRequest(req: Request, responseTime: number, statusCode: number): void {
    const userAgent = req.get('user-agent') || 'unknown';
    const ip = req.ip || req.connection.remoteAddress || 'unknown';
    
    this.baseLogger.http('API Request', {
      operation: 'api_request',
      method: req.method,
      url: req.originalUrl,
      statusCode,
      responseTime,
      ip,
      userAgent,
      userId: (req as any).user?.id,
      contentLength: req.get('content-length'),
      referer: req.get('referer')
    });
  }

  security(message: string, data: SecurityLogData & {
    userId?: string;
    walletAddress?: string;
    severity: 'low' | 'medium' | 'high' | 'critical';
    action?: string;
  }): void {
    const logLevel = data.severity === 'critical' ? 'error' : 'warn';
    
    this.baseLogger[logLevel](message, {
      operation: 'security',
      ...data
    });
  }

  blockchain(message: string, data: {
    operation: string;
    txHash?: string;
    blockNumber?: number;
    gasUsed?: number;
    success: boolean;
    error?: string;
    userId?: string;
    amount?: number;
  }): void {
    this.baseLogger.info(message, {
      operation: 'blockchain',
      ...data
    });
  }

  performance(message: string, data: {
    operation: string;
    duration: number;
    memory?: number;
    cpu?: number;
    cacheHit?: boolean;
    dbQueries?: number;
  }): void {
    this.baseLogger.verbose(message, {
      operation: 'performance',
      ...data
    });
  }

  // Utility methods
  formatAmount(amount: number, token: string = 'FIN'): string {
    return `${amount.toFixed(6)} ${token}`;
  }

  formatXP(xp: number): string {
    return `${xp.toLocaleString()} XP`;
  }

  formatRP(rp: number): string {
    return `${rp.toLocaleString()} RP`;
  }

  // Create child logger with default context
  child(defaultContext: FinovaLogContext): FinovaLogger {
    const childLogger = this.baseLogger.child(defaultContext);
    return new FinovaLogger(childLogger);
  }

  // Structured error logging for debugging
  logError(error: Error, context?: FinovaLogContext): void {
    this.error(`${error.name}: ${error.message}`, {
      ...context,
      stack: error.stack,
      timestamp: new Date().toISOString()
    }, error);
  }

  // Audit trail for important actions
  audit(action: string, data: {
    userId: string;
    resource: string;
    oldValue?: any;
    newValue?: any;
    ip?: string;
    userAgent?: string;
  }): void {
    this.baseLogger.warn(`AUDIT: ${action}`, {
      operation: 'audit',
      ...data,
      timestamp: new Date().toISOString()
    });
  }
}

// Create and export the enhanced logger instance
const finovaLogger = new FinovaLogger(logger);

// Request correlation ID for tracking
export const generateRequestId = (): string => {
  return `req_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
};

// Middleware helper for request logging
export const logRequest = (req: Request, responseTime: number, statusCode: number): void => {
  finovaLogger.apiRequest(req, responseTime, statusCode);
};

// Export logger and types
export default finovaLogger;
export { FinovaLogger, FinovaLogContext, SecurityLogData };

// Additional utility functions
export const logLevelsEnum = {
  ERROR: 'error',
  WARN: 'warn',
  INFO: 'info',
  HTTP: 'http',
  VERBOSE: 'verbose',
  DEBUG: 'debug',
  SILLY: 'silly'
} as const;

export type LogLevel = typeof logLevelsEnum[keyof typeof logLevelsEnum];
