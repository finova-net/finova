// client/typescript/src/types/index.ts

/**
 * Finova Network TypeScript SDK Types
 * 
 * Complete type definitions for the Finova Network ecosystem including:
 * - XP (Experience Points) System
 * - RP (Referral Points) System  
 * - Mining Mechanism with Exponential Regression
 * - NFT & Special Cards
 * - Staking & Enhanced Rewards
 * - Guild System & Governance
 * - Anti-Bot & Quality Assessment
 * 
 * @version 4.0
 * @author Finova Network Team
 * @license MIT
 */

import { PublicKey, AccountMeta } from '@solana/web3.js';
import BN from 'bn.js';

// ========================================
// CORE BLOCKCHAIN TYPES
// ========================================

export interface ProgramAddresses {
  finovaCore: PublicKey;
  finovaToken: PublicKey;
  finovaNft: PublicKey;
  finovaDefi: PublicKey;
  finovaOracle: PublicKey;
  finovaBridge: PublicKey;
}

export interface TokenMints {
  fin: PublicKey;
  sFin: PublicKey;
  usdFin: PublicKey;
  sUsdFin: PublicKey;
}

export interface AccountInfo {
  executable: boolean;
  owner: PublicKey;
  lamports: number;
  data: Buffer;
  rentEpoch?: number;
}

// ========================================
// NETWORK STATE & GLOBAL CONFIG
// ========================================

export interface NetworkState {
  authority: PublicKey;
  totalUsers: BN;
  totalMiningRewards: BN;
  currentPhase: MiningPhase;
  baseRate: BN; // Base mining rate in micro-FIN
  pioneerBonus: BN; // Current pioneer bonus multiplier (scaled by 1000)
  totalStaked: BN;
  lastUpdated: BN; // Unix timestamp
  emergencyPaused: boolean;
  bump: number;
}

export enum MiningPhase {
  Pioneer = 0, // 0-100K users
  Growth = 1,  // 100K-1M users
  Maturity = 2, // 1M-10M users
  Stability = 3 // 10M+ users
}

export interface PhaseConfig {
  userThreshold: number;
  baseRate: number; // FIN per hour
  pioneerBonus: number; // Multiplier
  maxDaily: number; // Maximum FIN per day
}

// ========================================
// USER MANAGEMENT
// ========================================

export interface UserState {
  owner: PublicKey;
  userId: BN;
  isKycVerified: boolean;
  registrationTime: BN;
  lastMiningClaim: BN;
  totalMined: BN;
  totalStaked: BN;
  currentLevel: number;
  guild?: PublicKey;
  referrer?: PublicKey;
  status: UserStatus;
  bump: number;
}

export enum UserStatus {
  Active = 0,
  Suspended = 1,
  Banned = 2,
  PendingKyc = 3
}

export interface UserProfile {
  displayName?: string;
  avatar?: string;
  bio?: string;
  socialLinks: SocialLink[];
  preferences: UserPreferences;
  stats: UserStats;
}

export interface SocialLink {
  platform: SocialPlatform;
  username: string;
  verified: boolean;
  connectedAt: Date;
}

export enum SocialPlatform {
  Instagram = 'instagram',
  TikTok = 'tiktok',
  YouTube = 'youtube',
  Facebook = 'facebook',
  TwitterX = 'twitter_x',
  LinkedIn = 'linkedin'
}

export interface UserPreferences {
  language: string;
  timezone: string;
  notifications: NotificationSettings;
  privacy: PrivacySettings;
}

export interface NotificationSettings {
  miningRewards: boolean;
  xpGains: boolean;
  referralActivity: boolean;
  guildUpdates: boolean;
  governance: boolean;
  marketing: boolean;
}

export interface PrivacySettings {
  showProfile: boolean;
  showStats: boolean;
  showReferrals: boolean;
  allowDirectMessages: boolean;
}

export interface UserStats {
  totalXp: number;
  totalRp: number;
  totalFin: number;
  daysActive: number;
  socialPosts: number;
  referralsCount: number;
  nftsOwned: number;
  guildsJoined: number;
}

// ========================================
// XP SYSTEM (EXPERIENCE POINTS)
// ========================================

export interface XpState {
  owner: PublicKey;
  totalXp: BN;
  currentLevel: number;
  levelProgress: number; // 0-100 percentage to next level
  streak: number; // Consecutive days active
  lastActivity: BN;
  platformMultipliers: PlatformMultiplier[];
  dailyXpEarned: BN;
  lastDailyReset: BN;
  bump: number;
}

export interface PlatformMultiplier {
  platform: SocialPlatform;
  multiplier: number; // Scaled by 1000 (1200 = 1.2x)
  lastUpdated: BN;
}

export interface XpActivity {
  type: ActivityType;
  platform: SocialPlatform;
  baseXp: number;
  qualityScore: number; // 0.5-2.0
  contentId?: string;
  timestamp: Date;
  processed: boolean;
}

export enum ActivityType {
  // Content Creation
  TextPost = 'text_post',
  ImagePost = 'image_post',
  VideoPost = 'video_post',
  Story = 'story',
  
  // Engagement
  Comment = 'comment',
  Like = 'like',
  Share = 'share',
  Follow = 'follow',
  
  // Platform Specific
  Login = 'login',
  Quest = 'quest',
  Milestone = 'milestone',
  Viral = 'viral' // 1K+ views/engagement
}

export interface LevelConfig {
  level: number;
  xpRequired: number;
  miningMultiplier: number; // 1.0-5.0x
  dailyFinCap: number;
  badge: BadgeTier;
  unlocks: string[];
}

export enum BadgeTier {
  BronzeI = 'bronze_1',
  BronzeX = 'bronze_10',
  SilverI = 'silver_1',
  SilverXV = 'silver_15',
  GoldI = 'gold_1',
  GoldXXV = 'gold_25',
  PlatinumI = 'platinum_1',
  PlatinumXXV = 'platinum_25',
  DiamondI = 'diamond_1',
  DiamondXXV = 'diamond_25',
  MythicI = 'mythic_1'
}

