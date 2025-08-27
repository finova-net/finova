// programs/finova-defi/src/instructions/add_liquidity.rs

use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer, Mint};
use crate::state::{Pool, LiquidityPosition};
use crate::math::{curve::calculate_liquidity_tokens, fees::calculate_deposit_fee};
use crate::errors::FinovaDefiError;
use crate::constants::*;
use crate::utils::{validate_slippage, check_pool_active, emit_liquidity_event};

/// Add liquidity to a DeFi pool
/// Implements constant product formula (x * y = k) with fee calculations
/// Supports both balanced and single-sided liquidity provision
#[derive(Accounts)]
#[instruction(amount_a: u64, amount_b: u64, min_liquidity: u64)]
pub struct AddLiquidity<'info> {
    #[account(
        mut,
        has_one = token_mint_a,
        has_one = token_mint_b,
        constraint = pool.is_active @ FinovaDefiError::PoolInactive,
        constraint = pool.liquidity_supply < MAX_POOL_LIQUIDITY @ FinovaDefiError::PoolCapacityExceeded
    )]
    pub pool: Account<'info, Pool>,

    /// Token A mint
    pub token_mint_a: Account<'info, Mint>,
    
    /// Token B mint  
    pub token_mint_b: Account<'info, Mint>,

    /// Pool's token A account
    #[account(
        mut,
        constraint = pool_token_a.mint == token_mint_a.key(),
        constraint = pool_token_a.owner == pool.key()
    )]
    pub pool_token_a: Account<'info, TokenAccount>,

    /// Pool's token B account
    #[account(
        mut,
        constraint = pool_token_b.mint == token_mint_b.key(),
        constraint = pool_token_b.owner == pool.key()
    )]
    pub pool_token_b: Account<'info, TokenAccount>,

    /// Liquidity token mint (LP tokens)
    #[account(
        mut,
        constraint = liquidity_mint.key() == pool.liquidity_mint @ FinovaDefiError::InvalidLiquidityMint
    )]
    pub liquidity_mint: Account<'info, Mint>,

    /// User's token A account
    #[account(
        mut,
        constraint = user_token_a.mint == token_mint_a.key(),
        constraint = user_token_a.owner == user.key(),
        constraint = user_token_a.amount >= amount_a @ FinovaDefiError::InsufficientFunds
    )]
    pub user_token_a: Account<'info, TokenAccount>,

    /// User's token B account
    #[account(
        mut,
        constraint = user_token_b.mint == token_mint_b.key(),
        constraint = user_token_b.owner == user.key(),
        constraint = user_token_b.amount >= amount_b @ FinovaDefiError::InsufficientFunds
    )]
    pub user_token_b: Account<'info, TokenAccount>,

    /// User's liquidity token account (to receive LP tokens)
    #[account(
        mut,
        constraint = user_liquidity.mint == liquidity_mint.key(),
        constraint = user_liquidity.owner == user.key()
    )]
    pub user_liquidity: Account<'info, TokenAccount>,

    /// User's liquidity position account
    #[account(
        init_if_needed,
        payer = user,
        space = LiquidityPosition::LEN,
        seeds = [
            b"liquidity_position",
            pool.key().as_ref(),
            user.key().as_ref()
        ],
        bump
    )]
    pub liquidity_position: Account<'info, LiquidityPosition>,

    /// Fee recipient account for protocol fees
    #[account(
        mut,
        constraint = fee_recipient.mint == token_mint_a.key() || fee_recipient.mint == token_mint_b.key()
    )]
    pub fee_recipient: Account<'info, TokenAccount>,

    /// User providing liquidity
    #[account(mut)]
    pub user: Signer<'info>,

    /// Pool authority PDA
    /// CHECK: This is validated through seeds constraint
    #[account(
        seeds = [b"pool_authority", pool.key().as_ref()],
        bump = pool.authority_bump
    )]
    pub pool_authority: UncheckedAccount<'info>,

    /// Token program
    pub token_program: Program<'info, Token>,
    
    /// System program
    pub system_program: Program<'info, System>,
    
    /// Clock sysvar for timestamp
    pub clock: Sysvar<'info, Clock>,
}

