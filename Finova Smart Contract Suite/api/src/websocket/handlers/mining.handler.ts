import { Socket } from 'socket.io';
import { Logger } from 'winston';
import { Redis } from 'ioredis';
import { Connection, PublicKey } from '@solana/web3.js';
import { MiningService } from '../../services/mining.service';
import { XPService } from '../../services/xp.service';
import { ReferralService } from '../../services/referral.service';
import { AntiBotService } from '../../services/anti-bot.service';
import { AIQualityService } from '../../services/ai-quality.service';
import { BlockchainService } from '../../services/blockchain.service';
import { NotificationService } from '../../services/notification.service';
import { AnalyticsService } from '../../services/analytics.service';
import { User } from '../../models/User.model';
import { MiningSession } from '../../models/Mining.model';
import { 
  MiningStartRequest, 
  MiningStopRequest, 
  MiningStatusRequest,
  ActivityBoostRequest,
  ClaimRewardsRequest,
  MiningResponse,
  ActivityData,
  QualityAssessmentResult
} from '../../types/mining.types';

export class MiningHandler {
  private logger: Logger;
  private redis: Redis;
  private solanaConnection: Connection;
  private miningService: MiningService;
  private xpService: XPService;
  private referralService: ReferralService;
  private antiBotService: AntiBotService;
  private aiQualityService: AIQualityService;
  private blockchainService: BlockchainService;
  private notificationService: NotificationService;
  private analyticsService: AnalyticsService;
  
  // Real-time mining sessions tracking
  private activeSessions: Map<string, MiningSession> = new Map();
  private userSockets: Map<string, Socket> = new Map();
  
  // Mining phase constants (from whitepaper)
  private readonly MINING_PHASES = {
    FINIZEN: { threshold: 100000, baseRate: 0.1, bonus: 2.0 },
    GROWTH: { threshold: 1000000, baseRate: 0.05, bonus: 1.5 },
    MATURITY: { threshold: 10000000, baseRate: 0.025, bonus: 1.2 },
    STABILITY: { threshold: Infinity, baseRate: 0.01, bonus: 1.0 }
  };

  constructor(
    logger: Logger,
    redis: Redis,
    solanaConnection: Connection,
    services: {
      miningService: MiningService;
      xpService: XPService;
      referralService: ReferralService;
      antiBotService: AntiBotService;
      aiQualityService: AIQualityService;
      blockchainService: BlockchainService;
      notificationService: NotificationService;
      analyticsService: AnalyticsService;
    }
  ) {
    this.logger = logger;
    this.redis = redis;
    this.solanaConnection = solanaConnection;
    
    // Inject services
    Object.assign(this, services);
    
    // Initialize mining calculation intervals
    this.initializeMiningCalculations();
    
    this.logger.info('MiningHandler initialized successfully');
  }

  /**
   * Handle new WebSocket connection for mining
   */
  public handleConnection(socket: Socket, user: User): void {
    try {
      this.userSockets.set(user.id, socket);
      
      // Set up event listeners
      this.setupEventListeners(socket, user);
      
      // Send initial mining status
      this.sendMiningStatus(socket, user);
      
      // Track connection analytics
      this.analyticsService.trackEvent('mining_websocket_connected', {
        userId: user.id,
        userLevel: user.xpLevel,
        rpTier: user.rpTier,
        timestamp: new Date()
      });
      
      this.logger.info(`Mining WebSocket connected for user ${user.id}`);
      
    } catch (error) {
      this.logger.error('Error handling mining connection:', error);
      socket.emit('mining:error', { message: 'Connection setup failed' });
    }
  }

  /**
   * Handle WebSocket disconnection
   */
  public handleDisconnection(socket: Socket, user: User): void {
    try {
      // Stop active mining session if exists
      if (this.activeSessions.has(user.id)) {
        this.stopMining(socket, user, { reason: 'disconnect' });
      }
      
      // Clean up tracking
      this.userSockets.delete(user.id);
      
      // Analytics
      this.analyticsService.trackEvent('mining_websocket_disconnected', {
        userId: user.id,
        timestamp: new Date()
      });
      
      this.logger.info(`Mining WebSocket disconnected for user ${user.id}`);
      
    } catch (error) {
      this.logger.error('Error handling mining disconnection:', error);
    }
  }

