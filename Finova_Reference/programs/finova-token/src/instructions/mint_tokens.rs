// programs/finova-token/src/instructions/mint_tokens.rs

use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount, MintTo};
use crate::state::{MintInfo, RewardPool};
use crate::errors::TokenError;
use crate::constants::*;

#[derive(Accounts)]
#[instruction(amount: u64)]
pub struct MintTokens<'info> {
    #[account(
        mut,
        seeds = [MINT_INFO_SEED],
        bump = mint_info.bump,
        has_one = authority,
    )]
    pub mint_info: Account<'info, MintInfo>,

    #[account(
        mut,
        address = mint_info.mint_address
    )]
    pub mint: Account<'info, Mint>,

    #[account(
        mut,
        token::mint = mint,
        token::authority = authority
    )]
    pub destination: Account<'info, TokenAccount>,

    #[account(
        mut,
        seeds = [REWARD_POOL_SEED],
        bump = reward_pool.bump,
    )]
    pub reward_pool: Account<'info, RewardPool>,

    /// Authority that can mint tokens (mining engine, rewards distributor, etc.)
    #[account(
        constraint = authority.key() == mint_info.authority || 
                    authority.key() == mint_info.mining_authority ||
                    authority.key() == mint_info.rewards_authority
                    @ TokenError::UnauthorizedMint
    )]
    pub authority: Signer<'info>,

    /// Mint authority PDA
    #[account(
        seeds = [MINT_AUTHORITY_SEED],
        bump = mint_info.mint_authority_bump,
    )]
    /// CHECK: This is a PDA used as mint authority
    pub mint_authority: UncheckedAccount<'info>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

impl<'info> MintTokens<'info> {
    pub fn mint_tokens(&mut self, amount: u64, mint_type: MintType) -> Result<()> {
        // Validate mint amount
        require!(amount > 0, TokenError::InvalidMintAmount);
        require!(amount <= MAX_MINT_AMOUNT, TokenError::ExceedsMaxMintAmount);

        // Check if minting is enabled
        require!(self.mint_info.is_minting_enabled, TokenError::MintingDisabled);

        // Apply rate limiting based on mint type
        self.apply_rate_limiting(amount, mint_type)?;

        // Calculate adjusted amount with regression if applicable
        let adjusted_amount = self.calculate_adjusted_amount(amount, mint_type)?;

        // Update circulating supply check
        let new_total_supply = self.mint.supply
            .checked_add(adjusted_amount)
            .ok_or(TokenError::MathOverflow)?;

        require!(
            new_total_supply <= self.mint_info.max_supply,
            TokenError::ExceedsMaxSupply
        );

        // Perform the mint operation
        self.perform_mint(adjusted_amount)?;

        // Update mint info statistics
        self.update_mint_statistics(adjusted_amount, mint_type)?;

        // Update reward pool if applicable
        if mint_type == MintType::Rewards {
            self.update_reward_pool(adjusted_amount)?;
        }

        // Emit mint event
        emit!(TokenMintEvent {
            mint: self.mint.key(),
            destination: self.destination.key(),
            authority: self.authority.key(),
            amount: adjusted_amount,
            mint_type,
            timestamp: Clock::get()?.unix_timestamp,
            new_total_supply,
        });

        Ok(())
    }

    fn apply_rate_limiting(&mut self, amount: u64, mint_type: MintType) -> Result<()> {
        let clock = Clock::get()?;
        let current_time = clock.unix_timestamp;
        
        // Reset daily limits if new day
        if current_time >= self.mint_info.daily_reset_timestamp {
            self.mint_info.daily_minted = 0;
            self.mint_info.daily_reset_timestamp = current_time + SECONDS_PER_DAY;
        }

        // Check daily mint limits
        let daily_limit = match mint_type {
            MintType::Mining => self.mint_info.daily_mining_limit,
            MintType::Rewards => self.mint_info.daily_rewards_limit,
            MintType::Staking => self.mint_info.daily_staking_limit,
            MintType::Emergency => self.mint_info.max_supply, // No daily limit for emergency
            MintType::Governance => self.mint_info.daily_governance_limit,
        };

        let new_daily_total = self.mint_info.daily_minted
            .checked_add(amount)
            .ok_or(TokenError::MathOverflow)?;

        require!(
            new_daily_total <= daily_limit,
            TokenError::ExceedsDailyLimit
        );

        // Apply per-transaction limits
        let transaction_limit = match mint_type {
            MintType::Mining => MAX_MINING_MINT_PER_TX,
            MintType::Rewards => MAX_REWARDS_MINT_PER_TX,
            MintType::Staking => MAX_STAKING_MINT_PER_TX,
            MintType::Emergency => MAX_EMERGENCY_MINT_PER_TX,
            MintType::Governance => MAX_GOVERNANCE_MINT_PER_TX,
        };

        require!(
            amount <= transaction_limit,
            TokenError::ExceedsTransactionLimit
        );

        Ok(())
    }

