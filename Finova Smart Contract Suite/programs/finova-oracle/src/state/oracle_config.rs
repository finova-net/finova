// programs/finova-oracle/src/state/oracle_config.rs

use anchor_lang::prelude::*;
use crate::constants::*;
use crate::errors::OracleError;

/// Oracle configuration account that manages system-wide oracle settings
#[account]
pub struct OracleConfig {
    /// Authority that can update oracle configuration
    pub authority: Pubkey,
    
    /// Emergency authority for critical updates
    pub emergency_authority: Pubkey,
    
    /// Minimum number of data sources required for valid price
    pub min_sources: u8,
    
    /// Maximum age of price data in seconds
    pub max_staleness: i64,
    
    /// Maximum deviation percentage between sources (basis points)
    pub max_deviation_bps: u16,
    
    /// Minimum confidence level required (basis points)
    pub min_confidence_bps: u16,
    
    /// Update frequency in seconds
    pub update_frequency: u64,
    
    /// List of authorized oracle providers
    pub authorized_providers: Vec<Pubkey>,
    
    /// Maximum number of providers allowed
    pub max_providers: u8,
    
    /// Oracle fee in lamports
    pub oracle_fee: u64,
    
    /// Emergency pause status
    pub is_paused: bool,
    
    /// Configuration version for upgrades
    pub version: u8,
    
    /// Circuit breaker settings
    pub circuit_breaker: CircuitBreakerConfig,
    
    /// Aggregation settings
    pub aggregation_method: AggregationMethod,
    
    /// Quality control settings
    pub quality_control: QualityControlConfig,
    
    /// Reserved space for future upgrades
    pub reserved: [u8; 64],
}

/// Circuit breaker configuration for emergency stops
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub struct CircuitBreakerConfig {
    /// Enable/disable circuit breaker
    pub enabled: bool,
    
    /// Price change threshold that triggers circuit breaker (basis points)
    pub price_change_threshold_bps: u16,
    
    /// Time window for price change detection (seconds)
    pub time_window: u64,
    
    /// Cool-down period after circuit breaker triggers (seconds)
    pub cooldown_period: u64,
    
    /// Maximum consecutive failures before triggering
    pub max_consecutive_failures: u8,
    
    /// Current failure count
    pub current_failures: u8,
    
    /// Last trigger timestamp
    pub last_trigger_ts: i64,
}

/// Aggregation method for combining multiple price sources
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum AggregationMethod {
    /// Simple arithmetic mean
    Mean,
    /// Weighted average based on confidence
    WeightedMean,
    /// Median value
    Median,
    /// Time-weighted average price (TWAP)
    TWAP { window: u64 },
    /// Volume-weighted average price (VWAP)
    VWAP { window: u64 },
}

/// Quality control configuration
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub struct QualityControlConfig {
    /// Enable outlier detection
    pub outlier_detection_enabled: bool,
    
    /// Standard deviation multiplier for outlier detection
    pub outlier_std_dev_multiplier: u16, // in basis points
    
    /// Minimum data points required for outlier detection
    pub min_data_points_for_outlier: u8,
    
    /// Enable data freshness checks
    pub freshness_check_enabled: bool,
    
    /// Enable cross-validation between sources
    pub cross_validation_enabled: bool,
    
    /// Minimum correlation coefficient for cross-validation (basis points)
    pub min_correlation_bps: u16,
}

/// Provider performance metrics
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, Default, PartialEq)]
pub struct ProviderMetrics {
    /// Total number of updates provided
    pub total_updates: u64,
    
    /// Number of successful updates
    pub successful_updates: u64,
    
    /// Number of failed updates
    pub failed_updates: u64,
    
    /// Average response time in milliseconds
    pub avg_response_time_ms: u32,
    
    /// Last update timestamp
    pub last_update_ts: i64,
    
    /// Reliability score (0-10000 basis points)
    pub reliability_score: u16,
    
    /// Current streak of successful updates
    pub success_streak: u32,
    
    /// Maximum observed streak
    pub max_success_streak: u32,
}

