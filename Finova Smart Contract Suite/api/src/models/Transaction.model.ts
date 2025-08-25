import { Schema, model, Document, Types } from 'mongoose';
import { v4 as uuidv4 } from 'uuid';

// Transaction types enum
export enum TransactionType {
  MINING_REWARD = 'mining_reward',
  XP_BONUS = 'xp_bonus',
  RP_BONUS = 'rp_bonus',
  STAKING_REWARD = 'staking_reward',
  NFT_PURCHASE = 'nft_purchase',
  NFT_SALE = 'nft_sale',
  TOKEN_TRANSFER = 'token_transfer',
  DEFI_SWAP = 'defi_swap',
  BRIDGE_LOCK = 'bridge_lock',
  BRIDGE_UNLOCK = 'bridge_unlock',
  REFERRAL_BONUS = 'referral_bonus',
  GUILD_REWARD = 'guild_reward',
  SPECIAL_CARD_BURN = 'special_card_burn',
  WHALE_TAX = 'whale_tax',
  TRANSACTION_FEE = 'transaction_fee',
  GOVERNANCE_VOTE = 'governance_vote',
  PREMIUM_SUBSCRIPTION = 'premium_subscription',
  BRAND_PARTNERSHIP = 'brand_partnership',
  E_WALLET_DEPOSIT = 'e_wallet_deposit',
  E_WALLET_WITHDRAWAL = 'e_wallet_withdrawal'
}

// Transaction status enum
export enum TransactionStatus {
  PENDING = 'pending',
  PROCESSING = 'processing',
  COMPLETED = 'completed',
  FAILED = 'failed',
  CANCELLED = 'cancelled',
  EXPIRED = 'expired'
}

// Token types
export enum TokenType {
  FIN = 'FIN',
  SFIN = 'sFIN',
  USDFIN = 'USDfin',
  SUSDFIN = 'sUSDfin',
  SOL = 'SOL',
  USDC = 'USDC'
}

// Platform source enum
export enum PlatformSource {
  FINOVA_APP = 'finova_app',
  INSTAGRAM = 'instagram',
  TIKTOK = 'tiktok',
  YOUTUBE = 'youtube',
  FACEBOOK = 'facebook',
  TWITTER = 'twitter',
  DEFI_PROTOCOL = 'defi_protocol',
  NFT_MARKETPLACE = 'nft_marketplace',
  E_WALLET = 'e_wallet',
  BLOCKCHAIN = 'blockchain'
}

// Mining calculation details interface
interface MiningCalculation {
  baseRate: number;
  pioneerBonus: number;
  referralBonus: number;
  securityBonus: number;
  regressionFactor: number;
  xpMultiplier: number;
  rpMultiplier: number;
  qualityScore: number;
  stakingBonus: number;
  cardBonus: number;
  guildBonus: number;
  networkPhase: number;
  totalMultiplier: number;
}

// XP calculation details interface
interface XPCalculation {
  baseXP: number;
  platformMultiplier: number;
  qualityScore: number;
  streakBonus: number;
  levelProgression: number;
  cardBonus: number;
  stakingBonus: number;
  guildBonus: number;
  viralBonus: number;
  totalXP: number;
}

// RP calculation details interface
interface RPCalculation {
  directReferralPoints: number;
  indirectNetworkPoints: number;
  networkQualityBonus: number;
  tierMultiplier: number;
  activityBonus: number;
  regressionFactor: number;
  totalRP: number;
}

// Blockchain transaction details
interface BlockchainDetails {
  transactionHash: string;
  blockNumber: number;
  blockHash: string;
  gasUsed: number;
  gasPrice: number;
  confirmations: number;
  networkFee: number;
  programId?: string;
  instructionIndex?: number;
  logMessages?: string[];
}

// NFT transaction details
interface NFTDetails {
  tokenId: string;
  contractAddress: string;
  metadata: {
    name: string;
    description: string;
    image: string;
    attributes: Array<{
      trait_type: string;
      value: string | number;
      rarity?: number;
    }>;
    rarity: string;
    collection: string;
  };
  marketplaceData?: {
    listingPrice: number;
    royaltyFee: number;
    marketplaceFee: number;
    seller?: string;
    buyer?: string;
  };
}

