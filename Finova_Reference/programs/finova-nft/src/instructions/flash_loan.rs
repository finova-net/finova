// programs/finova-defi/src/instructions/flash_loan.rs

use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer, Mint};
use crate::state::*;
use crate::errors::FinovaDefiError;
use crate::math::{curve::*, fees::*, oracle::*};
use crate::constants::*;
use std::collections::HashMap;

/// Flash loan instruction for borrowing tokens without collateral
/// Tokens must be repaid within the same transaction with fees
#[derive(Accounts)]
#[instruction(amount: u64, fee_rate: u64)]
pub struct InitiateFlashLoan<'info> {
    #[account(
        mut,
        seeds = [POOL_SEED, pool.token_a_mint.as_ref(), pool.token_b_mint.as_ref()],
        bump = pool.bump,
        constraint = pool.active @ FinovaDefiError::PoolInactive,
        constraint = amount > 0 @ FinovaDefiError::InvalidAmount,
        constraint = amount <= pool.total_liquidity / 10 @ FinovaDefiError::ExcessiveFlashLoanAmount,
    )]
    pub pool: Account<'info, Pool>,

    #[account(
        mut,
        constraint = pool_token_vault.mint == token_mint.key() @ FinovaDefiError::InvalidTokenMint,
        constraint = pool_token_vault.amount >= amount @ FinovaDefiError::InsufficientLiquidity,
    )]
    pub pool_token_vault: Account<'info, TokenAccount>,

    #[account(
        mut,
        constraint = borrower_token_account.mint == token_mint.key() @ FinovaDefiError::InvalidTokenMint,
        constraint = borrower_token_account.owner == borrower.key() @ FinovaDefiError::InvalidOwner,
    )]
    pub borrower_token_account: Account<'info, TokenAccount>,

    #[account(
        init,
        payer = borrower,
        space = 8 + FlashLoan::INIT_SPACE,
        seeds = [FLASH_LOAN_SEED, borrower.key().as_ref(), &pool.key().as_ref()],
        bump,
    )]
    pub flash_loan: Account<'info, FlashLoan>,

    pub token_mint: Account<'info, Mint>,

    #[account(mut)]
    pub borrower: Signer<'info>,

    pub pool_authority: AccountInfo<'info>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

/// Flash loan repayment instruction
#[derive(Accounts)]
pub struct RepayFlashLoan<'info> {
    #[account(
        mut,
        seeds = [POOL_SEED, pool.token_a_mint.as_ref(), pool.token_b_mint.as_ref()],
        bump = pool.bump,
    )]
    pub pool: Account<'info, Pool>,

    #[account(
        mut,
        constraint = pool_token_vault.mint == flash_loan.token_mint @ FinovaDefiError::InvalidTokenMint,
    )]
    pub pool_token_vault: Account<'info, TokenAccount>,

    #[account(
        mut,
        constraint = borrower_token_account.mint == flash_loan.token_mint @ FinovaDefiError::InvalidTokenMint,
        constraint = borrower_token_account.owner == borrower.key() @ FinovaDefiError::InvalidOwner,
        constraint = borrower_token_account.amount >= flash_loan.repay_amount @ FinovaDefiError::InsufficientFunds,
    )]
    pub borrower_token_account: Account<'info, TokenAccount>,

    #[account(
        mut,
        close = borrower,
        seeds = [FLASH_LOAN_SEED, borrower.key().as_ref(), &pool.key().as_ref()],
        bump = flash_loan.bump,
        constraint = flash_loan.borrower == borrower.key() @ FinovaDefiError::InvalidBorrower,
        constraint = flash_loan.active @ FinovaDefiError::FlashLoanNotActive,
        constraint = Clock::get()?.slot <= flash_loan.expiry_slot @ FinovaDefiError::FlashLoanExpired,
    )]
    pub flash_loan: Account<'info, FlashLoan>,

    #[account(
        mut,
        constraint = fee_vault.mint == flash_loan.token_mint @ FinovaDefiError::InvalidTokenMint,
    )]
    pub fee_vault: Account<'info, TokenAccount>,

    #[account(mut)]
    pub borrower: Signer<'info>,

    pub token_program: Program<'info, Token>,
}

/// Emergency flash loan liquidation for expired loans
#[derive(Accounts)]
pub struct LiquidateFlashLoan<'info> {
    #[account(
        mut,
        seeds = [POOL_SEED, pool.token_a_mint.as_ref(), pool.token_b_mint.as_ref()],
        bump = pool.bump,
    )]
    pub pool: Account<'info, Pool>,

    #[account(
        mut,
        close = liquidator,
        seeds = [FLASH_LOAN_SEED, flash_loan.borrower.as_ref(), &pool.key().as_ref()],
        bump = flash_loan.bump,
        constraint = !flash_loan.active || Clock::get()?.slot > flash_loan.expiry_slot @ FinovaDefiError::FlashLoanStillActive,
    )]
    pub flash_loan: Account<'info, FlashLoan>,

    #[account(
        mut,
        constraint = liquidator_reward_vault.mint == flash_loan.token_mint @ FinovaDefiError::InvalidTokenMint,
    )]
    pub liquidator_reward_vault: Account<'info, TokenAccount>,

    #[account(mut)]
    pub liquidator: Signer<'info>,

    pub token_program: Program<'info, Token>,
}

