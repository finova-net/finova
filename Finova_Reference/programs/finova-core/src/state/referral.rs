// programs/finova-core/src/state/referral.rs

use anchor_lang::prelude::*;
use crate::constants::*;
use crate::errors::FinovaError;

/// Referral network state account
/// Manages the complete referral tree and network effects
#[account]
#[derive(Debug)]
pub struct ReferralAccount {
    /// Authority of this referral account (user who owns it)
    pub authority: Pubkey,
    
    /// Referrer who invited this user (None for root users)
    pub referrer: Option<Pubkey>,
    
    /// Total number of direct referrals
    pub direct_referral_count: u32,
    
    /// Total number of indirect referrals (L2 + L3)
    pub indirect_referral_count: u32,
    
    /// Active referrals (active in last 30 days)
    pub active_referral_count: u32,
    
    /// Total RP (Referral Points) accumulated
    pub total_rp: u64,
    
    /// Available RP for rewards
    pub available_rp: u64,
    
    /// Current RP tier (0-4: Explorer, Connector, Influencer, Leader, Ambassador)
    pub rp_tier: u8,
    
    /// Network quality score (0-10000, representing 0.00% to 100.00%)
    pub network_quality_score: u16,
    
    /// Last activity timestamp
    pub last_activity: i64,
    
    /// Registration timestamp
    pub created_at: i64,
    
    /// Network statistics
    pub network_stats: NetworkStats,
    
    /// Tier benefits and multipliers
    pub tier_benefits: TierBenefits,
    
    /// Anti-abuse metrics
    pub abuse_metrics: AbuseMetrics,
    
    /// Reserved space for future upgrades
    pub reserved: [u8; 64],
}

/// Network statistics for referral system
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct NetworkStats {
    /// Total network size (all levels)
    pub total_network_size: u32,
    
    /// Level 2 network size
    pub l2_network_size: u32,
    
    /// Level 3 network size
    pub l3_network_size: u32,
    
    /// Total network value generated
    pub total_network_value: u64,
    
    /// Average referral level
    pub average_referral_level: u16, // Scaled by 100 (e.g., 1500 = 15.00)
    
    /// Network retention rate (percentage * 100)
    pub retention_rate: u16,
    
    /// Lifetime referral rewards earned
    pub lifetime_rewards: u64,
    
    /// Current monthly rewards
    pub monthly_rewards: u64,
}

/// Tier benefits and multipliers
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct TierBenefits {
    /// Mining bonus percentage (scaled by 100)
    pub mining_bonus: u16,
    
    /// Referral bonus percentage (scaled by 100)
    pub referral_bonus: u16,
    
    /// Maximum network cap
    pub network_cap: u32,
    
    /// Special benefits unlocked
    pub special_benefits: u64, // Bitmask for various benefits
    
    /// Governance voting power
    pub voting_power: u32,
    
    /// Tier upgrade timestamp
    pub tier_upgraded_at: i64,
}

/// Anti-abuse tracking metrics
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct AbuseMetrics {
    /// Suspicious activity score (0-10000)
    pub suspicious_score: u16,
    
    /// Circular referral detection
    pub circular_referral_count: u16,
    
    /// Rapid registration flags
    pub rapid_registration_flags: u16,
    
    /// Bot probability score (0-10000)
    pub bot_probability: u16,
    
    /// Last audit timestamp
    pub last_audit: i64,
    
    /// Penalty applied
    pub penalty_applied: bool,
    
    /// Penalty end timestamp
    pub penalty_end: i64,
}

/// Individual referral connection
#[account]
#[derive(Debug)]
pub struct ReferralConnection {
    /// Referrer (who invited)
    pub referrer: Pubkey,
    
    /// Referee (who was invited)
    pub referee: Pubkey,
    
    /// Connection level (1 = direct, 2 = L2, 3 = L3)
    pub level: u8,
    
    /// Connection status
    pub status: ReferralStatus,
    
    /// Timestamp when connection was created
    pub created_at: i64,
    
    /// Timestamp when referee completed KYC
    pub kyc_completed_at: Option<i64>,
    
    /// Total rewards earned from this connection
    pub total_rewards_earned: u64,
    
    /// Last activity from referee
    pub last_referee_activity: i64,
    
    /// Referee's contribution to network quality
    pub quality_contribution: u16,
    
    /// Reserved space
    pub reserved: [u8; 32],
}

/// Referral status enumeration
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum ReferralStatus {
    /// Pending - just registered with referral code
    Pending,
    /// Active - completed KYC and actively mining
    Active,
    /// Inactive - not active in last 30 days
    Inactive,
    /// Suspended - flagged for suspicious activity
    Suspended,
    /// Churned - inactive for more than 90 days
    Churned,
}

