// programs/finova-core/src/instructions/guild.rs

use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};
use crate::constants::*;
use crate::errors::*;
use crate::state::*;
use crate::utils::*;

/// Creates a new guild with specified parameters
#[derive(Accounts)]
#[instruction(guild_name: String, guild_description: String)]
pub struct CreateGuild<'info> {
    #[account(mut)]
    pub creator: Signer<'info>,
    
    #[account(
        mut,
        has_one = owner @ FinovaError::InvalidUserAccount,
        constraint = user_account.xp_level >= GUILD_CREATION_MIN_LEVEL @ FinovaError::InsufficientLevel
    )]
    pub user_account: Account<'info, UserAccount>,
    
    #[account(
        init,
        payer = creator,
        space = Guild::LEN,
        seeds = [
            GUILD_SEED.as_bytes(),
            guild_name.as_bytes(),
            creator.key().as_ref()
        ],
        bump
    )]
    pub guild: Account<'info, Guild>,
    
    #[account(
        mut,
        seeds = [NETWORK_STATE_SEED.as_bytes()],
        bump
    )]
    pub network_state: Account<'info, NetworkState>,
    
    pub system_program: Program<'info, System>,
}

/// Joins an existing guild
#[derive(Accounts)]
pub struct JoinGuild<'info> {
    #[account(mut)]
    pub member: Signer<'info>,
    
    #[account(
        mut,
        has_one = owner @ FinovaError::InvalidUserAccount,
        constraint = user_account.xp_level >= GUILD_JOIN_MIN_LEVEL @ FinovaError::InsufficientLevel
    )]
    pub user_account: Account<'info, UserAccount>,
    
    #[account(
        mut,
        constraint = guild.is_active @ FinovaError::GuildInactive,
        constraint = guild.members.len() < GUILD_MAX_MEMBERS @ FinovaError::GuildFull,
        constraint = !guild.members.contains(&member.key()) @ FinovaError::AlreadyGuildMember
    )]
    pub guild: Account<'info, Guild>,
    
    #[account(
        init,
        payer = member,
        space = GuildMember::LEN,
        seeds = [
            GUILD_MEMBER_SEED.as_bytes(),
            guild.key().as_ref(),
            member.key().as_ref()
        ],
        bump
    )]
    pub guild_member: Account<'info, GuildMember>,
    
    pub system_program: Program<'info, System>,
}

/// Leaves a guild
#[derive(Accounts)]
pub struct LeaveGuild<'info> {
    #[account(mut)]
    pub member: Signer<'info>,
    
    #[account(
        mut,
        has_one = owner @ FinovaError::InvalidUserAccount
    )]
    pub user_account: Account<'info, UserAccount>,
    
    #[account(
        mut,
        constraint = guild.members.contains(&member.key()) @ FinovaError::NotGuildMember
    )]
    pub guild: Account<'info, Guild>,
    
    #[account(
        mut,
        close = member,
        seeds = [
            GUILD_MEMBER_SEED.as_bytes(),
            guild.key().as_ref(),
            member.key().as_ref()
        ],
        bump
    )]
    pub guild_member: Account<'info, GuildMember>,
}

/// Promotes a guild member to officer
#[derive(Accounts)]
pub struct PromoteMember<'info> {
    #[account(mut)]
    pub guild_master: Signer<'info>,
    
    /// CHECK: Member to promote
    pub member_to_promote: AccountInfo<'info>,
    
    #[account(
        mut,
        has_one = guild_master @ FinovaError::NotGuildMaster,
        constraint = guild.members.contains(&member_to_promote.key()) @ FinovaError::NotGuildMember,
        constraint = guild.officers.len() < GUILD_MAX_OFFICERS @ FinovaError::TooManyOfficers
    )]
    pub guild: Account<'info, Guild>,
    
    #[account(
        mut,
        seeds = [
            GUILD_MEMBER_SEED.as_bytes(),
            guild.key().as_ref(),
            member_to_promote.key().as_ref()
        ],
        bump
    )]
    pub guild_member: Account<'info, GuildMember>,
}

/// Demotes a guild officer to regular member
#[derive(Accounts)]
pub struct DemoteMember<'info> {
    #[account(mut)]
    pub guild_master: Signer<'info>,
    
    /// CHECK: Member to demote
    pub member_to_demote: AccountInfo<'info>,
    
    #[account(
        mut,
        has_one = guild_master @ FinovaError::NotGuildMaster,
        constraint = guild.officers.contains(&member_to_demote.key()) @ FinovaError::NotGuildOfficer
    )]
    pub guild: Account<'info, Guild>,
    
    #[account(
        mut,
        seeds = [
            GUILD_MEMBER_SEED.as_bytes(),
            guild.key().as_ref(),
            member_to_demote.key().as_ref()
        ],
        bump
    )]
    pub guild_member: Account<'info, GuildMember>,
}

