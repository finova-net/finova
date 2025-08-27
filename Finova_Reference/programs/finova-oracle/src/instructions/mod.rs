// programs/finova-oracle/src/instructions/mod.rs

use anchor_lang::prelude::*;

pub mod initialize_oracle;
pub mod update_price;
pub mod aggregate_feeds;
pub mod emergency_update;

pub use initialize_oracle::*;
pub use update_price::*;
pub use aggregate_feeds::*;
pub use emergency_update::*;

use crate::errors::FinovaOracleError;
use crate::state::{PriceFeed, Aggregator, OracleConfig};
use crate::constants::*;

/// Oracle instruction data structures and validation
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum OracleInstruction {
    /// Initialize a new oracle with configuration
    InitializeOracle {
        /// Minimum number of price feeds required
        min_feeds: u8,
        /// Maximum allowed price deviation (in basis points)
        max_deviation: u16,
        /// Update frequency in seconds
        update_frequency: u64,
        /// Oracle authority public key
        authority: Pubkey,
        /// Validator threshold for consensus
        validator_threshold: u8,
    },
    
    /// Update price feed with new data
    UpdatePrice {
        /// New price value (scaled by PRICE_DECIMALS)
        price: u64,
        /// Confidence interval (scaled by CONFIDENCE_DECIMALS)
        confidence: u64,
        /// Timestamp of the price update
        timestamp: i64,
        /// Source identifier for the price feed
        source_id: u16,
        /// Validator signature for authenticity
        validator_signature: [u8; 64],
    },
    
    /// Aggregate multiple price feeds
    AggregateFeed {
        /// List of feed accounts to aggregate
        feed_accounts: Vec<Pubkey>,
        /// Aggregation method (weighted average, median, etc.)
        aggregation_method: AggregationMethod,
        /// Minimum number of valid feeds required
        min_valid_feeds: u8,
        /// Maximum age of feeds to include (in seconds)
        max_age: u64,
    },
    
    /// Emergency price update by authorized admin
    EmergencyUpdate {
        /// Emergency price value
        emergency_price: u64,
        /// Emergency confidence level
        emergency_confidence: u64,
        /// Reason code for emergency update
        reason_code: EmergencyReason,
        /// Duration of emergency mode (in seconds)
        emergency_duration: u64,
        /// Admin signature for verification
        admin_signature: [u8; 64],
    },
}

/// Aggregation methods for combining multiple price feeds
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum AggregationMethod {
    /// Simple arithmetic mean
    Mean,
    /// Weighted average based on confidence levels
    WeightedAverage,
    /// Median value (robust against outliers)
    Median,
    /// Volume-weighted average price
    VWAP,
    /// Time-weighted average price
    TWAP,
    /// Exponentially weighted moving average
    EWMA { alpha: u64 }, // scaled by ALPHA_DECIMALS
}

/// Emergency update reasons
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum EmergencyReason {
    /// Market manipulation detected
    MarketManipulation,
    /// Oracle feed failure
    FeedFailure,
    /// Network congestion
    NetworkCongestion,
    /// Security breach
    SecurityBreach,
    /// Extreme volatility
    ExtremeVolatility,
    /// Regulatory intervention
    RegulatoryIntervention,
    /// Technical maintenance
    TechnicalMaintenance,
    /// Other emergency (custom reason)
    Other { reason: String },
}

/// Price feed validation result
#[derive(Debug, Clone, PartialEq)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub deviation_bp: u16,
    pub age_seconds: u64,
    pub confidence_score: u64,
    pub error_reason: Option<String>,
}

/// Oracle instruction validation context
pub struct InstructionContext<'info> {
    pub oracle_config: &'info Account<'info, OracleConfig>,
    pub aggregator: &'info Account<'info, Aggregator>,
    pub clock: &'info Sysvar<'info, Clock>,
    pub authority: &'info Signer<'info>,
}

