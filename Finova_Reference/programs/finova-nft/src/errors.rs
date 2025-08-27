// programs/finova-nft/src/errors.rs

use anchor_lang::prelude::*;

/// Error codes for Finova NFT program
#[error_code]
pub enum FinovaNftError {
    #[msg("Unauthorized access - insufficient permissions")]
    Unauthorized = 6000,

    #[msg("Invalid collection configuration")]
    InvalidCollectionConfig = 6001,

    #[msg("Collection already exists")]
    CollectionAlreadyExists = 6002,

    #[msg("Collection not found")]
    CollectionNotFound = 6003,

    #[msg("Collection is full - maximum supply reached")]
    CollectionFull = 6004,

    #[msg("Invalid metadata provided")]
    InvalidMetadata = 6005,

    #[msg("Metadata too long - exceeds maximum allowed length")]
    MetadataTooLong = 6006,

    #[msg("NFT not found")]
    NftNotFound = 6007,

    #[msg("NFT already exists")]
    NftAlreadyExists = 6008,

    #[msg("NFT is not transferable")]
    NftNotTransferable = 6009,

    #[msg("NFT is already burned")]
    NftAlreadyBurned = 6010,

    #[msg("Cannot burn this NFT type")]
    NftNotBurnable = 6011,

    #[msg("Special card not found")]
    SpecialCardNotFound = 6012,

    #[msg("Special card already used")]
    SpecialCardAlreadyUsed = 6013,

    #[msg("Special card expired")]
    SpecialCardExpired = 6014,

    #[msg("Invalid special card type")]
    InvalidSpecialCardType = 6015,

    #[msg("Special card cannot be used by this user")]
    SpecialCardUnauthorized = 6016,

    #[msg("Special card effect already active")]
    SpecialCardEffectActive = 6017,

    #[msg("Invalid special card duration")]
    InvalidSpecialCardDuration = 6018,

    #[msg("Special card boost value out of range")]
    InvalidSpecialCardBoost = 6019,

    #[msg("Cannot stack this type of special card")]
    SpecialCardNotStackable = 6020,

    #[msg("Maximum stacked cards limit reached")]
    MaxStackedCardsReached = 6021,

    #[msg("Marketplace not initialized")]
    MarketplaceNotInitialized = 6022,

    #[msg("Invalid marketplace listing")]
    InvalidMarketplaceListing = 6023,

    #[msg("Marketplace listing not found")]
    MarketplaceListingNotFound = 6024,

    #[msg("Marketplace listing already exists")]
    MarketplaceListingExists = 6025,

    #[msg("Invalid listing price")]
    InvalidListingPrice = 6026,

    #[msg("Listing price too low")]
    ListingPriceTooLow = 6027,

    #[msg("Listing price too high")]
    ListingPriceTooHigh = 6028,

    #[msg("Cannot buy own listing")]
    CannotBuyOwnListing = 6029,

    #[msg("Insufficient funds for purchase")]
    InsufficientFunds = 6030,

    #[msg("Marketplace listing expired")]
    MarketplaceListingExpired = 6031,

    #[msg("Marketplace is paused")]
    MarketplacePaused = 6032,

    #[msg("Invalid marketplace fee")]
    InvalidMarketplaceFee = 6033,

    #[msg("Marketplace fee too high")]
    MarketplaceFeeTooHigh = 6034,

    #[msg("Invalid royalty configuration")]
    InvalidRoyaltyConfig = 6035,

    #[msg("Royalty percentage too high")]
    RoyaltyTooHigh = 6036,

    #[msg("Invalid creator share distribution")]
    InvalidCreatorShare = 6037,

    #[msg("Creator verification failed")]
    CreatorVerificationFailed = 6038,

    #[msg("Invalid badge configuration")]
    InvalidBadgeConfig = 6039,

    #[msg("Badge already owned by user")]
    BadgeAlreadyOwned = 6040,

    #[msg("Badge requirements not met")]
    BadgeRequirementsNotMet = 6041,

    #[msg("Badge is not upgradeable")]
    BadgeNotUpgradeable = 6042,

    #[msg("Insufficient achievement points for badge")]
    InsufficientAchievementPoints = 6043,

    #[msg("Invalid achievement NFT type")]
    InvalidAchievementType = 6044,

    #[msg("Achievement already claimed")]
    AchievementAlreadyClaimed = 6045,

