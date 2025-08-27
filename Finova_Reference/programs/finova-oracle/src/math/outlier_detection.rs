// programs/finova-oracle/src/math/outlier_detection.rs

use anchor_lang::prelude::*;
use std::cmp::Ordering;

/// Statistical methods for outlier detection in price feeds
/// Implements multiple algorithms to ensure data quality and prevent manipulation
#[derive(Clone, Debug)]
pub struct OutlierDetector {
    /// Z-score threshold for outlier detection (typically 2.0-3.0)
    pub z_score_threshold: f64,
    /// Minimum number of data points required for analysis
    pub min_data_points: usize,
    /// Maximum allowed deviation percentage from median
    pub max_deviation_percent: f64,
    /// Historical volatility factor for dynamic thresholds
    pub volatility_factor: f64,
}

impl Default for OutlierDetector {
    fn default() -> Self {
        Self {
            z_score_threshold: 2.5,
            min_data_points: 5,
            max_deviation_percent: 20.0,
            volatility_factor: 1.5,
        }
    }
}

impl OutlierDetector {
    /// Creates a new outlier detector with custom parameters
    pub fn new(
        z_score_threshold: f64,
        min_data_points: usize,
        max_deviation_percent: f64,
        volatility_factor: f64,
    ) -> Result<Self> {
        require!(z_score_threshold > 0.0, crate::errors::ErrorCode::InvalidThreshold);
        require!(min_data_points >= 3, crate::errors::ErrorCode::InsufficientDataPoints);
        require!(max_deviation_percent > 0.0, crate::errors::ErrorCode::InvalidDeviation);
        require!(volatility_factor > 0.0, crate::errors::ErrorCode::InvalidVolatilityFactor);

        Ok(Self {
            z_score_threshold,
            min_data_points,
            max_deviation_percent,
            volatility_factor,
        })
    }

    /// Detects outliers using multiple statistical methods
    pub fn detect_outliers(&self, prices: &[u64]) -> Result<Vec<bool>> {
        require!(!prices.is_empty(), crate::errors::ErrorCode::EmptyPriceArray);
        
        if prices.len() < self.min_data_points {
            // If insufficient data, mark all as valid (no outliers)
            return Ok(vec![false; prices.len()]);
        }

        let mut outlier_flags = vec![false; prices.len()];
        
        // Method 1: Z-score based detection
        let z_score_outliers = self.detect_z_score_outliers(prices)?;
        
        // Method 2: Interquartile Range (IQR) method
        let iqr_outliers = self.detect_iqr_outliers(prices)?;
        
        // Method 3: Modified Z-score using Median Absolute Deviation (MAD)
        let mad_outliers = self.detect_mad_outliers(prices)?;
        
        // Method 4: Percentage deviation from median
        let deviation_outliers = self.detect_deviation_outliers(prices)?;
        
        // Combine results using majority voting
        for i in 0..prices.len() {
            let outlier_count = [
                z_score_outliers[i],
                iqr_outliers[i],
                mad_outliers[i],
                deviation_outliers[i],
            ].iter().filter(|&&x| x).count();
            
            // Mark as outlier if at least 2 out of 4 methods detect it
            outlier_flags[i] = outlier_count >= 2;
        }

        Ok(outlier_flags)
    }

    /// Z-score based outlier detection
    /// Identifies data points that are more than z_score_threshold standard deviations from the mean
    fn detect_z_score_outliers(&self, prices: &[u64]) -> Result<Vec<bool>> {
        let mean = self.calculate_mean(prices)?;
        let std_dev = self.calculate_standard_deviation(prices, mean)?;
        
        if std_dev == 0.0 {
            // All values are the same, no outliers
            return Ok(vec![false; prices.len()]);
        }
        
        let mut outliers = vec![false; prices.len()];
        
        for (i, &price) in prices.iter().enumerate() {
            let z_score = ((price as f64) - mean).abs() / std_dev;
            outliers[i] = z_score > self.z_score_threshold;
        }
        
        Ok(outliers)
    }

