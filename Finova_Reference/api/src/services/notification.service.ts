import { Injectable } from '@nestjs/common';
import { InjectRepository } from '@nestjs/typeorm';
import { Repository } from 'typeorm';
import { Redis } from 'ioredis';
import { InjectRedis } from '@liaoliaots/nestjs-redis';
import { WebSocketGateway, WebSocketServer } from '@nestjs/websockets';
import { Server } from 'socket.io';
import { ConfigService } from '@nestjs/config';
import { HttpService } from '@nestjs/axios';
import { Queue } from 'bull';
import { InjectQueue } from '@nestjs/bull';
import { Logger } from '@nestjs/common';
import { Cron, CronExpression } from '@nestjs/schedule';
import { EventEmitter2 } from '@nestjs/event-emitter';

// Entities
import { User } from '../models/User.model';
import { NotificationTemplate } from '../models/NotificationTemplate.model';
import { NotificationLog } from '../models/NotificationLog.model';
import { UserPreferences } from '../models/UserPreferences.model';

// DTOs
interface NotificationPayload {
  userId: string;
  type: NotificationType;
  title: string;
  message: string;
  data?: Record<string, any>;
  priority: NotificationPriority;
  channels: NotificationChannel[];
  scheduled?: Date;
  expiresAt?: Date;
}

interface NotificationTemplate {
  id: string;
  type: NotificationType;
  title: string;
  message: string;
  variables: string[];
  channels: NotificationChannel[];
  isActive: boolean;
}

interface PushNotificationPayload {
  to: string[];
  sound: string;
  title: string;
  body: string;
  data?: Record<string, any>;
  badge?: number;
  priority?: 'high' | 'normal';
}

interface EmailPayload {
  to: string[];
  subject: string;
  html: string;
  attachments?: any[];
  templateId?: string;
  variables?: Record<string, any>;
}

interface SMSPayload {
  to: string[];
  message: string;
  templateId?: string;
  variables?: Record<string, any>;
}

enum NotificationType {
  MINING_REWARD = 'mining_reward',
  XP_MILESTONE = 'xp_milestone',
  RP_TIER_UP = 'rp_tier_up',
  REFERRAL_SUCCESS = 'referral_success',
  STAKING_REWARD = 'staking_reward',
  NFT_DROP = 'nft_drop',
  GUILD_INVITE = 'guild_invite',
  GUILD_EVENT = 'guild_event',
  SYSTEM_ANNOUNCEMENT = 'system_announcement',
  SECURITY_ALERT = 'security_alert',
  ACHIEVEMENT_UNLOCK = 'achievement_unlock',
  CARD_EXPIRY = 'card_expiry',
  MAINTENANCE = 'maintenance',
  PRICE_ALERT = 'price_alert',
  SOCIAL_INTERACTION = 'social_interaction',
  QUALITY_SCORE_UPDATE = 'quality_score_update'
}

enum NotificationPriority {
  LOW = 1,
  NORMAL = 2,
  HIGH = 3,
  URGENT = 4
}

enum NotificationChannel {
  IN_APP = 'in_app',
  PUSH = 'push',
  EMAIL = 'email',
  SMS = 'sms',
  WEBSOCKET = 'websocket',
  TELEGRAM = 'telegram',
  DISCORD = 'discord'
}

@Injectable()
@WebSocketGateway({ cors: true, namespace: '/notifications' })
export class NotificationService {
  @WebSocketServer()
  server: Server;

  private readonly logger = new Logger(NotificationService.name);

  constructor(
    @InjectRepository(User)
    private userRepository: Repository<User>,
    @InjectRepository(NotificationTemplate)
    private templateRepository: Repository<NotificationTemplate>,
    @InjectRepository(NotificationLog)
    private logRepository: Repository<NotificationLog>,
    @InjectRepository(UserPreferences)
    private preferencesRepository: Repository<UserPreferences>,
    @InjectRedis()
    private redis: Redis,
    private configService: ConfigService,
    private httpService: HttpService,
    @InjectQueue('notifications')
    private notificationQueue: Queue,
    @InjectQueue('email')
    private emailQueue: Queue,
    @InjectQueue('push')
    private pushQueue: Queue,
    private eventEmitter: EventEmitter2
  ) {}

