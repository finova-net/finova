// programs/finova-defi/src/state/liquidity_position.rs

use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount, Mint};
use crate::constants::*;
use crate::errors::FinovaDeFiError;

/// Liquidity Position Account
/// Represents a user's liquidity position in a specific pool
#[account]
#[derive(Debug)]
pub struct LiquidityPosition {
    /// The owner of this liquidity position
    pub owner: Pubkey,
    
    /// The pool this position belongs to
    pub pool: Pubkey,
    
    /// The mint address of the LP token
    pub lp_token_mint: Pubkey,
    
    /// Amount of LP tokens owned
    pub lp_token_amount: u64,
    
    /// Amount of token A when position was created
    pub token_a_deposited: u64,
    
    /// Amount of token B when position was created
    pub token_b_deposited: u64,
    
    /// Timestamp when position was created
    pub created_at: i64,
    
    /// Last time position was updated
    pub updated_at: i64,
    
    /// Accumulated fees earned (token A)
    pub fees_earned_a: u64,
    
    /// Accumulated fees earned (token B)
    pub fees_earned_b: u64,
    
    /// Accumulated yield farming rewards
    pub yield_rewards_earned: u64,
    
    /// Position status
    pub status: PositionStatus,
    
    /// Lock period for additional rewards (in seconds)
    pub lock_period: u64,
    
    /// When the lock period ends
    pub lock_expires_at: i64,
    
    /// Boost multiplier for locked positions (basis points)
    pub boost_multiplier: u16,
    
    /// Impermanent loss tracking
    pub impermanent_loss: i64,
    
    /// Total value when position was opened (in USD, 6 decimals)
    pub initial_usd_value: u64,
    
    /// Reserved space for future upgrades
    pub reserved: [u8; 128],
}

impl LiquidityPosition {
    pub const LEN: usize = 8 + // discriminator
        32 + // owner
        32 + // pool
        32 + // lp_token_mint
        8 + // lp_token_amount
        8 + // token_a_deposited
        8 + // token_b_deposited
        8 + // created_at
        8 + // updated_at
        8 + // fees_earned_a
        8 + // fees_earned_b
        8 + // yield_rewards_earned
        1 + // status
        8 + // lock_period
        8 + // lock_expires_at
        2 + // boost_multiplier
        8 + // impermanent_loss
        8 + // initial_usd_value
        128; // reserved
}

/// Position Status enumeration
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq)]
pub enum PositionStatus {
    /// Position is active and earning rewards
    Active,
    /// Position is locked for additional rewards
    Locked,
    /// Position is being withdrawn
    Withdrawing,
    /// Position has been fully withdrawn
    Withdrawn,
    /// Position is paused (emergency)
    Paused,
}

impl Default for PositionStatus {
    fn default() -> Self {
        PositionStatus::Active
    }
}

/// Yield Farm Position Account
/// Tracks yield farming rewards for staked LP tokens
#[account]
#[derive(Debug)]
pub struct YieldFarmPosition {
    /// The owner of this farm position
    pub owner: Pubkey,
    
    /// The yield farm this position belongs to
    pub farm: Pubkey,
    
    /// Associated liquidity position
    pub liquidity_position: Pubkey,
    
    /// Amount of LP tokens staked in farm
    pub staked_amount: u64,
    
    /// Timestamp when farming started
    pub started_at: i64,
    
    /// Last reward calculation timestamp
    pub last_reward_at: i64,
    
    /// Accumulated rewards (in farm token)
    pub rewards_earned: u64,
    
    /// Pending rewards to be claimed
    pub pending_rewards: u64,
    
    /// Reward debt for precision tracking
    pub reward_debt: u128,
    
    /// Farm position status
    pub status: FarmPositionStatus,
    
    /// Lock multiplier applied (basis points)
    pub lock_multiplier: u16,
    
    /// When lock period expires
    pub lock_expires_at: i64,
    
    /// Total rewards claimed
    pub total_claimed: u64,
    
    /// Farm tier level (affects rewards)
    pub tier_level: u8,
    
    /// Boost from NFT cards
    pub nft_boost: u16,
    
    /// Reserved space for future upgrades
    pub reserved: [u8; 96],
}

