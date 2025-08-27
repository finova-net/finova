// programs/finova-oracle/src/state/price_feed.rs

use anchor_lang::prelude::*;
use std::collections::VecDeque;

/// Price feed account storing historical price data and metadata
#[account]
#[derive(Debug)]
pub struct PriceFeed {
    /// Authority that can update this price feed
    pub authority: Pubkey,
    
    /// Token mint address for which this price feed provides data
    pub token_mint: Pubkey,
    
    /// Current price in lamports (scaled by 1e9 for precision)
    pub current_price: u64,
    
    /// Previous price for comparison
    pub previous_price: u64,
    
    /// Timestamp of the last price update
    pub last_updated: i64,
    
    /// Confidence interval of the current price (in basis points)
    pub confidence: u16,
    
    /// Number of sources that contributed to this price
    pub num_sources: u8,
    
    /// Price deviation threshold for triggering alerts (in basis points)
    pub deviation_threshold: u16,
    
    /// Maximum staleness allowed before price is considered invalid (seconds)
    pub max_staleness: u32,
    
    /// Aggregation method used (0=median, 1=mean, 2=weighted)
    pub aggregation_method: u8,
    
    /// Circuit breaker status (0=normal, 1=warning, 2=halted)
    pub circuit_breaker_status: u8,
    
    /// Historical prices (last 100 data points)
    pub price_history: Vec<PricePoint>,
    
    /// Rolling statistics for analysis
    pub statistics: PriceStatistics,
    
    /// Oracle sources configuration
    pub sources: Vec<OracleSource>,
    
    /// Emergency pause flag
    pub is_paused: bool,
    
    /// Version for upgrades
    pub version: u8,
    
    /// Reserved space for future upgrades
    pub reserved: [u8; 64],
}

/// Individual price point with timestamp
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub struct PricePoint {
    /// Price value in lamports (scaled)
    pub price: u64,
    
    /// Unix timestamp when this price was recorded
    pub timestamp: i64,
    
    /// Confidence level of this price point
    pub confidence: u16,
    
    /// Volume traded at this price (if available)
    pub volume: u64,
}

/// Rolling price statistics for analysis
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, Default)]
pub struct PriceStatistics {
    /// 24-hour moving average
    pub moving_average_24h: u64,
    
    /// 7-day moving average
    pub moving_average_7d: u64,
    
    /// 30-day moving average
    pub moving_average_30d: u64,
    
    /// Current volatility measure (standard deviation)
    pub volatility: u32,
    
    /// Highest price in last 24 hours
    pub high_24h: u64,
    
    /// Lowest price in last 24 hours
    pub low_24h: u64,
    
    /// Price change percentage in last 24 hours (basis points)
    pub change_24h: i32,
    
    /// Total volume in last 24 hours
    pub volume_24h: u64,
    
    /// Number of updates in last 24 hours
    pub update_count_24h: u32,
    
    /// Last calculation timestamp
    pub last_calculated: i64,
}

/// Oracle source configuration
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct OracleSource {
    /// Source identifier (e.g., "pyth", "chainlink", "switchboard")
    pub name: String,
    
    /// Source authority/program address
    pub authority: Pubkey,
    
    /// Weight of this source in aggregation (0-100)
    pub weight: u8,
    
    /// Is this source currently active
    pub is_active: bool,
    
    /// Last price received from this source
    pub last_price: u64,
    
    /// Last update timestamp from this source
    pub last_updated: i64,
    
    /// Reliability score (0-100)
    pub reliability_score: u8,
    
    /// Number of consecutive failures
    pub failure_count: u8,
    
    /// Maximum allowed staleness for this source
    pub max_staleness: u32,
}

/// Aggregation configuration for combining multiple price sources
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct AggregationConfig {
    /// Minimum number of sources required for valid aggregation
    pub min_sources: u8,
    
    /// Maximum deviation allowed between sources (basis points)
    pub max_source_deviation: u16,
    
    /// Outlier detection enabled
    pub outlier_detection: bool,
    
    /// Outlier threshold (standard deviations)
    pub outlier_threshold: u16,
    
    /// Time window for aggregation (seconds)
    pub aggregation_window: u32,
    
    /// Weighted aggregation based on reliability scores
    pub use_weighted_aggregation: bool,
}

