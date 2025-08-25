// programs/finova-token/src/state/stake_account.rs

use anchor_lang::prelude::*;
use crate::constants::*;
use crate::errors::FinovaTokenError;

/// Represents a user's staking position in the Finova ecosystem
/// Implements liquid staking with enhanced rewards based on XP, RP, and activity
#[account]
pub struct StakeAccount {
    /// The owner of this stake account
    pub owner: Pubkey,
    
    /// Amount of $FIN tokens staked
    pub staked_amount: u64,
    
    /// Amount of $sFIN tokens minted (liquid staking derivative)
    pub sfin_amount: u64,
    
    /// Timestamp when the stake was created
    pub stake_timestamp: i64,
    
    /// Last timestamp when rewards were claimed
    pub last_claim_timestamp: i64,
    
    /// Accumulated but unclaimed rewards
    pub pending_rewards: u64,
    
    /// Total rewards claimed lifetime
    pub total_rewards_claimed: u64,
    
    /// Staking tier (0-4: Bronze, Silver, Gold, Platinum, Diamond)
    pub staking_tier: u8,
    
    /// Base APY percentage (scaled by 100, e.g., 800 = 8%)
    pub base_apy: u16,
    
    /// Current multiplier from XP level (scaled by 100, e.g., 120 = 1.2x)
    pub xp_multiplier: u16,
    
    /// Current multiplier from RP tier (scaled by 100, e.g., 150 = 1.5x)
    pub rp_multiplier: u16,
    
    /// Loyalty bonus multiplier based on staking duration (scaled by 100)
    pub loyalty_multiplier: u16,
    
    /// Activity bonus multiplier based on daily activity (scaled by 100)
    pub activity_multiplier: u16,
    
    /// Effective APY with all multipliers applied (scaled by 100)
    pub effective_apy: u16,
    
    /// Lock period in seconds (0 for no lock)
    pub lock_period: u64,
    
    /// Timestamp when unlock becomes available
    pub unlock_timestamp: i64,
    
    /// Auto-compound flag
    pub auto_compound: bool,
    
    /// Compound frequency in seconds
    pub compound_frequency: u64,
    
    /// Last compound timestamp
    pub last_compound_timestamp: i64,
    
    /// Number of times compounded
    pub compound_count: u32,
    
    /// Special features unlocked (bitfield)
    /// Bit 0: Premium badge
    /// Bit 1: Priority support
    /// Bit 2: VIP features
    /// Bit 3: Guild master privileges
    /// Bit 4: DAO governance
    /// Bit 5: Max benefits
    pub unlocked_features: u8,
    
    /// Boost from special cards (temporary multiplier, scaled by 100)
    pub card_boost_multiplier: u16,
    
    /// Timestamp when card boost expires
    pub card_boost_expiry: i64,
    
    /// Performance metrics for dynamic adjustments
    pub performance_score: u16,
    
    /// Risk score for security purposes
    pub risk_score: u8,
    
    /// Emergency withdrawal available flag
    pub emergency_withdrawal_available: bool,
    
    /// Penalty applied (in basis points)
    pub penalty_rate: u16,
    
    /// Reserved space for future upgrades
    pub reserved: [u8; 64],
    
    /// Account bump for PDA
    pub bump: u8,
}

impl StakeAccount {
    /// Size of the account in bytes
    pub const SIZE: usize = 8 + // discriminator
        32 + // owner
        8 + // staked_amount
        8 + // sfin_amount
        8 + // stake_timestamp
        8 + // last_claim_timestamp
        8 + // pending_rewards
        8 + // total_rewards_claimed
        1 + // staking_tier
        2 + // base_apy
        2 + // xp_multiplier
        2 + // rp_multiplier
        2 + // loyalty_multiplier
        2 + // activity_multiplier
        2 + // effective_apy
        8 + // lock_period
        8 + // unlock_timestamp
        1 + // auto_compound
        8 + // compound_frequency
        8 + // last_compound_timestamp
        4 + // compound_count
        1 + // unlocked_features
        2 + // card_boost_multiplier
        8 + // card_boost_expiry
        2 + // performance_score
        1 + // risk_score
        1 + // emergency_withdrawal_available
        2 + // penalty_rate
        64 + // reserved
        1; // bump