/// Kicks a member from the guild
#[derive(Accounts)]
pub struct KickMember<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    
    /// CHECK: Member to kick
    pub member_to_kick: AccountInfo<'info>,
    
    #[account(
        mut,
        constraint = guild.guild_master == authority.key() 
            || guild.officers.contains(&authority.key()) @ FinovaError::InsufficientPermissions,
        constraint = guild.members.contains(&member_to_kick.key()) @ FinovaError::NotGuildMember,
        constraint = member_to_kick.key() != guild.guild_master @ FinovaError::CannotKickGuildMaster
    )]
    pub guild: Account<'info, Guild>,
    
    #[account(
        mut,
        close = authority,
        seeds = [
            GUILD_MEMBER_SEED.as_bytes(),
            guild.key().as_ref(),
            member_to_kick.key().as_ref()
        ],
        bump
    )]
    pub guild_member: Account<'info, GuildMember>,
}

/// Starts a guild challenge
#[derive(Accounts)]
#[instruction(challenge_type: u8, duration: u64, target_value: u64)]
pub struct StartGuildChallenge<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    
    #[account(
        mut,
        constraint = guild.guild_master == authority.key() 
            || guild.officers.contains(&authority.key()) @ FinovaError::InsufficientPermissions,
        constraint = guild.is_active @ FinovaError::GuildInactive,
        constraint = guild.active_challenge.is_none() @ FinovaError::ChallengeAlreadyActive
    )]
    pub guild: Account<'info, Guild>,
    
    #[account(
        init,
        payer = authority,
        space = GuildChallenge::LEN,
        seeds = [
            GUILD_CHALLENGE_SEED.as_bytes(),
            guild.key().as_ref(),
            &Clock::get()?.unix_timestamp.to_le_bytes()
        ],
        bump
    )]
    pub challenge: Account<'info, GuildChallenge>,
    
    pub system_program: Program<'info, System>,
    pub clock: Sysvar<'info, Clock>,
}

/// Participates in a guild challenge
#[derive(Accounts)]
#[instruction(contribution_value: u64)]
pub struct ParticipateInChallenge<'info> {
    #[account(mut)]
    pub member: Signer<'info>,
    
    #[account(
        mut,
        has_one = owner @ FinovaError::InvalidUserAccount
    )]
    pub user_account: Account<'info, UserAccount>,
    
    #[account(
        constraint = guild.members.contains(&member.key()) @ FinovaError::NotGuildMember,
        constraint = guild.is_active @ FinovaError::GuildInactive
    )]
    pub guild: Account<'info, Guild>,
    
    #[account(
        mut,
        constraint = challenge.is_active @ FinovaError::ChallengeInactive,
        constraint = challenge.end_time > Clock::get()?.unix_timestamp @ FinovaError::ChallengeExpired
    )]
    pub challenge: Account<'info, GuildChallenge>,
    
    #[account(
        mut,
        seeds = [
            GUILD_MEMBER_SEED.as_bytes(),
            guild.key().as_ref(),
            member.key().as_ref()
        ],
        bump
    )]
    pub guild_member: Account<'info, GuildMember>,
    
    pub clock: Sysvar<'info, Clock>,
}

/// Completes a guild challenge and distributes rewards
#[derive(Accounts)]
pub struct CompleteChallenge<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    
    #[account(
        mut,
        constraint = guild.guild_master == authority.key() 
            || guild.officers.contains(&authority.key()) @ FinovaError::InsufficientPermissions,
        constraint = guild.is_active @ FinovaError::GuildInactive
    )]
    pub guild: Account<'info, Guild>,
    
    #[account(
        mut,
        constraint = challenge.is_active @ FinovaError::ChallengeInactive,
        constraint = challenge.end_time <= Clock::get()?.unix_timestamp @ FinovaError::ChallengeNotExpired
    )]
    pub challenge: Account<'info, GuildChallenge>,
    
    #[account(
        mut,
        seeds = [REWARDS_POOL_SEED.as_bytes(), GUILD_REWARDS_IDENTIFIER.as_bytes()],
        bump
    )]
    pub guild_rewards_pool: Account<'info, RewardsPool>,
    
    pub clock: Sysvar<'info, Clock>,
}

/// Updates guild settings
#[derive(Accounts)]
#[instruction(new_description: Option<String>, new_privacy: Option<bool>)]
pub struct UpdateGuildSettings<'info> {
    #[account(mut)]
    pub guild_master: Signer<'info>,
    
    #[account(
        mut,
        has_one = guild_master @ FinovaError::NotGuildMaster,
        constraint = guild.is_active @ FinovaError::GuildInactive
    )]
    pub guild: Account<'info, Guild>,
}

