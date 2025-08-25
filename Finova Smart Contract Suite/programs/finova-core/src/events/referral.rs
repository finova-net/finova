// programs/finova-core/src/events/referral.rs

use anchor_lang::prelude::*;

/// Event emitted when a new referral relationship is established
#[event]
pub struct ReferralCreated {
    /// The public key of the referrer (who invited)
    pub referrer: Pubkey,
    /// The public key of the referee (who was invited)
    pub referee: Pubkey,
    /// The referral code used
    pub referral_code: String,
    /// Timestamp when the referral was created
    pub timestamp: i64,
    /// The tier of the referrer at the time of referral
    pub referrer_tier: u8,
    /// Initial bonus points awarded to referrer
    pub initial_bonus: u64,
}

/// Event emitted when referral points are earned
#[event]
pub struct ReferralPointsEarned {
    /// The public key of the user who earned points
    pub user: Pubkey,
    /// The public key of the referee who generated the activity
    pub referee: Pubkey,
    /// Points earned from this activity
    pub points_earned: u64,
    /// Type of activity that generated points
    pub activity_type: String,
    /// Level of the referral (L1, L2, L3)
    pub referral_level: u8,
    /// Percentage of referee's activity rewarded
    pub reward_percentage: u16,
    /// Total referral points after this earning
    pub total_rp: u64,
    /// Timestamp of the earning
    pub timestamp: i64,
}

/// Event emitted when a user's referral tier is upgraded
#[event]
pub struct ReferralTierUpgraded {
    /// The public key of the user whose tier was upgraded
    pub user: Pubkey,
    /// Previous tier (0=Explorer, 1=Connector, 2=Influencer, 3=Leader, 4=Ambassador)
    pub previous_tier: u8,
    /// New tier after upgrade
    pub new_tier: u8,
    /// Total referral points that triggered the upgrade
    pub total_rp: u64,
    /// Active referrals count at upgrade
    pub active_referrals: u32,
    /// Network size at upgrade
    pub network_size: u32,
    /// Mining bonus multiplier for new tier
    pub mining_bonus: u16,
    /// Referral reward percentage for new tier
    pub referral_percentage: u16,
    /// Timestamp of the upgrade
    pub timestamp: i64,
}

/// Event emitted when referral network statistics are updated
#[event]
pub struct ReferralNetworkUpdated {
    /// The public key of the network owner
    pub user: Pubkey,
    /// Total direct referrals (L1)
    pub direct_referrals: u32,
    /// Total indirect referrals (L2 + L3)
    pub indirect_referrals: u32,
    /// Total network size (all levels)
    pub total_network_size: u32,
    /// Active referrals in last 30 days
    pub active_referrals: u32,
    /// Network quality score (0-1000)
    pub network_quality_score: u16,
    /// Average activity level of network
    pub average_activity_level: u16,
    /// Network retention rate (percentage)
    pub retention_rate: u16,
    /// Timestamp of the update
    pub timestamp: i64,
}

/// Event emitted when referral rewards are claimed
#[event]
pub struct ReferralRewardsClaimed {
    /// The public key of the user claiming rewards
    pub user: Pubkey,
    /// Amount of $FIN tokens claimed
    pub fin_amount: u64,
    /// XP bonus claimed
    pub xp_bonus: u64,
    /// Referral points used for claim
    pub rp_used: u64,
    /// User's tier at time of claim
    pub user_tier: u8,
    /// Mining multiplier applied
    pub mining_multiplier: u16,
    /// Quality score applied
    pub quality_score: u16,
    /// Timestamp of the claim
    pub timestamp: i64,
}

/// Event emitted when referral code is generated or updated
#[event]
pub struct ReferralCodeUpdated {
    /// The public key of the user
    pub user: Pubkey,
    /// Previous referral code (empty if first time)
    pub previous_code: String,
    /// New referral code
    pub new_code: String,
    /// User's tier that allows custom codes
    pub user_tier: u8,
    /// Whether this is a custom code (true) or auto-generated (false)
    pub is_custom: bool,
    /// Timestamp of the update
    pub timestamp: i64,
}

/// Event emitted when referral network quality is assessed
#[event]
pub struct ReferralNetworkQualityAssessed {
    /// The public key of the network owner
    pub user: Pubkey,
    /// Previous quality score
    pub previous_quality_score: u16,
    /// New quality score after assessment
    pub new_quality_score: u16,
    /// Active users percentage in network
    pub active_users_percentage: u16,
    /// Average XP level of network
    pub average_xp_level: u16,
    /// Diversity score of network
    pub diversity_score: u16,
    /// Engagement quality score
    pub engagement_quality: u16,
    /// Regression factor applied
    pub regression_factor: u16,
    /// Timestamp of the assessment
    pub timestamp: i64,
}

