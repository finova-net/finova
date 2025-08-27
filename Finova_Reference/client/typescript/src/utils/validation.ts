// finova-net/finova/client/typescript/src/utils/validation.ts

/**
 * Finova Network - Client SDK Validation Utilities
 * Enterprise-grade validation for Social-Fi Super App
 * 
 * @version 3.0.0
 * @author Finova Network Team
 * @license MIT
 */

import { PublicKey } from '@solana/web3.js';
import { BN } from '@coral-xyz/anchor';

// ===== TYPES & INTERFACES =====

export interface ValidationResult {
  isValid: boolean;
  errors: string[];
  warnings?: string[];
}

export interface UserProfile {
  publicKey: string;
  username: string;
  email: string;
  phoneNumber?: string;
  referralCode?: string;
  socialAccounts: SocialAccount[];
}

export interface SocialAccount {
  platform: SocialPlatform;
  username: string;
  profileUrl: string;
  isVerified: boolean;
}

export interface MiningSession {
  userId: string;
  startTime: number;
  duration: number;
  baseRate: number;
  multipliers: MiningMultipliers;
}

export interface MiningMultipliers {
  xpMultiplier: number;
  rpMultiplier: number;
  stakingMultiplier: number;
  qualityMultiplier: number;
}

export interface XPActivity {
  type: ActivityType;
  platform: SocialPlatform;
  content: string;
  mediaUrls?: string[];
  engagement: EngagementMetrics;
}

export interface EngagementMetrics {
  likes: number;
  comments: number;
  shares: number;
  views: number;
  impressions?: number;
}

export interface ReferralData {
  referrerCode: string;
  refereePublicKey: string;
  level: number;
  networkSize: number;
  qualityScore: number;
}

export interface NFTMetadata {
  name: string;
  description: string;
  image: string;
  attributes: NFTAttribute[];
  rarity: NFTRarity;
  utility: NFTUtility;
}

export interface StakeData {
  amount: string; // BN string representation
  duration: number;
  tier: StakingTier;
  expectedApy: number;
}

// ===== ENUMS =====

export enum SocialPlatform {
  INSTAGRAM = 'instagram',
  TIKTOK = 'tiktok',
  YOUTUBE = 'youtube',
  FACEBOOK = 'facebook',
  TWITTER_X = 'twitter_x',
  LINKEDIN = 'linkedin'
}

export enum ActivityType {
  POST = 'post',
  COMMENT = 'comment',
  LIKE = 'like',
  SHARE = 'share',
  FOLLOW = 'follow',
  STORY = 'story',
  REEL = 'reel',
  VIDEO = 'video'
}

export enum NFTRarity {
  COMMON = 'common',
  UNCOMMON = 'uncommon',
  RARE = 'rare',
  EPIC = 'epic',
  LEGENDARY = 'legendary',
  MYTHIC = 'mythic'
}

export enum NFTUtility {
  MINING_BOOST = 'mining_boost',
  XP_ACCELERATOR = 'xp_accelerator',
  RP_AMPLIFIER = 'rp_amplifier',
  PROFILE_BADGE = 'profile_badge',
  ACCESS_CARD = 'access_card'
}

export enum StakingTier {
  BRONZE = 'bronze',
  SILVER = 'silver',
  GOLD = 'gold',
  PLATINUM = 'platinum',
  DIAMOND = 'diamond',
  MYTHIC = 'mythic'
}

// ===== CONSTANTS =====

export const VALIDATION_CONSTANTS = {
  // User validation
  USERNAME_MIN_LENGTH: 3,
  USERNAME_MAX_LENGTH: 30,
  EMAIL_REGEX: /^[^\s@]+@[^\s@]+\.[^\s@]+$/,
  PHONE_REGEX: /^(\+62|62|0)8[1-9][0-9]{6,9}$/,
  REFERRAL_CODE_LENGTH: 8,
  
  // Mining validation
  MIN_MINING_RATE: 0.001,
  MAX_MINING_RATE: 0.1,
  MAX_DAILY_MINING: 15.0,
  MIN_MINING_DURATION: 300, // 5 minutes
  MAX_MINING_DURATION: 86400, // 24 hours
  
  // XP validation
  MIN_XP_GAIN: 1,
  MAX_XP_GAIN: 2000,
  MAX_DAILY_XP: 10000,
  MIN_QUALITY_SCORE: 0.5,
  MAX_QUALITY_SCORE: 2.0,
  
  // RP validation
  MAX_REFERRAL_LEVELS: 3,
  MIN_NETWORK_QUALITY: 0.1,
  MAX_NETWORK_QUALITY: 1.0,
  MAX_DIRECT_REFERRALS: 1000,
  
  // Token validation
  FIN_DECIMALS: 9,
  MIN_STAKE_AMOUNT: 100,
  MAX_STAKE_AMOUNT: 1000000,
  MIN_STAKE_DURATION: 86400, // 1 day
  MAX_STAKE_DURATION: 31536000, // 1 year
  
  // Content validation
  MIN_CONTENT_LENGTH: 10,
  MAX_CONTENT_LENGTH: 2000,
  MAX_MEDIA_FILES: 10,
  MAX_FILE_SIZE: 50 * 1024 * 1024, // 50MB
  
  // Security
  MAX_API_CALLS_PER_MINUTE: 100,
  SESSION_TIMEOUT: 3600, // 1 hour
  MAX_LOGIN_ATTEMPTS: 5
} as const;

// ===== CORE VALIDATION CLASS =====

export class FinovaValidator {
  
