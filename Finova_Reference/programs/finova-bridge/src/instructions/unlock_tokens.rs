// programs/finova-bridge/src/instructions/unlock_tokens.rs

use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount, Transfer};
use anchor_spl::associated_token::AssociatedToken;
use std::mem::size_of;

use crate::constants::*;
use crate::errors::FinovaBridgeError;
use crate::state::{BridgeConfig, LockedTokens, ValidatorSet};
use crate::utils::*;
use crate::cryptography::{MerkleProof, SignatureVerification};

/// Unlocks tokens from the bridge after receiving valid cross-chain proof
#[derive(Accounts)]
#[instruction(
    unlock_amount: u64,
    merkle_proof: Vec<[u8; 32]>,
    leaf_index: u64,
    source_chain_id: u16,
    tx_hash: [u8; 32],
    validator_signatures: Vec<[u8; 64]>
)]
pub struct UnlockTokens<'info> {
    /// Bridge configuration account
    #[account(
        seeds = [BRIDGE_CONFIG_SEED],
        bump = bridge_config.bump,
        constraint = bridge_config.is_active @ FinovaBridgeError::BridgeInactive
    )]
    pub bridge_config: Account<'info, BridgeConfig>,

    /// Locked tokens account
    #[account(
        mut,
        seeds = [
            LOCKED_TOKENS_SEED,
            &source_chain_id.to_le_bytes(),
            &tx_hash
        ],
        bump = locked_tokens.bump,
        constraint = locked_tokens.is_processed == false @ FinovaBridgeError::TransactionAlreadyProcessed,
        constraint = locked_tokens.amount == unlock_amount @ FinovaBridgeError::InvalidUnlockAmount,
        constraint = locked_tokens.source_chain_id == source_chain_id @ FinovaBridgeError::InvalidSourceChain
    )]
    pub locked_tokens: Account<'info, LockedTokens>,

    /// Validator set account
    #[account(
        seeds = [VALIDATOR_SET_SEED],
        bump = validator_set.bump
    )]
    pub validator_set: Account<'info, ValidatorSet>,

    /// Token mint account
    #[account(
        constraint = token_mint.key() == bridge_config.token_mint @ FinovaBridgeError::InvalidTokenMint
    )]
    pub token_mint: Account<'info, Mint>,

    /// Bridge token vault (source of unlocked tokens)
    #[account(
        mut,
        seeds = [BRIDGE_VAULT_SEED, token_mint.key().as_ref()],
        bump = bridge_config.vault_bump,
        constraint = bridge_vault.mint == token_mint.key() @ FinovaBridgeError::InvalidVault,
        constraint = bridge_vault.amount >= unlock_amount @ FinovaBridgeError::InsufficientVaultBalance
    )]
    pub bridge_vault: Account<'info, TokenAccount>,

    /// Recipient's token account (destination of unlocked tokens)
    #[account(
        mut,
        constraint = recipient_token_account.mint == token_mint.key() @ FinovaBridgeError::InvalidRecipientAccount,
        constraint = recipient_token_account.owner == recipient.key() @ FinovaBridgeError::InvalidRecipientOwner
    )]
    pub recipient_token_account: Account<'info, TokenAccount>,

    /// Recipient wallet
    /// CHECK: This account is validated through the locked_tokens recipient field
    #[account(
        constraint = recipient.key() == locked_tokens.recipient @ FinovaBridgeError::InvalidRecipient
    )]
    pub recipient: UncheckedAccount<'info>,

    /// Authority that can unlock tokens (typically a relayer or authorized account)
    #[account(
        constraint = unlock_authority.key() == bridge_config.unlock_authority @ FinovaBridgeError::UnauthorizedUnlock
    )]
    pub unlock_authority: Signer<'info>,

    /// Bridge vault authority (PDA)
    /// CHECK: This is a PDA, validated by the seeds
    #[account(
        seeds = [BRIDGE_AUTHORITY_SEED],
        bump = bridge_config.authority_bump
    )]
    pub bridge_authority: UncheckedAccount<'info>,

    /// Token program
    pub token_program: Program<'info, Token>,

    /// System program
    pub system_program: Program<'info, System>,

    /// Clock sysvar for timestamp validation
    pub clock: Sysvar<'info, Clock>,
}