    /// Interquartile Range (IQR) method for outlier detection
    /// Identifies data points outside Q1 - 1.5*IQR and Q3 + 1.5*IQR
    fn detect_iqr_outliers(&self, prices: &[u64]) -> Result<Vec<bool>> {
        let mut sorted_prices: Vec<u64> = prices.to_vec();
        sorted_prices.sort_unstable();
        
        let (q1, q3) = self.calculate_quartiles(&sorted_prices)?;
        let iqr = q3 - q1;
        
        let lower_bound = q1 - (1.5 * iqr);
        let upper_bound = q3 + (1.5 * iqr);
        
        let mut outliers = vec![false; prices.len()];
        
        for (i, &price) in prices.iter().enumerate() {
            let price_f64 = price as f64;
            outliers[i] = price_f64 < lower_bound || price_f64 > upper_bound;
        }
        
        Ok(outliers)
    }

    /// Modified Z-score using Median Absolute Deviation (MAD)
    /// More robust to outliers than standard Z-score
    fn detect_mad_outliers(&self, prices: &[u64]) -> Result<Vec<bool>> {
        let median = self.calculate_median(prices)?;
        
        // Calculate absolute deviations from median
        let mut abs_deviations: Vec<f64> = prices
            .iter()
            .map(|&price| ((price as f64) - median).abs())
            .collect();
        
        abs_deviations.sort_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal));
        let mad = self.calculate_median(&abs_deviations.iter().map(|&x| x as u64).collect::<Vec<_>>())?;
        
        if mad == 0.0 {
            // All values are the same, no outliers
            return Ok(vec![false; prices.len()]);
        }
        
        let mut outliers = vec![false; prices.len()];
        
        for (i, &price) in prices.iter().enumerate() {
            let modified_z_score = 0.6745 * ((price as f64) - median).abs() / mad;
            outliers[i] = modified_z_score > 3.5; // Standard threshold for modified Z-score
        }
        
        Ok(outliers)
    }

    /// Percentage deviation from median outlier detection
    /// Identifies prices that deviate more than max_deviation_percent from median
    fn detect_deviation_outliers(&self, prices: &[u64]) -> Result<Vec<bool>> {
        let median = self.calculate_median(prices)?;
        let mut outliers = vec![false; prices.len()];
        
        for (i, &price) in prices.iter().enumerate() {
            let deviation_percent = ((price as f64) - median).abs() / median * 100.0;
            outliers[i] = deviation_percent > self.max_deviation_percent;
        }
        
        Ok(outliers)
    }

    /// Calculate mean of price array
    fn calculate_mean(&self, prices: &[u64]) -> Result<f64> {
        require!(!prices.is_empty(), crate::errors::ErrorCode::EmptyPriceArray);
        
        let sum: u128 = prices.iter().map(|&x| x as u128).sum();
        Ok((sum as f64) / (prices.len() as f64))
    }

    /// Calculate standard deviation
    fn calculate_standard_deviation(&self, prices: &[u64], mean: f64) -> Result<f64> {
        require!(!prices.is_empty(), crate::errors::ErrorCode::EmptyPriceArray);
        
        let variance = prices
            .iter()
            .map(|&price| {
                let diff = (price as f64) - mean;
                diff * diff
            })
            .sum::<f64>() / (prices.len() as f64);
        
        Ok(variance.sqrt())
    }

    /// Calculate median of price array
    fn calculate_median(&self, prices: &[u64]) -> Result<f64> {
        require!(!prices.is_empty(), crate::errors::ErrorCode::EmptyPriceArray);
        
        let mut sorted_prices = prices.to_vec();
        sorted_prices.sort_unstable();
        
        let len = sorted_prices.len();
        if len % 2 == 0 {
            let mid1 = sorted_prices[len / 2 - 1] as f64;
            let mid2 = sorted_prices[len / 2] as f64;
            Ok((mid1 + mid2) / 2.0)
        } else {
            Ok(sorted_prices[len / 2] as f64)
        }
    }

    /// Calculate first and third quartiles
    fn calculate_quartiles(&self, sorted_prices: &[u64]) -> Result<(f64, f64)> {
        require!(!sorted_prices.is_empty(), crate::errors::ErrorCode::EmptyPriceArray);
        
        let len = sorted_prices.len();
        
        let q1_index = len / 4;
        let q3_index = 3 * len / 4;
        
        let q1 = if len % 4 == 0 {
            (sorted_prices[q1_index - 1] as f64 + sorted_prices[q1_index] as f64) / 2.0
        } else {
            sorted_prices[q1_index] as f64
        };
        
        let q3 = if len % 4 == 0 {
            (sorted_prices[q3_index - 1] as f64 + sorted_prices[q3_index] as f64) / 2.0
        } else {
            sorted_prices[q3_index] as f64
        };
        
        Ok((q1, q3))
    }

    /// Remove outliers from price array and return cleaned data
    pub fn remove_outliers(&self, prices: &[u64]) -> Result<Vec<u64>> {
        let outlier_flags = self.detect_outliers(prices)?;
        
        let cleaned_prices: Vec<u64> = prices
            .iter()
            .zip(outlier_flags.iter())
            .filter_map(|(&price, &is_outlier)| {
                if !is_outlier {
                    Some(price)
                } else {
                    None
                }
            })
            .collect();
        
        require!(!cleaned_prices.is_empty(), crate::errors::ErrorCode::AllPricesAreOutliers);
        
        Ok(cleaned_prices)
    }

    /// Get outlier statistics for reporting
    pub fn get_outlier_statistics(&self, prices: &[u64]) -> Result<OutlierStatistics> {
        let outlier_flags = self.detect_outliers(prices)?;
        let outlier_count = outlier_flags.iter().filter(|&&x| x).count();
        let outlier_percentage = (outlier_count as f64) / (prices.len() as f64) * 100.0;
        
        let outlier_prices: Vec<u64> = prices
            .iter()
            .zip(outlier_flags.iter())
            .filter_map(|(&price, &is_outlier)| {
                if is_outlier {
                    Some(price)
                } else {
                    None
                }
            })
            .collect();
        
        let mean = self.calculate_mean(prices)?;
        let median = self.calculate_median(prices)?;
        let std_dev = self.calculate_standard_deviation(prices, mean)?;
        
        Ok(OutlierStatistics {
            total_points: prices.len(),
            outlier_count,
            outlier_percentage,
            outlier_prices,
            mean,
            median,
            standard_deviation: std_dev,
            outlier_flags,
        })
    }

    /// Dynamic threshold adjustment based on market volatility
    pub fn adjust_thresholds_for_volatility(&mut self, historical_volatility: f64) -> Result<()> {
        require!(historical_volatility >= 0.0, crate::errors::ErrorCode::InvalidVolatility);
        
        // Adjust Z-score threshold based on volatility
        // Higher volatility = more lenient outlier detection
        self.z_score_threshold = 2.5 + (historical_volatility * self.volatility_factor);
        
        // Cap the threshold to prevent it from becoming too lenient
        self.z_score_threshold = self.z_score_threshold.min(5.0);
        
        // Adjust deviation percentage threshold
        self.max_deviation_percent = 20.0 + (historical_volatility * 10.0);
        self.max_deviation_percent = self.max_deviation_percent.min(50.0);
        
        Ok(())
    }

    /// Confidence score for data quality (0.0 to 1.0)
    pub fn calculate_data_quality_score(&self, prices: &[u64]) -> Result<f64> {
        if prices.len() < self.min_data_points {
            return Ok(0.5); // Neutral score for insufficient data
        }
        
        let outlier_flags = self.detect_outliers(prices)?;
        let outlier_count = outlier_flags.iter().filter(|&&x| x).count();
        let outlier_ratio = outlier_count as f64 / prices.len() as f64;
        
        // Calculate coefficient of variation (CV) = std_dev / mean
        let mean = self.calculate_mean(prices)?;
        let std_dev = self.calculate_standard_deviation(prices, mean)?;
        let cv = if mean > 0.0 { std_dev / mean } else { 0.0 };
        
        // Quality score based on outlier ratio and coefficient of variation
        let outlier_score = 1.0 - (outlier_ratio * 2.0).min(1.0);
        let stability_score = 1.0 - (cv * 2.0).min(1.0);
        let data_size_score = (prices.len() as f64 / 20.0).min(1.0); // Bonus for more data points
        
        let quality_score = (outlier_score * 0.4 + stability_score * 0.4 + data_size_score * 0.2)
            .max(0.0)
            .min(1.0);
        
        Ok(quality_score)
    }

    /// Detect price manipulation patterns
    pub fn detect_manipulation_patterns(&self, prices: &[u64], timestamps: &[i64]) -> Result<Vec<ManipulationAlert>> {
        require!(prices.len() == timestamps.len(), crate::errors::ErrorCode::MismatchedArrayLengths);
        require!(!prices.is_empty(), crate::errors::ErrorCode::EmptyPriceArray);
        
        let mut alerts = Vec::new();
        
        // Pattern 1: Sudden spike followed by immediate reversion
        for i in 1..prices.len().saturating_sub(1) {
            let prev_price = prices[i - 1] as f64;
            let curr_price = prices[i] as f64;
            let next_price = prices[i + 1] as f64;
            
            let spike_up = (curr_price - prev_price) / prev_price > 0.15; // 15% spike
            let revert_down = (curr_price - next_price) / curr_price > 0.12; // 12% reversion
            
            if spike_up && revert_down {
                alerts.push(ManipulationAlert {
                    alert_type: ManipulationType::SpikeAndRevert,
                    timestamp: timestamps[i],
                    price: prices[i],
                    severity: AlertSeverity::High,
                    description: "Sudden price spike followed by immediate reversion detected".to_string(),
                });
            }
        }
        
        // Pattern 2: Unusual price clustering at round numbers
        let round_number_clustering = self.detect_round_number_clustering(prices)?;
        if round_number_clustering > 0.3 {
            alerts.push(ManipulationAlert {
                alert_type: ManipulationType::RoundNumberClustering,
                timestamp: timestamps[timestamps.len() - 1],
                price: prices[prices.len() - 1],
                severity: AlertSeverity::Medium,
                description: format!("Unusual clustering at round numbers: {:.1}%", round_number_clustering * 100.0),
            });
        }
        
        // Pattern 3: Identical consecutive prices (potential stale data or manipulation)
        let consecutive_identical = self.count_consecutive_identical_prices(prices);
        if consecutive_identical > prices.len() / 3 {
            alerts.push(ManipulationAlert {
                alert_type: ManipulationType::StaleData,
                timestamp: timestamps[timestamps.len() - 1],
                price: prices[prices.len() - 1],
                severity: AlertSeverity::Medium,
                description: format!("Too many identical consecutive prices: {}", consecutive_identical),
            });
        }
        
        Ok(alerts)
    }

    /// Detect clustering around round numbers (potential manipulation indicator)
    fn detect_round_number_clustering(&self, prices: &[u64]) -> Result<f64> {
        let round_number_count = prices
            .iter()
            .filter(|&&price| {
                // Check if price is close to round numbers (multiples of 100, 1000, etc.)
                let price_f64 = price as f64;
                let remainder_100 = price % 100;
                let remainder_1000 = price % 1000;
                
                remainder_100 == 0 || remainder_100 == 99 || remainder_100 == 1 ||
                remainder_1000 == 0 || remainder_1000 == 999 || remainder_1000 == 1
            })
            .count();
        
        Ok(round_number_count as f64 / prices.len() as f64)
    }

    /// Count consecutive identical prices
    fn count_consecutive_identical_prices(&self, prices: &[u64]) -> usize {
        let mut max_consecutive = 0;
        let mut current_consecutive = 0;
        let mut last_price = None;
        
        for &price in prices {
            if Some(price) == last_price {
                current_consecutive += 1;
                max_consecutive = max_consecutive.max(current_consecutive);
            } else {
                current_consecutive = 1;
            }
            last_price = Some(price);
        }
        
        max_consecutive
    }
}

