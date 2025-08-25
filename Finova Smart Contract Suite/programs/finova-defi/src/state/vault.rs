// programs/finova-defi/src/state/vault.rs

use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, TokenAccount};
use crate::constants::*;
use crate::errors::DefiError;

/// Vault configuration and state for yield farming
#[account]
#[derive(Debug)]
pub struct Vault {
    /// Vault authority (PDA)
    pub authority: Pubkey,
    
    /// Vault bump seed
    pub bump: u8,
    
    /// Vault name identifier
    pub name: [u8; 32],
    
    /// Base token mint (e.g., USDC, SOL)
    pub base_mint: Pubkey,
    
    /// Reward token mint (e.g., FIN)
    pub reward_mint: Pubkey,
    
    /// Vault token account holding base tokens
    pub base_vault: Pubkey,
    
    /// Vault token account holding reward tokens
    pub reward_vault: Pubkey,
    
    /// Total base tokens deposited in vault
    pub total_deposited: u64,
    
    /// Total shares issued by vault
    pub total_shares: u64,
    
    /// Current reward rate per second (scaled by 1e9)
    pub reward_rate: u64,
    
    /// Reward duration in seconds
    pub reward_duration: u64,
    
    /// Timestamp when current reward period ends
    pub reward_finish_at: i64,
    
    /// Last time rewards were updated
    pub last_update_time: i64,
    
    /// Reward per share stored (scaled by 1e18)
    pub reward_per_share_stored: u128,
    
    /// Minimum deposit amount
    pub min_deposit: u64,
    
    /// Maximum deposit amount per user
    pub max_deposit_per_user: u64,
    
    /// Vault fee rate (basis points, max 10000)
    pub fee_rate: u16,
    
    /// Performance fee rate (basis points, max 10000)
    pub performance_fee_rate: u16,
    
    /// Vault status flags
    pub status: VaultStatus,
    
    /// Strategy type
    pub strategy_type: StrategyType,
    
    /// Strategy parameters
    pub strategy_params: StrategyParams,
    
    /// Risk parameters
    pub risk_params: RiskParams,
    
    /// Vault creation timestamp
    pub created_at: i64,
    
    /// Last harvest timestamp
    pub last_harvest_at: i64,
    
    /// Total fees collected
    pub total_fees_collected: u64,
    
    /// Total performance fees collected
    pub total_performance_fees: u64,
    
    /// Reserved space for future upgrades
    pub reserved: [u64; 16],
}

impl Vault {
    /// Size of the Vault account
    pub const SIZE: usize = 8 + // discriminator
        32 + // authority
        1 + // bump
        32 + // name
        32 + // base_mint
        32 + // reward_mint
        32 + // base_vault
        32 + // reward_vault
        8 + // total_deposited
        8 + // total_shares
        8 + // reward_rate
        8 + // reward_duration
        8 + // reward_finish_at
        8 + // last_update_time
        16 + // reward_per_share_stored
        8 + // min_deposit
        8 + // max_deposit_per_user
        2 + // fee_rate
        2 + // performance_fee_rate
        1 + // status
        1 + // strategy_type
        StrategyParams::SIZE + // strategy_params
        RiskParams::SIZE + // risk_params
        8 + // created_at
        8 + // last_harvest_at
        8 + // total_fees_collected
        8 + // total_performance_fees
        128; // reserved

