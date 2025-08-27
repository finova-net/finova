import { Request, Response, NextFunction } from 'express';
import { JwtPayload } from 'jsonwebtoken';
import { User } from '../models/User.model';
import { logger } from '../utils/logger';
import { ApiError } from '../utils/ApiError';
import { KycService } from '../services/kyc.service';
import { AntiBot } from '../services/anti-bot.service';
import { RateLimiter } from '../utils/rateLimiter';

// Extend Request interface
interface AuthenticatedRequest extends Request {
  user?: User;
  fingerprint?: string;
  ipAddress?: string;
  userAgent?: string;
}

// KYC Status Enum
enum KycStatus {
  UNVERIFIED = 'unverified',
  PENDING = 'pending', 
  VERIFIED = 'verified',
  REJECTED = 'rejected',
  SUSPENDED = 'suspended'
}

// KYC Level Enum
enum KycLevel {
  BASIC = 'basic',      // Phone + Email
  STANDARD = 'standard', // + ID Document
  PREMIUM = 'premium',   // + Biometric + Address
  ENTERPRISE = 'enterprise' // + Enhanced Due Diligence
}

// Mining multipliers based on KYC status
const KYC_MINING_MULTIPLIERS = {
  [KycStatus.UNVERIFIED]: 0.5,
  [KycStatus.PENDING]: 0.8,
  [KycStatus.VERIFIED]: 1.2,
  [KycStatus.REJECTED]: 0.3,
  [KycStatus.SUSPENDED]: 0.0
};

const KYC_LEVEL_MULTIPLIERS = {
  [KycLevel.BASIC]: 1.0,
  [KycLevel.STANDARD]: 1.3,
  [KycLevel.PREMIUM]: 1.6,
  [KycLevel.ENTERPRISE]: 2.0
};

class KycMiddleware {
  private kycService: KycService;
  private antiBotService: AntiBot;
  private rateLimiter: RateLimiter;

  constructor() {
    this.kycService = new KycService();
    this.antiBotService = new AntiBot();
    this.rateLimiter = new RateLimiter({
      windowMs: 15 * 60 * 1000, // 15 minutes
      max: 100, // requests per window
      message: 'Too many KYC requests'
    });
  }

  // Basic KYC requirement (phone/email verified)
  requireBasicKyc = async (req: AuthenticatedRequest, res: Response, next: NextFunction) => {
    try {
      const user = req.user;
      if (!user) {
        throw new ApiError(401, 'Authentication required');
      }

      // Check basic verification
      if (!user.phoneVerified || !user.emailVerified) {
        return res.status(403).json({
          success: false,
          error: 'BASIC_KYC_REQUIRED',
          message: 'Phone and email verification required',
          requiredActions: [
            !user.phoneVerified ? 'VERIFY_PHONE' : null,
            !user.emailVerified ? 'VERIFY_EMAIL' : null
          ].filter(Boolean),
          kycLevel: 'basic'
        });
      }

      // Add KYC info to request
      req.user.kycMultiplier = this.calculateKycMultiplier(user);
      next();

    } catch (error) {
      logger.error('Basic KYC check failed:', error);
      next(error);
    }
  };

  // Standard KYC requirement (ID document verified)  
  requireStandardKyc = async (req: AuthenticatedRequest, res: Response, next: NextFunction) => {
    try {
      const user = req.user;
      if (!user) {
        throw new ApiError(401, 'Authentication required');
      }

      // Check standard KYC
      if (user.kycStatus !== KycStatus.VERIFIED || user.kycLevel < KycLevel.STANDARD) {
        return res.status(403).json({
          success: false,
          error: 'STANDARD_KYC_REQUIRED',
          message: 'ID document verification required',
          currentStatus: user.kycStatus,
          currentLevel: user.kycLevel,
          requiredLevel: 'standard',
          submissionUrl: `/api/kyc/submit-documents`
        });
      }

      req.user.kycMultiplier = this.calculateKycMultiplier(user);
      next();

    } catch (error) {
      logger.error('Standard KYC check failed:', error);
      next(error);
    }
  };

  // Premium KYC requirement (biometric + address verified)
  requirePremiumKyc = async (req: AuthenticatedRequest, res: Response, next: NextFunction) => {
    try {
      const user = req.user;
      if (!user) {
        throw new ApiError(401, 'Authentication required');
      }

      if (user.kycStatus !== KycStatus.VERIFIED || user.kycLevel < KycLevel.PREMIUM) {
        return res.status(403).json({
          success: false,
          error: 'PREMIUM_KYC_REQUIRED', 
          message: 'Premium KYC verification required for this feature',
          currentLevel: user.kycLevel,
          requiredLevel: 'premium',
          benefits: [
            'Higher mining rates (+60%)',
            'Access to premium features',
            'Reduced transaction fees',
            'Priority customer support'
          ],
          upgradeUrl: `/api/kyc/upgrade-premium`
        });
      }

      req.user.kycMultiplier = this.calculateKycMultiplier(user);
      next();

    } catch (error) {
      logger.error('Premium KYC check failed:', error);
      next(error);
    }
  };