/// Statistics about outlier detection results
#[derive(Clone, Debug)]
pub struct OutlierStatistics {
    pub total_points: usize,
    pub outlier_count: usize,
    pub outlier_percentage: f64,
    pub outlier_prices: Vec<u64>,
    pub mean: f64,
    pub median: f64,
    pub standard_deviation: f64,
    pub outlier_flags: Vec<bool>,
}

/// Manipulation alert information
#[derive(Clone, Debug)]
pub struct ManipulationAlert {
    pub alert_type: ManipulationType,
    pub timestamp: i64,
    pub price: u64,
    pub severity: AlertSeverity,
    pub description: String,
}

/// Types of manipulation patterns
#[derive(Clone, Debug, PartialEq)]
pub enum ManipulationType {
    SpikeAndRevert,
    RoundNumberClustering,
    StaleData,
    UnusualVolatility,
    PriceSupression,
    PriceInflation,
}

/// Alert severity levels
#[derive(Clone, Debug, PartialEq)]
pub enum AlertSeverity {
    Low,
    Medium,
    High,
    Critical,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_z_score_outlier_detection() {
        let detector = OutlierDetector::default();
        let prices = vec![100, 102, 98, 101, 99, 150, 97]; // 150 is an outlier
        
        let outliers = detector.detect_z_score_outliers(&prices).unwrap();
        assert!(outliers[5]); // 150 should be detected as outlier
        assert!(!outliers[0]); // 100 should not be an outlier
    }

