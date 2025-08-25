// programs/finova-token/src/events/mod.rs

//! # Finova Token Events Module
//! 
//! This module defines all events emitted by the Finova token program.
//! Events provide transparency and enable off-chain monitoring of token operations.

use anchor_lang::prelude::*;

/// Re-export all event modules for easy access
pub mod mint;
pub mod burn;
pub mod stake;
pub mod reward;
pub mod transfer;
pub mod governance;

pub use mint::*;
pub use burn::*;
pub use stake::*;
pub use reward::*;
pub use transfer::*;
pub use governance::*;

/// Event emitted when tokens are minted
#[event]
pub struct TokenMinted {
    /// The mint authority that authorized this mint
    pub mint_authority: Pubkey,
    /// The recipient of the minted tokens
    pub recipient: Pubkey,
    /// Amount of tokens minted
    pub amount: u64,
    /// The mint account public key
    pub mint: Pubkey,
    /// Reason for minting (mining reward, staking reward, etc.)
    pub mint_reason: MintReason,
    /// Timestamp when the mint occurred
    pub timestamp: i64,
    /// Block slot when the mint occurred
    pub slot: u64,
}

/// Event emitted when tokens are burned
#[event]
pub struct TokenBurned {
    /// The authority that authorized this burn
    pub burn_authority: Pubkey,
    /// The token account from which tokens were burned
    pub token_account: Pubkey,
    /// Amount of tokens burned
    pub amount: u64,
    /// The mint account public key
    pub mint: Pubkey,
    /// Reason for burning (fee payment, deflationary mechanism, etc.)
    pub burn_reason: BurnReason,
    /// Timestamp when the burn occurred
    pub timestamp: i64,
    /// Block slot when the burn occurred
    pub slot: u64,
}

/// Event emitted when tokens are staked
#[event]
pub struct TokensStaked {
    /// The user who staked the tokens
    pub staker: Pubkey,
    /// The stake account that holds the staked tokens
    pub stake_account: Pubkey,
    /// Amount of tokens staked
    pub amount: u64,
    /// The staking tier achieved
    pub staking_tier: StakingTier,
    /// Duration of the stake in seconds
    pub stake_duration: u64,
    /// Expected APY for this stake
    pub expected_apy: u16, // Basis points (e.g., 1000 = 10%)
    /// Timestamp when the stake was created
    pub timestamp: i64,
    /// Block slot when the stake was created
    pub slot: u64,
}

/// Event emitted when tokens are unstaked
#[event]
pub struct TokensUnstaked {
    /// The user who unstaked the tokens
    pub staker: Pubkey,
    /// The stake account that was closed
    pub stake_account: Pubkey,
    /// Amount of original tokens unstaked
    pub principal_amount: u64,
    /// Amount of reward tokens earned
    pub reward_amount: u64,
    /// Total amount returned to user
    pub total_amount: u64,
    /// Time the tokens were staked (in seconds)
    pub stake_duration: u64,
    /// Actual APY achieved
    pub achieved_apy: u16, // Basis points
    /// Early unstake penalty applied (if any)
    pub penalty_amount: u64,
    /// Timestamp when the unstake occurred
    pub timestamp: i64,
    /// Block slot when the unstake occurred
    pub slot: u64,
}

/// Event emitted when staking rewards are claimed
#[event]
pub struct StakingRewardsClaimed {
    /// The user who claimed the rewards
    pub claimant: Pubkey,
    /// The stake account from which rewards were claimed
    pub stake_account: Pubkey,
    /// Amount of rewards claimed
    pub reward_amount: u64,
    /// Type of rewards claimed
    pub reward_type: RewardType,
    /// Accumulated rewards before this claim
    pub accumulated_rewards: u64,
    /// Remaining unclaimed rewards
    pub remaining_rewards: u64,
    /// Timestamp when rewards were claimed
    pub timestamp: i64,
    /// Block slot when rewards were claimed
    pub slot: u64,
}

