// programs/finova-defi/src/utils.rs

use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount, Mint};
use std::convert::TryInto;
use crate::errors::DefiError;
use crate::constants::*;

/// Utility functions for DeFi operations in Finova Network
pub struct DefiUtils;

impl DefiUtils {
    /// Calculate swap output amount using constant product formula (x * y = k)
    /// Includes slippage protection and fee calculations
    pub fn calculate_swap_output(
        input_amount: u64,
        input_reserve: u64,
        output_reserve: u64,
        fee_rate: u16, // in basis points (e.g., 30 = 0.3%)
    ) -> Result<u64> {
        require!(input_amount > 0, DefiError::InvalidAmount);
        require!(input_reserve > 0 && output_reserve > 0, DefiError::InsufficientLiquidity);
        require!(fee_rate <= MAX_FEE_RATE, DefiError::InvalidFeeRate);

        // Calculate fee amount
        let fee_amount = (input_amount as u128)
            .checked_mul(fee_rate as u128)
            .ok_or(DefiError::MathOverflow)?
            .checked_div(BASIS_POINTS)
            .ok_or(DefiError::MathOverflow)? as u64;

        let input_amount_after_fee = input_amount
            .checked_sub(fee_amount)
            .ok_or(DefiError::MathOverflow)?;

        // Apply constant product formula: (x + dx) * (y - dy) = x * y
        // dy = (y * dx) / (x + dx)
        let numerator = (output_reserve as u128)
            .checked_mul(input_amount_after_fee as u128)
            .ok_or(DefiError::MathOverflow)?;

        let denominator = (input_reserve as u128)
            .checked_add(input_amount_after_fee as u128)
            .ok_or(DefiError::MathOverflow)?;

        let output_amount = numerator
            .checked_div(denominator)
            .ok_or(DefiError::MathOverflow)? as u64;

        require!(output_amount > 0, DefiError::InsufficientOutputAmount);
        require!(output_amount < output_reserve, DefiError::InsufficientLiquidity);

        Ok(output_amount)
    }

    /// Calculate liquidity tokens to mint when adding liquidity
    /// Uses geometric mean for fair pricing
    pub fn calculate_liquidity_tokens(
        token_a_amount: u64,
        token_b_amount: u64,
        reserve_a: u64,
        reserve_b: u64,
        total_supply: u64,
    ) -> Result<u64> {
        require!(token_a_amount > 0 && token_b_amount > 0, DefiError::InvalidAmount);

        if total_supply == 0 {
            // First liquidity provider - use geometric mean
            let liquidity = Self::sqrt(
                (token_a_amount as u128)
                    .checked_mul(token_b_amount as u128)
                    .ok_or(DefiError::MathOverflow)?
            )? as u64;
            
            require!(liquidity > MINIMUM_LIQUIDITY, DefiError::InsufficientLiquidity);
            Ok(liquidity.checked_sub(MINIMUM_LIQUIDITY).ok_or(DefiError::MathOverflow)?)
        } else {
            // Subsequent providers - maintain price ratio
            require!(reserve_a > 0 && reserve_b > 0, DefiError::InsufficientLiquidity);
            
            let liquidity_a = (token_a_amount as u128)
                .checked_mul(total_supply as u128)
                .ok_or(DefiError::MathOverflow)?
                .checked_div(reserve_a as u128)
                .ok_or(DefiError::MathOverflow)? as u64;

            let liquidity_b = (token_b_amount as u128)
                .checked_mul(total_supply as u128)
                .ok_or(DefiError::MathOverflow)?
                .checked_div(reserve_b as u128)
                .ok_or(DefiError::MathOverflow)? as u64;

            Ok(std::cmp::min(liquidity_a, liquidity_b))
        }
    }

    /// Calculate tokens to return when removing liquidity
    pub fn calculate_remove_liquidity(
        liquidity_amount: u64,
        total_supply: u64,
        reserve_a: u64,
        reserve_b: u64,
    ) -> Result<(u64, u64)> {
        require!(liquidity_amount > 0, DefiError::InvalidAmount);
        require!(total_supply > 0, DefiError::InsufficientLiquidity);
        require!(liquidity_amount <= total_supply, DefiError::InsufficientLiquidity);

        let token_a_amount = (reserve_a as u128)
            .checked_mul(liquidity_amount as u128)
            .ok_or(DefiError::MathOverflow)?
            .checked_div(total_supply as u128)
            .ok_or(DefiError::MathOverflow)? as u64;

        let token_b_amount = (reserve_b as u128)
            .checked_mul(liquidity_amount as u128)
            .ok_or(DefiError::MathOverflow)?
            .checked_div(total_supply as u128)
            .ok_or(DefiError::MathOverflow)? as u64;

        Ok((token_a_amount, token_b_amount))
    }

