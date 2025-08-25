// programs/finova-oracle/src/constants.rs

//! Constants for Finova Oracle Program
//! 
//! This module defines all constants used throughout the Oracle program including:
//! - Oracle configuration parameters
//! - Price feed settings
//! - Aggregation parameters
//! - Time intervals and delays
//! - Mathematical constants
//! - Security thresholds

use anchor_lang::prelude::*;

/// Program ID for the Finova Oracle
#[constant]
pub const ORACLE_PROGRAM_ID: &str = "oracLe5HvkQ2x3xJVp9YhLw8YrEp6rYxCf2jKGvn8K";

/// Seeds for PDA derivation
pub mod seeds {
    /// Seed for Oracle Configuration PDA
    pub const ORACLE_CONFIG: &[u8] = b"oracle_config";
    
    /// Seed for Price Feed PDA
    pub const PRICE_FEED: &[u8] = b"price_feed";
    
    /// Seed for Aggregator PDA
    pub const AGGREGATOR: &[u8] = b"aggregator";
    
    /// Seed for Oracle Authority PDA
    pub const ORACLE_AUTHORITY: &[u8] = b"oracle_authority";
    
    /// Seed for Price History PDA
    pub const PRICE_HISTORY: &[u8] = b"price_history";
    
    /// Seed for Validator Set PDA
    pub const VALIDATOR_SET: &[u8] = b"validator_set";
}

/// Oracle Configuration Constants
pub mod oracle_config {
    /// Maximum number of oracle validators
    pub const MAX_VALIDATORS: u8 = 21;
    
    /// Minimum number of validators required for consensus
    pub const MIN_VALIDATORS: u8 = 7;
    
    /// Minimum number of validators required for price updates
    pub const MIN_VALIDATORS_FOR_PRICE: u8 = 3;
    
    /// Maximum staleness allowed for oracle data (in seconds)
    pub const MAX_STALENESS: i64 = 300; // 5 minutes
    
    /// Confidence threshold for price aggregation (basis points)
    pub const CONFIDENCE_THRESHOLD: u16 = 500; // 5%
    
    /// Maximum deviation allowed between validators (basis points)
    pub const MAX_DEVIATION: u16 = 1000; // 10%
    
    /// Cooldown period between price updates (in seconds)
    pub const UPDATE_COOLDOWN: i64 = 30;
    
    /// Maximum price change per update (basis points)
    pub const MAX_PRICE_CHANGE: u16 = 2500; // 25%
    
    /// Number of historical prices to maintain
    pub const PRICE_HISTORY_SIZE: usize = 100;
    
    /// Minimum stake required to become a validator (in lamports)
    pub const MIN_VALIDATOR_STAKE: u64 = 1_000_000_000; // 1 SOL
}

/// Price Feed Constants
pub mod price_feed {
    /// Maximum number of supported price feeds
    pub const MAX_PRICE_FEEDS: u16 = 1000;
    
    /// Default decimal places for price representation
    pub const DEFAULT_DECIMALS: u8 = 8;
    
    /// Maximum decimal places allowed
    pub const MAX_DECIMALS: u8 = 18;
    
    /// Minimum price value (to prevent zero/negative prices)
    pub const MIN_PRICE: u64 = 1;
    
    /// Maximum price value (to prevent overflow)
    pub const MAX_PRICE: u64 = u64::MAX / 1000;
    
    /// Default confidence interval (basis points)
    pub const DEFAULT_CONFIDENCE: u16 = 100; // 1%
    
    /// Maximum confidence interval allowed (basis points)
    pub const MAX_CONFIDENCE: u16 = 5000; // 50%
    
    /// Price feed identifier length
    pub const FEED_ID_LENGTH: usize = 32;
    
    /// Price feed description maximum length
    pub const FEED_DESCRIPTION_LENGTH: usize = 64;
}

/// Aggregation Constants
pub mod aggregation {
    /// Weight for exponential moving average
    pub const EMA_WEIGHT: u64 = 2000; // 20% in basis points * 100
    
    /// Maximum number of data points for aggregation
    pub const MAX_DATA_POINTS: usize = 50;
    
    /// Minimum number of data points required for aggregation
    pub const MIN_DATA_POINTS: usize = 3;
    
    /// Outlier detection threshold (standard deviations)
    pub const OUTLIER_THRESHOLD: u64 = 200; // 2.0 in fixed point
    
    /// Maximum age of data points for aggregation (in seconds)
    pub const MAX_DATA_AGE: i64 = 600; // 10 minutes
    