// DeFi transaction details
interface DeFiDetails {
  protocol: string;
  poolAddress?: string;
  liquidityTokens?: {
    tokenA: { symbol: string; amount: number; };
    tokenB: { symbol: string; amount: number; };
  };
  swapDetails?: {
    fromToken: string;
    toToken: string;
    fromAmount: number;
    toAmount: number;
    slippage: number;
    priceImpact: number;
  };
  yieldFarmData?: {
    farmAddress: string;
    stakingDuration: number;
    apy: number;
    rewards: Array<{ token: string; amount: number; }>;
  };
}

// Anti-bot verification details
interface AntiBotVerification {
  humanProbabilityScore: number;
  biometricVerified: boolean;
  deviceFingerprint: string;
  ipAddress: string;
  geolocation: {
    country: string;
    region: string;
    city: string;
    coordinates?: [number, number];
  };
  behaviorAnalysis: {
    sessionDuration: number;
    clickPatterns: number[];
    typingPatterns: number[];
    navigationPatterns: string[];
    suspiciousActivities: string[];
  };
  riskScore: number;
  verificationStatus: 'verified' | 'pending' | 'flagged' | 'rejected';
}

// Main Transaction interface
export interface ITransaction extends Document {
  // Core identifiers
  transactionId: string;
  userId: Types.ObjectId;
  relatedUserId?: Types.ObjectId;
  sessionId: string;
  
  // Transaction details
  type: TransactionType;
  status: TransactionStatus;
  tokenType: TokenType;
  amount: number;
  fee: number;
  netAmount: number;
  
  // Platform and source information
  platformSource: PlatformSource;
  sourceActivity?: {
    activityType: string;
    activityId: string;
    contentId?: string;
    postUrl?: string;
    engagementMetrics?: {
      likes: number;
      shares: number;
      comments: number;
      views: number;
      reactions: { [key: string]: number };
    };
  };
  
  // Calculation details
  miningCalculation?: MiningCalculation;
  xpCalculation?: XPCalculation;
  rpCalculation?: RPCalculation;
  
  // Blockchain and technical details
  blockchainDetails?: BlockchainDetails;
  nftDetails?: NFTDetails;
  defiDetails?: DeFiDetails;
  antiBotVerification?: AntiBotVerification;
  
  // Wallet and payment information
  walletAddress: string;
  destinationWallet?: string;
  eWalletDetails?: {
    provider: 'ovo' | 'gopay' | 'dana' | 'shopeepay';
    accountNumber: string;
    recipientName: string;
    conversionRate: number;
    idrAmount: number;
  };
  
  // Referral and network information
  referralData?: {
    referrerId: Types.ObjectId;
    referralLevel: number;
    networkBonus: number;
    qualityScore: number;
  };
  
  // Guild and community information
  guildData?: {
    guildId: Types.ObjectId;
    guildBonus: number;
    competitionId?: string;
    teamContribution: number;
  };
  
  // Special features
  specialCards?: Array<{
    cardId: string;
    cardType: string;
    effect: string;
    multiplier: number;
    duration: number;
    usedAt: Date;
  }>;
  
  // Staking information
  stakingData?: {
    stakingTier: string;
    stakedAmount: number;
    stakingDuration: number;
    apy: number;
    lockPeriod: number;
    autoCompound: boolean;
  };
  
  // Quality and moderation
  qualityAssessment?: {
    aiScore: number;
    humanReview: boolean;
    contentFlags: string[];
    moderationStatus: 'approved' | 'pending' | 'rejected' | 'flagged';
    appealStatus?: 'none' | 'pending' | 'approved' | 'rejected';
  };
  
  // Metadata
  metadata: {
    userAgent: string;
    ipAddress: string;
    deviceInfo: {
      platform: string;
      os: string;
      browser: string;
      version: string;
    };
    appVersion: string;
    sdkVersion?: string;
    clientTimestamp: Date;
  };
  
  // Timestamps and tracking
  createdAt: Date;
  updatedAt: Date;
  completedAt?: Date;
  expiresAt?: Date;
  
