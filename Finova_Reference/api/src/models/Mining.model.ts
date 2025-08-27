import { Schema, model, Document } from 'mongoose';
import { IMiningActivity, IMiningBoost, IMiningStats, IMiningSession } from '../types/mining.types';

// Mining Activity Schema
const MiningActivitySchema = new Schema({
  userId: { type: String, required: true, index: true },
  sessionId: { type: String, required: true, index: true },
  activityType: { 
    type: String, 
    enum: ['social_post', 'engagement', 'referral', 'daily_login', 'special_event'],
    required: true 
  },
  platform: { 
    type: String, 
    enum: ['instagram', 'tiktok', 'youtube', 'facebook', 'twitter', 'app'],
    required: true 
  },
  baseReward: { type: Number, required: true, min: 0 },
  multipliers: {
    xpMultiplier: { type: Number, default: 1.0, min: 0.5, max: 5.0 },
    rpMultiplier: { type: Number, default: 1.0, min: 0.5, max: 3.0 },
    qualityScore: { type: Number, default: 1.0, min: 0.5, max: 2.0 },
    networkRegression: { type: Number, default: 1.0, min: 0.1, max: 1.0 },
    pioneerBonus: { type: Number, default: 1.0, min: 1.0, max: 2.0 }
  },
  finalReward: { type: Number, required: true, min: 0 },
  contentHash: { type: String, index: true }, // For quality/originality verification
  qualityMetrics: {
    originalityScore: { type: Number, min: 0, max: 1 },
    engagementPotential: { type: Number, min: 0, max: 1 },
    brandSafety: { type: Number, min: 0, max: 1 },
    humanGenerated: { type: Number, min: 0, max: 1 }
  },
  verified: { type: Boolean, default: false },
  verifiedAt: Date,
  createdAt: { type: Date, default: Date.now, index: true },
  processedAt: Date
}, {
  collection: 'mining_activities',
  timestamps: true
});

// Mining Session Schema
const MiningSessionSchema = new Schema({
  userId: { type: String, required: true, index: true },
  sessionId: { type: String, required: true, unique: true },
  startTime: { type: Date, required: true, default: Date.now },
  endTime: Date,
  duration: { type: Number, min: 0 }, // in seconds
  totalActivities: { type: Number, default: 0, min: 0 },
  totalRewards: { type: Number, default: 0, min: 0 },
  averageQuality: { type: Number, default: 0, min: 0, max: 1 },
  phase: { 
    type: String, 
    enum: ['finizen', 'growth', 'maturity', 'stability'],
    required: true 
  },
  baseMiningRate: { type: Number, required: true, min: 0 },
  activeBoosts: [{
    boostType: { type: String, enum: ['nft_card', 'staking', 'event', 'streak'] },
    multiplier: { type: Number, min: 1.0, max: 10.0 },
    duration: Number,
    expiresAt: Date
  }],
  suspiciousActivity: {
    score: { type: Number, default: 0, min: 0, max: 1 },
    flags: [{ type: String }],
    reviewed: { type: Boolean, default: false }
  },
  deviceFingerprint: String,
  ipAddress: String,
  userAgent: String,
  isActive: { type: Boolean, default: true },
  createdAt: { type: Date, default: Date.now, index: true }
}, {
  collection: 'mining_sessions',
  timestamps: true
});

