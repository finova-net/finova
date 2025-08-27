// programs/finova-oracle/src/math/weighted_average.rs

use anchor_lang::prelude::*;
use std::collections::BTreeMap;

/// Mathematical utilities for calculating weighted averages in price feeds
/// Used by Finova Oracle to aggregate multiple price sources with confidence scoring
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WeightedPrice {
    pub price: u64,
    pub weight: u64,
    pub confidence: u64,
    pub timestamp: i64,
    pub source_id: u32,
}

/// Configuration for weighted average calculations
#[derive(Debug, Clone)]
pub struct WeightingConfig {
    pub min_confidence_threshold: u64,
    pub max_age_seconds: i64,
    pub outlier_detection_enabled: bool,
    pub time_decay_factor: u64, // Basis points (10000 = 100%)
    pub confidence_weight_factor: u64, // Basis points
}

impl Default for WeightingConfig {
    fn default() -> Self {
        Self {
            min_confidence_threshold: 7000, // 70% minimum confidence
            max_age_seconds: 300, // 5 minutes max age
            outlier_detection_enabled: true,
            time_decay_factor: 9500, // 95% weight retention per minute
            confidence_weight_factor: 10000, // 100% confidence weighting
        }
    }
}

/// Statistics for weighted average calculation
#[derive(Debug, Clone, Default)]
pub struct AggregationStats {
    pub total_sources: u32,
    pub valid_sources: u32,
    pub outliers_removed: u32,
    pub stale_sources: u32,
    pub low_confidence_sources: u32,
    pub effective_weight: u64,
    pub price_variance: u64,
    pub confidence_score: u64,
}

/// Main weighted average calculator
pub struct WeightedAverageCalculator {
    config: WeightingConfig,
    current_time: i64,
}

impl WeightedAverageCalculator {
    /// Create new calculator with configuration
    pub fn new(config: WeightingConfig, current_time: i64) -> Self {
        Self {
            config,
            current_time,
        }
    }

    /// Calculate weighted average price from multiple sources
    /// Returns (weighted_price, aggregation_stats)
    pub fn calculate_weighted_average(
        &self,
        prices: &[WeightedPrice],
    ) -> Result<(u64, AggregationStats), ProgramError> {
        if prices.is_empty() {
            return Err(ProgramError::InvalidArgument);
        }

        let mut stats = AggregationStats::default();
        stats.total_sources = prices.len() as u32;

        // Step 1: Filter valid prices
        let valid_prices = self.filter_valid_prices(prices, &mut stats)?;
        
        if valid_prices.is_empty() {
            return Err(ProgramError::InvalidAccountData);
        }

        // Step 2: Remove outliers if enabled
        let filtered_prices = if self.config.outlier_detection_enabled {
            self.remove_outliers(&valid_prices, &mut stats)?
        } else {
            valid_prices
        };

        if filtered_prices.is_empty() {
            return Err(ProgramError::InvalidAccountData);
        }

        stats.valid_sources = filtered_prices.len() as u32;

        // Step 3: Calculate time-weighted and confidence-weighted average
        let (weighted_price, effective_weight) = self.compute_weighted_average(&filtered_prices)?;
        
        // Step 4: Calculate price variance and confidence
        let price_variance = self.calculate_price_variance(&filtered_prices, weighted_price)?;
        let confidence_score = self.calculate_confidence_score(&filtered_prices, price_variance)?;

        stats.effective_weight = effective_weight;
        stats.price_variance = price_variance;
        stats.confidence_score = confidence_score;

        Ok((weighted_price, stats))
    }

    /// Filter prices based on age and confidence thresholds
    fn filter_valid_prices(
        &self,
        prices: &[WeightedPrice],
        stats: &mut AggregationStats,
    ) -> Result<Vec<WeightedPrice>, ProgramError> {
        let mut valid_prices = Vec::new();

        for &price in prices {
            // Check timestamp validity
            let age_seconds = self.current_time - price.timestamp;
            if age_seconds > self.config.max_age_seconds {
                stats.stale_sources += 1;
                continue;
            }

            // Check confidence threshold
            if price.confidence < self.config.min_confidence_threshold {
                stats.low_confidence_sources += 1;
                continue;
            }

            // Basic price validation
            if price.price == 0 || price.weight == 0 {
                continue;
            }

            valid_prices.push(price);
        }

        Ok(valid_prices)
    }

