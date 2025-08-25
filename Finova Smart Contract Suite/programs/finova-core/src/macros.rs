// programs/finova-core/src/macros.rs

//! Finova Core - Macros Module
//! 
//! This module contains reusable macros for common operations throughout
//! the Finova Core smart contract program, including validation, calculation,
//! state management, and security checks.

use anchor_lang::prelude::*;
use crate::errors::FinovaError;

/// Macro for validating time-based conditions with security checks
#[macro_export]
macro_rules! validate_time_window {
    ($current_time:expr, $last_action_time:expr, $cooldown_period:expr) => {
        if $current_time < $last_action_time.saturating_add($cooldown_period) {
            return Err(FinovaError::CooldownPeriodActive.into());
        }
    };
    
    ($current_time:expr, $last_action_time:expr, $cooldown_period:expr, $max_time_window:expr) => {
        if $current_time < $last_action_time.saturating_add($cooldown_period) {
            return Err(FinovaError::CooldownPeriodActive.into());
        }
        if $current_time > $last_action_time.saturating_add($max_time_window) {
            return Err(FinovaError::ActionWindowExpired.into());
        }
    };
}

/// Macro for safe mathematical operations with overflow protection
#[macro_export]
macro_rules! safe_math {
    (add $a:expr, $b:expr) => {
        $a.checked_add($b).ok_or(FinovaError::MathOverflow)?
    };
    
    (sub $a:expr, $b:expr) => {
        $a.checked_sub($b).ok_or(FinovaError::MathUnderflow)?
    };
    
    (mul $a:expr, $b:expr) => {
        $a.checked_mul($b).ok_or(FinovaError::MathOverflow)?
    };
    
    (div $a:expr, $b:expr) => {
        if $b == 0 {
            return Err(FinovaError::DivisionByZero.into());
        }
        $a.checked_div($b).ok_or(FinovaError::MathOverflow)?
    };
    
    (pow $base:expr, $exp:expr) => {
        $base.checked_pow($exp).ok_or(FinovaError::MathOverflow)?
    };
}

/// Macro for calculating exponential regression with safety checks
#[macro_export]
macro_rules! calculate_regression {
    ($base_value:expr, $regression_factor:expr, $holdings_amount:expr) => {{
        let factor = if $holdings_amount == 0 {
            1.0
        } else {
            let exp_value = -$regression_factor * ($holdings_amount as f64);
            if exp_value < -100.0 {
                0.0001 // Minimum regression value
            } else {
                exp_value.exp().max(0.0001).min(1.0)
            }
        };
        
        let result = ($base_value as f64 * factor) as u64;
        result.max(1) // Ensure minimum return of 1
    }};
}

/// Macro for validating mining rate calculations
#[macro_export]
macro_rules! validate_mining_rate {
    ($rate:expr, $phase:expr) => {{
        use crate::constants::*;
        
        let max_rate = match $phase {
            1 => PHASE_1_MAX_MINING_RATE,
            2 => PHASE_2_MAX_MINING_RATE,
            3 => PHASE_3_MAX_MINING_RATE,
            4 => PHASE_4_MAX_MINING_RATE,
            _ => PHASE_4_MAX_MINING_RATE,
        };
        
        if $rate > max_rate {
            return Err(FinovaError::InvalidMiningRate.into());
        }
        
        if $rate == 0 {
            return Err(FinovaError::ZeroMiningRate.into());
        }
        
        $rate
    }};
}

/// Macro for XP level progression calculations
#[macro_export]
macro_rules! calculate_xp_level {
    ($total_xp:expr) => {{
        use crate::constants::*;
        
        if $total_xp < XP_LEVEL_BRONZE_THRESHOLD {
            ($total_xp / XP_PER_LEVEL_BASE).max(1) as u16
        } else if $total_xp < XP_LEVEL_SILVER_THRESHOLD {
            let bronze_levels = XP_LEVEL_BRONZE_THRESHOLD / XP_PER_LEVEL_BASE;
            let remaining_xp = $total_xp - XP_LEVEL_BRONZE_THRESHOLD;
            bronze_levels + (remaining_xp / (XP_PER_LEVEL_BASE * 2))
        } else if $total_xp < XP_LEVEL_GOLD_THRESHOLD {
            let prev_levels = (XP_LEVEL_BRONZE_THRESHOLD / XP_PER_LEVEL_BASE) + 
                             ((XP_LEVEL_SILVER_THRESHOLD - XP_LEVEL_BRONZE_THRESHOLD) / (XP_PER_LEVEL_BASE * 2));
            let remaining_xp = $total_xp - XP_LEVEL_SILVER_THRESHOLD;
            prev_levels + (remaining_xp / (XP_PER_LEVEL_BASE * 3))
        } else {
            let prev_levels = (XP_LEVEL_BRONZE_THRESHOLD / XP_PER_LEVEL_BASE) + 
                             ((XP_LEVEL_SILVER_THRESHOLD - XP_LEVEL_BRONZE_THRESHOLD) / (XP_PER_LEVEL_BASE * 2)) +
                             ((XP_LEVEL_GOLD_THRESHOLD - XP_LEVEL_SILVER_THRESHOLD) / (XP_PER_LEVEL_BASE * 3));
            let remaining_xp = $total_xp - XP_LEVEL_GOLD_THRESHOLD;
            prev_levels + (remaining_xp / (XP_PER_LEVEL_BASE * 4))
        }.min(MAX_XP_LEVEL as u64) as u16
    }};
}