/// Oracle provider information
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub struct OracleProvider {
    /// Provider's public key
    pub pubkey: Pubkey,
    
    /// Provider name/identifier
    pub name: String,
    
    /// Provider weight in aggregation (basis points)
    pub weight: u16,
    
    /// Whether provider is active
    pub is_active: bool,
    
    /// Provider's fee in lamports
    pub fee: u64,
    
    /// Provider's performance metrics
    pub metrics: ProviderMetrics,
    
    /// Provider registration timestamp
    pub registered_at: i64,
    
    /// Last heartbeat timestamp
    pub last_heartbeat: i64,
    
    /// Provider reputation score
    pub reputation_score: u16,
}

impl OracleConfig {
    /// Size of the oracle config account
    pub const SIZE: usize = 8 + // discriminator
        32 + // authority
        32 + // emergency_authority
        1 + // min_sources
        8 + // max_staleness
        2 + // max_deviation_bps
        2 + // min_confidence_bps
        8 + // update_frequency
        4 + (32 * MAX_ORACLE_PROVIDERS as usize) + // authorized_providers
        1 + // max_providers
        8 + // oracle_fee
        1 + // is_paused
        1 + // version
        CircuitBreakerConfig::SIZE + // circuit_breaker
        AggregationMethod::SIZE + // aggregation_method
        QualityControlConfig::SIZE + // quality_control
        64; // reserved

    /// Initialize oracle configuration with default values
    pub fn initialize(
        &mut self,
        authority: Pubkey,
        emergency_authority: Pubkey,
    ) -> Result<()> {
        self.authority = authority;
        self.emergency_authority = emergency_authority;
        self.min_sources = DEFAULT_MIN_SOURCES;
        self.max_staleness = DEFAULT_MAX_STALENESS;
        self.max_deviation_bps = DEFAULT_MAX_DEVIATION_BPS;
        self.min_confidence_bps = DEFAULT_MIN_CONFIDENCE_BPS;
        self.update_frequency = DEFAULT_UPDATE_FREQUENCY;
        self.authorized_providers = Vec::new();
        self.max_providers = MAX_ORACLE_PROVIDERS;
        self.oracle_fee = DEFAULT_ORACLE_FEE;
        self.is_paused = false;
        self.version = ORACLE_CONFIG_VERSION;
        
        self.circuit_breaker = CircuitBreakerConfig {
            enabled: true,
            price_change_threshold_bps: CIRCUIT_BREAKER_THRESHOLD_BPS,
            time_window: CIRCUIT_BREAKER_TIME_WINDOW,
            cooldown_period: CIRCUIT_BREAKER_COOLDOWN,
            max_consecutive_failures: MAX_CONSECUTIVE_FAILURES,
            current_failures: 0,
            last_trigger_ts: 0,
        };
        
        self.aggregation_method = AggregationMethod::WeightedMean;
        
        self.quality_control = QualityControlConfig {
            outlier_detection_enabled: true,
            outlier_std_dev_multiplier: OUTLIER_STD_DEV_MULTIPLIER_BPS,
            min_data_points_for_outlier: MIN_DATA_POINTS_FOR_OUTLIER,
            freshness_check_enabled: true,
            cross_validation_enabled: true,
            min_correlation_bps: MIN_CORRELATION_BPS,
        };
        
        self.reserved = [0; 64];
        
        Ok(())
    }

    /// Add an authorized oracle provider
    pub fn add_provider(&mut self, provider: Pubkey) -> Result<()> {
        require!(
            !self.is_paused,
            OracleError::OraclePaused
        );
        
        require!(
            self.authorized_providers.len() < self.max_providers as usize,
            OracleError::TooManyProviders
        );
        
        require!(
            !self.authorized_providers.contains(&provider),
            OracleError::ProviderAlreadyExists
        );
        
        self.authorized_providers.push(provider);
        
        Ok(())
    }

    /// Remove an authorized oracle provider
    pub fn remove_provider(&mut self, provider: Pubkey) -> Result<()> {
        require!(
            !self.is_paused,
            OracleError::OraclePaused
        );
        
        let index = self.authorized_providers
            .iter()
            .position(|&p| p == provider)
            .ok_or(OracleError::ProviderNotFound)?;
        
        self.authorized_providers.remove(index);
        
        // Ensure we still have minimum required providers
        require!(
            self.authorized_providers.len() >= self.min_sources as usize,
            OracleError::InsufficientProviders
        );
        
        Ok(())
    }

