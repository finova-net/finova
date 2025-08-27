// programs/finova-defi/src/instructions/yield_farm.rs

use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{self, Mint, Token, TokenAccount, Transfer},
};
use crate::{
    constants::*,
    errors::FinovaDefiError,
    math::{curve::calculate_rewards, fees::calculate_farm_fee},
    state::{farm::*, vault::*},
    utils::*,
};

/// Initialize a new yield farm
#[derive(Accounts)]
#[instruction(farm_id: u64)]
pub struct InitializeFarm<'info> {
    #[account(
        init,
        payer = authority,
        space = Farm::LEN,
        seeds = [b"farm", farm_id.to_le_bytes().as_ref()],
        bump
    )]
    pub farm: Account<'info, Farm>,

    #[account(
        init,
        payer = authority,
        space = FarmVault::LEN,
        seeds = [b"farm_vault", farm.key().as_ref()],
        bump
    )]
    pub farm_vault: Account<'info, FarmVault>,

    #[account(
        init,
        payer = authority,
        token::mint = stake_mint,
        token::authority = farm_vault,
        seeds = [b"stake_vault", farm.key().as_ref()],
        bump
    )]
    pub stake_vault: Account<'info, TokenAccount>,

    #[account(
        init,
        payer = authority,
        token::mint = reward_mint,
        token::authority = farm_vault,
        seeds = [b"reward_vault", farm.key().as_ref()],
        bump
    )]
    pub reward_vault: Account<'info, TokenAccount>,

    pub stake_mint: Account<'info, Mint>,
    pub reward_mint: Account<'info, Mint>,

    #[account(mut)]
    pub authority: Signer<'info>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub rent: Sysvar<'info, Rent>,
}

/// Stake tokens in yield farm
#[derive(Accounts)]
pub struct StakeInFarm<'info> {
    #[account(
        mut,
        constraint = farm.is_active @ FinovaDefiError::FarmInactive
    )]
    pub farm: Account<'info, Farm>,

    #[account(
        mut,
        seeds = [b"farm_vault", farm.key().as_ref()],
        bump = farm_vault.bump
    )]
    pub farm_vault: Account<'info, FarmVault>,

    #[account(
        mut,
        seeds = [b"stake_vault", farm.key().as_ref()],
        bump
    )]
    pub stake_vault: Account<'info, TokenAccount>,

    #[account(
        init_if_needed,
        payer = user,
        space = UserStake::LEN,
        seeds = [b"user_stake", farm.key().as_ref(), user.key().as_ref()],
        bump
    )]
    pub user_stake: Account<'info, UserStake>,

    #[account(
        mut,
        constraint = user_token_account.mint == farm.stake_mint @ FinovaDefiError::InvalidMint,
        constraint = user_token_account.owner == user.key() @ FinovaDefiError::InvalidOwner
    )]
    pub user_token_account: Account<'info, TokenAccount>,

    #[account(mut)]
    pub user: Signer<'info>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
}

/// Unstake tokens from yield farm
#[derive(Accounts)]
pub struct UnstakeFromFarm<'info> {
    #[account(
        mut,
        constraint = farm.is_active @ FinovaDefiError::FarmInactive
    )]
    pub farm: Account<'info, Farm>,

    #[account(
        mut,
        seeds = [b"farm_vault", farm.key().as_ref()],
        bump = farm_vault.bump
    )]
    pub farm_vault: Account<'info, FarmVault>,

    #[account(
        mut,
        seeds = [b"stake_vault", farm.key().as_ref()],
        bump
    )]
    pub stake_vault: Account<'info, TokenAccount>,

    #[account(
        mut,
        seeds = [b"user_stake", farm.key().as_ref(), user.key().as_ref()],
        bump = user_stake.bump,
        constraint = user_stake.amount > 0 @ FinovaDefiError::NoStakeFound
    )]
    pub user_stake: Account<'info, UserStake>,

    #[account(
        mut,
        constraint = user_token_account.mint == farm.stake_mint @ FinovaDefiError::InvalidMint,
        constraint = user_token_account.owner == user.key() @ FinovaDefiError::InvalidOwner
    )]
    pub user_token_account: Account<'info, TokenAccount>,

    #[account(mut)]
    pub user: Signer<'info>,

    pub token_program: Program<'info, Token>,
}

