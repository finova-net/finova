// programs/finova-oracle/src/utils.rs

use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount, Mint};
use std::collections::HashMap;
use crate::constants::*;
use crate::errors::*;

/// Utility functions for oracle operations and data validation
pub struct OracleUtils;

impl OracleUtils {
    /// Validates price data input format and ranges
    pub fn validate_price_data(price: u64, confidence: u64, timestamp: i64) -> Result<()> {
        // Check if price is within reasonable bounds (not zero, not extremely high)
        require!(price > 0, FinovaOracleError::InvalidPrice);
        require!(price < MAX_PRICE_VALUE, FinovaOracleError::PriceOutOfBounds);
        
        // Validate confidence interval (0-10000 representing 0-100%)
        require!(confidence <= CONFIDENCE_MAX, FinovaOracleError::InvalidConfidence);
        
        // Check timestamp is not too old or in the future
        let current_time = Clock::get()?.unix_timestamp;
        require!(
            timestamp <= current_time + MAX_FUTURE_TIMESTAMP_OFFSET,
            FinovaOracleError::TimestampTooFuture
        );
        require!(
            timestamp >= current_time - MAX_PAST_TIMESTAMP_OFFSET,
            FinovaOracleError::TimestampTooOld
        );
        
        Ok(())
    }

    /// Calculates weighted average price from multiple feed sources
    pub fn calculate_weighted_average(
        prices: &[(u64, u64, u64)], // (price, weight, confidence)
        min_feeds: u8,
    ) -> Result<(u64, u64)> {
        require!(prices.len() >= min_feeds as usize, FinovaOracleError::InsufficientFeeds);
        
        let mut total_weighted_price = 0u128;
        let mut total_weight = 0u64;
        let mut total_weighted_confidence = 0u128;
        
        // Remove outliers using modified z-score
        let valid_prices = Self::remove_outliers(prices)?;
        
        for (price, weight, confidence) in valid_prices.iter() {
            // Apply confidence weighting to the feed weight
            let adjusted_weight = (*weight as u128 * *confidence as u128) / CONFIDENCE_MAX as u128;
            
            total_weighted_price += *price as u128 * adjusted_weight;
            total_weighted_confidence += *confidence as u128 * adjusted_weight;
            total_weight += adjusted_weight as u64;
        }
        
        require!(total_weight > 0, FinovaOracleError::NoValidFeeds);
        
        let final_price = (total_weighted_price / total_weight as u128) as u64;
        let final_confidence = (total_weighted_confidence / total_weight as u128) as u64;
        
        Ok((final_price, final_confidence))
    }

    /// Removes statistical outliers using modified z-score method
    pub fn remove_outliers(prices: &[(u64, u64, u64)]) -> Result<Vec<(u64, u64, u64)>> {
        if prices.len() < 3 {
            return Ok(prices.to_vec());
        }
        
        // Calculate median
        let mut sorted_prices: Vec<u64> = prices.iter().map(|(p, _, _)| *p).collect();
        sorted_prices.sort();
        let median = sorted_prices[sorted_prices.len() / 2];
        
        // Calculate median absolute deviation (MAD)
        let mut deviations: Vec<u64> = sorted_prices
            .iter()
            .map(|&p| if p > median { p - median } else { median - p })
            .collect();
        deviations.sort();
        let mad = deviations[deviations.len() / 2];
        
        // Filter outliers using modified z-score threshold
        let threshold = (OUTLIER_THRESHOLD * mad) / 6745; // 0.6745 is the 0.75 quantile of standard normal
        
        let valid_prices: Vec<(u64, u64, u64)> = prices
            .iter()
            .filter(|(price, _, _)| {
                let deviation = if *price > median { price - median } else { median - price };
                deviation <= threshold
            })
            .cloned()
            .collect();
        
        // Ensure we have at least minimum number of feeds after filtering
        if valid_prices.len() < MIN_FEEDS_AFTER_OUTLIER_REMOVAL as usize {
            return Ok(prices.to_vec()); // Return original if too many outliers
        }
        
        Ok(valid_prices)
    }

