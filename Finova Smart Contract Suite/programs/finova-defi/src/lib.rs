// programs/finova-defi/src/lib.rs

use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Mint, Transfer};
use anchor_spl::associated_token::AssociatedToken;

// Import local modules
pub mod constants;
pub mod errors;
pub mod utils;
pub mod math;
pub mod state;
pub mod instructions;

use constants::*;
use errors::*;
use state::*;
use instructions::*;
use math::*;

declare_id!("FinovaDeFi11111111111111111111111111111111");

#[program]
pub mod finova_defi {
    use super::*;

    /// Initialize a new liquidity pool
    pub fn create_pool(
        ctx: Context<CreatePool>,
        fee_rate: u16,
        amp_factor: u64,
    ) -> Result<()> {
        instructions::create_pool::handler(ctx, fee_rate, amp_factor)
    }

    /// Add liquidity to an existing pool
    pub fn add_liquidity(
        ctx: Context<AddLiquidity>,
        amount_a_desired: u64,
        amount_b_desired: u64,
        amount_a_min: u64,
        amount_b_min: u64,
    ) -> Result<()> {
        instructions::add_liquidity::handler(
            ctx,
            amount_a_desired,
            amount_b_desired,
            amount_a_min,
            amount_b_min,
        )
    }

    /// Remove liquidity from a pool
    pub fn remove_liquidity(
        ctx: Context<RemoveLiquidity>,
        liquidity_amount: u64,
        amount_a_min: u64,
        amount_b_min: u64,
    ) -> Result<()> {
        instructions::remove_liquidity::handler(ctx, liquidity_amount, amount_a_min, amount_b_min)
    }

    /// Execute a token swap
    pub fn swap(
        ctx: Context<Swap>,
        amount_in: u64,
        amount_out_min: u64,
        is_a_to_b: bool,
    ) -> Result<()> {
        instructions::swap::handler(ctx, amount_in, amount_out_min, is_a_to_b)
    }

    /// Create a new yield farm
    pub fn create_farm(
        ctx: Context<CreateFarm>,
        reward_per_second: u64,
        start_time: i64,
        end_time: i64,
    ) -> Result<()> {
        instructions::yield_farm::create_farm_handler(ctx, reward_per_second, start_time, end_time)
    }

    /// Stake LP tokens in yield farm
    pub fn stake_farm(
        ctx: Context<StakeFarm>,
        amount: u64,
    ) -> Result<()> {
        instructions::yield_farm::stake_farm_handler(ctx, amount)
    }

    /// Unstake LP tokens from yield farm
    pub fn unstake_farm(
        ctx: Context<UnstakeFarm>,
        amount: u64,
    ) -> Result<()> {
        instructions::yield_farm::unstake_farm_handler(ctx, amount)
    }

    /// Harvest farm rewards
    pub fn harvest_farm(ctx: Context<HarvestFarm>) -> Result<()> {
        instructions::yield_farm::harvest_farm_handler(ctx)
    }

    /// Initialize flash loan
    pub fn flash_loan_begin(
        ctx: Context<FlashLoanBegin>,
        amount: u64,
    ) -> Result<()> {
        instructions::flash_loan::begin_handler(ctx, amount)
    }

    /// Complete flash loan with repayment
    pub fn flash_loan_end(
        ctx: Context<FlashLoanEnd>,
        amount_repaid: u64,
    ) -> Result<()> {
        instructions::flash_loan::end_handler(ctx, amount_repaid)
    }

    /// Update pool parameters (admin only)
    pub fn update_pool_config(
        ctx: Context<UpdatePoolConfig>,
        new_fee_rate: Option<u16>,
        new_amp_factor: Option<u64>,
    ) -> Result<()> {
        instructions::admin::update_pool_config_handler(ctx, new_fee_rate, new_amp_factor)
    }

    /// Emergency pause pool operations
    pub fn emergency_pause(
        ctx: Context<EmergencyPause>,
        pause_swaps: bool,
        pause_liquidity: bool,
    ) -> Result<()> {
        instructions::admin::emergency_pause_handler(ctx, pause_swaps, pause_liquidity)
    }

    /// Collect protocol fees
    pub fn collect_fees(ctx: Context<CollectFees>) -> Result<()> {
        instructions::admin::collect_fees_handler(ctx)
    }
}

