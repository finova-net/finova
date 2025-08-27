// programs/finova-token/src/state/mod.rs

//! State module for Finova Token Program
//! 
//! This module contains all the account state structures used in the Finova Token Program,
//! including mint information, staking accounts, and reward pools. All states implement
//! proper serialization, validation, and security measures.

use anchor_lang::prelude::*;

pub mod mint_info;
pub mod stake_account;
pub mod reward_pool;

pub use mint_info::*;
pub use stake_account::*;
pub use reward_pool::*;

/// Common trait for all state accounts in the token program
pub trait FinovaTokenState {
    /// Validates the account state
    fn validate(&self) -> Result<()>;
    
    /// Returns the version of the account structure
    fn version(&self) -> u8;
    
    /// Returns the account type identifier
    fn account_type(&self) -> AccountType;
}

/// Account type enumeration for different state structures
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum AccountType {
    MintInfo,
    StakeAccount,
    RewardPool,
    UserTokenAccount,
    VestingAccount,
    GovernanceAccount,
}

/// Token type enumeration for different tokens in the ecosystem
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum TokenType {
    /// Primary utility token
    FIN,
    /// Staked FIN token (liquid staking derivative)
    SFIN,
    /// Synthetic stablecoin
    USDFIN,
    /// Staked USDFIN
    SUSDFIN,
}

impl TokenType {
    /// Returns the decimals for each token type
    pub fn decimals(&self) -> u8 {
        match self {
            TokenType::FIN => 9,
            TokenType::SFIN => 9,
            TokenType::USDFIN => 6,
            TokenType::SUSDFIN => 6,
        }
    }
    
    /// Returns the symbol for each token type
    pub fn symbol(&self) -> &'static str {
        match self {
            TokenType::FIN => "FIN",
            TokenType::SFIN => "sFIN",
            TokenType::USDFIN => "USDfin",
            TokenType::SUSDFIN => "sUSDfin",
        }
    }
    
    /// Returns whether the token is stakeable
    pub fn is_stakeable(&self) -> bool {
        matches!(self, TokenType::FIN | TokenType::USDFIN)
    }
    
    /// Returns whether the token is a staked derivative
    pub fn is_staked_derivative(&self) -> bool {
        matches!(self, TokenType::SFIN | TokenType::SUSDFIN)
    }
}

/// Staking tier enumeration based on staked amounts
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum StakingTier {
    None,
    Basic,      // 100-499 FIN
    Premium,    // 500-999 FIN
    VIP,        // 1,000-4,999 FIN
    Elite,      // 5,000-9,999 FIN
    Legendary,  // 10,000+ FIN
}

impl StakingTier {
    /// Determines staking tier based on staked amount
    pub fn from_amount(amount: u64) -> Self {
        match amount {
            0..=99 => StakingTier::None,
            100..=499 => StakingTier::Basic,
            500..=999 => StakingTier::Premium,
            1000..=4999 => StakingTier::VIP,
            5000..=9999 => StakingTier::Elite,
            _ => StakingTier::Legendary,
        }
    }
    
    /// Returns the APY multiplier for the tier
    pub fn apy_multiplier(&self) -> u16 {
        match self {
            StakingTier::None => 0,
            StakingTier::Basic => 800,      // 8%
            StakingTier::Premium => 1000,   // 10%
            StakingTier::VIP => 1200,       // 12%
            StakingTier::Elite => 1400,     // 14%
            StakingTier::Legendary => 1500, // 15%
        }
    }
    
    /// Returns the mining boost multiplier (basis points)
    pub fn mining_boost(&self) -> u16 {
        match self {
            StakingTier::None => 0,
            StakingTier::Basic => 2000,     // +20%
            StakingTier::Premium => 3500,   // +35%
            StakingTier::VIP => 5000,       // +50%
            StakingTier::Elite => 7500,     // +75%
            StakingTier::Legendary => 10000, // +100%
        }
    }
    
    /// Returns the XP multiplier (basis points)
    pub fn xp_multiplier(&self) -> u16 {
        match self {
            StakingTier::None => 0,
            StakingTier::Basic => 1000,     // +10%
            StakingTier::Premium => 2000,   // +20%
            StakingTier::VIP => 3000,       // +30%
            StakingTier::Elite => 5000,     // +50%
            StakingTier::Legendary => 7500, // +75%
        }
    }
    
