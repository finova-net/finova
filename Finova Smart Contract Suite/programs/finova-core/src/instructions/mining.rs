// programs/finova-core/src/instructions/mining.rs

use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount, Transfer};
use crate::constants::*;
use crate::errors::*;
use crate::state::*;
use crate::utils::*;

/// Initialize mining for a user
#[derive(Accounts)]
pub struct InitializeMining<'info> {
    #[account(
        init,
        payer = user,
        space = MiningAccount::SPACE,
        seeds = [MINING_SEED, user.key().as_ref()],
        bump
    )]
    pub mining_account: Account<'info, MiningAccount>,
    
    #[account(
        mut,
        seeds = [USER_SEED, user.key().as_ref()],
        bump,
        constraint = user_account.owner == user.key() @ FinovaError::Unauthorized
    )]
    pub user_account: Account<'info, UserAccount>,
    
    #[account(
        seeds = [NETWORK_SEED],
        bump
    )]
    pub network_state: Account<'info, NetworkState>,
    
    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
}

/// Start mining session
#[derive(Accounts)]
pub struct StartMining<'info> {
    #[account(
        mut,
        seeds = [MINING_SEED, user.key().as_ref()],
        bump,
        constraint = mining_account.owner == user.key() @ FinovaError::Unauthorized
    )]
    pub mining_account: Account<'info, MiningAccount>,
    
    #[account(
        mut,
        seeds = [USER_SEED, user.key().as_ref()],
        bump,
        constraint = user_account.owner == user.key() @ FinovaError::Unauthorized
    )]
    pub user_account: Account<'info, UserAccount>,
    
    #[account(
        mut,
        seeds = [NETWORK_SEED],
        bump
    )]
    pub network_state: Account<'info, NetworkState>,
    
    #[account(mut)]
    pub user: Signer<'info>,
    pub clock: Sysvar<'info, Clock>,
}

/// Claim mining rewards
#[derive(Accounts)]
pub struct ClaimMiningRewards<'info> {
    #[account(
        mut,
        seeds = [MINING_SEED, user.key().as_ref()],
        bump,
        constraint = mining_account.owner == user.key() @ FinovaError::Unauthorized
    )]
    pub mining_account: Account<'info, MiningAccount>,
    
    #[account(
        mut,
        seeds = [USER_SEED, user.key().as_ref()],
        bump,
        constraint = user_account.owner == user.key() @ FinovaError::Unauthorized
    )]
    pub user_account: Account<'info, UserAccount>,
    
    #[account(
        mut,
        seeds = [NETWORK_SEED],
        bump
    )]
    pub network_state: Account<'info, NetworkState>,
    
    #[account(
        mut,
        seeds = [REWARDS_POOL_SEED],
        bump
    )]
    pub rewards_pool: Account<'info, RewardsPool>,
    
    #[account(mut)]
    pub fin_mint: Account<'info, Mint>,
    
    #[account(
        mut,
        constraint = user_token_account.owner == user.key() @ FinovaError::Unauthorized,
        constraint = user_token_account.mint == fin_mint.key() @ FinovaError::InvalidMint
    )]
    pub user_token_account: Account<'info, TokenAccount>,
    
    #[account(
        mut,
        constraint = pool_token_account.owner == rewards_pool.key() @ FinovaError::Unauthorized,
        constraint = pool_token_account.mint == fin_mint.key() @ FinovaError::InvalidMint
    )]
    pub pool_token_account: Account<'info, TokenAccount>,
    
    #[account(mut)]
    pub user: Signer<'info>,
    pub token_program: Program<'info, Token>,
    pub clock: Sysvar<'info, Clock>,
}

/// Update mining multipliers (special cards, boosts, etc.)
#[derive(Accounts)]
pub struct UpdateMiningMultipliers<'info> {
    #[account(
        mut,
        seeds = [MINING_SEED, user.key().as_ref()],
        bump,
        constraint = mining_account.owner == user.key() @ FinovaError::Unauthorized
    )]
    pub mining_account: Account<'info, MiningAccount>,
    
    #[account(
        seeds = [USER_SEED, user.key().as_ref()],
        bump,
        constraint = user_account.owner == user.key() @ FinovaError::Unauthorized
    )]
    pub user_account: Account<'info, UserAccount>,
    
    #[account(mut)]
    pub user: Signer<'info>,
    pub clock: Sysvar<'info, Clock>,
}

