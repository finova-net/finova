// programs/finova-core/src/state/mining.rs

use anchor_lang::prelude::*;

#[account]
#[derive(Default)]
pub struct MiningPool {
    /// Global mining pool authority
    pub authority: Pubkey,
    
    /// Pool initialization timestamp
    pub created_at: i64,
    
    /// Current mining phase (1-4)
    pub current_phase: u8,
    
    /// Total users in the network
    pub total_users: u64,
    
    /// Total active miners (mined in last 30 days)
    pub active_miners: u64,
    
    /// Total FIN tokens minted through mining
    pub total_fin_minted: u64,
    
    /// Current base mining rate (micro-FIN per hour)
    pub base_mining_rate: u64,
    
    /// Finizen bonus multiplier (stored as percentage * 100)
    pub finizen_bonus: u16,
    
    /// Last phase transition timestamp
    pub last_phase_transition: i64,
    
    /// Next phase transition threshold (user count)
    pub next_phase_threshold: u64,
    
    /// Pool reserves for mining rewards
    pub reward_reserves: u64,
    
    /// Daily mining limit per user (micro-FIN)
    pub daily_mining_limit: u64,
    
    /// Emergency pause status
    pub paused: bool,
    
    /// Pause reason code
    pub pause_reason: u8,
    
    /// Mining difficulty adjustment factor
    pub difficulty_adjustment: u16,
    
    /// Network hash rate (computational power metric)
    pub network_hash_rate: u64,
    
    /// Quality score threshold for mining eligibility
    pub quality_threshold: u16,
    
    /// Anti-bot detection enabled
    pub anti_bot_enabled: bool,
    
    /// Regression factor for whale protection
    pub whale_regression_factor: u16,
    
    /// Phase transition history
    pub phase_history: Vec<PhaseTransition>,
    
    /// Mining statistics
    pub mining_stats: MiningStatistics,
    
    /// Reserved space for future upgrades
    pub _reserved: [u8; 64],
}

#[account]
#[derive(Default)]
pub struct UserMiningAccount {
    /// User's public key
    pub user: Pubkey,
    
    /// Mining account creation timestamp
    pub created_at: i64,
    
    /// Last mining activity timestamp
    pub last_mining_activity: i64,
    
    /// User's current mining rate (micro-FIN per hour)
    pub current_mining_rate: u64,
    
    /// Base mining rate before multipliers
    pub base_rate: u64,
    
    /// Total mining multiplier (stored as percentage * 100)
    pub total_multiplier: u32,
    
    /// Finizen bonus applied
    pub finizen_bonus: u16,
    
    /// Referral network bonus
    pub referral_bonus: u16,
    
    /// Security bonus (KYC verification)
    pub security_bonus: u16,
    
    /// Staking bonus multiplier
    pub staking_bonus: u16,
    
    /// Special card effects multiplier
    pub card_effects_bonus: u16,
    
    /// Guild participation bonus
    pub guild_bonus: u16,
    
    /// Quality score bonus/penalty
    pub quality_bonus: i16, // Can be negative for penalties
    
    /// Daily mining progress
    pub daily_mining_progress: DailyMiningProgress,
    
    /// Weekly mining progress
    pub weekly_mining_progress: WeeklyMiningProgress,
    
    /// Monthly mining progress
    pub monthly_mining_progress: MonthlyMiningProgress,
    
    /// Mining achievements unlocked
    pub mining_achievements: Vec<MiningAchievement>,
    
    /// Exponential regression factor applied
    pub regression_factor: u32,
    
    /// Mining phase when user started
    pub starting_phase: u8,
    
    /// Phase progression bonuses
    pub phase_bonuses: Vec<PhaseBonus>,
    
    /// Consecutive mining days
    pub consecutive_mining_days: u32,
    
    /// Longest mining streak achieved
    pub longest_mining_streak: u32,
    
    /// Total mining sessions
    pub total_mining_sessions: u64,
    
    /// Average mining session duration (seconds)
    pub avg_session_duration: u32,
    
    /// Mining efficiency score (0-10000)
    pub mining_efficiency: u16,
    
    /// Bot detection flags and scores
    pub bot_detection: BotDetectionData,
    
    /// Mining penalties applied
    pub penalties: Vec<MiningPenalty>,
    
    /// Reserved space
    pub _reserved: [u8; 32],
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default)]
pub struct PhaseTransition {
    /// Phase number transitioned to
    pub to_phase: u8,
    
    /// Timestamp of transition
    pub timestamp: i64,
    
    /// User count at transition
    pub user_count_at_transition: u64,
    
    /// Old mining rate
    pub old_rate: u64,
    
    /// New mining rate
    pub new_rate: u64,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default)]
pub struct MiningStatistics {
    /// Total mining operations performed
    pub total_mining_operations: u64,
    
    /// Total computation time (seconds)
    pub total_computation_time: u64,
    
    /// Average mining rate across all users
    pub average_mining_rate: u64,
    
    /// Peak concurrent miners
    pub peak_concurrent_miners: u32,
    
    /// Total rewards distributed
    pub total_rewards_distributed: u64,
    
    /// Daily mining volume (last 24h)
    pub daily_volume: u64,
    
    /// Weekly mining volume (last 7 days)
    pub weekly_volume: u64,
    
    /// Monthly mining volume (last 30 days)
    pub monthly_volume: u64,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default)]