// Context structs for all instructions
#[derive(Accounts)]
pub struct CreatePool<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    
    #[account(
        init,
        payer = authority,
        space = Pool::SIZE,
        seeds = [
            POOL_SEED,
            token_a_mint.key().as_ref(),
            token_b_mint.key().as_ref()
        ],
        bump
    )]
    pub pool: Account<'info, Pool>,
    
    pub token_a_mint: Account<'info, Mint>,
    pub token_b_mint: Account<'info, Mint>,
    
    #[account(
        init,
        payer = authority,
        mint::decimals = 6,
        mint::authority = pool,
        seeds = [
            LP_TOKEN_SEED,
            pool.key().as_ref()
        ],
        bump
    )]
    pub lp_token_mint: Account<'info, Mint>,
    
    #[account(
        init,
        payer = authority,
        token::mint = token_a_mint,
        token::authority = pool,
        seeds = [
            POOL_TOKEN_A_SEED,
            pool.key().as_ref()
        ],
        bump
    )]
    pub pool_token_a_account: Account<'info, TokenAccount>,
    
    #[account(
        init,
        payer = authority,
        token::mint = token_b_mint,
        token::authority = pool,
        seeds = [
            POOL_TOKEN_B_SEED,
            pool.key().as_ref()
        ],
        bump
    )]
    pub pool_token_b_account: Account<'info, TokenAccount>,
    
    #[account(
        init,
        payer = authority,
        space = PoolFees::SIZE,
        seeds = [
            POOL_FEES_SEED,
            pool.key().as_ref()
        ],
        bump
    )]
    pub pool_fees: Account<'info, PoolFees>,
    
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct AddLiquidity<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    
    #[account(
        mut,
        seeds = [
            POOL_SEED,
            pool.token_a_mint.as_ref(),
            pool.token_b_mint.as_ref()
        ],
        bump = pool.bump
    )]
    pub pool: Account<'info, Pool>,
    
    #[account(
        mut,
        seeds = [
            LP_TOKEN_SEED,
            pool.key().as_ref()
        ],
        bump = pool.lp_token_bump
    )]
    pub lp_token_mint: Account<'info, Mint>,
    
    #[account(
        mut,
        seeds = [
            POOL_TOKEN_A_SEED,
            pool.key().as_ref()
        ],
        bump = pool.token_a_bump
    )]
    pub pool_token_a_account: Account<'info, TokenAccount>,
    
    #[account(
        mut,
        seeds = [
            POOL_TOKEN_B_SEED,
            pool.key().as_ref()
        ],
        bump = pool.token_b_bump
    )]
    pub pool_token_b_account: Account<'info, TokenAccount>,
    
    #[account(
        mut,
        token::mint = pool.token_a_mint,
        token::authority = user
    )]
    pub user_token_a_account: Account<'info, TokenAccount>,
    
    #[account(
        mut,
        token::mint = pool.token_b_mint,
        token::authority = user
    )]
    pub user_token_b_account: Account<'info, TokenAccount>,
    
    #[account(
        init_if_needed,
        payer = user,
        associated_token::mint = lp_token_mint,
        associated_token::authority = user
    )]
    pub user_lp_token_account: Account<'info, TokenAccount>,
    
    #[account(
        init_if_needed,
        payer = user,
        space = LiquidityPosition::SIZE,
        seeds = [
            LIQUIDITY_POSITION_SEED,
            pool.key().as_ref(),
            user.key().as_ref()
        ],
        bump
    )]
    pub liquidity_position: Account<'info, LiquidityPosition>,
    
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct RemoveLiquidity<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    
    #[account(
        mut,
        seeds = [
            POOL_SEED,
            pool.token_a_mint.as_ref(),
            pool.token_b_mint.as_ref()
        ],
        bump = pool.bump
    )]
    pub pool: Account<'info, Pool>,
    
    #[account(
        mut,
        seeds = [
            LP_TOKEN_SEED,
            pool.key().as_ref()
        ],
        bump = pool.lp_token_bump
    )]
    pub lp_token_mint: Account<'info, Mint>,
    
    #[account(
        mut,
        seeds = [
            POOL_TOKEN_A_SEED,
            pool.key().as_ref()
        ],
        bump = pool.token_a_bump
    )]
    pub pool_token_a_account: Account<'info, TokenAccount>,
    
    #[account(
        mut,
        seeds = [
            POOL_TOKEN_B_SEED,
            pool.key().as_ref()
        ],
        bump = pool.token_b_bump
    )]
    pub pool_token_b_account: Account<'info, TokenAccount>,
    
    #[account(
        mut,
        token::mint = pool.token_a_mint,
        token::authority = user
    )]
    pub user_token_a_account: Account<'info, TokenAccount>,
    
    #[account(
        mut,
        token::mint = pool.token_b_mint,
        token::authority = user
    )]
    pub user_token_b_account: Account<'info, TokenAccount>,
    
    #[account(
        mut,
        associated_token::mint = lp_token_mint,
        associated_token::authority = user
    )]
    pub user_lp_token_account: Account<'info, TokenAccount>,
    
    #[account(
        mut,
        seeds = [
            LIQUIDITY_POSITION_SEED,
            pool.key().as_ref(),
            user.key().as_ref()
        ],
        bump = liquidity_position.bump
    )]
    pub liquidity_position: Account<'info, LiquidityPosition>,
    
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct Swap<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    
    #[account(
        mut,
        seeds = [
            POOL_SEED,
            pool.token_a_mint.as_ref(),
            pool.token_b_mint.as_ref()
        ],
        bump = pool.bump
    )]
    pub pool: Account<'info, Pool>,
    
    #[account(
        mut,
        seeds = [
            POOL_TOKEN_A_SEED,
            pool.key().as_ref()
        ],
        bump = pool.token_a_bump
    )]
    pub pool_token_a_account: Account<'info, TokenAccount>,
    
    #[account(
        mut,
        seeds = [
            POOL_TOKEN_B_SEED,
            pool.key().as_ref()
        ],
        bump = pool.token_b_bump
    )]
    pub pool_token_b_account: Account<'info, TokenAccount>,
    
    #[account(
        mut,
        constraint = user_source_account.mint == if is_a_to_b { pool.token_a_mint } else { pool.token_b_mint }
    )]
    pub user_source_account: Account<'info, TokenAccount>,
    
    #[account(
        mut,
        constraint = user_destination_account.mint == if is_a_to_b { pool.token_b_mint } else { pool.token_a_mint }
    )]
    pub user_destination_account: Account<'info, TokenAccount>,
    
    #[account(
        mut,
        seeds = [
            POOL_FEES_SEED,
            pool.key().as_ref()
        ],
        bump = pool.fees_bump
    )]
    pub pool_fees: Account<'info, PoolFees>,
    
    /// CHECK: Oracle account for price validation (if enabled)
    pub oracle_account: Option<UncheckedAccount<'info>>,
    
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct CreateFarm<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    
    #[account(
        constraint = pool.authority == authority.key()
    )]
    pub pool: Account<'info, Pool>,
    
    pub reward_mint: Account<'info, Mint>,
    
    #[account(
        init,
        payer = authority,
        space = Farm::SIZE,
        seeds = [
            FARM_SEED,
            pool.key().as_ref(),
            reward_mint.key().as_ref()
        ],
        bump
    )]
    pub farm: Account<'info, Farm>,
    
    #[account(
        init,
        payer = authority,
        token::mint = reward_mint,
        token::authority = farm,
        seeds = [
            FARM_REWARD_VAULT_SEED,
            farm.key().as_ref()
        ],
        bump
    )]
    pub reward_vault: Account<'info, TokenAccount>,
    
    #[account(
        mut,
        token::mint = reward_mint,
        token::authority = authority
    )]
    pub authority_reward_account: Account<'info, TokenAccount>,
    
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct StakeFarm<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    
    #[account(mut)]
    pub farm: Account<'info, Farm>,
    
    #[account(
        constraint = farm.pool == pool.key()
    )]
    pub pool: Account<'info, Pool>,
    
    #[account(
        mut,
        associated_token::mint = pool.lp_token_mint,
        associated_token::authority = user
    )]
    pub user_lp_token_account: Account<'info, TokenAccount>,
    
    #[account(
        init_if_needed,
        payer = user,
        space = FarmPosition::SIZE,
        seeds = [
            FARM_POSITION_SEED,
            farm.key().as_ref(),
            user.key().as_ref()
        ],
        bump
    )]
    pub farm_position: Account<'info, FarmPosition>,
    
    #[account(
        init,
        payer = user,
        token::mint = pool.lp_token_mint,
        token::authority = farm,
        seeds = [
            FARM_STAKE_VAULT_SEED,
            farm.key().as_ref(),
            user.key().as_ref()
        ],
        bump
    )]
    pub stake_vault: Account<'info, TokenAccount>,
    
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct UnstakeFarm<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    
    #[account(mut)]
    pub farm: Account<'info, Farm>,
    
    #[account(
        constraint = farm.pool == pool.key()
    )]
    pub pool: Account<'info, Pool>,
    
    #[account(
        mut,
        associated_token::mint = pool.lp_token_mint,
        associated_token::authority = user
    )]
    pub user_lp_token_account: Account<'info, TokenAccount>,
    
    #[account(
        mut,
        seeds = [
            FARM_POSITION_SEED,
            farm.key().as_ref(),
            user.key().as_ref()
        ],
        bump = farm_position.bump
    )]
    pub farm_position: Account<'info, FarmPosition>,
    
    #[account(
        mut,
        seeds = [
            FARM_STAKE_VAULT_SEED,
            farm.key().as_ref(),
            user.key().as_ref()
        ],
        bump = farm_position.stake_vault_bump
    )]
    pub stake_vault: Account<'info, TokenAccount>,
    
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct HarvestFarm<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    
    #[account(mut)]
    pub farm: Account<'info, Farm>,
    
    #[account(
        mut,
        seeds = [
            FARM_POSITION_SEED,
            farm.key().as_ref(),
            user.key().as_ref()
        ],
        bump = farm_position.bump
    )]
    pub farm_position: Account<'info, FarmPosition>,
    
    #[account(
        mut,
        seeds = [
            FARM_REWARD_VAULT_SEED,
            farm.key().as_ref()
        ],
        bump = farm.reward_vault_bump
    )]
    pub reward_vault: Account<'info, TokenAccount>,
    
    #[account(
        init_if_needed,
        payer = user,
        associated_token::mint = farm.reward_mint,
        associated_token::authority = user
    )]
    pub user_reward_account: Account<'info, TokenAccount>,
    
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct FlashLoanBegin<'info> {
    #[account(mut)]
    pub borrower: Signer<'info>,
    
    #[account(
        mut,
        seeds = [
            POOL_SEED,
            pool.token_a_mint.as_ref(),
            pool.token_b_mint.as_ref()
        ],
        bump = pool.bump
    )]
    pub pool: Account<'info, Pool>,
    
    #[account(
        mut,
        seeds = [
            POOL_TOKEN_A_SEED,
            pool.key().as_ref()
        ],
        bump = pool.token_a_bump
    )]
    pub pool_token_account: Account<'info, TokenAccount>,
    
    #[account(
        mut,
        token::mint = pool_token_account.mint,
        token::authority = borrower
    )]
    pub borrower_token_account: Account<'info, TokenAccount>,
    
    #[account(
        init,
        payer = borrower,
        space = FlashLoan::SIZE,
        seeds = [
            FLASH_LOAN_SEED,
            pool.key().as_ref(),
            borrower.key().as_ref()
        ],
        bump
    )]
    pub flash_loan: Account<'info, FlashLoan>,
    
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct FlashLoanEnd<'info> {
    #[account(mut)]
    pub borrower: Signer<'info>,
    
    #[account(
        mut,
        seeds = [
            POOL_SEED,
            pool.token_a_mint.as_ref(),
            pool.token_b_mint.as_ref()
        ],
        bump = pool.bump
    )]
    pub pool: Account<'info, Pool>,
    
    #[account(
        mut,
        seeds = [
            POOL_TOKEN_A_SEED,
            pool.key().as_ref()
        ],
        bump = pool.token_a_bump
    )]
    pub pool_token_account: Account<'info, TokenAccount>,
    
    #[account(
        mut,
        token::mint = pool_token_account.mint,
        token::authority = borrower
    )]
    pub borrower_token_account: Account<'info, TokenAccount>,
    
    #[account(
        mut,
        close = borrower,
        seeds = [
            FLASH_LOAN_SEED,
            pool.key().as_ref(),
            borrower.key().as_ref()
        ],
        bump = flash_loan.bump
    )]
    pub flash_loan: Account<'info, FlashLoan>,
    
    #[account(
        mut,
        seeds = [
            POOL_FEES_SEED,
            pool.key().as_ref()
        ],
        bump = pool.fees_bump
    )]
    pub pool_fees: Account<'info, PoolFees>,
    
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct UpdatePoolConfig<'info> {
    #[account(
        mut,
        constraint = authority.key() == pool.authority
    )]
    pub authority: Signer<'info>,
    
    #[account(mut)]
    pub pool: Account<'info, Pool>,
}