    /// Initialize a new vault
    pub fn initialize(
        &mut self,
        authority: Pubkey,
        bump: u8,
        name: [u8; 32],
        base_mint: Pubkey,
        reward_mint: Pubkey,
        base_vault: Pubkey,
        reward_vault: Pubkey,
        reward_rate: u64,
        reward_duration: u64,
        min_deposit: u64,
        max_deposit_per_user: u64,
        fee_rate: u16,
        performance_fee_rate: u16,
        strategy_type: StrategyType,
        strategy_params: StrategyParams,
        risk_params: RiskParams,
    ) -> Result<()> {
        require!(fee_rate <= MAX_FEE_RATE, DefiError::InvalidFeeRate);
        require!(performance_fee_rate <= MAX_PERFORMANCE_FEE_RATE, DefiError::InvalidPerformanceFeeRate);
        require!(min_deposit > 0, DefiError::InvalidMinDeposit);
        require!(max_deposit_per_user >= min_deposit, DefiError::InvalidMaxDeposit);

        let clock = Clock::get()?;
        
        self.authority = authority;
        self.bump = bump;
        self.name = name;
        self.base_mint = base_mint;
        self.reward_mint = reward_mint;
        self.base_vault = base_vault;
        self.reward_vault = reward_vault;
        self.total_deposited = 0;
        self.total_shares = 0;
        self.reward_rate = reward_rate;
        self.reward_duration = reward_duration;
        self.reward_finish_at = clock.unix_timestamp + reward_duration as i64;
        self.last_update_time = clock.unix_timestamp;
        self.reward_per_share_stored = 0;
        self.min_deposit = min_deposit;
        self.max_deposit_per_user = max_deposit_per_user;
        self.fee_rate = fee_rate;
        self.performance_fee_rate = performance_fee_rate;
        self.status = VaultStatus::Active;
        self.strategy_type = strategy_type;
        self.strategy_params = strategy_params;
        self.risk_params = risk_params;
        self.created_at = clock.unix_timestamp;
        self.last_harvest_at = clock.unix_timestamp;
        self.total_fees_collected = 0;
        self.total_performance_fees = 0;
        self.reserved = [0; 16];

        Ok(())
    }

    /// Update reward variables
    pub fn update_reward(&mut self) -> Result<()> {
        let clock = Clock::get()?;
        self.reward_per_share_stored = self.reward_per_share()?;
        self.last_update_time = std::cmp::min(clock.unix_timestamp, self.reward_finish_at);
        Ok(())
    }

    /// Calculate current reward per share
    pub fn reward_per_share(&self) -> Result<u128> {
        if self.total_shares == 0 {
            return Ok(self.reward_per_share_stored);
        }

        let clock = Clock::get()?;
        let last_time_applicable = std::cmp::min(clock.unix_timestamp, self.reward_finish_at);
        let time_diff = (last_time_applicable - self.last_update_time) as u128;
        
        let additional_reward_per_share = time_diff
            .checked_mul(self.reward_rate as u128)
            .ok_or(DefiError::MathOverflow)?
            .checked_mul(REWARD_PRECISION)
            .ok_or(DefiError::MathOverflow)?
            .checked_div(self.total_shares as u128)
            .ok_or(DefiError::MathOverflow)?;

        self.reward_per_share_stored
            .checked_add(additional_reward_per_share)
            .ok_or_else(|| error!(DefiError::MathOverflow))
    }

    /// Calculate share price based on vault performance
    pub fn share_price(&self) -> Result<u64> {
        if self.total_shares == 0 {
            return Ok(INITIAL_SHARE_PRICE);
        }

        self.total_deposited
            .checked_mul(SHARE_PRECISION)
            .ok_or(DefiError::MathOverflow)?
            .checked_div(self.total_shares)
            .ok_or_else(|| error!(DefiError::MathOverflow))
    }

    /// Calculate shares to mint for deposit amount
    pub fn calculate_shares_to_mint(&self, deposit_amount: u64) -> Result<u64> {
        if self.total_shares == 0 {
            return Ok(deposit_amount);
        }

        let share_price = self.share_price()?;
        deposit_amount
            .checked_mul(SHARE_PRECISION)
            .ok_or(DefiError::MathOverflow)?
            .checked_div(share_price)
            .ok_or_else(|| error!(DefiError::MathOverflow))
    }

    /// Calculate tokens to return for shares
    pub fn calculate_tokens_from_shares(&self, shares: u64) -> Result<u64> {
        if self.total_shares == 0 {
            return Ok(0);
        }

        let share_price = self.share_price()?;
        shares
            .checked_mul(share_price)
            .ok_or(DefiError::MathOverflow)?
            .checked_div(SHARE_PRECISION)
            .ok_or_else(|| error!(DefiError::MathOverflow))
    }

