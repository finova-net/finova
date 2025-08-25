// programs/finova-bridge/src/instructions/validate_proof.rs

use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};
use crate::state::*;
use crate::errors::*;
use crate::cryptography::*;
use crate::constants::*;
use crate::utils::*;

#[derive(Accounts)]
#[instruction(proof_data: ProofData)]
pub struct ValidateProof<'info> {
    #[account(mut)]
    pub validator: Signer<'info>,
    
    #[account(
        mut,
        seeds = [BRIDGE_SEED],
        bump,
        constraint = bridge_config.is_active @ BridgeError::BridgeInactive,
        constraint = !bridge_config.emergency_pause @ BridgeError::BridgeEmergencyPause
    )]
    pub bridge_config: Account<'info, BridgeConfig>,
    
    #[account(
        mut,
        seeds = [VALIDATOR_SET_SEED],
        bump,
        constraint = validator_set.is_validator(&validator.key()) @ BridgeError::UnauthorizedValidator
    )]
    pub validator_set: Account<'info, ValidatorSet>,
    
    #[account(
        mut,
        seeds = [LOCKED_TOKENS_SEED, &proof_data.transaction_hash],
        bump,
        constraint = locked_tokens.status == TokenLockStatus::Pending @ BridgeError::InvalidTokenLockStatus
    )]
    pub locked_tokens: Account<'info, LockedTokens>,
    
    #[account(
        mut,
        constraint = vault_token_account.mint == locked_tokens.token_mint,
        constraint = vault_token_account.owner == bridge_config.key()
    )]
    pub vault_token_account: Account<'info, TokenAccount>,
    
    #[account(
        mut,
        constraint = recipient_token_account.mint == locked_tokens.token_mint,
        constraint = recipient_token_account.owner == locked_tokens.recipient
    )]
    pub recipient_token_account: Account<'info, TokenAccount>,
    
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub clock: Sysvar<'info, Clock>,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct ProofData {
    pub transaction_hash: [u8; 32],
    pub block_hash: [u8; 32],
    pub block_number: u64,
    pub merkle_proof: Vec<[u8; 32]>,
    pub merkle_root: [u8; 32],
    pub log_index: u32,
    pub transaction_index: u32,
    pub receipt_proof: Vec<u8>,
    pub header_rlp: Vec<u8>,
    pub validator_signatures: Vec<ValidatorSignature>,
    pub chain_id: u64,
    pub timestamp: i64,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct ValidatorSignature {
    pub validator_pubkey: Pubkey,
    pub signature: [u8; 64],
    pub recovery_id: u8,
    pub message_hash: [u8; 32],
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum ValidationResult {
    Valid,
    InvalidMerkleProof,
    InvalidValidatorSignatures,
    InsufficientValidatorSignatures,
    InvalidBlockHeader,
    InvalidTransactionProof,
    ExpiredProof,
    InvalidChainId,
    ReplayAttack,
}

pub fn validate_proof(ctx: Context<ValidateProof>, proof_data: ProofData) -> Result<()> {
    let bridge_config = &mut ctx.accounts.bridge_config;
    let validator_set = &mut ctx.accounts.validator_set;
    let locked_tokens = &mut ctx.accounts.locked_tokens;
    let clock = &ctx.accounts.clock;
    
    // Validate proof timestamp
    require!(
        clock.unix_timestamp - proof_data.timestamp <= PROOF_VALIDITY_PERIOD,
        BridgeError::ProofExpired
    );
    
    // Validate chain ID
    require!(
        proof_data.chain_id == bridge_config.source_chain_id,
        BridgeError::InvalidSourceChain
    );
    
    // Check for replay attack
    require!(
        !bridge_config.processed_proofs.contains(&proof_data.transaction_hash),
        BridgeError::ProofAlreadyProcessed
    );
    
    // Validate merkle proof
    let validation_result = validate_merkle_proof(&proof_data)?;
    require!(
        validation_result == ValidationResult::Valid,
        BridgeError::InvalidMerkleProof
    );
    
    // Validate validator signatures
    let signature_validation = validate_validator_signatures(
        &proof_data,
        validator_set,
        bridge_config.required_validator_threshold
    )?;
    require!(
        signature_validation == ValidationResult::Valid,
        BridgeError::InsufficientValidatorSignatures
    );
    
    // Validate block header
    let header_validation = validate_block_header(&proof_data)?;
    require!(
        header_validation == ValidationResult::Valid,
        BridgeError::InvalidBlockHeader
    );
    
    // Validate transaction receipt proof
    let receipt_validation = validate_transaction_receipt(&proof_data)?;
    require!(
        receipt_validation == ValidationResult::Valid,
        BridgeError::InvalidTransactionProof
    );
    
    // Additional security checks
    perform_additional_security_checks(&proof_data, bridge_config)?;
    
    // Execute token unlock if all validations pass
    execute_token_unlock(ctx, &proof_data)?;
    
    // Mark proof as processed to prevent replay
    bridge_config.processed_proofs.push(proof_data.transaction_hash);
    bridge_config.total_processed_proofs += 1;
    
    // Update validator statistics
    update_validator_statistics(validator_set, &ctx.accounts.validator.key())?;
    
    // Emit validation event
    emit!(ProofValidatedEvent {
        transaction_hash: proof_data.transaction_hash,
        validator: ctx.accounts.validator.key(),
        block_number: proof_data.block_number,
        chain_id: proof_data.chain_id,
        amount: locked_tokens.amount,
        recipient: locked_tokens.recipient,
        timestamp: clock.unix_timestamp,
    });
    
    Ok(())
}

fn validate_merkle_proof(proof_data: &ProofData) -> Result<ValidationResult> {
    // Reconstruct transaction leaf from proof data
    let transaction_leaf = construct_transaction_leaf(proof_data)?;
    
    // Verify merkle proof against the provided root
    let is_valid = merkle_proof::verify_merkle_proof(
        &transaction_leaf,
        &proof_data.merkle_proof,
        &proof_data.merkle_root,
        proof_data.log_index as usize
    )?;
    
    if is_valid {
        Ok(ValidationResult::Valid)
    } else {
        Ok(ValidationResult::InvalidMerkleProof)
    }
}

fn validate_validator_signatures(
    proof_data: &ProofData,
    validator_set: &ValidatorSet,
    required_threshold: u8
) -> Result<ValidationResult> {
    let mut valid_signatures = 0u8;
    let message_hash = construct_validation_message_hash(proof_data)?;
    
    // Validate each signature
    for signature in &proof_data.validator_signatures {
        // Check if signer is a valid validator
        if !validator_set.is_validator(&signature.validator_pubkey) {
            continue;
        }
        
        // Verify signature
        let is_valid = signature_verification::verify_validator_signature(
            &signature.validator_pubkey,
            &signature.signature,
            &message_hash,
            signature.recovery_id
        )?;
        
        if is_valid {
            valid_signatures += 1;
        }
    }
    
    // Check if we have sufficient valid signatures
    if valid_signatures >= required_threshold {
        Ok(ValidationResult::Valid)
    } else {
        Ok(ValidationResult::InsufficientValidatorSignatures)
    }
}

fn validate_block_header(proof_data: &ProofData) -> Result<ValidationResult> {
    // Parse RLP-encoded block header
    let header = parse_block_header_rlp(&proof_data.header_rlp)?;
    
    // Validate block hash matches
    require!(
        header.block_hash == proof_data.block_hash,
        BridgeError::BlockHashMismatch
    );
    
    // Validate block number matches
    require!(
        header.block_number == proof_data.block_number,
        BridgeError::BlockNumberMismatch
    );
    
    // Validate merkle root matches
    require!(
        header.transactions_root == proof_data.merkle_root,
        BridgeError::MerkleRootMismatch
    );
    
    // Additional header validations
    validate_block_difficulty(&header)?;
    validate_block_timestamp(&header, proof_data.timestamp)?;
    validate_block_gas_limit(&header)?;
    
    Ok(ValidationResult::Valid)
}

fn validate_transaction_receipt(proof_data: &ProofData) -> Result<ValidationResult> {
    // Parse transaction receipt proof
    let receipt = parse_transaction_receipt(&proof_data.receipt_proof)?;
    
    // Validate transaction hash matches
    require!(
        receipt.transaction_hash == proof_data.transaction_hash,
        BridgeError::TransactionHashMismatch
    );
    
    // Validate transaction was successful
    require!(
        receipt.status == 1,
        BridgeError::FailedTransaction
    );
    
    // Validate log index exists
    require!(
        receipt.logs.len() > proof_data.log_index as usize,
        BridgeError::InvalidLogIndex
    );
    
    // Validate lock event in logs
    let lock_event = &receipt.logs[proof_data.log_index as usize];
    validate_lock_event(lock_event)?;
    
    Ok(ValidationResult::Valid)
}

fn perform_additional_security_checks(
    proof_data: &ProofData,
    bridge_config: &BridgeConfig
) -> Result<()> {
    // Check block confirmation depth
    let current_block = get_current_block_number(proof_data.chain_id)?;
    let confirmations = current_block.saturating_sub(proof_data.block_number);
    
    require!(
        confirmations >= bridge_config.min_confirmations,
        BridgeError::InsufficientConfirmations
    );
    
    // Validate proof freshness
    let proof_age = Clock::get()?.unix_timestamp - proof_data.timestamp;
    require!(
        proof_age <= MAX_PROOF_AGE,
        BridgeError::ProofTooOld
    );
    
    // Check for known malicious blocks
    require!(
        !bridge_config.blacklisted_blocks.contains(&proof_data.block_hash),
        BridgeError::BlacklistedBlock
    );
    
    // Validate transaction index bounds
    require!(
        proof_data.transaction_index < MAX_TRANSACTIONS_PER_BLOCK,
        BridgeError::InvalidTransactionIndex
    );
    
    Ok(())
}

fn execute_token_unlock(
    ctx: Context<ValidateProof>,
    proof_data: &ProofData
) -> Result<()> {
    let locked_tokens = &mut ctx.accounts.locked_tokens;
    let bridge_config = &ctx.accounts.bridge_config;
    
    // Validate unlock amount matches locked amount
    let unlock_amount = extract_unlock_amount_from_proof(proof_data)?;
    require!(
        unlock_amount == locked_tokens.amount,
        BridgeError::AmountMismatch
    );
    
    // Calculate bridge fees
    let bridge_fee = calculate_bridge_fee(unlock_amount, bridge_config.fee_rate)?;
    let net_amount = unlock_amount.saturating_sub(bridge_fee);
    
    // Transfer tokens from vault to recipient
    let bridge_seeds = &[
        BRIDGE_SEED,
        &[ctx.bumps.bridge_config]
    ];
    let signer_seeds = &[&bridge_seeds[..]];
    
    let transfer_instruction = Transfer {
        from: ctx.accounts.vault_token_account.to_account_info(),
        to: ctx.accounts.recipient_token_account.to_account_info(),
        authority: bridge_config.to_account_info(),
    };
    
    let transfer_ctx = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        transfer_instruction,
        signer_seeds
    );
    
    token::transfer(transfer_ctx, net_amount)?;
    
    // Update locked tokens status
    locked_tokens.status = TokenLockStatus::Unlocked;
    locked_tokens.unlock_timestamp = Clock::get()?.unix_timestamp;
    locked_tokens.validator = ctx.accounts.validator.key();
    locked_tokens.bridge_fee = bridge_fee;
    
    // Update bridge statistics
    let bridge_config = &mut ctx.accounts.bridge_config;
    bridge_config.total_unlocked_amount += net_amount;
    bridge_config.total_bridge_fees += bridge_fee;
    bridge_config.successful_unlocks += 1;
    
    Ok(())
}

