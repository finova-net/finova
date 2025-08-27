// programs/finova-core/src/state/mod.rs

//! # Finova Core State Module
//! 
//! This module contains all state structures for the Finova Network core smart contract.
//! It implements the integrated XP + RP + $FIN mining system with exponential regression,
//! anti-bot mechanisms, and comprehensive user management.
//!
//! ## Features
//! - User account management with KYC integration
//! - Mining system with Pi Network-inspired mechanics
//! - Experience Points (XP) system with Hamster Kombat gamification
//! - Referral Points (RP) system with network effects
//! - Staking and enhanced rewards
//! - Guild system integration
//! - Anti-bot and quality assessment
//! - Network-wide statistics and governance
//!
//! ## Security Features
//! - Comprehensive input validation
//! - Overflow protection
//! - Access control mechanisms
//! - Anti-bot detection integration
//! - Economic attack prevention
//!
//! Version: 3.0
//! Last Updated: July 26, 2025

use anchor_lang::prelude::*;
use std::collections::BTreeMap;

// Re-export all state modules
pub mod user;
pub mod mining;
pub mod staking;
pub mod referral;
pub mod guild;
pub mod xp;
pub mod rewards;
pub mod network;

// Re-export all public types
pub use user::*;
pub use mining::*;
pub use staking::*;
pub use referral::*;
pub use guild::*;
pub use xp::*;
pub use rewards::*;
pub use network::*;

/// Maximum number of referrals per user to prevent spam
pub const MAX_REFERRALS_PER_USER: u32 = 1000;

/// Maximum number of guild members
pub const MAX_GUILD_MEMBERS: u32 = 50;

/// Maximum number of special cards a user can hold
pub const MAX_SPECIAL_CARDS: u32 = 100;

/// Maximum XP level to prevent overflow
pub const MAX_XP_LEVEL: u32 = 1000;

/// Mining phases based on total users
pub const PHASE_1_USER_LIMIT: u64 = 100_000;
pub const PHASE_2_USER_LIMIT: u64 = 1_000_000;
pub const PHASE_3_USER_LIMIT: u64 = 10_000_000;

/// Base mining rates per phase (in microFIN per hour)
pub const PHASE_1_BASE_RATE: u64 = 100_000; // 0.1 FIN/hour
pub const PHASE_2_BASE_RATE: u64 = 50_000;  // 0.05 FIN/hour
pub const PHASE_3_BASE_RATE: u64 = 25_000;  // 0.025 FIN/hour
pub const PHASE_4_BASE_RATE: u64 = 10_000;  // 0.01 FIN/hour

/// Finizen bonus multipliers
pub const PHASE_1_FINIZEN_BONUS: u64 = 200; // 2.0x
pub const PHASE_2_FINIZEN_BONUS: u64 = 150; // 1.5x
pub const PHASE_3_FINIZEN_BONUS: u64 = 120; // 1.2x
pub const PHASE_4_FINIZEN_BONUS: u64 = 100; // 1.0x

/// Security and KYC bonuses
pub const KYC_VERIFIED_BONUS: u64 = 120; // 1.2x
pub const KYC_UNVERIFIED_PENALTY: u64 = 80; // 0.8x

/// Referral system constants
pub const REFERRAL_BONUS_PER_ACTIVE: u64 = 10; // 0.1x per active referral
pub const MAX_REFERRAL_BONUS: u64 = 300; // Maximum 3.0x bonus

/// XP system constants
pub const XP_LEVEL_MULTIPLIER_BASE: u64 = 100; // Base for level multiplier calculation
pub const XP_DECAY_FACTOR: u64 = 1; // 0.01 decay factor for progression

/// RP system constants
pub const RP_TIER_MULTIPLIER: u64 = 20; // 0.2x per tier
pub const RP_NETWORK_REGRESSION_FACTOR: u64 = 1; // 0.0001 regression factor

/// Staking system constants
pub const MIN_STAKE_AMOUNT: u64 = 100_000_000; // 100 FIN minimum stake
pub const LOYALTY_BONUS_PER_MONTH: u64 = 5; // 0.05x per month
pub const ACTIVITY_BONUS_MULTIPLIER: u64 = 10; // 0.1x per activity score

/// Quality assessment constants
pub const MIN_QUALITY_SCORE: u64 = 50; // Minimum 0.5x quality score
pub const MAX_QUALITY_SCORE: u64 = 200; // Maximum 2.0x quality score

/// Anti-bot and regression constants
pub const REGRESSION_FACTOR_BASE: u64 = 1000; // Base for exponential regression
pub const WHALE_REGRESSION_THRESHOLD: u64 = 100_000_000; // 100K FIN threshold
pub const BOT_DETECTION_THRESHOLD: u64 = 50; // 0.5 human probability threshold

