/**
 * Finova Network Mobile SDK - Common Types
 *
 * @version 2.0.0
 * @author Finova Network Team
 * @description Common types and interfaces used across the Finova Network SDK.
 * This file has been consolidated and optimized for enterprise-grade use.
 */

// MARK: - Error and Response Interfaces

export interface FinovaError extends Error {
  code: string;
  category: 'network' | 'blockchain' | 'validation' | 'auth' | 'business' | 'system';
  severity: 'low' | 'medium' | 'high' | 'critical';
  retryable: boolean;
  context?: Record<string, any>;
  timestamp: Date;
}

export interface ApiResponse<T = any> {
  success: boolean;
  data?: T;
  error?: FinovaError;
  message?: string;
  metadata?: ResponseMetadata;
}

export interface PaginatedResponse<T> extends ApiResponse<T[]> {
  pagination: PaginationInfo;
}

export interface ResponseMetadata {
  timestamp: Date;
  requestId: string;
  version: string;
  rateLimit?: RateLimitInfo;
}

export interface PaginationInfo {
  page: number;
  limit: number;
  totalItems: number;
  totalPages: number;
  hasNextPage: boolean;
  hasPrevPage: boolean;
}

// MARK: - Core System Interfaces

export interface NetworkHealth {
  status: 'healthy' | 'degraded' | 'down';
  uptime: number; // Percentage
  responseTime: number; // Milliseconds
  errorRate: number; // Percentage
  lastCheck: Date;
}

export interface TimeWindow {
  start: Date;
  end: Date;
  duration: number; // Milliseconds
  type: 'daily' | 'weekly' | 'monthly' | 'custom';
}

// MARK: - Activity and Logging

export interface ActivityLog {
  id: string;
  userId: string;
  type: ActivityType;
  platform?: string;
  description: string;
  metadata: Record<string, any>;
  rewards?: ActivityReward[];
  timestamp: Date;
}

export type ActivityType =
  | 'mining_started' | 'mining_stopped' | 'mining_reward'
  | 'xp_gained' | 'level_up' | 'badge_earned'
  | 'referral_joined' | 'referral_reward'
  | 'social_post' | 'social_engagement' | 'content_viral'
  | 'nft_received' | 'nft_used' | 'nft_traded'
  | 'card_used' | 'card_purchased'
  | 'guild_joined' | 'guild_event'
  | 'staking_started' | 'staking_reward'
  | 'governance_vote' | 'proposal_created';

export interface ActivityReward {
  type: 'fin' | 'xp' | 'rp';
  amount: number;
  multiplier?: number;
  source: string;
}

// MARK: - Notifications

export interface Notification {
  id: string;
  userId: string;
  type: NotificationType;
  title: string;
  message: string;
  data?: Record<string, any>;
  read: boolean;
  priority: 'low' | 'normal' | 'high' | 'urgent';
  category: 'mining' | 'xp' | 'referral' | 'social' | 'nft' | 'system' | 'guild' | 'security';
  actionUrl?: string;
  imageUrl?: string;
  createdAt: Date;
  readAt?: Date;
  expiresAt?: Date;
}

export type NotificationType =
  | 'mining_reward' | 'mining_milestone'
  | 'xp_gained' | 'level_up' | 'badge_earned'
  | 'referral_joined' | 'referral_milestone'
  | 'social_viral' | 'social_milestone'
  | 'nft_received' | 'nft_rare_found'
  | 'card_expired' | 'card_available'
  | 'guild_invitation' | 'guild_event'
  | 'system_maintenance' | 'system_update'
  | 'security_alert';

// MARK: - Utility Types

export type Nullable<T> = T | null;
export type Optional<T> = T | undefined;
export type DeepPartial<T> = {
  [P in keyof T]?: T[P] extends object ? DeepPartial<T[P]> : T[P];
};

export type EventCallback<T = any> = (data: T) => void | Promise<void>;
