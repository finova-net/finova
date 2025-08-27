// programs/finova-token/src/instructions/burn_tokens.rs

use anchor_lang::prelude::*;
use anchor_spl::token::{self, Burn, Mint, Token, TokenAccount};
use crate::constants::*;
use crate::errors::TokenError;
use crate::state::{MintInfo, StakeAccount};
use crate::utils::*;

/// Burns tokens as part of deflationary mechanisms
/// Supports multiple burn types: transaction fees, whale tax, special card usage
#[derive(Accounts)]
#[instruction(params: BurnTokensParams)]
pub struct BurnTokens<'info> {
    #[account(
        mut,
        seeds = [MINT_INFO_SEED, mint.key().as_ref()],
        bump = mint_info.bump,
    )]
    pub mint_info: Account<'info, MintInfo>,
    
    #[account(mut)]
    pub mint: Account<'info, Mint>,
    
    #[account(
        mut,
        constraint = token_account.mint == mint.key() @ TokenError::InvalidMint,
        constraint = token_account.owner == authority.key() @ TokenError::InvalidOwner,
    )]
    pub token_account: Account<'info, TokenAccount>,
    
    #[account(
        constraint = authority.key() == token_account.owner @ TokenError::Unauthorized
    )]
    pub authority: Signer<'info>,
    
    /// Optional stake account for staking-related burns
    #[account(
        mut,
        seeds = [STAKE_ACCOUNT_SEED, authority.key().as_ref()],
        bump,
        constraint = stake_account.owner == authority.key() @ TokenError::InvalidOwner,
    )]
    pub stake_account: Option<Account<'info, StakeAccount>>,
    
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct BurnTokensParams {
    pub amount: u64,
    pub burn_type: BurnType,
    pub reason: String, // Transaction ID, card ID, or reason
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum BurnType {
    TransactionFee,      // 0.1% of all transfers
    WhaleTax,           // Progressive taxation on large holdings
    SpecialCardUsage,   // Single-use NFT consumption
    StakingPenalty,     // Early unstaking penalty
    GovernanceBurn,     // DAO-decided burns
    ManualBurn,         // User-initiated burn
}

impl BurnType {
    /// Get burn rate multiplier for different burn types
    pub fn get_rate_multiplier(&self) -> u64 {
        match self {
            BurnType::TransactionFee => 100,        // 0.1% = 1/1000
            BurnType::WhaleTax => 500,              // 0.5% base rate
            BurnType::SpecialCardUsage => 10000,    // 100% (full burn)
            BurnType::StakingPenalty => 1000,       // 1% penalty
            BurnType::GovernanceBurn => 10000,      // Variable, set by DAO
            BurnType::ManualBurn => 10000,          // User decides amount
        }
    }
    
    /// Check if burn type requires special authorization
    pub fn requires_special_auth(&self) -> bool {
        match self {
            BurnType::GovernanceBurn => true,
            _ => false,
        }
    }
}

pub fn burn_tokens(ctx: Context<BurnTokens>, params: BurnTokensParams) -> Result<()> {
    let mint_info = &mut ctx.accounts.mint_info;
    let token_account = &ctx.accounts.token_account;
    let authority = &ctx.accounts.authority;
    
    // Validate burn amount
    require!(params.amount > 0, TokenError::InvalidAmount);
    require!(
        token_account.amount >= params.amount,
        TokenError::InsufficientBalance
    );
    
    // Validate burn reason
    require!(
        params.reason.len() <= MAX_REASON_LENGTH,
        TokenError::ReasonTooLong
    );
    
    // Check burn limits based on type
    validate_burn_limits(&params, token_account.amount, mint_info)?;
    
    // Apply burn type specific logic
    match params.burn_type {
        BurnType::TransactionFee => {
            handle_transaction_fee_burn(&params, mint_info)?;
        },
        BurnType::WhaleTax => {
            handle_whale_tax_burn(&params, token_account.amount, mint_info)?;
        },
        BurnType::SpecialCardUsage => {
            handle_special_card_burn(&params, mint_info)?;
        },
        BurnType::StakingPenalty => {
            handle_staking_penalty_burn(&params, &ctx.accounts.stake_account, mint_info)?;
        },
        BurnType::GovernanceBurn => {
            handle_governance_burn(&params, mint_info)?;
        },
        BurnType::ManualBurn => {
            handle_manual_burn(&params, mint_info)?;
        },
    }
    
    // Perform the actual token burn
    let cpi_accounts = Burn {
        mint: ctx.accounts.mint.to_account_info(),
        from: ctx.accounts.token_account.to_account_info(),
        authority: authority.to_account_info(),
    };
    
    let cpi_program = ctx.accounts.token_program.to_account_info();
    let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
    
    token::burn(cpi_ctx, params.amount)?;
    
    // Update mint info statistics
    mint_info.total_burned = mint_info.total_burned
        .checked_add(params.amount)
        .ok_or(TokenError::ArithmeticOverflow)?;
    
    mint_info.burn_events = mint_info.burn_events
        .checked_add(1)
        .ok_or(TokenError::ArithmeticOverflow)?;
    
    // Update burn statistics by type
    update_burn_statistics(mint_info, &params)?;
    
    // Emit burn event
    emit!(TokenBurnEvent {
        authority: authority.key(),
        amount: params.amount,
        burn_type: params.burn_type.clone(),
        reason: params.reason.clone(),
        timestamp: Clock::get()?.unix_timestamp,
        total_burned: mint_info.total_burned,
    });
    
    msg!("Burned {} tokens of type {:?} for reason: {}", 
         params.amount, params.burn_type, params.reason);
    
    Ok(())
}

/// Validates burn limits based on burn type and current state
fn validate_burn_limits(
    params: &BurnTokensParams,
    user_balance: u64,
    mint_info: &MintInfo,
) -> Result<()> {
    let current_time = Clock::get()?.unix_timestamp;
    let time_since_last_burn = current_time
        .checked_sub(mint_info.last_burn_timestamp)
        .unwrap_or(0);
    
    match params.burn_type {
        BurnType::TransactionFee => {
            // Transaction fee burns are limited to 0.5% of transaction amount
            let max_fee_burn = user_balance
                .checked_mul(50)
                .and_then(|x| x.checked_div(10000))
                .ok_or(TokenError::ArithmeticOverflow)?;
            
            require!(
                params.amount <= max_fee_burn,
                TokenError::ExcessiveBurnAmount
            );
        },
        
        BurnType::WhaleTax => {
            // Whale tax can't exceed 5% of holdings per day
            let max_whale_tax = user_balance
                .checked_mul(500)
                .and_then(|x| x.checked_div(10000))
                .ok_or(TokenError::ArithmeticOverflow)?;
            
            require!(
                params.amount <= max_whale_tax,
                TokenError::ExcessiveBurnAmount
            );
            
            // Rate limit: max one whale tax burn per hour
            require!(
                time_since_last_burn >= 3600 || mint_info.last_burn_type != BurnType::WhaleTax,
                TokenError::BurnRateLimitExceeded
            );
        },
        
        BurnType::SpecialCardUsage => {
            // Special card burns are usually full amounts, but validate reasonable limits
            require!(
                params.amount <= MAX_SPECIAL_CARD_BURN,
                TokenError::ExcessiveBurnAmount
            );
        },
        
        BurnType::StakingPenalty => {
            // Staking penalties are limited to 10% of staked amount
            let max_penalty = user_balance
                .checked_mul(1000)
                .and_then(|x| x.checked_div(10000))
                .ok_or(TokenError::ArithmeticOverflow)?;
            
            require!(
                params.amount <= max_penalty,
                TokenError::ExcessiveBurnAmount
            );
        },
        
        BurnType::GovernanceBurn => {
            // Governance burns require special validation (handled elsewhere)
            require!(
                params.amount <= mint_info.governance_burn_limit,
                TokenError::ExceedsGovernanceLimit
            );
        },
        
        BurnType::ManualBurn => {
            // Manual burns are limited to user's balance (obviously)
            // Additional daily limit to prevent accidental large burns
            let daily_manual_limit = user_balance
                .checked_mul(2000)
                .and_then(|x| x.checked_div(10000))
                .ok_or(TokenError::ArithmeticOverflow)?; // 20% per day
            
            require!(
                params.amount <= daily_manual_limit,
                TokenError::ExcessiveBurnAmount
            );
        },
    }
    
    Ok(())
}

/// Handles transaction fee burn logic
fn handle_transaction_fee_burn(
    params: &BurnTokensParams,
    mint_info: &mut MintInfo,
) -> Result<()> {
    // Validate transaction ID format
    require!(
        params.reason.starts_with("tx_"),
        TokenError::InvalidTransactionId
    );
    
    // Update transaction fee statistics
    mint_info.total_transaction_fee_burns = mint_info.total_transaction_fee_burns
        .checked_add(params.amount)
        .ok_or(TokenError::ArithmeticOverflow)?;
    
    Ok(())
}

/// Handles whale tax burn logic
fn handle_whale_tax_burn(
    params: &BurnTokensParams,
    user_balance: u64,
    mint_info: &mut MintInfo,
) -> Result<()> {
    // Calculate progressive whale tax rate
    let whale_threshold = 100_000 * LAMPORTS_PER_TOKEN; // 100K FIN
    
    if user_balance >= whale_threshold {
        let tax_multiplier = calculate_whale_tax_multiplier(user_balance)?;
        let expected_tax = user_balance
            .checked_mul(tax_multiplier)
            .and_then(|x| x.checked_div(10000))
            .ok_or(TokenError::ArithmeticOverflow)?;
        
        // Allow some tolerance in tax calculation
        let tolerance = expected_tax
            .checked_div(100)
            .unwrap_or(0); // 1% tolerance
        
        require!(
            params.amount >= expected_tax.saturating_sub(tolerance) &&
            params.amount <= expected_tax.saturating_add(tolerance),
            TokenError::IncorrectTaxAmount
        );
    }
    
    // Update whale tax statistics
    mint_info.total_whale_tax_burns = mint_info.total_whale_tax_burns
        .checked_add(params.amount)
        .ok_or(TokenError::ArithmeticOverflow)?;
    
    Ok(())
}

/// Handles special card usage burn
fn handle_special_card_burn(
    params: &BurnTokensParams,
    mint_info: &mut MintInfo,
) -> Result<()> {
    // Validate card ID format
    require!(
        params.reason.starts_with("card_"),
        TokenError::InvalidCardId
    );
    
    // Update special card burn statistics
    mint_info.total_card_burns = mint_info.total_card_burns
        .checked_add(params.amount)
        .ok_or(TokenError::ArithmeticOverflow)?;
    
    Ok(())
}

/// Handles staking penalty burn
fn handle_staking_penalty_burn(
    params: &BurnTokensParams,
    stake_account: &Option<Account<StakeAccount>>,
    mint_info: &mut MintInfo,
) -> Result<()> {
    // Require stake account for penalty burns
    let stake_account = stake_account.as_ref()
        .ok_or(TokenError::StakeAccountRequired)?;
    
    // Validate penalty is proportional to early unstaking
    let penalty_rate = calculate_early_unstaking_penalty(
        stake_account.stake_start_time,
        stake_account.stake_duration,
    )?;
    
    let expected_penalty = stake_account.amount
        .checked_mul(penalty_rate)
        .and_then(|x| x.checked_div(10000))
        .ok_or(TokenError::ArithmeticOverflow)?;
    
    require!(
        params.amount == expected_penalty,
        TokenError::IncorrectPenaltyAmount
    );
    
    // Update penalty burn statistics
    mint_info.total_penalty_burns = mint_info.total_penalty_burns
        .checked_add(params.amount)
        .ok_or(TokenError::ArithmeticOverflow)?;
    
    Ok(())
}

/// Handles governance-decided burns
fn handle_governance_burn(
    params: &BurnTokensParams,
    mint_info: &mut MintInfo,
) -> Result<()> {
    // Validate governance proposal ID
    require!(
        params.reason.starts_with("gov_"),
        TokenError::InvalidGovernanceId
    );
    
    // Update governance burn statistics
    mint_info.total_governance_burns = mint_info.total_governance_burns
        .checked_add(params.amount)
        .ok_or(TokenError::ArithmeticOverflow)?;
    
    Ok(())
}

/// Handles manual user burns
fn handle_manual_burn(
    params: &BurnTokensParams,
    mint_info: &mut MintInfo,
) -> Result<()> {
    // Update manual burn statistics
    mint_info.total_manual_burns = mint_info.total_manual_burns
        .checked_add(params.amount)
        .ok_or(TokenError::ArithmeticOverflow)?;
    
    Ok(())
}

/// Updates burn statistics by type
fn update_burn_statistics(
    mint_info: &mut MintInfo,
    params: &BurnTokensParams,
) -> Result<()> {
    let current_time = Clock::get()?.unix_timestamp;
    
    // Update last burn info
    mint_info.last_burn_timestamp = current_time;
    mint_info.last_burn_type = params.burn_type.clone();
    mint_info.last_burn_amount = params.amount;
    
    // Update daily burn tracking
    let current_day = current_time / 86400; // Seconds per day
    let last_burn_day = mint_info.last_burn_timestamp / 86400;
    
    if current_day != last_burn_day {
        // Reset daily counters
        mint_info.daily_burn_amount = params.amount;
        mint_info.daily_burn_count = 1;
    } else {
        // Update daily counters
        mint_info.daily_burn_amount = mint_info.daily_burn_amount
            .checked_add(params.amount)
            .ok_or(TokenError::ArithmeticOverflow)?;
        
        mint_info.daily_burn_count = mint_info.daily_burn_count
            .checked_add(1)
            .ok_or(TokenError::ArithmeticOverflow)?;
    }
    
    Ok(())
}

/// Calculates whale tax multiplier based on holdings
fn calculate_whale_tax_multiplier(holdings: u64) -> Result<u64> {
    let whale_threshold = 100_000 * LAMPORTS_PER_TOKEN;
    
    if holdings < whale_threshold {
        return Ok(0);
    }
    
    // Progressive tax rates:
    // 100K-500K: 0.5%
    // 500K-1M: 1.0%  
    // 1M-5M: 1.5%
    // 5M+: 2.0%
    
    let tax_rate = if holdings < 500_000 * LAMPORTS_PER_TOKEN {
        50  // 0.5%
    } else if holdings < 1_000_000 * LAMPORTS_PER_TOKEN {
        100 // 1.0%
    } else if holdings < 5_000_000 * LAMPORTS_PER_TOKEN {
        150 // 1.5%
    } else {
        200 // 2.0%
    };
    
    Ok(tax_rate)
}

/// Calculates early unstaking penalty rate
fn calculate_early_unstaking_penalty(
    stake_start: i64,
    stake_duration: i64,
) -> Result<u64> {
    let current_time = Clock::get()?.unix_timestamp;
    let elapsed_time = current_time - stake_start;
    
    if elapsed_time >= stake_duration {
        return Ok(0); // No penalty for completed staking
    }
    
    let remaining_time = stake_duration - elapsed_time;
    let penalty_rate = (remaining_time as u64)
        .checked_mul(1000) // Base 1% penalty
        .and_then(|x| x.checked_div(stake_duration as u64))
        .ok_or(TokenError::ArithmeticOverflow)?;
    
    // Cap penalty at 10%
    Ok(penalty_rate.min(1000))
}

#[event]
pub struct TokenBurnEvent {
    #[index]
    pub authority: Pubkey,
    pub amount: u64,
    pub burn_type: BurnType,
    pub reason: String,
    pub timestamp: i64,
    pub total_burned: u64,
}

// Helper functions for burn validation
impl<'info> BurnTokens<'info> {
    /// Validates that the burn is authorized
    pub fn validate_authorization(&self, params: &BurnTokensParams) -> Result<()> {
        match params.burn_type {
            BurnType::GovernanceBurn => {
                // TODO: Add governance authorization check
                // This should verify that a valid governance proposal authorized this burn
                Ok(())
            },
            _ => Ok(())
        }
    }
    
    /// Gets the effective burn rate for the current context
    pub fn get_effective_burn_rate(&self, burn_type: &BurnType) -> u64 {
        let base_rate = burn_type.get_rate_multiplier();
        
        // Apply dynamic adjustments based on network conditions
        match burn_type {
            BurnType::TransactionFee => {
                // Could be adjusted based on network congestion
                base_rate
            },
            BurnType::WhaleTax => {
                // Could be adjusted based on token distribution
                base_rate
            },
            _ => base_rate
        }
    }
}
