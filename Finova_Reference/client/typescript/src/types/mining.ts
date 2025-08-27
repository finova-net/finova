/**
 * Finova Network - Mining System Types
 * Enterprise-grade TypeScript type definitions for the mining ecosystem
 * 
 * @version 4.0
 * @author Finova Network Team
 * @license MIT
 */

import { PublicKey, Connection } from '@solana/web3.js';
import { BN } from '@coral-xyz/anchor';

// ================================================================================================
// CORE MINING TYPES
// ================================================================================================

/**
 * Mining phase configuration based on user growth
 */
export enum MiningPhase {
  FINIZEN = 'finizen',           // 0-100K users
  GROWTH = 'growth',             // 100K-1M users  
  MATURITY = 'maturity',         // 1M-10M users
  STABILITY = 'stability'        // 10M+ users
}

/**
 * Mining phase configuration parameters
 */
export interface MiningPhaseConfig {
  readonly phase: MiningPhase;
  readonly userRange: [number, number];
  readonly baseRate: number;        // Base $FIN per hour
  readonly finizenBonus: number;    // Pioneer multiplier
  readonly maxDaily: number;        // Daily cap in $FIN
  readonly description: string;
}

/**
 * Core mining rate calculation parameters
 */
export interface MiningRateParams {
  readonly baseRate: number;                    // 0.001-0.1 $FIN/hour
  readonly finizenBonus: number;                // max(1.0, 2.0 - (totalUsers/1M))
  readonly referralBonus: number;               // 1 + (activeReferrals * 0.1)
  readonly securityBonus: number;               // KYC verified ? 1.2 : 0.8
  readonly regressionFactor: number;            // e^(-0.001 * userTotalHoldings)
  readonly xpMultiplier: number;                // 1.0x - 5.0x based on XP level
  readonly rpMultiplier: number;                // 1.0x - 3.0x based on RP tier
  readonly qualityScore: number;                // 0.5x - 2.0x AI-validated quality
  readonly networkRegression: number;           // Anti-whale exponential decay
}

/**
 * Mining calculation result with breakdown
 */
export interface MiningCalculation {
  readonly hourlyRate: number;
  readonly dailyRate: number;
  readonly monthlyProjection: number;
  readonly breakdown: {
    readonly base: number;
    readonly finizen: number;
    readonly referral: number;
    readonly security: number;
    readonly regression: number;
    readonly xp: number;
    readonly rp: number;
    readonly quality: number;
  };
  readonly cappedRate: number;
  readonly capReason?: string;
  readonly timestamp: number;
}

// ================================================================================================
// MINING STATE & ACCOUNTS
// ================================================================================================

/**
 * On-chain mining state account structure
 */
export interface MiningState {
  readonly user: PublicKey;
  readonly totalMined: BN;                      // Lifetime mined $FIN
  readonly currentStreak: number;               // Days of continuous mining
  readonly lastMiningTimestamp: BN;             // Last mining claim timestamp
  readonly miningRate: number;                  // Current hourly rate
  readonly dailyCap: number;                    // Daily mining limit
  readonly todayMined: number;                  // Today's mined amount
  readonly lastResetTimestamp: BN;              // Daily reset timestamp
  readonly activeBoosts: MiningBoost[];         // Active mining boosts
  readonly miningHistory: MiningHistoryEntry[]; // Recent mining history
  readonly phaseQualification: MiningPhase;     // Current phase eligibility
  readonly antiWhaleFactor: number;             // Whale regression multiplier
}

/**
 * Mining boost from special cards or events
 */
export interface MiningBoost {
  readonly boostId: string;
  readonly boostType: MiningBoostType;
  readonly multiplier: number;                  // Boost multiplier (e.g., 2.0 for 100% boost)
  readonly duration: number;                    // Boost duration in seconds
  readonly startTimestamp: BN;
  readonly endTimestamp: BN;
  readonly isActive: boolean;
  readonly source: string;                      // Card name or event source
  readonly stackable: boolean;
}

/**
 * Types of mining boosts available
 */
