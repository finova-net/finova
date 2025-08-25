import { Injectable, Logger, BadRequestException, NotFoundException } from '@nestjs/common';
import { InjectRepository } from '@nestjs/typeorm';
import { Repository, DataSource, QueryRunner } from 'typeorm';
import { User } from '../models/User.model';
import { Referral } from '../models/Referral.model';
import { XP } from '../models/XP.model';
import { Mining } from '../models/Mining.model';
import { Transaction } from '../models/Transaction.model';
import { BlockchainService } from './blockchain.service';
import { NotificationService } from './notification.service';
import { AnalyticsService } from './analytics.service';
import { AntiBotService } from './anti-bot.service';
import { Cron, CronExpression } from '@nestjs/schedule';
import { Cache } from 'cache-manager';
import { CACHE_MANAGER, Inject } from '@nestjs/common';
import * as crypto from 'crypto';

interface ReferralStats {
  totalReferrals: number;
  activeReferrals: number;
  totalRP: number;
  currentTier: string;
  networkSize: number;
  qualityScore: number;
  lifetimeEarnings: number;
}

interface NetworkData {
  level1: User[];
  level2: User[];
  level3: User[];
  totalSize: number;
  activeUsers: number;
  qualityScore: number;
}

interface ReferralReward {
  rpGained: number;
  finBonus: number;
  xpBonus: number;
  tierUpgrade?: string;
  achievements?: string[];
}

@Injectable()
export class ReferralService {
  private readonly logger = new Logger(ReferralService.name);

  // RP Tier Configuration
  private readonly RP_TIERS = {
    EXPLORER: { min: 0, max: 999, name: 'Explorer', miningBonus: 0, referralBonus: 0.10 },
    CONNECTOR: { min: 1000, max: 4999, name: 'Connector', miningBonus: 0.20, referralBonus: 0.15 },
    INFLUENCER: { min: 5000, max: 14999, name: 'Influencer', miningBonus: 0.50, referralBonus: 0.20 },
    LEADER: { min: 15000, max: 49999, name: 'Leader', miningBonus: 1.00, referralBonus: 0.25 },
    AMBASSADOR: { min: 50000, max: Infinity, name: 'Ambassador', miningBonus: 2.00, referralBonus: 0.30 }
  };

  // Network Level Bonuses
  private readonly NETWORK_BONUSES = {
    L1: 0.30, // 30% from direct referrals
    L2: 0.15, // 15% from level 2 network
    L3: 0.08  // 8% from level 3 network
  };

  constructor(
    @InjectRepository(User)
    private userRepository: Repository<User>,
    @InjectRepository(Referral)
    private referralRepository: Repository<Referral>,
    @InjectRepository(XP)
    private xpRepository: Repository<XP>,
    @InjectRepository(Mining)
    private miningRepository: Repository<Mining>,
    @InjectRepository(Transaction)
    private transactionRepository: Repository<Transaction>,
    private dataSource: DataSource,
    private blockchainService: BlockchainService,
    private notificationService: NotificationService,
    private analyticsService: AnalyticsService,
    private antiBotService: AntiBotService,
    @Inject(CACHE_MANAGER) private cacheManager: Cache,
  ) {}

  /**
   * Generate unique referral code
   */
  async generateReferralCode(userId: string): Promise<string> {
    const user = await this.userRepository.findOne({ where: { id: userId } });
    if (!user) throw new NotFoundException('User not found');

    // Generate unique 8-character code
    let code: string;
    let isUnique = false;
    let attempts = 0;

    do {
      code = this.createReferralCode(user.username || user.id);
      const existing = await this.referralRepository.findOne({ where: { referralCode: code } });
      isUnique = !existing;
      attempts++;
    } while (!isUnique && attempts < 10);

    if (!isUnique) {
      throw new BadRequestException('Unable to generate unique referral code');
    }

    // Update user with referral code
    await this.userRepository.update(userId, { referralCode: code });
    
    this.logger.log(`Generated referral code ${code} for user ${userId}`);
    return code;
  }

