// programs/finova-core/src/instructions/governance.rs

use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};
use crate::constants::*;
use crate::errors::*;
use crate::state::*;
use crate::utils::*;

/// Initialize governance system
#[derive(Accounts)]
pub struct InitializeGovernance<'info> {
    #[account(
        init,
        payer = authority,
        space = 8 + GovernanceConfig::INIT_SPACE,
        seeds = [GOVERNANCE_SEED],
        bump
    )]
    pub governance_config: Account<'info, GovernanceConfig>,
    
    #[account(mut)]
    pub authority: Signer<'info>,
    
    pub system_program: Program<'info, System>,
}

/// Create a governance proposal
#[derive(Accounts)]
#[instruction(proposal_id: u64)]
pub struct CreateProposal<'info> {
    #[account(
        init,
        payer = proposer,
        space = 8 + Proposal::INIT_SPACE,
        seeds = [PROPOSAL_SEED, proposal_id.to_le_bytes().as_ref()],
        bump
    )]
    pub proposal: Account<'info, Proposal>,
    
    #[account(
        mut,
        seeds = [GOVERNANCE_SEED],
        bump
    )]
    pub governance_config: Account<'info, GovernanceConfig>,
    
    #[account(
        mut,
        seeds = [USER_SEED, proposer.key().as_ref()],
        bump
    )]
    pub user_account: Account<'info, UserAccount>,
    
    #[account(
        seeds = [STAKING_SEED, proposer.key().as_ref()],
        bump
    )]
    pub staking_account: Account<'info, StakingAccount>,
    
    #[account(mut)]
    pub proposer: Signer<'info>,
    
    pub system_program: Program<'info, System>,
}

/// Cast a vote on a proposal
#[derive(Accounts)]
#[instruction(proposal_id: u64)]
pub struct CastVote<'info> {
    #[account(
        mut,
        seeds = [PROPOSAL_SEED, proposal_id.to_le_bytes().as_ref()],
        bump
    )]
    pub proposal: Account<'info, Proposal>,
    
    #[account(
        init_if_needed,
        payer = voter,
        space = 8 + Vote::INIT_SPACE,
        seeds = [VOTE_SEED, proposal_id.to_le_bytes().as_ref(), voter.key().as_ref()],
        bump
    )]
    pub vote: Account<'info, Vote>,
    
    #[account(
        mut,
        seeds = [USER_SEED, voter.key().as_ref()],
        bump
    )]
    pub user_account: Account<'info, UserAccount>,
    
    #[account(
        seeds = [STAKING_SEED, voter.key().as_ref()],
        bump
    )]
    pub staking_account: Account<'info, StakingAccount>,
    
    #[account(
        seeds = [XP_SEED, voter.key().as_ref()],
        bump
    )]
    pub xp_account: Account<'info, XPAccount>,
    
    #[account(
        seeds = [REFERRAL_SEED, voter.key().as_ref()],
        bump
    )]
    pub referral_account: Account<'info, ReferralAccount>,
    
    #[account(mut)]
    pub voter: Signer<'info>,
    
    pub system_program: Program<'info, System>,
}

/// Execute a passed proposal
#[derive(Accounts)]
#[instruction(proposal_id: u64)]
pub struct ExecuteProposal<'info> {
    #[account(
        mut,
        seeds = [PROPOSAL_SEED, proposal_id.to_le_bytes().as_ref()],
        bump,
        constraint = proposal.status == ProposalStatus::Passed @ FinovaError::ProposalNotPassed
    )]
    pub proposal: Account<'info, Proposal>,
    
    #[account(
        mut,
        seeds = [GOVERNANCE_SEED],
        bump
    )]
    pub governance_config: Account<'info, GovernanceConfig>,
    
    #[account(mut)]
    pub executor: Signer<'info>,
    
    pub system_program: Program<'info, System>,
}

/// Update governance parameters
#[derive(Accounts)]
pub struct UpdateGovernanceConfig<'info> {
    #[account(
        mut,
        seeds = [GOVERNANCE_SEED],
        bump,
        has_one = authority @ FinovaError::Unauthorized
    )]
    pub governance_config: Account<'info, GovernanceConfig>,
    
    #[account(mut)]
    pub authority: Signer<'info>,
}

