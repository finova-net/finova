// programs/finova-core/src/state/staking.rs

use anchor_lang::prelude::*;

#[account]
#[derive(Default)]
pub struct StakingPool {
    /// Pool authority
    pub authority: Pubkey,
    
    /// Pool creation timestamp
    pub created_at: i64,
    
    /// Total FIN tokens staked in pool
    pub total_staked: u64,
    
    /// Total sFIN tokens minted
    pub total_sfin_supply: u64,
    
    /// Current exchange rate (FIN to sFIN) stored as rate * 1e6
    pub exchange_rate: u64,
    
    /// Annual Percentage Yield (APY) stored as percentage * 100
    pub current_apy: u16,
    
    /// Base APY before multipliers
    pub base_apy: u16,
    
    /// Total reward reserves in pool
    pub reward_reserves: u64,
    
    /// Rewards distributed today
    pub daily_rewards_distributed: u64,
    
    /// Daily reward distribution limit
    pub daily_reward_limit: u64,
    
    /// Last reward distribution timestamp
    pub last_reward_distribution: i64,
    
    /// Reward distribution interval (seconds)
    pub reward_interval: u64,
    
    /// Minimum staking amount
    pub min_stake_amount: u64,
    
    /// Maximum staking amount per user
    pub max_stake_per_user: u64,
    
    /// Unstaking cooldown period (seconds)
    pub unstaking_cooldown: u64,
    
    /// Early unstaking penalty (percentage * 100)
    pub early_unstaking_penalty: u16,
    
    /// Pool utilization ratio (percentage * 100)
    pub utilization_ratio: u16,
    
    /// Staking tiers configuration
    pub staking_tiers: Vec<StakingTier>,
    
    /// Pool statistics
    pub pool_stats: StakingPoolStats,
    
    /// Emergency pause status
    pub paused: bool,
    
    /// Pause reason
    pub pause_reason: u8,
    
    /// Reserved space for future upgrades
    pub _reserved: [u8; 64],
}

#[account]
#[derive(Default)]
pub struct UserStakeAccount {
    /// User's public key
    pub user: Pubkey,
    
    /// Stake account creation timestamp
    pub created_at: i64,
    
    /// Amount of FIN tokens staked
    pub staked_amount: u64,
    
    /// Amount of sFIN tokens held
    pub sfin_balance: u64,
    
    /// Staking start timestamp
    pub staking_started_at: i64,
    
    /// Last reward claim timestamp
    pub last_reward_claim: i64,
    
    /// Total rewards earned
    pub total_rewards_earned: u64,
    
    /// Unclaimed rewards
    pub unclaimed_rewards: u64,
    
    /// Current staking tier (0-4)
    pub staking_tier: u8,
    
    /// Staking tier benefits active
    pub tier_benefits: StakingTierBenefits,
    
    /// Auto-compound enabled
    pub auto_compound: bool,
    
    /// Auto-compound frequency (seconds)
    pub compound_frequency: u64,
    
    /// Last auto-compound timestamp
    pub last_compound: i64,
    
    /// Staking multipliers applied
    pub multipliers: StakingMultipliers,
    
    /// Unstaking requests
    pub unstaking_requests: Vec<UnstakingRequest>,
    
    /// Total amount pending unstaking
    pub pending_unstake: u64,
    
    /// Staking achievements
    pub staking_achievements: Vec<StakingAchievement>,
    
    /// Loyalty bonus accumulation
    pub loyalty_bonus: LoyaltyBonus,
    
    /// Performance metrics
    pub performance_metrics: StakingPerformanceMetrics,
    
    /// Staking history
    pub staking_history: Vec<StakingTransaction>,
    
    /// Reserved space
    pub _reserved: [u8; 32],
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default)]
pub struct StakingTier {
    /// Tier level (0-4)
    pub tier: u8,
    
    /// Minimum staking amount for this tier
    pub min_amount: u64,
    
    /// Maximum staking amount for this tier
    pub max_amount: u64,
    
    /// APY for this tier (percentage * 100)
    pub apy: u16,
    
    /// Mining boost percentage
    pub mining_boost: u16,
    
    /// XP multiplier bonus
    pub xp_multiplier: u16,
    
    /// RP bonus percentage
    pub rp_bonus: u16,
    
    /// Special features unlocked
    pub special_features: u32, // Bitmask
    
    /// Tier name
    pub name: String,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default)]
pub struct StakingPoolStats {
    /// Total number of stakers
    pub total_stakers: u32,
    
