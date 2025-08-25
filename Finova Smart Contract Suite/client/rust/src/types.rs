// finova-net/finova/client/rust/src/types.rs

use anchor_lang::prelude::*;
use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;
use std::collections::HashMap;

// ============================================================================
// Core Account Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, AnchorSerialize, AnchorDeserialize)]
pub struct UserAccount {
    pub authority: Pubkey,
    pub username: String,
    pub email: String,
    pub kyc_verified: bool,
    pub created_at: i64,
    pub last_active: i64,
    pub total_fin_earned: u64,
    pub total_fin_holdings: u64,
    pub xp_level: u32,
    pub xp_points: u64,
    pub rp_tier: ReferralTier,
    pub rp_points: u64,
    pub mining_rate: u64, // in micro-FIN per hour
    pub referral_code: String,
    pub referred_by: Option<Pubkey>,
    pub guild_id: Option<Pubkey>,
    pub social_profiles: Vec<SocialProfile>,
    pub activity_streak: u32,
    pub quality_score: u64, // 0-10000 (0.0001 precision)
    pub human_probability: u64, // 0-10000 (0.0001 precision)
    pub network_size: u32,
    pub active_cards: Vec<ActiveCard>,
    pub achievements: Vec<Achievement>,
    pub staking_info: StakingInfo,
    pub bump: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize, AnchorSerialize, AnchorDeserialize)]
pub struct MiningAccount {
    pub user: Pubkey,
    pub base_rate: u64,
    pub current_rate: u64,
    pub finizen_bonus: u64, // 10000 = 1.0x
    pub referral_bonus: u64,
    pub security_bonus: u64,
    pub regression_factor: u64,
    pub last_claim: i64,
    pub accumulated_rewards: u64,
    pub mining_phase: MiningPhase,
    pub daily_cap: u64,
    pub claimed_today: u64,
    pub boost_multiplier: u64,
    pub boost_expires: i64,
    pub bump: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize, AnchorSerialize, AnchorDeserialize)]
pub struct XpAccount {
    pub user: Pubkey,
    pub current_level: u32,
    pub current_xp: u64,
    pub total_xp: u64,
    pub level_multiplier: u64, // 10000 = 1.0x
    pub daily_xp_earned: u64,
    pub last_reset: i64,
    pub streak_bonus: u64,
    pub activity_bonuses: HashMap<String, ActivityBonus>,
    pub platform_multipliers: HashMap<String, u64>,
    pub level_rewards_claimed: Vec<u32>,
    pub bump: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize, AnchorSerialize, AnchorDeserialize)]
pub struct ReferralAccount {
    pub user: Pubkey,
    pub referral_code: String,
    pub tier: ReferralTier,
    pub total_points: u64,
    pub direct_referrals: Vec<DirectReferral>,
    pub network_stats: NetworkStats,
    pub tier_multiplier: u64, // 10000 = 1.0x
    pub commission_rate: u64, // basis points
    pub network_quality_score: u64,
    pub regression_factor: u64,
    pub monthly_volume: u64,
    pub lifetime_earnings: u64,
    pub bump: u8,
}

// ============================================================================
// Token & NFT Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, AnchorSerialize, AnchorDeserialize)]
pub struct TokenMintInfo {
    pub mint: Pubkey,
    pub token_type: TokenType,
    pub total_supply: u64,
    pub max_supply: u64,
    pub mining_pool: u64,
    pub staking_pool: u64,
    pub treasury_pool: u64,
    pub burn_count: u64,
    pub decimals: u8,
    pub frozen: bool,
    pub bump: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize, AnchorSerialize, AnchorDeserialize)]
pub struct StakeAccount {
    pub user: Pubkey,
    pub amount: u64,
    pub stake_type: StakeType,
    pub start_time: i64,
    pub last_claim: i64,
    pub accumulated_rewards: u64,
    pub tier: StakingTier,
    pub multiplier: u64, // 10000 = 1.0x
    pub loyalty_bonus: u64,
    pub activity_bonus: u64,
    pub unlock_time: Option<i64>,
    pub auto_compound: bool,
    pub bump: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize, AnchorSerialize, AnchorDeserialize)]