export interface QualityScore {
  originality: number; // 0-1
  engagement: number; // 0-1
  brandSafety: number; // 0-1
  authenticity: number; // 0-1
  overall: number; // Combined score 0.5-2.0
  aiConfidence: number; // AI model confidence
}

// ========================================
// REFERRAL SYSTEM (RP - REFERRAL POINTS)
// ========================================

export interface ReferralState {
  owner: PublicKey;
  totalRp: BN;
  directReferrals: number;
  networkSize: number; // Total network including L2, L3
  currentTier: ReferralTier;
  networkQuality: number; // Scaled by 1000
  lastCalculated: BN;
  referralCode: string;
  referrer?: PublicKey;
  bump: number;
}

export enum ReferralTier {
  Explorer = 0,    // 0-999 RP
  Connector = 1,   // 1K-4.9K RP
  Influencer = 2,  // 5K-14.9K RP
  Leader = 3,      // 15K-49.9K RP
  Ambassador = 4   // 50K+ RP
}

export interface ReferralNetwork {
  level1: ReferralNode[]; // Direct referrals
  level2: ReferralNode[]; // Referrals of referrals
  level3: ReferralNode[]; // Third level
  totalNodes: number;
  averageActivity: number;
  retentionRate: number;
}

export interface ReferralNode {
  publicKey: PublicKey;
  joinedAt: BN;
  totalMined: BN;
  isActive: boolean; // Active in last 30 days
  currentLevel: number;
  contributionScore: number;
}

export interface ReferralReward {
  fromUser: PublicKey;
  rewardType: RewardType;
  amount: BN;
  timestamp: BN;
  level: number; // 1, 2, or 3
}

export enum RewardType {
  Registration = 'registration',
  KycCompletion = 'kyc_completion',
  DailyMining = 'daily_mining',
  XpActivity = 'xp_activity',
  Achievement = 'achievement',
  NetworkBonus = 'network_bonus'
}

export interface ReferralTierConfig {
  tier: ReferralTier;
  rpRange: [number, number];
  miningBonus: number; // Percentage bonus
  referralBonus: number; // Percentage from referrals
  networkCap: number; // Max referrals
  specialBenefits: string[];
}

// ========================================
// MINING MECHANISM
// ========================================

export interface MiningState {
  owner: PublicKey;
  hourlyRate: BN; // Current mining rate in micro-FIN
  lastMiningTime: BN;
  totalMined: BN;
  activeEffects: ActiveEffect[];
  regressionFactor: BN; // Scaled by 1e9
  dailyMined: BN;
  lastDailyReset: BN;
  consecutiveDays: number;
  bump: number;
}

export interface MiningCalculation {
  baseRate: number; // FIN per hour
  pioneerBonus: number; // Network bonus
  referralBonus: number; // From referral network
  xpMultiplier: number; // From XP level
  stakingBonus: number; // From staked tokens
  cardEffects: number; // From active NFT cards
  qualityScore: number; // Content quality
  regressionFactor: number; // Anti-whale mechanism
  finalRate: number; // Final hourly rate
  dailyProjection: number; // Expected daily earnings
}

export interface MiningReward {
  amount: BN;
  timestamp: BN;
  breakdown: RewardBreakdown;
  transactionSignature?: string;
}

export interface RewardBreakdown {
  baseMining: number;
  xpBonus: number;
  rpBonus: number;
  stakingBonus: number;
  cardBonus: number;
  qualityBonus: number;
  streakBonus: number;
  penalties: number;
  total: number;
}

export interface ActiveEffect {
  effectType: EffectType;
  multiplier: BN; // Scaled by 1000
  duration: BN; // Seconds remaining
  source: EffectSource;
  startTime: BN;
}

export enum EffectType {
  MiningBoost = 'mining_boost',
  XpBoost = 'xp_boost',
  RpBoost = 'rp_boost',
  QualityBoost = 'quality_boost',
  StreakProtection = 'streak_protection'
}

export enum EffectSource {
  SpecialCard = 'special_card',
  GuildEvent = 'guild_event',
  Achievement = 'achievement',
  Promotion = 'promotion',
  Staking = 'staking'
}

// ========================================
// STAKING SYSTEM
// ========================================

export interface StakingState {
  owner: PublicKey;
  stakedAmount: BN;
  stakingTier: StakingTier;
  stakingStartTime: BN;
  lastRewardClaim: BN;
  pendingRewards: BN;
  lockupPeriod: BN; // Seconds
  isLocked: boolean;
  autoCompound: boolean;
  bump: number;
}

export enum StakingTier {
  Basic = 0,      // 100-499 FIN
  Premium = 1,    // 500-999 FIN
  VIP = 2,        // 1K-4.9K FIN
  Elite = 3,      // 5K-9.9K FIN
  Legendary = 4   // 10K+ FIN
}

export interface StakingTierConfig {
  tier: StakingTier;
  minAmount: number;
  maxAmount?: number;
  apy: number; // Annual percentage yield
  miningBoost: number; // Mining rate bonus
  xpMultiplier: number; // XP bonus
  rpBonus: number; // RP bonus
  specialFeatures: string[];
}

export interface LiquidStakingInfo {
  sFinExchangeRate: number; // sFIN per FIN
  totalSFinSupply: BN;
  totalFinStaked: BN;
  currentApy: number;
  fees: StakingFees;
}

export interface StakingFees {
  depositFee: number; // Percentage
  withdrawFee: number; // Percentage
  performanceFee: number; // Percentage of rewards
}

// ========================================
// NFT & SPECIAL CARDS
// ========================================

export interface NftMetadata {
  name: string;
  symbol: string;
  description: string;
  image: string;
  attributes: NftAttribute[];
  collection?: PublicKey;
  royalties: number; // Percentage
  creators: Creator[];
}

export interface NftAttribute {
  trait_type: string;
  value: string | number;
  display_type?: string;
  max_value?: number;
}

export interface Creator {
  address: PublicKey;
  verified: boolean;
  share: number; // Percentage
}

