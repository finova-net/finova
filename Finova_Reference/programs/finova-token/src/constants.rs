// programs/finova-token/src/constants.rs

use anchor_lang::prelude::*;

/// Program version for upgrades and compatibility
pub const PROGRAM_VERSION: u8 = 1;

/// Token Configuration Constants
pub const TOKEN_NAME: &str = "Finova Network";
pub const TOKEN_SYMBOL: &str = "FIN";
pub const TOKEN_DECIMALS: u8 = 9;

/// Maximum supply: 100 billion tokens (100,000,000,000 * 10^9)
pub const MAX_SUPPLY: u64 = 100_000_000_000_000_000_000;

/// Initial supply allocations (in percentage of max supply)
pub const COMMUNITY_MINING_ALLOCATION: u16 = 5000; // 50%
pub const TEAM_ALLOCATION: u16 = 2000; // 20%
pub const INVESTOR_ALLOCATION: u16 = 1500; // 15%
pub const PUBLIC_SALE_ALLOCATION: u16 = 1000; // 10%
pub const TREASURY_ALLOCATION: u16 = 500; // 5%

/// Token Distribution Constants
pub const TOTAL_ALLOCATION_BASIS_POINTS: u16 = 10000; // 100%

/// Staking Constants
pub const MIN_STAKE_AMOUNT: u64 = 100_000_000_000; // 100 FIN (with decimals)
pub const MAX_STAKE_AMOUNT: u64 = 10_000_000_000_000_000_000; // 10 billion FIN
pub const MIN_STAKE_DURATION: i64 = 86400; // 1 day in seconds
pub const MAX_STAKE_DURATION: i64 = 31536000; // 1 year in seconds

/// Staking Tiers (minimum amounts in FIN with decimals)
pub const BRONZE_TIER_MIN: u64 = 100_000_000_000; // 100 FIN
pub const SILVER_TIER_MIN: u64 = 500_000_000_000; // 500 FIN
pub const GOLD_TIER_MIN: u64 = 1_000_000_000_000; // 1,000 FIN
pub const PLATINUM_TIER_MIN: u64 = 5_000_000_000_000; // 5,000 FIN
pub const DIAMOND_TIER_MIN: u64 = 10_000_000_000_000; // 10,000 FIN

/// Staking APY Rates (basis points - 10000 = 100%)
pub const BRONZE_BASE_APY: u16 = 800; // 8%
pub const SILVER_BASE_APY: u16 = 1000; // 10%
pub const GOLD_BASE_APY: u16 = 1200; // 12%
pub const PLATINUM_BASE_APY: u16 = 1400; // 14%
pub const DIAMOND_BASE_APY: u16 = 1500; // 15%

/// Staking Multipliers (basis points)
pub const BRONZE_MINING_MULTIPLIER: u16 = 1200; // 1.2x
pub const SILVER_MINING_MULTIPLIER: u16 = 1350; // 1.35x
pub const GOLD_MINING_MULTIPLIER: u16 = 1500; // 1.5x
pub const PLATINUM_MINING_MULTIPLIER: u16 = 1750; // 1.75x
pub const DIAMOND_MINING_MULTIPLIER: u16 = 2000; // 2.0x

/// XP Multipliers for staking tiers (basis points)
pub const BRONZE_XP_MULTIPLIER: u16 = 1100; // 1.1x
pub const SILVER_XP_MULTIPLIER: u16 = 1200; // 1.2x
pub const GOLD_XP_MULTIPLIER: u16 = 1300; // 1.3x
pub const PLATINUM_XP_MULTIPLIER: u16 = 1500; // 1.5x
pub const DIAMOND_XP_MULTIPLIER: u16 = 1750; // 1.75x

/// RP Bonus for staking tiers (basis points)
pub const BRONZE_RP_BONUS: u16 = 500; // 5%
pub const SILVER_RP_BONUS: u16 = 1000; // 10%
pub const GOLD_RP_BONUS: u16 = 2000; // 20%
pub const PLATINUM_RP_BONUS: u16 = 3500; // 35%
pub const DIAMOND_RP_BONUS: u16 = 5000; // 50%

