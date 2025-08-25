// programs/finova-nft/src/state/collection.rs

use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};
use std::collections::HashMap;

use crate::constants::*;
use crate::errors::FinovaNftError;

/// Collection state for managing NFT collections in Finova Network
/// Supports multiple collection types including Special Cards, Achievement Badges, and Profile NFTs
#[account]
#[derive(Debug)]
pub struct Collection {
    /// Collection authority who can mint and manage the collection
    pub authority: Pubkey,
    
    /// Collection mint address (used for Metaplex collection standard)
    pub collection_mint: Pubkey,
    
    /// Collection metadata URI
    pub uri: String,
    
    /// Collection name
    pub name: String,
    
    /// Collection symbol
    pub symbol: String,
    
    /// Collection description
    pub description: String,
    
    /// Collection type (SpecialCard, Achievement, Profile, etc.)
    pub collection_type: CollectionType,
    
    /// Total supply cap (0 = unlimited)
    pub max_supply: u64,
    
    /// Current minted count
    pub current_supply: u64,
    
    /// Royalty percentage (basis points, e.g., 500 = 5%)
    pub royalty_bps: u16,
    
    /// Royalty recipient
    pub royalty_recipient: Pubkey,
    
    /// Collection configuration flags
    pub config: CollectionConfig,
    
    /// Rarity distribution settings
    pub rarity_config: RarityConfig,
    
    /// Price configuration for different card types
    pub price_config: PriceConfig,
    
    /// Utility bonuses for different rarities
    pub utility_bonuses: UtilityBonuses,
    
    /// Collection statistics
    pub stats: CollectionStats,
    
    /// Creation timestamp
    pub created_at: i64,
    
    /// Last update timestamp
    pub updated_at: i64,
    
    /// Bump seed for PDA
    pub bump: u8,
    
    /// Reserved space for future upgrades
    pub reserved: [u8; 128],
}

impl Collection {
    pub const LEN: usize = 8 + // discriminator
        32 + // authority
        32 + // collection_mint
        256 + // uri (max 256 chars)
        64 + // name (max 64 chars)
        16 + // symbol (max 16 chars)
        512 + // description (max 512 chars)
        1 + // collection_type
        8 + // max_supply
        8 + // current_supply
        2 + // royalty_bps
        32 + // royalty_recipient
        CollectionConfig::LEN +
        RarityConfig::LEN +
        PriceConfig::LEN +
        UtilityBonuses::LEN +
        CollectionStats::LEN +
        8 + // created_at
        8 + // updated_at
        1 + // bump
        128; // reserved

    /// Initialize a new collection
    pub fn initialize(
        &mut self,
        authority: Pubkey,
        collection_mint: Pubkey,
        params: InitializeCollectionParams,
        bump: u8,
    ) -> Result<()> {
        self.authority = authority;
        self.collection_mint = collection_mint;
        self.uri = params.uri;
        self.name = params.name;
        self.symbol = params.symbol;
        self.description = params.description;
        self.collection_type = params.collection_type;
        self.max_supply = params.max_supply;
        self.current_supply = 0;
        self.royalty_bps = params.royalty_bps;
        self.royalty_recipient = params.royalty_recipient;
        self.config = params.config;
        self.rarity_config = params.rarity_config;
        self.price_config = params.price_config;
        self.utility_bonuses = params.utility_bonuses;
        self.stats = CollectionStats::default();
        self.created_at = Clock::get()?.unix_timestamp;
        self.updated_at = self.created_at;
        self.bump = bump;
        self.reserved = [0; 128];

        Ok(())
    }

    /// Mint a new NFT from this collection
    pub fn mint_nft(&mut self, rarity: Rarity) -> Result<u64> {
        // Check supply limits
        if self.max_supply > 0 && self.current_supply >= self.max_supply {
            return Err(FinovaNftError::MaxSupplyReached.into());
        }

        // Check if minting is enabled
        if !self.config.minting_enabled {
            return Err(FinovaNftError::MintingDisabled.into());
        }

        // Update supply and stats
        self.current_supply += 1;
        self.stats.update_mint_stats(rarity)?;
        self.updated_at = Clock::get()?.unix_timestamp;

        Ok(self.current_supply)
    }

