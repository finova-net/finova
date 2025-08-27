// programs/finova-nft/src/utils.rs

use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};
use solana_program::{
    hash::{hash, Hash},
    program_pack::Pack,
    pubkey::Pubkey,
    system_instruction,
    sysvar::{clock::Clock, rent::Rent},
};
use std::collections::HashMap;

use crate::constants::*;
use crate::errors::*;

/// Utility functions for Finova NFT program operations
pub struct NftUtils;

impl NftUtils {
    /// Validates NFT metadata URI format and accessibility
    pub fn validate_metadata_uri(uri: &str) -> Result<()> {
        // Check URI length
        if uri.len() > MAX_URI_LENGTH {
            return Err(FinovaNftError::UriTooLong.into());
        }

        // Check if URI is empty
        if uri.trim().is_empty() {
            return Err(FinovaNftError::InvalidUri.into());
        }

        // Validate URI format (basic checks)
        if !uri.starts_with("https://") && !uri.starts_with("ipfs://") && !uri.starts_with("ar://") {
            return Err(FinovaNftError::InvalidUriFormat.into());
        }

        // Check for common invalid characters
        let invalid_chars = ['<', '>', '"', '|', '\\', '^', '`', '{', '}'];
        for char in invalid_chars.iter() {
            if uri.contains(*char) {
                return Err(FinovaNftError::InvalidUriCharacters.into());
            }
        }

        Ok(())
    }

    /// Validates NFT name format and content
    pub fn validate_nft_name(name: &str) -> Result<()> {
        if name.trim().is_empty() {
            return Err(FinovaNftError::InvalidName.into());
        }

        if name.len() > MAX_NAME_LENGTH {
            return Err(FinovaNftError::NameTooLong.into());
        }

        // Check for invalid characters in name
        if name.chars().any(|c| c.is_control() || c == '\0') {
            return Err(FinovaNftError::InvalidNameCharacters.into());
        }

        Ok(())
    }

    /// Validates NFT symbol format
    pub fn validate_symbol(symbol: &str) -> Result<()> {
        if symbol.trim().is_empty() {
            return Err(FinovaNftError::InvalidSymbol.into());
        }

        if symbol.len() > MAX_SYMBOL_LENGTH {
            return Err(FinovaNftError::SymbolTooLong.into());
        }

        // Symbol should be alphanumeric and uppercase
        if !symbol.chars().all(|c| c.is_ascii_alphanumeric() && c.is_ascii_uppercase()) {
            return Err(FinovaNftError::InvalidSymbolFormat.into());
        }

        Ok(())
    }

    /// Calculates special card effectiveness based on rarity and type
    pub fn calculate_card_effectiveness(
        card_type: u8,
        rarity: u8,
        user_level: u64,
        current_time: i64,
    ) -> Result<f64> {
        let base_effectiveness = match rarity {
            RARITY_COMMON => 1.0,
            RARITY_UNCOMMON => 1.2,
            RARITY_RARE => 1.5,
            RARITY_EPIC => 2.0,
            RARITY_LEGENDARY => 3.0,
            RARITY_MYTHIC => 5.0,
            _ => return Err(FinovaNftError::InvalidRarity.into()),
        };

        let type_multiplier = match card_type {
            CARD_TYPE_MINING_BOOST => 1.0,
            CARD_TYPE_XP_BOOST => 1.1,
            CARD_TYPE_REFERRAL_BOOST => 0.9,
            CARD_TYPE_QUALITY_BOOST => 1.2,
            CARD_TYPE_NETWORK_BOOST => 0.8,
            CARD_TYPE_SPECIAL_EVENT => 1.5,
            _ => return Err(FinovaNftError::InvalidCardType.into()),
        };

        // Level-based scaling (higher level users get slightly better effectiveness)
        let level_multiplier = 1.0 + (user_level as f64 * 0.001).min(0.5);

        // Time-based decay for balance (optional for certain card types)
        let time_decay = if card_type == CARD_TYPE_SPECIAL_EVENT {
            1.0 - ((current_time as f64 - 1640995200.0) / 31536000.0 * 0.01).max(0.0).min(0.3)
        } else {
            1.0
        };

        let final_effectiveness = base_effectiveness * type_multiplier * level_multiplier * time_decay;
        
        Ok(final_effectiveness.max(0.1).min(10.0)) // Cap between 0.1x and 10x
    }

