// programs/finova-defi/src/instructions/mod.rs

use anchor_lang::prelude::*;

pub mod create_pool;
pub mod add_liquidity;
pub mod remove_liquidity;
pub mod swap;
pub mod yield_farm;
pub mod flash_loan;

pub use create_pool::*;
pub use add_liquidity::*;
pub use remove_liquidity::*;
pub use swap::*;
pub use yield_farm::*;
pub use flash_loan::*;

use crate::state::*;
use crate::errors::*;
use crate::constants::*;
use crate::utils::*;
use crate::math::*;

/// Main instruction enum for Finova DeFi operations
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum DeFiInstruction {
    /// Initialize a new liquidity pool
    CreatePool {
        token_a_mint: Pubkey,
        token_b_mint: Pubkey,
        fee_rate: u16,
        initial_price: u64,
    },
    
    /// Add liquidity to an existing pool
    AddLiquidity {
        pool: Pubkey,
        amount_a: u64,
        amount_b: u64,
        min_lp_tokens: u64,
        deadline: i64,
    },
    
    /// Remove liquidity from a pool
    RemoveLiquidity {
        pool: Pubkey,
        lp_tokens: u64,
        min_amount_a: u64,
        min_amount_b: u64,
        deadline: i64,
    },
    
    /// Execute a token swap
    Swap {
        pool: Pubkey,
        token_in: Pubkey,
        token_out: Pubkey,
        amount_in: u64,
        min_amount_out: u64,
        deadline: i64,
    },
    
    /// Stake LP tokens in yield farm
    StakeInFarm {
        farm: Pubkey,
        amount: u64,
    },
    
    /// Unstake LP tokens from yield farm
    UnstakeFromFarm {
        farm: Pubkey,
        amount: u64,
    },
    
    /// Claim yield farming rewards
    ClaimFarmRewards {
        farm: Pubkey,
    },
    
    /// Execute flash loan
    FlashLoan {
        pool: Pubkey,
        token: Pubkey,
        amount: u64,
        callback_data: Vec<u8>,
    },
    
    /// Repay flash loan
    RepayFlashLoan {
        pool: Pubkey,
        token: Pubkey,
        amount: u64,
        fee: u64,
    },
}

/// Common validation functions for DeFi instructions
pub struct DeFiValidation;

impl DeFiValidation {
    /// Validate deadline hasn't passed
    pub fn validate_deadline(deadline: i64) -> Result<()> {
        let current_time = Clock::get()?.unix_timestamp;
        require!(deadline >= current_time, DeFiError::DeadlineExceeded);
        Ok(())
    }
    
    /// Validate slippage protection
    pub fn validate_slippage(
        expected_amount: u64,
        actual_amount: u64,
        max_slippage_basis_points: u16,
    ) -> Result<()> {
        let min_amount = expected_amount
            .checked_mul(10000_u64.checked_sub(max_slippage_basis_points as u64).unwrap())
            .unwrap()
            .checked_div(10000)
            .unwrap();
        
        require!(actual_amount >= min_amount, DeFiError::SlippageExceeded);
        Ok(())
    }
    
    /// Validate pool reserves are not zero
    pub fn validate_pool_liquidity(reserve_a: u64, reserve_b: u64) -> Result<()> {
        require!(reserve_a > 0 && reserve_b > 0, DeFiError::InsufficientLiquidity);
        Ok(())
    }
    
    /// Validate user has sufficient balance
    pub fn validate_sufficient_balance(balance: u64, required: u64) -> Result<()> {
        require!(balance >= required, DeFiError::InsufficientBalance);
        Ok(())
    }
    
    /// Validate pool fee rate is within bounds
    pub fn validate_fee_rate(fee_rate: u16) -> Result<()> {
        require!(
            fee_rate <= MAX_FEE_RATE_BASIS_POINTS,
            DeFiError::InvalidFeeRate
        );
        Ok(())
    }
    
