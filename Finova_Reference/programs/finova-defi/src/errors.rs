// programs/finova-defi/src/errors.rs

use anchor_lang::prelude::*;

/// Custom error codes for Finova DeFi operations
#[error_code]
pub enum FinovaDeFiError {
    // Pool-related errors (3000-3099)
    #[msg("Pool is not initialized")]
    PoolNotInitialized = 3000,
    
    #[msg("Pool is already initialized")]
    PoolAlreadyInitialized = 3001,
    
    #[msg("Pool is paused")]
    PoolPaused = 3002,
    
    #[msg("Pool has insufficient liquidity")]
    InsufficientLiquidity = 3003,
    
    #[msg("Pool reserve ratio is invalid")]
    InvalidReserveRatio = 3004,
    
    #[msg("Pool fee exceeds maximum allowed")]
    ExcessivePoolFee = 3005,
    
    #[msg("Pool capacity exceeded")]
    PoolCapacityExceeded = 3006,
    
    #[msg("Pool minimum liquidity not met")]
    MinimumLiquidityNotMet = 3007,
    
    #[msg("Pool is not active")]
    PoolNotActive = 3008,
    
    #[msg("Pool configuration is invalid")]
    InvalidPoolConfiguration = 3009,
    
    #[msg("Pool token mismatch")]
    PoolTokenMismatch = 3010,
    
    #[msg("Pool oracle price is stale")]
    StaleOraclePrice = 3011,
    
    #[msg("Pool slippage tolerance exceeded")]
    SlippageToleranceExceeded = 3012,
    
    #[msg("Pool reserve imbalance detected")]
    ReserveImbalanceDetected = 3013,
    
    #[msg("Pool deadline exceeded")]
    DeadlineExceeded = 3014,
    
    // Liquidity provision errors (3100-3199)
    #[msg("Insufficient token A amount")]
    InsufficientTokenAAmount = 3100,
    
    #[msg("Insufficient token B amount")]
    InsufficientTokenBAmount = 3101,
    
    #[msg("Liquidity amount is too small")]
    LiquidityAmountTooSmall = 3102,
    
    #[msg("Liquidity amount is too large")]
    LiquidityAmountTooLarge = 3103,
    
    #[msg("Liquidity token mint failed")]
    LiquidityTokenMintFailed = 3104,
    
    #[msg("Liquidity token burn failed")]
    LiquidityTokenBurnFailed = 3105,
    
    #[msg("Liquidity position not found")]
    LiquidityPositionNotFound = 3106,
    
    #[msg("Liquidity position is locked")]
    LiquidityPositionLocked = 3107,
    
    #[msg("Liquidity withdrawal amount exceeds balance")]
    WithdrawalExceedsBalance = 3108,
    
    #[msg("Liquidity provider balance insufficient")]
    InsufficientProviderBalance = 3109,
    
    #[msg("Liquidity position minimum duration not met")]
    MinimumDurationNotMet = 3110,
    
    #[msg("Liquidity position already exists")]
    LiquidityPositionAlreadyExists = 3111,
    
    #[msg("Liquidity ratio calculation failed")]
    LiquidityRatioCalculationFailed = 3112,
    
    #[msg("Liquidity token supply insufficient")]
    InsufficientLiquidityTokenSupply = 3113,
    
    #[msg("Liquidity position value calculation error")]
    LiquidityValueCalculationError = 3114,
    
    // Swap-related errors (3200-3299)
    #[msg("Swap amount is zero")]
    SwapAmountZero = 3200,
    
    #[msg("Swap output amount too small")]
    SwapOutputTooSmall = 3201,
    
    #[msg("Swap input amount too large")]
    SwapInputTooLarge = 3202,
    
    #[msg("Swap price impact too high")]
    SwapPriceImpactTooHigh = 3203,
    
    #[msg("Swap route not found")]
    SwapRouteNotFound = 3204,
    
    #[msg("Swap calculation failed")]
    SwapCalculationFailed = 3205,
    
    #[msg("Swap token accounts mismatch")]
    SwapTokenAccountsMismatch = 3206,
    
    #[msg("Swap direction is invalid")]
    InvalidSwapDirection = 3207,
    
