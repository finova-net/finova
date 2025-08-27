// programs/finova-core/src/instructions/mod.rs

use anchor_lang::prelude::*;

/// Instructions module for Finova Core program
/// Contains all instruction implementations and their contexts

// Re-export all instruction functions
pub use initialize::*;
pub use mining::*;
pub use staking::*;
pub use referral::*;
pub use governance::*;
pub use xp::*;
pub use rewards::*;
pub use anti_bot::*;
pub use guild::*;
pub use quality::*;

// Module declarations
pub mod initialize;
pub mod mining;
pub mod staking;
pub mod referral;
pub mod governance;
pub mod xp;
pub mod rewards;
pub mod anti_bot;
pub mod guild;
pub mod quality;

/// Common instruction context trait for validation
pub trait InstructionContext {
    /// Validate the instruction context
    fn validate(&self) -> Result<()>;
    
    /// Check if the user is authorized for this action
    fn check_authorization(&self, user: &Pubkey) -> Result<()>;
}

/// Common validation functions used across instructions
pub mod validation {
    use super::*;
    use crate::state::*;
    use crate::errors::FinovaError;

    /// Validate that a user account exists and is initialized
    pub fn validate_user_account(user_account: &Account<UserAccount>) -> Result<()> {
        require!(
            user_account.is_initialized,
            FinovaError::UserNotInitialized
        );
        Ok(())
    }

    /// Validate that a user is KYC verified for premium actions
    pub fn validate_kyc_required(user_account: &Account<UserAccount>) -> Result<()> {
        require!(
            user_account.kyc_verified,
            FinovaError::KYCRequired
        );
        Ok(())
    }

    /// Validate that the system is not paused
    pub fn validate_system_operational(global_config: &Account<GlobalConfig>) -> Result<()> {
        require!(
            global_config.is_operational(),
            FinovaError::SystemPaused
        );
        Ok(())
    }

    /// Validate that a user meets minimum XP requirements
    pub fn validate_xp_requirement(
        user_account: &Account<UserAccount>,
        xp_account: &Account<XPAccount>,
        min_xp: u64
    ) -> Result<()> {
        require!(
            xp_account.total_xp >= min_xp,
            FinovaError::InsufficientXP
        );
        Ok(())
    }

    /// Validate that a user is not flagged as a bot
    pub fn validate_not_bot(user_account: &Account<UserAccount>) -> Result<()> {
        require!(
            user_account.bot_probability < 8000, // Less than 80%
            FinovaError::BotDetected
        );
        Ok(())
    }

    /// Validate that a user is not in cooldown
    pub fn validate_not_in_cooldown(user_account: &Account<UserAccount>) -> Result<()> {
        let current_time = Clock::get()?.unix_timestamp;
        require!(
            current_time >= user_account.cooldown_until,
            FinovaError::UserInCooldown
        );
        Ok(())
    }

    /// Validate daily limits for activities
    pub fn validate_daily_limit(
        current_count: u32,
        max_count: u32,
        error: FinovaError
    ) -> Result<()> {
        require!(
            current_count < max_count,
            error
        );
        Ok(())
    }

    /// Validate that a timestamp is within acceptable range
    pub fn validate_timestamp(timestamp: i64, max_drift: i64) -> Result<()> {
        let current_time = Clock::get()?.unix_timestamp;
        let time_diff = (current_time - timestamp).abs();
        
        require!(
            time_diff <= max_drift,
            FinovaError::InvalidTimestamp
        );
        Ok(())
    }

    /// Validate that an amount is within acceptable limits
    pub fn validate_amount_limits(
        amount: u64,
        min_amount: u64,
        max_amount: u64
    ) -> Result<()> {
        require!(
            amount >= min_amount && amount <= max_amount,
            FinovaError::InvalidAmount
        );
        Ok(())
    }

    /// Validate that a referral code is properly formatted
    pub fn validate_referral_code(code: &str) -> Result<()> {
        require!(
            code.len() >= 6 && code.len() <= 12,
            FinovaError::InvalidReferralCode
        );
        
        require!(
            code.chars().all(|c| c.is_alphanumeric()),
            FinovaError::InvalidReferralCode
        );
        
        Ok(())
    }

    /// Validate social platform enum
    pub fn validate_social_platform(platform: u8) -> Result<()> {
        require!(
            platform <= 6, // Based on SocialPlatform enum
            FinovaError::InvalidSocialPlatform
        );
        Ok(())
    }

