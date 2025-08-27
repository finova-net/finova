// programs/finova-token/src/instructions/initialize_mint.rs

use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};
use anchor_spl::associated_token::AssociatedToken;

use crate::constants::*;
use crate::errors::*;
use crate::state::*;
use crate::utils::*;

/// Initialize the $FIN token mint with comprehensive configuration
/// Supports multi-token ecosystem: $FIN, $sFIN, $USDfin, $sUSDfin
#[derive(Accounts)]
#[instruction(token_type: u8)]
pub struct InitializeMint<'info> {
    /// Authority that can initialize mints (protocol admin)
    #[account(
        mut,
        constraint = authority.key() == PROTOCOL_AUTHORITY @ FinovaTokenError::Unauthorized
    )]
    pub authority: Signer<'info>,

    /// Mint account for the token being initialized
    #[account(
        init,
        payer = authority,
        mint::decimals = 9,
        mint::authority = mint_authority,
        seeds = [
            TOKEN_MINT_SEED,
            &[token_type]
        ],
        bump
    )]
    pub mint: Account<'info, Mint>,

    /// Mint authority (PDA controlled by program)
    /// CHECK: This is a PDA and will be validated by seeds
    #[account(
        seeds = [MINT_AUTHORITY_SEED, &[token_type]],
        bump
    )]
    pub mint_authority: UncheckedAccount<'info>,

    /// Mint info account storing configuration and statistics
    #[account(
        init,
        payer = authority,
        space = MintInfo::SPACE,
        seeds = [
            MINT_INFO_SEED,
            mint.key().as_ref()
        ],
        bump
    )]
    pub mint_info: Account<'info, MintInfo>,

    /// Treasury token account for collecting fees and managing supply
    #[account(
        init,
        payer = authority,
        associated_token::mint = mint,
        associated_token::authority = treasury_authority
    )]
    pub treasury_account: Account<'info, TokenAccount>,

    /// Treasury authority (PDA for treasury management)
    /// CHECK: This is a PDA and will be validated by seeds
    #[account(
        seeds = [TREASURY_AUTHORITY_SEED],
        bump
    )]
    pub treasury_authority: UncheckedAccount<'info>,

    /// Reward pool account for distribution rewards
    #[account(
        init,
        payer = authority,
        space = RewardPool::SPACE,
        seeds = [
            REWARD_POOL_SEED,
            mint.key().as_ref()
        ],
        bump
    )]
    pub reward_pool: Account<'info, RewardPool>,

    /// Reward pool token account
    #[account(
        init,
        payer = authority,
        associated_token::mint = mint,
        associated_token::authority = reward_pool_authority
    )]
    pub reward_pool_account: Account<'info, TokenAccount>,

    /// Reward pool authority (PDA)
    /// CHECK: This is a PDA and will be validated by seeds
    #[account(
        seeds = [REWARD_POOL_AUTHORITY_SEED, mint.key().as_ref()],
        bump
    )]
    pub reward_pool_authority: UncheckedAccount<'info>,

    /// System program
    pub system_program: Program<'info, System>,
    /// Token program
    pub token_program: Program<'info, Token>,
    /// Associated token program
    pub associated_token_program: Program<'info, AssociatedToken>,
    /// Rent sysvar
    pub rent: Sysvar<'info, Rent>,
}

/// Token type enumeration
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum TokenType {
    /// $FIN - Primary utility token
    Fin = 0,
    /// $sFIN - Staked $FIN derivative
    StakedFin = 1,
    /// $USDfin - Synthetic stablecoin
    USDfin = 2,
    /// $sUSDfin - Staked $USDfin derivative
    StakedUSDfin = 3,
}

impl TokenType {
    pub fn from_u8(value: u8) -> Result<Self> {
        match value {
            0 => Ok(TokenType::Fin),
            1 => Ok(TokenType::StakedFin),
            2 => Ok(TokenType::USDfin),
            3 => Ok(TokenType::StakedUSDfin),
            _ => Err(FinovaTokenError::InvalidTokenType.into()),
        }
    }

