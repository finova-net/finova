/**
 * Finova Network Mobile SDK - Mining & XP System Types
 *
 * @version 2.0.0
 * @author Finova Network Team
 * @description Contains all type definitions for the core Mining and XP (Experience Points) systems.
 */

import { MiningPhase, QualityTier } from './core';
import { SocialPlatform } from './social';

// MARK: - Mining System

export interface MiningSession {
  id: string;
  userId: string;
  startTime: number; // Unix timestamp
  endTime?: number; // Unix timestamp
  baseRate: string; // BigNumber string
  finalRate: string; // BigNumber string
  amountMined: string; // BigNumber string
  multipliers: MiningMultipliers;
  phase: MiningPhase;
  qualityScore: number;
  isActive: boolean;
  deviceFingerprint: string;
  location?: string;
}

export interface MiningMultipliers {
  pioneer: number;
  referral: number;
  security: number;
  xpLevel: number;
  rpTier: number;
  staking: number;
  nftCards: number;
  guild: number;
  quality: number;
  regression: number;
  total: number;
}

export interface MiningConfig {
  baseRate: string;
  maxDailyAmount: string;
  sessionDuration: number; // in seconds
  cooldownPeriod: number; // in seconds
  qualityThreshold: number;
  regressionFactor: number;
  phaseMultipliers: Record<MiningPhase, number>;
}

export interface MiningCalculation {
  baseAmount: string;
  multipliedAmount: string;
  finalAmount: string;
  breakdown: MultiplierBreakdown[];
  estimatedTime: number; // in seconds
  nextSessionAt: number; // Unix timestamp
}

export interface MultiplierBreakdown {
  source: string;
  multiplier: number;
  contribution: string;
  description: string;
  expiresAt?: number;
}

export interface MiningHistory {
  sessions: MiningSession[];
  totalMined: string;
  averageRate: string;
  bestSession: string;
  streakCount: number;
  totalSessions: number;
  efficiency: number; // percentage
}

// MARK: - XP (Experience Points) System

export interface XPActivity {
  id: string;
  userId: string;
  type: XPActivityType;
  platform: SocialPlatform;
  contentId?: string;
  baseXP: number;
  multiplier: number;
  finalXP: number;
  qualityTier: QualityTier;
  timestamp: number;
  isVerified: boolean;
}

export enum XPActivityType {
  POST_TEXT = 'post_text',
  POST_IMAGE = 'post_image',
  POST_VIDEO = 'post_video',
  STORY = 'story',
  COMMENT = 'comment',
  LIKE = 'like',
  SHARE = 'share',
  FOLLOW = 'follow',
  LOGIN = 'login',
  QUEST_COMPLETED = 'quest_completed',
  MILESTONE_REACHED = 'milestone_reached',
  VIRAL_CONTENT = 'viral_content'
}

export interface XPLevelInfo {
  level: number;
  tier: XPTier;
  currentXP: number;
  nextLevelXP: number;
  totalXP: number;
  progressPercentage: number;
  miningMultiplierBonus: number;
  unlockedFeatures: string[];
  badge?: string; // Badge ID
}

export enum XPTier {
  BRONZE = 'bronze',
  SILVER = 'silver',
  GOLD = 'gold',
  PLATINUM = 'platinum',
  DIAMOND = 'diamond',
  MYTHIC = 'mythic'
}

export interface XPLeaderboard {
  timeframe: 'daily' | 'weekly' | 'monthly' | 'all_time';
  entries: LeaderboardEntry[];
  userRank?: number;
  totalParticipants: number;
}

export interface LeaderboardEntry {
  rank: number;
  userId: string;
  username: string;
  avatar?: string;
  xp: number;
  level: number;
  change: number; // Rank change
  badges: string[]; // Badge IDs
}

export interface XPBooster {
  id: string;
  type: 'multiplier' | 'flat_bonus' | 'streak_saver';
  value: number;
  duration: number; // in seconds
  activatedAt: number;
  expiresAt: number;
  usesRemaining?: number;
  source: 'purchase' | 'reward' | 'event';
}
