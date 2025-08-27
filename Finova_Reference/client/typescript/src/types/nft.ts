// finova-net/finova/client/typescript/src/types/nft.ts

/**
 * Finova Network - NFT Types
 * Complete TypeScript type definitions for the NFT ecosystem
 * 
 * @version 3.0
 * @author Finova Network Team
 * @license MIT
 */

import { PublicKey } from '@solana/web3.js';
import { BN } from '@coral-xyz/anchor';

// ============================================================================
// CORE NFT TYPES
// ============================================================================

/**
 * NFT Collection metadata and configuration
 */
export interface NFTCollection {
  /** Collection address on Solana */
  address: PublicKey;
  /** Collection name */
  name: string;
  /** Collection symbol */
  symbol: string;
  /** Collection description */
  description: string;
  /** Collection image URI */
  image: string;
  /** External URL */
  externalUrl?: string;
  /** Collection creator */
  creator: PublicKey;
  /** Total supply */
  totalSupply: number;
  /** Current minted count */
  minted: number;
  /** Royalty percentage (basis points) */
  royaltyBps: number;
  /** Collection verified status */
  verified: boolean;
  /** Creation timestamp */
  createdAt: Date;
  /** Last update timestamp */
  updatedAt: Date;
  /** Collection categories */
  categories: NFTCategory[];
  /** Collection traits */
  traits: CollectionTrait[];
}

/**
 * Individual NFT metadata
 */
export interface NFTMetadata {
  /** NFT mint address */
  mint: PublicKey;
  /** Token name */
  name: string;
  /** Token symbol */
  symbol: string;
  /** Token description */
  description: string;
  /** Main image URI */
  image: string;
  /** Animation/video URI */
  animationUrl?: string;
  /** External URL */
  externalUrl?: string;
  /** NFT attributes */
  attributes: NFTAttribute[];
  /** Collection reference */
  collection?: PublicKey;
  /** Creators with shares */
  creators: NFTCreator[];
  /** Royalty information */
  royalty: NFTRoyalty;
  /** Rarity information */
  rarity: NFTRarity;
  /** Utility information */
  utility: NFTUtility;
  /** Market data */
  marketData: NFTMarketData;
  /** Ownership history */
  ownershipHistory: NFTOwnership[];
}

/**
 * NFT attribute/trait definition
 */
export interface NFTAttribute {
  /** Trait type */
  traitType: string;
  /** Trait value */
  value: string | number;
  /** Display type (number, date, boost_number, boost_percentage) */
  displayType?: 'number' | 'date' | 'boost_number' | 'boost_percentage';
  /** Rarity percentage */
  rarity?: number;
  /** Maximum value (for boost types) */
  maxValue?: number;
}

/**
 * NFT creator information
 */
export interface NFTCreator {
  /** Creator address */
  address: PublicKey;
  /** Creator share percentage */
  share: number;
  /** Verified creator status */
  verified: boolean;
}

/**
 * NFT royalty configuration
 */
export interface NFTRoyalty {
  /** Royalty percentage (basis points) */
  basisPoints: number;
  /** Primary sale happened */
  primarySaleHappened: boolean;
  /** Royalty recipients */
  recipients: RoyaltyRecipient[];
}

/**
 * Royalty recipient information
 */
export interface RoyaltyRecipient {
  /** Recipient address */
  address: PublicKey;
  /** Share percentage */
  share: number;
}

// ============================================================================
// SPECIAL CARDS SYSTEM (Hamster Kombat-Inspired)
// ============================================================================

/**
 * Special card categories
 */
export enum SpecialCardCategory {
  MINING_BOOST = 'mining_boost',
  XP_ACCELERATOR = 'xp_accelerator',
  REFERRAL_POWER = 'referral_power',
  UTILITY = 'utility',
  COSMETIC = 'cosmetic'
}

/**
 * Card rarity levels
 */
export enum CardRarity {
  COMMON = 'common',
  UNCOMMON = 'uncommon',
  RARE = 'rare',
  EPIC = 'epic',
  LEGENDARY = 'legendary',
  MYTHIC = 'mythic'
}

/**
 * Special card definition
 */
export interface SpecialCard extends NFTMetadata {
  /** Card category */
  category: SpecialCardCategory;
  /** Card rarity */
  rarity: CardRarity;
  /** Card effects */
  effects: CardEffect[];
  /** Usage limitations */
  usage: CardUsage;
  /** Synergy bonuses */
  synergies: CardSynergy[];
  /** Prerequisites */
  prerequisites: CardPrerequisite[];
  /** Card evolution data */
  evolution?: CardEvolution;
}

/**
 * Card effect definition
 */