export interface SpecialCard {
  mint: PublicKey;
  cardType: CardType;
  rarity: CardRarity;
  effect: CardEffect;
  uses: number; // Remaining uses (-1 for unlimited)
  isActive: boolean;
  acquiredAt: BN;
  lastUsed?: BN;
}

export enum CardType {
  MiningBoost = 'mining_boost',
  XpAccelerator = 'xp_accelerator',
  ReferralPower = 'referral_power',
  QualityEnhancer = 'quality_enhancer',
  StreakProtector = 'streak_protector',
  NetworkAmplifier = 'network_amplifier'
}

export enum CardRarity {
  Common = 'common',
  Uncommon = 'uncommon',
  Rare = 'rare',
  Epic = 'epic',
  Legendary = 'legendary',
  Mythic = 'mythic'
}

export interface CardEffect {
  effectType: EffectType;
  magnitude: number; // Effect strength (multiplier or flat bonus)
  duration: number; // Effect duration in seconds
  conditions?: CardCondition[];
}

export interface CardCondition {
  type: ConditionType;
  value: any;
  operator: ComparisonOperator;
}

export enum ConditionType {
  MinLevel = 'min_level',
  MinStake = 'min_stake',
  Platform = 'platform',
  ActivityType = 'activity_type',
  TimeOfDay = 'time_of_day',
  DayOfWeek = 'day_of_week'
}

export enum ComparisonOperator {
  GreaterThan = 'gt',
  GreaterThanOrEqual = 'gte',
  LessThan = 'lt',
  LessThanOrEqual = 'lte',
  Equal = 'eq',
  NotEqual = 'neq',
  In = 'in',
  NotIn = 'not_in'
}

export interface NftMarketplace {
  totalListings: number;
  totalVolume: BN;
  floorPrice: BN;
  averagePrice: BN;
  recentSales: MarketplaceSale[];
}

export interface MarketplaceSale {
  mint: PublicKey;
  price: BN;
  seller: PublicKey;
  buyer: PublicKey;
  timestamp: BN;
  transactionSignature: string;
}

// ========================================
// GUILD SYSTEM
// ========================================

export interface GuildState {
  id: BN;
  name: string;
  description: string;
  guildMaster: PublicKey;
  officers: PublicKey[];
  members: PublicKey[];
  maxMembers: number;
  totalXp: BN;
  totalRp: BN;
  createdAt: BN;
  isActive: boolean;
  requirements: GuildRequirements;
  bump: number;
}

export interface GuildRequirements {
  minLevel: number;
  minStake?: BN;
  minRp?: BN;
  inviteOnly: boolean;
  kycRequired: boolean;
}

export interface GuildMember {
  publicKey: PublicKey;
  role: GuildRole;
  joinedAt: BN;
  contributionScore: BN;
  isActive: boolean;
}

export enum GuildRole {
  Member = 'member',
  Officer = 'officer',
  GuildMaster = 'guild_master'
}

export interface GuildEvent {
  id: BN;
  guild: PublicKey;
  eventType: EventType;
  name: string;
  description: string;
  startTime: BN;
  endTime: BN;
  rewards: EventReward[];
  participants: PublicKey[];
  isActive: boolean;
}

export enum EventType {
  DailyChallenge = 'daily_challenge',
  WeeklyWar = 'weekly_war',
  MonthlyChampionship = 'monthly_championship',
  SeasonalLeague = 'seasonal_league',
  SpecialEvent = 'special_event'
}

export interface EventReward {
  position: number; // 1st, 2nd, 3rd, etc.
  rewardType: EventRewardType;
  amount: BN;
  nftMint?: PublicKey;
}

export enum EventRewardType {
  Fin = 'fin',
  Xp = 'xp',
  Rp = 'rp',
  Nft = 'nft',
  SpecialCard = 'special_card'
}

// ========================================
// GOVERNANCE & DAO
// ========================================

export interface ProposalState {
  id: BN;
  proposer: PublicKey;
  title: string;
  description: string;
  proposalType: ProposalType;
  votingStartTime: BN;
  votingEndTime: BN;
  executionTime?: BN;
  yesVotes: BN;
  noVotes: BN;
  abstainVotes: BN;
  totalVotingPower: BN;
  status: ProposalStatus;
  parameters?: ProposalParameters;
  bump: number;
}

export enum ProposalType {
  ParameterChange = 'parameter_change',
  FeatureAddition = 'feature_addition',
  TreasuryAllocation = 'treasury_allocation',
  CommunityInitiative = 'community_initiative',
  Emergency = 'emergency'
}

export enum ProposalStatus {
  Pending = 'pending',
  Active = 'active',
  Succeeded = 'succeeded',
  Defeated = 'defeated',
  Executed = 'executed',
  Cancelled = 'cancelled'
}

export interface ProposalParameters {
  targetParameter?: string;
  newValue?: any;
  effectiveDate?: BN;
  treasuryAmount?: BN;
  recipient?: PublicKey;
}

export interface VoteRecord {
  voter: PublicKey;
  proposal: PublicKey;
  vote: VoteChoice;
  votingPower: BN;
  timestamp: BN;
  bump: number;
}

export enum VoteChoice {
  Yes = 'yes',
  No = 'no',
  Abstain = 'abstain'
}

export interface VotingPowerCalculation {
  stakedSFin: BN;
  xpLevelMultiplier: number;
  rpReputationScore: number;
  activityWeight: number;
  totalVotingPower: BN;
}

// ========================================
// ANTI-BOT & SECURITY
// ========================================

export interface HumanVerification {
  userId: PublicKey;
  humanProbability: number; // 0-1
  verificationMethods: VerificationMethod[];
  lastVerified: BN;
  isVerified: boolean;
  riskScore: number; // 0-1
}

export enum VerificationMethod {
  Biometric = 'biometric',
  Behavioral = 'behavioral',
  SocialGraph = 'social_graph',
  DeviceFingerprint = 'device_fingerprint',
  KYC = 'kyc'
}

export interface BehaviorPattern {
  userId: PublicKey;
  clickSpeed: number[];
  sessionDuration: number[];
  activityTimes: number[];
  interactionPatterns: string[];
  abnormalityScore: number; // 0-1
}

