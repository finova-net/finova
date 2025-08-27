// programs/finova-defi/src/math/curve.rs

// programs/finova-defi/src/math/curve.rs

use anchor_lang::prelude::*;
use anchor_lang::solana_program::log;
use std::collections::VecDeque;

/// Mathematical curve implementations for DeFi operations
/// Supports various AMM curves including constant product, stable swap, and hybrid curves

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum CurveType {
    /// Constant Product (x * y = k) - Uniswap V2 style
    ConstantProduct,
    /// Stable Swap - Curve.fi style for stable assets
    StableSwap,
    /// Hybrid curve - combination of constant product and stable swap
    Hybrid,
    /// Concentrated liquidity - Uniswap V3 style
    ConcentratedLiquidity,
    /// Custom Finova curve optimized for $FIN ecosystem
    FinovaCurve,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct CurveParameters {
    /// Amplification coefficient for stable swap (A parameter)
    pub amplification: u64,
    /// Fee rate in basis points (e.g., 30 = 0.3%)
    pub fee_rate: u64,
    /// Admin fee rate in basis points
    pub admin_fee_rate: u64,
    /// Price range for concentrated liquidity
    pub price_range: Option<PriceRange>,
    /// Custom parameters for Finova curve
    pub finova_params: Option<FinovaCurveParams>,
    /// Curve type identifier
    pub curve_type: CurveType,
    /// Precision multiplier for calculations
    pub precision: u128,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct PriceRange {
    /// Lower price bound
    pub lower_price: u128,
    /// Upper price bound  
    pub upper_price: u128,
    /// Current tick
    pub current_tick: i32,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct FinovaCurveParams {
    /// XP multiplier effect on trading
    pub xp_multiplier: u64,
    /// RP network effect on fees
    pub rp_multiplier: u64,
    /// Mining boost factor
    pub mining_boost: u64,
    /// Quality score impact
    pub quality_impact: u64,
    /// Dynamic fee adjustment
    pub dynamic_fee_enabled: bool,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct SwapResult {
    /// Amount of token out
    pub amount_out: u64,
    /// Fee amount charged
    pub fee_amount: u64,
    /// Admin fee amount
    pub admin_fee_amount: u64,
    /// Price impact in basis points
    pub price_impact: u64,
    /// New reserves after swap
    pub new_reserve_a: u64,
    pub new_reserve_b: u64,
    /// Slippage protection passed
    pub slippage_ok: bool,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct LiquidityResult {
    /// LP tokens minted/burned
    pub lp_amount: u64,
    /// Actual token amounts used
    pub token_a_amount: u64,
    pub token_b_amount: u64,
    /// Price impact
    pub price_impact: u64,
    /// Success flag
    pub success: bool,
}

/// Historical price data point for TWAP calculations
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct PricePoint {
    /// Timestamp of the price observation
    pub timestamp: i64,
    /// Price at this point (token_a / token_b)
    pub price: u128,
    /// Cumulative price for TWAP
    pub cumulative_price: u128,
    /// Volume at this point
    pub volume: u64,
}

/// Price oracle with TWAP functionality
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct PriceOracle {
    /// Historical price points (circular buffer)
    pub price_history: VecDeque<PricePoint>,
    /// Maximum history length
    pub max_history: usize,
    /// Last update timestamp
    pub last_update: i64,
    /// Current TWAP value
    pub twap: u128,
    /// TWAP period in seconds
    pub twap_period: i64,
}

impl Default for CurveParameters {
    fn default() -> Self {
        Self {
            amplification: 100,
            fee_rate: 30, // 0.3%
            admin_fee_rate: 5, // 0.05%
            price_range: None,
            finova_params: None,
            curve_type: CurveType::ConstantProduct,
            precision: 1_000_000_000_000_000_000u128, // 1e18
        }
    }
}

impl Default for PriceOracle {
    fn default() -> Self {
        Self {
            price_history: VecDeque::with_capacity(100),
            max_history: 100,
            last_update: 0,
            twap: 0,
            twap_period: 3600, // 1 hour
        }
    }
}

/// Main curve implementation with various AMM algorithms
pub struct Curve;

impl Curve {
    /// Constants for mathematical calculations
    const PRECISION: u128 = 1_000_000_000_000_000_000u128; // 1e18
    const MAX_ITERATIONS: u8 = 255;
    const CONVERGENCE_THRESHOLD: u128 = 1;
    const BASIS_POINTS: u64 = 10_000;

    /// Calculate swap output using specified curve
    pub fn calculate_swap(
        curve_params: &CurveParameters,
        reserve_a: u64,
        reserve_b: u64,
        amount_in: u64,
        a_to_b: bool,
        user_xp_level: u64,
        user_rp_tier: u64,
        slippage_tolerance: u64,
    ) -> Result<SwapResult> {
        require!(amount_in > 0, crate::errors::DeFiError::InvalidAmount);
        require!(reserve_a > 0 && reserve_b > 0, crate::errors::DeFiError::InsufficientLiquidity);

        match curve_params.curve_type {
            CurveType::ConstantProduct => {
                Self::constant_product_swap(
                    curve_params,
                    reserve_a,
                    reserve_b,
                    amount_in,
                    a_to_b,
                    slippage_tolerance,
                )
            }
            CurveType::StableSwap => {
                Self::stable_swap(
                    curve_params,
                    reserve_a,
                    reserve_b,
                    amount_in,
                    a_to_b,
                    slippage_tolerance,
                )
            }
            CurveType::Hybrid => {
                Self::hybrid_swap(
                    curve_params,
                    reserve_a,
                    reserve_b,
                    amount_in,
                    a_to_b,
                    slippage_tolerance,
                )
            }
            CurveType::ConcentratedLiquidity => {
                Self::concentrated_liquidity_swap(
                    curve_params,
                    reserve_a,
                    reserve_b,
                    amount_in,
                    a_to_b,
                    slippage_tolerance,
                )
            }
            CurveType::FinovaCurve => {
                Self::finova_curve_swap(
                    curve_params,
                    reserve_a,
                    reserve_b,
                    amount_in,
                    a_to_b,
                    user_xp_level,
                    user_rp_tier,
                    slippage_tolerance,
                )
            }
        }
    }

    /// Constant Product AMM (x * y = k)
    fn constant_product_swap(
        curve_params: &CurveParameters,
        reserve_a: u64,
        reserve_b: u64,
        amount_in: u64,
        a_to_b: bool,
        slippage_tolerance: u64,
    ) -> Result<SwapResult> {
        let precision = curve_params.precision;
        
        // Convert to high precision
        let reserve_a_hp = (reserve_a as u128) * precision;
        let reserve_b_hp = (reserve_b as u128) * precision;
        let amount_in_hp = (amount_in as u128) * precision;
        
        // Calculate k = x * y
        let k = reserve_a_hp
            .checked_mul(reserve_b_hp)
            .ok_or(crate::errors::DeFiError::MathOverflow)?;
        
        // Calculate fee
        let fee_rate = curve_params.fee_rate as u128;
        let fee_amount_hp = amount_in_hp
            .checked_mul(fee_rate)
            .ok_or(crate::errors::DeFiError::MathOverflow)?
            .checked_div(Self::BASIS_POINTS as u128)
            .ok_or(crate::errors::DeFiError::MathOverflow)?;
        
        let amount_in_after_fee = amount_in_hp
            .checked_sub(fee_amount_hp)
            .ok_or(crate::errors::DeFiError::MathOverflow)?;
        
        // Calculate output amount
        let (new_reserve_in, new_reserve_out, amount_out_hp) = if a_to_b {
            let new_reserve_a = reserve_a_hp
                .checked_add(amount_in_after_fee)
                .ok_or(crate::errors::DeFiError::MathOverflow)?;
            
            let new_reserve_b = k
                .checked_div(new_reserve_a)
                .ok_or(crate::errors::DeFiError::MathOverflow)?;
            
            let amount_out = reserve_b_hp
                .checked_sub(new_reserve_b)
                .ok_or(crate::errors::DeFiError::InsufficientLiquidity)?;
            
            (new_reserve_a, new_reserve_b, amount_out)
        } else {
            let new_reserve_b = reserve_b_hp
                .checked_add(amount_in_after_fee)
                .ok_or(crate::errors::DeFiError::MathOverflow)?;
            
            let new_reserve_a = k
                .checked_div(new_reserve_b)
                .ok_or(crate::errors::DeFiError::MathOverflow)?;
            
            let amount_out = reserve_a_hp
                .checked_sub(new_reserve_a)
                .ok_or(crate::errors::DeFiError::InsufficientLiquidity)?;
            
            (new_reserve_a, new_reserve_b, amount_out)
        };
        
        // Calculate price impact
        let price_before = if a_to_b {
            reserve_b_hp
                .checked_mul(precision)
                .ok_or(crate::errors::DeFiError::MathOverflow)?
                .checked_div(reserve_a_hp)
                .ok_or(crate::errors::DeFiError::MathOverflow)?
        } else {
            reserve_a_hp
                .checked_mul(precision)
                .ok_or(crate::errors::DeFiError::MathOverflow)?
                .checked_div(reserve_b_hp)
                .ok_or(crate::errors::DeFiError::MathOverflow)?
        };
        
        let price_after = if a_to_b {
            new_reserve_out
                .checked_mul(precision)
                .ok_or(crate::errors::DeFiError::MathOverflow)?
                .checked_div(new_reserve_in)
                .ok_or(crate::errors::DeFiError::MathOverflow)?
        } else {
            new_reserve_in
                .checked_mul(precision)
                .ok_or(crate::errors::DeFiError::MathOverflow)?
                .checked_div(new_reserve_out)
                .ok_or(crate::errors::DeFiError::MathOverflow)?
        };
        
        let price_impact = if price_after > price_before {
            (price_after - price_before)
                .checked_mul(Self::BASIS_POINTS as u128)
                .ok_or(crate::errors::DeFiError::MathOverflow)?
                .checked_div(price_before)
                .ok_or(crate::errors::DeFiError::MathOverflow)?
        } else {
            (price_before - price_after)
                .checked_mul(Self::BASIS_POINTS as u128)
                .ok_or(crate::errors::DeFiError::MathOverflow)?
                .checked_div(price_before)
                .ok_or(crate::errors::DeFiError::MathOverflow)?
        };
        
        // Check slippage
        let slippage_ok = price_impact <= slippage_tolerance as u128;
        
        // Calculate admin fee
        let admin_fee_rate = curve_params.admin_fee_rate as u128;
        let admin_fee_amount_hp = fee_amount_hp
            .checked_mul(admin_fee_rate)
            .ok_or(crate::errors::DeFiError::MathOverflow)?
            .checked_div(Self::BASIS_POINTS as u128)
            .ok_or(crate::errors::DeFiError::MathOverflow)?;
        
        // Convert back to normal precision
        let amount_out = (amount_out_hp / precision) as u64;
        let fee_amount = (fee_amount_hp / precision) as u64;
        let admin_fee_amount = (admin_fee_amount_hp / precision) as u64;
        let new_reserve_a_final = (new_reserve_in / precision) as u64;
        let new_reserve_b_final = (new_reserve_out / precision) as u64;
        
        Ok(SwapResult {
            amount_out,
            fee_amount,
            admin_fee_amount,
            price_impact: price_impact as u64,
            new_reserve_a: if a_to_b { new_reserve_a_final } else { new_reserve_a_final },
            new_reserve_b: if a_to_b { new_reserve_b_final } else { new_reserve_b_final },
            slippage_ok,
        })
    }

    /// Stable Swap AMM (Curve.fi style)
    fn stable_swap(
        curve_params: &CurveParameters,
        reserve_a: u64,
        reserve_b: u64,
        amount_in: u64,
        a_to_b: bool,
        slippage_tolerance: u64,
    ) -> Result<SwapResult> {
        let precision = curve_params.precision;
        let amp = curve_params.amplification;
        
        // Convert to arrays for easier calculation
        let mut balances = [reserve_a as u128, reserve_b as u128];
        let n_coins = 2u128;
        
        // Calculate D (invariant)
        let d = Self::calculate_d(&balances, amp, precision)?;
        
        // Apply input
        let input_index = if a_to_b { 0 } else { 1 };
        let output_index = if a_to_b { 1 } else { 0 };
        
        // Calculate fee
        let fee_rate = curve_params.fee_rate as u128;
        let fee_amount = (amount_in as u128)
            .checked_mul(fee_rate)
            .ok_or(crate::errors::DeFiError::MathOverflow)?
            .checked_div(Self::BASIS_POINTS as u128)
            .ok_or(crate::errors::DeFiError::MathOverflow)?;
        
        let amount_in_after_fee = (amount_in as u128)
            .checked_sub(fee_amount)
            .ok_or(crate::errors::DeFiError::MathOverflow)?;
        
        balances[input_index] = balances[input_index]
            .checked_add(amount_in_after_fee)
            .ok_or(crate::errors::DeFiError::MathOverflow)?;
        
        // Calculate new output balance
        let new_output_balance = Self::calculate_y(
            output_index,
            input_index,
            balances[input_index],
            &balances,
            amp,
            d,
            precision,
        )?;
        
        let amount_out = balances[output_index]
            .checked_sub(new_output_balance)
            .ok_or(crate::errors::DeFiError::InsufficientLiquidity)?;
        
        // Update balances
        balances[output_index] = new_output_balance;
        
        // Calculate price impact
        let price_impact = Self::calculate_price_impact(
            reserve_a as u128,
            reserve_b as u128,
            balances[0],
            balances[1],
            precision,
        )?;
        
        let slippage_ok = price_impact <= slippage_tolerance as u128;
        
        // Calculate admin fee
        let admin_fee_rate = curve_params.admin_fee_rate as u128;
        let admin_fee_amount = fee_amount
            .checked_mul(admin_fee_rate)
            .ok_or(crate::errors::DeFiError::MathOverflow)?
            .checked_div(Self::BASIS_POINTS as u128)
            .ok_or(crate::errors::DeFiError::MathOverflow)?;
        
        Ok(SwapResult {
            amount_out: amount_out as u64,
            fee_amount: fee_amount as u64,
            admin_fee_amount: admin_fee_amount as u64,
            price_impact: price_impact as u64,
            new_reserve_a: balances[0] as u64,
            new_reserve_b: balances[1] as u64,
            slippage_ok,
        })
    }

    /// Finova custom curve with XP/RP integration
    fn finova_curve_swap(
        curve_params: &CurveParameters,
        reserve_a: u64,
        reserve_b: u64,
        amount_in: u64,
        a_to_b: bool,
        user_xp_level: u64,
        user_rp_tier: u64,
        slippage_tolerance: u64,
    ) -> Result<SwapResult> {
        let finova_params = curve_params.finova_params.as_ref()
            .ok_or(crate::errors::DeFiError::InvalidCurveParameters)?;
        
        // Start with constant product base
        let mut result = Self::constant_product_swap(
            curve_params,
            reserve_a,
            reserve_b,
            amount_in,
            a_to_b,
            slippage_tolerance,
        )?;
        
        // Apply XP level multiplier (better output for higher XP)
        let xp_multiplier = Self::calculate_xp_multiplier(user_xp_level, finova_params.xp_multiplier);
        result.amount_out = ((result.amount_out as u128)
            .checked_mul(xp_multiplier)
            .ok_or(crate::errors::DeFiError::MathOverflow)?
            .checked_div(Self::BASIS_POINTS as u128)
            .ok_or(crate::errors::DeFiError::MathOverflow)?) as u64;
        
        // Apply RP tier fee reduction
        let rp_fee_reduction = Self::calculate_rp_fee_reduction(user_rp_tier, finova_params.rp_multiplier);
        result.fee_amount = ((result.fee_amount as u128)
            .checked_mul(Self::BASIS_POINTS as u128 - rp_fee_reduction)
            .ok_or(crate::errors::DeFiError::MathOverflow)?
            .checked_div(Self::BASIS_POINTS as u128)
            .ok_or(crate::errors::DeFiError::MathOverflow)?) as u64;
        
        // Dynamic fee adjustment based on network conditions
        if finova_params.dynamic_fee_enabled {
            let dynamic_adjustment = Self::calculate_dynamic_fee_adjustment(
                reserve_a,
                reserve_b,
                amount_in,
                finova_params.quality_impact,
            )?;
            
            result.fee_amount = ((result.fee_amount as u128)
                .checked_mul(dynamic_adjustment)
                .ok_or(crate::errors::DeFiError::MathOverflow)?
                .checked_div(Self::BASIS_POINTS as u128)
                .ok_or(crate::errors::DeFiError::MathOverflow)?) as u64;
        }
        
        msg!("Finova curve swap completed with XP level: {}, RP tier: {}", user_xp_level, user_rp_tier);
        
        Ok(result)
    }

    /// Hybrid curve combining constant product and stable swap
    fn hybrid_swap(
        curve_params: &CurveParameters,
        reserve_a: u64,
        reserve_b: u64,
        amount_in: u64,
        a_to_b: bool,
        slippage_tolerance: u64,
    ) -> Result<SwapResult> {
        // Calculate both curves
        let cp_result = Self::constant_product_swap(
            curve_params,
            reserve_a,
            reserve_b,
            amount_in,
            a_to_b,
            slippage_tolerance,
        )?;
        
        let stable_result = Self::stable_swap(
            curve_params,
            reserve_a,
            reserve_b,
            amount_in,
            a_to_b,
            slippage_tolerance,
        )?;
        
        // Blend results based on reserve ratio (more stable when balanced)
        let ratio_a = (reserve_a as u128 * Self::PRECISION) / (reserve_a as u128 + reserve_b as u128);
        let ratio_b = Self::PRECISION - ratio_a;
        
        // Calculate balance factor (0 = perfectly balanced, 1 = completely imbalanced)
        let balance_factor = if ratio_a > ratio_b {
            (ratio_a - ratio_b) * 2
        } else {
            (ratio_b - ratio_a) * 2
        };
        
        // Blend: more constant product when imbalanced, more stable when balanced
        let cp_weight = balance_factor;
        let stable_weight = Self::PRECISION - balance_factor;
        
        let blended_amount_out = ((cp_result.amount_out as u128 * cp_weight + 
                                   stable_result.amount_out as u128 * stable_weight) / Self::PRECISION) as u64;
        
        let blended_fee = ((cp_result.fee_amount as u128 * cp_weight + 
                           stable_result.fee_amount as u128 * stable_weight) / Self::PRECISION) as u64;
        
        Ok(SwapResult {
            amount_out: blended_amount_out,
            fee_amount: blended_fee,
            admin_fee_amount: cp_result.admin_fee_amount,
            price_impact: cp_result.price_impact,
            new_reserve_a: cp_result.new_reserve_a,
            new_reserve_b: cp_result.new_reserve_b,
            slippage_ok: cp_result.slippage_ok && stable_result.slippage_ok,
        })
    }

    /// Concentrated liquidity swap (Uniswap V3 style)
    fn concentrated_liquidity_swap(
        curve_params: &CurveParameters,
        reserve_a: u64,
        reserve_b: u64,
        amount_in: u64,
        a_to_b: bool,
        slippage_tolerance: u64,
    ) -> Result<SwapResult> {
        let price_range = curve_params.price_range.as_ref()
            .ok_or(crate::errors::DeFiError::InvalidCurveParameters)?;
        
        // Current price
        let current_price = (reserve_b as u128 * Self::PRECISION) / reserve_a as u128;
        
        // Check if within range
        require!(
            current_price >= price_range.lower_price && current_price <= price_range.upper_price,
            crate::errors::DeFiError::PriceOutOfRange
        );
        
        // Calculate virtual reserves based on price range
        let sqrt_lower = Self::sqrt(price_range.lower_price)?;
        let sqrt_upper = Self::sqrt(price_range.upper_price)?;
        let sqrt_current = Self::sqrt(current_price)?;
        
        // Virtual liquidity calculation
        let virtual_x = if sqrt_current <= sqrt_lower {
            0
        } else if sqrt_current >= sqrt_upper {
            reserve_a as u128
        } else {
            (reserve_a as u128 * (sqrt_upper - sqrt_current)) / (sqrt_upper - sqrt_lower)
        };
        
        let virtual_y = if sqrt_current <= sqrt_lower {
            reserve_b as u128
        } else if sqrt_current >= sqrt_upper {
            0
        } else {
            (reserve_b as u128 * (sqrt_current - sqrt_lower)) / (sqrt_upper - sqrt_lower)
        };
        
        // Apply constant product formula to virtual reserves
        let mut modified_params = curve_params.clone();
        modified_params.curve_type = CurveType::ConstantProduct;
        
        Self::constant_product_swap(
            &modified_params,
            virtual_x as u64,
            virtual_y as u64,
            amount_in,
            a_to_b,
            slippage_tolerance,
        )
    }

    /// Calculate liquidity provision
    pub fn calculate_add_liquidity(
        curve_params: &CurveParameters,
        reserve_a: u64,
        reserve_b: u64,
        amount_a: u64,
        amount_b: u64,
        total_supply: u64,
    ) -> Result<LiquidityResult> {
        require!(amount_a > 0 || amount_b > 0, crate::errors::DeFiError::InvalidAmount);
        require!(reserve_a > 0 && reserve_b > 0, crate::errors::DeFiError::InsufficientLiquidity);
        
        let precision = curve_params.precision;
        
        // Calculate optimal amounts based on current ratio
        let ratio = (reserve_b as u128 * precision) / reserve_a as u128;
        
        let (actual_amount_a, actual_amount_b) = if amount_a == 0 {
            // Only B provided, calculate required A
            let required_a = ((amount_b as u128 * precision) / ratio) as u64;
            (required_a, amount_b)
        } else if amount_b == 0 {
            // Only A provided, calculate required B
            let required_b = ((amount_a as u128 * ratio) / precision) as u64;
            (amount_a, required_b)
        } else {
            // Both provided, use the limiting one
            let required_b_for_a = ((amount_a as u128 * ratio) / precision) as u64;
            let required_a_for_b = ((amount_b as u128 * precision) / ratio) as u64;
            
            if required_b_for_a <= amount_b {
                (amount_a, required_b_for_a)
            } else {
                (required_a_for_b, amount_b)
            }
        };
        
        // Calculate LP tokens to mint
        let lp_amount = if total_supply == 0 {
            // Initial liquidity
            Self::sqrt((actual_amount_a as u128) * (actual_amount_b as u128))? as u64
        } else {
            // Proportional to existing pool
            let share_a = (actual_amount_a as u128 * total_supply as u128) / reserve_a as u128;
            let share_b = (actual_amount_b as u128 * total_supply as u128) / reserve_b as u128;
            std::cmp::min(share_a, share_b) as u64
        };
        
        // Calculate price impact
        let old_k = (reserve_a as u128) * (reserve_b as u128);
        let new_k = ((reserve_a + actual_amount_a) as u128) * ((reserve_b + actual_amount_b) as u128);
        let price_impact = if new_k > old_k {
            ((new_k - old_k) * Self::BASIS_POINTS as u128) / old_k
        } else {
            0
        };
        
        Ok(LiquidityResult {
            lp_amount,
            token_a_amount: actual_amount_a,
            token_b_amount: actual_amount_b,
            price_impact: price_impact as u64,
            success: true,
        })
    }

    /// Calculate liquidity removal
    pub fn calculate_remove_liquidity(
        curve_params: &CurveParameters,
        reserve_a: u64,
        reserve_b: u64,
        lp_amount: u64,
        total_supply: u64,
    ) -> Result<LiquidityResult> {
        require!(lp_amount > 0, crate::errors::DeFiError::InvalidAmount);
        require!(lp_amount <= total_supply, crate::errors::DeFiError::InsufficientLiquidity);
        require!(reserve_a > 0 && reserve_b > 0, crate::errors::DeFiError::InsufficientLiquidity);
        
        // Calculate proportional amounts
        let amount_a = ((lp_amount as u128 * reserve_a as u128) / total_supply as u128) as u64;
        let amount_b = ((lp_amount as u128 * reserve_b as u128) / total_supply as u128) as u64;
        
        // Calculate price impact (should be minimal for proportional removal)
        let price_impact = ((lp_amount as u128 * Self::BASIS_POINTS as u128) / total_supply as u128) as u64;
        
        Ok(LiquidityResult {
            lp_amount,
            token_a_amount: amount_a,
            token_b_amount: amount_b,
            price_impact,
            success: true,
        })
    }

    /// Helper functions for complex calculations

    /// Calculate D invariant for stable swap
    fn calculate_d(balances: &[u128], amp: u64, precision: u128) -> Result<u128> {
        let n_coins = balances.len() as u128;
        let mut sum_balances = 0u128;
        
        for balance in balances {
            sum_balances = sum_balances
                .checked_add(*balance)
                .ok_or(crate::errors::DeFiError::MathOverflow)?;
        }
        
        if sum_balances == 0 {
            return Ok(0);
        }
        
        let mut d = sum_balances;
        let ann = (amp as u128)
            .checked_mul(n_coins)
            .ok_or(crate::errors::DeFiError::MathOverflow)?;
        
        for _ in 0..Self::MAX_ITERATIONS {
            let mut d_product = d;
            for balance in balances {
                d_product = d_product
                    .checked_mul(d)
                    .ok_or(crate::errors::DeFiError::MathOverflow)?
                    .checked_div(balance.checked_mul(n_coins).ok_or(crate::errors::DeFiError::MathOverflow)?)
                    .ok_or(crate::errors::DeFiError::MathOverflow)?;
            }
            
            let d_prev = d;
            
            // d = (ann * sum_balances + d_product * n_coins) * d / ((ann - 1) * d + (n_coins + 1) * d_product)
            let numerator = ann
                .checked_mul(sum_balances)
                .ok_or(crate::errors::DeFiError::MathOverflow)?
                .checked_add(d_product.checked_mul(n_coins).ok_or(crate::errors::DeFiError::MathOverflow)?)
                .ok_or(crate::errors::DeFiError::MathOverflow)?
                .checked_mul(d)
                .ok_or(crate::errors::DeFiError::MathOverflow)?;
            
            let denominator = ann
                .checked_sub(1)
                .ok_or(crate::errors::DeFiError::MathOverflow)?
                .checked_mul(d)
                .ok_or(crate::errors::DeFiError::MathOverflow)?
                .checked_add(
                    n_coins
                        .checked_add(1)
                        .ok_or(crate::errors::DeFiError::MathOverflow)?
                        .checked_mul(d_product)
                        .ok_or(crate::errors::DeFiError::MathOverflow)?
                )
                .ok_or(crate::errors::DeFiError::MathOverflow)?;
            
            d = numerator
                .checked_div(denominator)
                .ok_or(crate::errors::DeFiError::MathOverflow)?;
            
            if d > d_prev {
                if d - d_prev <= Self::CONVERGENCE_THRESHOLD {
                    break;
                }
            } else if d_prev - d <= Self::CONVERGENCE_THRESHOLD {
                break;
            }
        }
        
        Ok(d)
    }

    /// Calculate Y for stable swap
    fn calculate_y(
        i: usize,
        j: usize,
        x: u128,
        xp: &[u128],
        amp: u64,
        d: u128,
        precision: u128,
    ) -> Result<u128> {
        require!(i != j, crate::errors::DeFiError::InvalidTokenIndex);
        require!(i < xp.len() && j < xp.len(), crate::errors::DeFiError::InvalidTokenIndex);
        
        let n_coins = xp.len() as u128;
        let ann = (amp as u128).checked_mul(n_coins).ok_or(crate::errors::DeFiError::MathOverflow)?;
        let mut c = d;
        let mut s = 0u128;
        
        for (k, balance) in xp.iter().enumerate() {
            if k == i {
                s = s.checked_add(x).ok_or(crate::errors::DeFiError::MathOverflow)?;
                c = c.checked_mul(d).ok_or(crate::errors::DeFiError::MathOverflow)?
                    .checked_div(x.checked_mul(n_coins).ok_or(crate::errors::DeFiError::MathOverflow)?)
                    .ok_or(crate::errors::DeFiError::MathOverflow)?;
            } else if k != j {
                s = s.checked_add(*balance).ok_or(crate::errors::DeFiError::MathOverflow)?;
                c = c.checked_mul(d).ok_or(crate::errors::DeFiError::MathOverflow)?
                    .checked_div(balance.checked_mul(n_coins).ok_or(crate::errors::DeFiError::MathOverflow)?)
                    .ok_or(crate::errors::DeFiError::MathOverflow)?;
            }
        }
        
        c = c.checked_mul(d).ok_or(crate::errors::DeFiError::MathOverflow)?
            .checked_div(ann.checked_mul(n_coins).ok_or(crate::errors::DeFiError::MathOverflow)?)
            .ok_or(crate::errors::DeFiError::MathOverflow)?;
        
        let b = s.checked_add(d.checked_div(ann).ok_or(crate::errors::DeFiError::MathOverflow)?)
            .ok_or(crate::errors::DeFiError::MathOverflow)?;
        
        let mut y = d;
        
        for _ in 0..Self::MAX_ITERATIONS {
            let y_prev = y;
            
            let y_numerator = y.checked_mul(y).ok_or(crate::errors::DeFiError::MathOverflow)?
                .checked_add(c)
                .ok_or(crate::errors::DeFiError::MathOverflow)?;
            
            let y_denominator = y.checked_mul(2).ok_or(crate::errors::DeFiError::MathOverflow)?
                .checked_add(b)
                .ok_or(crate::errors::DeFiError::MathOverflow)?
                .checked_sub(d)
                .ok_or(crate::errors::DeFiError::MathOverflow)?;
            
            y = y_numerator.checked_div(y_denominator).ok_or(crate::errors::DeFiError::MathOverflow)?;
            
            if y > y_prev {
                if y - y_prev <= Self::CONVERGENCE_THRESHOLD {
                    break;
                }
            } else if y_prev - y <= Self::CONVERGENCE_THRESHOLD {
                break;
            }
        }
        
        Ok(y)
    }

    /// Calculate price impact
    fn calculate_price_impact(
        old_a: u128,
        old_b: u128,
        new_a: u128,
        new_b: u128,
        precision: u128,
    ) -> Result<u128> {
        let old_price = old_b.checked_mul(precision).ok_or(crate::errors::DeFiError::MathOverflow)?
            .checked_div(old_a).ok_or(crate::errors::DeFiError::MathOverflow)?;
        
        let new_price = new_b.checked_mul(precision).ok_or(crate::errors::DeFiError::MathOverflow)?
            .checked_div(new_a).ok_or(crate::errors::DeFiError::MathOverflow)?;
        
        let price_impact = if new_price > old_price {
            (new_price - old_price).checked_mul(Self::BASIS_POINTS as u128)
                .ok_or(crate::errors::DeFiError::MathOverflow)?
                .checked_div(old_price)
                .ok_or(crate::errors::DeFiError::MathOverflow)?
        } else {
            (old_price - new_price).checked_mul(Self::BASIS_POINTS as u128)
                .ok_or(crate::errors::DeFiError::MathOverflow)?
                .checked_div(old_price)
                .ok_or(crate::errors::DeFiError::MathOverflow)?
        };
        
        Ok(price_impact)
    }

    /// Calculate XP multiplier for Finova curve
    fn calculate_xp_multiplier(xp_level: u64, base_multiplier: u64) -> u128 {
        // XP levels 1-100, multiplier increases logarithmically
        let level_bonus = (xp_level as f64).ln() * (base_multiplier as f64) / 100.0;
        let total_multiplier = Self::BASIS_POINTS as f64 + level_bonus.max(0.0).min(5000.0); // Cap at 50% bonus
        
        total_multiplier as u128
    }

    /// Calculate RP fee reduction
    fn calculate_rp_fee_reduction(rp_tier: u64, base_reduction: u64) -> u128 {
        // RP tiers 0-5, fee reduction increases linearly
        let tier_reduction = rp_tier * base_reduction;
        tier_reduction.min(5000) as u128 // Cap at 50% reduction
    }

    /// Calculate dynamic fee adjustment
    fn calculate_dynamic_fee_adjustment(
        reserve_a: u64,
        reserve_b: u64,
        amount_in: u64,
        quality_impact: u64,
    ) -> Result<u128> {
        // Calculate pool imbalance
        let total_reserves = reserve_a as u128 + reserve_b as u128;
        let imbalance = if reserve_a > reserve_b {
            ((reserve_a - reserve_b) as u128 * Self::BASIS_POINTS as u128) / total_reserves
        } else {
            ((reserve_b - reserve_a) as u128 * Self::BASIS_POINTS as u128) / total_reserves
        };
        
        // Calculate volume impact
        let volume_impact = ((amount_in as u128 * Self::BASIS_POINTS as u128) / total_reserves).min(1000); // Cap at 10%
        
        // Base adjustment
        let mut adjustment = Self::BASIS_POINTS as u128;
        
        // Increase fee for high imbalance
        if imbalance > 2000 { // > 20% imbalance
            adjustment += (imbalance - 2000) / 10; // Up to 10% fee increase
        }
        
        // Increase fee for large volumes
        adjustment += volume_impact / 10;
        
        // Apply quality impact
        let quality_adjustment = (quality_impact as u128 * imbalance) / Self::BASIS_POINTS as u128;
        adjustment += quality_adjustment;
        
        Ok(adjustment.min(15000)) // Cap total adjustment at 150%
    }

    /// Square root calculation using Newton's method
    fn sqrt(y: u128) -> Result<u128> {
        if y == 0 {
            return Ok(0);
        }
        
        let mut z = y;
        let mut x = y / 2 + 1;
        
        for _ in 0..Self::MAX_ITERATIONS {
            if x >= z {
                break;
            }
            z = x;
            x = (y / x + x) / 2;
        }
        
        Ok(z)
    }

    /// Get current price from reserves
    pub fn get_price(reserve_a: u64, reserve_b: u64, precision: u128) -> Result<u128> {
        require!(reserve_a > 0 && reserve_b > 0, crate::errors::DeFiError::InsufficientLiquidity);
        
        let price = (reserve_b as u128)
            .checked_mul(precision)
            .ok_or(crate::errors::DeFiError::MathOverflow)?
            .checked_div(reserve_a as u128)
            .ok_or(crate::errors::DeFiError::MathOverflow)?;
        
        Ok(price)
    }

    /// Calculate TWAP (Time Weighted Average Price)
    pub fn calculate_twap(
        oracle: &PriceOracle,
        period: i64,
        current_time: i64,
    ) -> Result<u128> {
        if oracle.price_history.is_empty() {
            return Ok(0);
        }
        
        let start_time = current_time - period;
        let mut weighted_sum = 0u128;
        let mut total_weight = 0u128;
        
        for price_point in oracle.price_history.iter() {
            if price_point.timestamp >= start_time {
                let weight = (current_time - price_point.timestamp).max(1) as u128;
                weighted_sum = weighted_sum
                    .checked_add(price_point.price.checked_mul(weight).ok_or(crate::errors::DeFiError::MathOverflow)?)
                    .ok_or(crate::errors::DeFiError::MathOverflow)?;
                total_weight = total_weight
                    .checked_add(weight)
                    .ok_or(crate::errors::DeFiError::MathOverflow)?;
            }
        }
        
        if total_weight == 0 {
            return Ok(oracle.price_history.back().unwrap().price);
        }
        
        let twap = weighted_sum
            .checked_div(total_weight)
            .ok_or(crate::errors::DeFiError::MathOverflow)?;
        
        Ok(twap)
    }

    /// Update price oracle with new price data
    pub fn update_price_oracle(
        oracle: &mut PriceOracle,
        new_price: u128,
        volume: u64,
        timestamp: i64,
    ) -> Result<()> {
        // Calculate cumulative price
        let time_elapsed = if oracle.last_update > 0 {
            timestamp - oracle.last_update
        } else {
            0
        };
        
        let cumulative_price = if let Some(last_point) = oracle.price_history.back() {
            last_point.cumulative_price
                .checked_add(last_point.price.checked_mul(time_elapsed as u128).ok_or(crate::errors::DeFiError::MathOverflow)?)
                .ok_or(crate::errors::DeFiError::MathOverflow)?
        } else {
            0
        };
        
        let new_point = PricePoint {
            timestamp,
            price: new_price,
            cumulative_price,
            volume,
        };
        
        // Add new point
        oracle.price_history.push_back(new_point);
        
        // Remove old points if exceeding max history
        while oracle.price_history.len() > oracle.max_history {
            oracle.price_history.pop_front();
        }
        
        // Update TWAP
        oracle.twap = Self::calculate_twap(oracle, oracle.twap_period, timestamp)?;
        oracle.last_update = timestamp;
        
        msg!("Price oracle updated: price={}, twap={}, timestamp={}", new_price, oracle.twap, timestamp);
        
        Ok(())
    }

    /// Validate curve parameters
    pub fn validate_curve_parameters(params: &CurveParameters) -> Result<()> {
        // Validate amplification coefficient
        require!(
            params.amplification >= 1 && params.amplification <= 1000000,
            crate::errors::DeFiError::InvalidCurveParameters
        );
        
        // Validate fee rates
        require!(
            params.fee_rate <= 1000, // Max 10%
            crate::errors::DeFiError::InvalidFee
        );
        
        require!(
            params.admin_fee_rate <= 5000, // Max 50% of trading fee
            crate::errors::DeFiError::InvalidFee
        );
        
        // Validate precision
        require!(
            params.precision >= 1_000_000 && params.precision <= 1_000_000_000_000_000_000_000_000,
            crate::errors::DeFiError::InvalidCurveParameters
        );
        
        // Validate price range for concentrated liquidity
        if let Some(range) = &params.price_range {
            require!(
                range.lower_price < range.upper_price,
                crate::errors::DeFiError::InvalidPriceRange
            );
            require!(
                range.lower_price > 0,
                crate::errors::DeFiError::InvalidPriceRange
            );
        }
        
        // Validate Finova parameters
        if let Some(finova_params) = &params.finova_params {
            require!(
                finova_params.xp_multiplier <= 10000, // Max 100% bonus
                crate::errors::DeFiError::InvalidCurveParameters
            );
            require!(
                finova_params.rp_multiplier <= 5000, // Max 50% fee reduction
                crate::errors::DeFiError::InvalidCurveParameters
            );
            require!(
                finova_params.mining_boost <= 20000, // Max 200% boost
                crate::errors::DeFiError::InvalidCurveParameters
            );
        }
        
        Ok(())
    }

    /// Calculate optimal arbitrage opportunity
    pub fn calculate_arbitrage(
        curve_params: &CurveParameters,
        reserve_a: u64,
        reserve_b: u64,
        external_price: u128,
        precision: u128,
    ) -> Result<u64> {
        let internal_price = Self::get_price(reserve_a, reserve_b, precision)?;
        
        // If prices are equal, no arbitrage opportunity
        if internal_price == external_price {
            return Ok(0);
        }
        
        // Calculate optimal arbitrage amount using binary search
        let mut low = 0u64;
        let mut high = std::cmp::max(reserve_a, reserve_b) / 10; // Max 10% of reserves
        let mut optimal_amount = 0u64;
        
        for _ in 0..20 { // 20 iterations should be enough for convergence
            let mid = (low + high) / 2;
            if mid == 0 {
                break;
            }
            
            // Calculate swap result
            let swap_result = Self::calculate_swap(
                curve_params,
                reserve_a,
                reserve_b,
                mid,
                internal_price > external_price, // Buy if internal > external
                0, 0, // No XP/RP for arbitrage calculation
                1000, // 10% slippage tolerance
            )?;
            
            // Calculate profit
            let new_price = Self::get_price(
                swap_result.new_reserve_a,
                swap_result.new_reserve_b,
                precision,
            )?;
            
            let price_diff = if new_price > external_price {
                new_price - external_price
            } else {
                external_price - new_price
            };
            
            if price_diff < precision / 1000 { // Within 0.1% of target
                optimal_amount = mid;
                break;
            } else if new_price > external_price {
                if internal_price > external_price {
                    high = mid - 1;
                } else {
                    low = mid + 1;
                }
            } else {
                if internal_price > external_price {
                    low = mid + 1;
                } else {
                    high = mid - 1;
                }
            }
        }
        
        Ok(optimal_amount)
    }
}

/// Price oracle implementation for TWAP calculations
impl PriceOracle {
    /// Create new price oracle
    pub fn new(max_history: usize, twap_period: i64) -> Self {
        Self {
            price_history: VecDeque::with_capacity(max_history),
            max_history,
            last_update: 0,
            twap: 0,
            twap_period,
        }
    }
    
    /// Get current price
    pub fn get_current_price(&self) -> Option<u128> {
        self.price_history.back().map(|point| point.price)
    }
    
    /// Get price at specific timestamp
    pub fn get_price_at_time(&self, timestamp: i64) -> Option<u128> {
        self.price_history
            .iter()
            .find(|point| point.timestamp <= timestamp)
            .map(|point| point.price)
    }
    
    /// Get volume weighted average price
    pub fn get_vwap(&self, period: i64, current_time: i64) -> Result<u128> {
        let start_time = current_time - period;
        let mut volume_weighted_sum = 0u128;
        let mut total_volume = 0u64;
        
        for point in self.price_history.iter() {
            if point.timestamp >= start_time {
                volume_weighted_sum = volume_weighted_sum
                    .checked_add(point.price.checked_mul(point.volume as u128).ok_or(crate::errors::DeFiError::MathOverflow)?)
                    .ok_or(crate::errors::DeFiError::MathOverflow)?;
                total_volume += point.volume;
            }
        }
        
        if total_volume == 0 {
            return Ok(self.get_current_price().unwrap_or(0));
        }
        
        let vwap = volume_weighted_sum
            .checked_div(total_volume as u128)
            .ok_or(crate::errors::DeFiError::MathOverflow)?;
        
        Ok(vwap)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_constant_product_swap() {
        let curve_params = CurveParameters::default();
        
        let result = Curve::calculate_swap(
            &curve_params,
            1000000, // 1M token A
            1000000, // 1M token B
            1000,    // 1K token A in
            true,    // A to B
            0, 0,    // No XP/RP
            500,     // 5% slippage
        ).unwrap();
        
        assert!(result.amount_out > 0);
        assert!(result.fee_amount > 0);
        assert!(result.slippage_ok);
    }
    
    #[test]
    fn test_sqrt() {
        assert_eq!(Curve::sqrt(0).unwrap(), 0);
        assert_eq!(Curve::sqrt(1).unwrap(), 1);
        assert_eq!(Curve::sqrt(4).unwrap(), 2);
        assert_eq!(Curve::sqrt(9).unwrap(), 3);
        assert_eq!(Curve::sqrt(100).unwrap(), 10);
    }
    
    #[test]
    fn test_price_calculation() {
        let price = Curve::get_price(1000, 2000, Curve::PRECISION).unwrap();
        assert_eq!(price, 2 * Curve::PRECISION);
    }
}
