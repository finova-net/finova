// programs/finova-oracle/src/lib.rs

use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Mint};
use std::collections::BTreeMap;

declare_id!("FinOracle11111111111111111111111111111111111");

pub mod constants;
pub mod errors;
pub mod utils;
pub mod state;
pub mod instructions;
pub mod events;
pub mod math;

use constants::*;
use errors::*;
use state::*;
use instructions::*;
use events::*;
use math::*;

#[program]
pub mod finova_oracle {
    use super::*;

    /// Initialize the Oracle program with configuration
    pub fn initialize_oracle(
        ctx: Context<InitializeOracle>,
        authority: Pubkey,
        update_interval: u64,
        deviation_threshold: u64,
        max_staleness: u64,
    ) -> Result<()> {
        instructions::initialize_oracle::handler(
            ctx,
            authority,
            update_interval,
            deviation_threshold,
            max_staleness,
        )
    }

    /// Update price feed with new data
    pub fn update_price(
        ctx: Context<UpdatePrice>,
        feed_id: String,
        price: u64,
        confidence: u64,
        timestamp: i64,
        source: String,
    ) -> Result<()> {
        instructions::update_price::handler(
            ctx,
            feed_id,
            price,
            confidence,
            timestamp,
            source,
        )
    }

    /// Aggregate multiple price feeds
    pub fn aggregate_feeds(
        ctx: Context<AggregateFeeds>,
        feed_ids: Vec<String>,
        weights: Vec<u64>,
    ) -> Result<()> {
        instructions::aggregate_feeds::handler(ctx, feed_ids, weights)
    }

    /// Emergency price update (governance only)
    pub fn emergency_update(
        ctx: Context<EmergencyUpdate>,
        feed_id: String,
        price: u64,
        reason: String,
    ) -> Result<()> {
        instructions::emergency_update::handler(ctx, feed_id, price, reason)
    }

    /// Add new price feed source
    pub fn add_price_feed(
        ctx: Context<AddPriceFeed>,
        feed_id: String,
        description: String,
        decimals: u8,
        min_sources: u8,
        max_deviation: u64,
    ) -> Result<()> {
        let oracle_config = &mut ctx.accounts.oracle_config;
        let price_feed = &mut ctx.accounts.price_feed;
        let clock = Clock::get()?;

        // Validate authority
        require!(
            ctx.accounts.authority.key() == oracle_config.authority,
            OracleError::Unauthorized
        );

        // Validate parameters
        require!(feed_id.len() <= MAX_FEED_ID_LENGTH, OracleError::InvalidFeedId);
        require!(description.len() <= MAX_DESCRIPTION_LENGTH, OracleError::InvalidDescription);
        require!(decimals <= MAX_DECIMALS, OracleError::InvalidDecimals);
        require!(min_sources > 0 && min_sources <= MAX_SOURCES, OracleError::InvalidMinSources);

        // Initialize price feed
        price_feed.feed_id = feed_id.clone();
        price_feed.description = description;
        price_feed.decimals = decimals;
        price_feed.min_sources = min_sources;
        price_feed.max_deviation = max_deviation;
        price_feed.current_price = 0;
        price_feed.confidence = 0;
        price_feed.last_update = clock.unix_timestamp;
        price_feed.sources = BTreeMap::new();
        price_feed.is_active = true;
        price_feed.aggregated_price = 0;
        price_feed.aggregated_confidence = 0;
        price_feed.update_count = 0;

        emit!(PriceFeedAdded {
            feed_id,
            description: price_feed.description.clone(),
            decimals,
            min_sources,
            authority: ctx.accounts.authority.key(),
            timestamp: clock.unix_timestamp,
        });

        Ok(())
    }

