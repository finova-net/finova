// programs/finova-token/src/errors.rs

use anchor_lang::prelude::*;

/// Custom error codes for Finova Token Program
/// 
/// This module defines all possible errors that can occur during token operations
/// including minting, burning, staking, unstaking, and reward claims.
/// Each error provides specific context for debugging and user feedback.
#[error_code]
pub enum FinovaTokenError {
    // ========================================
    // MINT & BURN ERRORS (6000-6099)
    // ========================================
    
    /// Mint authority mismatch - caller is not authorized to mint tokens
    #[msg("Unauthorized mint operation: Invalid mint authority")]
    UnauthorizedMint = 6000,
    
    /// Burn authority mismatch - caller is not authorized to burn tokens
    #[msg("Unauthorized burn operation: Invalid burn authority")]
    UnauthorizedBurn = 6001,
    
    /// Attempting to mint more tokens than the maximum supply allows
    #[msg("Mint operation exceeds maximum token supply limit")]
    ExceedsMaxSupply = 6002,
    
    /// Attempting to mint zero or negative amount of tokens
    #[msg("Invalid mint amount: Must be greater than zero")]
    InvalidMintAmount = 6003,
    
    /// Attempting to burn more tokens than available in account
    #[msg("Insufficient balance for burn operation")]
    InsufficientBalanceForBurn = 6004,
    
    /// Attempting to burn zero or negative amount of tokens
    #[msg("Invalid burn amount: Must be greater than zero")]
    InvalidBurnAmount = 6005,
    
    /// Mint is frozen and cannot perform operations
    #[msg("Mint is currently frozen")]
    MintFrozen = 6006,
    
    /// Token account is frozen and cannot perform operations
    #[msg("Token account is frozen")]
    AccountFrozen = 6007,
    
    /// Daily mint limit exceeded for anti-inflation protection
    #[msg("Daily mint limit exceeded")]
    DailyMintLimitExceeded = 6008,
    
    /// Mint cooldown period still active
    #[msg("Mint cooldown period active, please wait")]
    MintCooldownActive = 6009,

    // ========================================
    // STAKING ERRORS (6100-6199)
    // ========================================
    
    /// Attempting to stake zero or negative amount
    #[msg("Invalid stake amount: Must be greater than zero")]
    InvalidStakeAmount = 6100,
    
    /// Insufficient token balance for staking
    #[msg("Insufficient token balance for staking")]
    InsufficientBalanceForStaking = 6101,
    
    /// Attempting to unstake more than currently staked
    #[msg("Insufficient staked balance for unstaking")]
    InsufficientStakedBalance = 6102,
    
    /// Minimum staking period not met
    #[msg("Minimum staking period not met")]
    MinimumStakingPeriodNotMet = 6103,
    
    /// Maximum staking amount exceeded
    #[msg("Maximum staking amount exceeded")]
    MaximumStakingAmountExceeded = 6104,
    
    /// Staking pool is at maximum capacity
    #[msg("Staking pool at maximum capacity")]
    StakingPoolAtCapacity = 6105,
    
    /// Invalid staking tier configuration
    #[msg("Invalid staking tier configuration")]
    InvalidStakingTier = 6106,
    
    /// Staking is currently paused
    #[msg("Staking operations are currently paused")]
    StakingPaused = 6107,
    
    /// Unstaking is currently paused
    #[msg("Unstaking operations are currently paused")]
    UnstakingPaused = 6108,
    
    /// Emergency unstaking not authorized
    #[msg("Emergency unstaking not authorized")]
    EmergencyUnstakingNotAuthorized = 6109,
    
    /// Slashing condition detected
    #[msg("Slashing condition detected for stake")]
    SlashingConditionDetected = 6110,
    
    /// Compound staking limit reached
    #[msg("Compound staking limit reached")]
    CompoundStakingLimitReached = 6111,

    // ========================================
    // REWARD ERRORS (6200-6299)
    // ========================================
    
    /// No rewards available to claim
    #[msg("No rewards available to claim")]
    NoRewardsAvailable = 6200,
    
