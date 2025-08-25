// programs/finova-oracle/src/math/mod.rs

pub mod weighted_average;
pub mod outlier_detection;

use anchor_lang::prelude::*;
use std::collections::VecDeque;

/// Mathematical utilities for oracle price calculations
pub struct MathUtils;

impl MathUtils {
    /// Calculate exponential moving average
    pub fn exponential_moving_average(
        current_price: u64,
        previous_ema: u64,
        alpha: u64, // Smoothing factor (0-10000, representing 0.0000-1.0000)
    ) -> Result<u64> {
        require!(alpha <= 10000, crate::errors::OracleError::InvalidAlpha);
        
        let alpha_scaled = alpha as u128;
        let current_scaled = current_price as u128;
        let previous_scaled = previous_ema as u128;
        
        // EMA = α * current_price + (1 - α) * previous_ema
        let new_ema = (alpha_scaled * current_scaled + 
                      (10000 - alpha_scaled) * previous_scaled) / 10000;
        
        Ok(new_ema as u64)
    }
    
    /// Calculate simple moving average
    pub fn simple_moving_average(prices: &[u64]) -> Result<u64> {
        require!(!prices.is_empty(), crate::errors::OracleError::EmptyPriceArray);
        
        let sum: u128 = prices.iter().map(|&p| p as u128).sum();
        Ok((sum / prices.len() as u128) as u64)
    }
    
    /// Calculate weighted moving average based on timestamps
    pub fn time_weighted_average(
        prices: &[u64],
        timestamps: &[i64],
        current_time: i64,
    ) -> Result<u64> {
        require!(
            prices.len() == timestamps.len() && !prices.is_empty(),
            crate::errors::OracleError::MismatchedArrays
        );
        
        let mut weighted_sum: u128 = 0;
        let mut total_weights: u128 = 0;
        
        for (i, (&price, &timestamp)) in prices.iter().zip(timestamps.iter()).enumerate() {
            // Recent prices get higher weight
            let age = (current_time - timestamp).max(1) as u128;
            let weight = 1_000_000_000 / age; // Higher weight for recent prices
            
            weighted_sum += price as u128 * weight;
            total_weights += weight;
        }
        
        require!(total_weights > 0, crate::errors::OracleError::ZeroTotalWeight);
        Ok((weighted_sum / total_weights) as u64)
    }
    
    /// Calculate volatility using standard deviation
    pub fn calculate_volatility(prices: &[u64]) -> Result<u64> {
        require!(prices.len() >= 2, crate::errors::OracleError::InsufficientDataPoints);
        
        let mean = Self::simple_moving_average(prices)?;
        let mean_f64 = mean as f64;
        
        let variance: f64 = prices
            .iter()
            .map(|&price| {
                let diff = price as f64 - mean_f64;
                diff * diff
            })
            .sum::<f64>() / (prices.len() - 1) as f64;
        
        Ok(variance.sqrt() as u64)
    }
    
    /// Calculate confidence score based on data consistency
    pub fn calculate_confidence_score(
        prices: &[u64],
        timestamps: &[i64],
        max_age_seconds: i64,
        current_time: i64,
    ) -> Result<u8> {
        require!(!prices.is_empty(), crate::errors::OracleError::EmptyPriceArray);
        
        let mut confidence_factors = Vec::new();
        
        // Factor 1: Data freshness (0-40 points)
        let oldest_timestamp = timestamps.iter().min().unwrap_or(&current_time);
        let age = current_time - oldest_timestamp;
        let freshness_score = if age <= max_age_seconds / 4 {
            40
        } else if age <= max_age_seconds / 2 {
            30
        } else if age <= max_age_seconds {
            20
        } else {
            10
        };
        confidence_factors.push(freshness_score);
        
        // Factor 2: Data quantity (0-20 points)
        let quantity_score = match prices.len() {
            1 => 5,
            2..=3 => 10,
            4..=5 => 15,
            _ => 20,
        };
        confidence_factors.push(quantity_score);
        
        // Factor 3: Price consistency (0-40 points)
        if prices.len() >= 2 {
            let volatility = Self::calculate_volatility(prices)?;
            let mean = Self::simple_moving_average(prices)?;
            let volatility_ratio = if mean > 0 {
                (volatility * 100) / mean
            } else {
                100
            };
            
            let consistency_score = if volatility_ratio <= 5 {
                40
            } else if volatility_ratio <= 10 {
                30
            } else if volatility_ratio <= 20 {
                20
            } else {
                10
            };
            confidence_factors.push(consistency_score);
        } else {
            confidence_factors.push(20); // Neutral score for single data point
        }
        
        let total_score: u32 = confidence_factors.iter().sum();
        Ok((total_score.min(100)) as u8)
    }
    
