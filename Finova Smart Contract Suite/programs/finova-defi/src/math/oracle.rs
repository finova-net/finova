// programs/finova-defi/src/math/oracle.rs

use anchor_lang::prelude::*;
use std::collections::HashMap;

/// Maximum age of price data in seconds (5 minutes)
pub const MAX_PRICE_AGE: i64 = 300;

/// Maximum deviation threshold for price validation (10%)
pub const MAX_PRICE_DEVIATION: u64 = 1000; // 10% in basis points

/// Minimum number of data sources required for consensus
pub const MIN_CONSENSUS_SOURCES: usize = 3;

/// Price confidence threshold (95%)
pub const MIN_CONFIDENCE_THRESHOLD: u64 = 9500; // 95% in basis points

/// Oracle types supported by the system
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq)]
pub enum OracleType {
    Pyth,
    Switchboard,
    Chainlink,
    Custom,
    Twap,
}

/// Price data structure with validation
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug)]
pub struct PriceData {
    /// Price in USD with 8 decimal places
    pub price: u64,
    /// Confidence interval (basis points)
    pub confidence: u64,
    /// Timestamp of price update
    pub timestamp: i64,
    /// Oracle source type
    pub oracle_type: OracleType,
    /// Data source identifier
    pub source_id: u8,
}

/// Aggregated price information
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug)]
pub struct AggregatedPrice {
    /// Weighted average price
    pub price: u64,
    /// Combined confidence score
    pub confidence: u64,
    /// Last update timestamp
    pub last_update: i64,
    /// Number of contributing sources
    pub source_count: u8,
    /// Price deviation from previous
    pub deviation: u64,
}

/// Time-weighted average price calculator
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct TwapCalculator {
    /// Historical price points
    pub price_history: Vec<PricePoint>,
    /// Time window for TWAP calculation (in seconds)
    pub window_size: i64,
    /// Maximum number of historical points to store
    pub max_history: usize,
}

/// Individual price point for TWAP calculation
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug)]
pub struct PricePoint {
    pub price: u64,
    pub timestamp: i64,
    pub volume: u64,
}

/// Oracle validation results
#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub confidence_score: u64,
    pub error_message: Option<String>,
    pub recommendation: PriceRecommendation,
}

/// Price recommendation based on oracle analysis
#[derive(Debug, Clone, PartialEq)]
pub enum PriceRecommendation {
    Accept,
    Reject,
    RequireAdditionalSources,
    UseBackupPrice,
    Emergency,
}

/// Oracle configuration parameters
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct OracleConfig {
    /// Enabled oracle sources
    pub enabled_sources: Vec<OracleType>,
    /// Weight assigned to each source
    pub source_weights: Vec<u64>,
    /// Staleness threshold per source
    pub staleness_thresholds: Vec<i64>,
    /// Emergency fallback configuration
    pub emergency_config: EmergencyConfig,
}

/// Emergency oracle configuration
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct EmergencyConfig {
    /// Use backup price feeds
    pub use_backup_feeds: bool,
    /// Maximum price change allowed in emergency
    pub max_emergency_change: u64,
    /// Emergency mode timeout
    pub emergency_timeout: i64,
    /// Authorized emergency updaters
    pub emergency_updaters: Vec<Pubkey>,
}

impl PriceData {
    /// Create new price data with validation
    pub fn new(
        price: u64,
        confidence: u64,
        oracle_type: OracleType,
        source_id: u8,
    ) -> Result<Self> {
        require!(price > 0, crate::errors::FinovaDefiError::InvalidPrice);
        require!(
            confidence <= 10000,
            crate::errors::FinovaDefiError::InvalidConfidence
        );

        Ok(Self {
            price,
            confidence,
            timestamp: Clock::get()?.unix_timestamp,
            oracle_type,
            source_id,
        })
    }

    /// Check if price data is stale
    pub fn is_stale(&self, max_age: i64) -> bool {
        let current_time = Clock::get().unwrap().unix_timestamp;
        current_time - self.timestamp > max_age
    }

    /// Check if price data meets confidence threshold
    pub fn meets_confidence(&self, min_confidence: u64) -> bool {
        self.confidence >= min_confidence
    }

