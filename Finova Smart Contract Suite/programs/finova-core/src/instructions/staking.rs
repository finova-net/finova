// programs/finova-core/src/instructions/staking.rs

use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount, Transfer};
use crate::constants::*;
use crate::errors::FinovaError;
use crate::events::{StakeEvent, UnstakeEvent, ClaimRewardsEvent, UpdateStakingTierEvent};
use crate::state::{
    User, StakeAccount, StakingPool, RewardDistribution, StakingTier, StakingConfig
};
use crate::utils::{
    calculate_staking_multiplier, calculate_loyalty_bonus, calculate_activity_bonus,
    validate_staking_amount, get_current_timestamp, calculate_compound_interest
};

/// Initialize the staking system with configuration parameters
#[derive(Accounts)]
pub struct InitializeStaking<'info> {
    #[account(
        init,
        payer = authority,
        space = StakingConfig::SPACE,
        seeds = [STAKING_CONFIG_SEED],
        bump
    )]
    pub staking_config: Account<'info, StakingConfig>,
    
    #[account(
        init,
        payer = authority,
        space = StakingPool::SPACE,
        seeds = [STAKING_POOL_SEED],
        bump
    )]
    pub staking_pool: Account<'info, StakingPool>,
    
    #[account(
        init,
        payer = authority,
        space = RewardDistribution::SPACE,
        seeds = [REWARD_DISTRIBUTION_SEED],
        bump
    )]
    pub reward_distribution: Account<'info, RewardDistribution>,
    
    /// FIN token mint for staking
    pub fin_mint: Account<'info, Mint>,
    
    /// sFIN token mint for liquid staking derivative
    #[account(
        init,
        payer = authority,
        mint::decimals = fin_mint.decimals,
        mint::authority = staking_pool,
        seeds = [SFIN_MINT_SEED],
        bump
    )]
    pub sfin_mint: Account<'info, Mint>,
    
    /// Vault to hold staked FIN tokens
    #[account(
        init,
        payer = authority,
        token::mint = fin_mint,
        token::authority = staking_pool,
        seeds = [STAKING_VAULT_SEED],
        bump
    )]
    pub staking_vault: Account<'info, TokenAccount>,
    
    /// Treasury for reward distribution
    #[account(
        init,
        payer = authority,
        token::mint = fin_mint,
        token::authority = staking_pool,
        seeds = [REWARD_TREASURY_SEED],
        bump
    )]
    pub reward_treasury: Account<'info, TokenAccount>,
    
    #[account(mut)]
    pub authority: Signer<'info>,
    
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
}

/// Stake FIN tokens to receive sFIN with integrated multipliers
#[derive(Accounts)]
pub struct StakeTokens<'info> {
    #[account(
        mut,
        seeds = [STAKING_CONFIG_SEED],
        bump = staking_config.bump
    )]
    pub staking_config: Account<'info, StakingConfig>,
    
    #[account(
        mut,
        seeds = [STAKING_POOL_SEED],
        bump = staking_pool.bump
    )]
    pub staking_pool: Account<'info, StakingPool>,
    
    #[account(
        mut,
        seeds = [USER_SEED, user.key().as_ref()],
        bump = user_account.bump
    )]
    pub user_account: Account<'info, User>,
    
    #[account(
        init_if_needed,
        payer = user,
        space = StakeAccount::SPACE,
        seeds = [STAKE_ACCOUNT_SEED, user.key().as_ref()],
        bump
    )]
    pub stake_account: Account<'info, StakeAccount>,
    
    /// User's FIN token account
    #[account(
        mut,
        token::mint = staking_config.fin_mint,
        token::authority = user
    )]
    pub user_fin_account: Account<'info, TokenAccount>,
    
    /// User's sFIN token account
    #[account(
        init_if_needed,
        payer = user,
        token::mint = staking_config.sfin_mint,
        token::authority = user
    )]
    pub user_sfin_account: Account<'info, TokenAccount>,
    
    /// Staking vault
    #[account(
        mut,
        seeds = [STAKING_VAULT_SEED],
        bump = staking_pool.vault_bump
    )]
    pub staking_vault: Account<'info, TokenAccount>,
    
    /// sFIN mint
    #[account(
        mut,
        seeds = [SFIN_MINT_SEED],
        bump = staking_pool.sfin_mint_bump
    )]
    pub sfin_mint: Account<'info, Mint>,
    
    #[account(mut)]
    pub user: Signer<'info>,
    
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
}

/// Unstake sFIN tokens to receive FIN tokens back
#[derive(Accounts)]
pub struct UnstakeTokens<'info> {
    #[account(
        mut,
        seeds = [STAKING_CONFIG_SEED],
        bump = staking_config.bump
    )]
    pub staking_config: Account<'info, StakingConfig>,
    
    #[account(
        mut,
        seeds = [STAKING_POOL_SEED],
        bump = staking_pool.bump
    )]
    pub staking_pool: Account<'info, StakingPool>,
    
    #[account(
        mut,
        seeds = [USER_SEED, user.key().as_ref()],
        bump = user_account.bump
    )]
    pub user_account: Account<'info, User>,
    
    #[account(
        mut,
        seeds = [STAKE_ACCOUNT_SEED, user.key().as_ref()],
        bump = stake_account.bump
    )]
    pub stake_account: Account<'info, StakeAccount>,
    
    /// User's FIN token account
    #[account(
        mut,
        token::mint = staking_config.fin_mint,
        token::authority = user
    )]
    pub user_fin_account: Account<'info, TokenAccount>,
    
    /// User's sFIN token account
    #[account(
        mut,
        token::mint = staking_config.sfin_mint,
        token::authority = user
    )]
    pub user_sfin_account: Account<'info, TokenAccount>,
    
    /// Staking vault
    #[account(
        mut,
        seeds = [STAKING_VAULT_SEED],
        bump = staking_pool.vault_bump
    )]
    pub staking_vault: Account<'info, TokenAccount>,
    
    /// sFIN mint for burning
    #[account(
        mut,
        seeds = [SFIN_MINT_SEED],
        bump = staking_pool.sfin_mint_bump
    )]
    pub sfin_mint: Account<'info, Mint>,
    
    #[account(mut)]
    pub user: Signer<'info>,
    
    pub token_program: Program<'info, Token>,
}

