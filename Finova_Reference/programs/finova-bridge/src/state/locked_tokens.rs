// programs/finova-bridge/src/state/locked_tokens.rs

use anchor_lang::prelude::*;
use crate::constants::*;

/// Represents tokens locked in the bridge for cross-chain transfer
#[account]
#[derive(Default)]
pub struct LockedTokens {
    /// Unique identifier for this lock operation
    pub lock_id: u64,
    
    /// The user who locked the tokens
    pub user: Pubkey,
    
    /// The token mint being locked
    pub token_mint: Pubkey,
    
    /// Amount of tokens locked
    pub amount: u64,
    
    /// Destination chain identifier
    pub destination_chain: u8,
    
    /// Destination address on the target chain
    pub destination_address: [u8; 64],
    
    /// Source chain transaction hash
    pub source_tx_hash: [u8; 32],
    
    /// Timestamp when tokens were locked
    pub locked_at: i64,
    
    /// Timestamp when tokens were unlocked (0 if still locked)
    pub unlocked_at: i64,
    
    /// Status of the lock operation
    pub status: LockStatus,
    
    /// Fee paid for the bridge operation
    pub bridge_fee: u64,
    
    /// Validator signatures for unlock operation
    pub validator_signatures: Vec<ValidatorSignature>,
    
    /// Number of confirmations required
    pub required_confirmations: u8,
    
    /// Current number of confirmations
    pub current_confirmations: u8,
    
    /// Nonce for preventing replay attacks
    pub nonce: u64,
    
    /// Emergency pause flag
    pub is_paused: bool,
    
    /// Additional metadata for the lock operation
    pub metadata: LockMetadata,
    
    /// Merkle proof for the lock operation
    pub merkle_proof: Vec<[u8; 32]>,
    
    /// Root hash of the merkle tree
    pub merkle_root: [u8; 32],
    
    /// Bump seed for PDA derivation
    pub bump: u8,
}

impl LockedTokens {
    pub const LEN: usize = 8 + // discriminator
        8 + // lock_id
        32 + // user
        32 + // token_mint
        8 + // amount
        1 + // destination_chain
        64 + // destination_address
        32 + // source_tx_hash
        8 + // locked_at
        8 + // unlocked_at
        1 + // status
        8 + // bridge_fee
        4 + (ValidatorSignature::LEN * MAX_VALIDATORS) + // validator_signatures
        1 + // required_confirmations
        1 + // current_confirmations
        8 + // nonce
        1 + // is_paused
        LockMetadata::LEN + // metadata
        4 + (32 * MAX_MERKLE_PROOF_DEPTH) + // merkle_proof
        32 + // merkle_root
        1; // bump

    /// Initialize a new locked tokens account
    pub fn initialize(
        &mut self,
        lock_id: u64,
        user: Pubkey,
        token_mint: Pubkey,
        amount: u64,
        destination_chain: u8,
        destination_address: [u8; 64],
        source_tx_hash: [u8; 32],
        bridge_fee: u64,
        required_confirmations: u8,
        nonce: u64,
        bump: u8,
    ) -> Result<()> {
        self.lock_id = lock_id;
        self.user = user;
        self.token_mint = token_mint;
        self.amount = amount;
        self.destination_chain = destination_chain;
        self.destination_address = destination_address;
        self.source_tx_hash = source_tx_hash;
        self.locked_at = Clock::get()?.unix_timestamp;
        self.unlocked_at = 0;
        self.status = LockStatus::Locked;
        self.bridge_fee = bridge_fee;
        self.validator_signatures = Vec::new();
        self.required_confirmations = required_confirmations;
        self.current_confirmations = 0;
        self.nonce = nonce;
        self.is_paused = false;
        self.metadata = LockMetadata::default();
        self.merkle_proof = Vec::new();
        self.merkle_root = [0u8; 32];
        self.bump = bump;
        
        Ok(())
    }

    /// Add a validator signature for unlock operation
    pub fn add_validator_signature(
        &mut self,
        validator: Pubkey,
        signature: [u8; 64],
        message_hash: [u8; 32],
    ) -> Result<()> {
        // Check if validator already signed
        for existing_sig in &self.validator_signatures {
            if existing_sig.validator == validator {
                return Err(crate::errors::BridgeError::ValidatorAlreadySigned.into());
            }
        }

        // Add new signature
        let validator_signature = ValidatorSignature {
            validator,
            signature,
            message_hash,
        };

        self.validator_signatures.push(validator_signature);
        self.current_confirmations += 1;

        // Check if we have enough confirmations
        if self.current_confirmations >= self.required_confirmations {
            self.status = LockStatus::ReadyToUnlock;
        }

        Ok(())
    }

