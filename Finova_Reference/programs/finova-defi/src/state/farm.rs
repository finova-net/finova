// programs/finova-defi/src/state/farm.rs

use anchor_lang::prelude::*;
use crate::constants::*;
use crate::errors::FinovaDefiError;

/// Yield farming pool state for staking LP tokens and earning rewards
#[account]
#[derive(Debug)]
pub struct Farm {
    /// Farm identifier
    pub farm_id: u64,
    /// Authority that can update farm parameters
    pub authority: Pubkey,
    /// LP token mint that can be staked
    pub lp_token_mint: Pubkey,
    /// Reward token mint that is distributed
    pub reward_token_mint: Pubkey,
    /// Vault storing staked LP tokens
    pub lp_token_vault: Pubkey,
    /// Vault storing reward tokens
    pub reward_token_vault: Pubkey,
    /// Total LP tokens staked in this farm
    pub total_staked: u64,
    /// Total reward tokens allocated to this farm
    pub total_rewards: u64,
    /// Rewards per second distributed by this farm
    pub reward_rate: u64,
    /// Timestamp when farming started
    pub start_time: i64,
    /// Timestamp when farming ends
    pub end_time: i64,
    /// Last time rewards were calculated
    pub last_update_time: i64,
    /// Accumulated reward per staked token (scaled by PRECISION)
    pub reward_per_token_stored: u128,
    /// Minimum staking period in seconds
    pub lock_duration: i64,
    /// Early withdrawal penalty percentage (scaled by 10000)
    pub early_withdrawal_penalty: u16,
    /// Farm status (active, paused, closed)
    pub status: FarmStatus,
    /// Performance fee percentage for farm operations (scaled by 10000)
    pub performance_fee: u16,
    /// Treasury address for collecting fees
    pub treasury: Pubkey,
    /// Maximum staking amount per user
    pub max_stake_per_user: u64,
    /// Total number of stakers
    pub staker_count: u32,
    /// Farm multiplier for boosted rewards (scaled by 10000)
    pub multiplier: u16,
    /// Reserved space for future upgrades
    pub reserved: [u8; 128],
}

impl Farm {
    pub const LEN: usize = 8 + // discriminator
        8 + // farm_id
        32 + // authority
        32 + // lp_token_mint
        32 + // reward_token_mint
        32 + // lp_token_vault
        32 + // reward_token_vault
        8 + // total_staked
        8 + // total_rewards
        8 + // reward_rate
        8 + // start_time
        8 + // end_time
        8 + // last_update_time
        16 + // reward_per_token_stored
        8 + // lock_duration
        2 + // early_withdrawal_penalty
        1 + // status
        2 + // performance_fee
        32 + // treasury
        8 + // max_stake_per_user
        4 + // staker_count
        2 + // multiplier
        128; // reserved

    /// Initialize a new farm
    pub fn initialize(
        &mut self,
        farm_id: u64,
        authority: Pubkey,
        lp_token_mint: Pubkey,
        reward_token_mint: Pubkey,
        lp_token_vault: Pubkey,
        reward_token_vault: Pubkey,
        reward_rate: u64,
        start_time: i64,
        end_time: i64,
        lock_duration: i64,
        early_withdrawal_penalty: u16,
        performance_fee: u16,
        treasury: Pubkey,
        max_stake_per_user: u64,
        multiplier: u16,
    ) -> Result<()> {
        require!(
            end_time > start_time,
            FinovaDefiError::InvalidFarmDuration
        );
        require!(
            early_withdrawal_penalty <= MAX_PENALTY_BPS,
            FinovaDefiError::InvalidPenalty
        );
        require!(
            performance_fee <= MAX_FEE_BPS,
            FinovaDefiError::InvalidFee
        );
        require!(
            multiplier >= MIN_MULTIPLIER && multiplier <= MAX_MULTIPLIER,
            FinovaDefiError::InvalidMultiplier
        );

        let current_time = Clock::get()?.unix_timestamp;
        
        self.farm_id = farm_id;
        self.authority = authority;
        self.lp_token_mint = lp_token_mint;
        self.reward_token_mint = reward_token_mint;
        self.lp_token_vault = lp_token_vault;
        self.reward_token_vault = reward_token_vault;
        self.total_staked = 0;
        self.total_rewards = 0;
        self.reward_rate = reward_rate;
        self.start_time = start_time;
        self.end_time = end_time;
        self.last_update_time = std::cmp::max(current_time, start_time);
        self.reward_per_token_stored = 0;
        self.lock_duration = lock_duration;
        self.early_withdrawal_penalty = early_withdrawal_penalty;
        self.status = FarmStatus::Active;
        self.performance_fee = performance_fee;
        self.treasury = treasury;
        self.max_stake_per_user = max_stake_per_user;
        self.staker_count = 0;
        self.multiplier = multiplier;
        self.reserved = [0; 128];

        Ok(())
    }