impl<'info> InstructionContext<'info> {
    /// Create new instruction context
    pub fn new(
        oracle_config: &'info Account<'info, OracleConfig>,
        aggregator: &'info Account<'info, Aggregator>,
        clock: &'info Sysvar<'info, Clock>,
        authority: &'info Signer<'info>,
    ) -> Self {
        Self {
            oracle_config,
            aggregator,
            clock,
            authority,
        }
    }

    /// Validate if authority has permission for operation
    pub fn validate_authority(&self, required_role: AuthorityRole) -> Result<()> {
        match required_role {
            AuthorityRole::Admin => {
                if self.oracle_config.admin != *self.authority.key {
                    return Err(FinovaOracleError::UnauthorizedAccess.into());
                }
            }
            AuthorityRole::Validator => {
                if !self.oracle_config.validators.contains(self.authority.key) {
                    return Err(FinovaOracleError::UnauthorizedValidator.into());
                }
            }
            AuthorityRole::Updater => {
                if !self.oracle_config.price_updaters.contains(self.authority.key) {
                    return Err(FinovaOracleError::UnauthorizedUpdater.into());
                }
            }
        }
        Ok(())
    }

    /// Get current timestamp
    pub fn current_timestamp(&self) -> i64 {
        self.clock.unix_timestamp
    }

    /// Check if oracle is in emergency mode
    pub fn is_emergency_mode(&self) -> bool {
        let current_time = self.current_timestamp();
        self.oracle_config.emergency_mode_until > current_time
    }

    /// Validate update frequency
    pub fn validate_update_frequency(&self, last_update: i64) -> Result<()> {
        let current_time = self.current_timestamp();
        let time_diff = current_time - last_update;
        
        if time_diff < self.oracle_config.min_update_interval {
            return Err(FinovaOracleError::UpdateTooFrequent.into());
        }
        
        if time_diff > self.oracle_config.max_update_interval {
            msg!("Warning: Update interval exceeded maximum allowed time");
        }
        
        Ok(())
    }
}

/// Authority roles for oracle operations
#[derive(Debug, Clone, PartialEq)]
pub enum AuthorityRole {
    /// Full administrative access
    Admin,
    /// Price validation and consensus
    Validator,
    /// Price feed updates
    Updater,
}

/// Validate price feed data
pub fn validate_price_feed(
    price_feed: &PriceFeed,
    oracle_config: &OracleConfig,
    current_timestamp: i64,
) -> ValidationResult {
    let mut result = ValidationResult {
        is_valid: true,
        deviation_bp: 0,
        age_seconds: 0,
        confidence_score: price_feed.confidence,
        error_reason: None,
    };

    // Check feed age
    result.age_seconds = (current_timestamp - price_feed.last_update) as u64;
    if result.age_seconds > oracle_config.max_feed_age {
        result.is_valid = false;
        result.error_reason = Some("Feed too old".to_string());
        return result;
    }

    // Check minimum confidence level
    if price_feed.confidence < oracle_config.min_confidence {
        result.is_valid = false;
        result.error_reason = Some("Confidence too low".to_string());
        return result;
    }

    // Check price bounds
    if price_feed.price == 0 {
        result.is_valid = false;
        result.error_reason = Some("Invalid price (zero)".to_string());
        return result;
    }

    if price_feed.price > oracle_config.max_price_value {
        result.is_valid = false;
        result.error_reason = Some("Price exceeds maximum".to_string());
        return result;
    }

    // Check for circuit breaker conditions
    if price_feed.consecutive_failures > oracle_config.max_consecutive_failures {
        result.is_valid = false;
        result.error_reason = Some("Too many consecutive failures".to_string());
        return result;
    }

    result
}

/// Calculate price deviation between two values
pub fn calculate_price_deviation(price1: u64, price2: u64) -> u16 {
    if price1 == 0 || price2 == 0 {
        return u16::MAX; // Maximum deviation for invalid prices
    }

    let diff = if price1 > price2 {
        price1 - price2
    } else {
        price2 - price1
    };

    let base_price = if price1 > price2 { price1 } else { price2 };
    
    // Calculate deviation in basis points (1 basis point = 0.01%)
    let deviation_bp = (diff * BASIS_POINTS_SCALE) / base_price;
    
    // Cap at maximum u16 value
    if deviation_bp > u16::MAX as u64 {
        u16::MAX
    } else {
        deviation_bp as u16
    }
}