    /// Normalize price to standard decimal places
    pub fn normalize_price(price: u64, from_decimals: u8, to_decimals: u8) -> Result<u64> {
        require!(
            from_decimals <= 18 && to_decimals <= 18,
            crate::errors::OracleError::InvalidDecimals
        );
        
        if from_decimals == to_decimals {
            return Ok(price);
        }
        
        let price_scaled = price as u128;
        
        if from_decimals > to_decimals {
            let divisor = 10_u128.pow((from_decimals - to_decimals) as u32);
            Ok((price_scaled / divisor) as u64)
        } else {
            let multiplier = 10_u128.pow((to_decimals - from_decimals) as u32);
            let result = price_scaled * multiplier;
            require!(result <= u64::MAX as u128, crate::errors::OracleError::PriceOverflow);
            Ok(result as u64)
        }
    }
    
    /// Check if price deviation is within acceptable bounds
    pub fn is_price_deviation_acceptable(
        new_price: u64,
        reference_price: u64,
        max_deviation_bps: u16, // Basis points (10000 = 100%)
    ) -> bool {
        if reference_price == 0 {
            return true; // First price, always acceptable
        }
        
        let max_dev = max_deviation_bps as u128;
        let new_price_scaled = new_price as u128;
        let ref_price_scaled = reference_price as u128;
        
        let deviation = if new_price_scaled > ref_price_scaled {
            ((new_price_scaled - ref_price_scaled) * 10000) / ref_price_scaled
        } else {
            ((ref_price_scaled - new_price_scaled) * 10000) / ref_price_scaled
        };
        
        deviation <= max_dev
    }
    
    /// Calculate price impact based on trading volume
    pub fn calculate_price_impact(
        base_price: u64,
        trade_volume: u64,
        liquidity_depth: u64,
        impact_factor: u16, // Basis points
    ) -> Result<u64> {
        require!(liquidity_depth > 0, crate::errors::OracleError::ZeroLiquidity);
        
        let volume_ratio = (trade_volume as u128 * 10000) / liquidity_depth as u128;
        let impact_multiplier = (impact_factor as u128 * volume_ratio) / 10000;
        let price_impact = (base_price as u128 * impact_multiplier) / 10000;
        
        Ok((base_price as u128 + price_impact) as u64)
    }
    
    /// Calculate TWAP (Time-Weighted Average Price)
    pub fn calculate_twap(
        prices: &[u64],
        timestamps: &[i64],
        start_time: i64,
        end_time: i64,
    ) -> Result<u64> {
        require!(
            prices.len() == timestamps.len() && prices.len() >= 2,
            crate::errors::OracleError::InsufficientDataPoints
        );
        require!(end_time > start_time, crate::errors::OracleError::InvalidTimeRange);
        
        let mut weighted_sum: u128 = 0;
        let mut total_time: u128 = 0;
        
        for i in 0..prices.len() - 1 {
            let current_time = timestamps[i].max(start_time);
            let next_time = timestamps[i + 1].min(end_time);
            
            if next_time > current_time {
                let duration = (next_time - current_time) as u128;
                weighted_sum += prices[i] as u128 * duration;
                total_time += duration;
            }
            
            if timestamps[i + 1] >= end_time {
                break;
            }
        }
        
        require!(total_time > 0, crate::errors::OracleError::ZeroTotalTime);
        Ok((weighted_sum / total_time) as u64)
    }
    
    /// Apply circuit breaker logic
    pub fn check_circuit_breaker(
        new_price: u64,
        reference_prices: &[u64],
        timestamps: &[i64],
        circuit_breaker_threshold: u16, // Basis points
        min_data_points: usize,
    ) -> Result<bool> {
        if reference_prices.len() < min_data_points {
            return Ok(true); // Not enough data, allow the price
        }
        
        let reference_price = Self::simple_moving_average(reference_prices)?;
        
        Ok(Self::is_price_deviation_acceptable(
            new_price,
            reference_price,
            circuit_breaker_threshold,
        ))
    }
    
