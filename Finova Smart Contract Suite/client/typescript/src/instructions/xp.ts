// client/typescript/src/instructions/xp.ts

import {
  PublicKey,
  TransactionInstruction,
  SystemProgram,
  SYSVAR_CLOCK_PUBKEY,
} from '@solana/web3.js';
import { BN } from '@coral-xyz/anchor';
import { 
  FINOVA_CORE_PROGRAM_ID,
  XP_SEED,
  USER_SEED,
  NETWORK_SEED,
  ACTIVE_EFFECTS_SEED 
} from '../constants';

/**
 * XP Activity Types - matching the whitepaper specifications
 */
export enum XPActivityType {
  // Content Creation
  OriginalTextPost = 0,
  PhotoImagePost = 1,
  VideoContent = 2,
  StoryStatus = 3,
  
  // Engagement
  MeaningfulComment = 4,
  LikeReact = 5,
  ShareRepost = 6,
  FollowSubscribe = 7,
  
  // Special Actions
  FirstDailyLogin = 8,
  CompleteDailyQuest = 9,
  AchieveMilestone = 10,
  ViralContent = 11,
  
  // Platform Integration
  InstagramPost = 12,
  TikTokPost = 13,
  YouTubePost = 14,
  FacebookPost = 15,
  TwitterPost = 16,
  
  // Guild Activities
  GuildParticipation = 17,
  GuildLeadership = 18,
  GuildEvent = 19,
  
  // NFT Activities
  UseSpecialCard = 20,
  NFTTrade = 21,
  NFTCreate = 22,
}

/**
 * Social Platform Types for platform-specific multipliers
 */
export enum SocialPlatform {
  TikTok = 0,     // 1.3x multiplier
  Instagram = 1,  // 1.2x multiplier
  YouTube = 2,    // 1.4x multiplier
  Facebook = 3,   // 1.1x multiplier
  Twitter = 4,    // 1.2x multiplier
  App = 5,        // 1.0x multiplier
}

/**
 * XP Update Parameters
 */
export interface XPUpdateParams {
  /** Type of activity performed */
  activityType: XPActivityType;
  /** Platform where activity occurred */
  platform: SocialPlatform;
  /** Content quality score (0.5 - 2.0) */
  qualityScore: number;
  /** Engagement metrics for viral bonus calculation */
  engagementMetrics?: {
    views: number;
    likes: number;
    comments: number;
    shares: number;
  };
  /** Additional metadata for the activity */
  metadata?: string;
}

/**
 * Level Progression Parameters
 */
export interface LevelProgressionParams {
  /** Current XP amount */
  currentXP: number;
  /** Target level to check progression */
  targetLevel?: number;
}

/**
 * Streak Management Parameters
 */
export interface StreakManagementParams {
  /** Action type for streak management */
  action: 'maintain' | 'break' | 'restore';
  /** Days to extend streak (for restore action) */
  extensionDays?: number;
}

/**
 * Daily Quest Parameters
 */
export interface DailyQuestParams {
  /** Quest ID */
  questId: number;
  /** Quest completion status */
  completed: boolean;
  /** Bonus XP for completion */
  bonusXP?: number;
}

/**
 * Creates an instruction to update user XP
 */
