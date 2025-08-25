import { Request, Response } from 'express';
import { inject, injectable } from 'inversify';
import { 
  Controller, 
  Get, 
  Post, 
  Put, 
  Body, 
  Param, 
  Query, 
  UseMiddleware 
} from 'routing-controllers';
import { OpenAPI, ResponseSchema } from 'routing-controllers-openapi';
import { IsNumber, IsOptional, IsString, IsEnum, Min, Max } from 'class-validator';
import { MiningService } from '../services/mining.service';
import { XPService } from '../services/xp.service';
import { ReferralService } from '../services/referral.service';
import { AntiBotService } from '../services/anti-bot.service';
import { AIQualityService } from '../services/ai-quality.service';
import { BlockchainService } from '../services/blockchain.service';
import { AuthMiddleware } from '../middleware/auth.middleware';
import { RateLimitMiddleware } from '../middleware/rate-limit.middleware';
import { ValidateMiddleware } from '../middleware/validation.middleware';
import { Logger } from '../utils/logger';
import { ApiResponse, PaginatedResponse } from '../types/api.types';
import { 
  MiningSession, 
  MiningStats, 
  MiningPhase,
  MiningBooster,
  QualityScore 
} from '../types/mining.types';

// DTOs for request validation
export class StartMiningDto {
  @IsOptional()
  @IsString()
  deviceFingerprint?: string;

  @IsOptional()
  @IsString()
  location?: string;

  @IsOptional()
  @IsNumber()
  @Min(0)
  @Max(1)
  humanProbability?: number;
}

export class ClaimRewardsDto {
  @IsString()
  sessionId: string;

  @IsOptional()
  @IsString()
  walletAddress?: string;

  @IsOptional()
  @IsNumber()
  @Min(0)
  expectedAmount?: number;
}

export class UpdateMiningDto {
  @IsOptional()
  @IsEnum(['active', 'paused', 'stopped'])
  status?: 'active' | 'paused' | 'stopped';

  @IsOptional()
  @IsNumber()
  @Min(0)
  activityScore?: number;
}

export class ApplyBoosterDto {
  @IsString()
  boosterType: string;

  @IsString()
  nftId: string;

  @IsOptional()
  @IsNumber()
  @Min(1)
  duration?: number;
}

@injectable()
@Controller('/api/v1/mining')
@UseMiddleware(AuthMiddleware)
@UseMiddleware(RateLimitMiddleware)
export class MiningController {
  private readonly logger = new Logger('MiningController');

  constructor(
    @inject('MiningService') private miningService: MiningService,
    @inject('XPService') private xpService: XPService,
    @inject('ReferralService') private referralService: ReferralService,
    @inject('AntiBotService') private antiBotService: AntiBotService,
    @inject('AIQualityService') private aiQualityService: AIQualityService,
    @inject('BlockchainService') private blockchainService: BlockchainService
  ) {}

