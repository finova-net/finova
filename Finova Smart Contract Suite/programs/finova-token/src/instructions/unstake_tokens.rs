// programs/finova-token/src/instructions/unstake_tokens.rs

use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount, Transfer};
use crate::constants::*;
use crate::errors::*;
use crate::state::*;
use crate::utils::*;

/// Unstake tokens instruction with comprehensive reward calculation and security measures
#[derive(Accounts)]
#[instruction(amount: u64)]
pub struct UnstakeTokens<'info> {
    /// User's wallet - must be signer and fee payer
    #[account(mut)]
    pub user: Signer<'info>,
    
    /// User's stake account containing staking information
    #[account(
        mut,
        seeds = [STAKE_ACCOUNT_SEED, user.key().as_ref()],
        bump = stake_account.bump,
        has_one = user @ FinovaTokenError::InvalidUser,
        constraint = stake_account.staked_amount >= amount @ FinovaTokenError::InsufficientStakedTokens,
        constraint = !stake_account.emergency_locked @ FinovaTokenError::EmergencyLocked
    )]
    pub stake_account: Account<'info, StakeAccount>,
    
    /// $FIN token mint account
    #[account(
        constraint = mint.key() == FIN_MINT_PUBKEY @ FinovaTokenError::InvalidTokenMint
    )]
    pub mint: Account<'info, Mint>,
    
    /// $sFIN (staked FIN) token mint account - liquid staking derivative
    #[account(
        mut,
        constraint = sfin_mint.key() == SFIN_MINT_PUBKEY @ FinovaTokenError::InvalidStakedTokenMint
    )]
    pub sfin_mint: Account<'info, Mint>,
    
    /// User's $FIN token account (destination for unstaked tokens)
    #[account(
        mut,
        constraint = user_token_account.owner == user.key() @ FinovaTokenError::InvalidTokenAccount,
        constraint = user_token_account.mint == mint.key() @ FinovaTokenError::InvalidTokenAccount
    )]
    pub user_token_account: Account<'info, TokenAccount>,
    
    /// User's $sFIN token account (source for burning staked tokens)
    #[account(
        mut,
        constraint = user_sfin_account.owner == user.key() @ FinovaTokenError::InvalidTokenAccount,
        constraint = user_sfin_account.mint == sfin_mint.key() @ FinovaTokenError::InvalidTokenAccount,
        constraint = user_sfin_account.amount >= calculate_sfin_amount(amount, &stake_account)? @ FinovaTokenError::InsufficientStakedTokens
    )]
    pub user_sfin_account: Account<'info, TokenAccount>,
    
    /// Stake vault - holds staked $FIN tokens
    #[account(
        mut,
        seeds = [STAKE_VAULT_SEED],
        bump,
        constraint = stake_vault.mint == mint.key() @ FinovaTokenError::InvalidVault,
        constraint = stake_vault.amount >= amount @ FinovaTokenError::InsufficientVaultBalance
    )]
    pub stake_vault: Account<'info, TokenAccount>,
    
    /// Reward pool for calculating and distributing staking rewards
    #[account(
        mut,
        seeds = [REWARD_POOL_SEED],
        bump = reward_pool.bump,
        constraint = !reward_pool.paused @ FinovaTokenError::RewardPoolPaused
    )]
    pub reward_pool: Account<'info, RewardPool>,
    
    /// System programs
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub clock: Sysvar<'info, Clock>,
}

