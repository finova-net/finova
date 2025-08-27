// programs/finova-core/src/state/rewards.rs

use anchor_lang::prelude::*;
use crate::constants::*;
use crate::errors::FinovaError;

/// Reward pool account that manages the distribution of rewards across different categories
#[account]
#[derive(Debug)]
pub struct RewardPool {
    /// Bump seed for PDA derivation
    pub bump: u8,
    
    /// Authority that can manage the reward pool (typically program or admin)
    pub authority: Pubkey,
    
    /// Total rewards allocated to mining activities
    pub mining_pool: u64,
    
    /// Total rewards allocated to XP-based bonuses
    pub xp_pool: u64,
    
    /// Total rewards allocated to referral network rewards
    pub rp_pool: u64,
    
    /// Total rewards allocated to special events and bonuses
    pub special_events_pool: u64,
    
    /// Emergency reserve pool for unforeseen circumstances
    pub emergency_reserve: u64,
    
    /// Total rewards distributed in the current epoch
    pub total_distributed_current_epoch: u64,
    
    /// Total rewards distributed all time
    pub total_distributed_lifetime: u64,
    
    /// Current epoch number for reward calculations
    pub current_epoch: u64,
    
    /// Timestamp of the last pool update
    pub last_update_timestamp: i64,
    
    /// Daily distribution cap to prevent excessive inflation
    pub daily_distribution_cap: u64,
    
    /// Current day's distributed amount
    pub daily_distributed: u64,
    
    /// Last day timestamp for daily cap reset
    pub last_day_timestamp: i64,
    
    /// Pool status flags
    pub status: RewardPoolStatus,
    
    /// Reserved space for future upgrades
    pub reserved: [u64; 8],
}

/// Individual user's reward tracking account
#[account]
#[derive(Debug)]
pub struct UserRewards {
    /// Bump seed for PDA derivation
    pub bump: u8,
    
    /// User's public key
    pub user: Pubkey,
    
    /// Total mining rewards earned (all time)
    pub total_mining_rewards: u64,
    
    /// Total XP bonus rewards earned (all time)
    pub total_xp_rewards: u64,
    
    /// Total referral rewards earned (all time)
    pub total_referral_rewards: u64,
    
    /// Total special event rewards earned (all time)
    pub total_special_rewards: u64,
    
    /// Total staking rewards earned (all time)
    pub total_staking_rewards: u64,
    
    /// Pending rewards waiting to be claimed
    pub pending_mining_rewards: u64,
    pub pending_xp_rewards: u64,
    pub pending_referral_rewards: u64,
    pub pending_special_rewards: u64,
    pub pending_staking_rewards: u64,
    
    /// Last claim timestamp
    pub last_claim_timestamp: i64,
    
    /// Current mining streak (consecutive days)
    pub mining_streak: u32,
    
    /// Longest mining streak achieved
    pub longest_streak: u32,
    
    /// Total reward multiplier based on user status
    pub total_multiplier: u32, // Stored as basis points (10000 = 1.0x)
    
    /// Individual multipliers
    pub xp_multiplier: u32,     // From XP level
    pub rp_multiplier: u32,     // From referral tier
    pub staking_multiplier: u32, // From staking amount
    pub special_multiplier: u32, // From NFTs/cards
    
    /// Quality score affecting rewards (0-20000, 10000 = 1.0x)
    pub quality_score: u32,
    
    /// Network regression factor (0-10000, 10000 = 1.0x)
    pub regression_factor: u32,
    
    /// Last reward calculation timestamp
    pub last_calculation_timestamp: i64,
    
    /// Reward calculation history (last 30 days)
    pub daily_rewards: [DailyReward; 30],
    
    /// Current position in daily rewards array
    pub daily_rewards_index: u8,
    
    /// User reward flags and status
    pub flags: UserRewardFlags,
    
    /// Reserved space for future upgrades
    pub reserved: [u64; 6],
}

/// Daily reward snapshot for tracking user's reward history
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, Default)]
pub struct DailyReward {
    /// Timestamp of the reward day
    pub timestamp: i64,
    
