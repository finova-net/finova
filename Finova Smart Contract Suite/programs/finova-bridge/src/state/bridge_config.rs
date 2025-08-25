// programs/finova-bridge/src/state/bridge_config.rs

use anchor_lang::prelude::*;
use crate::constants::*;
use crate::errors::BridgeError;

/// Bridge configuration state managing cross-chain operations
#[account]
pub struct BridgeConfig {
    /// Unique identifier for this bridge instance
    pub bridge_id: u64,
    
    /// Authority that can modify bridge settings
    pub authority: Pubkey,
    
    /// Emergency pause authority (can pause but not unpause)
    pub emergency_authority: Pubkey,
    
    /// Pending authority for ownership transfers
    pub pending_authority: Option<Pubkey>,
    
    /// Source chain identifier (Solana = 1)
    pub source_chain_id: u16,
    
    /// Supported destination chain IDs
    pub supported_chains: [u16; MAX_SUPPORTED_CHAINS],
    
    /// Number of active supported chains
    pub active_chains_count: u8,
    
    /// Minimum required validator signatures
    pub min_validator_signatures: u8,
    
    /// Maximum validator set size
    pub max_validators: u8,
    
    /// Current validator set size
    pub current_validators: u8,
    
    /// Bridge operational status
    pub status: BridgeStatus,
    
    /// Minimum lock amount (prevents dust attacks)
    pub min_lock_amount: u64,
    
    /// Maximum lock amount per transaction
    pub max_lock_amount: u64,
    
    /// Daily lock limit to prevent massive drains
    pub daily_lock_limit: u64,
    
    /// Current daily locked amount
    pub daily_locked_amount: u64,
    
    /// Timestamp of current day (for daily limit reset)
    pub daily_limit_reset_time: i64,
    
    /// Bridge fee in basis points (100 = 1%)
    pub bridge_fee_bps: u16,
    
    /// Minimum fee amount in lamports
    pub min_fee_amount: u64,
    
    /// Fee recipient address
    pub fee_recipient: Pubkey,
    
    /// Challenge period for withdrawals (in seconds)
    pub challenge_period: u32,
    
    /// Grace period for emergency actions (in seconds)
    pub grace_period: u32,
    
    /// Total number of transactions processed
    pub total_transactions: u64,
    
    /// Total volume bridged (in lamports)
    pub total_volume: u128,
    
    /// Total fees collected (in lamports)
    pub total_fees: u64,
    
    /// Last update timestamp
    pub last_updated: i64,
    
    /// Bridge version for upgrades
    pub version: u8,
    
    /// Emergency pause expiry (automatic unpause)
    pub emergency_pause_expiry: Option<i64>,
    
    /// Rate limiting configuration
    pub rate_limit: RateLimitConfig,
    
    /// Security parameters
    pub security_config: SecurityConfig,
    
    /// Reward configuration for validators
    pub validator_rewards: ValidatorRewardConfig,
    
    /// Reserved space for future upgrades
    pub reserved: [u8; 128],
}

impl BridgeConfig {
    pub const LEN: usize = 8 + // discriminator
        8 + // bridge_id
        32 + // authority
        32 + // emergency_authority
        1 + 32 + // pending_authority (Option<Pubkey>)
        2 + // source_chain_id
        2 * MAX_SUPPORTED_CHAINS + // supported_chains
        1 + // active_chains_count
        1 + // min_validator_signatures
        1 + // max_validators
        1 + // current_validators
        1 + // status
        8 + // min_lock_amount
        8 + // max_lock_amount
        8 + // daily_lock_limit
        8 + // daily_locked_amount
        8 + // daily_limit_reset_time
        2 + // bridge_fee_bps
        8 + // min_fee_amount
        32 + // fee_recipient
        4 + // challenge_period
        4 + // grace_period
        8 + // total_transactions
        16 + // total_volume
        8 + // total_fees
        8 + // last_updated
        1 + // version
        1 + 8 + // emergency_pause_expiry (Option<i64>)
        RateLimitConfig::LEN +
        SecurityConfig::LEN +
        ValidatorRewardConfig::LEN +
        128; // reserved

