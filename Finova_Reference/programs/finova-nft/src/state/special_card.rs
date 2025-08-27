// programs/finova-nft/src/state/special_card.rs

use anchor_lang::prelude::*;
use crate::constants::*;
use crate::errors::FinovaNftError;

/// Special Card Categories as defined in whitepaper
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum SpecialCardCategory {
    MiningBoost,
    XpAccelerator,
    ReferralPower,
    UtilityCard,
}

/// Card Rarity levels affecting pricing and effects
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum CardRarity {
    Common,
    Uncommon,
    Rare,
    Epic,
    Legendary,
    Mythic,
}

/// Card Usage Type - single use or multi-use
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum CardUsageType {
    SingleUse,
    MultiUse { max_uses: u32 },
    Permanent,
    TimeBased { duration_seconds: u64 },
}

/// Card Effect Configuration
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct CardEffect {
    /// Mining rate multiplier (e.g., 2.0 for 100% boost, 3.0 for 200% boost)
    pub mining_multiplier: f64,
    /// XP gain multiplier
    pub xp_multiplier: f64,
    /// Referral bonus multiplier
    pub referral_multiplier: f64,
    /// Duration in seconds (0 for permanent effects)
    pub duration_seconds: u64,
    /// Stacking capability with other cards
    pub can_stack: bool,
    /// Maximum stack count if stackable
    pub max_stack: u8,
}

/// Special Card State Account
#[account]
pub struct SpecialCard {
    /// Unique identifier for this card type
    pub card_id: u64,
    /// Card name (e.g., "Double Mining", "XP Magnet")
    pub name: String,
    /// Card description
    pub description: String,
    /// Category classification
    pub category: SpecialCardCategory,
    /// Rarity level
    pub rarity: CardRarity,
    /// Usage type and limitations
    pub usage_type: CardUsageType,
    /// Effect configuration
    pub effect: CardEffect,
    /// Base price in $FIN tokens
    pub base_price: u64,
    /// Current supply of this card type
    pub current_supply: u64,
    /// Maximum supply (0 for unlimited)
    pub max_supply: u64,
    /// Whether this card is currently mintable
    pub is_active: bool,
    /// Creator/authority of this card type
    pub authority: Pubkey,
    /// Metadata URI for off-chain data
    pub metadata_uri: String,
    /// Creation timestamp
    pub created_at: i64,
    /// Last update timestamp
    pub updated_at: i64,
    /// Collection this card belongs to
    pub collection: Pubkey,
    /// Bump seed for PDA
    pub bump: u8,
}

impl SpecialCard {
    pub const LEN: usize = 8 + // discriminator
        8 + // card_id
        4 + 64 + // name (max 64 chars)
        4 + 256 + // description (max 256 chars)
        1 + // category
        1 + // rarity
        5 + // usage_type (enum + max data)
        56 + // effect (7 fields * 8 bytes)
        8 + // base_price
        8 + // current_supply
        8 + // max_supply
        1 + // is_active
        32 + // authority
        4 + 200 + // metadata_uri (max 200 chars)
        8 + // created_at
        8 + // updated_at
        32 + // collection
        1 + // bump
        100; // padding for future upgrades

    /// Initialize a new special card type
    pub fn initialize(
        &mut self,
        card_id: u64,
        name: String,
        description: String,
        category: SpecialCardCategory,
        rarity: CardRarity,
        usage_type: CardUsageType,
        effect: CardEffect,
        base_price: u64,
        max_supply: u64,
        authority: Pubkey,
        metadata_uri: String,
        collection: Pubkey,
        bump: u8,
    ) -> Result<()> {
        require!(name.len() <= 64, FinovaNftError::NameTooLong);
        require!(description.len() <= 256, FinovaNftError::DescriptionTooLong);
        require!(metadata_uri.len() <= 200, FinovaNftError::MetadataUriTooLong);
        require!(base_price > 0, FinovaNftError::InvalidPrice);
        require!(self.validate_effect(&effect)?, FinovaNftError::InvalidEffect);

        let now = Clock::get()?.unix_timestamp;

        self.card_id = card_id;
        self.name = name;
        self.description = description;
        self.category = category;
        self.rarity = rarity;
        self.usage_type = usage_type;
        self.effect = effect;
        self.base_price = base_price;
        self.current_supply = 0;
        self.max_supply = max_supply;
        self.is_active = true;
        self.authority = authority;
        self.metadata_uri = metadata_uri;
        self.created_at = now;
        self.updated_at = now;
        self.collection = collection;
        self.bump = bump;

        Ok(())
    }