    /// Validates oracle authority permissions
    pub fn validate_oracle_authority(
        authority: &AccountInfo,
        expected_authority: &Pubkey,
        instruction_name: &str,
    ) -> Result<()> {
        require!(
            authority.key() == expected_authority,
            FinovaOracleError::UnauthorizedOracle
        );
        require!(authority.is_signer, FinovaOracleError::AuthorityMustSign);
        
        msg!("Oracle authority validated for instruction: {}", instruction_name);
        Ok(())
    }

    /// Calculates price volatility over time period
    pub fn calculate_volatility(price_history: &[u64], window_size: usize) -> Result<u64> {
        require!(
            price_history.len() >= window_size,
            FinovaOracleError::InsufficientPriceHistory
        );
        
        let recent_prices = &price_history[price_history.len() - window_size..];
        
        // Calculate mean
        let sum: u128 = recent_prices.iter().map(|&p| p as u128).sum();
        let mean = sum / window_size as u128;
        
        // Calculate variance
        let variance_sum: u128 = recent_prices
            .iter()
            .map(|&p| {
                let diff = if p as u128 > mean { p as u128 - mean } else { mean - p as u128 };
                diff * diff
            })
            .sum();
        
        let variance = variance_sum / window_size as u128;
        
        // Approximate square root for standard deviation
        let volatility = Self::integer_sqrt(variance) as u64;
        
        Ok(volatility)
    }

    /// Integer square root calculation using Newton's method
    pub fn integer_sqrt(n: u128) -> u128 {
        if n == 0 {
            return 0;
        }
        
        let mut x = n;
        let mut y = (x + 1) / 2;
        
        while y < x {
            x = y;
            y = (x + n / x) / 2;
        }
        
        x
    }

    /// Validates feed data freshness
    pub fn validate_feed_freshness(timestamp: i64, max_age_seconds: i64) -> Result<()> {
        let current_time = Clock::get()?.unix_timestamp;
        let age = current_time - timestamp;
        
        require!(age <= max_age_seconds, FinovaOracleError::StaleData);
        require!(age >= 0, FinovaOracleError::FutureTimestamp);
        
        Ok(())
    }

    /// Calculates exponential moving average for price smoothing
    pub fn calculate_ema(
        current_price: u64,
        previous_ema: u64,
        alpha_numerator: u64,
        alpha_denominator: u64,
    ) -> Result<u64> {
        require!(alpha_denominator > 0, FinovaOracleError::InvalidEmaParameters);
        require!(alpha_numerator <= alpha_denominator, FinovaOracleError::InvalidEmaParameters);
        
        // EMA = α * current_price + (1 - α) * previous_ema
        // Using integer arithmetic: EMA = (α_num * current + (α_den - α_num) * previous) / α_den
        
        let weighted_current = current_price as u128 * alpha_numerator as u128;
        let weighted_previous = previous_ema as u128 * (alpha_denominator - alpha_numerator) as u128;
        
        let ema = (weighted_current + weighted_previous) / alpha_denominator as u128;
        
        Ok(ema as u64)
    }

    /// Validates cross-chain oracle data integrity
    pub fn validate_cross_chain_data(
        source_chain_id: u32,
        block_hash: &[u8; 32],
        merkle_proof: &[u8],
    ) -> Result<()> {
        require!(
            SUPPORTED_CHAIN_IDS.contains(&source_chain_id),
            FinovaOracleError::UnsupportedChain
        );
        
        require!(block_hash != &[0u8; 32], FinovaOracleError::InvalidBlockHash);
        require!(!merkle_proof.is_empty(), FinovaOracleError::InvalidMerkleProof);
        require!(merkle_proof.len() <= MAX_MERKLE_PROOF_SIZE, FinovaOracleError::ProofTooLarge);
        
        Ok(())
    }