export interface CardEffect {
  /** Effect type */
  type: EffectType;
  /** Effect value */
  value: number;
  /** Value type (percentage, flat, multiplier) */
  valueType: 'percentage' | 'flat' | 'multiplier';
  /** Effect duration in seconds */
  duration: number;
  /** Target of the effect */
  target: EffectTarget;
  /** Effect conditions */
  conditions?: EffectCondition[];
}

/**
 * Effect types
 */
export enum EffectType {
  // Mining effects
  MINING_RATE_BOOST = 'mining_rate_boost',
  MINING_DURATION_EXTEND = 'mining_duration_extend',
  MINING_QUALITY_BOOST = 'mining_quality_boost',
  
  // XP effects
  XP_MULTIPLIER = 'xp_multiplier',
  XP_STREAK_PROTECTION = 'xp_streak_protection',
  XP_INSTANT_GRANT = 'xp_instant_grant',
  XP_ACTIVITY_BOOST = 'xp_activity_boost',
  
  // Referral effects
  REFERRAL_BONUS_BOOST = 'referral_bonus_boost',
  REFERRAL_TIER_UPGRADE = 'referral_tier_upgrade',
  REFERRAL_NETWORK_AMPLIFY = 'referral_network_amplify',
  
  // Utility effects
  STAKING_REWARD_BOOST = 'staking_reward_boost',
  TRANSACTION_FEE_DISCOUNT = 'transaction_fee_discount',
  MARKETPLACE_FEE_DISCOUNT = 'marketplace_fee_discount',
  
  // Special effects
  GUILD_BONUS = 'guild_bonus',
  EVENT_ACCESS = 'event_access',
  PREMIUM_FEATURES = 'premium_features'
}

/**
 * Effect targets
 */
export enum EffectTarget {
  SELF = 'self',
  REFERRALS = 'referrals',
  GUILD = 'guild',
  NETWORK = 'network'
}

/**
 * Effect conditions
 */
export interface EffectCondition {
  /** Condition type */
  type: 'level_requirement' | 'activity_threshold' | 'time_window' | 'platform_specific';
  /** Condition value */
  value: any;
  /** Condition operator */
  operator: 'gt' | 'gte' | 'lt' | 'lte' | 'eq' | 'neq';
}

/**
 * Card usage limitations
 */
export interface CardUsage {
  /** Single use card */
  singleUse: boolean;
  /** Maximum uses (0 = unlimited) */
  maxUses: number;
  /** Current uses */
  currentUses: number;
  /** Cooldown period in seconds */
  cooldown: number;
  /** Last used timestamp */
  lastUsed?: Date;
  /** Transfer restrictions */
  transferRestrictions: TransferRestriction[];
}

/**
 * Transfer restrictions
 */
export interface TransferRestriction {
  /** Restriction type */
  type: 'time_lock' | 'level_requirement' | 'activity_requirement' | 'staking_requirement';
  /** Restriction value */
  value: any;
  /** Restriction active until */
  activeUntil?: Date;
}

/**
 * Card synergy bonuses
 */
export interface CardSynergy {
  /** Required cards for synergy */
  requiredCards: SpecialCardCategory[];
  /** Synergy bonus effects */
  bonusEffects: CardEffect[];
  /** Synergy name */
  name: string;
  /** Synergy description */
  description: string;
}

/**
 * Card prerequisites
 */
export interface CardPrerequisite {
  /** Prerequisite type */
  type: 'level' | 'xp' | 'rp' | 'staking' | 'achievement' | 'card_ownership';
  /** Required value */
  value: any;
  /** Human readable description */
  description: string;
}

/**
 * Card evolution system
 */
export interface CardEvolution {
  /** Can evolve */
  canEvolve: boolean;
  /** Evolution requirements */
  requirements: EvolutionRequirement[];
  /** Next evolution card */
  nextEvolution?: PublicKey;
  /** Evolution tree */
  evolutionTree: EvolutionNode[];
}

/**
 * Evolution requirements
 */
export interface EvolutionRequirement {
  /** Requirement type */
  type: 'usage_count' | 'time_held' | 'achievement' | 'sacrifice' | 'community_vote';
  /** Required value */
  value: any;
  /** Current progress */
  progress: any;
  /** Requirement met */
  met: boolean;
}

/**
 * Evolution tree node
 */
export interface EvolutionNode {
  /** Card mint address */
  cardMint: PublicKey;
  /** Evolution level */
  level: number;
  /** Parent card */
  parent?: PublicKey;
  /** Child cards */
  children: PublicKey[];
  /** Evolution cost */
  cost: EvolutionCost;
}

/**
 * Evolution cost
 */
export interface EvolutionCost {
  /** FIN token cost */
  finCost: BN;
  /** Required sacrifice cards */
  sacrificeCards: PublicKey[];
  /** Required achievements */
  achievements: string[];
  /** Other requirements */
  otherRequirements: Record<string, any>;
}

// ============================================================================
// PROFILE BADGE NFTS
// ============================================================================

