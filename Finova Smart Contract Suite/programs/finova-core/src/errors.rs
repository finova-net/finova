// programs/finova-core/src/errors.rs

use anchor_lang::prelude::*;

/// Finova Core Program Error Codes
/// 
/// This module defines all custom error types used throughout the Finova Core program.
/// Error codes are organized by functionality for better debugging and maintenance.
#[error_code]
pub enum FinovaError {
    // ============================================================================
    // General Errors (6000-6099)
    // ============================================================================
    
    #[msg("Mathematical overflow occurred during calculation")]
    MathOverflow = 6000,
    
    #[msg("Mathematical underflow occurred during calculation")]
    MathUnderflow = 6001,
    
    #[msg("Division by zero attempted")]
    DivisionByZero = 6002,
    
    #[msg("Invalid timestamp provided")]
    InvalidTimestamp = 6003,
    
    #[msg("Account not initialized properly")]
    AccountNotInitialized = 6004,
    
    #[msg("Account already initialized")]
    AccountAlreadyInitialized = 6005,
    
    #[msg("Insufficient account balance")]
    InsufficientBalance = 6006,
    
    #[msg("Invalid account owner")]
    InvalidAccountOwner = 6007,
    
    #[msg("Operation not authorized for this user")]
    Unauthorized = 6008,
    
    #[msg("Program is currently paused")]
    ProgramPaused = 6009,
    
    // ============================================================================
    // User Management Errors (6100-6199)
    // ============================================================================
    
    #[msg("User account not found")]
    UserNotFound = 6100,
    
    #[msg("User already exists")]
    UserAlreadyExists = 6101,
    
    #[msg("Invalid user level")]
    InvalidUserLevel = 6102,
    
    #[msg("User account is suspended")]
    UserSuspended = 6103,
    
    #[msg("User has not completed KYC verification")]
    KycNotVerified = 6104,
    
    #[msg("User KYC verification failed")]
    KycVerificationFailed = 6105,
    
    #[msg("Invalid user status")]
    InvalidUserStatus = 6106,
    
    #[msg("User registration limit exceeded")]
    UserRegistrationLimitExceeded = 6107,
    
    #[msg("Invalid referral code")]
    InvalidReferralCode = 6108,
    
    #[msg("Cannot refer yourself")]
    SelfReferralNotAllowed = 6109,
    
    // ============================================================================
    // Mining System Errors (6200-6299)
    // ============================================================================
    
    #[msg("Mining session not found")]
    MiningSessionNotFound = 6200,
    
    #[msg("Mining session already active")]
    MiningSessionAlreadyActive = 6201,
    
    #[msg("Mining session has expired")]
    MiningSessionExpired = 6202,
    
    #[msg("Mining rate calculation failed")]
    MiningRateCalculationFailed = 6203,
    
    #[msg("Mining rewards already claimed")]
    MiningRewardsAlreadyClaimed = 6204,
    
    #[msg("No mining rewards available")]
    NoMiningRewardsAvailable = 6205,
    
    #[msg("Mining daily limit exceeded")]
    MiningDailyLimitExceeded = 6206,
    
    #[msg("Mining phase transition not allowed")]
    MiningPhaseTransitionNotAllowed = 6207,
    
    #[msg("Invalid mining multiplier")]
    InvalidMiningMultiplier = 6208,
    
    #[msg("Mining cooldown period active")]
    MiningCooldownActive = 6209,
    
    #[msg("Mining regression factor calculation failed")]
    MiningRegressionCalculationFailed = 6210,
    
    #[msg("Mining bot detection triggered")]
    MiningBotDetected = 6211,
    
    // ============================================================================
    // XP System Errors (6300-6399)
    // ============================================================================
    
    #[msg("XP activity not found")]
    XpActivityNotFound = 6300,
    
    #[msg("XP daily limit exceeded for this activity")]
    XpDailyLimitExceeded = 6301,
    
    #[msg("Invalid XP activity type")]
    InvalidXpActivityType = 6302,
    
    #[msg("XP calculation failed")]
    XpCalculationFailed = 6303,
    
    #[msg("XP level calculation overflow")]
    XpLevelCalculationOverflow = 6304,
    
    #[msg("Invalid platform for XP activity")]
    InvalidXpPlatform = 6305,
    