#[derive(Accounts)]
pub struct EmergencyPause<'info> {
    #[account(
        constraint = authority.key() == pool.authority
    )]
    pub authority: Signer<'info>,
    
    #[account(mut)]
    pub pool: Account<'info, Pool>,
}

#[derive(Accounts)]
pub struct CollectFees<'info> {
    #[account(
        constraint = authority.key() == pool.authority
    )]
    pub authority: Signer<'info>,
    
    pub pool: Account<'info, Pool>,
    
    #[account(
        mut,
        seeds = [
            POOL_FEES_SEED,
            pool.key().as_ref()
        ],
        bump = pool.fees_bump
    )]
    pub pool_fees: Account<'info, PoolFees>,
    
    #[account(
        mut,
        seeds = [
            POOL_TOKEN_A_SEED,
            pool.key().as_ref()
        ],
        bump = pool.token_a_bump
    )]
    pub pool_token_a_account: Account<'info, TokenAccount>,
    
    #[account(
        mut,
        seeds = [
            POOL_TOKEN_B_SEED,
            pool.key().as_ref()
        ],
        bump = pool.token_b_bump
    )]
    pub pool_token_b_account: Account<'info, TokenAccount>,
    
    #[account(
        mut,
        token::mint = pool.token_a_mint,
        token::authority = authority
    )]
    pub authority_token_a_account: Account<'info, TokenAccount>,
    
    #[account(
        mut,
        token::mint = pool.token_b_mint,
        token::authority = authority
    )]
    pub authority_token_b_account: Account<'info, TokenAccount>,
    
    pub token_program: Program<'info, Token>,
}

