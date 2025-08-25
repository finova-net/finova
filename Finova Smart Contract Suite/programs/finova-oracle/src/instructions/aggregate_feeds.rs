// programs/finova-oracle/src/instructions/aggregate_feeds.rs

use anchor_lang::prelude::*;
use crate::state::{Aggregator, PriceFeed, OracleConfig};
use crate::errors::OracleError;
use crate::constants::*;
use crate::utils::{calculate_weighted_average, validate_price_deviation, calculate_confidence_score};

/// Instruction to aggregate multiple price feeds into a single weighted average price
#[derive(Accounts)]
#[instruction(feeds_count: u8)]
pub struct AggregatePriceFeeds<'info> {
    /// The aggregator account that will store the final aggregated price
    #[account(
        mut,
        seeds = [
            AGGREGATOR_SEED,
            aggregator.symbol.as_bytes(),
            aggregator.version.to_le_bytes().as_ref()
        ],
        bump = aggregator.bump,
        constraint = aggregator.is_active @ OracleError::AggregatorInactive,
        constraint = aggregator.feeds.len() >= MIN_REQUIRED_FEEDS @ OracleError::InsufficientFeeds
    )]
    pub aggregator: Account<'info, Aggregator>,

    /// Oracle configuration account containing global settings
    #[account(
        seeds = [ORACLE_CONFIG_SEED],
        bump = oracle_config.bump,
        constraint = oracle_config.is_active @ OracleError::OracleInactive
    )]
    pub oracle_config: Account<'info, OracleConfig>,

    /// Authority that can trigger aggregation (oracle operator or automated service)
    #[account(
        constraint = authority.key() == oracle_config.aggregation_authority 
            || authority.key() == oracle_config.emergency_authority
            @ OracleError::UnauthorizedAggregation
    )]
    pub authority: Signer<'info>,

    /// Clock sysvar for timestamp validation
    pub clock: Sysvar<'info, Clock>,
}

/// Additional accounts for price feeds (passed as remaining accounts)
#[derive(Clone)]
pub struct PriceFeedAccount<'info> {
    pub price_feed: Account<'info, PriceFeed>,
}

