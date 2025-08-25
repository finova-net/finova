// programs/finova-nft/src/events/mod.rs

use anchor_lang::prelude::*;

/// Collection creation event
#[event]
pub struct CollectionCreated {
    pub collection: Pubkey,
    pub authority: Pubkey,
    pub name: String,
    pub symbol: String,
    pub uri: String,
    pub max_supply: Option<u64>,
    pub timestamp: i64,
}

/// NFT minting event
#[event]
pub struct NFTMinted {
    pub collection: Pubkey,
    pub mint: Pubkey,
    pub owner: Pubkey,
    pub metadata: Pubkey,
    pub name: String,
    pub uri: String,
    pub rarity: u8,
    pub special_card_type: Option<u8>,
    pub timestamp: i64,
}

/// Metadata update event
#[event]
pub struct MetadataUpdated {
    pub mint: Pubkey,
    pub metadata: Pubkey,
    pub authority: Pubkey,
    pub old_uri: String,
    pub new_uri: String,
    pub timestamp: i64,
}

/// NFT transfer event
#[event]
pub struct NFTTransferred {
    pub mint: Pubkey,
    pub from: Pubkey,
    pub to: Pubkey,
    pub amount: u64,
    pub timestamp: i64,
}

/// NFT burn event
#[event]
pub struct NFTBurned {
    pub mint: Pubkey,
    pub owner: Pubkey,
    pub amount: u64,
    pub collection: Option<Pubkey>,
    pub timestamp: i64,
}

/// Special card usage event
#[event]
pub struct SpecialCardUsed {
    pub mint: Pubkey,
    pub user: Pubkey,
    pub card_type: u8,
    pub effect_type: u8,
    pub effect_value: u32,
    pub duration: u32,
    pub is_permanent: bool,
    pub timestamp: i64,
}

/// Marketplace listing event
#[event]
pub struct NFTListed {
    pub listing: Pubkey,
    pub mint: Pubkey,
    pub seller: Pubkey,
    pub price: u64,
    pub currency_mint: Pubkey,
    pub timestamp: i64,
}

/// Marketplace delisting event
#[event]
pub struct NFTDelisted {
    pub listing: Pubkey,
    pub mint: Pubkey,
    pub seller: Pubkey,
    pub timestamp: i64,
}

/// Marketplace sale event
#[event]
pub struct NFTSold {
    pub listing: Pubkey,
    pub mint: Pubkey,
    pub seller: Pubkey,
    pub buyer: Pubkey,
    pub price: u64,
    pub currency_mint: Pubkey,
    pub marketplace_fee: u64,
    pub seller_fee: u64,
    pub timestamp: i64,
}

/// Price update event
#[event]
pub struct PriceUpdated {
    pub listing: Pubkey,
    pub mint: Pubkey,
    pub seller: Pubkey,
    pub old_price: u64,
    pub new_price: u64,
    pub timestamp: i64,
}

/// Badge assignment event
#[event]
pub struct BadgeAssigned {
    pub mint: Pubkey,
    pub user: Pubkey,
    pub badge_type: u8,
    pub level: u8,
    pub permanent_bonus: u16,
    pub timestamp: i64,
}

/// Badge upgrade event
#[event]
pub struct BadgeUpgraded {
    pub mint: Pubkey,
    pub user: Pubkey,
    pub badge_type: u8,
    pub old_level: u8,
    pub new_level: u8,
    pub bonus_increase: u16,
    pub timestamp: i64,
}

/// Achievement NFT event
#[event]
pub struct AchievementUnlocked {
    pub mint: Pubkey,
    pub user: Pubkey,
    pub achievement_type: u8,
    pub milestone: u32,
    pub bonus_effect: u16,
    pub is_rare: bool,
    pub timestamp: i64,
}

/// Card synergy activation event
#[event]
pub struct SynergyActivated {
    pub user: Pubkey,
    pub cards: Vec<Pubkey>,
    pub synergy_type: u8,
    pub multiplier: u16,
    pub duration: u32,
    pub timestamp: i64,
}

/// Bulk transfer event for efficiency
#[event]
pub struct BulkTransfer {
    pub from: Pubkey,
    pub to: Pubkey,
    pub mints: Vec<Pubkey>,
    pub amounts: Vec<u64>,
    pub timestamp: i64,
}