// Helper function to perform CPI calls
pub fn transfer_tokens<'info>(
    from: &Account<'info, TokenAccount>,
    to: &Account<'info, TokenAccount>,
    authority: &Signer<'info>,
    token_program: &Program<'info, Token>,
    amount: u64,
) -> Result<()> {
    let cpi_accounts = Transfer {
        from: from.to_account_info(),
        to: to.to_account_info(),
        authority: authority.to_account_info(),
    };
    let cpi_program = token_program.to_account_info();
    let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
    
    token::transfer(cpi_ctx, amount)
}

// Additional validation context for the swap instruction
#[derive(Accounts)]
#[instruction(amount_in: u64, amount_out_min: u64, is_a_to_b: bool)]
pub struct Swap<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    
    #[account(
        mut,
        seeds = [
            POOL_SEED,
            pool.token_a_mint.as_ref(),
            pool.token_b_mint.as_ref()
        ],
        bump = pool.bump,
        constraint = !pool.is_paused @ FinovaDeFiError::PoolPaused
    )]
    pub pool: Account<'info, Pool>,
    
    #[account(
        mut,
        seeds = [
            POOL_TOKEN_A_SEED,
            pool.key().as_ref()
        ],
        bump = pool.token_a_bump
    )]
    pub pool_token_a_account: Account<'info, TokenAccount>,
    
    #[account(
        mut,
        seeds = [
            POOL_TOKEN_B_SEED,
            pool.key().as_ref()
        ],
        bump = pool.token_b_bump
    )]
    pub pool_token_b_account: Account<'info, TokenAccount>,
    
    #[account(
        mut,
        constraint = user_source_account.mint == if is_a_to_b { pool.token_a_mint } else { pool.token_b_mint },
        constraint = user_source_account.owner == user.key(),
        constraint = user_source_account.amount >= amount_in @ FinovaDeFiError::InsufficientBalance
    )]
    pub user_source_account: Account<'info, TokenAccount>,
    
    #[account(
        mut,
        constraint = user_destination_account.mint == if is_a_to_b { pool.token_b_mint } else { pool.token_a_mint },
        constraint = user_destination_account.owner == user.key()
    )]
    pub user_destination_account: Account<'info, TokenAccount>,
    
    #[account(
        mut,
        seeds = [
            POOL_FEES_SEED,
            pool.key().as_ref()
        ],
        bump = pool.fees_bump
    )]
    pub pool_fees: Account<'info, PoolFees>,
    
    pub token_program: Program<'info, Token>,
}

