// programs/finova-oracle/src/errors.rs

use anchor_lang::prelude::*;

/// Oracle-specific error codes for the Finova Network
#[error_code]
pub enum OracleError {
    // ============================================
    // INITIALIZATION ERRORS (6000-6099)
    // ============================================
    #[msg("Oracle already initialized")]
    AlreadyInitialized = 6000,

    #[msg("Invalid oracle configuration parameters")]
    InvalidConfiguration = 6001,

    #[msg("Oracle not initialized")]
    NotInitialized = 6002,

    #[msg("Invalid aggregator configuration")]
    InvalidAggregatorConfig = 6003,

    #[msg("Insufficient oracles for aggregation")]
    InsufficientOracles = 6004,

    #[msg("Maximum oracles per aggregator exceeded")]
    MaxOraclesExceeded = 6005,

    #[msg("Invalid update frequency")]
    InvalidUpdateFrequency = 6006,

    #[msg("Invalid price threshold")]
    InvalidPriceThreshold = 6007,

    #[msg("Invalid deviation threshold")]
    InvalidDeviationThreshold = 6008,

    #[msg("Duplicate oracle feed")]
    DuplicateOracleFeed = 6009,

    // ============================================
    // AUTHORIZATION ERRORS (6100-6199)
    // ============================================
    #[msg("Unauthorized oracle operator")]
    UnauthorizedOperator = 6100,

    #[msg("Invalid oracle authority")]
    InvalidAuthority = 6101,

    #[msg("Oracle operator not whitelisted")]
    OperatorNotWhitelisted = 6102,

    #[msg("Insufficient permissions for operation")]
    InsufficientPermissions = 6103,

    #[msg("Emergency pause active")]
    EmergencyPauseActive = 6104,

    #[msg("Oracle feed disabled")]
    OracleFeedDisabled = 6105,

    #[msg("Operator suspended")]
    OperatorSuspended = 6106,

    #[msg("Invalid signature for price update")]
    InvalidPriceSignature = 6107,

    #[msg("Signature verification failed")]
    SignatureVerificationFailed = 6108,

    #[msg("Nonce already used")]
    NonceAlreadyUsed = 6109,

    // ============================================
    // PRICE FEED ERRORS (6200-6299)
    // ============================================
    #[msg("Price feed not found")]
    PriceFeedNotFound = 6200,

    #[msg("Stale price data")]
    StalePriceData = 6201,

    #[msg("Invalid price value")]
    InvalidPriceValue = 6202,

    #[msg("Price deviation too high")]
    PriceDeviationTooHigh = 6203,

    #[msg("Insufficient price data for aggregation")]
    InsufficientPriceData = 6204,

    #[msg("Price update too frequent")]
    PriceUpdateTooFrequent = 6205,

    #[msg("Price update required")]
    PriceUpdateRequired = 6206,

    #[msg("Invalid confidence interval")]
    InvalidConfidenceInterval = 6207,

    #[msg("Price out of bounds")]
    PriceOutOfBounds = 6208,

    #[msg("Negative price not allowed")]
    NegativePriceNotAllowed = 6209,

    #[msg("Zero price not allowed")]
    ZeroPriceNotAllowed = 6210,

    #[msg("Price precision overflow")]
    PricePrecisionOverflow = 6211,

    #[msg("Invalid price timestamp")]
    InvalidPriceTimestamp = 6212,

    #[msg("Future timestamp not allowed")]
    FutureTimestampNotAllowed = 6213,

    #[msg("Price too old")]
    PriceTooOld = 6214,

    // ============================================
    // AGGREGATION ERRORS (6300-6399)
    // ============================================
    #[msg("Aggregation failed")]
    AggregationFailed = 6300,

    #[msg("Outlier detection failed")]
    OutlierDetectionFailed = 6301,

    #[msg("Weighted average calculation failed")]
    WeightedAverageCalculationFailed = 6302,

    #[msg("Median calculation failed")]
    MedianCalculationFailed = 6303,

    #[msg("Standard deviation calculation failed")]
    StandardDeviationCalculationFailed = 6304,

    #[msg("Confidence score calculation failed")]
    ConfidenceScoreCalculationFailed = 6305,

