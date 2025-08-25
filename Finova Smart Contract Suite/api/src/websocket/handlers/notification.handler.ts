import { WebSocket } from 'ws';
import { Redis } from 'ioredis';
import { EventEmitter } from 'events';
import { Logger } from 'winston';
import { validateNotificationData, sanitizeUserInput } from '../utils/validation';
import { encryptSensitiveData, decryptSensitiveData } from '../utils/encryption';
import { rateLimiter } from '../middleware/rate-limit.ws';
import { authenticateWsUser } from '../middleware/auth.ws';

// Enhanced Types for Finova Network
interface NotificationPayload {
  id: string;
  type: NotificationType;
  userId: string;
  title: string;
  message: string;
  data?: any;
  priority: NotificationPriority;
  category: NotificationCategory;
  timestamp: Date;
  expiresAt?: Date;
  actionUrl?: string;
  imageUrl?: string;
  soundEnabled: boolean;
  vibrationEnabled: boolean;
  encryptedData?: string;
}

enum NotificationType {
  // Mining & Rewards
  MINING_REWARD = 'mining_reward',
  MINING_BOOST_ACTIVATED = 'mining_boost_activated',
  MINING_MILESTONE = 'mining_milestone',
  DAILY_MINING_COMPLETE = 'daily_mining_complete',
  
  // XP System
  XP_GAINED = 'xp_gained',
  LEVEL_UP = 'level_up',
  XP_MILESTONE = 'xp_milestone',
  STREAK_BONUS = 'streak_bonus',
  VIRAL_CONTENT = 'viral_content',
  
  // Referral Points (RP)
  NEW_REFERRAL = 'new_referral',
  REFERRAL_MILESTONE = 'referral_milestone',
  RP_TIER_UPGRADE = 'rp_tier_upgrade',
  NETWORK_ACHIEVEMENT = 'network_achievement',
  
  // Social Integration
  SOCIAL_POST_SUCCESS = 'social_post_success',
  PLATFORM_CONNECTED = 'platform_connected',
  CONTENT_APPROVED = 'content_approved',
  ENGAGEMENT_REWARD = 'engagement_reward',
  
  // NFT & Special Cards
  NFT_RECEIVED = 'nft_received',
  CARD_ACTIVATED = 'card_activated',
  CARD_EXPIRED = 'card_expired',
  MARKETPLACE_SALE = 'marketplace_sale',
  
  // Staking & DeFi
  STAKING_REWARD = 'staking_reward',
  STAKE_UNLOCKED = 'stake_unlocked',
  YIELD_CLAIMED = 'yield_claimed',
  
  // Guild & Community
  GUILD_INVITE = 'guild_invite',
  GUILD_EVENT = 'guild_event',
  TOURNAMENT_UPDATE = 'tournament_update',
  GUILD_ACHIEVEMENT = 'guild_achievement',
  
  // System & Security
  SECURITY_ALERT = 'security_alert',
  SYSTEM_MAINTENANCE = 'system_maintenance',
  ACCOUNT_VERIFICATION = 'account_verification',
  SUSPICIOUS_ACTIVITY = 'suspicious_activity',
  
  // Governance & DAO
  PROPOSAL_CREATED = 'proposal_created',
  VOTING_REMINDER = 'voting_reminder',
  GOVERNANCE_UPDATE = 'governance_update',
  
  // Economy & Trading
  PRICE_ALERT = 'price_alert',
  TRADE_EXECUTED = 'trade_executed',
  MARKET_UPDATE = 'market_update'
}

enum NotificationPriority {
  LOW = 'low',
  NORMAL = 'normal',
  HIGH = 'high',
  URGENT = 'urgent',
  CRITICAL = 'critical'
}

enum NotificationCategory {
  REWARDS = 'rewards',
  SOCIAL = 'social',
  SECURITY = 'security',
  SYSTEM = 'system',
  COMMUNITY = 'community',
  TRADING = 'trading'
}

interface UserConnection {
  ws: WebSocket;
  userId: string;
  isAuthenticated: boolean;
  subscriptions: Set<string>;
  lastActivity: Date;
  rateLimit: {
    requests: number;
    resetTime: Date;
  };
  preferences: NotificationPreferences;
}

interface NotificationPreferences {
  enabledTypes: Set<NotificationType>;
  priority: NotificationPriority;
  soundEnabled: boolean;
  vibrationEnabled: boolean;
  quietHours: {
    start: string; // "22:00"
    end: string;   // "08:00"
    timezone: string;
  };
  maxFrequency: {
    perMinute: number;
    perHour: number;
  };
}

