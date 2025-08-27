// programs/finova-oracle/src/instructions/emergency_update.rs

use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount};
use crate::constants::*;
use crate::errors::*;
use crate::state::*;
use crate::utils::*;

/// Emergency update instruction context
#[derive(Accounts)]
#[instruction(price_data: Vec<PriceUpdateData>)]
pub struct EmergencyUpdate<'info> {
    #[account(
        mut,
        has_one = authority @ OracleError::UnauthorizedAuthority,
        constraint = oracle_config.is_emergency_enabled @ OracleError::EmergencyModeDisabled
    )]
    pub oracle_config: Account<'info, OracleConfig>,

    /// Emergency authority (multisig or governance)
    #[account(
        constraint = authority.key() == oracle_config.emergency_authority @ OracleError::InvalidEmergencyAuthority
    )]
    pub authority: Signer<'info>,

    /// Emergency validator (additional security layer)
    #[account(
        constraint = emergency_validator.key() == oracle_config.emergency_validator @ OracleError::InvalidEmergencyValidator
    )]
    pub emergency_validator: Signer<'info>,

    /// System program
    pub system_program: Program<'info, System>,
    
    /// Clock for timestamp validation
    pub clock: Sysvar<'info, Clock>,
}

/// Emergency update with circuit breaker context
#[derive(Accounts)]
#[instruction(feed_id: String)]
pub struct EmergencyCircuitBreaker<'info> {
    #[account(
        mut,
        has_one = authority @ OracleError::UnauthorizedAuthority
    )]
    pub oracle_config: Account<'info, OracleConfig>,

    #[account(
        mut,
        seeds = [PRICE_FEED_SEED, feed_id.as_bytes()],
        bump = price_feed.bump,
        constraint = price_feed.is_active @ OracleError::FeedInactive
    )]
    pub price_feed: Account<'info, PriceFeed>,

    /// Circuit breaker authority
    #[account(
        constraint = authority.key() == oracle_config.circuit_breaker_authority @ OracleError::InvalidCircuitBreakerAuthority
    )]
    pub authority: Signer<'info>,

    pub system_program: Program<'info, System>,
    pub clock: Sysvar<'info, Clock>,
}

/// Batch emergency update context
#[derive(Accounts)]
#[instruction(updates: Vec<EmergencyBatchUpdate>)]
pub struct BatchEmergencyUpdate<'info> {
    #[account(
        mut,
        has_one = authority @ OracleError::UnauthorizedAuthority,
        constraint = oracle_config.is_emergency_enabled @ OracleError::EmergencyModeDisabled
    )]
    pub oracle_config: Account<'info, OracleConfig>,

    #[account(
        constraint = authority.key() == oracle_config.emergency_authority @ OracleError::InvalidEmergencyAuthority
    )]
    pub authority: Signer<'info>,

    #[account(
        constraint = emergency_validator.key() == oracle_config.emergency_validator @ OracleError::InvalidEmergencyValidator
    )]
    pub emergency_validator: Signer<'info>,

    pub system_program: Program<'info, System>,
    pub clock: Sysvar<'info, Clock>,
}

/// Recovery mode activation context
#[derive(Accounts)]
pub struct ActivateRecoveryMode<'info> {
    #[account(
        mut,
        has_one = authority @ OracleError::UnauthorizedAuthority
    )]
    pub oracle_config: Account<'info, OracleConfig>,

    #[account(
        constraint = authority.key() == oracle_config.emergency_authority @ OracleError::InvalidEmergencyAuthority
    )]
    pub authority: Signer<'info>,

    #[account(
        constraint = recovery_validator.key() == oracle_config.recovery_validator @ OracleError::InvalidRecoveryValidator
    )]
    pub recovery_validator: Signer<'info>,

    pub system_program: Program<'info, System>,
    pub clock: Sysvar<'info, Clock>,
}

