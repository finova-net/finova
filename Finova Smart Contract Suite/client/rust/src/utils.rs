// finova-net/finova/client/rust/src/utils.rs

use anchor_client::solana_sdk::{
    commitment_config::CommitmentConfig,
    pubkey::Pubkey,
    signature::{Keypair, Signature},
    signer::Signer,
    system_instruction,
    transaction::Transaction,
};
use anchor_lang::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::str::FromStr;
use thiserror::Error;

/// Custom error types for Finova client operations
#[derive(Error, Debug)]
pub enum FinovaError {
    #[error("Invalid public key: {0}")]
    InvalidPubkey(String),
    #[error("Calculation error: {0}")]
    CalculationError(String),
    #[error("Network error: {0}")]
    NetworkError(String),
    #[error("Insufficient balance: required {required}, available {available}")]
    InsufficientBalance { required: u64, available: u64 },
    #[error("Invalid mining phase: {0}")]
    InvalidMiningPhase(u8),
    #[error("Anti-bot validation failed: score {0}")]
    AntiBotFailed(f64),
}

/// Mining phases with exponential regression
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MiningPhase {
    Finizen,    // 0-100K users
    Growth,     // 100K-1M users  
    Maturity,   // 1M-10M users
    Stability,  // 10M+ users
}

impl MiningPhase {
    pub fn from_user_count(count: u64) -> Self {
        match count {
            0..=100_000 => Self::Finizen,
            100_001..=1_000_000 => Self::Growth,
            1_000_001..=10_000_000 => Self::Maturity,
            _ => Self::Stability,
        }
    }

    pub fn base_rate(&self) -> f64 {
        match self {
            Self::Finizen => 0.1,
            Self::Growth => 0.05,
            Self::Maturity => 0.025,
            Self::Stability => 0.01,
        }
    }

    pub fn finizen_bonus(&self, total_users: u64) -> f64 {
        match self {
            Self::Finizen => 2.0,
            Self::Growth => (2.0 - (total_users as f64 / 1_000_000.0)).max(1.0),
            Self::Maturity => 1.2,
            Self::Stability => 1.0,
        }
    }
}

/// User level and XP calculations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserStats {
    pub xp: u64,
    pub level: u32,
    pub rp: u64,
    pub total_fin: u64,
    pub staked_fin: u64,
    pub referral_count: u32,
    pub active_referrals: u32,
    pub kyc_verified: bool,
    pub last_activity: i64,
    pub streak_days: u32,
}

impl UserStats {
    pub fn new() -> Self {
        Self {
            xp: 0,
            level: 1,
            rp: 0,
            total_fin: 0,
            staked_fin: 0,
            referral_count: 0,
            active_referrals: 0,
            kyc_verified: false,
            last_activity: 0,
            streak_days: 0,
        }
    }

    pub fn calculate_level(&self) -> u32 {
        // XP level calculation with exponential requirements
        match self.xp {
            0..=999 => (self.xp / 100 + 1).min(10) as u32,
            1_000..=4_999 => 11 + ((self.xp - 1_000) / 250) as u32,
            5_000..=19_999 => 26 + ((self.xp - 5_000) / 600) as u32,
            20_000..=49_999 => 51 + ((self.xp - 20_000) / 1_200) as u32,
            50_000..=99_999 => 76 + ((self.xp - 50_000) / 2_000) as u32,
            _ => 101 + ((self.xp - 100_000) / 5_000) as u32,
        }
    }

    pub fn get_xp_multiplier(&self) -> f64 {
        let level = self.calculate_level();
        match level {
            1..=10 => 1.0 + (level as f64 * 0.02),
            11..=25 => 1.3 + ((level - 10) as f64 * 0.033),
            26..=50 => 1.9 + ((level - 25) as f64 * 0.024),
            51..=75 => 2.6 + ((level - 50) as f64 * 0.024),
            76..=100 => 3.3 + ((level - 75) as f64 * 0.028),
            _ => 4.1 + ((level - 100) as f64 * 0.018),
        }
    }

    pub fn get_rp_tier(&self) -> u32 {
        match self.rp {
            0..=999 => 0,
            1_000..=4_999 => 1,
            5_000..=14_999 => 2,
            15_000..=49_999 => 3,
            _ => 4,
        }
    }

    pub fn get_rp_multiplier(&self) -> f64 {
        match self.get_rp_tier() {
            0 => 1.0,      // Explorer
            1 => 1.2,      // Connector  
            2 => 1.5,      // Influencer
            3 => 2.0,      // Leader
            4 => 3.0,      // Ambassador
            _ => 1.0,
        }
    }
}

