//! Instructions for staking $FIN tokens.

use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount, Transfer};
use crate::state::{StakingAccount, StakingVault, NetworkState};

/// # Context for staking tokens
///
/// This context handles the staking of $FIN tokens. It transfers tokens
/// from the user's token account to the central staking vault token account.
#[derive(Accounts)]
pub struct Stake<'info> {
    /// The user who is staking tokens.
    #[account(mut)]
    pub authority: Signer<'info>,

    /// The user's staking account, which tracks their staked amount.
    #[account(
        mut,
        seeds = [b"staking", authority.key().as_ref()],
        bump,
        constraint = staking_account.authority == authority.key()
    )]
    pub staking_account: Account<'info, StakingAccount>,

    /// The user's token account from which to transfer the tokens.
    #[account(
        mut,
        constraint = user_token_account.owner == authority.key(),
        constraint = user_token_account.mint == fin_mint.key()
    )]
    pub user_token_account: Account<'info, TokenAccount>,

    /// The mint of the $FIN token.
    #[account(
        constraint = fin_mint.key() == network_state.fin_token_mint
    )]
    pub fin_mint: Account<'info, Mint>,

    /// The metadata account for the staking vault.
    #[account(
        init_if_needed,
        payer = authority,
        space = StakingVault::SIZE,
        seeds = [b"staking_vault_meta"],
        bump
    )]
    pub staking_vault_meta: Account<'info, StakingVault>,

    /// The token account that holds all staked tokens.
    #[account(
        init_if_needed,
        payer = authority,
        token::mint = fin_mint,
        token::authority = staking_vault_meta, // The metadata PDA is the authority
        seeds = [b"staking_vault_tokens"],
        bump
    )]
    pub staking_vault_tokens: Account<'info, TokenAccount>,

    /// The global network state.
    #[account(seeds = [b"network_state"], bump)]
    pub network_state: Account<'info, NetworkState>,

    /// The SPL Token program.
    pub token_program: Program<'info, Token>,
    /// The system program, required for account creation.
    pub system_program: Program<'info, System>,
    /// The rent sysvar, required for account creation.
    pub rent: Sysvar<'info, Rent>,
}

/// # Context for unstaking tokens
#[derive(Accounts)]
pub struct Unstake<'info> {
    /// The user who is unstaking tokens.
    #[account(mut)]
    pub authority: Signer<'info>,

    /// The user's staking account.
    #[account(
        mut,
        seeds = [b"staking", authority.key().as_ref()],
        bump,
        constraint = staking_account.staked_amount > 0,
        constraint = staking_account.authority == authority.key()
    )]
    pub staking_account: Account<'info, StakingAccount>,

    /// The user's token account where the tokens will be returned.
    #[account(
        mut,
        constraint = user_token_account.owner == authority.key(),
        constraint = user_token_account.mint == fin_mint.key()
    )]
    pub user_token_account: Account<'info, TokenAccount>,

    /// The mint of the $FIN token.
    #[account(constraint = fin_mint.key() == network_state.fin_token_mint)]
    pub fin_mint: Account<'info, Mint>,

    /// The metadata account for the staking vault, which will sign the transfer.
    #[account(seeds = [b"staking_vault_meta"], bump)]
    pub staking_vault_meta: Account<'info, StakingVault>,

    /// The token account that holds all staked tokens.
    #[account(
        mut,
        seeds = [b"staking_vault_tokens"],
        bump,
        constraint = staking_vault_tokens.owner == staking_vault_meta.key()
    )]
    pub staking_vault_tokens: Account<'info, TokenAccount>,

    /// The global network state.
    #[account(seeds = [b"network_state"], bump)]
    pub network_state: Account<'info, NetworkState>,

    /// The SPL Token program.
    pub token_program: Program<'info, Token>,
}


/// # Handler for the `stake` instruction
pub fn stake_handler(ctx: Context<Stake>, amount: u64) -> Result<()> {
    require!(amount > 0, crate::errors::FinovaError::InvalidAmount);

    // Transfer tokens from user to the vault token account
    let cpi_accounts = Transfer {
        from: ctx.accounts.user_token_account.to_account_info(),
        to: ctx.accounts.staking_vault_tokens.to_account_info(),
        authority: ctx.accounts.authority.to_account_info(),
    };
    let cpi_program = ctx.accounts.token_program.to_account_info();
    let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
    token::transfer(cpi_ctx, amount)?;

    // Update user's staking account state
    let staking_account = &mut ctx.accounts.staking_account;
    staking_account.staked_amount = staking_account.staked_amount.saturating_add(amount);
    staking_account.last_staked_at = Clock::get()?.unix_timestamp;

    // Update vault metadata state
    let staking_vault_meta = &mut ctx.accounts.staking_vault_meta;
    staking_vault_meta.total_staked = staking_vault_meta.total_staked.saturating_add(amount);
    if staking_vault_meta.bump == 0 {
        staking_vault_meta.bump = ctx.bumps.staking_vault_meta;
    }

    msg!("User {} staked {} tokens.", ctx.accounts.authority.key(), amount);
    Ok(())
}

/// # Handler for the `unstake` instruction
pub fn unstake_handler(ctx: Context<Unstake>, amount: u64) -> Result<()> {
    let staking_account = &mut ctx.accounts.staking_account;
    require!(amount > 0, crate::errors::FinovaError::InvalidAmount);
    require!(staking_account.staked_amount >= amount, crate::errors::FinovaError::InsufficientStakedAmount);

    // Transfer tokens from vault back to user
    let seeds = &[
        b"staking_vault_meta",
        &[ctx.bumps.staking_vault_meta],
    ];
    let signer_seeds = &[&seeds[..]];

    let cpi_accounts = Transfer {
        from: ctx.accounts.staking_vault_tokens.to_account_info(),
        to: ctx.accounts.user_token_account.to_account_info(),
        authority: ctx.accounts.staking_vault_meta.to_account_info(),
    };
    let cpi_program = ctx.accounts.token_program.to_account_info();
    let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer_seeds);
    token::transfer(cpi_ctx, amount)?;

    // Update user's staking account state
    staking_account.staked_amount = staking_account.staked_amount.saturating_sub(amount);

    // Update vault metadata state
    let staking_vault_meta = &mut ctx.accounts.staking_vault_meta;
    staking_vault_meta.total_staked = staking_vault_meta.total_staked.saturating_sub(amount);

    msg!("User {} unstaked {} tokens.", ctx.accounts.authority.key(), amount);
    Ok(())
}