    /// Validate token amounts are non-zero
    pub fn validate_non_zero_amount(amount: u64) -> Result<()> {
        require!(amount > 0, DeFiError::InvalidAmount);
        Ok(())
    }
    
    /// Validate LP token supply constraints
    pub fn validate_lp_supply(total_supply: u64, user_tokens: u64) -> Result<()> {
        require!(user_tokens <= total_supply, DeFiError::InvalidLPTokenAmount);
        require!(total_supply > 0, DeFiError::ZeroLPSupply);
        Ok(())
    }
    
    /// Validate farm staking requirements
    pub fn validate_farm_stake(
        farm_active: bool,
        stake_amount: u64,
        user_lp_balance: u64,
    ) -> Result<()> {
        require!(farm_active, DeFiError::FarmNotActive);
        require!(stake_amount > 0, DeFiError::InvalidAmount);
        require!(user_lp_balance >= stake_amount, DeFiError::InsufficientBalance);
        Ok(())
    }
    
    /// Validate flash loan parameters
    pub fn validate_flash_loan(
        pool_reserve: u64,
        loan_amount: u64,
        max_loan_ratio: u16,
    ) -> Result<()> {
        let max_loan = pool_reserve
            .checked_mul(max_loan_ratio as u64)
            .unwrap()
            .checked_div(10000)
            .unwrap();
        
        require!(loan_amount <= max_loan, DeFiError::FlashLoanTooLarge);
        require!(loan_amount > 0, DeFiError::InvalidAmount);
        Ok(())
    }
    
    /// Validate oracle price data
    pub fn validate_oracle_price(
        price: u64,
        max_age_seconds: i64,
        last_update: i64,
    ) -> Result<()> {
        let current_time = Clock::get()?.unix_timestamp;
        require!(
            current_time - last_update <= max_age_seconds,
            DeFiError::StalePrice
        );
        require!(price > 0, DeFiError::InvalidPrice);
        Ok(())
    }
}

/// Helper functions for DeFi calculations
pub struct DeFiCalculations;

impl DeFiCalculations {
    /// Calculate LP tokens to mint for initial liquidity
    pub fn calculate_initial_lp_tokens(amount_a: u64, amount_b: u64) -> Result<u64> {
        // Use geometric mean for initial LP token calculation
        let lp_tokens = (amount_a as u128)
            .checked_mul(amount_b as u128)
            .unwrap();
        
        // Take square root (simplified implementation)
        let mut result = (lp_tokens as f64).sqrt() as u64;
        
        // Ensure minimum liquidity
        if result < MINIMUM_LIQUIDITY {
            result = MINIMUM_LIQUIDITY;
        }
        
        Ok(result)
    }
    
    /// Calculate LP tokens to mint for subsequent liquidity additions
    pub fn calculate_lp_tokens_to_mint(
        amount_a: u64,
        amount_b: u64,
        reserve_a: u64,
        reserve_b: u64,
        total_supply: u64,
    ) -> Result<u64> {
        let lp_from_a = (amount_a as u128)
            .checked_mul(total_supply as u128)
            .unwrap()
            .checked_div(reserve_a as u128)
            .unwrap();
        
        let lp_from_b = (amount_b as u128)
            .checked_mul(total_supply as u128)
            .unwrap()
            .checked_div(reserve_b as u128)
            .unwrap();
        
        // Take minimum to prevent arbitrage
        Ok(std::cmp::min(lp_from_a, lp_from_b) as u64)
    }
    
    /// Calculate amounts to receive when removing liquidity
    pub fn calculate_remove_liquidity_amounts(
        lp_tokens: u64,
        total_supply: u64,
        reserve_a: u64,
        reserve_b: u64,
    ) -> Result<(u64, u64)> {
        let amount_a = (lp_tokens as u128)
            .checked_mul(reserve_a as u128)
            .unwrap()
            .checked_div(total_supply as u128)
            .unwrap() as u64;
        
        let amount_b = (lp_tokens as u128)
            .checked_mul(reserve_b as u128)
            .unwrap()
            .checked_div(total_supply as u128)
            .unwrap() as u64;
        
        Ok((amount_a, amount_b))
    }
    
