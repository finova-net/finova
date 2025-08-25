// programs/finova-bridge/src/instructions/initialize_bridge.rs

use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};

use crate::constants::*;
use crate::errors::FinovaBridgeError;
use crate::state::{BridgeConfig, ValidatorSet};

#[derive(Accounts)]
#[instruction(bridge_id: u64)]
pub struct InitializeBridge<'info> {
    #[account(
        init,
        payer = admin,
        space = BridgeConfig::SPACE,
        seeds = [BRIDGE_CONFIG_SEED, bridge_id.to_le_bytes().as_ref()],
        bump
    )]
    pub bridge_config: Account<'info, BridgeConfig>,

    #[account(
        init,
        payer = admin,
        space = ValidatorSet::SPACE,
        seeds = [VALIDATOR_SET_SEED, bridge_id.to_le_bytes().as_ref()],
        bump
    )]
    pub validator_set: Account<'info, ValidatorSet>,

    #[account(
        init,
        payer = admin,
        token::mint = source_mint,
        token::authority = bridge_config,
        seeds = [BRIDGE_VAULT_SEED, bridge_id.to_le_bytes().as_ref()],
        bump
    )]
    pub bridge_vault: Account<'info, TokenAccount>,

    pub source_mint: Account<'info, Mint>,

    #[account(
        mut,
        constraint = admin.key() != Pubkey::default() @ FinovaBridgeError::InvalidAdmin
    )]
    pub admin: Signer<'info>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct InitializeBridgeParams {
    pub bridge_id: u64,
    pub target_chain_id: u64,
    pub min_confirmations: u32,
    pub fee_rate: u64, // In basis points (100 = 1%)
    pub daily_limit: u64,
    pub per_transaction_limit: u64,
    pub validator_threshold: u8,
    pub initial_validators: Vec<Pubkey>,
    pub emergency_pause_authority: Pubkey,
    pub fee_collector: Pubkey,
}