/// Multi-token flash loan for complex arbitrage strategies
#[derive(Accounts)]
#[instruction(loan_amounts: Vec<u64>)]
pub struct InitiateMultiFlashLoan<'info> {
    #[account(
        mut,
        seeds = [POOL_SEED, pool.token_a_mint.as_ref(), pool.token_b_mint.as_ref()],
        bump = pool.bump,
        constraint = pool.active @ FinovaDefiError::PoolInactive,
    )]
    pub pool: Account<'info, Pool>,

    #[account(
        init,
        payer = borrower,
        space = 8 + MultiFlashLoan::INIT_SPACE,
        seeds = [MULTI_FLASH_LOAN_SEED, borrower.key().as_ref(), &pool.key().as_ref()],
        bump,
    )]
    pub multi_flash_loan: Account<'info, MultiFlashLoan>,

    #[account(mut)]
    pub borrower: Signer<'info>,

    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

impl<'info> InitiateFlashLoan<'info> {
    pub fn process(&mut self, amount: u64, fee_rate: u64, bumps: &InitiateFlashLoanBumps) -> Result<()> {
        let clock = Clock::get()?;
        
        // Validate flash loan parameters
        self.validate_flash_loan_params(amount, fee_rate)?;
        
        // Calculate fees using dynamic fee structure
        let base_fee = self.calculate_base_fee(amount, fee_rate)?;
        let dynamic_fee = self.calculate_dynamic_fee(amount)?;
        let total_fee = base_fee.checked_add(dynamic_fee)
            .ok_or(FinovaDefiError::MathOverflow)?;
        
        let repay_amount = amount.checked_add(total_fee)
            .ok_or(FinovaDefiError::MathOverflow)?;

        // Initialize flash loan state
        self.flash_loan.set_inner(FlashLoan {
            borrower: self.borrower.key(),
            pool: self.pool.key(),
            token_mint: self.token_mint.key(),
            borrowed_amount: amount,
            fee_amount: total_fee,
            repay_amount,
            start_slot: clock.slot,
            expiry_slot: clock.slot.checked_add(FLASH_LOAN_EXPIRY_SLOTS)
                .ok_or(FinovaDefiError::MathOverflow)?,
            active: true,
            liquidation_threshold: self.calculate_liquidation_threshold(amount)?,
            bump: bumps.flash_loan,
        });

        // Transfer tokens to borrower
        self.transfer_to_borrower(amount)?;
        
        // Update pool statistics
        self.update_pool_flash_loan_stats(amount, total_fee)?;

        // Emit flash loan initiated event
        emit!(FlashLoanInitiated {
            borrower: self.borrower.key(),
            pool: self.pool.key(),
            token_mint: self.token_mint.key(),
            amount,
            fee_amount: total_fee,
            expiry_slot: self.flash_loan.expiry_slot,
        });

        Ok(())
    }

    fn validate_flash_loan_params(&self, amount: u64, fee_rate: u64) -> Result<()> {
        // Check minimum loan amount
        require!(amount >= MIN_FLASH_LOAN_AMOUNT, FinovaDefiError::FlashLoanTooSmall);
        
        // Check maximum fee rate
        require!(fee_rate <= MAX_FLASH_LOAN_FEE_RATE, FinovaDefiError::ExcessiveFeeRate);
        
        // Check pool utilization rate
        let utilization_rate = self.calculate_utilization_rate(amount)?;
        require!(utilization_rate <= MAX_FLASH_LOAN_UTILIZATION, FinovaDefiError::ExcessiveUtilization);
        
        // Check borrower's flash loan history
        self.validate_borrower_eligibility()?;

        Ok(())
    }

    fn calculate_base_fee(&self, amount: u64, fee_rate: u64) -> Result<u64> {
        let fee = amount
            .checked_mul(fee_rate)
            .ok_or(FinovaDefiError::MathOverflow)?
            .checked_div(FEE_RATE_DENOMINATOR)
            .ok_or(FinovaDefiError::MathOverflow)?;
        
        Ok(fee.max(MIN_FLASH_LOAN_FEE))
    }

    fn calculate_dynamic_fee(&self, amount: u64) -> Result<u64> {
        // Dynamic fee based on pool utilization and market volatility
        let utilization_rate = self.calculate_utilization_rate(amount)?;
        let volatility_multiplier = self.get_volatility_multiplier()?;
        
        let dynamic_fee_rate = BASE_DYNAMIC_FEE_RATE
            .checked_add(utilization_rate.checked_mul(UTILIZATION_FEE_MULTIPLIER)
                .ok_or(FinovaDefiError::MathOverflow)?)
            .ok_or(FinovaDefiError::MathOverflow)?
            .checked_mul(volatility_multiplier)
            .ok_or(FinovaDefiError::MathOverflow)?
            .checked_div(VOLATILITY_DENOMINATOR)
            .ok_or(FinovaDefiError::MathOverflow)?;

        let dynamic_fee = amount
            .checked_mul(dynamic_fee_rate)
            .ok_or(FinovaDefiError::MathOverflow)?
            .checked_div(FEE_RATE_DENOMINATOR)
            .ok_or(FinovaDefiError::MathOverflow)?;

        Ok(dynamic_fee)
    }