/// Price alert configuration
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct AlertConfig {
    /// Price change threshold for alerts (basis points)
    pub price_change_threshold: u16,
    
    /// Volatility threshold for alerts
    pub volatility_threshold: u32,
    
    /// Staleness threshold for alerts (seconds)
    pub staleness_threshold: u32,
    
    /// Confidence threshold below which to alert
    pub confidence_threshold: u16,
    
    /// Alert cooldown period (seconds)
    pub alert_cooldown: u32,
    
    /// Last alert timestamp
    pub last_alert: i64,
}

impl PriceFeed {
    /// Size calculation for account allocation
    pub const LEN: usize = 8 + // discriminator
        32 + // authority
        32 + // token_mint
        8 + // current_price
        8 + // previous_price
        8 + // last_updated
        2 + // confidence
        1 + // num_sources
        2 + // deviation_threshold
        4 + // max_staleness
        1 + // aggregation_method
        1 + // circuit_breaker_status
        4 + (100 * 32) + // price_history (max 100 points)
        128 + // statistics
        4 + (10 * 256) + // sources (max 10 sources)
        1 + // is_paused
        1 + // version
        64; // reserved
    
    /// Initialize a new price feed
    pub fn initialize(
        &mut self,
        authority: Pubkey,
        token_mint: Pubkey,
        deviation_threshold: u16,
        max_staleness: u32,
        aggregation_method: u8,
    ) -> Result<()> {
        self.authority = authority;
        self.token_mint = token_mint;
        self.current_price = 0;
        self.previous_price = 0;
        self.last_updated = Clock::get()?.unix_timestamp;
        self.confidence = 0;
        self.num_sources = 0;
        self.deviation_threshold = deviation_threshold;
        self.max_staleness = max_staleness;
        self.aggregation_method = aggregation_method;
        self.circuit_breaker_status = 0;
        self.price_history = Vec::new();
        self.statistics = PriceStatistics::default();
        self.sources = Vec::new();
        self.is_paused = false;
        self.version = 1;
        self.reserved = [0; 64];
        
        Ok(())
    }
    
    /// Update price with new data point
    pub fn update_price(
        &mut self,
        new_price: u64,
        confidence: u16,
        num_sources: u8,
        volume: u64,
    ) -> Result<()> {
        require!(!self.is_paused, OracleError::PriceFeedPaused);
        require!(
            self.circuit_breaker_status != 2,
            OracleError::CircuitBreakerTriggered
        );
        
        let current_timestamp = Clock::get()?.unix_timestamp;
        
        // Store previous price
        self.previous_price = self.current_price;
        
        // Check for circuit breaker conditions
        if self.current_price > 0 {
            let price_change = if new_price > self.current_price {
                ((new_price - self.current_price) * 10000) / self.current_price
            } else {
                ((self.current_price - new_price) * 10000) / self.current_price
            };
            
            if price_change > self.deviation_threshold as u64 {
                self.circuit_breaker_status = 1; // Warning
                
                // If change is too extreme, halt updates
                if price_change > (self.deviation_threshold as u64 * 2) {
                    self.circuit_breaker_status = 2; // Halted
                    return Err(OracleError::PriceDeviationTooHigh.into());
                }
            }
        }
        
        // Update current price
        self.current_price = new_price;
        self.last_updated = current_timestamp;
        self.confidence = confidence;
        self.num_sources = num_sources;
        
        // Add to price history
        let price_point = PricePoint {
            price: new_price,
            timestamp: current_timestamp,
            confidence,
            volume,
        };
        
        self.add_price_point(price_point);
        
        // Update statistics
        self.update_statistics()?;
        
        Ok(())
    }
    
    /// Add price point to history
    fn add_price_point(&mut self, price_point: PricePoint) {
        // Maintain maximum of 100 historical points
        if self.price_history.len() >= 100 {
            self.price_history.remove(0);
        }
        
        self.price_history.push(price_point);
    }
    
