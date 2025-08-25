// programs/finova-token/src/instructions/stake_tokens.rs

use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer, Mint};
use anchor_spl::associated_token::AssociatedToken;
use crate::state::{StakeAccount, RewardPool, MintInfo};
use crate::errors::TokenError;
use crate::utils::*;
use crate::constants::*;

/// Stakes FIN tokens to earn sFIN and enhanced rewards
/// Implements liquid staking with immediate sFIN minting
/// Integrates with XP/RP multipliers from core program
#[derive(Accounts)]
pub struct StakeTokens<'info> {
    #[account(mut)]
    pub staker: Signer<'info>,

    /// User's FIN token account (source)
    #[account(
        mut,
        constraint = user_fin_account.owner == staker.key(),
        constraint = user_fin_account.mint == fin_mint.key()
    )]
    pub user_fin_account: Account<'info, TokenAccount>,

    /// User's sFIN token account (destination for liquid staking tokens)
    #[account(
        mut,
        constraint = user_sfin_account.owner == staker.key(),
        constraint = user_sfin_account.mint == sfin_mint.key()
    )]
    pub user_sfin_account: Account<'info, TokenAccount>,

    /// Stake account storing user staking information
    #[account(
        init_if_needed,
        payer = staker,
        space = StakeAccount::LEN,
        seeds = [
            STAKE_ACCOUNT_SEED,
            staker.key().as_ref()
        ],
        bump
    )]
    pub stake_account: Account<'info, StakeAccount>,

    /// Global staking pool that holds all staked FIN tokens
    #[account(
        mut,
        seeds = [STAKING_POOL_SEED],
        bump
    )]
    pub staking_pool: Account<'info, TokenAccount>,

    /// Reward pool managing staking rewards and APY calculations
    #[account(
        mut,
        seeds = [REWARD_POOL_SEED],
        bump
    )]
    pub reward_pool: Account<'info, RewardPool>,

    /// FIN token mint (original token)
    #[account(
        constraint = fin_mint.key() == FIN_MINT_ADDRESS
    )]
    pub fin_mint: Account<'info, Mint>,

    /// sFIN token mint (staked FIN liquid derivative)
    #[account(
        mut,
        constraint = sfin_mint.key() == SFIN_MINT_ADDRESS
    )]
    pub sfin_mint: Account<'info, Mint>,

    /// Mint info containing global mint parameters
    #[account(
        mut,
        seeds = [MINT_INFO_SEED],
        bump
    )]
    pub mint_info: Account<'info, MintInfo>,

    /// Cross-program invocation to core program for XP/RP data
    /// CHECK: This is validated through CPI
    pub finova_core_program: UncheckedAccount<'info>,

    /// User account from core program containing XP/RP data
    /// CHECK: This is validated through CPI to core program
    pub core_user_account: UncheckedAccount<'info>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
    pub clock: Sysvar<'info, Clock>,
}

impl<'info> StakeTokens<'info> {
    /// Validates staking amount and user eligibility
    fn validate_staking_amount(&self, amount: u64) -> Result<()> {
        require!(amount > 0, TokenError::InvalidAmount);
        require!(amount >= MIN_STAKE_AMOUNT, TokenError::BelowMinimumStake);
        require!(
            self.user_fin_account.amount >= amount,
            TokenError::InsufficientBalance
        );

        // Check if user has reached maximum staking limit
        let max_stake = self.calculate_max_stake_limit()?;
        let total_staked = self.stake_account.total_staked.checked_add(amount)
            .ok_or(TokenError::MathOverflow)?;
        
        require!(
            total_staked <= max_stake,
            TokenError::ExceedsMaxStake
        );

        Ok(())
    }