    /// Returns the RP bonus multiplier (basis points)
    pub fn rp_bonus(&self) -> u16 {
        match self {
            StakingTier::None => 0,
            StakingTier::Basic => 500,      // +5%
            StakingTier::Premium => 1000,   // +10%
            StakingTier::VIP => 2000,       // +20%
            StakingTier::Elite => 3500,     // +35%
            StakingTier::Legendary => 5000, // +50%
        }
    }
}

/// Reward distribution configuration
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct RewardDistribution {
    /// Mining rewards percentage (basis points)
    pub mining_rewards: u16,
    /// XP bonuses percentage (basis points)  
    pub xp_bonuses: u16,
    /// RP network rewards percentage (basis points)
    pub rp_rewards: u16,
    /// Special events percentage (basis points)
    pub special_events: u16,
    /// Treasury reserve percentage (basis points)
    pub treasury_reserve: u16,
}

impl Default for RewardDistribution {
    fn default() -> Self {
        Self {
            mining_rewards: 4000,    // 40%
            xp_bonuses: 2500,        // 25%
            rp_rewards: 2000,        // 20%
            special_events: 1000,    // 10%
            treasury_reserve: 500,   // 5%
        }
    }
}

impl RewardDistribution {
    /// Validates that all percentages sum to 100%
    pub fn validate(&self) -> Result<()> {
        let total = self.mining_rewards + self.xp_bonuses + self.rp_rewards + 
                   self.special_events + self.treasury_reserve;
        
        require!(total == 10000, crate::errors::TokenError::InvalidRewardDistribution);
        Ok(())
    }
}

/// Token economics configuration
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct TokenEconomics {
    /// Maximum supply for FIN token
    pub max_supply: u64,
    /// Current circulating supply
    pub circulating_supply: u64,
    /// Total tokens staked
    pub total_staked: u64,
    /// Burn rate per transaction (basis points)
    pub burn_rate: u16,
    /// Whale tax threshold
    pub whale_threshold: u64,
    /// Progressive whale tax rates
    pub whale_tax_rates: [u16; 5],
    /// Current mining phase
    pub mining_phase: u8,
    /// Base mining rate for current phase
    pub base_mining_rate: u64,
}

impl Default for TokenEconomics {
    fn default() -> Self {
        Self {
            max_supply: 100_000_000_000 * 1_000_000_000, // 100B FIN with 9 decimals
            circulating_supply: 0,
            total_staked: 0,
            burn_rate: 10,                    // 0.1%
            whale_threshold: 100_000 * 1_000_000_000, // 100K FIN
            whale_tax_rates: [100, 200, 300, 500, 1000], // 1%, 2%, 3%, 5%, 10%
            mining_phase: 1,
            base_mining_rate: 100_000_000,    // 0.1 FIN/hour
        }
    }
}

impl TokenEconomics {
    /// Updates circulating supply
    pub fn update_circulating_supply(&mut self, change: i64) -> Result<()> {
        if change < 0 {
            let decrease = (-change) as u64;
            require!(
                self.circulating_supply >= decrease,
                crate::errors::TokenError::InsufficientSupply
            );
            self.circulating_supply -= decrease;
        } else {
            let increase = change as u64;
            require!(
                self.circulating_supply + increase <= self.max_supply,
                crate::errors::TokenError::ExceedsMaxSupply
            );
            self.circulating_supply += increase;
        }
        Ok(())
    }
    
    /// Calculates whale tax for a given holding amount
    pub fn calculate_whale_tax(&self, amount: u64) -> u64 {
        if amount < self.whale_threshold {
            return 0;
        }
        
        let excess = amount - self.whale_threshold;
        let tier = (excess / (50_000 * 1_000_000_000)).min(4) as usize; // 50K FIN tiers
        let tax_rate = self.whale_tax_rates[tier];
        
        (excess * tax_rate as u64) / 10000
    }
    
