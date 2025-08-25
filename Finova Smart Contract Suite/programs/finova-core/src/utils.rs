// programs/finova-core/src/utils.rs

use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount, Mint};
use std::collections::HashMap;
use std::convert::TryInto;

use crate::constants::*;
use crate::errors::FinovaError;

/// Mathematical utilities for mining calculations
pub struct MiningCalculator;

impl MiningCalculator {
    /// Calculate base mining rate based on current phase
    /// Formula: Base_Rate × Phase_Multiplier × Network_Regression
    pub fn calculate_base_mining_rate(
        total_users: u64,
        current_timestamp: i64,
        genesis_timestamp: i64,
    ) -> Result<u64> {
        let phase = Self::determine_mining_phase(total_users);
        let base_rate = Self::get_phase_base_rate(phase);
        let time_decay = Self::calculate_time_decay(current_timestamp, genesis_timestamp);
        
        let final_rate = (base_rate as f64 * time_decay) as u64;
        
        // Ensure minimum rate
        Ok(std::cmp::max(final_rate, MIN_MINING_RATE))
    }

    /// Determine current mining phase based on total users
    pub fn determine_mining_phase(total_users: u64) -> u8 {
        match total_users {
            0..=100_000 => 1,           // Finizen Phase
            100_001..=1_000_000 => 2,   // Growth Phase
            1_000_001..=10_000_000 => 3, // Maturity Phase
            _ => 4,                     // Stability Phase
        }
    }

    /// Get base rate for specific phase (in micro-FIN per hour)
    pub fn get_phase_base_rate(phase: u8) -> u64 {
        match phase {
            1 => 100_000, // 0.1 FIN/hour
            2 => 50_000,  // 0.05 FIN/hour
            3 => 25_000,  // 0.025 FIN/hour
            4 => 10_000,  // 0.01 FIN/hour
            _ => 10_000,
        }
    }

    /// Calculate time-based decay for sustainability
    fn calculate_time_decay(current_timestamp: i64, genesis_timestamp: i64) -> f64 {
        let days_since_genesis = (current_timestamp - genesis_timestamp) / 86400;
        let decay_factor = 0.999995; // Very slow decay
        
        decay_factor.powi(days_since_genesis as i32)
    }

    /// Calculate Finizen bonus based on early adoption
    /// Formula: max(1.0, 2.0 - (Total_Users / 1,000,000))
    pub fn calculate_finizen_bonus(total_users: u64) -> u64 {
        let ratio = total_users as f64 / 1_000_000.0;
        let bonus = (2.0 - ratio).max(1.0);
        
        // Convert to basis points (10000 = 1.0x)
        (bonus * 10000.0) as u64
    }

    /// Calculate referral network bonus
    /// Formula: 1 + (Active_Referrals × 0.1)
    pub fn calculate_referral_bonus(active_referrals: u32) -> u64 {
        let bonus = 1.0 + (active_referrals as f64 * 0.1);
        let capped_bonus = bonus.min(MAX_REFERRAL_MULTIPLIER as f64);
        
        (capped_bonus * 10000.0) as u64
    }

    /// Calculate security bonus for KYC verification
    pub fn calculate_security_bonus(is_kyc_verified: bool) -> u64 {
        if is_kyc_verified {
            12000 // 1.2x multiplier
        } else {
            8000  // 0.8x multiplier
        }
    }

    /// Calculate exponential regression factor to prevent whale dominance
    /// Formula: e^(-0.001 × Total_Holdings)
    pub fn calculate_regression_factor(total_holdings: u64) -> u64 {
        let exponent = -0.001 * (total_holdings as f64 / MICRO_FIN_DECIMALS as f64);
        let factor = exponent.exp();
        
        // Convert to basis points, ensure minimum 1% (100 basis points)
        std::cmp::max((factor * 10000.0) as u64, 100)
    }

