// programs/finova-oracle/src/instructions/update_price.rs

use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Mint};
use crate::constants::*;
use crate::errors::*;
use crate::state::*;
use crate::utils::*;
use std::collections::HashMap;

/// Update price feed with new price data
/// Implements weighted aggregation, outlier detection, and validator consensus
#[derive(Accounts)]
#[instruction(price_feed_key: String)]
pub struct UpdatePrice<'info> {
    #[account(
        mut,
        seeds = [
            PRICE_FEED_SEED.as_bytes(),
            price_feed_key.as_bytes()
        ],
        bump,
        constraint = price_feed.is_active @ OracleError::PriceFeedInactive,
        constraint = price_feed.validator_count > 0 @ OracleError::NoValidatorsRegistered
    )]
    pub price_feed: Account<'info, PriceFeed>,

    #[account(
        mut,
        seeds = [ORACLE_CONFIG_SEED.as_bytes()],
        bump,
        constraint = oracle_config.is_active @ OracleError::OracleInactive,
        constraint = !oracle_config.emergency_pause @ OracleError::EmergencyPauseActive
    )]
    pub oracle_config: Account<'info, OracleConfig>,

    #[account(
        mut,
        constraint = validator.is_active @ OracleError::ValidatorInactive,
        constraint = validator.reputation_score >= MIN_VALIDATOR_REPUTATION @ OracleError::InsufficientReputation,
        constraint = Clock::get()?.unix_timestamp - validator.last_update_time >= MIN_UPDATE_INTERVAL @ OracleError::UpdateTooFrequent
    )]
    pub validator: Account<'info, ValidatorAccount>,

    #[account(
        mut,
        constraint = validator_token_account.owner == validator.authority @ OracleError::InvalidValidatorAccount,
        constraint = validator_token_account.mint == oracle_config.reward_mint @ OracleError::InvalidRewardMint,
        constraint = validator_token_account.amount >= oracle_config.min_stake_amount @ OracleError::InsufficientStake
    )]
    pub validator_token_account: Account<'info, TokenAccount>,

    #[account(
        mut,
        seeds = [REWARD_POOL_SEED.as_bytes()],
        bump,
        constraint = reward_pool.mint == oracle_config.reward_mint @ OracleError::InvalidRewardPool
    )]
    pub reward_pool: Account<'info, TokenAccount>,

    #[account(
        constraint = fin_mint.key() == oracle_config.reward_mint @ OracleError::InvalidRewardMint
    )]
    pub fin_mint: Account<'info, Mint>,

    #[account(mut)]
    pub authority: Signer<'info>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub clock: Sysvar<'info, Clock>,
}