    /// Initialize a new bridge configuration
    pub fn initialize(
        &mut self,
        bridge_id: u64,
        authority: Pubkey,
        emergency_authority: Pubkey,
        source_chain_id: u16,
        min_validator_signatures: u8,
        max_validators: u8,
    ) -> Result<()> {
        require!(min_validator_signatures > 0, BridgeError::InvalidValidatorCount);
        require!(max_validators >= min_validator_signatures, BridgeError::InvalidValidatorCount);
        require!(max_validators <= MAX_VALIDATORS, BridgeError::TooManyValidators);

        self.bridge_id = bridge_id;
        self.authority = authority;
        self.emergency_authority = emergency_authority;
        self.pending_authority = None;
        self.source_chain_id = source_chain_id;
        self.supported_chains = [0; MAX_SUPPORTED_CHAINS];
        self.active_chains_count = 0;
        self.min_validator_signatures = min_validator_signatures;
        self.max_validators = max_validators;
        self.current_validators = 0;
        self.status = BridgeStatus::Active;
        
        // Set default limits
        self.min_lock_amount = DEFAULT_MIN_LOCK_AMOUNT;
        self.max_lock_amount = DEFAULT_MAX_LOCK_AMOUNT;
        self.daily_lock_limit = DEFAULT_DAILY_LOCK_LIMIT;
        self.daily_locked_amount = 0;
        self.daily_limit_reset_time = Clock::get()?.unix_timestamp;
        
        // Set default fees
        self.bridge_fee_bps = DEFAULT_BRIDGE_FEE_BPS;
        self.min_fee_amount = DEFAULT_MIN_FEE_AMOUNT;
        self.fee_recipient = authority;
        
        // Set default timeouts
        self.challenge_period = DEFAULT_CHALLENGE_PERIOD;
        self.grace_period = DEFAULT_GRACE_PERIOD;
        
        // Initialize counters
        self.total_transactions = 0;
        self.total_volume = 0;
        self.total_fees = 0;
        self.last_updated = Clock::get()?.unix_timestamp;
        self.version = BRIDGE_VERSION;
        self.emergency_pause_expiry = None;
        
        // Initialize rate limiting
        self.rate_limit = RateLimitConfig::default();
        
        // Initialize security config
        self.security_config = SecurityConfig::default();
        
        // Initialize validator rewards
        self.validator_rewards = ValidatorRewardConfig::default();
        
        self.reserved = [0; 128];
        
        Ok(())
    }

    /// Add a supported destination chain
    pub fn add_supported_chain(&mut self, chain_id: u16) -> Result<()> {
        require!(self.active_chains_count < MAX_SUPPORTED_CHAINS as u8, BridgeError::TooManyChains);
        require!(!self.is_chain_supported(chain_id), BridgeError::ChainAlreadySupported);
        
        self.supported_chains[self.active_chains_count as usize] = chain_id;
        self.active_chains_count += 1;
        self.last_updated = Clock::get()?.unix_timestamp;
        
        Ok(())
    }

    /// Remove a supported destination chain
    pub fn remove_supported_chain(&mut self, chain_id: u16) -> Result<()> {
        let mut found_index = None;
        
        for (i, &supported_chain) in self.supported_chains[..self.active_chains_count as usize].iter().enumerate() {
            if supported_chain == chain_id {
                found_index = Some(i);
                break;
            }
        }
        
        require!(found_index.is_some(), BridgeError::ChainNotSupported);
        let index = found_index.unwrap();
        
        // Shift remaining chains to fill the gap
        for i in index..self.active_chains_count as usize - 1 {
            self.supported_chains[i] = self.supported_chains[i + 1];
        }
        
        self.supported_chains[self.active_chains_count as usize - 1] = 0;
        self.active_chains_count -= 1;
        self.last_updated = Clock::get()?.unix_timestamp;
        
        Ok(())
    }

    /// Check if a chain is supported
    pub fn is_chain_supported(&self, chain_id: u16) -> bool {
        self.supported_chains[..self.active_chains_count as usize]
            .iter()
            .any(|&id| id == chain_id)
    }

