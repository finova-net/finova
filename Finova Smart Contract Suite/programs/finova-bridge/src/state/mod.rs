// programs/finova-bridge/src/state/mod.rs

use anchor_lang::prelude::*;
use crate::constants::*;
use crate::errors::FinovaBridgeError;

/// Re-export all state structs for easy access
pub mod bridge_config;
pub mod locked_tokens;
pub mod validator_set;

pub use bridge_config::*;
pub use locked_tokens::*;
pub use validator_set::*;

/// Bridge state management utilities and common traits
pub trait BridgeStateValidation {
    fn validate_state(&self) -> Result<()>;
    fn is_operational(&self) -> bool;
}

/// Common state serialization format for cross-chain compatibility
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub struct CrossChainMessage {
    /// Unique message identifier
    pub message_id: [u8; 32],
    /// Source chain identifier
    pub source_chain: u8,
    /// Target chain identifier  
    pub target_chain: u8,
    /// Message type discriminator
    pub message_type: MessageType,
    /// Serialized payload data
    pub payload: Vec<u8>,
    /// Message timestamp
    pub timestamp: i64,
    /// Message expiry time
    pub expiry: i64,
    /// Nonce for replay protection
    pub nonce: u64,
}

/// Types of cross-chain messages supported by the bridge
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum MessageType {
    /// Token lock/unlock operations
    TokenTransfer,
    /// NFT transfer operations
    NftTransfer,
    /// Validator set updates
    ValidatorUpdate,
    /// Emergency pause/unpause
    EmergencyControl,
    /// Configuration updates
    ConfigUpdate,
    /// Reward distribution
    RewardDistribution,
}

impl CrossChainMessage {
    /// Create a new cross-chain message
    pub fn new(
        source_chain: u8,
        target_chain: u8,
        message_type: MessageType,
        payload: Vec<u8>,
        nonce: u64,
    ) -> Self {
        let clock = Clock::get().unwrap();
        let timestamp = clock.unix_timestamp;
        
        // Generate deterministic message ID from content hash
        let mut hasher = solana_program::hash::Hasher::default();
        hasher.hash(&[source_chain]);
        hasher.hash(&[target_chain]);
        hasher.hash(&payload);
        hasher.hash(&timestamp.to_le_bytes());
        hasher.hash(&nonce.to_le_bytes());
        let message_id = hasher.result().to_bytes();

        Self {
            message_id,
            source_chain,
            target_chain,
            message_type,
            payload,
            timestamp,
            expiry: timestamp + MESSAGE_EXPIRY_SECONDS,
            nonce,
        }
    }

    /// Verify message has not expired
    pub fn is_valid(&self) -> bool {
        let clock = Clock::get().unwrap();
        clock.unix_timestamp <= self.expiry
    }

    /// Generate message hash for signature verification
    pub fn hash(&self) -> [u8; 32] {
        let mut hasher = solana_program::hash::Hasher::default();
        hasher.hash(&self.message_id);
        hasher.hash(&[self.source_chain]);
        hasher.hash(&[self.target_chain]);
        hasher.hash(&self.payload);
        hasher.hash(&self.timestamp.to_le_bytes());
        hasher.hash(&self.nonce.to_le_bytes());
        hasher.result().to_bytes()
    }
}

/// Bridge operation status tracking
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum BridgeOperationStatus {
    /// Operation initiated but not confirmed
    Pending,
    /// Operation confirmed by required validators
    Confirmed,
    /// Operation executed successfully
    Executed,
    /// Operation failed or expired
    Failed,
    /// Operation cancelled by admin
    Cancelled,
}

/// Bridge operation record for tracking cross-chain transactions
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct BridgeOperation {
    /// Unique operation identifier
    pub operation_id: [u8; 32],
    /// Associated cross-chain message
    pub message: CrossChainMessage,
    /// Current operation status
    pub status: BridgeOperationStatus,
    /// Number of validator confirmations received
    pub confirmations: u8,
    /// Required confirmations threshold
    pub required_confirmations: u8,
    /// Validator signatures collected
    pub signatures: Vec<ValidatorSignature>,
    /// Operation creation timestamp
    pub created_at: i64,
    /// Last update timestamp
    pub updated_at: i64,
    /// Gas fee for operation execution
    pub gas_fee: u64,
    /// User who initiated the operation
    pub initiator: Pubkey,
}