  // Core notification sending method
  async sendNotification(payload: NotificationPayload): Promise<void> {
    try {
      // Validate user and preferences
      const user = await this.userRepository.findOne({ 
        where: { id: payload.userId },
        relations: ['preferences']
      });

      if (!user) {
        this.logger.warn(`User not found: ${payload.userId}`);
        return;
      }

      const preferences = await this.getUserPreferences(payload.userId);
      if (!this.shouldSendNotification(payload.type, payload.channels, preferences)) {
        this.logger.debug(`Notification blocked by user preferences: ${payload.userId}`);
        return;
      }

      // Rate limiting check
      const rateLimited = await this.checkRateLimit(payload.userId, payload.type);
      if (rateLimited) {
        this.logger.warn(`Rate limited notification for user: ${payload.userId}`);
        return;
      }

      // Schedule or send immediately
      if (payload.scheduled && payload.scheduled > new Date()) {
        await this.scheduleNotification(payload);
      } else {
        await this.processNotification(payload, user);
      }

    } catch (error) {
      this.logger.error('Failed to send notification', error);
      throw error;
    }
  }

  // Mining reward notifications
  async sendMiningReward(userId: string, amount: number, multipliers: any): Promise<void> {
    const template = await this.getTemplate(NotificationType.MINING_REWARD);
    const message = this.interpolateTemplate(template.message, {
      amount: amount.toFixed(4),
      multiplier: multipliers.total.toFixed(2),
      currency: '$FIN'
    });

    await this.sendNotification({
      userId,
      type: NotificationType.MINING_REWARD,
      title: template.title,
      message,
      data: { amount, multipliers },
      priority: NotificationPriority.NORMAL,
      channels: [NotificationChannel.IN_APP, NotificationChannel.PUSH]
    });
  }

  // XP milestone notifications
  async sendXPMilestone(userId: string, level: number, xpGained: number, rewards: any): Promise<void> {
    const template = await this.getTemplate(NotificationType.XP_MILESTONE);
    const message = this.interpolateTemplate(template.message, {
      level,
      xpGained,
      rewards: JSON.stringify(rewards)
    });

    await this.sendNotification({
      userId,
      type: NotificationType.XP_MILESTONE,
      title: `Level ${level} Achieved!`,
      message,
      data: { level, xpGained, rewards },
      priority: NotificationPriority.HIGH,
      channels: [NotificationChannel.IN_APP, NotificationChannel.PUSH, NotificationChannel.EMAIL]
    });
  }

  // RP tier upgrade notifications
  async sendRPTierUpgrade(userId: string, newTier: string, benefits: any): Promise<void> {
    const template = await this.getTemplate(NotificationType.RP_TIER_UP);
    const message = this.interpolateTemplate(template.message, {
      tier: newTier,
      benefits: JSON.stringify(benefits)
    });

    await this.sendNotification({
      userId,
      type: NotificationType.RP_TIER_UP,
      title: `${newTier} Tier Unlocked!`,
      message,
      data: { newTier, benefits },
      priority: NotificationPriority.HIGH,
      channels: [NotificationChannel.IN_APP, NotificationChannel.PUSH, NotificationChannel.EMAIL]
    });
  }

  // Referral success notifications
  async sendReferralSuccess(userId: string, referralData: any): Promise<void> {
    const template = await this.getTemplate(NotificationType.REFERRAL_SUCCESS);
    const message = this.interpolateTemplate(template.message, {
      referralName: referralData.name,
      bonus: referralData.bonus,
      networkSize: referralData.networkSize
    });

    await this.sendNotification({
      userId,
      type: NotificationType.REFERRAL_SUCCESS,
      title: template.title,
      message,
      data: referralData,
      priority: NotificationPriority.NORMAL,
      channels: [NotificationChannel.IN_APP, NotificationChannel.PUSH]
    });
  }

  // Staking reward notifications
  async sendStakingReward(userId: string, reward: number, stakingInfo: any): Promise<void> {
    const template = await this.getTemplate(NotificationType.STAKING_REWARD);
    const message = this.interpolateTemplate(template.message, {
      reward: reward.toFixed(4),
      stakingTier: stakingInfo.tier,
      apy: stakingInfo.apy
    });

    await this.sendNotification({
      userId,
      type: NotificationType.STAKING_REWARD,
      title: template.title,
      message,
      data: { reward, stakingInfo },
      priority: NotificationPriority.NORMAL,
      channels: [NotificationChannel.IN_APP, NotificationChannel.PUSH]
    });
  }