    fn calculate_utilization_rate(&self, loan_amount: u64) -> Result<u64> {
        let total_available = self.pool_token_vault.amount;
        if total_available == 0 {
            return Ok(0);
        }

        let utilization = loan_amount
            .checked_mul(RATE_PRECISION)
            .ok_or(FinovaDefiError::MathOverflow)?
            .checked_div(total_available)
            .ok_or(FinovaDefiError::MathOverflow)?;

        Ok(utilization)
    }

    fn get_volatility_multiplier(&self) -> Result<u64> {
        // In a real implementation, this would query an oracle for price volatility
        // For now, we'll use a default multiplier
        Ok(DEFAULT_VOLATILITY_MULTIPLIER)
    }

    fn calculate_liquidation_threshold(&self, amount: u64) -> Result<u64> {
        let threshold_rate = LIQUIDATION_THRESHOLD_RATE
            .checked_add(self.calculate_utilization_rate(amount)?.checked_div(10)
                .ok_or(FinovaDefiError::MathOverflow)?)
            .ok_or(FinovaDefiError::MathOverflow)?;

        let threshold = amount
            .checked_mul(threshold_rate)
            .ok_or(FinovaDefiError::MathOverflow)?
            .checked_div(RATE_PRECISION)
            .ok_or(FinovaDefiError::MathOverflow)?;

        Ok(threshold)
    }

    fn validate_borrower_eligibility(&self) -> Result<()> {
        // Check if borrower has any active flash loans
        // In a real implementation, this would query existing flash loan accounts
        // For now, we'll allow the loan to proceed
        Ok(())
    }

    fn transfer_to_borrower(&mut self, amount: u64) -> Result<()> {
        let pool_key = self.pool.key();
        let seeds = &[
            POOL_SEED,
            self.pool.token_a_mint.as_ref(),
            self.pool.token_b_mint.as_ref(),
            &[self.pool.bump],
        ];
        let signer = &[&seeds[..]];

        token::transfer(
            CpiContext::new_with_signer(
                self.token_program.to_account_info(),
                Transfer {
                    from: self.pool_token_vault.to_account_info(),
                    to: self.borrower_token_account.to_account_info(),
                    authority: self.pool_authority.to_account_info(),
                },
                signer,
            ),
            amount,
        )?;

        Ok(())
    }

    fn update_pool_flash_loan_stats(&mut self, amount: u64, fee: u64) -> Result<()> {
        self.pool.total_flash_loans = self.pool.total_flash_loans
            .checked_add(1)
            .ok_or(FinovaDefiError::MathOverflow)?;
        
        self.pool.total_flash_loan_volume = self.pool.total_flash_loan_volume
            .checked_add(amount)
            .ok_or(FinovaDefiError::MathOverflow)?;
        
        self.pool.total_flash_loan_fees = self.pool.total_flash_loan_fees
            .checked_add(fee)
            .ok_or(FinovaDefiError::MathOverflow)?;

        Ok(())
    }
}

impl<'info> RepayFlashLoan<'info> {
    pub fn process(&mut self) -> Result<()> {
        // Validate repayment conditions
        self.validate_repayment()?;
        
        // Calculate actual repayment amounts
        let (principal_amount, fee_amount) = self.calculate_repayment_amounts()?;
        
        // Transfer principal back to pool
        self.transfer_principal(principal_amount)?;
        
        // Transfer fees to fee vault
        self.transfer_fees(fee_amount)?;
        
        // Update pool balances
        self.update_pool_balances(principal_amount, fee_amount)?;
        
        // Mark flash loan as repaid
        self.flash_loan.active = false;

        // Emit repayment event
        emit!(FlashLoanRepaid {
            borrower: self.borrower.key(),
            pool: self.pool.key(),
            token_mint: self.flash_loan.token_mint,
            principal_amount,
            fee_amount,
            total_repaid: self.flash_loan.repay_amount,
        });

        Ok(())
    }

    fn validate_repayment(&self) -> Result<()> {
        let clock = Clock::get()?;
        
        // Check if flash loan is still within expiry time
        require!(clock.slot <= self.flash_loan.expiry_slot, FinovaDefiError::FlashLoanExpired);
        
        // Verify sufficient balance for repayment
        require!(
            self.borrower_token_account.amount >= self.flash_loan.repay_amount,
            FinovaDefiError::InsufficientFunds
        );

        Ok(())
    }

    fn calculate_repayment_amounts(&self) -> Result<(u64, u64)> {
        let principal = self.flash_loan.borrowed_amount;
        let fee = self.flash_loan.fee_amount;
        
        // Apply any late fees if applicable
        let late_fee = self.calculate_late_fee()?;
        let total_fee = fee.checked_add(late_fee)
            .ok_or(FinovaDefiError::MathOverflow)?;

        Ok((principal, total_fee))
    }