  /**
   * Start a new mining session with integrated XP, RP, and anti-bot validation
   */
  @Post('/start')
  @OpenAPI({ 
    summary: 'Start Mining Session',
    description: 'Initiates a new mining session with comprehensive validation and rate calculation',
    security: [{ bearerAuth: [] }]
  })
  @ResponseSchema(ApiResponse)
  @UseMiddleware(ValidateMiddleware)
  async startMining(
    @Body() dto: StartMiningDto,
    req: Request,
    res: Response
  ): Promise<ApiResponse<MiningSession>> {
    try {
      const userId = req.user.id;
      const timestamp = new Date();

      this.logger.info(`Starting mining session for user ${userId}`, { dto });

      // Step 1: Anti-bot validation
      const humanProbability = await this.antiBotService.validateUser({
        userId,
        deviceFingerprint: dto.deviceFingerprint,
        location: dto.location,
        timestamp,
        ipAddress: req.ip,
        userAgent: req.get('User-Agent')
      });

      if (humanProbability < 0.6) {
        return {
          success: false,
          message: 'Mining session rejected due to suspicious activity',
          error: 'ANTI_BOT_VALIDATION_FAILED',
          data: null
        };
      }

      // Step 2: Check existing active sessions
      const existingSession = await this.miningService.getActiveSession(userId);
      if (existingSession) {
        return {
          success: false,
          message: 'Active mining session already exists',
          error: 'ACTIVE_SESSION_EXISTS',
          data: existingSession
        };
      }

      // Step 3: Get user data for calculations
      const [userStats, xpData, rpData] = await Promise.all([
        this.miningService.getUserMiningStats(userId),
        this.xpService.getUserXPData(userId),
        this.referralService.getUserReferralData(userId)
      ]);

      // Step 4: Calculate mining rates using integrated formula
      const miningRates = await this.calculateIntegratedMiningRate({
        userId,
        userStats,
        xpData,
        rpData,
        humanProbability,
        timestamp
      });

      // Step 5: Create mining session
      const session = await this.miningService.createSession({
        userId,
        baseRate: miningRates.baseRate,
        xpMultiplier: miningRates.xpMultiplier,
        rpMultiplier: miningRates.rpMultiplier,
        qualityScore: miningRates.qualityScore,
        regressionFactor: miningRates.regressionFactor,
        humanProbability,
        deviceFingerprint: dto.deviceFingerprint,
        location: dto.location,
        timestamp
      });

      // Step 6: Log mining start event
      await this.miningService.logMiningEvent({
        userId,
        sessionId: session.id,
        eventType: 'MINING_STARTED',
        rates: miningRates,
        timestamp
      });

      this.logger.info(`Mining session started successfully`, {
        userId,
        sessionId: session.id,
        rates: miningRates
      });

      return {
        success: true,
        message: 'Mining session started successfully',
        data: session
      };

    } catch (error) {
      this.logger.error('Error starting mining session', error);
      return {
        success: false,
        message: 'Failed to start mining session',
        error: error.message,
        data: null
      };
    }
  }

  /**
   * Get current mining session status and accumulated rewards
   */
  @Get('/status')
  @OpenAPI({ 
    summary: 'Get Mining Status',
    description: 'Retrieves current mining session status with real-time calculations'
  })
  @ResponseSchema(ApiResponse)
  async getMiningStatus(req: Request): Promise<ApiResponse<MiningSession>> {
    try {
      const userId = req.user.id;
      const timestamp = new Date();

      const session = await this.miningService.getActiveSession(userId);
      if (!session) {
        return {
          success: false,
          message: 'No active mining session found',
          error: 'NO_ACTIVE_SESSION',
          data: null
        };
      }

      // Calculate real-time accumulated rewards
      const accumulatedRewards = await this.calculateAccumulatedRewards(session, timestamp);

      // Update session with current calculations
      const updatedSession = {
        ...session,
        accumulatedRewards,
        lastCalculated: timestamp,
        hoursActive: this.calculateHoursActive(session.startedAt, timestamp)
      };

      return {
        success: true,
        message: 'Mining status retrieved successfully',
        data: updatedSession
      };

    } catch (error) {
      this.logger.error('Error getting mining status', error);
      return {
        success: false,
        message: 'Failed to get mining status',
        error: error.message,
        data: null
      };
    }
  }