class NotificationHandler extends EventEmitter {
  private connections: Map<string, UserConnection> = new Map();
  private redis: Redis;
  private logger: Logger;
  private rateLimiter: any;
  
  // Notification queues by priority
  private urgentQueue: NotificationPayload[] = [];
  private highQueue: NotificationPayload[] = [];
  private normalQueue: NotificationPayload[] = [];
  private lowQueue: NotificationPayload[] = [];
  
  // Processing intervals
  private urgentInterval: NodeJS.Timeout;
  private highInterval: NodeJS.Timeout;
  private normalInterval: NodeJS.Timeout;
  private lowInterval: NodeJS.Timeout;

  constructor(redis: Redis, logger: Logger) {
    super();
    this.redis = redis;
    this.logger = logger;
    this.rateLimiter = rateLimiter;
    this.initializeQueues();
    this.setupCleanupTasks();
  }

  /**
   * Handle new WebSocket connection
   */
  async handleConnection(ws: WebSocket, userId: string): Promise<void> {
    try {
      // Authenticate user
      const authResult = await authenticateWsUser(ws, userId);
      if (!authResult.success) {
        ws.close(1008, 'Authentication failed');
        return;
      }

      // Load user preferences
      const preferences = await this.loadUserPreferences(userId);
      
      // Create connection object
      const connection: UserConnection = {
        ws,
        userId,
        isAuthenticated: true,
        subscriptions: new Set(),
        lastActivity: new Date(),
        rateLimit: {
          requests: 0,
          resetTime: new Date(Date.now() + 60000) // 1 minute from now
        },
        preferences
      };

      this.connections.set(userId, connection);
      
      // Setup WebSocket event handlers
      this.setupWebSocketHandlers(ws, userId);
      
      // Send welcome notification
      await this.sendWelcomeNotification(userId);
      
      // Send pending notifications
      await this.sendPendingNotifications(userId);
      
      this.logger.info(`User ${userId} connected to notification handler`);
      
    } catch (error) {
      this.logger.error('Error handling connection:', error);
      ws.close(1011, 'Internal server error');
    }
  }

  /**
   * Setup WebSocket event handlers for a connection
   */
  private setupWebSocketHandlers(ws: WebSocket, userId: string): void {
    ws.on('message', async (data) => {
      try {
        await this.handleMessage(userId, data);
      } catch (error) {
        this.logger.error(`Error handling message from ${userId}:`, error);
      }
    });

    ws.on('close', () => {
      this.handleDisconnection(userId);
    });

    ws.on('error', (error) => {
      this.logger.error(`WebSocket error for user ${userId}:`, error);
      this.handleDisconnection(userId);
    });

    // Heartbeat mechanism
    ws.on('pong', () => {
      const connection = this.connections.get(userId);
      if (connection) {
        connection.lastActivity = new Date();
      }
    });
  }

  /**
   * Handle incoming WebSocket messages
   */
  private async handleMessage(userId: string, data: any): Promise<void> {
    const connection = this.connections.get(userId);
    if (!connection) return;

    // Rate limiting
    if (!this.checkRateLimit(connection)) {
      connection.ws.send(JSON.stringify({
        type: 'error',
        message: 'Rate limit exceeded'
      }));
      return;
    }

    try {
      const message = JSON.parse(data.toString());
      const validatedMessage = validateNotificationData(message);

      switch (validatedMessage.action) {
        case 'subscribe':
          await this.handleSubscription(userId, validatedMessage.channels);
          break;
          
        case 'unsubscribe':
          await this.handleUnsubscription(userId, validatedMessage.channels);
          break;
          
        case 'update_preferences':
          await this.updateUserPreferences(userId, validatedMessage.preferences);
          break;
          
        case 'mark_read':
          await this.markNotificationsRead(userId, validatedMessage.notificationIds);
          break;
          
        case 'get_history':
          await this.sendNotificationHistory(userId, validatedMessage.params);
          break;
          
        case 'ping':
          connection.ws.send(JSON.stringify({ type: 'pong', timestamp: Date.now() }));
          break;
          
        default:
          this.logger.warn(`Unknown action from user ${userId}:`, validatedMessage.action);
      }
      
    } catch (error) {
      this.logger.error(`Error processing message from ${userId}:`, error);
      connection.ws.send(JSON.stringify({
        type: 'error',
        message: 'Invalid message format'
      }));
    }
  }