/**
 * Profile badge levels
 */
export enum BadgeLevel {
  BRONZE = 'bronze',
  SILVER = 'silver',
  GOLD = 'gold',
  PLATINUM = 'platinum',
  DIAMOND = 'diamond',
  MYTHIC = 'mythic'
}

/**
 * Badge types
 */
export enum BadgeType {
  LEVEL = 'level',
  ACHIEVEMENT = 'achievement',
  SPECIAL_EVENT = 'special_event',
  FOUNDER = 'founder',
  COMMUNITY = 'community'
}

/**
 * Profile badge NFT
 */
export interface ProfileBadge extends NFTMetadata {
  /** Badge level */
  level: BadgeLevel;
  /** Badge type */
  type: BadgeType;
  /** Permanent bonuses */
  permanentBonuses: BadgeBonus[];
  /** Unlock requirements */
  unlockRequirements: BadgeRequirement[];
  /** Badge progression */
  progression: BadgeProgression;
  /** Display settings */
  display: BadgeDisplay;
}

/**
 * Badge bonus effects
 */
export interface BadgeBonus {
  /** Bonus type */
  type: 'mining_multiplier' | 'xp_bonus' | 'referral_bonus' | 'staking_bonus' | 'special_access';
  /** Bonus value */
  value: number;
  /** Bonus description */
  description: string;
  /** Permanent effect */
  permanent: boolean;
}

/**
 * Badge unlock requirements
 */
export interface BadgeRequirement {
  /** Requirement category */
  category: 'level' | 'achievement' | 'time' | 'activity' | 'community' | 'special';
  /** Specific requirement */
  requirement: string;
  /** Required value */
  value: any;
  /** Current progress */
  progress: any;
  /** Requirement completed */
  completed: boolean;
}

/**
 * Badge progression tracking
 */
export interface BadgeProgression {
  /** Current tier */
  currentTier: number;
  /** Next tier requirements */
  nextTierRequirements: BadgeRequirement[];
  /** Progression history */
  history: ProgressionEvent[];
}

/**
 * Progression event
 */
export interface ProgressionEvent {
  /** Event type */
  type: 'earned' | 'upgraded' | 'special';
  /** Event timestamp */
  timestamp: Date;
  /** Event description */
  description: string;
  /** Associated rewards */
  rewards?: any[];
}

/**
 * Badge display configuration
 */
export interface BadgeDisplay {
  /** Display on profile */
  showOnProfile: boolean;
  /** Display priority */
  priority: number;
  /** Custom display name */
  customName?: string;
  /** Animation settings */
  animation?: BadgeAnimation;
}

/**
 * Badge animation settings
 */
export interface BadgeAnimation {
  /** Animation type */
  type: 'glow' | 'pulse' | 'rotate' | 'sparkle' | 'none';
  /** Animation speed */
  speed: 'slow' | 'medium' | 'fast';
  /** Animation color */
  color?: string;
}

// ============================================================================
// ACHIEVEMENT NFTS
// ============================================================================

/**
 * Achievement categories
 */
export enum AchievementCategory {
  FIRST_TIME = 'first_time',
  MILESTONE = 'milestone',
  COMMUNITY = 'community',
  SPECIAL_EVENT = 'special_event',
  CREATOR = 'creator',
  SOCIAL = 'social'
}

/**
 * Achievement NFT
 */
export interface AchievementNFT extends NFTMetadata {
  /** Achievement category */
  category: AchievementCategory;
  /** Achievement rarity */
  rarity: CardRarity;
  /** Unlock criteria */
  unlockCriteria: AchievementCriteria;
  /** Rewards granted */
  rewards: AchievementReward[];
  /** Achievement statistics */
  stats: AchievementStats;
  /** Related achievements */
  relatedAchievements: PublicKey[];
}

/**
 * Achievement unlock criteria
 */
export interface AchievementCriteria {
  /** Criteria type */
  type: 'user_count' | 'activity_count' | 'time_based' | 'value_threshold' | 'special_event';
  /** Target value */
  target: any;
  /** Time window (if applicable) */
  timeWindow?: number;
  /** Additional conditions */
  conditions?: Record<string, any>;
}

/**
 * Achievement rewards
 */
export interface AchievementReward {
  /** Reward type */
  type: 'fin_tokens' | 'xp_boost' | 'rp_boost' | 'special_card' | 'badge' | 'title';
  /** Reward amount/value */
  value: any;
  /** Reward duration (if temporary) */
  duration?: number;
  /** Reward description */
  description: string;
}

/**
 * Achievement statistics
 */
export interface AchievementStats {
  /** Total holders */
  totalHolders: number;
  /** Unlock rate */
  unlockRate: number;
  /** First unlocked by */
  firstUnlockedBy?: PublicKey;
  /** First unlock timestamp */
  firstUnlocked?: Date;
  /** Rarity score */
  rarityScore: number;
}

