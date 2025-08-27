// programs/finova-defi/src/state/pool.rs

use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, TokenAccount};
use crate::constants::*;
use crate::errors::FinovaDefiError;
use std::collections::BTreeMap;

/// Liquidity Pool State Account
/// Manages AMM pools for FIN token pairs with dynamic fee structures
#[account]
#[derive(Debug)]
pub struct LiquidityPool {
    /// Pool authority (PDA)
    pub authority: Pubkey,
    
    /// Pool identifier
    pub pool_id: u64,
    
    /// Token A mint (e.g., FIN)
    pub token_a_mint: Pubkey,
    
    /// Token B mint (e.g., USDC, SOL)
    pub token_b_mint: Pubkey,
    
    /// Token A vault account
    pub token_a_vault: Pubkey,
    
    /// Token B vault account
    pub token_b_vault: Pubkey,
    
    /// LP token mint
    pub lp_mint: Pubkey,
    
    /// Current reserves of token A
    pub reserve_a: u64,
    
    /// Current reserves of token B
    pub reserve_b: u64,
    
    /// Total LP tokens minted
    pub lp_supply: u64,
    
    /// Pool fee (basis points, e.g., 30 = 0.3%)
    pub fee_rate: u16,
    
    /// Protocol fee share (basis points)
    pub protocol_fee_rate: u16,
    
    /// Pool creation timestamp
    pub created_at: i64,
    
    /// Last activity timestamp
    pub last_activity: i64,
    
    /// Pool status
    pub pool_status: PoolStatus,
    
    /// Trading volume (24h)
    pub volume_24h: u64,
    
    /// Trading volume (7d)
    pub volume_7d: u64,
    
    /// Trading volume (30d)
    pub volume_30d: u64,
    
    /// Total fees collected
    pub total_fees_collected: u64,
    
    /// Protocol fees collected
    pub protocol_fees_collected: u64,
    
    /// Number of liquidity providers
    pub lp_count: u32,
    
    /// Total transactions
    pub total_transactions: u64,
    
    /// Pool configuration flags
    pub config_flags: u32,
    
    /// Curve type for AMM (Constant Product, Stable, etc.)
    pub curve_type: CurveType,
    
    /// Curve parameters (amplification factor for stable curves)
    pub curve_params: [u64; 4],
    
    /// Oracle price feeds (if enabled)
    pub oracle_a: Option<Pubkey>,
    pub oracle_b: Option<Pubkey>,
    
    /// Price impact protection threshold (basis points)
    pub max_price_impact: u16,
    
    /// Maximum slippage allowed (basis points)
    pub max_slippage: u16,
    
    /// Minimum liquidity locked (to prevent rug pulls)
    pub min_liquidity_locked: u64,
    
    /// Pool creator
    pub creator: Pubkey,
    
    /// Pool admin (can modify parameters)
    pub admin: Pubkey,
    
    /// Fee recipient for protocol fees
    pub fee_recipient: Pubkey,
    
    /// Reserved space for future upgrades
    pub reserved: [u8; 256],
}

impl LiquidityPool {
    pub const LEN: usize = 8 + // discriminator
        32 + // authority
        8 +  // pool_id
        32 + // token_a_mint
        32 + // token_b_mint
        32 + // token_a_vault
        32 + // token_b_vault
        32 + // lp_mint
        8 +  // reserve_a
        8 +  // reserve_b
        8 +  // lp_supply
        2 +  // fee_rate
        2 +  // protocol_fee_rate
        8 +  // created_at
        8 +  // last_activity
        1 +  // pool_status
        8 +  // volume_24h
        8 +  // volume_7d
        8 +  // volume_30d
        8 +  // total_fees_collected
        8 +  // protocol_fees_collected
        4 +  // lp_count
        8 +  // total_transactions
        4 +  // config_flags
        1 +  // curve_type
        32 + // curve_params (8 * 4)
        33 + // oracle_a (1 + 32)
        33 + // oracle_b (1 + 32)
        2 +  // max_price_impact
        2 +  // max_slippage
        8 +  // min_liquidity_locked
        32 + // creator
        32 + // admin
        32 + // fee_recipient
        256; // reserved