    /// Calculate liquidity-adjusted price
    pub fn calculate_liquidity_adjusted_price(
        prices: &[u64],
        liquidity_weights: &[u64],
    ) -> Result<u64> {
        require!(
            prices.len() == liquidity_weights.len() && !prices.is_empty(),
            crate::errors::OracleError::MismatchedArrays
        );
        
        let mut weighted_sum: u128 = 0;
        let mut total_weights: u128 = 0;
        
        for (&price, &weight) in prices.iter().zip(liquidity_weights.iter()) {
            weighted_sum += price as u128 * weight as u128;
            total_weights += weight as u128;
        }
        
        require!(total_weights > 0, crate::errors::OracleError::ZeroTotalWeight);
        Ok((weighted_sum / total_weights) as u64)
    }
    
    /// Calculate exponential decay weight based on age
    pub fn calculate_decay_weight(
        age_seconds: i64,
        half_life_seconds: i64,
    ) -> u64 {
        if age_seconds <= 0 {
            return 1_000_000; // Maximum weight for current data
        }
        
        if half_life_seconds <= 0 {
            return 500_000; // Default moderate weight
        }
        
        // Approximate exponential decay: weight = e^(-age * ln(2) / half_life)
        // Using integer approximation to avoid floating point
        let decay_factor = (age_seconds * 693) / (half_life_seconds * 1000); // ln(2) ≈ 0.693
        
        if decay_factor >= 10 {
            return 1; // Minimum weight
        }
        
        // Approximate e^(-decay_factor) using Taylor series
        let weight = 1_000_000 / (1 + decay_factor as u64 * 100);
        weight.max(1).min(1_000_000)
    }
    
    /// Calculate correlation coefficient between two price series
    pub fn calculate_correlation(
        prices_a: &[u64],
        prices_b: &[u64],
    ) -> Result<i32> {
        require!(
            prices_a.len() == prices_b.len() && prices_a.len() >= 2,
            crate::errors::OracleError::InsufficientDataPoints
        );
        
        let mean_a = Self::simple_moving_average(prices_a)? as f64;
        let mean_b = Self::simple_moving_average(prices_b)? as f64;
        
        let mut numerator = 0.0;
        let mut sum_sq_a = 0.0;
        let mut sum_sq_b = 0.0;
        
        for (&price_a, &price_b) in prices_a.iter().zip(prices_b.iter()) {
            let diff_a = price_a as f64 - mean_a;
            let diff_b = price_b as f64 - mean_b;
            
            numerator += diff_a * diff_b;
            sum_sq_a += diff_a * diff_a;
            sum_sq_b += diff_b * diff_b;
        }
        
        let denominator = (sum_sq_a * sum_sq_b).sqrt();
        
        if denominator == 0.0 {
            return Ok(0); // No correlation if no variance
        }
        
        let correlation = numerator / denominator;
        Ok((correlation * 10000.0) as i32) // Scale to basis points
    }
    
    /// Validate price bounds
    pub fn validate_price_bounds(
        price: u64,
        min_price: u64,
        max_price: u64,
    ) -> bool {
        price >= min_price && price <= max_price && min_price <= max_price
    }
    
    /// Calculate compound interest for staking rewards
    pub fn calculate_compound_interest(
        principal: u64,
        annual_rate_bps: u16, // Annual interest rate in basis points
        periods: u32,         // Number of compounding periods
        periods_per_year: u32,
    ) -> Result<u64> {
        require!(periods_per_year > 0, crate::errors::OracleError::InvalidPeriods);
        
        let rate_per_period = annual_rate_bps as u128 / (periods_per_year as u128 * 10000);
        let principal_scaled = principal as u128;
        
        let mut result = principal_scaled;
        
        // Calculate compound interest using iterative approach to avoid overflow
        for _ in 0..periods {
            let interest = (result * rate_per_period) / 10000;
            result += interest;
            
            // Prevent overflow
            if result > u64::MAX as u128 {
                return Err(crate::errors::OracleError::CalculationOverflow.into());
            }
        }
        
        Ok(result as u64)
    }
    
    /// Calculate slippage for large trades
    pub fn calculate_slippage(
        trade_size: u64,
        liquidity_depth: u64,
        slippage_factor: u16, // Basis points per % of liquidity used
    ) -> Result<u16> {
        require!(liquidity_depth > 0, crate::errors::OracleError::ZeroLiquidity);
        
        let liquidity_percentage = (trade_size as u128 * 10000) / liquidity_depth as u128;
        let slippage_bps = (liquidity_percentage * slippage_factor as u128) / 10000;
        
        Ok(slippage_bps.min(10000) as u16) // Cap at 100%
    }
}

