// programs/finova-token/src/utils.rs

//! Finova Token Program Utilities
//! 
//! This module provides utility functions for token operations, mathematical calculations,
//! validation, and security checks within the Finova token ecosystem.

use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount, Mint, Transfer, MintTo, Burn};
use std::collections::HashMap;
use crate::errors::TokenError;

/// Mathematical precision for calculations
pub const PRECISION: u64 = 1_000_000_000; // 9 decimal places
pub const MAX_SUPPLY: u64 = 100_000_000_000 * PRECISION; // 100 billion tokens
pub const SECONDS_PER_HOUR: u64 = 3600;
pub const SECONDS_PER_DAY: u64 = 86400;
pub const BASIS_POINTS: u64 = 10000;

/// Token validation utilities
pub struct TokenValidator;

impl TokenValidator {
    /// Validates token amount for overflow and underflow
    pub fn validate_amount(amount: u64) -> Result<()> {
        require!(amount > 0, TokenError::InvalidAmount);
        require!(amount <= MAX_SUPPLY, TokenError::AmountTooLarge);
        Ok(())
    }

    /// Validates token account ownership
    pub fn validate_token_account_owner(
        token_account: &Account<TokenAccount>,
        expected_owner: &Pubkey,
    ) -> Result<()> {
        require!(
            token_account.owner == *expected_owner,
            TokenError::InvalidTokenAccountOwner
        );
        Ok(())
    }

    /// Validates mint authority
    pub fn validate_mint_authority(
        mint: &Account<Mint>,
        expected_authority: &Pubkey,
    ) -> Result<()> {
        require!(
            mint.mint_authority.unwrap() == *expected_authority,
            TokenError::InvalidMintAuthority
        );
        Ok(())
    }

    /// Validates sufficient token balance
    pub fn validate_sufficient_balance(
        token_account: &Account<TokenAccount>,
        required_amount: u64,
    ) -> Result<()> {
        require!(
            token_account.amount >= required_amount,
            TokenError::InsufficientBalance
        );
        Ok(())
    }

    /// Validates token account mint
    pub fn validate_token_mint(
        token_account: &Account<TokenAccount>,
        expected_mint: &Pubkey,
    ) -> Result<()> {
        require!(
            token_account.mint == *expected_mint,
            TokenError::InvalidMint
        );
        Ok(())
    }
}

/// Mathematical utility functions
pub struct MathUtils;

impl MathUtils {
    /// Calculates percentage with precision
    pub fn calculate_percentage(amount: u64, percentage: u64) -> Result<u64> {
        require!(percentage <= BASIS_POINTS, TokenError::InvalidPercentage);
        
        let result = (amount as u128)
            .checked_mul(percentage as u128)
            .ok_or(TokenError::MathOverflow)?
            .checked_div(BASIS_POINTS as u128)
            .ok_or(TokenError::MathOverflow)?;
        
        require!(result <= u64::MAX as u128, TokenError::MathOverflow);
        Ok(result as u64)
    }

    /// Calculates compound interest
    pub fn calculate_compound_interest(
        principal: u64,
        rate: u64, // in basis points
        time_periods: u64,
    ) -> Result<u64> {
        if time_periods == 0 {
            return Ok(principal);
        }

        let rate_plus_one = BASIS_POINTS
            .checked_add(rate)
            .ok_or(TokenError::MathOverflow)?;

        let mut result = principal as u128;
        for _ in 0..time_periods {
            result = result
                .checked_mul(rate_plus_one as u128)
                .ok_or(TokenError::MathOverflow)?
                .checked_div(BASIS_POINTS as u128)
                .ok_or(TokenError::MathOverflow)?;
        }

        require!(result <= u64::MAX as u128, TokenError::MathOverflow);
        Ok(result as u64)
    }

