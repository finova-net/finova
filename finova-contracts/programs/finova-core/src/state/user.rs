//! User-specific state accounts.

use anchor_lang::prelude::*;

/// # UserAccount
///
/// Stores the core, essential information about a user.
/// Each user has one `UserAccount` PDA.
///
/// Seeds: `[b"user", authority.key().as_ref()]`
#[account]
#[derive(Default, Debug)]
pub struct UserAccount {
    /// The user's wallet public key. This is the authority for the account.
    pub authority: Pubkey,
    /// The timestamp when the user account was created.
    pub created_at: i64,
    /// The last time the user interacted with the program.
    pub last_activity_at: i64,
    /// The user's Know Your Customer (KYC) verification status.
    pub kyc_verified: bool,
    /// The ID of the guild the user is a member of, if any.
    pub guild_id: u64, // 0 if not in a guild
    /// A unique, sequential ID for the user.
    pub user_id: u64,
    /// Bump seed for the PDA.
    pub bump: u8,
}

impl UserAccount {
    /// Static size for account initialization.
    /// `authority` + `created_at` + `last_activity_at` + `kyc_verified` + `guild_id` + `user_id` + `bump`
    pub const SIZE: usize = 8 + 32 + 8 + 8 + 1 + 8 + 8 + 1;
}


/// # UserProfile
///
/// Stores profile-specific information for a user, such as referral data.
/// This keeps less frequently accessed data separate from the main `UserAccount`.
///
/// Seeds: `[b"user_profile", authority.key().as_ref()]`
#[account]
#[derive(Default, Debug)]
pub struct UserProfile {
    /// The user's wallet public key.
    pub authority: Pubkey,
    /// The user's unique referral code that they can share with others.
    /// Stored as a fixed-size array for on-chain efficiency.
    pub referral_code: [u8; 16],
    /// The public key of the user who referred this user.
    pub referred_by: Pubkey,
    /// The total number of direct referrals this user has made.
    pub direct_referrals: u32,
    /// The total number of users in all levels of this user's network.
    pub total_network_size: u64,
    /// Bump seed for the PDA.
    pub bump: u8,
}

impl UserProfile {
    /// Static size for account initialization.
    /// `authority` + `referral_code` + `referred_by` + `direct_referrals` + `total_network_size` + `bump`
    pub const SIZE: usize = 8 + 32 + 16 + 32 + 4 + 8 + 1;
}
