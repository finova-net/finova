// programs/finova-token/src/lib.rs

use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount, Transfer, MintTo, Burn};
use std::mem::size_of;

declare_id!("FinTokenProgramId11111111111111111111111111");

pub mod instructions;
pub mod state;
pub mod events;
pub mod errors;
pub mod utils;

use instructions::*;
use state::*;
use events::*;
use errors::*;

/// Finova Token Program - Handles $FIN token operations, staking, and rewards
#[program]
pub mod finova_token {
    use super::*;

    /// Initialize the token mint and program state
    pub fn initialize_mint(
        ctx: Context<InitializeMint>,
        decimals: u8,
        max_supply: u64,
        initial_supply: u64,
        mint_authority_bump: u8,
    ) -> Result<()> {
        let mint_info = &mut ctx.accounts.mint_info;
        mint_info.mint = ctx.accounts.mint.key();
        mint_info.mint_authority = ctx.accounts.mint_authority.key();
        mint_info.decimals = decimals;
        mint_info.max_supply = max_supply;
        mint_info.current_supply = initial_supply;
        mint_info.mint_authority_bump = mint_authority_bump;
        mint_info.is_paused = false;
        mint_info.created_at = Clock::get()?.unix_timestamp;
        mint_info.updated_at = Clock::get()?.unix_timestamp;

        // Initialize reward pool
        let reward_pool = &mut ctx.accounts.reward_pool;
        reward_pool.total_staked = 0;
        reward_pool.total_rewards = 0;
        reward_pool.reward_rate = 800; // 8% APY in basis points
        reward_pool.last_update_time = Clock::get()?.unix_timestamp;
        reward_pool.created_at = Clock::get()?.unix_timestamp;

        // Mint initial supply if specified
        if initial_supply > 0 {
            let authority_seeds = &[
                b"mint_authority",
                &[mint_authority_bump],
            ];
            let signer = &[&authority_seeds[..]];

            token::mint_to(
                CpiContext::new_with_signer(
                    ctx.accounts.token_program.to_account_info(),
                    MintTo {
                        mint: ctx.accounts.mint.to_account_info(),
                        to: ctx.accounts.treasury_account.to_account_info(),
                        authority: ctx.accounts.mint_authority.to_account_info(),
                    },
                    signer,
                ),
                initial_supply,
            )?;
        }

        emit!(MintInitialized {
            mint: ctx.accounts.mint.key(),
            max_supply,
            initial_supply,
            decimals,
            timestamp: Clock::get()?.unix_timestamp,
        });

        Ok(())
    }

    /// Mint new tokens (restricted to authorized minters)
    pub fn mint_tokens(
        ctx: Context<MintTokens>,
        amount: u64,
        recipient: Pubkey,
    ) -> Result<()> {
        let mint_info = &mut ctx.accounts.mint_info;
        
        // Check if minting is paused
        require!(!mint_info.is_paused, FinovaTokenError::MintingPaused);
        
        // Check supply limits
        let new_supply = mint_info.current_supply
            .checked_add(amount)
            .ok_or(FinovaTokenError::MathOverflow)?;
        
        require!(
            new_supply <= mint_info.max_supply,
            FinovaTokenError::MaxSupplyExceeded
        );

        // Verify minter authorization
        require!(
            ctx.accounts.minter.key() == mint_info.mint_authority ||
            ctx.accounts.authorized_minters.minters.contains(&ctx.accounts.minter.key()),
            FinovaTokenError::UnauthorizedMinter
        );

        // Update supply tracking
        mint_info.current_supply = new_supply;
        mint_info.updated_at = Clock::get()?.unix_timestamp;

        // Mint tokens
        let authority_seeds = &[
            b"mint_authority",
            &[mint_info.mint_authority_bump],
        ];
        let signer = &[&authority_seeds[..]];

        token::mint_to(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                MintTo {
                    mint: ctx.accounts.mint.to_account_info(),
                    to: ctx.accounts.recipient_account.to_account_info(),
                    authority: ctx.accounts.mint_authority.to_account_info(),
                },
                signer,
            ),
            amount,
        )?;

        emit!(TokensMinted {
            mint: ctx.accounts.mint.key(),
            recipient,
            amount,
            minter: ctx.accounts.minter.key(),
            new_supply: new_supply,
            timestamp: Clock::get()?.unix_timestamp,
        });