  /**
   * Claim accumulated mining rewards
   */
  @Post('/claim')
  @OpenAPI({ 
    summary: 'Claim Mining Rewards',
    description: 'Claims accumulated mining rewards and transfers to user wallet'
  })
  @ResponseSchema(ApiResponse)
  @UseMiddleware(ValidateMiddleware)
  async claimRewards(
    @Body() dto: ClaimRewardsDto,
    req: Request
  ): Promise<ApiResponse<{ transactionHash: string; amount: number }>> {
    try {
      const userId = req.user.id;
      const timestamp = new Date();

      this.logger.info(`Claiming rewards for user ${userId}`, { dto });

      // Step 1: Validate session
      const session = await this.miningService.getSession(dto.sessionId);
      if (!session || session.userId !== userId) {
        return {
          success: false,
          message: 'Invalid mining session',
          error: 'INVALID_SESSION',
          data: null
        };
      }

      // Step 2: Calculate final rewards with all bonuses
      const finalRewards = await this.calculateFinalRewards(session, timestamp);

      if (finalRewards.totalAmount <= 0) {
        return {
          success: false,
          message: 'No rewards available to claim',
          error: 'NO_REWARDS_AVAILABLE',
          data: null
        };
      }

      // Step 3: Anti-bot final validation
      const finalValidation = await this.antiBotService.validateClaim({
        userId,
        sessionId: session.id,
        claimAmount: finalRewards.totalAmount,
        timestamp
      });

      if (!finalValidation.isValid) {
        return {
          success: false,
          message: 'Claim rejected due to suspicious activity',
          error: 'CLAIM_VALIDATION_FAILED',
          data: null
        };
      }

      // Step 4: Execute blockchain transaction
      const walletAddress = dto.walletAddress || req.user.walletAddress;
      const transaction = await this.blockchainService.mintTokens({
        toAddress: walletAddress,
        amount: finalRewards.totalAmount,
        userId,
        sessionId: session.id,
        metadata: {
          baseRewards: finalRewards.baseAmount,
          xpBonus: finalRewards.xpBonus,
          rpBonus: finalRewards.rpBonus,
          qualityBonus: finalRewards.qualityBonus,
          specialBonuses: finalRewards.specialBonuses
        }
      });

      // Step 5: Update user stats and session
      await Promise.all([
        this.miningService.updateUserStats(userId, {
          totalMined: finalRewards.totalAmount,
          totalSessions: 1,
          lastClaimAt: timestamp
        }),
        this.miningService.completeSession(session.id, {
          finalAmount: finalRewards.totalAmount,
          transactionHash: transaction.signature,
          claimedAt: timestamp
        }),
        this.xpService.addXP(userId, {
          amount: Math.floor(finalRewards.totalAmount * 10), // 10 XP per $FIN
          source: 'MINING_CLAIM',
          sessionId: session.id
        }),
        this.referralService.distributeReferralRewards(userId, finalRewards.totalAmount)
      ]);

      this.logger.info(`Rewards claimed successfully`, {
        userId,
        sessionId: session.id,
        amount: finalRewards.totalAmount,
        txHash: transaction.signature
      });

      return {
        success: true,
        message: 'Rewards claimed successfully',
        data: {
          transactionHash: transaction.signature,
          amount: finalRewards.totalAmount
        }
      };

    } catch (error) {
      this.logger.error('Error claiming rewards', error);
      return {
        success: false,
        message: 'Failed to claim rewards',
        error: error.message,
        data: null
      };
    }
  }

  /**
   * Apply special card booster to mining session
   */
  @Post('/booster/apply')
  @OpenAPI({ 
    summary: 'Apply Mining Booster',
    description: 'Applies a special card booster to enhance mining rates'
  })
  @ResponseSchema(ApiResponse)
  @UseMiddleware(ValidateMiddleware)
  async applyBooster(
    @Body() dto: ApplyBoosterDto,
    req: Request
  ): Promise<ApiResponse<MiningBooster>> {
    try {
      const userId = req.user.id;

      // Validate NFT ownership and type
      const nftCard = await this.miningService.validateNFTCard(userId, dto.nftId);
      if (!nftCard || nftCard.type !== dto.boosterType) {
        return {
          success: false,
          message: 'Invalid or unauthorized NFT card',
          error: 'INVALID_NFT_CARD',
          data: null
        };
      }

      // Get active session
      const session = await this.miningService.getActiveSession(userId);
      if (!session) {
        return {
          success: false,
          message: 'No active mining session found',
          error: 'NO_ACTIVE_SESSION',
          data: null
        };
      }

      // Apply booster
      const booster = await this.miningService.applyBooster({
        sessionId: session.id,
        boosterType: dto.boosterType,
        nftId: dto.nftId,
        duration: dto.duration || nftCard.defaultDuration,
        multiplier: nftCard.multiplier,
        userId
      });

      // Burn single-use NFT if applicable
      if (nftCard.singleUse) {
        await this.blockchainService.burnNFT(dto.nftId, userId);
      }

      return {
        success: true,
        message: 'Booster applied successfully',
        data: booster
      };

    } catch (error) {
      this.logger.error('Error applying booster', error);
      return {
        success: false,
        message: 'Failed to apply booster',
        error: error.message,
        data: null
      };
    }
  }

  /**
   * Get mining statistics and analytics
   */
  @Get('/stats')
  @OpenAPI({ 
    summary: 'Get Mining Statistics',
    description: 'Retrieves comprehensive mining statistics for the user'
  })
  @ResponseSchema(ApiResponse)
  async getMiningStats(
    @Query('period') period: string = '30d',
    req: Request
  ): Promise<ApiResponse<MiningStats>> {
    try {
      const userId = req.user.id;

      const stats = await this.miningService.getMiningStats(userId, {
        period,
        includeComparisons: true,
        includeProjections: true
      });

      return {
        success: true,
        message: 'Mining statistics retrieved successfully',
        data: stats
      };

    } catch (error) {
      this.logger.error('Error getting mining stats', error);
      return {
        success: false,
        message: 'Failed to get mining statistics',
        error: error.message,
        data: null
      };
    }
  }

