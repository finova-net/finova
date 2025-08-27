// programs/finova-oracle/src/instructions/initialize_oracle.rs

use anchor_lang::prelude::*;
use crate::constants::*;
use crate::errors::FinovaOracleError;
use crate::state::{OracleConfig, PriceFeed, Aggregator};
use crate::utils::*;

/// Initialize Oracle Instruction
/// Sets up the main oracle configuration and initial price feeds
#[derive(Accounts)]
#[instruction(oracle_id: u64)]
pub struct InitializeOracle<'info> {
    /// Oracle configuration account
    #[account(
        init,
        payer = authority,
        space = OracleConfig::LEN,
        seeds = [ORACLE_CONFIG_SEED, oracle_id.to_le_bytes().as_ref()],
        bump
    )]
    pub oracle_config: Account<'info, OracleConfig>,

    /// Primary aggregator account for price data
    #[account(
        init,
        payer = authority,
        space = Aggregator::LEN,
        seeds = [AGGREGATOR_SEED, oracle_config.key().as_ref()],
        bump
    )]
    pub primary_aggregator: Account<'info, Aggregator>,

    /// Secondary aggregator for redundancy
    #[account(
        init,
        payer = authority,
        space = Aggregator::LEN,
        seeds = [AGGREGATOR_SECONDARY_SEED, oracle_config.key().as_ref()],
        bump
    )]
    pub secondary_aggregator: Account<'info, Aggregator>,

    /// FIN token price feed
    #[account(
        init,
        payer = authority,
        space = PriceFeed::LEN,
        seeds = [PRICE_FEED_SEED, oracle_config.key().as_ref(), FIN_TOKEN_SYMBOL.as_bytes()],
        bump
    )]
    pub fin_price_feed: Account<'info, PriceFeed>,

    /// SOL price feed
    #[account(
        init,
        payer = authority,
        space = PriceFeed::LEN,
        seeds = [PRICE_FEED_SEED, oracle_config.key().as_ref(), SOL_TOKEN_SYMBOL.as_bytes()],
        bump
    )]
    pub sol_price_feed: Account<'info, PriceFeed>,

    /// USDC price feed
    #[account(
        init,
        payer = authority,
        space = PriceFeed::LEN,
        seeds = [PRICE_FEED_SEED, oracle_config.key().as_ref(), USDC_TOKEN_SYMBOL.as_bytes()],
        bump
    )]
    pub usdc_price_feed: Account<'info, PriceFeed>,

    /// Oracle authority (admin)
    #[account(mut)]
    pub authority: Signer<'info>,

    /// System program
    pub system_program: Program<'info, System>,

    /// Rent sysvar
    pub rent: Sysvar<'info, Rent>,

    /// Clock sysvar
    pub clock: Sysvar<'info, Clock>,
}

/// Initialize Oracle Parameters
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct InitializeOracleParams {
    /// Unique identifier for this oracle instance
    pub oracle_id: u64,
    /// Minimum number of valid price sources required
    pub min_sources: u8,
    /// Maximum allowed price deviation percentage (basis points)
    pub max_deviation_bps: u16,
    /// Price update heartbeat in seconds
    pub heartbeat: u64,
    /// Maximum staleness allowed for price data in seconds
    pub max_staleness: u64,
    /// Minimum confidence threshold for price updates
    pub min_confidence: u64,
    /// Emergency pause threshold for price deviation
    pub emergency_threshold_bps: u16,
    /// Initial FIN token price in USD (scaled by PRICE_PRECISION)
    pub initial_fin_price: u64,
    /// Initial SOL price in USD (scaled by PRICE_PRECISION)
    pub initial_sol_price: u64,
    /// Initial USDC price in USD (scaled by PRICE_PRECISION)
    pub initial_usdc_price: u64,
    /// List of authorized price feed operators
    pub authorized_operators: Vec<Pubkey>,
    /// Oracle configuration flags
    pub config_flags: u64,
}

