// programs/finova-bridge/src/instructions/emergency_pause.rs

use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Mint, Transfer, MintTo, Burn};
use crate::state::{BridgeConfig, LockedTokens, ValidatorSet};
use crate::errors::BridgeError;
use crate::utils::{verify_validator_signature, calculate_pause_hash};
use crate::constants::*;
use crate::cryptography::signature_verification::verify_multi_sig;
use std::collections::HashMap;

/// Emergency pause instruction to halt all bridge operations in case of security threats
/// Implements multi-signature governance with time-locked recovery mechanisms
#[derive(Accounts)]
#[instruction(pause_type: u8, reason_code: u32)]
pub struct EmergencyPause<'info> {
    #[account(
        mut,
        seeds = [BRIDGE_CONFIG_SEED],
        bump,
        constraint = bridge_config.is_initialized @ BridgeError::NotInitialized,
        constraint = !bridge_config.is_permanently_disabled @ BridgeError::PermanentlyDisabled
    )]
    pub bridge_config: Account<'info, BridgeConfig>,

    #[account(
        seeds = [VALIDATOR_SET_SEED],
        bump,
        constraint = validator_set.is_active @ BridgeError::ValidatorSetInactive
    )]
    pub validator_set: Account<'info, ValidatorSet>,

    #[account(
        mut,
        constraint = authority.key() == bridge_config.emergency_authority 
            || validator_set.validators.contains(&authority.key()) @ BridgeError::UnauthorizedEmergencyPause
    )]
    pub authority: Signer<'info>,

    /// Emergency multisig account for critical operations
    #[account(
        constraint = emergency_multisig.key() == bridge_config.emergency_multisig @ BridgeError::InvalidEmergencyMultisig
    )]
    pub emergency_multisig: Option<Account<'info, TokenAccount>>,

    pub system_program: Program<'info, System>,
    pub clock: Sysvar<'info, Clock>,
}

/// Pause levels with increasing severity
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq)]
pub enum PauseLevel {
    /// Level 1: Pause new locks only
    NewLocksOnly = 0,
    /// Level 2: Pause all locking operations
    AllLocks = 1,
    /// Level 3: Pause all unlock operations
    AllUnlocks = 2,
    /// Level 4: Complete bridge halt
    CompletePause = 3,
    /// Level 5: Emergency shutdown with asset protection
    EmergencyShutdown = 4,
}

/// Emergency pause reasons for audit trail
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq)]
pub enum PauseReason {
    SuspiciousActivity = 1000,
    ValidatorCompromise = 1001,
    SmartContractVulnerability = 1002,
    OracleManipulation = 1003,
    GovernanceAttack = 1004,
    ExternalChainIssue = 1005,
    LiquidityDrain = 1006,
    FlashLoanAttack = 1007,
    ReentrantAttack = 1008,
    UnauthorizedMinting = 1009,
    ValidatorCollusion = 1010,
    NetworkCongestion = 1011,
    RegulatoryCompliance = 1012,
    MaintenanceUpgrade = 1013,
    TestingPurpose = 1014,
}

/// Emergency pause state tracking
#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct EmergencyPauseState {
    pub pause_level: PauseLevel,
    pub reason_code: u32,
    pub initiated_by: Pubkey,
    pub initiated_at: i64,
    pub signatures_required: u8,
    pub signatures_collected: u8,
    pub validator_signatures: Vec<ValidatorSignature>,
    pub auto_resume_at: Option<i64>,
    pub manual_resume_required: bool,
    pub asset_protection_enabled: bool,
    pub recovery_multisig_threshold: u8,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct ValidatorSignature {
    pub validator: Pubkey,
    pub signature: [u8; 64],
    pub timestamp: i64,
    pub message_hash: [u8; 32],
}