        Ok(())
    }

    /// Burn tokens to reduce supply
    pub fn burn_tokens(
        ctx: Context<BurnTokens>,
        amount: u64,
    ) -> Result<()> {
        let mint_info = &mut ctx.accounts.mint_info;
        
        // Update supply tracking
        mint_info.current_supply = mint_info.current_supply
            .checked_sub(amount)
            .ok_or(FinovaTokenError::InsufficientSupply)?;
        
        mint_info.updated_at = Clock::get()?.unix_timestamp;

        // Burn tokens
        token::burn(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Burn {
                    mint: ctx.accounts.mint.to_account_info(),
                    from: ctx.accounts.user_token_account.to_account_info(),
                    authority: ctx.accounts.user.to_account_info(),
                },
            ),
            amount,
        )?;

        emit!(TokensBurned {
            mint: ctx.accounts.mint.key(),
            user: ctx.accounts.user.key(),
            amount,
            new_supply: mint_info.current_supply,
            timestamp: Clock::get()?.unix_timestamp,
        });

        Ok(())
    }

    /// Stake tokens to earn rewards and enhanced benefits
    pub fn stake_tokens(
        ctx: Context<StakeTokens>,
        amount: u64,
        lock_duration: u64, // in seconds
    ) -> Result<()> {
        let stake_account = &mut ctx.accounts.stake_account;
        let reward_pool = &mut ctx.accounts.reward_pool;
        let current_time = Clock::get()?.unix_timestamp;

        // Validate staking parameters
        require!(amount > 0, FinovaTokenError::InvalidAmount);
        require!(
            lock_duration >= 86400, // minimum 1 day
            FinovaTokenError::InvalidLockDuration
        );
        require!(
            lock_duration <= 31536000, // maximum 1 year
            FinovaTokenError::InvalidLockDuration
        );

        // Update reward pool before modifying stakes
        update_reward_pool(reward_pool, current_time)?;

        // Calculate rewards per token at stake time
        let reward_per_token_stored = if reward_pool.total_staked == 0 {
            0
        } else {
            reward_pool.reward_per_token_stored +
            ((current_time - reward_pool.last_update_time) as u64 *
             reward_pool.reward_rate *
             1_000_000) / reward_pool.total_staked
        };

        // Initialize or update stake account
        if stake_account.user == Pubkey::default() {
            stake_account.user = ctx.accounts.user.key();
            stake_account.staked_amount = amount;
            stake_account.reward_per_token_paid = reward_per_token_stored;
            stake_account.pending_rewards = 0;
            stake_account.created_at = current_time;
        } else {
            // Calculate and add pending rewards
            let earned_rewards = calculate_earned_rewards(
                stake_account.staked_amount,
                reward_per_token_stored,
                stake_account.reward_per_token_paid,
                stake_account.pending_rewards,
            )?;
            
            stake_account.pending_rewards = earned_rewards;
            stake_account.reward_per_token_paid = reward_per_token_stored;
            stake_account.staked_amount = stake_account.staked_amount
                .checked_add(amount)
                .ok_or(FinovaTokenError::MathOverflow)?;
        }

        // Set lock period with bonus calculation
        let lock_bonus_multiplier = calculate_lock_bonus(lock_duration);
        stake_account.lock_end_time = current_time + lock_duration as i64;
        stake_account.lock_bonus_multiplier = lock_bonus_multiplier;
        stake_account.updated_at = current_time;

        // Update reward pool
        reward_pool.total_staked = reward_pool.total_staked
            .checked_add(amount)
            .ok_or(FinovaTokenError::MathOverflow)?;
        reward_pool.reward_per_token_stored = reward_per_token_stored;
        reward_pool.last_update_time = current_time;

        // Transfer tokens to staking vault
        token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.user_token_account.to_account_info(),
                    to: ctx.accounts.staking_vault.to_account_info(),
                    authority: ctx.accounts.user.to_account_info(),
                },
            ),
            amount,
        )?;

        emit!(TokensStaked {
            user: ctx.accounts.user.key(),
            amount,
            lock_end_time: stake_account.lock_end_time,
            lock_bonus_multiplier,
            total_staked: stake_account.staked_amount,
            timestamp: current_time,
        });

        Ok(())
    }

    /// Unstake tokens (subject to lock period)
    pub fn unstake_tokens(
        ctx: Context<UnstakeTokens>,
        amount: u64,
    ) -> Result<()> {
        let stake_account = &mut ctx.accounts.stake_account;
        let reward_pool = &mut ctx.accounts.reward_pool;
        let current_time = Clock::get()?.unix_timestamp;

        // Validate unstaking parameters
        require!(amount > 0, FinovaTokenError::InvalidAmount);
        require!(
            amount <= stake_account.staked_amount,
            FinovaTokenError::InsufficientStakedAmount
        );

        // Check lock period (with early withdrawal penalty)
        let mut penalty_amount = 0u64;
        if current_time < stake_account.lock_end_time {
            // Calculate early withdrawal penalty (10% of unstaked amount)
            penalty_amount = amount
                .checked_mul(1000)
                .ok_or(FinovaTokenError::MathOverflow)?
                .checked_div(10000)
                .ok_or(FinovaTokenError::MathOverflow)?;
        }

        // Update reward pool
        update_reward_pool(reward_pool, current_time)?;

        // Calculate final rewards
        let reward_per_token_stored = reward_pool.reward_per_token_stored;
        let earned_rewards = calculate_earned_rewards(
            stake_account.staked_amount,
            reward_per_token_stored,
            stake_account.reward_per_token_paid,
            stake_account.pending_rewards,
        )?;

        // Update stake account
        stake_account.staked_amount = stake_account.staked_amount
            .checked_sub(amount)
            .ok_or(FinovaTokenError::InsufficientStakedAmount)?;
        
        stake_account.pending_rewards = earned_rewards;
        stake_account.reward_per_token_paid = reward_per_token_stored;
        stake_account.updated_at = current_time;

        // Update reward pool
        reward_pool.total_staked = reward_pool.total_staked
            .checked_sub(amount)
            .ok_or(FinovaTokenError::InsufficientStakedAmount)?;
        reward_pool.last_update_time = current_time;

        // Calculate final withdrawal amount after penalty
        let withdrawal_amount = amount
            .checked_sub(penalty_amount)
            .ok_or(FinovaTokenError::MathOverflow)?;

        // Transfer tokens from staking vault to user
        let vault_authority_seeds = &[
            b"staking_vault",
            ctx.accounts.user.key().as_ref(),
            &[ctx.accounts.stake_account.vault_bump],
        ];
        let signer = &[&vault_authority_seeds[..]];

        token::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.staking_vault.to_account_info(),
                    to: ctx.accounts.user_token_account.to_account_info(),
                    authority: ctx.accounts.vault_authority.to_account_info(),
                },
                signer,
            ),
            withdrawal_amount,
        )?;

        // Burn penalty amount if applicable
        if penalty_amount > 0 {
            let mint_info = &mut ctx.accounts.mint_info;
            mint_info.current_supply = mint_info.current_supply
                .checked_sub(penalty_amount)
                .ok_or(FinovaTokenError::InsufficientSupply)?;
        }

        emit!(TokensUnstaked {
            user: ctx.accounts.user.key(),
            amount,
            penalty_amount,
            withdrawal_amount,
            remaining_staked: stake_account.staked_amount,
            timestamp: current_time,
        });

        Ok(())
    }

    /// Claim accumulated staking rewards
    pub fn claim_rewards(
        ctx: Context<ClaimRewards>,
    ) -> Result<()> {
        let stake_account = &mut ctx.accounts.stake_account;
        let reward_pool = &mut ctx.accounts.reward_pool;
        let current_time = Clock::get()?.unix_timestamp;

        // Update reward pool
        update_reward_pool(reward_pool, current_time)?;

        // Calculate total earned rewards
        let reward_per_token_stored = reward_pool.reward_per_token_stored;
        let total_rewards = calculate_earned_rewards(
            stake_account.staked_amount,
            reward_per_token_stored,
            stake_account.reward_per_token_paid,
            stake_account.pending_rewards,
        )?;

        require!(total_rewards > 0, FinovaTokenError::NoRewardsToClaim);

        // Apply lock bonus multiplier
        let final_rewards = total_rewards
            .checked_mul(stake_account.lock_bonus_multiplier as u64)
            .ok_or(FinovaTokenError::MathOverflow)?
            .checked_div(10000)
            .ok_or(FinovaTokenError::MathOverflow)?;

        // Update stake account
        stake_account.pending_rewards = 0;
        stake_account.reward_per_token_paid = reward_per_token_stored;
        stake_account.total_rewards_claimed = stake_account.total_rewards_claimed
            .checked_add(final_rewards)
            .ok_or(FinovaTokenError::MathOverflow)?;
        stake_account.updated_at = current_time;

        // Update reward pool
        reward_pool.total_rewards = reward_pool.total_rewards
            .checked_add(final_rewards)
            .ok_or(FinovaTokenError::MathOverflow)?;

        // Mint reward tokens
        let mint_info = &mut ctx.accounts.mint_info;
        mint_info.current_supply = mint_info.current_supply
            .checked_add(final_rewards)
            .ok_or(FinovaTokenError::MathOverflow)?;

        let authority_seeds = &[
            b"mint_authority",
            &[mint_info.mint_authority_bump],
        ];
        let signer = &[&authority_seeds[..]];

        token::mint_to(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                MintTo {
                    mint: ctx.accounts.mint.to_account_info(),
                    to: ctx.accounts.user_token_account.to_account_info(),
                    authority: ctx.accounts.mint_authority.to_account_info(),
                },
                signer,
            ),
            final_rewards,
        )?;

        emit!(RewardsClaimed {
            user: ctx.accounts.user.key(),
            amount: final_rewards,
            lock_bonus_applied: stake_account.lock_bonus_multiplier,
            total_claimed: stake_account.total_rewards_claimed,
            timestamp: current_time,
        });

        Ok(())
    }

    /// Emergency pause token operations
    pub fn emergency_pause(
        ctx: Context<EmergencyPause>,
    ) -> Result<()> {
        let mint_info = &mut ctx.accounts.mint_info;
        
        require!(
            ctx.accounts.authority.key() == mint_info.mint_authority,
            FinovaTokenError::UnauthorizedAccess
        );

        mint_info.is_paused = true;
        mint_info.updated_at = Clock::get()?.unix_timestamp;

        emit!(EmergencyPauseActivated {
            authority: ctx.accounts.authority.key(),
            timestamp: Clock::get()?.unix_timestamp,
        });

        Ok(())
    }

    /// Resume token operations after pause
    pub fn resume_operations(
        ctx: Context<ResumeOperations>,
    ) -> Result<()> {
        let mint_info = &mut ctx.accounts.mint_info;
        
        require!(
            ctx.accounts.authority.key() == mint_info.mint_authority,
            FinovaTokenError::UnauthorizedAccess
        );

        mint_info.is_paused = false;
        mint_info.updated_at = Clock::get()?.unix_timestamp;

        emit!(OperationsResumed {
            authority: ctx.accounts.authority.key(),
            timestamp: Clock::get()?.unix_timestamp,
        });

        Ok(())
    }

    /// Update reward rate (governance function)
    pub fn update_reward_rate(
        ctx: Context<UpdateRewardRate>,
        new_rate: u64,
    ) -> Result<()> {
        let reward_pool = &mut ctx.accounts.reward_pool;
        let current_time = Clock::get()?.unix_timestamp;

        // Validate rate (max 20% APY = 2000 basis points)
        require!(new_rate <= 2000, FinovaTokenError::InvalidRewardRate);

        // Update reward pool before changing rate
        update_reward_pool(reward_pool, current_time)?;

        let old_rate = reward_pool.reward_rate;
        reward_pool.reward_rate = new_rate;

        emit!(RewardRateUpdated {
            old_rate,
            new_rate,
            timestamp: current_time,
        });

        Ok(())
    }
}

