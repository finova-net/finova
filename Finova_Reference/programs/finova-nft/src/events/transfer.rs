// programs/finova-nft/src/events/transfer.rs 

use anchor_lang::prelude::*;

/// Event emitted when an NFT is transferred between accounts
#[event]
pub struct NftTransferred {
    /// The mint address of the transferred NFT
    pub mint: Pubkey,
    /// The previous owner of the NFT
    pub from: Pubkey,
    /// The new owner of the NFT
    pub to: Pubkey,
    /// The token account that was transferred from
    pub from_token_account: Pubkey,
    /// The token account that was transferred to
    pub to_token_account: Pubkey,
    /// Timestamp of the transfer
    pub timestamp: i64,
    /// The collection this NFT belongs to (if any)
    pub collection: Option<Pubkey>,
    /// Transfer type (sale, gift, marketplace, etc.)
    pub transfer_type: TransferType,
    /// Optional price if this was a sale
    pub price: Option<u64>,
    /// Optional marketplace fee if this was a marketplace sale
    pub marketplace_fee: Option<u64>,
    /// Optional royalty amount paid to creator
    pub royalty_amount: Option<u64>,
    /// Transaction signature for reference
    pub transaction_signature: String,
}

/// Event emitted when an NFT transfer is initiated (for escrow scenarios)
#[event]
pub struct NftTransferInitiated {
    /// The mint address of the NFT being transferred
    pub mint: Pubkey,
    /// The current owner initiating the transfer
    pub from: Pubkey,
    /// The intended recipient
    pub to: Pubkey,
    /// The escrow account holding the NFT
    pub escrow_account: Pubkey,
    /// Timestamp when transfer was initiated
    pub timestamp: i64,
    /// Type of transfer being initiated
    pub transfer_type: TransferType,
    /// Expiration time for the transfer (if applicable)
    pub expires_at: Option<i64>,
    /// Conditions that must be met for transfer completion
    pub conditions: Vec<TransferCondition>,
}

/// Event emitted when an NFT transfer is completed from escrow
#[event]
pub struct NftTransferCompleted {
    /// The mint address of the transferred NFT
    pub mint: Pubkey,
    /// The original owner
    pub from: Pubkey,
    /// The final recipient
    pub to: Pubkey,
    /// The escrow account that held the NFT
    pub escrow_account: Pubkey,
    /// Timestamp of completion
    pub timestamp: i64,
    /// Final transfer details
    pub transfer_details: TransferDetails,
    /// Whether all conditions were met
    pub conditions_met: bool,
}

/// Event emitted when an NFT transfer is cancelled
#[event]
pub struct NftTransferCancelled {
    /// The mint address of the NFT
    pub mint: Pubkey,
    /// The owner who cancelled the transfer
    pub owner: Pubkey,
    /// The intended recipient
    pub intended_recipient: Pubkey,
    /// The escrow account (if applicable)
    pub escrow_account: Option<Pubkey>,
    /// Timestamp of cancellation
    pub timestamp: i64,
    /// Reason for cancellation
    pub cancellation_reason: CancellationReason,
    /// Whether any fees were refunded
    pub fees_refunded: bool,
}

/// Event emitted when NFT ownership is verified during transfer
#[event]
pub struct NftOwnershipVerified {
    /// The mint address of the NFT
    pub mint: Pubkey,
    /// The verified owner
    pub owner: Pubkey,
    /// The token account holding the NFT
    pub token_account: Pubkey,
    /// Timestamp of verification
    pub timestamp: i64,
    /// Verification method used
    pub verification_method: VerificationMethod,
    /// Whether the verification was successful
    pub verification_successful: bool,
}

/// Event emitted when NFT metadata is updated during transfer
#[event]
pub struct NftMetadataUpdatedOnTransfer {
    /// The mint address of the NFT
    pub mint: Pubkey,
    /// The new owner
    pub new_owner: Pubkey,
    /// Fields that were updated
    pub updated_fields: Vec<String>,
    /// Timestamp of update
    pub timestamp: i64,
    /// Whether update was automatic or manual
    pub update_type: MetadataUpdateType,
}