/// Mining calculations with exponential regression
pub struct MiningCalculator;

impl MiningCalculator {
    /// Calculate hourly mining rate with all bonuses and regression
    pub fn calculate_mining_rate(
        user_stats: &UserStats,
        total_users: u64,
        network_quality: f64,
    ) -> Result<f64, FinovaError> {
        let phase = MiningPhase::from_user_count(total_users);
        let base_rate = phase.base_rate();
        let finizen_bonus = phase.finizen_bonus(total_users);
        
        // Referral bonus calculation
        let referral_bonus = 1.0 + (user_stats.active_referrals as f64 * 0.1);
        
        // Security bonus for KYC verification
        let security_bonus = if user_stats.kyc_verified { 1.2 } else { 0.8 };
        
        // Anti-whale exponential regression
        let regression_factor = (-0.001 * user_stats.total_fin as f64).exp();
        
        // XP level multiplier
        let xp_multiplier = user_stats.get_xp_multiplier();
        
        // RP tier multiplier
        let rp_multiplier = user_stats.get_rp_multiplier();
        
        // Streak bonus
        let streak_bonus = 1.0 + (user_stats.streak_days as f64 * 0.02).min(1.0);
        
        // Quality score based on network behavior
        let quality_score = network_quality.clamp(0.5, 2.0);
        
        let final_rate = base_rate 
            * finizen_bonus 
            * referral_bonus 
            * security_bonus 
            * regression_factor
            * xp_multiplier
            * rp_multiplier
            * streak_bonus 
            * quality_score;
            
        Ok(final_rate.max(0.001)) // Minimum rate
    }

    /// Calculate XP gain from activity
    pub fn calculate_xp_gain(
        activity_type: &str,
        platform: &str,
        quality_score: f64,
        user_stats: &UserStats,
    ) -> Result<u64, FinovaError> {
        let base_xp = match activity_type {
            "post" => 50,
            "photo" => 75,
            "video" => 150,
            "story" => 25,
            "comment" => 25,
            "like" => 5,
            "share" => 15,
            "follow" => 20,
            "login" => 10,
            "quest" => 100,
            "viral" => 1000,
            _ => 0,
        };

        let platform_multiplier = match platform {
            "tiktok" => 1.3,
            "instagram" => 1.2,
            "youtube" => 1.4,
            "facebook" => 1.1,
            "twitter" => 1.2,
            _ => 1.0,
        };

        let quality_multiplier = quality_score.clamp(0.5, 2.0);
        let streak_bonus = 1.0 + (user_stats.streak_days as f64 * 0.02).min(1.0);
        let level_progression = (-0.01 * user_stats.level as f64).exp();

        let final_xp = (base_xp as f64 
            * platform_multiplier 
            * quality_multiplier 
            * streak_bonus 
            * level_progression) as u64;

        Ok(final_xp)
    }

    /// Calculate RP from referral network activity
    pub fn calculate_rp_gain(
        referral_stats: &HashMap<Pubkey, UserStats>,
        network_quality: f64,
    ) -> Result<u64, FinovaError> {
        let mut total_rp = 0u64;
        
        // Direct referral points
        for (_, referral) in referral_stats.iter() {
            let activity_points = (referral.xp as f64 * 0.05) as u64;
            let retention_bonus = if referral.last_activity > 0 { 1.2 } else { 0.8 };
            total_rp += (activity_points as f64 * retention_bonus) as u64;
        }
        
        // Network quality bonus
        let quality_multiplier = network_quality.clamp(0.1, 2.0);
        total_rp = (total_rp as f64 * quality_multiplier) as u64;
        
        // Network size regression to prevent farming
        let network_size = referral_stats.len() as f64;
        let regression_factor = (-0.0001 * network_size * network_quality).exp();
        total_rp = (total_rp as f64 * regression_factor) as u64;
        
        Ok(total_rp)
    }
}

/// Staking calculations and utilities
pub struct StakingCalculator;

impl StakingCalculator {
    /// Calculate staking APY based on amount and user stats
    pub fn calculate_apy(stake_amount: u64, user_stats: &UserStats) -> f64 {
        let base_apy = match stake_amount {
            100..=499 => 0.08,      // 8%
            500..=999 => 0.10,      // 10%
            1_000..=4_999 => 0.12,  // 12%
            5_000..=9_999 => 0.14,  // 14%
            _ => 0.15,              // 15%
        };

        // XP level bonus
        let xp_bonus = user_stats.get_xp_multiplier() * 0.1;
        
        // RP tier bonus
        let rp_bonus = user_stats.get_rp_multiplier() * 0.05;
        
        // Loyalty bonus (mock calculation - would use staking duration)
        let loyalty_bonus = 0.02;
        
        base_apy + xp_bonus + rp_bonus + loyalty_bonus
    }