  // NFT drop notifications
  async sendNFTDrop(userIds: string[], nftData: any): Promise<void> {
    const template = await this.getTemplate(NotificationType.NFT_DROP);
    const message = this.interpolateTemplate(template.message, {
      nftName: nftData.name,
      rarity: nftData.rarity,
      price: nftData.price
    });

    for (const userId of userIds) {
      await this.sendNotification({
        userId,
        type: NotificationType.NFT_DROP,
        title: template.title,
        message,
        data: nftData,
        priority: NotificationPriority.HIGH,
        channels: [NotificationChannel.IN_APP, NotificationChannel.PUSH, NotificationChannel.EMAIL]
      });
    }
  }

  // Guild event notifications
  async sendGuildEvent(guildMembers: string[], eventData: any): Promise<void> {
    const template = await this.getTemplate(NotificationType.GUILD_EVENT);
    const message = this.interpolateTemplate(template.message, {
      eventName: eventData.name,
      guildName: eventData.guildName,
      startTime: eventData.startTime,
      rewards: eventData.rewards
    });

    for (const userId of guildMembers) {
      await this.sendNotification({
        userId,
        type: NotificationType.GUILD_EVENT,
        title: template.title,
        message,
        data: eventData,
        priority: NotificationPriority.NORMAL,
        channels: [NotificationChannel.IN_APP, NotificationChannel.PUSH]
      });
    }
  }

  // System announcements
  async sendSystemAnnouncement(userIds: string[], announcement: any): Promise<void> {
    const template = await this.getTemplate(NotificationType.SYSTEM_ANNOUNCEMENT);
    
    for (const userId of userIds) {
      await this.sendNotification({
        userId,
        type: NotificationType.SYSTEM_ANNOUNCEMENT,
        title: announcement.title,
        message: announcement.message,
        data: announcement,
        priority: NotificationPriority.HIGH,
        channels: [NotificationChannel.IN_APP, NotificationChannel.PUSH, NotificationChannel.EMAIL]
      });
    }
  }

  // Security alerts
  async sendSecurityAlert(userId: string, alertData: any): Promise<void> {
    const template = await this.getTemplate(NotificationType.SECURITY_ALERT);
    const message = this.interpolateTemplate(template.message, {
      alertType: alertData.type,
      location: alertData.location,
      timestamp: alertData.timestamp
    });

    await this.sendNotification({
      userId,
      type: NotificationType.SECURITY_ALERT,
      title: template.title,
      message,
      data: alertData,
      priority: NotificationPriority.URGENT,
      channels: [NotificationChannel.IN_APP, NotificationChannel.PUSH, NotificationChannel.EMAIL, NotificationChannel.SMS]
    });
  }

  // Achievement unlock notifications
  async sendAchievementUnlock(userId: string, achievement: any): Promise<void> {
    const template = await this.getTemplate(NotificationType.ACHIEVEMENT_UNLOCK);
    const message = this.interpolateTemplate(template.message, {
      achievementName: achievement.name,
      description: achievement.description,
      rewards: JSON.stringify(achievement.rewards)
    });

    await this.sendNotification({
      userId,
      type: NotificationType.ACHIEVEMENT_UNLOCK,
      title: template.title,
      message,
      data: achievement,
      priority: NotificationPriority.HIGH,
      channels: [NotificationChannel.IN_APP, NotificationChannel.PUSH]
    });
  }

  // Card expiry warnings
  async sendCardExpiry(userId: string, cardData: any): Promise<void> {
    const template = await this.getTemplate(NotificationType.CARD_EXPIRY);
    const message = this.interpolateTemplate(template.message, {
      cardName: cardData.name,
      expiryTime: cardData.expiryTime,
      effect: cardData.effect
    });

    await this.sendNotification({
      userId,
      type: NotificationType.CARD_EXPIRY,
      title: template.title,
      message,
      data: cardData,
      priority: NotificationPriority.NORMAL,
      channels: [NotificationChannel.IN_APP, NotificationChannel.PUSH]
    });
  }

