// programs/finova-token/src/events/stake.rs

use anchor_lang::prelude::*;

/// Event emitted when tokens are staked
#[event]
pub struct TokensStaked {
    /// User's public key who staked tokens
    pub user: Pubkey,
    /// Amount of tokens staked
    pub amount: u64,
    /// Total staked amount after this stake
    pub total_staked: u64,
    /// Staking tier achieved
    pub staking_tier: u8,
    /// Timestamp when tokens were staked
    pub timestamp: i64,
    /// Expected APY at time of staking
    pub apy: u16,
    /// Lock period in seconds
    pub lock_period: u64,
    /// Bonus multiplier applied
    pub bonus_multiplier: u16,
    /// Mining boost percentage
    pub mining_boost: u16,
    /// XP multiplier gained
    pub xp_multiplier: u16,
    /// RP bonus percentage
    pub rp_bonus: u16,
}

/// Event emitted when tokens are unstaked
#[event]
pub struct TokensUnstaked {
    /// User's public key who unstaked tokens
    pub user: Pubkey,
    /// Amount of tokens unstaked
    pub amount: u64,
    /// Remaining staked amount
    pub remaining_staked: u64,
    /// New staking tier after unstaking
    pub new_staking_tier: u8,
    /// Timestamp when tokens were unstaked
    pub timestamp: i64,
    /// Early unstaking penalty applied (if any)
    pub penalty_amount: u64,
    /// Reason for penalty (0: none, 1: early unstake, 2: violation)
    pub penalty_reason: u8,
    /// Final amount received after penalties
    pub received_amount: u64,
    /// Days staked before unstaking
    pub days_staked: u64,
}

/// Event emitted when staking rewards are claimed
#[event]
pub struct StakingRewardsClaimed {
    /// User's public key who claimed rewards
    pub user: Pubkey,
    /// Amount of rewards claimed
    pub reward_amount: u64,
    /// Total rewards earned to date
    pub total_rewards_earned: u64,
    /// Timestamp of claim
    pub timestamp: i64,
    /// Current staking tier
    pub staking_tier: u8,
    /// Current APY
    pub current_apy: u16,
    /// Days since last claim
    pub days_since_last_claim: u64,
    /// Bonus rewards from multipliers
    pub bonus_rewards: u64,
    /// Compound interest earned
    pub compound_interest: u64,
}

/// Event emitted when staking tier is upgraded
#[event]
pub struct StakingTierUpgraded {
    /// User's public key
    pub user: Pubkey,
    /// Previous tier
    pub previous_tier: u8,
    /// New tier achieved
    pub new_tier: u8,
    /// Total staked amount
    pub total_staked: u64,
    /// Timestamp of upgrade
    pub timestamp: i64,
    /// New APY after upgrade
    pub new_apy: u16,
    /// New mining boost percentage
    pub new_mining_boost: u16,
    /// New XP multiplier
    pub new_xp_multiplier: u16,
    /// New RP bonus
    pub new_rp_bonus: u16,
    /// Special benefits unlocked
    pub special_benefits: u64, // Bitmask for various benefits
}

/// Event emitted when auto-compounding is enabled/disabled
#[event]
pub struct AutoCompoundingToggled {
    /// User's public key
    pub user: Pubkey,
    /// Whether auto-compounding is now enabled
    pub enabled: bool,
    /// Timestamp of change
    pub timestamp: i64,
    /// Current staked amount
    pub staked_amount: u64,
    /// Expected boost from compounding
    pub compounding_boost: u16,
    /// Frequency of compounding (in seconds)
    pub compound_frequency: u64,
}

/// Event emitted when staking rewards are auto-compounded
#[event]
pub struct RewardsAutoCompounded {
    /// User's public key
    pub user: Pubkey,
    /// Amount of rewards compounded
    pub compounded_amount: u64,
    /// New total staked amount
    pub new_total_staked: u64,
    /// Timestamp of compounding
    pub timestamp: i64,
    /// Compounding efficiency percentage
    pub efficiency: u16,
    /// Additional APY from compounding
    pub compound_apy_bonus: u16,
    /// Days since last compound
    pub days_since_last_compound: u64,
}