    /// Calculate swap output amount using constant product formula
    pub fn calculate_swap_amount_out(
        amount_in: u64,
        reserve_in: u64,
        reserve_out: u64,
        fee_rate: u16,
    ) -> Result<u64> {
        require!(amount_in > 0, DeFiError::InvalidAmount);
        require!(reserve_in > 0 && reserve_out > 0, DeFiError::InsufficientLiquidity);
        
        // Apply fee: amount_in_after_fee = amount_in * (10000 - fee_rate) / 10000
        let amount_in_after_fee = (amount_in as u128)
            .checked_mul((10000_u128).checked_sub(fee_rate as u128).unwrap())
            .unwrap()
            .checked_div(10000)
            .unwrap();
        
        // Constant product formula: (x + dx) * (y - dy) = x * y
        // Solving for dy: dy = (y * dx) / (x + dx)
        let numerator = amount_in_after_fee
            .checked_mul(reserve_out as u128)
            .unwrap();
        
        let denominator = (reserve_in as u128)
            .checked_add(amount_in_after_fee)
            .unwrap();
        
        let amount_out = numerator.checked_div(denominator).unwrap() as u64;
        
        require!(amount_out < reserve_out, DeFiError::InsufficientLiquidity);
        Ok(amount_out)
    }
    
    /// Calculate required input amount for desired output
    pub fn calculate_swap_amount_in(
        amount_out: u64,
        reserve_in: u64,
        reserve_out: u64,
        fee_rate: u16,
    ) -> Result<u64> {
        require!(amount_out > 0, DeFiError::InvalidAmount);
        require!(amount_out < reserve_out, DeFiError::InsufficientLiquidity);
        require!(reserve_in > 0 && reserve_out > 0, DeFiError::InsufficientLiquidity);
        
        // Reverse calculation: amount_in = (x * dy) / ((y - dy) * (10000 - fee_rate) / 10000)
        let numerator = (reserve_in as u128)
            .checked_mul(amount_out as u128)
            .unwrap()
            .checked_mul(10000)
            .unwrap();
        
        let denominator = ((reserve_out as u128)
            .checked_sub(amount_out as u128)
            .unwrap())
            .checked_mul((10000_u128).checked_sub(fee_rate as u128).unwrap())
            .unwrap();
        
        let amount_in = numerator.checked_div(denominator).unwrap() as u64;
        
        // Add 1 to account for rounding
        Ok(amount_in + 1)
    }
    
    /// Calculate yield farming rewards
    pub fn calculate_farm_rewards(
        user_stake: u64,
        total_stake: u64,
        reward_rate_per_second: u64,
        duration_seconds: u64,
    ) -> Result<u64> {
        if total_stake == 0 || user_stake == 0 {
            return Ok(0);
        }
        
        let total_rewards = reward_rate_per_second
            .checked_mul(duration_seconds)
            .unwrap();
        
        let user_rewards = (total_rewards as u128)
            .checked_mul(user_stake as u128)
            .unwrap()
            .checked_div(total_stake as u128)
            .unwrap() as u64;
        
        Ok(user_rewards)
    }
    
    /// Calculate flash loan fee
    pub fn calculate_flash_loan_fee(
        loan_amount: u64,
        fee_rate_basis_points: u16,
    ) -> Result<u64> {
        let fee = (loan_amount as u128)
            .checked_mul(fee_rate_basis_points as u128)
            .unwrap()
            .checked_div(10000)
            .unwrap() as u64;
        
        // Ensure minimum fee
        Ok(std::cmp::max(fee, 1))
    }
    
