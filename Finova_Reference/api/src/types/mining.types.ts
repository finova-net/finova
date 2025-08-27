/**
 * Finova Network - Mining System Type Definitions
 * Enterprise-grade TypeScript types for the complete mining ecosystem
 * Integrates XP, RP, and $FIN mining with exponential regression
 */

import { PublicKey } from '@solana/web3.js';

// ============================================================================
// BASE MINING TYPES
// ============================================================================

export interface MiningPhase {
  phase: 1 | 2 | 3 | 4;
  name: 'Finizen' | 'Growth' | 'Maturity' | 'Stability';
  userThreshold: number;
  baseRate: number; // $FIN per hour
  finizenBonus: number;
  maxDailyReward: number;
  startDate: Date;
  endDate?: Date;
}

export interface MiningRate {
  baseRate: number;
  finizenBonus: number;
  referralBonus: number;
  securityBonus: number;
  regressionFactor: number;
  xpMultiplier: number;
  rpMultiplier: number;
  qualityScore: number;
  finalHourlyRate: number;
  maxDailyReward: number;
}

export interface MiningSession {
  id: string;
  userId: string;
  walletAddress: PublicKey;
  sessionStart: Date;
  sessionEnd?: Date;
  duration: number; // minutes
  baseReward: number;
  bonusReward: number;
  totalReward: number;
  qualityScore: number;
  activityBoosts: MiningBoost[];
  cardEffects: SpecialCardEffect[];
  status: 'active' | 'completed' | 'paused' | 'cancelled';
  transactionHash?: string;
}

// ============================================================================
// MINING BOOST SYSTEM
// ============================================================================

export interface MiningBoost {
  id: string;
  type: 'daily_post' | 'daily_quest' | 'referral_kyc' | 'guild_participation' | 'special_card';
  multiplier: number; // 1.2 = +20%
  duration: number; // hours
  stackable: boolean;
  maxStacks: number;
  activatedAt: Date;
  expiresAt: Date;
  remainingUses?: number;
}

export interface ActivityBoostConfig {
  dailyPost: {
    multiplier: 1.2;
    duration: 24;
    maxStacks: 3;
    stackable: true;
  };
  dailyQuest: {
    multiplier: 1.5;
    duration: 12;
    maxStacks: 1;
    stackable: false;
  };
  referralKyc: {
    multiplier: 2.0;
    duration: 48;
    maxStacks: 5;
    stackable: true;
  };
  guildParticipation: {
    multiplier: 1.3;
    duration: 'event'; // Variable
    stackable: true;
  };
}

// ============================================================================
// REGRESSION & ANTI-WHALE MECHANICS
// ============================================================================

export interface RegressionData {
  totalUsers: number;
  userHoldings: number;
  networkQualityScore: number;
  finizenFactor: number; // 2.0 - (totalUsers / 1,000,000)
  whalePenalty: number; // e^(-0.001 × holdings)
  qualityMultiplier: number;
  finalRegressionFactor: number;
}

export interface WhaleProtection {
  holdingThreshold: number; // $FIN amount that triggers whale status
  penaltyRate: number; // Exponential decay rate
  maxPenalty: number; // Maximum penalty percentage
  progressiveTax: {
    tier1: { threshold: 10000; rate: 0.05 }; // 5% reduction
    tier2: { threshold: 50000; rate: 0.15 }; // 15% reduction  
    tier3: { threshold: 100000; rate: 0.30 }; // 30% reduction
    tier4: { threshold: 500000; rate: 0.50 }; // 50% reduction
  };
}

// ============================================================================
// QUALITY ASSESSMENT SYSTEM
// ============================================================================

export interface QualityMetrics {
  originality: number; // 0-1 score
  engagement: number; // Predicted engagement score
  platformRelevance: number; // Platform-specific relevance
  brandSafety: number; // Brand safety score
  humanGenerated: number; // AI vs human content probability
  networkEffect: number; // Viral potential score
  finalQualityScore: number; // Weighted composite (0.5x - 2.0x)
}

export interface ContentAnalysis {
  contentId: string;
  contentType: 'text' | 'image' | 'video' | 'mixed';
  platform: SocialPlatform;
  aiAnalysis: {
    qualityMetrics: QualityMetrics;
    suspiciousFlags: string[];
    confidenceScore: number;
    processingTime: number;
  };
  humanValidation?: {
    overrideScore?: number;
    validatorId: string;
    notes?: string;
    timestamp: Date;
  };
}

// ============================================================================
// INTEGRATED XP SYSTEM
// ============================================================================

export interface XPMiningIntegration {
  currentLevel: number;
  totalXP: number;
  levelMultiplier: number; // 1.0x - 5.0x based on level
  dailyXPGained: number;
  xpFromMining: number; // XP gained from mining activities
  miningFromXP: number; // Mining bonus from XP level
  streakBonus: number; // Consecutive day streak multiplier
  levelUpRewards: LevelUpReward[];
}

export interface LevelUpReward {
  level: number;
  finReward: number; // Bonus $FIN for level up
  miningBoost: number; // Permanent mining rate increase
  specialUnlocks: string[]; // Features unlocked at this level
  nftReward?: string; // Special NFT ID if applicable
}

