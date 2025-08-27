// programs/finova-defi/src/instructions/remove_liquidity.rs

use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};

use crate::constants::*;
use crate::errors::*;
use crate::math::{curve::CurveCalculator, fees::FeeCalculator};
use crate::state::{
    pool::{Pool, PoolStatus},
    liquidity_position::LiquidityPosition,
};
use crate::utils::*;

/// Remove liquidity from a trading pool
/// 
/// This instruction allows liquidity providers to remove their liquidity
/// from a pool and receive their proportional share of both tokens plus
/// accumulated fees and yield farming rewards.
#[derive(Accounts)]
pub struct RemoveLiquidity<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    
    /// Pool account containing trading pair information
    #[account(
        mut,
        seeds = [POOL_SEED, pool.token_a_mint.as_ref(), pool.token_b_mint.as_ref()],
        bump = pool.bump,
        constraint = pool.status == PoolStatus::Active @ FinovaDefiError::PoolNotActive,
        constraint = !pool.is_paused @ FinovaDefiError::PoolPaused
    )]
    pub pool: Account<'info, Pool>,
    
    /// User's liquidity position
    #[account(
        mut,
        seeds = [
            LIQUIDITY_POSITION_SEED,
            pool.key().as_ref(),
            user.key().as_ref()
        ],
        bump = liquidity_position.bump,
        constraint = liquidity_position.pool == pool.key() @ FinovaDefiError::InvalidPool,
        constraint = liquidity_position.owner == user.key() @ FinovaDefiError::Unauthorized,
        constraint = liquidity_position.liquidity_tokens > 0 @ FinovaDefiError::InsufficientLiquidity
    )]
    pub liquidity_position: Account<'info, LiquidityPosition>,
    
    /// Pool's token A vault
    #[account(
        mut,
        token::mint = pool.token_a_mint,
        token::authority = pool,
        constraint = pool_token_a_vault.key() == pool.token_a_vault @ FinovaDefiError::InvalidVault
    )]
    pub pool_token_a_vault: Account<'info, TokenAccount>,
    
    /// Pool's token B vault
    #[account(
        mut,
        token::mint = pool.token_b_mint,
        token::authority = pool,
        constraint = pool_token_b_vault.key() == pool.token_b_vault @ FinovaDefiError::InvalidVault
    )]
    pub pool_token_b_vault: Account<'info, TokenAccount>,
    
    /// User's token A account to receive withdrawn tokens
    #[account(
        mut,
        token::mint = pool.token_a_mint,
        token::authority = user
    )]
    pub user_token_a_account: Account<'info, TokenAccount>,
    
    /// User's token B account to receive withdrawn tokens
    #[account(
        mut,
        token::mint = pool.token_b_mint,
        token::authority = user
    )]
    pub user_token_b_account: Account<'info, TokenAccount>,
    
    /// Pool's LP token mint
    #[account(
        mut,
        constraint = lp_token_mint.key() == pool.lp_token_mint @ FinovaDefiError::InvalidLPTokenMint
    )]
    pub lp_token_mint: Account<'info, anchor_spl::token::Mint>,
    
    /// User's LP token account to burn tokens from
    #[account(
        mut,
        token::mint = lp_token_mint,
        token::authority = user,
        constraint = user_lp_token_account.amount >= liquidity_tokens_to_burn @ FinovaDefiError::InsufficientLPTokens
    )]
    pub user_lp_token_account: Account<'info, TokenAccount>,
    
    /// Pool's fee collector account for protocol fees
    #[account(
        mut,
        token::mint = pool.token_a_mint,
        constraint = fee_collector_a.key() == pool.fee_collector_a @ FinovaDefiError::InvalidFeeCollector
    )]
    pub fee_collector_a: Account<'info, TokenAccount>,
    
    #[account(
        mut,
        token::mint = pool.token_b_mint,
        constraint = fee_collector_b.key() == pool.fee_collector_b @ FinovaDefiError::InvalidFeeCollector
    )]
    pub fee_collector_b: Account<'info, TokenAccount>,
    
    pub token_program: Program<'info, Token>,
    
    pub system_program: Program<'info, System>,
}