// ============================================================================
// NFT MARKETPLACE
// ============================================================================

/**
 * Marketplace listing status
 */
export enum ListingStatus {
  ACTIVE = 'active',
  SOLD = 'sold',
  CANCELLED = 'cancelled',
  EXPIRED = 'expired'
}

/**
 * Auction types
 */
export enum AuctionType {
  FIXED_PRICE = 'fixed_price',
  ENGLISH_AUCTION = 'english_auction',
  DUTCH_AUCTION = 'dutch_auction',
  RESERVE_AUCTION = 'reserve_auction'
}

/**
 * NFT marketplace listing
 */
export interface NFTListing {
  /** Listing ID */
  id: string;
  /** NFT mint address */
  nftMint: PublicKey;
  /** Seller address */
  seller: PublicKey;
  /** Listing price */
  price: BN;
  /** Currency (FIN, SOL, USDC) */
  currency: string;
  /** Auction type */
  auctionType: AuctionType;
  /** Listing status */
  status: ListingStatus;
  /** Created at */
  createdAt: Date;
  /** Expires at */
  expiresAt?: Date;
  /** Auction details */
  auctionDetails?: AuctionDetails;
  /** Listing metadata */
  metadata: ListingMetadata;
  /** Bids (if auction) */
  bids: Bid[];
}

/**
 * Auction details
 */
export interface AuctionDetails {
  /** Starting price */
  startingPrice: BN;
  /** Reserve price */
  reservePrice?: BN;
  /** Buy now price */
  buyNowPrice?: BN;
  /** Minimum bid increment */
  minBidIncrement: BN;
  /** Auction duration */
  duration: number;
  /** Auto-extend on late bid */
  autoExtend: boolean;
  /** Extension duration */
  extensionDuration?: number;
}

/**
 * Listing metadata
 */
export interface ListingMetadata {
  /** Featured listing */
  featured: boolean;
  /** Listing category */
  category: string;
  /** Tags */
  tags: string[];
  /** Description */
  description?: string;
  /** Bundle information */
  bundle?: BundleInfo;
}

/**
 * Bundle information
 */
export interface BundleInfo {
  /** Bundle NFTs */
  nfts: PublicKey[];
  /** Bundle discount */
  discount?: number;
  /** Bundle title */
  title: string;
  /** Bundle description */
  description: string;
}

/**
 * Marketplace bid
 */
export interface Bid {
  /** Bid ID */
  id: string;
  /** Bidder address */
  bidder: PublicKey;
  /** Bid amount */
  amount: BN;
  /** Bid timestamp */
  timestamp: Date;
  /** Bid status */
  status: 'active' | 'outbid' | 'winning' | 'withdrawn';
  /** Bid metadata */
  metadata?: Record<string, any>;
}

/**
 * NFT sale record
 */
export interface NFTSale {
  /** Sale ID */
  id: string;
  /** NFT mint */
  nftMint: PublicKey;
  /** Seller */
  seller: PublicKey;
  /** Buyer */
  buyer: PublicKey;
  /** Sale price */
  price: BN;
  /** Currency */
  currency: string;
  /** Sale timestamp */
  timestamp: Date;
  /** Transaction signature */
  signature: string;
  /** Royalties paid */
  royaltiesPaid: RoyaltyPayment[];
  /** Platform fees */
  platformFees: BN;
}

/**
 * Royalty payment record
 */
export interface RoyaltyPayment {
  /** Recipient address */
  recipient: PublicKey;
  /** Amount paid */
  amount: BN;
  /** Percentage */
  percentage: number;
}

// ============================================================================
// NFT UTILITY & GAMIFICATION
// ============================================================================

/**
 * NFT utility configuration
 */
export interface NFTUtility {
  /** Utility type */
  type: 'functional' | 'cosmetic' | 'access' | 'boost';
  /** Active utility effects */
  activeEffects: UtilityEffect[];
  /** Passive utility effects */
  passiveEffects: UtilityEffect[];
  /** Utility constraints */
  constraints: UtilityConstraint[];
  /** Upgrade paths */
  upgradePaths: UpgradePath[];
}

/**
 * Utility effect
 */
export interface UtilityEffect {
  /** Effect ID */
  id: string;
  /** Effect name */
  name: string;
  /** Effect description */
  description: string;
  /** Effect value */
  value: number;
  /** Effect duration */
  duration?: number;
  /** Effect conditions */
  conditions: UtilityCondition[];
  /** Effect cooldown */
  cooldown?: number;
}

/**
 * Utility constraint
 */
export interface UtilityConstraint {
  /** Constraint type */
  type: 'time' | 'usage' | 'level' | 'ownership' | 'activity';
  /** Constraint value */
  value: any;
  /** Constraint active */
  active: boolean;
}