/// Helper function to update reward pool calculations
fn update_reward_pool(
    reward_pool: &mut RewardPool,
    current_time: i64,
) -> Result<()> {
    if reward_pool.total_staked > 0 {
        let time_diff = current_time - reward_pool.last_update_time;
        let reward_increment = (time_diff as u64)
            .checked_mul(reward_pool.reward_rate)
            .ok_or(FinovaTokenError::MathOverflow)?
            .checked_mul(1_000_000)
            .ok_or(FinovaTokenError::MathOverflow)?
            .checked_div(reward_pool.total_staked)
            .ok_or(FinovaTokenError::MathOverflow)?;

        reward_pool.reward_per_token_stored = reward_pool.reward_per_token_stored
            .checked_add(reward_increment)
            .ok_or(FinovaTokenError::MathOverflow)?;
    }
    
    reward_pool.last_update_time = current_time;
    Ok(())
}

/// Helper function to calculate earned rewards
fn calculate_earned_rewards(
    staked_amount: u64,
    reward_per_token_stored: u64,
    reward_per_token_paid: u64,
    pending_rewards: u64,
) -> Result<u64> {
    let reward_diff = reward_per_token_stored
        .checked_sub(reward_per_token_paid)
        .ok_or(FinovaTokenError::MathOverflow)?;
    
    let new_rewards = staked_amount
        .checked_mul(reward_diff)
        .ok_or(FinovaTokenError::MathOverflow)?
        .checked_div(1_000_000)
        .ok_or(FinovaTokenError::MathOverflow)?;
    
    pending_rewards
        .checked_add(new_rewards)
        .ok_or(FinovaTokenError::MathOverflow)
}

