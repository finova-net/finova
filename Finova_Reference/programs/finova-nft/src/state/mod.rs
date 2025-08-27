// programs/finova-nft/src/state/mod.rs

use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, TokenAccount};

pub mod collection;
pub mod nft_metadata;
pub mod special_card;
pub mod marketplace;

pub use collection::*;
pub use nft_metadata::*;
pub use special_card::*;
pub use marketplace::*;

/// Global NFT program configuration and statistics
#[account]
#[derive(Debug)]
pub struct NftProgramConfig {
    /// Program authority with upgrade capabilities
    pub authority: Pubkey,
    /// Treasury account for marketplace fees
    pub treasury: Pubkey,
    /// Fee basis points for marketplace transactions (e.g., 250 = 2.5%)
    pub marketplace_fee_bps: u16,
    /// Fee basis points for royalties (e.g., 500 = 5%)
    pub royalty_fee_bps: u16,
    /// Minimum price for NFT listings (in lamports)
    pub min_listing_price: u64,
    /// Maximum price for NFT listings (in lamports)
    pub max_listing_price: u64,
    /// Total number of collections created
    pub total_collections: u64,
    /// Total number of NFTs minted
    pub total_nfts_minted: u64,
    /// Total number of special cards minted
    pub total_special_cards: u64,
    /// Total marketplace volume (in lamports)
    pub total_marketplace_volume: u64,
    /// Emergency pause flag for the entire NFT system
    pub is_paused: bool,
    /// Version for upgrades compatibility
    pub version: u8,
    /// Reserved space for future upgrades
    pub reserved: [u8; 64],
}

impl NftProgramConfig {
    pub const SPACE: usize = 8 + // discriminator
        32 + // authority
        32 + // treasury
        2 +  // marketplace_fee_bps
        2 +  // royalty_fee_bps
        8 +  // min_listing_price
        8 +  // max_listing_price
        8 +  // total_collections
        8 +  // total_nfts_minted
        8 +  // total_special_cards
        8 +  // total_marketplace_volume
        1 +  // is_paused
        1 +  // version
        64;  // reserved

    /// Initialize new NFT program configuration
    pub fn initialize(
        &mut self,
        authority: Pubkey,
        treasury: Pubkey,
        marketplace_fee_bps: u16,
        royalty_fee_bps: u16,
        min_listing_price: u64,
        max_listing_price: u64,
    ) -> Result<()> {
        require!(marketplace_fee_bps <= 1000, ErrorCode::InvalidFeeRate); // Max 10%
        require!(royalty_fee_bps <= 1000, ErrorCode::InvalidFeeRate); // Max 10%
        require!(min_listing_price > 0, ErrorCode::InvalidPrice);
        require!(max_listing_price >= min_listing_price, ErrorCode::InvalidPrice);

        self.authority = authority;
        self.treasury = treasury;
        self.marketplace_fee_bps = marketplace_fee_bps;
        self.royalty_fee_bps = royalty_fee_bps;
        self.min_listing_price = min_listing_price;
        self.max_listing_price = max_listing_price;
        self.total_collections = 0;
        self.total_nfts_minted = 0;
        self.total_special_cards = 0;
        self.total_marketplace_volume = 0;
        self.is_paused = false;
        self.version = 1;
        self.reserved = [0; 64];

        Ok(())
    }

    /// Update configuration (authority only)
    pub fn update_config(
        &mut self,
        marketplace_fee_bps: Option<u16>,
        royalty_fee_bps: Option<u16>,
        min_listing_price: Option<u64>,
        max_listing_price: Option<u64>,
        treasury: Option<Pubkey>,
    ) -> Result<()> {
        if let Some(fee) = marketplace_fee_bps {
            require!(fee <= 1000, ErrorCode::InvalidFeeRate);
            self.marketplace_fee_bps = fee;
        }

        if let Some(fee) = royalty_fee_bps {
            require!(fee <= 1000, ErrorCode::InvalidFeeRate);
            self.royalty_fee_bps = fee;
        }

        if let Some(price) = min_listing_price {
            require!(price > 0, ErrorCode::InvalidPrice);
            self.min_listing_price = price;
        }

        if let Some(price) = max_listing_price {
            require!(price >= self.min_listing_price, ErrorCode::InvalidPrice);
            self.max_listing_price = price;
        }

        if let Some(new_treasury) = treasury {
            self.treasury = new_treasury;
        }

        Ok(())
    }

    /// Increment collection counter
    pub fn increment_collections(&mut self) {
        self.total_collections = self.total_collections.saturating_add(1);
    }