    /// Calculate current price based on supply and rarity
    pub fn get_current_price(&self) -> Result<u64> {
        let mut price = self.base_price;

        // Rarity multiplier
        let rarity_multiplier = match self.rarity {
            CardRarity::Common => 1.0,
            CardRarity::Uncommon => 1.5,
            CardRarity::Rare => 2.5,
            CardRarity::Epic => 4.0,
            CardRarity::Legendary => 8.0,
            CardRarity::Mythic => 15.0,
        };

        price = ((price as f64) * rarity_multiplier) as u64;

        // Supply scarcity multiplier
        if self.max_supply > 0 {
            let supply_ratio = (self.current_supply as f64) / (self.max_supply as f64);
            let scarcity_multiplier = 1.0 + (supply_ratio * 2.0); // Up to 3x price increase
            price = ((price as f64) * scarcity_multiplier) as u64;
        }

        Ok(price)
    }

    /// Check if card can be minted
    pub fn can_mint(&self) -> bool {
        self.is_active && 
        (self.max_supply == 0 || self.current_supply < self.max_supply)
    }

    /// Increment supply counter
    pub fn increment_supply(&mut self) -> Result<()> {
        require!(self.can_mint(), FinovaNftError::MintingNotAllowed);
        self.current_supply = self.current_supply.checked_add(1)
            .ok_or(FinovaNftError::SupplyOverflow)?;
        self.updated_at = Clock::get()?.unix_timestamp;
        Ok(())
    }

    /// Update card configuration (authority only)
    pub fn update_config(
        &mut self,
        name: Option<String>,
        description: Option<String>,
        base_price: Option<u64>,
        max_supply: Option<u64>,
        is_active: Option<bool>,
        metadata_uri: Option<String>,
    ) -> Result<()> {
        if let Some(new_name) = name {
            require!(new_name.len() <= 64, FinovaNftError::NameTooLong);
            self.name = new_name;
        }

        if let Some(new_description) = description {
            require!(new_description.len() <= 256, FinovaNftError::DescriptionTooLong);
            self.description = new_description;
        }

        if let Some(new_price) = base_price {
            require!(new_price > 0, FinovaNftError::InvalidPrice);
            self.base_price = new_price;
        }

        if let Some(new_max_supply) = max_supply {
            require!(
                new_max_supply == 0 || new_max_supply >= self.current_supply,
                FinovaNftError::InvalidSupplyUpdate
            );
            self.max_supply = new_max_supply;
        }

        if let Some(new_active) = is_active {
            self.is_active = new_active;
        }

        if let Some(new_uri) = metadata_uri {
            require!(new_uri.len() <= 200, FinovaNftError::MetadataUriTooLong);
            self.metadata_uri = new_uri;
        }

        self.updated_at = Clock::get()?.unix_timestamp;
        Ok(())
    }

    /// Validate card effects are within acceptable ranges
    fn validate_effect(&self, effect: &CardEffect) -> Result<bool> {
        // Mining multiplier should be between 1.0 and 10.0
        require!(
            effect.mining_multiplier >= 1.0 && effect.mining_multiplier <= 10.0,
            FinovaNftError::InvalidEffect
        );

        // XP multiplier should be between 1.0 and 5.0
        require!(
            effect.xp_multiplier >= 1.0 && effect.xp_multiplier <= 5.0,
            FinovaNftError::InvalidEffect
        );

        // Referral multiplier should be between 1.0 and 3.0
        require!(
            effect.referral_multiplier >= 1.0 && effect.referral_multiplier <= 3.0,
            FinovaNftError::InvalidEffect
        );

        // Duration should be reasonable (max 30 days)
        require!(
            effect.duration_seconds <= 30 * 24 * 3600,
            FinovaNftError::InvalidEffect
        );

        // Max stack should be reasonable
        require!(effect.max_stack <= 10, FinovaNftError::InvalidEffect);

        Ok(true)
    }