  /**
   * Set up all WebSocket event listeners for mining
   */
  private setupEventListeners(socket: Socket, user: User): void {
    // Core mining events
    socket.on('mining:start', (data: MiningStartRequest) => 
      this.startMining(socket, user, data));
    
    socket.on('mining:stop', (data: MiningStopRequest) => 
      this.stopMining(socket, user, data));
    
    socket.on('mining:status', (data: MiningStatusRequest) => 
      this.sendMiningStatus(socket, user, data));
    
    socket.on('mining:claim', (data: ClaimRewardsRequest) => 
      this.claimRewards(socket, user, data));
    
    // Activity and boost events
    socket.on('mining:activity', (data: ActivityData) => 
      this.processActivity(socket, user, data));
    
    socket.on('mining:boost', (data: ActivityBoostRequest) => 
      this.applyBoost(socket, user, data));
    
    // Real-time calculation requests
    socket.on('mining:calculate', () => 
      this.calculateRealTimeRewards(socket, user));
    
    // Heartbeat for connection health
    socket.on('mining:heartbeat', () => 
      this.handleHeartbeat(socket, user));
  }

  /**
   * Start mining session with comprehensive validation
   */
  private async startMining(
    socket: Socket, 
    user: User, 
    data: MiningStartRequest
  ): Promise<void> {
    try {
      // Anti-bot validation
      const humanScore = await this.antiBotService.validateUserActivity(user.id);
      if (humanScore < 0.7) {
        socket.emit('mining:error', { 
          code: 'BOT_DETECTED',
          message: 'Suspicious activity detected. Mining disabled.',
          humanScore 
        });
        return;
      }

      // Check existing session
      if (this.activeSessions.has(user.id)) {
        socket.emit('mining:error', { 
          code: 'SESSION_EXISTS',
          message: 'Mining session already active' 
        });
        return;
      }

      // Validate mining eligibility
      const eligibility = await this.validateMiningEligibility(user);
      if (!eligibility.eligible) {
        socket.emit('mining:error', { 
          code: 'NOT_ELIGIBLE',
          message: eligibility.reason 
        });
        return;
      }

      // Calculate initial mining rate
      const miningRate = await this.calculateMiningRate(user);
      
      // Create mining session
      const session: MiningSession = {
        id: `mining_${user.id}_${Date.now()}`,
        userId: user.id,
        startTime: new Date(),
        lastCalculation: new Date(),
        currentRate: miningRate.hourlyRate,
        totalEarned: 0,
        xpMultiplier: miningRate.xpMultiplier,
        rpMultiplier: miningRate.rpMultiplier,
        qualityScore: 1.0,
        boosts: [],
        activities: [],
        status: 'active'
      };

      // Store session
      this.activeSessions.set(user.id, session);
      await this.redis.setex(
        `mining_session:${user.id}`, 
        86400, // 24 hours
        JSON.stringify(session)
      );

      // Start blockchain mining transaction
      const miningTx = await this.blockchainService.startMining(
        user.walletAddress,
        miningRate.hourlyRate
      );

      // Send success response
      const response: MiningResponse = {
        success: true,
        session: {
          id: session.id,
          startTime: session.startTime,
          currentRate: session.currentRate,
          estimatedDaily: session.currentRate * 24,
          phase: this.getCurrentMiningPhase(),
          multipliers: {
            xp: session.xpMultiplier,
            rp: session.rpMultiplier,
            staking: await this.getStakingMultiplier(user),
            network: miningRate.networkBonus
          },
          humanScore,
          transactionHash: miningTx.signature
        }
      };

      socket.emit('mining:started', response);

      // Send to referral network
      await this.notifyReferralNetwork(user, 'mining_started', {
        rate: session.currentRate,
        multipliers: response.session.multipliers
      });

      // Analytics
      this.analyticsService.trackEvent('mining_session_started', {
        userId: user.id,
        rate: session.currentRate,
        phase: this.getCurrentMiningPhase(),
        humanScore,
        timestamp: new Date()
      });

      this.logger.info(`Mining started for user ${user.id} at rate ${session.currentRate}`);

    } catch (error) {
      this.logger.error('Error starting mining:', error);
      socket.emit('mining:error', { 
        code: 'START_FAILED',
        message: 'Failed to start mining session' 
      });
    }
  }