/// Validate aggregation parameters
pub fn validate_aggregation_params(
    method: &AggregationMethod,
    feed_accounts: &[Pubkey],
    min_valid_feeds: u8,
    max_age: u64,
) -> Result<()> {
    // Check minimum number of feeds
    if feed_accounts.len() < min_valid_feeds as usize {
        return Err(FinovaOracleError::InsufficientFeeds.into());
    }

    // Check maximum number of feeds
    if feed_accounts.len() > MAX_AGGREGATION_FEEDS {
        return Err(FinovaOracleError::TooManyFeeds.into());
    }

    // Check maximum age bounds
    if max_age == 0 || max_age > MAX_FEED_AGE {
        return Err(FinovaOracleError::InvalidFeedAge.into());
    }

    // Validate method-specific parameters
    match method {
        AggregationMethod::EWMA { alpha } => {
            if *alpha == 0 || *alpha > ALPHA_DECIMALS {
                return Err(FinovaOracleError::InvalidAggregationParams.into());
            }
        }
        _ => {} // Other methods don't have additional parameters to validate
    }

    Ok(())
}

/// Check if price update signature is valid
pub fn validate_price_signature(
    price: u64,
    timestamp: i64,
    validator_pubkey: &Pubkey,
    signature: &[u8; 64],
) -> Result<bool> {
    // Create message hash from price data
    let mut message = Vec::new();
    message.extend_from_slice(&price.to_le_bytes());
    message.extend_from_slice(&timestamp.to_le_bytes());
    message.extend_from_slice(validator_pubkey.as_ref());

    // In a real implementation, you would verify the signature using ed25519
    // For this example, we'll use a simplified validation
    let message_hash = solana_program::hash::hash(&message);
    
    // Verify signature against message hash (simplified)
    // In production, use proper cryptographic signature verification
    let is_valid = signature.iter().sum::<u8>() != 0; // Placeholder validation
    
    Ok(is_valid)
}

/// Emergency mode validation
pub fn validate_emergency_update(
    reason: &EmergencyReason,
    emergency_price: u64,
    emergency_confidence: u64,
    duration: u64,
    oracle_config: &OracleConfig,
) -> Result<()> {
    // Validate emergency duration
    if duration == 0 || duration > MAX_EMERGENCY_DURATION {
        return Err(FinovaOracleError::InvalidEmergencyDuration.into());
    }

    // Validate emergency price bounds
    if emergency_price == 0 || emergency_price > oracle_config.max_price_value {
        return Err(FinovaOracleError::InvalidEmergencyPrice.into());
    }

    // Validate emergency confidence
    if emergency_confidence > MAX_CONFIDENCE {
        return Err(FinovaOracleError::InvalidEmergencyConfidence.into());
    }

    // Validate reason-specific conditions
    match reason {
        EmergencyReason::Other { reason: custom_reason } => {
            if custom_reason.is_empty() || custom_reason.len() > MAX_REASON_LENGTH {
                return Err(FinovaOracleError::InvalidEmergencyReason.into());
            }
        }
        _ => {} // Standard reasons are always valid
    }

    Ok(())
}

/// Calculate consensus weight for validator
pub fn calculate_validator_weight(
    validator_pubkey: &Pubkey,
    oracle_config: &OracleConfig,
) -> u64 {
    // Find validator in the list and return their weight
    for (i, validator) in oracle_config.validators.iter().enumerate() {
        if validator == validator_pubkey {
            // Weight could be based on position, stake, reputation, etc.
            // For now, use equal weights with small variation based on position
            return VALIDATOR_BASE_WEIGHT + (i as u64 * VALIDATOR_WEIGHT_INCREMENT);
        }
    }
    0 // Validator not found
}

