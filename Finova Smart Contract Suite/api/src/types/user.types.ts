// finova-net/api/src/types/user.types.ts

import { PublicKey } from '@solana/web3.js';

// ================================
// BASE USER INTERFACES
// ================================

export interface BaseUser {
  id: string;
  walletAddress: PublicKey | string;
  email?: string;
  username: string;
  displayName: string;
  avatar?: string;
  bio?: string;
  country: string;
  timezone: string;
  language: string;
  isActive: boolean;
  createdAt: Date;
  updatedAt: Date;
}

export interface UserProfile extends BaseUser {
  // Authentication & Security
  authProvider: AuthProvider;
  kycStatus: KYCStatus;
  kycData?: KYCData;
  securitySettings: SecuritySettings;
  deviceFingerprint?: string[];
  
  // Core Stats
  totalFINEarned: number;
  totalFINBalance: number;
  totalXP: number;
  currentLevel: number;
  totalRP: number;
  rpTier: RPTier;
  
  // Mining Data
  miningData: MiningData;
  
  // Social Integration
  socialAccounts: SocialAccount[];
  
  // Referral Network
  referralData: ReferralData;
  
  // Staking Info
  stakingData?: StakingData;
  
  // Guild Membership
  guildMembership?: GuildMembership;
  
  // Settings & Preferences
  preferences: UserPreferences;
  notificationSettings: NotificationSettings;
  
  // Timestamps
  lastLoginAt: Date;
  lastMiningAt?: Date;
  lastActivityAt: Date;
}

// ================================
// AUTHENTICATION & SECURITY
// ================================

export enum AuthProvider {
  WALLET = 'wallet',
  GOOGLE = 'google',
  APPLE = 'apple',
  FACEBOOK = 'facebook',
  TWITTER = 'twitter'
}

export enum KYCStatus {
  PENDING = 'pending',
  SUBMITTED = 'submitted',
  UNDER_REVIEW = 'under_review',
  APPROVED = 'approved',
  REJECTED = 'rejected',
  EXPIRED = 'expired'
}

export interface KYCData {
  provider: string;
  documentType: 'passport' | 'national_id' | 'driving_license';
  documentNumber: string;
  fullName: string;
  dateOfBirth: Date;
  nationality: string;
  address: {
    street: string;
    city: string;
    state: string;
    postalCode: string;
    country: string;
  };
  biometricHash?: string;
  submittedAt: Date;
  approvedAt?: Date;
  expiresAt?: Date;
}

export interface SecuritySettings {
  twoFactorEnabled: boolean;
  biometricEnabled: boolean;
  loginNotifications: boolean;
  suspiciousActivityAlerts: boolean;
  deviceLockout: boolean;
  sessionTimeout: number; // minutes
}

// ================================
// XP SYSTEM
// ================================

export interface XPData {
  totalXP: number;
  currentLevel: number;
  xpToNextLevel: number;
  levelProgress: number; // 0-1
  weeklyXP: number;
  monthlyXP: number;
  
  // Level milestones
  levelHistory: LevelMilestone[];
  
  // Activity breakdown
  xpByActivity: {
    posts: number;
    comments: number;
    likes: number;
    shares: number;
    dailyLogin: number;
    quests: number;
    viral: number;
    referrals: number;
  };
  
  // Multipliers
  activeMultipliers: XPMultiplier[];
  
  // Streaks
  currentStreak: number;
  longestStreak: number;
  lastActivityDate: Date;
}

export interface LevelMilestone {
  level: number;
  reachedAt: Date;
  xpAtTime: number;
  rewards?: {
    finTokens?: number;
    nfts?: string[];
    badges?: string[];
    features?: string[];
  };
}

export interface XPMultiplier {
  source: string; // 'streak', 'card', 'event', 'guild'
  multiplier: number;
  expiresAt?: Date;
  isActive: boolean;
}

// ================================
// MINING SYSTEM
// ================================

export interface MiningData {
  // Current Mining Status
  isActiveMiner: boolean;
  currentMiningRate: number; // FIN per hour
  lastMiningUpdate: Date;
  nextMiningUpdate: Date;
  
