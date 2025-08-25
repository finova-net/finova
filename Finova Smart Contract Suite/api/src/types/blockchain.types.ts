/**
 * Finova Network - Blockchain Types Definition
 * Complete type definitions for blockchain interactions, smart contracts, and token operations
 */

import { PublicKey, Transaction, TransactionSignature, Connection } from '@solana/web3.js';
import { BN } from '@coral-xyz/anchor';

// ================================
// Core Blockchain Types
// ================================

export interface BlockchainConfig {
  network: 'devnet' | 'testnet' | 'mainnet-beta';
  rpcEndpoint: string;
  wsEndpoint?: string;
  programIds: ProgramIds;
  commitment: 'processed' | 'confirmed' | 'finalized';
}

export interface ProgramIds {
  finovaCore: PublicKey;
  finovaToken: PublicKey;
  finovaNft: PublicKey;
  finovaDefi: PublicKey;
  finovaBridge: PublicKey;
  finovaOracle: PublicKey;
}

// ================================
// Transaction Types
// ================================

export interface TransactionRequest {
  instruction: TransactionInstruction;
  signers?: PublicKey[];
  feePayer?: PublicKey;
  recentBlockhash?: string;
  priority?: 'low' | 'medium' | 'high';
}

export interface TransactionInstruction {
  programId: PublicKey;
  keys: AccountMeta[];
  data: Buffer;
}

export interface AccountMeta {
  pubkey: PublicKey;
  isSigner: boolean;
  isWritable: boolean;
}

export interface TransactionResponse {
  signature: TransactionSignature;
  slot: number;
  confirmationStatus: 'processed' | 'confirmed' | 'finalized';
  err: any;
  logs?: string[];
  timestamp?: number;
}

// ================================
// Token Economics Types
// ================================

export interface TokenInfo {
  mint: PublicKey;
  symbol: string;
  name: string;
  decimals: number;
  supply: BN;
  maxSupply?: BN;
  mintAuthority?: PublicKey;
  freezeAuthority?: PublicKey;
}

export interface FinTokens {
  FIN: TokenInfo;      // Primary utility token
  sFIN: TokenInfo;     // Staked FIN
  USDfin: TokenInfo;   // Synthetic stablecoin
  sUSDfin: TokenInfo;  // Staked USDfin
}

export interface TokenBalance {
  mint: PublicKey;
  owner: PublicKey;
  amount: BN;
  decimals: number;
  uiAmount: number;
  uiAmountString: string;
}

// ================================
// Mining System Types
// ================================

export interface MiningState {
  user: PublicKey;
  totalMined: BN;
  currentRate: number;
  lastClaimTime: BN;
  phase: MiningPhase;
  bonusMultipliers: BonusMultipliers;
  regressionFactor: number;
  dailyCap: BN;
  streak: number;
}

export interface MiningPhase {
  phase: 1 | 2 | 3 | 4;
  name: 'Finizen' | 'Growth' | 'Maturity' | 'Stability';
  baseRate: number;
  finovaBonus: number;
  userThreshold: number;
}

export interface BonusMultipliers {
  referral: number;
  security: number;
  xp: number;
  staking: number;
  activity: number;
}

export interface MiningReward {
  amount: BN;
  source: 'base' | 'referral' | 'bonus' | 'special';
  timestamp: BN;
  multiplier: number;
  quality: number;
}

// ================================
// XP System Types
// ================================

export interface XPState {
  user: PublicKey;
  totalXP: BN;
  currentLevel: number;
  xpToNextLevel: BN;
  tier: XPTier;
  streak: number;
  lastActivity: BN;
  multipliers: XPMultipliers;
}

export interface XPTier {
  name: 'Bronze' | 'Silver' | 'Gold' | 'Platinum' | 'Diamond' | 'Mythic';
  level: number;
  minXP: BN;
  miningMultiplier: number;
  dailyCap: BN;
  badge: PublicKey;
}

export interface XPMultipliers {
  platform: number;
  quality: number;
  streak: number;
  level: number;
  activity: number;
}

export interface XPActivity {
  type: ActivityType;
  platform: SocialPlatform;
  baseXP: number;
  quality: number;
  timestamp: BN;
  verified: boolean;
}

export enum ActivityType {
  POST = 'post',
  COMMENT = 'comment',
  LIKE = 'like',
  SHARE = 'share',
  FOLLOW = 'follow',
  STORY = 'story',
  VIDEO = 'video',
  LIVE = 'live',
  VIRAL = 'viral'
}