/// Event emitted when reward pool is updated
#[event]
pub struct RewardPoolUpdated {
    /// The authority that updated the pool
    pub authority: Pubkey,
    /// The reward pool account
    pub reward_pool: Pubkey,
    /// Previous total rewards in pool
    pub previous_amount: u64,
    /// New total rewards in pool
    pub new_amount: u64,
    /// Amount added or removed (signed)
    pub delta: i64,
    /// Reason for the update
    pub update_reason: PoolUpdateReason,
    /// Timestamp when the pool was updated
    pub timestamp: i64,
    /// Block slot when the pool was updated
    pub slot: u64,
}

/// Event emitted when transfer occurs
#[event]
pub struct TokenTransferred {
    /// The sender of the tokens
    pub from: Pubkey,
    /// The recipient of the tokens
    pub to: Pubkey,
    /// Amount of tokens transferred
    pub amount: u64,
    /// The mint account public key
    pub mint: Pubkey,
    /// Transfer fee applied (if any)
    pub fee_amount: u64,
    /// Type of transfer
    pub transfer_type: TransferType,
    /// Timestamp when the transfer occurred
    pub timestamp: i64,
    /// Block slot when the transfer occurred
    pub slot: u64,
}

/// Event emitted when mint configuration is updated
#[event]
pub struct MintConfigUpdated {
    /// The authority that updated the configuration
    pub authority: Pubkey,
    /// The mint account that was updated
    pub mint: Pubkey,
    /// Previous configuration hash
    pub previous_config_hash: [u8; 32],
    /// New configuration hash
    pub new_config_hash: [u8; 32],
    /// What was updated
    pub update_type: ConfigUpdateType,
    /// Timestamp when the configuration was updated
    pub timestamp: i64,
    /// Block slot when the configuration was updated
    pub slot: u64,
}

/// Event emitted when freeze/thaw operations occur
#[event]
pub struct AccountFreezeThaw {
    /// The authority that performed the operation
    pub authority: Pubkey,
    /// The token account that was frozen/thawed
    pub token_account: Pubkey,
    /// The mint account
    pub mint: Pubkey,
    /// Whether the account was frozen (true) or thawed (false)
    pub is_frozen: bool,
    /// Reason for the freeze/thaw
    pub reason: FreezeReason,
    /// Timestamp when the operation occurred
    pub timestamp: i64,
    /// Block slot when the operation occurred
    pub slot: u64,
}

/// Event emitted when governance proposal affects tokens
#[event]
pub struct GovernanceTokenEvent {
    /// The proposal ID that triggered this event
    pub proposal_id: u64,
    /// The governance authority
    pub governance_authority: Pubkey,
    /// Type of governance action
    pub action_type: GovernanceActionType,
    /// Affected mint account (if applicable)
    pub mint: Option<Pubkey>,
    /// Affected amount (if applicable)
    pub amount: Option<u64>,
    /// Parameters changed (serialized)
    pub parameters: Vec<u8>,
    /// Timestamp when the governance action occurred
    pub timestamp: i64,
    /// Block slot when the governance action occurred
    pub slot: u64,
}

/// Event emitted when token supply metrics change significantly
#[event]
pub struct SupplyMetricsUpdate {
    /// The mint account
    pub mint: Pubkey,
    /// Current total supply
    pub total_supply: u64,
    /// Amount currently staked
    pub staked_supply: u64,
    /// Amount in circulation (not staked)
    pub circulating_supply: u64,
    /// Amount burned to date
    pub total_burned: u64,
    /// Amount minted to date
    pub total_minted: u64,
    /// Supply change that triggered this event
    pub supply_delta: i64,
    /// Timestamp of the update
    pub timestamp: i64,
    /// Block slot of the update
    pub slot: u64,
}

/// Enum representing different reasons for minting tokens
#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub enum MintReason {
    /// Tokens minted as mining rewards
    MiningReward,
    /// Tokens minted as staking rewards
    StakingReward,
    /// Tokens minted as referral bonuses
    ReferralBonus,
    /// Tokens minted for governance purposes
    Governance,
    /// Tokens minted for liquidity provision
    Liquidity,
    /// Tokens minted for team/advisor allocation
    TeamAllocation,
    /// Tokens minted for development funding
    Development,
    /// Tokens minted for marketing activities
    Marketing,
    /// Tokens minted for ecosystem growth
    Ecosystem,
    /// Other reasons
    Other,
}