    /// Calculate yield farming rewards based on staking duration and multipliers
    pub fn calculate_farming_rewards(
        staked_amount: u64,
        duration_seconds: u64,
        base_apy: u16, // in basis points
        xp_multiplier: u16, // from user's XP level
        rp_multiplier: u16, // from user's RP tier
        nft_boost: u16, // from special NFT cards
    ) -> Result<u64> {
        require!(staked_amount > 0, DefiError::InvalidAmount);
        require!(base_apy <= MAX_APY, DefiError::InvalidAPY);

        // Calculate base rewards (annual rate converted to per-second)
        let annual_rate = (staked_amount as u128)
            .checked_mul(base_apy as u128)
            .ok_or(DefiError::MathOverflow)?
            .checked_div(BASIS_POINTS)
            .ok_or(DefiError::MathOverflow)?;

        let per_second_rate = annual_rate
            .checked_div(SECONDS_PER_YEAR)
            .ok_or(DefiError::MathOverflow)?;

        let base_rewards = per_second_rate
            .checked_mul(duration_seconds as u128)
            .ok_or(DefiError::MathOverflow)? as u64;

        // Apply multipliers
        let total_multiplier = BASIS_POINTS as u128
            + xp_multiplier as u128
            + rp_multiplier as u128
            + nft_boost as u128;

        let final_rewards = (base_rewards as u128)
            .checked_mul(total_multiplier)
            .ok_or(DefiError::MathOverflow)?
            .checked_div(BASIS_POINTS)
            .ok_or(DefiError::MathOverflow)? as u64;

        Ok(final_rewards)
    }

    /// Calculate flash loan fee
    pub fn calculate_flash_loan_fee(
        loan_amount: u64,
        fee_rate: u16, // in basis points
    ) -> Result<u64> {
        require!(loan_amount > 0, DefiError::InvalidAmount);
        require!(fee_rate <= MAX_FLASH_LOAN_FEE, DefiError::InvalidFeeRate);

        let fee = (loan_amount as u128)
            .checked_mul(fee_rate as u128)
            .ok_or(DefiError::MathOverflow)?
            .checked_div(BASIS_POINTS)
            .ok_or(DefiError::MathOverflow)? as u64;

        Ok(fee)
    }

    /// Calculate impermanent loss for liquidity providers
    pub fn calculate_impermanent_loss(
        initial_price_ratio: u64, // token_a / token_b * PRECISION
        current_price_ratio: u64,
    ) -> Result<u64> {
        require!(initial_price_ratio > 0 && current_price_ratio > 0, DefiError::InvalidAmount);

        let ratio_change = if current_price_ratio > initial_price_ratio {
            (current_price_ratio as u128)
                .checked_div(initial_price_ratio as u128)
                .ok_or(DefiError::MathOverflow)?
        } else {
            (initial_price_ratio as u128)
                .checked_div(current_price_ratio as u128)
                .ok_or(DefiError::MathOverflow)?
        };

        // Simplified impermanent loss calculation
        // IL = 2 * sqrt(price_ratio) / (1 + price_ratio) - 1
        let sqrt_ratio = Self::sqrt(ratio_change)?;
        let numerator = 2u128.checked_mul(sqrt_ratio).ok_or(DefiError::MathOverflow)?;
        let denominator = 1u128.checked_add(ratio_change).ok_or(DefiError::MathOverflow)?;
        
        let loss_factor = numerator
            .checked_div(denominator)
            .ok_or(DefiError::MathOverflow)?;

        // Convert to percentage (basis points)
        let impermanent_loss = if loss_factor > 1u128 {
            0u64 // No loss
        } else {
            let loss_percentage = 1u128
                .checked_sub(loss_factor)
                .ok_or(DefiError::MathOverflow)?
                .checked_mul(BASIS_POINTS)
                .ok_or(DefiError::MathOverflow)? as u64;
            loss_percentage
        };

        Ok(impermanent_loss)
    }

    /// Validate slippage tolerance
    pub fn validate_slippage(
        expected_amount: u64,
        actual_amount: u64,
        max_slippage_bps: u16,
    ) -> Result<()> {
        require!(max_slippage_bps <= MAX_SLIPPAGE_BPS, DefiError::InvalidSlippage);

        if actual_amount >= expected_amount {
            return Ok(()); // Better than expected
        }

        let slippage = (expected_amount.checked_sub(actual_amount).ok_or(DefiError::MathOverflow)? as u128)
            .checked_mul(BASIS_POINTS)
            .ok_or(DefiError::MathOverflow)?
            .checked_div(expected_amount as u128)
            .ok_or(DefiError::MathOverflow)? as u16;

        require!(slippage <= max_slippage_bps, DefiError::SlippageExceeded);
        Ok(())
    }

