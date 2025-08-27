import { Injectable, Logger, BadRequestException, UnauthorizedException } from '@nestjs/common';
import { InjectRepository } from '@nestjs/typeorm';
import { Repository, QueryRunner } from 'typeorm';
import { ConfigService } from '@nestjs/config';
import { EventEmitter2 } from '@nestjs/event-emitter';
import axios, { AxiosInstance } from 'axios';
import * as crypto from 'crypto';

// Entities
import { User } from '../models/User.model';
import { SocialAccount } from '../models/SocialAccount.model';
import { SocialPost } from '../models/SocialPost.model';
import { SocialEngagement } from '../models/SocialEngagement.model';
import { XPTransaction } from '../models/XP.model';
import { MiningActivity } from '../models/Mining.model';

// Services
import { AiQualityService } from './ai-quality.service';
import { AntiBotService } from './anti-bot.service';
import { XpService } from './xp.service';
import { MiningService } from './mining.service';
import { NotificationService } from './notification.service';

// DTOs and Types
interface SocialPlatform {
  name: 'instagram' | 'tiktok' | 'youtube' | 'facebook' | 'twitter' | 'x';
  apiBaseUrl: string;
  requiredScopes: string[];
  multiplier: number;
}

interface ContentAnalysis {
  qualityScore: number;
  originalityScore: number;
  engagementPotential: number;
  brandSafety: number;
  isViral: boolean;
  tags: string[];
  sentiment: 'positive' | 'negative' | 'neutral';
}

interface SocialMetrics {
  views: number;
  likes: number;
  comments: number;
  shares: number;
  saves?: number;
  reach?: number;
  impressions?: number;
}

interface ActivityReward {
  baseXP: number;
  bonusXP: number;
  finReward: number;
  rpBonus: number;
  qualityMultiplier: number;
  platformMultiplier: number;
  finalReward: number;
}

@Injectable()
export class SocialService {
  private readonly logger = new Logger(SocialService.name);
  private readonly platformClients: Map<string, AxiosInstance> = new Map();
  
  // Platform configurations with multipliers from whitepaper
  private readonly platforms: SocialPlatform[] = [
    {
      name: 'tiktok',
      apiBaseUrl: 'https://open-api.tiktok.com',
      requiredScopes: ['user.info.basic', 'video.list'],
      multiplier: 1.3 // From whitepaper
    },
    {
      name: 'instagram',
      apiBaseUrl: 'https://graph.instagram.com',
      requiredScopes: ['instagram_basic', 'instagram_content_publish'],
      multiplier: 1.2
    },
    {
      name: 'youtube',
      apiBaseUrl: 'https://www.googleapis.com/youtube/v3',
      requiredScopes: ['https://www.googleapis.com/auth/youtube.readonly'],
      multiplier: 1.4
    },
    {
      name: 'facebook',
      apiBaseUrl: 'https://graph.facebook.com',
      requiredScopes: ['pages_read_engagement', 'pages_manage_posts'],
      multiplier: 1.1
    },
    {
      name: 'x',
      apiBaseUrl: 'https://api.twitter.com/2',
      requiredScopes: ['tweet.read', 'users.read'],
      multiplier: 1.2
    }
  ];

  // XP Values from whitepaper
  private readonly xpValues = {
    originalPost: 50,
    photoPost: 75,
    videoContent: 150,
    storyStatus: 25,
    meaningfulComment: 25,
    likeReact: 5,
    shareRepost: 15,
    followSubscribe: 20,
    dailyLogin: 10,
    dailyQuest: 100,
    milestone: 500,
    viralContent: 1000
  };

  constructor(
    @InjectRepository(User)
    private readonly userRepository: Repository<User>,
    
    @InjectRepository(SocialAccount)
    private readonly socialAccountRepository: Repository<SocialAccount>,
    
    @InjectRepository(SocialPost)
    private readonly socialPostRepository: Repository<SocialPost>,
    
    @InjectRepository(SocialEngagement)
    private readonly socialEngagementRepository: Repository<SocialEngagement>,
    
    @InjectRepository(XPTransaction)
    private readonly xpTransactionRepository: Repository<XPTransaction>,
    
    @InjectRepository(MiningActivity)
    private readonly miningActivityRepository: Repository<MiningActivity>,
    
    private readonly configService: ConfigService,
    private readonly eventEmitter: EventEmitter2,
    private readonly aiQualityService: AiQualityService,
    private readonly antiBotService: AntiBotService,
    private readonly xpService: XpService,
    private readonly miningService: MiningService,
    private readonly notificationService: NotificationService,
  ) {
    this.initializePlatformClients();
  }

