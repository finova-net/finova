/**
 * Finova Network - Social Integration Types
 * Enterprise-grade TypeScript definitions for social media platform integration
 * 
 * @version 3.0.0
 * @date 2025-07-26
 * @author Finova Network Team
 */

import { ObjectId } from 'mongodb';

// ===== CORE ENUMS =====

export enum SocialPlatform {
  INSTAGRAM = 'instagram',
  TIKTOK = 'tiktok',
  YOUTUBE = 'youtube',
  FACEBOOK = 'facebook',
  X = 'x', // Twitter/X
  LINKEDIN = 'linkedin',
  THREADS = 'threads',
  SNAPCHAT = 'snapchat'
}

export enum ContentType {
  POST = 'post',
  STORY = 'story',
  REEL = 'reel',
  VIDEO = 'video',
  SHORT = 'short',
  LIVE = 'live',
  COMMENT = 'comment',
  SHARE = 'share',
  LIKE = 'like',
  FOLLOW = 'follow'
}

export enum ContentStatus {
  PENDING = 'pending',
  PROCESSING = 'processing',
  ANALYZED = 'analyzed',
  REWARDED = 'rewarded',
  REJECTED = 'rejected',
  FLAGGED = 'flagged'
}

export enum QualityTier {
  LOW = 'low',
  MEDIUM = 'medium',
  HIGH = 'high',
  VIRAL = 'viral',
  PREMIUM = 'premium'
}

export enum EngagementAction {
  VIEW = 'view',
  LIKE = 'like',
  COMMENT = 'comment',
  SHARE = 'share',
  SAVE = 'save',
  FOLLOW = 'follow',
  SUBSCRIBE = 'subscribe',
  REACT = 'react'
}

// ===== PLATFORM MULTIPLIERS =====

export interface PlatformMultipliers {
  platform: SocialPlatform;
  xpMultiplier: number;
  miningBonus: number;
  qualityWeight: number;
  engagementWeight: number;
  viralThreshold: number;
}

export const PLATFORM_CONFIG: Record<SocialPlatform, PlatformMultipliers> = {
  [SocialPlatform.TIKTOK]: {
    platform: SocialPlatform.TIKTOK,
    xpMultiplier: 1.3,
    miningBonus: 0.25,
    qualityWeight: 1.4,
    engagementWeight: 1.5,
    viralThreshold: 1000
  },
  [SocialPlatform.YOUTUBE]: {
    platform: SocialPlatform.YOUTUBE,
    xpMultiplier: 1.4,
    miningBonus: 0.3,
    qualityWeight: 1.6,
    engagementWeight: 1.3,
    viralThreshold: 5000
  },
  [SocialPlatform.INSTAGRAM]: {
    platform: SocialPlatform.INSTAGRAM,
    xpMultiplier: 1.2,
    miningBonus: 0.2,
    qualityWeight: 1.3,
    engagementWeight: 1.4,
    viralThreshold: 2000
  },
  [SocialPlatform.X]: {
    platform: SocialPlatform.X,
    xpMultiplier: 1.2,
    miningBonus: 0.15,
    qualityWeight: 1.2,
    engagementWeight: 1.2,
    viralThreshold: 10000
  },
  [SocialPlatform.FACEBOOK]: {
    platform: SocialPlatform.FACEBOOK,
    xpMultiplier: 1.1,
    miningBonus: 0.1,
    qualityWeight: 1.1,
    engagementWeight: 1.1,
    viralThreshold: 1500
  },
  [SocialPlatform.LINKEDIN]: {
    platform: SocialPlatform.LINKEDIN,
    xpMultiplier: 1.15,
    miningBonus: 0.12,
    qualityWeight: 1.5,
    engagementWeight: 1.0,
    viralThreshold: 500
  },
  [SocialPlatform.THREADS]: {
    platform: SocialPlatform.THREADS,
    xpMultiplier: 1.1,
    miningBonus: 0.1,
    qualityWeight: 1.2,
    engagementWeight: 1.3,
    viralThreshold: 1000
  },
  [SocialPlatform.SNAPCHAT]: {
    platform: SocialPlatform.SNAPCHAT,
    xpMultiplier: 1.0,
    miningBonus: 0.05,
    qualityWeight: 1.0,
    engagementWeight: 1.2,
    viralThreshold: 500
  }
};

// ===== SOCIAL ACCOUNT INTEGRATION =====

export interface SocialAccount {
  id: ObjectId;
  userId: ObjectId;
  platform: SocialPlatform;
  platformUserId: string;
  username: string;
  displayName: string;
  profileUrl: string;
  profileImageUrl?: string;
  followerCount: number;
  followingCount: number;
  isVerified: boolean;
  accessToken: string;
  refreshToken?: string;
  tokenExpiresAt: Date;
  lastSyncAt: Date;
  isActive: boolean;
  permissions: string[];
  metadata: Record<string, any>;
  createdAt: Date;
  updatedAt: Date;
}

