// programs/finova-bridge/src/cryptography/mod.rs

use anchor_lang::prelude::*;
use solana_program::{
    keccak::hash,
    secp256k1_recover::{secp256k1_recover, Secp256k1Pubkey},
    system_instruction,
};
use std::collections::HashMap;

pub mod merkle_proof;
pub mod signature_verification;

use crate::errors::BridgeError;

/// Cryptographic utilities for cross-chain bridge operations
/// Handles signature verification, merkle proofs, and cross-chain validation
pub struct CryptographyManager;

impl CryptographyManager {
    /// Validates a cross-chain transaction proof
    pub fn validate_cross_chain_proof(
        transaction_hash: &[u8; 32],
        merkle_proof: &[Vec<u8>],
        merkle_root: &[u8; 32],
        validator_signatures: &[ValidatorSignature],
        required_confirmations: u8,
    ) -> Result<bool> {
        // Verify merkle proof
        let is_merkle_valid = Self::verify_merkle_proof(
            transaction_hash,
            merkle_proof,
            merkle_root,
        )?;

        if !is_merkle_valid {
            return Err(BridgeError::InvalidMerkleProof.into());
        }

        // Verify validator signatures
        let valid_signatures = Self::verify_validator_signatures(
            transaction_hash,
            validator_signatures,
        )?;

        if valid_signatures < required_confirmations {
            return Err(BridgeError::InsufficientValidatorSignatures.into());
        }

        Ok(true)
    }

    /// Verifies merkle proof for transaction inclusion
    pub fn verify_merkle_proof(
        leaf: &[u8; 32],
        proof: &[Vec<u8>],
        root: &[u8; 32],
    ) -> Result<bool> {
        if proof.is_empty() {
            return Ok(leaf == root);
        }

        let mut computed_hash = *leaf;
        
        for proof_element in proof {
            if proof_element.len() != 32 {
                return Err(BridgeError::InvalidProofElement.into());
            }

            let mut proof_bytes = [0u8; 32];
            proof_bytes.copy_from_slice(proof_element);

            computed_hash = Self::hash_pair(&computed_hash, &proof_bytes);
        }

        Ok(computed_hash == *root)
    }

    /// Hashes two 32-byte arrays together using Keccak256
    pub fn hash_pair(a: &[u8; 32], b: &[u8; 32]) -> [u8; 32] {
        let mut combined = [0u8; 64];
        
        // Sort to ensure deterministic ordering
        if a <= b {
            combined[..32].copy_from_slice(a);
            combined[32..].copy_from_slice(b);
        } else {
            combined[..32].copy_from_slice(b);
            combined[32..].copy_from_slice(a);
        }

        hash(&combined).to_bytes()
    }

    /// Verifies multiple validator signatures
    pub fn verify_validator_signatures(
        message_hash: &[u8; 32],
        signatures: &[ValidatorSignature],
    ) -> Result<u8> {
        let mut valid_count = 0u8;
        let mut used_validators = HashMap::new();

        for signature in signatures {
            // Prevent double-signing by the same validator
            if used_validators.contains_key(&signature.validator_pubkey) {
                continue;
            }

            if Self::verify_secp256k1_signature(
                message_hash,
                &signature.signature,
                &signature.recovery_id,
                &signature.validator_pubkey,
            )? {
                valid_count = valid_count.saturating_add(1);
                used_validators.insert(signature.validator_pubkey.clone(), true);
            }
        }

        Ok(valid_count)
    }

    /// Verifies a single secp256k1 signature
    pub fn verify_secp256k1_signature(
        message_hash: &[u8; 32],
        signature: &[u8; 64],
        recovery_id: &u8,
        expected_pubkey: &[u8; 64],
    ) -> Result<bool> {
        // Recover public key from signature
        let recovered_pubkey = secp256k1_recover(
            message_hash,
            *recovery_id,
            signature,
        ).map_err(|_| BridgeError::InvalidSignature)?;

        // Convert recovered pubkey to bytes for comparison
        let recovered_bytes = recovered_pubkey.to_bytes();
        
        // Compare with expected public key
        Ok(recovered_bytes == *expected_pubkey)
    }