/// Claim rewards from yield farm
#[derive(Accounts)]
pub struct ClaimFarmRewards<'info> {
    #[account(
        mut,
        constraint = farm.is_active @ FinovaDefiError::FarmInactive
    )]
    pub farm: Account<'info, Farm>,

    #[account(
        mut,
        seeds = [b"farm_vault", farm.key().as_ref()],
        bump = farm_vault.bump
    )]
    pub farm_vault: Account<'info, FarmVault>,

    #[account(
        mut,
        seeds = [b"reward_vault", farm.key().as_ref()],
        bump
    )]
    pub reward_vault: Account<'info, TokenAccount>,

    #[account(
        mut,
        seeds = [b"user_stake", farm.key().as_ref(), user.key().as_ref()],
        bump = user_stake.bump,
        constraint = user_stake.amount > 0 @ FinovaDefiError::NoStakeFound
    )]
    pub user_stake: Account<'info, UserStake>,

    #[account(
        init_if_needed,
        payer = user,
        associated_token::mint = reward_mint,
        associated_token::authority = user
    )]
    pub user_reward_account: Account<'info, TokenAccount>,

    pub reward_mint: Account<'info, Mint>,

    #[account(mut)]
    pub user: Signer<'info>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub rent: Sysvar<'info, Rent>,
}

/// Update farm parameters (admin only)
#[derive(Accounts)]
pub struct UpdateFarm<'info> {
    #[account(
        mut,
        constraint = farm.authority == authority.key() @ FinovaDefiError::UnauthorizedAccess
    )]
    pub farm: Account<'info, Farm>,

    pub authority: Signer<'info>,
}

/// Emergency pause farm (admin only)
#[derive(Accounts)]
pub struct PauseFarm<'info> {
    #[account(
        mut,
        constraint = farm.authority == authority.key() @ FinovaDefiError::UnauthorizedAccess
    )]
    pub farm: Account<'info, Farm>,

    pub authority: Signer<'info>,
}

/// Compound rewards back into stake
#[derive(Accounts)]
pub struct CompoundRewards<'info> {
    #[account(
        mut,
        constraint = farm.is_active @ FinovaDefiError::FarmInactive,
        constraint = farm.compound_enabled @ FinovaDefiError::CompoundingDisabled
    )]
    pub farm: Account<'info, Farm>,

    #[account(
        mut,
        seeds = [b"farm_vault", farm.key().as_ref()],
        bump = farm_vault.bump
    )]
    pub farm_vault: Account<'info, FarmVault>,

    #[account(
        mut,
        seeds = [b"stake_vault", farm.key().as_ref()],
        bump
    )]
    pub stake_vault: Account<'info, TokenAccount>,

    #[account(
        mut,
        seeds = [b"reward_vault", farm.key().as_ref()],
        bump
    )]
    pub reward_vault: Account<'info, TokenAccount>,

    #[account(
        mut,
        seeds = [b"user_stake", farm.key().as_ref(), user.key().as_ref()],
        bump = user_stake.bump,
        constraint = user_stake.amount > 0 @ FinovaDefiError::NoStakeFound
    )]
    pub user_stake: Account<'info, UserStake>,

    #[account(mut)]
    pub user: Signer<'info>,

    pub token_program: Program<'info, Token>,
}

// Implementation functions
pub fn initialize_farm(
    ctx: Context<InitializeFarm>,
    farm_id: u64,
    reward_rate: u64,
    lock_period: i64,
    max_stake_per_user: u64,
    compound_enabled: bool,
) -> Result<()> {
    let farm = &mut ctx.accounts.farm;
    let farm_vault = &mut ctx.accounts.farm_vault;
    let clock = Clock::get()?;

    // Initialize farm state
    farm.farm_id = farm_id;
    farm.authority = ctx.accounts.authority.key();
    farm.stake_mint = ctx.accounts.stake_mint.key();
    farm.reward_mint = ctx.accounts.reward_mint.key();
    farm.stake_vault = ctx.accounts.stake_vault.key();
    farm.reward_vault = ctx.accounts.reward_vault.key();
    farm.vault = ctx.accounts.farm_vault.key();
    
    // Set farm parameters
    farm.reward_rate = reward_rate;
    farm.lock_period = lock_period;
    farm.max_stake_per_user = max_stake_per_user;
    farm.compound_enabled = compound_enabled;
    farm.is_active = true;
    
    // Initialize timing
    farm.start_time = clock.unix_timestamp;
    farm.last_update_time = clock.unix_timestamp;
    farm.end_time = 0; // Infinite farm initially
    
    // Initialize statistics
    farm.total_staked = 0;
    farm.total_rewards_distributed = 0;
    farm.total_users = 0;
    farm.accumulated_reward_per_share = 0;
    
    // Set bump
    farm.bump = ctx.bumps.farm;

    // Initialize farm vault
    farm_vault.farm = farm.key();
    farm_vault.authority = ctx.accounts.authority.key();
    farm_vault.stake_vault = ctx.accounts.stake_vault.key();
    farm_vault.reward_vault = ctx.accounts.reward_vault.key();
    farm_vault.bump = ctx.bumps.farm_vault;
    farm_vault.stake_vault_bump = ctx.bumps.stake_vault;
    farm_vault.reward_vault_bump = ctx.bumps.reward_vault;

    msg!("Farm initialized with ID: {}", farm_id);
    Ok(())
}