export interface SocialAccountStats {
  accountId: ObjectId;
  totalPosts: number;
  totalEngagement: number;
  averageEngagement: number;
  viralPosts: number;
  totalXpEarned: number;
  totalFinEarned: number;
  bestPerformingPost?: ObjectId;
  lastAnalysisAt: Date;
  monthlyStats: MonthlyStats[];
}

export interface MonthlyStats {
  month: string; // YYYY-MM format
  posts: number;
  engagement: number;
  xpEarned: number;
  finEarned: number;
  viralCount: number;
}

// ===== CONTENT ANALYSIS =====

export interface SocialContent {
  id: ObjectId;
  userId: ObjectId;
  accountId: ObjectId;
  platform: SocialPlatform;
  contentType: ContentType;
  platformContentId: string;
  url: string;
  caption?: string;
  hashtags: string[];
  mentions: string[];
  mediaUrls: string[];
  thumbnailUrl?: string;
  duration?: number; // for videos in seconds
  engagementMetrics: EngagementMetrics;
  qualityAnalysis: ContentQualityAnalysis;
  rewardCalculation: ContentRewardCalculation;
  status: ContentStatus;
  publishedAt: Date;
  analyzedAt?: Date;
  rewardedAt?: Date;
  metadata: ContentMetadata;
  createdAt: Date;
  updatedAt: Date;
}

export interface EngagementMetrics {
  views: number;
  likes: number;
  comments: number;
  shares: number;
  saves?: number;
  reactions?: Record<string, number>;
  engagementRate: number;
  reach?: number;
  impressions?: number;
  clickThroughRate?: number;
  lastUpdatedAt: Date;
}

export interface ContentQualityAnalysis {
  overallScore: number; // 0.5 - 2.0
  originality: number; // 0-1
  engagementPotential: number; // 0-1
  platformRelevance: number; // 0-1
  brandSafety: number; // 0-1
  humanGenerated: number; // 0-1
  languageQuality: number; // 0-1
  visualQuality?: number; // 0-1 for image/video content
  isViral: boolean;
  qualityTier: QualityTier;
  aiAnalysisDetails: {
    sentiment: 'positive' | 'neutral' | 'negative';
    topics: string[];
    entities: string[];
    language: string;
    readabilityScore?: number;
    toxicityScore: number;
  };
  moderationFlags: string[];
  analyzedAt: Date;
}

export interface ContentRewardCalculation {
  baseXp: number;
  platformMultiplier: number;
  qualityMultiplier: number;
  viralBonus: number;
  streakBonus: number;
  finalXp: number;
  baseFin: number;
  finMultiplier: number;
  finalFin: number;
  rpContribution: number;
  calculatedAt: Date;
}

export interface ContentMetadata {
  sourceMetadata: Record<string, any>;
  extractedText?: string;
  detectedObjects?: string[];
  faces?: number;
  location?: {
    lat: number;
    lng: number;
    name: string;
  };
  musicTrack?: {
    title: string;
    artist: string;
    duration: number;
  };
  collaborators?: string[];
  isSponsored: boolean;
  campaignId?: string;
}

// ===== ENGAGEMENT TRACKING =====

export interface UserEngagement {
  id: ObjectId;
  userId: ObjectId;
  contentId: ObjectId;
  action: EngagementAction;
  platform: SocialPlatform;
  timestamp: Date;
  metadata: {
    reactionType?: string;
    commentText?: string;
    shareMessage?: string;
    [key: string]: any;
  };
}

export interface DailyEngagementSummary {
  userId: ObjectId;
  date: string; // YYYY-MM-DD
  platform: SocialPlatform;
  totalActions: number;
  actionBreakdown: Record<EngagementAction, number>;
  xpEarned: number;
  finEarned: number;
  streakDay: number;
  qualityScore: number;
}

// ===== VIRAL CONTENT TRACKING =====

export interface ViralContent {
  contentId: ObjectId;
  userId: ObjectId;
  platform: SocialPlatform;
  viralScore: number;
  peakEngagement: number;
  viralThresholdReached: Date;
  currentStatus: 'trending' | 'peaked' | 'declining' | 'ended';
  bonusXp: number;
  bonusFin: number;
  shareableBonus: number; // bonus for referral network
  featuredInApp: boolean;
  viralityFactors: {
    shareVelocity: number;
    commentEngagement: number;
    crossPlatformSpread: number;
    influencerShares: number;
  };
}

// ===== SOCIAL CAMPAIGNS =====

