// programs/finova-nft/src/state/nft_metadata.rs

use anchor_lang::prelude::*;
use std::collections::HashMap;

/// NFT Metadata Account - Stores comprehensive metadata for Finova NFTs
#[account]
#[derive(Debug)]
pub struct NftMetadata {
    /// Collection address this NFT belongs to
    pub collection: Pubkey,
    
    /// Mint address of the NFT
    pub mint: Pubkey,
    
    /// Current owner of the NFT
    pub owner: Pubkey,
    
    /// Creator of the NFT
    pub creator: Pubkey,
    
    /// NFT name (max 32 characters)
    pub name: [u8; 32],
    
    /// NFT symbol (max 10 characters)
    pub symbol: [u8; 10],
    
    /// Description (max 200 characters)
    pub description: [u8; 200],
    
    /// Image URI (max 200 characters)
    pub image_uri: [u8; 200],
    
    /// External URL (max 200 characters)
    pub external_url: [u8; 200],
    
    /// NFT category/type
    pub nft_type: NftType,
    
    /// Rarity level
    pub rarity: NftRarity,
    
    /// Special attributes for different NFT types
    pub attributes: NftAttributes,
    
    /// Creation timestamp
    pub created_at: i64,
    
    /// Last updated timestamp
    pub updated_at: i64,
    
    /// Whether the NFT is transferable
    pub is_transferable: bool,
    
    /// Whether the NFT is mutable
    pub is_mutable: bool,
    
    /// Usage statistics
    pub usage_stats: UsageStats,
    
    /// Market data
    pub market_data: MarketData,
    
    /// Verification status
    pub verification: VerificationStatus,
    
    /// Reserved space for future upgrades
    pub reserved: [u8; 256],
}

impl NftMetadata {
    pub const LEN: usize = 8 + // discriminator
        32 + // collection
        32 + // mint
        32 + // owner
        32 + // creator
        32 + // name
        10 + // symbol
        200 + // description
        200 + // image_uri
        200 + // external_url
        1 + // nft_type
        1 + // rarity
        std::mem::size_of::<NftAttributes>() +
        8 + // created_at
        8 + // updated_at
        1 + // is_transferable
        1 + // is_mutable
        std::mem::size_of::<UsageStats>() +
        std::mem::size_of::<MarketData>() +
        std::mem::size_of::<VerificationStatus>() +
        256; // reserved
}

/// NFT Type enumeration
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum NftType {
    /// Mining boost cards (temporary effects)
    MiningCard {
        /// Boost percentage (e.g., 100 for 100% boost)
        boost_percentage: u16,
        /// Duration in hours
        duration_hours: u16,
        /// Maximum uses before card is consumed
        max_uses: u8,
    },
    
    /// XP accelerator cards
    XpCard {
        /// XP multiplier (e.g., 200 for 2x multiplier)
        xp_multiplier: u16,
        /// Duration in hours
        duration_hours: u16,
        /// Activity types affected
        affected_activities: u8, // Bitmask for activity types
    },
    
    /// Referral power cards
    ReferralCard {
        /// Referral bonus percentage
        bonus_percentage: u16,
        /// Duration in hours
        duration_hours: u16,
        /// Network levels affected (1-3)
        network_levels: u8,
    },
    
    /// Profile badge NFTs (permanent status)
    ProfileBadge {
        /// Badge level (Bronze=1, Silver=2, Gold=3, etc.)
        badge_level: u8,
        /// Permanent mining bonus percentage
        mining_bonus: u16,
        /// XP bonus percentage
        xp_bonus: u16,
        /// Special privileges bitmask
        privileges: u32,
    },
    
    /// Achievement NFTs (commemorative)
    Achievement {
        /// Achievement ID
        achievement_id: u32,
        /// Achievement category
        category: AchievementCategory,
        /// Timestamp when achievement was earned
        earned_at: i64,
        /// Associated bonus effects
        bonus_effects: BonusEffects,
    },
    
    /// Guild-related NFTs
    GuildNft {
        /// Guild identifier
        guild_id: u32,
        /// Member role in guild
        role: GuildRole,
        /// Contribution score
        contribution_score: u64,
        /// Joining timestamp
        joined_at: i64,
    },
    
    /// Event commemorative NFTs
    EventNft {
        /// Event identifier
        event_id: u32,
        /// Event type
        event_type: EventType,
        /// Participation level
        participation_level: u8,
        /// Event timestamp
        event_timestamp: i64,
    },
    
    /// Limited edition collector NFTs
    CollectorNft {
        /// Series number
        series: u16,
        /// Edition number within series
        edition: u32,
        /// Maximum supply in series
        max_supply: u32,
        /// Special collector benefits
        collector_benefits: CollectorBenefits,
    },
}