/// Claim staking rewards with compound interest
#[derive(Accounts)]
pub struct ClaimStakingRewards<'info> {
    #[account(
        mut,
        seeds = [STAKING_CONFIG_SEED],
        bump = staking_config.bump
    )]
    pub staking_config: Account<'info, StakingConfig>,
    
    #[account(
        mut,
        seeds = [STAKING_POOL_SEED],
        bump = staking_pool.bump
    )]
    pub staking_pool: Account<'info, StakingPool>,
    
    #[account(
        mut,
        seeds = [REWARD_DISTRIBUTION_SEED],
        bump = reward_distribution.bump
    )]
    pub reward_distribution: Account<'info, RewardDistribution>,
    
    #[account(
        mut,
        seeds = [USER_SEED, user.key().as_ref()],
        bump = user_account.bump
    )]
    pub user_account: Account<'info, User>,
    
    #[account(
        mut,
        seeds = [STAKE_ACCOUNT_SEED, user.key().as_ref()],
        bump = stake_account.bump
    )]
    pub stake_account: Account<'info, StakeAccount>,
    
    /// User's FIN token account for rewards
    #[account(
        mut,
        token::mint = staking_config.fin_mint,
        token::authority = user
    )]
    pub user_fin_account: Account<'info, TokenAccount>,
    
    /// Reward treasury
    #[account(
        mut,
        seeds = [REWARD_TREASURY_SEED],
        bump = staking_pool.treasury_bump
    )]
    pub reward_treasury: Account<'info, TokenAccount>,
    
    #[account(mut)]
    pub user: Signer<'info>,
    
    pub token_program: Program<'info, Token>,
}

/// Update staking tier based on stake amount
#[derive(Accounts)]
pub struct UpdateStakingTier<'info> {
    #[account(
        mut,
        seeds = [STAKING_CONFIG_SEED],
        bump = staking_config.bump
    )]
    pub staking_config: Account<'info, StakingConfig>,
    
    #[account(
        mut,
        seeds = [USER_SEED, user.key().as_ref()],
        bump = user_account.bump
    )]
    pub user_account: Account<'info, User>,
    
    #[account(
        mut,
        seeds = [STAKE_ACCOUNT_SEED, user.key().as_ref()],
        bump = stake_account.bump
    )]
    pub stake_account: Account<'info, StakeAccount>,
    
    pub user: Signer<'info>,
}

/// Emergency pause staking operations (admin only)
#[derive(Accounts)]
pub struct EmergencyPause<'info> {
    #[account(
        mut,
        seeds = [STAKING_CONFIG_SEED],
        bump = staking_config.bump,
        has_one = authority @ FinovaError::Unauthorized
    )]
    pub staking_config: Account<'info, StakingConfig>,
    
    pub authority: Signer<'info>,
}

/// Update staking parameters (admin only)
#[derive(Accounts)]
pub struct UpdateStakingParams<'info> {
    #[account(
        mut,
        seeds = [STAKING_CONFIG_SEED],
        bump = staking_config.bump,
        has_one = authority @ FinovaError::Unauthorized
    )]
    pub staking_config: Account<'info, StakingConfig>,
    
    #[account(
        mut,
        seeds = [REWARD_DISTRIBUTION_SEED],
        bump = reward_distribution.bump
    )]
    pub reward_distribution: Account<'info, RewardDistribution>,
    
    pub authority: Signer<'info>,
}

// Implementation functions
pub fn initialize_staking(
    ctx: Context<InitializeStaking>,
    base_apy: u64,           // Base APY in basis points (e.g., 800 = 8%)
    max_apy: u64,            // Maximum APY in basis points (e.g., 1500 = 15%)
    min_stake_amount: u64,   // Minimum stake amount
    cooldown_period: i64,    // Cooldown period in seconds
) -> Result<()> {
    let staking_config = &mut ctx.accounts.staking_config;
    let staking_pool = &mut ctx.accounts.staking_pool;
    let reward_distribution = &mut ctx.accounts.reward_distribution;
    
    // Initialize staking configuration
    staking_config.authority = ctx.accounts.authority.key();
    staking_config.fin_mint = ctx.accounts.fin_mint.key();
    staking_config.sfin_mint = ctx.accounts.sfin_mint.key();
    staking_config.base_apy = base_apy;
    staking_config.max_apy = max_apy;
    staking_config.min_stake_amount = min_stake_amount;
    staking_config.cooldown_period = cooldown_period;
    staking_config.is_paused = false;
    staking_config.bump = *ctx.bumps.get("staking_config").unwrap();
    
    // Initialize staking pool
    staking_pool.total_staked = 0;
    staking_pool.total_sfin_supply = 0;
    staking_pool.total_rewards_distributed = 0;
    staking_pool.exchange_rate = INITIAL_EXCHANGE_RATE; // 1:1 initially
    staking_pool.last_reward_update = get_current_timestamp();
    staking_pool.vault_bump = *ctx.bumps.get("staking_vault").unwrap();
    staking_pool.sfin_mint_bump = *ctx.bumps.get("sfin_mint").unwrap();
    staking_pool.treasury_bump = *ctx.bumps.get("reward_treasury").unwrap();
    staking_pool.bump = *ctx.bumps.get("staking_pool").unwrap();
    
    // Initialize reward distribution
    reward_distribution.weekly_reward_pool = 0;
    reward_distribution.activity_bonus_pool = 0;
    reward_distribution.loyalty_bonus_pool = 0;
    reward_distribution.performance_bonus_pool = 0;
    reward_distribution.special_event_pool = 0;
    reward_distribution.last_distribution = get_current_timestamp();
    reward_distribution.bump = *ctx.bumps.get("reward_distribution").unwrap();
    
    // Initialize staking tiers
    initialize_staking_tiers(staking_config)?;
    
    msg!("Staking system initialized successfully");
    Ok(())
}

