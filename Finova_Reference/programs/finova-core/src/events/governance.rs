// programs/finova-core/src/events/governance.rs

use anchor_lang::prelude::*;

/// Governance-related events for the Finova Network
/// Handles DAO governance, voting, proposals, and community decisions

#[event]
pub struct ProposalCreated {
    /// Unique identifier for the proposal
    pub proposal_id: u64,
    /// Creator of the proposal
    pub creator: Pubkey,
    /// Title of the proposal
    pub title: String,
    /// Description of the proposal
    pub description: String,
    /// Type of proposal (parameter change, feature addition, etc.)
    pub proposal_type: ProposalType,
    /// Voting power required to pass
    pub quorum_required: u64,
    /// Minimum voting power to create proposal
    pub creation_threshold: u64,
    /// Start time for voting
    pub voting_start: i64,
    /// End time for voting
    pub voting_end: i64,
    /// Execution delay after voting ends
    pub execution_delay: i64,
    /// Parameters being changed (if applicable)
    pub parameters: Vec<ProposalParameter>,
    /// Timestamp of creation
    pub timestamp: i64,
}

#[event]
pub struct VoteCast {
    /// Proposal being voted on
    pub proposal_id: u64,
    /// Voter's public key
    pub voter: Pubkey,
    /// Vote choice (for, against, abstain)
    pub vote: VoteChoice,
    /// Voting power used
    pub voting_power: u64,
    /// Reason for vote (optional)
    pub reason: Option<String>,
    /// Delegate who cast the vote (if applicable)
    pub delegate: Option<Pubkey>,
    /// XP level multiplier applied
    pub xp_multiplier: u64,
    /// RP tier multiplier applied
    pub rp_multiplier: u64,
    /// Staking multiplier applied
    pub staking_multiplier: u64,
    /// Activity weight applied
    pub activity_weight: u64,
    /// Total weighted voting power
    pub weighted_voting_power: u64,
    /// Timestamp of vote
    pub timestamp: i64,
}

#[event]
pub struct ProposalExecuted {
    /// Proposal that was executed
    pub proposal_id: u64,
    /// Executor of the proposal
    pub executor: Pubkey,
    /// Execution result
    pub success: bool,
    /// Error message if execution failed
    pub error_message: Option<String>,
    /// Parameters that were changed
    pub changed_parameters: Vec<ProposalParameter>,
    /// Old values of changed parameters
    pub old_values: Vec<String>,
    /// New values of changed parameters
    pub new_values: Vec<String>,
    /// Gas used for execution
    pub gas_used: u64,
    /// Timestamp of execution
    pub timestamp: i64,
}

#[event]
pub struct ProposalCanceled {
    /// Proposal that was canceled
    pub proposal_id: u64,
    /// Who canceled the proposal
    pub canceled_by: Pubkey,
    /// Reason for cancellation
    pub reason: String,
    /// Whether it was canceled by creator or emergency
    pub emergency_cancel: bool,
    /// Voting power at time of cancellation
    pub votes_for: u64,
    pub votes_against: u64,
    pub votes_abstain: u64,
    /// Timestamp of cancellation
    pub timestamp: i64,
}

#[event]
pub struct DelegationCreated {
    /// Delegator (user delegating voting power)
    pub delegator: Pubkey,
    /// Delegate (user receiving voting power)
    pub delegate: Pubkey,
    /// Amount of voting power delegated
    pub delegated_power: u64,
    /// Delegation duration (0 for permanent)
    pub duration: i64,
    /// Expiration timestamp (0 for permanent)
    pub expires_at: i64,
    /// Whether delegation is revocable
    pub revocable: bool,
    /// Specific proposals this applies to (empty for all)
    pub proposal_filter: Vec<u64>,
    /// Timestamp of delegation
    pub timestamp: i64,
}

#[event]
pub struct DelegationRevoked {
    /// Delegator who revoked
    pub delegator: Pubkey,
    /// Delegate who lost voting power
    pub delegate: Pubkey,
    /// Amount of voting power revoked
    pub revoked_power: u64,
    /// Reason for revocation
    pub reason: String,
    /// Whether revocation was voluntary
    pub voluntary: bool,
    /// Timestamp of revocation
    pub timestamp: i64,
}