pub struct NftCollection {
    pub collection_mint: Pubkey,
    pub authority: Pubkey,
    pub name: String,
    pub symbol: String,
    pub uri: String,
    pub category: NftCategory,
    pub max_supply: u32,
    pub current_supply: u32,
    pub royalty_percentage: u16, // basis points
    pub royalty_destination: Pubkey,
    pub verified: bool,
    pub frozen: bool,
    pub bump: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize, AnchorSerialize, AnchorDeserialize)]
pub struct NftMetadata {
    pub mint: Pubkey,
    pub collection: Pubkey,
    pub owner: Pubkey,
    pub name: String,
    pub symbol: String,
    pub uri: String,
    pub card_type: Option<CardType>,
    pub rarity: NftRarity,
    pub utility: NftUtility,
    pub created_at: i64,
    pub last_used: Option<i64>,
    pub use_count: u32,
    pub max_uses: Option<u32>,
    pub tradeable: bool,
    pub burnable: bool,
    pub bump: u8,
}

// ============================================================================
// DeFi & Economics Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, AnchorSerialize, AnchorDeserialize)]
pub struct LiquidityPool {
    pub token_a: Pubkey,
    pub token_b: Pubkey,
    pub token_a_vault: Pubkey,
    pub token_b_vault: Pubkey,
    pub lp_token_mint: Pubkey,
    pub fee_rate: u64, // basis points
    pub total_liquidity: u64,
    pub volume_24h: u64,
    pub fees_collected: u64,
    pub curve_type: CurveType,
    pub amplification: Option<u64>,
    pub last_update: i64,
    pub frozen: bool,
    pub bump: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize, AnchorSerialize, AnchorDeserialize)]
pub struct RewardPool {
    pub pool_id: String,
    pub reward_token: Pubkey,
    pub total_rewards: u64,
    pub distributed_rewards: u64,
    pub reward_rate: u64, // rewards per second
    pub start_time: i64,
    pub end_time: i64,
    pub participants: u32,
    pub qualification_criteria: QualificationCriteria,
    pub distribution_method: DistributionMethod,
    pub bump: u8,
}

// ============================================================================
// Guild & Governance Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, AnchorSerialize, AnchorDeserialize)]
pub struct Guild {
    pub guild_id: Pubkey,
    pub name: String,
    pub description: String,
    pub logo_uri: String,
    pub master: Pubkey,
    pub officers: Vec<Pubkey>,
    pub members: Vec<GuildMember>,
    pub level: u32,
    pub experience: u64,
    pub treasury: u64,
    pub max_members: u32,
    pub created_at: i64,
    pub competition_stats: CompetitionStats,
    pub benefits: GuildBenefits,
    pub requirements: JoinRequirements,
    pub bump: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize, AnchorSerialize, AnchorDeserialize)]
pub struct GovernanceProposal {
    pub proposal_id: u64,
    pub proposer: Pubkey,
    pub title: String,
    pub description: String,
    pub proposal_type: ProposalType,
    pub voting_power_required: u64,
    pub votes_for: u64,
    pub votes_against: u64,
    pub abstain: u64,
    pub start_time: i64,
    pub end_time: i64,
    pub execution_time: Option<i64>,
    pub status: ProposalStatus,
    pub parameters: ProposalParameters,
    pub bump: u8,
}

// ============================================================================
// Social & Activity Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, AnchorSerialize, AnchorDeserialize)]
pub struct SocialProfile {
    pub platform: SocialPlatform,
    pub username: String,
    pub profile_id: String,
    pub verified: bool,
    pub connected_at: i64,
    pub followers_count: u32,
    pub activity_score: u64,
    pub last_sync: i64,
    pub permissions: Vec<Permission>,
}