impl BridgeOperation {
    /// Create a new bridge operation
    pub fn new(
        message: CrossChainMessage,
        initiator: Pubkey,
        required_confirmations: u8,
        gas_fee: u64,
    ) -> Self {
        let clock = Clock::get().unwrap();
        let timestamp = clock.unix_timestamp;
        
        // Generate operation ID from message ID and initiator
        let mut hasher = solana_program::hash::Hasher::default();
        hasher.hash(&message.message_id);
        hasher.hash(initiator.as_ref());
        hasher.hash(&timestamp.to_le_bytes());
        let operation_id = hasher.result().to_bytes();

        Self {
            operation_id,
            message,
            status: BridgeOperationStatus::Pending,
            confirmations: 0,
            required_confirmations,
            signatures: Vec::new(),
            created_at: timestamp,
            updated_at: timestamp,
            gas_fee,
            initiator,
        }
    }

    /// Add validator signature to operation
    pub fn add_signature(&mut self, signature: ValidatorSignature) -> Result<()> {
        // Check if validator already signed
        if self.signatures.iter().any(|s| s.validator == signature.validator) {
            return Err(FinovaBridgeError::DuplicateSignature.into());
        }

        // Verify signature is valid for this operation
        let message_hash = self.message.hash();
        if !signature.verify(&message_hash) {
            return Err(FinovaBridgeError::InvalidSignature.into());
        }

        self.signatures.push(signature);
        self.confirmations += 1;
        self.updated_at = Clock::get()?.unix_timestamp;

        // Update status if threshold reached
        if self.confirmations >= self.required_confirmations {
            self.status = BridgeOperationStatus::Confirmed;
        }

        Ok(())
    }

    /// Check if operation can be executed
    pub fn can_execute(&self) -> bool {
        matches!(self.status, BridgeOperationStatus::Confirmed) &&
        self.confirmations >= self.required_confirmations &&
        self.message.is_valid()
    }

    /// Mark operation as executed
    pub fn mark_executed(&mut self) -> Result<()> {
        if !self.can_execute() {
            return Err(FinovaBridgeError::OperationNotReady.into());
        }
        
        self.status = BridgeOperationStatus::Executed;
        self.updated_at = Clock::get()?.unix_timestamp;
        Ok(())
    }

    /// Mark operation as failed
    pub fn mark_failed(&mut self, reason: &str) -> Result<()> {
        self.status = BridgeOperationStatus::Failed;
        self.updated_at = Clock::get()?.unix_timestamp;
        msg!("Bridge operation failed: {}", reason);
        Ok(())
    }
}

/// Validator signature for bridge operations
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct ValidatorSignature {
    /// Validator public key
    pub validator: Pubkey,
    /// Signature bytes
    pub signature: [u8; 64],
    /// Signature timestamp
    pub timestamp: i64,
    /// Recovery ID for signature verification
    pub recovery_id: u8,
}

impl ValidatorSignature {
    /// Create a new validator signature
    pub fn new(validator: Pubkey, signature: [u8; 64], recovery_id: u8) -> Self {
        Self {
            validator,
            signature,
            timestamp: Clock::get().unwrap().unix_timestamp,
            recovery_id,
        }
    }

    /// Verify signature against message hash
    pub fn verify(&self, message_hash: &[u8; 32]) -> bool {
        // In a real implementation, this would use cryptographic signature verification
        // For now, we'll do a basic validation that signature is not empty
        !self.signature.iter().all(|&b| b == 0) && 
        message_hash.len() == 32
    }

    /// Check if signature is still valid (not expired)
    pub fn is_valid(&self) -> bool {
        let clock = Clock::get().unwrap();
        clock.unix_timestamp - self.timestamp <= SIGNATURE_VALIDITY_SECONDS
    }
}

