// programs/finova-nft/src/events/mint.rs

use anchor_lang::prelude::*;
use crate::state::*;

/// Event emitted when a new NFT collection is created
#[event]
pub struct CollectionCreated {
    /// The collection's public key
    pub collection: Pubkey,
    /// The authority who created the collection
    pub authority: Pubkey,
    /// Collection name
    pub name: String,
    /// Collection symbol
    pub symbol: String,
    /// Collection description
    pub description: String,
    /// Maximum supply of NFTs in this collection (0 = unlimited)
    pub max_supply: u64,
    /// Creator royalty percentage (basis points, e.g., 500 = 5%)
    pub royalty_percentage: u16,
    /// Collection type (Regular, SpecialCard, Badge, etc.)
    pub collection_type: CollectionType,
    /// Timestamp when collection was created
    pub created_at: i64,
    /// Collection URI for metadata
    pub uri: String,
}

/// Event emitted when a new NFT is minted
#[event]
pub struct NftMinted {
    /// The minted NFT's public key
    pub nft: Pubkey,
    /// The collection this NFT belongs to
    pub collection: Pubkey,
    /// The mint authority (could be different from recipient)
    pub mint_authority: Pubkey,
    /// The recipient of the NFT
    pub recipient: Pubkey,
    /// Token ID within the collection
    pub token_id: u64,
    /// NFT name
    pub name: String,
    /// NFT description
    pub description: String,
    /// NFT image URI
    pub image_uri: String,
    /// NFT metadata URI
    pub metadata_uri: String,
    /// Rarity tier of the NFT
    pub rarity: RarityTier,
    /// Timestamp when NFT was minted
    pub minted_at: i64,
    /// Current supply of this collection after minting
    pub current_supply: u64,
    /// Whether this is a special card
    pub is_special_card: bool,
    /// If special card, the card type
    pub card_type: Option<SpecialCardType>,
    /// Attributes associated with the NFT
    pub attributes: Vec<NftAttribute>,
}

/// Event emitted when a special card NFT is minted with specific properties
#[event]
pub struct SpecialCardMinted {
    /// The minted special card's public key
    pub card: Pubkey,
    /// The collection this card belongs to
    pub collection: Pubkey,
    /// The recipient of the special card
    pub recipient: Pubkey,
    /// The type of special card
    pub card_type: SpecialCardType,
    /// The effect this card provides
    pub effect: CardEffect,
    /// Effect value (e.g., +100% for mining boost)
    pub effect_value: u64,
    /// Duration of the effect in seconds (0 = permanent)
    pub duration: u64,
    /// Number of uses for this card (0 = unlimited)
    pub uses: u32,
    /// Whether this card is transferable
    pub transferable: bool,
    /// Whether this card is tradeable on marketplace
    pub tradeable: bool,
    /// Rarity of the special card
    pub rarity: RarityTier,
    /// Price paid for this card (if purchased)
    pub price: Option<u64>,
    /// Currency used for purchase
    pub currency: Option<Pubkey>,
    /// Timestamp when card was minted
    pub minted_at: i64,
    /// Expiry timestamp (if applicable)
    pub expires_at: Option<i64>,
}

/// Event emitted when NFT metadata is updated
#[event]
pub struct NftMetadataUpdated {
    /// The NFT whose metadata was updated
    pub nft: Pubkey,
    /// The authority who updated the metadata
    pub update_authority: Pubkey,
    /// Old metadata URI
    pub old_metadata_uri: String,
    /// New metadata URI
    pub new_metadata_uri: String,
    /// Old name
    pub old_name: String,
    /// New name
    pub new_name: String,
    /// Old description
    pub old_description: String,
    /// New description
    pub new_description: String,
    /// Timestamp of update
    pub updated_at: i64,
    /// Reason for update
    pub update_reason: String,
}

/// Event emitted when an NFT's rarity is upgraded
#[event]
pub struct NftRarityUpgraded {
    /// The NFT whose rarity was upgraded
    pub nft: Pubkey,
    /// The owner of the NFT
    pub owner: Pubkey,
    /// Previous rarity tier
    pub old_rarity: RarityTier,
    /// New rarity tier
    pub new_rarity: RarityTier,
    /// Upgrade cost
    pub upgrade_cost: u64,
    /// Currency used for upgrade
    pub currency: Pubkey,
    /// New attributes after upgrade
    pub new_attributes: Vec<NftAttribute>,
    /// Timestamp of upgrade
    pub upgraded_at: i64,
    /// Upgrade method (burning other NFTs, payment, etc.)
    pub upgrade_method: UpgradeMethod,
}

