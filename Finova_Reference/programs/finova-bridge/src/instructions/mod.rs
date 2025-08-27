// programs/finova-bridge/src/instructions/mod.rs

use anchor_lang::prelude::*;
use crate::constants::*;
use crate::errors::*;
use crate::state::*;
use crate::cryptography::*;

pub mod initialize_bridge;
pub mod lock_tokens;
pub mod unlock_tokens;
pub mod validate_proof;
pub mod emergency_pause;

pub use initialize_bridge::*;
pub use lock_tokens::*;
pub use unlock_tokens::*;
pub use validate_proof::*;
pub use emergency_pause::*;

/// Bridge instruction discriminator enum
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum BridgeInstruction {
    InitializeBridge {
        chain_id: u64,
        initial_validators: Vec<Pubkey>,
        threshold: u8,
        fee_rate: u64,
    },
    LockTokens {
        amount: u64,
        destination_chain: u64,
        destination_address: [u8; 32],
        token_type: TokenType,
    },
    UnlockTokens {
        amount: u64,
        recipient: Pubkey,
        transaction_hash: [u8; 32],
        merkle_proof: Vec<[u8; 32]>,
        validator_signatures: Vec<ValidatorSignature>,
    },
    ValidateProof {
        proof_data: ProofData,
        validator_signatures: Vec<ValidatorSignature>,
    },
    EmergencyPause {
        pause_type: PauseType,
        reason: String,
    },
    UpdateValidators {
        new_validators: Vec<Pubkey>,
        new_threshold: u8,
    },
    SetFeeRate {
        new_fee_rate: u64,
    },
    WithdrawFees {
        amount: u64,
        recipient: Pubkey,
    },
    Resume {
        resume_type: PauseType,
    },
}

/// Token types supported by the bridge
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum TokenType {
    Fin,       // $FIN token
    SFin,      // $sFIN staked token
    UsdFin,    // $USDfin stablecoin
    SUsdFin,   // $sUSDfin staked stablecoin
    Custom(Pubkey), // Custom SPL token
}

impl TokenType {
    pub fn get_mint_address(&self) -> Result<Pubkey> {
        match self {
            TokenType::Fin => Ok(FIN_MINT_PUBKEY),
            TokenType::SFin => Ok(SFIN_MINT_PUBKEY),
            TokenType::UsdFin => Ok(USDFIN_MINT_PUBKEY),
            TokenType::SUsdFin => Ok(SUSDFIN_MINT_PUBKEY),
            TokenType::Custom(mint) => Ok(*mint),
        }
    }

    pub fn is_supported(&self) -> bool {
        match self {
            TokenType::Fin | TokenType::SFin | TokenType::UsdFin | TokenType::SUsdFin => true,
            TokenType::Custom(_) => false, // Custom tokens require additional validation
        }
    }

    pub fn get_decimals(&self) -> u8 {
        match self {
            TokenType::Fin => 9,
            TokenType::SFin => 9,
            TokenType::UsdFin => 6,
            TokenType::SUsdFin => 6,
            TokenType::Custom(_) => 9, // Default, should be validated
        }
    }
}

/// Pause types for emergency controls
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum PauseType {
    All,           // Pause all bridge operations
    Deposits,      // Pause only deposits (lock operations)
    Withdrawals,   // Pause only withdrawals (unlock operations)
    Validator,     // Pause validator operations
}

/// Validator signature structure
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub struct ValidatorSignature {
    pub validator: Pubkey,
    pub signature: [u8; 64],
    pub recovery_id: u8,
    pub timestamp: i64,
}

impl ValidatorSignature {
    pub fn new(validator: Pubkey, signature: [u8; 64], recovery_id: u8) -> Self {
        Self {
            validator,
            signature,
            recovery_id,
            timestamp: Clock::get().unwrap().unix_timestamp,
        }
    }

    pub fn verify(&self, message: &[u8], validator_pubkey: &Pubkey) -> Result<bool> {
        // Verify that the signature matches the validator
        if &self.validator != validator_pubkey {
            return Ok(false);
        }

        // Verify signature hasn't expired (24 hour window)
        let current_time = Clock::get()?.unix_timestamp;
        if current_time - self.timestamp > SIGNATURE_VALIDITY_PERIOD {
            return Ok(false);
        }

        // Verify the actual signature
        signature_verification::verify_signature(
            message,
            &self.signature,
            self.recovery_id,
            validator_pubkey
        )
    }
}