    #[msg("Too many outliers detected")]
    TooManyOutliersDetected = 6306,

    #[msg("All prices are outliers")]
    AllPricesAreOutliers = 6307,

    #[msg("Aggregation variance too high")]
    AggregationVarianceTooHigh = 6308,

    #[msg("Invalid weight distribution")]
    InvalidWeightDistribution = 6309,

    #[msg("Aggregation timeout")]
    AggregationTimeout = 6310,

    #[msg("Consensus not reached")]
    ConsensusNotReached = 6311,

    #[msg("Insufficient valid prices")]
    InsufficientValidPrices = 6312,

    #[msg("Price correlation too low")]
    PriceCorrelationTooLow = 6313,

    #[msg("Aggregation circuit breaker triggered")]
    AggregationCircuitBreakerTriggered = 6314,

    // ============================================
    // MATHEMATICAL ERRORS (6400-6499)
    // ============================================
    #[msg("Mathematical overflow")]
    MathematicalOverflow = 6400,

    #[msg("Mathematical underflow")]
    MathematicalUnderflow = 6401,

    #[msg("Division by zero")]
    DivisionByZero = 6402,

    #[msg("Square root of negative number")]
    SquareRootOfNegative = 6403,

    #[msg("Logarithm of non-positive number")]
    LogarithmOfNonPositive = 6404,

    #[msg("Invalid exponential calculation")]
    InvalidExponentialCalculation = 6405,

    #[msg("Precision loss in calculation")]
    PrecisionLoss = 6406,

    #[msg("Rounding error")]
    RoundingError = 6407,

    #[msg("Invalid decimal places")]
    InvalidDecimalPlaces = 6408,

    #[msg("Number too large for calculation")]
    NumberTooLarge = 6409,

    #[msg("Number too small for calculation")]
    NumberTooSmall = 6410,

    #[msg("Invalid mathematical operation")]
    InvalidMathematicalOperation = 6411,

    #[msg("Floating point error")]
    FloatingPointError = 6412,

    #[msg("Invalid percentage calculation")]
    InvalidPercentageCalculation = 6413,

    #[msg("Statistical calculation error")]
    StatisticalCalculationError = 6414,

    // ============================================
    // NETWORK ERRORS (6500-6599)
    // ============================================
    #[msg("Network connectivity error")]
    NetworkConnectivityError = 6500,

    #[msg("External API call failed")]
    ExternalApiCallFailed = 6501,

    #[msg("Data source unavailable")]
    DataSourceUnavailable = 6502,

    #[msg("Network timeout")]
    NetworkTimeout = 6503,

    #[msg("Rate limit exceeded")]
    RateLimitExceeded = 6504,

    #[msg("Invalid API response")]
    InvalidApiResponse = 6505,

    #[msg("API key invalid")]
    ApiKeyInvalid = 6506,

    #[msg("Service temporarily unavailable")]
    ServiceTemporarilyUnavailable = 6507,

    #[msg("Data format error")]
    DataFormatError = 6508,

    #[msg("Connection pool exhausted")]
    ConnectionPoolExhausted = 6509,

    #[msg("SSL/TLS error")]
    SslTlsError = 6510,

    #[msg("HTTP error")]
    HttpError = 6511,

    #[msg("JSON parsing error")]
    JsonParsingError = 6512,

    #[msg("XML parsing error")]
    XmlParsingError = 6513,

    #[msg("Data validation error")]
    DataValidationError = 6514,

    // ============================================
    // SECURITY ERRORS (6600-6699)
    // ============================================
    #[msg("Oracle manipulation detected")]
    OracleManipulationDetected = 6600,

    #[msg("Price manipulation attempt")]
    PriceManipulationAttempt = 6601,

    #[msg("Suspicious price pattern")]
    SuspiciousPricePattern = 6602,

    #[msg("Flash loan attack detected")]
    FlashLoanAttackDetected = 6603,

    #[msg("MEV attack detected")]
    MevAttackDetected = 6604,

    #[msg("Oracle front-running detected")]
    OracleFrontRunningDetected = 6605,

    #[msg("Sandwich attack detected")]
    SandwichAttackDetected = 6606,

