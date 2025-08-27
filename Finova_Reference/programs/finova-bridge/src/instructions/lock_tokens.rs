// programs/finova-bridge/src/instructions/lock_tokens.rs

use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};
use crate::constants::*;
use crate::errors::BridgeError;
use crate::state::{BridgeConfig, LockedTokens};
use crate::cryptography::signature_verification::verify_validator_signature;
use crate::utils::{calculate_fee, generate_unique_id};

/// Lock tokens for cross-chain transfer
/// This instruction securely locks tokens on the source chain for bridging
#[derive(Accounts)]
#[instruction(amount: u64, destination_chain: String, recipient: String)]
pub struct LockTokens<'info> {
    #[account(
        mut,
        seeds = [BRIDGE_CONFIG_SEED],
        bump = bridge_config.bump,
        constraint = bridge_config.is_active @ BridgeError::BridgeInactive,
        constraint = !bridge_config.emergency_pause @ BridgeError::EmergencyPause
    )]
    pub bridge_config: Account<'info, BridgeConfig>,

    #[account(
        init,
        payer = user,
        space = LockedTokens::SPACE,
        seeds = [
            LOCKED_TOKENS_SEED,
            user.key().as_ref(),
            &generate_unique_id().to_le_bytes()
        ],
        bump
    )]
    pub locked_tokens: Account<'info, LockedTokens>,

    #[account(
        mut,
        constraint = user_token_account.owner == user.key() @ BridgeError::InvalidTokenOwner,
        constraint = user_token_account.mint == bridge_config.supported_mint @ BridgeError::UnsupportedToken
    )]
    pub user_token_account: Account<'info, TokenAccount>,

    #[account(
        mut,
        seeds = [BRIDGE_VAULT_SEED, bridge_config.supported_mint.as_ref()],
        bump = bridge_config.vault_bump,
        constraint = bridge_vault.mint == bridge_config.supported_mint @ BridgeError::InvalidVault
    )]
    pub bridge_vault: Account<'info, TokenAccount>,

    #[account(mut)]
    pub user: Signer<'info>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

impl<'info> LockTokens<'info> {
    /// Validates the lock request parameters
    pub fn validate_lock_request(
        &self,
        amount: u64,
        destination_chain: &str,
        recipient: &str,
    ) -> Result<()> {
        // Validate amount
        require!(amount > 0, BridgeError::InvalidAmount);
        require!(
            amount >= self.bridge_config.min_bridge_amount,
            BridgeError::AmountTooSmall
        );
        require!(
            amount <= self.bridge_config.max_bridge_amount,
            BridgeError::AmountTooLarge
        );

        // Validate user token balance
        require!(
            self.user_token_account.amount >= amount,
            BridgeError::InsufficientBalance
        );

        // Validate destination chain
        require!(
            !destination_chain.is_empty() && destination_chain.len() <= MAX_CHAIN_NAME_LENGTH,
            BridgeError::InvalidDestinationChain
        );
        require!(
            self.bridge_config.supported_chains.contains(&destination_chain.to_string()),
            BridgeError::UnsupportedDestinationChain
        );

        // Validate recipient address
        require!(
            !recipient.is_empty() && recipient.len() <= MAX_RECIPIENT_LENGTH,
            BridgeError::InvalidRecipient
        );

        // Check daily limits
        let current_timestamp = Clock::get()?.unix_timestamp;
        let daily_reset_time = current_timestamp - (current_timestamp % SECONDS_PER_DAY);
        
        if self.bridge_config.daily_limit_reset_time < daily_reset_time {
            // Reset daily volume if it's a new day
            self.bridge_config.daily_volume_locked = 0;
            self.bridge_config.daily_limit_reset_time = daily_reset_time;
        }

        require!(
            self.bridge_config.daily_volume_locked + amount <= self.bridge_config.daily_limit,
            BridgeError::DailyLimitExceeded
        );

        Ok(())
    }

    /// Calculates and validates bridge fee
    pub fn calculate_bridge_fee(&self, amount: u64, destination_chain: &str) -> Result<u64> {
        let base_fee = calculate_fee(
            amount,
            self.bridge_config.base_fee_rate,
            self.bridge_config.min_fee,
            self.bridge_config.max_fee,
        )?;

        // Apply chain-specific multiplier
        let chain_multiplier = self.bridge_config
            .chain_fee_multipliers
            .get(destination_chain)
            .unwrap_or(&DEFAULT_CHAIN_FEE_MULTIPLIER);

        let total_fee = base_fee
            .checked_mul(*chain_multiplier)
            .ok_or(BridgeError::MathOverflow)?
            .checked_div(FEE_PRECISION)
            .ok_or(BridgeError::MathOverflow)?;

        Ok(total_fee)
    }

