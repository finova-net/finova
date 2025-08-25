import { Socket } from 'socket.io';
import { Redis } from 'ioredis';
import { Logger } from 'winston';
import { 
  XPActivity, 
  XPCalculationResult, 
  UserXPState, 
  XPMultiplier,
  LevelUpEvent,
  XPBadge,
  QualityScore,
  StreakBonus,
  PlatformMultiplier 
} from '../../types/xp.types';
import { UserService } from '../../services/user.service';
import { XPService } from '../../services/xp.service';
import { MiningService } from '../../services/mining.service';
import { ReferralService } from '../../services/referral.service';
import { AIQualityService } from '../../services/ai-quality.service';
import { NotificationService } from '../../services/notification.service';
import { AnalyticsService } from '../../services/analytics.service';
import { rateLimitCheck, validateAuth } from '../middleware/auth.ws';

interface XPHandlerDependencies {
  redis: Redis;
  logger: Logger;
  userService: UserService;
  xpService: XPService;
  miningService: MiningService;
  referralService: ReferralService;
  aiQualityService: AIQualityService;
  notificationService: NotificationService;
  analyticsService: AnalyticsService;
}

interface XPEventData {
  activityType: string;
  platform: string;
  content?: {
    text?: string;
    mediaUrl?: string;
    mediaType?: 'image' | 'video' | 'audio';
    engagement?: {
      views?: number;
      likes?: number;
      comments?: number;
      shares?: number;
    };
  };
  metadata?: Record<string, any>;
  timestamp: number;
}

interface XPLevelMultiplier {
  level: number;
  miningBonus: number;
  dailyCapIncrease: number;
  specialUnlocks: string[];
}

interface XPStreakData {
  currentStreak: number;
  longestStreak: number;
  lastActivityDate: string;
  streakMultiplier: number;
}

export class XPHandler {
  private redis: Redis;
  private logger: Logger;
  private userService: UserService;
  private xpService: XPService;
  private miningService: MiningService;
  private referralService: ReferralService;
  private aiQualityService: AIQualityService;
  private notificationService: NotificationService;
  private analyticsService: AnalyticsService;

  // XP Constants based on whitepaper
  private readonly XP_CONFIG = {
    ACTIVITIES: {
      'original_post': { baseXP: 50, dailyLimit: null, qualityWeight: 1.5 },
      'photo_post': { baseXP: 75, dailyLimit: 20, qualityWeight: 1.8 },
      'video_post': { baseXP: 150, dailyLimit: 10, qualityWeight: 2.0 },
      'story_status': { baseXP: 25, dailyLimit: 50, qualityWeight: 1.2 },
      'meaningful_comment': { baseXP: 25, dailyLimit: 100, qualityWeight: 1.5 },
      'like_react': { baseXP: 5, dailyLimit: 200, qualityWeight: 1.0 },
      'share_repost': { baseXP: 15, dailyLimit: 50, qualityWeight: 1.3 },
      'follow_subscribe': { baseXP: 20, dailyLimit: 25, qualityWeight: 1.0 },
      'daily_login': { baseXP: 10, dailyLimit: 1, qualityWeight: 1.0 },
      'daily_quest': { baseXP: 100, dailyLimit: 3, qualityWeight: 1.0 },
      'milestone_achievement': { baseXP: 500, dailyLimit: null, qualityWeight: 1.0 },
      'viral_content': { baseXP: 1000, dailyLimit: null, qualityWeight: 2.0 }
    },
    PLATFORMS: {
      'tiktok': 1.3,
      'instagram': 1.2,
      'youtube': 1.4,
      'facebook': 1.1,
      'twitter': 1.2,
      'app_native': 1.0
    },
    QUALITY_RANGES: {
      min: 0.5,
      max: 2.0
    },
    STREAK_MAX_MULTIPLIER: 3.0,
    LEVEL_PROGRESSION_DECAY: 0.01
  };