/// Event emitted when staking parameters are updated
#[event]
pub struct StakingParametersUpdated {
    /// Authority who updated parameters
    pub authority: Pubkey,
    /// Timestamp of update
    pub timestamp: i64,
    /// New base APY
    pub new_base_apy: u16,
    /// New tier thresholds (array of 5 tier minimums)
    pub new_tier_thresholds: [u64; 5],
    /// New tier multipliers
    pub new_tier_multipliers: [u16; 5],
    /// New lock period options
    pub new_lock_periods: [u64; 4],
    /// New early unstaking penalty rate
    pub new_penalty_rate: u16,
    /// New maximum staking capacity
    pub new_max_capacity: u64,
}

/// Event emitted when emergency pause is triggered
#[event]
pub struct StakingEmergencyPause {
    /// Authority who triggered pause
    pub authority: Pubkey,
    /// Timestamp of pause
    pub timestamp: i64,
    /// Reason for pause (encoded)
    pub reason_code: u16,
    /// Total staked amount at pause
    pub total_staked_at_pause: u64,
    /// Number of active stakers
    pub active_stakers_count: u64,
    /// Expected resolution time
    pub expected_resolution: i64,
}

/// Event emitted when staking is resumed after pause
#[event]
pub struct StakingResumed {
    /// Authority who resumed staking
    pub authority: Pubkey,
    /// Timestamp of resume
    pub timestamp: i64,
    /// Duration of pause in seconds
    pub pause_duration: u64,
    /// Compensation provided to stakers
    pub compensation_rate: u16,
    /// New safety measures implemented
    pub safety_measures: u64, // Bitmask
}

/// Event emitted when yield farming rewards are distributed
#[event]
pub struct YieldFarmingRewards {
    /// Pool identifier
    pub pool_id: u64,
    /// Total rewards distributed
    pub total_rewards: u64,
    /// Number of participants
    pub participant_count: u64,
    /// Timestamp of distribution
    pub timestamp: i64,
    /// Average reward per participant
    pub avg_reward_per_participant: u64,
    /// Bonus multiplier applied
    pub bonus_multiplier: u16,
    /// Performance metrics
    pub pool_performance: u16,
}

/// Event emitted when staking goal is achieved
#[event]
pub struct StakingGoalAchieved {
    /// User who achieved goal
    pub user: Pubkey,
    /// Goal type (1: amount, 2: duration, 3: tier, 4: rewards)
    pub goal_type: u8,
    /// Goal target value
    pub goal_target: u64,
    /// Actual achieved value
    pub achieved_value: u64,
    /// Timestamp of achievement
    pub timestamp: i64,
    /// Bonus rewards for achievement
    pub bonus_reward: u64,
    /// NFT badge earned (if any)
    pub nft_badge_id: Option<Pubkey>,
    /// Achievement tier (bronze, silver, gold, etc.)
    pub achievement_tier: u8,
}

/// Event emitted when staking metrics are calculated
#[event]
pub struct StakingMetricsUpdated {
    /// Timestamp of calculation
    pub timestamp: i64,
    /// Total value locked (TVL)
    pub total_value_locked: u64,
    /// Average staking duration
    pub avg_staking_duration: u64,
    /// Total rewards distributed
    pub total_rewards_distributed: u64,
    /// Number of active stakers
    pub active_stakers: u64,
    /// Staking participation rate
    pub participation_rate: u16,
    /// APY efficiency metric
    pub apy_efficiency: u16,
    /// Network health score
    pub network_health_score: u16,
}

/// Event emitted when loyalty bonus is applied
#[event]
pub struct LoyaltyBonusApplied {
    /// User receiving bonus
    pub user: Pubkey,
    /// Loyalty level achieved
    pub loyalty_level: u8,
    /// Bonus percentage applied
    pub bonus_percentage: u16,
    /// Bonus amount received
    pub bonus_amount: u64,
    /// Timestamp of bonus application
    pub timestamp: i64,
    /// Consecutive staking days
    pub consecutive_days: u64,
    /// Total loyalty points earned
    pub total_loyalty_points: u64,
    /// Next loyalty milestone
    pub next_milestone: u64,
}

