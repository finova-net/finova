/**
 * Finova Network - Social Integration Types
 * Enterprise-grade TypeScript definitions for social media integration
 * Version: 1.0.0
 * 
 * Supports: Instagram, TikTok, YouTube, Facebook, X/Twitter
 * Features: XP calculation, RP tracking, AI quality assessment, mining integration
 */

import { Types } from 'mongoose';

// ============================================================================
// CORE ENUMS
// ============================================================================

export enum SocialPlatform {
  INSTAGRAM = 'instagram',
  TIKTOK = 'tiktok',
  YOUTUBE = 'youtube',
  FACEBOOK = 'facebook',
  TWITTER_X = 'twitter_x',
  LINKEDIN = 'linkedin',
  FINOVA_NATIVE = 'finova_native'
}

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
  REACTION = 'reaction',
  SHARE = 'share'
}

export enum ActivityType {
  POST = 'post',
  COMMENT = 'comment',
  LIKE = 'like',
  SHARE = 'share',
  FOLLOW = 'follow',
  VIEW = 'view',
  STORY_VIEW = 'story_view',
  LIVE_WATCH = 'live_watch',
  DIRECT_MESSAGE = 'direct_message'
}

export enum ContentStatus {
  DRAFT = 'draft',
  PENDING_REVIEW = 'pending_review',
  APPROVED = 'approved',
  PUBLISHED = 'published',
  FLAGGED = 'flagged',
  REJECTED = 'rejected',
  ARCHIVED = 'archived'
}

export enum QualityTier {
  LOW = 'low',           // 0.5x multiplier
  MEDIUM = 'medium',     // 1.0x multiplier
  HIGH = 'high',         // 1.5x multiplier
  PREMIUM = 'premium',   // 2.0x multiplier
  VIRAL = 'viral'        // 2.5x multiplier
}

export enum EngagementLevel {
  MINIMAL = 'minimal',     // < 10 interactions
  LOW = 'low',            // 10-100 interactions
  MODERATE = 'moderate',   // 100-1K interactions
  HIGH = 'high',          // 1K-10K interactions
  VIRAL = 'viral'         // 10K+ interactions
}

// ============================================================================
// BASE INTERFACES
// ============================================================================

export interface BaseTimestamp {
  createdAt: Date;
  updatedAt: Date;
  deletedAt?: Date;
}

export interface UserReference {
  userId: string;
  username: string;
  displayName: string;
  avatar?: string;
  isVerified: boolean;
  level: number;
  rpTier: string;
}

export interface PlatformMetadata {
  platform: SocialPlatform;
  platformUserId: string;
  platformUsername: string;
  accessToken: string;
  refreshToken?: string;
  tokenExpiresAt: Date;
  isActive: boolean;
  permissions: string[];
  lastSync: Date;
}

// ============================================================================
// CONTENT & ACTIVITY TYPES
// ============================================================================

export interface SocialContent extends BaseTimestamp {
  _id: Types.ObjectId;
  contentId: string;
  user: UserReference;
  platform: SocialPlatform;
  contentType: ContentType;
  status: ContentStatus;
  
  // Content Data
  title?: string;
  description?: string;
  text?: string;
  mediaUrls: string[];
  thumbnailUrl?: string;
  duration?: number; // for videos in seconds
  hashtags: string[];
  mentions: string[];
  
  // Platform-specific IDs
  platformContentId: string;
  platformUrl: string;
  platformMetadata: Record<string, any>;
  
  // Engagement Metrics
  metrics: ContentMetrics;
  
  // Quality Assessment
  qualityScore: QualityAssessment;
  
  // Reward Calculation
  rewards: ContentRewards;
  
  // Moderation
  moderationFlags: ModerationFlag[];
  aiAnalysis: AIAnalysisResult;
  
  // Scheduling
  scheduledAt?: Date;
  publishedAt?: Date;
}

export interface ContentMetrics {
  views: number;
  likes: number;
  comments: number;
  shares: number;
  saves: number;
  reactions: Record<string, number>; // reaction_type -> count
  clickThroughRate?: number;
  watchTime?: number; // total watch time in seconds
  completionRate?: number; // for videos
  engagementRate: number;
  reachEstimate?: number;
  impressions?: number;
  
  // Time-series data for analytics
  dailyMetrics: DailyMetrics[];
  lastUpdated: Date;
}

export interface DailyMetrics {
  date: Date;
  views: number;
  likes: number;
  comments: number;
  shares: number;
  newFollowers: number;
  engagementRate: number;
}

export interface QualityAssessment {
  overallScore: number; // 0.5 - 2.0
  tier: QualityTier;
  factors: QualityFactors;
  aiConfidence: number; // 0.0 - 1.0
  humanVerified: boolean;
  lastAssessed: Date;
}

export interface QualityFactors {
  originality: number;        // 0.0 - 1.0
  engagement: number;         // 0.0 - 1.0
  platformRelevance: number;  // 0.0 - 1.0
  brandSafety: number;        // 0.0 - 1.0
  contentValue: number;       // 0.0 - 1.0
  creativityScore: number;    // 0.0 - 1.0
  technicalQuality: number;   // 0.0 - 1.0
  audienceAlignment: number;  // 0.0 - 1.0
}

export interface ContentRewards {
  baseXP: number;
  bonusXP: number;
  totalXP: number;
  xpMultiplier: number;
  
  baseFIN: number;
  bonusFIN: number;
  totalFIN: number;
  finMultiplier: number;
  
  rpContribution: number;
  
  // Calculation breakdown
  calculation: RewardCalculation;
  
  // Status
  isPaid: boolean;
  paidAt?: Date;
  transactionHash?: string;
}

export interface RewardCalculation {
  baseRate: number;
  platformMultiplier: number;
  qualityMultiplier: number;
  levelMultiplier: number;
  streakBonus: number;
  viralBonus: number;
  timingBonus: number; // posting at optimal times
  networkEffect: number; // RP tier influence
  
  // Penalties
  botPenalty: number;
  qualityPenalty: number;
  spamPenalty: number;
  
  finalMultiplier: number;
  timestamp: Date;
}

// ============================================================================
// SOCIAL ACTIVITIES
// ============================================================================

export interface SocialActivity extends BaseTimestamp {
  _id: Types.ObjectId;
  activityId: string;
  user: UserReference;
  platform: SocialPlatform;
  activityType: ActivityType;
  
  // Target content/user
  targetContentId?: string;
  targetUserId?: string;
  targetType: 'content' | 'user' | 'page';
  
  // Activity data
  data: ActivityData;
  
  // Rewards
  rewards: ActivityRewards;
  
  // Quality check
  isGenuine: boolean;
  botScore: number; // 0.0 - 1.0 (higher = more likely bot)
  
  // Platform tracking
  platformActivityId?: string;
  platformData: Record<string, any>;
}

export interface ActivityData {
  // Common fields
  text?: string;
  mediaUrls?: string[];
  duration?: number;
  
  // Platform-specific data
  platformSpecific: Record<string, any>;
  
  // Context
  deviceInfo: DeviceInfo;
  locationInfo?: LocationInfo;
  sessionInfo: SessionInfo;
}

export interface DeviceInfo {
  deviceType: 'mobile' | 'desktop' | 'tablet';
  os: string;
  browser?: string;
  appVersion?: string;
  screenResolution?: string;
  timezone: string;
}

export interface LocationInfo {
  country: string;
  city?: string;
  coordinates?: {
    latitude: number;
    longitude: number;
  };
  ipAddress: string; // hashed for privacy
}

export interface SessionInfo {
  sessionId: string;
  sessionStart: Date;
  userAgent: string;
  referrer?: string;
  campaignSource?: string;
}

export interface ActivityRewards {
  xpAwarded: number;
  finAwarded: number;
  rpContribution: number;
  
  multipliers: ActivityMultipliers;
  
  // Streak bonuses
  streakBonus: number;
  streakCount: number;
  
  // Quality bonuses
  qualityBonus: number;
  
  // Timing bonuses
  timingBonus: number;
  
  totalReward: number;
  isPaid: boolean;
  paidAt?: Date;
}

export interface ActivityMultipliers {
  base: number;
  platform: number;
  level: number;
  streak: number;
  quality: number;
  timing: number;
  network: number; // RP tier effect
  final: number;
}

// ============================================================================
// AI ANALYSIS & MODERATION
// ============================================================================

export interface AIAnalysisResult {
  contentId: string;
  analysisVersion: string;
  analysisDate: Date;
  
  // Quality scores
  qualityScores: QualityFactors;
  
  // Safety checks
  safetyFlags: SafetyFlag[];
  brandSafetyScore: number; // 0.0 - 1.0
  
  // Content classification
  categories: ContentCategory[];
  tags: string[];
  sentiment: SentimentAnalysis;
  
  // Bot detection
  botDetection: BotDetectionResult;
  
  // Language detection
  language: string;
  languageConfidence: number;
  
  // Visual analysis (for images/videos)
  visualAnalysis?: VisualAnalysis;
  
  // Text analysis
  textAnalysis?: TextAnalysis;
  
  // Audio analysis (for videos with sound)
  audioAnalysis?: AudioAnalysis;
}

export interface SafetyFlag {
  type: string;
  severity: 'low' | 'medium' | 'high' | 'critical';
  confidence: number;
  description: string;
  suggestedAction: 'approve' | 'review' | 'reject' | 'flag';
}

export interface ContentCategory {
  category: string;
  subcategory?: string;
  confidence: number;
  isAdult: boolean;
  isBrandSafe: boolean;
}

export interface SentimentAnalysis {
  overall: 'positive' | 'neutral' | 'negative';
  confidence: number;
  scores: {
    positive: number;
    neutral: number;
    negative: number;
  };
  emotions: EmotionScore[];
}

export interface EmotionScore {
  emotion: string;
  score: number; // 0.0 - 1.0
}