    /// Validate content type for XP activities
    pub fn validate_content_type(content_type: u8) -> Result<()> {
        require!(
            content_type <= 10, // Based on ContentType enum
            FinovaError::InvalidContentType
        );
        Ok(())
    }

    /// Validate guild name format
    pub fn validate_guild_name(name: &str) -> Result<()> {
        require!(
            name.len() >= 3 && name.len() <= 32,
            FinovaError::InvalidGuildName
        );
        
        require!(
            !name.is_empty() && name.trim() == name,
            FinovaError::InvalidGuildName
        );
        
        Ok(())
    }

    /// Validate that a user can perform an action based on their level
    pub fn validate_level_requirement(
        xp_account: &Account<XPAccount>,
        required_level: u16
    ) -> Result<()> {
        require!(
            xp_account.current_level >= required_level,
            FinovaError::InsufficientLevel
        );
        Ok(())
    }

    /// Validate signature for off-chain data
    pub fn validate_signature(
        message: &[u8],
        signature: &[u8; 64],
        signer: &Pubkey
    ) -> Result<()> {
        use anchor_lang::solana_program::secp256k1_recover::{
            secp256k1_recover,
        };
        
        // For Solana, we typically use ed25519 signatures
        // This is a placeholder for signature verification
        // In production, implement proper signature verification
        require!(
            signature.len() == 64,
            FinovaError::InvalidSignature
        );
        
        Ok(())
    }

    /// Validate network quality score
    pub fn validate_network_quality(
        quality_score: u16,
        min_quality: u16
    ) -> Result<()> {
        require!(
            quality_score >= min_quality,
            FinovaError::InsufficientNetworkQuality
        );
        Ok(())
    }

    /// Validate staking amount and duration
    pub fn validate_staking_params(
        amount: u64,
        duration_days: u16,
        min_amount: u64,
        max_duration: u16
    ) -> Result<()> {
        require!(
            amount >= min_amount,
            FinovaError::InsufficientStakeAmount
        );
        
        require!(
            duration_days > 0 && duration_days <= max_duration,
            FinovaError::InvalidStakeDuration
        );
        
        Ok(())
    }

    /// Validate that an account is owned by the correct program
    pub fn validate_account_owner(
        account_info: &AccountInfo,
        expected_owner: &Pubkey
    ) -> Result<()> {
        require!(
            account_info.owner == expected_owner,
            FinovaError::InvalidAccountOwner
        );
        Ok(())
    }

    /// Validate that an account has sufficient lamports
    pub fn validate_sufficient_lamports(
        account_info: &AccountInfo,
        required_lamports: u64
    ) -> Result<()> {
        require!(
            account_info.lamports() >= required_lamports,
            FinovaError::InsufficientLamports
        );
        Ok(())
    }

    /// Validate content quality scores
    pub fn validate_quality_scores(
        originality: u16,
        engagement: u16,
        brand_safety: u16,
        ai_score: u16
    ) -> Result<()> {
        // Scores should be in basis points (0-10000)
        require!(
            originality <= 10000 && 
            engagement <= 10000 && 
            brand_safety <= 10000 && 
            ai_score <= 10000,
            FinovaError::InvalidQualityScore
        );
        Ok(())
    }

    /// Validate mining session parameters
    pub fn validate_mining_session(
        session_start: i64,
        session_duration: u32,
        max_session_duration: u32
    ) -> Result<()> {
        let current_time = Clock::get()?.unix_timestamp;
        
        require!(
            session_start <= current_time,
            FinovaError::InvalidSessionStart
        );
        
        require!(
            session_duration <= max_session_duration,
            FinovaError::SessionTooLong
        );
        
        let session_end = session_start + session_duration as i64;
        require!(
            session_end <= current_time + 300, // 5 minute tolerance
            FinovaError::InvalidSessionDuration
        );
        
        Ok(())
    }

    /// Validate referral network depth and size
    pub fn validate_referral_network(
        network_size: u32,
        network_depth: u8,
        max_network_size: u32,
        max_depth: u8
    ) -> Result<()> {
        require!(
            network_size <= max_network_size,
            FinovaError::NetworkTooLarge
        );
        
        require!(
            network_depth <= max_depth,
            FinovaError::NetworkTooDeep
        );
        
        Ok(())
    }
}

/// Utility functions for instruction processing
pub mod utils {
    use super::*;
    use crate::constants::*;