  /**
   * Process referral registration
   */
  async processReferral(newUserId: string, referralCode: string): Promise<ReferralReward> {
    const queryRunner = this.dataSource.createQueryRunner();
    await queryRunner.connect();
    await queryRunner.startTransaction();

    try {
      // Find referrer
      const referrer = await queryRunner.manager.findOne(User, { 
        where: { referralCode },
        relations: ['referrals']
      });

      if (!referrer) {
        throw new BadRequestException('Invalid referral code');
      }

      // Validate new user
      const newUser = await queryRunner.manager.findOne(User, { where: { id: newUserId } });
      if (!newUser) {
        throw new NotFoundException('New user not found');
      }

      // Anti-bot validation
      const humanScore = await this.antiBotService.calculateHumanProbability({
        userId: newUserId,
        referrerId: referrer.id,
        registrationData: newUser
      });

      if (humanScore < 0.7) {
        this.logger.warn(`Suspicious referral detected: ${newUserId} -> ${referrer.id} (Human Score: ${humanScore})`);
        throw new BadRequestException('Referral failed security validation');
      }

      // Create referral record
      const referral = queryRunner.manager.create(Referral, {
        referrerId: referrer.id,
        referredUserId: newUserId,
        referralCode,
        status: 'registered',
        createdAt: new Date(),
        humanScore
      });

      await queryRunner.manager.save(referral);

      // Calculate initial RP reward
      const rpReward = await this.calculateRegistrationRP(referrer.id, queryRunner);
      
      // Update referrer's RP
      await this.updateUserRP(referrer.id, rpReward, 'referral_registration', queryRunner);

      // Calculate tier upgrade if applicable
      const newStats = await this.calculateReferralStats(referrer.id, queryRunner);
      const oldTier = this.getRPTier(referrer.referralPoints || 0);
      const newTier = this.getRPTier(newStats.totalRP);

      const reward: ReferralReward = {
        rpGained: rpReward,
        finBonus: 0,
        xpBonus: 100, // Welcome bonus XP
        tierUpgrade: oldTier.name !== newTier.name ? newTier.name : undefined
      };

      // Award welcome bonuses
      await this.awardWelcomeBonus(referrer.id, newUserId, queryRunner);

      await queryRunner.commitTransaction();

      // Send notifications
      await this.notificationService.sendReferralSuccess(referrer.id, newUser.username, rpReward);
      
      // Track analytics
      await this.analyticsService.trackReferralEvent('registration', {
        referrerId: referrer.id,
        referredId: newUserId,
        rpReward,
        humanScore
      });

      this.logger.log(`Processed referral: ${newUserId} -> ${referrer.id}, RP: ${rpReward}`);
      return reward;

    } catch (error) {
      await queryRunner.rollbackTransaction();
      throw error;
    } finally {
      await queryRunner.release();
    }
  }

  /**
   * Calculate referral rewards from network activity
   */
  async calculateNetworkRewards(userId: string, activity: string, baseReward: number): Promise<number> {
    const cacheKey = `network_rewards:${userId}:${Date.now()}`;
    const cached = await this.cacheManager.get<number>(cacheKey);
    if (cached !== undefined) return cached;

    // Get user's referral network
    const network = await this.getReferralNetwork(userId);
    
    let totalBonus = 0;

    // Level 1 referrals (direct)
    for (const referrer of network.level1) {
      if (await this.isUserActive(referrer.id)) {
        const bonus = baseReward * this.NETWORK_BONUSES.L1;
        await this.awardNetworkBonus(referrer.id, userId, bonus, 'L1', activity);
        totalBonus += bonus;
      }
    }

    // Level 2 referrals
    for (const referrer of network.level2) {
      if (await this.isUserActive(referrer.id)) {
        const bonus = baseReward * this.NETWORK_BONUSES.L2;
        await this.awardNetworkBonus(referrer.id, userId, bonus, 'L2', activity);
        totalBonus += bonus;
      }
    }

    // Level 3 referrals
    for (const referrer of network.level3) {
      if (await this.isUserActive(referrer.id)) {
        const bonus = baseReward * this.NETWORK_BONUSES.L3;
        await this.awardNetworkBonus(referrer.id, userId, bonus, 'L3', activity);
        totalBonus += bonus;
      }
    }

    await this.cacheManager.set(cacheKey, totalBonus, 300); // 5min cache
    return totalBonus;
  }

  /**
   * Get user's referral statistics
   */
  async getReferralStats(userId: string): Promise<ReferralStats> {
    const cacheKey = `referral_stats:${userId}`;
    const cached = await this.cacheManager.get<ReferralStats>(cacheKey);
    if (cached) return cached;

    const stats = await this.calculateReferralStats(userId);
    await this.cacheManager.set(cacheKey, stats, 1800); // 30min cache
    return stats;
  }