#[event]
pub struct PoolCreated {
    pub pool: Pubkey,
    pub token_a_mint: Pubkey,
    pub token_b_mint: Pubkey,
    pub lp_token_mint: Pubkey,
    pub fee_rate: u16,
    pub amp_factor: u64,
    pub timestamp: i64,
}

#[event]
pub struct LiquidityAdded {
    pub pool: Pubkey,
    pub user: Pubkey,
    pub amount_a: u64,
    pub amount_b: u64,
    pub lp_tokens_minted: u64,
    pub timestamp: i64,
}

#[event]
pub struct LiquidityRemoved {
    pub pool: Pubkey,
    pub user: Pubkey,
    pub lp_tokens_burned: u64,
    pub amount_a: u64,
    pub amount_b: u64,
    pub timestamp: i64,
}

#[event]
pub struct SwapExecuted {
    pub pool: Pubkey,
    pub user: Pubkey,
    pub amount_in: u64,
    pub amount_out: u64,
    pub fee_amount: u64,
    pub is_a_to_b: bool,
    pub timestamp: i64,
}

#[event]
pub struct FarmCreated {
    pub farm: Pubkey,
    pub pool: Pubkey,
    pub reward_mint: Pubkey,
    pub reward_per_second: u64,
    pub start_time: i64,
    pub end_time: i64,
}

#[event]
pub struct FarmStaked {
    pub farm: Pubkey,
    pub user: Pubkey,
    pub amount: u64,
    pub timestamp: i64,
}

#[event]
pub struct FarmUnstaked {
    pub farm: Pubkey,
    pub user: Pubkey,
    pub amount: u64,
    pub rewards_harvested: u64,
    pub timestamp: i64,
}

#[event]
pub struct FlashLoanExecuted {
    pub pool: Pubkey,
    pub borrower: Pubkey,
    pub amount: u64,
    pub fee: u64,
    pub timestamp: i64,
}

// Access control checks
impl<'info> CreatePool<'info> {
    pub fn validate(&self) -> Result<()> {
        require!(
            self.token_a_mint.key() != self.token_b_mint.key(),
            FinovaDeFiError::IdenticalTokens
        );
        Ok(())
    }
}

impl<'info> AddLiquidity<'info> {
    pub fn validate(&self, amount_a_desired: u64, amount_b_desired: u64) -> Result<()> {
        require!(
            !self.pool.is_paused,
            FinovaDeFiError::PoolPaused
        );
        require!(
            amount_a_desired > 0 && amount_b_desired > 0,
            FinovaDeFiError::InvalidAmount
        );
        require!(
            self.user_token_a_account.amount >= amount_a_desired,
            FinovaDeFiError::InsufficientBalance
        );
        require!(
            self.user_token_b_account.amount >= amount_b_desired,
            FinovaDeFiError::InsufficientBalance
        );
        Ok(())
    }
}

impl<'info> RemoveLiquidity<'info> {
    pub fn validate(&self, liquidity_amount: u64) -> Result<()> {
        require!(
            !self.pool.is_paused,
            FinovaDeFiError::PoolPaused
        );
        require!(
            liquidity_amount > 0,
            FinovaDeFiError::InvalidAmount
        );
        require!(
            self.user_lp_token_account.amount >= liquidity_amount,
            FinovaDeFiError::InsufficientBalance
        );
        Ok(())
    }
}

impl<'info> Swap<'info> {
    pub fn validate(&self, amount_in: u64, amount_out_min: u64) -> Result<()> {
        require!(
            !self.pool.is_paused,
            FinovaDeFiError::PoolPaused
        );
        require!(
            amount_in > 0,
            FinovaDeFiError::InvalidAmount
        );
        require!(
            amount_out_min > 0,
            FinovaDeFiError::InvalidAmount
        );
        Ok(())
    }
}

impl<'info> FlashLoanBegin<'info> {
    pub fn validate(&self, amount: u64) -> Result<()> {
        require!(
            !self.pool.is_paused,
            FinovaDeFiError::PoolPaused
        );
        require!(
            amount > 0,
            FinovaDeFiError::InvalidAmount
        );
        require!(
            self.pool_token_account.amount >= amount,
            FinovaDeFiError::InsufficientLiquidity
        );
        Ok(())
    }
}

impl<'info> FlashLoanEnd<'info> {
    pub fn validate(&self, amount_repaid: u64) -> Result<()> {
        let required_repayment = self.flash_loan.amount
            .checked_add(
                self.flash_loan.amount
                    .checked_mul(self.pool.flash_loan_fee_rate as u64)
                    .unwrap()
                    .checked_div(10000)
                    .unwrap()
            )
            .unwrap();

        require!(
            amount_repaid >= required_repayment,
            FinovaDeFiError::InsufficientRepayment
        );
        require!(
            self.borrower_token_account.amount >= amount_repaid,
            FinovaDeFiError::InsufficientBalance
        );
        Ok(())
    }
}

// Math utilities for DeFi calculations
pub mod defi_math {
    use super::*;

