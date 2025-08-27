// programs/finova-bridge/src/cryptography/signature_verification.rs

use anchor_lang::prelude::*;
use anchor_lang::solana_program::{
    ed25519_program,
    instruction::Instruction,
    sysvar::instructions::{load_current_index_checked, load_instruction_at_checked},
};
use std::convert::TryInto;
use crate::errors::BridgeError;
use crate::state::{ValidatorSet, BridgeConfig};

/// Maximum number of validators that can be verified in a single transaction
pub const MAX_VALIDATORS_PER_TX: usize = 10;

/// Minimum number of signatures required for a valid multi-sig
pub const MIN_SIGNATURE_THRESHOLD: u8 = 2;

/// Maximum age of a signature in slots (approximately 10 minutes)
pub const MAX_SIGNATURE_AGE: u64 = 1200;

/// Ed25519 signature size in bytes
pub const ED25519_SIGNATURE_SIZE: usize = 64;

/// Ed25519 public key size in bytes  
pub const ED25519_PUBKEY_SIZE: usize = 32;

/// Message hash size for signature verification
pub const MESSAGE_HASH_SIZE: usize = 32;

#[derive(Debug, Clone)]
pub struct SignatureData {
    pub signature: [u8; ED25519_SIGNATURE_SIZE],
    pub public_key: [u8; ED25519_PUBKEY_SIZE],
    pub message_hash: [u8; MESSAGE_HASH_SIZE],
    pub validator_index: u8,
    pub timestamp: i64,
}

#[derive(Debug, Clone)]
pub struct MultiSignatureVerification {
    pub signatures: Vec<SignatureData>,
    pub message: Vec<u8>,
    pub required_threshold: u8,
    pub verified_count: u8,
    pub is_valid: bool,
}

/// Verifies Ed25519 signature using Solana's native instruction
pub fn verify_ed25519_signature(
    signature: &[u8; ED25519_SIGNATURE_SIZE],
    public_key: &[u8; ED25519_PUBKEY_SIZE],
    message: &[u8],
) -> Result<bool> {
    // Validate input lengths
    if signature.len() != ED25519_SIGNATURE_SIZE {
        return Err(BridgeError::InvalidSignatureLength.into());
    }
    
    if public_key.len() != ED25519_PUBKEY_SIZE {
        return Err(BridgeError::InvalidPublicKeyLength.into());
    }

    if message.is_empty() || message.len() > 1024 {
        return Err(BridgeError::InvalidMessageLength.into());
    }

    // Check if Ed25519 verification instruction exists in the current transaction
    let current_index = load_current_index_checked()?;
    
    // Look for Ed25519 signature verification instruction
    for i in 0..current_index {
        let instruction = load_instruction_at_checked(i.into(), &ed25519_program::ID)?;
        
        if verify_ed25519_instruction_data(&instruction, signature, public_key, message)? {
            return Ok(true);
        }
    }

    Err(BridgeError::SignatureVerificationFailed.into())
}

/// Verifies the Ed25519 instruction data matches expected signature components
fn verify_ed25519_instruction_data(
    instruction: &Instruction,
    expected_signature: &[u8; ED25519_SIGNATURE_SIZE],
    expected_public_key: &[u8; ED25519_PUBKEY_SIZE],
    expected_message: &[u8],
) -> Result<bool> {
    // Ed25519 instruction data format:
    // [num_signatures: u8][padding: u8][signature_offset: u16][signature_instruction_index: u16]
    // [public_key_offset: u16][public_key_instruction_index: u16][message_data_offset: u16]
    // [message_data_size: u16][message_instruction_index: u16][signature: 64 bytes]
    // [public_key: 32 bytes][message: variable]

    let data = &instruction.data;
    if data.len() < 2 {
        return Ok(false);
    }

    let num_signatures = data[0];
    if num_signatures != 1 {
        return Ok(false);
    }

    // Skip to signature data (after header)
    let mut offset = 14; // Header size for single signature

    // Verify signature
    if data.len() < offset + ED25519_SIGNATURE_SIZE {
        return Ok(false);
    }
    
    let signature_slice = &data[offset..offset + ED25519_SIGNATURE_SIZE];
    if signature_slice != expected_signature {
        return Ok(false);
    }
    offset += ED25519_SIGNATURE_SIZE;

    // Verify public key
    if data.len() < offset + ED25519_PUBKEY_SIZE {
        return Ok(false);
    }
    
    let pubkey_slice = &data[offset..offset + ED25519_PUBKEY_SIZE];
    if pubkey_slice != expected_public_key {
        return Ok(false);
    }
    offset += ED25519_PUBKEY_SIZE;

    // Verify message
    if data.len() < offset + expected_message.len() {
        return Ok(false);
    }
    
    let message_slice = &data[offset..offset + expected_message.len()];
    if message_slice != expected_message {
        return Ok(false);
    }

    Ok(true)
}