/// Macro for calculating XP multipliers based on level
#[macro_export]
macro_rules! calculate_xp_multiplier {
    ($level:expr) => {{
        use crate::constants::*;
        
        let multiplier = if $level <= 10 {
            BASE_XP_MULTIPLIER + ($level as f64 * 0.02)
        } else if $level <= 25 {
            BASE_XP_MULTIPLIER + 0.2 + (($level - 10) as f64 * 0.03)
        } else if $level <= 50 {
            BASE_XP_MULTIPLIER + 0.65 + (($level - 25) as f64 * 0.04)
        } else if $level <= 75 {
            BASE_XP_MULTIPLIER + 1.65 + (($level - 50) as f64 * 0.05)
        } else if $level <= 100 {
            BASE_XP_MULTIPLIER + 2.9 + (($level - 75) as f64 * 0.06)
        } else {
            MAX_XP_MULTIPLIER
        };
        
        multiplier.min(MAX_XP_MULTIPLIER)
    }};
}

/// Macro for referral points tier calculation
#[macro_export]
macro_rules! calculate_rp_tier {
    ($total_rp:expr) => {{
        use crate::constants::*;
        
        if $total_rp < RP_TIER_CONNECTOR_THRESHOLD {
            0 // Explorer
        } else if $total_rp < RP_TIER_INFLUENCER_THRESHOLD {
            1 // Connector
        } else if $total_rp < RP_TIER_LEADER_THRESHOLD {
            2 // Influencer
        } else if $total_rp < RP_TIER_AMBASSADOR_THRESHOLD {
            3 // Leader
        } else {
            4 // Ambassador
        }
    }};
}

/// Macro for calculating referral network bonus
#[macro_export]
macro_rules! calculate_referral_bonus {
    ($active_referrals:expr, $rp_tier:expr) => {{
        let base_bonus = match $rp_tier {
            0 => 0.10, // Explorer: 10% of L1
            1 => 0.15, // Connector: 15% of L1, 5% of L2
            2 => 0.20, // Influencer: 20% of L1, 8% of L2, 3% of L3
            3 => 0.25, // Leader: 25% of L1, 10% of L2, 5% of L3
            4 => 0.30, // Ambassador: 30% of L1, 15% of L2, 8% of L3
            _ => 0.10,
        };
        
        let network_multiplier = 1.0 + ($active_referrals as f64 * 0.1).min(3.0);
        base_bonus * network_multiplier
    }};
}

/// Macro for anti-bot validation checks
#[macro_export]
macro_rules! validate_human_behavior {
    ($user_state:expr, $current_time:expr) => {{
        use crate::constants::*;
        
        // Check minimum time between actions
        if $current_time < $user_state.last_action_time.saturating_add(MIN_ACTION_INTERVAL) {
            return Err(FinovaError::ActionTooFrequent.into());
        }
        
        // Check daily action limits
        let daily_reset_time = $current_time - ($current_time % SECONDS_PER_DAY);
        if $user_state.daily_reset_time != daily_reset_time {
            $user_state.daily_actions = 0;
            $user_state.daily_reset_time = daily_reset_time;
        }
        
        if $user_state.daily_actions >= MAX_DAILY_ACTIONS {
            return Err(FinovaError::DailyLimitExceeded.into());
        }
        
        // Check suspicious behavior patterns
        if $user_state.suspicious_activity_score > MAX_SUSPICIOUS_SCORE {
            return Err(FinovaError::SuspiciousActivity.into());
        }
    }};
}

