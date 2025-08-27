// programs/finova-core/src/instructions/xp.rs

use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};
use crate::constants::*;
use crate::errors::*;
use crate::state::*;
use crate::utils::*;

/// Update user XP based on social media activity
#[derive(Accounts)]
pub struct UpdateXP<'info> {
    #[account(
        mut,
        seeds = [USER_SEED, user_authority.key().as_ref()],
        bump,
        constraint = user_account.authority == user_authority.key() @ FinovaError::Unauthorized
    )]
    pub user_account: Account<'info, UserAccount>,
    
    #[account(
        mut,
        seeds = [XP_SEED, user_authority.key().as_ref()],
        bump,
    )]
    pub xp_account: Account<'info, XPAccount>,
    
    #[account(
        mut,
        seeds = [MINING_SEED, user_authority.key().as_ref()],
        bump,
    )]
    pub mining_account: Account<'info, MiningAccount>,
    
    #[account(
        mut,
        seeds = [REWARDS_SEED, user_authority.key().as_ref()],
        bump,
    )]
    pub rewards_account: Account<'info, RewardsAccount>,
    
    #[account(mut)]
    pub user_authority: Signer<'info>,
    
    /// CHECK: XP oracle authority for validating social media activities
    #[account(
        constraint = xp_oracle.key() == XP_ORACLE_PUBKEY @ FinovaError::InvalidOracle
    )]
    pub xp_oracle: AccountInfo<'info>,
    
    pub system_program: Program<'info, System>,
    pub clock: Sysvar<'info, Clock>,
}

