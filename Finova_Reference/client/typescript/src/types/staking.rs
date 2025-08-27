// finova-net/finova/client/typescript/src/types/staking.ts

/**
 * Finova Network - Staking System Types
 * @file finova-net/finova/client/typescript/src/types/staking.ts
 * @version 3.0
 * @description Complete type definitions for Finova's liquid staking system
 * with integrated XP, RP, and mining rewards
 * @author Finova Network Development Team
 * @date July 2025
 */

import { PublicKey } from '@solana/web3.js';
import { BN } from '@coral-xyz/anchor';

// ============================================================================
// CORE STAKING TYPES
// ============================================================================

/**
 * Staking tier based on staked amount with integrated benefits
 */
export enum StakingTier {
  BRONZE = 'bronze',      // 100-499 $FIN
  SILVER = 'silver',      // 500-999 $FIN  
  GOLD = 'gold',          // 1,000-4,999 $FIN
  PLATINUM = 'platinum',  // 5,000-9,999 $FIN
  DIAMOND = 'diamond',    // 10,000+ $FIN
}

/**
 * Staking duration options with loyalty bonuses
 */
export enum StakingDuration {
  FLEXIBLE = 'flexible',    // No lock, lower rewards
  WEEK_1 = '1_week',       // 7 days lock
  MONTH_1 = '1_month',     // 30 days lock
  MONTH_3 = '3_months',    // 90 days lock
  MONTH_6 = '6_months',    // 180 days lock
  YEAR_1 = '1_year',       // 365 days lock
}

/**
 * Staking account state
 */
export enum StakingStatus {
  ACTIVE = 'active',
  PENDING = 'pending',
  UNSTAKING = 'unstaking',
  WITHDRAWN = 'withdrawn',
  SLASHED = 'slashed',
}

// ============================================================================
// STAKING CONFIGURATION
// ============================================================================

/**
 * Staking tier configuration with all benefits
 */
export interface StakingTierConfig {
  tier: StakingTier;
  minStakeAmount: BN;
  maxStakeAmount: BN | null;
  baseAPY: number; // Annual percentage yield (8-15%)
  miningBoost: number; // Mining rate multiplier (1.2x - 2.0x)
  xpMultiplier: number; // XP gain bonus (1.1x - 1.75x)
  rpBonus: number; // RP network bonus (1.05x - 1.5x)
  specialFeatures: StakingFeature[];
  votingWeight: number; // DAO governance weight multiplier
  prioritySupport: boolean;
  exclusiveEvents: boolean;
}

/**
 * Special features unlocked by staking tiers
 */
export enum StakingFeature {
  BASIC_REWARDS = 'basic_rewards',
  PREMIUM_BADGE = 'premium_badge',
  PRIORITY_SUPPORT = 'priority_support',
  VIP_FEATURES = 'vip_features',
  EXCLUSIVE_EVENTS = 'exclusive_events',
  GUILD_MASTER = 'guild_master',
  DAO_GOVERNANCE = 'dao_governance',
  CREATOR_MONETIZATION = 'creator_monetization',
  ADVANCED_ANALYTICS = 'advanced_analytics',
  CUSTOM_NFT_ACCESS = 'custom_nft_access',
}

/**
 * Staking pool configuration
 */
export interface StakingPoolConfig {
  poolId: string;
  name: string;
  description: string;
  totalStaked: BN;
  maxCapacity: BN | null;
  baseAPY: number;
  performanceAPY: number; // Additional APY based on pool performance
  lockPeriod: number; // Minimum lock period in seconds
  earlyWithdrawalPenalty: number; // Penalty percentage for early withdrawal
  createdAt: Date;
  updatedAt: Date;
  isActive: boolean;
}

// ============================================================================
// STAKING ACCOUNT TYPES
// ============================================================================

/**
 * Individual staking position
 */
export interface StakeAccount {
  // Account identifiers
  address: PublicKey;
  owner: PublicKey;
  poolId: string;
  
  // Staking details
  stakedAmount: BN; // Amount of $FIN staked
  sFINAmount: BN; // Amount of $sFIN received (liquid staking tokens)
  stakingTier: StakingTier;
  stakingDuration: StakingDuration;
  
  // Timestamps
  stakeTimestamp: BN; // When stake was created
  unlockTimestamp: BN; // When stake can be withdrawn
  lastRewardClaim: BN; // Last reward claim timestamp
  
  // Status and configuration
  status: StakingStatus;
  autoCompound: boolean; // Automatically compound rewards
  autoRenew: boolean; // Automatically renew lock period
  
