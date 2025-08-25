//! User-specific state account for Experience Points (XP) and leveling.

use anchor_lang::prelude::*;

/// # XpAccount
///
/// Stores all data related to a user's Experience Points (XP), level,
/// and daily activity streaks. Each user has one `XpAccount` PDA.
///
/// Seeds: `[b"xp", authority.key().as_ref()]`
#[account]
#[derive(Default, Debug)]
pub struct XpAccount {
    /// The user's wallet public key.
    pub authority: Pubkey,
    /// The user's current level.
    pub level: u32,
    /// The user's total accumulated XP over their lifetime.
    pub total_xp: u64,
    /// The amount of XP the user has accumulated within the current level.
    pub current_level_xp: u64,
    /// The amount of XP required to advance to the next level.
    pub next_level_xp: u64,
    /// The user's current daily login streak.
    pub daily_streak: u32,
    /// The timestamp of the user's last daily activity check-in.
    pub last_streak_at: i64,
    /// A record of recent activity IDs to prevent duplicate XP grants.
    /// This is a simple ring buffer to store the last N activity IDs.
    pub recent_activity: [u64; 10],
    /// The current index for the `recent_activity` ring buffer.
    pub activity_index: u8,
    /// Bump seed for the PDA.
    pub bump: u8,
}

impl XpAccount {
    /// Static size for account initialization.
    /// `authority` + `level` + `total_xp` + `current_level_xp` + `next_level_xp` + `daily_streak` + `last_streak_at`
    /// + `recent_activity` + `activity_index` + `bump`
    pub const SIZE: usize = 8 + 32 + 4 + 8 + 8 + 8 + 4 + 8 + (8 * 10) + 1 + 1;
}