    /// Update collection metadata
    pub fn update_metadata(&mut self, params: UpdateMetadataParams) -> Result<()> {
        if let Some(uri) = params.uri {
            self.uri = uri;
        }
        if let Some(name) = params.name {
            self.name = name;
        }
        if let Some(description) = params.description {
            self.description = description;
        }

        self.updated_at = Clock::get()?.unix_timestamp;
        Ok(())
    }

    /// Update collection configuration
    pub fn update_config(&mut self, config: CollectionConfig) -> Result<()> {
        self.config = config;
        self.updated_at = Clock::get()?.unix_timestamp;
        Ok(())
    }

    /// Update price configuration
    pub fn update_prices(&mut self, price_config: PriceConfig) -> Result<()> {
        self.price_config = price_config;
        self.updated_at = Clock::get()?.unix_timestamp;
        Ok(())
    }

    /// Get mint price for a specific rarity
    pub fn get_mint_price(&self, rarity: Rarity) -> u64 {
        match rarity {
            Rarity::Common => self.price_config.common_price,
            Rarity::Uncommon => self.price_config.uncommon_price,
            Rarity::Rare => self.price_config.rare_price,
            Rarity::Epic => self.price_config.epic_price,
            Rarity::Legendary => self.price_config.legendary_price,
        }
    }

    /// Get utility bonus for a specific rarity
    pub fn get_utility_bonus(&self, rarity: Rarity) -> UtilityBonus {
        match rarity {
            Rarity::Common => self.utility_bonuses.common_bonus,
            Rarity::Uncommon => self.utility_bonuses.uncommon_bonus,
            Rarity::Rare => self.utility_bonuses.rare_bonus,
            Rarity::Epic => self.utility_bonuses.epic_bonus,
            Rarity::Legendary => self.utility_bonuses.legendary_bonus,
        }
    }

    /// Determine rarity based on random seed and distribution
    pub fn determine_rarity(&self, random_seed: u64) -> Rarity {
        let roll = random_seed % 10000; // 0-9999 for basis point precision
        
        if roll < self.rarity_config.legendary_chance_bps {
            Rarity::Legendary
        } else if roll < self.rarity_config.legendary_chance_bps + self.rarity_config.epic_chance_bps {
            Rarity::Epic
        } else if roll < self.rarity_config.legendary_chance_bps + 
                      self.rarity_config.epic_chance_bps + 
                      self.rarity_config.rare_chance_bps {
            Rarity::Rare
        } else if roll < self.rarity_config.legendary_chance_bps + 
                      self.rarity_config.epic_chance_bps + 
                      self.rarity_config.rare_chance_bps +
                      self.rarity_config.uncommon_chance_bps {
            Rarity::Uncommon
        } else {
            Rarity::Common
        }
    }

    /// Check if collection can be traded
    pub fn is_tradeable(&self) -> bool {
        self.config.tradeable
    }

    /// Check if collection supports burning
    pub fn is_burnable(&self) -> bool {
        self.config.burnable
    }

    /// Get collection floor price
    pub fn get_floor_price(&self) -> u64 {
        self.stats.floor_price
    }

    /// Update trading stats
    pub fn update_trading_stats(&mut self, sale_price: u64) -> Result<()> {
        self.stats.total_volume += sale_price;
        self.stats.total_trades += 1;
        
        // Update floor price
        if self.stats.floor_price == 0 || sale_price < self.stats.floor_price {
            self.stats.floor_price = sale_price;
        }
        
        // Update average price
        self.stats.average_price = self.stats.total_volume / self.stats.total_trades;
        
        self.updated_at = Clock::get()?.unix_timestamp;
        Ok(())
    }
}

