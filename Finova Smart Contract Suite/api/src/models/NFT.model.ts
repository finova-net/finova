import { 
  Entity, 
  Column, 
  PrimaryGeneratedColumn, 
  CreateDateColumn, 
  UpdateDateColumn, 
  ManyToOne, 
  OneToMany, 
  JoinColumn, 
  Index,
  BeforeInsert,
  BeforeUpdate
} from 'typeorm';
import { IsEnum, IsString, IsNumber, IsBoolean, IsObject, IsOptional, Min, Max } from 'class-validator';
import { User } from './User.model';
import { Transaction } from './Transaction.model';

// NFT Categories from whitepaper
export enum NFTCategory {
  MINING_BOOST = 'mining_boost',
  XP_ACCELERATOR = 'xp_accelerator', 
  REFERRAL_POWER = 'referral_power',
  PROFILE_BADGE = 'profile_badge',
  ACHIEVEMENT = 'achievement',
  SPECIAL_CARD = 'special_card'
}

// Rarity levels affecting prices and effects
export enum NFTRarity {
  COMMON = 'common',
  UNCOMMON = 'uncommon', 
  RARE = 'rare',
  EPIC = 'epic',
  LEGENDARY = 'legendary',
  MYTHIC = 'mythic'
}

// NFT status for marketplace and usage tracking
export enum NFTStatus {
  ACTIVE = 'active',
  USED = 'used', // For single-use cards
  LISTED = 'listed', // On marketplace
  BURNED = 'burned', // Permanently destroyed
  LOCKED = 'locked' // In staking or special events
}

// Card types from whitepaper with specific effects
export enum SpecialCardType {
  // Mining Boost Cards
  DOUBLE_MINING = 'double_mining',
  TRIPLE_MINING = 'triple_mining', 
  MINING_FRENZY = 'mining_frenzy',
  ETERNAL_MINER = 'eternal_miner',
  
  // XP Accelerator Cards
  XP_DOUBLE = 'xp_double',
  STREAK_SAVER = 'streak_saver',
  LEVEL_RUSH = 'level_rush', 
  XP_MAGNET = 'xp_magnet',
  
  // Referral Power Cards
  REFERRAL_BOOST = 'referral_boost',
  NETWORK_AMPLIFIER = 'network_amplifier',
  AMBASSADOR_PASS = 'ambassador_pass',
  NETWORK_KING = 'network_king',
  
  // Profile Badges
  FINIZEN_BADGE = 'finizen_badge',
  CONTENT_KING = 'content_king', 
  AMBASSADOR_BADGE = 'ambassador_badge',
  DIAMOND_HANDS = 'diamond_hands'
}

// NFT Effects Configuration Interface
interface NFTEffect {
  type: 'mining' | 'xp' | 'referral' | 'staking' | 'special';
  multiplier: number; // 1.0 = no effect, 2.0 = double, etc.
  duration: number; // Hours, 0 = permanent
  stackable: boolean;
  maxStack?: number;
}

// Marketplace listing interface
interface MarketplaceListing {
  price: number; // In $FIN
  currency: 'FIN' | 'sFIN' | 'USDfin';
  listingDate: Date;
  expiryDate?: Date;
  buyNow: boolean;
  auction: boolean;
  minBid?: number;
  currentBid?: number;
  highestBidder?: string;
}

// NFT Metadata following Metaplex standards
interface NFTMetadata {
  name: string;
  description: string;
  image: string;
  animation_url?: string;
  external_url?: string;
  attributes: Array<{
    trait_type: string;
    value: string | number;
    display_type?: 'boost_number' | 'boost_percentage' | 'number' | 'date';
  }>;
  properties: {
    category: string;
    creators: Array<{
      address: string;
      share: number;
      verified?: boolean;
    }>;
  };
}

// Usage history for analytics and anti-abuse
interface UsageHistory {
  usedAt: Date;
  usedBy: string; // User ID
  context: string; // Where/how it was used
  effect: NFTEffect;
  transactionId?: string;
}

@Entity('nfts')
@Index(['owner_id', 'status'])
@Index(['category', 'rarity'])
@Index(['mint_address'])
@Index(['card_type'])
export class NFT {
  @PrimaryGeneratedColumn('uuid')
  id: string;

