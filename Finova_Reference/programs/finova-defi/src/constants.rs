// programs/finova-defi/src/constants.rs

use anchor_lang::prelude::*;

/// Program constants for Finova DeFi operations
pub mod finova_defi {
    use super::*;

    /// Maximum number of tokens supported in a single pool
    pub const MAX_POOL_TOKENS: usize = 8;
    
    /// Minimum liquidity required to create a pool
    pub const MIN_LIQUIDITY: u64 = 1000;
    
    /// Maximum slippage tolerance (in basis points)
    pub const MAX_SLIPPAGE_BPS: u16 = 5000; // 50%
    
    /// Default trading fee (in basis points)
    pub const DEFAULT_TRADING_FEE_BPS: u16 = 30; // 0.3%
    
    /// Maximum trading fee (in basis points)
    pub const MAX_TRADING_FEE_BPS: u16 = 1000; // 10%
    
    /// Protocol fee rate (in basis points)
    pub const PROTOCOL_FEE_BPS: u16 = 500; // 5% of trading fees
    
    /// Minimum swap amount to prevent dust attacks
    pub const MIN_SWAP_AMOUNT: u64 = 100;
    
    /// Maximum flash loan fee (in basis points)
    pub const MAX_FLASH_LOAN_FEE_BPS: u16 = 100; // 1%
    
    /// Default flash loan fee (in basis points)
    pub const DEFAULT_FLASH_LOAN_FEE_BPS: u16 = 5; // 0.05%
    
    /// Maximum number of active farms per user
    pub const MAX_FARMS_PER_USER: usize = 50;
    
    /// Minimum farming period (in seconds)
    pub const MIN_FARMING_PERIOD: i64 = 86400; // 1 day
    
    /// Maximum farming period (in seconds)
    pub const MAX_FARMING_PERIOD: i64 = 31536000; // 1 year
    
    /// Emergency withdrawal penalty (in basis points)
    pub const EMERGENCY_WITHDRAWAL_PENALTY_BPS: u16 = 1000; // 10%
    
    /// Minimum time between harvests (in seconds)
    pub const MIN_HARVEST_INTERVAL: i64 = 3600; // 1 hour
    
    /// Maximum multiplier for farm rewards
    pub const MAX_FARM_MULTIPLIER: u64 = 10_000; // 100x
    
    /// Default farm allocation points
    pub const DEFAULT_ALLOC_POINTS: u64 = 100;
    
    /// Maximum allocation points per farm
    pub const MAX_ALLOC_POINTS: u64 = 10_000;
    
    /// Precision factor for calculations
    pub const PRECISION_FACTOR: u128 = 1_000_000_000_000_000_000; // 1e18
    
    /// Minimum price impact threshold for large trades
    pub const MIN_PRICE_IMPACT_THRESHOLD: u64 = 100; // 1%
    
    /// Maximum price impact allowed
    pub const MAX_PRICE_IMPACT: u64 = 2000; // 20%
    
    /// Circuit breaker threshold (in basis points)
    pub const CIRCUIT_BREAKER_THRESHOLD_BPS: u16 = 5000; // 50%
    
    /// Maximum oracle price deviation (in basis points)
    pub const MAX_ORACLE_DEVIATION_BPS: u16 = 1000; // 10%
    
    /// Oracle price staleness threshold (in seconds)
    pub const ORACLE_STALENESS_THRESHOLD: i64 = 3600; // 1 hour
    
    /// Minimum collateralization ratio (in basis points)
    pub const MIN_COLLATERAL_RATIO_BPS: u16 = 15000; // 150%
    
    /// Liquidation threshold (in basis points)
    pub const LIQUIDATION_THRESHOLD_BPS: u16 = 11000; // 110%
    
    /// Liquidation penalty (in basis points)
    pub const LIQUIDATION_PENALTY_BPS: u16 = 500; // 5%
    
    /// Maximum leverage allowed
    pub const MAX_LEVERAGE: u8 = 20;
    
    /// Minimum position size for leveraged trading
    pub const MIN_LEVERAGED_POSITION: u64 = 10_000; // 10k tokens
    
    /// Maximum position size per user
    pub const MAX_POSITION_SIZE: u64 = 1_000_000_000; // 1B tokens
    