  // Reward tracking
  totalRewardsEarned: BN;
  pendingRewards: BN;
  claimedRewards: BN;
  
  // Multiplier effects
  multipliers: StakingMultipliers;
  
  // Integration data
  xpIntegration: XPStakingIntegration;
  rpIntegration: RPStakingIntegration;
  miningIntegration: MiningStakingIntegration;
}

/**
 * Comprehensive multiplier system for staking rewards
 */
export interface StakingMultipliers {
  // Base multipliers from tier
  tierMultiplier: number; // Based on staking tier
  loyaltyMultiplier: number; // Based on staking duration
  
  // Activity-based multipliers
  activityMultiplier: number; // Based on daily activity score
  qualityMultiplier: number; // Based on content quality
  
  // Integration multipliers
  xpLevelMultiplier: number; // Based on XP level (1.0x + XP_Level/100)
  rpTierMultiplier: number; // Based on RP tier (1.0x + RP_Tier*0.2)
  
  // Network multipliers
  networkEffectMultiplier: number; // Based on referral network size
  guildMultiplier: number; // Based on guild participation
  
  // Special multipliers
  eventMultiplier: number; // Special event bonuses
  nftMultiplier: number; // NFT-based bonuses
  
  // Final combined multiplier
  totalMultiplier: number; // Product of all applicable multipliers
}

// ============================================================================
// INTEGRATION TYPES
// ============================================================================

/**
 * XP system integration with staking
 */
export interface XPStakingIntegration {
  currentXPLevel: number;
  xpMultiplierBonus: number; // Bonus to XP gains from staking
  xpLevelMiningBonus: number; // Mining bonus from XP level
  levelBasedRewards: XPLevelReward[];
  stakingXPBonus: number; // XP gained from staking activities
}

/**
 * XP level rewards for stakers
 */
export interface XPLevelReward {
  level: number;
  miningBonus: number;
  stakingAPYBonus: number;
  unlockFeatures: string[];
}

/**
 * RP system integration with staking
 */
export interface RPStakingIntegration {
  currentRPTier: string;
  rpMultiplierBonus: number; // Bonus to RP gains from staking
  networkStakingBonus: number; // Bonus from referral network staking
  referralStakingRewards: BN; // Rewards from referrals' staking
  tierBasedBenefits: RPTierBenefit[];
}

/**
 * RP tier benefits for stakers
 */
export interface RPTierBenefit {
  tier: string;
  stakingAPYBonus: number;
  referralRewardShare: number; // Percentage of referrals' staking rewards
  networkMultiplier: number;
  specialAccess: string[];
}

/**
 * Mining system integration with staking
 */
export interface MiningStakingIntegration {
  stakingMiningBoost: number; // Direct mining rate boost from staking
  stakingBasedCap: BN; // Increased daily mining cap from staking
  specialMiningEvents: StakingMiningEvent[];
  stakingMiningHistory: StakingMiningRecord[];
}

/**
 * Special mining events for stakers
 */
export interface StakingMiningEvent {
  eventId: string;
  name: string;
  description: string;
  requiredTier: StakingTier;
  miningMultiplier: number;
  duration: number; // Duration in seconds
  startTime: Date;
  endTime: Date;
  isActive: boolean;
}

/**
 * Mining record for staking integration
 */
export interface StakingMiningRecord {
  timestamp: Date;
  baseMiningRate: number;
  stakingBoost: number;
  finalMiningRate: number;
  finMined: BN;
  multiplierBreakdown: StakingMultipliers;
}

// ============================================================================
// REWARD CALCULATION TYPES
// ============================================================================

/**
 * Comprehensive reward calculation parameters
 */
export interface StakingRewardCalculation {
  // Base calculation inputs
  stakedAmount: BN;
  stakingDuration: number; // Duration in seconds
  baseAPY: number;
  
  // Pool performance
  poolPerformance: number; // Pool-specific performance multiplier
  totalPoolStaked: BN;
  userPoolShare: number; // User's share of the pool
  
  // Multiplier effects
  multipliers: StakingMultipliers;
  
  // Integration effects
  xpBonus: number;
  rpBonus: number;
  miningIntegration: number;
  
  // Time-based calculations
  stakingAge: number; // How long staked in seconds
  compoundingPeriods: number; // Number of compounding periods
  
  // Final reward amounts
  baseRewards: BN;
  bonusRewards: BN;
  totalRewards: BN;
  projectedAPY: number; // Actual APY including all bonuses
}

/**
 * Reward distribution structure
 */
export interface RewardDistribution {
  // Core staking rewards (40% of pool)
  baseStakingRewards: BN;
  