  // Mining History
  totalMiningHours: number;
  totalFINMined: number;
  
  // Phase & Bonuses
  currentPhase: MiningPhase;
  baseRate: number;
  pioneerBonus: number;
  referralBonus: number;
  securityBonus: number;
  regressionFactor: number;
  
  // Active Boosts
  activeBoosts: MiningBoost[];
  
  // Daily Mining
  dailyMiningCap: number;
  dailyMiningEarned: number;
  dailyResetAt: Date;
  
  // Quality Score
  contentQualityScore: number; // 0.5 - 2.0
  qualityHistory: QualityScore[];
  
  // Anti-Bot Metrics
  humanProbability: number; // 0.0 - 1.0
  suspicionScore: number;
  behaviorPattern: BehaviorPattern;
}

export enum MiningPhase {
  FINIZEN = 'finizen', // 0-100K users
  GROWTH = 'growth',   // 100K-1M users
  MATURITY = 'maturity', // 1M-10M users
  STABILITY = 'stability' // 10M+ users
}

export interface MiningBoost {
  id: string;
  type: 'card' | 'event' | 'achievement' | 'staking';
  source: string;
  multiplier: number;
  duration: number; // hours
  startedAt: Date;
  expiresAt: Date;
  isActive: boolean;
}

export interface QualityScore {
  date: Date;
  score: number;
  factors: {
    originality: number;
    engagement: number;
    platformRelevance: number;
    brandSafety: number;
    humanGenerated: number;
  };
}

export interface BehaviorPattern {
  avgSessionDuration: number;
  clickSpeed: number;
  interactionVariance: number;
  temporalPatterns: number[];
  deviceConsistency: number;
  socialGraphValidity: number;
}

// ================================
// REFERRAL SYSTEM
// ================================

export interface ReferralData {
  // Referral Code & Link
  referralCode: string;
  customReferralCode?: string;
  referralLink: string;
  
  // Network Stats
  totalReferrals: number;
  activeReferrals: number; // active in last 30 days
  directReferrals: UserReferral[];
  
  // Network Levels
  level2Network: number;
  level3Network: number;
  totalNetworkSize: number;
  
  // RP Stats
  totalRP: number;
  currentRPTier: RPTier;
  rpMultiplier: number;
  
  // Network Quality
  networkQualityScore: number;
  retentionRate: number;
  avgReferralLevel: number;
  
  // Rewards
  totalReferralRewards: number;
  weeklyReferralRewards: number;
  monthlyReferralRewards: number;
  
  // Performance
  referralPerformance: ReferralPerformance;
}

export interface UserReferral {
  id: string;
  username: string;
  joinedAt: Date;
  isActive: boolean;
  totalFINEarned: number;
  currentLevel: number;
  lastActivityAt: Date;
  directReward: number;
}

export enum RPTier {
  EXPLORER = 'explorer',       // 0-999 RP
  CONNECTOR = 'connector',     // 1K-4.9K RP
  INFLUENCER = 'influencer',   // 5K-14.9K RP
  LEADER = 'leader',           // 15K-49.9K RP
  AMBASSADOR = 'ambassador'    // 50K+ RP
}

export interface ReferralPerformance {
  conversionRate: number; // signups to active users
  avgTimeToFirstMining: number; // hours
  avgLifetimeValue: number; // FIN
  churnRate: number; // 30-day
  networkGrowthRate: number; // weekly
}

// ================================
// SOCIAL INTEGRATION
// ================================

export interface SocialAccount {
  id: string;
  platform: SocialPlatform;
  platformUserId: string;
  username: string;
  displayName: string;
  profileUrl: string;
  avatar?: string;
  
  // Connection Status
  isConnected: boolean;
  connectedAt: Date;
  lastSyncAt: Date;
  
  // Platform Stats
  followers: number;
  following: number;
  postsCount: number;
  
  // Finova Integration
  totalXPEarned: number;
  totalPostsTracked: number;
  avgEngagementRate: number;
  platformMultiplier: number;
  
  // API Access
  accessToken?: string;
  refreshToken?: string;
  tokenExpiresAt?: Date;
  