    /// Remove price feed source
    pub fn remove_price_feed(
        ctx: Context<RemovePriceFeed>,
        feed_id: String,
    ) -> Result<()> {
        let oracle_config = &mut ctx.accounts.oracle_config;
        let price_feed = &mut ctx.accounts.price_feed;
        let clock = Clock::get()?;

        // Validate authority
        require!(
            ctx.accounts.authority.key() == oracle_config.authority,
            OracleError::Unauthorized
        );

        // Deactivate price feed
        price_feed.is_active = false;
        price_feed.last_update = clock.unix_timestamp;

        emit!(PriceFeedRemoved {
            feed_id,
            authority: ctx.accounts.authority.key(),
            timestamp: clock.unix_timestamp,
        });

        Ok(())
    }

    /// Update oracle configuration
    pub fn update_config(
        ctx: Context<UpdateConfig>,
        new_authority: Option<Pubkey>,
        new_update_interval: Option<u64>,
        new_deviation_threshold: Option<u64>,
        new_max_staleness: Option<u64>,
    ) -> Result<()> {
        let oracle_config = &mut ctx.accounts.oracle_config;
        let clock = Clock::get()?;

        // Validate authority
        require!(
            ctx.accounts.authority.key() == oracle_config.authority,
            OracleError::Unauthorized
        );

        // Update configuration
        if let Some(authority) = new_authority {
            oracle_config.authority = authority;
        }

        if let Some(interval) = new_update_interval {
            require!(
                interval >= MIN_UPDATE_INTERVAL && interval <= MAX_UPDATE_INTERVAL,
                OracleError::InvalidUpdateInterval
            );
            oracle_config.update_interval = interval;
        }

        if let Some(threshold) = new_deviation_threshold {
            require!(
                threshold <= MAX_DEVIATION_THRESHOLD,
                OracleError::InvalidDeviationThreshold
            );
            oracle_config.deviation_threshold = threshold;
        }

        if let Some(staleness) = new_max_staleness {
            require!(
                staleness >= MIN_STALENESS && staleness <= MAX_STALENESS,
                OracleError::InvalidStaleness
            );
            oracle_config.max_staleness = staleness;
        }

        oracle_config.last_update = clock.unix_timestamp;

        emit!(ConfigUpdated {
            authority: oracle_config.authority,
            update_interval: oracle_config.update_interval,
            deviation_threshold: oracle_config.deviation_threshold,
            max_staleness: oracle_config.max_staleness,
            timestamp: clock.unix_timestamp,
        });

        Ok(())
    }

    /// Get current price for a feed
    pub fn get_price(
        ctx: Context<GetPrice>,
        feed_id: String,
    ) -> Result<PriceData> {
        let price_feed = &ctx.accounts.price_feed;
        let clock = Clock::get()?;

        // Validate feed is active
        require!(price_feed.is_active, OracleError::FeedInactive);

        // Check staleness
        let time_since_update = clock.unix_timestamp - price_feed.last_update;
        require!(
            time_since_update <= ctx.accounts.oracle_config.max_staleness as i64,
            OracleError::StalePrice
        );

        // Return price data
        Ok(PriceData {
            price: price_feed.current_price,
            confidence: price_feed.confidence,
            timestamp: price_feed.last_update,
            feed_id: price_feed.feed_id.clone(),
            decimals: price_feed.decimals,
        })
    }

    /// Validate price deviation
    pub fn validate_price_deviation(
        ctx: Context<ValidatePriceDeviation>,
        feed_id: String,
        new_price: u64,
    ) -> Result<bool> {
        let price_feed = &ctx.accounts.price_feed;
        let oracle_config = &ctx.accounts.oracle_config;

        if price_feed.current_price == 0 {
            return Ok(true); // First price update
        }

        let deviation = utils::calculate_deviation(price_feed.current_price, new_price);
        Ok(deviation <= oracle_config.deviation_threshold)
    }

    /// Pause oracle operations (emergency)
    pub fn pause_oracle(ctx: Context<PauseOracle>) -> Result<()> {
        let oracle_config = &mut ctx.accounts.oracle_config;
        let clock = Clock::get()?;

        // Validate authority
        require!(
            ctx.accounts.authority.key() == oracle_config.authority,
            OracleError::Unauthorized
        );

        oracle_config.is_paused = true;
        oracle_config.last_update = clock.unix_timestamp;

        emit!(OraclePaused {
            authority: ctx.accounts.authority.key(),
            timestamp: clock.unix_timestamp,
        });

        Ok(())
    }