    #[msg("Achievement not available")]
    AchievementNotAvailable = 6046,

    #[msg("Achievement requirements not met")]
    AchievementRequirementsNotMet = 6047,

    #[msg("Invalid timestamp - cannot be in the future")]
    InvalidTimestamp = 6048,

    #[msg("Invalid duration - must be positive")]
    InvalidDuration = 6049,

    #[msg("Operation not allowed during maintenance")]
    MaintenanceMode = 6050,

    #[msg("Rate limit exceeded - too many operations")]
    RateLimitExceeded = 6051,

    #[msg("Invalid signature provided")]
    InvalidSignature = 6052,

    #[msg("Signature verification failed")]
    SignatureVerificationFailed = 6053,

    #[msg("Invalid proof of ownership")]
    InvalidProofOfOwnership = 6054,

    #[msg("NFT ownership verification failed")]
    OwnershipVerificationFailed = 6055,

    #[msg("Invalid account state")]
    InvalidAccountState = 6056,

    #[msg("Account is frozen")]
    AccountFrozen = 6057,

    #[msg("Account is not active")]
    AccountNotActive = 6058,

    #[msg("Insufficient permissions for operation")]
    InsufficientPermissions = 6059,

    #[msg("Invalid program authority")]
    InvalidProgramAuthority = 6060,

    #[msg("Authority verification failed")]
    AuthorityVerificationFailed = 6061,

    #[msg("Invalid mint authority")]
    InvalidMintAuthority = 6062,

    #[msg("Invalid update authority")]
    InvalidUpdateAuthority = 6063,

    #[msg("Invalid freeze authority")]
    InvalidFreezeAuthority = 6064,

    #[msg("Token account not found")]
    TokenAccountNotFound = 6065,

    #[msg("Invalid token account owner")]
    InvalidTokenAccountOwner = 6066,

    #[msg("Token account is frozen")]
    TokenAccountFrozen = 6067,

    #[msg("Insufficient token balance")]
    InsufficientTokenBalance = 6068,

    #[msg("Token transfer failed")]
    TokenTransferFailed = 6069,

    #[msg("Token mint failed")]
    TokenMintFailed = 6070,

    #[msg("Token burn failed")]
    TokenBurnFailed = 6071,

    #[msg("Invalid mint configuration")]
    InvalidMintConfig = 6072,

    #[msg("Mint is disabled")]
    MintDisabled = 6073,

    #[msg("Burn is disabled")]
    BurnDisabled = 6074,

    #[msg("Transfer is disabled")]
    TransferDisabled = 6075,

    #[msg("Invalid rarity configuration")]
    InvalidRarityConfig = 6076,

    #[msg("Rarity distribution error")]
    RarityDistributionError = 6077,

    #[msg("Invalid card synergy configuration")]
    InvalidCardSynergy = 6078,

    #[msg("Card synergy requirements not met")]
    CardSynergyRequirementsNotMet = 6079,

    #[msg("Maximum synergy bonus already achieved")]
    MaxSynergyBonusReached = 6080,

    #[msg("Invalid evolution path")]
    InvalidEvolutionPath = 6081,

    #[msg("Evolution requirements not met")]
    EvolutionRequirementsNotMet = 6082,

    #[msg("NFT cannot be evolved")]
    NftNotEvolvable = 6083,

    #[msg("Evolution materials insufficient")]
    InsufficientEvolutionMaterials = 6084,

    #[msg("Invalid breeding configuration")]
    InvalidBreedingConfig = 6085,

    #[msg("Breeding not available")]
    BreedingNotAvailable = 6086,

    #[msg("Breeding cooldown active")]
    BreedingCooldownActive = 6087,

    #[msg("Incompatible NFTs for breeding")]
    IncompatibleNftsForBreeding = 6088,

    #[msg("Maximum breeding attempts reached")]
    MaxBreedingAttemptsReached = 6089,

    #[msg("Invalid staking pool")]
    InvalidStakingPool = 6090,

    #[msg("NFT staking not allowed")]
    NftStakingNotAllowed = 6091,

    #[msg("NFT already staked")]
    NftAlreadyStaked = 6092,

    #[msg("NFT not staked")]
    NftNotStaked = 6093,

    #[msg("Staking pool is full")]
    StakingPoolFull = 6094,