  /**
   * Get referral network data
   */
  async getReferralNetwork(userId: string): Promise<NetworkData> {
    const cacheKey = `referral_network:${userId}`;
    const cached = await this.cacheManager.get<NetworkData>(cacheKey);
    if (cached) return cached;

    // Level 1 - Direct referrals
    const level1 = await this.userRepository
      .createQueryBuilder('user')
      .innerJoin('referrals', 'ref', 'ref.referredUserId = user.id')
      .where('ref.referrerId = :userId', { userId })
      .getMany();

    // Level 2 - Referrals of referrals
    const level2 = await this.userRepository
      .createQueryBuilder('user')
      .innerJoin('referrals', 'ref2', 'ref2.referredUserId = user.id')
      .innerJoin('referrals', 'ref1', 'ref1.referredUserId = ref2.referrerId')
      .where('ref1.referrerId = :userId', { userId })
      .getMany();

    // Level 3 - Third level referrals
    const level3 = await this.userRepository
      .createQueryBuilder('user')
      .innerJoin('referrals', 'ref3', 'ref3.referredUserId = user.id')
      .innerJoin('referrals', 'ref2', 'ref2.referredUserId = ref3.referrerId')
      .innerJoin('referrals', 'ref1', 'ref1.referredUserId = ref2.referrerId')
      .where('ref1.referrerId = :userId', { userId })
      .getMany();

    const totalSize = level1.length + level2.length + level3.length;
    const activeUsers = await this.countActiveUsers([...level1, ...level2, ...level3]);
    const qualityScore = totalSize > 0 ? activeUsers / totalSize : 0;

    const networkData: NetworkData = {
      level1,
      level2,
      level3,
      totalSize,
      activeUsers,
      qualityScore
    };

    await this.cacheManager.set(cacheKey, networkData, 900); // 15min cache
    return networkData;
  }

  /**
   * Calculate RP tier and benefits
   */
  getRPTier(totalRP: number) {
    for (const [key, tier] of Object.entries(this.RP_TIERS)) {
      if (totalRP >= tier.min && totalRP <= tier.max) {
        return { key, ...tier };
      }
    }
    return { key: 'EXPLORER', ...this.RP_TIERS.EXPLORER };
  }

  /**
   * Apply referral multipliers to mining/XP
   */
  async applyReferralMultipliers(userId: string, baseValue: number, type: 'mining' | 'xp'): Promise<number> {
    const stats = await this.getReferralStats(userId);
    const tier = this.getRPTier(stats.totalRP);
    
    let multiplier = 1.0;
    
    if (type === 'mining') {
      multiplier += tier.miningBonus;
    } else if (type === 'xp') {
      multiplier += (stats.qualityScore * 0.5); // Quality bonus for XP
    }

    // Network effect multiplier
    const networkMultiplier = 1 + (stats.activeReferrals * 0.02); // 2% per active referral
    multiplier *= Math.min(networkMultiplier, 3.0); // Cap at 3x

    // Apply regression for large networks
    const regressionFactor = Math.exp(-0.0001 * stats.networkSize * stats.qualityScore);
    multiplier *= Math.max(regressionFactor, 0.1); // Minimum 0.1x

    return baseValue * multiplier;
  }

  /**
   * Process KYC completion bonus
   */
  async processKYCBonus(userId: string): Promise<void> {
    const queryRunner = this.dataSource.createQueryRunner();
    await queryRunner.connect();
    await queryRunner.startTransaction();

    try {
      // Find who referred this user
      const referral = await queryRunner.manager.findOne(Referral, {
        where: { referredUserId: userId },
        relations: ['referrer']
      });

      if (referral) {
        const kycBonus = 100; // 100 RP for KYC completion
        await this.updateUserRP(referral.referrerId, kycBonus, 'referral_kyc', queryRunner);
        
        // Update referral status
        await queryRunner.manager.update(Referral, referral.id, { 
          status: 'kyc_verified',
          kycCompletedAt: new Date()
        });

        await queryRunner.commitTransaction();

        this.logger.log(`KYC bonus awarded: ${kycBonus} RP to user ${referral.referrerId}`);
      } else {
        await queryRunner.rollbackTransaction();
      }
    } catch (error) {
      await queryRunner.rollbackTransaction();
      throw error;
    } finally {
      await queryRunner.release();
    }
  }

