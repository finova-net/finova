// programs/finova-core/src/instructions/rewards.rs

use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};
use crate::constants::*;
use crate::errors::FinovaError;
use crate::state::*;
use crate::utils::*;
use std::collections::HashMap;

/// Calculate and distribute integrated rewards (XP + RP + $FIN mining)
#[derive(Accounts)]
pub struct CalculateRewards<'info> {
    #[account(mut)]
    pub user: Account<'info, User>,
    
    #[account(
        mut,
        seeds = [b"mining", user.key().as_ref()],
        bump = mining_account.bump,
    )]
    pub mining_account: Account<'info, MiningAccount>,
    
    #[account(
        mut,
        seeds = [b"xp", user.key().as_ref()],
        bump = xp_account.bump,
    )]
    pub xp_account: Account<'info, XPAccount>,
    
    #[account(
        mut,
        seeds = [b"referral", user.key().as_ref()],
        bump = referral_account.bump,
    )]
    pub referral_account: Account<'info, ReferralNetwork>,
    
    #[account(
        mut,
        seeds = [b"rewards", user.key().as_ref()],
        bump = rewards_account.bump,
    )]
    pub rewards_account: Account<'info, RewardsAccount>,
    
    #[account(mut)]
    pub global_state: Account<'info, GlobalState>,
    
    #[account(mut)]
    pub reward_pool: Account<'info, TokenAccount>,
    
    #[account(mut)]
    pub user_token_account: Account<'info, TokenAccount>,
    
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub clock: Sysvar<'info, Clock>,
}

/// Process activity-based rewards with quality assessment
#[derive(Accounts)]
pub struct ProcessActivityReward<'info> {
    #[account(mut)]
    pub user: Account<'info, User>,
    
    #[account(
        mut,
        seeds = [b"xp", user.key().as_ref()],
        bump = xp_account.bump,
    )]
    pub xp_account: Account<'info, XPAccount>,
    
    #[account(
        mut,
        seeds = [b"referral", user.key().as_ref()],
        bump = referral_account.bump,
    )]
    pub referral_account: Account<'info, ReferralNetwork>,
    
    #[account(
        mut,
        seeds = [b"rewards", user.key().as_ref()],
        bump = rewards_account.bump,
    )]
    pub rewards_account: Account<'info, RewardsAccount>,
    
    #[account(mut)]
    pub global_state: Account<'info, GlobalState>,
    
    pub system_program: Program<'info, System>,
    pub clock: Sysvar<'info, Clock>,
}

/// Distribute referral network rewards
#[derive(Accounts)]
pub struct DistributeReferralRewards<'info> {
    #[account(mut)]
    pub referrer: Account<'info, User>,
    
    #[account(mut)]
    pub referee: Account<'info, User>,
    
    #[account(
        mut,
        seeds = [b"referral", referrer.key().as_ref()],
        bump = referrer_network.bump,
    )]
    pub referrer_network: Account<'info, ReferralNetwork>,
    
    #[account(
        mut,
        seeds = [b"referral", referee.key().as_ref()],
        bump = referee_network.bump,
    )]
    pub referee_network: Account<'info, ReferralNetwork>,
    
    #[account(
        mut,
        seeds = [b"rewards", referrer.key().as_ref()],
        bump = referrer_rewards.bump,
    )]
    pub referrer_rewards: Account<'info, RewardsAccount>,
    
    #[account(
        mut,
        seeds = [b"rewards", referee.key().as_ref()],
        bump = referee_rewards.bump,
    )]
    pub referee_rewards: Account<'info, RewardsAccount>,
    
    #[account(mut)]
    pub global_state: Account<'info, GlobalState>,
    
    #[account(mut)]
    pub reward_pool: Account<'info, TokenAccount>,
    
    #[account(mut)]
    pub referrer_token_account: Account<'info, TokenAccount>,
    
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub clock: Sysvar<'info, Clock>,
}

/// Claim accumulated rewards
#[derive(Accounts)]
pub struct ClaimRewards<'info> {
    #[account(mut)]
    pub user: Account<'info, User>,
    
    #[account(
        mut,
        seeds = [b"rewards", user.key().as_ref()],
        bump = rewards_account.bump,
    )]
    pub rewards_account: Account<'info, RewardsAccount>,
    
    #[account(mut)]
    pub reward_pool: Account<'info, TokenAccount>,
    
    #[account(mut)]
    pub user_token_account: Account<'info, TokenAccount>,
    
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub clock: Sysvar<'info, Clock>,
}