    #[msg("XP quality score out of range")]
    XpQualityScoreOutOfRange = 6306,
    
    #[msg("XP streak bonus calculation failed")]
    XpStreakBonusCalculationFailed = 6307,
    
    #[msg("XP activity cooldown active")]
    XpActivityCooldownActive = 6308,
    
    #[msg("XP milestone already claimed")]
    XpMilestoneAlreadyClaimed = 6309,
    
    // ============================================================================
    // Referral System Errors (6400-6499)
    // ============================================================================
    
    #[msg("Referral network not found")]
    ReferralNetworkNotFound = 6400,
    
    #[msg("Referral code already used")]
    ReferralCodeAlreadyUsed = 6401,
    
    #[msg("Referral code expired")]
    ReferralCodeExpired = 6402,
    
    #[msg("Maximum referral depth exceeded")]
    MaxReferralDepthExceeded = 6403,
    
    #[msg("Referral points calculation failed")]
    ReferralPointsCalculationFailed = 6404,
    
    #[msg("Invalid referral tier")]
    InvalidReferralTier = 6405,
    
    #[msg("Referral network quality too low")]
    ReferralNetworkQualityTooLow = 6406,
    
    #[msg("Referral bonus already claimed")]
    ReferralBonusAlreadyClaimed = 6407,
    
    #[msg("Circular referral detected")]
    CircularReferralDetected = 6408,
    
    #[msg("Referral network size limit exceeded")]
    ReferralNetworkSizeLimitExceeded = 6409,
    
    // ============================================================================
    // Staking System Errors (6500-6599)
    // ============================================================================
    
    #[msg("Staking account not found")]
    StakingAccountNotFound = 6500,
    
    #[msg("Insufficient staking balance")]
    InsufficientStakingBalance = 6501,
    
    #[msg("Staking amount below minimum")]
    StakingAmountBelowMinimum = 6502,
    
    #[msg("Staking amount above maximum")]
    StakingAmountAboveMaximum = 6503,
    
    #[msg("Staking lockup period not met")]
    StakingLockupPeriodNotMet = 6504,
    
    #[msg("Staking rewards calculation failed")]
    StakingRewardsCalculationFailed = 6505,
    
    #[msg("Staking tier calculation failed")]
    StakingTierCalculationFailed = 6506,
    
    #[msg("Staked tokens are locked")]
    StakedTokensLocked = 6507,
    
    #[msg("Staking pool is full")]
    StakingPoolFull = 6508,
    
    #[msg("Staking rewards pool depleted")]
    StakingRewardsPoolDepleted = 6509,
    
    // ============================================================================
    // Guild System Errors (6600-6699)
    // ============================================================================
    
    #[msg("Guild not found")]
    GuildNotFound = 6600,
    
    #[msg("Guild already exists")]
    GuildAlreadyExists = 6601,
    
    #[msg("Guild is full")]
    GuildFull = 6602,
    
    #[msg("User is not a guild member")]
    NotGuildMember = 6603,
    
    #[msg("User is already in a guild")]
    AlreadyInGuild = 6604,
    
    #[msg("Insufficient guild permissions")]
    InsufficientGuildPermissions = 6605,
    
    #[msg("Guild master cannot leave guild")]
    GuildMasterCannotLeave = 6606,
    
    #[msg("Guild competition not found")]
    GuildCompetitionNotFound = 6607,
    
    #[msg("Guild competition already ended")]
    GuildCompetitionEnded = 6608,
    
    #[msg("Guild treasury insufficient")]
    GuildTreasuryInsufficient = 6609,
    
    // ============================================================================
    // Rewards System Errors (6700-6799)
    // ============================================================================
    
    #[msg("Reward pool not found")]
    RewardPoolNotFound = 6700,
    
    #[msg("Reward pool depleted")]
    RewardPoolDepleted = 6701,
    
    #[msg("Reward calculation failed")]
    RewardCalculationFailed = 6702,
    
    #[msg("Reward already claimed")]
    RewardAlreadyClaimed = 6703,
    
    #[msg("Reward claim period expired")]
    RewardClaimPeriodExpired = 6704,
    
    #[msg("Insufficient reward pool balance")]
    InsufficientRewardPoolBalance = 6705,
    