    fn calculate_late_fee(&self) -> Result<u64> {
        let clock = Clock::get()?;
        let slots_elapsed = clock.slot.saturating_sub(self.flash_loan.start_slot);
        
        if slots_elapsed <= FLASH_LOAN_GRACE_PERIOD_SLOTS {
            return Ok(0);
        }

        let late_slots = slots_elapsed.saturating_sub(FLASH_LOAN_GRACE_PERIOD_SLOTS);
        let late_fee_rate = late_slots
            .checked_mul(LATE_FEE_RATE_PER_SLOT)
            .ok_or(FinovaDefiError::MathOverflow)?
            .min(MAX_LATE_FEE_RATE);

        let late_fee = self.flash_loan.borrowed_amount
            .checked_mul(late_fee_rate)
            .ok_or(FinovaDefiError::MathOverflow)?
            .checked_div(FEE_RATE_DENOMINATOR)
            .ok_or(FinovaDefiError::MathOverflow)?;

        Ok(late_fee)
    }

    fn transfer_principal(&mut self, amount: u64) -> Result<()> {
        token::transfer(
            CpiContext::new(
                self.token_program.to_account_info(),
                Transfer {
                    from: self.borrower_token_account.to_account_info(),
                    to: self.pool_token_vault.to_account_info(),
                    authority: self.borrower.to_account_info(),
                },
            ),
            amount,
        )?;

        Ok(())
    }

    fn transfer_fees(&mut self, amount: u64) -> Result<()> {
        if amount > 0 {
            token::transfer(
                CpiContext::new(
                    self.token_program.to_account_info(),
                    Transfer {
                        from: self.borrower_token_account.to_account_info(),
                        to: self.fee_vault.to_account_info(),
                        authority: self.borrower.to_account_info(),
                    },
                ),
                amount,
            )?;
        }

        Ok(())
    }

    fn update_pool_balances(&mut self, principal: u64, fee: u64) -> Result<()> {
        // Update pool statistics
        self.pool.successful_flash_loans = self.pool.successful_flash_loans
            .checked_add(1)
            .ok_or(FinovaDefiError::MathOverflow)?;
        
        self.pool.total_fees_collected = self.pool.total_fees_collected
            .checked_add(fee)
            .ok_or(FinovaDefiError::MathOverflow)?;

        Ok(())
    }
}

impl<'info> LiquidateFlashLoan<'info> {
    pub fn process(&mut self) -> Result<()> {
        // Validate liquidation conditions
        self.validate_liquidation()?;
        
        // Calculate liquidation rewards
        let liquidation_reward = self.calculate_liquidation_reward()?;
        
        // Transfer liquidation reward to liquidator
        self.transfer_liquidation_reward(liquidation_reward)?;
        
        // Update pool bad debt
        self.update_pool_bad_debt()?;
        
        // Emit liquidation event
        emit!(FlashLoanLiquidated {
            borrower: self.flash_loan.borrower,
            liquidator: self.liquidator.key(),
            pool: self.pool.key(),
            borrowed_amount: self.flash_loan.borrowed_amount,
            liquidation_reward,
        });

        Ok(())
    }

    fn validate_liquidation(&self) -> Result<()> {
        let clock = Clock::get()?;
        
        // Check if flash loan has expired or is inactive
        require!(
            !self.flash_loan.active || clock.slot > self.flash_loan.expiry_slot,
            FinovaDefiError::FlashLoanStillActive
        );

        Ok(())
    }

    fn calculate_liquidation_reward(&self) -> Result<u64> {
        let base_reward = self.flash_loan.borrowed_amount
            .checked_mul(LIQUIDATION_REWARD_RATE)
            .ok_or(FinovaDefiError::MathOverflow)?
            .checked_div(RATE_PRECISION)
            .ok_or(FinovaDefiError::MathOverflow)?;

        Ok(base_reward.min(MAX_LIQUIDATION_REWARD))
    }

    fn transfer_liquidation_reward(&mut self, reward: u64) -> Result<()> {
        if reward > 0 {
            let pool_key = self.pool.key();
            let seeds = &[
                POOL_SEED,
                self.pool.token_a_mint.as_ref(),
                self.pool.token_b_mint.as_ref(),
                &[self.pool.bump],
            ];
            let signer = &[&seeds[..]];

            token::transfer(
                CpiContext::new_with_signer(
                    self.liquidator_reward_vault.programs.as_ref().unwrap().to_account_info(),
                    Transfer {
                        from: self.pool.token_vault.to_account_info(),
                        to: self.liquidator_reward_vault.to_account_info(),
                        authority: self.pool.authority.to_account_info(),
                    },
                    signer,
                ),
                reward,
            )?;
        }

        Ok(())
    }

    fn update_pool_bad_debt(&mut self) -> Result<()> {
        self.pool.total_bad_debt = self.pool.total_bad_debt
            .checked_add(self.flash_loan.borrowed_amount)
            .ok_or(FinovaDefiError::MathOverflow)?;
        
        self.pool.liquidated_flash_loans = self.pool.liquidated_flash_loans
            .checked_add(1)
            .ok_or(FinovaDefiError::MathOverflow)?;

        Ok(())
    }
}

