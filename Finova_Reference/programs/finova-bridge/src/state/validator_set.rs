// programs/finova-bridge/src/state/validator_set.rs

use anchor_lang::prelude::*;
use std::collections::BTreeMap;

/// Maximum number of validators in the set
pub const MAX_VALIDATORS: usize = 100;

/// Minimum stake required to become a validator (in lamports)
pub const MIN_VALIDATOR_STAKE: u64 = 1_000_000_000; // 1 SOL

/// Validator status enumeration
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum ValidatorStatus {
    /// Validator is active and participating in consensus
    Active,
    /// Validator is temporarily inactive
    Inactive,
    /// Validator is being slashed for malicious behavior
    Slashed,
    /// Validator has been permanently banned
    Banned,
    /// Validator is in the process of joining
    Pending,
}

impl Default for ValidatorStatus {
    fn default() -> Self {
        ValidatorStatus::Pending
    }
}

/// Individual validator information
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, Default)]
pub struct ValidatorInfo {
    /// Validator's public key
    pub pubkey: Pubkey,
    /// Amount of stake (in lamports)
    pub stake: u64,
    /// Validator status
    pub status: ValidatorStatus,
    /// Number of successful validations
    pub successful_validations: u64,
    /// Number of failed validations
    pub failed_validations: u64,
    /// Timestamp when validator was added
    pub joined_at: i64,
    /// Last activity timestamp
    pub last_activity: i64,
    /// Reputation score (0-1000)
    pub reputation: u16,
    /// Number of times slashed
    pub slash_count: u8,
    /// Total rewards earned
    pub total_rewards: u64,
    /// Network identifier (for cross-chain support)
    pub network_id: u8,
    /// Validator's commission rate (basis points, 0-10000)
    pub commission_rate: u16,
    /// Geographic region code (for redundancy)
    pub region_code: u8,
    /// Validator version/software version
    pub version: u32,
}

impl ValidatorInfo {
    /// Calculate validator's effective voting power based on stake and reputation
    pub fn calculate_voting_power(&self) -> u64 {
        if self.status != ValidatorStatus::Active {
            return 0;
        }

        let base_power = self.stake / 1_000_000; // Convert to voting units
        let reputation_multiplier = (self.reputation as u64).min(1000);
        
        // Apply reputation bonus (max 20% bonus)
        let bonus = (base_power * reputation_multiplier) / 5000;
        base_power + bonus
    }

    /// Check if validator is eligible for rewards
    pub fn is_eligible_for_rewards(&self) -> bool {
        matches!(self.status, ValidatorStatus::Active) 
            && self.reputation >= 500 
            && self.slash_count < 3
    }

    /// Update reputation based on validation performance
    pub fn update_reputation(&mut self, successful: bool) -> Result<()> {
        if successful {
            self.successful_validations = self.successful_validations.saturating_add(1);
            // Increase reputation (max 1000)
            self.reputation = (self.reputation.saturating_add(5)).min(1000);
        } else {
            self.failed_validations = self.failed_validations.saturating_add(1);
            // Decrease reputation
            self.reputation = self.reputation.saturating_sub(10);
        }

        self.last_activity = Clock::get()?.unix_timestamp;
        
        // Auto-slash if too many failures
        let total_validations = self.successful_validations + self.failed_validations;
        if total_validations > 100 {
            let failure_rate = (self.failed_validations * 100) / total_validations;
            if failure_rate > 20 { // More than 20% failure rate
                self.status = ValidatorStatus::Slashed;
                self.slash_count = self.slash_count.saturating_add(1);
            }
        }

        Ok(())
    }
}