export interface BotDetectionResult {
  isBot: boolean;
  botScore: number; // 0.0 - 1.0
  confidence: number;
  
  factors: BotDetectionFactors;
  
  humanProbability: number;
  riskLevel: 'low' | 'medium' | 'high' | 'critical';
  
  recommendations: string[];
}

export interface BotDetectionFactors {
  behavioralPattern: number;    // timing patterns, activity frequency
  contentOriginality: number;   // uniqueness of content
  socialGraphAnalysis: number;  // network connections quality
  deviceConsistency: number;    // device fingerprint analysis
  biometricConsistency: number; // if biometric data available
  linguisticAnalysis: number;   // writing style consistency
  engagementPatterns: number;   // how others interact with content
}

export interface VisualAnalysis {
  objectDetection: DetectedObject[];
  sceneClassification: SceneClass[];
  faceDetection?: FaceDetection[];
  textExtraction?: ExtractedText[];
  aestheticScore: number; // 0.0 - 1.0
  technicalQuality: TechnicalQuality;
  brandElements?: BrandElement[];
}

export interface DetectedObject {
  object: string;
  confidence: number;
  boundingBox: BoundingBox;
}

export interface BoundingBox {
  x: number;
  y: number;
  width: number;
  height: number;
}

export interface SceneClass {
  scene: string;
  confidence: number;
}

export interface FaceDetection {
  boundingBox: BoundingBox;
  confidence: number;
  attributes: FaceAttributes;
}

export interface FaceAttributes {
  age?: number;
  gender?: string;
  emotion?: string;
  ethnicity?: string;
}

export interface ExtractedText {
  text: string;
  boundingBox: BoundingBox;
  confidence: number;
  language?: string;
}

export interface TechnicalQuality {
  resolution: string;
  sharpness: number;   // 0.0 - 1.0
  brightness: number;  // 0.0 - 1.0
  contrast: number;    // 0.0 - 1.0
  colorfulness: number; // 0.0 - 1.0
  noiseLevel: number;  // 0.0 - 1.0 (lower is better)
}

export interface BrandElement {
  type: 'logo' | 'text' | 'product' | 'watermark';
  brand?: string;
  confidence: number;
  boundingBox: BoundingBox;
}

export interface TextAnalysis {
  wordCount: number;
  characterCount: number;
  sentenceCount: number;
  readabilityScore: number; // Flesch-Kincaid or similar
  
  keywords: Keyword[];
  entities: NamedEntity[];
  topics: Topic[];
  
  plagiarismCheck: PlagiarismResult;
  
  linguisticFeatures: LinguisticFeatures;
}

export interface Keyword {
  word: string;
  frequency: number;
  relevance: number;
  position: number[];
}

export interface NamedEntity {
  entity: string;
  type: string; // PERSON, ORG, LOCATION, etc.
  confidence: number;
  mentions: number;
}

export interface Topic {
  topic: string;
  confidence: number;
  keywords: string[];
}

export interface PlagiarismResult {
  isPlagiarized: boolean;
  similarityScore: number; // 0.0 - 1.0
  sources: PlagiarismSource[];
  originalityScore: number; // 0.0 - 1.0
}

export interface PlagiarismSource {
  url?: string;
  source: string;
  similarity: number;
  matchedText: string[];
}

export interface LinguisticFeatures {
  averageWordLength: number;
  averageSentenceLength: number;
  lexicalDiversity: number;
  formalityScore: number;
  complexityScore: number;
  grammarScore: number;
}

export interface AudioAnalysis {
  duration: number;
  sampleRate: number;
  channels: number;
  
  speechToText?: SpeechResult[];
  musicDetection?: MusicDetection;
  soundClassification: SoundClass[];
  
  audioQuality: AudioQuality;
  
  emotionAnalysis?: AudioEmotion[];
}

export interface SpeechResult {
  text: string;
  confidence: number;
  startTime: number;
  endTime: number;
  speaker?: string;
  language: string;
}

export interface MusicDetection {
  hasMusic: boolean;
  musicConfidence: number;
  genre?: string;
  mood?: string;
  energy?: number; // 0.0 - 1.0
  tempo?: number;
}

export interface SoundClass {
  sound: string;
  confidence: number;
  startTime: number;
  endTime: number;
}

export interface AudioQuality {
  overallScore: number; // 0.0 - 1.0
  noiseLevel: number;   // 0.0 - 1.0
  clarity: number;      // 0.0 - 1.0
  volumeLevel: number;  // dB
  dynamicRange: number; // dB
}

export interface AudioEmotion {
  emotion: string;
  confidence: number;
  startTime: number;
  endTime: number;
}

// ============================================================================
// MODERATION & FLAGS
// ============================================================================

export interface ModerationFlag {
  flagId: string;
  type: ModerationFlagType;
  severity: 'low' | 'medium' | 'high' | 'critical';
  source: 'ai' | 'user_report' | 'admin' | 'automated';
  
  reason: string;
  description: string;
  
  flaggedBy?: UserReference;
  flaggedAt: Date;
  
  status: ModerationStatus;
  reviewedBy?: UserReference;
  reviewedAt?: Date;
  reviewNotes?: string;
  
  // Actions taken
  actions: ModerationAction[];
  
  // Appeal info
  appealInfo?: AppealInfo;
}

export enum ModerationFlagType {
  SPAM = 'spam',
  INAPPROPRIATE_CONTENT = 'inappropriate_content',
  HATE_SPEECH = 'hate_speech',
  VIOLENCE = 'violence',
  NUDITY = 'nudity',
  COPYRIGHT = 'copyright',
  FAKE_NEWS = 'fake_news',
  HARASSMENT = 'harassment',
  BOT_ACTIVITY = 'bot_activity',
  MISLEADING = 'misleading',
  ADULT_CONTENT = 'adult_content',
  DANGEROUS_ACTIVITY = 'dangerous_activity'
}

export enum ModerationStatus {
  PENDING = 'pending',
  REVIEWED = 'reviewed',
  APPROVED = 'approved',
  REJECTED = 'rejected',
  ESCALATED = 'escalated',
  APPEALED = 'appealed'
}

export interface ModerationAction {
  action: 'warn' | 'restrict' | 'suspend' | 'ban' | 'remove_content' | 'reduce_rewards' | 'no_action';
  duration?: number; // in hours
  reason: string;
  appliedBy: UserReference;
  appliedAt: Date;
  isActive: boolean;
}

export interface AppealInfo {
  appealId: string;
  reason: string;
  submittedBy: UserReference;
  submittedAt: Date;
  status: 'pending' | 'reviewing' | 'approved' | 'rejected';
  reviewedBy?: UserReference;
  reviewedAt?: Date;
  decision?: string;
  evidence?: string[];
}

// ============================================================================
// PLATFORM INTEGRATION TYPES
// ============================================================================

export interface PlatformIntegration {
  platform: SocialPlatform;
  isEnabled: boolean;
  isConnected: boolean;
  
  // Authentication
  auth: PlatformAuth;
  
  // Configuration
  config: PlatformConfig;
  
  // Statistics
  stats: PlatformStats;
  
  // Rate limiting
  rateLimits: RateLimitInfo;
  
  // Sync status
  syncStatus: SyncStatus;
}

export interface PlatformAuth {
  accessToken: string;
  refreshToken?: string;
  tokenType: string;
  expiresAt: Date;
  scopes: string[];
  
  // User info from platform
  platformUserId: string;
  platformUsername: string;
  platformEmail?: string;
  platformVerified: boolean;
  
  // Connection status
  lastAuthAt: Date;
  authStatus: 'active' | 'expired' | 'revoked' | 'error';
}

export interface PlatformConfig {
  // Auto-posting settings
  autoPost: boolean;
  autoEngage: boolean;
  
  // Content filtering
  contentFilters: ContentFilter[];
  
  // Scheduling preferences
  schedulingPrefs: SchedulingPreferences;
  
  // Quality thresholds
  qualityThresholds: QualityThresholds;
  
  // Notification preferences
  notifications: NotificationPreferences;
  
  // Privacy settings
  privacy: PrivacySettings;
}

export interface ContentFilter {
  type: 'hashtag' | 'keyword' | 'user' | 'category';
  value: string;
  action: 'include' | 'exclude' | 'flag';
  isActive: boolean;
}

export interface SchedulingPreferences {
  timezone: string;
  optimalTimes: TimeSlot[];
  avoidTimes: TimeSlot[];
  maxPostsPerDay: number;
  minIntervalHours: number;
}

export interface TimeSlot {
  dayOfWeek: number; // 0-6 (Sunday-Saturday)
  hour: number;      // 0-23
  minute: number;    // 0-59
}

export interface QualityThresholds {
  minimumScore: number;        // 0.0 - 1.0
  autoRejectBelow: number;     // 0.0 - 1.0
  requireReviewBelow: number;  // 0.0 - 1.0
  autoApproveAbove: number;    // 0.0 - 1.0
}

export interface NotificationPreferences {
  newContent: boolean;
  qualityAlerts: boolean;
  rewardUpdates: boolean;
  moderationAlerts: boolean;
  performanceReports: boolean;
  
  // Delivery methods
  email: boolean;
  push: boolean;
  sms: boolean;
  discord?: string;
  telegram?: string;
}

export interface PrivacySettings {
  publicProfile: boolean;
  shareMetrics: boolean;
  allowTagging: boolean;
  allowMentions: boolean;
  dataSharingLevel: 'none' | 'aggregated' | 'pseudonymized' | 'full';
}

export interface PlatformStats {
  totalContent: number;
  totalViews: number;
  totalLikes: number;
  totalComments: number;
  totalShares: number;
  
  // Time-based stats
  last24h: PlatformMetrics;
  last7d: PlatformMetrics;
  last30d: PlatformMetrics;
  allTime: PlatformMetrics;
  
