// programs/finova-bridge/src/errors.rs

use anchor_lang::prelude::*;

/// Custom error codes for the Finova Bridge program
#[error_code]
pub enum BridgeError {
    // Bridge Configuration Errors
    #[msg("Bridge is not initialized")]
    BridgeNotInitialized,
    
    #[msg("Bridge is already initialized")]
    BridgeAlreadyInitialized,
    
    #[msg("Invalid bridge configuration")]
    InvalidBridgeConfig,
    
    #[msg("Bridge is paused")]
    BridgePaused,
    
    #[msg("Bridge is not paused")]
    BridgeNotPaused,
    
    // Authority & Permission Errors
    #[msg("Unauthorized: Invalid authority")]
    UnauthorizedAuthority,
    
    #[msg("Unauthorized: Invalid admin")]
    UnauthorizedAdmin,
    
    #[msg("Unauthorized: Invalid validator")]
    UnauthorizedValidator,
    
    #[msg("Unauthorized: Invalid relayer")]
    UnauthorizedRelayer,
    
    #[msg("Insufficient permissions")]
    InsufficientPermissions,
    
    // Token & Asset Errors
    #[msg("Unsupported token for bridging")]
    UnsupportedToken,
    
    #[msg("Insufficient token balance")]
    InsufficientBalance,
    
    #[msg("Invalid token amount")]
    InvalidTokenAmount,
    
    #[msg("Token amount exceeds maximum bridge limit")]
    ExceedsMaxBridgeAmount,
    
    #[msg("Token amount below minimum bridge threshold")]
    BelowMinBridgeAmount,
    
    #[msg("Daily bridge limit exceeded")]
    DailyLimitExceeded,
    
    #[msg("Token transfer failed")]
    TokenTransferFailed,
    
    // Validation & Proof Errors
    #[msg("Invalid Merkle proof")]
    InvalidMerkleProof,
    
    #[msg("Invalid signature")]
    InvalidSignature,
    
    #[msg("Signature verification failed")]
    SignatureVerificationFailed,
    
    #[msg("Invalid validator signature")]
    InvalidValidatorSignature,
    
    #[msg("Insufficient validator signatures")]
    InsufficientValidatorSignatures,
    
    #[msg("Duplicate validator signature")]
    DuplicateValidatorSignature,
    
    #[msg("Invalid proof format")]
    InvalidProofFormat,
    
    #[msg("Proof verification timeout")]
    ProofVerificationTimeout,
    
    // Transaction & State Errors
    #[msg("Transaction already processed")]
    TransactionAlreadyProcessed,
    
    #[msg("Transaction not found")]
    TransactionNotFound,
    
    #[msg("Invalid transaction hash")]
    InvalidTransactionHash,
    
    #[msg("Transaction expired")]
    TransactionExpired,
    
    #[msg("Invalid transaction nonce")]
    InvalidTransactionNonce,
    
    #[msg("Nonce already used")]
    NonceAlreadyUsed,
    
    #[msg("Invalid block height")]
    InvalidBlockHeight,
    
    #[msg("Block not confirmed")]
    BlockNotConfirmed,
    
    // Network & Chain Errors
    #[msg("Invalid source chain")]
    InvalidSourceChain,
    
    #[msg("Invalid destination chain")]
    InvalidDestinationChain,
    
    #[msg("Chain not supported")]
    ChainNotSupported,
    
    #[msg("Cross-chain communication failed")]
    CrossChainCommunicationFailed,
    
    #[msg("Invalid chain configuration")]
    InvalidChainConfig,
    
    #[msg("Chain synchronization error")]
    ChainSyncError,
    
    // Lock/Unlock Mechanism Errors
    #[msg("Token lock failed")]
    TokenLockFailed,
    
    #[msg("Token unlock failed")]
    TokenUnlockFailed,
    
    #[msg("Invalid lock period")]
    InvalidLockPeriod,
    
    #[msg("Lock period not expired")]
    LockPeriodNotExpired,
    
    #[msg("Token already locked")]
    TokenAlreadyLocked,
    
    #[msg("Token not locked")]
    TokenNotLocked,
    
    #[msg("Lock amount mismatch")]
    LockAmountMismatch,
    
    // Vault & Custody Errors
    #[msg("Vault insufficient funds")]
    VaultInsufficientFunds,
    
    #[msg("Vault capacity exceeded")]
    VaultCapacityExceeded,
    
    #[msg("Invalid vault configuration")]
    InvalidVaultConfig,
    
    #[msg("Vault access denied")]
    VaultAccessDenied,
    
    #[msg("Emergency vault freeze active")]
    VaultEmergencyFreeze,
    
    // Fee & Economics Errors
    #[msg("Invalid bridge fee")]
    InvalidBridgeFee,
    