    /// Get synergy bonus based on active cards combination
    pub fn calculate_synergy_bonus(active_cards: &[&SpecialCard]) -> f64 {
        if active_cards.is_empty() {
            return 1.0;
        }

        let mut synergy_multiplier = 1.0;
        let card_count = active_cards.len() as f64;

        // Base synergy for multiple cards
        synergy_multiplier += card_count * 0.1;

        // Rarity bonus
        let avg_rarity_bonus = active_cards.iter()
            .map(|card| match card.rarity {
                CardRarity::Common => 0.0,
                CardRarity::Uncommon => 0.05,
                CardRarity::Rare => 0.10,
                CardRarity::Epic => 0.20,
                CardRarity::Legendary => 0.35,
                CardRarity::Mythic => 0.50,
            })
            .sum::<f64>() / card_count;

        synergy_multiplier += avg_rarity_bonus;

        // Category match bonus
        let unique_categories: std::collections::HashSet<_> = active_cards.iter()
            .map(|card| &card.category)
            .collect();

        if unique_categories.len() >= 2 {
            synergy_multiplier += 0.15; // Same category bonus
        }

        if unique_categories.len() == 3 {
            synergy_multiplier += 0.30; // All categories bonus
        }

        // Cap maximum synergy
        synergy_multiplier.min(3.0)
    }

    /// Check if card effects can stack with another card
    pub fn can_stack_with(&self, other: &SpecialCard) -> bool {
        // Cards of same type generally don't stack unless explicitly allowed
        if self.card_id == other.card_id {
            return self.effect.can_stack;
        }

        // Different category cards usually stack
        if self.category != other.category {
            return true;
        }

        // Same category cards may have restrictions
        match self.category {
            SpecialCardCategory::MiningBoost => {
                // Mining boost cards can stack up to 3
                self.effect.can_stack && other.effect.can_stack
            },
            SpecialCardCategory::XpAccelerator => {
                // XP cards generally stack
                true
            },
            SpecialCardCategory::ReferralPower => {
                // Referral cards have limited stacking
                self.effect.can_stack && other.effect.can_stack
            },
            SpecialCardCategory::UtilityCard => {
                // Utility cards are case-by-case
                self.effect.can_stack && other.effect.can_stack
            },
        }
    }
}

/// Individual Card Instance (owned by users)
#[account]
pub struct CardInstance {
    /// Reference to the card type
    pub card_type: Pubkey,
    /// Owner of this card instance
    pub owner: Pubkey,
    /// Unique instance ID
    pub instance_id: u64,
    /// Current usage count (for multi-use cards)
    pub usage_count: u32,
    /// Activation timestamp (when card effect starts)
    pub activated_at: Option<i64>,
    /// Expiration timestamp (for time-based cards)
    pub expires_at: Option<i64>,
    /// Whether the card is currently active
    pub is_active: bool,
    /// Mint address of the NFT representing this card
    pub mint: Pubkey,
    /// Creation timestamp
    pub created_at: i64,
    /// Bump seed for PDA
    pub bump: u8,
}

impl CardInstance {
    pub const LEN: usize = 8 + // discriminator
        32 + // card_type
        32 + // owner
        8 + // instance_id
        4 + // usage_count
        9 + // activated_at (Option<i64>)
        9 + // expires_at (Option<i64>)
        1 + // is_active
        32 + // mint
        8 + // created_at
        1 + // bump
        50; // padding

    /// Initialize a new card instance
    pub fn initialize(
        &mut self,
        card_type: Pubkey,
        owner: Pubkey,
        instance_id: u64,
        mint: Pubkey,
        bump: u8,
    ) -> Result<()> {
        let now = Clock::get()?.unix_timestamp;

        self.card_type = card_type;
        self.owner = owner;
        self.instance_id = instance_id;
        self.usage_count = 0;
        self.activated_at = None;
        self.expires_at = None;
        self.is_active = false;
        self.mint = mint;
        self.created_at = now;
        self.bump = bump;

        Ok(())
    }