    /// Update reward per token calculation
    pub fn update_reward_per_token(&mut self) -> Result<()> {
        let current_time = Clock::get()?.unix_timestamp;
        
        if self.total_staked == 0 {
            self.last_update_time = current_time;
            return Ok(());
        }

        let effective_end_time = std::cmp::min(current_time, self.end_time);
        let time_elapsed = std::cmp::max(0, effective_end_time - self.last_update_time);
        
        if time_elapsed > 0 {
            let reward_to_distribute = (self.reward_rate as u128)
                .checked_mul(time_elapsed as u128)
                .ok_or(FinovaDefiError::MathOverflow)?;
            
            let boosted_reward = reward_to_distribute
                .checked_mul(self.multiplier as u128)
                .ok_or(FinovaDefiError::MathOverflow)?
                .checked_div(BASIS_POINTS as u128)
                .ok_or(FinovaDefiError::MathOverflow)?;
            
            let reward_per_token_increment = boosted_reward
                .checked_mul(PRECISION)
                .ok_or(FinovaDefiError::MathOverflow)?
                .checked_div(self.total_staked as u128)
                .ok_or(FinovaDefiError::MathOverflow)?;
            
            self.reward_per_token_stored = self.reward_per_token_stored
                .checked_add(reward_per_token_increment)
                .ok_or(FinovaDefiError::MathOverflow)?;
        }
        
        self.last_update_time = current_time;
        Ok(())
    }

    /// Calculate pending rewards for a user
    pub fn calculate_pending_rewards(
        &self,
        user_staked_amount: u64,
        user_reward_per_token_paid: u128,
        user_pending_rewards: u64,
    ) -> Result<u64> {
        if user_staked_amount == 0 {
            return Ok(user_pending_rewards);
        }

        let reward_per_token_diff = self.reward_per_token_stored
            .checked_sub(user_reward_per_token_paid)
            .ok_or(FinovaDefiError::MathOverflow)?;
        
        let earned_rewards = (user_staked_amount as u128)
            .checked_mul(reward_per_token_diff)
            .ok_or(FinovaDefiError::MathOverflow)?
            .checked_div(PRECISION)
            .ok_or(FinovaDefiError::MathOverflow)?;
        
        let total_rewards = (user_pending_rewards as u128)
            .checked_add(earned_rewards)
            .ok_or(FinovaDefiError::MathOverflow)?;
        
        Ok(total_rewards as u64)
    }

    /// Add stake to the farm
    pub fn add_stake(&mut self, amount: u64) -> Result<()> {
        require!(
            self.status == FarmStatus::Active,
            FinovaDefiError::FarmNotActive
        );
        
        let current_time = Clock::get()?.unix_timestamp;
        require!(
            current_time >= self.start_time && current_time < self.end_time,
            FinovaDefiError::FarmNotActive
        );

        self.total_staked = self.total_staked
            .checked_add(amount)
            .ok_or(FinovaDefiError::MathOverflow)?;
        
        Ok(())
    }

    /// Remove stake from the farm
    pub fn remove_stake(&mut self, amount: u64) -> Result<()> {
        require!(
            self.total_staked >= amount,
            FinovaDefiError::InsufficientStake
        );
        
        self.total_staked = self.total_staked
            .checked_sub(amount)
            .ok_or(FinovaDefiError::MathOverflow)?;
        
        Ok(())
    }

    /// Calculate early withdrawal penalty
    pub fn calculate_withdrawal_penalty(
        &self,
        amount: u64,
        stake_time: i64,
    ) -> Result<u64> {
        let current_time = Clock::get()?.unix_timestamp;
        let time_staked = current_time - stake_time;
        
        if time_staked >= self.lock_duration {
            return Ok(0);
        }
        
        let penalty_amount = (amount as u128)
            .checked_mul(self.early_withdrawal_penalty as u128)
            .ok_or(FinovaDefiError::MathOverflow)?
            .checked_div(BASIS_POINTS as u128)
            .ok_or(FinovaDefiError::MathOverflow)?;
        
        Ok(penalty_amount as u64)
    }