    /// Validate price data integrity
    pub fn validate(&self) -> ValidationResult {
        let mut errors = Vec::new();
        let current_time = Clock::get().unwrap().unix_timestamp;

        // Check staleness
        if self.is_stale(MAX_PRICE_AGE) {
            errors.push("Price data is stale".to_string());
        }

        // Check confidence
        if !self.meets_confidence(MIN_CONFIDENCE_THRESHOLD) {
            errors.push("Price confidence too low".to_string());
        }

        // Check price validity
        if self.price == 0 {
            errors.push("Invalid zero price".to_string());
        }

        // Check timestamp validity
        if self.timestamp > current_time + 60 {
            errors.push("Future timestamp detected".to_string());
        }

        let is_valid = errors.is_empty();
        let confidence_score = if is_valid { self.confidence } else { 0 };
        
        let recommendation = if is_valid {
            PriceRecommendation::Accept
        } else if errors.len() == 1 && errors[0].contains("confidence") {
            PriceRecommendation::RequireAdditionalSources
        } else {
            PriceRecommendation::Reject
        };

        ValidationResult {
            is_valid,
            confidence_score,
            error_message: if errors.is_empty() { None } else { Some(errors.join(", ")) },
            recommendation,
        }
    }
}

impl TwapCalculator {
    /// Create new TWAP calculator
    pub fn new(window_size: i64, max_history: usize) -> Self {
        Self {
            price_history: Vec::with_capacity(max_history),
            window_size,
            max_history,
        }
    }

    /// Add price point to history
    pub fn add_price_point(&mut self, price: u64, volume: u64) -> Result<()> {
        let timestamp = Clock::get()?.unix_timestamp;
        
        let price_point = PricePoint {
            price,
            timestamp,
            volume,
        };

        // Remove old entries beyond window
        self.cleanup_old_entries(timestamp);

        // Add new point
        self.price_history.push(price_point);

        // Maintain max history limit
        if self.price_history.len() > self.max_history {
            self.price_history.remove(0);
        }

        Ok(())
    }

    /// Calculate time-weighted average price
    pub fn calculate_twap(&self) -> Result<u64> {
        if self.price_history.is_empty() {
            return Err(crate::errors::FinovaDefiError::InsufficientPriceData.into());
        }

        let current_time = Clock::get()?.unix_timestamp;
        let window_start = current_time - self.window_size;

        let mut weighted_sum = 0u128;
        let mut total_weight = 0u128;

        for (i, point) in self.price_history.iter().enumerate() {
            if point.timestamp < window_start {
                continue;
            }

            let time_weight = if i == 0 {
                self.window_size as u128
            } else {
                (point.timestamp - self.price_history[i - 1].timestamp) as u128
            };

            let volume_weight = point.volume as u128;
            let combined_weight = time_weight * (1 + volume_weight / 1_000_000); // Volume scaling

            weighted_sum += (point.price as u128) * combined_weight;
            total_weight += combined_weight;
        }

        if total_weight == 0 {
            return Err(crate::errors::FinovaDefiError::InsufficientPriceData.into());
        }

        Ok((weighted_sum / total_weight) as u64)
    }

    /// Calculate price volatility over the window
    pub fn calculate_volatility(&self) -> Result<u64> {
        if self.price_history.len() < 2 {
            return Ok(0);
        }

        let prices: Vec<u64> = self.price_history
            .iter()
            .filter(|p| {
                let current_time = Clock::get().unwrap().unix_timestamp;
                p.timestamp >= current_time - self.window_size
            })
            .map(|p| p.price)
            .collect();

        if prices.len() < 2 {
            return Ok(0);
        }

        // Calculate standard deviation
        let mean = prices.iter().sum::<u64>() / prices.len() as u64;
        let variance = prices
            .iter()
            .map(|&price| {
                let diff = if price > mean { price - mean } else { mean - price };
                (diff as u128) * (diff as u128)
            })
            .sum::<u128>() / prices.len() as u128;

        // Approximate square root using binary search
        let std_dev = sqrt_u128(variance) as u64;
        
        // Return volatility as percentage of mean (basis points)
        Ok((std_dev * 10000) / mean)
    }

    /// Remove entries older than the window
    fn cleanup_old_entries(&mut self, current_time: i64) {
        let window_start = current_time - self.window_size;
        self.price_history.retain(|point| point.timestamp >= window_start);
    }
}

/// Oracle aggregation and consensus logic
pub struct OracleAggregator;