export enum SocialPlatform {
  INSTAGRAM = 'instagram',
  TIKTOK = 'tiktok',
  YOUTUBE = 'youtube',
  FACEBOOK = 'facebook',
  X_TWITTER = 'x_twitter',
  LINKEDIN = 'linkedin',
  DISCORD = 'discord'
}

// ================================
// Referral System Types
// ================================

export interface ReferralState {
  user: PublicKey;
  totalRP: BN;
  tier: RPTier;
  directReferrals: PublicKey[];
  networkSize: NetworkSize;
  qualityScore: number;
  bonusMultiplier: number;
  lastUpdate: BN;
}

export interface RPTier {
  name: 'Explorer' | 'Connector' | 'Influencer' | 'Leader' | 'Ambassador';
  minRP: BN;
  miningBonus: number;
  referralBonus: number;
  networkCap: number;
  specialBenefits: string[];
}

export interface NetworkSize {
  l1: number; // Direct referrals
  l2: number; // Second level
  l3: number; // Third level
  total: number;
  active: number; // Active in last 30 days
}

export interface ReferralReward {
  fromUser: PublicKey;
  toUser: PublicKey;
  amount: BN;
  level: 1 | 2 | 3;
  type: 'registration' | 'mining' | 'activity' | 'milestone';
  timestamp: BN;
}

// ================================
// Staking System Types
// ================================

export interface StakeAccount {
  user: PublicKey;
  stakedAmount: BN;
  stakingTier: StakingTier;
  stakingStartTime: BN;
  lastRewardClaim: BN;
  pendingRewards: BN;
  lockDuration: BN;
  autoCompound: boolean;
}

export interface StakingTier {
  name: string;
  minAmount: BN;
  maxAmount?: BN;
  apy: number;
  miningBoost: number;
  xpMultiplier: number;
  rpBonus: number;
  features: string[];
}

export interface StakingPool {
  poolId: PublicKey;
  totalStaked: BN;
  totalRewards: BN;
  rewardRate: number;
  lastUpdateTime: BN;
  participants: number;
  poolType: 'FIN' | 'sFIN' | 'USDfin' | 'sUSDfin';
}

// ================================
// NFT System Types
// ================================

export interface NFTMetadata {
  mint: PublicKey;
  name: string;
  symbol: string;
  description: string;
  image: string;
  attributes: NFTAttribute[];
  properties: NFTProperties;
  collection?: PublicKey;
  creators: Creator[];
}

export interface NFTAttribute {
  trait_type: string;
  value: string | number;
  display_type?: string;
}

export interface NFTProperties {
  category: 'special_card' | 'profile_badge' | 'achievement' | 'collectible';
  rarity: 'common' | 'uncommon' | 'rare' | 'epic' | 'legendary' | 'mythic';
  utility?: NFTUtility;
  expiration?: BN;
  usesRemaining?: number;
}

export interface NFTUtility {
  type: 'mining_boost' | 'xp_accelerator' | 'referral_power' | 'profile_badge';
  effect: number;
  duration: BN;
  stackable: boolean;
  category: string;
}

export interface Creator {
  address: PublicKey;
  verified: boolean;
  share: number;
}

export interface SpecialCard {
  mint: PublicKey;
  cardType: CardType;
  effect: CardEffect;
  rarity: CardRarity;
  usesRemaining: number;
  owner: PublicKey;
  active: boolean;
  activatedAt?: BN;
  expiresAt?: BN;
}

export enum CardType {
  MINING_BOOST = 'mining_boost',
  XP_ACCELERATOR = 'xp_accelerator',
  REFERRAL_POWER = 'referral_power',
  UTILITY = 'utility'
}

export interface CardEffect {
  miningMultiplier?: number;
  xpMultiplier?: number;
  referralBonus?: number;
  duration: BN;
  instant?: boolean;
}

export enum CardRarity {
  COMMON = 'common',
  UNCOMMON = 'uncommon',
  RARE = 'rare',
  EPIC = 'epic',
  LEGENDARY = 'legendary',
  MYTHIC = 'mythic'
}

// ================================
// DeFi Integration Types
// ================================

export interface LiquidityPool {
  poolId: PublicKey;
  tokenA: PublicKey;
  tokenB: PublicKey;
  reserveA: BN;
  reserveB: BN;
  totalShares: BN;
  feeRate: number;
  volume24h: BN;
  tvl: BN;
  apy: number;
}