/// Helper function to calculate lock bonus multiplier
fn calculate_lock_bonus(lock_duration: u64) -> u16 {
    match lock_duration {
        0..=86400 => 10000,           // 1 day: no bonus (100%)
        86401..=604800 => 10500,      // 1 week: 5% bonus
        604801..=2592000 => 11000,    // 1 month: 10% bonus
        2592001..=7776000 => 12000,   // 3 months: 20% bonus
        7776001..=15552000 => 13500,  // 6 months: 35% bonus
        _ => 15000,                   // 1 year: 50% bonus
    }
}

/// Account validation contexts
#[derive(Accounts)]
#[instruction(decimals: u8, max_supply: u64, initial_supply: u64, mint_authority_bump: u8)]
pub struct InitializeMint<'info> {
    #[account(
        init,
        payer = payer,
        mint::decimals = decimals,
        mint::authority = mint_authority,
        mint::freeze_authority = mint_authority,
    )]
    pub mint: Account<'info, Mint>,

    /// CHECK: PDA authority for mint operations
    #[account(
        seeds = [b"mint_authority"],
        bump = mint_authority_bump,
    )]
    pub mint_authority: UncheckedAccount<'info>,

    #[account(
        init,
        payer = payer,
        space = 8 + size_of::<MintInfo>(),
        seeds = [b"mint_info", mint.key().as_ref()],
        bump,
    )]
    pub mint_info: Account<'info, MintInfo>,

    #[account(
        init,
        payer = payer,
        space = 8 + size_of::<RewardPool>(),
        seeds = [b"reward_pool"],
        bump,
    )]
    pub reward_pool: Account<'info, RewardPool>,

    #[account(
        init,
        payer = payer,
        token::mint = mint,
        token::authority = mint_authority,
        seeds = [b"treasury"],
        bump,
    )]
    pub treasury_account: Account<'info, TokenAccount>,

    #[account(mut)]
    pub payer: Signer<'info>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct MintTokens<'info> {
    #[account(mut)]
    pub mint: Account<'info, Mint>,

    /// CHECK: PDA authority for mint operations
    #[account(
        seeds = [b"mint_authority"],
        bump = mint_info.mint_authority_bump,
    )]
    pub mint_authority: UncheckedAccount<'info>,

    #[account(
        mut,
        seeds = [b"mint_info", mint.key().as_ref()],
        bump,
    )]
    pub mint_info: Account<'info, MintInfo>,

    #[account(
        seeds = [b"authorized_minters"],
        bump,
    )]
    pub authorized_minters: Account<'info, AuthorizedMinters>,

    #[account(
        mut,
        token::mint = mint,
    )]
    pub recipient_account: Account<'info, TokenAccount>,

    pub minter: Signer<'info>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct BurnTokens<'info> {
    #[account(mut)]
    pub mint: Account<'info, Mint>,

    #[account(
        mut,
        seeds = [b"mint_info", mint.key().as_ref()],
        bump,
    )]
    pub mint_info: Account<'info, MintInfo>,

    #[account(
        mut,
        token::mint = mint,
        token::authority = user,
    )]
    pub user_token_account: Account<'info, TokenAccount>,

    pub user: Signer<'info>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct StakeTokens<'info> {
    #[account(
        init_if_needed,
        payer = user,
        space = 8 + size_of::<StakeAccount>(),
        seeds = [b"stake_account", user.key().as_ref()],
        bump,
    )]
    pub stake_account: Account<'info, StakeAccount>,

    #[account(
        mut,
        seeds = [b"reward_pool"],
        bump,
    )]
    pub reward_pool: Account<'info, RewardPool>,

    #[account(
        mut,
        token::mint = mint,
        token::authority = user,
    )]
    pub user_token_account: Account<'info, TokenAccount>,

    #[account(
        init_if_needed,
        payer = user,
        token::mint = mint,
        token::authority = vault_authority,
        seeds = [b"staking_vault", user.key().as_ref()],
        bump,
    )]
    pub staking_vault: Account<'info, TokenAccount>,

    /// CHECK: PDA authority for staking vault
    #[account(
        seeds = [b"vault_authority", user.key().as_ref()],
        bump,
    )]
    pub vault_authority: UncheckedAccount<'info>,

    pub mint: Account<'info, Mint>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct UnstakeTokens<'info> {
    #[account(
        mut,
        seeds = [b"stake_account", user.key().as_ref()],
        bump,
    )]
    pub stake_account: Account<'info, StakeAccount>,

    #[account(
        mut,
        seeds = [b"reward_pool"],
        bump,
    )]
    pub reward_pool: Account<'info, RewardPool>,

    #[account(
        mut,
        seeds = [b"mint_info", mint.key().as_ref()],
        bump,
    )]
    pub mint_info: Account<'info, MintInfo>,

    #[account(
        mut,
        token::mint = mint,
        token::authority = user,
    )]
    pub user_token_account: Account<'info, TokenAccount>,

    #[account(
        mut,
        token::mint = mint,
        token::authority = vault_authority,
        seeds = [b"staking_vault", user.key().as_ref()],
        bump,
    )]
    pub staking_vault: Account<'info, TokenAccount>,

    /// CHECK: PDA authority for staking vault
    #[account(
        seeds = [b"vault_authority", user.key().as_ref()],
        bump = stake_account.vault_bump,
    )]
    pub vault_authority: UncheckedAccount<'info>,

    pub mint: Account<'info, Mint>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct ClaimRewards<'info> {
    #[account(
        mut,
        seeds = [b"stake_account", user.key().as_ref()],
        bump,
    )]
    pub stake_account: Account<'info, StakeAccount>,

    #[account(
        mut,
        seeds = [b"reward_pool"],
        bump,
    )]
    pub reward_pool: Account<'info, RewardPool>,

    #[account(
        mut,
        seeds = [b"mint_info", mint.key().as_ref()],
        bump,
    )]
    pub mint_info: Account<'info, MintInfo>,

    #[account(mut)]
    pub mint: Account<'info, Mint>,

    /// CHECK: PDA authority for mint operations
    #[account(
        seeds = [b"mint_authority"],
        bump = mint_info.mint_authority_bump,
    )]
    pub mint_authority: UncheckedAccount<'info>,

    #[account(
        mut,
        token::mint = mint,
        token::authority = user,
    )]
    pub user_token_account: Account<'info, TokenAccount>,

    #[account(mut)]
    pub user: Signer<'info>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct EmergencyPause<'info> {
    #[account(
        mut,
        seeds = [b"mint_info", mint.key().as_ref()],
        bump,
    )]
    pub mint_info: Account<'info, MintInfo>,

    pub mint: Account<'info, Mint>,
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct ResumeOperations<'info> {
    #[account(
        mut,
        seeds = [b"mint_info", mint.key().as_ref()],
        bump,
    )]
    pub mint_info: Account<'info, MintInfo>,

    pub mint: Account<'info, Mint>,
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct UpdateRewardRate<'info> {
    #[account(
        mut,
        seeds = [b"reward_pool"],
        bump,
    )]
    pub reward_pool: Account<'info, RewardPool>,

    /// CHECK: Governance authority validation handled in instruction
    pub governance_authority: Signer<'info>,
}