    #[msg("Staking lock period not expired")]
    StakingLockPeriodActive = 6095,

    #[msg("Invalid staking rewards calculation")]
    InvalidStakingRewardsCalculation = 6096,

    #[msg("Staking rewards not available")]
    StakingRewardsNotAvailable = 6097,

    #[msg("Invalid fractional ownership")]
    InvalidFractionalOwnership = 6098,

    #[msg("Fractional shares not available")]
    FractionalSharesNotAvailable = 6099,

    #[msg("Invalid share percentage")]
    InvalidSharePercentage = 6100,

    #[msg("Insufficient shares for operation")]
    InsufficientShares = 6101,

    #[msg("Share transfer not allowed")]
    ShareTransferNotAllowed = 6102,

    #[msg("Invalid governance proposal")]
    InvalidGovernanceProposal = 6103,

    #[msg("Governance proposal not found")]
    GovernanceProposalNotFound = 6104,

    #[msg("Governance voting not active")]
    GovernanceVotingNotActive = 6105,

    #[msg("Already voted on proposal")]
    AlreadyVotedOnProposal = 6106,

    #[msg("Insufficient voting power")]
    InsufficientVotingPower = 6107,

    #[msg("Governance proposal execution failed")]
    GovernanceProposalExecutionFailed = 6108,

    #[msg("Invalid cross-chain bridge")]
    InvalidCrossChainBridge = 6109,

    #[msg("Cross-chain transfer not supported")]
    CrossChainTransferNotSupported = 6110,

    #[msg("Bridge is paused")]
    BridgePaused = 6111,

    #[msg("Invalid destination chain")]
    InvalidDestinationChain = 6112,

    #[msg("Bridge fee too high")]
    BridgeFeeTooHigh = 6113,

    #[msg("Cross-chain verification failed")]
    CrossChainVerificationFailed = 6114,

    #[msg("Invalid oracle price")]
    InvalidOraclePrice = 6115,

    #[msg("Oracle price too stale")]
    OraclePriceTooStale = 6116,

    #[msg("Oracle not available")]
    OracleNotAvailable = 6117,

    #[msg("Price manipulation detected")]
    PriceManipulationDetected = 6118,

    #[msg("Invalid slippage tolerance")]
    InvalidSlippageTolerance = 6119,

    #[msg("Slippage tolerance exceeded")]
    SlippageToleranceExceeded = 6120,

    #[msg("Arithmetic overflow in calculation")]
    ArithmeticOverflow = 6121,

    #[msg("Arithmetic underflow in calculation")]
    ArithmeticUnderflow = 6122,

    #[msg("Division by zero error")]
    DivisionByZero = 6123,

    #[msg("Invalid mathematical operation")]
    InvalidMathematicalOperation = 6124,

    #[msg("Precision loss in calculation")]
    PrecisionLoss = 6125,

    #[msg("Number too large for operation")]
    NumberTooLarge = 6126,

    #[msg("Number too small for operation")]
    NumberTooSmall = 6127,

    #[msg("Invalid percentage value")]
    InvalidPercentage = 6128,

    #[msg("System overloaded - try again later")]
    SystemOverloaded = 6129,

    #[msg("Network congestion detected")]
    NetworkCongestion = 6130,

    #[msg("Transaction deadline exceeded")]
    TransactionDeadlineExceeded = 6131,

    #[msg("Invalid transaction parameters")]
    InvalidTransactionParameters = 6132,

    #[msg("Transaction simulation failed")]
    TransactionSimulationFailed = 6133,

    #[msg("Gas limit exceeded")]
    GasLimitExceeded = 6134,

    #[msg("Invalid gas price")]
    InvalidGasPrice = 6135,

    #[msg("Nonce mismatch")]
    NonceMismatch = 6136,

    #[msg("Invalid chain ID")]
    InvalidChainId = 6137,

    #[msg("Replay attack detected")]
    ReplayAttackDetected = 6138,

    #[msg("Emergency pause activated")]
    EmergencyPause = 6139,

    #[msg("Circuit breaker triggered")]
    CircuitBreakerTriggered = 6140,

    #[msg("Invalid admin operation")]
    InvalidAdminOperation = 6141,

    #[msg("Admin privileges required")]
    AdminPrivilegesRequired = 6142,

    #[msg("Multi-signature threshold not met")]
    MultisigThresholdNotMet = 6143,