  @Column({ type: 'varchar', length: 88, unique: true })
  @Index()
  mint_address: string; // Solana mint address

  @Column({ type: 'varchar', length: 64 })
  @Index()
  collection_address: string; // Collection this NFT belongs to

  @Column({ type: 'uuid' })
  @Index()
  owner_id: string;

  @Column({ type: 'uuid', nullable: true })
  @Index()
  creator_id: string; // Original creator

  @Column({ 
    type: 'enum', 
    enum: NFTCategory,
    default: NFTCategory.SPECIAL_CARD
  })
  @IsEnum(NFTCategory)
  category: NFTCategory;

  @Column({ 
    type: 'enum', 
    enum: NFTRarity,
    default: NFTRarity.COMMON
  })
  @IsEnum(NFTRarity)
  rarity: NFTRarity;

  @Column({ 
    type: 'enum', 
    enum: NFTStatus,
    default: NFTStatus.ACTIVE
  })
  @IsEnum(NFTStatus)
  status: NFTStatus;

  @Column({ 
    type: 'enum', 
    enum: SpecialCardType,
    nullable: true
  })
  @IsOptional()
  @IsEnum(SpecialCardType)
  card_type?: SpecialCardType;

  @Column({ type: 'varchar', length: 100 })
  @IsString()
  name: string;

  @Column({ type: 'text' })
  @IsString()
  description: string;

  @Column({ type: 'varchar', length: 500 })
  @IsString()
  image_url: string;

  @Column({ type: 'varchar', length: 500, nullable: true })
  @IsOptional()
  @IsString()
  animation_url?: string;

  @Column({ type: 'jsonb' })
  @IsObject()
  metadata: NFTMetadata;

  @Column({ type: 'jsonb' })
  @IsObject()
  effects: NFTEffect[];

  @Column({ type: 'decimal', precision: 18, scale: 8, default: 0 })
  @IsNumber()
  @Min(0)
  mint_price: number; // Original mint price in $FIN

  @Column({ type: 'decimal', precision: 18, scale: 8, nullable: true })
  @IsOptional()
  @IsNumber()
  @Min(0)
  floor_price?: number; // Current floor price

  @Column({ type: 'decimal', precision: 18, scale: 8, nullable: true })
  @IsOptional()
  @IsNumber()
  @Min(0)
  last_sale_price?: number;

  @Column({ type: 'jsonb', nullable: true })
  @IsOptional()
  @IsObject()
  marketplace_listing?: MarketplaceListing;

  @Column({ type: 'boolean', default: true })
  @IsBoolean()
  is_tradeable: boolean;

  @Column({ type: 'boolean', default: false })
  @IsBoolean()
  is_single_use: boolean; // Cards that get burned after use

  @Column({ type: 'integer', default: 0 })
  @IsNumber()
  @Min(0)
  use_count: number; // How many times it's been used

  @Column({ type: 'integer', nullable: true })
  @IsOptional()
  @IsNumber()
  @Min(1)
  max_uses?: number; // Null = unlimited uses

  @Column({ type: 'jsonb', default: '[]' })
  @IsObject()
  usage_history: UsageHistory[];

  @Column({ type: 'timestamp', nullable: true })
  @IsOptional()
  active_until?: Date; // For temporary effects

  @Column({ type: 'timestamp', nullable: true })
  @IsOptional()
  cooldown_until?: Date; // Cooldown period before reuse

  @Column({ type: 'integer', default: 1 })
  @IsNumber()
  @Min(1)
  @Max(1000000)
  edition_number: number; // For limited editions

  @Column({ type: 'integer', nullable: true })
  @IsOptional()
  @IsNumber()
  @Min(1)
  total_supply?: number; // Total supply of this NFT type

  @Column({ type: 'jsonb', default: '{}' })
  @IsObject()
  stats: {
    total_mining_boost?: number;
    total_xp_gained?: number;
    total_rp_generated?: number;
    times_traded?: number;
    highest_sale_price?: number;
  };

  @Column({ type: 'boolean', default: false })
  @IsBoolean()
  is_verified: boolean; // Verified collection status