  /**
   * Get current mining phase and network statistics
   */
  @Get('/phase')
  @OpenAPI({ 
    summary: 'Get Mining Phase Info',
    description: 'Retrieves current mining phase and network statistics'
  })
  @ResponseSchema(ApiResponse)
  async getMiningPhase(): Promise<ApiResponse<MiningPhase>> {
    try {
      const phase = await this.miningService.getCurrentPhase();
      return {
        success: true,
        message: 'Mining phase retrieved successfully',
        data: phase
      };

    } catch (error) {
      this.logger.error('Error getting mining phase', error);
      return {
        success: false,
        message: 'Failed to get mining phase',
        error: error.message,
        data: null
      };
    }
  }

  /**
   * Update mining session (pause/resume)
   */
  @Put('/session/:sessionId')
  @OpenAPI({ 
    summary: 'Update Mining Session',
    description: 'Updates mining session status (pause, resume, stop)'
  })
  @ResponseSchema(ApiResponse)
  @UseMiddleware(ValidateMiddleware)
  async updateMiningSession(
    @Param('sessionId') sessionId: string,
    @Body() dto: UpdateMiningDto,
    req: Request
  ): Promise<ApiResponse<MiningSession>> {
    try {
      const userId = req.user.id;

      const session = await this.miningService.updateSession(sessionId, userId, dto);

      return {
        success: true,
        message: 'Mining session updated successfully',
        data: session
      };

    } catch (error) {
      this.logger.error('Error updating mining session', error);
      return {
        success: false,
        message: 'Failed to update mining session',
        error: error.message,
        data: null
      };
    }
  }

  // Private helper methods

  /**
   * Calculate integrated mining rate using the master formula:
   * Final_Reward = Base_Mining_Rate × XP_Multiplier × RP_Multiplier × Quality_Score × Network_Regression
   */
  private async calculateIntegratedMiningRate(params: {
    userId: string;
    userStats: any;
    xpData: any;
    rpData: any;
    humanProbability: number;
    timestamp: Date;
  }) {
    const { userId, userStats, xpData, rpData, humanProbability, timestamp } = params;

    // Get current mining phase
    const phase = await this.miningService.getCurrentPhase();
    
    // Base mining rate based on current phase
    let baseRate = phase.baseRate;

    // Pioneer bonus (Finizen bonus from whitepaper)
    const pioneerBonus = Math.max(1.0, 2.0 - (phase.totalUsers / 1000000));

    // XP Level multiplier (1.0x - 5.0x based on level)
    const xpMultiplier = this.calculateXPMultiplier(xpData.level, xpData.totalXP);

    // RP Network multiplier (1.0x - 3.0x based on referral tier)
    const rpMultiplier = this.calculateRPMultiplier(rpData.tier, rpData.networkSize);

    // Quality score based on recent activity
    const recentActivity = await this.miningService.getRecentActivity(userId, 7); // 7 days
    const qualityScore = await this.aiQualityService.calculateUserQualityScore(
      userId, 
      recentActivity
    );

    // Network regression factor (anti-whale mechanism)
    const regressionFactor = Math.exp(-0.001 * userStats.totalHoldings);

    // Security bonus for KYC verification
    const securityBonus = userStats.isKYCVerified ? 1.2 : 0.8;

    // Human probability penalty
    const humanBonus = Math.pow(humanProbability, 2); // Quadratic penalty for suspicious activity

    // Calculate final rate
    const finalRate = baseRate * pioneerBonus * xpMultiplier * rpMultiplier * 
                     qualityScore * regressionFactor * securityBonus * humanBonus;

    return {
      baseRate: finalRate,
      xpMultiplier,
      rpMultiplier,
      qualityScore,
      regressionFactor,
      pioneerBonus,
      securityBonus,
      humanBonus,
      components: {
        phase: phase.name,
        baseRate: phase.baseRate,
        totalUsers: phase.totalUsers,
        userLevel: xpData.level,
        referralTier: rpData.tier,
        totalHoldings: userStats.totalHoldings,
        isKYCVerified: userStats.isKYCVerified,
        humanProbability
      }
    };
  }