#[derive(Debug, Clone, Serialize, Deserialize, AnchorSerialize, AnchorDeserialize)]
pub struct ActivityRecord {
    pub user: Pubkey,
    pub activity_type: ActivityType,
    pub platform: SocialPlatform,
    pub content_hash: String,
    pub timestamp: i64,
    pub xp_earned: u64,
    pub quality_score: u64,
    pub engagement_metrics: EngagementMetrics,
    pub ai_analysis: AiAnalysis,
    pub verified: bool,
    pub rewards_claimed: bool,
}

// ============================================================================
// Enums
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, AnchorSerialize, AnchorDeserialize, PartialEq)]
pub enum ReferralTier {
    Explorer,     // 0-999 RP
    Connector,    // 1,000-4,999 RP
    Influencer,   // 5,000-14,999 RP
    Leader,       // 15,000-49,999 RP
    Ambassador,   // 50,000+ RP
}

#[derive(Debug, Clone, Serialize, Deserialize, AnchorSerialize, AnchorDeserialize, PartialEq)]
pub enum MiningPhase {
    Finizen,      // 0-100K users
    Growth,       // 100K-1M users
    Maturity,     // 1M-10M users
    Stability,    // 10M+ users
}

#[derive(Debug, Clone, Serialize, Deserialize, AnchorSerialize, AnchorDeserialize, PartialEq)]
pub enum TokenType {
    FIN,          // Primary utility token
    SFIN,         // Staked FIN
    USDFIN,       // Synthetic stablecoin
    SUSDFIN,      // Staked USDFIN
}

#[derive(Debug, Clone, Serialize, Deserialize, AnchorSerialize, AnchorDeserialize, PartialEq)]
pub enum StakeType {
    Flexible,     // No lock period
    Fixed30,      // 30 days lock
    Fixed90,      // 90 days lock
    Fixed180,     // 180 days lock
    Fixed365,     // 365 days lock
}

#[derive(Debug, Clone, Serialize, Deserialize, AnchorSerialize, AnchorDeserialize, PartialEq)]
pub enum StakingTier {
    Bronze,       // 100-499 FIN
    Silver,       // 500-999 FIN
    Gold,         // 1,000-4,999 FIN
    Platinum,     // 5,000-9,999 FIN
    Diamond,      // 10,000+ FIN
}

#[derive(Debug, Clone, Serialize, Deserialize, AnchorSerialize, AnchorDeserialize, PartialEq)]
pub enum NftCategory {
    ProfileBadge,
    SpecialCard,
    Achievement,
    Collectible,
    Utility,
}

#[derive(Debug, Clone, Serialize, Deserialize, AnchorSerialize, AnchorDeserialize, PartialEq)]
pub enum CardType {
    DoubleMining,
    TripleMining,
    MiningFrenzy,
    EternalMiner,
    XpDouble,
    StreakSaver,
    LevelRush,
    XpMagnet,
    ReferralBoost,
    NetworkAmplifier,
    AmbassadorPass,
    NetworkKing,
}

#[derive(Debug, Clone, Serialize, Deserialize, AnchorSerialize, AnchorDeserialize, PartialEq)]
pub enum NftRarity {
    Common,
    Uncommon,
    Rare,
    Epic,
    Legendary,
    Mythic,
}

#[derive(Debug, Clone, Serialize, Deserialize, AnchorSerialize, AnchorDeserialize, PartialEq)]
pub enum SocialPlatform {
    Instagram,
    TikTok,
    YouTube,
    Facebook,
    TwitterX,
    LinkedIn,
    Telegram,
}

#[derive(Debug, Clone, Serialize, Deserialize, AnchorSerialize, AnchorDeserialize, PartialEq)]
pub enum ActivityType {
    Post,
    Comment,
    Like,
    Share,
    Follow,
    Story,
    Reel,
    Video,
    Live,
    DailyLogin,
    QuestComplete,
    Referral,
    KycComplete,
}

