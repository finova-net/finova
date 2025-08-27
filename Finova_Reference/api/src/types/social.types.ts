/**
 * Finova Network - Social Integration Types
 *
 * Enterprise-grade TypeScript definitions for all aspects of social media integration,
 * content analysis, gamification, and reward systems. This file serves as the
 * single source of truth for all data types related to social interactions.
 *
 * @version 4.0.1
 * @date 2025-08-23
 * @author Finova Network Super App Developer
 */

import { Types } from 'mongoose';

// ============================================================================
// CORE ENUMS
// ============================================================================

/** Supported social media platforms. */
export enum SocialPlatform {
  INSTAGRAM = 'instagram',
  TIKTOK = 'tiktok',
  YOUTUBE = 'youtube',
  FACEBOOK = 'facebook',
  X = 'x', // Formerly Twitter
  LINKEDIN = 'linkedin',
  THREADS = 'threads',
  SNAPCHAT = 'snapchat',
  FINOVA_NATIVE = 'finova_native', // Content created within the Finova app
}

/** Types of content that can be tracked. */
export enum ContentType {
  TEXT_POST = 'text_post',
  IMAGE_POST = 'image_post',
  VIDEO_POST = 'video_post',
  STORY = 'story',
  REEL = 'reel',
  SHORT_VIDEO = 'short_video',
  LIVE_STREAM = 'live_stream',
  COMMENT = 'comment',
  REPLY = 'reply',
  SHARE = 'share',
  REACTION = 'reaction',
}

/** Types of user activities that can be tracked. */
export enum ActivityType {
  POST = 'post',
  COMMENT = 'comment',
  LIKE = 'like',
  SHARE = 'share',
  FOLLOW = 'follow',
  SUBSCRIBE = 'subscribe',
  SAVE = 'save',
  MENTION = 'mention',
  TAG = 'tag',
  VIEW = 'view',
}

/** The processing status of a piece of content. */
export enum ContentStatus {
  DRAFT = 'draft',
  PENDING_REVIEW = 'pending_review', // Awaiting AI or manual review
  APPROVED = 'approved', // Passed review, ready for rewards
  PUBLISHED = 'published', // Has been published on the platform
  REJECTED = 'rejected', // Rejected due to low quality or violations
  FLAGGED = 'flagged', // Marked for manual review
  ARCHIVED = 'archived', // Archived by the user or system
  PROCESSING_REWARDS = 'processing_rewards', // In the process of calculating rewards
  REWARDED = 'rewarded', // Rewards have been distributed
}

/** The quality tier of content as determined by AI. */
export enum QualityTier {
  LOW = 'low', // 0.5x multiplier
  MEDIUM = 'medium', // 1.0x multiplier
  HIGH = 'high', // 1.5x multiplier
  PREMIUM = 'premium', // 2.0x multiplier
  VIRAL = 'viral', // Additional bonus multiplier
}

// ============================================================================
// BASE INTERFACES
// ============================================================================

/** Standard timestamps for documents. */
export interface BaseTimestamp {
  createdAt: Date;
  updatedAt: Date;
  deletedAt?: Date;
}

/** A concise reference to a Finova user. */
export interface UserReference {
  userId: Types.ObjectId;
  username: string;
  displayName: string;
  avatarUrl?: string;
  isVerified: boolean;
  level: number;
}

/** Metadata for a connection to a social platform. */
export interface PlatformConnection extends BaseTimestamp {
  _id: Types.ObjectId;
  userId: Types.ObjectId;
  platform: SocialPlatform;
  platformUserId: string;
  username: string;
  accessToken: string; // Should be encrypted at rest
  refreshToken?: string; // Should be encrypted at rest
  tokenExpiresAt: Date;
  permissions: string[];
  lastSyncAt: Date;
  isActive: boolean;
  connectionStatus: 'active' | 'expired' | 'revoked' | 'error';
}

// ============================================================================
// SOCIAL CONTENT & ACTIVITY TYPES
// ============================================================================

/** A comprehensive representation of a piece of social content. */
export interface SocialContent extends BaseTimestamp {
  _id: Types.ObjectId;
  user: UserReference;
  platformConnection: PlatformConnection;
  platform: SocialPlatform;
  contentType: ContentType;
  status: ContentStatus;