  // Performance metrics
  averageQualityScore: number;
  averageEngagementRate: number;
  topPerformingContent: string[]; // content IDs
  
  // Reward stats
  totalXPEarned: number;
  totalFINEarned: number;
  totalRPContributed: number;
  
  lastUpdated: Date;
}

export interface PlatformMetrics {
  posts: number;
  views: number;
  likes: number;
  comments: number;
  shares: number;
  followers: number;
  following: number;
  engagementRate: number;
  reachRate: number;
}

export interface RateLimitInfo {
  requestsPerHour: number;
  requestsRemaining: number;
  resetTime: Date;
  
  // Platform-specific limits
  postsPerDay: number;
  commentsPerHour: number;
  likesPerHour: number;
  followsPerDay: number;
  
  // Current usage
  currentUsage: RateLimitUsage;
}

export interface RateLimitUsage {
  requests: number;
  posts: number;
  comments: number;
  likes: number;
  follows: number;
  
  windowStart: Date;
  windowEnd: Date;
}

export interface SyncStatus {
  lastSync: Date;
  nextSync: Date;
  syncFrequency: number; // minutes
  
  isRunning: boolean;
  progress: number; // 0.0 - 1.0
  
  status: 'idle' | 'syncing' | 'error' | 'paused';
  error?: SyncError;
  
  // Sync statistics
  stats: SyncStats;
}

export interface SyncError {
  code: string;
  message: string;
  timestamp: Date;
  retryCount: number;
  nextRetry?: Date;
}

export interface SyncStats {
  totalSynced: number;
  newContent: number;
  updatedContent: number;
  errors: number;
  
  avgSyncTime: number; // seconds
  lastSyncDuration: number; // seconds
  
  contentTypes: Record<ContentType, number>;
}

// ============================================================================
// SOCIAL NETWORK & REFERRAL TYPES
// ============================================================================

export interface SocialNetwork {
  userId: string;
  
  // Connections
  connections: Connection[];
  followingCount: number;
  followersCount: number;
  
  // Referral network
  referralNetwork: ReferralNode;
  
  // Social graph metrics
  networkMetrics: NetworkMetrics;
  
  // Influence scores
  influenceScores: InfluenceScores;
  
  lastUpdated: Date;
}

export interface Connection {
  userId: string;
  username: string;
  platform: SocialPlatform;
  connectionType: 'follower' | 'following' | 'mutual' | 'referral';
  
  connectedAt: Date;
  lastInteraction?: Date;
  interactionCount: number;
  
  // Mutual metrics
  mutualConnections: number;
  engagementScore: number; // how often they interact
  
  // Referral info (if applicable)
  referralInfo?: ReferralConnection;
}

export interface ReferralConnection {
  referralCode: string;
  level: number; // 1 = direct, 2 = L2, 3 = L3
  referredAt: Date;
  
  // Activity tracking
  isActive: boolean; // active in last 30 days
  lastActivity: Date;
  totalEarnings: number;
  
  // Performance metrics
  activityScore: number;  // 0.0 - 1.0
  qualityScore: number;   // 0.0 - 1.0
  loyaltyScore: number;   // 0.0 - 1.0
  
  // Contribution to referrer
  rpContribution: number;
  finContribution: number;
  xpContribution: number;
}

export interface ReferralNode {
  // Direct referrals (L1)
  directReferrals: ReferralConnection[];
  directCount: number;
  activeDirectCount: number;
  
  // Network referrals (L2, L3)
  networkReferrals: ReferralConnection[];
  l2Count: number;
  l3Count: number;
  activeNetworkCount: number;
  
  // Network quality metrics
  networkQuality: NetworkQuality;
  
  // Referral performance
  referralPerformance: ReferralPerformance;
  
  // Tier information
  currentTier: string;
  nextTier: string;
  progressToNext: number; // 0.0 - 1.0
}

export interface NetworkQuality {
  overall: number;        // 0.0 - 1.0
  retention: number;      // % still active after 30 days
  activity: number;       // average activity level
  diversity: number;      // geographic/demographic diversity
  authenticity: number;   // human verification score
  engagement: number;     // how engaged the network is
}

export interface ReferralPerformance {
  totalEarned: number;
  last30Days: number;
  conversionRate: number;    // successful referrals / invitations
  averageQuality: number;    // average quality of referrals
  networkGrowthRate: number; // monthly growth percentage
  
  // Leaderboard position
  globalRank: number;
  regionRank: number;
  tierRank: number;
  
  // Achievements
  achievements: ReferralAchievement[];
}

export interface ReferralAchievement {
  id: string;
  name: string;
  description: string;
  tier: 'bronze' | 'silver' | 'gold' | 'platinum' | 'diamond';
  
  earnedAt: Date;
  requirements: AchievementRequirement[];
  
  rewards: AchievementReward[];
  
  isPublic: boolean;
  rarity: number; // percentage of users who have this
}

export interface AchievementRequirement {
  type: string;
  value: number;
  description: string;
}

export interface AchievementReward {
  type: 'xp' | 'fin' | 'rp' | 'nft' | 'badge' | 'multiplier';
  value: number;
  duration?: number; // for temporary rewards
  description: string;
}

export interface NetworkMetrics {
  // Centrality measures
  betweennessCentrality: number;
  closenessCentrality: number;
  eigenvectorCentrality: number;
  pageRankScore: number;
  
  // Network size and reach
  networkSize: number;
  networkReach: number;        // potential audience size
  clusteringCoefficient: number;
  averagePathLength: number;
  
  // Influence metrics
  directInfluence: number;     // immediate followers
  networkInfluence: number;    // extended network
  viralCoefficient: number;    // content spread potential
  
  // Activity patterns
  activityCorrelation: number; // how network activity correlates
  responseTime: number;        // average response to user content
  engagementAmplification: number; // network engagement boost
}

export interface InfluenceScores {
  overall: number;          // 0.0 - 1.0
  contentCreator: number;   // influence on content trends
  networkBuilder: number;   // influence on network growth
  qualityLeader: number;    // influence on quality standards
  
  // Platform-specific influence
  platformInfluence: Record<SocialPlatform, number>;
  
  // Temporal influence
  currentTrend: number;     // trending up/down
  peakInfluence: number;    // historical peak
  sustainedInfluence: number; // long-term average
  
  // Calculated factors
  factors: InfluenceFactors;
  
  lastCalculated: Date;
}

export interface InfluenceFactors {
  followerCount: number;        // raw follower numbers
  engagementRate: number;       // how engaged followers are
  contentQuality: number;       // average content quality
  networkEffect: number;        // amplification through network
  consistency: number;          // posting consistency
  originality: number;          // content originality
  platformDiversity: number;    // presence across platforms
  crossPlatformSynergy: number; // how platforms work together
}

// ============================================================================
// SOCIAL CAMPAIGNS & CHALLENGES
// ============================================================================

export interface SocialCampaign extends BaseTimestamp {
  _id: Types.ObjectId;
  campaignId: string;
  
  // Basic info
  name: string;
  description: string;
  type: CampaignType;
  status: CampaignStatus;
  
  // Timing
  startDate: Date;
  endDate: Date;
  timezone: string;
  
  // Targeting
  targeting: CampaignTargeting;
  
  // Requirements
  requirements: CampaignRequirement[];
  
  // Rewards
  rewards: CampaignReward[];
  
  // Content guidelines
  contentGuidelines: ContentGuidelines;
  
  // Participation tracking
  participants: CampaignParticipant[];
  submissions: CampaignSubmission[];
  
  // Performance metrics
  metrics: CampaignMetrics;
  
  // Brand partnership info
  brandInfo?: BrandInfo;
  
  // Moderation
  moderationSettings: ModerationSettings;
}

export enum CampaignType {
  HASHTAG_CHALLENGE = 'hashtag_challenge',
  CONTENT_CONTEST = 'content_contest',
  BRAND_PARTNERSHIP = 'brand_partnership',
  COMMUNITY_CHALLENGE = 'community_challenge',
  SEASONAL_EVENT = 'seasonal_event',
  PRODUCT_LAUNCH = 'product_launch',
  AWARENESS_CAMPAIGN = 'awareness_campaign',
  USER_GENERATED_CONTENT = 'user_generated_content'
}

export enum CampaignStatus {
  DRAFT = 'draft',
  SCHEDULED = 'scheduled',
  ACTIVE = 'active',
  PAUSED = 'paused',
  COMPLETED = 'completed',
  CANCELLED = 'cancelled'
}

export interface CampaignTargeting {
  // Geographic targeting
  countries: string[];
  cities: string[];
  excludeRegions: string[];
  
  // User targeting
  minLevel: number;
  maxLevel?: number;
  rpTiers: string[];
  platforms: SocialPlatform[];
  
  // Demographic targeting
  ageRange?: AgeRange;
  languages: string[];
  interests: string[];
  
  // Behavioral targeting
  minFollowers?: number;
  maxFollowers?: number;
  engagementTiers: EngagementLevel[];
  contentTypes: ContentType[];
  
  // Quality filters
  minQualityScore: number;
  excludeBotUsers: boolean;
  kycRequired: boolean;
}

export interface AgeRange {
  min: number;
  max: number;
}

export interface CampaignRequirement {
  id: string;
  type: RequirementType;
  description: string;
  value: any;
  isOptional: boolean;
  points: number; // weight in scoring
}

export enum RequirementType {
  HASHTAG_USAGE = 'hashtag_usage',
  MENTION_BRAND = 'mention_brand',
  MIN_VIEWS = 'min_views',
  MIN_LIKES = 'min_likes',
  ORIGINAL_CONTENT = 'original_content',
  PLATFORM_SPECIFIC = 'platform_specific',
  CONTENT_LENGTH = 'content_length',
  INCLUDE_MEDIA = 'include_media',
  USER_LEVEL = 'user_level',
  FOLLOW_ACCOUNT = 'follow_account'
}