    /// Updates mining phase based on user count
    pub fn update_mining_phase(&mut self, total_users: u64) -> Result<()> {
        let new_phase = match total_users {
            0..=100_000 => 1,
            100_001..=1_000_000 => 2,
            1_000_001..=10_000_000 => 3,
            _ => 4,
        };
        
        if new_phase != self.mining_phase {
            self.mining_phase = new_phase;
            self.base_mining_rate = match new_phase {
                1 => 100_000_000,  // 0.1 FIN/hour
                2 => 50_000_000,   // 0.05 FIN/hour
                3 => 25_000_000,   // 0.025 FIN/hour
                4 => 10_000_000,   // 0.01 FIN/hour
                _ => 10_000_000,
            };
        }
        
        Ok(())
    }
}

/// Vesting schedule configuration
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct VestingSchedule {
    /// Total amount to be vested
    pub total_amount: u64,
    /// Amount already released
    pub released_amount: u64,
    /// Vesting start timestamp
    pub start_time: i64,
    /// Vesting duration in seconds
    pub duration: i64,
    /// Cliff period in seconds
    pub cliff_duration: i64,
    /// Linear or stepped vesting
    pub is_linear: bool,
    /// Vesting intervals for stepped vesting
    pub intervals: Vec<VestingInterval>,
}

/// Individual vesting interval
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct VestingInterval {
    /// Timestamp when this interval unlocks
    pub unlock_time: i64,
    /// Percentage of total amount (basis points)
    pub percentage: u16,
    /// Whether this interval has been claimed
    pub claimed: bool,
}

impl VestingSchedule {
    /// Calculates how much can be released at current time
    pub fn releasable_amount(&self, current_time: i64) -> u64 {
        if current_time < self.start_time + self.cliff_duration {
            return 0;
        }
        
        if self.is_linear {
            let elapsed = current_time - self.start_time;
            if elapsed >= self.duration {
                return self.total_amount - self.released_amount;
            }
            
            let vested_amount = (self.total_amount * elapsed as u64) / self.duration as u64;
            if vested_amount > self.released_amount {
                vested_amount - self.released_amount
            } else {
                0
            }
        } else {
            let mut releasable = 0u64;
            for interval in &self.intervals {
                if current_time >= interval.unlock_time && !interval.claimed {
                    releasable += (self.total_amount * interval.percentage as u64) / 10000;
                }
            }
            releasable
        }
    }
    
    /// Marks vesting intervals as claimed
    pub fn mark_claimed(&mut self, amount: u64, current_time: i64) -> Result<()> {
        if !self.is_linear {
            let mut remaining = amount;
            for interval in &mut self.intervals {
                if current_time >= interval.unlock_time && !interval.claimed && remaining > 0 {
                    let interval_amount = (self.total_amount * interval.percentage as u64) / 10000;
                    if remaining >= interval_amount {
                        interval.claimed = true;
                        remaining -= interval_amount;
                    }
                }
            }
        }
        
        self.released_amount += amount;
        Ok(())
    }
}

/// Fee configuration for various operations
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct FeeConfig {
    /// Transfer fee (basis points)
    pub transfer_fee: u16,
    /// Staking fee (basis points)
    pub staking_fee: u16,
    /// Unstaking fee (basis points)
    pub unstaking_fee: u16,
    /// Early unstaking penalty (basis points)
    pub early_unstaking_penalty: u16,
    /// NFT trading fee (basis points)
    pub nft_trading_fee: u16,
    /// DEX swap fee (basis points)
    pub dex_swap_fee: u16,
}

impl Default for FeeConfig {
    fn default() -> Self {
        Self {
            transfer_fee: 10,           // 0.1%
            staking_fee: 50,            // 0.5%
            unstaking_fee: 50,          // 0.5%
            early_unstaking_penalty: 500, // 5%
            nft_trading_fee: 250,       // 2.5%
            dex_swap_fee: 30,           // 0.3%
        }
    }
}

/// Emergency controls configuration
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct EmergencyControls {
    /// Whether the protocol is paused
    pub paused: bool,
    /// Pause timestamp
    pub pause_timestamp: i64,
    /// Emergency admin pubkey
    pub emergency_admin: Pubkey,
    /// Maximum pause duration (seconds)
    pub max_pause_duration: i64,
    /// Circuit breaker thresholds
    pub circuit_breaker: CircuitBreakerConfig,
}

