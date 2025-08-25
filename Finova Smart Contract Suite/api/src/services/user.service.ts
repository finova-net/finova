import { Injectable, Logger, BadRequestException, NotFoundException, UnauthorizedException } from '@nestjs/common';
import { InjectRepository } from '@nestjs/typeorm';
import { Repository, QueryRunner, DataSource } from 'typeorm';
import { User } from '../models/User.model';
import { Mining } from '../models/Mining.model';
import { XP } from '../models/XP.model';
import { Referral } from '../models/Referral.model';
import { RedisService } from './redis.service';
import { BlockchainService } from './blockchain.service';
import { AuthService } from './auth.service';
import { MiningService } from './mining.service';
import { XpService } from './xp.service';
import { ReferralService } from './referral.service';
import { AiQualityService } from './ai-quality.service';
import { AntiBotService } from './anti-bot.service';
import { NotificationService } from './notification.service';
import { AnalyticsService } from './analytics.service';
import { CreateUserDto, UpdateUserDto, UserActivityDto, UserStatsDto } from '../types/user.types';
import { ApiResponse } from '../types/api.types';
import * as crypto from 'crypto';
import * as bcrypt from 'bcrypt';

@Injectable()
export class UserService {
  private readonly logger = new Logger(UserService.name);

  constructor(
    @InjectRepository(User)
    private readonly userRepository: Repository<User>,
    @InjectRepository(Mining)
    private readonly miningRepository: Repository<Mining>,
    @InjectRepository(XP)
    private readonly xpRepository: Repository<XP>,
    @InjectRepository(Referral)
    private readonly referralRepository: Repository<Referral>,
    private readonly dataSource: DataSource,
    private readonly redisService: RedisService,
    private readonly blockchainService: BlockchainService,
    private readonly authService: AuthService,
    private readonly miningService: MiningService,
    private readonly xpService: XpService,
    private readonly referralService: ReferralService,
    private readonly aiQualityService: AiQualityService,
    private readonly antiBotService: AntiBotService,
    private readonly notificationService: NotificationService,
    private readonly analyticsService: AnalyticsService,
  ) {}

  /**
   * Create new user with integrated mining, XP, and referral initialization
   * Based on Finova's triple reward system architecture
   */
  async createUser(createUserDto: CreateUserDto): Promise<ApiResponse<User>> {
    const queryRunner = this.dataSource.createQueryRunner();
    await queryRunner.connect();
    await queryRunner.startTransaction();

    try {
      this.logger.log(`Creating new user: ${createUserDto.email}`);

      // Validate referral code if provided
      let referrer: User | null = null;
      if (createUserDto.referralCode) {
        referrer = await this.validateReferralCode(createUserDto.referralCode);
        if (!referrer) {
          throw new BadRequestException('Invalid referral code');
        }
      }

      // Generate unique user ID and wallet
      const userId = crypto.randomUUID();
      const { publicKey, privateKey } = await this.blockchainService.generateWallet();

      // Hash password
      const hashedPassword = await bcrypt.hash(createUserDto.password, 12);

      // Create user entity
      const user = queryRunner.manager.create(User, {
        id: userId,
        email: createUserDto.email,
        username: createUserDto.username,
        password: hashedPassword,
        firstName: createUserDto.firstName,
        lastName: createUserDto.lastName,
        phoneNumber: createUserDto.phoneNumber,
        country: createUserDto.country || 'ID',
        walletAddress: publicKey,
        referralCode: this.generateReferralCode(),
        referredBy: referrer?.id,
        isActive: true,
        isKycVerified: false,
        isBotVerified: false,
        createdAt: new Date(),
        updatedAt: new Date(),
        // Initialize user metrics
        totalFinEarned: 0,
        totalXpEarned: 0,
        totalRpEarned: 0,
        currentLevel: 1,
        currentRpTier: 'Explorer',
        miningRate: this.calculateInitialMiningRate(),
        qualityScore: 1.0,
        humanProbability: 0.5,
        lastActiveAt: new Date(),
        streakDays: 0,
        longestStreak: 0,
      });

      const savedUser = await queryRunner.manager.save(User, user);

      // Initialize mining account
      await this.miningService.initializeMiningAccount(savedUser.id, queryRunner);

      // Initialize XP system
      await this.xpService.initializeXpAccount(savedUser.id, queryRunner);

      // Process referral if applicable
      if (referrer) {
        await this.referralService.processNewReferral(referrer.id, savedUser.id, queryRunner);
        
        // Award referral bonuses
        await this.awardReferralSignupBonus(referrer, savedUser, queryRunner);
      }

      // Store wallet private key securely (encrypted)
      await this.storeUserWallet(savedUser.id, privateKey);

      // Initialize user cache
      await this.cacheUserData(savedUser);

      // Track analytics
      await this.analyticsService.trackUserRegistration(savedUser, referrer?.id);

      await queryRunner.commitTransaction();

      this.logger.log(`User created successfully: ${savedUser.id}`);

      // Send welcome notification
      await this.notificationService.sendWelcomeNotification(savedUser);

      return {
        success: true,
        data: this.sanitizeUserData(savedUser),
        message: 'User created successfully',
      };
    } catch (error) {
      await queryRunner.rollbackTransaction();
      this.logger.error(`Failed to create user: ${error.message}`, error.stack);
      throw error;
    } finally {
      await queryRunner.release();
    }
  }