pub fn stake_tokens(
    ctx: Context<StakeTokens>,
    amount: u64,
) -> Result<()> {
    let staking_config = &ctx.accounts.staking_config;
    let staking_pool = &mut ctx.accounts.staking_pool;
    let user_account = &mut ctx.accounts.user_account;
    let stake_account = &mut ctx.accounts.stake_account;
    
    // Validate staking is not paused
    require!(!staking_config.is_paused, FinovaError::StakingPaused);
    
    // Validate minimum stake amount
    require!(
        amount >= staking_config.min_stake_amount,
        FinovaError::InsufficientStakeAmount
    );
    
    // Validate user has sufficient balance
    require!(
        ctx.accounts.user_fin_account.amount >= amount,
        FinovaError::InsufficientBalance
    );
    
    let current_time = get_current_timestamp();
    
    // Initialize stake account if first time staking
    if stake_account.owner == Pubkey::default() {
        stake_account.owner = ctx.accounts.user.key();
        stake_account.total_staked = 0;
        stake_account.total_sfin_minted = 0;
        stake_account.last_stake_time = current_time;
        stake_account.last_reward_claim = current_time;
        stake_account.tier = StakingTier::Basic;
        stake_account.loyalty_multiplier = INITIAL_LOYALTY_MULTIPLIER;
        stake_account.activity_multiplier = INITIAL_ACTIVITY_MULTIPLIER;
        stake_account.pending_rewards = 0;
        stake_account.bump = *ctx.bumps.get("stake_account").unwrap();
    }
    
    // Calculate sFIN to mint based on current exchange rate
    let sfin_to_mint = calculate_sfin_amount(amount, staking_pool.exchange_rate)?;
    
    // Transfer FIN tokens from user to staking vault
    let transfer_ctx = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        Transfer {
            from: ctx.accounts.user_fin_account.to_account_info(),
            to: ctx.accounts.staking_vault.to_account_info(),
            authority: ctx.accounts.user.to_account_info(),
        },
    );
    token::transfer(transfer_ctx, amount)?;
    
    // Mint sFIN tokens to user
    let staking_pool_seeds = &[
        STAKING_POOL_SEED,
        &[staking_pool.bump]
    ];
    let signer_seeds = &[&staking_pool_seeds[..]];
    
    let mint_ctx = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        token::MintTo {
            mint: ctx.accounts.sfin_mint.to_account_info(),
            to: ctx.accounts.user_sfin_account.to_account_info(),
            authority: staking_pool.to_account_info(),
        },
        signer_seeds,
    );
    token::mint_to(mint_ctx, sfin_to_mint)?;
    
    // Update stake account
    stake_account.total_staked = stake_account.total_staked
        .checked_add(amount)
        .ok_or(FinovaError::MathOverflow)?;
    stake_account.total_sfin_minted = stake_account.total_sfin_minted
        .checked_add(sfin_to_mint)
        .ok_or(FinovaError::MathOverflow)?;
    stake_account.last_stake_time = current_time;
    
    // Update staking pool
    staking_pool.total_staked = staking_pool.total_staked
        .checked_add(amount)
        .ok_or(FinovaError::MathOverflow)?;
    staking_pool.total_sfin_supply = staking_pool.total_sfin_supply
        .checked_add(sfin_to_mint)
        .ok_or(FinovaError::MathOverflow)?;
    
    // Update user's mining and XP multipliers
    update_user_multipliers(user_account, stake_account)?;
    
    // Update staking tier if necessary
    update_staking_tier_internal(staking_config, stake_account)?;
    
    // Emit stake event
    emit!(StakeEvent {
        user: ctx.accounts.user.key(),
        amount_staked: amount,
        sfin_minted: sfin_to_mint,
        new_total_staked: stake_account.total_staked,
        tier: stake_account.tier,
        timestamp: current_time,
    });
    
    msg!("Successfully staked {} FIN tokens, minted {} sFIN", amount, sfin_to_mint);
    Ok(())
}