  @Column({ type: 'varchar', length: 64, nullable: true })
  @IsOptional()
  @IsString()
  verification_signature?: string; // Cryptographic verification

  @CreateDateColumn()
  created_at: Date;

  @UpdateDateColumn()
  updated_at: Date;

  @Column({ type: 'timestamp', nullable: true })
  @IsOptional()
  minted_at?: Date;

  @Column({ type: 'timestamp', nullable: true })
  @IsOptional()
  burned_at?: Date;

  // Relationships
  @ManyToOne(() => User, user => user.nfts, { onDelete: 'CASCADE' })
  @JoinColumn({ name: 'owner_id' })
  owner: User;

  @ManyToOne(() => User, { nullable: true })
  @JoinColumn({ name: 'creator_id' })
  creator?: User;

  @OneToMany(() => Transaction, transaction => transaction.nft)
  transactions: Transaction[];

  // Computed properties
  get is_expired(): boolean {
    return this.active_until ? new Date() > this.active_until : false;
  }

  get is_on_cooldown(): boolean {
    return this.cooldown_until ? new Date() < this.cooldown_until : false;
  }

  get is_usable(): boolean {
    return this.status === NFTStatus.ACTIVE && 
           !this.is_expired && 
           !this.is_on_cooldown &&
           (!this.max_uses || this.use_count < this.max_uses);
  }

  get current_effects(): NFTEffect[] {
    return this.effects.filter(effect => {
      if (effect.duration === 0) return true; // Permanent
      return !this.is_expired;
    });
  }

  get rarity_multiplier(): number {
    const multipliers = {
      [NFTRarity.COMMON]: 1.0,
      [NFTRarity.UNCOMMON]: 1.05,
      [NFTRarity.RARE]: 1.1,
      [NFTRarity.EPIC]: 1.2,
      [NFTRarity.LEGENDARY]: 1.35,
      [NFTRarity.MYTHIC]: 1.5
    };
    return multipliers[this.rarity] || 1.0;
  }

  // Methods for NFT operations
  canUse(userId: string): boolean {
    return this.owner_id === userId && this.is_usable;
  }

  calculateSynergyBonus(activeNFTs: NFT[]): number {
    const activeCount = activeNFTs.length;
    const rarityBonus = this.rarity_multiplier - 1.0;
    
    // Check for same category synergy
    const sameCategory = activeNFTs.filter(nft => nft.category === this.category).length;
    const categoryBonus = sameCategory >= 2 ? 0.15 : 0;
    
    // Check for all categories synergy
    const categories = new Set(activeNFTs.map(nft => nft.category));
    const allCategoryBonus = categories.size >= 3 ? 0.3 : 0;
    
    return 1.0 + (activeCount * 0.1) + rarityBonus + categoryBonus + allCategoryBonus;
  }

  use(userId: string, context: string): UsageHistory {
    if (!this.canUse(userId)) {
      throw new Error('NFT cannot be used');
    }

    const usage: UsageHistory = {
      usedAt: new Date(),
      usedBy: userId,
      context,
      effect: this.current_effects[0] // Primary effect
    };

    this.usage_history.push(usage);
    this.use_count++;

    // Handle single-use cards
    if (this.is_single_use) {
      this.status = NFTStatus.USED;
    }

    // Set cooldown if applicable
    if (this.card_type && this.getCooldownHours() > 0) {
      this.cooldown_until = new Date(Date.now() + this.getCooldownHours() * 60 * 60 * 1000);
    }

    return usage;
  }

  private getCooldownHours(): number {
    const cooldowns = {
      [SpecialCardType.DOUBLE_MINING]: 24,
      [SpecialCardType.TRIPLE_MINING]: 12,
      [SpecialCardType.MINING_FRENZY]: 48,
      [SpecialCardType.XP_DOUBLE]: 24,
      [SpecialCardType.LEVEL_RUSH]: 0, // Instant use
      [SpecialCardType.REFERRAL_BOOST]: 168, // 7 days
      [SpecialCardType.NETWORK_AMPLIFIER]: 24,
      [SpecialCardType.AMBASSADOR_PASS]: 48,
      [SpecialCardType.NETWORK_KING]: 72
    };
    return cooldowns[this.card_type] || 0;
  }