// Helper functions
fn construct_transaction_leaf(proof_data: &ProofData) -> Result<[u8; 32]> {
    let mut hasher = solana_program::keccak::Hasher::default();
    hasher.hash(&proof_data.transaction_hash);
    hasher.hash(&proof_data.block_hash);
    hasher.hash(&proof_data.block_number.to_le_bytes());
    hasher.hash(&proof_data.log_index.to_le_bytes());
    Ok(hasher.result().to_bytes())
}

fn construct_validation_message_hash(proof_data: &ProofData) -> Result<[u8; 32]> {
    let mut hasher = solana_program::keccak::Hasher::default();
    hasher.hash(&proof_data.transaction_hash);
    hasher.hash(&proof_data.block_hash);
    hasher.hash(&proof_data.merkle_root);
    hasher.hash(&proof_data.chain_id.to_le_bytes());
    hasher.hash(&proof_data.timestamp.to_le_bytes());
    Ok(hasher.result().to_bytes())
}

fn parse_block_header_rlp(header_rlp: &[u8]) -> Result<BlockHeader> {
    // Simplified RLP parsing - in production, use proper RLP library
    require!(header_rlp.len() >= MIN_BLOCK_HEADER_SIZE, BridgeError::InvalidBlockHeader);
    
    // Extract key fields from RLP-encoded header
    let block_hash = extract_bytes_from_rlp(header_rlp, 0, 32)?;
    let block_number = extract_u64_from_rlp(header_rlp, 32)?;
    let transactions_root = extract_bytes_from_rlp(header_rlp, 40, 32)?;
    let timestamp = extract_u64_from_rlp(header_rlp, 72)?;
    let difficulty = extract_u64_from_rlp(header_rlp, 80)?;
    let gas_limit = extract_u64_from_rlp(header_rlp, 88)?;
    
    Ok(BlockHeader {
        block_hash,
        block_number,
        transactions_root,
        timestamp,
        difficulty,
        gas_limit,
    })
}