/// Add XP with comprehensive validation and bonuses
pub fn update_xp(
    ctx: Context<UpdateXP>,
    activity_type: XPActivityType,
    platform: SocialPlatform,
    content_hash: [u8; 32],
    engagement_metrics: EngagementMetrics,
    quality_score: u16, // 50-200 (0.5x - 2.0x multiplier)
) -> Result<()> {
    let user_account = &mut ctx.accounts.user_account;
    let xp_account = &mut ctx.accounts.xp_account;
    let mining_account = &mut ctx.accounts.mining_account;
    let rewards_account = &mut ctx.accounts.rewards_account;
    let clock = &ctx.accounts.clock;
    
    // Validate oracle signature for activity verification
    require!(
        is_valid_oracle_signature(&ctx.accounts.xp_oracle, &content_hash)?,
        FinovaError::InvalidOracleSignature
    );
    
    // Check for duplicate activity to prevent spam
    require!(
        !xp_account.recent_activities.contains(&content_hash),
        FinovaError::DuplicateActivity
    );
    
    // Validate quality score range
    require!(
        quality_score >= 50 && quality_score <= 200,
        FinovaError::InvalidQualityScore
    );
    
    // Calculate base XP for activity type
    let base_xp = match activity_type {
        XPActivityType::OriginalPost => 50,
        XPActivityType::PhotoImagePost => 75,
        XPActivityType::VideoContent => 150,
        XPActivityType::StoryStatus => 25,
        XPActivityType::MeaningfulComment => 25,
        XPActivityType::LikeReact => 5,
        XPActivityType::ShareRepost => 15,
        XPActivityType::FollowSubscribe => 20,
        XPActivityType::DailyLogin => 10,
        XPActivityType::CompleteDailyQuest => 100,
        XPActivityType::AchieveMilestone => 500,
        XPActivityType::ViralContent => 1000,
    };
    
    // Apply daily limits to prevent farming
    let current_day = clock.unix_timestamp / 86400; // Day since epoch
    if xp_account.last_activity_day != current_day {
        // Reset daily counters
        xp_account.daily_posts = 0;
        xp_account.daily_comments = 0;
        xp_account.daily_likes = 0;
        xp_account.daily_shares = 0;
        xp_account.daily_follows = 0;
        xp_account.last_activity_day = current_day;
    }
    
    // Check daily limits
    match activity_type {
        XPActivityType::OriginalPost | XPActivityType::PhotoImagePost | XPActivityType::VideoContent => {
            require!(
                xp_account.daily_posts < MAX_DAILY_POSTS,
                FinovaError::DailyLimitExceeded
            );
            xp_account.daily_posts += 1;
        },
        XPActivityType::MeaningfulComment => {
            require!(
                xp_account.daily_comments < MAX_DAILY_COMMENTS,
                FinovaError::DailyLimitExceeded
            );
            xp_account.daily_comments += 1;
        },
        XPActivityType::LikeReact => {
            require!(
                xp_account.daily_likes < MAX_DAILY_LIKES,
                FinovaError::DailyLimitExceeded
            );
            xp_account.daily_likes += 1;
        },
        XPActivityType::ShareRepost => {
            require!(
                xp_account.daily_shares < MAX_DAILY_SHARES,
                FinovaError::DailyLimitExceeded
            );
            xp_account.daily_shares += 1;
        },
        XPActivityType::FollowSubscribe => {
            require!(
                xp_account.daily_follows < MAX_DAILY_FOLLOWS,
                FinovaError::DailyLimitExceeded
            );
            xp_account.daily_follows += 1;
        },
        _ => {}, // No daily limits for other activities
    }
    
    // Calculate platform multiplier
    let platform_multiplier = match platform {
        SocialPlatform::TikTok => 130,      // 1.3x
        SocialPlatform::Instagram => 120,   // 1.2x
        SocialPlatform::YouTube => 140,     // 1.4x
        SocialPlatform::Facebook => 110,    // 1.1x
        SocialPlatform::TwitterX => 120,    // 1.2x
        SocialPlatform::App => 100,         // 1.0x (native app)
    };
    
    // Calculate streak bonus
    let current_timestamp = clock.unix_timestamp;
    let streak_bonus = if current_timestamp - xp_account.last_activity_timestamp <= 172800 { // 48 hours
        // Continue streak
        if current_timestamp - xp_account.last_activity_timestamp >= 82800 { // 23 hours minimum
            xp_account.current_streak += 1;
            if xp_account.current_streak > xp_account.longest_streak {
                xp_account.longest_streak = xp_account.current_streak;
            }
        }
        calculate_streak_multiplier(xp_account.current_streak)
    } else {
        // Reset streak
        xp_account.current_streak = 1;
        100 // 1.0x multiplier
    };
    
    // Calculate level progression factor (exponential decay for higher levels)
    let level_progression = calculate_level_progression_factor(xp_account.current_level);
    
    // Check for viral content bonus
    let viral_multiplier = if engagement_metrics.views >= 1000 
        || engagement_metrics.likes >= 100 
        || engagement_metrics.shares >= 50 {
        200 // 2.0x multiplier for viral content
    } else {
        100 // 1.0x multiplier
    };
    
    // Calculate final XP with all multipliers
    let final_xp = (base_xp as u64)
        .checked_mul(platform_multiplier as u64).unwrap()
        .checked_mul(quality_score as u64).unwrap()
        .checked_mul(streak_bonus as u64).unwrap()
        .checked_mul(level_progression as u64).unwrap()
        .checked_mul(viral_multiplier as u64).unwrap()
        .checked_div(100_000_000).unwrap(); // Normalize from percentage multipliers
    
    // Update XP account
    xp_account.total_xp = xp_account.total_xp.checked_add(final_xp).unwrap();
    xp_account.last_activity_timestamp = current_timestamp;
    xp_account.activities_count += 1;
    
    // Add to recent activities (maintain sliding window)
    if xp_account.recent_activities.len() >= MAX_RECENT_ACTIVITIES {
        xp_account.recent_activities.remove(0);
    }
    xp_account.recent_activities.push(content_hash);
    
    // Update engagement metrics
    xp_account.total_posts += match activity_type {
        XPActivityType::OriginalPost | XPActivityType::PhotoImagePost | XPActivityType::VideoContent => 1,
        _ => 0,
    };
    xp_account.total_comments += match activity_type {
        XPActivityType::MeaningfulComment => 1,
        _ => 0,
    };
    xp_account.total_likes += engagement_metrics.likes;
    xp_account.total_shares += engagement_metrics.shares;
    xp_account.total_views += engagement_metrics.views;
    
    // Check for level up
    let new_level = calculate_level_from_xp(xp_account.total_xp);
    if new_level > xp_account.current_level {
        let level_difference = new_level - xp_account.current_level;
        xp_account.current_level = new_level;
        
        // Grant level up rewards
        let level_up_reward = level_difference.checked_mul(LEVEL_UP_REWARD_BASE).unwrap();
        rewards_account.pending_fin_rewards = rewards_account.pending_fin_rewards
            .checked_add(level_up_reward).unwrap();
        
        // Update mining multiplier based on new level
        mining_account.xp_multiplier = calculate_xp_mining_multiplier(new_level);
        
        // Emit level up event
        emit!(XPLevelUpEvent {
            user: user_account.authority,
            old_level: new_level - level_difference,
            new_level,
            xp_gained: final_xp,
            reward_granted: level_up_reward,
            timestamp: current_timestamp,
        });
    }
    
    // Update tier and badge
    let new_tier = calculate_xp_tier(xp_account.current_level);
    if new_tier != xp_account.current_tier {
        xp_account.current_tier = new_tier;
        
        // Grant tier upgrade rewards
        let tier_reward = calculate_tier_reward(new_tier);
        rewards_account.pending_fin_rewards = rewards_account.pending_fin_rewards
            .checked_add(tier_reward).unwrap();
    }
    
    // Update special achievements
    check_and_update_achievements(xp_account, &engagement_metrics, current_timestamp)?;
    
    emit!(XPUpdateEvent {
        user: user_account.authority,
        activity_type,
        platform,
        base_xp,
        final_xp,
        total_xp: xp_account.total_xp,
        current_level: xp_account.current_level,
        current_streak: xp_account.current_streak,
        quality_score,
        engagement_metrics,
        timestamp: current_timestamp,
    });
    
    Ok(())
}

