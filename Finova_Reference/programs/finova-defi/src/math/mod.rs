// programs/finova-defi/src/math/mod.rs

// programs/finova-defi/src/math/mod.rs

//! Mathematical utilities for Finova DeFi operations
//! 
//! This module provides mathematical functions for:
//! - AMM curve calculations (constant product, stable swap)
//! - Fee calculations with dynamic pricing
//! - Oracle price aggregation and validation
//! - Liquidity provision mathematics
//! - Yield farming reward calculations
//! 
//! All calculations use safe integer arithmetic to prevent overflow/underflow
//! and maintain precision through fixed-point decimal representations.

use anchor_lang::prelude::*;
use std::cmp::{max, min};

pub mod curve;
pub mod fees;
pub mod oracle;

// Re-export commonly used functions
pub use curve::*;
pub use fees::*;
pub use oracle::*;

/// Fixed-point decimal precision (18 decimals)
pub const PRECISION: u128 = 1_000_000_000_000_000_000;

/// Maximum allowed slippage (5%)
pub const MAX_SLIPPAGE: u64 = 500; // 5% = 500 basis points

/// Minimum liquidity lock to prevent rounding attacks
pub const MINIMUM_LIQUIDITY: u64 = 1000;

/// Maximum fee in basis points (10%)
pub const MAX_FEE_BPS: u64 = 1000;

/// Basis points denominator
pub const BASIS_POINTS: u64 = 10000;

/// Mathematical errors for DeFi operations
#[derive(Debug, Clone, PartialEq)]
pub enum MathError {
    /// Arithmetic overflow occurred
    Overflow,
    /// Arithmetic underflow occurred
    Underflow,
    /// Division by zero attempted
    DivisionByZero,
    /// Invalid precision or decimal places
    InvalidPrecision,
    /// Slippage exceeded maximum allowed
    SlippageExceeded,
    /// Insufficient liquidity for operation
    InsufficientLiquidity,
    /// Price deviation too high from oracle
    PriceDeviationTooHigh,
    /// Invalid curve parameters
    InvalidCurveParams,
}

impl From<MathError> for ProgramError {
    fn from(e: MathError) -> Self {
        match e {
            MathError::Overflow => ProgramError::ArithmeticOverflow,
            MathError::Underflow => ProgramError::ArithmeticOverflow,
            MathError::DivisionByZero => ProgramError::ArithmeticOverflow,
            _ => ProgramError::InvalidArgument,
        }
    }
}

/// Safe arithmetic operations with overflow protection
pub trait SafeMath<T> {
    fn safe_add(&self, other: T) -> Result<T, MathError>;
    fn safe_sub(&self, other: T) -> Result<T, MathError>;
    fn safe_mul(&self, other: T) -> Result<T, MathError>;
    fn safe_div(&self, other: T) -> Result<T, MathError>;
    fn safe_pow(&self, exp: u32) -> Result<T, MathError>;
}

impl SafeMath<u64> for u64 {
    fn safe_add(&self, other: u64) -> Result<u64, MathError> {
        self.checked_add(other).ok_or(MathError::Overflow)
    }

    fn safe_sub(&self, other: u64) -> Result<u64, MathError> {
        self.checked_sub(other).ok_or(MathError::Underflow)
    }

    fn safe_mul(&self, other: u64) -> Result<u64, MathError> {
        self.checked_mul(other).ok_or(MathError::Overflow)
    }

    fn safe_div(&self, other: u64) -> Result<u64, MathError> {
        if other == 0 {
            return Err(MathError::DivisionByZero);
        }
        self.checked_div(other).ok_or(MathError::Overflow)
    }

    fn safe_pow(&self, exp: u32) -> Result<u64, MathError> {
        self.checked_pow(exp).ok_or(MathError::Overflow)
    }
}

impl SafeMath<u128> for u128 {
    fn safe_add(&self, other: u128) -> Result<u128, MathError> {
        self.checked_add(other).ok_or(MathError::Overflow)
    }

    fn safe_sub(&self, other: u128) -> Result<u128, MathError> {
        self.checked_sub(other).ok_or(MathError::Underflow)
    }

    fn safe_mul(&self, other: u128) -> Result<u128, MathError> {
        self.checked_mul(other).ok_or(MathError::Overflow)
    }

