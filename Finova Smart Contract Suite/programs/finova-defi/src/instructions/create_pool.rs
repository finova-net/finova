// programs/finova-defi/src/instructions/create_pool.rs

use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{self, Mint, Token, TokenAccount, Transfer},
};
use crate::{
    constants::*,
    errors::FinovaDefiError,
    math::{curve::CurveCalculator, fees::FeeCalculator},
    state::{pool::Pool, liquidity_position::LiquidityPosition},
    utils::*,
};

#[derive(Accounts)]
#[instruction(pool_id: u64, fee_rate: u16)]
pub struct CreatePool<'info> {
    #[account(mut)]
    pub creator: Signer<'info>,

    /// Pool state account
    #[account(
        init,
        payer = creator,
        space = Pool::LEN,
        seeds = [
            POOL_SEED.as_bytes(),
            token_a_mint.key().as_ref(),
            token_b_mint.key().as_ref(),
            &pool_id.to_le_bytes()
        ],
        bump
    )]
    pub pool: Account<'info, Pool>,

    /// Token A mint
    pub token_a_mint: Account<'info, Mint>,

    /// Token B mint  
    pub token_b_mint: Account<'info, Mint>,

    /// LP token mint (to be created)
    #[account(
        init,
        payer = creator,
        mint::decimals = LP_TOKEN_DECIMALS,
        mint::authority = pool,
        seeds = [
            LP_TOKEN_SEED.as_bytes(),
            pool.key().as_ref()
        ],
        bump
    )]
    pub lp_token_mint: Account<'info, Mint>,

    /// Pool's token A vault
    #[account(
        init,
        payer = creator,
        associated_token::mint = token_a_mint,
        associated_token::authority = pool
    )]
    pub token_a_vault: Account<'info, TokenAccount>,

    /// Pool's token B vault
    #[account(
        init,
        payer = creator,
        associated_token::authority = pool,
        associated_token::mint = token_b_mint
    )]
    pub token_b_vault: Account<'info, TokenAccount>,

    /// Creator's token A account
    #[account(
        mut,
        associated_token::mint = token_a_mint,
        associated_token::authority = creator
    )]
    pub creator_token_a: Account<'info, TokenAccount>,

    /// Creator's token B account
    #[account(
        mut,
        associated_token::mint = token_b_mint,
        associated_token::authority = creator
    )]
    pub creator_token_b: Account<'info, TokenAccount>,

    /// Creator's LP token account
    #[account(
        init,
        payer = creator,
        associated_token::mint = lp_token_mint,
        associated_token::authority = creator
    )]
    pub creator_lp_tokens: Account<'info, TokenAccount>,

    /// Initial liquidity position account
    #[account(
        init,
        payer = creator,
        space = LiquidityPosition::LEN,
        seeds = [
            LIQUIDITY_POSITION_SEED.as_bytes(),
            pool.key().as_ref(),
            creator.key().as_ref()
        ],
        bump
    )]
    pub liquidity_position: Account<'info, LiquidityPosition>,

    /// Fee collector account (protocol treasury)
    /// CHECK: This is validated in the instruction
    pub fee_collector: UncheckedAccount<'info>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

impl<'info> CreatePool<'info> {
    pub fn validate_inputs(
        &self,
        pool_id: u64,
        fee_rate: u16,
        initial_token_a_amount: u64,
        initial_token_b_amount: u64,
    ) -> Result<()> {
        // Validate fee rate
        require!(
            fee_rate <= MAX_FEE_RATE && fee_rate >= MIN_FEE_RATE,
            FinovaDefiError::InvalidFeeRate
        );

        // Validate initial amounts
        require!(
            initial_token_a_amount >= MIN_INITIAL_LIQUIDITY &&
            initial_token_b_amount >= MIN_INITIAL_LIQUIDITY,
            FinovaDefiError::InsufficientInitialLiquidity
        );

        require!(
            initial_token_a_amount <= MAX_INITIAL_LIQUIDITY &&
            initial_token_b_amount <= MAX_INITIAL_LIQUIDITY,
            FinovaDefiError::ExcessiveInitialLiquidity
        );

        // Validate token mints are different
        require!(
            self.token_a_mint.key() != self.token_b_mint.key(),
            FinovaDefiError::IdenticalTokenMints
        );

        // Validate pool ID
        require!(pool_id > 0, FinovaDefiError::InvalidPoolId);

        // Validate creator has sufficient balances
        require!(
            self.creator_token_a.amount >= initial_token_a_amount,
            FinovaDefiError::InsufficientTokenABalance
        );

        require!(
            self.creator_token_b.amount >= initial_token_b_amount,
            FinovaDefiError::InsufficientTokenBBalance
        );

        // Validate fee collector
        require!(
            self.fee_collector.key() == PROTOCOL_FEE_COLLECTOR,
            FinovaDefiError::InvalidFeeCollector
        );

        Ok(())
    }