// ============================================================================
// REFERRAL POINTS (RP) INTEGRATION
// ============================================================================

export interface RPMiningIntegration {
  currentRP: number;
  rpTier: 'Explorer' | 'Connector' | 'Influencer' | 'Leader' | 'Ambassador';
  tierMultiplier: number; // 1.0x - 3.0x based on tier
  networkSize: {
    level1: number; // Direct referrals
    level2: number; // Second level
    level3: number; // Third level
    total: number;
  };
  networkQuality: {
    activeRatio: number; // Active users / total referrals
    averageLevel: number; // Average XP level of network
    retentionRate: number; // 30-day retention percentage
    qualityScore: number; // Composite quality metric
  };
  dailyRPEarned: number;
  miningBoostFromRP: number;
}

// ============================================================================
// SPECIAL CARDS & NFT EFFECTS
// ============================================================================

export interface SpecialCardEffect {
  cardId: string;
  cardName: string;
  cardType: 'mining_boost' | 'xp_accelerator' | 'referral_power' | 'utility';
  rarity: 'common' | 'uncommon' | 'rare' | 'epic' | 'legendary';
  effect: {
    miningMultiplier?: number;
    xpMultiplier?: number;
    rpMultiplier?: number;
    duration: number; // hours, or -1 for permanent
    stackable: boolean;
  };
  activatedAt: Date;
  expiresAt?: Date;
  usesRemaining?: number; // For limited-use cards
  synergy?: CardSynergy;
}

export interface CardSynergy {
  activeCards: string[]; // Other active card IDs
  synergyMultiplier: number; // Additional multiplier from card combination
  categoryBonus: number; // Same category bonus
  rarityBonus: number; // Mixed rarity bonus
}

// ============================================================================
// MINING STATISTICS & ANALYTICS
// ============================================================================

export interface MiningStats {
  user: {
    totalMined: number;
    dailyAverage: number;
    currentStreak: number;
    bestStreak: number;
    totalSessions: number;
    averageSessionDuration: number;
    efficiencyScore: number; // Mining rate vs theoretical max
  };
  network: {
    globalMiningRate: number;
    totalMinersActive: number;
    averageHoldings: number;
    topMiners: TopMinerStats[];
    phaseProgress: number; // Progress to next phase (0-1)
  };
  rewards: {
    pendingRewards: number;
    claimableRewards: number;
    totalClaimed: number;
    estimatedDailyEarnings: number;
    projectedMonthly: number;
  };
}

export interface TopMinerStats {
  rank: number;
  userId: string; // Anonymized
  dailyMining: number;
  level: number;
  rpTier: string;
  country?: string; // Optional for leaderboards
}

// ============================================================================
// SECURITY & ANTI-BOT TYPES
// ============================================================================

export interface SecurityAssessment {
  humanProbability: number; // 0-1 probability of being human
  botRiskScore: number; // 0-1 risk score
  suspiciousActivities: SuspiciousActivity[];
  biometricConsistency: number;
  behaviorPatterns: BehaviorPattern[];
  deviceAuthenticity: DeviceFingerprint;
  networkAnalysis: NetworkRiskAnalysis;
  lastAssessment: Date;
  riskLevel: 'low' | 'medium' | 'high' | 'critical';
}

export interface SuspiciousActivity {
  type: 'irregular_timing' | 'duplicate_content' | 'velocity_anomaly' | 'network_clustering';
  severity: number; // 0-1
  timestamp: Date;
  details: Record<string, any>;
  resolved: boolean;
}

export interface BehaviorPattern {
  pattern: 'click_speed' | 'session_rhythm' | 'content_timing' | 'interaction_variance';
  normalcy: number; // 0-1, higher is more normal
  deviationScore: number;
  sampleSize: number;
  confidence: number;
}

export interface DeviceFingerprint {
  deviceId: string;
  platform: 'ios' | 'android' | 'web';
  screenResolution: string;
  timezone: string;
  uniqueFeatures: string[];
  consistencyScore: number; // How consistent device appears over time
  riskFlags: string[];
}

export interface NetworkRiskAnalysis {
  connectionClustering: number; // How clustered user's connections are
  referralVelocity: number; // Rate of referral acquisition
  geographicDistribution: string[]; // Countries of referral network
  activityCorrelation: number; // How similar activity patterns are in network
  riskScore: number;
}

// ============================================================================
// STAKING INTEGRATION
// ============================================================================

export interface StakingMiningBonus {
  stakedAmount: number; // $FIN staked
  stakingTier: 'bronze' | 'silver' | 'gold' | 'platinum' | 'diamond';
  miningBoost: number; // 1.2x - 2.0x multiplier
  xpMultiplier: number; // Additional XP bonus
  rpBonus: number; // Additional RP bonus
  loyaltyBonus: number; // Time-based bonus
  activityBonus: number; // Engagement-based bonus
  totalMultiplier: number; // Combined effect
  stakingDuration: number; // Days staked
}