    /// Calculate LP tokens to mint for initial liquidity
    pub fn calculate_initial_lp_tokens(amount_a: u64, amount_b: u64) -> Result<u64> {
        let product = (amount_a as u128)
            .checked_mul(amount_b as u128)
            .ok_or(FinovaDeFiError::MathOverflow)?;
        
        let sqrt_result = integer_sqrt(product);
        let lp_tokens = sqrt_result
            .checked_sub(1000) // Minimum liquidity lock
            .ok_or(FinovaDeFiError::InsufficientLiquidityMinted)?;
        
        Ok(lp_tokens as u64)
    }

    /// Calculate LP tokens to mint for additional liquidity
    pub fn calculate_lp_tokens_for_deposit(
        amount_a: u64,
        amount_b: u64,
        reserve_a: u64,
        reserve_b: u64,
        total_supply: u64,
    ) -> Result<u64> {
        if total_supply == 0 {
            return calculate_initial_lp_tokens(amount_a, amount_b);
        }

        let lp_a = (amount_a as u128)
            .checked_mul(total_supply as u128)
            .ok_or(FinovaDeFiError::MathOverflow)?
            .checked_div(reserve_a as u128)
            .ok_or(FinovaDeFiError::MathOverflow)?;

        let lp_b = (amount_b as u128)
            .checked_mul(total_supply as u128)
            .ok_or(FinovaDeFiError::MathOverflow)?
            .checked_div(reserve_b as u128)
            .ok_or(FinovaDeFiError::MathOverflow)?;

        Ok(std::cmp::min(lp_a, lp_b) as u64)
    }

    /// Calculate output amount for swap using constant product formula
    pub fn calculate_swap_output(
        amount_in: u64,
        reserve_in: u64,
        reserve_out: u64,
        fee_rate: u16,
    ) -> Result<u64> {
        let fee_adjusted_input = amount_in
            .checked_mul(10000u64.checked_sub(fee_rate as u64).unwrap())
            .ok_or(FinovaDeFiError::MathOverflow)?
            .checked_div(10000)
            .ok_or(FinovaDeFiError::MathOverflow)?;

        let numerator = (fee_adjusted_input as u128)
            .checked_mul(reserve_out as u128)
            .ok_or(FinovaDeFiError::MathOverflow)?;

        let denominator = (reserve_in as u128)
            .checked_add(fee_adjusted_input as u128)
            .ok_or(FinovaDeFiError::MathOverflow)?;

        let amount_out = numerator
            .checked_div(denominator)
            .ok_or(FinovaDeFiError::MathOverflow)? as u64;

        require!(amount_out > 0, FinovaDeFiError::InsufficientOutputAmount);
        Ok(amount_out)
    }

    /// Calculate stable swap output using StableSwap invariant
    pub fn calculate_stable_swap_output(
        amount_in: u64,
        reserve_in: u64,
        reserve_out: u64,
        amp_factor: u64,
        fee_rate: u16,
    ) -> Result<u64> {
        let fee_adjusted_input = amount_in
            .checked_mul(10000u64.checked_sub(fee_rate as u64).unwrap())
            .ok_or(FinovaDeFiError::MathOverflow)?
            .checked_div(10000)
            .ok_or(FinovaDeFiError::MathOverflow)?;

        // Simplified StableSwap calculation for similar-valued assets
        let sum = reserve_in.checked_add(reserve_out).unwrap();
        let product = (reserve_in as u128).checked_mul(reserve_out as u128).unwrap();
        
        let d = calculate_d(sum, product, amp_factor)?;
        let new_reserve_in = reserve_in.checked_add(fee_adjusted_input).unwrap();
        let new_reserve_out = get_y(new_reserve_in, d, amp_factor)?;
        
        let amount_out = reserve_out.checked_sub(new_reserve_out).unwrap();
        require!(amount_out > 0, FinovaDeFiError::InsufficientOutputAmount);
        
        Ok(amount_out)
    }

    /// Calculate D parameter for StableSwap
    fn calculate_d(sum: u64, product: u128, amp_factor: u64) -> Result<u64> {
        if sum == 0 {
            return Ok(0);
        }

        let mut d_prev = 0u64;
        let mut d = sum;
        let ann = amp_factor.checked_mul(2).unwrap(); // N = 2 for two-token pools

        for _ in 0..255 {
            let mut d_p = d as u128;
            d_p = d_p.checked_mul(d as u128).unwrap().checked_div(product).unwrap();
            d_p = d_p.checked_mul(d as u128).unwrap().checked_div(4).unwrap();

            d_prev = d;
            let numerator = (ann as u128)
                .checked_mul(sum as u128)
                .unwrap()
                .checked_add(d_p.checked_mul(2).unwrap())
                .unwrap()
                .checked_mul(d as u128)
                .unwrap();

            let denominator = (ann as u128)
                .checked_sub(1)
                .unwrap()
                .checked_mul(d as u128)
                .unwrap()
                .checked_add(d_p.checked_mul(3).unwrap())
                .unwrap();

            d = (numerator.checked_div(denominator).unwrap()) as u64;

            if d > d_prev {
                if d.checked_sub(d_prev).unwrap() <= 1 {
                    break;
                }
            } else if d_prev.checked_sub(d).unwrap() <= 1 {
                break;
            }
        }

        Ok(d)
    }