/// NFT Rarity levels
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum NftRarity {
    Common,
    Uncommon,
    Rare,
    Epic,
    Legendary,
    Mythic,
}

impl NftRarity {
    /// Get rarity multiplier for various calculations
    pub fn get_multiplier(&self) -> f64 {
        match self {
            NftRarity::Common => 1.0,
            NftRarity::Uncommon => 1.25,
            NftRarity::Rare => 1.5,
            NftRarity::Epic => 2.0,
            NftRarity::Legendary => 3.0,
            NftRarity::Mythic => 5.0,
        }
    }
    
    /// Get rarity bonus percentage
    pub fn get_bonus_percentage(&self) -> u16 {
        match self {
            NftRarity::Common => 0,
            NftRarity::Uncommon => 5,
            NftRarity::Rare => 10,
            NftRarity::Epic => 20,
            NftRarity::Legendary => 35,
            NftRarity::Mythic => 50,
        }
    }
}

/// General NFT Attributes
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct NftAttributes {
    /// Trait type and value pairs (max 10 traits)
    pub traits: [Trait; 10],
    /// Number of active traits
    pub trait_count: u8,
    /// Overall power/value score
    pub power_score: u32,
    /// Synergy bonus with other NFTs
    pub synergy_bonus: u16,
    /// Evolution stage (for upgradeable NFTs)
    pub evolution_stage: u8,
    /// Maximum evolution stage possible
    pub max_evolution: u8,
    /// Upgrade requirements bitmask
    pub upgrade_requirements: u32,
}

/// Individual trait definition
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct Trait {
    /// Trait type (e.g., "Power", "Duration", "Efficiency")
    pub trait_type: [u8; 32],
    /// Trait value (e.g., "100", "24 hours", "High")
    pub value: [u8; 64],
    /// Numeric value for calculations (if applicable)
    pub numeric_value: Option<u32>,
    /// Trait rarity (affects overall NFT value)
    pub trait_rarity: u8,
}

/// Achievement categories
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum AchievementCategory {
    Pioneer,      // Early adoption achievements
    Creator,      // Content creation achievements
    Social,       // Social engagement achievements
    Mining,       // Mining milestone achievements
    Network,      // Referral network achievements
    Guild,        // Guild participation achievements
    Special,      // Special event achievements
    Collector,    // NFT collection achievements
}

/// Bonus effects for achievements
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct BonusEffects {
    /// Permanent mining bonus percentage
    pub mining_bonus: u16,
    /// XP gain bonus percentage
    pub xp_bonus: u16,
    /// Referral bonus percentage
    pub referral_bonus: u16,
    /// Special privileges unlocked
    pub privileges: u32,
    /// Access to exclusive features
    pub exclusive_access: u32,
}

/// Guild roles
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum GuildRole {
    Member,
    Officer,
    Leader,
    Founder,
}

/// Event types
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum EventType {
    LaunchEvent,
    SeasonalEvent,
    CommunityEvent,
    CompetitionEvent,
    MilestoneEvent,
    PartnershipEvent,
}

/// Collector benefits
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct CollectorBenefits {
    /// Exclusive access to future drops
    pub early_access: bool,
    /// Discount on marketplace transactions
    pub marketplace_discount: u8,
    /// Special voting weight in governance
    pub governance_weight: u16,
    /// Access to collector-only events
    pub exclusive_events: bool,
    /// Royalty sharing percentage
    pub royalty_share: u8,
}

/// Usage statistics for NFTs
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct UsageStats {
    /// Total number of times used (for consumable NFTs)
    pub total_uses: u32,
    /// Remaining uses (for limited-use NFTs)
    pub remaining_uses: u32,
    /// Last used timestamp
    pub last_used: i64,
    /// Total value generated through use
    pub value_generated: u64,
    /// Efficiency rating based on usage
    pub efficiency_rating: u16,
    /// Usage pattern analysis
    pub usage_pattern: UsagePattern,
}

/// Usage pattern analysis
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct UsagePattern {
    /// Average time between uses (in seconds)
    pub avg_interval: u32,
    /// Peak usage hours (24-hour bitmask)
    pub peak_hours: u32,
    /// Usage frequency trend (increasing/decreasing)
    pub trend: i8,
    /// Optimal usage recommendations
    pub recommendations: u8,
}

/// Market data for NFTs
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct MarketData {
    /// Last sale price in $FIN tokens
    pub last_sale_price: u64,
    /// Highest sale price ever
    pub highest_sale_price: u64,
    /// Average sale price (last 10 sales)
    pub average_price: u64,
    /// Number of times traded
    pub trade_count: u32,
    /// Current listing price (if listed)
    pub current_listing: Option<u64>,
    /// Market sentiment score
    pub sentiment_score: i16,
    /// Liquidity rating
    pub liquidity_rating: u8,
    /// Price volatility index
    pub volatility_index: u16,
}