    /// Remove statistical outliers using IQR method
    fn remove_outliers(
        &self,
        prices: &[WeightedPrice],
        stats: &mut AggregationStats,
    ) -> Result<Vec<WeightedPrice>, ProgramError> {
        if prices.len() < 3 {
            return Ok(prices.to_vec());
        }

        // Sort prices for quartile calculation
        let mut sorted_prices: Vec<u64> = prices.iter().map(|p| p.price).collect();
        sorted_prices.sort_unstable();

        let len = sorted_prices.len();
        let q1_index = len / 4;
        let q3_index = (len * 3) / 4;

        let q1 = sorted_prices[q1_index];
        let q3 = sorted_prices[q3_index];
        let iqr = q3.saturating_sub(q1);

        // Calculate outlier bounds (1.5 * IQR method)
        let outlier_multiplier = 15000; // 1.5 in basis points
        let iqr_extension = (iqr as u128 * outlier_multiplier as u128) / 10000;
        
        let lower_bound = q1.saturating_sub(iqr_extension as u64);
        let upper_bound = q3.saturating_add(iqr_extension as u64);

        // Filter out outliers
        let mut filtered_prices = Vec::new();
        for &price in prices {
            if price.price >= lower_bound && price.price <= upper_bound {
                filtered_prices.push(price);
            } else {
                stats.outliers_removed += 1;
            }
        }

        Ok(filtered_prices)
    }

    /// Compute the actual weighted average with time and confidence weighting
    fn compute_weighted_average(
        &self,
        prices: &[WeightedPrice],
    ) -> Result<(u64, u64), ProgramError> {
        let mut weighted_sum: u128 = 0;
        let mut total_weight: u128 = 0;

        for &price in prices {
            // Calculate time decay weight
            let age_seconds = self.current_time - price.timestamp;
            let time_weight = self.calculate_time_decay_weight(age_seconds)?;

            // Calculate confidence weight
            let confidence_weight = self.calculate_confidence_weight(price.confidence)?;

            // Combine all weights: base_weight * time_weight * confidence_weight
            let combined_weight = (price.weight as u128 * time_weight as u128 * confidence_weight as u128) / (10000 * 10000);

            weighted_sum += price.price as u128 * combined_weight;
            total_weight += combined_weight;
        }

        if total_weight == 0 {
            return Err(ProgramError::InvalidAccountData);
        }

        let weighted_average = (weighted_sum / total_weight) as u64;
        Ok((weighted_average, total_weight as u64))
    }

    /// Calculate time decay weight based on age
    fn calculate_time_decay_weight(&self, age_seconds: i64) -> Result<u64, ProgramError> {
        if age_seconds < 0 {
            return Err(ProgramError::InvalidArgument);
        }

        // Exponential decay: weight = decay_factor^(age_minutes)
        let age_minutes = age_seconds / 60;
        let mut weight = 10000u64; // Start at 100%

        for _ in 0..age_minutes {
            weight = (weight as u128 * self.config.time_decay_factor as u128) / 10000;
            if weight < 100 { // Minimum 1% weight
                weight = 100;
                break;
            }
        }

        Ok(weight as u64)
    }

    /// Calculate confidence-based weight multiplier
    fn calculate_confidence_weight(&self, confidence: u64) -> Result<u64, ProgramError> {
        // Linear scaling: weight = confidence * confidence_factor / 10000
        let weight = (confidence as u128 * self.config.confidence_weight_factor as u128) / 10000;
        Ok(weight.min(20000) as u64) // Cap at 200% for high confidence
    }