/// Transfers guild mastership
#[derive(Accounts)]
pub struct TransferGuildMastership<'info> {
    #[account(mut)]
    pub current_master: Signer<'info>,
    
    /// CHECK: New guild master
    pub new_master: AccountInfo<'info>,
    
    #[account(
        mut,
        has_one = guild_master @ FinovaError::NotGuildMaster,
        constraint = guild.members.contains(&new_master.key()) @ FinovaError::NotGuildMember,
        constraint = guild.is_active @ FinovaError::GuildInactive
    )]
    pub guild: Account<'info, Guild>,
    
    #[account(
        mut,
        seeds = [
            GUILD_MEMBER_SEED.as_bytes(),
            guild.key().as_ref(),
            new_master.key().as_ref()
        ],
        bump
    )]
    pub new_master_member: Account<'info, GuildMember>,
    
    #[account(
        mut,
        seeds = [
            GUILD_MEMBER_SEED.as_bytes(),
            guild.key().as_ref(),
            current_master.key().as_ref()
        ],
        bump
    )]
    pub current_master_member: Account<'info, GuildMember>,
}

/// Dissolves a guild (only if conditions are met)
#[derive(Accounts)]
pub struct DissolveGuild<'info> {
    #[account(mut)]
    pub guild_master: Signer<'info>,
    
    #[account(
        mut,
        has_one = guild_master @ FinovaError::NotGuildMaster,
        constraint = guild.is_active @ FinovaError::GuildInactive,
        constraint = guild.members.len() <= 1 @ FinovaError::GuildHasActiveMembers,
        close = guild_master
    )]
    pub guild: Account<'info, Guild>,
    
    #[account(
        mut,
        seeds = [NETWORK_STATE_SEED.as_bytes()],
        bump
    )]
    pub network_state: Account<'info, NetworkState>,
}

// Implementation functions
pub fn create_guild(
    ctx: Context<CreateGuild>,
    guild_name: String,
    guild_description: String,
    is_private: bool,
) -> Result<()> {
    let guild = &mut ctx.accounts.guild;
    let user_account = &mut ctx.accounts.user_account;
    let network_state = &mut ctx.accounts.network_state;
    let clock = Clock::get()?;

    // Validate inputs
    require!(
        guild_name.len() >= GUILD_NAME_MIN_LENGTH && guild_name.len() <= GUILD_NAME_MAX_LENGTH,
        FinovaError::InvalidGuildName
    );
    require!(
        guild_description.len() <= GUILD_DESCRIPTION_MAX_LENGTH,
        FinovaError::InvalidGuildDescription
    );

    // Initialize guild
    guild.guild_master = ctx.accounts.creator.key();
    guild.name = guild_name;
    guild.description = guild_description;
    guild.created_at = clock.unix_timestamp;
    guild.is_active = true;
    guild.is_private = is_private;
    guild.members = vec![ctx.accounts.creator.key()];
    guild.officers = Vec::new();
    guild.member_count = 1;
    guild.total_xp_earned = 0;
    guild.total_tokens_earned = 0;
    guild.challenges_completed = 0;
    guild.guild_level = 1;
    guild.active_challenge = None;
    guild.guild_treasury = 0;
    guild.reputation_score = INITIAL_GUILD_REPUTATION;
    guild.last_activity = clock.unix_timestamp;

    // Update user account
    user_account.guild_id = Some(guild.key());
    user_account.guild_role = GuildRole::Master;
    user_account.guild_joined_at = clock.unix_timestamp;

    // Update network state
    network_state.total_guilds = network_state.total_guilds.checked_add(1).unwrap();

    emit!(GuildCreatedEvent {
        guild: guild.key(),
        guild_master: ctx.accounts.creator.key(),
        name: guild.name.clone(),
        created_at: clock.unix_timestamp,
    });

    Ok(())
}

pub fn join_guild(ctx: Context<JoinGuild>) -> Result<()> {
    let guild = &mut ctx.accounts.guild;
    let user_account = &mut ctx.accounts.user_account;
    let guild_member = &mut ctx.accounts.guild_member;
    let clock = Clock::get()?;

    // Check if user is already in another guild
    require!(user_account.guild_id.is_none(), FinovaError::AlreadyInGuild);

    // Add member to guild
    guild.members.push(ctx.accounts.member.key());
    guild.member_count = guild.member_count.checked_add(1).unwrap();
    guild.last_activity = clock.unix_timestamp;

    // Initialize guild member account
    guild_member.guild = guild.key();
    guild_member.member = ctx.accounts.member.key();
    guild_member.joined_at = clock.unix_timestamp;
    guild_member.role = GuildRole::Member;
    guild_member.contribution_score = 0;
    guild_member.total_xp_contributed = 0;
    guild_member.total_tokens_contributed = 0;
    guild_member.challenges_participated = 0;
    guild_member.last_activity = clock.unix_timestamp;

    // Update user account
    user_account.guild_id = Some(guild.key());
    user_account.guild_role = GuildRole::Member;
    user_account.guild_joined_at = clock.unix_timestamp;

    emit!(GuildJoinedEvent {
        guild: guild.key(),
        member: ctx.accounts.member.key(),
        joined_at: clock.unix_timestamp,
    });

    Ok(())
}