/// Circuit breaker configuration
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct CircuitBreakerConfig {
    /// Maximum percentage of supply that can be transferred in 24h
    pub max_daily_transfer_percentage: u16,
    /// Maximum percentage of supply that can be staked in 24h
    pub max_daily_stake_percentage: u16,
    /// Maximum percentage of supply that can be unstaked in 24h
    pub max_daily_unstake_percentage: u16,
    /// Time window for tracking (seconds)
    pub time_window: i64,
    /// Current 24h transfer volume
    pub daily_transfer_volume: u64,
    /// Current 24h stake volume
    pub daily_stake_volume: u64,
    /// Current 24h unstake volume
    pub daily_unstake_volume: u64,
    /// Last reset timestamp
    pub last_reset: i64,
}

impl CircuitBreakerConfig {
    /// Checks if transfer would exceed daily limits
    pub fn check_transfer_limit(&self, amount: u64, total_supply: u64) -> Result<()> {
        let max_daily = (total_supply * self.max_daily_transfer_percentage as u64) / 10000;
        require!(
            self.daily_transfer_volume + amount <= max_daily,
            crate::errors::TokenError::DailyTransferLimitExceeded
        );
        Ok(())
    }
    
    /// Checks if stake would exceed daily limits
    pub fn check_stake_limit(&self, amount: u64, total_supply: u64) -> Result<()> {
        let max_daily = (total_supply * self.max_daily_stake_percentage as u64) / 10000;
        require!(
            self.daily_stake_volume + amount <= max_daily,
            crate::errors::TokenError::DailyStakeLimitExceeded
        );
        Ok(())
    }
    
    /// Resets daily volumes if time window has passed
    pub fn maybe_reset_daily_volumes(&mut self, current_time: i64) {
        if current_time - self.last_reset >= self.time_window {
            self.daily_transfer_volume = 0;
            self.daily_stake_volume = 0;
            self.daily_unstake_volume = 0;
            self.last_reset = current_time;
        }
    }
}

/// Global token program state
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct TokenProgramState {
    /// Program version
    pub version: u8,
    /// Program authority
    pub authority: Pubkey,
    /// Token economics configuration
    pub economics: TokenEconomics,
    /// Reward distribution configuration
    pub reward_distribution: RewardDistribution,
    /// Fee configuration
    pub fee_config: FeeConfig,
    /// Emergency controls
    pub emergency_controls: EmergencyControls,
    /// Last update timestamp
    pub last_update: i64,
    /// Program initialization timestamp
    pub initialized_at: i64,
}

impl TokenProgramState {
    pub const SIZE: usize = 8 + // discriminator
        1 + // version
        32 + // authority
        std::mem::size_of::<TokenEconomics>() +
        std::mem::size_of::<RewardDistribution>() +
        std::mem::size_of::<FeeConfig>() +
        std::mem::size_of::<EmergencyControls>() +
        8 + // last_update
        8 + // initialized_at
        64; // padding
}

impl Default for TokenProgramState {
    fn default() -> Self {
        Self {
            version: 1,
            authority: Pubkey::default(),
            economics: TokenEconomics::default(),
            reward_distribution: RewardDistribution::default(),
            fee_config: FeeConfig::default(),
            emergency_controls: EmergencyControls {
                paused: false,
                pause_timestamp: 0,
                emergency_admin: Pubkey::default(),
                max_pause_duration: 86400 * 7, // 7 days
                circuit_breaker: CircuitBreakerConfig {
                    max_daily_transfer_percentage: 1000, // 10%
                    max_daily_stake_percentage: 500,     // 5%
                    max_daily_unstake_percentage: 200,   // 2%
                    time_window: 86400,                  // 24 hours
                    daily_transfer_volume: 0,
                    daily_stake_volume: 0,
                    daily_unstake_volume: 0,
                    last_reset: 0,
                },
            },
            last_update: 0,
            initialized_at: 0,
        }
    }
}

impl FinovaTokenState for TokenProgramState {
    fn validate(&self) -> Result<()> {
        require!(self.version > 0, crate::errors::TokenError::InvalidVersion);
        require!(self.authority != Pubkey::default(), crate::errors::TokenError::InvalidAuthority);
        self.reward_distribution.validate()?;
        Ok(())
    }
    
