import { Schema, model, Document, Types } from 'mongoose';
import { User } from './User.model';
import { Mining } from './Mining.model';
import { XP } from './XP.model';

export interface IGuildMember {
  userId: Types.ObjectId;
  joinedAt: Date;
  role: 'member' | 'officer' | 'master';
  contributionScore: number;
  weeklyXP: number;
  weeklyMining: number;
  isActive: boolean;
  lastActivityAt: Date;
}

export interface IGuildChallenge {
  challengeId: string;
  type: 'daily' | 'weekly' | 'monthly' | 'seasonal';
  name: string;
  description: string;
  targetMetric: 'xp' | 'mining' | 'social_posts' | 'referrals' | 'combined';
  targetValue: number;
  currentValue: number;
  startDate: Date;
  endDate: Date;
  reward: {
    type: 'fin' | 'xp_multiplier' | 'mining_boost' | 'nft' | 'special_card';
    value: number;
    duration?: number; // in hours for temporary boosts
  };
  participants: Types.ObjectId[];
  isActive: boolean;
  isCompleted: boolean;
  completedAt?: Date;
}

export interface IGuildWar {
  warId: string;
  opponentGuildId: Types.ObjectId;
  startDate: Date;
  endDate: Date;
  battleType: 'xp_battle' | 'mining_battle' | 'hybrid_battle' | 'social_engagement';
  ourScore: number;
  opponentScore: number;
  status: 'pending' | 'active' | 'completed' | 'cancelled';
  winner?: Types.ObjectId;
  rewards: {
    winnerReward: {
      treasuryAmount: number;
      xpMultiplier: number;
      duration: number; // hours
    };
    loserReward: {
      treasuryAmount: number;
      xpMultiplier: number;
      duration: number;
    };
  };
  participants: {
    ourMembers: Types.ObjectId[];
    opponentMembers: Types.ObjectId[];
  };
}

export interface IGuildTreasury {
  totalBalance: number; // $FIN
  weeklyContribution: number;
  monthlyContribution: number;
  distributedRewards: number;
  transactions: {
    type: 'contribution' | 'reward_distribution' | 'war_prize' | 'challenge_reward';
    amount: number;
    description: string;
    timestamp: Date;
    memberId?: Types.ObjectId;
  }[];
  autoDistribution: {
    enabled: boolean;
    percentage: number; // % of treasury to distribute weekly
    lastDistribution: Date;
  };
}

export interface IGuildStats {
  totalXP: number;
  totalMining: number;
  weeklyXP: number;
  weeklyMining: number;
  monthlyXP: number;
  monthlyMining: number;
  averageMemberLevel: number;
  retentionRate: number; // % of members active in last 30 days
  growthRate: number; // % member increase per month
  challengesCompleted: number;
  warsWon: number;
  warsLost: number;
  rank: number; // Global guild ranking
  powerScore: number; // Combined metric for guild strength
  lastUpdated: Date;
}

export interface IGuildSettings {
  isPrivate: boolean;
  requiresApproval: boolean;
  minimumLevel: number;
  minimumXP: number;
  autoKickInactiveDays: number;
  membershipFee: number; // $FIN
  language: string;
  timezone: string;
  allowedCountries: string[];
  features: {
    warParticipation: boolean;
    publicChallenges: boolean;
    crossGuildEvents: boolean;
    treasurySharing: boolean;
  };
}

export interface IGuild extends Document {
  name: string;
  description: string;
  tagline: string;
  founderId: Types.ObjectId;
  createdAt: Date;
  updatedAt: Date;
  
  // Member management
  members: IGuildMember[];
  maxMembers: number;
  memberCount: number;
  
  // Guild identity
  logo?: string;
  banner?: string;
  colors: {
    primary: string;
    secondary: string;
  };
  tags: string[];
  
  // Gamification
  level: number;
  experience: number;
  experienceToNext: number;
  tier: 'bronze' | 'silver' | 'gold' | 'platinum' | 'diamond' | 'mythic';
  