    #[msg("Price deviation anomaly")]
    PriceDeviationAnomaly = 6607,

    #[msg("Volume anomaly detected")]
    VolumeAnomalyDetected = 6608,

    #[msg("Liquidity manipulation detected")]
    LiquidityManipulationDetected = 6609,

    #[msg("Coordinated attack detected")]
    CoordinatedAttackDetected = 6610,

    #[msg("Oracle eclipse attack")]
    OracleEclipseAttack = 6611,

    #[msg("Data source compromised")]
    DataSourceCompromised = 6612,

    #[msg("Security threshold breached")]
    SecurityThresholdBreached = 6613,

    #[msg("Malicious operator detected")]
    MaliciousOperatorDetected = 6614,

    // ============================================
    // CIRCUIT BREAKER ERRORS (6700-6799)
    // ============================================
    #[msg("Circuit breaker open")]
    CircuitBreakerOpen = 6700,

    #[msg("Circuit breaker half-open")]
    CircuitBreakerHalfOpen = 6701,

    #[msg("Circuit breaker failure threshold exceeded")]
    CircuitBreakerFailureThreshold = 6702,

    #[msg("Circuit breaker timeout")]
    CircuitBreakerTimeout = 6703,

    #[msg("Emergency stop activated")]
    EmergencyStopActivated = 6704,

    #[msg("Oracle feed suspended")]
    OracleFeedSuspended = 6705,

    #[msg("Price feed quarantined")]
    PriceFeedQuarantined = 6706,

    #[msg("Automated shutdown triggered")]
    AutomatedShutdownTriggered = 6707,

    #[msg("Manual intervention required")]
    ManualInterventionRequired = 6708,

    #[msg("Recovery mode active")]
    RecoveryModeActive = 6709,

    #[msg("Safe mode enabled")]
    SafeModeEnabled = 6710,

    #[msg("Circuit breaker cooldown active")]
    CircuitBreakerCooldownActive = 6711,

    #[msg("System degraded performance")]
    SystemDegradedPerformance = 6712,

    #[msg("Critical error threshold reached")]
    CriticalErrorThresholdReached = 6713,

    #[msg("Service isolation activated")]
    ServiceIsolationActivated = 6714,

    // ============================================
    // CONFIGURATION ERRORS (6800-6899)
    // ============================================
    #[msg("Invalid heartbeat interval")]
    InvalidHeartbeatInterval = 6800,

    #[msg("Invalid staleness threshold")]
    InvalidStalenessThreshold = 6801,

    #[msg("Invalid minimum sources")]
    InvalidMinimumSources = 6802,

    #[msg("Invalid maximum sources")]
    InvalidMaximumSources = 6803,

    #[msg("Invalid aggregation method")]
    InvalidAggregationMethod = 6804,

    #[msg("Invalid outlier threshold")]
    InvalidOutlierThreshold = 6805,

    #[msg("Invalid confidence threshold")]
    InvalidConfidenceThreshold = 6806,

    #[msg("Invalid price range")]
    InvalidPriceRange = 6807,

    #[msg("Invalid decimals configuration")]
    InvalidDecimalsConfiguration = 6808,

    #[msg("Configuration version mismatch")]
    ConfigurationVersionMismatch = 6809,

    #[msg("Feature not enabled")]
    FeatureNotEnabled = 6810,

    #[msg("Invalid feature flag")]
    InvalidFeatureFlag = 6811,

    #[msg("Configuration locked")]
    ConfigurationLocked = 6812,

    #[msg("Invalid upgrade path")]
    InvalidUpgradePath = 6813,

    #[msg("Migration required")]
    MigrationRequired = 6814,

    // ============================================
    // GOVERNANCE ERRORS (6900-6999)
    // ============================================
    #[msg("Governance proposal not found")]
    GovernanceProposalNotFound = 6900,

    #[msg("Proposal voting period ended")]
    ProposalVotingPeriodEnded = 6901,

    #[msg("Proposal not passed")]
    ProposalNotPassed = 6902,

    #[msg("Proposal already executed")]
    ProposalAlreadyExecuted = 6903,