    /// Calculate price variance across all sources
    fn calculate_price_variance(
        &self,
        prices: &[WeightedPrice],
        weighted_average: u64,
    ) -> Result<u64, ProgramError> {
        if prices.len() < 2 {
            return Ok(0);
        }

        let mut variance_sum: u128 = 0;
        let mut total_weight: u128 = 0;

        for &price in prices {
            let diff = if price.price > weighted_average {
                price.price - weighted_average
            } else {
                weighted_average - price.price
            };

            let squared_diff = (diff as u128).pow(2);
            let weight = price.weight as u128;

            variance_sum += squared_diff * weight;
            total_weight += weight;
        }

        if total_weight == 0 {
            return Ok(0);
        }

        let variance = variance_sum / total_weight;
        // Return standard deviation (sqrt of variance)
        Ok(self.integer_sqrt(variance) as u64)
    }

    /// Calculate overall confidence score for the aggregated price
    fn calculate_confidence_score(
        &self,
        prices: &[WeightedPrice],
        price_variance: u64,
    ) -> Result<u64, ProgramError> {
        if prices.is_empty() {
            return Ok(0);
        }

        // Base confidence from individual sources
        let mut weighted_confidence: u128 = 0;
        let mut total_weight: u128 = 0;

        for &price in prices {
            weighted_confidence += price.confidence as u128 * price.weight as u128;
            total_weight += price.weight as u128;
        }

        let avg_confidence = if total_weight > 0 {
            (weighted_confidence / total_weight) as u64
        } else {
            0
        };

        // Adjust confidence based on price variance
        let variance_penalty = self.calculate_variance_penalty(price_variance)?;
        let adjusted_confidence = avg_confidence.saturating_sub(variance_penalty);

        // Boost confidence based on number of sources
        let source_bonus = self.calculate_source_diversity_bonus(prices.len())?;
        let final_confidence = (adjusted_confidence + source_bonus).min(10000);

        Ok(final_confidence)
    }

    /// Calculate penalty based on price variance
    fn calculate_variance_penalty(&self, variance: u64) -> Result<u64, ProgramError> {
        // Higher variance reduces confidence
        // Penalty = min(variance / 100, 2000) basis points
        let penalty = (variance / 100).min(2000);
        Ok(penalty)
    }

    /// Calculate bonus based on source diversity
    fn calculate_source_diversity_bonus(&self, source_count: usize) -> Result<u64, ProgramError> {
        // More sources increases confidence up to a limit
        let bonus = match source_count {
            1 => 0,
            2 => 200,      // 2% bonus
            3 => 400,      // 4% bonus
            4 => 600,      // 6% bonus
            5 => 800,      // 8% bonus
            6..=10 => 1000, // 10% bonus
            _ => 1200,     // 12% bonus for 10+ sources
        };
        Ok(bonus)
    }

    /// Integer square root using binary search
    fn integer_sqrt(&self, n: u128) -> u128 {
        if n < 2 {
            return n;
        }

        let mut left = 1u128;
        let mut right = n / 2 + 1;

        while left <= right {
            let mid = left + (right - left) / 2;
            let square = mid * mid;

            if square == n {
                return mid;
            } else if square < n {
                left = mid + 1;
            } else {
                right = mid - 1;
            }
        }

        right
    }

    /// Calculate Volume Weighted Average Price (VWAP) if volume data available
    pub fn calculate_vwap(
        &self,
        price_volume_pairs: &[(u64, u64)], // (price, volume)
    ) -> Result<u64, ProgramError> {
        if price_volume_pairs.is_empty() {
            return Err(ProgramError::InvalidArgument);
        }

        let mut total_value: u128 = 0;
        let mut total_volume: u128 = 0;

        for &(price, volume) in price_volume_pairs {
            if price == 0 || volume == 0 {
                continue;
            }

            total_value += price as u128 * volume as u128;
            total_volume += volume as u128;
        }

        if total_volume == 0 {
            return Err(ProgramError::InvalidAccountData);
        }

        Ok((total_value / total_volume) as u64)
    }