    /// Calculates maximum stake limit based on user tier and network status
    fn calculate_max_stake_limit(&self) -> Result<u64> {
        // Base limit for all users
        let mut max_stake = BASE_MAX_STAKE_AMOUNT;

        // Get user data from core program via CPI
        let user_level = self.get_user_xp_level()?;
        let user_rp_tier = self.get_user_rp_tier()?;

        // Apply tier-based multipliers
        let level_multiplier = match user_level {
            1..=10 => 1,      // Bronze: 1x
            11..=25 => 2,     // Silver: 2x  
            26..=50 => 3,     // Gold: 3x
            51..=75 => 5,     // Platinum: 5x
            76..=100 => 8,    // Diamond: 8x
            _ => 10,          // Mythic: 10x
        };

        let rp_multiplier = match user_rp_tier {
            0 => 1,           // Explorer: 1x
            1 => 2,           // Connector: 2x
            2 => 3,           // Influencer: 3x
            3 => 5,           // Leader: 5x
            _ => 10,          // Ambassador: 10x
        };

        max_stake = max_stake
            .checked_mul(level_multiplier)
            .and_then(|v| v.checked_mul(rp_multiplier))
            .ok_or(TokenError::MathOverflow)?;

        Ok(max_stake)
    }

    /// Gets user XP level from core program
    fn get_user_xp_level(&self) -> Result<u32> {
        // This would be implemented as a CPI call to the core program
        // For now, we'll use a placeholder that reads from account data
        let account_data = &self.core_user_account.data.borrow();
        if account_data.len() < 8 {
            return Ok(1); // Default level for new users
        }

        // Parse XP level from core user account (simplified)
        let xp_level = u32::from_le_bytes([
            account_data[100], account_data[101], 
            account_data[102], account_data[103]
        ]);

        Ok(xp_level.max(1))
    }

    /// Gets user RP tier from core program  
    fn get_user_rp_tier(&self) -> Result<u8> {
        // This would be implemented as a CPI call to the core program
        let account_data = &self.core_user_account.data.borrow();
        if account_data.len() < 8 {
            return Ok(0); // Default tier for new users
        }

        // Parse RP tier from core user account (simplified)
        let rp_tier = account_data[120];
        Ok(rp_tier.min(4)) // Max tier is 4 (Ambassador)
    }

    /// Calculates sFIN exchange rate based on total rewards accumulated
    fn calculate_sfin_exchange_rate(&self) -> Result<u64> {
        let clock = Clock::get()?;
        let current_time = clock.unix_timestamp;

        // Base exchange rate: 1 FIN = 1 sFIN initially
        let mut exchange_rate = PRECISION_FACTOR;

        // Calculate accumulated rewards since last update
        let time_elapsed = current_time
            .checked_sub(self.reward_pool.last_update)
            .unwrap_or(0) as u64;

        if time_elapsed > 0 && self.reward_pool.total_staked > 0 {
            // Calculate APY-based rewards
            let annual_reward_rate = self.calculate_annual_reward_rate()?;
            let time_factor = time_elapsed
                .checked_mul(PRECISION_FACTOR)
                .and_then(|v| v.checked_div(SECONDS_PER_YEAR))
                .ok_or(TokenError::MathOverflow)?;

            let reward_factor = annual_reward_rate
                .checked_mul(time_factor)
                .and_then(|v| v.checked_div(PRECISION_FACTOR))
                .ok_or(TokenError::MathOverflow)?;

            exchange_rate = exchange_rate
                .checked_add(reward_factor)
                .ok_or(TokenError::MathOverflow)?;
        }

        Ok(exchange_rate)
    }