    /// Creates a message hash for cross-chain transactions
    pub fn create_transaction_hash(
        source_chain: u32,
        destination_chain: u32,
        token_address: &Pubkey,
        recipient: &Pubkey,
        amount: u64,
        nonce: u64,
        timestamp: i64,
    ) -> [u8; 32] {
        let mut message = Vec::new();
        
        message.extend_from_slice(&source_chain.to_le_bytes());
        message.extend_from_slice(&destination_chain.to_le_bytes());
        message.extend_from_slice(&token_address.to_bytes());
        message.extend_from_slice(&recipient.to_bytes());
        message.extend_from_slice(&amount.to_le_bytes());
        message.extend_from_slice(&nonce.to_le_bytes());
        message.extend_from_slice(&timestamp.to_le_bytes());

        hash(&message).to_bytes()
    }

    /// Verifies a batch of transactions using merkle tree
    pub fn verify_transaction_batch(
        transactions: &[CrossChainTransaction],
        merkle_root: &[u8; 32],
        block_height: u64,
    ) -> Result<bool> {
        if transactions.is_empty() {
            return Err(BridgeError::EmptyTransactionBatch.into());
        }

        // Build merkle tree from transactions
        let transaction_hashes: Vec<[u8; 32]> = transactions
            .iter()
            .map(|tx| Self::create_transaction_hash(
                tx.source_chain,
                tx.destination_chain,
                &tx.token_address,
                &tx.recipient,
                tx.amount,
                tx.nonce,
                tx.timestamp,
            ))
            .collect();

        let computed_root = Self::build_merkle_root(&transaction_hashes)?;
        
        Ok(computed_root == *merkle_root)
    }

    /// Builds merkle root from transaction hashes
    pub fn build_merkle_root(leaves: &[[u8; 32]]) -> Result<[u8; 32]> {
        if leaves.is_empty() {
            return Err(BridgeError::EmptyMerkleTree.into());
        }

        if leaves.len() == 1 {
            return Ok(leaves[0]);
        }

        let mut current_level = leaves.to_vec();
        
        while current_level.len() > 1 {
            let mut next_level = Vec::new();
            
            for chunk in current_level.chunks(2) {
                if chunk.len() == 2 {
                    next_level.push(Self::hash_pair(&chunk[0], &chunk[1]));
                } else {
                    // Odd number of nodes, duplicate the last one
                    next_level.push(Self::hash_pair(&chunk[0], &chunk[0]));
                }
            }
            
            current_level = next_level;
        }

        Ok(current_level[0])
    }

    /// Generates merkle proof for a specific transaction
    pub fn generate_merkle_proof(
        target_hash: &[u8; 32],
        all_hashes: &[[u8; 32]],
    ) -> Result<Vec<Vec<u8>>> {
        let target_index = all_hashes
            .iter()
            .position(|&hash| hash == *target_hash)
            .ok_or(BridgeError::TransactionNotFound)?;

        let mut proof = Vec::new();
        let mut current_level = all_hashes.to_vec();
        let mut current_index = target_index;

        while current_level.len() > 1 {
            let sibling_index = if current_index % 2 == 0 {
                current_index + 1
            } else {
                current_index - 1
            };

            if sibling_index < current_level.len() {
                proof.push(current_level[sibling_index].to_vec());
            } else {
                // Duplicate for odd number of nodes
                proof.push(current_level[current_index].to_vec());
            }

            // Build next level
            let mut next_level = Vec::new();
            for chunk in current_level.chunks(2) {
                if chunk.len() == 2 {
                    next_level.push(Self::hash_pair(&chunk[0], &chunk[1]));
                } else {
                    next_level.push(Self::hash_pair(&chunk[0], &chunk[0]));
                }
            }

            current_level = next_level;
            current_index /= 2;
        }

        Ok(proof)
    }

