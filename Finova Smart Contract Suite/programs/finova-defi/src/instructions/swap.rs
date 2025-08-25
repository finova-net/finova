// programs/finova-defi/src/instructions/swap.rs

use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};
use crate::constants::*;
use crate::errors::FinovaDeFiError;
use crate::math::{curve::*, fees::*, oracle::*};
use crate::state::{Pool, LiquidityPosition};
use crate::utils::*;

/// Swap tokens in a liquidity pool with advanced slippage protection
/// and dynamic fee calculation based on pool utilization
#[derive(Accounts)]
#[instruction(
    amount_in: u64,
    minimum_amount_out: u64,
    swap_direction: SwapDirection,
)]
pub struct SwapTokens<'info> {
    #[account(mut)]
    pub swapper: Signer<'info>,

    /// Pool account containing swap parameters and reserves
    #[account(
        mut,
        seeds = [POOL_SEED, pool.token_a_mint.as_ref(), pool.token_b_mint.as_ref()],
        bump = pool.bump,
        constraint = pool.is_active @ FinovaDeFiError::PoolInactive,
        constraint = !pool.is_emergency_paused @ FinovaDeFiError::PoolEmergencyPaused,
    )]
    pub pool: Account<'info, Pool>,

    /// Swapper's source token account (token being sold)
    #[account(
        mut,
        constraint = swapper_source.owner == swapper.key(),
        constraint = match swap_direction {
            SwapDirection::AToB => swapper_source.mint == pool.token_a_mint,
            SwapDirection::BToA => swapper_source.mint == pool.token_b_mint,
        } @ FinovaDeFiError::InvalidTokenAccount,
    )]
    pub swapper_source: Account<'info, TokenAccount>,

    /// Swapper's destination token account (token being bought)
    #[account(
        mut,
        constraint = swapper_destination.owner == swapper.key(),
        constraint = match swap_direction {
            SwapDirection::AToB => swapper_destination.mint == pool.token_b_mint,
            SwapDirection::BToA => swapper_destination.mint == pool.token_a_mint,
        } @ FinovaDeFiError::InvalidTokenAccount,
    )]
    pub swapper_destination: Account<'info, TokenAccount>,

    /// Pool's token A reserve vault
    #[account(
        mut,
        seeds = [POOL_VAULT_SEED, pool.key().as_ref(), pool.token_a_mint.as_ref()],
        bump = pool.token_a_vault_bump,
        constraint = pool_vault_a.mint == pool.token_a_mint,
    )]
    pub pool_vault_a: Account<'info, TokenAccount>,

    /// Pool's token B reserve vault
    #[account(
        mut,
        seeds = [POOL_VAULT_SEED, pool.key().as_ref(), pool.token_b_mint.as_ref()],
        bump = pool.token_b_vault_bump,
        constraint = pool_vault_b.mint == pool.token_b_mint,
    )]
    pub pool_vault_b: Account<'info, TokenAccount>,

    /// Pool authority for token transfers
    #[account(
        seeds = [POOL_AUTHORITY_SEED, pool.key().as_ref()],
        bump = pool.authority_bump,
    )]
    /// CHECK: Pool authority PDA
    pub pool_authority: UncheckedAccount<'info>,

    /// Fee collector account for protocol fees
    #[account(
        mut,
        seeds = [FEE_COLLECTOR_SEED],
        bump,
    )]
    /// CHECK: Fee collector PDA
    pub fee_collector: UncheckedAccount<'info>,

    /// Protocol fee vault for token A
    #[account(
        mut,
        seeds = [PROTOCOL_FEE_VAULT_SEED, pool.token_a_mint.as_ref()],
        bump,
    )]
    pub protocol_fee_vault_a: Account<'info, TokenAccount>,

    /// Protocol fee vault for token B
    #[account(
        mut,
        seeds = [PROTOCOL_FEE_VAULT_SEED, pool.token_b_mint.as_ref()],
        bump,
    )]
    pub protocol_fee_vault_b: Account<'info, TokenAccount>,

    /// Price oracle for additional validation (optional)
    #[account(
        constraint = oracle.key() == pool.price_oracle.unwrap_or(Pubkey::default()) @ FinovaDeFiError::InvalidOracle,
    )]
    /// CHECK: Price oracle account
    pub oracle: Option<UncheckedAccount<'info>>,

    pub token_program: Program<'info, Token>,
    pub clock: Sysvar<'info, Clock>,
}