    /// Update bridge status
    pub fn update_status(&mut self, new_status: BridgeStatus) -> Result<()> {
        // Additional validation for status transitions
        match (self.status, new_status) {
            (BridgeStatus::Paused, BridgeStatus::Active) => {
                // Only authority can unpause
                require!(self.emergency_pause_expiry.is_none() || 
                         Clock::get()?.unix_timestamp > self.emergency_pause_expiry.unwrap(),
                         BridgeError::StillInEmergencyPause);
            },
            (BridgeStatus::Maintenance, BridgeStatus::Active) => {
                // Ensure maintenance period is complete
                require!(Clock::get()?.unix_timestamp > self.last_updated + self.grace_period as i64,
                         BridgeError::MaintenancePeriodActive);
            },
            _ => {}
        }
        
        self.status = new_status;
        self.last_updated = Clock::get()?.unix_timestamp;
        
        Ok(())
    }

    /// Emergency pause with automatic expiry
    pub fn emergency_pause(&mut self, duration_seconds: u32) -> Result<()> {
        require!(duration_seconds <= MAX_EMERGENCY_PAUSE_DURATION, BridgeError::PauseDurationTooLong);
        
        self.status = BridgeStatus::Paused;
        self.emergency_pause_expiry = Some(Clock::get()?.unix_timestamp + duration_seconds as i64);
        self.last_updated = Clock::get()?.unix_timestamp;
        
        Ok(())
    }

    /// Check if bridge is operational
    pub fn is_operational(&self) -> bool {
        match self.status {
            BridgeStatus::Active => true,
            BridgeStatus::Paused => {
                // Check if emergency pause has expired
                if let Some(expiry) = self.emergency_pause_expiry {
                    Clock::get().map(|clock| clock.unix_timestamp > expiry).unwrap_or(false)
                } else {
                    false
                }
            },
            _ => false,
        }
    }

    /// Update daily lock limit tracking
    pub fn update_daily_locked_amount(&mut self, amount: u64) -> Result<()> {
        let current_time = Clock::get()?.unix_timestamp;
        
        // Reset daily counter if a new day has started
        if current_time >= self.daily_limit_reset_time + 86400 { // 24 hours
            self.daily_locked_amount = 0;
            self.daily_limit_reset_time = current_time;
        }
        
        require!(
            self.daily_locked_amount.saturating_add(amount) <= self.daily_lock_limit,
            BridgeError::DailyLimitExceeded
        );
        
        self.daily_locked_amount = self.daily_locked_amount.saturating_add(amount);
        
        Ok(())
    }

    /// Calculate bridge fee for a given amount
    pub fn calculate_fee(&self, amount: u64) -> u64 {
        let calculated_fee = (amount as u128 * self.bridge_fee_bps as u128 / 10000) as u64;
        calculated_fee.max(self.min_fee_amount)
    }

    /// Validate lock amount against limits
    pub fn validate_lock_amount(&self, amount: u64) -> Result<()> {
        require!(amount >= self.min_lock_amount, BridgeError::AmountTooSmall);
        require!(amount <= self.max_lock_amount, BridgeError::AmountTooLarge);
        
        Ok(())
    }

    /// Update transaction statistics
    pub fn update_stats(&mut self, amount: u64, fee: u64) -> Result<()> {
        self.total_transactions = self.total_transactions.saturating_add(1);
        self.total_volume = self.total_volume.saturating_add(amount as u128);
        self.total_fees = self.total_fees.saturating_add(fee);
        self.last_updated = Clock::get()?.unix_timestamp;
        
        Ok(())
    }

    /// Check rate limiting
    pub fn check_rate_limit(&self, user: &Pubkey) -> Result<()> {
        // Implementation would check user-specific rate limits
        // This is a simplified version
        Ok(())
    }

    /// Update validator count
    pub fn update_validator_count(&mut self, count: u8) -> Result<()> {
        require!(count <= self.max_validators, BridgeError::TooManyValidators);
        require!(count >= self.min_validator_signatures, BridgeError::InsufficientValidators);
        
        self.current_validators = count;
        self.last_updated = Clock::get()?.unix_timestamp;
        
        Ok(())
    }

    /// Transfer authority to pending authority
    pub fn finalize_authority_transfer(&mut self) -> Result<()> {
        require!(self.pending_authority.is_some(), BridgeError::NoPendingAuthority);
        
        self.authority = self.pending_authority.unwrap();
        self.pending_authority = None;
        self.last_updated = Clock::get()?.unix_timestamp;
        
        Ok(())
    }