    /// Calculate final hourly mining rate with all bonuses and penalties
    pub fn calculate_final_mining_rate(
        base_rate: u64,
        finizen_bonus: u64,
        referral_bonus: u64,
        security_bonus: u64,
        regression_factor: u64,
        xp_multiplier: u64,
        rp_multiplier: u64,
    ) -> Result<u64> {
        // All multipliers are in basis points (10000 = 1.0x)
        let total_multiplier = (finizen_bonus as u128)
            .checked_mul(referral_bonus as u128).ok_or(FinovaError::MathOverflow)?
            .checked_mul(security_bonus as u128).ok_or(FinovaError::MathOverflow)?
            .checked_mul(regression_factor as u128).ok_or(FinovaError::MathOverflow)?
            .checked_mul(xp_multiplier as u128).ok_or(FinovaError::MathOverflow)?
            .checked_mul(rp_multiplier as u128).ok_or(FinovaError::MathOverflow)?;

        // Divide by 10000^5 to convert from basis points (since we multiplied 6 basis point values)
        let divisor = 10000_u128.pow(5);
        let rate = (base_rate as u128)
            .checked_mul(total_multiplier).ok_or(FinovaError::MathOverflow)?
            .checked_div(divisor).ok_or(FinovaError::MathOverflow)?;

        // Cap at maximum rate
        Ok(std::cmp::min(rate as u64, MAX_HOURLY_MINING_RATE))
    }
}

/// XP (Experience Points) calculation utilities
pub struct XPCalculator;

impl XPCalculator {
    /// Calculate XP gained from activity
    /// Formula: Base_XP × Platform_Multiplier × Quality_Score × Streak_Bonus × Level_Progression
    pub fn calculate_xp_gain(
        activity_type: u8,
        platform_type: u8,
        quality_score: u32, // In basis points
        streak_days: u32,
        current_level: u32,
        daily_activity_count: u32,
    ) -> Result<u32> {
        let base_xp = Self::get_base_xp_for_activity(activity_type)?;
        let platform_multiplier = Self::get_platform_multiplier(platform_type);
        let streak_bonus = Self::calculate_streak_bonus(streak_days);
        let level_progression = Self::calculate_level_progression(current_level);
        let daily_cap_factor = Self::apply_daily_activity_cap(activity_type, daily_activity_count);

        let total_xp = (base_xp as u64)
            .checked_mul(platform_multiplier as u64).ok_or(FinovaError::MathOverflow)?
            .checked_mul(quality_score as u64).ok_or(FinovaError::MathOverflow)?
            .checked_mul(streak_bonus as u64).ok_or(FinovaError::MathOverflow)?
            .checked_mul(level_progression as u64).ok_or(FinovaError::MathOverflow)?
            .checked_mul(daily_cap_factor as u64).ok_or(FinovaError::MathOverflow)?;

        // Divide by 10000^4 to convert from basis points
        let divisor = 10000_u64.pow(4);
        let final_xp = total_xp.checked_div(divisor).ok_or(FinovaError::MathOverflow)?;

        Ok(final_xp as u32)
    }

    /// Get base XP for different activity types
    fn get_base_xp_for_activity(activity_type: u8) -> Result<u32> {
        match activity_type {
            1 => Ok(50),   // Original text post
            2 => Ok(75),   // Photo/image post
            3 => Ok(150),  // Video content
            4 => Ok(25),   // Story/status
            5 => Ok(25),   // Meaningful comment
            6 => Ok(5),    // Like/react
            7 => Ok(15),   // Share/repost
            8 => Ok(20),   // Follow/subscribe
            9 => Ok(10),   // Daily login
            10 => Ok(100), // Complete daily quest
            11 => Ok(500), // Achieve milestone
            12 => Ok(1000), // Viral content (1K+ views)
            _ => Err(FinovaError::InvalidActivityType.into()),
        }
    }

    /// Get platform-specific multipliers
    fn get_platform_multiplier(platform_type: u8) -> u32 {
        match platform_type {
            1 => 13000, // TikTok: 1.3x
            2 => 12000, // Instagram: 1.2x
            3 => 14000, // YouTube: 1.4x
            4 => 12000, // X (Twitter): 1.2x
            5 => 11000, // Facebook: 1.1x
            _ => 10000, // Default: 1.0x
        }
    }

    /// Calculate streak bonus
    /// Formula: 1.0x to 3.0x based on consecutive days
    fn calculate_streak_bonus(streak_days: u32) -> u32 {
        let bonus = 1.0 + (streak_days as f64 * 0.05).min(2.0); // Max 3.0x
        (bonus * 10000.0) as u32
    }