/// Event emitted when transfer fees are calculated and applied
#[event]
pub struct TransferFeesCalculated {
    /// The mint address of the NFT
    pub mint: Pubkey,
    /// The transfer amount (if sale)
    pub transfer_amount: Option<u64>,
    /// Platform fee amount
    pub platform_fee: u64,
    /// Royalty fee amount
    pub royalty_fee: u64,
    /// Gas fee amount
    pub gas_fee: u64,
    /// Total fees
    pub total_fees: u64,
    /// Fee recipients and amounts
    pub fee_distribution: Vec<FeeRecipient>,
    /// Timestamp of calculation
    pub timestamp: i64,
}

/// Event emitted when an NFT transfer fails
#[event]
pub struct NftTransferFailed {
    /// The mint address of the NFT
    pub mint: Pubkey,
    /// The attempted sender
    pub from: Pubkey,
    /// The attempted recipient
    pub to: Pubkey,
    /// Timestamp of failure
    pub timestamp: i64,
    /// Reason for failure
    pub failure_reason: TransferFailureReason,
    /// Error code
    pub error_code: u32,
    /// Additional error details
    pub error_details: String,
    /// Whether retry is possible
    pub retry_possible: bool,
}

/// Event emitted when batch NFT transfers are processed
#[event]
pub struct BatchNftTransfer {
    /// List of NFT mints being transferred
    pub mints: Vec<Pubkey>,
    /// The sender of all NFTs
    pub from: Pubkey,
    /// The recipient of all NFTs
    pub to: Pubkey,
    /// Number of successful transfers
    pub successful_transfers: u32,
    /// Number of failed transfers
    pub failed_transfers: u32,
    /// Total fees paid for batch
    pub total_fees: u64,
    /// Timestamp of batch operation
    pub timestamp: i64,
    /// Batch operation ID
    pub batch_id: String,
}

/// Types of NFT transfers
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum TransferType {
    /// Direct transfer between users
    Direct,
    /// Sale through marketplace
    Sale,
    /// Gift transfer
    Gift,
    /// Auction completion
    Auction,
    /// Rental transfer
    Rental,
    /// Staking operation
    Staking,
    /// Guild transfer
    Guild,
    /// Reward distribution
    Reward,
    /// Migration or upgrade
    Migration,
    /// Burn operation
    Burn,
}

/// Conditions that must be met for transfer
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub enum TransferCondition {
    /// Payment must be received
    PaymentReceived { amount: u64, token: Pubkey },
    /// Time condition must be met
    TimeCondition { after: i64, before: Option<i64> },
    /// Approval from specific account
    ApprovalRequired { approver: Pubkey },
    /// KYC verification required
    KycVerified,
    /// Minimum holding period
    HoldingPeriod { duration: i64 },
    /// Custom condition
    Custom { condition_id: String, params: Vec<u8> },
}

/// Transfer completion details
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct TransferDetails {
    /// Final sale price (if applicable)
    pub sale_price: Option<u64>,
    /// Currency used for payment
    pub payment_token: Option<Pubkey>,
    /// Platform fees paid
    pub platform_fees: u64,
    /// Royalties paid
    pub royalties: u64,
    /// Gas fees
    pub gas_fees: u64,
    /// Any bonus rewards given
    pub bonus_rewards: Option<u64>,
}

/// Reasons for transfer cancellation
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub enum CancellationReason {
    /// User requested cancellation
    UserRequested,
    /// Insufficient funds
    InsufficientFunds,
    /// Expired deadline
    Expired,
    /// Failed verification
    VerificationFailed,
    /// Technical error
    TechnicalError,
    /// Regulatory compliance
    Compliance,
    /// Market conditions
    MarketConditions,
    /// Platform maintenance
    Maintenance,
}

/// Methods of ownership verification
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub enum VerificationMethod {
    /// On-chain signature verification
    OnChainSignature,
    /// Token account balance check
    TokenBalance,
    /// Multi-signature verification
    MultiSig,
    /// Biometric verification
    Biometric,
    /// KYC document verification
    KycDocument,
    /// Time-based verification
    TimeBased,
}

/// Types of metadata updates during transfer
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub enum MetadataUpdateType {
    /// Automatic update by system
    Automatic,
    /// Manual update by user
    Manual,
    /// Update triggered by smart contract
    ContractTriggered,
    /// Update required by regulation
    RegulatoryRequired,
}

/// Fee recipient information
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct FeeRecipient {
    /// Recipient's public key
    pub recipient: Pubkey,
    /// Amount to be paid to recipient
    pub amount: u64,
    /// Type of fee
    pub fee_type: FeeType,
    /// Description of the fee
    pub description: String,
}

