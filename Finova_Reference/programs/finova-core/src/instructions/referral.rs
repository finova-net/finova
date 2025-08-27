// programs/finova-core/src/instructions/referral.rs

use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};
use crate::constants::*;
use crate::errors::*;
use crate::state::*;
use crate::utils::*;
use std::collections::HashMap;

/// Initialize referral system for a user
#[derive(Accounts)]
#[instruction(referral_code: String)]
pub struct InitializeReferral<'info> {
    #[account(
        init,
        payer = user,
        space = 8 + ReferralAccount::INIT_SPACE,
        seeds = [REFERRAL_SEED, user.key().as_ref()],
        bump
    )]
    pub referral_account: Account<'info, ReferralAccount>,
    
    #[account(
        mut,
        seeds = [USER_SEED, user.key().as_ref()],
        bump
    )]
    pub user_account: Account<'info, UserAccount>,
    
    #[account(mut)]
    pub user: Signer<'info>,
    
    #[account(
        mut,
        seeds = [NETWORK_SEED],
        bump
    )]
    pub network_state: Account<'info, NetworkState>,
    
    pub system_program: Program<'info, System>,
}

/// Register a referral using someone's referral code
#[derive(Accounts)]
#[instruction(referrer_code: String)]
pub struct RegisterReferral<'info> {
    #[account(
        mut,
        seeds = [REFERRAL_SEED, referee.key().as_ref()],
        bump,
        constraint = referral_account.owner == referee.key() @ FinovaError::UnauthorizedAccess
    )]
    pub referral_account: Account<'info, ReferralAccount>,
    
    #[account(
        mut,
        seeds = [USER_SEED, referee.key().as_ref()],
        bump
    )]
    pub referee_account: Account<'info, UserAccount>,
    
    #[account(
        mut,
        seeds = [REFERRAL_SEED, referrer.key().as_ref()],
        bump
    )]
    pub referrer_account: Account<'info, ReferralAccount>,
    
    #[account(
        mut,
        seeds = [USER_SEED, referrer.key().as_ref()],
        bump
    )]
    pub referrer_user_account: Account<'info, UserAccount>,
    
    #[account(mut)]
    pub referee: Signer<'info>,
    
    /// CHECK: Referrer account verified through PDA seeds
    pub referrer: UncheckedAccount<'info>,
    
    #[account(
        mut,
        seeds = [NETWORK_SEED],
        bump
    )]
    pub network_state: Account<'info, NetworkState>,
}

/// Update referral network when user gains XP
#[derive(Accounts)]
pub struct UpdateReferralRewards<'info> {
    #[account(
        mut,
        seeds = [REFERRAL_SEED, user.key().as_ref()],
        bump
    )]
    pub referral_account: Account<'info, ReferralAccount>,
    
    #[account(
        mut,
        seeds = [USER_SEED, user.key().as_ref()],
        bump
    )]
    pub user_account: Account<'info, UserAccount>,
    
    #[account(mut)]
    pub user: Signer<'info>,
    
    #[account(
        mut,
        seeds = [NETWORK_SEED],
        bump
    )]
    pub network_state: Account<'info, NetworkState>,
}

/// Claim referral rewards
#[derive(Accounts)]
pub struct ClaimReferralRewards<'info> {
    #[account(
        mut,
        seeds = [REFERRAL_SEED, user.key().as_ref()],
        bump,
        constraint = referral_account.owner == user.key() @ FinovaError::UnauthorizedAccess
    )]
    pub referral_account: Account<'info, ReferralAccount>,
    
    #[account(
        mut,
        seeds = [USER_SEED, user.key().as_ref()],
        bump
    )]
    pub user_account: Account<'info, UserAccount>,
    
    #[account(
        mut,
        constraint = user_token_account.owner == user.key() @ FinovaError::UnauthorizedAccess
    )]
    pub user_token_account: Account<'info, TokenAccount>,
    
    #[account(
        mut,
        seeds = [TREASURY_SEED],
        bump
    )]
    pub treasury_account: Account<'info, TokenAccount>,
    
    #[account(mut)]
    pub user: Signer<'info>,
    
    #[account(
        mut,
        seeds = [NETWORK_SEED],
        bump
    )]
    pub network_state: Account<'info, NetworkState>,
    
    pub token_program: Program<'info, Token>,
}