  // Settings
  autoSync: boolean;
  trackPosts: boolean;
  trackComments: boolean;
  trackLikes: boolean;
}

export enum SocialPlatform {
  INSTAGRAM = 'instagram',
  TIKTOK = 'tiktok',
  YOUTUBE = 'youtube',
  FACEBOOK = 'facebook',
  TWITTER = 'twitter',
  LINKEDIN = 'linkedin'
}

// ================================
// STAKING SYSTEM
// ================================

export interface StakingData {
  // Current Stakes
  totalStaked: number; // FIN
  sFINBalance: number; // Staked FIN tokens
  
  // Staking Tier
  stakingTier: StakingTier;
  tierBenefits: TierBenefits;
  
  // Rewards
  totalStakingRewards: number;
  pendingRewards: number;
  lastRewardClaim: Date;
  
  // APY & Multipliers
  currentAPY: number;
  miningMultiplier: number;
  xpMultiplier: number;
  rpBonus: number;
  
  // Staking History
  stakingPositions: StakingPosition[];
  rewardHistory: StakingReward[];
  
  // Time-based Bonuses
  stakingDuration: number; // days
  loyaltyBonus: number;
  nextTierRequirement?: number;
}

export enum StakingTier {
  NONE = 'none',           // 0 FIN
  BRONZE = 'bronze',       // 100-499 FIN
  SILVER = 'silver',       // 500-999 FIN
  GOLD = 'gold',           // 1K-4.9K FIN
  PLATINUM = 'platinum',   // 5K-9.9K FIN
  DIAMOND = 'diamond'      // 10K+ FIN
}

export interface TierBenefits {
  miningBoost: number;
  xpMultiplier: number;
  rpBonus: number;
  features: string[];
  dailyFINCapBonus: number;
  specialAccess: string[];
}

export interface StakingPosition {
  id: string;
  amount: number;
  stakedAt: Date;
  lockPeriod?: number; // days
  unlockAt?: Date;
  apy: number;
  isActive: boolean;
}

export interface StakingReward {
  id: string;
  amount: number;
  type: 'base' | 'activity' | 'loyalty' | 'performance';
  earnedAt: Date;
  claimedAt?: Date;
  source: string;
}

// ================================
// GUILD SYSTEM
// ================================

export interface GuildMembership {
  guildId: string;
  guildName: string;
  role: GuildRole;
  joinedAt: Date;
  
  // Contribution Stats
  guildXPContributed: number;
  guildEventsParticipated: number;
  guildRank: number;
  
  // Benefits
  guildBonuses: {
    xpBonus: number;
    miningBonus: number;
    specialAccess: string[];
  };
  
  // Activity
  lastGuildActivity: Date;
  isActiveGuildMember: boolean;
}

export enum GuildRole {
  MEMBER = 'member',
  OFFICER = 'officer',
  LEADER = 'leader',
  MASTER = 'master'
}

// ================================
// USER PREFERENCES & SETTINGS
// ================================

export interface UserPreferences {
  // Display
  theme: 'light' | 'dark' | 'auto';
  language: string;
  timezone: string;
  currency: string;
  
  // Privacy
  profileVisibility: 'public' | 'friends' | 'private';
  showMiningStats: boolean;
  showReferralLink: boolean;
  showXPProgress: boolean;
  
  // Features
  autoMining: boolean;
  smartNotifications: boolean;
  socialAutoSync: boolean;
  qualityFilterEnabled: boolean;
  
  // Marketing
  allowMarketingEmails: boolean;
  shareDataForRecommendations: boolean;
  participateInBetaTesting: boolean;
}

export interface NotificationSettings {
  // Push Notifications
  pushEnabled: boolean;
  miningUpdates: boolean;
  xpMilestones: boolean;
  referralActivity: boolean;
  guildEvents: boolean;
  
  // Email Notifications
  emailEnabled: boolean;
  weeklyReports: boolean;
  monthlyReports: boolean;
  securityAlerts: boolean;
  marketingEmails: boolean;
  
  // In-App Notifications
  showAchievements: boolean;
  showLevelUps: boolean;
  showReferralSuccess: boolean;
  showMiningBoosts: boolean;
  
