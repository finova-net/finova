// programs/finova-core/src/instructions/initialize.rs

use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};

use crate::constants::*;
use crate::errors::FinovaError;
use crate::state::{
    user::User,
    mining::MiningState,
    network::NetworkState,
    rewards::RewardPool,
};

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
        init,
        payer = authority,
        space = 8 + NetworkState::SPACE,
        seeds = [NETWORK_STATE_SEED],
        bump
    )]
    pub network_state: Account<'info, NetworkState>,

    #[account(
        init,
        payer = authority,
        space = 8 + MiningState::SPACE,
        seeds = [MINING_STATE_SEED],
        bump
    )]
    pub mining_state: Account<'info, MiningState>,

    #[account(
        init,
        payer = authority,
        space = 8 + RewardPool::SPACE,
        seeds = [REWARD_POOL_SEED],
        bump
    )]
    pub reward_pool: Account<'info, RewardPool>,

    #[account(mut)]
    pub authority: Signer<'info>,

    pub fin_mint: Account<'info, Mint>,
    pub sfin_mint: Account<'info, Mint>,
    pub usdfin_mint: Account<'info, Mint>,
    pub susdfin_mint: Account<'info, Mint>,

    #[account(
        init,
        payer = authority,
        token::mint = fin_mint,
        token::authority = network_state,
        seeds = [TREASURY_SEED],
        bump
    )]
    pub treasury: Account<'info, TokenAccount>,

    #[account(
        init,
        payer = authority,
        token::mint = fin_mint,
        token::authority = network_state,
        seeds = [REWARD_VAULT_SEED],
        bump
    )]
    pub reward_vault: Account<'info, TokenAccount>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct InitializeUser<'info> {
    #[account(
        init,
        payer = user,
        space = 8 + User::SPACE,
        seeds = [USER_SEED, user.key().as_ref()],
        bump
    )]
    pub user_account: Account<'info, User>,

    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        mut,
        seeds = [NETWORK_STATE_SEED],
        bump = network_state.bump
    )]
    pub network_state: Account<'info, NetworkState>,

    #[account(
        init,
        payer = user,
        token::mint = fin_mint,
        token::authority = user_account,
        seeds = [USER_TOKEN_ACCOUNT_SEED, user.key().as_ref()],
        bump
    )]
    pub user_token_account: Account<'info, TokenAccount>,

    pub fin_mint: Account<'info, Mint>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct UpdateNetworkConfig<'info> {
    #[account(
        mut,
        seeds = [NETWORK_STATE_SEED],
        bump = network_state.bump,
        has_one = authority @ FinovaError::Unauthorized
    )]
    pub network_state: Account<'info, NetworkState>,

    pub authority: Signer<'info>,
}

pub fn initialize(
    ctx: Context<Initialize>,
    config: NetworkConfig,
) -> Result<()> {
    let network_state = &mut ctx.accounts.network_state;
    let mining_state = &mut ctx.accounts.mining_state;
    let reward_pool = &mut ctx.accounts.reward_pool;

    // Initialize Network State
    network_state.initialize(
        ctx.accounts.authority.key(),
        *ctx.bumps.get("network_state").unwrap(),
        config,
        ctx.accounts.treasury.key(),
        ctx.accounts.reward_vault.key(),
    )?;

    // Initialize Mining State
    mining_state.initialize(
        *ctx.bumps.get("mining_state").unwrap(),
        network_state.config.clone(),
    )?;

    // Initialize Reward Pool
    reward_pool.initialize(
        *ctx.bumps.get("reward_pool").unwrap(),
        ctx.accounts.fin_mint.key(),
        ctx.accounts.sfin_mint.key(),
        ctx.accounts.usdfin_mint.key(),
        ctx.accounts.susdfin_mint.key(),
    )?;

    msg!("Finova Network initialized successfully");
    msg!("Network State: {}", network_state.key());
    msg!("Mining State: {}", mining_state.key());
    msg!("Reward Pool: {}", reward_pool.key());

    Ok(())
}

pub fn initialize_user(
    ctx: Context<InitializeUser>,
    referrer: Option<Pubkey>,
    kyc_data: Option<KycData>,
) -> Result<()> {
    let user_account = &mut ctx.accounts.user_account;
    let network_state = &mut ctx.accounts.network_state;
    let clock = Clock::get()?;

    // Validate referrer if provided
    if let Some(referrer_key) = referrer {
        require!(
            referrer_key != ctx.accounts.user.key(),
            FinovaError::SelfReferral
        );
        
        // Verify referrer exists and is active
        // This would typically require another account validation
        // For now, we'll assume the referrer validation is done off-chain
    }

    // Initialize user account
    user_account.initialize(
        ctx.accounts.user.key(),
        *ctx.bumps.get("user_account").unwrap(),
        ctx.accounts.user_token_account.key(),
        referrer,
        kyc_data,
        clock.unix_timestamp,
    )?;

    // Update network statistics
    network_state.total_users = network_state.total_users
        .checked_add(1)
        .ok_or(FinovaError::MathOverflow)?;

    // Determine current mining phase based on user count
    let current_phase = determine_mining_phase(network_state.total_users);
    if current_phase != network_state.current_mining_phase {
        network_state.current_mining_phase = current_phase;
        network_state.phase_transition_timestamp = clock.unix_timestamp;
        
        msg!("Mining phase transitioned to: {:?}", current_phase);
    }

    // Award early adopter bonuses
    if network_state.total_users <= FINIZEN_USER_LIMIT {
        user_account.is_finizen = true;
        user_account.finizen_bonus_multiplier = calculate_finizen_bonus(network_state.total_users);
        
        msg!("Finizen user registered! Bonus: {}x", user_account.finizen_bonus_multiplier);
    }

    msg!("User initialized: {}", ctx.accounts.user.key());
    msg!("Total users: {}", network_state.total_users);
    msg!("Current mining phase: {:?}", network_state.current_mining_phase);

    Ok(())
}