    /// Calculate staking rewards with compound bonuses
    pub fn calculate_staking_rewards(
        stake_amount: u64,
        duration_days: u32,
        user_stats: &UserStats,
    ) -> u64 {
        let apy = Self::calculate_apy(stake_amount, user_stats);
        let daily_rate = apy / 365.0;
        
        // Compound interest calculation
        let compound_factor = (1.0 + daily_rate).powf(duration_days as f64);
        let final_amount = stake_amount as f64 * compound_factor;
        
        (final_amount - stake_amount as f64) as u64
    }
}

/// NFT and Special Card utilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecialCard {
    pub card_type: String,
    pub rarity: String,
    pub effect: String,
    pub duration_hours: u32,
    pub multiplier: f64,
    pub price_fin: u64,
    pub is_active: bool,
}

impl SpecialCard {
    pub fn get_mining_cards() -> Vec<Self> {
        vec![
            Self {
                card_type: "double_mining".to_string(),
                rarity: "common".to_string(),
                effect: "+100% mining rate".to_string(),
                duration_hours: 24,
                multiplier: 2.0,
                price_fin: 50,
                is_active: false,
            },
            Self {
                card_type: "triple_mining".to_string(),
                rarity: "rare".to_string(),
                effect: "+200% mining rate".to_string(),
                duration_hours: 12,
                multiplier: 3.0,
                price_fin: 150,
                is_active: false,
            },
            Self {
                card_type: "mining_frenzy".to_string(),
                rarity: "epic".to_string(),
                effect: "+500% mining rate".to_string(),
                duration_hours: 4,
                multiplier: 6.0,
                price_fin: 500,
                is_active: false,
            },
        ]
    }

    pub fn calculate_synergy_bonus(active_cards: &[Self]) -> f64 {
        if active_cards.is_empty() {
            return 1.0;
        }

        let mut bonus = 1.0 + (active_cards.len() as f64 * 0.1);
        
        // Rarity bonus
        for card in active_cards {
            let rarity_bonus = match card.rarity.as_str() {
                "common" => 0.0,
                "uncommon" => 0.05,
                "rare" => 0.10,
                "epic" => 0.20,
                "legendary" => 0.35,
                _ => 0.0,
            };
            bonus += rarity_bonus;
        }
        
        // Type match bonus (simplified)
        let unique_types: std::collections::HashSet<_> = 
            active_cards.iter().map(|c| &c.card_type).collect();
        if unique_types.len() >= 3 {
            bonus += 0.30; // All categories active
        }
        
        bonus
    }
}

/// Anti-bot and quality assessment utilities
pub struct AntiBotSystem;

impl AntiBotSystem {
    /// Calculate human probability score
    pub fn calculate_human_score(
        click_patterns: &[f64],
        session_duration: u64,
        content_uniqueness: f64,
        social_connections: u32,
    ) -> f64 {
        let mut score = 0.0;
        
        // Click speed variance (human-like patterns)
        if !click_patterns.is_empty() {
            let mean = click_patterns.iter().sum::<f64>() / click_patterns.len() as f64;
            let variance = click_patterns.iter()
                .map(|x| (x - mean).powi(2))
                .sum::<f64>() / click_patterns.len() as f64;
            
            // Humans have natural variance, bots are too consistent
            let variance_score = (variance * 10.0).min(1.0);
            score += variance_score * 0.25;
        }
        
        // Session duration naturalness
        let session_score = match session_duration {
            0..=300 => 0.5,      // Too short
            301..=3600 => 1.0,   // Natural
            3601..=14400 => 0.8, // Long but possible
            _ => 0.3,            // Suspicious
        };
        score += session_score * 0.25;
        
        // Content uniqueness
        score += content_uniqueness.clamp(0.0, 1.0) * 0.3;
        
        // Social connections naturalness
        let social_score = match social_connections {
            0..=5 => 0.3,        // Very few connections
            6..=50 => 1.0,       // Natural range
            51..=200 => 0.8,     // Many but possible
            _ => 0.2,            // Suspicious
        };
        score += social_score * 0.2;
        
        score.clamp(0.1, 1.0)
    }