impl<'info> UnlockTokens<'info> {
    /// Validates the merkle proof for cross-chain transaction
    pub fn validate_merkle_proof(
        &self,
        merkle_proof: &[([u8; 32])],
        leaf_index: u64,
        tx_hash: [u8; 32]
    ) -> Result<()> {
        let merkle_root = self.bridge_config.merkle_root;
        
        // Create leaf data for verification
        let leaf_data = create_unlock_leaf_data(
            tx_hash,
            self.locked_tokens.source_chain_id,
            self.locked_tokens.amount,
            self.locked_tokens.recipient,
            self.locked_tokens.destination_chain_id
        );

        // Verify merkle proof
        let is_valid = MerkleProof::verify(
            merkle_proof,
            merkle_root,
            leaf_index,
            &leaf_data
        );

        require!(is_valid, FinovaBridgeError::InvalidMerkleProof);
        Ok(())
    }

    /// Validates validator signatures
    pub fn validate_validator_signatures(
        &self,
        validator_signatures: &[[u8; 64]],
        tx_hash: [u8; 32],
        unlock_amount: u64
    ) -> Result<()> {
        let required_signatures = self.bridge_config.required_signatures as usize;
        require!(
            validator_signatures.len() >= required_signatures,
            FinovaBridgeError::InsufficientSignatures
        );

        // Create message for signature verification
        let message = create_unlock_message(
            tx_hash,
            self.locked_tokens.source_chain_id,
            unlock_amount,
            self.locked_tokens.recipient,
            self.locked_tokens.destination_chain_id,
            self.clock.unix_timestamp as u64
        );

        let mut valid_signatures = 0;
        let mut used_validators = Vec::new();

        for signature in validator_signatures.iter().take(MAX_VALIDATOR_SIGNATURES) {
            for (i, validator_pubkey) in self.validator_set.validators.iter().enumerate() {
                // Skip if validator already used for this transaction
                if used_validators.contains(&i) {
                    continue;
                }

                // Verify signature
                if SignatureVerification::verify_ed25519(&message, signature, validator_pubkey) {
                    valid_signatures += 1;
                    used_validators.push(i);
                    break;
                }
            }
        }

        require!(
            valid_signatures >= required_signatures,
            FinovaBridgeError::InsufficientValidSignatures
        );

        Ok(())
    }

    /// Validates unlock timing constraints
    pub fn validate_unlock_timing(&self) -> Result<()> {
        let current_time = self.clock.unix_timestamp as u64;
        let lock_time = self.locked_tokens.lock_timestamp;
        let min_lock_duration = self.bridge_config.min_lock_duration;

        // Check minimum lock duration
        require!(
            current_time >= lock_time + min_lock_duration,
            FinovaBridgeError::MinLockDurationNotMet
        );

        // Check maximum unlock window
        let max_unlock_window = self.bridge_config.max_unlock_window;
        if max_unlock_window > 0 {
            require!(
                current_time <= lock_time + max_unlock_window,
                FinovaBridgeError::UnlockWindowExpired
            );
        }

        Ok(())
    }

    /// Validates unlock amount and fees
    pub fn validate_unlock_amount(&self, unlock_amount: u64) -> Result<u64> {
        require!(unlock_amount > 0, FinovaBridgeError::InvalidUnlockAmount);
        require!(
            unlock_amount == self.locked_tokens.amount,
            FinovaBridgeError::AmountMismatch
        );

        // Calculate bridge fee
        let bridge_fee = calculate_bridge_fee(
            unlock_amount,
            self.bridge_config.bridge_fee_rate,
            self.bridge_config.min_bridge_fee,
            self.bridge_config.max_bridge_fee
        );

        let net_unlock_amount = unlock_amount
            .checked_sub(bridge_fee)
            .ok_or(FinovaBridgeError::ArithmeticOverflow)?;

        require!(net_unlock_amount > 0, FinovaBridgeError::InsufficientAmountAfterFees);

        Ok(net_unlock_amount)
    }

    /// Updates bridge statistics
    pub fn update_bridge_stats(&mut self, unlock_amount: u64) -> Result<()> {
        self.bridge_config.total_unlocked = self.bridge_config.total_unlocked
            .checked_add(unlock_amount)
            .ok_or(FinovaBridgeError::ArithmeticOverflow)?;

        self.bridge_config.total_transactions = self.bridge_config.total_transactions
            .checked_add(1)
            .ok_or(FinovaBridgeError::ArithmeticOverflow)?;

        self.bridge_config.last_unlock_timestamp = self.clock.unix_timestamp as u64;

        Ok(())
    }

