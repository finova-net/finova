/**
 * Finova Network - Complete API Types Definition
 * Enterprise-grade TypeScript types for the entire Finova ecosystem
 * 
 * @version 3.0.0
 * @author Finova Development Team
 * @license MIT
 */

import { PublicKey } from '@solana/web3.js';

// ========================================
// BASE TYPES & UTILITIES
// ========================================

export type Timestamp = number;
export type SolanaAddress = string;
export type UUID = string;
export type BigNumberString = string;

export interface PaginationParams {
  page?: number;
  limit?: number;
  sortBy?: string;
  sortOrder?: 'asc' | 'desc';
}

export interface PaginatedResponse<T> {
  data: T[];
  pagination: {
    currentPage: number;
    totalPages: number;
    totalItems: number;
    itemsPerPage: number;
    hasNext: boolean;
    hasPrev: boolean;
  };
}

export interface ApiResponse<T = any> {
  success: boolean;
  data?: T;
  message?: string;
  error?: string;
  timestamp: Timestamp;
  requestId: UUID;
}

// ========================================
// AUTHENTICATION & USER MANAGEMENT
// ========================================

export interface AuthTokens {
  accessToken: string;
  refreshToken: string;
  expiresIn: number;
  tokenType: 'Bearer';
}

export interface LoginRequest {
  email?: string;
  phone?: string;
  password: string;
  deviceId: string;
  biometricHash?: string;
  captchaToken?: string;
}

export interface RegisterRequest {
  email: string;
  phone: string;
  password: string;
  confirmPassword: string;
  referralCode?: string;
  deviceId: string;
  acceptTerms: boolean;
  biometricData: BiometricData;
}

export interface BiometricData {
  faceHash: string;
  deviceFingerprint: string;
  liveness_score: number;
  quality_score: number;
}

export interface KYCRequest {
  personalInfo: {
    fullName: string;
    dateOfBirth: string;
    nationality: string;
    address: string;
    city: string;
    postalCode: string;
    idNumber: string;
  };
  documents: {
    idCardFront: string; // base64
    idCardBack: string;
    selfieImage: string;
    addressProof?: string;
  };
  biometric: BiometricData;
}

export interface User {
  id: UUID;
  email: string;
  phone: string;
  username: string;
  displayName: string;
  avatar?: string;
  walletAddress: SolanaAddress;
  kycStatus: 'pending' | 'verified' | 'rejected' | 'expired';
  accountLevel: number;
  totalXP: number;
  totalRP: number;
  miningRate: number;
  totalFINEarned: BigNumberString;
  referralCode: string;
  referredBy?: UUID;
  isActive: boolean;
  lastActiveAt: Timestamp;
  createdAt: Timestamp;
  updatedAt: Timestamp;
}

// ========================================
// MINING SYSTEM TYPES
// ========================================

export interface MiningSession {
  id: UUID;
  userId: UUID;
  startTime: Timestamp;
  endTime?: Timestamp;
  baseRate: number;
  multipliers: MiningMultipliers;
  totalEarned: BigNumberString;
  status: 'active' | 'paused' | 'completed';
  transactionHash?: string;
}

export interface MiningMultipliers {
  pioneerBonus: number;
  referralBonus: number;
  securityBonus: number;
  xpLevelBonus: number;
  rpTierBonus: number;
  stakingBonus: number;
  cardBonus: number;
  qualityBonus: number;
  regressionFactor: number;
}

export interface MiningStats {
  currentRate: number;
  totalMined: BigNumberString;
  dailyMined: BigNumberString;
  weeklyMined: BigNumberString;
  monthlyMined: BigNumberString;
  activeHours: number;
  efficiency: number;
  rank: number;
  nextMilestone: {
    target: BigNumberString;
    progress: number;
    reward: string;
  };
}

export interface MiningPhase {
  phase: 1 | 2 | 3 | 4;
  name: 'Finizen' | 'Growth' | 'Maturity' | 'Stability';
  userRange: [number, number];
  baseRate: number;
  pioneerMultiplier: number;
  maxDailyEarn: number;
  isActive: boolean;
}

// ========================================
// EXPERIENCE POINTS (XP) SYSTEM
// ========================================

export interface XPActivity {
  id: UUID;
  userId: UUID;
  activityType: XPActivityType;
  platform: SocialPlatform;
  baseXP: number;
  multipliers: XPMultipliers;
  finalXP: number;
  contentUrl?: string;
  contentHash?: string;
  qualityScore: number;
  timestamp: Timestamp;
  transactionHash?: string;
}

