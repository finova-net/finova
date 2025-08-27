// programs/finova-oracle/src/state/aggregator.rs

use anchor_lang::prelude::*;
use std::collections::BTreeMap;

/// Aggregator state for combining multiple price feeds
#[account]
#[derive(Debug)]
pub struct Aggregator {
    /// Authority that can update aggregator configuration
    pub authority: Pubkey,
    
    /// Unique identifier for this aggregator
    pub aggregator_id: u64,
    
    /// Description of what this aggregator tracks
    pub description: String,
    
    /// Number of decimal places for the aggregated price
    pub decimals: u8,
    
    /// Minimum number of valid feeds required for aggregation
    pub min_feeds_required: u8,
    
    /// Maximum allowed deviation between feeds (in basis points)
    pub max_deviation_bps: u16,
    
    /// Current aggregated price
    pub current_price: u64,
    
    /// Confidence level of the current price (0-10000 basis points)
    pub confidence: u16,
    
    /// Timestamp of last price update
    pub last_updated: i64,
    
    /// Number of feeds currently contributing to aggregation
    pub active_feeds_count: u8,
    
    /// Maximum number of feeds this aggregator can handle
    pub max_feeds: u8,
    
    /// List of price feed accounts this aggregator monitors
    pub price_feeds: Vec<Pubkey>,
    
    /// Weights for each price feed (index corresponds to price_feeds)
    pub feed_weights: Vec<u16>,
    
    /// Individual feed values and metadata
    pub feed_data: Vec<FeedData>,
    
    /// Historical price data for trend analysis
    pub price_history: CircularBuffer<PricePoint>,
    
    /// Aggregation method used
    pub aggregation_method: AggregationMethod,
    
    /// Status of the aggregator
    pub status: AggregatorStatus,
    
    /// Emergency circuit breaker settings
    pub circuit_breaker: CircuitBreaker,
    
    /// Statistics for monitoring
    pub stats: AggregatorStats,
    
    /// Reserved space for future upgrades
    pub reserved: [u8; 256],
}

/// Individual feed data within aggregator
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub struct FeedData {
    /// Price from this feed
    pub price: u64,
    
    /// Timestamp of this feed's last update
    pub timestamp: i64,
    
    /// Confidence level of this feed
    pub confidence: u16,
    
    /// Whether this feed is currently active
    pub is_active: bool,
    
    /// Number of consecutive failed updates
    pub failure_count: u8,
    
    /// Deviation from median (basis points)
    pub deviation_bps: i16,
}

/// Historical price point
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub struct PricePoint {
    /// Price at this point
    pub price: u64,
    
    /// Timestamp
    pub timestamp: i64,
    
    /// Confidence level
    pub confidence: u16,
    
    /// Volume or activity indicator
    pub volume: u64,
}

/// Circular buffer for price history
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub struct CircularBuffer<T> {
    /// Buffer data
    pub data: Vec<T>,
    
    /// Current write position
    pub head: usize,
    
    /// Maximum capacity
    pub capacity: usize,
    
    /// Current size
    pub size: usize,
}

/// Aggregation methods
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum AggregationMethod {
    /// Simple arithmetic mean
    Mean,
    
    /// Weighted average based on feed weights
    WeightedMean,
    
    /// Median value (outlier resistant)
    Median,
    
    /// Volume-weighted average price
    VWAP,
    
    /// Time-weighted average price
    TWAP,
    
    /// Exponentially weighted moving average
    EWMA,
}

/// Aggregator status
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum AggregatorStatus {
    /// Operating normally
    Active,
    
    /// Temporarily paused
    Paused,
    
    /// Insufficient feeds for reliable aggregation
    Degraded,
    
    /// Circuit breaker triggered
    CircuitBroken,
    
    /// Emergency shutdown
    Emergency,
}

/// Circuit breaker configuration
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub struct CircuitBreaker {
    /// Maximum price change percentage before triggering
    pub max_price_change_bps: u16,
    
    /// Time window for price change calculation (seconds)
    pub time_window: u32,
    
    /// Whether circuit breaker is enabled
    pub enabled: bool,
    
    /// Timestamp when circuit breaker was triggered
    pub triggered_at: i64,
    
    /// Cool-down period before reset (seconds)
    pub cooldown_period: u32,
    
    /// Number of times circuit breaker has triggered
    pub trigger_count: u32,
}