    /// Increment NFT counter
    pub fn increment_nfts(&mut self) {
        self.total_nfts_minted = self.total_nfts_minted.saturating_add(1);
    }

    /// Increment special card counter
    pub fn increment_special_cards(&mut self) {
        self.total_special_cards = self.total_special_cards.saturating_add(1);
    }

    /// Add to marketplace volume
    pub fn add_marketplace_volume(&mut self, amount: u64) {
        self.total_marketplace_volume = self.total_marketplace_volume.saturating_add(amount);
    }

    /// Toggle emergency pause
    pub fn toggle_pause(&mut self) {
        self.is_paused = !self.is_paused;
    }

    /// Check if system is not paused
    pub fn require_not_paused(&self) -> Result<()> {
        require!(!self.is_paused, ErrorCode::SystemPaused);
        Ok(())
    }
}

/// User's NFT portfolio and statistics
#[account]
#[derive(Debug)]
pub struct UserNftProfile {
    /// Owner of this profile
    pub owner: Pubkey,
    /// Total NFTs owned by this user
    pub total_nfts_owned: u32,
    /// Total special cards owned
    pub total_special_cards_owned: u32,
    /// Total NFTs created by this user
    pub total_nfts_created: u32,
    /// Total marketplace sales volume
    pub total_sales_volume: u64,
    /// Total marketplace purchases volume
    pub total_purchases_volume: u64,
    /// Number of successful sales
    pub successful_sales_count: u32,
    /// Number of successful purchases
    pub successful_purchases_count: u32,
    /// Creator reputation score (0-1000)
    pub creator_reputation: u16,
    /// Collector reputation score (0-1000)
    pub collector_reputation: u16,
    /// Last activity timestamp
    pub last_activity_timestamp: i64,
    /// Special card usage statistics
    pub special_cards_used: u32,
    /// Total mining boost hours gained from cards
    pub total_boost_hours_gained: u64,
    /// Favorite collection (most owned NFTs from)
    pub favorite_collection: Option<Pubkey>,
    /// Profile creation timestamp
    pub created_at: i64,
    /// Profile level based on activity
    pub profile_level: u8,
    /// Experience points in NFT ecosystem
    pub nft_experience_points: u64,
    /// Reserved space for future features
    pub reserved: [u8; 32],
}

impl UserNftProfile {
    pub const SPACE: usize = 8 + // discriminator
        32 + // owner
        4 +  // total_nfts_owned
        4 +  // total_special_cards_owned
        4 +  // total_nfts_created
        8 +  // total_sales_volume
        8 +  // total_purchases_volume
        4 +  // successful_sales_count
        4 +  // successful_purchases_count
        2 +  // creator_reputation
        2 +  // collector_reputation
        8 +  // last_activity_timestamp
        4 +  // special_cards_used
        8 +  // total_boost_hours_gained
        33 + // favorite_collection (1 + 32)
        8 +  // created_at
        1 +  // profile_level
        8 +  // nft_experience_points
        32;  // reserved

    /// Initialize new user NFT profile
    pub fn initialize(&mut self, owner: Pubkey) -> Result<()> {
        self.owner = owner;
        self.total_nfts_owned = 0;
        self.total_special_cards_owned = 0;
        self.total_nfts_created = 0;
        self.total_sales_volume = 0;
        self.total_purchases_volume = 0;
        self.successful_sales_count = 0;
        self.successful_purchases_count = 0;
        self.creator_reputation = 500; // Start with neutral reputation
        self.collector_reputation = 500;
        self.last_activity_timestamp = Clock::get()?.unix_timestamp;
        self.special_cards_used = 0;
        self.total_boost_hours_gained = 0;
        self.favorite_collection = None;
        self.created_at = Clock::get()?.unix_timestamp;
        self.profile_level = 1;
        self.nft_experience_points = 0;
        self.reserved = [0; 32];

        Ok(())
    }

    /// Update NFT ownership count
    pub fn update_nft_ownership(&mut self, delta: i32) {
        if delta > 0 {
            self.total_nfts_owned = self.total_nfts_owned.saturating_add(delta as u32);
        } else {
            self.total_nfts_owned = self.total_nfts_owned.saturating_sub((-delta) as u32);
        }
        self.update_activity();
    }

    /// Update special card ownership count
    pub fn update_special_card_ownership(&mut self, delta: i32) {
        if delta > 0 {
            self.total_special_cards_owned = self.total_special_cards_owned.saturating_add(delta as u32);
        } else {
            self.total_special_cards_owned = self.total_special_cards_owned.saturating_sub((-delta) as u32);
        }
        self.update_activity();
    }