pub struct DailyMiningProgress {
    /// Current day (YYYYMMDD format)
    pub current_day: u32,
    
    /// FIN mined today
    pub fin_mined_today: u64,
    
    /// Mining sessions today
    pub sessions_today: u32,
    
    /// Total mining time today (seconds)
    pub mining_time_today: u32,
    
    /// Daily mining limit
    pub daily_limit: u64,
    
    /// Daily limit reached
    pub limit_reached: bool,
    
    /// Bonus multiplier for today
    pub daily_bonus_multiplier: u16,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default)]
pub struct WeeklyMiningProgress {
    /// Current week (YYYYWW format)
    pub current_week: u32,
    
    /// FIN mined this week
    pub fin_mined_week: u64,
    
    /// Days mined this week
    pub days_mined: u8,
    
    /// Weekly mining target
    pub weekly_target: u64,
    
    /// Weekly bonus earned
    pub weekly_bonus: u64,
    
    /// Week completion percentage
    pub completion_percentage: u8,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default)]
pub struct MonthlyMiningProgress {
    /// Current month (YYYYMM format)
    pub current_month: u32,
    
    /// FIN mined this month
    pub fin_mined_month: u64,
    
    /// Days mined this month
    pub days_mined: u8,
    
    /// Monthly mining target
    pub monthly_target: u64,
    
    /// Monthly rank among all miners
    pub monthly_rank: u32,
    
    /// Monthly performance tier (Bronze, Silver, Gold, etc.)
    pub performance_tier: u8,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default)]
pub struct MiningAchievement {
    /// Achievement ID
    pub achievement_id: u32,
    
    /// Achievement unlocked timestamp
    pub unlocked_at: i64,
    
    /// Achievement tier/level
    pub tier: u8,
    
    /// Bonus mining rate from this achievement
    pub bonus_rate: u16,
    
    /// Achievement expiry (if temporary)
    pub expires_at: Option<i64>,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default)]
pub struct PhaseBonus {
    /// Phase number
    pub phase: u8,
    
    /// Length of participation in this phase (days)
    pub participation_days: u32,
    
    /// Bonus multiplier earned
    pub bonus_multiplier: u16,
    
    /// Phase loyalty bonus
    pub loyalty_bonus: u16,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default)]
pub struct BotDetectionData {
    /// Behavioral consistency score (0-10000)
    pub behavioral_score: u16,
    
    /// Timing pattern score (0-10000)
    pub timing_score: u16,
    
    /// Device fingerprint score (0-10000)
    pub device_score: u16,
    
    /// Social interaction score (0-10000)
    pub social_score: u16,
    
    /// Overall bot probability (0-10000)
    pub bot_probability: u16,
    
    /// Last bot detection check
    pub last_check: i64,
    
    /// Number of failed checks
    pub failed_checks: u32,
    
    /// Detection flags bitmask
    pub detection_flags: u32,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default)]
pub struct MiningPenalty {
    /// Penalty type (rate reduction, suspension, etc.)
    pub penalty_type: u8,
    
    /// Penalty severity (percentage reduction * 100)
    pub severity: u16,
    
    /// Penalty start timestamp
    pub applied_at: i64,
    
    /// Penalty duration (seconds, 0 = permanent)
    pub duration: u64,
    
    /// Penalty reason code
    pub reason_code: u16,
    
    /// Whether penalty is active
    pub active: bool,
}

impl MiningPool {
    pub const LEN: usize = 8 + // discriminator
        32 + // authority
        8 + // created_at
        1 + // current_phase
        8 + // total_users
        8 + // active_miners
        8 + // total_fin_minted
        8 + // base_mining_rate
        2 + // finizen_bonus
        8 + // last_phase_transition
        8 + // next_phase_threshold
        8 + // reward_reserves
        8 + // daily_mining_limit
        1 + // paused
        1 + // pause_reason
        2 + // difficulty_adjustment
        8 + // network_hash_rate
        2 + // quality_threshold
        1 + // anti_bot_enabled
        2 + // whale_regression_factor
        4 + 10 * 32 + // phase_history (Vec, assume max 10)
        128 + // mining_stats
        64; // _reserved
    
    /// Initialize the mining pool
    pub fn initialize(&mut self, authority: Pubkey, clock: &Clock) -> Result<()> {
        self.authority = authority;
        self.created_at = clock.unix_timestamp;
        self.current_phase = 1; // Start in Phase 1 (Finizen)
        self.total_users = 0;
        self.active_miners = 0;
        self.total_fin_minted = 0;
        self.base_mining_rate = 100_000; // 0.1 FIN/hour in micro-FIN
        self.finizen_bonus = 20000; // 200% bonus (2.0x)
        self.last_phase_transition = clock.unix_timestamp;
        self.next_phase_threshold = 100_000; // Phase 2 at 100K users
        self.reward_reserves = 0;
        self.daily_mining_limit = 2_400_000; // 2.4 FIN per day maximum
        self.paused = false;
        self.pause_reason = 0;
        self.difficulty_adjustment = 10000; // 1.0x (no adjustment)
        self.network_hash_rate = 0;
        self.quality_threshold = 5000; // 0.5 minimum quality score
        self.anti_bot_enabled = true;
        self.whale_regression_factor = 1000; // 0.1% factor
        self.phase_history = Vec::new();
        self.mining_stats = MiningStatistics::default();
        
        Ok(())
    }
    