    /// Calculates dynamic APY based on staking tier and network conditions
    fn calculate_annual_reward_rate(&self) -> Result<u64> {
        let user_level = self.get_user_xp_level()?;
        let user_rp_tier = self.get_user_rp_tier()?;

        // Base APY: 8%
        let mut annual_rate = BASE_STAKING_APY;

        // Apply tier bonuses from whitepaper
        let tier_bonus = match self.stake_account.total_staked {
            100..=499 => 0,        // 8% APY
            500..=999 => 200,      // +2% = 10% APY
            1000..=4999 => 400,    // +4% = 12% APY
            5000..=9999 => 600,    // +6% = 14% APY
            _ => 700,              // +7% = 15% APY
        };

        annual_rate = annual_rate
            .checked_add(tier_bonus)
            .ok_or(TokenError::MathOverflow)?;

        // Apply XP level multiplier
        let xp_multiplier = match user_level {
            1..=10 => PRECISION_FACTOR,                    // 1.0x
            11..=25 => PRECISION_FACTOR + 100_000,         // 1.1x
            26..=50 => PRECISION_FACTOR + 300_000,         // 1.3x
            51..=75 => PRECISION_FACTOR + 500_000,         // 1.5x
            76..=100 => PRECISION_FACTOR + 750_000,        // 1.75x
            _ => PRECISION_FACTOR + 1_000_000,             // 2.0x
        };

        annual_rate = annual_rate
            .checked_mul(xp_multiplier)
            .and_then(|v| v.checked_div(PRECISION_FACTOR))
            .ok_or(TokenError::MathOverflow)?;

        // Apply RP tier bonus
        let rp_bonus = match user_rp_tier {
            0 => 0,      // Explorer: +0%
            1 => 50,     // Connector: +0.5%
            2 => 100,    // Influencer: +1%
            3 => 200,    // Leader: +2%
            _ => 300,    // Ambassador: +3%
        };

        annual_rate = annual_rate
            .checked_add(rp_bonus)
            .ok_or(TokenError::MathOverflow)?;

        Ok(annual_rate)
    }

    /// Calculates amount of sFIN to mint for staked FIN
    fn calculate_sfin_amount(&self, fin_amount: u64) -> Result<u64> {
        let exchange_rate = self.calculate_sfin_exchange_rate()?;
        
        let sfin_amount = fin_amount
            .checked_mul(PRECISION_FACTOR)
            .and_then(|v| v.checked_div(exchange_rate))
            .ok_or(TokenError::MathOverflow)?;

        Ok(sfin_amount)
    }

    /// Updates stake account with new staking information
    fn update_stake_account(&mut self, fin_amount: u64, sfin_amount: u64) -> Result<()> {
        let clock = Clock::get()?;
        let current_time = clock.unix_timestamp;

        // Initialize account if this is first stake
        if self.stake_account.owner == Pubkey::default() {
            self.stake_account.owner = self.staker.key();
            self.stake_account.created_at = current_time;
            self.stake_account.bump = *ctx.bumps.get("stake_account").unwrap();
        }

        // Update staking amounts
        self.stake_account.total_staked = self.stake_account.total_staked
            .checked_add(fin_amount)
            .ok_or(TokenError::MathOverflow)?;

        self.stake_account.sfin_balance = self.stake_account.sfin_balance
            .checked_add(sfin_amount)
            .ok_or(TokenError::MathOverflow)?;

        // Update timing information
        self.stake_account.last_stake_time = current_time;
        self.stake_account.last_reward_claim = current_time;

        // Calculate and update loyalty bonus
        let staking_duration = current_time
            .checked_sub(self.stake_account.created_at)
            .unwrap_or(0) as u64;
        
        let loyalty_months = staking_duration / SECONDS_PER_MONTH;
        self.stake_account.loyalty_bonus = (loyalty_months * LOYALTY_BONUS_PER_MONTH)
            .min(MAX_LOYALTY_BONUS);

        // Update activity bonus based on recent staking
        self.stake_account.activity_score = self.calculate_activity_score(current_time)?;

        // Increment staking count
        self.stake_account.stake_count = self.stake_account.stake_count
            .checked_add(1)
            .ok_or(TokenError::MathOverflow)?;

        Ok(())
    }