export interface CampaignReward {
  tier: RewardTier;
  position: string; // "1st", "2nd-5th", "6th-20th", "participation"
  
  rewards: RewardItem[];
  
  // Requirements for this tier
  minScore: number;
  maxWinners: number;
  
  // Distribution method
  distributionMethod: DistributionMethod;
}

export enum RewardTier {
  GRAND_PRIZE = 'grand_prize',
  FIRST_PLACE = 'first_place',
  TOP_TIER = 'top_tier',
  MID_TIER = 'mid_tier',
  PARTICIPATION = 'participation',
  BONUS = 'bonus'
}

export interface RewardItem {
  type: 'fin' | 'xp' | 'rp' | 'nft' | 'special_card' | 'cash' | 'product';
  amount: number;
  description: string;
  
  // Special properties
  rarity?: string;
  duration?: number; // for temporary rewards
  conditions?: string[]; // additional conditions
  
  // NFT/Card specific
  nftMetadata?: NFTMetadata;
  cardType?: string;
}

export interface NFTMetadata {
  name: string;
  description: string;
  image: string;
  attributes: NFTAttribute[];
  rarity: string;
  collection: string;
}

export interface NFTAttribute {
  trait_type: string;
  value: string | number;
  rarity?: number; // percentage rarity
}

export enum DistributionMethod {
  AUTOMATIC = 'automatic',        // Auto-distribute based on scores
  MANUAL_REVIEW = 'manual_review', // Manual selection by judges
  COMMUNITY_VOTE = 'community_vote', // Community voting
  RANDOM_DRAW = 'random_draw',     // Random selection from qualified
  HYBRID = 'hybrid'                // Combination of methods
}

export interface ContentGuidelines {
  allowedContentTypes: ContentType[];
  requiredHashtags: string[];
  forbiddenContent: string[];
  
  // Technical requirements
  minResolution?: string;
  maxDuration?: number; // seconds
  minDuration?: number; // seconds
  fileFormats: string[];
  
  // Content requirements
  originalContentOnly: boolean;
  allowReposts: boolean;
  mustShowProduct: boolean;
  mustShowBrand: boolean;
  
  // Quality standards
  minQualityScore: number;
  brandSafetyRequired: boolean;
  languageRestrictions: string[];
  
  // Legal requirements
  disclosureRequired: boolean;
  disclosureText?: string;
  ageRestrictions?: AgeRange;
  geographicRestrictions: string[];
}

export interface CampaignParticipant {
  userId: string;
  username: string;
  joinedAt: Date;
  
  // Qualification status
  isQualified: boolean;
  qualificationChecks: QualificationCheck[];
  
  // Submissions
  submissionCount: number;
  bestSubmissionScore: number;
  
  // Rewards earned
  rewardsEarned: RewardItem[];
  
  // Engagement with campaign
  engagementScore: number;
  referralCount: number; // referred other participants
}

export interface QualificationCheck {
  requirement: string;
  status: 'pending' | 'passed' | 'failed';
  checkedAt: Date;
  notes?: string;
}

export interface CampaignSubmission extends BaseTimestamp {
  _id: Types.ObjectId;
  submissionId: string;
  
  campaignId: string;
  userId: string;
  contentId: string; // links to SocialContent
  
  // Submission details
  submittedAt: Date;
  platform: SocialPlatform;
  contentUrl: string;
  
  // Scoring
  score: SubmissionScore;
  
  // Review process
  reviewStatus: ReviewStatus;
  reviewedBy?: UserReference[];
  reviewNotes?: string;
  
  // Competition ranking
  rank?: number;
  percentile?: number;
  
  // Rewards
  rewardsAwarded: RewardItem[];
  rewardStatus: RewardStatus;
}

export interface SubmissionScore {
  total: number;
  breakdown: ScoreBreakdown;
  
  // Judge scores (if manual review)
  judgeScores: JudgeScore[];
  
  // Community voting (if applicable)
  communityVotes: CommunityVote[];
  
  // Automated scoring
  automatedScore: AutomatedScore;
  
  finalRank: number;
  lastUpdated: Date;
}

export interface ScoreBreakdown {
  creativity: number;
  qualityScore: number;
  engagementMetrics: number;
  requirementCompliance: number;
  brandAlignment: number;
  originalityScore: number;
  technicalQuality: number;
  communityResponse: number;
}

export interface JudgeScore {
  judgeId: string;
  judgeName: string;
  score: number;
  maxScore: number;
  feedback?: string;
  scoredAt: Date;
  
  // Detailed scoring
  criteriaScores: Record<string, number>;
}

export interface CommunityVote {
  voterId: string;
  voterLevel: number;
  vote: number; // 1-5 stars or similar
  votedAt: Date;
  
  // Vote weight based on voter quality
  weight: number;
  
  // Optional feedback
  feedback?: string;
}

export interface AutomatedScore {
  aiQualityScore: number;
  engagementVelocity: number;
  viralityScore: number;
  brandMentionScore: number;
  hashtagComplianceScore: number;
  originalityScore: number;
  
  // Platform-specific metrics
  platformOptimizationScore: number;
  
  // Penalties
  penalties: ScorePenalty[];
  
  confidence: number; // AI confidence in scoring
}

export interface ScorePenalty {
  type: string;
  description: string;
  penalty: number;
  appliedAt: Date;
}

export enum ReviewStatus {
  PENDING = 'pending',
  IN_REVIEW = 'in_review',
  APPROVED = 'approved',
  REJECTED = 'rejected',
  REQUIRES_REVISION = 'requires_revision',
  DISQUALIFIED = 'disqualified'
}

export enum RewardStatus {
  PENDING = 'pending',
  PROCESSING = 'processing',
  DISTRIBUTED = 'distributed',
  FAILED = 'failed',
  CANCELLED = 'cancelled'
}

export interface CampaignMetrics {
  // Participation metrics
  totalParticipants: number;
  qualifiedParticipants: number;
  totalSubmissions: number;
  approvedSubmissions: number;
  
  // Engagement metrics
  totalViews: number;
  totalLikes: number;
  totalComments: number;
  totalShares: number;
  hashtagUses: number;
  brandMentions: number;
  
  // Quality metrics
  averageQualityScore: number;
  averageEngagementRate: number;
  viralSubmissions: number; // submissions that went viral
  
  // Geographic distribution
  participantsByCountry: Record<string, number>;
  submissionsByPlatform: Record<SocialPlatform, number>;
  
  // Performance tracking
  dailyMetrics: DailyCampaignMetrics[];
  
  // ROI and effectiveness
  totalRewardsDistributed: number;
  costPerParticipant: number;
  costPerEngagement: number;
  brandAwarenessLift: number;
  
  lastUpdated: Date;
}

export interface DailyCampaignMetrics {
  date: Date;
  newParticipants: number;
  newSubmissions: number;
  totalViews: number;
  totalEngagements: number;
  hashtagMentions: number;
  qualityScoreAverage: number;
}

export interface BrandInfo {
  brandId: string;
  brandName: string;
  brandLogo: string;
  brandWebsite: string;
  
  // Contact information
  contactPerson: string;
  contactEmail: string;
  
  // Campaign budget
  totalBudget: number;
  spentBudget: number;
  
  // Brand preferences
  brandGuidelines: BrandGuidelines;
  targetAudience: AudienceProfile;
  
  // Performance expectations
  kpis: BrandKPI[];
  
  // Legal and compliance
  contractId?: string;
  approvalWorkflow: ApprovalWorkflow;
}

export interface BrandGuidelines {
  logoUsage: LogoUsage;
  colorPalette: string[];
  forbiddenContent: string[];
  requiredDisclosures: string[];
  
  // Messaging
  keyMessages: string[];
  tone: string;
  voiceGuidelines: string;
  
  // Visual guidelines
  visualStyle: string;
  imageFilters: string[];
  fontPreferences: string[];
}

export interface LogoUsage {
  required: boolean;
  placement: 'top' | 'bottom' | 'corner' | 'watermark' | 'anywhere';
  minSize: string;
  clearSpace: string;
  colorVariations: string[];
}

export interface AudienceProfile {
  primaryAge: AgeRange;
  secondaryAge?: AgeRange;
  genders: string[];
  interests: string[];
  behaviors: string[];
  
  // Geographic preferences
  primaryMarkets: string[];
  secondaryMarkets: string[];
  excludeMarkets: string[];
  
  // Platform preferences
  preferredPlatforms: SocialPlatform[];
  platformPriorities: Record<SocialPlatform, number>;
}

export interface BrandKPI {
  metric: string;
  target: number;
  current: number;
  unit: string;
  priority: 'high' | 'medium' | 'low';
  description: string;
}

export interface ApprovalWorkflow {
  requiresBrandApproval: boolean;
  approvalStages: ApprovalStage[];
  autoApprovalThreshold?: number;
  maxApprovalTime: number; // hours
}

export interface ApprovalStage {
  stage: string;
  approver: string;
  maxTime: number; // hours
  criteria: string[];
  isRequired: boolean;
}

export interface ModerationSettings {
  // Automated moderation
  aiModerationEnabled: boolean;
  autoRejectThreshold: number;
  autoApproveThreshold: number;
  
  // Human moderation
  requiresHumanReview: boolean;
  moderatorCount: number;
  moderationCriteria: ModerationCriteria[];
  
  // Appeal process
  allowAppeals: boolean;
  maxAppealTime: number; // hours
  appealReviewers: string[];
  
  // Escalation
  escalationThreshold: number;
  escalationContacts: string[];
}

export interface ModerationCriteria {
  criterion: string;
  weight: number;
  threshold: number;
  description: string;
  automatable: boolean;
}

// ============================================================================
// ANALYTICS & REPORTING TYPES
// ============================================================================

export interface SocialAnalytics {
  userId: string;
  period: AnalyticsPeriod;
  
  // Overview metrics
  overview: AnalyticsOverview;
  