    fn safe_div(&self, other: u128) -> Result<u128, MathError> {
        if other == 0 {
            return Err(MathError::DivisionByZero);
        }
        self.checked_div(other).ok_or(MathError::Overflow)
    }

    fn safe_pow(&self, exp: u32) -> Result<u128, MathError> {
        self.checked_pow(exp).ok_or(MathError::Overflow)
    }
}

/// Fixed-point decimal arithmetic
pub struct Decimal {
    pub value: u128,
}

impl Decimal {
    /// Create new decimal from integer
    pub fn from_integer(value: u64) -> Self {
        Self {
            value: (value as u128).safe_mul(PRECISION).unwrap_or(0),
        }
    }

    /// Create new decimal from raw value
    pub fn from_raw(value: u128) -> Self {
        Self { value }
    }

    /// Create decimal from ratio
    pub fn from_ratio(numerator: u64, denominator: u64) -> Result<Self, MathError> {
        if denominator == 0 {
            return Err(MathError::DivisionByZero);
        }
        
        let value = (numerator as u128)
            .safe_mul(PRECISION)?
            .safe_div(denominator as u128)?;
            
        Ok(Self { value })
    }

    /// Convert to integer (truncating decimals)
    pub fn to_integer(&self) -> u64 {
        (self.value / PRECISION) as u64
    }

    /// Get raw value
    pub fn raw(&self) -> u128 {
        self.value
    }

    /// Add two decimals
    pub fn add(&self, other: &Decimal) -> Result<Decimal, MathError> {
        Ok(Decimal {
            value: self.value.safe_add(other.value)?,
        })
    }

    /// Subtract two decimals
    pub fn sub(&self, other: &Decimal) -> Result<Decimal, MathError> {
        Ok(Decimal {
            value: self.value.safe_sub(other.value)?,
        })
    }

    /// Multiply two decimals
    pub fn mul(&self, other: &Decimal) -> Result<Decimal, MathError> {
        let result = self.value.safe_mul(other.value)?.safe_div(PRECISION)?;
        Ok(Decimal { value: result })
    }

    /// Divide two decimals
    pub fn div(&self, other: &Decimal) -> Result<Decimal, MathError> {
        if other.value == 0 {
            return Err(MathError::DivisionByZero);
        }
        
        let result = self.value.safe_mul(PRECISION)?.safe_div(other.value)?;
        Ok(Decimal { value: result })
    }

    /// Calculate square root using Newton's method
    pub fn sqrt(&self) -> Result<Decimal, MathError> {
        if self.value == 0 {
            return Ok(Decimal { value: 0 });
        }

        let mut x = self.value;
        let mut y = (x + 1) / 2;

        while y < x {
            x = y;
            y = (x + self.value / x) / 2;
        }

        Ok(Decimal { value: x })
    }

    /// Calculate power using binary exponentiation
    pub fn pow(&self, exp: u32) -> Result<Decimal, MathError> {
        if exp == 0 {
            return Ok(Decimal::from_integer(1));
        }
        
        let mut result = Decimal::from_integer(1);
        let mut base = *self;
        let mut exponent = exp;

        while exponent > 0 {
            if exponent % 2 == 1 {
                result = result.mul(&base)?;
            }
            base = base.mul(&base)?;
            exponent /= 2;
        }

        Ok(result)
    }

    /// Check if decimal is zero
    pub fn is_zero(&self) -> bool {
        self.value == 0
    }

    /// Get absolute difference between two decimals
    pub fn abs_diff(&self, other: &Decimal) -> Decimal {
        if self.value >= other.value {
            Decimal {
                value: self.value - other.value,
            }
        } else {
            Decimal {
                value: other.value - self.value,
            }
        }
    }

    /// Calculate percentage difference
    pub fn percentage_diff(&self, other: &Decimal) -> Result<Decimal, MathError> {
        if other.is_zero() {
            return Err(MathError::DivisionByZero);
        }

        let diff = self.abs_diff(other);
        let percentage = diff.div(other)?.mul(&Decimal::from_integer(100))?;
        Ok(percentage)
    }
}

impl Copy for Decimal {}

impl Clone for Decimal {
    fn clone(&self) -> Self {
        *self
    }
}

impl PartialEq for Decimal {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl PartialOrd for Decimal {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.value.partial_cmp(&other.value)
    }
}

/// Utility functions for common mathematical operations