    /// Initialize a new stake account
    pub fn initialize(
        &mut self,
        owner: Pubkey,
        staked_amount: u64,
        current_timestamp: i64,
        bump: u8,
    ) -> Result<()> {
        require!(staked_amount >= MIN_STAKE_AMOUNT, FinovaTokenError::InsufficientStakeAmount);
        
        self.owner = owner;
        self.staked_amount = staked_amount;
        self.sfin_amount = staked_amount; // 1:1 initial ratio
        self.stake_timestamp = current_timestamp;
        self.last_claim_timestamp = current_timestamp;
        self.last_compound_timestamp = current_timestamp;
        self.pending_rewards = 0;
        self.total_rewards_claimed = 0;
        self.staking_tier = self.calculate_staking_tier(staked_amount);
        self.base_apy = self.get_base_apy_for_tier(self.staking_tier);
        self.xp_multiplier = 100; // 1.0x default
        self.rp_multiplier = 100; // 1.0x default
        self.loyalty_multiplier = 100; // 1.0x default
        self.activity_multiplier = 100; // 1.0x default
        self.effective_apy = self.base_apy;
        self.lock_period = 0;
        self.unlock_timestamp = current_timestamp;
        self.auto_compound = true;
        self.compound_frequency = DAILY_SECONDS;
        self.compound_count = 0;
        self.unlocked_features = self.get_tier_features(self.staking_tier);
        self.card_boost_multiplier = 100; // 1.0x default
        self.card_boost_expiry = 0;
        self.performance_score = 100;
        self.risk_score = 0;
        self.emergency_withdrawal_available = true;
        self.penalty_rate = 0;
        self.reserved = [0; 64];
        self.bump = bump;
        
        Ok(())
    }

    /// Calculate staking tier based on amount
    pub fn calculate_staking_tier(&self, amount: u64) -> u8 {
        match amount {
            0..=499_000_000 => 0,        // Bronze: 100-499 $FIN
            500_000_000..=999_000_000 => 1,  // Silver: 500-999 $FIN
            1_000_000_000..=4_999_000_000 => 2, // Gold: 1,000-4,999 $FIN
            5_000_000_000..=9_999_000_000 => 3, // Platinum: 5,000-9,999 $FIN
            _ => 4,                      // Diamond: 10,000+ $FIN
        }
    }

    /// Get base APY for tier (scaled by 100)
    pub fn get_base_apy_for_tier(&self, tier: u8) -> u16 {
        match tier {
            0 => 800,  // 8% for Bronze
            1 => 1000, // 10% for Silver
            2 => 1200, // 12% for Gold
            3 => 1400, // 14% for Platinum
            4 => 1500, // 15% for Diamond
            _ => 800,
        }
    }

    /// Get unlocked features for tier
    pub fn get_tier_features(&self, tier: u8) -> u8 {
        match tier {
            0 => 0b00000000, // Bronze: No special features
            1 => 0b00000011, // Silver: Premium badge + Priority support
            2 => 0b00000111, // Gold: + VIP features
            3 => 0b00001111, // Platinum: + Guild master privileges
            4 => 0b00111111, // Diamond: + DAO governance + Max benefits
            _ => 0b00000000,
        }
    }

    /// Update XP multiplier based on user level
    pub fn update_xp_multiplier(&mut self, xp_level: u32) -> Result<()> {
        // Formula: 1.0x + (XP_Level / 100)
        let multiplier = 100 + (xp_level as u16).min(400); // Cap at 5.0x
        self.xp_multiplier = multiplier;
        self.recalculate_effective_apy();
        Ok(())
    }

    /// Update RP multiplier based on referral tier
    pub fn update_rp_multiplier(&mut self, rp_tier: u8) -> Result<()> {
        // Formula: 1.0x + (RP_Tier × 0.2)
        let multiplier = 100 + (rp_tier as u16 * 20).min(100); // Cap at 2.0x additional
        self.rp_multiplier = multiplier;
        self.recalculate_effective_apy();
        Ok(())
    }

    /// Update loyalty multiplier based on staking duration
    pub fn update_loyalty_multiplier(&mut self, current_timestamp: i64) -> Result<()> {
        let staking_duration_months = (current_timestamp - self.stake_timestamp) / (30 * DAILY_SECONDS);
        // Formula: 1.0x + (duration_months × 0.05)
        let multiplier = 100 + (staking_duration_months as u16 * 5).min(100); // Cap at 2.0x
        self.loyalty_multiplier = multiplier;
        self.recalculate_effective_apy();
        Ok(())
    }

    /// Update activity multiplier based on daily activity score
    pub fn update_activity_multiplier(&mut self, activity_score: u16) -> Result<()> {
        // Formula: 1.0x + (activity_score × 0.1), max 2.0x
        let multiplier = 100 + (activity_score * 10 / 100).min(100);
        self.activity_multiplier = multiplier;
        self.recalculate_effective_apy();
        Ok(())
    }