export interface QualityAssessment {
  contentId: string;
  originalityScore: number; // 0-1
  engagementQuality: number; // 0-1
  brandSafety: number; // 0-1
  aiConfidence: number; // 0-1
  humanReview: boolean;
  finalScore: number; // 0.5-2.0
}

export interface SecurityAlert {
  userId: PublicKey;
  alertType: AlertType;
  severity: AlertSeverity;
  description: string;
  timestamp: BN;
  resolved: boolean;
  investigationNotes?: string;
}

export enum AlertType {
  SuspiciousActivity = 'suspicious_activity',
  BotBehavior = 'bot_behavior',
  FraudAttempt = 'fraud_attempt',
  NetworkAbuse = 'network_abuse',
  TokenManipulation = 'token_manipulation'
}

export enum AlertSeverity {
  Low = 'low',
  Medium = 'medium',
  High = 'high',
  Critical = 'critical'
}

// ========================================
// API & CLIENT TYPES
// ========================================

export interface ClientConfig {
  programAddresses: ProgramAddresses;
  tokenMints: TokenMints;
  rpcEndpoint: string;
  wsEndpoint?: string;
  commitment?: 'processed' | 'confirmed' | 'finalized';
  skipPreflight?: boolean;
  preflightCommitment?: 'processed' | 'confirmed' | 'finalized';
}

export interface TransactionResult {
  signature: string;
  slot: number;
  blockTime?: number;
  confirmationStatus: 'processed' | 'confirmed' | 'finalized';
  err?: any;
}

export interface InstructionData {
  programId: PublicKey;
  accounts: AccountMeta[];
  data: Buffer;
}

export interface SimulationResult {
  success: boolean;
  logs: string[];
  unitsConsumed?: number;
  error?: string;
}

// ========================================
// API REQUEST/RESPONSE TYPES
// ========================================

export interface ApiResponse<T> {
  success: boolean;
  data?: T;
  error?: ApiError;
  metadata?: ResponseMetadata;
}

export interface ApiError {
  code: string;
  message: string;
  details?: any;
}

export interface ResponseMetadata {
  timestamp: string;
  requestId: string;
  processingTime: number;
  version: string;
}

export interface PaginatedResponse<T> {
  items: T[];
  totalCount: number;
  page: number;
  pageSize: number;
  hasNext: boolean;
  hasPrevious: boolean;
}

export interface QueryParams {
  page?: number;
  pageSize?: number;
  sortBy?: string;
  sortOrder?: 'asc' | 'desc';
  filters?: Record<string, any>;
}

// ========================================
// REAL-TIME UPDATES
// ========================================

export interface WebSocketMessage {
  type: MessageType;
  data: any;
  timestamp: string;
  userId?: string;
}

export enum MessageType {
  MiningUpdate = 'mining_update',
  XpGained = 'xp_gained',
  RpUpdated = 'rp_updated',
  LevelUp = 'level_up',
  ReferralActivity = 'referral_activity',
  GuildEvent = 'guild_event',
  NftActivity = 'nft_activity',
  GovernanceUpdate = 'governance_update',
  SecurityAlert = 'security_alert',
  SystemNotification = 'system_notification'
}

export interface MiningUpdateMessage {
  newRate: number;
  totalMined: number;
  dailyProgress: number;
  activeEffects: ActiveEffect[];
}

export interface XpGainedMessage {
  amount: number;
  activity: ActivityType;
  platform: SocialPlatform;
  newTotal: number;
  levelProgress: number;
}

export interface LevelUpMessage {
  newLevel: number;
  previousLevel: number;
  badgeEarned: BadgeTier;
  newBenefits: string[];
  celebrationAnimation: string;
}

// ========================================
// ANALYTICS & REPORTING
// ========================================

export interface UserAnalytics {
  userId: PublicKey;
  period: AnalyticsPeriod;
  metrics: AnalyticsMetrics;
  trends: AnalyticsTrend[];
  insights: AnalyticsInsight[];
}

export enum AnalyticsPeriod {
  Daily = 'daily',
  Weekly = 'weekly',
  Monthly = 'monthly',
  Quarterly = 'quarterly',
  Yearly = 'yearly',
  AllTime = 'all_time'
}

export interface AnalyticsMetrics {
  totalXp: number;
  xpGrowthRate: number;
  totalRp: number;
  rpGrowthRate: number;
  totalMined: number;
  miningEfficiency: number;
  socialEngagement: number;
  networkGrowth: number;
  qualityScore: number;
  retentionRate: number;
}

export interface AnalyticsTrend {
  metric: string;
  values: number[];
  timestamps: string[];
  direction: 'up' | 'down' | 'stable';
  changePercentage: number;
}

export interface AnalyticsInsight {
  type: InsightType;
  title: string;
  description: string;
  actionable: boolean;
  recommendation?: string;
  priority: 'low' | 'medium' | 'high';
}

export enum InsightType {
  Opportunity = 'opportunity',
  Warning = 'warning',
  Achievement = 'achievement',
  Optimization = 'optimization',
  Trend = 'trend'
}

// ========================================
// DEFI INTEGRATION
// ========================================

export interface LiquidityPool {
  poolId: PublicKey;
  tokenA: PublicKey;
  tokenB: PublicKey;
  reserveA: BN;
  reserveB: BN;
  totalSupply: BN;
  feeRate: number; // Basis points
  volume24h: BN;
  apy: number;
}

export interface LiquidityPosition {
  owner: PublicKey;
  poolId: PublicKey;
  liquidity: BN;
  tokenAAmount: BN;
  tokenBAmount: BN;
  rewards: BN;
  lastUpdate: BN;
}

export interface SwapQuote {
  inputMint: PublicKey;
  outputMint: PublicKey;
  inputAmount: BN;
  outputAmount: BN;
  minimumOutputAmount: BN;
  priceImpact: number;
  fee: BN;
  route: SwapRoute[];
}

export interface SwapRoute {
  poolId: PublicKey;
  inputMint: PublicKey;
  outputMint: PublicKey;
  percentage: number; // How much of the swap goes through this route
}