export enum MiningBoostType {
  DOUBLE_MINING = 'double_mining',              // +100% mining rate
  TRIPLE_MINING = 'triple_mining',              // +200% mining rate  
  MINING_FRENZY = 'mining_frenzy',              // +500% mining rate
  ETERNAL_MINER = 'eternal_miner',              // +50% for 30 days
  DAILY_SOCIAL = 'daily_social',                // +20% for social posts
  QUEST_COMPLETE = 'quest_complete',            // +50% for quest completion
  REFERRAL_SUCCESS = 'referral_success',        // +100% for referral KYC
  GUILD_PARTICIPATION = 'guild_participation'   // +30% for guild events
}

/**
 * Mining history entry for tracking and analytics
 */
export interface MiningHistoryEntry {
  readonly timestamp: BN;
  readonly amount: number;
  readonly rate: number;
  readonly boostsActive: string[];
  readonly transactionHash: string;
  readonly blockHeight: number;
}

// ================================================================================================
// MINING REWARDS & DISTRIBUTION
// ================================================================================================

/**
 * Mining reward calculation with integrated XP/RP bonuses
 */
export interface IntegratedMiningReward {
  readonly baseMining: number;                  // Core mining reward
  readonly xpBonus: number;                     // XP level bonus (20% of base)
  readonly rpBonus: number;                     // RP tier bonus (30% of base)
  readonly qualityMultiplier: number;           // AI quality score (0.5x-2.0x)
  readonly finalReward: number;                 // Total reward amount
  readonly formula: string;                     // Calculation formula used
}

/**
 * Mining reward pool configuration
 */
export interface MiningRewardPool {
  readonly totalAllocated: BN;                  // Total $FIN allocated for mining
  readonly dailyAllocation: BN;                 // Daily distribution limit
  readonly currentDistributed: BN;              // Already distributed today
  readonly remainingToday: BN;                  // Remaining for today
  readonly activeMiners: number;                // Currently active miners
  readonly averageRewardPerMiner: number;       // Average daily reward
  readonly poolHealth: PoolHealthStatus;       // Pool sustainability status
  readonly nextRefillTimestamp: BN;             // Next daily refill time
}

/**
 * Mining reward pool health status
 */
export enum PoolHealthStatus {
  HEALTHY = 'healthy',                          // >80% remaining
  MODERATE = 'moderate',                        // 50-80% remaining  
  LOW = 'low',                                  // 20-50% remaining
  CRITICAL = 'critical',                        // <20% remaining
  DEPLETED = 'depleted'                         // 0% remaining
}

// ================================================================================================
// MINING ACTIVITIES & BOOSTERS
// ================================================================================================

/**
 * Mining activity types that provide boosts
 */
export interface MiningActivity {
  readonly activityType: MiningActivityType;
  readonly miningBoostPercent: number;
  readonly duration: number;                    // Boost duration in seconds
  readonly stackable: boolean;
  readonly maxStack: number;
  readonly dailyLimit?: number;
  readonly requirements?: ActivityRequirement[];
}

/**
 * Types of activities that boost mining
 */
export enum MiningActivityType {
  DAILY_SOCIAL_POST = 'daily_social_post',      // +20% for 24h
  COMPLETE_DAILY_QUEST = 'complete_daily_quest', // +50% for 12h
  REFERRAL_KYC_SUCCESS = 'referral_kyc_success', // +100% for 48h
  USE_SPECIAL_CARD = 'use_special_card',        // Variable boost
  GUILD_PARTICIPATION = 'guild_participation',   // +30% for event duration
  VIRAL_CONTENT = 'viral_content',              // +200% for 48h
  STREAK_MILESTONE = 'streak_milestone',        // +25% per 7-day streak
  PLATFORM_INTEGRATION = 'platform_integration' // +15% per platform
}

/**
 * Requirements for mining activities
 */
export interface ActivityRequirement {
  readonly requirementType: RequirementType;
  readonly threshold: number;
  readonly description: string;
}

/**
 * Types of activity requirements
 */
export enum RequirementType {
  MIN_XP_LEVEL = 'min_xp_level',
  MIN_RP_TIER = 'min_rp_tier',
  MIN_FOLLOWERS = 'min_followers',
  KYC_VERIFIED = 'kyc_verified',
  GUILD_MEMBER = 'guild_member',
  PLATFORM_CONNECTED = 'platform_connected'
}

