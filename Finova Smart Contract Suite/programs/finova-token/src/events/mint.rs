// programs/finova-token/src/events/mint.rs 

use anchor_lang::prelude::*;
use crate::constants::*;

/// Event emitted when FIN tokens are minted through mining rewards
#[event]
pub struct TokensMinted {
    /// The user's public key who received the minted tokens
    pub user: Pubkey,
    /// Amount of tokens minted (in lamports, 9 decimals)
    pub amount: u64,
    /// Mining rate at the time of minting (scaled by 1e9)
    pub mining_rate: u64,
    /// Current mining phase (1-4)
    pub mining_phase: u8,
    /// User's current XP level
    pub xp_level: u32,
    /// User's referral tier
    pub referral_tier: u8,
    /// Quality score multiplier applied (scaled by 1000)
    pub quality_score: u16,
    /// Network regression factor applied (scaled by 1e9)
    pub regression_factor: u64,
    /// Total tokens in circulation after minting
    pub total_supply: u64,
    /// Timestamp of the minting event
    pub timestamp: i64,
    /// Mining session ID for tracking
    pub session_id: u64,
    /// Source of the mint (mining, referral bonus, etc.)
    pub mint_source: MintSource,
}

/// Event emitted when tokens are minted for referral rewards
#[event]
pub struct ReferralRewardMinted {
    /// The referrer who earned the reward
    pub referrer: Pubkey,
    /// The referee who triggered the reward
    pub referee: Pubkey,
    /// Amount of referral reward minted
    pub reward_amount: u64,
    /// Referral level (L1, L2, L3)
    pub referral_level: u8,
    /// Referrer's current RP tier
    pub referrer_tier: u8,
    /// Base reward percentage (scaled by 10000 for precision)
    pub reward_percentage: u16,
    /// Network quality bonus applied (scaled by 1000)
    pub quality_bonus: u16,
    /// Total referral rewards earned by referrer
    pub total_referral_rewards: u64,
    /// Timestamp of the reward minting
    pub timestamp: i64,
}

/// Event emitted when staking rewards are minted
#[event]
pub struct StakingRewardMinted {
    /// The staker who earned the reward
    pub staker: Pubkey,
    /// Amount of staking reward minted
    pub reward_amount: u64,
    /// Amount of tokens staked
    pub staked_amount: u64,
    /// Staking duration in seconds
    pub staking_duration: u64,
    /// APY rate applied (scaled by 10000 for precision)
    pub apy_rate: u16,
    /// XP level multiplier applied (scaled by 1000)
    pub xp_multiplier: u16,
    /// RP tier multiplier applied (scaled by 1000)
    pub rp_multiplier: u16,
    /// Loyalty bonus applied (scaled by 1000)
    pub loyalty_bonus: u16,
    /// Activity bonus applied (scaled by 1000)
    pub activity_bonus: u16,
    /// Total staking rewards earned by user
    pub total_staking_rewards: u64,
    /// Timestamp of the reward minting
    pub timestamp: i64,
}

/// Event emitted when special event bonus tokens are minted
#[event]
pub struct SpecialEventMinted {
    /// The user who received the bonus
    pub user: Pubkey,
    /// Amount of bonus tokens minted
    pub bonus_amount: u64,
    /// Type of special event
    pub event_type: SpecialEventType,
    /// Event ID for tracking
    pub event_id: u64,
    /// Multiplier applied for the event (scaled by 1000)
    pub event_multiplier: u16,
    /// User's achievement level that triggered the bonus
    pub achievement_level: u32,
    /// Additional context data for the event
    pub context_data: [u8; 32],
    /// Timestamp of the bonus minting
    pub timestamp: i64,
}

/// Event emitted when tokens are minted for guild rewards
#[event]
pub struct GuildRewardMinted {
    /// The guild that earned the reward
    pub guild: Pubkey,
    /// The user who contributed to the guild reward
    pub user: Pubkey,
    /// Amount of guild reward minted
    pub reward_amount: u64,
    /// Guild competition type
    pub competition_type: GuildCompetitionType,
    /// Guild's ranking in the competition
    pub guild_ranking: u16,
    /// User's contribution percentage (scaled by 10000)
    pub contribution_percentage: u16,
    /// Guild performance multiplier (scaled by 1000)
    pub guild_multiplier: u16,
    /// Competition reward pool size
    pub total_reward_pool: u64,
    /// Timestamp of the reward minting
    pub timestamp: i64,
}