/// Bridge statistics for monitoring and analytics
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, Default)]
pub struct BridgeStatistics {
    /// Total number of operations processed
    pub total_operations: u64,
    /// Number of successful operations
    pub successful_operations: u64,
    /// Number of failed operations
    pub failed_operations: u64,
    /// Total value locked in bridge
    pub total_value_locked: u64,
    /// Total fees collected
    pub total_fees_collected: u64,
    /// Average confirmation time in seconds
    pub average_confirmation_time: u64,
    /// Number of active validators
    pub active_validators: u8,
    /// Last update timestamp
    pub last_updated: i64,
}

impl BridgeStatistics {
    /// Update statistics with new operation
    pub fn update_with_operation(&mut self, operation: &BridgeOperation, value: u64) {
        self.total_operations += 1;
        
        match operation.status {
            BridgeOperationStatus::Executed => {
                self.successful_operations += 1;
                self.total_value_locked += value;
                self.total_fees_collected += operation.gas_fee;
                
                // Update average confirmation time
                let confirmation_time = operation.updated_at - operation.created_at;
                if self.successful_operations == 1 {
                    self.average_confirmation_time = confirmation_time as u64;
                } else {
                    // Running average calculation
                    let total_time = self.average_confirmation_time * (self.successful_operations - 1);
                    self.average_confirmation_time = (total_time + confirmation_time as u64) / self.successful_operations;
                }
            }
            BridgeOperationStatus::Failed => {
                self.failed_operations += 1;
            }
            _ => {}
        }
        
        self.last_updated = Clock::get().unwrap().unix_timestamp;
    }

    /// Calculate success rate as percentage
    pub fn success_rate(&self) -> u8 {
        if self.total_operations == 0 {
            return 100;
        }
        ((self.successful_operations * 100) / self.total_operations) as u8
    }

    /// Check if bridge is performing well
    pub fn is_healthy(&self) -> bool {
        self.success_rate() >= MINIMUM_SUCCESS_RATE_PERCENT &&
        self.active_validators >= MIN_ACTIVE_VALIDATORS
    }
}

/// Security context for bridge operations
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct SecurityContext {
    /// Security level required for operation
    pub security_level: SecurityLevel,
    /// Additional verification requirements
    pub verification_requirements: Vec<VerificationRequirement>,
    /// Rate limiting parameters
    pub rate_limits: RateLimits,
    /// Emergency controls status
    pub emergency_status: EmergencyStatus,
}

/// Security levels for different types of operations
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum SecurityLevel {
    /// Low security for small value transfers
    Low,
    /// Medium security for regular operations
    Medium,
    /// High security for large value transfers
    High,
    /// Critical security for admin operations
    Critical,
}

/// Additional verification requirements
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum VerificationRequirement {
    /// Require additional time delay
    TimeDelay(u64),
    /// Require multiple signatures
    MultiSignature(u8),
    /// Require whitelist verification
    WhitelistOnly,
    /// Require KYC verification
    KycRequired,
}

/// Rate limiting configuration
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct RateLimits {
    /// Maximum operations per time window
    pub max_operations_per_window: u32,
    /// Time window in seconds
    pub time_window_seconds: u64,
    /// Maximum value per time window
    pub max_value_per_window: u64,
    /// Current operation count in window
    pub current_operations: u32,
    /// Current value in window
    pub current_value: u64,
    /// Window start timestamp
    pub window_start: i64,
}

impl RateLimits {
    /// Check if operation is within rate limits
    pub fn check_limits(&mut self, operation_value: u64) -> Result<()> {
        let clock = Clock::get()?;
        let current_time = clock.unix_timestamp;

        // Reset window if expired
        if current_time - self.window_start >= self.time_window_seconds as i64 {
            self.current_operations = 0;
            self.current_value = 0;
            self.window_start = current_time;
        }

        // Check operation count limit
        if self.current_operations >= self.max_operations_per_window {
            return Err(FinovaBridgeError::RateLimitExceeded.into());
        }

        // Check value limit
        if self.current_value + operation_value > self.max_value_per_window {
            return Err(FinovaBridgeError::ValueLimitExceeded.into());
        }

        Ok(())
    }