    /// Interest rate precision
    pub const INTEREST_RATE_PRECISION: u64 = 1_000_000; // 1e6
    
    /// Base interest rate (annual, in precision units)
    pub const BASE_INTEREST_RATE: u64 = 50_000; // 5%
    
    /// Maximum interest rate (annual, in precision units)
    pub const MAX_INTEREST_RATE: u64 = 1_000_000; // 100%
    
    /// Utilization precision for interest calculations
    pub const UTILIZATION_PRECISION: u64 = 1_000_000; // 1e6
    
    /// Optimal utilization rate
    pub const OPTIMAL_UTILIZATION: u64 = 800_000; // 80%
    
    /// Interest accrual frequency (in seconds)
    pub const INTEREST_ACCRUAL_FREQUENCY: i64 = 3600; // 1 hour
    
    /// Maximum number of active positions per user
    pub const MAX_POSITIONS_PER_USER: usize = 100;
    
    /// Minimum time between position updates
    pub const MIN_POSITION_UPDATE_INTERVAL: i64 = 60; // 1 minute
    
    /// Maximum number of tokens in a vault
    pub const MAX_VAULT_TOKENS: usize = 20;
    
    /// Vault management fee (annual, in basis points)
    pub const VAULT_MANAGEMENT_FEE_BPS: u16 = 200; // 2%
    
    /// Performance fee (in basis points)
    pub const PERFORMANCE_FEE_BPS: u16 = 2000; // 20%
    
    /// Minimum vault deposit
    pub const MIN_VAULT_DEPOSIT: u64 = 1_000;
    
    /// Maximum vault deposit per transaction
    pub const MAX_VAULT_DEPOSIT: u64 = 100_000_000; // 100M tokens
    
    /// Vault rebalancing threshold (in basis points)
    pub const VAULT_REBALANCE_THRESHOLD_BPS: u16 = 500; // 5%
    
    /// Maximum deviation from target allocation
    pub const MAX_ALLOCATION_DEVIATION_BPS: u16 = 1000; // 10%
    
    /// Minimum time between rebalances
    pub const MIN_REBALANCE_INTERVAL: i64 = 86400; // 1 day
    
    /// Maximum number of strategies per vault
    pub const MAX_STRATEGIES_PER_VAULT: usize = 10;
    
    /// Default strategy allocation (in basis points)
    pub const DEFAULT_STRATEGY_ALLOCATION_BPS: u16 = 1000; // 10%
    
    /// Maximum strategy allocation (in basis points)
    pub const MAX_STRATEGY_ALLOCATION_BPS: u16 = 5000; // 50%
    
    /// Minimum strategy performance period
    pub const MIN_STRATEGY_PERFORMANCE_PERIOD: i64 = 604800; // 1 week
    
    /// Strategy cooldown period after poor performance
    pub const STRATEGY_COOLDOWN_PERIOD: i64 = 1209600; // 2 weeks
    
    /// Maximum allowed strategy loss (in basis points)
    pub const MAX_STRATEGY_LOSS_BPS: u16 = 2000; // 20%
    
    /// Risk score precision
    pub const RISK_SCORE_PRECISION: u64 = 10_000;
    
    /// Maximum risk score
    pub const MAX_RISK_SCORE: u64 = 10_000; // 100%
    
    /// Conservative risk threshold
    pub const CONSERVATIVE_RISK_THRESHOLD: u64 = 3_000; // 30%
    
    /// Aggressive risk threshold
    pub const AGGRESSIVE_RISK_THRESHOLD: u64 = 7_000; // 70%
    
    /// Maximum correlation between assets
    pub const MAX_ASSET_CORRELATION: u64 = 8_000; // 80%
    
    /// Minimum portfolio diversification score
    pub const MIN_DIVERSIFICATION_SCORE: u64 = 2_000; // 20%
    
    /// Maximum concentration in single asset
    pub const MAX_SINGLE_ASSET_CONCENTRATION_BPS: u16 = 5000; // 50%
    
    /// Default stop loss threshold (in basis points)
    pub const DEFAULT_STOP_LOSS_BPS: u16 = 1000; // 10%
    
    /// Maximum stop loss threshold (in basis points)
    pub const MAX_STOP_LOSS_BPS: u16 = 5000; // 50%
    