  /**
   * Get user by ID with comprehensive data loading
   * Includes mining stats, XP data, referral network, and current status
   */
  async getUserById(userId: string): Promise<ApiResponse<User>> {
    try {
      // Check cache first
      const cachedUser = await this.redisService.get(`user:${userId}`);
      if (cachedUser) {
        return {
          success: true,
          data: JSON.parse(cachedUser),
          message: 'User retrieved from cache',
        };
      }

      const user = await this.userRepository
        .createQueryBuilder('user')
        .leftJoinAndSelect('user.miningAccount', 'mining')
        .leftJoinAndSelect('user.xpAccount', 'xp')
        .leftJoinAndSelect('user.referralAccount', 'referral')
        .leftJoinAndSelect('user.nfts', 'nfts')
        .leftJoinAndSelect('user.guild', 'guild')
        .where('user.id = :userId', { userId })
        .getOne();

      if (!user) {
        throw new NotFoundException('User not found');
      }

      // Calculate real-time metrics
      const enhancedUser = await this.enhanceUserData(user);

      // Update cache
      await this.cacheUserData(enhancedUser);

      return {
        success: true,
        data: this.sanitizeUserData(enhancedUser),
        message: 'User retrieved successfully',
      };
    } catch (error) {
      this.logger.error(`Failed to get user ${userId}: ${error.message}`);
      throw error;
    }
  }

  /**
   * Update user profile with validation and security checks
   */
  async updateUser(userId: string, updateUserDto: UpdateUserDto): Promise<ApiResponse<User>> {
    const queryRunner = this.dataSource.createQueryRunner();
    await queryRunner.connect();
    await queryRunner.startTransaction();

    try {
      const user = await this.getUserEntityById(userId);

      // Security: Validate sensitive field changes
      if (updateUserDto.email && updateUserDto.email !== user.email) {
        await this.validateEmailChange(user, updateUserDto.email);
      }

      // Update allowed fields
      Object.assign(user, {
        ...updateUserDto,
        updatedAt: new Date(),
      });

      const updatedUser = await queryRunner.manager.save(User, user);

      // Update cache
      await this.cacheUserData(updatedUser);

      // Track analytics
      await this.analyticsService.trackUserUpdate(userId, updateUserDto);

      await queryRunner.commitTransaction();

      return {
        success: true,
        data: this.sanitizeUserData(updatedUser),
        message: 'User updated successfully',
      };
    } catch (error) {
      await queryRunner.rollbackTransaction();
      this.logger.error(`Failed to update user ${userId}: ${error.message}`);
      throw error;
    } finally {
      await queryRunner.release();
    }
  }