    /// Aggregation method types
    pub const AGGREGATION_MEDIAN: u8 = 0;
    pub const AGGREGATION_MEAN: u8 = 1;
    pub const AGGREGATION_WEIGHTED_MEAN: u8 = 2;
    pub const AGGREGATION_VWAP: u8 = 3;
    
    /// Default aggregation method
    pub const DEFAULT_AGGREGATION_METHOD: u8 = AGGREGATION_MEDIAN;
}

/// Time Constants
pub mod time {
    /// Seconds per minute
    pub const SECONDS_PER_MINUTE: i64 = 60;
    
    /// Seconds per hour
    pub const SECONDS_PER_HOUR: i64 = 3600;
    
    /// Seconds per day
    pub const SECONDS_PER_DAY: i64 = 86400;
    
    /// Seconds per week
    pub const SECONDS_PER_WEEK: i64 = 604800;
    
    /// Default heartbeat interval (in seconds)
    pub const DEFAULT_HEARTBEAT: i64 = 300; // 5 minutes
    
    /// Maximum heartbeat interval (in seconds)
    pub const MAX_HEARTBEAT: i64 = 3600; // 1 hour
    
    /// Minimum heartbeat interval (in seconds)
    pub const MIN_HEARTBEAT: i64 = 30; // 30 seconds
}

/// Mathematical Constants
pub mod math {
    /// Fixed point precision (decimal places)
    pub const PRECISION: u8 = 18;
    
    /// Fixed point multiplier (10^18)
    pub const PRECISION_MULTIPLIER: u128 = 1_000_000_000_000_000_000;
    
    /// Basis points multiplier (10^4)
    pub const BASIS_POINTS: u64 = 10_000;
    
    /// Percentage multiplier (10^2)
    pub const PERCENTAGE: u64 = 100;
    
    /// Maximum safe integer for calculations
    pub const MAX_SAFE_INT: u128 = u128::MAX / 1000;
    
    /// Minimum safe integer for calculations
    pub const MIN_SAFE_INT: u128 = 1;
    
    /// Pi constant in fixed point (18 decimals)
    pub const PI: u128 = 3_141_592_653_589_793_238;
    
    /// Euler's number in fixed point (18 decimals)
    pub const E: u128 = 2_718_281_828_459_045_235;
    
    /// Natural log of 2 in fixed point (18 decimals)
    pub const LN_2: u128 = 693_147_180_559_945_309;
}

/// Security Constants
pub mod security {
    /// Maximum number of failed validation attempts before lockout
    pub const MAX_FAILED_ATTEMPTS: u8 = 5;
    
    /// Lockout duration after failed attempts (in seconds)
    pub const LOCKOUT_DURATION: i64 = 3600; // 1 hour
    
    /// Minimum time between validator registrations (in seconds)
    pub const VALIDATOR_REGISTRATION_COOLDOWN: i64 = 86400; // 1 day
    
    /// Maximum number of simultaneous emergency updates
    pub const MAX_EMERGENCY_UPDATES: u8 = 3;
    
    /// Emergency update window (in seconds)
    pub const EMERGENCY_UPDATE_WINDOW: i64 = 300; // 5 minutes
    
    /// Slashing penalty for malicious behavior (basis points)
    pub const SLASHING_PENALTY: u16 = 1000; // 10%
    
    /// Minimum reputation score for validators
    pub const MIN_REPUTATION_SCORE: u64 = 7000; // 70%
    
    /// Reputation decay rate per day (basis points)
    pub const REPUTATION_DECAY_RATE: u16 = 10; // 0.1%
}

/// Network Constants
pub mod network {
    /// Default RPC timeout (in milliseconds)
    pub const DEFAULT_RPC_TIMEOUT: u64 = 30000; // 30 seconds
    
    /// Maximum RPC timeout (in milliseconds)
    pub const MAX_RPC_TIMEOUT: u64 = 300000; // 5 minutes
    
    /// Default number of RPC retries
    pub const DEFAULT_RPC_RETRIES: u8 = 3;
    
    /// Maximum number of RPC retries
    pub const MAX_RPC_RETRIES: u8 = 10;
    
    /// Default commitment level for transactions
    pub const DEFAULT_COMMITMENT: &str = "confirmed";
    
    /// Cluster types
    pub const CLUSTER_DEVNET: &str = "devnet";
    pub const CLUSTER_TESTNET: &str = "testnet";
    pub const CLUSTER_MAINNET: &str = "mainnet-beta";
}

/// Error Code Constants
pub mod error_codes {
    /// Invalid oracle configuration
    pub const INVALID_ORACLE_CONFIG: u32 = 6000;
    
    /// Insufficient validators
    pub const INSUFFICIENT_VALIDATORS: u32 = 6001;
    
