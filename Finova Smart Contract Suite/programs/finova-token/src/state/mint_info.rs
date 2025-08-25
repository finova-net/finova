// programs/finova-token/src/state/mint_info.rs

use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};
use crate::errors::FinovaTokenError;
use crate::constants::*;

/// Comprehensive mint information and configuration for Finova Token ecosystem
#[account]
#[derive(Debug)]
pub struct MintInfo {
    /// Authority that can update mint parameters
    pub authority: Pubkey,
    
    /// The actual token mint account
    pub mint: Pubkey,
    
    /// Total supply cap (100 billion tokens)
    pub max_supply: u64,
    
    /// Current circulating supply
    pub current_supply: u64,
    
    /// Mining phase configuration
    pub mining_phase: MiningPhase,
    
    /// Base mining rate per hour (in smallest token unit)
    pub base_mining_rate: u64,
    
    /// Current user count for phase calculations
    pub total_users: u64,
    
    /// Pioneer bonus multiplier (decreases as users grow)
    pub pioneer_bonus: u64, // Stored as basis points (10000 = 1.0x)
    
    /// Anti-whale regression factor
    pub regression_enabled: bool,
    pub regression_coefficient: u64, // Stored as basis points
    
    /// Staking configuration
    pub staking_enabled: bool,
    pub base_staking_apy: u64, // Stored as basis points (800 = 8%)
    
    /// Fee configuration
    pub transfer_fee_bps: u16, // Basis points (10 = 0.1%)
    pub burn_fee_bps: u16,     // Basis points for burning fees
    
    /// Treasury and reward pools
    pub treasury_vault: Pubkey,
    pub staking_pool: Pubkey,
    pub mining_pool: Pubkey,
    pub referral_pool: Pubkey,
    
    /// Security features
    pub emergency_pause: bool,
    pub kyc_required: bool,
    pub anti_bot_enabled: bool,
    
    /// Time-based parameters
    pub created_at: i64,
    pub last_updated: i64,
    pub phase_started_at: i64,
    
    /// Advanced tokenomics
    pub deflation_rate: u64,      // Basis points per year
    pub inflation_cap: u64,       // Maximum tokens that can be minted
    pub whale_threshold: u64,     // Threshold for whale detection
    pub max_daily_mint: u64,      // Maximum tokens that can be minted per day
    
    /// Governance parameters
    pub governance_enabled: bool,
    pub min_governance_stake: u64,
    pub voting_period: i64,       // Seconds
    
    /// Reserved space for future upgrades
    pub reserved: [u8; 256],
}

impl MintInfo {
    pub const LEN: usize = 8 + // discriminator
        32 + // authority
        32 + // mint
        8 + // max_supply
        8 + // current_supply
        std::mem::size_of::<MiningPhase>() + // mining_phase
        8 + // base_mining_rate
        8 + // total_users
        8 + // pioneer_bonus
        1 + // regression_enabled
        8 + // regression_coefficient
        1 + // staking_enabled
        8 + // base_staking_apy
        2 + // transfer_fee_bps
        2 + // burn_fee_bps
        32 + // treasury_vault
        32 + // staking_pool
        32 + // mining_pool
        32 + // referral_pool
        1 + // emergency_pause
        1 + // kyc_required
        1 + // anti_bot_enabled
        8 + // created_at
        8 + // last_updated
        8 + // phase_started_at
        8 + // deflation_rate
        8 + // inflation_cap
        8 + // whale_threshold
        8 + // max_daily_mint
        1 + // governance_enabled
        8 + // min_governance_stake
        8 + // voting_period
        256; // reserved