    /// Calculate level progression factor for balanced growth
    /// Formula: e^(-0.01 × Current_Level)
    fn calculate_level_progression(current_level: u32) -> u32 {
        let exponent = -0.01 * current_level as f64;
        let factor = exponent.exp();
        
        // Ensure minimum 10% progression (1000 basis points)
        std::cmp::max((factor * 10000.0) as u32, 1000)
    }

    /// Apply daily activity caps to prevent spam
    fn apply_daily_activity_cap(activity_type: u8, daily_count: u32) -> u32 {
        let (daily_limit, penalty_start) = match activity_type {
            2 => (20, 15),  // Photo posts: limit 20, penalty after 15
            3 => (10, 8),   // Video content: limit 10, penalty after 8
            5 => (100, 80), // Comments: limit 100, penalty after 80
            6 => (200, 150), // Likes: limit 200, penalty after 150
            7 => (50, 40),  // Shares: limit 50, penalty after 40
            8 => (25, 20),  // Follows: limit 25, penalty after 20
            _ => return 10000, // No cap for other activities
        };

        if daily_count <= penalty_start {
            10000 // 1.0x - no penalty
        } else if daily_count <= daily_limit {
            // Linear penalty from 1.0x to 0.1x
            let penalty_ratio = (daily_limit - daily_count) as f64 / (daily_limit - penalty_start) as f64;
            let factor = 0.1 + (penalty_ratio * 0.9);
            (factor * 10000.0) as u32
        } else {
            0 // No XP after daily limit
        }
    }

    /// Calculate XP level from total XP
    pub fn calculate_level_from_xp(total_xp: u64) -> u32 {
        match total_xp {
            0..=999 => ((total_xp / 100) + 1) as u32,           // Levels 1-10
            1000..=4999 => ((total_xp - 1000) / 250 + 11) as u32, // Levels 11-25
            5000..=19999 => ((total_xp - 5000) / 600 + 26) as u32, // Levels 26-50
            20000..=49999 => ((total_xp - 20000) / 1200 + 51) as u32, // Levels 51-75
            50000..=99999 => ((total_xp - 50000) / 2000 + 76) as u32, // Levels 76-100
            _ => ((total_xp - 100000) / 5000 + 101) as u32,     // Levels 101+
        }
    }

    /// Get mining multiplier based on XP level
    pub fn get_mining_multiplier_from_level(level: u32) -> u32 {
        match level {
            1..=10 => 10000 + (level - 1) * 200,        // 1.0x to 1.18x
            11..=25 => 11800 + (level - 11) * 334,      // 1.18x to 1.85x
            26..=50 => 18500 + (level - 26) * 240,      // 1.85x to 2.45x
            51..=75 => 24500 + (level - 51) * 280,      // 2.45x to 3.15x
            76..=100 => 31500 + (level - 76) * 340,     // 3.15x to 3.99x
            _ => 40000 + std::cmp::min((level - 101) * 100, 10000), // 4.0x to 5.0x max
        }
    }
}

/// Referral Points (RP) calculation utilities
pub struct RPCalculator;

impl RPCalculator {
    /// Calculate RP value from network activity
    /// Formula: Direct_Referral_Points + Indirect_Network_Points + Network_Quality_Bonus
    pub fn calculate_rp_value(
        direct_referrals: u32,
        l2_network_size: u32,
        l3_network_size: u32,
        network_quality_score: u32, // In basis points
        average_referral_level: u32,
        retention_rate: u32, // In basis points
    ) -> Result<u64> {
        let direct_points = Self::calculate_direct_referral_points(direct_referrals);
        let indirect_points = Self::calculate_indirect_network_points(l2_network_size, l3_network_size);
        let quality_bonus = Self::calculate_network_quality_bonus(
            network_quality_score,
            average_referral_level,
            retention_rate,
        )?;

        let total_rp = direct_points
            .checked_add(indirect_points).ok_or(FinovaError::MathOverflow)?
            .checked_mul(quality_bonus as u64).ok_or(FinovaError::MathOverflow)?
            .checked_div(10000).ok_or(FinovaError::MathOverflow)?; // Convert from basis points

        Ok(total_rp)
    }