  // Content Data
  platformContentId: string;
  platformUrl: string;
  text?: string;
  mediaUrls: string[];
  thumbnailUrl?: string;
  duration?: number; // in seconds for videos
  hashtags: string[];
  mentions: string[];

  // Engagement Metrics
  metrics: ContentMetrics;

  // AI Quality Assessment
  qualityAssessment: QualityAssessment;

  // Reward Calculation
  rewards: ContentRewards;

  // Moderation
  moderation?: ModerationInfo;

  // Additional Metadata
  metadata?: Record<string, any>;
}

/** Engagement metrics for a piece of content. */
export interface ContentMetrics {
  views: number;
  likes: number;
  comments: number;
  shares: number;
  saves: number;
  reactions: Record<string, number>; // e.g., { "love": 100, "haha": 50 }
  engagementRate: number;
  reach?: number;
  impressions?: number;
  lastUpdated: Date;
}

/** The result of the AI system's content quality assessment. */
export interface QualityAssessment {
  overallScore: number; // Final score between 0.5 - 2.0, used as a multiplier
  tier: QualityTier;
  factors: {
    originality: number; // 0.0 - 1.0
    engagementPotential: number; // 0.0 - 1.0
    platformRelevance: number; // 0.0 - 1.0
    brandSafety: number; // 0.0 - 1.0
    technicalQuality: number; // Visual/audio quality
  };
  isViral: boolean;
  aiConfidence: number; // AI's confidence level (0.0 - 1.0)
  humanVerified: boolean;
  lastAssessed: Date;
}

/** Details of the rewards earned from a piece of content. */
export interface ContentRewards {
  baseXP: number;
  bonusXP: number;
  totalXP: number;
  xpMultiplierDetails: Record<string, number>; // { quality: 1.5, viral: 0.5, streak: 0.2 }

  baseFIN: number;
  bonusFIN: number;
  totalFIN: number;
  finMultiplierDetails: Record<string, number>;

  rpContribution: number;
  isPaid: boolean;
  paidAt?: Date;
  transactionHash?: string;
}

/** Information related to content moderation. */
export interface ModerationInfo {
  status: 'pending' | 'approved' | 'rejected' | 'escalated';
  flags: ModerationFlag[];
  reviewedBy?: UserReference;
  reviewedAt?: Date;
  notes?: string;
}

/** A moderation flag applied to content or an activity. */
export interface ModerationFlag {
  type:
    | 'spam'
    | 'hate_speech'
    | 'violence'
    | 'nudity'
    | 'copyright'
    | 'misinformation'
    | 'harassment';
  severity: 'low' | 'medium' | 'high' | 'critical';
  source: 'ai' | 'user_report' | 'admin';
  confidence?: number;
}

/** Represents a user engagement activity. */
export interface SocialActivity extends BaseTimestamp {
  _id: Types.ObjectId;
  user: UserReference;
  platform: SocialPlatform;
  activityType: ActivityType;
  targetContentId?: Types.ObjectId;
  targetUserId?: Types.ObjectId;

  // Activity Data
  text?: string; // e.g., comment content

  // Activity Quality Assessment
  botScore: number; // 0.0 (human) - 1.0 (bot)
  sentiment?: 'positive' | 'neutral' | 'negative';

  // Rewards
  xpAwarded: number;
  finAwarded: number;
  rpContribution: number;
}

// ============================================================================
// SOCIAL CAMPAIGN & CHALLENGE TYPES
// ============================================================================

/** Represents a social campaign or challenge. */
export interface SocialCampaign extends BaseTimestamp {
  _id: Types.ObjectId;
  name: string;
  description: string;
  type: 'hashtag_challenge' | 'content_contest' | 'brand_partnership';
  status: 'draft' | 'active' | 'completed' | 'cancelled';
  startDate: Date;
  endDate: Date;

  // Rules & Requirements
  platforms: SocialPlatform[];
  requirements: CampaignRequirement[];
  contentGuidelines: string;

  // Rewards
  rewards: CampaignReward[];

  // Participation
  participants: CampaignParticipant[];
  submissions: Types.ObjectId[]; // Reference to SocialContent

  // Metrics
  metrics: CampaignMetrics;
}

/** A requirement for participating in a campaign. */
export interface CampaignRequirement {
  type: 'use_hashtag' | 'mention_user' | 'min_views' | 'follow_account';
  value: string | number;
  isOptional: boolean;
}