    pub fn transfer_initial_liquidity(
        &self,
        token_a_amount: u64,
        token_b_amount: u64,
    ) -> Result<()> {
        // Transfer token A to pool vault
        let transfer_a_ctx = CpiContext::new(
            self.token_program.to_account_info(),
            Transfer {
                from: self.creator_token_a.to_account_info(),
                to: self.token_a_vault.to_account_info(),
                authority: self.creator.to_account_info(),
            },
        );
        token::transfer(transfer_a_ctx, token_a_amount)?;

        // Transfer token B to pool vault
        let transfer_b_ctx = CpiContext::new(
            self.token_program.to_account_info(),
            Transfer {
                from: self.creator_token_b.to_account_info(),
                to: self.token_b_vault.to_account_info(),
                authority: self.creator.to_account_info(),
            },
        );
        token::transfer(transfer_b_ctx, token_b_amount)?;

        Ok(())
    }

    pub fn mint_initial_lp_tokens(
        &self,
        lp_token_amount: u64,
        bump: u8,
    ) -> Result<()> {
        let pool_key = self.pool.key();
        let seeds = &[
            LP_TOKEN_SEED.as_bytes(),
            pool_key.as_ref(),
            &[bump],
        ];
        let signer_seeds = &[&seeds[..]];

        let mint_ctx = CpiContext::new_with_signer(
            self.token_program.to_account_info(),
            token::MintTo {
                mint: self.lp_token_mint.to_account_info(),
                to: self.creator_lp_tokens.to_account_info(),
                authority: self.pool.to_account_info(),
            },
            signer_seeds,
        );

        token::mint_to(mint_ctx, lp_token_amount)?;

        Ok(())
    }