/// Aggregate price feeds with weighted average calculation
pub fn aggregate_price_feeds(
    ctx: Context<AggregatePriceFeeds>,
    feeds_count: u8,
) -> Result<()> {
    let aggregator = &mut ctx.accounts.aggregator;
    let oracle_config = &ctx.accounts.oracle_config;
    let clock = &ctx.accounts.clock;
    let current_time = clock.unix_timestamp;

    // Validate feeds count matches remaining accounts
    require!(
        ctx.remaining_accounts.len() == feeds_count as usize,
        OracleError::InvalidFeedsCount
    );

    require!(
        feeds_count >= MIN_REQUIRED_FEEDS,
        OracleError::InsufficientFeeds
    );

    require!(
        feeds_count <= MAX_FEEDS_PER_AGGREGATOR,
        OracleError::TooManyFeeds
    );

    // Check if aggregation is needed (based on time threshold or deviation)
    let time_since_last_update = current_time - aggregator.last_updated;
    if time_since_last_update < oracle_config.min_aggregation_interval {
        // Check if any feed has significant price deviation
        let mut needs_update = false;
        
        for account_info in ctx.remaining_accounts.iter() {
            let price_feed = Account::<PriceFeed>::try_from(account_info)?;
            
            // Validate feed is registered with this aggregator
            require!(
                aggregator.feeds.contains(&price_feed.key()),
                OracleError::UnregisteredFeed
            );

            // Check if this feed has updated since last aggregation
            if price_feed.last_updated > aggregator.last_updated {
                let deviation = calculate_price_deviation(
                    aggregator.price,
                    price_feed.price
                )?;
                
                if deviation > oracle_config.deviation_threshold {
                    needs_update = true;
                    break;
                }
            }
        }

        if !needs_update {
            return Err(OracleError::NoUpdateRequired.into());
        }
    }

    // Collect valid feeds with their weights and prices
    let mut valid_feeds = Vec::with_capacity(feeds_count as usize);
    let mut total_weight = 0u64;
    let mut latest_update_time = 0i64;

    for account_info in ctx.remaining_accounts.iter() {
        let price_feed = Account::<PriceFeed>::try_from(account_info)?;
        
        // Validate feed is registered and active
        require!(
            aggregator.feeds.contains(&price_feed.key()),
            OracleError::UnregisteredFeed
        );

        require!(
            price_feed.is_active,
            OracleError::InactiveFeed
        );

        // Check feed staleness
        let feed_age = current_time - price_feed.last_updated;
        require!(
            feed_age <= oracle_config.max_feed_staleness,
            OracleError::StaleFeed
        );

        // Validate price is within reasonable bounds
        require!(
            price_feed.price > 0 && price_feed.price < MAX_PRICE,
            OracleError::InvalidPrice
        );

        // Check confidence score threshold
        require!(
            price_feed.confidence >= oracle_config.min_confidence,
            OracleError::LowConfidence
        );

        // Calculate time-based weight adjustment (newer feeds get higher weight)
        let time_weight = calculate_time_weight(
            price_feed.last_updated,
            current_time,
            oracle_config.time_weight_decay
        )?;

        let adjusted_weight = (price_feed.weight as u64)
            .checked_mul(time_weight)
            .ok_or(OracleError::MathOverflow)?
            .checked_div(WEIGHT_PRECISION)
            .ok_or(OracleError::MathOverflow)?;

        valid_feeds.push(ValidFeed {
            price: price_feed.price,
            weight: adjusted_weight,
            confidence: price_feed.confidence,
            last_updated: price_feed.last_updated,
            source: price_feed.source.clone(),
        });

        total_weight = total_weight
            .checked_add(adjusted_weight)
            .ok_or(OracleError::MathOverflow)?;

        if price_feed.last_updated > latest_update_time {
            latest_update_time = price_feed.last_updated;
        }
    }

    // Ensure we have sufficient total weight
    require!(
        total_weight >= oracle_config.min_total_weight,
        OracleError::InsufficientWeight
    );

    // Remove outliers if enabled
    if oracle_config.outlier_detection_enabled {
        valid_feeds = remove_outliers(valid_feeds, oracle_config.outlier_threshold)?;
        
        // Recalculate total weight after outlier removal
        total_weight = valid_feeds.iter()
            .map(|f| f.weight)
            .sum();

        require!(
            valid_feeds.len() >= MIN_REQUIRED_FEEDS as usize,
            OracleError::InsufficientFeedsAfterOutlierRemoval
        );
    }

    // Calculate weighted average price
    let mut weighted_price_sum = 0u128;
    let mut weighted_confidence_sum = 0u128;

    for feed in &valid_feeds {
        let weighted_price = (feed.price as u128)
            .checked_mul(feed.weight as u128)
            .ok_or(OracleError::MathOverflow)?;
        
        let weighted_confidence = (feed.confidence as u128)
            .checked_mul(feed.weight as u128)
            .ok_or(OracleError::MathOverflow)?;

        weighted_price_sum = weighted_price_sum
            .checked_add(weighted_price)
            .ok_or(OracleError::MathOverflow)?;

        weighted_confidence_sum = weighted_confidence_sum
            .checked_add(weighted_confidence)
            .ok_or(OracleError::MathOverflow)?;
    }

    let aggregated_price = weighted_price_sum
        .checked_div(total_weight as u128)
        .ok_or(OracleError::MathOverflow)? as u64;

    let aggregated_confidence = weighted_confidence_sum
        .checked_div(total_weight as u128)
        .ok_or(OracleError::MathOverflow)? as u16;

    // Validate aggregated price is reasonable
    require!(
        aggregated_price > 0 && aggregated_price < MAX_PRICE,
        OracleError::InvalidAggregatedPrice
    );

    // Check for excessive price deviation from previous value
    if aggregator.price > 0 {
        let deviation = calculate_price_deviation(aggregator.price, aggregated_price)?;
        
        if deviation > oracle_config.max_price_change {
            // If deviation is too large, require emergency authority
            require!(
                ctx.accounts.authority.key() == oracle_config.emergency_authority,
                OracleError::ExcessivePriceDeviation
            );
        }
    }

    // Calculate price volatility over recent updates
    let volatility = calculate_price_volatility(
        &aggregator.price_history,
        aggregated_price
    )?;

    // Update aggregator state
    aggregator.price = aggregated_price;
    aggregator.confidence = aggregated_confidence;
    aggregator.last_updated = current_time;
    aggregator.update_count = aggregator.update_count
        .checked_add(1)
        .ok_or(OracleError::MathOverflow)?;
    
    aggregator.volatility = volatility;
    aggregator.feeds_used = valid_feeds.len() as u8;
    aggregator.total_weight_used = total_weight;

    // Update price history (circular buffer)
    update_price_history(&mut aggregator.price_history, aggregated_price, current_time)?;

    // Store feed sources for transparency
    aggregator.last_feed_sources.clear();
    for feed in &valid_feeds {
        if aggregator.last_feed_sources.len() < MAX_FEED_SOURCES {
            aggregator.last_feed_sources.push(feed.source.clone());
        }
    }

    // Emit aggregation event
    emit!(PriceAggregated {
        aggregator: aggregator.key(),
        symbol: aggregator.symbol.clone(),
        price: aggregated_price,
        confidence: aggregated_confidence,
        feeds_used: valid_feeds.len() as u8,
        total_weight: total_weight,
        volatility,
        timestamp: current_time,
    });

    msg!(
        "Price aggregated for {}: {} with confidence {} from {} feeds",
        aggregator.symbol,
        aggregated_price,
        aggregated_confidence,
        valid_feeds.len()
    );

    Ok(())
}

