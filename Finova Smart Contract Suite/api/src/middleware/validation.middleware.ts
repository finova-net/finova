import { Request, Response, NextFunction } from 'express';
import { body, param, query, validationResult, ValidationChain } from 'express-validator';
import { rateLimit } from 'express-rate-limit';
import jwt from 'jsonwebtoken';
import crypto from 'crypto';
import validator from 'validator';

// Types for validation
interface ValidationError {
  field: string;
  message: string;
  value?: any;
  code: string;
}

interface ValidatedRequest extends Request {
  user?: {
    id: string;
    walletAddress: string;
    kycVerified: boolean;
    level: number;
    rpTier: number;
    isStaking: boolean;
  };
  validatedData?: any;
}

// Constants from whitepaper
const FINOVA_CONSTANTS = {
  MAX_MINING_RATE: 0.1,
  MIN_MINING_RATE: 0.001,
  MAX_XP_LEVEL: 100,
  MAX_RP_TIER: 5,
  MAX_REFERRALS: 100,
  MAX_STAKE_AMOUNT: 1000000,
  MIN_STAKE_AMOUNT: 100,
  MAX_CONTENT_LENGTH: 2000,
  SUPPORTED_PLATFORMS: ['instagram', 'tiktok', 'youtube', 'facebook', 'twitter', 'x'],
  CARD_RARITIES: ['common', 'uncommon', 'rare', 'epic', 'legendary'],
  GUILD_SIZES: { min: 10, max: 50 },
};

// Custom validation functions
const customValidators = {
  isSolanaAddress: (value: string): boolean => {
    return /^[1-9A-HJ-NP-Za-km-z]{32,44}$/.test(value);
  },
  
  isFinovaAmount: (value: number): boolean => {
    return value >= 0 && value <= 1000000 && Number.isFinite(value);
  },
  
  isPlatformSupported: (platform: string): boolean => {
    return FINOVA_CONSTANTS.SUPPORTED_PLATFORMS.includes(platform.toLowerCase());
  },
  
  isValidLevel: (level: number): boolean => {
    return Number.isInteger(level) && level >= 1 && level <= FINOVA_CONSTANTS.MAX_XP_LEVEL;
  },
  
  isValidRPTier: (tier: number): boolean => {
    return Number.isInteger(tier) && tier >= 0 && tier <= FINOVA_CONSTANTS.MAX_RP_TIER;
  },
  
  isValidReferralCode: (code: string): boolean => {
    return /^[A-Z0-9]{6,12}$/.test(code);
  },
  
  isValidContentHash: (hash: string): boolean => {
    return /^[a-f0-9]{64}$/.test(hash);
  },
  
  isValidNFTMetadata: (metadata: any): boolean => {
    return metadata && 
           typeof metadata.name === 'string' && 
           typeof metadata.description === 'string' &&
           metadata.name.length > 0 && 
           metadata.name.length <= 100;
  }
};

// Rate limiting configurations
const rateLimits = {
  strict: rateLimit({
    windowMs: 15 * 60 * 1000, // 15 minutes
    max: 10, // 10 requests per window
    message: { error: 'Too many requests', code: 'RATE_LIMIT_EXCEEDED' },
    standardHeaders: true,
    legacyHeaders: false,
  }),
  
  normal: rateLimit({
    windowMs: 15 * 60 * 1000,
    max: 100,
    message: { error: 'Too many requests', code: 'RATE_LIMIT_EXCEEDED' },
  }),
  
  mining: rateLimit({
    windowMs: 60 * 1000, // 1 minute
    max: 1, // 1 mining request per minute
    message: { error: 'Mining rate limit exceeded', code: 'MINING_RATE_LIMIT' },
  }),
  
  social: rateLimit({
    windowMs: 5 * 60 * 1000, // 5 minutes
    max: 50, // 50 social actions per 5 minutes
    message: { error: 'Social action rate limit exceeded', code: 'SOCIAL_RATE_LIMIT' },
  })
};

// Main validation middleware class
export class ValidationMiddleware {
  // Authentication validation
  static validateAuth() {
    return [
      body('walletAddress')
        .custom(customValidators.isSolanaAddress)
        .withMessage('Invalid Solana wallet address'),
      body('signature')
        .isLength({ min: 64, max: 128 })
        .withMessage('Invalid signature length'),
      body('timestamp')
        .isNumeric()
        .custom((value) => {
          const now = Date.now();
          const timestamp = parseInt(value);
          return Math.abs(now - timestamp) < 300000; // 5 minutes tolerance
        })
        .withMessage('Invalid or expired timestamp'),
      ValidationMiddleware.handleValidationErrors
    ];
  }