  private readonly LEVEL_TIERS = [
    { range: [1, 10], badge: 'Bronze', miningMultiplier: [1.0, 1.2], dailyCap: [0.5, 2.0] },
    { range: [11, 25], badge: 'Silver', miningMultiplier: [1.3, 1.8], dailyCap: [2.0, 4.0] },
    { range: [26, 50], badge: 'Gold', miningMultiplier: [1.9, 2.5], dailyCap: [4.0, 6.0] },
    { range: [51, 75], badge: 'Platinum', miningMultiplier: [2.6, 3.2], dailyCap: [6.0, 8.0] },
    { range: [76, 100], badge: 'Diamond', miningMultiplier: [3.3, 4.0], dailyCap: [8.0, 10.0] },
    { range: [101, 999], badge: 'Mythic', miningMultiplier: [4.1, 5.0], dailyCap: [10.0, 15.0] }
  ];

  constructor(dependencies: XPHandlerDependencies) {
    this.redis = dependencies.redis;
    this.logger = dependencies.logger;
    this.userService = dependencies.userService;
    this.xpService = dependencies.xpService;
    this.miningService = dependencies.miningService;
    this.referralService = dependencies.referralService;
    this.aiQualityService = dependencies.aiQualityService;
    this.notificationService = dependencies.notificationService;
    this.analyticsService = dependencies.analyticsService;
  }

  /**
   * Handle XP-related WebSocket events
   */
  async handleConnection(socket: Socket): Promise<void> {
    this.logger.info('XP Handler connected', { socketId: socket.id });

    // Register XP event handlers
    socket.on('xp:activity', this.handleXPActivity.bind(this, socket));
    socket.on('xp:get_state', this.handleGetXPState.bind(this, socket));
    socket.on('xp:get_leaderboard', this.handleGetLeaderboard.bind(this, socket));
    socket.on('xp:claim_milestone', this.handleClaimMilestone.bind(this, socket));
    socket.on('xp:use_boost_card', this.handleUseBoostCard.bind(this, socket));
    socket.on('xp:sync_external', this.handleSyncExternal.bind(this, socket));

    // Subscribe to user's XP updates
    const userId = socket.data?.userId;
    if (userId) {
      await this.subscribeToUserXP(socket, userId);
    }
  }

  /**
   * Handle XP activity reporting
   */
  private async handleXPActivity(socket: Socket, data: XPEventData): Promise<void> {
    try {
      // Validate authentication and rate limits
      const authResult = await validateAuth(socket);
      if (!authResult.valid) {
        socket.emit('xp:error', { error: 'Authentication required' });
        return;
      }

      const rateLimitResult = await rateLimitCheck(socket, 'xp_activity', 100, 3600);
      if (!rateLimitResult.allowed) {
        socket.emit('xp:error', { 
          error: 'Rate limit exceeded', 
          resetTime: rateLimitResult.resetTime 
        });
        return;
      }

      const userId = authResult.userId;
      
      // Calculate XP for the activity
      const xpResult = await this.calculateXPForActivity(userId, data);
      
      if (xpResult.awarded > 0) {
        // Update user XP state
        await this.updateUserXP(userId, xpResult);
        
        // Check for level ups
        const levelUpResult = await this.checkLevelUp(userId, xpResult.newTotal);
        
        // Update mining rates if level changed
        if (levelUpResult.leveledUp) {
          await this.updateMiningMultiplier(userId, levelUpResult.newLevel);
        }

        // Send success response
        socket.emit('xp:activity_result', {
          success: true,
          xpAwarded: xpResult.awarded,
          totalXP: xpResult.newTotal,
          currentLevel: levelUpResult.newLevel,
          levelUp: levelUpResult.leveledUp,
          multipliers: xpResult.multipliers,
          qualityScore: xpResult.qualityScore
        });

        // Broadcast level up to user's network if applicable
        if (levelUpResult.leveledUp) {
          await this.broadcastLevelUp(userId, levelUpResult);
        }

        // Track analytics
        await this.trackXPActivity(userId, data, xpResult);
      } else {
        socket.emit('xp:activity_result', {
          success: false,
          reason: xpResult.reason || 'No XP awarded',
          xpAwarded: 0
        });
      }

    } catch (error) {
      this.logger.error('XP Activity Error:', error);
      socket.emit('xp:error', { error: 'Failed to process XP activity' });
    }
  }