    /// Calculate the current Unix timestamp
    pub fn current_timestamp() -> Result<i64> {
        Ok(Clock::get()?.unix_timestamp)
    }

    /// Calculate the start of the current day (UTC)
    pub fn start_of_day(timestamp: i64) -> i64 {
        timestamp - (timestamp % SECONDS_PER_DAY)
    }

    /// Check if two timestamps are on the same day
    pub fn is_same_day(timestamp1: i64, timestamp2: i64) -> bool {
        start_of_day(timestamp1) == start_of_day(timestamp2)
    }

    /// Calculate mining multiplier based on various factors
    pub fn calculate_mining_multiplier(
        base_multiplier: u16,
        xp_multiplier: u16,
        rp_multiplier: u16,
        quality_multiplier: u16,
        kyc_bonus: u16
    ) -> u64 {
        let total_multiplier = (base_multiplier as u64)
            .saturating_mul(xp_multiplier as u64)
            .saturating_mul(rp_multiplier as u64)
            .saturating_mul(quality_multiplier as u64)
            .saturating_mul(kyc_bonus as u64);
        
        // Divide by BASIS_POINTS^4 to get back to normal scale
        total_multiplier / (BASIS_POINTS as u64).pow(4)
    }

    /// Calculate exponential regression factor
    pub fn calculate_regression_factor(holdings: u64, regression_rate: u16) -> u16 {
        // Simplified exponential decay: e^(-rate * holdings / 1e9)
        // Using integer approximation for on-chain computation
        let scaled_holdings = holdings / 1_000_000_000; // Scale down
        let decay_factor = scaled_holdings.saturating_mul(regression_rate as u64) / BASIS_POINTS as u64;
        
        if decay_factor >= 10000 {
            1 // Minimum 0.01% of original
        } else {
            (10000 - decay_factor as u16).max(1)
        }
    }

    /// Convert basis points to percentage string (for logging)
    pub fn basis_points_to_percentage(bp: u16) -> String {
        format!("{:.2}%", bp as f64 / 100.0)
    }

    /// Calculate time-based decay for streaks and bonuses
    pub fn calculate_time_decay(
        last_activity: i64,
        current_time: i64,
        decay_rate: u16
    ) -> u16 {
        let time_diff = current_time - last_activity;
        let hours_passed = time_diff / 3600; // Convert to hours
        
        if hours_passed <= 0 {
            return BASIS_POINTS; // No decay
        }
        
        // Exponential decay: (decay_rate/10000)^hours
        let decay_factor = (decay_rate as u64 * hours_passed as u64) / BASIS_POINTS as u64;
        
        if decay_factor >= BASIS_POINTS as u64 {
            1 // Minimum decay
        } else {
            (BASIS_POINTS - decay_factor as u16).max(1)
        }
    }

    /// Generate a unique seed for PDAs
    pub fn generate_pda_seed(base: &str, user: &Pubkey, counter: u64) -> Vec<u8> {
        let mut seed = Vec::new();
        seed.extend_from_slice(base.as_bytes());
        seed.extend_from_slice(&user.to_bytes());
        seed.extend_from_slice(&counter.to_le_bytes());
        seed
    }

    /// Calculate network growth bonus
    pub fn calculate_network_growth_bonus(
        network_size: u32,
        growth_rate: u16,
        max_bonus: u16
    ) -> u16 {
        let bonus = (network_size as u64 * growth_rate as u64) / 100;
        (bonus as u16).min(max_bonus)
    }

    /// Calculate content virality multiplier
    pub fn calculate_virality_multiplier(
        views: u64,
        likes: u32,
        shares: u32,
        comments: u32
    ) -> u16 {
        let engagement_score = (likes as u64 * 1) + 
                               (shares as u64 * 3) + 
                               (comments as u64 * 2);
        
        let virality_ratio = if views > 0 {
            (engagement_score * 10000) / views
        } else {
            0
        };

        match virality_ratio {
            0..=100 => BASIS_POINTS,           // 1.0x
            101..=500 => BASIS_POINTS + 2000,  // 1.2x
            501..=1000 => BASIS_POINTS + 5000, // 1.5x
            1001..=2000 => 20000,              // 2.0x
            _ => 30000,                        // 3.0x max
        }
    }