    /// Mining rewards earned that day
    pub mining_rewards: u32,
    
    /// XP rewards earned that day
    pub xp_rewards: u32,
    
    /// Referral rewards earned that day
    pub referral_rewards: u32,
    
    /// Special rewards earned that day
    pub special_rewards: u32,
    
    /// Staking rewards earned that day
    pub staking_rewards: u32,
    
    /// Total multiplier applied that day
    pub multiplier: u32,
    
    /// Quality score for that day
    pub quality_score: u32,
}

/// Reward distribution configuration account
#[account]
#[derive(Debug)]
pub struct RewardConfig {
    /// Bump seed for PDA derivation
    pub bump: u8,
    
    /// Authority that can update the configuration
    pub authority: Pubkey,
    
    /// Base mining rate (per hour in smallest token unit)
    pub base_mining_rate: u64,
    
    /// Finizen bonus multiplier (basis points)
    pub finizen_bonus: u32,
    
    /// Maximum referral bonus multiplier (basis points)
    pub max_referral_bonus: u32,
    
    /// Security bonus for KYC verified users (basis points)
    pub security_bonus: u32,
    
    /// Regression coefficient for exponential decay
    pub regression_coefficient: u32, // Stored as fixed point (1000000 = 1.0)
    
    /// XP level multiplier coefficients
    pub xp_multiplier_base: u32,     // Base multiplier
    pub xp_multiplier_growth: u32,   // Growth rate per level
    
    /// RP tier multiplier coefficients
    pub rp_multiplier_base: u32,     // Base multiplier
    pub rp_multiplier_growth: u32,   // Growth rate per tier
    
    /// Staking multiplier configurations
    pub staking_multipliers: [StakingTier; 5],
    
    /// Quality score impact on rewards
    pub quality_min_multiplier: u32, // Minimum multiplier (basis points)
    pub quality_max_multiplier: u32, // Maximum multiplier (basis points)
    
    /// Daily caps and limits
    pub daily_mining_cap: u64,       // Maximum mining per day per user
    pub daily_xp_cap: u64,          // Maximum XP rewards per day
    pub daily_referral_cap: u64,    // Maximum referral rewards per day
    
    /// Phase configuration for network growth
    pub current_phase: u8,
    pub phase_user_thresholds: [u64; 4], // User count thresholds for each phase
    pub phase_mining_rates: [u64; 4],    // Mining rates for each phase
    pub phase_finizen_bonuses: [u32; 4], // Finizen bonuses for each phase
    
    /// Emergency controls
    pub emergency_pause: bool,
    pub emergency_rate_multiplier: u32, // Emergency rate adjustment
    
    /// Configuration update timestamp
    pub last_config_update: i64,
    
    /// Reserved space for future parameters
    pub reserved: [u64; 10],
}

/// Staking tier configuration for reward multipliers
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, Default)]
pub struct StakingTier {
    /// Minimum staking amount for this tier
    pub min_stake: u64,
    
    /// Maximum staking amount for this tier
    pub max_stake: u64,
    
    /// Mining boost multiplier (basis points)
    pub mining_boost: u32,
    
    /// XP multiplier bonus (basis points)
    pub xp_bonus: u32,
    
    /// RP bonus multiplier (basis points)
    pub rp_bonus: u32,
    
    /// APY for staking rewards (basis points)
    pub staking_apy: u32,
}

/// Guild reward pool for collective rewards
#[account]
#[derive(Debug)]
pub struct GuildRewardPool {
    /// Bump seed for PDA derivation
    pub bump: u8,
    
    /// Guild public key
    pub guild: Pubkey,
    
    /// Total rewards allocated to this guild
    pub total_allocated: u64,
    
    /// Total rewards distributed to guild members
    pub total_distributed: u64,
    
    /// Pending rewards waiting for distribution
    pub pending_rewards: u64,
    
    /// Guild performance multiplier (basis points)
    pub performance_multiplier: u32,
    