// ========================================
// ORACLE & PRICE FEEDS
// ========================================

export interface PriceFeed {
  mint: PublicKey;
  price: BN; // Price in USD (scaled)
  confidence: BN;
  timestamp: BN;
  source: string;
}

export interface AggregatedPrice {
  mint: PublicKey;
  weightedPrice: BN;
  sources: PriceSource[];
  confidence: number;
  lastUpdate: BN;
}

export interface PriceSource {
  name: string;
  price: BN;
  weight: number;
  confidence: number;
  timestamp: BN;
}

// ========================================
// BRIDGE & CROSS-CHAIN
// ========================================

export interface BridgeTransaction {
  id: string;
  sourceChain: string;
  targetChain: string;
  sourceToken: PublicKey;
  targetToken: string;
  amount: BN;
  sender: PublicKey;
  recipient: string;
  status: BridgeStatus;
  createdAt: BN;
  completedAt?: BN;
}

export enum BridgeStatus {
  Initiated = 'initiated',
  Locked = 'locked',
  Validated = 'validated',
  Minted = 'minted',
  Completed = 'completed',
  Failed = 'failed'
}

export interface ChainConfig {
  chainId: number;
  name: string;
  rpcUrl: string;
  explorerUrl: string;
  bridgeContract: string;
  supportedTokens: string[];
}

// ========================================
// UTILITY TYPES
// ========================================

export type Awaitable<T> = T | Promise<T>;

export interface Result<T, E = Error> {
  success: boolean;
  data?: T;
  error?: E;
}

export interface Optional<T> {
  value?: T;
  hasValue: boolean;
}

export type Partial<T> = {
  [P in keyof T]?: T[P];
};

export type DeepPartial<T> = {
  [P in keyof T]?: T[P] extends object ? DeepPartial<T[P]> : T[P];
};

export interface Serializable {
  serialize(): Buffer;
}

export interface Deserializable<T> {
  deserialize(data: Buffer): T;
}

// ========================================
// MATHEMATICAL CALCULATIONS
// ========================================

export interface MathConstants {
  SCALE_FACTOR: BN; // 1e9 for precision
  MAX_BPS: number; // 10000 basis points = 100%
  SECONDS_PER_HOUR: number; // 3600
  SECONDS_PER_DAY: number; // 86400
  DAYS_PER_YEAR: number; // 365
  REGRESSION_COEFFICIENT: number; // 0.001 for exponential decay
}

export interface ExponentialRegression {
  coefficient: number; // -0.001
  base: number; // e (Euler's number)
  threshold: BN; // Holdings threshold for regression
  calculate(holdings: BN): number; // Returns regression factor
}

export interface CompoundInterest {
  principal: BN;
  rate: number; // Annual percentage rate
  frequency: number; // Compounding frequency per year
  time: number; // Time in years
  futureValue: BN;
}

export interface NetworkEffect {
  nodeCount: number;
  connectionDensity: number;
  averageValue: number;
  metcalfeValue: number; // n^2 scaling
  reedValue: number; // 2^n scaling for groups
}

// ========================================
// ADVANCED FORMULA TYPES
// ========================================

export interface RewardFormula {
  name: string;
  version: string;
  components: FormulaComponent[];
  constants: Record<string, number>;
  calculate(inputs: FormulaInputs): FormulaResult;
}

export interface FormulaComponent {
  name: string;
  type: ComponentType;
  weight: number;
  function: string; // Mathematical expression
  constraints?: ComponentConstraint[];
}

export enum ComponentType {
  Linear = 'linear',
  Exponential = 'exponential',
  Logarithmic = 'logarithmic',
  Sigmoid = 'sigmoid',
  Polynomial = 'polynomial',
  Custom = 'custom'
}

export interface ComponentConstraint {
  type: ConstraintType;
  value: number;
  operator: ComparisonOperator;
}

export enum ConstraintType {
  MinValue = 'min_value',
  MaxValue = 'max_value',
  Range = 'range',
  Precision = 'precision'
}

export interface FormulaInputs {
  baseRate: number;
  xpLevel: number;
  rpTier: number;
  stakeAmount: number;
  qualityScore: number;
  networkSize: number;
  timeActive: number;
  [key: string]: number;
}

export interface FormulaResult {
  finalValue: number;
  breakdown: ComponentBreakdown[];
  confidence: number;
  metadata: CalculationMetadata;
}

export interface ComponentBreakdown {
  component: string;
  input: number;
  output: number;
  contribution: number; // Percentage of final result
}

export interface CalculationMetadata {
  timestamp: number;
  version: string;
  processingTime: number;
  warnings: string[];
}

// ========================================
// AI & MACHINE LEARNING TYPES
// ========================================

export interface AIModel {
  name: string;
  version: string;
  type: ModelType;
  accuracy: number;
  confidence: number;
  lastTrained: Date;
  features: ModelFeature[];
}

export enum ModelType {
  ContentQuality = 'content_quality',
  BotDetection = 'bot_detection',
  FraudPrevention = 'fraud_prevention',
  UserSegmentation = 'user_segmentation',
  RecommendationEngine = 'recommendation_engine',
  ChurnPrediction = 'churn_prediction'
}

export interface ModelFeature {
  name: string;
  type: FeatureType;
  importance: number; // 0-1
  description: string;
}

export enum FeatureType {
  Numerical = 'numerical',
  Categorical = 'categorical',
  Boolean = 'boolean',
  Text = 'text',
  Image = 'image',
  Temporal = 'temporal'
}

export interface PredictionResult {
  prediction: any;
  confidence: number;
  probabilities?: Record<string, number>;
  features: FeatureContribution[];
  modelUsed: string;
  timestamp: Date;
}

export interface FeatureContribution {
  feature: string;
  value: any;
  contribution: number; // Impact on prediction
  normalized: number; // -1 to 1
}

export interface ContentAnalysis {
  contentId: string;
  type: ContentType;
  analysis: AnalysisResult[];
  overallScore: number;
  recommendations: string[];
  flags: ContentFlag[];
}