fn parse_transaction_receipt(receipt_proof: &[u8]) -> Result<TransactionReceipt> {
    require!(receipt_proof.len() >= MIN_RECEIPT_SIZE, BridgeError::InvalidTransactionReceipt);
    
    let transaction_hash = extract_bytes_from_rlp(receipt_proof, 0, 32)?;
    let status = extract_u8_from_rlp(receipt_proof, 32)?;
    let logs_count = extract_u32_from_rlp(receipt_proof, 33)?;
    
    let mut logs = Vec::new();
    let mut offset = 37;
    
    for _ in 0..logs_count {
        let log = parse_log_from_rlp(receipt_proof, &mut offset)?;
        logs.push(log);
    }
    
    Ok(TransactionReceipt {
        transaction_hash,
        status,
        logs,
    })
}

fn validate_lock_event(log: &TransactionLog) -> Result<()> {
    // Validate log address matches bridge contract
    require!(
        log.address == BRIDGE_CONTRACT_ADDRESS,
        BridgeError::InvalidLogAddress
    );
    
    // Validate log topics match lock event signature
    require!(
        !log.topics.is_empty() && log.topics[0] == LOCK_EVENT_SIGNATURE,
        BridgeError::InvalidEventSignature
    );
    
    // Additional event data validation
    require!(log.data.len() >= MIN_LOCK_EVENT_DATA_SIZE, BridgeError::InvalidEventData);
    
    Ok(())
}