/// Apply special card mining boost
#[derive(Accounts)]
pub struct ApplySpecialCard<'info> {
    #[account(
        mut,
        seeds = [MINING_SEED, user.key().as_ref()],
        bump,
        constraint = mining_account.owner == user.key() @ FinovaError::Unauthorized
    )]
    pub mining_account: Account<'info, MiningAccount>,
    
    #[account(
        seeds = [USER_SEED, user.key().as_ref()],
        bump,
        constraint = user_account.owner == user.key() @ FinovaError::Unauthorized
    )]
    pub user_account: Account<'info, UserAccount>,
    
    #[account(mut)]
    pub user: Signer<'info>,
    pub clock: Sysvar<'info, Clock>,
}

/// Emergency pause mining (admin only)
#[derive(Accounts)]
pub struct PauseMining<'info> {
    #[account(
        mut,
        seeds = [NETWORK_SEED],
        bump,
        constraint = network_state.admin == admin.key() @ FinovaError::Unauthorized
    )]
    pub network_state: Account<'info, NetworkState>,
    
    #[account(mut)]
    pub admin: Signer<'info>,
}

// Mining instruction implementations
impl<'info> InitializeMining<'info> {
    pub fn process(&mut self, bumps: &InitializeMiningBumps) -> Result<()> {
        let clock = Clock::get()?;
        
        // Validate user is KYC verified for higher rates
        require!(
            self.user_account.is_initialized,
            FinovaError::UserNotInitialized
        );

        // Initialize mining account
        self.mining_account.set_inner(MiningAccount {
            owner: self.user.key(),
            base_rate: self.calculate_base_mining_rate(),
            current_multiplier: INITIAL_MINING_MULTIPLIER,
            last_claim_time: clock.unix_timestamp,
            total_mined: 0,
            mining_phase: self.get_current_mining_phase(),
            is_active: false,
            finizen_bonus: self.calculate_finizen_bonus(),
            referral_bonus: self.calculate_referral_bonus(),
            security_bonus: if self.user_account.kyc_verified { 
                SECURITY_BONUS_VERIFIED 
            } else { 
                SECURITY_BONUS_UNVERIFIED 
            },
            regression_factor: self.calculate_regression_factor(),
            special_card_boosts: Vec::new(),
            daily_mining_cap: self.calculate_daily_cap(),
            daily_mined_amount: 0,
            last_reset_day: get_current_day(clock.unix_timestamp),
            streak_bonus: INITIAL_STREAK_BONUS,
            quality_score_multiplier: INITIAL_QUALITY_MULTIPLIER,
            anti_bot_penalty: 0,
            bump: bumps.mining_account,
        });

        // Update network statistics
        self.network_state.total_miners += 1;
        self.network_state.total_active_users += 1;

        // Update user mining status
        self.user_account.mining_initialized = true;
        self.user_account.last_mining_activity = clock.unix_timestamp;

        msg!("Mining initialized for user: {}", self.user.key());
        
        Ok(())
    }

    fn calculate_base_mining_rate(&self) -> u64 {
        match self.get_current_mining_phase() {
            MiningPhase::Finizen => BASE_MINING_RATE_FINIZEN,
            MiningPhase::Growth => BASE_MINING_RATE_GROWTH,
            MiningPhase::Maturity => BASE_MINING_RATE_MATURITY,
            MiningPhase::Stability => BASE_MINING_RATE_STABILITY,
        }
    }

    fn get_current_mining_phase(&self) -> MiningPhase {
        let total_users = self.network_state.total_users;
        
        if total_users < FINIZEN_PHASE_LIMIT {
            MiningPhase::Finizen
        } else if total_users < GROWTH_PHASE_LIMIT {
            MiningPhase::Growth
        } else if total_users < MATURITY_PHASE_LIMIT {
            MiningPhase::Maturity
        } else {
            MiningPhase::Stability
        }
    }

    fn calculate_finizen_bonus(&self) -> u64 {
        let total_users = self.network_state.total_users;
        let bonus = FINIZEN_BONUS_MAX - (total_users * FINIZEN_BONUS_DECAY / FINIZEN_BONUS_DIVISOR);
        std::cmp::max(bonus, FINIZEN_BONUS_MIN)
    }