/// Event emitted when a batch of NFTs is minted
#[event]
pub struct BatchNftMinted {
    /// The collection these NFTs belong to
    pub collection: Pubkey,
    /// The mint authority
    pub mint_authority: Pubkey,
    /// List of minted NFT public keys
    pub nfts: Vec<Pubkey>,
    /// List of recipients (same length as nfts)
    pub recipients: Vec<Pubkey>,
    /// Number of NFTs minted in this batch
    pub count: u32,
    /// Batch ID for tracking
    pub batch_id: String,
    /// Timestamp when batch was minted
    pub minted_at: i64,
    /// Total supply after batch mint
    pub total_supply_after: u64,
}

/// Event emitted when an NFT is minted as a reward
#[event]
pub struct RewardNftMinted {
    /// The rewarded NFT's public key
    pub nft: Pubkey,
    /// The collection this NFT belongs to
    pub collection: Pubkey,
    /// The recipient of the reward
    pub recipient: Pubkey,
    /// The reason for the reward
    pub reward_reason: RewardReason,
    /// Achievement or milestone that triggered the reward
    pub trigger_event: String,
    /// Value associated with the trigger (e.g., level reached, mining amount)
    pub trigger_value: u64,
    /// The rarity of the rewarded NFT
    pub rarity: RarityTier,
    /// Special properties for reward NFTs
    pub reward_properties: RewardProperties,
    /// Timestamp when reward was minted
    pub rewarded_at: i64,
    /// Whether this reward is permanent or temporary
    pub is_permanent: bool,
}

/// Event emitted when a dynamic NFT's properties change
#[event]
pub struct DynamicNftUpdated {
    /// The dynamic NFT that was updated
    pub nft: Pubkey,
    /// The owner of the NFT
    pub owner: Pubkey,
    /// Previous dynamic properties
    pub old_properties: DynamicProperties,
    /// New dynamic properties
    pub new_properties: DynamicProperties,
    /// What triggered the update
    pub update_trigger: UpdateTrigger,
    /// Timestamp of the update
    pub updated_at: i64,
    /// Whether the visual appearance changed
    pub visual_changed: bool,
    /// New metadata URI if visual changed
    pub new_metadata_uri: Option<String>,
}

/// Event emitted when NFT minting phase changes
#[event]
pub struct MintingPhaseChanged {
    /// The collection affected
    pub collection: Pubkey,
    /// Previous minting phase
    pub old_phase: MintingPhase,
    /// New minting phase
    pub new_phase: MintingPhase,
    /// Authority who changed the phase
    pub authority: Pubkey,
    /// Timestamp of phase change
    pub changed_at: i64,
    /// Phase change reason
    pub reason: String,
    /// New phase parameters
    pub phase_params: PhaseParams,
}