impl YieldFarmPosition {
    pub const LEN: usize = 8 + // discriminator
        32 + // owner
        32 + // farm
        32 + // liquidity_position
        8 + // staked_amount
        8 + // started_at
        8 + // last_reward_at
        8 + // rewards_earned
        8 + // pending_rewards
        16 + // reward_debt
        1 + // status
        2 + // lock_multiplier
        8 + // lock_expires_at
        8 + // total_claimed
        1 + // tier_level
        2 + // nft_boost
        96; // reserved
}

/// Farm Position Status enumeration
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq)]
pub enum FarmPositionStatus {
    /// Position is actively farming
    Active,
    /// Position is locked for bonus rewards
    Locked,
    /// Position is being unstaked
    Unstaking,
    /// Position has been unstaked
    Unstaked,
    /// Position is paused
    Paused,
}

impl Default for FarmPositionStatus {
    fn default() -> Self {
        FarmPositionStatus::Active
    }
}

/// Staking Tier Configuration
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug)]
pub struct StakingTier {
    /// Minimum stake amount required
    pub min_stake: u64,
    
    /// Reward multiplier (basis points)
    pub reward_multiplier: u16,
    
    /// Lock period required (seconds)
    pub required_lock_period: u64,
    
    /// Tier benefits flags
    pub benefits: u32,
}

/// Position Snapshot for historical tracking
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct PositionSnapshot {
    /// Timestamp of snapshot
    pub timestamp: i64,
    
    /// LP token amount at snapshot
    pub lp_amount: u64,
    
    /// Token A amount at snapshot
    pub token_a_amount: u64,
    
    /// Token B amount at snapshot
    pub token_b_amount: u64,
    
    /// USD value at snapshot
    pub usd_value: u64,
    
    /// Pool price at snapshot
    pub pool_price: u64,
    
    /// Impermanent loss at snapshot
    pub impermanent_loss: i64,
}

impl LiquidityPosition {
    /// Initialize a new liquidity position
    pub fn initialize(
        &mut self,
        owner: Pubkey,
        pool: Pubkey,
        lp_token_mint: Pubkey,
        lp_token_amount: u64,
        token_a_deposited: u64,
        token_b_deposited: u64,
        initial_usd_value: u64,
        clock: &Clock,
    ) -> Result<()> {
        self.owner = owner;
        self.pool = pool;
        self.lp_token_mint = lp_token_mint;
        self.lp_token_amount = lp_token_amount;
        self.token_a_deposited = token_a_deposited;
        self.token_b_deposited = token_b_deposited;
        self.created_at = clock.unix_timestamp;
        self.updated_at = clock.unix_timestamp;
        self.fees_earned_a = 0;
        self.fees_earned_b = 0;
        self.yield_rewards_earned = 0;
        self.status = PositionStatus::Active;
        self.lock_period = 0;
        self.lock_expires_at = 0;
        self.boost_multiplier = BASIS_POINTS_SCALE; // 100% = 10000 basis points
        self.impermanent_loss = 0;
        self.initial_usd_value = initial_usd_value;
        
        Ok(())
    }
    
    /// Update position with new LP token amount
    pub fn update_position(
        &mut self,
        new_lp_amount: u64,
        token_a_change: i64,
        token_b_change: i64,
        clock: &Clock,
    ) -> Result<()> {
        require!(
            self.status == PositionStatus::Active || self.status == PositionStatus::Locked,
            FinovaDeFiError::PositionNotActive
        );
        
        self.lp_token_amount = new_lp_amount;
        
        // Update deposited amounts
        if token_a_change >= 0 {
            self.token_a_deposited = self.token_a_deposited
                .checked_add(token_a_change as u64)
                .ok_or(FinovaDeFiError::MathOverflow)?;
        } else {
            self.token_a_deposited = self.token_a_deposited
                .checked_sub((-token_a_change) as u64)
                .ok_or(FinovaDeFiError::InsufficientBalance)?;
        }
        
        if token_b_change >= 0 {
            self.token_b_deposited = self.token_b_deposited
                .checked_add(token_b_change as u64)
                .ok_or(FinovaDeFiError::MathOverflow)?;
        } else {
            self.token_b_deposited = self.token_b_deposited
                .checked_sub((-token_b_change) as u64)
                .ok_or(FinovaDeFiError::InsufficientBalance)?;
        }
        
        self.updated_at = clock.unix_timestamp;
        
        Ok(())
    }
    
