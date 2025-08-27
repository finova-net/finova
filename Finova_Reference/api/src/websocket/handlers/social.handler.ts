import { Socket } from 'socket.io';
import { Logger } from 'winston';
import { RedisClient } from 'redis';
import { PrismaClient } from '@prisma/client';
import { PublicKey } from '@solana/web3.js';
import { 
  SocialActivityPayload,
  XPCalculationResult,
  RPUpdateResult,
  QualityAssessmentResult,
  SocialEventType,
  PlatformType,
  ActivityType,
  UserSocketData,
  RealTimeStats,
  ViralThreshold,
  NetworkEffect
} from '../types/social.types';
import { 
  AIQualityService,
  AntiBotService,
  XPService,
  ReferralService,
  MiningService,
  NotificationService,
  AnalyticsService
} from '../services';
import { validateSocialActivity, sanitizeInput } from '../utils/validation';
import { SOCIAL_CONSTANTS, XP_CONSTANTS, RP_CONSTANTS } from '../constants';

export class SocialHandler {
  private logger: Logger;
  private redis: RedisClient;
  private prisma: PrismaClient;
  private aiQuality: AIQualityService;
  private antiBot: AntiBotService;
  private xpService: XPService;
  private referralService: ReferralService;
  private miningService: MiningService;
  private notificationService: NotificationService;
  private analyticsService: AnalyticsService;
  
  // Real-time tracking
  private activeUsers = new Map<string, UserSocketData>();
  private activityBuffer = new Map<string, SocialActivityPayload[]>();
  private viralContent = new Map<string, ViralThreshold>();
  
  constructor(
    logger: Logger,
    redis: RedisClient,
    prisma: PrismaClient,
    services: {
      aiQuality: AIQualityService;
      antiBot: AntiBotService;
      xpService: XPService;
      referralService: ReferralService;
      miningService: MiningService;
      notificationService: NotificationService;
      analyticsService: AnalyticsService;
    }
  ) {
    this.logger = logger;
    this.redis = redis;
    this.prisma = prisma;
    Object.assign(this, services);
    
    // Initialize periodic tasks
    this.initializePeriodicTasks();
  }

  /**
   * Handle new socket connection
   */
  async handleConnection(socket: Socket): Promise<void> {
    try {
      const { userId, walletAddress, token } = socket.handshake.auth;
      
      // Validate authentication
      const user = await this.validateUser(userId, token);
      if (!user) {
        socket.emit('auth_error', { message: 'Invalid authentication' });
        socket.disconnect();
        return;
      }

      // Anti-bot verification
      const humanProbability = await this.antiBot.calculateHumanProbability({
        userId,
        socketId: socket.id,
        connectionTime: new Date(),
        ipAddress: socket.handshake.address,
        userAgent: socket.handshake.headers['user-agent']
      });

      if (humanProbability < SOCIAL_CONSTANTS.MIN_HUMAN_PROBABILITY) {
        this.logger.warn(`Suspicious connection rejected: ${userId}`, { humanProbability });
        socket.emit('security_warning', { message: 'Connection flagged for review' });
        socket.disconnect();
        return;
      }

      // Register user session
      const userData: UserSocketData = {
        userId,
        walletAddress: new PublicKey(walletAddress),
        socketId: socket.id,
        connectedAt: new Date(),
        lastActivity: new Date(),
        currentLevel: user.xpLevel,
        rpTier: user.rpTier,
        miningRate: user.currentMiningRate,
        humanProbability,
        activityCount: 0,
        qualityScore: user.averageQualityScore || 1.0
      };

      this.activeUsers.set(socket.id, userData);
      await this.redis.setex(`user_session:${userId}`, 3600, JSON.stringify(userData));

      // Join user-specific room
      await socket.join(`user:${userId}`);
      await socket.join(`level:${user.xpLevel}`);
      await socket.join(`tier:${user.rpTier}`);

      // Send initial data
      socket.emit('connection_success', {
        userData: {
          currentXP: user.totalXP,
          currentLevel: user.xpLevel,
          rpTier: user.rpTier,
          miningRate: user.currentMiningRate,
          dailyLimit: user.dailyMiningLimit,
          streakDays: user.streakDays
        },
        realTimeStats: await this.getRealTimeStats()
      });

      this.setupSocketListeners(socket);
      this.logger.info(`User connected: ${userId} (${socket.id})`);

    } catch (error) {
      this.logger.error('Connection error:', error);
      socket.emit('connection_error', { message: 'Failed to establish connection' });
      socket.disconnect();
    }
  }

