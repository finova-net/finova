// programs/finova-defi/src/state/mod.rs

use anchor_lang::prelude::*;

pub mod pool;
pub mod liquidity_position;
pub mod farm;
pub mod vault;

pub use pool::*;
pub use liquidity_position::*;
pub use farm::*;
pub use vault::*;

use crate::constants::*;
use crate::errors::*;

/// Core DeFi state management for Finova Network
/// Handles liquidity pools, yield farming, and vault operations
/// 
/// State Architecture:
/// - Pool: AMM liquidity pools with constant product formula
/// - LiquidityPosition: User's LP token positions and rewards
/// - Farm: Yield farming pools with time-weighted rewards
/// - Vault: Strategy vaults for automated yield optimization

/// Pool fee structure for different trading pairs
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq)]
pub struct FeeStructure {
    /// Trading fee in basis points (e.g., 30 = 0.3%)
    pub trade_fee_bps: u16,
    /// Protocol fee in basis points
    pub protocol_fee_bps: u16,
    /// LP fee in basis points
    pub lp_fee_bps: u16,
    /// Finova Network fee in basis points
    pub finova_fee_bps: u16,
}

impl Default for FeeStructure {
    fn default() -> Self {
        Self {
            trade_fee_bps: DEFAULT_TRADE_FEE_BPS,
            protocol_fee_bps: DEFAULT_PROTOCOL_FEE_BPS,
            lp_fee_bps: DEFAULT_LP_FEE_BPS,
            finova_fee_bps: DEFAULT_FINOVA_FEE_BPS,
        }
    }
}

impl FeeStructure {
    /// Calculate total fees in basis points
    pub fn total_fee_bps(&self) -> u16 {
        self.trade_fee_bps + self.protocol_fee_bps + self.lp_fee_bps + self.finova_fee_bps
    }

    /// Validate fee structure doesn't exceed maximum
    pub fn validate(&self) -> Result<()> {
        require!(
            self.total_fee_bps() <= MAX_TOTAL_FEE_BPS,
            FinovaDefiError::InvalidFeeStructure
        );
        Ok(())
    }

    /// Calculate fee amount for a given trade size
    pub fn calculate_fee(&self, amount: u64) -> u64 {
        (amount as u128 * self.total_fee_bps() as u128 / 10000) as u64
    }

    /// Calculate protocol fee portion
    pub fn calculate_protocol_fee(&self, total_fee: u64) -> u64 {
        if self.total_fee_bps() == 0 {
            return 0;
        }
        (total_fee as u128 * self.protocol_fee_bps as u128 / self.total_fee_bps() as u128) as u64
    }

    /// Calculate Finova Network fee portion
    pub fn calculate_finova_fee(&self, total_fee: u64) -> u64 {
        if self.total_fee_bps() == 0 {
            return 0;
        }
        (total_fee as u128 * self.finova_fee_bps as u128 / self.total_fee_bps() as u128) as u64
    }
}

/// Price oracle data for pool operations
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq)]
pub struct PriceData {
    /// Price in 10^18 precision
    pub price: u128,
    /// Confidence interval in basis points
    pub confidence: u16,
    /// Last update timestamp
    pub updated_at: i64,
    /// Number of price feeds aggregated
    pub feed_count: u8,
}

impl PriceData {
    pub fn new(price: u128, confidence: u16) -> Self {
        Self {
            price,
            confidence,
            updated_at: Clock::get().unwrap().unix_timestamp,
            feed_count: 1,
        }
    }

    /// Check if price data is stale
    pub fn is_stale(&self, max_age_seconds: i64) -> bool {
        let current_time = Clock::get().unwrap().unix_timestamp;
        current_time - self.updated_at > max_age_seconds
    }

    /// Check if price data has sufficient confidence
    pub fn is_reliable(&self, min_confidence_bps: u16) -> bool {
        self.confidence >= min_confidence_bps && self.feed_count > 0
    }