    /// Unlock the tokens
    pub fn unlock(&mut self) -> Result<()> {
        require!(
            self.status == LockStatus::ReadyToUnlock,
            crate::errors::BridgeError::InsufficientConfirmations
        );

        require!(!self.is_paused, crate::errors::BridgeError::BridgePaused);

        self.status = LockStatus::Unlocked;
        self.unlocked_at = Clock::get()?.unix_timestamp;

        Ok(())
    }

    /// Emergency pause the lock operation
    pub fn emergency_pause(&mut self) -> Result<()> {
        self.is_paused = true;
        self.status = LockStatus::Paused;
        Ok(())
    }

    /// Resume from emergency pause
    pub fn resume(&mut self) -> Result<()> {
        require!(self.is_paused, crate::errors::BridgeError::NotPaused);
        
        self.is_paused = false;
        
        // Restore previous status based on confirmations
        if self.current_confirmations >= self.required_confirmations {
            self.status = LockStatus::ReadyToUnlock;
        } else {
            self.status = LockStatus::Locked;
        }
        
        Ok(())
    }

    /// Cancel the lock operation (only if not unlocked)
    pub fn cancel(&mut self) -> Result<()> {
        require!(
            self.status != LockStatus::Unlocked,
            crate::errors::BridgeError::AlreadyUnlocked
        );

        self.status = LockStatus::Cancelled;
        Ok(())
    }

    /// Check if the lock has expired
    pub fn is_expired(&self) -> Result<bool> {
        let current_time = Clock::get()?.unix_timestamp;
        let expiry_time = self.locked_at + LOCK_EXPIRY_DURATION;
        Ok(current_time > expiry_time)
    }

    /// Calculate time remaining before expiry
    pub fn time_until_expiry(&self) -> Result<i64> {
        let current_time = Clock::get()?.unix_timestamp;
        let expiry_time = self.locked_at + LOCK_EXPIRY_DURATION;
        Ok((expiry_time - current_time).max(0))
    }

    /// Set merkle proof for verification
    pub fn set_merkle_proof(
        &mut self,
        proof: Vec<[u8; 32]>,
        root: [u8; 32],
    ) -> Result<()> {
        require!(
            proof.len() <= MAX_MERKLE_PROOF_DEPTH,
            crate::errors::BridgeError::InvalidMerkleProof
        );

        self.merkle_proof = proof;
        self.merkle_root = root;
        Ok(())
    }

    /// Verify merkle proof
    pub fn verify_merkle_proof(&self, leaf_hash: [u8; 32]) -> bool {
        if self.merkle_proof.is_empty() {
            return false;
        }

        let mut computed_hash = leaf_hash;
        for proof_element in &self.merkle_proof {
            computed_hash = if computed_hash <= *proof_element {
                anchor_lang::solana_program::keccak::hash(&[
                    computed_hash.as_ref(),
                    proof_element.as_ref(),
                ].concat()).to_bytes()
            } else {
                anchor_lang::solana_program::keccak::hash(&[
                    proof_element.as_ref(),
                    computed_hash.as_ref(),
                ].concat()).to_bytes()
            };
        }

        computed_hash == self.merkle_root
    }

    /// Update metadata
    pub fn update_metadata(
        &mut self,
        metadata: LockMetadata,
    ) -> Result<()> {
        self.metadata = metadata;
        Ok(())
    }

    /// Get lock duration in seconds
    pub fn get_lock_duration(&self) -> Result<i64> {
        let current_time = Clock::get()?.unix_timestamp;
        Ok(current_time - self.locked_at)
    }

    /// Check if lock is active (locked and not expired)
    pub fn is_active(&self) -> Result<bool> {
        Ok(self.status == LockStatus::Locked && !self.is_expired()?)
    }

    /// Get effective fee rate based on amount and chain
    pub fn calculate_effective_fee_rate(&self) -> u64 {
        let base_rate = match self.destination_chain {
            ETHEREUM_CHAIN_ID => ETHEREUM_BASE_FEE_RATE,
            BSC_CHAIN_ID => BSC_BASE_FEE_RATE,
            POLYGON_CHAIN_ID => POLYGON_BASE_FEE_RATE,
            _ => DEFAULT_BASE_FEE_RATE,
        };

        // Apply volume discount
        let volume_discount = if self.amount >= HIGH_VOLUME_THRESHOLD {
            HIGH_VOLUME_DISCOUNT
        } else if self.amount >= MEDIUM_VOLUME_THRESHOLD {
            MEDIUM_VOLUME_DISCOUNT
        } else {
            0
        };

        base_rate.saturating_sub(volume_discount)
    }