    /// Check if oracle price is fresh and within acceptable bounds
    pub fn validate_oracle_price(
        price: u64,
        timestamp: i64,
        max_age_seconds: i64,
        min_price: u64,
        max_price: u64,
    ) -> Result<()> {
        let current_time = Clock::get()?.unix_timestamp;
        let price_age = current_time.checked_sub(timestamp).ok_or(DefiError::MathOverflow)?;
        
        require!(price_age <= max_age_seconds, DefiError::StalePrice);
        require!(price >= min_price && price <= max_price, DefiError::InvalidPrice);
        require!(price > 0, DefiError::InvalidPrice);

        Ok(())
    }

    /// Calculate time-weighted average price (TWAP)
    pub fn calculate_twap(
        prices: &[u64],
        timestamps: &[i64],
        period_seconds: i64,
    ) -> Result<u64> {
        require!(prices.len() == timestamps.len(), DefiError::InvalidInput);
        require!(prices.len() >= 2, DefiError::InsufficientData);

        let mut weighted_sum = 0u128;
        let mut total_weight = 0u128;

        for i in 1..prices.len() {
            let time_diff = timestamps[i]
                .checked_sub(timestamps[i - 1])
                .ok_or(DefiError::MathOverflow)?;
            
            if time_diff > 0 && time_diff <= period_seconds {
                weighted_sum = weighted_sum
                    .checked_add((prices[i - 1] as u128).checked_mul(time_diff as u128).ok_or(DefiError::MathOverflow)?)
                    .ok_or(DefiError::MathOverflow)?;
                total_weight = total_weight
                    .checked_add(time_diff as u128)
                    .ok_or(DefiError::MathOverflow)?;
            }
        }

        require!(total_weight > 0, DefiError::InsufficientData);

        let twap = weighted_sum
            .checked_div(total_weight)
            .ok_or(DefiError::MathOverflow)? as u64;

        Ok(twap)
    }

    /// Calculate optimal swap route through multiple pools
    pub fn calculate_optimal_route(
        input_amount: u64,
        pools: &[(u64, u64, u16)], // (reserve_in, reserve_out, fee_rate)
    ) -> Result<(Vec<usize>, u64)> {
        require!(!pools.is_empty(), DefiError::InvalidInput);

        let mut best_output = 0u64;
        let mut best_route = Vec::new();

        // Single hop
        for (i, &(reserve_in, reserve_out, fee_rate)) in pools.iter().enumerate() {
            if let Ok(output) = Self::calculate_swap_output(input_amount, reserve_in, reserve_out, fee_rate) {
                if output > best_output {
                    best_output = output;
                    best_route = vec![i];
                }
            }
        }

        // Multi-hop (up to 3 hops for gas efficiency)
        for i in 0..pools.len() {
            let first_output = Self::calculate_swap_output(
                input_amount,
                pools[i].0,
                pools[i].1,
                pools[i].2,
            )?;

            for j in 0..pools.len() {
                if i == j { continue; }
                
                if let Ok(second_output) = Self::calculate_swap_output(
                    first_output,
                    pools[j].0,
                    pools[j].1,
                    pools[j].2,
                ) {
                    if second_output > best_output {
                        best_output = second_output;
                        best_route = vec![i, j];
                    }
                }
            }
        }

        require!(!best_route.is_empty(), DefiError::NoValidRoute);
        Ok((best_route, best_output))
    }

    /// Integer square root implementation
    pub fn sqrt(y: u128) -> Result<u128> {
        if y == 0 {
            return Ok(0);
        }

        let mut z = y;
        let mut x = y / 2 + 1;

        while x < z {
            z = x;
            x = (y / x + x) / 2;
        }

        Ok(z)
    }

    /// Check if account is a valid token account
    pub fn validate_token_account(
        account: &AccountInfo,
        mint: &Pubkey,
        owner: &Pubkey,
    ) -> Result<()> {
        let token_account: Account<TokenAccount> = Account::try_from(account)?;
        
        require!(token_account.mint == *mint, DefiError::InvalidTokenAccount);
        require!(token_account.owner == *owner, DefiError::InvalidTokenAccount);
        
        Ok(())
    }

    /// Calculate pool share percentage
    pub fn calculate_pool_share(
        user_liquidity: u64,
        total_liquidity: u64,
    ) -> Result<u16> {
        require!(total_liquidity > 0, DefiError::InvalidAmount);
        
        let share = (user_liquidity as u128)
            .checked_mul(BASIS_POINTS)
            .ok_or(DefiError::MathOverflow)?
            .checked_div(total_liquidity as u128)
            .ok_or(DefiError::MathOverflow)? as u16;

        Ok(share)
    }

