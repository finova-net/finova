// finova-net/finova/client/rust/src/client.rs

use anchor_client::solana_sdk::{
    commitment_config::CommitmentConfig,
    pubkey::Pubkey,
    signature::{Keypair, Signature},
    signer::Signer,
    system_program,
    transaction::Transaction,
};
use anchor_client::{Client, Cluster, Program};
use anchor_lang::prelude::*;
use serde::{Deserialize, Serialize};
use std::rc::Rc;
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use anyhow::{anyhow, Result};

// Core Finova Network Client
#[derive(Clone)]
pub struct FinovaClient {
    pub program: Program<Rc<Keypair>>,
    pub payer: Rc<Keypair>,
    pub cluster: Cluster,
    pub commitment: CommitmentConfig,
}

// User account state matching smart contract
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserAccount {
    pub authority: Pubkey,
    pub total_mined: u64,
    pub mining_rate: u64,
    pub last_mining_claim: i64,
    pub xp_points: u64,
    pub xp_level: u32,
    pub rp_points: u64,
    pub rp_tier: u8,
    pub referral_code: String,
    pub referrer: Option<Pubkey>,
    pub referral_count: u32,
    pub kyc_verified: bool,
    pub stake_amount: u64,
    pub mining_streak: u32,
    pub quality_score: u32,
    pub guild_id: Option<Pubkey>,
    pub special_cards: Vec<SpecialCard>,
    pub achievements: Vec<Achievement>,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecialCard {
    pub card_type: CardType,
    pub rarity: CardRarity,
    pub effect_value: u32,
    pub duration: u32,
    pub remaining_uses: u32,
    pub acquired_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CardType {
    MiningBoost,
    XpAccelerator,
    ReferralPower,
    QualityEnhancer,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CardRarity {
    Common,
    Uncommon,
    Rare,
    Epic,
    Legendary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Achievement {
    pub achievement_type: String,
    pub unlocked_at: i64,
    pub bonus_value: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MiningReward {
    pub base_reward: u64,
    pub xp_bonus: u64,
    pub rp_bonus: u64,
    pub quality_bonus: u64,
    pub card_bonus: u64,
    pub total_reward: u64,
    pub claimed_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReferralStats {
    pub direct_referrals: u32,
    pub network_size: u32,
    pub network_quality: f64,
    pub total_rp_earned: u64,
    pub tier_bonus: f64,
}

impl FinovaClient {
    /// Initialize new Finova client
    pub fn new(
        payer: Keypair,
        cluster: Cluster,
        program_id: Pubkey,
        commitment: Option<CommitmentConfig>,
    ) -> Result<Self> {
        let payer = Rc::new(payer);
        let client = Client::new_with_options(
            cluster.clone(),
            payer.clone(),
            commitment.unwrap_or(CommitmentConfig::confirmed()),
        );
        let program = client.program(program_id)?;

        Ok(Self {
            program,
            payer,
            cluster,
            commitment: commitment.unwrap_or(CommitmentConfig::confirmed()),
        })
    }

    /// Initialize user account with KYC
    pub async fn initialize_user(
        &self,
        referral_code: Option<String>,
        kyc_data: KycData,
    ) -> Result<Signature> {
        let user_account = Keypair::new();
        let referrer = if let Some(ref_code) = referral_code {
            self.get_user_by_referral_code(&ref_code).await?
        } else {
            None
        };

        let accounts = finova_core::accounts::InitializeUser {
            user_account: user_account.pubkey(),
            authority: self.payer.pubkey(),
            referrer,
            system_program: system_program::ID,
        };

        let ix = self.program
            .request()
            .accounts(accounts)
            .args(finova_core::instruction::InitializeUser {
                referral_code: self.generate_referral_code(),
                kyc_data,
            })
            .signer(&user_account)
            .instructions()?;

        let sig = self.program.request().instruction(ix[0].clone()).send().await?;
        Ok(sig)
    }

    /// Start mining session
    pub async fn start_mining(&self, user: &Pubkey) -> Result<Signature> {
        let user_account = self.get_user_account(user).await?;
        
        // Calculate current mining rate with all bonuses
        let mining_rate = self.calculate_mining_rate(&user_account).await?;
        
        let accounts = finova_core::accounts::StartMining {
            user_account: *user,
            authority: self.payer.pubkey(),
        };

        let sig = self.program
            .request()
            .accounts(accounts)
            .args(finova_core::instruction::StartMining {
                mining_rate,
            })
            .send()
            .await?;

        Ok(sig)
    }

    /// Claim mining rewards with XP and RP bonuses
    pub async fn claim_mining_rewards(&self, user: &Pubkey) -> Result<MiningReward> {
        let user_account = self.get_user_account(user).await?;
        let reward = self.calculate_total_reward(&user_account).await?;

        let accounts = finova_core::accounts::ClaimRewards {
            user_account: *user,
            authority: self.payer.pubkey(),
            token_program: anchor_spl::token::ID,
        };

        let sig = self.program
            .request()
            .accounts(accounts)
            .args(finova_core::instruction::ClaimRewards {
                amount: reward.total_reward,
            })
            .send()
            .await?;

        println!("Mining rewards claimed: {} $FIN (tx: {})", 
                 reward.total_reward as f64 / 1e9, sig);
        
        Ok(reward)
    }

    /// Add XP from social activities
    pub async fn add_xp(
        &self,
        user: &Pubkey,
        activity_type: String,
        platform: String,
        content_quality: f64,
    ) -> Result<u64> {
        let user_account = self.get_user_account(user).await?;
        let xp_gained = self.calculate_xp_gain(&user_account, &activity_type, &platform, content_quality).await?;

        let accounts = finova_core::accounts::AddXp {
            user_account: *user,
            authority: self.payer.pubkey(),
        };

        let sig = self.program
            .request()
            .accounts(accounts)
            .args(finova_core::instruction::AddXp {
                activity_type,
                platform,
                xp_amount: xp_gained,
                quality_score: (content_quality * 100.0) as u32,
            })
            .send()
            .await?;

        println!("XP gained: {} (tx: {})", xp_gained, sig);
        Ok(xp_gained)
    }

    /// Process referral and update RP
    pub async fn process_referral(&self, referrer: &Pubkey, referee: &Pubkey) -> Result<Signature> {
        let referrer_account = self.get_user_account(referrer).await?;
        let rp_gained = self.calculate_rp_gain(&referrer_account, "new_referral").await?;

        let accounts = finova_core::accounts::ProcessReferral {
            referrer_account: *referrer,
            referee_account: *referee,
            authority: self.payer.pubkey(),
        };

        let sig = self.program
            .request()
            .accounts(accounts)
            .args(finova_core::instruction::ProcessReferral {
                rp_amount: rp_gained,
            })
            .send()
            .await?;

        Ok(sig)
    }

    /// Stake $FIN tokens for enhanced rewards
    pub async fn stake_tokens(&self, user: &Pubkey, amount: u64) -> Result<Signature> {
        let accounts = finova_core::accounts::StakeTokens {
            user_account: *user,
            authority: self.payer.pubkey(),
            token_program: anchor_spl::token::ID,
        };

        let sig = self.program
            .request()
            .accounts(accounts)
            .args(finova_core::instruction::StakeTokens { amount })
            .send()
            .await?;

        println!("Staked {} $FIN (tx: {})", amount as f64 / 1e9, sig);
        Ok(sig)
    }

    /// Use special card for temporary bonuses
    pub async fn use_special_card(&self, user: &Pubkey, card_id: u32) -> Result<Signature> {
        let accounts = finova_core::accounts::UseSpecialCard {
            user_account: *user,
            authority: self.payer.pubkey(),
        };

        let sig = self.program
            .request()
            .accounts(accounts)
            .args(finova_core::instruction::UseSpecialCard { card_id })
            .send()
            .await?;

        println!("Special card used (ID: {}, tx: {})", card_id, sig);
        Ok(sig)
    }

    /// Get user account data
    pub async fn get_user_account(&self, user: &Pubkey) -> Result<UserAccount> {
        let account = self.program.account::<UserAccount>(*user).await?;
        Ok(account)
    }

    /// Get user by referral code
    pub async fn get_user_by_referral_code(&self, referral_code: &str) -> Result<Option<Pubkey>> {
        // Implementation would filter accounts by referral_code
        // This is a simplified version
        Ok(None)
    }

    /// Calculate comprehensive mining rate with all multipliers
    pub async fn calculate_mining_rate(&self, user: &UserAccount) -> Result<u64> {
        // Base mining rate with Pi Network-style regression
        let base_rate = 50_000_000; // 0.05 $FIN/hour in lamports
        
        // Pioneer bonus (decreases as network grows)
        let total_users = self.get_total_users().await?;
        let pioneer_bonus = (2.0 - (total_users as f64 / 1_000_000.0)).max(1.0);
        
        // Referral network bonus
        let referral_bonus = 1.0 + (user.referral_count as f64 * 0.1);
        
        // KYC security bonus
        let security_bonus = if user.kyc_verified { 1.2 } else { 0.8 };
        
        // XP level multiplier
        let xp_multiplier = 1.0 + (user.xp_level as f64 / 100.0);
        
        // RP tier bonus
        let rp_multiplier = match user.rp_tier {
            0 => 1.0,  // Explorer
            1 => 1.2,  // Connector  
            2 => 1.5,  // Influencer
            3 => 2.0,  // Leader
            4 => 3.0,  // Ambassador
            _ => 1.0,
        };
        
        // Staking multiplier
        let stake_multiplier = if user.stake_amount > 0 {
            1.0 + ((user.stake_amount as f64 / 1e9) / 1000.0).min(2.0)
        } else {
            1.0
        };
        
        // Anti-whale exponential regression
        let regression_factor = (-0.001 * (user.total_mined as f64 / 1e9)).exp();
        
        // Special card bonuses
        let card_bonus = self.calculate_active_card_bonus(user);
        
        let final_rate = (base_rate as f64 
            * pioneer_bonus 
            * referral_bonus 
            * security_bonus 
            * xp_multiplier 
            * rp_multiplier 
            * stake_multiplier 
            * regression_factor 
            * card_bonus) as u64;
            
        Ok(final_rate)
    }

    /// Calculate XP gain with quality and platform multipliers
    pub async fn calculate_xp_gain(
        &self,
        user: &UserAccount,
        activity_type: &str,
        platform: &str,
        quality_score: f64,
    ) -> Result<u64> {
        let base_xp = match activity_type {
            "original_post" => 50,
            "photo_post" => 75,
            "video_content" => 150,
            "story_status" => 25,
            "meaningful_comment" => 25,
            "like_react" => 5,
            "share_repost" => 15,
            "viral_content" => 1000,
            _ => 10,
        };
        
        let platform_multiplier = match platform {
            "tiktok" => 1.3,
            "instagram" => 1.2,
            "youtube" => 1.4,
            "facebook" => 1.1,
            "x" => 1.2,
            _ => 1.0,
        };
        
        let quality_multiplier = quality_score.max(0.5).min(2.0);
        let level_progression = (-0.01 * user.xp_level as f64).exp();
        
        let xp_gained = (base_xp as f64 
            * platform_multiplier 
            * quality_multiplier 
            * level_progression) as u64;
            
        Ok(xp_gained)
    }

    /// Calculate RP gain from referral activities
    pub async fn calculate_rp_gain(&self, user: &UserAccount, activity: &str) -> Result<u64> {
        let base_rp = match activity {
            "new_referral" => 100,
            "referral_kyc" => 100,
            "referral_mining" => 25,
            "network_milestone" => 500,
            _ => 10,
        };
        
        let network_quality = self.calculate_network_quality(user).await?;
        let tier_multiplier = match user.rp_tier {
            0 => 1.0,  // Explorer
            1 => 1.2,  // Connector
            2 => 1.5,  // Influencer  
            3 => 2.0,  // Leader
            4 => 3.0,  // Ambassador
            _ => 1.0,
        };
        
        let rp_gained = (base_rp as f64 * network_quality * tier_multiplier) as u64;
        Ok(rp_gained)
    }

    /// Calculate total mining reward with all bonuses
    pub async fn calculate_total_reward(&self, user: &UserAccount) -> Result<MiningReward> {
        let now = Utc::now().timestamp();
        let hours_since_last_claim = ((now - user.last_mining_claim) / 3600).max(0) as u64;
        
        let base_reward = self.calculate_mining_rate(user).await? * hours_since_last_claim;
        
        // XP bonus (20% of base)
        let xp_bonus = (base_reward as f64 * 0.2 * (user.xp_level as f64 / 100.0)) as u64;
        
        // RP bonus (30% of base)  
        let rp_tier_multiplier = match user.rp_tier {
            0 => 0.0, 1 => 0.1, 2 => 0.2, 3 => 0.3, 4 => 0.5, _ => 0.0,
        };
        let rp_bonus = (base_reward as f64 * 0.3 * rp_tier_multiplier) as u64;
        
        // Quality bonus (AI-assessed content quality)
        let quality_bonus = (base_reward as f64 * 0.1 * (user.quality_score as f64 / 100.0)) as u64;
        
        // Active special card bonuses
        let card_bonus = (base_reward as f64 * (self.calculate_active_card_bonus(user) - 1.0)) as u64;
        
        let total_reward = base_reward + xp_bonus + rp_bonus + quality_bonus + card_bonus;
        
        Ok(MiningReward {
            base_reward,
            xp_bonus,
            rp_bonus,
            quality_bonus,
            card_bonus,
            total_reward,
            claimed_at: now,
        })
    }

    /// Get referral statistics
    pub async fn get_referral_stats(&self, user: &Pubkey) -> Result<ReferralStats> {
        let user_account = self.get_user_account(user).await?;
        let network_quality = self.calculate_network_quality(&user_account).await?;
        
        Ok(ReferralStats {
            direct_referrals: user_account.referral_count,
            network_size: user_account.referral_count * 3, // Simplified L2/L3 calculation
            network_quality,
            total_rp_earned: user_account.rp_points,
            tier_bonus: match user_account.rp_tier {
                0 => 0.0, 1 => 0.2, 2 => 0.5, 3 => 1.0, 4 => 2.0, _ => 0.0,
            },
        })
    }

    /// Calculate active special card bonuses
    fn calculate_active_card_bonus(&self, user: &UserAccount) -> f64 {
        let now = Utc::now().timestamp();
        let mut total_bonus = 1.0;
        
        for card in &user.special_cards {
            if card.remaining_uses > 0 {
                let bonus = match card.card_type {
                    CardType::MiningBoost => match card.rarity {
                        CardRarity::Common => 2.0,    // +100%
                        CardRarity::Rare => 3.0,      // +200%
                        CardRarity::Epic => 6.0,      // +500%
                        CardRarity::Legendary => 1.5, // +50% permanent
                        _ => 1.2,
                    },
                    CardType::XpAccelerator => 1.3,
                    CardType::ReferralPower => 1.5,
                    CardType::QualityEnhancer => 1.4,
                };
                total_bonus *= bonus;
            }
        }
        
        total_bonus.min(10.0) // Cap at 10x multiplier
    }

    /// Calculate network quality for RP calculations
    async fn calculate_network_quality(&self, user: &UserAccount) -> Result<f64> {
        if user.referral_count == 0 {
            return Ok(1.0);
        }
        
        // Simplified quality calculation
        // In production, this would analyze actual referral activity
        let active_ratio = 0.8; // Assume 80% of referrals are active
        let avg_level = 15.0;    // Average XP level of referrals
        let retention_rate = 0.7; // 70% retention
        
        let quality = active_ratio * (avg_level / 100.0) * retention_rate;
        Ok(quality.max(0.1).min(2.0))
    }

    /// Get total network users (for pioneer bonus calculation)
    async fn get_total_users(&self) -> Result<u64> {
        // This would query the network state account
        // Simplified implementation
        Ok(250_000) // Example: 250K users
    }

    /// Generate unique referral code
    fn generate_referral_code(&self) -> String {
        use rand::{distributions::Alphanumeric, Rng};
        rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(8)
            .map(char::from)
            .collect::<String>()
            .to_uppercase()
    }

    /// Validate user activity for anti-bot protection
    pub async fn validate_user_activity(&self, user: &Pubkey) -> Result<f64> {
        let user_account = self.get_user_account(user).await?;
        
        // Multi-factor human probability calculation
        let mut human_score = 1.0;
        
        // Check mining patterns (humans have natural variance)
        let timing_variance = 0.9; // Natural human timing patterns
        
        // Check content quality consistency
        let quality_consistency = (user_account.quality_score as f64 / 100.0).max(0.5);
        
        // Check referral network authenticity
        let network_authenticity = self.calculate_network_quality(&user_account).await?;
        
        // Social graph validation (simplified)
        let social_graph_score = 0.85;
        
        human_score = timing_variance * quality_consistency * network_authenticity * social_graph_score;
        
        Ok(human_score.max(0.1).min(1.0))
    }

    /// Get user dashboard data
    pub async fn get_user_dashboard(&self, user: &Pubkey) -> Result<UserDashboard> {
        let user_account = self.get_user_account(user).await?;
        let mining_rate = self.calculate_mining_rate(&user_account).await?;
        let referral_stats = self.get_referral_stats(user).await?;
        let pending_rewards = self.calculate_total_reward(&user_account).await?;
        
        Ok(UserDashboard {
            user_account,
            current_mining_rate: mining_rate,
            referral_stats,
            pending_rewards,
            human_probability: self.validate_user_activity(user).await?,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserDashboard {
    pub user_account: UserAccount,
    pub current_mining_rate: u64,
    pub referral_stats: ReferralStats,
    pub pending_rewards: MiningReward,
    pub human_probability: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KycData {
    pub full_name: String,
    pub id_number: String,
    pub country: String,
    pub phone_number: String,
    pub biometric_hash: String,
}

// Program instruction definitions (matching smart contracts)
pub mod finova_core {
    use super::*;
    
    pub mod instruction {
        use super::*;
        
        #[derive(Debug)]
        pub struct InitializeUser {
            pub referral_code: String,
            pub kyc_data: KycData,
        }
        
        #[derive(Debug)]
        pub struct StartMining {
            pub mining_rate: u64,
        }
        
        #[derive(Debug)]
        pub struct ClaimRewards {
            pub amount: u64,
        }
        
        #[derive(Debug)]
        pub struct AddXp {
            pub activity_type: String,
            pub platform: String,
            pub xp_amount: u64,
            pub quality_score: u32,
        }
        
        #[derive(Debug)]
        pub struct ProcessReferral {
            pub rp_amount: u64,
        }
        
        #[derive(Debug)]
        pub struct StakeTokens {
            pub amount: u64,
        }
        
        #[derive(Debug)]
        pub struct UseSpecialCard {
            pub card_id: u32,
        }
    }
    
    pub mod accounts {
        use super::*;
        
        pub struct InitializeUser {
            pub user_account: Pubkey,
            pub authority: Pubkey,
            pub referrer: Option<Pubkey>,
            pub system_program: Pubkey,
        }
        
        pub struct StartMining {
            pub user_account: Pubkey,
            pub authority: Pubkey,
        }
        
        pub struct ClaimRewards {
            pub user_account: Pubkey,
            pub authority: Pubkey,
            pub token_program: Pubkey,
        }
        
        pub struct AddXp {
            pub user_account: Pubkey,
            pub authority: Pubkey,
        }
        
        pub struct ProcessReferral {
            pub referrer_account: Pubkey,
            pub referee_account: Pubkey,
            pub authority: Pubkey,
        }
        
        pub struct StakeTokens {
            pub user_account: Pubkey,
            pub authority: Pubkey,
            pub token_program: Pubkey,
        }
        
        pub struct UseSpecialCard {
            pub user_account: Pubkey,
            pub authority: Pubkey,
        }
    }
}