/// Macro for updating user activity metrics
#[macro_export]
macro_rules! update_activity_metrics {
    ($user_state:expr, $current_time:expr, $action_type:expr) => {{
        $user_state.last_action_time = $current_time;
        $user_state.daily_actions = safe_math!(add $user_state.daily_actions, 1);
        $user_state.total_actions = safe_math!(add $user_state.total_actions, 1);
        
        // Update streak
        let hours_since_last = ($current_time - $user_state.last_daily_checkin) / 3600;
        if hours_since_last >= 24 && hours_since_last <= 48 {
            $user_state.streak_days = safe_math!(add $user_state.streak_days, 1);
        } else if hours_since_last > 48 {
            $user_state.streak_days = 1;
        }
        
        // Update activity type counter
        match $action_type {
            0 => $user_state.mining_actions = safe_math!(add $user_state.mining_actions, 1),
            1 => $user_state.social_actions = safe_math!(add $user_state.social_actions, 1),
            2 => $user_state.staking_actions = safe_math!(add $user_state.staking_actions, 1),
            3 => $user_state.referral_actions = safe_math!(add $user_state.referral_actions, 1),
            _ => {},
        }
    }};
}

/// Macro for calculating quality score based on multiple factors
#[macro_export]
macro_rules! calculate_quality_score {
    ($content_quality:expr, $engagement_rate:expr, $platform_bonus:expr, $user_reputation:expr) => {{
        let base_quality = ($content_quality as f64 / 100.0).max(0.5).min(2.0);
        let engagement_bonus = ($engagement_rate as f64 / 100.0).max(0.8).min(1.5);
        let platform_multiplier = match $platform_bonus {
            1 => 1.1, // Instagram
            2 => 1.2, // TikTok  
            3 => 1.3, // YouTube
            4 => 1.2, // Twitter/X
            5 => 1.1, // Facebook
            _ => 1.0,
        };
        let reputation_factor = (1.0 + $user_reputation as f64 / 1000.0).min(1.5);
        
        let final_score = base_quality * engagement_bonus * platform_multiplier * reputation_factor;
        final_score.max(0.5).min(2.0)
    }};
}

/// Macro for calculating network regression factor
#[macro_export]
macro_rules! calculate_network_regression {
    ($total_network_size:expr, $network_quality:expr) => {{
        let regression_coefficient = 0.0001;
        let quality_adjustment = $network_quality.max(0.1).min(1.0);
        let regression_factor = -regression_coefficient * ($total_network_size as f64) * quality_adjustment;
        
        if regression_factor < -100.0 {
            0.0001
        } else {
            regression_factor.exp().max(0.0001).min(1.0)
        }
    }};
}

/// Macro for validating staking operations
#[macro_export]
macro_rules! validate_staking_operation {
    ($amount:expr, $user_balance:expr, $min_stake:expr, $max_stake:expr) => {{
        if $amount < $min_stake {
            return Err(FinovaError::InsufficientStakeAmount.into());
        }
        
        if $amount > $max_stake {
            return Err(FinovaError::ExcessiveStakeAmount.into());
        }
        
        if $amount > $user_balance {
            return Err(FinovaError::InsufficientBalance.into());
        }
    }};
}

/// Macro for calculating staking rewards with compound interest
#[macro_export]
macro_rules! calculate_staking_rewards {
    ($principal:expr, $apy:expr, $duration_seconds:expr, $compounding_frequency:expr) => {{
        let annual_rate = $apy / 100.0;
        let periods_per_year = $compounding_frequency as f64;
        let years = $duration_seconds as f64 / (365.25 * 24.0 * 3600.0);
        
        let compound_factor = (1.0 + annual_rate / periods_per_year).powf(periods_per_year * years);
        let final_amount = $principal as f64 * compound_factor;
        let rewards = final_amount - $principal as f64;
        
        rewards.max(0.0) as u64
    }};
}

/// Macro for guild operations validation
#[macro_export]
macro_rules! validate_guild_operation {
    ($user_level:expr, $min_level:expr, $guild_size:expr, $max_size:expr) => {{
        if $user_level < $min_level {
            return Err(FinovaError::InsufficientLevel.into());
        }
        
        if $guild_size >= $max_size {
            return Err(FinovaError::GuildFull.into());
        }
    }};
}

/// Macro for event emission with proper formatting
#[macro_export]
macro_rules! emit_finova_event {
    ($event:expr) => {{
        emit!($event);
        msg!("Event emitted: {}", stringify!($event));
    }};
}

/// Macro for secure random number generation using clock and slot
#[macro_export]
macro_rules! generate_secure_random {
    ($seed:expr, $range:expr) => {{
        let clock = Clock::get()?;
        let combined_seed = $seed
            .wrapping_mul(clock.unix_timestamp as u64)
            .wrapping_add(clock.slot);
        
        (combined_seed % $range as u64) as u32
    }};
}

/// Macro for validating account ownership and signatures
#[macro_export]
macro_rules! validate_account_access {
    ($account:expr, $expected_owner:expr, $signer:expr) => {{
        if $account.owner != $expected_owner {
            return Err(FinovaError::InvalidAccountOwner.into());
        }
        
        if !$signer.is_signer {
            return Err(FinovaError::UnauthorizedAccess.into());
        }
    }};
}