  /**
   * Setup socket event listeners
   */
  private setupSocketListeners(socket: Socket): void {
    // Social activity submission
    socket.on('social_activity', (data) => this.handleSocialActivity(socket, data));
    
    // Real-time XP updates
    socket.on('request_xp_update', () => this.sendXPUpdate(socket));
    
    // RP network updates
    socket.on('request_rp_update', () => this.sendRPUpdate(socket));
    
    // Quality content submission
    socket.on('submit_content', (data) => this.handleContentSubmission(socket, data));
    
    // Viral content notifications
    socket.on('viral_achievement', (data) => this.handleViralAchievement(socket, data));
    
    // Guild activities
    socket.on('guild_activity', (data) => this.handleGuildActivity(socket, data));
    
    // Real-time challenges
    socket.on('join_challenge', (data) => this.handleChallengeJoin(socket, data));
    
    // Disconnect handling
    socket.on('disconnect', () => this.handleDisconnection(socket));
    
    // Heartbeat for activity tracking
    socket.on('heartbeat', () => this.updateLastActivity(socket));
  }

  /**
   * Handle social activity submission
   */
  async handleSocialActivity(socket: Socket, payload: SocialActivityPayload): Promise<void> {
    try {
      const userData = this.activeUsers.get(socket.id);
      if (!userData) {
        socket.emit('error', { message: 'User session not found' });
        return;
      }

      // Validate payload
      const validation = validateSocialActivity(payload);
      if (!validation.isValid) {
        socket.emit('validation_error', { errors: validation.errors });
        return;
      }

      // Sanitize input
      const sanitizedPayload = sanitizeInput(payload);

      // Rate limiting check
      const rateLimitKey = `rate_limit:${userData.userId}:${payload.activityType}`;
      const currentCount = await this.redis.incr(rateLimitKey);
      
      if (currentCount === 1) {
        await this.redis.expire(rateLimitKey, 3600); // 1 hour window
      }

      const limit = this.getActivityRateLimit(payload.activityType);
      if (currentCount > limit) {
        socket.emit('rate_limit_exceeded', { 
          activityType: payload.activityType,
          limit,
          resetTime: 3600
        });
        return;
      }

      // Anti-bot verification for this activity
      const activityVerification = await this.antiBot.verifyActivity({
        userId: userData.userId,
        activityType: payload.activityType,
        timestamp: new Date(),
        metadata: payload.metadata
      });

      if (!activityVerification.isHuman) {
        this.logger.warn(`Suspicious activity detected: ${userData.userId}`, {
          activityType: payload.activityType,
          suspicionLevel: activityVerification.suspicionLevel
        });
        
        socket.emit('activity_flagged', {
          message: 'Activity flagged for review',
          suspicionLevel: activityVerification.suspicionLevel
        });
        return;
      }

      // Quality assessment
      const qualityResult = await this.assessContentQuality(sanitizedPayload);
      
      // Calculate XP reward
      const xpResult = await this.calculateXPReward(userData, sanitizedPayload, qualityResult);
      
      // Calculate RP impact
      const rpResult = await this.calculateRPImpact(userData, sanitizedPayload, qualityResult);
      
      // Update mining rate if applicable
      const miningUpdate = await this.updateMiningRate(userData, xpResult, rpResult);

      // Process the activity
      const processedActivity = await this.processActivity({
        userData,
        payload: sanitizedPayload,
        qualityResult,
        xpResult,
        rpResult,
        miningUpdate
      });

      // Update user data
      await this.updateUserProgress(userData, processedActivity);

      // Check for achievements and milestones
      const achievements = await this.checkAchievements(userData, processedActivity);

      // Send real-time updates
      socket.emit('activity_processed', {
        activity: processedActivity,
        xpGained: xpResult.xpGained,
        newLevel: xpResult.newLevel,
        rpGained: rpResult.rpGained,
        newTier: rpResult.newTier,
        miningBoost: miningUpdate.newRate,
        qualityScore: qualityResult.score,
        achievements
      });

      // Broadcast to referral network if significant
      if (processedActivity.isSignificant) {
        await this.broadcastToNetwork(userData, processedActivity);
      }

      // Update activity buffer for batch processing
      this.addToActivityBuffer(userData.userId, sanitizedPayload);

      // Analytics tracking
      await this.analyticsService.trackSocialActivity({
        userId: userData.userId,
        activityType: payload.activityType,
        platform: payload.platform,
        xpGained: xpResult.xpGained,
        qualityScore: qualityResult.score,
        timestamp: new Date()
      });

      this.logger.info(`Activity processed: ${userData.userId}`, {
        activityType: payload.activityType,
        xpGained: xpResult.xpGained,
        qualityScore: qualityResult.score
      });

    } catch (error) {
      this.logger.error('Social activity processing error:', error);
      socket.emit('processing_error', { 
        message: 'Failed to process activity',
        activityId: payload.id 
      });
    }
  }