export interface LiquidityPosition {
  owner: PublicKey;
  poolId: PublicKey;
  shares: BN;
  tokenA: BN;
  tokenB: BN;
  createdAt: BN;
  lastUpdate: BN;
  rewardsEarned: BN;
}

export interface SwapQuote {
  inputMint: PublicKey;
  outputMint: PublicKey;
  inputAmount: BN;
  outputAmount: BN;
  minOutputAmount: BN;
  priceImpact: number;
  fee: BN;
  route: SwapRoute[];
}

export interface SwapRoute {
  poolId: PublicKey;
  inputMint: PublicKey;
  outputMint: PublicKey;
  percentage: number;
}

// ================================
// Governance Types
// ================================

export interface Proposal {
  id: BN;
  proposer: PublicKey;
  title: string;
  description: string;
  proposalType: ProposalType;
  startTime: BN;
  endTime: BN;
  executionTime?: BN;
  status: ProposalStatus;
  votes: Votes;
  quorum: BN;
  parameters?: ProposalParameters;
}

export enum ProposalType {
  PARAMETER_CHANGE = 'parameter_change',
  FEATURE_ADDITION = 'feature_addition',
  TREASURY_ALLOCATION = 'treasury_allocation',
  COMMUNITY_INITIATIVE = 'community_initiative',
  EMERGENCY_ACTION = 'emergency_action'
}

export enum ProposalStatus {
  PENDING = 'pending',
  ACTIVE = 'active',
  SUCCEEDED = 'succeeded',
  DEFEATED = 'defeated',
  QUEUED = 'queued',
  EXECUTED = 'executed',
  CANCELLED = 'cancelled'
}

export interface Votes {
  for: BN;
  against: BN;
  abstain: BN;
  total: BN;
  participants: number;
}

export interface ProposalParameters {
  key: string;
  currentValue: any;
  proposedValue: any;
  impact: string;
}

export interface VotingPower {
  user: PublicKey;
  stakedsFIN: BN;
  xpLevel: number;
  rpTier: RPTier;
  totalPower: BN;
  multiplier: number;
}

// ================================
// Guild System Types
// ================================

export interface Guild {
  id: PublicKey;
  name: string;
  description: string;
  master: PublicKey;
  officers: PublicKey[];
  members: PublicKey[];
  maxMembers: number;
  level: number;
  xp: BN;
  treasury: BN;
  competitions: Competition[];
  createdAt: BN;
  stats: GuildStats;
}

export interface GuildStats {
  totalMining: BN;
  totalXP: BN;
  averageLevel: number;
  activeMembers: number;
  competitionsWon: number;
  ranking: number;
}

export interface Competition {
  id: PublicKey;
  name: string;
  type: 'daily' | 'weekly' | 'monthly' | 'seasonal';
  startTime: BN;
  endTime: BN;
  rewards: CompetitionReward[];
  participants: PublicKey[];
  leaderboard: LeaderboardEntry[];
  status: 'upcoming' | 'active' | 'completed';
}

export interface CompetitionReward {
  position: number;
  amount: BN;
  type: 'FIN' | 'NFT' | 'XP' | 'special';
  item?: PublicKey;
}

export interface LeaderboardEntry {
  user: PublicKey;
  guild?: PublicKey;
  score: BN;
  position: number;
  prizes: CompetitionReward[];
}

// ================================
// Anti-Bot & Security Types
// ================================

export interface UserVerification {
  user: PublicKey;
  kycStatus: KYCStatus;
  humanScore: number;
  riskScore: number;
  verificationLevel: VerificationLevel;
  biometricHash?: string;
  deviceFingerprint: string;
  lastVerification: BN;
  verificationHistory: VerificationEvent[];
}

export enum KYCStatus {
  NOT_STARTED = 'not_started',
  PENDING = 'pending',
  VERIFIED = 'verified',
  REJECTED = 'rejected',
  EXPIRED = 'expired'
}

export enum VerificationLevel {
  BASIC = 'basic',
  ENHANCED = 'enhanced',
  PREMIUM = 'premium'
}

export interface VerificationEvent {
  type: 'kyc' | 'biometric' | 'device' | 'behavior';
  timestamp: BN;
  result: 'pass' | 'fail' | 'pending';
  score: number;
  details?: any;
}

export interface AntiBotCheck {
  user: PublicKey;
  checkType: 'behavior' | 'pattern' | 'network' | 'temporal';
  suspicionLevel: number;
  flags: string[];
  timestamp: BN;
  action: 'none' | 'warning' | 'rate_limit' | 'suspend';
}