    /// Generates deterministic NFT attributes based on seed
    pub fn generate_nft_attributes(seed: &[u8], rarity: u8) -> Result<HashMap<String, String>> {
        let mut attributes = HashMap::new();
        let hash_result = hash(seed);
        let hash_bytes = hash_result.to_bytes();

        // Base attributes for all NFTs
        attributes.insert("Rarity".to_string(), Self::rarity_to_string(rarity)?);
        attributes.insert("Generation".to_string(), "Genesis".to_string());

        // Rarity-specific attributes
        match rarity {
            RARITY_COMMON => {
                attributes.insert("Power".to_string(), ((hash_bytes[0] % 10) + 1).to_string());
                attributes.insert("Element".to_string(), Self::get_element(hash_bytes[1] % 4));
            },
            RARITY_UNCOMMON => {
                attributes.insert("Power".to_string(), ((hash_bytes[0] % 15) + 5).to_string());
                attributes.insert("Element".to_string(), Self::get_element(hash_bytes[1] % 4));
                attributes.insert("Bonus".to_string(), Self::get_bonus_trait(hash_bytes[2] % 3));
            },
            RARITY_RARE => {
                attributes.insert("Power".to_string(), ((hash_bytes[0] % 20) + 10).to_string());
                attributes.insert("Element".to_string(), Self::get_element(hash_bytes[1] % 6));
                attributes.insert("Bonus".to_string(), Self::get_bonus_trait(hash_bytes[2] % 5));
                attributes.insert("Special".to_string(), Self::get_special_trait(hash_bytes[3] % 3));
            },
            RARITY_EPIC => {
                attributes.insert("Power".to_string(), ((hash_bytes[0] % 30) + 20).to_string());
                attributes.insert("Element".to_string(), Self::get_element(hash_bytes[1] % 8));
                attributes.insert("Bonus".to_string(), Self::get_bonus_trait(hash_bytes[2] % 7));
                attributes.insert("Special".to_string(), Self::get_special_trait(hash_bytes[3] % 5));
                attributes.insert("Legendary".to_string(), Self::get_legendary_trait(hash_bytes[4] % 2));
            },
            RARITY_LEGENDARY => {
                attributes.insert("Power".to_string(), ((hash_bytes[0] % 50) + 40).to_string());
                attributes.insert("Element".to_string(), Self::get_element(hash_bytes[1] % 10));
                attributes.insert("Bonus".to_string(), Self::get_bonus_trait(hash_bytes[2] % 10));
                attributes.insert("Special".to_string(), Self::get_special_trait(hash_bytes[3] % 8));
                attributes.insert("Legendary".to_string(), Self::get_legendary_trait(hash_bytes[4] % 5));
                attributes.insert("Mythical".to_string(), "True".to_string());
            },
            RARITY_MYTHIC => {
                attributes.insert("Power".to_string(), ((hash_bytes[0] % 100) + 75).to_string());
                attributes.insert("Element".to_string(), "Cosmic".to_string());
                attributes.insert("Bonus".to_string(), "Ultimate".to_string());
                attributes.insert("Special".to_string(), "Transcendent".to_string());
                attributes.insert("Legendary".to_string(), "Divine".to_string());
                attributes.insert("Mythical".to_string(), "Absolute".to_string());
                attributes.insert("Unique".to_string(), "One of One".to_string());
            },
            _ => return Err(FinovaNftError::InvalidRarity.into()),
        }

        Ok(attributes)
    }