  /**
   * Process user activity with integrated XP, RP, and mining calculations
   * Core function for Finova's triple reward system
   */
  async processUserActivity(userId: string, activityDto: UserActivityDto): Promise<ApiResponse<any>> {
    const queryRunner = this.dataSource.createQueryRunner();
    await queryRunner.connect();
    await queryRunner.startTransaction();

    try {
      const user = await this.getUserEntityById(userId);

      // Anti-bot validation
      const botScore = await this.antiBotService.analyzeUserActivity(user, activityDto);
      if (botScore > 0.8) {
        throw new UnauthorizedException('Suspicious activity detected');
      }

      // AI quality assessment
      const qualityScore = await this.aiQualityService.analyzeContent(activityDto.content);

      // Calculate rewards based on Finova's integrated formula
      const rewards = await this.calculateIntegratedRewards(user, activityDto, qualityScore);

      // Process XP rewards
      if (rewards.xpGained > 0) {
        await this.xpService.awardXp(userId, rewards.xpGained, activityDto.type, queryRunner);
      }

      // Process mining rewards
      if (rewards.finMined > 0) {
        await this.miningService.processMiningReward(userId, rewards.finMined, queryRunner);
      }

      // Process referral rewards
      if (rewards.rpGained > 0) {
        await this.referralService.awardReferralPoints(userId, rewards.rpGained, queryRunner);
      }

      // Update user activity metrics
      await this.updateUserActivityMetrics(user, activityDto, rewards, queryRunner);

      // Update user level and tier if needed
      await this.checkAndUpdateUserProgression(user, queryRunner);

      await queryRunner.commitTransaction();

      // Send real-time notifications
      if (rewards.totalValue > 0) {
        await this.notificationService.sendRewardNotification(user, rewards);
      }

      // Track analytics
      await this.analyticsService.trackUserActivity(userId, activityDto, rewards);

      return {
        success: true,
        data: {
          rewards,
          newLevel: user.currentLevel,
          newRpTier: user.currentRpTier,
          qualityScore,
        },
        message: 'Activity processed successfully',
      };
    } catch (error) {
      await queryRunner.rollbackTransaction();
      this.logger.error(`Failed to process activity for user ${userId}: ${error.message}`);
      throw error;
    } finally {
      await queryRunner.release();
    }
  }

  /**
   * Get comprehensive user statistics
   * Including mining, XP, referral data, and network effects
   */
  async getUserStats(userId: string): Promise<ApiResponse<UserStatsDto>> {
    try {
      const user = await this.getUserEntityById(userId);

      // Calculate comprehensive stats
      const stats: UserStatsDto = {
        // Basic user info
        userId: user.id,
        username: user.username,
        level: user.currentLevel,
        rpTier: user.currentRpTier,
        qualityScore: user.qualityScore,
        
        // Mining statistics
        mining: {
          totalEarned: user.totalFinEarned,
          currentRate: await this.miningService.getCurrentMiningRate(userId),
          dailyLimit: await this.miningService.getDailyMiningLimit(user),
          todayEarned: await this.miningService.getTodayEarnings(userId),
          efficiency: await this.calculateMiningEfficiency(user),
          nextRateUpdate: await this.miningService.getNextRateUpdate(),
        },

        // XP statistics
        xp: {
          totalEarned: user.totalXpEarned,
          currentLevel: user.currentLevel,
          xpToNextLevel: await this.xpService.getXpToNextLevel(userId),
          todayEarned: await this.xpService.getTodayXp(userId),
          weeklyAverage: await this.xpService.getWeeklyAverageXp(userId),
          levelMultiplier: this.xpService.getLevelMultiplier(user.currentLevel),
        },

        // Referral statistics
        referral: {
          totalRpEarned: user.totalRpEarned,
          currentTier: user.currentRpTier,
          directReferrals: await this.referralService.getDirectReferralCount(userId),
          activeReferrals: await this.referralService.getActiveReferralCount(userId),
          networkSize: await this.referralService.getTotalNetworkSize(userId),
          networkQuality: await this.referralService.getNetworkQualityScore(userId),
          tierMultiplier: this.referralService.getRpTierMultiplier(user.currentRpTier),
        },

        // Activity statistics
        activity: {
          streakDays: user.streakDays,
          longestStreak: user.longestStreak,
          lastActiveAt: user.lastActiveAt,
          totalActiveDays: await this.calculateTotalActiveDays(userId),
          averageDailyActivity: await this.calculateAverageDailyActivity(userId),
        },

        // Calculated values
        totalValue: user.totalFinEarned + (user.totalXpEarned * 0.001) + (user.totalRpEarned * 0.01),
        projectedDailyEarnings: await this.calculateProjectedDailyEarnings(user),
        networkEffect: await this.calculateNetworkEffect(user),
      };

      // Cache stats for 5 minutes
      await this.redisService.setex(`user_stats:${userId}`, 300, JSON.stringify(stats));

      return {
        success: true,
        data: stats,
        message: 'User statistics retrieved successfully',
      };
    } catch (error) {
      this.logger.error(`Failed to get user stats ${userId}: ${error.message}`);
      throw error;
    }
  }

