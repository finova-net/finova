//! Instruction: Initialize User
//!
//! This instruction is called by a new user to create their set of on-chain accounts.

use anchor_lang::prelude::*;
use crate::state::{UserAccount, UserProfile, MiningAccount, XpAccount, StakingAccount, NetworkState};
use crate::utils::generate_referral_code; // A utility function we will create later

/// # Context for initializing a user
///
/// This context defines all the accounts that need to be created for a new user.
#[derive(Accounts)]
#[instruction(referral_code: Option<String>)]
pub struct InitializeUser<'info> {
    /// The new user, who pays for the account creation.
    #[account(mut)]
    pub authority: Signer<'info>,

    /// The global network state, used to get the total user count for the new user's ID.
    #[account(
        mut,
        seeds = [b"network_state"],
        bump
    )]
    pub network_state: Account<'info, NetworkState>,

    /// The user's main account PDA.
    #[account(
        init,
        payer = authority,
        space = UserAccount::SIZE,
        seeds = [b"user", authority.key().as_ref()],
        bump
    )]
    pub user_account: Account<'info, UserAccount>,

    /// The user's profile account PDA for less-frequently accessed data.
    #[account(
        init,
        payer = authority,
        space = UserProfile::SIZE,
        seeds = [b"user_profile", authority.key().as_ref()],
        bump
    )]
    pub user_profile: Account<'info, UserProfile>,

    /// The user's mining account PDA.
    #[account(
        init,
        payer = authority,
        space = MiningAccount::SIZE,
        seeds = [b"mining", authority.key().as_ref()],
        bump
    )]
    pub mining_account: Account<'info, MiningAccount>,

    /// The user's XP account PDA.
    #[account(
        init,
        payer = authority,
        space = XpAccount::SIZE,
        seeds = [b"xp", authority.key().as_ref()],
        bump
    )]
    pub xp_account: Account<'info, XpAccount>,

    /// The user's staking account PDA.
    #[account(
        init,
        payer = authority,
        space = StakingAccount::SIZE,
        seeds = [b"staking", authority.key().as_ref()],
        bump
    )]
    pub staking_account: Account<'info, StakingAccount>,

    /// The system program, required by Anchor for account creation.
    pub system_program: Program<'info, System>,
}

/// # Handler for the `initialize_user` instruction
///
/// This function executes the logic to create and initialize a new user's accounts.
///
/// ## Arguments
///
/// * `ctx` - The context containing the required accounts.
/// * `referral_code` - The referral code of the user who referred this new user (optional).
pub fn handler(ctx: Context<InitializeUser>, _referral_code: Option<String>) -> Result<()> {
    let clock = Clock::get()?;
    let authority_key = ctx.accounts.authority.key();

    // Increment total user count and assign the new ID
    ctx.accounts.network_state.total_users += 1;
    let user_id = ctx.accounts.network_state.total_users;

    // --- Initialize UserAccount ---
    let user_account = &mut ctx.accounts.user_account;
    user_account.authority = authority_key;
    user_account.created_at = clock.unix_timestamp;
    user_account.last_activity_at = clock.unix_timestamp;
    user_account.kyc_verified = false;
    user_account.guild_id = 0; // 0 indicates no guild
    user_account.user_id = user_id;
    user_account.bump = ctx.bumps.user_account;

    // --- Initialize UserProfile ---
    let user_profile = &mut ctx.accounts.user_profile;
    user_profile.authority = authority_key;
    user_profile.referral_code = generate_referral_code(user_id); // Generate a unique referral code
    // Note: Logic to link to the referrer would go here.
    // This requires resolving the provided `referral_code` to a Pubkey, which is complex.
    // For now, we'll leave `referred_by` as the default (zero pubkey).
    user_profile.referred_by = Pubkey::default();
    user_profile.bump = ctx.bumps.user_profile;

    // --- Initialize MiningAccount ---
    let mining_account = &mut ctx.accounts.mining_account;
    mining_account.authority = authority_key;
    mining_account.is_active = false;
    // TODO: Determine base_rate based on network phase
    mining_account.base_rate = 10_000; // Placeholder: 0.01 FIN/hr in micro-FIN
    mining_account.last_claim_at = clock.unix_timestamp;
    mining_account.bump = ctx.bumps.mining_account;

    // --- Initialize XpAccount ---
    let xp_account = &mut ctx.accounts.xp_account;
    xp_account.authority = authority_key;
    xp_account.level = 1;
    xp_account.next_level_xp = 100; // First level requires 100 XP
    xp_account.bump = ctx.bumps.xp_account;

    // --- Initialize StakingAccount ---
    let staking_account = &mut ctx.accounts.staking_account;
    staking_account.authority = authority_key;
    staking_account.bump = ctx.bumps.staking_account;

    msg!("User #{} initialized for authority: {}", user_id, authority_key);

    Ok(())
}