    #[msg("Swap minimum output not met")]
    SwapMinimumOutputNotMet = 3208,
    
    #[msg("Swap maximum input exceeded")]
    SwapMaximumInputExceeded = 3209,
    
    #[msg("Swap curve parameters invalid")]
    InvalidSwapCurveParameters = 3210,
    
    #[msg("Swap fee calculation error")]
    SwapFeeCalculationError = 3211,
    
    #[msg("Swap reserves insufficient for trade")]
    InsufficientReservesForSwap = 3212,
    
    #[msg("Swap token balance insufficient")]
    InsufficientSwapTokenBalance = 3213,
    
    #[msg("Swap transaction expired")]
    SwapTransactionExpired = 3214,
    
    // Yield farming errors (3300-3399)
    #[msg("Farm is not initialized")]
    FarmNotInitialized = 3300,
    
    #[msg("Farm is not active")]
    FarmNotActive = 3301,
    
    #[msg("Farm has ended")]
    FarmEnded = 3302,
    
    #[msg("Farm rewards depleted")]
    FarmRewardsDepleted = 3303,
    
    #[msg("Farm staking amount too small")]
    FarmStakingAmountTooSmall = 3304,
    
    #[msg("Farm staking amount too large")]
    FarmStakingAmountTooLarge = 3305,
    
    #[msg("Farm unstaking amount exceeds staked")]
    UnstakingExceedsStaked = 3306,
    
    #[msg("Farm reward calculation failed")]
    FarmRewardCalculationFailed = 3307,
    
    #[msg("Farm user not found")]
    FarmUserNotFound = 3308,
    
    #[msg("Farm cooldown period active")]
    FarmCooldownActive = 3309,
    
    #[msg("Farm minimum staking period not met")]
    FarmMinimumStakingPeriodNotMet = 3310,
    
    #[msg("Farm maximum participants reached")]
    FarmMaximumParticipantsReached = 3311,
    
    #[msg("Farm reward rate invalid")]
    InvalidFarmRewardRate = 3312,
    
    #[msg("Farm token allocation exceeded")]
    FarmTokenAllocationExceeded = 3313,
    
    #[msg("Farm multiplier calculation error")]
    FarmMultiplierCalculationError = 3314,
    
    // Flash loan errors (3400-3499)
    #[msg("Flash loan not supported")]
    FlashLoanNotSupported = 3400,
    
    #[msg("Flash loan amount exceeds limit")]
    FlashLoanAmountExceedsLimit = 3401,
    
    #[msg("Flash loan fee calculation failed")]
    FlashLoanFeeCalculationFailed = 3402,
    
    #[msg("Flash loan not repaid in same transaction")]
    FlashLoanNotRepaidInTransaction = 3403,
    
    #[msg("Flash loan repayment amount insufficient")]
    FlashLoanRepaymentInsufficient = 3404,
    
    #[msg("Flash loan callback failed")]
    FlashLoanCallbackFailed = 3405,
    
    #[msg("Flash loan reentrancy detected")]
    FlashLoanReentrancyDetected = 3406,
    
    #[msg("Flash loan recipient invalid")]
    FlashLoanRecipientInvalid = 3407,
    
    #[msg("Flash loan pool reserves insufficient")]
    FlashLoanInsufficientReserves = 3408,
    
    #[msg("Flash loan maximum concurrent loans exceeded")]
    FlashLoanMaxConcurrentExceeded = 3409,
    
    #[msg("Flash loan execution failed")]
    FlashLoanExecutionFailed = 3410,
    
    #[msg("Flash loan fee payment failed")]
    FlashLoanFeePaymentFailed = 3411,
    
    #[msg("Flash loan security check failed")]
    FlashLoanSecurityCheckFailed = 3412,
    
    #[msg("Flash loan balance verification failed")]
    FlashLoanBalanceVerificationFailed = 3413,
    
    #[msg("Flash loan timeout exceeded")]
    FlashLoanTimeoutExceeded = 3414,
    
    // Oracle and pricing errors (3500-3599)
    #[msg("Oracle price feed not found")]
    OraclePriceFeedNotFound = 3500,
    
    #[msg("Oracle price is stale")]
    OraclePriceStale = 3501,
    
    #[msg("Oracle price deviation too high")]
    OraclePriceDeviationTooHigh = 3502,
    