    /// Update price with new data point
    pub fn update(&mut self, new_price: u128, new_confidence: u16) {
        // Weighted average of old and new price
        let weight_old = self.confidence as u128;
        let weight_new = new_confidence as u128;
        let total_weight = weight_old + weight_new;

        if total_weight > 0 {
            self.price = (self.price * weight_old + new_price * weight_new) / total_weight;
            self.confidence = ((weight_old + weight_new) / 2) as u16;
        } else {
            self.price = new_price;
            self.confidence = new_confidence;
        }

        self.updated_at = Clock::get().unwrap().unix_timestamp;
        self.feed_count = self.feed_count.saturating_add(1);
    }
}

/// Reward calculation data for farms and vaults
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq)]
pub struct RewardCalculation {
    /// Accumulated reward per share (scaled by 10^12)
    pub acc_reward_per_share: u128,
    /// Last reward calculation timestamp
    pub last_reward_time: i64,
    /// Total staked amount in the pool
    pub total_staked: u64,
    /// Reward rate per second
    pub reward_rate: u64,
    /// Reward period end timestamp
    pub reward_end_time: i64,
}

impl RewardCalculation {
    pub fn new(reward_rate: u64, reward_duration: i64) -> Self {
        let current_time = Clock::get().unwrap().unix_timestamp;
        Self {
            acc_reward_per_share: 0,
            last_reward_time: current_time,
            total_staked: 0,
            reward_rate,
            reward_end_time: current_time + reward_duration,
        }
    }

    /// Update accumulated reward per share
    pub fn update(&mut self) -> Result<()> {
        let current_time = Clock::get().unwrap().unix_timestamp;
        
        if current_time <= self.last_reward_time || self.total_staked == 0 {
            self.last_reward_time = current_time;
            return Ok(());
        }

        let effective_end_time = std::cmp::min(current_time, self.reward_end_time);
        let time_elapsed = effective_end_time - self.last_reward_time;
        
        if time_elapsed > 0 {
            let reward_per_second = self.reward_rate;
            let total_reward = (reward_per_second as u128) * (time_elapsed as u128);
            let reward_per_share = total_reward * REWARD_PRECISION / (self.total_staked as u128);
            
            self.acc_reward_per_share = self.acc_reward_per_share.saturating_add(reward_per_share);
        }
        
        self.last_reward_time = current_time;
        Ok(())
    }

    /// Calculate pending rewards for a user
    pub fn calculate_pending_reward(
        &self,
        user_staked: u64,
        user_reward_debt: u128,
    ) -> u64 {
        if user_staked == 0 {
            return 0;
        }

        let user_share_reward = (user_staked as u128) * self.acc_reward_per_share / REWARD_PRECISION;
        
        if user_share_reward > user_reward_debt {
            ((user_share_reward - user_reward_debt) / REWARD_PRECISION) as u64
        } else {
            0
        }
    }

    /// Add stake to the pool
    pub fn add_stake(&mut self, amount: u64) -> Result<()> {
        self.update()?;
        self.total_staked = self.total_staked.checked_add(amount)
            .ok_or(FinovaDefiError::MathOverflow)?;
        Ok(())
    }

    /// Remove stake from the pool
    pub fn remove_stake(&mut self, amount: u64) -> Result<()> {
        self.update()?;
        require!(
            self.total_staked >= amount,
            FinovaDefiError::InsufficientStake
        );
        self.total_staked = self.total_staked.checked_sub(amount)
            .ok_or(FinovaDefiError::MathUnderflow)?;
        Ok(())
    }

    /// Check if reward period is active
    pub fn is_active(&self) -> bool {
        let current_time = Clock::get().unwrap().unix_timestamp;
        current_time < self.reward_end_time
    }

    /// Extend reward period
    pub fn extend_rewards(&mut self, additional_rewards: u64, additional_duration: i64) -> Result<()> {
        self.update()?;
        
        let current_time = Clock::get().unwrap().unix_timestamp;
        let remaining_time = if self.reward_end_time > current_time {
            self.reward_end_time - current_time
        } else {
            0
        };
        
        let remaining_rewards = (self.reward_rate as u128) * (remaining_time as u128);
        let total_new_rewards = remaining_rewards + (additional_rewards as u128);
        let total_new_duration = remaining_time + additional_duration;
        
        if total_new_duration > 0 {
            self.reward_rate = (total_new_rewards / (total_new_duration as u128)) as u64;
            self.reward_end_time = current_time + total_new_duration;
        }
        
        Ok(())
    }
}

