import {
  Entity,
  PrimaryGeneratedColumn,
  Column,
  CreateDateColumn,
  UpdateDateColumn,
  Index,
  OneToMany,
  ManyToOne,
  JoinColumn,
  BeforeInsert,
  BeforeUpdate
} from 'typeorm';
import { IsEmail, IsOptional, IsEnum, IsNumber, IsBoolean, IsString, IsDate } from 'class-validator';
import { Exclude, Transform } from 'class-transformer';
import * as bcrypt from 'bcrypt';
import { Mining } from './Mining.model';
import { XP } from './XP.model';
import { Referral } from './Referral.model';
import { NFT } from './NFT.model';
import { Guild } from './Guild.model';
import { Transaction } from './Transaction.model';

export enum UserStatus {
  PENDING = 'pending',
  ACTIVE = 'active',
  SUSPENDED = 'suspended',
  BANNED = 'banned',
  KYC_PENDING = 'kyc_pending',
  KYC_VERIFIED = 'kyc_verified',
  KYC_REJECTED = 'kyc_rejected'
}

export enum XPTier {
  BRONZE_I = 'bronze_1',
  BRONZE_X = 'bronze_10',
  SILVER_I = 'silver_1',
  SILVER_XV = 'silver_15',
  GOLD_I = 'gold_1',
  GOLD_XXV = 'gold_25',
  PLATINUM_I = 'platinum_1',
  PLATINUM_XXV = 'platinum_25',
  DIAMOND_I = 'diamond_1',
  DIAMOND_XXV = 'diamond_25',
  MYTHIC = 'mythic'
}

export enum RPTier {
  EXPLORER = 'explorer',
  CONNECTOR = 'connector',
  INFLUENCER = 'influencer',
  LEADER = 'leader',
  AMBASSADOR = 'ambassador'
}

@Entity('users')
@Index(['email'], { unique: true })
@Index(['username'], { unique: true })
@Index(['referralCode'], { unique: true })
@Index(['walletAddress'])
@Index(['status'])
@Index(['kycStatus'])
@Index(['createdAt'])
@Index(['totalFIN', 'xpLevel', 'rpPoints'])
export class User {
  @PrimaryGeneratedColumn('uuid')
  id: string;

  // Basic Information
  @Column({ type: 'varchar', length: 255, unique: true })
  @IsEmail()
  email: string;

  @Column({ type: 'varchar', length: 50, unique: true })
  @IsString()
  username: string;

  @Column({ type: 'varchar', length: 255 })
  @Exclude({ toPlainOnly: true })
  password: string;

  @Column({ type: 'varchar', length: 100, nullable: true })
  @IsOptional()
  @IsString()
  firstName?: string;

  @Column({ type: 'varchar', length: 100, nullable: true })
  @IsOptional()
  @IsString()
  lastName?: string;

  @Column({ type: 'varchar', length: 255, nullable: true })
  @IsOptional()
  profilePicture?: string;

  @Column({ type: 'text', nullable: true })
  @IsOptional()
  bio?: string;

  @Column({ type: 'varchar', length: 10, nullable: true })
  @IsOptional()
  country?: string;

  @Column({ type: 'varchar', length: 20, nullable: true })
  @IsOptional()
  phoneNumber?: string;

  @Column({ type: 'date', nullable: true })
  @IsOptional()
  @IsDate()
  dateOfBirth?: Date;

  // Account Status
  @Column({
    type: 'enum',
    enum: UserStatus,
    default: UserStatus.PENDING
  })
  @IsEnum(UserStatus)
  status: UserStatus;

  @Column({ type: 'boolean', default: false })
  @IsBoolean()
  emailVerified: boolean;

  @Column({ type: 'boolean', default: false })
  @IsBoolean()
  phoneVerified: boolean;

  @Column({ type: 'varchar', length: 20, default: UserStatus.KYC_PENDING })
  kycStatus: string;

  @Column({ type: 'text', nullable: true })
  kycDocuments?: string; // JSON string of document URLs

  @Column({ type: 'timestamp', nullable: true })
  kycVerifiedAt?: Date;

  @Column({ type: 'timestamp', nullable: true })
  lastLoginAt?: Date;

  // Blockchain & Wallet
  @Column({ type: 'varchar', length: 64, nullable: true, unique: true })
  @Index()
  walletAddress?: string;

  @Column({ type: 'varchar', length: 128, nullable: true })
  @Exclude({ toPlainOnly: true })
  privateKeyEncrypted?: string;

  // Mining System
  @Column({ type: 'decimal', precision: 18, scale: 8, default: '0' })
  @Transform(({ value }) => parseFloat(value))
  totalFIN: number;

