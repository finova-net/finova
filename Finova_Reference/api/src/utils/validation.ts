import { z } from 'zod';
import { isAddress } from '@solana/web3.js';
import validator from 'validator';
import bcrypt from 'bcryptjs';

// Constants for validation limits
export const VALIDATION_LIMITS = {
  PASSWORD_MIN_LENGTH: 8,
  PASSWORD_MAX_LENGTH: 128,
  USERNAME_MIN_LENGTH: 3,
  USERNAME_MAX_LENGTH: 30,
  EMAIL_MAX_LENGTH: 254,
  PHONE_MIN_LENGTH: 10,
  PHONE_MAX_LENGTH: 20,
  BIO_MAX_LENGTH: 500,
  POST_CONTENT_MAX_LENGTH: 2000,
  COMMENT_MAX_LENGTH: 500,
  REFERRAL_CODE_LENGTH: 8,
  KYC_DOCUMENT_MAX_SIZE: 10 * 1024 * 1024, // 10MB
  AVATAR_MAX_SIZE: 2 * 1024 * 1024, // 2MB
  MAX_REFERRAL_DEPTH: 3,
  MAX_DAILY_POSTS: 50,
  MAX_DAILY_COMMENTS: 200,
  MIN_STAKING_AMOUNT: 100,
  MAX_STAKING_AMOUNT: 1000000,
  XP_MIN_VALUE: 0,
  XP_MAX_VALUE: 1000000,
  RP_MIN_VALUE: 0,
  RP_MAX_VALUE: 100000,
  MINING_RATE_MIN: 0.001,
  MINING_RATE_MAX: 0.5,
} as const;

// Platform-specific validation
export const SUPPORTED_PLATFORMS = [
  'instagram',
  'tiktok', 
  'youtube',
  'facebook',
  'twitter',
  'linkedin'
] as const;

export const NFT_CARD_TYPES = [
  'mining_boost',
  'xp_accelerator', 
  'referral_power',
  'profile_badge',
  'achievement'
] as const;

export const NFT_RARITIES = [
  'common',
  'uncommon', 
  'rare',
  'epic',
  'legendary'
] as const;

// Base validation schemas
const passwordSchema = z.string()
  .min(VALIDATION_LIMITS.PASSWORD_MIN_LENGTH, 'Password too short')
  .max(VALIDATION_LIMITS.PASSWORD_MAX_LENGTH, 'Password too long')
  .regex(/^(?=.*[a-z])(?=.*[A-Z])(?=.*\d)(?=.*[@$!%*?&])[A-Za-z\d@$!%*?&]/, 
    'Password must contain uppercase, lowercase, number and special character');

const emailSchema = z.string()
  .email('Invalid email format')
  .max(VALIDATION_LIMITS.EMAIL_MAX_LENGTH, 'Email too long')
  .transform(val => val.toLowerCase());

const usernameSchema = z.string()
  .min(VALIDATION_LIMITS.USERNAME_MIN_LENGTH, 'Username too short')
  .max(VALIDATION_LIMITS.USERNAME_MAX_LENGTH, 'Username too long')
  .regex(/^[a-zA-Z0-9_]+$/, 'Username can only contain letters, numbers and underscores')
  .transform(val => val.toLowerCase());

const phoneSchema = z.string()
  .min(VALIDATION_LIMITS.PHONE_MIN_LENGTH, 'Phone number too short')
  .max(VALIDATION_LIMITS.PHONE_MAX_LENGTH, 'Phone number too long')
  .regex(/^\+?[1-9]\d{1,14}$/, 'Invalid phone number format');

const solanaAddressSchema = z.string()
  .refine(val => isAddress(val), 'Invalid Solana address');

const referralCodeSchema = z.string()
  .length(VALIDATION_LIMITS.REFERRAL_CODE_LENGTH, 'Invalid referral code length')
  .regex(/^[A-Z0-9]+$/, 'Invalid referral code format');

// User validation schemas
export const userRegistrationSchema = z.object({
  email: emailSchema,
  username: usernameSchema,
  password: passwordSchema,
  confirmPassword: z.string(),
  fullName: z.string()
    .min(2, 'Full name too short')
    .max(100, 'Full name too long')
    .regex(/^[a-zA-Z\s]+$/, 'Full name can only contain letters and spaces'),
  phoneNumber: phoneSchema.optional(),
  dateOfBirth: z.string()
    .datetime('Invalid date format')
    .refine(val => {
      const birthDate = new Date(val);
      const age = new Date().getFullYear() - birthDate.getFullYear();
      return age >= 13 && age <= 120;
    }, 'Age must be between 13 and 120'),
  countryCode: z.string()
    .length(2, 'Country code must be 2 characters')
    .regex(/^[A-Z]{2}$/, 'Invalid country code format'),
  referralCode: referralCodeSchema.optional(),
  acceptTerms: z.boolean().refine(val => val === true, 'Must accept terms'),
  acceptPrivacy: z.boolean().refine(val => val === true, 'Must accept privacy policy')
}).refine(data => data.password === data.confirmPassword, {
  message: "Passwords don't match",
  path: ["confirmPassword"],
});