/// Liquidity pool curve types for different AMM formulas
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq)]
pub enum CurveType {
    /// Constant product formula (x * y = k)
    ConstantProduct,
    /// Stable curve for similar assets
    StableCurve { amp: u64 },
    /// Concentrated liquidity curve
    ConcentratedLiquidity { price_range: PriceRange },
}

/// Price range for concentrated liquidity
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq)]
pub struct PriceRange {
    pub min_price: u128,
    pub max_price: u128,
}

impl PriceRange {
    pub fn new(min_price: u128, max_price: u128) -> Result<Self> {
        require!(max_price > min_price, FinovaDefiError::InvalidPriceRange);
        require!(min_price > 0, FinovaDefiError::InvalidPriceRange);
        Ok(Self { min_price, max_price })
    }

    pub fn contains_price(&self, price: u128) -> bool {
        price >= self.min_price && price <= self.max_price
    }

    pub fn width(&self) -> u128 {
        self.max_price - self.min_price
    }
}

/// Pool statistics for analytics and monitoring
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq)]
pub struct PoolStats {
    /// Total volume traded (24h)
    pub volume_24h: u64,
    /// Total volume traded (all time)
    pub volume_total: u64,
    /// Number of trades (24h)
    pub trades_24h: u64,
    /// Number of trades (all time)
    pub trades_total: u64,
    /// Total fees collected
    pub fees_collected: u64,
    /// Last stats update timestamp
    pub last_updated: i64,
    /// Average trade size (24h)
    pub avg_trade_size_24h: u64,
    /// Peak liquidity reached
    pub peak_liquidity: u64,
}

impl Default for PoolStats {
    fn default() -> Self {
        Self {
            volume_24h: 0,
            volume_total: 0,
            trades_24h: 0,
            trades_total: 0,
            fees_collected: 0,
            last_updated: Clock::get().unwrap().unix_timestamp,
            avg_trade_size_24h: 0,
            peak_liquidity: 0,
        }
    }
}

impl PoolStats {
    /// Update stats with new trade
    pub fn update_trade(&mut self, trade_amount: u64, fee_amount: u64) {
        let current_time = Clock::get().unwrap().unix_timestamp;
        
        // Reset 24h stats if more than 24h have passed
        if current_time - self.last_updated > 86400 {
            self.volume_24h = 0;
            self.trades_24h = 0;
            self.avg_trade_size_24h = 0;
        }
        
        // Update counters
        self.volume_24h = self.volume_24h.saturating_add(trade_amount);
        self.volume_total = self.volume_total.saturating_add(trade_amount);
        self.trades_24h = self.trades_24h.saturating_add(1);
        self.trades_total = self.trades_total.saturating_add(1);
        self.fees_collected = self.fees_collected.saturating_add(fee_amount);
        
        // Update average trade size
        if self.trades_24h > 0 {
            self.avg_trade_size_24h = self.volume_24h / self.trades_24h;
        }
        
        self.last_updated = current_time;
    }

    /// Update peak liquidity if current is higher
    pub fn update_peak_liquidity(&mut self, current_liquidity: u64) {
        if current_liquidity > self.peak_liquidity {
            self.peak_liquidity = current_liquidity;
        }
    }

    /// Get APR based on fees and liquidity
    pub fn calculate_fee_apr(&self, total_liquidity: u64) -> u64 {
        if total_liquidity == 0 || self.last_updated == 0 {
            return 0;
        }
        
        // Annualize fees based on 24h performance
        let daily_fees = if self.volume_24h > 0 {
            self.fees_collected * self.volume_24h / self.volume_total.max(self.volume_24h)
        } else {
            0
        };
        
        // APR = (daily_fees * 365) / total_liquidity * 100
        if daily_fees > 0 && total_liquidity > 0 {
            (daily_fees as u128 * 36500 / total_liquidity as u128) as u64
        } else {
            0
        }
    }
}