    /// Calculate deposit fee
    pub fn calculate_deposit_fee(&self, amount: u64) -> Result<u64> {
        amount
            .checked_mul(self.fee_rate as u64)
            .ok_or(DefiError::MathOverflow)?
            .checked_div(BASIS_POINTS)
            .ok_or_else(|| error!(DefiError::MathOverflow))
    }

    /// Calculate performance fee
    pub fn calculate_performance_fee(&self, profit: u64) -> Result<u64> {
        profit
            .checked_mul(self.performance_fee_rate as u64)
            .ok_or(DefiError::MathOverflow)?
            .checked_div(BASIS_POINTS)
            .ok_or_else(|| error!(DefiError::MathOverflow))
    }

    /// Deposit tokens into vault
    pub fn deposit(&mut self, amount: u64, user_deposit: &mut UserDeposit) -> Result<u64> {
        require!(self.status == VaultStatus::Active, DefiError::VaultNotActive);
        require!(amount >= self.min_deposit, DefiError::DepositTooSmall);
        
        let new_user_total = user_deposit.amount.checked_add(amount).ok_or(DefiError::MathOverflow)?;
        require!(new_user_total <= self.max_deposit_per_user, DefiError::DepositTooLarge);

        self.update_reward()?;

        let deposit_fee = self.calculate_deposit_fee(amount)?;
        let net_amount = amount.checked_sub(deposit_fee).ok_or(DefiError::MathOverflow)?;
        
        let shares_to_mint = self.calculate_shares_to_mint(net_amount)?;

        // Update vault state
        self.total_deposited = self.total_deposited
            .checked_add(net_amount)
            .ok_or(DefiError::MathOverflow)?;
        self.total_shares = self.total_shares
            .checked_add(shares_to_mint)
            .ok_or(DefiError::MathOverflow)?;
        self.total_fees_collected = self.total_fees_collected
            .checked_add(deposit_fee)
            .ok_or(DefiError::MathOverflow)?;

        // Update user deposit
        user_deposit.deposit(amount, shares_to_mint, self.reward_per_share()?)?;

        Ok(shares_to_mint)
    }

    /// Withdraw tokens from vault
    pub fn withdraw(&mut self, shares: u64, user_deposit: &mut UserDeposit) -> Result<u64> {
        require!(self.status != VaultStatus::Frozen, DefiError::VaultFrozen);
        require!(shares > 0, DefiError::InvalidAmount);
        require!(user_deposit.shares >= shares, DefiError::InsufficientShares);

        self.update_reward()?;

        let tokens_to_return = self.calculate_tokens_from_shares(shares)?;
        
        // Update vault state
        self.total_deposited = self.total_deposited
            .checked_sub(tokens_to_return)
            .ok_or(DefiError::MathOverflow)?;
        self.total_shares = self.total_shares
            .checked_sub(shares)
            .ok_or(DefiError::MathOverflow)?;

        // Update user deposit
        user_deposit.withdraw(shares, self.reward_per_share()?)?;

        Ok(tokens_to_return)
    }

    /// Set vault status
    pub fn set_status(&mut self, status: VaultStatus) -> Result<()> {
        self.status = status;
        Ok(())
    }

    /// Update reward rate
    pub fn update_reward_rate(&mut self, new_rate: u64, duration: u64) -> Result<()> {
        self.update_reward()?;
        
        let clock = Clock::get()?;
        self.reward_rate = new_rate;
        self.reward_duration = duration;
        self.reward_finish_at = clock.unix_timestamp + duration as i64;
        
        Ok(())
    }