pub fn leave_guild(ctx: Context<LeaveGuild>) -> Result<()> {
    let guild = &mut ctx.accounts.guild;
    let user_account = &mut ctx.accounts.user_account;
    let clock = Clock::get()?;

    // Cannot leave if you're the guild master and there are other members
    require!(
        guild.guild_master != ctx.accounts.member.key() || guild.members.len() == 1,
        FinovaError::GuildMasterCannotLeave
    );

    // Remove member from guild
    guild.members.retain(|&member| member != ctx.accounts.member.key());
    guild.officers.retain(|&officer| officer != ctx.accounts.member.key());
    guild.member_count = guild.member_count.checked_sub(1).unwrap();
    guild.last_activity = clock.unix_timestamp;

    // Update user account
    user_account.guild_id = None;
    user_account.guild_role = GuildRole::None;
    user_account.guild_joined_at = 0;

    emit!(GuildLeftEvent {
        guild: guild.key(),
        member: ctx.accounts.member.key(),
        left_at: clock.unix_timestamp,
    });

    Ok(())
}

pub fn promote_member(ctx: Context<PromoteMember>) -> Result<()> {
    let guild = &mut ctx.accounts.guild;
    let guild_member = &mut ctx.accounts.guild_member;
    let clock = Clock::get()?;

    // Add to officers list
    guild.officers.push(ctx.accounts.member_to_promote.key());
    guild.last_activity = clock.unix_timestamp;

    // Update member role
    guild_member.role = GuildRole::Officer;
    guild_member.last_activity = clock.unix_timestamp;

    emit!(MemberPromotedEvent {
        guild: guild.key(),
        member: ctx.accounts.member_to_promote.key(),
        promoted_by: ctx.accounts.guild_master.key(),
        new_role: GuildRole::Officer,
        promoted_at: clock.unix_timestamp,
    });

    Ok(())
}

pub fn demote_member(ctx: Context<DemoteMember>) -> Result<()> {
    let guild = &mut ctx.accounts.guild;
    let guild_member = &mut ctx.accounts.guild_member;
    let clock = Clock::get()?;

    // Remove from officers list
    guild.officers.retain(|&officer| officer != ctx.accounts.member_to_demote.key());
    guild.last_activity = clock.unix_timestamp;

    // Update member role
    guild_member.role = GuildRole::Member;
    guild_member.last_activity = clock.unix_timestamp;

    emit!(MemberDemotedEvent {
        guild: guild.key(),
        member: ctx.accounts.member_to_demote.key(),
        demoted_by: ctx.accounts.guild_master.key(),
        new_role: GuildRole::Member,
        demoted_at: clock.unix_timestamp,
    });

    Ok(())
}

pub fn kick_member(ctx: Context<KickMember>) -> Result<()> {
    let guild = &mut ctx.accounts.guild;
    let clock = Clock::get()?;

    // Remove member from guild
    guild.members.retain(|&member| member != ctx.accounts.member_to_kick.key());
    guild.officers.retain(|&officer| officer != ctx.accounts.member_to_kick.key());
    guild.member_count = guild.member_count.checked_sub(1).unwrap();
    guild.last_activity = clock.unix_timestamp;

    emit!(MemberKickedEvent {
        guild: guild.key(),
        member: ctx.accounts.member_to_kick.key(),
        kicked_by: ctx.accounts.authority.key(),
        kicked_at: clock.unix_timestamp,
    });

    Ok(())
}

pub fn start_guild_challenge(
    ctx: Context<StartGuildChallenge>,
    challenge_type: u8,
    duration: u64,
    target_value: u64,
) -> Result<()> {
    let guild = &mut ctx.accounts.guild;
    let challenge = &mut ctx.accounts.challenge;
    let clock = Clock::get()?;

    require!(
        challenge_type < GUILD_CHALLENGE_TYPES_COUNT,
        FinovaError::InvalidChallengeType
    );
    require!(
        duration >= MIN_CHALLENGE_DURATION && duration <= MAX_CHALLENGE_DURATION,
        FinovaError::InvalidChallengeDuration
    );
    require!(target_value > 0, FinovaError::InvalidTargetValue);

    // Initialize challenge
    challenge.guild = guild.key();
    challenge.challenge_type = challenge_type;
    challenge.start_time = clock.unix_timestamp;
    challenge.end_time = clock.unix_timestamp.checked_add(duration as i64).unwrap();
    challenge.target_value = target_value;
    challenge.current_progress = 0;
    challenge.is_active = true;
    challenge.participants = Vec::new();
    challenge.created_by = ctx.accounts.authority.key();
    challenge.reward_pool = calculate_challenge_reward_pool(challenge_type, target_value, guild.member_count)?;

    // Update guild
    guild.active_challenge = Some(challenge.key());
    guild.last_activity = clock.unix_timestamp;

    emit!(GuildChallengeStartedEvent {
        guild: guild.key(),
        challenge: challenge.key(),
        challenge_type,
        target_value,
        duration,
        started_by: ctx.accounts.authority.key(),
        started_at: clock.unix_timestamp,
    });

    Ok(())
}