    /// Initialize a new liquidity pool
    pub fn initialize(
        &mut self,
        authority: Pubkey,
        pool_id: u64,
        token_a_mint: Pubkey,
        token_b_mint: Pubkey,
        token_a_vault: Pubkey,
        token_b_vault: Pubkey,
        lp_mint: Pubkey,
        fee_rate: u16,
        protocol_fee_rate: u16,
        creator: Pubkey,
        admin: Pubkey,
        fee_recipient: Pubkey,
        curve_type: CurveType,
        clock: &Clock,
    ) -> Result<()> {
        require!(
            fee_rate <= MAX_FEE_RATE,
            FinovaDefiError::FeeRateTooHigh
        );
        require!(
            protocol_fee_rate <= MAX_PROTOCOL_FEE_RATE,
            FinovaDefiError::ProtocolFeeRateTooHigh
        );

        self.authority = authority;
        self.pool_id = pool_id;
        self.token_a_mint = token_a_mint;
        self.token_b_mint = token_b_mint;
        self.token_a_vault = token_a_vault;
        self.token_b_vault = token_b_vault;
        self.lp_mint = lp_mint;
        self.reserve_a = 0;
        self.reserve_b = 0;
        self.lp_supply = 0;
        self.fee_rate = fee_rate;
        self.protocol_fee_rate = protocol_fee_rate;
        self.created_at = clock.unix_timestamp;
        self.last_activity = clock.unix_timestamp;
        self.pool_status = PoolStatus::Active;
        self.volume_24h = 0;
        self.volume_7d = 0;
        self.volume_30d = 0;
        self.total_fees_collected = 0;
        self.protocol_fees_collected = 0;
        self.lp_count = 0;
        self.total_transactions = 0;
        self.config_flags = 0;
        self.curve_type = curve_type;
        self.curve_params = [0; 4];
        self.oracle_a = None;
        self.oracle_b = None;
        self.max_price_impact = DEFAULT_MAX_PRICE_IMPACT;
        self.max_slippage = DEFAULT_MAX_SLIPPAGE;
        self.min_liquidity_locked = MIN_LIQUIDITY_THRESHOLD;
        self.creator = creator;
        self.admin = admin;
        self.fee_recipient = fee_recipient;
        self.reserved = [0; 256];

        Ok(())
    }

    /// Calculate swap output using constant product formula
    pub fn calculate_swap_output(
        &self,
        input_amount: u64,
        input_reserve: u64,
        output_reserve: u64,
    ) -> Result<(u64, u64)> {
        require!(
            input_amount > 0,
            FinovaDefiError::InvalidSwapAmount
        );
        require!(
            input_reserve > 0 && output_reserve > 0,
            FinovaDefiError::InsufficientLiquidity
        );

        match self.curve_type {
            CurveType::ConstantProduct => {
                self.calculate_constant_product_swap(input_amount, input_reserve, output_reserve)
            }
            CurveType::Stable => {
                self.calculate_stable_swap(input_amount, input_reserve, output_reserve)
            }
            CurveType::Weighted => {
                self.calculate_weighted_swap(input_amount, input_reserve, output_reserve)
            }
        }
    }

    /// Constant product formula: x * y = k
    fn calculate_constant_product_swap(
        &self,
        input_amount: u64,
        input_reserve: u64,
        output_reserve: u64,
    ) -> Result<(u64, u64)> {
        // Apply fee to input amount
        let fee_amount = (input_amount as u128)
            .checked_mul(self.fee_rate as u128)
            .ok_or(FinovaDefiError::MathOverflow)?
            .checked_div(10000)
            .ok_or(FinovaDefiError::MathOverflow)? as u64;

        let input_amount_after_fee = input_amount
            .checked_sub(fee_amount)
            .ok_or(FinovaDefiError::MathOverflow)?;

        // Calculate output: dy = y * dx / (x + dx)
        let numerator = (output_reserve as u128)
            .checked_mul(input_amount_after_fee as u128)
            .ok_or(FinovaDefiError::MathOverflow)?;

        let denominator = (input_reserve as u128)
            .checked_add(input_amount_after_fee as u128)
            .ok_or(FinovaDefiError::MathOverflow)?;

        let output_amount = numerator
            .checked_div(denominator)
            .ok_or(FinovaDefiError::MathOverflow)? as u64;

        // Check price impact
        let price_impact = self.calculate_price_impact(
            input_amount,
            output_amount,
            input_reserve,
            output_reserve,
        )?;

        require!(
            price_impact <= self.max_price_impact,
            FinovaDefiError::PriceImpactTooHigh
        );

        Ok((output_amount, fee_amount))
    }