impl<'info> InitiateMultiFlashLoan<'info> {
    pub fn process(&mut self, loan_amounts: Vec<u64>, bumps: &InitiateMultiFlashLoanBumps) -> Result<()> {
        let clock = Clock::get()?;
        
        // Validate multi-flash loan parameters
        self.validate_multi_loan_params(&loan_amounts)?;
        
        // Calculate total fees for all loans
        let mut total_fees = HashMap::new();
        let mut total_repay_amounts = HashMap::new();
        
        for (i, &amount) in loan_amounts.iter().enumerate() {
            let fee = self.calculate_multi_loan_fee(amount, i)?;
            let repay_amount = amount.checked_add(fee)
                .ok_or(FinovaDefiError::MathOverflow)?;
            
            total_fees.insert(i, fee);
            total_repay_amounts.insert(i, repay_amount);
        }

        // Initialize multi-flash loan state
        self.multi_flash_loan.set_inner(MultiFlashLoan {
            borrower: self.borrower.key(),
            pool: self.pool.key(),
            loan_count: loan_amounts.len() as u8,
            start_slot: clock.slot,
            expiry_slot: clock.slot.checked_add(MULTI_FLASH_LOAN_EXPIRY_SLOTS)
                .ok_or(FinovaDefiError::MathOverflow)?,
            active: true,
            bump: bumps.multi_flash_loan,
        });

        // Emit multi-flash loan initiated event
        emit!(MultiFlashLoanInitiated {
            borrower: self.borrower.key(),
            pool: self.pool.key(),
            loan_count: loan_amounts.len() as u8,
            total_amount: loan_amounts.iter().sum::<u64>(),
            expiry_slot: self.multi_flash_loan.expiry_slot,
        });

        Ok(())
    }

    fn validate_multi_loan_params(&self, loan_amounts: &[u64]) -> Result<()> {
        // Check maximum number of concurrent loans
        require!(
            loan_amounts.len() <= MAX_MULTI_FLASH_LOANS as usize,
            FinovaDefiError::TooManyFlashLoans
        );
        
        // Validate each loan amount
        for &amount in loan_amounts {
            require!(amount >= MIN_FLASH_LOAN_AMOUNT, FinovaDefiError::FlashLoanTooSmall);
        }
        
        // Check total utilization
        let total_amount: u64 = loan_amounts.iter().sum();
        let utilization_rate = self.calculate_total_utilization_rate(total_amount)?;
        require!(
            utilization_rate <= MAX_MULTI_FLASH_LOAN_UTILIZATION,
            FinovaDefiError::ExcessiveMultiLoanUtilization
        );

        Ok(())
    }

    fn calculate_multi_loan_fee(&self, amount: u64, loan_index: usize) -> Result<u64> {
        let base_fee_rate = MULTI_FLASH_LOAN_BASE_FEE_RATE
            .checked_add(loan_index as u64 * MULTI_LOAN_INCREMENT_RATE)
            .ok_or(FinovaDefiError::MathOverflow)?;
        
        let fee = amount
            .checked_mul(base_fee_rate)
            .ok_or(FinovaDefiError::MathOverflow)?
            .checked_div(FEE_RATE_DENOMINATOR)
            .ok_or(FinovaDefiError::MathOverflow)?;

        Ok(fee.max(MIN_FLASH_LOAN_FEE))
    }

    fn calculate_total_utilization_rate(&self, total_amount: u64) -> Result<u64> {
        let total_available = self.pool.total_liquidity;
        if total_available == 0 {
            return Ok(0);
        }

        let utilization = total_amount
            .checked_mul(RATE_PRECISION)
            .ok_or(FinovaDefiError::MathOverflow)?
            .checked_div(total_available)
            .ok_or(FinovaDefiError::MathOverflow)?;

        Ok(utilization)
    }
}

// Flash loan events
#[event]
pub struct FlashLoanInitiated {
    pub borrower: Pubkey,
    pub pool: Pubkey,
    pub token_mint: Pubkey,
    pub amount: u64,
    pub fee_amount: u64,
    pub expiry_slot: u64,
}

#[event]
pub struct FlashLoanRepaid {
    pub borrower: Pubkey,
    pub pool: Pubkey,
    pub token_mint: Pubkey,
    pub principal_amount: u64,
    pub fee_amount: u64,
    pub total_repaid: u64,
}

#[event]
pub struct FlashLoanLiquidated {
    pub borrower: Pubkey,
    pub liquidator: Pubkey,
    pub pool: Pubkey,
    pub borrowed_amount: u64,
    pub liquidation_reward: u64,
}

#[event]
pub struct MultiFlashLoanInitiated {
    pub borrower: Pubkey,
    pub pool: Pubkey,
    pub loan_count: u8,
    pub total_amount: u64,
    pub expiry_slot: u64,
}

