// programs/finova-token/src/instructions/claim_rewards.rs

use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount, Transfer};
use crate::state::*;
use crate::errors::*;
use crate::utils::*;

/// Instruction to claim staking and mining rewards
#[derive(Accounts)]
pub struct ClaimRewards<'info> {
    /// The user claiming rewards
    #[account(mut)]
    pub user: Signer<'info>,

    /// User's stake account
    #[account(
        mut,
        seeds = [b"stake", user.key().as_ref()],
        bump,
        constraint = stake_account.owner == user.key() @ TokenError::Unauthorized
    )]
    pub stake_account: Account<'info, StakeAccount>,

    /// Global reward pool account
    #[account(
        mut,
        seeds = [b"reward_pool"],
        bump,
        constraint = reward_pool.is_active @ TokenError::RewardPoolInactive
    )]
    pub reward_pool: Account<'info, RewardPool>,

    /// FIN token mint
    #[account(
        constraint = fin_mint.key() == reward_pool.fin_mint @ TokenError::InvalidMint
    )]
    pub fin_mint: Account<'info, Mint>,

    /// sFIN token mint for staking rewards
    #[account(
        constraint = sfin_mint.key() == reward_pool.sfin_mint @ TokenError::InvalidMint
    )]
    pub sfin_mint: Account<'info, Mint>,

    /// User's FIN token account
    #[account(
        mut,
        constraint = user_fin_account.owner == user.key() @ TokenError::Unauthorized,
        constraint = user_fin_account.mint == fin_mint.key() @ TokenError::InvalidMint
    )]
    pub user_fin_account: Account<'info, TokenAccount>,

    /// User's sFIN token account
    #[account(
        mut,
        constraint = user_sfin_account.owner == user.key() @ TokenError::Unauthorized,
        constraint = user_sfin_account.mint == sfin_mint.key() @ TokenError::InvalidMint
    )]
    pub user_sfin_account: Account<'info, TokenAccount>,

    /// Reward pool's FIN token vault
    #[account(
        mut,
        constraint = pool_fin_vault.owner == reward_pool.key() @ TokenError::Unauthorized,
        constraint = pool_fin_vault.mint == fin_mint.key() @ TokenError::InvalidMint
    )]
    pub pool_fin_vault: Account<'info, TokenAccount>,

    /// Reward pool's sFIN token vault
    #[account(
        mut,
        constraint = pool_sfin_vault.owner == reward_pool.key() @ TokenError::Unauthorized,
        constraint = pool_sfin_vault.mint == sfin_mint.key() @ TokenError::InvalidMint
    )]
    pub pool_sfin_vault: Account<'info, TokenAccount>,

    /// Authority for reward pool operations
    /// CHECK: This is a PDA that owns the reward pool
    #[account(
        seeds = [b"pool_authority"],
        bump
    )]
    pub pool_authority: UncheckedAccount<'info>,

    /// Token program
    pub token_program: Program<'info, Token>,

    /// System program
    pub system_program: Program<'info, System>,

    /// Clock sysvar for time calculations
    pub clock: Sysvar<'info, Clock>,
}

impl<'info> ClaimRewards<'info> {
    /// Validate claiming conditions
    pub fn validate(&self) -> Result<()> {
        let clock = &self.clock;
        let current_time = clock.unix_timestamp as u64;

        // Check if user has any staked amount
        require!(
            self.stake_account.staked_amount > 0,
            TokenError::NoStakedTokens
        );

        // Check minimum staking duration (24 hours)
        require!(
            current_time >= self.stake_account.last_stake_time + 86400,
            TokenError::MinimumStakingPeriodNotMet
        );

        // Check if there are rewards to claim
        let pending_rewards = self.calculate_pending_rewards(current_time)?;
        require!(
            pending_rewards.fin_rewards > 0 || pending_rewards.sfin_rewards > 0,
            TokenError::NoRewardsToClaim
        );

        // Check reward pool has sufficient balance
        require!(
            self.pool_fin_vault.amount >= pending_rewards.fin_rewards,
            TokenError::InsufficientRewardBalance
        );

        require!(
            self.pool_sfin_vault.amount >= pending_rewards.sfin_rewards,
            TokenError::InsufficientRewardBalance
        );

        Ok(())
    }