impl<'info> RemoveLiquidity<'info> {
    /// Calculate amounts to withdraw based on LP tokens being burned
    pub fn calculate_withdrawal_amounts(&self, liquidity_tokens_to_burn: u64) -> Result<(u64, u64, u64, u64)> {
        let pool = &self.pool;
        let liquidity_position = &self.liquidity_position;
        
        // Validate liquidity tokens to burn
        require!(
            liquidity_tokens_to_burn <= liquidity_position.liquidity_tokens,
            FinovaDefiError::InsufficientLiquidity
        );
        
        require!(
            liquidity_tokens_to_burn > 0,
            FinovaDefiError::ZeroAmount
        );
        
        let total_lp_supply = self.lp_token_mint.supply;
        require!(
            total_lp_supply > 0,
            FinovaDefiError::NoLiquidity
        );
        
        // Calculate proportional share of pool reserves
        let token_a_reserve = self.pool_token_a_vault.amount;
        let token_b_reserve = self.pool_token_b_vault.amount;
        
        let token_a_amount = calculate_proportional_amount(
            liquidity_tokens_to_burn,
            total_lp_supply,
            token_a_reserve,
        )?;
        
        let token_b_amount = calculate_proportional_amount(
            liquidity_tokens_to_burn,
            total_lp_supply,
            token_b_reserve,
        )?;
        
        // Calculate accumulated fees for this position
        let (fee_a_amount, fee_b_amount) = self.calculate_accumulated_fees(liquidity_tokens_to_burn)?;
        
        // Apply minimum withdrawal protection
        require!(
            token_a_amount >= MIN_WITHDRAWAL_AMOUNT,
            FinovaDefiError::WithdrawalTooSmall
        );
        
        require!(
            token_b_amount >= MIN_WITHDRAWAL_AMOUNT,
            FinovaDefiError::WithdrawalTooSmall
        );
        
        Ok((token_a_amount, token_b_amount, fee_a_amount, fee_b_amount))
    }
    
    /// Calculate accumulated fees for the liquidity position
    pub fn calculate_accumulated_fees(&self, liquidity_tokens_to_burn: u64) -> Result<(u64, u64)> {
        let pool = &self.pool;
        let position = &self.liquidity_position;
        
        // Calculate fees earned since last update
        let total_fee_growth_a = pool.cumulative_fee_growth_a
            .checked_sub(position.fee_growth_checkpoint_a)
            .unwrap_or(0);
            
        let total_fee_growth_b = pool.cumulative_fee_growth_b
            .checked_sub(position.fee_growth_checkpoint_b)
            .unwrap_or(0);
        
        // Calculate fees proportional to liquidity being removed
        let fee_share = calculate_fee_share(
            liquidity_tokens_to_burn,
            position.liquidity_tokens,
        )?;
        
        let fee_a_amount = calculate_fee_amount(
            total_fee_growth_a,
            position.liquidity_tokens,
            fee_share,
        )?;
        
        let fee_b_amount = calculate_fee_amount(
            total_fee_growth_b,
            position.liquidity_tokens,
            fee_share,
        )?;
        
        Ok((fee_a_amount, fee_b_amount))
    }
    
    /// Update position state after liquidity removal
    pub fn update_position_state(&mut self, liquidity_tokens_burned: u64) -> Result<()> {
        let position = &mut self.liquidity_position;
        let pool = &self.pool;
        
        // Update liquidity tokens
        position.liquidity_tokens = position.liquidity_tokens
            .checked_sub(liquidity_tokens_burned)
            .ok_or(FinovaDefiError::ArithmeticOverflow)?;
        
        // Update fee checkpoints to current values
        position.fee_growth_checkpoint_a = pool.cumulative_fee_growth_a;
        position.fee_growth_checkpoint_b = pool.cumulative_fee_growth_b;
        
        // Update yield farming checkpoints
        position.reward_debt_a = calculate_reward_debt(
            position.liquidity_tokens,
            pool.acc_reward_per_share_a,
        )?;
        
        position.reward_debt_b = calculate_reward_debt(
            position.liquidity_tokens,
            pool.acc_reward_per_share_b,
        )?;
        
        // Update timestamps
        position.last_interaction = Clock::get()?.unix_timestamp;
        
        // Update position value tracking
        let current_value = self.calculate_position_value(position.liquidity_tokens)?;
        position.total_value_withdrawn = position.total_value_withdrawn
            .checked_add(current_value)
            .ok_or(FinovaDefiError::ArithmeticOverflow)?;
        
        Ok(())
    }
    