/// Verifies multiple signatures from bridge validators
pub fn verify_multi_signature(
    signatures: &[SignatureData],
    validator_set: &ValidatorSet,
    bridge_config: &BridgeConfig,
    message: &[u8],
    current_slot: u64,
) -> Result<MultiSignatureVerification> {
    if signatures.is_empty() {
        return Err(BridgeError::NoSignaturesProvided.into());
    }

    if signatures.len() > MAX_VALIDATORS_PER_TX {
        return Err(BridgeError::TooManySignatures.into());
    }

    let required_threshold = calculate_signature_threshold(
        validator_set.validator_count,
        bridge_config.signature_threshold_percentage,
    )?;

    let mut verification = MultiSignatureVerification {
        signatures: signatures.to_vec(),
        message: message.to_vec(),
        required_threshold,
        verified_count: 0,
        is_valid: false,
    };

    let mut verified_validators = std::collections::HashSet::new();

    // Verify each signature
    for signature_data in signatures {
        // Check signature age
        if is_signature_expired(signature_data.timestamp, current_slot)? {
            msg!("Signature expired: validator {}", signature_data.validator_index);
            continue;
        }

        // Validate validator index
        if !is_valid_validator_index(signature_data.validator_index, validator_set)? {
            msg!("Invalid validator index: {}", signature_data.validator_index);
            continue;
        }

        // Check for duplicate validator signatures
        if verified_validators.contains(&signature_data.validator_index) {
            msg!("Duplicate signature from validator: {}", signature_data.validator_index);
            continue;
        }

        // Get validator public key
        let validator_pubkey = get_validator_public_key(
            signature_data.validator_index,
            validator_set,
        )?;

        // Verify the signature matches the expected public key
        if signature_data.public_key != validator_pubkey {
            msg!("Public key mismatch for validator: {}", signature_data.validator_index);
            continue;
        }

        // Verify message hash
        let computed_hash = hash_message_for_signature(message)?;
        if signature_data.message_hash != computed_hash {
            msg!("Message hash mismatch for validator: {}", signature_data.validator_index);
            continue;
        }

        // Verify Ed25519 signature
        if verify_ed25519_signature(
            &signature_data.signature,
            &signature_data.public_key,
            message,
        )? {
            verified_validators.insert(signature_data.validator_index);
            verification.verified_count += 1;
            msg!("Verified signature from validator: {}", signature_data.validator_index);
        } else {
            msg!("Failed to verify signature from validator: {}", signature_data.validator_index);
        }
    }

    // Check if we have enough valid signatures
    verification.is_valid = verification.verified_count >= required_threshold;

    if !verification.is_valid {
        msg!(
            "Insufficient signatures: {} verified, {} required",
            verification.verified_count,
            required_threshold
        );
        return Err(BridgeError::InsufficientSignatures.into());
    }

    msg!(
        "Multi-signature verification successful: {}/{} signatures verified",
        verification.verified_count,
        required_threshold
    );

    Ok(verification)
}

/// Calculates the required signature threshold based on validator count and percentage
pub fn calculate_signature_threshold(
    validator_count: u8,
    threshold_percentage: u8,
) -> Result<u8> {
    if validator_count == 0 {
        return Err(BridgeError::NoValidatorsInSet.into());
    }

    if threshold_percentage > 100 {
        return Err(BridgeError::InvalidThresholdPercentage.into());
    }

    let calculated_threshold = ((validator_count as u16 * threshold_percentage as u16) / 100) as u8;
    let minimum_threshold = std::cmp::max(MIN_SIGNATURE_THRESHOLD, calculated_threshold);
    let final_threshold = std::cmp::min(minimum_threshold, validator_count);

    Ok(final_threshold)
}

/// Checks if a signature has expired based on timestamp and current slot
pub fn is_signature_expired(signature_timestamp: i64, current_slot: u64) -> Result<bool> {
    let current_timestamp = Clock::get()?.unix_timestamp;
    let signature_age = current_timestamp - signature_timestamp;
    
    // Convert slot-based age to time-based age (approximately 400ms per slot)
    let max_age_seconds = (MAX_SIGNATURE_AGE * 400) / 1000;
    
    Ok(signature_age > max_age_seconds as i64)
}

/// Validates if a validator index exists in the validator set
pub fn is_valid_validator_index(
    validator_index: u8,
    validator_set: &ValidatorSet,
) -> Result<bool> {
    Ok(validator_index < validator_set.validator_count && 
       validator_set.validators[validator_index as usize].is_active)
}

