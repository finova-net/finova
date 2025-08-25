// programs/finova-oracle/src/state/mod.rs

use anchor_lang::prelude::*;

pub mod price_feed;
pub mod aggregator;
pub mod oracle_config;

pub use price_feed::*;
pub use aggregator::*;
pub use oracle_config::*;

/// Oracle state validation trait
pub trait OracleStateValidation {
    fn validate_state(&self) -> Result<()>;
    fn is_active(&self) -> bool;
    fn last_update_time(&self) -> i64;
    fn staleness_threshold(&self) -> i64;
}

/// Price data validation
pub trait PriceValidation {
    fn validate_price(&self, price: u64) -> Result<()>;
    fn validate_confidence(&self, confidence: u64) -> Result<()>;
    fn validate_timestamp(&self, timestamp: i64) -> Result<()>;
}

/// Common oracle constants
pub const MAX_PRICE_FEEDS: usize = 50;
pub const MIN_CONFIDENCE_THRESHOLD: u64 = 10_000; // 1% in basis points
pub const MAX_PRICE_DEVIATION: u64 = 1_000_000; // 100% in basis points
pub const DEFAULT_STALENESS_THRESHOLD: i64 = 300; // 5 minutes in seconds
pub const EMERGENCY_STALENESS_THRESHOLD: i64 = 3600; // 1 hour in seconds

/// Oracle status enumeration
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq)]
pub enum OracleStatus {
    Active,
    Paused,
    Emergency,
    Deprecated,
}

impl Default for OracleStatus {
    fn default() -> Self {
        OracleStatus::Active
    }
}

/// Price aggregation method
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq)]
pub enum AggregationMethod {
    MedianPrice,
    WeightedAverage,
    VolumeWeightedAverage,
    CustomAlgorithm,
}

impl Default for AggregationMethod {
    fn default() -> Self {
        AggregationMethod::MedianPrice
    }
}

/// Oracle source type
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq)]
pub enum OracleSourceType {
    Chainlink,
    Pyth,
    Switchboard,
    Custom,
    Internal,
}

impl Default for OracleSourceType {
    fn default() -> Self {
        OracleSourceType::Pyth
    }
}

/// Price feed configuration
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct PriceFeedConfig {
    pub feed_id: [u8; 32],
    pub source_type: OracleSourceType,
    pub weight: u64,
    pub staleness_threshold: i64,
    pub min_confidence: u64,
    pub max_deviation: u64,
    pub is_active: bool,
}

impl Default for PriceFeedConfig {
    fn default() -> Self {
        Self {
            feed_id: [0; 32],
            source_type: OracleSourceType::default(),
            weight: 100,
            staleness_threshold: DEFAULT_STALENESS_THRESHOLD,
            min_confidence: MIN_CONFIDENCE_THRESHOLD,
            max_deviation: MAX_PRICE_DEVIATION,
            is_active: true,
        }
    }
}

/// Aggregated price data
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct AggregatedPrice {
    pub price: u64,
    pub confidence: u64,
    pub timestamp: i64,
    pub method: AggregationMethod,
    pub feed_count: u8,
    pub deviation: u64,
}

impl Default for AggregatedPrice {
    fn default() -> Self {
        Self {
            price: 0,
            confidence: 0,
            timestamp: 0,
            method: AggregationMethod::default(),
            feed_count: 0,
            deviation: 0,
        }
    }
}

/// Oracle metrics for monitoring
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct OracleMetrics {
    pub total_updates: u64,
    pub successful_updates: u64,
    pub failed_updates: u64,
    pub average_deviation: u64,
    pub uptime_percentage: u64,
    pub last_heartbeat: i64,
}

impl Default for OracleMetrics {
    fn default() -> Self {
        Self {
            total_updates: 0,
            successful_updates: 0,
            failed_updates: 0,
            average_deviation: 0,
            uptime_percentage: 10000, // 100% in basis points
            last_heartbeat: 0,
        }
    }
}

/// Circuit breaker configuration
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct CircuitBreakerConfig {
    pub enabled: bool,
    pub max_price_deviation: u64,
    pub max_confidence_drop: u64,
    pub max_consecutive_failures: u8,
    pub cooldown_period: i64,
    pub emergency_contacts: Vec<Pubkey>,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_price_deviation: 500_000, // 50% in basis points
            max_confidence_drop: 500_000, // 50% in basis points
            max_consecutive_failures: 5,
            cooldown_period: 1800, // 30 minutes
            emergency_contacts: Vec::new(),
        }
    }
}

/// Oracle access control
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct AccessControl {
    pub admin: Pubkey,
    pub updaters: Vec<Pubkey>,
    pub emergency_contacts: Vec<Pubkey>,
    pub whitelist_enabled: bool,
    pub whitelisted_programs: Vec<Pubkey>,
}

impl Default for AccessControl {
    fn default() -> Self {
        Self {
            admin: Pubkey::default(),
            updaters: Vec::new(),
            emergency_contacts: Vec::new(),
            whitelist_enabled: false,
            whitelisted_programs: Vec::new(),
        }
    }
}

/// Validation result for oracle operations
#[derive(Debug)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub error_code: Option<u32>,
    pub confidence_score: u64,
    pub staleness_score: u64,
}

impl ValidationResult {
    pub fn new(is_valid: bool) -> Self {
        Self {
            is_valid,
            error_code: None,
            confidence_score: 0,
            staleness_score: 0,
        }
    }

    pub fn with_error(mut self, error_code: u32) -> Self {
        self.error_code = Some(error_code);
        self.is_valid = false;
        self
    }