/// Apply special card boosts
#[derive(Accounts)]
pub struct ApplySpecialCardBoost<'info> {
    #[account(mut)]
    pub user: Account<'info, User>,
    
    #[account(
        mut,
        seeds = [b"rewards", user.key().as_ref()],
        bump = rewards_account.bump,
    )]
    pub rewards_account: Account<'info, RewardsAccount>,
    
    pub special_card_mint: Account<'info, anchor_spl::token::Mint>,
    
    #[account(mut)]
    pub user_card_account: Account<'info, TokenAccount>,
    
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub clock: Sysvar<'info, Clock>,
}

impl<'info> CalculateRewards<'info> {
    /// Master formula for integrated reward calculation
    /// Final_Reward = Base_Mining_Rate × XP_Multiplier × RP_Multiplier × Quality_Score × Network_Regression
    pub fn calculate_integrated_rewards(&mut self) -> Result<u64> {
        let clock = &self.clock;
        let current_time = clock.unix_timestamp;
        
        // Validate user is eligible for rewards
        require!(
            self.user.is_verified,
            FinovaError::UserNotVerified
        );
        
        require!(
            current_time >= self.user.last_reward_calculation + MIN_REWARD_INTERVAL,
            FinovaError::RewardCalculationTooSoon
        );
        
        // Calculate base mining rate with phase progression
        let base_mining_rate = self.calculate_base_mining_rate()?;
        
        // Calculate XP multiplier (1.0x - 5.0x)
        let xp_multiplier = self.calculate_xp_multiplier()?;
        
        // Calculate RP multiplier (1.0x - 3.0x)
        let rp_multiplier = self.calculate_rp_multiplier()?;
        
        // Calculate quality score (0.5x - 2.0x)
        let quality_score = self.calculate_quality_score()?;
        
        // Calculate network regression factor
        let network_regression = self.calculate_network_regression()?;
        
        // Apply integrated formula
        let final_reward = (base_mining_rate as f64 
            * xp_multiplier 
            * rp_multiplier 
            * quality_score 
            * network_regression) as u64;
        
        // Update reward account
        self.rewards_account.total_earned += final_reward;
        self.rewards_account.pending_rewards += final_reward;
        self.rewards_account.last_calculation = current_time;
        self.rewards_account.calculation_count += 1;
        
        // Update mining account
        self.mining_account.total_mined += final_reward;
        self.mining_account.last_mining_time = current_time;
        
        // Update user stats
        self.user.last_reward_calculation = current_time;
        self.user.total_rewards_earned += final_reward;
        
        // Update global statistics
        self.global_state.total_rewards_distributed += final_reward;
        self.global_state.total_active_miners += 1;
        
        emit!(RewardCalculated {
            user: self.user.key(),
            base_rate: base_mining_rate,
            xp_multiplier,
            rp_multiplier,
            quality_score,
            network_regression,
            final_reward,
            timestamp: current_time,
        });
        
        Ok(final_reward)
    }
    
    /// Calculate base mining rate with exponential regression
    fn calculate_base_mining_rate(&self) -> Result<u64> {
        let total_users = self.global_state.total_users;
        let current_phase = self.determine_mining_phase(total_users);
        
        let base_rate = match current_phase {
            MiningPhase::Finizen => BASE_MINING_RATE_PHASE_1,
            MiningPhase::Growth => BASE_MINING_RATE_PHASE_2,
            MiningPhase::Maturity => BASE_MINING_RATE_PHASE_3,
            MiningPhase::Stability => BASE_MINING_RATE_PHASE_4,
        };
        
        // Apply Finizen bonus
        let finizen_bonus = self.calculate_finizen_bonus(total_users)?;
        
        // Apply security bonus
        let security_bonus = if self.user.is_kyc_verified {
            SECURITY_BONUS_KYC
        } else {
            SECURITY_BONUS_NON_KYC
        };
        
        let final_rate = (base_rate as f64 * finizen_bonus * security_bonus) as u64;
        
        Ok(final_rate.min(MAX_HOURLY_MINING_RATE))
    }
    
    /// Calculate XP-based multiplier
    fn calculate_xp_multiplier(&self) -> Result<f64> {
        let xp_level = self.xp_account.current_level;
        let base_multiplier = 1.0;
        let level_bonus = (xp_level as f64 / 20.0).min(4.0); // Max 5.0x at level 100
        
        // Apply level progression decay
        let level_progression = (-0.01 * xp_level as f64).exp();
        
        let multiplier = (base_multiplier + level_bonus) * level_progression;
        
        Ok(multiplier.max(XP_MULTIPLIER_MIN).min(XP_MULTIPLIER_MAX))
    }
    