  /**
   * Send notification to specific user
   */
  async sendNotification(notification: NotificationPayload): Promise<boolean> {
    try {
      // Validate notification data
      const validatedNotification = validateNotificationData(notification);
      
      // Check if user is connected
      const connection = this.connections.get(notification.userId);
      
      if (connection && connection.ws.readyState === WebSocket.OPEN) {
        // Check user preferences
        if (!this.shouldSendNotification(connection, validatedNotification)) {
          return false;
        }
        
        // Check quiet hours
        if (this.isQuietHours(connection.preferences)) {
          await this.queueNotificationForLater(validatedNotification);
          return true;
        }
        
        // Send immediately
        await this.sendNotificationImmediate(connection, validatedNotification);
        
        // Store in history
        await this.storeNotificationHistory(validatedNotification);
        
        return true;
      } else {
        // User not connected, queue for later
        await this.queueNotificationForLater(validatedNotification);
        
        // Send push notification if enabled
        await this.sendPushNotification(validatedNotification);
        
        return true;
      }
      
    } catch (error) {
      this.logger.error('Error sending notification:', error);
      return false;
    }
  }

  /**
   * Broadcast notification to multiple users
   */
  async broadcastNotification(userIds: string[], notification: Omit<NotificationPayload, 'userId'>): Promise<void> {
    const tasks = userIds.map(userId => 
      this.sendNotification({ ...notification, userId })
    );
    
    await Promise.allSettled(tasks);
  }

  /**
   * Send mining reward notification
   */
  async sendMiningRewardNotification(userId: string, amount: number, multiplier: number): Promise<void> {
    const notification: NotificationPayload = {
      id: `mining_${Date.now()}_${userId}`,
      type: NotificationType.MINING_REWARD,
      userId,
      title: '🎉 Mining Reward Earned!',
      message: `You earned ${amount.toFixed(4)} $FIN with ${multiplier}x multiplier!`,
      data: {
        amount,
        multiplier,
        timestamp: new Date().toISOString()
      },
      priority: NotificationPriority.NORMAL,
      category: NotificationCategory.REWARDS,
      timestamp: new Date(),
      soundEnabled: true,
      vibrationEnabled: false,
      actionUrl: '/mining/dashboard'
    };

    await this.sendNotification(notification);
  }

  /**
   * Send XP level up notification
   */
  async sendLevelUpNotification(userId: string, newLevel: number, xpGained: number): Promise<void> {
    const notification: NotificationPayload = {
      id: `levelup_${Date.now()}_${userId}`,
      type: NotificationType.LEVEL_UP,
      userId,
      title: '🎊 Level Up!',
      message: `Congratulations! You've reached Level ${newLevel}! (+${xpGained} XP)`,
      data: {
        newLevel,
        xpGained,
        unlockedFeatures: await this.getUnlockedFeatures(newLevel)
      },
      priority: NotificationPriority.HIGH,
      category: NotificationCategory.REWARDS,
      timestamp: new Date(),
      soundEnabled: true,
      vibrationEnabled: true,
      actionUrl: '/profile/progress'
    };

    await this.sendNotification(notification);
  }

  /**
   * Send referral success notification
   */
  async sendReferralNotification(userId: string, referralName: string, rpGained: number): Promise<void> {
    const notification: NotificationPayload = {
      id: `referral_${Date.now()}_${userId}`,
      type: NotificationType.NEW_REFERRAL,
      userId,
      title: '👥 New Referral Success!',
      message: `${referralName} joined your network! You earned ${rpGained} RP points.`,
      data: {
        referralName,
        rpGained,
        networkSize: await this.getUserNetworkSize(userId)
      },
      priority: NotificationPriority.HIGH,
      category: NotificationCategory.SOCIAL,
      timestamp: new Date(),
      soundEnabled: true,
      vibrationEnabled: true,
      actionUrl: '/referrals/network'
    };

    await this.sendNotification(notification);
  }

  /**
   * Send security alert notification
   */
  async sendSecurityAlert(userId: string, alertType: string, details: any): Promise<void> {
    const notification: NotificationPayload = {
      id: `security_${Date.now()}_${userId}`,
      type: NotificationType.SECURITY_ALERT,
      userId,
      title: '🔒 Security Alert',
      message: `Suspicious activity detected: ${alertType}`,
      data: {
        alertType,
        details: encryptSensitiveData(JSON.stringify(details)),
        timestamp: new Date().toISOString(),
        actionRequired: true
      },
      priority: NotificationPriority.CRITICAL,
      category: NotificationCategory.SECURITY,
      timestamp: new Date(),
      soundEnabled: true,
      vibrationEnabled: true,
      actionUrl: '/security/alerts'
    };

    await this.sendNotification(notification);
  }

