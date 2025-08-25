// finova-net/finova/client/typescript/src/types/referral.ts

/**
 * Finova Network - Referral System Types
 * 
 * Enterprise-grade TypeScript type definitions for the referral system
 * Implements the RP (Referral Points) system as described in the whitepaper
 * 
 * @version 3.0.0
 * @author Finova Network Team
 * @license MIT
 */

import { PublicKey } from '@solana/web3.js';
import { BN } from '@project-serum/anchor';

// ============================================================================
// CORE REFERRAL TYPES
// ============================================================================

/**
 * Referral tier enum based on RP (Referral Points) accumulated
 * Each tier unlocks different benefits and multipliers
 */
export enum ReferralTier {
  EXPLORER = 'Explorer',        // 0-999 RP
  CONNECTOR = 'Connector',      // 1,000-4,999 RP
  INFLUENCER = 'Influencer',    // 5,000-14,999 RP
  LEADER = 'Leader',            // 15,000-49,999 RP
  AMBASSADOR = 'Ambassador'     // 50,000+ RP
}

/**
 * Types of referral actions that generate RP
 */
export enum ReferralActionType {
  SIGNUP = 'signup',
  KYC_COMPLETION = 'kyc_completion',
  FIRST_MINING = 'first_mining',
  DAILY_ACTIVITY = 'daily_activity',
  MILESTONE_ACHIEVEMENT = 'milestone_achievement',
  NETWORK_GROWTH = 'network_growth'
}

/**
 * Referral network level (L1, L2, L3)
 */
export enum ReferralLevel {
  L1 = 1,  // Direct referrals
  L2 = 2,  // Second level referrals
  L3 = 3   // Third level referrals
}

/**
 * Referral status for tracking user state
 */
export enum ReferralStatus {
  PENDING = 'pending',
  ACTIVE = 'active',
  INACTIVE = 'inactive',
  VERIFIED = 'verified',
  SUSPENDED = 'suspended'
}

// ============================================================================
// REFERRAL ACCOUNT STRUCTURES
// ============================================================================

/**
 * Main referral account structure stored on-chain
 */
export interface ReferralAccount {
  /** User's public key */
  user: PublicKey;
  
  /** Referrer's public key (who invited this user) */
  referrer: PublicKey | null;
  
  /** Custom referral code for this user */
  referralCode: string;
  
  /** Total Referral Points (RP) accumulated */
  totalRp: BN;
  
  /** Current referral tier */
  tier: ReferralTier;
  
  /** Direct referrals count */
  directReferrals: number;
  
  /** Total network size (L1 + L2 + L3) */
  totalNetworkSize: number;
  
  /** Network quality score (0.0 - 1.0) */
  networkQualityScore: number;
  
  /** Active referrals in last 30 days */
  activeReferrals: number;
  
  /** Total earnings from referral bonuses */
  totalEarnings: BN;
  
  /** Account creation timestamp */
  createdAt: BN;
  
  /** Last activity timestamp */
  lastActivity: BN;
  
  /** Account status */
  status: ReferralStatus;
  
  /** Reserved space for future upgrades */
  reserved: number[];
}

/**
 * Referral network node representing a single referral relationship
 */
export interface ReferralNode {
  /** Referral ID */
  id: string;
  
  /** Referred user's public key */
  referredUser: PublicKey;
  
  /** Referrer's public key */
  referrer: PublicKey;
  
  /** Referral level (L1, L2, L3) */
  level: ReferralLevel;
  
  /** Registration timestamp */
  registeredAt: BN;
  
  /** KYC completion timestamp */
  kycCompletedAt: BN | null;
  
  /** First mining activity timestamp */
  firstMiningAt: BN | null;
  
  /** Last activity timestamp */
  lastActivityAt: BN;
  
  /** Total activity score */
  activityScore: number;
  
  /** User's current XP level */
  xpLevel: number;
  
  /** Total FIN mined by this referral */
  totalFinMined: BN;
  