    /// Check if phase transition is needed
    pub fn check_phase_transition(&mut self, clock: &Clock) -> Result<bool> {
        if self.total_users >= self.next_phase_threshold && self.current_phase < 4 {
            self.transition_to_next_phase(clock)?;
            return Ok(true);
        }
        Ok(false)
    }
    
    /// Transition to the next mining phase
    fn transition_to_next_phase(&mut self, clock: &Clock) -> Result<()> {
        let old_rate = self.base_mining_rate;
        let old_phase = self.current_phase;
        
        // Record transition history
        let transition = PhaseTransition {
            to_phase: self.current_phase + 1,
            timestamp: clock.unix_timestamp,
            user_count_at_transition: self.total_users,
            old_rate,
            new_rate: 0, // Will be set below
        };
        
        // Update phase and associated parameters
        self.current_phase += 1;
        self.last_phase_transition = clock.unix_timestamp;
        
        match self.current_phase {
            2 => {
                // Growth Phase
                self.base_mining_rate = 50_000; // 0.05 FIN/hour
                self.finizen_bonus = 15000; // 150% bonus (1.5x)
                self.next_phase_threshold = 1_000_000; // Phase 3 at 1M users
                self.daily_mining_limit = 1_800_000; // 1.8 FIN per day
            }
            3 => {
                // Maturity Phase
                self.base_mining_rate = 25_000; // 0.025 FIN/hour
                self.finizen_bonus = 12000; // 120% bonus (1.2x)
                self.next_phase_threshold = 10_000_000; // Phase 4 at 10M users
                self.daily_mining_limit = 720_000; // 0.72 FIN per day
            }
            4 => {
                // Stability Phase
                self.base_mining_rate = 10_000; // 0.01 FIN/hour
                self.finizen_bonus = 10000; // 100% bonus (1.0x, no bonus)
                self.next_phase_threshold = u64::MAX; // No more transitions
                self.daily_mining_limit = 240_000; // 0.24 FIN per day
            }
            _ => return Err(ErrorCode::InvalidPhaseTransition.into()),
        }
        
        // Update transition record with new rate
        let mut updated_transition = transition;
        updated_transition.new_rate = self.base_mining_rate;
        self.phase_history.push(updated_transition);
        
        Ok(())
    }
    
    /// Calculate current Finizen bonus based on network size
    pub fn calculate_finizen_bonus(&self) -> u16 {
        if self.total_users == 0 {
            return self.finizen_bonus;
        }
        
        // Finizen_Bonus = max(1.0, 2.0 - (Total_Users / 1,000,000))
        let bonus_float = 2.0 - (self.total_users as f64 / 1_000_000.0);
        let bonus = if bonus_float < 1.0 { 1.0 } else { bonus_float };
        
        (bonus * 10000.0) as u16 // Convert to basis points
    }
    
    /// Add a new user to the mining pool
    pub fn add_user(&mut self, clock: &Clock) -> Result<()> {
        self.total_users += 1;
        
        // Update Finizen bonus
        self.finizen_bonus = self.calculate_finizen_bonus();
        
        // Check for phase transition
        self.check_phase_transition(clock)?;
        
        Ok(())
    }
    
    /// Update active miners count
    pub fn update_active_miners(&mut self, count: u64) -> Result<()> {
        self.active_miners = count;
        Ok(())
    }
    
    /// Record mining activity
    pub fn record_mining(&mut self, amount: u64) -> Result<()> {
        self.total_fin_minted = self.total_fin_minted.checked_add(amount).unwrap();
        self.mining_stats.total_mining_operations += 1;
        self.mining_stats.total_rewards_distributed = self.mining_stats.total_rewards_distributed.checked_add(amount).unwrap();
        Ok(())
    }
    
    /// Emergency pause mining
    pub fn emergency_pause(&mut self, reason: u8) -> Result<()> {
        self.paused = true;
        self.pause_reason = reason;
        Ok(())
    }
    
    /// Resume mining after pause
    pub fn resume_mining(&mut self) -> Result<()> {
        self.paused = false;
        self.pause_reason = 0;
        Ok(())
    }
    
    /// Get effective mining rate for calculations
    pub fn get_effective_base_rate(&self) -> u64 {
        if self.paused {
            return 0;
        }
        
        // Apply difficulty adjustment
        (self.base_mining_rate as u128 * self.difficulty_adjustment as u128 / 10000) as u64
    }
}

impl UserMiningAccount {
    pub const LEN: usize = 8 + // discriminator
        32 + // user
        8 + // created_at
        8 + // last_mining_activity
        8 + // current_mining_rate
        8 + // base_rate
        4 + // total_multiplier
        2 + // finizen_bonus
        2 + // referral_bonus
        2 + // security_bonus
        2 + // staking_bonus
        2 + // card_effects_bonus
        2 + // guild_bonus
        2 + // quality_bonus (i16)
        64 + // daily_mining_progress
        64 + // weekly_mining_progress
        64 + // monthly_mining_progress
        4 + 10 * 32 + // mining_achievements (Vec, assume max 10)
        4 + // regression_factor
        1 + // starting_phase
        4 + 4 * 16 + // phase_bonuses (Vec, assume max 4 phases)
        4 + // consecutive_mining_days
        4 + // longest_mining_streak
        8 + // total_mining_sessions
        4 + // avg_session_duration
        2 + // mining_efficiency
        64 + // bot_detection
        4 + 5 * 32 + // penalties (Vec, assume max 5)
        32; // _reserved
    