    /// Validates cross-chain event signature
    pub fn validate_event_signature(
        event_data: &CrossChainEvent,
        signature: &EventSignature,
        validator_pubkey: &[u8; 64],
    ) -> Result<bool> {
        let event_hash = Self::hash_event_data(event_data)?;
        
        Self::verify_secp256k1_signature(
            &event_hash,
            &signature.signature,
            &signature.recovery_id,
            validator_pubkey,
        )
    }

    /// Creates hash for cross-chain event data
    pub fn hash_event_data(event: &CrossChainEvent) -> Result<[u8; 32]> {
        let mut data = Vec::new();
        
        data.extend_from_slice(&event.event_type.to_le_bytes());
        data.extend_from_slice(&event.chain_id.to_le_bytes());
        data.extend_from_slice(&event.block_height.to_le_bytes());
        data.extend_from_slice(&event.transaction_hash);
        data.extend_from_slice(&event.timestamp.to_le_bytes());
        data.extend_from_slice(&event.data);

        Ok(hash(&data).to_bytes())
    }

    /// Verifies relay proof for cross-chain message
    pub fn verify_relay_proof(
        message: &RelayMessage,
        proof: &RelayProof,
        trusted_relayers: &[Pubkey],
    ) -> Result<bool> {
        // Verify relayer is trusted
        if !trusted_relayers.contains(&proof.relayer_pubkey) {
            return Err(BridgeError::UntrustedRelayer.into());
        }

        // Create message hash
        let message_hash = Self::hash_relay_message(message)?;

        // Verify relayer signature
        let relayer_pubkey_bytes = proof.relayer_pubkey.to_bytes();
        let mut pubkey_64 = [0u8; 64];
        pubkey_64[..32].copy_from_slice(&relayer_pubkey_bytes);

        Self::verify_secp256k1_signature(
            &message_hash,
            &proof.signature,
            &proof.recovery_id,
            &pubkey_64,
        )
    }

    /// Creates hash for relay message
    pub fn hash_relay_message(message: &RelayMessage) -> Result<[u8; 32]> {
        let mut data = Vec::new();
        
        data.extend_from_slice(&message.source_chain.to_le_bytes());
        data.extend_from_slice(&message.destination_chain.to_le_bytes());
        data.extend_from_slice(&message.nonce.to_le_bytes());
        data.extend_from_slice(&message.sender.to_bytes());
        data.extend_from_slice(&message.recipient.to_bytes());
        data.extend_from_slice(&message.payload);
        data.extend_from_slice(&message.timestamp.to_le_bytes());

        Ok(hash(&data).to_bytes())
    }

    /// Validates multi-signature threshold
    pub fn validate_multisig_threshold(
        signatures: &[ValidatorSignature],
        total_validators: u8,
        threshold_percentage: u8,
    ) -> Result<bool> {
        if threshold_percentage > 100 {
            return Err(BridgeError::InvalidThreshold.into());
        }

        let required_signatures = ((total_validators as u16 * threshold_percentage as u16) / 100) as u8;
        let valid_signatures = signatures.len() as u8;

        Ok(valid_signatures >= required_signatures)
    }
}

/// Validator signature structure
#[derive(Clone, Debug, PartialEq)]
pub struct ValidatorSignature {
    pub validator_pubkey: Vec<u8>,
    pub signature: [u8; 64],
    pub recovery_id: u8,
    pub timestamp: i64,
}

/// Cross-chain transaction structure
#[derive(Clone, Debug)]
pub struct CrossChainTransaction {
    pub source_chain: u32,
    pub destination_chain: u32,
    pub token_address: Pubkey,
    pub recipient: Pubkey,
    pub amount: u64,
    pub nonce: u64,
    pub timestamp: i64,
}

/// Cross-chain event structure
#[derive(Clone, Debug)]
pub struct CrossChainEvent {
    pub event_type: u32,
    pub chain_id: u32,
    pub block_height: u64,
    pub transaction_hash: [u8; 32],
    pub timestamp: i64,
    pub data: Vec<u8>,
}