#[derive(Debug, Clone, Serialize, Deserialize, AnchorSerialize, AnchorDeserialize, PartialEq)]
pub enum ProposalType {
    ParameterChange,
    FeatureAddition,
    TreasuryAllocation,
    CommunityInitiative,
    EmergencyAction,
}

#[derive(Debug, Clone, Serialize, Deserialize, AnchorSerialize, AnchorDeserialize, PartialEq)]
pub enum ProposalStatus {
    Draft,
    Active,
    Passed,
    Rejected,
    Executed,
    Expired,
}

#[derive(Debug, Clone, Serialize, Deserialize, AnchorSerialize, AnchorDeserialize, PartialEq)]
pub enum CurveType {
    ConstantProduct,
    StableSwap,
    Concentrated,
}

#[derive(Debug, Clone, Serialize, Deserialize, AnchorSerialize, AnchorDeserialize, PartialEq)]
pub enum DistributionMethod {
    Linear,
    Exponential,
    Merit,
    Lottery,
}

// ============================================================================
// Nested Structs
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, AnchorSerialize, AnchorDeserialize)]
pub struct StakingInfo {
    pub total_staked: u64,
    pub current_tier: StakingTier,
    pub multiplier: u64,
    pub last_reward: i64,
    pub claimable_rewards: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, AnchorSerialize, AnchorDeserialize)]
pub struct ActiveCard {
    pub card_mint: Pubkey,
    pub card_type: CardType,
    pub effect_value: u64,
    pub expires_at: i64,
    pub uses_remaining: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, AnchorSerialize, AnchorDeserialize)]
pub struct Achievement {
    pub achievement_id: String,
    pub name: String,
    pub description: String,
    pub earned_at: i64,
    pub nft_mint: Option<Pubkey>,
    pub bonus_multiplier: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, AnchorSerialize, AnchorDeserialize)]
pub struct ActivityBonus {
    pub activity_type: ActivityType,
    pub multiplier: u64,
    pub expires_at: i64,
    pub max_daily_uses: Option<u32>,
    pub current_uses: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, AnchorSerialize, AnchorDeserialize)]
pub struct DirectReferral {
    pub user: Pubkey,
    pub joined_at: i64,
    pub total_earned: u64,
    pub last_active: i64,
    pub tier: ReferralTier,
}

#[derive(Debug, Clone, Serialize, Deserialize, AnchorSerialize, AnchorDeserialize)]
pub struct NetworkStats {
    pub total_size: u32,
    pub active_size: u32,
    pub level_2_size: u32,
    pub level_3_size: u32,
    pub retention_rate: u64, // basis points
    pub quality_score: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, AnchorSerialize, AnchorDeserialize)]
pub struct NftUtility {
    pub mining_boost: Option<u64>,
    pub xp_boost: Option<u64>,
    pub rp_boost: Option<u64>,
    pub special_access: Vec<String>,
    pub governance_weight: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, AnchorSerialize, AnchorDeserialize)]
pub struct QualificationCriteria {
    pub min_level: Option<u32>,
    pub min_stake: Option<u64>,
    pub min_activity: Option<u64>,
    pub kyc_required: bool,
    pub social_connected: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, AnchorSerialize, AnchorDeserialize)]
pub struct GuildMember {
    pub user: Pubkey,
    pub role: GuildRole,
    pub joined_at: i64,
    pub contribution_score: u64,
    pub last_active: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, AnchorSerialize, AnchorDeserialize)]
pub struct CompetitionStats {
    pub battles_won: u32,
    pub battles_lost: u32,
    pub tournaments_won: u32,
    pub total_prizes: u64,
    pub current_rank: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, AnchorSerialize, AnchorDeserialize)]