  /**
   * Validate Solana public key
   */
  static validatePublicKey(pubkey: string): ValidationResult {
    const errors: string[] = [];
    
    try {
      if (!pubkey || typeof pubkey !== 'string') {
        errors.push('Public key must be a non-empty string');
        return { isValid: false, errors };
      }
      
      // Validate base58 format and length
      if (pubkey.length !== 44) {
        errors.push('Public key must be 44 characters long');
      }
      
      // Validate using Solana's PublicKey constructor
      new PublicKey(pubkey);
      
      return { isValid: errors.length === 0, errors };
    } catch (error) {
      errors.push(`Invalid public key format: ${error instanceof Error ? error.message : 'Unknown error'}`);
      return { isValid: false, errors };
    }
  }
  
  /**
   * Validate user profile data
   */
  static validateUserProfile(profile: Partial<UserProfile>): ValidationResult {
    const errors: string[] = [];
    const warnings: string[] = [];
    
    // Validate public key
    if (profile.publicKey) {
      const pubkeyResult = this.validatePublicKey(profile.publicKey);
      if (!pubkeyResult.isValid) {
        errors.push(...pubkeyResult.errors);
      }
    } else {
      errors.push('Public key is required');
    }
    
    // Validate username
    if (profile.username) {
      const usernameResult = this.validateUsername(profile.username);
      if (!usernameResult.isValid) {
        errors.push(...usernameResult.errors);
      }
    } else {
      errors.push('Username is required');
    }
    
    // Validate email
    if (profile.email) {
      const emailResult = this.validateEmail(profile.email);
      if (!emailResult.isValid) {
        errors.push(...emailResult.errors);
      }
    } else {
      errors.push('Email is required');
    }
    
    // Validate phone number (optional)
    if (profile.phoneNumber) {
      const phoneResult = this.validatePhoneNumber(profile.phoneNumber);
      if (!phoneResult.isValid) {
        warnings.push(...phoneResult.errors);
      }
    }
    
    // Validate referral code (optional)
    if (profile.referralCode) {
      const referralResult = this.validateReferralCode(profile.referralCode);
      if (!referralResult.isValid) {
        errors.push(...referralResult.errors);
      }
    }
    
    // Validate social accounts
    if (profile.socialAccounts && profile.socialAccounts.length > 0) {
      profile.socialAccounts.forEach((account, index) => {
        const accountResult = this.validateSocialAccount(account);
        if (!accountResult.isValid) {
          errors.push(...accountResult.errors.map(err => `Social account ${index + 1}: ${err}`));
        }
      });
    }
    
    return { isValid: errors.length === 0, errors, warnings };
  }
  
  /**
   * Validate mining session data
   */
  static validateMiningSession(session: Partial<MiningSession>): ValidationResult {
    const errors: string[] = [];
    
    // Validate user ID
    if (!session.userId) {
      errors.push('User ID is required');
    } else {
      const userIdResult = this.validatePublicKey(session.userId);
      if (!userIdResult.isValid) {
        errors.push('Invalid user ID format');
      }
    }
    
    // Validate timestamps
    if (session.startTime) {
      if (session.startTime <= 0 || session.startTime > Date.now()) {
        errors.push('Invalid start time');
      }
    } else {
      errors.push('Start time is required');
    }
    
    // Validate duration
    if (session.duration !== undefined) {
      if (session.duration < VALIDATION_CONSTANTS.MIN_MINING_DURATION ||
          session.duration > VALIDATION_CONSTANTS.MAX_MINING_DURATION) {
        errors.push(`Duration must be between ${VALIDATION_CONSTANTS.MIN_MINING_DURATION} and ${VALIDATION_CONSTANTS.MAX_MINING_DURATION} seconds`);
      }
    } else {
      errors.push('Duration is required');
    }
    
    // Validate base rate
    if (session.baseRate !== undefined) {
      if (session.baseRate < VALIDATION_CONSTANTS.MIN_MINING_RATE ||
          session.baseRate > VALIDATION_CONSTANTS.MAX_MINING_RATE) {
        errors.push(`Base rate must be between ${VALIDATION_CONSTANTS.MIN_MINING_RATE} and ${VALIDATION_CONSTANTS.MAX_MINING_RATE}`);
      }
    } else {
      errors.push('Base rate is required');
    }
    
    // Validate multipliers
    if (session.multipliers) {
      const multipliersResult = this.validateMiningMultipliers(session.multipliers);
      if (!multipliersResult.isValid) {
        errors.push(...multipliersResult.errors);
      }
    }
    
    return { isValid: errors.length === 0, errors };
  }
  
  /**
   * Validate XP activity
   */
  static validateXPActivity(activity: Partial<XPActivity>): ValidationResult {
    const errors: string[] = [];
    
    // Validate activity type
    if (!activity.type || !Object.values(ActivityType).includes(activity.type)) {
      errors.push('Valid activity type is required');
    }
    
    // Validate platform
    if (!activity.platform || !Object.values(SocialPlatform).includes(activity.platform)) {
      errors.push('Valid social platform is required');
    }
    
    // Validate content
    if (activity.content) {
      const contentResult = this.validateContent(activity.content);
      if (!contentResult.isValid) {
        errors.push(...contentResult.errors);
      }
    } else if (activity.type === ActivityType.POST || activity.type === ActivityType.COMMENT) {
      errors.push('Content is required for posts and comments');
    }
    
    // Validate media URLs
    if (activity.mediaUrls && activity.mediaUrls.length > 0) {
      if (activity.mediaUrls.length > VALIDATION_CONSTANTS.MAX_MEDIA_FILES) {
        errors.push(`Maximum ${VALIDATION_CONSTANTS.MAX_MEDIA_FILES} media files allowed`);
      }
      
      activity.mediaUrls.forEach((url, index) => {
        if (!this.isValidUrl(url)) {
          errors.push(`Invalid media URL at index ${index}`);
        }
      });
    }
    
    // Validate engagement metrics
    if (activity.engagement) {
      const engagementResult = this.validateEngagementMetrics(activity.engagement);
      if (!engagementResult.isValid) {
        errors.push(...engagementResult.errors);
      }
    }
    
    return { isValid: errors.length === 0, errors };
  }
  