  // Activities
  challenges: IGuildChallenge[];
  activeWars: IGuildWar[];
  warHistory: IGuildWar[];
  
  // Economics
  treasury: IGuildTreasury;
  
  // Performance
  stats: IGuildStats;
  achievements: {
    achievementId: string;
    name: string;
    description: string;
    unlockedAt: Date;
    rarity: 'common' | 'rare' | 'epic' | 'legendary';
  }[];
  
  // Configuration
  settings: IGuildSettings;
  
  // Status
  isActive: boolean;
  isVerified: boolean;
  status: 'active' | 'inactive' | 'suspended' | 'disbanded';
  
  // Virtual fields and methods
  activeMemberCount: number;
  weeklyActivity: number;
  powerRanking: number;
}

const GuildMemberSchema = new Schema<IGuildMember>({
  userId: { type: Schema.Types.ObjectId, ref: 'User', required: true },
  joinedAt: { type: Date, default: Date.now },
  role: { 
    type: String, 
    enum: ['member', 'officer', 'master'], 
    default: 'member' 
  },
  contributionScore: { type: Number, default: 0 },
  weeklyXP: { type: Number, default: 0 },
  weeklyMining: { type: Number, default: 0 },
  isActive: { type: Boolean, default: true },
  lastActivityAt: { type: Date, default: Date.now }
});

const GuildChallengeSchema = new Schema<IGuildChallenge>({
  challengeId: { type: String, required: true, unique: true },
  type: { 
    type: String, 
    enum: ['daily', 'weekly', 'monthly', 'seasonal'], 
    required: true 
  },
  name: { type: String, required: true },
  description: { type: String, required: true },
  targetMetric: {
    type: String,
    enum: ['xp', 'mining', 'social_posts', 'referrals', 'combined'],
    required: true
  },
  targetValue: { type: Number, required: true },
  currentValue: { type: Number, default: 0 },
  startDate: { type: Date, required: true },
  endDate: { type: Date, required: true },
  reward: {
    type: {
      type: String,
      enum: ['fin', 'xp_multiplier', 'mining_boost', 'nft', 'special_card'],
      required: true
    },
    value: { type: Number, required: true },
    duration: { type: Number } // hours
  },
  participants: [{ type: Schema.Types.ObjectId, ref: 'User' }],
  isActive: { type: Boolean, default: true },
  isCompleted: { type: Boolean, default: false },
  completedAt: { type: Date }
});

const GuildWarSchema = new Schema<IGuildWar>({
  warId: { type: String, required: true, unique: true },
  opponentGuildId: { type: Schema.Types.ObjectId, ref: 'Guild', required: true },
  startDate: { type: Date, required: true },
  endDate: { type: Date, required: true },
  battleType: {
    type: String,
    enum: ['xp_battle', 'mining_battle', 'hybrid_battle', 'social_engagement'],
    required: true
  },
  ourScore: { type: Number, default: 0 },
  opponentScore: { type: Number, default: 0 },
  status: {
    type: String,
    enum: ['pending', 'active', 'completed', 'cancelled'],
    default: 'pending'
  },
  winner: { type: Schema.Types.ObjectId, ref: 'Guild' },
  rewards: {
    winnerReward: {
      treasuryAmount: { type: Number, required: true },
      xpMultiplier: { type: Number, required: true },
      duration: { type: Number, required: true }
    },
    loserReward: {
      treasuryAmount: { type: Number, required: true },
      xpMultiplier: { type: Number, required: true },
      duration: { type: Number, required: true }
    }
  },
  participants: {
    ourMembers: [{ type: Schema.Types.ObjectId, ref: 'User' }],
    opponentMembers: [{ type: Schema.Types.ObjectId, ref: 'User' }]
  }
});