    /// Harvest vault profits and compound
    pub fn harvest(&mut self, profit: u64) -> Result<u64> {
        let clock = Clock::get()?;
        self.last_harvest_at = clock.unix_timestamp;

        if profit == 0 {
            return Ok(0);
        }

        let performance_fee = self.calculate_performance_fee(profit)?;
        let net_profit = profit.checked_sub(performance_fee).ok_or(DefiError::MathOverflow)?;

        // Compound the net profit back into the vault
        self.total_deposited = self.total_deposited
            .checked_add(net_profit)
            .ok_or(DefiError::MathOverflow)?;
        
        self.total_performance_fees = self.total_performance_fees
            .checked_add(performance_fee)
            .ok_or(DefiError::MathOverflow)?;

        Ok(performance_fee)
    }

    /// Check if vault is healthy based on risk parameters
    pub fn is_healthy(&self) -> bool {
        // Implementation depends on strategy type and risk parameters
        match self.strategy_type {
            StrategyType::Conservative => true, // Conservative strategy is always healthy
            StrategyType::Moderate => {
                // Check moderate risk thresholds
                self.total_deposited > 0 && 
                self.risk_params.max_drawdown <= 1000 // 10% max drawdown
            },
            StrategyType::Aggressive => {
                // Check aggressive risk thresholds
                self.total_deposited > 0 && 
                self.risk_params.max_drawdown <= 2000 // 20% max drawdown
            },
        }
    }
}

/// User's deposit information in a vault
#[account]
#[derive(Debug)]
pub struct UserDeposit {
    /// User wallet address
    pub user: Pubkey,
    
    /// Vault this deposit belongs to
    pub vault: Pubkey,
    
    /// User's total deposited amount (base tokens)
    pub amount: u64,
    
    /// User's share balance
    pub shares: u64,
    
    /// User's reward per share paid
    pub reward_per_share_paid: u128,
    
    /// Accumulated rewards
    pub rewards: u64,
    
    /// Deposit timestamp
    pub deposited_at: i64,
    
    /// Last interaction timestamp
    pub last_interaction_at: i64,
    
    /// Lock period end timestamp (0 if no lock)
    pub lock_end_at: i64,
    
    /// Boost multiplier (basis points)
    pub boost_multiplier: u16,
    
    /// Reserved space for future upgrades
    pub reserved: [u64; 8],
}

impl UserDeposit {
    /// Size of the UserDeposit account
    pub const SIZE: usize = 8 + // discriminator
        32 + // user
        32 + // vault
        8 + // amount
        8 + // shares
        16 + // reward_per_share_paid
        8 + // rewards
        8 + // deposited_at
        8 + // last_interaction_at
        8 + // lock_end_at
        2 + // boost_multiplier
        64; // reserved

    /// Initialize user deposit
    pub fn initialize(&mut self, user: Pubkey, vault: Pubkey) -> Result<()> {
        let clock = Clock::get()?;
        
        self.user = user;
        self.vault = vault;
        self.amount = 0;
        self.shares = 0;
        self.reward_per_share_paid = 0;
        self.rewards = 0;
        self.deposited_at = clock.unix_timestamp;
        self.last_interaction_at = clock.unix_timestamp;
        self.lock_end_at = 0;
        self.boost_multiplier = BASIS_POINTS as u16; // 100% = no boost
        self.reserved = [0; 8];

        Ok(())
    }

    /// Update user rewards
    pub fn update_rewards(&mut self, reward_per_share: u128) -> Result<()> {
        let earned = self.calculate_earned_rewards(reward_per_share)?;
        self.rewards = self.rewards.checked_add(earned).ok_or(DefiError::MathOverflow)?;
        self.reward_per_share_paid = reward_per_share;
        Ok(())
    }