/// Supporting enums and structs for events

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum RarityTier {
    Common,
    Uncommon,
    Rare,
    Epic,
    Legendary,
    Mythic,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub enum SpecialCardType {
    MiningBoost,
    XpAccelerator,
    ReferralPower,
    GuildBuff,
    StakingBonus,
    QualityMultiplier,
    NetworkAmplifier,
    TimeExtender,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub enum CardEffect {
    MiningRateMultiplier,
    XpGainMultiplier,
    ReferralBonusMultiplier,
    StakingRewardMultiplier,
    QualityScoreBoost,
    NetworkEffectBoost,
    DurationExtension,
    CooldownReduction,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct NftAttribute {
    pub trait_type: String,
    pub value: String,
    pub display_type: Option<String>,
    pub max_value: Option<u64>,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub enum UpgradeMethod {
    Payment,
    BurnNfts,
    Achievement,
    TimeGated,
    Combination,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub enum RewardReason {
    LevelMilestone,
    MiningAchievement,
    ReferralMilestone,
    SocialEngagement,
    GuildParticipation,
    SeasonalEvent,
    SpecialPromotion,
    BugBounty,
    CommunityContribution,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct RewardProperties {
    pub bonus_multiplier: u64,
    pub duration_bonus: u64,
    pub special_abilities: Vec<String>,
    pub unlock_requirements: Vec<String>,
    pub upgrade_path: Option<String>,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct DynamicProperties {
    pub level: u32,
    pub experience: u64,
    pub power_rating: u64,
    pub evolution_stage: u8,
    pub special_traits: Vec<String>,
    pub boost_multipliers: Vec<BoostMultiplier>,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct BoostMultiplier {
    pub boost_type: String,
    pub multiplier: u64,
    pub expires_at: Option<i64>,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub enum UpdateTrigger {
    UserActivity,
    TimePassage,
    ExternalEvent,
    OwnerAction,
    NetworkCondition,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub enum MintingPhase {
    Preparation,
    Presale,
    PublicSale,
    RewardOnly,
    Paused,
    Completed,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct PhaseParams {
    pub price: u64,
    pub max_per_wallet: u32,
    pub max_per_transaction: u32,
    pub whitelist_required: bool,
    pub start_time: i64,
    pub end_time: Option<i64>,
    pub daily_limit: Option<u32>,
}

/// Event emitted when NFT staking begins
#[event]
pub struct NftStakingStarted {
    /// The NFT being staked
    pub nft: Pubkey,
    /// The owner/staker
    pub staker: Pubkey,
    /// The staking pool
    pub staking_pool: Pubkey,
    /// Duration of staking (0 = flexible)
    pub staking_duration: u64,
    /// Expected rewards per day
    pub daily_rewards: u64,
    /// Multiplier based on NFT rarity
    pub rarity_multiplier: u64,
    /// Timestamp when staking started
    pub staked_at: i64,
    /// Estimated rewards at maturity
    pub estimated_rewards: u64,
}

/// Event emitted when staked NFT rewards are claimed
#[event]
pub struct StakingRewardsClaimed {
    /// The staked NFT
    pub nft: Pubkey,
    /// The staker claiming rewards
    pub staker: Pubkey,
    /// Amount of rewards claimed
    pub rewards_claimed: u64,
    /// Token mint of the rewards
    pub reward_token: Pubkey,
    /// Days staked at time of claim
    pub days_staked: u32,
    /// Remaining rewards available
    pub remaining_rewards: u64,
    /// Timestamp of claim
    pub claimed_at: i64,
    /// Whether this was a partial or full claim
    pub claim_type: ClaimType,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub enum ClaimType {
    Partial,
    Full,
    Emergency,
}

/// Event emitted when NFT gains experience or levels up
#[event]
pub struct NftExperienceGained {
    /// The NFT that gained experience
    pub nft: Pubkey,
    /// The owner of the NFT
    pub owner: Pubkey,
    /// Experience points gained
    pub xp_gained: u64,
    /// Total XP after gain
    pub total_xp: u64,
    /// Previous level
    pub old_level: u32,
    /// New level (if leveled up)
    pub new_level: u32,
    /// Source of experience
    pub xp_source: XpSource,
    /// Activity that generated XP
    pub activity: String,
    /// Timestamp when XP was gained
    pub gained_at: i64,
    /// Whether NFT leveled up
    pub leveled_up: bool,
    /// New abilities unlocked (if any)
    pub new_abilities: Vec<String>,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub enum XpSource {
    Mining,
    SocialActivity,
    Staking,
    Trading,
    GuildActivity,
    SpecialEvent,
    Achievement,
}

/// Event emitted when NFT evolves or transforms
#[event]
pub struct NftEvolved {
    /// The NFT that evolved
    pub nft: Pubkey,
    /// The owner of the NFT
    pub owner: Pubkey,
    /// Previous evolution stage
    pub old_stage: u8,
    /// New evolution stage
    pub new_stage: u8,
    /// Evolution requirements met
    pub requirements_met: Vec<String>,
    /// New attributes gained
    pub new_attributes: Vec<NftAttribute>,
    /// New image URI
    pub new_image_uri: String,
    /// New metadata URI
    pub new_metadata_uri: String,
    /// Evolution cost (if any)
    pub evolution_cost: u64,
    /// Timestamp of evolution
    pub evolved_at: i64,
    /// Special evolution bonuses
    pub evolution_bonuses: Vec<EvolutionBonus>,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct EvolutionBonus {
    pub bonus_type: String,
    pub bonus_value: u64,
    pub duration: Option<u64>,
    pub permanent: bool,
}

/// Event emitted when NFT marketplace listing is created
#[event]
pub struct NftListedForSale {
    /// The NFT being listed
    pub nft: Pubkey,
    /// The seller
    pub seller: Pubkey,
    /// Listing price
    pub price: u64,
    /// Currency for the sale
    pub currency: Pubkey,
    /// Listing expiration time
    pub expires_at: i64,
    /// Marketplace listing ID
    pub listing_id: String,
    /// Whether auction or fixed price
    pub sale_type: SaleType,
    /// Minimum bid (for auctions)
    pub minimum_bid: Option<u64>,
    /// Auction duration (for auctions)
    pub auction_duration: Option<u64>,
    /// Timestamp when listed
    pub listed_at: i64,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub enum SaleType {
    FixedPrice,
    Auction,
    DutchAuction,
    Bundle,
}

/// Event for tracking NFT utility usage
#[event]
pub struct NftUtilityUsed {
    /// The NFT whose utility was used
    pub nft: Pubkey,
    /// The user who used the utility
    pub user: Pubkey,
    /// Type of utility used
    pub utility_type: UtilityType,
    /// Value/effect of the utility
    pub utility_value: u64,
    /// Duration of the effect
    pub effect_duration: u64,
    /// Uses remaining (if limited use)
    pub uses_remaining: Option<u32>,
    /// Timestamp when utility was used
    pub used_at: i64,
    /// Context of usage
    pub usage_context: String,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub enum UtilityType {
    MiningBoost,
    XpMultiplier,
    ReferralBonus,
    StakingBonus,
    AccessPass,
    PowerUp,
    Shield,
    Teleport,
}

/// Comprehensive event for NFT state changes
#[event]
pub struct NftStateChanged {
    /// The NFT whose state changed
    pub nft: Pubkey,
    /// The owner of the NFT
    pub owner: Pubkey,
    /// Previous state hash
    pub old_state_hash: [u8; 32],
    /// New state hash
    pub new_state_hash: [u8; 32],
    /// Fields that changed
    pub changed_fields: Vec<String>,
    /// Reason for state change
    pub change_reason: StateChangeReason,
    /// Transaction that caused the change
    pub transaction_signature: String,
    /// Timestamp of change
    pub changed_at: i64,
    /// Additional context data
    pub context_data: Vec<u8>,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub enum StateChangeReason {
    Transfer,
    Upgrade,
    Evolution,
    Staking,
    Unstaking,
    Burning,
    MetadataUpdate,
    UtilityUsage,
    AdminAction,
}

impl CollectionCreated {
    pub fn new(
        collection: Pubkey,
        authority: Pubkey,
        name: String,
        symbol: String,
        description: String,
        max_supply: u64,
        royalty_percentage: u16,
        collection_type: CollectionType,
        uri: String,
    ) -> Self {
        Self {
            collection,
            authority,
            name,
            symbol,
            description,
            max_supply,
            royalty_percentage,
            collection_type,
            created_at: Clock::get().unwrap().unix_timestamp,
            uri,
        }
    }
}

impl NftMinted {
    pub fn new(
        nft: Pubkey,
        collection: Pubkey,
        mint_authority: Pubkey,
        recipient: Pubkey,
        token_id: u64,
        name: String,
        description: String,
        image_uri: String,
        metadata_uri: String,
        rarity: RarityTier,
        current_supply: u64,
        is_special_card: bool,
        card_type: Option<SpecialCardType>,
        attributes: Vec<NftAttribute>,
    ) -> Self {
        Self {
            nft,
            collection,
            mint_authority,
            recipient,
            token_id,
            name,
            description,
            image_uri,
            metadata_uri,
            rarity,
            minted_at: Clock::get().unwrap().unix_timestamp,
            current_supply,
            is_special_card,
            card_type,
            attributes,
        }
    }
}

impl SpecialCardMinted {
    pub fn new(
        card: Pubkey,
        collection: Pubkey,
        recipient: Pubkey,
        card_type: SpecialCardType,
        effect: CardEffect,
        effect_value: u64,
        duration: u64,
        uses: u32,
        transferable: bool,
        tradeable: bool,
        rarity: RarityTier,
        price: Option<u64>,
        currency: Option<Pubkey>,
        expires_at: Option<i64>,
    ) -> Self {
        Self {
            card,
            collection,
            recipient,
            card_type,
            effect,
            effect_value,
            duration,
            uses,
            transferable,
            tradeable,
            rarity,
            price,
            currency,
            minted_at: Clock::get().unwrap().unix_timestamp,
            expires_at,
        }
    }
}