  // Platform breakdown
  platformAnalytics: Record<SocialPlatform, PlatformAnalytics>;
  
  // Content performance
  contentAnalytics: ContentAnalytics;
  
  // Audience insights
  audienceInsights: AudienceInsights;
  
  // Growth metrics
  growthMetrics: GrowthMetrics;
  
  // Reward analytics
  rewardAnalytics: RewardAnalytics;
  
  // Competitive analysis
  competitiveAnalysis: CompetitiveAnalysis;
  
  // Predictions and recommendations
  predictions: AnalyticsPredictions;
  recommendations: AnalyticsRecommendations[];
  
  generatedAt: Date;
  validUntil: Date;
}

export interface AnalyticsPeriod {
  start: Date;
  end: Date;
  granularity: 'hour' | 'day' | 'week' | 'month';
  timezone: string;
}

export interface AnalyticsOverview {
  // Content metrics
  totalContent: number;
  totalViews: number;
  totalEngagements: number;
  averageEngagementRate: number;
  
  // Quality metrics
  averageQualityScore: number;
  qualityTrend: 'up' | 'down' | 'stable';
  qualityDistribution: Record<QualityTier, number>;
  
  // Rewards summary
  totalXPEarned: number;
  totalFINEarned: number;
  totalRPEarned: number;
  
  // Growth indicators
  followerGrowth: number;
  engagementGrowth: number;
  rewardGrowth: number;
  
  // Performance indicators
  topPerformer: boolean; // top 10% in tier
  trendingContent: number; // pieces of trending content
  viralContent: number; // pieces that went viral
  
  // Comparative metrics
  percentileRank: number; // 0-100, where they rank among peers
  tierPosition: number; // position within their RP tier
}

export interface PlatformAnalytics {
  platform: SocialPlatform;
  
  // Basic metrics
  metrics: PlatformMetrics;
  
  // Performance trends
  trends: PlatformTrends;
  
  // Content performance
  topContent: ContentPerformance[];
  contentTypePerformance: Record<ContentType, ContentPerformance>;
  
  // Audience data
  audienceData: PlatformAudienceData;
  
  // Optimal timing
  bestTimes: OptimalTiming;
  
  // Platform-specific insights
  platformInsights: PlatformSpecificInsights;
}

export interface PlatformTrends {
  viewsTrend: TrendData;
  likesTrend: TrendData;
  commentsTrend: TrendData;
  sharesTrend: TrendData;
  followersTrend: TrendData;
  engagementRateTrend: TrendData;
  qualityScoreTrend: TrendData;
}

export interface TrendData {
  current: number;
  previous: number;
  change: number; // percentage change
  direction: 'up' | 'down' | 'stable';
  isSignificant: boolean; // statistically significant change
  
  // Time series data points
  dataPoints: TimeSeriesPoint[];
  
  // Trend analysis
  trendType: 'linear' | 'exponential' | 'logarithmic' | 'seasonal' | 'irregular';
  correlation: number; // correlation with time
  volatility: number; // how much it varies
}

export interface TimeSeriesPoint {
  timestamp: Date;
  value: number;
  
  // Additional context
  events?: string[]; // special events that might have influenced this point
  anomaly?: boolean; // is this an anomalous data point
  confidence?: number; // confidence in this data point
}

export interface ContentPerformance {
  contentId: string;
  contentType: ContentType;
  
  // Performance metrics
  views: number;
  engagements: number;
  engagementRate: number;
  qualityScore: number;
  viralityScore: number;
  
  // Rewards earned
  xpEarned: number;
  finEarned: number;
  rpContribution: number;
  
  // Performance indicators
  isTopPerformer: boolean;
  performancePercentile: number;
  
  // Content characteristics
  characteristics: ContentCharacteristics;
  
  publishedAt: Date;
  peakPerformanceTime: Date;
}

export interface ContentCharacteristics {
  hashtags: string[];
  mentions: string[];
  topics: string[];
  sentiment: string;
  
  // Media characteristics
  hasImage: boolean;
  hasVideo: boolean;
  mediaCount: number;
  videoDuration?: number;
  
  // Text characteristics
  wordCount?: number;
  readabilityScore?: number;
  languageComplexity?: number;
  
  // Timing characteristics
  publishTime: {
    hour: number;
    dayOfWeek: number;
    isOptimalTime: boolean;
  };
}

export interface PlatformAudienceData {
  // Demographics
  ageDistribution: Record<string, number>;
  genderDistribution: Record<string, number>;
  locationDistribution: Record<string, number>;
  languageDistribution: Record<string, number>;
  
  // Behavior patterns
  activeHours: Record<number, number>; // hour -> activity level
  activeDays: Record<number, number>; // day of week -> activity level
  deviceUsage: Record<string, number>; // device type -> percentage
  
  // Engagement patterns
  engagementByDemographic: Record<string, EngagementMetrics>;
  loyaltyMetrics: AudienceLoyalty;
  
  // Audience quality
  audienceQuality: AudienceQuality;
}

export interface EngagementMetrics {
  averageEngagementRate: number;
  averageTimeSpent: number;
  averageInteractionsPerSession: number;
  returnRate: number;
}

export interface AudienceLoyalty {
  returnVisitorRate: number;
  averageSessionLength: number;
  loyaltyScore: number; // 0-1, how loyal the audience is
  churnRate: number; // percentage of audience lost per month
}

export interface AudienceQuality {
  realUserPercentage: number; // percentage of real vs bot users
  engagedUserPercentage: number; // percentage of highly engaged users
  valuableUserPercentage: number; // percentage of high-value users
  audienceOverlapWithCompetitors: number; // 0-1, how much overlap
}

export interface OptimalTiming {
  bestHours: number[]; // hours of day (0-23)
  bestDays: number[]; // days of week (0-6)
  timezoneSensitive: boolean;
  
  // Performance by time
  hourlyPerformance: Record<number, number>; // hour -> avg engagement rate
  dailyPerformance: Record<number, number>; // day -> avg engagement rate
  
  // Seasonal patterns
  seasonalPatterns: SeasonalPattern[];
  
  // Recommendations
  recommendedPostTimes: RecommendedTime[];
}

export interface SeasonalPattern {
  season: string;
  pattern: string;
  strength: number; // how strong the seasonal effect is
  description: string;
}

export interface RecommendedTime {
  dayOfWeek: number;
  hour: number;
  expectedEngagementBoost: number; // percentage boost expected
  confidence: number; // confidence in this recommendation
  reasoning: string;
}

export interface PlatformSpecificInsights {
  // Instagram specific
  storyPerformance?: StoryAnalytics;
  reelPerformance?: ReelAnalytics;
  igtv Performance?: IGTVAnalytics;
  
  // TikTok specific
  hashtagTrends?: HashtagTrendAnalytics;
  soundTrends?: SoundTrendAnalytics;
  effectsUsage?: EffectsAnalytics;
  
  // YouTube specific
  videoMetrics?: YouTubeVideoAnalytics;
  searchRankings?: SearchRankingAnalytics;
  thumbnailPerformance?: ThumbnailAnalytics;
  
  // Twitter/X specific
  tweetTypes?: TweetTypeAnalytics;
  threadPerformance?: ThreadAnalytics;
  retweetAnalysis?: RetweetAnalytics;
  
  // Facebook specific
  pageInsights?: FacebookPageAnalytics;
  adPerformance?: FacebookAdAnalytics;
  groupActivity?: GroupActivityAnalytics;
}

// Platform-specific analytics interfaces would be defined here...
// (Truncated for brevity, but would include detailed analytics for each platform)

export interface ContentAnalytics {
  // Content performance overview
  totalContent: number;
  averagePerformance: ContentPerformanceMetrics;
  
  // Content type analysis
  performanceByType: Record<ContentType, ContentPerformanceMetrics>;
  
  // Topic analysis
  topPerformingTopics: TopicPerformance[];
  trendingTopics: TrendingTopic[];
  
  // Quality analysis
  qualityDistribution: QualityDistribution;
  qualityTrends: QualityTrends;
  
  // Content lifecycle
  contentLifecycle: ContentLifecycleAnalytics;
  
  // Optimization opportunities
  optimizationOpportunities: OptimizationOpportunity[];
}

export interface ContentPerformanceMetrics {
  averageViews: number;
  averageLikes: number;
  averageComments: number;
  averageShares: number;
  averageEngagementRate: number;
  averageQualityScore: number;
  averageXPEarned: number;
  averageFINEarned: number;
}

export interface TopicPerformance {
  topic: string;
  contentCount: number;
  averageEngagementRate: number;
  totalViews: number;
  trendinessScore: number;
  competitionLevel: 'low' | 'medium' | 'high';
  recommendationScore: number;
}

export interface TrendingTopic {
  topic: string;
  trendScore: number;
  growthRate: number;
  peakTime: Date;
  platforms: SocialPlatform[];
  relatedHashtags: string[];
  opportunityLevel: 'low' | 'medium' | 'high';
}

export interface QualityDistribution {
  low: number;     // percentage of content in each quality tier
  medium: number;
  high: number;
  premium: number;
  viral: number;
  
  // Quality progression over time
  qualityProgression: QualityProgressionPoint[];
}

export interface QualityProgressionPoint {
  date: Date;
  averageQuality: number;
  qualityConsistency: number; // how consistent quality is
  improvementRate: number; // rate of quality improvement
}

export interface QualityTrends {
  overallTrend: 'improving' | 'declining' | 'stable';
  trendStrength: number; // how strong the trend is
  
  // Factors affecting quality
  improvingFactors: string[];
  decliningFactors: string[];
  
  // Predictions
  predictedQuality: number; // predicted quality score for next period
  qualityGoals: QualityGoal[];
}

export interface QualityGoal {
  target: number;
  timeframe: number; // days to achieve
  probability: number; // probability of achieving goal
  requiredActions: string[];
}