/// Macro for rate limiting checks
#[macro_export]
macro_rules! check_rate_limit {
    ($last_action:expr, $current_time:expr, $rate_limit:expr, $action_count:expr, $max_actions:expr) => {{
        let time_window = $current_time - $last_action;
        
        if time_window < $rate_limit {
            if $action_count >= $max_actions {
                return Err(FinovaError::RateLimitExceeded.into());
            }
        }
    }};
}

/// Macro for calculating dynamic fees based on network conditions
#[macro_export]
macro_rules! calculate_dynamic_fee {
    ($base_fee:expr, $network_congestion:expr, $user_tier:expr) => {{
        let congestion_multiplier = match $network_congestion {
            0..=25 => 1.0,
            26..=50 => 1.2,
            51..=75 => 1.5,
            76..=90 => 2.0,
            _ => 3.0,
        };
        
        let tier_discount = match $user_tier {
            0 => 1.0,    // Explorer
            1 => 0.95,   // Connector
            2 => 0.90,   // Influencer  
            3 => 0.85,   // Leader
            4 => 0.80,   // Ambassador
            _ => 1.0,
        };
        
        let final_fee = ($base_fee as f64 * congestion_multiplier * tier_discount) as u64;
        final_fee.max(1) // Minimum fee of 1
    }};
}

/// Macro for blockchain state consistency checks
#[macro_export]
macro_rules! verify_state_consistency {
    ($state:expr, $expected_version:expr) => {{
        if $state.version != $expected_version {
            return Err(FinovaError::StateVersionMismatch.into());
        }
        
        if $state.is_frozen {
            return Err(FinovaError::StateFrozen.into());
        }
    }};
}

/// Macro for emergency pause checks
#[macro_export]
macro_rules! check_emergency_pause {
    ($global_state:expr) => {{
        if $global_state.is_paused {
            return Err(FinovaError::SystemPaused.into());
        }
        
        if $global_state.emergency_mode {
            return Err(FinovaError::EmergencyMode.into());
        }
    }};
}

/// Macro for logging with context information
#[macro_export]
macro_rules! log_with_context {
    ($level:expr, $message:expr, $($key:expr => $value:expr),*) => {{
        msg!("[{}] {} | Context: {}", 
            $level, 
            $message,
            format!($("{}: {}, "),* $($key, $value),*)
        );
    }};
}

/// Macro for batch operations with transaction limits
#[macro_export]
macro_rules! process_batch_operation {
    ($items:expr, $batch_size:expr, $processor:expr) => {{
        let mut processed = 0;
        let mut failed = 0;
        
        for chunk in $items.chunks($batch_size) {
            for item in chunk {
                match $processor(item) {
                    Ok(_) => processed += 1,
                    Err(_) => failed += 1,
                }
            }
            
            // Prevent transaction timeout
            if processed + failed >= MAX_BATCH_SIZE {
                break;
            }
        }
        
        (processed, failed)
    }};
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_safe_math_operations() {
        // Test addition
        assert_eq!(safe_math!(add 100u64, 50u64), 150u64);
        
        // Test subtraction  
        assert_eq!(safe_math!(sub 100u64, 30u64), 70u64);
        
        // Test multiplication
        assert_eq!(safe_math!(mul 10u64, 5u64), 50u64);
        
        // Test division
        assert_eq!(safe_math!(div 100u64, 4u64), 25u64);
    }

    #[test]
    fn test_xp_level_calculation() {
        // Test basic level calculation
        let level = calculate_xp_level!(1000u64);
        assert!(level > 0 && level <= 100);
        
        // Test maximum level cap
        let max_level = calculate_xp_level!(u64::MAX);
        assert_eq!(max_level, 100);
    }

    #[test]
    fn test_regression_calculation() {
        let base_value = 1000u64;
        let regression_factor = 0.001f64;
        let holdings = 5000u64;
        
        let result = calculate_regression!(base_value, regression_factor, holdings);
        assert!(result > 0 && result <= base_value);
    }

    #[test]
    fn test_quality_score_calculation() {
        let quality = calculate_quality_score!(80u32, 90u32, 2u8, 500u32);
        assert!(quality >= 0.5 && quality <= 2.0);
    }

    #[test]
    fn test_rp_tier_calculation() {
        // Test Explorer tier
        assert_eq!(calculate_rp_tier!(500u64), 0);
        
        // Test Connector tier
        assert_eq!(calculate_rp_tier!(2500u64), 1);
        
        // Test Ambassador tier
        assert_eq!(calculate_rp_tier!(60000u64), 4);
    }
}