fn extract_unlock_amount_from_proof(proof_data: &ProofData) -> Result<u64> {
    // Extract amount from transaction receipt logs
    // This is a simplified implementation
    let receipt = parse_transaction_receipt(&proof_data.receipt_proof)?;
    let lock_log = &receipt.logs[proof_data.log_index as usize];
    
    // Extract amount from log data (bytes 32-40 typically contain amount)
    require!(lock_log.data.len() >= 40, BridgeError::InvalidEventData);
    let amount_bytes = &lock_log.data[32..40];
    let amount = u64::from_le_bytes(amount_bytes.try_into().unwrap());
    
    Ok(amount)
}

fn calculate_bridge_fee(amount: u64, fee_rate: u16) -> Result<u64> {
    let fee = (amount as u128)
        .checked_mul(fee_rate as u128)
        .ok_or(BridgeError::CalculationOverflow)?
        .checked_div(FEE_DENOMINATOR as u128)
        .ok_or(BridgeError::CalculationOverflow)?;
    
    Ok(fee as u64)
}

fn get_current_block_number(chain_id: u64) -> Result<u64> {
    // In production, this would query an oracle or external service
    // For now, return a mock value
    match chain_id {
        1 => Ok(18_500_000), // Ethereum mainnet
        56 => Ok(32_000_000), // BSC
        137 => Ok(48_000_000), // Polygon
        _ => Err(BridgeError::UnsupportedChain.into()),
    }
}

fn update_validator_statistics(validator_set: &mut ValidatorSet, validator: &Pubkey) -> Result<()> {
    if let Some(validator_info) = validator_set.validators.iter_mut()
        .find(|v| v.pubkey == *validator) {
        validator_info.successful_validations += 1;
        validator_info.last_validation_timestamp = Clock::get()?.unix_timestamp;
        
        // Update validator reputation score
        validator_info.reputation_score = calculate_validator_reputation(validator_info)?;
    }
    
    Ok(())
}