  /**
   * Send viral content notification
   */
  async sendViralContentNotification(userId: string, platform: string, views: number, xpBonus: number): Promise<void> {
    const notification: NotificationPayload = {
      id: `viral_${Date.now()}_${userId}`,
      type: NotificationType.VIRAL_CONTENT,
      userId,
      title: '🔥 Viral Content!',
      message: `Your ${platform} post went viral! ${views.toLocaleString()} views, +${xpBonus} XP bonus!`,
      data: {
        platform,
        views,
        xpBonus,
        viralThreshold: 1000
      },
      priority: NotificationPriority.HIGH,
      category: NotificationCategory.SOCIAL,
      timestamp: new Date(),
      soundEnabled: true,
      vibrationEnabled: true,
      actionUrl: '/social/analytics'
    };

    await this.sendNotification(notification);
  }

  /**
   * Handle user disconnection
   */
  private handleDisconnection(userId: string): void {
    const connection = this.connections.get(userId);
    if (connection) {
      this.connections.delete(userId);
      this.logger.info(`User ${userId} disconnected from notification handler`);
    }
  }

  /**
   * Initialize notification queues with different processing intervals
   */
  private initializeQueues(): void {
    // Process urgent notifications immediately
    this.urgentInterval = setInterval(() => {
      this.processQueue(this.urgentQueue, 'urgent');
    }, 100);

    // Process high priority every 500ms
    this.highInterval = setInterval(() => {
      this.processQueue(this.highQueue, 'high');
    }, 500);

    // Process normal priority every 2 seconds
    this.normalInterval = setInterval(() => {
      this.processQueue(this.normalQueue, 'normal');
    }, 2000);

    // Process low priority every 10 seconds
    this.lowInterval = setInterval(() => {
      this.processQueue(this.lowQueue, 'low');
    }, 10000);
  }

  /**
   * Process notification queue by priority
   */
  private async processQueue(queue: NotificationPayload[], priority: string): Promise<void> {
    if (queue.length === 0) return;

    const batch = queue.splice(0, priority === 'urgent' ? 10 : 5);
    
    for (const notification of batch) {
      try {
        await this.sendNotification(notification);
      } catch (error) {
        this.logger.error(`Error processing ${priority} notification:`, error);
        // Re-queue failed notifications with lower priority
        if (priority !== 'low') {
          this.lowQueue.push(notification);
        }
      }
    }
  }

  /**
   * Check if notification should be sent based on user preferences
   */
  private shouldSendNotification(connection: UserConnection, notification: NotificationPayload): boolean {
    const { preferences } = connection;
    
    // Check if notification type is enabled
    if (!preferences.enabledTypes.has(notification.type)) {
      return false;
    }
    
    // Check priority threshold
    const priorityLevels = {
      [NotificationPriority.LOW]: 1,
      [NotificationPriority.NORMAL]: 2,
      [NotificationPriority.HIGH]: 3,
      [NotificationPriority.URGENT]: 4,
      [NotificationPriority.CRITICAL]: 5
    };
    
    if (priorityLevels[notification.priority] < priorityLevels[preferences.priority]) {
      return false;
    }
    
    return true;
  }

  /**
   * Check if current time is within quiet hours
   */
  private isQuietHours(preferences: NotificationPreferences): boolean {
    const now = new Date();
    const currentTime = now.toLocaleTimeString('en-US', { 
      hour12: false, 
      hour: '2-digit', 
      minute: '2-digit',
      timeZone: preferences.quietHours.timezone 
    });
    
    const { start, end } = preferences.quietHours;
    
    // Handle quiet hours that span midnight
    if (start > end) {
      return currentTime >= start || currentTime <= end;
    }
    
    return currentTime >= start && currentTime <= end;
  }

  /**
   * Send notification immediately to connected user
   */
  private async sendNotificationImmediate(connection: UserConnection, notification: NotificationPayload): Promise<void> {
    const payload = {
      type: 'notification',
      data: {
        ...notification,
        encryptedData: notification.data ? encryptSensitiveData(JSON.stringify(notification.data)) : undefined
      }
    };

    connection.ws.send(JSON.stringify(payload));
    
    // Track delivery
    await this.redis.incr(`notification_delivered:${notification.userId}:${new Date().toDateString()}`);
  }