    /// Validates special card usage conditions
    pub fn validate_card_usage(
        card_type: u8,
        user_level: u64,
        user_mining_rate: f64,
        last_usage: i64,
        current_time: i64,
    ) -> Result<()> {
        // Check cooldown period
        let cooldown_period = match card_type {
            CARD_TYPE_MINING_BOOST => MINING_BOOST_COOLDOWN,
            CARD_TYPE_XP_BOOST => XP_BOOST_COOLDOWN,
            CARD_TYPE_REFERRAL_BOOST => REFERRAL_BOOST_COOLDOWN,
            CARD_TYPE_QUALITY_BOOST => QUALITY_BOOST_COOLDOWN,
            CARD_TYPE_NETWORK_BOOST => NETWORK_BOOST_COOLDOWN,
            CARD_TYPE_SPECIAL_EVENT => 0, // No cooldown for special events
            _ => return Err(FinovaNftError::InvalidCardType.into()),
        };

        if current_time - last_usage < cooldown_period {
            return Err(FinovaNftError::CardOnCooldown.into());
        }

        // Level requirements for certain cards
        let min_level = match card_type {
            CARD_TYPE_MINING_BOOST => 1,
            CARD_TYPE_XP_BOOST => 5,
            CARD_TYPE_REFERRAL_BOOST => 10,
            CARD_TYPE_QUALITY_BOOST => 15,
            CARD_TYPE_NETWORK_BOOST => 25,
            CARD_TYPE_SPECIAL_EVENT => 1,
            _ => 1,
        };

        if user_level < min_level {
            return Err(FinovaNftError::InsufficientLevel.into());
        }

        // Mining rate requirements for boost cards
        if matches!(card_type, CARD_TYPE_MINING_BOOST | CARD_TYPE_NETWORK_BOOST) {
            if user_mining_rate < MIN_MINING_RATE_FOR_BOOST {
                return Err(FinovaNftError::InsufficientMiningRate.into());
            }
        }

        Ok(())
    }

    /// Calculates marketplace fees based on NFT value and seller tier
    pub fn calculate_marketplace_fees(
        sale_price: u64,
        seller_tier: u8,
        is_creator: bool,
    ) -> Result<(u64, u64, u64)> {
        // Base marketplace fee (2.5%)
        let mut marketplace_fee_rate = 250; // 2.5% in basis points

        // Tier-based fee reduction
        let tier_discount = match seller_tier {
            0..=10 => 0,      // No discount
            11..=25 => 25,    // 0.25% discount
            26..=50 => 50,    // 0.5% discount
            51..=75 => 75,    // 0.75% discount
            76..=100 => 100,  // 1% discount
            _ => 100,         // Max 1% discount
        };

        marketplace_fee_rate = marketplace_fee_rate.saturating_sub(tier_discount);

        // Creator royalty (5% for original creator)
        let creator_royalty_rate = if is_creator { 0 } else { 500 }; // 5% in basis points

        // Calculate fees
        let marketplace_fee = (sale_price as u128 * marketplace_fee_rate as u128 / 10000) as u64;
        let creator_royalty = (sale_price as u128 * creator_royalty_rate as u128 / 10000) as u64;
        let seller_amount = sale_price
            .saturating_sub(marketplace_fee)
            .saturating_sub(creator_royalty);

        Ok((marketplace_fee, creator_royalty, seller_amount))
    }

    /// Validates collection creation parameters
    pub fn validate_collection_params(
        name: &str,
        symbol: &str,
        description: &str,
        max_supply: u64,
    ) -> Result<()> {
        Self::validate_nft_name(name)?;
        Self::validate_symbol(symbol)?;

        if description.len() > MAX_DESCRIPTION_LENGTH {
            return Err(FinovaNftError::DescriptionTooLong.into());
        }

        if max_supply == 0 || max_supply > MAX_COLLECTION_SIZE {
            return Err(FinovaNftError::InvalidMaxSupply.into());
        }

        Ok(())
    }

