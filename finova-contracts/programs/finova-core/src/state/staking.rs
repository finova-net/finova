//! User-specific state account for staking activities.

use anchor_lang::prelude::*;

/// # StakingAccount
///
/// Stores all data related to a user's staking of $FIN tokens.
/// Each user has one `StakingAccount` PDA.
///
/// Seeds: `[b"staking", authority.key().as_ref()]`
#[account]
#[derive(Default, Debug)]
pub struct StakingAccount {
    /// The user's wallet public key.
    pub authority: Pubkey,
    /// The total amount of $FIN tokens currently staked by the user.
    pub staked_amount: u64,
    /// The timestamp when the user last staked or modified their stake.
    pub last_staked_at: i64,
    /// The total rewards claimed from staking.
    pub rewards_claimed: u64,
    /// Bump seed for the PDA.
    pub bump: u8,
}

impl StakingAccount {
    /// Static size for account initialization.
    /// `authority` + `staked_amount` + `last_staked_at` + `rewards_claimed` + `bump`
    pub const SIZE: usize = 8 + 32 + 8 + 8 + 8 + 1;
}

/// # StakingVault
///
/// A global account that holds all staked $FIN tokens.
/// There is only one instance of this account, a PDA.
///
/// Seeds: `[b"staking_vault"]`
#[account]
#[derive(Default, Debug)]
pub struct StakingVault {
    /// The total amount of tokens held in the vault.
    pub total_staked: u64,
    /// Bump seed for the PDA.
    pub bump: u8,
}

impl StakingVault {
    /// Static size for account initialization.
    /// `total_staked` + `bump`
    pub const SIZE: usize = 8 + 8 + 1;
}