/// Aggregator statistics
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub struct AggregatorStats {
    /// Total number of price updates
    pub total_updates: u64,
    
    /// Number of successful aggregations
    pub successful_aggregations: u64,
    
    /// Number of failed aggregations
    pub failed_aggregations: u64,
    
    /// Average confidence level over time
    pub avg_confidence: u16,
    
    /// Maximum observed price
    pub max_price: u64,
    
    /// Minimum observed price
    pub min_price: u64,
    
    /// Standard deviation of prices (scaled)
    pub price_volatility: u64,
    
    /// Average number of active feeds
    pub avg_active_feeds: u8,
    
    /// Uptime percentage (basis points)
    pub uptime_bps: u16,
    
    /// Last maintenance timestamp
    pub last_maintenance: i64,
}

impl Aggregator {
    /// Size calculation for account allocation
    pub const LEN: usize = 8 + // discriminator
        32 + // authority
        8 + // aggregator_id
        64 + // description (max length)
        1 + // decimals
        1 + // min_feeds_required
        2 + // max_deviation_bps
        8 + // current_price
        2 + // confidence
        8 + // last_updated
        1 + // active_feeds_count
        1 + // max_feeds
        4 + 32 * 50 + // price_feeds (max 50 feeds)
        4 + 2 * 50 + // feed_weights
        4 + std::mem::size_of::<FeedData>() * 50 + // feed_data
        std::mem::size_of::<CircularBuffer<PricePoint>>() + 
        std::mem::size_of::<PricePoint>() * 100 + // price_history (100 points)
        std::mem::size_of::<AggregationMethod>() +
        std::mem::size_of::<AggregatorStatus>() +
        std::mem::size_of::<CircuitBreaker>() +
        std::mem::size_of::<AggregatorStats>() +
        256; // reserved

    /// Initialize a new aggregator
    pub fn initialize(
        &mut self,
        authority: Pubkey,
        aggregator_id: u64,
        description: String,
        decimals: u8,
        min_feeds_required: u8,
        max_deviation_bps: u16,
        max_feeds: u8,
        aggregation_method: AggregationMethod,
    ) -> Result<()> {
        require!(max_feeds > 0 && max_feeds <= 50, crate::errors::OracleError::InvalidConfiguration);
        require!(min_feeds_required > 0 && min_feeds_required <= max_feeds, 
                crate::errors::OracleError::InvalidConfiguration);
        require!(description.len() <= 64, crate::errors::OracleError::InvalidConfiguration);

        self.authority = authority;
        self.aggregator_id = aggregator_id;
        self.description = description;
        self.decimals = decimals;
        self.min_feeds_required = min_feeds_required;
        self.max_deviation_bps = max_deviation_bps;
        self.max_feeds = max_feeds;
        self.aggregation_method = aggregation_method;
        self.status = AggregatorStatus::Active;
        
        // Initialize collections
        self.price_feeds = Vec::new();
        self.feed_weights = Vec::new();
        self.feed_data = Vec::new();
        
        // Initialize circular buffer for price history
        self.price_history = CircularBuffer {
            data: Vec::with_capacity(100),
            head: 0,
            capacity: 100,
            size: 0,
        };
        
        // Initialize circuit breaker
        self.circuit_breaker = CircuitBreaker {
            max_price_change_bps: 2000, // 20% default
            time_window: 300, // 5 minutes
            enabled: true,
            triggered_at: 0,
            cooldown_period: 3600, // 1 hour
            trigger_count: 0,
        };
        
        // Initialize stats
        self.stats = AggregatorStats {
            total_updates: 0,
            successful_aggregations: 0,
            failed_aggregations: 0,
            avg_confidence: 0,
            max_price: 0,
            min_price: u64::MAX,
            price_volatility: 0,
            avg_active_feeds: 0,
            uptime_bps: 10000, // 100%
            last_maintenance: Clock::get()?.unix_timestamp,
        };
        
        self.current_price = 0;
        self.confidence = 0;
        self.last_updated = 0;
        self.active_feeds_count = 0;
        self.reserved = [0; 256];

        Ok(())
    }

    /// Add a price feed to the aggregator
    pub fn add_price_feed(&mut self, feed_pubkey: Pubkey, weight: u16) -> Result<()> {
        require!(self.price_feeds.len() < self.max_feeds as usize, 
                crate::errors::OracleError::TooManyFeeds);
        require!(weight > 0, crate::errors::OracleError::InvalidWeight);
        require!(!self.price_feeds.contains(&feed_pubkey), 
                crate::errors::OracleError::FeedAlreadyExists);

        self.price_feeds.push(feed_pubkey);
        self.feed_weights.push(weight);
        self.feed_data.push(FeedData {
            price: 0,
            timestamp: 0,
            confidence: 0,
            is_active: false,
            failure_count: 0,
            deviation_bps: 0,
        });

        Ok(())
    }