impl<'info> UnstakeTokens<'info> {
    /// Calculate comprehensive staking rewards based on multiple factors
    pub fn calculate_comprehensive_rewards(&self) -> Result<u64> {
        let clock = &self.clock;
        let current_time = clock.unix_timestamp;
        let stake_duration = current_time
            .checked_sub(self.stake_account.stake_timestamp)
            .ok_or(FinovaTokenError::InvalidTimestamp)?;
        
        // Base reward calculation using exponential staking formula
        let base_reward = self.calculate_base_staking_reward(stake_duration)?;
        
        // XP level multiplier from integrated reward system
        let xp_multiplier = self.calculate_xp_multiplier()?;
        
        // RP (Referral Points) tier bonus
        let rp_multiplier = self.calculate_rp_multiplier()?;
        
        // Loyalty bonus based on staking duration (increases over time)
        let loyalty_bonus = self.calculate_loyalty_bonus(stake_duration)?;
        
        // Activity bonus based on recent user engagement
        let activity_bonus = self.calculate_activity_bonus()?;
        
        // Network participation bonus (guild membership, governance, etc.)
        let network_bonus = self.calculate_network_participation_bonus()?;
        
        // Anti-whale regression factor to prevent centralization
        let regression_factor = self.calculate_anti_whale_regression()?;
        
        // Final reward calculation with all multipliers
        let total_multiplier = xp_multiplier
            .checked_mul(rp_multiplier)?
            .checked_mul(loyalty_bonus)?
            .checked_mul(activity_bonus)?
            .checked_mul(network_bonus)?
            .checked_mul(regression_factor)?
            .checked_div(MULTIPLIER_PRECISION.pow(5))?; // Normalize for precision
        
        let final_reward = base_reward
            .checked_mul(total_multiplier)?
            .checked_div(MULTIPLIER_PRECISION)?;
        
        // Cap rewards to prevent overflow and maintain economic balance
        let max_reward = self.stake_account.staked_amount
            .checked_mul(MAX_REWARD_PERCENTAGE)?
            .checked_div(100)?;
        
        Ok(std::cmp::min(final_reward, max_reward))
    }
    
    /// Calculate base staking reward using time-weighted formula
    fn calculate_base_staking_reward(&self, stake_duration: i64) -> Result<u64> {
        if stake_duration < MIN_STAKE_DURATION {
            return Ok(0); // No rewards for very short staking periods
        }
        
        // Convert duration to days for calculation
        let days_staked = stake_duration.checked_div(SECONDS_PER_DAY)? as u64;
        
        // Base APY calculation with compound interest
        let annual_rate = self.get_current_apy()?;
        let daily_rate = annual_rate.checked_div(365)?;
        
        // Compound interest formula: A = P(1 + r)^t
        let compound_factor = calculate_compound_interest(
            MULTIPLIER_PRECISION,
            daily_rate,
            days_staked,
        )?;
        
        let base_reward = self.stake_account.staked_amount
            .checked_mul(compound_factor)?
            .checked_div(MULTIPLIER_PRECISION)?
            .checked_sub(self.stake_account.staked_amount)?;
        
        Ok(base_reward)
    }
    
    /// Get current APY based on total staked amount and network conditions
    fn get_current_apy(&self) -> Result<u64> {
        let total_staked = self.reward_pool.total_staked;
        let target_staked = self.reward_pool.target_stake_amount;
        
        // Dynamic APY based on staking ratio
        let staking_ratio = if target_staked > 0 {
            total_staked.checked_mul(MULTIPLIER_PRECISION)?.checked_div(target_staked)?
        } else {
            MULTIPLIER_PRECISION
        };
        
        // Higher APY when less tokens are staked (incentivize staking)
        // Lower APY when more tokens are staked (maintain sustainability)
        let base_apy = if staking_ratio < MULTIPLIER_PRECISION / 2 {
            MAX_APY // 15%
        } else if staking_ratio < MULTIPLIER_PRECISION {
            BASE_APY // 10%
        } else {
            MIN_APY // 8%
        };
        
        Ok(base_apy)
    }
    
    /// Calculate XP level multiplier based on user's experience points
    fn calculate_xp_multiplier(&self) -> Result<u64> {
        let xp_level = self.stake_account.user_xp_level;
        
        // XP multiplier: 1.0x + (XP_Level / 100)
        // Level 100 = 2.0x multiplier, Level 50 = 1.5x, etc.
        let xp_bonus = xp_level.checked_mul(MULTIPLIER_PRECISION)?.checked_div(100)?;
        let multiplier = MULTIPLIER_PRECISION.checked_add(xp_bonus)?;
        
        // Cap at maximum XP multiplier
        Ok(std::cmp::min(multiplier, MAX_XP_MULTIPLIER))
    }
    
