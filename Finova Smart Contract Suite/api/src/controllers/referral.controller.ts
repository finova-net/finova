import { Request, Response, NextFunction } from 'express';
import { z } from 'zod';
import { ReferralService } from '../services/referral.service';
import { UserService } from '../services/user.service';
import { MiningService } from '../services/mining.service';
import { AntiBotService } from '../services/anti-bot.service';
import { logger } from '../utils/logger';
import { ApiError } from '../utils/errors';
import { calculateRegressionFactor, validateReferralCode } from '../utils/calculations';

// Validation schemas
const CreateReferralSchema = z.object({
  referralCode: z.string().min(6).max(12).regex(/^[A-Z0-9]+$/),
  customMessage: z.string().max(200).optional()
});

const JoinReferralSchema = z.object({
  referralCode: z.string().min(6).max(12),
  referrerUserId: z.string().uuid()
});

const UpdateReferralSchema = z.object({
  customMessage: z.string().max(200).optional(),
  isActive: z.boolean().optional()
});

export class ReferralController {
  private referralService: ReferralService;
  private userService: UserService;
  private miningService: MiningService;
  private antiBotService: AntiBotService;

  constructor() {
    this.referralService = new ReferralService();
    this.userService = new UserService();
    this.miningService = new MiningService();
    this.antiBotService = new AntiBotService();
  }

  /**
   * Create referral link with custom code
   * POST /api/referrals/create
   */
  async createReferral(req: Request, res: Response, next: NextFunction) {
    try {
      const userId = req.user.id;
      const validatedData = CreateReferralSchema.parse(req.body);

      // Check user eligibility (min Silver level)
      const user = await this.userService.getUserById(userId);
      if (user.xpLevel < 11) {
        throw new ApiError(403, 'Minimum Silver level (Level 11) required');
      }

      // Anti-bot verification
      const humanScore = await this.antiBotService.calculateHumanProbability(userId);
      if (humanScore < 0.7) {
        throw new ApiError(403, 'Anti-bot verification failed');
      }

      // Check referral code uniqueness
      const existingCode = await this.referralService.getReferralByCode(validatedData.referralCode);
      if (existingCode) {
        throw new ApiError(409, 'Referral code already exists');
      }

      const referral = await this.referralService.createReferral({
        userId,
        referralCode: validatedData.referralCode,
        customMessage: validatedData.customMessage,
        isActive: true,
        createdAt: new Date(),
        updatedAt: new Date()
      });

      logger.info(`Referral created: ${referral.id} by user ${userId}`);

      res.status(201).json({
        success: true,
        data: {
          referralId: referral.id,
          referralCode: referral.referralCode,
          referralLink: `${process.env.APP_BASE_URL}/join?ref=${referral.referralCode}`,
          customMessage: referral.customMessage,
          isActive: referral.isActive,
          createdAt: referral.createdAt
        }
      });
    } catch (error) {
      next(error);
    }
  }

  /**
   * Join network using referral code
   * POST /api/referrals/join
   */
  async joinWithReferral(req: Request, res: Response, next: NextFunction) {
    try {
      const newUserId = req.user.id;
      const validatedData = JoinReferralSchema.parse(req.body);

      // Validate referral code
      const referral = await this.referralService.getReferralByCode(validatedData.referralCode);
      if (!referral || !referral.isActive) {
        throw new ApiError(404, 'Invalid or inactive referral code');
      }

      // Prevent self-referral
      if (referral.userId === newUserId) {
        throw new ApiError(400, 'Cannot refer yourself');
      }

      // Check if user already has a referrer
      const existingReferral = await this.referralService.getUserReferrer(newUserId);
      if (existingReferral) {
        throw new ApiError(409, 'User already has a referrer');
      }

      // Anti-bot checks for both users
      const [newUserHumanScore, referrerHumanScore] = await Promise.all([
        this.antiBotService.calculateHumanProbability(newUserId),
        this.antiBotService.calculateHumanProbability(referral.userId)
      ]);

      if (newUserHumanScore < 0.6 || referrerHumanScore < 0.7) {
        throw new ApiError(403, 'Anti-bot verification failed');
      }

      // Create referral relationship
      const referralRelationship = await this.referralService.createReferralRelationship({
        referrerId: referral.userId,
        referredUserId: newUserId,
        referralCode: referral.referralCode,
        joinedAt: new Date()
      });

      // Award initial RP bonuses
      await this.awardReferralBonuses(referral.userId, newUserId, 'registration');

      logger.info(`User ${newUserId} joined via referral ${referral.referralCode} from ${referral.userId}`);

      res.status(200).json({
        success: true,
        data: {
          referrerId: referral.userId,
          referralCode: referral.referralCode,
          bonusAwarded: true,
          message: 'Successfully joined referral network'
        }
      });
    } catch (error) {
      next(error);
    }
  }