    /// Initialize new mint info with default parameters
    pub fn initialize(
        &mut self,
        authority: Pubkey,
        mint: Pubkey,
        treasury_vault: Pubkey,
        staking_pool: Pubkey,
        mining_pool: Pubkey,
        referral_pool: Pubkey,
    ) -> Result<()> {
        let clock = Clock::get()?;
        
        self.authority = authority;
        self.mint = mint;
        self.max_supply = MAX_TOTAL_SUPPLY;
        self.current_supply = 0;
        self.mining_phase = MiningPhase::Pioneer;
        self.base_mining_rate = INITIAL_MINING_RATE;
        self.total_users = 0;
        self.pioneer_bonus = 20000; // 2.0x multiplier
        self.regression_enabled = true;
        self.regression_coefficient = 100; // 0.001 coefficient
        self.staking_enabled = true;
        self.base_staking_apy = 800; // 8% APY
        self.transfer_fee_bps = 10; // 0.1%
        self.burn_fee_bps = 5; // 0.05%
        self.treasury_vault = treasury_vault;
        self.staking_pool = staking_pool;
        self.mining_pool = mining_pool;
        self.referral_pool = referral_pool;
        self.emergency_pause = false;
        self.kyc_required = true;
        self.anti_bot_enabled = true;
        self.created_at = clock.unix_timestamp;
        self.last_updated = clock.unix_timestamp;
        self.phase_started_at = clock.unix_timestamp;
        self.deflation_rate = 200; // 2% per year
        self.inflation_cap = MAX_TOTAL_SUPPLY;
        self.whale_threshold = 100_000 * LAMPORTS_PER_TOKEN; // 100K tokens
        self.max_daily_mint = 1_000_000 * LAMPORTS_PER_TOKEN; // 1M tokens per day
        self.governance_enabled = false;
        self.min_governance_stake = 10_000 * LAMPORTS_PER_TOKEN; // 10K tokens
        self.voting_period = 7 * 24 * 60 * 60; // 7 days
        self.reserved = [0; 256];
        
        Ok(())
    }

    /// Calculate current mining rate based on phase and user count
    pub fn calculate_mining_rate(&self, user_holdings: u64, referral_count: u32, is_kyc: bool) -> Result<u64> {
        require!(!self.emergency_pause, FinovaTokenError::EmergencyPause);
        
        // Base rate according to current phase
        let phase_rate = match self.mining_phase {
            MiningPhase::Pioneer => self.base_mining_rate,
            MiningPhase::Growth => self.base_mining_rate / 2,
            MiningPhase::Maturity => self.base_mining_rate / 4,
            MiningPhase::Stability => self.base_mining_rate / 10,
        };

        // Pioneer bonus calculation
        let pioneer_multiplier = if self.total_users < 1_000_000 {
            self.pioneer_bonus.saturating_sub(
                (self.total_users * 1000) / 1_000_000 // Decreases as users grow
            )
        } else {
            10000 // 1.0x when over 1M users
        };

        // Referral bonus (10% per active referral, max 250%)
        let referral_multiplier = 10000 + (referral_count as u64 * 1000).min(15000);

        // KYC security bonus
        let security_multiplier = if is_kyc { 12000 } else { 8000 }; // 1.2x vs 0.8x

        // Anti-whale exponential regression
        let regression_factor = if self.regression_enabled && user_holdings > 0 {
            let exponent = (user_holdings / LAMPORTS_PER_TOKEN) * self.regression_coefficient / 10000;
            // Approximate exponential decay: e^(-x) ≈ 1/(1+x) for small x
            10000 / (10000 + exponent).max(1)
        } else {
            10000
        };

        // Calculate final mining rate
        let rate = phase_rate
            .saturating_mul(pioneer_multiplier) / 10000
            .saturating_mul(referral_multiplier) / 10000
            .saturating_mul(security_multiplier) / 10000
            .saturating_mul(regression_factor) / 10000;

        Ok(rate)
    }

    /// Calculate staking rewards based on amount and duration
    pub fn calculate_staking_rewards(&self, staked_amount: u64, duration_seconds: i64) -> Result<u64> {
        require!(self.staking_enabled, FinovaTokenError::StakingDisabled);
        require!(!self.emergency_pause, FinovaTokenError::EmergencyPause);
        require!(duration_seconds > 0, FinovaTokenError::InvalidDuration);

        // Annual rewards = staked_amount * APY / 10000
        let annual_rewards = staked_amount
            .saturating_mul(self.base_staking_apy as u64) / 10000;

        // Calculate rewards for the given duration
        let rewards = annual_rewards
            .saturating_mul(duration_seconds as u64) / (365 * 24 * 60 * 60);

        Ok(rewards)
    }