/// Enum representing different reasons for burning tokens
#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub enum BurnReason {
    /// Tokens burned as transaction fees
    TransactionFee,
    /// Tokens burned for deflationary purposes
    Deflationary,
    /// Tokens burned as penalty for violations
    Penalty,
    /// Tokens burned for NFT purchases
    NftPurchase,
    /// Tokens burned for special card usage
    SpecialCardUse,
    /// Tokens burned for governance proposals
    GovernanceProposal,
    /// Tokens burned for early unstaking penalty
    EarlyUnstakePenalty,
    /// Tokens burned for bot detection violations
    BotPenalty,
    /// Tokens burned for quality violations
    QualityPenalty,
    /// Other reasons
    Other,
}

/// Enum representing different staking tiers
#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub enum StakingTier {
    /// Basic tier: 100-499 FIN
    Basic,
    /// Premium tier: 500-999 FIN
    Premium,
    /// VIP tier: 1,000-4,999 FIN
    Vip,
    /// Elite tier: 5,000-9,999 FIN
    Elite,
    /// Legendary tier: 10,000+ FIN
    Legendary,
}

/// Enum representing different types of rewards
#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub enum RewardType {
    /// Base staking rewards
    BaseStaking,
    /// XP level bonus rewards
    XpBonus,
    /// Referral network bonus rewards
    ReferralBonus,
    /// Activity-based bonus rewards
    ActivityBonus,
    /// Loyalty bonus for long-term staking
    LoyaltyBonus,
    /// Special event rewards
    SpecialEvent,
    /// Guild participation rewards
    GuildReward,
    /// Governance participation rewards
    GovernanceReward,
}

/// Enum representing reasons for reward pool updates
#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub enum PoolUpdateReason {
    /// Regular funding from revenue
    RegularFunding,
    /// Emergency funding injection
    EmergencyFunding,
    /// Reward distribution to users
    RewardDistribution,
    /// Pool rebalancing between different reward types
    Rebalancing,
    /// Governance-mandated adjustment
    GovernanceAdjustment,
    /// Migration between pool versions
    Migration,
    /// Fee collection deposit
    FeeCollection,
    /// Partnership reward deposit
    PartnershipReward,
}

/// Enum representing different types of transfers
#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub enum TransferType {
    /// Regular user-to-user transfer
    UserTransfer,
    /// Transfer from/to staking contract
    StakingTransfer,
    /// Transfer for mining reward distribution
    MiningReward,
    /// Transfer for referral reward distribution
    ReferralReward,
    /// Transfer for NFT marketplace transaction
    NftTransaction,
    /// Transfer for DEX trading
    DexTrade,
    /// Transfer for cross-chain bridge
    BridgeTransfer,
    /// Transfer for governance operations
    GovernanceTransfer,
    /// Transfer for fee payment
    FeePayment,
}

/// Enum representing different types of configuration updates
#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub enum ConfigUpdateType {
    /// Mint authority changed
    MintAuthority,
    /// Freeze authority changed
    FreezeAuthority,
    /// Mint rate parameters changed
    MintRateParameters,
    /// Staking parameters changed
    StakingParameters,
    /// Fee structure changed
    FeeStructure,
    /// Supply caps changed
    SupplyCaps,
    /// Reward pool configuration changed
    RewardPoolConfig,
    /// Emergency controls updated
    EmergencyControls,
}

/// Enum representing reasons for account freeze/thaw
#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub enum FreezeReason {
    /// Account frozen due to suspicious activity
    SuspiciousActivity,
    /// Account frozen due to KYC/AML compliance
    Compliance,
    /// Account frozen due to bot detection
    BotDetection,
    /// Account frozen due to governance decision
    GovernanceDecision,
    /// Account frozen due to security breach
    SecurityBreach,
    /// Account frozen due to user request
    UserRequest,
    /// Account frozen during system maintenance
    Maintenance,
    /// Account frozen due to legal requirements
    LegalHold,
}

/// Enum representing different types of governance actions
#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub enum GovernanceActionType {
    /// Change to token mint parameters
    MintParameterChange,
    /// Change to staking parameters
    StakingParameterChange,
    /// Change to reward distribution
    RewardDistributionChange,
    /// Emergency pause/unpause
    EmergencyToggle,
    /// Supply cap adjustment
    SupplyCapAdjustment,
    /// Fee structure change
    FeeStructureChange,
    /// Authority transfer
    AuthorityTransfer,
    /// Protocol upgrade
    ProtocolUpgrade,
}