    /// Active stakers (staked in last 30 days)
    pub active_stakers: u32,
    
    /// Average stake amount
    pub average_stake: u64,
    
    /// Largest single stake
    pub largest_stake: u64,
    
    /// Total rewards distributed all-time
    pub total_rewards_distributed: u64,
    
    /// Average staking duration (seconds)
    pub average_staking_duration: u64,
    
    /// Staker retention rate (percentage * 100)
    pub retention_rate: u16,
    
    /// Pool growth rate (percentage * 100)
    pub growth_rate: i16, // Can be negative
    
    /// Daily staking volume
    pub daily_staking_volume: u64,
    
    /// Daily unstaking volume
    pub daily_unstaking_volume: u64,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default)]
pub struct StakingTierBenefits {
    /// Mining rate boost active
    pub mining_boost_active: bool,
    
    /// XP multiplier active
    pub xp_boost_active: bool,
    
    /// RP bonus active
    pub rp_boost_active: bool,
    
    /// VIP features access
    pub vip_access: bool,
    
    /// Guild leadership privileges
    pub guild_privileges: bool,
    
    /// DAO governance weight
    pub governance_weight: u32,
    
    /// Priority support access
    pub priority_support: bool,
    
    /// Exclusive events access
    pub exclusive_events: bool,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default)]
pub struct StakingMultipliers {
    /// XP level multiplier bonus
    pub xp_level_bonus: u16,
    
    /// RP tier multiplier bonus
    pub rp_tier_bonus: u16,
    
    /// Loyalty multiplier bonus
    pub loyalty_bonus: u16,
    
    /// Activity multiplier bonus
    pub activity_bonus: u16,
    
    /// Guild participation bonus
    pub guild_bonus: u16,
    
    /// Achievement bonus
    pub achievement_bonus: u16,
    
    /// Total combined multiplier
    pub total_multiplier: u32,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default)]
pub struct UnstakingRequest {
    /// Request ID
    pub request_id: u64,
    
    /// Amount to unstake
    pub amount: u64,
    
    /// Request timestamp
    pub requested_at: i64,
    
    /// Available for withdrawal timestamp
    pub available_at: i64,
    
    /// Early unstaking penalty applied
    pub penalty_applied: u16,
    
    /// Penalty amount
    pub penalty_amount: u64,
    
    /// Request status (pending, ready, completed, cancelled)
    pub status: u8,
    
    /// Reason for unstaking
    pub reason: u8,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default)]
pub struct StakingAchievement {
    /// Achievement ID
    pub achievement_id: u32,
    
    /// Achievement earned timestamp
    pub earned_at: i64,
    
    /// Achievement tier
    pub tier: u8,
    
    /// Staking bonus earned (percentage * 100)
    pub staking_bonus: u16,
    
    /// Achievement description
    pub description: String,
    
    /// Achievement expiry (if temporary)
    pub expires_at: Option<i64>,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default)]
pub struct LoyaltyBonus {
    /// Loyalty level (0-10)
    pub loyalty_level: u8,
    
    /// Consecutive staking months
    pub consecutive_months: u32,
    
    /// Loyalty points accumulated
    pub loyalty_points: u64,
    
    /// Loyalty bonus multiplier (percentage * 100)
    pub bonus_multiplier: u16,
    
    /// Last loyalty update
    pub last_update: i64,
    
    /// Loyalty milestones achieved
    pub milestones_achieved: Vec<u32>,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default)]
pub struct StakingPerformanceMetrics {
    /// Total return on stake (percentage * 100)
    pub total_return: u32,
    
    /// Annualized return (percentage * 100)
    pub annualized_return: u16,
    
    /// Compound growth rate
    pub compound_growth: u16,
    
    /// Efficiency score (0-10000)
    pub efficiency_score: u16,
    
    /// Risk-adjusted return
    pub risk_adjusted_return: u16,
    
    /// Best month performance
    pub best_month_return: u16,
    
    /// Worst month performance
    pub worst_month_return: u16,
    
    /// Consistency score (0-10000)
    pub consistency_score: u16,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default)]
pub struct StakingTransaction {
    /// Transaction type (stake, unstake, claim, compound)
    pub tx_type: u8,
    
    /// Transaction timestamp
    pub timestamp: i64,
    
    /// Amount involved
    pub amount: u64,
    
    /// Exchange rate at time of transaction
    pub exchange_rate: u64,
    
    /// APY at time of transaction
    pub apy_at_time: u16,
    