/// Types of fees in transfers
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub enum FeeType {
    /// Platform transaction fee
    PlatformFee,
    /// Creator royalty
    Royalty,
    /// Gas/network fee
    GasFee,
    /// Marketplace fee
    MarketplaceFee,
    /// Insurance fee
    InsuranceFee,
    /// Regulatory fee
    RegulatoryFee,
    /// Custom fee
    Custom { fee_name: String },
}

/// Reasons for transfer failure
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub enum TransferFailureReason {
    /// Insufficient balance
    InsufficientBalance,
    /// Invalid recipient
    InvalidRecipient,
    /// NFT is locked/staked
    NftLocked,
    /// Transfer not authorized
    Unauthorized,
    /// Network congestion
    NetworkCongestion,
    /// Smart contract error
    ContractError,
    /// Metadata validation failed
    MetadataValidationFailed,
    /// Royalty calculation error
    RoyaltyCalculationError,
    /// KYC requirement not met
    KycNotMet,
    /// Regulatory restriction
    RegulatoryRestriction,
    /// Technical system error
    SystemError,
}

/// Helper functions for transfer events
impl NftTransferred {
    /// Create a new NFT transferred event
    pub fn new(
        mint: Pubkey,
        from: Pubkey,
        to: Pubkey,
        from_token_account: Pubkey,
        to_token_account: Pubkey,
        collection: Option<Pubkey>,
        transfer_type: TransferType,
        price: Option<u64>,
        marketplace_fee: Option<u64>,
        royalty_amount: Option<u64>,
        transaction_signature: String,
    ) -> Self {
        Self {
            mint,
            from,
            to,
            from_token_account,
            to_token_account,
            timestamp: Clock::get().unwrap().unix_timestamp,
            collection,
            transfer_type,
            price,
            marketplace_fee,
            royalty_amount,
            transaction_signature,
        }
    }

    /// Check if this was a sale transaction
    pub fn is_sale(&self) -> bool {
        matches!(self.transfer_type, TransferType::Sale | TransferType::Auction)
    }

    /// Get total value transacted (price + fees)
    pub fn total_value(&self) -> u64 {
        let price = self.price.unwrap_or(0);
        let marketplace_fee = self.marketplace_fee.unwrap_or(0);
        let royalty = self.royalty_amount.unwrap_or(0);
        price + marketplace_fee + royalty
    }

    /// Check if transfer involved marketplace
    pub fn is_marketplace_transfer(&self) -> bool {
        self.marketplace_fee.is_some()
    }

    /// Check if royalties were paid
    pub fn has_royalties(&self) -> bool {
        self.royalty_amount.is_some() && self.royalty_amount.unwrap() > 0
    }
}

impl NftTransferInitiated {
    /// Create a new transfer initiation event
    pub fn new(
        mint: Pubkey,
        from: Pubkey,
        to: Pubkey,
        escrow_account: Pubkey,
        transfer_type: TransferType,
        expires_at: Option<i64>,
        conditions: Vec<TransferCondition>,
    ) -> Self {
        Self {
            mint,
            from,
            to,
            escrow_account,
            timestamp: Clock::get().unwrap().unix_timestamp,
            transfer_type,
            expires_at,
            conditions,
        }
    }

    /// Check if transfer has expired
    pub fn is_expired(&self) -> bool {
        if let Some(expires_at) = self.expires_at {
            Clock::get().unwrap().unix_timestamp > expires_at
        } else {
            false
        }
    }

    /// Get time remaining until expiration
    pub fn time_remaining(&self) -> Option<i64> {
        self.expires_at.map(|expires_at| {
            expires_at - Clock::get().unwrap().unix_timestamp
        })
    }

    /// Check if all conditions are time-based
    pub fn has_time_conditions(&self) -> bool {
        self.conditions.iter().any(|condition| {
            matches!(condition, TransferCondition::TimeCondition { .. })
        })
    }
}

impl TransferFeesCalculated {
    /// Create a new fee calculation event
    pub fn new(
        mint: Pubkey,
        transfer_amount: Option<u64>,
        platform_fee: u64,
        royalty_fee: u64,
        gas_fee: u64,
        fee_distribution: Vec<FeeRecipient>,
    ) -> Self {
        let total_fees = platform_fee + royalty_fee + gas_fee;
        
        Self {
            mint,
            transfer_amount,
            platform_fee,
            royalty_fee,
            gas_fee,
            total_fees,
            fee_distribution,
            timestamp: Clock::get().unwrap().unix_timestamp,
        }
    }