  // Activity-based bonuses (25% of pool)
  activityBonuses: BN;
  xpActivityRewards: BN;
  rpActivityRewards: BN;
  miningActivityRewards: BN;
  
  // Loyalty rewards (20% of pool)
  loyaltyRewards: BN;
  durationBonus: BN;
  tierLoyaltyBonus: BN;
  
  // Performance incentives (10% of pool)
  performanceRewards: BN;
  qualityContentBonus: BN;
  networkGrowthBonus: BN;
  
  // Special event bonuses (5% of pool)
  eventRewards: BN;
  seasonalBonuses: BN;
  achievementRewards: BN;
  
  // Total distributed
  totalDistributed: BN;
}

// ============================================================================
// TRANSACTION TYPES
// ============================================================================

/**
 * Staking transaction types
 */
export enum StakingTransactionType {
  STAKE = 'stake',
  UNSTAKE = 'unstake',
  CLAIM_REWARDS = 'claim_rewards',
  COMPOUND = 'compound',
  UPGRADE_TIER = 'upgrade_tier',
  EXTEND_LOCK = 'extend_lock',
  EMERGENCY_WITHDRAW = 'emergency_withdraw',
}

/**
 * Staking transaction record
 */
export interface StakingTransaction {
  signature: string;
  type: StakingTransactionType;
  user: PublicKey;
  stakeAccount: PublicKey;
  
  // Transaction details
  amount: BN;
  timestamp: Date;
  blockHeight: number;
  
  // Before/after state
  beforeState: Partial<StakeAccount>;
  afterState: Partial<StakeAccount>;
  
  // Transaction metadata
  metadata: {
    tier?: StakingTier;
    duration?: StakingDuration;
    multipliers?: StakingMultipliers;
    fees?: BN;
    slippage?: number;
  };
  
  // Status
  status: 'pending' | 'confirmed' | 'failed';
  error?: string;
}

// ============================================================================
// API INTERFACES
// ============================================================================

/**
 * Staking API request types
 */
export interface StakeRequest {
  amount: BN;
  duration: StakingDuration;
  autoCompound?: boolean;
  autoRenew?: boolean;
  poolId?: string;
}

export interface UnstakeRequest {
  stakeAccountAddress: PublicKey;
  amount?: BN; // Partial unstake amount, full if not specified
  forceWithdraw?: boolean; // Accept penalty for early withdrawal
}

export interface ClaimRewardsRequest {
  stakeAccountAddress: PublicKey;
  compound?: boolean; // Compound rewards instead of claiming
}

export interface StakingQuery {
  userAddress?: PublicKey;
  poolId?: string;
  tier?: StakingTier;
  status?: StakingStatus;
  minAmount?: BN;
  maxAmount?: BN;
  createdAfter?: Date;
  createdBefore?: Date;
  sortBy?: 'amount' | 'rewards' | 'apy' | 'created_at';
  sortOrder?: 'asc' | 'desc';
  limit?: number;
  offset?: number;
}

/**
 * Staking API response types
 */
export interface StakingResponse<T> {
  success: boolean;
  data?: T;
  error?: {
    code: string;
    message: string;
    details?: any;
  };
  metadata?: {
    timestamp: Date;
    requestId: string;
    processingTime: number;
  };
}

export interface StakingListResponse {
  stakes: StakeAccount[];
  totalCount: number;
  totalStaked: BN;
  averageAPY: number;
  pagination: {
    limit: number;
    offset: number;
    hasMore: boolean;
  };
}

// ============================================================================
// ANALYTICS TYPES
// ============================================================================

/**
 * Staking analytics and metrics
 */
export interface StakingAnalytics {
  // Overall metrics
  totalValueLocked: BN; // TVL in $FIN
  totalStakers: number;
  averageStakeSize: BN;
  totalRewardsDistributed: BN;
  
  // Tier distribution
  tierDistribution: Record<StakingTier, {
    count: number;
    totalStaked: BN;
    averageAPY: number;
  }>;
  
  // Pool metrics
  poolMetrics: Record<string, {
    tvl: BN;
    stakers: number;
    apy: number;
    performance: number;
  }>;
  
  // Time-series data
  historical: StakingHistoricalData[];
  
  // Performance metrics
  rewardDistributionEfficiency: number;
  stakingParticipationRate: number;
  averageStakingDuration: number;
  churnRate: number;
}

/**
 * Historical staking data point
 */
export interface StakingHistoricalData {
  timestamp: Date;
  tvl: BN;
  stakers: number;
  averageAPY: number;
  rewardsDistributed: BN;
  newStakes: number;
  unstakes: number;
  netFlow: BN;
}