    /// Validate lock parameters
    pub fn validate_lock_params(&self) -> Result<()> {
        require!(self.amount > 0, crate::errors::BridgeError::InvalidAmount);
        
        require!(
            self.amount >= MIN_LOCK_AMOUNT,
            crate::errors::BridgeError::AmountTooSmall
        );
        
        require!(
            self.amount <= MAX_LOCK_AMOUNT,
            crate::errors::BridgeError::AmountTooLarge
        );

        require!(
            self.destination_chain != 0,
            crate::errors::BridgeError::InvalidDestinationChain
        );

        require!(
            self.required_confirmations >= MIN_REQUIRED_CONFIRMATIONS &&
            self.required_confirmations <= MAX_REQUIRED_CONFIRMATIONS,
            crate::errors::BridgeError::InvalidConfirmationCount
        );

        Ok(())
    }
}

/// Status of a lock operation
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq)]
pub enum LockStatus {
    /// Tokens are locked and waiting for confirmations
    Locked,
    /// Enough confirmations received, ready to unlock
    ReadyToUnlock,
    /// Tokens have been unlocked
    Unlocked,
    /// Lock operation was cancelled
    Cancelled,
    /// Lock operation is paused
    Paused,
    /// Lock operation failed
    Failed,
}

impl Default for LockStatus {
    fn default() -> Self {
        LockStatus::Locked
    }
}

/// Validator signature for unlock operation
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct ValidatorSignature {
    /// Validator's public key
    pub validator: Pubkey,
    /// Signature over the unlock message
    pub signature: [u8; 64],
    /// Hash of the message that was signed
    pub message_hash: [u8; 32],
}

impl ValidatorSignature {
    pub const LEN: usize = 32 + 64 + 32;
}

/// Metadata for lock operations
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, Default)]
pub struct LockMetadata {
    /// Optional description of the lock operation
    pub description: String,
    /// Tags for categorization
    pub tags: Vec<String>,
    /// Custom data fields
    pub custom_data: Vec<u8>,
    /// Priority level (0 = normal, 1 = high, 2 = urgent)
    pub priority: u8,
    /// Gas limit for destination chain
    pub gas_limit: u64,
    /// Gas price for destination chain
    pub gas_price: u64,
}

impl LockMetadata {
    pub const LEN: usize = 4 + 256 + // description (max 256 chars)
        4 + (4 + 64) * 10 + // tags (max 10 tags, 64 chars each)
        4 + 512 + // custom_data (max 512 bytes)
        1 + // priority
        8 + // gas_limit
        8; // gas_price
}

/// Historical lock record for analytics
#[account]
#[derive(Default)]
pub struct LockHistory {
    /// User who performed the lock
    pub user: Pubkey,
    /// Lock operations count
    pub total_locks: u64,
    /// Total amount locked across all operations
    pub total_amount_locked: u64,
    /// Total fees paid
    pub total_fees_paid: u64,
    /// Last lock timestamp
    pub last_lock_at: i64,
    /// Average lock amount
    pub average_lock_amount: u64,
    /// Bump seed for PDA derivation
    pub bump: u8,
}

impl LockHistory {
    pub const LEN: usize = 8 + // discriminator
        32 + // user
        8 + // total_locks
        8 + // total_amount_locked
        8 + // total_fees_paid
        8 + // last_lock_at
        8 + // average_lock_amount
        1; // bump

    /// Initialize lock history
    pub fn initialize(&mut self, user: Pubkey, bump: u8) -> Result<()> {
        self.user = user;
        self.total_locks = 0;
        self.total_amount_locked = 0;
        self.total_fees_paid = 0;
        self.last_lock_at = 0;
        self.average_lock_amount = 0;
        self.bump = bump;
        Ok(())
    }

    /// Update history with new lock
    pub fn record_lock(&mut self, amount: u64, fee: u64) -> Result<()> {
        self.total_locks += 1;
        self.total_amount_locked += amount;
        self.total_fees_paid += fee;
        self.last_lock_at = Clock::get()?.unix_timestamp;
        self.average_lock_amount = self.total_amount_locked / self.total_locks;
        Ok(())
    }

    /// Calculate user tier based on history
    pub fn get_user_tier(&self) -> UserTier {
        if self.total_amount_locked >= PLATINUM_TIER_THRESHOLD {
            UserTier::Platinum
        } else if self.total_amount_locked >= GOLD_TIER_THRESHOLD {
            UserTier::Gold
        } else if self.total_amount_locked >= SILVER_TIER_THRESHOLD {
            UserTier::Silver
        } else {
            UserTier::Bronze
        }
    }

    /// Check if user qualifies for fee discount
    pub fn get_fee_discount(&self) -> u64 {
        match self.get_user_tier() {
            UserTier::Platinum => PLATINUM_FEE_DISCOUNT,
            UserTier::Gold => GOLD_FEE_DISCOUNT,
            UserTier::Silver => SILVER_FEE_DISCOUNT,
            UserTier::Bronze => 0,
        }
    }
}