    /// Record new operation
    pub fn record_operation(&mut self, operation_value: u64) {
        self.current_operations += 1;
        self.current_value += operation_value;
    }
}

/// Emergency status for bridge operations
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum EmergencyStatus {
    /// Normal operations
    Normal,
    /// Partial pause (only withdrawals allowed)
    PartialPause,
    /// Full pause (no operations allowed)
    FullPause,
    /// Emergency mode (admin only)
    Emergency,
}

/// Bridge health monitoring
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct BridgeHealth {
    /// Overall health status
    pub status: HealthStatus,
    /// Component health checks
    pub components: Vec<ComponentHealth>,
    /// Last health check timestamp
    pub last_check: i64,
    /// Health check interval in seconds
    pub check_interval: u64,
}

/// Health status levels
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum HealthStatus {
    /// All systems operational
    Healthy,
    /// Minor issues detected
    Degraded,
    /// Major issues detected
    Unhealthy,
    /// Critical failure
    Critical,
}

/// Individual component health
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct ComponentHealth {
    /// Component name
    pub component: String,
    /// Component status
    pub status: HealthStatus,
    /// Last check result
    pub last_result: String,
    /// Check timestamp
    pub timestamp: i64,
}

impl BridgeHealth {
    /// Perform health checks on all components
    pub fn check_health(&mut self, statistics: &BridgeStatistics, validator_set: &ValidatorSet) -> Result<()> {
        let clock = Clock::get()?;
        self.last_check = clock.unix_timestamp;
        self.components.clear();

        // Check validator health
        let validator_health = if validator_set.active_count >= MIN_ACTIVE_VALIDATORS {
            HealthStatus::Healthy
        } else {
            HealthStatus::Critical
        };
        
        self.components.push(ComponentHealth {
            component: "validators".to_string(),
            status: validator_health.clone(),
            last_result: format!("Active validators: {}", validator_set.active_count),
            timestamp: self.last_check,
        });

        // Check success rate
        let success_rate_health = if statistics.success_rate() >= MINIMUM_SUCCESS_RATE_PERCENT {
            HealthStatus::Healthy
        } else if statistics.success_rate() >= 80 {
            HealthStatus::Degraded
        } else {
            HealthStatus::Unhealthy
        };

        self.components.push(ComponentHealth {
            component: "success_rate".to_string(),
            status: success_rate_health.clone(),
            last_result: format!("Success rate: {}%", statistics.success_rate()),
            timestamp: self.last_check,
        });

        // Determine overall health
        self.status = if self.components.iter().all(|c| c.status == HealthStatus::Healthy) {
            HealthStatus::Healthy
        } else if self.components.iter().any(|c| c.status == HealthStatus::Critical) {
            HealthStatus::Critical
        } else if self.components.iter().any(|c| c.status == HealthStatus::Unhealthy) {
            HealthStatus::Unhealthy
        } else {
            HealthStatus::Degraded
        };

        Ok(())
    }
}

/// Utility functions for state management
pub mod utils {
    use super::*;

    /// Calculate required confirmations based on operation value
    pub fn calculate_required_confirmations(value: u64, total_validators: u8) -> u8 {
        let min_confirmations = if value >= HIGH_VALUE_THRESHOLD {
            // High value requires 75% of validators
            (total_validators * 3 / 4).max(MIN_CONFIRMATIONS_HIGH_VALUE)
        } else if value >= MEDIUM_VALUE_THRESHOLD {
            // Medium value requires 60% of validators
            (total_validators * 3 / 5).max(MIN_CONFIRMATIONS_MEDIUM_VALUE)
        } else {
            // Low value requires 50% of validators
            (total_validators / 2).max(MIN_CONFIRMATIONS_LOW_VALUE)
        };

        min_confirmations.min(MAX_CONFIRMATIONS)
    }