    /// Activate the card effect
    pub fn activate(&mut self, card_config: &SpecialCard) -> Result<()> {
        require!(!self.is_active, FinovaNftError::CardAlreadyActive);
        
        let now = Clock::get()?.unix_timestamp;
        
        // Check usage limits
        match &card_config.usage_type {
            CardUsageType::SingleUse => {
                require!(self.usage_count == 0, FinovaNftError::CardExhausted);
            },
            CardUsageType::MultiUse { max_uses } => {
                require!(self.usage_count < *max_uses, FinovaNftError::CardExhausted);
            },
            CardUsageType::Permanent => {
                // No usage restrictions
            },
            CardUsageType::TimeBased { duration_seconds } => {
                // Set expiration time
                self.expires_at = Some(now + (*duration_seconds as i64));
            },
        }

        self.is_active = true;
        self.activated_at = Some(now);
        self.usage_count = self.usage_count.checked_add(1)
            .ok_or(FinovaNftError::UsageCountOverflow)?;

        Ok(())
    }

    /// Deactivate the card (natural expiration or manual)
    pub fn deactivate(&mut self) -> Result<()> {
        self.is_active = false;
        Ok(())
    }

    /// Check if card is currently expired
    pub fn is_expired(&self) -> Result<bool> {
        if let Some(expires_at) = self.expires_at {
            let now = Clock::get()?.unix_timestamp;
            Ok(now >= expires_at)
        } else {
            Ok(false)
        }
    }

    /// Transfer ownership of the card
    pub fn transfer(&mut self, new_owner: Pubkey) -> Result<()> {
        require!(!self.is_active, FinovaNftError::CannotTransferActiveCard);
        self.owner = new_owner;
        Ok(())
    }

    /// Check if card can be used
    pub fn can_use(&self, card_config: &SpecialCard) -> Result<bool> {
        // Check if already active
        if self.is_active {
            return Ok(false);
        }

        // Check expiration
        if self.is_expired()? {
            return Ok(false);
        }

        // Check usage limits
        match &card_config.usage_type {
            CardUsageType::SingleUse => {
                Ok(self.usage_count == 0)
            },
            CardUsageType::MultiUse { max_uses } => {
                Ok(self.usage_count < *max_uses)
            },
            CardUsageType::Permanent => {
                Ok(true)
            },
            CardUsageType::TimeBased { .. } => {
                Ok(true)
            },
        }
    }
}

/// Card Collection State for organizing cards
#[account]
pub struct CardCollection {
    /// Collection identifier
    pub collection_id: u64,
    /// Collection name
    pub name: String,
    /// Collection description
    pub description: String,
    /// Authority managing this collection
    pub authority: Pubkey,
    /// Total cards in this collection
    pub total_cards: u64,
    /// Whether new cards can be added
    pub is_mutable: bool,
    /// Collection metadata URI
    pub metadata_uri: String,
    /// Creation timestamp
    pub created_at: i64,
    /// Bump seed for PDA
    pub bump: u8,
}

impl CardCollection {
    pub const LEN: usize = 8 + // discriminator
        8 + // collection_id
        4 + 64 + // name
        4 + 256 + // description
        32 + // authority
        8 + // total_cards
        1 + // is_mutable
        4 + 200 + // metadata_uri
        8 + // created_at
        1 + // bump
        50; // padding

    /// Initialize a new card collection
    pub fn initialize(
        &mut self,
        collection_id: u64,
        name: String,
        description: String,
        authority: Pubkey,
        metadata_uri: String,
        bump: u8,
    ) -> Result<()> {
        require!(name.len() <= 64, FinovaNftError::NameTooLong);
        require!(description.len() <= 256, FinovaNftError::DescriptionTooLong);
        require!(metadata_uri.len() <= 200, FinovaNftError::MetadataUriTooLong);

        let now = Clock::get()?.unix_timestamp;

        self.collection_id = collection_id;
        self.name = name;
        self.description = description;
        self.authority = authority;
        self.total_cards = 0;
        self.is_mutable = true;
        self.metadata_uri = metadata_uri;
        self.created_at = now;
        self.bump = bump;

        Ok(())
    }

    /// Add a card to this collection
    pub fn add_card(&mut self) -> Result<()> {
        require!(self.is_mutable, FinovaNftError::CollectionImmutable);
        self.total_cards = self.total_cards.checked_add(1)
            .ok_or(FinovaNftError::CollectionOverflow)?;
        Ok(())
    }

    /// Freeze the collection (make immutable)
    pub fn freeze(&mut self) -> Result<()> {
        self.is_mutable = false;
        Ok(())
    }
}