/// Calculate geometric mean of two values
pub fn geometric_mean(a: u64, b: u64) -> Result<u64, MathError> {
    let product = (a as u128).safe_mul(b as u128)?;
    let sqrt_val = isqrt(product);
    Ok(sqrt_val as u64)
}

/// Integer square root using binary search
pub fn isqrt(n: u128) -> u128 {
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

/// Calculate compound growth using continuous compounding
pub fn compound_growth(
    principal: u64,
    rate: u64, // in basis points
    time_periods: u64,
) -> Result<u64, MathError> {
    if rate == 0 || time_periods == 0 {
        return Ok(principal);
    }

    let rate_decimal = Decimal::from_ratio(rate, BASIS_POINTS)?;
    let principal_decimal = Decimal::from_integer(principal);
    
    // A = P * (1 + r)^t
    let growth_factor = Decimal::from_integer(1).add(&rate_decimal)?;
    let compound_factor = growth_factor.pow(time_periods as u32)?;
    let result = principal_decimal.mul(&compound_factor)?;
    
    Ok(result.to_integer())
}

/// Calculate weighted average
pub fn weighted_average(values: &[(u64, u64)]) -> Result<u64, MathError> {
    if values.is_empty() {
        return Err(MathError::InvalidPrecision);
    }

    let mut weighted_sum: u128 = 0;
    let mut total_weight: u128 = 0;

    for (value, weight) in values {
        weighted_sum = weighted_sum.safe_add((*value as u128).safe_mul(*weight as u128)?)?;
        total_weight = total_weight.safe_add(*weight as u128)?;
    }

    if total_weight == 0 {
        return Err(MathError::DivisionByZero);
    }

    Ok((weighted_sum / total_weight) as u64)
}

/// Calculate exponential moving average
pub fn exponential_moving_average(
    current_avg: u64,
    new_value: u64,
    smoothing_factor: u64, // in basis points (0-10000)
) -> Result<u64, MathError> {
    let alpha = Decimal::from_ratio(smoothing_factor, BASIS_POINTS)?;
    let one_minus_alpha = Decimal::from_integer(1).sub(&alpha)?;
    
    let current_component = Decimal::from_integer(current_avg).mul(&one_minus_alpha)?;
    let new_component = Decimal::from_integer(new_value).mul(&alpha)?;
    
    let result = current_component.add(&new_component)?;
    Ok(result.to_integer())
}

/// Validate slippage is within acceptable bounds
pub fn validate_slippage(
    expected_amount: u64,
    actual_amount: u64,
    max_slippage_bps: u64,
) -> Result<(), MathError> {
    if max_slippage_bps > MAX_SLIPPAGE {
        return Err(MathError::SlippageExceeded);
    }

    let expected_decimal = Decimal::from_integer(expected_amount);
    let actual_decimal = Decimal::from_integer(actual_amount);
    
    let percentage_diff = expected_decimal.percentage_diff(&actual_decimal)?;
    let max_slippage_decimal = Decimal::from_ratio(max_slippage_bps, BASIS_POINTS)?
        .mul(&Decimal::from_integer(100))?;

    if percentage_diff > max_slippage_decimal {
        return Err(MathError::SlippageExceeded);
    }

    Ok(())
}

/// Calculate impermanent loss for LP positions
pub fn calculate_impermanent_loss(
    initial_price_ratio: u64, // token1/token0 * PRECISION
    current_price_ratio: u64, // token1/token0 * PRECISION
) -> Result<u64, MathError> {
    let initial_ratio = Decimal::from_raw(initial_price_ratio as u128);
    let current_ratio = Decimal::from_raw(current_price_ratio as u128);
    
    // IL = 2 * sqrt(price_ratio) / (1 + price_ratio) - 1
    let sqrt_ratio = current_ratio.sqrt()?;
    let numerator = Decimal::from_integer(2).mul(&sqrt_ratio)?;
    let denominator = Decimal::from_integer(1).add(&current_ratio)?;
    
    let lp_multiplier = numerator.div(&denominator)?;
    
    // Compare with holding
    let hold_sqrt_ratio = initial_ratio.sqrt()?;
    let hold_numerator = Decimal::from_integer(2).mul(&hold_sqrt_ratio)?;
    let hold_denominator = Decimal::from_integer(1).add(&initial_ratio)?;
    let hold_multiplier = hold_numerator.div(&hold_denominator)?;
    
    if lp_multiplier < hold_multiplier {
        let loss = hold_multiplier.sub(&lp_multiplier)?;
        let loss_percentage = loss.div(&hold_multiplier)?.mul(&Decimal::from_integer(100))?;
        Ok(loss_percentage.to_integer())
    } else {
        Ok(0) // No impermanent loss
    }
}

/// Calculate optimal swap amount for arbitrage
pub fn calculate_arbitrage_amount(
    reserve_a: u64,
    reserve_b: u64,
    external_price: u64, // price of B in terms of A * PRECISION
    fee_bps: u64,
) -> Result<u64, MathError> {
    let fee_factor = BASIS_POINTS.safe_sub(fee_bps)?;
    let external_price_decimal = Decimal::from_raw(external_price as u128);
    
    let reserve_a_decimal = Decimal::from_integer(reserve_a);
    let reserve_b_decimal = Decimal::from_integer(reserve_b);
    
    // Current pool price = reserve_a / reserve_b
    let pool_price = reserve_a_decimal.div(&reserve_b_decimal)?;
    
    if external_price_decimal == pool_price {
        return Ok(0); // No arbitrage opportunity
    }
    
    // Simplified calculation for optimal arbitrage amount
    let price_diff = external_price_decimal.abs_diff(&pool_price);
    let total_liquidity = reserve_a_decimal.add(&reserve_b_decimal)?;
    
    // Estimate optimal amount as percentage of liquidity based on price difference
    let arbitrage_factor = price_diff.div(&external_price_decimal)?;
    let optimal_amount = total_liquidity.mul(&arbitrage_factor)?.div(&Decimal::from_integer(4))?;
    
    Ok(min(optimal_amount.to_integer(), reserve_a / 10)) // Cap at 10% of reserve
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_safe_math_operations() {
        assert_eq!(100u64.safe_add(50).unwrap(), 150);
        assert_eq!(100u64.safe_sub(30).unwrap(), 70);
        assert_eq!(10u64.safe_mul(5).unwrap(), 50);
        assert_eq!(100u64.safe_div(4).unwrap(), 25);
        
        // Test overflow
        assert!(u64::MAX.safe_add(1).is_err());
        assert!(0u64.safe_sub(1).is_err());
        assert!(u64::MAX.safe_mul(2).is_err());
        assert!(100u64.safe_div(0).is_err());
    }

    #[test]
    fn test_decimal_operations() {
        let a = Decimal::from_integer(10);
        let b = Decimal::from_integer(5);
        
        assert_eq!(a.add(&b).unwrap().to_integer(), 15);
        assert_eq!(a.sub(&b).unwrap().to_integer(), 5);
        assert_eq!(a.mul(&b).unwrap().to_integer(), 50);
        assert_eq!(a.div(&b).unwrap().to_integer(), 2);
    }

    #[test]
    fn test_geometric_mean() {
        assert_eq!(geometric_mean(16, 9).unwrap(), 12);
        assert_eq!(geometric_mean(100, 400).unwrap(), 200);
    }

    #[test]
    fn test_compound_growth() {
        // 10% APR for 1 period
        let result = compound_growth(1000, 1000, 1).unwrap();
        assert_eq!(result, 1100);
        
        // 5% APR for 2 periods
        let result = compound_growth(1000, 500, 2).unwrap();
        assert_eq!(result, 1102);
    }

    #[test]
    fn test_weighted_average() {
        let values = vec![(100, 3), (200, 2), (300, 1)];
        let avg = weighted_average(&values).unwrap();
        assert_eq!(avg, 166); // (100*3 + 200*2 + 300*1) / (3+2+1) = 1000/6 = 166
    }

    #[test]
    fn test_slippage_validation() {
        // 2% slippage should pass
        assert!(validate_slippage(1000, 980, 300).is_ok());
        
        // 6% slippage should fail with 5% max
        assert!(validate_slippage(1000, 940, 500).is_err());
    }

    #[test]
    fn test_impermanent_loss() {
        // Price doubles, should have some IL
        let initial_price = PRECISION as u64; // 1:1
        let current_price = 2 * PRECISION as u64; // 2:1
        
        let il = calculate_impermanent_loss(initial_price, current_price).unwrap();
        assert!(il > 0);
    }
}