    fn calculate_adjusted_amount(&self, amount: u64, mint_type: MintType) -> Result<u64> {
        match mint_type {
            MintType::Mining => {
                // Apply exponential regression for mining
                let total_supply = self.mint.supply;
                let regression_factor = self.calculate_mining_regression(total_supply)?;
                
                let adjusted = (amount as f64 * regression_factor) as u64;
                Ok(adjusted.max(1)) // Ensure at least 1 token
            },
            MintType::Rewards => {
                // Apply network size regression for rewards
                let network_size = self.reward_pool.total_participants;
                let regression_factor = self.calculate_rewards_regression(network_size)?;
                
                let adjusted = (amount as f64 * regression_factor) as u64;
                Ok(adjusted.max(1))
            },
            _ => Ok(amount), // No adjustment for other types
        }
    }

    fn calculate_mining_regression(&self, total_supply: u64) -> Result<f64> {
        // Exponential regression: factor = e^(-0.001 * (total_supply / 1_000_000))
        let supply_millions = total_supply as f64 / 1_000_000.0;
        let regression_exponent = -0.001 * supply_millions;
        let regression_factor = regression_exponent.exp();
        
        // Ensure minimum factor of 0.01 (1% of original)
        Ok(regression_factor.max(0.01))
    }

    fn calculate_rewards_regression(&self, network_size: u64) -> Result<f64> {
        // Network effect regression: factor = 1 / (1 + network_size / 10_000)
        let network_factor = network_size as f64 / 10_000.0;
        let regression_factor = 1.0 / (1.0 + network_factor);
        
        // Ensure minimum factor of 0.1 (10% of original)
        Ok(regression_factor.max(0.1))
    }

    fn perform_mint(&mut self, amount: u64) -> Result<()> {
        let mint_authority_seeds = &[
            MINT_AUTHORITY_SEED,
            &[self.mint_info.mint_authority_bump],
        ];
        let signer_seeds = &[&mint_authority_seeds[..]];

        let cpi_accounts = MintTo {
            mint: self.mint.to_account_info(),
            to: self.destination.to_account_info(),
            authority: self.mint_authority.to_account_info(),
        };

        let cpi_program = self.token_program.to_account_info();
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer_seeds);

        token::mint_to(cpi_ctx, amount)
    }

    fn update_mint_statistics(&mut self, amount: u64, mint_type: MintType) -> Result<()> {
        // Update daily minted amount
        self.mint_info.daily_minted = self.mint_info.daily_minted
            .checked_add(amount)
            .ok_or(TokenError::MathOverflow)?;

        // Update total minted by type
        match mint_type {
            MintType::Mining => {
                self.mint_info.total_mining_minted = self.mint_info.total_mining_minted
                    .checked_add(amount)
                    .ok_or(TokenError::MathOverflow)?;
            },
            MintType::Rewards => {
                self.mint_info.total_rewards_minted = self.mint_info.total_rewards_minted
                    .checked_add(amount)
                    .ok_or(TokenError::MathOverflow)?;
            },
            MintType::Staking => {
                self.mint_info.total_staking_minted = self.mint_info.total_staking_minted
                    .checked_add(amount)
                    .ok_or(TokenError::MathOverflow)?;
            },
            MintType::Governance => {
                self.mint_info.total_governance_minted = self.mint_info.total_governance_minted
                    .checked_add(amount)
                    .ok_or(TokenError::MathOverflow)?;
            },
            MintType::Emergency => {
                self.mint_info.total_emergency_minted = self.mint_info.total_emergency_minted
                    .checked_add(amount)
                    .ok_or(TokenError::MathOverflow)?;
            },
        }

        // Update mint count
        self.mint_info.mint_count = self.mint_info.mint_count
            .checked_add(1)
            .ok_or(TokenError::MathOverflow)?;

        // Update last mint timestamp
        self.mint_info.last_mint_timestamp = Clock::get()?.unix_timestamp;

        Ok(())
    }

    fn update_reward_pool(&mut self, amount: u64) -> Result<()> {
        self.reward_pool.total_rewards_allocated = self.reward_pool.total_rewards_allocated
            .checked_add(amount)
            .ok_or(TokenError::MathOverflow)?;

        self.reward_pool.available_rewards = self.reward_pool.available_rewards
            .checked_add(amount)
            .ok_or(TokenError::MathOverflow)?;

        self.reward_pool.last_updated = Clock::get()?.unix_timestamp;

        Ok(())
    }
}