    /// Remove a price feed from the aggregator
    pub fn remove_price_feed(&mut self, feed_pubkey: Pubkey) -> Result<()> {
        if let Some(index) = self.price_feeds.iter().position(|&x| x == feed_pubkey) {
            self.price_feeds.remove(index);
            self.feed_weights.remove(index);
            self.feed_data.remove(index);
            
            // Update active feeds count
            self.update_active_feeds_count();
            
            Ok(())
        } else {
            Err(crate::errors::OracleError::FeedNotFound.into())
        }
    }

    /// Update feed data and recalculate aggregated price
    pub fn update_feed_data(&mut self, feed_pubkey: Pubkey, price: u64, confidence: u16) -> Result<()> {
        let current_time = Clock::get()?.unix_timestamp;
        
        if let Some(index) = self.price_feeds.iter().position(|&x| x == feed_pubkey) {
            // Update feed data
            self.feed_data[index] = FeedData {
                price,
                timestamp: current_time,
                confidence,
                is_active: true,
                failure_count: 0,
                deviation_bps: 0,
            };
            
            // Recalculate aggregated price
            self.aggregate_price()?;
            
            // Update statistics
            self.stats.total_updates += 1;
            
            Ok(())
        } else {
            Err(crate::errors::OracleError::FeedNotFound.into())
        }
    }

    /// Aggregate price from all active feeds
    pub fn aggregate_price(&mut self) -> Result<()> {
        let current_time = Clock::get()?.unix_timestamp;
        
        // Filter active feeds
        let active_feeds: Vec<(usize, &FeedData, u16)> = self.feed_data
            .iter()
            .enumerate()
            .filter_map(|(i, feed)| {
                if feed.is_active && current_time - feed.timestamp <= 300 { // 5 minutes max age
                    Some((i, feed, self.feed_weights[i]))
                } else {
                    None
                }
            })
            .collect();

        self.active_feeds_count = active_feeds.len() as u8;

        // Check if we have enough feeds
        if self.active_feeds_count < self.min_feeds_required {
            self.status = AggregatorStatus::Degraded;
            self.stats.failed_aggregations += 1;
            return Err(crate::errors::OracleError::InsufficientFeeds.into());
        }

        // Calculate aggregated price based on method
        let (aggregated_price, aggregated_confidence) = match self.aggregation_method {
            AggregationMethod::Mean => self.calculate_mean(&active_feeds)?,
            AggregationMethod::WeightedMean => self.calculate_weighted_mean(&active_feeds)?,
            AggregationMethod::Median => self.calculate_median(&active_feeds)?,
            AggregationMethod::VWAP => self.calculate_vwap(&active_feeds)?,
            AggregationMethod::TWAP => self.calculate_twap(&active_feeds)?,
            AggregationMethod::EWMA => self.calculate_ewma(&active_feeds)?,
        };

        // Check circuit breaker
        if self.circuit_breaker.enabled {
            self.check_circuit_breaker(aggregated_price, current_time)?;
        }

        // Update current values
        let old_price = self.current_price;
        self.current_price = aggregated_price;
        self.confidence = aggregated_confidence;
        self.last_updated = current_time;

        // Add to price history
        self.add_to_history(PricePoint {
            price: aggregated_price,
            timestamp: current_time,
            confidence: aggregated_confidence,
            volume: 0, // Can be enhanced with volume data
        });

        // Update statistics
        self.update_statistics(old_price, aggregated_price);
        self.stats.successful_aggregations += 1;

        // Update status
        if self.status == AggregatorStatus::Degraded && self.active_feeds_count >= self.min_feeds_required {
            self.status = AggregatorStatus::Active;
        }

        Ok(())
    }

    /// Calculate simple arithmetic mean
    fn calculate_mean(&self, active_feeds: &[(usize, &FeedData, u16)]) -> Result<(u64, u16)> {
        let sum: u128 = active_feeds.iter().map(|(_, feed, _)| feed.price as u128).sum();
        let count = active_feeds.len() as u128;
        let mean_price = (sum / count) as u64;
        
        let confidence_sum: u32 = active_feeds.iter().map(|(_, feed, _)| feed.confidence as u32).sum();
        let mean_confidence = (confidence_sum / active_feeds.len() as u32) as u16;
        
        Ok((mean_price, mean_confidence))
    }