  /**
   * Validate referral data
   */
  static validateReferralData(referral: Partial<ReferralData>): ValidationResult {
    const errors: string[] = [];
    
    // Validate referrer code
    if (referral.referrerCode) {
      const codeResult = this.validateReferralCode(referral.referrerCode);
      if (!codeResult.isValid) {
        errors.push(...codeResult.errors);
      }
    } else {
      errors.push('Referrer code is required');
    }
    
    // Validate referee public key
    if (referral.refereePublicKey) {
      const pubkeyResult = this.validatePublicKey(referral.refereePublicKey);
      if (!pubkeyResult.isValid) {
        errors.push('Invalid referee public key');
      }
    } else {
      errors.push('Referee public key is required');
    }
    
    // Validate level
    if (referral.level !== undefined) {
      if (referral.level < 1 || referral.level > VALIDATION_CONSTANTS.MAX_REFERRAL_LEVELS) {
        errors.push(`Referral level must be between 1 and ${VALIDATION_CONSTANTS.MAX_REFERRAL_LEVELS}`);
      }
    } else {
      errors.push('Referral level is required');
    }
    
    // Validate network size
    if (referral.networkSize !== undefined) {
      if (referral.networkSize < 0 || referral.networkSize > VALIDATION_CONSTANTS.MAX_DIRECT_REFERRALS) {
        errors.push(`Network size must be between 0 and ${VALIDATION_CONSTANTS.MAX_DIRECT_REFERRALS}`);
      }
    }
    
    // Validate quality score
    if (referral.qualityScore !== undefined) {
      if (referral.qualityScore < VALIDATION_CONSTANTS.MIN_NETWORK_QUALITY ||
          referral.qualityScore > VALIDATION_CONSTANTS.MAX_NETWORK_QUALITY) {
        errors.push(`Quality score must be between ${VALIDATION_CONSTANTS.MIN_NETWORK_QUALITY} and ${VALIDATION_CONSTANTS.MAX_NETWORK_QUALITY}`);
      }
    }
    
    return { isValid: errors.length === 0, errors };
  }
  
  /**
   * Validate NFT metadata
   */
  static validateNFTMetadata(metadata: Partial<NFTMetadata>): ValidationResult {
    const errors: string[] = [];
    
    // Validate name
    if (!metadata.name || metadata.name.trim().length === 0) {
      errors.push('NFT name is required');
    } else if (metadata.name.length > 100) {
      errors.push('NFT name must be 100 characters or less');
    }
    
    // Validate description
    if (!metadata.description || metadata.description.trim().length === 0) {
      errors.push('NFT description is required');
    } else if (metadata.description.length > 500) {
      errors.push('NFT description must be 500 characters or less');
    }
    
    // Validate image URL
    if (!metadata.image) {
      errors.push('NFT image URL is required');
    } else if (!this.isValidUrl(metadata.image)) {
      errors.push('Invalid NFT image URL');
    }
    
    // Validate rarity
    if (!metadata.rarity || !Object.values(NFTRarity).includes(metadata.rarity)) {
      errors.push('Valid NFT rarity is required');
    }
    
    // Validate utility
    if (!metadata.utility || !Object.values(NFTUtility).includes(metadata.utility)) {
      errors.push('Valid NFT utility is required');
    }
    
    // Validate attributes
    if (metadata.attributes && metadata.attributes.length > 0) {
      metadata.attributes.forEach((attr, index) => {
        if (!attr.trait_type || !attr.value) {
          errors.push(`Invalid attribute at index ${index}: trait_type and value are required`);
        }
      });
    }
    
    return { isValid: errors.length === 0, errors };
  }
  
  /**
   * Validate staking data
   */
  static validateStakeData(stake: Partial<StakeData>): ValidationResult {
    const errors: string[] = [];
    
    // Validate amount
    if (!stake.amount) {
      errors.push('Stake amount is required');
    } else {
      try {
        const amount = new BN(stake.amount);
        const minAmount = new BN(VALIDATION_CONSTANTS.MIN_STAKE_AMOUNT).mul(new BN(10).pow(new BN(VALIDATION_CONSTANTS.FIN_DECIMALS)));
        const maxAmount = new BN(VALIDATION_CONSTANTS.MAX_STAKE_AMOUNT).mul(new BN(10).pow(new BN(VALIDATION_CONSTANTS.FIN_DECIMALS)));
        
        if (amount.lt(minAmount)) {
          errors.push(`Minimum stake amount is ${VALIDATION_CONSTANTS.MIN_STAKE_AMOUNT} FIN`);
        }
        if (amount.gt(maxAmount)) {
          errors.push(`Maximum stake amount is ${VALIDATION_CONSTANTS.MAX_STAKE_AMOUNT} FIN`);
        }
      } catch (error) {
        errors.push('Invalid stake amount format');
      }
    }
    
    // Validate duration
    if (stake.duration !== undefined) {
      if (stake.duration < VALIDATION_CONSTANTS.MIN_STAKE_DURATION ||
          stake.duration > VALIDATION_CONSTANTS.MAX_STAKE_DURATION) {
        errors.push(`Stake duration must be between ${VALIDATION_CONSTANTS.MIN_STAKE_DURATION} and ${VALIDATION_CONSTANTS.MAX_STAKE_DURATION} seconds`);
      }
    } else {
      errors.push('Stake duration is required');
    }
    
    // Validate tier
    if (stake.tier && !Object.values(StakingTier).includes(stake.tier)) {
      errors.push('Valid staking tier is required');
    }
    
    // Validate expected APY
    if (stake.expectedApy !== undefined) {
      if (stake.expectedApy < 0 || stake.expectedApy > 100) {
        errors.push('Expected APY must be between 0% and 100%');
      }
    }
    
    return { isValid: errors.length === 0, errors };
  }
  