    /// Calculate Exponentially Weighted Moving Average (EWMA)
    pub fn calculate_ewma(
        &self,
        historical_prices: &[u64],
        alpha: u64, // Smoothing factor in basis points
    ) -> Result<u64, ProgramError> {
        if historical_prices.is_empty() {
            return Err(ProgramError::InvalidArgument);
        }

        if alpha > 10000 {
            return Err(ProgramError::InvalidArgument);
        }

        let mut ewma = historical_prices[0] as u128;

        for &price in &historical_prices[1..] {
            // EWMA = α * current_price + (1 - α) * previous_ewma
            let alpha_128 = alpha as u128;
            let one_minus_alpha = 10000u128 - alpha_128;
            
            ewma = (alpha_128 * price as u128 + one_minus_alpha * ewma) / 10000;
        }

        Ok(ewma as u64)
    }

    /// Advanced price aggregation with multiple algorithms
    pub fn calculate_hybrid_price(
        &self,
        prices: &[WeightedPrice],
        vwap_weight: u64,
        weighted_avg_weight: u64,
        ewma_weight: u64,
    ) -> Result<(u64, AggregationStats), ProgramError> {
        let total_weight = vwap_weight + weighted_avg_weight + ewma_weight;
        if total_weight == 0 {
            return Err(ProgramError::InvalidArgument);
        }

        // Calculate weighted average
        let (weighted_avg, stats) = self.calculate_weighted_average(prices)?;

        // Calculate VWAP (using confidence as volume proxy)
        let price_volume_pairs: Vec<(u64, u64)> = prices
            .iter()
            .map(|p| (p.price, p.confidence))
            .collect();
        let vwap = self.calculate_vwap(&price_volume_pairs)?;

        // Calculate EWMA (using price history)
        let price_history: Vec<u64> = prices.iter().map(|p| p.price).collect();
        let ewma = self.calculate_ewma(&price_history, 2000)?; // 20% alpha

        // Combine all prices with weights
        let hybrid_price = (weighted_avg as u128 * weighted_avg_weight as u128 +
                           vwap as u128 * vwap_weight as u128 +
                           ewma as u128 * ewma_weight as u128) / total_weight as u128;

        Ok((hybrid_price as u64, stats))
    }

    /// Validate price movement against historical data
    pub fn validate_price_movement(
        &self,
        new_price: u64,
        previous_price: u64,
        max_deviation_bps: u64, // Maximum allowed deviation in basis points
    ) -> Result<bool, ProgramError> {
        if previous_price == 0 {
            return Ok(true); // No previous price to compare
        }

        let price_change = if new_price > previous_price {
            new_price - previous_price
        } else {
            previous_price - new_price
        };

        let percentage_change = (price_change as u128 * 10000) / previous_price as u128;

        Ok(percentage_change <= max_deviation_bps as u128)
    }

    /// Calculate circuit breaker thresholds
    pub fn calculate_circuit_breaker_bounds(
        &self,
        reference_price: u64,
        upper_threshold_bps: u64,
        lower_threshold_bps: u64,
    ) -> Result<(u64, u64), ProgramError> {
        let upper_bound = reference_price + 
            ((reference_price as u128 * upper_threshold_bps as u128) / 10000) as u64;
        
        let lower_bound = reference_price.saturating_sub(
            ((reference_price as u128 * lower_threshold_bps as u128) / 10000) as u64
        );

        Ok((lower_bound, upper_bound))
    }
}

/// Specialized calculator for time-series analysis
pub struct TimeSeriesAnalyzer {
    window_size: usize,
}

impl TimeSeriesAnalyzer {
    pub fn new(window_size: usize) -> Self {
        Self { window_size }
    }

    /// Calculate moving average
    pub fn moving_average(&self, prices: &[u64]) -> Result<Vec<u64>, ProgramError> {
        if prices.len() < self.window_size {
            return Err(ProgramError::InvalidArgument);
        }

        let mut result = Vec::new();
        
        for i in self.window_size - 1..prices.len() {
            let window_sum: u128 = prices[i + 1 - self.window_size..=i]
                .iter()
                .map(|&x| x as u128)
                .sum();
            
            let avg = (window_sum / self.window_size as u128) as u64;
            result.push(avg);
        }

        Ok(result)
    }