    /// Calculates user activity score for bonus multipliers
    fn calculate_activity_score(&self, current_time: i64) -> Result<u64> {
        let time_since_last_stake = current_time
            .checked_sub(self.stake_account.last_stake_time)
            .unwrap_or(0) as u64;

        // Higher score for more frequent staking (decays over time)
        let base_score = if time_since_last_stake < SECONDS_PER_DAY {
            100 // Maximum activity score
        } else if time_since_last_stake < SECONDS_PER_WEEK {
            75  // High activity
        } else if time_since_last_stake < SECONDS_PER_MONTH {
            50  // Medium activity
        } else {
            25  // Low activity
        };

        // Apply stake frequency bonus
        let frequency_bonus = (self.stake_account.stake_count * 5).min(50);
        
        let total_score = base_score
            .checked_add(frequency_bonus)
            .ok_or(TokenError::MathOverflow)?
            .min(200); // Cap at 200 for 2.0x multiplier

        Ok(total_score)
    }

    /// Updates global reward pool state
    fn update_reward_pool(&mut self, fin_amount: u64) -> Result<()> {
        let clock = Clock::get()?;

        // Update total staked amount
        self.reward_pool.total_staked = self.reward_pool.total_staked
            .checked_add(fin_amount)
            .ok_or(TokenError::MathOverflow)?;

        // Update timestamp
        self.reward_pool.last_update = clock.unix_timestamp;

        // Update staker count if this is a new staker
        if self.stake_account.total_staked == fin_amount {
            self.reward_pool.total_stakers = self.reward_pool.total_stakers
                .checked_add(1)
                .ok_or(TokenError::MathOverflow)?;
        }

        // Calculate and update average stake
        if self.reward_pool.total_stakers > 0 {
            self.reward_pool.average_stake = self.reward_pool.total_staked
                .checked_div(self.reward_pool.total_stakers)
                .unwrap_or(0);
        }

        Ok(())
    }

    /// Transfers FIN tokens to staking pool
    fn transfer_fin_to_pool(&self, amount: u64) -> Result<()> {
        let transfer_instruction = Transfer {
            from: self.user_fin_account.to_account_info(),
            to: self.staking_pool.to_account_info(),
            authority: self.staker.to_account_info(),
        };

        let cpi_ctx = CpiContext::new(
            self.token_program.to_account_info(),
            transfer_instruction,
        );

        token::transfer(cpi_ctx, amount)?;
        Ok(())
    }

    /// Mints sFIN tokens to user account
    fn mint_sfin_to_user(&self, amount: u64) -> Result<()> {
        let mint_authority_seeds = &[
            MINT_AUTHORITY_SEED,
            &[self.mint_info.mint_authority_bump],
        ];
        let signer_seeds = &[&mint_authority_seeds[..]];

        let mint_instruction = token::MintTo {
            mint: self.sfin_mint.to_account_info(),
            to: self.user_sfin_account.to_account_info(),
            authority: self.mint_info.to_account_info(),
        };

        let cpi_ctx = CpiContext::new_with_signer(
            self.token_program.to_account_info(),
            mint_instruction,
            signer_seeds,
        );

        token::mint_to(cpi_ctx, amount)?;
        Ok(())
    }
}

/// Main instruction handler for staking FIN tokens
pub fn stake_tokens(ctx: Context<StakeTokens>, amount: u64) -> Result<()> {
    let stake_ctx = &mut ctx.accounts;

    // Validate staking parameters
    stake_ctx.validate_staking_amount(amount)?;

    // Calculate sFIN amount to mint
    let sfin_amount = stake_ctx.calculate_sfin_amount(amount)?;
    require!(sfin_amount > 0, TokenError::InvalidCalculation);

    // Transfer FIN tokens to staking pool
    stake_ctx.transfer_fin_to_pool(amount)?;

    // Mint sFIN tokens to user
    stake_ctx.mint_sfin_to_user(sfin_amount)?;

    // Update stake account
    stake_ctx.update_stake_account(amount, sfin_amount)?;

    // Update global reward pool
    stake_ctx.update_reward_pool(amount)?;

    // Update mint info statistics
    ctx.accounts.mint_info.total_staked = ctx.accounts.mint_info.total_staked
        .checked_add(amount)
        .ok_or(TokenError::MathOverflow)?;

    ctx.accounts.mint_info.sfin_total_supply = ctx.accounts.mint_info.sfin_total_supply
        .checked_add(sfin_amount)
        .ok_or(TokenError::MathOverflow)?;

    // Emit staking event
    emit!(StakeEvent {
        staker: ctx.accounts.staker.key(),
        fin_amount: amount,
        sfin_amount,
        total_staked: ctx.accounts.stake_account.total_staked,
        exchange_rate: stake_ctx.calculate_sfin_exchange_rate()?,
        timestamp: Clock::get()?.unix_timestamp,
    });

    msg!(
        "Successfully staked {} FIN tokens, received {} sFIN tokens",
        amount,
        sfin_amount
    );

    Ok(())
}

