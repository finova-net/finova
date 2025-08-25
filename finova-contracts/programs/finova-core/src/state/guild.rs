//! State account for guilds.

use anchor_lang::prelude::*;

/// # Guild
///
/// Stores all data related to a specific guild.
/// Each guild has one `Guild` PDA, seeded by its unique ID.
///
/// Seeds: `[b"guild", guild_id.to_le_bytes().as_ref()]`
#[account]
#[derive(Default, Debug)]
pub struct Guild {
    /// A unique, sequential ID for the guild.
    pub id: u64,
    /// The public key of the user who created and leads the guild.
    pub leader: Pubkey,
    /// The name of the guild.
    pub name: String,
    /// A unique, human-readable handle for the guild.
    pub handle: String,
    /// A URI pointing to the guild's metadata (e.g., logo, description).
    pub metadata_uri: String,
    /// The total number of members in the guild.
    pub member_count: u32,
    /// The timestamp when the guild was created.
    pub created_at: i64,
    /// A flag indicating if the guild is active and can accept new members.
    pub is_active: bool,
    /// Bump seed for the PDA.
    pub bump: u8,
}

impl Guild {
    /// Static size for account initialization.
    /// `id` + `leader` + `name` + `handle` + `metadata_uri` + `member_count` + `created_at` + `is_active` + `bump`
    /// We add a buffer for the strings.
    pub const SIZE: usize = 8 + 8 + 32 + (4 + 32) + (4 + 16) + (4 + 128) + 4 + 8 + 1 + 1;
}