    /// Resume oracle operations
    pub fn resume_oracle(ctx: Context<ResumeOracle>) -> Result<()> {
        let oracle_config = &mut ctx.accounts.oracle_config;
        let clock = Clock::get()?;

        // Validate authority
        require!(
            ctx.accounts.authority.key() == oracle_config.authority,
            OracleError::Unauthorized
        );

        oracle_config.is_paused = false;
        oracle_config.last_update = clock.unix_timestamp;

        emit!(OracleResumed {
            authority: ctx.accounts.authority.key(),
            timestamp: clock.unix_timestamp,
        });

        Ok(())
    }

    /// Add authorized price updater
    pub fn add_price_updater(
        ctx: Context<AddPriceUpdater>,
        updater: Pubkey,
        permissions: u8,
    ) -> Result<()> {
        let oracle_config = &mut ctx.accounts.oracle_config;
        let price_updater = &mut ctx.accounts.price_updater;
        let clock = Clock::get()?;

        // Validate authority
        require!(
            ctx.accounts.authority.key() == oracle_config.authority,
            OracleError::Unauthorized
        );

        // Initialize price updater
        price_updater.updater = updater;
        price_updater.permissions = permissions;
        price_updater.is_active = true;
        price_updater.last_update = clock.unix_timestamp;
        price_updater.update_count = 0;

        emit!(PriceUpdaterAdded {
            updater,
            permissions,
            authority: ctx.accounts.authority.key(),
            timestamp: clock.unix_timestamp,
        });

        Ok(())
    }

    /// Remove authorized price updater
    pub fn remove_price_updater(
        ctx: Context<RemovePriceUpdater>,
        updater: Pubkey,
    ) -> Result<()> {
        let oracle_config = &mut ctx.accounts.oracle_config;
        let price_updater = &mut ctx.accounts.price_updater;
        let clock = Clock::get()?;

        // Validate authority
        require!(
            ctx.accounts.authority.key() == oracle_config.authority,
            OracleError::Unauthorized
        );

        // Deactivate price updater
        price_updater.is_active = false;
        price_updater.last_update = clock.unix_timestamp;

        emit!(PriceUpdaterRemoved {
            updater,
            authority: ctx.accounts.authority.key(),
            timestamp: clock.unix_timestamp,
        });

        Ok(())
    }

    /// Batch update multiple prices
    pub fn batch_update_prices(
        ctx: Context<BatchUpdatePrices>,
        updates: Vec<PriceUpdate>,
    ) -> Result<()> {
        let oracle_config = &ctx.accounts.oracle_config;
        let clock = Clock::get()?;

        // Validate oracle is not paused
        require!(!oracle_config.is_paused, OracleError::OraclePaused);

        // Validate batch size
        require!(
            updates.len() <= MAX_BATCH_SIZE,
            OracleError::BatchTooLarge
        );

        // Process each update
        for (i, update) in updates.iter().enumerate() {
            // Get corresponding price feed account
            let price_feed = &mut ctx.remaining_accounts[i];
            
            // Validate and update price
            utils::validate_and_update_price(
                price_feed,
                &update,
                oracle_config,
                clock.unix_timestamp,
            )?;
        }

        emit!(BatchPricesUpdated {
            count: updates.len() as u32,
            updater: ctx.accounts.updater.key(),
            timestamp: clock.unix_timestamp,
        });

        Ok(())
    }