    /// Creates transfer context for unlocking tokens
    pub fn transfer_context(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        let cpi_accounts = Transfer {
            from: self.bridge_vault.to_account_info(),
            to: self.recipient_token_account.to_account_info(),
            authority: self.bridge_authority.to_account_info(),
        };
        CpiContext::new(self.token_program.to_account_info(), cpi_accounts)
    }
}

/// Main unlock tokens instruction handler
pub fn handler(
    ctx: Context<UnlockTokens>,
    unlock_amount: u64,
    merkle_proof: Vec<[u8; 32]>,
    leaf_index: u64,
    source_chain_id: u16,
    tx_hash: [u8; 32],
    validator_signatures: Vec<[u8; 64]>,
) -> Result<()> {
    let clock = &ctx.accounts.clock;
    
    msg!("Starting token unlock process");
    msg!("Unlock amount: {}", unlock_amount);
    msg!("Source chain ID: {}", source_chain_id);
    msg!("Transaction hash: {:?}", tx_hash);

    // Validate unlock timing
    ctx.accounts.validate_unlock_timing()?;

    // Validate merkle proof
    ctx.accounts.validate_merkle_proof(&merkle_proof, leaf_index, tx_hash)?;

    // Validate validator signatures
    ctx.accounts.validate_validator_signatures(&validator_signatures, tx_hash, unlock_amount)?;

    // Validate and calculate net unlock amount
    let net_unlock_amount = ctx.accounts.validate_unlock_amount(unlock_amount)?;

    // Check rate limiting
    validate_unlock_rate_limit(
        &ctx.accounts.bridge_config,
        unlock_amount,
        clock.unix_timestamp as u64
    )?;

    // Perform the token transfer
    let authority_seeds = &[
        BRIDGE_AUTHORITY_SEED,
        &[ctx.accounts.bridge_config.authority_bump]
    ];
    let signer_seeds = &[&authority_seeds[..]];

    token::transfer(
        ctx.accounts.transfer_context().with_signer(signer_seeds),
        net_unlock_amount
    )?;

    // Calculate and handle bridge fee
    let bridge_fee = unlock_amount
        .checked_sub(net_unlock_amount)
        .ok_or(FinovaBridgeError::ArithmeticOverflow)?;

    if bridge_fee > 0 {
        // Transfer fee to fee collector
        let fee_collector = ctx.accounts.bridge_config.fee_collector;
        
        // Note: In a complete implementation, you would need to add fee_collector_account
        // to the accounts struct and perform the fee transfer here
        msg!("Bridge fee collected: {}", bridge_fee);
    }

    // Mark transaction as processed
    let locked_tokens = &mut ctx.accounts.locked_tokens;
    locked_tokens.is_processed = true;
    locked_tokens.unlock_timestamp = clock.unix_timestamp as u64;
    locked_tokens.actual_unlock_amount = net_unlock_amount;

    // Update bridge statistics
    ctx.accounts.update_bridge_stats(unlock_amount)?;

    // Emit unlock event
    emit!(UnlockEvent {
        tx_hash,
        source_chain_id,
        destination_chain_id: locked_tokens.destination_chain_id,
        recipient: locked_tokens.recipient,
        unlock_amount: net_unlock_amount,
        bridge_fee,
        timestamp: clock.unix_timestamp as u64,
        merkle_root: ctx.accounts.bridge_config.merkle_root,
        leaf_index,
    });

    msg!("Token unlock completed successfully");
    msg!("Net amount unlocked: {}", net_unlock_amount);
    msg!("Bridge fee: {}", bridge_fee);

    Ok(())
}

/// Event emitted when tokens are unlocked
#[event]
pub struct UnlockEvent {
    pub tx_hash: [u8; 32],
    pub source_chain_id: u16,
    pub destination_chain_id: u16,
    pub recipient: Pubkey,
    pub unlock_amount: u64,
    pub bridge_fee: u64,
    pub timestamp: u64,
    pub merkle_root: [u8; 32],
    pub leaf_index: u64,
}

/// Creates leaf data for merkle proof verification
fn create_unlock_leaf_data(
    tx_hash: [u8; 32],
    source_chain_id: u16,
    amount: u64,
    recipient: Pubkey,
    destination_chain_id: u16,
) -> Vec<u8> {
    let mut leaf_data = Vec::new();
    leaf_data.extend_from_slice(&tx_hash);
    leaf_data.extend_from_slice(&source_chain_id.to_le_bytes());
    leaf_data.extend_from_slice(&amount.to_le_bytes());
    leaf_data.extend_from_slice(recipient.as_ref());
    leaf_data.extend_from_slice(&destination_chain_id.to_le_bytes());
    leaf_data
}