    /// Set pending authority for two-step transfer
    pub fn set_pending_authority(&mut self, new_authority: Pubkey) -> Result<()> {
        self.pending_authority = Some(new_authority);
        self.last_updated = Clock::get()?.unix_timestamp;
        
        Ok(())
    }
}

/// Bridge operational status
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq)]
pub enum BridgeStatus {
    /// Bridge is fully operational
    Active,
    /// Bridge is temporarily paused
    Paused,
    /// Bridge is under maintenance
    Maintenance,
    /// Bridge is deprecated and will be shut down
    Deprecated,
}

impl Default for BridgeStatus {
    fn default() -> Self {
        BridgeStatus::Active
    }
}

/// Rate limiting configuration
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy)]
pub struct RateLimitConfig {
    /// Maximum transactions per user per hour
    pub max_tx_per_hour: u16,
    /// Maximum amount per user per hour
    pub max_amount_per_hour: u64,
    /// Rate limit window in seconds
    pub window_seconds: u32,
    /// Enable rate limiting
    pub enabled: bool,
}

impl RateLimitConfig {
    pub const LEN: usize = 2 + 8 + 4 + 1;
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            max_tx_per_hour: DEFAULT_MAX_TX_PER_HOUR,
            max_amount_per_hour: DEFAULT_MAX_AMOUNT_PER_HOUR,
            window_seconds: 3600, // 1 hour
            enabled: true,
        }
    }
}

/// Security configuration parameters
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy)]
pub struct SecurityConfig {
    /// Require additional verification for large amounts
    pub large_amount_threshold: u64,
    /// Additional verification delay for large amounts (seconds)
    pub large_amount_delay: u32,
    /// Enable fraud detection
    pub fraud_detection_enabled: bool,
    /// Maximum failed attempts before lockout
    pub max_failed_attempts: u8,
    /// Lockout duration in seconds
    pub lockout_duration: u32,
    /// Enable IP-based restrictions
    pub ip_restrictions_enabled: bool,
    /// Enable time-based restrictions
    pub time_restrictions_enabled: bool,
    /// Minimum time between transactions (seconds)
    pub min_tx_interval: u16,
}

impl SecurityConfig {
    pub const LEN: usize = 8 + 4 + 1 + 1 + 4 + 1 + 1 + 2;
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            large_amount_threshold: DEFAULT_LARGE_AMOUNT_THRESHOLD,
            large_amount_delay: DEFAULT_LARGE_AMOUNT_DELAY,
            fraud_detection_enabled: true,
            max_failed_attempts: 5,
            lockout_duration: 3600, // 1 hour
            ip_restrictions_enabled: false,
            time_restrictions_enabled: true,
            min_tx_interval: 60, // 1 minute
        }
    }
}

/// Validator reward configuration
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy)]
pub struct ValidatorRewardConfig {
    /// Base reward per validation (in lamports)
    pub base_reward: u64,
    /// Performance multiplier (basis points)
    pub performance_multiplier_bps: u16,
    /// Minimum uptime requirement (basis points)
    pub min_uptime_bps: u16,
    /// Slash amount for malicious behavior (lamports)
    pub slash_amount: u64,
    /// Reward distribution frequency (seconds)
    pub distribution_frequency: u32,
    /// Enable dynamic rewards based on activity
    pub dynamic_rewards: bool,
    /// Maximum reward per epoch
    pub max_reward_per_epoch: u64,
}

impl ValidatorRewardConfig {
    pub const LEN: usize = 8 + 2 + 2 + 8 + 4 + 1 + 8;
}

impl Default for ValidatorRewardConfig {
    fn default() -> Self {
        Self {
            base_reward: DEFAULT_VALIDATOR_BASE_REWARD,
            performance_multiplier_bps: 10000, // 100%
            min_uptime_bps: 9500, // 95%
            slash_amount: DEFAULT_VALIDATOR_SLASH_AMOUNT,
            distribution_frequency: 86400, // 24 hours
            dynamic_rewards: true,
            max_reward_per_epoch: DEFAULT_MAX_REWARD_PER_EPOCH,
        }
    }
}

/// Bridge configuration seeds for PDA generation
impl BridgeConfig {
    pub const SEED_PREFIX: &'static [u8] = b"bridge_config";
    