    /// Reward calculation overflow
    #[msg("Reward calculation resulted in overflow")]
    RewardCalculationOverflow = 6201,
    
    /// Reward pool insufficient funds
    #[msg("Reward pool has insufficient funds")]
    RewardPoolInsufficientFunds = 6202,
    
    /// Reward claim too early
    #[msg("Reward claim too early, minimum interval not met")]
    RewardClaimTooEarly = 6203,
    
    /// Invalid reward rate configuration
    #[msg("Invalid reward rate configuration")]
    InvalidRewardRate = 6204,
    
    /// Reward distribution not initialized
    #[msg("Reward distribution not properly initialized")]
    RewardDistributionNotInitialized = 6205,
    
    /// Duplicate reward claim attempt
    #[msg("Duplicate reward claim detected")]
    DuplicateRewardClaim = 6206,
    
    /// Reward period expired
    #[msg("Reward claiming period has expired")]
    RewardPeriodExpired = 6207,
    
    /// Reward calculation precision loss
    #[msg("Reward calculation precision loss detected")]
    RewardCalculationPrecisionLoss = 6208,
    
    /// Maximum reward per user exceeded
    #[msg("Maximum reward per user exceeded")]
    MaximumRewardPerUserExceeded = 6209,

    // ========================================
    // ACCOUNT VALIDATION ERRORS (6300-6399)
    // ========================================
    
    /// Invalid token account provided
    #[msg("Invalid token account provided")]
    InvalidTokenAccount = 6300,
    
    /// Token account has wrong mint
    #[msg("Token account mint mismatch")]
    TokenAccountMintMismatch = 6301,
    
    /// Token account has wrong owner
    #[msg("Token account owner mismatch")]
    TokenAccountOwnerMismatch = 6302,
    
    /// Stake account not found or invalid
    #[msg("Stake account not found or invalid")]
    InvalidStakeAccount = 6303,
    
    /// Reward pool account invalid
    #[msg("Invalid reward pool account")]
    InvalidRewardPoolAccount = 6304,
    
    /// Treasury account invalid
    #[msg("Invalid treasury account")]
    InvalidTreasuryAccount = 6305,
    
    /// Vault account invalid
    #[msg("Invalid vault account")]
    InvalidVaultAccount = 6306,
    
    /// PDA derivation failed
    #[msg("Program derived address derivation failed")]
    PDADerivationFailed = 6307,
    
    /// Account data size mismatch
    #[msg("Account data size mismatch")]
    AccountDataSizeMismatch = 6308,
    
    /// Account not initialized
    #[msg("Account not properly initialized")]
    AccountNotInitialized = 6309,
    
    /// Account already initialized
    #[msg("Account already initialized")]
    AccountAlreadyInitialized = 6310,

    // ========================================
    // MATHEMATICAL ERRORS (6400-6499)
    // ========================================
    
    /// Mathematical overflow in calculation
    #[msg("Mathematical overflow in calculation")]
    MathematicalOverflow = 6400,
    
    /// Mathematical underflow in calculation
    #[msg("Mathematical underflow in calculation")]
    MathematicalUnderflow = 6401,
    
    /// Division by zero attempted
    #[msg("Division by zero attempted")]
    DivisionByZero = 6402,
    
    /// Square root of negative number
    #[msg("Square root of negative number")]
    SquareRootOfNegative = 6403,
    
    /// Logarithm of non-positive number
    #[msg("Logarithm of non-positive number")]
    LogarithmOfNonPositive = 6404,
    
    /// Exponential calculation overflow
    #[msg("Exponential calculation overflow")]
    ExponentialOverflow = 6405,
    
    /// Invalid mathematical operation
    #[msg("Invalid mathematical operation")]
    InvalidMathematicalOperation = 6406,
    
    /// Precision loss in calculation
    #[msg("Precision loss detected in calculation")]
    PrecisionLoss = 6407,
    
    /// Interest rate calculation error
    #[msg("Interest rate calculation error")]
    InterestRateCalculationError = 6408,
    
