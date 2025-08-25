import { Router, Request, Response, NextFunction } from 'express';
import { body, param, query, validationResult } from 'express-validator';
import rateLimit from 'express-rate-limit';
import { authMiddleware } from '../middleware/auth.middleware';
import { kycMiddleware } from '../middleware/kyc.middleware';
import { validationMiddleware } from '../middleware/validation.middleware';
import { 
  createReferralCode, 
  applyReferralCode, 
  getReferralNetwork, 
  calculateReferralRewards,
  getReferralAnalytics,
  updateReferralTier,
  claimReferralRewards,
  getReferralLeaderboard,
  validateReferralCode,
  checkNetworkQuality,
  getNetworkStats
} from '../services/referral.service';
import { 
  trackReferralActivity,
  logReferralEvent,
  updateReferralMetrics 
} from '../services/analytics.service';
import { 
  detectSybilAttack,
  validateReferralNetwork,
  checkFraudulentActivity 
} from '../services/anti-bot.service';
import { asyncHandler } from '../utils/async-handler';
import { ApiResponse } from '../types/api.types';
import { ReferralTier, ReferralActivity, NetworkStats } from '../types/referral.types';

const router = Router();

// Rate limiting for referral operations
const referralRateLimit = rateLimit({
  windowMs: 15 * 60 * 1000, // 15 minutes
  max: 50, // 50 requests per 15 minutes
  message: 'Too many referral requests, please try again later'
});

const codeGenerationLimit = rateLimit({
  windowMs: 24 * 60 * 60 * 1000, // 24 hours
  max: 3, // 3 code generations per day
  message: 'Maximum referral code generations reached for today'
});

const applyReferralLimit = rateLimit({
  windowMs: 60 * 60 * 1000, // 1 hour
  max: 5, // 5 referral applications per hour
  message: 'Too many referral applications, please try again later'
});

// Input validation schemas
const createReferralCodeValidation = [
  body('customCode')
    .optional()
    .isLength({ min: 6, max: 20 })
    .matches(/^[A-Za-z0-9_-]+$/)
    .withMessage('Custom code must be 6-20 characters, alphanumeric with underscore/hyphen only'),
  body('description')
    .optional()
    .isLength({ max: 255 })
    .withMessage('Description must be less than 255 characters')
];

const applyReferralValidation = [
  body('referralCode')
    .notEmpty()
    .isLength({ min: 6, max: 20 })
    .withMessage('Referral code is required and must be 6-20 characters'),
  body('platform')
    .optional()
    .isIn(['app', 'instagram', 'tiktok', 'youtube', 'facebook', 'twitter'])
    .withMessage('Invalid platform specified')
];

const networkQueryValidation = [
  query('level')
    .optional()
    .isInt({ min: 1, max: 3 })
    .withMessage('Level must be 1, 2, or 3'),
  query('limit')
    .optional()
    .isInt({ min: 1, max: 100 })
    .withMessage('Limit must be between 1 and 100'),
  query('offset')
    .optional()
    .isInt({ min: 0 })
    .withMessage('Offset must be non-negative')
];

// Helper function to calculate RP value based on whitepaper formula
const calculateRPValue = async (userId: string): Promise<number> => {
  try {
    const networkData = await getReferralNetwork(userId, 3); // Get 3 levels
    
    // Direct_Referral_Points = Σ(Referral_Activity × Referral_Level × Time_Decay)
    const directRP = networkData.level1.reduce((sum, referral) => {
      const activityScore = referral.activityLevel || 1;
      const levelMultiplier = referral.level || 1;
      const timeDays = Math.floor((Date.now() - referral.joinedAt.getTime()) / (1000 * 60 * 60 * 24));
      const timeDecay = Math.max(0.5, 1 - (timeDays * 0.001)); // Gradual decay over time
      return sum + (activityScore * levelMultiplier * timeDecay);
    }, 0);

    // Indirect_Network_Points = Σ(L2_Activity × 0.3) + Σ(L3_Activity × 0.1)
    const l2Points = networkData.level2.reduce((sum, ref) => sum + (ref.activityLevel * 0.3), 0);
    const l3Points = networkData.level3.reduce((sum, ref) => sum + (ref.activityLevel * 0.1), 0);
    const indirectRP = l2Points + l3Points;

    // Network_Quality_Bonus = Network_Diversity × Average_Referral_Level × Retention_Rate
    const totalReferrals = networkData.level1.length;
    const activeReferrals = networkData.level1.filter(ref => ref.isActive).length;
    const retentionRate = totalReferrals > 0 ? activeReferrals / totalReferrals : 0;
    const avgLevel = networkData.level1.reduce((sum, ref) => sum + ref.level, 0) / Math.max(1, totalReferrals);
    const networkDiversity = Math.min(1.0, totalReferrals / 50); // Max diversity at 50 referrals
    
    const qualityBonus = networkDiversity * avgLevel * retentionRate;

    // Final RP calculation with regression factor
    const baseRP = directRP + indirectRP;
    const totalNetworkSize = networkData.level1.length + networkData.level2.length + networkData.level3.length;
    const networkQualityScore = retentionRate;
    const regressionFactor = Math.exp(-0.0001 * totalNetworkSize * networkQualityScore);
    
    return (baseRP * qualityBonus * regressionFactor) || 0;
  } catch (error) {
    console.error('Error calculating RP value:', error);
    return 0;
  }
};