pub fn initialize_oracle(
    ctx: Context<InitializeOracle>,
    params: InitializeOracleParams,
) -> Result<()> {
    let clock = &ctx.accounts.clock;
    let current_timestamp = clock.unix_timestamp;

    // Validate initialization parameters
    validate_initialize_params(&params)?;

    // Initialize Oracle Configuration
    let oracle_config = &mut ctx.accounts.oracle_config;
    oracle_config.oracle_id = params.oracle_id;
    oracle_config.authority = ctx.accounts.authority.key();
    oracle_config.min_sources = params.min_sources;
    oracle_config.max_deviation_bps = params.max_deviation_bps;
    oracle_config.heartbeat = params.heartbeat;
    oracle_config.max_staleness = params.max_staleness;
    oracle_config.min_confidence = params.min_confidence;
    oracle_config.emergency_threshold_bps = params.emergency_threshold_bps;
    oracle_config.config_flags = params.config_flags;
    oracle_config.total_feeds = 3; // FIN, SOL, USDC
    oracle_config.active_feeds = 0; // Will be incremented as feeds are activated
    oracle_config.emergency_mode = false;
    oracle_config.last_update_timestamp = current_timestamp;
    oracle_config.creation_timestamp = current_timestamp;
    oracle_config.bump = ctx.bumps.oracle_config;

    // Copy authorized operators (max 10)
    let max_operators = std::cmp::min(params.authorized_operators.len(), MAX_AUTHORIZED_OPERATORS);
    oracle_config.authorized_operators = [Pubkey::default(); MAX_AUTHORIZED_OPERATORS];
    for (i, operator) in params.authorized_operators.iter().take(max_operators).enumerate() {
        oracle_config.authorized_operators[i] = *operator;
    }
    oracle_config.operator_count = max_operators as u8;

    // Initialize Primary Aggregator
    let primary_aggregator = &mut ctx.accounts.primary_aggregator;
    primary_aggregator.oracle_config = oracle_config.key();
    primary_aggregator.aggregator_type = 0; // Primary
    primary_aggregator.active_feeds = 0;
    primary_aggregator.total_weight = 0;
    primary_aggregator.last_update_timestamp = current_timestamp;
    primary_aggregator.confidence_score = INITIAL_CONFIDENCE_SCORE;
    primary_aggregator.deviation_threshold_bps = params.max_deviation_bps;
    primary_aggregator.is_active = true;
    primary_aggregator.bump = ctx.bumps.primary_aggregator;

    // Initialize feed weights (equal weight initially)
    primary_aggregator.feed_weights = [0; MAX_PRICE_FEEDS];
    primary_aggregator.feed_weights[0] = DEFAULT_FEED_WEIGHT; // FIN
    primary_aggregator.feed_weights[1] = DEFAULT_FEED_WEIGHT; // SOL
    primary_aggregator.feed_weights[2] = DEFAULT_FEED_WEIGHT; // USDC

    // Initialize Secondary Aggregator
    let secondary_aggregator = &mut ctx.accounts.secondary_aggregator;
    secondary_aggregator.oracle_config = oracle_config.key();
    secondary_aggregator.aggregator_type = 1; // Secondary
    secondary_aggregator.active_feeds = 0;
    secondary_aggregator.total_weight = 0;
    secondary_aggregator.last_update_timestamp = current_timestamp;
    secondary_aggregator.confidence_score = INITIAL_CONFIDENCE_SCORE;
    secondary_aggregator.deviation_threshold_bps = params.max_deviation_bps;
    secondary_aggregator.is_active = true;
    secondary_aggregator.bump = ctx.bumps.secondary_aggregator;

    // Initialize secondary aggregator feed weights
    secondary_aggregator.feed_weights = [0; MAX_PRICE_FEEDS];
    secondary_aggregator.feed_weights[0] = DEFAULT_FEED_WEIGHT; // FIN
    secondary_aggregator.feed_weights[1] = DEFAULT_FEED_WEIGHT; // SOL
    secondary_aggregator.feed_weights[2] = DEFAULT_FEED_WEIGHT; // USDC

    // Initialize FIN Price Feed
    initialize_price_feed(
        &mut ctx.accounts.fin_price_feed,
        oracle_config.key(),
        FIN_TOKEN_SYMBOL.to_string(),
        params.initial_fin_price,
        current_timestamp,
        ctx.bumps.fin_price_feed,
    )?;

    // Initialize SOL Price Feed
    initialize_price_feed(
        &mut ctx.accounts.sol_price_feed,
        oracle_config.key(),
        SOL_TOKEN_SYMBOL.to_string(),
        params.initial_sol_price,
        current_timestamp,
        ctx.bumps.sol_price_feed,
    )?;

    // Initialize USDC Price Feed
    initialize_price_feed(
        &mut ctx.accounts.usdc_price_feed,
        oracle_config.key(),
        USDC_TOKEN_SYMBOL.to_string(),
        params.initial_usdc_price,
        current_timestamp,
        ctx.bumps.usdc_price_feed,
    )?;

    // Update aggregator statistics
    primary_aggregator.active_feeds = 3;
    primary_aggregator.total_weight = DEFAULT_FEED_WEIGHT * 3;
    secondary_aggregator.active_feeds = 3;
    secondary_aggregator.total_weight = DEFAULT_FEED_WEIGHT * 3;

    // Update oracle config
    oracle_config.active_feeds = 3;

    // Emit initialization event
    emit!(OracleInitializedEvent {
        oracle_config: oracle_config.key(),
        authority: ctx.accounts.authority.key(),
        oracle_id: params.oracle_id,
        timestamp: current_timestamp,
        initial_feeds: vec![
            InitialFeedData {
                symbol: FIN_TOKEN_SYMBOL.to_string(),
                price: params.initial_fin_price,
                feed_address: ctx.accounts.fin_price_feed.key(),
            },
            InitialFeedData {
                symbol: SOL_TOKEN_SYMBOL.to_string(),
                price: params.initial_sol_price,
                feed_address: ctx.accounts.sol_price_feed.key(),
            },
            InitialFeedData {
                symbol: USDC_TOKEN_SYMBOL.to_string(),
                price: params.initial_usdc_price,
                feed_address: ctx.accounts.usdc_price_feed.key(),
            },
        ],
    });

    msg!(
        "Oracle initialized successfully - ID: {}, Authority: {}, Feeds: {}",
        params.oracle_id,
        ctx.accounts.authority.key(),
        oracle_config.active_feeds
    );

    Ok(())
}