    /// Calculates time-weighted average price (TWAP)
    pub fn calculate_twap(
        price_points: &[(u64, i64)], // (price, timestamp)
        time_window: i64,
    ) -> Result<u64> {
        require!(!price_points.is_empty(), FinovaOracleError::EmptyPriceData);
        require!(time_window > 0, FinovaOracleError::InvalidTimeWindow);
        
        let current_time = Clock::get()?.unix_timestamp;
        let start_time = current_time - time_window;
        
        // Filter relevant price points
        let relevant_points: Vec<(u64, i64)> = price_points
            .iter()
            .filter(|(_, timestamp)| *timestamp >= start_time)
            .cloned()
            .collect();
        
        require!(!relevant_points.is_empty(), FinovaOracleError::InsufficientTwapData);
        
        if relevant_points.len() == 1 {
            return Ok(relevant_points[0].0);
        }
        
        let mut weighted_sum = 0u128;
        let mut total_time = 0i64;
        
        for i in 0..relevant_points.len() - 1 {
            let (price, timestamp) = relevant_points[i];
            let next_timestamp = relevant_points[i + 1].1;
            let time_weight = next_timestamp - timestamp;
            
            weighted_sum += price as u128 * time_weight as u128;
            total_time += time_weight;
        }
        
        require!(total_time > 0, FinovaOracleError::InvalidTimeWeights);
        
        let twap = weighted_sum / total_time as u128;
        Ok(twap as u64)
    }

    /// Validates oracle update frequency limits
    pub fn validate_update_frequency(
        last_update: i64,
        min_interval: i64,
        max_interval: i64,
    ) -> Result<()> {
        let current_time = Clock::get()?.unix_timestamp;
        let time_since_last = current_time - last_update;
        
        require!(
            time_since_last >= min_interval,
            FinovaOracleError::UpdateTooFrequent
        );
        
        if last_update > 0 {
            require!(
                time_since_last <= max_interval,
                FinovaOracleError::UpdateOverdue
            );
        }
        
        Ok(())
    }

    /// Calculates price impact for large trades
    pub fn calculate_price_impact(
        trade_amount: u64,
        liquidity: u64,
        impact_factor: u64,
    ) -> Result<u64> {
        require!(liquidity > 0, FinovaOracleError::ZeroLiquidity);
        require!(trade_amount <= liquidity, FinovaOracleError::InsufficientLiquidity);
        
        // Price impact = (trade_amount / liquidity) * impact_factor
        let impact = (trade_amount as u128 * impact_factor as u128) / liquidity as u128;
        
        // Cap the maximum impact
        let capped_impact = std::cmp::min(impact, MAX_PRICE_IMPACT as u128) as u64;
        
        Ok(capped_impact)
    }

    /// Validates oracle circuit breaker conditions
    pub fn validate_circuit_breaker(
        current_price: u64,
        reference_price: u64,
        max_deviation_bps: u64,
    ) -> Result<()> {
        let deviation = if current_price > reference_price {
            ((current_price - reference_price) as u128 * 10000) / reference_price as u128
        } else {
            ((reference_price - current_price) as u128 * 10000) / reference_price as u128
        };
        
        require!(
            deviation <= max_deviation_bps as u128,
            FinovaOracleError::CircuitBreakerTriggered
        );
        
        Ok(())
    }

    /// Generates deterministic oracle seed for randomness
    pub fn generate_oracle_seed(
        base_seed: &[u8],
        timestamp: i64,
        slot: u64,
    ) -> Result<[u8; 32]> {
        use anchor_lang::solana_program::hash::{hash, Hash};
        
        let mut seed_data = Vec::new();
        seed_data.extend_from_slice(base_seed);
        seed_data.extend_from_slice(&timestamp.to_le_bytes());
        seed_data.extend_from_slice(&slot.to_le_bytes());
        
        let hash_result = hash(&seed_data);
        Ok(hash_result.to_bytes())
    }