// ================================================================================================
// ANTI-BOT & FAIR DISTRIBUTION
// ================================================================================================

/**
 * Anti-bot detection parameters
 */
export interface AntiBotParams {
  readonly humanProbability: number;            // 0.0 - 1.0 human likelihood
  readonly behaviorScore: number;               // Behavioral analysis score
  readonly patternScore: number;                // Pattern recognition score
  readonly networkScore: number;                // Social network validity
  readonly deviceScore: number;                 // Device authenticity score
  readonly temporalScore: number;               // Temporal pattern score
  readonly overallRiskScore: number;            // Combined risk assessment
  readonly actionRequired: AntiBotAction;       // Required action
}

/**
 * Anti-bot actions based on risk assessment
 */
export enum AntiBotAction {
  NONE = 'none',                               // No action needed
  MINOR_PENALTY = 'minor_penalty',             // 10% mining reduction
  MODERATE_PENALTY = 'moderate_penalty',       // 25% mining reduction
  MAJOR_PENALTY = 'major_penalty',             // 50% mining reduction
  VERIFICATION_REQUIRED = 'verification_required', // Additional KYC
  TEMPORARY_SUSPENSION = 'temporary_suspension',   // 24h mining pause
  PERMANENT_BAN = 'permanent_ban'              // Account termination
}

/**
 * Fair distribution mechanisms
 */
export interface FairDistributionConfig {
  readonly exponentialRegression: boolean;      // Enable regression for whales
  readonly dailyCaps: boolean;                  // Enforce daily mining limits
  readonly qualityRequirements: boolean;       // Require content quality
  readonly coolingPeriods: boolean;            // Mandatory breaks
  readonly progressiveDifficulty: boolean;     // Increasing difficulty
  readonly antiSybilMeasures: boolean;         // Sybil attack prevention
}

// ================================================================================================
// MINING ANALYTICS & STATISTICS
// ================================================================================================

/**
 * Mining network statistics
 */
export interface MiningNetworkStats {
  readonly totalUsers: number;
  readonly activeMiners: number;                // Last 24h activity
  readonly totalMinedToday: BN;
  readonly totalMinedAllTime: BN;
  readonly averageMiningRate: number;
  readonly topMinersByVolume: MinerRanking[];
  readonly topMinersByEfficiency: MinerRanking[];
  readonly phaseDistribution: PhaseDistribution;
  readonly networkHealth: NetworkHealthMetrics;
}

/**
 * Individual miner ranking
 */
export interface MinerRanking {
  readonly rank: number;
  readonly user: PublicKey;
  readonly username?: string;
  readonly totalMined: BN;
  readonly dailyAverage: number;
  readonly efficiency: number;                  // Mining per activity ratio
  readonly streak: number;
}

/**
 * Distribution across mining phases
 */
export interface PhaseDistribution {
  readonly finizen: number;                     // Users in Finizen phase
  readonly growth: number;                      // Users in Growth phase
  readonly maturity: number;                    // Users in Maturity phase
  readonly stability: number;                   // Users in Stability phase
}

/**
 * Network health metrics for mining ecosystem
 */
export interface NetworkHealthMetrics {
  readonly decentralizationIndex: number;       // 0-1, higher = more decentralized
  readonly giniCoefficient: number;             // Wealth distribution inequality
  readonly activeParticipationRate: number;    // % of users actively mining
  readonly sustainabilityRatio: number;        // Reward pool sustainability
  readonly growthRate: number;                  // Network growth rate
  readonly retentionRate: number;               // User retention rate
}

// ================================================================================================
// MINING TRANSACTIONS & OPERATIONS
// ================================================================================================

/**
 * Mining claim transaction parameters
 */
export interface MiningClaimParams {
  readonly user: PublicKey;
  readonly amount: BN;
  readonly timestamp: BN;
  readonly proofOfActivity?: ActivityProof[];
  readonly boostsToApply?: string[];
  readonly qualityScore?: number;
}

/**
 * Proof of mining-eligible activity
 */
export interface ActivityProof {
  readonly activityType: MiningActivityType;
  readonly platform: string;
  readonly contentHash: string;
  readonly engagement: EngagementMetrics;
  readonly timestamp: BN;
  readonly verified: boolean;
}