    #[msg("Invalid reward type")]
    InvalidRewardType = 6706,
    
    #[msg("Reward multiplier out of range")]
    RewardMultiplierOutOfRange = 6707,
    
    #[msg("Daily reward limit exceeded")]
    DailyRewardLimitExceeded = 6708,
    
    #[msg("Reward quality threshold not met")]
    RewardQualityThresholdNotMet = 6709,
    
    // ============================================================================
    // Anti-Bot System Errors (6800-6899)
    // ============================================================================
    
    #[msg("Bot behavior detected")]
    BotBehaviorDetected = 6800,
    
    #[msg("Suspicious activity pattern detected")]
    SuspiciousActivityPattern = 6801,
    
    #[msg("Human verification required")]
    HumanVerificationRequired = 6802,
    
    #[msg("Activity rate limit exceeded")]
    ActivityRateLimitExceeded = 6803,
    
    #[msg("Device fingerprint mismatch")]
    DeviceFingerprintMismatch = 6804,
    
    #[msg("Behavioral analysis failed")]
    BehavioralAnalysisFailed = 6805,
    
    #[msg("Multiple account violation")]
    MultipleAccountViolation = 6806,
    
    #[msg("Automated behavior detected")]
    AutomatedBehaviorDetected = 6807,
    
    #[msg("Captcha verification failed")]
    CaptchaVerificationFailed = 6808,
    
    #[msg("Account flagged for review")]
    AccountFlaggedForReview = 6809,
    
    // ============================================================================
    // Quality Assessment Errors (6900-6999)
    // ============================================================================
    
    #[msg("Content quality assessment failed")]
    ContentQualityAssessmentFailed = 6900,
    
    #[msg("Content quality score too low")]
    ContentQualityScoreTooLow = 6901,
    
    #[msg("Content originality check failed")]
    ContentOriginalityCheckFailed = 6902,
    
    #[msg("Content contains inappropriate material")]
    ContentInappropriate = 6903,
    
    #[msg("Content platform mismatch")]
    ContentPlatformMismatch = 6904,
    
    #[msg("Content engagement prediction failed")]
    ContentEngagementPredictionFailed = 6905,
    
    #[msg("Content brand safety check failed")]
    ContentBrandSafetyCheckFailed = 6906,
    
    #[msg("Content AI generation detected")]
    ContentAiGenerationDetected = 6907,
    
    #[msg("Content quality analysis timeout")]
    ContentQualityAnalysisTimeout = 6908,
    
    #[msg("Content metadata validation failed")]
    ContentMetadataValidationFailed = 6909,
    
    // ============================================================================
    // Governance Errors (7000-7099)
    // ============================================================================
    
    #[msg("Proposal not found")]
    ProposalNotFound = 7000,
    
    #[msg("Proposal already exists")]
    ProposalAlreadyExists = 7001,
    
    #[msg("Proposal voting period ended")]
    ProposalVotingPeriodEnded = 7002,
    
    #[msg("Proposal voting period not started")]
    ProposalVotingPeriodNotStarted = 7003,
    
    #[msg("Insufficient voting power")]
    InsufficientVotingPower = 7004,
    
    #[msg("Already voted on this proposal")]
    AlreadyVoted = 7005,
    
    #[msg("Proposal execution failed")]
    ProposalExecutionFailed = 7006,
    
    #[msg("Proposal quorum not met")]
    ProposalQuorumNotMet = 7007,
    
    #[msg("Invalid proposal type")]
    InvalidProposalType = 7008,
    
    #[msg("Proposal threshold not met")]
    ProposalThresholdNotMet = 7009,
    
    // ============================================================================
    // Network & Economic Errors (7100-7199)
    // ============================================================================
    
    #[msg("Network regression calculation failed")]
    NetworkRegressionCalculationFailed = 7100,
    
    #[msg("Economic model violation detected")]
    EconomicModelViolation = 7101,
    
    #[msg("Token supply limit exceeded")]
    TokenSupplyLimitExceeded = 7102,
    
    #[msg("Inflation rate out of bounds")]
    InflationRateOutOfBounds = 7103,
    
    #[msg("Deflation mechanism failed")]
    DeflationMechanismFailed = 7104,
    