    /// Lock position for additional rewards
    pub fn lock_position(
        &mut self,
        lock_period: u64,
        boost_multiplier: u16,
        clock: &Clock,
    ) -> Result<()> {
        require!(
            self.status == PositionStatus::Active,
            FinovaDeFiError::PositionNotActive
        );
        
        require!(
            lock_period >= MIN_LOCK_PERIOD && lock_period <= MAX_LOCK_PERIOD,
            FinovaDeFiError::InvalidLockPeriod
        );
        
        self.status = PositionStatus::Locked;
        self.lock_period = lock_period;
        self.lock_expires_at = clock.unix_timestamp
            .checked_add(lock_period as i64)
            .ok_or(FinovaDeFiError::MathOverflow)?;
        self.boost_multiplier = boost_multiplier;
        self.updated_at = clock.unix_timestamp;
        
        Ok(())
    }
    
    /// Unlock position after lock period expires
    pub fn unlock_position(&mut self, clock: &Clock) -> Result<()> {
        require!(
            self.status == PositionStatus::Locked,
            FinovaDeFiError::PositionNotLocked
        );
        
        require!(
            clock.unix_timestamp >= self.lock_expires_at,
            FinovaDeFiError::LockPeriodNotExpired
        );
        
        self.status = PositionStatus::Active;
        self.lock_period = 0;
        self.lock_expires_at = 0;
        self.boost_multiplier = BASIS_POINTS_SCALE;
        self.updated_at = clock.unix_timestamp;
        
        Ok(())
    }
    
    /// Add fees earned to position
    pub fn add_fees(&mut self, fees_a: u64, fees_b: u64, clock: &Clock) -> Result<()> {
        self.fees_earned_a = self.fees_earned_a
            .checked_add(fees_a)
            .ok_or(FinovaDeFiError::MathOverflow)?;
        
        self.fees_earned_b = self.fees_earned_b
            .checked_add(fees_b)
            .ok_or(FinovaDeFiError::MathOverflow)?;
        
        self.updated_at = clock.unix_timestamp;
        
        Ok(())
    }
    
    /// Calculate current position value with impermanent loss
    pub fn calculate_current_value(
        &self,
        token_a_price: u64,
        token_b_price: u64,
        pool_token_a_amount: u64,
        pool_token_b_amount: u64,
        pool_lp_supply: u64,
    ) -> Result<(u64, i64)> {
        if pool_lp_supply == 0 {
            return Ok((0, 0));
        }
        
        // Calculate current token amounts based on LP token share
        let current_token_a = (pool_token_a_amount as u128)
            .checked_mul(self.lp_token_amount as u128)
            .ok_or(FinovaDeFiError::MathOverflow)?
            .checked_div(pool_lp_supply as u128)
            .ok_or(FinovaDeFiError::DivisionByZero)? as u64;
        
        let current_token_b = (pool_token_b_amount as u128)
            .checked_mul(self.lp_token_amount as u128)
            .ok_or(FinovaDeFiError::MathOverflow)?
            .checked_div(pool_lp_supply as u128)
            .ok_or(FinovaDeFiError::DivisionByZero)? as u64;
        
        // Calculate current USD value
        let current_usd_value = (current_token_a as u128)
            .checked_mul(token_a_price as u128)
            .ok_or(FinovaDeFiError::MathOverflow)?
            .checked_add(
                (current_token_b as u128)
                    .checked_mul(token_b_price as u128)
                    .ok_or(FinovaDeFiError::MathOverflow)?
            )
            .ok_or(FinovaDeFiError::MathOverflow)?
            .checked_div(PRICE_DECIMALS as u128)
            .ok_or(FinovaDeFiError::DivisionByZero)? as u64;
        
        // Calculate what value would be if just holding tokens
        let hold_value = (self.token_a_deposited as u128)
            .checked_mul(token_a_price as u128)
            .ok_or(FinovaDeFiError::MathOverflow)?
            .checked_add(
                (self.token_b_deposited as u128)
                    .checked_mul(token_b_price as u128)
                    .ok_or(FinovaDeFiError::MathOverflow)?
            )
            .ok_or(FinovaDeFiError::MathOverflow)?
            .checked_div(PRICE_DECIMALS as u128)
            .ok_or(FinovaDeFiError::DivisionByZero)? as u64;
        
        // Calculate impermanent loss
        let impermanent_loss = (current_usd_value as i128)
            .checked_sub(hold_value as i128)
            .ok_or(FinovaDeFiError::MathOverflow)? as i64;
        
        Ok((current_usd_value, impermanent_loss))
    }
    