    /// Transaction hash
    pub tx_hash: Vec<u8>,
    
    /// Gas fees paid
    pub gas_fees: u64,
}

impl StakingPool {
    pub const LEN: usize = 8 + // discriminator
        32 + // authority
        8 + // created_at
        8 + // total_staked
        8 + // total_sfin_supply
        8 + // exchange_rate
        2 + // current_apy
        2 + // base_apy
        8 + // reward_reserves
        8 + // daily_rewards_distributed
        8 + // daily_reward_limit
        8 + // last_reward_distribution
        8 + // reward_interval
        8 + // min_stake_amount
        8 + // max_stake_per_user
        8 + // unstaking_cooldown
        2 + // early_unstaking_penalty
        2 + // utilization_ratio
        4 + 5 * 64 + // staking_tiers (5 tiers)
        128 + // pool_stats
        1 + // paused
        1 + // pause_reason
        64; // _reserved
    
    /// Initialize staking pool
    pub fn initialize(&mut self, authority: Pubkey, clock: &Clock) -> Result<()> {
        self.authority = authority;
        self.created_at = clock.unix_timestamp;
        self.total_sfin_supply = 0;
        self.exchange_rate = 1_000_000; // 1:1 initial rate (stored as 1e6)
        self.current_apy = 1000; // 10% APY (stored as percentage * 100)
        self.base_apy = 800; // 8% base APY
        self.reward_reserves = 0;
        self.daily_rewards_distributed = 0;
        self.daily_reward_limit = 100_000_000_000; // 100,000 FIN per day
        self.last_reward_distribution = clock.unix_timestamp;
        self.reward_interval = 3600; // 1 hour
        self.min_stake_amount = 100_000_000; // 100 FIN minimum
        self.max_stake_per_user = 10_000_000_000_000; // 10M FIN maximum
        self.unstaking_cooldown = 604800; // 7 days
        self.early_unstaking_penalty = 500; // 5% penalty
        self.utilization_ratio = 0;
        self.paused = false;
        self.pause_reason = 0;
        
        // Initialize staking tiers
        self.initialize_staking_tiers();
        
        // Initialize pool stats
        self.pool_stats = StakingPoolStats::default();
        
        Ok(())
    }
    
    /// Initialize the 5 staking tiers
    fn initialize_staking_tiers(&mut self) {
        self.staking_tiers = vec![
            StakingTier {
                tier: 0,
                min_amount: 100_000_000, // 100 FIN
                max_amount: 499_000_000, // 499 FIN
                apy: 800, // 8%
                mining_boost: 2000, // +20%
                xp_multiplier: 1000, // +10%
                rp_bonus: 500, // +5%
                special_features: 0b0001, // Basic features
                name: "Bronze Staker".to_string(),
            },
            StakingTier {
                tier: 1,
                min_amount: 500_000_000, // 500 FIN
                max_amount: 999_000_000, // 999 FIN
                apy: 1000, // 10%
                mining_boost: 3500, // +35%
                xp_multiplier: 2000, // +20%
                rp_bonus: 1000, // +10%
                special_features: 0b0011, // Premium badge + priority support
                name: "Silver Staker".to_string(),
            },
            StakingTier {
                tier: 2,
                min_amount: 1_000_000_000, // 1,000 FIN
                max_amount: 4_999_000_000, // 4,999 FIN
                apy: 1200, // 12%
                mining_boost: 5000, // +50%
                xp_multiplier: 3000, // +30%
                rp_bonus: 2000, // +20%
                special_features: 0b0111, // VIP features + exclusive events
                name: "Gold Staker".to_string(),
            },
            StakingTier {
                tier: 3,
                min_amount: 5_000_000_000, // 5,000 FIN
                max_amount: 9_999_000_000, // 9,999 FIN
                apy: 1400, // 14%
                mining_boost: 7500, // +75%
                xp_multiplier: 5000, // +50%
                rp_bonus: 3500, // +35%
                special_features: 0b1111, // Guild master privileges
                name: "Platinum Staker".to_string(),
            },
            StakingTier {
                tier: 4,
                min_amount: 10_000_000_000, // 10,000 FIN
                max_amount: u64::MAX,
                apy: 1500, // 15%
                mining_boost: 10000, // +100%
                xp_multiplier: 7500, // +75%
                rp_bonus: 5000, // +50%
                special_features: 0b11111, // DAO governance + max benefits
                name: "Diamond Staker".to_string(),
            },
        ];
    }
    