pub fn unstake_tokens(
    ctx: Context<UnstakeTokens>,
    sfin_amount: u64,
) -> Result<()> {
    let staking_config = &ctx.accounts.staking_config;
    let staking_pool = &mut ctx.accounts.staking_pool;
    let user_account = &mut ctx.accounts.user_account;
    let stake_account = &mut ctx.accounts.stake_account;
    
    // Validate staking is not paused
    require!(!staking_config.is_paused, FinovaError::StakingPaused);
    
    // Validate user has sufficient sFIN balance
    require!(
        ctx.accounts.user_sfin_account.amount >= sfin_amount,
        FinovaError::InsufficientBalance
    );
    
    // Validate cooldown period has passed
    let current_time = get_current_timestamp();
    require!(
        current_time >= stake_account.last_stake_time + staking_config.cooldown_period,
        FinovaError::CooldownNotMet
    );
    
    // Calculate FIN amount to return based on current exchange rate
    let fin_to_return = calculate_fin_amount(sfin_amount, staking_pool.exchange_rate)?;
    
    // Validate vault has sufficient balance
    require!(
        ctx.accounts.staking_vault.amount >= fin_to_return,
        FinovaError::InsufficientVaultBalance
    );
    
    // Burn sFIN tokens from user
    let burn_ctx = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        token::Burn {
            mint: ctx.accounts.sfin_mint.to_account_info(),
            from: ctx.accounts.user_sfin_account.to_account_info(),
            authority: ctx.accounts.user.to_account_info(),
        },
    );
    token::burn(burn_ctx, sfin_amount)?;
    
    // Transfer FIN tokens from vault to user
    let staking_pool_seeds = &[
        STAKING_POOL_SEED,
        &[staking_pool.bump]
    ];
    let signer_seeds = &[&staking_pool_seeds[..]];
    
    let transfer_ctx = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        Transfer {
            from: ctx.accounts.staking_vault.to_account_info(),
            to: ctx.accounts.user_fin_account.to_account_info(),
            authority: staking_pool.to_account_info(),
        },
        signer_seeds,
    );
    token::transfer(transfer_ctx, fin_to_return)?;
    
    // Update stake account
    stake_account.total_staked = stake_account.total_staked
        .checked_sub(fin_to_return)
        .ok_or(FinovaError::MathUnderflow)?;
    stake_account.total_sfin_minted = stake_account.total_sfin_minted
        .checked_sub(sfin_amount)
        .ok_or(FinovaError::MathUnderflow)?;
    
    // Update staking pool
    staking_pool.total_staked = staking_pool.total_staked
        .checked_sub(fin_to_return)
        .ok_or(FinovaError::MathUnderflow)?;
    staking_pool.total_sfin_supply = staking_pool.total_sfin_supply
        .checked_sub(sfin_amount)
        .ok_or(FinovaError::MathUnderflow)?;
    
    // Update user's mining and XP multipliers
    update_user_multipliers(user_account, stake_account)?;
    
    // Update staking tier if necessary
    update_staking_tier_internal(staking_config, stake_account)?;
    
    // Emit unstake event
    emit!(UnstakeEvent {
        user: ctx.accounts.user.key(),
        sfin_burned: sfin_amount,
        fin_returned: fin_to_return,
        new_total_staked: stake_account.total_staked,
        tier: stake_account.tier,
        timestamp: current_time,
    });
    
    msg!("Successfully unstaked {} sFIN, returned {} FIN", sfin_amount, fin_to_return);
    Ok(())
}

pub fn claim_staking_rewards(
    ctx: Context<ClaimStakingRewards>,
) -> Result<()> {
    let staking_config = &ctx.accounts.staking_config;
    let staking_pool = &mut ctx.accounts.staking_pool;
    let reward_distribution = &mut ctx.accounts.reward_distribution;
    let user_account = &ctx.accounts.user_account;
    let stake_account = &mut ctx.accounts.stake_account;
    
    // Validate staking is not paused
    require!(!staking_config.is_paused, FinovaError::StakingPaused);
    
    // Calculate rewards to claim
    let current_time = get_current_timestamp();
    let rewards_to_claim = calculate_staking_rewards(
        staking_config,
        staking_pool,
        user_account,
        stake_account,
        current_time,
    )?;
    
    require!(rewards_to_claim > 0, FinovaError::NoRewardsToClaim);
    
    // Validate treasury has sufficient balance
    require!(
        ctx.accounts.reward_treasury.amount >= rewards_to_claim,
        FinovaError::InsufficientTreasuryBalance
    );
    
    // Transfer rewards from treasury to user
    let staking_pool_seeds = &[
        STAKING_POOL_SEED,
        &[staking_pool.bump]
    ];
    let signer_seeds = &[&staking_pool_seeds[..]];
    
    let transfer_ctx = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        Transfer {
            from: ctx.accounts.reward_treasury.to_account_info(),
            to: ctx.accounts.user_fin_account.to_account_info(),
            authority: staking_pool.to_account_info(),
        },
        signer_seeds,
    );
    token::transfer(transfer_ctx, rewards_to_claim)?;
    
    // Update stake account
    stake_account.last_reward_claim = current_time;
    stake_account.pending_rewards = 0;
    stake_account.total_rewards_claimed = stake_account.total_rewards_claimed
        .checked_add(rewards_to_claim)
        .ok_or(FinovaError::MathOverflow)?;
    
    // Update loyalty multiplier based on claim frequency
    update_loyalty_multiplier(stake_account, current_time)?;
    
    // Update staking pool
    staking_pool.total_rewards_distributed = staking_pool.total_rewards_distributed
        .checked_add(rewards_to_claim)
        .ok_or(FinovaError::MathOverflow)?;
    staking_pool.last_reward_update = current_time;
    
    // Update reward distribution
    reward_distribution.last_distribution = current_time;
    
    // Emit claim rewards event
    emit!(ClaimRewardsEvent {
        user: ctx.accounts.user.key(),
        rewards_claimed: rewards_to_claim,
        new_loyalty_multiplier: stake_account.loyalty_multiplier,
        timestamp: current_time,
    });
    
    msg!("Successfully claimed {} FIN rewards", rewards_to_claim);
    Ok(())
}