export type XPActivityType = 
  | 'original_post' | 'photo_post' | 'video_post' | 'story_post'
  | 'meaningful_comment' | 'like_react' | 'share_repost' | 'follow_subscribe'
  | 'daily_login' | 'complete_quest' | 'achieve_milestone' | 'viral_content';

export type SocialPlatform = 
  | 'instagram' | 'tiktok' | 'youtube' | 'facebook' | 'twitter' | 'app';

export interface XPMultipliers {
  platformMultiplier: number;
  qualityScore: number;
  streakBonus: number;
  levelProgression: number;
  cardBonus: number;
}

export interface XPLevel {
  level: number;
  tier: 'Bronze' | 'Silver' | 'Gold' | 'Platinum' | 'Diamond' | 'Mythic';
  badgeNumber: string; // e.g., "Bronze I", "Silver XV"
  xpRequired: number;
  miningMultiplier: number;
  dailyFINCap: number;
  specialUnlocks: string[];
}

export interface UserXPStats {
  currentXP: number;
  currentLevel: number;
  currentTier: string;
  xpToNextLevel: number;
  dailyXP: number;
  weeklyXP: number;
  monthlyXP: number;
  allTimeXP: number;
  streakDays: number;
  bestStreak: number;
  activitiesCount: Record<XPActivityType, number>;
}

// ========================================
// REFERRAL POINTS (RP) SYSTEM
// ========================================

export interface ReferralNetwork {
  id: UUID;
  referrerId: UUID;
  directReferrals: ReferralUser[];
  l2Network: ReferralUser[];
  l3Network: ReferralUser[];
  totalRP: number;
  tier: RPTier;
  networkQuality: number;
  activeReferrals: number;
  lifetimeEarnings: BigNumberString;
}

export interface ReferralUser {
  userId: UUID;
  username: string;
  level: number;
  joinDate: Timestamp;
  lastActive: Timestamp;
  totalContribution: number;
  isActive: boolean;
  kycVerified: boolean;
}

export type RPTier = 'Explorer' | 'Connector' | 'Influencer' | 'Leader' | 'Ambassador';

export interface RPTierInfo {
  tier: RPTier;
  rpRange: [number, number];
  miningBonus: number;
  referralBonus: {
    l1: number;
    l2: number;
    l3: number;
  };
  networkCap: number;
  specialBenefits: string[];
}

export interface ReferralActivity {
  id: UUID;
  referrerId: UUID;
  referralId: UUID;
  activityType: 'signup' | 'kyc_complete' | 'first_mine' | 'daily_mining' | 'xp_gain' | 'achievement';
  rpEarned: number;
  finEarned: BigNumberString;
  timestamp: Timestamp;
  level: 1 | 2 | 3; // L1, L2, L3
}

// ========================================
// TOKEN ECONOMICS
// ========================================

export interface TokenBalance {
  fin: BigNumberString;
  sFin: BigNumberString;
  usdFin: BigNumberString;
  sUsdFin: BigNumberString;
  totalValueUSD: number;
  lastUpdated: Timestamp;
}

export interface StakingPosition {
  id: UUID;
  userId: UUID;
  stakedAmount: BigNumberString;
  stakingTier: StakingTier;
  startDate: Timestamp;
  lockPeriod: number; // days
  currentAPY: number;
  totalRewards: BigNumberString;
  claimableRewards: BigNumberString;
  multipliers: StakingMultipliers;
  autoCompound: boolean;
  status: 'active' | 'unstaking' | 'completed';
}

export type StakingTier = 'Basic' | 'Premium' | 'VIP' | 'Guild Master' | 'Elite';

export interface StakingMultipliers {
  xpLevelBonus: number;
  rpTierBonus: number;
  loyaltyBonus: number;
  activityBonus: number;
}

export interface Transaction {
  id: UUID;
  userId: UUID;
  type: TransactionType;
  amount: BigNumberString;
  token: 'FIN' | 'sFIN' | 'USDfin' | 'sUSDfin';
  status: 'pending' | 'confirmed' | 'failed';
  blockchainTxHash: string;
  fromAddress: SolanaAddress;
  toAddress: SolanaAddress;
  fee: BigNumberString;
  timestamp: Timestamp;
  metadata?: Record<string, any>;
}

export type TransactionType = 
  | 'mining_reward' | 'xp_bonus' | 'rp_bonus' | 'referral_reward'
  | 'stake' | 'unstake' | 'claim_rewards' | 'nft_purchase' | 'nft_sale'
  | 'card_purchase' | 'card_use' | 'transfer_in' | 'transfer_out';

// ========================================
// NFT & SPECIAL CARDS
// ========================================