/// Gets the public key for a specific validator
pub fn get_validator_public_key(
    validator_index: u8,
    validator_set: &ValidatorSet,
) -> Result<[u8; ED25519_PUBKEY_SIZE]> {
    if validator_index >= validator_set.validator_count {
        return Err(BridgeError::ValidatorIndexOutOfBounds.into());
    }

    let validator = &validator_set.validators[validator_index as usize];
    if !validator.is_active {
        return Err(BridgeError::ValidatorNotActive.into());
    }

    Ok(validator.public_key)
}

/// Hashes a message for signature verification using SHA-256
pub fn hash_message_for_signature(message: &[u8]) -> Result<[u8; MESSAGE_HASH_SIZE]> {
    use anchor_lang::solana_program::hash::{hashv, Hash};
    
    if message.is_empty() {
        return Err(BridgeError::EmptyMessage.into());
    }

    // Create a domain separator for bridge signatures
    let domain_separator = b"FINOVA_BRIDGE_SIGNATURE";
    let hash = hashv(&[domain_separator, message]);
    
    Ok(hash.to_bytes())
}

/// Verifies a batch of signatures efficiently
pub fn verify_signature_batch(
    signatures: &[SignatureData],
    validator_set: &ValidatorSet,
    bridge_config: &BridgeConfig,
    messages: &[Vec<u8>],
    current_slot: u64,
) -> Result<Vec<MultiSignatureVerification>> {
    if signatures.len() != messages.len() {
        return Err(BridgeError::SignatureMessageCountMismatch.into());
    }

    let mut verifications = Vec::with_capacity(messages.len());
    
    for (i, message) in messages.iter().enumerate() {
        let signature_slice = &signatures[i..i+1];
        let verification = verify_multi_signature(
            signature_slice,
            validator_set,
            bridge_config,
            message,
            current_slot,
        )?;
        verifications.push(verification);
    }

    Ok(verifications)
}

/// Creates signature data for testing purposes
#[cfg(feature = "test-utils")]
pub fn create_test_signature_data(
    validator_index: u8,
    message: &[u8],
) -> Result<SignatureData> {
    use anchor_lang::solana_program::hash::hashv;
    
    // Generate test signature and public key (not cryptographically secure)
    let mut signature = [0u8; ED25519_SIGNATURE_SIZE];
    let mut public_key = [0u8; ED25519_PUBKEY_SIZE];
    
    // Use message hash as pseudo-random source
    let hash = hashv(&[message, &[validator_index]]);
    signature[..32].copy_from_slice(&hash.to_bytes());
    public_key.copy_from_slice(&hash.to_bytes());
    
    let message_hash = hash_message_for_signature(message)?;
    let timestamp = Clock::get()?.unix_timestamp;

    Ok(SignatureData {
        signature,
        public_key,
        message_hash,
        validator_index,
        timestamp,
    })
}

/// Validates signature format and basic constraints
pub fn validate_signature_format(signature_data: &SignatureData) -> Result<()> {
    // Check signature is not all zeros
    if signature_data.signature.iter().all(|&b| b == 0) {
        return Err(BridgeError::InvalidSignatureFormat.into());
    }

    // Check public key is not all zeros
    if signature_data.public_key.iter().all(|&b| b == 0) {
        return Err(BridgeError::InvalidPublicKeyFormat.into());
    }

    // Check message hash is not all zeros
    if signature_data.message_hash.iter().all(|&b| b == 0) {
        return Err(BridgeError::InvalidMessageHashFormat.into());
    }

    // Check timestamp is reasonable (not too far in the future or past)
    let current_timestamp = Clock::get()?.unix_timestamp;
    let time_diff = (current_timestamp - signature_data.timestamp).abs();
    
    if time_diff > 3600 { // 1 hour
        return Err(BridgeError::SignatureTimestampOutOfRange.into());
    }

    Ok(())
}

/// Aggregates signature verification results for reporting
pub fn aggregate_verification_results(
    verifications: &[MultiSignatureVerification],
) -> Result<VerificationSummary> {
    let total_signatures = verifications.len();
    let successful_verifications = verifications.iter()
        .filter(|v| v.is_valid)
        .count();
    
    let total_signature_count = verifications.iter()
        .map(|v| v.signatures.len())
        .sum::<usize>();
    
    let total_verified_count = verifications.iter()
        .map(|v| v.verified_count as usize)
        .sum::<usize>();

    Ok(VerificationSummary {
        total_messages: total_signatures,
        successful_verifications,
        failed_verifications: total_signatures - successful_verifications,
        total_signatures_checked: total_signature_count,
        total_signatures_verified: total_verified_count,
        success_rate: if total_signatures > 0 {
            (successful_verifications * 100) / total_signatures
        } else {
            0
        },
    })
}