    /// Calculates exponential decay for anti-whale mechanism
    pub fn calculate_exponential_decay(
        base_amount: u64,
        holdings: u64,
        decay_factor: u64, // in basis points
    ) -> Result<u64> {
        if holdings == 0 {
            return Ok(base_amount);
        }

        // Approximate e^(-decay_factor * holdings / PRECISION) using Taylor series
        let exponent = (decay_factor as u128)
            .checked_mul(holdings as u128)
            .ok_or(TokenError::MathOverflow)?
            .checked_div(PRECISION as u128)
            .ok_or(TokenError::MathOverflow)?;

        // e^(-x) ≈ 1 - x + x²/2 - x³/6 + x⁴/24 (first 5 terms)
        let x = exponent;
        let x2 = x.checked_mul(x).ok_or(TokenError::MathOverflow)?;
        let x3 = x2.checked_mul(x).ok_or(TokenError::MathOverflow)?;
        let x4 = x3.checked_mul(x).ok_or(TokenError::MathOverflow)?;

        let term1 = PRECISION as u128;
        let term2 = x;
        let term3 = x2.checked_div(2).ok_or(TokenError::MathOverflow)?;
        let term4 = x3.checked_div(6).ok_or(TokenError::MathOverflow)?;
        let term5 = x4.checked_div(24).ok_or(TokenError::MathOverflow)?;

        let decay_multiplier = term1
            .checked_sub(term2)
            .and_then(|v| v.checked_add(term3))
            .and_then(|v| v.checked_sub(term4))
            .and_then(|v| v.checked_add(term5))
            .ok_or(TokenError::MathOverflow)?;

        let result = (base_amount as u128)
            .checked_mul(decay_multiplier)
            .ok_or(TokenError::MathOverflow)?
            .checked_div(PRECISION as u128)
            .ok_or(TokenError::MathOverflow)?;

        require!(result <= u64::MAX as u128, TokenError::MathOverflow);
        Ok(result as u64)
    }

    /// Calculates staking rewards based on time and rate
    pub fn calculate_staking_rewards(
        staked_amount: u64,
        annual_rate: u64, // in basis points
        time_elapsed: u64, // in seconds
    ) -> Result<u64> {
        let seconds_per_year = 365 * SECONDS_PER_DAY;
        
        let time_fraction = (time_elapsed as u128)
            .checked_mul(PRECISION as u128)
            .ok_or(TokenError::MathOverflow)?
            .checked_div(seconds_per_year as u128)
            .ok_or(TokenError::MathOverflow)?;

        let rewards = (staked_amount as u128)
            .checked_mul(annual_rate as u128)
            .ok_or(TokenError::MathOverflow)?
            .checked_mul(time_fraction)
            .ok_or(TokenError::MathOverflow)?
            .checked_div(BASIS_POINTS as u128)
            .ok_or(TokenError::MathOverflow)?
            .checked_div(PRECISION as u128)
            .ok_or(TokenError::MathOverflow)?;

        require!(rewards <= u64::MAX as u128, TokenError::MathOverflow);
        Ok(rewards as u64)
    }

    /// Safe addition with overflow check
    pub fn safe_add(a: u64, b: u64) -> Result<u64> {
        a.checked_add(b).ok_or_else(|| error!(TokenError::MathOverflow))
    }

    /// Safe subtraction with underflow check
    pub fn safe_sub(a: u64, b: u64) -> Result<u64> {
        a.checked_sub(b).ok_or_else(|| error!(TokenError::MathUnderflow))
    }

    /// Safe multiplication with overflow check
    pub fn safe_mul(a: u64, b: u64) -> Result<u64> {
        a.checked_mul(b).ok_or_else(|| error!(TokenError::MathOverflow))
    }

    /// Safe division with zero check
    pub fn safe_div(a: u64, b: u64) -> Result<u64> {
        require!(b > 0, TokenError::DivisionByZero);
        Ok(a / b)
    }
}

/// Time utility functions
pub struct TimeUtils;

impl TimeUtils {
    /// Gets current Unix timestamp
    pub fn current_timestamp() -> Result<u64> {
        let clock = Clock::get()?;
        Ok(clock.unix_timestamp as u64)
    }

