// programs/finova-core/src/events/mining.rs

use anchor_lang::prelude::*;
use crate::state::{MiningAccount, UserAccount, NetworkState};

/// Event emitted when a user starts mining
#[event]
pub struct MiningStarted {
    /// The user's public key
    pub user: Pubkey,
    /// Timestamp when mining started
    pub timestamp: i64,
    /// Initial mining rate per hour
    pub base_rate: u64,
    /// Finizen bonus multiplier (scaled by 1000)
    pub finizen_bonus: u64,
    /// Referral bonus multiplier (scaled by 1000)
    pub referral_bonus: u64,
    /// Security bonus multiplier (scaled by 1000)
    pub security_bonus: u64,
    /// Final effective mining rate
    pub effective_rate: u64,
    /// Current mining phase
    pub phase: u8,
    /// Total network users count
    pub network_users: u64,
}

/// Event emitted when mining rewards are claimed
#[event]
pub struct RewardsClaimed {
    /// The user's public key
    pub user: Pubkey,
    /// Amount of FIN tokens claimed
    pub amount: u64,
    /// Timestamp of claim
    pub timestamp: i64,
    /// Mining duration in seconds
    pub mining_duration: i64,
    /// Effective mining rate used
    pub mining_rate: u64,
    /// XP multiplier applied (scaled by 1000)
    pub xp_multiplier: u64,
    /// RP multiplier applied (scaled by 1000)
    pub rp_multiplier: u64,
    /// Quality score applied (scaled by 1000)
    pub quality_multiplier: u64,
    /// Total accumulated rewards before claim
    pub total_accumulated: u64,
    /// User's new total FIN balance
    pub new_balance: u64,
}

/// Event emitted when mining rate is updated
#[event]
pub struct MiningRateUpdated {
    /// The user's public key
    pub user: Pubkey,
    /// Previous mining rate
    pub old_rate: u64,
    /// New mining rate
    pub new_rate: u64,
    /// Reason for rate change
    pub reason: MiningRateChangeReason,
    /// Timestamp of update
    pub timestamp: i64,
    /// New regression factor applied (scaled by 1000000)
    pub regression_factor: u64,
    /// User's current holdings that triggered regression
    pub user_holdings: u64,
}

/// Event emitted when mining phase changes
#[event]
pub struct MiningPhaseChanged {
    /// Previous phase
    pub old_phase: u8,
    /// New phase
    pub new_phase: u8,
    /// Total network users that triggered phase change
    pub network_users: u64,
    /// New base mining rate for the phase
    pub new_base_rate: u64,
    /// New Finizen bonus for the phase
    pub new_finizen_bonus: u64,
    /// Timestamp of phase change
    pub timestamp: i64,
    /// Estimated users needed for next phase
    pub next_phase_threshold: u64,
}

/// Event emitted when mining boost is applied
#[event]
pub struct MiningBoostApplied {
    /// The user's public key
    pub user: Pubkey,
    /// Type of boost applied
    pub boost_type: MiningBoostType,
    /// Boost multiplier (scaled by 1000)
    pub multiplier: u64,
    /// Duration of boost in seconds
    pub duration: i64,
    /// Timestamp when boost was applied
    pub timestamp: i64,
    /// Boost expiry timestamp
    pub expires_at: i64,
    /// Source of the boost (NFT card, achievement, etc.)
    pub source: Pubkey,
}

/// Event emitted when mining boost expires
#[event]
pub struct MiningBoostExpired {
    /// The user's public key
    pub user: Pubkey,
    /// Type of boost that expired
    pub boost_type: MiningBoostType,
    /// Multiplier that was removed (scaled by 1000)
    pub multiplier: u64,
    /// Timestamp when boost expired
    pub timestamp: i64,
    /// User's new effective mining rate after boost removal
    pub new_mining_rate: u64,
}