  // User registration validation
  static validateUserRegistration() {
    return [
      body('walletAddress')
        .custom(customValidators.isSolanaAddress)
        .withMessage('Invalid Solana wallet address'),
      body('referralCode')
        .optional()
        .custom(customValidators.isValidReferralCode)
        .withMessage('Invalid referral code format'),
      body('deviceFingerprint')
        .isLength({ min: 32, max: 64 })
        .withMessage('Invalid device fingerprint'),
      body('email')
        .optional()
        .isEmail()
        .normalizeEmail()
        .withMessage('Invalid email address'),
      body('username')
        .optional()
        .isLength({ min: 3, max: 30 })
        .matches(/^[a-zA-Z0-9_]+$/)
        .withMessage('Username must be 3-30 characters, alphanumeric and underscore only'),
      ValidationMiddleware.handleValidationErrors
    ];
  }

  // KYC validation
  static validateKYC() {
    return [
      body('fullName')
        .isLength({ min: 2, max: 100 })
        .matches(/^[a-zA-Z\s]+$/)
        .withMessage('Full name must contain only letters and spaces'),
      body('dateOfBirth')
        .isISO8601()
        .custom((value) => {
          const age = new Date().getFullYear() - new Date(value).getFullYear();
          return age >= 18 && age <= 100;
        })
        .withMessage('Must be between 18 and 100 years old'),
      body('nationality')
        .isLength({ min: 2, max: 3 })
        .isAlpha()
        .withMessage('Invalid nationality code'),
      body('idNumber')
        .isLength({ min: 10, max: 20 })
        .isAlphanumeric()
        .withMessage('Invalid ID number format'),
      body('documentType')
        .isIn(['passport', 'national_id', 'driving_license'])
        .withMessage('Invalid document type'),
      body('biometricHash')
        .custom(customValidators.isValidContentHash)
        .withMessage('Invalid biometric hash'),
      ValidationMiddleware.handleValidationErrors
    ];
  }

  // Mining validation
  static validateMining() {
    return [
      rateLimits.mining,
      body('activityProof')
        .isObject()
        .withMessage('Activity proof must be an object'),
      body('activityProof.platform')
        .custom(customValidators.isPlatformSupported)
        .withMessage('Unsupported platform'),
      body('activityProof.contentHash')
        .optional()
        .custom(customValidators.isValidContentHash)
        .withMessage('Invalid content hash'),
      body('activityProof.timestamp')
        .isNumeric()
        .custom((value) => {
          const now = Date.now();
          const timestamp = parseInt(value);
          return timestamp <= now && (now - timestamp) < 86400000; // Within 24 hours
        })
        .withMessage('Invalid or expired activity timestamp'),
      body('humanVerification')
        .isObject()
        .withMessage('Human verification required'),
      ValidationMiddleware.handleValidationErrors
    ];
  }

  // XP activity validation
  static validateXPActivity() {
    return [
      rateLimits.social,
      body('activityType')
        .isIn(['post', 'comment', 'like', 'share', 'follow', 'story', 'video'])
        .withMessage('Invalid activity type'),
      body('platform')
        .custom(customValidators.isPlatformSupported)
        .withMessage('Unsupported platform'),
      body('contentData')
        .isObject()
        .withMessage('Content data must be an object'),
      body('contentData.text')
        .optional()
        .isLength({ max: FINOVA_CONSTANTS.MAX_CONTENT_LENGTH })
        .withMessage(`Content too long (max ${FINOVA_CONSTANTS.MAX_CONTENT_LENGTH} characters)`),
      body('contentData.mediaUrls')
        .optional()
        .isArray({ max: 10 })
        .withMessage('Maximum 10 media URLs allowed'),
      body('contentData.hashtags')
        .optional()
        .isArray({ max: 30 })
        .withMessage('Maximum 30 hashtags allowed'),
      body('qualityMetrics')
        .isObject()
        .withMessage('Quality metrics required'),
      ValidationMiddleware.handleValidationErrors
    ];
  }

  // Referral validation
  static validateReferral() {
    return [
      body('referralCode')
        .custom(customValidators.isValidReferralCode)
        .withMessage('Invalid referral code format'),
      body('refereeWallet')
        .custom(customValidators.isSolanaAddress)
        .withMessage('Invalid referee wallet address'),
      body('networkData')
        .optional()
        .isObject()
        .withMessage('Network data must be an object'),
      ValidationMiddleware.handleValidationErrors
    ];
  }