/// Loyalty Bonus Constants
pub const LOYALTY_BONUS_MAX: u16 = 5000; // 50% max bonus
pub const LOYALTY_BONUS_PER_MONTH: u16 = 500; // 5% per month
pub const LOYALTY_BONUS_CAP_MONTHS: u8 = 10; // Cap at 10 months

/// Activity Bonus Constants
pub const ACTIVITY_BONUS_MAX: u16 = 2000; // 20% max bonus
pub const ACTIVITY_SCORE_THRESHOLD: u64 = 1000; // Min activity score for bonus
pub const ACTIVITY_BONUS_PER_100_SCORE: u16 = 100; // 1% per 100 activity score

/// Reward Pool Distribution (basis points)
pub const BASE_STAKING_REWARDS: u16 = 4000; // 40%
pub const ACTIVITY_BONUSES: u16 = 2500; // 25%
pub const LOYALTY_REWARDS: u16 = 2000; // 20%
pub const PERFORMANCE_INCENTIVES: u16 = 1000; // 10%
pub const SPECIAL_EVENT_BONUSES: u16 = 500; // 5%

/// Unstaking Penalties (basis points)
pub const EARLY_UNSTAKE_PENALTY_MAX: u16 = 2500; // 25% max penalty
pub const EARLY_UNSTAKE_PENALTY_MIN: u16 = 100; // 1% min penalty
pub const PENALTY_REDUCTION_PER_DAY: u16 = 10; // 0.1% reduction per day

/// Cool-down Periods (in seconds)
pub const UNSTAKE_COOLDOWN: i64 = 259200; // 3 days
pub const CLAIM_COOLDOWN: i64 = 86400; // 1 day
pub const RESTAKE_COOLDOWN: i64 = 3600; // 1 hour

/// Token Economics Constants
pub const BURN_RATE_BASIS_POINTS: u16 = 10; // 0.1% of transactions
pub const WHALE_TAX_THRESHOLD: u64 = 100_000_000_000_000; // 100,000 FIN
pub const WHALE_TAX_RATE: u16 = 500; // 5% additional tax

/// Progressive Whale Tax Rates (basis points)
pub const WHALE_TAX_TIER_1: u64 = 100_000_000_000_000; // 100K FIN
pub const WHALE_TAX_TIER_2: u64 = 500_000_000_000_000; // 500K FIN
pub const WHALE_TAX_TIER_3: u64 = 1_000_000_000_000_000; // 1M FIN
pub const WHALE_TAX_TIER_4: u64 = 5_000_000_000_000_000; // 5M FIN

pub const WHALE_TAX_RATE_1: u16 = 200; // 2%
pub const WHALE_TAX_RATE_2: u16 = 350; // 3.5%
pub const WHALE_TAX_RATE_3: u16 = 500; // 5%
pub const WHALE_TAX_RATE_4: u16 = 750; // 7.5%

/// Deflationary Mechanisms
pub const TRANSACTION_BURN_RATE: u16 = 10; // 0.1%
pub const NFT_USAGE_BURN_RATE: u16 = 10000; // 100% (single-use cards)
pub const STAKING_REWARD_BURN_RATE: u16 = 50; // 0.5% of rewards burned

/// Yield Generation Constants
pub const BASE_YIELD_RATE: u16 = 500; // 5% base yield for sUSDfin
pub const MAX_YIELD_RATE: u16 = 800; // 8% max yield
pub const YIELD_CALCULATION_PERIOD: i64 = 86400; // Daily yield calculation

/// Synthetic Token Constants (sUSDfin)
pub const SUSD_FIN_DECIMALS: u8 = 6; // USDC-like decimals
pub const USD_PEG_TOLERANCE: u16 = 500; // 5% tolerance for USD peg
pub const REBALANCE_THRESHOLD: u16 = 200; // 2% threshold for rebalancing

/// Cross-chain Bridge Constants
pub const MIN_BRIDGE_AMOUNT: u64 = 10_000_000_000; // 10 FIN minimum
pub const MAX_BRIDGE_AMOUNT: u64 = 10_000_000_000_000_000; // 10M FIN maximum
pub const BRIDGE_FEE_RATE: u16 = 100; // 1% bridge fee