    #[msg("Insufficient fee payment")]
    InsufficientFeePayment,
    
    #[msg("Fee calculation error")]
    FeeCalculationError,
    
    #[msg("Fee collection failed")]
    FeeCollectionFailed,
    
    #[msg("Invalid fee recipient")]
    InvalidFeeRecipient,
    
    // Time & Scheduling Errors
    #[msg("Invalid timestamp")]
    InvalidTimestamp,
    
    #[msg("Future timestamp not allowed")]
    FutureTimestampNotAllowed,
    
    #[msg("Timestamp too old")]
    TimestampTooOld,
    
    #[msg("Invalid time window")]
    InvalidTimeWindow,
    
    #[msg("Operation timeout")]
    OperationTimeout,
    
    // Validator Set Errors
    #[msg("Invalid validator set")]
    InvalidValidatorSet,
    
    #[msg("Validator set update failed")]
    ValidatorSetUpdateFailed,
    
    #[msg("Minimum validator threshold not met")]
    MinValidatorThresholdNotMet,
    
    #[msg("Maximum validator limit exceeded")]
    MaxValidatorLimitExceeded,
    
    #[msg("Validator already exists")]
    ValidatorAlreadyExists,
    
    #[msg("Validator not found")]
    ValidatorNotFound,
    
    #[msg("Validator is inactive")]
    ValidatorInactive,
    
    #[msg("Validator stake insufficient")]
    ValidatorStakeInsufficient,
    
    // Relay & Communication Errors
    #[msg("Relay message failed")]
    RelayMessageFailed,
    
    #[msg("Invalid relay data")]
    InvalidRelayData,
    
    #[msg("Relay timeout")]
    RelayTimeout,
    
    #[msg("Relay verification failed")]
    RelayVerificationFailed,
    
    #[msg("Communication protocol error")]
    CommunicationProtocolError,
    
    // Emergency & Security Errors
    #[msg("Emergency pause activated")]
    EmergencyPauseActivated,
    
    #[msg("Security breach detected")]
    SecurityBreachDetected,
    
    #[msg("Suspicious activity detected")]
    SuspiciousActivityDetected,
    
    #[msg("Rate limit exceeded")]
    RateLimitExceeded,
    
    #[msg("Anti-fraud check failed")]
    AntiFraudCheckFailed,
    
    #[msg("Blacklisted address")]
    BlacklistedAddress,
    
    // Cryptographic Errors
    #[msg("Hash verification failed")]
    HashVerificationFailed,
    
    #[msg("Invalid hash format")]
    InvalidHashFormat,
    
    #[msg("Encryption failed")]
    EncryptionFailed,
    
    #[msg("Decryption failed")]
    DecryptionFailed,
    
    #[msg("Key derivation failed")]
    KeyDerivationFailed,
    
    #[msg("Invalid public key")]
    InvalidPublicKey,
    
    #[msg("Invalid private key")]
    InvalidPrivateKey,
    
    // Data & Serialization Errors
    #[msg("Serialization error")]
    SerializationError,
    
    #[msg("Deserialization error")]
    DeserializationError,
    
    #[msg("Invalid data format")]
    InvalidDataFormat,
    
    #[msg("Data corruption detected")]
    DataCorruption,
    
    #[msg("Encoding error")]
    EncodingError,
    
    #[msg("Decoding error")]
    DecodingError,
    
    // Account & PDA Errors
    #[msg("Invalid account data")]
    InvalidAccountData,
    
    #[msg("Account not owned by program")]
    AccountNotOwnedByProgram,
    
    #[msg("Invalid PDA derivation")]
    InvalidPDADerivation,
    
    #[msg("PDA seeds mismatch")]
    PDASeedsMismatch,
    
    #[msg("Account size insufficient")]
    AccountSizeInsufficient,
    
    // Math & Calculation Errors
    #[msg("Mathematical overflow")]
    MathematicalOverflow,
    
    #[msg("Mathematical underflow")]
    MathematicalUnderflow,
    
    #[msg("Division by zero")]
    DivisionByZero,
    
    #[msg("Invalid calculation result")]
    InvalidCalculationResult,
    
    #[msg("Precision loss detected")]
    PrecisionLoss,
    
    // Configuration & Parameter Errors
    #[msg("Invalid parameter value")]
    InvalidParameterValue,
    
    #[msg("Parameter out of range")]
    ParameterOutOfRange,
    
    #[msg("Missing required parameter")]
    MissingRequiredParameter,
    
    #[msg("Conflicting parameters")]
    ConflictingParameters,
    
    #[msg("Configuration validation failed")]
    ConfigurationValidationFailed,
    
    // System & Resource Errors
    #[msg("System overloaded")]
    SystemOverloaded,
    
