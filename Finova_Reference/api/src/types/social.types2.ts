/**
 * Finova Network - Social Integration Types
 * Enterprise-grade TypeScript definitions for social media platform integrations
 * Supports: Instagram, TikTok, YouTube, Facebook, X (Twitter)
 */

// ========================================
// CORE SOCIAL PLATFORM ENUMS
// ========================================

export enum SocialPlatform {
  INSTAGRAM = 'instagram',
  TIKTOK = 'tiktok',
  YOUTUBE = 'youtube',
  FACEBOOK = 'facebook',
  X = 'x', // formerly Twitter
  FINOVA_APP = 'finova_app'
}

export enum ContentType {
  TEXT_POST = 'text_post',
  IMAGE_POST = 'image_post',
  VIDEO_POST = 'video_post',
  STORY = 'story',
  REEL = 'reel',
  SHORT = 'short',
  COMMENT = 'comment',
  LIKE = 'like',
  SHARE = 'share',
  FOLLOW = 'follow'
}

export enum ContentStatus {
  PENDING = 'pending',
  PROCESSING = 'processing',
  APPROVED = 'approved',
  REJECTED = 'rejected',
  VIRAL = 'viral', // 1K+ views
  SUSPENDED = 'suspended'
}

export enum EngagementType {
  LIKE = 'like',
  COMMENT = 'comment',
  SHARE = 'share',
  SAVE = 'save',
  FOLLOW = 'follow',
  MENTION = 'mention',
  TAG = 'tag',
  REACTION = 'reaction'
}

// ========================================
// SOCIAL ACCOUNT & AUTHENTICATION
// ========================================

export interface SocialAccountConnection {
  id: string;
  userId: string;
  platform: SocialPlatform;
  platformUserId: string;
  username: string;
  displayName: string;
  profilePictureUrl?: string;
  followerCount?: number;
  followingCount?: number;
  isVerified: boolean;
  accessToken: string; // Encrypted
  refreshToken?: string; // Encrypted
  tokenExpiry: Date;
  scopes: string[];
  connectionStatus: 'active' | 'expired' | 'revoked' | 'suspended';
  lastSyncAt: Date;
  createdAt: Date;
  updatedAt: Date;
}

export interface SocialAuthRequest {
  platform: SocialPlatform;
  authCode: string;
  redirectUri: string;
  state?: string;
}

export interface SocialAuthResponse {
  success: boolean;
  connection?: SocialAccountConnection;
  error?: string;
  requiresReauth?: boolean;
}

// ========================================
// CONTENT MANAGEMENT
// ========================================

export interface SocialContent {
  id: string;
  userId: string;
  platform: SocialPlatform;
  contentType: ContentType;
  platformContentId: string;
  title?: string;
  description?: string;
  content: string;
  mediaUrls: string[];
  thumbnailUrl?: string;
  hashtags: string[];
  mentions: string[];
  location?: GeoLocation;
  publishedAt: Date;
  status: ContentStatus;
  
  // Engagement metrics
  viewCount: number;
  likeCount: number;
  commentCount: number;
  shareCount: number;
  saveCount: number;
  
  // Quality assessment
  qualityScore: number; // 0.5 - 2.0
  originalityScore: number; // 0.0 - 1.0
  engagementRate: number;
  viralityScore: number;
  brandSafetyScore: number;
  aiGeneratedProbability: number;
  
  // Finova rewards
  xpAwarded: number;
  finEarned: number;
  rpGenerated: number;
  bonusMultipliers: ContentBonus[];
  
  // Metadata
  processingStatus: 'pending' | 'analyzing' | 'completed' | 'failed';
  lastUpdated: Date;
  syncedAt: Date;
  createdAt: Date;
}

export interface ContentBonus {
  type: 'viral' | 'quality' | 'engagement' | 'originality' | 'streak';
  multiplier: number;
  reason: string;
  appliedAt: Date;
}

export interface GeoLocation {
  latitude: number;
  longitude: number;
  city?: string;
  country?: string;
  timezone?: string;
}

// ========================================
// ENGAGEMENT TRACKING
// ========================================

export interface SocialEngagement {
  id: string;
  userId: string;
  contentId: string;
  platform: SocialPlatform;
  engagementType: EngagementType;
  targetUserId?: string; // For follows, mentions
  targetContentId?: string; // For comments, shares
  content?: string; // Comment text, share message
  timestamp: Date;
  