    /// Stable swap formula for correlated assets
    fn calculate_stable_swap(
        &self,
        input_amount: u64,
        input_reserve: u64,
        output_reserve: u64,
    ) -> Result<(u64, u64)> {
        let amp = self.curve_params[0];
        require!(amp > 0, FinovaDefiError::InvalidCurveParams);

        // Simplified stable swap calculation
        // In production, this would use the full StableSwap invariant
        let fee_amount = (input_amount as u128)
            .checked_mul(self.fee_rate as u128)
            .ok_or(FinovaDefiError::MathOverflow)?
            .checked_div(10000)
            .ok_or(FinovaDefiError::MathOverflow)? as u64;

        let input_amount_after_fee = input_amount
            .checked_sub(fee_amount)
            .ok_or(FinovaDefiError::MathOverflow)?;

        // For stable pairs, output is approximately 1:1 with lower slippage
        let stable_factor = 10000 - (amp as u64 / 100); // Simplified factor
        let output_amount = (input_amount_after_fee as u128)
            .checked_mul(stable_factor as u128)
            .ok_or(FinovaDefiError::MathOverflow)?
            .checked_div(10000)
            .ok_or(FinovaDefiError::MathOverflow)? as u64;

        Ok((output_amount, fee_amount))
    }

    /// Weighted swap for multi-asset pools
    fn calculate_weighted_swap(
        &self,
        input_amount: u64,
        input_reserve: u64,
        output_reserve: u64,
    ) -> Result<(u64, u64)> {
        let weight_in = self.curve_params[0];
        let weight_out = self.curve_params[1];
        
        require!(
            weight_in > 0 && weight_out > 0,
            FinovaDefiError::InvalidCurveParams
        );

        // Weighted formula: more complex calculation
        // Simplified for this implementation
        let fee_amount = (input_amount as u128)
            .checked_mul(self.fee_rate as u128)
            .ok_or(FinovaDefiError::MathOverflow)?
            .checked_div(10000)
            .ok_or(FinovaDefiError::MathOverflow)? as u64;

        let input_amount_after_fee = input_amount
            .checked_sub(fee_amount)
            .ok_or(FinovaDefiError::MathOverflow)?;

        // Apply weight factors
        let weight_ratio = (weight_out as u128)
            .checked_mul(10000)
            .ok_or(FinovaDefiError::MathOverflow)?
            .checked_div(weight_in as u128)
            .ok_or(FinovaDefiError::MathOverflow)?;

        let output_amount = (input_amount_after_fee as u128)
            .checked_mul(weight_ratio)
            .ok_or(FinovaDefiError::MathOverflow)?
            .checked_div(10000)
            .ok_or(FinovaDefiError::MathOverflow)? as u64;

        Ok((output_amount, fee_amount))
    }