    /// Calculate direct referral points
    fn calculate_direct_referral_points(direct_referrals: u32) -> u64 {
        (direct_referrals as u64) * 100 // Base 100 RP per direct referral
    }

    /// Calculate indirect network points (L2: 30%, L3: 10%)
    fn calculate_indirect_network_points(l2_network_size: u32, l3_network_size: u32) -> u64 {
        let l2_points = (l2_network_size as u64) * 30; // 30 RP per L2 referral
        let l3_points = (l3_network_size as u64) * 10; // 10 RP per L3 referral
        
        l2_points + l3_points
    }

    /// Calculate network quality bonus multiplier
    fn calculate_network_quality_bonus(
        network_quality_score: u32,
        average_referral_level: u32,
        retention_rate: u32,
    ) -> Result<u32> {
        let quality_factor = network_quality_score as f64 / 10000.0; // Convert from basis points
        let level_factor = (average_referral_level as f64).min(50.0) / 50.0; // Cap at level 50
        let retention_factor = retention_rate as f64 / 10000.0; // Convert from basis points

        let bonus = quality_factor * level_factor * retention_factor * 20.0; // Scale factor
        let capped_bonus = (1.0 + bonus).min(50.0); // Cap at 50x multiplier

        Ok((capped_bonus * 10000.0) as u32)
    }

    /// Get RP tier based on total RP
    pub fn get_rp_tier(total_rp: u64) -> u8 {
        match total_rp {
            0..=999 => 1,           // Explorer
            1000..=4999 => 2,       // Connector
            5000..=14999 => 3,      // Influencer
            15000..=49999 => 4,     // Leader
            _ => 5,                 // Ambassador
        }
    }

    /// Get mining multiplier based on RP tier
    pub fn get_mining_multiplier_from_rp_tier(tier: u8) -> u32 {
        match tier {
            1 => 10000, // Explorer: 1.0x
            2 => 12000, // Connector: 1.2x
            3 => 15000, // Influencer: 1.5x
            4 => 20000, // Leader: 2.0x
            5 => 30000, // Ambassador: 3.0x
            _ => 10000,
        }
    }

    /// Calculate network regression factor for quality control
    pub fn calculate_network_regression_factor(
        total_network_size: u32,
        network_quality_score: u32,
    ) -> u32 {
        let size_factor = total_network_size as f64;
        let quality_factor = network_quality_score as f64 / 10000.0;
        
        let exponent = -0.0001 * size_factor * quality_factor;
        let regression = exponent.exp();
        
        // Ensure minimum 5% effectiveness (500 basis points)
        std::cmp::max((regression * 10000.0) as u32, 500)
    }
}

/// Quality assessment utilities
pub struct QualityAssessment;

impl QualityAssessment {
    /// Calculate content quality score based on multiple factors
    pub fn calculate_quality_score(
        originality_score: u32,      // 0-10000 basis points
        engagement_potential: u32,   // 0-10000 basis points
        platform_relevance: u32,     // 0-10000 basis points
        brand_safety: u32,           // 0-10000 basis points
        human_generated: u32,        // 0-10000 basis points
    ) -> u32 {
        let weights = [
            (originality_score, 3000),      // 30% weight
            (engagement_potential, 2500),   // 25% weight
            (platform_relevance, 2000),    // 20% weight
            (brand_safety, 1500),           // 15% weight
            (human_generated, 1000),        // 10% weight
        ];

        let weighted_sum = weights.iter()
            .map(|(score, weight)| (score * weight) / 10000)
            .sum::<u32>();

        // Clamp between 0.5x and 2.0x (5000 to 20000 basis points)
        std::cmp::max(std::cmp::min(weighted_sum, 20000), 5000)
    }