    /// Get staking tier for given amount
    pub fn get_staking_tier(&self, amount: u64) -> u8 {
        for tier in &self.staking_tiers {
            if amount >= tier.min_amount && amount <= tier.max_amount {
                return tier.tier;
            }
        }
        0 // Default to Bronze tier
    }
    
    /// Calculate exchange rate (FIN to sFIN)
    pub fn calculate_exchange_rate(&self) -> u64 {
        if self.total_sfin_supply == 0 {
            return self.exchange_rate; // Initial rate
        }
        
        // Exchange rate increases as rewards accumulate
        // Rate = (total_staked + accumulated_rewards) / total_sfin_supply
        let total_value = self.total_staked + self.calculate_accumulated_rewards();
        (total_value * 1_000_000 / self.total_sfin_supply)
    }
    
    /// Calculate accumulated rewards not yet distributed
    fn calculate_accumulated_rewards(&self) -> u64 {
        // This would calculate pending rewards based on time elapsed
        // Simplified calculation for now
        let time_elapsed = std::cmp::max(0, chrono::Utc::now().timestamp() - self.last_reward_distribution);
        let hours_elapsed = time_elapsed / 3600;
        
        if hours_elapsed <= 0 {
            return 0;
        }
        
        // Calculate hourly rewards: total_staked * APY / (365 * 24)
        let hourly_rate = self.current_apy as u128 * self.total_staked as u128 / (365 * 24 * 10000) as u128;
        (hourly_rate * hours_elapsed as u128) as u64
    }
    
    /// Update exchange rate and distribute rewards
    pub fn distribute_rewards(&mut self, clock: &Clock) -> Result<u64> {
        if clock.unix_timestamp < self.last_reward_distribution + self.reward_interval as i64 {
            return Ok(0); // Too early for next distribution
        }
        
        let rewards_to_distribute = self.calculate_accumulated_rewards();
        
        if rewards_to_distribute == 0 || rewards_to_distribute > self.reward_reserves {
            return Ok(0); // No rewards or insufficient reserves
        }
        
        // Check daily limit
        let today_start = clock.unix_timestamp - (clock.unix_timestamp % 86400);
        if self.last_reward_distribution < today_start {
            self.daily_rewards_distributed = 0; // Reset daily counter
        }
        
        if self.daily_rewards_distributed + rewards_to_distribute > self.daily_reward_limit {
            return Ok(0); // Daily limit exceeded
        }
        
        // Distribute rewards by updating exchange rate
        self.exchange_rate = self.calculate_exchange_rate();
        self.reward_reserves -= rewards_to_distribute;
        self.daily_rewards_distributed += rewards_to_distribute;
        self.last_reward_distribution = clock.unix_timestamp;
        
        // Update pool stats
        self.pool_stats.total_rewards_distributed += rewards_to_distribute;
        
        Ok(rewards_to_distribute)
    }
    
    /// Add rewards to the pool
    pub fn add_rewards(&mut self, amount: u64) -> Result<()> {
        self.reward_reserves = self.reward_reserves.checked_add(amount).unwrap();
        Ok(())
    }
    
    /// Process staking
    pub fn process_stake(&mut self, amount: u64, clock: &Clock) -> Result<u64> {
        if self.paused {
            return Err(ErrorCode::StakingPaused.into());
        }
        
        if amount < self.min_stake_amount {
            return Err(ErrorCode::StakeAmountTooLow.into());
        }
        
        // Update exchange rate before processing
        self.distribute_rewards(clock)?;
        
        // Calculate sFIN tokens to mint
        let sfin_to_mint = (amount * 1_000_000) / self.exchange_rate;
        
        // Update pool state
        self.total_staked = self.total_staked.checked_add(amount).unwrap();
        self.total_sfin_supply = self.total_sfin_supply.checked_add(sfin_to_mint).unwrap();
        
        // Update pool stats
        self.pool_stats.total_stakers += 1;
        self.pool_stats.daily_staking_volume = self.pool_stats.daily_staking_volume.checked_add(amount).unwrap();
        
        if amount > self.pool_stats.largest_stake {
            self.pool_stats.largest_stake = amount;
        }
        
        // Update utilization ratio
        self.update_utilization_ratio();
        
        Ok(sfin_to_mint)
    }
    