/// Upgrade referral tier based on achievements
#[derive(Accounts)]
pub struct UpgradeReferralTier<'info> {
    #[account(
        mut,
        seeds = [REFERRAL_SEED, user.key().as_ref()],
        bump,
        constraint = referral_account.owner == user.key() @ FinovaError::UnauthorizedAccess
    )]
    pub referral_account: Account<'info, ReferralAccount>,
    
    #[account(
        mut,
        seeds = [USER_SEED, user.key().as_ref()],
        bump
    )]
    pub user_account: Account<'info, UserAccount>,
    
    #[account(mut)]
    pub user: Signer<'info>,
    
    #[account(
        mut,
        seeds = [NETWORK_SEED],
        bump
    )]
    pub network_state: Account<'info, NetworkState>,
}

/// Calculate network quality and apply regression
#[derive(Accounts)]
pub struct CalculateNetworkQuality<'info> {
    #[account(
        mut,
        seeds = [REFERRAL_SEED, user.key().as_ref()],
        bump
    )]
    pub referral_account: Account<'info, ReferralAccount>,
    
    #[account(
        mut,
        seeds = [USER_SEED, user.key().as_ref()],
        bump
    )]
    pub user_account: Account<'info, UserAccount>,
    
    #[account(mut)]
    pub user: Signer<'info>,
    
    #[account(
        mut,
        seeds = [NETWORK_SEED],
        bump
    )]
    pub network_state: Account<'info, NetworkState>,
}

pub fn initialize_referral(
    ctx: Context<InitializeReferral>,
    referral_code: String,
) -> Result<()> {
    let referral_account = &mut ctx.accounts.referral_account;
    let user_account = &mut ctx.accounts.user_account;
    let network_state = &mut ctx.accounts.network_state;
    let clock = Clock::get()?;

    require!(
        referral_code.len() >= MIN_REFERRAL_CODE_LENGTH 
        && referral_code.len() <= MAX_REFERRAL_CODE_LENGTH,
        FinovaError::InvalidReferralCode
    );

    require!(
        is_valid_referral_code(&referral_code),
        FinovaError::InvalidReferralCode
    );

    // Check if referral code is unique (simplified check)
    require!(
        !network_state.used_referral_codes.contains(&referral_code),
        FinovaError::ReferralCodeAlreadyExists
    );

    // Initialize referral account
    referral_account.owner = ctx.accounts.user.key();
    referral_account.referral_code = referral_code.clone();
    referral_account.total_referrals = 0;
    referral_account.active_referrals = 0;
    referral_account.total_rp = 0;
    referral_account.claimable_rewards = 0;
    referral_account.referrer = None;
    referral_account.tier = ReferralTier::Explorer;
    referral_account.network_size = 0;
    referral_account.network_quality_score = 1000; // Start with 100.0% (scaled by 10)
    referral_account.last_activity = clock.unix_timestamp;
    referral_account.created_at = clock.unix_timestamp;
    referral_account.direct_referrals = Vec::new();
    referral_account.level_2_referrals = Vec::new();
    referral_account.level_3_referrals = Vec::new();
    referral_account.monthly_stats = ReferralMonthlyStats::default();
    referral_account.lifetime_stats = ReferralLifetimeStats::default();
    referral_account.regression_factor = 1000; // 100.0% (scaled by 10)
    referral_account.bump = ctx.bumps.referral_account;

    // Update user account
    user_account.has_referral_system = true;
    user_account.referral_tier = ReferralTier::Explorer;

    // Update network state
    network_state.used_referral_codes.push(referral_code);
    network_state.total_referral_accounts += 1;

    emit!(ReferralSystemInitialized {
        user: ctx.accounts.user.key(),
        referral_code: referral_account.referral_code.clone(),
        timestamp: clock.unix_timestamp,
    });

    Ok(())
}

