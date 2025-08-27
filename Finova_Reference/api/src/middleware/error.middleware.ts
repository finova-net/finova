/**
 * Finova Network - Error Middleware
 * Enterprise-grade error handling for the Finova API
 * Handles mining, XP, RP, NFT, social integration, and security errors
 */

import { Request, Response, NextFunction } from 'express';
import { ValidationError } from 'joi';
import { TokenExpiredError, JsonWebTokenError } from 'jsonwebtoken';
import { logger } from '../utils/logger';
import { ApiError } from '../types/api.types';

// Custom error classes for Finova-specific functionality
export class FinovaError extends Error {
  public statusCode: number;
  public code: string;
  public isOperational: boolean;
  public timestamp: Date;

  constructor(message: string, statusCode: number = 500, code: string = 'FINOVA_ERROR') {
    super(message);
    this.name = 'FinovaError';
    this.statusCode = statusCode;
    this.code = code;
    this.isOperational = true;
    this.timestamp = new Date();

    Error.captureStackTrace(this, this.constructor);
  }
}

export class MiningError extends FinovaError {
  constructor(message: string, code: string = 'MINING_ERROR') {
    super(message, 400, code);
    this.name = 'MiningError';
  }
}

export class XPError extends FinovaError {
  constructor(message: string, code: string = 'XP_ERROR') {
    super(message, 400, code);
    this.name = 'XPError';
  }
}

export class RPError extends FinovaError {
  constructor(message: string, code: string = 'RP_ERROR') {
    super(message, 400, code);
    this.name = 'RPError';
  }
}

export class NFTError extends FinovaError {
  constructor(message: string, code: string = 'NFT_ERROR') {
    super(message, 400, code);
    this.name = 'NFTError';
  }
}

export class SocialError extends FinovaError {
  constructor(message: string, code: string = 'SOCIAL_ERROR') {
    super(message, 400, code);
    this.name = 'SocialError';
  }
}

export class SecurityError extends FinovaError {
  constructor(message: string, code: string = 'SECURITY_ERROR') {
    super(message, 403, code);
    this.name = 'SecurityError';
  }
}

export class BlockchainError extends FinovaError {
  constructor(message: string, code: string = 'BLOCKCHAIN_ERROR') {
    super(message, 503, code);
    this.name = 'BlockchainError';
  }
}

// Error type mapping for consistent responses
const ERROR_TYPES = {
  // Authentication & Authorization
  'TokenExpiredError': { status: 401, code: 'TOKEN_EXPIRED' },
  'JsonWebTokenError': { status: 401, code: 'INVALID_TOKEN' },
  'UnauthorizedError': { status: 401, code: 'UNAUTHORIZED' },
  'ForbiddenError': { status: 403, code: 'FORBIDDEN' },
  
  // Validation
  'ValidationError': { status: 400, code: 'VALIDATION_ERROR' },
  'CastError': { status: 400, code: 'INVALID_ID_FORMAT' },
  
  // Mining System
  'MINING_COOLDOWN_ACTIVE': { status: 429, code: 'MINING_COOLDOWN' },
  'MINING_RATE_LIMIT': { status: 429, code: 'MINING_RATE_EXCEEDED' },
  'MINING_BOT_DETECTED': { status: 403, code: 'BOT_DETECTED' },
  'MINING_REGRESSION_LIMIT': { status: 400, code: 'REGRESSION_LIMIT_REACHED' },
  
  // XP System
  'XP_DAILY_LIMIT_REACHED': { status: 429, code: 'XP_LIMIT_EXCEEDED' },
  'XP_INVALID_ACTIVITY': { status: 400, code: 'INVALID_XP_ACTIVITY' },
  'XP_QUALITY_TOO_LOW': { status: 400, code: 'CONTENT_QUALITY_LOW' },
  
  // RP System
  'RP_CIRCULAR_REFERRAL': { status: 400, code: 'CIRCULAR_REFERRAL' },
  'RP_REFERRAL_LIMIT': { status: 400, code: 'REFERRAL_LIMIT_EXCEEDED' },
  'RP_INVALID_CODE': { status: 400, code: 'INVALID_REFERRAL_CODE' },
  
  // NFT System
  'NFT_INSUFFICIENT_FUNDS': { status: 402, code: 'INSUFFICIENT_BALANCE' },
  'NFT_NOT_FOUND': { status: 404, code: 'NFT_NOT_FOUND' },
  'NFT_ALREADY_USED': { status: 409, code: 'NFT_ALREADY_CONSUMED' },
  
  // Social Integration
  'SOCIAL_PLATFORM_ERROR': { status: 503, code: 'PLATFORM_UNAVAILABLE' },
  'SOCIAL_RATE_LIMIT': { status: 429, code: 'SOCIAL_RATE_LIMIT' },
  'SOCIAL_INVALID_TOKEN': { status: 401, code: 'SOCIAL_TOKEN_INVALID' },
  
  // Blockchain
  'BLOCKCHAIN_CONNECTION': { status: 503, code: 'BLOCKCHAIN_UNAVAILABLE' },
  'TRANSACTION_FAILED': { status: 500, code: 'TRANSACTION_ERROR' },
  'INSUFFICIENT_SOL': { status: 402, code: 'INSUFFICIENT_SOL' },
  
  // Database
  'SequelizeValidationError': { status: 400, code: 'DATABASE_VALIDATION' },
  'SequelizeUniqueConstraintError': { status: 409, code: 'DUPLICATE_ENTRY' },
  'SequelizeConnectionError': { status: 503, code: 'DATABASE_UNAVAILABLE' },
  
  // Rate Limiting
  'RATE_LIMIT_EXCEEDED': { status: 429, code: 'RATE_LIMIT' },
  
  // Generic
  'NotFoundError': { status: 404, code: 'NOT_FOUND' },
  'ConflictError': { status: 409, code: 'CONFLICT' },
  'BadRequestError': { status: 400, code: 'BAD_REQUEST' }
};