  // ===== HELPER VALIDATION METHODS =====
  
  private static validateUsername(username: string): ValidationResult {
    const errors: string[] = [];
    
    if (username.length < VALIDATION_CONSTANTS.USERNAME_MIN_LENGTH) {
      errors.push(`Username must be at least ${VALIDATION_CONSTANTS.USERNAME_MIN_LENGTH} characters`);
    }
    
    if (username.length > VALIDATION_CONSTANTS.USERNAME_MAX_LENGTH) {
      errors.push(`Username must be no more than ${VALIDATION_CONSTANTS.USERNAME_MAX_LENGTH} characters`);
    }
    
    if (!/^[a-zA-Z0-9_]+$/.test(username)) {
      errors.push('Username can only contain letters, numbers, and underscores');
    }
    
    if (/^[0-9_]/.test(username)) {
      errors.push('Username cannot start with a number or underscore');
    }
    
    return { isValid: errors.length === 0, errors };
  }
  
  private static validateEmail(email: string): ValidationResult {
    const errors: string[] = [];
    
    if (!VALIDATION_CONSTANTS.EMAIL_REGEX.test(email)) {
      errors.push('Invalid email format');
    }
    
    if (email.length > 254) {
      errors.push('Email address too long');
    }
    
    return { isValid: errors.length === 0, errors };
  }
  
  private static validatePhoneNumber(phone: string): ValidationResult {
    const errors: string[] = [];
    
    if (!VALIDATION_CONSTANTS.PHONE_REGEX.test(phone)) {
      errors.push('Invalid Indonesian phone number format');
    }
    
    return { isValid: errors.length === 0, errors };
  }
  
  private static validateReferralCode(code: string): ValidationResult {
    const errors: string[] = [];
    
    if (code.length !== VALIDATION_CONSTANTS.REFERRAL_CODE_LENGTH) {
      errors.push(`Referral code must be exactly ${VALIDATION_CONSTANTS.REFERRAL_CODE_LENGTH} characters`);
    }
    
    if (!/^[A-Z0-9]+$/.test(code)) {
      errors.push('Referral code can only contain uppercase letters and numbers');
    }
    
    return { isValid: errors.length === 0, errors };
  }
  
  private static validateSocialAccount(account: Partial<SocialAccount>): ValidationResult {
    const errors: string[] = [];
    
    if (!account.platform || !Object.values(SocialPlatform).includes(account.platform)) {
      errors.push('Valid social platform is required');
    }
    
    if (!account.username || account.username.trim().length === 0) {
      errors.push('Social username is required');
    }
    
    if (!account.profileUrl || !this.isValidUrl(account.profileUrl)) {
      errors.push('Valid profile URL is required');
    }
    
    return { isValid: errors.length === 0, errors };
  }
  
  private static validateMiningMultipliers(multipliers: Partial<MiningMultipliers>): ValidationResult {
    const errors: string[] = [];
    
    const validateMultiplier = (value: number | undefined, name: string, min = 0.1, max = 5.0) => {
      if (value !== undefined) {
        if (value < min || value > max) {
          errors.push(`${name} must be between ${min} and ${max}`);
        }
      }
    };
    
    validateMultiplier(multipliers.xpMultiplier, 'XP multiplier');
    validateMultiplier(multipliers.rpMultiplier, 'RP multiplier');
    validateMultiplier(multipliers.stakingMultiplier, 'Staking multiplier');
    validateMultiplier(multipliers.qualityMultiplier, 'Quality multiplier', 0.5, 2.0);
    
    return { isValid: errors.length === 0, errors };
  }
  
  private static validateContent(content: string): ValidationResult {
    const errors: string[] = [];
    
    if (content.length < VALIDATION_CONSTANTS.MIN_CONTENT_LENGTH) {
      errors.push(`Content must be at least ${VALIDATION_CONSTANTS.MIN_CONTENT_LENGTH} characters`);
    }
    
    if (content.length > VALIDATION_CONSTANTS.MAX_CONTENT_LENGTH) {
      errors.push(`Content must be no more than ${VALIDATION_CONSTANTS.MAX_CONTENT_LENGTH} characters`);
    }
    
    // Check for suspicious patterns (basic spam detection)
    const suspiciousPatterns = [
      /(.)\1{10,}/, // Repeated characters
      /https?:\/\/[^\s]+/gi, // Multiple URLs (limit to 3)
    ];
    
    const urlMatches = content.match(suspiciousPatterns[1]);
    if (urlMatches && urlMatches.length > 3) {
      errors.push('Content contains too many URLs');
    }
    
    if (suspiciousPatterns[0].test(content)) {
      errors.push('Content contains suspicious repeated characters');
    }
    
    return { isValid: errors.length === 0, errors };
  }
  
  private static validateEngagementMetrics(metrics: Partial<EngagementMetrics>): ValidationResult {
    const errors: string[] = [];
    
    const validateMetric = (value: number | undefined, name: string) => {
      if (value !== undefined && (value < 0 || !Number.isInteger(value))) {
        errors.push(`${name} must be a non-negative integer`);
      }
    };
    
    validateMetric(metrics.likes, 'Likes');
    validateMetric(metrics.comments, 'Comments');
    validateMetric(metrics.shares, 'Shares');
    validateMetric(metrics.views, 'Views');
    validateMetric(metrics.impressions, 'Impressions');
    
    // Logical validation
    if (metrics.views && metrics.impressions && metrics.views > metrics.impressions) {
      errors.push('Views cannot exceed impressions');
    }
    
    return { isValid: errors.length === 0, errors };
  }
  