    /// Generates collection-specific metadata
    pub fn generate_collection_metadata(
        name: &str,
        symbol: &str,
        description: &str,
        creator: Pubkey,
        max_supply: u64,
    ) -> HashMap<String, String> {
        let mut metadata = HashMap::new();
        
        metadata.insert("name".to_string(), name.to_string());
        metadata.insert("symbol".to_string(), symbol.to_string());
        metadata.insert("description".to_string(), description.to_string());
        metadata.insert("creator".to_string(), creator.to_string());
        metadata.insert("max_supply".to_string(), max_supply.to_string());
        metadata.insert("network".to_string(), "Solana".to_string());
        metadata.insert("standard".to_string(), "Metaplex".to_string());
        metadata.insert("version".to_string(), "1.0".to_string());
        
        metadata
    }

    // Helper functions for attribute generation
    fn rarity_to_string(rarity: u8) -> Result<String> {
        match rarity {
            RARITY_COMMON => Ok("Common".to_string()),
            RARITY_UNCOMMON => Ok("Uncommon".to_string()),
            RARITY_RARE => Ok("Rare".to_string()),
            RARITY_EPIC => Ok("Epic".to_string()),
            RARITY_LEGENDARY => Ok("Legendary".to_string()),
            RARITY_MYTHIC => Ok("Mythic".to_string()),
            _ => Err(FinovaNftError::InvalidRarity.into()),
        }
    }

    fn get_element(seed: u8) -> String {
        match seed % 10 {
            0 => "Fire".to_string(),
            1 => "Water".to_string(),
            2 => "Earth".to_string(),
            3 => "Air".to_string(),
            4 => "Lightning".to_string(),
            5 => "Ice".to_string(),
            6 => "Nature".to_string(),
            7 => "Shadow".to_string(),
            8 => "Light".to_string(),
            _ => "Neutral".to_string(),
        }
    }

    fn get_bonus_trait(seed: u8) -> String {
        match seed % 10 {
            0 => "Mining Efficiency".to_string(),
            1 => "XP Multiplier".to_string(),
            2 => "Referral Bonus".to_string(),
            3 => "Quality Boost".to_string(),
            4 => "Network Effect".to_string(),
            5 => "Durability".to_string(),
            6 => "Speed".to_string(),
            7 => "Luck".to_string(),
            8 => "Stability".to_string(),
            _ => "Balance".to_string(),
        }
    }

    fn get_special_trait(seed: u8) -> String {
        match seed % 8 {
            0 => "Shimmering".to_string(),
            1 => "Glowing".to_string(),
            2 => "Pulsing".to_string(),
            3 => "Sparkling".to_string(),
            4 => "Radiating".to_string(),
            5 => "Crystalline".to_string(),
            6 => "Ethereal".to_string(),
            _ => "Mystical".to_string(),
        }
    }

    fn get_legendary_trait(seed: u8) -> String {
        match seed % 5 {
            0 => "Transcendent".to_string(),
            1 => "Omnipotent".to_string(),
            2 => "Infinite".to_string(),
            3 => "Eternal".to_string(),
            _ => "Divine".to_string(),
        }
    }

    /// Validates NFT transfer conditions
    pub fn validate_transfer_conditions(
        nft_type: u8,
        is_locked: bool,
        transfer_count: u32,
        current_time: i64,
        last_transfer: i64,
    ) -> Result<()> {
        // Check if NFT is locked
        if is_locked {
            return Err(FinovaNftError::NftLocked.into());
        }

        // Transfer cooldown for certain NFT types
        let cooldown = match nft_type {
            CARD_TYPE_SPECIAL_EVENT => SPECIAL_TRANSFER_COOLDOWN,
            CARD_TYPE_MINING_BOOST if transfer_count > 3 => FREQUENT_TRANSFER_COOLDOWN,
            _ => 0,
        };

        if current_time - last_transfer < cooldown {
            return Err(FinovaNftError::TransferCooldown.into());
        }

        // Maximum transfer limit for special cards
        if nft_type == CARD_TYPE_SPECIAL_EVENT && transfer_count >= MAX_SPECIAL_TRANSFERS {
            return Err(FinovaNftError::MaxTransfersReached.into());
        }

        Ok(())
    }