  /**
   * Store notification for later delivery
   */
  private async queueNotificationForLater(notification: NotificationPayload): Promise<void> {
    const queueKey = `notification_queue:${notification.userId}`;
    await this.redis.lpush(queueKey, JSON.stringify(notification));
    await this.redis.expire(queueKey, 86400); // 24 hours
  }

  /**
   * Load user notification preferences
   */
  private async loadUserPreferences(userId: string): Promise<NotificationPreferences> {
    try {
      const cached = await this.redis.get(`user_preferences:${userId}`);
      
      if (cached) {
        return JSON.parse(cached);
      }
      
      // Default preferences
      const defaultPreferences: NotificationPreferences = {
        enabledTypes: new Set(Object.values(NotificationType)),
        priority: NotificationPriority.NORMAL,
        soundEnabled: true,
        vibrationEnabled: true,
        quietHours: {
          start: '22:00',
          end: '08:00',
          timezone: 'Asia/Jakarta'
        },
        maxFrequency: {
          perMinute: 5,
          perHour: 50
        }
      };
      
      await this.redis.setex(
        `user_preferences:${userId}`, 
        3600, 
        JSON.stringify(defaultPreferences)
      );
      
      return defaultPreferences;
      
    } catch (error) {
      this.logger.error(`Error loading preferences for user ${userId}:`, error);
      throw error;
    }
  }

  /**
   * Setup cleanup tasks
   */
  private setupCleanupTasks(): void {
    // Clean up stale connections every 5 minutes
    setInterval(() => {
      this.cleanupStaleConnections();
    }, 300000);

    // Clean up old notifications every hour
    setInterval(() => {
      this.cleanupOldNotifications();
    }, 3600000);
  }

  /**
   * Clean up stale WebSocket connections
   */
  private cleanupStaleConnections(): void {
    const now = new Date();
    const staleThreshold = 5 * 60 * 1000; // 5 minutes

    for (const [userId, connection] of this.connections.entries()) {
      if (now.getTime() - connection.lastActivity.getTime() > staleThreshold) {
        if (connection.ws.readyState !== WebSocket.OPEN) {
          this.connections.delete(userId);
          this.logger.info(`Cleaned up stale connection for user ${userId}`);
        } else {
          // Send ping to check if connection is still alive
          connection.ws.ping();
        }
      }
    }
  }

  /**
   * Clean up old notifications from Redis
   */
  private async cleanupOldNotifications(): Promise<void> {
    try {
      const keys = await this.redis.keys('notification_queue:*');
      const expiredKeys = [];

      for (const key of keys) {
        const ttl = await this.redis.ttl(key);
        if (ttl <= 0) {
          expiredKeys.push(key);
        }
      }

      if (expiredKeys.length > 0) {
        await this.redis.del(...expiredKeys);
        this.logger.info(`Cleaned up ${expiredKeys.length} expired notification queues`);
      }
    } catch (error) {
      this.logger.error('Error cleaning up old notifications:', error);
    }
  }

  /**
   * Check rate limiting for user connection
   */
  private checkRateLimit(connection: UserConnection): boolean {
    const now = new Date();
    
    if (now > connection.rateLimit.resetTime) {
      connection.rateLimit.requests = 0;
      connection.rateLimit.resetTime = new Date(now.getTime() + 60000);
    }
    
    if (connection.rateLimit.requests >= 60) { // 60 requests per minute
      return false;
    }
    
    connection.rateLimit.requests++;
    return true;
  }

  /**
   * Additional helper methods
   */
  private async getUnlockedFeatures(level: number): Promise<string[]> {
    // Implementation based on XP level system from whitepaper
    const features = [];
    if (level >= 11) features.push('Special Cards Access');
    if (level >= 26) features.push('Guild Leadership');
    if (level >= 51) features.push('Creator Monetization');
    if (level >= 76) features.push('Exclusive Events');
    if (level >= 101) features.push('DAO Governance');
    return features;
  }

  private async getUserNetworkSize(userId: string): Promise<number> {
    // Get network size from Redis or database
    const networkData = await this.redis.get(`user_network:${userId}`);
    return networkData ? JSON.parse(networkData).size : 0;
  }