    /// Calculate fee percentage of total transaction
    pub fn fee_percentage(&self) -> Option<f64> {
        self.transfer_amount.map(|amount| {
            if amount > 0 {
                (self.total_fees as f64 / amount as f64) * 100.0
            } else {
                0.0
            }
        })
    }

    /// Get platform fee percentage
    pub fn platform_fee_percentage(&self) -> Option<f64> {
        self.transfer_amount.map(|amount| {
            if amount > 0 {
                (self.platform_fee as f64 / amount as f64) * 100.0
            } else {
                0.0
            }
        })
    }

    /// Get royalty percentage
    pub fn royalty_percentage(&self) -> Option<f64> {
        self.transfer_amount.map(|amount| {
            if amount > 0 {
                (self.royalty_fee as f64 / amount as f64) * 100.0
            } else {
                0.0
            }
        })
    }
}

impl BatchNftTransfer {
    /// Create a new batch transfer event
    pub fn new(
        mints: Vec<Pubkey>,
        from: Pubkey,
        to: Pubkey,
        successful_transfers: u32,
        failed_transfers: u32,
        total_fees: u64,
        batch_id: String,
    ) -> Self {
        Self {
            mints,
            from,
            to,
            successful_transfers,
            failed_transfers,
            total_fees,
            timestamp: Clock::get().unwrap().unix_timestamp,
            batch_id,
        }
    }

    /// Get total number of NFTs in batch
    pub fn total_nfts(&self) -> u32 {
        self.mints.len() as u32
    }

    /// Get success rate as percentage
    pub fn success_rate(&self) -> f64 {
        let total = self.total_nfts();
        if total > 0 {
            (self.successful_transfers as f64 / total as f64) * 100.0
        } else {
            0.0
        }
    }

    /// Check if batch was fully successful
    pub fn is_fully_successful(&self) -> bool {
        self.failed_transfers == 0 && self.successful_transfers == self.total_nfts()
    }

    /// Get average fee per NFT
    pub fn average_fee_per_nft(&self) -> u64 {
        let total = self.total_nfts();
        if total > 0 {
            self.total_fees / total as u64
        } else {
            0
        }
    }
}

/// Event emission helper functions
pub mod event_emitters {
    use super::*;

    /// Emit NFT transferred event
    pub fn emit_nft_transferred(
        mint: Pubkey,
        from: Pubkey,
        to: Pubkey,
        from_token_account: Pubkey,
        to_token_account: Pubkey,
        collection: Option<Pubkey>,
        transfer_type: TransferType,
        price: Option<u64>,
        marketplace_fee: Option<u64>,
        royalty_amount: Option<u64>,
        transaction_signature: String,
    ) {
        emit!(NftTransferred::new(
            mint,
            from,
            to,
            from_token_account,
            to_token_account,
            collection,
            transfer_type,
            price,
            marketplace_fee,
            royalty_amount,
            transaction_signature,
        ));
    }

    /// Emit transfer initiated event
    pub fn emit_transfer_initiated(
        mint: Pubkey,
        from: Pubkey,
        to: Pubkey,
        escrow_account: Pubkey,
        transfer_type: TransferType,
        expires_at: Option<i64>,
        conditions: Vec<TransferCondition>,
    ) {
        emit!(NftTransferInitiated::new(
            mint,
            from,
            to,
            escrow_account,
            transfer_type,
            expires_at,
            conditions,
        ));
    }

    /// Emit transfer completed event
    pub fn emit_transfer_completed(
        mint: Pubkey,
        from: Pubkey,
        to: Pubkey,
        escrow_account: Pubkey,
        transfer_details: TransferDetails,
        conditions_met: bool,
    ) {
        emit!(NftTransferCompleted {
            mint,
            from,
            to,
            escrow_account,
            timestamp: Clock::get().unwrap().unix_timestamp,
            transfer_details,
            conditions_met,
        });
    }