/**
 * Utility condition
 */
export interface UtilityCondition {
  /** Condition type */
  type: string;
  /** Condition value */
  value: any;
  /** Condition met */
  met: boolean;
}

/**
 * NFT upgrade path
 */
export interface UpgradePath {
  /** Upgrade ID */
  id: string;
  /** Target NFT */
  targetNft: PublicKey;
  /** Upgrade requirements */
  requirements: UpgradeRequirement[];
  /** Upgrade cost */
  cost: UpgradeCost;
  /** Upgrade benefits */
  benefits: string[];
}

/**
 * Upgrade requirement
 */
export interface UpgradeRequirement {
  /** Requirement type */
  type: string;
  /** Required value */
  value: any;
  /** Current progress */
  progress: any;
}

/**
 * Upgrade cost
 */
export interface UpgradeCost {
  /** FIN token cost */
  finCost?: BN;
  /** Material NFTs required */
  materials?: PublicKey[];
  /** Other costs */
  other?: Record<string, any>;
}

// ============================================================================
// NFT RARITY & MARKET DATA
// ============================================================================

/**
 * NFT rarity information
 */
export interface NFTRarity {
  /** Overall rarity rank */
  rank: number;
  /** Rarity score */
  score: number;
  /** Rarity tier */
  tier: CardRarity;
  /** Trait rarities */
  traitRarities: TraitRarity[];
  /** Collection size */
  collectionSize: number;
  /** Rarity calculation method */
  calculationMethod: 'trait_sum' | 'statistical' | 'manual';
}

/**
 * Individual trait rarity
 */
export interface TraitRarity {
  /** Trait type */
  traitType: string;
  /** Trait value */
  value: string | number;
  /** Occurrence count */
  count: number;
  /** Rarity percentage */
  percentage: number;
  /** Rarity score contribution */
  scoreContribution: number;
}

/**
 * NFT market data
 */
export interface NFTMarketData {
  /** Last sale price */
  lastSalePrice?: BN;
  /** Last sale currency */
  lastSaleCurrency?: string;
  /** Last sale date */
  lastSaleDate?: Date;
  /** Current floor price */
  floorPrice?: BN;
  /** Average price (30 days) */
  avgPrice30d?: BN;
  /** Price history */
  priceHistory: PricePoint[];
  /** Trading volume */
  volume24h?: BN;
  /** Number of sales */
  totalSales: number;
  /** Market cap contribution */
  marketCapContribution?: BN;
}

/**
 * Price point for history
 */
export interface PricePoint {
  /** Price */
  price: BN;
  /** Currency */
  currency: string;
  /** Timestamp */
  timestamp: Date;
  /** Transaction type */
  type: 'sale' | 'listing' | 'bid';
}

/**
 * NFT ownership record
 */
export interface NFTOwnership {
  /** Owner address */
  owner: PublicKey;
  /** Ownership start */
  ownedSince: Date;
  /** Ownership end (if transferred) */
  ownedUntil?: Date;
  /** Transfer transaction */
  transferTx?: string;
  /** Purchase price */
  purchasePrice?: BN;
  /** Purchase currency */
  purchaseCurrency?: string;
}

// ============================================================================
// COLLECTION & CATEGORY TYPES
// ============================================================================

/**
 * NFT categories
 */
export enum NFTCategory {
  SPECIAL_CARDS = 'special_cards',
  PROFILE_BADGES = 'profile_badges',
  ACHIEVEMENTS = 'achievements',
  COSMETICS = 'cosmetics',
  UTILITIES = 'utilities',
  COLLECTIBLES = 'collectibles'
}

/**
 * Collection trait definition
 */
export interface CollectionTrait {
  /** Trait type */
  type: string;
  /** Possible values */
  values: (string | number)[];
  /** Value frequencies */
  frequencies: Record<string, number>;
  /** Trait importance weight */
  weight: number;
}

/**
 * Collection statistics
 */
export interface CollectionStats {
  /** Total NFTs */
  totalNfts: number;
  /** Unique owners */
  uniqueOwners: number;
  /** Floor price */
  floorPrice: BN;
  /** Total volume */
  totalVolume: BN;
  /** Average price */
  averagePrice: BN;
  /** Market cap */
  marketCap: BN;
  /** Listed count */
  listedCount: number;
  /** Listing percentage */
  listingPercentage: number;
}

// ============================================================================
// EVENTS & ACTIVITIES
// ============================================================================

/**
 * NFT event types
 */
export enum NFTEventType {
  MINT = 'mint',
  TRANSFER = 'transfer',
  SALE = 'sale',
  LIST = 'list',
  DELIST = 'delist',
  BID = 'bid',
  USE = 'use',
  UPGRADE = 'upgrade',
  BURN = 'burn'
}

/**
 * NFT activity event
 */
