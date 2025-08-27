// programs/finova-core/src/lib.rs

use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Mint};
use std::collections::BTreeMap;

declare_id!("FinovaCoreProgram11111111111111111111111111");

pub mod instructions;
pub mod state;
pub mod events;
pub mod constants;
pub mod errors;
pub mod utils;
pub mod macros;

use instructions::*;
use state::*;
use events::*;
use errors::*;
use constants::*;

#[program]
pub mod finova_core {
    use super::*;

    /// Initialize the Finova Core program
    pub fn initialize(
        ctx: Context<Initialize>,
        network_config: NetworkConfig,
    ) -> Result<()> {
        instructions::initialize::handler(ctx, network_config)
    }

    /// Initialize a user account
    pub fn initialize_user(
        ctx: Context<InitializeUser>,
        user_params: UserInitParams,
    ) -> Result<()> {
        instructions::initialize::initialize_user_handler(ctx, user_params)
    }

    /// Start mining session
    pub fn start_mining(ctx: Context<StartMining>) -> Result<()> {
        instructions::mining::start_mining_handler(ctx)
    }

    /// Claim mining rewards
    pub fn claim_mining_rewards(ctx: Context<ClaimMiningRewards>) -> Result<()> {
        instructions::mining::claim_rewards_handler(ctx)
    }

    /// Update user XP from social activity
    pub fn update_xp(
        ctx: Context<UpdateXP>,
        activity: SocialActivity,
    ) -> Result<()> {
        instructions::xp::update_xp_handler(ctx, activity)
    }

    /// Process referral rewards
    pub fn process_referral(
        ctx: Context<ProcessReferral>,
        referral_code: String,
    ) -> Result<()> {
        instructions::referral::process_referral_handler(ctx, referral_code)
    }

    /// Stake tokens for enhanced rewards
    pub fn stake_tokens(
        ctx: Context<StakeTokens>,
        amount: u64,
        stake_duration: StakeDuration,
    ) -> Result<()> {
        instructions::staking::stake_tokens_handler(ctx, amount, stake_duration)
    }

    /// Unstake tokens
    pub fn unstake_tokens(ctx: Context<UnstakeTokens>) -> Result<()> {
        instructions::staking::unstake_tokens_handler(ctx)
    }

    /// Create a guild
    pub fn create_guild(
        ctx: Context<CreateGuild>,
        guild_params: GuildParams,
    ) -> Result<()> {
        instructions::guild::create_guild_handler(ctx, guild_params)
    }

    /// Join a guild
    pub fn join_guild(
        ctx: Context<JoinGuild>,
        guild_id: u64,
    ) -> Result<()> {
        instructions::guild::join_guild_handler(ctx, guild_id)
    }

    /// Submit governance proposal
    pub fn submit_proposal(
        ctx: Context<SubmitProposal>,
        proposal: ProposalData,
    ) -> Result<()> {
        instructions::governance::submit_proposal_handler(ctx, proposal)
    }

    /// Vote on governance proposal
    pub fn vote_proposal(
        ctx: Context<VoteProposal>,
        proposal_id: u64,
        vote: VoteType,
    ) -> Result<()> {
        instructions::governance::vote_proposal_handler(ctx, proposal_id, vote)
    }

    /// Execute governance proposal
    pub fn execute_proposal(
        ctx: Context<ExecuteProposal>,
        proposal_id: u64,
    ) -> Result<()> {
        instructions::governance::execute_proposal_handler(ctx, proposal_id)
    }

    /// Update quality score (admin only)
    pub fn update_quality_score(
        ctx: Context<UpdateQualityScore>,
        user_pubkey: Pubkey,
        new_score: u16,
        reason: String,
    ) -> Result<()> {
        instructions::quality::update_quality_score_handler(ctx, user_pubkey, new_score, reason)
    }

    /// Report suspicious activity
    pub fn report_suspicious_activity(
        ctx: Context<ReportSuspiciousActivity>,
        reported_user: Pubkey,
        report_type: ReportType,
        evidence: String,
    ) -> Result<()> {
        instructions::anti_bot::report_suspicious_handler(ctx, reported_user, report_type, evidence)
    }

    /// Emergency pause (admin only)
    pub fn emergency_pause(ctx: Context<EmergencyPause>) -> Result<()> {
        let network_state = &mut ctx.accounts.network_state;
        require!(
            ctx.accounts.authority.key() == network_state.admin,
            FinovaError::UnauthorizedAccess
        );
        network_state.is_paused = true;
        
        emit!(EmergencyPauseEvent {
            timestamp: Clock::get()?.unix_timestamp,
            admin: ctx.accounts.authority.key(),
        });
        
        Ok(())
    }