  /**
   * Stop mining session and calculate final rewards
   */
  private async stopMining(
    socket: Socket, 
    user: User, 
    data: MiningStopRequest
  ): Promise<void> {
    try {
      const session = this.activeSessions.get(user.id);
      if (!session) {
        socket.emit('mining:error', { 
          code: 'NO_SESSION',
          message: 'No active mining session found' 
        });
        return;
      }

      // Calculate final rewards
      const finalRewards = await this.calculateSessionRewards(session, user);

      // Update user balance
      await this.miningService.addUserRewards(user.id, {
        finTokens: finalRewards.totalFIN,
        xpPoints: finalRewards.totalXP,
        rpPoints: finalRewards.totalRP
      });

      // Stop blockchain mining
      await this.blockchainService.stopMining(user.walletAddress);

      // Clean up session
      this.activeSessions.delete(user.id);
      await this.redis.del(`mining_session:${user.id}`);

      // Update session status
      session.status = 'completed';
      session.endTime = new Date();
      session.totalEarned = finalRewards.totalFIN;

      // Store completed session for analytics
      await this.miningService.saveMiningSession(session, finalRewards);

      // Send final response
      const response: MiningResponse = {
        success: true,
        rewards: {
          finTokens: finalRewards.totalFIN,
          xpPoints: finalRewards.totalXP,
          rpPoints: finalRewards.totalRP,
          sessionDuration: session.endTime.getTime() - session.startTime.getTime(),
          averageRate: finalRewards.totalFIN / (
            (session.endTime.getTime() - session.startTime.getTime()) / 3600000
          )
        }
      };

      socket.emit('mining:stopped', response);

      // Notify referral network
      await this.notifyReferralNetwork(user, 'mining_stopped', {
        rewards: finalRewards,
        duration: response.rewards.sessionDuration
      });

      // Analytics
      this.analyticsService.trackEvent('mining_session_completed', {
        userId: user.id,
        duration: response.rewards.sessionDuration,
        totalRewards: finalRewards.totalFIN,
        averageRate: response.rewards.averageRate,
        timestamp: new Date()
      });

      this.logger.info(`Mining stopped for user ${user.id}. Total earned: ${finalRewards.totalFIN} FIN`);

    } catch (error) {
      this.logger.error('Error stopping mining:', error);
      socket.emit('mining:error', { 
        code: 'STOP_FAILED',
        message: 'Failed to stop mining session' 
      });
    }
  }

  /**
   * Process social media activity for XP and mining boosts
   */
  private async processActivity(
    socket: Socket, 
    user: User, 
    data: ActivityData
  ): Promise<void> {
    try {
      // Validate activity data
      if (!this.validateActivityData(data)) {
        socket.emit('mining:error', { 
          code: 'INVALID_ACTIVITY',
          message: 'Invalid activity data provided' 
        });
        return;
      }

      // AI quality assessment
      const qualityResult = await this.aiQualityService.assessContent({
        type: data.type,
        content: data.content,
        platform: data.platform,
        userId: user.id
      });

      // Anti-bot validation for activity
      const activityValid = await this.antiBotService.validateActivity(user.id, data);
      if (!activityValid) {
        socket.emit('mining:error', { 
          code: 'SUSPICIOUS_ACTIVITY',
          message: 'Activity appears to be automated' 
        });
        return;
      }

      // Calculate XP reward
      const xpReward = await this.xpService.calculateActivityXP(user, data, qualityResult);

      // Apply XP to user
      await this.xpService.addUserXP(user.id, xpReward);

      // Update mining session if active
      const session = this.activeSessions.get(user.id);
      if (session) {
        session.activities.push({
          type: data.type,
          platform: data.platform,
          timestamp: new Date(),
          xpEarned: xpReward.total,
          qualityScore: qualityResult.score,
          miningBoost: xpReward.miningBoost
        });

        // Recalculate mining rate with activity boost
        const newRate = await this.recalculateMiningRate(user, session);
        session.currentRate = newRate;

        // Update Redis
        await this.redis.setex(
          `mining_session:${user.id}`,
          86400,
          JSON.stringify(session)
        );
      }

      // Send activity processed response
      socket.emit('mining:activity_processed', {
        activity: {
          type: data.type,
          platform: data.platform,
          xpEarned: xpReward.total,
          qualityScore: qualityResult.score,
          miningBoost: session ? xpReward.miningBoost : 0
        },
        updatedMiningRate: session?.currentRate || 0,
        newXPLevel: await this.xpService.getUserLevel(user.id)
      });

      // Update referral network
      await this.referralService.processReferralActivity(user.id, {
        type: 'activity',
        xpEarned: xpReward.total,
        platform: data.platform
      });

      this.logger.info(`Activity processed for user ${user.id}: ${xpReward.total} XP earned`);

    } catch (error) {
      this.logger.error('Error processing activity:', error);
      socket.emit('mining:error', { 
        code: 'ACTIVITY_FAILED',
        message: 'Failed to process activity' 
      });
    }
  }