  /**
   * Process KYC verification with enhanced security
   */
  async processKycVerification(userId: string, kycData: any): Promise<ApiResponse<any>> {
    const queryRunner = this.dataSource.createQueryRunner();
    await queryRunner.connect();
    await queryRunner.startTransaction();

    try {
      const user = await this.getUserEntityById(userId);

      if (user.isKycVerified) {
        throw new BadRequestException('User is already KYC verified');
      }

      // Process KYC with external service
      const kycResult = await this.processKycWithProvider(kycData);

      if (kycResult.approved) {
        // Update user KYC status
        user.isKycVerified = true;
        user.kycVerifiedAt = new Date();
        user.kycProvider = kycResult.provider;
        user.updatedAt = new Date();

        await queryRunner.manager.save(User, user);

        // Award KYC bonus (20% mining rate increase)
        await this.miningService.applyKycBonus(userId, queryRunner);

        // Award referral bonus if user was referred
        if (user.referredBy) {
          await this.referralService.processKycReferralBonus(user.referredBy, userId, queryRunner);
        }

        await queryRunner.commitTransaction();

        // Send notification
        await this.notificationService.sendKycApprovedNotification(user);

        // Track analytics
        await this.analyticsService.trackKycVerification(userId, true);

        return {
          success: true,
          data: { verified: true, bonusApplied: true },
          message: 'KYC verification completed successfully',
        };
      } else {
        await queryRunner.rollbackTransaction();
        
        // Track failed KYC
        await this.analyticsService.trackKycVerification(userId, false);

        return {
          success: false,
          data: { verified: false, reason: kycResult.reason },
          message: 'KYC verification failed',
        };
      }
    } catch (error) {
      await queryRunner.rollbackTransaction();
      this.logger.error(`KYC verification failed for user ${userId}: ${error.message}`);
      throw error;
    } finally {
      await queryRunner.release();
    }
  }

  /**
   * Calculate integrated rewards based on Finova's triple system
   * Formula: Total_Reward = Base_Mining × XP_Multiplier × RP_Multiplier × Quality_Score
   */
  private async calculateIntegratedRewards(user: User, activity: UserActivityDto, qualityScore: number) {
    // Get base values
    const baseMiningRate = await this.miningService.getBaseMiningRate();
    const baseXp = this.xpService.getBaseXp(activity.type);
    const baseRp = this.referralService.getBaseRp(activity.type);

    // Calculate multipliers
    const xpLevelMultiplier = this.xpService.getLevelMultiplier(user.currentLevel);
    const rpTierMultiplier = this.referralService.getRpTierMultiplier(user.currentRpTier);
    const networkRegressionFactor = await this.calculateNetworkRegression(user);
    const platformMultiplier = this.getPlatformMultiplier(activity.platform);
    const streakBonus = this.calculateStreakBonus(user.streakDays);

    // Apply Finova's integrated formula
    const finMined = baseMiningRate * 
      xpLevelMultiplier * 
      rpTierMultiplier * 
      qualityScore * 
      networkRegressionFactor * 
      platformMultiplier;

    const xpGained = baseXp * 
      platformMultiplier * 
      qualityScore * 
      streakBonus * 
      Math.exp(-0.01 * user.currentLevel); // Level progression dampening

    const rpGained = baseRp * 
      qualityScore * 
      this.calculateNetworkQualityBonus(user);

    return {
      finMined: Math.max(0, finMined),
      xpGained: Math.max(0, Math.floor(xpGained)),
      rpGained: Math.max(0, Math.floor(rpGained)),
      qualityScore,
      multipliers: {
        xpLevel: xpLevelMultiplier,
        rpTier: rpTierMultiplier,
        platform: platformMultiplier,
        streak: streakBonus,
        networkRegression: networkRegressionFactor,
      },
      totalValue: finMined + (xpGained * 0.001) + (rpGained * 0.01),
    };
  }