/// Emergency price update data structure
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct PriceUpdateData {
    pub feed_id: String,
    pub price: u64,
    pub confidence: u64,
    pub timestamp: i64,
    pub reason: EmergencyReason,
    pub validator_signatures: Vec<ValidatorSignature>,
}

/// Emergency batch update data
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct EmergencyBatchUpdate {
    pub feed_id: String,
    pub price: u64,
    pub confidence: u64,
    pub emergency_level: EmergencyLevel,
}

/// Emergency reason enumeration
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum EmergencyReason {
    ExtremeVolatility,
    FeedFailure,
    SecurityBreach,
    MarketHalt,
    GovernanceDecision,
    TechnicalIssue,
    ExternalOracle,
    FlashCrash,
}

/// Emergency severity levels
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum EmergencyLevel {
    Low,      // 1 validator required
    Medium,   // 2 validators required
    High,     // 3 validators required
    Critical, // All validators required
}

/// Validator signature for emergency updates
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct ValidatorSignature {
    pub validator: Pubkey,
    pub signature: [u8; 64],
    pub timestamp: i64,
}

/// Emergency update implementation
pub fn emergency_update_price(
    ctx: Context<EmergencyUpdate>,
    price_data: Vec<PriceUpdateData>,
) -> Result<()> {
    let oracle_config = &mut ctx.accounts.oracle_config;
    let clock = &ctx.accounts.clock;
    
    // Validate emergency conditions
    require!(
        oracle_config.is_emergency_enabled,
        OracleError::EmergencyModeDisabled
    );
    
    require!(
        clock.unix_timestamp <= oracle_config.emergency_expiry,
        OracleError::EmergencyModeExpired
    );

    // Rate limiting check
    let time_since_last = clock.unix_timestamp - oracle_config.last_emergency_update;
    require!(
        time_since_last >= EMERGENCY_UPDATE_COOLDOWN,
        OracleError::EmergencyUpdateTooFrequent
    );

    // Validate batch size
    require!(
        price_data.len() <= MAX_EMERGENCY_BATCH_SIZE,
        OracleError::BatchSizeTooLarge
    );

    let mut updates_processed = 0u16;
    let mut total_value_updated = 0u64;

    for update_data in price_data.iter() {
        // Validate individual update
        validate_emergency_update(update_data, clock.unix_timestamp)?;
        
        // Validate signatures based on emergency reason
        validate_emergency_signatures(update_data, oracle_config)?;
        
        // Calculate price deviation
        let price_feed_key = derive_price_feed_key(&update_data.feed_id)?;
        
        // Update the price feed (this would require additional account context in practice)
        // For this implementation, we'll log the update
        emit!(EmergencyPriceUpdated {
            feed_id: update_data.feed_id.clone(),
            old_price: 0, // Would get from actual feed
            new_price: update_data.price,
            confidence: update_data.confidence,
            reason: update_data.reason.clone(),
            timestamp: clock.unix_timestamp,
            authority: ctx.accounts.authority.key(),
        });

        updates_processed += 1;
        total_value_updated = total_value_updated.saturating_add(update_data.price);
    }

    // Update oracle config
    oracle_config.last_emergency_update = clock.unix_timestamp;
    oracle_config.emergency_update_count = oracle_config.emergency_update_count.saturating_add(1);
    oracle_config.total_emergency_updates = oracle_config.total_emergency_updates.saturating_add(updates_processed);

    // Check if emergency mode should be disabled
    if oracle_config.emergency_update_count >= MAX_EMERGENCY_UPDATES_PER_PERIOD {
        oracle_config.is_emergency_enabled = false;
        
        emit!(EmergencyModeDisabled {
            reason: "Maximum emergency updates reached".to_string(),
            timestamp: clock.unix_timestamp,
            authority: ctx.accounts.authority.key(),
        });
    }

    emit!(EmergencyUpdateCompleted {
        updates_processed,
        total_value_updated,
        timestamp: clock.unix_timestamp,
        authority: ctx.accounts.authority.key(),
    });

    Ok(())
}