/** The reward structure for campaign winners. */
export interface CampaignReward {
  tier: 'grand_prize' | 'top_10' | 'participation';
  description: string;
  items: {
    type: 'fin' | 'xp' | 'nft' | 'special_card';
    amount: number;
    itemId?: string; // ID for NFT or Special Card
  }[];
}

/** Data for a campaign participant. */
export interface CampaignParticipant {
  user: UserReference;
  joinedAt: Date;
  submissionCount: number;
  isQualified: boolean;
  rank?: number;
}

/** Performance metrics for a campaign. */
export interface CampaignMetrics {
  totalParticipants: number;
  totalSubmissions: number;
  totalViews: number;
  totalEngagement: number;
  averageQualityScore: number;
  roi?: number; // Return on Investment for paid campaigns
}

// ============================================================================
// ANALYTICS & REPORTING TYPES
// ============================================================================

/** A social analytics report for a user. */
export interface SocialAnalyticsReport extends BaseTimestamp {
  _id: Types.ObjectId;
  userId: Types.ObjectId;
  period: 'daily' | 'weekly' | 'monthly';
  startDate: Date;
  endDate: Date;

  // Summary
  overview: AnalyticsOverview;

  // Platform Breakdown
  platformBreakdown: PlatformAnalytics[];

  // Insights & Recommendations
  insights: string[];
  recommendations: AnalyticsRecommendation[];
}

/** A summary of performance metrics in an analytics report. */
export interface AnalyticsOverview {
  totalContent: number;
  totalViews: number;
  totalEngagement: number;
  averageEngagementRate: number;
  averageQualityScore: number;
  followerGrowth: number;
  totalXPEarned: number;
  totalFINEarned: number;
  totalRPContributed: number;
}

/** Detailed analytics for a single platform. */
export interface PlatformAnalytics {
  platform: SocialPlatform;
  contentCount: number;
  engagementCount: number;
  xpEarned: number;
  followerCount: number;
  viralContentCount: number;
  topPerformingContent: {
    contentId: Types.ObjectId;
    platformUrl: string;
    score: number;
  }[];
}

/** An actionable recommendation for the user. */
export interface AnalyticsRecommendation {
  type: 'timing' | 'content' | 'platform' | 'hashtag' | 'quality';
  description: string;
  priority: 'high' | 'medium' | 'low';
  potentialImpact: number; // Estimated performance increase in percent
}

// ============================================================================
// API & UTILITY TYPES
// ============================================================================

/** Standard API response. */
export interface ApiResponse<T = any> {
  success: boolean;
  message: string;
  data?: T;
  error?: {
    code: string;
    details?: string;
  };
  timestamp: Date;
}

/** Structure for paginated results. */
export interface PaginatedResponse<T> {
  data: T[];
  pagination: {
    total: number;
    page: number;
    limit: number;
    totalPages: number;
    hasNext: boolean;
    hasPrev: boolean;
  };
}

/** Standard error codes for the social API. */
export enum SocialApiErrorCode {
  // Authentication & Permissions
  INVALID_TOKEN = 'E401_INVALID_TOKEN',
  EXPIRED_TOKEN = 'E401_EXPIRED_TOKEN',
  INSUFFICIENT_PERMISSIONS = 'E403_INSUFFICIENT_PERMISSIONS',
  ACCOUNT_NOT_CONNECTED = 'E404_ACCOUNT_NOT_CONNECTED',

  // Validation & Input
  VALIDATION_ERROR = 'E400_VALIDATION_ERROR',
  MISSING_PARAMETER = 'E400_MISSING_PARAMETER',

  // External Platforms
  PLATFORM_API_ERROR = 'E502_PLATFORM_API_ERROR',
  RATE_LIMIT_EXCEEDED = 'E429_RATE_LIMIT_EXCEEDED',
  PLATFORM_UNAVAILABLE = 'E503_PLATFORM_UNAVAILABLE',

  // Internal Logic
  CONTENT_NOT_FOUND = 'E404_CONTENT_NOT_FOUND',
  ANALYSIS_FAILED = 'E500_ANALYSIS_FAILED',
  SYNC_JOB_FAILED = 'E500_SYNC_JOB_FAILED',
  DATABASE_ERROR = 'E500_DATABASE_ERROR',
}