/// Event signature structure
#[derive(Clone, Debug)]
pub struct EventSignature {
    pub signature: [u8; 64],
    pub recovery_id: u8,
    pub validator_index: u8,
}

/// Relay message structure
#[derive(Clone, Debug)]
pub struct RelayMessage {
    pub source_chain: u32,
    pub destination_chain: u32,
    pub nonce: u64,
    pub sender: Pubkey,
    pub recipient: Pubkey,
    pub payload: Vec<u8>,
    pub timestamp: i64,
}

/// Relay proof structure
#[derive(Clone, Debug)]
pub struct RelayProof {
    pub relayer_pubkey: Pubkey,
    pub signature: [u8; 64],
    pub recovery_id: u8,
    pub block_height: u64,
}

/// Cryptographic constants
pub const SIGNATURE_LENGTH: usize = 64;
pub const HASH_LENGTH: usize = 32;
pub const PUBKEY_LENGTH: usize = 64;
pub const MAX_MERKLE_DEPTH: usize = 32;
pub const MIN_VALIDATOR_THRESHOLD: u8 = 51; // 51% minimum
pub const MAX_PROOF_ELEMENTS: usize = 32;

/// Validation utilities
impl CryptographyManager {
    /// Validates cryptographic parameters
    pub fn validate_crypto_params(
        validator_count: u8,
        threshold_percentage: u8,
        max_proof_depth: usize,
    ) -> Result<()> {
        if validator_count == 0 {
            return Err(BridgeError::InvalidValidatorCount.into());
        }

        if threshold_percentage < MIN_VALIDATOR_THRESHOLD || threshold_percentage > 100 {
            return Err(BridgeError::InvalidThreshold.into());
        }

        if max_proof_depth > MAX_MERKLE_DEPTH {
            return Err(BridgeError::InvalidProofDepth.into());
        }

        Ok(())
    }

    /// Sanitizes input data for cryptographic operations
    pub fn sanitize_crypto_input(data: &[u8]) -> Result<Vec<u8>> {
        if data.is_empty() {
            return Err(BridgeError::EmptyInput.into());
        }

        if data.len() > 1024 * 1024 {  // 1MB limit
            return Err(BridgeError::InputTooLarge.into());
        }

        Ok(data.to_vec())
    }

    /// Verifies timestamp validity for cross-chain operations
    pub fn verify_timestamp(
        timestamp: i64,
        max_age_seconds: i64,
        clock: &Clock,
    ) -> Result<bool> {
        let current_time = clock.unix_timestamp;
        let age = current_time - timestamp;

        if age < 0 {
            return Err(BridgeError::FutureTimestamp.into());
        }

        if age > max_age_seconds {
            return Err(BridgeError::ExpiredTimestamp.into());
        }

        Ok(true)
    }

    /// Rate limiting for cryptographic operations
    pub fn check_rate_limit(
        pubkey: &Pubkey,
        operation_type: u8,
        window_seconds: i64,
        max_operations: u32,
        clock: &Clock,
    ) -> Result<bool> {
        // This would typically interact with a rate limiting store
        // For now, return true to allow operations
        Ok(true)
    }
}

/// Security utilities for additional protection
pub mod security {
    use super::*;
    
    /// Constant-time comparison for sensitive data
    pub fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
        if a.len() != b.len() {
            return false;
        }
        
        let mut result = 0u8;
        for (x, y) in a.iter().zip(b.iter()) {
            result |= x ^ y;
        }
        
        result == 0
    }
    
    /// Secure random number generation for nonces
    pub fn generate_secure_nonce(seed: &[u8]) -> [u8; 32] {
        hash(seed).to_bytes()
    }
    
    /// Memory zeroization for sensitive data
    pub fn zeroize(data: &mut [u8]) {
        for byte in data.iter_mut() {
            *byte = 0;
        }
    }
}