export interface SocialCampaign {
  id: ObjectId;
  name: string;
  description: string;
  brandId?: ObjectId;
  platforms: SocialPlatform[];
  startDate: Date;
  endDate: Date;
  budget: number;
  objectives: CampaignObjective[];
  targetAudience: TargetAudience;
  requirements: CampaignRequirements;
  rewards: CampaignRewards;
  participants: ObjectId[];
  status: 'draft' | 'active' | 'paused' | 'completed' | 'cancelled';
  metrics: CampaignMetrics;
  createdAt: Date;
  updatedAt: Date;
}

export interface CampaignObjective {
  type: 'awareness' | 'engagement' | 'conversion' | 'reach';
  target: number;
  current: number;
  weight: number;
}

export interface TargetAudience {
  minLevel: number;
  maxLevel?: number;
  platforms: SocialPlatform[];
  minFollowers?: number;
  countries?: string[];
  languages?: string[];
  interests?: string[];
}

export interface CampaignRequirements {
  minPosts: number;
  requiredHashtags: string[];
  requiredMentions: string[];
  contentGuidelines: string;
  qualityThreshold: number;
  originalContentOnly: boolean;
}

export interface CampaignRewards {
  baseXp: number;
  baseFin: number;
  bonusMultiplier: number;
  specialNfts?: ObjectId[];
  completionBonus: number;
  topPerformerBonus: number;
}

export interface CampaignMetrics {
  totalParticipants: number;
  totalPosts: number;
  totalEngagement: number;
  totalReach: number;
  averageQualityScore: number;
  completionRate: number;
  topPerformers: ObjectId[];
  roi: number;
}

// ===== INFLUENCER PROGRAM =====

export interface InfluencerProfile {
  userId: ObjectId;
  tier: 'micro' | 'macro' | 'mega' | 'celebrity';
  specialties: string[];
  averageEngagementRate: number;
  totalFollowers: number;
  crossPlatformReach: number;
  brandCollaborations: number;
  finpvaScore: number; // Finova Influencer Performance Score
  rates: {
    postRate: number;
    storyRate: number;
    videoRate: number;
    packageDeals: PackageDeal[];
  };
  portfolio: PortfolioItem[];
  verificationStatus: 'pending' | 'verified' | 'rejected';
  isAvailable: boolean;
  nextAvailableDate?: Date;
}

export interface PackageDeal {
  name: string;
  description: string;
  deliverables: string[];
  price: number;
  duration: number;
  platforms: SocialPlatform[];
}

export interface PortfolioItem {
  contentId: ObjectId;
  campaignId?: ObjectId;
  brandName?: string;
  metrics: EngagementMetrics;
  isHighlight: boolean;
}

// ===== SOCIAL ANALYTICS =====

export interface SocialAnalytics {
  userId: ObjectId;
  timeframe: 'daily' | 'weekly' | 'monthly' | 'yearly';
  startDate: Date;
  endDate: Date;
  platformBreakdown: PlatformAnalytics[];
  topContent: ObjectId[];
  growthMetrics: GrowthMetrics;
  earningsBreakdown: EarningsBreakdown;
  engagementTrends: EngagementTrend[];
  audienceInsights: AudienceInsights;
  recommendations: string[];
  generatedAt: Date;
}

export interface PlatformAnalytics {
  platform: SocialPlatform;
  totalPosts: number;
  totalEngagement: number;
  averageQualityScore: number;
  xpEarned: number;
  finEarned: number;
  followerGrowth: number;
  topPerformingContent: ObjectId[];
  engagementRate: number;
}

export interface GrowthMetrics {
  followerGrowthRate: number;
  engagementGrowthRate: number;
  contentVolumeGrowth: number;
  qualityImprovement: number;
  earningsGrowth: number;
  virality: number;
}

export interface EarningsBreakdown {
  totalXp: number;
  totalFin: number;
  fromPosts: number;
  fromEngagement: number;
  fromViral: number;
  fromCampaigns: number;
  fromReferrals: number;
  projectedMonthly: number;
}

export interface EngagementTrend {
  date: string;
  totalEngagement: number;
  engagementRate: number;
  reach: number;
  impressions: number;
}

export interface AudienceInsights {
  demographics: {
    ageGroups: Record<string, number>;
    genders: Record<string, number>;
    locations: Record<string, number>;
  };
  interests: string[];
  activeHours: number[];
  engagementPatterns: {
    bestPostTimes: string[];
    bestDays: string[];
    contentPreferences: Record<ContentType, number>;
  };
}

// ===== API REQUEST/RESPONSE TYPES =====

export interface ConnectSocialAccountRequest {
  platform: SocialPlatform;
  authCode: string;
  redirectUri: string;
}

export interface ConnectSocialAccountResponse {
  success: boolean;
  accountId: ObjectId;
  account: SocialAccount;
  message: string;
}

export interface SyncContentRequest {
  accountId: ObjectId;
  platform: SocialPlatform;
  since?: Date;
  limit?: number;
}