  // Price alerts
  async sendPriceAlert(userId: string, priceData: any): Promise<void> {
    const template = await this.getTemplate(NotificationType.PRICE_ALERT);
    const message = this.interpolateTemplate(template.message, {
      symbol: priceData.symbol,
      price: priceData.price,
      change: priceData.change,
      threshold: priceData.threshold
    });

    await this.sendNotification({
      userId,
      type: NotificationType.PRICE_ALERT,
      title: template.title,
      message,
      data: priceData,
      priority: NotificationPriority.NORMAL,
      channels: [NotificationChannel.IN_APP, NotificationChannel.PUSH]
    });
  }

  // Social interaction notifications
  async sendSocialInteraction(userId: string, interaction: any): Promise<void> {
    const template = await this.getTemplate(NotificationType.SOCIAL_INTERACTION);
    const message = this.interpolateTemplate(template.message, {
      interactionType: interaction.type,
      username: interaction.username,
      contentType: interaction.contentType
    });

    await this.sendNotification({
      userId,
      type: NotificationType.SOCIAL_INTERACTION,
      title: template.title,
      message,
      data: interaction,
      priority: NotificationPriority.LOW,
      channels: [NotificationChannel.IN_APP]
    });
  }

  // Quality score update notifications
  async sendQualityScoreUpdate(userId: string, scoreData: any): Promise<void> {
    const template = await this.getTemplate(NotificationType.QUALITY_SCORE_UPDATE);
    const message = this.interpolateTemplate(template.message, {
      oldScore: scoreData.oldScore,
      newScore: scoreData.newScore,
      reason: scoreData.reason,
      impact: scoreData.impact
    });

    await this.sendNotification({
      userId,
      type: NotificationType.QUALITY_SCORE_UPDATE,
      title: template.title,
      message,
      data: scoreData,
      priority: NotificationPriority.NORMAL,
      channels: [NotificationChannel.IN_APP, NotificationChannel.PUSH]
    });
  }

  // Bulk notification methods
  async sendBulkNotifications(notifications: NotificationPayload[]): Promise<void> {
    const chunks = this.chunkArray(notifications, 100);
    
    for (const chunk of chunks) {
      await Promise.all(chunk.map(notification => 
        this.notificationQueue.add('process', notification, {
          priority: notification.priority,
          delay: 0,
          attempts: 3,
          backoff: { type: 'exponential', delay: 2000 }
        })
      ));
    }
  }

  // Template management
  async getTemplate(type: NotificationType): Promise<NotificationTemplate> {
    const cacheKey = `notification_template:${type}`;
    let template = await this.redis.get(cacheKey);
    
    if (!template) {
      const dbTemplate = await this.templateRepository.findOne({ 
        where: { type, isActive: true } 
      });
      
      if (dbTemplate) {
        template = JSON.stringify(dbTemplate);
        await this.redis.setex(cacheKey, 3600, template);
      }
    }
    
    return template ? JSON.parse(template) : this.getDefaultTemplate(type);
  }

  // User preferences management
  async getUserPreferences(userId: string): Promise<any> {
    const cacheKey = `user_preferences:${userId}`;
    let preferences = await this.redis.get(cacheKey);
    
    if (!preferences) {
      const dbPreferences = await this.preferencesRepository.findOne({ 
        where: { userId } 
      });
      
      if (dbPreferences) {
        preferences = JSON.stringify(dbPreferences);
        await this.redis.setex(cacheKey, 1800, preferences);
      }
    }
    
    return preferences ? JSON.parse(preferences) : this.getDefaultPreferences();
  }

  // Rate limiting
  async checkRateLimit(userId: string, type: NotificationType): Promise<boolean> {
    const limits = this.getRateLimits(type);
    const key = `rate_limit:${userId}:${type}`;
    
    const current = await this.redis.incr(key);
    if (current === 1) {
      await this.redis.expire(key, limits.windowSeconds);
    }
    
    return current > limits.maxCount;
  }

  // WebSocket real-time notifications
  async sendRealtimeNotification(userId: string, notification: any): Promise<void> {
    this.server.to(`user:${userId}`).emit('notification', notification);
    
    // Also emit to user's devices
    const userSockets = await this.redis.smembers(`user_sockets:${userId}`);
    userSockets.forEach(socketId => {
      this.server.to(socketId).emit('notification', notification);
    });
  }