    pub fn with_confidence(mut self, confidence: u64) -> Self {
        self.confidence_score = confidence;
        self
    }

    pub fn with_staleness(mut self, staleness: u64) -> Self {
        self.staleness_score = staleness;
        self
    }
}

/// Price history entry for trend analysis
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct PriceHistoryEntry {
    pub timestamp: i64,
    pub price: u64,
    pub confidence: u64,
    pub volume: u64,
}

impl Default for PriceHistoryEntry {
    fn default() -> Self {
        Self {
            timestamp: 0,
            price: 0,
            confidence: 0,
            volume: 0,
        }
    }
}

/// Oracle health status
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum HealthStatus {
    Healthy,
    Warning,
    Critical,
    Offline,
}

impl Default for HealthStatus {
    fn default() -> Self {
        HealthStatus::Healthy
    }
}

/// Health check configuration
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct HealthCheckConfig {
    pub check_interval: i64,
    pub max_staleness: i64,
    pub min_confidence: u64,
    pub max_deviation: u64,
    pub alert_thresholds: HealthThresholds,
}

impl Default for HealthCheckConfig {
    fn default() -> Self {
        Self {
            check_interval: 60, // 1 minute
            max_staleness: DEFAULT_STALENESS_THRESHOLD,
            min_confidence: MIN_CONFIDENCE_THRESHOLD,
            max_deviation: MAX_PRICE_DEVIATION,
            alert_thresholds: HealthThresholds::default(),
        }
    }
}

/// Health monitoring thresholds
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct HealthThresholds {
    pub warning_staleness: i64,
    pub critical_staleness: i64,
    pub warning_confidence: u64,
    pub critical_confidence: u64,
    pub warning_deviation: u64,
    pub critical_deviation: u64,
}

impl Default for HealthThresholds {
    fn default() -> Self {
        Self {
            warning_staleness: 180, // 3 minutes
            critical_staleness: 600, // 10 minutes
            warning_confidence: 50_000, // 5%
            critical_confidence: 100_000, // 10%
            warning_deviation: 200_000, // 20%
            critical_deviation: 500_000, // 50%
        }
    }
}

/// Oracle fee structure
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct FeeStructure {
    pub base_fee: u64,
    pub premium_fee: u64,
    pub update_fee: u64,
    pub subscription_fee: u64,
    pub fee_recipient: Pubkey,
    pub fee_decimals: u8,
}

impl Default for FeeStructure {
    fn default() -> Self {
        Self {
            base_fee: 1000, // 0.1 FIN
            premium_fee: 5000, // 0.5 FIN
            update_fee: 100, // 0.01 FIN
            subscription_fee: 10000, // 1 FIN per month
            fee_recipient: Pubkey::default(),
            fee_decimals: 4,
        }
    }
}

/// Utility functions for oracle state management
impl OracleStatus {
    pub fn is_operational(&self) -> bool {
        matches!(self, OracleStatus::Active)
    }

    pub fn can_update(&self) -> bool {
        matches!(self, OracleStatus::Active | OracleStatus::Emergency)
    }

    pub fn requires_emergency_approval(&self) -> bool {
        matches!(self, OracleStatus::Emergency)
    }
}

impl AggregationMethod {
    pub fn requires_weights(&self) -> bool {
        matches!(self, AggregationMethod::WeightedAverage | AggregationMethod::VolumeWeightedAverage)
    }

    pub fn min_feed_count(&self) -> usize {
        match self {
            AggregationMethod::MedianPrice => 3,
            AggregationMethod::WeightedAverage => 2,
            AggregationMethod::VolumeWeightedAverage => 2,
            AggregationMethod::CustomAlgorithm => 1,
        }
    }
}

impl HealthStatus {
    pub fn is_operational(&self) -> bool {
        matches!(self, HealthStatus::Healthy | HealthStatus::Warning)
    }

    pub fn requires_attention(&self) -> bool {
        matches!(self, HealthStatus::Warning | HealthStatus::Critical)
    }

    pub fn is_critical(&self) -> bool {
        matches!(self, HealthStatus::Critical | HealthStatus::Offline)
    }
}

/// State size calculation helpers
pub const fn get_price_feed_size() -> usize {
    8 + // discriminator
    32 + // feed_id
    32 + // oracle_authority
    8 + // price
    8 + // confidence
    8 + // timestamp
    1 + // status
    1 + // source_type
    8 + // weight
    8 + // staleness_threshold
    8 + // min_confidence
    8 + // max_deviation
    1 + // is_active
    32 + // last_updater
    8 + // update_count
    8 + // total_volume
    8 + // average_price
    8   // reserved
}

pub const fn get_aggregator_size() -> usize {
    8 + // discriminator
    32 + // authority
    32 + // base_token
    32 + // quote_token
    4 + 32 * MAX_PRICE_FEEDS + // price_feeds
    8 + // current_price
    8 + // confidence
    8 + // timestamp
    1 + // aggregation_method
    1 + // status
    8 + // min_feeds
    8 + // max_deviation
    8 + // staleness_threshold
    200 // reserved space
}

pub const fn get_oracle_config_size() -> usize {
    8 + // discriminator
    32 + // admin
    32 + // emergency_authority
    4 + 32 * 10 + // updaters (max 10)
    4 + 32 * 5 + // emergency_contacts (max 5)
    1 + // whitelist_enabled
    4 + 32 * 20 + // whitelisted_programs (max 20)
    100 + // circuit_breaker_config
    50 + // health_check_config
    50 + // fee_structure
    8 + // creation_timestamp
    8 + // last_update_timestamp
    100 // reserved space
}