/// Time constants (in seconds)
pub const SECONDS_PER_HOUR: i64 = 3600;
pub const SECONDS_PER_DAY: i64 = 86400;
pub const SECONDS_PER_WEEK: i64 = 604800;
pub const SECONDS_PER_MONTH: i64 = 2592000; // 30 days

/// Economic constants
pub const MICRO_FIN_DECIMALS: u8 = 6; // 1 FIN = 1,000,000 microFIN
pub const MAX_DAILY_MINING_CAP: u64 = 15_000_000; // 15 FIN daily cap
pub const BURN_RATE_BASIS_POINTS: u64 = 10; // 0.1% burn rate

/// Platform multipliers for social media integration
pub const PLATFORM_MULTIPLIER_TIKTOK: u64 = 130; // 1.3x
pub const PLATFORM_MULTIPLIER_INSTAGRAM: u64 = 120; // 1.2x
pub const PLATFORM_MULTIPLIER_YOUTUBE: u64 = 140; // 1.4x
pub const PLATFORM_MULTIPLIER_FACEBOOK: u64 = 110; // 1.1x
pub const PLATFORM_MULTIPLIER_X: u64 = 120; // 1.2x
pub const PLATFORM_MULTIPLIER_DEFAULT: u64 = 100; // 1.0x

/// Guild system constants
pub const GUILD_CREATION_FEE: u64 = 1000_000_000; // 1000 FIN
pub const GUILD_PARTICIPATION_BONUS: u64 = 30; // 0.3x bonus
pub const MAX_GUILDS_PER_USER: u32 = 3;

/// NFT and Special Cards constants
pub const CARD_USAGE_COOLDOWN: i64 = 86400; // 24 hours
pub const MAX_ACTIVE_CARDS: u32 = 5;
pub const CARD_SYNERGY_BONUS: u64 = 15; // 0.15x synergy bonus

/// Governance constants
pub const MIN_GOVERNANCE_STAKE: u64 = 10_000_000_000; // 10K FIN for voting
pub const PROPOSAL_THRESHOLD: u64 = 100_000_000_000; // 100K FIN to create proposal
pub const VOTING_PERIOD: i64 = 604800; // 7 days

/// Error codes for state validation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StateValidationError {
    InvalidUserData,
    InvalidMiningData,
    InvalidStakingData,
    InvalidReferralData,
    InvalidXPData,
    InvalidGuildData,
    InvalidNetworkData,
    OverflowError,
    UnderflowError,
    AccessDenied,
    InsufficientFunds,
    InvalidTimestamp,
    ExceedsMaxLimit,
    BelowMinThreshold,
    InvalidQualityScore,
    BotDetectionFailed,
    NetworkRegression,
    InvalidPhase,
}

/// Result type for state operations
pub type StateResult<T> = Result<T, StateValidationError>;

/// Utility trait for state validation
pub trait StateValidation {
    /// Validates the state structure
    fn validate(&self) -> StateResult<()>;
    
    /// Checks if the state is within acceptable limits
    fn check_limits(&self) -> StateResult<()>;
    
    /// Performs security checks
    fn security_check(&self) -> StateResult<()>;
}

/// Utility trait for mathematical operations with overflow protection
pub trait SafeMath {
    /// Safe addition with overflow check
    fn safe_add(&self, other: Self) -> StateResult<Self> where Self: Sized;
    
    /// Safe subtraction with underflow check
    fn safe_sub(&self, other: Self) -> StateResult<Self> where Self: Sized;
    
    /// Safe multiplication with overflow check
    fn safe_mul(&self, other: Self) -> StateResult<Self> where Self: Sized;
    
    /// Safe division with zero check
    fn safe_div(&self, other: Self) -> StateResult<Self> where Self: Sized;
}

impl SafeMath for u64 {
    fn safe_add(&self, other: u64) -> StateResult<u64> {
        self.checked_add(other).ok_or(StateValidationError::OverflowError)
    }
    
    fn safe_sub(&self, other: u64) -> StateResult<u64> {
        self.checked_sub(other).ok_or(StateValidationError::UnderflowError)
    }
    
    fn safe_mul(&self, other: u64) -> StateResult<u64> {
        self.checked_mul(other).ok_or(StateValidationError::OverflowError)
    }
    
    fn safe_div(&self, other: u64) -> StateResult<u64> {
        if other == 0 {
            return Err(StateValidationError::UnderflowError);
        }
        Ok(self / other)
    }
}