    /// Record NFT creation
    pub fn record_nft_creation(&mut self) {
        self.total_nfts_created = self.total_nfts_created.saturating_add(1);
        self.add_experience_points(100); // Base XP for creating NFT
        self.update_activity();
    }

    /// Record successful sale
    pub fn record_sale(&mut self, amount: u64) {
        self.total_sales_volume = self.total_sales_volume.saturating_add(amount);
        self.successful_sales_count = self.successful_sales_count.saturating_add(1);
        self.improve_creator_reputation(5);
        self.add_experience_points(50);
        self.update_activity();
    }

    /// Record successful purchase
    pub fn record_purchase(&mut self, amount: u64) {
        self.total_purchases_volume = self.total_purchases_volume.saturating_add(amount);
        self.successful_purchases_count = self.successful_purchases_count.saturating_add(1);
        self.improve_collector_reputation(3);
        self.add_experience_points(25);
        self.update_activity();
    }

    /// Record special card usage
    pub fn record_special_card_usage(&mut self, boost_hours: u64) {
        self.special_cards_used = self.special_cards_used.saturating_add(1);
        self.total_boost_hours_gained = self.total_boost_hours_gained.saturating_add(boost_hours);
        self.add_experience_points(20);
        self.update_activity();
    }

    /// Add experience points and update level
    pub fn add_experience_points(&mut self, points: u64) {
        self.nft_experience_points = self.nft_experience_points.saturating_add(points);
        
        // Update level based on experience points
        let new_level = match self.nft_experience_points {
            0..=999 => 1,
            1000..=2499 => 2,
            2500..=4999 => 3,
            5000..=9999 => 4,
            10000..=19999 => 5,
            20000..=39999 => 6,
            40000..=79999 => 7,
            80000..=159999 => 8,
            160000..=319999 => 9,
            _ => 10,
        };

        if new_level > self.profile_level {
            self.profile_level = new_level;
        }
    }

    /// Improve creator reputation
    pub fn improve_creator_reputation(&mut self, points: u16) {
        self.creator_reputation = std::cmp::min(1000, self.creator_reputation.saturating_add(points));
    }

    /// Improve collector reputation
    pub fn improve_collector_reputation(&mut self, points: u16) {
        self.collector_reputation = std::cmp::min(1000, self.collector_reputation.saturating_add(points));
    }

    /// Decrease reputation (for bad behavior)
    pub fn decrease_reputation(&mut self, creator_penalty: u16, collector_penalty: u16) {
        self.creator_reputation = self.creator_reputation.saturating_sub(creator_penalty);
        self.collector_reputation = self.collector_reputation.saturating_sub(collector_penalty);
    }

    /// Update favorite collection based on ownership
    pub fn update_favorite_collection(&mut self, collection: Pubkey) {
        self.favorite_collection = Some(collection);
        self.update_activity();
    }

    /// Update last activity timestamp
    fn update_activity(&mut self) {
        self.last_activity_timestamp = Clock::get().unwrap().unix_timestamp;
    }

    /// Check if user is active (activity within last 30 days)
    pub fn is_active(&self) -> bool {
        let current_time = Clock::get().unwrap().unix_timestamp;
        current_time - self.last_activity_timestamp < 30 * 24 * 60 * 60 // 30 days
    }

    /// Get user's trading efficiency score
    pub fn get_trading_efficiency(&self) -> u16 {
        if self.successful_purchases_count == 0 {
            return 500; // Neutral score
        }

        let success_rate = (self.successful_sales_count * 100) / self.successful_purchases_count;
        std::cmp::min(1000, success_rate as u16 * 10)
    }

    /// Get creator tier based on reputation and activity
    pub fn get_creator_tier(&self) -> CreatorTier {
        match (self.creator_reputation, self.total_nfts_created) {
            (900..=1000, 50..) => CreatorTier::Legendary,
            (800..=899, 25..) => CreatorTier::Epic,
            (700..=799, 10..) => CreatorTier::Rare,
            (600..=699, 5..) => CreatorTier::Uncommon,
            _ => CreatorTier::Common,
        }
    }

    /// Get collector tier based on reputation and collection size
    pub fn get_collector_tier(&self) -> CollectorTier {
        match (self.collector_reputation, self.total_nfts_owned) {
            (900..=1000, 100..) => CollectorTier::Whale,
            (800..=899, 50..) => CollectorTier::Collector,
            (700..=799, 25..) => CollectorTier::Enthusiast,
            (600..=699, 10..) => CollectorTier::Hobbyist,
            _ => CollectorTier::Beginner,
        }
    }
}