    /// Compound interest overflow
    #[msg("Compound interest calculation overflow")]
    CompoundInterestOverflow = 6409,

    // ========================================
    // TIMING ERRORS (6500-6599)
    // ========================================
    
    /// Operation timestamp too early
    #[msg("Operation timestamp too early")]
    TimestampTooEarly = 6500,
    
    /// Operation timestamp too late
    #[msg("Operation timestamp too late")]
    TimestampTooLate = 6501,
    
    /// Invalid time window for operation
    #[msg("Invalid time window for operation")]
    InvalidTimeWindow = 6502,
    
    /// Lock period still active
    #[msg("Lock period still active")]
    LockPeriodActive = 6503,
    
    /// Vesting schedule not met
    #[msg("Vesting schedule requirements not met")]
    VestingScheduleNotMet = 6504,
    
    /// Cooldown period active
    #[msg("Cooldown period active")]
    CooldownPeriodActive = 6505,
    
    /// Time-based limit exceeded
    #[msg("Time-based operation limit exceeded")]
    TimeBasedLimitExceeded = 6506,
    
    /// Invalid epoch for operation
    #[msg("Invalid epoch for operation")]
    InvalidEpoch = 6507,
    
    /// Scheduled operation not ready
    #[msg("Scheduled operation not ready")]
    ScheduledOperationNotReady = 6508,
    
    /// Time synchronization error
    #[msg("Time synchronization error")]
    TimeSynchronizationError = 6509,

    // ========================================
    // GOVERNANCE ERRORS (6600-6699)
    // ========================================
    
    /// Insufficient voting power
    #[msg("Insufficient voting power for operation")]
    InsufficientVotingPower = 6600,
    
    /// Proposal not active
    #[msg("Proposal is not active")]
    ProposalNotActive = 6601,
    
    /// Voting period expired
    #[msg("Voting period has expired")]
    VotingPeriodExpired = 6602,
    
    /// Already voted on proposal
    #[msg("Already voted on this proposal")]
    AlreadyVoted = 6603,
    
    /// Proposal execution failed
    #[msg("Proposal execution failed")]
    ProposalExecutionFailed = 6604,
    
    /// Insufficient quorum
    #[msg("Insufficient quorum for proposal")]
    InsufficientQuorum = 6605,
    
    /// Invalid proposal configuration
    #[msg("Invalid proposal configuration")]
    InvalidProposalConfiguration = 6606,
    
    /// Governance not initialized
    #[msg("Governance system not initialized")]
    GovernanceNotInitialized = 6607,
    
    /// Emergency pause active
    #[msg("Emergency pause is active")]
    EmergencyPauseActive = 6608,
    
    /// Unauthorized governance action
    #[msg("Unauthorized governance action")]
    UnauthorizedGovernanceAction = 6609,

    // ========================================
    // SECURITY ERRORS (6700-6799)
    // ========================================
    
    /// Reentrancy attack detected
    #[msg("Reentrancy attack detected")]
    ReentrancyAttackDetected = 6700,
    
    /// Flash loan attack detected
    #[msg("Flash loan attack detected")]
    FlashLoanAttackDetected = 6701,
    
    /// Suspicious activity pattern
    #[msg("Suspicious activity pattern detected")]
    SuspiciousActivityPattern = 6702,
    
    /// Rate limit exceeded
    #[msg("Rate limit exceeded")]
    RateLimitExceeded = 6703,
    
    /// Invalid signature
    #[msg("Invalid signature provided")]
    InvalidSignature = 6704,
    
    /// Nonce already used
    #[msg("Nonce already used")]
    NonceAlreadyUsed = 6705,
    
    /// Security check failed
    #[msg("Security check failed")]
    SecurityCheckFailed = 6706,
    
    /// Blacklisted account
    #[msg("Account is blacklisted")]
    BlacklistedAccount = 6707,
    
    /// Whitelisting required
    #[msg("Account not whitelisted")]
    WhitelistingRequired = 6708,
    