    /// Detect suspicious activity patterns
    pub fn detect_suspicious_patterns(
        click_speed_variance: f64,
        session_pattern_score: u32,
        content_uniqueness: u32,
        temporal_patterns: u32,
    ) -> u32 {
        let mut suspicion_score = 0u32;

        // Check click speed variance (human-like should have variance)
        if click_speed_variance < 0.1 {
            suspicion_score += 2500; // Very suspicious
        } else if click_speed_variance < 0.3 {
            suspicion_score += 1000; // Somewhat suspicious
        }

        // Check session patterns
        if session_pattern_score < 3000 { // Less than 30% human-like
            suspicion_score += 2000;
        }

        // Check content uniqueness
        if content_uniqueness < 5000 { // Less than 50% unique
            suspicion_score += 1500;
        }

        // Check temporal patterns
        if temporal_patterns < 4000 { // Less than 40% natural
            suspicion_score += 1000;
        }

        // Return inverted score (lower suspicion = higher human probability)
        10000_u32.saturating_sub(suspicion_score)
    }
}

/// Time-based utilities
pub struct TimeUtils;

impl TimeUtils {
    /// Check if two timestamps are within the same day (UTC)
    pub fn is_same_day(timestamp1: i64, timestamp2: i64) -> bool {
        let day1 = timestamp1 / 86400;
        let day2 = timestamp2 / 86400;
        day1 == day2
    }

    /// Calculate hours between two timestamps
    pub fn hours_between(start_timestamp: i64, end_timestamp: i64) -> u64 {
        ((end_timestamp - start_timestamp) / 3600) as u64
    }

    /// Get current hour of day (0-23)
    pub fn get_hour_of_day(timestamp: i64) -> u8 {
        ((timestamp % 86400) / 3600) as u8
    }

    /// Check if timestamp is within business hours (configurable)
    pub fn is_business_hours(timestamp: i64, start_hour: u8, end_hour: u8) -> bool {
        let hour = Self::get_hour_of_day(timestamp);
        hour >= start_hour && hour <= end_hour
    }

    /// Calculate streak days from last activity
    pub fn calculate_streak_days(
        current_timestamp: i64,
        last_activity_timestamp: i64,
        previous_streak: u32,
    ) -> u32 {
        if Self::is_same_day(current_timestamp, last_activity_timestamp) {
            // Same day, return existing streak
            previous_streak
        } else {
            let days_diff = (current_timestamp - last_activity_timestamp) / 86400;
            
            if days_diff == 1 {
                // Consecutive day, increment streak
                previous_streak + 1
            } else {
                // Streak broken, reset to 1
                1
            }
        }
    }
}

/// Security utilities
pub struct SecurityUtils;

impl SecurityUtils {
    /// Validate wallet signature for anti-bot measures
    pub fn validate_signature(
        message: &[u8],
        signature: &[u8],
        public_key: &Pubkey,
    ) -> bool {
        // Implementation would use ed25519 signature verification
        // For now, return true (actual implementation needed)
        true
    }

    /// Rate limiting check
    pub fn check_rate_limit(
        user_pubkey: &Pubkey,
        action_type: u8,
        current_timestamp: i64,
        last_action_timestamp: i64,
    ) -> Result<bool> {
        let min_interval = match action_type {
            1 => 60,    // Mining: 1 minute
            2 => 10,    // Social post: 10 seconds
            3 => 5,     // Like/comment: 5 seconds
            4 => 300,   // Referral: 5 minutes
            _ => 1,     // Default: 1 second
        };

        let time_diff = current_timestamp - last_action_timestamp;
        
        if time_diff < min_interval {
            return Err(FinovaError::RateLimitExceeded.into());
        }

        Ok(true)
    }

    /// Generate pseudo-random number using timestamp and user data
    pub fn generate_pseudo_random(
        seed: u64,
        user_pubkey: &Pubkey,
        timestamp: i64,
    ) -> u64 {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        std::hash::Hasher::write(&mut hasher, &seed.to_le_bytes());
        std::hash::Hasher::write(&mut hasher, user_pubkey.as_ref());
        std::hash::Hasher::write(&mut hasher, &timestamp.to_le_bytes());
        
        std::hash::Hasher::finish(&hasher)
    }
}

/// Token utilities
pub struct TokenUtils;

impl TokenUtils {
    /// Convert human-readable FIN amount to micro-FIN
    pub fn fin_to_micro_fin(fin_amount: f64) -> u64 {
        (fin_amount * MICRO_FIN_DECIMALS as f64) as u64
    }