    /// Calculates dynamic pricing for NFT minting based on supply and demand
    pub fn calculate_dynamic_mint_price(
        base_price: u64,
        current_supply: u64,
        max_supply: u64,
        demand_multiplier: f64,
        rarity: u8,
    ) -> Result<u64> {
        // Supply scarcity multiplier
        let supply_ratio = current_supply as f64 / max_supply as f64;
        let scarcity_multiplier = 1.0 + (supply_ratio * supply_ratio * 2.0);

        // Rarity multiplier
        let rarity_multiplier = match rarity {
            RARITY_COMMON => 1.0,
            RARITY_UNCOMMON => 2.0,
            RARITY_RARE => 5.0,
            RARITY_EPIC => 12.0,
            RARITY_LEGENDARY => 30.0,
            RARITY_MYTHIC => 100.0,
            _ => return Err(FinovaNftError::InvalidRarity.into()),
        };

        // Calculate final price
        let dynamic_price = (base_price as f64 
            * scarcity_multiplier 
            * demand_multiplier 
            * rarity_multiplier) as u64;

        // Apply reasonable bounds
        let min_price = base_price / 2;
        let max_price = base_price * 100;

        Ok(dynamic_price.max(min_price).min(max_price))
    }

    /// Validates card combination for fusion/evolution
    pub fn validate_card_fusion(
        card1_type: u8,
        card1_rarity: u8,
        card2_type: u8,
        card2_rarity: u8,
    ) -> Result<(u8, u8)> {
        // Cards must be of same type for fusion
        if card1_type != card2_type {
            return Err(FinovaNftError::IncompatibleCardTypes.into());
        }

        // Cards must be same rarity or adjacent rarities
        let rarity_diff = (card1_rarity as i8 - card2_rarity as i8).abs();
        if rarity_diff > 1 {
            return Err(FinovaNftError::IncompatibleRarities.into());
        }

        // Cannot fuse mythic cards
        if card1_rarity == RARITY_MYTHIC || card2_rarity == RARITY_MYTHIC {
            return Err(FinovaNftError::CannotFuseMythic.into());
        }

        // Determine result rarity (always upgrade to higher)
        let result_rarity = card1_rarity.max(card2_rarity) + 1;
        if result_rarity > RARITY_MYTHIC {
            return Err(FinovaNftError::MaxRarityReached.into());
        }

        Ok((card1_type, result_rarity))
    }

    /// Generates fusion success probability based on rarity and user level
    pub fn calculate_fusion_success_rate(
        base_rarity: u8,
        target_rarity: u8,
        user_level: u64,
        fusion_attempts: u32,
    ) -> f64 {
        // Base success rate decreases with higher target rarity
        let base_rate = match target_rarity {
            RARITY_UNCOMMON => 0.9,
            RARITY_RARE => 0.7,
            RARITY_EPIC => 0.5,
            RARITY_LEGENDARY => 0.3,
            RARITY_MYTHIC => 0.1,
            _ => 0.95,
        };

        // User level bonus (max 20% bonus)
        let level_bonus = (user_level as f64 * 0.002).min(0.2);

        // Pity system - increase success rate after failed attempts
        let pity_bonus = (fusion_attempts as f64 * 0.05).min(0.3);

        (base_rate + level_bonus + pity_bonus).min(0.95)
    }
}