  /**
   * Calculate XP for a given activity
   */
  private async calculateXPForActivity(userId: string, activity: XPEventData): Promise<XPCalculationResult> {
    // Get user's current XP state
    const userState = await this.getUserXPState(userId);
    const activityConfig = this.XP_CONFIG.ACTIVITIES[activity.activityType];
    
    if (!activityConfig) {
      return { awarded: 0, newTotal: userState.totalXP, reason: 'Unknown activity type' };
    }

    // Check daily limits
    const dailyCount = await this.getDailyActivityCount(userId, activity.activityType);
    if (activityConfig.dailyLimit && dailyCount >= activityConfig.dailyLimit) {
      return { awarded: 0, newTotal: userState.totalXP, reason: 'Daily limit reached' };
    }

    // Calculate base XP
    let baseXP = activityConfig.baseXP;

    // Apply platform multiplier
    const platformMultiplier = this.XP_CONFIG.PLATFORMS[activity.platform] || 1.0;

    // Calculate quality score using AI
    const qualityScore = await this.calculateQualityScore(activity);

    // Calculate streak bonus
    const streakBonus = await this.calculateStreakBonus(userId);

    // Apply level progression decay
    const levelProgression = Math.exp(-this.XP_CONFIG.LEVEL_PROGRESSION_DECAY * userState.currentLevel);

    // Check for viral content bonus
    const viralMultiplier = this.calculateViralMultiplier(activity);

    // Final XP calculation
    const finalXP = Math.floor(
      baseXP * 
      platformMultiplier * 
      qualityScore * 
      streakBonus * 
      levelProgression * 
      viralMultiplier
    );

    // Apply any active boost cards
    const boostMultiplier = await this.getActiveBoostMultiplier(userId);
    const boostedXP = Math.floor(finalXP * boostMultiplier);

    return {
      awarded: boostedXP,
      newTotal: userState.totalXP + boostedXP,
      multipliers: {
        platform: platformMultiplier,
        quality: qualityScore,
        streak: streakBonus,
        levelProgression,
        viral: viralMultiplier,
        boost: boostMultiplier
      },
      qualityScore,
      breakdown: {
        base: baseXP,
        afterPlatform: Math.floor(baseXP * platformMultiplier),
        afterQuality: Math.floor(baseXP * platformMultiplier * qualityScore),
        final: boostedXP
      }
    };
  }

  /**
   * Calculate AI-powered quality score
   */
  private async calculateQualityScore(activity: XPEventData): Promise<number> {
    try {
      if (!activity.content || activity.activityType === 'like_react' || activity.activityType === 'daily_login') {
        return 1.0; // Default for activities without content analysis
      }

      const qualityAnalysis = await this.aiQualityService.analyzeContent({
        text: activity.content.text,
        mediaUrl: activity.content.mediaUrl,
        mediaType: activity.content.mediaType,
        engagement: activity.content.engagement,
        platform: activity.platform
      });

      // Quality score factors from whitepaper
      const factors = {
        originality: qualityAnalysis.originality || 0.8,
        engagementPotential: qualityAnalysis.engagement_potential || 0.7,
        platformRelevance: qualityAnalysis.platform_relevance || 0.8,
        brandSafety: qualityAnalysis.brand_safety || 1.0,
        humanGenerated: qualityAnalysis.human_generated || 0.9
      };

      const weights = {
        originality: 0.3,
        engagementPotential: 0.25,
        platformRelevance: 0.2,
        brandSafety: 0.15,
        humanGenerated: 0.1
      };

      const weightedScore = Object.keys(factors).reduce((sum, key) => {
        return sum + (factors[key] * weights[key]);
      }, 0);

      // Clamp between min and max quality ranges
      return Math.max(
        this.XP_CONFIG.QUALITY_RANGES.min,
        Math.min(this.XP_CONFIG.QUALITY_RANGES.max, weightedScore * 2)
      );

    } catch (error) {
      this.logger.warn('Quality score calculation failed, using default', error);
      return 1.0;
    }
  }