pub fn stake_in_farm(
    ctx: Context<StakeInFarm>, 
    amount: u64
) -> Result<()> {
    require!(amount > 0, FinovaDefiError::InvalidAmount);
    
    let farm = &mut ctx.accounts.farm;
    let farm_vault = &mut ctx.accounts.farm_vault;
    let user_stake = &mut ctx.accounts.user_stake;
    let clock = Clock::get()?;

    // Check maximum stake limit
    let new_user_total = user_stake.amount.checked_add(amount)
        .ok_or(FinovaDefiError::MathOverflow)?;
    require!(
        new_user_total <= farm.max_stake_per_user,
        FinovaDefiError::ExceedsMaxStake
    );

    // Update farm rewards before changing stake
    update_farm_rewards(farm, clock.unix_timestamp)?;

    // Initialize user stake if first time
    if user_stake.amount == 0 {
        user_stake.user = ctx.accounts.user.key();
        user_stake.farm = farm.key();
        user_stake.amount = 0;
        user_stake.reward_debt = 0;
        user_stake.last_stake_time = clock.unix_timestamp;
        user_stake.pending_rewards = 0;
        user_stake.bump = ctx.bumps.user_stake;
        
        // Increment user count
        farm.total_users = farm.total_users.checked_add(1)
            .ok_or(FinovaDefiError::MathOverflow)?;
    }

    // Calculate pending rewards before updating stake
    if user_stake.amount > 0 {
        let pending = calculate_pending_rewards(
            user_stake.amount,
            farm.accumulated_reward_per_share,
            user_stake.reward_debt,
        )?;
        user_stake.pending_rewards = user_stake.pending_rewards
            .checked_add(pending)
            .ok_or(FinovaDefiError::MathOverflow)?;
    }

    // Transfer tokens from user to farm
    let transfer_ctx = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        Transfer {
            from: ctx.accounts.user_token_account.to_account_info(),
            to: ctx.accounts.stake_vault.to_account_info(),
            authority: ctx.accounts.user.to_account_info(),
        },
    );
    token::transfer(transfer_ctx, amount)?;

    // Update state
    user_stake.amount = user_stake.amount.checked_add(amount)
        .ok_or(FinovaDefiError::MathOverflow)?;
    user_stake.last_stake_time = clock.unix_timestamp;
    user_stake.reward_debt = calculate_reward_debt(
        user_stake.amount,
        farm.accumulated_reward_per_share,
    )?;

    farm.total_staked = farm.total_staked.checked_add(amount)
        .ok_or(FinovaDefiError::MathOverflow)?;

    msg!("Staked {} tokens in farm {}", amount, farm.farm_id);
    Ok(())
}