export const userLoginSchema = z.object({
  identifier: z.string().min(1, 'Email or username required'),
  password: z.string().min(1, 'Password required'),
  rememberMe: z.boolean().optional().default(false),
  deviceFingerprint: z.string().optional()
});

export const userProfileUpdateSchema = z.object({
  fullName: z.string().min(2).max(100).optional(),
  bio: z.string().max(VALIDATION_LIMITS.BIO_MAX_LENGTH).optional(),
  avatar: z.string().url('Invalid avatar URL').optional(),
  location: z.string().max(100).optional(),
  website: z.string().url('Invalid website URL').optional(),
  socialLinks: z.object({
    instagram: z.string().url().optional(),
    tiktok: z.string().url().optional(),
    youtube: z.string().url().optional(),
    facebook: z.string().url().optional(),
    twitter: z.string().url().optional(),
    linkedin: z.string().url().optional()
  }).optional()
});

// KYC validation schemas
export const kycSubmissionSchema = z.object({
  documentType: z.enum(['passport', 'national_id', 'driver_license']),
  documentNumber: z.string().min(5).max(50),
  documentFrontImage: z.string().url('Invalid document front image URL'),
  documentBackImage: z.string().url('Invalid document back image URL').optional(),
  selfieImage: z.string().url('Invalid selfie image URL'),
  dateOfBirth: z.string().datetime(),
  fullName: z.string().min(2).max(100),
  address: z.object({
    street: z.string().min(5).max(200),
    city: z.string().min(2).max(100),
    state: z.string().min(2).max(100),
    postalCode: z.string().min(3).max(20),
    country: z.string().length(2)
  }),
  occupation: z.string().max(100).optional(),
  sourceOfFunds: z.enum(['employment', 'business', 'investment', 'other']).optional()
});

// Mining validation schemas
export const startMiningSchema = z.object({
  walletAddress: solanaAddressSchema,
  deviceId: z.string().min(10).max(100),
  sessionToken: z.string().min(32).max(256)
});

export const claimMiningRewardsSchema = z.object({
  walletAddress: solanaAddressSchema,
  amount: z.number()
    .min(VALIDATION_LIMITS.MINING_RATE_MIN)
    .max(VALIDATION_LIMITS.MINING_RATE_MAX * 24), // Max 24 hours of mining
  signature: z.string().min(64).max(256)
});

// XP System validation schemas
export const xpActivitySchema = z.object({
  activityType: z.enum([
    'post_create', 'post_like', 'post_share', 'post_comment',
    'story_create', 'story_view', 'follow_user', 'daily_login',
    'complete_quest', 'achieve_milestone', 'viral_content'
  ]),
  platform: z.enum(SUPPORTED_PLATFORMS),
  contentId: z.string().max(200).optional(),
  contentUrl: z.string().url().optional(),
  metadata: z.object({
    engagementCount: z.number().min(0).optional(),
    contentLength: z.number().min(0).optional(),
    hasMedia: z.boolean().optional(),
    isOriginal: z.boolean().optional(),
    qualityScore: z.number().min(0).max(2).optional()
  }).optional()
});

export const xpClaimSchema = z.object({
  userId: z.string().uuid(),
  activities: z.array(xpActivitySchema).min(1).max(100),
  totalXP: z.number().min(VALIDATION_LIMITS.XP_MIN_VALUE).max(VALIDATION_LIMITS.XP_MAX_VALUE),
  signature: z.string().min(64).max(256)
});

// Referral validation schemas
export const referralRegistrationSchema = z.object({
  referralCode: referralCodeSchema,
  newUserEmail: emailSchema,
  referrerUserId: z.string().uuid()
});

export const referralClaimSchema = z.object({
  referralId: z.string().uuid(),
  activityType: z.enum(['signup', 'kyc_complete', 'first_mining', 'milestone_reached']),
  rewardAmount: z.number().min(VALIDATION_LIMITS.RP_MIN_VALUE).max(VALIDATION_LIMITS.RP_MAX_VALUE)
});

