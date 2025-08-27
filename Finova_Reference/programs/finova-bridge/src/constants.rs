// programs/finova-bridge/src/constants.rs

use anchor_lang::prelude::*;

/// Program constants for Finova Bridge
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BridgeConstants;

impl BridgeConstants {
    // === BRIDGE CONFIGURATION ===
    
    /// Maximum number of validators in the validator set
    pub const MAX_VALIDATORS: usize = 21;
    
    /// Minimum number of validators required for consensus
    pub const MIN_VALIDATORS: usize = 7;
    
    /// Required threshold for validator consensus (2/3 + 1)
    pub const CONSENSUS_THRESHOLD: u8 = 67; // 67% = 2/3
    
    /// Maximum validator stake weight (prevents centralization)
    pub const MAX_VALIDATOR_WEIGHT: u64 = 100_000_000; // 100M tokens
    
    /// Minimum validator stake required
    pub const MIN_VALIDATOR_STAKE: u64 = 1_000_000; // 1M tokens
    
    // === BRIDGE LIMITS ===
    
    /// Maximum tokens that can be locked in a single transaction
    pub const MAX_LOCK_AMOUNT: u64 = 1_000_000_000_000; // 1T tokens
    
    /// Minimum tokens that can be locked in a single transaction
    pub const MIN_LOCK_AMOUNT: u64 = 1_000; // 1K tokens
    
    /// Maximum daily bridge volume per user
    pub const MAX_DAILY_VOLUME_PER_USER: u64 = 10_000_000_000; // 10B tokens
    
    /// Maximum total daily bridge volume
    pub const MAX_DAILY_TOTAL_VOLUME: u64 = 100_000_000_000; // 100B tokens
    
    /// Maximum pending transactions per user
    pub const MAX_PENDING_TXS_PER_USER: u8 = 5;
    
    /// Maximum total pending transactions
    pub const MAX_TOTAL_PENDING_TXS: u16 = 1000;
    
    // === TIMELOCK PARAMETERS ===
    
    /// Standard timelock for regular transfers (in seconds)
    pub const STANDARD_TIMELOCK: i64 = 300; // 5 minutes
    
    /// Extended timelock for large transfers (in seconds)
    pub const LARGE_TRANSFER_TIMELOCK: i64 = 3600; // 1 hour
    
    /// Emergency timelock for suspicious activity (in seconds)
    pub const EMERGENCY_TIMELOCK: i64 = 86400; // 24 hours
    
    /// Large transfer threshold (triggers extended timelock)
    pub const LARGE_TRANSFER_THRESHOLD: u64 = 100_000_000; // 100M tokens
    
    /// Maximum timelock duration
    pub const MAX_TIMELOCK_DURATION: i64 = 604800; // 7 days
    
    // === SECURITY PARAMETERS ===
    
    /// Maximum proof age in seconds (prevents replay attacks)
    pub const MAX_PROOF_AGE: i64 = 600; // 10 minutes
    
    /// Merkle tree depth for transaction proofs
    pub const MERKLE_TREE_DEPTH: u8 = 32;
    
    /// Maximum signature verification attempts
    pub const MAX_SIGNATURE_ATTEMPTS: u8 = 3;
    
    /// Nonce increment value
    pub const NONCE_INCREMENT: u64 = 1;
    
    /// Maximum nonce value (prevents overflow)
    pub const MAX_NONCE: u64 = u64::MAX - 1000;
    
    // === FEE STRUCTURE ===
    
    /// Base bridge fee in basis points (0.1%)
    pub const BASE_BRIDGE_FEE_BPS: u16 = 10;
    
    /// Large transfer fee in basis points (0.05%)
    pub const LARGE_TRANSFER_FEE_BPS: u16 = 5;
    
    /// Emergency transfer fee in basis points (1%)
    pub const EMERGENCY_FEE_BPS: u16 = 100;
    
    /// Maximum fee in basis points (2%)
    pub const MAX_FEE_BPS: u16 = 200;
    
    /// Validator reward percentage of fees (50%)
    pub const VALIDATOR_REWARD_PERCENTAGE: u8 = 50;
    
    /// Protocol fee percentage (30%)
    pub const PROTOCOL_FEE_PERCENTAGE: u8 = 30;
    
    /// Treasury fee percentage (20%)
    pub const TREASURY_FEE_PERCENTAGE: u8 = 20;
    