const GuildTreasurySchema = new Schema<IGuildTreasury>({
  totalBalance: { type: Number, default: 0 },
  weeklyContribution: { type: Number, default: 0 },
  monthlyContribution: { type: Number, default: 0 },
  distributedRewards: { type: Number, default: 0 },
  transactions: [{
    type: {
      type: String,
      enum: ['contribution', 'reward_distribution', 'war_prize', 'challenge_reward'],
      required: true
    },
    amount: { type: Number, required: true },
    description: { type: String, required: true },
    timestamp: { type: Date, default: Date.now },
    memberId: { type: Schema.Types.ObjectId, ref: 'User' }
  }],
  autoDistribution: {
    enabled: { type: Boolean, default: false },
    percentage: { type: Number, default: 10 },
    lastDistribution: { type: Date }
  }
});

const GuildStatsSchema = new Schema<IGuildStats>({
  totalXP: { type: Number, default: 0 },
  totalMining: { type: Number, default: 0 },
  weeklyXP: { type: Number, default: 0 },
  weeklyMining: { type: Number, default: 0 },
  monthlyXP: { type: Number, default: 0 },
  monthlyMining: { type: Number, default: 0 },
  averageMemberLevel: { type: Number, default: 1 },
  retentionRate: { type: Number, default: 100 },
  growthRate: { type: Number, default: 0 },
  challengesCompleted: { type: Number, default: 0 },
  warsWon: { type: Number, default: 0 },
  warsLost: { type: Number, default: 0 },
  rank: { type: Number, default: 0 },
  powerScore: { type: Number, default: 0 },
  lastUpdated: { type: Date, default: Date.now }
});

const GuildSettingsSchema = new Schema<IGuildSettings>({
  isPrivate: { type: Boolean, default: false },
  requiresApproval: { type: Boolean, default: true },
  minimumLevel: { type: Number, default: 11 }, // Silver level requirement
  minimumXP: { type: Number, default: 1000 },
  autoKickInactiveDays: { type: Number, default: 30 },
  membershipFee: { type: Number, default: 0 },
  language: { type: String, default: 'en' },
  timezone: { type: String, default: 'UTC' },
  allowedCountries: { type: [String], default: [] },
  features: {
    warParticipation: { type: Boolean, default: true },
    publicChallenges: { type: Boolean, default: true },
    crossGuildEvents: { type: Boolean, default: true },
    treasurySharing: { type: Boolean, default: true }
  }
});

const GuildSchema = new Schema<IGuild>({
  name: {
    type: String,
    required: true,
    unique: true,
    trim: true,
    maxlength: 50,
    minlength: 3
  },
  description: {
    type: String,
    required: true,
    maxlength: 500
  },
  tagline: {
    type: String,
    maxlength: 100
  },
  founderId: {
    type: Schema.Types.ObjectId,
    ref: 'User',
    required: true
  },
  
  // Member management
  members: [GuildMemberSchema],
  maxMembers: { type: Number, default: 50 },
  memberCount: { type: Number, default: 1 },
  
  // Guild identity
  logo: { type: String },
  banner: { type: String },
  colors: {
    primary: { type: String, default: '#3B82F6' },
    secondary: { type: String, default: '#1E40AF' }
  },
  tags: [{ type: String, maxlength: 20 }],
  
  // Gamification
  level: { type: Number, default: 1 },
  experience: { type: Number, default: 0 },
  experienceToNext: { type: Number, default: 1000 },
  tier: {
    type: String,
    enum: ['bronze', 'silver', 'gold', 'platinum', 'diamond', 'mythic'],
    default: 'bronze'
  },
  
  // Activities
  challenges: [GuildChallengeSchema],
  activeWars: [GuildWarSchema],
  warHistory: [GuildWarSchema],
  
  // Economics
  treasury: { type: GuildTreasurySchema, default: {} },
  
  // Performance
  stats: { type: GuildStatsSchema, default: {} },
  achievements: [{
    achievementId: { type: String, required: true },
    name: { type: String, required: true },
    description: { type: String, required: true },
    unlockedAt: { type: Date, default: Date.now },
    rarity: {
      type: String,
      enum: ['common', 'rare', 'epic', 'legendary'],
      default: 'common'
    }
  }],
  
  // Configuration
  settings: { type: GuildSettingsSchema, default: {} },
  
  // Status
  isActive: { type: Boolean, default: true },
  isVerified: { type: Boolean, default: false },
  status: {
    type: String,
    enum: ['active', 'inactive', 'suspended', 'disbanded'],
    default: 'active'
  }
}, {
  timestamps: true,
  toJSON: { virtuals: true },
  toObject: { virtuals: true }
});

