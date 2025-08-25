import { Injectable, Logger } from '@nestjs/common';
import { InjectRepository } from '@nestjs/typeorm';
import { Repository, DataSource } from 'typeorm';
import { User } from '../models/User.model';
import { Mining } from '../models/Mining.model';
import { XP } from '../models/XP.model';
import { Referral } from '../models/Referral.model';
import { BlockchainService } from './blockchain.service';
import { XPService } from './xp.service';
import { ReferralService } from './referral.service';
import { AntiBotService } from './anti-bot.service';
import { AIQualityService } from './ai-quality.service';
import { NotificationService } from './notification.service';
import { RedisService } from '@nestjs-modules/ioredis';
import { Cron, CronExpression } from '@nestjs/schedule';
import { ConfigService } from '@nestjs/config';

export enum MiningPhase {
  FINIZEN = 'finizen',
  GROWTH = 'growth',
  MATURITY = 'maturity',
  STABILITY = 'stability'
}

export interface MiningCalculationResult {
  baseRate: number;
  finazenBonus: number;
  referralBonus: number;
  securityBonus: number;
  regressionFactor: number;
  xpMultiplier: number;
  rpMultiplier: number;
  qualityScore: number;
  activityBonus: number;
  stakingBonus: number;
  specialCardBonus: number;
  finalRate: number;
  dailyCap: number;
  projectedDaily: number;
}

export interface MiningSession {
  userId: string;
  sessionId: string;
  startTime: Date;
  isActive: boolean;
  currentRate: number;
  totalEarned: number;
  lastActivityTime: Date;
  humanityScore: number;
  qualityScore: number;
}

export interface NetworkPhaseData {
  totalUsers: number;
  activeMiners: number;
  totalSupply: number;
  currentPhase: MiningPhase;
  phaseProgress: number;
}

@Injectable()
export class MiningService {
  private readonly logger = new Logger(MiningService.name);
  
  // Mining configuration constants
  private readonly MINING_PHASES = {
    [MiningPhase.FINIZEN]: {
      userThreshold: 100000,
      baseRate: 0.1,
      finazenMultiplier: 2.0,
      maxDaily: 4.8
    },
    [MiningPhase.GROWTH]: {
      userThreshold: 1000000,
      baseRate: 0.05,
      finazenMultiplier: 1.5,
      maxDaily: 1.8
    },
    [MiningPhase.MATURITY]: {
      userThreshold: 10000000,
      baseRate: 0.025,
      finazenMultiplier: 1.2,
      maxDaily: 0.72
    },
    [MiningPhase.STABILITY]: {
      userThreshold: Infinity,
      baseRate: 0.01,
      finazenMultiplier: 1.0,
      maxDaily: 0.24
    }
  };

  private readonly REGRESSION_CONFIG = {
    whaleThreshold: 10000,
    regressionCoefficient: 0.001,
    networkRegressionCoefficient: 0.0001,
    qualityRegressionCoefficient: 0.01
  };

  constructor(
    @InjectRepository(User)
    private readonly userRepository: Repository<User>,
    
    @InjectRepository(Mining)
    private readonly miningRepository: Repository<Mining>,
    
    private readonly dataSource: DataSource,
    private readonly blockchainService: BlockchainService,
    private readonly xpService: XPService,
    private readonly referralService: ReferralService,
    private readonly antiBotService: AntiBotService,
    private readonly aiQualityService: AIQualityService,
    private readonly notificationService: NotificationService,
    private readonly redisService: RedisService,
    private readonly configService: ConfigService
  ) {}