/// Collection type enumeration
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum CollectionType {
    /// Special mining boost cards
    SpecialCard,
    /// Achievement and milestone badges
    Achievement,
    /// User profile customization NFTs
    Profile,
    /// Guild-specific NFTs
    Guild,
    /// Limited edition commemorative NFTs
    Commemorative,
    /// Utility NFTs with specific functions
    Utility,
}

/// NFT rarity levels
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq, Copy)]
pub enum Rarity {
    Common,
    Uncommon,
    Rare,
    Epic,
    Legendary,
}

/// Collection configuration flags
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, Default)]
pub struct CollectionConfig {
    /// Whether minting is currently enabled
    pub minting_enabled: bool,
    /// Whether NFTs can be traded
    pub tradeable: bool,
    /// Whether NFTs can be burned
    pub burnable: bool,
    /// Whether collection is verified
    pub verified: bool,
    /// Whether collection requires KYC to mint
    pub kyc_required: bool,
    /// Whether collection has whitelist
    pub whitelist_enabled: bool,
    /// Maximum mints per user (0 = unlimited)
    pub max_per_user: u16,
    /// Whether royalties are enforced
    pub royalty_enforced: bool,
}

impl CollectionConfig {
    pub const LEN: usize = 1 + 1 + 1 + 1 + 1 + 1 + 2 + 1;
}

/// Rarity distribution configuration
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct RarityConfig {
    /// Legendary chance in basis points (e.g., 100 = 1%)
    pub legendary_chance_bps: u16,
    /// Epic chance in basis points
    pub epic_chance_bps: u16,
    /// Rare chance in basis points
    pub rare_chance_bps: u16,
    /// Uncommon chance in basis points
    pub uncommon_chance_bps: u16,
    /// Common gets the remainder (10000 - sum of above)
}

impl RarityConfig {
    pub const LEN: usize = 2 + 2 + 2 + 2;

    /// Create default rarity distribution
    pub fn default() -> Self {
        Self {
            legendary_chance_bps: 100,   // 1%
            epic_chance_bps: 400,        // 4%
            rare_chance_bps: 1500,       // 15%
            uncommon_chance_bps: 3000,   // 30%
            // Common: 50%
        }
    }

    /// Validate that all chances sum to <= 10000
    pub fn validate(&self) -> Result<()> {
        let total = self.legendary_chance_bps + 
                   self.epic_chance_bps + 
                   self.rare_chance_bps + 
                   self.uncommon_chance_bps;
        
        if total > 10000 {
            return Err(FinovaNftError::InvalidRarityDistribution.into());
        }
        
        Ok(())
    }
}

/// Price configuration for different rarities
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct PriceConfig {
    /// Price in lamports for common NFTs
    pub common_price: u64,
    /// Price for uncommon NFTs
    pub uncommon_price: u64,
    /// Price for rare NFTs
    pub rare_price: u64,
    /// Price for epic NFTs
    pub epic_price: u64,
    /// Price for legendary NFTs
    pub legendary_price: u64,
    /// Currency type (SOL, USDC, FIN, etc.)
    pub currency: Currency,
}

impl PriceConfig {
    pub const LEN: usize = 8 + 8 + 8 + 8 + 8 + 1;

    /// Create default price configuration in FIN tokens
    pub fn default_fin_prices() -> Self {
        Self {
            common_price: 50_000_000,      // 50 FIN
            uncommon_price: 150_000_000,   // 150 FIN
            rare_price: 500_000_000,       // 500 FIN
            epic_price: 2_000_000_000,     // 2000 FIN
            legendary_price: 10_000_000_000, // 10000 FIN
            currency: Currency::FIN,
        }
    }
}

/// Currency types for pricing
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum Currency {
    SOL,
    USDC,
    FIN,
    SFIN,
}

/// Utility bonuses provided by NFTs
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, Copy)]
pub struct UtilityBonus {
    /// Mining rate multiplier (basis points, e.g., 10000 = 100% = 2x)
    pub mining_multiplier_bps: u16,
    /// XP gain multiplier
    pub xp_multiplier_bps: u16,
    /// RP gain multiplier
    pub rp_multiplier_bps: u16,
    /// Duration in hours (0 = permanent)
    pub duration_hours: u32,
    /// Staking reward bonus
    pub staking_bonus_bps: u16,
}

