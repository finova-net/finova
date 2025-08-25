/**
 * Finova Network Mobile SDK - Referral (RP) System Types
 *
 * @version 2.0.0
 * @author Finova Network Team
 * @description Defines the structures for the Referral Points (RP) system,
 * including network, tiers, and rewards.
 */

import { BaseEntity } from './';

// MARK: - Referral Tiers & Network

export enum RPTier {
  EXPLORER = 'Explorer',       // 0-999 RP
  CONNECTOR = 'Connector',     // 1K-4.999K RP
  INFLUENCER = 'Influencer',   // 5K-14.999K RP
  LEADER = 'Leader',           // 15K-49.999K RP
  AMBASSADOR = 'Ambassador'    // 50K+ RP
}

export interface ReferralTierInfo {
  name: RPTier;
  level: number;
  minRP: number;
  maxRP?: number;
  miningBonus: number;
  referralBonus: number;
}

export interface ReferralNetwork {
  userId: string;
  referralCode: string;
  totalRP: number;
  tier: RPTier;
  directReferrals: ReferralMember[];
  level2Referrals: ReferralMember[];
  level3Referrals: ReferralMember[];
  stats: ReferralNetworkStats;
}

export interface ReferralMember {
  userId: string;
  username: string;
  avatarUrl?: string;
  level: number;
  joinedAt: Date;
  isActive: boolean;
  lastActivityAt: Date;
  contribution: ReferralContribution;
  qualityScore: number; // 0-100
}

// MARK: - Contributions & Stats

export interface ReferralContribution {
  finGenerated: number;
  xpGenerated: number;
  rpGenerated: number;
}

export interface ReferralNetworkStats {
  totalSize: number;
  activeSize: number;
  retentionRate: number; // percentage
  growthRate24h: number; // percentage
  averageLevel: number;
  networkQualityScore: number; // 0-100
}

// MARK: - Rewards & Calculation

export interface ReferralReward {
  id: string;
  type: 'signup' | 'kyc_completion' | 'first_mining' | 'activity_bonus';
  fromUserId: string;
  amount: number;
  currency: 'FIN' | 'XP' | 'RP';
  networkLevel: 1 | 2 | 3;
  timestamp: Date;
  status: 'pending' | 'confirmed' | 'rejected';
}

export interface RPCalculation {
  directPoints: number;
  networkPoints: number;
  qualityBonus: number;
  regressionFactor: number;
  totalRP: number;
  breakdown: { source: string; points: number }[];
}

export interface ReferralAnalytics {
  totalInvites: number;
  conversionRate: number;
  topPerformers: { userId: string; contribution: number }[];
  earningsProjection: { next30days: number };
}