    /// Calculate RP tier multiplier based on referral network
    fn calculate_rp_multiplier(&self) -> Result<u64> {
        let rp_tier = self.stake_account.user_rp_tier;
        
        // RP multiplier based on tier: 1.0x + (Tier × 0.2)
        // Ambassador (Tier 5) = 2.0x, Leader (Tier 4) = 1.8x, etc.
        let rp_bonus = rp_tier.checked_mul(20)?.checked_mul(MULTIPLIER_PRECISION)?.checked_div(100)?;
        let multiplier = MULTIPLIER_PRECISION.checked_add(rp_bonus)?;
        
        Ok(std::cmp::min(multiplier, MAX_RP_MULTIPLIER))
    }
    
    /// Calculate loyalty bonus based on staking duration
    fn calculate_loyalty_bonus(&self, stake_duration: i64) -> Result<u64> {
        let months_staked = stake_duration.checked_div(SECONDS_PER_MONTH)? as u64;
        
        // Loyalty bonus: 1.0x + (Months × 0.05), capped at 3.0x
        let loyalty_bonus = months_staked.checked_mul(5)?.checked_mul(MULTIPLIER_PRECISION)?.checked_div(100)?;
        let multiplier = MULTIPLIER_PRECISION.checked_add(loyalty_bonus)?;
        
        Ok(std::cmp::min(multiplier, MAX_LOYALTY_MULTIPLIER))
    }
    
    /// Calculate activity bonus based on recent user engagement
    fn calculate_activity_bonus(&self) -> Result<u64> {
        let activity_score = self.stake_account.recent_activity_score;
        
        // Activity bonus: 1.0x + (Activity_Score × 0.1), max 2.0x
        let activity_bonus = activity_score.checked_mul(MULTIPLIER_PRECISION)?.checked_div(10)?;
        let multiplier = MULTIPLIER_PRECISION.checked_add(activity_bonus)?;
        
        Ok(std::cmp::min(multiplier, MAX_ACTIVITY_MULTIPLIER))
    }
    
    /// Calculate network participation bonus (guilds, governance, etc.)
    fn calculate_network_participation_bonus(&self) -> Result<u64> {
        let mut bonus = 0u64;
        
        // Guild membership bonus
        if self.stake_account.is_guild_member {
            bonus = bonus.checked_add(GUILD_MEMBER_BONUS)?;
        }
        
        // Guild leadership bonus
        if self.stake_account.is_guild_leader {
            bonus = bonus.checked_add(GUILD_LEADER_BONUS)?;
        }
        
        // Governance participation bonus
        if self.stake_account.governance_participation_count > 10 {
            bonus = bonus.checked_add(GOVERNANCE_BONUS)?;
        }
        
        // DAO voting bonus
        if self.stake_account.dao_voting_power > 0 {
            bonus = bonus.checked_add(DAO_VOTING_BONUS)?;
        }
        
        let multiplier = MULTIPLIER_PRECISION.checked_add(bonus)?;
        Ok(std::cmp::min(multiplier, MAX_NETWORK_MULTIPLIER))
    }
    
    /// Calculate anti-whale regression to prevent centralization
    fn calculate_anti_whale_regression(&self) -> Result<u64> {
        let user_holdings = self.stake_account.total_token_holdings;
        
        // Exponential regression: e^(-0.001 × Holdings)
        // Large holders get reduced multipliers to prevent centralization
        let regression_factor = if user_holdings > WHALE_THRESHOLD {
            let excess_holdings = user_holdings.checked_sub(WHALE_THRESHOLD)?;
            let regression_power = excess_holdings.checked_div(REGRESSION_DIVISOR)?;
            
            // Approximate e^(-x) using Taylor series for on-chain calculation
            calculate_exponential_decay(regression_power)?
        } else {
            MULTIPLIER_PRECISION // No regression for normal holders
        };
        
        Ok(std::cmp::max(regression_factor, MIN_REGRESSION_FACTOR))
    }
    