pub fn unstake_from_farm(
    ctx: Context<UnstakeFromFarm>,
    amount: u64,
) -> Result<()> {
    require!(amount > 0, FinovaDefiError::InvalidAmount);
    
    let farm = &mut ctx.accounts.farm;
    let user_stake = &mut ctx.accounts.user_stake;
    let clock = Clock::get()?;

    require!(
        user_stake.amount >= amount,
        FinovaDefiError::InsufficientStake
    );

    // Check lock period
    let time_staked = clock.unix_timestamp - user_stake.last_stake_time;
    require!(
        time_staked >= farm.lock_period,
        FinovaDefiError::StillLocked
    );

    // Update farm rewards
    update_farm_rewards(farm, clock.unix_timestamp)?;

    // Calculate pending rewards
    let pending = calculate_pending_rewards(
        user_stake.amount,
        farm.accumulated_reward_per_share,
        user_stake.reward_debt,
    )?;
    user_stake.pending_rewards = user_stake.pending_rewards
        .checked_add(pending)
        .ok_or(FinovaDefiError::MathOverflow)?;

    // Calculate unstaking fee if applicable
    let fee = calculate_farm_fee(amount, UNSTAKING_FEE_BPS)?;
    let amount_after_fee = amount.checked_sub(fee)
        .ok_or(FinovaDefiError::MathOverflow)?;

    // Transfer tokens back to user (minus fee)
    let farm_key = farm.key();
    let seeds = &[
        b"farm_vault",
        farm_key.as_ref(),
        &[ctx.accounts.farm_vault.bump],
    ];
    let signer_seeds = &[&seeds[..]];

    let transfer_ctx = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        Transfer {
            from: ctx.accounts.stake_vault.to_account_info(),
            to: ctx.accounts.user_token_account.to_account_info(),
            authority: ctx.accounts.farm_vault.to_account_info(),
        },
        signer_seeds,
    );
    token::transfer(transfer_ctx, amount_after_fee)?;

    // Update state
    user_stake.amount = user_stake.amount.checked_sub(amount)
        .ok_or(FinovaDefiError::MathOverflow)?;
    user_stake.reward_debt = calculate_reward_debt(
        user_stake.amount,
        farm.accumulated_reward_per_share,
    )?;

    farm.total_staked = farm.total_staked.checked_sub(amount)
        .ok_or(FinovaDefiError::MathOverflow)?;

    // Update user count if fully unstaked
    if user_stake.amount == 0 {
        farm.total_users = farm.total_users.checked_sub(1)
            .ok_or(FinovaDefiError::MathOverflow)?;
    }

    msg!("Unstaked {} tokens from farm {}", amount, farm.farm_id);
    Ok(())
}

pub fn claim_farm_rewards(ctx: Context<ClaimFarmRewards>) -> Result<()> {
    let farm = &mut ctx.accounts.farm;
    let user_stake = &mut ctx.accounts.user_stake;
    let clock = Clock::get()?;

    // Update farm rewards
    update_farm_rewards(farm, clock.unix_timestamp)?;

    // Calculate total rewards to claim
    let pending = calculate_pending_rewards(
        user_stake.amount,
        farm.accumulated_reward_per_share,
        user_stake.reward_debt,
    )?;
    
    let total_rewards = user_stake.pending_rewards
        .checked_add(pending)
        .ok_or(FinovaDefiError::MathOverflow)?;

    require!(total_rewards > 0, FinovaDefiError::NoRewardsToClaim);

    // Transfer rewards to user
    let farm_key = farm.key();
    let seeds = &[
        b"farm_vault",
        farm_key.as_ref(),
        &[ctx.accounts.farm_vault.bump],
    ];
    let signer_seeds = &[&seeds[..]];

    let transfer_ctx = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        Transfer {
            from: ctx.accounts.reward_vault.to_account_info(),
            to: ctx.accounts.user_reward_account.to_account_info(),
            authority: ctx.accounts.farm_vault.to_account_info(),
        },
        signer_seeds,
    );
    token::transfer(transfer_ctx, total_rewards)?;

    // Update state
    user_stake.pending_rewards = 0;
    user_stake.reward_debt = calculate_reward_debt(
        user_stake.amount,
        farm.accumulated_reward_per_share,
    )?;

    farm.total_rewards_distributed = farm.total_rewards_distributed
        .checked_add(total_rewards)
        .ok_or(FinovaDefiError::MathOverflow)?;

    msg!("Claimed {} reward tokens from farm {}", total_rewards, farm.farm_id);
    Ok(())
}