    #[msg("Invalid multi-signature configuration")]
    InvalidMultisigConfig = 6144,

    #[msg("Proposal timelock not expired")]
    ProposalTimelockNotExpired = 6145,

    #[msg("Invalid timelock duration")]
    InvalidTimelockDuration = 6146,

    #[msg("Security audit required")]
    SecurityAuditRequired = 6147,

    #[msg("Version incompatibility detected")]
    VersionIncompatibility = 6148,

    #[msg("Upgrade not authorized")]
    UpgradeNotAuthorized = 6149,

    #[msg("Migration in progress")]
    MigrationInProgress = 6150,

    #[msg("Data corruption detected")]
    DataCorruption = 6151,

    #[msg("Checksum validation failed")]
    ChecksumValidationFailed = 6152,

    #[msg("Invalid data format")]
    InvalidDataFormat = 6153,

    #[msg("Data version mismatch")]
    DataVersionMismatch = 6154,

    #[msg("Backup operation failed")]
    BackupOperationFailed = 6155,

    #[msg("Restore operation failed")]
    RestoreOperationFailed = 6156,

    #[msg("Invalid backup format")]
    InvalidBackupFormat = 6157,

    #[msg("Backup not found")]
    BackupNotFound = 6158,

    #[msg("Recovery mode active")]
    RecoveryModeActive = 6159,

    #[msg("Unknown error occurred")]
    UnknownError = 6160,
}

/// Helper macros for error handling
#[macro_export]
macro_rules! require_auth {
    ($condition:expr, $authority:expr) => {
        if !$condition {
            msg!("Authorization failed for authority: {:?}", $authority);
            return Err(FinovaNftError::Unauthorized.into());
        }
    };
}

#[macro_export]
macro_rules! require_valid_metadata {
    ($metadata:expr, $max_length:expr) => {
        if $metadata.len() > $max_length {
            msg!("Metadata length {} exceeds maximum {}", $metadata.len(), $max_length);
            return Err(FinovaNftError::MetadataTooLong.into());
        }
        if $metadata.is_empty() {
            msg!("Metadata cannot be empty");
            return Err(FinovaNftError::InvalidMetadata.into());
        }
    };
}

#[macro_export]
macro_rules! require_valid_timestamp {
    ($timestamp:expr) => {
        let current_timestamp = Clock::get()?.unix_timestamp;
        if $timestamp > current_timestamp {
            msg!("Timestamp {} cannot be in the future", $timestamp);
            return Err(FinovaNftError::InvalidTimestamp.into());
        }
    };
}

#[macro_export]
macro_rules! require_not_expired {
    ($expiry:expr) => {
        let current_timestamp = Clock::get()?.unix_timestamp;
        if $expiry < current_timestamp {
            msg!("Item expired at {}, current time {}", $expiry, current_timestamp);
            return Err(FinovaNftError::SpecialCardExpired.into());
        }
    };
}

#[macro_export]
macro_rules! require_sufficient_balance {
    ($balance:expr, $required:expr) => {
        if $balance < $required {
            msg!("Insufficient balance: {} < {}", $balance, $required);
            return Err(FinovaNftError::InsufficientFunds.into());
        }
    };
}

#[macro_export]
macro_rules! require_valid_price {
    ($price:expr, $min:expr, $max:expr) => {
        if $price < $min {
            msg!("Price {} below minimum {}", $price, $min);
            return Err(FinovaNftError::ListingPriceTooLow.into());
        }
        if $price > $max {
            msg!("Price {} above maximum {}", $price, $max);
            return Err(FinovaNftError::ListingPriceTooHigh.into());
        }
    };
}

#[macro_export]
macro_rules! require_not_paused {
    ($paused:expr) => {
        if $paused {
            msg!("Operation not allowed while system is paused");
            return Err(FinovaNftError::MarketplacePaused.into());
        }
    };
}

#[macro_export]
macro_rules! require_valid_percentage {
    ($percentage:expr, $max:expr) => {
        if $percentage > $max {
            msg!("Percentage {} exceeds maximum {}", $percentage, $max);
            return Err(FinovaNftError::InvalidPercentage.into());
        }
    };
}

#[macro_export]
macro_rules! checked_add {
    ($a:expr, $b:expr) => {
        $a.checked_add($b)
            .ok_or(FinovaNftError::ArithmeticOverflow)?
    };
}