    /// Calculates time difference in seconds
    pub fn time_difference(start: u64, end: u64) -> Result<u64> {
        require!(end >= start, TokenError::InvalidTimeRange);
        Ok(end - start)
    }

    /// Checks if a time period has elapsed
    pub fn has_time_elapsed(start_time: u64, duration: u64) -> Result<bool> {
        let current_time = Self::current_timestamp()?;
        Ok(current_time >= start_time + duration)
    }

    /// Converts days to seconds
    pub fn days_to_seconds(days: u64) -> u64 {
        days * SECONDS_PER_DAY
    }

    /// Converts hours to seconds  
    pub fn hours_to_seconds(hours: u64) -> u64 {
        hours * SECONDS_PER_HOUR
    }

    /// Gets the start of current day timestamp
    pub fn get_day_start() -> Result<u64> {
        let current_time = Self::current_timestamp()?;
        Ok((current_time / SECONDS_PER_DAY) * SECONDS_PER_DAY)
    }

    /// Checks if two timestamps are on the same day
    pub fn same_day(timestamp1: u64, timestamp2: u64) -> bool {
        (timestamp1 / SECONDS_PER_DAY) == (timestamp2 / SECONDS_PER_DAY)
    }
}

/// Token transfer utilities
pub struct TransferUtils;

impl TransferUtils {
    /// Creates transfer instruction context
    pub fn create_transfer_context<'info>(
        token_program: &Program<'info, Token>,
        from: &Account<'info, TokenAccount>,
        to: &Account<'info, TokenAccount>,
        authority: &Signer<'info>,
    ) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        CpiContext::new(
            token_program.to_account_info(),
            Transfer {
                from: from.to_account_info(),
                to: to.to_account_info(),
                authority: authority.to_account_info(),
            },
        )
    }

    /// Creates mint tokens instruction context
    pub fn create_mint_context<'info>(
        token_program: &Program<'info, Token>,
        mint: &Account<'info, Mint>,
        to: &Account<'info, TokenAccount>,
        authority: &Signer<'info>,
    ) -> CpiContext<'_, '_, '_, 'info, MintTo<'info>> {
        CpiContext::new(
            token_program.to_account_info(),
            MintTo {
                mint: mint.to_account_info(),
                to: to.to_account_info(),
                authority: authority.to_account_info(),
            },
        )
    }

    /// Creates burn tokens instruction context
    pub fn create_burn_context<'info>(
        token_program: &Program<'info, Token>,
        mint: &Account<'info, Mint>,
        from: &Account<'info, TokenAccount>,
        authority: &Signer<'info>,
    ) -> CpiContext<'_, '_, '_, 'info, Burn<'info>> {
        CpiContext::new(
            token_program.to_account_info(),
            Burn {
                mint: mint.to_account_info(),
                from: from.to_account_info(),
                authority: authority.to_account_info(),
            },
        )
    }

    /// Transfers tokens with validation
    pub fn transfer_tokens_checked<'info>(
        ctx: CpiContext<'_, '_, '_, 'info, Transfer<'info>>,
        amount: u64,
    ) -> Result<()> {
        TokenValidator::validate_amount(amount)?;
        anchor_spl::token::transfer(ctx, amount)?;
        Ok(())
    }

    /// Mints tokens with validation
    pub fn mint_tokens_checked<'info>(
        ctx: CpiContext<'_, '_, '_, 'info, MintTo<'info>>,
        amount: u64,
    ) -> Result<()> {
        TokenValidator::validate_amount(amount)?;
        anchor_spl::token::mint_to(ctx, amount)?;
        Ok(())
    }

    /// Burns tokens with validation
    pub fn burn_tokens_checked<'info>(
        ctx: CpiContext<'_, '_, '_, 'info, Burn<'info>>,
        amount: u64,
    ) -> Result<()> {
        TokenValidator::validate_amount(amount)?;
        anchor_spl::token::burn(ctx, amount)?;
        Ok(())
    }
}

/// Security and access control utilities
pub struct SecurityUtils;