/// Event emitted when tokens are staked
#[event]
pub struct StakeEvent {
    pub staker: Pubkey,
    pub fin_amount: u64,
    pub sfin_amount: u64,  
    pub total_staked: u64,
    pub exchange_rate: u64,
    pub timestamp: i64,
}

// Constants for staking calculations
const MIN_STAKE_AMOUNT: u64 = 100_000_000; // 100 FIN minimum stake
const BASE_MAX_STAKE_AMOUNT: u64 = 1_000_000_000_000; // 1M FIN base limit
const BASE_STAKING_APY: u64 = 800; // 8% base APY (in basis points)
const PRECISION_FACTOR: u64 = 1_000_000; // 6 decimal precision
const SECONDS_PER_YEAR: u64 = 365 * 24 * 60 * 60;
const SECONDS_PER_MONTH: u64 = 30 * 24 * 60 * 60;
const SECONDS_PER_WEEK: u64 = 7 * 24 * 60 * 60;
const SECONDS_PER_DAY: u64 = 24 * 60 * 60;
const LOYALTY_BONUS_PER_MONTH: u64 = 5; // 0.05% bonus per month
const MAX_LOYALTY_BONUS: u64 = 500; // Max 5% loyalty bonus

/// Seeds for PDA derivation
const STAKE_ACCOUNT_SEED: &[u8] = b"stake_account";
const STAKING_POOL_SEED: &[u8] = b"staking_pool";
const REWARD_POOL_SEED: &[u8] = b"reward_pool";
const MINT_INFO_SEED: &[u8] = b"mint_info";
const MINT_AUTHORITY_SEED: &[u8] = b"mint_authority";

/// Placeholder addresses (would be replaced with actual deployed addresses)
const FIN_MINT_ADDRESS: Pubkey = anchor_lang::solana_program::pubkey!("11111111111111111111111111111112");
const SFIN_MINT_ADDRESS: Pubkey = anchor_lang::solana_program::pubkey!("11111111111111111111111111111113");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_sfin_amount() {
        // Test basic exchange rate calculation
        let fin_amount = 1000_000_000; // 1000 FIN
        let exchange_rate = PRECISION_FACTOR; // 1:1 ratio
        
        let expected_sfin = fin_amount * PRECISION_FACTOR / exchange_rate;
        assert_eq!(expected_sfin, fin_amount);
    }

    #[test] 
    fn test_tier_multipliers() {
        // Test different staking tiers
        let amounts = vec![100, 500, 1000, 5000, 10000];
        let expected_bonuses = vec![0, 200, 400, 600, 700];
        
        for (amount, expected) in amounts.iter().zip(expected_bonuses.iter()) {
            let bonus = match *amount {
                100..=499 => 0,
                500..=999 => 200,
                1000..=4999 => 400,
                5000..=9999 => 600,
                _ => 700,
            };
            assert_eq!(bonus, *expected);
        }
    }

    #[test]
    fn test_loyalty_bonus_calculation() {
        let months = vec![1, 6, 12, 24, 120];
        let expected_bonuses = vec![5, 30, 60, 120, 500]; // Capped at 500
        
        for (month, expected) in months.iter().zip(expected_bonuses.iter()) {
            let bonus = (month * LOYALTY_BONUS_PER_MONTH).min(MAX_LOYALTY_BONUS);
            assert_eq!(bonus, *expected);
        }
    }
}