// Routes

/**
 * POST /api/referral/create-code
 * Create a new referral code for the authenticated user
 */
router.post('/create-code', 
  referralRateLimit,
  codeGenerationLimit,
  authMiddleware,
  createReferralCodeValidation,
  validationMiddleware,
  asyncHandler(async (req: Request, res: Response) => {
    const { customCode, description } = req.body;
    const userId = req.user.id;

    // Check if user already has maximum codes
    const existingCodes = await getReferralNetwork(userId, 0);
    if (existingCodes.activeCodes >= 5) {
      return res.status(400).json({
        success: false,
        message: 'Maximum referral codes limit reached (5 codes per user)',
        data: null
      });
    }

    // Anti-fraud check
    const fraudCheck = await checkFraudulentActivity(userId, 'referral_code_creation');
    if (fraudCheck.isSuspicious) {
      return res.status(403).json({
        success: false,
        message: 'Account flagged for suspicious activity. Please contact support.',
        data: null
      });
    }

    const result = await createReferralCode(userId, customCode, description);

    // Track analytics
    await trackReferralActivity(userId, 'code_created', {
      codeId: result.id,
      customCode: customCode || false
    });

    res.status(201).json({
      success: true,
      message: 'Referral code created successfully',
      data: result
    });
  })
);

/**
 * POST /api/referral/apply-code
 * Apply a referral code for a new user
 */
router.post('/apply-code',
  applyReferralLimit,
  authMiddleware,
  applyReferralValidation,
  validationMiddleware,
  asyncHandler(async (req: Request, res: Response) => {
    const { referralCode, platform } = req.body;
    const userId = req.user.id;

    // Validate referral code exists and is active
    const codeValidation = await validateReferralCode(referralCode);
    if (!codeValidation.isValid) {
      return res.status(400).json({
        success: false,
        message: codeValidation.reason || 'Invalid referral code',
        data: null
      });
    }

    // Check if user already used a referral code
    const existingReferral = await getReferralNetwork(userId, 0);
    if (existingReferral.referredBy) {
      return res.status(400).json({
        success: false,
        message: 'User has already used a referral code',
        data: null
      });
    }

    // Sybil attack detection
    const sybilCheck = await detectSybilAttack(userId, codeValidation.referrerId);
    if (sybilCheck.detected) {
      return res.status(403).json({
        success: false,
        message: 'Suspicious referral activity detected',
        data: { reason: sybilCheck.reason }
      });
    }

    const result = await applyReferralCode(userId, referralCode, platform);

    // Update referrer's RP value
    const referrerRPValue = await calculateRPValue(codeValidation.referrerId);
    await updateReferralTier(codeValidation.referrerId, referrerRPValue);

    // Track analytics
    await trackReferralActivity(codeValidation.referrerId, 'referral_success', {
      newUserId: userId,
      platform,
      codeUsed: referralCode
    });

    await logReferralEvent('referral_applied', {
      referrerId: codeValidation.referrerId,
      refereeId: userId,
      code: referralCode,
      platform
    });

    res.status(200).json({
      success: true,
      message: 'Referral code applied successfully',
      data: result
    });
  })
);

/**
 * GET /api/referral/network
 * Get user's referral network with detailed statistics
 */