    /// Process unstaking request
    pub fn process_unstake_request(&mut self, sfin_amount: u64, clock: &Clock) -> Result<(u64, u64)> {
        if self.paused {
            return Err(ErrorCode::StakingPaused.into());
        }
        
        // Update exchange rate before processing
        self.distribute_rewards(clock)?;
        
        // Calculate FIN tokens to return
        let fin_to_return = (sfin_amount * self.exchange_rate) / 1_000_000;
        
        // Calculate early unstaking penalty if applicable
        // This would check individual user's staking duration
        let penalty_amount = 0; // Simplified - would calculate based on user's stake duration
        let net_amount = fin_to_return - penalty_amount;
        
        // Update pool state
        self.total_staked = self.total_staked.saturating_sub(net_amount);
        self.total_sfin_supply = self.total_sfin_supply.saturating_sub(sfin_amount);
        
        // Update pool stats
        self.pool_stats.daily_unstaking_volume = self.pool_stats.daily_unstaking_volume.checked_add(net_amount).unwrap();
        
        // Update utilization ratio
        self.update_utilization_ratio();
        
        Ok((net_amount, penalty_amount))
    }
    
    /// Update utilization ratio
    fn update_utilization_ratio(&mut self) {
        if self.reward_reserves > 0 {
            self.utilization_ratio = ((self.total_staked * 10000) / (self.total_staked + self.reward_reserves)) as u16;
        }
    }
    
    /// Calculate dynamic APY based on utilization
    pub fn calculate_dynamic_apy(&mut self) -> u16 {
        let base_apy = self.base_apy as u32;
        let utilization = self.utilization_ratio as u32;
        
        // APY increases with utilization to incentivize staking
        let dynamic_adjustment = if utilization > 8000 { // > 80% utilization
            (utilization - 8000) * 5 / 1000 // Up to +1% bonus
        } else if utilization < 2000 { // < 20% utilization
            (2000 - utilization) * 2 / 1000 // Up to -0.4% penalty
        } else {
            0
        };
        
        self.current_apy = (base_apy + dynamic_adjustment) as u16;
        self.current_apy
    }
    
    /// Emergency pause staking
    pub fn emergency_pause(&mut self, reason: u8) -> Result<()> {
        self.paused = true;
        self.pause_reason = reason;
        Ok(())
    }
    
    /// Resume staking
    pub fn resume_staking(&mut self) -> Result<()> {
        self.paused = false;
        self.pause_reason = 0;
        Ok(())
    }
}

impl UserStakeAccount {
    pub const LEN: usize = 8 + // discriminator
        32 + // user
        8 + // created_at
        8 + // staked_amount
        8 + // sfin_balance
        8 + // staking_started_at
        8 + // last_reward_claim
        8 + // total_rewards_earned
        8 + // unclaimed_rewards
        1 + // staking_tier
        64 + // tier_benefits
        1 + // auto_compound
        8 + // compound_frequency
        8 + // last_compound
        64 + // multipliers
        4 + 5 * 64 + // unstaking_requests (max 5)
        8 + // pending_unstake
        4 + 10 * 64 + // staking_achievements (max 10)
        64 + // loyalty_bonus
        64 + // performance_metrics
        4 + 20 * 64 + // staking_history (max 20)
        32; // _reserved
    
    /// Initialize user stake account
    pub fn initialize(&mut self, user: Pubkey, clock: &Clock) -> Result<()> {
        self.user = user;
        self.created_at = clock.unix_timestamp;
        self.staked_amount = 0;
        self.sfin_balance = 0;
        self.staking_started_at = 0;
        self.last_reward_claim = clock.unix_timestamp;
        self.total_rewards_earned = 0;
        self.unclaimed_rewards = 0;
        self.staking_tier = 0;
        self.tier_benefits = StakingTierBenefits::default();
        self.auto_compound = false;
        self.compound_frequency = 86400; // Daily by default
        self.last_compound = clock.unix_timestamp;
        self.multipliers = StakingMultipliers::default();
        self.unstaking_requests = Vec::new();
        self.pending_unstake = 0;
        self.staking_achievements = Vec::new();
        self.loyalty_bonus = LoyaltyBonus::default();
        self.performance_metrics = StakingPerformanceMetrics::default();
        self.staking_history = Vec::new();
        
        Ok(())
    }
    
    /// Process user stake
    pub fn stake(&mut self, amount: u64, sfin_received: u64, pool: &StakingPool, clock: &Clock) -> Result<()> {
        // First stake
        if self.staking_started_at == 0 {
            self.staking_started_at = clock.unix_timestamp;
        }
        
        // Update amounts
        self.staked_amount = self.staked_amount.checked_add(amount).unwrap();
        self.sfin_balance = self.sfin_balance.checked_add(sfin_received).unwrap();
        
        // Update staking tier
        self.update_staking_tier(pool);
        
        // Record transaction
        self.record_transaction(0, amount, pool.exchange_rate, pool.current_apy, clock)?;
        
        // Calculate and update multipliers
        self.update_multipliers()?;
        
        Ok(())
    }
    