impl OracleAggregator {
    /// Aggregate multiple price sources using weighted average
    pub fn aggregate_prices(
        price_feeds: &[PriceData],
        weights: &[u64],
    ) -> Result<AggregatedPrice> {
        require!(
            !price_feeds.is_empty(),
            crate::errors::FinovaDefiError::NoPriceFeeds
        );
        require!(
            price_feeds.len() == weights.len(),
            crate::errors::FinovaDefiError::MismatchedWeights
        );

        let current_time = Clock::get()?.unix_timestamp;
        let mut valid_feeds = Vec::new();
        let mut valid_weights = Vec::new();

        // Filter valid feeds
        for (i, feed) in price_feeds.iter().enumerate() {
            let validation = feed.validate();
            if validation.is_valid || validation.recommendation == PriceRecommendation::Accept {
                valid_feeds.push(*feed);
                valid_weights.push(weights[i]);
            }
        }

        require!(
            valid_feeds.len() >= MIN_CONSENSUS_SOURCES,
            crate::errors::FinovaDefiError::InsufficientConsensus
        );

        // Calculate weighted average
        let mut weighted_price_sum = 0u128;
        let mut weighted_confidence_sum = 0u128;
        let mut total_weight = 0u128;

        for (feed, &weight) in valid_feeds.iter().zip(valid_weights.iter()) {
            let adjusted_weight = weight as u128 * feed.confidence as u128 / 10000u128;
            
            weighted_price_sum += feed.price as u128 * adjusted_weight;
            weighted_confidence_sum += feed.confidence as u128 * adjusted_weight;
            total_weight += adjusted_weight;
        }

        require!(total_weight > 0, crate::errors::FinovaDefiError::ZeroWeight);

        let aggregated_price = (weighted_price_sum / total_weight) as u64;
        let aggregated_confidence = (weighted_confidence_sum / total_weight) as u64;

        // Calculate price deviation
        let deviation = Self::calculate_price_deviation(&valid_feeds, aggregated_price)?;

        Ok(AggregatedPrice {
            price: aggregated_price,
            confidence: aggregated_confidence,
            last_update: current_time,
            source_count: valid_feeds.len() as u8,
            deviation,
        })
    }

    /// Detect price manipulation or anomalies
    pub fn detect_price_anomaly(
        current_price: u64,
        historical_prices: &[u64],
        volatility_threshold: u64,
    ) -> bool {
        if historical_prices.is_empty() {
            return false;
        }

        // Calculate recent average
        let recent_avg = historical_prices.iter().sum::<u64>() / historical_prices.len() as u64;
        
        // Calculate deviation from recent average
        let deviation = if current_price > recent_avg {
            ((current_price - recent_avg) as u128 * 10000u128) / recent_avg as u128
        } else {
            ((recent_avg - current_price) as u128 * 10000u128) / recent_avg as u128
        };

        deviation > volatility_threshold as u128
    }

    /// Calculate circuit breaker trigger conditions
    pub fn should_trigger_circuit_breaker(
        price_change: u64,
        volume_change: u64,
        confidence_drop: u64,
    ) -> bool {
        const MAX_PRICE_CHANGE: u64 = 2000; // 20%
        const MAX_VOLUME_SPIKE: u64 = 500; // 5x volume
        const MIN_CONFIDENCE_EMERGENCY: u64 = 5000; // 50%

        price_change > MAX_PRICE_CHANGE
            || volume_change > MAX_VOLUME_SPIKE
            || confidence_drop < MIN_CONFIDENCE_EMERGENCY
    }

    /// Emergency price validation with strict checks
    pub fn emergency_price_validation(
        new_price: u64,
        last_valid_price: u64,
        emergency_config: &EmergencyConfig,
    ) -> ValidationResult {
        let price_change = if new_price > last_valid_price {
            ((new_price - last_valid_price) as u128 * 10000u128) / last_valid_price as u128
        } else {
            ((last_valid_price - new_price) as u128 * 10000u128) / last_valid_price as u128
        };

        if price_change > emergency_config.max_emergency_change as u128 {
            return ValidationResult {
                is_valid: false,
                confidence_score: 0,
                error_message: Some("Price change exceeds emergency threshold".to_string()),
                recommendation: PriceRecommendation::Emergency,
            };
        }

        ValidationResult {
            is_valid: true,
            confidence_score: 7500, // Reduced confidence in emergency
            error_message: None,
            recommendation: PriceRecommendation::Accept,
        }
    }

