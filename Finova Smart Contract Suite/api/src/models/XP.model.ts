import { 
  Entity, 
  PrimaryGeneratedColumn, 
  Column, 
  ManyToOne, 
  OneToMany,
  CreateDateColumn, 
  UpdateDateColumn, 
  Index,
  JoinColumn
} from 'typeorm';
import { User } from './User.model';
import { XPActivity } from './XPActivity.model';

export enum XPActivityType {
  ORIGINAL_POST = 'original_post',
  PHOTO_POST = 'photo_post',
  VIDEO_CONTENT = 'video_content',
  STORY_STATUS = 'story_status',
  MEANINGFUL_COMMENT = 'meaningful_comment',
  LIKE_REACT = 'like_react',
  SHARE_REPOST = 'share_repost',
  FOLLOW_SUBSCRIBE = 'follow_subscribe',
  DAILY_LOGIN = 'daily_login',
  DAILY_QUEST = 'daily_quest',
  MILESTONE_ACHIEVEMENT = 'milestone_achievement',
  VIRAL_CONTENT = 'viral_content',
  REFERRAL_SUCCESS = 'referral_success',
  GUILD_PARTICIPATION = 'guild_participation',
  SPECIAL_EVENT = 'special_event'
}

export enum SocialPlatform {
  INSTAGRAM = 'instagram',
  TIKTOK = 'tiktok',
  YOUTUBE = 'youtube',
  FACEBOOK = 'facebook',
  TWITTER_X = 'twitter_x',
  FINOVA_APP = 'finova_app'
}

export enum XPTier {
  BRONZE = 'bronze',
  SILVER = 'silver',
  GOLD = 'gold',
  PLATINUM = 'platinum',
  DIAMOND = 'diamond',
  MYTHIC = 'mythic'
}

@Entity('xp_records')
@Index(['userId', 'createdAt'])
@Index(['activityType', 'createdAt'])
@Index(['platform', 'createdAt'])
export class XP {
  @PrimaryGeneratedColumn('uuid')
  id: string;

  @Column('uuid')
  @Index()
  userId: string;

  @ManyToOne(() => User, user => user.xpRecords, { onDelete: 'CASCADE' })
  @JoinColumn({ name: 'userId' })
  user: User;

  @Column({
    type: 'enum',
    enum: XPActivityType,
    nullable: false
  })
  @Index()
  activityType: XPActivityType;

  @Column({
    type: 'enum',
    enum: SocialPlatform,
    nullable: false
  })
  @Index()
  platform: SocialPlatform;

  // Core XP Values
  @Column('decimal', { precision: 10, scale: 2, default: 0 })
  baseXP: number;

  @Column('decimal', { precision: 4, scale: 2, default: 1.0 })
  platformMultiplier: number;

  @Column('decimal', { precision: 4, scale: 2, default: 1.0 })
  qualityScore: number;

  @Column('decimal', { precision: 4, scale: 2, default: 1.0 })
  streakBonus: number;

  @Column('decimal', { precision: 4, scale: 2, default: 1.0 })
  levelProgression: number;

  @Column('decimal', { precision: 10, scale: 2, default: 0 })
  finalXP: number;

  // Activity Details
  @Column('text', { nullable: true })
  contentHash: string;

  @Column('text', { nullable: true })
  contentUrl: string;

  @Column('jsonb', { nullable: true })
  contentMetadata: {
    views?: number;
    likes?: number;
    shares?: number;
    comments?: number;
    duration?: number;
    wordCount?: number;
    hashtags?: string[];
    mentions?: string[];
    isOriginal?: boolean;
    viralScore?: number;
  };

  // Quality Assessment
  @Column('jsonb', { nullable: true })
  qualityMetrics: {
    originality: number;
    engagement: number;
    relevance: number;
    brandSafety: number;
    humanGenerated: number;
    overallScore: number;
  };

  // Streak Information
  @Column('int', { default: 1 })
  currentStreak: number;

  @Column('int', { default: 0 })
  bestStreak: number;

  @Column('date', { nullable: true })
  lastActivityDate: Date;

  // Mining Impact
  @Column('decimal', { precision: 4, scale: 2, default: 0 })
  miningBoostMultiplier: number;

  @Column('int', { default: 0 })
  miningBoostDurationHours: number;

  // Level & Tier Information
  @Column('int', { default: 1 })
  userLevelAtTime: number;

  @Column({
    type: 'enum',
    enum: XPTier,
    default: XPTier.BRONZE
  })
  userTierAtTime: XPTier;

  // Validation & Security
  @Column('boolean', { default: false })
  isValidated: boolean;

  @Column('boolean', { default: false })
  isFlagged: boolean;

  @Column('text', { nullable: true })
  flagReason: string;

  @Column('uuid', { nullable: true })
  validatedBy: string;