// ================================
// Analytics & Metrics Types
// ================================

export interface UserMetrics {
  user: PublicKey;
  registrationDate: BN;
  totalSessions: number;
  avgSessionDuration: number;
  totalTransactions: number;
  totalValue: BN;
  engagementScore: number;
  lifetimeValue: BN;
  churnProbability: number;
}

export interface NetworkMetrics {
  totalUsers: number;
  activeUsers: number;
  totalSupply: BN;
  circulatingSupply: BN;
  totalStaked: BN;
  tvl: BN;
  volume24h: BN;
  transactions24h: number;
  averageReward: BN;
}

export interface PlatformMetrics {
  platform: SocialPlatform;
  activeUsers: number;
  totalPosts: number;
  avgEngagement: number;
  rewardsDistributed: BN;
  topCreators: PublicKey[];
}

// ================================
// Event Types
// ================================

export interface BlockchainEvent {
  signature: TransactionSignature;
  slot: number;
  timestamp: BN;
  programId: PublicKey;
  eventType: string;
  data: any;
}

export interface MiningEvent extends BlockchainEvent {
  eventType: 'mining_started' | 'mining_claimed' | 'rate_updated';
  user: PublicKey;
  amount?: BN;
  rate?: number;
}

export interface XPEvent extends BlockchainEvent {
  eventType: 'xp_gained' | 'level_up' | 'milestone_reached';
  user: PublicKey;
  xpGained?: BN;
  newLevel?: number;
  milestone?: string;
}

export interface ReferralEvent extends BlockchainEvent {
  eventType: 'referral_joined' | 'referral_rewarded' | 'tier_upgraded';
  referrer: PublicKey;
  referee?: PublicKey;
  reward?: BN;
  newTier?: RPTier;
}

// ================================
// API Response Types
// ================================

export interface ApiResponse<T = any> {
  success: boolean;
  data?: T;
  error?: string;
  code?: number;
  timestamp: number;
}

export interface PaginatedResponse<T> extends ApiResponse<T[]> {
  pagination: {
    page: number;
    limit: number;
    total: number;
    hasMore: boolean;
  };
}

export interface BlockchainQueryParams {
  commitment?: 'processed' | 'confirmed' | 'finalized';
  minContextSlot?: number;
  dataSlice?: {
    offset: number;
    length: number;
  };
  filters?: any[];
}

// ================================
// Utility Types
// ================================

export type Wallet = {
  publicKey: PublicKey;
  signTransaction: (tx: Transaction) => Promise<Transaction>;
  signAllTransactions: (txs: Transaction[]) => Promise<Transaction[]>;
  signMessage?: (message: Uint8Array) => Promise<Uint8Array>;
};

export type PriorityLevel = 'low' | 'medium' | 'high' | 'veryHigh';

export interface ComputeUnitPrice {
  microLamports: number;
  units: number;
}

export interface AccountInfo {
  lamports: number;
  owner: PublicKey;
  rentEpoch: number;
  executable: boolean;
  data: Buffer;
}

// ================================
// Error Types
// ================================

export interface BlockchainError extends Error {
  code: number;
  logs?: string[];
  signature?: TransactionSignature;
  instruction?: number;
}

export enum ErrorCode {
  INSUFFICIENT_FUNDS = 1001,
  INVALID_INSTRUCTION = 1002,
  ACCOUNT_NOT_FOUND = 1003,
  UNAUTHORIZED = 1004,
  RATE_LIMIT_EXCEEDED = 1005,
  NETWORK_ERROR = 1006,
  PROGRAM_ERROR = 1007,
  VALIDATION_ERROR = 1008,
  TIMEOUT = 1009,
  UNKNOWN = 9999
}

// ================================
// Configuration Types
// ================================

export interface NetworkConfig {
  name: string;
  rpcUrl: string;
  wsUrl?: string;
  explorerUrl: string;
  chainId: number;
  programs: ProgramIds;
  tokens: FinTokens;
}

export interface FeatureFlags {
  miningEnabled: boolean;
  stakingEnabled: boolean;
  nftEnabled: boolean;
  defiEnabled: boolean;
  governanceEnabled: boolean;
  guildEnabled: boolean;
  bridgeEnabled: boolean;
  maintenanceMode: boolean;
}

// ================================
// Export All Types
// ================================

export * from './mining.types';
export * from './xp.types';
export * from './referral.types';
export * from './nft.types';
export * from './defi.types';
export * from './governance.types';