  /**
   * Apply special card or boost to mining session
   */
  private async applyBoost(
    socket: Socket, 
    user: User, 
    data: ActivityBoostRequest
  ): Promise<void> {
    try {
      const session = this.activeSessions.get(user.id);
      if (!session) {
        socket.emit('mining:error', { 
          code: 'NO_SESSION',
          message: 'No active mining session to boost' 
        });
        return;
      }

      // Validate boost item ownership
      const hasBoost = await this.miningService.validateBoostOwnership(user.id, data.boostId);
      if (!hasBoost) {
        socket.emit('mining:error', { 
          code: 'BOOST_NOT_OWNED',
          message: 'You do not own this boost item' 
        });
        return;
      }

      // Get boost details
      const boostDetails = await this.miningService.getBoostDetails(data.boostId);
      
      // Apply boost to session
      const boost = {
        id: data.boostId,
        type: boostDetails.type,
        multiplier: boostDetails.multiplier,
        duration: boostDetails.duration,
        startTime: new Date(),
        endTime: new Date(Date.now() + boostDetails.duration * 1000)
      };

      session.boosts.push(boost);

      // Recalculate mining rate with boost
      const newRate = await this.recalculateMiningRate(user, session);
      session.currentRate = newRate;

      // Consume boost item if single-use
      if (boostDetails.singleUse) {
        await this.miningService.consumeBoost(user.id, data.boostId);
      }

      // Update Redis
      await this.redis.setex(
        `mining_session:${user.id}`,
        86400,
        JSON.stringify(session)
      );

      // Send boost applied response
      socket.emit('mining:boost_applied', {
        boost: {
          type: boostDetails.type,
          multiplier: boostDetails.multiplier,
          duration: boostDetails.duration,
          endTime: boost.endTime
        },
        newMiningRate: newRate,
        estimatedAdditionalRewards: (newRate - session.currentRate) * (boostDetails.duration / 3600)
      });

      this.logger.info(`Boost ${data.boostId} applied to user ${user.id} mining session`);

    } catch (error) {
      this.logger.error('Error applying boost:', error);
      socket.emit('mining:error', { 
        code: 'BOOST_FAILED',
        message: 'Failed to apply boost' 
      });
    }
  }

  /**
   * Send real-time mining status
   */
  private async sendMiningStatus(
    socket: Socket, 
    user: User, 
    data?: MiningStatusRequest
  ): Promise<void> {
    try {
      const session = this.activeSessions.get(user.id);
      const userStats = await this.miningService.getUserMiningStats(user.id);
      const networkStats = await this.miningService.getNetworkStats();

      const status = {
        isActive: !!session,
        session: session ? {
          id: session.id,
          startTime: session.startTime,
          currentRate: session.currentRate,
          totalEarned: session.totalEarned,
          duration: Date.now() - session.startTime.getTime(),
          activeBoosts: session.boosts.filter(b => b.endTime > new Date()),
          recentActivities: session.activities.slice(-5)
        } : null,
        user: {
          totalFIN: userStats.totalFIN,
          totalXP: userStats.totalXP,
          xpLevel: userStats.xpLevel,
          rpTier: userStats.rpTier,
          stakingMultiplier: await this.getStakingMultiplier(user),
          humanScore: await this.antiBotService.getUserHumanScore(user.id)
        },
        network: {
          phase: this.getCurrentMiningPhase(),
          totalUsers: networkStats.totalUsers,
          totalMiners: networkStats.activeMiners,
          networkRate: networkStats.averageRate
        },
        estimatedRewards: session ? 
          await this.calculateEstimatedRewards(session, user) : null
      };

      socket.emit('mining:status', status);

    } catch (error) {
      this.logger.error('Error sending mining status:', error);
      socket.emit('mining:error', { 
        code: 'STATUS_FAILED',
        message: 'Failed to get mining status' 
      });
    }
  }