// Indexes for performance
GuildSchema.index({ name: 1 });
GuildSchema.index({ founderId: 1 });
GuildSchema.index({ 'members.userId': 1 });
GuildSchema.index({ level: -1, 'stats.powerScore': -1 });
GuildSchema.index({ 'stats.rank': 1 });
GuildSchema.index({ tier: 1, level: -1 });
GuildSchema.index({ status: 1, isActive: 1 });
GuildSchema.index({ createdAt: -1 });
GuildSchema.index({ 'settings.language': 1 });
GuildSchema.index({ tags: 1 });

// Virtual fields
GuildSchema.virtual('activeMemberCount').get(function() {
  const thirtyDaysAgo = new Date(Date.now() - 30 * 24 * 60 * 60 * 1000);
  return this.members.filter(member => 
    member.isActive && member.lastActivityAt > thirtyDaysAgo
  ).length;
});

GuildSchema.virtual('weeklyActivity').get(function() {
  const sevenDaysAgo = new Date(Date.now() - 7 * 24 * 60 * 60 * 1000);
  return this.members.filter(member => 
    member.lastActivityAt > sevenDaysAgo
  ).length;
});

GuildSchema.virtual('powerRanking').get(function() {
  // Complex formula combining multiple factors
  const memberQuality = this.stats.averageMemberLevel * this.activeMemberCount;
  const activityScore = this.stats.weeklyXP + this.stats.weeklyMining * 10;
  const achievementBonus = this.achievements.length * 100;
  const retentionBonus = this.stats.retentionRate * 10;
  
  return memberQuality + activityScore + achievementBonus + retentionBonus;
});

// Methods
GuildSchema.methods.addMember = async function(userId: Types.ObjectId, role: string = 'member') {
  if (this.memberCount >= this.maxMembers) {
    throw new Error('Guild is at maximum capacity');
  }
  
  const existingMember = this.members.find(m => m.userId.toString() === userId.toString());
  if (existingMember) {
    throw new Error('User is already a member');
  }

  this.members.push({
    userId,
    role,
    joinedAt: new Date(),
    contributionScore: 0,
    weeklyXP: 0,
    weeklyMining: 0,
    isActive: true,
    lastActivityAt: new Date()
  });
  
  this.memberCount++;
  await this.save();
};

GuildSchema.methods.removeMember = async function(userId: Types.ObjectId) {
  const memberIndex = this.members.findIndex(m => m.userId.toString() === userId.toString());
  if (memberIndex === -1) {
    throw new Error('User is not a member');
  }
  
  this.members.splice(memberIndex, 1);
  this.memberCount--;
  await this.save();
};

GuildSchema.methods.updateMemberRole = async function(userId: Types.ObjectId, newRole: string) {
  const member = this.members.find(m => m.userId.toString() === userId.toString());
  if (!member) {
    throw new Error('User is not a member');
  }
  
  member.role = newRole as any;
  member.lastActivityAt = new Date();
  await this.save();
};

GuildSchema.methods.calculatePowerScore = function() {
  const baseScore = this.level * 100;
  const memberScore = this.activeMemberCount * 50;
  const activityScore = (this.stats.weeklyXP / 100) + (this.stats.weeklyMining * 10);
  const achievementScore = this.achievements.length * 25;
  const warScore = (this.stats.warsWon * 100) - (this.stats.warsLost * 25);
  const retentionScore = this.stats.retentionRate * 5;
  
  return Math.round(baseScore + memberScore + activityScore + achievementScore + warScore + retentionScore);
};