/// RP tier enumeration with associated benefits
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum RPTier {
    Explorer,      // 0-999 RP
    Connector,     // 1,000-4,999 RP
    Influencer,    // 5,000-14,999 RP
    Leader,        // 15,000-49,999 RP
    Ambassador,    // 50,000+ RP
}

impl ReferralAccount {
    pub const LEN: usize = 8 + // discriminator
        32 + // authority
        33 + // referrer (Option<Pubkey>)
        4 + // direct_referral_count
        4 + // indirect_referral_count
        4 + // active_referral_count
        8 + // total_rp
        8 + // available_rp
        1 + // rp_tier
        2 + // network_quality_score
        8 + // last_activity
        8 + // created_at
        NetworkStats::LEN + // network_stats
        TierBenefits::LEN + // tier_benefits
        AbuseMetrics::LEN + // abuse_metrics
        64; // reserved

    /// Initialize a new referral account
    pub fn initialize(
        &mut self,
        authority: Pubkey,
        referrer: Option<Pubkey>,
        current_timestamp: i64,
    ) -> Result<()> {
        self.authority = authority;
        self.referrer = referrer;
        self.direct_referral_count = 0;
        self.indirect_referral_count = 0;
        self.active_referral_count = 0;
        self.total_rp = 0;
        self.available_rp = 0;
        self.rp_tier = 0; // Explorer
        self.network_quality_score = DEFAULT_NETWORK_QUALITY;
        self.last_activity = current_timestamp;
        self.created_at = current_timestamp;
        
        // Initialize network stats
        self.network_stats = NetworkStats {
            total_network_size: 0,
            l2_network_size: 0,
            l3_network_size: 0,
            total_network_value: 0,
            average_referral_level: 100, // 1.00
            retention_rate: 10000, // 100%
            lifetime_rewards: 0,
            monthly_rewards: 0,
        };
        
        // Initialize tier benefits
        self.tier_benefits = TierBenefits {
            mining_bonus: 0, // 0% for Explorer
            referral_bonus: 1000, // 10% for Explorer
            network_cap: 10, // 10 referrals for Explorer
            special_benefits: 0,
            voting_power: 1,
            tier_upgraded_at: current_timestamp,
        };
        
        // Initialize abuse metrics
        self.abuse_metrics = AbuseMetrics {
            suspicious_score: 0,
            circular_referral_count: 0,
            rapid_registration_flags: 0,
            bot_probability: 0,
            last_audit: current_timestamp,
            penalty_applied: false,
            penalty_end: 0,
        };
        
        self.reserved = [0; 64];
        
        Ok(())
    }

    /// Add a new direct referral
    pub fn add_direct_referral(&mut self, current_timestamp: i64) -> Result<()> {
        // Check network cap
        if self.direct_referral_count >= self.get_network_cap() {
            return Err(FinovaError::NetworkCapExceeded.into());
        }
        
        self.direct_referral_count += 1;
        self.active_referral_count += 1;
        self.network_stats.total_network_size += 1;
        self.last_activity = current_timestamp;
        
        // Award RP for successful referral
        self.add_rp(REFERRAL_SUCCESS_RP, current_timestamp)?;
        
        Ok(())
    }

    /// Add indirect referral (L2 or L3)
    pub fn add_indirect_referral(&mut self, level: u8, current_timestamp: i64) -> Result<()> {
        self.indirect_referral_count += 1;
        
        match level {
            2 => self.network_stats.l2_network_size += 1,
            3 => self.network_stats.l3_network_size += 1,
            _ => return Err(FinovaError::InvalidReferralLevel.into()),
        }
        
        self.network_stats.total_network_size += 1;
        self.last_activity = current_timestamp;
        
        // Award scaled RP for indirect referrals
        let rp_reward = match level {
            2 => REFERRAL_SUCCESS_RP / 2, // 50% for L2
            3 => REFERRAL_SUCCESS_RP / 4, // 25% for L3
            _ => 0,
        };
        
        if rp_reward > 0 {
            self.add_rp(rp_reward, current_timestamp)?;
        }
        
        Ok(())
    }

    /// Add RP points and check for tier upgrades
    pub fn add_rp(&mut self, amount: u64, current_timestamp: i64) -> Result<()> {
        self.total_rp += amount;
        self.available_rp += amount;
        self.last_activity = current_timestamp;
        
        // Check for tier upgrade
        let new_tier = self.calculate_rp_tier();
        if new_tier > self.rp_tier {
            self.upgrade_tier(new_tier, current_timestamp)?;
        }
        
        Ok(())
    }