    #[msg("Oracle aggregation failed")]
    OracleAggregationFailed = 3503,
    
    #[msg("Oracle confidence level too low")]
    OracleConfidenceTooLow = 3504,
    
    #[msg("Oracle update frequency insufficient")]
    OracleUpdateFrequencyInsufficient = 3505,
    
    #[msg("Oracle data source unavailable")]
    OracleDataSourceUnavailable = 3506,
    
    #[msg("Oracle price validation failed")]
    OraclePriceValidationFailed = 3507,
    
    #[msg("Oracle circuit breaker triggered")]
    OracleCircuitBreakerTriggered = 3508,
    
    #[msg("Oracle price range exceeded")]
    OraclePriceRangeExceeded = 3509,
    
    #[msg("Oracle heartbeat timeout")]
    OracleHeartbeatTimeout = 3510,
    
    #[msg("Oracle signature verification failed")]
    OracleSignatureVerificationFailed = 3511,
    
    #[msg("Oracle round data incomplete")]
    OracleRoundDataIncomplete = 3512,
    
    #[msg("Oracle price manipulation detected")]
    OraclePriceManipulationDetected = 3513,
    
    #[msg("Oracle emergency pause active")]
    OracleEmergencyPauseActive = 3514,
    
    // Mathematical calculation errors (3600-3699)
    #[msg("Mathematical overflow detected")]
    MathematicalOverflow = 3600,
    
    #[msg("Mathematical underflow detected")]
    MathematicalUnderflow = 3601,
    
    #[msg("Division by zero")]
    DivisionByZero = 3602,
    
    #[msg("Square root of negative number")]
    SquareRootNegative = 3603,
    
    #[msg("Logarithm of non-positive number")]
    LogarithmNonPositive = 3604,
    
    #[msg("Exponential overflow")]
    ExponentialOverflow = 3605,
    
    #[msg("Precision loss in calculation")]
    PrecisionLoss = 3606,
    
    #[msg("Invalid mathematical parameters")]
    InvalidMathematicalParameters = 3607,
    
    #[msg("Numerical instability detected")]
    NumericalInstability = 3608,
    
    #[msg("Convergence failure in iteration")]
    ConvergenceFailure = 3609,
    
    #[msg("Invalid curve parameters")]
    InvalidCurveParameters = 3610,
    
    #[msg("Invariant calculation failed")]
    InvariantCalculationFailed = 3611,
    
    #[msg("Fixed point arithmetic error")]
    FixedPointArithmeticError = 3612,
    
    #[msg("Floating point precision error")]
    FloatingPointPrecisionError = 3613,
    
    #[msg("Mathematical function domain error")]
    MathematicalDomainError = 3614,
    
    // Access control and permission errors (3700-3799)
    #[msg("Unauthorized access to pool")]
    UnauthorizedPoolAccess = 3700,
    
    #[msg("Admin privileges required")]
    AdminPrivilegesRequired = 3701,
    
    #[msg("Liquidity provider privileges required")]
    LiquidityProviderPrivilegesRequired = 3702,
    
    #[msg("Farm manager privileges required")]
    FarmManagerPrivilegesRequired = 3703,
    
    #[msg("Oracle updater privileges required")]
    OracleUpdaterPrivilegesRequired = 3704,
    
    #[msg("Emergency operator privileges required")]
    EmergencyOperatorPrivilegesRequired = 3705,
    
    #[msg("Governance proposal required for this action")]
    GovernanceProposalRequired = 3706,
    
    #[msg("Multisig approval required")]
    MultisigApprovalRequired = 3707,
    
    #[msg("Time lock delay not satisfied")]
    TimeLockDelayNotSatisfied = 3708,
    
    #[msg("Account is blacklisted")]
    AccountBlacklisted = 3709,
    
    #[msg("Operation requires whitelisted account")]
    WhitelistedAccountRequired = 3710,
    
    #[msg("KYC verification required")]
    KycVerificationRequired = 3711,
    
    #[msg("Geographic restriction violated")]
    GeographicRestrictionViolated = 3712,
    
    #[msg("Account suspended")]
    AccountSuspended = 3713,
    