impl SecurityUtils {
    /// Validates program derived address
    pub fn validate_pda(
        address: &Pubkey,
        seeds: &[&[u8]],
        program_id: &Pubkey,
    ) -> Result<u8> {
        let (expected_address, bump) = Pubkey::find_program_address(seeds, program_id);
        require!(
            *address == expected_address,
            TokenError::InvalidPDA
        );
        Ok(bump)
    }

    /// Checks if signer is authorized
    pub fn check_authorization(
        signer: &Pubkey,
        authorized_keys: &[Pubkey],
    ) -> Result<()> {
        require!(
            authorized_keys.contains(signer),
            TokenError::UnauthorizedAccess
        );
        Ok(())
    }

    /// Validates signature freshness (prevents replay attacks)
    pub fn validate_signature_freshness(
        timestamp: u64,
        max_age: u64,
    ) -> Result<()> {
        let current_time = TimeUtils::current_timestamp()?;
        require!(
            current_time <= timestamp + max_age,
            TokenError::ExpiredSignature
        );
        Ok(())
    }

    /// Generates deterministic seed for PDA
    pub fn generate_seed(base: &str, identifier: &Pubkey) -> Vec<u8> {
        let mut seed = base.as_bytes().to_vec();
        seed.extend_from_slice(identifier.as_ref());
        seed
    }

    /// Rate limiting check
    pub fn check_rate_limit(
        last_action_time: u64,
        min_interval: u64,
    ) -> Result<()> {
        let current_time = TimeUtils::current_timestamp()?;
        require!(
            current_time >= last_action_time + min_interval,
            TokenError::RateLimitExceeded
        );
        Ok(())
    }
}

/// Token economics and calculation utilities
pub struct TokenEconomics;

impl TokenEconomics {
    /// Calculates mining rate based on network size and user holdings
    pub fn calculate_mining_rate(
        base_rate: u64,
        total_users: u64,
        user_holdings: u64,
        referral_count: u64,
        is_kyc_verified: bool,
    ) -> Result<u64> {
        // Pioneer bonus: decreases as network grows
        let pioneer_bonus = if total_users < 1_000_000 {
            let factor = 2_000_000u128
                .checked_sub(total_users as u128)
                .ok_or(TokenError::MathOverflow)?
                .checked_div(1_000_000u128)
                .ok_or(TokenError::MathOverflow)?;
            factor as u64
        } else {
            PRECISION
        };

        // Referral bonus: increases with active referrals (max 3x)
        let referral_bonus = PRECISION + (referral_count * 100_000_000).min(2 * PRECISION);

        // Security bonus for KYC verified users
        let security_bonus = if is_kyc_verified {
            1_200_000_000 // 1.2x
        } else {
            800_000_000 // 0.8x
        };

        // Exponential regression factor for large holders
        let regression_factor = MathUtils::calculate_exponential_decay(
            PRECISION,
            user_holdings,
            1000, // 0.1% decay factor
        )?;

        let rate = (base_rate as u128)
            .checked_mul(pioneer_bonus as u128)
            .ok_or(TokenError::MathOverflow)?
            .checked_div(PRECISION as u128)
            .ok_or(TokenError::MathOverflow)?
            .checked_mul(referral_bonus as u128)
            .ok_or(TokenError::MathOverflow)?
            .checked_div(PRECISION as u128)
            .ok_or(TokenError::MathOverflow)?
            .checked_mul(security_bonus as u128)
            .ok_or(TokenError::MathOverflow)?
            .checked_div(PRECISION as u128)
            .ok_or(TokenError::MathOverflow)?
            .checked_mul(regression_factor as u128)
            .ok_or(TokenError::MathOverflow)?
            .checked_div(PRECISION as u128)
            .ok_or(TokenError::MathOverflow)?;

        require!(rate <= u64::MAX as u128, TokenError::MathOverflow);
        Ok(rate as u64)
    }