    /// Calculate price deviation among sources
    fn calculate_price_deviation(feeds: &[PriceData], average_price: u64) -> Result<u64> {
        if feeds.is_empty() || average_price == 0 {
            return Ok(0);
        }

        let mut max_deviation = 0u64;

        for feed in feeds {
            let deviation = if feed.price > average_price {
                ((feed.price - average_price) as u128 * 10000u128) / average_price as u128
            } else {
                ((average_price - feed.price) as u128 * 10000u128) / average_price as u128
            };

            max_deviation = max_deviation.max(deviation as u64);
        }

        Ok(max_deviation)
    }
}

/// Utility functions for oracle operations
pub struct OracleUtils;

impl OracleUtils {
    /// Convert price from different decimal formats
    pub fn normalize_price(price: u64, from_decimals: u8, to_decimals: u8) -> Result<u64> {
        if from_decimals == to_decimals {
            return Ok(price);
        }

        if from_decimals > to_decimals {
            let divisor = 10u64.pow((from_decimals - to_decimals) as u32);
            Ok(price / divisor)
        } else {
            let multiplier = 10u64.pow((to_decimals - from_decimals) as u32);
            price.checked_mul(multiplier)
                .ok_or(crate::errors::FinovaDefiError::MathOverflow.into())
        }
    }

    /// Calculate exponential moving average
    pub fn calculate_ema(current_price: u64, previous_ema: u64, alpha: u64) -> Result<u64> {
        // EMA = alpha * current_price + (1 - alpha) * previous_ema
        // Alpha is in basis points (0-10000)
        require!(alpha <= 10000, crate::errors::FinovaDefiError::InvalidAlpha);

        let alpha_complement = 10000 - alpha;
        
        let weighted_current = (current_price as u128 * alpha as u128) / 10000u128;
        let weighted_previous = (previous_ema as u128 * alpha_complement as u128) / 10000u128;
        
        Ok((weighted_current + weighted_previous) as u64)
    }

    /// Validate oracle signature (placeholder for actual implementation)
    pub fn validate_oracle_signature(
        data: &[u8],
        signature: &[u8],
        oracle_pubkey: &Pubkey,
    ) -> bool {
        // In a real implementation, this would verify the cryptographic signature
        // For now, we'll do basic validation
        !data.is_empty() && !signature.is_empty() && oracle_pubkey != &Pubkey::default()
    }

    /// Calculate confidence score based on multiple factors
    pub fn calculate_confidence_score(
        price_feeds: &[PriceData],
        historical_volatility: u64,
        market_conditions: MarketConditions,
    ) -> u64 {
        if price_feeds.is_empty() {
            return 0;
        }

        // Base confidence from individual feeds
        let avg_confidence = price_feeds.iter()
            .map(|feed| feed.confidence)
            .sum::<u64>() / price_feeds.len() as u64;

        // Consensus bonus (more sources = higher confidence)
        let consensus_bonus = match price_feeds.len() {
            1 => 0,
            2 => 500,  // 5%
            3 => 1000, // 10%
            4 => 1500, // 15%
            _ => 2000, // 20%
        };

        // Volatility penalty (higher volatility = lower confidence)
        let volatility_penalty = match historical_volatility {
            0..=500 => 0,      // 0-5% volatility
            501..=1000 => 500, // 5-10% volatility
            1001..=2000 => 1000, // 10-20% volatility
            _ => 2000,         // >20% volatility
        };

        // Market conditions adjustment
        let market_adjustment = match market_conditions {
            MarketConditions::Normal => 0,
            MarketConditions::Volatile => -1000,
            MarketConditions::Crisis => -2000,
            MarketConditions::Emergency => -3000,
        };

        let final_confidence = (avg_confidence as i64 + consensus_bonus as i64 
            - volatility_penalty as i64 + market_adjustment)
            .max(0)
            .min(10000) as u64;

        final_confidence
    }
}

/// Market condition indicators
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MarketConditions {
    Normal,
    Volatile,
    Crisis,
    Emergency,
}