  /**
   * Assess content quality using AI
   */
  private async assessContentQuality(payload: SocialActivityPayload): Promise<QualityAssessmentResult> {
    try {
      const assessment = await this.aiQuality.analyzeContent({
        content: payload.content,
        platform: payload.platform,
        activityType: payload.activityType,
        metadata: payload.metadata
      });

      const qualityScore = Math.max(0.5, Math.min(2.0, assessment.score));

      return {
        score: qualityScore,
        factors: {
          originality: assessment.originality,
          engagement: assessment.engagementPotential,
          relevance: assessment.platformRelevance,
          safety: assessment.brandSafety,
          human: assessment.humanGenerated
        },
        feedback: assessment.feedback,
        recommendations: assessment.recommendations
      };

    } catch (error) {
      this.logger.error('Quality assessment error:', error);
      return {
        score: 1.0, // Default neutral score
        factors: {},
        feedback: 'Assessment unavailable',
        recommendations: []
      };
    }
  }

  /**
   * Calculate XP reward with integrated formula
   */
  private async calculateXPReward(
    userData: UserSocketData,
    payload: SocialActivityPayload,
    qualityResult: QualityAssessmentResult
  ): Promise<XPCalculationResult> {
    
    const baseXP = this.getBaseXP(payload.activityType);
    const platformMultiplier = this.getPlatformMultiplier(payload.platform);
    const qualityScore = qualityResult.score;
    
    // Get current user data
    const user = await this.prisma.user.findUnique({
      where: { id: userData.userId },
      include: { xpHistory: true }
    });

    const streakBonus = this.calculateStreakBonus(user.streakDays);
    const levelProgression = Math.exp(-0.01 * user.xpLevel);

    // Master XP Formula from whitepaper
    const xpGained = Math.floor(
      baseXP * platformMultiplier * qualityScore * streakBonus * levelProgression
    );

    const newTotalXP = user.totalXP + xpGained;
    const newLevel = this.calculateLevel(newTotalXP);
    const leveledUp = newLevel > user.xpLevel;

    return {
      xpGained,
      newTotalXP,
      newLevel,
      leveledUp,
      multipliers: {
        platform: platformMultiplier,
        quality: qualityScore,
        streak: streakBonus,
        progression: levelProgression
      }
    };
  }

  /**
   * Calculate RP impact on referral network
   */
  private async calculateRPImpact(
    userData: UserSocketData,
    payload: SocialActivityPayload,
    qualityResult: QualityAssessmentResult
  ): Promise<RPUpdateResult> {
    
    const referralNetwork = await this.referralService.getUserNetwork(userData.userId);
    
    if (!referralNetwork || referralNetwork.directReferrals.length === 0) {
      return {
        rpGained: 0,
        newTier: userData.rpTier,
        networkImpact: 0,
        tierUpgraded: false
      };
    }

    // Calculate RP based on activity value
    const activityValue = this.calculateActivityValue(payload, qualityResult);
    const rpGained = Math.floor(activityValue * 0.05); // 5% to RP

    // Update referral network bonuses
    const networkBonus = await this.referralService.updateNetworkBonus({
      userId: userData.userId,
      activityValue,
      qualityScore: qualityResult.score,
      activityType: payload.activityType
    });

    const newTotalRP = referralNetwork.totalRP + rpGained;
    const newTier = this.calculateRPTier(newTotalRP);
    const tierUpgraded = newTier > userData.rpTier;

    return {
      rpGained,
      newTier,
      networkImpact: networkBonus.totalImpact,
      tierUpgraded,
      affectedReferrals: networkBonus.affectedUsers
    };
  }