/// Validator set account storing all validator information
#[account]
pub struct ValidatorSet {
    /// Bridge configuration this validator set belongs to
    pub bridge_config: Pubkey,
    /// Current epoch number
    pub current_epoch: u64,
    /// Total number of validators
    pub validator_count: u16,
    /// Active validator count
    pub active_validator_count: u16,
    /// Total stake in the validator set
    pub total_stake: u64,
    /// Minimum threshold for consensus (e.g., 67% for BFT)
    pub consensus_threshold: u16, // basis points (6700 = 67%)
    /// Last update timestamp
    pub last_updated: i64,
    /// Validator rotation frequency (in seconds)
    pub rotation_frequency: u64,
    /// Next rotation timestamp
    pub next_rotation: i64,
    /// Validators array (up to MAX_VALIDATORS)
    pub validators: Vec<ValidatorInfo>,
    /// Validator indices mapping for O(1) lookups
    pub validator_indices: BTreeMap<Pubkey, usize>,
    /// Epoch history for the last 10 epochs
    pub epoch_history: Vec<EpochInfo>,
    /// Slashing parameters
    pub slashing_config: SlashingConfig,
    /// Reward distribution parameters
    pub reward_config: RewardConfig,
    /// Geographic distribution requirements
    pub geo_distribution: GeographicDistribution,
    /// Version requirements
    pub version_requirements: VersionRequirements,
}

impl ValidatorSet {
    /// Account space calculation
    pub const MAX_SIZE: usize = 8 + // discriminator
        32 + // bridge_config
        8 + // current_epoch
        2 + // validator_count
        2 + // active_validator_count
        8 + // total_stake
        2 + // consensus_threshold
        8 + // last_updated
        8 + // rotation_frequency
        8 + // next_rotation
        4 + (MAX_VALIDATORS * ValidatorInfo::MAX_SIZE) + // validators
        4 + (MAX_VALIDATORS * (32 + 8)) + // validator_indices (approx)
        4 + (10 * EpochInfo::MAX_SIZE) + // epoch_history
        SlashingConfig::MAX_SIZE + // slashing_config
        RewardConfig::MAX_SIZE + // reward_config
        GeographicDistribution::MAX_SIZE + // geo_distribution
        VersionRequirements::MAX_SIZE; // version_requirements

    /// Initialize a new validator set
    pub fn initialize(
        &mut self,
        bridge_config: Pubkey,
        consensus_threshold: u16,
        rotation_frequency: u64,
    ) -> Result<()> {
        require!(consensus_threshold >= 5100 && consensus_threshold <= 9000, 
                crate::errors::BridgeError::InvalidConsensusThreshold);
        require!(rotation_frequency >= 3600, // Minimum 1 hour
                crate::errors::BridgeError::InvalidRotationFrequency);

        let clock = Clock::get()?;
        
        self.bridge_config = bridge_config;
        self.current_epoch = 0;
        self.validator_count = 0;
        self.active_validator_count = 0;
        self.total_stake = 0;
        self.consensus_threshold = consensus_threshold;
        self.last_updated = clock.unix_timestamp;
        self.rotation_frequency = rotation_frequency;
        self.next_rotation = clock.unix_timestamp + rotation_frequency as i64;
        self.validators = Vec::with_capacity(MAX_VALIDATORS);
        self.validator_indices = BTreeMap::new();
        self.epoch_history = Vec::with_capacity(10);
        
        // Initialize default slashing configuration
        self.slashing_config = SlashingConfig::default();
        self.reward_config = RewardConfig::default();
        self.geo_distribution = GeographicDistribution::default();
        self.version_requirements = VersionRequirements::default();

        Ok(())
    }