    /// Update mining phase based on user count
    pub fn update_mining_phase(&mut self) -> Result<bool> {
        let new_phase = match self.total_users {
            0..=100_000 => MiningPhase::Pioneer,
            100_001..=1_000_000 => MiningPhase::Growth,
            1_000_001..=10_000_000 => MiningPhase::Maturity,
            _ => MiningPhase::Stability,
        };

        if new_phase != self.mining_phase {
            self.mining_phase = new_phase;
            self.phase_started_at = Clock::get()?.unix_timestamp;
            self.last_updated = Clock::get()?.unix_timestamp;
            return Ok(true);
        }

        Ok(false)
    }

    /// Calculate transfer fees
    pub fn calculate_transfer_fee(&self, amount: u64) -> u64 {
        amount.saturating_mul(self.transfer_fee_bps as u64) / 10000
    }

    /// Calculate burn amount from fees
    pub fn calculate_burn_amount(&self, fee_amount: u64) -> u64 {
        fee_amount.saturating_mul(self.burn_fee_bps as u64) / 10000
    }

    /// Check if address is considered a whale
    pub fn is_whale(&self, holdings: u64) -> bool {
        holdings > self.whale_threshold
    }

    /// Check if daily minting limit is reached
    pub fn check_daily_mint_limit(&self, additional_mint: u64, daily_minted: u64) -> Result<()> {
        require!(
            daily_minted.saturating_add(additional_mint) <= self.max_daily_mint,
            FinovaTokenError::DailyMintLimitExceeded
        );
        Ok(())
    }

    /// Validate mint operation
    pub fn validate_mint(&self, amount: u64) -> Result<()> {
        require!(!self.emergency_pause, FinovaTokenError::EmergencyPause);
        require!(amount > 0, FinovaTokenError::InvalidAmount);
        require!(
            self.current_supply.saturating_add(amount) <= self.max_supply,
            FinovaTokenError::SupplyCapExceeded
        );
        Ok(())
    }

    /// Validate burn operation
    pub fn validate_burn(&self, amount: u64) -> Result<()> {
        require!(!self.emergency_pause, FinovaTokenError::EmergencyPause);
        require!(amount > 0, FinovaTokenError::InvalidAmount);
        require!(amount <= self.current_supply, FinovaTokenError::InsufficientSupply);
        Ok(())
    }

    /// Update supply after mint
    pub fn update_supply_mint(&mut self, amount: u64) -> Result<()> {
        self.current_supply = self.current_supply.saturating_add(amount);
        self.last_updated = Clock::get()?.unix_timestamp;
        Ok(())
    }

    /// Update supply after burn
    pub fn update_supply_burn(&mut self, amount: u64) -> Result<()> {
        self.current_supply = self.current_supply.saturating_sub(amount);
        self.last_updated = Clock::get()?.unix_timestamp;
        Ok(())
    }

    /// Check governance requirements
    pub fn check_governance_eligibility(&self, stake_amount: u64) -> bool {
        self.governance_enabled && stake_amount >= self.min_governance_stake
    }

    /// Calculate deflation amount based on time
    pub fn calculate_deflation(&self, time_elapsed: i64) -> u64 {
        if self.deflation_rate == 0 || time_elapsed <= 0 {
            return 0;
        }

        let annual_deflation = self.current_supply
            .saturating_mul(self.deflation_rate as u64) / 10000;
        
        annual_deflation
            .saturating_mul(time_elapsed as u64) / (365 * 24 * 60 * 60)
    }

    /// Update user count and potentially trigger phase change
    pub fn update_user_count(&mut self, new_count: u64) -> Result<bool> {
        self.total_users = new_count;
        self.update_mining_phase()
    }

    /// Emergency pause functionality
    pub fn set_emergency_pause(&mut self, paused: bool) -> Result<()> {
        self.emergency_pause = paused;
        self.last_updated = Clock::get()?.unix_timestamp;
        Ok(())
    }

    /// Update authority with proper checks
    pub fn update_authority(&mut self, new_authority: Pubkey) -> Result<()> {
        require!(!self.emergency_pause, FinovaTokenError::EmergencyPause);
        self.authority = new_authority;
        self.last_updated = Clock::get()?.unix_timestamp;
        Ok(())
    }