  // Methods
  calculateRewards(): Promise<number>;
  validateTransaction(): Promise<boolean>;
  processAntiBot(): Promise<boolean>;
  updateStatus(status: TransactionStatus): Promise<void>;
}

// Transaction schema
const TransactionSchema = new Schema<ITransaction>({
  // Core identifiers
  transactionId: {
    type: String,
    required: true,
    unique: true,
    default: () => `txn_${uuidv4()}`,
    index: true
  },
  userId: {
    type: Schema.Types.ObjectId,
    ref: 'User',
    required: true,
    index: true
  },
  relatedUserId: {
    type: Schema.Types.ObjectId,
    ref: 'User',
    index: true
  },
  sessionId: {
    type: String,
    required: true,
    index: true
  },
  
  // Transaction details
  type: {
    type: String,
    enum: Object.values(TransactionType),
    required: true,
    index: true
  },
  status: {
    type: String,
    enum: Object.values(TransactionStatus),
    default: TransactionStatus.PENDING,
    index: true
  },
  tokenType: {
    type: String,
    enum: Object.values(TokenType),
    required: true
  },
  amount: {
    type: Number,
    required: true,
    min: 0,
    validate: {
      validator: (v: number) => v >= 0 && Number.isFinite(v),
      message: 'Amount must be a valid positive number'
    }
  },
  fee: {
    type: Number,
    default: 0,
    min: 0
  },
  netAmount: {
    type: Number,
    required: true,
    min: 0
  },
  
  // Platform and source
  platformSource: {
    type: String,
    enum: Object.values(PlatformSource),
    required: true,
    index: true
  },
  sourceActivity: {
    activityType: String,
    activityId: String,
    contentId: String,
    postUrl: String,
    engagementMetrics: {
      likes: { type: Number, default: 0 },
      shares: { type: Number, default: 0 },
      comments: { type: Number, default: 0 },
      views: { type: Number, default: 0 },
      reactions: { type: Map, of: Number }
    }
  },
  
  // Calculation details
  miningCalculation: {
    baseRate: Number,
    pioneerBonus: Number,
    referralBonus: Number,
    securityBonus: Number,
    regressionFactor: Number,
    xpMultiplier: Number,
    rpMultiplier: Number,
    qualityScore: Number,
    stakingBonus: Number,
    cardBonus: Number,
    guildBonus: Number,
    networkPhase: Number,
    totalMultiplier: Number
  },
  xpCalculation: {
    baseXP: Number,
    platformMultiplier: Number,
    qualityScore: Number,
    streakBonus: Number,
    levelProgression: Number,
    cardBonus: Number,
    stakingBonus: Number,
    guildBonus: Number,
    viralBonus: Number,
    totalXP: Number
  },
  rpCalculation: {
    directReferralPoints: Number,
    indirectNetworkPoints: Number,
    networkQualityBonus: Number,
    tierMultiplier: Number,
    activityBonus: Number,
    regressionFactor: Number,
    totalRP: Number
  },
  
  // Blockchain details
  blockchainDetails: {
    transactionHash: String,
    blockNumber: Number,
    blockHash: String,
    gasUsed: Number,
    gasPrice: Number,
    confirmations: { type: Number, default: 0 },
    networkFee: Number,
    programId: String,
    instructionIndex: Number,
    logMessages: [String]
  },
  
  // NFT details
  nftDetails: {
    tokenId: String,
    contractAddress: String,
    metadata: {
      name: String,
      description: String,
      image: String,
      attributes: [{
        trait_type: String,
        value: Schema.Types.Mixed,
        rarity: Number
      }],
      rarity: String,
      collection: String
    },
    marketplaceData: {
      listingPrice: Number,
      royaltyFee: Number,
      marketplaceFee: Number,
      seller: String,
      buyer: String
    }
  },
  
  // DeFi details
  defiDetails: {
    protocol: String,
    poolAddress: String,
    liquidityTokens: {
      tokenA: {
        symbol: String,
        amount: Number
      },
      tokenB: {
        symbol: String,
        amount: Number
      }
    },
    swapDetails: {
      fromToken: String,
      toToken: String,
      fromAmount: Number,
      toAmount: Number,
      slippage: Number,
      priceImpact: Number
    },
    yieldFarmData: {
      farmAddress: String,
      stakingDuration: Number,
      apy: Number,
      rewards: [{
        token: String,
        amount: Number
      }]
    }
  },
  
  // Anti-bot verification
  antiBotVerification: {
    humanProbabilityScore: {
      type: Number,
      min: 0,
      max: 1,
      required: true
    },
    biometricVerified: { type: Boolean, default: false },
    deviceFingerprint: String,
    ipAddress: String,
    geolocation: {
      country: String,
      region: String,
      city: String,
      coordinates: [Number]
    },
    behaviorAnalysis: {
      sessionDuration: Number,
      clickPatterns: [Number],
      typingPatterns: [Number],
      navigationPatterns: [String],
      suspiciousActivities: [String]
    },
    riskScore: {
      type: Number,
      min: 0,
      max: 100,
      required: true
    },
    verificationStatus: {
      type: String,
      enum: ['verified', 'pending', 'flagged', 'rejected'],
      default: 'pending'
    }
  },
  
  // Wallet information
  walletAddress: {
    type: String,
    required: true,
    index: true
  },
  destinationWallet: String,
  eWalletDetails: {
    provider: {
      type: String,
      enum: ['ovo', 'gopay', 'dana', 'shopeepay']
    },
    accountNumber: String,
    recipientName: String,
    conversionRate: Number,
    idrAmount: Number
  },
  
  // Referral data
  referralData: {
    referrerId: {
      type: Schema.Types.ObjectId,
      ref: 'User'
    },
    referralLevel: Number,
    networkBonus: Number,
    qualityScore: Number
  },
  
  // Guild data
  guildData: {
    guildId: {
      type: Schema.Types.ObjectId,
      ref: 'Guild'
    },
    guildBonus: Number,
    competitionId: String,
    teamContribution: Number
  },
  
  // Special cards
  specialCards: [{
    cardId: String,
    cardType: String,
    effect: String,
    multiplier: Number,
    duration: Number,
    usedAt: Date
  }],
  
  // Staking data
  stakingData: {
    stakingTier: String,
    stakedAmount: Number,
    stakingDuration: Number,
    apy: Number,
    lockPeriod: Number,
    autoCompound: { type: Boolean, default: false }
  },
  
  // Quality assessment
  qualityAssessment: {
    aiScore: Number,
    humanReview: { type: Boolean, default: false },
    contentFlags: [String],
    moderationStatus: {
      type: String,
      enum: ['approved', 'pending', 'rejected', 'flagged'],
      default: 'pending'
    },
    appealStatus: {
      type: String,
      enum: ['none', 'pending', 'approved', 'rejected'],
      default: 'none'
    }
  },
  
  // Metadata
  metadata: {
    userAgent: { type: String, required: true },
    ipAddress: { type: String, required: true },
    deviceInfo: {
      platform: String,
      os: String,
      browser: String,
      version: String
    },
    appVersion: { type: String, required: true },
    sdkVersion: String,
    clientTimestamp: { type: Date, required: true }
  },
  
  // Timestamps
  completedAt: Date,
  expiresAt: Date
}, {
  timestamps: true,
  versionKey: false,
  toJSON: { virtuals: true },
  toObject: { virtuals: true }
});