    /// Add a new validator to the set
    pub fn add_validator(
        &mut self,
        validator_pubkey: Pubkey,
        stake: u64,
        network_id: u8,
        commission_rate: u16,
        region_code: u8,
    ) -> Result<()> {
        require!(self.validator_count < MAX_VALIDATORS as u16, 
                crate::errors::BridgeError::ValidatorSetFull);
        require!(stake >= MIN_VALIDATOR_STAKE, 
                crate::errors::BridgeError::InsufficientStake);
        require!(commission_rate <= 10000, 
                crate::errors::BridgeError::InvalidCommissionRate);
        require!(!self.validator_indices.contains_key(&validator_pubkey), 
                crate::errors::BridgeError::ValidatorAlreadyExists);

        let clock = Clock::get()?;
        let validator_info = ValidatorInfo {
            pubkey: validator_pubkey,
            stake,
            status: ValidatorStatus::Pending,
            successful_validations: 0,
            failed_validations: 0,
            joined_at: clock.unix_timestamp,
            last_activity: clock.unix_timestamp,
            reputation: 500, // Start with neutral reputation
            slash_count: 0,
            total_rewards: 0,
            network_id,
            commission_rate,
            region_code,
            version: self.version_requirements.minimum_version,
        };

        let index = self.validators.len();
        self.validators.push(validator_info);
        self.validator_indices.insert(validator_pubkey, index);
        self.validator_count += 1;
        self.total_stake += stake;
        self.last_updated = clock.unix_timestamp;

        // Check if we can activate the validator
        self.try_activate_validator(index)?;

        Ok(())
    }

    /// Remove a validator from the set
    pub fn remove_validator(&mut self, validator_pubkey: Pubkey) -> Result<u64> {
        let index = *self.validator_indices.get(&validator_pubkey)
            .ok_or(crate::errors::BridgeError::ValidatorNotFound)?;

        let validator = &self.validators[index];
        let stake_to_return = validator.stake;

        // Update counters
        if validator.status == ValidatorStatus::Active {
            self.active_validator_count -= 1;
        }
        self.validator_count -= 1;
        self.total_stake -= validator.stake;

        // Remove from arrays (swap with last element for O(1) removal)
        let last_index = self.validators.len() - 1;
        if index != last_index {
            let last_validator_pubkey = self.validators[last_index].pubkey;
            self.validators.swap(index, last_index);
            self.validator_indices.insert(last_validator_pubkey, index);
        }
        
        self.validators.pop();
        self.validator_indices.remove(&validator_pubkey);
        self.last_updated = Clock::get()?.unix_timestamp;

        Ok(stake_to_return)
    }

    /// Activate a pending validator
    pub fn try_activate_validator(&mut self, index: usize) -> Result<()> {
        require!(index < self.validators.len(), 
                crate::errors::BridgeError::ValidatorNotFound);

        let validator = &mut self.validators[index];
        if validator.status != ValidatorStatus::Pending {
            return Ok(()); // Already processed
        }

        // Check activation requirements
        if self.check_activation_requirements(validator)? {
            validator.status = ValidatorStatus::Active;
            self.active_validator_count += 1;
            self.last_updated = Clock::get()?.unix_timestamp;
        }

        Ok(())
    }

    /// Check if a validator meets activation requirements
    fn check_activation_requirements(&self, validator: &ValidatorInfo) -> Result<bool> {
        // Check minimum stake
        if validator.stake < MIN_VALIDATOR_STAKE {
            return Ok(false);
        }

        // Check version requirements
        if validator.version < self.version_requirements.minimum_version {
            return Ok(false);
        }

        // Check geographic distribution if required
        if self.geo_distribution.enforce_distribution {
            let region_count = self.validators.iter()
                .filter(|v| v.status == ValidatorStatus::Active && v.region_code == validator.region_code)
                .count();
            
            let max_per_region = (self.active_validator_count as usize * 
                self.geo_distribution.max_percentage_per_region as usize) / 10000;
            
            if region_count >= max_per_region {
                return Ok(false);
            }
        }

        Ok(true)
    }