    /// Check if position can be withdrawn
    pub fn can_withdraw(&self, clock: &Clock) -> bool {
        match self.status {
            PositionStatus::Active => true,
            PositionStatus::Locked => clock.unix_timestamp >= self.lock_expires_at,
            _ => false,
        }
    }
    
    /// Get effective boost multiplier
    pub fn get_effective_boost(&self, clock: &Clock) -> u16 {
        match self.status {
            PositionStatus::Locked => {
                if clock.unix_timestamp < self.lock_expires_at {
                    self.boost_multiplier
                } else {
                    BASIS_POINTS_SCALE
                }
            }
            _ => BASIS_POINTS_SCALE,
        }
    }
    
    /// Create position snapshot
    pub fn create_snapshot(
        &self,
        token_a_amount: u64,
        token_b_amount: u64,
        usd_value: u64,
        pool_price: u64,
        clock: &Clock,
    ) -> PositionSnapshot {
        PositionSnapshot {
            timestamp: clock.unix_timestamp,
            lp_amount: self.lp_token_amount,
            token_a_amount,
            token_b_amount,
            usd_value,
            pool_price,
            impermanent_loss: self.impermanent_loss,
        }
    }
}

impl YieldFarmPosition {
    /// Initialize a new yield farm position
    pub fn initialize(
        &mut self,
        owner: Pubkey,
        farm: Pubkey,
        liquidity_position: Pubkey,
        staked_amount: u64,
        tier_level: u8,
        clock: &Clock,
    ) -> Result<()> {
        self.owner = owner;
        self.farm = farm;
        self.liquidity_position = liquidity_position;
        self.staked_amount = staked_amount;
        self.started_at = clock.unix_timestamp;
        self.last_reward_at = clock.unix_timestamp;
        self.rewards_earned = 0;
        self.pending_rewards = 0;
        self.reward_debt = 0;
        self.status = FarmPositionStatus::Active;
        self.lock_multiplier = BASIS_POINTS_SCALE;
        self.lock_expires_at = 0;
        self.total_claimed = 0;
        self.tier_level = tier_level;
        self.nft_boost = 0;
        
        Ok(())
    }
    
    /// Update staked amount
    pub fn update_stake(&mut self, new_amount: u64, clock: &Clock) -> Result<()> {
        require!(
            self.status == FarmPositionStatus::Active || self.status == FarmPositionStatus::Locked,
            FinovaDeFiError::FarmPositionNotActive
        );
        
        self.staked_amount = new_amount;
        self.last_reward_at = clock.unix_timestamp;
        
        Ok(())
    }
    
    /// Update pending rewards
    pub fn update_rewards(
        &mut self,
        additional_rewards: u64,
        new_reward_debt: u128,
        clock: &Clock,
    ) -> Result<()> {
        self.pending_rewards = self.pending_rewards
            .checked_add(additional_rewards)
            .ok_or(FinovaDeFiError::MathOverflow)?;
        
        self.rewards_earned = self.rewards_earned
            .checked_add(additional_rewards)
            .ok_or(FinovaDeFiError::MathOverflow)?;
        
        self.reward_debt = new_reward_debt;
        self.last_reward_at = clock.unix_timestamp;
        
        Ok(())
    }
    
    /// Claim pending rewards
    pub fn claim_rewards(&mut self, amount: u64, clock: &Clock) -> Result<()> {
        require!(
            amount <= self.pending_rewards,
            FinovaDeFiError::InsufficientRewards
        );
        
        self.pending_rewards = self.pending_rewards
            .checked_sub(amount)
            .ok_or(FinovaDeFiError::InsufficientBalance)?;
        
        self.total_claimed = self.total_claimed
            .checked_add(amount)
            .ok_or(FinovaDeFiError::MathOverflow)?;
        
        self.last_reward_at = clock.unix_timestamp;
        
        Ok(())
    }
    