  @Column('timestamp', { nullable: true })
  validatedAt: Date;

  // Blockchain Integration
  @Column('text', { nullable: true })
  transactionHash: string;

  @Column('boolean', { default: false })
  isOnChain: boolean;

  // Daily Limits Tracking
  @Column('date')
  @Index()
  activityDate: Date;

  @Column('int', { default: 1 })
  dailyActivityCount: number;

  @CreateDateColumn()
  createdAt: Date;

  @UpdateDateColumn()
  updatedAt: Date;

  // Computed Properties
  get isViralContent(): boolean {
    return this.contentMetadata?.views >= 1000 || this.contentMetadata?.viralScore >= 0.8;
  }

  get effectiveMiningBoost(): number {
    const now = new Date();
    const createdTime = new Date(this.createdAt);
    const hoursElapsed = (now.getTime() - createdTime.getTime()) / (1000 * 60 * 60);
    
    if (hoursElapsed <= this.miningBoostDurationHours) {
      return this.miningBoostMultiplier;
    }
    return 0;
  }

  get qualityGrade(): string {
    const score = this.qualityScore;
    if (score >= 1.8) return 'S+';
    if (score >= 1.6) return 'S';
    if (score >= 1.4) return 'A';
    if (score >= 1.2) return 'B';
    if (score >= 1.0) return 'C';
    return 'D';
  }

  // Static Methods for XP Calculation
  static calculateBaseXP(activityType: XPActivityType): number {
    const baseXPMap: Record<XPActivityType, number> = {
      [XPActivityType.ORIGINAL_POST]: 50,
      [XPActivityType.PHOTO_POST]: 75,
      [XPActivityType.VIDEO_CONTENT]: 150,
      [XPActivityType.STORY_STATUS]: 25,
      [XPActivityType.MEANINGFUL_COMMENT]: 25,
      [XPActivityType.LIKE_REACT]: 5,
      [XPActivityType.SHARE_REPOST]: 15,
      [XPActivityType.FOLLOW_SUBSCRIBE]: 20,
      [XPActivityType.DAILY_LOGIN]: 10,
      [XPActivityType.DAILY_QUEST]: 100,
      [XPActivityType.MILESTONE_ACHIEVEMENT]: 500,
      [XPActivityType.VIRAL_CONTENT]: 1000,
      [XPActivityType.REFERRAL_SUCCESS]: 100,
      [XPActivityType.GUILD_PARTICIPATION]: 50,
      [XPActivityType.SPECIAL_EVENT]: 200
    };
    return baseXPMap[activityType] || 0;
  }

  static calculatePlatformMultiplier(platform: SocialPlatform): number {
    const multiplierMap: Record<SocialPlatform, number> = {
      [SocialPlatform.TIKTOK]: 1.3,
      [SocialPlatform.YOUTUBE]: 1.4,
      [SocialPlatform.INSTAGRAM]: 1.2,
      [SocialPlatform.TWITTER_X]: 1.2,
      [SocialPlatform.FACEBOOK]: 1.1,
      [SocialPlatform.FINOVA_APP]: 1.0
    };
    return multiplierMap[platform] || 1.0;
  }

  static calculateStreakBonus(streakDays: number): number {
    if (streakDays >= 30) return 3.0;
    if (streakDays >= 14) return 2.5;
    if (streakDays >= 7) return 2.0;
    if (streakDays >= 3) return 1.5;
    return 1.0;
  }

  static calculateLevelProgression(currentLevel: number): number {
    return Math.exp(-0.01 * currentLevel);
  }

  static calculateFinalXP(
    baseXP: number,
    platformMultiplier: number,
    qualityScore: number,
    streakBonus: number,
    levelProgression: number
  ): number {
    return Math.round(baseXP * platformMultiplier * qualityScore * streakBonus * levelProgression * 100) / 100;
  }

  static getLevelFromTotalXP(totalXP: number): number {
    if (totalXP >= 100000) return Math.floor(100 + Math.log10(totalXP / 100000) * 50);
    if (totalXP >= 50000) return Math.floor(76 + (totalXP - 50000) / 2000);
    if (totalXP >= 20000) return Math.floor(51 + (totalXP - 20000) / 1200);
    if (totalXP >= 5000) return Math.floor(26 + (totalXP - 5000) / 600);
    if (totalXP >= 1000) return Math.floor(11 + (totalXP - 1000) / 300);
    return Math.floor(1 + totalXP / 100);
  }

  static getTierFromLevel(level: number): XPTier {
    if (level >= 101) return XPTier.MYTHIC;
    if (level >= 76) return XPTier.DIAMOND;
    if (level >= 51) return XPTier.PLATINUM;
    if (level >= 26) return XPTier.GOLD;
    if (level >= 11) return XPTier.SILVER;
    return XPTier.BRONZE;
  }