    /// Calculate RP-based multiplier
    fn calculate_rp_multiplier(&self) -> Result<f64> {
        let rp_tier = self.referral_account.tier;
        let active_referrals = self.referral_account.active_referrals;
        
        let base_multiplier = 1.0;
        let tier_bonus = match rp_tier {
            ReferralTier::Explorer => 0.0,
            ReferralTier::Connector => 0.2,
            ReferralTier::Influencer => 0.5,
            ReferralTier::Leader => 1.0,
            ReferralTier::Ambassador => 2.0,
        };
        
        // Add active referral bonus (max 0.5x additional)
        let referral_bonus = (active_referrals as f64 * 0.02).min(0.5);
        
        let multiplier = base_multiplier + tier_bonus + referral_bonus;
        
        Ok(multiplier.max(RP_MULTIPLIER_MIN).min(RP_MULTIPLIER_MAX))
    }
    
    /// Calculate AI-powered quality score
    fn calculate_quality_score(&self) -> Result<f64> {
        let recent_activities = &self.rewards_account.recent_activities;
        
        if recent_activities.is_empty() {
            return Ok(QUALITY_SCORE_DEFAULT);
        }
        
        let mut total_quality = 0.0;
        let mut activity_count = 0;
        
        for activity in recent_activities.iter().take(10) { // Last 10 activities
            let quality = self.assess_activity_quality(activity)?;
            total_quality += quality;
            activity_count += 1;
        }
        
        let average_quality = if activity_count > 0 {
            total_quality / activity_count as f64
        } else {
            QUALITY_SCORE_DEFAULT
        };
        
        Ok(average_quality.max(QUALITY_SCORE_MIN).min(QUALITY_SCORE_MAX))
    }
    
    /// Calculate network regression factor to prevent whale dominance
    fn calculate_network_regression(&self) -> Result<f64> {
        let total_holdings = self.user.total_holdings;
        let regression_factor = (-REGRESSION_COEFFICIENT * total_holdings as f64).exp();
        
        Ok(regression_factor.max(NETWORK_REGRESSION_MIN))
    }
    
    /// Determine current mining phase based on user count
    fn determine_mining_phase(&self, total_users: u64) -> MiningPhase {
        match total_users {
            0..=PHASE_1_USER_LIMIT => MiningPhase::Finizen,
            PHASE_1_USER_LIMIT..=PHASE_2_USER_LIMIT => MiningPhase::Growth,
            PHASE_2_USER_LIMIT..=PHASE_3_USER_LIMIT => MiningPhase::Maturity,
            _ => MiningPhase::Stability,
        }
    }
    
    /// Calculate Finizen bonus (early adopter bonus)
    fn calculate_finizen_bonus(&self, total_users: u64) -> Result<f64> {
        let bonus = FINIZEN_BONUS_MAX - (total_users as f64 / FINIZEN_BONUS_DENOMINATOR);
        Ok(bonus.max(FINIZEN_BONUS_MIN))
    }
    
    /// Assess individual activity quality using AI-like scoring
    fn assess_activity_quality(&self, activity: &ActivityRecord) -> Result<f64> {
        let mut quality_score = QUALITY_SCORE_DEFAULT;
        
        // Platform multiplier
        quality_score *= match activity.platform {
            Platform::TikTok => 1.3,
            Platform::Instagram => 1.2,
            Platform::YouTube => 1.4,
            Platform::Facebook => 1.1,
            Platform::TwitterX => 1.2,
            Platform::App => 1.0,
        };
        
        // Activity type scoring
        quality_score *= match activity.activity_type {
            ActivityType::OriginalPost => 1.5,
            ActivityType::QualityComment => 1.2,
            ActivityType::VideoContent => 2.0,
            ActivityType::ViralContent => 2.5,
            ActivityType::DailyLogin => 0.8,
            ActivityType::ShareRepost => 1.0,
        };
        
        // Engagement multiplier
        if activity.engagement_metrics.views > 1000 {
            quality_score *= 1.8;
        } else if activity.engagement_metrics.views > 100 {
            quality_score *= 1.3;
        }
        
        // Originality boost
        if activity.is_original {
            quality_score *= 1.5;
        }
        
        Ok(quality_score.max(QUALITY_SCORE_MIN).min(QUALITY_SCORE_MAX))
    }
}

