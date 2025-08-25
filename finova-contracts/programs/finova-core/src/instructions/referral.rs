//! Instruction for processing referrals.

use anchor_lang::prelude::*;
use crate::state::{UserProfile};
use crate::errors::FinovaError;

/// # Context for processing a referral
///
/// This context links a new user (referee) to their referrer.
/// It assumes the referee has just been initialized.
#[derive(Accounts)]
pub struct ProcessReferral<'info> {
    /// The user who is being referred (the new user).
    #[account(mut)]
    pub referee: Signer<'info>,

    /// The referee's user profile account.
    #[account(
        mut,
        seeds = [b"user_profile", referee.key().as_ref()],
        bump,
        constraint = referee_profile.authority == referee.key(),
        // Ensure this user has not been referred yet.
        constraint = referee_profile.referred_by == Pubkey::default()
    )]
    pub referee_profile: Account<'info, UserProfile>,

    /// The user profile account of the person who made the referral.
    #[account(
        mut,
        // The referrer's pubkey is passed in as an account,
        // assuming the client resolved it from a referral code.
        constraint = referrer_profile.authority == referrer.key()
    )]
    pub referrer_profile: Account<'info, UserProfile>,

    /// The account of the referrer.
    /// CHECK: This is safe because we are only using its key and validating it against the referrer_profile.
    pub referrer: AccountInfo<'info>,
}


/// # Handler for the `process_referral` instruction
///
/// This function links a referee to a referrer and updates the referrer's stats.
pub fn process_handler(ctx: Context<ProcessReferral>) -> Result<()> {
    // Link the referee to the referrer
    let referee_profile = &mut ctx.accounts.referee_profile;
    referee_profile.referred_by = ctx.accounts.referrer.key();

    // Increment the referrer's direct referral count
    let referrer_profile = &mut ctx.accounts.referrer_profile;
    referrer_profile.direct_referrals = referrer_profile.direct_referrals.saturating_add(1);

    msg!("Referral successful: {} was referred by {}",
        ctx.accounts.referee.key(),
        ctx.accounts.referrer.key()
    );

    Ok(())
}