    /// Execute the unstaking process with comprehensive validation
    pub fn process_unstaking(&mut self, amount: u64) -> Result<()> {
        let clock = &self.clock;
        let current_time = clock.unix_timestamp;
        
        // Validate minimum staking period
        let stake_duration = current_time
            .checked_sub(self.stake_account.stake_timestamp)
            .ok_or(FinovaTokenError::InvalidTimestamp)?;
        
        require!(
            stake_duration >= MIN_STAKE_DURATION,
            FinovaTokenError::MinimumStakeDurationNotMet
        );
        
        // Check for emergency lock conditions
        require!(
            !self.stake_account.emergency_locked,
            FinovaTokenError::EmergencyLocked
        );
        
        // Validate unstaking amount
        require!(
            amount >= MIN_UNSTAKE_AMOUNT,
            FinovaTokenError::AmountTooSmall
        );
        
        require!(
            amount <= self.stake_account.staked_amount,
            FinovaTokenError::InsufficientStakedTokens
        );
        
        // Calculate comprehensive rewards
        let total_rewards = self.calculate_comprehensive_rewards()?;
        
        // Calculate $sFIN tokens to burn
        let sfin_to_burn = calculate_sfin_amount(amount, &self.stake_account)?;
        
        // Update stake account state
        self.stake_account.staked_amount = self.stake_account.staked_amount
            .checked_sub(amount)?;
        self.stake_account.total_rewards_earned = self.stake_account.total_rewards_earned
            .checked_add(total_rewards)?;
        self.stake_account.last_unstake_timestamp = current_time;
        self.stake_account.unstake_count = self.stake_account.unstake_count
            .checked_add(1)?;
        
        // Update reward pool statistics
        self.reward_pool.total_staked = self.reward_pool.total_staked
            .checked_sub(amount)?;
        self.reward_pool.total_rewards_distributed = self.reward_pool.total_rewards_distributed
            .checked_add(total_rewards)?;
        
        // Transfer unstaked tokens from vault to user
        let total_withdrawal = amount.checked_add(total_rewards)?;
        
        let vault_authority_seeds = &[
            STAKE_VAULT_AUTHORITY_SEED,
            &[self.reward_pool.vault_authority_bump],
        ];
        let signer_seeds = &[&vault_authority_seeds[..]];
        
        let transfer_ctx = CpiContext::new_with_signer(
            self.token_program.to_account_info(),
            Transfer {
                from: self.stake_vault.to_account_info(),
                to: self.user_token_account.to_account_info(),
                authority: self.reward_pool.to_account_info(),
            },
            signer_seeds,
        );
        
        token::transfer(transfer_ctx, total_withdrawal)?;
        
        // Burn corresponding $sFIN tokens
        let burn_ctx = CpiContext::new(
            self.token_program.to_account_info(),
            token::Burn {
                mint: self.sfin_mint.to_account_info(),
                from: self.user_sfin_account.to_account_info(),
                authority: self.user.to_account_info(),
            },
        );
        
        token::burn(burn_ctx, sfin_to_burn)?;
        
        // Apply cooldown period for rapid unstaking prevention
        if self.stake_account.unstake_count > MAX_RAPID_UNSTAKES {
            self.stake_account.cooldown_until = current_time
                .checked_add(UNSTAKE_COOLDOWN_PERIOD)?;
        }
        
        // Update user tier if stake amount falls below threshold
        self.update_user_staking_tier()?;
        
        // Emit unstaking event for off-chain tracking
        emit!(UnstakeEvent {
            user: self.user.key(),
            amount_unstaked: amount,
            rewards_earned: total_rewards,
            sfin_burned: sfin_to_burn,
            new_staked_balance: self.stake_account.staked_amount,
            timestamp: current_time,
        });
        
        msg!(
            "Successfully unstaked {} $FIN with {} rewards for user {}",
            amount,
            total_rewards,
            self.user.key()
        );
        
        Ok(())
    }
    
    /// Update user's staking tier based on remaining staked amount
    fn update_user_staking_tier(&mut self) -> Result<()> {
        let staked_amount = self.stake_account.staked_amount;
        
        let new_tier = if staked_amount >= DIAMOND_TIER_THRESHOLD {
            StakingTier::Diamond
        } else if staked_amount >= PLATINUM_TIER_THRESHOLD {
            StakingTier::Platinum
        } else if staked_amount >= GOLD_TIER_THRESHOLD {
            StakingTier::Gold
        } else if staked_amount >= SILVER_TIER_THRESHOLD {
            StakingTier::Silver
        } else {
            StakingTier::Bronze
        };
        
        self.stake_account.staking_tier = new_tier;
        Ok(())
    }
}