// Mining Statistics Schema  
const MiningStatsSchema = new Schema({
  userId: { type: String, required: true, unique: true },
  totalMined: { type: Number, default: 0, min: 0 },
  totalActivities: { type: Number, default: 0, min: 0 },
  averageQuality: { type: Number, default: 0, min: 0, max: 1 },
  bestStreak: { type: Number, default: 0, min: 0 },
  currentStreak: { type: Number, default: 0, min: 0 },
  lastMiningDate: Date,
  miningPhase: { 
    type: String, 
    enum: ['finizen', 'growth', 'maturity', 'stability'],
    default: 'finizen' 
  },
  dailyStats: {
    today: { type: Number, default: 0, min: 0 },
    yesterday: { type: Number, default: 0, min: 0 },
    thisWeek: { type: Number, default: 0, min: 0 },
    thisMonth: { type: Number, default: 0, min: 0 }
  },
  platformBreakdown: {
    instagram: { type: Number, default: 0, min: 0 },
    tiktok: { type: Number, default: 0, min: 0 },
    youtube: { type: Number, default: 0, min: 0 },
    facebook: { type: Number, default: 0, min: 0 },
    twitter: { type: Number, default: 0, min: 0 },
    app: { type: Number, default: 0, min: 0 }
  },
  qualityHistory: [{
    date: { type: Date, required: true },
    score: { type: Number, required: true, min: 0, max: 1 },
    activities: { type: Number, required: true, min: 0 }
  }],
  regressionFactor: { type: Number, default: 1.0, min: 0.1, max: 1.0 },
  lastCalculatedAt: { type: Date, default: Date.now },
  createdAt: { type: Date, default: Date.now, index: true }
}, {
  collection: 'mining_stats',
  timestamps: true
});

// Mining Boost Schema
const MiningBoostSchema = new Schema({
  userId: { type: String, required: true, index: true },
  boostId: { type: String, required: true, unique: true },
  boostType: { 
    type: String, 
    enum: ['nft_card', 'staking', 'event', 'streak', 'referral', 'achievement'],
    required: true 
  },
  name: { type: String, required: true },
  description: String,
  multiplier: { type: Number, required: true, min: 1.0, max: 10.0 },
  duration: { type: Number, required: true, min: 0 }, // in seconds
  startTime: { type: Date, required: true, default: Date.now },
  endTime: { type: Date, required: true },
  isActive: { type: Boolean, default: true },
  stackable: { type: Boolean, default: false },
  maxStack: { type: Number, default: 1, min: 1 },
  currentStack: { type: Number, default: 1, min: 1 },
  source: {
    type: { type: String, enum: ['nft', 'staking', 'achievement', 'event'] },
    id: String,
    metadata: Schema.Types.Mixed
  },
  activatedAt: { type: Date, default: Date.now },
  lastUsedAt: Date,
  totalRewardsBoosted: { type: Number, default: 0, min: 0 },
  createdAt: { type: Date, default: Date.now, index: true }
}, {
  collection: 'mining_boosts',
  timestamps: true
});

// Indexes for optimization
MiningActivitySchema.index({ userId: 1, createdAt: -1 });
MiningActivitySchema.index({ sessionId: 1, createdAt: -1 });
MiningActivitySchema.index({ verified: 1, createdAt: -1 });
MiningActivitySchema.index({ 'qualityMetrics.originalityScore': -1 });

MiningSessionSchema.index({ userId: 1, startTime: -1 });
MiningSessionSchema.index({ isActive: 1, userId: 1 });
MiningSessionSchema.index({ 'suspiciousActivity.score': -1 });

MiningStatsSchema.index({ userId: 1 }, { unique: true });
MiningStatsSchema.index({ totalMined: -1 });
MiningStatsSchema.index({ lastCalculatedAt: 1 });

MiningBoostSchema.index({ userId: 1, isActive: 1 });
MiningBoostSchema.index({ endTime: 1, isActive: 1 });
MiningBoostSchema.index({ boostType: 1, userId: 1 });

// Static methods for Mining Calculations
MiningActivitySchema.statics.calculateMiningReward = function(
  baseRate: number,
  userLevel: number,
  rpTier: number,
  qualityScore: number,
  totalHoldings: number,
  totalUsers: number
): number {
  // Finizen bonus calculation
  const fionizenBonus = Math.max(1.0, 2.0 - (totalUsers / 1000000));
  
  // XP level multiplier
  const xpMultiplier = 1.0 + (userLevel * 0.02); // 2% per level
  
  // RP tier multiplier
  const rpMultiplier = 1.0 + (rpTier * 0.1); // 10% per tier
  
  // Exponential regression factor
  const regressionFactor = Math.exp(-0.001 * totalHoldings);
  
  // Final calculation
  const finalReward = baseRate * fionizenBonus * xpMultiplier * 
                     rpMultiplier * qualityScore * regressionFactor;
  
  return Math.max(0.001, finalReward); // Minimum reward
};