// Indexes for performance
TransactionSchema.index({ userId: 1, createdAt: -1 });
TransactionSchema.index({ type: 1, status: 1, createdAt: -1 });
TransactionSchema.index({ 'blockchainDetails.transactionHash': 1 });
TransactionSchema.index({ walletAddress: 1, createdAt: -1 });
TransactionSchema.index({ platformSource: 1, createdAt: -1 });
TransactionSchema.index({ status: 1, expiresAt: 1 });

// Virtual fields
TransactionSchema.virtual('isExpired').get(function() {
  return this.expiresAt && this.expiresAt < new Date();
});

TransactionSchema.virtual('isPending').get(function() {
  return this.status === TransactionStatus.PENDING;
});

TransactionSchema.virtual('isCompleted').get(function() {
  return this.status === TransactionStatus.COMPLETED;
});

// Instance methods
TransactionSchema.methods.calculateRewards = async function(): Promise<number> {
  let totalReward = this.amount;
  
  if (this.miningCalculation) {
    totalReward *= this.miningCalculation.totalMultiplier;
  }
  
  if (this.xpCalculation) {
    totalReward += this.xpCalculation.totalXP * 0.001; // XP to FIN conversion
  }
  
  if (this.rpCalculation) {
    totalReward += this.rpCalculation.totalRP * 0.0005; // RP to FIN conversion
  }
  
  return Math.max(0, totalReward - this.fee);
};