/// Helper function to emit token minted event
pub fn emit_token_minted(
    mint_authority: Pubkey,
    recipient: Pubkey,
    amount: u64,
    mint: Pubkey,
    mint_reason: MintReason,
) {
    emit!(TokenMinted {
        mint_authority,
        recipient,
        amount,
        mint,
        mint_reason,
        timestamp: Clock::get().unwrap().unix_timestamp,
        slot: Clock::get().unwrap().slot,
    });
}

/// Helper function to emit token burned event
pub fn emit_token_burned(
    burn_authority: Pubkey,
    token_account: Pubkey,
    amount: u64,
    mint: Pubkey,
    burn_reason: BurnReason,
) {
    emit!(TokenBurned {
        burn_authority,
        token_account,
        amount,
        mint,
        burn_reason,
        timestamp: Clock::get().unwrap().unix_timestamp,
        slot: Clock::get().unwrap().slot,
    });
}

/// Helper function to emit tokens staked event
pub fn emit_tokens_staked(
    staker: Pubkey,
    stake_account: Pubkey,
    amount: u64,
    staking_tier: StakingTier,
    stake_duration: u64,
    expected_apy: u16,
) {
    emit!(TokensStaked {
        staker,
        stake_account,
        amount,
        staking_tier,
        stake_duration,
        expected_apy,
        timestamp: Clock::get().unwrap().unix_timestamp,
        slot: Clock::get().unwrap().slot,
    });
}

/// Helper function to emit tokens unstaked event
pub fn emit_tokens_unstaked(
    staker: Pubkey,
    stake_account: Pubkey,
    principal_amount: u64,
    reward_amount: u64,
    total_amount: u64,
    stake_duration: u64,
    achieved_apy: u16,
    penalty_amount: u64,
) {
    emit!(TokensUnstaked {
        staker,
        stake_account,
        principal_amount,
        reward_amount,
        total_amount,
        stake_duration,
        achieved_apy,
        penalty_amount,
        timestamp: Clock::get().unwrap().unix_timestamp,
        slot: Clock::get().unwrap().slot,
    });
}

/// Helper function to emit staking rewards claimed event
pub fn emit_staking_rewards_claimed(
    claimant: Pubkey,
    stake_account: Pubkey,
    reward_amount: u64,
    reward_type: RewardType,
    accumulated_rewards: u64,
    remaining_rewards: u64,
) {
    emit!(StakingRewardsClaimed {
        claimant,
        stake_account,
        reward_amount,
        reward_type,
        accumulated_rewards,
        remaining_rewards,
        timestamp: Clock::get().unwrap().unix_timestamp,
        slot: Clock::get().unwrap().slot,
    });
}

/// Helper function to emit reward pool updated event
pub fn emit_reward_pool_updated(
    authority: Pubkey,
    reward_pool: Pubkey,
    previous_amount: u64,
    new_amount: u64,
    delta: i64,
    update_reason: PoolUpdateReason,
) {
    emit!(RewardPoolUpdated {
        authority,
        reward_pool,
        previous_amount,
        new_amount,
        delta,
        update_reason,
        timestamp: Clock::get().unwrap().unix_timestamp,
        slot: Clock::get().unwrap().slot,
    });
}

/// Helper function to emit token transferred event
pub fn emit_token_transferred(
    from: Pubkey,
    to: Pubkey,
    amount: u64,
    mint: Pubkey,
    fee_amount: u64,
    transfer_type: TransferType,
) {
    emit!(TokenTransferred {
        from,
        to,
        amount,
        mint,
        fee_amount,
        transfer_type,
        timestamp: Clock::get().unwrap().unix_timestamp,
        slot: Clock::get().unwrap().slot,
    });
}