/// Event emitted when mining is paused
#[event]
pub struct MiningPaused {
    /// The user's public key
    pub user: Pubkey,
    /// Reason for pause
    pub reason: MiningPauseReason,
    /// Timestamp when paused
    pub timestamp: i64,
    /// Accumulated rewards at time of pause
    pub accumulated_rewards: u64,
    /// Duration of mining session before pause
    pub session_duration: i64,
}

/// Event emitted when mining is resumed
#[event]
pub struct MiningResumed {
    /// The user's public key
    pub user: Pubkey,
    /// Timestamp when resumed
    pub timestamp: i64,
    /// New mining rate after resume
    pub mining_rate: u64,
    /// Duration mining was paused
    pub pause_duration: i64,
    /// Reason mining was resumed
    pub reason: String,
}

/// Event emitted when regression factor is calculated
#[event]
pub struct RegressionCalculated {
    /// The user's public key
    pub user: Pubkey,
    /// User's total FIN holdings
    pub holdings: u64,
    /// Calculated regression factor (scaled by 1000000)
    pub regression_factor: u64,
    /// Previous regression factor
    pub previous_factor: u64,
    /// Base mining rate before regression
    pub base_rate: u64,
    /// Final mining rate after regression
    pub final_rate: u64,
    /// Timestamp of calculation
    pub timestamp: i64,
}

/// Event emitted when anti-whale mechanism is triggered
#[event]
pub struct AntiWhaleMechanismTriggered {
    /// The user's public key (whale)
    pub user: Pubkey,
    /// User's total holdings that triggered mechanism
    pub holdings: u64,
    /// Whale threshold that was exceeded
    pub whale_threshold: u64,
    /// Penalty factor applied (scaled by 1000)
    pub penalty_factor: u64,
    /// Previous mining rate
    pub old_rate: u64,
    /// New mining rate after penalty
    pub new_rate: u64,
    /// Timestamp when triggered
    pub timestamp: i64,
}

/// Event emitted when daily mining cap is reached
#[event]
pub struct DailyCapReached {
    /// The user's public key
    pub user: Pubkey,
    /// Daily cap amount
    pub daily_cap: u64,
    /// Amount mined today
    pub mined_today: u64,
    /// Timestamp when cap was reached
    pub timestamp: i64,
    /// Time until cap resets (seconds)
    pub reset_in: i64,
}

/// Event emitted when mining statistics are updated
#[event]
pub struct MiningStatsUpdated {
    /// The user's public key
    pub user: Pubkey,
    /// Total FIN mined by user (all time)
    pub total_mined: u64,
    /// Total mining sessions
    pub total_sessions: u64,
    /// Average mining rate over last 30 days
    pub avg_mining_rate: u64,
    /// Total mining hours
    pub total_mining_hours: u64,
    /// Current mining streak (days)
    pub mining_streak: u32,
    /// Best mining streak (days)
    pub best_streak: u32,
    /// Timestamp of update
    pub timestamp: i64,
}

/// Event emitted when network mining metrics are updated
#[event]
pub struct NetworkMiningMetrics {
    /// Total FIN mined across network
    pub total_network_mined: u64,
    /// Total active miners in last 24h
    pub active_miners_24h: u64,
    /// Total active miners in last 7d
    pub active_miners_7d: u64,
    /// Average mining rate across network
    pub avg_network_rate: u64,
    /// Current mining phase
    pub current_phase: u8,
    /// Total registered users
    pub total_users: u64,
    /// Total KYC verified users
    pub kyc_verified_users: u64,
    /// Timestamp of metrics update
    pub timestamp: i64,
}