    /// Get effective reward multiplier
    pub fn get_reward_multiplier(&self, clock: &Clock) -> u32 {
        let mut multiplier = BASIS_POINTS_SCALE as u32;
        
        // Apply lock multiplier
        if self.status == FarmPositionStatus::Locked && clock.unix_timestamp < self.lock_expires_at {
            multiplier = (multiplier as u64)
                .checked_mul(self.lock_multiplier as u64)
                .unwrap_or(u64::MAX)
                .checked_div(BASIS_POINTS_SCALE as u64)
                .unwrap_or(0) as u32;
        }
        
        // Apply NFT boost
        if self.nft_boost > 0 {
            multiplier = (multiplier as u64)
                .checked_mul((BASIS_POINTS_SCALE + self.nft_boost) as u64)
                .unwrap_or(u64::MAX)
                .checked_div(BASIS_POINTS_SCALE as u64)
                .unwrap_or(0) as u32;
        }
        
        multiplier
    }
}

/// Position management helper functions
pub mod helpers {
    use super::*;
    
    /// Calculate optimal withdrawal amounts to minimize impermanent loss
    pub fn calculate_optimal_withdrawal(
        position: &LiquidityPosition,
        withdrawal_percentage: u16, // basis points
        pool_token_a: u64,
        pool_token_b: u64,
        pool_lp_supply: u64,
    ) -> Result<(u64, u64, u64)> {
        require!(
            withdrawal_percentage <= BASIS_POINTS_SCALE,
            FinovaDeFiError::InvalidPercentage
        );
        
        let lp_to_withdraw = (position.lp_token_amount as u128)
            .checked_mul(withdrawal_percentage as u128)
            .ok_or(FinovaDeFiError::MathOverflow)?
            .checked_div(BASIS_POINTS_SCALE as u128)
            .ok_or(FinovaDeFiError::DivisionByZero)? as u64;
        
        if pool_lp_supply == 0 {
            return Ok((lp_to_withdraw, 0, 0));
        }
        
        let token_a_out = (pool_token_a as u128)
            .checked_mul(lp_to_withdraw as u128)
            .ok_or(FinovaDeFiError::MathOverflow)?
            .checked_div(pool_lp_supply as u128)
            .ok_or(FinovaDeFiError::DivisionByZero)? as u64;
        
        let token_b_out = (pool_token_b as u128)
            .checked_mul(lp_to_withdraw as u128)
            .ok_or(FinovaDeFiError::MathOverflow)?
            .checked_div(pool_lp_supply as u128)
            .ok_or(FinovaDeFiError::DivisionByZero)? as u64;
        
        Ok((lp_to_withdraw, token_a_out, token_b_out))
    }
    
    /// Calculate position APY based on fees and rewards
    pub fn calculate_position_apy(
        position: &LiquidityPosition,
        current_timestamp: i64,
        token_a_price: u64,
        token_b_price: u64,
    ) -> Result<u32> {
        let time_elapsed = current_timestamp
            .checked_sub(position.created_at)
            .ok_or(FinovaDeFiError::MathOverflow)?;
        
        if time_elapsed <= 0 {
            return Ok(0);
        }
        
        // Calculate total fees earned in USD
        let fees_usd = (position.fees_earned_a as u128)
            .checked_mul(token_a_price as u128)
            .ok_or(FinovaDeFiError::MathOverflow)?
            .checked_add(
                (position.fees_earned_b as u128)
                    .checked_mul(token_b_price as u128)
                    .ok_or(FinovaDeFiError::MathOverflow)?
            )
            .ok_or(FinovaDeFiError::MathOverflow)?
            .checked_div(PRICE_DECIMALS as u128)
            .ok_or(FinovaDeFiError::DivisionByZero)? as u64;
        
        if position.initial_usd_value == 0 {
            return Ok(0);
        }
        
        // Calculate annualized return
        let seconds_per_year = 365 * 24 * 60 * 60;
        let apy = (fees_usd as u128)
            .checked_mul(BASIS_POINTS_SCALE as u128)
            .ok_or(FinovaDeFiError::MathOverflow)?
            .checked_mul(seconds_per_year as u128)
            .ok_or(FinovaDeFiError::MathOverflow)?
            .checked_div(position.initial_usd_value as u128)
            .ok_or(FinovaDeFiError::DivisionByZero)?
            .checked_div(time_elapsed as u128)
            .ok_or(FinovaDeFiError::DivisionByZero)? as u32;
        
        Ok(apy)
    }
}