#[event]
pub struct ParameterChanged {
    /// Parameter that was changed
    pub parameter_name: String,
    /// Category of parameter
    pub category: ParameterCategory,
    /// Old value
    pub old_value: String,
    /// New value
    pub new_value: String,
    /// Proposal that caused the change
    pub proposal_id: u64,
    /// Who executed the change
    pub changed_by: Pubkey,
    /// When the change takes effect
    pub effective_at: i64,
    /// Impact assessment
    pub impact_level: ImpactLevel,
    /// Timestamp of change
    pub timestamp: i64,
}

#[event]
pub struct GovernanceConfigUpdated {
    /// What configuration was updated
    pub config_type: GovernanceConfigType,
    /// Old configuration values
    pub old_config: String,
    /// New configuration values
    pub new_config: String,
    /// Who updated the configuration
    pub updated_by: Pubkey,
    /// Proposal that authorized the update
    pub proposal_id: Option<u64>,
    /// Whether this was an emergency update
    pub emergency_update: bool,
    /// Timestamp of update
    pub timestamp: i64,
}

#[event]
pub struct TreasuryAction {
    /// Type of treasury action
    pub action_type: TreasuryActionType,
    /// Amount involved in the action
    pub amount: u64,
    /// Token involved
    pub token_mint: Pubkey,
    /// Recipient (for disbursements)
    pub recipient: Option<Pubkey>,
    /// Source (for deposits)
    pub source: Option<Pubkey>,
    /// Proposal that authorized this action
    pub proposal_id: u64,
    /// Executor of the action
    pub executor: Pubkey,
    /// Purpose/description of the action
    pub purpose: String,
    /// Timestamp of action
    pub timestamp: i64,
}

#[event]
pub struct EmergencyAction {
    /// Type of emergency action taken
    pub action_type: EmergencyActionType,
    /// Who initiated the emergency action
    pub initiator: Pubkey,
    /// Reason for emergency action
    pub reason: String,
    /// Parameters affected
    pub affected_parameters: Vec<String>,
    /// Duration of emergency action
    pub duration: i64,
    /// Whether community vote is required to lift
    pub requires_vote_to_lift: bool,
    /// Timestamp of action
    pub timestamp: i64,
}

#[event]
pub struct QuorumReached {
    /// Proposal that reached quorum
    pub proposal_id: u64,
    /// Total voting power at quorum
    pub total_voting_power: u64,
    /// Required quorum
    pub required_quorum: u64,
    /// Time taken to reach quorum
    pub time_to_quorum: i64,
    /// Number of unique voters
    pub unique_voters: u32,
    /// Distribution of votes
    pub votes_for: u64,
    pub votes_against: u64,
    pub votes_abstain: u64,
    /// Timestamp when quorum was reached
    pub timestamp: i64,
}

#[event]
pub struct GovernanceReward {
    /// User receiving the reward
    pub user: Pubkey,
    /// Type of governance participation
    pub participation_type: GovernanceParticipationType,
    /// Proposal involved (if applicable)
    pub proposal_id: Option<u64>,
    /// Reward amount in FIN tokens
    pub reward_amount: u64,
    /// Bonus multiplier applied
    pub bonus_multiplier: u64,
    /// Quality score of participation
    pub quality_score: u64,
    /// Consistency bonus applied
    pub consistency_bonus: u64,
    /// Total reward after all bonuses
    pub total_reward: u64,
    /// Timestamp of reward
    pub timestamp: i64,
}

#[event]
pub struct VotingPowerCalculated {
    /// User whose voting power was calculated
    pub user: Pubkey,
    /// Base voting power from staked tokens
    pub base_power: u64,
    /// XP level multiplier
    pub xp_multiplier: u64,
    /// RP tier multiplier
    pub rp_multiplier: u64,
    /// Activity weight multiplier
    pub activity_multiplier: u64,
    /// Delegation power received
    pub delegated_power: u64,
    /// Delegation power given away
    pub delegated_away: u64,
    /// Final calculated voting power
    pub final_voting_power: u64,
    /// Calculation timestamp
    pub timestamp: i64,
}