  /** Is user active in last 30 days */
  isActive: boolean;
  
  /** Referral status */
  status: ReferralStatus;
}

/**
 * Referral statistics and metrics
 */
export interface ReferralStats {
  /** Direct referrals breakdown */
  directReferrals: {
    total: number;
    active: number;
    verified: number;
    inactive: number;
  };
  
  /** Network breakdown by level */
  networkByLevel: {
    l1: { total: number; active: number };
    l2: { total: number; active: number };
    l3: { total: number; active: number };
  };
  
  /** Total network statistics */
  totalNetwork: {
    size: number;
    activeUsers: number;
    qualityScore: number;
    retentionRate: number;
  };
  
  /** Earnings breakdown */
  earnings: {
    totalRp: BN;
    totalFinEarned: BN;
    dailyAverage: BN;
    monthlyAverage: BN;
  };
  
  /** Performance metrics */
  performance: {
    conversionRate: number;
    averageActivityLevel: number;
    networkGrowthRate: number;
    churnRate: number;
  };
}

// ============================================================================
// REFERRAL CALCULATION TYPES
// ============================================================================

/**
 * Referral bonus calculation parameters
 */
export interface ReferralBonusParams {
  /** Base referral activity */
  baseActivity: number;
  
  /** Referral's current level */
  referralLevel: number;
  
  /** Time decay factor */
  timeDecayFactor: number;
  
  /** Quality multiplier */
  qualityMultiplier: number;
  
  /** Network effect multiplier */
  networkMultiplier: number;
}

/**
 * RP calculation result
 */
export interface RpCalculationResult {
  /** Direct RP from L1 referrals */
  directRp: BN;
  
  /** Indirect RP from L2 and L3 network */
  indirectRp: BN;
  
  /** Network quality bonus */
  qualityBonus: BN;
  
  /** Total RP earned */
  totalRp: BN;
  
  /** Applied multipliers */
  multipliers: {
    tierMultiplier: number;
    qualityMultiplier: number;
    networkMultiplier: number;
    regressionFactor: number;
  };
  
  /** Calculation breakdown */
  breakdown: {
    l1Contribution: BN;
    l2Contribution: BN;
    l3Contribution: BN;
    bonusContribution: BN;
  };
}

/**
 * Mining bonus from referral system
 */
export interface ReferralMiningBonus {
  /** Base mining rate */
  baseMiningRate: BN;
  
  /** RP tier multiplier */
  rpTierMultiplier: number;
  
  /** Network effect bonus */
  networkBonus: number;
  
  /** Final mining rate with referral bonuses */
  finalMiningRate: BN;
  
  /** Bonus breakdown */
  bonusBreakdown: {
    tierBonus: number;
    networkBonus: number;
    qualityBonus: number;
    totalBonus: number;
  };
}

// ============================================================================
// REFERRAL TIER CONFIGURATION
// ============================================================================

/**
 * Configuration for each referral tier
 */
export interface ReferralTierConfig {
  /** Tier identifier */
  tier: ReferralTier;
  
  /** Minimum RP required */
  minRp: number;
  
  /** Maximum RP for this tier */
  maxRp: number;
  
  /** Mining bonus percentage */
  miningBonus: number;
  
  /** Referral bonus percentages by level */
  referralBonuses: {
    l1: number;
    l2: number;
    l3: number;
  };
  
  /** Maximum network cap */
  networkCap: number;
  
  /** Special benefits */
  benefits: string[];
  
  /** Required conditions */
  requirements: {
    minDirectReferrals?: number;
    minActiveReferrals?: number;
    minNetworkQuality?: number;
    kycRequired?: boolean;
  };
}

/**
 * Complete referral tier configuration mapping
 */