  // Marketplace operations
  listOnMarketplace(price: number, currency: 'FIN' | 'sFIN' | 'USDfin' = 'FIN', auction = false): void {
    if (!this.is_tradeable) {
      throw new Error('NFT is not tradeable');
    }

    this.marketplace_listing = {
      price,
      currency,
      listingDate: new Date(),
      buyNow: !auction,
      auction,
      minBid: auction ? price : undefined
    };
    this.status = NFTStatus.LISTED;
  }

  removeFromMarketplace(): void {
    this.marketplace_listing = undefined;
    this.status = NFTStatus.ACTIVE;
  }

  transfer(newOwnerId: string, price?: number): void {
    this.owner_id = newOwnerId;
    this.removeFromMarketplace();
    
    if (price) {
      this.last_sale_price = price;
      this.stats.times_traded = (this.stats.times_traded || 0) + 1;
      if (!this.stats.highest_sale_price || price > this.stats.highest_sale_price) {
        this.stats.highest_sale_price = price;
      }
    }
  }

  burn(): void {
    this.status = NFTStatus.BURNED;
    this.burned_at = new Date();
    this.removeFromMarketplace();
  }

  // Lifecycle hooks
  @BeforeInsert()
  setDefaults(): void {
    if (!this.minted_at) {
      this.minted_at = new Date();
    }
    if (!this.stats) {
      this.stats = {};
    }
    if (!this.usage_history) {
      this.usage_history = [];
    }
  }

  @BeforeUpdate()
  updateTimestamp(): void {
    this.updated_at = new Date();
  }

  // Validation methods
  validateMetadata(): boolean {
    return !!(
      this.metadata.name &&
      this.metadata.description &&
      this.metadata.image &&
      this.metadata.attributes &&
      Array.isArray(this.metadata.attributes)
    );
  }

  validateEffects(): boolean {
    return this.effects.every(effect => 
      effect.type && 
      typeof effect.multiplier === 'number' &&
      typeof effect.duration === 'number' &&
      typeof effect.stackable === 'boolean'
    );
  }

  // Factory methods for creating specific card types
  static createMiningBoostCard(type: SpecialCardType, rarity: NFTRarity): Partial<NFT> {
    const effects = {
      [SpecialCardType.DOUBLE_MINING]: { multiplier: 2.0, duration: 24 },
      [SpecialCardType.TRIPLE_MINING]: { multiplier: 3.0, duration: 12 },
      [SpecialCardType.MINING_FRENZY]: { multiplier: 6.0, duration: 4 },
      [SpecialCardType.ETERNAL_MINER]: { multiplier: 1.5, duration: 0 }
    };

    const effect = effects[type];
    return {
      category: NFTCategory.MINING_BOOST,
      card_type: type,
      rarity,
      is_single_use: type !== SpecialCardType.ETERNAL_MINER,
      effects: [{
        type: 'mining',
        multiplier: effect.multiplier,
        duration: effect.duration,
        stackable: false
      }]
    };
  }

  static createXPAcceleratorCard(type: SpecialCardType, rarity: NFTRarity): Partial<NFT> {
    const configs = {
      [SpecialCardType.XP_DOUBLE]: { multiplier: 2.0, duration: 24, stackable: true },
      [SpecialCardType.STREAK_SAVER]: { multiplier: 1.0, duration: 168, stackable: false }, // 7 days
      [SpecialCardType.LEVEL_RUSH]: { multiplier: 1.0, duration: 0, stackable: false }, // Instant XP
      [SpecialCardType.XP_MAGNET]: { multiplier: 4.0, duration: 48, stackable: false }
    };

    const config = configs[type];
    return {
      category: NFTCategory.XP_ACCELERATOR,
      card_type: type,
      rarity,
      is_single_use: type === SpecialCardType.LEVEL_RUSH,
      effects: [{
        type: 'xp',
        multiplier: config.multiplier,
        duration: config.duration,
        stackable: config.stackable,
        maxStack: config.stackable ? 3 : undefined
      }]
    };
  }
}