  // Push notification service
  async sendPushNotification(payload: PushNotificationPayload): Promise<void> {
    try {
      const expoPushToken = this.configService.get('EXPO_PUSH_TOKEN');
      const fcmServerKey = this.configService.get('FCM_SERVER_KEY');
      
      // Expo notifications
      if (payload.to.some(token => token.startsWith('ExponentPushToken'))) {
        await this.sendExpoPushNotification(payload);
      }
      
      // FCM notifications
      if (payload.to.some(token => !token.startsWith('ExponentPushToken'))) {
        await this.sendFCMNotification(payload);
      }
      
    } catch (error) {
      this.logger.error('Failed to send push notification', error);
    }
  }

  // Email service integration
  async sendEmail(payload: EmailPayload): Promise<void> {
    await this.emailQueue.add('send', payload, {
      priority: 5,
      attempts: 3,
      backoff: { type: 'exponential', delay: 2000 }
    });
  }

  // SMS service integration
  async sendSMS(payload: SMSPayload): Promise<void> {
    try {
      const twilioSid = this.configService.get('TWILIO_ACCOUNT_SID');
      const twilioToken = this.configService.get('TWILIO_AUTH_TOKEN');
      
      // Implementation would depend on SMS provider
      // This is a placeholder for the actual SMS sending logic
      
    } catch (error) {
      this.logger.error('Failed to send SMS', error);
    }
  }

  // Scheduled notifications cleanup
  @Cron(CronExpression.EVERY_HOUR)
  async cleanupExpiredNotifications(): Promise<void> {
    const expiredCount = await this.logRepository
      .createQueryBuilder()
      .delete()
      .where('expiresAt < :now', { now: new Date() })
      .execute();
      
    this.logger.debug(`Cleaned up ${expiredCount.affected} expired notifications`);
  }

  // Analytics and metrics
  async getNotificationMetrics(userId?: string): Promise<any> {
    const baseQuery = this.logRepository.createQueryBuilder('log');
    
    if (userId) {
      baseQuery.where('log.userId = :userId', { userId });
    }
    
    const [sent, delivered, opened, clicked] = await Promise.all([
      baseQuery.clone().getCount(),
      baseQuery.clone().where('log.status = :status', { status: 'delivered' }).getCount(),
      baseQuery.clone().where('log.status = :status', { status: 'opened' }).getCount(),
      baseQuery.clone().where('log.status = :status', { status: 'clicked' }).getCount()
    ]);
    
    return {
      sent,
      delivered,
      opened,
      clicked,
      deliveryRate: delivered / sent * 100,
      openRate: opened / delivered * 100,
      clickRate: clicked / opened * 100
    };
  }

  // Private helper methods
  private async processNotification(payload: NotificationPayload, user: any): Promise<void> {
    for (const channel of payload.channels) {
      switch (channel) {
        case NotificationChannel.IN_APP:
          await this.saveInAppNotification(payload);
          break;
        case NotificationChannel.WEBSOCKET:
          await this.sendRealtimeNotification(payload.userId, payload);
          break;
        case NotificationChannel.PUSH:
          if (user.pushToken) {
            await this.sendPushNotification({
              to: [user.pushToken],
              title: payload.title,
              body: payload.message,
              data: payload.data,
              sound: 'default',
              priority: payload.priority >= NotificationPriority.HIGH ? 'high' : 'normal'
            });
          }
          break;
        case NotificationChannel.EMAIL:
          if (user.email) {
            await this.sendEmail({
              to: [user.email],
              subject: payload.title,
              html: payload.message
            });
          }
          break;
        case NotificationChannel.SMS:
          if (user.phone) {
            await this.sendSMS({
              to: [user.phone],
              message: `${payload.title}: ${payload.message}`
            });
          }
          break;
      }
    }
    
    // Log notification
    await this.logNotification(payload, 'sent');
  }

  private async saveInAppNotification(payload: NotificationPayload): Promise<void> {
    const notification = {
      userId: payload.userId,
      type: payload.type,
      title: payload.title,
      message: payload.message,
      data: payload.data,
      isRead: false,
      createdAt: new Date(),
      expiresAt: payload.expiresAt
    };
    
    await this.redis.lpush(
      `notifications:${payload.userId}`, 
      JSON.stringify(notification)
    );
    
    // Keep only latest 100 notifications per user
    await this.redis.ltrim(`notifications:${payload.userId}`, 0, 99);
  }