    /// Update rolling statistics
    fn update_statistics(&mut self) -> Result<()> {
        let current_time = Clock::get()?.unix_timestamp;
        
        if self.price_history.is_empty() {
            return Ok(());
        }
        
        // Calculate time windows
        let day_ago = current_time - 86400; // 24 hours
        let week_ago = current_time - 604800; // 7 days
        let month_ago = current_time - 2592000; // 30 days
        
        // Filter prices by time windows
        let prices_24h: Vec<&PricePoint> = self.price_history
            .iter()
            .filter(|p| p.timestamp >= day_ago)
            .collect();
            
        let prices_7d: Vec<&PricePoint> = self.price_history
            .iter()
            .filter(|p| p.timestamp >= week_ago)
            .collect();
            
        let prices_30d: Vec<&PricePoint> = self.price_history
            .iter()
            .filter(|p| p.timestamp >= month_ago)
            .collect();
        
        // Calculate moving averages
        self.statistics.moving_average_24h = self.calculate_average(&prices_24h);
        self.statistics.moving_average_7d = self.calculate_average(&prices_7d);
        self.statistics.moving_average_30d = self.calculate_average(&prices_30d);
        
        // Calculate 24h statistics
        if !prices_24h.is_empty() {
            let prices: Vec<u64> = prices_24h.iter().map(|p| p.price).collect();
            
            self.statistics.high_24h = *prices.iter().max().unwrap_or(&0);
            self.statistics.low_24h = *prices.iter().min().unwrap_or(&0);
            self.statistics.volume_24h = prices_24h.iter().map(|p| p.volume).sum();
            self.statistics.update_count_24h = prices_24h.len() as u32;
            
            // Calculate price change
            if let Some(oldest) = prices_24h.first() {
                if oldest.price > 0 {
                    let change = if self.current_price > oldest.price {
                        ((self.current_price - oldest.price) * 10000) / oldest.price
                    } else {
                        -(((oldest.price - self.current_price) * 10000) / oldest.price) as i64
                    };
                    self.statistics.change_24h = change as i32;
                }
            }
            
            // Calculate volatility (simplified standard deviation)
            self.statistics.volatility = self.calculate_volatility(&prices);
        }
        
        self.statistics.last_calculated = current_time;
        
        Ok(())
    }
    
    /// Calculate average price from price points
    fn calculate_average(&self, prices: &[&PricePoint]) -> u64 {
        if prices.is_empty() {
            return 0;
        }
        
        let sum: u64 = prices.iter().map(|p| p.price).sum();
        sum / prices.len() as u64
    }
    
    /// Calculate price volatility (simplified)
    fn calculate_volatility(&self, prices: &[u64]) -> u32 {
        if prices.len() < 2 {
            return 0;
        }
        
        let mean = prices.iter().sum::<u64>() / prices.len() as u64;
        let variance: u64 = prices
            .iter()
            .map(|&price| {
                let diff = if price > mean { price - mean } else { mean - price };
                diff * diff
            })
            .sum::<u64>() / prices.len() as u64;
        
        // Return square root approximation
        (variance as f64).sqrt() as u32
    }
    
    /// Add or update oracle source
    pub fn add_source(
        &mut self,
        name: String,
        authority: Pubkey,
        weight: u8,
        max_staleness: u32,
    ) -> Result<()> {
        require!(self.sources.len() < 10, OracleError::TooManySources);
        require!(weight <= 100, OracleError::InvalidWeight);
        
        // Check if source already exists
        if let Some(existing) = self.sources.iter_mut().find(|s| s.name == name) {
            existing.authority = authority;
            existing.weight = weight;
            existing.max_staleness = max_staleness;
            existing.is_active = true;
        } else {
            let source = OracleSource {
                name,
                authority,
                weight,
                is_active: true,
                last_price: 0,
                last_updated: 0,
                reliability_score: 100,
                failure_count: 0,
                max_staleness,
            };
            
            self.sources.push(source);
        }
        
        Ok(())
    }
    