pub fn update_staking_tier(
    ctx: Context<UpdateStakingTier>,
) -> Result<()> {
    let staking_config = &ctx.accounts.staking_config;
    let user_account = &mut ctx.accounts.user_account;
    let stake_account = &mut ctx.accounts.stake_account;
    
    let old_tier = stake_account.tier;
    update_staking_tier_internal(staking_config, stake_account)?;
    
    // Update user multipliers if tier changed
    if stake_account.tier != old_tier {
        update_user_multipliers(user_account, stake_account)?;
        
        emit!(UpdateStakingTierEvent {
            user: ctx.accounts.user.key(),
            old_tier,
            new_tier: stake_account.tier,
            total_staked: stake_account.total_staked,
            timestamp: get_current_timestamp(),
        });
        
        msg!("Staking tier updated from {:?} to {:?}", old_tier, stake_account.tier);
    }
    
    Ok(())
}

pub fn emergency_pause(
    ctx: Context<EmergencyPause>,
    pause_state: bool,
) -> Result<()> {
    let staking_config = &mut ctx.accounts.staking_config;
    staking_config.is_paused = pause_state;
    
    let action = if pause_state { "paused" } else { "unpaused" };
    msg!("Staking operations {}", action);
    
    Ok(())
}

pub fn update_staking_params(
    ctx: Context<UpdateStakingParams>,
    new_base_apy: Option<u64>,
    new_max_apy: Option<u64>,
    new_min_stake_amount: Option<u64>,
    new_cooldown_period: Option<i64>,
    weekly_reward_pool: Option<u64>,
) -> Result<()> {
    let staking_config = &mut ctx.accounts.staking_config;
    let reward_distribution = &mut ctx.accounts.reward_distribution;
    
    if let Some(base_apy) = new_base_apy {
        require!(base_apy <= MAX_APY_BASIS_POINTS, FinovaError::InvalidAPY);
        staking_config.base_apy = base_apy;
    }
    
    if let Some(max_apy) = new_max_apy {
        require!(max_apy <= MAX_APY_BASIS_POINTS, FinovaError::InvalidAPY);
        require!(max_apy >= staking_config.base_apy, FinovaError::InvalidAPY);
        staking_config.max_apy = max_apy;
    }
    
    if let Some(min_amount) = new_min_stake_amount {
        staking_config.min_stake_amount = min_amount;
    }
    
    if let Some(cooldown) = new_cooldown_period {
        require!(cooldown >= MIN_COOLDOWN_PERIOD, FinovaError::InvalidCooldownPeriod);
        staking_config.cooldown_period = cooldown;
    }
    
    if let Some(reward_pool) = weekly_reward_pool {
        reward_distribution.weekly_reward_pool = reward_pool;
        // Distribute pools according to whitepaper specifications
        reward_distribution.activity_bonus_pool = reward_pool
            .checked_mul(25)
            .and_then(|x| x.checked_div(100))
            .ok_or(FinovaError::MathOverflow)?;
        reward_distribution.loyalty_bonus_pool = reward_pool
            .checked_mul(20)
            .and_then(|x| x.checked_div(100))
            .ok_or(FinovaError::MathOverflow)?;
        reward_distribution.performance_bonus_pool = reward_pool
            .checked_mul(10)
            .and_then(|x| x.checked_div(100))
            .ok_or(FinovaError::MathOverflow)?;
        reward_distribution.special_event_pool = reward_pool
            .checked_mul(5)
            .and_then(|x| x.checked_div(100))
            .ok_or(FinovaError::MathOverflow)?;
    }
    
    msg!("Staking parameters updated successfully");
    Ok(())
}

// Helper functions for staking calculations and utilities

/// Calculate sFIN amount to mint based on FIN amount and exchange rate
fn calculate_sfin_amount(fin_amount: u64, exchange_rate: u64) -> Result<u64> {
    fin_amount
        .checked_mul(PRECISION_FACTOR)
        .and_then(|x| x.checked_div(exchange_rate))
        .ok_or(FinovaError::MathOverflow.into())
}

/// Calculate FIN amount to return based on sFIN amount and exchange rate
fn calculate_fin_amount(sfin_amount: u64, exchange_rate: u64) -> Result<u64> {
    sfin_amount
        .checked_mul(exchange_rate)
        .and_then(|x| x.checked_div(PRECISION_FACTOR))
        .ok_or(FinovaError::MathOverflow.into())
}