    /// Multi-signature required
    #[msg("Multi-signature authorization required")]
    MultiSignatureRequired = 6709,
    
    /// Circuit breaker triggered
    #[msg("Circuit breaker triggered")]
    CircuitBreakerTriggered = 6710,

    // ========================================
    // INTEGRATION ERRORS (6800-6899)
    // ========================================
    
    /// Oracle price feed error
    #[msg("Oracle price feed error")]
    OraclePriceFeedError = 6800,
    
    /// Cross-chain bridge error
    #[msg("Cross-chain bridge communication error")]
    CrossChainBridgeError = 6801,
    
    /// External contract call failed
    #[msg("External contract call failed")]
    ExternalContractCallFailed = 6802,
    
    /// API integration error
    #[msg("API integration error")]
    APIIntegrationError = 6803,
    
    /// Data synchronization error
    #[msg("Data synchronization error")]
    DataSynchronizationError = 6804,
    
    /// Protocol version mismatch
    #[msg("Protocol version mismatch")]
    ProtocolVersionMismatch = 6805,
    
    /// Network connectivity issue
    #[msg("Network connectivity issue")]
    NetworkConnectivityIssue = 6806,
    
    /// Service unavailable
    #[msg("Required service unavailable")]
    ServiceUnavailable = 6807,
    
    /// Dependency failure
    #[msg("Critical dependency failure")]
    DependencyFailure = 6808,
    
    /// Integration timeout
    #[msg("Integration operation timeout")]
    IntegrationTimeout = 6809,

    // ========================================
    // CONFIGURATION ERRORS (6900-6999)
    // ========================================
    
    /// Invalid configuration parameter
    #[msg("Invalid configuration parameter")]
    InvalidConfigurationParameter = 6900,
    
    /// Configuration not found
    #[msg("Required configuration not found")]
    ConfigurationNotFound = 6901,
    
    /// Configuration locked
    #[msg("Configuration is locked and cannot be modified")]
    ConfigurationLocked = 6902,
    
    /// Invalid upgrade path
    #[msg("Invalid upgrade path")]
    InvalidUpgradePath = 6903,
    
    /// Feature not enabled
    #[msg("Feature not enabled")]
    FeatureNotEnabled = 6904,
    
    /// Maintenance mode active
    #[msg("Maintenance mode is active")]
    MaintenanceModeActive = 6905,
    
    /// Version compatibility issue
    #[msg("Version compatibility issue")]
    VersionCompatibilityIssue = 6906,
    
    /// Environment mismatch
    #[msg("Environment mismatch detected")]
    EnvironmentMismatch = 6907,
    
    /// Resource exhausted
    #[msg("System resource exhausted")]
    ResourceExhausted = 6908,
    
    /// Capacity limit reached
    #[msg("System capacity limit reached")]
    CapacityLimitReached = 6909,
}

impl FinovaTokenError {
    /// Returns the error code as u32
    pub fn to_code(&self) -> u32 {
        *self as u32
    }
    
    /// Returns whether this error is recoverable
    pub fn is_recoverable(&self) -> bool {
        matches!(self,
            FinovaTokenError::RateLimitExceeded |
            FinovaTokenError::CooldownPeriodActive |
            FinovaTokenError::RewardClaimTooEarly |
            FinovaTokenError::MinimumStakingPeriodNotMet |
            FinovaTokenError::StakingPoolAtCapacity |
            FinovaTokenError::ServiceUnavailable |
            FinovaTokenError::NetworkConnectivityIssue |
            FinovaTokenError::MaintenanceModeActive
        )
    }
    
    /// Returns whether this error is security-related
    pub fn is_security_error(&self) -> bool {
        matches!(self,
            FinovaTokenError::ReentrancyAttackDetected |
            FinovaTokenError::FlashLoanAttackDetected |
            FinovaTokenError::SuspiciousActivityPattern |
            FinovaTokenError::InvalidSignature |
            FinovaTokenError::BlacklistedAccount |
            FinovaTokenError::SecurityCheckFailed |
            FinovaTokenError::CircuitBreakerTriggered
        )
    }
    