  @Column({ type: 'decimal', precision: 18, scale: 8, default: '0' })
  @Transform(({ value }) => parseFloat(value))
  stakedFIN: number;

  @Column({ type: 'decimal', precision: 12, scale: 8, default: '0.05' })
  @Transform(({ value }) => parseFloat(value))
  baseMiningRate: number;

  @Column({ type: 'decimal', precision: 8, scale: 4, default: '1.0' })
  @Transform(({ value }) => parseFloat(value))
  miningMultiplier: number;

  @Column({ type: 'timestamp', nullable: true })
  lastMiningAt?: Date;

  @Column({ type: 'int', default: 0 })
  @IsNumber()
  consecutiveMiningDays: number;

  @Column({ type: 'timestamp', nullable: true })
  miningStreakStarted?: Date;

  // XP System
  @Column({ type: 'bigint', default: '0' })
  @Transform(({ value }) => parseInt(value))
  totalXP: number;

  @Column({ type: 'int', default: 1 })
  @IsNumber()
  xpLevel: number;

  @Column({
    type: 'enum',
    enum: XPTier,
    default: XPTier.BRONZE_I
  })
  @IsEnum(XPTier)
  xpTier: XPTier;

  @Column({ type: 'bigint', default: '0' })
  @Transform(({ value }) => parseInt(value))
  dailyXP: number;

  @Column({ type: 'timestamp', nullable: true })
  lastXPResetAt?: Date;

  @Column({ type: 'int', default: 0 })
  @IsNumber()
  consecutiveXPDays: number;

  // Referral System (RP)
  @Column({ type: 'varchar', length: 12, unique: true })
  referralCode: string;

  @Column({ type: 'varchar', length: 12, nullable: true })
  @IsOptional()
  usedReferralCode?: string;

  @ManyToOne(() => User, { nullable: true })
  @JoinColumn({ name: 'referredBy' })
  referrer?: User;

  @Column({ type: 'uuid', nullable: true })
  referredBy?: string;

  @Column({ type: 'bigint', default: '0' })
  @Transform(({ value }) => parseInt(value))
  rpPoints: number;

  @Column({
    type: 'enum',
    enum: RPTier,
    default: RPTier.EXPLORER
  })
  @IsEnum(RPTier)
  rpTier: RPTier;

  @Column({ type: 'int', default: 0 })
  @IsNumber()
  directReferrals: number;

  @Column({ type: 'int', default: 0 })
  @IsNumber()
  totalNetworkSize: number;

  @Column({ type: 'decimal', precision: 6, scale: 3, default: '0.0' })
  @Transform(({ value }) => parseFloat(value))
  networkQualityScore: number;

  // Social Media Integration
  @Column({ type: 'json', nullable: true })
  socialAccounts?: {
    instagram?: { username: string; verified: boolean; followersCount?: number };
    tiktok?: { username: string; verified: boolean; followersCount?: number };
    youtube?: { username: string; verified: boolean; subscribersCount?: number };
    facebook?: { username: string; verified: boolean; friendsCount?: number };
    twitter?: { username: string; verified: boolean; followersCount?: number };
  };

  @Column({ type: 'int', default: 0 })
  @IsNumber()
  totalSocialPosts: number;

  @Column({ type: 'int', default: 0 })
  @IsNumber()
  viralContentCount: number;

  @Column({ type: 'timestamp', nullable: true })
  lastSocialActivityAt?: Date;

  // Activity & Engagement
  @Column({ type: 'decimal', precision: 8, scale: 4, default: '0.5' })
  @Transform(({ value }) => parseFloat(value))
  humanProbabilityScore: number;

  @Column({ type: 'decimal', precision: 6, scale: 3, default: '1.0' })
  @Transform(({ value }) => parseFloat(value))
  qualityScore: number;

  @Column({ type: 'int', default: 0 })
  @IsNumber()
  suspiciousActivityCount: number;

  @Column({ type: 'timestamp', nullable: true })
  lastActivityAt?: Date;

  @Column({ type: 'int', default: 0 })
  @IsNumber()
  dailyInteractions: number;

  @Column({ type: 'timestamp', nullable: true })
  lastInteractionResetAt?: Date;

  // Staking & DeFi
  @Column({ type: 'decimal', precision: 18, scale: 8, default: '0' })
  @Transform(({ value }) => parseFloat(value))
  totalStakedAmount: number;

  @Column({ type: 'decimal', precision: 8, scale: 4, default: '0.08' })
  @Transform(({ value }) => parseFloat(value))
  stakingAPY: number;

  @Column({ type: 'timestamp', nullable: true })
  firstStakeAt?: Date;

  @Column({ type: 'int', default: 0 })
  @IsNumber()
  stakingTier: number;