// Flash loan constants
pub const FLASH_LOAN_SEED: &[u8] = b"flash_loan";
pub const MULTI_FLASH_LOAN_SEED: &[u8] = b"multi_flash_loan";
pub const FLASH_LOAN_EXPIRY_SLOTS: u64 = 432000; // ~1 day at 400ms blocks
pub const MULTI_FLASH_LOAN_EXPIRY_SLOTS: u64 = 216000; // ~12 hours
pub const FLASH_LOAN_GRACE_PERIOD_SLOTS: u64 = 21600; // ~2 hours
pub const MIN_FLASH_LOAN_AMOUNT: u64 = 1_000_000; // 1 token (6 decimals)
pub const MIN_FLASH_LOAN_FEE: u64 = 1000; // 0.001 token
pub const MAX_FLASH_LOAN_FEE_RATE: u64 = 1000; // 10%
pub const MAX_FLASH_LOAN_UTILIZATION: u64 = 1000; // 10%
pub const MAX_MULTI_FLASH_LOAN_UTILIZATION: u64 = 2000; // 20%
pub const MAX_MULTI_FLASH_LOANS: u8 = 10;
pub const BASE_DYNAMIC_FEE_RATE: u64 = 5; // 0.05%
pub const UTILIZATION_FEE_MULTIPLIER: u64 = 2;
pub const DEFAULT_VOLATILITY_MULTIPLIER: u64 = 1000;
pub const VOLATILITY_DENOMINATOR: u64 = 1000;
pub const LIQUIDATION_THRESHOLD_RATE: u64 = 8000; // 80%
pub const LIQUIDATION_REWARD_RATE: u64 = 500; // 5%
pub const MAX_LIQUIDATION_REWARD: u64 = 10_000_000; // 10 tokens
pub const LATE_FEE_RATE_PER_SLOT: u64 = 1; // 0.01% per slot
pub const MAX_LATE_FEE_RATE: u64 = 2000; // 20%
pub const MULTI_FLASH_LOAN_BASE_FEE_RATE: u64 = 30; // 0.3%
pub const MULTI_LOAN_INCREMENT_RATE: u64 = 5; // 0.05% per additional loan

// Flash loan state definitions
#[account]
pub struct FlashLoan {
    /// The borrower's public key
    pub borrower: Pubkey,
    /// The pool from which tokens were borrowed
    pub pool: Pubkey,
    /// The mint of the borrowed token
    pub token_mint: Pubkey,
    /// Amount borrowed (principal)
    pub borrowed_amount: u64,
    /// Fee amount to be paid
    pub fee_amount: u64,
    /// Total amount to be repaid (principal + fee)
    pub repay_amount: u64,
    /// Slot when the loan was initiated
    pub start_slot: u64,
    /// Slot when the loan expires
    pub expiry_slot: u64,
    /// Whether the loan is currently active
    pub active: bool,
    /// Liquidation threshold for the loan
    pub liquidation_threshold: u64,
    /// PDA bump seed
    pub bump: u8,
}

impl FlashLoan {
    pub const INIT_SPACE: usize = 32 + 32 + 32 + 8 + 8 + 8 + 8 + 8 + 1 + 8 + 1;
}

#[account]
pub struct MultiFlashLoan {
    /// The borrower's public key
    pub borrower: Pubkey,
    /// The pool involved in the multi-loan
    pub pool: Pubkey,
    /// Number of concurrent loans
    pub loan_count: u8,
    /// Slot when the multi-loan was initiated
    pub start_slot: u64,
    /// Slot when the multi-loan expires
    pub expiry_slot: u64,
    /// Whether the multi-loan is currently active
    pub active: bool,
    /// PDA bump seed
    pub bump: u8,
}

impl MultiFlashLoan {
    pub const INIT_SPACE: usize = 32 + 32 + 1 + 8 + 8 + 1 + 1;
}

// Advanced flash loan utilities
pub struct FlashLoanValidator;

impl FlashLoanValidator {
    /// Validates that a flash loan can be safely executed
    pub fn validate_loan_safety(
        pool: &Pool,
        amount: u64,
        current_utilization: u64,
    ) -> Result<()> {
        // Check pool health metrics
        let health_score = Self::calculate_pool_health(pool)?;
        require!(
            health_score >= MIN_POOL_HEALTH_SCORE,
            FinovaDefiError::PoolUnhealthy
        );

        // Validate against maximum single loan size
        let max_single_loan = pool.total_liquidity
            .checked_mul(MAX_SINGLE_FLASH_LOAN_RATIO)
            .ok_or(FinovaDefiError::MathOverflow)?
            .checked_div(RATIO_PRECISION)
            .ok_or(FinovaDefiError::MathOverflow)?;

        require!(amount <= max_single_loan, FinovaDefiError::ExcessiveFlashLoanAmount);

        // Check utilization limits
        let new_utilization = current_utilization
            .checked_add(amount.checked_mul(RATE_PRECISION)
                .ok_or(FinovaDefiError::MathOverflow)?
                .checked_div(pool.total_liquidity)
                .ok_or(FinovaDefiError::MathOverflow)?)
            .ok_or(FinovaDefiError::MathOverflow)?;

        require!(
            new_utilization <= MAX_POOL_UTILIZATION_FOR_FLASH_LOANS,
            FinovaDefiError::ExcessivePoolUtilization
        );

        Ok(())
    }