impl<'info> AddLiquidity<'info> {
    /// Transfer tokens from user to pool
    fn transfer_tokens_to_pool(&self, amount_a: u64, amount_b: u64) -> Result<()> {
        // Transfer token A to pool
        if amount_a > 0 {
            let transfer_a_ctx = CpiContext::new(
                self.token_program.to_account_info(),
                Transfer {
                    from: self.user_token_a.to_account_info(),
                    to: self.pool_token_a.to_account_info(),
                    authority: self.user.to_account_info(),
                },
            );
            token::transfer(transfer_a_ctx, amount_a)?;
        }

        // Transfer token B to pool
        if amount_b > 0 {
            let transfer_b_ctx = CpiContext::new(
                self.token_program.to_account_info(),
                Transfer {
                    from: self.user_token_b.to_account_info(),
                    to: self.pool_token_b.to_account_info(),
                    authority: self.user.to_account_info(),
                },
            );
            token::transfer(transfer_b_ctx, amount_b)?;
        }

        Ok(())
    }

    /// Mint liquidity tokens to user
    fn mint_liquidity_tokens(&self, liquidity_amount: u64) -> Result<()> {
        let pool_key = self.pool.key();
        let authority_seeds = &[
            b"pool_authority",
            pool_key.as_ref(),
            &[self.pool.authority_bump],
        ];
        let signer_seeds = &[&authority_seeds[..]];

        let mint_ctx = CpiContext::new_with_signer(
            self.token_program.to_account_info(),
            token::MintTo {
                mint: self.liquidity_mint.to_account_info(),
                to: self.user_liquidity.to_account_info(),
                authority: self.pool_authority.to_account_info(),
            },
            signer_seeds,
        );

        token::mint_to(mint_ctx, liquidity_amount)?;
        Ok(())
    }

    /// Calculate optimal amounts for balanced liquidity provision
    fn calculate_optimal_amounts(&self, desired_a: u64, desired_b: u64) -> Result<(u64, u64)> {
        let reserve_a = self.pool_token_a.amount;
        let reserve_b = self.pool_token_b.amount;

        // If this is the first liquidity provision
        if reserve_a == 0 && reserve_b == 0 {
            require!(desired_a > 0 && desired_b > 0, FinovaDefiError::InvalidLiquidityAmount);
            return Ok((desired_a, desired_b));
        }

        // Calculate optimal amounts to maintain pool ratio
        let amount_b_optimal = (desired_a as u128)
            .checked_mul(reserve_b as u128)
            .ok_or(FinovaDefiError::MathOverflow)?
            .checked_div(reserve_a as u128)
            .ok_or(FinovaDefiError::MathOverflow)? as u64;

        if amount_b_optimal <= desired_b {
            Ok((desired_a, amount_b_optimal))
        } else {
            let amount_a_optimal = (desired_b as u128)
                .checked_mul(reserve_a as u128)
                .ok_or(FinovaDefiError::MathOverflow)?
                .checked_div(reserve_b as u128)
                .ok_or(FinovaDefiError::MathOverflow)? as u64;
            
            require!(amount_a_optimal <= desired_a, FinovaDefiError::InsufficientLiquidity);
            Ok((amount_a_optimal, desired_b))
        }
    }

    /// Calculate deposit fees
    fn calculate_fees(&self, amount_a: u64, amount_b: u64) -> Result<(u64, u64)> {
        let fee_a = calculate_deposit_fee(amount_a, self.pool.fee_rate)?;
        let fee_b = calculate_deposit_fee(amount_b, self.pool.fee_rate)?;
        Ok((fee_a, fee_b))
    }