    /// Update fee parameters
    pub fn update_fees(&mut self, transfer_fee_bps: u16, burn_fee_bps: u16) -> Result<()> {
        require!(transfer_fee_bps <= 1000, FinovaTokenError::FeeTooHigh); // Max 10%
        require!(burn_fee_bps <= 500, FinovaTokenError::FeeTooHigh); // Max 5%
        
        self.transfer_fee_bps = transfer_fee_bps;
        self.burn_fee_bps = burn_fee_bps;
        self.last_updated = Clock::get()?.unix_timestamp;
        Ok(())
    }

    /// Get current phase information
    pub fn get_phase_info(&self) -> PhaseInfo {
        PhaseInfo {
            current_phase: self.mining_phase,
            phase_start_time: self.phase_started_at,
            current_users: self.total_users,
            next_phase_threshold: match self.mining_phase {
                MiningPhase::Pioneer => 100_000,
                MiningPhase::Growth => 1_000_000,
                MiningPhase::Maturity => 10_000_000,
                MiningPhase::Stability => u64::MAX,
            },
            base_rate: self.base_mining_rate,
            pioneer_bonus: self.pioneer_bonus,
        }
    }

    /// Get supply statistics
    pub fn get_supply_stats(&self) -> SupplyStats {
        SupplyStats {
            current_supply: self.current_supply,
            max_supply: self.max_supply,
            supply_percentage: (self.current_supply * 10000) / self.max_supply,
            remaining_mintable: self.max_supply.saturating_sub(self.current_supply),
            is_deflationary: self.deflation_rate > 0,
            daily_mint_limit: self.max_daily_mint,
        }
    }
}

/// Mining phase enumeration
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq)]
pub enum MiningPhase {
    Pioneer,   // 0-100K users
    Growth,    // 100K-1M users
    Maturity,  // 1M-10M users
    Stability, // 10M+ users
}

impl Default for MiningPhase {
    fn default() -> Self {
        MiningPhase::Pioneer
    }
}

/// Phase information structure
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct PhaseInfo {
    pub current_phase: MiningPhase,
    pub phase_start_time: i64,
    pub current_users: u64,
    pub next_phase_threshold: u64,
    pub base_rate: u64,
    pub pioneer_bonus: u64,
}

/// Supply statistics structure
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct SupplyStats {
    pub current_supply: u64,
    pub max_supply: u64,
    pub supply_percentage: u64, // In basis points (10000 = 100%)
    pub remaining_mintable: u64,
    pub is_deflationary: bool,
    pub daily_mint_limit: u64,
}

/// Daily mint tracking account
#[account]
#[derive(Debug)]
pub struct DailyMintTracker {
    /// The day this tracker represents (Unix timestamp / 86400)
    pub day: i64,
    
    /// Amount minted today
    pub minted_today: u64,
    
    /// Reference to mint info
    pub mint_info: Pubkey,
    
    /// Last update timestamp
    pub last_updated: i64,
}

impl DailyMintTracker {
    pub const LEN: usize = 8 + // discriminator
        8 + // day
        8 + // minted_today
        32 + // mint_info
        8; // last_updated

    /// Initialize daily mint tracker
    pub fn initialize(&mut self, mint_info: Pubkey) -> Result<()> {
        let clock = Clock::get()?;
        let current_day = clock.unix_timestamp / 86400;
        
        self.day = current_day;
        self.minted_today = 0;
        self.mint_info = mint_info;
        self.last_updated = clock.unix_timestamp;
        
        Ok(())
    }

    /// Add minted amount and check limits
    pub fn add_minted(&mut self, amount: u64, daily_limit: u64) -> Result<()> {
        let clock = Clock::get()?;
        let current_day = clock.unix_timestamp / 86400;
        
        // Reset if it's a new day
        if current_day > self.day {
            self.day = current_day;
            self.minted_today = 0;
        }
        
        // Check daily limit
        require!(
            self.minted_today.saturating_add(amount) <= daily_limit,
            FinovaTokenError::DailyMintLimitExceeded
        );
        
        self.minted_today = self.minted_today.saturating_add(amount);
        self.last_updated = clock.unix_timestamp;
        
        Ok(())
    }

    /// Get remaining mintable amount for today
    pub fn get_remaining_mintable(&self, daily_limit: u64) -> u64 {
        daily_limit.saturating_sub(self.minted_today)
    }
}

/// Whale detection and management
#[account]
#[derive(Debug)]
pub struct WhaleInfo {
    /// User's public key
    pub user: Pubkey,
    
