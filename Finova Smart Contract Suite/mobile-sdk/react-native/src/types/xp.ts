/**
 * Finova Network Mobile SDK - Experience Points (XP) System Types
 *
 * @version 2.0.0
 * @author Finova Network Team
 * @description Defines all structures for the XP system, including levels, tiers,
 * activities, streaks, achievements, and leaderboards.
 */

import { BaseEntity } from './';
import { SocialPlatform } from './social';

// MARK: - Core XP System & Levels

export interface XPSystem {
  userId: string;
  totalXP: number;
  level: XPLevel;
  tier: XPTier;
  streak: XPStreak;
  statistics: XPStatistics;
}

export interface XPLevel {
  current: number;
  xpInLevel: number;
  xpForNextLevel: number;
  progressPercentage: number;
  benefits: LevelBenefit[];
  unlockedFeatures: string[];
}

export enum XPTier {
  BRONZE = 'Bronze',
  SILVER = 'Silver',
  GOLD = 'Gold',
  PLATINUM = 'Platinum',
  DIAMOND = 'Diamond',
  MYTHIC = 'Mythic'
}

export interface LevelBenefit {
  type: 'mining_multiplier' | 'referral_bonus' | 'staking_apy_boost';
  value: number;
  description: string;
}

// MARK: - XP Activities & Sources

export interface XPGainEvent extends BaseEntity {
  userId: string;
  activityType: XPActivityType;
  baseXP: number;
  finalXP: number;
  multipliers: AppliedMultiplier[];
  platform?: SocialPlatform;
  qualityScore?: number;
}

export enum XPActivityType {
  SOCIAL_POST = 'social_post',
  SOCIAL_ENGAGEMENT = 'social_engagement',
  CONTENT_VIRAL = 'content_viral',
  MILESTONE_REACHED = 'milestone_reached',
  QUEST_COMPLETED = 'quest_completed',
  DAILY_LOGIN = 'daily_login',
}

export interface AppliedMultiplier {
  type: 'quality' | 'streak' | 'level' | 'card' | 'event';
  value: number;
}

// MARK: - Streaks & Achievements

export interface XPStreak {
  current: number;
  longest: number;
  lastActivityAt: Date;
  bonusMultiplier: number;
}

export interface XPAchievement extends BaseEntity {
  name: string;
  description: string;
  category: 'social' | 'consistency' | 'quality' | 'milestone';
  tier: 'bronze' | 'silver' | 'gold';
  xpReward: number;
  progress: number;
  target: number;
  isCompleted: boolean;
  completedAt?: Date;
}

// MARK: - Analytics & Leaderboards

export interface XPStatistics {
  dailyAverage: number;
  weeklyGrowth: number; // percentage
  sourceBreakdown: Record<XPActivityType, number>; // percentage
  platformBreakdown: Record<SocialPlatform, number>; // percentage
}

export interface XPLeaderboard {
  timeframe: 'daily' | 'weekly' | 'all_time';
  entries: {
    rank: number;
    userId: string;
    username: string;
    xp: number;
    level: number;
  }[];
  userRank?: number;
}