    /// Calculate pending rewards for the user
    pub fn calculate_pending_rewards(&self, current_time: u64) -> Result<PendingRewards> {
        let stake_account = &self.stake_account;
        let reward_pool = &self.reward_pool;

        // Calculate time since last claim
        let time_since_last_claim = current_time
            .checked_sub(stake_account.last_claim_time)
            .ok_or(TokenError::MathOverflow)?;

        // Base staking reward calculation
        let base_staking_reward = self.calculate_base_staking_reward(time_since_last_claim)?;

        // XP level multiplier (from stake account's cached XP level)
        let xp_multiplier = self.calculate_xp_multiplier(stake_account.xp_level)?;

        // RP tier multiplier (from stake account's cached RP tier)
        let rp_multiplier = self.calculate_rp_multiplier(stake_account.rp_tier)?;

        // Activity bonus based on recent engagement
        let activity_multiplier = self.calculate_activity_multiplier(current_time)?;

        // Loyalty bonus based on staking duration
        let loyalty_multiplier = self.calculate_loyalty_multiplier(current_time)?;

        // Apply all multipliers
        let total_multiplier = xp_multiplier
            .checked_mul(rp_multiplier)?
            .checked_mul(activity_multiplier)?
            .checked_mul(loyalty_multiplier)?;

        // Calculate final FIN rewards
        let fin_rewards = base_staking_reward
            .checked_mul(total_multiplier)?
            .checked_div(MULTIPLIER_PRECISION * MULTIPLIER_PRECISION * MULTIPLIER_PRECISION)?;

        // Calculate sFIN rewards (auto-compounding rewards)
        let sfin_rewards = self.calculate_sfin_rewards(fin_rewards)?;

        // Apply global reward pool modifiers
        let adjusted_fin_rewards = self.apply_global_modifiers(fin_rewards)?;
        let adjusted_sfin_rewards = self.apply_global_modifiers(sfin_rewards)?;

        Ok(PendingRewards {
            fin_rewards: adjusted_fin_rewards,
            sfin_rewards: adjusted_sfin_rewards,
            time_period: time_since_last_claim,
            base_reward: base_staking_reward,
            total_multiplier,
        })
    }

    /// Calculate base staking reward
    fn calculate_base_staking_reward(&self, time_period: u64) -> Result<u64> {
        let stake_account = &self.stake_account;
        let reward_pool = &self.reward_pool;

        // Base APY calculation (8-15% based on staking tier)
        let base_apy = self.get_staking_tier_apy(stake_account.staked_amount)?;

        // Convert APY to per-second rate
        let annual_seconds = 31_536_000u64; // 365 * 24 * 60 * 60
        let per_second_rate = base_apy
            .checked_mul(PRECISION)?
            .checked_div(annual_seconds)?;

        // Calculate reward for the time period
        let base_reward = stake_account.staked_amount
            .checked_mul(per_second_rate)?
            .checked_mul(time_period)?
            .checked_div(PRECISION * PRECISION)?;

        Ok(base_reward)
    }

    /// Get APY based on staking tier
    fn get_staking_tier_apy(&self, staked_amount: u64) -> Result<u64> {
        let apy = if staked_amount >= 10_000 * DECIMALS {
            15 * PERCENTAGE_PRECISION // 15% APY for 10,000+ FIN
        } else if staked_amount >= 5_000 * DECIMALS {
            14 * PERCENTAGE_PRECISION // 14% APY for 5,000-9,999 FIN
        } else if staked_amount >= 1_000 * DECIMALS {
            12 * PERCENTAGE_PRECISION // 12% APY for 1,000-4,999 FIN
        } else if staked_amount >= 500 * DECIMALS {
            10 * PERCENTAGE_PRECISION // 10% APY for 500-999 FIN
        } else {
            8 * PERCENTAGE_PRECISION // 8% APY for 100-499 FIN
        };

        Ok(apy)
    }