    /// Calculate price impact for large swaps
    pub fn calculate_price_impact(
        amount_in: u64,
        reserve_in: u64,
        reserve_out: u64,
    ) -> Result<u16> {
        let price_before = (reserve_out as u128)
            .checked_mul(10000)
            .unwrap()
            .checked_div(reserve_in as u128)
            .unwrap();
        
        let new_reserve_in = (reserve_in as u128)
            .checked_add(amount_in as u128)
            .unwrap();
        
        let price_after = (reserve_out as u128)
            .checked_mul(10000)
            .unwrap()
            .checked_div(new_reserve_in)
            .unwrap();
        
        let price_impact = if price_before > price_after {
            ((price_before - price_after) * 10000 / price_before) as u16
        } else {
            0
        };
        
        Ok(price_impact)
    }
}

/// Event emission helpers
pub struct DeFiEvents;

impl DeFiEvents {
    /// Emit pool creation event
    pub fn emit_pool_created(
        pool: Pubkey,
        token_a: Pubkey,
        token_b: Pubkey,
        fee_rate: u16,
        creator: Pubkey,
    ) {
        emit!(PoolCreated {
            pool,
            token_a,
            token_b,
            fee_rate,
            creator,
            timestamp: Clock::get().unwrap().unix_timestamp,
        });
    }
    
    /// Emit liquidity addition event
    pub fn emit_liquidity_added(
        pool: Pubkey,
        provider: Pubkey,
        amount_a: u64,
        amount_b: u64,
        lp_tokens: u64,
    ) {
        emit!(LiquidityAdded {
            pool,
            provider,
            amount_a,
            amount_b,
            lp_tokens,
            timestamp: Clock::get().unwrap().unix_timestamp,
        });
    }
    
    /// Emit swap event
    pub fn emit_swap(
        pool: Pubkey,
        user: Pubkey,
        token_in: Pubkey,
        token_out: Pubkey,
        amount_in: u64,
        amount_out: u64,
        fee: u64,
    ) {
        emit!(SwapExecuted {
            pool,
            user,
            token_in,
            token_out,
            amount_in,
            amount_out,
            fee,
            timestamp: Clock::get().unwrap().unix_timestamp,
        });
    }
}

/// Pool management utilities
pub struct PoolManager;

impl PoolManager {
    /// Update pool reserves after swap
    pub fn update_reserves_after_swap(
        pool: &mut Pool,
        token_in: Pubkey,
        amount_in: u64,
        amount_out: u64,
    ) -> Result<()> {
        if token_in == pool.token_a_mint {
            pool.reserve_a = pool.reserve_a.checked_add(amount_in).unwrap();
            pool.reserve_b = pool.reserve_b.checked_sub(amount_out).unwrap();
        } else {
            pool.reserve_b = pool.reserve_b.checked_add(amount_in).unwrap();
            pool.reserve_a = pool.reserve_a.checked_sub(amount_out).unwrap();
        }
        
        pool.last_update_timestamp = Clock::get()?.unix_timestamp;
        Ok(())
    }
    
    /// Update pool reserves after liquidity change
    pub fn update_reserves_after_liquidity(
        pool: &mut Pool,
        amount_a_change: i64,
        amount_b_change: i64,
    ) -> Result<()> {
        if amount_a_change >= 0 {
            pool.reserve_a = pool.reserve_a.checked_add(amount_a_change as u64).unwrap();
        } else {
            pool.reserve_a = pool.reserve_a.checked_sub((-amount_a_change) as u64).unwrap();
        }
        
        if amount_b_change >= 0 {
            pool.reserve_b = pool.reserve_b.checked_add(amount_b_change as u64).unwrap();
        } else {
            pool.reserve_b = pool.reserve_b.checked_sub((-amount_b_change) as u64).unwrap();
        }
        
        pool.last_update_timestamp = Clock::get()?.unix_timestamp;
        Ok(())
    }
    