/// Risk parameters for pools and vaults
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq)]
pub struct RiskParameters {
    /// Maximum slippage allowed in basis points
    pub max_slippage_bps: u16,
    /// Maximum price impact in basis points
    pub max_price_impact_bps: u16,
    /// Minimum liquidity required
    pub min_liquidity: u64,
    /// Maximum single trade size
    pub max_trade_size: u64,
    /// Liquidation threshold for leveraged positions
    pub liquidation_threshold_bps: u16,
    /// Emergency pause threshold
    pub emergency_threshold_bps: u16,
}

impl Default for RiskParameters {
    fn default() -> Self {
        Self {
            max_slippage_bps: DEFAULT_MAX_SLIPPAGE_BPS,
            max_price_impact_bps: DEFAULT_MAX_PRICE_IMPACT_BPS,
            min_liquidity: DEFAULT_MIN_LIQUIDITY,
            max_trade_size: DEFAULT_MAX_TRADE_SIZE,
            liquidation_threshold_bps: DEFAULT_LIQUIDATION_THRESHOLD_BPS,
            emergency_threshold_bps: DEFAULT_EMERGENCY_THRESHOLD_BPS,
        }
    }
}

impl RiskParameters {
    /// Validate trade against risk parameters
    pub fn validate_trade(
        &self,
        trade_size: u64,
        price_impact_bps: u16,
        slippage_bps: u16,
        pool_liquidity: u64,
    ) -> Result<()> {
        require!(
            trade_size <= self.max_trade_size,
            FinovaDefiError::TradeSizeTooLarge
        );
        
        require!(
            price_impact_bps <= self.max_price_impact_bps,
            FinovaDefiError::PriceImpactTooHigh
        );
        
        require!(
            slippage_bps <= self.max_slippage_bps,
            FinovaDefiError::SlippageTooHigh
        );
        
        require!(
            pool_liquidity >= self.min_liquidity,
            FinovaDefiError::InsufficientLiquidity
        );
        
        Ok(())
    }

    /// Check if emergency conditions are met
    pub fn check_emergency_conditions(&self, price_drop_bps: u16) -> bool {
        price_drop_bps >= self.emergency_threshold_bps
    }

    /// Check if position should be liquidated
    pub fn should_liquidate(&self, health_ratio_bps: u16) -> bool {
        health_ratio_bps <= self.liquidation_threshold_bps
    }
}

/// Integration with Finova Network's XP and RP systems
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq)]
pub struct FinovaIntegration {
    /// XP multiplier for DeFi activities
    pub xp_multiplier: u16,
    /// RP bonus for referrals using DeFi
    pub rp_bonus_bps: u16,
    /// $FIN mining boost for DeFi users
    pub mining_boost_bps: u16,
    /// Guild bonus for DeFi participation
    pub guild_bonus_bps: u16,
    /// Last integration update
    pub last_updated: i64,
}

impl Default for FinovaIntegration {
    fn default() -> Self {
        Self {
            xp_multiplier: DEFAULT_DEFI_XP_MULTIPLIER,
            rp_bonus_bps: DEFAULT_DEFI_RP_BONUS_BPS,
            mining_boost_bps: DEFAULT_DEFI_MINING_BOOST_BPS,
            guild_bonus_bps: DEFAULT_DEFI_GUILD_BONUS_BPS,
            last_updated: Clock::get().unwrap().unix_timestamp,
        }
    }
}

impl FinovaIntegration {
    /// Calculate XP reward for DeFi activity
    pub fn calculate_xp_reward(&self, base_xp: u64) -> u64 {
        (base_xp as u128 * self.xp_multiplier as u128 / 100) as u64
    }

    /// Calculate RP bonus for referral DeFi activity
    pub fn calculate_rp_bonus(&self, base_rp: u64) -> u64 {
        (base_rp as u128 * self.rp_bonus_bps as u128 / 10000) as u64
    }