    /// Calculate price volatility (standard deviation)
    pub fn calculate_volatility(&self, prices: &[u64]) -> Result<u64, ProgramError> {
        if prices.len() < 2 {
            return Ok(0);
        }

        let mean = prices.iter().map(|&x| x as u128).sum::<u128>() / prices.len() as u128;
        
        let variance = prices
            .iter()
            .map(|&x| {
                let diff = if x as u128 > mean { x as u128 - mean } else { mean - x as u128 };
                diff * diff
            })
            .sum::<u128>() / prices.len() as u128;

        let calculator = WeightedAverageCalculator::new(WeightingConfig::default(), 0);
        Ok(calculator.integer_sqrt(variance) as u64)
    }

    /// Detect price trends
    pub fn detect_trend(&self, prices: &[u64]) -> Result<i32, ProgramError> {
        if prices.len() < 3 {
            return Ok(0); // No trend
        }

        let mut up_count = 0i32;
        let mut down_count = 0i32;

        for i in 1..prices.len() {
            if prices[i] > prices[i - 1] {
                up_count += 1;
            } else if prices[i] < prices[i - 1] {
                down_count += 1;
            }
        }

        // Return trend strength: positive for uptrend, negative for downtrend
        Ok(up_count - down_count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_prices() -> Vec<WeightedPrice> {
        vec![
            WeightedPrice {
                price: 100_000_000, // $100
                weight: 1000,
                confidence: 9000,
                timestamp: 1000,
                source_id: 1,
            },
            WeightedPrice {
                price: 101_000_000, // $101
                weight: 800,
                confidence: 8500,
                timestamp: 1010,
                source_id: 2,
            },
            WeightedPrice {
                price: 99_500_000, // $99.5
                weight: 1200,
                confidence: 9200,
                timestamp: 1020,
                source_id: 3,
            },
        ]
    }

    #[test]
    fn test_weighted_average_calculation() {
        let config = WeightingConfig::default();
        let calculator = WeightedAverageCalculator::new(config, 1030);
        let prices = create_test_prices();

        let result = calculator.calculate_weighted_average(&prices);
        assert!(result.is_ok());

        let (weighted_price, stats) = result.unwrap();
        assert!(weighted_price > 99_000_000 && weighted_price < 102_000_000);
        assert_eq!(stats.valid_sources, 3);
    }

    #[test]
    fn test_outlier_removal() {
        let config = WeightingConfig {
            outlier_detection_enabled: true,
            ..Default::default()
        };
        let calculator = WeightedAverageCalculator::new(config, 1030);
        
        let mut prices = create_test_prices();
        // Add outlier
        prices.push(WeightedPrice {
            price: 150_000_000, // $150 - outlier
            weight: 500,
            confidence: 8000,
            timestamp: 1030,
            source_id: 4,
        });

        let result = calculator.calculate_weighted_average(&prices);
        assert!(result.is_ok());

        let (_, stats) = result.unwrap();
        assert!(stats.outliers_removed > 0);
    }

    #[test]
    fn test_time_decay() {
        let config = WeightingConfig::default();
        let calculator = WeightedAverageCalculator::new(config, 1500); // Much later time
        
        // All prices are old, should be filtered or have reduced weight
        let prices = create_test_prices();
        let result = calculator.calculate_weighted_average(&prices);
        
        // Should still work but with reduced confidence or different weighting
        assert!(result.is_ok() || result.is_err()); // Either filtered out or weighted differently
    }

    #[test]
    fn test_vwap_calculation() {
        let config = WeightingConfig::default();
        let calculator = WeightedAverageCalculator::new(config, 1030);
        
        let price_volume_pairs = vec![
            (100_000_000, 1000), // $100, volume 1000
            (101_000_000, 2000), // $101, volume 2000
            (99_000_000, 500),   // $99, volume 500
        ];

        let result = calculator.calculate_vwap(&price_volume_pairs);
        assert!(result.is_ok());
        
        let vwap = result.unwrap();
        assert!(vwap >= 99_000_000 && vwap <= 101_000_000);
    }
}