/// Additional account structures for authorized minters
#[account]
pub struct AuthorizedMinters {
    pub minters: Vec<Pubkey>,
    pub max_minters: u8,
    pub created_at: i64,
    pub updated_at: i64,
}

impl AuthorizedMinters {
    pub const MAX_SIZE: usize = 8 + // discriminator
        4 + (32 * 10) + // Vec<Pubkey> with max 10 minters
        1 + // max_minters
        8 + // created_at
        8; // updated_at
}

/// Staking tier system for enhanced rewards
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq)]
pub enum StakingTier {
    Bronze,    // 100-499 FIN
    Silver,    // 500-999 FIN
    Gold,      // 1,000-4,999 FIN
    Platinum,  // 5,000-9,999 FIN
    Diamond,   // 10,000+ FIN
}

impl StakingTier {
    pub fn from_amount(amount: u64) -> Self {
        match amount {
            100..=499 => StakingTier::Bronze,
            500..=999 => StakingTier::Silver,
            1000..=4999 => StakingTier::Gold,
            5000..=9999 => StakingTier::Platinum,
            _ => StakingTier::Diamond,
        }
    }

    pub fn get_multiplier(&self) -> u16 {
        match self {
            StakingTier::Bronze => 10000,   // 100% (no bonus)
            StakingTier::Silver => 10200,   // 102% (2% bonus)
            StakingTier::Gold => 10500,     // 105% (5% bonus)
            StakingTier::Platinum => 11000, // 110% (10% bonus)
            StakingTier::Diamond => 12000,  // 120% (20% bonus)
        }
    }