/// Collection verification event
#[event]
pub struct CollectionVerified {
    pub collection: Pubkey,
    pub verifier: Pubkey,
    pub verified: bool,
    pub verification_level: u8,
    pub timestamp: i64,
}

/// Royalty configuration event
#[event]
pub struct RoyaltyConfigured {
    pub collection: Pubkey,
    pub creator: Pubkey,
    pub royalty_percentage: u16,
    pub royalty_recipient: Pubkey,
    pub timestamp: i64,
}

/// Metadata freeze event
#[event]
pub struct MetadataFrozen {
    pub mint: Pubkey,
    pub authority: Pubkey,
    pub frozen: bool,
    pub timestamp: i64,
}

/// Collection authority update event
#[event]
pub struct CollectionAuthorityUpdated {
    pub collection: Pubkey,
    pub old_authority: Pubkey,
    pub new_authority: Pubkey,
    pub timestamp: i64,
}

/// Special card effectiveness event
#[event]
pub struct CardEffectApplied {
    pub user: Pubkey,
    pub card_mint: Pubkey,
    pub effect_type: u8,
    pub base_value: u32,
    pub multiplied_value: u32,
    pub synergy_bonus: u16,
    pub duration_remaining: u32,
    pub timestamp: i64,
}

/// Marketplace fee configuration event
#[event]
pub struct MarketplaceFeeUpdated {
    pub authority: Pubkey,
    pub old_fee_percentage: u16,
    pub new_fee_percentage: u16,
    pub fee_recipient: Pubkey,
    pub timestamp: i64,
}

/// Batch minting event for gas efficiency
#[event]
pub struct BatchMinted {
    pub collection: Pubkey,
    pub authority: Pubkey,
    pub mints: Vec<Pubkey>,
    pub recipients: Vec<Pubkey>,
    pub rarities: Vec<u8>,
    pub batch_size: u16,
    pub timestamp: i64,
}

/// Card combination/fusion event
#[event]
pub struct CardsCombined {
    pub user: Pubkey,
    pub input_cards: Vec<Pubkey>,
    pub output_card: Pubkey,
    pub combination_type: u8,
    pub success_rate: u16,
    pub bonus_applied: bool,
    pub timestamp: i64,
}

/// Staking rewards from NFT ownership
#[event]
pub struct NFTStakingReward {
    pub user: Pubkey,
    pub staked_nfts: Vec<Pubkey>,
    pub reward_amount: u64,
    pub reward_mint: Pubkey,
    pub staking_duration: u32,
    pub bonus_multiplier: u16,
    pub timestamp: i64,
}

/// Guild NFT bonus event
#[event]
pub struct GuildNFTBonus {
    pub guild: Pubkey,
    pub user: Pubkey,
    pub nft_mint: Pubkey,
    pub bonus_type: u8,
    pub bonus_value: u32,
    pub guild_level_bonus: u16,
    pub timestamp: i64,
}

/// Seasonal/Event NFT distribution
#[event]
pub struct EventNFTDistributed {
    pub event_id: u32,
    pub mint: Pubkey,
    pub recipient: Pubkey,
    pub event_type: u8,
    pub rarity_tier: u8,
    pub limited_edition: bool,
    pub edition_number: Option<u32>,
    pub timestamp: i64,
}

/// NFT level up/evolution event
#[event]
pub struct NFTEvolved {
    pub mint: Pubkey,
    pub owner: Pubkey,
    pub old_level: u8,
    pub new_level: u8,
    pub evolution_type: u8,
    pub stat_increases: Vec<u16>,
    pub timestamp: i64,
}

/// Cross-program NFT integration
#[event]
pub struct CrossProgramIntegration {
    pub nft_mint: Pubkey,
    pub user: Pubkey,
    pub target_program: Pubkey,
    pub integration_type: u8,
    pub bonus_applied: u32,
    pub duration: u32,
    pub timestamp: i64,
}

/// NFT rental/lending event
#[event]
pub struct NFTRented {
    pub mint: Pubkey,
    pub owner: Pubkey,
    pub renter: Pubkey,
    pub rental_price: u64,
    pub rental_duration: u32,
    pub benefits_shared: bool,
    pub timestamp: i64,
}