    /// Last distribution timestamp
    pub last_distribution: i64,
    
    /// Distribution schedule (how often to distribute)
    pub distribution_frequency: i64, // In seconds
    
    /// Member count for reward calculation
    pub active_member_count: u32,
    
    /// Guild reward status
    pub status: GuildRewardStatus,
    
    /// Reserved space
    pub reserved: [u64; 6],
}

/// Special event reward configuration
#[account]
#[derive(Debug)]
pub struct SpecialEventReward {
    /// Bump seed for PDA derivation
    pub bump: u8,
    
    /// Event unique identifier
    pub event_id: u64,
    
    /// Event authority
    pub authority: Pubkey,
    
    /// Event name (32 bytes)
    pub event_name: [u8; 32],
    
    /// Event description (64 bytes)
    pub event_description: [u8; 64],
    
    /// Total reward pool for this event
    pub total_pool: u64,
    
    /// Rewards distributed so far
    pub distributed: u64,
    
    /// Event start timestamp
    pub start_timestamp: i64,
    
    /// Event end timestamp
    pub end_timestamp: i64,
    
    /// Reward per participant
    pub reward_per_participant: u64,
    
    /// Maximum participants
    pub max_participants: u32,
    
    /// Current participant count
    pub participant_count: u32,
    
    /// Event requirements
    pub requirements: EventRequirements,
    
    /// Event status
    pub status: EventStatus,
    
    /// Reserved space
    pub reserved: [u64; 8],
}

/// Event participation requirements
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, Default)]
pub struct EventRequirements {
    /// Minimum XP level required
    pub min_xp_level: u32,
    
    /// Minimum RP tier required
    pub min_rp_tier: u32,
    
    /// Minimum staking amount required
    pub min_staking_amount: u64,
    
    /// KYC verification required
    pub kyc_required: bool,
    
    /// Guild membership required
    pub guild_membership_required: bool,
    
    /// Specific NFT ownership required
    pub required_nft_collection: Option<Pubkey>,
    
    /// Social media verification required
    pub social_verification_required: bool,
}

/// Reward pool status flags
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub struct RewardPoolStatus {
    /// Pool is active and distributing rewards
    pub active: bool,
    
    /// Pool is paused (emergency or maintenance)
    pub paused: bool,
    
    /// Pool is in maintenance mode
    pub maintenance: bool,
    
    /// Pool has emergency restrictions
    pub emergency_mode: bool,
}

impl Default for RewardPoolStatus {
    fn default() -> Self {
        Self {
            active: true,
            paused: false,
            maintenance: false,
            emergency_mode: false,
        }
    }
}

/// User reward flags
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub struct UserRewardFlags {
    /// User is eligible for rewards
    pub eligible: bool,
    
    /// User is KYC verified (security bonus)
    pub kyc_verified: bool,
    
    /// User has quality content bonus
    pub quality_bonus: bool,
    
    /// User is in a guild
    pub guild_member: bool,
    
    /// User has active staking
    pub staking_active: bool,
    
    /// User is flagged for review
    pub flagged: bool,
    
    /// User account is suspended
    pub suspended: bool,
}

impl Default for UserRewardFlags {
    fn default() -> Self {
        Self {
            eligible: true,
            kyc_verified: false,
            quality_bonus: false,
            guild_member: false,
            staking_active: false,
            flagged: false,
            suspended: false,
        }
    }
}

/// Guild reward status
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub struct GuildRewardStatus {
    /// Guild reward pool is active
    pub active: bool,
    
    /// Guild is eligible for collective rewards
    pub eligible: bool,
    
    /// Guild is in performance bonus tier
    pub performance_bonus: bool,
}

impl Default for GuildRewardStatus {
    fn default() -> Self {
        Self {
            active: true,
            eligible: true,
            performance_bonus: false,
        }
    }
}

/// Special event status
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum EventStatus {
    /// Event is scheduled but not started
    Scheduled,
    /// Event is currently active
    Active,
    /// Event has ended
    Ended,
    /// Event was cancelled
    Cancelled,
    /// Event is paused
    Paused,
}