    /// Stale price data
    pub const STALE_PRICE_DATA: u32 = 6002;
    
    /// Price deviation too high
    pub const PRICE_DEVIATION_TOO_HIGH: u32 = 6003;
    
    /// Invalid price feed
    pub const INVALID_PRICE_FEED: u32 = 6004;
    
    /// Unauthorized validator
    pub const UNAUTHORIZED_VALIDATOR: u32 = 6005;
    
    /// Validator lockout
    pub const VALIDATOR_LOCKOUT: u32 = 6006;
    
    /// Emergency mode active
    pub const EMERGENCY_MODE_ACTIVE: u32 = 6007;
    
    /// Invalid aggregation method
    pub const INVALID_AGGREGATION_METHOD: u32 = 6008;
    
    /// Insufficient stake
    pub const INSUFFICIENT_STAKE: u32 = 6009;
}

/// Event Type Constants
pub mod events {
    /// Price update event
    pub const PRICE_UPDATE: &str = "PriceUpdate";
    
    /// Validator registration event
    pub const VALIDATOR_REGISTRATION: &str = "ValidatorRegistration";
    
    /// Validator removal event
    pub const VALIDATOR_REMOVAL: &str = "ValidatorRemoval";
    
    /// Emergency update event
    pub const EMERGENCY_UPDATE: &str = "EmergencyUpdate";
    
    /// Configuration change event
    pub const CONFIG_CHANGE: &str = "ConfigChange";
    
    /// Aggregation event
    pub const AGGREGATION: &str = "Aggregation";
    
    /// Validation failure event
    pub const VALIDATION_FAILURE: &str = "ValidationFailure";
}

/// Account Size Constants
pub mod account_sizes {
    /// Size of Oracle Configuration account
    pub const ORACLE_CONFIG_SIZE: usize = 8 + // discriminator
        32 + // authority
        1 + // max_validators
        1 + // min_validators
        8 + // max_staleness
        2 + // confidence_threshold
        2 + // max_deviation
        8 + // update_cooldown
        2 + // max_price_change
        1 + // emergency_mode
        32 + // emergency_authority
        8 + // created_at
        8 + // updated_at
        64; // padding
    
    /// Size of Price Feed account
    pub const PRICE_FEED_SIZE: usize = 8 + // discriminator
        32 + // feed_id
        64 + // description
        8 + // price
        8 + // confidence
        8 + // timestamp
        1 + // decimals
        1 + // status
        32 + // authority
        8 + // heartbeat
        8 + // min_price
        8 + // max_price
        8 + // created_at
        8 + // updated_at
        64; // padding
    
    /// Size of Aggregator account
    pub const AGGREGATOR_SIZE: usize = 8 + // discriminator
        32 + // feed_id
        8 + // aggregated_price
        8 + // confidence
        8 + // timestamp
        1 + // aggregation_method
        1 + // data_points_count
        4 + (50 * 24) + // data_points (max 50 points, 24 bytes each)
        8 + // last_update
        32 + // authority
        64; // padding
    
    /// Size of Validator account
    pub const VALIDATOR_SIZE: usize = 8 + // discriminator
        32 + // validator_pubkey
        32 + // stake_authority
        8 + // stake_amount
        8 + // reputation_score
        1 + // status
        8 + // registered_at
        8 + // last_update
        1 + // failed_attempts
        8 + // lockout_until
        32 + // metadata
        64; // padding
}

/// Default Configuration Values
pub mod defaults {
    use super::*;
    
    /// Default oracle configuration
    pub const DEFAULT_MAX_VALIDATORS: u8 = oracle_config::MAX_VALIDATORS;
    pub const DEFAULT_MIN_VALIDATORS: u8 = oracle_config::MIN_VALIDATORS;
    pub const DEFAULT_MAX_STALENESS: i64 = oracle_config::MAX_STALENESS;
    pub const DEFAULT_CONFIDENCE_THRESHOLD: u16 = oracle_config::CONFIDENCE_THRESHOLD;
    pub const DEFAULT_MAX_DEVIATION: u16 = oracle_config::MAX_DEVIATION;
    pub const DEFAULT_UPDATE_COOLDOWN: i64 = oracle_config::UPDATE_COOLDOWN;
    pub const DEFAULT_MAX_PRICE_CHANGE: u16 = oracle_config::MAX_PRICE_CHANGE;
    
    /// Default price feed configuration
    pub const DEFAULT_HEARTBEAT: i64 = time::DEFAULT_HEARTBEAT;
    pub const DEFAULT_DECIMALS: u8 = price_feed::DEFAULT_DECIMALS;
    pub const DEFAULT_MIN_PRICE: u64 = price_feed::MIN_PRICE;
    pub const DEFAULT_MAX_PRICE: u64 = price_feed::MAX_PRICE;
    