impl SafeMath for u32 {
    fn safe_add(&self, other: u32) -> StateResult<u32> {
        self.checked_add(other).ok_or(StateValidationError::OverflowError)
    }
    
    fn safe_sub(&self, other: u32) -> StateResult<u32> {
        self.checked_sub(other).ok_or(StateValidationError::UnderflowError)
    }
    
    fn safe_mul(&self, other: u32) -> StateResult<u32> {
        self.checked_mul(other).ok_or(StateValidationError::OverflowError)
    }
    
    fn safe_div(&self, other: u32) -> StateResult<u32> {
        if other == 0 {
            return Err(StateValidationError::UnderflowError);
        }
        Ok(self / other)
    }
}

/// Utility functions for common calculations
pub mod utils {
    use super::*;
    
    /// Calculates the current mining phase based on total users
    pub fn get_current_phase(total_users: u64) -> u8 {
        match total_users {
            0..=PHASE_1_USER_LIMIT => 1,
            PHASE_1_USER_LIMIT..=PHASE_2_USER_LIMIT => 2,
            PHASE_2_USER_LIMIT..=PHASE_3_USER_LIMIT => 3,
            _ => 4,
        }
    }
    
    /// Gets base mining rate for current phase
    pub fn get_base_mining_rate(phase: u8) -> u64 {
        match phase {
            1 => PHASE_1_BASE_RATE,
            2 => PHASE_2_BASE_RATE,
            3 => PHASE_3_BASE_RATE,
            _ => PHASE_4_BASE_RATE,
        }
    }
    
    /// Gets Finizen bonus for current phase
    pub fn get_finizen_bonus(phase: u8, total_users: u64) -> u64 {
        let base_bonus = match phase {
            1 => PHASE_1_FINIZEN_BONUS,
            2 => PHASE_2_FINIZEN_BONUS,
            3 => PHASE_3_FINIZEN_BONUS,
            _ => PHASE_4_FINIZEN_BONUS,
        };
        
        // Apply progressive reduction based on user growth
        let reduction = (total_users / 1_000_000).min(100);
        base_bonus.saturating_sub(reduction)
    }
    
    /// Calculates exponential regression factor for anti-whale mechanism
    pub fn calculate_regression_factor(total_holdings: u64) -> u64 {
        if total_holdings < WHALE_REGRESSION_THRESHOLD {
            return 100; // No regression for small holders
        }
        
        // Exponential regression: e^(-0.001 * holdings)
        // Approximated using integer math for efficiency
        let exponent = (total_holdings / REGRESSION_FACTOR_BASE).min(10);
        match exponent {
            0 => 100,
            1 => 90,
            2 => 82,
            3 => 74,
            4 => 67,
            5 => 61,
            6 => 55,
            7 => 50,
            8 => 45,
            9 => 41,
            _ => 37, // Minimum regression factor
        }
    }
    
    /// Calculates XP level multiplier with progression decay
    pub fn calculate_xp_multiplier(xp_level: u32) -> u64 {
        let base_multiplier = 100; // 1.0x base
        let level_bonus = (xp_level as u64 * XP_LEVEL_MULTIPLIER_BASE) / 100;
        
        // Apply exponential decay for high levels
        let decay_factor = if xp_level > 50 {
            let excess_levels = xp_level.saturating_sub(50);
            100_u64.saturating_sub((excess_levels as u64 * XP_DECAY_FACTOR) / 10)
        } else {
            100
        };
        
        (base_multiplier + level_bonus).saturating_mul(decay_factor) / 100
    }
    
    /// Calculates RP tier multiplier
    pub fn calculate_rp_multiplier(rp_tier: u8) -> u64 {
        100 + (rp_tier as u64 * RP_TIER_MULTIPLIER)
    }
    
    /// Validates timestamp is not in the future
    pub fn validate_timestamp(timestamp: i64) -> StateResult<()> {
        let current_time = Clock::get()?.unix_timestamp;
        if timestamp > current_time + 300 { // Allow 5 minutes future for clock skew
            return Err(StateValidationError::InvalidTimestamp);
        }
        Ok(())
    }
    
    /// Calculates platform multiplier based on social media platform
    pub fn get_platform_multiplier(platform: &str) -> u64 {
        match platform.to_lowercase().as_str() {
            "tiktok" => PLATFORM_MULTIPLIER_TIKTOK,
            "instagram" => PLATFORM_MULTIPLIER_INSTAGRAM,
            "youtube" => PLATFORM_MULTIPLIER_YOUTUBE,
            "facebook" => PLATFORM_MULTIPLIER_FACEBOOK,
            "x" | "twitter" => PLATFORM_MULTIPLIER_X,
            _ => PLATFORM_MULTIPLIER_DEFAULT,
        }
    }
    