TransactionSchema.methods.validateTransaction = async function(): Promise<boolean> {
  // Basic validation
  if (!this.userId || !this.walletAddress || this.amount <= 0) {
    return false;
  }
  
  // Anti-bot validation
  if (this.antiBotVerification) {
    return this.antiBotVerification.humanProbabilityScore >= 0.7 && 
           this.antiBotVerification.riskScore <= 30;
  }
  
  return true;
};

TransactionSchema.methods.processAntiBot = async function(): Promise<boolean> {
  if (!this.antiBotVerification) {
    return false;
  }
  
  const verification = this.antiBotVerification;
  
  // Calculate final risk score
  let riskScore = verification.riskScore;
  
  if (verification.humanProbabilityScore < 0.5) riskScore += 30;
  if (!verification.biometricVerified) riskScore += 15;
  if (verification.behaviorAnalysis.suspiciousActivities.length > 3) riskScore += 20;
  
  verification.riskScore = Math.min(100, riskScore);
  
  if (verification.riskScore <= 30) {
    verification.verificationStatus = 'verified';
    return true;
  } else if (verification.riskScore <= 60) {
    verification.verificationStatus = 'flagged';
    return false;
  } else {
    verification.verificationStatus = 'rejected';
    return false;
  }
};

TransactionSchema.methods.updateStatus = async function(status: TransactionStatus): Promise<void> {
  this.status = status;
  if (status === TransactionStatus.COMPLETED) {
    this.completedAt = new Date();
  }
  await this.save();
};

// Static methods
TransactionSchema.statics.findByUser = function(userId: string, limit: number = 50) {
  return this.find({ userId })
    .sort({ createdAt: -1 })
    .limit(limit)
    .populate('relatedUserId', 'username email')
    .populate('referralData.referrerId', 'username')
    .populate('guildData.guildId', 'name');
};

TransactionSchema.statics.findPendingTransactions = function() {
  return this.find({ 
    status: TransactionStatus.PENDING,
    $or: [
      { expiresAt: { $exists: false } },
      { expiresAt: { $gt: new Date() } }
    ]
  });
};

TransactionSchema.statics.getTotalRewardsByUser = function(userId: string, startDate?: Date, endDate?: Date) {
  const match: any = { 
    userId,
    status: TransactionStatus.COMPLETED,
    type: { $in: [TransactionType.MINING_REWARD, TransactionType.XP_BONUS, TransactionType.RP_BONUS] }
  };
  
  if (startDate || endDate) {
    match.completedAt = {};
    if (startDate) match.completedAt.$gte = startDate;
    if (endDate) match.completedAt.$lte = endDate;
  }
  
  return this.aggregate([
    { $match: match },
    { $group: {
      _id: '$tokenType',
      totalAmount: { $sum: '$netAmount' },
      count: { $sum: 1 }
    }}
  ]);
};

// Pre-save middleware
TransactionSchema.pre('save', function(next) {
  if (this.isNew) {
    this.netAmount = this.amount - this.fee;
    
    // Set expiration for certain transaction types
    if ([TransactionType.NFT_PURCHASE, TransactionType.DEFI_SWAP].includes(this.type)) {
      this.expiresAt = new Date(Date.now() + 30 * 60 * 1000); // 30 minutes
    }
  }
  
  next();
});

// Post-save middleware for logging
TransactionSchema.post('save', function() {
  console.log(`Transaction ${this.transactionId} saved with status: ${this.status}`);
});

export const Transaction = model<ITransaction>('Transaction', TransactionSchema);