pub fn update_network_config(
    ctx: Context<UpdateNetworkConfig>,
    new_config: NetworkConfig,
) -> Result<()> {
    let network_state = &mut ctx.accounts.network_state;
    let clock = Clock::get()?;

    // Validate configuration parameters
    require!(
        new_config.base_mining_rate > 0 && new_config.base_mining_rate <= MAX_MINING_RATE,
        FinovaError::InvalidMiningRate
    );

    require!(
        new_config.max_referral_depth <= MAX_REFERRAL_DEPTH,
        FinovaError::InvalidReferralDepth
    );

    require!(
        new_config.quality_threshold >= MIN_QUALITY_THRESHOLD && 
        new_config.quality_threshold <= MAX_QUALITY_THRESHOLD,
        FinovaError::InvalidQualityThreshold
    );

    // Store previous configuration for audit trail
    network_state.previous_config = Some(network_state.config.clone());
    network_state.config = new_config;
    network_state.last_config_update = clock.unix_timestamp;

    msg!("Network configuration updated by: {}", ctx.accounts.authority.key());

    Ok(())
}

// Helper functions
fn determine_mining_phase(total_users: u64) -> MiningPhase {
    match total_users {
        0..=PHASE_1_USER_LIMIT => MiningPhase::Finizen,
        PHASE_1_USER_LIMIT_PLUS_ONE..=PHASE_2_USER_LIMIT => MiningPhase::Growth,
        PHASE_2_USER_LIMIT_PLUS_ONE..=PHASE_3_USER_LIMIT => MiningPhase::Maturity,
        _ => MiningPhase::Stability,
    }
}

fn calculate_finizen_bonus(user_number: u64) -> u64 {
    // Earlier users get higher bonuses
    let bonus_base = 200; // 2.0x base
    let decay_rate = user_number / 1000; // Gradual decay
    
    std::cmp::max(100, bonus_base - decay_rate) // Minimum 1.0x, maximum 2.0x
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct NetworkConfig {
    pub base_mining_rate: u64,        // Base mining rate in micro-FIN per hour
    pub max_referral_depth: u8,       // Maximum referral network depth
    pub quality_threshold: u64,       // Minimum quality score (0-100)
    pub anti_bot_threshold: u64,      // Suspicious activity threshold
    pub kyc_bonus_multiplier: u64,    // KYC verification bonus (100 = 1.0x)
    pub max_daily_mining: u64,        // Maximum FIN per day per user
    pub regression_coefficient: u64,  // Exponential regression coefficient
    pub guild_bonus_rate: u64,        // Guild participation bonus
    pub viral_content_threshold: u64, // Views required for viral bonus
    pub staking_reward_rate: u64,     // Annual staking reward rate
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            base_mining_rate: 100_000, // 0.1 FIN per hour in micro-FIN
            max_referral_depth: 10,
            quality_threshold: 50,
            anti_bot_threshold: 80,
            kyc_bonus_multiplier: 120, // 1.2x
            max_daily_mining: 2_400_000, // 2.4 FIN per day in micro-FIN
            regression_coefficient: 1000, // 0.001 in fixed point
            guild_bonus_rate: 130, // 1.3x
            viral_content_threshold: 1000,
            staking_reward_rate: 1500, // 15% APY
        }
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct KycData {
    pub verification_level: KycLevel,
    pub verification_timestamp: i64,
    pub verification_provider: String,
    pub document_hash: [u8; 32],
    pub biometric_hash: Option<[u8; 32]>,
    pub risk_score: u8, // 0-100, lower is better
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum KycLevel {
    None,
    Basic,      // Email + Phone verification
    Standard,   // + Government ID
    Premium,    // + Biometric verification
    Enterprise, // + Enhanced due diligence
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum MiningPhase {
    Finizen,   // 0-100K users
    Growth,    // 100K-1M users  
    Maturity,  // 1M-10M users
    Stability, // 10M+ users
}

// Event definitions
#[event]
pub struct NetworkInitialized {
    pub network_state: Pubkey,
    pub authority: Pubkey,
    pub timestamp: i64,
    pub initial_config: NetworkConfig,
}

#[event]
pub struct UserInitialized {
    pub user: Pubkey,
    pub user_account: Pubkey,
    pub referrer: Option<Pubkey>,
    pub is_finizen: bool,
    pub user_number: u64,
    pub timestamp: i64,
}

#[event]
pub struct MiningPhaseTransition {
    pub previous_phase: MiningPhase,
    pub new_phase: MiningPhase,
    pub total_users: u64,
    pub timestamp: i64,
}

#[event]
pub struct NetworkConfigUpdated {
    pub authority: Pubkey,
    pub previous_config: NetworkConfig,
    pub new_config: NetworkConfig,
    pub timestamp: i64,
}