    /// Slash a validator for malicious behavior
    pub fn slash_validator(
        &mut self, 
        validator_pubkey: Pubkey, 
        slash_type: SlashType
    ) -> Result<u64> {
        let index = *self.validator_indices.get(&validator_pubkey)
            .ok_or(crate::errors::BridgeError::ValidatorNotFound)?;

        let validator = &mut self.validators[index];
        let slash_amount = self.calculate_slash_amount(validator, &slash_type);

        validator.status = ValidatorStatus::Slashed;
        validator.slash_count = validator.slash_count.saturating_add(1);
        validator.stake = validator.stake.saturating_sub(slash_amount);
        validator.reputation = validator.reputation.saturating_sub(100);

        // If slashed too many times, ban the validator
        if validator.slash_count >= self.slashing_config.max_slashes_before_ban {
            validator.status = ValidatorStatus::Banned;
            if validator.status == ValidatorStatus::Active {
                self.active_validator_count -= 1;
            }
        }

        self.total_stake = self.total_stake.saturating_sub(slash_amount);
        self.last_updated = Clock::get()?.unix_timestamp;

        // Record slashing event in epoch history
        self.record_slash_event(validator_pubkey, slash_type, slash_amount)?;

        Ok(slash_amount)
    }

    /// Calculate slash amount based on type and validator stake
    fn calculate_slash_amount(&self, validator: &ValidatorInfo, slash_type: &SlashType) -> u64 {
        let base_stake = validator.stake;
        let slash_percentage = match slash_type {
            SlashType::DoubleSigning => self.slashing_config.double_signing_slash_rate,
            SlashType::Unavailability => self.slashing_config.unavailability_slash_rate,
            SlashType::InvalidSignature => self.slashing_config.invalid_signature_slash_rate,
            SlashType::Malicious => self.slashing_config.malicious_behavior_slash_rate,
        };

        (base_stake * slash_percentage as u64) / 10000
    }

    /// Start a new epoch and rotate validators if necessary
    pub fn rotate_epoch(&mut self) -> Result<()> {
        let clock = Clock::get()?;
        
        require!(clock.unix_timestamp >= self.next_rotation, 
                crate::errors::BridgeError::RotationNotDue);

        // Record current epoch info
        let epoch_info = EpochInfo {
            epoch: self.current_epoch,
            start_time: self.last_updated,
            end_time: clock.unix_timestamp,
            active_validators: self.active_validator_count,
            total_validations: self.calculate_total_validations(),
            successful_validations: self.calculate_successful_validations(),
            total_rewards_distributed: self.calculate_epoch_rewards(),
            average_reputation: self.calculate_average_reputation(),
        };

        // Add to history (keep only last 10 epochs)
        self.epoch_history.push(epoch_info);
        if self.epoch_history.len() > 10 {
            self.epoch_history.remove(0);
        }

        // Update epoch
        self.current_epoch += 1;
        self.next_rotation = clock.unix_timestamp + self.rotation_frequency as i64;
        self.last_updated = clock.unix_timestamp;

        // Perform validator set adjustments
        self.adjust_validator_set()?;

        Ok(())
    }

    /// Adjust validator set based on performance and requirements
    fn adjust_validator_set(&mut self) -> Result<()> {
        // Activate pending validators that meet requirements
        let mut indices_to_check: Vec<usize> = (0..self.validators.len()).collect();
        for index in indices_to_check {
            if index < self.validators.len() {
                self.try_activate_validator(index)?;
            }
        }

        // Deactivate poorly performing validators
        for validator in &mut self.validators {
            if validator.status == ValidatorStatus::Active {
                if validator.reputation < self.slashing_config.min_reputation_threshold {
                    validator.status = ValidatorStatus::Inactive;
                    self.active_validator_count -= 1;
                }
            }
        }

        Ok(())
    }

    /// Get validator by public key
    pub fn get_validator(&self, validator_pubkey: &Pubkey) -> Option<&ValidatorInfo> {
        self.validator_indices.get(validator_pubkey)
            .and_then(|&index| self.validators.get(index))
    }

    /// Get mutable validator by public key
    pub fn get_validator_mut(&mut self, validator_pubkey: &Pubkey) -> Option<&mut ValidatorInfo> {
        if let Some(&index) = self.validator_indices.get(validator_pubkey) {
            self.validators.get_mut(index)
        } else {
            None
        }
    }