    /// Resume operations (admin only)
    pub fn resume_operations(ctx: Context<ResumeOperations>) -> Result<()> {
        let network_state = &mut ctx.accounts.network_state;
        require!(
            ctx.accounts.authority.key() == network_state.admin,
            FinovaError::UnauthorizedAccess
        );
        network_state.is_paused = false;
        
        emit!(ResumeOperationsEvent {
            timestamp: Clock::get()?.unix_timestamp,
            admin: ctx.accounts.authority.key(),
        });
        
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
        init,
        payer = authority,
        space = NetworkState::SIZE,
        seeds = [b"network_state"],
        bump
    )]
    pub network_state: Account<'info, NetworkState>,
    
    #[account(
        init,
        payer = authority,
        space = RewardPool::SIZE,
        seeds = [b"reward_pool"],
        bump
    )]
    pub reward_pool: Account<'info, RewardPool>,
    
    #[account(mut)]
    pub authority: Signer<'info>,
    
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct InitializeUser<'info> {
    #[account(
        init,
        payer = user,
        space = UserState::SIZE,
        seeds = [b"user", user.key().as_ref()],
        bump
    )]
    pub user_state: Account<'info, UserState>,
    
    #[account(
        init,
        payer = user,
        space = MiningState::SIZE,
        seeds = [b"mining", user.key().as_ref()],
        bump
    )]
    pub mining_state: Account<'info, MiningState>,
    
    #[account(
        init,
        payer = user,
        space = XPState::SIZE,
        seeds = [b"xp", user.key().as_ref()],
        bump
    )]
    pub xp_state: Account<'info, XPState>,
    
    #[account(
        init,
        payer = user,
        space = ReferralState::SIZE,
        seeds = [b"referral", user.key().as_ref()],
        bump
    )]
    pub referral_state: Account<'info, ReferralState>,
    
    #[account(
        seeds = [b"network_state"],
        bump,
        constraint = !network_state.is_paused @ FinovaError::SystemPaused
    )]
    pub network_state: Account<'info, NetworkState>,
    
    #[account(mut)]
    pub user: Signer<'info>,
    
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct StartMining<'info> {
    #[account(
        mut,
        seeds = [b"user", user.key().as_ref()],
        bump,
        constraint = user_state.is_active @ FinovaError::UserNotActive
    )]
    pub user_state: Account<'info, UserState>,
    
    #[account(
        mut,
        seeds = [b"mining", user.key().as_ref()],
        bump
    )]
    pub mining_state: Account<'info, MiningState>,
    
    #[account(
        seeds = [b"network_state"],
        bump,
        constraint = !network_state.is_paused @ FinovaError::SystemPaused
    )]
    pub network_state: Account<'info, NetworkState>,
    
    pub user: Signer<'info>,
    pub clock: Sysvar<'info, Clock>,
}

#[derive(Accounts)]
pub struct ClaimMiningRewards<'info> {
    #[account(
        mut,
        seeds = [b"user", user.key().as_ref()],
        bump
    )]
    pub user_state: Account<'info, UserState>,
    
    #[account(
        mut,
        seeds = [b"mining", user.key().as_ref()],
        bump
    )]
    pub mining_state: Account<'info, MiningState>,
    
    #[account(
        mut,
        seeds = [b"reward_pool"],
        bump
    )]
    pub reward_pool: Account<'info, RewardPool>,
    
    #[account(
        seeds = [b"network_state"],
        bump,
        constraint = !network_state.is_paused @ FinovaError::SystemPaused
    )]
    pub network_state: Account<'info, NetworkState>,
    
    #[account(mut)]
    pub user_token_account: Account<'info, TokenAccount>,
    
    #[account(mut)]
    pub reward_mint: Account<'info, Mint>,
    
    pub user: Signer<'info>,
    pub token_program: Program<'info, Token>,
    pub clock: Sysvar<'info, Clock>,
}

#[derive(Accounts)]
pub struct UpdateXP<'info> {
    #[account(
        mut,
        seeds = [b"user", user.key().as_ref()],
        bump
    )]
    pub user_state: Account<'info, UserState>,
    
    #[account(
        mut,
        seeds = [b"xp", user.key().as_ref()],
        bump
    )]
    pub xp_state: Account<'info, XPState>,
    
    #[account(
        seeds = [b"network_state"],
        bump,
        constraint = !network_state.is_paused @ FinovaError::SystemPaused
    )]
    pub network_state: Account<'info, NetworkState>,
    
    pub user: Signer<'info>,
    pub clock: Sysvar<'info, Clock>,
}