impl UtilityBonus {
    pub const LEN: usize = 2 + 2 + 2 + 4 + 2;
}

/// Utility bonuses for all rarities
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct UtilityBonuses {
    pub common_bonus: UtilityBonus,
    pub uncommon_bonus: UtilityBonus,
    pub rare_bonus: UtilityBonus,
    pub epic_bonus: UtilityBonus,
    pub legendary_bonus: UtilityBonus,
}

impl UtilityBonuses {
    pub const LEN: usize = UtilityBonus::LEN * 5;

    /// Create default utility bonuses
    pub fn default() -> Self {
        Self {
            common_bonus: UtilityBonus {
                mining_multiplier_bps: 2000,  // 20% bonus
                xp_multiplier_bps: 1000,      // 10% bonus
                rp_multiplier_bps: 500,       // 5% bonus
                duration_hours: 24,           // 24 hours
                staking_bonus_bps: 500,       // 5% bonus
            },
            uncommon_bonus: UtilityBonus {
                mining_multiplier_bps: 3500,  // 35% bonus
                xp_multiplier_bps: 2000,      // 20% bonus
                rp_multiplier_bps: 1000,      // 10% bonus
                duration_hours: 48,           // 48 hours
                staking_bonus_bps: 1000,      // 10% bonus
            },
            rare_bonus: UtilityBonus {
                mining_multiplier_bps: 5000,  // 50% bonus
                xp_multiplier_bps: 3000,      // 30% bonus
                rp_multiplier_bps: 2000,      // 20% bonus
                duration_hours: 72,           // 72 hours
                staking_bonus_bps: 2000,      // 20% bonus
            },
            epic_bonus: UtilityBonus {
                mining_multiplier_bps: 10000, // 100% bonus (2x)
                xp_multiplier_bps: 5000,      // 50% bonus
                rp_multiplier_bps: 3500,      // 35% bonus
                duration_hours: 168,          // 1 week
                staking_bonus_bps: 3500,      // 35% bonus
            },
            legendary_bonus: UtilityBonus {
                mining_multiplier_bps: 20000, // 200% bonus (3x)
                xp_multiplier_bps: 7500,      // 75% bonus
                rp_multiplier_bps: 5000,      // 50% bonus
                duration_hours: 0,            // Permanent
                staking_bonus_bps: 5000,      // 50% bonus
            },
        }
    }
}

/// Collection statistics
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, Default)]
pub struct CollectionStats {
    /// Total trading volume in lamports
    pub total_volume: u64,
    /// Total number of trades
    pub total_trades: u64,
    /// Current floor price
    pub floor_price: u64,
    /// Average sale price
    pub average_price: u64,
    /// Number of unique holders
    pub unique_holders: u32,
    /// Rarity counts
    pub common_minted: u32,
    pub uncommon_minted: u32,
    pub rare_minted: u32,
    pub epic_minted: u32,
    pub legendary_minted: u32,
    /// Total burned count
    pub total_burned: u32,
}

impl CollectionStats {
    pub const LEN: usize = 8 + 8 + 8 + 8 + 4 + 4 + 4 + 4 + 4 + 4 + 4;

    /// Update mint statistics
    pub fn update_mint_stats(&mut self, rarity: Rarity) -> Result<()> {
        match rarity {
            Rarity::Common => self.common_minted += 1,
            Rarity::Uncommon => self.uncommon_minted += 1,
            Rarity::Rare => self.rare_minted += 1,
            Rarity::Epic => self.epic_minted += 1,
            Rarity::Legendary => self.legendary_minted += 1,
        }
        Ok(())
    }