/// Cancel a proposal (only by proposer before voting ends)
#[derive(Accounts)]
#[instruction(proposal_id: u64)]
pub struct CancelProposal<'info> {
    #[account(
        mut,
        seeds = [PROPOSAL_SEED, proposal_id.to_le_bytes().as_ref()],
        bump,
        has_one = proposer @ FinovaError::Unauthorized,
        constraint = proposal.status == ProposalStatus::Active @ FinovaError::ProposalNotActive
    )]
    pub proposal: Account<'info, Proposal>,
    
    #[account(mut)]
    pub proposer: Signer<'info>,
}

/// Finalize voting (after voting period ends)
#[derive(Accounts)]
#[instruction(proposal_id: u64)]
pub struct FinalizeVoting<'info> {
    #[account(
        mut,
        seeds = [PROPOSAL_SEED, proposal_id.to_le_bytes().as_ref()],
        bump,
        constraint = proposal.status == ProposalStatus::Active @ FinovaError::ProposalNotActive
    )]
    pub proposal: Account<'info, Proposal>,
    
    #[account(
        seeds = [GOVERNANCE_SEED],
        bump
    )]
    pub governance_config: Account<'info, GovernanceConfig>,
    
    pub finalizer: Signer<'info>,
}

/// Delegate voting power to another user
#[derive(Accounts)]
pub struct DelegateVotingPower<'info> {
    #[account(
        mut,
        seeds = [USER_SEED, delegator.key().as_ref()],
        bump
    )]
    pub delegator_account: Account<'info, UserAccount>,
    
    #[account(
        mut,
        seeds = [USER_SEED, delegate.as_ref()],
        bump
    )]
    pub delegate_account: Account<'info, UserAccount>,
    
    #[account(mut)]
    pub delegator: Signer<'info>,
    
    /// CHECK: This is validated in the instruction
    pub delegate: AccountInfo<'info>,
}

/// Governance state definitions
#[account]
pub struct GovernanceConfig {
    pub authority: Pubkey,
    pub voting_period: u64,        // Duration in slots
    pub min_voting_power: u64,     // Minimum power to create proposals
    pub quorum_threshold: u64,     // Minimum participation for validity
    pub approval_threshold: u64,   // Percentage needed to pass (in basis points)
    pub execution_delay: u64,      // Delay between passing and execution
    pub proposal_count: u64,       // Total proposals created
    pub active_proposals: u64,     // Currently active proposals
    pub proposal_deposit: u64,     // Required deposit to create proposal
    pub treasury_balance: u64,     // DAO treasury balance
    pub bump: u8,
}

impl GovernanceConfig {
    pub const INIT_SPACE: usize = 32 + 8 * 9 + 1; // 105 bytes
}

#[account]
pub struct Proposal {
    pub id: u64,
    pub proposer: Pubkey,
    pub title: String,             // Max 64 chars
    pub description: String,       // Max 512 chars
    pub proposal_type: ProposalType,
    pub status: ProposalStatus,
    pub votes_for: u64,
    pub votes_against: u64,
    pub votes_abstain: u64,
    pub total_voting_power: u64,
    pub created_at: i64,
    pub voting_starts_at: i64,
    pub voting_ends_at: i64,
    pub executed_at: Option<i64>,
    pub execution_data: Vec<u8>,   // Serialized execution parameters
    pub deposit_amount: u64,
    pub deposit_returned: bool,
    pub bump: u8,
}

impl Proposal {
    pub const INIT_SPACE: usize = 8 + 32 + 4 + 64 + 4 + 512 + 1 + 1 + 8 * 6 + 8 + 1 + 8 * 3 + 4 + 256 + 8 + 1 + 1; // ~1000 bytes
}

#[account]
pub struct Vote {
    pub proposal_id: u64,
    pub voter: Pubkey,
    pub vote_type: VoteType,
    pub voting_power: u64,
    pub delegated_power: u64,      // Power delegated from others
    pub timestamp: i64,
    pub reason: Option<String>,    // Optional voting reason (max 256 chars)
    pub bump: u8,
}