export enum ContentType {
  Text = 'text',
  Image = 'image',
  Video = 'video',
  Audio = 'audio',
  Mixed = 'mixed'
}

export interface AnalysisResult {
  dimension: AnalysisDimension;
  score: number; // 0-1
  confidence: number; // 0-1
  details: Record<string, any>;
}

export enum AnalysisDimension {
  Originality = 'originality',
  Quality = 'quality',
  Engagement = 'engagement',
  Sentiment = 'sentiment',
  Toxicity = 'toxicity',
  BrandSafety = 'brand_safety',
  Compliance = 'compliance'
}

export interface ContentFlag {
  type: FlagType;
  severity: FlagSeverity;
  description: string;
  confidence: number;
  autoResolved: boolean;
}

export enum FlagType {
  Spam = 'spam',
  Inappropriate = 'inappropriate',
  Copyright = 'copyright',
  Misleading = 'misleading',
  Harmful = 'harmful',
  LowQuality = 'low_quality'
}

export enum FlagSeverity {
  Info = 'info',
  Warning = 'warning',
  Error = 'error',
  Critical = 'critical'
}

// ========================================
// MOBILE SDK TYPES
// ========================================

export interface MobileConfig {
  apiBaseUrl: string;
  wsBaseUrl: string;
  authTokenKey: string;
  biometricEnabled: boolean;
  pushNotifications: boolean;
  analytics: MobileAnalyticsConfig;
  security: MobileSecurityConfig;
}

export interface MobileAnalyticsConfig {
  enabled: boolean;
  trackingId: string;
  events: EventTracking[];
  anonymizeData: boolean;
}

export interface EventTracking {
  name: string;
  properties: string[];
  frequency: TrackingFrequency;
}

export enum TrackingFrequency {
  Always = 'always',
  Session = 'session',
  Daily = 'daily',
  Weekly = 'weekly'
}

export interface MobileSecurityConfig {
  certificatePinning: boolean;
  rootDetection: boolean;
  debugDetection: boolean;
  emulatorDetection: boolean;
  tampering: TamperingProtection;
}

export interface TamperingProtection {
  enabled: boolean;
  checksumValidation: boolean;
  signatureValidation: boolean;
  environmentValidation: boolean;
}

export interface BiometricAuthResult {
  success: boolean;
  authType: BiometricType;
  error?: BiometricError;
  fallbackUsed?: boolean;
}

export enum BiometricType {
  Fingerprint = 'fingerprint',
  FaceID = 'face_id',
  TouchID = 'touch_id',
  Voice = 'voice',
  Iris = 'iris'
}

export interface BiometricError {
  code: BiometricErrorCode;
  message: string;
  recoverable: boolean;
}

export enum BiometricErrorCode {
  NotAvailable = 'not_available',
  NotEnrolled = 'not_enrolled',
  Failed = 'failed',
  Cancelled = 'cancelled',
  Timeout = 'timeout',
  Locked = 'locked'
}

// ========================================
// NOTIFICATION SYSTEM
// ========================================

export interface NotificationConfig {
  channels: NotificationChannel[];
  templates: NotificationTemplate[];
  triggers: NotificationTrigger[];
  preferences: NotificationPreference[];
}

export interface NotificationChannel {
  id: string;
  name: string;
  type: ChannelType;
  enabled: boolean;
  settings: ChannelSettings;
}

export enum ChannelType {
  InApp = 'in_app',
  Push = 'push',
  Email = 'email',
  SMS = 'sms',
  WebSocket = 'websocket',
  Webhook = 'webhook'
}

export interface ChannelSettings {
  [key: string]: any;
  retryAttempts?: number;
  retryDelay?: number;
  batchSize?: number;
  rateLimit?: number;
}

export interface NotificationTemplate {
  id: string;
  name: string;
  category: NotificationCategory;
  title: string;
  body: string;
  data?: Record<string, any>;
  actions?: NotificationAction[];
  personalization: PersonalizationRule[];
}

export enum NotificationCategory {
  Mining = 'mining',
  XP = 'xp',
  Referral = 'referral',
  Guild = 'guild',
  NFT = 'nft',
  Governance = 'governance',
  Security = 'security',
  Marketing = 'marketing',
  System = 'system'
}

export interface NotificationAction {
  id: string;
  title: string;
  type: ActionType;
  url?: string;
  data?: Record<string, any>;
}

export enum ActionType {
  Open = 'open',
  Dismiss = 'dismiss',
  Snooze = 'snooze',
  Navigate = 'navigate',
  Execute = 'execute'
}

export interface PersonalizationRule {
  condition: string;
  replacement: string;
  fallback?: string;
}

export interface NotificationTrigger {
  id: string;
  event: TriggerEvent;
  conditions: TriggerCondition[];
  template: string;
  channels: string[];
  timing: TriggerTiming;
}

export enum TriggerEvent {
  UserRegistration = 'user_registration',
  MiningReward = 'mining_reward',
  LevelUp = 'level_up',
  ReferralJoined = 'referral_joined',
  GuildInvite = 'guild_invite',
  NFTReceived = 'nft_received',
  ProposalCreated = 'proposal_created',
  SecurityAlert = 'security_alert'
}

export interface TriggerCondition {
  field: string;
  operator: ComparisonOperator;
  value: any;
}

export interface TriggerTiming {
  immediate: boolean;
  delay?: number; // Seconds
  schedule?: CronExpression;
  timezone?: string;
}

export type CronExpression = string; // Standard cron format

export interface NotificationPreference {
  userId: PublicKey;
  category: NotificationCategory;
  channels: ChannelPreference[];
  frequency: NotificationFrequency;
  quietHours?: QuietHours;
}

export interface ChannelPreference {
  channel: ChannelType;
  enabled: boolean;
  settings?: Record<string, any>;
}

export enum NotificationFrequency {
  RealTime = 'real_time',
  Batched = 'batched',
  Daily = 'daily',
  Weekly = 'weekly',
  Never = 'never'
}