  /**
   * Initialize API clients for all social platforms
   */
  private initializePlatformClients(): void {
    this.platforms.forEach(platform => {
      const client = axios.create({
        baseURL: platform.apiBaseUrl,
        timeout: 30000,
        headers: {
          'User-Agent': 'Finova-Network/1.0',
          'Accept': 'application/json',
        }
      });

      // Add request interceptor for authentication
      client.interceptors.request.use((config) => {
        const token = this.configService.get(`${platform.name.toUpperCase()}_ACCESS_TOKEN`);
        if (token) {
          config.headers.Authorization = `Bearer ${token}`;
        }
        return config;
      });

      // Add response interceptor for error handling
      client.interceptors.response.use(
        (response) => response,
        (error) => {
          this.logger.error(`${platform.name} API Error:`, error.response?.data || error.message);
          return Promise.reject(error);
        }
      );

      this.platformClients.set(platform.name, client);
    });
  }

  /**
   * Connect user's social media account
   */
  async connectSocialAccount(
    userId: string,
    platform: string,
    accessToken: string,
    refreshToken?: string,
    additionalData?: any
  ): Promise<SocialAccount> {
    const user = await this.userRepository.findOne({ where: { id: userId } });
    if (!user) {
      throw new BadRequestException('User not found');
    }

    // Verify token and get profile info
    const profileInfo = await this.verifyAndGetProfile(platform, accessToken);
    
    // Check if account already connected by platform user ID
    const existingAccount = await this.socialAccountRepository.findOne({
      where: { platformUserId: profileInfo.id, platform }
    });

    if (existingAccount && existingAccount.userId !== userId) {
      throw new BadRequestException('This social account is already connected to another user');
    }

    // Create or update social account
    const socialAccount = existingAccount || new SocialAccount();
    socialAccount.userId = userId;
    socialAccount.platform = platform;
    socialAccount.platformUserId = profileInfo.id;
    socialAccount.username = profileInfo.username;
    socialAccount.displayName = profileInfo.displayName;
    socialAccount.profilePicture = profileInfo.profilePicture;
    socialAccount.accessToken = this.encryptToken(accessToken);
    socialAccount.refreshToken = refreshToken ? this.encryptToken(refreshToken) : null;
    socialAccount.isActive = true;
    socialAccount.followerCount = profileInfo.followerCount || 0;
    socialAccount.followingCount = profileInfo.followingCount || 0;
    socialAccount.postsCount = profileInfo.postsCount || 0;
    socialAccount.lastSyncAt = new Date();
    socialAccount.metadata = additionalData || {};

    const savedAccount = await this.socialAccountRepository.save(socialAccount);

    // Award connection bonus XP
    await this.awardConnectionBonus(userId, platform);

    // Emit event
    this.eventEmitter.emit('social.account.connected', {
      userId,
      platform,
      accountId: savedAccount.id
    });

    this.logger.log(`User ${userId} connected ${platform} account: ${profileInfo.username}`);
    
    return savedAccount;
  }