    /// Spend available RP
    pub fn spend_rp(&mut self, amount: u64) -> Result<()> {
        if self.available_rp < amount {
            return Err(FinovaError::InsufficientRP.into());
        }
        
        self.available_rp -= amount;
        Ok(())
    }

    /// Calculate current RP tier based on total RP
    pub fn calculate_rp_tier(&self) -> u8 {
        match self.total_rp {
            0..=999 => 0,           // Explorer
            1000..=4999 => 1,       // Connector
            5000..=14999 => 2,      // Influencer
            15000..=49999 => 3,     // Leader
            _ => 4,                 // Ambassador
        }
    }

    /// Upgrade to new RP tier
    pub fn upgrade_tier(&mut self, new_tier: u8, current_timestamp: i64) -> Result<()> {
        self.rp_tier = new_tier;
        
        // Update tier benefits
        match new_tier {
            0 => { // Explorer
                self.tier_benefits.mining_bonus = 0;
                self.tier_benefits.referral_bonus = 1000; // 10%
                self.tier_benefits.network_cap = 10;
            },
            1 => { // Connector
                self.tier_benefits.mining_bonus = 2000; // 20%
                self.tier_benefits.referral_bonus = 1500; // 15%
                self.tier_benefits.network_cap = 25;
            },
            2 => { // Influencer
                self.tier_benefits.mining_bonus = 5000; // 50%
                self.tier_benefits.referral_bonus = 2000; // 20%
                self.tier_benefits.network_cap = 50;
            },
            3 => { // Leader
                self.tier_benefits.mining_bonus = 10000; // 100%
                self.tier_benefits.referral_bonus = 2500; // 25%
                self.tier_benefits.network_cap = 100;
            },
            4 => { // Ambassador
                self.tier_benefits.mining_bonus = 20000; // 200%
                self.tier_benefits.referral_bonus = 3000; // 30%
                self.tier_benefits.network_cap = u32::MAX; // Unlimited
            },
            _ => return Err(FinovaError::InvalidRPTier.into()),
        }
        
        self.tier_benefits.tier_upgraded_at = current_timestamp;
        Ok(())
    }

    /// Get current network capacity based on tier
    pub fn get_network_cap(&self) -> u32 {
        self.tier_benefits.network_cap
    }

    /// Get mining bonus multiplier (scaled by 10000)
    pub fn get_mining_multiplier(&self) -> u16 {
        10000 + self.tier_benefits.mining_bonus // Base 1.0x + bonus
    }

    /// Get referral bonus multiplier (scaled by 10000)
    pub fn get_referral_multiplier(&self) -> u16 {
        self.tier_benefits.referral_bonus
    }

    /// Calculate network quality score
    pub fn calculate_network_quality(&self) -> u16 {
        if self.network_stats.total_network_size == 0 {
            return DEFAULT_NETWORK_QUALITY;
        }
        
        let active_ratio = (self.active_referral_count as u64 * 10000) / 
                          (self.network_stats.total_network_size as u64);
        
        let retention_weight = self.network_stats.retention_rate as u64;
        let level_weight = self.network_stats.average_referral_level as u64;
        
        // Weighted average of metrics
        let quality = (active_ratio * 40 + retention_weight * 40 + level_weight * 20) / 100;
        
        std::cmp::min(quality as u16, 10000)
    }

    /// Update network quality score
    pub fn update_network_quality(&mut self) {
        self.network_quality_score = self.calculate_network_quality();
    }

    /// Calculate network regression factor for anti-whale protection
    pub fn calculate_network_regression(&self) -> u32 {
        let network_size = self.network_stats.total_network_size as f64;
        let quality_factor = self.network_quality_score as f64 / 10000.0;
        
        // Exponential regression: e^(-0.0001 * network_size * quality_factor)
        let exponent = -0.0001 * network_size * quality_factor;
        let regression = (exponent.exp() * 10000.0) as u32;
        
        std::cmp::max(regression, 1000) // Minimum 10% efficiency
    }

    /// Check if referral is suspicious
    pub fn is_suspicious(&self) -> bool {
        self.abuse_metrics.suspicious_score > SUSPICIOUS_THRESHOLD ||
        self.abuse_metrics.circular_referral_count > MAX_CIRCULAR_REFERRALS ||
        self.abuse_metrics.bot_probability > BOT_PROBABILITY_THRESHOLD
    }