/// Verification status
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct VerificationStatus {
    /// Whether the NFT is verified by Finova
    pub is_verified: bool,
    /// Verification timestamp
    pub verified_at: Option<i64>,
    /// Verifier authority
    pub verifier: Option<Pubkey>,
    /// Verification level (1-5)
    pub verification_level: u8,
    /// Authenticity hash
    pub authenticity_hash: [u8; 32],
    /// Content verification status
    pub content_verified: bool,
    /// Metadata verification status
    pub metadata_verified: bool,
    /// Creator verification status
    pub creator_verified: bool,
}

impl NftMetadata {
    /// Initialize new NFT metadata
    pub fn initialize(
        &mut self,
        collection: Pubkey,
        mint: Pubkey,
        owner: Pubkey,
        creator: Pubkey,
        name: String,
        symbol: String,
        description: String,
        image_uri: String,
        external_url: String,
        nft_type: NftType,
        rarity: NftRarity,
        is_transferable: bool,
        is_mutable: bool,
    ) -> Result<()> {
        self.collection = collection;
        self.mint = mint;
        self.owner = owner;
        self.creator = creator;
        
        // Copy strings to fixed-size arrays
        self.copy_string_to_array(&name, &mut self.name)?;
        self.copy_string_to_array(&symbol, &mut self.symbol)?;
        self.copy_string_to_array(&description, &mut self.description)?;
        self.copy_string_to_array(&image_uri, &mut self.image_uri)?;
        self.copy_string_to_array(&external_url, &mut self.external_url)?;
        
        self.nft_type = nft_type;
        self.rarity = rarity;
        self.is_transferable = is_transferable;
        self.is_mutable = is_mutable;
        
        let current_time = Clock::get()?.unix_timestamp;
        self.created_at = current_time;
        self.updated_at = current_time;
        
        // Initialize default values
        self.attributes = NftAttributes::default();
        self.usage_stats = UsageStats::default();
        self.market_data = MarketData::default();
        self.verification = VerificationStatus::default();
        
        Ok(())
    }
    
    /// Update NFT metadata (only if mutable)
    pub fn update_metadata(
        &mut self,
        name: Option<String>,
        description: Option<String>,
        image_uri: Option<String>,
        external_url: Option<String>,
    ) -> Result<()> {
        require!(self.is_mutable, crate::errors::FinovaNftError::NotMutable);
        
        if let Some(name) = name {
            self.copy_string_to_array(&name, &mut self.name)?;
        }
        
        if let Some(description) = description {
            self.copy_string_to_array(&description, &mut self.description)?;
        }
        
        if let Some(image_uri) = image_uri {
            self.copy_string_to_array(&image_uri, &mut self.image_uri)?;
        }
        
        if let Some(external_url) = external_url {
            self.copy_string_to_array(&external_url, &mut self.external_url)?;
        }
        
        self.updated_at = Clock::get()?.unix_timestamp;
        Ok(())
    }
    
    /// Transfer ownership
    pub fn transfer_ownership(&mut self, new_owner: Pubkey) -> Result<()> {
        require!(self.is_transferable, crate::errors::FinovaNftError::NotTransferable);
        
        let old_owner = self.owner;
        self.owner = new_owner;
        self.updated_at = Clock::get()?.unix_timestamp;
        
        // Update market data
        self.market_data.trade_count = self.market_data.trade_count.saturating_add(1);
        
        Ok(())
    }
    
    /// Use NFT (for consumable NFTs)
    pub fn use_nft(&mut self) -> Result<u32> {
        self.usage_stats.total_uses = self.usage_stats.total_uses.saturating_add(1);
        
        if self.usage_stats.remaining_uses > 0 {
            self.usage_stats.remaining_uses = self.usage_stats.remaining_uses.saturating_sub(1);
        }
        
        self.usage_stats.last_used = Clock::get()?.unix_timestamp;
        self.updated_at = Clock::get()?.unix_timestamp;
        
        Ok(self.usage_stats.remaining_uses)
    }
    
    /// Add trait to NFT
    pub fn add_trait(&mut self, trait_type: String, value: String, numeric_value: Option<u32>, trait_rarity: u8) -> Result<()> {
        require!(self.is_mutable, crate::errors::FinovaNftError::NotMutable);
        require!(self.attributes.trait_count < 10, crate::errors::FinovaNftError::MaxTraitsReached);
        
        let trait_index = self.attributes.trait_count as usize;
        let mut trait_obj = Trait::default();
        
        self.copy_string_to_array(&trait_type, &mut trait_obj.trait_type)?;
        self.copy_string_to_array(&value, &mut trait_obj.value)?;
        trait_obj.numeric_value = numeric_value;
        trait_obj.trait_rarity = trait_rarity;
        
        self.attributes.traits[trait_index] = trait_obj;
        self.attributes.trait_count += 1;
        self.updated_at = Clock::get()?.unix_timestamp;
        
        Ok(())
    }
    