/// Oracle Integration Constants
pub const PRICE_UPDATE_TOLERANCE: u16 = 1000; // 10% price change tolerance
pub const ORACLE_STALENESS_THRESHOLD: i64 = 3600; // 1 hour
pub const MIN_ORACLE_SOURCES: u8 = 3; // Minimum oracle sources

/// Security Constants
pub const MAX_SLIPPAGE_TOLERANCE: u16 = 500; // 5% max slippage
pub const EMERGENCY_PAUSE_DURATION: i64 = 86400; // 24 hours
pub const ADMIN_TIMELOCK_DURATION: i64 = 259200; // 3 days

/// Rate Limiting Constants
pub const MAX_TRANSACTIONS_PER_BLOCK: u8 = 10;
pub const MAX_STAKE_OPERATIONS_PER_DAY: u8 = 5;
pub const MAX_CLAIM_OPERATIONS_PER_DAY: u8 = 10;

/// Mathematical Constants
pub const BASIS_POINTS_DENOMINATOR: u64 = 10000; // 100%
pub const SECONDS_PER_DAY: i64 = 86400;
pub const SECONDS_PER_HOUR: i64 = 3600;
pub const SECONDS_PER_YEAR: i64 = 31536000;
pub const DAYS_PER_YEAR: u16 = 365;
pub const MONTHS_PER_YEAR: u8 = 12;

/// Precision Constants
pub const CALCULATION_PRECISION: u64 = 1_000_000_000; // 10^9 for calculations
pub const PERCENTAGE_PRECISION: u64 = 10000; // For percentage calculations
pub const DECIMAL_PRECISION: u8 = 9; // Token decimal precision

/// Account Space Constants
pub const USER_ACCOUNT_SPACE: usize = 8 + 1024; // Account discriminator + user data
pub const STAKE_ACCOUNT_SPACE: usize = 8 + 512; // Account discriminator + stake data
pub const REWARD_POOL_SPACE: usize = 8 + 256; // Account discriminator + pool data
pub const MINT_INFO_SPACE: usize = 8 + 128; // Account discriminator + mint info

/// String Length Limits
pub const MAX_NAME_LENGTH: usize = 64;
pub const MAX_SYMBOL_LENGTH: usize = 16;
pub const MAX_URI_LENGTH: usize = 256;
pub const MAX_DESCRIPTION_LENGTH: usize = 512;

/// Validation Constants
pub const MIN_ACCOUNT_BALANCE: u64 = 5_000_000; // 0.005 SOL for rent exemption
pub const MAX_ACCOUNTS_PER_USER: u8 = 10;
pub const MAX_CONCURRENT_STAKES: u8 = 5;

/// Event Constants
pub const MAX_EVENT_DATA_SIZE: usize = 1024;
pub const EVENT_RETENTION_DAYS: u16 = 30;

/// Network Phase Constants (matching core program)
pub const PHASE_1_USER_THRESHOLD: u64 = 100_000;
pub const PHASE_2_USER_THRESHOLD: u64 = 1_000_000;
pub const PHASE_3_USER_THRESHOLD: u64 = 10_000_000;

/// Mining Integration Constants
pub const MINING_BOOST_PRECISION: u64 = 10000; // For mining multiplier calculations
pub const MIN_MINING_BOOST: u16 = 10000; // 1.0x (no boost)
pub const MAX_MINING_BOOST: u16 = 50000; // 5.0x maximum boost

/// Governance Integration Constants
pub const MIN_GOVERNANCE_STAKE: u64 = 1_000_000_000_000; // 1,000 FIN for governance
pub const GOVERNANCE_VOTE_WEIGHT_PRECISION: u64 = 10000;
pub const PROPOSAL_THRESHOLD: u64 = 10_000_000_000_000; // 10,000 FIN to create proposal