  private async logNotification(payload: NotificationPayload, status: string): Promise<void> {
    await this.logRepository.save({
      userId: payload.userId,
      type: payload.type,
      title: payload.title,
      message: payload.message,
      channels: payload.channels,
      status,
      sentAt: new Date(),
      data: payload.data
    });
  }

  private interpolateTemplate(template: string, variables: Record<string, any>): string {
    return template.replace(/\{\{(\w+)\}\}/g, (match, key) => {
      return variables[key] !== undefined ? variables[key] : match;
    });
  }

  private shouldSendNotification(type: NotificationType, channels: NotificationChannel[], preferences: any): boolean {
    if (!preferences) return true;
    
    const typePrefs = preferences[type] || {};
    return channels.some(channel => typePrefs[channel] !== false);
  }

  private async scheduleNotification(payload: NotificationPayload): Promise<void> {
    const delay = payload.scheduled.getTime() - Date.now();
    
    await this.notificationQueue.add('process', payload, {
      delay,
      priority: payload.priority,
      attempts: 3,
      backoff: { type: 'exponential', delay: 2000 }
    });
  }

  private getRateLimits(type: NotificationType): { maxCount: number; windowSeconds: number } {
    const limits = {
      [NotificationType.MINING_REWARD]: { maxCount: 100, windowSeconds: 3600 },
      [NotificationType.SOCIAL_INTERACTION]: { maxCount: 50, windowSeconds: 3600 },
      [NotificationType.SECURITY_ALERT]: { maxCount: 5, windowSeconds: 3600 },
      default: { maxCount: 20, windowSeconds: 3600 }
    };
    
    return limits[type] || limits.default;
  }

  private getDefaultTemplate(type: NotificationType): NotificationTemplate {
    const templates = {
      [NotificationType.MINING_REWARD]: {
        id: 'default',
        type,
        title: 'Mining Reward Earned!',
        message: 'You earned {{amount}} {{currency}} with {{multiplier}}x multiplier!',
        variables: ['amount', 'currency', 'multiplier'],
        channels: [NotificationChannel.IN_APP, NotificationChannel.PUSH],
        isActive: true
      }
    };
    
    return templates[type] || {
      id: 'default',
      type,
      title: 'Notification',
      message: 'You have a new notification',
      variables: [],
      channels: [NotificationChannel.IN_APP],
      isActive: true
    };
  }

  private getDefaultPreferences(): any {
    return {
      [NotificationType.MINING_REWARD]: {
        [NotificationChannel.IN_APP]: true,
        [NotificationChannel.PUSH]: true,
        [NotificationChannel.EMAIL]: false
      },
      [NotificationType.SECURITY_ALERT]: {
        [NotificationChannel.IN_APP]: true,
        [NotificationChannel.PUSH]: true,
        [NotificationChannel.EMAIL]: true,
        [NotificationChannel.SMS]: true
      }
    };
  }

  private async sendExpoPushNotification(payload: PushNotificationPayload): Promise<void> {
    // Expo push notification implementation
    const messages = payload.to.map(token => ({
      to: token,
      sound: payload.sound,
      title: payload.title,
      body: payload.body,
      data: payload.data,
      badge: payload.badge,
      priority: payload.priority
    }));
    
    // Send to Expo push service
    // Implementation would use Expo SDK
  }

  private async sendFCMNotification(payload: PushNotificationPayload): Promise<void> {
    // FCM push notification implementation
    const fcmPayload = {
      registration_ids: payload.to,
      notification: {
        title: payload.title,
        body: payload.body,
        sound: payload.sound,
        badge: payload.badge
      },
      data: payload.data,
      priority: payload.priority
    };
    
    // Send to FCM service
    // Implementation would use FCM SDK
  }

  private chunkArray<T>(array: T[], size: number): T[][] {
    const chunks: T[][] = [];
    for (let i = 0; i < array.length; i += size) {
      chunks.push(array.slice(i, i + size));
    }
    return chunks;
  }
}