    /// Apply special card boost
    pub fn apply_card_boost(&mut self, boost_multiplier: u16, duration_seconds: u64, current_timestamp: i64) -> Result<()> {
        self.card_boost_multiplier = boost_multiplier;
        self.card_boost_expiry = current_timestamp + duration_seconds as i64;
        self.recalculate_effective_apy();
        Ok(())
    }

    /// Check and remove expired card boost
    pub fn check_card_boost_expiry(&mut self, current_timestamp: i64) -> Result<()> {
        if current_timestamp >= self.card_boost_expiry {
            self.card_boost_multiplier = 100;
            self.card_boost_expiry = 0;
            self.recalculate_effective_apy();
        }
        Ok(())
    }

    /// Recalculate effective APY with all multipliers
    pub fn recalculate_effective_apy(&mut self) {
        let base_apy = self.base_apy as u32;
        let xp_mult = self.xp_multiplier as u32;
        let rp_mult = self.rp_multiplier as u32;
        let loyalty_mult = self.loyalty_multiplier as u32;
        let activity_mult = self.activity_multiplier as u32;
        let card_mult = self.card_boost_multiplier as u32;
        
        // Formula: Base_APY × XP_Multiplier × RP_Multiplier × Loyalty_Multiplier × Activity_Multiplier × Card_Multiplier
        let effective_apy = (base_apy * xp_mult * rp_mult * loyalty_mult * activity_mult * card_mult) 
            / (100_u32.pow(5)); // Divide by 100^5 since all multipliers are scaled by 100
        
        self.effective_apy = effective_apy.min(5000) as u16; // Cap at 50% APY
    }

    /// Calculate pending rewards
    pub fn calculate_pending_rewards(&self, current_timestamp: i64) -> Result<u64> {
        if current_timestamp <= self.last_claim_timestamp {
            return Ok(self.pending_rewards);
        }
        
        let time_elapsed = current_timestamp - self.last_claim_timestamp;
        let annual_reward = (self.staked_amount as u128 * self.effective_apy as u128) / 10000; // Divide by 10000 (100 for percentage, 100 for scaling)
        let period_reward = (annual_reward * time_elapsed as u128) / YEAR_SECONDS as u128;
        
        let new_rewards = period_reward as u64;
        Ok(self.pending_rewards + new_rewards)
    }

    /// Update pending rewards
    pub fn update_pending_rewards(&mut self, current_timestamp: i64) -> Result<()> {
        let total_pending = self.calculate_pending_rewards(current_timestamp)?;
        self.pending_rewards = total_pending;
        self.last_claim_timestamp = current_timestamp;
        Ok(())
    }

    /// Claim rewards
    pub fn claim_rewards(&mut self, current_timestamp: i64) -> Result<u64> {
        self.update_pending_rewards(current_timestamp)?;
        
        let rewards_to_claim = self.pending_rewards;
        require!(rewards_to_claim > 0, FinovaTokenError::NoRewardsToClaim);
        
        self.pending_rewards = 0;
        self.total_rewards_claimed += rewards_to_claim;
        
        Ok(rewards_to_claim)
    }

    /// Compound rewards (add to staked amount)
    pub fn compound_rewards(&mut self, current_timestamp: i64) -> Result<u64> {
        require!(self.auto_compound, FinovaTokenError::AutoCompoundDisabled);
        require!(
            current_timestamp >= self.last_compound_timestamp + self.compound_frequency as i64,
            FinovaTokenError::CompoundTooEarly
        );
        
        self.update_pending_rewards(current_timestamp)?;
        
        let rewards_to_compound = self.pending_rewards;
        if rewards_to_compound == 0 {
            return Ok(0);
        }
        
        // Add rewards to staked amount
        self.staked_amount += rewards_to_compound;
        self.sfin_amount += rewards_to_compound; // Maintain 1:1 ratio for simplicity
        self.pending_rewards = 0;
        self.compound_count += 1;
        self.last_compound_timestamp = current_timestamp;
        
        // Update tier if necessary
        let new_tier = self.calculate_staking_tier(self.staked_amount);
        if new_tier != self.staking_tier {
            self.staking_tier = new_tier;
            self.base_apy = self.get_base_apy_for_tier(new_tier);
            self.unlocked_features = self.get_tier_features(new_tier);
            self.recalculate_effective_apy();
        }
        
        Ok(rewards_to_compound)
    }