    /// Calculate performance fee on rewards
    pub fn calculate_performance_fee(&self, reward_amount: u64) -> Result<u64> {
        let fee_amount = (reward_amount as u128)
            .checked_mul(self.performance_fee as u128)
            .ok_or(FinovaDefiError::MathOverflow)?
            .checked_div(BASIS_POINTS as u128)
            .ok_or(FinovaDefiError::MathOverflow)?;
        
        Ok(fee_amount as u64)
    }

    /// Update farm parameters (only authority)
    pub fn update_parameters(
        &mut self,
        new_reward_rate: Option<u64>,
        new_end_time: Option<i64>,
        new_early_withdrawal_penalty: Option<u16>,
        new_performance_fee: Option<u16>,
        new_max_stake_per_user: Option<u64>,
        new_multiplier: Option<u16>,
    ) -> Result<()> {
        if let Some(reward_rate) = new_reward_rate {
            self.reward_rate = reward_rate;
        }
        
        if let Some(end_time) = new_end_time {
            require!(
                end_time > Clock::get()?.unix_timestamp,
                FinovaDefiError::InvalidFarmDuration
            );
            self.end_time = end_time;
        }
        
        if let Some(penalty) = new_early_withdrawal_penalty {
            require!(
                penalty <= MAX_PENALTY_BPS,
                FinovaDefiError::InvalidPenalty
            );
            self.early_withdrawal_penalty = penalty;
        }
        
        if let Some(fee) = new_performance_fee {
            require!(
                fee <= MAX_FEE_BPS,
                FinovaDefiError::InvalidFee
            );
            self.performance_fee = fee;
        }
        
        if let Some(max_stake) = new_max_stake_per_user {
            self.max_stake_per_user = max_stake;
        }
        
        if let Some(multiplier) = new_multiplier {
            require!(
                multiplier >= MIN_MULTIPLIER && multiplier <= MAX_MULTIPLIER,
                FinovaDefiError::InvalidMultiplier
            );
            self.multiplier = multiplier;
        }
        
        Ok(())
    }

    /// Pause/unpause the farm
    pub fn set_status(&mut self, status: FarmStatus) -> Result<()> {
        self.status = status;
        Ok(())
    }

    /// Add rewards to the farm
    pub fn add_rewards(&mut self, amount: u64) -> Result<()> {
        self.total_rewards = self.total_rewards
            .checked_add(amount)
            .ok_or(FinovaDefiError::MathOverflow)?;
        Ok(())
    }

    /// Remove rewards from the farm (emergency only)
    pub fn remove_rewards(&mut self, amount: u64) -> Result<()> {
        require!(
            self.total_rewards >= amount,
            FinovaDefiError::InsufficientRewards
        );
        
        self.total_rewards = self.total_rewards
            .checked_sub(amount)
            .ok_or(FinovaDefiError::MathOverflow)?;
        Ok(())
    }

    /// Increment staker count
    pub fn increment_staker_count(&mut self) -> Result<()> {
        self.staker_count = self.staker_count
            .checked_add(1)
            .ok_or(FinovaDefiError::MathOverflow)?;
        Ok(())
    }

    /// Decrement staker count
    pub fn decrement_staker_count(&mut self) -> Result<()> {
        require!(
            self.staker_count > 0,
            FinovaDefiError::InvalidStakerCount
        );
        
        self.staker_count = self.staker_count
            .checked_sub(1)
            .ok_or(FinovaDefiError::MathOverflow)?;
        Ok(())
    }

    /// Check if farm is currently active for staking
    pub fn is_active_for_staking(&self) -> Result<bool> {
        let current_time = Clock::get()?.unix_timestamp;
        Ok(self.status == FarmStatus::Active 
            && current_time >= self.start_time 
            && current_time < self.end_time)
    }

    /// Check if farm has ended
    pub fn has_ended(&self) -> Result<bool> {
        let current_time = Clock::get()?.unix_timestamp;
        Ok(current_time >= self.end_time)
    }