/// Event emitted when tokens are minted for content quality rewards
#[event]
pub struct QualityRewardMinted {
    /// The creator who earned the quality reward
    pub creator: Pubkey,
    /// Amount of quality reward minted
    pub reward_amount: u64,
    /// Content ID that earned the reward
    pub content_id: [u8; 32],
    /// Platform where content was posted
    pub platform: SocialPlatform,
    /// AI quality score (scaled by 1000)
    pub quality_score: u16,
    /// Engagement metrics score (scaled by 1000)
    pub engagement_score: u16,
    /// Viral multiplier applied (scaled by 1000)
    pub viral_multiplier: u16,
    /// Number of views/interactions
    pub interaction_count: u64,
    /// Time since content creation
    pub content_age: u64,
    /// Timestamp of the reward minting
    pub timestamp: i64,
}

/// Event emitted when emergency mint is executed
#[event]
pub struct EmergencyMint {
    /// Authority who executed the emergency mint
    pub authority: Pubkey,
    /// Recipient of the emergency minted tokens
    pub recipient: Pubkey,
    /// Amount of tokens emergency minted
    pub amount: u64,
    /// Reason code for the emergency mint
    pub reason_code: EmergencyMintReason,
    /// Additional context for the emergency mint
    pub context: [u8; 64],
    /// Multisig approval count
    pub approval_count: u8,
    /// Required approvals for this operation
    pub required_approvals: u8,
    /// Timestamp of the emergency mint
    pub timestamp: i64,
}

/// Event emitted when batch minting is completed
#[event]
pub struct BatchMintCompleted {
    /// Authority who initiated the batch mint
    pub authority: Pubkey,
    /// Number of users in the batch
    pub user_count: u32,
    /// Total amount minted in the batch
    pub total_minted: u64,
    /// Batch processing duration in milliseconds
    pub processing_duration: u64,
    /// Average mint amount per user
    pub average_per_user: u64,
    /// Batch ID for tracking
    pub batch_id: u64,
    /// Success rate (scaled by 10000)
    pub success_rate: u16,
    /// Failed mint count
    pub failed_count: u32,
    /// Timestamp when batch completed
    pub timestamp: i64,
}

/// Event emitted when anti-bot penalty reduces mint amount
#[event]
pub struct AntiBotPenaltyApplied {
    /// User who received the penalty
    pub user: Pubkey,
    /// Original mint amount before penalty
    pub original_amount: u64,
    /// Final mint amount after penalty
    pub penalized_amount: u64,
    /// Penalty percentage applied (scaled by 10000)
    pub penalty_percentage: u16,
    /// Suspicious activity score (scaled by 1000)
    pub suspicious_score: u16,
    /// Human probability score (scaled by 1000)
    pub human_score: u16,
    /// Penalty reason flags
    pub penalty_flags: u32,
    /// Number of penalties applied to this user
    pub penalty_count: u32,
    /// Timestamp of the penalty application
    pub timestamp: i64,
}

/// Event emitted when daily mint cap is reached
#[event]
pub struct DailyCapReached {
    /// User who reached the daily cap
    pub user: Pubkey,
    /// Daily cap amount
    pub daily_cap: u64,
    /// Amount that would have been minted
    pub attempted_amount: u64,
    /// Current user level
    pub user_level: u32,
    /// Cap reset timestamp
    pub reset_timestamp: i64,
    /// Consecutive days at cap
    pub consecutive_cap_days: u16,
    /// Timestamp when cap was reached
    pub timestamp: i64,
}

/// Enumeration of mint sources
#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub enum MintSource {
    /// Regular mining activity
    Mining,
    /// Referral bonus reward
    ReferralBonus,
    /// Staking reward
    StakingReward,
    /// Special event bonus
    SpecialEvent,
    /// Guild reward
    GuildReward,
    /// Quality content reward
    QualityReward,
    /// Emergency mint
    Emergency,
    /// Batch processing
    BatchMint,
    /// Developer reward
    Developer,
    /// Community reward
    Community,
}