router.get('/network',
  referralRateLimit,
  authMiddleware,
  networkQueryValidation,
  validationMiddleware,
  asyncHandler(async (req: Request, res: Response) => {
    const userId = req.user.id;
    const level = parseInt(req.query.level as string) || 3;
    const limit = parseInt(req.query.limit as string) || 50;
    const offset = parseInt(req.query.offset as string) || 0;

    const networkData = await getReferralNetwork(userId, level, { limit, offset });
    const rpValue = await calculateRPValue(userId);
    const networkQuality = await checkNetworkQuality(userId);

    // Calculate tier information based on RP value
    let tier: ReferralTier = 'Explorer';
    let tierBenefits = {
      miningBonus: 0,
      referralBonus: 10,
      networkCap: 10
    };

    if (rpValue >= 50000) {
      tier = 'Ambassador';
      tierBenefits = { miningBonus: 200, referralBonus: 30, networkCap: Infinity };
    } else if (rpValue >= 15000) {
      tier = 'Leader';
      tierBenefits = { miningBonus: 100, referralBonus: 25, networkCap: 100 };
    } else if (rpValue >= 5000) {
      tier = 'Influencer';
      tierBenefits = { miningBonus: 50, referralBonus: 20, networkCap: 50 };
    } else if (rpValue >= 1000) {
      tier = 'Connector';
      tierBenefits = { miningBonus: 20, referralBonus: 15, networkCap: 25 };
    }

    res.status(200).json({
      success: true,
      message: 'Referral network retrieved successfully',
      data: {
        network: networkData,
        rpValue,
        tier,
        tierBenefits,
        networkQuality,
        statistics: {
          totalReferrals: networkData.level1.length,
          activeReferrals: networkData.level1.filter(r => r.isActive).length,
          totalNetworkSize: networkData.level1.length + networkData.level2.length + networkData.level3.length,
          retentionRate: networkQuality.retentionRate,
          averageActivity: networkQuality.averageActivityLevel
        }
      }
    });
  })
);

/**
 * GET /api/referral/analytics
 * Get detailed referral analytics and performance metrics
 */
router.get('/analytics',
  referralRateLimit,
  authMiddleware,
  asyncHandler(async (req: Request, res: Response) => {
    const userId = req.user.id;
    const timeframe = req.query.timeframe as string || '30d';

    const analytics = await getReferralAnalytics(userId, timeframe);
    const networkStats = await getNetworkStats(userId);

    res.status(200).json({
      success: true,
      message: 'Referral analytics retrieved successfully',
      data: {
        analytics,
        networkStats,
        trends: {
          referralGrowthRate: analytics.growthMetrics.referralGrowthRate,
          networkExpansionRate: analytics.growthMetrics.networkExpansionRate,
          qualityScore: analytics.qualityMetrics.averageQualityScore,
          retentionTrend: analytics.retentionMetrics.retentionTrend
        }
      }
    });
  })
);

/**
 * GET /api/referral/rewards
 * Get referral rewards information and history
 */
router.get('/rewards',
  referralRateLimit,
  authMiddleware,
  asyncHandler(async (req: Request, res: Response) => {
    const userId = req.user.id;
    const page = parseInt(req.query.page as string) || 1;
    const limit = parseInt(req.query.limit as string) || 20;

    const rewards = await calculateReferralRewards(userId, { page, limit });
    const rpValue = await calculateRPValue(userId);

    res.status(200).json({
      success: true,
      message: 'Referral rewards retrieved successfully',
      data: {
        currentRP: rpValue,
        pendingRewards: rewards.pending,
        claimedRewards: rewards.claimed,
        totalEarned: rewards.totalEarned,
        rewardHistory: rewards.history,
        nextClaimableAt: rewards.nextClaimableAt,
        estimatedDailyEarnings: rewards.estimatedDailyEarnings
      }
    });
  })
);

/**
 * POST /api/referral/claim-rewards
 * Claim available referral rewards
 */
router.post('/claim-rewards',
  referralRateLimit,
  authMiddleware,
  kycMiddleware, // KYC required for reward claims
  asyncHandler(async (req: Request, res: Response) => {
    const userId = req.user.id;

    // Check if rewards are claimable
    const rewards = await calculateReferralRewards(userId);
    if (rewards.pending <= 0) {
      return res.status(400).json({
        success: false,
        message: 'No rewards available to claim',
        data: null
      });
    }

    // Anti-fraud check
    const fraudCheck = await checkFraudulentActivity(userId, 'reward_claim');
    if (fraudCheck.isSuspicious) {
      return res.status(403).json({
        success: false,
        message: 'Account flagged for suspicious activity. Reward claim blocked.',
        data: null
      });
    }

    const claimResult = await claimReferralRewards(userId);

    // Track analytics
    await trackReferralActivity(userId, 'rewards_claimed', {
      amount: claimResult.claimedAmount,
      rewardType: 'referral_bonus'
    });

    res.status(200).json({
      success: true,
      message: 'Referral rewards claimed successfully',
      data: claimResult
    });
  })
);