    /// Get current APR (Annual Percentage Rate)
    pub fn calculate_apr(&self) -> Result<u64> {
        if self.total_staked == 0 {
            return Ok(0);
        }

        let annual_rewards = self.reward_rate
            .checked_mul(SECONDS_PER_YEAR)
            .ok_or(FinovaDefiError::MathOverflow)?;
        
        let boosted_annual_rewards = annual_rewards
            .checked_mul(self.multiplier as u64)
            .ok_or(FinovaDefiError::MathOverflow)?
            .checked_div(BASIS_POINTS as u64)
            .ok_or(FinovaDefiError::MathOverflow)?;
        
        let apr = boosted_annual_rewards
            .checked_mul(BASIS_POINTS as u64)
            .ok_or(FinovaDefiError::MathOverflow)?
            .checked_div(self.total_staked)
            .ok_or(FinovaDefiError::MathOverflow)?;
        
        Ok(apr)
    }

    /// Get time remaining until farm ends
    pub fn time_until_end(&self) -> Result<i64> {
        let current_time = Clock::get()?.unix_timestamp;
        Ok(std::cmp::max(0, self.end_time - current_time))
    }

    /// Validate farm configuration
    pub fn validate_config(&self) -> Result<()> {
        require!(
            self.end_time > self.start_time,
            FinovaDefiError::InvalidFarmDuration
        );
        require!(
            self.early_withdrawal_penalty <= MAX_PENALTY_BPS,
            FinovaDefiError::InvalidPenalty
        );
        require!(
            self.performance_fee <= MAX_FEE_BPS,
            FinovaDefiError::InvalidFee
        );
        require!(
            self.multiplier >= MIN_MULTIPLIER && self.multiplier <= MAX_MULTIPLIER,
            FinovaDefiError::InvalidMultiplier
        );
        require!(
            self.lock_duration >= 0,
            FinovaDefiError::InvalidLockDuration
        );
        
        Ok(())
    }
}

/// Farm status enumeration
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq)]
pub enum FarmStatus {
    /// Farm is active and accepting stakes
    Active,
    /// Farm is paused (no new stakes, but rewards still accrue)
    Paused,
    /// Farm is closed (no new stakes, no rewards)
    Closed,
    /// Farm is in emergency mode (withdrawals only)
    Emergency,
}

impl Default for FarmStatus {
    fn default() -> Self {
        FarmStatus::Active
    }
}

/// User's farming position
#[account]
#[derive(Debug)]
pub struct FarmPosition {
    /// Farm this position belongs to
    pub farm: Pubkey,
    /// User who owns this position
    pub user: Pubkey,
    /// Amount of LP tokens staked
    pub staked_amount: u64,
    /// Reward per token paid when last updated
    pub reward_per_token_paid: u128,
    /// Pending rewards not yet claimed
    pub pending_rewards: u64,
    /// Timestamp when position was created
    pub created_at: i64,
    /// Timestamp when position was last updated
    pub last_updated_at: i64,
    /// Total rewards claimed by this position
    pub total_rewards_claimed: u64,
    /// Number of times user has compounded rewards
    pub compound_count: u32,
    /// Lock end time for this position
    pub lock_end_time: i64,
    /// Reserved space for future upgrades
    pub reserved: [u8; 64],
}

impl FarmPosition {
    pub const LEN: usize = 8 + // discriminator
        32 + // farm
        32 + // user
        8 + // staked_amount
        16 + // reward_per_token_paid
        8 + // pending_rewards
        8 + // created_at
        8 + // last_updated_at
        8 + // total_rewards_claimed
        4 + // compound_count
        8 + // lock_end_time
        64; // reserved

    /// Initialize a new farm position
    pub fn initialize(
        &mut self,
        farm: Pubkey,
        user: Pubkey,
        current_reward_per_token: u128,
        lock_duration: i64,
    ) -> Result<()> {
        let current_time = Clock::get()?.unix_timestamp;
        
        self.farm = farm;
        self.user = user;
        self.staked_amount = 0;
        self.reward_per_token_paid = current_reward_per_token;
        self.pending_rewards = 0;
        self.created_at = current_time;
        self.last_updated_at = current_time;
        self.total_rewards_claimed = 0;
        self.compound_count = 0;
        self.lock_end_time = current_time + lock_duration;
        self.reserved = [0; 64];
        
        Ok(())
    }

    /// Update position with new stake
    pub fn add_stake(
        &mut self,
        amount: u64,
        current_reward_per_token: u128,
        lock_duration: i64,
    ) -> Result<()> {
        // Update pending rewards before changing stake
        self.update_pending_rewards(current_reward_per_token)?;
        
        self.staked_amount = self.staked_amount
            .checked_add(amount)
            .ok_or(FinovaDefiError::MathOverflow)?;
        
        // Extend lock time if adding more stake
        let current_time = Clock::get()?.unix_timestamp;
        let new_lock_end = current_time + lock_duration;
        if new_lock_end > self.lock_end_time {
            self.lock_end_time = new_lock_end;
        }
        
        self.last_updated_at = current_time;
        Ok(())
    }