/// Enumeration of special event types
#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub enum SpecialEventType {
    /// First 1000 users bonus
    EarlyAdopter,
    /// Daily login streak milestone
    LoginStreak,
    /// Viral content achievement
    ViralContent,
    /// Platform integration milestone
    PlatformMilestone,
    /// KYC completion bonus
    KycCompletion,
    /// Community challenge reward
    CommunityChallenge,
    /// Seasonal event bonus
    SeasonalEvent,
    /// Achievement unlock bonus
    AchievementUnlock,
    /// Network milestone bonus
    NetworkMilestone,
    /// Partnership bonus
    Partnership,
}

/// Enumeration of guild competition types
#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub enum GuildCompetitionType {
    /// Daily challenges
    DailyChallenge,
    /// Weekly wars
    WeeklyWar,
    /// Monthly championships
    MonthlyChampionship,
    /// Seasonal leagues
    SeasonalLeague,
    /// Special tournaments
    SpecialTournament,
    /// Cross-guild events
    CrossGuildEvent,
}

/// Enumeration of social platforms
#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub enum SocialPlatform {
    /// Instagram platform
    Instagram,
    /// TikTok platform
    TikTok,
    /// YouTube platform
    YouTube,
    /// Facebook platform
    Facebook,
    /// Twitter/X platform
    TwitterX,
    /// LinkedIn platform
    LinkedIn,
    /// Discord platform
    Discord,
    /// Telegram platform
    Telegram,
    /// Custom platform
    Custom,
}

/// Enumeration of emergency mint reasons
#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub enum EmergencyMintReason {
    /// System error compensation
    SystemError,
    /// Mining calculation error
    CalculationError,
    /// Network congestion compensation
    NetworkCongestion,
    /// Bug fix compensation
    BugFix,
    /// Governance decision
    GovernanceDecision,
    /// Security incident recovery
    SecurityRecovery,
    /// Migration compensation
    Migration,
    /// Partnership agreement
    Partnership,
    /// Community vote result
    CommunityVote,
    /// Developer compensation
    Developer,
}

/// Helper struct for mint calculation details
#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct MintCalculationDetails {
    /// Base mining rate (scaled by 1e9)
    pub base_rate: u64,
    /// Pioneer bonus multiplier (scaled by 1000)
    pub pioneer_bonus: u16,
    /// Referral bonus multiplier (scaled by 1000)
    pub referral_bonus: u16,
    /// Security bonus multiplier (scaled by 1000)
    pub security_bonus: u16,
    /// XP level bonus multiplier (scaled by 1000)
    pub xp_bonus: u16,
    /// Quality score multiplier (scaled by 1000)
    pub quality_multiplier: u16,
    /// Network regression factor (scaled by 1e9)
    pub regression_factor: u64,
    /// Final calculated amount
    pub final_amount: u64,
}

/// Helper struct for tracking mint statistics
#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct MintStatistics {
    /// Total tokens minted today
    pub daily_minted: u64,
    /// Total tokens minted this week
    pub weekly_minted: u64,
    /// Total tokens minted this month
    pub monthly_minted: u64,
    /// Average daily mint rate
    pub avg_daily_rate: u64,
    /// Peak mining hour
    pub peak_hour: u8,
    /// Total unique miners today
    pub unique_miners: u32,
}

impl MintSource {
    /// Check if mint source is from user activity
    pub fn is_user_activity(&self) -> bool {
        matches!(
            self,
            MintSource::Mining 
            | MintSource::ReferralBonus 
            | MintSource::QualityReward
            | MintSource::GuildReward
        )
    }

    /// Check if mint source is from rewards
    pub fn is_reward(&self) -> bool {
        matches!(
            self,
            MintSource::StakingReward 
            | MintSource::SpecialEvent 
            | MintSource::Community
        )
    }

    /// Check if mint source is administrative
    pub fn is_administrative(&self) -> bool {
        matches!(
            self,
            MintSource::Emergency 
            | MintSource::BatchMint 
            | MintSource::Developer
        )
    }
}