    /// Process unstaking request
    pub fn request_unstake(&mut self, sfin_amount: u64, pool: &StakingPool, clock: &Clock) -> Result<u64> {
        if sfin_amount > self.sfin_balance {
            return Err(ErrorCode::InsufficientStakeBalance.into());
        }
        
        // Calculate penalty if early unstaking
        let staking_duration = clock.unix_timestamp - self.staking_started_at;
        let is_early = staking_duration < pool.unstaking_cooldown as i64;
        
        let penalty = if is_early {
            pool.early_unstaking_penalty
        } else {
            0
        };
        
        // Create unstaking request
        let request_id = self.unstaking_requests.len() as u64;
        let available_at = if is_early {
            clock.unix_timestamp + pool.unstaking_cooldown as i64
        } else {
            clock.unix_timestamp
        };
        
        let unstake_request = UnstakingRequest {
            request_id,
            amount: sfin_amount,
            requested_at: clock.unix_timestamp,
            available_at,
            penalty_applied: penalty,
            penalty_amount: (sfin_amount as u128 * penalty as u128 / 10000) as u64,
            status: 1, // Pending
            reason: 0, // User initiated
        };
        
        self.unstaking_requests.push(unstake_request);
        self.pending_unstake = self.pending_unstake.checked_add(sfin_amount).unwrap();
        
        Ok(request_id)
    }
    
    /// Complete unstaking for ready requests
    pub fn complete_unstaking(&mut self, request_id: u64, pool: &StakingPool, clock: &Clock) -> Result<(u64, u64)> {
        let request_index = self.unstaking_requests
            .iter()
            .position(|r| r.request_id == request_id)
            .ok_or(ErrorCode::UnstakeRequestNotFound)?;
        
        let request = &mut self.unstaking_requests[request_index];
        
        if request.status != 1 { // Not pending
            return Err(ErrorCode::InvalidUnstakeStatus.into());
        }
        
        if clock.unix_timestamp < request.available_at {
            return Err(ErrorCode::UnstakeCooldownNotMet.into());
        }
        
        // Calculate FIN to return
        let fin_amount = (request.amount * pool.exchange_rate) / 1_000_000;
        let net_amount = fin_amount - request.penalty_amount;
        
        // Update balances
        self.sfin_balance = self.sfin_balance.saturating_sub(request.amount);
        self.staked_amount = self.staked_amount.saturating_sub(net_amount);
        self.pending_unstake = self.pending_unstake.saturating_sub(request.amount);
        
        // Mark request as completed
        request.status = 3; // Completed
        
        // Update staking tier
        self.update_staking_tier(pool);
        
        // Record transaction
        self.record_transaction(1, net_amount, pool.exchange_rate, pool.current_apy, clock)?;
        
        Ok((net_amount, request.penalty_amount))
    }
    
    /// Calculate pending rewards
    pub fn calculate_pending_rewards(&self, pool: &StakingPool, clock: &Clock) -> u64 {
        if self.sfin_balance == 0 {
            return 0;
        }
        
        // Calculate time-based rewards
        let time_since_last_claim = clock.unix_timestamp - self.last_reward_claim;
        let hours_elapsed = time_since_last_claim / 3600;
        
        if hours_elapsed <= 0 {
            return 0;
        }
        
        // Base rewards calculation
        let current_value = (self.sfin_balance * pool.exchange_rate) / 1_000_000;
        let original_value = self.staked_amount;
        let base_rewards = current_value.saturating_sub(original_value);
        
        // Apply multipliers
        let total_multiplier = self.multipliers.total_multiplier;
        let multiplied_rewards = (base_rewards as u128 * total_multiplier as u128 / 10000) as u64;
        
        multiplied_rewards + self.unclaimed_rewards
    }
    