  // Rewards calculation
  xpValue: number;
  qualityMultiplier: number;
  platformBonus: number;
  streakBonus: number;
  finalXpAwarded: number;
  
  // Quality metrics
  sentimentScore: number; // -1 to 1
  toxicityScore: number; // 0 to 1
  spamProbability: number; // 0 to 1
  authenticityScore: number; // 0 to 1
  
  // Metadata
  processed: boolean;
  processedAt?: Date;
  createdAt: Date;
}

export interface EngagementSummary {
  userId: string;
  platform: SocialPlatform;
  date: string; // YYYY-MM-DD
  totalEngagements: number;
  engagementsByType: Record<EngagementType, number>;
  totalXpEarned: number;
  qualityScore: number;
  streakDays: number;
  rankPosition?: number;
}

// ========================================
// PLATFORM-SPECIFIC CONFIGURATIONS
// ========================================

export interface PlatformConfig {
  platform: SocialPlatform;
  isActive: boolean;
  baseXpMultipliers: Record<ContentType, number>;
  platformBonus: number; // 1.0x - 1.4x
  dailyLimits: Record<ContentType, number>;
  qualityThresholds: {
    minimum: number;
    excellent: number;
  };
  viralThresholds: {
    views: number;
    engagement: number;
  };
  apiConfig: {
    baseUrl: string;
    version: string;
    rateLimits: {
      requests: number;
      windowMs: number;
    };
  };
}

export const PLATFORM_CONFIGS: Record<SocialPlatform, Partial<PlatformConfig>> = {
  [SocialPlatform.INSTAGRAM]: {
    platformBonus: 1.2,
    baseXpMultipliers: {
      [ContentType.IMAGE_POST]: 75,
      [ContentType.VIDEO_POST]: 100,
      [ContentType.STORY]: 25,
      [ContentType.REEL]: 150,
      [ContentType.COMMENT]: 25,
      [ContentType.LIKE]: 5,
      [ContentType.SHARE]: 15,
      [ContentType.FOLLOW]: 20,
      [ContentType.TEXT_POST]: 50,
      [ContentType.SHORT]: 100
    },
    viralThresholds: {
      views: 1000,
      engagement: 100
    }
  },
  [SocialPlatform.TIKTOK]: {
    platformBonus: 1.3,
    baseXpMultipliers: {
      [ContentType.VIDEO_POST]: 150,
      [ContentType.SHORT]: 150,
      [ContentType.COMMENT]: 25,
      [ContentType.LIKE]: 5,
      [ContentType.SHARE]: 20,
      [ContentType.FOLLOW]: 20,
      [ContentType.TEXT_POST]: 30,
      [ContentType.IMAGE_POST]: 50,
      [ContentType.STORY]: 25,
      [ContentType.REEL]: 150
    },
    viralThresholds: {
      views: 5000,
      engagement: 200
    }
  },
  [SocialPlatform.YOUTUBE]: {
    platformBonus: 1.4,
    baseXpMultipliers: {
      [ContentType.VIDEO_POST]: 200,
      [ContentType.SHORT]: 100,
      [ContentType.COMMENT]: 30,
      [ContentType.LIKE]: 5,
      [ContentType.SHARE]: 25,
      [ContentType.FOLLOW]: 25,
      [ContentType.TEXT_POST]: 40,
      [ContentType.IMAGE_POST]: 60,
      [ContentType.STORY]: 20,
      [ContentType.REEL]: 120
    },
    viralThresholds: {
      views: 10000,
      engagement: 500
    }
  },
  [SocialPlatform.FACEBOOK]: {
    platformBonus: 1.1,
    baseXpMultipliers: {
      [ContentType.TEXT_POST]: 50,
      [ContentType.IMAGE_POST]: 70,
      [ContentType.VIDEO_POST]: 120,
      [ContentType.STORY]: 25,
      [ContentType.COMMENT]: 25,
      [ContentType.LIKE]: 5,
      [ContentType.SHARE]: 15,
      [ContentType.FOLLOW]: 20,
      [ContentType.REEL]: 100,
      [ContentType.SHORT]: 80
    },
    viralThresholds: {
      views: 2000,
      engagement: 150
    }
  },
  [SocialPlatform.X]: {
    platformBonus: 1.2,
    baseXpMultipliers: {
      [ContentType.TEXT_POST]: 50,
      [ContentType.IMAGE_POST]: 75,
      [ContentType.VIDEO_POST]: 100,
      [ContentType.COMMENT]: 25,
      [ContentType.LIKE]: 5,
      [ContentType.SHARE]: 20,
      [ContentType.FOLLOW]: 20,
      [ContentType.STORY]: 20,
      [ContentType.REEL]: 80,
      [ContentType.SHORT]: 80
    },
    viralThresholds: {
      views: 1000,
      engagement: 100
    }
  },
  [SocialPlatform.FINOVA_APP]: {
    platformBonus: 1.0,
    baseXpMultipliers: {
      [ContentType.TEXT_POST]: 50,
      [ContentType.IMAGE_POST]: 75,
      [ContentType.VIDEO_POST]: 150,
      [ContentType.COMMENT]: 25,
      [ContentType.LIKE]: 5,
      [ContentType.SHARE]: 15,
      [ContentType.FOLLOW]: 20,
      [ContentType.STORY]: 25,
      [ContentType.REEL]: 100,
      [ContentType.SHORT]: 100
    },
    viralThresholds: {
      views: 500,
      engagement: 50
    }
  }
};