// Mint tokens for mining rewards
pub fn mint_mining_tokens(ctx: Context<MintTokens>, amount: u64) -> Result<()> {
    ctx.accounts.mint_tokens(amount, MintType::Mining)
}

// Mint tokens for staking rewards
pub fn mint_staking_tokens(ctx: Context<MintTokens>, amount: u64) -> Result<()> {
    ctx.accounts.mint_tokens(amount, MintType::Staking)
}

// Mint tokens for general rewards
pub fn mint_reward_tokens(ctx: Context<MintTokens>, amount: u64) -> Result<()> {
    ctx.accounts.mint_tokens(amount, MintType::Rewards)
}

// Mint tokens for governance purposes
pub fn mint_governance_tokens(ctx: Context<MintTokens>, amount: u64) -> Result<()> {
    ctx.accounts.mint_tokens(amount, MintType::Governance)
}

// Emergency mint function (requires special authority)
pub fn emergency_mint_tokens(ctx: Context<EmergencyMint>, amount: u64) -> Result<()> {
    require!(
        ctx.accounts.authority.key() == ctx.accounts.mint_info.emergency_authority,
        TokenError::UnauthorizedEmergencyMint
    );
    
    ctx.accounts.mint_tokens(amount, MintType::Emergency)
}

#[derive(Accounts)]
#[instruction(amount: u64)]
pub struct EmergencyMint<'info> {
    #[account(
        mut,
        seeds = [MINT_INFO_SEED],
        bump = mint_info.bump,
    )]
    pub mint_info: Account<'info, MintInfo>,

    #[account(
        mut,
        address = mint_info.mint_address
    )]
    pub mint: Account<'info, Mint>,

    #[account(
        mut,
        token::mint = mint,
        token::authority = authority
    )]
    pub destination: Account<'info, TokenAccount>,

    #[account(
        mut,
        seeds = [REWARD_POOL_SEED],
        bump = reward_pool.bump,
    )]
    pub reward_pool: Account<'info, RewardPool>,

    /// Emergency authority - should be multi-sig or DAO
    #[account(
        constraint = authority.key() == mint_info.emergency_authority @ TokenError::UnauthorizedEmergencyMint
    )]
    pub authority: Signer<'info>,

    #[account(
        seeds = [MINT_AUTHORITY_SEED],
        bump = mint_info.mint_authority_bump,
    )]
    /// CHECK: This is a PDA used as mint authority
    pub mint_authority: UncheckedAccount<'info>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

impl<'info> MintTokens<'info> for EmergencyMint<'info> {
    // Delegate to the main MintTokens implementation
    fn mint_tokens(&mut self, amount: u64, mint_type: MintType) -> Result<()> {
        let mint_ctx = MintTokens {
            mint_info: self.mint_info.clone(),
            mint: self.mint.clone(),
            destination: self.destination.clone(),
            reward_pool: self.reward_pool.clone(),
            authority: self.authority.clone(),
            mint_authority: self.mint_authority.clone(),
            token_program: self.token_program.clone(),
            system_program: self.system_program.clone(),
        };
        
        // Override some safety checks for emergency mints
        let mut ctx = mint_ctx;
        ctx.mint_tokens(amount, mint_type)
    }
}

// Batch mint function for efficient reward distribution
#[derive(Accounts)]
pub struct BatchMint<'info> {
    #[account(
        mut,
        seeds = [MINT_INFO_SEED],
        bump = mint_info.bump,
        has_one = authority,
    )]
    pub mint_info: Account<'info, MintInfo>,

    #[account(
        mut,
        address = mint_info.mint_address
    )]
    pub mint: Account<'info, Mint>,

    #[account(
        mut,
        seeds = [REWARD_POOL_SEED],
        bump = reward_pool.bump,
    )]
    pub reward_pool: Account<'info, RewardPool>,

    pub authority: Signer<'info>,

    #[account(
        seeds = [MINT_AUTHORITY_SEED],
        bump = mint_info.mint_authority_bump,
    )]
    /// CHECK: This is a PDA used as mint authority
    pub mint_authority: UncheckedAccount<'info>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