  /**
   * Update mining rate based on XP/RP changes
   */
  private async updateMiningRate(
    userData: UserSocketData,
    xpResult: XPCalculationResult,
    rpResult: RPUpdateResult
  ): Promise<{ newRate: number; boost: number }> {
    
    const currentMiningRate = await this.miningService.getCurrentMiningRate(userData.userId);
    
    // Calculate new multipliers
    const xpMultiplier = this.getXPMiningMultiplier(xpResult.newLevel);
    const rpMultiplier = this.getRPMiningMultiplier(rpResult.newTier);
    
    const newRate = await this.miningService.updateUserMiningRate({
      userId: userData.userId,
      xpLevel: xpResult.newLevel,
      rpTier: rpResult.newTier,
      activityBoost: true
    });

    const boost = newRate / currentMiningRate;

    return { newRate, boost };
  }

  /**
   * Handle viral content achievement
   */
  async handleViralAchievement(socket: Socket, data: { contentId: string; views: number; platform: PlatformType }): Promise<void> {
    try {
      const userData = this.activeUsers.get(socket.id);
      if (!userData) return;

      const { contentId, views, platform } = data;

      // Verify viral threshold
      const threshold = SOCIAL_CONSTANTS.VIRAL_THRESHOLDS[platform];
      if (views < threshold) {
        socket.emit('viral_verification_failed', { required: threshold, actual: views });
        return;
      }

      // Calculate viral bonus
      const viralMultiplier = Math.min(5.0, 1.0 + Math.log10(views / threshold));
      const bonusXP = Math.floor(XP_CONSTANTS.VIRAL_BASE_XP * viralMultiplier);
      const bonusFIN = XP_CONSTANTS.VIRAL_BASE_FIN * viralMultiplier;

      // Apply rewards
      await this.xpService.addXP({
        userId: userData.userId,
        amount: bonusXP,
        source: 'viral_content',
        metadata: { contentId, views, platform }
      });

      await this.miningService.addBonus({
        userId: userData.userId,
        amount: bonusFIN,
        duration: 24 * 60 * 60 * 1000, // 24 hours
        source: 'viral_achievement'
      });

      // Broadcast achievement
      socket.broadcast.emit('viral_achievement_notification', {
        userId: userData.userId,
        contentId,
        views,
        platform,
        bonusXP,
        bonusFIN
      });

      // Update RP network
      await this.referralService.distributeViralBonus({
        userId: userData.userId,
        bonusAmount: bonusFIN * 0.1, // 10% to referral network
        contentId
      });

      socket.emit('viral_achievement_processed', {
        bonusXP,
        bonusFIN,
        multiplier: viralMultiplier,
        networkBonus: bonusFIN * 0.1
      });

      this.logger.info(`Viral achievement processed: ${userData.userId}`, {
        contentId,
        views,
        bonusXP,
        bonusFIN
      });

    } catch (error) {
      this.logger.error('Viral achievement error:', error);
      socket.emit('viral_processing_error', { message: 'Failed to process viral achievement' });
    }
  }

  /**
   * Handle guild activity
   */
  async handleGuildActivity(socket: Socket, data: { guildId: string; activityType: string; contribution: number }): Promise<void> {
    try {
      const userData = this.activeUsers.get(socket.id);
      if (!userData) return;

      const { guildId, activityType, contribution } = data;

      // Verify guild membership
      const guildMember = await this.prisma.guildMember.findFirst({
        where: {
          userId: userData.userId,
          guildId,
          status: 'ACTIVE'
        },
        include: { guild: true }
      });

      if (!guildMember) {
        socket.emit('guild_error', { message: 'Not a member of this guild' });
        return;
      }

      // Calculate guild bonus
      const guildBonus = SOCIAL_CONSTANTS.GUILD_BONUSES[activityType] || 1.0;
      const bonusXP = Math.floor(contribution * guildBonus);

      // Apply guild XP bonus
      await this.xpService.addXP({
        userId: userData.userId,
        amount: bonusXP,
        source: 'guild_activity',
        metadata: { guildId, activityType, contribution }
      });

      // Update guild statistics
      await this.prisma.guild.update({
        where: { id: guildId },
        data: {
          totalXP: { increment: bonusXP },
          activityCount: { increment: 1 },
          lastActivity: new Date()
        }
      });

      // Broadcast to guild members
      socket.to(`guild:${guildId}`).emit('guild_activity_update', {
        userId: userData.userId,
        activityType,
        contribution: bonusXP,
        guildTotal: guildMember.guild.totalXP + bonusXP
      });

      socket.emit('guild_activity_processed', {
        bonusXP,
        guildBonus,
        newGuildTotal: guildMember.guild.totalXP + bonusXP
      });

    } catch (error) {
      this.logger.error('Guild activity error:', error);
      socket.emit('guild_processing_error', { message: 'Failed to process guild activity' });
    }
  }