#[derive(Accounts)]
pub struct ProcessReferral<'info> {
    #[account(
        mut,
        seeds = [b"user", referrer.key().as_ref()],
        bump
    )]
    pub referrer_state: Account<'info, UserState>,
    
    #[account(
        mut,
        seeds = [b"referral", referrer.key().as_ref()],
        bump
    )]
    pub referrer_referral_state: Account<'info, ReferralState>,
    
    #[account(
        mut,
        seeds = [b"user", referee.key().as_ref()],
        bump
    )]
    pub referee_state: Account<'info, UserState>,
    
    #[account(
        seeds = [b"network_state"],
        bump,
        constraint = !network_state.is_paused @ FinovaError::SystemPaused
    )]
    pub network_state: Account<'info, NetworkState>,
    
    pub referrer: Signer<'info>,
    /// CHECK: This account is verified through the referral code
    pub referee: AccountInfo<'info>,
    pub clock: Sysvar<'info, Clock>,
}

#[derive(Accounts)]
pub struct StakeTokens<'info> {
    #[account(
        mut,
        seeds = [b"user", user.key().as_ref()],
        bump
    )]
    pub user_state: Account<'info, UserState>,
    
    #[account(
        init_if_needed,
        payer = user,
        space = StakingState::SIZE,
        seeds = [b"staking", user.key().as_ref()],
        bump
    )]
    pub staking_state: Account<'info, StakingState>,
    
    #[account(
        mut,
        constraint = user_token_account.owner == user.key()
    )]
    pub user_token_account: Account<'info, TokenAccount>,
    
    #[account(
        mut,
        seeds = [b"staking_vault"],
        bump
    )]
    pub staking_vault: Account<'info, TokenAccount>,
    
    #[account(
        seeds = [b"network_state"],
        bump,
        constraint = !network_state.is_paused @ FinovaError::SystemPaused
    )]
    pub network_state: Account<'info, NetworkState>,
    
    #[account(mut)]
    pub user: Signer<'info>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub clock: Sysvar<'info, Clock>,
}

#[derive(Accounts)]
pub struct UnstakeTokens<'info> {
    #[account(
        mut,
        seeds = [b"user", user.key().as_ref()],
        bump
    )]
    pub user_state: Account<'info, UserState>,
    
    #[account(
        mut,
        seeds = [b"staking", user.key().as_ref()],
        bump,
        constraint = staking_state.staked_amount > 0 @ FinovaError::NoStakedTokens
    )]
    pub staking_state: Account<'info, StakingState>,
    
    #[account(mut)]
    pub user_token_account: Account<'info, TokenAccount>,
    
    #[account(
        mut,
        seeds = [b"staking_vault"],
        bump
    )]
    pub staking_vault: Account<'info, TokenAccount>,
    
    pub user: Signer<'info>,
    pub token_program: Program<'info, Token>,
    pub clock: Sysvar<'info, Clock>,
}

#[derive(Accounts)]
pub struct CreateGuild<'info> {
    #[account(
        init,
        payer = creator,
        space = GuildState::SIZE,
        seeds = [b"guild", &network_state.next_guild_id.to_le_bytes()],
        bump
    )]
    pub guild_state: Account<'info, GuildState>,
    
    #[account(
        mut,
        seeds = [b"network_state"],
        bump,
        constraint = !network_state.is_paused @ FinovaError::SystemPaused
    )]
    pub network_state: Account<'info, NetworkState>,
    
    #[account(
        mut,
        seeds = [b"user", creator.key().as_ref()],
        bump,
        constraint = user_state.xp_level >= MIN_GUILD_CREATION_LEVEL @ FinovaError::InsufficientLevel
    )]
    pub user_state: Account<'info, UserState>,
    
    #[account(mut)]
    pub creator: Signer<'info>,
    pub system_program: Program<'info, System>,
    pub clock: Sysvar<'info, Clock>,
}

#[derive(Accounts)]
pub struct JoinGuild<'info> {
    #[account(
        mut,
        seeds = [b"guild", &guild_id.to_le_bytes()],
        bump,
        constraint = guild_state.is_active @ FinovaError::GuildNotActive,
        constraint = guild_state.member_count < MAX_GUILD_MEMBERS @ FinovaError::GuildFull
    )]
    pub guild_state: Account<'info, GuildState>,
    
    #[account(
        mut,
        seeds = [b"user", user.key().as_ref()],
        bump,
        constraint = user_state.guild_id == 0 @ FinovaError::AlreadyInGuild
    )]
    pub user_state: Account<'info, UserState>,
    
    pub user: Signer<'info>,
    pub clock: Sysvar<'info, Clock>,
}

