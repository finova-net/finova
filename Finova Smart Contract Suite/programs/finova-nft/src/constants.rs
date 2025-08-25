// programs/finova-nft/src/constants.rs

use anchor_lang::prelude::*;

/// Program constants for Finova NFT system
/// Implements special cards system inspired by Hamster Kombat mechanics

/// Program seeds for PDA derivation
pub const COLLECTION_SEED: &[u8] = b"collection";
pub const NFT_METADATA_SEED: &[u8] = b"nft_metadata";
pub const SPECIAL_CARD_SEED: &[u8] = b"special_card";
pub const MARKETPLACE_SEED: &[u8] = b"marketplace";
pub const USER_NFT_SEED: &[u8] = b"user_nft";
pub const CARD_REGISTRY_SEED: &[u8] = b"card_registry";
pub const COLLECTION_AUTHORITY_SEED: &[u8] = b"collection_authority";
pub const ESCROW_SEED: &[u8] = b"escrow";
pub const ROYALTY_SEED: &[u8] = b"royalty";

/// Special Card Type IDs
pub const MINING_BOOST_CARD: u8 = 1;
pub const XP_ACCELERATOR_CARD: u8 = 2;
pub const REFERRAL_POWER_CARD: u8 = 3;
pub const PROFILE_BADGE_CARD: u8 = 4;
pub const ACHIEVEMENT_CARD: u8 = 5;
pub const LIMITED_EDITION_CARD: u8 = 6;
pub const SEASONAL_CARD: u8 = 7;
pub const GUILD_CARD: u8 = 8;

/// Special Card Rarity Levels
pub const COMMON_RARITY: u8 = 1;
pub const UNCOMMON_RARITY: u8 = 2;
pub const RARE_RARITY: u8 = 3;
pub const EPIC_RARITY: u8 = 4;
pub const LEGENDARY_RARITY: u8 = 5;
pub const MYTHIC_RARITY: u8 = 6;

/// Mining Boost Cards - Effect percentages (basis points)
pub const DOUBLE_MINING_BOOST: u16 = 10000; // 100% boost
pub const TRIPLE_MINING_BOOST: u16 = 20000; // 200% boost
pub const MINING_FRENZY_BOOST: u16 = 50000; // 500% boost
pub const ETERNAL_MINER_BOOST: u16 = 5000;  // 50% boost

/// Mining Boost Cards - Duration in seconds
pub const DOUBLE_MINING_DURATION: i64 = 86400;   // 24 hours
pub const TRIPLE_MINING_DURATION: i64 = 43200;   // 12 hours
pub const MINING_FRENZY_DURATION: i64 = 14400;   // 4 hours
pub const ETERNAL_MINER_DURATION: i64 = 2592000; // 30 days

/// XP Accelerator Cards - Effect percentages (basis points)
pub const XP_DOUBLE_BOOST: u16 = 10000;    // 100% boost
pub const XP_STREAK_SAVER: u16 = 0;        // Special effect, no percentage
pub const XP_LEVEL_RUSH: u16 = 0;          // Instant XP gain
pub const XP_MAGNET_BOOST: u16 = 30000;    // 300% boost

/// XP Accelerator Cards - Duration in seconds
pub const XP_DOUBLE_DURATION: i64 = 86400;  // 24 hours
pub const XP_STREAK_DURATION: i64 = 604800; // 7 days
pub const XP_LEVEL_RUSH_XP: u32 = 500;      // Instant XP amount
pub const XP_MAGNET_DURATION: i64 = 172800; // 48 hours

/// Referral Power Cards - Effect percentages (basis points)
pub const REFERRAL_BOOST: u16 = 5000;      // 50% boost
pub const NETWORK_AMPLIFIER: u8 = 2;       // +2 levels to RP tier
pub const AMBASSADOR_PASS: u16 = 0;         // Special unlock
pub const NETWORK_KING_BOOST: u16 = 10000; // 100% boost

/// Referral Power Cards - Duration in seconds
pub const REFERRAL_BOOST_DURATION: i64 = 604800; // 7 days
pub const NETWORK_AMPLIFIER_DURATION: i64 = 86400; // 24 hours
pub const AMBASSADOR_PASS_DURATION: i64 = 172800; // 48 hours
pub const NETWORK_KING_DURATION: i64 = 43200; // 12 hours

/// Profile Badge NFT Levels
pub const BRONZE_BADGE: u8 = 1;
pub const SILVER_BADGE: u8 = 2;
pub const GOLD_BADGE: u8 = 3;
pub const PLATINUM_BADGE: u8 = 4;
pub const DIAMOND_BADGE: u8 = 5;
pub const MYTHIC_BADGE: u8 = 6;

