//! On-chain events for the Finova Core program.
//!
//! These events can be subscribed to by off-chain clients to monitor
//! important state changes and activities within the program.

use anchor_lang::prelude::*;

#[event]
pub struct NetworkInitialized {
    pub admin: Pubkey,
    pub fin_token_mint: Pubkey,
    pub timestamp: i64,
}

#[event]
pub struct UserInitialized {
    pub authority: Pubkey,
    pub user_id: u64,
    pub timestamp: i64,
}

#[event]
pub struct Staked {
    pub user: Pubkey,
    pub amount: u64,
    pub timestamp: i64,
}

#[event]
pub struct Unstaked {
    pub user: Pubkey,
    pub amount: u64,
    pub timestamp: i64,
}

#[event]
pub struct XpGranted {
    pub user: Pubkey,
    pub amount: u64,
    pub new_total_xp: u64,
}

#[event]
pub struct LeveledUp {
    pub user: Pubkey,
    pub new_level: u32,
}

#[event]
pub struct GuildCreated {
    pub leader: Pubkey,
    pub guild_id: u64,
    pub name: String,
}

#[event]
pub struct GuildJoined {
    pub member: Pubkey,
    pub guild_id: u64,
}