/// Calculate comprehensive staking rewards with all multipliers
fn calculate_staking_rewards(
    staking_config: &StakingConfig,
    staking_pool: &StakingPool,
    user_account: &User,
    stake_account: &StakeAccount,
    current_time: i64,
) -> Result<u64> {
    if stake_account.total_staked == 0 {
        return Ok(0);
    }
    
    let time_since_last_claim = current_time - stake_account.last_reward_claim;
    if time_since_last_claim <= 0 {
        return Ok(0);
    }
    
    // Base staking rewards calculation
    let base_apy = get_tier_apy(staking_config, stake_account.tier)?;
    let annual_reward = stake_account.total_staked
        .checked_mul(base_apy)
        .and_then(|x| x.checked_div(BASIS_POINTS))
        .ok_or(FinovaError::MathOverflow)?;
    
    let time_reward = annual_reward
        .checked_mul(time_since_last_claim as u64)
        .and_then(|x| x.checked_div(SECONDS_PER_YEAR))
        .ok_or(FinovaError::MathOverflow)?;
    
    // Apply integrated multipliers from XP, RP, and activity
    let xp_multiplier = calculate_xp_staking_multiplier(user_account.xp_level)?;
    let rp_multiplier = calculate_rp_staking_multiplier(user_account.rp_tier)?;
    let loyalty_multiplier = stake_account.loyalty_multiplier;
    let activity_multiplier = calculate_activity_bonus(user_account, current_time)?;
    
    // Compound all multipliers
    let total_multiplier = xp_multiplier
        .checked_mul(rp_multiplier)
        .and_then(|x| x.checked_mul(loyalty_multiplier))
        .and_then(|x| x.checked_mul(activity_multiplier))
        .and_then(|x| x.checked_div(PRECISION_FACTOR.pow(3)))
        .ok_or(FinovaError::MathOverflow)?;
    
    let final_reward = time_reward
        .checked_mul(total_multiplier)
        .and_then(|x| x.checked_div(PRECISION_FACTOR))
        .ok_or(FinovaError::MathOverflow)?;
    
    // Add any pending rewards
    let total_reward = final_reward
        .checked_add(stake_account.pending_rewards)
        .ok_or(FinovaError::MathOverflow)?;
    
    Ok(total_reward)
}

/// Calculate XP-based staking multiplier
fn calculate_xp_staking_multiplier(xp_level: u32) -> Result<u64> {
    // XP_Level_Multiplier = 1.0x + (XP_Level / 100)
    let multiplier = PRECISION_FACTOR
        .checked_add(
            (xp_level as u64)
                .checked_mul(PRECISION_FACTOR)
                .and_then(|x| x.checked_div(100))
                .ok_or(FinovaError::MathOverflow)?
        )
        .ok_or(FinovaError::MathOverflow)?;
    
    // Cap at maximum XP multiplier (5.0x based on whitepaper)
    Ok(multiplier.min(5 * PRECISION_FACTOR))
}

/// Calculate RP-based staking multiplier
fn calculate_rp_staking_multiplier(rp_tier: u8) -> Result<u64> {
    // RP_Tier_Bonus = 1.0x + (RP_Tier × 0.2)
    let tier_bonus = (rp_tier as u64)
        .checked_mul(PRECISION_FACTOR)
        .and_then(|x| x.checked_mul(20))
        .and_then(|x| x.checked_div(100))
        .ok_or(FinovaError::MathOverflow)?;
        
    let multiplier = PRECISION_FACTOR
        .checked_add(tier_bonus)
        .ok_or(FinovaError::MathOverflow)?;
    
    // Cap at maximum RP multiplier (3.0x based on whitepaper)
    Ok(multiplier.min(3 * PRECISION_FACTOR))
}

/// Calculate activity-based bonus multiplier
fn calculate_activity_bonus(user_account: &User, current_time: i64) -> Result<u64> {
    let days_since_last_activity = (current_time - user_account.last_activity_time) / SECONDS_PER_DAY;
    
    if days_since_last_activity > 7 {
        // No activity bonus if inactive for more than a week
        return Ok(PRECISION_FACTOR);
    }
    
    // Activity_Bonus = 1.0x + (Daily_Activity_Score × 0.1)
    let activity_score = user_account.daily_activity_score.min(200); // Cap at 2.0x
    let bonus = activity_score as u64
        .checked_mul(PRECISION_FACTOR)
        .and_then(|x| x.checked_div(1000))
        .ok_or(FinovaError::MathOverflow)?;
    
    let multiplier = PRECISION_FACTOR
        .checked_add(bonus)
        .ok_or(FinovaError::MathOverflow)?;
    
    Ok(multiplier.min(2 * PRECISION_FACTOR))
}

/// Get APY based on staking tier
fn get_tier_apy(staking_config: &StakingConfig, tier: StakingTier) -> Result<u64> {
    let base_apy = staking_config.base_apy;
    let tier_multiplier = match tier {
        StakingTier::Basic => 100,      // 8% base
        StakingTier::Silver => 125,     // 10% (+25%)
        StakingTier::Gold => 150,       // 12% (+50%)
        StakingTier::Platinum => 175,   // 14% (+75%)
        StakingTier::Diamond => 187,    // 15% (+87.5%)
    };
    
    base_apy
        .checked_mul(tier_multiplier)
        .and_then(|x| x.checked_div(100))
        .ok_or(FinovaError::MathOverflow.into())
}

/// Update user's mining and XP multipliers based on staking
fn update_user_multipliers(user_account: &mut User, stake_account: &StakeAccount) -> Result<()> {
    // Update mining multiplier based on staking tier
    user_account.mining_multiplier = get_tier_mining_multiplier(stake_account.tier)?;
    
    // Update XP multiplier based on staking tier
    user_account.xp_multiplier = get_tier_xp_multiplier(stake_account.tier)?;
    
    // Update RP bonus based on staking tier
    user_account.rp_bonus = get_tier_rp_bonus(stake_account.tier)?;
    
    Ok(())
}

/// Get mining multiplier based on staking tier
fn get_tier_mining_multiplier(tier: StakingTier) -> Result<u64> {
    let multiplier = match tier {
        StakingTier::Basic => 120,      // +20%
        StakingTier::Silver => 135,     // +35%
        StakingTier::Gold => 150,       // +50%
        StakingTier::Platinum => 175,   // +75%
        StakingTier::Diamond => 200,    // +100%
    };
    Ok(multiplier * PRECISION_FACTOR / 100)
}