    /// Initialize user mining account
    pub fn initialize(&mut self, user: Pubkey, base_rate: u64, phase: u8, clock: &Clock) -> Result<()> {
        self.user = user;
        self.created_at = clock.unix_timestamp;
        self.last_mining_activity = clock.unix_timestamp;
        self.base_rate = base_rate;
        self.current_mining_rate = base_rate;
        self.total_multiplier = 10000; // 1.0x base multiplier
        self.finizen_bonus = 20000; // Default Finizen bonus
        self.referral_bonus = 10000; // 1.0x (no referral bonus initially)
        self.security_bonus = 8000; // 0.8x (not KYC verified)
        self.staking_bonus = 10000; // 1.0x (no staking)
        self.card_effects_bonus = 10000; // 1.0x (no cards)
        self.guild_bonus = 10000; // 1.0x (no guild)
        self.quality_bonus = 0; // Neutral quality
        self.regression_factor = 10000; // 1.0x (no regression)
        self.starting_phase = phase;
        self.consecutive_mining_days = 0;
        self.longest_mining_streak = 0;
        self.total_mining_sessions = 0;
        self.avg_session_duration = 0;
        self.mining_efficiency = 5000; // 0.5 initial efficiency
        self.daily_mining_progress = DailyMiningProgress::default();
        self.weekly_mining_progress = WeeklyMiningProgress::default();
        self.monthly_mining_progress = MonthlyMiningProgress::default();
        self.mining_achievements = Vec::new();
        self.phase_bonuses = Vec::new();
        self.bot_detection = BotDetectionData::default();
        self.penalties = Vec::new();
        
        Ok(())
    }
    
    /// Calculate effective mining rate with all bonuses and penalties
    pub fn calculate_effective_mining_rate(&mut self, pool: &MiningPool, clock: &Clock) -> Result<u64> {
        if pool.paused {
            return Ok(0);
        }
        
        // Start with base rate from pool
        let base_rate = pool.get_effective_base_rate();
        
        // Update bonuses
        self.finizen_bonus = pool.calculate_finizen_bonus();
        
        // Calculate total multiplier
        let mut total_multiplier = 10000u128; // Base 1.0x
        
        // Apply all bonuses
        total_multiplier = total_multiplier * self.finizen_bonus as u128 / 10000;
        total_multiplier = total_multiplier * self.referral_bonus as u128 / 10000;
        total_multiplier = total_multiplier * self.security_bonus as u128 / 10000;
        total_multiplier = total_multiplier * self.staking_bonus as u128 / 10000;
        total_multiplier = total_multiplier * self.card_effects_bonus as u128 / 10000;
        total_multiplier = total_multiplier * self.guild_bonus as u128 / 10000;
        
        // Apply quality bonus/penalty
        if self.quality_bonus >= 0 {
            total_multiplier = total_multiplier * (10000 + self.quality_bonus as u16) as u128 / 10000;
        } else {
            total_multiplier = total_multiplier * (10000u16.saturating_sub((-self.quality_bonus) as u16)) as u128 / 10000;
        }
        
        // Apply regression factor (whale protection)
        total_multiplier = total_multiplier * self.regression_factor as u128 / 10000;
        
        // Apply active penalties
        for penalty in &self.penalties {
            if penalty.active && (penalty.duration == 0 || 
                clock.unix_timestamp < penalty.applied_at + penalty.duration as i64) {
                let penalty_factor = 10000u16.saturating_sub(penalty.severity);
                total_multiplier = total_multiplier * penalty_factor as u128 / 10000;
            }
        }
        
        // Calculate final rate
        let effective_rate = (base_rate as u128 * total_multiplier / 10000) as u64;
        
        // Update stored values
        self.base_rate = base_rate;
        self.total_multiplier = std::cmp::min(total_multiplier as u32, u32::MAX);
        self.current_mining_rate = effective_rate;
        
        Ok(effective_rate)
    }
    
    /// Update referral bonus based on network size and activity
    pub fn update_referral_bonus(&mut self, direct_referrals: u32, active_referrals: u32) -> Result<()> {
        // Referral_Bonus = 1 + (Active_Referrals × 0.1), capped at 3.5x
        let bonus_multiplier = 1.0 + (active_referrals as f64 * 0.1);
        let capped_bonus = if bonus_multiplier > 3.5 { 3.5 } else { bonus_multiplier };
        
        self.referral_bonus = (capped_bonus * 10000.0) as u16;
        
        Ok(())
    }
    
    /// Update security bonus based on KYC status
    pub fn update_security_bonus(&mut self, kyc_verified: bool) -> Result<()> {
        self.security_bonus = if kyc_verified { 12000 } else { 8000 }; // 1.2x or 0.8x
        Ok(())
    }
    
    /// Update staking bonus based on staked amount
    pub fn update_staking_bonus(&mut self, staked_amount: u64) -> Result<()> {
        self.staking_bonus = match staked_amount {
            100..=499 => 12000,    // +20%
            500..=999 => 13500,    // +35%
            1000..=4999 => 15000,  // +50%
            5000..=9999 => 17500,  // +75%
            10000.. => 20000,      // +100%
            _ => 10000,            // No bonus
        };
        Ok(())
    }
    