  /**
   * Daily RP decay for inactive users
   */
  @Cron(CronExpression.EVERY_DAY_AT_MIDNIGHT)
  async processRPDecay(): Promise<void> {
    this.logger.log('Processing daily RP decay...');
    
    const inactiveThreshold = new Date();
    inactiveThreshold.setDate(inactiveThreshold.getDate() - 30); // 30 days inactive

    const inactiveUsers = await this.userRepository
      .createQueryBuilder('user')
      .where('user.lastActivity < :threshold', { threshold: inactiveThreshold })
      .andWhere('user.referralPoints > 0')
      .getMany();

    for (const user of inactiveUsers) {
      const decayRate = 0.01; // 1% daily decay
      const newRP = Math.floor(user.referralPoints * (1 - decayRate));
      
      await this.userRepository.update(user.id, { 
        referralPoints: newRP,
        lastRPDecay: new Date()
      });

      this.logger.debug(`RP decay: User ${user.id} ${user.referralPoints} -> ${newRP}`);
    }

    this.logger.log(`Processed RP decay for ${inactiveUsers.length} users`);
  }

  /**
   * Weekly network quality assessment
   */
  @Cron(CronExpression.EVERY_WEEK)
  async assessNetworkQuality(): Promise<void> {
    this.logger.log('Assessing network quality...');

    const users = await this.userRepository.find({ where: { referralPoints: { $gt: 0 } } });

    for (const user of users) {
      const network = await this.getReferralNetwork(user.id);
      const qualityScore = await this.calculateNetworkQuality(user.id, network);
      
      await this.userRepository.update(user.id, { 
        networkQualityScore: qualityScore,
        lastQualityCheck: new Date()
      });

      // Apply quality-based adjustments
      if (qualityScore < 0.3) {
        await this.applyLowQualityPenalty(user.id);
      } else if (qualityScore > 0.8) {
        await this.applyHighQualityBonus(user.id);
      }
    }
  }

  // Private helper methods

  private createReferralCode(seed: string): string {
    const hash = crypto.createHash('md5').update(seed + Date.now()).digest('hex');
    return hash.substring(0, 8).toUpperCase();
  }

  private async calculateRegistrationRP(referrerId: string, queryRunner?: QueryRunner): Promise<number> {
    const manager = queryRunner ? queryRunner.manager : this.dataSource.manager;
    
    // Base registration reward
    let rpReward = 50;
    
    // Bonus based on referrer's current tier
    const referrer = await manager.findOne(User, { where: { id: referrerId } });
    if (referrer) {
      const tier = this.getRPTier(referrer.referralPoints || 0);
      rpReward += tier.min / 100; // Small tier-based bonus
    }

    return rpReward;
  }

  private async calculateReferralStats(userId: string, queryRunner?: QueryRunner): Promise<ReferralStats> {
    const manager = queryRunner ? queryRunner.manager : this.dataSource.manager;
    
    const user = await manager.findOne(User, { where: { id: userId } });
    if (!user) throw new NotFoundException('User not found');

    const referrals = await manager.find(Referral, { 
      where: { referrerId: userId },
      relations: ['referredUser']
    });

    const network = await this.getReferralNetwork(userId);
    const activeReferrals = await this.countActiveUsers(referrals.map(r => r.referredUser));

    return {
      totalReferrals: referrals.length,
      activeReferrals,
      totalRP: user.referralPoints || 0,
      currentTier: this.getRPTier(user.referralPoints || 0).name,
      networkSize: network.totalSize,
      qualityScore: network.qualityScore,
      lifetimeEarnings: user.lifetimeRPEarnings || 0
    };
  }

  private async updateUserRP(
    userId: string, 
    rpAmount: number, 
    source: string, 
    queryRunner?: QueryRunner
  ): Promise<void> {
    const manager = queryRunner ? queryRunner.manager : this.dataSource.manager;
    
    await manager.increment(User, { id: userId }, 'referralPoints', rpAmount);
    await manager.increment(User, { id: userId }, 'lifetimeRPEarnings', rpAmount);

    // Create transaction record
    const transaction = manager.create(Transaction, {
      userId,
      type: 'rp_earned',
      amount: rpAmount,
      source,
      status: 'completed',
      createdAt: new Date()
    });

    await manager.save(transaction);
  }