    /// Calculates staking tier based on staked amount
    pub fn calculate_staking_tier(staked_amount: u64) -> u8 {
        match staked_amount {
            0..=99 => 0,                    // No staking
            100..=499 => 1,                 // Tier 1: 100-499 FIN
            500..=999 => 2,                 // Tier 2: 500-999 FIN  
            1000..=4999 => 3,               // Tier 3: 1K-4.9K FIN
            5000..=9999 => 4,               // Tier 4: 5K-9.9K FIN
            _ => 5,                         // Tier 5: 10K+ FIN
        }
    }

    /// Calculates burn amount for deflationary mechanism
    pub fn calculate_burn_amount(
        transaction_amount: u64,
        burn_rate: u64, // in basis points
    ) -> Result<u64> {
        MathUtils::calculate_percentage(transaction_amount, burn_rate)
    }

    /// Calculates token distribution for new phases
    pub fn calculate_phase_distribution(
        total_supply: u64,
        current_phase: u8,
    ) -> Result<(u64, u64)> { // Returns (community_allocation, team_allocation)
        let phase_multiplier = match current_phase {
            1 => 5000, // 50% community, 20% team in phase 1
            2 => 4500, // 45% community, 25% team in phase 2  
            3 => 4000, // 40% community, 30% team in phase 3
            _ => 3500, // 35% community, 35% team in later phases
        };

        let community_allocation = MathUtils::calculate_percentage(
            total_supply,
            phase_multiplier,
        )?;

        let team_multiplier = match current_phase {
            1 => 2000,
            2 => 2500,
            3 => 3000,
            _ => 3500,
        };

        let team_allocation = MathUtils::calculate_percentage(
            total_supply,
            team_multiplier,
        )?;

        Ok((community_allocation, team_allocation))
    }
}

/// Data validation and sanitization utilities
pub struct ValidationUtils;

impl ValidationUtils {
    /// Validates user input for special characters and SQL injection
    pub fn sanitize_string(input: &str) -> Result<String> {
        require!(input.len() <= 256, TokenError::StringTooLong);
        require!(!input.is_empty(), TokenError::EmptyString);
        
        // Remove potentially dangerous characters
        let sanitized: String = input
            .chars()
            .filter(|c| c.is_alphanumeric() || " -_@.".contains(*c))
            .collect();
        
        require!(!sanitized.is_empty(), TokenError::InvalidString);
        Ok(sanitized)
    }

    /// Validates email format (basic validation)
    pub fn validate_email(email: &str) -> Result<()> {
        require!(email.len() <= 100, TokenError::EmailTooLong);
        require!(email.contains('@'), TokenError::InvalidEmailFormat);
        require!(email.matches('@').count() == 1, TokenError::InvalidEmailFormat);
        
        let parts: Vec<&str> = email.split('@').collect();
        require!(parts.len() == 2, TokenError::InvalidEmailFormat);
        require!(!parts[0].is_empty(), TokenError::InvalidEmailFormat);
        require!(!parts[1].is_empty(), TokenError::InvalidEmailFormat);
        require!(parts[1].contains('.'), TokenError::InvalidEmailFormat);
        
        Ok(())
    }

    /// Validates numeric ranges
    pub fn validate_range(value: u64, min: u64, max: u64) -> Result<()> {
        require!(value >= min, TokenError::ValueTooLow);
        require!(value <= max, TokenError::ValueTooHigh);
        Ok(())
    }

    /// Validates percentage values (0-100%)
    pub fn validate_percentage(percentage: u64) -> Result<()> {
        require!(percentage <= BASIS_POINTS, TokenError::InvalidPercentage);
        Ok(())
    }
}

/// Logging and event utilities
pub struct LogUtils;

impl LogUtils {
    /// Emits a standardized log event
    pub fn emit_event<T: AnchorSerialize>(event_type: &str, data: &T) -> Result<()> {
        msg!("EVENT: {} - {:?}", event_type, data);
        Ok(())
    }

    /// Logs error with context
    pub fn log_error(error: &str, context: &str) {
        msg!("ERROR: {} - Context: {}", error, context);
    }