  /**
   * Calculate comprehensive mining rate with all multipliers
   */
  private async calculateMiningRate(user: User): Promise<any> {
    try {
      // Get current network phase
      const phase = this.getCurrentMiningPhase();
      const totalUsers = await this.miningService.getTotalUsers();
      
      // Base calculations from whitepaper formulas
      const baseRate = phase.baseRate;
      const finicenBonus = Math.max(1.0, phase.bonus - (totalUsers / 1000000));
      
      // Get user referral network
      const referralData = await this.referralService.getUserReferralData(user.id);
      const referralBonus = 1 + (referralData.activeReferrals * 0.1);
      
      // Security and KYC bonus
      const securityBonus = user.isKYCVerified ? 1.2 : 0.8;
      
      // Exponential regression factor (anti-whale mechanism)
      const regressionFactor = Math.exp(-0.001 * user.totalFINHoldings);
      
      // XP level multiplier
      const xpMultiplier = 1.0 + (user.xpLevel / 100);
      
      // RP tier multiplier
      const rpMultiplier = 1.0 + (referralData.tier * 0.2);
      
      // Staking multiplier
      const stakingMultiplier = await this.getStakingMultiplier(user);
      
      // Final mining rate calculation
      const hourlyRate = baseRate * finicenBonus * referralBonus * securityBonus * 
                        regressionFactor * xpMultiplier * rpMultiplier * stakingMultiplier;

      return {
        hourlyRate: Math.max(0.001, hourlyRate), // Minimum rate
        xpMultiplier,
        rpMultiplier,
        stakingMultiplier,
        networkBonus: finicenBonus,
        regressionFactor,
        components: {
          baseRate,
          finicenBonus,
          referralBonus,
          securityBonus,
          regressionFactor
        }
      };

    } catch (error) {
      this.logger.error('Error calculating mining rate:', error);
      throw error;
    }
  }

  /**
   * Initialize real-time mining calculations
   */
  private initializeMiningCalculations(): void {
    // Update mining rewards every minute
    setInterval(async () => {
      try {
        for (const [userId, session] of this.activeSessions.entries()) {
          const user = await User.findById(userId);
          if (!user) continue;

          // Calculate time-based rewards
          const timeDiff = (Date.now() - session.lastCalculation.getTime()) / 3600000; // hours
          const timeRewards = session.currentRate * timeDiff;
          
          session.totalEarned += timeRewards;
          session.lastCalculation = new Date();

          // Update Redis
          await this.redis.setex(
            `mining_session:${userId}`,
            86400,
            JSON.stringify(session)
          );

          // Send real-time update to connected socket
          const socket = this.userSockets.get(userId);
          if (socket) {
            socket.emit('mining:realtime_update', {
              totalEarned: session.totalEarned,
              currentRate: session.currentRate,
              lastUpdate: session.lastCalculation,
              estimatedDaily: session.currentRate * 24
            });
          }
        }
      } catch (error) {
        this.logger.error('Error in mining calculations interval:', error);
      }
    }, 60000); // Every minute

    // Clean up expired boosts every 5 minutes
    setInterval(async () => {
      try {
        for (const [userId, session] of this.activeSessions.entries()) {
          const now = new Date();
          const activeBoosts = session.boosts.filter(boost => boost.endTime > now);
          
          if (activeBoosts.length !== session.boosts.length) {
            session.boosts = activeBoosts;
            
            // Recalculate mining rate without expired boosts
            const user = await User.findById(userId);
            if (user) {
              const newRate = await this.recalculateMiningRate(user, session);
              session.currentRate = newRate;

              // Update Redis
              await this.redis.setex(
                `mining_session:${userId}`,
                86400,
                JSON.stringify(session)
              );

              // Notify user of boost expiration
              const socket = this.userSockets.get(userId);
              if (socket) {
                socket.emit('mining:boost_expired', {
                  newMiningRate: newRate,
                  activeBoosts: session.boosts
                });
              }
            }
          }
        }
      } catch (error) {
        this.logger.error('Error cleaning expired boosts:', error);
      }
    }, 300000); // Every 5 minutes

    this.logger.info('Mining calculation intervals initialized');
  }

  /**
   * Get current mining phase based on total users
   */
  private getCurrentMiningPhase(): any {
    // This would typically fetch from database
    const totalUsers = 50000; // Placeholder
    
    for (const [phase, config] of Object.entries(this.MINING_PHASES)) {
      if (totalUsers < config.threshold) {
        return { name: phase, ...config };
      }
    }
    
    return { name: 'STABILITY', ...this.MINING_PHASES.STABILITY };
  }

