import { Schema, model, Document, Types } from 'mongoose';
import { BaseModel } from './Base.model';

// Interfaces
export interface IReferralStats {
  directReferrals: number;
  activeReferrals: number;
  l2NetworkSize: number;
  l3NetworkSize: number;
  totalNetworkSize: number;
  networkQualityScore: number;
  averageReferralLevel: number;
  retentionRate: number;
  monthlyEarnings: number;
  lifetimeEarnings: number;
  lastUpdated: Date;
}

export interface IReferralTier {
  name: string;
  minRP: number;
  maxRP: number;
  miningBonus: number;
  referralBonusL1: number;
  referralBonusL2: number;
  referralBonusL3: number;
  networkCap: number;
  specialBenefits: string[];
  isActive: boolean;
}

export interface IReferralActivity {
  activityId: Types.ObjectId;
  activityType: 'signup' | 'kyc' | 'mining' | 'xp_gain' | 'achievement' | 'purchase';
  referralLevel: 1 | 2 | 3;
  baseAmount: number;
  bonusAmount: number;
  qualityMultiplier: number;
  timestamp: Date;
  metadata: {
    platform?: string;
    achievement?: string;
    xpGained?: number;
    miningAmount?: number;
  };
}

export interface IReferralCode {
  code: string;
  customCode?: string;
  isActive: boolean;
  maxUses: number;
  currentUses: number;
  createdAt: Date;
  expiresAt?: Date;
  metadata: {
    campaign?: string;
    source?: string;
    customization?: object;
  };
}

export interface IReferralNetwork {
  level: 1 | 2 | 3;
  referralId: Types.ObjectId;
  userId: Types.ObjectId;
  isActive: boolean;
  joinedAt: Date;
  lastActiveAt: Date;
  totalEarningsGenerated: number;
  qualityScore: number;
  activityStreak: number;
}

export interface IReferral extends Document {
  // Basic Info
  userId: Types.ObjectId;
  referralCode: IReferralCode;
  referredBy?: Types.ObjectId;
  referralLevel: 1 | 2 | 3;
  
  // Network Structure
  directReferrals: Types.ObjectId[];
  referralNetwork: IReferralNetwork[];
  uplineReferrals: Types.ObjectId[];
  
  // Statistics
  stats: IReferralStats;
  currentTier: IReferralTier;
  
  // Points & Rewards
  totalRP: number;
  availableRP: number;
  spentRP: number;
  lifetimeEarnings: number;
  pendingRewards: number;
  
  // Activity Tracking
  recentActivities: IReferralActivity[];
  lastActivityAt: Date;
  
  // Quality & Validation
  isQualityNetwork: boolean;
  fraudScore: number;
  validationStatus: 'pending' | 'verified' | 'flagged' | 'banned';
  
  // Performance Metrics
  conversionRate: number;
  averageReferralValue: number;
  retentionRate: number;
  networkGrowthRate: number;
  
  // Timestamps
  createdAt: Date;
  updatedAt: Date;
  
  // Methods
  calculateRPValue(): Promise<number>;
  updateNetworkStats(): Promise<void>;
  validateNetworkQuality(): Promise<boolean>;
  addReferralActivity(activity: Partial<IReferralActivity>): Promise<void>;
  getTierBenefits(): IReferralTier;
  calculateNetworkBonus(amount: number): number;
  checkTierUpgrade(): Promise<boolean>;
  getNetworkAnalytics(): Promise<object>;
}

// Schema Definitions
const ReferralStatsSchema = new Schema<IReferralStats>({
  directReferrals: { type: Number, default: 0, min: 0 },
  activeReferrals: { type: Number, default: 0, min: 0 },
  l2NetworkSize: { type: Number, default: 0, min: 0 },
  l3NetworkSize: { type: Number, default: 0, min: 0 },
  totalNetworkSize: { type: Number, default: 0, min: 0 },
  networkQualityScore: { type: Number, default: 0, min: 0, max: 1 },
  averageReferralLevel: { type: Number, default: 0, min: 0 },
  retentionRate: { type: Number, default: 0, min: 0, max: 1 },
  monthlyEarnings: { type: Number, default: 0, min: 0 },
  lifetimeEarnings: { type: Number, default: 0, min: 0 },
  lastUpdated: { type: Date, default: Date.now }
}, { _id: false });