    /// Calculate current pool price
    pub fn get_pool_price(pool: &Pool, base_token: Pubkey) -> Result<u64> {
        require!(
            pool.reserve_a > 0 && pool.reserve_b > 0,
            DeFiError::InsufficientLiquidity
        );
        
        if base_token == pool.token_a_mint {
            Ok((pool.reserve_b as u128)
                .checked_mul(PRICE_PRECISION as u128)
                .unwrap()
                .checked_div(pool.reserve_a as u128)
                .unwrap() as u64)
        } else {
            Ok((pool.reserve_a as u128)
                .checked_mul(PRICE_PRECISION as u128)
                .unwrap()
                .checked_div(pool.reserve_b as u128)
                .unwrap() as u64)
        }
    }
}

/// Security utilities for DeFi operations
pub struct DeFiSecurity;

impl DeFiSecurity {
    /// Check for flash loan attack patterns
    pub fn check_flash_loan_attack(
        pool_balance_before: u64,
        pool_balance_after: u64,
        loan_amount: u64,
    ) -> Result<()> {
        // Ensure pool balance is properly restored
        require!(
            pool_balance_after >= pool_balance_before,
            DeFiError::FlashLoanNotRepaid
        );
        
        // Check for suspicious balance increase (potential exploit)
        let balance_increase = pool_balance_after - pool_balance_before;
        let expected_fee = loan_amount / 1000; // 0.1% fee
        
        require!(
            balance_increase >= expected_fee,
            DeFiError::InsufficientFlashLoanFee
        );
        
        Ok(())
    }
    
    /// Validate transaction sequence for MEV protection
    pub fn validate_transaction_sequence(
        last_tx_slot: u64,
        current_slot: u64,
        min_slot_distance: u64,
    ) -> Result<()> {
        require!(
            current_slot >= last_tx_slot + min_slot_distance,
            DeFiError::TransactionTooFrequent
        );
        Ok(())
    }
    
    /// Check for sandwich attack patterns
    pub fn check_sandwich_attack(
        price_before: u64,
        price_after: u64,
        max_price_impact: u16,
    ) -> Result<()> {
        let price_change = if price_after > price_before {
            ((price_after - price_before) * 10000) / price_before
        } else {
            ((price_before - price_after) * 10000) / price_before
        };
        
        require!(
            price_change <= max_price_impact as u64,
            DeFiError::ExcessivePriceImpact
        );
        
        Ok(())
    }
}

/// Events for DeFi operations
#[event]
pub struct PoolCreated {
    pub pool: Pubkey,
    pub token_a: Pubkey,
    pub token_b: Pubkey,
    pub fee_rate: u16,
    pub creator: Pubkey,
    pub timestamp: i64,
}

#[event]
pub struct LiquidityAdded {
    pub pool: Pubkey,
    pub provider: Pubkey,
    pub amount_a: u64,
    pub amount_b: u64,
    pub lp_tokens: u64,
    pub timestamp: i64,
}

#[event]
pub struct LiquidityRemoved {
    pub pool: Pubkey,
    pub provider: Pubkey,
    pub amount_a: u64,
    pub amount_b: u64,
    pub lp_tokens: u64,
    pub timestamp: i64,
}

#[event]
pub struct SwapExecuted {
    pub pool: Pubkey,
    pub user: Pubkey,
    pub token_in: Pubkey,
    pub token_out: Pubkey,
    pub amount_in: u64,
    pub amount_out: u64,
    pub fee: u64,
    pub timestamp: i64,
}

#[event]
pub struct FlashLoanExecuted {
    pub pool: Pubkey,
    pub borrower: Pubkey,
    pub token: Pubkey,
    pub amount: u64,
    pub fee: u64,
    pub timestamp: i64,
}

#[event]
pub struct FarmStakeChanged {
    pub farm: Pubkey,
    pub user: Pubkey,
    pub amount: u64,
    pub is_stake: bool,
    pub timestamp: i64,
}

#[event]
pub struct RewardsClaimed {
    pub farm: Pubkey,
    pub user: Pubkey,
    pub reward_amount: u64,
    pub timestamp: i64,
}