    /// Returns whether this error requires admin intervention
    pub fn requires_admin_intervention(&self) -> bool {
        matches!(self,
            FinovaTokenError::EmergencyPauseActive |
            FinovaTokenError::CircuitBreakerTriggered |
            FinovaTokenError::BlacklistedAccount |
            FinovaTokenError::MaintenanceModeActive |
            FinovaTokenError::ResourceExhausted |
            FinovaTokenError::CapacityLimitReached
        )
    }
    
    /// Returns the error category for logging and monitoring
    pub fn get_category(&self) -> &'static str {
        match self {
            FinovaTokenError::UnauthorizedMint..=FinovaTokenError::MintCooldownActive => "mint_burn",
            FinovaTokenError::InvalidStakeAmount..=FinovaTokenError::CompoundStakingLimitReached => "staking",
            FinovaTokenError::NoRewardsAvailable..=FinovaTokenError::MaximumRewardPerUserExceeded => "rewards",
            FinovaTokenError::InvalidTokenAccount..=FinovaTokenError::AccountAlreadyInitialized => "account_validation",
            FinovaTokenError::MathematicalOverflow..=FinovaTokenError::CompoundInterestOverflow => "mathematical",
            FinovaTokenError::TimestampTooEarly..=FinovaTokenError::TimeSynchronizationError => "timing",
            FinovaTokenError::InsufficientVotingPower..=FinovaTokenError::UnauthorizedGovernanceAction => "governance",
            FinovaTokenError::ReentrancyAttackDetected..=FinovaTokenError::CircuitBreakerTriggered => "security",
            FinovaTokenError::OraclePriceFeedError..=FinovaTokenError::IntegrationTimeout => "integration",
            FinovaTokenError::InvalidConfigurationParameter..=FinovaTokenError::CapacityLimitReached => "configuration",
        }
    }
}

/// Helper function to convert Anchor errors to FinovaTokenError
pub fn convert_anchor_error(error: anchor_lang::error::Error) -> FinovaTokenError {
    match error {
        anchor_lang::error::Error::AccountNotInitialized => FinovaTokenError::AccountNotInitialized,
        anchor_lang::error::Error::AccountOwnedByWrongProgram => FinovaTokenError::TokenAccountOwnerMismatch,
        anchor_lang::error::Error::InstructionMissing => FinovaTokenError::InvalidConfigurationParameter,
        anchor_lang::error::Error::InstructionFallbackNotFound => FinovaTokenError::InvalidConfigurationParameter,
        anchor_lang::error::Error::InstructionDidNotDeserialize => FinovaTokenError::AccountDataSizeMismatch,
        anchor_lang::error::Error::InstructionDidNotSerialize => FinovaTokenError::AccountDataSizeMismatch,
        anchor_lang::error::Error::IdlInstructionStub => FinovaTokenError::InvalidConfigurationParameter,
        anchor_lang::error::Error::IdlInstructionInvalidProgram => FinovaTokenError::InvalidConfigurationParameter,
        anchor_lang::error::Error::ConstraintMut => FinovaTokenError::SecurityCheckFailed,
        anchor_lang::error::Error::ConstraintHasOne => FinovaTokenError::SecurityCheckFailed,
        anchor_lang::error::Error::ConstraintSigner => FinovaTokenError::InvalidSignature,
        anchor_lang::error::Error::ConstraintRaw => FinovaTokenError::SecurityCheckFailed,
        anchor_lang::error::Error::ConstraintOwner => FinovaTokenError::TokenAccountOwnerMismatch,
        anchor_lang::error::Error::ConstraintRentExempt => FinovaTokenError::InvalidConfigurationParameter,
        anchor_lang::error::Error::ConstraintSeeds => FinovaTokenError::PDADerivationFailed,
        anchor_lang::error::Error::ConstraintExecutable => FinovaTokenError::SecurityCheckFailed,
        anchor_lang::error::Error::ConstraintState => FinovaTokenError::AccountNotInitialized,
        anchor_lang::error::Error::ConstraintAssociated => FinovaTokenError::InvalidTokenAccount,
        anchor_lang::error::Error::ConstraintAssociatedInit => FinovaTokenError::AccountAlreadyInitialized,
        anchor_lang::error::Error::ConstraintClose => FinovaTokenError::SecurityCheckFailed,
        anchor_lang::error::Error::ConstraintAddress => FinovaTokenError::InvalidTokenAccount,
        anchor_lang::error::Error::ConstraintZero => FinovaTokenError::InvalidConfigurationParameter,
        anchor_lang::error::Error::ConstraintTokenMint => FinovaTokenError::TokenAccountMintMismatch,
        anchor_lang::error::Error::ConstraintTokenOwner => FinovaTokenError::TokenAccountOwnerMismatch,
        anchor_lang::error::Error::ConstraintMintMintAuthority => FinovaTokenError::UnauthorizedMint,
        anchor_lang::error::Error::ConstraintMintFreezeAuthority => FinovaTokenError::UnauthorizedMint,
        anchor_lang::error::Error::ConstraintMintDecimals => FinovaTokenError::InvalidConfigurationParameter,
        anchor_lang::error::Error::ConstraintSpace => FinovaTokenError::AccountDataSizeMismatch,
        _ => FinovaTokenError::SecurityCheckFailed,
    }
}