    /// Calculate Y parameter for StableSwap
    fn get_y(x: u64, d: u64, amp_factor: u64) -> Result<u64> {
        let ann = amp_factor.checked_mul(2).unwrap();
        let c = (d as u128)
            .checked_mul(d as u128)
            .unwrap()
            .checked_div(x as u128)
            .unwrap()
            .checked_mul(d as u128)
            .unwrap()
            .checked_div(ann as u128)
            .unwrap()
            .checked_div(4)
            .unwrap();

        let b = (x as u128).checked_add(
            (d as u128).checked_div(ann as u128).unwrap()
        ).unwrap();

        let mut y_prev = 0u64;
        let mut y = d;

        for _ in 0..255 {
            y_prev = y;
            let y_numerator = (y as u128)
                .checked_mul(y as u128)
                .unwrap()
                .checked_add(c)
                .unwrap();
            let y_denominator = (y as u128)
                .checked_mul(2)
                .unwrap()
                .checked_add(b)
                .unwrap()
                .checked_sub(d as u128)
                .unwrap();

            y = (y_numerator.checked_div(y_denominator).unwrap()) as u64;

            if y > y_prev {
                if y.checked_sub(y_prev).unwrap() <= 1 {
                    break;
                }
            } else if y_prev.checked_sub(y).unwrap() <= 1 {
                break;
            }
        }

        Ok(y)
    }

    /// Calculate farm rewards for a user
    pub fn calculate_farm_rewards(
        staked_amount: u64,
        reward_per_second: u64,
        time_elapsed: i64,
        total_staked: u64,
    ) -> Result<u64> {
        if total_staked == 0 || staked_amount == 0 || time_elapsed <= 0 {
            return Ok(0);
        }

        let total_rewards = (reward_per_second as u128)
            .checked_mul(time_elapsed as u128)
            .ok_or(FinovaDeFiError::MathOverflow)?;

        let user_share = total_rewards
            .checked_mul(staked_amount as u128)
            .ok_or(FinovaDeFiError::MathOverflow)?
            .checked_div(total_staked as u128)
            .ok_or(FinovaDeFiError::MathOverflow)?;

        Ok(user_share as u64)
    }

    /// Calculate flash loan fee
    pub fn calculate_flash_loan_fee(amount: u64, fee_rate: u16) -> Result<u64> {
        amount
            .checked_mul(fee_rate as u64)
            .ok_or(FinovaDeFiError::MathOverflow)?
            .checked_div(10000)
            .ok_or(FinovaDeFiError::MathOverflow)
    }

    /// Integer square root implementation
    fn integer_sqrt(value: u128) -> u128 {
        if value == 0 {
            return 0;
        }

        let mut z = value;
        let mut x = value / 2 + 1;
        
        while x < z {
            z = x;
            x = (value / x + x) / 2;
        }
        
        z
    }

    /// Check if price impact is within acceptable limits
    pub fn check_price_impact(
        amount_in: u64,
        amount_out: u64,
        reserve_in: u64,
        reserve_out: u64,
        max_impact_bps: u16, // basis points (e.g., 300 = 3%)
    ) -> Result<()> {
        let expected_out = (amount_in as u128)
            .checked_mul(reserve_out as u128)
            .unwrap()
            .checked_div(reserve_in as u128)
            .unwrap() as u64;

        if amount_out < expected_out {
            let impact = expected_out
                .checked_sub(amount_out)
                .unwrap()
                .checked_mul(10000)
                .unwrap()
                .checked_div(expected_out)
                .unwrap();

            require!(
                impact <= max_impact_bps as u64,
                FinovaDeFiError::ExcessivePriceImpact
            );
        }

        Ok(())
    }
}

// Oracle integration utilities
pub mod oracle_utils {
    use super::*;

    /// Validate price from oracle feed
    pub fn validate_oracle_price(
        oracle_account: &UncheckedAccount,
        expected_price_range: (u64, u64), // (min, max)
        max_staleness: i64, // seconds
    ) -> Result<u64> {
        // This would integrate with Pyth, Switchboard, or other oracle networks
        // For now, we'll implement a basic validation structure
        
        let clock = Clock::get()?;
        
        // Parse oracle data (implementation depends on oracle provider)
        // This is a simplified example - real implementation would parse
        // the actual oracle account data format
        
        // Placeholder validation logic
        require!(
            oracle_account.data_is_empty() == false,
            FinovaDeFiError::InvalidOracle
        );

        // Return a mock price for now - real implementation would
        // parse the oracle data and return the actual price
        let mock_price = (expected_price_range.0 + expected_price_range.1) / 2;
        
        Ok(mock_price)
    }

    /// Calculate Time-Weighted Average Price (TWAP)
    pub fn calculate_twap(
        price_history: &[(u64, i64)], // (price, timestamp) pairs
        window_seconds: i64,
    ) -> Result<u64> {
        let current_time = Clock::get()?.unix_timestamp;
        let cutoff_time = current_time.checked_sub(window_seconds).unwrap();

        let mut total_weighted_price = 0u128;
        let mut total_weight = 0u128;

        for window in price_history.windows(2) {
            if let [prev, curr] = window {
                if curr.1 > cutoff_time {
                    let weight = (curr.1 - std::cmp::max(prev.1, cutoff_time)) as u128;
                    total_weighted_price = total_weighted_price
                        .checked_add(prev.0 as u128 * weight)
                        .unwrap();
                    total_weight = total_weight.checked_add(weight).unwrap();
                }
            }
        }

        if total_weight == 0 {
            return Err(FinovaDeFiError::InsufficientPriceData.into());
        }

        Ok((total_weighted_price / total_weight) as u64)
    }
}