impl<'info> EmergencyPause<'info> {
    /// Execute emergency pause with comprehensive security checks
    pub fn execute_emergency_pause(
        &mut self,
        pause_type: u8,
        reason_code: u32,
        auto_resume_minutes: Option<u32>,
        validator_signatures: Vec<ValidatorSignature>,
    ) -> Result<()> {
        let clock = &self.clock;
        let current_time = clock.unix_timestamp;

        // Convert pause type to enum
        let pause_level = match pause_type {
            0 => PauseLevel::NewLocksOnly,
            1 => PauseLevel::AllLocks,
            2 => PauseLevel::AllUnlocks,
            3 => PauseLevel::CompletePause,
            4 => PauseLevel::EmergencyShutdown,
            _ => return Err(BridgeError::InvalidPauseLevel.into()),
        };

        // Validate reason code
        self.validate_pause_reason(reason_code)?;

        // Check authority and signature requirements
        let required_signatures = self.calculate_required_signatures(pause_level)?;
        self.verify_pause_authorization(pause_level, &validator_signatures, required_signatures)?;

        // Execute pause based on level
        match pause_level {
            PauseLevel::NewLocksOnly => self.pause_new_locks()?,
            PauseLevel::AllLocks => self.pause_all_locks()?,
            PauseLevel::AllUnlocks => self.pause_all_unlocks()?,
            PauseLevel::CompletePause => self.pause_complete_bridge()?,
            PauseLevel::EmergencyShutdown => self.emergency_shutdown()?,
        }

        // Set auto-resume if specified
        let auto_resume_at = auto_resume_minutes.map(|minutes| {
            current_time + (minutes as i64 * 60)
        });

        // Update bridge config with pause state
        self.bridge_config.emergency_pause_state = Some(EmergencyPauseState {
            pause_level,
            reason_code,
            initiated_by: self.authority.key(),
            initiated_at: current_time,
            signatures_required: required_signatures,
            signatures_collected: validator_signatures.len() as u8,
            validator_signatures,
            auto_resume_at,
            manual_resume_required: matches!(pause_level, PauseLevel::EmergencyShutdown),
            asset_protection_enabled: matches!(pause_level, PauseLevel::EmergencyShutdown),
            recovery_multisig_threshold: self.calculate_recovery_threshold(pause_level),
        });

        // Update operational flags
        self.bridge_config.is_paused = true;
        self.bridge_config.pause_initiated_at = current_time;
        self.bridge_config.last_emergency_action = current_time;

        // Emit emergency pause event
        emit!(EmergencyPauseEvent {
            pause_level: pause_level as u8,
            reason_code,
            initiated_by: self.authority.key(),
            initiated_at: current_time,
            auto_resume_at,
            total_locked_value: self.bridge_config.total_locked_value,
            affected_chains: self.bridge_config.supported_chains.clone(),
        });

        msg!("Emergency pause executed: Level {}, Reason: {}", pause_type, reason_code);
        Ok(())
    }

    /// Validate pause reason code
    fn validate_pause_reason(&self, reason_code: u32) -> Result<()> {
        match reason_code {
            1000..=1014 => Ok(()),
            _ => Err(BridgeError::InvalidPauseReason.into()),
        }
    }

    /// Calculate required signatures based on pause severity
    fn calculate_required_signatures(&self, pause_level: PauseLevel) -> Result<u8> {
        let total_validators = self.validator_set.validators.len() as u8;
        
        let required = match pause_level {
            PauseLevel::NewLocksOnly => (total_validators / 3).max(1), // 33%
            PauseLevel::AllLocks => (total_validators / 2).max(2), // 50%
            PauseLevel::AllUnlocks => (total_validators * 2 / 3).max(2), // 67%
            PauseLevel::CompletePause => (total_validators * 3 / 4).max(3), // 75%
            PauseLevel::EmergencyShutdown => (total_validators * 4 / 5).max(4), // 80%
        };

        Ok(required.min(total_validators))
    }

    /// Verify pause authorization with multi-signature validation
    fn verify_pause_authorization(
        &self,
        pause_level: PauseLevel,
        signatures: &[ValidatorSignature],
        required_signatures: u8,
    ) -> Result<()> {
        // Emergency authority can execute lower level pauses without signatures
        if self.authority.key() == self.bridge_config.emergency_authority {
            match pause_level {
                PauseLevel::NewLocksOnly | PauseLevel::AllLocks => return Ok(()),
                _ => {}, // Higher levels require validator signatures
            }
        }

        // Verify we have enough signatures
        require!(
            signatures.len() >= required_signatures as usize,
            BridgeError::InsufficientSignatures
        );

        // Verify each signature
        let message_hash = self.calculate_pause_message_hash(pause_level)?;
        let current_time = self.clock.unix_timestamp;

        for sig in signatures {
            // Check signature age (must be within last 10 minutes)
            require!(
                current_time - sig.timestamp <= 600,
                BridgeError::SignatureTooOld
            );

            // Verify validator is active
            require!(
                self.validator_set.validators.contains(&sig.validator),
                BridgeError::UnknownValidator
            );

            // Verify message hash matches
            require!(
                sig.message_hash == message_hash,
                BridgeError::InvalidMessageHash
            );

            // Verify signature
            verify_validator_signature(&sig.validator, &message_hash, &sig.signature)?;
        }

        Ok(())
    }