  /**
   * Start mining session for a user
   */
  async startMining(userId: string): Promise<MiningSession> {
    try {
      const user = await this.validateUser(userId);
      const existingSession = await this.getActiveMiningSession(userId);
      
      if (existingSession?.isActive) {
        throw new Error('Mining session already active');
      }

      // Perform anti-bot validation
      const humanityScore = await this.antiBotService.validateHuman(userId);
      if (humanityScore < 0.7) {
        throw new Error('Anti-bot validation failed');
      }

      // Calculate current mining rate
      const calculation = await this.calculateMiningRate(userId);
      
      // Create new mining session
      const sessionId = this.generateSessionId();
      const session: MiningSession = {
        userId,
        sessionId,
        startTime: new Date(),
        isActive: true,
        currentRate: calculation.finalRate,
        totalEarned: 0,
        lastActivityTime: new Date(),
        humanityScore,
        qualityScore: calculation.qualityScore
      };

      // Store session in Redis with 24-hour expiry
      await this.redisService.setex(
        `mining:session:${userId}`, 
        86400, 
        JSON.stringify(session)
      );

      // Record mining start in database
      await this.miningRepository.save({
        userId,
        sessionId,
        startTime: session.startTime,
        initialRate: calculation.finalRate,
        isActive: true,
        calculation: JSON.stringify(calculation)
      });

      this.logger.log(`Mining started for user ${userId} at rate ${calculation.finalRate} FIN/hour`);
      
      return session;
    } catch (error) {
      this.logger.error(`Failed to start mining for user ${userId}:`, error);
      throw error;
    }
  }

  /**
   * Stop mining session and process final rewards
   */
  async stopMining(userId: string): Promise<{ totalEarned: number; sessionDuration: number }> {
    try {
      const session = await this.getActiveMiningSession(userId);
      if (!session?.isActive) {
        throw new Error('No active mining session found');
      }

      const sessionDuration = Date.now() - session.startTime.getTime();
      const totalEarned = await this.calculateSessionEarnings(session, sessionDuration);

      // Update session as inactive
      session.isActive = false;
      session.totalEarned = totalEarned;

      await this.redisService.setex(
        `mining:session:${userId}`, 
        3600, // Keep for 1 hour for reference
        JSON.stringify(session)
      );

      // Update mining record in database
      await this.miningRepository.update(
        { userId, sessionId: session.sessionId },
        { 
          endTime: new Date(),
          totalEarned,
          isActive: false,
          sessionDuration: Math.floor(sessionDuration / 1000)
        }
      );

      // Process blockchain reward distribution
      if (totalEarned > 0) {
        await this.distributeMiningRewards(userId, totalEarned, session);
      }

      this.logger.log(`Mining stopped for user ${userId}. Earned: ${totalEarned} FIN`);
      
      return { totalEarned, sessionDuration: Math.floor(sessionDuration / 1000) };
    } catch (error) {
      this.logger.error(`Failed to stop mining for user ${userId}:`, error);
      throw error;
    }
  }