  /**
   * Validate mining eligibility with comprehensive checks
   */
  private async validateMiningEligibility(user: User): Promise<{ eligible: boolean; reason?: string }> {
    // Check KYC status
    if (!user.isKYCVerified) {
      return { eligible: false, reason: 'KYC verification required' };
    }

    // Check daily mining limit
    const dailyMined = await this.miningService.getDailyMined(user.id);
    const maxDaily = await this.miningService.getMaxDailyMining(user);
    
    if (dailyMined >= maxDaily) {
      return { eligible: false, reason: 'Daily mining limit reached' };
    }

    // Check suspension status
    if (user.isSuspended) {
      return { eligible: false, reason: 'Account suspended' };
    }

    // Anti-bot check
    const humanScore = await this.antiBotService.getUserHumanScore(user.id);
    if (humanScore < 0.5) {
      return { eligible: false, reason: 'Account flagged for suspicious activity' };
    }

    return { eligible: true };
  }

  /**
   * Additional utility methods for comprehensive functionality
   */
  private async getStakingMultiplier(user: User): Promise<number> {
    const stakingData = await this.miningService.getUserStakingData(user.id);
    if (!stakingData || stakingData.stakedAmount === 0) return 1.0;

    // Staking multiplier based on tier (from whitepaper)
    if (stakingData.stakedAmount >= 10000) return 2.0;
    if (stakingData.stakedAmount >= 5000) return 1.75;
    if (stakingData.stakedAmount >= 1000) return 1.5;
    if (stakingData.stakedAmount >= 500) return 1.35;
    if (stakingData.stakedAmount >= 100) return 1.2;
    
    return 1.0;
  }

  private validateActivityData(data: ActivityData): boolean {
    return !!(data.type && data.platform && data.content && 
             ['post', 'comment', 'like', 'share', 'story'].includes(data.type) &&
             ['instagram', 'tiktok', 'youtube', 'facebook', 'twitter'].includes(data.platform));
  }

  private async calculateEstimatedRewards(session: MiningSession, user: User): Promise<any> {
    const hoursRemaining = 24 - ((Date.now() - session.startTime.getTime()) / 3600000);
    return {
      nextHour: session.currentRate,
      remainingToday: session.currentRate * Math.max(0, hoursRemaining),
      nextWeek: session.currentRate * 24 * 7,
      nextMonth: session.currentRate * 24 * 30
    };
  }

  private async recalculateMiningRate(user: User, session: MiningSession): Promise<number> {
    const baseRate = await this.calculateMiningRate(user);
    
    // Apply active boosts
    let boostMultiplier = 1.0;
    const now = new Date();
    
    for (const boost of session.boosts) {
      if (boost.endTime > now) {
        boostMultiplier *= boost.multiplier;
      }
    }

    // Apply activity quality bonus
    const avgQuality = session.activities.length > 0 ? 
      session.activities.reduce((sum, act) => sum + act.qualityScore, 0) / session.activities.length : 1.0;

    return baseRate.hourlyRate * boostMultiplier * avgQuality;
  }

  private async calculateSessionRewards(session: MiningSession, user: User): Promise<any> {
    const sessionHours = (Date.now() - session.startTime.getTime()) / 3600000;
    const totalFIN = session.totalEarned;
    
    // Calculate XP rewards based on activities
    const totalXP = session.activities.reduce((sum, act) => sum + act.xpEarned, 0);
    
    // Calculate RP rewards from network activity during session
    const rpRewards = await this.referralService.calculateSessionRP(user.id, session.startTime);

    return {
      totalFIN,
      totalXP,
      totalRP: rpRewards,
      sessionHours,
      averageRate: totalFIN / sessionHours
    };
  }

  private async notifyReferralNetwork(user: User, event: string, data: any): Promise<void> {
    const referrals = await this.referralService.getUserReferrals(user.id);
    
    for (const referral of referrals) {
      const referralSocket = this.userSockets.get(referral.id);
      if (referralSocket) {
        referralSocket.emit('mining:referral_update', {
          referrerId: user.id,
          referrerName: user.displayName,
          event,
          data,
          rpBonus: await this.referralService.calculateRPBonus(referral.id, event, data)
        });
      }
      
      // Send push notification for major events
      if (['mining_started', 'mining_stopped'].includes(event)) {
        await this.notificationService.sendPushNotification(referral.id, {
          title: 'Referral Network Activity',
          body: `${user.displayName} ${event.replace('_', ' ')}`,
          data: { type: 'referral_mining', userId: user.id }
        });
      }
    }
  }

  private async handleHeartbeat(socket: Socket, user: User): Promise<void> {
    try {
      const session = this.activeSessions.get(user.id);
      if (session) {
        // Update last activity timestamp
        session.lastCalculation = new Date();
        
        // Send heartbeat response with current status
        socket.emit('mining:heartbeat_ack', {
          timestamp: new Date(),
          sessionActive: true,
          currentRate: session.currentRate,
          totalEarned: session.totalEarned
        });
      } else {
        socket.emit('mining:heartbeat_ack', {
          timestamp: new Date(),
          sessionActive: false
        });
      }
    } catch (error) {
      this.logger.error('Error handling heartbeat:', error);
    }
  }