/**
 * Social media engagement metrics
 */
export interface EngagementMetrics {
  readonly likes: number;
  readonly comments: number;
  readonly shares: number;
  readonly views: number;
  readonly reach: number;
  readonly impressions: number;
  readonly engagementRate: number;
}

/**
 * Mining transaction result
 */
export interface MiningTransactionResult {
  readonly success: boolean;
  readonly transactionHash: string;
  readonly amountMined: BN;
  readonly newTotalMined: BN;
  readonly updatedMiningRate: number;
  readonly boostsApplied: MiningBoost[];
  readonly error?: string;
  readonly gasUsed: number;
  readonly confirmationTime: number;
}

// ================================================================================================
// MINING OPTIMIZATION & STRATEGIES
// ================================================================================================

/**
 * Mining optimization suggestions
 */
export interface MiningOptimization {
  readonly currentEfficiency: number;           // Current mining efficiency %
  readonly potentialIncrease: number;           // Potential increase %
  readonly suggestions: OptimizationSuggestion[];
  readonly estimatedDailyIncrease: number;      // Estimated daily $FIN increase
  readonly implementationDifficulty: DifficultyLevel;
  readonly timeToImplement: number;             // Hours to implement
}

/**
 * Individual optimization suggestion
 */
export interface OptimizationSuggestion {
  readonly suggestionType: OptimizationType;
  readonly description: string;
  readonly impact: number;                      // Expected % improvement
  readonly effort: DifficultyLevel;
  readonly requirements: string[];
  readonly estimatedCost: number;               // Cost in $FIN or time
}

/**
 * Types of mining optimizations
 */
export enum OptimizationType {
  INCREASE_XP_LEVEL = 'increase_xp_level',
  BUILD_REFERRAL_NETWORK = 'build_referral_network',
  IMPROVE_CONTENT_QUALITY = 'improve_content_quality',
  USE_MINING_CARDS = 'use_mining_cards',
  JOIN_ACTIVE_GUILD = 'join_active_guild',
  COMPLETE_KYC = 'complete_kyc',
  MAINTAIN_STREAK = 'maintain_streak',
  DIVERSIFY_PLATFORMS = 'diversify_platforms'
}

/**
 * Implementation difficulty levels
 */
export enum DifficultyLevel {
  EASY = 'easy',                               // <1 hour
  MODERATE = 'moderate',                       // 1-24 hours
  HARD = 'hard',                              // 1-7 days
  EXPERT = 'expert'                           // >7 days
}

// ================================================================================================
// MINING EVENTS & NOTIFICATIONS
// ================================================================================================

/**
 * Mining-related events
 */
export interface MiningEvent {
  readonly eventType: MiningEventType;
  readonly user: PublicKey;
  readonly timestamp: BN;
  readonly data: MiningEventData;
  readonly severity: EventSeverity;
}

/**
 * Types of mining events
 */
export enum MiningEventType {
  MINING_CLAIMED = 'mining_claimed',
  BOOST_ACTIVATED = 'boost_activated',
  BOOST_EXPIRED = 'boost_expired',
  STREAK_MILESTONE = 'streak_milestone',
  DAILY_CAP_REACHED = 'daily_cap_reached',
  PHASE_TRANSITION = 'phase_transition',
  ANTI_BOT_TRIGGERED = 'anti_bot_triggered',
  POOL_LOW = 'pool_low',
  OPTIMIZATION_AVAILABLE = 'optimization_available'
}

/**
 * Mining event data payload
 */
export type MiningEventData = 
  | MiningClaimedData
  | BoostEventData
  | StreakMilestoneData
  | PhaseTransitionData
  | AntiBotEventData
  | PoolStatusData
  | OptimizationData;

/**
 * Mining claimed event data
 */
export interface MiningClaimedData {
  readonly amount: BN;
  readonly rate: number;
  readonly boostsUsed: string[];
  readonly efficiency: number;
}

/**
 * Boost activation/expiration event data
 */
export interface BoostEventData {
  readonly boostId: string;
  readonly boostType: MiningBoostType;
  readonly multiplier: number;
  readonly duration?: number;
}

/**
 * Streak milestone event data
 */
