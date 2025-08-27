/**
 * Finova Network Mobile SDK - Guild System Types
 *
 * @version 2.0.0
 * @author Finova Network Team
 * @description Defines all structures related to the Guild (community) features.
 */

import { BaseEntity } from './';

// MARK: - Core Guild & Member Types

export type GuildRole = 'member' | 'officer' | 'leader' | 'founder';
export type GuildPermission = 'invite' | 'kick' | 'promote' | 'manage_competitions' | 'manage_treasury';

export interface Guild extends BaseEntity {
  name: string;
  description: string;
  logoUrl: string;
  bannerUrl?: string;
  tags: string[];
  privacy: 'public' | 'private' | 'invite_only';
  members: GuildMember[];
  leadership: {
    founderId: string;
    leaderId: string;
    officerIds: string[];
  };
  stats: GuildStats;
  treasury: GuildTreasury;
  settings: GuildSettings;
}

export interface GuildMember {
  userId: string;
  username: string;
  avatarUrl?: string;
  role: GuildRole;
  contribution: number; // Total XP contributed
  joinedAt: Date;
  lastActiveAt: Date;
  status: 'active' | 'inactive';
}

// MARK: - Guild Stats & Treasury

export interface GuildStats {
  level: number;
  totalXP: number;
  memberCount: number;
  maxMembers: number;
  globalRank: number;
  performance: {
    totalMined: number;
    competitionsWon: number;
  };
}

export interface GuildTreasury {
  balance: number; // in FIN
  income24h: number;
  expenses24h: number;
  assets: { nftId: string }[]; // NFTs owned by the guild
}

// MARK: - Competitions & Activities

export type CompetitionType = 'mining_marathon' | 'xp_rush' | 'referral_drive' | 'social_blitz';
export type CompetitionStatus = 'upcoming' | 'active' | 'completed';

export interface GuildCompetition extends BaseEntity {
  name: string;
  type: CompetitionType;
  description: string;
  startDate: Date;
  endDate: Date;
  rewards: CompetitionReward[];
  leaderboard: { userId: string; score: number }[];
  status: CompetitionStatus;
}

export interface CompetitionReward {
  rank: number;
  type: 'fin_pool' | 'nft' | 'xp_bonus';
  value: number | string; // Amount or Item ID
}

// MARK: - Guild Settings & Requirements

export interface GuildSettings {
  joinRequirements: {
    minLevel?: number;
    minContribution?: number;
    requiresApproval: boolean;
  };
  permissions: Record<GuildRole, GuildPermission[]>;
}