    /// Apply special card effects
    pub fn apply_card_effects(&mut self, effects: &[crate::state::user::CardEffect], clock: &Clock) -> Result<()> {
        let mut card_multiplier = 10000u32; // Base 1.0x
        
        for effect in effects {
            // Check if effect is still active
            if clock.unix_timestamp <= effect.started_at + effect.duration as i64 {
                if effect.effect_type == 0 { // Mining boost effect
                    if effect.stackable {
                        card_multiplier += effect.multiplier as u32;
                    } else {
                        card_multiplier = std::cmp::max(card_multiplier, 10000 + effect.multiplier as u32);
                    }
                }
            }
        }
        
        self.card_effects_bonus = card_multiplier as u16;
        Ok(())
    }
    
    /// Update mining efficiency based on performance
    pub fn update_mining_efficiency(&mut self, session_duration: u32, expected_duration: u32) -> Result<()> {
        // Calculate efficiency as actual vs expected performance
        let efficiency = if expected_duration > 0 {
            (session_duration as f64 / expected_duration as f64).min(1.0)
        } else {
            0.5
        };
        
        // Smooth update: new_efficiency = 0.1 * current + 0.9 * old
        let current_efficiency = self.mining_efficiency as f64 / 10000.0;
        let updated_efficiency = 0.1 * efficiency + 0.9 * current_efficiency;
        
        self.mining_efficiency = (updated_efficiency * 10000.0) as u16;
        
        Ok(())
    }
    
    /// Record a mining session
    pub fn record_mining_session(&mut self, duration: u32, amount_mined: u64, clock: &Clock) -> Result<()> {
        self.last_mining_activity = clock.unix_timestamp;
        self.total_mining_sessions += 1;
        
        // Update average session duration
        let total_time = self.avg_session_duration as u64 * (self.total_mining_sessions - 1);
        self.avg_session_duration = ((total_time + duration as u64) / self.total_mining_sessions) as u32;
        
        // Update daily progress
        self.update_daily_progress(amount_mined, clock)?;
        
        // Update weekly progress
        self.update_weekly_progress(amount_mined, clock)?;
        
        // Update monthly progress
        self.update_monthly_progress(amount_mined, clock)?;
        
        Ok(())
    }
    
    /// Update daily mining progress
    fn update_daily_progress(&mut self, amount: u64, clock: &Clock) -> Result<()> {
        let current_day = self.get_day_number(clock.unix_timestamp);
        
        if self.daily_mining_progress.current_day != current_day {
            // New day, reset progress
            self.daily_mining_progress = DailyMiningProgress {
                current_day,
                fin_mined_today: amount,
                sessions_today: 1,
                mining_time_today: 0, // Would need to track this separately
                daily_limit: 2_400_000, // Default limit
                limit_reached: false,
                daily_bonus_multiplier: 10000,
            };
        } else {
            // Same day, update progress
            self.daily_mining_progress.fin_mined_today += amount;
            self.daily_mining_progress.sessions_today += 1;
            
            if self.daily_mining_progress.fin_mined_today >= self.daily_mining_progress.daily_limit {
                self.daily_mining_progress.limit_reached = true;
            }
        }
        
        Ok(())
    }
    
    /// Update weekly mining progress
    fn update_weekly_progress(&mut self, amount: u64, clock: &Clock) -> Result<()> {
        let current_week = self.get_week_number(clock.unix_timestamp);
        
        if self.weekly_mining_progress.current_week != current_week {
            // New week, reset progress
            self.weekly_mining_progress = WeeklyMiningProgress {
                current_week,
                fin_mined_week: amount,
                days_mined: 1,
                weekly_target: 10_000_000, // 10 FIN per week target
                weekly_bonus: 0,
                completion_percentage: 0,
            };
        } else {
            // Same week, update progress
            self.weekly_mining_progress.fin_mined_week += amount;
            // days_mined would be updated based on unique days
        }
        
        // Calculate completion percentage
        if self.weekly_mining_progress.weekly_target > 0 {
            self.weekly_mining_progress.completion_percentage = 
                std::cmp::min(100, 
                    (self.weekly_mining_progress.fin_mined_week * 100 / self.weekly_mining_progress.weekly_target) as u8
                );
        }
        
        Ok(())
    }
    
    /// Update monthly mining progress
    fn update_monthly_progress(&mut self, amount: u64, clock: &Clock) -> Result<()> {
        let current_month = self.get_month_number(clock.unix_timestamp);
        
        if self.monthly_mining_progress.current_month != current_month {
            // New month, reset progress
            self.monthly_mining_progress = MonthlyMiningProgress {
                current_month,
                fin_mined_month: amount,
                days_mined: 1,
                monthly_target: 50_000_000, // 50 FIN per month target
                monthly_rank: 0, // Would be calculated by ranking system
                performance_tier: 1, // Bronze tier by default
            };
        } else {
            // Same month, update progress
            self.monthly_mining_progress.fin_mined_month += amount;
        }
        
        Ok(())
    }
    
    /// Get day number in YYYYMMDD format
    fn get_day_number(&self, timestamp: i64) -> u32 {
        // Simple conversion - in production would use proper date library
        let days_since_epoch = timestamp / 86400;
        let year = 1970 + days_since_epoch / 365; // Approximation
        let day_of_year = days_since_epoch % 365;
        let month = day_of_year / 30 + 1; // Rough approximation
        let day = day_of_year % 30 + 1;
        
        (year * 10000 + month * 100 + day) as u32
    }
    