pub fn participate_in_challenge(
    ctx: Context<ParticipateInChallenge>,
    contribution_value: u64,
) -> Result<()> {
    let guild = &ctx.accounts.guild;
    let challenge = &mut ctx.accounts.challenge;
    let guild_member = &mut ctx.accounts.guild_member;
    let user_account = &mut ctx.accounts.user_account;
    let clock = Clock::get()?;

    require!(contribution_value > 0, FinovaError::InvalidContributionValue);

    // Validate contribution based on challenge type
    validate_challenge_contribution(
        challenge.challenge_type,
        contribution_value,
        user_account,
    )?;

    // Add participant if not already participating
    if !challenge.participants.contains(&ctx.accounts.member.key()) {
        challenge.participants.push(ctx.accounts.member.key());
    }

    // Update challenge progress
    challenge.current_progress = challenge.current_progress
        .checked_add(contribution_value)
        .unwrap();

    // Update member contribution
    guild_member.contribution_score = guild_member.contribution_score
        .checked_add(contribution_value)
        .unwrap();
    guild_member.challenges_participated = guild_member.challenges_participated
        .checked_add(1)
        .unwrap();
    guild_member.last_activity = clock.unix_timestamp;

    // Calculate XP bonus for participation
    let xp_bonus = calculate_challenge_xp_bonus(
        challenge.challenge_type,
        contribution_value,
        guild.member_count,
    )?;

    // Award XP bonus
    user_account.total_xp = user_account.total_xp.checked_add(xp_bonus).unwrap();
    update_xp_level(user_account)?;

    emit!(ChallengeParticipationEvent {
        guild: guild.key(),
        challenge: challenge.key(),
        participant: ctx.accounts.member.key(),
        contribution_value,
        xp_bonus,
        participated_at: clock.unix_timestamp,
    });

    Ok(())
}

pub fn complete_challenge(ctx: Context<CompleteChallenge>) -> Result<()> {
    let guild = &mut ctx.accounts.guild;
    let challenge = &mut ctx.accounts.challenge;
    let guild_rewards_pool = &mut ctx.accounts.guild_rewards_pool;
    let clock = Clock::get()?;

    // Check if challenge was successful
    let success = challenge.current_progress >= challenge.target_value;

    // Calculate rewards
    let base_reward = challenge.reward_pool;
    let success_multiplier = if success { 
        CHALLENGE_SUCCESS_MULTIPLIER 
    } else { 
        CHALLENGE_PARTIAL_MULTIPLIER 
    };
    
    let total_reward = (base_reward as f64 * success_multiplier) as u64;

    // Update guild stats
    if success {
        guild.challenges_completed = guild.challenges_completed.checked_add(1).unwrap();
        guild.reputation_score = guild.reputation_score
            .checked_add(CHALLENGE_SUCCESS_REPUTATION_BONUS)
            .unwrap();
    }

    guild.total_tokens_earned = guild.total_tokens_earned
        .checked_add(total_reward)
        .unwrap();
    guild.guild_treasury = guild.guild_treasury
        .checked_add(total_reward)
        .unwrap();
    guild.active_challenge = None;
    guild.last_activity = clock.unix_timestamp;

    // Update guild level based on completed challenges
    update_guild_level(guild)?;

    // Mark challenge as completed
    challenge.is_active = false;

    // Distribute rewards to guild treasury
    guild_rewards_pool.available_tokens = guild_rewards_pool.available_tokens
        .checked_sub(total_reward)
        .unwrap();

    emit!(ChallengeCompletedEvent {
        guild: guild.key(),
        challenge: challenge.key(),
        success,
        total_reward,
        participants_count: challenge.participants.len() as u32,
        completed_at: clock.unix_timestamp,
    });

    Ok(())
}

pub fn update_guild_settings(
    ctx: Context<UpdateGuildSettings>,
    new_description: Option<String>,
    new_privacy: Option<bool>,
) -> Result<()> {
    let guild = &mut ctx.accounts.guild;
    let clock = Clock::get()?;

    if let Some(description) = new_description {
        require!(
            description.len() <= GUILD_DESCRIPTION_MAX_LENGTH,
            FinovaError::InvalidGuildDescription
        );
        guild.description = description;
    }

    if let Some(privacy) = new_privacy {
        guild.is_private = privacy;
    }

    guild.last_activity = clock.unix_timestamp;

    emit!(GuildSettingsUpdatedEvent {
        guild: guild.key(),
        updated_by: ctx.accounts.guild_master.key(),
        updated_at: clock.unix_timestamp,
    });

    Ok(())
}