// Staking validation schemas
export const stakeTokensSchema = z.object({
  amount: z.number()
    .min(VALIDATION_LIMITS.MIN_STAKING_AMOUNT)
    .max(VALIDATION_LIMITS.MAX_STAKING_AMOUNT),
  duration: z.enum(['flexible', '30days', '90days', '180days', '365days']),
  walletAddress: solanaAddressSchema,
  signature: z.string().min(64).max(256)
});

export const unstakeTokensSchema = z.object({
  stakeId: z.string().uuid(),
  amount: z.number().min(1),
  walletAddress: solanaAddressSchema,
  signature: z.string().min(64).max(256)
});

// NFT validation schemas
export const nftMintSchema = z.object({
  cardType: z.enum(NFT_CARD_TYPES),
  rarity: z.enum(NFT_RARITIES),
  metadata: z.object({
    name: z.string().min(1).max(100),
    description: z.string().max(500),
    image: z.string().url(),
    attributes: z.array(z.object({
      trait_type: z.string(),
      value: z.union([z.string(), z.number()])
    })).optional()
  }),
  recipientAddress: solanaAddressSchema
});

export const nftUseCardSchema = z.object({
  cardId: z.string().uuid(),
  walletAddress: solanaAddressSchema,
  targetUserId: z.string().uuid().optional(),
  signature: z.string().min(64).max(256)
});

// Social integration validation schemas
export const socialAccountLinkSchema = z.object({
  platform: z.enum(SUPPORTED_PLATFORMS),
  platformUserId: z.string().min(1).max(100),
  platformUsername: z.string().min(1).max(100),
  accessToken: z.string().min(10).max(1000),
  refreshToken: z.string().min(10).max(1000).optional(),
  expiresAt: z.string().datetime()
});