    #[msg("Economic equilibrium disrupted")]
    EconomicEquilibriumDisrupted = 7105,
    
    #[msg("Network effect calculation failed")]
    NetworkEffectCalculationFailed = 7106,
    
    #[msg("Value transfer limit exceeded")]
    ValueTransferLimitExceeded = 7107,
    
    #[msg("Economic incentive misalignment")]
    EconomicIncentiveMisalignment = 7108,
    
    #[msg("Market maker reserves insufficient")]
    MarketMakerReservesInsufficient = 7109,
    
    // ============================================================================
    // Integration & External Service Errors (7200-7299)
    // ============================================================================
    
    #[msg("External service unavailable")]
    ExternalServiceUnavailable = 7200,
    
    #[msg("Social platform integration failed")]
    SocialPlatformIntegrationFailed = 7201,
    
    #[msg("API rate limit exceeded")]
    ApiRateLimitExceeded = 7202,
    
    #[msg("Authentication token expired")]
    AuthenticationTokenExpired = 7203,
    
    #[msg("External service response invalid")]
    ExternalServiceResponseInvalid = 7204,
    
    #[msg("Webhook signature verification failed")]
    WebhookSignatureVerificationFailed = 7205,
    
    #[msg("Data synchronization failed")]
    DataSynchronizationFailed = 7206,
    
    #[msg("External service timeout")]
    ExternalServiceTimeout = 7207,
    
    #[msg("Integration configuration invalid")]
    IntegrationConfigurationInvalid = 7208,
    
    #[msg("Third party service error")]
    ThirdPartyServiceError = 7209,
    
    // ============================================================================
    // Security & Compliance Errors (7300-7399)
    // ============================================================================
    
    #[msg("Security violation detected")]
    SecurityViolationDetected = 7300,
    
    #[msg("Compliance check failed")]
    ComplianceCheckFailed = 7301,
    
    #[msg("Regulatory requirement not met")]
    RegulatoryRequirementNotMet = 7302,
    
    #[msg("AML check failed")]
    AmlCheckFailed = 7303,
    
    #[msg("Sanctions screening failed")]
    SanctionsScreeningFailed = 7304,
    
    #[msg("Data privacy violation")]
    DataPrivacyViolation = 7305,
    
    #[msg("Audit trail incomplete")]
    AuditTrailIncomplete = 7306,
    
    #[msg("Security token validation failed")]
    SecurityTokenValidationFailed = 7307,
    
    #[msg("Encryption verification failed")]
    EncryptionVerificationFailed = 7308,
    
    #[msg("Access control violation")]
    AccessControlViolation = 7309,
    
    // ============================================================================
    // NFT & Special Card Errors (7400-7499)
    // ============================================================================
    
    #[msg("NFT not found")]
    NftNotFound = 7400,
    
    #[msg("NFT already used")]
    NftAlreadyUsed = 7401,
    
    #[msg("NFT not owned by user")]
    NftNotOwnedByUser = 7402,
    
    #[msg("Special card effect expired")]
    SpecialCardEffectExpired = 7403,
    
    #[msg("Special card usage limit exceeded")]
    SpecialCardUsageLimitExceeded = 7404,
    
    #[msg("Invalid special card type")]
    InvalidSpecialCardType = 7405,
    
    #[msg("Special card metadata invalid")]
    SpecialCardMetadataInvalid = 7406,
    
    #[msg("NFT marketplace transaction failed")]
    NftMarketplaceTransactionFailed = 7407,
    
    #[msg("NFT royalty calculation failed")]
    NftRoyaltyCalculationFailed = 7408,
    
    #[msg("Special card synergy limit exceeded")]
    SpecialCardSynergyLimitExceeded = 7409,
    
    // ============================================================================
    // System Configuration Errors (7500-7599)
    // ============================================================================
    
    #[msg("Invalid system configuration")]
    InvalidSystemConfiguration = 7500,
    
    #[msg("Configuration parameter out of range")]
    ConfigurationParameterOutOfRange = 7501,
    
    #[msg("System upgrade required")]
    SystemUpgradeRequired = 7502,
    
    #[msg("Maintenance mode active")]
    MaintenanceModeActive = 7503,
    
    #[msg("Feature not enabled")]
    FeatureNotEnabled = 7504,
    