/// Price update data structure
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct PriceUpdateData {
    /// New price value (scaled by decimals)
    pub price: u64,
    /// Confidence interval (basis points)
    pub confidence: u64,
    /// External source identifier
    pub source_id: u8,
    /// Timestamp when price was observed
    pub observed_timestamp: i64,
    /// Digital signature for authenticity
    pub signature: [u8; 64],
    /// Additional metadata
    pub metadata: PriceMetadata,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct PriceMetadata {
    /// Volume weighted average price
    pub vwap: Option<u64>,
    /// 24h trading volume
    pub volume_24h: Option<u64>,
    /// Market cap
    pub market_cap: Option<u64>,
    /// Circulating supply
    pub circulating_supply: Option<u64>,
    /// Exchange-specific data
    pub exchange_data: Vec<ExchangeData>,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct ExchangeData {
    pub exchange_id: u8,
    pub price: u64,
    pub volume: u64,
    pub last_trade_time: i64,
}

/// Advanced price validation parameters
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct ValidationParams {
    /// Maximum allowed price deviation (basis points)
    pub max_deviation: u64,
    /// Minimum confidence threshold
    pub min_confidence: u64,
    /// Maximum allowed staleness (seconds)
    pub max_staleness: i64,
    /// Required number of sources
    pub min_sources: u8,
    /// Outlier detection sensitivity
    pub outlier_threshold: u64,
}

impl Default for ValidationParams {
    fn default() -> Self {
        Self {
            max_deviation: 1000, // 10%
            min_confidence: 8000, // 80%
            max_staleness: 300, // 5 minutes
            min_sources: 3,
            outlier_threshold: 2000, // 20%
        }
    }
}

pub fn update_price(
    ctx: Context<UpdatePrice>,
    price_feed_key: String,
    price_data: PriceUpdateData,
    validation_params: Option<ValidationParams>,
) -> Result<()> {
    let clock = Clock::get()?;
    let current_time = clock.unix_timestamp;
    let price_feed = &mut ctx.accounts.price_feed;
    let validator = &mut ctx.accounts.validator;
    let oracle_config = &ctx.accounts.oracle_config;

    // Input validation
    require!(
        price_feed_key.len() <= MAX_PRICE_FEED_KEY_LENGTH,
        OracleError::InvalidPriceFeedKey
    );
    
    require!(
        price_data.price > 0,
        OracleError::InvalidPrice
    );

    require!(
        price_data.confidence <= 10000, // Max 100%
        OracleError::InvalidConfidence
    );

    require!(
        current_time - price_data.observed_timestamp <= MAX_PRICE_STALENESS,
        OracleError::StalePrice
    );

    // Get validation parameters
    let params = validation_params.unwrap_or_default();

    // Verify validator signature
    verify_price_signature(&price_data, &validator.public_key)?;

    // Advanced price validation
    validate_price_data(&price_data, &params, price_feed, current_time)?;

    // Detect and handle outliers
    let is_outlier = detect_price_outlier(&price_data, price_feed, &params)?;
    
    if is_outlier {
        // Penalize validator for submitting outlier
        validator.reputation_score = validator.reputation_score
            .saturating_sub(OUTLIER_REPUTATION_PENALTY);
        
        // Require additional validation for outliers
        require!(
            price_data.confidence >= params.min_confidence + 1000, // Extra 10% confidence
            OracleError::OutlierRequiresHighConfidence
        );
        
        msg!("Outlier price detected from validator: {}", validator.authority);
    }

    // Calculate weighted price using multiple algorithms
    let weighted_price = calculate_weighted_price(price_feed, &price_data, &params)?;
    
    // Update price feed with exponential moving average
    let alpha = calculate_smoothing_factor(price_feed.update_frequency, oracle_config.smoothing_period);
    let new_price = apply_exponential_smoothing(price_feed.current_price, weighted_price, alpha);

    // Confidence decay based on time since last update
    let time_since_update = current_time - price_feed.last_update_time;
    let confidence_decay = calculate_confidence_decay(time_since_update, oracle_config.confidence_decay_rate);
    let adjusted_confidence = (price_data.confidence as u128 * confidence_decay / 10000) as u64;

    // Circuit breaker for extreme price movements
    let price_change_pct = calculate_price_change_percentage(price_feed.current_price, new_price);
    if price_change_pct > oracle_config.circuit_breaker_threshold {
        // Trigger circuit breaker
        oracle_config.emergency_pause = true;
        oracle_config.circuit_breaker_triggered_at = current_time;
        
        emit!(CircuitBreakerTriggered {
            price_feed_key: price_feed_key.clone(),
            old_price: price_feed.current_price,
            new_price,
            change_percentage: price_change_pct,
            timestamp: current_time,
        });
        
        return Err(OracleError::CircuitBreakerTriggered.into());
    }

    // Update price feed state
    price_feed.previous_price = price_feed.current_price;
    price_feed.current_price = new_price;
    price_feed.confidence = adjusted_confidence;
    price_feed.last_update_time = current_time;
    price_feed.update_count += 1;
    price_feed.total_volume += price_data.metadata.volume_24h.unwrap_or(0);

    // Update validator statistics
    validator.last_update_time = current_time;
    validator.total_updates += 1;
    validator.accuracy_score = calculate_validator_accuracy(validator, &price_data, price_feed);

    // Reward validator based on performance
    let reward_amount = calculate_validator_reward(
        validator,
        &price_data,
        price_feed,
        oracle_config,
        is_outlier,
    )?;

    if reward_amount > 0 {
        // Transfer FIN tokens as reward
        let cpi_accounts = token::Transfer {
            from: ctx.accounts.reward_pool.to_account_info(),
            to: ctx.accounts.validator_token_account.to_account_info(),
            authority: ctx.accounts.oracle_config.to_account_info(),
        };

        let seeds = &[
            ORACLE_CONFIG_SEED.as_bytes(),
            &[ctx.accounts.oracle_config.bump],
        ];
        let signer = &[&seeds[..]];

        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);

        token::transfer(cpi_ctx, reward_amount)?;

        // Update validator rewards
        validator.total_rewards_earned += reward_amount;
        validator.reputation_score = validator.reputation_score
            .saturating_add(SUCCESSFUL_UPDATE_REPUTATION_BONUS);
    }

    // Update oracle statistics
    oracle_config.total_updates += 1;
    oracle_config.last_global_update = current_time;

    // Emit price update event
    emit!(PriceUpdated {
        price_feed_key,
        validator: validator.authority,
        old_price: price_feed.previous_price,
        new_price: price_feed.current_price,
        confidence: price_feed.confidence,
        timestamp: current_time,
        reward_amount,
        is_outlier,
        accuracy_score: validator.accuracy_score,
    });

    Ok(())
}

/// Verify cryptographic signature of price data
fn verify_price_signature(
    price_data: &PriceUpdateData,
    validator_pubkey: &Pubkey,
) -> Result<()> {
    // Create message hash from price data
    let message = create_price_message_hash(price_data)?;
    
    // Verify Ed25519 signature
    let signature = solana_program::ed25519_program::Signature::from(price_data.signature);
    
    solana_program::ed25519_program::verify(
        &validator_pubkey.to_bytes(),
        &message,
        &signature.0,
    ).map_err(|_| OracleError::InvalidSignature)?;

    Ok(())
}

/// Create deterministic hash of price data for signature verification
fn create_price_message_hash(price_data: &PriceUpdateData) -> Result<[u8; 32]> {
    use solana_program::hash::{hash, Hash};
    
    let mut message = Vec::new();
    message.extend_from_slice(&price_data.price.to_le_bytes());
    message.extend_from_slice(&price_data.confidence.to_le_bytes());
    message.extend_from_slice(&[price_data.source_id]);
    message.extend_from_slice(&price_data.observed_timestamp.to_le_bytes());
    
    // Include metadata hash
    if let Some(vwap) = price_data.metadata.vwap {
        message.extend_from_slice(&vwap.to_le_bytes());
    }
    
    let hash_result = hash(&message);
    Ok(hash_result.to_bytes())
}

/// Comprehensive price data validation
fn validate_price_data(
    price_data: &PriceUpdateData,
    params: &ValidationParams,
    price_feed: &PriceFeed,
    current_time: i64,
) -> Result<()> {
    // Staleness check
    let age = current_time - price_data.observed_timestamp;
    require!(
        age <= params.max_staleness,
        OracleError::StalePrice
    );

    // Confidence threshold
    require!(
        price_data.confidence >= params.min_confidence,
        OracleError::InsufficientConfidence
    );

    // Source count validation
    require!(
        price_data.metadata.exchange_data.len() >= params.min_sources as usize,
        OracleError::InsufficientSources
    );

    // Price deviation check (if previous price exists)
    if price_feed.current_price > 0 {
        let deviation = calculate_price_deviation(price_feed.current_price, price_data.price);
        require!(
            deviation <= params.max_deviation,
            OracleError::ExcessivePriceDeviation
        );
    }

    // Cross-source validation
    validate_cross_source_consistency(&price_data.metadata.exchange_data, params)?;

    Ok(())
}

/// Detect price outliers using statistical methods
fn detect_price_outlier(
    price_data: &PriceUpdateData,
    price_feed: &PriceFeed,
    params: &ValidationParams,
) -> Result<bool> {
    // Use Z-score method for outlier detection
    if price_feed.price_history.len() < MIN_HISTORY_FOR_OUTLIER_DETECTION {
        return Ok(false); // Not enough data
    }

    let mean = calculate_historical_mean(&price_feed.price_history);
    let std_dev = calculate_historical_std_dev(&price_feed.price_history, mean);
    
    if std_dev == 0 {
        return Ok(false); // No variance
    }

    let z_score = ((price_data.price as i64 - mean as i64).abs() as f64) / (std_dev as f64);
    let threshold = (params.outlier_threshold as f64) / 1000.0; // Convert basis points to decimal

    Ok(z_score > threshold)
}

/// Calculate weighted price using multiple data sources
fn calculate_weighted_price(
    price_feed: &PriceFeed,
    price_data: &PriceUpdateData,
    _params: &ValidationParams,
) -> Result<u64> {
    let exchange_data = &price_data.metadata.exchange_data;
    
    if exchange_data.is_empty() {
        return Ok(price_data.price);
    }

    let mut total_weighted_price: u128 = 0;
    let mut total_weight: u128 = 0;

    for exchange in exchange_data {
        // Weight by volume and recency
        let volume_weight = (exchange.volume as u128).min(1_000_000_000_000); // Cap weight
        let recency_weight = calculate_recency_weight(exchange.last_trade_time, Clock::get()?.unix_timestamp);
        let combined_weight = (volume_weight * recency_weight) / 10000;

        total_weighted_price += (exchange.price as u128) * combined_weight;
        total_weight += combined_weight;
    }

    if total_weight == 0 {
        return Ok(price_data.price);
    }

    let weighted_price = (total_weighted_price / total_weight) as u64;
    Ok(weighted_price)
}

/// Calculate exponential moving average smoothing factor
fn calculate_smoothing_factor(update_frequency: u64, smoothing_period: u64) -> u64 {
    if smoothing_period == 0 {
        return 10000; // No smoothing (100%)
    }
    
    // Alpha = 2 / (N + 1) where N is smoothing period
    let alpha = (20000 / (smoothing_period + 1)).min(10000);
    alpha
}

/// Apply exponential smoothing to price
fn apply_exponential_smoothing(old_price: u64, new_price: u64, alpha: u64) -> u64 {
    if old_price == 0 {
        return new_price;
    }

    let alpha_scaled = alpha as u128;
    let old_weight = 10000 - alpha_scaled;
    
    let smoothed = ((new_price as u128 * alpha_scaled) + (old_price as u128 * old_weight)) / 10000;
    smoothed as u64
}

/// Calculate confidence decay based on time
fn calculate_confidence_decay(time_elapsed: i64, decay_rate: u64) -> u128 {
    if time_elapsed <= 0 || decay_rate == 0 {
        return 10000; // No decay
    }

    // Exponential decay: confidence = initial * e^(-rate * time)
    let decay_factor = (decay_rate as u128 * time_elapsed as u128) / 3600; // Per hour
    let decay_multiplier = 10000u128.saturating_sub(decay_factor.min(9000)); // Min 10% confidence
    
    decay_multiplier
}

/// Calculate percentage change between prices
fn calculate_price_change_percentage(old_price: u64, new_price: u64) -> u64 {
    if old_price == 0 {
        return 0;
    }

    let change = if new_price > old_price {
        new_price - old_price
    } else {
        old_price - new_price
    };

    ((change as u128 * 10000) / old_price as u128) as u64
}

/// Calculate validator accuracy score
fn calculate_validator_accuracy(
    validator: &ValidatorAccount,
    price_data: &PriceUpdateData,
    price_feed: &PriceFeed,
) -> u64 {
    if price_feed.current_price == 0 {
        return validator.accuracy_score;
    }

    let deviation = calculate_price_deviation(price_feed.current_price, price_data.price);
    let accuracy = 10000u64.saturating_sub(deviation.min(10000));
    
    // Weighted average with historical accuracy
    let weight = 100; // Weight for new measurement
    let historical_weight = validator.total_updates.min(900); // Max historical weight
    
    if historical_weight == 0 {
        return accuracy;
    }

    let weighted_accuracy = ((accuracy as u128 * weight as u128) + 
                           (validator.accuracy_score as u128 * historical_weight as u128)) /
                          (weight as u128 + historical_weight as u128);
    
    weighted_accuracy as u64
}

/// Calculate validator reward based on performance
fn calculate_validator_reward(
    validator: &ValidatorAccount,
    price_data: &PriceUpdateData,
    price_feed: &PriceFeed,
    oracle_config: &OracleConfig,
    is_outlier: bool,
) -> Result<u64> {
    let base_reward = oracle_config.base_validator_reward;
    
    if is_outlier {
        return Ok(base_reward / 10); // Reduced reward for outliers
    }

    // Performance multipliers
    let confidence_multiplier = (price_data.confidence * 100) / 10000; // 0-100%
    let accuracy_multiplier = (validator.accuracy_score * 100) / 10000; // 0-100%
    let reputation_multiplier = (validator.reputation_score.min(10000) * 100) / 10000; // 0-100%
    
    let performance_score = (confidence_multiplier + accuracy_multiplier + reputation_multiplier) / 3;
    let reward = (base_reward as u128 * performance_score as u128) / 100;
    
    Ok(reward.min(base_reward as u128 * 2) as u64) // Cap at 2x base reward
}

/// Validate consistency across multiple data sources
fn validate_cross_source_consistency(
    exchange_data: &[ExchangeData],
    params: &ValidationParams,
) -> Result<()> {
    if exchange_data.len() < 2 {
        return Ok(()); // Can't validate consistency with single source
    }

    let prices: Vec<u64> = exchange_data.iter().map(|e| e.price).collect();
    let mean_price = prices.iter().sum::<u64>() / prices.len() as u64;
    
    for price in &prices {
        let deviation = calculate_price_deviation(mean_price, *price);
        require!(
            deviation <= params.max_deviation * 2, // Allow 2x deviation for cross-source
            OracleError::InconsistentSources
        );
    }

    Ok(())
}

/// Calculate price deviation in basis points
fn calculate_price_deviation(base_price: u64, comparison_price: u64) -> u64 {
    if base_price == 0 {
        return 0;
    }

    let difference = if comparison_price > base_price {
        comparison_price - base_price
    } else {
        base_price - comparison_price
    };

    ((difference as u128 * 10000) / base_price as u128) as u64
}

/// Calculate recency weight for time-based decay
fn calculate_recency_weight(timestamp: i64, current_time: i64) -> u128 {
    let age = current_time - timestamp;
    if age <= 0 {
        return 10000; // Future timestamp, full weight
    }

    // Decay over 1 hour: weight = e^(-age/3600)
    let decay_factor = (age as u128).min(7200); // Cap at 2 hours
    let weight = 10000u128.saturating_sub((decay_factor * 1000) / 3600);
    weight.max(1000) // Minimum 10% weight
}

/// Calculate historical mean price
fn calculate_historical_mean(price_history: &[u64]) -> u64 {
    if price_history.is_empty() {
        return 0;
    }
    
    let sum: u128 = price_history.iter().map(|&p| p as u128).sum();
    (sum / price_history.len() as u128) as u64
}

/// Calculate historical standard deviation
fn calculate_historical_std_dev(price_history: &[u64], mean: u64) -> u64 {
    if price_history.len() < 2 {
        return 0;
    }

    let variance: u128 = price_history
        .iter()
        .map(|&price| {
            let diff = if price > mean { price - mean } else { mean - price };
            (diff as u128).pow(2)
        })
        .sum::<u128>() / (price_history.len() - 1) as u128;

    // Integer square root approximation
    integer_sqrt(variance) as u64
}

/// Integer square root using binary search
fn integer_sqrt(n: u128) -> u128 {
    if n == 0 {
        return 0;
    }
    
    let mut x = n;
    let mut y = (x + 1) / 2;
    
    while y < x {
        x = y;
        y = (x + n / x) / 2;
    }
    
    x
}

// Events
#[event]
pub struct PriceUpdated {
    pub price_feed_key: String,
    pub validator: Pubkey,
    pub old_price: u64,
    pub new_price: u64,
    pub confidence: u64,
    pub timestamp: i64,
    pub reward_amount: u64,
    pub is_outlier: bool,
    pub accuracy_score: u64,
}

#[event]
pub struct CircuitBreakerTriggered {
    pub price_feed_key: String,
    pub old_price: u64,
    pub new_price: u64,
    pub change_percentage: u64,
    pub timestamp: i64,
}

// Constants
const MAX_PRICE_FEED_KEY_LENGTH: usize = 32;
const MAX_PRICE_STALENESS: i64 = 300; // 5 minutes
const MIN_VALIDATOR_REPUTATION: u64 = 1000;
const MIN_UPDATE_INTERVAL: i64 = 10; // 10 seconds
const OUTLIER_REPUTATION_PENALTY: u64 = 100;
const SUCCESSFUL_UPDATE_REPUTATION_BONUS: u64 = 10;
const MIN_HISTORY_FOR_OUTLIER_DETECTION: usize = 10;
const PRICE_FEED_SEED: &str = "price_feed";
const ORACLE_CONFIG_SEED: &str = "oracle_config";
const REWARD_POOL_SEED: &str = "reward_pool";
