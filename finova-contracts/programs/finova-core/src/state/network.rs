//! Network-wide state and configuration accounts.

use anchor_lang::prelude::*;

/// # NetworkState
///
/// Holds the global configuration and state for the entire Finova Network program.
/// There is only one instance of this account, stored as a PDA.
///
/// Seeds: `[b"network_state"]`
#[account]
#[derive(Default, Debug)]
pub struct NetworkState {
    /// The administrator account with the authority to perform privileged actions.
    pub admin: Pubkey,
    /// The mint address of the $FIN token.
    pub fin_token_mint: Pubkey,
    /// A flag to pause or resume all program activity in case of an emergency.
    pub is_paused: bool,
    /// The total number of registered users.
    pub total_users: u64,
    /// The total number of created guilds.
    pub total_guilds: u64,
    /// The ID to be assigned to the next created guild.
    pub next_guild_id: u64,
    /// The ID to be assigned to the next submitted proposal.
    pub next_proposal_id: u64,
    /// Configuration for network parameters.
    pub config: NetworkConfig,
    /// Bump seed for the PDA.
    pub bump: u8,
}

impl NetworkState {
    /// Static size for account initialization.
    /// `admin` + `fin_token_mint` + `is_paused` + `total_users` + `total_guilds` + `next_guild_id` + `next_proposal_id` + `config` + `bump`
    pub const SIZE: usize = 8 + 32 + 32 + 1 + 8 + 8 + 8 + 8 + NetworkConfig::SIZE + 1;
}

/// # NetworkConfig
///
/// A struct to hold various configurable parameters for the network.
/// This is embedded within `NetworkState`.
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default, Debug)]
pub struct NetworkConfig {
    /// The minimum XP level required to create a guild.
    pub min_guild_creation_level: u32,
    /// The minimum governance weight required to submit a proposal.
    pub min_proposal_weight: u64,
}

impl NetworkConfig {
    /// Static size for account initialization.
    pub const SIZE: usize = 4 + 8;
}

/// # RewardPool
///
/// Holds the central pool of $FIN tokens used for distributing mining rewards.
/// There is only one instance of this account, stored as a PDA.
///
/// Seeds: `[b"reward_pool"]`
#[account]
#[derive(Default, Debug)]
pub struct RewardPool {
    /// The total amount of rewards ever deposited into the pool.
    pub total_deposited: u64,
    /// The total amount of rewards distributed from the pool.
    pub total_distributed: u64,
    /// Bump seed for the PDA.
    pub bump: u8,
}

impl RewardPool {
    /// Static size for account initialization.
    /// `total_deposited` + `total_distributed` + `bump`
    pub const SIZE: usize = 8 + 8 + 8 + 1;
}