/// Anti-fraud detection event
#[event]
pub struct SuspiciousActivityDetected {
    pub user: Pubkey,
    pub nft_mint: Option<Pubkey>,
    pub activity_type: u8,
    pub risk_score: u16,
    pub action_taken: u8,
    pub timestamp: i64,
}

/// Marketplace statistics update
#[event]
pub struct MarketplaceStats {
    pub total_listings: u64,
    pub total_sales: u64,
    pub total_volume: u64,
    pub average_price: u64,
    pub top_collection: Pubkey,
    pub timestamp: i64,
}

/// Dynamic pricing event
#[event]
pub struct DynamicPriceUpdate {
    pub listing: Pubkey,
    pub mint: Pubkey,
    pub base_price: u64,
    pub demand_multiplier: u16,
    pub rarity_multiplier: u16,
    pub final_price: u64,
    pub timestamp: i64,
}

/// Whitelist management event
#[event]
pub struct WhitelistUpdated {
    pub collection: Pubkey,
    pub authority: Pubkey,
    pub user: Pubkey,
    pub added: bool,
    pub whitelist_type: u8,
    pub timestamp: i64,
}

/// Airdrop event for community rewards
#[event]
pub struct AirdropExecuted {
    pub collection: Pubkey,
    pub authority: Pubkey,
    pub recipients: Vec<Pubkey>,
    pub mints: Vec<Pubkey>,
    pub airdrop_type: u8,
    pub total_distributed: u16,
    pub timestamp: i64,
}

/// NFT utility activation
#[event]
pub struct UtilityActivated {
    pub mint: Pubkey,
    pub user: Pubkey,
    pub utility_type: u8,
    pub activation_cost: u64,
    pub duration: Option<u32>,
    pub benefits: Vec<u32>,
    pub timestamp: i64,
}

/// Governance voting with NFT weight
#[event]
pub struct NFTGovernanceVote {
    pub proposal_id: u64,
    pub voter: Pubkey,
    pub nft_mints: Vec<Pubkey>,
    pub voting_power: u64,
    pub vote_option: u8,
    pub timestamp: i64,
}

/// Revenue sharing from NFT collections
#[event]
pub struct RevenueShared {
    pub collection: Pubkey,
    pub holders: Vec<Pubkey>,
    pub amounts: Vec<u64>,
    pub total_distributed: u64,
    pub revenue_source: u8,
    pub timestamp: i64,
}

/// NFT analytics tracking
#[event]
pub struct NFTAnalytics {
    pub mint: Pubkey,
    pub view_count: u64,
    pub interaction_count: u64,
    pub transfer_count: u32,
    pub last_sale_price: Option<u64>,
    pub popularity_score: u16,
    pub timestamp: i64,
}

/// Emergency pause/unpause event
#[event]
pub struct EmergencyAction {
    pub authority: Pubkey,
    pub action_type: u8, // 0: pause, 1: unpause, 2: emergency_transfer
    pub target: Option<Pubkey>,
    pub reason: String,
    pub timestamp: i64,
}

/// Community features activation
#[event]
pub struct CommunityFeatureActivated {
    pub collection: Pubkey,
    pub feature_type: u8,
    pub activation_threshold: u64,
    pub current_holders: u64,
    pub benefits_unlocked: Vec<u8>,
    pub timestamp: i64,
}

/// Cross-chain bridge preparation
#[event]
pub struct BridgePreparation {
    pub mint: Pubkey,
    pub owner: Pubkey,
    pub target_chain: u8,
    pub bridge_fee: u64,
    pub estimated_arrival: i64,
    pub timestamp: i64,
}

/// Quality score update for dynamic NFTs
#[event]
pub struct QualityScoreUpdated {
    pub mint: Pubkey,
    pub old_score: u16,
    pub new_score: u16,
    pub factors: Vec<u8>,
    pub bonus_tier_changed: bool,
    pub timestamp: i64,
}

/// Environmental impact tracking
#[event]
pub struct CarbonFootprint {
    pub transaction_type: u8,
    pub energy_consumed: u64,
    pub carbon_offset: u64,
    pub eco_friendly_bonus: bool,
    pub timestamp: i64,
}

/// Social media integration tracking
#[event]
pub struct SocialIntegration {
    pub nft_mint: Pubkey,
    pub user: Pubkey,
    pub platform: u8,
    pub shares: u32,
    pub likes: u32,
    pub bonus_earned: u64,
    pub timestamp: i64,
}