    // === CHAIN CONFIGURATIONS ===
    
    /// Solana chain ID
    pub const SOLANA_CHAIN_ID: u16 = 101;
    
    /// Ethereum chain ID
    pub const ETHEREUM_CHAIN_ID: u16 = 1;
    
    /// BSC chain ID
    pub const BSC_CHAIN_ID: u16 = 56;
    
    /// Polygon chain ID
    pub const POLYGON_CHAIN_ID: u16 = 137;
    
    /// Avalanche chain ID
    pub const AVALANCHE_CHAIN_ID: u16 = 43114;
    
    /// Arbitrum chain ID
    pub const ARBITRUM_CHAIN_ID: u16 = 42161;
    
    /// Maximum supported chains
    pub const MAX_SUPPORTED_CHAINS: u8 = 10;
    
    // === BRIDGE STATES ===
    
    /// Bridge is active and operational
    pub const BRIDGE_STATE_ACTIVE: u8 = 1;
    
    /// Bridge is paused (maintenance mode)
    pub const BRIDGE_STATE_PAUSED: u8 = 2;
    
    /// Bridge is in emergency mode
    pub const BRIDGE_STATE_EMERGENCY: u8 = 3;
    
    /// Bridge is disabled
    pub const BRIDGE_STATE_DISABLED: u8 = 4;
    
    // === TRANSACTION STATES ===
    
    /// Transaction is pending validation
    pub const TX_STATE_PENDING: u8 = 1;
    
    /// Transaction is validated and ready for execution
    pub const TX_STATE_VALIDATED: u8 = 2;
    
    /// Transaction is being executed
    pub const TX_STATE_EXECUTING: u8 = 3;
    
    /// Transaction is completed successfully
    pub const TX_STATE_COMPLETED: u8 = 4;
    
    /// Transaction failed during execution
    pub const TX_STATE_FAILED: u8 = 5;
    
    /// Transaction was cancelled
    pub const TX_STATE_CANCELLED: u8 = 6;
    
    /// Transaction is expired
    pub const TX_STATE_EXPIRED: u8 = 7;
    
    // === VALIDATOR STATES ===
    
    /// Validator is active
    pub const VALIDATOR_STATE_ACTIVE: u8 = 1;
    
    /// Validator is inactive (temporarily)
    pub const VALIDATOR_STATE_INACTIVE: u8 = 2;
    
    /// Validator is slashed
    pub const VALIDATOR_STATE_SLASHED: u8 = 3;
    
    /// Validator is jailed
    pub const VALIDATOR_STATE_JAILED: u8 = 4;
    
    // === SLASHING PARAMETERS ===
    
    /// Slashing penalty for double signing (50%)
    pub const DOUBLE_SIGN_SLASH_PERCENTAGE: u8 = 50;
    
    /// Slashing penalty for downtime (1%)
    pub const DOWNTIME_SLASH_PERCENTAGE: u8 = 1;
    
    /// Slashing penalty for invalid signature (10%)
    pub const INVALID_SIGNATURE_SLASH_PERCENTAGE: u8 = 10;
    
    /// Maximum consecutive failures before slashing
    pub const MAX_CONSECUTIVE_FAILURES: u8 = 3;
    
    /// Jail duration in seconds (24 hours)
    pub const JAIL_DURATION: i64 = 86400;
    
    // === REWARDS PARAMETERS ===
    
    /// Block reward for validators (in tokens)
    pub const VALIDATOR_BLOCK_REWARD: u64 = 1000;
    
    /// Transaction fee reward multiplier
    pub const TX_FEE_REWARD_MULTIPLIER: u64 = 2;
    
    /// Minimum uptime required for rewards (95%)
    pub const MIN_UPTIME_FOR_REWARDS: u8 = 95;
    
    /// Reward distribution interval (in seconds) - 24 hours
    pub const REWARD_DISTRIBUTION_INTERVAL: i64 = 86400;
    
    // === EMERGENCY PARAMETERS ===
    
    /// Emergency pause duration (in seconds) - 1 hour
    pub const EMERGENCY_PAUSE_DURATION: i64 = 3600;
    
    /// Maximum emergency pause extensions
    pub const MAX_EMERGENCY_EXTENSIONS: u8 = 3;
    
    /// Emergency recovery threshold (requires supermajority)
    pub const EMERGENCY_RECOVERY_THRESHOLD: u8 = 75; // 75%
    