    #[test]
    fn test_iqr_outlier_detection() {
        let detector = OutlierDetector::default();
        let prices = vec![100, 102, 98, 101, 99, 200, 97]; // 200 is an outlier
        
        let outliers = detector.detect_iqr_outliers(&prices).unwrap();
        assert!(outliers[5]); // 200 should be detected as outlier
    }

    #[test]
    fn test_remove_outliers() {
        let detector = OutlierDetector::default();
        let prices = vec![100, 102, 98, 101, 99, 200, 97];
        
        let cleaned = detector.remove_outliers(&prices).unwrap();
        assert!(!cleaned.contains(&200));
        assert!(cleaned.len() < prices.len());
    }

    #[test]
    fn test_data_quality_score() {
        let detector = OutlierDetector::default();
        
        // Good data
        let good_prices = vec![100, 101, 99, 102, 98, 103, 97];
        let good_score = detector.calculate_data_quality_score(&good_prices).unwrap();
        
        // Bad data with outliers
        let bad_prices = vec![100, 200, 50, 300, 25, 400, 10];
        let bad_score = detector.calculate_data_quality_score(&bad_prices).unwrap();
        
        assert!(good_score > bad_score);
    }

    #[test]
    fn test_manipulation_detection() {
        let detector = OutlierDetector::default();
        let prices = vec![100, 150, 95]; // Spike and revert pattern
        let timestamps = vec![1000, 2000, 3000];
        
        let alerts = detector.detect_manipulation_patterns(&prices, &timestamps).unwrap();
        assert!(!alerts.is_empty());
        assert_eq!(alerts[0].alert_type, ManipulationType::SpikeAndRevert);
    }
}