#[derive(Debug, Clone)]
pub struct VerificationSummary {
    pub total_messages: usize,
    pub successful_verifications: usize,
    pub failed_verifications: usize,
    pub total_signatures_checked: usize,
    pub total_signatures_verified: usize,
    pub success_rate: usize, // Percentage
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::{Validator, ValidatorSet, BridgeConfig};

    fn create_test_validator_set() -> ValidatorSet {
        let mut validators = [Validator::default(); 32];
        
        // Create 5 active validators
        for i in 0..5 {
            validators[i] = Validator {
                public_key: [i as u8; 32],
                is_active: true,
                stake_amount: 1000000,
                last_activity_slot: 0,
                reputation_score: 100,
                total_votes: 0,
                successful_votes: 0,
            };
        }

        ValidatorSet {
            validators,
            validator_count: 5,
            active_validator_count: 5,
            total_stake: 5000000,
            last_update_slot: 0,
        }
    }

    fn create_test_bridge_config() -> BridgeConfig {
        BridgeConfig {
            admin: Pubkey::default(),
            is_paused: false,
            signature_threshold_percentage: 67, // 67% threshold
            max_transaction_amount: 1000000,
            min_transaction_amount: 1000,
            daily_withdrawal_limit: 10000000,
            fee_percentage: 100, // 1%
            treasury_account: Pubkey::default(),
            supported_chains: [0; 16],
            supported_chain_count: 1,
            total_locked_amount: 0,
            total_transactions: 0,
            last_update_slot: 0,
        }
    }

    #[test]
    fn test_calculate_signature_threshold() {
        assert_eq!(calculate_signature_threshold(5, 67).unwrap(), 3);
        assert_eq!(calculate_signature_threshold(3, 67).unwrap(), 2);
        assert_eq!(calculate_signature_threshold(10, 51).unwrap(), 5);
        assert_eq!(calculate_signature_threshold(1, 100).unwrap(), 2); // Minimum threshold applied
    }

    #[test]
    fn test_validate_signature_format() {
        let valid_signature = SignatureData {
            signature: [1u8; 64],
            public_key: [2u8; 32],
            message_hash: [3u8; 32],
            validator_index: 0,
            timestamp: 1640995200, // Valid timestamp
        };

        assert!(validate_signature_format(&valid_signature).is_ok());

        // Test invalid signature (all zeros)
        let invalid_signature = SignatureData {
            signature: [0u8; 64],
            public_key: [2u8; 32],
            message_hash: [3u8; 32],
            validator_index: 0,
            timestamp: 1640995200,
        };

        assert!(validate_signature_format(&invalid_signature).is_err());
    }

    #[test]
    fn test_hash_message_for_signature() {
        let message = b"test message";
        let hash1 = hash_message_for_signature(message).unwrap();
        let hash2 = hash_message_for_signature(message).unwrap();
        
        // Same message should produce same hash
        assert_eq!(hash1, hash2);
        
        // Different message should produce different hash
        let different_message = b"different message";
        let hash3 = hash_message_for_signature(different_message).unwrap();
        assert_ne!(hash1, hash3);
    }

    #[test]
    fn test_validator_operations() {
        let validator_set = create_test_validator_set();
        
        // Test valid validator index
        assert!(is_valid_validator_index(0, &validator_set).unwrap());
        assert!(is_valid_validator_index(4, &validator_set).unwrap());
        
        // Test invalid validator index
        assert!(!is_valid_validator_index(5, &validator_set).unwrap());
        assert!(!is_valid_validator_index(10, &validator_set).unwrap());
        
        // Test getting validator public key
        let pubkey = get_validator_public_key(0, &validator_set).unwrap();
        assert_eq!(pubkey, [0u8; 32]);
        
        let pubkey2 = get_validator_public_key(2, &validator_set).unwrap();
        assert_eq!(pubkey2, [2u8; 32]);
    }

    #[test]
    fn test_aggregate_verification_results() {
        let verifications = vec![
            MultiSignatureVerification {
                signatures: vec![],
                message: vec![],
                required_threshold: 3,
                verified_count: 3,
                is_valid: true,
            },
            MultiSignatureVerification {
                signatures: vec![],
                message: vec![],
                required_threshold: 3,
                verified_count: 2,
                is_valid: false,
            },
        ];

        let summary = aggregate_verification_results(&verifications).unwrap();
        assert_eq!(summary.total_messages, 2);
        assert_eq!(summary.successful_verifications, 1);
        assert_eq!(summary.failed_verifications, 1);
        assert_eq!(summary.success_rate, 50);
    }
}