    pub fn get_mining_boost(&self) -> u16 {
        match self {
            StakingTier::Bronze => 10200,   // 2% mining boost
            StakingTier::Silver => 10350,   // 3.5% mining boost
            StakingTier::Gold => 10500,     // 5% mining boost
            StakingTier::Platinum => 10750, // 7.5% mining boost
            StakingTier::Diamond => 11000,  // 10% mining boost
        }
    }
}

/// Integration with Finova Core for cross-program calls
pub mod cross_program {
    use super::*;
    use anchor_lang::solana_program::program::invoke_signed;

    /// Update user's staking tier in Finova Core
    pub fn update_staking_tier(
        core_program: &AccountInfo,
        user: &AccountInfo,
        user_account: &AccountInfo,
        new_tier: StakingTier,
        seeds: &[&[&[u8]]],
    ) -> Result<()> {
        let instruction_data = finova_core::instruction::UpdateStakingTier {
            tier: new_tier as u8,
        };

        let accounts = finova_core::accounts::UpdateStakingTier {
            user: *user.key,
            user_account: *user_account.key,
        };

        let instruction = anchor_lang::solana_program::instruction::Instruction {
            program_id: *core_program.key,
            accounts: accounts.to_account_metas(None),
            data: instruction_data.data(),
        };

        invoke_signed(
            &instruction,
            &[user.clone(), user_account.clone()],
            seeds,
        )?;

        Ok(())
    }