#[macro_export]
macro_rules! checked_sub {
    ($a:expr, $b:expr) => {
        $a.checked_sub($b)
            .ok_or(FinovaNftError::ArithmeticUnderflow)?
    };
}

#[macro_export]
macro_rules! checked_mul {
    ($a:expr, $b:expr) => {
        $a.checked_mul($b)
            .ok_or(FinovaNftError::ArithmeticOverflow)?
    };
}

#[macro_export]
macro_rules! checked_div {
    ($a:expr, $b:expr) => {
        if $b == 0 {
            return Err(FinovaNftError::DivisionByZero.into());
        }
        $a.checked_div($b)
            .ok_or(FinovaNftError::ArithmeticOverflow)?
    };
}

/// Error result type for convenience
pub type FinovaNftResult<T = ()> = std::result::Result<T, FinovaNftError>;

/// Custom error handling trait
pub trait ErrorContext<T> {
    fn with_context(self, context: &str) -> Result<T>;
}

impl<T> ErrorContext<T> for FinovaNftResult<T> {
    fn with_context(self, context: &str) -> Result<T> {
        self.map_err(|e| {
            msg!("Error context: {}", context);
            ProgramError::from(e)
        })
    }
}

/// Validation helpers
pub mod validation {
    use super::*;

    pub fn validate_mint_authority(authority: &Pubkey, expected: &Pubkey) -> FinovaNftResult {
        if authority != expected {
            msg!("Invalid mint authority: expected {}, got {}", expected, authority);
            return Err(FinovaNftError::InvalidMintAuthority);
        }
        Ok(())
    }

    pub fn validate_update_authority(authority: &Pubkey, expected: &Pubkey) -> FinovaNftResult {
        if authority != expected {
            msg!("Invalid update authority: expected {}, got {}", expected, authority);
            return Err(FinovaNftError::InvalidUpdateAuthority);
        }
        Ok(())
    }

    pub fn validate_collection_size(current: u64, max: u64) -> FinovaNftResult {
        if current >= max {
            msg!("Collection full: {}/{}", current, max);
            return Err(FinovaNftError::CollectionFull);
        }
        Ok(())
    }

    pub fn validate_rarity_distribution(rarities: &[u16]) -> FinovaNftResult {
        let total: u32 = rarities.iter().map(|&x| x as u32).sum();
        if total != 10000 {
            msg!("Rarity distribution must sum to 10000 (100%), got {}", total);
            return Err(FinovaNftError::RarityDistributionError);
        }
        Ok(())
    }

    pub fn validate_royalty_percentage(percentage: u16) -> FinovaNftResult {
        if percentage > 1000 {
            msg!("Royalty percentage {} exceeds maximum 10%", percentage);
            return Err(FinovaNftError::RoyaltyTooHigh);
        }
        Ok(())
    }

    pub fn validate_marketplace_fee(fee: u16) -> FinovaNftResult {
        if fee > 500 {
            msg!("Marketplace fee {} exceeds maximum 5%", fee);
            return Err(FinovaNftError::MarketplaceFeeTooHigh);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_codes() {
        assert_eq!(FinovaNftError::Unauthorized as u32, 6000);
        assert_eq!(FinovaNftError::InvalidCollectionConfig as u32, 6001);
        assert_eq!(FinovaNftError::UnknownError as u32, 6160);
    }

    #[test]
    fn test_validation_helpers() {
        let key1 = Pubkey::new_unique();
        let key2 = Pubkey::new_unique();
        
        assert!(validation::validate_mint_authority(&key1, &key1).is_ok());
        assert!(validation::validate_mint_authority(&key1, &key2).is_err());
        
        assert!(validation::validate_collection_size(99, 100).is_ok());
        assert!(validation::validate_collection_size(100, 100).is_err());
        
        assert!(validation::validate_rarity_distribution(&[5000, 3000, 2000]).is_ok());
        assert!(validation::validate_rarity_distribution(&[5000, 3000, 1000]).is_err());
        
        assert!(validation::validate_royalty_percentage(500).is_ok());
        assert!(validation::validate_royalty_percentage(1500).is_err());
        
        assert!(validation::validate_marketplace_fee(250).is_ok());
        assert!(validation::validate_marketplace_fee(600).is_err());
    }
}