    /// Circuit breaker threshold (unusual activity detection)
    pub const CIRCUIT_BREAKER_THRESHOLD: u64 = 1_000_000_000; // 1B tokens per hour
    
    // === GOVERNANCE PARAMETERS ===
    
    /// Proposal voting period (in seconds) - 7 days
    pub const PROPOSAL_VOTING_PERIOD: i64 = 604800;
    
    /// Proposal execution delay (in seconds) - 2 days
    pub const PROPOSAL_EXECUTION_DELAY: i64 = 172800;
    
    /// Minimum proposal stake required
    pub const MIN_PROPOSAL_STAKE: u64 = 10_000_000; // 10M tokens
    
    /// Quorum threshold for proposals (20%)
    pub const PROPOSAL_QUORUM_THRESHOLD: u8 = 20;
    
    /// Approval threshold for proposals (60%)
    pub const PROPOSAL_APPROVAL_THRESHOLD: u8 = 60;
    
    // === RATE LIMITING ===
    
    /// Rate limit per user per minute
    pub const RATE_LIMIT_PER_USER_PER_MINUTE: u8 = 10;
    
    /// Rate limit per IP per minute
    pub const RATE_LIMIT_PER_IP_PER_MINUTE: u8 = 50;
    
    /// Global rate limit per minute
    pub const GLOBAL_RATE_LIMIT_PER_MINUTE: u16 = 1000;
    
    /// Rate limit window in seconds
    pub const RATE_LIMIT_WINDOW: i64 = 60;
    
    // === ACCOUNT SIZES ===
    
    /// Bridge config account size
    pub const BRIDGE_CONFIG_SIZE: usize = 8 + // discriminator
        32 + // authority
        2 + // chain_id
        1 + // state
        8 + // total_locked
        8 + // total_unlocked
        4 + // validator_count
        32 + // fee_recipient
        2 + // fee_bps
        8 + // min_lock_amount
        8 + // max_lock_amount
        8 + // timelock_duration
        8 + // last_update_timestamp
        64; // reserved space
    
    /// Locked tokens account size
    pub const LOCKED_TOKENS_SIZE: usize = 8 + // discriminator
        32 + // user
        32 + // token_mint
        8 + // amount
        2 + // target_chain_id
        32 + // target_address
        8 + // lock_timestamp
        8 + // unlock_timestamp
        1 + // state
        32 + // transaction_hash
        8 + // nonce
        64; // reserved space
    
    /// Validator account size
    pub const VALIDATOR_SIZE: usize = 8 + // discriminator
        32 + // validator_key
        32 + // stake_account
        8 + // stake_amount
        1 + // state
        8 + // last_active_timestamp
        4 + // consecutive_failures
        8 + // total_rewards
        8 + // slash_amount
        32 + // commission_rate
        64; // reserved space
    
    /// Validator set account size
    pub const VALIDATOR_SET_SIZE: usize = 8 + // discriminator
        4 + // validator_count
        (32 * Self::MAX_VALIDATORS) + // validator_keys
        (8 * Self::MAX_VALIDATORS) + // validator_weights
        8 + // total_stake
        8 + // last_update_epoch
        64; // reserved space
    
    // === MATHEMATICAL CONSTANTS ===
    
    /// Basis points divisor (10,000 = 100%)
    pub const BASIS_POINTS_DIVISOR: u64 = 10_000;
    
    /// Percentage divisor (100 = 100%)
    pub const PERCENTAGE_DIVISOR: u64 = 100;
    
    /// Precision multiplier for calculations
    pub const PRECISION_MULTIPLIER: u64 = 1_000_000_000; // 1B
    
    /// Maximum calculation precision
    pub const MAX_PRECISION: u8 = 18;
    
    // === VALIDATION CONSTANTS ===
    
    /// Maximum address length
    pub const MAX_ADDRESS_LENGTH: usize = 64;
    
    /// Maximum transaction hash length
    pub const MAX_TX_HASH_LENGTH: usize = 32;
    
    /// Maximum proof data length
    pub const MAX_PROOF_DATA_LENGTH: usize = 1024;
    
    /// Maximum metadata length
    pub const MAX_METADATA_LENGTH: usize = 256;
    
    // === NETWORK CONSTANTS ===
    