impl<'info> ProcessActivityReward<'info> {
    /// Process activity-based rewards with quality assessment
    pub fn process_activity(&mut self, activity_data: ActivityData) -> Result<()> {
        let clock = &self.clock;
        let current_time = clock.unix_timestamp;
        
        // Validate activity data
        require!(
            activity_data.is_valid(),
            FinovaError::InvalidActivityData
        );
        
        // Check daily limits
        let daily_activity_count = self.rewards_account.get_today_activity_count(current_time);
        require!(
            daily_activity_count < MAX_DAILY_ACTIVITIES,
            FinovaError::DailyActivityLimitExceeded
        );
        
        // Calculate base XP for activity
        let base_xp = self.calculate_base_xp(&activity_data)?;
        
        // Apply quality multipliers
        let quality_multiplier = self.assess_real_time_quality(&activity_data)?;
        
        // Apply streak bonus
        let streak_bonus = self.calculate_streak_bonus()?;
        
        // Apply level progression
        let level_progression = (-0.01 * self.xp_account.current_level as f64).exp();
        
        // Calculate final XP
        let final_xp = (base_xp as f64 * quality_multiplier * streak_bonus * level_progression) as u64;
        
        // Update XP account
        self.xp_account.total_xp += final_xp;
        self.xp_account.update_level()?;
        self.xp_account.last_activity_time = current_time;
        
        // Calculate mining bonus from activity
        let mining_bonus = (final_xp as f64 * ACTIVITY_MINING_MULTIPLIER) as u64;
        
        // Update rewards account
        let activity_record = ActivityRecord {
            activity_type: activity_data.activity_type,
            platform: activity_data.platform,
            timestamp: current_time,
            xp_earned: final_xp,
            mining_bonus,
            quality_score: quality_multiplier,
            engagement_metrics: activity_data.engagement_metrics,
            is_original: activity_data.is_original,
        };
        
        self.rewards_account.add_activity_record(activity_record)?;
        self.rewards_account.pending_rewards += mining_bonus;
        
        // Update referral network if applicable
        if let Some(referrer) = self.user.referrer {
            self.distribute_referral_activity_bonus(referrer, final_xp, mining_bonus)?;
        }
        
        emit!(ActivityProcessed {
            user: self.user.key(),
            activity_type: activity_data.activity_type,
            platform: activity_data.platform,
            xp_earned: final_xp,
            mining_bonus,
            quality_score: quality_multiplier,
            timestamp: current_time,
        });
        
        Ok(())
    }
    
    /// Calculate base XP for different activity types
    fn calculate_base_xp(&self, activity: &ActivityData) -> Result<u64> {
        let base_xp = match activity.activity_type {
            ActivityType::OriginalPost => XP_ORIGINAL_POST,
            ActivityType::QualityComment => XP_QUALITY_COMMENT,
            ActivityType::VideoContent => XP_VIDEO_CONTENT,
            ActivityType::ViralContent => XP_VIRAL_CONTENT,
            ActivityType::DailyLogin => XP_DAILY_LOGIN,
            ActivityType::ShareRepost => XP_SHARE_REPOST,
        };
        
        Ok(base_xp)
    }
    
    /// Assess real-time content quality
    fn assess_real_time_quality(&self, activity: &ActivityData) -> Result<f64> {
        let mut quality_score = 1.0;
        
        // Platform-specific multipliers
        quality_score *= match activity.platform {
            Platform::TikTok => 1.3,
            Platform::Instagram => 1.2,
            Platform::YouTube => 1.4,
            Platform::Facebook => 1.1,
            Platform::TwitterX => 1.2,
            Platform::App => 1.0,
        };
        
        // Engagement quality assessment
        let engagement = &activity.engagement_metrics;
        if engagement.views > 0 {
            let engagement_rate = (engagement.likes + engagement.comments + engagement.shares) as f64 
                / engagement.views as f64;
            
            if engagement_rate > 0.1 {
                quality_score *= 1.8; // High engagement
            } else if engagement_rate > 0.05 {
                quality_score *= 1.4; // Medium engagement
            } else if engagement_rate > 0.01 {
                quality_score *= 1.1; // Low engagement
            }
        }
        
        // Originality bonus
        if activity.is_original {
            quality_score *= 1.5;
        }
        
        // Content length consideration (for text content)
        if let Some(content_length) = activity.content_length {
            if content_length > 500 {
                quality_score *= 1.3; // Long-form content
            } else if content_length < 50 {
                quality_score *= 0.8; // Very short content
            }
        }
        
        Ok(quality_score.max(QUALITY_SCORE_MIN).min(QUALITY_SCORE_MAX))
    }
    
    /// Calculate streak bonus multiplier
    fn calculate_streak_bonus(&self) -> Result<f64> {
        let streak_days = self.xp_account.streak_days;
        let streak_bonus = match streak_days {
            0..=2 => 1.0,
            3..=6 => 1.2,
            7..=13 => 1.5,
            14..=29 => 2.0,
            30..=59 => 2.5,
            _ => 3.0, // 60+ days
        };
        
        Ok(streak_bonus)
    }
    