  // Quiet Hours
  quietHoursEnabled: boolean;
  quietHoursStart: string; // "22:00"
  quietHoursEnd: string;   // "08:00"
}

// ================================
// USER ACTIVITY & ANALYTICS
// ================================

export interface UserActivity {
  // Session Data
  totalSessions: number;
  avgSessionDuration: number;
  lastSessionDuration: number;
  
  // Engagement Metrics
  dailyActiveUser: boolean;
  weeklyActiveUser: boolean;
  monthlyActiveUser: boolean;
  
  // Content Activity
  postsCreated: number;
  commentsCreated: number;
  likesGiven: number;
  sharesCreated: number;
  
  // Platform Distribution
  activityByPlatform: Record<SocialPlatform, number>;
  
  // Time Patterns
  mostActiveHours: number[];
  mostActiveDays: string[];
  
  // Quality Metrics
  avgContentQuality: number;
  viralContentCount: number;
  originalContentPercentage: number;
}

// ================================
// API RESPONSE TYPES
// ================================

export interface UserResponse {
  user: UserProfile;
  stats: UserStats;
  permissions: UserPermissions;
}

export interface UserStats {
  // Rankings
  globalRank: number;
  countryRank: number;
  levelRank: number;
  
  // Comparisons
  percentileRank: number;
  avgUserComparison: {
    miningRate: number;
    xpGain: number;
    networkSize: number;
  };
  
  // Achievements
  achievementsCount: number;
  recentAchievements: Achievement[];
  nextMilestones: Milestone[];
}

export interface UserPermissions {
  canMine: boolean;
  canRefer: boolean;
  canStake: boolean;
  canTrade: boolean;
  canParticipateInGuild: boolean;
  canAccessPremiumFeatures: boolean;
  apiAccess: boolean;
}

export interface Achievement {
  id: string;
  name: string;
  description: string;
  icon: string;
  rarity: 'common' | 'rare' | 'epic' | 'legendary';
  unlockedAt: Date;
  rewards: {
    fin?: number;
    xp?: number;
    nft?: string;
    badge?: string;
  };
}

export interface Milestone {
  id: string;
  name: string;
  description: string;
  targetValue: number;
  currentValue: number;
  progress: number; // 0-1
  estimatedCompletion?: Date;
  rewards: {
    fin?: number;
    xp?: number;
    nft?: string;
    feature?: string;
  };
}

// ================================
// CREATE & UPDATE TYPES
// ================================

export interface CreateUserInput {
  walletAddress: string;
  username: string;
  email?: string;
  displayName?: string;
  country: string;
  referralCode?: string;
  authProvider: AuthProvider;
  deviceFingerprint?: string;
}

export interface UpdateUserInput {
  displayName?: string;
  bio?: string;
  avatar?: string;
  country?: string;
  timezone?: string;
  language?: string;
  preferences?: Partial<UserPreferences>;
  notificationSettings?: Partial<NotificationSettings>;
}

export interface UserQuery {
  id?: string;
  username?: string;
  walletAddress?: string;
  email?: string;
  kycStatus?: KYCStatus;
  isActive?: boolean;
  minLevel?: number;
  minMiningRate?: number;
  rpTier?: RPTier;
  stakingTier?: StakingTier;
  country?: string;
  createdAfter?: Date;
  createdBefore?: Date;
  lastActiveAfter?: Date;
  lastActiveBefore?: Date;
}

// ================================
// UTILITY TYPES
// ================================

export type UserSortField = 
  | 'createdAt' 
  | 'lastActivityAt' 
  | 'totalFINEarned' 
  | 'currentLevel' 
  | 'totalRP' 
  | 'referralCount'
  | 'miningRate';

export type SortDirection = 'asc' | 'desc';

export interface UserListQuery extends UserQuery {
  page?: number;
  limit?: number;
  sortBy?: UserSortField;
  sortDirection?: SortDirection;
  search?: string;
}

export interface UserListResponse {
  users: UserProfile[];
  pagination: {
    page: number;
    limit: number;
    total: number;
    totalPages: number;
    hasNextPage: boolean;
    hasPreviousPage: boolean;
  };
}