    /// Default RPC timeout in milliseconds
    pub const DEFAULT_RPC_TIMEOUT: u64 = 30000; // 30 seconds
    
    /// Maximum retry attempts for network calls
    pub const MAX_NETWORK_RETRIES: u8 = 3;
    
    /// Network confirmation requirements
    pub const REQUIRED_CONFIRMATIONS: u8 = 32;
    
    /// Block finality timeout in seconds
    pub const BLOCK_FINALITY_TIMEOUT: i64 = 300; // 5 minutes
    
    // === UPGRADE CONSTANTS ===
    
    /// Program upgrade authority buffer size
    pub const UPGRADE_AUTHORITY_SIZE: usize = 32;
    
    /// Maximum program upgrade size
    pub const MAX_UPGRADE_SIZE: usize = 1_048_576; // 1MB
    
    /// Upgrade proposal voting period
    pub const UPGRADE_VOTING_PERIOD: i64 = 432000; // 5 days
    
    /// Minimum upgrade stake required
    pub const MIN_UPGRADE_STAKE: u64 = 50_000_000; // 50M tokens
}

// === SEED CONSTANTS ===

/// Bridge config PDA seed
pub const BRIDGE_CONFIG_SEED: &[u8] = b"bridge_config";

/// Locked tokens PDA seed
pub const LOCKED_TOKENS_SEED: &[u8] = b"locked_tokens";

/// Validator PDA seed
pub const VALIDATOR_SEED: &[u8] = b"validator";

/// Validator set PDA seed
pub const VALIDATOR_SET_SEED: &[u8] = b"validator_set";

/// Bridge treasury PDA seed
pub const BRIDGE_TREASURY_SEED: &[u8] = b"bridge_treasury";

/// Proposal PDA seed
pub const PROPOSAL_SEED: &[u8] = b"proposal";

/// Vote PDA seed
pub const VOTE_SEED: &[u8] = b"vote";

// === ERROR CODES ===

/// Invalid chain ID
pub const ERROR_INVALID_CHAIN_ID: u32 = 6000;

/// Insufficient stake
pub const ERROR_INSUFFICIENT_STAKE: u32 = 6001;

/// Invalid validator
pub const ERROR_INVALID_VALIDATOR: u32 = 6002;

/// Bridge paused
pub const ERROR_BRIDGE_PAUSED: u32 = 6003;

/// Amount exceeds limits
pub const ERROR_AMOUNT_EXCEEDS_LIMITS: u32 = 6004;

/// Invalid proof
pub const ERROR_INVALID_PROOF: u32 = 6005;

/// Proof expired
pub const ERROR_PROOF_EXPIRED: u32 = 6006;

/// Transaction already processed
pub const ERROR_TX_ALREADY_PROCESSED: u32 = 6007;

/// Insufficient consensus
pub const ERROR_INSUFFICIENT_CONSENSUS: u32 = 6008;

/// Timelock not expired
pub const ERROR_TIMELOCK_NOT_EXPIRED: u32 = 6009;

/// Rate limit exceeded
pub const ERROR_RATE_LIMIT_EXCEEDED: u32 = 6010;

// === HELPER FUNCTIONS ===

impl BridgeConstants {
    /// Calculate fee amount based on transfer amount and fee rate
    pub fn calculate_fee(amount: u64, fee_bps: u16) -> u64 {
        (amount as u128 * fee_bps as u128 / Self::BASIS_POINTS_DIVISOR as u128) as u64
    }
    
    /// Check if amount is considered a large transfer
    pub fn is_large_transfer(amount: u64) -> bool {
        amount >= Self::LARGE_TRANSFER_THRESHOLD
    }
    
    /// Get timelock duration based on amount and risk level
    pub fn get_timelock_duration(amount: u64, is_suspicious: bool) -> i64 {
        if is_suspicious {
            Self::EMERGENCY_TIMELOCK
        } else if Self::is_large_transfer(amount) {
            Self::LARGE_TRANSFER_TIMELOCK
        } else {
            Self::STANDARD_TIMELOCK
        }
    }
    
    /// Calculate validator reward based on stake and performance
    pub fn calculate_validator_reward(
        stake: u64,
        total_stake: u64,
        block_reward: u64,
        uptime_percentage: u8,
    ) -> u64 {
        if uptime_percentage < Self::MIN_UPTIME_FOR_REWARDS {
            return 0;
        }
        
        let base_reward = (block_reward as u128 * stake as u128 / total_stake as u128) as u64;
        (base_reward as u128 * uptime_percentage as u128 / 100) as u128 as u64
    }
    