  /**
   * Calculate comprehensive mining rate for a user
   */
  async calculateMiningRate(userId: string): Promise<MiningCalculationResult> {
    try {
      const user = await this.userRepository.findOne({ 
        where: { id: userId },
        relations: ['referrals', 'stakingAccounts', 'nfts']
      });
      
      if (!user) throw new Error('User not found');

      const networkData = await this.getNetworkPhaseData();
      const phase = this.getCurrentMiningPhase(networkData.totalUsers);
      const phaseConfig = this.MINING_PHASES[phase];

      // Base calculations
      const baseRate = phaseConfig.baseRate;
      const finazenBonus = Math.max(1.0, phaseConfig.finazenMultiplier - (networkData.totalUsers / 1000000));
      
      // Referral bonus calculation
      const activeReferrals = user.referrals?.filter(r => r.isActive && r.lastActivityAt > new Date(Date.now() - 30 * 24 * 60 * 60 * 1000)).length || 0;
      const referralBonus = 1 + Math.min(activeReferrals * 0.1, 2.0); // Cap at 3x total

      // Security bonus
      const securityBonus = user.isKYCVerified ? 1.2 : 0.8;

      // Whale regression factor
      const totalHoldings = user.totalFinBalance || 0;
      const regressionFactor = Math.exp(-this.REGRESSION_CONFIG.regressionCoefficient * totalHoldings);

      // XP level multiplier
      const xpData = await this.xpService.getUserXPData(userId);
      const xpMultiplier = 1.0 + (xpData.level / 200); // Max 2.5x at level 300

      // RP tier multiplier
      const rpData = await this.referralService.getUserRPData(userId);
      const rpMultiplier = 1.0 + (rpData.tier * 0.2); // Tier-based bonus

      // Quality score from recent activities
      const qualityScore = await this.aiQualityService.getUserQualityScore(userId);

      // Activity bonus for recent engagement
      const activityBonus = await this.calculateActivityBonus(userId);

      // Staking bonus
      const stakingBonus = await this.calculateStakingBonus(userId);

      // Special NFT card bonus
      const specialCardBonus = await this.calculateSpecialCardBonus(userId);

      // Final rate calculation
      const finalRate = baseRate * 
                       finazenBonus * 
                       referralBonus * 
                       securityBonus * 
                       regressionFactor * 
                       xpMultiplier * 
                       rpMultiplier * 
                       qualityScore * 
                       activityBonus * 
                       stakingBonus * 
                       specialCardBonus;

      const dailyCap = phaseConfig.maxDaily;
      const projectedDaily = Math.min(finalRate * 24, dailyCap);

      const result: MiningCalculationResult = {
        baseRate,
        finazenBonus,
        referralBonus,
        securityBonus,
        regressionFactor,
        xpMultiplier,
        rpMultiplier,
        qualityScore,
        activityBonus,
        stakingBonus,
        specialCardBonus,
        finalRate,
        dailyCap,
        projectedDaily
      };

      // Cache calculation for 5 minutes
      await this.redisService.setex(
        `mining:calculation:${userId}`, 
        300, 
        JSON.stringify(result)
      );

      return result;
    } catch (error) {
      this.logger.error(`Failed to calculate mining rate for user ${userId}:`, error);
      throw error;
    }
  }

  /**
   * Process continuous mining rewards (called every minute)
   */
  @Cron(CronExpression.EVERY_MINUTE)
  async processContinuousMining(): Promise<void> {
    try {
      const activeSessions = await this.getAllActiveSessions();
      this.logger.log(`Processing ${activeSessions.length} active mining sessions`);

      const batchSize = 50;
      for (let i = 0; i < activeSessions.length; i += batchSize) {
        const batch = activeSessions.slice(i, i + batchSize);
        await Promise.all(batch.map(session => this.processSessionReward(session)));
      }
    } catch (error) {
      this.logger.error('Error processing continuous mining:', error);
    }
  }

  /**
   * Process individual session reward
   */
  private async processSessionReward(session: MiningSession): Promise<void> {
    try {
      const now = new Date();
      const timeDiff = now.getTime() - session.lastActivityTime.getTime();
      const minutesElapsed = Math.floor(timeDiff / (1000 * 60));

      if (minutesElapsed < 1) return; // Less than a minute elapsed

      // Re-validate user activity and humanity
      const humanityScore = await this.antiBotService.validateSessionActivity(session.userId, session.sessionId);
      if (humanityScore < 0.5) {
        await this.suspendMiningSession(session.userId, 'Anti-bot validation failed');
        return;
      }

      // Calculate reward for elapsed time
      const hourlyRate = session.currentRate;
      const minutelyReward = hourlyRate / 60;
      const reward = minutelyReward * minutesElapsed;

      // Apply daily cap check
      const todayEarned = await this.getTodayEarnings(session.userId);
      const calculation = await this.calculateMiningRate(session.userId);
      const remainingCap = Math.max(0, calculation.dailyCap - todayEarned);
      const actualReward = Math.min(reward, remainingCap);

      if (actualReward > 0) {
        // Update session
        session.totalEarned += actualReward;
        session.lastActivityTime = now;
        session.humanityScore = humanityScore;

        await this.redisService.setex(
          `mining:session:${session.userId}`, 
          86400, 
          JSON.stringify(session)
        );

        // Record reward in database
        await this.recordMiningReward(session.userId, actualReward, 'continuous_mining');
        
        // Update user balance
        await this.updateUserBalance(session.userId, actualReward);
      }

      // Check for session timeout (24 hours max)
      const sessionDuration = now.getTime() - session.startTime.getTime();
      if (sessionDuration > 24 * 60 * 60 * 1000) {
        await this.stopMining(session.userId);
      }
    } catch (error) {
      this.logger.error(`Error processing session reward for user ${session.userId}:`, error);
    }
  }