  /**
   * Process social media activity and calculate rewards
   */
  async processActivity(
    userId: string,
    platform: string,
    activityType: string,
    contentData: any,
    metrics?: SocialMetrics
  ): Promise<ActivityReward> {
    const user = await this.userRepository.findOne({
      where: { id: userId },
      relations: ['xpProfile', 'referralProfile', 'miningProfile']
    });

    if (!user) {
      throw new BadRequestException('User not found');
    }

    // Check for bot behavior
    const humanProbability = await this.antiBotService.calculateHumanProbability({
      userId,
      activityType,
      platform,
      contentData,
      timestamp: new Date()
    });

    if (humanProbability < 0.3) {
      this.logger.warn(`Suspicious activity detected for user ${userId}`);
      throw new BadRequestException('Activity flagged as suspicious');
    }

    // Get platform multiplier
    const platformConfig = this.platforms.find(p => p.name === platform);
    const platformMultiplier = platformConfig?.multiplier || 1.0;

    // Analyze content quality
    const contentAnalysis = await this.analyzeContentQuality(contentData, activityType);

    // Calculate base XP
    const baseXP = this.calculateBaseXP(activityType, metrics);

    // Apply multipliers from whitepaper formula
    const qualityMultiplier = contentAnalysis.qualityScore;
    const streakBonus = await this.calculateStreakBonus(userId);
    const levelProgression = Math.exp(-0.01 * (user.xpProfile?.currentLevel || 1));

    const totalXP = Math.floor(
      baseXP * platformMultiplier * qualityMultiplier * streakBonus * levelProgression
    );

    // Calculate mining rewards
    const miningBoost = this.calculateMiningBoost(activityType, contentAnalysis.isViral);
    const finReward = await this.miningService.calculateActivityReward(userId, miningBoost);

    // Calculate RP bonus
    const rpBonus = this.calculateRPBonus(user.referralProfile?.currentTier || 0, totalXP);

    // Store activity record
    await this.recordSocialActivity(userId, platform, activityType, contentData, {
      baseXP,
      totalXP,
      finReward,
      rpBonus,
      qualityScore: contentAnalysis.qualityScore,
      platformMultiplier,
      metrics: metrics || {}
    });

    // Award XP
    await this.xpService.awardXP(userId, totalXP, `${platform}_${activityType}`, {
      platform,
      activityType,
      qualityScore: contentAnalysis.qualityScore
    });

    // Process mining reward
    if (finReward > 0) {
      await this.miningService.processActivityReward(userId, finReward, activityType);
    }

    // Check for achievements and milestones
    await this.checkAchievements(userId, activityType, metrics);

    const reward: ActivityReward = {
      baseXP,
      bonusXP: totalXP - baseXP,
      finReward,
      rpBonus,
      qualityMultiplier,
      platformMultiplier,
      finalReward: totalXP + finReward + rpBonus
    };

    // Emit event
    this.eventEmitter.emit('social.activity.processed', {
      userId,
      platform,
      activityType,
      reward,
      contentAnalysis
    });

    return reward;
  }

  /**
   * Sync user's social media content
   */
  async syncUserContent(userId: string, platform: string): Promise<void> {
    const socialAccount = await this.socialAccountRepository.findOne({
      where: { userId, platform, isActive: true }
    });

    if (!socialAccount) {
      throw new BadRequestException('Social account not connected');
    }

    try {
      const client = this.platformClients.get(platform);
      if (!client) {
        throw new BadRequestException('Platform not supported');
      }

      // Decrypt access token
      const accessToken = this.decryptToken(socialAccount.accessToken);
      
      // Fetch recent content based on platform
      const recentContent = await this.fetchRecentContent(platform, accessToken, socialAccount.platformUserId);

      // Process each piece of content
      for (const content of recentContent) {
        // Check if already processed
        const existingPost = await this.socialPostRepository.findOne({
          where: {
            userId,
            platform,
            platformPostId: content.id
          }
        });

        if (!existingPost) {
          // Process new content
          await this.processActivity(
            userId,
            platform,
            content.type,
            content,
            content.metrics
          );

          // Store post record
          const socialPost = new SocialPost();
          socialPost.userId = userId;
          socialPost.platform = platform;
          socialPost.platformPostId = content.id;
          socialPost.content = content.text || content.caption;
          socialPost.mediaType = content.mediaType;
          socialPost.mediaUrl = content.mediaUrl;
          socialPost.metrics = content.metrics;
          socialPost.processingStatus = 'completed';
          socialPost.createdAt = new Date(content.timestamp);

          await this.socialPostRepository.save(socialPost);
        }
      }

      // Update last sync time
      socialAccount.lastSyncAt = new Date();
      await this.socialAccountRepository.save(socialAccount);

      this.logger.log(`Synced content for user ${userId} on ${platform}`);
    } catch (error) {
      this.logger.error(`Failed to sync content for user ${userId} on ${platform}:`, error);
      throw error;
    }
  }