/// Event emitted when referral bonus multiplier is applied
#[event]
pub struct ReferralBonusApplied {
    /// The public key of the user receiving bonus
    pub user: Pubkey,
    /// The referee who triggered the bonus
    pub referee: Pubkey,
    /// Base activity value before bonus
    pub base_value: u64,
    /// Bonus multiplier applied (in basis points)
    pub bonus_multiplier: u16,
    /// Final value after bonus
    pub final_value: u64,
    /// Type of activity (mining, xp, etc.)
    pub activity_type: String,
    /// Referral level that generated bonus (L1, L2, L3)
    pub referral_level: u8,
    /// User's current RP tier
    pub user_tier: u8,
    /// Timestamp of the bonus application
    pub timestamp: i64,
}

/// Event emitted when referral network milestone is achieved
#[event]
pub struct ReferralMilestoneAchieved {
    /// The public key of the user achieving milestone
    pub user: Pubkey,
    /// Type of milestone achieved
    pub milestone_type: String,
    /// Milestone value (e.g., 10, 25, 50, 100 referrals)
    pub milestone_value: u32,
    /// Bonus RP awarded for milestone
    pub bonus_rp: u64,
    /// Mining multiplier bonus awarded
    pub mining_bonus: u16,
    /// Special benefits unlocked
    pub benefits_unlocked: Vec<String>,
    /// Previous tier before milestone
    pub previous_tier: u8,
    /// New tier after milestone (if upgraded)
    pub new_tier: u8,
    /// Timestamp of the achievement
    pub timestamp: i64,
}

/// Event emitted when referral activity validation occurs
#[event]
pub struct ReferralActivityValidated {
    /// The public key of the referee whose activity was validated
    pub referee: Pubkey,
    /// The public key of the referrer receiving validation
    pub referrer: Pubkey,
    /// Activity ID being validated
    pub activity_id: String,
    /// Type of activity validated
    pub activity_type: String,
    /// Original activity value
    pub original_value: u64,
    /// Validated activity value (after quality checks)
    pub validated_value: u64,
    /// Quality score applied (0-2000 basis points)
    pub quality_score: u16,
    /// Whether activity passed anti-bot checks
    pub passed_anti_bot: bool,
    /// Validation status (approved, rejected, pending)
    pub validation_status: String,
    /// Timestamp of the validation
    pub timestamp: i64,
}

/// Event emitted when referral leaderboard is updated
#[event]
pub struct ReferralLeaderboardUpdated {
    /// The public key of the user on leaderboard
    pub user: Pubkey,
    /// Previous rank on leaderboard (0 if not ranked)
    pub previous_rank: u32,
    /// New rank on leaderboard
    pub new_rank: u32,
    /// Total referral points for ranking
    pub total_rp: u64,
    /// Active network size for ranking
    pub active_network_size: u32,
    /// Network quality score for ranking
    pub network_quality: u16,
    /// Period type (daily, weekly, monthly, all-time)
    pub period_type: String,
    /// Leaderboard category (network size, quality, earnings, etc.)
    pub category: String,
    /// Timestamp of the update
    pub timestamp: i64,
}

/// Event emitted when referral commission is distributed
#[event]
pub struct ReferralCommissionDistributed {
    /// The public key of the referrer receiving commission
    pub referrer: Pubkey,
    /// The public key of the referee generating commission
    pub referee: Pubkey,
    /// Transaction that generated the commission
    pub transaction_id: String,
    /// Base commission amount before bonuses
    pub base_commission: u64,
    /// Tier bonus applied
    pub tier_bonus: u64,
    /// Quality bonus applied
    pub quality_bonus: u64,
    /// Final commission amount distributed
    pub final_commission: u64,
    /// Commission type (mining, trading, staking, etc.)
    pub commission_type: String,
    /// Referral level (L1, L2, L3)
    pub referral_level: u8,
    /// Timestamp of the distribution
    pub timestamp: i64,
}

/// Event emitted when referral network is analyzed for fraud
#[event]
pub struct ReferralNetworkAnalyzed {
    /// The public key of the network owner being analyzed
    pub user: Pubkey,
    /// Total network size analyzed
    pub network_size: u32,
    /// Suspicious patterns detected
    pub suspicious_patterns: Vec<String>,
    /// Risk score (0-1000, higher = more risky)
    pub risk_score: u16,
    /// Confidence level of analysis (0-100%)
    pub confidence_level: u8,
    /// Actions taken (none, warning, restriction, ban)
    pub actions_taken: Vec<String>,
    /// Human probability score (0-1000)
    pub human_probability: u16,
    /// Network authenticity score (0-1000)
    pub authenticity_score: u16,
    /// Timestamp of the analysis
    pub timestamp: i64,
}