/// Helper struct for valid feed data
#[derive(Clone)]
struct ValidFeed {
    price: u64,
    weight: u64,
    confidence: u16,
    last_updated: i64,
    source: String,
}

/// Calculate time-based weight adjustment
fn calculate_time_weight(
    feed_timestamp: i64,
    current_timestamp: i64,
    decay_rate: u64,
) -> Result<u64> {
    let age = current_timestamp
        .checked_sub(feed_timestamp)
        .ok_or(OracleError::InvalidTimestamp)? as u64;

    if age == 0 {
        return Ok(WEIGHT_PRECISION);
    }

    // Apply exponential decay: weight = e^(-decay_rate * age / WEIGHT_PRECISION)
    let decay_factor = age
        .checked_mul(decay_rate)
        .ok_or(OracleError::MathOverflow)?
        .checked_div(WEIGHT_PRECISION)
        .ok_or(OracleError::MathOverflow)?;

    if decay_factor > 20 {
        // Avoid underflow for very old feeds
        return Ok(1); // Minimum weight
    }

    // Approximate e^(-x) for small x using Taylor series
    let weight = WEIGHT_PRECISION
        .checked_sub(decay_factor)
        .unwrap_or(1);

    Ok(weight.max(1)) // Ensure minimum weight of 1
}

/// Calculate price deviation percentage
fn calculate_price_deviation(old_price: u64, new_price: u64) -> Result<u64> {
    if old_price == 0 {
        return Ok(0);
    }

    let diff = if new_price > old_price {
        new_price - old_price
    } else {
        old_price - new_price
    };

    let deviation = (diff as u128)
        .checked_mul(PERCENTAGE_PRECISION as u128)
        .ok_or(OracleError::MathOverflow)?
        .checked_div(old_price as u128)
        .ok_or(OracleError::MathOverflow)? as u64;

    Ok(deviation)
}

/// Remove statistical outliers from feed data
fn remove_outliers(
    mut feeds: Vec<ValidFeed>,
    outlier_threshold: u64,
) -> Result<Vec<ValidFeed>> {
    if feeds.len() < 3 {
        return Ok(feeds); // Need at least 3 points for outlier detection
    }

    // Sort by price for median calculation
    feeds.sort_by(|a, b| a.price.cmp(&b.price));

    let median_price = if feeds.len() % 2 == 0 {
        let mid = feeds.len() / 2;
        (feeds[mid - 1].price + feeds[mid].price) / 2
    } else {
        feeds[feeds.len() / 2].price
    };

    // Calculate median absolute deviation (MAD)
    let mut deviations: Vec<u64> = feeds.iter()
        .map(|f| {
            if f.price > median_price {
                f.price - median_price
            } else {
                median_price - f.price
            }
        })
        .collect();

    deviations.sort_unstable();
    let mad = if deviations.len() % 2 == 0 {
        let mid = deviations.len() / 2;
        (deviations[mid - 1] + deviations[mid]) / 2
    } else {
        deviations[deviations.len() / 2]
    };

    // Filter out outliers using modified Z-score
    let modified_z_threshold = outlier_threshold;
    let filtered_feeds: Vec<ValidFeed> = feeds.into_iter()
        .filter(|feed| {
            if mad == 0 {
                return true; // No variation, keep all feeds
            }

            let deviation = if feed.price > median_price {
                feed.price - median_price
            } else {
                median_price - feed.price
            };

            let modified_z_score = (deviation * PERCENTAGE_PRECISION) / mad;
            modified_z_score <= modified_z_threshold
        })
        .collect();

    Ok(filtered_feeds)
}