    /// Calculate liquidity tokens to mint
    pub fn calculate_liquidity_tokens(
        &self,
        amount_a: u64,
        amount_b: u64,
    ) -> Result<u64> {
        if self.lp_supply == 0 {
            // First liquidity provision - geometric mean
            let liquidity = ((amount_a as u128)
                .checked_mul(amount_b as u128)
                .ok_or(FinovaDefiError::MathOverflow)?)
                .integer_sqrt() as u64;
            
            require!(
                liquidity > MINIMUM_LIQUIDITY,
                FinovaDefiError::InsufficientLiquidityMinted
            );
            
            Ok(liquidity.checked_sub(MINIMUM_LIQUIDITY).unwrap())
        } else {
            // Subsequent liquidity provision - proportional
            let liquidity_a = (amount_a as u128)
                .checked_mul(self.lp_supply as u128)
                .ok_or(FinovaDefiError::MathOverflow)?
                .checked_div(self.reserve_a as u128)
                .ok_or(FinovaDefiError::MathOverflow)?;

            let liquidity_b = (amount_b as u128)
                .checked_mul(self.lp_supply as u128)
                .ok_or(FinovaDefiError::MathOverflow)?
                .checked_div(self.reserve_b as u128)
                .ok_or(FinovaDefiError::MathOverflow)?;

            Ok(std::cmp::min(liquidity_a, liquidity_b) as u64)
        }
    }

    /// Calculate price impact percentage
    pub fn calculate_price_impact(
        &self,
        input_amount: u64,
        output_amount: u64,
        input_reserve: u64,
        output_reserve: u64,
    ) -> Result<u16> {
        // Price before swap
        let price_before = (output_reserve as u128)
            .checked_mul(10000)
            .ok_or(FinovaDefiError::MathOverflow)?
            .checked_div(input_reserve as u128)
            .ok_or(FinovaDefiError::MathOverflow)?;

        // Price after swap
        let new_input_reserve = input_reserve
            .checked_add(input_amount)
            .ok_or(FinovaDefiError::MathOverflow)?;
        let new_output_reserve = output_reserve
            .checked_sub(output_amount)
            .ok_or(FinovaDefiError::MathOverflow)?;

        let price_after = (new_output_reserve as u128)
            .checked_mul(10000)
            .ok_or(FinovaDefiError::MathOverflow)?
            .checked_div(new_input_reserve as u128)
            .ok_or(FinovaDefiError::MathOverflow)?;

        // Calculate impact percentage
        if price_after >= price_before {
            Ok(0)
        } else {
            let impact = (price_before - price_after)
                .checked_mul(10000)
                .ok_or(FinovaDefiError::MathOverflow)?
                .checked_div(price_before)
                .ok_or(FinovaDefiError::MathOverflow)?;
            
            Ok(impact as u16)
        }
    }

    /// Update pool reserves and statistics
    pub fn update_after_swap(
        &mut self,
        new_reserve_a: u64,
        new_reserve_b: u64,
        volume: u64,
        fee_amount: u64,
        clock: &Clock,
    ) -> Result<()> {
        self.reserve_a = new_reserve_a;
        self.reserve_b = new_reserve_b;
        self.last_activity = clock.unix_timestamp;
        self.total_transactions = self.total_transactions
            .checked_add(1)
            .ok_or(FinovaDefiError::MathOverflow)?;

        // Update volume tracking (simplified - would need time-based sliding windows)
        self.volume_24h = self.volume_24h
            .checked_add(volume)
            .ok_or(FinovaDefiError::MathOverflow)?;
        self.volume_7d = self.volume_7d
            .checked_add(volume)
            .ok_or(FinovaDefiError::MathOverflow)?;
        self.volume_30d = self.volume_30d
            .checked_add(volume)
            .ok_or(FinovaDefiError::MathOverflow)?;

        // Update fee tracking
        self.total_fees_collected = self.total_fees_collected
            .checked_add(fee_amount)
            .ok_or(FinovaDefiError::MathOverflow)?;

        let protocol_fee = (fee_amount as u128)
            .checked_mul(self.protocol_fee_rate as u128)
            .ok_or(FinovaDefiError::MathOverflow)?
            .checked_div(10000)
            .ok_or(FinovaDefiError::MathOverflow)? as u64;

        self.protocol_fees_collected = self.protocol_fees_collected
            .checked_add(protocol_fee)
            .ok_or(FinovaDefiError::MathOverflow)?;

        Ok(())
    }