  /**
   * Get real-time network statistics
   */
  private async getRealTimeStats(): Promise<RealTimeStats> {
    const [
      activeUsers,
      totalMining,
      totalXP,
      viralContent,
      networkGrowth
    ] = await Promise.all([
      this.redis.get('stats:active_users'),
      this.redis.get('stats:total_mining_today'),
      this.redis.get('stats:total_xp_today'),
      this.redis.get('stats:viral_content_today'),
      this.redis.get('stats:network_growth_today')
    ]);

    return {
      activeUsers: parseInt(activeUsers || '0'),
      totalMiningToday: parseFloat(totalMining || '0'),
      totalXPToday: parseInt(totalXP || '0'),
      viralContentToday: parseInt(viralContent || '0'),
      networkGrowthToday: parseInt(networkGrowth || '0'),
      currentPhase: await this.miningService.getCurrentPhase(),
      globalMiningRate: await this.miningService.getGlobalMiningRate()
    };
  }

  /**
   * Broadcast significant activities to referral network
   */
  private async broadcastToNetwork(userData: UserSocketData, activity: any): Promise<void> {
    const network = await this.referralService.getUserNetwork(userData.userId);
    
    if (!network) return;

    const notification = {
      type: 'network_activity',
      fromUser: userData.userId,
      activity: {
        type: activity.type,
        xpGained: activity.xpGained,
        qualityScore: activity.qualityScore,
        platform: activity.platform
      },
      networkBonus: activity.networkBonus,
      timestamp: new Date()
    };

    // Send to direct referrals
    for (const referral of network.directReferrals) {
      const referralSockets = await this.redis.smembers(`user_sockets:${referral.id}`);
      for (const socketId of referralSockets) {
        if (this.activeUsers.has(socketId)) {
          const socket = this.getSocketById(socketId);
          socket?.emit('network_activity', notification);
        }
      }
    }
  }

  /**
   * Handle user disconnection
   */
  async handleDisconnection(socket: Socket): Promise<void> {
    try {
      const userData = this.activeUsers.get(socket.id);
      if (!userData) return;

      // Calculate session statistics
      const sessionDuration = Date.now() - userData.connectedAt.getTime();
      const activitiesCount = userData.activityCount;

      // Update user session stats
      await this.prisma.userSession.create({
        data: {
          userId: userData.userId,
          duration: sessionDuration,
          activitiesCount,
          qualityScore: userData.qualityScore,
          disconnectedAt: new Date()
        }
      });

      // Process any buffered activities
      await this.processPendingActivities(userData.userId);

      // Clean up
      this.activeUsers.delete(socket.id);
      await this.redis.del(`user_session:${userData.userId}`);
      
      this.logger.info(`User disconnected: ${userData.userId}`, {
        sessionDuration,
        activitiesCount
      });

    } catch (error) {
      this.logger.error('Disconnection handling error:', error);
    }
  }

  /**
   * Utility methods
   */
  private getBaseXP(activityType: ActivityType): number {
    return XP_CONSTANTS.BASE_XP[activityType] || 10;
  }

  private getPlatformMultiplier(platform: PlatformType): number {
    return XP_CONSTANTS.PLATFORM_MULTIPLIERS[platform] || 1.0;
  }

  private calculateStreakBonus(streakDays: number): number {
    return Math.min(3.0, 1.0 + (streakDays * 0.1));
  }