  // Flexible KYC checker with configurable requirements
  requireKycLevel = (requiredLevel: KycLevel) => {
    return async (req: AuthenticatedRequest, res: Response, next: NextFunction) => {
      try {
        const user = req.user;
        if (!user) {
          throw new ApiError(401, 'Authentication required');
        }

        const userLevel = this.getKycLevelOrder(user.kycLevel);
        const requiredLevelOrder = this.getKycLevelOrder(requiredLevel);

        if (user.kycStatus !== KycStatus.VERIFIED || userLevel < requiredLevelOrder) {
          const requirements = this.getKycRequirements(requiredLevel);
          
          return res.status(403).json({
            success: false,
            error: 'KYC_LEVEL_INSUFFICIENT',
            message: `${requiredLevel.toUpperCase()} KYC verification required`,
            currentLevel: user.kycLevel,
            currentStatus: user.kycStatus,
            requiredLevel,
            requirements,
            benefits: this.getKycBenefits(requiredLevel),
            upgradeUrl: `/api/kyc/upgrade/${requiredLevel}`
          });
        }

        req.user.kycMultiplier = this.calculateKycMultiplier(user);
        next();

      } catch (error) {
        logger.error(`KYC level ${requiredLevel} check failed:`, error);
        next(error);
      }
    };
  };

  // Anti-bot KYC verification with behavioral analysis
  verifyHumanBehavior = async (req: AuthenticatedRequest, res: Response, next: NextFunction) => {
    try {
      const user = req.user;
      const fingerprint = req.fingerprint;
      const ipAddress = req.ipAddress;
      const userAgent = req.userAgent;

      if (!user) {
        throw new ApiError(401, 'Authentication required');
      }

      // Apply rate limiting
      await this.rateLimiter.consume(ipAddress);

      // Behavioral analysis
      const behaviorAnalysis = await this.antiBotService.analyzeBehavior({
        userId: user.id,
        fingerprint,
        ipAddress,
        userAgent,
        sessionData: req.session,
        activityHistory: user.recentActivity
      });

      // Check human probability score
      if (behaviorAnalysis.humanProbability < 0.7) {
        logger.warn(`Low human probability for user ${user.id}:`, {
          score: behaviorAnalysis.humanProbability,
          factors: behaviorAnalysis.suspiciousFacts,
          ipAddress
        });

        // Require additional verification
        return res.status(403).json({
          success: false,
          error: 'HUMAN_VERIFICATION_REQUIRED',
          message: 'Please complete human verification',
          verificationMethods: [
            'CAPTCHA_CHALLENGE',
            'BIOMETRIC_SCAN',
            'PHONE_VERIFICATION'
          ],
          humanScore: behaviorAnalysis.humanProbability,
          verificationUrl: `/api/kyc/human-verification`
        });
      }

      // Apply behavior-based mining penalty if suspicious
      if (behaviorAnalysis.humanProbability < 0.9) {
        const penalty = 1 - (0.9 - behaviorAnalysis.humanProbability);
        req.user.behaviorPenalty = Math.max(0.5, penalty);
      }

      next();

    } catch (error) {
      if (error.name === 'RateLimitError') {
        return res.status(429).json({
          success: false,
          error: 'RATE_LIMIT_EXCEEDED',
          message: 'Too many requests, please try again later'
        });
      }
      
      logger.error('Human verification failed:', error);
      next(error);
    }
  };

  // Country/Region restriction middleware
  requireRegionAccess = (allowedCountries: string[]) => {
    return async (req: AuthenticatedRequest, res: Response, next: NextFunction) => {
      try {
        const user = req.user;
        const ipCountry = req.headers['cf-ipcountry'] || req.headers['x-country-code'] || 'UNKNOWN';

        if (!user) {
          throw new ApiError(401, 'Authentication required');
        }

        // Check user's verified country (from KYC)
        const userCountry = user.kycData?.country || ipCountry;

        if (!allowedCountries.includes(userCountry as string)) {
          return res.status(403).json({
            success: false,
            error: 'REGION_NOT_SUPPORTED',
            message: 'This feature is not available in your region',
            userCountry,
            allowedCountries,
            ipCountry
          });
        }

        next();

      } catch (error) {
        logger.error('Region access check failed:', error);
        next(error);
      }
    };
  };