/// Get XP multiplier based on staking tier
fn get_tier_xp_multiplier(tier: StakingTier) -> Result<u64> {
    let multiplier = match tier {
        StakingTier::Basic => 110,      // +10%
        StakingTier::Silver => 120,     // +20%
        StakingTier::Gold => 130,       // +30%
        StakingTier::Platinum => 150,   // +50%
        StakingTier::Diamond => 175,    // +75%
    };
    Ok(multiplier * PRECISION_FACTOR / 100)
}

/// Get RP bonus based on staking tier
fn get_tier_rp_bonus(tier: StakingTier) -> Result<u64> {
    let bonus = match tier {
        StakingTier::Basic => 5,        // +5%
        StakingTier::Silver => 10,      // +10%
        StakingTier::Gold => 20,        // +20%
        StakingTier::Platinum => 35,    // +35%
        StakingTier::Diamond => 50,     // +50%
    };
    Ok(bonus * PRECISION_FACTOR / 100)
}

/// Update staking tier based on current stake amount
fn update_staking_tier_internal(
    staking_config: &StakingConfig,
    stake_account: &mut StakeAccount,
) -> Result<()> {
    let new_tier = determine_staking_tier(stake_account.total_staked, staking_config)?;
    stake_account.tier = new_tier;
    Ok(())
}

/// Determine staking tier based on stake amount
fn determine_staking_tier(stake_amount: u64, staking_config: &StakingConfig) -> Result<StakingTier> {
    // Convert amounts to FIN tokens (assuming 9 decimals)
    let amount_in_fin = stake_amount / 10u64.pow(9);
    
    let tier = if amount_in_fin >= 10_000 {
        StakingTier::Diamond    // 10,000+ FIN
    } else if amount_in_fin >= 5_000 {
        StakingTier::Platinum   // 5,000-9,999 FIN
    } else if amount_in_fin >= 1_000 {
        StakingTier::Gold       // 1,000-4,999 FIN
    } else if amount_in_fin >= 500 {
        StakingTier::Silver     // 500-999 FIN
    } else {
        StakingTier::Basic      // 100-499 FIN
    };
    
    Ok(tier)
}

/// Update loyalty multiplier based on staking duration and claim frequency
fn update_loyalty_multiplier(stake_account: &mut StakeAccount, current_time: i64) -> Result<()> {
    let staking_duration_months = (current_time - stake_account.last_stake_time) / (SECONDS_PER_DAY * 30);
    
    // Loyalty_Bonus = 1.0x + (Staking_Duration_Months × 0.05)
    let loyalty_bonus = (staking_duration_months as u64)
        .checked_mul(5)
        .and_then(|x| x.checked_mul(PRECISION_FACTOR))
        .and_then(|x| x.checked_div(100))
        .ok_or(FinovaError::MathOverflow)?;
    
    stake_account.loyalty_multiplier = PRECISION_FACTOR
        .checked_add(loyalty_bonus)
        .ok_or(FinovaError::MathOverflow)?
        .min(2 * PRECISION_FACTOR); // Cap at 2.0x
    
    Ok(())
}

/// Initialize staking tier configurations
fn initialize_staking_tiers(staking_config: &mut StakingConfig) -> Result<()> {
    // Set tier thresholds based on whitepaper specifications
    staking_config.tier_thresholds = [
        100 * 10u64.pow(9),      // Basic: 100-499 FIN
        500 * 10u64.pow(9),      // Silver: 500-999 FIN  
        1_000 * 10u64.pow(9),    // Gold: 1,000-4,999 FIN
        5_000 * 10u64.pow(9),    // Platinum: 5,000-9,999 FIN
        10_000 * 10u64.pow(9),   // Diamond: 10,000+ FIN
    ];
    
    Ok(())
}

/// Calculate compound interest for auto-compounding sFIN
pub fn calculate_compound_rewards(
    principal: u64,
    rate: u64,
    time_periods: u64,
    compounding_frequency: u64,
) -> Result<u64> {
    // A = P(1 + r/n)^(nt)
    // Where: P = principal, r = annual rate, n = compounding frequency, t = time in years
    
    let rate_per_period = rate
        .checked_div(compounding_frequency)
        .ok_or(FinovaError::MathOverflow)?;
    
    let base = PRECISION_FACTOR
        .checked_add(rate_per_period)
        .ok_or(FinovaError::MathOverflow)?;
    
    let exponent = time_periods
        .checked_mul(compounding_frequency)
        .ok_or(FinovaError::MathOverflow)?;
    
    // Approximate compound interest using Taylor series for gas efficiency
    let compound_factor = approximate_power(base, exponent as u32)?;
    
    principal
        .checked_mul(compound_factor)
        .and_then(|x| x.checked_div(PRECISION_FACTOR))
        .ok_or(FinovaError::MathOverflow.into())
}

/// Approximate power function using Taylor series expansion
fn approximate_power(base: u64, exponent: u32) -> Result<u64> {
    if exponent == 0 {
        return Ok(PRECISION_FACTOR);
    }
    
    if exponent == 1 {
        return Ok(base);
    }
    
    // Use binary exponentiation for efficiency
    let mut result = PRECISION_FACTOR;
    let mut base_power = base;
    let mut exp = exponent;
    
    while exp > 0 {
        if exp % 2 == 1 {
            result = result
                .checked_mul(base_power)
                .and_then(|x| x.checked_div(PRECISION_FACTOR))
                .ok_or(FinovaError::MathOverflow)?;
        }
        base_power = base_power
            .checked_mul(base_power)
            .and_then(|x| x.checked_div(PRECISION_FACTOR))
            .ok_or(FinovaError::MathOverflow)?;
        exp /= 2;
    }
    
    Ok(result)
}