export interface StreakMilestoneData {
  readonly streakDays: number;
  readonly milestone: number;
  readonly bonus: number;
  readonly nextMilestone: number;
}

/**
 * Phase transition event data
 */
export interface PhaseTransitionData {
  readonly fromPhase: MiningPhase;
  readonly toPhase: MiningPhase;
  readonly newBaseRate: number;
  readonly reason: string;
}

/**
 * Anti-bot event data
 */
export interface AntiBotEventData {
  readonly riskScore: number;
  readonly action: AntiBotAction;
  readonly reason: string;
  readonly duration?: number;
}

/**
 * Pool status event data
 */
export interface PoolStatusData {
  readonly currentLevel: number;
  readonly healthStatus: PoolHealthStatus;
  readonly timeToRefill: number;
  readonly affectedUsers: number;
}

/**
 * Optimization suggestion event data
 */
export interface OptimizationData {
  readonly currentEfficiency: number;
  readonly potentialGain: number;
  readonly topSuggestion: OptimizationType;
  readonly effort: DifficultyLevel;
}

/**
 * Event severity levels
 */
export enum EventSeverity {
  INFO = 'info',
  WARNING = 'warning',
  ERROR = 'error',
  CRITICAL = 'critical'
}

// ================================================================================================
// MINING CLIENT INTERFACES
// ================================================================================================

/**
 * Mining client interface for blockchain interactions
 */
export interface MiningClient {
  // Core mining operations
  claimMiningRewards(params: MiningClaimParams): Promise<MiningTransactionResult>;
  getMiningState(user: PublicKey): Promise<MiningState>;
  calculateMiningRate(user: PublicKey): Promise<MiningCalculation>;
  
  // Boost management
  activateBoost(user: PublicKey, boostId: string): Promise<boolean>;
  getActiveBoosts(user: PublicKey): Promise<MiningBoost[]>;
  
  // Analytics and stats
  getNetworkStats(): Promise<MiningNetworkStats>;
  getUserRanking(user: PublicKey): Promise<MinerRanking>;
  getMiningHistory(user: PublicKey, limit?: number): Promise<MiningHistoryEntry[]>;
  
  // Optimization
  getOptimizationSuggestions(user: PublicKey): Promise<MiningOptimization>;
  
  // Events
  subscribeToMiningEvents(user: PublicKey, callback: (event: MiningEvent) => void): () => void;
}

/**
 * Mining service configuration
 */
export interface MiningServiceConfig {
  readonly connection: Connection;
  readonly programId: PublicKey;
  readonly cluster: 'devnet' | 'testnet' | 'mainnet-beta';
  readonly retryAttempts: number;
  readonly timeoutMs: number;
  readonly cacheTtlMs: number;
  readonly enableEvents: boolean;
  readonly enableOptimization: boolean;
}

// ================================================================================================
// UTILITY TYPES & HELPERS
// ================================================================================================

/**
 * Mining calculation utilities
 */
export interface MiningUtils {
  calculateHourlyRate(params: MiningRateParams): number;
  applyRegressionFactor(baseRate: number, totalHoldings: BN): number;
  calculateBoostMultiplier(boosts: MiningBoost[]): number;
  estimateOptimalMiningTime(user: PublicKey): Promise<number>;
  validateActivityProof(proof: ActivityProof): boolean;
  formatMiningAmount(amount: BN, decimals?: number): string;
  parseMiningAmount(amount: string): BN;
}

/**
 * Mining constants and limits
 */