pub fn batch_mint_tokens(
    ctx: Context<BatchMint>, 
    recipients: Vec<Pubkey>, 
    amounts: Vec<u64>,
    mint_type: MintType
) -> Result<()> {
    require!(
        recipients.len() == amounts.len(),
        TokenError::InvalidBatchMintData
    );
    
    require!(
        recipients.len() <= MAX_BATCH_MINT_SIZE,
        TokenError::BatchSizeTooLarge
    );

    let total_amount: u64 = amounts.iter().sum();
    
    // Validate total amount
    require!(total_amount > 0, TokenError::InvalidMintAmount);
    require!(total_amount <= MAX_BATCH_MINT_AMOUNT, TokenError::ExceedsBatchMintLimit);

    // Check if we have enough allocation for batch mint
    let new_total_supply = ctx.accounts.mint.supply
        .checked_add(total_amount)
        .ok_or(TokenError::MathOverflow)?;

    require!(
        new_total_supply <= ctx.accounts.mint_info.max_supply,
        TokenError::ExceedsMaxSupply
    );

    // Update statistics for batch mint
    ctx.accounts.mint_info.daily_minted = ctx.accounts.mint_info.daily_minted
        .checked_add(total_amount)
        .ok_or(TokenError::MathOverflow)?;

    ctx.accounts.mint_info.batch_mint_count = ctx.accounts.mint_info.batch_mint_count
        .checked_add(1)
        .ok_or(TokenError::MathOverflow)?;

    // Emit batch mint event
    emit!(BatchMintEvent {
        mint: ctx.accounts.mint.key(),
        authority: ctx.accounts.authority.key(),
        recipients: recipients.clone(),
        amounts: amounts.clone(),
        total_amount,
        mint_type,
        timestamp: Clock::get()?.unix_timestamp,
    });

    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, AnchorSerialize, AnchorDeserialize)]
pub enum MintType {
    Mining,
    Rewards,
    Staking,
    Governance,
    Emergency,
}

#[event]
pub struct TokenMintEvent {
    pub mint: Pubkey,
    pub destination: Pubkey,
    pub authority: Pubkey,
    pub amount: u64,
    pub mint_type: MintType,
    pub timestamp: i64,
    pub new_total_supply: u64,
}

#[event]
pub struct BatchMintEvent {
    pub mint: Pubkey,
    pub authority: Pubkey,
    pub recipients: Vec<Pubkey>,
    pub amounts: Vec<u64>,
    pub total_amount: u64,
    pub mint_type: MintType,
    pub timestamp: i64,
}

// Constants used in this module
const MAX_MINT_AMOUNT: u64 = 1_000_000 * 10_u64.pow(9); // 1M tokens
const MAX_MINING_MINT_PER_TX: u64 = 100_000 * 10_u64.pow(9); // 100K tokens
const MAX_REWARDS_MINT_PER_TX: u64 = 500_000 * 10_u64.pow(9); // 500K tokens
const MAX_STAKING_MINT_PER_TX: u64 = 200_000 * 10_u64.pow(9); // 200K tokens
const MAX_GOVERNANCE_MINT_PER_TX: u64 = 1_000_000 * 10_u64.pow(9); // 1M tokens
const MAX_EMERGENCY_MINT_PER_TX: u64 = 10_000_000 * 10_u64.pow(9); // 10M tokens
const MAX_BATCH_MINT_SIZE: usize = 100;
const MAX_BATCH_MINT_AMOUNT: u64 = 10_000_000 * 10_u64.pow(9); // 10M tokens
const SECONDS_PER_DAY: i64 = 86400;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mining_regression_calculation() {
        // Test regression calculation at different supply levels
        let mint_info = MintInfo::default();
        
        // Test at 1M supply
        let regression_1m = calculate_mining_regression_test(1_000_000 * 10_u64.pow(9));
        assert!(regression_1m > 0.99 && regression_1m <= 1.0);
        
        // Test at 100M supply
        let regression_100m = calculate_mining_regression_test(100_000_000 * 10_u64.pow(9));
        assert!(regression_100m < regression_1m);
        
        // Test at 1B supply
        let regression_1b = calculate_mining_regression_test(1_000_000_000 * 10_u64.pow(9));
        assert!(regression_1b < regression_100m);
        assert!(regression_1b >= 0.01); // Minimum regression factor
    }

    fn calculate_mining_regression_test(total_supply: u64) -> f64 {
        let supply_millions = total_supply as f64 / 1_000_000.0;
        let regression_exponent = -0.001 * supply_millions;
        let regression_factor = regression_exponent.exp();
        regression_factor.max(0.01)
    }

    #[test]
    fn test_rewards_regression_calculation() {
        // Test network size regression
        let regression_1k = calculate_rewards_regression_test(1_000);
        let regression_10k = calculate_rewards_regression_test(10_000);
        let regression_100k = calculate_rewards_regression_test(100_000);
        
        assert!(regression_1k > regression_10k);
        assert!(regression_10k > regression_100k);
        assert!(regression_100k >= 0.1); // Minimum regression factor
    }

    fn calculate_rewards_regression_test(network_size: u64) -> f64 {
        let network_factor = network_size as f64 / 10_000.0;
        let regression_factor = 1.0 / (1.0 + network_factor);
        regression_factor.max(0.1)
    }
}