    /// Calculate weighted mean
    fn calculate_weighted_mean(&self, active_feeds: &[(usize, &FeedData, u16)]) -> Result<(u64, u16)> {
        let mut weighted_sum: u128 = 0;
        let mut total_weight: u128 = 0;
        let mut confidence_weighted_sum: u128 = 0;

        for (_, feed, weight) in active_feeds {
            let weight = *weight as u128;
            weighted_sum += feed.price as u128 * weight;
            confidence_weighted_sum += feed.confidence as u128 * weight;
            total_weight += weight;
        }

        require!(total_weight > 0, crate::errors::OracleError::InvalidWeight);

        let weighted_price = (weighted_sum / total_weight) as u64;
        let weighted_confidence = (confidence_weighted_sum / total_weight) as u16;

        Ok((weighted_price, weighted_confidence))
    }

    /// Calculate median price
    fn calculate_median(&self, active_feeds: &[(usize, &FeedData, u16)]) -> Result<(u64, u16)> {
        let mut prices: Vec<u64> = active_feeds.iter().map(|(_, feed, _)| feed.price).collect();
        prices.sort_unstable();

        let median_price = if prices.len() % 2 == 0 {
            let mid = prices.len() / 2;
            (prices[mid - 1] + prices[mid]) / 2
        } else {
            prices[prices.len() / 2]
        };

        // Calculate confidence as average
        let confidence_sum: u32 = active_feeds.iter().map(|(_, feed, _)| feed.confidence as u32).sum();
        let avg_confidence = (confidence_sum / active_feeds.len() as u32) as u16;

        Ok((median_price, avg_confidence))
    }

    /// Calculate Volume Weighted Average Price (placeholder)
    fn calculate_vwap(&self, active_feeds: &[(usize, &FeedData, u16)]) -> Result<(u64, u16)> {
        // For now, use weighted mean as VWAP placeholder
        // In real implementation, this would use volume data
        self.calculate_weighted_mean(active_feeds)
    }

    /// Calculate Time Weighted Average Price
    fn calculate_twap(&self, active_feeds: &[(usize, &FeedData, u16)]) -> Result<(u64, u16)> {
        let current_time = Clock::get()?.unix_timestamp;
        let mut time_weighted_sum: u128 = 0;
        let mut total_time_weight: u128 = 0;

        for (_, feed, _) in active_feeds {
            let time_weight = std::cmp::max(1, 301 - (current_time - feed.timestamp)) as u128;
            time_weighted_sum += feed.price as u128 * time_weight;
            total_time_weight += time_weight;
        }

        require!(total_time_weight > 0, crate::errors::OracleError::InvalidWeight);

        let twap_price = (time_weighted_sum / total_time_weight) as u64;
        
        // Calculate confidence
        let confidence_sum: u32 = active_feeds.iter().map(|(_, feed, _)| feed.confidence as u32).sum();
        let avg_confidence = (confidence_sum / active_feeds.len() as u32) as u16;

        Ok((twap_price, avg_confidence))
    }

    /// Calculate Exponentially Weighted Moving Average
    fn calculate_ewma(&self, active_feeds: &[(usize, &FeedData, u16)]) -> Result<(u64, u16)> {
        if self.current_price == 0 {
            // First calculation, use simple mean
            return self.calculate_mean(active_feeds);
        }

        // EWMA with alpha = 0.3
        let alpha = 30; // 30% in basis points
        let new_price = self.calculate_mean(active_feeds)?.0;
        
        let ewma_price = ((self.current_price as u128 * (10000 - alpha as u128) + 
                          new_price as u128 * alpha as u128) / 10000) as u64;

        let confidence_sum: u32 = active_feeds.iter().map(|(_, feed, _)| feed.confidence as u32).sum();
        let avg_confidence = (confidence_sum / active_feeds.len() as u32) as u16;

        Ok((ewma_price, avg_confidence))
    }

    /// Check circuit breaker conditions
    fn check_circuit_breaker(&mut self, new_price: u64, current_time: i64) -> Result<()> {
        if self.current_price == 0 {
            return Ok(()); // No previous price to compare
        }

        let price_change_bps = if new_price > self.current_price {
            ((new_price - self.current_price) as u128 * 10000 / self.current_price as u128) as u16
        } else {
            ((self.current_price - new_price) as u128 * 10000 / self.current_price as u128) as u16
        };

        if price_change_bps > self.circuit_breaker.max_price_change_bps {
            self.circuit_breaker.triggered_at = current_time;
            self.circuit_breaker.trigger_count += 1;
            self.status = AggregatorStatus::CircuitBroken;
            return Err(crate::errors::OracleError::CircuitBreakerTriggered.into());
        }

        Ok(())
    }