/// Helper function to emit supply metrics update event
pub fn emit_supply_metrics_update(
    mint: Pubkey,
    total_supply: u64,
    staked_supply: u64,
    circulating_supply: u64,
    total_burned: u64,
    total_minted: u64,
    supply_delta: i64,
) {
    emit!(SupplyMetricsUpdate {
        mint,
        total_supply,
        staked_supply,
        circulating_supply,
        total_burned,
        total_minted,
        supply_delta,
        timestamp: Clock::get().unwrap().unix_timestamp,
        slot: Clock::get().unwrap().slot,
    });
}

/// Helper function to get staking tier from amount
pub fn get_staking_tier(amount: u64) -> StakingTier {
    match amount {
        100..=499 => StakingTier::Basic,
        500..=999 => StakingTier::Premium,
        1000..=4999 => StakingTier::Vip,
        5000..=9999 => StakingTier::Elite,
        _ if amount >= 10000 => StakingTier::Legendary,
        _ => StakingTier::Basic, // Default for amounts < 100
    }
}

/// Helper function to get expected APY for staking tier
pub fn get_expected_apy(tier: &StakingTier) -> u16 {
    match tier {
        StakingTier::Basic => 800,     // 8%
        StakingTier::Premium => 1000,  // 10%
        StakingTier::Vip => 1200,      // 12%
        StakingTier::Elite => 1400,    // 14%
        StakingTier::Legendary => 1500, // 15%
    }
}

/// Helper function to calculate early unstake penalty
pub fn calculate_early_unstake_penalty(
    amount: u64,
    stake_duration: u64,
    minimum_duration: u64,
) -> u64 {
    if stake_duration >= minimum_duration {
        return 0;
    }

    // Penalty decreases linearly from 20% to 0% as we approach minimum duration
    let penalty_rate = (minimum_duration - stake_duration) * 2000 / minimum_duration; // Max 20% (2000 basis points)
    amount * penalty_rate / 10000
}

/// Helper function to validate mint reason
pub fn is_valid_mint_reason(reason: &MintReason, authority: &Pubkey, expected_authority: &Pubkey) -> bool {
    match reason {
        MintReason::MiningReward | MintReason::StakingReward | MintReason::ReferralBonus => {
            // These can only be minted by the core program
            authority == expected_authority
        }
        MintReason::Governance => {
            // Only governance can mint for governance purposes
            // This would need additional validation against governance accounts
            true
        }
        _ => {
            // Other reasons require proper authority validation
            authority == expected_authority
        }
    }
}

/// Helper function to validate burn reason
pub fn is_valid_burn_reason(reason: &BurnReason, context: &str) -> bool {
    match reason {
        BurnReason::TransactionFee => context == "fee_payment",
        BurnReason::Deflationary => context == "deflationary_burn",
        BurnReason::Penalty => context.starts_with("penalty"),
        BurnReason::NftPurchase => context == "nft_purchase",
        BurnReason::SpecialCardUse => context == "card_usage",
        _ => true, // Other reasons are generally valid
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_staking_tier_calculation() {
        assert_eq!(get_staking_tier(50), StakingTier::Basic);
        assert_eq!(get_staking_tier(250), StakingTier::Basic);
        assert_eq!(get_staking_tier(750), StakingTier::Premium);
        assert_eq!(get_staking_tier(2500), StakingTier::Vip);
        assert_eq!(get_staking_tier(7500), StakingTier::Elite);
        assert_eq!(get_staking_tier(15000), StakingTier::Legendary);
    }

    #[test]
    fn test_expected_apy_calculation() {
        assert_eq!(get_expected_apy(&StakingTier::Basic), 800);
        assert_eq!(get_expected_apy(&StakingTier::Premium), 1000);
        assert_eq!(get_expected_apy(&StakingTier::Vip), 1200);
        assert_eq!(get_expected_apy(&StakingTier::Elite), 1400);
        assert_eq!(get_expected_apy(&StakingTier::Legendary), 1500);
    }

    #[test]
    fn test_early_unstake_penalty() {
        // No penalty if staked for minimum duration
        assert_eq!(calculate_early_unstake_penalty(1000, 30, 30), 0);
        
        // Full penalty (20%) if unstaked immediately
        assert_eq!(calculate_early_unstake_penalty(1000, 0, 30), 200);
        
        // Half penalty (10%) if unstaked at half minimum duration
        assert_eq!(calculate_early_unstake_penalty(1000, 15, 30), 100);
    }
}