/// Proof data structure for cross-chain verification
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub struct ProofData {
    pub transaction_hash: [u8; 32],
    pub block_hash: [u8; 32],
    pub merkle_root: [u8; 32],
    pub merkle_proof: Vec<[u8; 32]>,
    pub chain_id: u64,
    pub amount: u64,
    pub recipient: Pubkey,
    pub token_type: TokenType,
    pub nonce: u64,
}

impl ProofData {
    pub fn new(
        transaction_hash: [u8; 32],
        block_hash: [u8; 32],
        merkle_root: [u8; 32],
        merkle_proof: Vec<[u8; 32]>,
        chain_id: u64,
        amount: u64,
        recipient: Pubkey,
        token_type: TokenType,
        nonce: u64,
    ) -> Self {
        Self {
            transaction_hash,
            block_hash,
            merkle_root,
            merkle_proof,
            chain_id,
            amount,
            recipient,
            token_type,
            nonce,
        }
    }

    pub fn hash(&self) -> Result<[u8; 32]> {
        let mut data = Vec::new();
        data.extend_from_slice(&self.transaction_hash);
        data.extend_from_slice(&self.block_hash);
        data.extend_from_slice(&self.merkle_root);
        data.extend_from_slice(&self.chain_id.to_le_bytes());
        data.extend_from_slice(&self.amount.to_le_bytes());
        data.extend_from_slice(&self.recipient.to_bytes());
        data.extend_from_slice(&(self.token_type.clone() as u8).to_le_bytes());
        data.extend_from_slice(&self.nonce.to_le_bytes());

        Ok(solana_program::keccak::hash(&data).to_bytes())
    }

    pub fn verify_merkle_proof(&self) -> Result<bool> {
        merkle_proof::verify_merkle_proof(
            &self.transaction_hash,
            &self.merkle_proof,
            &self.merkle_root
        )
    }
}

/// Bridge operation result
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub struct BridgeOperationResult {
    pub operation_id: u64,
    pub transaction_hash: [u8; 32],
    pub amount: u64,
    pub fee: u64,
    pub timestamp: i64,
    pub status: OperationStatus,
}

/// Operation status enum
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum OperationStatus {
    Pending,
    Confirmed,
    Failed,
    Cancelled,
}

/// Bridge fee calculator
pub struct FeeCalculator;

impl FeeCalculator {
    pub fn calculate_lock_fee(amount: u64, fee_rate: u64) -> Result<u64> {
        let fee = amount
            .checked_mul(fee_rate)
            .ok_or(BridgeError::MathOverflow)?
            .checked_div(FEE_DENOMINATOR)
            .ok_or(BridgeError::MathOverflow)?;

        // Minimum fee check
        Ok(fee.max(MIN_BRIDGE_FEE))
    }

    pub fn calculate_unlock_fee(amount: u64, fee_rate: u64) -> Result<u64> {
        let fee = amount
            .checked_mul(fee_rate)
            .ok_or(BridgeError::MathOverflow)?
            .checked_div(FEE_DENOMINATOR)
            .ok_or(BridgeError::MathOverflow)?;

        // Minimum fee check
        Ok(fee.max(MIN_BRIDGE_FEE))
    }

    pub fn calculate_validator_reward(total_fee: u64, validator_count: u8) -> Result<u64> {
        if validator_count == 0 {
            return Ok(0);
        }

        let validator_portion = total_fee
            .checked_mul(VALIDATOR_REWARD_PERCENTAGE)
            .ok_or(BridgeError::MathOverflow)?
            .checked_div(100)
            .ok_or(BridgeError::MathOverflow)?;

        validator_portion
            .checked_div(validator_count as u64)
            .ok_or(BridgeError::MathOverflow.into())
    }
}

/// Bridge state validator
pub struct BridgeValidator;

impl BridgeValidator {
    pub fn validate_chain_id(chain_id: u64) -> Result<()> {
        if !SUPPORTED_CHAIN_IDS.contains(&chain_id) {
            return Err(BridgeError::UnsupportedChain.into());
        }
        Ok(())
    }