    /// Notify mining boost update
    pub fn update_mining_boost(
        core_program: &AccountInfo,
        user: &AccountInfo,
        mining_account: &AccountInfo,
        boost_multiplier: u16,
        seeds: &[&[&[u8]]],
    ) -> Result<()> {
        let instruction_data = finova_core::instruction::UpdateMiningBoost {
            multiplier: boost_multiplier,
            source: 1, // Staking source
        };

        let accounts = finova_core::accounts::UpdateMiningBoost {
            user: *user.key,
            mining_account: *mining_account.key,
        };

        let instruction = anchor_lang::solana_program::instruction::Instruction {
            program_id: *core_program.key,
            accounts: accounts.to_account_metas(None),
            data: instruction_data.data(),
        };

        invoke_signed(
            &instruction,
            &[user.clone(), mining_account.clone()],
            seeds,
        )?;

        Ok(())
    }
}

/// Advanced staking features
impl StakeAccount {
    /// Calculate current staking tier based on staked amount
    pub fn get_current_tier(&self) -> StakingTier {
        StakingTier::from_amount(self.staked_amount)
    }

    /// Check if user is eligible for tier upgrade
    pub fn is_eligible_for_upgrade(&self, new_amount: u64) -> bool {
        let current_tier = self.get_current_tier();
        let new_tier = StakingTier::from_amount(self.staked_amount + new_amount);
        new_tier as u8 > current_tier as u8
    }

    /// Calculate effective staking power with all bonuses
    pub fn get_effective_staking_power(&self) -> Result<u64> {
        let base_power = self.staked_amount;
        let tier_multiplier = self.get_current_tier().get_multiplier();
        let lock_multiplier = self.lock_bonus_multiplier;

        // Combine tier and lock bonuses
        let total_multiplier = tier_multiplier
            .checked_mul(lock_multiplier)
            .ok_or(FinovaTokenError::MathOverflow)?
            .checked_div(10000)
            .ok_or(FinovaTokenError::MathOverflow)?;

        base_power
            .checked_mul(total_multiplier as u64)
            .ok_or(FinovaTokenError::MathOverflow)?
            .checked_div(10000)
            .ok_or(FinovaTokenError::MathOverflow)
    }

    /// Check if stake is currently locked
    pub fn is_locked(&self, current_time: i64) -> bool {
        current_time < self.lock_end_time
    }

    /// Get remaining lock time in seconds
    pub fn get_remaining_lock_time(&self, current_time: i64) -> i64 {
        if self.is_locked(current_time) {
            self.lock_end_time - current_time
        } else {
            0
        }
    }
}

/// Reward pool management utilities
impl RewardPool {
    /// Calculate current APY based on total staked and reward distribution
    pub fn get_current_apy(&self) -> u64 {
        if self.total_staked == 0 {
            return self.reward_rate;
        }

        // Dynamic APY calculation based on staking ratio
        let staking_ratio = self.total_staked
            .checked_mul(10000)
            .unwrap_or(0)
            .checked_div(self.total_rewards.max(1))
            .unwrap_or(0);

        // Adjust APY based on staking participation
        match staking_ratio {
            0..=1000 => self.reward_rate + 200,     // Low participation: +2% APY
            1001..=3000 => self.reward_rate + 100,  // Medium participation: +1% APY
            3001..=5000 => self.reward_rate,        // Target participation: base APY
            _ => self.reward_rate.saturating_sub(100), // High participation: -1% APY
        }
    }