    /// Check if consensus threshold is met
    pub fn is_consensus_met(approvals: u8, total_validators: u8) -> bool {
        let required = (total_validators as u16 * Self::CONSENSUS_THRESHOLD as u16 + 99) / 100;
        approvals as u16 >= required
    }
    
    /// Calculate slashing amount based on penalty type
    pub fn calculate_slash_amount(stake: u64, penalty_type: u8) -> u64 {
        let percentage = match penalty_type {
            1 => Self::DOUBLE_SIGN_SLASH_PERCENTAGE,
            2 => Self::DOWNTIME_SLASH_PERCENTAGE,
            3 => Self::INVALID_SIGNATURE_SLASH_PERCENTAGE,
            _ => 0,
        };
        
        (stake as u128 * percentage as u128 / Self::PERCENTAGE_DIVISOR as u128) as u64
    }
}

// === TYPE ALIASES ===

/// Bridge transaction hash type
pub type BridgeTxHash = [u8; 32];

/// Chain ID type
pub type ChainID = u16;

/// Validator index type
pub type ValidatorIndex = u8;

/// Bridge state type
pub type BridgeState = u8;

/// Transaction state type
pub type TxState = u8;

/// Validator state type
pub type ValidatorState = u8;

// === CONFIGURATION STRUCTS ===

/// Chain configuration
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ChainConfig {
    pub chain_id: ChainID,
    pub min_confirmations: u8,
    pub block_time: u16, // in seconds
    pub gas_limit: u64,
    pub supported_tokens: u8,
}

/// Fee configuration
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FeeConfig {
    pub base_fee_bps: u16,
    pub large_transfer_fee_bps: u16,
    pub emergency_fee_bps: u16,
    pub validator_percentage: u8,
    pub protocol_percentage: u8,
    pub treasury_percentage: u8,
}

impl Default for FeeConfig {
    fn default() -> Self {
        Self {
            base_fee_bps: BridgeConstants::BASE_BRIDGE_FEE_BPS,
            large_transfer_fee_bps: BridgeConstants::LARGE_TRANSFER_FEE_BPS,
            emergency_fee_bps: BridgeConstants::EMERGENCY_FEE_BPS,
            validator_percentage: BridgeConstants::VALIDATOR_REWARD_PERCENTAGE,
            protocol_percentage: BridgeConstants::PROTOCOL_FEE_PERCENTAGE,
            treasury_percentage: BridgeConstants::TREASURY_FEE_PERCENTAGE,
        }
    }
}

/// Security configuration
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SecurityConfig {
    pub max_proof_age: i64,
    pub merkle_tree_depth: u8,
    pub max_signature_attempts: u8,
    pub circuit_breaker_threshold: u64,
    pub emergency_pause_duration: i64,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            max_proof_age: BridgeConstants::MAX_PROOF_AGE,
            merkle_tree_depth: BridgeConstants::MERKLE_TREE_DEPTH,
            max_signature_attempts: BridgeConstants::MAX_SIGNATURE_ATTEMPTS,
            circuit_breaker_threshold: BridgeConstants::CIRCUIT_BREAKER_THRESHOLD,
            emergency_pause_duration: BridgeConstants::EMERGENCY_PAUSE_DURATION,
        }
    }
}

// === TEST CONSTANTS (for development and testing only) ===
#[cfg(test)]
pub mod test_constants {
    use super::*;
    
    /// Test validator private keys (NOT FOR PRODUCTION)
    pub const TEST_VALIDATOR_KEYS: &[&str] = &[
        "test_validator_1_private_key",
        "test_validator_2_private_key",
        "test_validator_3_private_key",
    ];
    
    /// Test token mint addresses
    pub const TEST_TOKEN_MINTS: &[&str] = &[
        "So11111111111111111111111111111111111111112", // Wrapped SOL
        "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v", // USDC
    ];
    
    /// Reduced timeouts for testing
    pub const TEST_TIMELOCK_DURATION: i64 = 10; // 10 seconds
    pub const TEST_PROOF_AGE: i64 = 30; // 30 seconds
    pub const TEST_RATE_LIMIT: u8 = 100; // Higher limit for testing
}
