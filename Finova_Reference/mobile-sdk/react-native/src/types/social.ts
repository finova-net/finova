/**
 * Finova Network Mobile SDK - Social Integration Types
 *
 * @version 2.0.0
 * @author Finova Network Team
 * @description Defines the structures for social media integration, content analysis,
 * and user engagement within the Finova ecosystem.
 */

import { BaseEntity } from './';
import { QualityTier } from './core';

// MARK: - Social Platform & Account

export enum SocialPlatform {
  INSTAGRAM = 'instagram',
  TIKTOK = 'tiktok',
  YOUTUBE = 'youtube',
  FACEBOOK = 'facebook',
  TWITTER = 'twitter',
  LINKEDIN = 'linkedin'
}

export interface SocialConnection {
  platform: SocialPlatform;
  userId: string; // Platform-specific user ID
  username: string;
  displayName: string;
  avatarUrl?: string;
  followers: number;
  isConnected: boolean;
  connectedAt: Date;
  lastSyncAt?: Date;
  permissions: string[];
}

// MARK: - Social Content & Engagement

export interface SocialPost extends BaseEntity {
  userId: string;
  platform: SocialPlatform;
  platformPostId: string;
  content: PostContent;
  engagement: EngagementMetrics;
  qualityScore: number;
  xpEarned: number;
  finEarned: number;
  isVerified: boolean; // Verified by Finova's system
  status: 'pending' | 'processed' | 'rejected';
}

export interface PostContent {
  type: 'image' | 'video' | 'text' | 'story' | 'reel';
  text?: string;
  mediaUrls: string[];
  hashtags: string[];
  mentions: string[];
  location?: string;
}

export interface EngagementMetrics {
  views: number;
  likes: number;
  comments: number;
  shares: number;
  saves: number;
  engagementRate: number;
  isViral: boolean;
}

// MARK: - Content Quality Analysis

export interface QualityAnalysis {
  overallScore: number; // 0.5 - 2.0
  tier: QualityTier;
  originality: number;
  engagement: number;
  brandSafety: number;
  humanGeneratedScore: number; // Probability of being human-generated
  factors: QualityFactor[];
  aiConfidence: number;
}

export interface QualityFactor {
  name: 'content' | 'timing' | 'context' | 'engagement_pattern';
  score: number;
  weight: number;
  description: string;
}

// MARK: - Social Challenges & Trends

export interface SocialChallenge extends BaseEntity {
  title: string;
  description: string;
  hashtag: string;
  platforms: SocialPlatform[];
  startDate: Date;
  endDate: Date;
  rewards: ChallengeReward[];
  participantCount: number;
  status: 'upcoming' | 'active' | 'completed';
}

export interface ChallengeReward {
  rank: number | [number, number]; // e.g., 1 or [2, 10]
  type: 'fin' | 'xp' | 'nft' | 'badge';
  amount?: number;
  itemId?: string; // For NFT or Badge
  description: string;
}

export interface TrendingTopic {
  topic: string;
  volume: number;
  growthRate: number; // Percentage
  platforms: SocialPlatform[];
}