  // Gaming & NFT
  @Column({ type: 'int', default: 0 })
  @IsNumber()
  nftCount: number;

  @Column({ type: 'json', nullable: true })
  activeCards?: {
    cardId: string;
    effect: string;
    duration: number;
    expiresAt: Date;
  }[];

  @Column({ type: 'decimal', precision: 18, scale: 8, default: '0' })
  @Transform(({ value }) => parseFloat(value))
  totalCardSpent: number;

  // Guild System
  @ManyToOne(() => Guild, { nullable: true })
  @JoinColumn({ name: 'guildId' })
  guild?: Guild;

  @Column({ type: 'uuid', nullable: true })
  guildId?: string;

  @Column({ type: 'varchar', length: 20, nullable: true })
  guildRole?: string;

  @Column({ type: 'timestamp', nullable: true })
  guildJoinedAt?: Date;

  @Column({ type: 'int', default: 0 })
  @IsNumber()
  guildContributions: number;

  // E-wallet Integration
  @Column({ type: 'json', nullable: true })
  eWallets?: {
    ovo?: { phoneNumber: string; verified: boolean };
    gopay?: { phoneNumber: string; verified: boolean };
    dana?: { phoneNumber: string; verified: boolean };
    shopeepay?: { phoneNumber: string; verified: boolean };
  };

  // Analytics & Metrics
  @Column({ type: 'json', nullable: true })
  weeklyStats?: {
    week: string;
    miningAmount: number;
    xpGained: number;
    rpEarned: number;
    postsCreated: number;
    interactions: number;
  }[];

  @Column({ type: 'json', nullable: true })
  monthlyStats?: {
    month: string;
    totalRewards: number;
    networkGrowth: number;
    qualityScore: number;
  }[];

  // Security & Device Info
  @Column({ type: 'varchar', length: 255, nullable: true })
  deviceFingerprint?: string;

  @Column({ type: 'json', nullable: true })
  loginHistory?: {
    timestamp: Date;
    ipAddress: string;
    userAgent: string;
    location?: string;
  }[];

  @Column({ type: 'varchar', length: 255, nullable: true })
  @Exclude({ toPlainOnly: true })
  twoFactorSecret?: string;

  @Column({ type: 'boolean', default: false })
  @IsBoolean()
  twoFactorEnabled: boolean;

  // Compliance & Legal
  @Column({ type: 'boolean', default: false })
  @IsBoolean()
  termsAccepted: boolean;

  @Column({ type: 'timestamp', nullable: true })
  termsAcceptedAt?: Date;

  @Column({ type: 'boolean', default: false })
  @IsBoolean()
  privacyPolicyAccepted: boolean;

  @Column({ type: 'varchar', length: 10, nullable: true })
  preferredLanguage?: string;

  @Column({ type: 'varchar', length: 10, nullable: true })
  timezone?: string;

  // Timestamps
  @CreateDateColumn()
  createdAt: Date;

  @UpdateDateColumn()
  updatedAt: Date;

  @Column({ type: 'timestamp', nullable: true })
  deletedAt?: Date;

  // Relations
  @OneToMany(() => Mining, mining => mining.user)
  miningRecords: Mining[];

  @OneToMany(() => XP, xp => xp.user)
  xpRecords: XP[];

  @OneToMany(() => Referral, referral => referral.referrer)
  referrals: Referral[];

  @OneToMany(() => Referral, referral => referral.referred)
  referredUsers: Referral[];

  @OneToMany(() => NFT, nft => nft.owner)
  ownedNFTs: NFT[];

  @OneToMany(() => Transaction, transaction => transaction.user)
  transactions: Transaction[];

  // Hooks
  @BeforeInsert()
  @BeforeUpdate()
  async hashPassword(): Promise<void> {
    if (this.password && !this.password.startsWith('$2b$')) {
      this.password = await bcrypt.hash(this.password, 12);
    }
  }

  @BeforeInsert()
  generateReferralCode(): void {
    if (!this.referralCode) {
      this.referralCode = this.generateUniqueCode();
    }
  }

  @BeforeInsert()
  @BeforeUpdate()
  updateTiers(): void {
    this.updateXPTier();
    this.updateRPTier();
    this.updateStakingTier();
  }

  // Methods
  async validatePassword(password: string): Promise<boolean> {
    return bcrypt.compare(password, this.password);
  }

  calculateMiningRate(): number {
    const baseRate = this.baseMiningRate;
    const pioneerBonus = Math.max(1.0, 2.0 - (this.totalNetworkSize / 1000000));
    const referralBonus = 1 + (this.directReferrals * 0.1);
    const securityBonus = this.kycStatus === 'kyc_verified' ? 1.2 : 0.8;
    const regressionFactor = Math.exp(-0.001 * this.totalFIN);
    const xpMultiplier = this.getXPMiningMultiplier();
    const rpMultiplier = this.getRPMiningMultiplier();

    return baseRate * pioneerBonus * referralBonus * securityBonus * 
           regressionFactor * xpMultiplier * rpMultiplier * this.qualityScore;
  }