pub fn initialize_bridge(
    ctx: Context<InitializeBridge>,
    params: InitializeBridgeParams,
) -> Result<()> {
    let bridge_config = &mut ctx.accounts.bridge_config;
    let validator_set = &mut ctx.accounts.validator_set;
    let current_time = Clock::get()?.unix_timestamp;

    // Validate initialization parameters
    require!(
        params.bridge_id > 0,
        FinovaBridgeError::InvalidBridgeId
    );

    require!(
        params.target_chain_id > 0 && params.target_chain_id != params.bridge_id,
        FinovaBridgeError::InvalidTargetChain
    );

    require!(
        params.min_confirmations >= MIN_CONFIRMATIONS && 
        params.min_confirmations <= MAX_CONFIRMATIONS,
        FinovaBridgeError::InvalidConfirmations
    );

    require!(
        params.fee_rate <= MAX_FEE_RATE,
        FinovaBridgeError::ExcessiveFeeRate
    );

    require!(
        params.daily_limit > 0 && params.per_transaction_limit > 0,
        FinovaBridgeError::InvalidLimits
    );

    require!(
        params.per_transaction_limit <= params.daily_limit,
        FinovaBridgeError::TransactionLimitExceedsDailyLimit
    );

    require!(
        params.validator_threshold >= MIN_VALIDATOR_THRESHOLD &&
        params.validator_threshold <= MAX_VALIDATOR_THRESHOLD,
        FinovaBridgeError::InvalidValidatorThreshold
    );

    require!(
        !params.initial_validators.is_empty() &&
        params.initial_validators.len() <= MAX_VALIDATORS as usize,
        FinovaBridgeError::InvalidValidatorCount
    );

    require!(
        params.initial_validators.len() >= params.validator_threshold as usize,
        FinovaBridgeError::InsufficientValidators
    );

    require!(
        params.emergency_pause_authority != Pubkey::default(),
        FinovaBridgeError::InvalidEmergencyAuthority
    );

    require!(
        params.fee_collector != Pubkey::default(),
        FinovaBridgeError::InvalidFeeCollector
    );

    // Check for duplicate validators
    let mut unique_validators = params.initial_validators.clone();
    unique_validators.sort();
    unique_validators.dedup();
    require!(
        unique_validators.len() == params.initial_validators.len(),
        FinovaBridgeError::DuplicateValidators
    );

    // Initialize bridge configuration
    bridge_config.bridge_id = params.bridge_id;
    bridge_config.admin = ctx.accounts.admin.key();
    bridge_config.source_mint = ctx.accounts.source_mint.key();
    bridge_config.target_chain_id = params.target_chain_id;
    bridge_config.bridge_vault = ctx.accounts.bridge_vault.key();
    bridge_config.validator_set = ctx.accounts.validator_set.key();
    
    bridge_config.min_confirmations = params.min_confirmations;
    bridge_config.fee_rate = params.fee_rate;
    bridge_config.daily_limit = params.daily_limit;
    bridge_config.per_transaction_limit = params.per_transaction_limit;
    bridge_config.daily_volume = 0;
    bridge_config.total_volume = 0;
    bridge_config.last_reset_timestamp = current_time;
    
    bridge_config.is_paused = false;
    bridge_config.emergency_pause_authority = params.emergency_pause_authority;
    bridge_config.fee_collector = params.fee_collector;
    
    bridge_config.created_at = current_time;
    bridge_config.updated_at = current_time;
    bridge_config.nonce = 0;
    bridge_config.bump = ctx.bumps.bridge_config;

    // Initialize validator set
    validator_set.bridge_id = params.bridge_id;
    validator_set.threshold = params.validator_threshold;
    validator_set.validator_count = params.initial_validators.len() as u8;
    
    // Initialize validators array with default values
    validator_set.validators = [Pubkey::default(); MAX_VALIDATORS as usize];
    validator_set.validator_powers = [0u64; MAX_VALIDATORS as usize];
    validator_set.is_active = [false; MAX_VALIDATORS as usize];
    
    // Set initial validators
    for (i, validator) in params.initial_validators.iter().enumerate() {
        validator_set.validators[i] = *validator;
        validator_set.validator_powers[i] = DEFAULT_VALIDATOR_POWER;
        validator_set.is_active[i] = true;
    }
    
    validator_set.total_power = (params.initial_validators.len() as u64) * DEFAULT_VALIDATOR_POWER;
    validator_set.created_at = current_time;
    validator_set.updated_at = current_time;
    validator_set.bump = ctx.bumps.validator_set;

    // Emit initialization event
    emit!(BridgeInitialized {
        bridge_id: params.bridge_id,
        admin: ctx.accounts.admin.key(),
        source_mint: ctx.accounts.source_mint.key(),
        target_chain_id: params.target_chain_id,
        validator_count: params.initial_validators.len() as u8,
        validator_threshold: params.validator_threshold,
        min_confirmations: params.min_confirmations,
        fee_rate: params.fee_rate,
        daily_limit: params.daily_limit,
        per_transaction_limit: params.per_transaction_limit,
        timestamp: current_time,
    });

    msg!(
        "Bridge initialized successfully. Bridge ID: {}, Target Chain: {}, Validators: {}",
        params.bridge_id,
        params.target_chain_id,
        params.initial_validators.len()
    );

    Ok(())
}

// Update bridge configuration (admin only)
#[derive(Accounts)]
pub struct UpdateBridgeConfig<'info> {
    #[account(
        mut,
        seeds = [BRIDGE_CONFIG_SEED, bridge_config.bridge_id.to_le_bytes().as_ref()],
        bump = bridge_config.bump,
        constraint = bridge_config.admin == admin.key() @ FinovaBridgeError::UnauthorizedAdmin
    )]
    pub bridge_config: Account<'info, BridgeConfig>,

    pub admin: Signer<'info>,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct UpdateBridgeConfigParams {
    pub min_confirmations: Option<u32>,
    pub fee_rate: Option<u64>,
    pub daily_limit: Option<u64>,
    pub per_transaction_limit: Option<u64>,
    pub emergency_pause_authority: Option<Pubkey>,
    pub fee_collector: Option<Pubkey>,
}