const ReferralTierSchema = new Schema<IReferralTier>({
  name: { type: String, required: true },
  minRP: { type: Number, required: true, min: 0 },
  maxRP: { type: Number, required: true, min: 0 },
  miningBonus: { type: Number, required: true, min: 0 },
  referralBonusL1: { type: Number, required: true, min: 0, max: 1 },
  referralBonusL2: { type: Number, required: true, min: 0, max: 1 },
  referralBonusL3: { type: Number, required: true, min: 0, max: 1 },
  networkCap: { type: Number, required: true, min: 0 },
  specialBenefits: [{ type: String }],
  isActive: { type: Boolean, default: true }
}, { _id: false });

const ReferralActivitySchema = new Schema<IReferralActivity>({
  activityId: { type: Types.ObjectId, required: true, index: true },
  activityType: { 
    type: String, 
    required: true, 
    enum: ['signup', 'kyc', 'mining', 'xp_gain', 'achievement', 'purchase'],
    index: true
  },
  referralLevel: { type: Number, required: true, enum: [1, 2, 3] },
  baseAmount: { type: Number, required: true, min: 0 },
  bonusAmount: { type: Number, required: true, min: 0 },
  qualityMultiplier: { type: Number, required: true, min: 0.5, max: 2.0 },
  timestamp: { type: Date, default: Date.now, index: true },
  metadata: {
    platform: { type: String },
    achievement: { type: String },
    xpGained: { type: Number, min: 0 },
    miningAmount: { type: Number, min: 0 }
  }
}, { _id: false });

const ReferralCodeSchema = new Schema<IReferralCode>({
  code: { type: String, required: true, unique: true, uppercase: true },
  customCode: { type: String, unique: true, sparse: true },
  isActive: { type: Boolean, default: true, index: true },
  maxUses: { type: Number, default: -1 }, // -1 = unlimited
  currentUses: { type: Number, default: 0, min: 0 },
  createdAt: { type: Date, default: Date.now },
  expiresAt: { type: Date, index: true },
  metadata: {
    campaign: { type: String },
    source: { type: String },
    customization: { type: Schema.Types.Mixed }
  }
}, { _id: false });

const ReferralNetworkSchema = new Schema<IReferralNetwork>({
  level: { type: Number, required: true, enum: [1, 2, 3], index: true },
  referralId: { type: Types.ObjectId, required: true, ref: 'Referral', index: true },
  userId: { type: Types.ObjectId, required: true, ref: 'User', index: true },
  isActive: { type: Boolean, default: true, index: true },
  joinedAt: { type: Date, default: Date.now },
  lastActiveAt: { type: Date, default: Date.now, index: true },
  totalEarningsGenerated: { type: Number, default: 0, min: 0 },
  qualityScore: { type: Number, default: 1.0, min: 0, max: 2.0 },
  activityStreak: { type: Number, default: 0, min: 0 }
}, { _id: false });

// Main Referral Schema
const ReferralSchema = new Schema<IReferral>({
  userId: { 
    type: Types.ObjectId, 
    required: true, 
    ref: 'User', 
    unique: true,
    index: true 
  },
  referralCode: { type: ReferralCodeSchema, required: true },
  referredBy: { type: Types.ObjectId, ref: 'User', index: true },
  referralLevel: { type: Number, enum: [1, 2, 3], default: 1 },
  
  // Network Structure
  directReferrals: [{ type: Types.ObjectId, ref: 'User', index: true }],
  referralNetwork: [ReferralNetworkSchema],
  uplineReferrals: [{ type: Types.ObjectId, ref: 'User' }],
  
  // Statistics & Tier
  stats: { type: ReferralStatsSchema, default: () => ({}) },
  currentTier: { type: ReferralTierSchema, required: true },
  
  // Points & Rewards
  totalRP: { type: Number, default: 0, min: 0, index: true },
  availableRP: { type: Number, default: 0, min: 0 },
  spentRP: { type: Number, default: 0, min: 0 },
  lifetimeEarnings: { type: Number, default: 0, min: 0 },
  pendingRewards: { type: Number, default: 0, min: 0 },
  
  // Activity & Quality
  recentActivities: [ReferralActivitySchema],
  lastActivityAt: { type: Date, default: Date.now, index: true },
  isQualityNetwork: { type: Boolean, default: true, index: true },
  fraudScore: { type: Number, default: 0, min: 0, max: 1 },
  validationStatus: { 
    type: String, 
    enum: ['pending', 'verified', 'flagged', 'banned'],
    default: 'pending',
    index: true 
  },
  
  // Performance
  conversionRate: { type: Number, default: 0, min: 0, max: 1 },
  averageReferralValue: { type: Number, default: 0, min: 0 },
  retentionRate: { type: Number, default: 0, min: 0, max: 1 },
  networkGrowthRate: { type: Number, default: 0, min: 0 }
}, {
  timestamps: true,
  collection: 'referrals'
});