    fn calculate_referral_bonus(&self) -> u64 {
        let active_referrals = self.user_account.active_referrals;
        REFERRAL_BONUS_BASE + (active_referrals * REFERRAL_BONUS_PER_REFERRAL)
    }

    fn calculate_regression_factor(&self) -> u64 {
        let total_holdings = self.user_account.total_fin_holdings;
        let regression = ((-1.0 * REGRESSION_COEFFICIENT * total_holdings as f64).exp() * PRECISION as f64) as u64;
        std::cmp::max(regression, MIN_REGRESSION_FACTOR)
    }

    fn calculate_daily_cap(&self) -> u64 {
        let base_cap = match self.get_current_mining_phase() {
            MiningPhase::Finizen => DAILY_CAP_FINIZEN,
            MiningPhase::Growth => DAILY_CAP_GROWTH,
            MiningPhase::Maturity => DAILY_CAP_MATURITY,
            MiningPhase::Stability => DAILY_CAP_STABILITY,
        };

        // Apply user-specific multipliers
        let xp_multiplier = self.user_account.xp_level_multiplier();
        let staking_multiplier = if self.user_account.is_staking { 
            STAKING_MULTIPLIER 
        } else { 
            PRECISION 
        };

        base_cap * xp_multiplier * staking_multiplier / (PRECISION * PRECISION)
    }
}

impl<'info> StartMining<'info> {
    pub fn process(&mut self) -> Result<()> {
        let clock = Clock::get()?;
        
        // Validate mining account is initialized
        require!(
            self.mining_account.owner == self.user.key(),
            FinovaError::Unauthorized
        );

        // Check if mining is already active
        require!(
            !self.mining_account.is_active,
            FinovaError::MiningAlreadyActive
        );

        // Validate network is not paused
        require!(
            !self.network_state.mining_paused,
            FinovaError::MiningPaused
        );

        // Check anti-bot measures
        self.validate_human_behavior()?;

        // Update mining state
        self.mining_account.is_active = true;
        self.mining_account.last_claim_time = clock.unix_timestamp;
        
        // Update daily reset if needed
        let current_day = get_current_day(clock.unix_timestamp);
        if current_day != self.mining_account.last_reset_day {
            self.mining_account.daily_mined_amount = 0;
            self.mining_account.last_reset_day = current_day;
        }

        // Update user activity
        self.user_account.last_mining_activity = clock.unix_timestamp;
        self.user_account.daily_login_streak = self.calculate_streak(clock.unix_timestamp);

        // Update network statistics
        self.network_state.active_miners += 1;
        self.network_state.last_activity_time = clock.unix_timestamp;

        msg!("Mining started for user: {}", self.user.key());
        
        Ok(())
    }

    fn validate_human_behavior(&self) -> Result<()> {
        let user_account = &self.user_account;
        
        // Check for suspicious patterns
        if user_account.suspicious_activity_score > MAX_SUSPICIOUS_SCORE {
            return Err(FinovaError::SuspiciousActivity.into());
        }

        // Validate session timing patterns
        let now = Clock::get()?.unix_timestamp;
        if let Some(last_session) = user_account.last_mining_activity {
            let time_diff = now - last_session;
            
            // Too frequent sessions indicate bot activity
            if time_diff < MIN_SESSION_INTERVAL {
                return Err(FinovaError::TooFrequentActivity.into());
            }
        }

        Ok(())
    }

    fn calculate_streak(&self, current_time: i64) -> u64 {
        let last_activity = self.user_account.last_mining_activity.unwrap_or(0);
        let time_diff = current_time - last_activity;
        
        if time_diff <= STREAK_GRACE_PERIOD {
            self.user_account.daily_login_streak + 1
        } else if time_diff <= STREAK_BREAK_THRESHOLD {
            self.user_account.daily_login_streak
        } else {
            1 // Reset streak
        }
    }
}