MiningActivitySchema.statics.getCurrentPhaseRate = function(totalUsers: number): number {
  if (totalUsers < 100000) return 0.1; // Finizen phase
  if (totalUsers < 1000000) return 0.05; // Growth phase  
  if (totalUsers < 10000000) return 0.025; // Maturity phase
  return 0.01; // Stability phase
};

MiningActivitySchema.statics.getMiningPhase = function(totalUsers: number): string {
  if (totalUsers < 100000) return 'finizen';
  if (totalUsers < 1000000) return 'growth';
  if (totalUsers < 10000000) return 'maturity';
  return 'stability';
};

// Instance methods
MiningSessionSchema.methods.addActivity = async function(activity: IMiningActivity) {
  this.totalActivities += 1;
  this.totalRewards += activity.finalReward;
  
  // Update average quality
  const currentTotal = this.averageQuality * (this.totalActivities - 1);
  const activityQuality = activity.qualityMetrics?.originalityScore || 0;
  this.averageQuality = (currentTotal + activityQuality) / this.totalActivities;
  
  await this.save();
};

MiningSessionSchema.methods.checkSuspiciousActivity = function(): boolean {
  // Check for unusual patterns
  const avgTimeBetweenActivities = this.duration / Math.max(1, this.totalActivities);
  const isToolFast = avgTimeBetweenActivities < 5; // Less than 5 seconds
  const tooManyActivities = this.totalActivities > 100; // More than 100 activities per session
  const lowQuality = this.averageQuality < 0.3;
  
  let suspiciousScore = 0;
  const flags: string[] = [];
  
  if (isToolFast) {
    suspiciousScore += 0.4;
    flags.push('rapid_activity');
  }
  
  if (tooManyActivities) {
    suspiciousScore += 0.3;
    flags.push('excessive_activities');
  }
  
  if (lowQuality) {
    suspiciousScore += 0.3;
    flags.push('low_quality_content');
  }
  
  this.suspiciousActivity = {
    score: Math.min(1.0, suspiciousScore),
    flags,
    reviewed: false
  };
  
  return suspiciousScore > 0.7;
};

MiningBoostSchema.methods.isExpired = function(): boolean {
  return new Date() > this.endTime;
};

MiningBoostSchema.methods.activate = function() {
  this.isActive = true;
  this.activatedAt = new Date();
  this.lastUsedAt = new Date();
};

MiningBoostSchema.methods.deactivate = function() {
  this.isActive = false;
};

// Virtual fields
MiningSessionSchema.virtual('isExpired').get(function() {
  return this.endTime && new Date() > this.endTime;
});

MiningSessionSchema.virtual('remainingDuration').get(function() {
  if (!this.endTime) return null;
  return Math.max(0, this.endTime.getTime() - Date.now());
});

// Export models
export interface IMiningActivityDocument extends IMiningActivity, Document {}
export interface IMiningSessionDocument extends IMiningSession, Document {
  addActivity(activity: IMiningActivity): Promise<void>;
  checkSuspiciousActivity(): boolean;
}
export interface IMiningStatsDocument extends IMiningStats, Document {}
export interface IMiningBoostDocument extends IMiningBoost, Document {
  isExpired(): boolean;
  activate(): void;
  deactivate(): void;
}

export const MiningActivity = model<IMiningActivityDocument>('MiningActivity', MiningActivitySchema);
export const MiningSession = model<IMiningSessionDocument>('MiningSession', MiningSessionSchema);
export const MiningStats = model<IMiningStatsDocument>('MiningStats', MiningStatsSchema);
export const MiningBoost = model<IMiningBoostDocument>('MiningBoost', MiningBoostSchema);

export default {
  MiningActivity,
  MiningSession, 
  MiningStats,
  MiningBoost
};