/// Profile Badge Mining Multipliers (basis points)
pub const BRONZE_BADGE_MULTIPLIER: u16 = 2500;  // 25%
pub const SILVER_BADGE_MULTIPLIER: u16 = 5000;  // 50%
pub const GOLD_BADGE_MULTIPLIER: u16 = 7500;    // 75%
pub const PLATINUM_BADGE_MULTIPLIER: u16 = 10000; // 100%
pub const DIAMOND_BADGE_MULTIPLIER: u16 = 12500;  // 125%
pub const MYTHIC_BADGE_MULTIPLIER: u16 = 15000;   // 150%

/// Achievement NFT Types
pub const FIRST_1000_USERS: u8 = 1;
pub const VIRAL_CREATOR: u8 = 2;
pub const NETWORK_BUILDER: u8 = 3;
pub const WHALE_STAKER: u8 = 4;
pub const GUILD_MASTER: u8 = 5;
pub const LOYAL_MEMBER: u8 = 6;
pub const CONTENT_KING: u8 = 7;
pub const SOCIAL_BUTTERFLY: u8 = 8;

/// Achievement NFT Bonuses (basis points)
pub const FINIZEN_BONUS: u16 = 2500;        // 25% lifetime mining
pub const VIRAL_CREATOR_BONUS: u16 = 5000;  // 50% XP from posts
pub const NETWORK_BUILDER_BONUS: u16 = 3000; // 30% referral rewards
pub const WHALE_STAKER_BONUS: u16 = 2000;   // 20% staking rewards
pub const GUILD_MASTER_BONUS: u16 = 3500;   // 35% guild rewards
pub const LOYAL_MEMBER_BONUS: u16 = 1500;   // 15% all rewards
pub const CONTENT_KING_BONUS: u16 = 4000;   // 40% content rewards
pub const SOCIAL_BUTTERFLY_BONUS: u16 = 2500; // 25% social rewards

/// Marketplace Configuration
pub const MARKETPLACE_FEE_BASIS_POINTS: u16 = 250; // 2.5% marketplace fee
pub const ROYALTY_FEE_BASIS_POINTS: u16 = 500;     // 5% creator royalty
pub const MAX_ROYALTY_BASIS_POINTS: u16 = 1000;    // 10% max royalty
pub const MIN_LISTING_PRICE: u64 = 1_000_000;      // 0.001 SOL minimum
pub const MAX_LISTING_PRICE: u64 = 1_000_000_000_000; // 1000 SOL maximum

/// Card Synergy System
pub const SYNERGY_BASE_BONUS: u16 = 1000;      // 10% base synergy
pub const SAME_CATEGORY_BONUS: u16 = 1500;     // 15% same category
pub const ALL_CATEGORIES_BONUS: u16 = 3000;    // 30% all categories
pub const MAX_SYNERGY_CARDS: u8 = 5;           // Maximum active cards

/// Rarity Synergy Bonuses (basis points)
pub const COMMON_SYNERGY_BONUS: u16 = 0;       // 0%
pub const UNCOMMON_SYNERGY_BONUS: u16 = 500;   // 5%
pub const RARE_SYNERGY_BONUS: u16 = 1000;      // 10%
pub const EPIC_SYNERGY_BONUS: u16 = 2000;      // 20%
pub const LEGENDARY_SYNERGY_BONUS: u16 = 3500; // 35%
pub const MYTHIC_SYNERGY_BONUS: u16 = 5000;    // 50%

/// Collection Configuration
pub const MAX_COLLECTION_SIZE: u32 = 100_000;  // Maximum NFTs per collection
pub const COLLECTION_SYMBOL_MAX_LEN: usize = 10;
pub const COLLECTION_NAME_MAX_LEN: usize = 32;
pub const COLLECTION_URI_MAX_LEN: usize = 200;

/// NFT Metadata Configuration
pub const NFT_NAME_MAX_LEN: usize = 32;
pub const NFT_SYMBOL_MAX_LEN: usize = 10;
pub const NFT_URI_MAX_LEN: usize = 200;
pub const NFT_DESCRIPTION_MAX_LEN: usize = 500;
pub const ATTRIBUTE_KEY_MAX_LEN: usize = 32;
pub const ATTRIBUTE_VALUE_MAX_LEN: usize = 64;
pub const MAX_ATTRIBUTES: usize = 20;