    /// Calculate message hash for pause operation
    fn calculate_pause_message_hash(&self, pause_level: PauseLevel) -> Result<[u8; 32]> {
        use anchor_lang::solana_program::hash::{hash, Hash};
        
        let message = format!(
            "EMERGENCY_PAUSE:{}:{}:{}:{}",
            pause_level as u8,
            self.bridge_config.key(),
            self.clock.unix_timestamp / 300, // 5-minute window
            self.bridge_config.nonce
        );

        Ok(hash(message.as_bytes()).to_bytes())
    }

    /// Pause new lock operations only
    fn pause_new_locks(&mut self) -> Result<()> {
        self.bridge_config.new_locks_paused = true;
        msg!("New locks paused");
        Ok(())
    }

    /// Pause all lock operations
    fn pause_all_locks(&mut self) -> Result<()> {
        self.bridge_config.new_locks_paused = true;
        self.bridge_config.all_locks_paused = true;
        msg!("All locks paused");
        Ok(())
    }

    /// Pause all unlock operations
    fn pause_all_unlocks(&mut self) -> Result<()> {
        self.bridge_config.unlocks_paused = true;
        msg!("All unlocks paused");
        Ok(())
    }

    /// Pause complete bridge operations
    fn pause_complete_bridge(&mut self) -> Result<()> {
        self.bridge_config.new_locks_paused = true;
        self.bridge_config.all_locks_paused = true;
        self.bridge_config.unlocks_paused = true;
        self.bridge_config.validator_operations_paused = true;
        msg!("Complete bridge paused");
        Ok(())
    }

    /// Emergency shutdown with asset protection
    fn emergency_shutdown(&mut self) -> Result<()> {
        self.bridge_config.new_locks_paused = true;
        self.bridge_config.all_locks_paused = true;
        self.bridge_config.unlocks_paused = true;
        self.bridge_config.validator_operations_paused = true;
        self.bridge_config.oracle_updates_paused = true;
        self.bridge_config.emergency_mode = true;
        
        // Enable asset protection mode
        self.bridge_config.asset_protection_mode = true;
        self.bridge_config.recovery_mode_enabled = true;
        
        msg!("Emergency shutdown activated with asset protection");
        Ok(())
    }

    /// Calculate recovery threshold based on pause level
    fn calculate_recovery_threshold(&self, pause_level: PauseLevel) -> u8 {
        let total_validators = self.validator_set.validators.len() as u8;
        
        match pause_level {
            PauseLevel::NewLocksOnly => 1,
            PauseLevel::AllLocks => 2,
            PauseLevel::AllUnlocks => (total_validators / 2).max(2),
            PauseLevel::CompletePause => (total_validators * 2 / 3).max(3),
            PauseLevel::EmergencyShutdown => (total_validators * 4 / 5).max(4),
        }
    }

    /// Resume operations from pause state
    pub fn resume_operations(
        &mut self,
        resume_signatures: Vec<ValidatorSignature>,
    ) -> Result<()> {
        let pause_state = self.bridge_config.emergency_pause_state
            .as_ref()
            .ok_or(BridgeError::NotInPauseState)?;

        let current_time = self.clock.unix_timestamp;

        // Check if auto-resume time has passed
        if let Some(auto_resume_at) = pause_state.auto_resume_at {
            if current_time >= auto_resume_at && !pause_state.manual_resume_required {
                return self.auto_resume_operations();
            }
        }

        // Verify manual resume signatures
        require!(
            resume_signatures.len() >= pause_state.recovery_multisig_threshold as usize,
            BridgeError::InsufficientResumeSignatures
        );

        // Verify resume signatures
        let resume_message_hash = self.calculate_resume_message_hash()?;
        for sig in &resume_signatures {
            require!(
                self.validator_set.validators.contains(&sig.validator),
                BridgeError::UnknownValidator
            );
            
            verify_validator_signature(&sig.validator, &resume_message_hash, &sig.signature)?;
        }

        // Execute resume
        self.execute_resume()?;

        emit!(EmergencyResumeEvent {
            resumed_by: self.authority.key(),
            resumed_at: current_time,
            pause_duration: current_time - pause_state.initiated_at,
            resume_signatures: resume_signatures.len() as u8,
        });

        msg!("Bridge operations resumed");
        Ok(())
    }