    /// Calculate current USD value of position for tracking
    pub fn calculate_position_value(&self, liquidity_tokens: u64) -> Result<u64> {
        let pool = &self.pool;
        let total_lp_supply = self.lp_token_mint.supply;
        
        if total_lp_supply == 0 || liquidity_tokens == 0 {
            return Ok(0);
        }
        
        let token_a_amount = calculate_proportional_amount(
            liquidity_tokens,
            total_lp_supply,
            self.pool_token_a_vault.amount,
        )?;
        
        let token_b_amount = calculate_proportional_amount(
            liquidity_tokens,
            total_lp_supply,
            self.pool_token_b_vault.amount,
        )?;
        
        // Use oracle price or pool price for valuation
        let token_a_value = calculate_token_value(token_a_amount, pool.token_a_price)?;
        let token_b_value = calculate_token_value(token_b_amount, pool.token_b_price)?;
        
        Ok(token_a_value.checked_add(token_b_value).unwrap_or(0))
    }
    
    /// Update pool state after liquidity removal
    pub fn update_pool_state(&mut self, liquidity_tokens_burned: u64, token_a_amount: u64, token_b_amount: u64) -> Result<()> {
        let pool = &mut self.pool;
        
        // Update total liquidity
        pool.total_liquidity = pool.total_liquidity
            .checked_sub(liquidity_tokens_burned)
            .ok_or(FinovaDefiError::ArithmeticOverflow)?;
        
        // Update pool reserves tracking
        pool.reserve_a = pool.reserve_a
            .checked_sub(token_a_amount)
            .ok_or(FinovaDefiError::InsufficientReserves)?;
            
        pool.reserve_b = pool.reserve_b
            .checked_sub(token_b_amount)
            .ok_or(FinovaDefiError::InsufficientReserves)?;
        
        // Update volume tracking
        pool.total_volume_usd = pool.total_volume_usd
            .checked_add(self.calculate_position_value(liquidity_tokens_burned)?)
            .ok_or(FinovaDefiError::ArithmeticOverflow)?;
        
        // Update transaction counter
        pool.total_transactions = pool.total_transactions
            .checked_add(1)
            .ok_or(FinovaDefiError::ArithmeticOverflow)?;
        
        // Update last interaction timestamp
        pool.last_interaction = Clock::get()?.unix_timestamp;
        
        // Update price impact metrics
        self.update_price_impact_metrics(token_a_amount, token_b_amount)?;
        
        Ok(())
    }
    
    /// Update price impact metrics for analytics
    pub fn update_price_impact_metrics(&mut self, token_a_amount: u64, token_b_amount: u64) -> Result<()> {
        let pool = &mut self.pool;
        
        // Calculate price before withdrawal
        let price_before = CurveCalculator::calculate_price(
            pool.reserve_a.checked_add(token_a_amount).unwrap_or(0),
            pool.reserve_b.checked_add(token_b_amount).unwrap_or(0),
            pool.curve_type,
        )?;
        
        // Calculate price after withdrawal (current state)
        let price_after = CurveCalculator::calculate_price(
            pool.reserve_a,
            pool.reserve_b,
            pool.curve_type,
        )?;
        
        // Calculate price impact
        let price_impact = calculate_price_impact(price_before, price_after)?;
        
        // Update maximum price impact if this is larger
        if price_impact > pool.max_price_impact {
            pool.max_price_impact = price_impact;
        }
        
        // Update average price impact
        pool.avg_price_impact = calculate_moving_average(
            pool.avg_price_impact,
            price_impact,
            pool.total_transactions,
        )?;
        
        Ok(())
    }
    