export interface NFTCollection {
  id: UUID;
  name: string;
  symbol: string;
  description: string;
  imageUrl: string;
  mintAddress: SolanaAddress;
  totalSupply: number;
  floorPrice: BigNumberString;
  volume24h: BigNumberString;
  category: NFTCategory;
  attributes: NFTAttribute[];
}

export type NFTCategory = 
  | 'mining_boost' | 'xp_accelerator' | 'referral_power' 
  | 'profile_badge' | 'achievement' | 'special_edition';

export interface NFT {
  id: UUID;
  tokenId: string;
  mintAddress: SolanaAddress;
  collectionId: UUID;
  ownerId: UUID;
  name: string;
  description: string;
  imageUrl: string;
  animationUrl?: string;
  rarity: NFTRarity;
  attributes: NFTAttribute[];
  utility: NFTUtility;
  marketPrice?: BigNumberString;
  isListed: boolean;
  createdAt: Timestamp;
}

export type NFTRarity = 'Common' | 'Uncommon' | 'Rare' | 'Epic' | 'Legendary';

export interface NFTAttribute {
  trait_type: string;
  value: string | number;
  display_type?: 'boost_number' | 'boost_percentage' | 'number' | 'date';
}

export interface NFTUtility {
  type: NFTCategory;
  effect: string;
  value: number;
  duration: number; // hours, -1 for permanent
  stackable: boolean;
  usageCount: number;
  maxUsage: number;
}

export interface SpecialCard extends NFT {
  cardType: 'mining_boost' | 'xp_accelerator' | 'referral_power';
  isActive: boolean;
  activatedAt?: Timestamp;
  expiresAt?: Timestamp;
  synergyBonus: number;
}

// ========================================
// MARKETPLACE
// ========================================

export interface MarketplaceListing {
  id: UUID;
  nftId: UUID;
  sellerId: UUID;
  price: BigNumberString;
  currency: 'FIN' | 'USDfin';
  status: 'active' | 'sold' | 'cancelled' | 'expired';
  createdAt: Timestamp;
  expiresAt: Timestamp;
  views: number;
  favorites: number;
}

export interface MarketplaceTrade {
  id: UUID;
  listingId: UUID;
  buyerId: UUID;
  sellerId: UUID;
  nftId: UUID;
  price: BigNumberString;
  currency: 'FIN' | 'USDfin';
  marketplaceFee: BigNumberString;
  royaltyFee: BigNumberString;
  netAmount: BigNumberString;
  transactionHash: string;
  timestamp: Timestamp;
}

// ========================================
// GUILD SYSTEM
// ========================================

export interface Guild {
  id: UUID;
  name: string;
  description: string;
  logoUrl?: string;
  masterUserId: UUID;
  officerUserIds: UUID[];
  memberCount: number;
  maxMembers: number;
  totalRP: number;
  level: number;
  rank: number;
  isPublic: boolean;
  requirements: GuildRequirements;
  benefits: GuildBenefits;
  createdAt: Timestamp;
  isActive: boolean;
}

export interface GuildRequirements {
  minLevel: number;
  minXP: number;
  minRP: number;
  requiredNFTs?: UUID[];
  inviteOnly: boolean;
}

export interface GuildBenefits {
  xpBonus: number;
  miningBonus: number;
  rpBonus: number;
  exclusiveEvents: boolean;
  customBadge: boolean;
}

export interface GuildMember {
  userId: UUID;
  guildId: UUID;
  role: 'master' | 'officer' | 'member';
  joinDate: Timestamp;
  contributionScore: number;
  isActive: boolean;
  permissions: GuildPermission[];
}

export type GuildPermission = 
  | 'invite_members' | 'remove_members' | 'edit_guild' 
  | 'manage_events' | 'view_analytics' | 'moderate_chat';

export interface GuildCompetition {
  id: UUID;
  name: string;
  description: string;
  type: 'daily_challenge' | 'weekly_war' | 'monthly_championship' | 'seasonal_league';
  startDate: Timestamp;
  endDate: Timestamp;
  participatingGuilds: UUID[];
  rewards: CompetitionReward[];
  leaderboard: GuildCompetitionEntry[];
  status: 'upcoming' | 'active' | 'completed' | 'cancelled';
}

export interface CompetitionReward {
  rank: number;
  finReward: BigNumberString;
  xpReward: number;
  rpReward: number;
  nftReward?: UUID;
  specialBadge?: string;
}

export interface GuildCompetitionEntry {
  guildId: UUID;
  guildName: string;
  score: number;
  rank: number;
  membersParticipated: number;
  lastUpdate: Timestamp;
}

// ========================================
// SOCIAL INTEGRATION
// ========================================