/// Grant bonus XP for special achievements
#[derive(Accounts)]
pub struct GrantBonusXP<'info> {
    #[account(
        mut,
        seeds = [XP_SEED, user_authority.key().as_ref()],
        bump,
    )]
    pub xp_account: Account<'info, XPAccount>,
    
    #[account(
        mut,
        seeds = [REWARDS_SEED, user_authority.key().as_ref()],
        bump,
    )]
    pub rewards_account: Account<'info, RewardsAccount>,
    
    pub user_authority: Pubkey,
    
    /// CHECK: Admin authority for granting bonus XP
    #[account(
        constraint = admin_authority.key() == ADMIN_PUBKEY @ FinovaError::Unauthorized
    )]
    pub admin_authority: Signer<'info>,
    
    pub clock: Sysvar<'info, Clock>,
}

pub fn grant_bonus_xp(
    ctx: Context<GrantBonusXP>,
    bonus_xp: u64,
    bonus_type: BonusXPType,
    reason: String,
) -> Result<()> {
    let xp_account = &mut ctx.accounts.xp_account;
    let rewards_account = &mut ctx.accounts.rewards_account;
    let clock = &ctx.accounts.clock;
    
    require!(bonus_xp <= MAX_BONUS_XP, FinovaError::ExcessiveBonusXP);
    require!(reason.len() <= MAX_REASON_LENGTH, FinovaError::ReasonTooLong);
    
    // Apply bonus XP
    xp_account.total_xp = xp_account.total_xp.checked_add(bonus_xp).unwrap();
    
    // Check for level up
    let old_level = xp_account.current_level;
    let new_level = calculate_level_from_xp(xp_account.total_xp);
    
    if new_level > old_level {
        xp_account.current_level = new_level;
        
        // Grant level up rewards
        let level_difference = new_level - old_level;
        let level_up_reward = level_difference.checked_mul(LEVEL_UP_REWARD_BASE).unwrap();
        rewards_account.pending_fin_rewards = rewards_account.pending_fin_rewards
            .checked_add(level_up_reward).unwrap();
    }
    
    // Record bonus in history
    if xp_account.bonus_history.len() >= MAX_BONUS_HISTORY {
        xp_account.bonus_history.remove(0);
    }
    xp_account.bonus_history.push(BonusXPRecord {
        bonus_type,
        amount: bonus_xp,
        timestamp: clock.unix_timestamp,
        reason: reason.clone(),
    });
    
    emit!(BonusXPGrantedEvent {
        user: ctx.accounts.user_authority,
        bonus_xp,
        bonus_type,
        reason,
        old_level,
        new_level,
        timestamp: clock.unix_timestamp,
    });
    
    Ok(())
}

/// Reset daily XP activities (called by cron job)
#[derive(Accounts)]
pub struct ResetDailyXP<'info> {
    #[account(
        mut,
        seeds = [XP_SEED, user_authority.key().as_ref()],
        bump,
    )]
    pub xp_account: Account<'info, XPAccount>,
    
    pub user_authority: Pubkey,
    
    /// CHECK: System authority for daily resets
    #[account(
        constraint = system_authority.key() == SYSTEM_AUTHORITY_PUBKEY @ FinovaError::Unauthorized
    )]
    pub system_authority: Signer<'info>,
    
    pub clock: Sysvar<'info, Clock>,
}