    /// Get active validators sorted by voting power
    pub fn get_active_validators_by_power(&self) -> Vec<&ValidatorInfo> {
        let mut active_validators: Vec<&ValidatorInfo> = self.validators.iter()
            .filter(|v| v.status == ValidatorStatus::Active)
            .collect();

        active_validators.sort_by(|a, b| {
            b.calculate_voting_power().cmp(&a.calculate_voting_power())
        });

        active_validators
    }

    /// Check if consensus threshold is met for a given set of signers
    pub fn check_consensus(&self, signers: &[Pubkey]) -> Result<bool> {
        let mut total_voting_power = 0u64;
        let mut signer_voting_power = 0u64;

        // Calculate total voting power of active validators
        for validator in &self.validators {
            if validator.status == ValidatorStatus::Active {
                total_voting_power += validator.calculate_voting_power();
            }
        }

        // Calculate voting power of signers
        for signer in signers {
            if let Some(validator) = self.get_validator(signer) {
                if validator.status == ValidatorStatus::Active {
                    signer_voting_power += validator.calculate_voting_power();
                }
            }
        }

        if total_voting_power == 0 {
            return Ok(false);
        }

        let consensus_percentage = (signer_voting_power * 10000) / total_voting_power;
        Ok(consensus_percentage >= self.consensus_threshold as u64)
    }

    /// Calculate total validations across all validators
    fn calculate_total_validations(&self) -> u64 {
        self.validators.iter()
            .map(|v| v.successful_validations + v.failed_validations)
            .sum()
    }

    /// Calculate successful validations across all validators
    fn calculate_successful_validations(&self) -> u64 {
        self.validators.iter()
            .map(|v| v.successful_validations)
            .sum()
    }

    /// Calculate total rewards for the epoch
    fn calculate_epoch_rewards(&self) -> u64 {
        self.validators.iter()
            .map(|v| v.total_rewards)
            .sum()
    }

    /// Calculate average reputation
    fn calculate_average_reputation(&self) -> u16 {
        if self.validators.is_empty() {
            return 0;
        }

        let total_reputation: u32 = self.validators.iter()
            .map(|v| v.reputation as u32)
            .sum();

        (total_reputation / self.validators.len() as u32) as u16
    }

    /// Record a slash event
    fn record_slash_event(
        &mut self, 
        validator: Pubkey, 
        slash_type: SlashType, 
        amount: u64
    ) -> Result<()> {
        // Implementation would record the slash event for auditing
        // This could be stored in the epoch history or a separate log
        Ok(())
    }

    /// Update validator stake (for staking/unstaking)
    pub fn update_validator_stake(
        &mut self, 
        validator_pubkey: Pubkey, 
        new_stake: u64
    ) -> Result<()> {
        let validator = self.get_validator_mut(&validator_pubkey)
            .ok_or(crate::errors::BridgeError::ValidatorNotFound)?;

        let old_stake = validator.stake;
        validator.stake = new_stake;
        
        // Update total stake
        self.total_stake = self.total_stake.saturating_sub(old_stake).saturating_add(new_stake);

        // Check if validator should be deactivated due to insufficient stake
        if new_stake < MIN_VALIDATOR_STAKE && validator.status == ValidatorStatus::Active {
            validator.status = ValidatorStatus::Inactive;
            self.active_validator_count -= 1;
        }

        self.last_updated = Clock::get()?.unix_timestamp;
        Ok(())
    }
}

/// Information about a completed epoch
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct EpochInfo {
    pub epoch: u64,
    pub start_time: i64,
    pub end_time: i64,
    pub active_validators: u16,
    pub total_validations: u64,
    pub successful_validations: u64,
    pub total_rewards_distributed: u64,
    pub average_reputation: u16,
}

impl EpochInfo {
    pub const MAX_SIZE: usize = 8 + 8 + 8 + 2 + 8 + 8 + 8 + 2;
}

