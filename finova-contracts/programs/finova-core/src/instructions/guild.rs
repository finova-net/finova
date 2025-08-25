//! Instructions for creating and managing guilds.

use anchor_lang::prelude::*;
use crate::state::{Guild, UserAccount, NetworkState, XpAccount};
use crate::errors::FinovaError;

const MAX_GUILD_MEMBERS: u32 = 50;

/// # Context for creating a guild
#[derive(Accounts)]
#[instruction(name: String, handle: String, metadata_uri: String)]
pub struct CreateGuild<'info> {
    /// The user creating the guild, who will become the leader.
    #[account(mut)]
    pub leader: Signer<'info>,

    /// The user's account, to verify they can create a guild and to update their status.
    #[account(
        mut,
        seeds = [b"user", leader.key().as_ref()],
        bump,
        constraint = user_account.authority == leader.key(),
        constraint = user_account.guild_id == 0 @ FinovaError::AlreadyInGuild
    )]
    pub user_account: Account<'info, UserAccount>,

    /// The user's XP account, to verify they meet the level requirement.
    #[account(
        seeds = [b"xp", leader.key().as_ref()],
        bump,
        constraint = xp_account.authority == leader.key(),
        constraint = xp_account.level >= network_state.config.min_guild_creation_level @ FinovaError::InsufficientLevel
    )]
    pub xp_account: Account<'info, XpAccount>,

    /// The global network state, used for getting the next guild ID.
    #[account(
        mut,
        seeds = [b"network_state"],
        bump
    )]
    pub network_state: Account<'info, NetworkState>,

    /// The new guild account to be created.
    #[account(
        init,
        payer = leader,
        space = Guild::SIZE,
        seeds = [b"guild", network_state.next_guild_id.to_le_bytes().as_ref()],
        bump
    )]
    pub guild: Account<'info, Guild>,

    /// The system program, required for account creation.
    pub system_program: Program<'info, System>,
}

/// # Context for joining a guild
#[derive(Accounts)]
pub struct JoinGuild<'info> {
    /// The user joining the guild.
    #[account(mut)]
    pub member: Signer<'info>,

    /// The user's account, to be updated with the new guild ID.
    #[account(
        mut,
        seeds = [b"user", member.key().as_ref()],
        bump,
        constraint = user_account.authority == member.key(),
        constraint = user_account.guild_id == 0 @ FinovaError::AlreadyInGuild
    )]
    pub user_account: Account<'info, UserAccount>,

    /// The guild account to be joined.
    #[account(
        mut,
        constraint = guild.is_active @ FinovaError::GuildNotActive,
        constraint = guild.member_count < MAX_GUILD_MEMBERS @ FinovaError::GuildFull
    )]
    pub guild: Account<'info, Guild>,
}

/// # Context for leaving a guild
#[derive(Accounts)]
pub struct LeaveGuild<'info> {
    /// The user leaving the guild.
    #[account(mut)]
    pub member: Signer<'info>,

    /// The user's account, to be updated.
    #[account(
        mut,
        seeds = [b"user", member.key().as_ref()],
        bump,
        constraint = user_account.authority == member.key(),
        constraint = user_account.guild_id != 0 @ FinovaError::NotInGuild
    )]
    pub user_account: Account<'info, UserAccount>,

    /// The guild account to be left.
    #[account(
        mut,
        constraint = guild.id == user_account.guild_id
    )]
    pub guild: Account<'info, Guild>,
}


/// # Handler for the `create_guild` instruction
pub fn create_handler(ctx: Context<CreateGuild>, name: String, handle: String, metadata_uri: String) -> Result<()> {
    // Initialize the Guild account
    let guild = &mut ctx.accounts.guild;
    let guild_id = ctx.accounts.network_state.next_guild_id;

    guild.id = guild_id;
    guild.leader = ctx.accounts.leader.key();
    guild.name = name;
    guild.handle = handle;
    guild.metadata_uri = metadata_uri;
    guild.member_count = 1; // The leader is the first member
    guild.created_at = Clock::get()?.unix_timestamp;
    guild.is_active = true;
    guild.bump = ctx.bumps.guild;

    // Update the user's account to link them to the new guild
    ctx.accounts.user_account.guild_id = guild_id;

    // Increment the network's guild count
    let network_state = &mut ctx.accounts.network_state;
    network_state.total_guilds += 1;
    network_state.next_guild_id += 1;

    msg!("Guild '{}' created with ID {} by user {}", guild.name, guild.id, guild.leader);
    Ok(())
}

/// # Handler for the `join_guild` instruction
pub fn join_handler(ctx: Context<JoinGuild>) -> Result<()> {
    // Update the user's account to link them to the guild
    ctx.accounts.user_account.guild_id = ctx.accounts.guild.id;

    // Increment the guild's member count
    ctx.accounts.guild.member_count += 1;

    msg!("User {} joined guild {}", ctx.accounts.member.key(), ctx.accounts.guild.id);
    Ok(())
}

/// # Handler for the `leave_guild` instruction
pub fn leave_handler(ctx: Context<LeaveGuild>) -> Result<()> {
    // A guild leader cannot leave the guild; they must dissolve it (future instruction).
    require!(ctx.accounts.guild.leader != ctx.accounts.member.key(), FinovaError::LeaderCannotLeaveGuild);

    // Reset the user's guild ID
    ctx.accounts.user_account.guild_id = 0;

    // Decrement the guild's member count
    ctx.accounts.guild.member_count -= 1;

    msg!("User {} left guild {}", ctx.accounts.member.key(), ctx.accounts.guild.id);
    Ok(())
}