    /// Calculates the health score of a pool for flash loan eligibility
    pub fn calculate_pool_health(pool: &Pool) -> Result<u64> {
        let mut health_score = BASE_HEALTH_SCORE;

        // Factor in liquidity depth
        let liquidity_score = if pool.total_liquidity > HIGH_LIQUIDITY_THRESHOLD {
            LIQUIDITY_BONUS_HIGH
        } else if pool.total_liquidity > MEDIUM_LIQUIDITY_THRESHOLD {
            LIQUIDITY_BONUS_MEDIUM
        } else {
            0
        };

        health_score = health_score
            .checked_add(liquidity_score)
            .ok_or(FinovaDefiError::MathOverflow)?;

        // Factor in volume consistency
        let volume_consistency = Self::calculate_volume_consistency(pool)?;
        health_score = health_score
            .checked_add(volume_consistency)
            .ok_or(FinovaDefiError::MathOverflow)?;

        // Penalize for bad debt ratio
        let bad_debt_ratio = if pool.total_liquidity > 0 {
            pool.total_bad_debt
                .checked_mul(RATIO_PRECISION)
                .ok_or(FinovaDefiError::MathOverflow)?
                .checked_div(pool.total_liquidity)
                .ok_or(FinovaDefiError::MathOverflow)?
        } else {
            0
        };

        let bad_debt_penalty = bad_debt_ratio
            .checked_mul(BAD_DEBT_PENALTY_MULTIPLIER)
            .ok_or(FinovaDefiError::MathOverflow)?;

        health_score = health_score.saturating_sub(bad_debt_penalty);

        Ok(health_score.min(MAX_HEALTH_SCORE))
    }

    /// Calculates volume consistency score for pool health
    fn calculate_volume_consistency(pool: &Pool) -> Result<u64> {
        // In a real implementation, this would analyze historical volume data
        // For now, we'll use a simplified calculation based on current metrics
        let base_consistency = if pool.total_volume > 0 {
            VOLUME_CONSISTENCY_BASE
        } else {
            0
        };

        Ok(base_consistency)
    }

    /// Validates arbitrage opportunity for flash loans
    pub fn validate_arbitrage_opportunity(
        token_mint: &Pubkey,
        amount: u64,
        expected_profit: u64,
    ) -> Result<()> {
        // Minimum profit threshold
        let min_profit = amount
            .checked_mul(MIN_ARBITRAGE_PROFIT_RATE)
            .ok_or(FinovaDefiError::MathOverflow)?
            .checked_div(RATE_PRECISION)
            .ok_or(FinovaDefiError::MathOverflow)?;

        require!(
            expected_profit >= min_profit,
            FinovaDefiError::InsufficientArbitrageProfit
        );

        Ok(())
    }
}

// Flash loan risk management
pub struct FlashLoanRiskManager;

impl FlashLoanRiskManager {
    /// Calculates risk-adjusted fee for flash loans
    pub fn calculate_risk_adjusted_fee(
        pool: &Pool,
        amount: u64,
        borrower_history: &BorrowerHistory,
    ) -> Result<u64> {
        let base_fee = amount
            .checked_mul(BASE_FLASH_LOAN_FEE_RATE)
            .ok_or(FinovaDefiError::MathOverflow)?
            .checked_div(FEE_RATE_DENOMINATOR)
            .ok_or(FinovaDefiError::MathOverflow)?;

        // Risk multiplier based on borrower history
        let risk_multiplier = Self::calculate_borrower_risk_multiplier(borrower_history)?;
        
        // Pool risk multiplier
        let pool_risk_multiplier = Self::calculate_pool_risk_multiplier(pool)?;
        
        // Size risk multiplier
        let size_risk_multiplier = Self::calculate_size_risk_multiplier(amount, pool.total_liquidity)?;

        let total_multiplier = risk_multiplier
            .checked_mul(pool_risk_multiplier)
            .ok_or(FinovaDefiError::MathOverflow)?
            .checked_mul(size_risk_multiplier)
            .ok_or(FinovaDefiError::MathOverflow)?
            .checked_div(RISK_MULTIPLIER_PRECISION.pow(2))
            .ok_or(FinovaDefiError::MathOverflow)?;

        let risk_adjusted_fee = base_fee
            .checked_mul(total_multiplier)
            .ok_or(FinovaDefiError::MathOverflow)?
            .checked_div(RISK_MULTIPLIER_PRECISION)
            .ok_or(FinovaDefiError::MathOverflow)?;

        Ok(risk_adjusted_fee.min(MAX_RISK_ADJUSTED_FEE))
    }

    fn calculate_borrower_risk_multiplier(history: &BorrowerHistory) -> Result<u64> {
        let mut multiplier = RISK_MULTIPLIER_PRECISION;

        // Good history reduces fees
        if history.successful_loans > GOOD_BORROWER_THRESHOLD {
            multiplier = multiplier
                .checked_mul(GOOD_BORROWER_DISCOUNT)
                .ok_or(FinovaDefiError::MathOverflow)?
                .checked_div(DISCOUNT_PRECISION)
                .ok_or(FinovaDefiError::MathOverflow)?;
        }

        // Bad history increases fees
        if history.defaulted_loans > 0 {
            let penalty = history.defaulted_loans
                .checked_mul(DEFAULT_PENALTY_PER_LOAN)
                .ok_or(FinovaDefiError::MathOverflow)?;
            
            multiplier = multiplier
                .checked_add(penalty)
                .ok_or(FinovaDefiError::MathOverflow)?;
        }

        Ok(multiplier.min(MAX_BORROWER_RISK_MULTIPLIER))
    }