/// Enum for mining rate change reasons
#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub enum MiningRateChangeReason {
    /// Rate changed due to phase transition
    PhaseChange,
    /// Rate changed due to referral network growth
    ReferralGrowth,
    /// Rate changed due to XP level increase
    XpLevelUp,
    /// Rate changed due to quality score improvement
    QualityImprovement,
    /// Rate changed due to regression mechanism
    RegressionApplied,
    /// Rate changed due to boost application
    BoostApplied,
    /// Rate changed due to boost expiry
    BoostExpired,
    /// Rate changed due to anti-whale mechanism
    AntiWhalePenalty,
    /// Rate changed due to KYC verification
    KycVerified,
    /// Rate changed due to staking
    StakingBonus,
    /// Rate changed due to guild participation
    GuildBonus,
}

/// Enum for mining boost types
#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub enum MiningBoostType {
    /// Double mining rate boost
    DoubleMining,
    /// Triple mining rate boost
    TripleMining,
    /// Mining frenzy boost (5x)
    MiningFrenzy,
    /// Eternal miner boost (permanent small boost)
    EternalMiner,
    /// Daily social post boost
    DailySocialPost,
    /// Quest completion boost
    QuestCompletion,
    /// Referral KYC success boost
    ReferralKycSuccess,
    /// Special card boost
    SpecialCard,
    /// Guild participation boost
    GuildParticipation,
    /// Staking tier boost
    StakingTier,
    /// Viral content boost
    ViralContent,
    /// Loyalty streak boost
    LoyaltyStreak,
}

/// Enum for mining pause reasons
#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub enum MiningPauseReason {
    /// User manually paused mining
    UserPaused,
    /// Suspicious activity detected
    SuspiciousActivity,
    /// KYC verification required
    KycRequired,
    /// Anti-bot mechanism triggered
    AntiBotTriggered,
    /// Daily cap reached
    DailyCapReached,
    /// Account under review
    AccountReview,
    /// Technical maintenance
    Maintenance,
    /// Regulatory compliance check
    ComplianceCheck,
}

/// Helper functions for event emission
impl MiningStarted {
    pub fn emit_mining_started(
        user: Pubkey,
        mining_account: &MiningAccount,
        user_account: &UserAccount,
        network_state: &NetworkState,
    ) {
        emit!(MiningStarted {
            user,
            timestamp: Clock::get().unwrap().unix_timestamp,
            base_rate: mining_account.base_mining_rate,
            finizen_bonus: mining_account.finizen_bonus,
            referral_bonus: mining_account.referral_bonus,
            security_bonus: if user_account.kyc_verified { 1200 } else { 800 },
            effective_rate: mining_account.effective_mining_rate,
            phase: network_state.current_phase,
            network_users: network_state.total_users,
        });
    }
}

impl RewardsClaimed {
    pub fn emit_rewards_claimed(
        user: Pubkey,
        amount: u64,
        mining_account: &MiningAccount,
        user_account: &UserAccount,
        mining_duration: i64,
    ) {
        emit!(RewardsClaimed {
            user,
            amount,
            timestamp: Clock::get().unwrap().unix_timestamp,
            mining_duration,
            mining_rate: mining_account.effective_mining_rate,
            xp_multiplier: mining_account.xp_multiplier,
            rp_multiplier: mining_account.rp_multiplier,
            quality_multiplier: mining_account.quality_multiplier,
            total_accumulated: mining_account.accumulated_rewards,
            new_balance: user_account.fin_balance,
        });
    }
}

impl MiningRateUpdated {
    pub fn emit_rate_updated(
        user: Pubkey,
        old_rate: u64,
        new_rate: u64,
        reason: MiningRateChangeReason,
        regression_factor: u64,
        user_holdings: u64,
    ) {
        emit!(MiningRateUpdated {
            user,
            old_rate,
            new_rate,
            reason,
            timestamp: Clock::get().unwrap().unix_timestamp,
            regression_factor,
            user_holdings,
        });
    }
}