    /// Total holdings across all accounts
    pub total_holdings: u64,
    
    /// Whale tier based on holdings
    pub whale_tier: WhaleTier,
    
    /// Additional restrictions applied
    pub restrictions_applied: u64, // Bitflags
    
    /// Last evaluation timestamp
    pub last_evaluated: i64,
    
    /// Warning count for suspicious activities
    pub warning_count: u32,
}

impl WhaleInfo {
    pub const LEN: usize = 8 + // discriminator
        32 + // user
        8 + // total_holdings
        std::mem::size_of::<WhaleTier>() + // whale_tier
        8 + // restrictions_applied
        8 + // last_evaluated
        4; // warning_count

    /// Update whale status based on holdings
    pub fn update_whale_status(&mut self, new_holdings: u64, whale_threshold: u64) -> Result<()> {
        self.total_holdings = new_holdings;
        
        self.whale_tier = if new_holdings >= whale_threshold * 10 {
            WhaleTier::Mega
        } else if new_holdings >= whale_threshold * 5 {
            WhaleTier::Super
        } else if new_holdings >= whale_threshold {
            WhaleTier::Standard
        } else {
            WhaleTier::None
        };
        
        self.last_evaluated = Clock::get()?.unix_timestamp;
        Ok(())
    }

    /// Apply whale restrictions
    pub fn apply_restrictions(&mut self, restrictions: u64) -> Result<()> {
        self.restrictions_applied |= restrictions;
        self.last_evaluated = Clock::get()?.unix_timestamp;
        Ok(())
    }

    /// Check if specific restriction is applied
    pub fn has_restriction(&self, restriction: u64) -> bool {
        (self.restrictions_applied & restriction) != 0
    }
}

/// Whale tier classification
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq)]
pub enum WhaleTier {
    None,     // Below whale threshold
    Standard, // 1x-5x whale threshold
    Super,    // 5x-10x whale threshold
    Mega,     // 10x+ whale threshold
}

impl Default for WhaleTier {
    fn default() -> Self {
        WhaleTier::None
    }
}

// Whale restriction flags
pub const RESTRICTION_TRANSFER_LIMIT: u64 = 1 << 0;
pub const RESTRICTION_MINING_PENALTY: u64 = 1 << 1;
pub const RESTRICTION_GOVERNANCE_LIMIT: u64 = 1 << 2;
pub const RESTRICTION_STAKING_PENALTY: u64 = 1 << 3;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mining_rate_calculation() {
        let mut mint_info = MintInfo::default();
        mint_info.base_mining_rate = 100_000; // 0.1 FIN per hour
        mint_info.pioneer_bonus = 20000; // 2.0x
        mint_info.total_users = 50_000;
        mint_info.regression_enabled = true;
        mint_info.regression_coefficient = 100;

        // Test new user
        let rate = mint_info.calculate_mining_rate(0, 0, true).unwrap();
        assert!(rate > 0);

        // Test user with holdings (should have regression)
        let rate_with_holdings = mint_info.calculate_mining_rate(
            10_000 * LAMPORTS_PER_TOKEN, 
            5, 
            true
        ).unwrap();
        assert!(rate_with_holdings < rate);
    }

    #[test]
    fn test_phase_transitions() {
        let mut mint_info = MintInfo::default();
        
        // Start in Pioneer phase
        assert_eq!(mint_info.mining_phase, MiningPhase::Pioneer);
        
        // Transition to Growth
        let changed = mint_info.update_user_count(150_000).unwrap();
        assert!(changed);
        assert_eq!(mint_info.mining_phase, MiningPhase::Growth);
        
        // Transition to Maturity
        let changed = mint_info.update_user_count(2_000_000).unwrap();
        assert!(changed);
        assert_eq!(mint_info.mining_phase, MiningPhase::Maturity);
    }

    #[test]
    fn test_daily_mint_tracking() {
        let mut tracker = DailyMintTracker::default();
        let mint_info_key = Pubkey::new_unique();
        
        tracker.initialize(mint_info_key).unwrap();
        
        // Test adding minted amount
        tracker.add_minted(1000, 10000).unwrap();
        assert_eq!(tracker.minted_today, 1000);
        
        // Test daily limit
        let result = tracker.add_minted(10000, 10000);
        assert!(result.is_err());
    }
}