    /// Converts FIN to microFIN
    pub fn fin_to_micro_fin(fin_amount: u64) -> StateResult<u64> {
        fin_amount.safe_mul(10_u64.pow(MICRO_FIN_DECIMALS as u32))
    }
    
    /// Converts microFIN to FIN
    pub fn micro_fin_to_fin(micro_fin_amount: u64) -> u64 {
        micro_fin_amount / 10_u64.pow(MICRO_FIN_DECIMALS as u32)
    }
    
    /// Calculates quality score based on AI analysis
    pub fn calculate_quality_score(
        originality: u64,
        engagement_potential: u64,
        brand_safety: u64,
        platform_relevance: u64,
    ) -> u64 {
        let weighted_score = (originality * 30 + 
                             engagement_potential * 25 + 
                             brand_safety * 25 + 
                             platform_relevance * 20) / 100;
        
        weighted_score.max(MIN_QUALITY_SCORE).min(MAX_QUALITY_SCORE)
    }
    
    /// Calculates daily mining cap based on user level and tier
    pub fn calculate_daily_cap(xp_level: u32, rp_tier: u8, stake_tier: u8) -> u64 {
        let base_cap = match (xp_level, rp_tier, stake_tier) {
            (0..=10, 0..=1, 0..=1) => 500_000,     // 0.5 FIN
            (11..=25, 0..=2, 0..=2) => 2_000_000,  // 2.0 FIN
            (26..=50, 0..=3, 0..=3) => 4_000_000,  // 4.0 FIN
            (51..=75, 0..=4, 0..=4) => 6_000_000,  // 6.0 FIN
            (76..=100, 0..=5, 0..=5) => 8_000_000, // 8.0 FIN
            _ => 10_000_000, // 10.0 FIN maximum
        };
        
        base_cap.min(MAX_DAILY_MINING_CAP)
    }
}

/// Comprehensive state initialization and management
#[derive(Debug, Clone)]
pub struct StateManager {
    /// Current network phase
    pub current_phase: u8,
    /// Total registered users
    pub total_users: u64,
    /// Total FIN in circulation
    pub total_fin_supply: u64,
    /// Network health metrics
    pub network_health: NetworkHealth,
}

impl StateManager {
    /// Creates a new state manager
    pub fn new() -> Self {
        Self {
            current_phase: 1,
            total_users: 0,
            total_fin_supply: 0,
            network_health: NetworkHealth::new(),
        }
    }
    
    /// Updates network statistics
    pub fn update_network_stats(&mut self, user_count_delta: i64, supply_delta: i64) -> StateResult<()> {
        // Update user count safely
        if user_count_delta >= 0 {
            self.total_users = self.total_users.safe_add(user_count_delta as u64)?;
        } else {
            self.total_users = self.total_users.safe_sub((-user_count_delta) as u64)?;
        }
        
        // Update supply safely
        if supply_delta >= 0 {
            self.total_fin_supply = self.total_fin_supply.safe_add(supply_delta as u64)?;
        } else {
            self.total_fin_supply = self.total_fin_supply.safe_sub((-supply_delta) as u64)?;
        }
        
        // Update current phase
        self.current_phase = utils::get_current_phase(self.total_users);
        
        // Update network health
        self.network_health.update(self.total_users, self.total_fin_supply)?;
        
        Ok(())
    }
}

impl Default for StateManager {
    fn default() -> Self {
        Self::new()
    }
}

impl StateValidation for StateManager {
    fn validate(&self) -> StateResult<()> {
        if self.current_phase < 1 || self.current_phase > 4 {
            return Err(StateValidationError::InvalidPhase);
        }
        
        self.network_health.validate()?;
        
        Ok(())
    }
    
    fn check_limits(&self) -> StateResult<()> {
        if self.total_users > u32::MAX as u64 {
            return Err(StateValidationError::ExceedsMaxLimit);
        }
        
        if self.total_fin_supply > 100_000_000_000_000_000 { // 100B FIN max supply
            return Err(StateValidationError::ExceedsMaxLimit);
        }
        
        Ok(())
    }
    
    fn security_check(&self) -> StateResult<()> {
        // Check for reasonable network health
        if self.network_health.active_user_ratio < 10 { // Minimum 10% active users
            return Err(StateValidationError::NetworkRegression);
        }
        
        Ok(())
    }
}