impl MiningPhaseChanged {
    pub fn emit_phase_changed(
        old_phase: u8,
        new_phase: u8,
        network_users: u64,
        new_base_rate: u64,
        new_finizen_bonus: u64,
        next_phase_threshold: u64,
    ) {
        emit!(MiningPhaseChanged {
            old_phase,
            new_phase,
            network_users,
            new_base_rate,
            new_finizen_bonus,
            timestamp: Clock::get().unwrap().unix_timestamp,
            next_phase_threshold,
        });
    }
}

impl MiningBoostApplied {
    pub fn emit_boost_applied(
        user: Pubkey,
        boost_type: MiningBoostType,
        multiplier: u64,
        duration: i64,
        source: Pubkey,
    ) {
        let timestamp = Clock::get().unwrap().unix_timestamp;
        emit!(MiningBoostApplied {
            user,
            boost_type,
            multiplier,
            duration,
            timestamp,
            expires_at: timestamp + duration,
            source,
        });
    }
}

impl RegressionCalculated {
    pub fn emit_regression_calculated(
        user: Pubkey,
        holdings: u64,
        regression_factor: u64,
        previous_factor: u64,
        base_rate: u64,
        final_rate: u64,
    ) {
        emit!(RegressionCalculated {
            user,
            holdings,
            regression_factor,
            previous_factor,
            base_rate,
            final_rate,
            timestamp: Clock::get().unwrap().unix_timestamp,
        });
    }
}

impl AntiWhaleMechanismTriggered {
    pub fn emit_anti_whale_triggered(
        user: Pubkey,
        holdings: u64,
        whale_threshold: u64,
        penalty_factor: u64,
        old_rate: u64,
        new_rate: u64,
    ) {
        emit!(AntiWhaleMechanismTriggered {
            user,
            holdings,
            whale_threshold,
            penalty_factor,
            old_rate,
            new_rate,
            timestamp: Clock::get().unwrap().unix_timestamp,
        });
    }
}

impl NetworkMiningMetrics {
    pub fn emit_network_metrics(
        total_network_mined: u64,
        active_miners_24h: u64,
        active_miners_7d: u64,
        avg_network_rate: u64,
        current_phase: u8,
        total_users: u64,
        kyc_verified_users: u64,
    ) {
        emit!(NetworkMiningMetrics {
            total_network_mined,
            active_miners_24h,
            active_miners_7d,
            avg_network_rate,
            current_phase,
            total_users,
            kyc_verified_users,
            timestamp: Clock::get().unwrap().unix_timestamp,
        });
    }
}

/// Mining event utilities for calculations and validations
pub mod mining_event_utils {
    use super::*;
    use crate::constants::*;
    use crate::errors::FinovaError;

    /// Calculate effective mining rate with all multipliers
    pub fn calculate_effective_mining_rate(
        base_rate: u64,
        finizen_bonus: u64,
        referral_bonus: u64,
        security_bonus: u64,
        xp_multiplier: u64,
        rp_multiplier: u64,
        quality_multiplier: u64,
        regression_factor: u64,
    ) -> Result<u64> {
        // All multipliers are scaled by 1000 except regression_factor (scaled by 1000000)
        let intermediate = base_rate
            .checked_mul(finizen_bonus)
            .ok_or(FinovaError::CalculationOverflow)?
            .checked_div(1000)
            .ok_or(FinovaError::CalculationOverflow)?;

        let intermediate = intermediate
            .checked_mul(referral_bonus)
            .ok_or(FinovaError::CalculationOverflow)?
            .checked_div(1000)
            .ok_or(FinovaError::CalculationOverflow)?;

        let intermediate = intermediate
            .checked_mul(security_bonus)
            .ok_or(FinovaError::CalculationOverflow)?
            .checked_div(1000)
            .ok_or(FinovaError::CalculationOverflow)?;

        let intermediate = intermediate
            .checked_mul(xp_multiplier)
            .ok_or(FinovaError::CalculationOverflow)?
            .checked_div(1000)
            .ok_or(FinovaError::CalculationOverflow)?;

        let intermediate = intermediate
            .checked_mul(rp_multiplier)
            .ok_or(FinovaError::CalculationOverflow)?
            .checked_div(1000)
            .ok_or(FinovaError::CalculationOverflow)?;

        let intermediate = intermediate
            .checked_mul(quality_multiplier)
            .ok_or(FinovaError::CalculationOverflow)?
            .checked_div(1000)
            .ok_or(FinovaError::CalculationOverflow)?;

        let final_rate = intermediate
            .checked_mul(regression_factor)
            .ok_or(FinovaError::CalculationOverflow)?
            .checked_div(1000000)
            .ok_or(FinovaError::CalculationOverflow)?;

        Ok(final_rate)
    }