    pub fn get_config(&self) -> TokenConfig {
        match self {
            TokenType::Fin => TokenConfig {
                max_supply: MAX_FIN_SUPPLY,
                initial_mining_rate: INITIAL_MINING_RATE,
                burn_fee_rate: FIN_BURN_FEE_RATE,
                transfer_fee_rate: FIN_TRANSFER_FEE_RATE,
                staking_enabled: true,
                mining_enabled: true,
                governance_weight: 100, // 1.0x voting power
            },
            TokenType::StakedFin => TokenConfig {
                max_supply: MAX_SFIN_SUPPLY,
                initial_mining_rate: 0, // No direct mining
                burn_fee_rate: 0, // No burn fees for staked tokens
                transfer_fee_rate: SFIN_TRANSFER_FEE_RATE,
                staking_enabled: false, // Cannot stake staked tokens
                mining_enabled: false,
                governance_weight: 150, // 1.5x voting power
            },
            TokenType::USDfin => TokenConfig {
                max_supply: MAX_USDFIN_SUPPLY,
                initial_mining_rate: 0, // Synthetic token
                burn_fee_rate: USDFIN_BURN_FEE_RATE,
                transfer_fee_rate: USDFIN_TRANSFER_FEE_RATE,
                staking_enabled: true,
                mining_enabled: false,
                governance_weight: 50, // 0.5x voting power
            },
            TokenType::StakedUSDfin => TokenConfig {
                max_supply: MAX_SUSDFIN_SUPPLY,
                initial_mining_rate: 0,
                burn_fee_rate: 0,
                transfer_fee_rate: SUSDFIN_TRANSFER_FEE_RATE,
                staking_enabled: false,
                mining_enabled: false,
                governance_weight: 75, // 0.75x voting power
            },
        }
    }
}

/// Token configuration structure
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy)]
pub struct TokenConfig {
    pub max_supply: u64,
    pub initial_mining_rate: u64,
    pub burn_fee_rate: u16, // Basis points (100 = 1%)
    pub transfer_fee_rate: u16,
    pub staking_enabled: bool,
    pub mining_enabled: bool,
    pub governance_weight: u16, // Basis points (100 = 1.0x)
}