    /// Logs transaction details
    pub fn log_transaction(
        from: &Pubkey,
        to: &Pubkey,
        amount: u64,
        transaction_type: &str,
    ) {
        msg!(
            "TRANSACTION: {} | From: {} | To: {} | Amount: {}",
            transaction_type,
            from,
            to,
            amount
        );
    }

    /// Logs mining activity
    pub fn log_mining_activity(
        user: &Pubkey,
        amount_mined: u64,
        rate: u64,
        bonuses: &str,
    ) {
        msg!(
            "MINING: User: {} | Mined: {} | Rate: {} | Bonuses: {}",
            user,
            amount_mined,
            rate,
            bonuses
        );
    }
}

/// Testing utilities (only compiled in test configuration)
#[cfg(test)]
pub struct TestUtils;

#[cfg(test)]
impl TestUtils {
    /// Creates a mock token account for testing
    pub fn create_mock_token_account(
        mint: Pubkey,
        owner: Pubkey,
        amount: u64,
    ) -> TokenAccount {
        TokenAccount {
            mint,
            owner,
            amount,
            delegate: None,
            state: anchor_spl::token::spl_token::state::AccountState::Initialized,
            is_native: None,
            delegated_amount: 0,
            close_authority: None,
        }
    }

    /// Generates test pubkeys
    pub fn generate_test_pubkey(seed: u8) -> Pubkey {
        let mut bytes = [0u8; 32];
        bytes[0] = seed;
        Pubkey::new_from_array(bytes)
    }

    /// Creates test timestamp
    pub fn create_test_timestamp(days_ago: u64) -> u64 {
        let current = 1640995200; // Jan 1, 2022 as base
        current - (days_ago * SECONDS_PER_DAY)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_percentage() {
        let result = MathUtils::calculate_percentage(1000, 500).unwrap(); // 5%
        assert_eq!(result, 50);
    }

    #[test]
    fn test_exponential_decay() {
        let result = MathUtils::calculate_exponential_decay(
            1000 * PRECISION,
            10000 * PRECISION,
            1000,
        ).unwrap();
        assert!(result < 1000 * PRECISION); // Should be less than original
    }

    #[test]
    fn test_staking_tier_calculation() {
        assert_eq!(TokenEconomics::calculate_staking_tier(50), 0);
        assert_eq!(TokenEconomics::calculate_staking_tier(250), 1);
        assert_eq!(TokenEconomics::calculate_staking_tier(750), 2);
        assert_eq!(TokenEconomics::calculate_staking_tier(2500), 3);
        assert_eq!(TokenEconomics::calculate_staking_tier(7500), 4);
        assert_eq!(TokenEconomics::calculate_staking_tier(15000), 5);
    }

    #[test]
    fn test_time_difference() {
        let start = 1640995200;
        let end = start + 3600; // 1 hour later
        let diff = TimeUtils::time_difference(start, end).unwrap();
        assert_eq!(diff, 3600);
    }

    #[test]
    fn test_email_validation() {
        assert!(ValidationUtils::validate_email("test@example.com").is_ok());
        assert!(ValidationUtils::validate_email("invalid-email").is_err());
        assert!(ValidationUtils::validate_email("test@@example.com").is_err());
        assert!(ValidationUtils::validate_email("@example.com").is_err());
        assert!(ValidationUtils::validate_email("test@").is_err());
    }

    #[test]
    fn test_safe_math_operations() {
        assert_eq!(MathUtils::safe_add(100, 200).unwrap(), 300);
        assert_eq!(MathUtils::safe_sub(300, 100).unwrap(), 200);
        assert_eq!(MathUtils::safe_mul(10, 20).unwrap(), 200);
        assert_eq!(MathUtils::safe_div(100, 5).unwrap(), 20);
        
        // Test overflow
        assert!(MathUtils::safe_add(u64::MAX, 1).is_err());
        assert!(MathUtils::safe_sub(100, 200).is_err());
        assert!(MathUtils::safe_div(100, 0).is_err());
    }
}