/// Direction of the token swap
#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub enum SwapDirection {
    /// Swap token A for token B
    AToB,
    /// Swap token B for token A
    BToA,
}

/// Advanced swap calculation result with detailed breakdown
#[derive(Debug, Clone)]
pub struct SwapCalculation {
    pub amount_in: u64,
    pub amount_out: u64,
    pub protocol_fee: u64,
    pub liquidity_provider_fee: u64,
    pub price_impact: u64, // Basis points (10000 = 100%)
    pub new_reserve_a: u64,
    pub new_reserve_b: u64,
    pub effective_price: u64, // Price per unit in basis points
}

impl<'info> SwapTokens<'info> {
    /// Execute token swap with comprehensive validation and fee distribution
    pub fn process_swap(
        &mut self,
        amount_in: u64,
        minimum_amount_out: u64,
        swap_direction: SwapDirection,
        max_price_impact_bps: u16, // Maximum allowed price impact in basis points
    ) -> Result<()> {
        // Validate swap parameters
        self.validate_swap_parameters(amount_in, minimum_amount_out, max_price_impact_bps)?;

        // Get current pool state
        let current_time = self.clock.unix_timestamp;
        let pool = &mut self.pool;

        // Update pool state if needed
        pool.update_time_weighted_average_price(current_time)?;

        // Calculate swap amounts with advanced AMM curve
        let swap_calc = self.calculate_swap_amounts(
            amount_in,
            swap_direction.clone(),
            max_price_impact_bps,
        )?;

        // Validate slippage protection
        require!(
            swap_calc.amount_out >= minimum_amount_out,
            FinovaDeFiError::SlippageExceeded
        );

        // Validate oracle price if available
        if let Some(_oracle) = &self.oracle {
            self.validate_oracle_price(&swap_calc, &swap_direction)?;
        }

        // Execute token transfers
        self.execute_swap_transfers(&swap_calc, &swap_direction)?;

        // Update pool reserves and statistics
        self.update_pool_state(&swap_calc)?;

        // Emit swap event
        emit!(SwapEvent {
            pool: pool.key(),
            swapper: self.swapper.key(),
            amount_in: swap_calc.amount_in,
            amount_out: swap_calc.amount_out,
            protocol_fee: swap_calc.protocol_fee,
            liquidity_provider_fee: swap_calc.liquidity_provider_fee,
            price_impact_bps: swap_calc.price_impact,
            swap_direction: swap_direction.clone(),
            timestamp: current_time,
        });

        msg!(
            "Swap completed: {} -> {}, Price Impact: {}bps, Effective Price: {}",
            swap_calc.amount_in,
            swap_calc.amount_out,
            swap_calc.price_impact,
            swap_calc.effective_price
        );

        Ok(())
    }

    /// Validate swap parameters and constraints
    fn validate_swap_parameters(
        &self,
        amount_in: u64,
        minimum_amount_out: u64,
        max_price_impact_bps: u16,
    ) -> Result<()> {
        // Validate input amount
        require!(amount_in > 0, FinovaDeFiError::InvalidAmount);
        require!(amount_in <= MAX_SWAP_AMOUNT, FinovaDeFiError::ExceedsMaxSwapAmount);
        require!(minimum_amount_out > 0, FinovaDeFiError::InvalidAmount);

        // Validate price impact limit
        require!(
            max_price_impact_bps <= MAX_PRICE_IMPACT_BPS,
            FinovaDeFiError::ExceedsMaxPriceImpact
        );

        // Check pool has sufficient liquidity
        let pool = &self.pool;
        require!(
            pool.token_a_reserve >= MIN_LIQUIDITY && pool.token_b_reserve >= MIN_LIQUIDITY,
            FinovaDeFiError::InsufficientLiquidity
        );

        // Validate pool utilization limits
        let utilization_a = calculate_utilization(amount_in, pool.token_a_reserve)?;
        let utilization_b = calculate_utilization(amount_in, pool.token_b_reserve)?;
        
        require!(
            utilization_a <= MAX_POOL_UTILIZATION_BPS && utilization_b <= MAX_POOL_UTILIZATION_BPS,
            FinovaDeFiError::ExceedsMaxUtilization
        );

        Ok(())
    }