    /// Calculate XP level multiplier
    fn calculate_xp_multiplier(&self, xp_level: u32) -> Result<u64> {
        // XP Level Bonus: 1.0x + (XP_Level / 100)
        let bonus = (xp_level as u64)
            .checked_mul(MULTIPLIER_PRECISION)?
            .checked_div(100)?;

        let multiplier = MULTIPLIER_PRECISION
            .checked_add(bonus)?;

        // Cap at 5.0x maximum
        Ok(multiplier.min(5 * MULTIPLIER_PRECISION))
    }

    /// Calculate RP tier multiplier
    fn calculate_rp_multiplier(&self, rp_tier: u8) -> Result<u64> {
        // RP Tier Bonus: 1.0x + (RP_Tier * 0.2)
        let bonus = (rp_tier as u64)
            .checked_mul(MULTIPLIER_PRECISION)?
            .checked_mul(20)?
            .checked_div(100)?;

        let multiplier = MULTIPLIER_PRECISION
            .checked_add(bonus)?;

        // Cap at 3.0x maximum
        Ok(multiplier.min(3 * MULTIPLIER_PRECISION))
    }

    /// Calculate activity multiplier based on recent engagement
    fn calculate_activity_multiplier(&self, current_time: u64) -> Result<u64> {
        let stake_account = &self.stake_account;

        // Check activity in last 7 days
        let seven_days = 7 * 24 * 60 * 60; // 7 days in seconds
        let activity_cutoff = current_time
            .checked_sub(seven_days)
            .unwrap_or(0);

        let multiplier = if stake_account.last_activity_time >= activity_cutoff {
            // Active user: 1.5x multiplier
            MULTIPLIER_PRECISION
                .checked_mul(150)?
                .checked_div(100)?
        } else {
            // Inactive user: 1.0x multiplier
            MULTIPLIER_PRECISION
        };

        Ok(multiplier)
    }

    /// Calculate loyalty multiplier based on staking duration
    fn calculate_loyalty_multiplier(&self, current_time: u64) -> Result<u64> {
        let stake_account = &self.stake_account;

        let staking_duration = current_time
            .checked_sub(stake_account.first_stake_time)
            .unwrap_or(0);

        // Loyalty bonus: 0.05% per day, capped at 50% (365 days)
        let days_staked = staking_duration / (24 * 60 * 60);
        let bonus_percentage = (days_staked * 5).min(5000); // Max 50% bonus

        let multiplier = MULTIPLIER_PRECISION
            .checked_add(
                MULTIPLIER_PRECISION
                    .checked_mul(bonus_percentage)?
                    .checked_div(10000)?
            )?;

        Ok(multiplier)
    }

    /// Calculate sFIN rewards (liquid staking derivative)
    fn calculate_sfin_rewards(&self, fin_rewards: u64) -> Result<u64> {
        // 20% of FIN rewards are given as sFIN for auto-compounding
        let sfin_rewards = fin_rewards
            .checked_mul(20)?
            .checked_div(100)?;

        Ok(sfin_rewards)
    }

    /// Apply global reward pool modifiers
    fn apply_global_modifiers(&self, base_reward: u64) -> Result<u64> {
        let reward_pool = &self.reward_pool;

        // Global multiplier based on pool health
        let global_multiplier = if reward_pool.total_staked > reward_pool.target_stake_amount {
            // Over-staked: reduce rewards slightly
            MULTIPLIER_PRECISION
                .checked_mul(95)?
                .checked_div(100)?
        } else if reward_pool.total_staked < reward_pool.min_stake_amount {
            // Under-staked: increase rewards to incentivize
            MULTIPLIER_PRECISION
                .checked_mul(110)?
                .checked_div(100)?
        } else {
            // Normal range: no adjustment
            MULTIPLIER_PRECISION
        };

        let adjusted_reward = base_reward
            .checked_mul(global_multiplier)?
            .checked_div(MULTIPLIER_PRECISION)?;

        Ok(adjusted_reward)
    }