/// Circuit breaker activation
pub fn activate_circuit_breaker(
    ctx: Context<EmergencyCircuitBreaker>,
    feed_id: String,
    reason: EmergencyReason,
    duration_seconds: i64,
) -> Result<()> {
    let oracle_config = &mut ctx.accounts.oracle_config;
    let price_feed = &mut ctx.accounts.price_feed;
    let clock = &ctx.accounts.clock;

    // Validate circuit breaker conditions
    require!(
        duration_seconds <= MAX_CIRCUIT_BREAKER_DURATION,
        OracleError::CircuitBreakerDurationTooLong
    );

    // Activate circuit breaker
    price_feed.is_circuit_breaker_active = true;
    price_feed.circuit_breaker_start = clock.unix_timestamp;
    price_feed.circuit_breaker_end = clock.unix_timestamp + duration_seconds;
    price_feed.circuit_breaker_reason = reason.clone();

    // Update global stats
    oracle_config.active_circuit_breakers = oracle_config.active_circuit_breakers.saturating_add(1);

    emit!(CircuitBreakerActivated {
        feed_id,
        reason,
        start_time: price_feed.circuit_breaker_start,
        end_time: price_feed.circuit_breaker_end,
        authority: ctx.accounts.authority.key(),
    });

    Ok(())
}

/// Batch emergency update
pub fn batch_emergency_update(
    ctx: Context<BatchEmergencyUpdate>,
    updates: Vec<EmergencyBatchUpdate>,
) -> Result<()> {
    let oracle_config = &mut ctx.accounts.oracle_config;
    let clock = &ctx.accounts.clock;

    // Validate batch emergency conditions
    require!(
        oracle_config.is_emergency_enabled,
        OracleError::EmergencyModeDisabled
    );

    require!(
        updates.len() <= MAX_BATCH_EMERGENCY_SIZE,
        OracleError::BatchSizeTooLarge
    );

    let mut processed_count = 0u16;
    let mut high_priority_updates = 0u16;

    for update in updates.iter() {
        // Validate emergency level requirements
        validate_emergency_level_requirements(&update.emergency_level, oracle_config)?;
        
        // Apply emergency update logic
        match update.emergency_level {
            EmergencyLevel::Critical => {
                // Requires all validators and immediate processing
                require!(
                    oracle_config.validator_count >= MIN_VALIDATORS_FOR_CRITICAL,
                    OracleError::InsufficientValidators
                );
                high_priority_updates += 1;
            },
            EmergencyLevel::High => {
                // Requires majority validators
                require!(
                    oracle_config.active_validators >= (oracle_config.validator_count * 2 / 3),
                    OracleError::InsufficientValidators
                );
            },
            EmergencyLevel::Medium => {
                // Requires minimum validators
                require!(
                    oracle_config.active_validators >= MIN_VALIDATORS_FOR_MEDIUM,
                    OracleError::InsufficientValidators
                );
            },
            EmergencyLevel::Low => {
                // Single validator sufficient
            }
        }

        emit!(BatchEmergencyUpdateProcessed {
            feed_id: update.feed_id.clone(),
            price: update.price,
            confidence: update.confidence,
            emergency_level: update.emergency_level.clone(),
            timestamp: clock.unix_timestamp,
        });

        processed_count += 1;
    }

    // Update oracle configuration
    oracle_config.last_batch_emergency_update = clock.unix_timestamp;
    oracle_config.total_batch_updates = oracle_config.total_batch_updates.saturating_add(1);

    emit!(BatchEmergencyCompleted {
        total_updates: processed_count,
        high_priority_updates,
        timestamp: clock.unix_timestamp,
        authority: ctx.accounts.authority.key(),
    });

    Ok(())
}