export interface ContentLifecycleAnalytics {
  // Average lifecycle metrics
  averageTimeToViralityy: number; // hours
  averagePeakTime: number; // hours after publish
  averageContentLifespan: number; // days content stays relevant
  
  // Lifecycle stages
  initialPerformance: PerformanceMetrics; // first 24 hours
  earlyPerformance: PerformanceMetrics; // first week
  maturePerformance: PerformanceMetrics; // after first week
  
  // Decay analysis
  engagementDecayRate: number; // how fast engagement drops
  longtailPerformance: number; // how well content performs long-term
  
  // Viral analysis
  viralityPredictors: ViralityPredictor[];
  viralContentCharacteristics: ContentCharacteristics[];
}

export interface PerformanceMetrics {
  averageViews: number;
  averageEngagementRate: number;
  averageQualityScore: number;
  viralityRate: number; // percentage that go viral in this stage
}

export interface ViralityPredictor {
  factor: string;
  importance: number; // 0-1, how important this factor is
  currentValue: number;
  optimalValue: number;
  improvementPotential: number;
}

export interface OptimizationOpportunity {
  type: 'content_type' | 'timing' | 'hashtags' | 'platform' | 'quality' | 'audience';
  description: string;
  potentialImprovement: number; // percentage improvement expected
  effort: 'low' | 'medium' | 'high';
  priority: 'low' | 'medium' | 'high';
  
  // Specific recommendations
  specificActions: OptimizationAction[];
  
  // Success probability
  successProbability: number;
  expectedTimeframe: number; // days to see results
}

export interface OptimizationAction {
  action: string;
  description: string;
  resources: string[]; // what resources are needed
  timeline: string; // when to implement
  measurableOutcome: string; // how to measure success
}

export interface AudienceInsights {
  // Audience overview
  totalAudience: number;
  audienceGrowthRate: number;
  audienceRetentionRate: number;
  
  // Demographic insights
  demographics: DetailedDemographics;
  
  // Behavioral insights
  behaviorPatterns: BehaviorPatterns;
  
  // Interest analysis
  interestAnalysis: InterestAnalysis;
  
  // Audience segments
  segments: AudienceSegment[];
  
  // Audience journey
  customerJourney: CustomerJourneyAnalytics;
  
  // Engagement insights
  engagementInsights: DetailedEngagementInsights;
}

export interface DetailedDemographics {
  age: AgeDistributionDetailed;
  gender: GenderDistribution;
  location: LocationDistribution;
  language: LanguageDistribution;
  education: EducationDistribution;
  interests: InterestDistribution;
  deviceUsage: DeviceDistribution;
  incomeLevel: IncomeDistribution;
}

export interface AgeDistributionDetailed {
  '13-17': number;
  '18-24': number;
  '25-34': number;
  '35-44': number;
  '45-54': number;
  '55-64': number;
  '65+': number;
  
  averageAge: number;
  medianAge: number;
  ageVariance: number;
}

export interface GenderDistribution {
  male: number;
  female: number;
  nonBinary: number;
  preferNotToSay: number;
  unknown: number;
}

export interface LocationDistribution {
  countries: Record<string, number>;
  cities: Record<string, number>;
  regions: Record<string, number>;
  timezones: Record<string, number>;
  
  // Geographic diversity metrics
  geographicDiversity: number; // 0-1, how geographically diverse
  primaryMarket: string;
  secondaryMarkets: string[];
}

export interface LanguageDistribution {
  languages: Record<string, number>;
  primaryLanguage: string;
  multilingualUsers: number; // percentage who use multiple languages
  languageDiversity: number; // 0-1, how linguistically diverse
}

export interface EducationDistribution {
  highSchool: number;
  college: number;
  graduate: number;
  postgraduate: number;
  other: number;
  unknown: number;
}

export interface InterestDistribution {
  categories: Record<string, number>;
  subcategories: Record<string, number>;
  brands: Record<string, number>;
  topics: Record<string, number>;
  
  // Interest evolution
  growingInterests: string[];
  decliningInterests: string[];
  stableInterests: string[];
}

export interface DeviceDistribution {
  mobile: number;
  desktop: number;
  tablet: number;
  smartTV: number;
  other: number;
  
  // Operating systems
  ios: number;
  android: number;
  windows: number;
  mac: number;
  linux: number;
}

export interface IncomeDistribution {
  low: number;      // <$30k
  medium: number;   // $30k-$80k
  high: number;     // $80k-$150k
  premium: number;  // >$150k
  unknown: number;
}

export interface BehaviorPatterns {
  // Activity patterns
  dailyActivityPattern: Record<number, number>; // hour -> activity level
  weeklyActivityPattern: Record<number, number>; // day -> activity level
  monthlyActivityPattern: Record<number, number>; // day of month -> activity
  
  // Engagement patterns
  engagementDepth: EngagementDepthAnalysis;
  contentPreferences: ContentPreferences;
  platformUsagePatterns: PlatformUsagePatterns;
  
  // Interaction patterns
  interactionTypes: Record<ActivityType, number>;
  averageSessionDuration: number;
  averageInteractionsPerSession: number;
  
  // Loyalty patterns
  loyaltyMetrics: DetailedLoyaltyMetrics;
  churnRiskFactors: ChurnRiskFactor[];
  
  // Social behavior
  socialBehavior: SocialBehaviorAnalysis;
}

export interface EngagementDepthAnalysis {
  // Engagement levels
  superficial: number;  // just likes
  moderate: number;     // likes + comments
  deep: number;         // likes + comments + shares
  advocate: number;     // creates content about you
  
  // Engagement progression
  engagementJourney: EngagementJourneyStep[];
  
  // Depth metrics
  averageEngagementActions: number;
  averageTimeSpent: number;
  contentCompletionRate: number;
}

export interface EngagementJourneyStep {
  step: string;
  percentage: number; // what % of audience reaches this step
  averageTimeToReach: number; // days
  dropOffRate: number; // what % drop off at this step
  keyTriggers: string[]; // what causes progression to this step
}

export interface ContentPreferences {
  preferredTypes: Record<ContentType, number>;
  preferredTopics: Record<string, number>;
  preferredLength: ContentLengthPreference;
  preferredStyle: ContentStylePreference;
  
  // Temporal preferences
  preferredTiming: TimingPreferences;
  
  // Quality preferences
  qualityThreshold: number;
  qualitySensitivity: number;
}

export interface ContentLengthPreference {
  text: {
    short: number;    // <100 words
    medium: number;   // 100-300 words
    long: number;     // >300 words
  };
  video: {
    short: number;    // <30 seconds
    medium: number;   // 30s-3min
    long: number;     // >3 minutes
  };
}

export interface ContentStylePreference {
  formal: number;
  casual: number;
  humorous: number;
  educational: number;
  entertaining: number;
  inspirational: number;
  promotional: number;
}

export interface TimingPreferences {
  preferredHours: number[];
  preferredDays: number[];
  timezoneSensitivity: number;
  seasonalPreferences: Record<string, number>;
}

export interface PlatformUsagePatterns {
  primaryPlatform: SocialPlatform;
  platformDistribution: Record<SocialPlatform, number>;
  crossPlatformBehavior: CrossPlatformBehavior;
  platformLoyalty: Record<SocialPlatform, number>;
}

export interface CrossPlatformBehavior {
  multiPlatformUsers: number; // percentage using multiple platforms
  platformSynergy: Record<string, number>; // platform combinations
  crossPlatformEngagement: number; // how engagement varies across platforms
  migrationPatterns: PlatformMigration[];
}

export interface PlatformMigration {
  fromPlatform: SocialPlatform;
  toPlatform: SocialPlatform;
  migrationRate: number;
  commonReasons: string[];
  timeframe: number; // average days to migrate
}

export interface DetailedLoyaltyMetrics {
  overallLoyalty: number; // 0-1
  loyaltyFactors: LoyaltyFactor[];
  loyaltyTiers: LoyaltyTier[];
  
  // Retention metrics
  day1Retention: number;
  day7Retention: number;
  day30Retention: number;
  day90Retention: number;
  
  // Engagement consistency
  engagementConsistency: number; // how consistent engagement is over time
  seasonalLoyalty: Record<string, number>; // seasonal loyalty variations
}

export interface LoyaltyFactor {
  factor: string;
  impact: number; // how much this affects loyalty (-1 to 1)
  currentLevel: number; // current level for this user
  improvementPotential: number; // potential for improvement
}

export interface LoyaltyTier {
  tier: string;
  percentage: number; // what % of audience is in this tier
  characteristics: string[];
  averageValue: number; // average value of users in this tier
  churnRate: number;
}

export interface ChurnRiskFactor {
  factor: string;
  riskLevel: 'low' | 'medium' | 'high';
  affectedPercentage: number;
  predictivePower: number; // how predictive this factor is
  intervention: string; // suggested intervention
}

export interface SocialBehaviorAnalysis {
  // Sharing behavior
  sharingPropensity: number; // how likely to share content
  shareTypes: Record<string, number>; // what types of content they share
  viralityContribution: number; // how much they contribute to virality
  
  // Influence behavior
  influenceLevel: number; // how much they influence others
  influenceType: 'thought_leader' | 'connector' | 'maven' | 'casual';
  networkPosition: 'central' | 'peripheral' | 'bridge' | 'isolate';
  
  // Community behavior
  communityParticipation: number; // how active in communities
  leadershipPotential: number; // potential to become community leader
  collaborationTendency: number; // tendency to collaborate
}

export interface InterestAnalysis {
  // Interest categories
  primaryInterests: InterestCategory[];
  secondaryInterests: InterestCategory[];
  emergingInterests: InterestCategory[];
  decliningInterests: InterestCategory[];
  
  // Interest evolution
  interestStability: number; // how stable interests are over time
  interestDiversity: number; // how diverse interests are
  