    /// Distribute referral activity bonus
    fn distribute_referral_activity_bonus(
        &mut self,
        referrer: Pubkey,
        xp_earned: u64,
        mining_bonus: u64,
    ) -> Result<()> {
        // Calculate referral bonuses
        let rp_bonus = (xp_earned as f64 * REFERRAL_XP_PERCENTAGE) as u64;
        let mining_referral_bonus = (mining_bonus as f64 * REFERRAL_MINING_PERCENTAGE) as u64;
        
        // Add to referral account RP
        self.referral_account.total_rp += rp_bonus;
        
        // Emit referral bonus event
        emit!(ReferralActivityBonus {
            referrer,
            referee: self.user.key(),
            xp_bonus: rp_bonus,
            mining_bonus: mining_referral_bonus,
            timestamp: self.clock.unix_timestamp,
        });
        
        Ok(())
    }
}

impl<'info> DistributeReferralRewards<'info> {
    /// Distribute network-based referral rewards
    pub fn distribute_network_rewards(&mut self, activity_value: u64) -> Result<()> {
        let clock = &self.clock;
        let current_time = clock.unix_timestamp;
        
        // Calculate L1 (direct) referral bonus
        let l1_bonus = self.calculate_l1_bonus(activity_value)?;
        
        // Calculate L2 (indirect) referral bonus
        let l2_bonus = self.calculate_l2_bonus(activity_value)?;
        
        // Calculate L3 (indirect) referral bonus
        let l3_bonus = self.calculate_l3_bonus(activity_value)?;
        
        // Apply network quality multiplier
        let network_quality = self.calculate_network_quality()?;
        
        let final_l1_bonus = (l1_bonus as f64 * network_quality) as u64;
        let final_l2_bonus = (l2_bonus as f64 * network_quality) as u64;
        let final_l3_bonus = (l3_bonus as f64 * network_quality) as u64;
        
        // Update referrer rewards
        self.referrer_rewards.pending_rewards += final_l1_bonus;
        self.referrer_rewards.referral_earnings += final_l1_bonus;
        
        // Update referrer network stats
        self.referrer_network.total_earnings += final_l1_bonus;
        self.referrer_network.last_earning_time = current_time;
        
        // Update referee network contribution
        self.referee_network.contribution_to_referrer += final_l1_bonus;
        
        // Transfer tokens if sufficient balance
        if self.reward_pool.amount >= final_l1_bonus {
            let transfer_ctx = CpiContext::new(
                self.token_program.to_account_info(),
                Transfer {
                    from: self.reward_pool.to_account_info(),
                    to: self.referrer_token_account.to_account_info(),
                    authority: self.global_state.to_account_info(),
                },
            );
            
            token::transfer(transfer_ctx, final_l1_bonus)?;
        } else {
            // Add to pending if insufficient pool balance
            self.referrer_rewards.pending_rewards += final_l1_bonus;
        }
        
        emit!(ReferralRewardDistributed {
            referrer: self.referrer.key(),
            referee: self.referee.key(),
            l1_bonus: final_l1_bonus,
            l2_bonus: final_l2_bonus,
            l3_bonus: final_l3_bonus,
            network_quality,
            timestamp: current_time,
        });
        
        Ok(())
    }
    
    /// Calculate L1 (direct referral) bonus
    fn calculate_l1_bonus(&self, activity_value: u64) -> Result<u64> {
        let tier = self.referrer_network.tier;
        let percentage = match tier {
            ReferralTier::Explorer => 0.10,
            ReferralTier::Connector => 0.15,
            ReferralTier::Influencer => 0.20,
            ReferralTier::Leader => 0.25,
            ReferralTier::Ambassador => 0.30,
        };
        
        Ok((activity_value as f64 * percentage) as u64)
    }
    
    /// Calculate L2 (second level) bonus
    fn calculate_l2_bonus(&self, activity_value: u64) -> Result<u64> {
        let tier = self.referrer_network.tier;
        let percentage = match tier {
            ReferralTier::Explorer => 0.0,
            ReferralTier::Connector => 0.05,
            ReferralTier::Influencer => 0.08,
            ReferralTier::Leader => 0.10,
            ReferralTier::Ambassador => 0.15,
        };
        
        Ok((activity_value as f64 * percentage) as u64)
    }
    