pub fn reset_daily_xp(ctx: Context<ResetDailyXP>) -> Result<()> {
    let xp_account = &mut ctx.accounts.xp_account;
    let clock = &ctx.accounts.clock;
    
    let current_day = clock.unix_timestamp / 86400;
    
    // Only reset if it's a new day
    if current_day > xp_account.last_activity_day {
        xp_account.daily_posts = 0;
        xp_account.daily_comments = 0;
        xp_account.daily_likes = 0;
        xp_account.daily_shares = 0;
        xp_account.daily_follows = 0;
        xp_account.last_activity_day = current_day;
        
        emit!(DailyXPResetEvent {
            user: ctx.accounts.user_authority,
            reset_day: current_day,
            timestamp: clock.unix_timestamp,
        });
    }
    
    Ok(())
}

/// Use special XP booster card
#[derive(Accounts)]
pub struct UseXPBooster<'info> {
    #[account(
        mut,
        seeds = [XP_SEED, user_authority.key().as_ref()],
        bump,
    )]
    pub xp_account: Account<'info, XPAccount>,
    
    #[account(mut)]
    pub user_authority: Signer<'info>,
    
    /// CHECK: NFT account representing the booster card
    pub booster_nft: AccountInfo<'info>,
    
    pub clock: Sysvar<'info, Clock>,
}

pub fn use_xp_booster(
    ctx: Context<UseXPBooster>,
    booster_type: XPBoosterType,
    duration_hours: u32,
) -> Result<()> {
    let xp_account = &mut ctx.accounts.xp_account;
    let clock = &ctx.accounts.clock;
    
    // Validate booster NFT ownership and type
    require!(
        validate_nft_ownership(&ctx.accounts.booster_nft, &ctx.accounts.user_authority.key())?,
        FinovaError::InvalidNFTOwnership
    );
    
    require!(duration_hours <= MAX_BOOSTER_DURATION_HOURS, FinovaError::InvalidBoosterDuration);
    
    let current_timestamp = clock.unix_timestamp;
    let expiry_timestamp = current_timestamp + (duration_hours as i64 * 3600);
    
    // Check if there's already an active booster
    if xp_account.active_booster_expiry > current_timestamp {
        // Extend the existing booster if same type, otherwise replace
        if xp_account.active_booster_type == booster_type {
            xp_account.active_booster_expiry = xp_account.active_booster_expiry
                .checked_add(duration_hours as i64 * 3600).unwrap();
        } else {
            // Replace with new booster
            xp_account.active_booster_type = booster_type;
            xp_account.active_booster_expiry = expiry_timestamp;
        }
    } else {
        // Activate new booster
        xp_account.active_booster_type = booster_type;
        xp_account.active_booster_expiry = expiry_timestamp;
    }
    
    xp_account.boosters_used += 1;
    
    emit!(XPBoosterUsedEvent {
        user: ctx.accounts.user_authority.key(),
        booster_type,
        duration_hours,
        expiry_timestamp,
        timestamp: current_timestamp,
    });
    
    Ok(())
}

// Helper functions
fn calculate_streak_multiplier(streak_days: u32) -> u64 {
    match streak_days {
        0..=6 => 100,          // 1.0x
        7..=13 => 110,         // 1.1x
        14..=29 => 125,        // 1.25x
        30..=59 => 150,        // 1.5x
        60..=99 => 200,        // 2.0x
        100..=199 => 250,      // 2.5x
        _ => 300,              // 3.0x max
    }
}

fn calculate_level_progression_factor(level: u32) -> u64 {
    // Exponential decay: e^(-0.01 * level) * 100
    let decay_factor = (-0.01 * level as f64).exp();
    (decay_factor * 100.0) as u64
}

fn calculate_level_from_xp(total_xp: u64) -> u32 {
    // Level calculation based on cumulative XP requirements
    match total_xp {
        0..=999 => total_xp as u32 / 100 + 1,                    // Levels 1-10
        1000..=4999 => 10 + (total_xp - 1000) as u32 / 266 + 1,  // Levels 11-25
        5000..=19999 => 25 + (total_xp - 5000) as u32 / 600 + 1, // Levels 26-50
        20000..=49999 => 50 + (total_xp - 20000) as u32 / 1200 + 1, // Levels 51-75
        50000..=99999 => 75 + (total_xp - 50000) as u32 / 2000 + 1, // Levels 76-100
        _ => 100 + (total_xp - 100000) as u32 / 5000 + 1,        // Levels 101+
    }
}

fn calculate_xp_tier(level: u32) -> XPTier {
    match level {
        1..=10 => XPTier::Bronze,
        11..=25 => XPTier::Silver,
        26..=50 => XPTier::Gold,
        51..=75 => XPTier::Platinum,
        76..=100 => XPTier::Diamond,
        _ => XPTier::Mythic,
    }
}