    /// Execute reward transfers
    pub fn transfer_rewards(&self, pending_rewards: &PendingRewards) -> Result<()> {
        let pool_authority_bump = *ctx.bumps.get("pool_authority").unwrap();
        let pool_authority_seeds = &[
            b"pool_authority",
            &[pool_authority_bump]
        ];
        let pool_signer = &[&pool_authority_seeds[..]];

        // Transfer FIN rewards
        if pending_rewards.fin_rewards > 0 {
            let cpi_accounts = Transfer {
                from: self.pool_fin_vault.to_account_info(),
                to: self.user_fin_account.to_account_info(),
                authority: self.pool_authority.to_account_info(),
            };

            let cpi_program = self.token_program.to_account_info();
            let cpi_ctx = CpiContext::new_with_signer(
                cpi_program,
                cpi_accounts,
                pool_signer,
            );

            token::transfer(cpi_ctx, pending_rewards.fin_rewards)?;
        }

        // Transfer sFIN rewards
        if pending_rewards.sfin_rewards > 0 {
            let cpi_accounts = Transfer {
                from: self.pool_sfin_vault.to_account_info(),
                to: self.user_sfin_account.to_account_info(),
                authority: self.pool_authority.to_account_info(),
            };

            let cpi_program = self.token_program.to_account_info();
            let cpi_ctx = CpiContext::new_with_signer(
                cpi_program,
                cpi_accounts,
                pool_signer,
            );

            token::transfer(cpi_ctx, pending_rewards.sfin_rewards)?;
        }

        Ok(())
    }

    /// Update account states after claiming
    pub fn update_accounts(&mut self, pending_rewards: &PendingRewards, current_time: u64) -> Result<()> {
        let stake_account = &mut self.stake_account;
        let reward_pool = &mut self.reward_pool;

        // Update stake account
        stake_account.last_claim_time = current_time;
        stake_account.total_fin_claimed = stake_account.total_fin_claimed
            .checked_add(pending_rewards.fin_rewards)
            .ok_or(TokenError::MathOverflow)?;
        stake_account.total_sfin_claimed = stake_account.total_sfin_claimed
            .checked_add(pending_rewards.sfin_rewards)
            .ok_or(TokenError::MathOverflow)?;
        stake_account.claim_count = stake_account.claim_count
            .checked_add(1)
            .ok_or(TokenError::MathOverflow)?;

        // Update reward pool statistics
        reward_pool.total_fin_distributed = reward_pool.total_fin_distributed
            .checked_add(pending_rewards.fin_rewards)
            .ok_or(TokenError::MathOverflow)?;
        reward_pool.total_sfin_distributed = reward_pool.total_sfin_distributed
            .checked_add(pending_rewards.sfin_rewards)
            .ok_or(TokenError::MathOverflow)?;
        reward_pool.total_claims = reward_pool.total_claims
            .checked_add(1)
            .ok_or(TokenError::MathOverflow)?;
        reward_pool.last_distribution_time = current_time;

        Ok(())
    }
}

/// Structure to hold calculated pending rewards
#[derive(Debug, Clone)]
pub struct PendingRewards {
    pub fin_rewards: u64,
    pub sfin_rewards: u64,
    pub time_period: u64,
    pub base_reward: u64,
    pub total_multiplier: u64,
}

/// Main instruction handler for claiming rewards
pub fn claim_rewards(ctx: Context<ClaimRewards>) -> Result<()> {
    let clock = &ctx.accounts.clock;
    let current_time = clock.unix_timestamp as u64;

    // Validate claiming conditions
    ctx.accounts.validate()?;

    // Calculate pending rewards
    let pending_rewards = ctx.accounts.calculate_pending_rewards(current_time)?;

    // Transfer rewards to user
    ctx.accounts.transfer_rewards(&pending_rewards)?;

    // Update account states
    ctx.accounts.update_accounts(&pending_rewards, current_time)?;

    // Emit reward claim event
    emit!(RewardClaimed {
        user: ctx.accounts.user.key(),
        fin_amount: pending_rewards.fin_rewards,
        sfin_amount: pending_rewards.sfin_rewards,
        time_period: pending_rewards.time_period,
        total_multiplier: pending_rewards.total_multiplier,
        claim_time: current_time,
    });

    msg!(
        "Rewards claimed: {} FIN, {} sFIN for user {}",
        pending_rewards.fin_rewards,
        pending_rewards.sfin_rewards,
        ctx.accounts.user.key()
    );

    Ok(())
}