export const REFERRAL_TIER_CONFIG: Record<ReferralTier, ReferralTierConfig> = {
  [ReferralTier.EXPLORER]: {
    tier: ReferralTier.EXPLORER,
    minRp: 0,
    maxRp: 999,
    miningBonus: 0,
    referralBonuses: { l1: 10, l2: 0, l3: 0 },
    networkCap: 10,
    benefits: ['Basic referral link'],
    requirements: {}
  },
  [ReferralTier.CONNECTOR]: {
    tier: ReferralTier.CONNECTOR,
    minRp: 1000,
    maxRp: 4999,
    miningBonus: 20,
    referralBonuses: { l1: 15, l2: 5, l3: 0 },
    networkCap: 25,
    benefits: ['Custom referral code', 'Basic analytics'],
    requirements: { minDirectReferrals: 5 }
  },
  [ReferralTier.INFLUENCER]: {
    tier: ReferralTier.INFLUENCER,
    minRp: 5000,
    maxRp: 14999,
    miningBonus: 50,
    referralBonuses: { l1: 20, l2: 8, l3: 3 },
    networkCap: 50,
    benefits: ['Referral analytics', 'Priority support'],
    requirements: { minDirectReferrals: 10, minNetworkQuality: 0.6 }
  },
  [ReferralTier.LEADER]: {
    tier: ReferralTier.LEADER,
    minRp: 15000,
    maxRp: 49999,
    miningBonus: 100,
    referralBonuses: { l1: 25, l2: 10, l3: 5 },
    networkCap: 100,
    benefits: ['Exclusive events', 'Advanced analytics', 'Guild master privileges'],
    requirements: { minDirectReferrals: 20, minNetworkQuality: 0.7, kycRequired: true }
  },
  [ReferralTier.AMBASSADOR]: {
    tier: ReferralTier.AMBASSADOR,
    minRp: 50000,
    maxRp: Number.MAX_SAFE_INTEGER,
    miningBonus: 200,
    referralBonuses: { l1: 30, l2: 15, l3: 8 },
    networkCap: Number.MAX_SAFE_INTEGER,
    benefits: ['DAO governance', 'Maximum benefits', 'Ambassador privileges'],
    requirements: { minDirectReferrals: 50, minNetworkQuality: 0.8, kycRequired: true }
  }
};

// ============================================================================
// REFERRAL EVENTS AND ACTIVITIES
// ============================================================================

/**
 * Referral activity event
 */
export interface ReferralActivity {
  /** Activity ID */
  id: string;
  
  /** User who performed the activity */
  user: PublicKey;
  
  /** Activity type */
  type: ReferralActionType;
  
  /** RP earned from this activity */
  rpEarned: BN;
  
  /** Activity timestamp */
  timestamp: BN;
  
  /** Additional metadata */
  metadata: {
    [key: string]: any;
  };
  
  /** Network impact */
  networkImpact: {
    affectedUsers: PublicKey[];
    rpDistributed: BN;
    bonusMultiplier: number;
  };
}

/**
 * Referral reward distribution event
 */
export interface ReferralRewardEvent {
  /** Event ID */
  id: string;
  
  /** Referrer receiving the reward */
  referrer: PublicKey;
  
  /** User who triggered the reward */
  referredUser: PublicKey;
  
  /** Reward type */
  rewardType: 'rp' | 'fin' | 'xp';
  
  /** Reward amount */
  amount: BN;
  
  /** Referral level that triggered this reward */
  triggerLevel: ReferralLevel;
  
  /** Applied multipliers */
  multipliers: {
    tierMultiplier: number;
    qualityMultiplier: number;
    networkMultiplier: number;
  };
  
  /** Event timestamp */
  timestamp: BN;
  
  /** Transaction signature */
  signature: string;
}

// ============================================================================
// API REQUEST/RESPONSE TYPES
// ============================================================================

/**
 * Request to create a referral link
 */
export interface CreateReferralLinkRequest {
  /** Custom referral code (optional) */
  customCode?: string;
  
  /** Campaign identifier (optional) */
  campaign?: string;
  
  /** Expiration date (optional) */
  expiresAt?: Date;
}

/**
 * Response for referral link creation
 */