    /// Claim rewards
    pub fn claim_rewards(&mut self, pool: &StakingPool, clock: &Clock) -> Result<u64> {
        let rewards = self.calculate_pending_rewards(pool, clock);
        
        if rewards == 0 {
            return Ok(0);
        }
        
        // Update reward tracking
        self.total_rewards_earned = self.total_rewards_earned.checked_add(rewards).unwrap();
        self.unclaimed_rewards = 0;
        self.last_reward_claim = clock.unix_timestamp;
        
        // Record transaction
        self.record_transaction(2, rewards, pool.exchange_rate, pool.current_apy, clock)?;
        
        // Update performance metrics
        self.update_performance_metrics(rewards, pool.current_apy);
        
        // Check for achievements
        self.check_reward_achievements(clock)?;
        
        Ok(rewards)
    }
    
    /// Auto-compound rewards
    pub fn auto_compound(&mut self, pool: &StakingPool, clock: &Clock) -> Result<u64> {
        if !self.auto_compound {
            return Ok(0);
        }
        
        if clock.unix_timestamp < self.last_compound + self.compound_frequency as i64 {
            return Ok(0); // Too early for next compound
        }
        
        let rewards = self.calculate_pending_rewards(pool, clock);
        
        if rewards == 0 {
            return Ok(0);
        }
        
        // Convert rewards to additional sFIN
        let additional_sfin = (rewards * 1_000_000) / pool.exchange_rate;
        
        // Update balances
        self.sfin_balance = self.sfin_balance.checked_add(additional_sfin).unwrap();
        self.staked_amount = self.staked_amount.checked_add(rewards).unwrap();
        self.total_rewards_earned = self.total_rewards_earned.checked_add(rewards).unwrap();
        self.unclaimed_rewards = 0;
        self.last_compound = clock.unix_timestamp;
        
        // Record transaction
        self.record_transaction(3, rewards, pool.exchange_rate, pool.current_apy, clock)?;
        
        // Update staking tier (might have increased)
        self.update_staking_tier(pool);
        
        Ok(rewards)
    }
    
    /// Update staking tier based on current staked amount
    fn update_staking_tier(&mut self, pool: &StakingPool) {
        let new_tier = pool.get_staking_tier(self.staked_amount);
        
        if new_tier != self.staking_tier {
            self.staking_tier = new_tier;
            self.update_tier_benefits(pool);
        }
    }
    
    /// Update tier benefits based on current tier
    fn update_tier_benefits(&mut self, pool: &StakingPool) {
        if let Some(tier_config) = pool.staking_tiers.get(self.staking_tier as usize) {
            self.tier_benefits = StakingTierBenefits {
                mining_boost_active: tier_config.mining_boost > 0,
                xp_boost_active: tier_config.xp_multiplier > 0,
                rp_boost_active: tier_config.rp_bonus > 0,
                vip_access: tier_config.special_features & 0b0010 != 0,
                guild_privileges: tier_config.special_features & 0b1000 != 0,
                governance_weight: match self.staking_tier {
                    0 => 1,
                    1 => 2,
                    2 => 5,
                    3 => 10,
                    4 => 20,
                    _ => 1,
                },
                priority_support: self.staking_tier >= 1,
                exclusive_events: self.staking_tier >= 2,
            };
        }
    }
    
    /// Update all multipliers
    fn update_multipliers(&mut self) -> Result<()> {
        // Calculate individual multipliers (these would come from other systems)
        self.multipliers.xp_level_bonus = 1000; // Placeholder - would get from user XP level
        self.multipliers.rp_tier_bonus = 500; // Placeholder - would get from user RP tier
        self.multipliers.loyalty_bonus = self.loyalty_bonus.bonus_multiplier;
        self.multipliers.activity_bonus = 1000; // Placeholder - based on user activity
        self.multipliers.guild_bonus = 0; // Placeholder - based on guild participation
        self.multipliers.achievement_bonus = self.get_achievement_bonus();
        
        // Calculate total multiplier
        let base = 10000u32; // 1.0x base
        let total = base
            + self.multipliers.xp_level_bonus as u32
            + self.multipliers.rp_tier_bonus as u32
            + self.multipliers.loyalty_bonus as u32
            + self.multipliers.activity_bonus as u32
            + self.multipliers.guild_bonus as u32
            + self.multipliers.achievement_bonus as u32;
        
        self.multipliers.total_multiplier = total;
        
        Ok(())
    }
    
    /// Get total achievement bonus
    fn get_achievement_bonus(&self) -> u16 {
        self.staking_achievements
            .iter()
            .map(|a| a.staking_bonus)
            .sum()
    }
    
