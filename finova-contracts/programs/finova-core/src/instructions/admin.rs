//! Instructions for administrative actions.

use anchor_lang::prelude::*;
use crate::state::{NetworkState, NetworkConfig};
use crate::errors::FinovaError;

/// # Context for setting the paused state of the program
#[derive(Accounts)]
pub struct SetPaused<'info> {
    /// The administrator of the network.
    pub admin: Signer<'info>,

    /// The network state account to be modified.
    #[account(
        mut,
        seeds = [b"network_state"],
        bump,
        constraint = network_state.admin == admin.key()
    )]
    pub network_state: Account<'info, NetworkState>,
}

/// # Context for updating network configuration
#[derive(Accounts)]
pub struct UpdateNetworkConfig<'info> {
    /// The administrator of the network.
    pub admin: Signer<'info>,

    /// The network state account to be modified.
    #[account(
        mut,
        seeds = [b"network_state"],
        bump,
        constraint = network_state.admin == admin.key()
    )]
    pub network_state: Account<'info, NetworkState>,
}


/// # Handler for the `set_paused` instruction
///
/// This allows the admin to pause or unpause the entire program.
pub fn set_paused_handler(ctx: Context<SetPaused>, paused: bool) -> Result<()> {
    ctx.accounts.network_state.is_paused = paused;
    if paused {
        msg!("Program has been paused by the admin.");
    } else {
        msg!("Program has been unpaused by the admin.");
    }
    Ok(())
}

/// # Handler for the `update_network_config` instruction
///
/// This allows the admin to update the configurable parameters of the network.
pub fn update_network_config_handler(ctx: Context<UpdateNetworkConfig>, config: NetworkConfig) -> Result<()> {
    ctx.accounts.network_state.config = config;
    msg!("Network configuration has been updated by the admin.");
    Ok(())
}