    /// Calculate L3 (third level) bonus
    fn calculate_l3_bonus(&self, activity_value: u64) -> Result<u64> {
        let tier = self.referrer_network.tier;
        let percentage = match tier {
            ReferralTier::Explorer => 0.0,
            ReferralTier::Connector => 0.0,
            ReferralTier::Influencer => 0.03,
            ReferralTier::Leader => 0.05,
            ReferralTier::Ambassador => 0.08,
        };
        
        Ok((activity_value as f64 * percentage) as u64)
    }
    
    /// Calculate network quality multiplier
    fn calculate_network_quality(&self) -> Result<f64> {
        let total_referrals = self.referrer_network.total_referrals;
        let active_referrals = self.referrer_network.active_referrals;
        
        if total_referrals == 0 {
            return Ok(1.0);
        }
        
        let activity_rate = active_referrals as f64 / total_referrals as f64;
        let quality_multiplier = 0.5 + (activity_rate * 1.5); // 0.5x to 2.0x based on activity
        
        Ok(quality_multiplier.max(0.5).min(2.0))
    }
}

impl<'info> ClaimRewards<'info> {
    /// Claim accumulated rewards
    pub fn claim_accumulated_rewards(&mut self) -> Result<()> {
        let clock = &self.clock;
        let current_time = clock.unix_timestamp;
        
        let pending_rewards = self.rewards_account.pending_rewards;
        
        require!(
            pending_rewards > 0,
            FinovaError::NoPendingRewards
        );
        
        require!(
            pending_rewards <= self.reward_pool.amount,
            FinovaError::InsufficientRewardPool
        );
        
        // Validate minimum claim interval
        require!(
            current_time >= self.rewards_account.last_claim_time + MIN_CLAIM_INTERVAL,
            FinovaError::ClaimTooSoon
        );
        
        // Transfer tokens from reward pool to user
        let transfer_ctx = CpiContext::new(
            self.token_program.to_account_info(),
            Transfer {
                from: self.reward_pool.to_account_info(),
                to: self.user_token_account.to_account_info(),
                authority: self.reward_pool.to_account_info(),
            },
        );
        
        token::transfer(transfer_ctx, pending_rewards)?;
        
        // Update rewards account
        self.rewards_account.pending_rewards = 0;
        self.rewards_account.total_claimed += pending_rewards;
        self.rewards_account.last_claim_time = current_time;
        self.rewards_account.claim_count += 1;
        
        // Update user stats
        self.user.total_claimed += pending_rewards;
        self.user.last_claim_time = current_time;
        
        emit!(RewardsClaimed {
            user: self.user.key(),
            amount: pending_rewards,
            total_claimed: self.rewards_account.total_claimed,
            timestamp: current_time,
        });
        
        Ok(())
    }
}

impl<'info> ApplySpecialCardBoost<'info> {
    /// Apply special card boost to rewards
    pub fn apply_card_boost(&mut self, card_type: SpecialCardType) -> Result<()> {
        let clock = &self.clock;
        let current_time = clock.unix_timestamp;
        
        // Verify user owns the special card
        require!(
            self.user_card_account.amount > 0,
            FinovaError::NoSpecialCardOwned
        );
        
        // Check if card is already active
        let existing_boost = self.rewards_account.active_boosts.iter()
            .find(|boost| boost.card_type == card_type && boost.is_active(current_time));
        
        require!(
            existing_boost.is_none(),
            FinovaError::CardBoostAlreadyActive
        );
        
        // Calculate boost parameters
        let (multiplier, duration) = self.get_card_boost_parameters(card_type)?;
        
        // Create new boost
        let new_boost = SpecialCardBoost {
            card_type,
            multiplier,
            start_time: current_time,
            end_time: current_time + duration,
            uses_remaining: self.get_card_uses(card_type),
        };
        
        // Add boost to active boosts
        require!(
            self.rewards_account.active_boosts.len() < MAX_ACTIVE_BOOSTS,
            FinovaError::TooManyActiveBoosts
        );
        
        self.rewards_account.active_boosts.push(new_boost);
        
        // Burn the card if it's single-use
        if self.is_single_use_card(card_type) {
            let burn_ctx = CpiContext::new(
                self.token_program.to_account_info(),
                token::Burn {
                    mint: self.special_card_mint.to_account_info(),
                    from: self.user_card_account.to_account_info(),
                    authority: self.user.to_account_info(),
                },
            );
            
            token::burn(burn_ctx, 1)?;
        }
        
        emit!(SpecialCardActivated {
            user: self.user.key(),
            card_type,
            multiplier,
            duration,
            timestamp: current_time,
        });
        
        Ok(())
    }
    