  /**
   * Calculate streak bonus
   */
  private async calculateStreakBonus(userId: string): Promise<number> {
    const streakData = await this.getStreakData(userId);
    const today = new Date().toISOString().split('T')[0];
    
    // Update streak
    if (streakData.lastActivityDate !== today) {
      const yesterday = new Date();
      yesterday.setDate(yesterday.getDate() - 1);
      const yesterdayStr = yesterday.toISOString().split('T')[0];
      
      if (streakData.lastActivityDate === yesterdayStr) {
        // Continue streak
        streakData.currentStreak += 1;
      } else {
        // Reset streak
        streakData.currentStreak = 1;
      }
      
      streakData.lastActivityDate = today;
      streakData.longestStreak = Math.max(streakData.longestStreak, streakData.currentStreak);
      
      await this.saveStreakData(userId, streakData);
    }

    // Calculate streak multiplier (1.0x to 3.0x based on streak)
    const streakMultiplier = Math.min(
      this.XP_CONFIG.STREAK_MAX_MULTIPLIER,
      1.0 + (streakData.currentStreak * 0.1)
    );

    return streakMultiplier;
  }

  /**
   * Calculate viral content multiplier
   */
  private calculateViralMultiplier(activity: XPEventData): number {
    if (!activity.content?.engagement) return 1.0;

    const { views = 0, likes = 0, comments = 0, shares = 0 } = activity.content.engagement;
    
    // Viral thresholds based on platform
    const viralThresholds = {
      'tiktok': { views: 10000, engagement: 1000 },
      'instagram': { views: 5000, engagement: 500 },
      'youtube': { views: 50000, engagement: 5000 },
      'facebook': { views: 10000, engagement: 1000 },
      'twitter': { views: 10000, engagement: 1000 },
      'app_native': { views: 1000, engagement: 100 }
    };

    const threshold = viralThresholds[activity.platform] || viralThresholds['app_native'];
    const totalEngagement = likes + comments + shares;

    if (views >= threshold.views || totalEngagement >= threshold.engagement) {
      // Viral content gets 2x multiplier as per whitepaper
      return 2.0;
    }

    return 1.0;
  }

  /**
   * Get active boost multiplier from NFT cards
   */
  private async getActiveBoostMultiplier(userId: string): Promise<number> {
    const activeBoosts = await this.redis.hgetall(`user:${userId}:active_boosts`);
    let multiplier = 1.0;

    for (const [cardType, expiry] of Object.entries(activeBoosts)) {
      if (parseInt(expiry) > Date.now()) {
        switch (cardType) {
          case 'xp_double': multiplier *= 2.0; break;
          case 'xp_magnet': multiplier *= 3.0; break;
          case 'level_rush': multiplier *= 1.5; break;
          default: break;
        }
      }
    }

    return multiplier;
  }

  /**
   * Update user XP state
   */
  private async updateUserXP(userId: string, xpResult: XPCalculationResult): Promise<void> {
    const pipeline = this.redis.pipeline();
    
    // Update total XP
    pipeline.hset(`user:${userId}:xp`, 'totalXP', xpResult.newTotal);
    pipeline.hset(`user:${userId}:xp`, 'lastActivity', Date.now());
    
    // Increment daily XP
    const today = new Date().toISOString().split('T')[0];
    pipeline.hincrby(`user:${userId}:xp:daily:${today}`, 'totalXP', xpResult.awarded);
    pipeline.expire(`user:${userId}:xp:daily:${today}`, 86400 * 7); // Keep for 7 days
    
    await pipeline.exec();
    
    // Update database
    await this.xpService.updateUserXP(userId, {
      totalXP: xpResult.newTotal,
      lastXPGain: xpResult.awarded,
      lastActivity: new Date()
    });
  }