  // Interest-based segmentation
  interestSegments: InterestSegment[];
  
  // Brand affinity
  brandAffinities: BrandAffinity[];
  
  // Content interest mapping
  contentInterestMap: ContentInterestMapping;
}

export interface InterestCategory {
  category: string;
  subcategories: string[];
  engagementLevel: number; // how engaged with this interest
  growthRate: number; // how this interest is growing/declining
  commercialIntent: number; // likelihood to purchase related products
  influenceLevel: number; // how much they influence others in this area
}

export interface InterestSegment {
  segmentName: string;
  interests: string[];
  audienceSize: number;
  engagementLevel: number;
  monetizationPotential: number;
  contentOpportunities: string[];
}

export interface BrandAffinity {
  brand: string;
  affinityScore: number; // 0-1
  engagementHistory: BrandEngagementHistory;
  purchaseIntent: number; // 0-1
  brandLoyalty: number; // 0-1
  influenceOnBrand: number; // how much they influence brand perception
}

export interface BrandEngagementHistory {
  totalInteractions: number;
  lastInteraction: Date;
  interactionTypes: Record<string, number>;
  sentimentHistory: SentimentTrend[];
  campaignParticipation: string[]; // campaigns they participated in
}

export interface SentimentTrend {
  date: Date;
  sentiment: 'positive' | 'neutral' | 'negative';
  intensity: number; // 0-1
  context: string; // what caused this sentiment
}

export interface ContentInterestMapping {
  interestToContent: Record<string, ContentType[]>;
  contentToInterest: Record<ContentType, string[]>;
  performanceByInterest: Record<string, number>;
  opportunityMatrix: InterestOpportunityMatrix;
}

export interface InterestOpportunityMatrix {
  highInterestHighPerformance: string[];
  highInterestLowPerformance: string[];
  lowInterestHighPerformance: string[];
  lowInterestLowPerformance: string[];
}

export interface AudienceSegment {
  segmentId: string;
  segmentName: string;
  description: string;
  
  // Segment characteristics
  size: number;
  percentage: number;
  
  // Demographics
  demographics: SegmentDemographics;
  
  // Behavior patterns
  behaviorProfile: SegmentBehaviorProfile;
  
  // Value metrics
  valueMetrics: SegmentValueMetrics;
  
  // Engagement patterns
  engagementProfile: SegmentEngagementProfile;
  
  // Growth potential
  growthPotential: SegmentGrowthPotential;
  
  // Recommendations
  recommendedStrategies: SegmentStrategy[];
}

export interface SegmentDemographics {
  dominantAge: string;
  dominantGender: string;
  dominantLocation: string;
  dominantLanguage: string;
  dominantEducation: string;
  dominantIncome: string;
}

export interface SegmentBehaviorProfile {
  activityLevel: 'low' | 'medium' | 'high';
  engagementDepth: 'superficial' | 'moderate' | 'deep';
  platformPreference: SocialPlatform;
  contentPreference: ContentType;
  timingPreference: string;
  loyaltyLevel: 'low' | 'medium' | 'high';
}

export interface SegmentValueMetrics {
  averageXPGenerated: number;
  averageFINGenerated: number;
  averageRPContribution: number;
  lifetimeValue: number;
  acquisitionCost: number;
  roi: number;
}

export interface SegmentEngagementProfile {
  engagementRate: number;
  preferredEngagementTypes: ActivityType[];
  responseTime: number; // average response time to content
  viralityPotential: number;
  influenceLevel: number;
}

export interface SegmentGrowthPotential {
  growthRate: number; // monthly growth rate
  maxPotentialSize: number;
  timeToMaxSize: number; // months
  growthFactors: string[];
  growthBarriers: string[];
}

export interface SegmentStrategy {
  strategy: string;
  description: string;
  expectedOutcome: string;
  implementation: ImplementationPlan;
  metrics: string[]; // how to measure success
}

export interface ImplementationPlan {
  phases: ImplementationPhase[];
  totalDuration: number; // weeks
  resources: string[];
  budget: number;
  risks: string[];
}

export interface ImplementationPhase {
  phase: string;
  duration: number; // weeks
  activities: string[];
  deliverables: string[];
  successCriteria: string[];
}

export interface CustomerJourneyAnalytics {
  // Journey stages
  stages: JourneyStage[];
  
  // Conversion funnel
  conversionFunnel: ConversionFunnel;
  
  // Touchpoints
  touchpoints: Touchpoint[];
  
  // Journey insights
  insights: JourneyInsight[];
  
  // Optimization opportunities
  optimizationOpportunities: JourneyOptimization[];
}

export interface JourneyStage {
  stage: string;
  description: string;
  averageDuration: number; // days in this stage
  completionRate: number; // % who complete this stage
  dropOffRate: number; // % who drop off at this stage
  keyActions: string[];
  successMetrics: Record<string, number>;
}

export interface ConversionFunnel {
  stages: FunnelStage[];
  overallConversionRate: number;
  averageTimeToConvert: number; // days
  conversionFactors: ConversionFactor[];
}

export interface FunnelStage {
  stage: string;
  entrants: number;
  completions: number;
  conversionRate: number;
  averageTime: number; // days to complete this stage
  dropOffReasons: string[];
}

export interface ConversionFactor {
  factor: string;
  impact: number; // how much this affects conversion rate
  controllable: boolean; // whether we can influence this factor
  currentLevel: number;
  optimizationPotential: number;
}

export interface Touchpoint {
  touchpoint: string;
  platform: SocialPlatform;
  frequency: number; // how often users interact at this touchpoint
  effectiveness: number; // how effective this touchpoint is
  sentimentAtTouchpoint: number; // average sentiment at this touchpoint
  conversionContribution: number; // how much this contributes to conversion
}

export interface JourneyInsight {
  insight: string;
  category: 'behavior' | 'preference' | 'barrier' | 'opportunity';
  impact: 'low' | 'medium' | 'high';
  confidence: number; // confidence in this insight
  supportingData: string[];
  actionableRecommendations: string[];
}

export interface JourneyOptimization {
  opportunity: string;
  stage: string;
  expectedImprovement: number; // expected % improvement
  effort: 'low' | 'medium' | 'high';
  priority: 'low' | 'medium' | 'high';
  implementation: string;
  timeline: number; // weeks to implement
}

export interface DetailedEngagementInsights {
  // Overall engagement
  overallEngagementHealth: EngagementHealth;
  
  // Engagement drivers
  engagementDrivers: EngagementDriver[];
  
  // Engagement barriers
  engagementBarriers: EngagementBarrier[];
  
  // Engagement patterns
  engagementPatterns: EngagementPattern[];
  
  // Peer comparisons
  peerComparisons: PeerComparison[];
  
  // Engagement predictions
  engagementPredictions: EngagementPrediction[];
}

export interface EngagementHealth {
  overallScore: number; // 0-100
  trend: 'improving' | 'stable' | 'declining';
  healthFactors: HealthFactor[];
  riskFactors: RiskFactor[];
  recommendations: HealthRecommendation[];
}

export interface HealthFactor {
  factor: string;
  contribution: number; // how much this contributes to health score
  currentLevel: number;
  benchmarkLevel: number;
  trend: 'improving' | 'stable' | 'declining';
}

export interface RiskFactor {
  factor: string;
  riskLevel: 'low' | 'medium' | 'high';
  probability: number; // probability this will become a problem
  impact: number; // impact if this becomes a problem
  mitigation: string; // how to mitigate this risk
}

export interface HealthRecommendation {
  recommendation: string;
  priority: 'low' | 'medium' | 'high';
  expectedImpact: number; // expected improvement in health score
  effort: 'low' | 'medium' | 'high';
  timeline: string;
}

export interface EngagementDriver {
  driver: string;
  importance: number; // 0-1
  currentPerformance: number; // how well we're leveraging this driver
  optimizationPotential: number; // how much we could improve
  tactics: string[]; // specific tactics to leverage this driver
}

export interface EngagementBarrier {
  barrier: string;
  severity: 'low' | 'medium' | 'high';
  affectedAudience: number; // % of audience affected
  removalDifficulty: 'easy' | 'medium' | 'hard';
  removalStrategies: string[];
  expectedImpact: number; // expected engagement improvement if removed
}

export interface EngagementPattern {
  pattern: string;
  description: string;
  frequency: number; // how often this pattern occurs
  trigger: string; // what triggers this pattern
  outcome: string; // typical outcome
  leverageOpportunity: string; // how to leverage this pattern
}

export interface PeerComparison {
  metric: string;
  userValue: number;
  peerAverage: number;
  peerPercentile: number; // what percentile the user is in
  topPerformerValue: number;
  gapToPeers: number;
  gapToTopPerformer: number;
  improvementPotential: number;
}

export interface EngagementPrediction {
  metric: string;
  currentValue: number;
  predictedValue: number;
  timeframe: number; // days
  confidence: number; // confidence in prediction
  factors: PredictionFactor[];
  scenarios: PredictionScenario[];
}

export interface PredictionFactor {
  factor: string;
  influence: number; // how much this factor influences the prediction
  direction: 'positive' | 'negative' | 'neutral';
  controllable: boolean;
}

export interface PredictionScenario {
  scenario: string;
  probability: number;
  predictedOutcome: number;
  keyAssumptions: string[];
  actionImplications: string[];
}

export interface GrowthMetrics {
  // Overall growth
  overallGrowthRate: number; // monthly growth rate
  growthTrend: 'accelerating' | 'stable' | 'decelerating';
  
  // Growth components
  organicGrowth: GrowthComponent;
  referralGrowth: GrowthComponent;
  campaignGrowth: GrowthComponent;
  
  // Growth by segment
  segmentGrowth: Record<string, SegmentGrowthMetrics>;
  
  // Growth drivers
  growthDrivers: GrowthDriver[];
  
  // Growth barriers
  growthBarriers: GrowthBarrier[];
  