    /// Apply penalty for suspicious activity
    pub fn apply_penalty(&mut self, duration_seconds: i64, current_timestamp: i64) -> Result<()> {
        self.abuse_metrics.penalty_applied = true;
        self.abuse_metrics.penalty_end = current_timestamp + duration_seconds;
        
        // Reduce available RP by 50% as penalty
        self.available_rp = self.available_rp / 2;
        
        Ok(())
    }

    /// Check if currently under penalty
    pub fn is_under_penalty(&self, current_timestamp: i64) -> bool {
        self.abuse_metrics.penalty_applied && 
        current_timestamp < self.abuse_metrics.penalty_end
    }

    /// Update activity timestamp
    pub fn update_activity(&mut self, current_timestamp: i64) {
        self.last_activity = current_timestamp;
    }

    /// Add network value contribution
    pub fn add_network_value(&mut self, value: u64) {
        self.network_stats.total_network_value += value;
        self.network_stats.monthly_rewards += value;
    }

    /// Reset monthly statistics
    pub fn reset_monthly_stats(&mut self) {
        self.network_stats.monthly_rewards = 0;
    }
}

impl NetworkStats {
    pub const LEN: usize = 
        4 + // total_network_size
        4 + // l2_network_size
        4 + // l3_network_size
        8 + // total_network_value
        2 + // average_referral_level
        2 + // retention_rate
        8 + // lifetime_rewards
        8;  // monthly_rewards
}

impl TierBenefits {
    pub const LEN: usize = 
        2 + // mining_bonus
        2 + // referral_bonus
        4 + // network_cap
        8 + // special_benefits
        4 + // voting_power
        8;  // tier_upgraded_at
}

impl AbuseMetrics {
    pub const LEN: usize = 
        2 + // suspicious_score
        2 + // circular_referral_count
        2 + // rapid_registration_flags
        2 + // bot_probability
        8 + // last_audit
        1 + // penalty_applied
        8;  // penalty_end
}

impl ReferralConnection {
    pub const LEN: usize = 8 + // discriminator
        32 + // referrer
        32 + // referee
        1 + // level
        1 + // status (enum size)
        8 + // created_at
        9 + // kyc_completed_at (Option<i64>)
        8 + // total_rewards_earned
        8 + // last_referee_activity
        2 + // quality_contribution
        32; // reserved

    /// Initialize a new referral connection
    pub fn initialize(
        &mut self,
        referrer: Pubkey,
        referee: Pubkey,
        level: u8,
        current_timestamp: i64,
    ) -> Result<()> {
        self.referrer = referrer;
        self.referee = referee;
        self.level = level;
        self.status = ReferralStatus::Pending;
        self.created_at = current_timestamp;
        self.kyc_completed_at = None;
        self.total_rewards_earned = 0;
        self.last_referee_activity = current_timestamp;
        self.quality_contribution = DEFAULT_NETWORK_QUALITY;
        self.reserved = [0; 32];
        
        Ok(())
    }

    /// Activate the referral connection (when referee completes KYC)
    pub fn activate(&mut self, current_timestamp: i64) -> Result<()> {
        if self.status != ReferralStatus::Pending {
            return Err(FinovaError::InvalidReferralStatus.into());
        }
        
        self.status = ReferralStatus::Active;
        self.kyc_completed_at = Some(current_timestamp);
        self.last_referee_activity = current_timestamp;
        
        Ok(())
    }

    /// Update activity status based on referee's last activity
    pub fn update_activity_status(&mut self, last_activity: i64, current_timestamp: i64) {
        self.last_referee_activity = last_activity;
        
        let days_inactive = (current_timestamp - last_activity) / 86400; // seconds to days
        
        self.status = match days_inactive {
            0..=30 => ReferralStatus::Active,
            31..=90 => ReferralStatus::Inactive,
            _ => ReferralStatus::Churned,
        };
    }

    /// Add reward earned from this connection
    pub fn add_reward(&mut self, amount: u64) {
        self.total_rewards_earned += amount;
    }

    /// Check if connection is currently earning rewards
    pub fn is_earning(&self) -> bool {
        matches!(self.status, ReferralStatus::Active)
    }

    /// Calculate quality contribution based on referee's performance
    pub fn calculate_quality_contribution(&self, referee_xp_level: u16, referee_mining_rate: u64) -> u16 {
        let level_factor = std::cmp::min(referee_xp_level * 10, 5000); // Max 50% from level
        let mining_factor = std::cmp::min(referee_mining_rate / 1000, 5000); // Max 50% from mining
        
        std::cmp::min(level_factor + mining_factor, 10000)
    }