    /// Update liquidity position
    fn update_liquidity_position(&mut self, liquidity_tokens: u64, amount_a: u64, amount_b: u64) -> Result<()> {
        let position = &mut self.liquidity_position;
        let clock = &self.clock;

        // Initialize position if needed
        if position.pool.eq(&Pubkey::default()) {
            position.pool = self.pool.key();
            position.owner = self.user.key();
            position.created_at = clock.unix_timestamp;
            position.liquidity_tokens = 0;
            position.total_deposited_a = 0;
            position.total_deposited_b = 0;
            position.rewards_earned = 0;
            position.last_claim_time = clock.unix_timestamp;
            position.bump = *ctx.bumps.get("liquidity_position").unwrap();
        }

        // Update position
        position.liquidity_tokens = position.liquidity_tokens
            .checked_add(liquidity_tokens)
            .ok_or(FinovaDefiError::MathOverflow)?;
        
        position.total_deposited_a = position.total_deposited_a
            .checked_add(amount_a)
            .ok_or(FinovaDefiError::MathOverflow)?;
        
        position.total_deposited_b = position.total_deposited_b
            .checked_add(amount_b)
            .ok_or(FinovaDefiError::MathOverflow)?;
        
        position.last_update_time = clock.unix_timestamp;

        Ok(())
    }

    /// Validate slippage protection
    fn validate_slippage(&self, liquidity_tokens: u64, min_liquidity: u64) -> Result<()> {
        require!(
            liquidity_tokens >= min_liquidity,
            FinovaDefiError::SlippageExceeded
        );
        Ok(())
    }

    /// Update pool statistics
    fn update_pool_stats(&mut self, amount_a: u64, amount_b: u64, liquidity_tokens: u64) -> Result<()> {
        let pool = &mut self.pool;
        let clock = &self.clock;

        // Update reserves
        pool.reserve_a = pool.reserve_a
            .checked_add(amount_a)
            .ok_or(FinovaDefiError::MathOverflow)?;
        
        pool.reserve_b = pool.reserve_b
            .checked_add(amount_b)
            .ok_or(FinovaDefiError::MathOverflow)?;

        // Update liquidity supply
        pool.liquidity_supply = pool.liquidity_supply
            .checked_add(liquidity_tokens)
            .ok_or(FinovaDefiError::MathOverflow)?;

        // Update pool statistics
        pool.total_volume_a = pool.total_volume_a
            .checked_add(amount_a)
            .ok_or(FinovaDefiError::MathOverflow)?;
        
        pool.total_volume_b = pool.total_volume_b
            .checked_add(amount_b)
            .ok_or(FinovaDefiError::MathOverflow)?;

        pool.liquidity_providers_count = if self.liquidity_position.liquidity_tokens == liquidity_tokens {
            // New liquidity provider
            pool.liquidity_providers_count
                .checked_add(1)
                .ok_or(FinovaDefiError::MathOverflow)?
        } else {
            pool.liquidity_providers_count
        };

        pool.last_update_time = clock.unix_timestamp;

        // Update K constant for AMM invariant
        pool.k_last = (pool.reserve_a as u128)
            .checked_mul(pool.reserve_b as u128)
            .ok_or(FinovaDefiError::MathOverflow)?;

        Ok(())
    }
}