/// Special Card Usage Limits
pub const MAX_ACTIVE_MINING_CARDS: u8 = 3;     // Max 3 mining cards
pub const MAX_ACTIVE_XP_CARDS: u8 = 2;         // Max 2 XP cards
pub const MAX_ACTIVE_REFERRAL_CARDS: u8 = 2;   // Max 2 referral cards
pub const MAX_DAILY_CARD_USAGE: u8 = 10;       // Max 10 cards per day
pub const CARD_COOLDOWN_SECONDS: i64 = 3600;   // 1 hour cooldown

/// Marketplace Timing
pub const MIN_AUCTION_DURATION: i64 = 3600;    // 1 hour minimum
pub const MAX_AUCTION_DURATION: i64 = 604800;  // 7 days maximum
pub const DEFAULT_AUCTION_DURATION: i64 = 86400; // 24 hours default
pub const AUCTION_EXTENSION_TIME: i64 = 300;   // 5 minutes extension
pub const BID_INCREMENT_BASIS_POINTS: u16 = 500; // 5% minimum bid increment

/// Card Pricing Tiers (in lamports)
pub const COMMON_CARD_BASE_PRICE: u64 = 10_000_000;      // 0.01 SOL
pub const UNCOMMON_CARD_BASE_PRICE: u64 = 25_000_000;    // 0.025 SOL
pub const RARE_CARD_BASE_PRICE: u64 = 50_000_000;        // 0.05 SOL
pub const EPIC_CARD_BASE_PRICE: u64 = 100_000_000;       // 0.1 SOL
pub const LEGENDARY_CARD_BASE_PRICE: u64 = 500_000_000;  // 0.5 SOL
pub const MYTHIC_CARD_BASE_PRICE: u64 = 1_000_000_000;   // 1 SOL

/// Limited Edition Collections
pub const GENESIS_COLLECTION_SIZE: u32 = 1000;     // Genesis collection
pub const SEASONAL_COLLECTION_SIZE: u32 = 5000;    // Seasonal collections
pub const EVENT_COLLECTION_SIZE: u32 = 2500;       // Event collections
pub const PARTNERSHIP_COLLECTION_SIZE: u32 = 10000; // Partnership collections

/// Staking Integration for NFTs
pub const NFT_STAKING_MULTIPLIER: u16 = 1500;      // 15% bonus for staked NFTs
pub const MIN_NFT_STAKE_DURATION: i64 = 86400;     // 24 hours minimum
pub const MAX_NFT_STAKE_DURATION: i64 = 31536000;  // 1 year maximum
pub const NFT_UNSTAKE_COOLDOWN: i64 = 3600;        // 1 hour cooldown

/// Economic Constants
pub const BURN_FEE_PERCENTAGE: u8 = 10;            // 10% tokens burned on trade
pub const CREATOR_REWARD_PERCENTAGE: u8 = 5;       // 5% to original creator
pub const TREASURY_PERCENTAGE: u8 = 3;             // 3% to treasury
pub const LIQUIDITY_PERCENTAGE: u8 = 2;            // 2% to liquidity pool

/// Anti-Bot and Security
pub const MAX_NFTS_PER_TRANSACTION: u8 = 10;       // Limit NFTs per tx
pub const MAX_NFTS_PER_USER_PER_DAY: u16 = 100;    // Daily limit per user
pub const SUSPICIOUS_ACTIVITY_THRESHOLD: u16 = 50;  // Threshold for flagging
pub const KYC_REQUIRED_THRESHOLD: u64 = 100_000_000; // 0.1 SOL threshold

/// Card Evolution System
pub const EVOLUTION_THRESHOLD_1: u32 = 10;         // 10 uses to evolve
pub const EVOLUTION_THRESHOLD_2: u32 = 25;         // 25 uses to evolve
pub const EVOLUTION_THRESHOLD_3: u32 = 50;         // 50 uses to evolve
pub const MAX_EVOLUTION_LEVEL: u8 = 3;             // Maximum evolution level
pub const EVOLUTION_BONUS_PER_LEVEL: u16 = 500;    // 5% bonus per level

/// Guild Integration
pub const GUILD_NFT_BONUS: u16 = 1000;             // 10% bonus for guild NFTs
pub const GUILD_MASTER_NFT_BONUS: u16 = 2000;      // 20% bonus for guild masters
pub const MAX_GUILD_NFTS: u8 = 100;                // Max NFTs per guild
pub const GUILD_NFT_VOTING_POWER: u16 = 150;       // 1.5x voting power