  /**
   * Calculate XP multiplier based on user level
   */
  private calculateXPMultiplier(level: number, totalXP: number): number {
    // Level-based multiplier (1.0x to 5.0x)
    const levelMultiplier = Math.min(5.0, 1.0 + (level / 20));
    
    // Additional bonus for high XP users
    const xpBonus = Math.min(1.5, 1.0 + (totalXP / 100000));
    
    return levelMultiplier * xpBonus;
  }

  /**
   * Calculate RP multiplier based on referral tier and network size
   */
  private calculateRPMultiplier(tier: string, networkSize: number): number {
    const tierMultipliers = {
      'Explorer': 1.0,
      'Connector': 1.2,
      'Influencer': 1.5,
      'Leader': 2.0,
      'Ambassador': 3.0
    };

    const baseMultiplier = tierMultipliers[tier] || 1.0;
    const networkBonus = Math.min(1.5, 1.0 + (networkSize / 100));
    
    return baseMultiplier * networkBonus;
  }

  /**
   * Calculate accumulated rewards for active session
   */
  private async calculateAccumulatedRewards(session: MiningSession, currentTime: Date) {
    const hoursActive = this.calculateHoursActive(session.startedAt, currentTime);
    const maxHours = 24; // Maximum mining hours per session
    const effectiveHours = Math.min(hoursActive, maxHours);

    // Base calculation
    let baseAmount = session.baseRate * effectiveHours;

    // Apply active boosters
    const activeBoosters = await this.miningService.getActiveBoosters(session.id, currentTime);
    let boosterMultiplier = 1.0;

    for (const booster of activeBoosters) {
      boosterMultiplier *= booster.multiplier;
    }

    const totalAmount = baseAmount * boosterMultiplier;

    return {
      baseAmount,
      boostedAmount: totalAmount,
      hoursActive: effectiveHours,
      boosterMultiplier,
      activeBoosters: activeBoosters.length
    };
  }

  /**
   * Calculate final rewards including all bonuses
   */
  private async calculateFinalRewards(session: MiningSession, claimTime: Date) {
    const accumulated = await this.calculateAccumulatedRewards(session, claimTime);
    
    // Additional bonuses at claim time
    const xpBonus = accumulated.baseAmount * 0.1; // 10% XP bonus
    const rpBonus = accumulated.baseAmount * (session.rpMultiplier - 1.0) * 0.2; // RP network bonus
    const qualityBonus = accumulated.baseAmount * (session.qualityScore - 1.0); // Quality bonus
    
    // Special bonuses (streaks, achievements, etc.)
    const specialBonuses = await this.calculateSpecialBonuses(session.userId, session);

    const totalAmount = accumulated.boostedAmount + xpBonus + rpBonus + qualityBonus + specialBonuses;

    return {
      baseAmount: accumulated.baseAmount,
      boostedAmount: accumulated.boostedAmount,
      xpBonus,
      rpBonus,
      qualityBonus,
      specialBonuses,
      totalAmount: Math.max(0, totalAmount) // Ensure non-negative
    };
  }

  /**
   * Calculate special bonuses (streaks, achievements, first-time, etc.)
   */
  private async calculateSpecialBonuses(userId: string, session: MiningSession): Promise<number> {
    let bonuses = 0;

    // Daily streak bonus
    const streakDays = await this.miningService.getUserStreak(userId);
    bonuses += Math.min(streakDays * 0.01, 0.5) * session.baseRate; // Max 50% bonus

    // First-time miner bonus
    const userStats = await this.miningService.getUserMiningStats(userId);
    if (userStats.totalSessions === 0) {
      bonuses += session.baseRate * 0.5; // 50% first-time bonus
    }

    // Achievement bonuses
    const achievements = await this.miningService.getUnclaimedAchievements(userId);
    for (const achievement of achievements) {
      bonuses += achievement.bonusAmount;
    }

    return bonuses;
  }

  /**
   * Calculate hours active between two dates
   */
  private calculateHoursActive(startTime: Date, endTime: Date): number {
    const diffMs = endTime.getTime() - startTime.getTime();
    return Math.max(0, diffMs / (1000 * 60 * 60)); // Convert to hours
  }
}