/// Check if enough validators have signed for consensus
pub fn check_validator_consensus(
    signatures: &[(Pubkey, [u8; 64])],
    oracle_config: &OracleConfig,
) -> Result<bool> {
    let mut total_weight = 0u64;
    let mut valid_signatures = 0u32;

    for (validator_pubkey, signature) in signatures {
        if oracle_config.validators.contains(validator_pubkey) {
            // In production, verify each signature
            let is_signature_valid = signature.iter().sum::<u8>() != 0; // Placeholder
            
            if is_signature_valid {
                total_weight += calculate_validator_weight(validator_pubkey, oracle_config);
                valid_signatures += 1;
            }
        }
    }

    // Check if we meet the minimum threshold
    let required_weight = (oracle_config.total_validator_weight * oracle_config.consensus_threshold as u64) / 100;
    let meets_weight_threshold = total_weight >= required_weight;
    let meets_count_threshold = valid_signatures >= oracle_config.min_validator_signatures;

    Ok(meets_weight_threshold && meets_count_threshold)
}

/// Circuit breaker logic for price feeds
pub fn check_circuit_breaker(
    current_price: u64,
    previous_price: u64,
    oracle_config: &OracleConfig,
) -> Result<bool> {
    if previous_price == 0 {
        return Ok(false); // No previous price to compare
    }

    let deviation_bp = calculate_price_deviation(current_price, previous_price);
    
    // Check if deviation exceeds circuit breaker threshold
    if deviation_bp > oracle_config.circuit_breaker_threshold {
        msg!(
            "Circuit breaker triggered: deviation {}bp exceeds threshold {}bp",
            deviation_bp,
            oracle_config.circuit_breaker_threshold
        );
        return Ok(true);
    }

    Ok(false)
}

/// Rate limiting for price updates
pub fn check_rate_limit(
    feed_account: &Pubkey,
    last_update_times: &std::collections::HashMap<Pubkey, i64>,
    current_timestamp: i64,
    min_interval: i64,
) -> Result<()> {
    if let Some(&last_update) = last_update_times.get(feed_account) {
        let time_diff = current_timestamp - last_update;
        if time_diff < min_interval {
            return Err(FinovaOracleError::RateLimitExceeded.into());
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_price_deviation() {
        assert_eq!(calculate_price_deviation(100, 101), 100); // 1% = 100bp
        assert_eq!(calculate_price_deviation(100, 110), 1000); // 10% = 1000bp
        assert_eq!(calculate_price_deviation(0, 100), u16::MAX);
        assert_eq!(calculate_price_deviation(100, 100), 0);
    }

    #[test]
    fn test_validate_aggregation_params() {
        let feeds = vec![Pubkey::default(); 5];
        let method = AggregationMethod::Mean;
        
        assert!(validate_aggregation_params(&method, &feeds, 3, 300).is_ok());
        assert!(validate_aggregation_params(&method, &feeds, 10, 300).is_err()); // Too few feeds
        assert!(validate_aggregation_params(&method, &feeds, 3, 0).is_err()); // Invalid age
    }

    #[test]
    fn test_validate_emergency_update() {
        let reason = EmergencyReason::MarketManipulation;
        let oracle_config = OracleConfig::default();
        
        assert!(validate_emergency_update(&reason, 1000, 5000, 3600, &oracle_config).is_ok());
        assert!(validate_emergency_update(&reason, 0, 5000, 3600, &oracle_config).is_err()); // Invalid price
        assert!(validate_emergency_update(&reason, 1000, 15000, 3600, &oracle_config).is_err()); // Invalid confidence
    }

    #[test]
    fn test_calculate_validator_weight() {
        let validator = Pubkey::default();
        let mut oracle_config = OracleConfig::default();
        oracle_config.validators.push(validator);
        
        let weight = calculate_validator_weight(&validator, &oracle_config);
        assert_eq!(weight, VALIDATOR_BASE_WEIGHT);
    }
}