  private async calculateRealTimeRewards(socket: Socket, user: User): Promise<void> {
    try {
      const session = this.activeSessions.get(user.id);
      if (!session) {
        socket.emit('mining:calculation_result', { error: 'No active session' });
        return;
      }

      // Calculate current rewards
      const currentRewards = await this.calculateSessionRewards(session, user);
      
      // Get projections
      const projections = await this.calculateEstimatedRewards(session, user);
      
      // Get network comparison
      const networkStats = await this.miningService.getNetworkStats();
      const userRank = await this.miningService.getUserRank(user.id);

      socket.emit('mining:calculation_result', {
        current: currentRewards,
        projections,
        network: {
          averageRate: networkStats.averageRate,
          userRank,
          percentile: await this.miningService.getUserPercentile(user.id)
        },
        optimization: await this.generateOptimizationSuggestions(user, session)
      });

    } catch (error) {
      this.logger.error('Error calculating real-time rewards:', error);
      socket.emit('mining:error', { 
        code: 'CALCULATION_FAILED',
        message: 'Failed to calculate rewards' 
      });
    }
  }

  private async claimRewards(
    socket: Socket, 
    user: User, 
    data: ClaimRewardsRequest
  ): Promise<void> {
    try {
      // Validate claim eligibility
      const claimable = await this.miningService.getClaimableRewards(user.id);
      if (claimable.amount === 0) {
        socket.emit('mining:error', { 
          code: 'NO_REWARDS',
          message: 'No rewards available to claim' 
        });
        return;
      }

      // Anti-bot validation for claiming
      const humanScore = await this.antiBotService.validateUserActivity(user.id);
      if (humanScore < 0.8) {
        socket.emit('mining:error', { 
          code: 'CLAIM_BLOCKED',
          message: 'Claim blocked due to suspicious activity' 
        });
        return;
      }

      // Process blockchain transaction
      const claimTx = await this.blockchainService.claimRewards(
        user.walletAddress,
        claimable.amount
      );

      // Update user balance
      await this.miningService.processRewardClaim(user.id, {
        amount: claimable.amount,
        transactionHash: claimTx.signature,
        timestamp: new Date()
      });

      // Send success response
      socket.emit('mining:rewards_claimed', {
        amount: claimable.amount,
        transactionHash: claimTx.signature,
        newBalance: await this.miningService.getUserBalance(user.id),
        timestamp: new Date()
      });

      // Update referral network
      await this.referralService.processReferralReward(user.id, claimable.amount);

      // Analytics
      this.analyticsService.trackEvent('rewards_claimed', {
        userId: user.id,
        amount: claimable.amount,
        transactionHash: claimTx.signature,
        timestamp: new Date()
      });

      this.logger.info(`Rewards claimed by user ${user.id}: ${claimable.amount} FIN`);

    } catch (error) {
      this.logger.error('Error claiming rewards:', error);
      socket.emit('mining:error', { 
        code: 'CLAIM_FAILED',
        message: 'Failed to claim rewards' 
      });
    }
  }

  private async generateOptimizationSuggestions(
    user: User, 
    session: MiningSession
  ): Promise<any[]> {
    const suggestions = [];

    // XP level suggestion
    if (user.xpLevel < 25) {
      suggestions.push({
        type: 'xp_boost',
        title: 'Increase Your XP Level',
        description: 'Post more quality content to boost your mining rate',
        impact: '+20-50% mining rate',
        action: 'create_content'
      });
    }

    // Staking suggestion
    const stakingData = await this.miningService.getUserStakingData(user.id);
    if (!stakingData || stakingData.stakedAmount < 100) {
      suggestions.push({
        type: 'staking',
        title: 'Start Staking FIN Tokens',
        description: 'Stake your FIN tokens for up to 100% mining bonus',
        impact: '+20-100% mining rate',
        action: 'stake_tokens'
      });
    }

    // Referral network suggestion
    const referralData = await this.referralService.getUserReferralData(user.id);
    if (referralData.activeReferrals < 10) {
      suggestions.push({
        type: 'referrals',
        title: 'Grow Your Network',
        description: 'Invite friends to increase your referral bonus',
        impact: '+10% per active referral',
        action: 'invite_friends'
      });
    }

    // Special card suggestion
    const availableBoosts = await this.miningService.getAvailableBoosts(user.id);
    if (availableBoosts.length > 0) {
      suggestions.push({
        type: 'boost_cards',
        title: 'Use Special Cards',
        description: 'Activate mining boost cards for temporary rate increases',
        impact: '+100-500% for limited time',
        action: 'use_boost_card'
      });
    }

    return suggestions;
  }