/**
 * GET /api/referral/leaderboard
 * Get referral leaderboard with top performers
 */
router.get('/leaderboard',
  referralRateLimit,
  asyncHandler(async (req: Request, res: Response) => {
    const category = req.query.category as string || 'total_referrals';
    const timeframe = req.query.timeframe as string || 'all_time';
    const limit = parseInt(req.query.limit as string) || 100;

    const leaderboard = await getReferralLeaderboard(category, timeframe, limit);

    res.status(200).json({
      success: true,
      message: 'Referral leaderboard retrieved successfully',
      data: {
        category,
        timeframe,
        rankings: leaderboard,
        lastUpdated: new Date().toISOString()
      }
    });
  })
);

/**
 * GET /api/referral/validate-code/:code
 * Validate a referral code without applying it
 */
router.get('/validate-code/:code',
  referralRateLimit,
  param('code').isLength({ min: 6, max: 20 }).withMessage('Invalid code format'),
  validationMiddleware,
  asyncHandler(async (req: Request, res: Response) => {
    const { code } = req.params;

    const validation = await validateReferralCode(code);

    res.status(200).json({
      success: true,
      message: 'Referral code validation completed',
      data: {
        isValid: validation.isValid,
        reason: validation.reason,
        codeInfo: validation.isValid ? {
          createdAt: validation.createdAt,
          usageCount: validation.usageCount,
          maxUses: validation.maxUses,
          referrerInfo: {
            username: validation.referrerUsername,
            level: validation.referrerLevel,
            tier: validation.referrerTier
          }
        } : null
      }
    });
  })
);

/**
 * PUT /api/referral/tier
 * Manual tier update (admin or automatic system trigger)
 */
router.put('/tier',
  referralRateLimit,
  authMiddleware,
  asyncHandler(async (req: Request, res: Response) => {
    const userId = req.user.id;

    // Recalculate RP value
    const rpValue = await calculateRPValue(userId);
    
    // Update tier based on current RP value
    const tierUpdate = await updateReferralTier(userId, rpValue);

    // Update user metrics
    await updateReferralMetrics(userId);

    res.status(200).json({
      success: true,
      message: 'Referral tier updated successfully',
      data: {
        previousTier: tierUpdate.previousTier,
        newTier: tierUpdate.newTier,
        rpValue,
        tierBenefits: tierUpdate.tierBenefits,
        nextTierRequirement: tierUpdate.nextTierRequirement
      }
    });
  })
);

/**
 * GET /api/referral/network-health
 * Get network health and quality metrics
 */
router.get('/network-health',
  referralRateLimit,
  authMiddleware,
  asyncHandler(async (req: Request, res: Response) => {
    const userId = req.user.id;

    const networkValidation = await validateReferralNetwork(userId);
    const qualityMetrics = await checkNetworkQuality(userId);

    res.status(200).json({
      success: true,
      message: 'Network health metrics retrieved successfully',
      data: {
        overallHealth: networkValidation.healthScore,
        qualityMetrics,
        warnings: networkValidation.warnings,
        recommendations: networkValidation.recommendations,
        fraudRisk: networkValidation.fraudRisk,
        networkIntegrity: {
          authenticityScore: networkValidation.authenticityScore,
          diversityScore: networkValidation.diversityScore,
          activityScore: networkValidation.activityScore,
          retentionScore: networkValidation.retentionScore
        }
      }
    });
  })
);

// Error handling middleware specific to referral routes
router.use((error: any, req: Request, res: Response, next: NextFunction) => {
  console.error('Referral route error:', error);
  
  if (error.name === 'ValidationError') {
    return res.status(400).json({
      success: false,
      message: 'Validation error in referral operation',
      data: { errors: error.errors }
    });
  }

  if (error.code === 'REFERRAL_LIMIT_EXCEEDED') {
    return res.status(429).json({
      success: false,
      message: 'Referral operation limit exceeded',
      data: null
    });
  }

  if (error.code === 'SUSPICIOUS_ACTIVITY') {
    return res.status(403).json({
      success: false,
      message: 'Suspicious referral activity detected',
      data: { reason: error.reason }
    });
  }

  res.status(500).json({
    success: false,
    message: 'Internal server error in referral system',
    data: null
  });
});

export default router;