export interface QuietHours {
  enabled: boolean;
  startTime: string; // HH:MM format
  endTime: string; // HH:MM format
  timezone: string;
  exceptions: string[]; // High-priority categories
}

// ========================================
// BACKUP & RECOVERY
// ========================================

export interface BackupConfig {
  strategy: BackupStrategy;
  frequency: BackupFrequency;
  retention: RetentionPolicy;
  encryption: EncryptionConfig;
  destinations: BackupDestination[];
}

export enum BackupStrategy {
  Full = 'full',
  Incremental = 'incremental',
  Differential = 'differential',
  Continuous = 'continuous'
}

export enum BackupFrequency {
  RealTime = 'real_time',
  Hourly = 'hourly',
  Daily = 'daily',
  Weekly = 'weekly',
  Monthly = 'monthly'
}

export interface RetentionPolicy {
  daily: number; // Days to keep daily backups
  weekly: number; // Weeks to keep weekly backups
  monthly: number; // Months to keep monthly backups
  yearly: number; // Years to keep yearly backups
}

export interface EncryptionConfig {
  enabled: boolean;
  algorithm: EncryptionAlgorithm;
  keyRotation: boolean;
  keyRotationInterval: number; // Days
}

export enum EncryptionAlgorithm {
  AES256 = 'aes256',
  ChaCha20 = 'chacha20',
  RSA2048 = 'rsa2048',
  RSA4096 = 'rsa4096'
}

export interface BackupDestination {
  id: string;
  type: DestinationType;
  config: DestinationConfig;
  priority: number;
  enabled: boolean;
}

export enum DestinationType {
  Local = 'local',
  S3 = 's3',
  GCS = 'gcs',
  Azure = 'azure',
  IPFS = 'ipfs',
  Arweave = 'arweave'
}

export interface DestinationConfig {
  [key: string]: any;
  endpoint?: string;
  credentials?: Record<string, string>;
  bucket?: string;
  path?: string;
}

// ========================================
// FEATURE FLAGS & A/B TESTING
// ========================================

export interface FeatureFlag {
  key: string;
  name: string;
  description: string;
  enabled: boolean;
  rolloutPercentage: number; // 0-100
  conditions: FlagCondition[];
  variants: FlagVariant[];
  metrics: FlagMetric[];
}

export interface FlagCondition {
  attribute: string;
  operator: ComparisonOperator;
  value: any;
}

export interface FlagVariant {
  key: string;
  name: string;
  weight: number; // Percentage allocation
  payload?: Record<string, any>;
}

export interface FlagMetric {
  name: string;
  type: MetricType;
  goal: MetricGoal;
  significance: number; // Statistical significance threshold
}

export enum MetricType {
  Conversion = 'conversion',
  Revenue = 'revenue',
  Engagement = 'engagement',
  Retention = 'retention',
  Custom = 'custom'
}

export enum MetricGoal {
  Increase = 'increase',
  Decrease = 'decrease',
  Maintain = 'maintain'
}

export interface ABTestResult {
  testKey: string;
  variant: string;
  metrics: ABTestMetric[];
  significance: number;
  confidence: number;
  winningVariant?: string;
  recommendation: TestRecommendation;
}

export interface ABTestMetric {
  name: string;
  control: MetricValue;
  variant: MetricValue;
  lift: number; // Percentage change
  pValue: number; // Statistical significance
}

export interface MetricValue {
  value: number;
  sampleSize: number;
  standardDeviation: number;
  confidence: [number, number]; // Confidence interval
}

export enum TestRecommendation {
  RolloutVariant = 'rollout_variant',
  KeepControl = 'keep_control',
  RunLonger = 'run_longer',
  IncreaseSample = 'increase_sample',
  Inconclusive = 'inconclusive'
}

// ========================================
// EXPORT ALL TYPES
// ========================================

export * from './mining';
export * from './xp';
export * from './referral';
export * from './staking';
export * from './nft';
export * from './guild';
export * from './governance';
export * from './security';
export * from './api';
export * from './mobile';
export * from './analytics';
export * from './defi';
export * from './bridge';
export * from './notifications';

// ========================================
// TYPE GUARDS & UTILITY FUNCTIONS
// ========================================

export function isPublicKey(value: any): value is PublicKey {
  return value && typeof value.toBase58 === 'function';
}

export function isBN(value: any): value is BN {
  return value && typeof value.toNumber === 'function';
}

export function isUserState(value: any): value is UserState {
  return value && isPublicKey(value.owner) && typeof value.userId !== 'undefined';
}

export function isXpState(value: any): value is XpState {
  return value && isPublicKey(value.owner) && typeof value.totalXp !== 'undefined';
}

export function isReferralState(value: any): value is ReferralState {
  return value && isPublicKey(value.owner) && typeof value.totalRp !== 'undefined';
}

export function isStakingState(value: any): value is StakingState {
  return value && isPublicKey(value.owner) && typeof value.stakedAmount !== 'undefined';
}

export function isGuildState(value: any): value is GuildState {
  return value && typeof value.name === 'string' && isPublicKey(value.guildMaster);
}

export function isValidSocialPlatform(platform: string): platform is SocialPlatform {
  return Object.values(SocialPlatform).includes(platform as SocialPlatform);
}

export function isValidActivityType(activity: string): activity is ActivityType {
  return Object.values(ActivityType).includes(activity as ActivityType);
}

export function isValidCardType(cardType: string): cardType is CardType {
  return Object.values(CardType).includes(cardType as CardType);
}

export function isValidMiningPhase(phase: number): phase is MiningPhase {
  return phase >= 0 && phase <= 3 && Number.isInteger(phase);
}

export function isValidReferralTier(tier: number): tier is ReferralTier {
  return tier >= 0 && tier <= 4 && Number.isInteger(tier);
}

export function isValidStakingTier(tier: number): tier is StakingTier {
  return tier >= 0 && tier <= 4 && Number.isInteger(tier);
}

// ========================================
// CONSTANTS & DEFAULTS
// ========================================

export const DEFAULT_CONFIG: Partial<ClientConfig> = {
  commitment: 'confirmed',
  skipPreflight: false,
  preflightCommitment: 'processed'
};