  // Growth predictions
  growthPredictions: GrowthPrediction[];
  
  // Benchmarking
  competitiveBenchmarks: CompetitiveBenchmark[];
}

export interface GrowthComponent {
  growthRate: number;
  contribution: number; // % of total growth from this component
  efficiency: number; // cost per acquisition for this component
  quality: number; // quality score of users acquired through this component
  scalability: number; // how scalable this growth component is
}

export interface SegmentGrowthMetrics {
  growthRate: number;
  retentionRate: number;
  valueGrowth: number; // growth in value generated by this segment
  acquisitionCost: number;
  lifetimeValue: number;
  paybackPeriod: number; // months to payback acquisition cost
}

export interface GrowthDriver {
  driver: string;
  impact: number; // impact on growth rate
  sustainability: number; // how sustainable this driver is
  investmentRequired: number; // relative investment required
  timeToImpact: number; // months to see impact
  riskLevel: 'low' | 'medium' | 'high';
}

export interface GrowthBarrier {
  barrier: string;
  impact: number; // how much this limits growth
  prevalence: number; // how widespread this barrier is
  removalDifficulty: 'easy' | 'medium' | 'hard';
  removalCost: number; // relative cost to remove
  priority: 'low' | 'medium' | 'high';
}

export interface GrowthPrediction {
  timeframe: number; // months
  predictedGrowthRate: number;
  predictedUserCount: number;
  confidence: number;
  assumptions: string[];
  riskFactors: string[];
  opportunities: string[];
}

export interface CompetitiveBenchmark {
  competitor: string;
  metric: string;
  theirValue: number;
  ourValue: number;
  gap: number;
  importance: 'low' | 'medium' | 'high';
  catchUpDifficulty: 'easy' | 'medium' | 'hard';
  strategicImportance: number;
}

export interface RewardAnalytics {
  // Reward overview
  totalRewardsEarned: RewardSummary;
  rewardEfficiency: RewardEfficiency;
  
  // Reward trends
  rewardTrends: RewardTrends;
  
  // Reward optimization
  optimizationInsights: RewardOptimizationInsight[];
  
  // Comparative analysis
  peerComparisons: RewardPeerComparison[];
  
  // Prediction models
  rewardPredictions: RewardPrediction[];
  
  // ROI analysis
  rewardROI: RewardROIAnalysis;
}

export interface RewardSummary {
  xp: RewardTypeBreakdown;
  fin: RewardTypeBreakdown;
  rp: RewardTypeBreakdown;
  
  totalValue: number; // total USD equivalent value
  growthRate: number; // monthly growth rate
  consistency: number; // how consistent rewards are
}

export interface RewardTypeBreakdown {
  total: number;
  fromContent: number;
  fromActivity: number;
  fromReferrals: number;
  fromBonuses: number;
  fromCampaigns: number;
  
  averagePerDay: number;
  averagePerContent: number;
  efficiency: number; // rewards per unit of effort
}

export interface RewardEfficiency {
  overallEfficiency: number; // rewards per hour of activity
  efficiencyByActivity: Record<ActivityType, number>;
  efficiencyByPlatform: Record<SocialPlatform, number>;
  efficiencyByContentType: Record<ContentType, number>;
  
  improvementOpportunities: EfficiencyImprovement[];
  benchmarkComparison: EfficiencyBenchmark[];
}

export interface EfficiencyImprovement {
  area: string;
  currentEfficiency: number;
  potentialEfficiency: number;
  improvementStrategy: string;
  effort: 'low' | 'medium' | 'high';
  timeToResults: number; // days
}

export interface EfficiencyBenchmark {
  benchmark: string;
  userValue: number;
  benchmarkValue: number;
  percentile: number;
  improvementPotential: number;
}

export interface RewardTrends {
  xpTrend: TrendData;
  finTrend: TrendData;
  rpTrend: TrendData;
  
  seasonalPatterns: SeasonalRewardPattern[];
  cyclicalPatterns: CyclicalPattern[];
  
  trendPredictions: TrendPrediction[];
}

export interface SeasonalRewardPattern {
  season: string;
  pattern: 'increase' | 'decrease' | 'stable';
  magnitude: number; // how much it increases/decreases
  duration: number; // how long the pattern lasts
  reliability: number; // how reliable this pattern is
}

export interface CyclicalPattern {
  type: 'daily' | 'weekly' | 'monthly';
  pattern: number[]; // values for each period in the cycle
  strength: number; // how strong the cyclical pattern is
  phase: number; // where we currently are in the cycle
}

export interface TrendPrediction {
  rewardType: 'xp' | 'fin' | 'rp';
  timeframe: number; // days
  predictedValue: number;
  confidence: number;
  factors: string[];
}

export interface RewardOptimizationInsight {
  insight: string;
  category: 'efficiency' | 'volume' | 'quality' | 'timing' | 'platform';
  impact: 'low' | 'medium' | 'high';
  effort: 'low' | 'medium' | 'high';
  
  currentState: number;
  potentialState: number;
  improvementPotential: number;
  
  actionItems: OptimizationAction[];
  successMetrics: string[];
  timeline: number; // weeks to implement
}

export interface RewardPeerComparison {
  metric: string;
  userValue: number;
  peerTierAverage: number;
  globalAverage: number;
  topPerformerValue: number;
  
  percentileInTier: number;
  globalPercentile: number;
  
  gapToPeers: number;
  gapToTop: number;
  
  catchUpStrategy: string;
  catchUpTimeframe: number; // days
}

export interface RewardPrediction {
  type: 'xp' | 'fin' | 'rp';
  timeframe: number; // days
  
  conservative: PredictionValue;
  realistic: PredictionValue;
  optimistic: PredictionValue;
  
  keyAssumptions: string[];
  riskFactors: string[];
  opportunities: string[];
}

export interface PredictionValue {
  value: number;
  probability: number; // probability of achieving this value
  requiredActions: string[];
}

export interface RewardROIAnalysis {
  // Time investment
  timeInvested: number; // hours per week
  rewardPerHour: number;
  
  // Quality vs quantity
  qualityROI: number; // return on quality investment
  quantityROI: number; // return on quantity investment
  
  // Platform ROI
  platformROI: Record<SocialPlatform, PlatformROI>;
  
  // Activity ROI
  activityROI: Record<ActivityType, ActivityROI>;
  
  // Optimization recommendations
  roiOptimizations: ROIOptimization[];
}

export interface PlatformROI {
  timeInvested: number;
  rewardsEarned: number;
  roi: number;
  
  efficiency: number;
  potential: number; // untapped potential
  recommendations: string[];
}

export interface ActivityROI {
  timeInvested: number;
  rewardsEarned: number;
  roi: number;
  
  scalability: number; // how scalable this activity is
  sustainability: number; // how sustainable the ROI is
  optimization: string[]; // how to optimize this activity
}

export interface ROIOptimization {
  optimization: string;
  currentROI: number;
  projectedROI: number;
  improvement: number;
  
  implementation: ImplementationPlan;
  riskLevel: 'low' | 'medium' | 'high';
  timeToResults: number; // days
}

export interface CompetitiveAnalysis {
  // Competitor identification
  competitors: Competitor[];
  
  // Market positioning
  marketPosition: MarketPosition;
  
  // Performance comparison
  performanceComparison: PerformanceComparison;
  
  // Opportunity analysis
  competitiveOpportunities: CompetitiveOpportunity[];
  
  // Threat analysis
  competitiveThreats: CompetitiveThreat[];
  
  // Strategic recommendations
  strategicRecommendations: StrategicRecommendation[];
}

export interface Competitor {
  name: string;
  type: 'direct' | 'indirect' | 'substitute';
  
  // Basic metrics
  followerCount: number;
  engagementRate: number;
  contentFrequency: number;
  
  // Strengths and weaknesses
  strengths: string[];
  weaknesses: string[];
  
  // Strategy insights
  strategy: CompetitorStrategy;
  
  // Performance trends
  trends: CompetitorTrends;
  
  // Threat level
  threatLevel: 'low' | 'medium' | 'high';
}

export interface CompetitorStrategy {
  contentStrategy: string;
  engagementStrategy: string;
  growthStrategy: string;
  platformFocus: SocialPlatform[];
  targetAudience: string;
  differentiators: string[];
}

export interface CompetitorTrends {
  growthRate: number;
  engagementTrend: 'up' | 'down' | 'stable';
  contentQualityTrend: 'improving' | 'stable' | 'declining';
  innovationRate: number; // how often they try new things
}

export interface MarketPosition {
  // Overall position
  overallRank: number;
  marketShare: number;
  
  // Category positions
  categoryRankings: Record<string, number>;
  
  // Positioning strengths
  strengths: PositioningStrength[];
  
  // Positioning gaps
  gaps: PositioningGap[];
  
  // Unique value proposition
  uniqueValue: string[];
  competitiveAdvantages: string[];
}

export interface PositioningStrength {
  strength: string;
  advantage: number; // how much better than competitors
  sustainability: 'low' | 'medium' | 'high';
  leverageOpportunity: string;
}

export interface PositioningGap {
  gap: string;
  severity: 'low' | 'medium' | 'high';
  competitorAdvantage: number;
  closingDifficulty: 'easy' | 'medium' | 'hard';
  closingStrategy: string;
}

export interface PerformanceComparison {
  // Key metrics comparison
  metricsComparison: MetricComparison[];
  
  // Performance gaps
  performanceGaps: PerformanceGap[];
  
  // Performance advantages
  performanceAdvantages: PerformanceAdvantage[];
  
  // Benchmarking insights
  benchmarkInsights: BenchmarkInsight[];
}

export interface MetricComparison {
  metric: string;
  userValue: number;
  competitorAverage: number;
  topCompetitor: number;
  marketLeader: number;
  
  relativePerformance: number; // user value / competitor average
  gap: number; // difference from market leader