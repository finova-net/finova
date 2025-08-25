//! Instructions for mining operations.

use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount, Transfer};
use crate::state::{MiningAccount, NetworkState, RewardPool};

/// # Context for starting a mining session
#[derive(Accounts)]
pub struct StartMining<'info> {
    /// The user starting the mining session.
    #[account(mut)]
    pub authority: Signer<'info>,

    /// The user's mining account.
    #[account(
        mut,
        seeds = [b"mining", authority.key().as_ref()],
        bump,
        constraint = mining_account.authority == authority.key()
    )]
    pub mining_account: Account<'info, MiningAccount>,

    /// The global network state.
    #[account(
        seeds = [b"network_state"],
        bump,
        constraint = !network_state.is_paused
    )]
    pub network_state: Account<'info, NetworkState>,
}

/// # Context for claiming mining rewards
#[derive(Accounts)]
pub struct ClaimRewards<'info> {
    /// The user claiming rewards.
    #[account(mut)]
    pub authority: Signer<'info>,

    /// The user's mining account.
    #[account(
        mut,
        seeds = [b"mining", authority.key().as_ref()],
        bump,
        constraint = mining_account.authority == authority.key()
    )]
    pub mining_account: Account<'info, MiningAccount>,

    /// The user's token account where rewards will be sent.
    #[account(
        mut,
        constraint = user_token_account.owner == authority.key(),
        constraint = user_token_account.mint == network_state.fin_token_mint
    )]
    pub user_token_account: Account<'info, TokenAccount>,

    /// The global network state.
    #[account(
        seeds = [b"network_state"],
        bump
    )]
    pub network_state: Account<'info, NetworkState>,

    /// The central reward pool.
    #[account(
        mut,
        seeds = [b"reward_pool"],
        bump
    )]
    pub reward_pool: Account<'info, RewardPool>,

    /// The PDA that holds the reward tokens.
    /// This account is the authority for the `reward_vault`.
    /// CHECK: The authority of the vault is checked via the seeds of the CPI call.
    #[account(
        mut,
        seeds = [b"reward_vault_authority"],
        bump
    )]
    pub reward_vault_authority: AccountInfo<'info>,

    /// The token account that holds the mining rewards.
    #[account(
        mut,
        constraint = reward_vault.owner == reward_vault_authority.key(),
        constraint = reward_vault.mint == network_state.fin_token_mint
    )]
    pub reward_vault: Account<'info, TokenAccount>,

    /// The SPL Token program.
    pub token_program: Program<'info, Token>,
}


/// # Handler for the `start_mining` instruction
pub fn start_handler(ctx: Context<StartMining>) -> Result<()> {
    let mining_account = &mut ctx.accounts.mining_account;
    let clock = Clock::get()?;

    require!(!mining_account.is_active, crate::errors::FinovaError::MiningAlreadyActive);

    mining_account.is_active = true;
    mining_account.session_started_at = clock.unix_timestamp;

    msg!("Mining session started for user: {}", ctx.accounts.authority.key());
    Ok(())
}

/// # Handler for the `claim_rewards` instruction
pub fn claim_handler(ctx: Context<ClaimRewards>) -> Result<()> {
    let mining_account = &mut ctx.accounts.mining_account;
    let clock = Clock::get()?;

    require!(mining_account.is_active, crate::errors::FinovaError::MiningNotActive);

    // --- Reward Calculation ---
    // This is a simplified placeholder. The actual implementation would involve
    // the complex formulas from the whitepaper (XP, RP, multipliers, etc.).
    let seconds_elapsed = clock.unix_timestamp.saturating_sub(mining_account.last_claim_at);
    require!(seconds_elapsed > 0, crate::errors::FinovaError::NoRewardsToClaim);

    // Placeholder: 10 micro-FIN per second
    let rewards_to_claim = (seconds_elapsed as u64).saturating_mul(10);

    require!(rewards_to_claim > 0, crate::errors::FinovaError::NoRewardsToClaim);
    require!(ctx.accounts.reward_vault.amount >= rewards_to_claim, crate::errors::FinovaError::InsufficientRewardsPool);

    // --- Transfer Rewards ---
    let seeds = &[
        b"reward_vault_authority",
        &[ctx.bumps.reward_vault_authority],
    ];
    let signer_seeds = &[&seeds[..]];

    let cpi_accounts = Transfer {
        from: ctx.accounts.reward_vault.to_account_info(),
        to: ctx.accounts.user_token_account.to_account_info(),
        authority: ctx.accounts.reward_vault_authority.to_account_info(),
    };
    let cpi_program = ctx.accounts.token_program.to_account_info();
    let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer_seeds);

    token::transfer(cpi_ctx, rewards_to_claim)?;

    // --- Update State ---
    mining_account.last_claim_at = clock.unix_timestamp;
    mining_account.total_mined = mining_account.total_mined.saturating_add(rewards_to_claim);
    // For simplicity, we make the session inactive after claiming. User must start a new one.
    mining_account.is_active = false;

    let reward_pool = &mut ctx.accounts.reward_pool;
    reward_pool.total_distributed = reward_pool.total_distributed.saturating_add(rewards_to_claim);

    msg!("User {} claimed {} rewards.", ctx.accounts.authority.key(), rewards_to_claim);

    Ok(())
}