  /**
   * Get referral network statistics
   * GET /api/referrals/stats
   */
  async getReferralStats(req: Request, res: Response, next: NextFunction) {
    try {
      const userId = req.user.id;
      
      const [
        directReferrals,
        networkStats,
        rpBalance,
        tierInfo,
        monthlyEarnings
      ] = await Promise.all([
        this.referralService.getDirectReferrals(userId),
        this.referralService.getNetworkStats(userId),
        this.referralService.getRPBalance(userId),
        this.calculateRPTier(userId),
        this.referralService.getMonthlyEarnings(userId)
      ]);

      // Calculate network quality score
      const networkQualityScore = this.calculateNetworkQuality(networkStats);
      
      // Calculate regression factor
      const regressionFactor = calculateRegressionFactor(
        networkStats.totalNetworkSize,
        networkQualityScore
      );

      res.status(200).json({
        success: true,
        data: {
          userId,
          rpBalance: rpBalance.total,
          rpTier: tierInfo,
          networkStats: {
            directReferrals: directReferrals.length,
            activeReferrals: directReferrals.filter(r => r.isActive).length,
            totalNetworkSize: networkStats.totalNetworkSize,
            l2NetworkSize: networkStats.l2NetworkSize,
            l3NetworkSize: networkStats.l3NetworkSize,
            networkQualityScore,
            regressionFactor
          },
          monthlyEarnings: {
            rpEarned: monthlyEarnings.rpEarned,
            finEarned: monthlyEarnings.finEarned,
            xpEarned: monthlyEarnings.xpEarned
          },
          directReferrals: directReferrals.map(ref => ({
            userId: ref.referredUserId,
            username: ref.username,
            level: ref.xpLevel,
            isActive: ref.isActive,
            joinedAt: ref.joinedAt,
            monthlyContribution: ref.monthlyContribution
          }))
        }
      });
    } catch (error) {
      next(error);
    }
  }

  /**
   * Calculate RP rewards for referral activity
   * POST /api/referrals/calculate-rewards
   */
  async calculateRPRewards(req: Request, res: Response, next: NextFunction) {
    try {
      const userId = req.user.id;
      const { activityType, activityValue, referredUserId } = req.body;

      // Validate activity type
      const validActivities = ['mining', 'xp_gain', 'kyc_completion', 'achievement'];
      if (!validActivities.includes(activityType)) {
        throw new ApiError(400, 'Invalid activity type');
      }

      // Get network stats
      const networkStats = await this.referralService.getNetworkStats(userId);
      const rpTier = await this.calculateRPTier(userId);

      // Calculate RP rewards based on whitepaper formulas
      const rpRewards = this.calculateActivityRPRewards(
        activityType,
        activityValue,
        rpTier,
        networkStats
      );

      res.status(200).json({
        success: true,
        data: {
          userId,
          activityType,
          activityValue,
          rpTier: rpTier.tier,
          rpRewards: {
            directRP: rpRewards.directRP,
            networkRP: rpRewards.networkRP,
            qualityBonus: rpRewards.qualityBonus,
            totalRP: rpRewards.total
          },
          networkMultiplier: rpRewards.networkMultiplier,
          regressionFactor: rpRewards.regressionFactor
        }
      });
    } catch (error) {
      next(error);
    }
  }

  /**
   * Get referral network tree (up to 3 levels)
   * GET /api/referrals/network-tree
   */
  async getNetworkTree(req: Request, res: Response, next: NextFunction) {
    try {
      const userId = req.user.id;
      const depth = Math.min(parseInt(req.query.depth as string) || 3, 3);

      const networkTree = await this.buildNetworkTree(userId, depth);
      
      res.status(200).json({
        success: true,
        data: {
          userId,
          networkTree,
          totalNodes: this.countNetworkNodes(networkTree),
          maxDepth: depth
        }
      });
    } catch (error) {
      next(error);
    }
  }

  /**
   * Update referral settings
   * PUT /api/referrals/:referralId
   */
  async updateReferral(req: Request, res: Response, next: NextFunction) {
    try {
      const userId = req.user.id;
      const referralId = req.params.referralId;
      const validatedData = UpdateReferralSchema.parse(req.body);

      const referral = await this.referralService.getReferralById(referralId);
      if (!referral || referral.userId !== userId) {
        throw new ApiError(404, 'Referral not found or unauthorized');
      }

      const updatedReferral = await this.referralService.updateReferral(referralId, {
        ...validatedData,
        updatedAt: new Date()
      });

      logger.info(`Referral updated: ${referralId} by user ${userId}`);

      res.status(200).json({
        success: true,
        data: {
          referralId: updatedReferral.id,
          referralCode: updatedReferral.referralCode,
          customMessage: updatedReferral.customMessage,
          isActive: updatedReferral.isActive,
          updatedAt: updatedReferral.updatedAt
        }
      });
    } catch (error) {
      next(error);
    }
  }