    /// Execute token transfers for withdrawal
    pub fn execute_withdrawal(&self, token_a_amount: u64, token_b_amount: u64, fee_a_amount: u64, fee_b_amount: u64) -> Result<()> {
        let pool_seeds = &[
            POOL_SEED,
            self.pool.token_a_mint.as_ref(),
            self.pool.token_b_mint.as_ref(),
            &[self.pool.bump],
        ];
        let signer_seeds = &[&pool_seeds[..]];
        
        // Transfer token A to user
        if token_a_amount > 0 {
            let transfer_a_ctx = CpiContext::new_with_signer(
                self.token_program.to_account_info(),
                Transfer {
                    from: self.pool_token_a_vault.to_account_info(),
                    to: self.user_token_a_account.to_account_info(),
                    authority: self.pool.to_account_info(),
                },
                signer_seeds,
            );
            
            token::transfer(transfer_a_ctx, token_a_amount)?;
        }
        
        // Transfer token B to user
        if token_b_amount > 0 {
            let transfer_b_ctx = CpiContext::new_with_signer(
                self.token_program.to_account_info(),
                Transfer {
                    from: self.pool_token_b_vault.to_account_info(),
                    to: self.user_token_b_account.to_account_info(),
                    authority: self.pool.to_account_info(),
                },
                signer_seeds,
            );
            
            token::transfer(transfer_b_ctx, token_b_amount)?;
        }
        
        // Transfer accumulated fees to user
        if fee_a_amount > 0 {
            let fee_transfer_a_ctx = CpiContext::new_with_signer(
                self.token_program.to_account_info(),
                Transfer {
                    from: self.fee_collector_a.to_account_info(),
                    to: self.user_token_a_account.to_account_info(),
                    authority: self.pool.to_account_info(),
                },
                signer_seeds,
            );
            
            token::transfer(fee_transfer_a_ctx, fee_a_amount)?;
        }
        
        if fee_b_amount > 0 {
            let fee_transfer_b_ctx = CpiContext::new_with_signer(
                self.token_program.to_account_info(),
                Transfer {
                    from: self.fee_collector_b.to_account_info(),
                    to: self.user_token_b_account.to_account_info(),
                    authority: self.pool.to_account_info(),
                },
                signer_seeds,
            );
            
            token::transfer(fee_transfer_b_ctx, fee_b_amount)?;
        }
        
        Ok(())
    }
    
    /// Burn LP tokens
    pub fn burn_lp_tokens(&self, liquidity_tokens_to_burn: u64) -> Result<()> {
        let burn_ctx = CpiContext::new(
            self.token_program.to_account_info(),
            anchor_spl::token::Burn {
                mint: self.lp_token_mint.to_account_info(),
                from: self.user_lp_token_account.to_account_info(),
                authority: self.user.to_account_info(),
            },
        );
        
        anchor_spl::token::burn(burn_ctx, liquidity_tokens_to_burn)?;
        
        Ok(())
    }
    
    /// Validate slippage protection
    pub fn validate_slippage_protection(
        &self,
        expected_token_a: u64,
        expected_token_b: u64,
        actual_token_a: u64,
        actual_token_b: u64,
        max_slippage_bps: u16,
    ) -> Result<()> {
        let slippage_a = calculate_slippage(expected_token_a, actual_token_a)?;
        let slippage_b = calculate_slippage(expected_token_b, actual_token_b)?;
        
        require!(
            slippage_a <= max_slippage_bps,
            FinovaDefiError::SlippageExceeded
        );
        
        require!(
            slippage_b <= max_slippage_bps,
            FinovaDefiError::SlippageExceeded
        );
        
        Ok(())
    }
}