    /// Transfers tokens to bridge vault
    pub fn transfer_to_vault(&self, amount: u64) -> Result<()> {
        let cpi_accounts = Transfer {
            from: self.user_token_account.to_account_info(),
            to: self.bridge_vault.to_account_info(),
            authority: self.user.to_account_info(),
        };

        let cpi_program = self.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);

        token::transfer(cpi_ctx, amount)
    }

    /// Updates bridge statistics
    pub fn update_bridge_stats(&mut self, amount: u64, fee: u64) -> Result<()> {
        self.bridge_config.total_locked = self.bridge_config
            .total_locked
            .checked_add(amount)
            .ok_or(BridgeError::MathOverflow)?;

        self.bridge_config.daily_volume_locked = self.bridge_config
            .daily_volume_locked
            .checked_add(amount)
            .ok_or(BridgeError::MathOverflow)?;

        self.bridge_config.total_fees_collected = self.bridge_config
            .total_fees_collected
            .checked_add(fee)
            .ok_or(BridgeError::MathOverflow)?;

        self.bridge_config.total_transactions = self.bridge_config
            .total_transactions
            .checked_add(1)
            .ok_or(BridgeError::MathOverflow)?;

        Ok(())
    }

    /// Initializes locked tokens account
    pub fn initialize_locked_tokens(
        &mut self,
        amount: u64,
        fee: u64,
        destination_chain: String,
        recipient: String,
        bump: u8,
    ) -> Result<()> {
        let current_timestamp = Clock::get()?.unix_timestamp;
        let unique_id = generate_unique_id();

        self.locked_tokens.set_inner(LockedTokens {
            user: self.user.key(),
            amount,
            fee,
            destination_chain: destination_chain.clone(),
            recipient: recipient.clone(),
            lock_timestamp: current_timestamp,
            unlock_timestamp: 0,
            status: crate::state::LockStatus::Locked,
            transaction_id: unique_id,
            validator_signatures: Vec::new(),
            required_confirmations: self.bridge_config.required_confirmations,
            current_confirmations: 0,
            merkle_root: [0u8; 32],
            merkle_proof: Vec::new(),
            bump,
        });

        // Emit lock event
        emit!(LockTokensEvent {
            user: self.user.key(),
            amount,
            fee,
            destination_chain,
            recipient,
            transaction_id: unique_id,
            timestamp: current_timestamp,
        });

        Ok(())
    }
}

/// Processes the token locking operation
pub fn handler(
    ctx: Context<LockTokens>,
    amount: u64,
    destination_chain: String,
    recipient: String,
) -> Result<()> {
    let accounts = &mut ctx.accounts;

    // Validate the lock request
    accounts.validate_lock_request(amount, &destination_chain, &recipient)?;

    // Calculate bridge fee
    let fee = accounts.calculate_bridge_fee(amount, &destination_chain)?;
    
    // Ensure user has enough tokens including fee
    let total_amount = amount.checked_add(fee).ok_or(BridgeError::MathOverflow)?;
    require!(
        accounts.user_token_account.amount >= total_amount,
        BridgeError::InsufficientBalance
    );

    // Transfer tokens to vault (amount + fee)
    accounts.transfer_to_vault(total_amount)?;

    // Update bridge statistics
    accounts.update_bridge_stats(amount, fee)?;

    // Initialize locked tokens account
    let bump = ctx.bumps.locked_tokens;
    accounts.initialize_locked_tokens(
        amount,
        fee,
        destination_chain,
        recipient,
        bump,
    )?;

    msg!(
        "Tokens locked successfully: amount={}, fee={}, destination={}, recipient={}",
        amount,
        fee,
        &ctx.accounts.locked_tokens.destination_chain,
        &ctx.accounts.locked_tokens.recipient
    );

    Ok(())
}

/// Event emitted when tokens are locked
#[event]
pub struct LockTokensEvent {
    pub user: Pubkey,
    pub amount: u64,
    pub fee: u64,
    pub destination_chain: String,
    pub recipient: String,
    pub transaction_id: u64,
    pub timestamp: i64,
}