  private static isValidUrl(url: string): boolean {
    try {
      const urlObject = new URL(url);
      return ['http:', 'https:'].includes(urlObject.protocol);
    } catch {
      return false;
    }
  }
}

// ===== UTILITY FUNCTIONS =====

/**
 * Generate secure referral code
 */
export function generateReferralCode(): string {
  const chars = 'ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789';
  let result = '';
  for (let i = 0; i < VALIDATION_CONSTANTS.REFERRAL_CODE_LENGTH; i++) {
    result += chars.charAt(Math.floor(Math.random() * chars.length));
  }
  return result;
}

/**
 * Calculate mining rate with all multipliers
 */
export function calculateFinalMiningRate(
  baseRate: number,
  multipliers: MiningMultipliers,
  networkRegression: number = 1.0
): number {
  return baseRate * 
    multipliers.xpMultiplier * 
    multipliers.rpMultiplier * 
    multipliers.stakingMultiplier * 
    multipliers.qualityMultiplier * 
    networkRegression;
}

/**
 * Calculate XP with quality and streak bonuses
 */
export function calculateXPGain(
  baseXP: number,
  platformMultiplier: number,
  qualityScore: number,
  streakBonus: number,
  levelProgression: number
): number {
  return Math.floor(
    baseXP * platformMultiplier * qualityScore * streakBonus * levelProgression
  );
}

/**
 * Validate and sanitize user input
 */