/// Event emitted when referral rewards pool is updated
#[event]
pub struct ReferralRewardsPoolUpdated {
    /// Current pool balance in $FIN
    pub pool_balance: u64,
    /// Total rewards distributed this period
    pub distributed_this_period: u64,
    /// Total rewards distributed all-time
    pub total_distributed: u64,
    /// Number of active claimants
    pub active_claimants: u32,
    /// Average reward per claimant
    pub average_reward: u64,
    /// Pool replenishment amount
    pub replenishment_amount: u64,
    /// Pool utilization rate (percentage)
    pub utilization_rate: u16,
    /// Next scheduled replenishment
    pub next_replenishment: i64,
    /// Timestamp of the update
    pub timestamp: i64,
}

/// Event emitted when referral network experiences regression
#[event]
pub struct ReferralNetworkRegression {
    /// The public key of the user experiencing regression
    pub user: Pubkey,
    /// Previous network effectiveness score
    pub previous_effectiveness: u16,
    /// New network effectiveness score
    pub new_effectiveness: u16,
    /// Regression factor applied (basis points)
    pub regression_factor: u16,
    /// Cause of regression (inactivity, quality decline, etc.)
    pub regression_cause: String,
    /// Number of inactive referrals
    pub inactive_referrals: u32,
    /// Quality score decline amount
    pub quality_decline: u16,
    /// Impact on mining multiplier
    pub mining_impact: i16,
    /// Recovery recommendations
    pub recovery_suggestions: Vec<String>,
    /// Timestamp of the regression
    pub timestamp: i64,
}

/// Event emitted when referral network diversity is calculated
#[event]
pub struct ReferralNetworkDiversityCalculated {
    /// The public key of the network owner
    pub user: Pubkey,
    /// Geographic diversity score (0-1000)
    pub geographic_diversity: u16,
    /// Platform usage diversity score (0-1000)
    pub platform_diversity: u16,
    /// Activity type diversity score (0-1000)
    pub activity_diversity: u16,
    /// Time zone diversity score (0-1000)
    pub timezone_diversity: u16,
    /// Overall diversity score (0-1000)
    pub overall_diversity: u16,
    /// Diversity bonus applied to rewards (basis points)
    pub diversity_bonus: u16,
    /// Number of unique countries in network
    pub unique_countries: u16,
    /// Number of unique platforms used
    pub unique_platforms: u16,
    /// Timestamp of the calculation
    pub timestamp: i64,
}

/// Event emitted when referral contest results are announced
#[event]
pub struct ReferralContestResults {
    /// Contest ID
    pub contest_id: String,
    /// Contest period start
    pub contest_start: i64,
    /// Contest period end
    pub contest_end: i64,
    /// Winner's public key
    pub winner: Pubkey,
    /// Winner's achievement metric
    pub winning_metric: u64,
    /// Total prize pool distributed
    pub total_prizes: u64,
    /// Winner's prize amount
    pub winner_prize: u64,
    /// Number of participants
    pub total_participants: u32,
    /// Contest category (network growth, quality, retention, etc.)
    pub contest_category: String,
    /// Next contest announcement
    pub next_contest: i64,
    /// Timestamp of results announcement
    pub timestamp: i64,
}

/// Event emitted when referral network health check is performed
#[event]
pub struct ReferralNetworkHealthCheck {
    /// The public key of the network owner
    pub user: Pubkey,
    /// Overall network health score (0-1000)
    pub health_score: u16,
    /// Growth rate (percentage)
    pub growth_rate: i16,
    /// Retention rate (percentage)
    pub retention_rate: u16,
    /// Activity consistency score (0-1000)
    pub activity_consistency: u16,
    /// Quality trend (improving, stable, declining)
    pub quality_trend: String,
    /// Engagement depth score (0-1000)
    pub engagement_depth: u16,
    /// Network resilience score (0-1000)
    pub resilience_score: u16,
    /// Health recommendations
    pub recommendations: Vec<String>,
    /// Alert level (none, low, medium, high, critical)
    pub alert_level: String,
    /// Timestamp of the health check
    pub timestamp: i64,
}

/// Event emitted when referral smart recommendations are generated
#[event]
pub struct ReferralSmartRecommendations {
    /// The public key of the user receiving recommendations
    pub user: Pubkey,
    /// Current network performance score
    pub current_performance: u16,
    /// Potential performance score with improvements
    pub potential_performance: u16,
    /// Recommended actions for network growth
    pub growth_recommendations: Vec<String>,
    /// Recommended actions for quality improvement
    pub quality_recommendations: Vec<String>,
    /// Recommended engagement strategies
    pub engagement_strategies: Vec<String>,
    /// Estimated impact of recommendations (basis points)
    pub estimated_impact: u16,
    /// Implementation difficulty (easy, medium, hard)
    pub implementation_difficulty: String,
    /// Expected timeline for results (days)
    pub expected_timeline: u16,
    /// Confidence in recommendations (0-100%)
    pub recommendation_confidence: u8,
    /// Timestamp of recommendation generation
    pub timestamp: i64,
}