export const socialPostValidationSchema = z.object({
  platform: z.enum(SUPPORTED_PLATFORMS),
  postId: z.string().min(1).max(200),
  content: z.string().max(VALIDATION_LIMITS.POST_CONTENT_MAX_LENGTH),
  mediaUrls: z.array(z.string().url()).max(10).optional(),
  hashtags: z.array(z.string().regex(/^#\w+$/)).max(30).optional(),
  mentions: z.array(z.string().regex(/^@\w+$/)).max(20).optional(),
  engagement: z.object({
    likes: z.number().min(0).optional(),
    comments: z.number().min(0).optional(),
    shares: z.number().min(0).optional(),
    views: z.number().min(0).optional()
  }).optional()
});

// Guild validation schemas
export const guildCreateSchema = z.object({
  name: z.string().min(3).max(50),
  description: z.string().max(500),
  logo: z.string().url().optional(),
  isPrivate: z.boolean().default(false),
  maxMembers: z.number().min(10).max(50).default(25),
  requirements: z.object({
    minLevel: z.number().min(11).max(100),
    minRP: z.number().min(0).optional(),
    minStaking: z.number().min(0).optional()
  })
});

export const guildJoinSchema = z.object({
  guildId: z.string().uuid(),
  message: z.string().max(200).optional()
});

// Admin validation schemas
export const adminActionSchema = z.object({
  action: z.enum([
    'ban_user', 'unban_user', 'verify_kyc', 'reject_kyc',
    'adjust_mining_rate', 'create_special_event', 'mint_special_nft'
  ]),
  targetId: z.string().uuid(),
  reason: z.string().max(500),
  parameters: z.record(z.unknown()).optional()
});

// Custom validation functions
export class ValidationHelper {
  static validateWalletSignature(
    message: string,
    signature: string,
    publicKey: string
  ): boolean {
    try {
      // Implement Solana signature verification logic
      return true; // Placeholder
    } catch (error) {
      return false;
    }
  }

  static validateContentQuality(content: string): number {
    // AI-based content quality scoring (0.5 - 2.0)
    const wordCount = content.split(/\s+/).length;
    const hasLinks = /https?:\/\//.test(content);
    const hasHashtags = /#\w+/.test(content);
    const hasEmojis = /[\u{1F600}-\u{1F64F}]|[\u{1F300}-\u{1F5FF}]|[\u{1F680}-\u{1F6FF}]|[\u{1F1E0}-\u{1F1FF}]/u.test(content);
    
    let score = 1.0;
    
    if (wordCount > 50) score += 0.2;
    if (wordCount > 100) score += 0.3;
    if (hasLinks) score += 0.1;
    if (hasHashtags) score += 0.1;
    if (hasEmojis) score += 0.1;
    
    // Check for spam patterns
    if (content.match(/(.)\1{4,}/)) score -= 0.3; // Repeated characters
    if ((content.match(/[!?]{3,}/g) || []).length > 0) score -= 0.2; // Excessive punctuation
    
    return Math.max(0.5, Math.min(2.0, score));
  }

  static validateBiometricData(biometricHash: string): boolean {
    // Validate biometric hash format and integrity
    return /^[a-f0-9]{64}$/i.test(biometricHash);
  }

  static validateDeviceFingerprint(fingerprint: string): boolean {
    // Validate device fingerprint format
    return /^[a-zA-Z0-9]{32,128}$/.test(fingerprint);
  }

  static validateIPAddress(ip: string): boolean {
    return validator.isIP(ip);
  }

  static validateReferralNetwork(
    referrerId: string,
    newUserId: string,
    existingNetwork: string[]
  ): boolean {
    // Check for circular referrals
    if (existingNetwork.includes(newUserId)) return false;
    if (existingNetwork.includes(referrerId)) return false;
    
    // Check depth limit
    return existingNetwork.length < VALIDATION_LIMITS.MAX_REFERRAL_DEPTH;
  }

  static calculateMiningRateLimit(
    userLevel: number,
    stakingAmount: number,
    totalHoldings: number
  ): number {
    const baseRate = 0.1;
    const levelMultiplier = 1 + (userLevel / 100);
    const stakingBonus = stakingAmount > 0 ? 1 + Math.log10(stakingAmount / 100) * 0.1 : 1;
    const regressionFactor = Math.exp(-0.001 * totalHoldings);
    
    return baseRate * levelMultiplier * stakingBonus * regressionFactor;
  }

  static validateXPClaim(
    activities: any[],
    expectedXP: number,
    userLevel: number
  ): boolean {
    let calculatedXP = 0;
    
    for (const activity of activities) {
      const baseXP = this.getBaseXPForActivity(activity.activityType);
      const platformMultiplier = this.getPlatformMultiplier(activity.platform);
      const qualityScore = activity.metadata?.qualityScore || 1.0;
      const levelProgression = Math.exp(-0.01 * userLevel);
      
      calculatedXP += baseXP * platformMultiplier * qualityScore * levelProgression;
    }
    
    // Allow 5% tolerance for calculation differences
    const tolerance = expectedXP * 0.05;
    return Math.abs(calculatedXP - expectedXP) <= tolerance;
  }

  private static getBaseXPForActivity(activityType: string): number {
    const xpMap: Record<string, number> = {
      'post_create': 50,
      'post_like': 5,
      'post_share': 15,
      'post_comment': 25,
      'story_create': 25,
      'story_view': 2,
      'follow_user': 20,
      'daily_login': 10,
      'complete_quest': 100,
      'achieve_milestone': 500,
      'viral_content': 1000
    };
    return xpMap[activityType] || 0;
  }

  private static getPlatformMultiplier(platform: string): number {
    const multipliers: Record<string, number> = {
      'instagram': 1.2,
      'tiktok': 1.3,
      'youtube': 1.4,
      'facebook': 1.1,
      'twitter': 1.2,
      'linkedin': 1.0
    };
    return multipliers[platform] || 1.0;
  }

  static sanitizeInput(input: string): string {
    return validator.escape(input.trim());
  }

  static validateFileUpload(
    file: { size: number; mimetype: string },
    maxSize: number,
    allowedTypes: string[]
  ): { valid: boolean; error?: string } {
    if (file.size > maxSize) {
      return { valid: false, error: 'File too large' };
    }
    
    if (!allowedTypes.includes(file.mimetype)) {
      return { valid: false, error: 'Invalid file type' };
    }
    
    return { valid: true };
  }

  static generateSecureReferralCode(): string {
    const chars = 'ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789';
    let result = '';
    for (let i = 0; i < VALIDATION_LIMITS.REFERRAL_CODE_LENGTH; i++) {
      result += chars.charAt(Math.floor(Math.random() * chars.length));
    }
    return result;
  }

  static hashPassword(password: string): Promise<string> {
    return bcrypt.hash(password, 12);
  }

  static comparePassword(password: string, hash: string): Promise<boolean> {
    return bcrypt.compare(password, hash);
  }
}

// Rate limiting validation
export const rateLimitSchema = z.object({
  endpoint: z.string(),
  identifier: z.string(),
  limit: z.number().min(1),
  windowMs: z.number().min(1000),
  skipSuccessfulRequests: z.boolean().default(false)
});

// Export all schemas for use in controllers
export {
  passwordSchema,
  emailSchema,
  usernameSchema,
  phoneSchema,
  solanaAddressSchema,
  referralCodeSchema
};