pub fn transfer_guild_mastership(ctx: Context<TransferGuildMastership>) -> Result<()> {
    let guild = &mut ctx.accounts.guild;
    let new_master_member = &mut ctx.accounts.new_master_member;
    let current_master_member = &mut ctx.accounts.current_master_member;
    let clock = Clock::get()?;

    // Transfer mastership
    guild.guild_master = ctx.accounts.new_master.key();
    guild.last_activity = clock.unix_timestamp;

    // Update member roles
    new_master_member.role = GuildRole::Master;
    current_master_member.role = GuildRole::Member;

    // Remove new master from officers list if present
    guild.officers.retain(|&officer| officer != ctx.accounts.new_master.key());

    emit!(GuildMastershipTransferredEvent {
        guild: guild.key(),
        old_master: ctx.accounts.current_master.key(),
        new_master: ctx.accounts.new_master.key(),
        transferred_at: clock.unix_timestamp,
    });

    Ok(())
}

pub fn dissolve_guild(ctx: Context<DissolveGuild>) -> Result<()> {
    let guild = &ctx.accounts.guild;
    let network_state = &mut ctx.accounts.network_state;
    let clock = Clock::get()?;

    // Update network state
    network_state.total_guilds = network_state.total_guilds.checked_sub(1).unwrap();

    emit!(GuildDissolvedEvent {
        guild: guild.key(),
        guild_master: ctx.accounts.guild_master.key(),
        dissolved_at: clock.unix_timestamp,
    });

    Ok(())
}

// Helper functions
fn calculate_challenge_reward_pool(
    challenge_type: u8,
    target_value: u64,
    member_count: u32,
) -> Result<u64> {
    let base_reward = match challenge_type {
        0 => DAILY_CHALLENGE_BASE_REWARD,  // Daily XP Challenge
        1 => WEEKLY_CHALLENGE_BASE_REWARD, // Weekly Mining Challenge
        2 => MONTHLY_CHALLENGE_BASE_REWARD, // Monthly Social Challenge
        _ => return Err(FinovaError::InvalidChallengeType.into()),
    };

    let size_multiplier = (member_count as f64 / 10.0).min(5.0).max(1.0);
    let difficulty_multiplier = (target_value as f64 / 1000.0).min(10.0).max(1.0);

    Ok((base_reward as f64 * size_multiplier * difficulty_multiplier) as u64)
}

fn validate_challenge_contribution(
    challenge_type: u8,
    contribution_value: u64,
    user_account: &UserAccount,
) -> Result<()> {
    match challenge_type {
        0 => { // XP Challenge
            require!(
                contribution_value <= user_account.daily_xp_earned,
                FinovaError::InsufficientXPContribution
            );
        },
        1 => { // Mining Challenge
            require!(
                contribution_value <= user_account.daily_tokens_mined,
                FinovaError::InsufficientMiningContribution
            );
        },
        2 => { // Social Challenge
            require!(
                contribution_value <= user_account.daily_social_interactions,
                FinovaError::InsufficientSocialContribution
            );
        },
        _ => return Err(FinovaError::InvalidChallengeType.into()),
    }
    Ok(())
}

fn calculate_challenge_xp_bonus(
    challenge_type: u8,
    contribution_value: u64,
    member_count: u32,
) -> Result<u64> {
    let base_xp_bonus = match challenge_type {
        0 => contribution_value / 10,  // 10% of XP contribution as bonus
        1 => contribution_value / 5,   // 20% of mining contribution as XP
        2 => contribution_value * 2,   // 2x social interactions as XP
        _ => return Err(FinovaError::InvalidChallengeType.into()),
    };

    let guild_size_bonus = if member_count >= 50 {
        base_xp_bonus / 2  // 50% bonus for large guilds
    } else if member_count >= 25 {
        base_xp_bonus / 4  // 25% bonus for medium guilds
    } else {
        base_xp_bonus / 10 // 10% bonus for small guilds
    };

    Ok(base_xp_bonus.checked_add(guild_size_bonus).unwrap())
}

fn update_guild_level(guild: &mut Guild) -> Result<()> {
    let new_level = calculate_guild_level(
        guild.challenges_completed,
        guild.total_xp_earned,
        guild.member_count,
        guild.reputation_score,
    )?;

    if new_level > guild.guild_level {
        guild.guild_level = new_level;
        
        // Award level up bonuses
        let level_bonus = new_level * GUILD_LEVEL_BONUS_MULTIPLIER;
        guild.guild_treasury = guild.guild_treasury
            .checked_add(level_bonus)
            .unwrap();
    }

    Ok(())
}