    /// Update source reliability score
    pub fn update_source_reliability(
        &mut self,
        source_name: &str,
        success: bool,
    ) -> Result<()> {
        if let Some(source) = self.sources.iter_mut().find(|s| s.name == source_name) {
            if success {
                source.failure_count = 0;
                if source.reliability_score < 100 {
                    source.reliability_score = std::cmp::min(100, source.reliability_score + 1);
                }
            } else {
                source.failure_count += 1;
                source.reliability_score = source.reliability_score.saturating_sub(5);
                
                // Deactivate source if too many failures
                if source.failure_count >= 5 || source.reliability_score < 20 {
                    source.is_active = false;
                }
            }
        }
        
        Ok(())
    }
    
    /// Check if price feed is stale
    pub fn is_stale(&self) -> Result<bool> {
        let current_time = Clock::get()?.unix_timestamp;
        Ok(current_time - self.last_updated > self.max_staleness as i64)
    }
    
    /// Get price with confidence check
    pub fn get_price_with_confidence(&self) -> Result<(u64, u16)> {
        require!(!self.is_stale()?, OracleError::StalePriceFeed);
        require!(!self.is_paused, OracleError::PriceFeedPaused);
        require!(
            self.circuit_breaker_status != 2,
            OracleError::CircuitBreakerTriggered
        );
        
        Ok((self.current_price, self.confidence))
    }
    
    /// Aggregate prices from multiple sources
    pub fn aggregate_prices(&self, source_prices: &[(String, u64, u16, i64)]) -> Result<(u64, u16)> {
        let current_time = Clock::get()?.unix_timestamp;
        let mut valid_prices = Vec::new();
        let mut total_weight = 0u16;
        let mut weighted_sum = 0u128;
        let mut confidence_sum = 0u32;
        
        // Filter valid prices from active sources
        for (source_name, price, confidence, timestamp) in source_prices {
            if let Some(source) = self.sources.iter().find(|s| s.name == *source_name && s.is_active) {
                // Check staleness
                if current_time - timestamp <= source.max_staleness as i64 {
                    valid_prices.push((*price, *confidence, source.weight));
                    
                    if self.aggregation_method == 2 { // Weighted aggregation
                        weighted_sum += (*price as u128) * (source.weight as u128);
                        total_weight += source.weight as u16;
                    }
                    
                    confidence_sum += *confidence as u32;
                }
            }
        }
        
        require!(!valid_prices.is_empty(), OracleError::NoValidSources);
        
        let aggregated_price = match self.aggregation_method {
            0 => self.calculate_median(&valid_prices), // Median
            1 => self.calculate_mean(&valid_prices),   // Mean
            2 => {                                     // Weighted
                if total_weight > 0 {
                    (weighted_sum / total_weight as u128) as u64
                } else {
                    self.calculate_mean(&valid_prices)
                }
            }
            _ => return Err(OracleError::InvalidAggregationMethod.into()),
        };
        
        let average_confidence = (confidence_sum / valid_prices.len() as u32) as u16;
        
        Ok((aggregated_price, average_confidence))
    }
    
    /// Calculate median price
    fn calculate_median(&self, prices: &[(u64, u16, u8)]) -> u64 {
        let mut sorted_prices: Vec<u64> = prices.iter().map(|(p, _, _)| *p).collect();
        sorted_prices.sort_unstable();
        
        let len = sorted_prices.len();
        if len % 2 == 0 {
            (sorted_prices[len / 2 - 1] + sorted_prices[len / 2]) / 2
        } else {
            sorted_prices[len / 2]
        }
    }
    
    /// Calculate mean price
    fn calculate_mean(&self, prices: &[(u64, u16, u8)]) -> u64 {
        let sum: u64 = prices.iter().map(|(p, _, _)| *p).sum();
        sum / prices.len() as u64
    }
    
    /// Emergency pause function
    pub fn emergency_pause(&mut self, pause: bool) -> Result<()> {
        self.is_paused = pause;
        Ok(())
    }
    
    /// Reset circuit breaker
    pub fn reset_circuit_breaker(&mut self) -> Result<()> {
        self.circuit_breaker_status = 0;
        Ok(())
    }
    