    /// Convert micro-FIN to human-readable FIN amount
    pub fn micro_fin_to_fin(micro_fin_amount: u64) -> f64 {
        micro_fin_amount as f64 / MICRO_FIN_DECIMALS as f64
    }

    /// Validate token account ownership
    pub fn validate_token_account_owner(
        token_account: &Account<TokenAccount>,
        expected_owner: &Pubkey,
    ) -> Result<()> {
        if token_account.owner != *expected_owner {
            return Err(FinovaError::InvalidTokenAccountOwner.into());
        }
        Ok(())
    }

    /// Calculate token transfer amount with fees
    pub fn calculate_transfer_with_fees(
        amount: u64,
        fee_basis_points: u32,
    ) -> Result<(u64, u64)> {
        let fee = amount
            .checked_mul(fee_basis_points as u64).ok_or(FinovaError::MathOverflow)?
            .checked_div(10000).ok_or(FinovaError::MathOverflow)?;
        
        let net_amount = amount
            .checked_sub(fee).ok_or(FinovaError::InsufficientFunds)?;
        
        Ok((net_amount, fee))
    }
}

/// General utilities
pub struct GeneralUtils;

impl GeneralUtils {
    /// Convert string to fixed-size array for storage
    pub fn string_to_fixed_array<const N: usize>(s: &str) -> [u8; N] {
        let mut array = [0u8; N];
        let bytes = s.as_bytes();
        let len = std::cmp::min(bytes.len(), N);
        array[..len].copy_from_slice(&bytes[..len]);
        array
    }

    /// Convert fixed-size array back to string
    pub fn fixed_array_to_string<const N: usize>(array: &[u8; N]) -> String {
        let end = array.iter().position(|&b| b == 0).unwrap_or(N);
        String::from_utf8_lossy(&array[..end]).to_string()
    }

    /// Validate pubkey is not default/empty
    pub fn validate_pubkey_not_default(pubkey: &Pubkey) -> Result<()> {
        if *pubkey == Pubkey::default() {
            return Err(FinovaError::InvalidPubkey.into());
        }
        Ok(())
    }

    /// Calculate percentage with basis points precision
    pub fn calculate_percentage(value: u64, percentage_bp: u32) -> Result<u64> {
        value
            .checked_mul(percentage_bp as u64).ok_or(FinovaError::MathOverflow)?
            .checked_div(10000).ok_or(FinovaError::MathOverflow)
            .map_err(|_| FinovaError::MathOverflow.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mining_phase_determination() {
        assert_eq!(MiningCalculator::determine_mining_phase(50_000), 1);
        assert_eq!(MiningCalculator::determine_mining_phase(500_000), 2);
        assert_eq!(MiningCalculator::determine_mining_phase(5_000_000), 3);
        assert_eq!(MiningCalculator::determine_mining_phase(50_000_000), 4);
    }

    #[test]
    fn test_xp_level_calculation() {
        assert_eq!(XPCalculator::calculate_level_from_xp(500), 6);
        assert_eq!(XPCalculator::calculate_level_from_xp(2500), 17);
        assert_eq!(XPCalculator::calculate_level_from_xp(25000), 38);
    }

    #[test]
    fn test_rp_tier_calculation() {
        assert_eq!(RPCalculator::get_rp_tier(500), 1);
        assert_eq!(RPCalculator::get_rp_tier(2500), 2);
        assert_eq!(RPCalculator::get_rp_tier(10000), 3);
        assert_eq!(RPCalculator::get_rp_tier(25000), 4);
        assert_eq!(RPCalculator::get_rp_tier(100000), 5);
    }

    #[test]
    fn test_time_utilities() {
        let timestamp1 = 1627776000; // 2021-08-01 00:00:00 UTC
        let timestamp2 = 1627862400; // 2021-08-02 00:00:00 UTC
        
        assert!(!TimeUtils::is_same_day(timestamp1, timestamp2));
        assert_eq!(TimeUtils::hours_between(timestamp1, timestamp2), 24);
    }

    #[test]
    fn test_token_conversions() {
        assert_eq!(TokenUtils::fin_to_micro_fin(1.0), 1_000_000);
        assert_eq!(TokenUtils::micro_fin_to_fin(1_000_000), 1.0);
    }
}