/// Network health metrics for monitoring
#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize)]
pub struct NetworkHealth {
    /// Percentage of active users (0-100)
    pub active_user_ratio: u64,
    /// Average XP level across all users
    pub average_xp_level: u64,
    /// Network growth rate (users per day)
    pub growth_rate: u64,
    /// Mining efficiency (FIN mined per user per day)
    pub mining_efficiency: u64,
    /// Quality score average
    pub average_quality_score: u64,
    /// Bot detection success rate
    pub bot_detection_rate: u64,
    /// Last updated timestamp
    pub last_updated: i64,
}

impl NetworkHealth {
    pub fn new() -> Self {
        Self {
            active_user_ratio: 100,
            average_xp_level: 1,
            growth_rate: 0,
            mining_efficiency: 0,
            average_quality_score: 100,
            bot_detection_rate: 95,
            last_updated: 0,
        }
    }
    
    pub fn update(&mut self, total_users: u64, total_supply: u64) -> StateResult<()> {
        let current_time = Clock::get()?.unix_timestamp;
        
        // Update mining efficiency
        if total_users > 0 {
            self.mining_efficiency = total_supply / total_users;
        }
        
        // Update timestamp
        self.last_updated = current_time;
        
        Ok(())
    }
}

impl Default for NetworkHealth {
    fn default() -> Self {
        Self::new()
    }
}

impl StateValidation for NetworkHealth {
    fn validate(&self) -> StateResult<()> {
        if self.active_user_ratio > 100 {
            return Err(StateValidationError::InvalidNetworkData);
        }
        
        if self.average_quality_score < MIN_QUALITY_SCORE || 
           self.average_quality_score > MAX_QUALITY_SCORE {
            return Err(StateValidationError::InvalidQualityScore);
        }
        
        if self.bot_detection_rate > 100 {
            return Err(StateValidationError::InvalidNetworkData);
        }
        
        utils::validate_timestamp(self.last_updated)?;
        
        Ok(())
    }
    
    fn check_limits(&self) -> StateResult<()> {
        if self.growth_rate > 1_000_000 { // Max 1M users per day
            return Err(StateValidationError::ExceedsMaxLimit);
        }
        
        Ok(())
    }
    
    fn security_check(&self) -> StateResult<()> {
        if self.bot_detection_rate < 80 { // Minimum 80% bot detection rate
            return Err(StateValidationError::BotDetectionFailed);
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_safe_math_operations() {
        let a: u64 = 100;
        let b: u64 = 50;
        
        assert_eq!(a.safe_add(b).unwrap(), 150);
        assert_eq!(a.safe_sub(b).unwrap(), 50);
        assert_eq!(a.safe_mul(b).unwrap(), 5000);
        assert_eq!(a.safe_div(b).unwrap(), 2);
        
        // Test overflow
        let max_val = u64::MAX;
        assert!(max_val.safe_add(1).is_err());
        
        // Test underflow
        let min_val = 0u64;
        assert!(min_val.safe_sub(1).is_err());
        
        // Test division by zero
        assert!(a.safe_div(0).is_err());
    }
    
    #[test]
    fn test_phase_calculations() {
        assert_eq!(utils::get_current_phase(50_000), 1);
        assert_eq!(utils::get_current_phase(500_000), 2);
        assert_eq!(utils::get_current_phase(5_000_000), 3);
        assert_eq!(utils::get_current_phase(50_000_000), 4);
    }
    
    #[test]
    fn test_regression_factor() {
        assert_eq!(utils::calculate_regression_factor(50_000), 100); // No regression
        assert_eq!(utils::calculate_regression_factor(200_000), 90);  // Light regression
        assert!(utils::calculate_regression_factor(10_000_000) < 50); // Heavy regression
    }
    
    #[test]
    fn test_quality_score_calculation() {
        let score = utils::calculate_quality_score(80, 90, 85, 75);
        assert!(score >= MIN_QUALITY_SCORE && score <= MAX_QUALITY_SCORE);
    }
    
    #[test]
    fn test_state_manager_initialization() {
        let manager = StateManager::new();
        assert_eq!(manager.current_phase, 1);
        assert_eq!(manager.total_users, 0);
        assert_eq!(manager.total_fin_supply, 0);
        assert!(manager.validate().is_ok());
    }
    
    #[test]
    fn test_network_health() {
        let health = NetworkHealth::new();
        assert!(health.validate().is_ok());
        assert!(health.check_limits().is_ok());
        assert!(health.security_check().is_ok());
    }
    
    #[test]
    fn test_fin_conversion() {
        let fin_amount = 100;
        let micro_fin = utils::fin_to_micro_fin(fin_amount).unwrap();
        assert_eq!(micro_fin, 100_000_000);
        
        let converted_back = utils::micro_fin_to_fin(micro_fin);
        assert_eq!(converted_back, fin_amount);
    }
}