// Security-sensitive errors that shouldn't expose details
const SECURITY_SENSITIVE_CODES = [
  'SECURITY_ERROR',
  'BOT_DETECTED',
  'SUSPICIOUS_ACTIVITY',
  'KYC_VERIFICATION_FAILED',
  'ANTI_FRAUD_TRIGGERED'
];

// Response formatter for consistent API responses
const formatErrorResponse = (error: any, req: Request): ApiError => {
  const isDevelopment = process.env.NODE_ENV === 'development';
  const isSecuritySensitive = SECURITY_SENSITIVE_CODES.includes(error.code);
  
  return {
    success: false,
    error: {
      code: error.code || 'INTERNAL_ERROR',
      message: isSecuritySensitive && !isDevelopment 
        ? 'Access denied for security reasons' 
        : error.message,
      timestamp: error.timestamp || new Date().toISOString(),
      path: req.path,
      method: req.method,
      requestId: req.headers['x-request-id'] as string || generateRequestId(),
      ...(isDevelopment && !isSecuritySensitive && {
        stack: error.stack,
        details: error.details
      })
    }
  };
};

// Generate unique request ID for tracking
const generateRequestId = (): string => {
  return `fin_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
};

// Log error with appropriate level and context
const logError = (error: any, req: Request, res: Response) => {
  const logContext = {
    requestId: req.headers['x-request-id'] || generateRequestId(),
    userId: req.user?.id || 'anonymous',
    path: req.path,
    method: req.method,
    ip: req.ip,
    userAgent: req.headers['user-agent'],
    error: {
      name: error.name,
      message: error.message,
      code: error.code,
      stack: error.stack
    }
  };

  // Log level based on status code
  if (error.statusCode >= 500) {
    logger.error('Server Error', logContext);
  } else if (error.statusCode >= 400) {
    logger.warn('Client Error', logContext);
  } else {
    logger.info('Request Error', logContext);
  }

  // Special logging for mining/XP/RP system errors
  if (error instanceof MiningError || error instanceof XPError || error instanceof RPError) {
    logger.info('Finova System Error', {
      ...logContext,
      system: error.name.replace('Error', ''),
      userId: req.user?.id,
      userLevel: req.user?.xpLevel,
      referralTier: req.user?.rpTier
    });
  }

  // Security incident logging
  if (error instanceof SecurityError || SECURITY_SENSITIVE_CODES.includes(error.code)) {
    logger.error('Security Incident', {
      ...logContext,
      severity: 'HIGH',
      alertRequired: true,
      securityContext: {
        suspicious: true,
        requiresInvestigation: true
      }
    });
  }
};

// Main error middleware
export const errorHandler = (
  err: any,
  req: Request,
  res: Response,
  next: NextFunction
): void => {
  // Don't handle if response already sent
  if (res.headersSent) {
    return next(err);
  }

  let error = { ...err };
  error.message = err.message;

  // Handle specific error types
  if (err instanceof ValidationError) {
    const message = err.details.map(detail => detail.message).join(', ');
    error = new FinovaError(message, 400, 'VALIDATION_ERROR');
  }

  if (err instanceof TokenExpiredError) {
    error = new FinovaError('Token has expired', 401, 'TOKEN_EXPIRED');
  }

  if (err instanceof JsonWebTokenError) {
    error = new FinovaError('Invalid token', 401, 'INVALID_TOKEN');
  }

  // MongoDB/Mongoose errors
  if (err.name === 'CastError') {
    error = new FinovaError('Invalid resource ID format', 400, 'INVALID_ID');
  }

  if (err.code === 11000) {
    const field = Object.keys(err.keyValue)[0];
    error = new FinovaError(`Duplicate ${field} value`, 409, 'DUPLICATE_ENTRY');
  }

  // Sequelize errors (if using PostgreSQL)
  if (err.name === 'SequelizeValidationError') {
    const message = err.errors.map((e: any) => e.message).join(', ');
    error = new FinovaError(message, 400, 'VALIDATION_ERROR');
  }

  if (err.name === 'SequelizeUniqueConstraintError') {
    error = new FinovaError('Duplicate entry', 409, 'DUPLICATE_ENTRY');
  }

  // Set defaults for unknown errors
  if (!error.statusCode) {
    error.statusCode = 500;
  }

  if (!error.code) {
    error.code = 'INTERNAL_ERROR';
  }

  if (!error.timestamp) {
    error.timestamp = new Date();
  }

  // Log the error
  logError(error, req, res);

  // Format and send response
  const errorResponse = formatErrorResponse(error, req);
  
  res.status(error.statusCode).json(errorResponse);
};

// 404 handler for undefined routes
export const notFoundHandler = (req: Request, res: Response, next: NextFunction): void => {
  const error = new FinovaError(
    `Route ${req.originalUrl} not found`,
    404,
    'ROUTE_NOT_FOUND'
  );
  
  next(error);
};

// Async error wrapper for route handlers
export const asyncHandler = (fn: Function) => (
  req: Request,
  res: Response,
  next: NextFunction
) => {
  Promise.resolve(fn(req, res, next)).catch(next);
};

// Validation error formatter
export const formatValidationError = (errors: any[]): string => {
  return errors.map(error => {
    if (error.context) {
      return `${error.context.key}: ${error.message}`;
    }
    return error.message;
  }).join(', ');
};

// Rate limit error handler
export const rateLimitHandler = (req: Request, res: Response) => {
  const error = new FinovaError(
    'Too many requests, please try again later',
    429,
    'RATE_LIMIT_EXCEEDED'
  );
  
  logError(error, req, res);
  
  const errorResponse = formatErrorResponse(error, req);
  res.status(429).json(errorResponse);
};

// Graceful shutdown error handler
export const handleUncaughtExceptions = () => {
  process.on('uncaughtException', (err: Error) => {
    logger.error('Uncaught Exception', {
      error: {
        name: err.name,
        message: err.message,
        stack: err.stack
      }
    });
    
    // Graceful shutdown
    process.exit(1);
  });

  process.on('unhandledRejection', (reason: any, promise: Promise<any>) => {
    logger.error('Unhandled Rejection', {
      reason,
      promise: promise.toString()
    });
    
    // Graceful shutdown
    process.exit(1);
  });
};

// Error factory functions for common Finova errors
export const createMiningError = (message: string, code?: string) => 
  new MiningError(message, code);

export const createXPError = (message: string, code?: string) => 
  new XPError(message, code);

export const createRPError = (message: string, code?: string) => 
  new RPError(message, code);

export const createNFTError = (message: string, code?: string) => 
  new NFTError(message, code);

export const createSocialError = (message: string, code?: string) => 
  new SocialError(message, code);

export const createSecurityError = (message: string, code?: string) => 
  new SecurityError(message, code);

export const createBlockchainError = (message: string, code?: string) => 
  new BlockchainError(message, code);

// Export all error classes and utilities
export {
  ERROR_TYPES,
  SECURITY_SENSITIVE_CODES
};