    /// Determine security level based on operation parameters
    pub fn determine_security_level(value: u64, message_type: &MessageType) -> SecurityLevel {
        match message_type {
            MessageType::ValidatorUpdate | MessageType::EmergencyControl | MessageType::ConfigUpdate => {
                SecurityLevel::Critical
            }
            MessageType::TokenTransfer | MessageType::NftTransfer => {
                if value >= HIGH_VALUE_THRESHOLD {
                    SecurityLevel::High
                } else if value >= MEDIUM_VALUE_THRESHOLD {
                    SecurityLevel::Medium
                } else {
                    SecurityLevel::Low
                }
            }
            MessageType::RewardDistribution => SecurityLevel::Medium,
        }
    }

    /// Create default rate limits based on security level
    pub fn create_default_rate_limits(security_level: &SecurityLevel) -> RateLimits {
        let clock = Clock::get().unwrap();
        
        match security_level {
            SecurityLevel::Low => RateLimits {
                max_operations_per_window: 100,
                time_window_seconds: 3600, // 1 hour
                max_value_per_window: 10_000 * LAMPORTS_PER_FIN,
                current_operations: 0,
                current_value: 0,
                window_start: clock.unix_timestamp,
            },
            SecurityLevel::Medium => RateLimits {
                max_operations_per_window: 50,
                time_window_seconds: 3600,
                max_value_per_window: 100_000 * LAMPORTS_PER_FIN,
                current_operations: 0,
                current_value: 0,
                window_start: clock.unix_timestamp,
            },
            SecurityLevel::High => RateLimits {
                max_operations_per_window: 20,
                time_window_seconds: 3600,
                max_value_per_window: 1_000_000 * LAMPORTS_PER_FIN,
                current_operations: 0,
                current_value: 0,
                window_start: clock.unix_timestamp,
            },
            SecurityLevel::Critical => RateLimits {
                max_operations_per_window: 5,
                time_window_seconds: 3600,
                max_value_per_window: 10_000_000 * LAMPORTS_PER_FIN,
                current_operations: 0,
                current_value: 0,
                window_start: clock.unix_timestamp,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cross_chain_message_creation() {
        let payload = vec![1, 2, 3, 4];
        let message = CrossChainMessage::new(1, 2, MessageType::TokenTransfer, payload.clone(), 1);
        
        assert_eq!(message.source_chain, 1);
        assert_eq!(message.target_chain, 2);
        assert_eq!(message.payload, payload);
        assert_eq!(message.nonce, 1);
        assert!(message.is_valid());
    }

    #[test]
    fn test_bridge_operation_lifecycle() {
        let message = CrossChainMessage::new(1, 2, MessageType::TokenTransfer, vec![1, 2, 3], 1);
        let initiator = Pubkey::new_unique();
        let mut operation = BridgeOperation::new(message, initiator, 2, 1000);
        
        assert_eq!(operation.status, BridgeOperationStatus::Pending);
        assert_eq!(operation.confirmations, 0);
        assert!(!operation.can_execute());
        
        // Add signatures
        let sig1 = ValidatorSignature::new(Pubkey::new_unique(), [1; 64], 0);
        let sig2 = ValidatorSignature::new(Pubkey::new_unique(), [2; 64], 0);
        
        operation.add_signature(sig1).unwrap();
        assert_eq!(operation.confirmations, 1);
        
        operation.add_signature(sig2).unwrap();
        assert_eq!(operation.confirmations, 2);
        assert_eq!(operation.status, BridgeOperationStatus::Confirmed);
        assert!(operation.can_execute());
    }

    #[test]
    fn test_rate_limits() {
        let mut rate_limits = RateLimits {
            max_operations_per_window: 2,
            time_window_seconds: 3600,
            max_value_per_window: 1000,
            current_operations: 0,
            current_value: 0,
            window_start: 0,
        };

        // First operation should pass
        assert!(rate_limits.check_limits(500).is_ok());
        rate_limits.record_operation(500);

        // Second operation should pass
        assert!(rate_limits.check_limits(400).is_ok());
        rate_limits.record_operation(400);

        // Third operation should fail (exceeds count limit)
        assert!(rate_limits.check_limits(50).is_err());

        // Reset and test value limit
        rate_limits.current_operations = 0;
        rate_limits.current_value = 0;
        
        // Should fail due to value limit
        assert!(rate_limits.check_limits(1001).is_err());
    }
}