  /**
   * Get user's social media statistics
   */
  async getUserSocialStats(userId: string): Promise<any> {
    const socialAccounts = await this.socialAccountRepository.find({
      where: { userId, isActive: true }
    });

    const stats = {
      connectedPlatforms: socialAccounts.length,
      totalFollowers: 0,
      totalPosts: 0,
      platformBreakdown: {},
      recentActivity: [],
      topPlatform: null,
      engagementRate: 0
    };

    for (const account of socialAccounts) {
      stats.totalFollowers += account.followerCount;
      stats.totalPosts += account.postsCount;
      
      stats.platformBreakdown[account.platform] = {
        followers: account.followerCount,
        posts: account.postsCount,
        lastSync: account.lastSyncAt,
        isActive: account.isActive
      };
    }

    // Get recent social activities
    const recentPosts = await this.socialPostRepository.find({
      where: { userId },
      order: { createdAt: 'DESC' },
      take: 10
    });

    stats.recentActivity = recentPosts.map(post => ({
      platform: post.platform,
      type: post.mediaType,
      metrics: post.metrics,
      createdAt: post.createdAt
    }));

    // Calculate top platform by engagement
    if (socialAccounts.length > 0) {
      const topAccount = socialAccounts.reduce((prev, current) => {
        return (prev.followerCount > current.followerCount) ? prev : current;
      });
      stats.topPlatform = topAccount.platform;
    }

    return stats;
  }

  /**
   * Calculate daily social media bonuses
   */
  async calculateDailyBonuses(userId: string): Promise<any> {
    const today = new Date();
    today.setHours(0, 0, 0, 0);
    
    const tomorrow = new Date(today);
    tomorrow.setDate(tomorrow.getDate() + 1);

    // Get today's social activities
    const todayActivities = await this.socialPostRepository.find({
      where: {
        userId,
        createdAt: {
          $gte: today,
          $lt: tomorrow
        } as any
      }
    });

    const bonuses = {
      dailyPostBonus: 0,
      platformDiversityBonus: 0,
      qualityBonus: 0,
      engagementBonus: 0,
      totalBonus: 0
    };

    // Daily post bonus (from whitepaper: +20% for 24 hours)
    if (todayActivities.length > 0) {
      bonuses.dailyPostBonus = 0.2; // 20% bonus
    }

    // Platform diversity bonus (posting on multiple platforms)
    const uniquePlatforms = new Set(todayActivities.map(a => a.platform));
    bonuses.platformDiversityBonus = Math.min(uniquePlatforms.size * 0.1, 0.5); // Max 50% bonus

    // Quality bonus based on average engagement
    const totalEngagement = todayActivities.reduce((sum, activity) => {
      const metrics = activity.metrics as any;
      return sum + (metrics?.likes || 0) + (metrics?.comments || 0) + (metrics?.shares || 0);
    }, 0);

    if (todayActivities.length > 0) {
      const avgEngagement = totalEngagement / todayActivities.length;
      bonuses.qualityBonus = Math.min(avgEngagement / 1000, 0.3); // Max 30% bonus
    }

    bonuses.totalBonus = bonuses.dailyPostBonus + bonuses.platformDiversityBonus + bonuses.qualityBonus;

    return bonuses;
  }

  // Private helper methods

  private async verifyAndGetProfile(platform: string, accessToken: string): Promise<any> {
    const client = this.platformClients.get(platform);
    if (!client) {
      throw new BadRequestException('Platform not supported');
    }

    try {
      let response;
      
      switch (platform) {
        case 'instagram':
          response = await client.get('/me', {
            headers: { Authorization: `Bearer ${accessToken}` },
            params: { fields: 'id,username,account_type,media_count,followers_count' }
          });
          return {
            id: response.data.id,
            username: response.data.username,
            displayName: response.data.username,
            followerCount: response.data.followers_count,
            postsCount: response.data.media_count
          };

        case 'tiktok':
          response = await client.post('/v2/user/info/', {}, {
            headers: { Authorization: `Bearer ${accessToken}` }
          });
          return {
            id: response.data.data.user.union_id,
            username: response.data.data.user.username,
            displayName: response.data.data.user.display_name,
            profilePicture: response.data.data.user.avatar_url,
            followerCount: response.data.data.user.follower_count,
            followingCount: response.data.data.user.following_count
          };

        // Add other platforms...
        default:
          throw new BadRequestException('Platform verification not implemented');
      }
    } catch (error) {
      this.logger.error(`Failed to verify ${platform} token:`, error);
      throw new UnauthorizedException('Invalid access token');
    }
  }