    /// Update market price
    pub fn update_market_price(&mut self, new_price: u64) -> Result<()> {
        if new_price > self.market_data.highest_sale_price {
            self.market_data.highest_sale_price = new_price;
        }
        
        // Update average price (simple moving average of last price and new price)
        self.market_data.average_price = (self.market_data.last_sale_price + new_price) / 2;
        self.market_data.last_sale_price = new_price;
        
        Ok(())
    }
    
    /// Verify NFT
    pub fn verify(&mut self, verifier: Pubkey, verification_level: u8) -> Result<()> {
        self.verification.is_verified = true;
        self.verification.verified_at = Some(Clock::get()?.unix_timestamp);
        self.verification.verifier = Some(verifier);
        self.verification.verification_level = verification_level;
        self.updated_at = Clock::get()?.unix_timestamp;
        
        Ok(())
    }
    
    /// Get effective power score considering rarity and traits
    pub fn get_effective_power_score(&self) -> u32 {
        let base_score = self.attributes.power_score;
        let rarity_multiplier = self.rarity.get_multiplier();
        let synergy_bonus = self.attributes.synergy_bonus as f64 / 100.0;
        
        ((base_score as f64) * rarity_multiplier * (1.0 + synergy_bonus)) as u32
    }
    
    /// Check if NFT can be upgraded
    pub fn can_upgrade(&self) -> bool {
        self.attributes.evolution_stage < self.attributes.max_evolution
    }
    
    /// Get NFT summary as string
    pub fn get_name_as_string(&self) -> String {
        String::from_utf8_lossy(&self.name)
            .trim_end_matches('\0')
            .to_string()
    }
    
    pub fn get_description_as_string(&self) -> String {
        String::from_utf8_lossy(&self.description)
            .trim_end_matches('\0')
            .to_string()
    }
    
    /// Helper function to copy string to fixed-size array
    fn copy_string_to_array<const N: usize>(&self, source: &str, target: &mut [u8; N]) -> Result<()> {
        let bytes = source.as_bytes();
        require!(bytes.len() <= N, crate::errors::FinovaNftError::StringTooLong);
        
        target.fill(0);
        target[..bytes.len()].copy_from_slice(bytes);
        Ok(())
    }
}

// Default implementations
impl Default for NftAttributes {
    fn default() -> Self {
        Self {
            traits: [Trait::default(); 10],
            trait_count: 0,
            power_score: 0,
            synergy_bonus: 0,
            evolution_stage: 0,
            max_evolution: 0,
            upgrade_requirements: 0,
        }
    }
}

impl Default for Trait {
    fn default() -> Self {
        Self {
            trait_type: [0; 32],
            value: [0; 64],
            numeric_value: None,
            trait_rarity: 0,
        }
    }
}

impl Default for BonusEffects {
    fn default() -> Self {
        Self {
            mining_bonus: 0,
            xp_bonus: 0,
            referral_bonus: 0,
            privileges: 0,
            exclusive_access: 0,
        }
    }
}

impl Default for CollectorBenefits {
    fn default() -> Self {
        Self {
            early_access: false,
            marketplace_discount: 0,
            governance_weight: 0,
            exclusive_events: false,
            royalty_share: 0,
        }
    }
}

impl Default for UsageStats {
    fn default() -> Self {
        Self {
            total_uses: 0,
            remaining_uses: 0,
            last_used: 0,
            value_generated: 0,
            efficiency_rating: 0,
            usage_pattern: UsagePattern::default(),
        }
    }
}

impl Default for UsagePattern {
    fn default() -> Self {
        Self {
            avg_interval: 0,
            peak_hours: 0,
            trend: 0,
            recommendations: 0,
        }
    }
}

impl Default for MarketData {
    fn default() -> Self {
        Self {
            last_sale_price: 0,
            highest_sale_price: 0,
            average_price: 0,
            trade_count: 0,
            current_listing: None,
            sentiment_score: 0,
            liquidity_rating: 0,
            volatility_index: 0,
        }
    }
}

impl Default for VerificationStatus {
    fn default() -> Self {
        Self {
            is_verified: false,
            verified_at: None,
            verifier: None,
            verification_level: 0,
            authenticity_hash: [0; 32],
            content_verified: false,
            metadata_verified: false,
            creator_verified: false,
        }
    }
}