// ========================================
// AI QUALITY ASSESSMENT
// ========================================

export interface ContentQualityAnalysis {
  contentId: string;
  platform: SocialPlatform;
  analysisTimestamp: Date;
  
  // Core quality metrics
  overallScore: number; // 0.5 - 2.0 (XP multiplier)
  originalityScore: number; // 0.0 - 1.0
  engagementPrediction: number; // 0.0 - 1.0
  brandSafetyScore: number; // 0.0 - 1.0
  platformRelevance: number; // 0.0 - 1.0
  
  // Detection scores
  aiGeneratedProbability: number; // 0.0 - 1.0
  spamProbability: number; // 0.0 - 1.0
  toxicityScore: number; // 0.0 - 1.0
  clickbaitScore: number; // 0.0 - 1.0
  
  // Content analysis
  languageDetection: string;
  sentimentAnalysis: {
    score: number; // -1 to 1
    magnitude: number; // 0 to 1
    emotion: string;
  };
  topicClassification: string[];
  keywordExtraction: string[];
  
  // Metadata
  modelVersion: string;
  processingTimeMs: number;
  confidence: number; // 0.0 - 1.0
}

export interface QualityAssessmentRequest {
  contentId: string;
  platform: SocialPlatform;
  contentType: ContentType;
  textContent?: string;
  mediaUrls?: string[];
  metadata?: Record<string, any>;
  priority?: 'low' | 'normal' | 'high';
}

// ========================================
// SOCIAL ANALYTICS & INSIGHTS
// ========================================

export interface SocialAnalytics {
  userId: string;
  period: 'daily' | 'weekly' | 'monthly';
  startDate: Date;
  endDate: Date;
  
  // Platform breakdown
  platformStats: Record<SocialPlatform, PlatformStats>;
  
  // Overall metrics
  totalContent: number;
  totalEngagements: number;
  totalXpEarned: number;
  totalFinEarned: number;
  averageQualityScore: number;
  streakDays: number;
  
  // Growth metrics
  followerGrowth: number;
  engagementGrowth: number;
  qualityImprovement: number;
  
  // Performance insights
  bestPerformingContent: SocialContent[];
  topHashtags: string[];
  optimalPostingTimes: TimeSlot[];
  recommendedActions: string[];
  
  createdAt: Date;
}

export interface PlatformStats {
  platform: SocialPlatform;
  contentCount: number;
  engagementCount: number;
  xpEarned: number;
  averageQualityScore: number;
  followerCount: number;
  viralContent: number;
  topContentTypes: ContentType[];
}

export interface TimeSlot {
  hour: number; // 0-23
  dayOfWeek: number; // 0-6
  engagementRate: number;
  optimalScore: number;
}

// ========================================
// REAL-TIME SYNCHRONIZATION
// ========================================

export interface SyncRequest {
  userId: string;
  platforms: SocialPlatform[];
  syncType: 'full' | 'incremental' | 'content_only' | 'engagement_only';
  since?: Date;
  priority?: 'low' | 'normal' | 'high';
}