/// Main instruction handler for removing liquidity
pub fn handler(
    ctx: Context<RemoveLiquidity>,
    liquidity_tokens_to_burn: u64,
    min_token_a_amount: u64,
    min_token_b_amount: u64,
    max_slippage_bps: u16,
) -> Result<()> {
    // Validate inputs
    require!(
        liquidity_tokens_to_burn > 0,
        FinovaDefiError::ZeroAmount
    );
    
    require!(
        max_slippage_bps <= MAX_SLIPPAGE_BPS,
        FinovaDefiError::InvalidSlippage
    );
    
    // Calculate withdrawal amounts
    let (token_a_amount, token_b_amount, fee_a_amount, fee_b_amount) = 
        ctx.accounts.calculate_withdrawal_amounts(liquidity_tokens_to_burn)?;
    
    // Validate minimum amounts (slippage protection)
    require!(
        token_a_amount >= min_token_a_amount,
        FinovaDefiError::InsufficientOutputAmount
    );
    
    require!(
        token_b_amount >= min_token_b_amount,
        FinovaDefiError::InsufficientOutputAmount
    );
    
    // Additional slippage validation
    ctx.accounts.validate_slippage_protection(
        min_token_a_amount,
        min_token_b_amount,
        token_a_amount,
        token_b_amount,
        max_slippage_bps,
    )?;
    
    // Execute withdrawal transfers
    ctx.accounts.execute_withdrawal(token_a_amount, token_b_amount, fee_a_amount, fee_b_amount)?;
    
    // Burn LP tokens
    ctx.accounts.burn_lp_tokens(liquidity_tokens_to_burn)?;
    
    // Update position state
    ctx.accounts.update_position_state(liquidity_tokens_to_burn)?;
    
    // Update pool state
    ctx.accounts.update_pool_state(liquidity_tokens_to_burn, token_a_amount, token_b_amount)?;
    
    // Emit removal event
    emit!(crate::events::LiquidityRemovedEvent {
        pool: ctx.accounts.pool.key(),
        user: ctx.accounts.user.key(),
        liquidity_tokens_burned: liquidity_tokens_to_burn,
        token_a_amount,
        token_b_amount,
        fee_a_amount,
        fee_b_amount,
        timestamp: Clock::get()?.unix_timestamp,
    });
    
    msg!(
        "Liquidity removed successfully: {} LP tokens burned, {} token A, {} token B withdrawn",
        liquidity_tokens_to_burn,
        token_a_amount,
        token_b_amount
    );
    
    Ok(())
}

/// Helper function to calculate proportional amount
fn calculate_proportional_amount(
    liquidity_tokens: u64,
    total_supply: u64,
    reserve: u64,
) -> Result<u64> {
    if total_supply == 0 {
        return Ok(0);
    }
    
    let amount = (liquidity_tokens as u128)
        .checked_mul(reserve as u128)
        .ok_or(FinovaDefiError::ArithmeticOverflow)?
        .checked_div(total_supply as u128)
        .ok_or(FinovaDefiError::ArithmeticOverflow)?;
    
    Ok(amount as u64)
}

/// Helper function to calculate fee share
fn calculate_fee_share(tokens_to_burn: u64, total_tokens: u64) -> Result<u64> {
    if total_tokens == 0 {
        return Ok(0);
    }
    
    let share = (tokens_to_burn as u128)
        .checked_mul(PRECISION as u128)
        .ok_or(FinovaDefiError::ArithmeticOverflow)?
        .checked_div(total_tokens as u128)
        .ok_or(FinovaDefiError::ArithmeticOverflow)?;
    
    Ok(share as u64)
}

/// Helper function to calculate fee amount
fn calculate_fee_amount(
    fee_growth: u64,
    liquidity_tokens: u64,
    fee_share: u64,
) -> Result<u64> {
    let amount = (fee_growth as u128)
        .checked_mul(liquidity_tokens as u128)
        .ok_or(FinovaDefiError::ArithmeticOverflow)?
        .checked_mul(fee_share as u128)
        .ok_or(FinovaDefiError::ArithmeticOverflow)?
        .checked_div((PRECISION as u128).checked_mul(PRECISION as u128).unwrap())
        .ok_or(FinovaDefiError::ArithmeticOverflow)?;
    
    Ok(amount as u64)
}