    /// Get boost parameters for different card types
    fn get_card_boost_parameters(&self, card_type: SpecialCardType) -> Result<(f64, i64)> {
        let (multiplier, duration_hours) = match card_type {
            SpecialCardType::DoubleMining => (2.0, 24),
            SpecialCardType::TripleMining => (3.0, 12),
            SpecialCardType::MiningFrenzy => (6.0, 4),
            SpecialCardType::EternalMiner => (1.5, 24 * 30), // 30 days
            SpecialCardType::XPDouble => (2.0, 24),
            SpecialCardType::StreakSaver => (1.0, 24 * 7), // 7 days
            SpecialCardType::LevelRush => (1.0, 0), // Instant effect
            SpecialCardType::XPMagnet => (4.0, 48),
            SpecialCardType::ReferralBoost => (1.5, 24 * 7), // 7 days
            SpecialCardType::NetworkAmplifier => (3.0, 24),
            SpecialCardType::AmbassadorPass => (2.0, 48),
            SpecialCardType::NetworkKing => (10.0, 12),
        };
        
        Ok((multiplier, duration_hours * 3600)) // Convert hours to seconds
    }
    
    /// Get number of uses for different card types
    fn get_card_uses(&self, card_type: SpecialCardType) -> Option<u32> {
        match card_type {
            SpecialCardType::DoubleMining => None, // Unlimited uses during duration
            SpecialCardType::TripleMining => None,
            SpecialCardType::MiningFrenzy => None,
            SpecialCardType::EternalMiner => None,
            SpecialCardType::XPDouble => None,
            SpecialCardType::StreakSaver => Some(1), // One-time save
            SpecialCardType::LevelRush => Some(1), // One-time XP boost
            SpecialCardType::XPMagnet => None,
            SpecialCardType::ReferralBoost => None,
            SpecialCardType::NetworkAmplifier => None,
            SpecialCardType::AmbassadorPass => None,
            SpecialCardType::NetworkKing => None,
        }
    }
    
    /// Check if card is single-use (gets burned after activation)
    fn is_single_use_card(&self, card_type: SpecialCardType) -> bool {
        matches!(card_type, 
            SpecialCardType::MiningFrenzy | 
            SpecialCardType::LevelRush | 
            SpecialCardType::NetworkKing
        )
    }
}

// Event definitions
#[event]
pub struct RewardCalculated {
    pub user: Pubkey,
    pub base_rate: u64,
    pub xp_multiplier: f64,
    pub rp_multiplier: f64,
    pub quality_score: f64,
    pub network_regression: f64,
    pub final_reward: u64,
    pub timestamp: i64,
}

#[event]
pub struct ActivityProcessed {
    pub user: Pubkey,
    pub activity_type: ActivityType,
    pub platform: Platform,
    pub xp_earned: u64,
    pub mining_bonus: u64,
    pub quality_score: f64,
    pub timestamp: i64,
}

#[event]
pub struct ReferralActivityBonus {
    pub referrer: Pubkey,
    pub referee: Pubkey,
    pub xp_bonus: u64,
    pub mining_bonus: u64,
    pub timestamp: i64,
}

#[event]
pub struct ReferralRewardDistributed {
    pub referrer: Pubkey,
    pub referee: Pubkey,
    pub l1_bonus: u64,
    pub l2_bonus: u64,
    pub l3_bonus: u64,
    pub network_quality: f64,
    pub timestamp: i64,
}

#[event]
pub struct RewardsClaimed {
    pub user: Pubkey,
    pub amount: u64,
    pub total_claimed: u64,
    pub timestamp: i64,
}

#[event]
pub struct SpecialCardActivated {
    pub user: Pubkey,
    pub card_type: SpecialCardType,
    pub multiplier: f64,
    pub duration: i64,
    pub timestamp: i64,
}

// Supporting enums and structs
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq)]
pub enum MiningPhase {
    Finizen,
    Growth,
    Maturity,
    Stability,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq)]
pub enum Platform {
    TikTok,
    Instagram,
    YouTube,
    Facebook,
    TwitterX,
    App,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq)]
pub enum ActivityType {
    OriginalPost,
    QualityComment,
    VideoContent,
    ViralContent,
    DailyLogin,
    ShareRepost,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq)]
pub enum ReferralTier {
    Explorer,
    Connector,
    Influencer,
    Leader,
    Ambassador,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq)]
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

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct ActivityData {
    pub activity_type: ActivityType,
    pub platform: Platform,
    pub engagement_metrics: EngagementMetrics,
    pub is_original: bool,
    pub content_length: Option<u32>,
}