pub fn compound_rewards(ctx: Context<CompoundRewards>) -> Result<()> {
    let farm = &mut ctx.accounts.farm;
    let user_stake = &mut ctx.accounts.user_stake;
    let clock = Clock::get()?;

    // Update farm rewards
    update_farm_rewards(farm, clock.unix_timestamp)?;

    // Calculate rewards to compound
    let pending = calculate_pending_rewards(
        user_stake.amount,
        farm.accumulated_reward_per_share,
        user_stake.reward_debt,
    )?;
    
    let total_rewards = user_stake.pending_rewards
        .checked_add(pending)
        .ok_or(FinovaDefiError::MathOverflow)?;

    require!(total_rewards > 0, FinovaDefiError::NoRewardsToClaim);

    // Check if reward token is same as stake token for compounding
    require!(
        farm.stake_mint == farm.reward_mint,
        FinovaDefiError::CompoundingNotSupported
    );

    // Transfer rewards from reward vault to stake vault (compounding)
    let farm_key = farm.key();
    let seeds = &[
        b"farm_vault",
        farm_key.as_ref(),
        &[ctx.accounts.farm_vault.bump],
    ];
    let signer_seeds = &[&seeds[..]];

    let transfer_ctx = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        Transfer {
            from: ctx.accounts.reward_vault.to_account_info(),
            to: ctx.accounts.stake_vault.to_account_info(),
            authority: ctx.accounts.farm_vault.to_account_info(),
        },
        signer_seeds,
    );
    token::transfer(transfer_ctx, total_rewards)?;

    // Update user stake with compounded amount
    user_stake.amount = user_stake.amount
        .checked_add(total_rewards)
        .ok_or(FinovaDefiError::MathOverflow)?;
    user_stake.pending_rewards = 0;
    user_stake.reward_debt = calculate_reward_debt(
        user_stake.amount,
        farm.accumulated_reward_per_share,
    )?;

    // Update farm total staked
    farm.total_staked = farm.total_staked
        .checked_add(total_rewards)
        .ok_or(FinovaDefiError::MathOverflow)?;

    farm.total_rewards_distributed = farm.total_rewards_distributed
        .checked_add(total_rewards)
        .ok_or(FinovaDefiError::MathOverflow)?;

    msg!("Compounded {} tokens in farm {}", total_rewards, farm.farm_id);
    Ok(())
}

pub fn update_farm(
    ctx: Context<UpdateFarm>,
    new_reward_rate: Option<u64>,
    new_max_stake: Option<u64>,
    new_end_time: Option<i64>,
) -> Result<()> {
    let farm = &mut ctx.accounts.farm;
    let clock = Clock::get()?;

    // Update farm rewards before changing parameters
    update_farm_rewards(farm, clock.unix_timestamp)?;

    if let Some(reward_rate) = new_reward_rate {
        farm.reward_rate = reward_rate;
        msg!("Updated reward rate to: {}", reward_rate);
    }

    if let Some(max_stake) = new_max_stake {
        farm.max_stake_per_user = max_stake;
        msg!("Updated max stake per user to: {}", max_stake);
    }

    if let Some(end_time) = new_end_time {
        farm.end_time = end_time;
        msg!("Updated farm end time to: {}", end_time);
    }

    Ok(())
}

pub fn pause_farm(ctx: Context<PauseFarm>) -> Result<()> {
    let farm = &mut ctx.accounts.farm;
    farm.is_active = false;
    msg!("Farm {} paused", farm.farm_id);
    Ok(())
}

// Helper functions
fn update_farm_rewards(farm: &mut Farm, current_time: i64) -> Result<()> {
    if farm.total_staked == 0 {
        farm.last_update_time = current_time;
        return Ok(());
    }

    let time_diff = current_time - farm.last_update_time;
    if time_diff <= 0 {
        return Ok(());
    }

    let rewards = calculate_rewards(
        farm.reward_rate,
        time_diff as u64,
        farm.total_staked,
    )?;

    if rewards > 0 {
        farm.accumulated_reward_per_share = farm.accumulated_reward_per_share
            .checked_add(rewards.checked_mul(PRECISION).unwrap().checked_div(farm.total_staked).unwrap())
            .ok_or(FinovaDefiError::MathOverflow)?;
    }

    farm.last_update_time = current_time;
    Ok(())
}

fn calculate_pending_rewards(
    user_amount: u64,
    accumulated_reward_per_share: u128,
    reward_debt: u128,
) -> Result<u64> {
    let user_rewards = (user_amount as u128)
        .checked_mul(accumulated_reward_per_share)
        .ok_or(FinovaDefiError::MathOverflow)?
        .checked_div(PRECISION)
        .ok_or(FinovaDefiError::MathOverflow)?;

    if user_rewards > reward_debt {
        Ok((user_rewards - reward_debt) as u64)
    } else {
        Ok(0)
    }
}

fn calculate_reward_debt(
    user_amount: u64,
    accumulated_reward_per_share: u128,
) -> Result<u128> {
    (user_amount as u128)
        .checked_mul(accumulated_reward_per_share)
        .ok_or(FinovaDefiError::MathOverflow)?
        .checked_div(PRECISION)
        .ok_or(FinovaDefiError::MathOverflow.into())
}

const PRECISION: u128 = 1_000_000_000_000; // 1e12 for precision
const UNSTAKING_FEE_BPS: u64 = 50; // 0.5% unstaking fee