  private async analyzeContentQuality(contentData: any, activityType: string): Promise<ContentAnalysis> {
    try {
      const analysis = await this.aiQualityService.analyzeContent({
        text: contentData.text || contentData.caption || '',
        mediaUrl: contentData.mediaUrl,
        mediaType: contentData.mediaType,
        activityType
      });

      return {
        qualityScore: Math.max(0.5, Math.min(2.0, analysis.qualityScore)), // Clamp between 0.5x - 2.0x
        originalityScore: analysis.originalityScore,
        engagementPotential: analysis.engagementPotential,
        brandSafety: analysis.brandSafety,
        isViral: analysis.engagementPotential > 0.8,
        tags: analysis.extractedTags || [],
        sentiment: analysis.sentiment
      };
    } catch (error) {
      this.logger.error('Content analysis failed, using default values:', error);
      return {
        qualityScore: 1.0,
        originalityScore: 0.7,
        engagementPotential: 0.5,
        brandSafety: 0.9,
        isViral: false,
        tags: [],
        sentiment: 'neutral'
      };
    }
  }

  private calculateBaseXP(activityType: string, metrics?: SocialMetrics): number {
    let baseXP = 0;

    switch (activityType) {
      case 'post':
      case 'original_post':
        baseXP = this.xpValues.originalPost;
        break;
      case 'photo':
      case 'image_post':
        baseXP = this.xpValues.photoPost;
        break;
      case 'video':
      case 'video_post':
        baseXP = this.xpValues.videoContent;
        break;
      case 'story':
      case 'status':
        baseXP = this.xpValues.storyStatus;
        break;
      case 'comment':
        baseXP = this.xpValues.meaningfulComment;
        break;
      case 'like':
      case 'reaction':
        baseXP = this.xpValues.likeReact;
        break;
      case 'share':
      case 'repost':
        baseXP = this.xpValues.shareRepost;
        break;
      case 'follow':
      case 'subscribe':
        baseXP = this.xpValues.followSubscribe;
        break;
      default:
        baseXP = 10; // Default value
    }

    // Viral content bonus (1K+ views from whitepaper)
    if (metrics && (metrics.views >= 1000 || metrics.likes >= 100)) {
      baseXP = this.xpValues.viralContent;
    }

    return baseXP;
  }

  private async calculateStreakBonus(userId: string): Promise<number> {
    // Get user's activity streak
    const streak = await this.getUserActivityStreak(userId);
    return Math.min(1 + (streak * 0.1), 3.0); // Max 3x bonus from whitepaper
  }

  private calculateMiningBoost(activityType: string, isViral: boolean): number {
    let boost = 0.01; // Base boost

    switch (activityType) {
      case 'post':
      case 'original_post':
        boost = 0.05; // 0.05 $FIN from whitepaper
        break;
      case 'video':
      case 'video_post':
        boost = 0.08;
        break;
      case 'comment':
        boost = 0.02;
        break;
    }

    // Viral content gets 10x boost
    if (isViral) {
      boost *= 10;
    }

    return boost;
  }

  private calculateRPBonus(rpTier: number, xpGained: number): number {
    const rpPercentage = Math.min(rpTier * 0.05, 0.3); // Max 30% from RP
    return Math.floor(xpGained * rpPercentage);
  }

  private async recordSocialActivity(
    userId: string,
    platform: string,
    activityType: string,
    contentData: any,
    rewards: any
  ): Promise<void> {
    const engagement = new SocialEngagement();
    engagement.userId = userId;
    engagement.platform = platform;
    engagement.activityType = activityType;
    engagement.contentId = contentData.id;
    engagement.xpAwarded = rewards.totalXP;
    engagement.finAwarded = rewards.finReward;
    engagement.qualityScore = rewards.qualityScore;
    engagement.platformMultiplier = rewards.platformMultiplier;
    engagement.metadata = {
      contentData,
      metrics: rewards.metrics
    };

    await this.socialEngagementRepository.save(engagement);
  }