pub fn register_referral(
    ctx: Context<RegisterReferral>,
    referrer_code: String,
) -> Result<()> {
    let referral_account = &mut ctx.accounts.referral_account;
    let referee_account = &mut ctx.accounts.referee_account;
    let referrer_account = &mut ctx.accounts.referrer_account;
    let referrer_user_account = &mut ctx.accounts.referrer_user_account;
    let network_state = &mut ctx.accounts.network_state;
    let clock = Clock::get()?;

    // Validate referrer code exists and matches
    require!(
        referrer_account.referral_code == referrer_code,
        FinovaError::InvalidReferralCode
    );

    // Cannot refer yourself
    require!(
        ctx.accounts.referee.key() != ctx.accounts.referrer.key(),
        FinovaError::CannotReferSelf
    );

    // User hasn't been referred before
    require!(
        referral_account.referrer.is_none(),
        FinovaError::AlreadyReferred
    );

    // Check referrer's network capacity
    let max_referrals = get_max_referrals_for_tier(referrer_account.tier);
    require!(
        referrer_account.direct_referrals.len() < max_referrals,
        FinovaError::ReferralCapacityExceeded
    );

    // Register the referral relationship
    referral_account.referrer = Some(ctx.accounts.referrer.key());
    
    // Add to referrer's direct referrals
    referrer_account.direct_referrals.push(ReferralInfo {
        user: ctx.accounts.referee.key(),
        joined_at: clock.unix_timestamp,
        total_xp_contributed: 0,
        total_mining_contributed: 0,
        last_activity: clock.unix_timestamp,
        is_active: true,
        kyc_verified: referee_account.kyc_verified,
    });

    referrer_account.total_referrals += 1;
    referrer_account.active_referrals += 1;
    referrer_account.network_size += 1;

    // Award initial RP to referrer
    let initial_rp = calculate_initial_referral_rp(&referee_account);
    referrer_account.total_rp += initial_rp;
    referrer_account.claimable_rewards += initial_rp;

    // Update monthly stats
    referrer_account.monthly_stats.new_referrals += 1;
    referrer_account.monthly_stats.rp_earned += initial_rp;

    // Update lifetime stats
    referrer_account.lifetime_stats.total_referrals += 1;
    referrer_account.lifetime_stats.total_rp_earned += initial_rp;

    // Check for tier upgrade
    let new_tier = calculate_referral_tier(referrer_account.total_rp, referrer_account.active_referrals);
    if new_tier as u8 > referrer_account.tier as u8 {
        referrer_account.tier = new_tier;
        referrer_user_account.referral_tier = new_tier;
        
        emit!(ReferralTierUpgraded {
            user: ctx.accounts.referrer.key(),
            old_tier: referrer_account.tier,
            new_tier,
            timestamp: clock.unix_timestamp,
        });
    }

    // Update network state
    network_state.total_referral_connections += 1;
    network_state.total_rp_distributed += initial_rp;

    // Calculate and update network quality
    update_network_quality_score(referrer_account);

    emit!(ReferralRegistered {
        referee: ctx.accounts.referee.key(),
        referrer: ctx.accounts.referrer.key(),
        referrer_code,
        rp_awarded: initial_rp,
        timestamp: clock.unix_timestamp,
    });

    Ok(())
}

pub fn update_referral_rewards(
    ctx: Context<UpdateReferralRewards>,
    xp_gained: u64,
    mining_amount: u64,
) -> Result<()> {
    let referral_account = &mut ctx.accounts.referral_account;
    let user_account = &ctx.accounts.user_account;
    let network_state = &mut ctx.accounts.network_state;
    let clock = Clock::get()?;

    // Only process if user has a referrer
    if let Some(referrer_key) = referral_account.referrer {
        // Calculate RP rewards based on activity
        let xp_rp = calculate_xp_referral_rp(xp_gained, user_account.xp_level);
        let mining_rp = calculate_mining_referral_rp(mining_amount, user_account.mining_rate);

        let total_rp = xp_rp + mining_rp;

        if total_rp > 0 {
            // Update user's contribution to referrer
            referral_account.last_activity = clock.unix_timestamp;

            // Note: In a full implementation, we would need to iterate through
            // referrer accounts and update their rewards. This is simplified
            // for space constraints.

            network_state.total_rp_distributed += total_rp;

            emit!(ReferralRewardsUpdated {
                user: ctx.accounts.user.key(),
                referrer: referrer_key,
                xp_rp,
                mining_rp,
                total_rp,
                timestamp: clock.unix_timestamp,
            });
        }
    }

    Ok(())
}