impl SpecialEventType {
    /// Get the default multiplier for this event type
    pub fn default_multiplier(&self) -> u16 {
        match self {
            SpecialEventType::EarlyAdopter => 2000,      // 2.0x
            SpecialEventType::LoginStreak => 1500,       // 1.5x
            SpecialEventType::ViralContent => 3000,      // 3.0x
            SpecialEventType::PlatformMilestone => 1200, // 1.2x
            SpecialEventType::KycCompletion => 1300,     // 1.3x
            SpecialEventType::CommunityChallenge => 1800, // 1.8x
            SpecialEventType::SeasonalEvent => 2500,     // 2.5x
            SpecialEventType::AchievementUnlock => 1400, // 1.4x
            SpecialEventType::NetworkMilestone => 2200,  // 2.2x
            SpecialEventType::Partnership => 1600,       // 1.6x
        }
    }

    /// Check if event is time-limited
    pub fn is_time_limited(&self) -> bool {
        matches!(
            self,
            SpecialEventType::EarlyAdopter 
            | SpecialEventType::SeasonalEvent 
            | SpecialEventType::CommunityChallenge
            | SpecialEventType::Partnership
        )
    }
}

impl SocialPlatform {
    /// Get the platform multiplier for content quality scoring
    pub fn quality_multiplier(&self) -> u16 {
        match self {
            SocialPlatform::Instagram => 1200, // 1.2x
            SocialPlatform::TikTok => 1300,    // 1.3x
            SocialPlatform::YouTube => 1400,   // 1.4x
            SocialPlatform::Facebook => 1100,  // 1.1x
            SocialPlatform::TwitterX => 1200,  // 1.2x
            SocialPlatform::LinkedIn => 1150,  // 1.15x
            SocialPlatform::Discord => 1050,   // 1.05x
            SocialPlatform::Telegram => 1050,  // 1.05x
            SocialPlatform::Custom => 1000,    // 1.0x
        }
    }

    /// Get the minimum interaction threshold for quality rewards
    pub fn min_interaction_threshold(&self) -> u64 {
        match self {
            SocialPlatform::Instagram => 100,
            SocialPlatform::TikTok => 1000,
            SocialPlatform::YouTube => 500,
            SocialPlatform::Facebook => 50,
            SocialPlatform::TwitterX => 10,
            SocialPlatform::LinkedIn => 25,
            SocialPlatform::Discord => 5,
            SocialPlatform::Telegram => 10,
            SocialPlatform::Custom => 10,
        }
    }
}

impl EmergencyMintReason {
    /// Check if reason requires multisig approval
    pub fn requires_multisig(&self) -> bool {
        matches!(
            self,
            EmergencyMintReason::SecurityRecovery 
            | EmergencyMintReason::GovernanceDecision
            | EmergencyMintReason::Migration
            | EmergencyMintReason::Partnership
        )
    }

    /// Get required approval count for this reason
    pub fn required_approvals(&self) -> u8 {
        match self {
            EmergencyMintReason::SystemError => 2,
            EmergencyMintReason::CalculationError => 2,
            EmergencyMintReason::NetworkCongestion => 1,
            EmergencyMintReason::BugFix => 2,
            EmergencyMintReason::GovernanceDecision => 5,
            EmergencyMintReason::SecurityRecovery => 4,
            EmergencyMintReason::Migration => 3,
            EmergencyMintReason::Partnership => 3,
            EmergencyMintReason::CommunityVote => 1,
            EmergencyMintReason::Developer => 2,
        }
    }
}

impl MintCalculationDetails {
    /// Calculate the effective multiplier from all bonuses
    pub fn effective_multiplier(&self) -> u64 {
        let multiplier = (self.pioneer_bonus as u64 * self.referral_bonus as u64 
            * self.security_bonus as u64 * self.xp_bonus as u64 
            * self.quality_multiplier as u64) / (1000_u64.pow(4));
        
        // Apply regression factor
        (multiplier * self.regression_factor) / 1_000_000_000
    }