    /// Get week number in YYYYWW format  
    fn get_week_number(&self, timestamp: i64) -> u32 {
        let weeks_since_epoch = timestamp / (86400 * 7);
        let year = 1970 + weeks_since_epoch / 52;
        let week = weeks_since_epoch % 52 + 1;
        
        (year * 100 + week) as u32
    }
    
    /// Get month number in YYYYMM format
    fn get_month_number(&self, timestamp: i64) -> u32 {
        let months_since_epoch = timestamp / (86400 * 30); // Approximation
        let year = 1970 + months_since_epoch / 12;
        let month = months_since_epoch % 12 + 1;
        
        (year * 100 + month) as u32
    }
    
    /// Apply mining penalty
    pub fn apply_penalty(&mut self, penalty_type: u8, severity: u16, duration: u64, reason_code: u16, clock: &Clock) -> Result<()> {
        let penalty = MiningPenalty {
            penalty_type,
            severity,
            applied_at: clock.unix_timestamp,
            duration,
            reason_code,
            active: true,
        };
        
        self.penalties.push(penalty);
        
        // Recalculate mining rate with penalty applied
        // This would typically trigger a rate recalculation
        
        Ok(())
    }
    
    /// Remove expired penalties
    pub fn clean_expired_penalties(&mut self, clock: &Clock) -> Result<()> {
        for penalty in &mut self.penalties {
            if penalty.active && penalty.duration > 0 {
                if clock.unix_timestamp >= penalty.applied_at + penalty.duration as i64 {
                    penalty.active = false;
                }
            }
        }
        
        // Remove inactive penalties to save space
        self.penalties.retain(|p| p.active || p.duration == 0);
        
        Ok(())
    }
    
    /// Update bot detection scores
    pub fn update_bot_detection(&mut self, behavioral_score: u16, timing_score: u16, device_score: u16, social_score: u16, clock: &Clock) -> Result<()> {
        self.bot_detection.behavioral_score = behavioral_score;
        self.bot_detection.timing_score = timing_score;
        self.bot_detection.device_score = device_score;
        self.bot_detection.social_score = social_score;
        self.bot_detection.last_check = clock.unix_timestamp;
        
        // Calculate overall bot probability
        let scores = [behavioral_score, timing_score, device_score, social_score];
        let avg_score = scores.iter().sum::<u16>() / scores.len() as u16;
        
        // Bot probability is inverse of average score
        self.bot_detection.bot_probability = 10000 - avg_score;
        
        // Update failed checks if scores are too low
        if avg_score < 3000 { // Below 30% human-like behavior
            self.bot_detection.failed_checks += 1;
            
            // Apply automatic penalties for repeated failures
            if self.bot_detection.failed_checks >= 5 {
                self.apply_penalty(
                    1, // Rate reduction penalty
                    5000, // 50% reduction
                    86400 * 7, // 7 days
                    1001, // Bot detection reason code
                    clock
                )?;
            }
        } else if avg_score > 7000 { // Above 70% human-like
            // Reset failed checks for good behavior
            self.bot_detection.failed_checks = self.bot_detection.failed_checks.saturating_sub(1);
        }
        
        Ok(())
    }
    
    /// Calculate regression factor for whale protection
    pub fn calculate_regression_factor(&mut self, total_holdings: u64) -> Result<()> {
        // Regression_Factor = e^(-0.001 × User_Total_Holdings)
        // Approximated for integer math
        
        if total_holdings <= 1000 {
            self.regression_factor = 10000; // No regression
        } else if total_holdings <= 5000 {
            self.regression_factor = 8000; // 20% reduction
        } else if total_holdings <= 10000 {
            self.regression_factor = 5000; // 50% reduction
        } else if total_holdings <= 50000 {
            self.regression_factor = 2000; // 80% reduction
        } else {
            self.regression_factor = 100; // 99% reduction (minimum 1%)
        }
        
        Ok(())
    }
    
    /// Add mining achievement
    pub fn add_achievement(&mut self, achievement_id: u32, tier: u8, bonus_rate: u16, expires_at: Option<i64>, clock: &Clock) -> Result<()> {
        let achievement = MiningAchievement {
            achievement_id,
            unlocked_at: clock.unix_timestamp,
            tier,
            bonus_rate,
            expires_at,
        };
        
        self.mining_achievements.push(achievement);
        
        Ok(())
    }
    
    /// Get active achievement bonuses
    pub fn get_active_achievement_bonus(&self, clock: &Clock) -> u16 {
        let mut total_bonus = 0u16;
        
        for achievement in &self.mining_achievements {
            // Check if achievement is still active
            if let Some(expires_at) = achievement.expires_at {
                if clock.unix_timestamp > expires_at {
                    continue; // Expired
                }
            }
            
            total_bonus += achievement.bonus_rate;
        }
        
        total_bonus
    }
    
    /// Check if daily mining limit is reached
    pub fn is_daily_limit_reached(&self) -> bool {
        self.daily_mining_progress.limit_reached
    }
    