  private async sendWelcomeNotification(userId: string): Promise<void> {
    const notification: NotificationPayload = {
      id: `welcome_${Date.now()}_${userId}`,
      type: NotificationType.SYSTEM_MAINTENANCE,
      userId,
      title: '🎉 Welcome to Finova Network!',
      message: 'Start mining, earn XP, and build your referral network!',
      data: { isWelcome: true },
      priority: NotificationPriority.NORMAL,
      category: NotificationCategory.SYSTEM,
      timestamp: new Date(),
      soundEnabled: true,
      vibrationEnabled: false
    };

    await this.sendNotification(notification);
  }

  private async sendPendingNotifications(userId: string): Promise<void> {
    const queueKey = `notification_queue:${userId}`;
    const notifications = await this.redis.lrange(queueKey, 0, -1);
    
    for (const notificationData of notifications) {
      try {
        const notification = JSON.parse(notificationData);
        await this.sendNotification(notification);
      } catch (error) {
        this.logger.error('Error sending pending notification:', error);
      }
    }
    
    // Clear the queue
    await this.redis.del(queueKey);
  }

  private async handleSubscription(userId: string, channels: string[]): Promise<void> {
    const connection = this.connections.get(userId);
    if (!connection) return;

    for (const channel of channels) {
      connection.subscriptions.add(channel);
    }

    connection.ws.send(JSON.stringify({
      type: 'subscription_success',
      channels
    }));
  }

  private async handleUnsubscription(userId: string, channels: string[]): Promise<void> {
    const connection = this.connections.get(userId);
    if (!connection) return;

    for (const channel of channels) {
      connection.subscriptions.delete(channel);
    }

    connection.ws.send(JSON.stringify({
      type: 'unsubscription_success',
      channels
    }));
  }

  private async updateUserPreferences(userId: string, preferences: Partial<NotificationPreferences>): Promise<void> {
    const connection = this.connections.get(userId);
    if (!connection) return;

    // Update connection preferences
    Object.assign(connection.preferences, preferences);

    // Update Redis cache
    await this.redis.setex(
      `user_preferences:${userId}`,
      3600,
      JSON.stringify(connection.preferences)
    );

    connection.ws.send(JSON.stringify({
      type: 'preferences_updated',
      preferences: connection.preferences
    }));
  }

  private async markNotificationsRead(userId: string, notificationIds: string[]): Promise<void> {
    // Mark notifications as read in database/Redis
    for (const id of notificationIds) {
      await this.redis.setex(`notification_read:${userId}:${id}`, 86400, '1');
    }

    const connection = this.connections.get(userId);
    if (connection) {
      connection.ws.send(JSON.stringify({
        type: 'notifications_marked_read',
        notificationIds
      }));
    }
  }

  private async sendNotificationHistory(userId: string, params: any): Promise<void> {
    const connection = this.connections.get(userId);
    if (!connection) return;

    // Get notification history from database
    const historyKey = `notification_history:${userId}`;
    const history = await this.redis.lrange(historyKey, 0, params.limit || 50);

    connection.ws.send(JSON.stringify({
      type: 'notification_history',
      notifications: history.map(h => JSON.parse(h))
    }));
  }

  private async storeNotificationHistory(notification: NotificationPayload): Promise<void> {
    const historyKey = `notification_history:${notification.userId}`;
    await this.redis.lpush(historyKey, JSON.stringify(notification));
    await this.redis.ltrim(historyKey, 0, 999); // Keep last 1000 notifications
    await this.redis.expire(historyKey, 86400 * 30); // 30 days
  }

  private async sendPushNotification(notification: NotificationPayload): Promise<void> {
    // Implementation for mobile push notifications
    // This would integrate with FCM, APNS, etc.
    this.logger.info(`Would send push notification to ${notification.userId}: ${notification.title}`);
  }

  /**
   * Shutdown handler
   */
  async shutdown(): Promise<void> {
    // Clear intervals
    clearInterval(this.urgentInterval);
    clearInterval(this.highInterval);
    clearInterval(this.normalInterval);
    clearInterval(this.lowInterval);

    // Close all connections
    for (const [userId, connection] of this.connections.entries()) {
      connection.ws.close(1001, 'Server shutting down');
    }

    this.connections.clear();
    this.logger.info('Notification handler shutdown complete');
  }
}

export default NotificationHandler;
export { 
  NotificationPayload, 
  NotificationType, 
  NotificationPriority, 
  NotificationCategory,
  UserConnection,
  NotificationPreferences
};