  /**
   * Handle emergency mining session cleanup
   */
  public async emergencyCleanup(): Promise<void> {
    try {
      this.logger.warn('Starting emergency mining session cleanup');

      for (const [userId, session] of this.activeSessions.entries()) {
        try {
          // Save current session state
          const user = await User.findById(userId);
          if (user) {
            const rewards = await this.calculateSessionRewards(session, user);
            await this.miningService.emergencySaveSession(session, rewards);
          }

          // Clear from memory
          this.activeSessions.delete(userId);

          // Clear from Redis
          await this.redis.del(`mining_session:${userId}`);

        } catch (error) {
          this.logger.error(`Error cleaning up session for user ${userId}:`, error);
        }
      }

      // Clear socket connections
      this.userSockets.clear();

      this.logger.info('Emergency cleanup completed');

    } catch (error) {
      this.logger.error('Error during emergency cleanup:', error);
    }
  }

  /**
   * Get comprehensive mining statistics
   */
  public async getMiningStatistics(): Promise<any> {
    try {
      const totalSessions = this.activeSessions.size;
      const totalUsers = await this.miningService.getTotalUsers();
      const networkStats = await this.miningService.getNetworkStats();

      return {
        activeSessions: totalSessions,
        totalUsers,
        activeMinersRatio: totalSessions / totalUsers,
        currentPhase: this.getCurrentMiningPhase(),
        networkAverageRate: networkStats.averageRate,
        totalFINMined: networkStats.totalMined,
        topMiners: await this.miningService.getTopMiners(10),
        recentActivities: await this.miningService.getRecentActivities(20)
      };

    } catch (error) {
      this.logger.error('Error getting mining statistics:', error);
      throw error;
    }
  }

  /**
   * Advanced anti-bot monitoring during active sessions
   */
  private async monitorSessionBehavior(userId: string, session: MiningSession): Promise<void> {
    try {
      // Analyze session patterns
      const behaviorScore = await this.antiBotService.analyzeSessionBehavior(userId, {
        sessionDuration: Date.now() - session.startTime.getTime(),
        activityCount: session.activities.length,
        activityPattern: session.activities.map(a => ({
          type: a.type,
          timestamp: a.timestamp,
          qualityScore: a.qualityScore
        })),
        boostUsage: session.boosts.length
      });

      // Take action if suspicious behavior detected
      if (behaviorScore < 0.3) {
        // Temporarily reduce mining rate
        session.currentRate *= 0.5;
        
        // Notify user
        const socket = this.userSockets.get(userId);
        if (socket) {
          socket.emit('mining:warning', {
            type: 'suspicious_behavior',
            message: 'Unusual activity detected. Mining rate temporarily reduced.',
            behaviorScore,
            action: 'rate_reduced'
          });
        }

        // Log for review
        this.logger.warn(`Suspicious behavior detected for user ${userId}. Score: ${behaviorScore}`);
        
        // Analytics
        this.analyticsService.trackEvent('suspicious_mining_behavior', {
          userId,
          behaviorScore,
          sessionData: session,
          timestamp: new Date()
        });
      }

    } catch (error) {
      this.logger.error('Error monitoring session behavior:', error);
    }
  }

  /**
   * Graceful shutdown procedure
   */
  public async shutdown(): Promise<void> {
    try {
      this.logger.info('Starting graceful shutdown of mining handler');

      // Save all active sessions
      const savePromises = Array.from(this.activeSessions.entries()).map(
        async ([userId, session]) => {
          try {
            const user = await User.findById(userId);
            if (user) {
              const rewards = await this.calculateSessionRewards(session, user);
              await this.miningService.saveMiningSession(session, rewards);
            }
          } catch (error) {
            this.logger.error(`Error saving session for user ${userId}:`, error);
          }
        }
      );

      await Promise.all(savePromises);

      // Clear all data structures
      this.activeSessions.clear();
      this.userSockets.clear();

      this.logger.info('Mining handler shutdown completed successfully');

    } catch (error) {
      this.logger.error('Error during mining handler shutdown:', error);
      throw error;
    }
  }
}