/// Creator tier enumeration
#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone, PartialEq)]
pub enum CreatorTier {
    Common,
    Uncommon,
    Rare,
    Epic,
    Legendary,
}

/// Collector tier enumeration
#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone, PartialEq)]
pub enum CollectorTier {
    Beginner,
    Hobbyist,
    Enthusiast,
    Collector,
    Whale,
}

/// Global marketplace statistics
#[account]
#[derive(Debug)]
pub struct MarketplaceStats {
    /// Total number of listings ever created
    pub total_listings: u64,
    /// Total number of successful sales
    pub total_sales: u64,
    /// Total trading volume in lamports
    pub total_volume: u64,
    /// Total fees collected in lamports
    pub total_fees_collected: u64,
    /// Average sale price in lamports
    pub average_sale_price: u64,
    /// Highest sale price in lamports
    pub highest_sale_price: u64,
    /// Most active collection (by volume)
    pub most_active_collection: Option<Pubkey>,
    /// Total number of active listings
    pub active_listings: u32,
    /// Total number of unique traders
    pub unique_traders: u32,
    /// Last updated timestamp
    pub last_updated: i64,
    /// Reserved space for future statistics
    pub reserved: [u8; 64],
}

impl MarketplaceStats {
    pub const SPACE: usize = 8 + // discriminator
        8 +  // total_listings
        8 +  // total_sales
        8 +  // total_volume
        8 +  // total_fees_collected
        8 +  // average_sale_price
        8 +  // highest_sale_price
        33 + // most_active_collection (1 + 32)
        4 +  // active_listings
        4 +  // unique_traders
        8 +  // last_updated
        64;  // reserved

    /// Initialize marketplace statistics
    pub fn initialize(&mut self) -> Result<()> {
        self.total_listings = 0;
        self.total_sales = 0;
        self.total_volume = 0;
        self.total_fees_collected = 0;
        self.average_sale_price = 0;
        self.highest_sale_price = 0;
        self.most_active_collection = None;
        self.active_listings = 0;
        self.unique_traders = 0;
        self.last_updated = Clock::get()?.unix_timestamp;
        self.reserved = [0; 64];

        Ok(())
    }

    /// Record new listing
    pub fn record_listing(&mut self) {
        self.total_listings = self.total_listings.saturating_add(1);
        self.active_listings = self.active_listings.saturating_add(1);
        self.update_timestamp();
    }

    /// Record successful sale
    pub fn record_sale(&mut self, price: u64, fees: u64) {
        self.total_sales = self.total_sales.saturating_add(1);
        self.total_volume = self.total_volume.saturating_add(price);
        self.total_fees_collected = self.total_fees_collected.saturating_add(fees);
        self.active_listings = self.active_listings.saturating_sub(1);

        // Update average price
        if self.total_sales > 0 {
            self.average_sale_price = self.total_volume / self.total_sales;
        }

        // Update highest sale price
        if price > self.highest_sale_price {
            self.highest_sale_price = price;
        }

        self.update_timestamp();
    }

    /// Record listing cancellation
    pub fn record_cancellation(&mut self) {
        self.active_listings = self.active_listings.saturating_sub(1);
        self.update_timestamp();
    }

    /// Add unique trader
    pub fn add_unique_trader(&mut self) {
        self.unique_traders = self.unique_traders.saturating_add(1);
        self.update_timestamp();
    }

    /// Update most active collection
    pub fn update_most_active_collection(&mut self, collection: Pubkey) {
        self.most_active_collection = Some(collection);
        self.update_timestamp();
    }

    /// Update timestamp
    fn update_timestamp(&mut self) {
        self.last_updated = Clock::get().unwrap().unix_timestamp;
    }

    /// Get marketplace efficiency (sales/listings ratio)
    pub fn get_efficiency_ratio(&self) -> u16 {
        if self.total_listings == 0 {
            return 0;
        }
        std::cmp::min(10000, ((self.total_sales * 10000) / self.total_listings) as u16)
    }
}