export const MINING_CONSTANTS = {
  // Base rates by phase
  PHASE_RATES: {
    [MiningPhase.FINIZEN]: 0.1,
    [MiningPhase.GROWTH]: 0.05,
    [MiningPhase.MATURITY]: 0.025,
    [MiningPhase.STABILITY]: 0.01
  },
  
  // Multiplier limits
  MAX_XP_MULTIPLIER: 5.0,
  MAX_RP_MULTIPLIER: 3.0,
  MAX_QUALITY_SCORE: 2.0,
  MIN_QUALITY_SCORE: 0.5,
  
  // Security parameters
  KYC_BONUS: 1.2,
  NON_KYC_PENALTY: 0.8,
  
  // Regression parameters
  REGRESSION_COEFFICIENT: 0.001,
  WHALE_THRESHOLD: 100000, // $FIN
  
  // Time constants
  SECONDS_PER_HOUR: 3600,
  SECONDS_PER_DAY: 86400,
  MILLISECONDS_PER_SECOND: 1000,
  
  // Pool health thresholds
  HEALTHY_THRESHOLD: 0.8,
  MODERATE_THRESHOLD: 0.5,
  LOW_THRESHOLD: 0.2,
  CRITICAL_THRESHOLD: 0.05,
  
  // Mining limits
  MAX_DAILY_CAP: 15.0, // $FIN for Mythic level
  MIN_DAILY_CAP: 0.24,  // $FIN for Stability phase
  
  // Boost limits
  MAX_BOOST_MULTIPLIER: 6.0, // 500% + base
  MAX_BOOST_DURATION: 2592000, // 30 days in seconds
  MAX_SIMULTANEOUS_BOOSTS: 5
} as const;

/**
 * Type guards for mining types
 */
export const MiningTypeGuards = {
  isMiningState: (obj: any): obj is MiningState => {
    return obj && typeof obj.user === 'object' && typeof obj.totalMined === 'object';
  },
  
  isMiningBoost: (obj: any): obj is MiningBoost => {
    return obj && typeof obj.boostId === 'string' && typeof obj.multiplier === 'number';
  },
  
  isMiningEvent: (obj: any): obj is MiningEvent => {
    return obj && typeof obj.eventType === 'string' && typeof obj.user === 'object';
  },
  
  isValidMiningPhase: (phase: string): phase is MiningPhase => {
    return Object.values(MiningPhase).includes(phase as MiningPhase);
  }
} as const;

/**
 * Mining error types
 */
export enum MiningErrorCode {
  INSUFFICIENT_BALANCE = 'insufficient_balance',
  DAILY_CAP_EXCEEDED = 'daily_cap_exceeded',
  BOT_DETECTED = 'bot_detected',
  INVALID_PROOF = 'invalid_proof',
  BOOST_EXPIRED = 'boost_expired',
  NETWORK_ERROR = 'network_error',
  UNAUTHORIZED = 'unauthorized',
  RATE_LIMITED = 'rate_limited',
  POOL_DEPLETED = 'pool_depleted',
  INVALID_PARAMETERS = 'invalid_parameters'
}

/**
 * Mining error class
 */
export class MiningError extends Error {
  constructor(
    public readonly code: MiningErrorCode,
    message: string,
    public readonly details?: any
  ) {
    super(message);
    this.name = 'MiningError';
  }
}

// ================================================================================================
// EXPORT ALL TYPES
// ================================================================================================

export * from './user.js';
export * from './xp.js';
export * from './referral.js';
export * from './nft.js';

// Re-export commonly used Solana types
export type { PublicKey, Connection, Transaction, TransactionSignature } from '@solana/web3.js';
export type { BN } from '@coral-xyz/anchor';

/**
 * Complete mining system type exports
 */
export type MiningSystemTypes = {
  // Core types
  MiningPhase: typeof MiningPhase;
  MiningState: MiningState;
  MiningCalculation: MiningCalculation;
  MiningRateParams: MiningRateParams;
  
  // Boosts and activities
  MiningBoost: MiningBoost;
  MiningActivity: MiningActivity;
  MiningActivityType: typeof MiningActivityType;
  
  // Analytics and stats
  MiningNetworkStats: MiningNetworkStats;
  MinerRanking: MinerRanking;
  
  // Events and notifications
  MiningEvent: MiningEvent;
  MiningEventType: typeof MiningEventType;
  
  // Client interface
  MiningClient: MiningClient;
  
  // Utils and constants
  MiningUtils: MiningUtils;
  MINING_CONSTANTS: typeof MINING_CONSTANTS;
};

// Default export for convenience
export default {
  MiningPhase,
  MiningActivityType,
  MiningBoostType,
  MiningEventType,
  MiningErrorCode,
  AntiBotAction,
  PoolHealthStatus,
  OptimizationType,
  DifficultyLevel,
  EventSeverity,
  MINING_CONSTANTS,
  MiningTypeGuards,
  MiningError
} as const;