    /// Emit transfer cancelled event
    pub fn emit_transfer_cancelled(
        mint: Pubkey,
        owner: Pubkey,
        intended_recipient: Pubkey,
        escrow_account: Option<Pubkey>,
        cancellation_reason: CancellationReason,
        fees_refunded: bool,
    ) {
        emit!(NftTransferCancelled {
            mint,
            owner,
            intended_recipient,
            escrow_account,
            timestamp: Clock::get().unwrap().unix_timestamp,
            cancellation_reason,
            fees_refunded,
        });
    }

    /// Emit ownership verified event
    pub fn emit_ownership_verified(
        mint: Pubkey,
        owner: Pubkey,
        token_account: Pubkey,
        verification_method: VerificationMethod,
        verification_successful: bool,
    ) {
        emit!(NftOwnershipVerified {
            mint,
            owner,
            token_account,
            timestamp: Clock::get().unwrap().unix_timestamp,
            verification_method,
            verification_successful,
        });
    }

    /// Emit transfer fees calculated event
    pub fn emit_fees_calculated(
        mint: Pubkey,
        transfer_amount: Option<u64>,
        platform_fee: u64,
        royalty_fee: u64,
        gas_fee: u64,
        fee_distribution: Vec<FeeRecipient>,
    ) {
        emit!(TransferFeesCalculated::new(
            mint,
            transfer_amount,
            platform_fee,
            royalty_fee,
            gas_fee,
            fee_distribution,
        ));
    }

    /// Emit transfer failed event
    pub fn emit_transfer_failed(
        mint: Pubkey,
        from: Pubkey,
        to: Pubkey,
        failure_reason: TransferFailureReason,
        error_code: u32,
        error_details: String,
        retry_possible: bool,
    ) {
        emit!(NftTransferFailed {
            mint,
            from,
            to,
            timestamp: Clock::get().unwrap().unix_timestamp,
            failure_reason,
            error_code,
            error_details,
            retry_possible,
        });
    }

    /// Emit batch transfer event
    pub fn emit_batch_transfer(
        mints: Vec<Pubkey>,
        from: Pubkey,
        to: Pubkey,
        successful_transfers: u32,
        failed_transfers: u32,
        total_fees: u64,
        batch_id: String,
    ) {
        emit!(BatchNftTransfer::new(
            mints,
            from,
            to,
            successful_transfers,
            failed_transfers,
            total_fees,
            batch_id,
        ));
    }
}

/// Event validation and processing helpers
pub mod event_validators {
    use super::*;

    /// Validate transfer event data
    pub fn validate_transfer_event(
        mint: &Pubkey,
        from: &Pubkey,
        to: &Pubkey,
        transfer_type: &TransferType,
    ) -> Result<()> {
        // Validate addresses are not zero
        require!(*mint != Pubkey::default(), crate::errors::FinovaNftError::InvalidMint);
        require!(*from != Pubkey::default(), crate::errors::FinovaNftError::InvalidOwner);
        require!(*to != Pubkey::default(), crate::errors::FinovaNftError::InvalidRecipient);
        
        // Validate from and to are different (unless it's a special case)
        if !matches!(transfer_type, TransferType::Staking | TransferType::Migration) {
            require!(*from != *to, crate::errors::FinovaNftError::SelfTransfer);
        }

        Ok(())
    }

    /// Validate fee calculation data
    pub fn validate_fee_calculation(
        transfer_amount: Option<u64>,
        platform_fee: u64,
        royalty_fee: u64,
        gas_fee: u64,
    ) -> Result<()> {
        // Validate fees don't exceed transfer amount
        if let Some(amount) = transfer_amount {
            let total_fees = platform_fee + royalty_fee + gas_fee;
            require!(
                total_fees <= amount,
                crate::errors::FinovaNftError::ExcessiveFees
            );
        }

        Ok(())
    }

    /// Validate batch transfer data
    pub fn validate_batch_transfer(
        mints: &[Pubkey],
        successful_transfers: u32,
        failed_transfers: u32,
    ) -> Result<()> {
        let total_mints = mints.len() as u32;
        let total_processed = successful_transfers + failed_transfers;
        
        require!(
            total_processed == total_mints,
            crate::errors::FinovaNftError::BatchCountMismatch
        );

        // Validate no duplicate mints
        let mut unique_mints = std::collections::HashSet::new();
        for mint in mints {
            require!(
                unique_mints.insert(*mint),
                crate::errors::FinovaNftError::DuplicateMint
            );
        }

        Ok(())
    }
}