    /// Update burn statistics
    pub fn update_burn_stats(&mut self, rarity: Rarity) -> Result<()> {
        self.total_burned += 1;
        match rarity {
            Rarity::Common => self.common_minted = self.common_minted.saturating_sub(1),
            Rarity::Uncommon => self.uncommon_minted = self.uncommon_minted.saturating_sub(1),
            Rarity::Rare => self.rare_minted = self.rare_minted.saturating_sub(1),
            Rarity::Epic => self.epic_minted = self.epic_minted.saturating_sub(1),
            Rarity::Legendary => self.legendary_minted = self.legendary_minted.saturating_sub(1),
        }
        Ok(())
    }
}

/// Parameters for initializing a collection
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct InitializeCollectionParams {
    pub uri: String,
    pub name: String,
    pub symbol: String,
    pub description: String,
    pub collection_type: CollectionType,
    pub max_supply: u64,
    pub royalty_bps: u16,
    pub royalty_recipient: Pubkey,
    pub config: CollectionConfig,
    pub rarity_config: RarityConfig,
    pub price_config: PriceConfig,
    pub utility_bonuses: UtilityBonuses,
}

/// Parameters for updating collection metadata
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct UpdateMetadataParams {
    pub uri: Option<String>,
    pub name: Option<String>,
    pub description: Option<String>,
}

/// Collection whitelist entry
#[account]
#[derive(Debug)]
pub struct WhitelistEntry {
    /// The whitelisted user
    pub user: Pubkey,
    /// Collection this whitelist entry belongs to
    pub collection: Pubkey,
    /// Maximum mints allowed for this user
    pub max_mints: u16,
    /// Current mints used
    pub mints_used: u16,
    /// Whitelist tier (for different benefits)
    pub tier: WhitelistTier,
    /// Expiration timestamp (0 = no expiration)
    pub expires_at: i64,
    /// Whether the entry is active
    pub active: bool,
    /// Bump seed
    pub bump: u8,
}

impl WhitelistEntry {
    pub const LEN: usize = 8 + 32 + 32 + 2 + 2 + 1 + 8 + 1 + 1;

    /// Check if whitelist entry is valid
    pub fn is_valid(&self) -> bool {
        self.active && 
        (self.expires_at == 0 || self.expires_at > Clock::get().unwrap().unix_timestamp) &&
        self.mints_used < self.max_mints
    }

    /// Use a mint from this whitelist entry
    pub fn use_mint(&mut self) -> Result<()> {
        if !self.is_valid() {
            return Err(FinovaNftError::WhitelistExpiredOrExhausted.into());
        }

        self.mints_used += 1;
        Ok(())
    }
}

/// Whitelist tiers with different benefits
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum WhitelistTier {
    Bronze,
    Silver,
    Gold,
    Platinum,
    Diamond,
}

/// User mint tracking for per-user limits
#[account]
#[derive(Debug)]
pub struct UserMintTracker {
    /// User who minted
    pub user: Pubkey,
    /// Collection minted from
    pub collection: Pubkey,
    /// Number of mints from this collection
    pub mint_count: u16,
    /// Timestamp of first mint
    pub first_mint_at: i64,
    /// Timestamp of last mint
    pub last_mint_at: i64,
    /// Bump seed
    pub bump: u8,
}

impl UserMintTracker {
    pub const LEN: usize = 8 + 32 + 32 + 2 + 8 + 8 + 1;

    /// Initialize user mint tracker
    pub fn initialize(&mut self, user: Pubkey, collection: Pubkey, bump: u8) -> Result<()> {
        self.user = user;
        self.collection = collection;
        self.mint_count = 0;
        self.first_mint_at = 0;
        self.last_mint_at = 0;
        self.bump = bump;
        Ok(())
    }

    /// Record a new mint
    pub fn record_mint(&mut self) -> Result<()> {
        let now = Clock::get()?.unix_timestamp;
        
        if self.mint_count == 0 {
            self.first_mint_at = now;
        }
        
        self.mint_count += 1;
        self.last_mint_at = now;
        
        Ok(())
    }

    /// Check if user can mint based on collection limits
    pub fn can_mint(&self, collection: &Collection) -> bool {
        if collection.config.max_per_user == 0 {
            return true; // No limit
        }
        
        self.mint_count < collection.config.max_per_user
    }
}