/// Price history management for calculations
#[derive(Clone, Debug)]
pub struct PriceHistory {
    pub prices: VecDeque<u64>,
    pub timestamps: VecDeque<i64>,
    pub max_size: usize,
}

impl PriceHistory {
    pub fn new(max_size: usize) -> Self {
        Self {
            prices: VecDeque::new(),
            timestamps: VecDeque::new(),
            max_size,
        }
    }
    
    pub fn add_price(&mut self, price: u64, timestamp: i64) {
        if self.prices.len() >= self.max_size {
            self.prices.pop_front();
            self.timestamps.pop_front();
        }
        
        self.prices.push_back(price);
        self.timestamps.push_back(timestamp);
    }
    
    pub fn get_recent_prices(&self, count: usize) -> Vec<u64> {
        self.prices
            .iter()
            .rev()
            .take(count)
            .copied()
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect()
    }
    
    pub fn get_recent_timestamps(&self, count: usize) -> Vec<i64> {
        self.timestamps
            .iter()
            .rev()
            .take(count)
            .copied()
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect()
    }
    
    pub fn calculate_sma(&self, periods: usize) -> Result<u64> {
        let recent_prices = self.get_recent_prices(periods);
        if recent_prices.is_empty() {
            return Err(crate::errors::OracleError::EmptyPriceArray.into());
        }
        MathUtils::simple_moving_average(&recent_prices)
    }
    
    pub fn calculate_ema(&self, periods: usize, alpha: u64) -> Result<u64> {
        let recent_prices = self.get_recent_prices(periods);
        if recent_prices.is_empty() {
            return Err(crate::errors::OracleError::EmptyPriceArray.into());
        }
        
        let mut ema = recent_prices[0];
        for &price in recent_prices.iter().skip(1) {
            ema = MathUtils::exponential_moving_average(price, ema, alpha)?;
        }
        
        Ok(ema)
    }
    
    pub fn is_empty(&self) -> bool {
        self.prices.is_empty()
    }
    
    pub fn len(&self) -> usize {
        self.prices.len()
    }
    
    pub fn latest_price(&self) -> Option<u64> {
        self.prices.back().copied()
    }
    
    pub fn latest_timestamp(&self) -> Option<i64> {
        self.timestamps.back().copied()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_simple_moving_average() {
        let prices = vec![100, 110, 120, 130, 140];
        let avg = MathUtils::simple_moving_average(&prices).unwrap();
        assert_eq!(avg, 120);
    }
    
    #[test]
    fn test_exponential_moving_average() {
        let current_price = 110;
        let previous_ema = 100;
        let alpha = 2000; // 0.2
        
        let ema = MathUtils::exponential_moving_average(current_price, previous_ema, alpha).unwrap();
        assert_eq!(ema, 102); // 0.2 * 110 + 0.8 * 100 = 102
    }
    
    #[test]
    fn test_price_deviation() {
        let new_price = 110;
        let reference_price = 100;
        let max_deviation_bps = 1000; // 10%
        
        assert!(MathUtils::is_price_deviation_acceptable(
            new_price,
            reference_price,
            max_deviation_bps
        ));
        
        let new_price = 120;
        assert!(!MathUtils::is_price_deviation_acceptable(
            new_price,
            reference_price,
            max_deviation_bps
        ));
    }
    
    #[test]
    fn test_price_history() {
        let mut history = PriceHistory::new(3);
        
        history.add_price(100, 1000);
        history.add_price(110, 2000);
        history.add_price(120, 3000);
        history.add_price(130, 4000); // Should evict first price
        
        assert_eq!(history.len(), 3);
        assert_eq!(history.latest_price(), Some(130));
        
        let recent_prices = history.get_recent_prices(2);
        assert_eq!(recent_prices, vec![120, 130]);
    }
    
    #[test]
    fn test_normalize_price() {
        let price = 1000000; // 1.0 with 6 decimals
        let normalized = MathUtils::normalize_price(price, 6, 8).unwrap();
        assert_eq!(normalized, 100000000); // 1.0 with 8 decimals
        
        let normalized_down = MathUtils::normalize_price(100000000, 8, 6).unwrap();
        assert_eq!(normalized_down, 1000000); // Back to 6 decimals
    }
}