/// Calculate price volatility based on historical data
fn calculate_price_volatility(
    price_history: &[PricePoint],
    current_price: u64,
) -> Result<u16> {
    if price_history.is_empty() {
        return Ok(0);
    }

    let mut returns = Vec::with_capacity(price_history.len());
    let mut prev_price = price_history[0].price;

    for point in price_history.iter().skip(1) {
        if prev_price > 0 {
            let return_val = if point.price > prev_price {
                ((point.price - prev_price) as u128 * PERCENTAGE_PRECISION as u128) / prev_price as u128
            } else {
                ((prev_price - point.price) as u128 * PERCENTAGE_PRECISION as u128) / prev_price as u128
            };
            returns.push(return_val as u64);
        }
        prev_price = point.price;
    }

    // Add current price return
    if prev_price > 0 {
        let current_return = if current_price > prev_price {
            ((current_price - prev_price) as u128 * PERCENTAGE_PRECISION as u128) / prev_price as u128
        } else {
            ((prev_price - current_price) as u128 * PERCENTAGE_PRECISION as u128) / prev_price as u128
        };
        returns.push(current_return as u64);
    }

    if returns.is_empty() {
        return Ok(0);
    }

    // Calculate standard deviation of returns
    let mean_return = returns.iter().sum::<u64>() / returns.len() as u64;
    
    let variance = returns.iter()
        .map(|&r| {
            let diff = if r > mean_return { r - mean_return } else { mean_return - r };
            (diff as u128).pow(2)
        })
        .sum::<u128>() / returns.len() as u128;

    // Approximate square root for volatility
    let volatility = (variance as f64).sqrt() as u64;
    
    Ok((volatility / (PERCENTAGE_PRECISION / 10000)) as u16) // Convert to basis points
}

/// Update circular price history buffer
fn update_price_history(
    price_history: &mut Vec<PricePoint>,
    price: u64,
    timestamp: i64,
) -> Result<()> {
    let new_point = PricePoint { price, timestamp };

    if price_history.len() >= MAX_PRICE_HISTORY {
        // Remove oldest entry
        price_history.remove(0);
    }

    price_history.push(new_point);
    Ok(())
}

/// Price point for historical data
#[derive(Clone, Copy)]
pub struct PricePoint {
    pub price: u64,
    pub timestamp: i64,
}

/// Event emitted when prices are aggregated
#[event]
pub struct PriceAggregated {
    pub aggregator: Pubkey,
    pub symbol: String,
    pub price: u64,
    pub confidence: u16,
    pub feeds_used: u8,
    pub total_weight: u64,
    pub volatility: u16,
    pub timestamp: i64,
}

// Constants for aggregation logic
const MIN_REQUIRED_FEEDS: u8 = 3;
const MAX_FEEDS_PER_AGGREGATOR: u8 = 10;
const MAX_PRICE: u64 = 1_000_000_000_000_000; // 1M with 9 decimals
const WEIGHT_PRECISION: u64 = 10_000;
const PERCENTAGE_PRECISION: u64 = 10_000; // 100.00%
const MAX_FEED_SOURCES: usize = 10;
const MAX_PRICE_HISTORY: usize = 100;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_time_weight() {
        let current_time = 1000;
        let recent_time = 950;
        let old_time = 500;
        let decay_rate = 100;

        let recent_weight = calculate_time_weight(recent_time, current_time, decay_rate).unwrap();
        let old_weight = calculate_time_weight(old_time, current_time, decay_rate).unwrap();

        assert!(recent_weight > old_weight);
        assert!(recent_weight <= WEIGHT_PRECISION);
    }

    #[test]
    fn test_calculate_price_deviation() {
        let old_price = 1000;
        let new_price = 1100;
        
        let deviation = calculate_price_deviation(old_price, new_price).unwrap();
        assert_eq!(deviation, 1000); // 10.00%
    }

    #[test]
    fn test_remove_outliers() {
        let feeds = vec![
            ValidFeed { price: 100, weight: 100, confidence: 100, last_updated: 0, source: "A".to_string() },
            ValidFeed { price: 105, weight: 100, confidence: 100, last_updated: 0, source: "B".to_string() },
            ValidFeed { price: 200, weight: 100, confidence: 100, last_updated: 0, source: "C".to_string() }, // Outlier
            ValidFeed { price: 102, weight: 100, confidence: 100, last_updated: 0, source: "D".to_string() },
        ];

        let filtered = remove_outliers(feeds, 3500).unwrap(); // 35% threshold
        assert_eq!(filtered.len(), 3); // Should remove the outlier
    }
}