fn calculate_guild_level(
    challenges_completed: u32,
    total_xp_earned: u64,
    member_count: u32,
    reputation_score: u64,
) -> Result<u32> {
    let challenge_score = challenges_completed * CHALLENGE_LEVEL_WEIGHT;
    let xp_score = (total_xp_earned / XP_LEVEL_DIVISOR) as u32;
    let member_score = member_count * MEMBER_LEVEL_WEIGHT;
    let reputation_score = (reputation_score / REPUTATION_LEVEL_DIVISOR) as u32;

    let total_score = challenge_score
        .checked_add(xp_score)
        .and_then(|s| s.checked_add(member_score))
        .and_then(|s| s.checked_add(reputation_score))
        .unwrap();

    let level = (total_score / GUILD_LEVEL_THRESHOLD).max(1).min(MAX_GUILD_LEVEL);
    Ok(level)
}

// Event definitions
#[event]
pub struct GuildCreatedEvent {
    pub guild: Pubkey,
    pub guild_master: Pubkey,
    pub name: String,
    pub created_at: i64,
}

#[event]
pub struct GuildJoinedEvent {
    pub guild: Pubkey,
    pub member: Pubkey,
    pub joined_at: i64,
}

#[event]
pub struct GuildLeftEvent {
    pub guild: Pubkey,
    pub member: Pubkey,
    pub left_at: i64,
}

#[event]
pub struct MemberPromotedEvent {
    pub guild: Pubkey,
    pub member: Pubkey,
    pub promoted_by: Pubkey,
    pub new_role: GuildRole,
    pub promoted_at: i64,
}

#[event]
pub struct MemberDemotedEvent {
    pub guild: Pubkey,
    pub member: Pubkey,
    pub demoted_by: Pubkey,
    pub new_role: GuildRole,
    pub demoted_at: i64,
}

#[event]
pub struct MemberKickedEvent {
    pub guild: Pubkey,
    pub member: Pubkey,
    pub kicked_by: Pubkey,
    pub kicked_at: i64,
}

#[event]
pub struct GuildChallengeStartedEvent {
    pub guild: Pubkey,
    pub challenge: Pubkey,
    pub challenge_type: u8,
    pub target_value: u64,
    pub duration: u64,
    pub started_by: Pubkey,
    pub started_at: i64,
}

#[event]
pub struct ChallengeParticipationEvent {
    pub guild: Pubkey,
    pub challenge: Pubkey,
    pub participant: Pubkey,
    pub contribution_value: u64,
    pub xp_bonus: u64,
    pub participated_at: i64,
}

#[event]
pub struct ChallengeCompletedEvent {
    pub guild: Pubkey,
    pub challenge: Pubkey,
    pub success: bool,
    pub total_reward: u64,
    pub participants_count: u32,
    pub completed_at: i64,
}

#[event]
pub struct GuildSettingsUpdatedEvent {
    pub guild: Pubkey,
    pub updated_by: Pubkey,
    pub updated_at: i64,
}

#[event]
pub struct GuildMastershipTransferredEvent {
    pub guild: Pubkey,
    pub old_master: Pubkey,
    pub new_master: Pubkey,
    pub transferred_at: i64,
}

#[event]
pub struct GuildDissolvedEvent {
    pub guild: Pubkey,
    pub guild_master: Pubkey,
    pub dissolved_at: i64,
}