    pub fn validate_amount(amount: u64, token_type: &TokenType) -> Result<()> {
        if amount == 0 {
            return Err(BridgeError::InvalidAmount.into());
        }

        let min_amount = match token_type {
            TokenType::Fin => MIN_BRIDGE_AMOUNT_FIN,
            TokenType::SFin => MIN_BRIDGE_AMOUNT_SFIN,
            TokenType::UsdFin => MIN_BRIDGE_AMOUNT_USDFIN,
            TokenType::SUsdFin => MIN_BRIDGE_AMOUNT_SUSDFIN,
            TokenType::Custom(_) => MIN_BRIDGE_AMOUNT_CUSTOM,
        };

        if amount < min_amount {
            return Err(BridgeError::AmountTooSmall.into());
        }

        let max_amount = match token_type {
            TokenType::Fin => MAX_BRIDGE_AMOUNT_FIN,
            TokenType::SFin => MAX_BRIDGE_AMOUNT_SFIN,
            TokenType::UsdFin => MAX_BRIDGE_AMOUNT_USDFIN,
            TokenType::SUsdFin => MAX_BRIDGE_AMOUNT_SUSDFIN,
            TokenType::Custom(_) => MAX_BRIDGE_AMOUNT_CUSTOM,
        };

        if amount > max_amount {
            return Err(BridgeError::AmountTooLarge.into());
        }

        Ok(())
    }

    pub fn validate_validator_threshold(validator_count: u8, threshold: u8) -> Result<()> {
        if threshold == 0 {
            return Err(BridgeError::InvalidThreshold.into());
        }

        if threshold > validator_count {
            return Err(BridgeError::ThresholdTooHigh.into());
        }

        // Require at least 51% for security
        let min_threshold = (validator_count as f64 * 0.51).ceil() as u8;
        if threshold < min_threshold {
            return Err(BridgeError::ThresholdTooLow.into());
        }

        Ok(())
    }

    pub fn validate_signatures(
        signatures: &[ValidatorSignature],
        validators: &[Pubkey],
        threshold: u8,
        message: &[u8],
    ) -> Result<()> {
        if signatures.len() < threshold as usize {
            return Err(BridgeError::InsufficientSignatures.into());
        }

        let mut valid_signatures = 0u8;
        let mut used_validators = std::collections::HashSet::new();

        for signature in signatures {
            // Check if validator is in the validator set
            if !validators.contains(&signature.validator) {
                continue;
            }

            // Check for duplicate validators
            if used_validators.contains(&signature.validator) {
                continue;
            }

            // Verify signature
            if signature.verify(message, &signature.validator)? {
                valid_signatures += 1;
                used_validators.insert(signature.validator);
            }

            // Early exit if we have enough valid signatures
            if valid_signatures >= threshold {
                break;
            }
        }

        if valid_signatures < threshold {
            return Err(BridgeError::InsufficientValidSignatures.into());
        }

        Ok(())
    }

    pub fn validate_nonce(nonce: u64, last_nonce: u64) -> Result<()> {
        if nonce <= last_nonce {
            return Err(BridgeError::InvalidNonce.into());
        }

        // Prevent nonce too far in the future (replay attack protection)
        if nonce > last_nonce + MAX_NONCE_GAP {
            return Err(BridgeError::NonceTooHigh.into());
        }

        Ok(())
    }
}

/// Bridge event emitter
pub struct BridgeEventEmitter;

impl BridgeEventEmitter {
    pub fn emit_lock_event(
        user: Pubkey,
        amount: u64,
        fee: u64,
        destination_chain: u64,
        destination_address: [u8; 32],
        token_type: TokenType,
        nonce: u64,
    ) -> Result<()> {
        emit!(crate::events::BridgeLockEvent {
            user,
            amount,
            fee,
            destination_chain,
            destination_address,
            token_type,
            nonce,
            timestamp: Clock::get()?.unix_timestamp,
        });
        Ok(())
    }

    pub fn emit_unlock_event(
        user: Pubkey,
        amount: u64,
        fee: u64,
        source_chain: u64,
        transaction_hash: [u8; 32],
        token_type: TokenType,
    ) -> Result<()> {
        emit!(crate::events::BridgeUnlockEvent {
            user,
            amount,
            fee,
            source_chain,
            transaction_hash,
            token_type,
            timestamp: Clock::get()?.unix_timestamp,
        });
        Ok(())
    }