  private async getUserActivityStreak(userId: string): Promise<number> {
    // Implementation to calculate consecutive days of activity
    const recentDays = 30;
    const activities = await this.socialEngagementRepository.find({
      where: { userId },
      order: { createdAt: 'DESC' },
      take: recentDays
    });

    let streak = 0;
    let currentDate = new Date();
    currentDate.setHours(0, 0, 0, 0);

    for (let i = 0; i < recentDays; i++) {
      const dayActivities = activities.filter(activity => {
        const activityDate = new Date(activity.createdAt);
        activityDate.setHours(0, 0, 0, 0);
        return activityDate.getTime() === currentDate.getTime();
      });

      if (dayActivities.length > 0) {
        streak++;
        currentDate.setDate(currentDate.getDate() - 1);
      } else {
        break;
      }
    }

    return streak;
  }

  private async awardConnectionBonus(userId: string, platform: string): Promise<void> {
    const bonusXP = 100; // Connection bonus
    await this.xpService.awardXP(userId, bonusXP, 'social_connection', { platform });
  }

  private async fetchRecentContent(platform: string, accessToken: string, platformUserId: string): Promise<any[]> {
    const client = this.platformClients.get(platform);
    if (!client) return [];

    try {
      // Platform-specific content fetching logic
      switch (platform) {
        case 'instagram':
          const response = await client.get(`/${platformUserId}/media`, {
            headers: { Authorization: `Bearer ${accessToken}` },
            params: {
              fields: 'id,caption,media_type,media_url,timestamp,like_count,comments_count',
              limit: 25
            }
          });
          
          return response.data.data.map((item: any) => ({
            id: item.id,
            type: 'post',
            mediaType: item.media_type,
            mediaUrl: item.media_url,
            caption: item.caption,
            timestamp: item.timestamp,
            metrics: {
              likes: item.like_count,
              comments: item.comments_count
            }
          }));

        // Add other platforms...
        default:
          return [];
      }
    } catch (error) {
      this.logger.error(`Failed to fetch content from ${platform}:`, error);
      return [];
    }
  }

  private async checkAchievements(userId: string, activityType: string, metrics?: SocialMetrics): Promise<void> {
    // Check for various achievements
    const achievements = [];

    // First viral post achievement
    if (metrics && metrics.views >= 1000) {
      achievements.push({
        type: 'first_viral',
        xpBonus: 500,
        title: 'Viral Creator'
      });
    }

    // Daily activity achievements
    if (activityType === 'post') {
      const todayPosts = await this.getTodayPostCount(userId);
      if (todayPosts === 5) {
        achievements.push({
          type: 'daily_creator',
          xpBonus: 200,
          title: 'Daily Creator'
        });
      }
    }

    // Award achievements
    for (const achievement of achievements) {
      await this.xpService.awardXP(userId, achievement.xpBonus, 'achievement', achievement);
      
      this.eventEmitter.emit('user.achievement.unlocked', {
        userId,
        achievement
      });
    }
  }

  private async getTodayPostCount(userId: string): Promise<number> {
    const today = new Date();
    today.setHours(0, 0, 0, 0);
    
    const tomorrow = new Date(today);
    tomorrow.setDate(tomorrow.getDate() + 1);

    return await this.socialPostRepository.count({
      where: {
        userId,
        createdAt: {
          $gte: today,
          $lt: tomorrow
        } as any
      }
    });
  }

  private encryptToken(token: string): string {
    const algorithm = 'aes-256-gcm';
    const key = crypto.scryptSync(this.configService.get('ENCRYPTION_PASSWORD'), 'salt', 32);
    const iv = crypto.randomBytes(16);
    const cipher = crypto.createCipher(algorithm, key);
    
    let encrypted = cipher.update(token, 'utf8', 'hex');
    encrypted += cipher.final('hex');
    
    return `${iv.toString('hex')}:${encrypted}`;
  }

  private decryptToken(encryptedToken: string): string {
    const algorithm = 'aes-256-gcm';
    const key = crypto.scryptSync(this.configService.get('ENCRYPTION_PASSWORD'), 'salt', 32);
    const [ivHex, encrypted] = encryptedToken.split(':');
    const iv = Buffer.from(ivHex, 'hex');
    const decipher = crypto.createDecipher(algorithm, key);
    
    let decrypted = decipher.update(encrypted, 'hex', 'utf8');
    decrypted += decipher.final('utf8');
    
    return decrypted;
  }
}