    /// Auto-resume operations when timer expires
    fn auto_resume_operations(&mut self) -> Result<()> {
        let current_time = self.clock.unix_timestamp;
        
        self.execute_resume()?;

        emit!(EmergencyAutoResumeEvent {
            resumed_at: current_time,
            auto_resume_triggered: true,
        });

        msg!("Bridge operations auto-resumed");
        Ok(())
    }

    /// Execute the actual resume operation
    fn execute_resume(&mut self) -> Result<()> {
        // Clear all pause flags
        self.bridge_config.is_paused = false;
        self.bridge_config.new_locks_paused = false;
        self.bridge_config.all_locks_paused = false;
        self.bridge_config.unlocks_paused = false;
        self.bridge_config.validator_operations_paused = false;
        self.bridge_config.oracle_updates_paused = false;
        self.bridge_config.emergency_mode = false;
        
        // Clear emergency pause state
        self.bridge_config.emergency_pause_state = None;
        self.bridge_config.pause_initiated_at = 0;
        
        // Increment nonce for security
        self.bridge_config.nonce += 1;

        Ok(())
    }

    /// Calculate message hash for resume operation
    fn calculate_resume_message_hash(&self) -> Result<[u8; 32]> {
        use anchor_lang::solana_program::hash::{hash, Hash};
        
        let message = format!(
            "EMERGENCY_RESUME:{}:{}:{}",
            self.bridge_config.key(),
            self.clock.unix_timestamp / 300, // 5-minute window
            self.bridge_config.nonce
        );

        Ok(hash(message.as_bytes()).to_bytes())
    }

    /// Check if operations are allowed based on current pause state
    pub fn check_operation_allowed(&self, operation_type: &str) -> Result<()> {
        if !self.bridge_config.is_paused {
            return Ok(());
        }

        let pause_state = self.bridge_config.emergency_pause_state
            .as_ref()
            .ok_or(BridgeError::NotInPauseState)?;

        match (operation_type, pause_state.pause_level) {
            ("new_lock", PauseLevel::NewLocksOnly) => Err(BridgeError::OperationPaused.into()),
            ("lock", PauseLevel::AllLocks | PauseLevel::CompletePause | PauseLevel::EmergencyShutdown) => {
                Err(BridgeError::OperationPaused.into())
            },
            ("unlock", PauseLevel::AllUnlocks | PauseLevel::CompletePause | PauseLevel::EmergencyShutdown) => {
                Err(BridgeError::OperationPaused.into())
            },
            ("validator_op", PauseLevel::CompletePause | PauseLevel::EmergencyShutdown) => {
                Err(BridgeError::OperationPaused.into())
            },
            _ => Ok(()),
        }
    }