export interface SocialAccount {
  id: UUID;
  userId: UUID;
  platform: SocialPlatform;
  platformUserId: string;
  username: string;
  profileUrl: string;
  isVerified: boolean;
  accessToken: string; // encrypted
  refreshToken?: string; // encrypted
  tokenExpiry: Timestamp;
  permissions: string[];
  isActive: boolean;
  lastSync: Timestamp;
  connectedAt: Timestamp;
}

export interface SocialPost {
  id: UUID;
  userId: UUID;
  socialAccountId: UUID;
  platform: SocialPlatform;
  platformPostId: string;
  postUrl: string;
  content: string;
  mediaUrls: string[];
  hashtags: string[];
  mentions: string[];
  likes: number;
  comments: number;
  shares: number;
  views: number;
  engagementRate: number;
  qualityScore: number;
  xpEarned: number;
  finEarned: BigNumberString;
  postedAt: Timestamp;
  lastUpdated: Timestamp;
}

export interface ContentAnalysis {
  id: UUID;
  postId: UUID;
  originalityScore: number;
  engagementPotential: number;
  platformRelevance: number;
  brandSafety: number;
  humanGenerated: number;
  finalQualityScore: number;
  aiModel: string;
  analysisTimestamp: Timestamp;
  flags: ContentFlag[];
}

export interface ContentFlag {
  type: 'spam' | 'duplicate' | 'low_quality' | 'inappropriate' | 'ai_generated';
  confidence: number;
  description: string;
}

// ========================================
// ANALYTICS & REPORTING
// ========================================

export interface UserAnalytics {
  userId: UUID;
  period: 'daily' | 'weekly' | 'monthly' | 'yearly';
  metrics: {
    totalFINEarned: BigNumberString;
    totalXPGained: number;
    totalRPEarned: number;
    activeDays: number;
    contentCreated: number;
    engagementReceived: number;
    referralsAcquired: number;
    nftsTraded: number;
  };
  rankings: {
    globalRank: number;
    tierRank: number;
    guildRank?: number;
    countryRank: number;
  };
  achievements: Achievement[];
  timestamp: Timestamp;
}

export interface Achievement {
  id: UUID;
  name: string;
  description: string;
  category: 'mining' | 'social' | 'referral' | 'nft' | 'guild' | 'special';
  iconUrl: string;
  rarity: NFTRarity;
  finReward: BigNumberString;
  xpReward: number;
  rpReward: number;
  nftReward?: UUID;
  unlockedAt: Timestamp;
  progress?: {
    current: number;
    target: number;
    percentage: number;
  };
}

export interface NetworkAnalytics {
  totalUsers: number;
  activeUsers24h: number;
  activeUsers7d: number;
  activeUsers30d: number;
  totalFINCirculation: BigNumberString;
  totalFINStaked: BigNumberString;
  totalXPGenerated: number;
  totalRPGenerated: number;
  nftTradingVolume: BigNumberString;
  currentMiningPhase: MiningPhase;
  averageMiningRate: number;
  topGuilds: Guild[];
  topMiners: UserAnalytics[];
  timestamp: Timestamp;
}

// ========================================
// ANTI-BOT & SECURITY
// ========================================

export interface BotDetectionResult {
  userId: UUID;
  humanProbability: number;
  suspiciousActivities: SuspiciousActivity[];
  riskScore: number;
  recommendation: 'allow' | 'flag' | 'block' | 'manual_review';
  factors: {
    biometricConsistency: number;
    behavioralPatterns: number;
    socialGraphValidity: number;
    deviceAuthenticity: number;
    interactionQuality: number;
  };
  timestamp: Timestamp;
}

export interface SuspiciousActivity {
  type: 'unusual_timing' | 'automated_behavior' | 'network_pattern' | 'device_anomaly';
  confidence: number;
  description: string;
  evidence: Record<string, any>;
  severity: 'low' | 'medium' | 'high' | 'critical';
}

export interface SecurityEvent {
  id: UUID;
  userId: UUID;
  eventType: 'login' | 'failed_login' | 'password_change' | 'kyc_attempt' | 'suspicious_activity';
  severity: 'info' | 'warning' | 'error' | 'critical';
  ipAddress: string;
  userAgent: string;
  location?: {
    country: string;
    region: string;
    city: string;
  };
  metadata: Record<string, any>;
  timestamp: Timestamp;
}

// ========================================
// GOVERNANCE & DAO
// ========================================