impl Vote {
    pub const INIT_SPACE: usize = 8 + 32 + 1 + 8 * 2 + 8 + 4 + 256 + 1; // ~320 bytes
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub enum ProposalType {
    ParameterChange,    // Change mining rates, fees, etc.
    TreasurySpending,   // Allocate treasury funds
    FeatureAddition,    // Add new platform integrations
    UpgradeContract,    // Smart contract upgrades
    CommunityInitiative, // Community programs, events
    EmergencyAction,    // Emergency pause, security measures
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub enum ProposalStatus {
    Draft,      // Being prepared
    Active,     // Currently voting
    Passed,     // Passed, awaiting execution
    Failed,     // Failed to meet requirements
    Executed,   // Successfully executed
    Cancelled,  // Cancelled by proposer
    Expired,    // Expired without execution
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub enum VoteType {
    For,
    Against,
    Abstain,
}

/// Initialize governance system
pub fn initialize_governance(
    ctx: Context<InitializeGovernance>,
    voting_period: u64,
    min_voting_power: u64,
    quorum_threshold: u64,
    approval_threshold: u64,
    execution_delay: u64,
    proposal_deposit: u64,
) -> Result<()> {
    let governance_config = &mut ctx.accounts.governance_config;
    
    // Validate parameters
    require!(
        approval_threshold <= 10000 && approval_threshold >= 5000,
        FinovaError::InvalidParameter
    );
    require!(
        quorum_threshold <= 10000 && quorum_threshold >= 1000,
        FinovaError::InvalidParameter
    );
    require!(voting_period >= MIN_VOTING_PERIOD, FinovaError::VotingPeriodTooShort);
    require!(voting_period <= MAX_VOTING_PERIOD, FinovaError::VotingPeriodTooLong);
    
    governance_config.authority = ctx.accounts.authority.key();
    governance_config.voting_period = voting_period;
    governance_config.min_voting_power = min_voting_power;
    governance_config.quorum_threshold = quorum_threshold;
    governance_config.approval_threshold = approval_threshold;
    governance_config.execution_delay = execution_delay;
    governance_config.proposal_count = 0;
    governance_config.active_proposals = 0;
    governance_config.proposal_deposit = proposal_deposit;
    governance_config.treasury_balance = 0;
    governance_config.bump = ctx.bumps.governance_config;
    
    emit!(GovernanceInitialized {
        authority: ctx.accounts.authority.key(),
        voting_period,
        min_voting_power,
        quorum_threshold,
        approval_threshold,
    });
    
    Ok(())
}

/// Create a new governance proposal
pub fn create_proposal(
    ctx: Context<CreateProposal>,
    proposal_id: u64,
    title: String,
    description: String,
    proposal_type: ProposalType,
    execution_data: Vec<u8>,
) -> Result<()> {
    let proposal = &mut ctx.accounts.proposal;
    let governance_config = &mut ctx.accounts.governance_config;
    let user_account = &ctx.accounts.user_account;
    let staking_account = &ctx.accounts.staking_account;
    
    let clock = Clock::get()?;
    let current_time = clock.unix_timestamp;
    
    // Validate proposal ID
    require!(
        proposal_id == governance_config.proposal_count + 1,
        FinovaError::InvalidProposalId
    );
    
    // Validate input lengths
    require!(title.len() <= 64, FinovaError::TitleTooLong);
    require!(description.len() <= 512, FinovaError::DescriptionTooLong);
    require!(execution_data.len() <= 256, FinovaError::ExecutionDataTooLong);
    
    // Calculate voting power
    let voting_power = calculate_voting_power(user_account, staking_account, &ctx.accounts.proposer.key())?;
    
    // Check minimum voting power requirement
    require!(
        voting_power >= governance_config.min_voting_power,
        FinovaError::InsufficientVotingPower
    );
    
    // Check if user has enough for deposit (this would need token transfer in full implementation)
    require!(
        user_account.fin_balance >= governance_config.proposal_deposit,
        FinovaError::InsufficientBalance
    );
    
    // Initialize proposal
    proposal.id = proposal_id;
    proposal.proposer = ctx.accounts.proposer.key();
    proposal.title = title;
    proposal.description = description;
    proposal.proposal_type = proposal_type.clone();
    proposal.status = ProposalStatus::Active;
    proposal.votes_for = 0;
    proposal.votes_against = 0;
    proposal.votes_abstain = 0;
    proposal.total_voting_power = 0;
    proposal.created_at = current_time;
    proposal.voting_starts_at = current_time;
    proposal.voting_ends_at = current_time + governance_config.voting_period as i64;
    proposal.executed_at = None;
    proposal.execution_data = execution_data;
    proposal.deposit_amount = governance_config.proposal_deposit;
    proposal.deposit_returned = false;
    proposal.bump = ctx.bumps.proposal;
    
    // Update governance config
    governance_config.proposal_count += 1;
    governance_config.active_proposals += 1;
    
    emit!(ProposalCreated {
        proposal_id,
        proposer: ctx.accounts.proposer.key(),
        title: proposal.title.clone(),
        proposal_type,
        voting_ends_at: proposal.voting_ends_at,
    });
    
    Ok(())
}

/// Cast a vote on a proposal
pub fn cast_vote(
    ctx: Context<CastVote>,
    proposal_id: u64,
    vote_type: VoteType,
    reason: Option<String>,
) -> Result<()> {
    let proposal = &mut ctx.accounts.proposal;
    let vote = &mut ctx.accounts.vote;
    let user_account = &ctx.accounts.user_account;
    let staking_account = &ctx.accounts.staking_account;
    let xp_account = &ctx.accounts.xp_account;
    let referral_account = &ctx.accounts.referral_account;
    
    let clock = Clock::get()?;
    let current_time = clock.unix_timestamp;
    
    // Validate voting period
    require!(
        current_time >= proposal.voting_starts_at && current_time <= proposal.voting_ends_at,
        FinovaError::VotingPeriodEnded
    );
    
    require!(proposal.status == ProposalStatus::Active, FinovaError::ProposalNotActive);
    
    // Validate reason length if provided
    if let Some(ref r) = reason {
        require!(r.len() <= 256, FinovaError::ReasonTooLong);
    }
    
    // Calculate comprehensive voting power
    let base_voting_power = calculate_voting_power(user_account, staking_account, &ctx.accounts.voter.key())?;
    let xp_multiplier = calculate_xp_voting_multiplier(xp_account)?;
    let rp_multiplier = calculate_rp_voting_multiplier(referral_account)?;
    
    let total_voting_power = (base_voting_power as f64 * xp_multiplier * rp_multiplier) as u64;
    
    // Get delegated power to this user
    let delegated_power = user_account.delegated_voting_power;
    let final_voting_power = total_voting_power + delegated_power;
    
    require!(final_voting_power > 0, FinovaError::NoVotingPower);
    
    // Update vote counts (subtract old vote if changing)
    if vote.voting_power > 0 {
        match vote.vote_type {
            VoteType::For => proposal.votes_for -= vote.voting_power + vote.delegated_power,
            VoteType::Against => proposal.votes_against -= vote.voting_power + vote.delegated_power,
            VoteType::Abstain => proposal.votes_abstain -= vote.voting_power + vote.delegated_power,
        }
        proposal.total_voting_power -= vote.voting_power + vote.delegated_power;
    }
    
    // Add new vote
    match vote_type {
        VoteType::For => proposal.votes_for += final_voting_power,
        VoteType::Against => proposal.votes_against += final_voting_power,
        VoteType::Abstain => proposal.votes_abstain += final_voting_power,
    }
    proposal.total_voting_power += final_voting_power;
    
    // Update vote record
    vote.proposal_id = proposal_id;
    vote.voter = ctx.accounts.voter.key();
    vote.vote_type = vote_type.clone();
    vote.voting_power = total_voting_power;
    vote.delegated_power = delegated_power;
    vote.timestamp = current_time;
    vote.reason = reason;
    vote.bump = ctx.bumps.vote;
    
    emit!(VoteCast {
        proposal_id,
        voter: ctx.accounts.voter.key(),
        vote_type,
        voting_power: final_voting_power,
        timestamp: current_time,
    });
    
    Ok(())
}

/// Finalize voting after period ends
pub fn finalize_voting(
    ctx: Context<FinalizeVoting>,
    proposal_id: u64,
) -> Result<()> {
    let proposal = &mut ctx.accounts.proposal;
    let governance_config = &ctx.accounts.governance_config;
    
    let clock = Clock::get()?;
    let current_time = clock.unix_timestamp;
    
    // Check if voting period has ended
    require!(current_time > proposal.voting_ends_at, FinovaError::VotingStillActive);
    
    // Calculate results
    let total_votes = proposal.votes_for + proposal.votes_against + proposal.votes_abstain;
    let quorum_met = total_votes >= governance_config.quorum_threshold;
    
    let approval_rate = if proposal.votes_for + proposal.votes_against > 0 {
        (proposal.votes_for * 10000) / (proposal.votes_for + proposal.votes_against)
    } else {
        0
    };
    
    let passed = quorum_met && approval_rate >= governance_config.approval_threshold;
    
    // Update proposal status
    proposal.status = if passed {
        ProposalStatus::Passed
    } else {
        ProposalStatus::Failed
    };
    
    // Update governance config
    ctx.accounts.governance_config.active_proposals -= 1;
    
    emit!(VotingFinalized {
        proposal_id,
        passed,
        votes_for: proposal.votes_for,
        votes_against: proposal.votes_against,
        votes_abstain: proposal.votes_abstain,
        total_voting_power: proposal.total_voting_power,
    });
    
    Ok(())
}

/// Execute a passed proposal
pub fn execute_proposal(
    ctx: Context<ExecuteProposal>,
    proposal_id: u64,
) -> Result<()> {
    let proposal = &mut ctx.accounts.proposal;
    let governance_config = &mut ctx.accounts.governance_config;
    
    let clock = Clock::get()?;
    let current_time = clock.unix_timestamp;
    
    // Check execution delay
    require!(
        current_time >= proposal.voting_ends_at + governance_config.execution_delay as i64,
        FinovaError::ExecutionDelayNotMet
    );
    
    // Execute based on proposal type
    match proposal.proposal_type {
        ProposalType::ParameterChange => {
            execute_parameter_change(proposal, governance_config)?;
        },
        ProposalType::TreasurySpending => {
            execute_treasury_spending(proposal, governance_config)?;
        },
        ProposalType::FeatureAddition => {
            execute_feature_addition(proposal)?;
        },
        ProposalType::UpgradeContract => {
            execute_contract_upgrade(proposal)?;
        },
        ProposalType::CommunityInitiative => {
            execute_community_initiative(proposal, governance_config)?;
        },
        ProposalType::EmergencyAction => {
            execute_emergency_action(proposal)?;
        },
    }
    
    // Update proposal
    proposal.status = ProposalStatus::Executed;
    proposal.executed_at = Some(current_time);
    
    emit!(ProposalExecuted {
        proposal_id,
        proposal_type: proposal.proposal_type.clone(),
        executor: ctx.accounts.executor.key(),
        executed_at: current_time,
    });
    
    Ok(())
}

/// Delegate voting power to another user
pub fn delegate_voting_power(
    ctx: Context<DelegateVotingPower>,
    delegate: Pubkey,
) -> Result<()> {
    let delegator_account = &mut ctx.accounts.delegator_account;
    let delegate_account = &mut ctx.accounts.delegate_account;
    
    require!(delegate != ctx.accounts.delegator.key(), FinovaError::CannotDelegateToSelf);
    
    // Remove previous delegation if exists
    if let Some(previous_delegate) = delegator_account.voting_delegate {
        // In a full implementation, we'd need to find and update the previous delegate's account
        delegator_account.delegated_voting_power = 0;
    }
    
    // Calculate current voting power
    let voting_power = delegator_account.fin_balance / 1000; // Simplified calculation
    
    // Update delegation
    delegator_account.voting_delegate = Some(delegate);
    delegate_account.delegated_voting_power += voting_power;
    
    emit!(VotingPowerDelegated {
        delegator: ctx.accounts.delegator.key(),
        delegate,
        voting_power,
    });
    
    Ok(())
}

/// Cancel a proposal (only by proposer)
pub fn cancel_proposal(
    ctx: Context<CancelProposal>,
    proposal_id: u64,
) -> Result<()> {
    let proposal = &mut ctx.accounts.proposal;
    
    let clock = Clock::get()?;
    let current_time = clock.unix_timestamp;
    
    // Only allow cancellation before voting ends
    require!(current_time < proposal.voting_ends_at, FinovaError::CannotCancelAfterVoting);
    
    proposal.status = ProposalStatus::Cancelled;
    
    emit!(ProposalCancelled {
        proposal_id,
        proposer: ctx.accounts.proposer.key(),
    });
    
    Ok(())
}

// Helper functions for proposal execution
fn execute_parameter_change(proposal: &Proposal, governance_config: &mut GovernanceConfig) -> Result<()> {
    // Parse execution data and update parameters
    // This is a simplified implementation
    msg!("Executing parameter change for proposal {}", proposal.id);
    Ok(())
}

fn execute_treasury_spending(proposal: &Proposal, governance_config: &mut GovernanceConfig) -> Result<()> {
    // Execute treasury spending
    msg!("Executing treasury spending for proposal {}", proposal.id);
    Ok(())
}

fn execute_feature_addition(proposal: &Proposal) -> Result<()> {
    // Add new features/integrations
    msg!("Executing feature addition for proposal {}", proposal.id);
    Ok(())
}

fn execute_contract_upgrade(proposal: &Proposal) -> Result<()> {
    // Handle contract upgrades
    msg!("Executing contract upgrade for proposal {}", proposal.id);
    Ok(())
}

fn execute_community_initiative(proposal: &Proposal, governance_config: &mut GovernanceConfig) -> Result<()> {
    // Execute community initiatives
    msg!("Executing community initiative for proposal {}", proposal.id);
    Ok(())
}

fn execute_emergency_action(proposal: &Proposal) -> Result<()> {
    // Execute emergency actions
    msg!("Executing emergency action for proposal {}", proposal.id);
    Ok(())
}

// Helper function to calculate comprehensive voting power
fn calculate_voting_power(
    user_account: &UserAccount,
    staking_account: &StakingAccount,
    user_key: &Pubkey,
) -> Result<u64> {
    let base_power = user_account.fin_balance / 1000; // 1000 FIN = 1 voting power
    let staking_bonus = staking_account.staked_amount / 500; // Staked tokens have double weight
    
    Ok(base_power + staking_bonus)
}

fn calculate_xp_voting_multiplier(xp_account: &XPAccount) -> Result<f64> {
    let level_multiplier = 1.0 + (xp_account.level as f64 / 100.0);
    Ok(level_multiplier.min(2.0)) // Cap at 2x multiplier
}

fn calculate_rp_voting_multiplier(referral_account: &ReferralAccount) -> Result<f64> {
    let tier_multiplier = match referral_account.tier {
        ReferralTier::Explorer => 1.0,
        ReferralTier::Connector => 1.1,
        ReferralTier::Influencer => 1.2,
        ReferralTier::Leader => 1.3,
        ReferralTier::Ambassador => 1.5,
    };
    Ok(tier_multiplier)
}

// Events
#[event]
pub struct GovernanceInitialized {
    pub authority: Pubkey,
    pub voting_period: u64,
    pub min_voting_power: u64,
    pub quorum_threshold: u64,
    pub approval_threshold: u64,
}

#[event]
pub struct ProposalCreated {
    pub proposal_id: u64,
    pub proposer: Pubkey,
    pub title: String,
    pub proposal_type: ProposalType,
    pub voting_ends_at: i64,
}

#[event]
pub struct VoteCast {
    pub proposal_id: u64,
    pub voter: Pubkey,
    pub vote_type: VoteType,
    pub voting_power: u64,
    pub timestamp: i64,
}

#[event]
pub struct VotingFinalized {
    pub proposal_id: u64,
    pub passed: bool,
    pub votes_for: u64,
    pub votes_against: u64,
    pub votes_abstain: u64,
    pub total_voting_power: u64,
}

#[event]
pub struct ProposalExecuted {
    pub proposal_id: u64,
    pub proposal_type: ProposalType,
    pub executor: Pubkey,
    pub executed_at: i64,
}

#[event]
pub struct VotingPowerDelegated {
    pub delegator: Pubkey,
    pub delegate: Pubkey,
    pub voting_power: u64,
}

#[event]
pub struct ProposalCancelled {
    pub proposal_id: u64,
    pub proposer: Pubkey,
}