#[derive(Accounts)]
pub struct SubmitProposal<'info> {
    #[account(
        init,
        payer = proposer,
        space = ProposalState::SIZE,
        seeds = [b"proposal", &network_state.next_proposal_id.to_le_bytes()],
        bump
    )]
    pub proposal_state: Account<'info, ProposalState>,
    
    #[account(
        mut,
        seeds = [b"network_state"],
        bump,
        constraint = !network_state.is_paused @ FinovaError::SystemPaused
    )]
    pub network_state: Account<'info, NetworkState>,
    
    #[account(
        seeds = [b"user", proposer.key().as_ref()],
        bump,
        constraint = user_state.governance_weight >= MIN_PROPOSAL_WEIGHT @ FinovaError::InsufficientGovernanceWeight
    )]
    pub user_state: Account<'info, UserState>,
    
    #[account(mut)]
    pub proposer: Signer<'info>,
    pub system_program: Program<'info, System>,
    pub clock: Sysvar<'info, Clock>,
}

#[derive(Accounts)]
pub struct VoteProposal<'info> {
    #[account(
        mut,
        seeds = [b"proposal", &proposal_id.to_le_bytes()],
        bump,
        constraint = proposal_state.is_active @ FinovaError::ProposalNotActive,
        constraint = Clock::get()?.unix_timestamp <= proposal_state.voting_end_time @ FinovaError::VotingPeriodEnded
    )]
    pub proposal_state: Account<'info, ProposalState>,
    
    #[account(
        seeds = [b"user", voter.key().as_ref()],
        bump,
        constraint = user_state.governance_weight > 0 @ FinovaError::NoVotingPower
    )]
    pub user_state: Account<'info, UserState>,
    
    #[account(
        init_if_needed,
        payer = voter,
        space = VoteRecord::SIZE,
        seeds = [b"vote", &proposal_id.to_le_bytes(), voter.key().as_ref()],
        bump
    )]
    pub vote_record: Account<'info, VoteRecord>,
    
    #[account(mut)]
    pub voter: Signer<'info>,
    pub system_program: Program<'info, System>,
    pub clock: Sysvar<'info, Clock>,
}

#[derive(Accounts)]
pub struct ExecuteProposal<'info> {
    #[account(
        mut,
        seeds = [b"proposal", &proposal_id.to_le_bytes()],
        bump,
        constraint = proposal_state.is_active @ FinovaError::ProposalNotActive,
        constraint = Clock::get()?.unix_timestamp > proposal_state.voting_end_time @ FinovaError::VotingStillActive,
        constraint = proposal_state.votes_for > proposal_state.votes_against @ FinovaError::ProposalRejected
    )]
    pub proposal_state: Account<'info, ProposalState>,
    
    #[account(
        mut,
        seeds = [b"network_state"],
        bump
    )]
    pub network_state: Account<'info, NetworkState>,
    
    pub executor: Signer<'info>,
    pub clock: Sysvar<'info, Clock>,
}

#[derive(Accounts)]
pub struct UpdateQualityScore<'info> {
    #[account(
        mut,
        seeds = [b"user", user_pubkey.as_ref()],
        bump
    )]
    pub user_state: Account<'info, UserState>,
    
    #[account(
        seeds = [b"network_state"],
        bump,
        constraint = network_state.admin == authority.key() @ FinovaError::UnauthorizedAccess
    )]
    pub network_state: Account<'info, NetworkState>,
    
    pub authority: Signer<'info>,
    pub clock: Sysvar<'info, Clock>,
}

#[derive(Accounts)]
pub struct ReportSuspiciousActivity<'info> {
    #[account(
        init,
        payer = reporter,
        space = SuspiciousActivityReport::SIZE,
        seeds = [
            b"report",
            reporter.key().as_ref(),
            reported_user.as_ref(),
            &Clock::get()?.unix_timestamp.to_le_bytes()
        ],
        bump
    )]
    pub report: Account<'info, SuspiciousActivityReport>,
    
    #[account(
        seeds = [b"user", reporter.key().as_ref()],
        bump,
        constraint = user_state.is_active @ FinovaError::UserNotActive
    )]
    pub user_state: Account<'info, UserState>,
    
    #[account(mut)]
    pub reporter: Signer<'info>,
    pub system_program: Program<'info, System>,
    pub clock: Sysvar<'info, Clock>,
}

#[derive(Accounts)]
pub struct EmergencyPause<'info> {
    #[account(
        mut,
        seeds = [b"network_state"],
        bump
    )]
    pub network_state: Account<'info, NetworkState>,
    
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct ResumeOperations<'info> {
    #[account(
        mut,
        seeds = [b"network_state"],
        bump
    )]
    pub network_state: Account<'info, NetworkState>,
    
    pub authority: Signer<'info>,
}