  // Staking validation
  static validateStaking() {
    return [
      body('amount')
        .isNumeric()
        .custom(customValidators.isFinovaAmount)
        .custom((value) => {
          return value >= FINOVA_CONSTANTS.MIN_STAKE_AMOUNT && 
                 value <= FINOVA_CONSTANTS.MAX_STAKE_AMOUNT;
        })
        .withMessage(`Stake amount must be between ${FINOVA_CONSTANTS.MIN_STAKE_AMOUNT} and ${FINOVA_CONSTANTS.MAX_STAKE_AMOUNT} FIN`),
      body('duration')
        .optional()
        .isIn([30, 90, 180, 365])
        .withMessage('Invalid staking duration (30, 90, 180, or 365 days)'),
      body('stakingTier')
        .optional()
        .isIn(['basic', 'premium', 'vip', 'guild_master', 'mythic'])
        .withMessage('Invalid staking tier'),
      ValidationMiddleware.handleValidationErrors
    ];
  }

  // NFT validation
  static validateNFT() {
    return [
      body('tokenId')
        .optional()
        .isNumeric()
        .withMessage('Invalid token ID'),
      body('metadata')
        .custom(customValidators.isValidNFTMetadata)
        .withMessage('Invalid NFT metadata'),
      body('rarity')
        .isIn(FINOVA_CONSTANTS.CARD_RARITIES)
        .withMessage('Invalid rarity level'),
      body('cardType')
        .isIn(['mining_boost', 'xp_accelerator', 'referral_power', 'profile_badge', 'achievement'])
        .withMessage('Invalid card type'),
      body('effectData')
        .isObject()
        .withMessage('Effect data must be an object'),
      body('effectData.multiplier')
        .optional()
        .isFloat({ min: 0.5, max: 10.0 })
        .withMessage('Invalid effect multiplier'),
      body('effectData.duration')
        .optional()
        .isInt({ min: 3600, max: 2592000 }) // 1 hour to 30 days
        .withMessage('Invalid effect duration'),
      ValidationMiddleware.handleValidationErrors
    ];
  }

  // Guild validation
  static validateGuild() {
    return [
      body('name')
        .isLength({ min: 3, max: 50 })
        .matches(/^[a-zA-Z0-9\s]+$/)
        .withMessage('Guild name must be 3-50 characters, alphanumeric and spaces only'),
      body('description')
        .optional()
        .isLength({ max: 500 })
        .withMessage('Description too long (max 500 characters)'),
      body('memberLimit')
        .isInt({ min: FINOVA_CONSTANTS.GUILD_SIZES.min, max: FINOVA_CONSTANTS.GUILD_SIZES.max })
        .withMessage(`Member limit must be between ${FINOVA_CONSTANTS.GUILD_SIZES.min} and ${FINOVA_CONSTANTS.GUILD_SIZES.max}`),
      body('requirements')
        .optional()
        .isObject()
        .withMessage('Requirements must be an object'),
      body('requirements.minLevel')
        .optional()
        .custom(customValidators.isValidLevel)
        .withMessage('Invalid minimum level requirement'),
      ValidationMiddleware.handleValidationErrors
    ];
  }

  // Social integration validation
  static validateSocialIntegration() {
    return [
      body('platform')
        .custom(customValidators.isPlatformSupported)
        .withMessage('Unsupported platform'),
      body('accessToken')
        .isLength({ min: 10, max: 500 })
        .withMessage('Invalid access token'),
      body('platformUserId')
        .isLength({ min: 1, max: 100 })
        .withMessage('Invalid platform user ID'),
      body('permissions')
        .isArray({ min: 1 })
        .withMessage('At least one permission required'),
      body('profileData')
        .isObject()
        .withMessage('Profile data must be an object'),
      ValidationMiddleware.handleValidationErrors
    ];
  }

  // Query parameter validation
  static validateQueryParams() {
    return [
      query('page')
        .optional()
        .isInt({ min: 1, max: 1000 })
        .withMessage('Invalid page number'),
      query('limit')
        .optional()
        .isInt({ min: 1, max: 100 })
        .withMessage('Invalid limit (1-100)'),
      query('sortBy')
        .optional()
        .isIn(['createdAt', 'updatedAt', 'level', 'mining', 'xp', 'rp'])
        .withMessage('Invalid sort field'),
      query('order')
        .optional()
        .isIn(['asc', 'desc'])
        .withMessage('Invalid sort order'),
      query('startDate')
        .optional()
        .isISO8601()
        .withMessage('Invalid start date'),
      query('endDate')
        .optional()
        .isISO8601()
        .withMessage('Invalid end date'),
      ValidationMiddleware.handleValidationErrors
    ];
  }

  // Parameter validation
  static validateParams(paramName: string, validationType: 'address' | 'id' | 'code') {
    const validators: { [key: string]: ValidationChain } = {
      address: param(paramName).custom(customValidators.isSolanaAddress),
      id: param(paramName).isNumeric(),
      code: param(paramName).custom(customValidators.isValidReferralCode)
    };

    return [
      validators[validationType].withMessage(`Invalid ${paramName}`),
      ValidationMiddleware.handleValidationErrors
    ];
  }