/// Event emitted when staking pool capacity is reached
#[event]
pub struct StakingCapacityReached {
    /// Pool identifier
    pub pool_id: u64,
    /// Maximum capacity
    pub max_capacity: u64,
    /// Current staked amount
    pub current_staked: u64,
    /// Timestamp when capacity was reached
    pub timestamp: i64,
    /// Waiting list count
    pub waiting_list_count: u64,
    /// Priority allocation rules
    pub priority_rules: u32,
}

/// Event emitted when unstaking cooldown period starts
#[event]
pub struct UnstakingCooldownStarted {
    /// User initiating unstaking
    pub user: Pubkey,
    /// Amount to be unstaked
    pub unstake_amount: u64,
    /// Cooldown period in seconds
    pub cooldown_period: u64,
    /// Timestamp when cooldown starts
    pub timestamp: i64,
    /// Expected completion time
    pub completion_time: i64,
    /// Penalty if cancelled early
    pub early_cancellation_penalty: u64,
    /// Rewards earned during cooldown
    pub cooldown_rewards: bool,
}

/// Event emitted for referral staking bonuses
#[event]
pub struct ReferralStakingBonus {
    /// Referrer receiving bonus
    pub referrer: Pubkey,
    /// Referee who staked
    pub referee: Pubkey,
    /// Referee's staked amount
    pub referee_stake_amount: u64,
    /// Bonus amount for referrer
    pub bonus_amount: u64,
    /// Bonus percentage applied
    pub bonus_percentage: u16,
    /// Timestamp of bonus
    pub timestamp: i64,
    /// Referral tier level
    pub referral_tier: u8,
    /// Network effect multiplier
    pub network_multiplier: u16,
}

/// Event emitted when special staking event occurs
#[event]
pub struct SpecialStakingEvent {
    /// Event type identifier
    pub event_type: u16,
    /// Event name/description hash
    pub event_name_hash: u64,
    /// Start timestamp
    pub start_time: i64,
    /// End timestamp
    pub end_time: i64,
    /// Special bonus multiplier
    pub bonus_multiplier: u16,
    /// Minimum stake requirement
    pub min_stake_requirement: u64,
    /// Maximum participants
    pub max_participants: u64,
    /// Current participant count
    pub current_participants: u64,
    /// Event rewards pool
    pub event_rewards_pool: u64,
}

/// Event emitted when staking insurance is claimed
#[event]
pub struct StakingInsuranceClaimed {
    /// User claiming insurance
    pub user: Pubkey,
    /// Insurance claim amount
    pub claim_amount: u64,
    /// Reason for claim (encoded)
    pub claim_reason: u16,
    /// Timestamp of claim
    pub timestamp: i64,
    /// Investigation period
    pub investigation_period: u64,
    /// Approval status
    pub approval_status: u8,
    /// Insurance pool remaining
    pub insurance_pool_remaining: u64,
}

/// Event emitted for staking analytics summary
#[event]
pub struct StakingAnalyticsSummary {
    /// Reporting period start
    pub period_start: i64,
    /// Reporting period end
    pub period_end: i64,
    /// New stakers in period
    pub new_stakers: u64,
    /// Stakers who unstaked
    pub unstaked_count: u64,
    /// Net staking growth
    pub net_growth: i64,
    /// Average stake size
    pub avg_stake_size: u64,
    /// Total rewards paid out
    pub total_rewards_paid: u64,
    /// Average holding period
    pub avg_holding_period: u64,
    /// Retention rate percentage
    pub retention_rate: u16,
    /// Churn rate percentage
    pub churn_rate: u16,
}

/// Event emitted when cross-chain staking bridge is used
#[event]
pub struct CrossChainStakingBridge {
    /// User initiating cross-chain stake
    pub user: Pubkey,
    /// Source chain identifier
    pub source_chain: u16,
    /// Destination chain identifier
    pub dest_chain: u16,
    /// Amount being bridged
    pub bridge_amount: u64,
    /// Bridge transaction hash
    pub bridge_tx_hash: [u8; 32],
    /// Timestamp of bridge
    pub timestamp: i64,
    /// Bridge fee paid
    pub bridge_fee: u64,
    /// Expected completion time
    pub expected_completion: i64,
}