    /// Add price point to history
    fn add_to_history(&mut self, point: PricePoint) {
        if self.price_history.size < self.price_history.capacity {
            self.price_history.data.push(point);
            self.price_history.size += 1;
        } else {
            self.price_history.data[self.price_history.head] = point;
        }
        
        self.price_history.head = (self.price_history.head + 1) % self.price_history.capacity;
    }

    /// Update aggregator statistics
    fn update_statistics(&mut self, old_price: u64, new_price: u64) {
        // Update min/max prices
        if new_price > self.stats.max_price {
            self.stats.max_price = new_price;
        }
        if new_price < self.stats.min_price {
            self.stats.min_price = new_price;
        }

        // Update average confidence (simple moving average)
        if self.stats.successful_aggregations > 0 {
            self.stats.avg_confidence = ((self.stats.avg_confidence as u64 * (self.stats.successful_aggregations - 1) + 
                                        self.confidence as u64) / self.stats.successful_aggregations) as u16;
        } else {
            self.stats.avg_confidence = self.confidence;
        }

        // Update average active feeds
        self.stats.avg_active_feeds = ((self.stats.avg_active_feeds as u64 * self.stats.total_updates + 
                                      self.active_feeds_count as u64) / (self.stats.total_updates + 1)) as u8;
    }

    /// Update count of active feeds
    fn update_active_feeds_count(&mut self) {
        let current_time = Clock::get().unwrap().unix_timestamp;
        self.active_feeds_count = self.feed_data
            .iter()
            .filter(|feed| feed.is_active && current_time - feed.timestamp <= 300)
            .count() as u8;
    }

    /// Reset circuit breaker
    pub fn reset_circuit_breaker(&mut self) -> Result<()> {
        let current_time = Clock::get()?.unix_timestamp;
        
        require!(
            current_time - self.circuit_breaker.triggered_at >= self.circuit_breaker.cooldown_period as i64,
            crate::errors::OracleError::CircuitBreakerCooldown
        );

        self.status = AggregatorStatus::Active;
        self.circuit_breaker.triggered_at = 0;

        Ok(())
    }

    /// Update aggregator configuration
    pub fn update_config(
        &mut self,
        min_feeds_required: Option<u8>,
        max_deviation_bps: Option<u16>,
        aggregation_method: Option<AggregationMethod>,
        circuit_breaker_settings: Option<CircuitBreaker>,
    ) -> Result<()> {
        if let Some(min_feeds) = min_feeds_required {
            require!(min_feeds > 0 && min_feeds <= self.max_feeds, 
                    crate::errors::OracleError::InvalidConfiguration);
            self.min_feeds_required = min_feeds;
        }

        if let Some(max_dev) = max_deviation_bps {
            self.max_deviation_bps = max_dev;
        }

        if let Some(method) = aggregation_method {
            self.aggregation_method = method;
        }

        if let Some(cb_settings) = circuit_breaker_settings {
            self.circuit_breaker = cb_settings;
        }

        Ok(())
    }

    /// Get current aggregated price with metadata
    pub fn get_current_price(&self) -> (u64, u16, i64, AggregatorStatus) {
        (self.current_price, self.confidence, self.last_updated, self.status.clone())
    }

    /// Get price history
    pub fn get_price_history(&self) -> &CircularBuffer<PricePoint> {
        &self.price_history
    }

    /// Get aggregator health score (0-100)
    pub fn get_health_score(&self) -> u8 {
        let mut score = 100u8;

        // Reduce score based on inactive feeds
        if self.active_feeds_count < self.min_feeds_required {
            score = score.saturating_sub(50);
        } else if self.active_feeds_count < self.price_feeds.len() as u8 {
            let inactive_ratio = (self.price_feeds.len() as u8 - self.active_feeds_count) * 100 / self.price_feeds.len() as u8;
            score = score.saturating_sub(inactive_ratio / 2);
        }

        // Reduce score based on confidence
        if self.confidence < 5000 { // Less than 50%
            score = score.saturating_sub(30);
        } else if self.confidence < 8000 { // Less than 80%
            score = score.saturating_sub(10);
        }

        // Reduce score based on status
        match self.status {
            AggregatorStatus::Active => {},
            AggregatorStatus::Paused => score = score.saturating_sub(20),
            AggregatorStatus::Degraded => score = score.saturating_sub(40),
            AggregatorStatus::CircuitBroken => score = score.saturating_sub(60),
            AggregatorStatus::Emergency => score = 0,
        }

        score
    }
}