/// Main instruction handler for adding liquidity
pub fn handler(
    ctx: Context<AddLiquidity>,
    amount_a: u64,
    amount_b: u64,
    min_liquidity: u64,
) -> Result<()> {
    msg!("Adding liquidity: {} token A, {} token B", amount_a, amount_b);

    // Validate input amounts
    require!(amount_a > 0 || amount_b > 0, FinovaDefiError::InvalidLiquidityAmount);
    require!(amount_a <= MAX_TOKEN_AMOUNT, FinovaDefiError::AmountTooLarge);
    require!(amount_b <= MAX_TOKEN_AMOUNT, FinovaDefiError::AmountTooLarge);

    // Check pool is active
    check_pool_active(&ctx.accounts.pool)?;

    // Calculate optimal amounts for balanced liquidity
    let (optimal_amount_a, optimal_amount_b) = ctx.accounts
        .calculate_optimal_amounts(amount_a, amount_b)?;

    msg!("Optimal amounts: {} token A, {} token B", optimal_amount_a, optimal_amount_b);

    // Calculate fees
    let (fee_a, fee_b) = ctx.accounts.calculate_fees(optimal_amount_a, optimal_amount_b)?;
    let net_amount_a = optimal_amount_a.checked_sub(fee_a).ok_or(FinovaDefiError::MathOverflow)?;
    let net_amount_b = optimal_amount_b.checked_sub(fee_b).ok_or(FinovaDefiError::MathOverflow)?;

    // Calculate liquidity tokens to mint
    let liquidity_tokens = calculate_liquidity_tokens(
        net_amount_a,
        net_amount_b,
        ctx.accounts.pool.reserve_a,
        ctx.accounts.pool.reserve_b,
        ctx.accounts.liquidity_mint.supply,
    )?;

    msg!("Liquidity tokens to mint: {}", liquidity_tokens);

    // Validate slippage protection
    ctx.accounts.validate_slippage(liquidity_tokens, min_liquidity)?;

    // Perform token transfers
    ctx.accounts.transfer_tokens_to_pool(optimal_amount_a, optimal_amount_b)?;

    // Mint liquidity tokens to user
    ctx.accounts.mint_liquidity_tokens(liquidity_tokens)?;

    // Update liquidity position
    ctx.accounts.update_liquidity_position(liquidity_tokens, net_amount_a, net_amount_b)?;

    // Update pool statistics
    ctx.accounts.update_pool_stats(net_amount_a, net_amount_b, liquidity_tokens)?;

    // Emit liquidity added event
    emit_liquidity_event(
        "liquidity_added",
        &ctx.accounts.pool.key(),
        &ctx.accounts.user.key(),
        optimal_amount_a,
        optimal_amount_b,
        liquidity_tokens,
        fee_a + fee_b,
    )?;

    msg!("Successfully added liquidity to pool");
    Ok(())
}

/// Context for single-sided liquidity provision
#[derive(Accounts)]
#[instruction(token_amount: u64, token_side: u8)]
pub struct AddSingleSidedLiquidity<'info> {
    #[account(
        mut,
        constraint = pool.is_active @ FinovaDefiError::PoolInactive,
        constraint = pool.supports_single_sided @ FinovaDefiError::SingleSidedNotSupported
    )]
    pub pool: Account<'info, Pool>,

    /// User's token account (either A or B based on token_side)
    #[account(
        mut,
        constraint = user_token.owner == user.key(),
        constraint = user_token.amount >= token_amount @ FinovaDefiError::InsufficientFunds
    )]
    pub user_token: Account<'info, TokenAccount>,

    /// Pool's corresponding token account
    #[account(mut)]
    pub pool_token: Account<'info, TokenAccount>,

    /// Liquidity token mint
    #[account(mut)]
    pub liquidity_mint: Account<'info, Mint>,

    /// User's liquidity token account
    #[account(mut)]
    pub user_liquidity: Account<'info, TokenAccount>,

    /// User's liquidity position
    #[account(
        init_if_needed,
        payer = user,
        space = LiquidityPosition::LEN,
        seeds = [
            b"liquidity_position",
            pool.key().as_ref(),
            user.key().as_ref()
        ],
        bump
    )]
    pub liquidity_position: Account<'info, LiquidityPosition>,

    #[account(mut)]
    pub user: Signer<'info>,

    /// CHECK: Pool authority PDA
    #[account(
        seeds = [b"pool_authority", pool.key().as_ref()],
        bump = pool.authority_bump
    )]
    pub pool_authority: UncheckedAccount<'info>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub clock: Sysvar<'info, Clock>,
}