    /// Get mining performance metrics
    pub fn get_performance_metrics(&self) -> MiningPerformanceMetrics {
        MiningPerformanceMetrics {
            efficiency_score: self.mining_efficiency,
            consistency_score: self.bot_detection.behavioral_score,
            streak_bonus: std::cmp::min(self.consecutive_mining_days * 50, 1500) as u16,
            achievement_bonus: self.mining_achievements.len() as u16 * 100,
            penalty_factor: self.get_active_penalty_factor(),
            overall_rating: self.calculate_overall_rating(),
        }
    }
    
    /// Get active penalty factor
    fn get_active_penalty_factor(&self) -> u16 {
        let mut total_penalty = 0u16;
        
        for penalty in &self.penalties {
            if penalty.active {
                total_penalty += penalty.severity;
            }
        }
        
        std::cmp::min(total_penalty, 9000) // Maximum 90% penalty
    }
    
    /// Calculate overall mining rating (0-10000)
    fn calculate_overall_rating(&self) -> u16 {
        let mut rating = 5000u32; // Base 50% rating
        
        // Add bonuses
        rating += self.mining_efficiency as u32 / 2; // Up to +50%
        rating += std::cmp::min(self.consecutive_mining_days * 10, 1000) as u32; // Up to +10%
        rating += self.mining_achievements.len() as u32 * 50; // +5% per achievement
        
        // Subtract penalties
        rating = rating.saturating_sub(self.get_active_penalty_factor() as u32);
        rating = rating.saturating_sub(self.bot_detection.bot_probability as u32 / 2);
        
        std::cmp::min(rating as u16, 10000)
    }
}

/// Mining performance metrics structure
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default)]
pub struct MiningPerformanceMetrics {
    pub efficiency_score: u16,
    pub consistency_score: u16,
    pub streak_bonus: u16,
    pub achievement_bonus: u16,
    pub penalty_factor: u16,
    pub overall_rating: u16,
}

/// Mining session data structure
#[account]
#[derive(Default)]
pub struct MiningSession {
    /// User conducting the session
    pub user: Pubkey,
    
    /// Session start timestamp
    pub started_at: i64,
    
    /// Session end timestamp (0 if ongoing)
    pub ended_at: i64,
    
    /// Session duration in seconds
    pub duration: u32,
    
    /// FIN amount mined in this session
    pub fin_mined: u64,
    
    /// Mining rate during session
    pub mining_rate: u64,
    
    /// Session type (manual, auto, scheduled)
    pub session_type: u8,
    
    /// Quality score for this session
    pub quality_score: u16,
    
    /// Bot detection score for session
    pub bot_score: u16,
    
    /// Social activities during session
    pub social_activities: u32,
    
    /// Platform interactions count
    pub platform_interactions: u32,
    
    /// Session efficiency rating
    pub efficiency_rating: u16,
    
    /// Bonuses applied during session
    pub bonuses_applied: Vec<SessionBonus>,
    
    /// Penalties applied during session
    pub penalties_applied: Vec<SessionPenalty>,
    
    /// Session metadata
    pub metadata: SessionMetadata,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default)]
pub struct SessionBonus {
    pub bonus_type: u8,
    pub bonus_amount: u16,
    pub bonus_reason: String,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default)]
pub struct SessionPenalty {
    pub penalty_type: u8,
    pub penalty_amount: u16,
    pub penalty_reason: String,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default)]
pub struct SessionMetadata {
    pub device_fingerprint: Vec<u8>,
    pub ip_hash: Vec<u8>,
    pub platform_used: u8,
    pub app_version: String,
    pub session_id: String,
}

impl MiningSession {
    pub const LEN: usize = 8 + // discriminator
        32 + // user
        8 + // started_at
        8 + // ended_at
        4 + // duration
        8 + // fin_mined
        8 + // mining_rate
        1 + // session_type
        2 + // quality_score
        2 + // bot_score
        4 + // social_activities
        4 + // platform_interactions
        2 + // efficiency_rating
        4 + 10 * 32 + // bonuses_applied (assume max 10)
        4 + 5 * 32 + // penalties_applied (assume max 5)
        128; // metadata
    
    /// Start a new mining session
    pub fn start_session(&mut self, user: Pubkey, session_type: u8, mining_rate: u64, clock: &Clock) -> Result<()> {
        self.user = user;
        self.started_at = clock.unix_timestamp;
        self.ended_at = 0;
        self.duration = 0;
        self.fin_mined = 0;
        self.mining_rate = mining_rate;
        self.session_type = session_type;
        self.quality_score = 5000; // Default neutral quality
        self.bot_score = 5000; // Default neutral bot score
        self.social_activities = 0;
        self.platform_interactions = 0;
        self.efficiency_rating = 5000;
        self.bonuses_applied = Vec::new();
        self.penalties_applied = Vec::new();
        self.metadata = SessionMetadata::default();
        
        Ok(())
    }
    
    /// End the mining session
    pub fn end_session(&mut self, clock: &Clock) -> Result<u64> {
        if self.ended_at != 0 {
            return Err(ErrorCode::SessionAlreadyEnded.into());
        }
        
        self.ended_at = clock.unix_timestamp;
        self.duration = (self.ended_at - self.started_at) as u32;
        
        // Calculate final mined amount based on duration and rate
        let hours = self.duration as f64 / 3600.0;
        self.fin_mined = (self.mining_rate as f64 * hours) as u64;
        
        // Apply session bonuses and penalties
        let mut final_amount = self.fin_mined;
        
        for bonus in &self.bonuses_applied {
            final_amount += (final_amount as u128 * bonus.bonus_amount as u128 / 10000) as u64;
        }
        
        for penalty in &self.penalties_applied {
            final_amount = final_amount.saturating_sub((final_amount as u128 * penalty.penalty_amount as u128 / 10000) as u64);
        }
        
        self.fin_mined = final_amount;
        
        Ok(final_amount)
    }
    