    /// Generate deterministic pool address
    pub fn derive_pool_address(
        program_id: &Pubkey,
        mint_a: &Pubkey,
        mint_b: &Pubkey,
    ) -> Result<(Pubkey, u8)> {
        let (mint_a, mint_b) = if mint_a < mint_b {
            (mint_a, mint_b)
        } else {
            (mint_b, mint_a)
        };

        let seeds = &[
            POOL_SEED,
            mint_a.as_ref(),
            mint_b.as_ref(),
        ];

        Pubkey::find_program_address(seeds, program_id)
            .map_err(|_| DefiError::InvalidPoolAddress.into())
    }

    /// Calculate dynamic fee based on pool volatility
    pub fn calculate_dynamic_fee(
        base_fee: u16,
        volatility_factor: u16,
        max_fee: u16,
    ) -> u16 {
        let dynamic_fee = base_fee
            .saturating_add(volatility_factor);
        
        std::cmp::min(dynamic_fee, max_fee)
    }

    /// Validate minimum output amount considering price impact
    pub fn validate_minimum_output(
        output_amount: u64,
        minimum_output: u64,
        max_price_impact_bps: u16,
        input_amount: u64,
        input_price: u64,
        output_price: u64,
    ) -> Result<()> {
        require!(output_amount >= minimum_output, DefiError::InsufficientOutputAmount);

        // Calculate price impact
        let expected_output = (input_amount as u128)
            .checked_mul(input_price as u128)
            .ok_or(DefiError::MathOverflow)?
            .checked_div(output_price as u128)
            .ok_or(DefiError::MathOverflow)? as u64;

        let price_impact = if expected_output > output_amount {
            let impact = expected_output
                .checked_sub(output_amount)
                .ok_or(DefiError::MathOverflow)?;
            
            (impact as u128)
                .checked_mul(BASIS_POINTS)
                .ok_or(DefiError::MathOverflow)?
                .checked_div(expected_output as u128)
                .ok_or(DefiError::MathOverflow)? as u16
        } else {
            0u16
        };

        require!(price_impact <= max_price_impact_bps, DefiError::PriceImpactTooHigh);
        Ok(())
    }

    /// Calculate governance voting power based on staked tokens and lock duration
    pub fn calculate_voting_power(
        staked_amount: u64,
        lock_duration_days: u16,
        max_lock_days: u16,
        base_multiplier: u16,
    ) -> Result<u64> {
        require!(lock_duration_days <= max_lock_days, DefiError::InvalidLockDuration);
        
        let time_multiplier = (lock_duration_days as u128)
            .checked_mul(base_multiplier as u128)
            .ok_or(DefiError::MathOverflow)?
            .checked_div(max_lock_days as u128)
            .ok_or(DefiError::MathOverflow)?;
        
        let total_multiplier = BASIS_POINTS as u128
            .checked_add(time_multiplier)
            .ok_or(DefiError::MathOverflow)?;
        
        let voting_power = (staked_amount as u128)
            .checked_mul(total_multiplier)
            .ok_or(DefiError::MathOverflow)?
            .checked_div(BASIS_POINTS)
            .ok_or(DefiError::MathOverflow)? as u64;
        
        Ok(voting_power)
    }

    /// Validate that reserves are balanced within acceptable range
    pub fn validate_reserve_balance(
        reserve_a: u64,
        reserve_b: u64,
        price_a: u64,
        price_b: u64,
        max_imbalance_bps: u16,
    ) -> Result<()> {
        let value_a = (reserve_a as u128)
            .checked_mul(price_a as u128)
            .ok_or(DefiError::MathOverflow)?;
        
        let value_b = (reserve_b as u128)
            .checked_mul(price_b as u128)
            .ok_or(DefiError::MathOverflow)?;
        
        let total_value = value_a
            .checked_add(value_b)
            .ok_or(DefiError::MathOverflow)?;
        
        if total_value == 0 {
            return Ok(());
        }
        
        let balance_a = (value_a as u128)
            .checked_mul(BASIS_POINTS)
            .ok_or(DefiError::MathOverflow)?
            .checked_div(total_value)
            .ok_or(DefiError::MathOverflow)? as u16;
        
        let expected_balance = BASIS_POINTS / 2; // 50%
        let imbalance = if balance_a > expected_balance {
            balance_a - expected_balance
        } else {
            expected_balance - balance_a
        };
        
        require!(imbalance <= max_imbalance_bps, DefiError::ReserveImbalance);
        Ok(())
    }
}