    /// Take profit threshold (in basis points)
    pub const DEFAULT_TAKE_PROFIT_BPS: u16 = 2000; // 20%
    
    /// Maximum take profit threshold (in basis points)
    pub const MAX_TAKE_PROFIT_BPS: u16 = 10000; // 100%
    
    /// Position size calculation precision
    pub const POSITION_SIZE_PRECISION: u64 = 1_000_000;
    
    /// Risk-adjusted position sizing factor
    pub const RISK_ADJUSTMENT_FACTOR: u64 = 500_000; // 50%
    
    /// Maximum daily trading volume per user
    pub const MAX_DAILY_VOLUME_PER_USER: u64 = 10_000_000; // 10M tokens
    
    /// Anti-MEV protection delay (in slots)
    pub const MEV_PROTECTION_DELAY: u64 = 10;
    
    /// Maximum sandwich attack protection window
    pub const MAX_SANDWICH_PROTECTION_WINDOW: u64 = 50;
    
    /// Front-running protection threshold
    pub const FRONT_RUNNING_PROTECTION_THRESHOLD: u64 = 1000;
    
    /// Maximum number of pending orders per user
    pub const MAX_PENDING_ORDERS: usize = 50;
    
    /// Order expiration time (in seconds)
    pub const DEFAULT_ORDER_EXPIRATION: i64 = 86400; // 1 day
    
    /// Maximum order expiration time (in seconds)
    pub const MAX_ORDER_EXPIRATION: i64 = 2592000; // 30 days
    
    /// Minimum order size
    pub const MIN_ORDER_SIZE: u64 = 10;
    
    /// Maximum order size
    pub const MAX_ORDER_SIZE: u64 = 1_000_000_000; // 1B tokens
    
    /// Order book depth limit
    pub const MAX_ORDER_BOOK_DEPTH: usize = 1000;
    
    /// Price tick size precision
    pub const PRICE_TICK_PRECISION: u64 = 1_000_000;
    
    /// Minimum price tick size
    pub const MIN_PRICE_TICK: u64 = 1;
    
    /// Maximum price deviation from oracle (in basis points)
    pub const MAX_PRICE_DEVIATION_FROM_ORACLE_BPS: u16 = 500; // 5%
    
    /// Governance proposal minimum voting period
    pub const MIN_GOVERNANCE_VOTING_PERIOD: i64 = 259200; // 3 days
    
    /// Governance proposal maximum voting period
    pub const MAX_GOVERNANCE_VOTING_PERIOD: i64 = 1209600; // 14 days
    
    /// Minimum governance proposal threshold
    pub const MIN_PROPOSAL_THRESHOLD: u64 = 100_000; // 100k tokens
    
    /// Governance quorum threshold (in basis points)
    pub const GOVERNANCE_QUORUM_BPS: u16 = 1000; // 10%
    
    /// Maximum governance proposals per user per day
    pub const MAX_PROPOSALS_PER_USER_PER_DAY: u8 = 3;
    
    /// Timelock delay for critical governance changes
    pub const GOVERNANCE_TIMELOCK_DELAY: i64 = 172800; // 2 days
    
    /// Emergency pause duration
    pub const EMERGENCY_PAUSE_DURATION: i64 = 86400; // 1 day
    
    /// Maximum emergency pause extensions
    pub const MAX_EMERGENCY_PAUSE_EXTENSIONS: u8 = 3;
    
    /// Multi-signature threshold for emergency actions
    pub const EMERGENCY_MULTISIG_THRESHOLD: u8 = 3;
    
    /// Maximum number of emergency signers
    pub const MAX_EMERGENCY_SIGNERS: usize = 7;
    
    /// Insurance fund contribution rate (in basis points)
    pub const INSURANCE_FUND_RATE_BPS: u16 = 10; // 0.1%
    
    /// Maximum insurance fund payout per incident (in basis points)
    pub const MAX_INSURANCE_PAYOUT_BPS: u16 = 1000; // 10%
    
    /// Minimum insurance fund reserve ratio
    pub const MIN_INSURANCE_RESERVE_RATIO_BPS: u16 = 500; // 5%
}

/// Mathematical constants for DeFi calculations
pub mod math {
    use super::*;

    /// Maximum number for safe arithmetic operations
    pub const MAX_SAFE_NUMBER: u128 = u128::MAX / 2;
    