    /// Calculate earned rewards
    pub fn calculate_earned_rewards(&self, reward_per_share: u128) -> Result<u64> {
        let reward_diff = reward_per_share
            .checked_sub(self.reward_per_share_paid)
            .ok_or(DefiError::MathOverflow)?;
        
        let base_rewards = (self.shares as u128)
            .checked_mul(reward_diff)
            .ok_or(DefiError::MathOverflow)?
            .checked_div(REWARD_PRECISION)
            .ok_or(DefiError::MathOverflow)? as u64;

        // Apply boost multiplier
        base_rewards
            .checked_mul(self.boost_multiplier as u64)
            .ok_or(DefiError::MathOverflow)?
            .checked_div(BASIS_POINTS)
            .ok_or_else(|| error!(DefiError::MathOverflow))
    }

    /// Deposit tokens
    pub fn deposit(&mut self, amount: u64, shares: u64, reward_per_share: u128) -> Result<()> {
        self.update_rewards(reward_per_share)?;
        
        let clock = Clock::get()?;
        self.amount = self.amount.checked_add(amount).ok_or(DefiError::MathOverflow)?;
        self.shares = self.shares.checked_add(shares).ok_or(DefiError::MathOverflow)?;
        self.last_interaction_at = clock.unix_timestamp;

        Ok(())
    }

    /// Withdraw tokens
    pub fn withdraw(&mut self, shares: u64, reward_per_share: u128) -> Result<()> {
        self.update_rewards(reward_per_share)?;
        
        let clock = Clock::get()?;
        require!(clock.unix_timestamp >= self.lock_end_at, DefiError::DepositLocked);
        
        self.shares = self.shares.checked_sub(shares).ok_or(DefiError::MathOverflow)?;
        self.last_interaction_at = clock.unix_timestamp;

        Ok(())
    }

    /// Claim rewards
    pub fn claim_rewards(&mut self, reward_per_share: u128) -> Result<u64> {
        self.update_rewards(reward_per_share)?;
        
        let rewards_to_claim = self.rewards;
        self.rewards = 0;
        
        let clock = Clock::get()?;
        self.last_interaction_at = clock.unix_timestamp;

        Ok(rewards_to_claim)
    }

    /// Set boost multiplier
    pub fn set_boost_multiplier(&mut self, multiplier: u16) -> Result<()> {
        require!(multiplier >= BASIS_POINTS as u16, DefiError::InvalidBoostMultiplier);
        require!(multiplier <= MAX_BOOST_MULTIPLIER, DefiError::InvalidBoostMultiplier);
        
        self.boost_multiplier = multiplier;
        Ok(())
    }

    /// Set lock period
    pub fn set_lock_period(&mut self, lock_duration: i64) -> Result<()> {
        let clock = Clock::get()?;
        self.lock_end_at = clock.unix_timestamp + lock_duration;
        Ok(())
    }
}

/// Vault status enumeration
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq)]
pub enum VaultStatus {
    Active,
    Paused,
    Frozen,
    Deprecated,
}

/// Strategy type enumeration
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq)]
pub enum StrategyType {
    Conservative,
    Moderate,
    Aggressive,
}

/// Strategy parameters
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug)]
pub struct StrategyParams {
    /// Rebalancing threshold (basis points)
    pub rebalance_threshold: u16,
    
    /// Maximum slippage tolerance (basis points)
    pub max_slippage: u16,
    
    /// Minimum liquidity threshold
    pub min_liquidity: u64,
    
    /// Strategy-specific parameter 1
    pub param1: u64,
    
    /// Strategy-specific parameter 2
    pub param2: u64,
    
    /// Strategy-specific parameter 3
    pub param3: u64,
}

impl StrategyParams {
    pub const SIZE: usize = 2 + 2 + 8 + 8 + 8 + 8;
}

/// Risk management parameters
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug)]
pub struct RiskParams {
    /// Maximum drawdown (basis points)
    pub max_drawdown: u16,
    
    /// Stop loss threshold (basis points)
    pub stop_loss: u16,
    
    /// Maximum leverage multiplier
    pub max_leverage: u16,
    
    /// Risk assessment score
    pub risk_score: u8,
    
    /// Reserved parameters
    pub reserved: [u32; 4],
}

impl RiskParams {
    pub const SIZE: usize = 2 + 2 + 2 + 1 + 16;
}