    /// Calculate regression factor based on user holdings
    pub fn calculate_regression_factor(holdings: u64) -> u64 {
        // Exponential regression: e^(-0.001 * holdings)
        // Approximated using integer math for on-chain calculation
        // Max regression factor is 1000000 (no reduction)
        // Min regression factor approaches 0 for very large holdings
        
        if holdings == 0 {
            return 1000000; // No regression
        }

        // Use Taylor series approximation for e^(-x)
        // e^(-x) ≈ 1 - x + x²/2! - x³/3! + x⁴/4! - ...
        let x = holdings.min(10000); // Cap at 10000 to prevent overflow
        let x_scaled = x * 1000; // Scale for precision

        let term1 = 1000000;
        let term2 = x_scaled;
        let term3 = x_scaled.saturating_mul(x_scaled) / 2000;
        let term4 = x_scaled.saturating_mul(x_scaled).saturating_mul(x_scaled) / 6000000;

        let result = term1
            .saturating_sub(term2)
            .saturating_add(term3)
            .saturating_sub(term4);

        result.max(45) // Minimum regression factor (0.000045x from whitepaper)
    }

    /// Determine mining phase based on network size
    pub fn determine_mining_phase(total_users: u64) -> u8 {
        match total_users {
            0..=100_000 => PHASE_FINIZEN,
            100_001..=1_000_000 => PHASE_GROWTH,
            1_000_001..=10_000_000 => PHASE_MATURITY,
            _ => PHASE_STABILITY,
        }
    }

    /// Get base mining rate for phase
    pub fn get_phase_base_rate(phase: u8) -> u64 {
        match phase {
            PHASE_FINIZEN => BASE_MINING_RATE_FINIZEN,
            PHASE_GROWTH => BASE_MINING_RATE_GROWTH,
            PHASE_MATURITY => BASE_MINING_RATE_MATURITY,
            PHASE_STABILITY => BASE_MINING_RATE_STABILITY,
            _ => BASE_MINING_RATE_STABILITY,
        }
    }

    /// Get Finizen bonus for phase
    pub fn get_phase_finizen_bonus(phase: u8, total_users: u64) -> u64 {
        match phase {
            PHASE_FINIZEN => 2000, // 2.0x
            PHASE_GROWTH => {
                // Linear decrease from 2.0x to 1.5x
                let progress = (total_users - 100_000) * 500 / 900_000;
                2000_u64.saturating_sub(progress)
            }
            PHASE_MATURITY => 1200, // 1.2x
            PHASE_STABILITY => 1000, // 1.0x
            _ => 1000,
        }
    }

    /// Calculate daily mining cap based on phase and rate
    pub fn calculate_daily_cap(phase: u8, base_rate: u64) -> u64 {
        let hours_per_day = 24u64;
        let base_daily = base_rate * hours_per_day;
        
        match phase {
            PHASE_FINIZEN => base_daily * 2, // 4.8 FIN max (from whitepaper)
            PHASE_GROWTH => (base_daily * 18) / 10, // 1.8 FIN max
            PHASE_MATURITY => (base_daily * 72) / 100, // 0.72 FIN max
            PHASE_STABILITY => (base_daily * 24) / 100, // 0.24 FIN max
            _ => base_daily / 4,
        }
    }