pub fn update_bridge_config(
    ctx: Context<UpdateBridgeConfig>,
    params: UpdateBridgeConfigParams,
) -> Result<()> {
    let bridge_config = &mut ctx.accounts.bridge_config;
    let current_time = Clock::get()?.unix_timestamp;

    // Update min confirmations if provided
    if let Some(min_confirmations) = params.min_confirmations {
        require!(
            min_confirmations >= MIN_CONFIRMATIONS && 
            min_confirmations <= MAX_CONFIRMATIONS,
            FinovaBridgeError::InvalidConfirmations
        );
        bridge_config.min_confirmations = min_confirmations;
    }

    // Update fee rate if provided
    if let Some(fee_rate) = params.fee_rate {
        require!(
            fee_rate <= MAX_FEE_RATE,
            FinovaBridgeError::ExcessiveFeeRate
        );
        bridge_config.fee_rate = fee_rate;
    }

    // Update daily limit if provided
    if let Some(daily_limit) = params.daily_limit {
        require!(
            daily_limit > 0,
            FinovaBridgeError::InvalidLimits
        );
        
        // Ensure per transaction limit doesn't exceed new daily limit
        require!(
            bridge_config.per_transaction_limit <= daily_limit,
            FinovaBridgeError::TransactionLimitExceedsDailyLimit
        );
        
        bridge_config.daily_limit = daily_limit;
    }

    // Update per transaction limit if provided
    if let Some(per_transaction_limit) = params.per_transaction_limit {
        require!(
            per_transaction_limit > 0 && per_transaction_limit <= bridge_config.daily_limit,
            FinovaBridgeError::InvalidLimits
        );
        bridge_config.per_transaction_limit = per_transaction_limit;
    }

    // Update emergency pause authority if provided
    if let Some(emergency_pause_authority) = params.emergency_pause_authority {
        require!(
            emergency_pause_authority != Pubkey::default(),
            FinovaBridgeError::InvalidEmergencyAuthority
        );
        bridge_config.emergency_pause_authority = emergency_pause_authority;
    }

    // Update fee collector if provided
    if let Some(fee_collector) = params.fee_collector {
        require!(
            fee_collector != Pubkey::default(),
            FinovaBridgeError::InvalidFeeCollector
        );
        bridge_config.fee_collector = fee_collector;
    }

    bridge_config.updated_at = current_time;

    emit!(BridgeConfigUpdated {
        bridge_id: bridge_config.bridge_id,
        admin: ctx.accounts.admin.key(),
        timestamp: current_time,
    });

    msg!("Bridge configuration updated successfully");

    Ok(())
}

// Transfer bridge admin (current admin only)
#[derive(Accounts)]
pub struct TransferBridgeAdmin<'info> {
    #[account(
        mut,
        seeds = [BRIDGE_CONFIG_SEED, bridge_config.bridge_id.to_le_bytes().as_ref()],
        bump = bridge_config.bump,
        constraint = bridge_config.admin == current_admin.key() @ FinovaBridgeError::UnauthorizedAdmin
    )]
    pub bridge_config: Account<'info, BridgeConfig>,

    pub current_admin: Signer<'info>,
    
    /// CHECK: New admin pubkey is validated in the instruction
    pub new_admin: UncheckedAccount<'info>,
}