    #[msg("Resource unavailable")]
    ResourceUnavailable,
    
    #[msg("Memory allocation failed")]
    MemoryAllocationFailed,
    
    #[msg("Storage capacity exceeded")]
    StorageCapacityExceeded,
    
    #[msg("Network congestion")]
    NetworkCongestion,
    
    // Version & Compatibility Errors
    #[msg("Incompatible version")]
    IncompatibleVersion,
    
    #[msg("Upgrade required")]
    UpgradeRequired,
    
    #[msg("Deprecated feature")]
    DeprecatedFeature,
    
    #[msg("Feature not supported")]
    FeatureNotSupported,
    
    // Custom Protocol Errors
    #[msg("Invalid protocol message")]
    InvalidProtocolMessage,
    
    #[msg("Protocol version mismatch")]
    ProtocolVersionMismatch,
    
    #[msg("Protocol handshake failed")]
    ProtocolHandshakeFailed,
    
    #[msg("Protocol state error")]
    ProtocolStateError,
    
    // Finova-Specific Errors
    #[msg("Mining integration failed")]
    MiningIntegrationFailed,
    
    #[msg("XP calculation error")]
    XPCalculationError,
    
    #[msg("Referral validation failed")]
    ReferralValidationFailed,
    
    #[msg("NFT bridge not supported")]
    NFTBridgeNotSupported,
    
    #[msg("Staking integration error")]
    StakingIntegrationError,
    
    #[msg("Invalid Finova token")]
    InvalidFinovaToken,
    
    #[msg("Bridge reward calculation failed")]
    BridgeRewardCalculationFailed,
    
    #[msg("Cross-chain XP sync failed")]
    CrossChainXPSyncFailed,
    
    #[msg("Multi-chain referral error")]
    MultiChainReferralError,
    
    #[msg("Bridge governance violation")]
    BridgeGovernanceViolation,
}

impl From<BridgeError> for ProgramError {
    fn from(e: BridgeError) -> Self {
        ProgramError::Custom(e as u32)
    }
}

impl From<BridgeError> for Error {
    fn from(error: BridgeError) -> Self {
        Error::from(error)
    }
}

/// Bridge error result type
pub type BridgeResult<T> = Result<T, BridgeError>;

/// Utility macro for creating bridge-specific errors
#[macro_export]
macro_rules! bridge_error {
    ($error:expr) => {
        Err($crate::errors::BridgeError::$error.into())
    };
    ($error:expr, $msg:expr) => {
        Err($crate::errors::BridgeError::$error.into()).map_err(|e| {
            msg!($msg);
            e
        })
    };
}

/// Validation helper macros
#[macro_export]
macro_rules! require_bridge {
    ($condition:expr, $error:expr) => {
        if !($condition) {
            return bridge_error!($error);
        }
    };
    ($condition:expr, $error:expr, $msg:expr) => {
        if !($condition) {
            return bridge_error!($error, $msg);
        }
    };
}

/// Authority validation macro
#[macro_export]
macro_rules! require_authority {
    ($expected:expr, $actual:expr) => {
        require_bridge!(
            $expected == $actual,
            UnauthorizedAuthority,
            "Authority mismatch in bridge operation"
        );
    };
}

/// Validator signature verification macro
#[macro_export]
macro_rules! require_validator_signature {
    ($signature_count:expr, $required:expr) => {
        require_bridge!(
            $signature_count >= $required,
            InsufficientValidatorSignatures,
            "Insufficient validator signatures for bridge operation"
        );
    };
}

/// Bridge state validation macro
#[macro_export]
macro_rules! require_bridge_active {
    ($is_paused:expr) => {
        require_bridge!(
            !$is_paused,
            BridgePaused,
            "Bridge is currently paused"
        );
    };
}

/// Amount validation macro
#[macro_export]
macro_rules! require_valid_amount {
    ($amount:expr, $min:expr, $max:expr) => {
        require_bridge!(
            $amount >= $min,
            BelowMinBridgeAmount,
            "Amount below minimum bridge threshold"
        );
        require_bridge!(
            $amount <= $max,
            ExceedsMaxBridgeAmount,
            "Amount exceeds maximum bridge limit"
        );
    };
}

/// Time validation macro
#[macro_export]
macro_rules! require_valid_timestamp {
    ($timestamp:expr, $current:expr, $max_age:expr) => {
        require_bridge!(
            $timestamp <= $current,
            FutureTimestampNotAllowed,
            "Future timestamps not allowed"
        );
        require_bridge!(
            $current - $timestamp <= $max_age,
            TimestampTooOld,
            "Timestamp too old for processing"
        );
    };
}

/// Chain validation macro
#[macro_export]
macro_rules! require_supported_chain {
    ($chain_id:expr, $supported_chains:expr) => {
        require_bridge!(
            $supported_chains.contains(&$chain_id),
            ChainNotSupported,
            "Chain not supported by bridge"
        );
    };
}