    pub fn calculate_initial_lp_tokens(
        &self,
        token_a_amount: u64,
        token_b_amount: u64,
    ) -> Result<u64> {
        // For initial liquidity, use geometric mean
        let lp_tokens = CurveCalculator::calculate_initial_lp_tokens(
            token_a_amount,
            token_b_amount,
            self.token_a_mint.decimals,
            self.token_b_mint.decimals,
        )?;

        // Ensure minimum LP tokens are minted
        require!(
            lp_tokens >= MIN_LP_TOKENS,
            FinovaDefiError::InsufficientLPTokens
        );

        Ok(lp_tokens)
    }
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct CreatePoolParams {
    pub pool_id: u64,
    pub fee_rate: u16,
    pub initial_token_a_amount: u64,
    pub initial_token_b_amount: u64,
    pub curve_type: u8, // 0 = ConstantProduct, 1 = StableSwap, 2 = Weighted
    pub amp_factor: Option<u64>, // For StableSwap curves
    pub weights: Option<[u16; 2]>, // For Weighted pools [weight_a, weight_b]
}

pub fn handler(
    ctx: Context<CreatePool>,
    params: CreatePoolParams,
) -> Result<()> {
    let CreatePoolParams {
        pool_id,
        fee_rate,
        initial_token_a_amount,
        initial_token_b_amount,
        curve_type,
        amp_factor,
        weights,
    } = params;

    // Validate all inputs
    ctx.accounts.validate_inputs(
        pool_id,
        fee_rate,
        initial_token_a_amount,
        initial_token_b_amount,
    )?;

    // Validate curve parameters
    match curve_type {
        0 => {}, // ConstantProduct - no additional params needed
        1 => {
            // StableSwap
            require!(
                amp_factor.is_some() && 
                amp_factor.unwrap() >= MIN_AMP_FACTOR && 
                amp_factor.unwrap() <= MAX_AMP_FACTOR,
                FinovaDefiError::InvalidAmpFactor
            );
        },
        2 => {
            // Weighted
            require!(weights.is_some(), FinovaDefiError::MissingWeights);
            let weights = weights.unwrap();
            require!(
                weights[0] + weights[1] == TOTAL_WEIGHT &&
                weights[0] >= MIN_WEIGHT &&
                weights[1] >= MIN_WEIGHT,
                FinovaDefiError::InvalidWeights
            );
        },
        _ => return Err(FinovaDefiError::InvalidCurveType.into()),
    }

    // Calculate initial LP tokens
    let lp_token_amount = ctx.accounts.calculate_initial_lp_tokens(
        initial_token_a_amount,
        initial_token_b_amount,
    )?;

    // Transfer initial liquidity to pool
    ctx.accounts.transfer_initial_liquidity(
        initial_token_a_amount,
        initial_token_b_amount,
    )?;

    // Get bump for LP token mint
    let (_, lp_bump) = Pubkey::find_program_address(
        &[
            LP_TOKEN_SEED.as_bytes(),
            ctx.accounts.pool.key().as_ref(),
        ],
        ctx.program_id,
    );

    // Mint initial LP tokens to creator
    ctx.accounts.mint_initial_lp_tokens(lp_token_amount, lp_bump)?;

    // Initialize pool state
    let pool = &mut ctx.accounts.pool;
    pool.bump = ctx.bumps.pool;
    pool.pool_id = pool_id;
    pool.creator = ctx.accounts.creator.key();
    pool.token_a_mint = ctx.accounts.token_a_mint.key();
    pool.token_b_mint = ctx.accounts.token_b_mint.key();
    pool.lp_token_mint = ctx.accounts.lp_token_mint.key();
    pool.token_a_vault = ctx.accounts.token_a_vault.key();
    pool.token_b_vault = ctx.accounts.token_b_vault.key();
    pool.fee_rate = fee_rate;
    pool.protocol_fee_rate = PROTOCOL_FEE_RATE;
    pool.curve_type = curve_type;
    pool.amp_factor = amp_factor.unwrap_or(0);
    pool.weights = weights.unwrap_or([5000, 5000]); // Default 50-50
    pool.token_a_reserve = initial_token_a_amount;
    pool.token_b_reserve = initial_token_b_amount;
    pool.lp_token_supply = lp_token_amount;
    pool.cumulative_volume_a = 0;
    pool.cumulative_volume_b = 0;
    pool.cumulative_fees_a = 0;
    pool.cumulative_fees_b = 0;
    pool.last_update_slot = Clock::get()?.slot;
    pool.created_at = Clock::get()?.unix_timestamp;
    pool.is_active = true;
    pool.is_frozen = false;
    pool.emergency_mode = false;

    // Initialize liquidity position
    let liquidity_position = &mut ctx.accounts.liquidity_position;
    liquidity_position.bump = ctx.bumps.liquidity_position;
    liquidity_position.pool = ctx.accounts.pool.key();
    liquidity_position.owner = ctx.accounts.creator.key();
    liquidity_position.lp_token_amount = lp_token_amount;
    liquidity_position.token_a_deposited = initial_token_a_amount;
    liquidity_position.token_b_deposited = initial_token_b_amount;
    liquidity_position.fees_earned_a = 0;
    liquidity_position.fees_earned_b = 0;
    liquidity_position.created_at = Clock::get()?.unix_timestamp;
    liquidity_position.last_update = Clock::get()?.unix_timestamp;

    // Calculate and validate initial price
    let initial_price = CurveCalculator::calculate_price(
        initial_token_a_amount,
        initial_token_b_amount,
        curve_type,
        amp_factor.unwrap_or(0),
        &pool.weights,
    )?;

    require!(
        initial_price > 0,
        FinovaDefiError::InvalidInitialPrice
    );

    pool.last_price = initial_price;

    // Emit pool creation event
    emit!(PoolCreated {
        pool: ctx.accounts.pool.key(),
        creator: ctx.accounts.creator.key(),
        token_a_mint: ctx.accounts.token_a_mint.key(),
        token_b_mint: ctx.accounts.token_b_mint.key(),
        lp_token_mint: ctx.accounts.lp_token_mint.key(),
        pool_id,
        fee_rate,
        curve_type,
        initial_token_a_amount,
        initial_token_b_amount,
        initial_lp_tokens: lp_token_amount,
        initial_price,
        timestamp: Clock::get()?.unix_timestamp,
    });

    // Update global statistics (if global state exists)
    // This would typically be done in a separate account
    msg!(
        "Pool created successfully: {}, LP tokens: {}, Price: {}",
        ctx.accounts.pool.key(),
        lp_token_amount,
        initial_price
    );

    Ok(())
}

#[event]
pub struct PoolCreated {
    pub pool: Pubkey,
    pub creator: Pubkey,
    pub token_a_mint: Pubkey,
    pub token_b_mint: Pubkey,
    pub lp_token_mint: Pubkey,
    pub pool_id: u64,
    pub fee_rate: u16,
    pub curve_type: u8,
    pub initial_token_a_amount: u64,
    pub initial_token_b_amount: u64,
    pub initial_lp_tokens: u64,
    pub initial_price: u64,
    pub timestamp: i64,
}

// Helper functions for additional validations
impl<'info> CreatePool<'info> {
    pub fn validate_token_decimals(&self) -> Result<()> {
        require!(
            self.token_a_mint.decimals <= MAX_TOKEN_DECIMALS &&
            self.token_b_mint.decimals <= MAX_TOKEN_DECIMALS,
            FinovaDefiError::InvalidTokenDecimals
        );

        Ok(())
    }