  // Handle validation errors
  static handleValidationErrors(req: Request, res: Response, next: NextFunction) {
    const errors = validationResult(req);
    
    if (!errors.isEmpty()) {
      const formattedErrors: ValidationError[] = errors.array().map(error => ({
        field: error.type === 'field' ? error.path : 'unknown',
        message: error.msg,
        value: error.type === 'field' ? error.value : undefined,
        code: 'VALIDATION_ERROR'
      }));

      return res.status(400).json({
        success: false,
        error: 'Validation failed',
        code: 'VALIDATION_ERROR',
        details: formattedErrors,
        timestamp: new Date().toISOString()
      });
    }

    next();
  }

  // Anti-bot validation
  static validateHumanProof() {
    return [
      body('humanProof')
        .isObject()
        .withMessage('Human proof required'),
      body('humanProof.challenge')
        .isString()
        .isLength({ min: 10, max: 100 })
        .withMessage('Invalid challenge'),
      body('humanProof.response')
        .isString()
        .isLength({ min: 1, max: 200 })
        .withMessage('Invalid response'),
      body('humanProof.timestamp')
        .isNumeric()
        .custom((value) => {
          const now = Date.now();
          const timestamp = parseInt(value);
          return Math.abs(now - timestamp) < 300000; // 5 minutes tolerance
        })
        .withMessage('Challenge expired'),
      body('deviceFingerprint')
        .isLength({ min: 32, max: 64 })
        .withMessage('Invalid device fingerprint'),
      ValidationMiddleware.handleValidationErrors
    ];
  }

  // File upload validation
  static validateFileUpload() {
    return (req: Request, res: Response, next: NextFunction) => {
      const allowedTypes = ['image/jpeg', 'image/png', 'image/webp', 'video/mp4'];
      const maxSize = 50 * 1024 * 1024; // 50MB

      if (!req.file) {
        return res.status(400).json({
          success: false,
          error: 'File is required',
          code: 'FILE_REQUIRED'
        });
      }

      if (!allowedTypes.includes(req.file.mimetype)) {
        return res.status(400).json({
          success: false,
          error: 'Invalid file type',
          code: 'INVALID_FILE_TYPE',
          allowed: allowedTypes
        });
      }

      if (req.file.size > maxSize) {
        return res.status(400).json({
          success: false,
          error: 'File too large',
          code: 'FILE_TOO_LARGE',
          maxSize: maxSize
        });
      }

      next();
    };
  }

  // Content validation for AI quality assessment
  static validateContent() {
    return [
      body('content')
        .isString()
        .isLength({ min: 1, max: FINOVA_CONSTANTS.MAX_CONTENT_LENGTH })
        .withMessage(`Content must be 1-${FINOVA_CONSTANTS.MAX_CONTENT_LENGTH} characters`),
      body('contentType')
        .isIn(['text', 'image', 'video', 'mixed'])
        .withMessage('Invalid content type'),
      body('platform')
        .custom(customValidators.isPlatformSupported)
        .withMessage('Unsupported platform'),
      body('engagementData')
        .optional()
        .isObject()
        .withMessage('Engagement data must be an object'),
      ValidationMiddleware.handleValidationErrors
    ];
  }

  // Transaction validation
  static validateTransaction() {
    return [
      body('transactionType')
        .isIn(['mining', 'staking', 'unstaking', 'nft_purchase', 'nft_sale', 'referral_reward'])
        .withMessage('Invalid transaction type'),
      body('amount')
        .isNumeric()
        .custom(customValidators.isFinovaAmount)
        .withMessage('Invalid amount'),
      body('signature')
        .isLength({ min: 64, max: 128 })
        .withMessage('Invalid transaction signature'),
      body('blockHash')
        .isLength({ min: 32, max: 64 })
        .withMessage('Invalid block hash'),
      ValidationMiddleware.handleValidationErrors
    ];
  }

  // Security validation for sensitive operations
  static validateSensitiveOperation() {
    return [
      body('confirmationCode')
        .isNumeric()
        .isLength({ min: 6, max: 6 })
        .withMessage('Invalid confirmation code'),
      body('twoFactorToken')
        .optional()
        .isNumeric()
        .isLength({ min: 6, max: 6 })
        .withMessage('Invalid 2FA token'),
      body('biometricProof')
        .optional()
        .custom(customValidators.isValidContentHash)
        .withMessage('Invalid biometric proof'),
      ValidationMiddleware.handleValidationErrors
    ];
  }
}

// Export rate limits for use in routes
export { rateLimits };

// Export custom validators for external use
export { customValidators };

// Export validation middleware
export default ValidationMiddleware;
