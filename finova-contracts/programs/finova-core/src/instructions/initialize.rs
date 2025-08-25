//! Instruction: Initialize Network
//!
//! This instruction is called once to set up the global state of the Finova Network.
//! It creates the `NetworkState` and `RewardPool` accounts.

use anchor_lang::prelude::*;
use crate::state::{NetworkState, NetworkConfig, RewardPool};

/// # Context for initializing the network
///
/// This context defines the accounts required to initialize the Finova Network.
/// It creates the `NetworkState` and `RewardPool` PDAs.
#[derive(Accounts)]
pub struct InitializeNetwork<'info> {
    /// The account that will pay for the account creation.
    /// This will also be set as the initial admin.
    #[account(mut)]
    pub authority: Signer<'info>,

    /// The `NetworkState` PDA, which will be created and initialized.
    #[account(
        init,
        payer = authority,
        space = NetworkState::SIZE,
        seeds = [b"network_state"],
        bump
    )]
    pub network_state: Account<'info, NetworkState>,

    /// The `RewardPool` PDA, which will be created and initialized.
    #[account(
        init,
        payer = authority,
        space = RewardPool::SIZE,
        seeds = [b"reward_pool"],
        bump
    )]
    pub reward_pool: Account<'info, RewardPool>,

    /// The system program, required by Anchor for account creation.
    pub system_program: Program<'info, System>,
}

/// # Handler for the `initialize_network` instruction
///
/// This function executes the logic to initialize the network.
///
/// ## Arguments
///
/// * `ctx` - The context containing the required accounts.
/// * `admin` - The public key of the account to be designated as the network admin.
/// * `fin_token_mint` - The public key of the $FIN token's mint account.
pub fn handler(ctx: Context<InitializeNetwork>, admin: Pubkey, fin_token_mint: Pubkey) -> Result<()> {
    // Initialize the NetworkState account with the provided parameters.
    let network_state = &mut ctx.accounts.network_state;
    network_state.admin = admin;
    network_state.fin_token_mint = fin_token_mint;
    network_state.is_paused = false;
    network_state.total_users = 0;
    network_state.total_guilds = 0;
    network_state.next_guild_id = 1;
    network_state.next_proposal_id = 1;
    network_state.config = NetworkConfig {
        min_guild_creation_level: 10, // Default: Level 10 to create a guild
        min_proposal_weight: 1000,    // Default: 1000 governance weight to create proposal
    };
    network_state.bump = ctx.bumps.network_state;

    // Initialize the RewardPool account.
    let reward_pool = &mut ctx.accounts.reward_pool;
    reward_pool.total_deposited = 0;
    reward_pool.total_distributed = 0;
    reward_pool.bump = ctx.bumps.reward_pool;

    msg!("Finova Network initialized successfully!");
    msg!("Admin: {}", admin);
    msg!("$FIN Token Mint: {}", fin_token_mint);

    Ok(())
}