    #[msg("Daily transaction limit exceeded")]
    DailyTransactionLimitExceeded = 3714,
    
    // Security and validation errors (3800-3899)
    #[msg("Reentrancy attack detected")]
    ReentrancyAttackDetected = 3800,
    
    #[msg("Front-running protection triggered")]
    FrontRunningProtectionTriggered = 3801,
    
    #[msg("MEV protection active")]
    MevProtectionActive = 3802,
    
    #[msg("Sandwich attack detected")]
    SandwichAttackDetected = 3803,
    
    #[msg("Flash loan attack detected")]
    FlashLoanAttackDetected = 3804,
    
    #[msg("Price manipulation attempt detected")]
    PriceManipulationDetected = 3805,
    
    #[msg("Unusual trading pattern detected")]
    UnusualTradingPatternDetected = 3806,
    
    #[msg("Bot activity detected")]
    BotActivityDetected = 3807,
    
    #[msg("Transaction frequency limit exceeded")]
    TransactionFrequencyLimitExceeded = 3808,
    
    #[msg("Suspicious account behavior")]
    SuspiciousAccountBehavior = 3809,
    
    #[msg("Risk threshold exceeded")]
    RiskThresholdExceeded = 3810,
    
    #[msg("Security pause triggered")]
    SecurityPauseTriggered = 3811,
    
    #[msg("Invalid signature")]
    InvalidSignature = 3812,
    
    #[msg("Nonce already used")]
    NonceAlreadyUsed = 3813,
    
    #[msg("Transaction replay detected")]
    TransactionReplayDetected = 3814,
    
    // System and network errors (3900-3999)
    #[msg("System is under maintenance")]
    SystemUnderMaintenance = 3900,
    
    #[msg("Network congestion detected")]
    NetworkCongestionDetected = 3901,
    
    #[msg("System overload protection active")]
    SystemOverloadProtection = 3902,
    
    #[msg("Circuit breaker activated")]
    CircuitBreakerActivated = 3903,
    
    #[msg("Rate limit exceeded")]
    RateLimitExceeded = 3904,
    
    #[msg("Service temporarily unavailable")]
    ServiceTemporarilyUnavailable = 3905,
    
    #[msg("Emergency shutdown active")]
    EmergencyShutdownActive = 3906,
    
    #[msg("Version mismatch detected")]
    VersionMismatchDetected = 3907,
    
    #[msg("Protocol upgrade required")]
    ProtocolUpgradeRequired = 3908,
    
    #[msg("Deprecated function called")]
    DeprecatedFunctionCalled = 3909,
    
    #[msg("Feature not yet implemented")]
    FeatureNotYetImplemented = 3910,
    
    #[msg("Configuration error")]
    ConfigurationError = 3911,
    
    #[msg("Resource allocation failed")]
    ResourceAllocationFailed = 3912,
    
    #[msg("Memory limit exceeded")]
    MemoryLimitExceeded = 3913,
    
    #[msg("Computation limit exceeded")]
    ComputationLimitExceeded = 3914,
    
    #[msg("Storage limit exceeded")]
    StorageLimitExceeded = 3915,
    
    #[msg("Network timeout")]
    NetworkTimeout = 3916,
    
    #[msg("Database connection failed")]
    DatabaseConnectionFailed = 3917,
    
    #[msg("External service unavailable")]
    ExternalServiceUnavailable = 3918,
    
    #[msg("Critical system error")]
    CriticalSystemError = 3919,
}

impl From<FinovaDeFiError> for ProgramError {
    fn from(e: FinovaDeFiError) -> Self {
        ProgramError::Custom(e as u32)
    }
}

impl From<FinovaDeFiError> for Error {
    fn from(error: FinovaDeFiError) -> Self {
        Error::from(ProgramError::from(error))
    }
}

/// Result type alias for DeFi operations
pub type DeFiResult<T = ()> = Result<T, FinovaDeFiError>;

/// Helper macro for error handling with context
#[macro_export]
macro_rules! defi_error_with_context {
    ($error:expr, $context:expr) => {
        Err($error).with_context(|| $context)
    };
}