// Indexes
ReferralSchema.index({ userId: 1, 'referralCode.code': 1 });
ReferralSchema.index({ referredBy: 1, createdAt: -1 });
ReferralSchema.index({ totalRP: -1, validationStatus: 1 });
ReferralSchema.index({ 'stats.networkQualityScore': -1 });
ReferralSchema.index({ lastActivityAt: -1, isQualityNetwork: 1 });

// Virtual Fields
ReferralSchema.virtual('activeReferralCount').get(function() {
  const thirtyDaysAgo = new Date(Date.now() - 30 * 24 * 60 * 60 * 1000);
  return this.referralNetwork.filter(r => r.isActive && r.lastActiveAt > thirtyDaysAgo).length;
});

ReferralSchema.virtual('tierName').get(function() {
  return this.currentTier.name;
});

// Instance Methods
ReferralSchema.methods.calculateRPValue = async function(): Promise<number> {
  // Direct referral points calculation
  const directRP = this.directReferrals.length * 100;
  
  // Network effect calculation
  const l2Bonus = this.stats.l2NetworkSize * 30;
  const l3Bonus = this.stats.l3NetworkSize * 10;
  
  // Quality bonus calculation
  const qualityBonus = this.stats.networkQualityScore * this.stats.averageReferralLevel * this.stats.retentionRate;
  
  // Network regression factor (anti-whale mechanism)
  const regressionFactor = Math.exp(-0.0001 * this.stats.totalNetworkSize * this.stats.networkQualityScore);
  
  const totalRP = (directRP + l2Bonus + l3Bonus) * qualityBonus * regressionFactor;
  
  this.totalRP = Math.max(0, totalRP);
  return this.totalRP;
};

ReferralSchema.methods.updateNetworkStats = async function(): Promise<void> {
  const User = model('User');
  const thirtyDaysAgo = new Date(Date.now() - 30 * 24 * 60 * 60 * 1000);
  
  // Count active referrals
  const activeReferrals = await User.countDocuments({
    _id: { $in: this.directReferrals },
    lastActiveAt: { $gte: thirtyDaysAgo }
  });
  
  // Calculate quality score
  const totalReferrals = this.directReferrals.length;
  const qualityScore = totalReferrals > 0 ? (activeReferrals / totalReferrals) : 0;
  
  // Update stats
  this.stats.directReferrals = totalReferrals;
  this.stats.activeReferrals = activeReferrals;
  this.stats.networkQualityScore = qualityScore;
  this.stats.retentionRate = qualityScore;
  this.stats.totalNetworkSize = this.referralNetwork.length;
  this.stats.lastUpdated = new Date();
  
  await this.save();
};

ReferralSchema.methods.validateNetworkQuality = async function(): Promise<boolean> {
  const qualityThreshold = 0.3; // 30% minimum active rate
  const maxSuspiciousConnections = 5;
  
  // Check quality score
  if (this.stats.networkQualityScore < qualityThreshold) {
    this.isQualityNetwork = false;
    this.fraudScore += 0.2;
  }
  
  // Check for suspicious patterns
  const recentSignups = this.referralNetwork.filter(r => {
    const oneDayAgo = new Date(Date.now() - 24 * 60 * 60 * 1000);
    return r.joinedAt > oneDayAgo;
  }).length;
  
  if (recentSignups > maxSuspiciousConnections) {
    this.fraudScore += 0.3;
  }
  
  // Update validation status
  if (this.fraudScore > 0.7) {
    this.validationStatus = 'flagged';
    this.isQualityNetwork = false;
  }
  
  return this.isQualityNetwork;
};

ReferralSchema.methods.addReferralActivity = async function(
  activity: Partial<IReferralActivity>
): Promise<void> {
  const newActivity: IReferralActivity = {
    activityId: activity.activityId!,
    activityType: activity.activityType!,
    referralLevel: activity.referralLevel!,
    baseAmount: activity.baseAmount || 0,
    bonusAmount: activity.bonusAmount || 0,
    qualityMultiplier: activity.qualityMultiplier || 1.0,
    timestamp: new Date(),
    metadata: activity.metadata || {}
  };
  
  this.recentActivities.unshift(newActivity);
  
  // Keep only last 100 activities
  if (this.recentActivities.length > 100) {
    this.recentActivities = this.recentActivities.slice(0, 100);
  }
  
  this.lastActivityAt = new Date();
  await this.save();
};

ReferralSchema.methods.getTierBenefits = function(): IReferralTier {
  return this.currentTier;
};

ReferralSchema.methods.calculateNetworkBonus = function(amount: number): number {
  const tier = this.currentTier;
  return amount * (tier.referralBonusL1 || 0);
};