  calculateXPMultiplier(baseXP: number): number {
    const levelProgression = Math.exp(-0.01 * this.xpLevel);
    const streakBonus = Math.min(3.0, 1.0 + (this.consecutiveXPDays * 0.1));
    return baseXP * this.qualityScore * streakBonus * levelProgression;
  }

  calculateRPValue(): number {
    const directRP = this.rpPoints;
    const networkQualityBonus = this.networkQualityScore;
    const regressionFactor = Math.exp(-0.0001 * this.totalNetworkSize * this.networkQualityScore);
    return directRP * networkQualityBonus * regressionFactor;
  }

  private generateUniqueCode(): string {
    const chars = 'ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789';
    let result = '';
    for (let i = 0; i < 8; i++) {
      result += chars.charAt(Math.floor(Math.random() * chars.length));
    }
    return result;
  }

  private updateXPTier(): void {
    if (this.xpLevel >= 101) this.xpTier = XPTier.MYTHIC;
    else if (this.xpLevel >= 76) this.xpTier = XPTier.DIAMOND_I;
    else if (this.xpLevel >= 51) this.xpTier = XPTier.PLATINUM_I;
    else if (this.xpLevel >= 26) this.xpTier = XPTier.GOLD_I;
    else if (this.xpLevel >= 11) this.xpTier = XPTier.SILVER_I;
    else this.xpTier = XPTier.BRONZE_I;
  }

  private updateRPTier(): void {
    if (this.rpPoints >= 50000) this.rpTier = RPTier.AMBASSADOR;
    else if (this.rpPoints >= 15000) this.rpTier = RPTier.LEADER;
    else if (this.rpPoints >= 5000) this.rpTier = RPTier.INFLUENCER;
    else if (this.rpPoints >= 1000) this.rpTier = RPTier.CONNECTOR;
    else this.rpTier = RPTier.EXPLORER;
  }

  private updateStakingTier(): void {
    if (this.totalStakedAmount >= 10000) this.stakingTier = 5;
    else if (this.totalStakedAmount >= 5000) this.stakingTier = 4;
    else if (this.totalStakedAmount >= 1000) this.stakingTier = 3;
    else if (this.totalStakedAmount >= 500) this.stakingTier = 2;
    else if (this.totalStakedAmount >= 100) this.stakingTier = 1;
    else this.stakingTier = 0;
  }

  private getXPMiningMultiplier(): number {
    const tierMultipliers = {
      [XPTier.BRONZE_I]: 1.0, [XPTier.BRONZE_X]: 1.2,
      [XPTier.SILVER_I]: 1.3, [XPTier.SILVER_XV]: 1.8,
      [XPTier.GOLD_I]: 1.9, [XPTier.GOLD_XXV]: 2.5,
      [XPTier.PLATINUM_I]: 2.6, [XPTier.PLATINUM_XXV]: 3.2,
      [XPTier.DIAMOND_I]: 3.3, [XPTier.DIAMOND_XXV]: 4.0,
      [XPTier.MYTHIC]: 5.0
    };
    return tierMultipliers[this.xpTier] || 1.0;
  }

  private getRPMiningMultiplier(): number {
    const tierMultipliers = {
      [RPTier.EXPLORER]: 1.0,
      [RPTier.CONNECTOR]: 1.2,
      [RPTier.INFLUENCER]: 1.5,
      [RPTier.LEADER]: 2.0,
      [RPTier.AMBASSADOR]: 3.0
    };
    return tierMultipliers[this.rpTier] || 1.0;
  }

  isActive(): boolean {
    return this.status === UserStatus.ACTIVE;
  }

  isKYCVerified(): boolean {
    return this.kycStatus === 'kyc_verified';
  }

  canMine(): boolean {
    return this.isActive() && this.humanProbabilityScore > 0.5;
  }

  getTotalValue(): number {
    return this.totalFIN + this.stakedFIN;
  }

  getNetworkStats() {
    return {
      directReferrals: this.directReferrals,
      totalNetworkSize: this.totalNetworkSize,
      networkQualityScore: this.networkQualityScore,
      rpTier: this.rpTier,
      rpPoints: this.rpPoints
    };
  }

  getMiningStats() {
    return {
      currentRate: this.calculateMiningRate(),
      totalMined: this.totalFIN,
      consecutiveDays: this.consecutiveMiningDays,
      qualityScore: this.qualityScore,
      humanScore: this.humanProbabilityScore
    };
  }
}