/// Marketplace utilities for NFT trading
pub struct MarketplaceUtils;

impl MarketplaceUtils {
    /// Validates listing parameters
    pub fn validate_listing(
        price: u64,
        duration: i64,
        current_time: i64,
    ) -> Result<()> {
        if price < MIN_LISTING_PRICE {
            return Err(FinovaNftError::PriceTooLow.into());
        }

        if price > MAX_LISTING_PRICE {
            return Err(FinovaNftError::PriceTooHigh.into());
        }

        if duration < MIN_LISTING_DURATION || duration > MAX_LISTING_DURATION {
            return Err(FinovaNftError::InvalidListingDuration.into());
        }

        Ok(())
    }

    /// Calculates auction end time and minimum bid increment
    pub fn calculate_auction_params(
        starting_price: u64,
        duration: i64,
        current_time: i64,
    ) -> (i64, u64) {
        let end_time = current_time + duration;
        let min_bid_increment = (starting_price * MIN_BID_INCREMENT_PERCENT / 100).max(MIN_BID_INCREMENT);
        
        (end_time, min_bid_increment)
    }

    /// Validates bid parameters for auctions
    pub fn validate_bid(
        bid_amount: u64,
        current_highest_bid: u64,
        min_increment: u64,
        auction_end_time: i64,
        current_time: i64,
    ) -> Result<()> {
        if current_time >= auction_end_time {
            return Err(FinovaNftError::AuctionEnded.into());
        }

        let required_minimum = current_highest_bid + min_increment;
        if bid_amount < required_minimum {
            return Err(FinovaNftError::BidTooLow.into());
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_metadata_uri() {
        // Valid URIs
        assert!(NftUtils::validate_metadata_uri("https://example.com/metadata.json").is_ok());
        assert!(NftUtils::validate_metadata_uri("ipfs://QmHash123").is_ok());
        assert!(NftUtils::validate_metadata_uri("ar://tx123").is_ok());

        // Invalid URIs
        assert!(NftUtils::validate_metadata_uri("").is_err());
        assert!(NftUtils::validate_metadata_uri("http://example.com").is_err());
        assert!(NftUtils::validate_metadata_uri("https://example.com/test<>.json").is_err());
    }

    #[test]
    fn test_calculate_card_effectiveness() {
        let result = NftUtils::calculate_card_effectiveness(
            CARD_TYPE_MINING_BOOST,
            RARITY_EPIC,
            50,
            1640995200,
        );
        assert!(result.is_ok());
        assert!(result.unwrap() > 1.0);
    }

    #[test]
    fn test_calculate_marketplace_fees() {
        let (marketplace_fee, creator_royalty, seller_amount) = 
            NftUtils::calculate_marketplace_fees(1000, 10, false).unwrap();
        
        assert_eq!(marketplace_fee, 25); // 2.5%
        assert_eq!(creator_royalty, 50);  // 5%
        assert_eq!(seller_amount, 925);   // Remaining
    }

    #[test]
    fn test_validate_card_fusion() {
        // Valid fusion
        let result = NftUtils::validate_card_fusion(
            CARD_TYPE_MINING_BOOST,
            RARITY_COMMON,
            CARD_TYPE_MINING_BOOST,
            RARITY_COMMON,
        );
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), (CARD_TYPE_MINING_BOOST, RARITY_UNCOMMON));

        // Invalid fusion - different types
        let result = NftUtils::validate_card_fusion(
            CARD_TYPE_MINING_BOOST,
            RARITY_COMMON,
            CARD_TYPE_XP_BOOST,
            RARITY_COMMON,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_dynamic_mint_price() {
        let price = NftUtils::calculate_dynamic_mint_price(
            1000,  // base price
            100,   // current supply
            1000,  // max supply
            1.5,   // demand multiplier
            RARITY_RARE,
        ).unwrap();

        assert!(price > 1000); // Price should be higher due to rarity and demand
    }
}