ReferralSchema.methods.checkTierUpgrade = async function(): Promise<boolean> {
  const REFERRAL_TIERS: IReferralTier[] = [
    { name: 'Explorer', minRP: 0, maxRP: 999, miningBonus: 0, referralBonusL1: 0.1, referralBonusL2: 0, referralBonusL3: 0, networkCap: 10, specialBenefits: ['Basic referral link'], isActive: true },
    { name: 'Connector', minRP: 1000, maxRP: 4999, miningBonus: 20, referralBonusL1: 0.15, referralBonusL2: 0.05, referralBonusL3: 0, networkCap: 25, specialBenefits: ['Custom referral code', 'Priority support'], isActive: true },
    { name: 'Influencer', minRP: 5000, maxRP: 14999, miningBonus: 50, referralBonusL1: 0.2, referralBonusL2: 0.08, referralBonusL3: 0.03, networkCap: 50, specialBenefits: ['Referral analytics', 'Special badge'], isActive: true },
    { name: 'Leader', minRP: 15000, maxRP: 49999, miningBonus: 100, referralBonusL1: 0.25, referralBonusL2: 0.1, referralBonusL3: 0.05, networkCap: 100, specialBenefits: ['Exclusive events', 'Advanced analytics'], isActive: true },
    { name: 'Ambassador', minRP: 50000, maxRP: Infinity, miningBonus: 200, referralBonusL1: 0.3, referralBonusL2: 0.15, referralBonusL3: 0.08, networkCap: -1, specialBenefits: ['DAO governance', 'Maximum benefits'], isActive: true }
  ];
  
  const newTier = REFERRAL_TIERS.find(tier => 
    this.totalRP >= tier.minRP && this.totalRP <= tier.maxRP
  );
  
  if (newTier && newTier.name !== this.currentTier.name) {
    this.currentTier = newTier;
    await this.save();
    return true;
  }
  
  return false;
};

ReferralSchema.methods.getNetworkAnalytics = async function(): Promise<object> {
  const User = model('User');
  
  const analytics = {
    networkSize: {
      direct: this.stats.directReferrals,
      level2: this.stats.l2NetworkSize,
      level3: this.stats.l3NetworkSize,
      total: this.stats.totalNetworkSize
    },
    performance: {
      qualityScore: this.stats.networkQualityScore,
      retentionRate: this.stats.retentionRate,
      conversionRate: this.conversionRate,
      averageValue: this.averageReferralValue
    },
    earnings: {
      lifetime: this.lifetimeEarnings,
      monthly: this.stats.monthlyEarnings,
      pending: this.pendingRewards
    },
    tier: {
      current: this.currentTier.name,
      benefits: this.currentTier.specialBenefits,
      nextTier: this.totalRP < 50000 ? 'Upgrade available' : 'Maximum tier reached'
    }
  };
  
  return analytics;
};

// Static Methods
ReferralSchema.statics.findByReferralCode = function(code: string) {
  return this.findOne({ 'referralCode.code': code.toUpperCase() });
};

ReferralSchema.statics.getLeaderboard = function(limit: number = 100) {
  return this.find({ validationStatus: 'verified' })
    .sort({ totalRP: -1, 'stats.networkQualityScore': -1 })
    .limit(limit)
    .populate('userId', 'username avatar level');
};

ReferralSchema.statics.getNetworkStatistics = async function() {
  const stats = await this.aggregate([
    { $match: { validationStatus: { $ne: 'banned' } } },
    {
      $group: {
        _id: null,
        totalUsers: { $sum: 1 },
        totalNetworkSize: { $sum: '$stats.totalNetworkSize' },
        averageQuality: { $avg: '$stats.networkQualityScore' },
        totalEarnings: { $sum: '$lifetimeEarnings' }
      }
    }
  ]);
  
  return stats[0] || {};
};

// Pre-save middleware
ReferralSchema.pre('save', async function(next) {
  if (this.isModified('totalRP')) {
    await this.checkTierUpgrade();
  }
  
  if (this.isModified('directReferrals') || this.isModified('referralNetwork')) {
    await this.updateNetworkStats();
    await this.validateNetworkQuality();
  }
  
  next();
});

// Post-save middleware
ReferralSchema.post('save', function() {
  // Emit events for real-time updates
  if (this.isModified('currentTier')) {
    // Emit tier upgrade event
    this.constructor.emit('tierUpgrade', {
      userId: this.userId,
      newTier: this.currentTier.name,
      timestamp: new Date()
    });
  }
});

export const Referral = model<IReferral>('Referral', ReferralSchema);