    /// Calculate mining boost
    pub fn calculate_mining_boost(&self, base_mining: u64) -> u64 {
        (base_mining as u128 * self.mining_boost_bps as u128 / 10000) as u64
    }

    /// Update integration parameters
    pub fn update_parameters(
        &mut self,
        xp_multiplier: Option<u16>,
        rp_bonus_bps: Option<u16>,
        mining_boost_bps: Option<u16>,
        guild_bonus_bps: Option<u16>,
    ) {
        if let Some(xp) = xp_multiplier {
            self.xp_multiplier = xp;
        }
        if let Some(rp) = rp_bonus_bps {
            self.rp_bonus_bps = rp;
        }
        if let Some(mining) = mining_boost_bps {
            self.mining_boost_bps = mining;
        }
        if let Some(guild) = guild_bonus_bps {
            self.guild_bonus_bps = guild;
        }
        self.last_updated = Clock::get().unwrap().unix_timestamp;
    }
}

/// Utility functions for state management
pub mod utils {
    use super::*;

    /// Calculate square root using Babylonian method
    pub fn sqrt(x: u128) -> u128 {
        if x == 0 {
            return 0;
        }
        
        let mut z = (x + 1) / 2;
        let mut y = x;
        
        while z < y {
            y = z;
            z = (x / z + z) / 2;
        }
        
        y
    }

    /// Calculate constant product invariant (k = x * y)
    pub fn calculate_invariant(reserve_a: u64, reserve_b: u64) -> u128 {
        (reserve_a as u128) * (reserve_b as u128)
    }

    /// Calculate output amount for constant product swap
    pub fn calculate_swap_output(
        input_amount: u64,
        input_reserve: u64,
        output_reserve: u64,
        fee_bps: u16,
    ) -> Result<u64> {
        require!(input_amount > 0, FinovaDefiError::InvalidAmount);
        require!(input_reserve > 0 && output_reserve > 0, FinovaDefiError::InsufficientLiquidity);

        let input_amount_with_fee = (input_amount as u128) * (10000 - fee_bps as u128) / 10000;
        let numerator = input_amount_with_fee * (output_reserve as u128);
        let denominator = (input_reserve as u128) + input_amount_with_fee;
        
        if denominator == 0 {
            return Err(FinovaDefiError::MathError.into());
        }
        
        Ok((numerator / denominator) as u64)
    }

    /// Calculate price impact in basis points
    pub fn calculate_price_impact(
        input_amount: u64,
        input_reserve: u64,
        output_reserve: u64,
    ) -> u16 {
        if input_reserve == 0 || output_reserve == 0 {
            return 10000; // 100% impact if no liquidity
        }

        let initial_price = (output_reserve as u128 * PRICE_PRECISION) / (input_reserve as u128);
        let new_input_reserve = input_reserve as u128 + input_amount as u128;
        let new_output_reserve = (input_reserve as u128 * output_reserve as u128) / new_input_reserve;
        let final_price = (new_output_reserve * PRICE_PRECISION) / new_input_reserve;

        if initial_price == 0 {
            return 10000;
        }

        let price_change = if initial_price > final_price {
            initial_price - final_price
        } else {
            final_price - initial_price
        };

        ((price_change * 10000) / initial_price) as u16
    }

    /// Validate account ownership and state
    pub fn validate_account_state<T: AccountSerialize + AccountDeserialize + Clone>(
        account: &Account<T>,
        expected_owner: &Pubkey,
    ) -> Result<()> {
        require!(
            account.owner == expected_owner,
            FinovaDefiError::InvalidAccountOwner
        );
        Ok(())
    }

    /// Calculate time-weighted average price
    pub fn calculate_twap(prices: &[(u128, i64)], current_time: i64) -> u128 {
        if prices.is_empty() {
            return 0;
        }

        let mut weighted_sum = 0u128;
        let mut total_weight = 0u128;

        for (price, timestamp) in prices {
            let weight = (current_time - timestamp).max(1) as u128;
            weighted_sum += price * weight;
            total_weight += weight;
        }

        if total_weight > 0 {
            weighted_sum / total_weight
        } else {
            prices[0].0
        }
    }
}