pub struct GuildBenefits {
    pub xp_bonus: u64,
    pub mining_bonus: u64,
    pub exclusive_cards: Vec<CardType>,
    pub tournament_access: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, AnchorSerialize, AnchorDeserialize)]
pub struct JoinRequirements {
    pub min_level: u32,
    pub min_stake: u64,
    pub invite_only: bool,
    pub application_required: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, AnchorSerialize, AnchorDeserialize)]
pub struct ProposalParameters {
    pub target_program: Option<Pubkey>,
    pub instruction_data: Option<Vec<u8>>,
    pub accounts_meta: Option<Vec<AccountMeta>>,
    pub numeric_params: HashMap<String, u64>,
    pub string_params: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, AnchorSerialize, AnchorDeserialize)]
pub struct EngagementMetrics {
    pub views: u64,
    pub likes: u64,
    pub comments: u64,
    pub shares: u64,
    pub saves: u64,
    pub click_rate: u64, // basis points
    pub completion_rate: u64, // basis points
}

#[derive(Debug, Clone, Serialize, Deserialize, AnchorSerialize, AnchorDeserialize)]
pub struct AiAnalysis {
    pub quality_score: u64, // 0-10000
    pub originality_score: u64,
    pub engagement_prediction: u64,
    pub brand_safety_score: u64,
    pub human_generated_probability: u64,
    pub content_category: String,
    pub sentiment_score: i32, // -10000 to +10000
    pub language_detected: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, AnchorSerialize, AnchorDeserialize, PartialEq)]
pub enum GuildRole {
    Member,
    Officer,
    Master,
}

#[derive(Debug, Clone, Serialize, Deserialize, AnchorSerialize, AnchorDeserialize, PartialEq)]
pub enum Permission {
    ReadProfile,
    PostContent,
    ReadMessages,
    ManageContent,
    AnalyzeEngagement,
}