    /// Check if a provider is authorized
    pub fn is_provider_authorized(&self, provider: &Pubkey) -> bool {
        self.authorized_providers.contains(provider)
    }

    /// Update oracle configuration parameters
    pub fn update_config(
        &mut self,
        min_sources: Option<u8>,
        max_staleness: Option<i64>,
        max_deviation_bps: Option<u16>,
        min_confidence_bps: Option<u16>,
        update_frequency: Option<u64>,
        oracle_fee: Option<u64>,
    ) -> Result<()> {
        require!(
            !self.is_paused,
            OracleError::OraclePaused
        );

        if let Some(sources) = min_sources {
            require!(
                sources > 0 && sources <= MAX_ORACLE_PROVIDERS,
                OracleError::InvalidMinSources
            );
            self.min_sources = sources;
        }

        if let Some(staleness) = max_staleness {
            require!(
                staleness > 0 && staleness <= MAX_STALENESS_LIMIT,
                OracleError::InvalidStaleness
            );
            self.max_staleness = staleness;
        }

        if let Some(deviation) = max_deviation_bps {
            require!(
                deviation <= MAX_DEVIATION_BPS_LIMIT,
                OracleError::InvalidDeviation
            );
            self.max_deviation_bps = deviation;
        }

        if let Some(confidence) = min_confidence_bps {
            require!(
                confidence <= MAX_CONFIDENCE_BPS,
                OracleError::InvalidConfidence
            );
            self.min_confidence_bps = confidence;
        }

        if let Some(frequency) = update_frequency {
            require!(
                frequency >= MIN_UPDATE_FREQUENCY && frequency <= MAX_UPDATE_FREQUENCY,
                OracleError::InvalidUpdateFrequency
            );
            self.update_frequency = frequency;
        }

        if let Some(fee) = oracle_fee {
            require!(
                fee <= MAX_ORACLE_FEE,
                OracleError::ExcessiveOracleFee
            );
            self.oracle_fee = fee;
        }

        Ok(())
    }

    /// Update circuit breaker configuration
    pub fn update_circuit_breaker(
        &mut self,
        enabled: Option<bool>,
        threshold_bps: Option<u16>,
        time_window: Option<u64>,
        cooldown_period: Option<u64>,
        max_failures: Option<u8>,
    ) -> Result<()> {
        if let Some(enabled) = enabled {
            self.circuit_breaker.enabled = enabled;
        }

        if let Some(threshold) = threshold_bps {
            require!(
                threshold <= MAX_CIRCUIT_BREAKER_THRESHOLD_BPS,
                OracleError::InvalidCircuitBreakerThreshold
            );
            self.circuit_breaker.price_change_threshold_bps = threshold;
        }

        if let Some(window) = time_window {
            require!(
                window >= MIN_CIRCUIT_BREAKER_TIME_WINDOW,
                OracleError::InvalidTimeWindow
            );
            self.circuit_breaker.time_window = window;
        }

        if let Some(cooldown) = cooldown_period {
            require!(
                cooldown >= MIN_CIRCUIT_BREAKER_COOLDOWN,
                OracleError::InvalidCooldownPeriod
            );
            self.circuit_breaker.cooldown_period = cooldown;
        }

        if let Some(max_failures) = max_failures {
            require!(
                max_failures > 0 && max_failures <= MAX_CONSECUTIVE_FAILURES_LIMIT,
                OracleError::InvalidMaxFailures
            );
            self.circuit_breaker.max_consecutive_failures = max_failures;
        }

        Ok(())
    }

    /// Check if circuit breaker should be triggered
    pub fn should_trigger_circuit_breaker(
        &self,
        price_change_bps: u16,
        current_time: i64,
    ) -> bool {
        if !self.circuit_breaker.enabled {
            return false;
        }

        // Check if still in cooldown period
        if current_time - self.circuit_breaker.last_trigger_ts < self.circuit_breaker.cooldown_period as i64 {
            return false;
        }

        // Check price change threshold
        price_change_bps >= self.circuit_breaker.price_change_threshold_bps
    }