  /**
   * Calculate network regression factor (exponential decay for whales)
   * Based on Pi Network's fair distribution model
   */
  private async calculateNetworkRegression(user: User): Promise<number> {
    const totalHoldings = user.totalFinEarned;
    const networkSize = await this.getTotalNetworkSize();
    
    // Exponential regression: e^(-0.001 * holdings)
    const holdingsRegression = Math.exp(-0.001 * totalHoldings);
    
    // Network effect: decreasing rewards as network grows
    const networkRegression = Math.max(0.1, 2.0 - (networkSize / 1000000));
    
    return holdingsRegression * networkRegression;
  }

  /**
   * Calculate initial mining rate for new users
   */
  private calculateInitialMiningRate(): number {
    // Phase-based mining rate (Pi Network inspired)
    const baseRate = 0.1; // Phase 1 rate
    return baseRate;
  }

  /**
   * Generate unique referral code
   */
  private generateReferralCode(): string {
    return crypto.randomBytes(4).toString('hex').toUpperCase();
  }

  /**
   * Validate referral code
   */
  private async validateReferralCode(code: string): Promise<User | null> {
    return this.userRepository.findOne({
      where: { referralCode: code, isActive: true },
    });
  }

  /**
   * Award referral signup bonus
   */
  private async awardReferralSignupBonus(referrer: User, newUser: User, queryRunner: QueryRunner) {
    // Award RP to referrer
    await this.referralService.awardReferralPoints(referrer.id, 100, queryRunner);
    
    // Award welcome bonus to new user
    await this.xpService.awardXp(newUser.id, 50, 'SIGNUP', queryRunner);
  }

  /**
   * Store user wallet securely
   */
  private async storeUserWallet(userId: string, privateKey: string) {
    // Encrypt private key before storage
    const encrypted = await this.authService.encrypt(privateKey);
    await this.redisService.set(`wallet:${userId}`, encrypted);
  }

  /**
   * Cache user data
   */
  private async cacheUserData(user: User) {
    const sanitizedUser = this.sanitizeUserData(user);
    await this.redisService.setex(`user:${user.id}`, 3600, JSON.stringify(sanitizedUser));
  }

  /**
   * Sanitize user data for API response
   */
  private sanitizeUserData(user: User): any {
    const { password, ...sanitized } = user;
    return sanitized;
  }

  /**
   * Get user entity by ID (internal method)
   */
  private async getUserEntityById(userId: string): Promise<User> {
    const user = await this.userRepository.findOne({
      where: { id: userId },
      relations: ['miningAccount', 'xpAccount', 'referralAccount'],
    });

    if (!user) {
      throw new NotFoundException('User not found');
    }

    return user;
  }

  /**
   * Enhance user data with calculated metrics
   */
  private async enhanceUserData(user: User): Promise<User> {
    // Add real-time calculated fields
    user.currentMiningRate = await this.miningService.getCurrentMiningRate(user.id);
    user.projectedDailyEarnings = await this.calculateProjectedDailyEarnings(user);
    user.networkEffect = await this.calculateNetworkEffect(user);
    
    return user;
  }

  /**
   * Additional helper methods for calculations
   */
  private async getTotalNetworkSize(): Promise<number> {
    return this.userRepository.count({ where: { isActive: true } });
  }

  private getPlatformMultiplier(platform: string): number {
    const multipliers = {
      'tiktok': 1.3,
      'instagram': 1.2,
      'youtube': 1.4,
      'facebook': 1.1,
      'twitter': 1.2,
      'x': 1.2,
    };
    return multipliers[platform?.toLowerCase()] || 1.0;
  }

  private calculateStreakBonus(streakDays: number): number {
    return Math.min(3.0, 1.0 + (streakDays * 0.1));
  }