/// Main unstaking instruction handler
pub fn handler(ctx: Context<UnstakeTokens>, amount: u64) -> Result<()> {
    let unstake_accounts = &mut ctx.accounts;
    
    // Comprehensive security checks
    require!(
        amount > 0,
        FinovaTokenError::AmountMustBePositive
    );
    
    require!(
        !unstake_accounts.reward_pool.emergency_pause,
        FinovaTokenError::EmergencyPaused
    );
    
    // Check cooldown period
    let current_time = unstake_accounts.clock.unix_timestamp;
    require!(
        current_time >= unstake_accounts.stake_account.cooldown_until,
        FinovaTokenError::CooldownPeriodActive
    );
    
    // Process the unstaking with all calculations and validations
    unstake_accounts.process_unstaking(amount)?;
    
    Ok(())
}

/// Emergency unstaking handler for critical situations
pub fn emergency_unstake_handler(ctx: Context<UnstakeTokens>, amount: u64) -> Result<()> {
    let unstake_accounts = &mut ctx.accounts;
    
    // Only allow emergency unstaking if authorized
    require!(
        unstake_accounts.reward_pool.emergency_unstake_enabled,
        FinovaTokenError::EmergencyUnstakingDisabled
    );
    
    // Emergency unstaking with reduced rewards (penalty applied)
    let penalty_rate = EMERGENCY_UNSTAKE_PENALTY; // 10% penalty
    let penalty_amount = amount.checked_mul(penalty_rate)?.checked_div(100)?;
    let net_amount = amount.checked_sub(penalty_amount)?;
    
    // Update stake account
    unstake_accounts.stake_account.staked_amount = unstake_accounts.stake_account.staked_amount
        .checked_sub(amount)?;
    unstake_accounts.stake_account.total_penalties_paid = unstake_accounts.stake_account.total_penalties_paid
        .checked_add(penalty_amount)?;
    
    // Transfer tokens with penalty applied
    let vault_authority_seeds = &[
        STAKE_VAULT_AUTHORITY_SEED,
        &[unstake_accounts.reward_pool.vault_authority_bump],
    ];
    let signer_seeds = &[&vault_authority_seeds[..]];
    
    let transfer_ctx = CpiContext::new_with_signer(
        unstake_accounts.token_program.to_account_info(),
        Transfer {
            from: unstake_accounts.stake_vault.to_account_info(),
            to: unstake_accounts.user_token_account.to_account_info(),
            authority: unstake_accounts.reward_pool.to_account_info(),
        },
        signer_seeds,
    );
    
    token::transfer(transfer_ctx, net_amount)?;
    
    // Burn corresponding $sFIN tokens
    let sfin_to_burn = calculate_sfin_amount(amount, &unstake_accounts.stake_account)?;
    let burn_ctx = CpiContext::new(
        unstake_accounts.token_program.to_account_info(),
        token::Burn {
            mint: unstake_accounts.sfin_mint.to_account_info(),
            from: unstake_accounts.user_sfin_account.to_account_info(),
            authority: unstake_accounts.user.to_account_info(),
        },
    );
    
    token::burn(burn_ctx, sfin_to_burn)?;
    
    emit!(EmergencyUnstakeEvent {
        user: unstake_accounts.user.key(),
        amount_unstaked: amount,
        penalty_applied: penalty_amount,
        net_received: net_amount,
        timestamp: unstake_accounts.clock.unix_timestamp,
    });
    
    Ok(())
}

/// Events for off-chain tracking and analytics
#[event]
pub struct UnstakeEvent {
    pub user: Pubkey,
    pub amount_unstaked: u64,
    pub rewards_earned: u64,
    pub sfin_burned: u64,
    pub new_staked_balance: u64,
    pub timestamp: i64,
}

#[event]
pub struct EmergencyUnstakeEvent {
    pub user: Pubkey,
    pub amount_unstaked: u64,
    pub penalty_applied: u64,
    pub net_received: u64,
    pub timestamp: i64,
}