    /// Trigger circuit breaker
    pub fn trigger_circuit_breaker(&mut self, current_time: i64) -> Result<()> {
        self.circuit_breaker.current_failures += 1;
        self.circuit_breaker.last_trigger_ts = current_time;
        self.is_paused = true;
        
        Ok(())
    }

    /// Reset circuit breaker after successful operation
    pub fn reset_circuit_breaker(&mut self) -> Result<()> {
        self.circuit_breaker.current_failures = 0;
        
        Ok(())
    }

    /// Check if oracle is in emergency pause
    pub fn is_emergency_paused(&self) -> bool {
        self.is_paused
    }

    /// Emergency pause the oracle
    pub fn emergency_pause(&mut self) -> Result<()> {
        self.is_paused = true;
        Ok(())
    }

    /// Resume oracle operations
    pub fn resume(&mut self) -> Result<()> {
        self.is_paused = false;
        self.reset_circuit_breaker()?;
        Ok(())
    }

    /// Get active provider count
    pub fn get_active_provider_count(&self) -> usize {
        self.authorized_providers.len()
    }

    /// Validate configuration consistency
    pub fn validate_config(&self) -> Result<()> {
        require!(
            self.min_sources > 0,
            OracleError::InvalidMinSources
        );

        require!(
            self.authorized_providers.len() >= self.min_sources as usize,
            OracleError::InsufficientProviders
        );

        require!(
            self.max_staleness > 0,
            OracleError::InvalidStaleness
        );

        require!(
            self.max_deviation_bps <= MAX_DEVIATION_BPS_LIMIT,
            OracleError::InvalidDeviation
        );

        require!(
            self.min_confidence_bps <= MAX_CONFIDENCE_BPS,
            OracleError::InvalidConfidence
        );

        require!(
            self.update_frequency >= MIN_UPDATE_FREQUENCY,
            OracleError::InvalidUpdateFrequency
        );

        Ok(())
    }
}

impl CircuitBreakerConfig {
    pub const SIZE: usize = 1 + 2 + 8 + 8 + 1 + 1 + 8;
}

impl AggregationMethod {
    pub const SIZE: usize = 1 + 8; // enum variant + max data size
}

impl QualityControlConfig {
    pub const SIZE: usize = 1 + 2 + 1 + 1 + 1 + 2;
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            price_change_threshold_bps: CIRCUIT_BREAKER_THRESHOLD_BPS,
            time_window: CIRCUIT_BREAKER_TIME_WINDOW,
            cooldown_period: CIRCUIT_BREAKER_COOLDOWN,
            max_consecutive_failures: MAX_CONSECUTIVE_FAILURES,
            current_failures: 0,
            last_trigger_ts: 0,
        }
    }
}

impl Default for AggregationMethod {
    fn default() -> Self {
        Self::WeightedMean
    }
}

impl Default for QualityControlConfig {
    fn default() -> Self {
        Self {
            outlier_detection_enabled: true,
            outlier_std_dev_multiplier: OUTLIER_STD_DEV_MULTIPLIER_BPS,
            min_data_points_for_outlier: MIN_DATA_POINTS_FOR_OUTLIER,
            freshness_check_enabled: true,
            cross_validation_enabled: true,
            min_correlation_bps: MIN_CORRELATION_BPS,
        }
    }
}

impl Default for OracleProvider {
    fn default() -> Self {
        Self {
            pubkey: Pubkey::default(),
            name: String::new(),
            weight: BASIS_POINTS_MAX / 10, // 10% default weight
            is_active: true,
            fee: DEFAULT_ORACLE_FEE,
            metrics: ProviderMetrics::default(),
            registered_at: 0,
            last_heartbeat: 0,
            reputation_score: BASIS_POINTS_MAX / 2, // 50% initial reputation
        }
    }
}

/// Helper function to calculate provider weight based on performance
pub fn calculate_dynamic_weight(metrics: &ProviderMetrics, base_weight: u16) -> u16 {
    let reliability_factor = metrics.reliability_score as u32;
    let success_rate = if metrics.total_updates > 0 {
        (metrics.successful_updates * BASIS_POINTS_MAX as u64) / metrics.total_updates
    } else {
        BASIS_POINTS_MAX as u64 / 2 // 50% default
    };
    
    let dynamic_weight = (base_weight as u32 * reliability_factor * success_rate as u32) 
        / (BASIS_POINTS_MAX as u32 * BASIS_POINTS_MAX as u32);
    
    std::cmp::min(dynamic_weight as u16, BASIS_POINTS_MAX)
}