impl<'info> ClaimMiningRewards<'info> {
    pub fn process(&mut self) -> Result<()> {
        let clock = Clock::get()?;
        
        // Validate mining is active
        require!(
            self.mining_account.is_active,
            FinovaError::MiningNotActive
        );

        // Calculate pending rewards
        let pending_rewards = self.calculate_pending_rewards(clock.unix_timestamp)?;
        
        // Validate rewards are available
        require!(
            pending_rewards > 0,
            FinovaError::NoRewardsToClaim
        );

        // Check daily mining cap
        let total_daily_amount = self.mining_account.daily_mined_amount + pending_rewards;
        require!(
            total_daily_amount <= self.mining_account.daily_mining_cap,
            FinovaError::DailyCapExceeded
        );

        // Validate rewards pool has sufficient balance
        require!(
            self.pool_token_account.amount >= pending_rewards,
            FinovaError::InsufficientRewardsPool
        );

        // Transfer rewards to user
        self.transfer_rewards(pending_rewards)?;

        // Update mining account state
        self.mining_account.last_claim_time = clock.unix_timestamp;
        self.mining_account.total_mined += pending_rewards;
        self.mining_account.daily_mined_amount += pending_rewards;

        // Update user account
        self.user_account.total_fin_holdings += pending_rewards;
        self.user_account.last_mining_activity = clock.unix_timestamp;

        // Update network statistics
        self.network_state.total_fin_mined += pending_rewards;
        self.network_state.last_activity_time = clock.unix_timestamp;

        // Update rewards pool
        self.rewards_pool.total_distributed += pending_rewards;

        msg!("Claimed {} FIN tokens for user: {}", pending_rewards, self.user.key());
        
        Ok(())
    }

    fn calculate_pending_rewards(&self, current_time: i64) -> Result<u64> {
        let time_diff = current_time - self.mining_account.last_claim_time;
        
        // Must wait at least minimum claim interval
        require!(
            time_diff >= MIN_CLAIM_INTERVAL,
            FinovaError::ClaimTooEarly
        );

        // Calculate base mining amount
        let hours_passed = time_diff as u64 / SECONDS_PER_HOUR;
        let base_amount = self.mining_account.base_rate * hours_passed;

        // Apply all multipliers
        let total_multiplier = self.calculate_total_multiplier();
        let gross_amount = base_amount * total_multiplier / PRECISION;

        // Apply quality score and anti-bot penalties
        let quality_adjusted = gross_amount * self.mining_account.quality_score_multiplier / PRECISION;
        let penalty_adjusted = if self.mining_account.anti_bot_penalty > 0 {
            quality_adjusted * (PRECISION - self.mining_account.anti_bot_penalty) / PRECISION
        } else {
            quality_adjusted
        };

        // Ensure minimum reward threshold
        let final_amount = std::cmp::max(penalty_adjusted, MIN_REWARD_THRESHOLD);

        Ok(final_amount)
    }

    fn calculate_total_multiplier(&self) -> u64 {
        let mut total_multiplier = PRECISION;

        // Finizen bonus
        total_multiplier = total_multiplier * self.mining_account.finizen_bonus / PRECISION;

        // Referral bonus
        total_multiplier = total_multiplier * self.mining_account.referral_bonus / PRECISION;

        // Security bonus
        total_multiplier = total_multiplier * self.mining_account.security_bonus / PRECISION;

        // Regression factor (anti-whale mechanism)
        total_multiplier = total_multiplier * self.mining_account.regression_factor / PRECISION;

        // XP level multiplier
        let xp_multiplier = self.user_account.xp_level_multiplier();
        total_multiplier = total_multiplier * xp_multiplier / PRECISION;

        // Staking multiplier
        if self.user_account.is_staking {
            total_multiplier = total_multiplier * STAKING_MULTIPLIER / PRECISION;
        }

        // Streak bonus
        total_multiplier = total_multiplier * self.mining_account.streak_bonus / PRECISION;

        // Special card boosts
        for boost in &self.mining_account.special_card_boosts {
            if boost.expires_at > Clock::get().unwrap().unix_timestamp {
                total_multiplier = total_multiplier * boost.multiplier / PRECISION;
            }
        }

        total_multiplier
    }

    fn transfer_rewards(&mut self, amount: u64) -> Result<()> {
        let seeds = &[
            REWARDS_POOL_SEED,
            &[self.rewards_pool.bump],
        ];
        let signer_seeds = &[&seeds[..]];

        let transfer_ctx = CpiContext::new_with_signer(
            self.token_program.to_account_info(),
            Transfer {
                from: self.pool_token_account.to_account_info(),
                to: self.user_token_account.to_account_info(),
                authority: self.rewards_pool.to_account_info(),
            },
            signer_seeds,
        );

        token::transfer(transfer_ctx, amount)?;
        
        Ok(())
    }
}