    /// Default aggregation configuration
    pub const DEFAULT_AGGREGATION_METHOD: u8 = aggregation::DEFAULT_AGGREGATION_METHOD;
    pub const DEFAULT_MIN_DATA_POINTS: usize = aggregation::MIN_DATA_POINTS;
    pub const DEFAULT_MAX_DATA_AGE: i64 = aggregation::MAX_DATA_AGE;
}

/// Version Constants
pub mod version {
    /// Current program version
    pub const PROGRAM_VERSION: &str = "1.0.0";
    
    /// Minimum supported client version
    pub const MIN_CLIENT_VERSION: &str = "1.0.0";
    
    /// API version
    pub const API_VERSION: u8 = 1;
    
    /// Schema version for account structures
    pub const SCHEMA_VERSION: u8 = 1;
}

/// Feature Flags
pub mod features {
    /// Enable advanced aggregation methods
    pub const ADVANCED_AGGREGATION: bool = true;
    
    /// Enable price history tracking
    pub const PRICE_HISTORY: bool = true;
    
    /// Enable validator reputation system
    pub const VALIDATOR_REPUTATION: bool = true;
    
    /// Enable emergency mode
    pub const EMERGENCY_MODE: bool = true;
    
    /// Enable cross-chain support
    pub const CROSS_CHAIN: bool = false;
    
    /// Enable MEV protection
    pub const MEV_PROTECTION: bool = true;
}

/// Utility functions for constants
impl oracle_config {
    /// Check if the number of validators is valid
    pub fn is_valid_validator_count(count: u8) -> bool {
        count >= Self::MIN_VALIDATORS && count <= Self::MAX_VALIDATORS
    }
    
    /// Check if staleness is within acceptable range
    pub fn is_valid_staleness(staleness: i64) -> bool {
        staleness > 0 && staleness <= Self::MAX_STALENESS
    }
    
    /// Check if confidence threshold is valid
    pub fn is_valid_confidence(confidence: u16) -> bool {
        confidence <= math::BASIS_POINTS as u16
    }
    
    /// Check if deviation is within acceptable range
    pub fn is_valid_deviation(deviation: u16) -> bool {
        deviation <= Self::MAX_DEVIATION
    }
}

impl price_feed {
    /// Check if decimals are within valid range
    pub fn is_valid_decimals(decimals: u8) -> bool {
        decimals <= Self::MAX_DECIMALS
    }
    
    /// Check if price is within valid range
    pub fn is_valid_price(price: u64) -> bool {
        price >= Self::MIN_PRICE && price <= Self::MAX_PRICE
    }
    
    /// Check if confidence is within valid range
    pub fn is_valid_confidence(confidence: u16) -> bool {
        confidence <= Self::MAX_CONFIDENCE
    }
}

impl time {
    /// Check if heartbeat interval is valid
    pub fn is_valid_heartbeat(heartbeat: i64) -> bool {
        heartbeat >= Self::MIN_HEARTBEAT && heartbeat <= Self::MAX_HEARTBEAT
    }
    
    /// Convert seconds to minutes
    pub fn seconds_to_minutes(seconds: i64) -> i64 {
        seconds / Self::SECONDS_PER_MINUTE
    }
    
    /// Convert minutes to seconds
    pub fn minutes_to_seconds(minutes: i64) -> i64 {
        minutes * Self::SECONDS_PER_MINUTE
    }
    
    /// Check if timestamp is recent
    pub fn is_recent(timestamp: i64, max_age: i64) -> bool {
        let now = Clock::get().unwrap().unix_timestamp;
        now - timestamp <= max_age
    }
}

/// Test constants (only compiled in test builds)
#[cfg(test)]
pub mod test_constants {
    /// Test oracle authority
    pub const TEST_ORACLE_AUTHORITY: &str = "11111111111111111111111111111112";
    
    /// Test validator pubkey
    pub const TEST_VALIDATOR: &str = "11111111111111111111111111111113";
    
    /// Test price feed ID
    pub const TEST_FEED_ID: &str = "test_feed_btc_usd";
    
    /// Test price value
    pub const TEST_PRICE: u64 = 50000_00000000; // $50,000 with 8 decimals
    
    /// Test confidence value
    pub const TEST_CONFIDENCE: u16 = 100; // 1%
    
    /// Test stake amount
    pub const TEST_STAKE_AMOUNT: u64 = 1_000_000_000; // 1 SOL
}