impl Default for EventStatus {
    fn default() -> Self {
        EventStatus::Scheduled
    }
}

impl RewardPool {
    pub const LEN: usize = 8 + // discriminator
        1 +  // bump
        32 + // authority
        8 +  // mining_pool
        8 +  // xp_pool
        8 +  // rp_pool
        8 +  // special_events_pool
        8 +  // emergency_reserve
        8 +  // total_distributed_current_epoch
        8 +  // total_distributed_lifetime
        8 +  // current_epoch
        8 +  // last_update_timestamp
        8 +  // daily_distribution_cap
        8 +  // daily_distributed
        8 +  // last_day_timestamp
        (1 + 1 + 1 + 1) + // status flags
        (8 * 8); // reserved
    
    /// Initialize a new reward pool
    pub fn initialize(
        &mut self,
        bump: u8,
        authority: Pubkey,
        initial_pools: (u64, u64, u64, u64, u64), // (mining, xp, rp, special, emergency)
        daily_cap: u64,
    ) -> Result<()> {
        self.bump = bump;
        self.authority = authority;
        self.mining_pool = initial_pools.0;
        self.xp_pool = initial_pools.1;
        self.rp_pool = initial_pools.2;
        self.special_events_pool = initial_pools.3;
        self.emergency_reserve = initial_pools.4;
        self.total_distributed_current_epoch = 0;
        self.total_distributed_lifetime = 0;
        self.current_epoch = 0;
        self.last_update_timestamp = Clock::get()?.unix_timestamp;
        self.daily_distribution_cap = daily_cap;
        self.daily_distributed = 0;
        self.last_day_timestamp = Clock::get()?.unix_timestamp;
        self.status = RewardPoolStatus::default();
        self.reserved = [0; 8];
        
        Ok(())
    }
    
    /// Check if daily distribution cap allows for more rewards
    pub fn can_distribute(&mut self, amount: u64) -> Result<bool> {
        let current_time = Clock::get()?.unix_timestamp;
        
        // Reset daily counter if it's a new day
        if current_time - self.last_day_timestamp >= SECONDS_PER_DAY {
            self.daily_distributed = 0;
            self.last_day_timestamp = current_time;
        }
        
        Ok(self.daily_distributed + amount <= self.daily_distribution_cap)
    }
    
    /// Distribute rewards from the pool
    pub fn distribute_rewards(
        &mut self,
        mining_amount: u64,
        xp_amount: u64,
        rp_amount: u64,
        special_amount: u64,
    ) -> Result<()> {
        // Verify pool has sufficient funds
        require!(self.mining_pool >= mining_amount, FinovaError::InsufficientRewardPool);
        require!(self.xp_pool >= xp_amount, FinovaError::InsufficientRewardPool);
        require!(self.rp_pool >= rp_amount, FinovaError::InsufficientRewardPool);
        require!(self.special_events_pool >= special_amount, FinovaError::InsufficientRewardPool);
        
        let total_amount = mining_amount + xp_amount + rp_amount + special_amount;
        
        // Check daily distribution cap
        require!(self.can_distribute(total_amount)?, FinovaError::DailyCapExceeded);
        
        // Deduct from pools
        self.mining_pool -= mining_amount;
        self.xp_pool -= xp_amount;
        self.rp_pool -= rp_amount;
        self.special_events_pool -= special_amount;
        
        // Update distributed amounts
        self.total_distributed_current_epoch += total_amount;
        self.total_distributed_lifetime += total_amount;
        self.daily_distributed += total_amount;
        
        self.last_update_timestamp = Clock::get()?.unix_timestamp;
        
        Ok(())
    }
    
    /// Replenish reward pools
    pub fn replenish_pools(
        &mut self,
        mining_amount: u64,
        xp_amount: u64,
        rp_amount: u64,
        special_amount: u64,
        emergency_amount: u64,
    ) -> Result<()> {
        self.mining_pool += mining_amount;
        self.xp_pool += xp_amount;
        self.rp_pool += rp_amount;
        self.special_events_pool += special_amount;
        self.emergency_reserve += emergency_amount;
        
        self.last_update_timestamp = Clock::get()?.unix_timestamp;
        
        Ok(())
    }
}