    /// Add bonus to session
    pub fn add_bonus(&mut self, bonus_type: u8, bonus_amount: u16, reason: String) -> Result<()> {
        let bonus = SessionBonus {
            bonus_type,
            bonus_amount,
            bonus_reason: reason,
        };
        
        self.bonuses_applied.push(bonus);
        Ok(())
    }
    
    /// Add penalty to session
    pub fn add_penalty(&mut self, penalty_type: u8, penalty_amount: u16, reason: String) -> Result<()> {
        let penalty = SessionPenalty {
            penalty_type,
            penalty_amount,
            penalty_reason: reason,
        };
        
        self.penalties_applied.push(penalty);
        Ok(())
    }
    
    /// Update session quality score
    pub fn update_quality_score(&mut self, score: u16) -> Result<()> {
        self.quality_score = std::cmp::min(score, 10000);
        Ok(())
    }
    
    /// Record social activity during session
    pub fn record_social_activity(&mut self) -> Result<()> {
        self.social_activities += 1;
        
        // Bonus for social engagement
        if self.social_activities % 5 == 0 {
            self.add_bonus(1, 500, "Social engagement bonus".to_string())?;
        }
        
        Ok(())
    }
    
    /// Record platform interaction
    pub fn record_platform_interaction(&mut self) -> Result<()> {
        self.platform_interactions += 1;
        Ok(())
    }
    
    /// Calculate session efficiency
    pub fn calculate_efficiency(&self, expected_duration: u32) -> u16 {
        if expected_duration == 0 || self.duration == 0 {
            return 5000; // Neutral efficiency
        }
        
        let efficiency = if self.duration <= expected_duration {
            10000 // Perfect efficiency
        } else {
            // Efficiency decreases with longer sessions
            let ratio = expected_duration as f64 / self.duration as f64;
            (ratio * 10000.0) as u16
        };
        
        std::cmp::min(efficiency, 10000)
    }
}

/// Network-wide mining statistics
#[account]
#[derive(Default)]
pub struct NetworkMiningStats {
    /// Total network hash rate
    pub total_hash_rate: u64,
    
    /// Average mining rate per user
    pub avg_mining_rate: u64,
    
    /// Peak concurrent miners in last 24h
    pub peak_concurrent_miners: u32,
    
    /// Total mining sessions today
    pub daily_sessions: u64,
    
    /// Total FIN mined today
    pub daily_mining_volume: u64,
    
    /// Top miners leaderboard
    pub top_miners: Vec<TopMiner>,
    
    /// Mining difficulty adjustments
    pub difficulty_history: Vec<DifficultyAdjustment>,
    
    /// Network health metrics
    pub network_health: NetworkHealth,
    
    /// Last update timestamp
    pub last_updated: i64,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default)]
pub struct TopMiner {
    pub user: Pubkey,
    pub total_mined: u64,
    pub mining_rate: u64,
    pub rank: u32,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default)]
pub struct DifficultyAdjustment {
    pub timestamp: i64,
    pub old_difficulty: u16,
    pub new_difficulty: u16,
    pub reason: String,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default)]
pub struct NetworkHealth {
    pub active_miners_ratio: u16, // Percentage of active vs total miners
    pub average_session_duration: u32,
    pub bot_detection_accuracy: u16,
    pub quality_score_average: u16,
    pub network_stability_score: u16,
}

impl NetworkMiningStats {
    pub const LEN: usize = 8 + // discriminator
        8 + // total_hash_rate
        8 + // avg_mining_rate
        4 + // peak_concurrent_miners
        8 + // daily_sessions
        8 + // daily_mining_volume
        4 + 100 * 48 + // top_miners (assume top 100)
        4 + 50 * 64 + // difficulty_history (assume 50 adjustments)
        64 + // network_health
        8; // last_updated
    
    /// Update network statistics
    pub fn update_stats(&mut self, total_users: u64, active_miners: u64, daily_volume: u64, clock: &Clock) -> Result<()> {
        self.daily_mining_volume = daily_volume;
        self.daily_sessions += 1;
        
        if active_miners > 0 {
            self.avg_mining_rate = daily_volume / active_miners;
        }
        
        // Update network health
        if total_users > 0 {
            self.network_health.active_miners_ratio = ((active_miners * 10000) / total_users) as u16;
        }
        
        self.last_updated = clock.unix_timestamp;
        
        Ok(())
    }
    
    /// Add difficulty adjustment
    pub fn add_difficulty_adjustment(&mut self, old_difficulty: u16, new_difficulty: u16, reason: String, clock: &Clock) -> Result<()> {
        let adjustment = DifficultyAdjustment {
            timestamp: clock.unix_timestamp,
            old_difficulty,
            new_difficulty,
            reason,
        };
        
        self.difficulty_history.push(adjustment);
        
        // Keep only last 50 adjustments
        if self.difficulty_history.len() > 50 {
            self.difficulty_history.remove(0);
        }
        
        Ok(())
    }
}