// ============================================================================
// Client Request/Response Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InitializeUserRequest {
    pub username: String,
    pub email: String,
    pub referral_code: Option<String>,
    pub social_profiles: Vec<SocialProfile>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaimMiningRewardsRequest {
    pub user: Pubkey,
    pub amount: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateXpRequest {
    pub user: Pubkey,
    pub activity: ActivityRecord,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StakeTokensRequest {
    pub amount: u64,
    pub stake_type: StakeType,
    pub auto_compound: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateGuildRequest {
    pub name: String,
    pub description: String,
    pub logo_uri: String,
    pub max_members: u32,
    pub requirements: JoinRequirements,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserStatsResponse {
    pub user_account: UserAccount,
    pub mining_account: MiningAccount,
    pub xp_account: XpAccount,
    pub referral_account: ReferralAccount,
    pub stake_accounts: Vec<StakeAccount>,
    pub nft_holdings: Vec<NftMetadata>,
    pub guild_membership: Option<Guild>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkStatsResponse {
    pub total_users: u64,
    pub active_miners: u64,
    pub current_phase: MiningPhase,
    pub total_fin_supply: u64,
    pub total_staked: u64,
    pub average_mining_rate: u64,
    pub network_health_score: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeaderboardEntry {
    pub rank: u32,
    pub user: Pubkey,
    pub username: String,
    pub score: u64,
    pub tier: String,
    pub change_24h: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeaderboardResponse {
    pub mining_leaders: Vec<LeaderboardEntry>,
    pub xp_leaders: Vec<LeaderboardEntry>,
    pub referral_leaders: Vec<LeaderboardEntry>,
    pub guild_leaders: Vec<LeaderboardEntry>,
}

// ============================================================================
// Error Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FinovaError {
    InvalidUser,
    InsufficientBalance,
    MiningCapExceeded,
    InvalidReferralCode,
    KycNotVerified,
    SocialNotConnected,
    GuildFull,
    InsufficientPermissions,
    CardExpired,
    InvalidActivity,
    RateLimitExceeded,
    MaintenanceMode,
}

// ============================================================================
// Constants
// ============================================================================

pub const FINIZEN_MULTIPLIER: u64 = 20000; // 2.0x
pub const MAX_DAILY_MINING_CAP: u64 = 15_000_000; // 15 FIN (micro-FIN)
pub const XP_LEVEL_BASE: u64 = 1000;
pub const RP_TIER_THRESHOLDS: [u64; 5] = [1000, 5000, 15000, 50000, u64::MAX];
pub const REGRESSION_COEFFICIENT: f64 = 0.001;
pub const QUALITY_SCORE_PRECISION: u64 = 10000; // 4 decimal places
pub const BASIS_POINTS: u64 = 10000; // 100.00%

// ============================================================================
// Helper Functions
// ============================================================================

impl ReferralTier {
    pub fn from_points(points: u64) -> Self {
        match points {
            0..=999 => ReferralTier::Explorer,
            1000..=4999 => ReferralTier::Connector,
            5000..=14999 => ReferralTier::Influencer,
            15000..=49999 => ReferralTier::Leader,
            _ => ReferralTier::Ambassador,
        }
    }

    pub fn multiplier(&self) -> u64 {
        match self {
            ReferralTier::Explorer => 10000,    // 1.0x
            ReferralTier::Connector => 12000,   // 1.2x
            ReferralTier::Influencer => 15000,  // 1.5x
            ReferralTier::Leader => 20000,      // 2.0x
            ReferralTier::Ambassador => 30000,  // 3.0x
        }
    }
}

impl StakingTier {
    pub fn from_amount(amount: u64) -> Self {
        match amount {
            100_000_000..=499_000_000 => StakingTier::Bronze,    // 100-499 FIN
            500_000_000..=999_000_000 => StakingTier::Silver,    // 500-999 FIN
            1_000_000_000..=4_999_000_000 => StakingTier::Gold,  // 1K-4.999K FIN
            5_000_000_000..=9_999_000_000 => StakingTier::Platinum, // 5K-9.999K FIN
            _ => StakingTier::Diamond, // 10K+ FIN
        }
    }
}

impl MiningPhase {
    pub fn from_user_count(users: u64) -> Self {
        match users {
            0..=100_000 => MiningPhase::Finizen,
            100_001..=1_000_000 => MiningPhase::Growth,
            1_000_001..=10_000_000 => MiningPhase::Maturity,
            _ => MiningPhase::Stability,
        }
    }

    pub fn base_rate(&self) -> u64 {
        match self {
            MiningPhase::Finizen => 100_000,    // 0.1 FIN/hour
            MiningPhase::Growth => 50_000,      // 0.05 FIN/hour
            MiningPhase::Maturity => 25_000,    // 0.025 FIN/hour
            MiningPhase::Stability => 10_000,   // 0.01 FIN/hour
        }
    }
}

impl Default for UserAccount {
    fn default() -> Self {
        Self {
            authority: Pubkey::default(),
            username: String::new(),
            email: String::new(),
            kyc_verified: false,
            created_at: 0,
            last_active: 0,
            total_fin_earned: 0,
            total_fin_holdings: 0,
            xp_level: 1,
            xp_points: 0,
            rp_tier: ReferralTier::Explorer,
            rp_points: 0,
            mining_rate: 0,
            referral_code: String::new(),
            referred_by: None,
            guild_id: None,
            social_profiles: Vec::new(),
            activity_streak: 0,
            quality_score: 5000, // Default to 0.5
            human_probability: 5000, // Default to 0.5
            network_size: 0,
            active_cards: Vec::new(),
            achievements: Vec::new(),
            staking_info: StakingInfo::default(),
            bump: 0,
        }
    }
}

impl Default for StakingInfo {
    fn default() -> Self {
        Self {
            total_staked: 0,
            current_tier: StakingTier::Bronze,
            multiplier: 10000, // 1.0x
            last_reward: 0,
            claimable_rewards: 0,
        }
    }
}