    /// Validate mining boost parameters
    pub fn validate_boost_parameters(
        boost_type: &MiningBoostType,
        multiplier: u64,
        duration: i64,
    ) -> Result<()> {
        // Validate multiplier ranges
        match boost_type {
            MiningBoostType::DoubleMining => {
                require!(multiplier == 2000, FinovaError::InvalidBoostMultiplier);
                require!(duration == 86400, FinovaError::InvalidBoostDuration); // 24 hours
            }
            MiningBoostType::TripleMining => {
                require!(multiplier == 3000, FinovaError::InvalidBoostMultiplier);
                require!(duration == 43200, FinovaError::InvalidBoostDuration); // 12 hours
            }
            MiningBoostType::MiningFrenzy => {
                require!(multiplier == 6000, FinovaError::InvalidBoostMultiplier);
                require!(duration == 14400, FinovaError::InvalidBoostDuration); // 4 hours
            }
            MiningBoostType::EternalMiner => {
                require!(multiplier == 1500, FinovaError::InvalidBoostMultiplier);
                require!(duration == 2592000, FinovaError::InvalidBoostDuration); // 30 days
            }
            MiningBoostType::DailySocialPost => {
                require!(multiplier <= 1200, FinovaError::InvalidBoostMultiplier);
                require!(duration == 86400, FinovaError::InvalidBoostDuration); // 24 hours
            }
            _ => {
                require!(multiplier >= 1000 && multiplier <= 10000, FinovaError::InvalidBoostMultiplier);
                require!(duration > 0 && duration <= 2592000, FinovaError::InvalidBoostDuration);
            }
        }

        Ok(())
    }

    /// Check if user has reached daily mining cap
    pub fn check_daily_cap(
        mined_today: u64,
        daily_cap: u64,
        additional_amount: u64,
    ) -> bool {
        mined_today + additional_amount <= daily_cap
    }

    /// Calculate time until daily cap reset
    pub fn time_until_cap_reset() -> i64 {
        let clock = Clock::get().unwrap();
        let current_timestamp = clock.unix_timestamp;
        let seconds_in_day = 86400i64;
        
        // Calculate seconds until next UTC midnight
        let seconds_since_midnight = current_timestamp % seconds_in_day;
        seconds_in_day - seconds_since_midnight
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::mining_event_utils::*;

    #[test]
    fn test_regression_factor_calculation() {
        // Test cases from whitepaper
        assert_eq!(calculate_regression_factor(0), 1000000); // No holdings = no regression
        assert_eq!(calculate_regression_factor(10000), 45); // High holdings = max regression
        
        // Test intermediate values
        let factor_1000 = calculate_regression_factor(1000);
        let factor_5000 = calculate_regression_factor(5000);
        assert!(factor_1000 > factor_5000); // More holdings = more regression
    }

    #[test]
    fn test_phase_determination() {
        assert_eq!(determine_mining_phase(50000), PHASE_FINIZEN);
        assert_eq!(determine_mining_phase(500000), PHASE_GROWTH);
        assert_eq!(determine_mining_phase(5000000), PHASE_MATURITY);
        assert_eq!(determine_mining_phase(50000000), PHASE_STABILITY);
    }

    #[test]
    fn test_effective_mining_rate() {
        let rate = calculate_effective_mining_rate(
            100000, // 0.1 FIN base rate (scaled by 1000000)
            2000,   // 2.0x Finizen bonus
            1000,   // 1.0x referral bonus
            1200,   // 1.2x security bonus (KYC)
            1000,   // 1.0x XP multiplier
            1000,   // 1.0x RP multiplier
            1000,   // 1.0x quality multiplier
            1000000 // 1.0x regression factor
        ).unwrap();
        
        // Expected: 100000 * 2.0 * 1.0 * 1.2 * 1.0 * 1.0 * 1.0 * 1.0 = 240000
        assert_eq!(rate, 240000);
    }
}