impl UserRewards {
    pub const LEN: usize = 8 + // discriminator
        1 +  // bump
        32 + // user
        8 +  // total_mining_rewards
        8 +  // total_xp_rewards
        8 +  // total_referral_rewards
        8 +  // total_special_rewards
        8 +  // total_staking_rewards
        8 +  // pending_mining_rewards
        8 +  // pending_xp_rewards
        8 +  // pending_referral_rewards
        8 +  // pending_special_rewards
        8 +  // pending_staking_rewards
        8 +  // last_claim_timestamp
        4 +  // mining_streak
        4 +  // longest_streak
        4 +  // total_multiplier
        4 +  // xp_multiplier
        4 +  // rp_multiplier
        4 +  // staking_multiplier
        4 +  // special_multiplier
        4 +  // quality_score
        4 +  // regression_factor
        8 +  // last_calculation_timestamp
        (30 * 32) + // daily_rewards array (30 * DailyReward size)
        1 +  // daily_rewards_index
        (1 + 1 + 1 + 1 + 1 + 1 + 1) + // flags
        (8 * 6); // reserved
    
    /// Initialize user rewards account
    pub fn initialize(&mut self, bump: u8, user: Pubkey) -> Result<()> {
        self.bump = bump;
        self.user = user;
        self.total_mining_rewards = 0;
        self.total_xp_rewards = 0;
        self.total_referral_rewards = 0;
        self.total_special_rewards = 0;
        self.total_staking_rewards = 0;
        self.pending_mining_rewards = 0;
        self.pending_xp_rewards = 0;
        self.pending_referral_rewards = 0;
        self.pending_special_rewards = 0;
        self.pending_staking_rewards = 0;
        self.last_claim_timestamp = Clock::get()?.unix_timestamp;
        self.mining_streak = 0;
        self.longest_streak = 0;
        self.total_multiplier = 10000; // 1.0x in basis points
        self.xp_multiplier = 10000;
        self.rp_multiplier = 10000;
        self.staking_multiplier = 10000;
        self.special_multiplier = 10000;
        self.quality_score = 10000; // 1.0x in basis points
        self.regression_factor = 10000; // 1.0x in basis points
        self.last_calculation_timestamp = Clock::get()?.unix_timestamp;
        self.daily_rewards = [DailyReward::default(); 30];
        self.daily_rewards_index = 0;
        self.flags = UserRewardFlags::default();
        self.reserved = [0; 6];
        
        Ok(())
    }
    
    /// Add pending rewards
    pub fn add_pending_rewards(
        &mut self,
        mining: u64,
        xp: u64,
        referral: u64,
        special: u64,
        staking: u64,
    ) -> Result<()> {
        self.pending_mining_rewards += mining;
        self.pending_xp_rewards += xp;
        self.pending_referral_rewards += referral;
        self.pending_special_rewards += special;
        self.pending_staking_rewards += staking;
        
        // Update daily rewards tracking
        self.update_daily_rewards(mining, xp, referral, special, staking)?;
        
        Ok(())
    }
    
    /// Claim all pending rewards
    pub fn claim_rewards(&mut self) -> Result<(u64, u64, u64, u64, u64)> {
        let mining = self.pending_mining_rewards;
        let xp = self.pending_xp_rewards;
        let referral = self.pending_referral_rewards;
        let special = self.pending_special_rewards;
        let staking = self.pending_staking_rewards;
        
        // Add to total earned
        self.total_mining_rewards += mining;
        self.total_xp_rewards += xp;
        self.total_referral_rewards += referral;
        self.total_special_rewards += special;
        self.total_staking_rewards += staking;
        
        // Clear pending
        self.pending_mining_rewards = 0;
        self.pending_xp_rewards = 0;
        self.pending_referral_rewards = 0;
        self.pending_special_rewards = 0;
        self.pending_staking_rewards = 0;
        
        self.last_claim_timestamp = Clock::get()?.unix_timestamp;
        
        Ok((mining, xp, referral, special, staking))
    }
    