  // Mining eligibility checker
  requireMiningEligibility = async (req: AuthenticatedRequest, res: Response, next: NextFunction) => {
    try {
      const user = req.user;
      
      if (!user) {
        throw new ApiError(401, 'Authentication required');
      }

      // Check account status
      if (user.status === 'suspended' || user.status === 'banned') {
        return res.status(403).json({
          success: false,
          error: 'ACCOUNT_SUSPENDED',
          message: 'Your account is suspended and cannot participate in mining',
          suspensionReason: user.suspensionReason,
          appealUrl: `/api/support/appeal`
        });
      }

      // Check minimum KYC for mining  
      if (!user.phoneVerified) {
        return res.status(403).json({
          success: false,
          error: 'MINING_KYC_REQUIRED',
          message: 'Phone verification required for mining',
          requiredActions: ['VERIFY_PHONE'],
          miningRestriction: true
        });
      }

      // Calculate comprehensive mining multiplier
      const miningMultiplier = this.calculateMiningMultiplier(user);
      req.user.miningMultiplier = miningMultiplier;

      // Log mining eligibility
      logger.info(`Mining eligibility check for user ${user.id}:`, {
        kycStatus: user.kycStatus,
        kycLevel: user.kycLevel,
        miningMultiplier,
        humanScore: user.humanScore
      });

      next();

    } catch (error) {
      logger.error('Mining eligibility check failed:', error);
      next(error);
    }
  };

  // Staking eligibility (requires higher KYC)
  requireStakingEligibility = async (req: AuthenticatedRequest, res: Response, next: NextFunction) => {
    try {
      const user = req.user;
      
      if (!user) {
        throw new ApiError(401, 'Authentication required');
      }

      // Staking requires standard KYC minimum
      if (user.kycStatus !== KycStatus.VERIFIED || user.kycLevel < KycLevel.STANDARD) {
        return res.status(403).json({
          success: false,
          error: 'STAKING_KYC_REQUIRED',
          message: 'Standard KYC verification required for staking',
          currentLevel: user.kycLevel,
          requiredLevel: 'standard',
          benefits: [
            'Stake $FIN tokens',
            'Earn staking rewards (8-15% APY)',
            'Enhanced mining rates',
            'Governance participation'
          ],
          upgradeUrl: `/api/kyc/upgrade/standard`
        });
      }

      // Check for any staking restrictions
      if (user.stakingRestricted) {
        return res.status(403).json({
          success: false,
          error: 'STAKING_RESTRICTED',
          message: 'Your account has staking restrictions',
          restrictionReason: user.stakingRestrictionReason,
          restrictionExpiry: user.stakingRestrictionExpiry
        });
      }

      next();

    } catch (error) {
      logger.error('Staking eligibility check failed:', error);
      next(error);
    }
  };

  // Helper methods
  private calculateKycMultiplier(user: User): number {
    const statusMultiplier = KYC_MINING_MULTIPLIERS[user.kycStatus] || 0.5;
    const levelMultiplier = KYC_LEVEL_MULTIPLIERS[user.kycLevel] || 1.0;
    return statusMultiplier * levelMultiplier;
  }

  private calculateMiningMultiplier(user: User): number {
    const kycMultiplier = this.calculateKycMultiplier(user);
    const behaviorPenalty = user.behaviorPenalty || 1.0;
    const securityBonus = user.securityFeatures?.twoFactorEnabled ? 1.1 : 1.0;
    
    return kycMultiplier * behaviorPenalty * securityBonus;
  }

  private getKycLevelOrder(level: KycLevel): number {
    const order = {
      [KycLevel.BASIC]: 1,
      [KycLevel.STANDARD]: 2, 
      [KycLevel.PREMIUM]: 3,
      [KycLevel.ENTERPRISE]: 4
    };
    return order[level] || 0;
  }

  private getKycRequirements(level: KycLevel): string[] {
    const requirements = {
      [KycLevel.BASIC]: ['Phone verification', 'Email verification'],
      [KycLevel.STANDARD]: ['ID document upload', 'Selfie verification'],
      [KycLevel.PREMIUM]: ['Address verification', 'Biometric scan', 'Video call'],
      [KycLevel.ENTERPRISE]: ['Enhanced due diligence', 'Source of funds', 'Business verification']
    };
    return requirements[level] || [];
  }

  private getKycBenefits(level: KycLevel): string[] {
    const benefits = {
      [KycLevel.BASIC]: ['Basic mining (+0%)', 'Standard features'],
      [KycLevel.STANDARD]: ['Enhanced mining (+30%)', 'Staking access', 'NFT trading'],
      [KycLevel.PREMIUM]: ['Premium mining (+60%)', 'VIP features', 'Priority support'],
      [KycLevel.ENTERPRISE]: ['Maximum mining (+100%)', 'All features', 'Dedicated account manager']
    };
    return benefits[level] || [];
  }
}

// Export singleton instance
export const kycMiddleware = new KycMiddleware();

// Export individual middleware functions
export const {
  requireBasicKyc,
  requireStandardKyc, 
  requirePremiumKyc,
  requireKycLevel,
  verifyHumanBehavior,
  requireRegionAccess,
  requireMiningEligibility,
  requireStakingEligibility
} = kycMiddleware;

// Export types and enums
export { KycStatus, KycLevel, AuthenticatedRequest };