    pub fn get_pda(bridge_id: u64) -> (Pubkey, u8) {
        Pubkey::find_program_address(
            &[
                Self::SEED_PREFIX,
                &bridge_id.to_le_bytes(),
            ],
            &crate::ID,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bridge_config_initialization() {
        let mut config = BridgeConfig {
            bridge_id: 0,
            authority: Pubkey::default(),
            emergency_authority: Pubkey::default(),
            pending_authority: None,
            source_chain_id: 0,
            supported_chains: [0; MAX_SUPPORTED_CHAINS],
            active_chains_count: 0,
            min_validator_signatures: 0,
            max_validators: 0,
            current_validators: 0,
            status: BridgeStatus::Active,
            min_lock_amount: 0,
            max_lock_amount: 0,
            daily_lock_limit: 0,
            daily_locked_amount: 0,
            daily_limit_reset_time: 0,
            bridge_fee_bps: 0,
            min_fee_amount: 0,
            fee_recipient: Pubkey::default(),
            challenge_period: 0,
            grace_period: 0,
            total_transactions: 0,
            total_volume: 0,
            total_fees: 0,
            last_updated: 0,
            version: 0,
            emergency_pause_expiry: None,
            rate_limit: RateLimitConfig::default(),
            security_config: SecurityConfig::default(),
            validator_rewards: ValidatorRewardConfig::default(),
            reserved: [0; 128],
        };

        // Test successful initialization
        let result = config.initialize(
            1,
            Pubkey::new_unique(),
            Pubkey::new_unique(),
            1,
            3,
            5,
        );
        
        assert!(result.is_ok());
        assert_eq!(config.bridge_id, 1);
        assert_eq!(config.min_validator_signatures, 3);
        assert_eq!(config.max_validators, 5);
    }

    #[test]
    fn test_chain_management() {
        let mut config = BridgeConfig {
            bridge_id: 1,
            supported_chains: [0; MAX_SUPPORTED_CHAINS],
            active_chains_count: 0,
            ..Default::default()
        };

        // Test adding supported chain
        assert!(config.add_supported_chain(2).is_ok());
        assert_eq!(config.active_chains_count, 1);
        assert!(config.is_chain_supported(2));

        // Test removing supported chain
        assert!(config.remove_supported_chain(2).is_ok());
        assert_eq!(config.active_chains_count, 0);
        assert!(!config.is_chain_supported(2));
    }

    #[test]
    fn test_fee_calculation() {
        let config = BridgeConfig {
            bridge_fee_bps: 50, // 0.5%
            min_fee_amount: 1000,
            ..Default::default()
        };

        // Test normal fee calculation
        assert_eq!(config.calculate_fee(100000), 500);
        
        // Test minimum fee
        assert_eq!(config.calculate_fee(10), 1000);
    }
}

impl Default for BridgeConfig {
    fn default() -> Self {
        Self {
            bridge_id: 0,
            authority: Pubkey::default(),
            emergency_authority: Pubkey::default(),
            pending_authority: None,
            source_chain_id: 1, // Solana
            supported_chains: [0; MAX_SUPPORTED_CHAINS],
            active_chains_count: 0,
            min_validator_signatures: 1,
            max_validators: 1,
            current_validators: 0,
            status: BridgeStatus::Active,
            min_lock_amount: DEFAULT_MIN_LOCK_AMOUNT,
            max_lock_amount: DEFAULT_MAX_LOCK_AMOUNT,
            daily_lock_limit: DEFAULT_DAILY_LOCK_LIMIT,
            daily_locked_amount: 0,
            daily_limit_reset_time: 0,
            bridge_fee_bps: DEFAULT_BRIDGE_FEE_BPS,
            min_fee_amount: DEFAULT_MIN_FEE_AMOUNT,
            fee_recipient: Pubkey::default(),
            challenge_period: DEFAULT_CHALLENGE_PERIOD,
            grace_period: DEFAULT_GRACE_PERIOD,
            total_transactions: 0,
            total_volume: 0,
            total_fees: 0,
            last_updated: 0,
            version: BRIDGE_VERSION,
            emergency_pause_expiry: None,
            rate_limit: RateLimitConfig::default(),
            security_config: SecurityConfig::default(),
            validator_rewards: ValidatorRewardConfig::default(),
            reserved: [0; 128],
        }
    }
}