pub fn claim_referral_rewards(
    ctx: Context<ClaimReferralRewards>,
) -> Result<()> {
    let referral_account = &mut ctx.accounts.referral_account;
    let user_account = &mut ctx.accounts.user_account;
    let network_state = &mut ctx.accounts.network_state;
    let clock = Clock::get()?;

    require!(
        referral_account.claimable_rewards > 0,
        FinovaError::NoRewardsToClaim
    );

    let rewards_to_claim = referral_account.claimable_rewards;

    // Apply tier multiplier
    let tier_multiplier = get_referral_tier_multiplier(referral_account.tier);
    let final_rewards = (rewards_to_claim * tier_multiplier) / 1000; // Divide by 1000 for scaling

    // Apply network regression
    let regressed_rewards = (final_rewards * referral_account.regression_factor) / 1000;

    require!(
        regressed_rewards > 0,
        FinovaError::InsufficientRewards
    );

    // Transfer tokens from treasury to user
    let cpi_accounts = Transfer {
        from: ctx.accounts.treasury_account.to_account_info(),
        to: ctx.accounts.user_token_account.to_account_info(),
        authority: ctx.accounts.network_state.to_account_info(),
    };

    let seeds = &[
        NETWORK_SEED,
        &[ctx.bumps.network_state],
    ];
    let signer_seeds = &[&seeds[..]];

    let cpi_program = ctx.accounts.token_program.to_account_info();
    let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer_seeds);

    token::transfer(cpi_ctx, regressed_rewards)?;

    // Update accounts
    referral_account.claimable_rewards = 0;
    referral_account.lifetime_stats.total_rewards_claimed += regressed_rewards;
    user_account.total_rewards_claimed += regressed_rewards;

    // Update network state
    network_state.total_rewards_distributed += regressed_rewards;

    emit!(ReferralRewardsClaimed {
        user: ctx.accounts.user.key(),
        rewards_claimed: regressed_rewards,
        tier_multiplier,
        regression_factor: referral_account.regression_factor,
        timestamp: clock.unix_timestamp,
    });

    Ok(())
}

pub fn upgrade_referral_tier(
    ctx: Context<UpgradeReferralTier>,
) -> Result<()> {
    let referral_account = &mut ctx.accounts.referral_account;
    let user_account = &mut ctx.accounts.user_account;
    let clock = Clock::get()?;

    let current_tier = referral_account.tier;
    let new_tier = calculate_referral_tier(referral_account.total_rp, referral_account.active_referrals);

    require!(
        new_tier as u8 > current_tier as u8,
        FinovaError::TierUpgradeNotAvailable
    );

    // Update tier
    referral_account.tier = new_tier;
    user_account.referral_tier = new_tier;

    // Award tier upgrade bonus
    let upgrade_bonus = get_tier_upgrade_bonus(new_tier);
    referral_account.total_rp += upgrade_bonus;
    referral_account.claimable_rewards += upgrade_bonus;

    emit!(ReferralTierUpgraded {
        user: ctx.accounts.user.key(),
        old_tier: current_tier,
        new_tier,
        timestamp: clock.unix_timestamp,
    });

    Ok(())
}

pub fn calculate_network_quality(
    ctx: Context<CalculateNetworkQuality>,
) -> Result<()> {
    let referral_account = &mut ctx.accounts.referral_account;
    let clock = Clock::get()?;

    // Calculate network quality based on various factors
    let quality_score = calculate_network_quality_score(referral_account, clock.unix_timestamp);
    referral_account.network_quality_score = quality_score;

    // Calculate regression factor based on network size and quality
    let regression_factor = calculate_network_regression_factor(
        referral_account.network_size,
        quality_score,
    );
    referral_account.regression_factor = regression_factor;

    emit!(NetworkQualityUpdated {
        user: ctx.accounts.user.key(),
        quality_score,
        regression_factor,
        network_size: referral_account.network_size,
        timestamp: clock.unix_timestamp,
    });

    Ok(())
}

// Helper functions

fn is_valid_referral_code(code: &str) -> bool {
    code.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-')
}

fn get_max_referrals_for_tier(tier: ReferralTier) -> usize {
    match tier {
        ReferralTier::Explorer => 10,
        ReferralTier::Connector => 25,
        ReferralTier::Influencer => 50,
        ReferralTier::Leader => 100,
        ReferralTier::Ambassador => usize::MAX,
    }
}

fn calculate_initial_referral_rp(referee_account: &UserAccount) -> u64 {
    let base_rp = if referee_account.kyc_verified { 100 } else { 50 };
    let level_bonus = referee_account.xp_level / 10;
    base_rp + level_bonus
}

fn calculate_referral_tier(total_rp: u64, active_referrals: u32) -> ReferralTier {
    match total_rp {
        0..=999 => ReferralTier::Explorer,
        1000..=4999 => ReferralTier::Connector,
        5000..=14999 => ReferralTier::Influencer,
        15000..=49999 => ReferralTier::Leader,
        _ => ReferralTier::Ambassador,
    }
}