  private calculateLevel(totalXP: number): number {
    return Math.floor(Math.sqrt(totalXP / 100)) + 1;
  }

  private calculateRPTier(totalRP: number): number {
    if (totalRP >= 50000) return 5; // Ambassador
    if (totalRP >= 15000) return 4; // Leader
    if (totalRP >= 5000) return 3;  // Influencer
    if (totalRP >= 1000) return 2;  // Connector
    return 1; // Explorer
  }

  private getXPMiningMultiplier(level: number): number {
    if (level >= 101) return 5.0;
    if (level >= 76) return 4.0;
    if (level >= 51) return 3.0;
    if (level >= 26) return 2.0;
    if (level >= 11) return 1.5;
    return 1.0;
  }

  private getRPMiningMultiplier(tier: number): number {
    return 1.0 + (tier * 0.5);
  }

  private getActivityRateLimit(activityType: ActivityType): number {
    return SOCIAL_CONSTANTS.RATE_LIMITS[activityType] || 100;
  }

  private calculateActivityValue(payload: SocialActivityPayload, quality: QualityAssessmentResult): number {
    const baseValue = this.getBaseXP(payload.activityType);
    return baseValue * quality.score;
  }

  private addToActivityBuffer(userId: string, activity: SocialActivityPayload): void {
    if (!this.activityBuffer.has(userId)) {
      this.activityBuffer.set(userId, []);
    }
    this.activityBuffer.get(userId)!.push(activity);
  }

  private async processPendingActivities(userId: string): Promise<void> {
    const activities = this.activityBuffer.get(userId);
    if (!activities || activities.length === 0) return;

    // Batch process activities
    await this.prisma.socialActivity.createMany({
      data: activities.map(activity => ({
        userId,
        activityType: activity.activityType,
        platform: activity.platform,
        content: activity.content,
        metadata: activity.metadata,
        timestamp: new Date()
      }))
    });

    this.activityBuffer.delete(userId);
  }

  private async validateUser(userId: string, token: string): Promise<any> {
    // Implement JWT validation and user lookup
    return await this.prisma.user.findUnique({ where: { id: userId } });
  }

  private getSocketById(socketId: string): Socket | undefined {
    // Implementation would depend on your Socket.IO setup
    return undefined;
  }

  private updateLastActivity(socket: Socket): void {
    const userData = this.activeUsers.get(socket.id);
    if (userData) {
      userData.lastActivity = new Date();
      userData.activityCount++;
    }
  }

  private async checkAchievements(userData: UserSocketData, activity: any): Promise<any[]> {
    // Implementation for achievement checking
    return [];
  }

  private async processActivity(params: any): Promise<any> {
    // Main activity processing logic
    return {
      id: params.payload.id,
      type: params.payload.activityType,
      xpGained: params.xpResult.xpGained,
      qualityScore: params.qualityResult.score,
      platform: params.payload.platform,
      isSignificant: params.xpResult.xpGained > 100,
      networkBonus: params.rpResult.rpGained
    };
  }

  private async updateUserProgress(userData: UserSocketData, activity: any): Promise<void> {
    await this.prisma.user.update({
      where: { id: userData.userId },
      data: {
        totalXP: { increment: activity.xpGained },
        lastActivity: new Date(),
        activityCount: { increment: 1 }
      }
    });
  }

  /**
   * Initialize periodic tasks
   */
  private initializePeriodicTasks(): void {
    // Update real-time stats every 30 seconds
    setInterval(async () => {
      try {
        const stats = await this.getRealTimeStats();
        // Broadcast to all connected users
        for (const [socketId, userData] of this.activeUsers) {
          const socket = this.getSocketById(socketId);
          socket?.emit('stats_update', stats);
        }
      } catch (error) {
        this.logger.error('Stats update error:', error);
      }
    }, 30000);

    // Process activity buffers every 5 minutes
    setInterval(async () => {
      for (const [userId] of this.activityBuffer) {
        await this.processPendingActivities(userId);
      }
    }, 300000);

    // Clean up inactive sessions every 10 minutes
    setInterval(async () => {
      const now = Date.now();
      for (const [socketId, userData] of this.activeUsers) {
        if (now - userData.lastActivity.getTime() > 600000) { // 10 minutes
          this.activeUsers.delete(socketId);
        }
      }
    }, 600000);
  }
}