pub fn handler(
    ctx: Context<InitializeMint>,
    token_type: u8,
    initial_supply: Option<u64>,
    metadata_uri: Option<String>,
) -> Result<()> {
    let clock = Clock::get()?;
    
    // Validate token type
    let token_type_enum = TokenType::from_u8(token_type)?;
    let config = token_type_enum.get_config();

    // Validate initial supply
    if let Some(supply) = initial_supply {
        require!(
            supply <= config.max_supply,
            FinovaTokenError::ExceedsMaxSupply
        );
    }

    // Initialize mint info
    let mint_info = &mut ctx.accounts.mint_info;
    mint_info.mint = ctx.accounts.mint.key();
    mint_info.token_type = token_type;
    mint_info.authority = ctx.accounts.authority.key();
    mint_info.mint_authority = ctx.accounts.mint_authority.key();
    mint_info.treasury_account = ctx.accounts.treasury_account.key();
    mint_info.reward_pool = ctx.accounts.reward_pool.key();
    mint_info.config = config;
    mint_info.total_supply = 0;
    mint_info.circulating_supply = 0;
    mint_info.total_burned = 0;
    mint_info.mining_phase = if config.mining_enabled { 1 } else { 0 };
    mint_info.current_mining_rate = config.initial_mining_rate;
    mint_info.last_mining_update = clock.unix_timestamp;
    mint_info.is_paused = false;
    mint_info.created_at = clock.unix_timestamp;
    mint_info.updated_at = clock.unix_timestamp;
    mint_info.bump = ctx.bumps.mint_info;

    // Set metadata if provided
    if let Some(uri) = metadata_uri {
        require!(
            uri.len() <= MAX_METADATA_URI_LENGTH,
            FinovaTokenError::MetadataUriTooLong
        );
        mint_info.metadata_uri = uri;
    }

    // Initialize reward pool
    let reward_pool = &mut ctx.accounts.reward_pool;
    reward_pool.mint = ctx.accounts.mint.key();
    reward_pool.token_account = ctx.accounts.reward_pool_account.key();
    reward_pool.authority = ctx.accounts.reward_pool_authority.key();
    reward_pool.total_rewards = 0;
    reward_pool.distributed_rewards = 0;
    reward_pool.pending_rewards = 0;
    reward_pool.rewards_per_second = 0;
    reward_pool.last_update_time = clock.unix_timestamp;
    reward_pool.is_active = true;
    reward_pool.created_at = clock.unix_timestamp;
    reward_pool.bump = ctx.bumps.reward_pool;

    // Initialize reward distribution based on token type
    match token_type_enum {
        TokenType::Fin => {
            // $FIN gets mining rewards and staking rewards
            reward_pool.reward_types = vec![
                RewardType::Mining as u8,
                RewardType::Staking as u8,
                RewardType::Referral as u8,
                RewardType::XP as u8,
            ];
            reward_pool.total_allocation = TOTAL_MINING_REWARDS;
        },
        TokenType::StakedFin => {
            // $sFIN gets staking rewards only
            reward_pool.reward_types = vec![
                RewardType::Staking as u8,
            ];
            reward_pool.total_allocation = TOTAL_STAKING_REWARDS;
        },
        TokenType::USDfin => {
            // $USDfin gets stability rewards
            reward_pool.reward_types = vec![
                RewardType::Stability as u8,
            ];
            reward_pool.total_allocation = TOTAL_STABILITY_REWARDS;
        },
        TokenType::StakedUSDfin => {
            // $sUSDfin gets yield rewards
            reward_pool.reward_types = vec![
                RewardType::Yield as u8,
            ];
            reward_pool.total_allocation = TOTAL_YIELD_REWARDS;
        },
    }

    // Mint initial supply if specified
    if let Some(initial_supply) = initial_supply {
        // Mint to treasury account
        let seeds = &[
            MINT_AUTHORITY_SEED,
            &[token_type],
            &[ctx.bumps.mint_authority],
        ];
        let signer = &[&seeds[..]];

        anchor_spl::token::mint_to(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                anchor_spl::token::MintTo {
                    mint: ctx.accounts.mint.to_account_info(),
                    to: ctx.accounts.treasury_account.to_account_info(),
                    authority: ctx.accounts.mint_authority.to_account_info(),
                },
                signer,
            ),
            initial_supply,
        )?;

        // Update mint info
        mint_info.total_supply = initial_supply;
        mint_info.circulating_supply = initial_supply;
        mint_info.updated_at = clock.unix_timestamp;
    }

    // Emit initialization event
    emit!(MintInitializedEvent {
        mint: ctx.accounts.mint.key(),
        token_type,
        authority: ctx.accounts.authority.key(),
        initial_supply: initial_supply.unwrap_or(0),
        max_supply: config.max_supply,
        timestamp: clock.unix_timestamp,
    });

    // Log important information
    msg!(
        "Initialized {} mint: {} with max supply: {} and initial supply: {}",
        get_token_name(token_type_enum),
        ctx.accounts.mint.key(),
        config.max_supply,
        initial_supply.unwrap_or(0)
    );

    Ok(())
}

/// Upgrade mint configuration (admin only)
#[derive(Accounts)]
pub struct UpgradeMintConfig<'info> {
    /// Authority that can upgrade configuration
    #[account(
        mut,
        constraint = authority.key() == PROTOCOL_AUTHORITY @ FinovaTokenError::Unauthorized
    )]
    pub authority: Signer<'info>,

    /// Mint info account to update
    #[account(
        mut,
        has_one = authority @ FinovaTokenError::Unauthorized
    )]
    pub mint_info: Account<'info, MintInfo>,
}

pub fn upgrade_mint_config_handler(
    ctx: Context<UpgradeMintConfig>,
    new_config: TokenConfig,
) -> Result<()> {
    let clock = Clock::get()?;
    let mint_info = &mut ctx.accounts.mint_info;

    // Validate new configuration
    require!(
        new_config.max_supply >= mint_info.total_supply,
        FinovaTokenError::InvalidMaxSupply
    );

    require!(
        new_config.burn_fee_rate <= MAX_FEE_RATE &&
        new_config.transfer_fee_rate <= MAX_FEE_RATE,
        FinovaTokenError::ExcessiveFeeRate
    );

    // Update configuration
    let old_config = mint_info.config;
    mint_info.config = new_config;
    mint_info.updated_at = clock.unix_timestamp;

    // Emit configuration update event
    emit!(MintConfigUpdatedEvent {
        mint: mint_info.mint,
        authority: ctx.accounts.authority.key(),
        old_config,
        new_config,
        timestamp: clock.unix_timestamp,
    });

    msg!(
        "Updated mint configuration for: {}",
        mint_info.mint
    );

    Ok(())
}