    #[msg("Insufficient voting power")]
    InsufficientVotingPower = 6904,

    #[msg("Already voted on proposal")]
    AlreadyVotedOnProposal = 6905,

    #[msg("Invalid proposal parameters")]
    InvalidProposalParameters = 6906,

    #[msg("Proposal execution failed")]
    ProposalExecutionFailed = 6907,

    #[msg("Quorum not reached")]
    QuorumNotReached = 6908,

    #[msg("Timelock not expired")]
    TimelockNotExpired = 6909,

    #[msg("Invalid governance action")]
    InvalidGovernanceAction = 6910,

    #[msg("Governance paused")]
    GovernancePaused = 6911,

    #[msg("Emergency governance override")]
    EmergencyGovernanceOverride = 6912,

    #[msg("Proposal cooldown active")]
    ProposalCooldownActive = 6913,

    #[msg("Invalid voting delegation")]
    InvalidVotingDelegation = 6914,
}

impl From<OracleError> for Error {
    fn from(e: OracleError) -> Self {
        Error::from(ErrorCode::Custom(e as u32))
    }
}

/// Helper trait for result handling
pub trait OracleResult<T> {
    fn oracle_error(self, error: OracleError) -> Result<T>;
}

impl<T> OracleResult<T> for Option<T> {
    fn oracle_error(self, error: OracleError) -> Result<T> {
        self.ok_or_else(|| Error::from(error))
    }
}

impl<T, E> OracleResult<T> for std::result::Result<T, E> {
    fn oracle_error(self, error: OracleError) -> Result<T> {
        self.map_err(|_| Error::from(error))
    }
}

/// Error categories for better error handling
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCategory {
    Initialization,
    Authorization,
    PriceFeed,
    Aggregation,
    Mathematical,
    Network,
    Security,
    CircuitBreaker,
    Configuration,
    Governance,
}

impl OracleError {
    /// Get the category of an error
    pub fn category(&self) -> ErrorCategory {
        match *self as u32 {
            6000..=6099 => ErrorCategory::Initialization,
            6100..=6199 => ErrorCategory::Authorization,
            6200..=6299 => ErrorCategory::PriceFeed,
            6300..=6399 => ErrorCategory::Aggregation,
            6400..=6499 => ErrorCategory::Mathematical,
            6500..=6599 => ErrorCategory::Network,
            6600..=6699 => ErrorCategory::Security,
            6700..=6799 => ErrorCategory::CircuitBreaker,
            6800..=6899 => ErrorCategory::Configuration,
            6900..=6999 => ErrorCategory::Governance,
            _ => ErrorCategory::Configuration, // Default
        }
    }

    /// Check if error is critical (should trigger emergency procedures)
    pub fn is_critical(&self) -> bool {
        matches!(
            self.category(),
            ErrorCategory::Security | ErrorCategory::CircuitBreaker
        ) || matches!(
            self,
            OracleError::OracleManipulationDetected
                | OracleError::FlashLoanAttackDetected
                | OracleError::MevAttackDetected
                | OracleError::EmergencyStopActivated
                | OracleError::CriticalErrorThresholdReached
                | OracleError::DataSourceCompromised
        )
    }

    /// Check if error is recoverable without manual intervention
    pub fn is_recoverable(&self) -> bool {
        matches!(
            self.category(),
            ErrorCategory::Network | ErrorCategory::Mathematical
        ) && !self.is_critical()
    }

    /// Get error severity level
    pub fn severity(&self) -> ErrorSeverity {
        if self.is_critical() {
            ErrorSeverity::Critical
        } else if matches!(
            self.category(),
            ErrorCategory::Security | ErrorCategory::Authorization | ErrorCategory::PriceFeed
        ) {
            ErrorSeverity::High
        } else if matches!(
            self.category(),
            ErrorCategory::Aggregation | ErrorCategory::Configuration
        ) {
            ErrorSeverity::Medium
        } else {
            ErrorSeverity::Low
        }
    }
}

/// Error severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ErrorSeverity {
    Low = 1,
    Medium = 2,
    High = 3,
    Critical = 4,
}