  /**
   * Check for level up
   */
  private async checkLevelUp(userId: string, newTotalXP: number): Promise<{ leveledUp: boolean; newLevel: number; previousLevel: number; rewards?: any }> {
    const currentLevel = this.calculateLevelFromXP(newTotalXP);
    const userState = await this.getUserXPState(userId);
    const previousLevel = userState.currentLevel;

    if (currentLevel > previousLevel) {
      // Update level in cache and database
      await this.redis.hset(`user:${userId}:xp`, 'currentLevel', currentLevel);
      await this.xpService.updateUserLevel(userId, currentLevel);

      // Calculate level rewards
      const rewards = await this.calculateLevelRewards(currentLevel);
      
      // Apply level rewards
      if (rewards.finTokens > 0) {
        await this.miningService.addBonusTokens(userId, rewards.finTokens, 'level_up');
      }

      return {
        leveledUp: true,
        newLevel: currentLevel,
        previousLevel,
        rewards
      };
    }

    return {
      leveledUp: false,
      newLevel: currentLevel,
      previousLevel
    };
  }

  /**
   * Calculate level from total XP using exponential formula
   */
  private calculateLevelFromXP(totalXP: number): number {
    // XP requirements: Level 1=0, Level 2=100, Level 3=250, etc.
    // Formula: XP_required = level^2 * 50 - 50
    return Math.floor(Math.sqrt((totalXP + 50) / 50));
  }

  /**
   * Calculate XP required for next level
   */
  private calculateXPForLevel(level: number): number {
    return level * level * 50 - 50;
  }

  /**
   * Update mining multiplier based on new level
   */
  private async updateMiningMultiplier(userId: string, newLevel: number): Promise<void> {
    const tier = this.LEVEL_TIERS.find(t => newLevel >= t.range[0] && newLevel <= t.range[1]);
    
    if (tier) {
      // Calculate mining multiplier within tier range
      const tierProgress = (newLevel - tier.range[0]) / (tier.range[1] - tier.range[0]);
      const miningMultiplier = tier.miningMultiplier[0] + 
        (tier.miningMultiplier[1] - tier.miningMultiplier[0]) * tierProgress;
      
      // Update mining service
      await this.miningService.updateXPMultiplier(userId, miningMultiplier);
      
      this.logger.info('Updated mining multiplier for level up', {
        userId,
        newLevel,
        miningMultiplier,
        tier: tier.badge
      });
    }
  }

  /**
   * Get current user XP state
   */
  private async getUserXPState(userId: string): Promise<UserXPState> {
    const cached = await this.redis.hmget(
      `user:${userId}:xp`,
      'totalXP', 'currentLevel', 'lastActivity'
    );

    if (cached[0]) {
      return {
        totalXP: parseInt(cached[0]) || 0,
        currentLevel: parseInt(cached[1]) || 1,
        lastActivity: cached[2] ? new Date(parseInt(cached[2])) : new Date(),
        nextLevelXP: this.calculateXPForLevel(parseInt(cached[1]) + 1),
        currentLevelXP: this.calculateXPForLevel(parseInt(cached[1]))
      };
    }

    // Load from database if not cached
    const dbState = await this.xpService.getUserXPState(userId);
    
    // Cache for future use
    await this.redis.hmset(`user:${userId}:xp`, {
      totalXP: dbState.totalXP,
      currentLevel: dbState.currentLevel,
      lastActivity: dbState.lastActivity.getTime()
    });

    return dbState;
  }