  /**
   * Get referral leaderboard
   * GET /api/referrals/leaderboard
   */
  async getReferralLeaderboard(req: Request, res: Response, next: NextFunction) {
    try {
      const limit = Math.min(parseInt(req.query.limit as string) || 50, 100);
      const period = req.query.period as string || 'all_time';

      const leaderboard = await this.referralService.getLeaderboard(limit, period);

      res.status(200).json({
        success: true,
        data: {
          leaderboard: leaderboard.map((entry, index) => ({
            rank: index + 1,
            userId: entry.userId,
            username: entry.username,
            totalRP: entry.totalRP,
            networkSize: entry.networkSize,
            tier: entry.tier,
            badges: entry.badges
          })),
          period,
          timestamp: new Date().toISOString()
        }
      });
    } catch (error) {
      next(error);
    }
  }

  // Private helper methods

  private async awardReferralBonuses(referrerId: string, referredUserId: string, type: string) {
    const bonuses = {
      registration: { rpBonus: 50, finBonus: 0.1 },
      kyc_completion: { rpBonus: 100, finBonus: 0.2 },
      first_mining: { rpBonus: 25, finBonus: 0.05 }
    };

    const bonus = bonuses[type];
    if (!bonus) return;

    await Promise.all([
      this.referralService.awardRP(referrerId, bonus.rpBonus, `${type}_bonus`),
      this.miningService.awardBonus(referrerId, bonus.finBonus, `referral_${type}`)
    ]);
  }

  private async calculateRPTier(userId: string) {
    const rpBalance = await this.referralService.getRPBalance(userId);
    const networkStats = await this.referralService.getNetworkStats(userId);

    // RP tier calculation based on whitepaper
    const tiers = [
      { min: 0, max: 999, name: 'Explorer', miningBonus: 0, referralBonus: 0.10 },
      { min: 1000, max: 4999, name: 'Connector', miningBonus: 0.20, referralBonus: 0.15 },
      { min: 5000, max: 14999, name: 'Influencer', miningBonus: 0.50, referralBonus: 0.20 },
      { min: 15000, max: 49999, name: 'Leader', miningBonus: 1.00, referralBonus: 0.25 },
      { min: 50000, max: Infinity, name: 'Ambassador', miningBonus: 2.00, referralBonus: 0.30 }
    ];

    const tier = tiers.find(t => rpBalance.total >= t.min && rpBalance.total <= t.max);
    
    return {
      tier: tier.name,
      rpRequired: rpBalance.total,
      miningBonus: tier.miningBonus,
      referralBonus: tier.referralBonus,
      networkCap: this.getNetworkCap(tier.name),
      nextTier: tiers[tiers.indexOf(tier) + 1]
    };
  }

  private calculateNetworkQuality(networkStats: any): number {
    const activeRatio = networkStats.activeUsers / Math.max(networkStats.totalNetworkSize, 1);
    const retentionScore = networkStats.averageRetentionDays / 30; // Normalize to 30 days
    const activityScore = networkStats.averageActivityLevel / 100; // Normalize to 100%

    return Math.min(1.0, (activeRatio * 0.4 + retentionScore * 0.3 + activityScore * 0.3));
  }

  private calculateActivityRPRewards(activityType: string, value: number, tier: any, networkStats: any) {
    const baseMultipliers = {
      mining: 0.10,
      xp_gain: 0.05,
      kyc_completion: 1.0,
      achievement: 0.5
    };

    const baseRP = value * (baseMultipliers[activityType] || 0);
    const tierMultiplier = 1 + tier.referralBonus;
    const networkQuality = this.calculateNetworkQuality(networkStats);
    const regressionFactor = calculateRegressionFactor(networkStats.totalNetworkSize, networkQuality);

    const directRP = baseRP * tierMultiplier;
    const networkRP = directRP * 0.3 * (networkStats.l2NetworkSize * 0.3 + networkStats.l3NetworkSize * 0.1);
    const qualityBonus = networkQuality * 0.5;

    return {
      directRP,
      networkRP,
      qualityBonus,
      total: (directRP + networkRP) * (1 + qualityBonus) * regressionFactor,
      networkMultiplier: tierMultiplier,
      regressionFactor
    };
  }

  private async buildNetworkTree(userId: string, depth: number): Promise<any> {
    if (depth <= 0) return null;

    const directReferrals = await this.referralService.getDirectReferrals(userId);
    
    const tree = {
      userId,
      level: 0,
      children: await Promise.all(
        directReferrals.map(async (referral) => ({
          userId: referral.referredUserId,
          username: referral.username,
          level: 1,
          isActive: referral.isActive,
          joinedAt: referral.joinedAt,
          children: await this.buildNetworkTree(referral.referredUserId, depth - 1)
        }))
      )
    };

    return tree;
  }

  private countNetworkNodes(tree: any): number {
    if (!tree || !tree.children) return 0;
    return tree.children.length + tree.children.reduce((sum, child) => sum + this.countNetworkNodes(child), 0);
  }

  private getNetworkCap(tierName: string): number {
    const caps = {
      Explorer: 10,
      Connector: 25,
      Influencer: 50,
      Leader: 100,
      Ambassador: Infinity
    };
    return caps[tierName] || 10;
  }
}

export default new ReferralController();