GuildSchema.methods.updateStats = async function() {
  // This would be called periodically to update guild statistics
  const now = new Date();
  const weekAgo = new Date(now.getTime() - 7 * 24 * 60 * 60 * 1000);
  const monthAgo = new Date(now.getTime() - 30 * 24 * 60 * 60 * 1000);
  
  // Calculate weekly and monthly activity
  let weeklyXP = 0;
  let weeklyMining = 0;
  let totalLevels = 0;
  let activeMembers = 0;
  
  for (const member of this.members) {
    if (member.isActive) {
      weeklyXP += member.weeklyXP;
      weeklyMining += member.weeklyMining;
      totalLevels += await this.getMemberLevel(member.userId);
      
      if (member.lastActivityAt > weekAgo) {
        activeMembers++;
      }
    }
  }
  
  this.stats.weeklyXP = weeklyXP;
  this.stats.weeklyMining = weeklyMining;
  this.stats.averageMemberLevel = this.memberCount > 0 ? totalLevels / this.memberCount : 1;
  this.stats.retentionRate = this.memberCount > 0 ? (activeMembers / this.memberCount) * 100 : 100;
  this.stats.powerScore = this.calculatePowerScore();
  this.stats.lastUpdated = now;
  
  await this.save();
};

GuildSchema.methods.getMemberLevel = async function(userId: Types.ObjectId) {
  // This would fetch the user's XP level from the User or XP model
  // Placeholder implementation
  return 25; // Default level
};

GuildSchema.methods.distributeTreasuryRewards = async function() {
  if (!this.treasury.autoDistribution.enabled) return;
  
  const distributionAmount = this.treasury.totalBalance * (this.treasury.autoDistribution.percentage / 100);
  const activeMembers = this.members.filter(m => m.isActive);
  
  if (activeMembers.length === 0) return;
  
  const rewardPerMember = distributionAmount / activeMembers.length;
  
  for (const member of activeMembers) {
    // Update member's balance (this would integrate with the User model)
    this.treasury.transactions.push({
      type: 'reward_distribution',
      amount: rewardPerMember,
      description: `Weekly treasury distribution`,
      timestamp: new Date(),
      memberId: member.userId
    });
  }
  
  this.treasury.totalBalance -= distributionAmount;
  this.treasury.distributedRewards += distributionAmount;
  this.treasury.autoDistribution.lastDistribution = new Date();
  
  await this.save();
};

// Static methods
GuildSchema.statics.findByRank = function(rank: number) {
  return this.findOne({ 'stats.rank': rank });
};

GuildSchema.statics.getTopGuilds = function(limit: number = 10) {
  return this.find({ status: 'active' })
    .sort({ 'stats.powerScore': -1, level: -1 })
    .limit(limit)
    .populate('founderId', 'username avatar')
    .populate('members.userId', 'username avatar level');
};

GuildSchema.statics.searchGuilds = function(query: string, filters: any = {}) {
  const searchCriteria: any = {
    status: 'active',
    $or: [
      { name: { $regex: query, $options: 'i' } },
      { description: { $regex: query, $options: 'i' } },
      { tags: { $in: [query] } }
    ]
  };
  
  if (filters.tier) searchCriteria.tier = filters.tier;
  if (filters.minLevel) searchCriteria.level = { $gte: filters.minLevel };
  if (filters.language) searchCriteria['settings.language'] = filters.language;
  if (filters.hasOpenSlots) {
    searchCriteria.$expr = { $lt: ['$memberCount', '$maxMembers'] };
  }
  
  return this.find(searchCriteria)
    .sort({ 'stats.powerScore': -1 })
    .populate('founderId', 'username avatar');
};

// Pre-save middleware
GuildSchema.pre('save', function(next) {
  // Update tier based on level
  if (this.level >= 100) this.tier = 'mythic';
  else if (this.level >= 75) this.tier = 'diamond';
  else if (this.level >= 50) this.tier = 'platinum';
  else if (this.level >= 25) this.tier = 'gold';
  else if (this.level >= 10) this.tier = 'silver';
  else this.tier = 'bronze';
  
  // Calculate experience to next level
  this.experienceToNext = (this.level + 1) * 1000 - this.experience;
  
  next();
});

export const Guild = model<IGuild>('Guild', GuildSchema);
export default Guild;