    pub fn validate_economic_parameters(
        &self,
        initial_token_a_amount: u64,
        initial_token_b_amount: u64,
    ) -> Result<()> {
        // Check for reasonable price ratios to prevent manipulation
        let decimal_a = 10_u64.pow(self.token_a_mint.decimals as u32);
        let decimal_b = 10_u64.pow(self.token_b_mint.decimals as u32);
        
        let normalized_a = initial_token_a_amount * decimal_b;
        let normalized_b = initial_token_b_amount * decimal_a;
        
        let ratio = if normalized_a > normalized_b {
            normalized_a / normalized_b
        } else {
            normalized_b / normalized_a
        };

        require!(
            ratio <= MAX_PRICE_RATIO,
            FinovaDefiError::ExtremePriceRatio
        );

        Ok(())
    }

    pub fn check_anti_rug_measures(&self) -> Result<()> {
        // Implement anti-rug pull measures
        // For example, check if creator locks some tokens
        // This could be implemented as a separate instruction
        
        // Minimum lock period for initial liquidity
        // Could be stored in pool state and enforced during withdrawals
        
        Ok(())
    }
}

// Additional helper for calculating optimal initial amounts
pub fn calculate_optimal_amounts(
    desired_token_a_amount: u64,
    desired_token_b_amount: u64,
    token_a_decimals: u8,
    token_b_decimals: u8,
) -> Result<(u64, u64)> {
    // Normalize amounts to same decimal precision
    let decimal_adjustment = if token_a_decimals > token_b_decimals {
        10_u64.pow((token_a_decimals - token_b_decimals) as u32)
    } else {
        10_u64.pow((token_b_decimals - token_a_decimals) as u32)
    };

    let (normalized_a, normalized_b) = if token_a_decimals > token_b_decimals {
        (desired_token_a_amount, desired_token_b_amount * decimal_adjustment)
    } else {
        (desired_token_a_amount * decimal_adjustment, desired_token_b_amount)
    };

    // Calculate optimal ratio
    let ratio = CurveCalculator::calculate_optimal_ratio(normalized_a, normalized_b)?;

    let optimal_a = (normalized_a as f64 * ratio) as u64;
    let optimal_b = (normalized_b as f64 * ratio) as u64;

    // Convert back to original decimals
    let (final_a, final_b) = if token_a_decimals > token_b_decimals {
        (optimal_a, optimal_b / decimal_adjustment)
    } else {
        (optimal_a / decimal_adjustment, optimal_b)
    };

    Ok((final_a, final_b))
}
