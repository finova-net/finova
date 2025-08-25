//! User-specific state account for mining activities.

use anchor_lang::prelude::*;

/// # MiningAccount
///
/// Stores all data related to a user's mining status and rewards.
/// Each user has one `MiningAccount` PDA.
///
/// Seeds: `[b"mining", authority.key().as_ref()]`
#[account]
#[derive(Default, Debug)]
pub struct MiningAccount {
    /// The user's wallet public key.
    pub authority: Pubkey,
    /// The timestamp of the last time the user claimed their mining rewards.
    pub last_claim_at: i64,
    /// The total amount of $FIN tokens the user has mined over their lifetime.
    pub total_mined: u64,
    /// The base mining rate for the user, determined by the network phase when they joined.
    pub base_rate: u64, // Stored in micro-FIN per hour
    /// A flag indicating if the user's mining session is currently active.
    pub is_active: bool,
    /// The timestamp when the current mining session was started.
    pub session_started_at: i64,
    /// Bump seed for the PDA.
    pub bump: u8,
}

impl MiningAccount {
    /// Static size for account initialization.
    /// `authority` + `last_claim_at` + `total_mined` + `base_rate` + `is_active` + `session_started_at` + `bump`
    pub const SIZE: usize = 8 + 32 + 8 + 8 + 8 + 1 + 8 + 1;
}