/// Result type alias for token operations
pub type TokenResult<T> = Result<T, FinovaTokenError>;

/// Helper macro for handling mathematical operations with overflow protection
#[macro_export]
macro_rules! safe_math {
    ($op:expr) => {
        $op.map_err(|_| FinovaTokenError::MathematicalOverflow)?
    };
}

/// Helper macro for timing validations
#[macro_export]
macro_rules! validate_timing {
    ($timestamp:expr, $min:expr, $max:expr) => {
        if $timestamp < $min {
            return Err(FinovaTokenError::TimestampTooEarly.into());
        }
        if $timestamp > $max {
            return Err(FinovaTokenError::TimestampTooLate.into());
        }
    };
}

/// Helper macro for amount validations
#[macro_export]
macro_rules! validate_amount {
    ($amount:expr, $min:expr, $max:expr) => {
        if $amount < $min {
            return Err(FinovaTokenError::InvalidMintAmount.into());
        }
        if $amount > $max {
            return Err(FinovaTokenError::ExceedsMaxSupply.into());
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_codes() {
        assert_eq!(FinovaTokenError::UnauthorizedMint.to_code(), 6000);
        assert_eq!(FinovaTokenError::InvalidStakeAmount.to_code(), 6100);
        assert_eq!(FinovaTokenError::NoRewardsAvailable.to_code(), 6200);
    }

    #[test]
    fn test_error_categories() {
        assert_eq!(FinovaTokenError::UnauthorizedMint.get_category(), "mint_burn");
        assert_eq!(FinovaTokenError::InvalidStakeAmount.get_category(), "staking");
        assert_eq!(FinovaTokenError::NoRewardsAvailable.get_category(), "rewards");
    }

    #[test]
    fn test_recoverable_errors() {
        assert!(FinovaTokenError::RateLimitExceeded.is_recoverable());
        assert!(FinovaTokenError::CooldownPeriodActive.is_recoverable());
        assert!(!FinovaTokenError::UnauthorizedMint.is_recoverable());
    }

    #[test]
    fn test_security_errors() {
        assert!(FinovaTokenError::ReentrancyAttackDetected.is_security_error());
        assert!(FinovaTokenError::FlashLoanAttackDetected.is_security_error());
        assert!(!FinovaTokenError::InvalidMintAmount.is_security_error());
    }

    #[test]
    fn test_admin_intervention_required() {
        assert!(FinovaTokenError::EmergencyPauseActive.requires_admin_intervention());
        assert!(FinovaTokenError::CircuitBreakerTriggered.requires_admin_intervention());
        assert!(!FinovaTokenError::InvalidMintAmount.requires_admin_intervention());
    }
}