pub fn transfer_bridge_admin(
    ctx: Context<TransferBridgeAdmin>,
) -> Result<()> {
    let bridge_config = &mut ctx.accounts.bridge_config;
    let current_time = Clock::get()?.unix_timestamp;

    require!(
        ctx.accounts.new_admin.key() != Pubkey::default(),
        FinovaBridgeError::InvalidAdmin
    );

    require!(
        ctx.accounts.new_admin.key() != bridge_config.admin,
        FinovaBridgeError::SameAdmin
    );

    let old_admin = bridge_config.admin;
    bridge_config.admin = ctx.accounts.new_admin.key();
    bridge_config.updated_at = current_time;

    emit!(BridgeAdminTransferred {
        bridge_id: bridge_config.bridge_id,
        old_admin,
        new_admin: bridge_config.admin,
        timestamp: current_time,
    });

    msg!(
        "Bridge admin transferred from {} to {}",
        old_admin,
        bridge_config.admin
    );

    Ok(())
}

// Reset daily volume (automated or admin)
#[derive(Accounts)]
pub struct ResetDailyVolume<'info> {
    #[account(
        mut,
        seeds = [BRIDGE_CONFIG_SEED, bridge_config.bridge_id.to_le_bytes().as_ref()],
        bump = bridge_config.bump
    )]
    pub bridge_config: Account<'info, BridgeConfig>,
}

pub fn reset_daily_volume(
    ctx: Context<ResetDailyVolume>,
) -> Result<()> {
    let bridge_config = &mut ctx.accounts.bridge_config;
    let current_time = Clock::get()?.unix_timestamp;

    // Check if 24 hours have passed since last reset
    let time_since_reset = current_time - bridge_config.last_reset_timestamp;
    require!(
        time_since_reset >= SECONDS_PER_DAY,
        FinovaBridgeError::ResetTooEarly
    );

    bridge_config.daily_volume = 0;
    bridge_config.last_reset_timestamp = current_time;
    bridge_config.updated_at = current_time;

    emit!(DailyVolumeReset {
        bridge_id: bridge_config.bridge_id,
        timestamp: current_time,
    });

    msg!("Daily volume reset successfully");

    Ok(())
}

// Events
#[event]
pub struct BridgeInitialized {
    pub bridge_id: u64,
    pub admin: Pubkey,
    pub source_mint: Pubkey,
    pub target_chain_id: u64,
    pub validator_count: u8,
    pub validator_threshold: u8,
    pub min_confirmations: u32,
    pub fee_rate: u64,
    pub daily_limit: u64,
    pub per_transaction_limit: u64,
    pub timestamp: i64,
}

#[event]
pub struct BridgeConfigUpdated {
    pub bridge_id: u64,
    pub admin: Pubkey,
    pub timestamp: i64,
}

#[event]
pub struct BridgeAdminTransferred {
    pub bridge_id: u64,
    pub old_admin: Pubkey,
    pub new_admin: Pubkey,
    pub timestamp: i64,
}

#[event]
pub struct DailyVolumeReset {
    pub bridge_id: u64,
    pub timestamp: i64,
}

// Helper functions for validation
impl InitializeBridgeParams {
    pub fn validate(&self) -> Result<()> {
        // Additional validation logic can be added here
        require!(
            self.initial_validators.len() >= self.validator_threshold as usize,
            FinovaBridgeError::InsufficientValidators
        );

        // Validate validator addresses
        for validator in &self.initial_validators {
            require!(
                *validator != Pubkey::default(),
                FinovaBridgeError::InvalidValidator
            );
        }

        Ok(())
    }
}

// Constants for validation
const MIN_CONFIRMATIONS: u32 = 1;
const MAX_CONFIRMATIONS: u32 = 100;
const MAX_FEE_RATE: u64 = 1000; // 10% in basis points
const MIN_VALIDATOR_THRESHOLD: u8 = 1;
const MAX_VALIDATOR_THRESHOLD: u8 = 100;
const MAX_VALIDATORS: u8 = 21;
const DEFAULT_VALIDATOR_POWER: u64 = 1;
const SECONDS_PER_DAY: i64 = 86400;