  /**
   * Calculate activity bonus based on recent engagement
   */
  private async calculateActivityBonus(userId: string): Promise<number> {
    try {
      const cacheKey = `activity:bonus:${userId}`;
      const cached = await this.redisService.get(cacheKey);
      if (cached) return parseFloat(cached);

      // Get recent activities (last 7 days)
      const recentActivities = await this.xpService.getRecentActivities(userId, 7);
      const totalActivities = recentActivities.length;
      const uniquePlatforms = new Set(recentActivities.map(a => a.platform)).size;
      const avgQuality = recentActivities.reduce((sum, a) => sum + a.qualityScore, 0) / totalActivities || 1;

      // Calculate bonus: base 1.0x, up to 2.0x for very active users
      const activityScore = Math.min(totalActivities / 50, 1); // Up to 50 activities for max bonus
      const diversityScore = Math.min(uniquePlatforms / 5, 1); // Up to 5 platforms for max bonus
      const bonus = 1.0 + (activityScore * diversityScore * avgQuality * 1.0);

      // Cache for 1 hour
      await this.redisService.setex(cacheKey, 3600, bonus.toString());
      
      return Math.min(bonus, 2.0); // Cap at 2.0x
    } catch (error) {
      this.logger.error(`Error calculating activity bonus for user ${userId}:`, error);
      return 1.0; // Default to no bonus on error
    }
  }

  /**
   * Calculate staking bonus
   */
  private async calculateStakingBonus(userId: string): Promise<number> {
    try {
      const user = await this.userRepository.findOne({
        where: { id: userId },
        relations: ['stakingAccounts']
      });

      if (!user?.stakingAccounts?.length) return 1.0;

      const totalStaked = user.stakingAccounts.reduce((sum, account) => sum + account.stakedAmount, 0);
      
      // Staking bonus tiers
      if (totalStaked >= 10000) return 2.0;      // 10K+ FIN: 100% bonus
      if (totalStaked >= 5000) return 1.75;      // 5K+ FIN: 75% bonus  
      if (totalStaked >= 1000) return 1.5;       // 1K+ FIN: 50% bonus
      if (totalStaked >= 500) return 1.35;       // 500+ FIN: 35% bonus
      if (totalStaked >= 100) return 1.2;        // 100+ FIN: 20% bonus
      
      return 1.0;
    } catch (error) {
      this.logger.error(`Error calculating staking bonus for user ${userId}:`, error);
      return 1.0;
    }
  }

  /**
   * Calculate special NFT card bonuses
   */
  private async calculateSpecialCardBonus(userId: string): Promise<number> {
    try {
      const activeCards = await this.getActiveSpecialCards(userId);
      if (!activeCards.length) return 1.0;

      let totalBonus = 1.0;
      let cardCount = 0;

      for (const card of activeCards) {
        switch (card.type) {
          case 'double_mining':
            totalBonus *= 2.0;
            cardCount++;
            break;
          case 'triple_mining':
            totalBonus *= 3.0;
            cardCount++;
            break;
          case 'mining_frenzy':
            totalBonus *= 6.0;
            cardCount++;
            break;
          case 'eternal_miner':
            totalBonus *= 1.5;
            cardCount++;
            break;
        }
      }

      // Apply synergy bonus for multiple cards
      if (cardCount > 1) {
        const synergyBonus = 1 + (cardCount * 0.1);
        totalBonus *= synergyBonus;
      }

      return Math.min(totalBonus, 20.0); // Cap total bonus at 20x
    } catch (error) {
      this.logger.error(`Error calculating special card bonus for user ${userId}:`, error);
      return 1.0;
    }
  }