/// Activate recovery mode
pub fn activate_recovery_mode(
    ctx: Context<ActivateRecoveryMode>,
    recovery_duration: i64,
    recovery_reason: String,
) -> Result<()> {
    let oracle_config = &mut ctx.accounts.oracle_config;
    let clock = &ctx.accounts.clock;

    // Validate recovery mode conditions
    require!(
        !oracle_config.is_recovery_mode,
        OracleError::RecoveryModeAlreadyActive
    );

    require!(
        recovery_duration <= MAX_RECOVERY_MODE_DURATION,
        OracleError::RecoveryDurationTooLong
    );

    // Activate recovery mode
    oracle_config.is_recovery_mode = true;
    oracle_config.recovery_start_time = clock.unix_timestamp;
    oracle_config.recovery_end_time = clock.unix_timestamp + recovery_duration;
    oracle_config.recovery_reason = recovery_reason.clone();

    // Disable all non-essential operations
    oracle_config.is_emergency_enabled = false;
    oracle_config.allow_price_updates = false;

    emit!(RecoveryModeActivated {
        reason: recovery_reason,
        start_time: oracle_config.recovery_start_time,
        end_time: oracle_config.recovery_end_time,
        authority: ctx.accounts.authority.key(),
    });

    Ok(())
}

/// Deactivate circuit breaker
pub fn deactivate_circuit_breaker(
    ctx: Context<EmergencyCircuitBreaker>,
    feed_id: String,
) -> Result<()> {
    let oracle_config = &mut ctx.accounts.oracle_config;
    let price_feed = &mut ctx.accounts.price_feed;
    let clock = &ctx.accounts.clock;

    // Validate circuit breaker is active
    require!(
        price_feed.is_circuit_breaker_active,
        OracleError::CircuitBreakerNotActive
    );

    // Check if minimum duration has passed (if not emergency override)
    if clock.unix_timestamp < price_feed.circuit_breaker_end {
        require!(
            oracle_config.emergency_authority == ctx.accounts.authority.key(),
            OracleError::CircuitBreakerStillActive
        );
    }

    // Deactivate circuit breaker
    price_feed.is_circuit_breaker_active = false;
    price_feed.circuit_breaker_start = 0;
    price_feed.circuit_breaker_end = 0;

    // Update global stats
    oracle_config.active_circuit_breakers = oracle_config.active_circuit_breakers.saturating_sub(1);

    emit!(CircuitBreakerDeactivated {
        feed_id,
        deactivated_at: clock.unix_timestamp,
        authority: ctx.accounts.authority.key(),
    });

    Ok(())
}

/// Helper function to validate emergency update
fn validate_emergency_update(
    update_data: &PriceUpdateData,
    current_timestamp: i64,
) -> Result<()> {
    // Validate timestamp freshness
    require!(
        update_data.timestamp >= current_timestamp - MAX_PRICE_STALENESS,
        OracleError::PriceDataTooOld
    );

    require!(
        update_data.timestamp <= current_timestamp + MAX_FUTURE_TIMESTAMP_TOLERANCE,
        OracleError::PriceDataFromFuture
    );

    // Validate price bounds
    require!(
        update_data.price > 0,
        OracleError::InvalidPrice
    );

    require!(
        update_data.confidence <= MAX_CONFIDENCE_VALUE,
        OracleError::InvalidConfidence
    );

    // Validate feed ID
    require!(
        !update_data.feed_id.is_empty() && update_data.feed_id.len() <= MAX_FEED_ID_LENGTH,
        OracleError::InvalidFeedId
    );

    Ok(())
}

/// Validate emergency signatures
fn validate_emergency_signatures(
    update_data: &PriceUpdateData,
    oracle_config: &OracleConfig,
) -> Result<()> {
    let required_signatures = match update_data.reason {
        EmergencyReason::SecurityBreach | EmergencyReason::FlashCrash => 3,
        EmergencyReason::ExtremeVolatility | EmergencyReason::FeedFailure => 2,
        _ => 1,
    };

    require!(
        update_data.validator_signatures.len() >= required_signatures,
        OracleError::InsufficientSignatures
    );

    // Validate each signature (simplified - in practice would verify cryptographic signatures)
    for signature in &update_data.validator_signatures {
        require!(
            oracle_config.authorized_validators.contains(&signature.validator),
            OracleError::UnauthorizedValidator
        );
    }

    Ok(())
}