  private calculateNetworkQualityBonus(user: User): number {
    // Placeholder - would involve complex referral network analysis
    return 1.0;
  }

  private async calculateProjectedDailyEarnings(user: User): Promise<number> {
    const currentRate = await this.miningService.getCurrentMiningRate(user.id);
    return currentRate * 24; // 24 hours
  }

  private async calculateNetworkEffect(user: User): Promise<number> {
    // Calculate compound effect of referral network
    const networkSize = await this.referralService.getTotalNetworkSize(user.id);
    const networkQuality = await this.referralService.getNetworkQualityScore(user.id);
    return networkSize * networkQuality * 0.01;
  }

  private async calculateTotalActiveDays(userId: string): Promise<number> {
    // Query for distinct active dates
    const result = await this.userRepository.query(
      'SELECT COUNT(DISTINCT DATE(created_at)) as active_days FROM user_activities WHERE user_id = $1',
      [userId]
    );
    return result[0]?.active_days || 0;
  }

  private async calculateAverageDailyActivity(userId: string): Promise<number> {
    // Calculate average activities per day
    const totalDays = await this.calculateTotalActiveDays(userId);
    if (totalDays === 0) return 0;
    
    const totalActivities = await this.userRepository.query(
      'SELECT COUNT(*) as total FROM user_activities WHERE user_id = $1',
      [userId]
    );
    
    return totalActivities[0]?.total / totalDays || 0;
  }

  private async calculateMiningEfficiency(user: User): Promise<number> {
    // Calculate mining efficiency based on various factors
    const baseEfficiency = 1.0;
    const kycBonus = user.isKycVerified ? 0.2 : 0;
    const levelBonus = user.currentLevel * 0.01;
    const qualityBonus = user.qualityScore - 1.0;
    
    return baseEfficiency + kycBonus + levelBonus + qualityBonus;
  }

  private async processKycWithProvider(kycData: any): Promise<any> {
    // Placeholder for KYC provider integration
    // In real implementation, integrate with providers like Jumio, Onfido, etc.
    return {
      approved: true,
      provider: 'mock_provider',
      confidence: 0.95,
    };
  }

  private async validateEmailChange(user: User, newEmail: string): Promise<void> {
    const existingUser = await this.userRepository.findOne({
      where: { email: newEmail },
    });
    
    if (existingUser && existingUser.id !== user.id) {
      throw new BadRequestException('Email already in use');
    }
  }

  private async updateUserActivityMetrics(
    user: User,
    activity: UserActivityDto,
    rewards: any,
    queryRunner: QueryRunner,
  ): Promise<void> {
    // Update user activity metrics
    user.totalFinEarned += rewards.finMined;
    user.totalXpEarned += rewards.xpGained;
    user.totalRpEarned += rewards.rpGained;
    user.lastActiveAt = new Date();
    user.updatedAt = new Date();

    // Update streak
    const lastActivity = user.lastActiveAt;
    const today = new Date();
    const daysDiff = Math.floor((today.getTime() - lastActivity.getTime()) / (1000 * 60 * 60 * 24));
    
    if (daysDiff <= 1) {
      user.streakDays += 1;
      user.longestStreak = Math.max(user.longestStreak, user.streakDays);
    } else {
      user.streakDays = 1;
    }

    await queryRunner.manager.save(User, user);
  }

  private async checkAndUpdateUserProgression(user: User, queryRunner: QueryRunner): Promise<void> {
    // Check XP level progression
    const newLevel = this.xpService.calculateLevel(user.totalXpEarned);
    if (newLevel > user.currentLevel) {
      user.currentLevel = newLevel;
      // Award level up bonus
      await this.xpService.awardLevelUpBonus(user.id, newLevel, queryRunner);
    }

    // Check RP tier progression
    const newRpTier = this.referralService.calculateRpTier(user.totalRpEarned);
    if (newRpTier !== user.currentRpTier) {
      user.currentRpTier = newRpTier;
      // Award tier up bonus
      await this.referralService.awardTierUpBonus(user.id, newRpTier, queryRunner);
    }

    await queryRunner.manager.save(User, user);
  }
}