// Security utilities
pub mod security {
    use super::*;

    /// Check for reentrancy attacks
    pub fn check_reentrancy(pool: &Account<Pool>) -> Result<()> {
        require!(!pool.locked, FinovaDeFiError::ReentrancyDetected);
        Ok(())
    }

    /// Validate slippage protection
    pub fn validate_slippage(
        expected_amount: u64,
        actual_amount: u64,
        max_slippage_bps: u16,
    ) -> Result<()> {
        if actual_amount < expected_amount {
            let slippage = expected_amount
                .checked_sub(actual_amount)
                .unwrap()
                .checked_mul(10000)
                .unwrap()
                .checked_div(expected_amount)
                .unwrap();

            require!(
                slippage <= max_slippage_bps as u64,
                FinovaDeFiError::SlippageExceeded
            );
        }

        Ok(())
    }

    /// Rate limiting for flash loans
    pub fn check_flash_loan_rate_limit(
        user: &Pubkey,
        amount: u64,
        rate_limit_per_hour: u64,
    ) -> Result<()> {
        // This would check against a rate limiting storage
        // For now, we implement basic validation
        require!(
            amount <= rate_limit_per_hour,
            FinovaDeFiError::RateLimitExceeded
        );
        Ok(())
    }
}

// Integration helpers for Finova ecosystem
pub mod finova_integration {
    use super::*;
    use finova_core::state::User as FinovaUser;

    /// Apply Finova Network bonuses to DeFi rewards
    pub fn apply_finova_bonuses(
        base_rewards: u64,
        user_account: &Account<FinovaUser>,
    ) -> Result<u64> {
        let xp_multiplier = calculate_xp_bonus(user_account.xp_level);
        let rp_multiplier = calculate_rp_bonus(user_account.rp_tier);
        let staking_multiplier = calculate_staking_bonus(user_account.staked_amount);

        let total_multiplier = (10000u64)
            .checked_add(xp_multiplier)
            .unwrap()
            .checked_add(rp_multiplier)
            .unwrap()
            .checked_add(staking_multiplier)
            .unwrap();

        let enhanced_rewards = base_rewards
            .checked_mul(total_multiplier)
            .unwrap()
            .checked_div(10000)
            .unwrap();

        Ok(enhanced_rewards)
    }

    fn calculate_xp_bonus(xp_level: u32) -> u64 {
        std::cmp::min(xp_level as u64 * 10, 500) // Max 5% bonus from XP
    }

    fn calculate_rp_bonus(rp_tier: u8) -> u64 {
        match rp_tier {
            0 => 0,    // Explorer: 0%
            1 => 200,  // Connector: 2%
            2 => 500,  // Influencer: 5%
            3 => 1000, // Leader: 10%
            4 => 2000, // Ambassador: 20%
            _ => 0,
        }
    }

    fn calculate_staking_bonus(staked_amount: u64) -> u64 {
        if staked_amount >= 10000 * 1_000_000 { // 10K+ FIN
            1500 // 15%
        } else if staked_amount >= 5000 * 1_000_000 { // 5K+ FIN
            1000 // 10%
        } else if staked_amount >= 1000 * 1_000_000 { // 1K+ FIN
            500 // 5%
        } else if staked_amount >= 500 * 1_000_000 { // 500+ FIN
            250 // 2.5%
        } else {
            0
        }
    }
}

// Tests
#[cfg(test)]
mod tests {
    use super::*;
    use super::defi_math::*;

    #[test]
    fn test_swap_calculation() {
        let amount_in = 1000;
        let reserve_in = 100000;
        let reserve_out = 50000;
        let fee_rate = 30; // 0.3%

        let result = calculate_swap_output(amount_in, reserve_in, reserve_out, fee_rate);
        assert!(result.is_ok());
        
        let amount_out = result.unwrap();
        assert!(amount_out > 0);
        assert!(amount_out < reserve_out);
    }

    #[test]
    fn test_lp_token_calculation() {
        let amount_a = 1000;
        let amount_b = 2000;
        
        let result = calculate_initial_lp_tokens(amount_a, amount_b);
        assert!(result.is_ok());
        
        let lp_tokens = result.unwrap();
        assert!(lp_tokens > 0);
    }

    #[test]
    fn test_farm_rewards_calculation() {
        let staked_amount = 1000;
        let reward_per_second = 10;
        let time_elapsed = 3600; // 1 hour
        let total_staked = 10000;

        let result = calculate_farm_rewards(
            staked_amount,
            reward_per_second,
            time_elapsed,
            total_staked,
        );
        assert!(result.is_ok());
        
        let rewards = result.unwrap();
        assert_eq!(rewards, 3600); // 10% of total rewards for 1 hour
    }

    #[test]
    fn test_flash_loan_fee() {
        let amount = 10000;
        let fee_rate = 9; // 0.09%

        let result = calculate_flash_loan_fee(amount, fee_rate);
        assert!(result.is_ok());
        
        let fee = result.unwrap();
        assert_eq!(fee, 9); // 0.09% of 10000
    }
}