fn calculate_validator_reputation(validator_info: &ValidatorInfo) -> Result<u32> {
    let success_rate = if validator_info.total_validations > 0 {
        (validator_info.successful_validations * 100) / validator_info.total_validations
    } else {
        100
    };
    
    let base_score = success_rate * 10; // Base score from success rate
    let uptime_bonus = validator_info.uptime_percentage / 10; // Bonus from uptime
    let slashing_penalty = validator_info.slashing_count * 50; // Penalty from slashing
    
    let reputation = base_score
        .saturating_add(uptime_bonus)
        .saturating_sub(slashing_penalty);
    
    Ok(reputation.min(1000)) // Cap at 1000
}

// Helper functions for RLP parsing (simplified)
fn extract_bytes_from_rlp(data: &[u8], offset: usize, length: usize) -> Result<[u8; 32]> {
    require!(data.len() >= offset + length, BridgeError::InvalidRLPData);
    let mut result = [0u8; 32];
    result[..length].copy_from_slice(&data[offset..offset + length]);
    Ok(result)
}

fn extract_u64_from_rlp(data: &[u8], offset: usize) -> Result<u64> {
    require!(data.len() >= offset + 8, BridgeError::InvalidRLPData);
    let bytes: [u8; 8] = data[offset..offset + 8].try_into()
        .map_err(|_| BridgeError::InvalidRLPData)?;
    Ok(u64::from_le_bytes(bytes))
}

fn extract_u32_from_rlp(data: &[u8], offset: usize) -> Result<u32> {
    require!(data.len() >= offset + 4, BridgeError::InvalidRLPData);
    let bytes: [u8; 4] = data[offset..offset + 4].try_into()
        .map_err(|_| BridgeError::InvalidRLPData)?;
    Ok(u32::from_le_bytes(bytes))
}

fn extract_u8_from_rlp(data: &[u8], offset: usize) -> Result<u8> {
    require!(data.len() > offset, BridgeError::InvalidRLPData);
    Ok(data[offset])
}

fn parse_log_from_rlp(data: &[u8], offset: &mut usize) -> Result<TransactionLog> {
    let address = extract_bytes_from_rlp(data, *offset, 20)?;
    *offset += 20;
    
    let topics_count = extract_u8_from_rlp(data, *offset)? as usize;
    *offset += 1;
    
    let mut topics = Vec::new();
    for _ in 0..topics_count {
        let topic = extract_bytes_from_rlp(data, *offset, 32)?;
        topics.push(topic);
        *offset += 32;
    }
    
    let data_length = extract_u32_from_rlp(data, *offset)? as usize;
    *offset += 4;
    
    let mut log_data = vec![0u8; data_length];
    log_data.copy_from_slice(&data[*offset..*offset + data_length]);
    *offset += data_length;
    
    Ok(TransactionLog {
        address,
        topics,
        data: log_data,
    })
}

// Additional validation helper functions
fn validate_block_difficulty(header: &BlockHeader) -> Result<()> {
    require!(
        header.difficulty >= MIN_BLOCK_DIFFICULTY,
        BridgeError::InvalidBlockDifficulty
    );
    Ok(())
}

fn validate_block_timestamp(header: &BlockHeader, proof_timestamp: i64) -> Result<()> {
    let timestamp_diff = (header.timestamp as i64 - proof_timestamp).abs();
    require!(
        timestamp_diff <= MAX_TIMESTAMP_DRIFT,
        BridgeError::InvalidBlockTimestamp
    );
    Ok(())
}

fn validate_block_gas_limit(header: &BlockHeader) -> Result<()> {
    require!(
        header.gas_limit >= MIN_GAS_LIMIT && header.gas_limit <= MAX_GAS_LIMIT,
        BridgeError::InvalidGasLimit
    );
    Ok(())
}

// Supporting data structures
#[derive(Debug)]
struct BlockHeader {
    block_hash: [u8; 32],
    block_number: u64,
    transactions_root: [u8; 32],
    timestamp: u64,
    difficulty: u64,
    gas_limit: u64,
}

#[derive(Debug)]
struct TransactionReceipt {
    transaction_hash: [u8; 32],
    status: u8,
    logs: Vec<TransactionLog>,
}

#[derive(Debug)]
struct TransactionLog {
    address: [u8; 32],
    topics: Vec<[u8; 32]>,
    data: Vec<u8>,
}

#[event]
pub struct ProofValidatedEvent {
    pub transaction_hash: [u8; 32],
    pub validator: Pubkey,
    pub block_number: u64,
    pub chain_id: u64,
    pub amount: u64,
    pub recipient: Pubkey,
    pub timestamp: i64,
}