fn get_referral_tier_multiplier(tier: ReferralTier) -> u64 {
    match tier {
        ReferralTier::Explorer => 1000,    // 1.0x
        ReferralTier::Connector => 1200,   // 1.2x
        ReferralTier::Influencer => 1500,  // 1.5x
        ReferralTier::Leader => 2000,      // 2.0x
        ReferralTier::Ambassador => 3000,  // 3.0x
    }
}

fn get_tier_upgrade_bonus(tier: ReferralTier) -> u64 {
    match tier {
        ReferralTier::Explorer => 0,
        ReferralTier::Connector => 500,
        ReferralTier::Influencer => 1500,
        ReferralTier::Leader => 5000,
        ReferralTier::Ambassador => 15000,
    }
}

fn calculate_xp_referral_rp(xp_gained: u64, referrer_level: u32) -> u64 {
    let base_rp = (xp_gained * 5) / 100; // 5% of XP as RP
    let level_multiplier = 1000 + (referrer_level * 10); // Level bonus
    (base_rp * level_multiplier) / 1000
}

fn calculate_mining_referral_rp(mining_amount: u64, mining_rate: u64) -> u64 {
    let base_rp = (mining_amount * 10) / 100; // 10% of mining as RP
    let rate_multiplier = if mining_rate > 50000 { 1500 } else { 1000 }; // 1.5x for high miners
    (base_rp * rate_multiplier) / 1000
}

fn update_network_quality_score(referral_account: &mut ReferralAccount) {
    let quality_score = calculate_network_quality_score(referral_account, Clock::get().unwrap().unix_timestamp);
    referral_account.network_quality_score = quality_score;
    
    let regression_factor = calculate_network_regression_factor(
        referral_account.network_size,
        quality_score,
    );
    referral_account.regression_factor = regression_factor;
}

fn calculate_network_quality_score(referral_account: &ReferralAccount, current_time: i64) -> u64 {
    if referral_account.total_referrals == 0 {
        return 1000; // 100.0%
    }

    // Calculate active ratio
    let active_ratio = (referral_account.active_referrals * 1000) / referral_account.total_referrals;
    
    // Calculate average activity level (simplified)
    let activity_score = if referral_account.last_activity > 0 {
        let days_since_activity = (current_time - referral_account.last_activity) / 86400;
        if days_since_activity < 30 { 1000 } else { 500 }
    } else { 500 };

    // Calculate diversity bonus (simplified)
    let diversity_bonus = if referral_account.network_size > referral_account.total_referrals {
        200 // Bonus for having multi-level network
    } else { 0 };

    // Weighted average
    ((active_ratio * 6) + (activity_score * 3) + (diversity_bonus * 1)) / 10
}

fn calculate_network_regression_factor(network_size: u32, quality_score: u64) -> u64 {
    // Base regression: e^(-0.0001 × network_size × quality_factor)
    // Simplified exponential approximation
    let quality_factor = quality_score as f64 / 1000.0; // Convert back to decimal
    let exponent = -0.0001 * network_size as f64 * quality_factor;
    
    // Approximate e^x for small negative values
    let regression = if exponent > -5.0 {
        (1000.0 * (1.0 + exponent + (exponent * exponent / 2.0))) as u64
    } else {
        50 // Minimum 5% for very large networks
    };

    regression.max(50).min(1000) // Clamp between 5% and 100%
}

// Events
#[event]
pub struct ReferralSystemInitialized {
    pub user: Pubkey,
    pub referral_code: String,
    pub timestamp: i64,
}

#[event]
pub struct ReferralRegistered {
    pub referee: Pubkey,
    pub referrer: Pubkey,
    pub referrer_code: String,
    pub rp_awarded: u64,
    pub timestamp: i64,
}

#[event]
pub struct ReferralRewardsUpdated {
    pub user: Pubkey,
    pub referrer: Pubkey,
    pub xp_rp: u64,
    pub mining_rp: u64,
    pub total_rp: u64,
    pub timestamp: i64,
}

#[event]
pub struct ReferralRewardsClaimed {
    pub user: Pubkey,
    pub rewards_claimed: u64,
    pub tier_multiplier: u64,
    pub regression_factor: u64,
    pub timestamp: i64,
}

#[event]
pub struct ReferralTierUpgraded {
    pub user: Pubkey,
    pub old_tier: ReferralTier,
    pub new_tier: ReferralTier,
    pub timestamp: i64,
}

#[event]
pub struct NetworkQualityUpdated {
    pub user: Pubkey,
    pub quality_score: u64,
    pub regression_factor: u64,
    pub network_size: u32,
    pub timestamp: i64,
}