    pub fn emit_validator_update_event(
        old_validators: Vec<Pubkey>,
        new_validators: Vec<Pubkey>,
        old_threshold: u8,
        new_threshold: u8,
    ) -> Result<()> {
        emit!(crate::events::ValidatorUpdateEvent {
            old_validators,
            new_validators,
            old_threshold,
            new_threshold,
            timestamp: Clock::get()?.unix_timestamp,
        });
        Ok(())
    }

    pub fn emit_emergency_pause_event(
        pause_type: PauseType,
        reason: String,
        admin: Pubkey,
    ) -> Result<()> {
        emit!(crate::events::EmergencyPauseEvent {
            pause_type,
            reason,
            admin,
            timestamp: Clock::get()?.unix_timestamp,
        });
        Ok(())
    }
}

/// Utility functions for bridge operations
pub mod bridge_utils {
    use super::*;

    pub fn generate_operation_id() -> u64 {
        Clock::get().unwrap().unix_timestamp as u64
    }

    pub fn calculate_destination_address(recipient: &Pubkey, chain_id: u64) -> [u8; 32] {
        let mut data = Vec::new();
        data.extend_from_slice(&recipient.to_bytes());
        data.extend_from_slice(&chain_id.to_le_bytes());
        solana_program::keccak::hash(&data).to_bytes()
    }

    pub fn is_emergency_admin(admin: &Pubkey) -> bool {
        EMERGENCY_ADMINS.contains(admin)
    }

    pub fn get_bridge_version() -> u8 {
        BRIDGE_VERSION
    }

    pub fn format_transaction_hash(hash: &[u8; 32]) -> String {
        hex::encode(hash)
    }

    pub fn parse_transaction_hash(hash_str: &str) -> Result<[u8; 32]> {
        let bytes = hex::decode(hash_str).map_err(|_| BridgeError::InvalidTransactionHash)?;
        if bytes.len() != 32 {
            return Err(BridgeError::InvalidTransactionHash.into());
        }
        let mut hash = [0u8; 32];
        hash.copy_from_slice(&bytes);
        Ok(hash)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_type_mint_address() {
        assert_eq!(TokenType::Fin.get_mint_address().unwrap(), FIN_MINT_PUBKEY);
        assert_eq!(TokenType::SFin.get_mint_address().unwrap(), SFIN_MINT_PUBKEY);
        assert_eq!(TokenType::UsdFin.get_mint_address().unwrap(), USDFIN_MINT_PUBKEY);
        assert_eq!(TokenType::SUsdFin.get_mint_address().unwrap(), SUSDFIN_MINT_PUBKEY);
    }

    #[test]
    fn test_fee_calculation() {
        let amount = 1000_000_000; // 1000 tokens
        let fee_rate = 100; // 1% (100/10000)
        let expected_fee = 10_000_000; // 10 tokens

        let fee = FeeCalculator::calculate_lock_fee(amount, fee_rate).unwrap();
        assert_eq!(fee, expected_fee);
    }

    #[test]
    fn test_validator_threshold_validation() {
        assert!(BridgeValidator::validate_validator_threshold(5, 3).is_ok());
        assert!(BridgeValidator::validate_validator_threshold(5, 2).is_err()); // Too low
        assert!(BridgeValidator::validate_validator_threshold(5, 6).is_err()); // Too high
        assert!(BridgeValidator::validate_validator_threshold(5, 0).is_err()); // Zero
    }

    #[test]
    fn test_amount_validation() {
        assert!(BridgeValidator::validate_amount(1000, &TokenType::Fin).is_ok());
        assert!(BridgeValidator::validate_amount(0, &TokenType::Fin).is_err());
        assert!(BridgeValidator::validate_amount(u64::MAX, &TokenType::Fin).is_err());
    }

    #[test]
    fn test_nonce_validation() {
        assert!(BridgeValidator::validate_nonce(5, 4).is_ok());
        assert!(BridgeValidator::validate_nonce(4, 4).is_err()); // Same nonce
        assert!(BridgeValidator::validate_nonce(3, 4).is_err()); // Lower nonce
    }

    #[test]
    fn test_proof_data_hash() {
        let proof = ProofData::new(
            [1u8; 32],
            [2u8; 32],
            [3u8; 32],
            vec![[4u8; 32]],
            1,
            1000,
            Pubkey::default(),
            TokenType::Fin,
            1,
        );

        let hash1 = proof.hash().unwrap();
        let hash2 = proof.hash().unwrap();
        assert_eq!(hash1, hash2); // Should be deterministic
    }
}