  private async awardWelcomeBonus(
    referrerId: string, 
    newUserId: string, 
    queryRunner: QueryRunner
  ): Promise<void> {
    // Award XP to both users
    const referrerXP = queryRunner.manager.create(XP, {
      userId: referrerId,
      amount: 100,
      source: 'referral_welcome',
      description: 'New referral welcome bonus',
      createdAt: new Date()
    });

    const newUserXP = queryRunner.manager.create(XP, {
      userId: newUserId,
      amount: 50,
      source: 'referred_welcome',
      description: 'Referred user welcome bonus',
      createdAt: new Date()
    });

    await queryRunner.manager.save([referrerXP, newUserXP]);
  }

  private async awardNetworkBonus(
    referrerId: string,
    sourceUserId: string,
    bonus: number,
    level: string,
    activity: string
  ): Promise<void> {
    await this.updateUserRP(referrerId, bonus, `network_${level}_${activity}`);
    
    // Create notification
    await this.notificationService.sendNetworkBonus(referrerId, bonus, level, activity);
  }

  private async isUserActive(userId: string): Promise<boolean> {
    const thirtyDaysAgo = new Date();
    thirtyDaysAgo.setDate(thirtyDaysAgo.getDate() - 30);

    const user = await this.userRepository.findOne({ 
      where: { 
        id: userId,
        lastActivity: { $gte: thirtyDaysAgo }
      }
    });

    return !!user;
  }

  private async countActiveUsers(users: User[]): Promise<number> {
    let count = 0;
    for (const user of users) {
      if (await this.isUserActive(user.id)) {
        count++;
      }
    }
    return count;
  }

  private async calculateNetworkQuality(userId: string, network: NetworkData): Promise<number> {
    const activityScores = await Promise.all([
      ...network.level1.map(u => this.getUserActivityScore(u.id)),
      ...network.level2.map(u => this.getUserActivityScore(u.id)),
      ...network.level3.map(u => this.getUserActivityScore(u.id))
    ]);

    const avgActivityScore = activityScores.reduce((a, b) => a + b, 0) / activityScores.length;
    const retentionRate = network.activeUsers / network.totalSize;
    const diversityScore = this.calculateNetworkDiversity(network);

    return (avgActivityScore * 0.4) + (retentionRate * 0.4) + (diversityScore * 0.2);
  }

  private async getUserActivityScore(userId: string): Promise<number> {
    const thirtyDaysAgo = new Date();
    thirtyDaysAgo.setDate(thirtyDaysAgo.getDate() - 30);

    const activities = await this.xpRepository.count({
      where: {
        userId,
        createdAt: { $gte: thirtyDaysAgo }
      }
    });

    return Math.min(activities / 30, 1.0); // Normalize to 0-1 scale
  }

  private calculateNetworkDiversity(network: NetworkData): number {
    // Calculate diversity based on referral distribution
    const level1Count = network.level1.length;
    const level2Count = network.level2.length;
    const level3Count = network.level3.length;
    const total = level1Count + level2Count + level3Count;

    if (total === 0) return 0;

    const l1Ratio = level1Count / total;
    const l2Ratio = level2Count / total;
    const l3Ratio = level3Count / total;

    // Shannon diversity index
    const entropy = -(l1Ratio * Math.log2(l1Ratio || 1) + 
                     l2Ratio * Math.log2(l2Ratio || 1) + 
                     l3Ratio * Math.log2(l3Ratio || 1));

    return entropy / Math.log2(3); // Normalize to 0-1
  }

  private async applyLowQualityPenalty(userId: string): Promise<void> {
    const penalty = 0.05; // 5% RP reduction
    const user = await this.userRepository.findOne({ where: { id: userId } });
    
    if (user && user.referralPoints > 0) {
      const newRP = Math.floor(user.referralPoints * (1 - penalty));
      await this.userRepository.update(userId, { referralPoints: newRP });
      
      this.logger.warn(`Low quality penalty applied to user ${userId}: ${user.referralPoints} -> ${newRP}`);
    }
  }

  private async applyHighQualityBonus(userId: string): Promise<void> {
    const bonus = 100; // Flat 100 RP bonus for high quality networks
    await this.updateUserRP(userId, bonus, 'network_quality_bonus');
    
    this.logger.log(`High quality bonus awarded to user ${userId}: +${bonus} RP`);
  }
}