/// Pause/unpause mint operations
#[derive(Accounts)]
pub struct PauseMint<'info> {
    /// Authority that can pause mints
    #[account(
        mut,
        constraint = authority.key() == PROTOCOL_AUTHORITY || 
                    authority.key() == EMERGENCY_AUTHORITY @ FinovaTokenError::Unauthorized
    )]
    pub authority: Signer<'info>,

    /// Mint info account to pause/unpause
    #[account(mut)]
    pub mint_info: Account<'info, MintInfo>,
}

pub fn pause_mint_handler(
    ctx: Context<PauseMint>,
    paused: bool,
) -> Result<()> {
    let clock = Clock::get()?;
    let mint_info = &mut ctx.accounts.mint_info;

    require!(
        mint_info.is_paused != paused,
        FinovaTokenError::InvalidPauseState
    );

    mint_info.is_paused = paused;
    mint_info.updated_at = clock.unix_timestamp;

    emit!(MintPausedEvent {
        mint: mint_info.mint,
        authority: ctx.accounts.authority.key(),
        paused,
        timestamp: clock.unix_timestamp,
    });

    msg!(
        "Mint {} {}: {}",
        mint_info.mint,
        if paused { "paused" } else { "unpaused" },
        ctx.accounts.authority.key()
    );

    Ok(())
}

// Helper functions
fn get_token_name(token_type: TokenType) -> &'static str {
    match token_type {
        TokenType::Fin => "$FIN",
        TokenType::StakedFin => "$sFIN",
        TokenType::USDfin => "$USDfin",
        TokenType::StakedUSDfin => "$sUSDfin",
    }
}

// Events
#[event]
pub struct MintInitializedEvent {
    pub mint: Pubkey,
    pub token_type: u8,
    pub authority: Pubkey,
    pub initial_supply: u64,
    pub max_supply: u64,
    pub timestamp: i64,
}

#[event]
pub struct MintConfigUpdatedEvent {
    pub mint: Pubkey,
    pub authority: Pubkey,
    pub old_config: TokenConfig,
    pub new_config: TokenConfig,
    pub timestamp: i64,
}

#[event]
pub struct MintPausedEvent {
    pub mint: Pubkey,
    pub authority: Pubkey,
    pub paused: bool,
    pub timestamp: i64,
}

// Additional reward types for different tokens
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum RewardType {
    Mining = 0,
    Staking = 1,
    Referral = 2,
    XP = 3,
    Stability = 4,
    Yield = 5,
    Governance = 6,
    Liquidity = 7,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_type_conversion() {
        assert_eq!(TokenType::from_u8(0).unwrap(), TokenType::Fin);
        assert_eq!(TokenType::from_u8(1).unwrap(), TokenType::StakedFin);
        assert_eq!(TokenType::from_u8(2).unwrap(), TokenType::USDfin);
        assert_eq!(TokenType::from_u8(3).unwrap(), TokenType::StakedUSDfin);
        assert!(TokenType::from_u8(4).is_err());
    }

    #[test]
    fn test_token_config() {
        let fin_config = TokenType::Fin.get_config();
        assert_eq!(fin_config.max_supply, MAX_FIN_SUPPLY);
        assert!(fin_config.mining_enabled);
        assert!(fin_config.staking_enabled);
        assert_eq!(fin_config.governance_weight, 100);

        let sfin_config = TokenType::StakedFin.get_config();
        assert!(!sfin_config.mining_enabled);
        assert!(!sfin_config.staking_enabled);
        assert_eq!(sfin_config.governance_weight, 150);
    }

    #[test]
    fn test_reward_type_values() {
        assert_eq!(RewardType::Mining as u8, 0);
        assert_eq!(RewardType::Staking as u8, 1);
        assert_eq!(RewardType::Referral as u8, 2);
        assert_eq!(RewardType::XP as u8, 3);
    }
}