  static getMiningMultiplier(level: number): number {
    if (level >= 101) return 4.1 + (level - 101) * 0.02; // Max 5.0x
    if (level >= 76) return 3.3 + (level - 76) * 0.032;
    if (level >= 51) return 2.6 + (level - 51) * 0.028;
    if (level >= 26) return 1.9 + (level - 26) * 0.028;
    if (level >= 11) return 1.3 + (level - 11) * 0.04;
    return 1.0 + (level - 1) * 0.02;
  }

  static getDailyFINCap(level: number): number {
    if (level >= 101) return 10.0 + (level - 101) * 0.1; // Max 15.0
    if (level >= 76) return 8.0 + (level - 76) * 0.08;
    if (level >= 51) return 6.0 + (level - 51) * 0.08;
    if (level >= 26) return 4.0 + (level - 26) * 0.08;
    if (level >= 11) return 2.0 + (level - 11) * 0.133;
    return 0.5 + (level - 1) * 0.15;
  }

  static checkDailyLimits(activityType: XPActivityType, currentCount: number): boolean {
    const dailyLimits: Partial<Record<XPActivityType, number>> = {
      [XPActivityType.PHOTO_POST]: 20,
      [XPActivityType.VIDEO_CONTENT]: 10,
      [XPActivityType.STORY_STATUS]: 50,
      [XPActivityType.MEANINGFUL_COMMENT]: 100,
      [XPActivityType.LIKE_REACT]: 200,
      [XPActivityType.SHARE_REPOST]: 50,
      [XPActivityType.FOLLOW_SUBSCRIBE]: 25,
      [XPActivityType.DAILY_LOGIN]: 1,
      [XPActivityType.DAILY_QUEST]: 3
    };
    
    const limit = dailyLimits[activityType];
    return limit === undefined || currentCount < limit;
  }

  // Instance Methods
  calculateMiningImpact(): { multiplier: number; durationHours: number } {
    const multiplierMap: Partial<Record<XPActivityType, number>> = {
      [XPActivityType.ORIGINAL_POST]: 1.2,
      [XPActivityType.VIDEO_CONTENT]: 1.5,
      [XPActivityType.VIRAL_CONTENT]: 2.0,
      [XPActivityType.DAILY_QUEST]: 1.5,
      [XPActivityType.MILESTONE_ACHIEVEMENT]: 2.5,
      [XPActivityType.GUILD_PARTICIPATION]: 1.3
    };

    const durationMap: Partial<Record<XPActivityType, number>> = {
      [XPActivityType.ORIGINAL_POST]: 24,
      [XPActivityType.VIDEO_CONTENT]: 24,
      [XPActivityType.VIRAL_CONTENT]: 48,
      [XPActivityType.DAILY_QUEST]: 12,
      [XPActivityType.MILESTONE_ACHIEVEMENT]: 72,
      [XPActivityType.GUILD_PARTICIPATION]: 24
    };

    return {
      multiplier: multiplierMap[this.activityType] || 1.0,
      durationHours: durationMap[this.activityType] || 0
    };
  }

  updateQualityMetrics(metrics: {
    originality: number;
    engagement: number;
    relevance: number;
    brandSafety: number;
    humanGenerated: number;
  }): void {
    this.qualityMetrics = {
      ...metrics,
      overallScore: (metrics.originality + metrics.engagement + metrics.relevance + 
                    metrics.brandSafety + metrics.humanGenerated) / 5
    };
    this.qualityScore = Math.max(0.5, Math.min(2.0, this.qualityMetrics.overallScore));
  }

  flagForReview(reason: string): void {
    this.isFlagged = true;
    this.flagReason = reason;
    this.isValidated = false;
  }

  validate(validatorId: string): void {
    this.isValidated = true;
    this.isFlagged = false;
    this.validatedBy = validatorId;
    this.validatedAt = new Date();
    this.flagReason = null;
  }

  toBlockchainData(): object {
    return {
      userId: this.userId,
      activityType: this.activityType,
      platform: this.platform,
      finalXP: this.finalXP,
      qualityScore: this.qualityScore,
      timestamp: this.createdAt.getTime(),
      contentHash: this.contentHash
    };
  }
}

// XP Activity Detail Entity for transaction history
@Entity('xp_activities')
@Index(['xpRecordId'])
export class XPActivity {
  @PrimaryGeneratedColumn('uuid')
  id: string;

  @Column('uuid')
  xpRecordId: string;

  @ManyToOne(() => XP, { onDelete: 'CASCADE' })
  @JoinColumn({ name: 'xpRecordId' })
  xpRecord: XP;

  @Column('text', { nullable: true })
  description: string;

  @Column('jsonb', { nullable: true })
  additionalData: Record<string, any>;

  @CreateDateColumn()
  createdAt: Date;
}

export default XP;