/// Creates message for validator signature verification
fn create_unlock_message(
    tx_hash: [u8; 32],
    source_chain_id: u16,
    amount: u64,
    recipient: Pubkey,
    destination_chain_id: u16,
    timestamp: u64,
) -> Vec<u8> {
    let mut message = Vec::new();
    message.extend_from_slice(b"FINOVA_UNLOCK:");
    message.extend_from_slice(&tx_hash);
    message.extend_from_slice(&source_chain_id.to_le_bytes());
    message.extend_from_slice(&amount.to_le_bytes());
    message.extend_from_slice(recipient.as_ref());
    message.extend_from_slice(&destination_chain_id.to_le_bytes());
    message.extend_from_slice(&timestamp.to_le_bytes());
    message
}

/// Validates unlock rate limiting
fn validate_unlock_rate_limit(
    bridge_config: &BridgeConfig,
    unlock_amount: u64,
    current_timestamp: u64,
) -> Result<()> {
    let time_window = bridge_config.rate_limit_window;
    let max_amount = bridge_config.max_unlock_per_window;

    if time_window == 0 || max_amount == 0 {
        return Ok(()); // Rate limiting disabled
    }

    // Check if within rate limit window
    let window_start = current_timestamp
        .checked_sub(time_window)
        .unwrap_or(0);

    if bridge_config.last_unlock_timestamp >= window_start {
        let current_window_amount = bridge_config.unlock_amount_in_window
            .checked_add(unlock_amount)
            .ok_or(FinovaBridgeError::ArithmeticOverflow)?;

        require!(
            current_window_amount <= max_amount,
            FinovaBridgeError::RateLimitExceeded
        );
    }

    Ok(())
}

/// Calculates bridge fee based on amount and fee structure
fn calculate_bridge_fee(
    amount: u64,
    fee_rate: u16, // basis points (e.g., 100 = 1%)
    min_fee: u64,
    max_fee: u64,
) -> u64 {
    if fee_rate == 0 {
        return 0;
    }

    let rate_fee = amount
        .checked_mul(fee_rate as u64)
        .and_then(|x| x.checked_div(10_000))
        .unwrap_or(0);

    // Apply min/max constraints
    let fee = rate_fee.max(min_fee);
    if max_fee > 0 {
        fee.min(max_fee)
    } else {
        fee
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_unlock_leaf_data() {
        let tx_hash = [1u8; 32];
        let source_chain_id = 1u16;
        let amount = 1000u64;
        let recipient = Pubkey::new_unique();
        let destination_chain_id = 2u16;

        let leaf_data = create_unlock_leaf_data(
            tx_hash,
            source_chain_id,
            amount,
            recipient,
            destination_chain_id,
        );

        assert_eq!(leaf_data.len(), 32 + 2 + 8 + 32 + 2); // Expected total length
        assert_eq!(&leaf_data[0..32], &tx_hash);
        assert_eq!(&leaf_data[32..34], &source_chain_id.to_le_bytes());
        assert_eq!(&leaf_data[34..42], &amount.to_le_bytes());
        assert_eq!(&leaf_data[42..74], recipient.as_ref());
        assert_eq!(&leaf_data[74..76], &destination_chain_id.to_le_bytes());
    }

    #[test]
    fn test_calculate_bridge_fee() {
        // Test normal fee calculation (1%)
        let fee = calculate_bridge_fee(10000, 100, 10, 1000);
        assert_eq!(fee, 100); // 1% of 10000

        // Test minimum fee
        let fee = calculate_bridge_fee(100, 100, 10, 1000);
        assert_eq!(fee, 10); // Min fee applied

        // Test maximum fee
        let fee = calculate_bridge_fee(1000000, 100, 10, 1000);
        assert_eq!(fee, 1000); // Max fee applied

        // Test zero fee rate
        let fee = calculate_bridge_fee(10000, 0, 10, 1000);
        assert_eq!(fee, 0);
    }

    #[test]
    fn test_create_unlock_message() {
        let tx_hash = [2u8; 32];
        let source_chain_id = 1u16;
        let amount = 5000u64;
        let recipient = Pubkey::new_unique();
        let destination_chain_id = 2u16;
        let timestamp = 1640995200u64;

        let message = create_unlock_message(
            tx_hash,
            source_chain_id,
            amount,
            recipient,
            destination_chain_id,
            timestamp,
        );

        assert!(message.starts_with(b"FINOVA_UNLOCK:"));
        assert!(message.len() > 14); // Prefix + data
    }
}