    /// Get current pause status for external queries
    pub fn get_pause_status(&self) -> PauseStatus {
        if let Some(pause_state) = &self.bridge_config.emergency_pause_state {
            PauseStatus {
                is_paused: true,
                pause_level: pause_state.pause_level as u8,
                reason_code: pause_state.reason_code,
                initiated_at: pause_state.initiated_at,
                auto_resume_at: pause_state.auto_resume_at,
                signatures_required: pause_state.signatures_required,
                signatures_collected: pause_state.signatures_collected,
            }
        } else {
            PauseStatus {
                is_paused: false,
                pause_level: 0,
                reason_code: 0,
                initiated_at: 0,
                auto_resume_at: None,
                signatures_required: 0,
                signatures_collected: 0,
            }
        }
    }
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct PauseStatus {
    pub is_paused: bool,
    pub pause_level: u8,
    pub reason_code: u32,
    pub initiated_at: i64,
    pub auto_resume_at: Option<i64>,
    pub signatures_required: u8,
    pub signatures_collected: u8,
}

// Events for emergency pause operations
#[event]
pub struct EmergencyPauseEvent {
    pub pause_level: u8,
    pub reason_code: u32,
    pub initiated_by: Pubkey,
    pub initiated_at: i64,
    pub auto_resume_at: Option<i64>,
    pub total_locked_value: u64,
    pub affected_chains: Vec<u32>,
}

#[event]
pub struct EmergencyResumeEvent {
    pub resumed_by: Pubkey,
    pub resumed_at: i64,
    pub pause_duration: i64,
    pub resume_signatures: u8,
}

#[event]
pub struct EmergencyAutoResumeEvent {
    pub resumed_at: i64,
    pub auto_resume_triggered: bool,
}

#[event]
pub struct PauseStatusQueryEvent {
    pub is_paused: bool,
    pub pause_level: u8,
    pub reason_code: u32,
    pub queried_at: i64,
}

/// Instruction for querying pause status
#[derive(Accounts)]
pub struct QueryPauseStatus<'info> {
    #[account(
        seeds = [BRIDGE_CONFIG_SEED],
        bump
    )]
    pub bridge_config: Account<'info, BridgeConfig>,
    
    pub clock: Sysvar<'info, Clock>,
}

impl<'info> QueryPauseStatus<'info> {
    pub fn query_status(&self) -> Result<PauseStatus> {
        let status = if let Some(pause_state) = &self.bridge_config.emergency_pause_state {
            PauseStatus {
                is_paused: true,
                pause_level: pause_state.pause_level as u8,
                reason_code: pause_state.reason_code,
                initiated_at: pause_state.initiated_at,
                auto_resume_at: pause_state.auto_resume_at,
                signatures_required: pause_state.signatures_required,
                signatures_collected: pause_state.signatures_collected,
            }
        } else {
            PauseStatus {
                is_paused: false,
                pause_level: 0,
                reason_code: 0,
                initiated_at: 0,
                auto_resume_at: None,
                signatures_required: 0,
                signatures_collected: 0,
            }
        };

        emit!(PauseStatusQueryEvent {
            is_paused: status.is_paused,
            pause_level: status.pause_level,
            reason_code: status.reason_code,
            queried_at: self.clock.unix_timestamp,
        });

        Ok(status)
    }
}

/// Helper functions for emergency pause validation
pub fn validate_emergency_conditions(
    bridge_config: &BridgeConfig,
    validator_set: &ValidatorSet,
    reason_code: u32,
) -> Result<()> {
    // Check if bridge is in a state that allows emergency pause
    require!(!bridge_config.is_permanently_disabled, BridgeError::PermanentlyDisabled);
    require!(validator_set.is_active, BridgeError::ValidatorSetInactive);
    
    // Validate emergency conditions based on reason
    match reason_code {
        1000 => {}, // Suspicious activity - always allowed
        1001 => {   // Validator compromise - check validator health
            require!(
                validator_set.unhealthy_validators.len() > validator_set.validators.len() / 3,
                BridgeError::InsufficientValidatorCompromise
            );
        },
        1002 => {}, // Smart contract vulnerability - always allowed
        1003 => {   // Oracle manipulation - check oracle deviation
            require!(
                bridge_config.oracle_deviation_detected,
                BridgeError::NoOracleDeviation
            );
        },
        _ => {}, // Other reasons allowed without specific conditions
    }

    Ok(())
}

/// Calculate the impact score of an emergency pause
pub fn calculate_pause_impact_score(
    pause_level: PauseLevel,
    total_locked_value: u64,
    active_operations: u32,
) -> u32 {
    let level_multiplier = match pause_level {
        PauseLevel::NewLocksOnly => 1,
        PauseLevel::AllLocks => 3,
        PauseLevel::AllUnlocks => 5,
        PauseLevel::CompletePause => 8,
        PauseLevel::EmergencyShutdown => 10,
    };

    let value_impact = (total_locked_value / 1_000_000).min(100) as u32; // Cap at 100
    let operation_impact = active_operations.min(50); // Cap at 50

    level_multiplier * (10 + value_impact + operation_impact)
}