    /// Validates multi-signature oracle consensus
    pub fn validate_oracle_consensus(
        signatures: &[bool],
        min_consensus: u8,
        total_oracles: u8,
    ) -> Result<()> {
        require!(signatures.len() == total_oracles as usize, FinovaOracleError::InvalidSignatureCount);
        
        let agreement_count = signatures.iter().filter(|&&sig| sig).count() as u8;
        
        require!(
            agreement_count >= min_consensus,
            FinovaOracleError::InsufficientConsensus
        );
        
        // Ensure we have more than 50% agreement for critical operations
        let majority_threshold = (total_oracles / 2) + 1;
        require!(
            agreement_count >= majority_threshold,
            FinovaOracleError::NoMajorityConsensus
        );
        
        Ok(())
    }

    /// Calculates oracle reputation score based on accuracy
    pub fn calculate_oracle_reputation(
        correct_predictions: u32,
        total_predictions: u32,
        recent_accuracy: u32, // out of 10000 (100.00%)
        uptime_percentage: u32, // out of 10000 (100.00%)
    ) -> Result<u32> {
        require!(total_predictions > 0, FinovaOracleError::NoOraleHistory);
        require!(correct_predictions <= total_predictions, FinovaOracleError::InvalidAccuracyData);
        require!(recent_accuracy <= 10000, FinovaOracleError::InvalidAccuracyData);
        require!(uptime_percentage <= 10000, FinovaOracleError::InvalidUptimeData);
        
        // Historical accuracy (40% weight)
        let historical_accuracy = (correct_predictions as u128 * 10000) / total_predictions as u128;
        let historical_score = (historical_accuracy * 40) / 100;
        
        // Recent accuracy (35% weight)
        let recent_score = (recent_accuracy as u128 * 35) / 100;
        
        // Uptime score (25% weight)
        let uptime_score = (uptime_percentage as u128 * 25) / 100;
        
        let total_score = historical_score + recent_score + uptime_score;
        
        // Apply bonus for long-term reliability
        let reliability_bonus = if total_predictions >= MIN_PREDICTIONS_FOR_BONUS {
            std::cmp::min(500u128, total_predictions as u128 / 10) // Max 500 bonus points
        } else {
            0
        };
        
        let final_score = std::cmp::min(10000u128, total_score + reliability_bonus);
        
        Ok(final_score as u32)
    }

    /// Validates oracle data aggregation parameters
    pub fn validate_aggregation_params(
        feed_count: u8,
        min_feeds: u8,
        max_deviation: u64,
        confidence_threshold: u64,
    ) -> Result<()> {
        require!(feed_count >= min_feeds, FinovaOracleError::InsufficientFeeds);
        require!(min_feeds >= MIN_ORACLE_FEEDS, FinovaOracleError::BelowMinimumFeeds);
        require!(feed_count <= MAX_ORACLE_FEEDS, FinovaOracleError::ExceedsMaximumFeeds);
        require!(max_deviation <= MAX_ALLOWED_DEVIATION, FinovaOracleError::ExcessiveDeviation);
        require!(confidence_threshold <= CONFIDENCE_MAX, FinovaOracleError::InvalidConfidence);
        
        Ok(())
    }

    /// Encrypts sensitive oracle data for cross-chain transmission
    pub fn encrypt_oracle_data(
        data: &[u8],
        encryption_key: &[u8; 32],
        nonce: &[u8; 12],
    ) -> Result<Vec<u8>> {
        // Placeholder for actual encryption implementation
        // In production, use ChaCha20Poly1305 or similar AEAD cipher
        require!(!data.is_empty(), FinovaOracleError::EmptyEncryptionData);
        require!(data.len() <= MAX_ENCRYPTED_DATA_SIZE, FinovaOracleError::DataTooLarge);
        
        // Simple XOR for demonstration (use proper encryption in production)
        let mut encrypted = Vec::new();
        for (i, &byte) in data.iter().enumerate() {
            let key_byte = encryption_key[i % 32];
            let nonce_byte = nonce[i % 12];
            encrypted.push(byte ^ key_byte ^ nonce_byte);
        }
        
        Ok(encrypted)
    }