/// Helper function to calculate reward debt
fn calculate_reward_debt(liquidity_tokens: u64, acc_reward_per_share: u64) -> Result<u64> {
    let debt = (liquidity_tokens as u128)
        .checked_mul(acc_reward_per_share as u128)
        .ok_or(FinovaDefiError::ArithmeticOverflow)?
        .checked_div(PRECISION as u128)
        .ok_or(FinovaDefiError::ArithmeticOverflow)?;
    
    Ok(debt as u64)
}

/// Helper function to calculate token value in USD
fn calculate_token_value(amount: u64, price: u64) -> Result<u64> {
    let value = (amount as u128)
        .checked_mul(price as u128)
        .ok_or(FinovaDefiError::ArithmeticOverflow)?
        .checked_div(PRICE_PRECISION as u128)
        .ok_or(FinovaDefiError::ArithmeticOverflow)?;
    
    Ok(value as u64)
}

/// Helper function to calculate price impact
fn calculate_price_impact(price_before: u64, price_after: u64) -> Result<u64> {
    if price_before == 0 {
        return Ok(0);
    }
    
    let diff = if price_after > price_before {
        price_after - price_before
    } else {
        price_before - price_after
    };
    
    let impact = (diff as u128)
        .checked_mul(BASIS_POINTS as u128)
        .ok_or(FinovaDefiError::ArithmeticOverflow)?
        .checked_div(price_before as u128)
        .ok_or(FinovaDefiError::ArithmeticOverflow)?;
    
    Ok(impact as u64)
}

/// Helper function to calculate moving average
fn calculate_moving_average(current_avg: u64, new_value: u64, count: u64) -> Result<u64> {
    if count == 0 {
        return Ok(new_value);
    }
    
    let total = (current_avg as u128)
        .checked_mul((count - 1) as u128)
        .ok_or(FinovaDefiError::ArithmeticOverflow)?
        .checked_add(new_value as u128)
        .ok_or(FinovaDefiError::ArithmeticOverflow)?;
    
    let avg = total
        .checked_div(count as u128)
        .ok_or(FinovaDefiError::ArithmeticOverflow)?;
    
    Ok(avg as u64)
}

/// Helper function to calculate slippage
fn calculate_slippage(expected: u64, actual: u64) -> Result<u16> {
    if expected == 0 {
        return Ok(0);
    }
    
    let diff = if expected > actual {
        expected - actual
    } else {
        actual - expected
    };
    
    let slippage = (diff as u128)
        .checked_mul(BASIS_POINTS as u128)
        .ok_or(FinovaDefiError::ArithmeticOverflow)?
        .checked_div(expected as u128)
        .ok_or(FinovaDefiError::ArithmeticOverflow)?;
    
    Ok(slippage as u16)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_calculate_proportional_amount() {
        let result = calculate_proportional_amount(1000, 10000, 5000).unwrap();
        assert_eq!(result, 500);
        
        let result = calculate_proportional_amount(0, 10000, 5000).unwrap();
        assert_eq!(result, 0);
        
        let result = calculate_proportional_amount(1000, 0, 5000).unwrap();
        assert_eq!(result, 0);
    }
    
    #[test]
    fn test_calculate_fee_share() {
        let result = calculate_fee_share(1000, 10000).unwrap();
        assert_eq!(result, PRECISION / 10);
        
        let result = calculate_fee_share(0, 10000).unwrap();
        assert_eq!(result, 0);
        
        let result = calculate_fee_share(1000, 0).unwrap();
        assert_eq!(result, 0);
    }
    
    #[test]
    fn test_calculate_slippage() {
        let result = calculate_slippage(1000, 950).unwrap();
        assert_eq!(result, 500); // 5% slippage = 500 basis points
        
        let result = calculate_slippage(1000, 1000).unwrap();
        assert_eq!(result, 0);
        
        let result = calculate_slippage(0, 100).unwrap();
        assert_eq!(result, 0);
    }
}