/// Time Constants
pub const SECONDS_PER_DAY: i64 = 86400;
pub const SECONDS_PER_WEEK: i64 = 604800;
pub const SECONDS_PER_MONTH: i64 = 2592000;
pub const SECONDS_PER_YEAR: i64 = 31536000;

/// Error Codes Range (NFT specific: 6000-6999)
pub const NFT_ERROR_START: u32 = 6000;

/// Version Control
pub const NFT_PROGRAM_VERSION: u8 = 1;
pub const METADATA_VERSION: u8 = 1;
pub const COLLECTION_VERSION: u8 = 1;

/// Feature Flags
pub const ENABLE_CARD_STACKING: bool = true;       // Allow multiple same cards
pub const ENABLE_CARD_TRADING: bool = true;        // Allow peer-to-peer trading
pub const ENABLE_CARD_BURNING: bool = true;        // Allow burning for rewards
pub const ENABLE_CARD_EVOLUTION: bool = true;      // Allow card evolution
pub const ENABLE_MARKETPLACE_AUCTIONS: bool = true; // Enable auction system

/// Mathematical Constants
pub const BASIS_POINTS_DIVISOR: u16 = 10000;       // For percentage calculations
pub const PRECISION_MULTIPLIER: u64 = 1_000_000;   // For precise calculations
pub const HALF_PRECISION: u64 = 500_000;           // For rounding

/// Default Values
pub const DEFAULT_CARD_USES: u32 = 1;              // Single-use by default
pub const DEFAULT_COLLECTION_ROYALTY: u16 = 500;   // 5% default royalty
pub const DEFAULT_MARKETPLACE_DURATION: i64 = 86400 * 7; // 7 days default

/// Integration Constants
pub const METAPLEX_PROGRAM_ID: &str = "metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s";
pub const TOKEN_PROGRAM_ID: &str = "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA";
pub const ASSOCIATED_TOKEN_PROGRAM_ID: &str = "ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL";

/// Quality Score Integration
pub const QUALITY_SCORE_MULTIPLIER: u16 = 200;     // 2x max for quality
pub const MIN_QUALITY_SCORE: u16 = 50;             // 0.5x minimum
pub const VIRAL_CONTENT_THRESHOLD: u32 = 1000;     // 1K views for viral
pub const HIGH_QUALITY_THRESHOLD: u16 = 150;       // 1.5x quality threshold

/// Network Effect Constants
pub const NETWORK_EFFECT_BASE: u16 = 100;          // Base network effect
pub const NETWORK_EFFECT_PER_USER: u16 = 1;        // Per user bonus
pub const MAX_NETWORK_EFFECT: u16 = 500;           // Maximum network effect
pub const NETWORK_DECAY_RATE: u16 = 5;             // Decay rate for inactive users

/// Finova-specific Integration
pub const FINOVA_CORE_PROGRAM_INTEGRATION: bool = true;
pub const FINOVA_TOKEN_INTEGRATION: bool = true;
pub const CROSS_PROGRAM_INVOCATION_ENABLED: bool = true;

#[macro_export]
macro_rules! card_effect_multiplier {
    ($rarity:expr) => {
        match $rarity {
            COMMON_RARITY => 10000,        // 100% base effect
            UNCOMMON_RARITY => 11000,      // 110% effect
            RARE_RARITY => 12500,          // 125% effect
            EPIC_RARITY => 15000,          // 150% effect
            LEGENDARY_RARITY => 20000,     // 200% effect
            MYTHIC_RARITY => 30000,        // 300% effect
            _ => 10000,                    // Default to base
        }
    };
}

#[macro_export]
macro_rules! calculate_marketplace_fee {
    ($price:expr) => {
        ($price * MARKETPLACE_FEE_BASIS_POINTS as u64) / BASIS_POINTS_DIVISOR as u64
    };
}

#[macro_export]
macro_rules! calculate_royalty_fee {
    ($price:expr, $royalty:expr) => {
        ($price * $royalty as u64) / BASIS_POINTS_DIVISOR as u64
    };
}

#[macro_export]
macro_rules! is_card_expired {
    ($activation_time:expr, $duration:expr) => {
        Clock::get()?.unix_timestamp > $activation_time + $duration
    };
}

#[macro_export]
macro_rules! calculate_synergy_bonus {
    ($active_cards:expr, $same_category:expr, $all_categories:expr) => {
        {
            let mut bonus = SYNERGY_BASE_BONUS * $active_cards as u16;
            if $same_category {
                bonus += SAME_CATEGORY_BONUS;
            }
            if $all_categories {
                bonus += ALL_CATEGORIES_BONUS;
            }
            bonus
        }
    };
}