export async function createUpdateXPInstruction(
  user: PublicKey,
  params: XPUpdateParams
): Promise<TransactionInstruction> {
  const [userStatePDA] = PublicKey.findProgramAddressSync(
    [Buffer.from(USER_SEED), user.toBuffer()],
    FINOVA_CORE_PROGRAM_ID
  );

  const [xpStatePDA] = PublicKey.findProgramAddressSync(
    [Buffer.from(XP_SEED), user.toBuffer()],
    FINOVA_CORE_PROGRAM_ID
  );

  const [networkStatePDA] = PublicKey.findProgramAddressSync(
    [Buffer.from(NETWORK_SEED)],
    FINOVA_CORE_PROGRAM_ID
  );

  const [activeEffectsPDA] = PublicKey.findProgramAddressSync(
    [Buffer.from(ACTIVE_EFFECTS_SEED), user.toBuffer()],
    FINOVA_CORE_PROGRAM_ID
  );

  // Prepare instruction data
  const data = Buffer.alloc(256);
  let offset = 0;

  // Instruction discriminator for update_xp
  data.writeUInt8(3, offset); // Assuming update_xp is instruction index 3
  offset += 1;

  // Activity type
  data.writeUInt8(params.activityType, offset);
  offset += 1;

  // Platform
  data.writeUInt8(params.platform, offset);
  offset += 1;

  // Quality score (scaled by 1000 for precision)
  const qualityScoreScaled = Math.floor(params.qualityScore * 1000);
  data.writeUInt32LE(qualityScoreScaled, offset);
  offset += 4;

  // Engagement metrics (optional)
  if (params.engagementMetrics) {
    data.writeUInt8(1, offset); // Has engagement metrics
    offset += 1;
    
    data.writeUInt32LE(params.engagementMetrics.views, offset);
    offset += 4;
    data.writeUInt32LE(params.engagementMetrics.likes, offset);
    offset += 4;
    data.writeUInt32LE(params.engagementMetrics.comments, offset);
    offset += 4;
    data.writeUInt32LE(params.engagementMetrics.shares, offset);
    offset += 4;
  } else {
    data.writeUInt8(0, offset); // No engagement metrics
    offset += 1;
  }

  // Metadata length and content
  const metadataBytes = params.metadata ? Buffer.from(params.metadata, 'utf-8') : Buffer.alloc(0);
  data.writeUInt16LE(metadataBytes.length, offset);
  offset += 2;
  metadataBytes.copy(data, offset);

  return new TransactionInstruction({
    keys: [
      { pubkey: user, isSigner: true, isWritable: false },
      { pubkey: userStatePDA, isSigner: false, isWritable: true },
      { pubkey: xpStatePDA, isSigner: false, isWritable: true },
      { pubkey: networkStatePDA, isSigner: false, isWritable: false },
      { pubkey: activeEffectsPDA, isSigner: false, isWritable: false },
      { pubkey: SYSVAR_CLOCK_PUBKEY, isSigner: false, isWritable: false },
      { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
    ],
    programId: FINOVA_CORE_PROGRAM_ID,
    data: data.subarray(0, offset + metadataBytes.length),
  });
}

/**
 * Creates an instruction to claim XP level rewards
 */
export async function createClaimXPRewardsInstruction(
  user: PublicKey,
  targetLevel: number
): Promise<TransactionInstruction> {
  const [userStatePDA] = PublicKey.findProgramAddressSync(
    [Buffer.from(USER_SEED), user.toBuffer()],
    FINOVA_CORE_PROGRAM_ID
  );

  const [xpStatePDA] = PublicKey.findProgramAddressSync(
    [Buffer.from(XP_SEED), user.toBuffer()],
    FINOVA_CORE_PROGRAM_ID
  );

  const [networkStatePDA] = PublicKey.findProgramAddressSync(
    [Buffer.from(NETWORK_SEED)],
    FINOVA_CORE_PROGRAM_ID
  );

  // Prepare instruction data
  const data = Buffer.alloc(8);
  let offset = 0;

  // Instruction discriminator for claim_xp_rewards
  data.writeUInt8(4, offset); // Assuming claim_xp_rewards is instruction index 4
  offset += 1;

  // Target level
  data.writeUInt32LE(targetLevel, offset);
  offset += 4;

  return new TransactionInstruction({
    keys: [
      { pubkey: user, isSigner: true, isWritable: false },
      { pubkey: userStatePDA, isSigner: false, isWritable: true },
      { pubkey: xpStatePDA, isSigner: false, isWritable: true },
      { pubkey: networkStatePDA, isSigner: false, isWritable: false },
      { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
    ],
    programId: FINOVA_CORE_PROGRAM_ID,
    data: data.subarray(0, offset),
  });
}

/**
 * Creates an instruction to manage daily streak
 */
export async function createManageStreakInstruction(
  user: PublicKey,
  params: StreakManagementParams
): Promise<TransactionInstruction> {
  const [userStatePDA] = PublicKey.findProgramAddressSync(
    [Buffer.from(USER_SEED), user.toBuffer()],
    FINOVA_CORE_PROGRAM_ID
  );

  const [xpStatePDA] = PublicKey.findProgramAddressSync(
    [Buffer.from(XP_SEED), user.toBuffer()],
    FINOVA_CORE_PROGRAM_ID
  );

  const [networkStatePDA] = PublicKey.findProgramAddressSync(
    [Buffer.from(NETWORK_SEED)],
    FINOVA_CORE_PROGRAM_ID
  );

  // Prepare instruction data
  const data = Buffer.alloc(16);
  let offset = 0;

  // Instruction discriminator for manage_streak
  data.writeUInt8(5, offset); // Assuming manage_streak is instruction index 5
  offset += 1;

  // Action type
  const actionMap = { maintain: 0, break: 1, restore: 2 };
  data.writeUInt8(actionMap[params.action], offset);
  offset += 1;

  // Extension days (for restore action)
  const extensionDays = params.extensionDays || 0;
  data.writeUInt32LE(extensionDays, offset);
  offset += 4;

  return new TransactionInstruction({
    keys: [
      { pubkey: user, isSigner: true, isWritable: false },
      { pubkey: userStatePDA, isSigner: false, isWritable: true },
      { pubkey: xpStatePDA, isSigner: false, isWritable: true },
      { pubkey: networkStatePDA, isSigner: false, isWritable: false },
      { pubkey: SYSVAR_CLOCK_PUBKEY, isSigner: false, isWritable: false },
    ],
    programId: FINOVA_CORE_PROGRAM_ID,
    data: data.subarray(0, offset),
  });
}

/**
 * Creates an instruction to complete daily quest
 */
export async function createCompleteDailyQuestInstruction(
  user: PublicKey,
  params: DailyQuestParams
): Promise<TransactionInstruction> {
  const [userStatePDA] = PublicKey.findProgramAddressSync(
    [Buffer.from(USER_SEED), user.toBuffer()],
    FINOVA_CORE_PROGRAM_ID
  );

  const [xpStatePDA] = PublicKey.findProgramAddressSync(
    [Buffer.from(XP_SEED), user.toBuffer()],
    FINOVA_CORE_PROGRAM_ID
  );

  const [networkStatePDA] = PublicKey.findProgramAddressSync(
    [Buffer.from(NETWORK_SEED)],
    FINOVA_CORE_PROGRAM_ID
  );

  // Prepare instruction data
  const data = Buffer.alloc(16);
  let offset = 0;

  // Instruction discriminator for complete_daily_quest
  data.writeUInt8(6, offset); // Assuming complete_daily_quest is instruction index 6
  offset += 1;

  // Quest ID
  data.writeUInt32LE(params.questId, offset);
  offset += 4;

  // Completion status
  data.writeUInt8(params.completed ? 1 : 0, offset);
  offset += 1;

  // Bonus XP
  const bonusXP = params.bonusXP || 0;
  data.writeUInt32LE(bonusXP, offset);
  offset += 4;

  return new TransactionInstruction({
    keys: [
      { pubkey: user, isSigner: true, isWritable: false },
      { pubkey: userStatePDA, isSigner: false, isWritable: true },
      { pubkey: xpStatePDA, isSigner: false, isWritable: true },
      { pubkey: networkStatePDA, isSigner: false, isWritable: false },
      { pubkey: SYSVAR_CLOCK_PUBKEY, isSigner: false, isWritable: false },
    ],
    programId: FINOVA_CORE_PROGRAM_ID,
    data: data.subarray(0, offset),
  });
}

/**
 * Creates an instruction to apply viral content bonus
 */
export async function createApplyViralBonusInstruction(
  user: PublicKey,
  contentId: string,
  viralMetrics: {
    views: number;
    engagement_rate: number;
    shares: number;
    platform: SocialPlatform;
  }
): Promise<TransactionInstruction> {
  const [userStatePDA] = PublicKey.findProgramAddressSync(
    [Buffer.from(USER_SEED), user.toBuffer()],
    FINOVA_CORE_PROGRAM_ID
  );

  const [xpStatePDA] = PublicKey.findProgramAddressSync(
    [Buffer.from(XP_SEED), user.toBuffer()],
    FINOVA_CORE_PROGRAM_ID
  );

  const [networkStatePDA] = PublicKey.findProgramAddressSync(
    [Buffer.from(NETWORK_SEED)],
    FINOVA_CORE_PROGRAM_ID
  );

  // Prepare instruction data
  const data = Buffer.alloc(256);
  let offset = 0;

  // Instruction discriminator for apply_viral_bonus
  data.writeUInt8(7, offset); // Assuming apply_viral_bonus is instruction index 7
  offset += 1;

  // Content ID length and content
  const contentIdBytes = Buffer.from(contentId, 'utf-8');
  data.writeUInt16LE(contentIdBytes.length, offset);
  offset += 2;
  contentIdBytes.copy(data, offset);
  offset += contentIdBytes.length;

  // Viral metrics
  data.writeUInt32LE(viralMetrics.views, offset);
  offset += 4;
  
  // Engagement rate (scaled by 10000 for precision)
  const engagementRateScaled = Math.floor(viralMetrics.engagement_rate * 10000);
  data.writeUInt32LE(engagementRateScaled, offset);
  offset += 4;
  
  data.writeUInt32LE(viralMetrics.shares, offset);
  offset += 4;
  
  data.writeUInt8(viralMetrics.platform, offset);
  offset += 1;

  return new TransactionInstruction({
    keys: [
      { pubkey: user, isSigner: true, isWritable: false },
      { pubkey: userStatePDA, isSigner: false, isWritable: true },
      { pubkey: xpStatePDA, isSigner: false, isWritable: true },
      { pubkey: networkStatePDA, isSigner: false, isWritable: false },
      { pubkey: SYSVAR_CLOCK_PUBKEY, isSigner: false, isWritable: false },
    ],
    programId: FINOVA_CORE_PROGRAM_ID,
    data: data.subarray(0, offset),
  });
}

/**
 * Utility function to calculate XP required for a specific level
 */
export function calculateXPForLevel(level: number): number {
  if (level <= 10) {
    return level * 100; // Bronze: 100 XP per level
  } else if (level <= 25) {
    return 1000 + (level - 10) * 200; // Silver: 200 XP per level after Bronze
  } else if (level <= 50) {
    return 4000 + (level - 25) * 400; // Gold: 400 XP per level after Silver
  } else if (level <= 75) {
    return 14000 + (level - 50) * 600; // Platinum: 600 XP per level after Gold
  } else if (level <= 100) {
    return 29000 + (level - 75) * 800; // Diamond: 800 XP per level after Platinum
  } else {
    return 49000 + (level - 100) * 1000; // Mythic: 1000 XP per level after Diamond
  }
}

/**
 * Utility function to get level from XP amount
 */
export function getLevelFromXP(xp: number): number {
  let level = 1;
  let requiredXP = 0;

  while (requiredXP <= xp) {
    level++;
    requiredXP = calculateXPForLevel(level);
  }

  return level - 1;
}

/**
 * Utility function to get XP multiplier for a specific level
 */
export function getXPMultiplierForLevel(level: number): number {
  if (level <= 10) {
    return 1.0 + (level - 1) * 0.02; // 1.0x to 1.18x
  } else if (level <= 25) {
    return 1.2 + (level - 10) * 0.04; // 1.2x to 1.8x
  } else if (level <= 50) {
    return 1.9 + (level - 25) * 0.024; // 1.9x to 2.5x
  } else if (level <= 75) {
    return 2.6 + (level - 50) * 0.024; // 2.6x to 3.2x
  } else if (level <= 100) {
    return 3.3 + (level - 75) * 0.028; // 3.3x to 4.0x
  } else {
    return 4.1 + (level - 100) * 0.009; // 4.1x to 5.0x (max)
  }
}

/**
 * Utility function to get badge tier name for a level
 */
export function getBadgeTierForLevel(level: number): string {
  if (level <= 10) {
    return `Bronze ${level}`;
  } else if (level <= 25) {
    return `Silver ${level - 10}`;
  } else if (level <= 50) {
    return `Gold ${level - 25}`;
  } else if (level <= 75) {
    return `Platinum ${level - 50}`;
  } else if (level <= 100) {
    return `Diamond ${level - 75}`;
  } else {
    return `Mythic ${level - 100}`;
  }
}

/**
 * Utility function to calculate platform multiplier
 */
export function getPlatformMultiplier(platform: SocialPlatform): number {
  const multipliers = {
    [SocialPlatform.TikTok]: 1.3,
    [SocialPlatform.Instagram]: 1.2,
    [SocialPlatform.YouTube]: 1.4,
    [SocialPlatform.Facebook]: 1.1,
    [SocialPlatform.Twitter]: 1.2,
    [SocialPlatform.App]: 1.0,
  };
  
  return multipliers[platform] || 1.0;
}

/**
 * Utility function to calculate base XP for activity type
 */
export function getBaseXPForActivity(activityType: XPActivityType): number {
  const baseXPMap = {
    [XPActivityType.OriginalTextPost]: 50,
    [XPActivityType.PhotoImagePost]: 75,
    [XPActivityType.VideoContent]: 150,
    [XPActivityType.StoryStatus]: 25,
    [XPActivityType.MeaningfulComment]: 25,
    [XPActivityType.LikeReact]: 5,
    [XPActivityType.ShareRepost]: 15,
    [XPActivityType.FollowSubscribe]: 20,
    [XPActivityType.FirstDailyLogin]: 10,
    [XPActivityType.CompleteDailyQuest]: 100,
    [XPActivityType.AchieveMilestone]: 500,
    [XPActivityType.ViralContent]: 1000,
    [XPActivityType.InstagramPost]: 75,
    [XPActivityType.TikTokPost]: 75,
    [XPActivityType.YouTubePost]: 150,
    [XPActivityType.FacebookPost]: 50,
    [XPActivityType.TwitterPost]: 50,
    [XPActivityType.GuildParticipation]: 30,
    [XPActivityType.GuildLeadership]: 100,
    [XPActivityType.GuildEvent]: 200,
    [XPActivityType.UseSpecialCard]: 50,
    [XPActivityType.NFTTrade]: 100,
    [XPActivityType.NFTCreate]: 200,
  };

  return baseXPMap[activityType] || 10;
}

/**
 * Utility function to calculate daily XP limits for activity type
 */
export function getDailyLimitForActivity(activityType: XPActivityType): number | null {
  const dailyLimits = {
    [XPActivityType.PhotoImagePost]: 20,
    [XPActivityType.VideoContent]: 10,
    [XPActivityType.StoryStatus]: 50,
    [XPActivityType.MeaningfulComment]: 100,
    [XPActivityType.LikeReact]: 200,
    [XPActivityType.ShareRepost]: 50,
    [XPActivityType.FollowSubscribe]: 25,
    [XPActivityType.FirstDailyLogin]: 1,
    [XPActivityType.CompleteDailyQuest]: 3,
  };

  return dailyLimits[activityType] || null; // null means no limit
}

/**
 * Export all XP-related types and functions
 */
export {
  // Re-export types for convenience
  type XPUpdateParams,
  type LevelProgressionParams,
  type StreakManagementParams,
  type DailyQuestParams,
};