// Additional helper structures for guild challenges
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub struct ChallengeParticipant {
    pub member: Pubkey,
    pub contribution: u64,
    pub joined_at: i64,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub struct GuildStats {
    pub total_xp_this_week: u64,
    pub total_tokens_this_week: u64,
    pub active_members_this_week: u32,
    pub challenges_won: u32,
    pub challenges_lost: u32,
    pub average_member_level: u32,
    pub guild_rank: u32,
}

impl GuildStats {
    pub fn new() -> Self {
        Self {
            total_xp_this_week: 0,
            total_tokens_this_week: 0,
            active_members_this_week: 0,
            challenges_won: 0,
            challenges_lost: 0,
            average_member_level: 1,
            guild_rank: 0,
        }
    }

    pub fn update_weekly_stats(&mut self, xp_gained: u64, tokens_gained: u64) {
        self.total_xp_this_week = self.total_xp_this_week.checked_add(xp_gained).unwrap();
        self.total_tokens_this_week = self.total_tokens_this_week.checked_add(tokens_gained).unwrap();
    }

    pub fn reset_weekly_stats(&mut self) {
        self.total_xp_this_week = 0;
        self.total_tokens_this_week = 0;
        self.active_members_this_week = 0;
    }
}

// Guild ranking system
pub fn update_guild_rankings(guilds: &mut [Guild]) -> Result<()> {
    // Sort guilds by combined score
    guilds.sort_by(|a, b| {
        let score_a = calculate_guild_ranking_score(a);
        let score_b = calculate_guild_ranking_score(b);
        score_b.partial_cmp(&score_a).unwrap()
    });

    // Update rankings
    for (index, guild) in guilds.iter_mut().enumerate() {
        guild.guild_rank = (index + 1) as u32;
    }

    Ok(())
}

fn calculate_guild_ranking_score(guild: &Guild) -> f64 {
    let level_score = guild.guild_level as f64 * 100.0;
    let member_score = guild.member_count as f64 * 10.0;
    let activity_score = guild.challenges_completed as f64 * 50.0;
    let reputation_score = guild.reputation_score as f64;
    let treasury_score = (guild.guild_treasury as f64 / 1000.0).min(1000.0);

    // Apply activity bonus/penalty based on recent activity
    let current_time = Clock::get().unwrap().unix_timestamp;
    let days_since_activity = (current_time - guild.last_activity) / 86400;
    let activity_multiplier = if days_since_activity <= 1 {
        1.2 // 20% bonus for recent activity
    } else if days_since_activity <= 7 {
        1.0 // No penalty/bonus
    } else if days_since_activity <= 30 {
        0.8 // 20% penalty for inactive guilds
    } else {
        0.5 // 50% penalty for very inactive guilds
    };

    (level_score + member_score + activity_score + reputation_score + treasury_score) * activity_multiplier
}

// Guild competition system
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub struct GuildWar {
    pub war_id: u64,
    pub guild_a: Pubkey,
    pub guild_b: Pubkey,
    pub start_time: i64,
    pub end_time: i64,
    pub war_type: u8, // 0: XP battle, 1: Mining battle, 2: Social battle
    pub guild_a_score: u64,
    pub guild_b_score: u64,
    pub winner: Option<Pubkey>,
    pub prize_pool: u64,
    pub is_active: bool,
}

impl GuildWar {
    pub const LEN: usize = 8 + // discriminator
        8 + // war_id
        32 + // guild_a
        32 + // guild_b
        8 + // start_time
        8 + // end_time
        1 + // war_type
        8 + // guild_a_score
        8 + // guild_b_score
        1 + 32 + // winner (Option<Pubkey>)
        8 + // prize_pool
        1; // is_active
}

// Guild treasury management
pub fn distribute_guild_rewards(
    guild: &mut Guild,
    guild_members: &mut [GuildMember],
    total_reward: u64,
) -> Result<()> {
    require!(!guild_members.is_empty(), FinovaError::NoGuildMembers);

    // Calculate total contribution score
    let total_contribution: u64 = guild_members
        .iter()
        .map(|member| member.contribution_score)
        .sum();

    require!(total_contribution > 0, FinovaError::NoContributions);

    // Distribute rewards based on contribution
    for member in guild_members.iter_mut() {
        let member_share = (total_reward as f64 * member.contribution_score as f64 / total_contribution as f64) as u64;
        member.total_tokens_contributed = member.total_tokens_contributed
            .checked_add(member_share)
            .unwrap();
    }

    // Update guild treasury
    guild.guild_treasury = guild.guild_treasury.checked_sub(total_reward).unwrap();

    Ok(())
}

// Constants for guild system (these would be defined in constants.rs)
const GUILD_CREATION_MIN_LEVEL: u32 = 11; // Silver level minimum
const GUILD_JOIN_MIN_LEVEL: u32 = 1;
const GUILD_MAX_MEMBERS: usize = 50;
const GUILD_MAX_OFFICERS: usize = 5;
const GUILD_NAME_MIN_LENGTH: usize = 3;
const GUILD_NAME_MAX_LENGTH: usize = 30;
const GUILD_DESCRIPTION_MAX_LENGTH: usize = 200;
const INITIAL_GUILD_REPUTATION: u64 = 1000;
const GUILD_CHALLENGE_TYPES_COUNT: u8 = 3;
const MIN_CHALLENGE_DURATION: u64 = 3600; // 1 hour
const MAX_CHALLENGE_DURATION: u64 = 604800; // 1 week
const CHALLENGE_SUCCESS_MULTIPLIER: f64 = 1.5;
const CHALLENGE_PARTIAL_MULTIPLIER: f64 = 0.7;
const CHALLENGE_SUCCESS_REPUTATION_BONUS: u64 = 100;
const DAILY_CHALLENGE_BASE_REWARD: u64 = 100;
const WEEKLY_CHALLENGE_BASE_REWARD: u64 = 500;
const MONTHLY_CHALLENGE_BASE_REWARD: u64 = 2000;
const GUILD_LEVEL_BONUS_MULTIPLIER: u64 = 50;
const CHALLENGE_LEVEL_WEIGHT: u32 = 100;
const XP_LEVEL_DIVISOR: u64 = 10000;
const MEMBER_LEVEL_WEIGHT: u32 = 20;
const REPUTATION_LEVEL_DIVISOR: u64 = 1000;
const GUILD_LEVEL_THRESHOLD: u32 = 1000;
const MAX_GUILD_LEVEL: u32 = 100;