// Enums for event data

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum ProposalType {
    ParameterChange,
    FeatureAddition,
    TreasuryAllocation,
    SystemUpgrade,
    EmergencyAction,
    CommunityInitiative,
    PartnershipApproval,
    TokenomicsChange,
    GovernanceChange,
    Other(String),
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum VoteChoice {
    For,
    Against,
    Abstain,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub struct ProposalParameter {
    pub name: String,
    pub current_value: String,
    pub proposed_value: String,
    pub parameter_type: String,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum ParameterCategory {
    Mining,
    Staking,
    XP,
    Referral,
    NFT,
    Treasury,
    Governance,
    Security,
    Other(String),
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum ImpactLevel {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum GovernanceConfigType {
    VotingParameters,
    QuorumRules,
    ProposalThresholds,
    ExecutionDelays,
    EmergencyProcedures,
    RewardStructure,
    Other(String),
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum TreasuryActionType {
    Deposit,
    Withdrawal,
    Investment,
    Grant,
    Burn,
    Mint,
    Transfer,
    Reserve,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum EmergencyActionType {
    PauseSystem,
    UnpauseSystem,
    PauseMining,
    UnpauseMining,
    PauseStaking,
    UnpauseStaking,
    PauseTrading,
    UnpauseTrading,
    AdjustParameters,
    SecurityLockdown,
    Other(String),
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum GovernanceParticipationType {
    ProposalCreation,
    Voting,
    Delegation,
    ProposalExecution,
    ParameterMonitoring,
    CommunityModeration,
    SecurityReporting,
    Other(String),
}

// Utility functions for governance events

impl ProposalCreated {
    pub fn new(
        proposal_id: u64,
        creator: Pubkey,
        title: String,
        description: String,
        proposal_type: ProposalType,
        voting_duration: i64,
        execution_delay: i64,
        parameters: Vec<ProposalParameter>,
    ) -> Self {
        let now = Clock::get().unwrap().unix_timestamp;
        
        Self {
            proposal_id,
            creator,
            title,
            description,
            proposal_type,
            quorum_required: Self::calculate_quorum(proposal_id),
            creation_threshold: Self::calculate_creation_threshold(),
            voting_start: now,
            voting_end: now + voting_duration,
            execution_delay,
            parameters,
            timestamp: now,
        }
    }

    fn calculate_quorum(proposal_id: u64) -> u64 {
        // Different quorum requirements based on proposal type and impact
        match proposal_id % 10 {
            0..=3 => 100_000_000, // 100M voting power for critical changes
            4..=6 => 50_000_000,  // 50M for major changes
            7..=8 => 25_000_000,  // 25M for moderate changes
            _ => 10_000_000,      // 10M for minor changes
        }
    }

    fn calculate_creation_threshold() -> u64 {
        1_000_000 // 1M voting power required to create proposals
    }
}

impl VoteCast {
    pub fn new(
        proposal_id: u64,
        voter: Pubkey,
        vote: VoteChoice,
        base_voting_power: u64,
        xp_level: u32,
        rp_tier: u32,
        staked_amount: u64,
        activity_score: u64,
    ) -> Self {
        let xp_multiplier = 1000 + (xp_level as u64 * 10); // 1.0x + level/100
        let rp_multiplier = 1000 + (rp_tier as u64 * 200); // 1.0x + tier*0.2
        let staking_multiplier = 1000 + (staked_amount / 1000); // Based on stake
        let activity_weight = std::cmp::min(2000, 1000 + activity_score); // Max 2.0x

        let weighted_voting_power = base_voting_power
            * xp_multiplier
            * rp_multiplier
            * staking_multiplier
            * activity_weight
            / 1_000_000_000_000; // Normalize

        Self {
            proposal_id,
            voter,
            vote,
            voting_power: base_voting_power,
            reason: None,
            delegate: None,
            xp_multiplier,
            rp_multiplier,
            staking_multiplier,
            activity_weight,
            weighted_voting_power,
            timestamp: Clock::get().unwrap().unix_timestamp,
        }
    }

    pub fn with_delegation(mut self, delegate: Pubkey) -> Self {
        self.delegate = Some(delegate);
        self
    }

    pub fn with_reason(mut self, reason: String) -> Self {
        self.reason = Some(reason);
        self
    }
}

impl GovernanceReward {
    pub fn calculate_reward(
        user: Pubkey,
        participation_type: GovernanceParticipationType,
        base_reward: u64,
        quality_score: u64,
        consistency_multiplier: u64,
    ) -> Self {
        let bonus_multiplier = match participation_type {
            GovernanceParticipationType::ProposalCreation => 5000, // 5x for creating proposals
            GovernanceParticipationType::Voting => 1000,           // 1x base for voting
            GovernanceParticipationType::Delegation => 2000,      // 2x for delegation
            GovernanceParticipationType::ProposalExecution => 3000, // 3x for execution
            GovernanceParticipationType::SecurityReporting => 10000, // 10x for security
            _ => 1500, // 1.5x for other activities
        };

        let quality_bonus = quality_score * 100; // Quality multiplier
        let total_reward = base_reward 
            * bonus_multiplier 
            * quality_bonus 
            * consistency_multiplier 
            / 1_000_000; // Normalize

        Self {
            user,
            participation_type,
            proposal_id: None,
            reward_amount: base_reward,
            bonus_multiplier,
            quality_score,
            consistency_bonus: consistency_multiplier,
            total_reward,
            timestamp: Clock::get().unwrap().unix_timestamp,
        }
    }
}

impl VotingPowerCalculated {
    pub fn calculate(
        user: Pubkey,
        staked_tokens: u64,
        xp_level: u32,
        rp_tier: u32,
        activity_score: u64,
        delegated_received: u64,
        delegated_given: u64,
    ) -> Self {
        // Base power is staked tokens
        let base_power = staked_tokens;

        // XP multiplier: 1.0x + level/100
        let xp_multiplier = 1000 + (xp_level as u64 * 10);

        // RP multiplier: 1.0x + tier*0.2
        let rp_multiplier = 1000 + (rp_tier as u64 * 200);

        // Activity multiplier: max 2.0x based on recent activity
        let activity_multiplier = std::cmp::min(2000, 1000 + activity_score);

        // Calculate final voting power
        let calculated_power = base_power 
            * xp_multiplier 
            * rp_multiplier 
            * activity_multiplier 
            / 1_000_000_000; // Normalize

        let final_voting_power = calculated_power + delegated_received - delegated_given;

        Self {
            user,
            base_power,
            xp_multiplier,
            rp_multiplier,
            activity_multiplier,
            delegated_power: delegated_received,
            delegated_away: delegated_given,
            final_voting_power,
            timestamp: Clock::get().unwrap().unix_timestamp,
        }
    }
}

// Event emission helpers

pub fn emit_proposal_created(event: ProposalCreated) {
    emit!(event);
}

pub fn emit_vote_cast(event: VoteCast) {
    emit!(event);
}

pub fn emit_proposal_executed(event: ProposalExecuted) {
    emit!(event);
}

pub fn emit_proposal_canceled(event: ProposalCanceled) {
    emit!(event);
}

pub fn emit_delegation_created(event: DelegationCreated) {
    emit!(event);
}

pub fn emit_delegation_revoked(event: DelegationRevoked) {
    emit!(event);
}

pub fn emit_parameter_changed(event: ParameterChanged) {
    emit!(event);
}

pub fn emit_governance_config_updated(event: GovernanceConfigUpdated) {
    emit!(event);
}

pub fn emit_treasury_action(event: TreasuryAction) {
    emit!(event);
}

pub fn emit_emergency_action(event: EmergencyAction) {
    emit!(event);
}

pub fn emit_quorum_reached(event: QuorumReached) {
    emit!(event);
}

pub fn emit_governance_reward(event: GovernanceReward) {
    emit!(event);
}

pub fn emit_voting_power_calculated(event: VotingPowerCalculated) {
    emit!(event);
}

// Governance analytics and insights

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct GovernanceMetrics {
    pub total_proposals: u64,
    pub active_proposals: u64,
    pub executed_proposals: u64,
    pub canceled_proposals: u64,
    pub total_votes_cast: u64,
    pub unique_voters: u32,
    pub average_participation_rate: u64,
    pub total_voting_power: u64,
    pub delegated_power: u64,
    pub governance_rewards_distributed: u64,
    pub last_updated: i64,
}

impl GovernanceMetrics {
    pub fn new() -> Self {
        Self {
            total_proposals: 0,
            active_proposals: 0,
            executed_proposals: 0,
            canceled_proposals: 0,
            total_votes_cast: 0,
            unique_voters: 0,
            average_participation_rate: 0,
            total_voting_power: 0,
            delegated_power: 0,
            governance_rewards_distributed: 0,
            last_updated: Clock::get().unwrap().unix_timestamp,
        }
    }

    pub fn update_proposal_metrics(&mut self, proposal_type: &str, status: &str) {
        match status {
            "created" => {
                self.total_proposals += 1;
                self.active_proposals += 1;
            },
            "executed" => {
                self.executed_proposals += 1;
                self.active_proposals = self.active_proposals.saturating_sub(1);
            },
            "canceled" => {
                self.canceled_proposals += 1;
                self.active_proposals = self.active_proposals.saturating_sub(1);
            },
            _ => {}
        }
        self.last_updated = Clock::get().unwrap().unix_timestamp;
    }

    pub fn update_voting_metrics(&mut self, voting_power: u64, is_new_voter: bool) {
        self.total_votes_cast += 1;
        self.total_voting_power += voting_power;
        
        if is_new_voter {
            self.unique_voters += 1;
        }

        // Recalculate participation rate
        if self.total_proposals > 0 {
            self.average_participation_rate = 
                (self.unique_voters as u64 * 10000) / self.total_proposals;
        }

        self.last_updated = Clock::get().unwrap().unix_timestamp;
    }
}

// Governance validation helpers

pub fn validate_proposal_parameters(parameters: &[ProposalParameter]) -> Result<()> {
    for param in parameters {
        // Validate parameter names are recognized
        match param.name.as_str() {
            "mining_rate" | "staking_apy" | "xp_multiplier" | "referral_bonus" 
            | "nft_mint_fee" | "governance_threshold" | "quorum_requirement" => {
                // Valid parameters
            },
            _ => {
                return Err(error!(crate::errors::FinovaError::InvalidParameter));
            }
        }

        // Validate parameter values are reasonable
        if param.proposed_value.is_empty() {
            return Err(error!(crate::errors::FinovaError::InvalidParameterValue));
        }
    }

    Ok(())
}

pub fn calculate_proposal_impact(parameters: &[ProposalParameter]) -> ImpactLevel {
    let mut max_impact = ImpactLevel::Low;

    for param in parameters {
        let impact = match param.name.as_str() {
            "mining_rate" | "staking_apy" => ImpactLevel::High,
            "governance_threshold" | "quorum_requirement" => ImpactLevel::Critical,
            "xp_multiplier" | "referral_bonus" => ImpactLevel::Medium,
            "nft_mint_fee" => ImpactLevel::Low,
            _ => ImpactLevel::Medium,
        };

        max_impact = match (max_impact, impact) {
            (ImpactLevel::Critical, _) | (_, ImpactLevel::Critical) => ImpactLevel::Critical,
            (ImpactLevel::High, _) | (_, ImpactLevel::High) => ImpactLevel::High,
            (ImpactLevel::Medium, _) | (_, ImpactLevel::Medium) => ImpactLevel::Medium,
            _ => ImpactLevel::Low,
        };
    }

    max_impact
}

// Tests
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_voting_power_calculation() {
        let user = Pubkey::new_unique();
        let voting_power = VotingPowerCalculated::calculate(
            user,
            1000000, // 1M staked
            50,      // Level 50
            3,       // RP Tier 3
            800,     // High activity
            500000,  // 500K delegated to them
            100000,  // 100K delegated away
        );

        assert!(voting_power.final_voting_power > 0);
        assert_eq!(voting_power.user, user);
    }

    #[test]
    fn test_governance_reward_calculation() {
        let user = Pubkey::new_unique();
        let reward = GovernanceReward::calculate_reward(
            user,
            GovernanceParticipationType::ProposalCreation,
            1000,  // Base reward
            90,    // High quality
            150,   // Good consistency
        );

        assert!(reward.total_reward > reward.reward_amount);
        assert_eq!(reward.participation_type, GovernanceParticipationType::ProposalCreation);
    }

    #[test]
    fn test_proposal_parameter_validation() {
        let valid_params = vec![
            ProposalParameter {
                name: "mining_rate".to_string(),
                current_value: "0.05".to_string(),
                proposed_value: "0.04".to_string(),
                parameter_type: "float".to_string(),
            }
        ];

        assert!(validate_proposal_parameters(&valid_params).is_ok());

        let invalid_params = vec![
            ProposalParameter {
                name: "invalid_param".to_string(),
                current_value: "0.05".to_string(),
                proposed_value: "0.04".to_string(),
                parameter_type: "float".to_string(),
            }
        ];

        assert!(validate_proposal_parameters(&invalid_params).is_err());
    }
}