    /// Update quality contribution
    pub fn update_quality_contribution(&mut self, referee_xp_level: u16, referee_mining_rate: u64) {
        self.quality_contribution = self.calculate_quality_contribution(referee_xp_level, referee_mining_rate);
    }
}

// Helper functions for referral calculations

/// Calculate referral bonus based on tier and network quality
pub fn calculate_referral_bonus(
    base_amount: u64,
    referral_multiplier: u16,
    network_quality: u16,
    regression_factor: u32,
) -> u64 {
    let tier_bonus = (base_amount * referral_multiplier as u64) / 10000;
    let quality_adjusted = (tier_bonus * network_quality as u64) / 10000;
    let final_amount = (quality_adjusted * regression_factor as u64) / 10000;
    
    final_amount
}

/// Calculate network effect multiplier for mining
pub fn calculate_network_mining_multiplier(
    active_referrals: u32,
    network_quality: u16,
    tier: u8,
) -> u16 {
    let base_multiplier = 10000; // 1.0x
    let referral_bonus = std::cmp::min(active_referrals * 100, 3000); // Max 30% from referrals
    let quality_bonus = (network_quality * 20) / 100; // Up to 20% from quality
    let tier_bonus = match tier {
        0 => 0,    // Explorer: 0%
        1 => 500,  // Connector: 5%
        2 => 1000, // Influencer: 10%
        3 => 1500, // Leader: 15%
        4 => 2000, // Ambassador: 20%
        _ => 0,
    };
    
    base_multiplier + referral_bonus as u16 + quality_bonus + tier_bonus
}

/// Validate referral chain to prevent circular references
pub fn validate_referral_chain(
    potential_referrer: &Pubkey,
    potential_referee: &Pubkey,
    existing_connections: &[ReferralConnection],
) -> Result<bool> {
    // Check for direct circular reference
    if potential_referrer == potential_referee {
        return Ok(false);
    }
    
    // Check for indirect circular references (up to 3 levels)
    let mut current = *potential_referrer;
    for _ in 0..3 {
        if let Some(connection) = existing_connections.iter()
            .find(|c| c.referee == current) {
            if connection.referrer == *potential_referee {
                return Ok(false); // Circular reference detected
            }
            current = connection.referrer;
        } else {
            break;
        }
    }
    
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rp_tier_calculation() {
        let mut referral = ReferralAccount::default();
        
        // Test Explorer tier
        referral.total_rp = 500;
        assert_eq!(referral.calculate_rp_tier(), 0);
        
        // Test Connector tier
        referral.total_rp = 2500;
        assert_eq!(referral.calculate_rp_tier(), 1);
        
        // Test Ambassador tier
        referral.total_rp = 75000;
        assert_eq!(referral.calculate_rp_tier(), 4);
    }

    #[test]
    fn test_network_quality_calculation() {
        let mut referral = ReferralAccount::default();
        referral.active_referral_count = 8;
        referral.network_stats.total_network_size = 10;
        referral.network_stats.retention_rate = 8000; // 80%
        referral.network_stats.average_referral_level = 1200; // 12.00
        
        let quality = referral.calculate_network_quality();
        
        // Should be weighted average: (80% * 40 + 80% * 40 + 12% * 20) / 100
        assert!(quality > 6000 && quality < 8000);
    }

    #[test]
    fn test_referral_bonus_calculation() {
        let base_amount = 1000;
        let multiplier = 1500; // 15%
        let quality = 8000; // 80%
        let regression = 9000; // 90%
        
        let bonus = calculate_referral_bonus(base_amount, multiplier, quality, regression);
        
        // Expected: 1000 * 0.15 * 0.8 * 0.9 = 108
        assert_eq!(bonus, 108);
    }

    #[test]
    fn test_circular_reference_detection() {
        let alice = Pubkey::new_unique();
        let bob = Pubkey::new_unique();
        let charlie = Pubkey::new_unique();
        
        let connections = vec![
            ReferralConnection {
                referrer: bob,
                referee: charlie,
                level: 1,
                status: ReferralStatus::Active,
                created_at: 0,
                kyc_completed_at: Some(0),
                total_rewards_earned: 0,
                last_referee_activity: 0,
                quality_contribution: 5000,
                reserved: [0; 32],
            }
        ];
        
        // This should detect circular reference: Alice -> Bob -> Charlie -> Alice
        let result = validate_referral_chain(&charlie, &alice, &connections);
        assert!(result.is_ok());
        
        // Direct circular reference
        let result = validate_referral_chain(&alice, &alice, &connections);
        assert!(!result.unwrap());
    }
}