    /// Update loyalty bonus
    pub fn update_loyalty_bonus(&mut self, clock: &Clock) -> Result<()> {
        let staking_duration = clock.unix_timestamp - self.staking_started_at;
        let months_staked = (staking_duration / (30 * 86400)) as u32; // Approximate months
        
        if months_staked > self.loyalty_bonus.consecutive_months {
            self.loyalty_bonus.consecutive_months = months_staked;
            self.loyalty_bonus.loyalty_points += months_staked * 100; // 100 points per month
            
            // Update loyalty level (0-10)
            self.loyalty_bonus.loyalty_level = std::cmp::min(10, months_staked / 6) as u8;
            
            // Update bonus multiplier: 1% per month, up to 50%
            self.loyalty_bonus.bonus_multiplier = std::cmp::min(5000, months_staked * 100) as u16;
            
            self.loyalty_bonus.last_update = clock.unix_timestamp;
        }
        
        Ok(())
    }
    
    /// Update performance metrics
    fn update_performance_metrics(&mut self, rewards_earned: u64, current_apy: u16) {
        if self.staked_amount == 0 {
            return;
        }
        
        // Calculate total return percentage
        let total_return = (self.total_rewards_earned * 10000) / self.staked_amount;
        self.performance_metrics.total_return = total_return as u32;
        
        // Update annualized return (simplified calculation)
        self.performance_metrics.annualized_return = current_apy;
        
        // Update efficiency score based on actual vs expected returns
        let expected_return = (self.staked_amount as u128 * current_apy as u128 / 10000) as u64;
        if expected_return > 0 {
            let efficiency = (self.total_rewards_earned * 10000) / expected_return;
            self.performance_metrics.efficiency_score = std::cmp::min(efficiency as u16, 10000);
        }
    }
    
    /// Record staking transaction
    fn record_transaction(&mut self, tx_type: u8, amount: u64, exchange_rate: u64, apy: u16, clock: &Clock) -> Result<()> {
        let transaction = StakingTransaction {
            tx_type,
            timestamp: clock.unix_timestamp,
            amount,
            exchange_rate,
            apy_at_time: apy,
            tx_hash: Vec::new(), // Would be filled with actual transaction hash
            gas_fees: 0, // Would be calculated based on actual transaction
        };
        
        self.staking_history.push(transaction);
        
        // Keep only last 20 transactions
        if self.staking_history.len() > 20 {
            self.staking_history.remove(0);
        }
        
        Ok(())
    }
    
    /// Check and award achievements
    fn check_reward_achievements(&mut self, clock: &Clock) -> Result<()> {
        // Check for milestones
        let milestones = vec![
            (1_000_000_000, 1, "First 1000 FIN staked"),
            (10_000_000_000, 2, "10,000 FIN milestone"),
            (100_000_000_000, 3, "100,000 FIN milestone"),
        ];
        
        for (threshold, tier, description) in milestones {
            if self.staked_amount >= threshold {
                // Check if achievement already exists
                if !self.staking_achievements.iter().any(|a| a.description == description) {
                    let achievement = StakingAchievement {
                        achievement_id: self.staking_achievements.len() as u32,
                        earned_at: clock.unix_timestamp,
                        tier,
                        staking_bonus: tier * 100, // Bonus based on tier
                        description: description.to_string(),
                        expires_at: None, // Permanent achievement
                    };
                    
                    self.staking_achievements.push(achievement);
                }
            }
        }
        
        Ok(())
    }
    
    /// Get user's governance voting power
    pub fn get_voting_power(&self) -> u64 {
        let base_power = self.sfin_balance;
        let tier_multiplier = self.tier_benefits.governance_weight;
        let loyalty_multiplier = 1 + (self.loyalty_bonus.loyalty_level as u64 / 10);
        
        base_power * tier_multiplier as u64 * loyalty_multiplier
    }
    
    /// Get user's mining boost from staking
    pub fn get_mining_boost(&self, pool: &StakingPool) -> u16 {
        if let Some(tier_config) = pool.staking_tiers.get(self.staking_tier as usize) {
            return tier_config.mining_boost;
        }
        0
    }
    
    /// Get user's XP multiplier from staking
    pub fn get_xp_multiplier(&self, pool: &StakingPool) -> u16 {
        if let Some(tier_config) = pool.staking_tiers.get(self.staking_tier as usize) {
            return tier_config.xp_multiplier;
        }
        0
    }
    
    /// Get user's RP bonus from staking
    pub fn get_rp_bonus(&self, pool: &StakingPool) -> u16 {
        if let Some(tier_config) = pool.staking_tiers.get(self.staking_tier as usize) {
            return tier_config.rp_bonus;
        }
        0
    }
}