    /// Add more stake
    pub fn add_stake(&mut self, additional_amount: u64, current_timestamp: i64) -> Result<()> {
        require!(additional_amount > 0, FinovaTokenError::InvalidAmount);
        
        // Update pending rewards before adding stake
        self.update_pending_rewards(current_timestamp)?;
        
        self.staked_amount += additional_amount;
        self.sfin_amount += additional_amount;
        
        // Update tier if necessary
        let new_tier = self.calculate_staking_tier(self.staked_amount);
        if new_tier != self.staking_tier {
            self.staking_tier = new_tier;
            self.base_apy = self.get_base_apy_for_tier(new_tier);
            self.unlocked_features = self.get_tier_features(new_tier);
            self.recalculate_effective_apy();
        }
        
        Ok(())
    }

    /// Initiate unstaking (with potential lock period)
    pub fn initiate_unstake(&mut self, amount: u64, current_timestamp: i64) -> Result<i64> {
        require!(amount > 0, FinovaTokenError::InvalidAmount);
        require!(amount <= self.staked_amount, FinovaTokenError::InsufficientBalance);
        require!(current_timestamp >= self.unlock_timestamp, FinovaTokenError::StakeLocked);
        
        // Check if remaining stake meets minimum requirement
        if self.staked_amount - amount < MIN_STAKE_AMOUNT && self.staked_amount - amount > 0 {
            return Err(FinovaTokenError::BelowMinimumStake.into());
        }
        
        // Update pending rewards
        self.update_pending_rewards(current_timestamp)?;
        
        // Calculate unstaking timestamp (immediate for no lock period)
        let unstake_timestamp = if self.lock_period > 0 {
            current_timestamp + self.lock_period as i64
        } else {
            current_timestamp
        };
        
        // For this implementation, we'll process immediate unstaking
        // In a full implementation, you'd create an unstaking queue
        self.staked_amount -= amount;
        self.sfin_amount -= amount;
        
        // Update tier if necessary
        if self.staked_amount > 0 {
            let new_tier = self.calculate_staking_tier(self.staked_amount);
            if new_tier != self.staking_tier {
                self.staking_tier = new_tier;
                self.base_apy = self.get_base_apy_for_tier(new_tier);
                self.unlocked_features = self.get_tier_features(new_tier);
                self.recalculate_effective_apy();
            }
        }
        
        Ok(unstake_timestamp)
    }

    /// Emergency unstake with penalty
    pub fn emergency_unstake(&mut self, amount: u64, current_timestamp: i64) -> Result<(u64, u64)> {
        require!(self.emergency_withdrawal_available, FinovaTokenError::EmergencyWithdrawalDisabled);
        require!(amount > 0, FinovaTokenError::InvalidAmount);
        require!(amount <= self.staked_amount, FinovaTokenError::InsufficientBalance);
        
        // Update pending rewards
        self.update_pending_rewards(current_timestamp)?;
        
        // Calculate penalty (default 5% for emergency withdrawal)
        let penalty_amount = (amount * EMERGENCY_WITHDRAWAL_PENALTY) / 10000;
        let amount_after_penalty = amount - penalty_amount;
        
        self.staked_amount -= amount;
        self.sfin_amount -= amount;
        
        // Update tier if necessary
        if self.staked_amount > 0 {
            let new_tier = self.calculate_staking_tier(self.staked_amount);
            if new_tier != self.staking_tier {
                self.staking_tier = new_tier;
                self.base_apy = self.get_base_apy_for_tier(new_tier);
                self.unlocked_features = self.get_tier_features(new_tier);
                self.recalculate_effective_apy();
            }
        }
        
        Ok((amount_after_penalty, penalty_amount))
    }

    /// Set lock period for enhanced rewards
    pub fn set_lock_period(&mut self, lock_seconds: u64, current_timestamp: i64) -> Result<()> {
        require!(lock_seconds <= MAX_LOCK_PERIOD, FinovaTokenError::LockPeriodTooLong);
        
        self.lock_period = lock_seconds;
        self.unlock_timestamp = current_timestamp + lock_seconds as i64;
        
        // Apply lock bonus to APY
        let lock_bonus = match lock_seconds {
            0 => 0,
            1..=2629746 => 10, // 1 month: +0.1%
            2629747..=7889238 => 25, // 3 months: +0.25%
            7889239..=15778476 => 50, // 6 months: +0.5%
            15778477..=31556952 => 100, // 1 year: +1%
            _ => 150, // >1 year: +1.5%
        };
        
        self.base_apy += lock_bonus;
        self.recalculate_effective_apy();
        
        Ok(())
    }

    /// Toggle auto-compound
    pub fn toggle_auto_compound(&mut self, enabled: bool, frequency: Option<u64>) -> Result<()> {
        self.auto_compound = enabled;
        
        if let Some(freq) = frequency {
            require!(freq >= MIN_COMPOUND_FREQUENCY, FinovaTokenError::CompoundFrequencyTooLow);
            require!(freq <= MAX_COMPOUND_FREQUENCY, FinovaTokenError::CompoundFrequencyTooHigh);
            self.compound_frequency = freq;
        }
        
        Ok(())
    }