    /// Update daily rewards tracking
    fn update_daily_rewards(
        &mut self,
        mining: u64,
        xp: u64,
        referral: u64,
        special: u64,
        staking: u64,
    ) -> Result<()> {
        let current_time = Clock::get()?.unix_timestamp;
        let current_day = current_time / SECONDS_PER_DAY;
        
        // Check if we need to move to next day slot
        let last_recorded_day = if self.daily_rewards_index > 0 {
            self.daily_rewards[(self.daily_rewards_index - 1) as usize].timestamp / SECONDS_PER_DAY
        } else {
            0
        };
        
        if current_day != last_recorded_day {
            // Move to next slot
            self.daily_rewards_index = (self.daily_rewards_index + 1) % 30;
            
            // Initialize new day entry
            self.daily_rewards[self.daily_rewards_index as usize] = DailyReward {
                timestamp: current_time,
                mining_rewards: mining as u32,
                xp_rewards: xp as u32,
                referral_rewards: referral as u32,
                special_rewards: special as u32,
                staking_rewards: staking as u32,
                multiplier: self.total_multiplier,
                quality_score: self.quality_score,
            };
        } else {
            // Add to existing day
            let current_day_rewards = &mut self.daily_rewards[self.daily_rewards_index as usize];
            current_day_rewards.mining_rewards += mining as u32;
            current_day_rewards.xp_rewards += xp as u32;
            current_day_rewards.referral_rewards += referral as u32;
            current_day_rewards.special_rewards += special as u32;
            current_day_rewards.staking_rewards += staking as u32;
        }
        
        Ok(())
    }
    
    /// Calculate total reward multiplier from all sources
    pub fn calculate_total_multiplier(&mut self) -> u32 {
        let base_multiplier = 10000u64; // 1.0x
        
        let xp_factor = (self.xp_multiplier as u64) * base_multiplier / 10000;
        let rp_factor = (self.rp_multiplier as u64) * base_multiplier / 10000;
        let staking_factor = (self.staking_multiplier as u64) * base_multiplier / 10000;
        let special_factor = (self.special_multiplier as u64) * base_multiplier / 10000;
        let quality_factor = (self.quality_score as u64) * base_multiplier / 10000;
        let regression_factor = (self.regression_factor as u64) * base_multiplier / 10000;
        
        // Compound all multipliers
        let total = (base_multiplier * xp_factor / 10000)
            .saturating_mul(rp_factor).saturating_div(10000)
            .saturating_mul(staking_factor).saturating_div(10000)
            .saturating_mul(special_factor).saturating_div(10000)
            .saturating_mul(quality_factor).saturating_div(10000)
            .saturating_mul(regression_factor).saturating_div(10000);
            
        self.total_multiplier = std::cmp::min(total as u32, 50000); // Cap at 5.0x
        self.total_multiplier
    }
    
    /// Update mining streak
    pub fn update_mining_streak(&mut self, is_consecutive: bool) -> Result<()> {
        if is_consecutive {
            self.mining_streak += 1;
            if self.mining_streak > self.longest_streak {
                self.longest_streak = self.mining_streak;
            }
        } else {
            self.mining_streak = 1; // Reset to 1 (current day)
        }
        
        Ok(())
    }
    
    /// Get total pending rewards
    pub fn get_total_pending(&self) -> u64 {
        self.pending_mining_rewards +
        self.pending_xp_rewards +
        self.pending_referral_rewards +
        self.pending_special_rewards +
        self.pending_staking_rewards
    }
    
    /// Get total lifetime rewards
    pub fn get_total_lifetime(&self) -> u64 {
        self.total_mining_rewards +
        self.total_xp_rewards +
        self.total_referral_rewards +
        self.total_special_rewards +
        self.total_staking_rewards
    }
}