/// Advanced lock tokens with additional security features
#[derive(Accounts)]
#[instruction(
    amount: u64,
    destination_chain: String,
    recipient: String,
    validator_signature: Vec<u8>
)]
pub struct LockTokensAdvanced<'info> {
    #[account(
        mut,
        seeds = [BRIDGE_CONFIG_SEED],
        bump = bridge_config.bump,
        constraint = bridge_config.is_active @ BridgeError::BridgeInactive,
        constraint = !bridge_config.emergency_pause @ BridgeError::EmergencyPause,
        constraint = bridge_config.advanced_security_enabled @ BridgeError::AdvancedSecurityRequired
    )]
    pub bridge_config: Account<'info, BridgeConfig>,

    #[account(
        init,
        payer = user,
        space = LockedTokens::SPACE,
        seeds = [
            LOCKED_TOKENS_SEED,
            user.key().as_ref(),
            &generate_unique_id().to_le_bytes()
        ],
        bump
    )]
    pub locked_tokens: Account<'info, LockedTokens>,

    #[account(
        mut,
        constraint = user_token_account.owner == user.key() @ BridgeError::InvalidTokenOwner,
        constraint = user_token_account.mint == bridge_config.supported_mint @ BridgeError::UnsupportedToken
    )]
    pub user_token_account: Account<'info, TokenAccount>,

    #[account(
        mut,
        seeds = [BRIDGE_VAULT_SEED, bridge_config.supported_mint.as_ref()],
        bump = bridge_config.vault_bump,
        constraint = bridge_vault.mint == bridge_config.supported_mint @ BridgeError::InvalidVault
    )]
    pub bridge_vault: Account<'info, TokenAccount>,

    #[account(mut)]
    pub user: Signer<'info>,

    /// Validator account for signature verification
    /// CHECK: Validated through signature verification
    pub validator: AccountInfo<'info>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

impl<'info> LockTokensAdvanced<'info> {
    /// Validates validator signature for advanced security
    pub fn validate_validator_signature(
        &self,
        amount: u64,
        destination_chain: &str,
        recipient: &str,
        signature: &[u8],
    ) -> Result<()> {
        // Check if validator is authorized
        require!(
            self.bridge_config.authorized_validators.contains(&self.validator.key()),
            BridgeError::UnauthorizedValidator
        );

        // Prepare message for signature verification
        let message = format!(
            "lock:{}:{}:{}:{}:{}",
            self.user.key(),
            amount,
            destination_chain,
            recipient,
            Clock::get()?.unix_timestamp
        );

        // Verify signature
        verify_validator_signature(
            &self.validator.key(),
            message.as_bytes(),
            signature,
        )?;

        Ok(())
    }

    /// Enhanced security checks for high-value transactions
    pub fn enhanced_security_checks(&self, amount: u64) -> Result<()> {
        // High-value transaction checks
        if amount >= self.bridge_config.high_value_threshold {
            // Require additional confirmations for high-value transfers
            require!(
                self.bridge_config.required_confirmations >= HIGH_VALUE_MIN_CONFIRMATIONS,
                BridgeError::InsufficientConfirmationsRequired
            );

            // Check if user is whitelisted for high-value transfers
            require!(
                self.bridge_config.high_value_whitelist.contains(&self.user.key()),
                BridgeError::HighValueNotWhitelisted
            );
        }

        // Rate limiting for individual users
        let current_timestamp = Clock::get()?.unix_timestamp;
        let user_key = self.user.key();
        
        if let Some(last_transaction_time) = self.bridge_config.user_last_transaction.get(&user_key) {
            let time_diff = current_timestamp - last_transaction_time;
            require!(
                time_diff >= self.bridge_config.min_transaction_interval,
                BridgeError::TransactionTooFrequent
            );
        }

        Ok(())
    }
}