    /// Calculate swap amounts using advanced constant product curve with fees
    fn calculate_swap_amounts(
        &self,
        amount_in: u64,
        swap_direction: SwapDirection,
        max_price_impact_bps: u16,
    ) -> Result<SwapCalculation> {
        let pool = &self.pool;
        let (reserve_in, reserve_out) = match swap_direction {
            SwapDirection::AToB => (pool.token_a_reserve, pool.token_b_reserve),
            SwapDirection::BToA => (pool.token_b_reserve, pool.token_a_reserve),
        };

        // Calculate dynamic fees based on pool utilization and volatility
        let base_fee_bps = pool.fee_rate_bps;
        let volatility_multiplier = calculate_volatility_multiplier(pool)?;
        let utilization_multiplier = calculate_utilization_multiplier(amount_in, reserve_in)?;
        
        let dynamic_fee_bps = (base_fee_bps as u128)
            .checked_mul(volatility_multiplier as u128)?
            .checked_mul(utilization_multiplier as u128)?
            .checked_div(10000_u128)?
            .checked_div(10000_u128)? as u16;

        let effective_fee_bps = std::cmp::min(dynamic_fee_bps, MAX_DYNAMIC_FEE_BPS);

        // Calculate protocol fee (percentage of total fee)
        let protocol_fee_share_bps = pool.protocol_fee_share_bps;
        let protocol_fee_bps = (effective_fee_bps as u128)
            .checked_mul(protocol_fee_share_bps as u128)?
            .checked_div(10000_u128)? as u16;

        let lp_fee_bps = effective_fee_bps.checked_sub(protocol_fee_bps)?;

        // Calculate fee amounts
        let total_fee_amount = calculate_fee_amount(amount_in, effective_fee_bps)?;
        let protocol_fee_amount = calculate_fee_amount(amount_in, protocol_fee_bps)?;
        let lp_fee_amount = total_fee_amount.checked_sub(protocol_fee_amount)?;

        // Calculate net input amount after fees
        let amount_in_after_fee = amount_in.checked_sub(total_fee_amount)?;

        // Apply constant product formula: x * y = k
        let amount_out = calculate_constant_product_output(
            amount_in_after_fee,
            reserve_in,
            reserve_out,
        )?;

        // Calculate price impact
        let price_before = calculate_price(reserve_out, reserve_in)?;
        let new_reserve_in = reserve_in.checked_add(amount_in)?;
        let new_reserve_out = reserve_out.checked_sub(amount_out)?;
        let price_after = calculate_price(new_reserve_out, new_reserve_in)?;
        
        let price_impact = calculate_price_impact(price_before, price_after)?;

        // Validate price impact doesn't exceed maximum
        require!(
            price_impact <= max_price_impact_bps as u64,
            FinovaDeFiError::PriceImpactTooHigh
        );

        // Calculate effective price
        let effective_price = if amount_in > 0 {
            (amount_out as u128)
                .checked_mul(10000_u128)?
                .checked_div(amount_in as u128)? as u64
        } else {
            0
        };

        let (new_reserve_a, new_reserve_b) = match swap_direction {
            SwapDirection::AToB => (new_reserve_in, new_reserve_out),
            SwapDirection::BToA => (new_reserve_out, new_reserve_in),
        };

        Ok(SwapCalculation {
            amount_in,
            amount_out,
            protocol_fee: protocol_fee_amount,
            liquidity_provider_fee: lp_fee_amount,
            price_impact,
            new_reserve_a,
            new_reserve_b,
            effective_price,
        })
    }

    /// Validate swap price against oracle if available
    fn validate_oracle_price(
        &self,
        swap_calc: &SwapCalculation,
        swap_direction: &SwapDirection,
    ) -> Result<()> {
        if let Some(oracle) = &self.oracle {
            let oracle_price = get_oracle_price(oracle)?;
            let swap_price = swap_calc.effective_price;
            
            let price_deviation = calculate_price_deviation(oracle_price, swap_price)?;
            
            require!(
                price_deviation <= MAX_ORACLE_DEVIATION_BPS,
                FinovaDeFiError::OraclePriceDeviation
            );
        }
        Ok(())
    }