    /// Apply anti-bot penalties
    pub fn apply_bot_penalties(base_reward: f64, human_score: f64) -> Result<f64, FinovaError> {
        if human_score < 0.3 {
            return Err(FinovaError::AntiBotFailed(human_score));
        }
        
        let penalty_factor = if human_score < 0.7 {
            human_score // Linear penalty below 0.7
        } else {
            1.0 // No penalty for high human scores
        };
        
        Ok(base_reward * penalty_factor)
    }
}

/// Utility functions for common operations
pub mod utils {
    use super::*;

    /// Convert lamports to FIN tokens (assuming 1 FIN = 1e9 lamports)
    pub fn lamports_to_fin(lamports: u64) -> f64 {
        lamports as f64 / 1_000_000_000.0
    }

    /// Convert FIN tokens to lamports
    pub fn fin_to_lamports(fin: f64) -> u64 {
        (fin * 1_000_000_000.0) as u64
    }

    /// Format FIN amount for display
    pub fn format_fin_amount(amount: u64) -> String {
        let fin_amount = lamports_to_fin(amount);
        format!("{:.6} FIN", fin_amount)
    }

    /// Validate Solana public key
    pub fn validate_pubkey(key_str: &str) -> Result<Pubkey, FinovaError> {
        Pubkey::from_str(key_str)
            .map_err(|_| FinovaError::InvalidPubkey(key_str.to_string()))
    }

    /// Calculate time until next mining claim (24-hour cooldown)
    pub fn time_until_next_claim(last_claim: i64) -> i64 {
        let now = chrono::Utc::now().timestamp();
        let next_claim = last_claim + 86400; // 24 hours
        (next_claim - now).max(0)
    }

    /// Generate referral code from public key
    pub fn generate_referral_code(pubkey: &Pubkey) -> String {
        let key_str = pubkey.to_string();
        format!("FIN{}", &key_str[..8].to_uppercase())
    }

    /// Calculate network growth rate
    pub fn calculate_network_growth(
        current_users: u64,
        previous_users: u64,
        days_elapsed: u32,
    ) -> f64 {
        if previous_users == 0 || days_elapsed == 0 {
            return 0.0;
        }
        
        let growth_rate = (current_users as f64 / previous_users as f64 - 1.0) 
            * (365.0 / days_elapsed as f64);
        growth_rate * 100.0 // Return as percentage
    }

    /// Estimate time to next level
    pub fn estimate_time_to_level(
        current_xp: u64,
        daily_xp_rate: u64,
        target_level: u32,
    ) -> Option<u32> {
        let target_xp = match target_level {
            1..=10 => (target_level - 1) as u64 * 100,
            11..=25 => 1000 + (target_level - 11) as u64 * 250,
            26..=50 => 5000 + (target_level - 26) as u64 * 600,
            51..=75 => 20000 + (target_level - 51) as u64 * 1200,
            76..=100 => 50000 + (target_level - 76) as u64 * 2000,
            _ => 100000 + (target_level - 101) as u64 * 5000,
        };
        
        if current_xp >= target_xp || daily_xp_rate == 0 {
            return None;
        }
        
        let days_needed = (target_xp - current_xp) / daily_xp_rate;
        Some(days_needed as u32)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mining_calculation() {
        let user_stats = UserStats {
            xp: 1000,
            level: 11,
            total_fin: 500,
            active_referrals: 5,
            kyc_verified: true,
            streak_days: 7,
            ..UserStats::new()
        };
        
        let rate = MiningCalculator::calculate_mining_rate(&user_stats, 50000, 0.8)
            .expect("Mining calculation should succeed");
        
        assert!(rate > 0.0);
        assert!(rate < 1.0); // Reasonable upper bound
    }

    #[test]
    fn test_xp_calculation() {
        let user_stats = UserStats {
            level: 15,
            streak_days: 5,
            ..UserStats::new()
        };
        
        let xp = MiningCalculator::calculate_xp_gain("video", "tiktok", 1.5, &user_stats)
            .expect("XP calculation should succeed");
        
        assert!(xp > 0);
    }

    #[test]
    fn test_human_score() {
        let click_patterns = vec![0.2, 0.3, 0.25, 0.4, 0.18];
        let score = AntiBotSystem::calculate_human_score(&click_patterns, 1800, 0.8, 25);
        
        assert!(score >= 0.1 && score <= 1.0);
    }

    #[test]
    fn test_utility_functions() {
        let lamports = 1_000_000_000u64;
        let fin = utils::lamports_to_fin(lamports);
        assert_eq!(fin, 1.0);
        
        let back_to_lamports = utils::fin_to_lamports(fin);
        assert_eq!(back_to_lamports, lamports);
    }
}