    #[msg("System capacity exceeded")]
    SystemCapacityExceeded = 7505,
    
    #[msg("Configuration validation failed")]
    ConfigurationValidationFailed = 7506,
    
    #[msg("System dependencies not met")]
    SystemDependenciesNotMet = 7507,
    
    #[msg("Version compatibility issue")]
    VersionCompatibilityIssue = 7508,
    
    #[msg("System initialization failed")]
    SystemInitializationFailed = 7509,
}

/// Result type alias for Finova operations
pub type FinovaResult<T = ()> = Result<T, FinovaError>;

/// Helper trait for converting various error types to FinovaError
pub trait IntoFinovaError<T> {
    fn into_finova_error(self) -> FinovaResult<T>;
}

impl<T> IntoFinovaError<T> for Result<T, std::num::TryFromIntError> {
    fn into_finova_error(self) -> FinovaResult<T> {
        self.map_err(|_| FinovaError::MathOverflow)
    }
}

impl<T> IntoFinovaError<T> for Option<T> {
    fn into_finova_error(self) -> FinovaResult<T> {
        self.ok_or(FinovaError::AccountNotInitialized)
    }
}

/// Macro for easy error propagation with context
#[macro_export]
macro_rules! require_finova {
    ($condition:expr, $error:expr) => {
        if !$condition {
            return Err($error.into());
        }
    };
    ($condition:expr, $error:expr, $msg:expr) => {
        if !$condition {
            msg!("Requirement failed: {}", $msg);
            return Err($error.into());
        }
    };
}

/// Macro for safe mathematical operations
#[macro_export]
macro_rules! safe_math {
    ($a:expr + $b:expr) => {
        $a.checked_add($b).ok_or(FinovaError::MathOverflow)
    };
    ($a:expr - $b:expr) => {
        $a.checked_sub($b).ok_or(FinovaError::MathUnderflow)
    };
    ($a:expr * $b:expr) => {
        $a.checked_mul($b).ok_or(FinovaError::MathOverflow)
    };
    ($a:expr / $b:expr) => {
        if $b == 0 {
            Err(FinovaError::DivisionByZero)
        } else {
            Ok($a / $b)
        }
    };
}

/// Utility functions for error handling
impl FinovaError {
    /// Get error severity level
    pub fn severity(&self) -> ErrorSeverity {
        match self {
            // Critical errors that require immediate attention
            FinovaError::MathOverflow
            | FinovaError::MathUnderflow
            | FinovaError::DivisionByZero
            | FinovaError::SecurityViolationDetected
            | FinovaError::BotBehaviorDetected
            | FinovaError::EconomicModelViolation => ErrorSeverity::Critical,
            
            // High severity errors
            FinovaError::UserSuspended
            | FinovaError::KycVerificationFailed
            | FinovaError::Unauthorized
            | FinovaError::ComplianceCheckFailed
            | FinovaError::AmlCheckFailed => ErrorSeverity::High,
            
            // Medium severity errors
            FinovaError::MiningSessionExpired
            | FinovaError::XpDailyLimitExceeded
            | FinovaError::InsufficientBalance
            | FinovaError::ReferralCodeExpired => ErrorSeverity::Medium,
            
            // Low severity errors (user input validation, etc.)
            _ => ErrorSeverity::Low,
        }
    }
    
    /// Check if error is recoverable
    pub fn is_recoverable(&self) -> bool {
        match self {
            FinovaError::MathOverflow
            | FinovaError::MathUnderflow
            | FinovaError::DivisionByZero
            | FinovaError::SecurityViolationDetected
            | FinovaError::AccountNotInitialized => false,
            _ => true,
        }
    }
    