/// Helper macro for checking conditions and returning errors
#[macro_export]
macro_rules! require_defi {
    ($condition:expr, $error:expr) => {
        if !$condition {
            return Err($error.into());
        }
    };
    ($condition:expr, $error:expr, $message:expr) => {
        if !$condition {
            msg!("DeFi Error: {}", $message);
            return Err($error.into());
        }
    };
}

/// Helper macro for safe mathematical operations
#[macro_export]
macro_rules! safe_math {
    (add, $a:expr, $b:expr) => {
        $a.checked_add($b)
            .ok_or(FinovaDeFiError::MathematicalOverflow)
    };
    (sub, $a:expr, $b:expr) => {
        $a.checked_sub($b)
            .ok_or(FinovaDeFiError::MathematicalUnderflow)
    };
    (mul, $a:expr, $b:expr) => {
        $a.checked_mul($b)
            .ok_or(FinovaDeFiError::MathematicalOverflow)
    };
    (div, $a:expr, $b:expr) => {
        if $b == 0 {
            return Err(FinovaDeFiError::DivisionByZero.into());
        }
        $a.checked_div($b)
            .ok_or(FinovaDeFiError::MathematicalUnderflow)
    };
}

/// Error context for better debugging
#[derive(Debug, Clone)]
pub struct ErrorContext {
    pub operation: String,
    pub pool_id: Option<String>,
    pub user: Option<String>,
    pub amount: Option<u64>,
    pub timestamp: i64,
}

impl ErrorContext {
    pub fn new(operation: impl Into<String>) -> Self {
        Self {
            operation: operation.into(),
            pool_id: None,
            user: None,
            amount: None,
            timestamp: Clock::get().unwrap().unix_timestamp,
        }
    }
    
    pub fn with_pool(mut self, pool_id: impl Into<String>) -> Self {
        self.pool_id = Some(pool_id.into());
        self
    }
    
    pub fn with_user(mut self, user: impl Into<String>) -> Self {
        self.user = Some(user.into());
        self
    }
    
    pub fn with_amount(mut self, amount: u64) -> Self {
        self.amount = Some(amount);
        self
    }
}

/// Trait for error context extension
pub trait ErrorContextExt<T> {
    fn with_context<F>(self, f: F) -> Result<T, Error>
    where
        F: FnOnce() -> ErrorContext;
}

impl<T, E> ErrorContextExt<T> for Result<T, E>
where
    E: Into<Error>,
{
    fn with_context<F>(self, f: F) -> Result<T, Error>
    where
        F: FnOnce() -> ErrorContext,
    {
        self.map_err(|e| {
            let context = f();
            msg!(
                "DeFi Error Context - Operation: {}, Pool: {:?}, User: {:?}, Amount: {:?}, Timestamp: {}",
                context.operation,
                context.pool_id,
                context.user,
                context.amount,
                context.timestamp
            );
            e.into()
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_error_codes_are_unique() {
        // Ensure all error codes are in the correct ranges
        assert_eq!(FinovaDeFiError::PoolNotInitialized as u32, 3000);
        assert_eq!(FinovaDeFiError::InsufficientTokenAAmount as u32, 3100);
        assert_eq!(FinovaDeFiError::SwapAmountZero as u32, 3200);
        assert_eq!(FinovaDeFiError::FarmNotInitialized as u32, 3300);
        assert_eq!(FinovaDeFiError::FlashLoanNotSupported as u32, 3400);
        assert_eq!(FinovaDeFiError::OraclePriceFeedNotFound as u32, 3500);
        assert_eq!(FinovaDeFiError::MathematicalOverflow as u32, 3600);
        assert_eq!(FinovaDeFiError::UnauthorizedPoolAccess as u32, 3700);
        assert_eq!(FinovaDeFiError::ReentrancyAttackDetected as u32, 3800);
        assert_eq!(FinovaDeFiError::SystemUnderMaintenance as u32, 3900);
    }
    
    #[test]
    fn test_error_context_creation() {
        let context = ErrorContext::new("test_operation")
            .with_pool("pool_123")
            .with_user("user_456")
            .with_amount(1000);
            
        assert_eq!(context.operation, "test_operation");
        assert_eq!(context.pool_id, Some("pool_123".to_string()));
        assert_eq!(context.user, Some("user_456".to_string()));
        assert_eq!(context.amount, Some(1000));
    }
}