export interface SyncResponse {
  requestId: string;
  userId: string;
  status: 'pending' | 'processing' | 'completed' | 'failed';
  platformResults: Record<SocialPlatform, PlatformSyncResult>;
  totalProcessed: number;
  totalErrors: number;
  startedAt: Date;
  completedAt?: Date;
  nextSyncAt?: Date;
}

export interface PlatformSyncResult {
  platform: SocialPlatform;
  success: boolean;
  contentSynced: number;
  engagementsSynced: number;
  xpAwarded: number;
  errors: string[];
  lastSyncedAt: Date;
}

// ========================================
// WEBHOOK & REAL-TIME EVENTS
// ========================================

export interface SocialWebhookEvent {
  id: string;
  platform: SocialPlatform;
  eventType: 'content_posted' | 'engagement_received' | 'account_updated' | 'milestone_reached';
  userId: string;
  data: Record<string, any>;
  timestamp: Date;
  processed: boolean;
  retryCount: number;
}

export interface SocialNotification {
  id: string;
  userId: string;
  type: 'xp_earned' | 'milestone' | 'viral_content' | 'quality_improvement' | 'streak_bonus';
  title: string;
  message: string;
  data: Record<string, any>;
  read: boolean;
  createdAt: Date;
}

// ========================================
// CONTENT SCHEDULING & AUTOMATION
// ========================================

export interface ScheduledContent {
  id: string;
  userId: string;
  platforms: SocialPlatform[];
  contentType: ContentType;
  content: string;
  mediaUrls?: string[];
  hashtags: string[];
  scheduledFor: Date;
  timezone: string;
  status: 'scheduled' | 'publishing' | 'published' | 'failed' | 'cancelled';
  publishResults?: Record<SocialPlatform, {
    success: boolean;
    contentId?: string;
    error?: string;
  }>;
  createdAt: Date;
  updatedAt: Date;
}

// ========================================
// ERROR HANDLING & VALIDATION
// ========================================

export interface SocialError {
  code: string;
  message: string;
  platform?: SocialPlatform;
  contentId?: string;
  userId?: string;
  details?: Record<string, any>;
  timestamp: Date;
  resolved: boolean;
}

export const SOCIAL_ERROR_CODES = {
  // Authentication errors
  INVALID_TOKEN: 'INVALID_TOKEN',
  EXPIRED_TOKEN: 'EXPIRED_TOKEN',
  INSUFFICIENT_PERMISSIONS: 'INSUFFICIENT_PERMISSIONS',
  RATE_LIMIT_EXCEEDED: 'RATE_LIMIT_EXCEEDED',
  
  // Content errors
  CONTENT_NOT_FOUND: 'CONTENT_NOT_FOUND',
  CONTENT_BLOCKED: 'CONTENT_BLOCKED',
  INVALID_CONTENT_TYPE: 'INVALID_CONTENT_TYPE',
  QUALITY_THRESHOLD_NOT_MET: 'QUALITY_THRESHOLD_NOT_MET',
  
  // Platform errors
  PLATFORM_UNAVAILABLE: 'PLATFORM_UNAVAILABLE',
  API_ERROR: 'API_ERROR',
  SYNC_FAILED: 'SYNC_FAILED',
  WEBHOOK_FAILED: 'WEBHOOK_FAILED',
  
  // Processing errors
  ANALYSIS_FAILED: 'ANALYSIS_FAILED',
  MEDIA_PROCESSING_FAILED: 'MEDIA_PROCESSING_FAILED',
  DATABASE_ERROR: 'DATABASE_ERROR'
} as const;

export type SocialErrorCode = typeof SOCIAL_ERROR_CODES[keyof typeof SOCIAL_ERROR_CODES];

// ========================================
// UTILITY TYPES
// ========================================

export interface PaginatedResult<T> {
  data: T[];
  total: number;
  page: number;
  limit: number;
  hasNext: boolean;
  hasPrev: boolean;
}

export interface SocialAPIResponse<T = any> {
  success: boolean;
  data?: T;
  error?: SocialError;
  timestamp: Date;
  requestId?: string;
}

export interface BulkOperation<T> {
  items: T[];
  batchSize: number;
  parallelLimit: number;
  onProgress?: (completed: number, total: number) => void;
  onError?: (error: SocialError, item: T) => void;
}

// ========================================
// EXPORT COLLECTIONS
// ========================================

export * from './mining.types';
export * from './user.types';
export * from './referral.types';