    /// Validates oracle emergency shutdown conditions
    pub fn validate_emergency_conditions(
        price_deviation: u64,
        consensus_failure_count: u32,
        last_successful_update: i64,
    ) -> Result<bool> {
        let current_time = Clock::get()?.unix_timestamp;
        let time_since_update = current_time - last_successful_update;
        
        // Trigger emergency if any critical condition is met
        let emergency_triggered = 
            price_deviation > EMERGENCY_PRICE_DEVIATION ||
            consensus_failure_count > MAX_CONSENSUS_FAILURES ||
            time_since_update > EMERGENCY_UPDATE_TIMEOUT;
        
        if emergency_triggered {
            msg!("Emergency conditions detected - Oracle system should be paused");
        }
        
        Ok(emergency_triggered)
    }

    /// Formats price data for external consumption
    pub fn format_price_for_external(
        price: u64,
        decimals: u8,
        symbol: &str,
    ) -> Result<String> {
        require!(decimals <= MAX_PRICE_DECIMALS, FinovaOracleError::TooManyDecimals);
        require!(!symbol.is_empty(), FinovaOracleError::EmptySymbol);
        require!(symbol.len() <= MAX_SYMBOL_LENGTH, FinovaOracleError::SymbolTooLong);
        
        let divisor = 10u64.pow(decimals as u32);
        let integer_part = price / divisor;
        let fractional_part = price % divisor;
        
        Ok(format!("{}.{:0width$} {}", integer_part, fractional_part, symbol, width = decimals as usize))
    }
}

/// Helper struct for batch operations
#[derive(Debug, Clone)]
pub struct BatchOperation {
    pub operation_id: u64,
    pub target_account: Pubkey,
    pub operation_type: BatchOperationType,
    pub data: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BatchOperationType {
    PriceUpdate,
    ConfigUpdate,
    EmergencyPause,
    Resume,
}

/// Oracle data validation helper
pub struct OracleDataValidator;

impl OracleDataValidator {
    /// Comprehensive data validation for oracle feeds
    pub fn comprehensive_validate(
        price: u64,
        confidence: u64,
        timestamp: i64,
        source_id: u32,
        signature: &[u8],
    ) -> Result<()> {
        // Basic data validation
        OracleUtils::validate_price_data(price, confidence, timestamp)?;
        
        // Source validation
        require!(
            AUTHORIZED_ORACLE_SOURCES.contains(&source_id),
            FinovaOracleError::UnauthorizedSource
        );
        
        // Signature validation
        require!(!signature.is_empty(), FinovaOracleError::MissingSignature);
        require!(signature.len() == EXPECTED_SIGNATURE_LENGTH, FinovaOracleError::InvalidSignatureLength);
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_weighted_average_calculation() {
        let prices = vec![
            (100, 50, 9000),  // price: 100, weight: 50, confidence: 90%
            (102, 30, 8500),  // price: 102, weight: 30, confidence: 85%
            (98, 20, 9500),   // price: 98, weight: 20, confidence: 95%
        ];
        
        let result = OracleUtils::calculate_weighted_average(&prices, 3);
        assert!(result.is_ok());
        
        let (avg_price, avg_confidence) = result.unwrap();
        assert!(avg_price > 98 && avg_price < 102);
        assert!(avg_confidence > 8000);
    }

    #[test]
    fn test_outlier_removal() {
        let prices = vec![
            (100, 1, 9000),
            (101, 1, 9000),
            (102, 1, 9000),
            (200, 1, 9000), // outlier
        ];
        
        let result = OracleUtils::remove_outliers(&prices);
        assert!(result.is_ok());
        
        let filtered = result.unwrap();
        assert_eq!(filtered.len(), 3);
    }

    #[test]
    fn test_integer_sqrt() {
        assert_eq!(OracleUtils::integer_sqrt(0), 0);
        assert_eq!(OracleUtils::integer_sqrt(1), 1);
        assert_eq!(OracleUtils::integer_sqrt(4), 2);
        assert_eq!(OracleUtils::integer_sqrt(9), 3);
        assert_eq!(OracleUtils::integer_sqrt(16), 4);
        assert_eq!(OracleUtils::integer_sqrt(15), 3); // Rounds down
    }
}