    /// Execute all token transfers for the swap
    fn execute_swap_transfers(
        &mut self,
        swap_calc: &SwapCalculation,
        swap_direction: &SwapDirection,
    ) -> Result<()> {
        let pool_key = self.pool.key();
        let authority_seeds = &[
            POOL_AUTHORITY_SEED,
            pool_key.as_ref(),
            &[self.pool.authority_bump],
        ];
        let signer_seeds = &[&authority_seeds[..]];

        match swap_direction {
            SwapDirection::AToB => {
                // Transfer token A from swapper to pool
                token::transfer(
                    CpiContext::new(
                        self.token_program.to_account_info(),
                        Transfer {
                            from: self.swapper_source.to_account_info(),
                            to: self.pool_vault_a.to_account_info(),
                            authority: self.swapper.to_account_info(),
                        },
                    ),
                    swap_calc.amount_in,
                )?;

                // Transfer token B from pool to swapper
                token::transfer(
                    CpiContext::new_with_signer(
                        self.token_program.to_account_info(),
                        Transfer {
                            from: self.pool_vault_b.to_account_info(),
                            to: self.swapper_destination.to_account_info(),
                            authority: self.pool_authority.to_account_info(),
                        },
                        signer_seeds,
                    ),
                    swap_calc.amount_out,
                )?;

                // Transfer protocol fee to protocol fee vault
                if swap_calc.protocol_fee > 0 {
                    token::transfer(
                        CpiContext::new_with_signer(
                            self.token_program.to_account_info(),
                            Transfer {
                                from: self.pool_vault_a.to_account_info(),
                                to: self.protocol_fee_vault_a.to_account_info(),
                                authority: self.pool_authority.to_account_info(),
                            },
                            signer_seeds,
                        ),
                        swap_calc.protocol_fee,
                    )?;
                }
            }
            SwapDirection::BToA => {
                // Transfer token B from swapper to pool
                token::transfer(
                    CpiContext::new(
                        self.token_program.to_account_info(),
                        Transfer {
                            from: self.swapper_source.to_account_info(),
                            to: self.pool_vault_b.to_account_info(),
                            authority: self.swapper.to_account_info(),
                        },
                    ),
                    swap_calc.amount_in,
                )?;

                // Transfer token A from pool to swapper
                token::transfer(
                    CpiContext::new_with_signer(
                        self.token_program.to_account_info(),
                        Transfer {
                            from: self.pool_vault_a.to_account_info(),
                            to: self.swapper_destination.to_account_info(),
                            authority: self.pool_authority.to_account_info(),
                        },
                        signer_seeds,
                    ),
                    swap_calc.amount_out,
                )?;

                // Transfer protocol fee to protocol fee vault
                if swap_calc.protocol_fee > 0 {
                    token::transfer(
                        CpiContext::new_with_signer(
                            self.token_program.to_account_info(),
                            Transfer {
                                from: self.pool_vault_b.to_account_info(),
                                to: self.protocol_fee_vault_b.to_account_info(),
                                authority: self.pool_authority.to_account_info(),
                            },
                            signer_seeds,
                        ),
                        swap_calc.protocol_fee,
                    )?;
                }
            }
        }

        Ok(())
    }

    /// Update pool state with new reserves and statistics
    fn update_pool_state(&mut self, swap_calc: &SwapCalculation) -> Result<()> {
        let pool = &mut self.pool;
        let current_time = self.clock.unix_timestamp;

        // Update reserves
        pool.token_a_reserve = swap_calc.new_reserve_a;
        pool.token_b_reserve = swap_calc.new_reserve_b;

        // Update volume statistics
        pool.total_volume_token_a = pool.total_volume_token_a
            .checked_add(swap_calc.amount_in)?;
        pool.total_volume_token_b = pool.total_volume_token_b
            .checked_add(swap_calc.amount_out)?;

        // Update swap count
        pool.total_swaps = pool.total_swaps.checked_add(1)?;

        // Update accumulated fees
        pool.accumulated_protocol_fees_a = pool.accumulated_protocol_fees_a
            .checked_add(swap_calc.protocol_fee)?;
        pool.accumulated_lp_fees_a = pool.accumulated_lp_fees_a
            .checked_add(swap_calc.liquidity_provider_fee)?;

        // Update last swap timestamp
        pool.last_swap_timestamp = current_time;

        // Update price and volatility metrics
        pool.update_price_metrics(swap_calc.effective_price, current_time)?;

        // Calculate and update liquidity score
        pool.liquidity_score = calculate_liquidity_score(
            pool.token_a_reserve,
            pool.token_b_reserve,
            pool.total_swaps,
        )?;

        Ok(())
    }
}

/// Swap event emitted when a swap is completed
#[event]
pub struct SwapEvent {
    pub pool: Pubkey,
    pub swapper: Pubkey,
    pub amount_in: u64,
    pub amount_out: u64,
    pub protocol_fee: u64,
    pub liquidity_provider_fee: u64,
    pub price_impact_bps: u64,
    pub swap_direction: SwapDirection,
    pub timestamp: i64,
}