    /// Validate price feed configuration
    pub fn validate_config(&self) -> Result<()> {
        require!(self.deviation_threshold > 0, OracleError::InvalidConfiguration);
        require!(self.max_staleness > 0, OracleError::InvalidConfiguration);
        require!(self.aggregation_method <= 2, OracleError::InvalidAggregationMethod);
        
        // Validate sources
        let total_weight: u16 = self.sources.iter()
            .filter(|s| s.is_active)
            .map(|s| s.weight as u16)
            .sum();
            
        require!(total_weight > 0, OracleError::NoActiveSources);
        require!(total_weight <= 1000, OracleError::InvalidTotalWeight); // Allow over 100% for redundancy
        
        Ok(())
    }
}

/// Oracle-specific error codes
#[error_code]
pub enum OracleError {
    #[msg("Price feed is currently paused")]
    PriceFeedPaused,
    
    #[msg("Circuit breaker has been triggered")]
    CircuitBreakerTriggered,
    
    #[msg("Price deviation exceeds threshold")]
    PriceDeviationTooHigh,
    
    #[msg("Price feed is stale")]
    StalePriceFeed,
    
    #[msg("Too many oracle sources")]
    TooManySources,
    
    #[msg("Invalid weight value")]
    InvalidWeight,
    
    #[msg("No valid price sources available")]
    NoValidSources,
    
    #[msg("Invalid aggregation method")]
    InvalidAggregationMethod,
    
    #[msg("Invalid configuration")]
    InvalidConfiguration,
    
    #[msg("No active sources")]
    NoActiveSources,
    
    #[msg("Invalid total weight")]
    InvalidTotalWeight,
}

impl Default for PriceFeed {
    fn default() -> Self {
        Self {
            authority: Pubkey::default(),
            token_mint: Pubkey::default(),
            current_price: 0,
            previous_price: 0,
            last_updated: 0,
            confidence: 0,
            num_sources: 0,
            deviation_threshold: 1000, // 10%
            max_staleness: 300,        // 5 minutes
            aggregation_method: 1,     // Mean
            circuit_breaker_status: 0,
            price_history: Vec::new(),
            statistics: PriceStatistics::default(),
            sources: Vec::new(),
            is_paused: false,
            version: 1,
            reserved: [0; 64],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_price_feed_initialization() {
        let mut price_feed = PriceFeed::default();
        let authority = Pubkey::new_unique();
        let token_mint = Pubkey::new_unique();
        
        price_feed.initialize(
            authority,
            token_mint,
            1000, // 10% deviation threshold
            300,  // 5 minutes staleness
            1,    // Mean aggregation
        ).unwrap();
        
        assert_eq!(price_feed.authority, authority);
        assert_eq!(price_feed.token_mint, token_mint);
        assert_eq!(price_feed.deviation_threshold, 1000);
        assert_eq!(price_feed.max_staleness, 300);
        assert_eq!(price_feed.aggregation_method, 1);
    }
    
    #[test]
    fn test_calculate_median() {
        let price_feed = PriceFeed::default();
        let prices = vec![
            (100, 95, 20),
            (105, 98, 25),
            (95, 90, 15),
            (110, 99, 30),
            (102, 96, 22),
        ];
        
        let median = price_feed.calculate_median(&prices);
        assert_eq!(median, 102); // Middle value when sorted: [95, 100, 102, 105, 110]
    }
    
    #[test]
    fn test_calculate_mean() {
        let price_feed = PriceFeed::default();
        let prices = vec![
            (100, 95, 20),
            (200, 98, 25),
        ];
        
        let mean = price_feed.calculate_mean(&prices);
        assert_eq!(mean, 150); // (100 + 200) / 2
    }
    
    #[test]
    fn test_add_source() {
        let mut price_feed = PriceFeed::default();
        
        price_feed.add_source(
            "pyth".to_string(),
            Pubkey::new_unique(),
            50,
            300,
        ).unwrap();
        
        assert_eq!(price_feed.sources.len(), 1);
        assert_eq!(price_feed.sources[0].name, "pyth");
        assert_eq!(price_feed.sources[0].weight, 50);
        assert!(price_feed.sources[0].is_active);
    }
}