/// Handler for single-sided liquidity provision
pub fn add_single_sided_liquidity_handler(
    ctx: Context<AddSingleSidedLiquidity>,
    token_amount: u64,
    token_side: u8, // 0 for token A, 1 for token B
    min_liquidity: u64,
) -> Result<()> {
    msg!("Adding single-sided liquidity: {} tokens, side: {}", token_amount, token_side);

    require!(token_amount > 0, FinovaDefiError::InvalidLiquidityAmount);
    require!(token_side <= 1, FinovaDefiError::InvalidTokenSide);

    // Validate pool supports single-sided liquidity
    require!(ctx.accounts.pool.supports_single_sided, FinovaDefiError::SingleSidedNotSupported);

    // Calculate equivalent liquidity with impact penalty
    let impact_penalty = calculate_single_sided_impact(
        token_amount,
        if token_side == 0 { ctx.accounts.pool.reserve_a } else { ctx.accounts.pool.reserve_b },
        if token_side == 0 { ctx.accounts.pool.reserve_b } else { ctx.accounts.pool.reserve_a },
    )?;

    let effective_amount = token_amount
        .checked_sub(impact_penalty)
        .ok_or(FinovaDefiError::MathOverflow)?;

    // Calculate liquidity tokens (reduced due to single-sided penalty)
    let liquidity_tokens = if token_side == 0 {
        calculate_liquidity_tokens(
            effective_amount,
            0,
            ctx.accounts.pool.reserve_a,
            ctx.accounts.pool.reserve_b,
            ctx.accounts.liquidity_mint.supply,
        )?
    } else {
        calculate_liquidity_tokens(
            0,
            effective_amount,
            ctx.accounts.pool.reserve_a,
            ctx.accounts.pool.reserve_b,
            ctx.accounts.liquidity_mint.supply,
        )?
    };

    // Apply single-sided penalty (typically 50-80% of balanced liquidity)
    let adjusted_liquidity = liquidity_tokens
        .checked_mul(SINGLE_SIDED_PENALTY_MULTIPLIER)
        .ok_or(FinovaDefiError::MathOverflow)?
        .checked_div(10000)
        .ok_or(FinovaDefiError::MathOverflow)?;

    require!(adjusted_liquidity >= min_liquidity, FinovaDefiError::SlippageExceeded);

    // Transfer tokens to pool
    let transfer_ctx = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        Transfer {
            from: ctx.accounts.user_token.to_account_info(),
            to: ctx.accounts.pool_token.to_account_info(),
            authority: ctx.accounts.user.to_account_info(),
        },
    );
    token::transfer(transfer_ctx, token_amount)?;

    // Mint adjusted liquidity tokens
    let pool_key = ctx.accounts.pool.key();
    let authority_seeds = &[
        b"pool_authority",
        pool_key.as_ref(),
        &[ctx.accounts.pool.authority_bump],
    ];
    let signer_seeds = &[&authority_seeds[..]];

    let mint_ctx = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        token::MintTo {
            mint: ctx.accounts.liquidity_mint.to_account_info(),
            to: ctx.accounts.user_liquidity.to_account_info(),
            authority: ctx.accounts.pool_authority.to_account_info(),
        },
        signer_seeds,
    );
    token::mint_to(mint_ctx, adjusted_liquidity)?;

    msg!("Successfully added single-sided liquidity");
    Ok(())
}

/// Calculate impact penalty for single-sided liquidity provision
fn calculate_single_sided_impact(
    deposit_amount: u64,
    same_token_reserve: u64,
    other_token_reserve: u64,
) -> Result<u64> {
    // Calculate impact based on how much the deposit changes the pool balance
    let impact_ratio = (deposit_amount as u128)
        .checked_mul(10000)
        .ok_or(FinovaDefiError::MathOverflow)?
        .checked_div(same_token_reserve as u128)
        .ok_or(FinovaDefiError::MathOverflow)? as u64;

    // Progressive penalty: higher impact = higher penalty
    let penalty_rate = if impact_ratio < 100 { // < 1%
        50 // 0.5%
    } else if impact_ratio < 500 { // < 5%
        200 // 2%
    } else if impact_ratio < 1000 { // < 10%
        500 // 5%
    } else {
        1000 // 10% max penalty
    };

    let penalty = deposit_amount
        .checked_mul(penalty_rate)
        .ok_or(FinovaDefiError::MathOverflow)?
        .checked_div(10000)
        .ok_or(FinovaDefiError::MathOverflow)?;

    Ok(penalty)
}