  /**
   * Distribute mining rewards to blockchain and referral network
   */
  private async distributeMiningRewards(userId: string, amount: number, session: MiningSession): Promise<void> {
    const queryRunner = this.dataSource.createQueryRunner();
    await queryRunner.connect();
    await queryRunner.startTransaction();

    try {
      // 1. Mint tokens to user
      await this.blockchainService.mintTokens(userId, amount);
      
      // 2. Distribute referral rewards
      const referralRewards = await this.referralService.calculateReferralRewards(userId, amount);
      for (const reward of referralRewards) {
        await this.blockchainService.mintTokens(reward.referrerId, reward.amount);
        await this.recordMiningReward(reward.referrerId, reward.amount, 'referral_bonus');
      }

      // 3. Record XP gains from mining
      const xpGain = Math.floor(amount * 10); // 10 XP per FIN mined
      await this.xpService.addXP(userId, xpGain, 'mining_reward');

      // 4. Update user statistics
      await queryRunner.manager.increment(User, { id: userId }, 'totalMinedFin', amount);
      await queryRunner.manager.increment(User, { id: userId }, 'totalFinBalance', amount);

      // 5. Send notifications
      await this.notificationService.sendMiningRewardNotification(userId, amount);

      await queryRunner.commitTransaction();
      
      this.logger.log(`Successfully distributed mining rewards: ${amount} FIN to user ${userId}`);
    } catch (error) {
      await queryRunner.rollbackTransaction();
      this.logger.error(`Failed to distribute mining rewards for user ${userId}:`, error);
      throw error;
    } finally {
      await queryRunner.release();
    }
  }

  /**
   * Get current mining phase based on total users
   */
  private getCurrentMiningPhase(totalUsers: number): MiningPhase {
    if (totalUsers < this.MINING_PHASES[MiningPhase.FINIZEN].userThreshold) {
      return MiningPhase.FINIZEN;
    } else if (totalUsers < this.MINING_PHASES[MiningPhase.GROWTH].userThreshold) {
      return MiningPhase.GROWTH;
    } else if (totalUsers < this.MINING_PHASES[MiningPhase.MATURITY].userThreshold) {
      return MiningPhase.MATURITY;
    } else {
      return MiningPhase.STABILITY;
    }
  }

  /**
   * Get network phase data
   */
  private async getNetworkPhaseData(): Promise<NetworkPhaseData> {
    const cacheKey = 'network:phase:data';
    const cached = await this.redisService.get(cacheKey);
    if (cached) return JSON.parse(cached);

    const totalUsers = await this.userRepository.count();
    const activeMiners = await this.userRepository.count({ 
      where: { lastMiningAt: new Date(Date.now() - 24 * 60 * 60 * 1000) } 
    });
    const totalSupply = await this.blockchainService.getTotalSupply();
    const currentPhase = this.getCurrentMiningPhase(totalUsers);
    
    const phaseConfig = this.MINING_PHASES[currentPhase];
    const phaseProgress = totalUsers / phaseConfig.userThreshold;

    const data: NetworkPhaseData = {
      totalUsers,
      activeMiners,
      totalSupply,
      currentPhase,
      phaseProgress: Math.min(phaseProgress, 1.0)
    };

    // Cache for 5 minutes
    await this.redisService.setex(cacheKey, 300, JSON.stringify(data));
    return data;
  }

  /**
   * Utility methods
   */
  private async validateUser(userId: string): Promise<User> {
    const user = await this.userRepository.findOne({ where: { id: userId } });
    if (!user) throw new Error('User not found');
    if (user.isBanned) throw new Error('User is banned');
    if (!user.isActive) throw new Error('User account is inactive');
    return user;
  }

