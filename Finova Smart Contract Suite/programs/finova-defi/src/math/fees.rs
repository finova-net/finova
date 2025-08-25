// programs/finova-defi/src/math/fees.rs

use anchor_lang::prelude::*;
use std::ops::{Add, Div, Mul, Sub};

/// Fee calculation utilities for Finova DeFi operations
/// Implements dynamic fee structures with anti-whale mechanisms
/// and user tier-based discounts from the whitepaper

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FeeConfig {
    /// Base trading fee in basis points (0.1% = 10 bp)
    pub base_trading_fee_bp: u16,
    /// Liquidity provider fee in basis points
    pub lp_fee_bp: u16,
    /// Protocol fee in basis points
    pub protocol_fee_bp: u16,
    /// Flash loan fee in basis points
    pub flash_loan_fee_bp: u16,
    /// Maximum fee cap in basis points
    pub max_fee_cap_bp: u16,
    /// Minimum fee floor in basis points
    pub min_fee_floor_bp: u16,
}

impl Default for FeeConfig {
    fn default() -> Self {
        Self {
            base_trading_fee_bp: 30,     // 0.3%
            lp_fee_bp: 20,               // 0.2%
            protocol_fee_bp: 10,         // 0.1%
            flash_loan_fee_bp: 50,       // 0.5%
            max_fee_cap_bp: 100,         // 1.0%
            min_fee_floor_bp: 5,         // 0.05%
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UserTier {
    pub xp_level: u32,
    pub rp_tier: u32,
    pub staked_fin: u64,
    pub trading_volume_30d: u64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FeeBreakdown {
    pub total_fee: u64,
    pub lp_fee: u64,
    pub protocol_fee: u64,
    pub burn_fee: u64,
    pub effective_rate_bp: u16,
}

/// Fee calculation errors
#[error_code]
pub enum FeeError {
    #[msg("Fee calculation overflow")]
    FeeCalculationOverflow,
    #[msg("Invalid fee parameters")]
    InvalidFeeParameters,
    #[msg("Fee exceeds maximum cap")]
    FeeExceedsMaxCap,
    #[msg("Insufficient amount for fee")]
    InsufficientAmountForFee,
    #[msg("Invalid user tier data")]
    InvalidUserTierData,
}

/// Main fee calculator with dynamic pricing based on user metrics
pub struct FeeCalculator {
    config: FeeConfig,
}

impl FeeCalculator {
    pub fn new(config: FeeConfig) -> Self {
        Self { config }
    }

    /// Calculate dynamic trading fee based on user tier and volume
    /// Formula: base_fee * volume_multiplier * tier_discount * whale_penalty
    pub fn calculate_trading_fee(
        &self,
        amount: u64,
        user_tier: &UserTier,
        pool_tvl: u64,
    ) -> Result<FeeBreakdown> {
        require!(amount > 0, FeeError::InsufficientAmountForFee);
        require!(pool_tvl > 0, FeeError::InvalidFeeParameters);

        // Calculate base fee rate with dynamic adjustments
        let base_rate = self.get_dynamic_base_rate(amount, pool_tvl)?;
        
        // Apply user tier discount (XP + RP benefits from whitepaper)
        let tier_discount = self.calculate_tier_discount(user_tier)?;
        
        // Apply volume-based discount
        let volume_discount = self.calculate_volume_discount(user_tier.trading_volume_30d)?;
        
        // Apply whale penalty for large trades (anti-whale mechanism)
        let whale_penalty = self.calculate_whale_penalty(amount, pool_tvl)?;
        
        // Calculate effective rate with all modifiers
        let effective_rate_bp = self.apply_fee_modifiers(
            base_rate,
            tier_discount,
            volume_discount,
            whale_penalty,
        )?;

        // Calculate total fee amount
        let total_fee = self.calculate_fee_amount(amount, effective_rate_bp)?;
        
        // Break down fee distribution
        self.breakdown_fees(total_fee, effective_rate_bp)
    }

    /// Calculate liquidity provision fees with staking bonuses
    pub fn calculate_lp_fee(
        &self,
        liquidity_amount: u64,
        user_tier: &UserTier,
        is_initial_lp: bool,
    ) -> Result<FeeBreakdown> {
        require!(liquidity_amount > 0, FeeError::InsufficientAmountForFee);

        let mut base_rate = self.config.lp_fee_bp;

        // First-time LP discount (ecosystem growth incentive)
        if is_initial_lp {
            base_rate = base_rate.saturating_sub(5); // 0.05% discount
        }

        // Staking tier bonus (from whitepaper staking system)
        let staking_discount = self.calculate_staking_discount(user_tier.staked_fin)?;
        
        // XP level bonus
        let xp_discount = self.calculate_xp_discount(user_tier.xp_level)?;

        let effective_rate_bp = base_rate
            .saturating_sub(staking_discount)
            .saturating_sub(xp_discount)
            .max(self.config.min_fee_floor_bp);

        let total_fee = self.calculate_fee_amount(liquidity_amount, effective_rate_bp)?;
        
        self.breakdown_fees(total_fee, effective_rate_bp)
    }

    /// Calculate flash loan fees with progressive pricing
    pub fn calculate_flash_loan_fee(
        &self,
        loan_amount: u64,
        user_tier: &UserTier,
        pool_utilization: u64, // in basis points (0-10000)
    ) -> Result<FeeBreakdown> {
        require!(loan_amount > 0, FeeError::InsufficientAmountForFee);
        require!(pool_utilization <= 10000, FeeError::InvalidFeeParameters);

        let mut base_rate = self.config.flash_loan_fee_bp;

        // Utilization-based pricing (higher utilization = higher fees)
        let utilization_multiplier = self.calculate_utilization_multiplier(pool_utilization)?;
        
        // Size-based penalty for large loans
        let size_penalty = self.calculate_loan_size_penalty(loan_amount)?;
        
        // User tier discount
        let tier_discount = self.calculate_tier_discount(user_tier)?;

        let effective_rate_bp = ((base_rate as u32)
            .saturating_mul(utilization_multiplier as u32) / 100)
            .saturating_add(size_penalty as u32)
            .saturating_sub(tier_discount as u32)
            .min(self.config.max_fee_cap_bp as u32) as u16;

        let total_fee = self.calculate_fee_amount(loan_amount, effective_rate_bp)?;
        
        self.breakdown_fees(total_fee, effective_rate_bp)
    }

    /// Calculate yield farming withdrawal fees with time-based penalties
    pub fn calculate_farm_withdrawal_fee(
        &self,
        withdrawal_amount: u64,
        stake_duration_days: u32,
        user_tier: &UserTier,
    ) -> Result<FeeBreakdown> {
        require!(withdrawal_amount > 0, FeeError::InsufficientAmountForFee);

        // Early withdrawal penalty (decreases over time)
        let early_withdrawal_penalty = self.calculate_early_withdrawal_penalty(stake_duration_days)?;
        
        // Base withdrawal fee
        let base_rate = 25u16; // 0.25% base withdrawal fee
        
        // User tier discount
        let tier_discount = self.calculate_tier_discount(user_tier)?;

        let effective_rate_bp = base_rate
            .saturating_add(early_withdrawal_penalty)
            .saturating_sub(tier_discount)
            .min(self.config.max_fee_cap_bp);

        let total_fee = self.calculate_fee_amount(withdrawal_amount, effective_rate_bp)?;
        
        self.breakdown_fees(total_fee, effective_rate_bp)
    }

    /// Private helper methods

    fn get_dynamic_base_rate(&self, amount: u64, pool_tvl: u64) -> Result<u16> {
        // Dynamic pricing based on trade size relative to pool
        let trade_ratio = (amount as u128)
            .checked_mul(10000)
            .ok_or(FeeError::FeeCalculationOverflow)?
            .checked_div(pool_tvl as u128)
            .ok_or(FeeError::FeeCalculationOverflow)? as u64;

        let base_rate = if trade_ratio < 10 { // < 0.1% of pool
            self.config.base_trading_fee_bp
        } else if trade_ratio < 50 { // 0.1% - 0.5% of pool
            self.config.base_trading_fee_bp.saturating_add(5)
        } else if trade_ratio < 100 { // 0.5% - 1% of pool
            self.config.base_trading_fee_bp.saturating_add(10)
        } else { // > 1% of pool (whale territory)
            self.config.base_trading_fee_bp.saturating_add(20)
        };

        Ok(base_rate)
    }

    fn calculate_tier_discount(&self, user_tier: &UserTier) -> Result<u16> {
        // XP level discount (from whitepaper XP system)
        let xp_discount = match user_tier.xp_level {
            1..=10 => 0,     // Bronze: 0% discount
            11..=25 => 1,    // Silver: 0.01% discount
            26..=50 => 2,    // Gold: 0.02% discount
            51..=75 => 3,    // Platinum: 0.03% discount
            76..=100 => 5,   // Diamond: 0.05% discount
            101.. => 8,      // Mythic: 0.08% discount
            _ => 0,
        };

        // RP tier discount (from whitepaper RP system)
        let rp_discount = match user_tier.rp_tier {
            0 => 0,          // Explorer: 0% discount
            1 => 1,          // Connector: 0.01% discount
            2 => 2,          // Influencer: 0.02% discount
            3 => 3,          // Leader: 0.03% discount
            4 => 5,          // Ambassador: 0.05% discount
            _ => 0,
        };

        Ok(xp_discount.saturating_add(rp_discount))
    }

    fn calculate_volume_discount(&self, volume_30d: u64) -> Result<u16> {
        // Volume-based discount tiers
        let discount = if volume_30d < 1_000_000 { // < 1M $FIN
            0
        } else if volume_30d < 10_000_000 { // 1M - 10M $FIN
            1 // 0.01% discount
        } else if volume_30d < 100_000_000 { // 10M - 100M $FIN
            2 // 0.02% discount
        } else { // > 100M $FIN
            3 // 0.03% discount
        };

        Ok(discount)
    }

    fn calculate_whale_penalty(&self, amount: u64, pool_tvl: u64) -> Result<u16> {
        // Anti-whale mechanism from whitepaper
        let trade_ratio = (amount as u128)
            .checked_mul(10000)
            .ok_or(FeeError::FeeCalculationOverflow)?
            .checked_div(pool_tvl as u128)
            .ok_or(FeeError::FeeCalculationOverflow)? as u64;

        let penalty = if trade_ratio > 500 { // > 5% of pool
            20 // 0.2% penalty
        } else if trade_ratio > 200 { // 2% - 5% of pool
            10 // 0.1% penalty
        } else if trade_ratio > 100 { // 1% - 2% of pool
            5 // 0.05% penalty
        } else {
            0
        };

        Ok(penalty)
    }

    fn calculate_staking_discount(&self, staked_amount: u64) -> Result<u16> {
        // Staking tier discount (from whitepaper staking system)
        let discount = if staked_amount >= 10_000_000 { // 10M+ $FIN
            5 // 0.05% discount
        } else if staked_amount >= 5_000_000 { // 5M - 10M $FIN
            3 // 0.03% discount
        } else if staked_amount >= 1_000_000 { // 1M - 5M $FIN
            2 // 0.02% discount
        } else if staked_amount >= 500_000 { // 500K - 1M $FIN
            1 // 0.01% discount
        } else {
            0
        };

        Ok(discount)
    }

    fn calculate_xp_discount(&self, xp_level: u32) -> Result<u16> {
        // XP-based discount for LP operations
        let discount = match xp_level {
            26..=50 => 1,    // Gold: 0.01% discount
            51..=75 => 2,    // Platinum: 0.02% discount  
            76..=100 => 3,   // Diamond: 0.03% discount
            101.. => 5,      // Mythic: 0.05% discount
            _ => 0,
        };

        Ok(discount)
    }

    fn calculate_utilization_multiplier(&self, utilization_bp: u64) -> Result<u16> {
        // Utilization-based multiplier for flash loans
        let multiplier = if utilization_bp > 9000 { // > 90% utilization
            150 // 1.5x multiplier
        } else if utilization_bp > 8000 { // 80% - 90% utilization
            130 // 1.3x multiplier
        } else if utilization_bp > 7000 { // 70% - 80% utilization
            120 // 1.2x multiplier
        } else if utilization_bp > 5000 { // 50% - 70% utilization
            110 // 1.1x multiplier
        } else {
            100 // 1.0x multiplier
        };

        Ok(multiplier)
    }

    fn calculate_loan_size_penalty(&self, loan_amount: u64) -> Result<u16> {
        // Size-based penalty for large flash loans
        let penalty = if loan_amount > 100_000_000 { // > 100M $FIN
            15 // 0.15% penalty
        } else if loan_amount > 50_000_000 { // 50M - 100M $FIN
            10 // 0.1% penalty
        } else if loan_amount > 10_000_000 { // 10M - 50M $FIN
            5 // 0.05% penalty
        } else {
            0
        };

        Ok(penalty)
    }

    fn calculate_early_withdrawal_penalty(&self, stake_duration_days: u32) -> Result<u16> {
        // Early withdrawal penalty that decreases over time
        let penalty = if stake_duration_days < 7 { // < 1 week
            50 // 0.5% penalty
        } else if stake_duration_days < 30 { // 1 week - 1 month
            25 // 0.25% penalty
        } else if stake_duration_days < 90 { // 1 month - 3 months
            10 // 0.1% penalty
        } else {
            0 // No penalty after 3 months
        };

        Ok(penalty)
    }

    fn apply_fee_modifiers(
        &self,
        base_rate: u16,
        tier_discount: u16,
        volume_discount: u16,
        whale_penalty: u16,
    ) -> Result<u16> {
        let effective_rate = base_rate
            .saturating_sub(tier_discount)
            .saturating_sub(volume_discount)
            .saturating_add(whale_penalty)
            .min(self.config.max_fee_cap_bp)
            .max(self.config.min_fee_floor_bp);

        Ok(effective_rate)
    }

    fn calculate_fee_amount(&self, amount: u64, rate_bp: u16) -> Result<u64> {
        let fee = (amount as u128)
            .checked_mul(rate_bp as u128)
            .ok_or(FeeError::FeeCalculationOverflow)?
            .checked_div(10000)
            .ok_or(FeeError::FeeCalculationOverflow)? as u64;

        require!(fee <= amount, FeeError::FeeExceedsMaxCap);
        Ok(fee)
    }

    fn breakdown_fees(&self, total_fee: u64, effective_rate_bp: u16) -> Result<FeeBreakdown> {
        // Fee distribution based on whitepaper economics
        let lp_portion = 60; // 60% to LPs
        let protocol_portion = 30; // 30% to protocol
        let burn_portion = 10; // 10% burned (deflationary)

        let lp_fee = total_fee
            .checked_mul(lp_portion)
            .ok_or(FeeError::FeeCalculationOverflow)?
            .checked_div(100)
            .ok_or(FeeError::FeeCalculationOverflow)?;

        let protocol_fee = total_fee
            .checked_mul(protocol_portion) 
            .ok_or(FeeError::FeeCalculationOverflow)?
            .checked_div(100)
            .ok_or(FeeError::FeeCalculationOverflow)?;

        let burn_fee = total_fee
            .saturating_sub(lp_fee)
            .saturating_sub(protocol_fee);

        Ok(FeeBreakdown {
            total_fee,
            lp_fee,
            protocol_fee,
            burn_fee,
            effective_rate_bp,
        })
    }
}

/// Utility functions for fee calculations

/// Calculate APY-adjusted fees for yield farming
pub fn calculate_apy_adjusted_fee(
    base_fee: u64,
    current_apy_bp: u16, // in basis points
    target_apy_bp: u16,
) -> Result<u64> {
    if current_apy_bp == 0 {
        return Ok(base_fee);
    }

    let adjustment_factor = if current_apy_bp > target_apy_bp {
        // High APY = higher fees to balance incentives
        (current_apy_bp as u128)
            .checked_div(target_apy_bp as u128)
            .unwrap_or(1)
    } else {
        // Low APY = lower fees to attract users
        (target_apy_bp as u128)
            .checked_div(current_apy_bp as u128)
            .unwrap_or(1)
            .checked_div(2) // Half the adjustment for downward
            .unwrap_or(1)
    };

    let adjusted_fee = (base_fee as u128)
        .checked_mul(adjustment_factor)
        .ok_or(FeeError::FeeCalculationOverflow)?
        .min(base_fee as u128 * 3) // Cap at 3x original fee
        .max(base_fee as u128 / 2) as u64; // Floor at 0.5x original fee

    Ok(adjusted_fee)
}

/// Calculate time-weighted average fee for long-term positions
pub fn calculate_time_weighted_fee(
    fees: &[(u64, u64)], // (fee_amount, timestamp)
    total_duration: u64,
) -> Result<u64> {
    if fees.is_empty() || total_duration == 0 {
        return Ok(0);
    }

    let mut weighted_sum = 0u128;
    let mut prev_timestamp = 0u64;

    for (i, &(fee, timestamp)) in fees.iter().enumerate() {
        let duration = if i == 0 {
            timestamp
        } else {
            timestamp.saturating_sub(prev_timestamp)
        };

        weighted_sum = weighted_sum
            .checked_add((fee as u128).checked_mul(duration as u128)
                .ok_or(FeeError::FeeCalculationOverflow)?)
            .ok_or(FeeError::FeeCalculationOverflow)?;

        prev_timestamp = timestamp;
    }

    let time_weighted_fee = weighted_sum
        .checked_div(total_duration as u128)
        .ok_or(FeeError::FeeCalculationOverflow)? as u64;

    Ok(time_weighted_fee)
}

/// Calculate compound fee effect for multiple operations
pub fn calculate_compound_fee_effect(
    initial_amount: u64,
    fee_rate_bp: u16,
    operation_count: u32,
) -> Result<u64> {
    let mut current_amount = initial_amount as u128;
    let fee_multiplier = 10000u128.saturating_sub(fee_rate_bp as u128);

    for _ in 0..operation_count {
        current_amount = current_amount
            .checked_mul(fee_multiplier)
            .ok_or(FeeError::FeeCalculationOverflow)?
            .checked_div(10000)
            .ok_or(FeeError::FeeCalculationOverflow)?;
    }

    let total_fees = (initial_amount as u128)
        .saturating_sub(current_amount);

    Ok(total_fees as u64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_trading_fee_calculation() {
        let config = FeeConfig::default();
        let calculator = FeeCalculator::new(config);
        
        let user_tier = UserTier {
            xp_level: 15, // Silver tier
            rp_tier: 1,   // Connector
            staked_fin: 500_000,
            trading_volume_30d: 5_000_000,
        };

        let result = calculator.calculate_trading_fee(
            1_000_000, // 1M $FIN trade
            &user_tier,
            100_000_000, // 100M $FIN pool
        ).unwrap();

        assert!(result.total_fee > 0);
        assert!(result.effective_rate_bp <= config.max_fee_cap_bp);
        assert!(result.effective_rate_bp >= config.min_fee_floor_bp);
    }

    #[test]
    fn test_whale_penalty_application() {
        let config = FeeConfig::default();
        let calculator = FeeCalculator::new(config);
        
        let whale_tier = UserTier {
            xp_level: 50,
            rp_tier: 2,
            staked_fin: 1_000_000,
            trading_volume_30d: 50_000_000,
        };

        // Large trade (10% of pool)
        let whale_result = calculator.calculate_trading_fee(
            10_000_000, // 10M $FIN trade
            &whale_tier,
            100_000_000, // 100M $FIN pool
        ).unwrap();

        // Small trade (0.1% of pool)
        let normal_result = calculator.calculate_trading_fee(
            100_000, // 100K $FIN trade
            &whale_tier,
            100_000_000, // 100M $FIN pool  
        ).unwrap();

        // Whale should pay higher effective rate
        assert!(whale_result.effective_rate_bp > normal_result.effective_rate_bp);
    }

    #[test]
    fn test_tier_discount_application() {
        let config = FeeConfig::default();
        let calculator = FeeCalculator::new(config);

        let bronze_tier = UserTier {
            xp_level: 5,
            rp_tier: 0,
            staked_fin: 0,
            trading_volume_30d: 100_000,
        };

        let mythic_tier = UserTier {
            xp_level: 105,
            rp_tier: 4,
            staked_fin: 15_000_000,
            trading_volume_30d: 200_000_000,
        };

        let bronze_result = calculator.calculate_trading_fee(
            1_000_000,
            &bronze_tier,
            100_000_000,
        ).unwrap();

        let mythic_result = calculator.calculate_trading_fee(
            1_000_000,
            &mythic_tier,
            100_000_000,
        ).unwrap();

        // Mythic tier should pay lower fees
        assert!(mythic_result.effective_rate_bp < bronze_result.effective_rate_bp);
    }

    #[test]
    fn test_flash_loan_utilization_pricing() {
        let config = FeeConfig::default();
        let calculator = FeeCalculator::new(config);
        
        let user_tier = UserTier {
            xp_level: 25,
            rp_tier: 1,
            staked_fin: 1_000_000,
            trading_volume_30d: 10_000_000,
        };

        // Low utilization
        let low_util_result = calculator.calculate_flash_loan_fee(
            5_000_000,
            &user_tier,
            3000, // 30% utilization
        ).unwrap();

        // High utilization
        let high_util_result = calculator.calculate_flash_loan_fee(
            5_000_000,
            &user_tier,
            9500, // 95% utilization
        ).unwrap();

        // High utilization should cost more
        assert!(high_util_result.effective_rate_bp > low_util_result.effective_rate_bp);
    }

    #[test]
    fn test_early_withdrawal_penalty() {
        let config = FeeConfig::default();
        let calculator = FeeCalculator::new(config);
        
        let user_tier = UserTier {
            xp_level: 30,
            rp_tier: 2,
            staked_fin: 2_000_000,
            trading_volume_30d: 15_000_000,
        };

        // Early withdrawal (3 days)
        let early_result = calculator.calculate_farm_withdrawal_fee(
            1_000_000,
            3, // 3 days
            &user_tier,
        ).unwrap();

        // Late withdrawal (100 days)
        let late_result = calculator.calculate_farm_withdrawal_fee(
            1_000_000,
            100, // 100 days
            &user_tier,
        ).unwrap();

        // Early withdrawal should cost more
        assert!(early_result.effective_rate_bp > late_result.effective_rate_bp);
    }

    #[test]
    fn test_fee_breakdown_distribution() {
        let config = FeeConfig::default();
        let calculator = FeeCalculator::new(config);
        
        let user_tier = UserTier {
            xp_level: 20,
            rp_tier: 1,
            staked_fin: 750_000,
            trading_volume_30d: 8_000_000,
        };

        let result = calculator.calculate_trading_fee(
            2_000_000,
            &user_tier,
            150_000_000,
        ).unwrap();

        // Check fee breakdown sums to total
        let breakdown_sum = result.lp_fee + result.protocol_fee + result.burn_fee;
        assert_eq!(breakdown_sum, result.total_fee);

        // Check proportions are reasonable
        assert!(result.lp_fee > result.protocol_fee); // LPs get the most
        assert!(result.protocol_fee > result.burn_fee); // Protocol > burn
    }
}