/// Helper function to update provider metrics
pub fn update_provider_metrics(
    metrics: &mut ProviderMetrics,
    success: bool,
    response_time_ms: u32,
    current_time: i64,
) {
    metrics.total_updates += 1;
    metrics.last_update_ts = current_time;
    
    if success {
        metrics.successful_updates += 1;
        metrics.success_streak += 1;
        metrics.max_success_streak = std::cmp::max(metrics.max_success_streak, metrics.success_streak);
    } else {
        metrics.failed_updates += 1;
        metrics.success_streak = 0;
    }
    
    // Update rolling average response time
    let total_weight = metrics.total_updates;
    let old_weight = total_weight - 1;
    metrics.avg_response_time_ms = ((metrics.avg_response_time_ms as u64 * old_weight + response_time_ms as u64) / total_weight) as u32;
    
    // Calculate reliability score based on recent performance
    let success_rate_bps = if metrics.total_updates > 0 {
        ((metrics.successful_updates * BASIS_POINTS_MAX as u64) / metrics.total_updates) as u16
    } else {
        BASIS_POINTS_MAX / 2
    };
    
    // Weight recent performance more heavily
    let recency_factor = std::cmp::min(100, metrics.success_streak) as u16 * 10; // Up to 1000 basis points bonus
    metrics.reliability_score = std::cmp::min(
        BASIS_POINTS_MAX,
        success_rate_bps + recency_factor
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_oracle_config_initialization() {
        let mut config = OracleConfig::default();
        let authority = Pubkey::new_unique();
        let emergency_authority = Pubkey::new_unique();
        
        config.initialize(authority, emergency_authority).unwrap();
        
        assert_eq!(config.authority, authority);
        assert_eq!(config.emergency_authority, emergency_authority);
        assert_eq!(config.min_sources, DEFAULT_MIN_SOURCES);
        assert!(!config.is_paused);
        assert_eq!(config.version, ORACLE_CONFIG_VERSION);
    }

    #[test]
    fn test_provider_management() {
        let mut config = OracleConfig::default();
        config.initialize(Pubkey::new_unique(), Pubkey::new_unique()).unwrap();
        
        let provider1 = Pubkey::new_unique();
        let provider2 = Pubkey::new_unique();
        
        // Add providers
        config.add_provider(provider1).unwrap();
        config.add_provider(provider2).unwrap();
        
        assert!(config.is_provider_authorized(&provider1));
        assert!(config.is_provider_authorized(&provider2));
        assert_eq!(config.get_active_provider_count(), 2);
        
        // Remove provider
        config.remove_provider(provider1).unwrap();
        assert!(!config.is_provider_authorized(&provider1));
        assert_eq!(config.get_active_provider_count(), 1);
    }

    #[test]
    fn test_circuit_breaker() {
        let mut config = OracleConfig::default();
        config.initialize(Pubkey::new_unique(), Pubkey::new_unique()).unwrap();
        
        let current_time = 1000000;
        let high_price_change = config.circuit_breaker.price_change_threshold_bps + 100;
        
        assert!(config.should_trigger_circuit_breaker(high_price_change, current_time));
        
        config.trigger_circuit_breaker(current_time).unwrap();
        assert!(config.is_emergency_paused());
        
        // Should not trigger again during cooldown
        assert!(!config.should_trigger_circuit_breaker(high_price_change, current_time + 1));
        
        config.resume().unwrap();
        assert!(!config.is_emergency_paused());
    }

    #[test]
    fn test_dynamic_weight_calculation() {
        let mut metrics = ProviderMetrics::default();
        metrics.total_updates = 100;
        metrics.successful_updates = 95;
        metrics.reliability_score = 9500; // 95%
        
        let base_weight = 1000; // 10%
        let dynamic_weight = calculate_dynamic_weight(&metrics, base_weight);
        
        assert!(dynamic_weight > base_weight);
    }
}