impl ActivityData {
    pub fn is_valid(&self) -> bool {
        // Basic validation logic
        match self.activity_type {
            ActivityType::ViralContent => self.engagement_metrics.views >= 1000,
            ActivityType::VideoContent => self.content_length.unwrap_or(0) > 0,
            _ => true,
        }
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default)]
pub struct EngagementMetrics {
    pub views: u64,
    pub likes: u64,
    pub comments: u64,
    pub shares: u64,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct ActivityRecord {
    pub activity_type: ActivityType,
    pub platform: Platform,
    pub timestamp: i64,
    pub xp_earned: u64,
    pub mining_bonus: u64,
    pub quality_score: f64,
    pub engagement_metrics: EngagementMetrics,
    pub is_original: bool,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct SpecialCardBoost {
    pub card_type: SpecialCardType,
    pub multiplier: f64,
    pub start_time: i64,
    pub end_time: i64,
    pub uses_remaining: Option<u32>,
}

impl SpecialCardBoost {
    pub fn is_active(&self, current_time: i64) -> bool {
        current_time >= self.start_time && 
        current_time <= self.end_time &&
        self.uses_remaining.map_or(true, |uses| uses > 0)
    }
    
    pub fn apply_boost(&mut self) -> bool {
        if let Some(ref mut uses) = self.uses_remaining {
            if *uses > 0 {
                *uses -= 1;
                true
            } else {
                false
            }
        } else {
            true
        }
    }
}

// Instruction handlers
pub fn calculate_rewards(ctx: Context<CalculateRewards>) -> Result<()> {
    ctx.accounts.calculate_integrated_rewards()?;
    Ok(())
}

pub fn process_activity_reward(
    ctx: Context<ProcessActivityReward>,
    activity_data: ActivityData,
) -> Result<()> {
    ctx.accounts.process_activity(activity_data)?;
    Ok(())
}

pub fn distribute_referral_rewards(
    ctx: Context<DistributeReferralRewards>,
    activity_value: u64,
) -> Result<()> {
    ctx.accounts.distribute_network_rewards(activity_value)?;
    Ok(())
}

pub fn claim_rewards(ctx: Context<ClaimRewards>) -> Result<()> {
    ctx.accounts.claim_accumulated_rewards()?;
    Ok(())
}

pub fn apply_special_card_boost(
    ctx: Context<ApplySpecialCardBoost>,
    card_type: SpecialCardType,
) -> Result<()> {
    ctx.accounts.apply_card_boost(card_type)?;
    Ok(())
}

// Advanced reward calculation helpers
pub fn calculate_synergy_bonus(active_boosts: &[SpecialCardBoost], current_time: i64) -> f64 {
    let active_count = active_boosts.iter()
        .filter(|boost| boost.is_active(current_time))
        .count();
    
    if active_count == 0 {
        return 1.0;
    }
    
    let base_synergy = 1.0 + (active_count as f64 * 0.1);
    
    // Check for specific synergy combinations
    let has_mining_boost = active_boosts.iter().any(|boost| 
        boost.is_active(current_time) && matches!(boost.card_type, 
            SpecialCardType::DoubleMining | 
            SpecialCardType::TripleMining | 
            SpecialCardType::MiningFrenzy
        )
    );
    
    let has_xp_boost = active_boosts.iter().any(|boost| 
        boost.is_active(current_time) && matches!(boost.card_type, 
            SpecialCardType::XPDouble | 
            SpecialCardType::XPMagnet
        )
    );
    
    let has_referral_boost = active_boosts.iter().any(|boost| 
        boost.is_active(current_time) && matches!(boost.card_type, 
            SpecialCardType::ReferralBoost | 
            SpecialCardType::NetworkAmplifier
        )
    );
    
    // Triple synergy bonus
    if has_mining_boost && has_xp_boost && has_referral_boost {
        base_synergy * 1.5
    } else if (has_mining_boost && has_xp_boost) || 
              (has_mining_boost && has_referral_boost) || 
              (has_xp_boost && has_referral_boost) {
        base_synergy * 1.25
    } else {
        base_synergy
    }
}

pub fn calculate_whale_penalty(total_holdings: u64) -> f64 {
    if total_holdings <= WHALE_THRESHOLD {
        1.0
    } else {
        let excess = total_holdings - WHALE_THRESHOLD;
        let penalty_rate = (excess as f64 / WHALE_PENALTY_DENOMINATOR).min(WHALE_PENALTY_MAX);
        1.0 - penalty_rate
    }
}

pub fn calculate_network_effect_bonus(
    referral_network_size: u64,
    network_quality_score: f64,
) -> f64 {
    let size_bonus = (referral_network_size as f64).sqrt() / 100.0;
    let quality_multiplier = network_quality_score;
    
    (1.0 + size_bonus * quality_multiplier).min(NETWORK_EFFECT_MAX)
}