/// Handler for advanced lock tokens with enhanced security
pub fn handler_advanced(
    ctx: Context<LockTokensAdvanced>,
    amount: u64,
    destination_chain: String,
    recipient: String,
    validator_signature: Vec<u8>,
) -> Result<()> {
    let accounts = &mut ctx.accounts;

    // Validate basic lock request
    accounts.validate_lock_request(amount, &destination_chain, &recipient)?;

    // Validate validator signature
    accounts.validate_validator_signature(
        amount,
        &destination_chain,
        &recipient,
        &validator_signature,
    )?;

    // Enhanced security checks
    accounts.enhanced_security_checks(amount)?;

    // Calculate bridge fee with potential discounts for verified users
    let base_fee = accounts.calculate_bridge_fee(amount, &destination_chain)?;
    let fee = if accounts.bridge_config.verified_users.contains(&accounts.user.key()) {
        base_fee
            .checked_mul(VERIFIED_USER_FEE_DISCOUNT)
            .ok_or(BridgeError::MathOverflow)?
            .checked_div(FEE_PRECISION)
            .ok_or(BridgeError::MathOverflow)?
    } else {
        base_fee
    };

    // Transfer tokens to vault
    let total_amount = amount.checked_add(fee).ok_or(BridgeError::MathOverflow)?;
    accounts.transfer_to_vault(total_amount)?;

    // Update bridge statistics and user tracking
    accounts.update_bridge_stats(amount, fee)?;
    accounts.bridge_config.user_last_transaction.insert(
        accounts.user.key(),
        Clock::get()?.unix_timestamp,
    );

    // Initialize locked tokens account with enhanced data
    let bump = ctx.bumps.locked_tokens;
    accounts.initialize_locked_tokens(
        amount,
        fee,
        destination_chain,
        recipient,
        bump,
    )?;

    // Store validator signature for verification record
    accounts.locked_tokens.validator_signatures.push(crate::state::ValidatorSignature {
        validator: accounts.validator.key(),
        signature: validator_signature,
        timestamp: Clock::get()?.unix_timestamp,
    });

    emit!(LockTokensAdvancedEvent {
        user: accounts.user.key(),
        amount,
        fee,
        destination_chain: accounts.locked_tokens.destination_chain.clone(),
        recipient: accounts.locked_tokens.recipient.clone(),
        transaction_id: accounts.locked_tokens.transaction_id,
        validator: accounts.validator.key(),
        timestamp: Clock::get()?.unix_timestamp,
        security_level: "advanced".to_string(),
    });

    msg!(
        "Advanced tokens locked successfully with enhanced security: amount={}, fee={}, validator={}",
        amount,
        fee,
        accounts.validator.key()
    );

    Ok(())
}

/// Event for advanced lock tokens operation
#[event]
pub struct LockTokensAdvancedEvent {
    pub user: Pubkey,
    pub amount: u64,
    pub fee: u64,
    pub destination_chain: String,
    pub recipient: String,
    pub transaction_id: u64,
    pub validator: Pubkey,
    pub timestamp: i64,
    pub security_level: String,
}

/// Emergency lock tokens for critical situations
pub fn emergency_lock_handler(
    ctx: Context<LockTokens>,
    amount: u64,
    destination_chain: String,
    recipient: String,
    emergency_code: String,
) -> Result<()> {
    let accounts = &mut ctx.accounts;

    // Validate emergency code
    require!(
        emergency_code == accounts.bridge_config.emergency_unlock_code,
        BridgeError::InvalidEmergencyCode
    );

    // Bypass normal validations in emergency mode
    require!(
        accounts.bridge_config.emergency_mode_enabled,
        BridgeError::EmergencyModeNotEnabled
    );

    // Process with minimal validation
    let fee = 0; // No fees in emergency mode
    accounts.transfer_to_vault(amount)?;
    
    let bump = ctx.bumps.locked_tokens;
    accounts.initialize_locked_tokens(
        amount,
        fee,
        destination_chain,
        recipient,
        bump,
    )?;

    // Mark as emergency transaction
    accounts.locked_tokens.status = crate::state::LockStatus::EmergencyLocked;

    emit!(EmergencyLockEvent {
        user: accounts.user.key(),
        amount,
        transaction_id: accounts.locked_tokens.transaction_id,
        timestamp: Clock::get()?.unix_timestamp,
    });

    msg!("Emergency lock completed for user: {}", accounts.user.key());

    Ok(())
}

/// Emergency lock event
#[event]
pub struct EmergencyLockEvent {
    pub user: Pubkey,
    pub amount: u64,
    pub transaction_id: u64,
    pub timestamp: i64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fee_calculation() {
        // Test fee calculation logic
        let amount = 1000000; // 1 FIN
        let base_fee_rate = 50; // 0.5%
        let min_fee = 1000;
        let max_fee = 100000;
        
        let fee = calculate_fee(amount, base_fee_rate, min_fee, max_fee).unwrap();
        assert_eq!(fee, 5000); // 0.5% of 1M = 5000
    }

    #[test]
    fn test_chain_multiplier() {
        let base_fee = 5000;
        let multiplier = 1500; // 1.5x
        let precision = 1000;
        
        let total_fee = base_fee * multiplier / precision;
        assert_eq!(total_fee, 7500);
    }
}