/// Validate emergency level requirements
fn validate_emergency_level_requirements(
    emergency_level: &EmergencyLevel,
    oracle_config: &OracleConfig,
) -> Result<()> {
    match emergency_level {
        EmergencyLevel::Critical => {
            require!(
                oracle_config.active_validators >= oracle_config.validator_count,
                OracleError::InsufficientValidators
            );
        },
        EmergencyLevel::High => {
            require!(
                oracle_config.active_validators >= (oracle_config.validator_count * 3 / 4),
                OracleError::InsufficientValidators
            );
        },
        EmergencyLevel::Medium => {
            require!(
                oracle_config.active_validators >= (oracle_config.validator_count / 2),
                OracleError::InsufficientValidators
            );
        },
        EmergencyLevel::Low => {
            // No special requirements
        }
    }

    Ok(())
}

/// Derive price feed key
fn derive_price_feed_key(feed_id: &str) -> Result<Pubkey> {
    let (key, _bump) = Pubkey::find_program_address(
        &[PRICE_FEED_SEED, feed_id.as_bytes()],
        &crate::ID,
    );
    Ok(key)
}

/// Emergency update events
#[event]
pub struct EmergencyPriceUpdated {
    pub feed_id: String,
    pub old_price: u64,
    pub new_price: u64,
    pub confidence: u64,
    pub reason: EmergencyReason,
    pub timestamp: i64,
    pub authority: Pubkey,
}

#[event]
pub struct EmergencyModeDisabled {
    pub reason: String,
    pub timestamp: i64,
    pub authority: Pubkey,
}

#[event]
pub struct EmergencyUpdateCompleted {
    pub updates_processed: u16,
    pub total_value_updated: u64,
    pub timestamp: i64,
    pub authority: Pubkey,
}

#[event]
pub struct CircuitBreakerActivated {
    pub feed_id: String,
    pub reason: EmergencyReason,
    pub start_time: i64,
    pub end_time: i64,
    pub authority: Pubkey,
}

#[event]
pub struct CircuitBreakerDeactivated {
    pub feed_id: String,
    pub deactivated_at: i64,
    pub authority: Pubkey,
}

#[event]
pub struct BatchEmergencyUpdateProcessed {
    pub feed_id: String,
    pub price: u64,
    pub confidence: u64,
    pub emergency_level: EmergencyLevel,
    pub timestamp: i64,
}

#[event]
pub struct BatchEmergencyCompleted {
    pub total_updates: u16,
    pub high_priority_updates: u16,
    pub timestamp: i64,
    pub authority: Pubkey,
}

#[event]
pub struct RecoveryModeActivated {
    pub reason: String,
    pub start_time: i64,
    pub end_time: i64,
    pub authority: Pubkey,
}

// Constants for emergency operations
const EMERGENCY_UPDATE_COOLDOWN: i64 = 300; // 5 minutes
const MAX_EMERGENCY_BATCH_SIZE: usize = 50;
const MAX_EMERGENCY_UPDATES_PER_PERIOD: u32 = 10;
const MAX_CIRCUIT_BREAKER_DURATION: i64 = 86400; // 24 hours
const MAX_BATCH_EMERGENCY_SIZE: usize = 100;
const MIN_VALIDATORS_FOR_CRITICAL: u16 = 5;
const MIN_VALIDATORS_FOR_MEDIUM: u16 = 2;
const MAX_RECOVERY_MODE_DURATION: i64 = 604800; // 7 days
const MAX_PRICE_STALENESS: i64 = 3600; // 1 hour
const MAX_FUTURE_TIMESTAMP_TOLERANCE: i64 = 300; // 5 minutes
const MAX_CONFIDENCE_VALUE: u64 = 1000000; // 1M basis points
const MAX_FEED_ID_LENGTH: usize = 32;
const PRICE_FEED_SEED: &[u8] = b"price_feed";