export interface CreateReferralLinkResponse {
  /** Generated referral code */
  referralCode: string;
  
  /** Full referral URL */
  referralUrl: string;
  
  /** Creation timestamp */
  createdAt: Date;
  
  /** Expiration timestamp */
  expiresAt: Date | null;
  
  /** QR code data URL */
  qrCode: string;
}

/**
 * Request to get referral analytics
 */
export interface GetReferralAnalyticsRequest {
  /** User's public key */
  user: string;
  
  /** Date range start */
  startDate?: Date;
  
  /** Date range end */
  endDate?: Date;
  
  /** Include network breakdown */
  includeNetwork?: boolean;
  
  /** Include historical data */
  includeHistory?: boolean;
}

/**
 * Response for referral analytics
 */
export interface GetReferralAnalyticsResponse {
  /** User's referral account */
  account: ReferralAccount;
  
  /** Referral statistics */
  stats: ReferralStats;
  
  /** Recent activities */
  recentActivities: ReferralActivity[];
  
  /** Network visualization data */
  networkData?: {
    nodes: ReferralNode[];
    connections: { from: string; to: string; level: number }[];
  };
  
  /** Historical performance */
  historicalData?: {
    date: Date;
    rpEarned: number;
    finEarned: number;
    newReferrals: number;
    activeReferrals: number;
  }[];
}

// ============================================================================
// REFERRAL VALIDATION TYPES
// ============================================================================

/**
 * Referral code validation result
 */
export interface ReferralCodeValidation {
  /** Is the code valid */
  isValid: boolean;
  
  /** Validation errors */
  errors: string[];
  
  /** Code owner information */
  owner?: {
    publicKey: PublicKey;
    tier: ReferralTier;
    isActive: boolean;
  };
  
  /** Code expiration info */
  expiration?: {
    isExpired: boolean;
    expiresAt: Date;
  };
  
  /** Usage statistics */
  usage?: {
    totalUses: number;
    maxUses: number | null;
    canUse: boolean;
  };
}

/**
 * Network quality validation parameters
 */
export interface NetworkQualityValidation {
  /** Total network size */
  totalSize: number;
  
  /** Active users count */
  activeUsers: number;
  
  /** Average activity level */
  averageActivity: number;
  
  /** Retention rate */
  retentionRate: number;
  
  /** Diversity score (prevents circular referrals) */
  diversityScore: number;
  
  /** Calculated quality score */
  qualityScore: number;
  
  /** Quality category */
  qualityCategory: 'poor' | 'fair' | 'good' | 'excellent';
}

// ============================================================================
// REFERRAL REGRESSION TYPES
// ============================================================================

/**
 * Exponential regression parameters for anti-whale mechanism
 */
export interface ReferralRegressionParams {
  /** Base regression coefficient */
  baseCoefficient: number;
  
  /** Network size factor */
  networkSizeFactor: number;
  
  /** Quality score factor */
  qualityScoreFactor: number;
  
  /** Time decay factor */
  timeDecayFactor: number;
  
  /** Maximum regression limit */
  maxRegressionLimit: number;
}

/**
 * Regression calculation result
 */
export interface RegressionCalculationResult {
  /** Original value before regression */
  originalValue: BN;
  
  /** Regression factor applied */
  regressionFactor: number;
  
  /** Final value after regression */
  finalValue: BN;
  
  /** Regression breakdown */
  breakdown: {
    networkSizeImpact: number;
    qualityScoreImpact: number;
    timeDecayImpact: number;
    totalRegression: number;
  };
}

// ============================================================================
// HELPER FUNCTIONS AND UTILITIES
// ============================================================================

/**
 * Utility function to determine referral tier based on RP
 */
export function getReferralTier(rp: number): ReferralTier {
  if (rp >= 50000) return ReferralTier.AMBASSADOR;
  if (rp >= 15000) return ReferralTier.LEADER;
  if (rp >= 5000) return ReferralTier.INFLUENCER;
  if (rp >= 1000) return ReferralTier.CONNECTOR;
  return ReferralTier.EXPLORER;
}