/// Price feed source configuration
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct PriceFeedSource {
    /// Oracle type
    pub oracle_type: OracleType,
    /// Source account address
    pub source_account: Pubkey,
    /// Weight in aggregation (basis points)
    pub weight: u64,
    /// Maximum staleness allowed (seconds)
    pub max_staleness: i64,
    /// Minimum confidence required
    pub min_confidence: u64,
    /// Whether source is currently active
    pub is_active: bool,
}

impl PriceFeedSource {
    /// Create new price feed source configuration
    pub fn new(
        oracle_type: OracleType,
        source_account: Pubkey,
        weight: u64,
        max_staleness: i64,
        min_confidence: u64,
    ) -> Result<Self> {
        require!(weight > 0 && weight <= 10000, crate::errors::FinovaDefiError::InvalidWeight);
        require!(max_staleness > 0, crate::errors::FinovaDefiError::InvalidStaleness);
        require!(min_confidence <= 10000, crate::errors::FinovaDefiError::InvalidConfidence);

        Ok(Self {
            oracle_type,
            source_account,
            weight,
            max_staleness,
            min_confidence,
            is_active: true,
        })
    }

    /// Validate if source meets requirements
    pub fn validate_source(&self, price_data: &PriceData) -> bool {
        self.is_active
            && price_data.oracle_type == self.oracle_type
            && !price_data.is_stale(self.max_staleness)
            && price_data.meets_confidence(self.min_confidence)
    }
}

/// Helper function for integer square root
fn sqrt_u128(n: u128) -> u128 {
    if n == 0 {
        return 0;
    }

    let mut x = n;
    let mut y = (n + 1) / 2;

    while y < x {
        x = y;
        y = (x + n / x) / 2;
    }

    x
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_price_data_validation() {
        let price_data = PriceData::new(100_000_000, 9000, OracleType::Pyth, 1).unwrap();
        let validation = price_data.validate();
        assert!(validation.is_valid);
        assert_eq!(validation.recommendation, PriceRecommendation::Accept);
    }

    #[test]
    fn test_twap_calculation() {
        let mut twap = TwapCalculator::new(300, 100); // 5 minute window
        
        // Add some price points
        twap.add_price_point(100_000_000, 1000).unwrap();
        twap.add_price_point(101_000_000, 1500).unwrap();
        twap.add_price_point(99_000_000, 800).unwrap();
        
        let twap_price = twap.calculate_twap().unwrap();
        assert!(twap_price > 99_000_000 && twap_price < 101_000_000);
    }

    #[test]
    fn test_price_aggregation() {
        let feeds = vec![
            PriceData::new(100_000_000, 9500, OracleType::Pyth, 1).unwrap(),
            PriceData::new(100_500_000, 9200, OracleType::Switchboard, 2).unwrap(),
            PriceData::new(99_800_000, 9300, OracleType::Chainlink, 3).unwrap(),
        ];
        
        let weights = vec![4000, 3000, 3000]; // 40%, 30%, 30%
        
        let aggregated = OracleAggregator::aggregate_prices(&feeds, &weights).unwrap();
        assert!(aggregated.price > 99_000_000 && aggregated.price < 101_000_000);
        assert!(aggregated.confidence > 9000);
        assert_eq!(aggregated.source_count, 3);
    }

    #[test]
    fn test_price_normalization() {
        // Convert from 6 decimals to 8 decimals
        let normalized = OracleUtils::normalize_price(1_000_000, 6, 8).unwrap();
        assert_eq!(normalized, 100_000_000);
        
        // Convert from 8 decimals to 6 decimals
        let normalized = OracleUtils::normalize_price(100_000_000, 8, 6).unwrap();
        assert_eq!(normalized, 1_000_000);
    }

    #[test]
    fn test_anomaly_detection() {
        let historical = vec![100_000_000, 101_000_000, 99_500_000, 100_200_000];
        
        // Normal price should not trigger anomaly
        assert!(!OracleAggregator::detect_price_anomaly(100_500_000, &historical, 2000));
        
        // Extreme price should trigger anomaly
        assert!(OracleAggregator::detect_price_anomaly(130_000_000, &historical, 2000));
    }

    #[test]
    fn test_ema_calculation() {
        let current_price = 100_000_000;
        let previous_ema = 99_000_000;
        let alpha = 2000; // 20% smoothing factor
        
        let ema = OracleUtils::calculate_ema(current_price, previous_ema, alpha).unwrap();
        assert!(ema > previous_ema && ema < current_price);
    }
}