/// Token validation macro
#[macro_export]
macro_rules! require_supported_token {
    ($token_mint:expr, $supported_tokens:expr) => {
        require_bridge!(
            $supported_tokens.contains(&$token_mint),
            UnsupportedToken,
            "Token not supported for bridging"
        );
    };
}

/// Vault balance validation macro
#[macro_export]
macro_rules! require_vault_balance {
    ($vault_balance:expr, $required_amount:expr) => {
        require_bridge!(
            $vault_balance >= $required_amount,
            VaultInsufficientFunds,
            "Vault has insufficient funds for operation"
        );
    };
}

/// Nonce validation macro
#[macro_export]
macro_rules! require_valid_nonce {
    ($nonce:expr, $expected_nonce:expr) => {
        require_bridge!(
            $nonce == $expected_nonce,
            InvalidTransactionNonce,
            "Transaction nonce mismatch"
        );
    };
}

/// Anti-replay protection macro
#[macro_export]
macro_rules! require_not_processed {
    ($is_processed:expr) => {
        require_bridge!(
            !$is_processed,
            TransactionAlreadyProcessed,
            "Transaction has already been processed"
        );
    };
}

/// Emergency controls macro
#[macro_export]
macro_rules! require_not_emergency {
    ($emergency_mode:expr) => {
        require_bridge!(
            !$emergency_mode,
            EmergencyPauseActivated,
            "Emergency pause is currently active"
        );
    };
}

/// Rate limiting macro
#[macro_export]
macro_rules! require_rate_limit {
    ($current_count:expr, $limit:expr, $window:expr) => {
        require_bridge!(
            $current_count < $limit,
            RateLimitExceeded,
            format!("Rate limit exceeded: {} operations in {} seconds", $current_count, $window)
        );
    };
}

/// Daily limit validation macro
#[macro_export]
macro_rules! require_daily_limit {
    ($daily_amount:expr, $amount:expr, $daily_limit:expr) => {
        require_bridge!(
            $daily_amount + $amount <= $daily_limit,
            DailyLimitExceeded,
            "Daily bridge limit would be exceeded"
        );
    };
}

/// Validator set validation macro
#[macro_export]
macro_rules! require_valid_validator_set {
    ($validator_count:expr, $min_validators:expr, $max_validators:expr) => {
        require_bridge!(
            $validator_count >= $min_validators,
            MinValidatorThresholdNotMet,
            "Minimum validator threshold not met"
        );
        require_bridge!(
            $validator_count <= $max_validators,
            MaxValidatorLimitExceeded,
            "Maximum validator limit exceeded"
        );
    };
}

/// Cross-chain message validation macro
#[macro_export]
macro_rules! require_valid_message {
    ($message_hash:expr, $expected_hash:expr) => {
        require_bridge!(
            $message_hash == $expected_hash,
            HashVerificationFailed,
            "Cross-chain message hash verification failed"
        );
    };
}

/// Bridge fee validation macro
#[macro_export]
macro_rules! require_valid_fee {
    ($provided_fee:expr, $required_fee:expr) => {
        require_bridge!(
            $provided_fee >= $required_fee,
            InsufficientFeePayment,
            "Insufficient bridge fee provided"
        );
    };
}

/// Security check macro for suspicious activity
#[macro_export]
macro_rules! require_not_suspicious {
    ($is_suspicious:expr) => {
        require_bridge!(
            !$is_suspicious,
            SuspiciousActivityDetected,
            "Suspicious activity detected, operation blocked"
        );
    };
}

/// Whitelist validation macro
#[macro_export]
macro_rules! require_not_blacklisted {
    ($address:expr, $blacklist:expr) => {
        require_bridge!(
            !$blacklist.contains(&$address),
            BlacklistedAddress,
            "Address is blacklisted from bridge operations"
        );
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_codes() {
        // Test that error codes are properly defined
        let error = BridgeError::BridgeNotInitialized;
        assert_eq!(error as u32, 6000);
        
        let program_error: ProgramError = error.into();
        assert_eq!(program_error, ProgramError::Custom(6000));
    }

    #[test]
    fn test_error_messages() {
        // Test that error messages are descriptive
        let error = BridgeError::InvalidMerkleProof;
        let error_msg = format!("{}", error);
        assert!(error_msg.contains("Invalid Merkle proof"));
    }

    #[test]
    fn test_bridge_result() {
        // Test BridgeResult type
        let success: BridgeResult<u64> = Ok(100);
        assert!(success.is_ok());
        
        let failure: BridgeResult<u64> = Err(BridgeError::InsufficientBalance);
        assert!(failure.is_err());
    }
}