    /// Get error category for logging and monitoring
    pub fn category(&self) -> ErrorCategory {
        match self {
            FinovaError::MathOverflow
            | FinovaError::MathUnderflow
            | FinovaError::DivisionByZero
            | FinovaError::InvalidTimestamp => ErrorCategory::System,
            
            FinovaError::UserNotFound
            | FinovaError::UserAlreadyExists
            | FinovaError::InvalidUserLevel
            | FinovaError::UserSuspended
            | FinovaError::KycNotVerified
            | FinovaError::KycVerificationFailed => ErrorCategory::User,
            
            FinovaError::MiningSessionNotFound
            | FinovaError::MiningSessionAlreadyActive
            | FinovaError::MiningSessionExpired
            | FinovaError::MiningRateCalculationFailed => ErrorCategory::Mining,
            
            FinovaError::XpActivityNotFound
            | FinovaError::XpDailyLimitExceeded
            | FinovaError::InvalidXpActivityType
            | FinovaError::XpCalculationFailed => ErrorCategory::Experience,
            
            FinovaError::ReferralNetworkNotFound
            | FinovaError::ReferralCodeAlreadyUsed
            | FinovaError::ReferralCodeExpired
            | FinovaError::MaxReferralDepthExceeded => ErrorCategory::Referral,
            
            FinovaError::StakingAccountNotFound
            | FinovaError::InsufficientStakingBalance
            | FinovaError::StakingAmountBelowMinimum => ErrorCategory::Staking,
            
            FinovaError::GuildNotFound
            | FinovaError::GuildAlreadyExists
            | FinovaError::GuildFull
            | FinovaError::NotGuildMember => ErrorCategory::Guild,
            
            FinovaError::SecurityViolationDetected
            | FinovaError::BotBehaviorDetected
            | FinovaError::SuspiciousActivityPattern
            | FinovaError::HumanVerificationRequired => ErrorCategory::Security,
            
            _ => ErrorCategory::General,
        }
    }
}

/// Error severity levels for monitoring and alerting
#[derive(Debug, Clone, PartialEq)]
pub enum ErrorSeverity {
    Critical,
    High,
    Medium,
    Low,
}

/// Error categories for classification and routing
#[derive(Debug, Clone, PartialEq)]
pub enum ErrorCategory {
    System,
    User,
    Mining,
    Experience,
    Referral,
    Staking,
    Guild,
    Security,
    Network,
    Integration,
    Compliance,
    General,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_severity_classification() {
        assert_eq!(FinovaError::MathOverflow.severity(), ErrorSeverity::Critical);
        assert_eq!(FinovaError::UserSuspended.severity(), ErrorSeverity::High);
        assert_eq!(FinovaError::MiningSessionExpired.severity(), ErrorSeverity::Medium);
        assert_eq!(FinovaError::UserNotFound.severity(), ErrorSeverity::Low);
    }

    #[test]
    fn test_error_recoverability() {
        assert!(!FinovaError::MathOverflow.is_recoverable());
        assert!(!FinovaError::SecurityViolationDetected.is_recoverable());
        assert!(FinovaError::UserNotFound.is_recoverable());
        assert!(FinovaError::MiningSessionExpired.is_recoverable());
    }

    #[test]
    fn test_error_categorization() {
        assert_eq!(FinovaError::MathOverflow.category(), ErrorCategory::System);
        assert_eq!(FinovaError::UserNotFound.category(), ErrorCategory::User);
        assert_eq!(FinovaError::MiningSessionExpired.category(), ErrorCategory::Mining);
        assert_eq!(FinovaError::XpActivityNotFound.category(), ErrorCategory::Experience);
        assert_eq!(FinovaError::ReferralNetworkNotFound.category(), ErrorCategory::Referral);
        assert_eq!(FinovaError::StakingAccountNotFound.category(), ErrorCategory::Staking);
        assert_eq!(FinovaError::GuildNotFound.category(), ErrorCategory::Guild);
        assert_eq!(FinovaError::BotBehaviorDetected.category(), ErrorCategory::Security);
    }

    #[test]
    fn test_safe_math_macros() {
        // Test addition overflow
        let result = safe_math!(u64::MAX + 1);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), FinovaError::MathOverflow);

        // Test subtraction underflow  
        let result = safe_math!(0u64 - 1);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), FinovaError::MathUnderflow);

        // Test division by zero
        let result = safe_math!(10u64 / 0);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), FinovaError::DivisionByZero);

        // Test successful operations
        assert_eq!(safe_math!(5u64 + 3).unwrap(), 8);
        assert_eq!(safe_math!(10u64 - 3).unwrap(), 7);
        assert_eq!(safe_math!(6u64 * 7).unwrap(), 42);
        assert_eq!(safe_math!(15u64 / 3).unwrap(), 5);
    }
}