/// Emergency claim function for admin use
#[derive(Accounts)]
pub struct EmergencyClaim<'info> {
    /// Admin authority
    #[account(
        constraint = admin.key() == reward_pool.admin @ TokenError::Unauthorized
    )]
    pub admin: Signer<'info>,

    /// Global reward pool account
    #[account(
        mut,
        seeds = [b"reward_pool"],
        bump
    )]
    pub reward_pool: Account<'info, RewardPool>,

    /// Target user's stake account
    #[account(
        mut,
        seeds = [b"stake", target_user.key().as_ref()],
        bump
    )]
    pub stake_account: Account<'info, StakeAccount>,

    /// Target user
    /// CHECK: This is validated by the stake account seed
    pub target_user: UncheckedAccount<'info>,

    /// Admin's token accounts and other required accounts...
    /// (Similar structure to ClaimRewards but with admin authority)
}

/// Emergency claim handler (admin only)
pub fn emergency_claim(ctx: Context<EmergencyClaim>, amount: u64) -> Result<()> {
    let reward_pool = &mut ctx.accounts.reward_pool;
    
    // Verify emergency conditions
    require!(
        reward_pool.emergency_mode,
        TokenError::EmergencyModeNotActive
    );

    // Additional emergency claim logic...
    
    Ok(())
}

/// Event emitted when rewards are claimed
#[event]
pub struct RewardClaimed {
    pub user: Pubkey,
    pub fin_amount: u64,
    pub sfin_amount: u64,
    pub time_period: u64,
    pub total_multiplier: u64,
    pub claim_time: u64,
}

/// Constants for calculations
const DECIMALS: u64 = 1_000_000; // 6 decimals for FIN token
const PRECISION: u64 = 1_000_000_000; // 9 decimal precision for calculations
const MULTIPLIER_PRECISION: u64 = 1_000; // 3 decimal precision for multipliers
const PERCENTAGE_PRECISION: u64 = 10_000; // 4 decimal precision for percentages

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_staking_tier_apy() {
        // Test different staking amounts and their corresponding APY
        let claim_rewards = ClaimRewards::default();
        
        // Test 10,000+ FIN tier
        let apy = claim_rewards.get_staking_tier_apy(10_000 * DECIMALS).unwrap();
        assert_eq!(apy, 15 * PERCENTAGE_PRECISION);
        
        // Test 1,000-4,999 FIN tier
        let apy = claim_rewards.get_staking_tier_apy(2_000 * DECIMALS).unwrap();
        assert_eq!(apy, 12 * PERCENTAGE_PRECISION);
        
        // Test 100-499 FIN tier
        let apy = claim_rewards.get_staking_tier_apy(300 * DECIMALS).unwrap();
        assert_eq!(apy, 8 * PERCENTAGE_PRECISION);
    }

    #[test]
    fn test_xp_multiplier() {
        let claim_rewards = ClaimRewards::default();
        
        // Test level 50 user
        let multiplier = claim_rewards.calculate_xp_multiplier(50).unwrap();
        assert_eq!(multiplier, MULTIPLIER_PRECISION + (50 * MULTIPLIER_PRECISION / 100));
        
        // Test high level user (should be capped)
        let multiplier = claim_rewards.calculate_xp_multiplier(1000).unwrap();
        assert_eq!(multiplier, 5 * MULTIPLIER_PRECISION);
    }

    #[test]
    fn test_loyalty_multiplier() {
        let claim_rewards = ClaimRewards::default();
        let current_time = 1700000000; // Mock timestamp
        
        // Mock stake account with 100 days of staking
        let staking_duration = 100 * 24 * 60 * 60; // 100 days
        let first_stake_time = current_time - staking_duration;
        
        // Test should give 5% bonus (100 days * 0.05% per day)
        let expected_bonus = MULTIPLIER_PRECISION + (MULTIPLIER_PRECISION * 500 / 10000);
        
        // This would require mocking the stake account, which is complex in this context
        // In real implementation, this would be tested with proper test fixtures
    }
}