    /// Validate calculation parameters
    pub fn is_valid(&self) -> bool {
        self.base_rate > 0 
            && self.pioneer_bonus >= 500 && self.pioneer_bonus <= 3000
            && self.referral_bonus >= 1000 && self.referral_bonus <= 5000
            && self.security_bonus >= 800 && self.security_bonus <= 1500
            && self.xp_bonus >= 1000 && self.xp_bonus <= 6000
            && self.quality_multiplier >= 500 && self.quality_multiplier <= 2500
            && self.regression_factor <= 1_000_000_000
    }
}

/// Event emission helper functions
impl TokensMinted {
    /// Create a new TokensMinted event
    pub fn new(
        user: Pubkey,
        amount: u64,
        mining_details: &MintCalculationDetails,
        user_stats: (u32, u8), // (xp_level, referral_tier)
        network_stats: (u64, u8, u64), // (total_supply, mining_phase, session_id)
        timestamp: i64,
        mint_source: MintSource,
    ) -> Self {
        Self {
            user,
            amount,
            mining_rate: mining_details.base_rate,
            mining_phase: network_stats.1,
            xp_level: user_stats.0,
            referral_tier: user_stats.1,
            quality_score: mining_details.quality_multiplier,
            regression_factor: mining_details.regression_factor,
            total_supply: network_stats.0,
            timestamp,
            session_id: network_stats.2,
            mint_source,
        }
    }
}

/// Constants for mint event validation
pub const MAX_MINT_AMOUNT: u64 = 1_000_000 * PRECISION; // 1M FIN max single mint
pub const MIN_MINT_AMOUNT: u64 = 1 * PRECISION / 1000; // 0.001 FIN min single mint
pub const MAX_DAILY_MINTS_PER_USER: u32 = 24 * 60; // One per minute maximum
pub const MAX_EMERGENCY_MINT: u64 = 10_000_000 * PRECISION; // 10M FIN emergency limit

/// Event validation functions
pub fn validate_mint_amount(amount: u64) -> bool {
    amount >= MIN_MINT_AMOUNT && amount <= MAX_MINT_AMOUNT
}

pub fn validate_emergency_mint(amount: u64, reason: &EmergencyMintReason) -> bool {
    match reason {
        EmergencyMintReason::SystemError => amount <= MAX_EMERGENCY_MINT / 10,
        EmergencyMintReason::SecurityRecovery => amount <= MAX_EMERGENCY_MINT,
        EmergencyMintReason::Migration => amount <= MAX_EMERGENCY_MINT * 5,
        _ => amount <= MAX_EMERGENCY_MINT / 2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mint_source_classification() {
        assert!(MintSource::Mining.is_user_activity());
        assert!(MintSource::StakingReward.is_reward());
        assert!(MintSource::Emergency.is_administrative());
    }

    #[test]
    fn test_special_event_multipliers() {
        assert_eq!(SpecialEventType::EarlyAdopter.default_multiplier(), 2000);
        assert_eq!(SpecialEventType::ViralContent.default_multiplier(), 3000);
        assert!(SpecialEventType::SeasonalEvent.is_time_limited());
    }

    #[test]
    fn test_platform_multipliers() {
        assert_eq!(SocialPlatform::TikTok.quality_multiplier(), 1300);
        assert_eq!(SocialPlatform::YouTube.min_interaction_threshold(), 500);
    }

    #[test]
    fn test_emergency_mint_approvals() {
        assert!(EmergencyMintReason::SecurityRecovery.requires_multisig());
        assert_eq!(EmergencyMintReason::GovernanceDecision.required_approvals(), 5);
    }

    #[test]
    fn test_mint_calculation_validation() {
        let details = MintCalculationDetails {
            base_rate: 100_000_000, // 0.1 FIN
            pioneer_bonus: 1500,
            referral_bonus: 2000,
            security_bonus: 1200,
            xp_bonus: 1800,
            quality_multiplier: 1300,
            regression_factor: 800_000_000,
            final_amount: 500_000_000,
        };
        assert!(details.is_valid());
    }

    #[test]
    fn test_mint_amount_validation() {
        assert!(validate_mint_amount(1_000_000_000)); // 1 FIN
        assert!(!validate_mint_amount(0)); // Too small
        assert!(!validate_mint_amount(2_000_000_000_000_000_000)); // Too large
    }
}
