// finova-net/finova/client/rust/src/instructions.rs

use anchor_client::solana_sdk::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    signature::Keypair,
    system_program,
};
use anchor_lang::{prelude::*, InstructionData, ToAccountMetas};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FinovaMiningConfig {
    pub base_rate: f64,
    pub finizen_bonus: f64,
    pub max_daily_mining: f64,
    pub regression_factor: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XPConfig {
    pub base_xp: u64,
    pub platform_multiplier: f64,
    pub quality_multiplier: f64,
    pub streak_bonus: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RPConfig {
    pub direct_bonus: f64,
    pub network_multiplier: f64,
    pub quality_threshold: f64,
}

/// Main Finova Client Instructions Handler
pub struct FinovaInstructions {
    pub program_id: Pubkey,
    pub token_program_id: Pubkey,
    pub nft_program_id: Pubkey,
    pub defi_program_id: Pubkey,
    pub oracle_program_id: Pubkey,
}

impl FinovaInstructions {
    pub fn new(
        program_id: Pubkey,
        token_program_id: Pubkey,
        nft_program_id: Pubkey,
        defi_program_id: Pubkey,
        oracle_program_id: Pubkey,
    ) -> Self {
        Self {
            program_id,
            token_program_id,
            nft_program_id,
            defi_program_id,
            oracle_program_id,
        }
    }

    /// Initialize user account with integrated XP, RP, and Mining systems
    pub fn initialize_user(
        &self,
        user: &Pubkey,
        user_account: &Pubkey,
        referrer: Option<Pubkey>,
    ) -> Result<Instruction, Box<dyn std::error::Error>> {
        let accounts = vec![
            AccountMeta::new(*user, true),
            AccountMeta::new(*user_account, false),
            AccountMeta::new_readonly(system_program::id(), false),
        ];

        let mut accounts_with_referrer = accounts;
        if let Some(ref_key) = referrer {
            accounts_with_referrer.push(AccountMeta::new(ref_key, false));
        }

        let data = InitializeUserArgs {
            referrer_code: referrer.map(|r| r.to_string()),
        };

        Ok(Instruction {
            program_id: self.program_id,
            accounts: accounts_with_referrer,
            data: data.try_to_vec()?,
        })
    }

    /// Start mining with integrated reward calculation
    pub fn start_mining(
        &self,
        user: &Pubkey,
        user_account: &Pubkey,
        mining_account: &Pubkey,
        staking_account: Option<Pubkey>,
    ) -> Result<Instruction, Box<dyn std::error::Error>> {
        let mut accounts = vec![
            AccountMeta::new(*user, true),
            AccountMeta::new(*user_account, false),
            AccountMeta::new(*mining_account, false),
            AccountMeta::new_readonly(self.oracle_program_id, false),
        ];

        if let Some(stake_acc) = staking_account {
            accounts.push(AccountMeta::new_readonly(stake_acc, false));
        }

        let data = StartMiningArgs {
            expected_rate: None, // Auto-calculate based on XP/RP
        };

        Ok(Instruction {
            program_id: self.program_id,
            accounts,
            data: data.try_to_vec()?,
        })
    }

    /// Claim mining rewards with XP and RP bonuses
    pub fn claim_mining_rewards(
        &self,
        user: &Pubkey,
        user_account: &Pubkey,
        mining_account: &Pubkey,
        token_account: &Pubkey,
    ) -> Result<Instruction, Box<dyn std::error::Error>> {
        let accounts = vec![
            AccountMeta::new(*user, true),
            AccountMeta::new(*user_account, false),
            AccountMeta::new(*mining_account, false),
            AccountMeta::new(*token_account, false),
            AccountMeta::new_readonly(self.token_program_id, false),
        ];

        let data = ClaimMiningRewardsArgs {};

        Ok(Instruction {
            program_id: self.program_id,
            accounts,
            data: data.try_to_vec()?,
        })
    }

    /// Add XP from social media activity
    pub fn add_xp_activity(
        &self,
        user: &Pubkey,
        user_account: &Pubkey,
        activity_type: ActivityType,
        platform: SocialPlatform,
        content_hash: String,
        quality_score: f64,
    ) -> Result<Instruction, Box<dyn std::error::Error>> {
        let accounts = vec![
            AccountMeta::new(*user, true),
            AccountMeta::new(*user_account, false),
            AccountMeta::new_readonly(self.oracle_program_id, false), // For quality verification
        ];

        let data = AddXPActivityArgs {
            activity_type,
            platform,
            content_hash,
            quality_score,
            timestamp: chrono::Utc::now().timestamp(),
        };

        Ok(Instruction {
            program_id: self.program_id,
            accounts,
            data: data.try_to_vec()?,
        })
    }

    /// Process referral reward distribution
    pub fn process_referral_reward(
        &self,
        referrer: &Pubkey,
        referrer_account: &Pubkey,
        referee: &Pubkey,
        referee_account: &Pubkey,
        reward_type: ReferralRewardType,
        amount: u64,
    ) -> Result<Instruction, Box<dyn std::error::Error>> {
        let accounts = vec![
            AccountMeta::new(*referrer, false),
            AccountMeta::new(*referrer_account, false),
            AccountMeta::new_readonly(*referee, false),
            AccountMeta::new_readonly(*referee_account, false),
        ];

        let data = ProcessReferralRewardArgs {
            reward_type,
            amount,
            timestamp: chrono::Utc::now().timestamp(),
        };

        Ok(Instruction {
            program_id: self.program_id,
            accounts,
            data: data.try_to_vec()?,
        })
    }

    /// Stake tokens for enhanced rewards
    pub fn stake_tokens(
        &self,
        user: &Pubkey,
        user_token_account: &Pubkey,
        staking_account: &Pubkey,
        amount: u64,
        lock_period: u32,
    ) -> Result<Instruction, Box<dyn std::error::Error>> {
        let accounts = vec![
            AccountMeta::new(*user, true),
            AccountMeta::new(*user_token_account, false),
            AccountMeta::new(*staking_account, false),
            AccountMeta::new_readonly(self.token_program_id, false),
        ];

        let data = StakeTokensArgs {
            amount,
            lock_period,
        };

        Ok(Instruction {
            program_id: self.token_program_id,
            accounts,
            data: data.try_to_vec()?,
        })
    }

    /// Unstake tokens with calculated rewards
    pub fn unstake_tokens(
        &self,
        user: &Pubkey,
        user_account: &Pubkey,
        staking_account: &Pubkey,
        token_account: &Pubkey,
        amount: u64,
    ) -> Result<Instruction, Box<dyn std::error::Error>> {
        let accounts = vec![
            AccountMeta::new(*user, true),
            AccountMeta::new(*user_account, false),
            AccountMeta::new(*staking_account, false),
            AccountMeta::new(*token_account, false),
            AccountMeta::new_readonly(self.token_program_id, false),
        ];

        let data = UnstakeTokensArgs { amount };

        Ok(Instruction {
            program_id: self.token_program_id,
            accounts,
            data: data.try_to_vec()?,
        })
    }

    /// Use special NFT card for bonuses
    pub fn use_special_card(
        &self,
        user: &Pubkey,
        user_account: &Pubkey,
        nft_account: &Pubkey,
        card_type: SpecialCardType,
    ) -> Result<Instruction, Box<dyn std::error::Error>> {
        let accounts = vec![
            AccountMeta::new(*user, true),
            AccountMeta::new(*user_account, false),
            AccountMeta::new(*nft_account, false),
            AccountMeta::new_readonly(self.nft_program_id, false),
        ];

        let data = UseSpecialCardArgs { card_type };

        Ok(Instruction {
            program_id: self.nft_program_id,
            accounts,
            data: data.try_to_vec()?,
        })
    }

    /// Create/join guild for community features
    pub fn join_guild(
        &self,
        user: &Pubkey,
        user_account: &Pubkey,
        guild_account: &Pubkey,
        guild_id: String,
    ) -> Result<Instruction, Box<dyn std::error::Error>> {
        let accounts = vec![
            AccountMeta::new(*user, true),
            AccountMeta::new(*user_account, false),
            AccountMeta::new(*guild_account, false),
        ];

        let data = JoinGuildArgs { guild_id };

        Ok(Instruction {
            program_id: self.program_id,
            accounts,
            data: data.try_to_vec()?,
        })
    }

    /// Verify KYC for security bonus
    pub fn verify_kyc(
        &self,
        user: &Pubkey,
        user_account: &Pubkey,
        verification_data: KYCData,
    ) -> Result<Instruction, Box<dyn std::error::Error>> {
        let accounts = vec![
            AccountMeta::new(*user, true),
            AccountMeta::new(*user_account, false),
        ];

        let data = VerifyKYCArgs { verification_data };

        Ok(Instruction {
            program_id: self.program_id,
            accounts,
            data: data.try_to_vec()?,
        })
    }

    /// Submit governance proposal
    pub fn submit_proposal(
        &self,
        proposer: &Pubkey,
        proposer_account: &Pubkey,
        proposal_account: &Pubkey,
        proposal_type: ProposalType,
        description: String,
    ) -> Result<Instruction, Box<dyn std::error::Error>> {
        let accounts = vec![
            AccountMeta::new(*proposer, true),
            AccountMeta::new_readonly(*proposer_account, false),
            AccountMeta::new(*proposal_account, false),
            AccountMeta::new_readonly(system_program::id(), false),
        ];

        let data = SubmitProposalArgs {
            proposal_type,
            description,
            voting_period: 7 * 24 * 3600, // 7 days
        };

        Ok(Instruction {
            program_id: self.program_id,
            accounts,
            data: data.try_to_vec()?,
        })
    }

    /// Vote on governance proposal
    pub fn vote_proposal(
        &self,
        voter: &Pubkey,
        voter_account: &Pubkey,
        proposal_account: &Pubkey,
        vote: Vote,
    ) -> Result<Instruction, Box<dyn std::error::Error>> {
        let accounts = vec![
            AccountMeta::new(*voter, true),
            AccountMeta::new_readonly(*voter_account, false),
            AccountMeta::new(*proposal_account, false),
        ];

        let data = VoteProposalArgs { vote };

        Ok(Instruction {
            program_id: self.program_id,
            accounts,
            data: data.try_to_vec()?,
        })
    }

    /// Anti-bot verification challenge
    pub fn submit_anti_bot_challenge(
        &self,
        user: &Pubkey,
        user_account: &Pubkey,
        challenge_response: ChallengeResponse,
    ) -> Result<Instruction, Box<dyn std::error::Error>> {
        let accounts = vec![
            AccountMeta::new(*user, true),
            AccountMeta::new(*user_account, false),
        ];

        let data = SubmitAntiBotChallengeArgs { challenge_response };

        Ok(Instruction {
            program_id: self.program_id,
            accounts,
            data: data.try_to_vec()?,
        })
    }

    /// Bridge tokens to other chains
    pub fn bridge_tokens(
        &self,
        user: &Pubkey,
        user_token_account: &Pubkey,
        bridge_account: &Pubkey,
        target_chain: u8,
        target_address: String,
        amount: u64,
    ) -> Result<Instruction, Box<dyn std::error::Error>> {
        let accounts = vec![
            AccountMeta::new(*user, true),
            AccountMeta::new(*user_token_account, false),
            AccountMeta::new(*bridge_account, false),
            AccountMeta::new_readonly(self.token_program_id, false),
        ];

        let data = BridgeTokensArgs {
            target_chain,
            target_address,
            amount,
        };

        Ok(Instruction {
            program_id: self.program_id,
            accounts,
            data: data.try_to_vec()?,
        })
    }

    /// Batch instruction creator for complex operations
    pub fn create_batch_instruction(
        &self,
        instructions: Vec<Instruction>,
    ) -> Result<Instruction, Box<dyn std::error::Error>> {
        let accounts = vec![AccountMeta::new_readonly(self.program_id, false)];

        let data = BatchInstructionArgs {
            instructions: instructions
                .into_iter()
                .map(|ix| SerializableInstruction::from(ix))
                .collect(),
        };

        Ok(Instruction {
            program_id: self.program_id,
            accounts,
            data: data.try_to_vec()?,
        })
    }
}

// Instruction argument structures
#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct InitializeUserArgs {
    pub referrer_code: Option<String>,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct StartMiningArgs {
    pub expected_rate: Option<f64>,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct ClaimMiningRewardsArgs {}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct AddXPActivityArgs {
    pub activity_type: ActivityType,
    pub platform: SocialPlatform,
    pub content_hash: String,
    pub quality_score: f64,
    pub timestamp: i64,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct ProcessReferralRewardArgs {
    pub reward_type: ReferralRewardType,
    pub amount: u64,
    pub timestamp: i64,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct StakeTokensArgs {
    pub amount: u64,
    pub lock_period: u32,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct UnstakeTokensArgs {
    pub amount: u64,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct UseSpecialCardArgs {
    pub card_type: SpecialCardType,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct JoinGuildArgs {
    pub guild_id: String,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct VerifyKYCArgs {
    pub verification_data: KYCData,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct SubmitProposalArgs {
    pub proposal_type: ProposalType,
    pub description: String,
    pub voting_period: i64,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct VoteProposalArgs {
    pub vote: Vote,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct SubmitAntiBotChallengeArgs {
    pub challenge_response: ChallengeResponse,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct BridgeTokensArgs {
    pub target_chain: u8,
    pub target_address: String,
    pub amount: u64,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct BatchInstructionArgs {
    pub instructions: Vec<SerializableInstruction>,
}

// Enums and types
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub enum ActivityType {
    Post,
    Comment,
    Like,
    Share,
    Follow,
    VideoUpload,
    StoryPost,
    LiveStream,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub enum SocialPlatform {
    Instagram,
    TikTok,
    YouTube,
    Facebook,
    Twitter,
    LinkedIn,
    Discord,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub enum ReferralRewardType {
    Registration,
    KYCCompletion,
    FirstMining,
    DailyMining,
    XPBonus,
    Milestone,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub enum SpecialCardType {
    DoubleMining,
    TripleMining,
    MiningFrenzy,
    EternalMiner,
    XPDouble,
    StreakSaver,
    LevelRush,
    XPMagnet,
    ReferralBoost,
    NetworkAmplifier,
    AmbassadorPass,
    NetworkKing,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub enum ProposalType {
    ParameterChange,
    FeatureAddition,
    TreasuryAllocation,
    CommunityInitiative,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub enum Vote {
    Yes,
    No,
    Abstain,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct KYCData {
    pub document_type: String,
    pub document_hash: String,
    pub biometric_hash: String,
    pub verification_level: u8,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct ChallengeResponse {
    pub challenge_id: String,
    pub response_data: Vec<u8>,
    pub timestamp: i64,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct SerializableInstruction {
    pub program_id: Pubkey,
    pub accounts: Vec<AccountMeta>,
    pub data: Vec<u8>,
}

impl From<Instruction> for SerializableInstruction {
    fn from(ix: Instruction) -> Self {
        Self {
            program_id: ix.program_id,
            accounts: ix.accounts,
            data: ix.data,
        }
    }
}

// Utility functions for instruction building
pub mod utils {
    use super::*;

    /// Calculate expected mining rate based on user state
    pub fn calculate_mining_rate(
        base_rate: f64,
        user_level: u32,
        rp_tier: u32,
        total_holdings: u64,
        is_kyc_verified: bool,
        active_referrals: u32,
    ) -> f64 {
        let finizen_bonus = 2.0 - (50000.0 / 1000000.0); // Assuming current user count
        let referral_bonus = 1.0 + (active_referrals as f64 * 0.1);
        let security_bonus = if is_kyc_verified { 1.2 } else { 0.8 };
        let regression_factor = (-0.001 * total_holdings as f64).exp();
        let xp_multiplier = 1.0 + (user_level as f64 / 100.0);
        let rp_multiplier = 1.0 + (rp_tier as f64 * 0.2);

        base_rate * finizen_bonus * referral_bonus * security_bonus 
            * regression_factor * xp_multiplier * rp_multiplier
    }

    /// Calculate XP gain for activity
    pub fn calculate_xp_gain(
        activity_type: &ActivityType,
        platform: &SocialPlatform,
        quality_score: f64,
        streak_days: u32,
        user_level: u32,
    ) -> u64 {
        let base_xp = match activity_type {
            ActivityType::Post => 50,
            ActivityType::VideoUpload => 150,
            ActivityType::Comment => 25,
            ActivityType::Like => 5,
            ActivityType::Share => 15,
            ActivityType::Follow => 20,
            ActivityType::StoryPost => 25,
            ActivityType::LiveStream => 200,
        };

        let platform_multiplier = match platform {
            SocialPlatform::TikTok => 1.3,
            SocialPlatform::YouTube => 1.4,
            SocialPlatform::Instagram => 1.2,
            SocialPlatform::Twitter => 1.2,
            SocialPlatform::Facebook => 1.1,
            _ => 1.0,
        };

        let streak_bonus = 1.0 + (streak_days.min(30) as f64 * 0.05);
        let level_progression = (-0.01 * user_level as f64).exp();

        (base_xp as f64 * platform_multiplier * quality_score * streak_bonus * level_progression) as u64
    }

    /// Validate instruction arguments
    pub fn validate_instruction_args<T>(args: &T) -> Result<(), Box<dyn std::error::Error>>
    where
        T: std::fmt::Debug,
    {
        // Add validation logic here
        Ok(())
    }

    /// Create program derived address
    pub fn create_pda(seeds: &[&[u8]], program_id: &Pubkey) -> (Pubkey, u8) {
        Pubkey::find_program_address(seeds, program_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mining_rate_calculation() {
        let rate = utils::calculate_mining_rate(
            0.1, // base_rate
            25,  // user_level
            2,   // rp_tier
            1000, // total_holdings
            true, // is_kyc_verified
            5,    // active_referrals
        );
        assert!(rate > 0.0);
    }

    #[test]
    fn test_xp_calculation() {
        let xp = utils::calculate_xp_gain(
            &ActivityType::Post,
            &SocialPlatform::TikTok,
            1.5, // quality_score
            7,   // streak_days
            15,  // user_level
        );
        assert!(xp > 50);
    }

    #[test]
    fn test_instruction_creation() {
        let program_id = Pubkey::new_unique();
        let token_program_id = Pubkey::new_unique();
        let nft_program_id = Pubkey::new_unique();
        let defi_program_id = Pubkey::new_unique();
        let oracle_program_id = Pubkey::new_unique();

        let instructions = FinovaInstructions::new(
            program_id,
            token_program_id,
            nft_program_id,
            defi_program_id,
            oracle_program_id,
        );

        let user = Pubkey::new_unique();
        let user_account = Pubkey::new_unique();

        let ix = instructions.initialize_user(&user, &user_account, None);
        assert!(ix.is_ok());
    }
}