    /// Get aggregated price for multiple feeds
    pub fn get_aggregated_price(
        ctx: Context<GetAggregatedPrice>,
        feed_ids: Vec<String>,
        weights: Vec<u64>,
    ) -> Result<AggregatedPriceData> {
        let oracle_config = &ctx.accounts.oracle_config;
        let clock = Clock::get()?;

        // Validate inputs
        require!(
            feed_ids.len() == weights.len(),
            OracleError::MismatchedArrays
        );
        require!(
            feed_ids.len() <= MAX_AGGREGATION_FEEDS,
            OracleError::TooManyFeeds
        );

        let mut total_weighted_price: u128 = 0;
        let mut total_weight: u64 = 0;
        let mut min_confidence = u64::MAX;
        let mut latest_timestamp = 0i64;

        // Process each feed
        for (i, feed_id) in feed_ids.iter().enumerate() {
            let price_feed_account = &ctx.remaining_accounts[i];
            let price_feed = PriceFeed::try_deserialize(&mut price_feed_account.data.borrow())?;

            // Validate feed
            require!(price_feed.is_active, OracleError::FeedInactive);
            require!(
                price_feed.feed_id == *feed_id,
                OracleError::InvalidFeedId
            );

            // Check staleness
            let time_since_update = clock.unix_timestamp - price_feed.last_update;
            require!(
                time_since_update <= oracle_config.max_staleness as i64,
                OracleError::StalePrice
            );

            // Calculate weighted contribution
            let weight = weights[i];
            total_weighted_price += (price_feed.current_price as u128) * (weight as u128);
            total_weight += weight;

            // Track minimum confidence and latest timestamp
            min_confidence = min_confidence.min(price_feed.confidence);
            latest_timestamp = latest_timestamp.max(price_feed.last_update);
        }

        // Calculate aggregated price
        require!(total_weight > 0, OracleError::ZeroWeight);
        let aggregated_price = (total_weighted_price / total_weight as u128) as u64;

        Ok(AggregatedPriceData {
            price: aggregated_price,
            confidence: min_confidence,
            timestamp: latest_timestamp,
            feed_count: feed_ids.len() as u8,
            total_weight,
        })
    }
}

// Context structures for instructions
#[derive(Accounts)]
pub struct GetPrice<'info> {
    #[account(
        seeds = [ORACLE_CONFIG_SEED],
        bump
    )]
    pub oracle_config: Account<'info, OracleConfig>,
    
    #[account(
        seeds = [PRICE_FEED_SEED, price_feed.feed_id.as_bytes()],
        bump
    )]
    pub price_feed: Account<'info, PriceFeed>,
}

#[derive(Accounts)]
pub struct ValidatePriceDeviation<'info> {
    #[account(
        seeds = [ORACLE_CONFIG_SEED],
        bump
    )]
    pub oracle_config: Account<'info, OracleConfig>,
    
    #[account(
        seeds = [PRICE_FEED_SEED, price_feed.feed_id.as_bytes()],
        bump
    )]
    pub price_feed: Account<'info, PriceFeed>,
}

#[derive(Accounts)]
pub struct AddPriceFeed<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    
    #[account(
        mut,
        seeds = [ORACLE_CONFIG_SEED],
        bump
    )]
    pub oracle_config: Account<'info, OracleConfig>,
    
    #[account(
        init,
        payer = authority,
        space = PriceFeed::LEN,
        seeds = [PRICE_FEED_SEED, feed_id.as_bytes()],
        bump
    )]
    pub price_feed: Account<'info, PriceFeed>,
    
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct RemovePriceFeed<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    
    #[account(
        seeds = [ORACLE_CONFIG_SEED],
        bump
    )]
    pub oracle_config: Account<'info, OracleConfig>,
    
    #[account(
        mut,
        seeds = [PRICE_FEED_SEED, price_feed.feed_id.as_bytes()],
        bump
    )]
    pub price_feed: Account<'info, PriceFeed>,
}

#[derive(Accounts)]
pub struct UpdateConfig<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    
    #[account(
        mut,
        seeds = [ORACLE_CONFIG_SEED],
        bump
    )]
    pub oracle_config: Account<'info, OracleConfig>,
}

#[derive(Accounts)]
pub struct PauseOracle<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    
    #[account(
        mut,
        seeds = [ORACLE_CONFIG_SEED],
        bump
    )]
    pub oracle_config: Account<'info, OracleConfig>,
}