    /// Update position with stake removal
    pub fn remove_stake(
        &mut self,
        amount: u64,
        current_reward_per_token: u128,
    ) -> Result<()> {
        require!(
            self.staked_amount >= amount,
            FinovaDefiError::InsufficientStake
        );
        
        // Update pending rewards before changing stake
        self.update_pending_rewards(current_reward_per_token)?;
        
        self.staked_amount = self.staked_amount
            .checked_sub(amount)
            .ok_or(FinovaDefiError::MathOverflow)?;
        
        self.last_updated_at = Clock::get()?.unix_timestamp;
        Ok(())
    }

    /// Update pending rewards calculation
    pub fn update_pending_rewards(&mut self, current_reward_per_token: u128) -> Result<()> {
        if self.staked_amount == 0 {
            self.reward_per_token_paid = current_reward_per_token;
            return Ok(());
        }

        let reward_per_token_diff = current_reward_per_token
            .checked_sub(self.reward_per_token_paid)
            .ok_or(FinovaDefiError::MathOverflow)?;
        
        let earned_rewards = (self.staked_amount as u128)
            .checked_mul(reward_per_token_diff)
            .ok_or(FinovaDefiError::MathOverflow)?
            .checked_div(PRECISION)
            .ok_or(FinovaDefiError::MathOverflow)?;
        
        self.pending_rewards = self.pending_rewards
            .checked_add(earned_rewards as u64)
            .ok_or(FinovaDefiError::MathOverflow)?;
        
        self.reward_per_token_paid = current_reward_per_token;
        self.last_updated_at = Clock::get()?.unix_timestamp;
        
        Ok(())
    }

    /// Claim rewards
    pub fn claim_rewards(&mut self, amount: u64) -> Result<()> {
        require!(
            self.pending_rewards >= amount,
            FinovaDefiError::InsufficientRewards
        );
        
        self.pending_rewards = self.pending_rewards
            .checked_sub(amount)
            .ok_or(FinovaDefiError::MathOverflow)?;
        
        self.total_rewards_claimed = self.total_rewards_claimed
            .checked_add(amount)
            .ok_or(FinovaDefiError::MathOverflow)?;
        
        self.last_updated_at = Clock::get()?.unix_timestamp;
        Ok(())
    }

    /// Compound rewards (restake as LP tokens)
    pub fn compound_rewards(&mut self, compounded_amount: u64) -> Result<()> {
        require!(
            self.pending_rewards >= compounded_amount,
            FinovaDefiError::InsufficientRewards
        );
        
        self.pending_rewards = self.pending_rewards
            .checked_sub(compounded_amount)
            .ok_or(FinovaDefiError::MathOverflow)?;
        
        self.compound_count = self.compound_count
            .checked_add(1)
            .ok_or(FinovaDefiError::MathOverflow)?;
        
        self.last_updated_at = Clock::get()?.unix_timestamp;
        Ok(())
    }

    /// Check if position is locked
    pub fn is_locked(&self) -> Result<bool> {
        let current_time = Clock::get()?.unix_timestamp;
        Ok(current_time < self.lock_end_time)
    }

    /// Get time remaining until unlock
    pub fn time_until_unlock(&self) -> Result<i64> {
        let current_time = Clock::get()?.unix_timestamp;
        Ok(std::cmp::max(0, self.lock_end_time - current_time))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_farm_initialization() {
        let mut farm = Farm::default();
        let result = farm.initialize(
            1,
            Pubkey::default(),
            Pubkey::default(),
            Pubkey::default(),
            Pubkey::default(),
            Pubkey::default(),
            100,
            1000,
            2000,
            86400,
            500,
            200,
            Pubkey::default(),
            1000000,
            10000,
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_reward_calculation() {
        let farm = Farm {
            reward_per_token_stored: 1000000000000000000, // 1e18
            ..Default::default()
        };
        
        let result = farm.calculate_pending_rewards(
            1000000, // 1M tokens staked
            500000000000000000, // 0.5e18 paid
            0
        );
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 500000); // Should be 0.5M rewards
    }
}