    fn calculate_pool_risk_multiplier(pool: &Pool) -> Result<u64> {
        let mut multiplier = RISK_MULTIPLIER_PRECISION;

        // High utilization increases fees
        let utilization_rate = if pool.total_liquidity > 0 {
            pool.total_liquidity
                .checked_sub(pool.available_liquidity)
                .unwrap_or(0)
                .checked_mul(RATE_PRECISION)
                .ok_or(FinovaDefiError::MathOverflow)?
                .checked_div(pool.total_liquidity)
                .ok_or(FinovaDefiError::MathOverflow)?
        } else {
            0
        };

        if utilization_rate > HIGH_UTILIZATION_THRESHOLD {
            let utilization_penalty = utilization_rate
                .checked_sub(HIGH_UTILIZATION_THRESHOLD)
                .unwrap_or(0)
                .checked_mul(UTILIZATION_PENALTY_MULTIPLIER)
                .ok_or(FinovaDefiError::MathOverflow)?
                .checked_div(RATE_PRECISION)
                .ok_or(FinovaDefiError::MathOverflow)?;

            multiplier = multiplier
                .checked_add(utilization_penalty)
                .ok_or(FinovaDefiError::MathOverflow)?;
        }

        Ok(multiplier.min(MAX_POOL_RISK_MULTIPLIER))
    }

    fn calculate_size_risk_multiplier(amount: u64, total_liquidity: u64) -> Result<u64> {
        if total_liquidity == 0 {
            return Ok(RISK_MULTIPLIER_PRECISION);
        }

        let size_ratio = amount
            .checked_mul(RATIO_PRECISION)
            .ok_or(FinovaDefiError::MathOverflow)?
            .checked_div(total_liquidity)
            .ok_or(FinovaDefiError::MathOverflow)?;

        let multiplier = if size_ratio > LARGE_LOAN_THRESHOLD {
            let size_penalty = size_ratio
                .checked_sub(LARGE_LOAN_THRESHOLD)
                .unwrap_or(0)
                .checked_mul(SIZE_PENALTY_MULTIPLIER)
                .ok_or(FinovaDefiError::MathOverflow)?
                .checked_div(RATIO_PRECISION)
                .ok_or(FinovaDefiError::MathOverflow)?;

            RISK_MULTIPLIER_PRECISION
                .checked_add(size_penalty)
                .ok_or(FinovaDefiError::MathOverflow)?
        } else {
            RISK_MULTIPLIER_PRECISION
        };

        Ok(multiplier.min(MAX_SIZE_RISK_MULTIPLIER))
    }
}

// Borrower history tracking
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default)]
pub struct BorrowerHistory {
    pub total_loans: u64,
    pub successful_loans: u64,
    pub defaulted_loans: u64,
    pub total_volume: u64,
    pub average_loan_size: u64,
    pub last_loan_slot: u64,
}

// Additional constants for risk management
pub const MIN_POOL_HEALTH_SCORE: u64 = 5000;
pub const BASE_HEALTH_SCORE: u64 = 5000;
pub const MAX_HEALTH_SCORE: u64 = 10000;
pub const HIGH_LIQUIDITY_THRESHOLD: u64 = 1_000_000_000; // 1000 tokens
pub const MEDIUM_LIQUIDITY_THRESHOLD: u64 = 100_000_000; // 100 tokens
pub const LIQUIDITY_BONUS_HIGH: u64 = 2000;
pub const LIQUIDITY_BONUS_MEDIUM: u64 = 1000;
pub const VOLUME_CONSISTENCY_BASE: u64 = 1000;
pub const BAD_DEBT_PENALTY_MULTIPLIER: u64 = 10;
pub const MAX_SINGLE_FLASH_LOAN_RATIO: u64 = 1000; // 10%
pub const RATIO_PRECISION: u64 = 10000;
pub const MAX_POOL_UTILIZATION_FOR_FLASH_LOANS: u64 = 8000; // 80%
pub const MIN_ARBITRAGE_PROFIT_RATE: u64 = 50; // 0.5%
pub const BASE_FLASH_LOAN_FEE_RATE: u64 = 30; // 0.3%
pub const RISK_MULTIPLIER_PRECISION: u64 = 10000;
pub const MAX_RISK_ADJUSTED_FEE: u64 = 1_000_000; // 1 token max fee
pub const GOOD_BORROWER_THRESHOLD: u64 = 10;
pub const GOOD_BORROWER_DISCOUNT: u64 = 8000; // 20% discount
pub const DISCOUNT_PRECISION: u64 = 10000;
pub const DEFAULT_PENALTY_PER_LOAN: u64 = 1000;
pub const MAX_BORROWER_RISK_MULTIPLIER: u64 = 50000; // 5x max
pub const HIGH_UTILIZATION_THRESHOLD: u64 = 7000; // 70%
pub const UTILIZATION_PENALTY_MULTIPLIER: u64 = 2;
pub const MAX_POOL_RISK_MULTIPLIER: u64 = 30000; // 3x max
pub const LARGE_LOAN_THRESHOLD: u64 = 500; // 5%
pub const SIZE_PENALTY_MULTIPLIER: u64 = 2;
pub const MAX_SIZE_RISK_MULTIPLIER: u64 = 20000; // 2x max