export interface NFTEvent {
  /** Event ID */
  id: string;
  /** Event type */
  type: NFTEventType;
  /** NFT mint address */
  nftMint: PublicKey;
  /** Event timestamp */
  timestamp: Date;
  /** Transaction signature */
  signature: string;
  /** From address */
  from?: PublicKey;
  /** To address */
  to?: PublicKey;
  /** Event data */
  data: Record<string, any>;
  /** Block number */
  blockNumber: number;
}

// ============================================================================
// API RESPONSE TYPES
// ============================================================================

/**
 * Paginated NFT response
 */
export interface PaginatedNFTResponse<T> {
  /** Items */
  items: T[];
  /** Total count */
  total: number;
  /** Page size */
  pageSize: number;
  /** Current page */
  page: number;
  /** Total pages */
  totalPages: number;
  /** Has next page */
  hasNext: boolean;
  /** Has previous page */
  hasPrevious: boolean;
}

/**
 * NFT search filters
 */
export interface NFTSearchFilters {
  /** Collection filter */
  collection?: PublicKey;
  /** Category filter */
  category?: NFTCategory;
  /** Rarity filter */
  rarity?: CardRarity;
  /** Price range */
  priceRange?: {
    min: BN;
    max: BN;
    currency: string;
  };
  /** Trait filters */
  traits?: Record<string, (string | number)[]>;
  /** Owner filter */
  owner?: PublicKey;
  /** Listed only */
  listedOnly?: boolean;
  /** Sort by */
  sortBy?: 'price' | 'rarity' | 'recent' | 'name';
  /** Sort direction */
  sortDirection?: 'asc' | 'desc';
}

// ============================================================================
// UTILITY FUNCTIONS & CONSTANTS
// ============================================================================

/**
 * NFT-related constants
 */
export const NFT_CONSTANTS = {
  // Marketplace
  MARKETPLACE_FEE_BPS: 250, // 2.5%
  CREATOR_ROYALTY_BPS: 500, // 5%
  
  // Special Cards
  MAX_ACTIVE_CARDS: 5,
  CARD_COOLDOWN_SECONDS: 86400, // 24 hours
  
  // Rarity thresholds
  RARITY_THRESHOLDS: {
    COMMON: 0,
    UNCOMMON: 10,
    RARE: 25,
    EPIC: 50,
    LEGENDARY: 80,
    MYTHIC: 95
  },
  
  // Badge levels
  BADGE_LEVEL_THRESHOLDS: {
    BRONZE: 0,
    SILVER: 1000,
    GOLD: 5000,
    PLATINUM: 20000,
    DIAMOND: 50000,
    MYTHIC: 100000
  }
} as const;

/**
 * Type guards for NFT types
 */
export const isSpecialCard = (nft: NFTMetadata): nft is SpecialCard => {
  return 'category' in nft && 'effects' in nft;
};

export const isProfileBadge = (nft: NFTMetadata): nft is ProfileBadge => {
  return 'level' in nft && 'permanentBonuses' in nft;
};

export const isAchievementNFT = (nft: NFTMetadata): nft is AchievementNFT => {
  return 'unlockCriteria' in nft && 'rewards' in nft;
};

// ============================================================================
// EXPORT ALL TYPES
// ============================================================================

export type {
  // Core types
  NFTCollection,
  NFTMetadata,
  NFTAttribute,
  NFTCreator,
  NFTRoyalty,
  RoyaltyRecipient,
  
  // Special Cards
  SpecialCard,
  CardEffect,
  CardUsage,
  CardSynergy,
  CardPrerequisite,
  CardEvolution,
  EvolutionRequirement,
  EvolutionNode,
  EvolutionCost,
  
  // Profile Badges
  ProfileBadge,
  BadgeBonus,
  BadgeRequirement,
  BadgeProgression,
  ProgressionEvent,
  BadgeDisplay,
  BadgeAnimation,
  
  // Achievements
  AchievementNFT,
  AchievementCriteria,
  AchievementReward,
  AchievementStats,
  
  // Marketplace
  NFTListing,
  AuctionDetails,
  ListingMetadata,
  BundleInfo,
  Bid,
  NFTSale,
  RoyaltyPayment,
  
  // Utility & Gamification
  NFTUtility,
  UtilityEffect,
  UtilityConstraint,
  UtilityCondition,
  UpgradePath,
  UpgradeRequirement,
  UpgradeCost,
  
  // Rarity & Market Data
  NFTRarity,
  TraitRarity,
  NFTMarketData,
  PricePoint,
  NFTOwnership,
  
  // Collections
  CollectionTrait,
  CollectionStats,
  
  // Events
  NFTEvent,
  
  // API
  PaginatedNFTResponse,
  NFTSearchFilters
};

// ============================================================================
// ADDITIONAL UTILITY TYPES
// ============================================================================