/// Macro for creating custom oracle errors with context
#[macro_export]
macro_rules! oracle_error {
    ($error:expr) => {
        Err(Error::from($error))
    };
    ($error:expr, $msg:expr) => {
        Err(Error::from($error).with_source(source!()).with_account_name($msg))
    };
}

/// Macro for requiring conditions with oracle errors
#[macro_export]
macro_rules! require_oracle {
    ($condition:expr, $error:expr) => {
        if !($condition) {
            return oracle_error!($error);
        }
    };
    ($condition:expr, $error:expr, $msg:expr) => {
        if !($condition) {
            return oracle_error!($error, $msg);
        }
    };
}

/// Macro for checking mathematical operations
#[macro_export]
macro_rules! checked_math {
    ($op:expr) => {
        $op.ok_or(Error::from(OracleError::MathematicalOverflow))?
    };
}

/// Helper function to map common errors to oracle errors
pub fn map_parse_error<T>(_: T) -> Error {
    Error::from(OracleError::DataFormatError)
}

pub fn map_network_error<T>(_: T) -> Error {
    Error::from(OracleError::NetworkConnectivityError)
}

pub fn map_math_error<T>(_: T) -> Error {
    Error::from(OracleError::MathematicalOverflow)
}

/// Error context for debugging
#[derive(Debug, Clone)]
pub struct ErrorContext {
    pub operation: String,
    pub oracle_feed: Option<String>,
    pub timestamp: i64,
    pub additional_data: Vec<(String, String)>,
}

impl ErrorContext {
    pub fn new(operation: &str) -> Self {
        Self {
            operation: operation.to_string(),
            oracle_feed: None,
            timestamp: Clock::get().unwrap().unix_timestamp,
            additional_data: Vec::new(),
        }
    }

    pub fn with_feed(mut self, feed: &str) -> Self {
        self.oracle_feed = Some(feed.to_string());
        self
    }

    pub fn with_data(mut self, key: &str, value: &str) -> Self {
        self.additional_data.push((key.to_string(), value.to_string()));
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_categories() {
        assert_eq!(OracleError::AlreadyInitialized.category(), ErrorCategory::Initialization);
        assert_eq!(OracleError::UnauthorizedOperator.category(), ErrorCategory::Authorization);
        assert_eq!(OracleError::PriceFeedNotFound.category(), ErrorCategory::PriceFeed);
        assert_eq!(OracleError::AggregationFailed.category(), ErrorCategory::Aggregation);
        assert_eq!(OracleError::MathematicalOverflow.category(), ErrorCategory::Mathematical);
        assert_eq!(OracleError::NetworkConnectivityError.category(), ErrorCategory::Network);
        assert_eq!(OracleError::OracleManipulationDetected.category(), ErrorCategory::Security);
        assert_eq!(OracleError::CircuitBreakerOpen.category(), ErrorCategory::CircuitBreaker);
        assert_eq!(OracleError::InvalidHeartbeatInterval.category(), ErrorCategory::Configuration);
        assert_eq!(OracleError::GovernanceProposalNotFound.category(), ErrorCategory::Governance);
    }

    #[test]
    fn test_error_severity() {
        assert_eq!(OracleError::OracleManipulationDetected.severity(), ErrorSeverity::Critical);
        assert_eq!(OracleError::UnauthorizedOperator.severity(), ErrorSeverity::High);
        assert_eq!(OracleError::AggregationFailed.severity(), ErrorSeverity::Medium);
        assert_eq!(OracleError::NetworkTimeout.severity(), ErrorSeverity::Low);
    }

    #[test]
    fn test_critical_errors() {
        assert!(OracleError::OracleManipulationDetected.is_critical());
        assert!(OracleError::FlashLoanAttackDetected.is_critical());
        assert!(OracleError::EmergencyStopActivated.is_critical());
        assert!(!OracleError::NetworkTimeout.is_critical());
    }

    #[test]
    fn test_recoverable_errors() {
        assert!(OracleError::NetworkTimeout.is_recoverable());
        assert!(OracleError::MathematicalOverflow.is_recoverable());
        assert!(!OracleError::OracleManipulationDetected.is_recoverable());
        assert!(!OracleError::EmergencyStopActivated.is_recoverable());
    }
}