/**
 * Utility function to get tier configuration
 */
export function getTierConfig(tier: ReferralTier): ReferralTierConfig {
  return REFERRAL_TIER_CONFIG[tier];
}

/**
 * Utility function to validate referral code format
 */
export function isValidReferralCodeFormat(code: string): boolean {
  // Custom referral codes: 6-12 alphanumeric characters
  const customCodeRegex = /^[A-Za-z0-9]{6,12}$/;
  return customCodeRegex.test(code);
}

/**
 * Utility function to calculate network quality score
 */
export function calculateNetworkQuality(
  totalUsers: number,
  activeUsers: number,
  averageActivity: number
): number {
  if (totalUsers === 0) return 0;
  
  const activityRatio = activeUsers / totalUsers;
  const normalizedActivity = Math.min(averageActivity / 100, 1);
  
  return (activityRatio * 0.7) + (normalizedActivity * 0.3);
}

// ============================================================================
// CONSTANTS
// ============================================================================

/**
 * Referral system constants
 */
export const REFERRAL_CONSTANTS = {
  /** Maximum referral code length */
  MAX_REFERRAL_CODE_LENGTH: 12,
  
  /** Minimum referral code length */
  MIN_REFERRAL_CODE_LENGTH: 6,
  
  /** Maximum network depth */
  MAX_NETWORK_DEPTH: 3,
  
  /** Activity timeout (30 days in seconds) */
  ACTIVITY_TIMEOUT: 30 * 24 * 60 * 60,
  
  /** Default regression coefficient */
  DEFAULT_REGRESSION_COEFFICIENT: 0.0001,
  
  /** Maximum daily RP earning */
  MAX_DAILY_RP: 1000,
  
  /** RP to FIN conversion rates by tier */
  RP_TO_FIN_RATES: {
    [ReferralTier.EXPLORER]: 0.001,
    [ReferralTier.CONNECTOR]: 0.0012,
    [ReferralTier.INFLUENCER]: 0.0015,
    [ReferralTier.LEADER]: 0.002,
    [ReferralTier.AMBASSADOR]: 0.0025
  }
} as const;

// ============================================================================
// ERROR TYPES
// ============================================================================

/**
 * Referral system error types
 */
export enum ReferralErrorType {
  INVALID_REFERRAL_CODE = 'INVALID_REFERRAL_CODE',
  REFERRAL_CODE_EXPIRED = 'REFERRAL_CODE_EXPIRED',
  REFERRAL_CODE_USED = 'REFERRAL_CODE_USED',
  SELF_REFERRAL_NOT_ALLOWED = 'SELF_REFERRAL_NOT_ALLOWED',
  CIRCULAR_REFERRAL_DETECTED = 'CIRCULAR_REFERRAL_DETECTED',
  NETWORK_LIMIT_EXCEEDED = 'NETWORK_LIMIT_EXCEEDED',
  INSUFFICIENT_RP = 'INSUFFICIENT_RP',
  TIER_REQUIREMENTS_NOT_MET = 'TIER_REQUIREMENTS_NOT_MET',
  NETWORK_QUALITY_TOO_LOW = 'NETWORK_QUALITY_TOO_LOW',
  ACCOUNT_SUSPENDED = 'ACCOUNT_SUSPENDED'
}

/**
 * Referral system error
 */
export class ReferralError extends Error {
  constructor(
    public type: ReferralErrorType,
    message: string,
    public details?: any
  ) {
    super(message);
    this.name = 'ReferralError';
  }
}

// ============================================================================
// EXPORT ALL TYPES
// ============================================================================

export * from './mining';
export * from './staking';
export * from './nft';

// Re-export commonly used types
export type {
  ReferralAccount,
  ReferralNode,
  ReferralStats,
  RpCalculationResult,
  ReferralMiningBonus,
  ReferralTierConfig,
  ReferralActivity,
  NetworkQualityValidation
};