export interface SyncContentResponse {
  success: boolean;
  syncedCount: number;
  newContent: number;
  updatedContent: number;
  errors: string[];
  nextSyncAt: Date;
}

export interface GetContentAnalysisRequest {
  contentId: ObjectId;
  includeMetrics?: boolean;
  includeRewards?: boolean;
}

export interface GetContentAnalysisResponse {
  content: SocialContent;
  analytics?: SocialAnalytics;
  recommendations?: string[];
}

export interface CreateCampaignRequest {
  campaign: Omit<SocialCampaign, 'id' | 'participants' | 'metrics' | 'createdAt' | 'updatedAt'>;
}

export interface JoinCampaignRequest {
  campaignId: ObjectId;
  message?: string;
}

export interface GetInfluencerProfileRequest {
  userId?: ObjectId;
  includeRates?: boolean;
  includePortfolio?: boolean;
}

// ===== WEBHOOK TYPES =====

export interface SocialWebhookPayload {
  platform: SocialPlatform;
  event: 'post_created' | 'post_updated' | 'engagement_received' | 'follower_gained';
  accountId: ObjectId;
  data: Record<string, any>;
  timestamp: Date;
  signature: string;
}

export interface WebhookVerification {
  isValid: boolean;
  platform: SocialPlatform;
  challenge?: string;
}

// ===== ERROR TYPES =====

export interface SocialIntegrationError {
  code: string;
  message: string;
  platform: SocialPlatform;
  accountId?: ObjectId;
  details?: Record<string, any>;
  timestamp: Date;
}

export enum SocialErrorCode {
  INVALID_TOKEN = 'INVALID_TOKEN',
  RATE_LIMIT_EXCEEDED = 'RATE_LIMIT_EXCEEDED',
  PLATFORM_API_ERROR = 'PLATFORM_API_ERROR',
  CONTENT_NOT_FOUND = 'CONTENT_NOT_FOUND',
  ANALYSIS_FAILED = 'ANALYSIS_FAILED',
  WEBHOOK_VERIFICATION_FAILED = 'WEBHOOK_VERIFICATION_FAILED',
  INSUFFICIENT_PERMISSIONS = 'INSUFFICIENT_PERMISSIONS',
  ACCOUNT_SUSPENDED = 'ACCOUNT_SUSPENDED'
}

// ===== UTILITY TYPES =====

export interface PaginatedSocialResponse<T> {
  data: T[];
  pagination: {
    page: number;
    limit: number;
    total: number;
    totalPages: number;
    hasNext: boolean;
    hasPrev: boolean;
  };
  filters?: Record<string, any>;
  sort?: Record<string, 'asc' | 'desc'>;
}

export interface SocialActivityFeed {
  activities: SocialActivity[];
  hasMore: boolean;
  nextCursor?: string;
}

export interface SocialActivity {
  id: ObjectId;
  type: 'post_created' | 'went_viral' | 'campaign_joined' | 'milestone_reached';
  userId: ObjectId;
  platform: SocialPlatform;
  title: string;
  description: string;
  metadata: Record<string, any>;
  timestamp: Date;
}

// ===== CONSTANTS =====

export const SOCIAL_CONSTANTS = {
  MAX_DAILY_POSTS: {
    [ContentType.POST]: 50,
    [ContentType.STORY]: 100,
    [ContentType.COMMENT]: 200,
    [ContentType.LIKE]: 500,
    [ContentType.SHARE]: 50
  },
  XP_BASE_VALUES: {
    [ContentType.POST]: 50,
    [ContentType.STORY]: 25,
    [ContentType.VIDEO]: 150,
    [ContentType.COMMENT]: 25,
    [ContentType.LIKE]: 5,
    [ContentType.SHARE]: 15,
    [ContentType.FOLLOW]: 20
  },
  QUALITY_THRESHOLDS: {
    [QualityTier.LOW]: 0.5,
    [QualityTier.MEDIUM]: 0.8,
    [QualityTier.HIGH]: 1.2,
    [QualityTier.VIRAL]: 1.5,
    [QualityTier.PREMIUM]: 2.0
  },
  VIRAL_THRESHOLDS: {
    [SocialPlatform.TIKTOK]: 1000,
    [SocialPlatform.YOUTUBE]: 5000,
    [SocialPlatform.INSTAGRAM]: 2000,
    [SocialPlatform.X]: 10000,
    [SocialPlatform.FACEBOOK]: 1500,
    [SocialPlatform.LINKEDIN]: 500,
    [SocialPlatform.THREADS]: 1000,
    [SocialPlatform.SNAPCHAT]: 500
  }
} as const;

// Export default configuration
export default {
  PLATFORM_CONFIG,
  SOCIAL_CONSTANTS
};