fn calculate_xp_mining_multiplier(level: u32) -> u64 {
    match level {
        1..=10 => 100 + (level as u64 * 2),      // 1.0x - 1.2x
        11..=25 => 130 + ((level - 10) as u64 * 3), // 1.3x - 1.8x
        26..=50 => 190 + ((level - 25) as u64 * 2), // 1.9x - 2.4x
        51..=75 => 260 + ((level - 50) as u64 * 2), // 2.6x - 3.1x
        76..=100 => 330 + ((level - 75) as u64 * 2), // 3.3x - 3.8x
        _ => 410 + ((level - 100) as u64 * 1),   // 4.1x+ (max 5.0x)
    }.min(500) // Cap at 5.0x
}

fn calculate_tier_reward(tier: XPTier) -> u64 {
    match tier {
        XPTier::Bronze => 100,      // 100 $FIN
        XPTier::Silver => 500,      // 500 $FIN
        XPTier::Gold => 2000,       // 2000 $FIN
        XPTier::Platinum => 5000,   // 5000 $FIN
        XPTier::Diamond => 15000,   // 15000 $FIN
        XPTier::Mythic => 50000,    // 50000 $FIN
    }
}

fn check_and_update_achievements(
    xp_account: &mut XPAccount,
    engagement_metrics: &EngagementMetrics,
    timestamp: i64,
) -> Result<()> {
    // Check for viral content achievement
    if engagement_metrics.views >= 10000 && !xp_account.achievements.contains(&Achievement::ViralCreator) {
        xp_account.achievements.push(Achievement::ViralCreator);
        
        emit!(AchievementUnlockedEvent {
            user: xp_account.authority,
            achievement: Achievement::ViralCreator,
            timestamp,
        });
    }
    
    // Check for consistency achievement
    if xp_account.current_streak >= 30 && !xp_account.achievements.contains(&Achievement::ConsistentCreator) {
        xp_account.achievements.push(Achievement::ConsistentCreator);
        
        emit!(AchievementUnlockedEvent {
            user: xp_account.authority,
            achievement: Achievement::ConsistentCreator,
            timestamp,
        });
    }
    
    // Check for engagement master achievement
    if xp_account.total_likes >= 10000 && !xp_account.achievements.contains(&Achievement::EngagementMaster) {
        xp_account.achievements.push(Achievement::EngagementMaster);
        
        emit!(AchievementUnlockedEvent {
            user: xp_account.authority,
            achievement: Achievement::EngagementMaster,
            timestamp,
        });
    }
    
    Ok(())
}

fn is_valid_oracle_signature(oracle: &AccountInfo, content_hash: &[u8; 32]) -> Result<bool> {
    // Implement oracle signature verification logic
    // This would verify that the oracle has signed off on the social media activity
    // For now, return true (in production, implement proper signature verification)
    Ok(true)
}

fn validate_nft_ownership(nft_account: &AccountInfo, owner: &Pubkey) -> Result<bool> {
    // Implement NFT ownership verification
    // This would check that the user owns the specific booster NFT
    // For now, return true (in production, implement proper NFT verification)
    Ok(true)
}

// Events
#[event]
pub struct XPUpdateEvent {
    pub user: Pubkey,
    pub activity_type: XPActivityType,
    pub platform: SocialPlatform,
    pub base_xp: u32,
    pub final_xp: u64,
    pub total_xp: u64,
    pub current_level: u32,
    pub current_streak: u32,
    pub quality_score: u16,
    pub engagement_metrics: EngagementMetrics,
    pub timestamp: i64,
}

#[event]
pub struct XPLevelUpEvent {
    pub user: Pubkey,
    pub old_level: u32,
    pub new_level: u32,
    pub xp_gained: u64,
    pub reward_granted: u64,
    pub timestamp: i64,
}

#[event]
pub struct BonusXPGrantedEvent {
    pub user: Pubkey,
    pub bonus_xp: u64,
    pub bonus_type: BonusXPType,
    pub reason: String,
    pub old_level: u32,
    pub new_level: u32,
    pub timestamp: i64,
}

#[event]
pub struct DailyXPResetEvent {
    pub user: Pubkey,
    pub reset_day: i64,
    pub timestamp: i64,
}

#[event]
pub struct XPBoosterUsedEvent {
    pub user: Pubkey,
    pub booster_type: XPBoosterType,
    pub duration_hours: u32,
    pub expiry_timestamp: i64,
    pub timestamp: i64,
}

#[event]
pub struct AchievementUnlockedEvent {
    pub user: Pubkey,
    pub achievement: Achievement,
    pub timestamp: i64,
}