/// User tier based on bridge usage
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq)]
pub enum UserTier {
    Bronze,
    Silver,
    Gold,
    Platinum,
}

/// Bridge statistics for monitoring
#[account]
#[derive(Default)]
pub struct BridgeStatistics {
    /// Total number of lock operations
    pub total_locks: u64,
    /// Total number of unlock operations
    pub total_unlocks: u64,
    /// Total volume locked across all chains
    pub total_volume_locked: u64,
    /// Total fees collected
    pub total_fees_collected: u64,
    /// Number of active locks
    pub active_locks: u64,
    /// Average lock duration
    pub average_lock_duration: u64,
    /// Last update timestamp
    pub last_updated: i64,
    /// Daily statistics (last 30 days)
    pub daily_stats: Vec<DailyStats>,
    /// Bump seed for PDA derivation
    pub bump: u8,
}

impl BridgeStatistics {
    pub const LEN: usize = 8 + // discriminator
        8 + // total_locks
        8 + // total_unlocks
        8 + // total_volume_locked
        8 + // total_fees_collected
        8 + // active_locks
        8 + // average_lock_duration
        8 + // last_updated
        4 + (DailyStats::LEN * 30) + // daily_stats
        1; // bump

    /// Update statistics with new lock
    pub fn record_lock(&mut self, amount: u64, fee: u64) -> Result<()> {
        self.total_locks += 1;
        self.total_volume_locked += amount;
        self.total_fees_collected += fee;
        self.active_locks += 1;
        self.last_updated = Clock::get()?.unix_timestamp;
        
        // Update daily stats
        self.update_daily_stats(amount, fee, true)?;
        
        Ok(())
    }

    /// Update statistics with unlock
    pub fn record_unlock(&mut self, duration: u64) -> Result<()> {
        self.total_unlocks += 1;
        self.active_locks = self.active_locks.saturating_sub(1);
        
        // Update average duration
        let total_duration = self.average_lock_duration * (self.total_unlocks - 1) + duration;
        self.average_lock_duration = total_duration / self.total_unlocks;
        
        self.last_updated = Clock::get()?.unix_timestamp;
        
        // Update daily stats
        self.update_daily_stats(0, 0, false)?;
        
        Ok(())
    }

    /// Update daily statistics
    fn update_daily_stats(&mut self, amount: u64, fee: u64, is_lock: bool) -> Result<()> {
        let current_day = Clock::get()?.unix_timestamp / 86400; // Convert to days
        
        // Find or create today's stats
        let today_index = self.daily_stats.iter().position(|stats| stats.day == current_day);
        
        if let Some(index) = today_index {
            if is_lock {
                self.daily_stats[index].locks += 1;
                self.daily_stats[index].volume += amount;
                self.daily_stats[index].fees += fee;
            } else {
                self.daily_stats[index].unlocks += 1;
            }
        } else {
            // Create new daily stats entry
            let new_stats = DailyStats {
                day: current_day,
                locks: if is_lock { 1 } else { 0 },
                unlocks: if is_lock { 0 } else { 1 },
                volume: amount,
                fees: fee,
            };
            
            // Keep only last 30 days
            if self.daily_stats.len() >= 30 {
                self.daily_stats.remove(0);
            }
            
            self.daily_stats.push(new_stats);
        }
        
        Ok(())
    }
}

/// Daily statistics entry
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, Default)]
pub struct DailyStats {
    /// Day since epoch
    pub day: i64,
    /// Number of locks on this day
    pub locks: u64,
    /// Number of unlocks on this day
    pub unlocks: u64,
    /// Total volume locked on this day
    pub volume: u64,
    /// Total fees collected on this day
    pub fees: u64,
}

impl DailyStats {
    pub const LEN: usize = 8 + 8 + 8 + 8 + 8;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lock_status_default() {
        assert_eq!(LockStatus::default(), LockStatus::Locked);
    }

    #[test]
    fn test_user_tier_ordering() {
        let bronze = UserTier::Bronze;
        let silver = UserTier::Silver;
        let gold = UserTier::Gold;
        let platinum = UserTier::Platinum;
        
        assert_ne!(bronze, silver);
        assert_ne!(silver, gold);
        assert_ne!(gold, platinum);
    }

    #[test]
    fn test_validator_signature_len() {
        assert_eq!(ValidatorSignature::LEN, 32 + 64 + 32);
    }

    #[test]
    fn test_daily_stats_len() {
        assert_eq!(DailyStats::LEN, 8 + 8 + 8 + 8 + 8);
    }
}