/// Advanced mathematical functions for swap calculations
impl SwapCalculation {
    /// Calculate optimal swap amount for minimizing price impact
    pub fn calculate_optimal_swap_amount(
        target_amount: u64,
        reserve_in: u64,
        reserve_out: u64,
        fee_bps: u16,
    ) -> Result<u64> {
        // Use binary search to find optimal amount
        let mut low = 0u64;
        let mut high = target_amount;
        let mut optimal_amount = 0u64;

        while low <= high && high > low {
            let mid = (low + high) / 2;
            let price_impact = calculate_price_impact_for_amount(mid, reserve_in, reserve_out, fee_bps)?;
            
            if price_impact <= OPTIMAL_PRICE_IMPACT_BPS {
                optimal_amount = mid;
                low = mid + 1;
            } else {
                high = mid - 1;
            }
        }

        Ok(optimal_amount)
    }

    /// Calculate multi-hop swap amounts through multiple pools
    pub fn calculate_multi_hop_swap(
        pools: &[Pool],
        amount_in: u64,
        path: &[SwapDirection],
    ) -> Result<Vec<SwapCalculation>> {
        require!(pools.len() == path.len(), FinovaDeFiError::InvalidSwapPath);
        
        let mut calculations = Vec::new();
        let mut current_amount = amount_in;

        for (pool, direction) in pools.iter().zip(path.iter()) {
            let swap_calc = Self::calculate_single_hop(pool, current_amount, direction.clone())?;
            current_amount = swap_calc.amount_out;
            calculations.push(swap_calc);
        }

        Ok(calculations)
    }

    /// Calculate single hop swap for multi-hop routing
    fn calculate_single_hop(
        pool: &Pool,
        amount_in: u64,
        direction: SwapDirection,
    ) -> Result<SwapCalculation> {
        // Simplified calculation for multi-hop - would use full calculation in practice
        let (reserve_in, reserve_out) = match direction {
            SwapDirection::AToB => (pool.token_a_reserve, pool.token_b_reserve),
            SwapDirection::BToA => (pool.token_b_reserve, pool.token_a_reserve),
        };

        let fee_amount = calculate_fee_amount(amount_in, pool.fee_rate_bps)?;
        let amount_in_after_fee = amount_in.checked_sub(fee_amount)?;
        let amount_out = calculate_constant_product_output(amount_in_after_fee, reserve_in, reserve_out)?;

        Ok(SwapCalculation {
            amount_in,
            amount_out,
            protocol_fee: fee_amount / 2, // Simplified
            liquidity_provider_fee: fee_amount / 2,
            price_impact: calculate_price_impact_for_amount(amount_in, reserve_in, reserve_out, pool.fee_rate_bps)?,
            new_reserve_a: match direction {
                SwapDirection::AToB => reserve_in + amount_in,
                SwapDirection::BToA => reserve_out - amount_out,
            },
            new_reserve_b: match direction {
                SwapDirection::AToB => reserve_out - amount_out,
                SwapDirection::BToA => reserve_in + amount_in,
            },
            effective_price: if amount_in > 0 { (amount_out * 10000) / amount_in } else { 0 },
        })
    }
}

/// Helper function to calculate price impact for a given swap amount
fn calculate_price_impact_for_amount(
    amount_in: u64,
    reserve_in: u64,
    reserve_out: u64,
    fee_bps: u16,
) -> Result<u64> {
    let fee_amount = calculate_fee_amount(amount_in, fee_bps)?;
    let amount_in_after_fee = amount_in.checked_sub(fee_amount)?;
    
    let price_before = (reserve_out as u128 * 10000) / reserve_in as u128;
    let new_reserve_in = reserve_in.checked_add(amount_in_after_fee)?;
    let amount_out = calculate_constant_product_output(amount_in_after_fee, reserve_in, reserve_out)?;
    let new_reserve_out = reserve_out.checked_sub(amount_out)?;
    let price_after = (new_reserve_out as u128 * 10000) / new_reserve_in as u128;
    
    let price_impact = if price_before > price_after {
        ((price_before - price_after) * 10000) / price_before
    } else {
        ((price_after - price_before) * 10000) / price_before
    };

    Ok(price_impact as u64)
}

/// Entry point for swap instruction
pub fn swap_tokens(
    ctx: Context<SwapTokens>,
    amount_in: u64,
    minimum_amount_out: u64,
    swap_direction: SwapDirection,
    max_price_impact_bps: u16,
) -> Result<()> {
    ctx.accounts.process_swap(
        amount_in,
        minimum_amount_out,
        swap_direction,
        max_price_impact_bps,
    )
}