    /// Update performance score based on user behavior
    pub fn update_performance_score(&mut self, score: u16) -> Result<()> {
        self.performance_score = score.min(200); // Cap at 2.0x
        
        // Apply performance bonus to effective APY
        if score > 100 {
            let performance_bonus = (score - 100) / 2; // 0.5x bonus for every 1.0x performance
            self.base_apy += performance_bonus;
            self.recalculate_effective_apy();
        }
        
        Ok(())
    }

    /// Update risk score for security
    pub fn update_risk_score(&mut self, score: u8) -> Result<()> {
        self.risk_score = score;
        
        // Disable emergency withdrawal for high-risk accounts
        if score > HIGH_RISK_THRESHOLD {
            self.emergency_withdrawal_available = false;
        }
        
        // Apply penalty for suspicious behavior
        if score > PENALTY_RISK_THRESHOLD {
            self.penalty_rate = (score as u16 - PENALTY_RISK_THRESHOLD as u16) * 10; // 0.1% penalty per risk point
        }
        
        Ok(())
    }

    /// Check if account has feature unlocked
    pub fn has_feature(&self, feature_bit: u8) -> bool {
        (self.unlocked_features & (1 << feature_bit)) != 0
    }

    /// Get current stake value in USD (would need oracle integration)
    pub fn get_stake_value_usd(&self, fin_price_usd: u64) -> u64 {
        (self.staked_amount * fin_price_usd) / PRICE_PRECISION
    }

    /// Get estimated annual rewards
    pub fn get_estimated_annual_rewards(&self) -> u64 {
        (self.staked_amount as u128 * self.effective_apy as u128 / 10000) as u64
    }

    /// Validate account state
    pub fn validate(&self) -> Result<()> {
        require!(self.staked_amount >= MIN_STAKE_AMOUNT, FinovaTokenError::BelowMinimumStake);
        require!(self.sfin_amount > 0, FinovaTokenError::InvalidSfinAmount);
        require!(self.staking_tier <= 4, FinovaTokenError::InvalidStakingTier);
        require!(self.effective_apy <= MAX_APY, FinovaTokenError::ApyTooHigh);
        Ok(())
    }
}

/// Staking statistics and global state
#[account]
pub struct StakingStats {
    /// Total amount staked across all users
    pub total_staked: u64,
    
    /// Total sFIN tokens in circulation
    pub total_sfin_supply: u64,
    
    /// Number of active stakers
    pub active_stakers: u32,
    
    /// Average staking duration
    pub average_stake_duration: u64,
    
    /// Total rewards distributed
    pub total_rewards_distributed: u64,
    
    /// Current global APY modifier
    pub global_apy_modifier: u16,
    
    /// Last update timestamp
    pub last_update: i64,
    
    /// Reserved for future use
    pub reserved: [u8; 64],
    
    /// Account bump
    pub bump: u8,
}

impl StakingStats {
    pub const SIZE: usize = 8 + // discriminator
        8 + // total_staked
        8 + // total_sfin_supply
        4 + // active_stakers
        8 + // average_stake_duration
        8 + // total_rewards_distributed
        2 + // global_apy_modifier
        8 + // last_update
        64 + // reserved
        1; // bump

    pub fn initialize(&mut self, bump: u8) -> Result<()> {
        self.total_staked = 0;
        self.total_sfin_supply = 0;
        self.active_stakers = 0;
        self.average_stake_duration = 0;
        self.total_rewards_distributed = 0;
        self.global_apy_modifier = 100; // 1.0x default
        self.last_update = Clock::get()?.unix_timestamp;
        self.reserved = [0; 64];
        self.bump = bump;
        Ok(())
    }

    pub fn update_stats(&mut self, stake_change: i64, staker_change: i32, rewards_distributed: u64) -> Result<()> {
        if stake_change > 0 {
            self.total_staked += stake_change as u64;
        } else if stake_change < 0 {
            self.total_staked = self.total_staked.saturating_sub((-stake_change) as u64);
        }
        
        if staker_change > 0 {
            self.active_stakers += staker_change as u32;
        } else if staker_change < 0 {
            self.active_stakers = self.active_stakers.saturating_sub((-staker_change) as u32);
        }
        
        self.total_rewards_distributed += rewards_distributed;
        self.last_update = Clock::get()?.unix_timestamp;
        
        Ok(())
    }
}