/// Slashing configuration parameters
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct SlashingConfig {
    /// Slash rate for double signing (basis points)
    pub double_signing_slash_rate: u16,
    /// Slash rate for unavailability (basis points)
    pub unavailability_slash_rate: u16,
    /// Slash rate for invalid signatures (basis points)
    pub invalid_signature_slash_rate: u16,
    /// Slash rate for malicious behavior (basis points)
    pub malicious_behavior_slash_rate: u16,
    /// Maximum slashes before permanent ban
    pub max_slashes_before_ban: u8,
    /// Minimum reputation threshold to stay active
    pub min_reputation_threshold: u16,
}

impl Default for SlashingConfig {
    fn default() -> Self {
        Self {
            double_signing_slash_rate: 500,  // 5%
            unavailability_slash_rate: 100,  // 1%
            invalid_signature_slash_rate: 200, // 2%
            malicious_behavior_slash_rate: 1000, // 10%
            max_slashes_before_ban: 3,
            min_reputation_threshold: 200,
        }
    }
}

impl SlashingConfig {
    pub const MAX_SIZE: usize = 2 + 2 + 2 + 2 + 1 + 2;
}

/// Reward distribution configuration
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct RewardConfig {
    /// Base reward per validation (in smallest token unit)
    pub base_reward_per_validation: u64,
    /// Bonus multiplier for high reputation (basis points)
    pub reputation_bonus_multiplier: u16,
    /// Commission cap (basis points)
    pub max_commission_rate: u16,
    /// Minimum validations for reward eligibility
    pub min_validations_for_rewards: u32,
}

impl Default for RewardConfig {
    fn default() -> Self {
        Self {
            base_reward_per_validation: 1000,
            reputation_bonus_multiplier: 2000, // 20% max bonus
            max_commission_rate: 1000, // 10% max commission
            min_validations_for_rewards: 10,
        }
    }
}

impl RewardConfig {
    pub const MAX_SIZE: usize = 8 + 2 + 2 + 4;
}

/// Geographic distribution requirements
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct GeographicDistribution {
    /// Whether to enforce geographic distribution
    pub enforce_distribution: bool,
    /// Maximum percentage of validators per region (basis points)
    pub max_percentage_per_region: u16,
    /// Required minimum number of regions
    pub min_regions_required: u8,
}

impl Default for GeographicDistribution {
    fn default() -> Self {
        Self {
            enforce_distribution: true,
            max_percentage_per_region: 3000, // 30% max per region
            min_regions_required: 3,
        }
    }
}

impl GeographicDistribution {
    pub const MAX_SIZE: usize = 1 + 2 + 1;
}

/// Version requirements for validators
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct VersionRequirements {
    /// Minimum required version
    pub minimum_version: u32,
    /// Recommended version
    pub recommended_version: u32,
    /// Grace period for upgrades (in seconds)
    pub upgrade_grace_period: u64,
}

impl Default for VersionRequirements {
    fn default() -> Self {
        Self {
            minimum_version: 1,
            recommended_version: 1,
            upgrade_grace_period: 604800, // 1 week
        }
    }
}

impl VersionRequirements {
    pub const MAX_SIZE: usize = 4 + 4 + 8;
}

/// Types of slashing events
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub enum SlashType {
    /// Validator signed conflicting messages
    DoubleSigning,
    /// Validator was unavailable for too long
    Unavailability,
    /// Validator provided invalid signature
    InvalidSignature,
    /// Validator exhibited malicious behavior
    Malicious,
}

impl ValidatorInfo {
    pub const MAX_SIZE: usize = 
        32 + // pubkey
        8 +  // stake
        1 +  // status (enum)
        8 +  // successful_validations
        8 +  // failed_validations
        8 +  // joined_at
        8 +  // last_activity
        2 +  // reputation
        1 +  // slash_count
        8 +  // total_rewards
        1 +  // network_id
        2 +  // commission_rate
        1 +  // region_code
        4;   // version
}