    /// Calculate level-based multiplier
    pub fn calculate_level_multiplier(level: u16) -> u16 {
        match level {
            0..=10 => BASIS_POINTS,                    // 1.0x - 1.2x
            11..=25 => BASIS_POINTS + (level * 20),    // 1.2x - 1.8x
            26..=50 => BASIS_POINTS + 500 + (level * 30), // 1.8x - 2.5x
            51..=75 => BASIS_POINTS + 1200 + (level * 40), // 2.5x - 3.2x
            76..=100 => BASIS_POINTS + 2200 + (level * 50), // 3.2x - 4.0x
            _ => BASIS_POINTS + 3700 + ((level - 100) * 10), // 4.0x - 5.0x max
        }
    }

    /// Calculate anti-whale regression
    pub fn calculate_whale_regression(total_holdings: u64) -> u16 {
        const WHALE_THRESHOLD: u64 = 100_000 * 1_000_000_000; // 100K FIN in lamports
        
        if total_holdings <= WHALE_THRESHOLD {
            return BASIS_POINTS; // No regression
        }

        let excess_holdings = total_holdings - WHALE_THRESHOLD;
        let regression_factor = (excess_holdings / 1_000_000_000) as u16; // Per 1 FIN excess
        
        // Maximum 90% reduction (1000 basis points minimum)
        (BASIS_POINTS - regression_factor.min(9000)).max(1000)
    }

    /// Generate deterministic randomness for fair distribution
    pub fn generate_deterministic_random(
        user: &Pubkey,
        timestamp: i64,
        nonce: u64
    ) -> u64 {
        use anchor_lang::solana_program::keccak;
        
        let mut input = Vec::new();
        input.extend_from_slice(&user.to_bytes());
        input.extend_from_slice(&timestamp.to_le_bytes());
        input.extend_from_slice(&nonce.to_le_bytes());
        
        let hash = keccak::hash(&input);
        u64::from_le_bytes([
            hash.0[0], hash.0[1], hash.0[2], hash.0[3],
            hash.0[4], hash.0[5], hash.0[6], hash.0[7],
        ])
    }
}

/// Event emission utilities
pub mod events {
    use super::*;
    use crate::events::*;

    /// Emit mining reward event
    pub fn emit_mining_reward(
        user: Pubkey,
        amount: u64,
        multiplier: u16,
        session_duration: u32
    ) -> Result<()> {
        emit!(MiningRewardEvent {
            user,
            amount,
            multiplier,
            session_duration,
            timestamp: Clock::get()?.unix_timestamp,
        });
        Ok(())
    }

    /// Emit XP gained event
    pub fn emit_xp_gained(
        user: Pubkey,
        amount: u64,
        activity_type: u8,
        platform: u8,
        quality_multiplier: u16
    ) -> Result<()> {
        emit!(XPGainedEvent {
            user,
            amount,
            activity_type,
            platform,
            quality_multiplier,
            timestamp: Clock::get()?.unix_timestamp,
        });
        Ok(())
    }

    /// Emit referral reward event
    pub fn emit_referral_reward(
        referrer: Pubkey,
        referee: Pubkey,
        reward_amount: u64,
        reward_type: u8
    ) -> Result<()> {
        emit!(ReferralRewardEvent {
            referrer,
            referee,
            reward_amount,
            reward_type,
            timestamp: Clock::get()?.unix_timestamp,
        });
        Ok(())
    }

    /// Emit level up event
    pub fn emit_level_up(
        user: Pubkey,
        old_level: u16,
        new_level: u16,
        total_xp: u64
    ) -> Result<()> {
        emit!(LevelUpEvent {
            user,
            old_level,
            new_level,
            total_xp,
            timestamp: Clock::get()?.unix_timestamp,
        });
        Ok(())
    }

    /// Emit guild event
    pub fn emit_guild_event(
        guild: Pubkey,
        user: Pubkey,
        event_type: u8,
        data: Vec<u8>
    ) -> Result<()> {
        emit!(GuildEvent {
            guild,
            user,
            event_type,
            data,
            timestamp: Clock::get()?.unix_timestamp,
        });
        Ok(())
    }
}

/// Macro for common instruction validation pattern
#[macro_export]
macro_rules! validate_instruction {
    ($ctx:expr, $($validation:expr),*) => {
        $(
            $validation?;
        )*
    };
}

/// Macro for calculating and applying multipliers
#[macro_export]
macro_rules! apply_multipliers {
    ($base:expr, $($multiplier:expr),*) => {
        {
            let mut result = $base;
            $(
                result = (result * $multiplier as u64) / BASIS_POINTS as u64;
            )*
            result
        }
    };
}