// ============================================================================
// EVENT TYPES
// ============================================================================

/**
 * Staking-related events for real-time updates
 */
export enum StakingEventType {
  STAKE_CREATED = 'stake_created',
  STAKE_UPDATED = 'stake_updated',
  REWARDS_CLAIMED = 'rewards_claimed',
  TIER_UPGRADED = 'tier_upgraded',
  LOCK_EXTENDED = 'lock_extended',
  UNSTAKE_INITIATED = 'unstake_initiated',
  WITHDRAWAL_COMPLETED = 'withdrawal_completed',
  MULTIPLIER_UPDATED = 'multiplier_updated',
}

/**
 * Staking event payload
 */
export interface StakingEvent {
  type: StakingEventType;
  timestamp: Date;
  user: PublicKey;
  stakeAccount: PublicKey;
  data: {
    amount?: BN;
    tier?: StakingTier;
    rewards?: BN;
    multiplier?: number;
    metadata?: Record<string, any>;
  };
}

// ============================================================================
// UTILITY TYPES
// ============================================================================

/**
 * Staking calculator input parameters
 */
export interface StakingCalculatorInput {
  amount: BN;
  duration: StakingDuration;
  currentXPLevel: number;
  currentRPTier: string;
  hasActiveNFTs: boolean;
  guildParticipation: boolean;
  expectedActivity: 'low' | 'medium' | 'high';
}

/**
 * Staking calculator results
 */
export interface StakingCalculatorResult {
  // Basic calculations
  tier: StakingTier;
  baseAPY: number;
  projectedAPY: number;
  
  // Reward projections
  dailyRewards: BN;
  monthlyRewards: BN;
  yearlyRewards: BN;
  totalProjectedRewards: BN;
  
  // Multiplier breakdown
  multiplierBreakdown: {
    tier: number;
    loyalty: number;
    xpLevel: number;
    rpTier: number;
    activity: number;
    total: number;
  };
  
  // Integration benefits
  miningBoost: number;
  xpMultiplier: number;
  rpBonus: number;
  
  // Risk assessment
  lockPeriod: number;
  earlyWithdrawalPenalty: number;
  riskLevel: 'low' | 'medium' | 'high';
}

// ============================================================================
// ERROR TYPES
// ============================================================================

/**
 * Staking-specific error codes
 */
export enum StakingErrorCode {
  INSUFFICIENT_BALANCE = 'insufficient_balance',
  MINIMUM_STAKE_NOT_MET = 'minimum_stake_not_met',
  MAXIMUM_STAKE_EXCEEDED = 'maximum_stake_exceeded',
  STAKE_LOCKED = 'stake_locked',
  INVALID_DURATION = 'invalid_duration',
  POOL_FULL = 'pool_full',
  POOL_INACTIVE = 'pool_inactive',
  UNAUTHORIZED = 'unauthorized',
  ACCOUNT_NOT_FOUND = 'account_not_found',
  CALCULATION_ERROR = 'calculation_error',
  NETWORK_ERROR = 'network_error',
  CONTRACT_ERROR = 'contract_error',
}

/**
 * Staking error with context
 */
export interface StakingError extends Error {
  code: StakingErrorCode;
  context?: {
    userAddress?: PublicKey;
    stakeAccount?: PublicKey;
    amount?: BN;
    tier?: StakingTier;
    additionalInfo?: Record<string, any>;
  };
}

// ============================================================================
// EXPORT TYPES
// ============================================================================

/**
 * Main staking types export
 */
export type {
  // Core types
  StakeAccount,
  StakingTierConfig,
  StakingPoolConfig,
  StakingMultipliers,
  
  // Integration types
  XPStakingIntegration,
  RPStakingIntegration,
  MiningStakingIntegration,
  
  // Calculation types
  StakingRewardCalculation,
  RewardDistribution,
  
  // Transaction types
  StakingTransaction,
  
  // API types
  StakeRequest,
  UnstakeRequest,
  ClaimRewardsRequest,
  StakingQuery,
  StakingResponse,
  StakingListResponse,
  
  // Analytics types
  StakingAnalytics,
  StakingHistoricalData,
  
  // Event types
  StakingEvent,
  
  // Utility types
  StakingCalculatorInput,
  StakingCalculatorResult,
  StakingError,
};

/**
 * Default export with all enums and interfaces
 */
export default {
  // Enums
  StakingTier,
  StakingDuration,
  StakingStatus,
  StakingFeature,
  StakingTransactionType,
  StakingEventType,
  StakingErrorCode,
  
  // Type guards and utilities will be added in separate utility files
};