    /// Update liquidity provider count
    pub fn update_lp_count(&mut self, is_adding: bool) -> Result<()> {
        if is_adding {
            self.lp_count = self.lp_count
                .checked_add(1)
                .ok_or(FinovaDefiError::MathOverflow)?;
        } else {
            self.lp_count = self.lp_count
                .checked_sub(1)
                .ok_or(FinovaDefiError::MathOverflow)?;
        }
        Ok(())
    }

    /// Check if pool is active
    pub fn is_active(&self) -> bool {
        matches!(self.pool_status, PoolStatus::Active)
    }

    /// Pause/unpause pool
    pub fn set_status(&mut self, status: PoolStatus) {
        self.pool_status = status;
    }

    /// Update fee rates (admin only)
    pub fn update_fee_rates(
        &mut self,
        new_fee_rate: u16,
        new_protocol_fee_rate: u16,
    ) -> Result<()> {
        require!(
            new_fee_rate <= MAX_FEE_RATE,
            FinovaDefiError::FeeRateTooHigh
        );
        require!(
            new_protocol_fee_rate <= MAX_PROTOCOL_FEE_RATE,
            FinovaDefiError::ProtocolFeeRateTooHigh
        );

        self.fee_rate = new_fee_rate;
        self.protocol_fee_rate = new_protocol_fee_rate;
        Ok(())
    }

    /// Get current pool price (token B per token A)
    pub fn get_price(&self) -> Result<u64> {
        require!(
            self.reserve_a > 0 && self.reserve_b > 0,
            FinovaDefiError::InsufficientLiquidity
        );

        Ok((self.reserve_b as u128)
            .checked_mul(PRICE_PRECISION as u128)
            .ok_or(FinovaDefiError::MathOverflow)?
            .checked_div(self.reserve_a as u128)
            .ok_or(FinovaDefiError::MathOverflow)? as u64)
    }

    /// Calculate APY based on fees and volume
    pub fn calculate_apy(&self) -> Result<u64> {
        if self.lp_supply == 0 || self.volume_24h == 0 {
            return Ok(0);
        }

        // Annual fee revenue = daily_fees * 365
        let daily_fees = (self.volume_24h as u128)
            .checked_mul(self.fee_rate as u128)
            .ok_or(FinovaDefiError::MathOverflow)?
            .checked_div(10000)
            .ok_or(FinovaDefiError::MathOverflow)?;

        let annual_fees = daily_fees
            .checked_mul(365)
            .ok_or(FinovaDefiError::MathOverflow)?;

        // Total pool value (simplified)
        let pool_value = (self.reserve_a as u128)
            .checked_add(self.reserve_b as u128)
            .ok_or(FinovaDefiError::MathOverflow)?;

        if pool_value == 0 {
            return Ok(0);
        }

        // APY = (annual_fees / pool_value) * 100
        let apy = annual_fees
            .checked_mul(10000) // 100 * 100 for percentage
            .ok_or(FinovaDefiError::MathOverflow)?
            .checked_div(pool_value)
            .ok_or(FinovaDefiError::MathOverflow)?;

        Ok(apy as u64)
    }
}

/// Pool status enumeration
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum PoolStatus {
    Active,
    Paused,
    Deprecated,
    Emergency,
}

/// Curve type for different AMM formulas
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum CurveType {
    ConstantProduct, // x * y = k (Uniswap V2 style)
    Stable,          // StableSwap for correlated assets
    Weighted,        // Balancer-style weighted pools
}

/// Pool configuration flags
pub struct PoolConfigFlags;

impl PoolConfigFlags {
    pub const ENABLE_ORACLE: u32 = 1 << 0;
    pub const ENABLE_FLASH_LOANS: u32 = 1 << 1;
    pub const RESTRICT_DEPOSITS: u32 = 1 << 2;
    pub const RESTRICT_WITHDRAWALS: u32 = 1 << 3;
    pub const ENABLE_DYNAMIC_FEES: u32 = 1 << 4;
}

/// Liquidity Position Account
/// Tracks individual LP positions
#[account]
#[derive(Debug)]
pub struct LiquidityPosition {
    /// Position owner
    pub owner: Pubkey,
    
    /// Pool this position belongs to
    pub pool: Pubkey,
    
    /// LP tokens held
    pub lp_tokens: u64,
    