/// Validate staking parameters
pub fn validate_staking_parameters(
    base_apy: u64,
    max_apy: u64,
    min_stake_amount: u64,
    cooldown_period: i64,
) -> Result<()> {
    require!(base_apy <= MAX_APY_BASIS_POINTS, FinovaError::InvalidAPY);
    require!(max_apy <= MAX_APY_BASIS_POINTS, FinovaError::InvalidAPY);
    require!(max_apy >= base_apy, FinovaError::InvalidAPY);
    require!(min_stake_amount > 0, FinovaError::InvalidMinStakeAmount);
    require!(cooldown_period >= MIN_COOLDOWN_PERIOD, FinovaError::InvalidCooldownPeriod);
    
    Ok(())
}

/// Update exchange rate based on rewards and staking activity
pub fn update_exchange_rate(staking_pool: &mut StakingPool, rewards_added: u64) -> Result<()> {
    if staking_pool.total_sfin_supply == 0 {
        return Ok(());
    }
    
    let new_total_value = staking_pool.total_staked
        .checked_add(rewards_added)
        .ok_or(FinovaError::MathOverflow)?;
    
    // Exchange rate = (Total FIN Value * PRECISION_FACTOR) / Total sFIN Supply
    staking_pool.exchange_rate = new_total_value
        .checked_mul(PRECISION_FACTOR)
        .and_then(|x| x.checked_div(staking_pool.total_sfin_supply))
        .ok_or(FinovaError::MathOverflow)?;
    
    Ok(())
}

/// Calculate slashing penalty for early unstaking (if implemented)
pub fn calculate_slashing_penalty(
    stake_amount: u64,
    time_staked: i64,
    required_time: i64,
) -> Result<u64> {
    if time_staked >= required_time {
        return Ok(0);
    }
    
    // Penalty = (Required_Time - Time_Staked) / Required_Time * Base_Penalty_Rate
    let time_ratio = (required_time - time_staked) as u64;
    let penalty_rate = time_ratio
        .checked_mul(SLASHING_PENALTY_RATE)
        .and_then(|x| x.checked_div(required_time as u64))
        .ok_or(FinovaError::MathOverflow)?;
    
    stake_amount
        .checked_mul(penalty_rate)
        .and_then(|x| x.checked_div(BASIS_POINTS))
        .ok_or(FinovaError::MathOverflow.into())
}

/// Rebalance reward pools based on utilization
pub fn rebalance_reward_pools(
    reward_distribution: &mut RewardDistribution,
    utilization_metrics: &UtilizationMetrics,
) -> Result<()> {
    let total_pool = reward_distribution.weekly_reward_pool;
    
    // Adjust pool allocations based on actual utilization
    let activity_usage_ratio = utilization_metrics.activity_pool_usage
        .checked_mul(100)
        .and_then(|x| x.checked_div(reward_distribution.activity_bonus_pool.max(1)))
        .unwrap_or(100);
    
    // Rebalance based on usage patterns
    if activity_usage_ratio > 150 {
        // High activity usage, increase activity pool
        reward_distribution.activity_bonus_pool = total_pool
            .checked_mul(30)
            .and_then(|x| x.checked_div(100))
            .ok_or(FinovaError::MathOverflow)?;
        reward_distribution.loyalty_bonus_pool = total_pool
            .checked_mul(15)
            .and_then(|x| x.checked_div(100))
            .ok_or(FinovaError::MathOverflow)?;
    } else if activity_usage_ratio < 50 {
        // Low activity usage, decrease activity pool
        reward_distribution.activity_bonus_pool = total_pool
            .checked_mul(20)
            .and_then(|x| x.checked_div(100))
            .ok_or(FinovaError::MathOverflow)?;
        reward_distribution.loyalty_bonus_pool = total_pool
            .checked_mul(25)
            .and_then(|x| x.checked_div(100))
            .ok_or(FinovaError::MathOverflow)?;
    }
    
    Ok(())
}

// Structure for tracking pool utilization metrics
#[derive(Debug, Clone)]
pub struct UtilizationMetrics {
    pub activity_pool_usage: u64,
    pub loyalty_pool_usage: u64,
    pub performance_pool_usage: u64,
    pub special_event_usage: u64,
}

/// Emergency withdrawal function (admin only, for security issues)
pub fn emergency_withdraw(
    ctx: Context<EmergencyWithdraw>,
    amount: u64,
) -> Result<()> {
    let staking_config = &ctx.accounts.staking_config;
    
    // Only allow if system is paused and caller is authority
    require!(staking_config.is_paused, FinovaError::NotInEmergencyMode);
    require!(ctx.accounts.authority.key() == staking_config.authority, FinovaError::Unauthorized);
    
    // Emergency withdrawal logic would go here
    // This is typically used for security incidents or contract upgrades
    
    msg!("Emergency withdrawal executed: {} tokens", amount);
    Ok(())
}

#[derive(Accounts)]
pub struct EmergencyWithdraw<'info> {
    #[account(
        mut,
        seeds = [STAKING_CONFIG_SEED],
        bump = staking_config.bump
    )]
    pub staking_config: Account<'info, StakingConfig>,
    
    #[account(
        mut,
        seeds = [STAKING_VAULT_SEED],
        bump
    )]
    pub staking_vault: Account<'info, TokenAccount>,
    
    #[account(mut)]
    pub emergency_destination: Account<'info, TokenAccount>,
    
    pub authority: Signer<'info>,
    pub token_program: Program<'info, Token>,
}