#[derive(Accounts)]
pub struct ResumeOracle<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    
    #[account(
        mut,
        seeds = [ORACLE_CONFIG_SEED],
        bump
    )]
    pub oracle_config: Account<'info, OracleConfig>,
}

#[derive(Accounts)]
pub struct AddPriceUpdater<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    
    #[account(
        seeds = [ORACLE_CONFIG_SEED],
        bump
    )]
    pub oracle_config: Account<'info, OracleConfig>,
    
    #[account(
        init,
        payer = authority,
        space = PriceUpdater::LEN,
        seeds = [PRICE_UPDATER_SEED, updater.key().as_ref()],
        bump
    )]
    pub price_updater: Account<'info, PriceUpdater>,
    
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct RemovePriceUpdater<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    
    #[account(
        seeds = [ORACLE_CONFIG_SEED],
        bump
    )]
    pub oracle_config: Account<'info, OracleConfig>,
    
    #[account(
        mut,
        seeds = [PRICE_UPDATER_SEED, price_updater.updater.as_ref()],
        bump
    )]
    pub price_updater: Account<'info, PriceUpdater>,
}

#[derive(Accounts)]
pub struct BatchUpdatePrices<'info> {
    #[account(mut)]
    pub updater: Signer<'info>,
    
    #[account(
        seeds = [ORACLE_CONFIG_SEED],
        bump
    )]
    pub oracle_config: Account<'info, OracleConfig>,
    
    #[account(
        seeds = [PRICE_UPDATER_SEED, updater.key().as_ref()],
        bump
    )]
    pub price_updater: Account<'info, PriceUpdater>,
}

#[derive(Accounts)]
pub struct GetAggregatedPrice<'info> {
    #[account(
        seeds = [ORACLE_CONFIG_SEED],
        bump
    )]
    pub oracle_config: Account<'info, OracleConfig>,
}

// Data structures
#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct PriceData {
    pub price: u64,
    pub confidence: u64,
    pub timestamp: i64,
    pub feed_id: String,
    pub decimals: u8,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct AggregatedPriceData {
    pub price: u64,
    pub confidence: u64,
    pub timestamp: i64,
    pub feed_count: u8,
    pub total_weight: u64,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct PriceUpdate {
    pub feed_id: String,
    pub price: u64,
    pub confidence: u64,
    pub source: String,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct PriceUpdater {
    pub updater: Pubkey,
    pub permissions: u8,
    pub is_active: bool,
    pub last_update: i64,
    pub update_count: u64,
}

impl PriceUpdater {
    pub const LEN: usize = 32 + 1 + 1 + 8 + 8 + 8; // 58 bytes
}

// Events
#[event]
pub struct PriceFeedAdded {
    pub feed_id: String,
    pub description: String,
    pub decimals: u8,
    pub min_sources: u8,
    pub authority: Pubkey,
    pub timestamp: i64,
}

#[event]
pub struct PriceFeedRemoved {
    pub feed_id: String,
    pub authority: Pubkey,
    pub timestamp: i64,
}

#[event]
pub struct ConfigUpdated {
    pub authority: Pubkey,
    pub update_interval: u64,
    pub deviation_threshold: u64,
    pub max_staleness: u64,
    pub timestamp: i64,
}

#[event]
pub struct OraclePaused {
    pub authority: Pubkey,
    pub timestamp: i64,
}

#[event]
pub struct OracleResumed {
    pub authority: Pubkey,
    pub timestamp: i64,
}

#[event]
pub struct PriceUpdaterAdded {
    pub updater: Pubkey,
    pub permissions: u8,
    pub authority: Pubkey,
    pub timestamp: i64,
}

#[event]
pub struct PriceUpdaterRemoved {
    pub updater: Pubkey,
    pub authority: Pubkey,
    pub timestamp: i64,
}

#[event]
pub struct BatchPricesUpdated {
    pub count: u32,
    pub updater: Pubkey,
    pub timestamp: i64,
}
