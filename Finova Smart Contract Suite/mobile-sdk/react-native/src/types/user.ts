/**
 * Finova Network Mobile SDK - User & Authentication Types
 *
 * @version 2.0.0
 * @author Finova Network Team
 * @description Consolidated and enterprise-grade type definitions for User profiles,
 * authentication, security, preferences, stats, and KYC processes.
 */

import { BaseEntity } from './';
import { MiningStats } from './mining';
import { XPStats } from './xp';
import { ReferralStats } from './referral';
import { NFTStats } from './nft';
import { StakingStats } from './staking';
import { GuildStats } from './guild';
import { SocialStats, SocialLink } from './social';

// MARK: - Core User Interface

export interface User extends BaseEntity {
  publicKey: string;
  username: string;
  email?: string;
  phone?: string;
  profile: UserProfile;
  stats: UserStats;
  preferences: UserPreferences;
  security: UserSecurity;
  kycStatus: KYCStatus;
  status: 'active' | 'inactive' | 'suspended' | 'banned';
  lastActiveAt: Date;
}

// MARK: - User Profile

export interface UserProfile {
  displayName: string;
  avatarUrl?: string;
  bio?: string;
  location?: string;
  website?: string;
  socialLinks: SocialLink[];
  achievements: string[]; // IDs of unlocked achievements
  badges: string[]; // IDs of equipped badges
  tier: UserTier;
  reputationScore: number;
}

export type UserTier = 'Bronze' | 'Silver' | 'Gold' | 'Platinum' | 'Diamond' | 'Mythic';

// MARK: - User Statistics

export interface UserStats {
  mining: MiningStats;
  xp: XPStats;
  referral: ReferralStats;
  social: SocialStats;
  nft: NFTStats;
  staking: StakingStats;
  guild: GuildStats;
}

// MARK: - User Security & KYC

export type KYCStatus = 'not_submitted' | 'pending' | 'approved' | 'rejected' | 'expired';

export interface UserSecurity {
  twoFactorEnabled: boolean;
  biometricEnabled: boolean;
  sessionTimeout: number; // in seconds
  trustedDevices: TrustedDevice[];
  securityLevel: 'basic' | 'standard' | 'high';
  loginHistory: LoginHistoryEntry[];
}

export interface TrustedDevice {
  id: string;
  name: string;
  platform: 'ios' | 'android' | 'web';
  lastUsedAt: Date;
  ipAddress: string;
}

export interface LoginHistoryEntry {
  id: string;
  timestamp: Date;
  ipAddress: string;
  location: string;
  device: string;
  wasSuccessful: boolean;
}

// MARK: - User Preferences

export interface UserPreferences {
  notifications: NotificationPreferences;
  privacy: PrivacyPreferences;
  appearance: AppearancePreferences;
  mining: MiningPreferences;
}

export interface NotificationPreferences {
  enabled: boolean;
  types: Record<'mining' | 'xp' | 'referral' | 'social' | 'guild' | 'security', boolean>;
  quietHours?: { start: string; end: string; }; // "HH:mm"
}

export interface PrivacyPreferences {
  profileVisibility: 'public' | 'friends' | 'private';
  showStats: boolean;
  showReferralNetwork: boolean;
  allowDataAnalytics: boolean;
}

export interface AppearancePreferences {
  theme: 'light' | 'dark' | 'auto';
  language: string;
  currency: 'USD' | 'IDR' | 'EUR';
}

export interface MiningPreferences {
  autoStart: boolean;
  backgroundMining: boolean;
  lowBatteryMode: boolean;
}