    fn version(&self) -> u8 {
        self.version
    }
    
    fn account_type(&self) -> AccountType {
        AccountType::MintInfo
    }
}

/// Utility functions for token state management
pub mod utils {
    use super::*;
    
    /// Calculates compound interest
    pub fn calculate_compound_interest(
        principal: u64,
        rate: u16, // APY in basis points
        time_elapsed: i64, // in seconds
    ) -> u64 {
        if rate == 0 || time_elapsed <= 0 {
            return 0;
        }
        
        let annual_rate = rate as f64 / 10000.0;
        let years = time_elapsed as f64 / (365.25 * 24.0 * 3600.0);
        let compound_factor = (1.0 + annual_rate).powf(years);
        
        ((principal as f64 * compound_factor) as u64).saturating_sub(principal)
    }
    
    /// Applies exponential decay for anti-whale mechanism
    pub fn apply_exponential_decay(base_amount: u64, holdings: u64, decay_factor: f64) -> u64 {
        if decay_factor <= 0.0 || holdings == 0 {
            return base_amount;
        }
        
        let decay = (-decay_factor * holdings as f64 / 1_000_000_000.0).exp();
        (base_amount as f64 * decay) as u64
    }
    
    /// Calculates time-weighted average
    pub fn calculate_time_weighted_average(
        values: &[(u64, i64)], // (value, timestamp) pairs
        current_time: i64,
        window_duration: i64,
    ) -> u64 {
        if values.is_empty() {
            return 0;
        }
        
        let cutoff_time = current_time - window_duration;
        let relevant_values: Vec<_> = values
            .iter()
            .filter(|(_, timestamp)| *timestamp >= cutoff_time)
            .collect();
        
        if relevant_values.is_empty() {
            return values.last().unwrap().0;
        }
        
        let total_weight: i64 = relevant_values
            .windows(2)
            .map(|w| w[1].1 - w[0].1)
            .sum();
        
        if total_weight == 0 {
            return relevant_values.last().unwrap().0;
        }
        
        let weighted_sum: u64 = relevant_values
            .windows(2)
            .map(|w| w[0].0 * (w[1].1 - w[0].1) as u64)
            .sum();
        
        weighted_sum / total_weight as u64
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_staking_tier_from_amount() {
        assert_eq!(StakingTier::from_amount(50), StakingTier::None);
        assert_eq!(StakingTier::from_amount(150), StakingTier::Basic);
        assert_eq!(StakingTier::from_amount(750), StakingTier::Premium);
        assert_eq!(StakingTier::from_amount(2500), StakingTier::VIP);
        assert_eq!(StakingTier::from_amount(7500), StakingTier::Elite);
        assert_eq!(StakingTier::from_amount(15000), StakingTier::Legendary);
    }
    
    #[test]
    fn test_token_economics_whale_tax() {
        let economics = TokenEconomics::default();
        assert_eq!(economics.calculate_whale_tax(50_000 * 1_000_000_000), 0);
        assert_eq!(
            economics.calculate_whale_tax(150_000 * 1_000_000_000),
            (50_000 * 1_000_000_000 * 100) / 10000
        );
    }
    
    #[test]
    fn test_compound_interest_calculation() {
        let interest = utils::calculate_compound_interest(
            1000 * 1_000_000_000, // 1000 FIN
            1000,                  // 10% APY
            365 * 24 * 3600,      // 1 year in seconds
        );
        assert!(interest > 95 * 1_000_000_000); // Should be close to 100 FIN
        assert!(interest < 105 * 1_000_000_000);
    }
    
    #[test]
    fn test_reward_distribution_validation() {
        let valid_distribution = RewardDistribution::default();
        assert!(valid_distribution.validate().is_ok());
        
        let invalid_distribution = RewardDistribution {
            mining_rewards: 5000,
            xp_bonuses: 3000,
            rp_rewards: 3000,
            special_events: 1000,
            treasury_reserve: 1000, // Total = 13000 (130%), should fail
        };
        assert!(invalid_distribution.validate().is_err());
    }
}