export const MATH_CONSTANTS: MathConstants = {
  SCALE_FACTOR: new BN(1_000_000_000), // 1e9
  MAX_BPS: 10_000,
  SECONDS_PER_HOUR: 3_600,
  SECONDS_PER_DAY: 86_400,
  DAYS_PER_YEAR: 365,
  REGRESSION_COEFFICIENT: 0.001
};

export const PHASE_CONFIGS: Record<MiningPhase, PhaseConfig> = {
  [MiningPhase.Pioneer]: {
    userThreshold: 100_000,
    baseRate: 0.1,
    pioneerBonus: 2.0,
    maxDaily: 4.8
  },
  [MiningPhase.Growth]: {
    userThreshold: 1_000_000,
    baseRate: 0.05,
    pioneerBonus: 1.5,
    maxDaily: 1.8
  },
  [MiningPhase.Maturity]: {
    userThreshold: 10_000_000,
    baseRate: 0.025,
    pioneerBonus: 1.2,
    maxDaily: 0.72
  },
  [MiningPhase.Stability]: {
    userThreshold: Number.MAX_SAFE_INTEGER,
    baseRate: 0.01,
    pioneerBonus: 1.0,
    maxDaily: 0.24
  }
};

export const LEVEL_CONFIGS: LevelConfig[] = [
  // Bronze levels 1-10
  ...Array.from({ length: 10 }, (_, i) => ({
    level: i + 1,
    xpRequired: (i + 1) * 100,
    miningMultiplier: 1.0 + (i * 0.02),
    dailyFinCap: 0.5 + (i * 0.15),
    badge: `bronze_${i + 1}` as BadgeTier,
    unlocks: ['basic_features']
  })),
  // Silver levels 11-25
  ...Array.from({ length: 15 }, (_, i) => ({
    level: i + 11,
    xpRequired: 1000 + (i * 250),
    miningMultiplier: 1.3 + (i * 0.033),
    dailyFinCap: 2.0 + (i * 0.133),
    badge: `silver_${i + 1}` as BadgeTier,
    unlocks: ['special_cards', 'enhanced_referrals']
  })),
  // Gold levels 26-50
  ...Array.from({ length: 25 }, (_, i) => ({
    level: i + 26,
    xpRequired: 5000 + (i * 600),
    miningMultiplier: 1.9 + (i * 0.024),
    dailyFinCap: 4.0 + (i * 0.08),
    badge: `gold_${i + 1}` as BadgeTier,
    unlocks: ['guild_leadership', 'premium_features']
  }))
];

export const REFERRAL_TIER_CONFIGS: Record<ReferralTier, ReferralTierConfig> = {
  [ReferralTier.Explorer]: {
    tier: ReferralTier.Explorer,
    rpRange: [0, 999],
    miningBonus: 0,
    referralBonus: 10,
    networkCap: 10,
    specialBenefits: ['basic_referral_link']
  },
  [ReferralTier.Connector]: {
    tier: ReferralTier.Connector,
    rpRange: [1000, 4999],
    miningBonus: 20,
    referralBonus: 15,
    networkCap: 25,
    specialBenefits: ['custom_referral_code', 'basic_analytics']
  },
  [ReferralTier.Influencer]: {
    tier: ReferralTier.Influencer,
    rpRange: [5000, 14999],
    miningBonus: 50,
    referralBonus: 20,
    networkCap: 50,
    specialBenefits: ['advanced_analytics', 'priority_support']
  },
  [ReferralTier.Leader]: {
    tier: ReferralTier.Leader,
    rpRange: [15000, 49999],
    miningBonus: 100,
    referralBonus: 25,
    networkCap: 100,
    specialBenefits: ['exclusive_events', 'beta_features']
  },
  [ReferralTier.Ambassador]: {
    tier: ReferralTier.Ambassador,
    rpRange: [50000, Number.MAX_SAFE_INTEGER],
    miningBonus: 200,
    referralBonus: 30,
    networkCap: Number.MAX_SAFE_INTEGER,
    specialBenefits: ['dao_governance', 'revenue_sharing']
  }
};

export const STAKING_TIER_CONFIGS: Record<StakingTier, StakingTierConfig> = {
  [StakingTier.Basic]: {
    tier: StakingTier.Basic,
    minAmount: 100,
    maxAmount: 499,
    apy: 8,
    miningBoost: 20,
    xpMultiplier: 10,
    rpBonus: 5,
    specialFeatures: ['basic_staking_rewards']
  },
  [StakingTier.Premium]: {
    tier: StakingTier.Premium,
    minAmount: 500,
    maxAmount: 999,
    apy: 10,
    miningBoost: 35,
    xpMultiplier: 20,
    rpBonus: 10,
    specialFeatures: ['premium_badge', 'priority_support']
  },
  [StakingTier.VIP]: {
    tier: StakingTier.VIP,
    minAmount: 1000,
    maxAmount: 4999,
    apy: 12,
    miningBoost: 50,
    xpMultiplier: 30,
    rpBonus: 20,
    specialFeatures: ['vip_features', 'exclusive_events']
  },
  [StakingTier.Elite]: {
    tier: StakingTier.Elite,
    minAmount: 5000,
    maxAmount: 9999,
    apy: 14,
    miningBoost: 75,
    xpMultiplier: 50,
    rpBonus: 35,
    specialFeatures: ['guild_master_privileges', 'beta_access']
  },
  [StakingTier.Legendary]: {
    tier: StakingTier.Legendary,
    minAmount: 10000,
    apy: 15,
    miningBoost: 100,
    xpMultiplier: 75,
    rpBonus: 50,
    specialFeatures: ['dao_governance', 'maximum_benefits']
  }
};

// ========================================
// VERSION INFO
// ========================================

export const SDK_VERSION = '4.0.0';
export const PROTOCOL_VERSION = '4.0.0';
export const API_VERSION = 'v4';
export const MINIMUM_SOLANA_VERSION = '1.16.0';
export const MINIMUM_ANCHOR_VERSION = '0.28.0';