  private generateSessionId(): string {
    return `session_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
  }

  private async getActiveMiningSession(userId: string): Promise<MiningSession | null> {
    const sessionData = await this.redisService.get(`mining:session:${userId}`);
    return sessionData ? JSON.parse(sessionData) : null;
  }

  private async getAllActiveSessions(): Promise<MiningSession[]> {
    const keys = await this.redisService.keys('mining:session:*');
    const sessions = await Promise.all(
      keys.map(async (key) => {
        const data = await this.redisService.get(key);
        return data ? JSON.parse(data) : null;
      })
    );
    return sessions.filter(s => s?.isActive);
  }

  private async calculateSessionEarnings(session: MiningSession, duration: number): Promise<number> {
    const hoursElapsed = duration / (1000 * 60 * 60);
    const baseEarnings = session.currentRate * hoursElapsed;
    
    // Apply daily cap
    const calculation = await this.calculateMiningRate(session.userId);
    return Math.min(baseEarnings, calculation.dailyCap);
  }

  private async getTodayEarnings(userId: string): Promise<number> {
    const startOfDay = new Date();
    startOfDay.setHours(0, 0, 0, 0);
    
    const result = await this.miningRepository
      .createQueryBuilder('mining')
      .select('SUM(mining.totalEarned)', 'total')
      .where('mining.userId = :userId', { userId })
      .andWhere('mining.createdAt >= :startOfDay', { startOfDay })
      .getRawOne();
    
    return parseFloat(result.total) || 0;
  }

  private async recordMiningReward(userId: string, amount: number, type: string): Promise<void> {
    await this.miningRepository.save({
      userId,
      amount,
      type,
      timestamp: new Date()
    });
  }

  private async updateUserBalance(userId: string, amount: number): Promise<void> {
    await this.userRepository.increment({ id: userId }, 'totalFinBalance', amount);
  }

  private async getActiveSpecialCards(userId: string): Promise<any[]> {
    // Implementation would query NFT service for active special cards
    // This is a placeholder for the actual implementation
    return [];
  }

  private async suspendMiningSession(userId: string, reason: string): Promise<void> {
    await this.redisService.del(`mining:session:${userId}`);
    await this.miningRepository.update(
      { userId, isActive: true },
      { isActive: false, suspensionReason: reason }
    );
    
    this.logger.warn(`Mining session suspended for user ${userId}: ${reason}`);
  }

  /**
   * Public API methods for frontend/mobile clients
   */
  async getUserMiningStatus(userId: string): Promise<{
    isActive: boolean;
    currentRate: number;
    todayEarned: number;
    totalEarned: number;
    dailyCap: number;
    timeRemaining: number;
    nextPhaseAt: number;
  }> {
    const session = await this.getActiveMiningSession(userId);
    const calculation = await this.calculateMiningRate(userId);
    const todayEarned = await this.getTodayEarnings(userId);
    const networkData = await this.getNetworkPhaseData();
    
    const user = await this.userRepository.findOne({ where: { id: userId } });
    const totalEarned = user?.totalMinedFin || 0;
    
    const timeRemaining = session ? 
      Math.max(0, 24 * 60 * 60 * 1000 - (Date.now() - session.startTime.getTime())) : 0;
    
    const currentPhase = this.MINING_PHASES[networkData.currentPhase];
    const nextPhaseAt = currentPhase.userThreshold === Infinity ? 0 : currentPhase.userThreshold;

    return {
      isActive: !!session?.isActive,
      currentRate: calculation.finalRate,
      todayEarned,
      totalEarned,
      dailyCap: calculation.dailyCap,
      timeRemaining: Math.floor(timeRemaining / 1000),
      nextPhaseAt
    };
  }

  async getMiningLeaderboard(limit: number = 100): Promise<Array<{
    userId: string;
    username: string;
    totalMined: number;
    rank: number;
  }>> {
    const results = await this.userRepository
      .createQueryBuilder('user')
      .select(['user.id', 'user.username', 'user.totalMinedFin'])
      .where('user.totalMinedFin > 0')
      .orderBy('user.totalMinedFin', 'DESC')
      .limit(limit)
      .getMany();

    return results.map((user, index) => ({
      userId: user.id,
      username: user.username,
      totalMined: user.totalMinedFin || 0,
      rank: index + 1
    }));
  }
}