/// Event emitted for governance voting by stakers
#[event]
pub struct StakerGovernanceVote {
    /// Voter's public key
    pub voter: Pubkey,
    /// Proposal identifier
    pub proposal_id: u64,
    /// Vote choice (0: no, 1: yes, 2: abstain)
    pub vote: u8,
    /// Voting power based on stake
    pub voting_power: u64,
    /// Timestamp of vote
    pub timestamp: i64,
    /// Stake amount at time of vote
    pub stake_at_vote: u64,
    /// Staking tier at time of vote
    pub tier_at_vote: u8,
    /// Vote weight multiplier
    pub vote_weight_multiplier: u16,
}

impl TokensStaked {
    pub fn new(
        user: Pubkey,
        amount: u64,
        total_staked: u64,
        staking_tier: u8,
        apy: u16,
        lock_period: u64,
        bonus_multiplier: u16,
        mining_boost: u16,
        xp_multiplier: u16,
        rp_bonus: u16,
    ) -> Self {
        Self {
            user,
            amount,
            total_staked,
            staking_tier,
            timestamp: Clock::get().unwrap().unix_timestamp,
            apy,
            lock_period,
            bonus_multiplier,
            mining_boost,
            xp_multiplier,
            rp_bonus,
        }
    }
}

impl TokensUnstaked {
    pub fn new(
        user: Pubkey,
        amount: u64,
        remaining_staked: u64,
        new_staking_tier: u8,
        penalty_amount: u64,
        penalty_reason: u8,
        received_amount: u64,
        days_staked: u64,
    ) -> Self {
        Self {
            user,
            amount,
            remaining_staked,
            new_staking_tier,
            timestamp: Clock::get().unwrap().unix_timestamp,
            penalty_amount,
            penalty_reason,
            received_amount,
            days_staked,
        }
    }
}

impl StakingRewardsClaimed {
    pub fn new(
        user: Pubkey,
        reward_amount: u64,
        total_rewards_earned: u64,
        staking_tier: u8,
        current_apy: u16,
        days_since_last_claim: u64,
        bonus_rewards: u64,
        compound_interest: u64,
    ) -> Self {
        Self {
            user,
            reward_amount,
            total_rewards_earned,
            timestamp: Clock::get().unwrap().unix_timestamp,
            staking_tier,
            current_apy,
            days_since_last_claim,
            bonus_rewards,
            compound_interest,
        }
    }
}

// Helper functions for event creation
pub fn emit_tokens_staked(
    user: Pubkey,
    amount: u64,
    total_staked: u64,
    staking_tier: u8,
    apy: u16,
    lock_period: u64,
    bonus_multiplier: u16,
    mining_boost: u16,
    xp_multiplier: u16,
    rp_bonus: u16,
) {
    emit!(TokensStaked::new(
        user,
        amount,
        total_staked,
        staking_tier,
        apy,
        lock_period,
        bonus_multiplier,
        mining_boost,
        xp_multiplier,
        rp_bonus
    ));
}

pub fn emit_tokens_unstaked(
    user: Pubkey,
    amount: u64,
    remaining_staked: u64,
    new_staking_tier: u8,
    penalty_amount: u64,
    penalty_reason: u8,
    received_amount: u64,
    days_staked: u64,
) {
    emit!(TokensUnstaked::new(
        user,
        amount,
        remaining_staked,
        new_staking_tier,
        penalty_amount,
        penalty_reason,
        received_amount,
        days_staked
    ));
}

pub fn emit_staking_rewards_claimed(
    user: Pubkey,
    reward_amount: u64,
    total_rewards_earned: u64,
    staking_tier: u8,
    current_apy: u16,
    days_since_last_claim: u64,
    bonus_rewards: u64,
    compound_interest: u64,
) {
    emit!(StakingRewardsClaimed::new(
        user,
        reward_amount,
        total_rewards_earned,
        staking_tier,
        current_apy,
        days_since_last_claim,
        bonus_rewards,
        compound_interest
    ));
}