    /// Square root precision for AMM calculations
    pub const SQRT_PRECISION: u128 = 1_000_000_000_000; // 1e12
    
    /// Natural logarithm precision
    pub const LN_PRECISION: u128 = 1_000_000_000_000_000_000; // 1e18
    
    /// Exponential precision
    pub const EXP_PRECISION: u128 = 1_000_000_000_000_000_000; // 1e18
    
    /// Pi constant scaled by precision
    pub const PI_SCALED: u128 = 3_141_592_653_589_793_238; // π * 1e18
    
    /// Euler's number scaled by precision
    pub const E_SCALED: u128 = 2_718_281_828_459_045_235; // e * 1e18
    
    /// Maximum iterations for iterative calculations
    pub const MAX_ITERATIONS: u32 = 100;
    
    /// Convergence threshold for iterative calculations
    pub const CONVERGENCE_THRESHOLD: u128 = 1_000_000; // 1e6
    
    /// Black-Scholes calculation precision
    pub const BLACK_SCHOLES_PRECISION: u128 = 1_000_000_000_000_000_000; // 1e18
    
    /// Volatility calculation periods
    pub const VOLATILITY_PERIODS: u32 = 30;
    
    /// Maximum volatility value
    pub const MAX_VOLATILITY: u64 = 10_000_000; // 1000%
    
    /// Minimum volatility value
    pub const MIN_VOLATILITY: u64 = 100; // 0.01%
}

/// Oracle-related constants
pub mod oracle {
    use super::*;

    /// Maximum number of price feeds per oracle
    pub const MAX_PRICE_FEEDS: usize = 20;
    
    /// Price update frequency (in seconds)
    pub const PRICE_UPDATE_FREQUENCY: i64 = 60; // 1 minute
    
    /// Maximum price age before considered stale
    pub const MAX_PRICE_AGE: i64 = 3600; // 1 hour
    
    /// Minimum number of sources for price consensus
    pub const MIN_PRICE_SOURCES: usize = 3;
    
    /// Maximum price deviation between sources (in basis points)
    pub const MAX_SOURCE_DEVIATION_BPS: u16 = 500; // 5%
    
    /// Oracle confidence threshold
    pub const MIN_ORACLE_CONFIDENCE: u64 = 90; // 90%
    
    /// Circuit breaker price change threshold
    pub const CIRCUIT_BREAKER_PRICE_CHANGE_BPS: u16 = 2000; // 20%
    
    /// Oracle update gas limit
    pub const ORACLE_UPDATE_GAS_LIMIT: u64 = 100_000;
    
    /// Maximum oracle response time (in milliseconds)
    pub const MAX_ORACLE_RESPONSE_TIME: u64 = 5000; // 5 seconds
}

/// Security and access control constants
pub mod security {
    use super::*;

    /// Maximum number of admins
    pub const MAX_ADMINS: usize = 5;
    
    /// Maximum number of operators
    pub const MAX_OPERATORS: usize = 20;
    
    /// Admin role change delay
    pub const ADMIN_CHANGE_DELAY: i64 = 172800; // 2 days
    
    /// Emergency pause authority threshold
    pub const EMERGENCY_PAUSE_THRESHOLD: u8 = 2;
    
    /// Rate limiting window (in seconds)
    pub const RATE_LIMIT_WINDOW: i64 = 3600; // 1 hour
    
    /// Maximum transactions per user per window
    pub const MAX_TXS_PER_USER_PER_WINDOW: u32 = 1000;
    
    /// Maximum value per transaction for new users
    pub const MAX_VALUE_NEW_USER: u64 = 10_000;
    
    /// Trusted user threshold (minimum account age in days)
    pub const TRUSTED_USER_THRESHOLD_DAYS: u32 = 30;
    
    /// Maximum daily withdrawal for new users
    pub const MAX_DAILY_WITHDRAWAL_NEW_USER: u64 = 50_000;
    
    /// Anti-bot challenge frequency
    pub const ANTI_BOT_CHALLENGE_FREQUENCY: u32 = 100; // Every 100 transactions
    
    /// Maximum failed attempts before lockout
    pub const MAX_FAILED_ATTEMPTS: u8 = 5;
    
    /// Account lockout duration (in seconds)
    pub const ACCOUNT_LOCKOUT_DURATION: i64 = 3600; // 1 hour
}