/// Helper function to initialize individual price feeds
fn initialize_price_feed(
    price_feed: &mut Account<PriceFeed>,
    oracle_config: Pubkey,
    symbol: String,
    initial_price: u64,
    timestamp: i64,
    bump: u8,
) -> Result<()> {
    price_feed.oracle_config = oracle_config;
    price_feed.symbol = symbol;
    price_feed.current_price = initial_price;
    price_feed.previous_price = initial_price;
    price_feed.price_history = [0; PRICE_HISTORY_SIZE];
    price_feed.price_history[0] = initial_price;
    price_feed.history_index = 0;
    price_feed.confidence = INITIAL_CONFIDENCE_SCORE;
    price_feed.last_update_timestamp = timestamp;
    price_feed.creation_timestamp = timestamp;
    price_feed.update_count = 1;
    price_feed.source_count = 1; // Initial bootstrap source
    price_feed.deviation_count = 0;
    price_feed.max_deviation_bps = DEFAULT_MAX_DEVIATION_BPS;
    price_feed.is_active = true;
    price_feed.is_emergency_paused = false;
    price_feed.weight = DEFAULT_FEED_WEIGHT;
    price_feed.bump = bump;

    // Initialize price sources
    price_feed.price_sources = [PriceSource::default(); MAX_PRICE_SOURCES];
    price_feed.price_sources[0] = PriceSource {
        source_id: BOOTSTRAP_SOURCE_ID,
        price: initial_price,
        confidence: INITIAL_CONFIDENCE_SCORE,
        timestamp: timestamp,
        is_active: true,
        weight: DEFAULT_SOURCE_WEIGHT,
        deviation_count: 0,
        last_valid_timestamp: timestamp,
    };

    // Calculate initial statistics
    price_feed.price_variance = 0;
    price_feed.moving_average = initial_price;
    price_feed.volume_weighted_price = initial_price;
    price_feed.exponential_moving_average = initial_price;

    Ok(())
}