/// Macro for safe arithmetic operations
#[macro_export]
macro_rules! safe_add {
    ($a:expr, $b:expr) => {
        $a.checked_add($b).ok_or(FinovaError::ArithmeticOverflow)?
    };
}

#[macro_export]
macro_rules! safe_sub {
    ($a:expr, $b:expr) => {
        $a.checked_sub($b).ok_or(FinovaError::ArithmeticUnderflow)?
    };
}

#[macro_export]
macro_rules! safe_mul {
    ($a:expr, $b:expr) => {
        $a.checked_mul($b).ok_or(FinovaError::ArithmeticOverflow)?
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::utils::*;

    #[test]
    fn test_calculate_mining_multiplier() {
        let result = calculate_mining_multiplier(
            10000, // 1.0x base
            12000, // 1.2x XP
            15000, // 1.5x RP
            18000, // 1.8x quality
            12000, // 1.2x KYC
        );
        
        // Expected: 1.0 * 1.2 * 1.5 * 1.8 * 1.2 = 3.888
        assert!(result > 38000 && result < 40000);
    }

    #[test]
    fn test_calculate_regression_factor() {
        // Test no regression for small holdings
        let factor = calculate_regression_factor(1_000_000_000, 1000); // 1 FIN, 10% rate
        assert_eq!(factor, 10000);

        // Test regression for large holdings
        let factor = calculate_regression_factor(100_000_000_000_000, 1000); // 100K FIN
        assert!(factor < 10000);
    }

    #[test]
    fn test_virality_multiplier() {
        // Test low engagement
        let multiplier = calculate_virality_multiplier(1000, 10, 2, 5);
        assert_eq!(multiplier, BASIS_POINTS);

        // Test high engagement
        let multiplier = calculate_virality_multiplier(1000, 100, 50, 25);
        assert!(multiplier > BASIS_POINTS);
    }

    #[test]
    fn test_level_multiplier() {
        // Test beginner level
        let multiplier = calculate_level_multiplier(5);
        assert_eq!(multiplier, BASIS_POINTS);

        // Test mid level
        let multiplier = calculate_level_multiplier(50);
        assert!(multiplier > BASIS_POINTS && multiplier < 30000);

        // Test high level
        let multiplier = calculate_level_multiplier(100);
        assert!(multiplier >= 40000);
    }

    #[test]
    fn test_whale_regression() {
        // Test normal holdings
        let regression = calculate_whale_regression(50_000 * 1_000_000_000); // 50K FIN
        assert_eq!(regression, BASIS_POINTS);

        // Test whale holdings
        let regression = calculate_whale_regression(500_000 * 1_000_000_000); // 500K FIN
        assert!(regression < BASIS_POINTS);
    }

    #[test]
    fn test_time_decay() {
        let current_time = Clock::get().unwrap().unix_timestamp;
        let one_hour_ago = current_time - 3600;
        
        let decay = calculate_time_decay(one_hour_ago, current_time, 9000); // 90% decay rate
        assert!(decay < BASIS_POINTS);
    }

    #[test]
    fn test_deterministic_random() {
        let user = Pubkey::new_unique();
        let timestamp = Clock::get().unwrap().unix_timestamp;
        
        let random1 = generate_deterministic_random(&user, timestamp, 1);
        let random2 = generate_deterministic_random(&user, timestamp, 1);
        let random3 = generate_deterministic_random(&user, timestamp, 2);
        
        // Same inputs should produce same output
        assert_eq!(random1, random2);
        
        // Different nonce should produce different output
        assert_ne!(random1, random3);
    }

    #[test]
    fn test_validation_functions() {
        // Test referral code validation
        assert!(validation::validate_referral_code("ABC123").is_ok());
        assert!(validation::validate_referral_code("12345").is_err()); // Too short
        assert!(validation::validate_referral_code("TOOLONGCODE123").is_err()); // Too long
        assert!(validation::validate_referral_code("ABC@123").is_err()); // Invalid chars

        // Test guild name validation
        assert!(validation::validate_guild_name("CoolGuild").is_ok());
        assert!(validation::validate_guild_name("AB").is_err()); // Too short
        assert!(validation::validate_guild_name("").is_err()); // Empty
        assert!(validation::validate_guild_name(" SpacedName ").is_err()); // Leading/trailing spaces
    }
}