    /// Original deposit amount A
    pub deposited_a: u64,
    
    /// Original deposit amount B
    pub deposited_b: u64,
    
    /// Position creation timestamp
    pub created_at: i64,
    
    /// Last interaction timestamp
    pub last_interaction: i64,
    
    /// Fees earned (unclaimed)
    pub unclaimed_fees_a: u64,
    pub unclaimed_fees_b: u64,
    
    /// Total fees claimed
    pub total_fees_claimed_a: u64,
    pub total_fees_claimed_b: u64,
    
    /// Position status
    pub status: PositionStatus,
    
    /// Lock period (for incentivized positions)
    pub lock_until: i64,
    
    /// Multiplier for rewards (based on lock period)
    pub reward_multiplier: u16,
    
    /// Reserved space
    pub reserved: [u8; 128],
}

impl LiquidityPosition {
    pub const LEN: usize = 8 + // discriminator
        32 + // owner
        32 + // pool
        8 +  // lp_tokens
        8 +  // deposited_a
        8 +  // deposited_b
        8 +  // created_at
        8 +  // last_interaction
        8 +  // unclaimed_fees_a
        8 +  // unclaimed_fees_b
        8 +  // total_fees_claimed_a
        8 +  // total_fees_claimed_b
        1 +  // status
        8 +  // lock_until
        2 +  // reward_multiplier
        128; // reserved

    pub fn initialize(
        &mut self,
        owner: Pubkey,
        pool: Pubkey,
        lp_tokens: u64,
        deposited_a: u64,
        deposited_b: u64,
        lock_period: i64,
        clock: &Clock,
    ) -> Result<()> {
        self.owner = owner;
        self.pool = pool;
        self.lp_tokens = lp_tokens;
        self.deposited_a = deposited_a;
        self.deposited_b = deposited_b;
        self.created_at = clock.unix_timestamp;
        self.last_interaction = clock.unix_timestamp;
        self.unclaimed_fees_a = 0;
        self.unclaimed_fees_b = 0;
        self.total_fees_claimed_a = 0;
        self.total_fees_claimed_b = 0;
        self.status = PositionStatus::Active;
        self.lock_until = clock.unix_timestamp + lock_period;
        
        // Calculate reward multiplier based on lock period
        self.reward_multiplier = if lock_period == 0 {
            10000 // 1.0x (no lock bonus)
        } else if lock_period <= 30 * 24 * 3600 { // 30 days
            11000 // 1.1x
        } else if lock_period <= 90 * 24 * 3600 { // 90 days
            12000 // 1.2x
        } else if lock_period <= 365 * 24 * 3600 { // 1 year
            15000 // 1.5x
        } else {
            20000 // 2.0x for longer locks
        };
        
        self.reserved = [0; 128];
        Ok(())
    }

    pub fn is_locked(&self, current_time: i64) -> bool {
        current_time < self.lock_until
    }

    pub fn update_fees(&mut self, fees_a: u64, fees_b: u64) {
        self.unclaimed_fees_a += fees_a;
        self.unclaimed_fees_b += fees_b;
    }

    pub fn claim_fees(&mut self) -> (u64, u64) {
        let claimable_a = self.unclaimed_fees_a;
        let claimable_b = self.unclaimed_fees_b;
        
        self.unclaimed_fees_a = 0;
        self.unclaimed_fees_b = 0;
        self.total_fees_claimed_a += claimable_a;
        self.total_fees_claimed_b += claimable_b;
        
        (claimable_a, claimable_b)
    }
}

/// Position status
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum PositionStatus {
    Active,
    Withdrawn,
    Locked,
}

/// Integer square root implementation
trait IntegerSquareRoot {
    fn integer_sqrt(self) -> Self;
}

impl IntegerSquareRoot for u128 {
    fn integer_sqrt(self) -> Self {
        if self < 2 {
            return self;
        }
        
        let mut x = self;
        let mut y = (x + 1) / 2;
        
        while y < x {
            x = y;
            y = (x + self / x) / 2;
        }
        
        x
    }
}
