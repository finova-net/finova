//! Instructions for XP and social activities.

use anchor_lang::prelude::*;
use crate::state::XpAccount;
use crate::errors::FinovaError;

/// # Context for granting XP to a user
#[derive(Accounts)]
pub struct GrantXp<'info> {
    /// The authority granting the XP (could be the user themselves or an admin).
    #[account(mut)]
    pub authority: Signer<'info>,

    /// The user's XP account to be updated.
    #[account(
        mut,
        seeds = [b"xp", user.key().as_ref()],
        bump
    )]
    pub xp_account: Account<'info, XpAccount>,

    /// The user account, to verify authority.
    /// CHECK: This is a simplified check. A more robust system might have a dedicated admin signer.
    #[account(constraint = xp_account.authority == authority.key())]
    pub user: AccountInfo<'info>,
}

/// # Context for updating a user's daily streak
#[derive(Accounts)]
pub struct UpdateStreak<'info> {
    /// The user whose streak is being updated.
    #[account(mut)]
    pub authority: Signer<'info>,

    /// The user's XP account.
    #[account(
        mut,
        seeds = [b"xp", authority.key().as_ref()],
        bump,
        constraint = xp_account.authority == authority.key()
    )]
    pub xp_account: Account<'info, XpAccount>,
}


/// # Handler for the `grant_xp` instruction
pub fn grant_handler(ctx: Context<GrantXp>, xp_amount: u64, activity_id: u64) -> Result<()> {
    require!(xp_amount > 0, FinovaError::InvalidAmount);
    let xp_account = &mut ctx.accounts.xp_account;

    // --- Prevent Duplicate Activity ---
    // Check if this activity ID has already been processed recently.
    require!(!xp_account.recent_activity.contains(&activity_id), FinovaError::DuplicateActivity);

    // Add the new activity ID to the ring buffer.
    let index = xp_account.activity_index as usize;
    xp_account.recent_activity[index] = activity_id;
    xp_account.activity_index = (xp_account.activity_index + 1) % 10; // 10 is the size of the buffer

    // --- Add XP and Level Up ---
    xp_account.total_xp = xp_account.total_xp.saturating_add(xp_amount);
    xp_account.current_level_xp = xp_account.current_level_xp.saturating_add(xp_amount);

    let mut leveled_up = false;
    while xp_account.current_level_xp >= xp_account.next_level_xp {
        xp_account.level += 1;
        xp_account.current_level_xp -= xp_account.next_level_xp;

        // Exponential growth for next level's XP requirement: base * level^1.5
        let base = 100.0;
        let level_f64 = xp_account.level as f64;
        xp_account.next_level_xp = (base * level_f64.powf(1.5)) as u64;
        leveled_up = true;
    }

    if leveled_up {
        msg!("User {} leveled up to level {}!", ctx.accounts.authority.key(), xp_account.level);
    } else {
        msg!("User {} granted {} XP.", ctx.accounts.authority.key(), xp_amount);
    }

    Ok(())
}

/// # Handler for the `update_streak` instruction
pub fn streak_handler(ctx: Context<UpdateStreak>) -> Result<()> {
    let xp_account = &mut ctx.accounts.xp_account;
    let clock = Clock::get()?;

    const ONE_DAY: i64 = 86_400; // seconds in a day

    let last_day = xp_account.last_streak_at / ONE_DAY;
    let current_day = clock.unix_timestamp / ONE_DAY;

    if current_day > last_day {
        if current_day == last_day + 1 {
            // Consecutive day, increment streak
            xp_account.daily_streak += 1;
        } else {
            // Streak broken, reset to 1
            xp_account.daily_streak = 1;
        }
        xp_account.last_streak_at = clock.unix_timestamp;
        msg!("Daily streak for user {} is now {}.", ctx.accounts.authority.key(), xp_account.daily_streak);
    } else {
        // Already checked in today, do nothing.
        msg!("User {} already checked in today.", ctx.accounts.authority.key());
    }

    Ok(())
}