/// Error Code Ranges
pub const TOKEN_ERROR_CODE_START: u32 = 6000;
pub const STAKING_ERROR_CODE_START: u32 = 6100;
pub const REWARD_ERROR_CODE_START: u32 = 6200;
pub const MINT_ERROR_CODE_START: u32 = 6300;

/// Feature Flags
pub const ENABLE_WHALE_TAX: bool = true;
pub const ENABLE_BURN_MECHANISM: bool = true;
pub const ENABLE_LOYALTY_BONUS: bool = true;
pub const ENABLE_ACTIVITY_BONUS: bool = true;
pub const ENABLE_PROGRESSIVE_TAXATION: bool = true;

/// Development Constants (remove in production)
#[cfg(feature = "development")]
pub const FAST_UNSTAKE_COOLDOWN: i64 = 60; // 1 minute for testing

#[cfg(feature = "development")]
pub const FAST_CLAIM_COOLDOWN: i64 = 10; // 10 seconds for testing

/// Test Constants
#[cfg(test)]
pub const TEST_TOKEN_SUPPLY: u64 = 1_000_000_000_000_000; // 1M FIN for testing
#[cfg(test)]
pub const TEST_STAKE_AMOUNT: u64 = 1_000_000_000_000; // 1K FIN for testing

/// Helper Functions for Constants
impl Default for StakingTier {
    fn default() -> Self {
        StakingTier::Bronze
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StakingTier {
    Bronze,
    Silver,
    Gold,
    Platinum,
    Diamond,
}

impl StakingTier {
    pub fn from_amount(amount: u64) -> Self {
        if amount >= DIAMOND_TIER_MIN {
            StakingTier::Diamond
        } else if amount >= PLATINUM_TIER_MIN {
            StakingTier::Platinum
        } else if amount >= GOLD_TIER_MIN {
            StakingTier::Gold
        } else if amount >= SILVER_TIER_MIN {
            StakingTier::Silver
        } else {
            StakingTier::Bronze
        }
    }

    pub fn base_apy(&self) -> u16 {
        match self {
            StakingTier::Bronze => BRONZE_BASE_APY,
            StakingTier::Silver => SILVER_BASE_APY,
            StakingTier::Gold => GOLD_BASE_APY,
            StakingTier::Platinum => PLATINUM_BASE_APY,
            StakingTier::Diamond => DIAMOND_BASE_APY,
        }
    }

    pub fn mining_multiplier(&self) -> u16 {
        match self {
            StakingTier::Bronze => BRONZE_MINING_MULTIPLIER,
            StakingTier::Silver => SILVER_MINING_MULTIPLIER,
            StakingTier::Gold => GOLD_MINING_MULTIPLIER,
            StakingTier::Platinum => PLATINUM_MINING_MULTIPLIER,
            StakingTier::Diamond => DIAMOND_MINING_MULTIPLIER,
        }
    }

    pub fn xp_multiplier(&self) -> u16 {
        match self {
            StakingTier::Bronze => BRONZE_XP_MULTIPLIER,
            StakingTier::Silver => SILVER_XP_MULTIPLIER,
            StakingTier::Gold => GOLD_XP_MULTIPLIER,
            StakingTier::Platinum => PLATINUM_XP_MULTIPLIER,
            StakingTier::Diamond => DIAMOND_XP_MULTIPLIER,
        }
    }

    pub fn rp_bonus(&self) -> u16 {
        match self {
            StakingTier::Bronze => BRONZE_RP_BONUS,
            StakingTier::Silver => SILVER_RP_BONUS,
            StakingTier::Gold => GOLD_RP_BONUS,
            StakingTier::Platinum => PLATINUM_RP_BONUS,
            StakingTier::Diamond => DIAMOND_RP_BONUS,
        }
    }

    pub fn min_amount(&self) -> u64 {
        match self {
            StakingTier::Bronze => BRONZE_TIER_MIN,
            StakingTier::Silver => SILVER_TIER_MIN,
            StakingTier::Gold => GOLD_TIER_MIN,
            StakingTier::Platinum => PLATINUM_TIER_MIN,
            StakingTier::Diamond => DIAMOND_TIER_MIN,
        }
    }
}

/// Helper function to calculate whale tax rate based on amount
pub fn calculate_whale_tax_rate(amount: u64) -> u16 {
    if amount >= WHALE_TAX_TIER_4 {
        WHALE_TAX_RATE_4
    } else if amount >= WHALE_TAX_TIER_3 {
        WHALE_TAX_RATE_3
    } else if amount >= WHALE_TAX_TIER_2 {
        WHALE_TAX_RATE_2
    } else if amount >= WHALE_TAX_TIER_1 {
        WHALE_TAX_RATE_1
    } else {
        0
    }
}

/// Helper function to calculate loyalty bonus
pub fn calculate_loyalty_bonus(stake_duration_days: u32) -> u16 {
    let months = stake_duration_days / 30;
    let bonus = (months as u16).min(LOYALTY_BONUS_CAP_MONTHS as u16) * LOYALTY_BONUS_PER_MONTH;
    bonus.min(LOYALTY_BONUS_MAX)
}

/// Helper function to calculate activity bonus
pub fn calculate_activity_bonus(activity_score: u64) -> u16 {
    if activity_score < ACTIVITY_SCORE_THRESHOLD {
        return 0;
    }
    
    let bonus_multiplier = (activity_score - ACTIVITY_SCORE_THRESHOLD) / 100;
    let bonus = (bonus_multiplier * ACTIVITY_BONUS_PER_100_SCORE as u64) as u16;
    bonus.min(ACTIVITY_BONUS_MAX)
}

/// Helper function to validate token amounts
pub fn is_valid_token_amount(amount: u64) -> bool {
    amount > 0 && amount <= MAX_SUPPLY
}

/// Helper function to validate stake amounts
pub fn is_valid_stake_amount(amount: u64) -> bool {
    amount >= MIN_STAKE_AMOUNT && amount <= MAX_STAKE_AMOUNT
}

/// Helper function to validate stake duration
pub fn is_valid_stake_duration(duration: i64) -> bool {
    duration >= MIN_STAKE_DURATION && duration <= MAX_STAKE_DURATION
}

/// Helper function to calculate annual percentage yield
pub fn calculate_apy(principal: u64, reward: u64, duration_seconds: i64) -> u16 {
    if principal == 0 || duration_seconds == 0 {
        return 0;
    }
    
    let annual_reward = (reward * SECONDS_PER_YEAR as u64) / duration_seconds as u64;
    let apy = (annual_reward * BASIS_POINTS_DENOMINATOR) / principal;
    apy as u16
}

/// Helper function to calculate compound interest
pub fn calculate_compound_interest(
    principal: u64,
    rate: u16,
    time_periods: u64,
    compounds_per_period: u64,
) -> u64 {
    if rate == 0 || time_periods == 0 {
        return principal;
    }
    
    let rate_decimal = rate as f64 / BASIS_POINTS_DENOMINATOR as f64;
    let compound_rate = rate_decimal / compounds_per_period as f64;
    let total_compounds = compounds_per_period * time_periods;
    
    let compound_multiplier = (1.0 + compound_rate).powf(total_compounds as f64);
    (principal as f64 * compound_multiplier) as u64
}

/// Helper function to get current network phase
pub fn get_network_phase(total_users: u64) -> u8 {
    if total_users >= PHASE_3_USER_THRESHOLD {
        4
    } else if total_users >= PHASE_2_USER_THRESHOLD {
        3
    } else if total_users >= PHASE_1_USER_THRESHOLD {
        2
    } else {
        1
    }
}

/// Macro for creating constants with documentation
#[macro_export]
macro_rules! define_constant {
    ($name:ident, $type:ty, $value:expr, $doc:expr) => {
        #[doc = $doc]
        pub const $name: $type = $value;
    };
}

/// Version compatibility check
pub fn is_compatible_version(version: u8) -> bool {
    version <= PROGRAM_VERSION
}

/// Test module for constants validation
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_staking_tier_from_amount() {
        assert_eq!(StakingTier::from_amount(50_000_000_000), StakingTier::Bronze);
        assert_eq!(StakingTier::from_amount(500_000_000_000), StakingTier::Silver);
        assert_eq!(StakingTier::from_amount(1_000_000_000_000), StakingTier::Gold);
        assert_eq!(StakingTier::from_amount(5_000_000_000_000), StakingTier::Platinum);
        assert_eq!(StakingTier::from_amount(10_000_000_000_000), StakingTier::Diamond);
    }

    #[test]
    fn test_whale_tax_calculation() {
        assert_eq!(calculate_whale_tax_rate(50_000_000_000_000), 0);
        assert_eq!(calculate_whale_tax_rate(100_000_000_000_000), WHALE_TAX_RATE_1);
        assert_eq!(calculate_whale_tax_rate(500_000_000_000_000), WHALE_TAX_RATE_2);
        assert_eq!(calculate_whale_tax_rate(1_000_000_000_000_000), WHALE_TAX_RATE_3);
        assert_eq!(calculate_whale_tax_rate(5_000_000_000_000_000), WHALE_TAX_RATE_4);
    }

    #[test]
    fn test_loyalty_bonus_calculation() {
        assert_eq!(calculate_loyalty_bonus(0), 0);
        assert_eq!(calculate_loyalty_bonus(30), LOYALTY_BONUS_PER_MONTH);
        assert_eq!(calculate_loyalty_bonus(60), LOYALTY_BONUS_PER_MONTH * 2);
        assert_eq!(calculate_loyalty_bonus(300), LOYALTY_BONUS_MAX);
        assert_eq!(calculate_loyalty_bonus(400), LOYALTY_BONUS_MAX);
    }

    #[test]
    fn test_activity_bonus_calculation() {
        assert_eq!(calculate_activity_bonus(500), 0);
        assert_eq!(calculate_activity_bonus(1000), 0);
        assert_eq!(calculate_activity_bonus(1100), ACTIVITY_BONUS_PER_100_SCORE);
        assert_eq!(calculate_activity_bonus(3000), ACTIVITY_BONUS_MAX);
        assert_eq!(calculate_activity_bonus(5000), ACTIVITY_BONUS_MAX);
    }

    #[test]
    fn test_validation_functions() {
        assert!(is_valid_token_amount(1_000_000_000));
        assert!(!is_valid_token_amount(0));
        assert!(!is_valid_token_amount(MAX_SUPPLY + 1));

        assert!(is_valid_stake_amount(MIN_STAKE_AMOUNT));
        assert!(is_valid_stake_amount(MAX_STAKE_AMOUNT));
        assert!(!is_valid_stake_amount(MIN_STAKE_AMOUNT - 1));
        assert!(!is_valid_stake_amount(MAX_STAKE_AMOUNT + 1));

        assert!(is_valid_stake_duration(MIN_STAKE_DURATION));
        assert!(is_valid_stake_duration(MAX_STAKE_DURATION));
        assert!(!is_valid_stake_duration(MIN_STAKE_DURATION - 1));
        assert!(!is_valid_stake_duration(MAX_STAKE_DURATION + 1));
    }

    #[test]
    fn test_network_phase() {
        assert_eq!(get_network_phase(50_000), 1);
        assert_eq!(get_network_phase(500_000), 2);
        assert_eq!(get_network_phase(5_000_000), 3);
        assert_eq!(get_network_phase(50_000_000), 4);
    }

    #[test]
    fn test_allocation_percentages() {
        let total = COMMUNITY_MINING_ALLOCATION
            + TEAM_ALLOCATION
            + INVESTOR_ALLOCATION
            + PUBLIC_SALE_ALLOCATION
            + TREASURY_ALLOCATION;
        assert_eq!(total, TOTAL_ALLOCATION_BASIS_POINTS);
    }

    #[test]
    fn test_reward_pool_distribution() {
        let total = BASE_STAKING_REWARDS
            + ACTIVITY_BONUSES
            + LOYALTY_REWARDS
            + PERFORMANCE_INCENTIVES
            + SPECIAL_EVENT_BONUSES;
        assert_eq!(total, TOTAL_ALLOCATION_BASIS_POINTS);
    }
}