/// Validate initialization parameters
fn validate_initialize_params(params: &InitializeOracleParams) -> Result<()> {
    // Validate oracle ID
    require!(
        params.oracle_id > 0,
        FinovaOracleError::InvalidOracleId
    );

    // Validate minimum sources
    require!(
        params.min_sources >= MIN_REQUIRED_SOURCES && params.min_sources <= MAX_PRICE_SOURCES as u8,
        FinovaOracleError::InvalidMinSources
    );

    // Validate deviation threshold
    require!(
        params.max_deviation_bps > 0 && params.max_deviation_bps <= MAX_DEVIATION_BPS,
        FinovaOracleError::InvalidDeviationThreshold
    );

    // Validate heartbeat
    require!(
        params.heartbeat >= MIN_HEARTBEAT && params.heartbeat <= MAX_HEARTBEAT,
        FinovaOracleError::InvalidHeartbeat
    );

    // Validate staleness threshold
    require!(
        params.max_staleness >= params.heartbeat && params.max_staleness <= MAX_STALENESS,
        FinovaOracleError::InvalidStalenessThreshold
    );

    // Validate confidence threshold
    require!(
        params.min_confidence >= MIN_CONFIDENCE && params.min_confidence <= MAX_CONFIDENCE,
        FinovaOracleError::InvalidConfidenceThreshold
    );

    // Validate emergency threshold
    require!(
        params.emergency_threshold_bps > params.max_deviation_bps,
        FinovaOracleError::InvalidEmergencyThreshold
    );

    // Validate initial prices
    require!(
        params.initial_fin_price > 0 && params.initial_fin_price <= MAX_PRICE,
        FinovaOracleError::InvalidInitialPrice
    );

    require!(
        params.initial_sol_price > 0 && params.initial_sol_price <= MAX_PRICE,
        FinovaOracleError::InvalidInitialPrice
    );

    require!(
        params.initial_usdc_price >= MIN_STABLECOIN_PRICE && params.initial_usdc_price <= MAX_STABLECOIN_PRICE,
        FinovaOracleError::InvalidInitialPrice
    );

    // Validate operators count
    require!(
        params.authorized_operators.len() <= MAX_AUTHORIZED_OPERATORS,
        FinovaOracleError::TooManyOperators
    );

    // Validate unique operators
    for (i, op1) in params.authorized_operators.iter().enumerate() {
        for op2 in params.authorized_operators.iter().skip(i + 1) {
            require!(
                op1 != op2,
                FinovaOracleError::DuplicateOperator
            );
        }
    }

    Ok(())
}

/// Oracle initialization event
#[event]
pub struct OracleInitializedEvent {
    pub oracle_config: Pubkey,
    pub authority: Pubkey,
    pub oracle_id: u64,
    pub timestamp: i64,
    pub initial_feeds: Vec<InitialFeedData>,
}

/// Initial feed data structure for events
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct InitialFeedData {
    pub symbol: String,
    pub price: u64,
    pub feed_address: Pubkey,
}

/// Price source structure for internal use
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, Default)]
pub struct PriceSource {
    pub source_id: u32,
    pub price: u64,
    pub confidence: u64,
    pub timestamp: i64,
    pub is_active: bool,
    pub weight: u32,
    pub deviation_count: u32,
    pub last_valid_timestamp: i64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use anchor_lang::prelude::*;

    #[test]
    fn test_validate_initialize_params() {
        let valid_params = InitializeOracleParams {
            oracle_id: 1,
            min_sources: 3,
            max_deviation_bps: 500, // 5%
            heartbeat: 60,
            max_staleness: 300,
            min_confidence: 8000,
            emergency_threshold_bps: 1000, // 10%
            initial_fin_price: 1_000_000, // $1.00
            initial_sol_price: 100_000_000, // $100.00
            initial_usdc_price: 1_000_000, // $1.00
            authorized_operators: vec![],
            config_flags: 0,
        };

        assert!(validate_initialize_params(&valid_params).is_ok());

        // Test invalid oracle ID
        let mut invalid_params = valid_params.clone();
        invalid_params.oracle_id = 0;
        assert!(validate_initialize_params(&invalid_params).is_err());

        // Test invalid min sources
        let mut invalid_params = valid_params.clone();
        invalid_params.min_sources = 0;
        assert!(validate_initialize_params(&invalid_params).is_err());

        // Test invalid deviation threshold
        let mut invalid_params = valid_params.clone();
        invalid_params.max_deviation_bps = 0;
        assert!(validate_initialize_params(&invalid_params).is_err());

        // Test invalid emergency threshold
        let mut invalid_params = valid_params.clone();
        invalid_params.emergency_threshold_bps = 100; // Less than max_deviation_bps
        assert!(validate_initialize_params(&invalid_params).is_err());
    }

    #[test]
    fn test_price_source_default() {
        let source = PriceSource::default();
        assert_eq!(source.source_id, 0);
        assert_eq!(source.price, 0);
        assert_eq!(source.confidence, 0);
        assert_eq!(source.timestamp, 0);
        assert!(!source.is_active);
        assert_eq!(source.weight, 0);
        assert_eq!(source.deviation_count, 0);
        assert_eq!(source.last_valid_timestamp, 0);
    }
}