/**
 * NFT transaction context
 */
export interface NFTTransactionContext {
  /** Transaction type */
  type: 'mint' | 'transfer' | 'sale' | 'use' | 'upgrade' | 'burn';
  /** NFT involved */
  nft: PublicKey;
  /** User initiating transaction */
  user: PublicKey;
  /** Transaction metadata */
  metadata: Record<string, any>;
  /** Gas estimation */
  gasEstimate?: BN;
  /** Priority fee */
  priorityFee?: BN;
}

/**
 * NFT validation result
 */
export interface NFTValidationResult {
  /** Is valid */
  valid: boolean;
  /** Validation errors */
  errors: ValidationError[];
  /** Validation warnings */
  warnings: ValidationWarning[];
  /** Validation score */
  score: number;
}

/**
 * Validation error
 */
export interface ValidationError {
  /** Error code */
  code: string;
  /** Error message */
  message: string;
  /** Error field */
  field?: string;
  /** Error severity */
  severity: 'low' | 'medium' | 'high' | 'critical';
}

/**
 * Validation warning
 */
export interface ValidationWarning {
  /** Warning code */
  code: string;
  /** Warning message */
  message: string;
  /** Warning field */
  field?: string;
  /** Recommendation */
  recommendation?: string;
}

/**
 * NFT analytics data
 */
export interface NFTAnalytics {
  /** Collection performance */
  collectionPerformance: CollectionPerformance;
  /** User engagement */
  userEngagement: UserEngagement;
  /** Market trends */
  marketTrends: MarketTrend[];
  /** Usage statistics */
  usageStats: UsageStatistics;
}

/**
 * Collection performance metrics
 */
export interface CollectionPerformance {
  /** Collection address */
  collection: PublicKey;
  /** Performance period */
  period: 'daily' | 'weekly' | 'monthly' | 'yearly';
  /** Volume traded */
  volume: BN;
  /** Number of sales */
  sales: number;
  /** Unique buyers */
  uniqueBuyers: number;
  /** Average sale price */
  avgSalePrice: BN;
  /** Floor price change */
  floorPriceChange: number;
  /** Volume change */
  volumeChange: number;
}

/**
 * User engagement metrics
 */
export interface UserEngagement {
  /** Active users */
  activeUsers: number;
  /** New users */
  newUsers: number;
  /** Returning users */
  returningUsers: number;
  /** Engagement rate */
  engagementRate: number;
  /** Average session duration */
  avgSessionDuration: number;
  /** Page views */
  pageViews: number;
}

/**
 * Market trend data
 */
export interface MarketTrend {
  /** Trend category */
  category: 'price' | 'volume' | 'rarity' | 'utility';
  /** Trend direction */
  direction: 'up' | 'down' | 'stable';
  /** Trend strength */
  strength: number;
  /** Trend description */
  description: string;
  /** Supporting data */
  data: Record<string, any>;
}

/**
 * Usage statistics
 */
export interface UsageStatistics {
  /** Card usage stats */
  cardUsage: CardUsageStats;
  /** Badge display stats */
  badgeDisplay: BadgeDisplayStats;
  /** Achievement unlock stats */
  achievementUnlocks: AchievementUnlockStats;
}

/**
 * Card usage statistics
 */
export interface CardUsageStats {
  /** Most used cards */
  mostUsed: Array<{
    card: PublicKey;
    usageCount: number;
  }>;
  /** Usage by category */
  byCategory: Record<SpecialCardCategory, number>;
  /** Usage by rarity */
  byRarity: Record<CardRarity, number>;
  /** Daily usage trend */
  dailyTrend: Array<{
    date: Date;
    count: number;
  }>;
}

/**
 * Badge display statistics
 */
export interface BadgeDisplayStats {
  /** Most displayed badges */
  mostDisplayed: Array<{
    badge: PublicKey;
    displayCount: number;
  }>;
  /** Display by level */
  byLevel: Record<BadgeLevel, number>;
  /** Custom name usage */
  customNameUsage: number;
}

/**
 * Achievement unlock statistics
 */
export interface AchievementUnlockStats {
  /** Recent unlocks */
  recentUnlocks: Array<{
    achievement: PublicKey;
    unlockedBy: PublicKey;
    timestamp: Date;
  }>;
  /** Unlock rate by category */
  byCategory: Record<AchievementCategory, {
    unlocks: number;
    rate: number;
  }>;
  /** Hardest to unlock */
  hardestToUnlock: Array<{
    achievement: PublicKey;
    unlockRate: number;
  }>;
}

// ============================================================================
// INTEGRATION TYPES
// ============================================================================

/**
 * Social media integration for NFTs
 */
export interface NFTSocialIntegration {
  /** Platform integrations */
  platforms: SocialPlatformIntegration[];
  /** Sharing templates */
  sharingTemplates: SharingTemplate[];
  /** Social proofs */
  socialProofs: SocialProof[];
}