  /**
   * Handle get XP state request
   */
  private async handleGetXPState(socket: Socket): Promise<void> {
    try {
      const authResult = await validateAuth(socket);
      if (!authResult.valid) {
        socket.emit('xp:error', { error: 'Authentication required' });
        return;
      }

      const userId = authResult.userId;
      const xpState = await this.getUserXPState(userId);
      const streakData = await this.getStreakData(userId);
      const activeBoosts = await this.getActiveBoosts(userId);
      const dailyStats = await this.getDailyXPStats(userId);

      socket.emit('xp:state', {
        ...xpState,
        streak: streakData,
        activeBoosts,
        dailyStats,
        tierInfo: this.getTierInfo(xpState.currentLevel),
        progressToNext: this.calculateLevelProgress(xpState.totalXP, xpState.currentLevel)
      });

    } catch (error) {
      this.logger.error('Get XP State Error:', error);
      socket.emit('xp:error', { error: 'Failed to get XP state' });
    }
  }

  /**
   * Broadcast level up to user's network
   */
  private async broadcastLevelUp(userId: string, levelUpResult: any): Promise<void> {
    // Get user's referral network
    const network = await this.referralService.getUserNetwork(userId);
    
    // Send notification to referrers about network member level up
    const levelUpNotification = {
      type: 'network_level_up',
      userId,
      newLevel: levelUpResult.newLevel,
      previousLevel: levelUpResult.previousLevel,
      timestamp: new Date()
    };

    // Broadcast to WebSocket rooms
    const io = socket.server;
    network.referrers.forEach(referrerId => {
      io.to(`user:${referrerId}`).emit('xp:network_level_up', levelUpNotification);
    });

    // Send push notifications
    await this.notificationService.sendNetworkLevelUpNotifications(userId, levelUpResult);
  }

  /**
   * Get streak data for user
   */
  private async getStreakData(userId: string): Promise<XPStreakData> {
    const cached = await this.redis.hmget(
      `user:${userId}:streak`,
      'currentStreak', 'longestStreak', 'lastActivityDate'
    );

    return {
      currentStreak: parseInt(cached[0]) || 0,
      longestStreak: parseInt(cached[1]) || 0,
      lastActivityDate: cached[2] || '',
      streakMultiplier: Math.min(3.0, 1.0 + (parseInt(cached[0]) || 0) * 0.1)
    };
  }

  /**
   * Save streak data
   */
  private async saveStreakData(userId: string, streakData: XPStreakData): Promise<void> {
    await this.redis.hmset(`user:${userId}:streak`, {
      currentStreak: streakData.currentStreak,
      longestStreak: streakData.longestStreak,
      lastActivityDate: streakData.lastActivityDate
    });
  }

  /**
   * Track XP activity analytics
   */
  private async trackXPActivity(userId: string, activity: XPEventData, result: XPCalculationResult): Promise<void> {
    const analyticsData = {
      userId,
      activityType: activity.activityType,
      platform: activity.platform,
      xpAwarded: result.awarded,
      qualityScore: result.qualityScore,
      multipliers: result.multipliers,
      timestamp: new Date()
    };

    await this.analyticsService.trackXPEvent(analyticsData);
  }

  /**
   * Subscribe to user XP updates
   */
  private async subscribeToUserXP(socket: Socket, userId: string): Promise<void> {
    socket.join(`user:${userId}`);
    socket.join(`xp_updates:${userId}`);
    
    this.logger.debug('Subscribed to XP updates', { userId, socketId: socket.id });
  }

  /**
   * Get daily activity count
   */
  private async getDailyActivityCount(userId: string, activityType: string): Promise<number> {
    const today = new Date().toISOString().split('T')[0];
    const count = await this.redis.hget(`user:${userId}:daily_activities:${today}`, activityType);
    return parseInt(count) || 0;
  }

  /**
   * Increment daily activity count
   */
  private async incrementDailyActivityCount(userId: string, activityType: string): Promise<void> {
    const today = new Date().toISOString().split('T')[0];
    const pipeline = this.redis.pipeline();
    
    pipeline.hincrby(`user:${userId}:daily_activities:${today}`, activityType, 1);
    pipeline.expire(`user:${userId}:daily_activities:${today}`, 86400); // Expire at end of day
    
    await pipeline.exec();
  }
}