export function sanitizeInput(input: string, maxLength: number = 1000): string {
  return input
    .trim()
    .slice(0, maxLength)
    .replace(/[<>\"']/g, '') // Remove potentially dangerous characters
    .replace(/\s+/g, ' '); // Normalize whitespace
}

/**
 * Check if user has exceeded rate limits
 */
export function checkRateLimit(
  userId: string,
  action: string,
  timestamps: number[],
  maxRequests: number = VALIDATION_CONSTANTS.MAX_API_CALLS_PER_MINUTE,
  windowMs: number = 60000
): boolean {
  const now = Date.now();
  const windowStart = now - windowMs;
  const recentRequests = timestamps.filter(timestamp => timestamp > windowStart);
  
  return recentRequests.length < maxRequests;
}

/**
 * Validate transaction signature
 */
export function validateTransactionSignature(signature: string): ValidationResult {
  const errors: string[] = [];
  
  if (!signature || typeof signature !== 'string') {
    errors.push('Transaction signature must be a non-empty string');
  } else if (signature.length !== 88) {
    errors.push('Transaction signature must be 88 characters long');
  } else if (!/^[1-9A-HJ-NP-Za-km-z]+$/.test(signature)) {
    errors.push('Invalid transaction signature format (must be base58)');
  }
  
  return { isValid: errors.length === 0, errors };
}

/**
 * Batch validation for multiple items
 */
export function batchValidate<T>(
  items: T[],
  validator: (item: T) => ValidationResult
): { validItems: T[]; invalidItems: { item: T; errors: string[] }[] } {
  const validItems: T[] = [];
  const invalidItems: { item: T; errors: string[] }[] = [];
  
  items.forEach(item => {
    const result = validator(item);
    if (result.isValid) {
      validItems.push(item);
    } else {
      invalidItems.push({ item, errors: result.errors });
    }
  });
  
  return { validItems, invalidItems };
}

// ===== SECURITY HELPERS =====

/**
 * Detect potential bot behavior patterns
 */
export function detectBotBehavior(
  userActions: Array<{ timestamp: number; type: string; duration?: number }>
): { isSuspicious: boolean; confidence: number; reasons: string[] } {
  const reasons: string[] = [];
  let suspiciousCount = 0;
  
  // Check for inhuman timing patterns
  const intervals = userActions.slice(1).map((action, i) => 
    action.timestamp - userActions[i].timestamp
  );
  
  const avgInterval = intervals.reduce((sum, interval) => sum + interval, 0) / intervals.length;
  const variance = intervals.reduce((sum, interval) => sum + Math.pow(interval - avgInterval, 2), 0) / intervals.length;
  
  // Too consistent timing (likely bot)
  if (variance < avgInterval * 0.1 && intervals.length > 10) {
    reasons.push('Suspiciously consistent timing patterns');
    suspiciousCount++;
  }
  
  // Too fast actions
  const fastActions = intervals.filter(interval => interval < 1000).length;
  if (fastActions > intervals.length * 0.5) {
    reasons.push('Too many rapid-fire actions');
    suspiciousCount++;
  }
  
  // Check for repetitive patterns
  const actionTypes = userActions.map(action => action.type);
  const uniqueTypes = new Set(actionTypes).size;
  if (uniqueTypes < actionTypes.length * 0.3 && actionTypes.length > 20) {
    reasons.push('Highly repetitive action patterns');
    suspiciousCount++;
  }
  
  // Check session duration patterns
  const sessionsWithDuration = userActions.filter(action => action.duration);
  if (sessionsWithDuration.length > 5) {
    const durations = sessionsWithDuration.map(action => action.duration!);
    const durationVariance = durations.reduce((sum, duration) => {
      const avgDuration = durations.reduce((s, d) => s + d, 0) / durations.length;
      return sum + Math.pow(duration - avgDuration, 2);
    }, 0) / durations.length;
    
    const avgDuration = durations.reduce((s, d) => s + d, 0) / durations.length;
    if (durationVariance < avgDuration * 0.05) {
      reasons.push('Suspiciously consistent session durations');
      suspiciousCount++;
    }
  }
  
  const confidence = Math.min(suspiciousCount / 3, 1.0);
  return {
    isSuspicious: suspiciousCount >= 2,
    confidence,
    reasons
  };
}

/**
 * Validate KYC document data
 */
export function validateKYCDocument(document: {
  type: 'passport' | 'national_id' | 'driving_license';
  number: string;
  issueDate: string;
  expiryDate: string;
  country: string;
}): ValidationResult {
  const errors: string[] = [];
  
  // Validate document type
  const validTypes = ['passport', 'national_id', 'driving_license'];
  if (!validTypes.includes(document.type)) {
    errors.push('Invalid document type');
  }
  
  // Validate document number
  if (!document.number || document.number.trim().length === 0) {
    errors.push('Document number is required');
  } else {
    // Indonesian ID validation patterns
    switch (document.type) {
      case 'national_id':
        if (!/^\d{16}$/.test(document.number)) {
          errors.push('Indonesian National ID must be 16 digits');
        }
        break;
      case 'passport':
        if (!/^[A-Z]\d{7}$/.test(document.number)) {
          errors.push('Indonesian passport format: 1 letter + 7 digits');
        }
        break;
      case 'driving_license':
        if (!/^\d{12}$/.test(document.number)) {
          errors.push('Indonesian driving license must be 12 digits');
        }
        break;
    }
  }
  
  // Validate dates
  const issueDate = new Date(document.issueDate);
  const expiryDate = new Date(document.expiryDate);
  const now = new Date();
  
  if (isNaN(issueDate.getTime())) {
    errors.push('Invalid issue date');
  } else if (issueDate > now) {
    errors.push('Issue date cannot be in the future');
  }
  
  if (isNaN(expiryDate.getTime())) {
    errors.push('Invalid expiry date');
  } else if (expiryDate <= now) {
    errors.push('Document has expired');
  } else if (expiryDate <= issueDate) {
    errors.push('Expiry date must be after issue date');
  }
  
  // Validate country
  if (!document.country || document.country.length !== 2) {
    errors.push('Valid 2-letter country code is required');
  }
  
  return { isValid: errors.length === 0, errors };
}

/**
 * Validate social media post for quality scoring
 */
export function validateSocialPost(post: {
  content: string;
  mediaUrls?: string[];
  hashtags?: string[];
  mentions?: string[];
  platform: SocialPlatform;
}): ValidationResult & { qualityScore?: number } {
  const errors: string[] = [];
  const warnings: string[] = [];
  let qualityScore = 1.0;
  
  // Basic content validation
  const contentResult = FinovaValidator.validateContent(post.content);
  if (!contentResult.isValid) {
    errors.push(...contentResult.errors);
  }
  
  // Quality scoring factors
  const contentLength = post.content.length;
  
  // Length scoring
  if (contentLength < 50) {
    qualityScore *= 0.8;
    warnings.push('Short content may receive lower rewards');
  } else if (contentLength > 200) {
    qualityScore *= 1.2;
  }
  
  // Hashtag analysis
  if (post.hashtags) {
    if (post.hashtags.length > 10) {
      qualityScore *= 0.7;
      warnings.push('Too many hashtags may appear spammy');
    } else if (post.hashtags.length >= 3 && post.hashtags.length <= 7) {
      qualityScore *= 1.1;
    }
    
    // Check hashtag quality
    const spamHashtags = post.hashtags.filter(tag => 
      /^(follow|like|comment|subscribe|viral|trending)$/i.test(tag)
    );
    if (spamHashtags.length > 0) {
      qualityScore *= 0.6;
      warnings.push('Generic hashtags may reduce content quality score');
    }
  }
  
  // Media presence bonus
  if (post.mediaUrls && post.mediaUrls.length > 0) {
    qualityScore *= 1.15;
  }
  
  // Platform-specific adjustments
  switch (post.platform) {
    case SocialPlatform.TIKTOK:
      if (!post.mediaUrls || post.mediaUrls.length === 0) {
        qualityScore *= 0.5;
        warnings.push('TikTok posts should include video content');
      }
      break;
    case SocialPlatform.INSTAGRAM:
      if (!post.mediaUrls || post.mediaUrls.length === 0) {
        qualityScore *= 0.7;
        warnings.push('Instagram posts perform better with images/videos');
      }
      break;
    case SocialPlatform.TWITTER_X:
      if (contentLength > 280) {
        errors.push('Twitter/X posts cannot exceed 280 characters');
      }
      break;
  }
  
  // Clamp quality score
  qualityScore = Math.max(0.5, Math.min(2.0, qualityScore));
  
  return { 
    isValid: errors.length === 0, 
    errors, 
    warnings,
    qualityScore: errors.length === 0 ? qualityScore : undefined
  };
}

/**
 * Validate guild creation data
 */
export function validateGuildData(guild: {
  name: string;
  description: string;
  founderPublicKey: string;
  memberLimit: number;
  entryFee?: string;
  requirements?: {
    minLevel: number;
    minStake: string;
    minRP: number;
  };
}): ValidationResult {
  const errors: string[] = [];
  
  // Validate guild name
  if (!guild.name || guild.name.trim().length === 0) {
    errors.push('Guild name is required');
  } else if (guild.name.length < 3 || guild.name.length > 50) {
    errors.push('Guild name must be between 3 and 50 characters');
  } else if (!/^[a-zA-Z0-9\s\-_]+$/.test(guild.name)) {
    errors.push('Guild name can only contain letters, numbers, spaces, hyphens, and underscores');
  }
  
  // Validate description
  if (!guild.description || guild.description.trim().length === 0) {
    errors.push('Guild description is required');
  } else if (guild.description.length > 500) {
    errors.push('Guild description cannot exceed 500 characters');
  }
  
  // Validate founder
  const founderResult = FinovaValidator.validatePublicKey(guild.founderPublicKey);
  if (!founderResult.isValid) {
    errors.push('Invalid founder public key');
  }
  
  // Validate member limit
  if (guild.memberLimit < 10 || guild.memberLimit > 100) {
    errors.push('Member limit must be between 10 and 100');
  }
  
  // Validate entry fee (optional)
  if (guild.entryFee) {
    try {
      const fee = new BN(guild.entryFee);
      if (fee.isNeg()) {
        errors.push('Entry fee cannot be negative');
      }
    } catch {
      errors.push('Invalid entry fee format');
    }
  }
  
  // Validate requirements (optional)
  if (guild.requirements) {
    if (guild.requirements.minLevel < 1 || guild.requirements.minLevel > 100) {
      errors.push('Minimum level requirement must be between 1 and 100');
    }
    
    if (guild.requirements.minStake) {
      try {
        const minStake = new BN(guild.requirements.minStake);
        if (minStake.isNeg()) {
          errors.push('Minimum stake requirement cannot be negative');
        }
      } catch {
        errors.push('Invalid minimum stake format');
      }
    }
    
    if (guild.requirements.minRP < 0) {
      errors.push('Minimum RP requirement cannot be negative');
    }
  }
  
  return { isValid: errors.length === 0, errors };
}

/**
 * Validate tournament/competition entry
 */
export function validateTournamentEntry(entry: {
  tournamentId: string;
  participantPublicKey: string;
  teamMembers?: string[];
  entryFee?: string;
  requiredNFTs?: string[];
}): ValidationResult {
  const errors: string[] = [];
  
  // Validate tournament ID
  if (!entry.tournamentId || entry.tournamentId.trim().length === 0) {
    errors.push('Tournament ID is required');
  }
  
  // Validate participant
  const participantResult = FinovaValidator.validatePublicKey(entry.participantPublicKey);
  if (!participantResult.isValid) {
    errors.push('Invalid participant public key');
  }
  
  // Validate team members (optional)
  if (entry.teamMembers) {
    if (entry.teamMembers.length > 10) {
      errors.push('Maximum 10 team members allowed');
    }
    
    entry.teamMembers.forEach((member, index) => {
      const memberResult = FinovaValidator.validatePublicKey(member);
      if (!memberResult.isValid) {
        errors.push(`Invalid team member public key at index ${index}`);
      }
    });
    
    // Check for duplicates
    const uniqueMembers = new Set(entry.teamMembers);
    if (uniqueMembers.size !== entry.teamMembers.length) {
      errors.push('Duplicate team members not allowed');
    }
    
    // Ensure participant is not in team members list
    if (entry.teamMembers.includes(entry.participantPublicKey)) {
      errors.push('Participant cannot be listed as team member');
    }
  }
  
  // Validate entry fee (optional)
  if (entry.entryFee) {
    try {
      const fee = new BN(entry.entryFee);
      if (fee.isNeg()) {
        errors.push('Entry fee cannot be negative');
      }
    } catch {
      errors.push('Invalid entry fee format');
    }
  }
  
  // Validate required NFTs (optional)
  if (entry.requiredNFTs) {
    entry.requiredNFTs.forEach((nftMint, index) => {
      const nftResult = FinovaValidator.validatePublicKey(nftMint);
      if (!nftResult.isValid) {
        errors.push(`Invalid NFT mint at index ${index}`);
      }
    });
  }
  
  return { isValid: errors.length === 0, errors };
}

// ===== ADVANCED VALIDATION RULES =====

/**
 * Validate complex mining calculation
 */
export function validateMiningCalculation(params: {
  userId: string;
  baseRate: number;
  xpLevel: number;
  rpTier: number;
  stakingAmount: string;
  networkSize: number;
  qualityScore: number;
  timeMultiplier: number;
}): ValidationResult & { calculatedRate?: number } {
  const errors: string[] = [];
  
  // Validate user ID
  const userResult = FinovaValidator.validatePublicKey(params.userId);
  if (!userResult.isValid) {
    errors.push('Invalid user ID');
  }
  
  // Validate base rate
  if (params.baseRate < VALIDATION_CONSTANTS.MIN_MINING_RATE || 
      params.baseRate > VALIDATION_CONSTANTS.MAX_MINING_RATE) {
    errors.push(`Base rate must be between ${VALIDATION_CONSTANTS.MIN_MINING_RATE} and ${VALIDATION_CONSTANTS.MAX_MINING_RATE}`);
  }
  
  // Validate XP level
  if (params.xpLevel < 1 || params.xpLevel > 1000) {
    errors.push('XP level must be between 1 and 1000');
  }
  
  // Validate RP tier
  if (params.rpTier < 0 || params.rpTier > 5) {
    errors.push('RP tier must be between 0 and 5');
  }
  
  // Validate staking amount
  try {
    const stakingAmount = new BN(params.stakingAmount);
    if (stakingAmount.isNeg()) {
      errors.push('Staking amount cannot be negative');
    }
  } catch {
    errors.push('Invalid staking amount format');
  }
  
  // Validate network size
  if (params.networkSize < 0 || params.networkSize > 1000000) {
    errors.push('Network size must be between 0 and 1,000,000');
  }
  
  // Validate quality score
  if (params.qualityScore < VALIDATION_CONSTANTS.MIN_QUALITY_SCORE || 
      params.qualityScore > VALIDATION_CONSTANTS.MAX_QUALITY_SCORE) {
    errors.push(`Quality score must be between ${VALIDATION_CONSTANTS.MIN_QUALITY_SCORE} and ${VALIDATION_CONSTANTS.MAX_QUALITY_SCORE}`);
  }
  
  // Validate time multiplier
  if (params.timeMultiplier < 0.1 || params.timeMultiplier > 5.0) {
    errors.push('Time multiplier must be between 0.1 and 5.0');
  }
  
  if (errors.length === 0) {
    // Calculate final mining rate
    const xpMultiplier = 1 + (params.xpLevel / 100);
    const rpMultiplier = 1 + (params.rpTier * 0.2);
    const stakingMultiplier = 1 + (parseFloat(params.stakingAmount) / 10000 * 0.1);
    const regressionFactor = Math.exp(-0.001 * params.networkSize * 0.01);
    
    const calculatedRate = params.baseRate * 
      xpMultiplier * 
      rpMultiplier * 
      stakingMultiplier * 
      params.qualityScore * 
      params.timeMultiplier * 
      regressionFactor;
    
    return { 
      isValid: true, 
      errors: [], 
      calculatedRate: Math.max(0.001, calculatedRate) 
    };
  }
  
  return { isValid: false, errors };
}

/**
 * Comprehensive user activity validation
 */
export function validateUserActivity(activity: {
  userId: string;
  sessionId: string;
  activities: Array<{
    type: ActivityType;
    platform: SocialPlatform;
    timestamp: number;
    duration?: number;
    metadata?: Record<string, any>;
  }>;
  deviceInfo: {
    userAgent: string;
    screenResolution: string;
    timezone: string;
    language: string;
  };
}): ValidationResult & { suspiciousActivityDetected?: boolean } {
  const errors: string[] = [];
  const warnings: string[] = [];
  
  // Validate basic fields
  const userResult = FinovaValidator.validatePublicKey(activity.userId);
  if (!userResult.isValid) {
    errors.push('Invalid user ID');
  }
  
  if (!activity.sessionId || activity.sessionId.length < 16) {
    errors.push('Valid session ID is required');
  }
  
  // Validate activities array
  if (!activity.activities || activity.activities.length === 0) {
    errors.push('At least one activity is required');
  } else {
    // Validate individual activities
    activity.activities.forEach((act, index) => {
      if (!Object.values(ActivityType).includes(act.type)) {
        errors.push(`Invalid activity type at index ${index}`);
      }
      
      if (!Object.values(SocialPlatform).includes(act.platform)) {
        errors.push(`Invalid platform at index ${index}`);
      }
      
      if (act.timestamp <= 0 || act.timestamp > Date.now()) {
        errors.push(`Invalid timestamp at index ${index}`);
      }
      
      if (act.duration !== undefined && (act.duration < 0 || act.duration > 3600)) {
        warnings.push(`Unusual activity duration at index ${index}`);
      }
    });
    
    // Check for suspicious patterns
    const botDetection = detectBotBehavior(
      activity.activities.map(act => ({
        timestamp: act.timestamp,
        type: act.type,
        duration: act.duration
      }))
    );
    
    if (botDetection.isSuspicious) {
      warnings.push(...botDetection.reasons);
    }
  }
  
  // Validate device info
  if (!activity.deviceInfo.userAgent || activity.deviceInfo.userAgent.length < 10) {
    warnings.push('Incomplete device information');
  }
  
  if (!activity.deviceInfo.timezone) {
    warnings.push('Timezone information missing');
  }
  
  return {
    isValid: errors.length === 0,
    errors,
    warnings,
    suspiciousActivityDetected: warnings.some(w => w.includes('suspicious') || w.includes('bot'))
  };
}

// ===== EXPORT DEFAULT VALIDATOR =====

export default FinovaValidator;

// ===== TYPE EXPORTS =====

export interface NFTAttribute {
  trait_type: string;
  value: string | number;
  display_type?: 'boost_number' | 'boost_percentage' | 'number' | 'date';
}

// ===== FINAL VALIDATION SUMMARY =====

/**
 * Generate comprehensive validation report
 */
export function generateValidationReport(data: {
  userProfile?: Partial<UserProfile>;
  miningSession?: Partial<MiningSession>;
  xpActivity?: Partial<XPActivity>;
  referralData?: Partial<ReferralData>;
  nftMetadata?: Partial<NFTMetadata>;
  stakeData?: Partial<StakeData>;
}): {
  overallValid: boolean;
  totalErrors: number;
  totalWarnings: number;
  results: Record<string, ValidationResult>;
  summary: string;
} {
  const results: Record<string, ValidationResult> = {};
  let totalErrors = 0;
  let totalWarnings = 0;
  
  // Validate each provided data type
  if (data.userProfile) {
    results.userProfile = FinovaValidator.validateUserProfile(data.userProfile);
    totalErrors += results.userProfile.errors.length;
    totalWarnings += results.userProfile.warnings?.length || 0;
  }
  
  if (data.miningSession) {
    results.miningSession = FinovaValidator.validateMiningSession(data.miningSession);
    totalErrors += results.miningSession.errors.length;
    totalWarnings += results.miningSession.warnings?.length || 0;
  }
  
  if (data.xpActivity) {
    results.xpActivity = FinovaValidator.validateXPActivity(data.xpActivity);
    totalErrors += results.xpActivity.errors.length;
    totalWarnings += results.xpActivity.warnings?.length || 0;
  }
  
  if (data.referralData) {
    results.referralData = FinovaValidator.validateReferralData(data.referralData);
    totalErrors += results.referralData.errors.length;
    totalWarnings += results.referralData.warnings?.length || 0;
  }
  
  if (data.nftMetadata) {
    results.nftMetadata = FinovaValidator.validateNFTMetadata(data.nftMetadata);
    totalErrors += results.nftMetadata.errors.length;
    totalWarnings += results.nftMetadata.warnings?.length || 0;
  }
  
  if (data.stakeData) {
    results.stakeData = FinovaValidator.validateStakeData(data.stakeData);
    totalErrors += results.stakeData.errors.length;
    totalWarnings += results.stakeData.warnings?.length || 0;
  }
  
  const overallValid = totalErrors === 0;
  const summary = `Validation completed: ${overallValid ? 'PASSED' : 'FAILED'} | Errors: ${totalErrors} | Warnings: ${totalWarnings}`;
  
  return {
    overallValid,
    totalErrors,
    totalWarnings,
    results,
    summary
  };
}