// ============================================================================
// ECONOMIC BALANCING TYPES
// ============================================================================

export interface EconomicMetrics {
  totalSupply: number;
  circulatingSupply: number;
  dailyInflation: number;
  burnRate: number;
  stakingRatio: number; // Percentage of supply staked
  liquidityMetrics: {
    dexLiquidity: number;
    tradingVolume24h: number;
    priceImpact: number;
  };
  distributionMetrics: {
    giniCoefficient: number; // Wealth distribution inequality
    top10Holders: number; // Percentage held by top 10%
    activeHolders: number;
    averageHolding: number;
  };
}

export interface BalancingParameters {
  maxDailyInflation: number; // Maximum daily token creation
  targetStakingRatio: number; // Optimal staking percentage
  whaleThreshold: number; // Holdings that trigger whale mechanics
  qualityThreshold: number; // Minimum quality score for full rewards
  networkEffectCap: number; // Maximum network size bonus
  emergencyBrakes: {
    inflationPause: boolean;
    miningPause: boolean;
    stakingPause: boolean;
    lastActivated?: Date;
    reason?: string;
  };
}

// ============================================================================
// API RESPONSE TYPES
// ============================================================================

export interface MiningStatusResponse {
  isActive: boolean;
  currentRate: MiningRate;
  session?: MiningSession;
  stats: MiningStats;
  boosts: MiningBoost[];
  nextRewards: {
    estimatedHourly: number;
    timeToNextClaim: number;
    pendingAmount: number;
  };
  phase: MiningPhase;
  security: SecurityAssessment;
}

export interface StartMiningRequest {
  userId: string;
  walletAddress: string;
  deviceFingerprint: DeviceFingerprint;
  location?: {
    country: string;
    timezone: string;
  };
}

export interface ClaimRewardsRequest {
  userId: string;
  sessionId: string;
  walletAddress: string;
  securityChallenge?: string; // For high-value claims
}

export interface ClaimRewardsResponse {
  success: boolean;
  claimedAmount: number;
  transactionHash: string;
  newBalance: number;
  bonusRewards?: {
    xpGained: number;
    rpGained: number;
    specialRewards?: string[];
  };
  nextClaimEligible: Date;
}

// ============================================================================
// SOCIAL PLATFORM INTEGRATION
// ============================================================================

export type SocialPlatform = 'instagram' | 'tiktok' | 'youtube' | 'facebook' | 'twitter' | 'linkedin';

export interface PlatformIntegration {
  platform: SocialPlatform;
  userId: string;
  platformUserId: string;
  accessToken: string; // Encrypted
  refreshToken?: string; // Encrypted
  permissions: string[];
  connected: boolean;
  lastSync: Date;
  syncErrors?: string[];
  miningMultiplier: number; // Platform-specific bonus
}

export interface SocialActivity {
  id: string;
  platform: SocialPlatform;
  activityType: 'post' | 'comment' | 'like' | 'share' | 'follow';
  contentId: string;
  contentUrl?: string;
  engagement: {
    likes: number;
    comments: number;
    shares: number;
    views?: number;
  };
  qualityAnalysis: ContentAnalysis;
  xpEarned: number;
  finEarned: number;
  timestamp: Date;
  verified: boolean; // Platform verification status
}

// ============================================================================
// GOVERNANCE & DAO TYPES
// ============================================================================

export interface GovernanceIntegration {
  votingPower: number; // Based on staked $FIN + XP + RP
  participationScore: number; // Historical governance participation
  proposalsVoted: number;
  proposalsCreated: number;
  delegatedVotes: number; // Votes delegated to this user
  delegatingTo?: string; // User ID if delegating votes
  governanceLevel: 'citizen' | 'delegate' | 'councilor' | 'senator';
  specialRoles: string[]; // Special governance roles
}

// ============================================================================
// ERROR TYPES
// ============================================================================

export interface MiningError {
  code: string;
  message: string;
  type: 'validation' | 'security' | 'rate_limit' | 'technical' | 'economic';
  details?: Record<string, any>;
  retryable: boolean;
  timestamp: Date;
}

// ============================================================================
// UTILITY TYPES
// ============================================================================

export type MiningCalculationInput = {
  userId: string;
  totalUsers: number;
  userHoldings: number;
  xpLevel: number;
  rpTier: string;
  networkQuality: number;
  kycVerified: boolean;
  activeReferrals: number;
  qualityScore: number;
  activeBoosts: MiningBoost[];
  activeCards: SpecialCardEffect[];
  stakingBonus: StakingMiningBonus;
};

export type MiningCalculationResult = {
  baseRate: number;
  multipliers: {
    finizen: number;
    referral: number;
    security: number;
    xp: number;
    rp: number;
    quality: number;
    staking: number;
    cards: number;
  };
  regression: {
    factor: number;
    reason: string;
  };
  finalHourlyRate: number;
  dailyCap: number;
  estimatedDaily: number;
};

// ============================================================================
// EXPORT ALL TYPES
// ============================================================================

export * from './user.types';
export * from './social.types';
export * from './blockchain.types';