export interface GovernanceProposal {
  id: UUID;
  proposerId: UUID;
  title: string;
  description: string;
  category: 'parameter_change' | 'feature_addition' | 'treasury_allocation' | 'community_initiative';
  proposalData: Record<string, any>;
  votingStartTime: Timestamp;
  votingEndTime: Timestamp;
  requiredQuorum: number;
  totalVotingPower: BigNumberString;
  votes: {
    for: BigNumberString;
    against: BigNumberString;
    abstain: BigNumberString;
  };
  status: 'draft' | 'active' | 'passed' | 'rejected' | 'executed' | 'cancelled';
  executionTime?: Timestamp;
  createdAt: Timestamp;
}

export interface Vote {
  id: UUID;
  proposalId: UUID;
  voterId: UUID;
  votingPower: BigNumberString;
  choice: 'for' | 'against' | 'abstain';
  reason?: string;
  timestamp: Timestamp;
  transactionHash: string;
}

// ========================================
// API REQUEST & RESPONSE TYPES
// ========================================

// Authentication API
export interface LoginResponse extends ApiResponse<{
  user: User;
  tokens: AuthTokens;
}> {}

export interface RegisterResponse extends ApiResponse<{
  user: User;
  tokens: AuthTokens;
  referralCode: string;
}> {}

// Mining API
export interface StartMiningRequest {
  deviceId: string;
  biometricHash: string;
}

export interface StartMiningResponse extends ApiResponse<{
  session: MiningSession;
  currentRate: number;
  multipliers: MiningMultipliers;
}> {}

export interface MiningStatsResponse extends ApiResponse<MiningStats> {}

// XP API
export interface SubmitActivityRequest {
  activityType: XPActivityType;
  platform: SocialPlatform;
  contentUrl?: string;
  contentData?: any;
}

export interface SubmitActivityResponse extends ApiResponse<{
  activity: XPActivity;
  xpGained: number;
  newLevel?: XPLevel;
  achievements?: Achievement[];
}> {}

// Referral API
export interface ReferralStatsResponse extends ApiResponse<{
  network: ReferralNetwork;
  tier: RPTierInfo;
  earnings: BigNumberString;
  leaderboard: { rank: number; totalRP: number; };
}> {}

// NFT API
export interface MintNFTRequest {
  collectionId: UUID;
  name: string;
  description: string;
  attributes: NFTAttribute[];
  recipientAddress: SolanaAddress;
}

export interface ListNFTRequest {
  nftId: UUID;
  price: BigNumberString;
  currency: 'FIN' | 'USDfin';
  duration: number; // hours
}

// Guild API
export interface CreateGuildRequest {
  name: string;
  description: string;
  logoUrl?: string;
  isPublic: boolean;
  requirements: GuildRequirements;
}

export interface JoinGuildRequest {
  guildId: UUID;
  message?: string;
}

// Analytics API
export interface GetAnalyticsRequest {
  period: 'daily' | 'weekly' | 'monthly' | 'yearly';
  startDate?: Timestamp;
  endDate?: Timestamp;
  metrics?: string[];
}

export interface AnalyticsResponse extends ApiResponse<UserAnalytics> {}

// WebSocket Event Types
export interface WebSocketEvent {
  type: WebSocketEventType;
  data: any;
  timestamp: Timestamp;
  userId?: UUID;
}

export type WebSocketEventType = 
  | 'mining_update' | 'xp_gained' | 'level_up' | 'referral_joined'
  | 'nft_received' | 'achievement_unlocked' | 'guild_invitation'
  | 'competition_started' | 'new_message' | 'system_announcement';

// Error Types
export interface ApiError {
  code: string;
  message: string;
  details?: Record<string, any>;
  timestamp: Timestamp;
  requestId: UUID;
}

export interface ValidationError {
  field: string;
  message: string;
  code: string;
}

// Configuration Types
export interface AppConfig {
  mining: {
    baseRate: number;
    phases: MiningPhase[];
    maxDailyHours: number;
  };
  xp: {
    baseValues: Record<XPActivityType, number>;
    platformMultipliers: Record<SocialPlatform, number>;
    levelRequirements: XPLevel[];
  };
  referral: {
    tiers: RPTierInfo[];
    maxNetworkLevels: number;
    qualityThresholds: Record<string, number>;
  };
  staking: {
    tiers: { amount: BigNumberString; apy: number; }[];
    lockPeriods: number[];
    penalties: Record<string, number>;
  };
  security: {
    maxLoginAttempts: number;
    sessionTimeout: number;
    kycRequirements: string[];
  };
}

// Export all types as a namespace for easier importing
export namespace FinovaTypes {
  export type {
    User, MiningSession, XPActivity, ReferralNetwork, NFT, Guild,
    Transaction, BotDetectionResult, GovernanceProposal, ApiResponse
  };
}