impl<'info> UpdateMiningMultipliers<'info> {
    pub fn process(&mut self, xp_multiplier: Option<u64>, referral_multiplier: Option<u64>) -> Result<()> {
        let clock = Clock::get()?;

        // Update XP multiplier if provided
        if let Some(multiplier) = xp_multiplier {
            require!(
                multiplier >= MIN_XP_MULTIPLIER && multiplier <= MAX_XP_MULTIPLIER,
                FinovaError::InvalidMultiplier
            );
            // XP multiplier is calculated from user account, so we update user XP instead
        }

        // Update referral multiplier if provided
        if let Some(multiplier) = referral_multiplier {
            require!(
                multiplier >= MIN_REFERRAL_MULTIPLIER && multiplier <= MAX_REFERRAL_MULTIPLIER,
                FinovaError::InvalidMultiplier
            );
            self.mining_account.referral_bonus = multiplier;
        }

        // Recalculate regression factor based on current holdings
        self.mining_account.regression_factor = self.calculate_current_regression_factor();

        // Update quality score multiplier based on recent activity
        self.mining_account.quality_score_multiplier = self.calculate_quality_score();

        // Clean up expired special card boosts
        self.mining_account.special_card_boosts.retain(|boost| {
            boost.expires_at > clock.unix_timestamp
        });

        msg!("Mining multipliers updated for user: {}", self.user.key());
        
        Ok(())
    }

    fn calculate_current_regression_factor(&self) -> u64 {
        let total_holdings = self.user_account.total_fin_holdings;
        let regression = ((-1.0 * REGRESSION_COEFFICIENT * total_holdings as f64).exp() * PRECISION as f64) as u64;
        std::cmp::max(regression, MIN_REGRESSION_FACTOR)
    }

    fn calculate_quality_score(&self) -> u64 {
        // This would integrate with off-chain AI quality assessment
        // For now, use a simplified calculation based on user activity
        let base_score = PRECISION;
        let activity_bonus = if self.user_account.high_quality_posts > 0 {
            (self.user_account.high_quality_posts * QUALITY_BONUS_PER_POST).min(MAX_QUALITY_BONUS)
        } else {
            0
        };
        
        std::cmp::max(base_score + activity_bonus, MIN_QUALITY_SCORE)
    }
}

impl<'info> ApplySpecialCard<'info> {
    pub fn process(&mut self, card_type: SpecialCardType, duration_hours: u64, multiplier: u64) -> Result<()> {
        let clock = Clock::get()?;
        
        // Validate multiplier is within bounds
        require!(
            multiplier >= MIN_CARD_MULTIPLIER && multiplier <= MAX_CARD_MULTIPLIER,
            FinovaError::InvalidMultiplier
        );

        // Validate duration
        require!(
            duration_hours > 0 && duration_hours <= MAX_CARD_DURATION_HOURS,
            FinovaError::InvalidDuration
        );

        // Check if user has the special card (this would be validated by NFT ownership)
        // For now, we assume the card ownership is validated off-chain

        // Calculate expiry time
        let expires_at = clock.unix_timestamp + (duration_hours * SECONDS_PER_HOUR) as i64;

        // Add boost to active boosts
        let boost = SpecialCardBoost {
            card_type,
            multiplier,
            expires_at,
            applied_at: clock.unix_timestamp,
        };

        // Check if we have space for more boosts
        require!(
            self.mining_account.special_card_boosts.len() < MAX_ACTIVE_CARD_BOOSTS,
            FinovaError::TooManyActiveBoosts
        );

        self.mining_account.special_card_boosts.push(boost);

        msg!("Special card applied for user: {}, type: {:?}, multiplier: {}%", 
             self.user.key(), card_type, multiplier);
        
        Ok(())
    }
}

impl<'info> PauseMining<'info> {
    pub fn process(&mut self, pause: bool) -> Result<()> {
        self.network_state.mining_paused = pause;
        
        let status = if pause { "paused" } else { "resumed" };
        msg!("Mining has been {} by admin: {}", status, self.admin.key());
        
        Ok(())
    }
}