    /// Estimate future rewards for a given stake amount and duration
    pub fn estimate_rewards(&self, stake_amount: u64, duration_seconds: u64) -> Result<u64> {
        let annual_reward_rate = self.get_current_apy();
        let duration_years = duration_seconds as f64 / (365.25 * 24.0 * 3600.0);
        
        let estimated_rewards = (stake_amount as f64)
            * (annual_reward_rate as f64 / 10000.0)
            * duration_years;

        Ok(estimated_rewards as u64)
    }

    /// Check if reward pool needs rebalancing
    pub fn needs_rebalancing(&self) -> bool {
        let current_time = Clock::get().unwrap().unix_timestamp;
        let time_since_update = current_time - self.last_update_time;
        
        // Rebalance if more than 1 hour since last update or significant staking changes
        time_since_update > 3600 || self.total_staked > self.total_rewards * 2
    }
}

/// Utility functions for token operations
pub mod token_utils {
    use super::*;

    /// Calculate optimal staking amount based on user's token balance
    pub fn calculate_optimal_stake(
        token_balance: u64,
        risk_tolerance: u8, // 1-10 scale
    ) -> u64 {
        let risk_percentage = match risk_tolerance {
            1..=3 => 20,   // Conservative: 20% max stake
            4..=6 => 50,   // Moderate: 50% max stake
            7..=8 => 75,   // Aggressive: 75% max stake
            9..=10 => 90,  // Very aggressive: 90% max stake
            _ => 30,       // Default: 30% max stake
        };

        token_balance
            .checked_mul(risk_percentage)
            .unwrap_or(0)
            .checked_div(100)
            .unwrap_or(0)
    }

    /// Calculate early withdrawal penalty
    pub fn calculate_early_withdrawal_penalty(
        stake_amount: u64,
        remaining_lock_time: i64,
        base_penalty_rate: u16, // in basis points
    ) -> u64 {
        if remaining_lock_time <= 0 {
            return 0;
        }

        // Penalty scales with remaining lock time
        let time_factor = (remaining_lock_time as u64).min(31536000); // Cap at 1 year
        let penalty_multiplier = time_factor
            .checked_mul(base_penalty_rate as u64)
            .unwrap_or(0)
            .checked_div(31536000) // Scale to annual basis
            .unwrap_or(0)
            .max(base_penalty_rate as u64); // Minimum base penalty

        stake_amount
            .checked_mul(penalty_multiplier)
            .unwrap_or(0)
            .checked_div(10000)
            .unwrap_or(0)
    }

    /// Validate token transfer amount
    pub fn validate_transfer_amount(
        amount: u64,
        account_balance: u64,
        min_amount: u64,
        max_amount: u64,
    ) -> Result<()> {
        require!(amount >= min_amount, FinovaTokenError::AmountTooSmall);
        require!(amount <= max_amount, FinovaTokenError::AmountTooLarge);
        require!(amount <= account_balance, FinovaTokenError::InsufficientBalance);
        Ok(())
    }
}

/// Constants for token operations
pub mod constants {
    /// Minimum staking amount (100 FIN)
    pub const MIN_STAKE_AMOUNT: u64 = 100_000_000; // 100 FIN with 6 decimals
    
    /// Maximum staking amount (1M FIN)
    pub const MAX_STAKE_AMOUNT: u64 = 1_000_000_000_000; // 1M FIN with 6 decimals
    
    /// Minimum lock duration (1 day)
    pub const MIN_LOCK_DURATION: u64 = 86400;
    
    /// Maximum lock duration (1 year)
    pub const MAX_LOCK_DURATION: u64 = 31536000;
    
    /// Base early withdrawal penalty (10%)
    pub const BASE_EARLY_WITHDRAWAL_PENALTY: u16 = 1000; // 10% in basis points
    
    /// Maximum reward rate (20% APY)
    pub const MAX_REWARD_RATE: u64 = 2000; // 20% in basis points
    
    /// Reward calculation precision
    pub const REWARD_PRECISION: u64 = 1_000_000;
    
    /// Maximum number of authorized minters
    pub const MAX_AUTHORIZED_MINTERS: u8 = 10;
}