/**
 * Social platform integration
 */
export interface SocialPlatformIntegration {
  /** Platform name */
  platform: 'instagram' | 'tiktok' | 'youtube' | 'facebook' | 'twitter';
  /** Integration enabled */
  enabled: boolean;
  /** Display settings */
  display: {
    showInBio: boolean;
    showInPosts: boolean;
    customMessage?: string;
  };
  /** Verification status */
  verified: boolean;
}

/**
 * Sharing template
 */
export interface SharingTemplate {
  /** Template ID */
  id: string;
  /** Template name */
  name: string;
  /** Template content */
  content: string;
  /** Supported platforms */
  platforms: string[];
  /** Template variables */
  variables: string[];
}

/**
 * Social proof
 */
export interface SocialProof {
  /** Proof type */
  type: 'ownership' | 'achievement' | 'rarity' | 'value';
  /** Proof data */
  data: any;
  /** Verification status */
  verified: boolean;
  /** Public visibility */
  public: boolean;
}

/**
 * NFT bridge information for cross-chain
 */
export interface NFTBridgeInfo {
  /** Source chain */
  sourceChain: string;
  /** Target chain */
  targetChain: string;
  /** Bridge contract */
  bridgeContract: PublicKey;
  /** Bridge status */
  status: 'pending' | 'confirmed' | 'failed';
  /** Bridge transaction */
  bridgeTx?: string;
  /** Bridge fee */
  bridgeFee: BN;
  /** Estimated time */
  estimatedTime: number;
}

// ============================================================================
// ERROR TYPES
// ============================================================================

/**
 * NFT-related error types
 */
export enum NFTErrorCode {
  // General errors
  INVALID_NFT = 'INVALID_NFT',
  NFT_NOT_FOUND = 'NFT_NOT_FOUND',
  INVALID_COLLECTION = 'INVALID_COLLECTION',
  
  // Ownership errors
  NOT_OWNER = 'NOT_OWNER',
  UNAUTHORIZED = 'UNAUTHORIZED',
  TRANSFER_RESTRICTED = 'TRANSFER_RESTRICTED',
  
  // Marketplace errors
  INVALID_PRICE = 'INVALID_PRICE',
  LISTING_EXPIRED = 'LISTING_EXPIRED',
  INSUFFICIENT_FUNDS = 'INSUFFICIENT_FUNDS',
  AUCTION_ENDED = 'AUCTION_ENDED',
  
  // Card errors
  CARD_ON_COOLDOWN = 'CARD_ON_COOLDOWN',
  CARD_ALREADY_USED = 'CARD_ALREADY_USED',
  INVALID_CARD_USAGE = 'INVALID_CARD_USAGE',
  PREREQUISITE_NOT_MET = 'PREREQUISITE_NOT_MET',
  
  // Evolution errors
  CANNOT_EVOLVE = 'CANNOT_EVOLVE',
  EVOLUTION_REQUIREMENTS_NOT_MET = 'EVOLUTION_REQUIREMENTS_NOT_MET',
  INSUFFICIENT_MATERIALS = 'INSUFFICIENT_MATERIALS',
  
  // Validation errors
  INVALID_METADATA = 'INVALID_METADATA',
  INVALID_ATTRIBUTES = 'INVALID_ATTRIBUTES',
  DUPLICATE_NFT = 'DUPLICATE_NFT'
}

/**
 * NFT error details
 */
export interface NFTError {
  /** Error code */
  code: NFTErrorCode;
  /** Error message */
  message: string;
  /** Additional details */
  details?: Record<string, any>;
  /** Error timestamp */
  timestamp: Date;
  /** Context information */
  context?: NFTTransactionContext;
}

// ============================================================================
// FINAL EXPORTS
// ============================================================================

export {
  // Enums
  SpecialCardCategory,
  CardRarity,
  EffectType,
  EffectTarget,
  BadgeLevel,
  BadgeType,
  AchievementCategory,
  ListingStatus,
  AuctionType,
  NFTCategory,
  NFTEventType,
  NFTErrorCode
};

// Re-export constants
export { NFT_CONSTANTS };

// Re-export type guards
export { isSpecialCard, isProfileBadge, isAchievementNFT };

/**
 * Default export with all NFT-related functionality
 */
export default {
  // Constants
  CONSTANTS: NFT_CONSTANTS,
  
  // Type guards
  isSpecialCard,
  isProfileBadge,
  isAchievementNFT,
  
  // Enums
  SpecialCardCategory,
  CardRarity,
  EffectType,
  EffectTarget,
  BadgeLevel,
  BadgeType,
  AchievementCategory,
  ListingStatus,
  AuctionType,
  NFTCategory,
  NFTEventType,
  NFTErrorCode
};