/// Custom error codes for NFT program
#[error_code]
pub enum ErrorCode {
    #[msg("Invalid fee rate - must be <= 1000 basis points")]
    InvalidFeeRate,
    #[msg("Invalid price - must be > 0")]
    InvalidPrice,
    #[msg("System is currently paused")]
    SystemPaused,
    #[msg("Insufficient funds for transaction")]
    InsufficientFunds,
    #[msg("NFT not found or invalid")]
    InvalidNft,
    #[msg("Unauthorized access")]
    Unauthorized,
    #[msg("Collection not found")]
    CollectionNotFound,
    #[msg("Invalid metadata")]
    InvalidMetadata,
    #[msg("Marketplace listing not found")]
    ListingNotFound,
    #[msg("Special card already used")]
    CardAlreadyUsed,
    #[msg("Invalid card type")]
    InvalidCardType,
    #[msg("Card expired")]
    CardExpired,
    #[msg("Maximum supply reached")]
    MaxSupplyReached,
    #[msg("Invalid collection authority")]
    InvalidCollectionAuthority,
    #[msg("Reputation too low for this action")]
    InsufficientReputation,
    #[msg("Profile not found")]
    ProfileNotFound,
    #[msg("Invalid signature")]
    InvalidSignature,
    #[msg("Transaction already processed")]
    AlreadyProcessed,
    #[msg("Invalid timestamp")]
    InvalidTimestamp,
    #[msg("Data serialization error")]
    SerializationError,
    #[msg("Account already initialized")]
    AlreadyInitialized,
    #[msg("Account not initialized")]
    NotInitialized,
    #[msg("Invalid account owner")]
    InvalidAccountOwner,
    #[msg("Numerical overflow")]
    NumericalOverflow,
}

/// Utility functions for state management
pub mod utils {
    use super::*;

    /// Calculate marketplace fee
    pub fn calculate_marketplace_fee(price: u64, fee_bps: u16) -> u64 {
        (price as u128 * fee_bps as u128 / 10000) as u64
    }

    /// Calculate royalty fee
    pub fn calculate_royalty_fee(price: u64, royalty_bps: u16) -> u64 {
        (price as u128 * royalty_bps as u128 / 10000) as u64
    }

    /// Validate metadata URI format
    pub fn validate_metadata_uri(uri: &str) -> bool {
        uri.len() <= 200 && (uri.starts_with("https://") || uri.starts_with("ipfs://"))
    }

    /// Generate collection seed
    pub fn generate_collection_seed(creator: &Pubkey, name: &str) -> Vec<u8> {
        let mut seed = Vec::new();
        seed.extend_from_slice(b"collection");
        seed.extend_from_slice(creator.as_ref());
        seed.extend_from_slice(name.as_bytes());
        seed
    }

    /// Generate NFT seed
    pub fn generate_nft_seed(collection: &Pubkey, token_id: u64) -> Vec<u8> {
        let mut seed = Vec::new();
        seed.extend_from_slice(b"nft");
        seed.extend_from_slice(collection.as_ref());
        seed.extend_from_slice(&token_id.to_le_bytes());
        seed
    }

    /// Generate marketplace listing seed
    pub fn generate_listing_seed(nft_mint: &Pubkey, seller: &Pubkey) -> Vec<u8> {
        let mut seed = Vec::new();
        seed.extend_from_slice(b"listing");
        seed.extend_from_slice(nft_mint.as_ref());
        seed.extend_from_slice(seller.as_ref());
        seed
    }

    /// Generate user profile seed
    pub fn generate_user_profile_seed(owner: &Pubkey) -> Vec<u8> {
        let mut seed = Vec::new();
        seed.extend_from_slice(b"user_nft_profile");
        seed.extend_from_slice(owner.as_ref());
        seed
    }

    /// Check if timestamp is within valid range (not too old, not in future)
    pub fn is_valid_timestamp(timestamp: i64) -> bool {
        let current_time = Clock::get().unwrap().unix_timestamp;
        let one_year_seconds = 365 * 24 * 60 * 60;
        
        timestamp >= current_time - one_year_seconds && timestamp <= current_time + 300 // 5 minutes in future
    }

    /// Calculate experience points based on action type
    pub fn calculate_experience_points(action: NftAction) -> u64 {
        match action {
            NftAction::CreateCollection => 500,
            NftAction::MintNft => 100,
            NftAction::ListNft => 25,
            NftAction::BuyNft => 50,
            NftAction::SellNft => 75,
            NftAction::UseSpecialCard => 20,
            NftAction::CreateSpecialCard => 200,
        }
    }
}

/// NFT action types for experience calculation
#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone, PartialEq)]
pub enum NftAction {
    CreateCollection,
    MintNft,
    ListNft,
    BuyNft,
    SellNft,
    UseSpecialCard,
    CreateSpecialCard,
}