/// Feature flags for different DeFi modules
pub mod features {
    /// Enable flash loan functionality
    pub const FLASH_LOANS_ENABLED: bool = true;
    
    /// Enable leveraged trading
    pub const LEVERAGED_TRADING_ENABLED: bool = true;
    
    /// Enable yield farming
    pub const YIELD_FARMING_ENABLED: bool = true;
    
    /// Enable auto-compounding
    pub const AUTO_COMPOUNDING_ENABLED: bool = true;
    
    /// Enable cross-chain functionality
    pub const CROSS_CHAIN_ENABLED: bool = false;
    
    /// Enable options trading
    pub const OPTIONS_TRADING_ENABLED: bool = false;
    
    /// Enable futures trading
    pub const FUTURES_TRADING_ENABLED: bool = false;
    
    /// Enable synthetic assets
    pub const SYNTHETIC_ASSETS_ENABLED: bool = false;
    
    /// Enable governance participation rewards
    pub const GOVERNANCE_REWARDS_ENABLED: bool = true;
    
    /// Enable dynamic fee adjustment
    pub const DYNAMIC_FEES_ENABLED: bool = true;
}

/// Network and performance constants
pub mod network {
    use super::*;

    /// Maximum transaction size
    pub const MAX_TRANSACTION_SIZE: usize = 1232; // Solana limit
    
    /// Maximum compute units per transaction
    pub const MAX_COMPUTE_UNITS: u32 = 1_400_000;
    
    /// Priority fee multiplier for urgent transactions
    pub const PRIORITY_FEE_MULTIPLIER: u64 = 2;
    
    /// Transaction retry attempts
    pub const MAX_RETRY_ATTEMPTS: u8 = 3;
    
    /// RPC timeout (in milliseconds)
    pub const RPC_TIMEOUT_MS: u64 = 30_000;
    
    /// Confirmation timeout (in seconds)
    pub const CONFIRMATION_TIMEOUT: u64 = 60;
    
    /// Maximum number of concurrent transactions
    pub const MAX_CONCURRENT_TXS: usize = 100;
    
    /// Transaction pool size
    pub const TRANSACTION_POOL_SIZE: usize = 10_000;
    
    /// Mempool monitoring interval (in milliseconds)
    pub const MEMPOOL_MONITOR_INTERVAL_MS: u64 = 1000;
}

/// Version and compatibility constants
pub mod version {
    /// Current program version
    pub const PROGRAM_VERSION: u32 = 1;
    
    /// Minimum compatible client version
    pub const MIN_CLIENT_VERSION: u32 = 1;
    
    /// Protocol version
    pub const PROTOCOL_VERSION: u32 = 1;
    
    /// Data structure version
    pub const DATA_VERSION: u32 = 1;
    
    /// API version
    pub const API_VERSION: &str = "v1";
}

/// Default account sizes for rent calculation
pub mod account_sizes {
    use super::*;

    /// Size of Pool account
    pub const POOL_SIZE: usize = 8 + 32 + 32 + 32 + 8 + 8 + 8 + 2 + 2 + 8 + 8 + 1 + 64; // ~283 bytes
    
    /// Size of LiquidityPosition account
    pub const LIQUIDITY_POSITION_SIZE: usize = 8 + 32 + 32 + 8 + 8 + 8 + 8 + 8; // ~152 bytes
    
    /// Size of Farm account
    pub const FARM_SIZE: usize = 8 + 32 + 32 + 32 + 8 + 8 + 8 + 8 + 8 + 8 + 1 + 32; // ~203 bytes
    
    /// Size of Vault account
    pub const VAULT_SIZE: usize = 8 + 32 + 32 + 4 + (32 * 20) + 4 + (2 * 20) + 8 + 8 + 8 + 2 + 2 + 1 + 128; // ~1000+ bytes
    
    /// Size of Position account
    pub const POSITION_SIZE: usize = 8 + 32 + 32 + 32 + 8 + 8 + 8 + 8 + 8 + 8 + 1 + 1 + 64; // ~267 bytes
    
    /// Size of Order account
    pub const ORDER_SIZE: usize = 8 + 32 + 32 + 32 + 8 + 8 + 8 + 8 + 1 + 1 + 1 + 32; // ~219 bytes
}
