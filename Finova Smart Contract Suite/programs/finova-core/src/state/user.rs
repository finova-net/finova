// programs/finova-core/src/state/user.rs

use anchor_lang::prelude::*;
use std::collections::HashMap;

#[account]
#[derive(Default)]
pub struct User {
    /// User's wallet public key
    pub authority: Pubkey,
    
    /// Unique user ID
    pub user_id: u64,
    
    /// User registration timestamp
    pub created_at: i64,
    
    /// Last activity timestamp
    pub last_activity: i64,
    
    /// KYC verification status
    pub kyc_verified: bool,
    
    /// KYC verification timestamp
    pub kyc_verified_at: Option<i64>,
    
    /// User's current level based on XP
    pub level: u32,
    
    /// Total experience points
    pub total_xp: u64,
    
    /// Current XP in current level
    pub current_level_xp: u64,
    
    /// XP required for next level
    pub next_level_xp: u64,
    
    /// Daily XP gained today
    pub daily_xp: u64,
    
    /// Daily XP reset timestamp
    pub daily_xp_reset: i64,
    
    /// Current daily streak
    pub daily_streak: u32,
    
    /// Longest daily streak achieved
    pub longest_streak: u32,
    
    /// Last daily login timestamp
    pub last_daily_login: i64,
    
    /// Total $FIN tokens mined
    pub total_fin_mined: u64,
    
    /// Total $FIN tokens currently held
    pub fin_balance: u64,
    
    /// Total $FIN tokens staked
    pub staked_fin: u64,
    
    /// Staking start timestamp
    pub staking_started_at: Option<i64>,
    
    /// Current mining rate per hour (in micro-FIN)
    pub mining_rate: u64,
    
    /// Last mining claim timestamp
    pub last_mining_claim: i64,
    
    /// Unclaimed mining rewards
    pub unclaimed_mining: u64,
    
    /// Mining phase when user joined (1-4)
    pub mining_phase: u8,
    
    /// Referral code used during registration
    pub referral_code_used: Option<String>,
    
    /// User's unique referral code
    pub referral_code: String,
    
    /// Direct referrals count
    pub direct_referrals: u32,
    
    /// Total network size (all levels)
    pub total_network_size: u32,
    
    /// Active referrals (active in last 30 days)
    pub active_referrals: u32,
    
    /// Total referral points earned
    pub total_rp: u64,
    
    /// Current RP tier (0-4: Explorer, Connector, Influencer, Leader, Ambassador)
    pub rp_tier: u8,
    
    /// Network quality score (0.0 - 1.0)
    pub network_quality_score: u32, // Stored as u32 for precision (multiply by 10000)
    
    /// Bot probability score (0.0 - 1.0, lower is more human)
    pub bot_probability: u32, // Stored as u32 for precision (multiply by 10000)
    
    /// Human verification score (0.0 - 1.0, higher is more human)
    pub human_verification_score: u32, // Stored as u32 for precision (multiply by 10000)
    
    /// Number of suspicious activities detected
    pub suspicious_activities: u32,
    
    /// Account restriction level (0: none, 1: warning, 2: limited, 3: suspended)
    pub restriction_level: u8,
    
    /// Restriction timestamp
    pub restricted_at: Option<i64>,
    
    /// Total content posts made
    pub total_posts: u64,
    
    /// Total comments made
    pub total_comments: u64,
    
    /// Total likes given
    pub total_likes_given: u64,
    
    /// Total likes received
    pub total_likes_received: u64,
    
    /// Total shares made
    pub total_shares: u64,
    
    /// Total viral content created (1K+ views)
    pub viral_content_count: u32,
    
    /// Current guild membership
    pub guild_id: Option<u64>,
    
    /// Guild join timestamp
    pub guild_joined_at: Option<i64>,
    
    /// Guild role (0: member, 1: officer, 2: leader)
    pub guild_role: u8,
    
    /// Total NFTs owned
    pub nft_count: u32,
    
    /// Special cards owned count
    pub special_cards_count: u32,
    
    /// Active special card effects
    pub active_card_effects: Vec<CardEffect>,
    
    /// Social platform connections bitmask
    /// Bit 0: Instagram, Bit 1: TikTok, Bit 2: YouTube, Bit 3: Facebook, Bit 4: X/Twitter
    pub connected_platforms: u8,
    
    /// Platform-specific user IDs (encrypted)
    pub platform_user_ids: Vec<PlatformConnection>,
    
    /// User preferences and settings
    pub preferences: UserPreferences,
    
    /// Achievement badges earned
    pub achievements: Vec<Achievement>,
    
    /// Statistics for analytics
    pub stats: UserStats,
    
    /// Reserved space for future upgrades
    pub _reserved: [u8; 128],
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default)]
pub struct CardEffect {
    /// Card type ID
    pub card_type: u16,
    
    /// Effect type (mining boost, XP boost, etc.)
    pub effect_type: u8,
    
    /// Effect multiplier (stored as percentage * 100)
    pub multiplier: u16,
    
    /// Effect start timestamp
    pub started_at: i64,
    
    /// Effect duration in seconds
    pub duration: u32,
    
    /// Whether effect is stackable
    pub stackable: bool,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default)]
pub struct PlatformConnection {
    /// Platform ID (0: Instagram, 1: TikTok, 2: YouTube, 3: Facebook, 4: X)
    pub platform_id: u8,
    
    /// Encrypted platform user ID
    pub encrypted_user_id: Vec<u8>,
    
    /// Connection timestamp
    pub connected_at: i64,
    
    /// Last sync timestamp
    pub last_sync: i64,
    
    /// Connection status (active, suspended, etc.)
    pub status: u8,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default)]
pub struct UserPreferences {
    /// Notification settings bitmask
    pub notifications: u16,
    
    /// Privacy settings bitmask
    pub privacy: u16,
    
    /// Auto-mining enabled
    pub auto_mining: bool,
    
    /// Auto-claim rewards enabled
    pub auto_claim: bool,
    
    /// Language preference (ISO code)
    pub language: String,
    
    /// Timezone offset in minutes
    pub timezone_offset: i16,
    
    /// Currency preference for display
    pub currency: String,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default)]
pub struct Achievement {
    /// Achievement ID
    pub achievement_id: u32,
    
    /// Achievement earned timestamp
    pub earned_at: i64,
    
    /// Achievement tier/level
    pub tier: u8,
    
    /// Associated rewards claimed
    pub rewards_claimed: bool,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default)]
pub struct UserStats {
    /// Total session time in seconds
    pub total_session_time: u64,
    
    /// Average session duration in seconds
    pub avg_session_duration: u32,
    
    /// Total transactions made
    pub total_transactions: u64,
    
    /// Total gas fees paid
    pub total_gas_paid: u64,
    
    /// Favorite platform (most activity)
    pub favorite_platform: u8,
    
    /// Peak daily XP earned
    pub peak_daily_xp: u64,
    
    /// Peak mining rate achieved
    pub peak_mining_rate: u64,
    
    /// Total referral rewards earned
    pub total_referral_rewards: u64,
    
    /// Best monthly performance metrics
    pub best_monthly_performance: MonthlyPerformance,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default)]
pub struct MonthlyPerformance {
    /// Month/year (YYYYMM format)
    pub period: u32,
    
    /// XP earned in that month
    pub xp_earned: u64,
    
    /// FIN mined in that month
    pub fin_mined: u64,
    
    /// Content created count
    pub content_created: u32,
    
    /// Referrals brought in
    pub new_referrals: u32,
}

impl User {
    pub const LEN: usize = 8 + // discriminator
        32 + // authority
        8 + // user_id  
        8 + // created_at
        8 + // last_activity
        1 + // kyc_verified
        9 + // kyc_verified_at (Option<i64>)
        4 + // level
        8 + // total_xp
        8 + // current_level_xp
        8 + // next_level_xp
        8 + // daily_xp
        8 + // daily_xp_reset
        4 + // daily_streak
        4 + // longest_streak
        8 + // last_daily_login
        8 + // total_fin_mined
        8 + // fin_balance
        8 + // staked_fin
        9 + // staking_started_at
        8 + // mining_rate
        8 + // last_mining_claim
        8 + // unclaimed_mining
        1 + // mining_phase
        4 + 32 + // referral_code_used (Option<String>)
        4 + 32 + // referral_code
        4 + // direct_referrals
        4 + // total_network_size
        4 + // active_referrals
        8 + // total_rp
        1 + // rp_tier
        4 + // network_quality_score
        4 + // bot_probability
        4 + // human_verification_score
        4 + // suspicious_activities
        1 + // restriction_level
        9 + // restricted_at
        8 + // total_posts
        8 + // total_comments
        8 + // total_likes_given
        8 + // total_likes_received
        8 + // total_shares
        4 + // viral_content_count
        9 + // guild_id
        9 + // guild_joined_at
        1 + // guild_role
        4 + // nft_count
        4 + // special_cards_count
        4 + 10 * 32 + // active_card_effects (Vec, assume max 10)
        1 + // connected_platforms
        4 + 5 * 64 + // platform_user_ids (Vec, assume max 5 platforms)
        128 + // preferences (estimate)
        4 + 20 * 16 + // achievements (Vec, assume max 20)
        256 + // stats (estimate)
        128; // _reserved
    
    /// Initialize a new user account
    pub fn initialize(
        &mut self,
        authority: Pubkey,
        user_id: u64,
        referral_code: String,
        referral_code_used: Option<String>,
        clock: &Clock,
    ) -> Result<()> {
        self.authority = authority;
        self.user_id = user_id;
        self.created_at = clock.unix_timestamp;
        self.last_activity = clock.unix_timestamp;
        self.kyc_verified = false;
        self.level = 1;
        self.total_xp = 0;
        self.current_level_xp = 0;
        self.next_level_xp = 100; // First level requires 100 XP
        self.daily_xp = 0;
        self.daily_xp_reset = clock.unix_timestamp;
        self.daily_streak = 0;
        self.longest_streak = 0;
        self.last_daily_login = 0;
        self.total_fin_mined = 0;
        self.fin_balance = 0;
        self.staked_fin = 0;
        self.mining_rate = self.calculate_initial_mining_rate();
        self.last_mining_claim = clock.unix_timestamp;
        self.unclaimed_mining = 0;
        self.mining_phase = self.determine_current_phase();
        self.referral_code_used = referral_code_used;
        self.referral_code = referral_code;
        self.direct_referrals = 0;
        self.total_network_size = 0;
        self.active_referrals = 0;
        self.total_rp = 0;
        self.rp_tier = 0; // Explorer
        self.network_quality_score = 10000; // 1.0 * 10000
        self.bot_probability = 0;
        self.human_verification_score = 5000; // 0.5 * 10000 (neutral)
        self.suspicious_activities = 0;
        self.restriction_level = 0;
        self.total_posts = 0;
        self.total_comments = 0;
        self.total_likes_given = 0;
        self.total_likes_received = 0;
        self.total_shares = 0;
        self.viral_content_count = 0;
        self.guild_role = 0;
        self.nft_count = 0;
        self.special_cards_count = 0;
        self.active_card_effects = Vec::new();
        self.connected_platforms = 0;
        self.platform_user_ids = Vec::new();
        self.preferences = UserPreferences::default();
        self.achievements = Vec::new();
        self.stats = UserStats::default();
        
        Ok(())
    }
    
    /// Calculate initial mining rate based on current network size
    fn calculate_initial_mining_rate(&self) -> u64 {
        // This would be replaced with actual network size query
        let total_users = 50000; // Placeholder
        
        match total_users {
            0..=100_000 => 100_000, // 0.1 FIN/hour in micro-FIN
            100_001..=1_000_000 => 50_000, // 0.05 FIN/hour
            1_000_001..=10_000_000 => 25_000, // 0.025 FIN/hour
            _ => 10_000, // 0.01 FIN/hour
        }
    }
    
    /// Determine current mining phase
    fn determine_current_phase(&self) -> u8 {
        // This would be replaced with actual network size query
        let total_users = 50000; // Placeholder
        
        match total_users {
            0..=100_000 => 1,
            100_001..=1_000_000 => 2,
            1_000_001..=10_000_000 => 3,
            _ => 4,
        }
    }
    
    /// Update last activity timestamp
    pub fn update_activity(&mut self, clock: &Clock) {
        self.last_activity = clock.unix_timestamp;
    }
    
    /// Add XP and handle level progression
    pub fn add_xp(&mut self, xp_amount: u64, clock: &Clock) -> Result<bool> {
        let old_level = self.level;
        
        // Reset daily XP if it's a new day
        self.reset_daily_xp_if_needed(clock);
        
        // Add XP
        self.total_xp = self.total_xp.checked_add(xp_amount).unwrap();
        self.current_level_xp = self.current_level_xp.checked_add(xp_amount).unwrap();
        self.daily_xp = self.daily_xp.checked_add(xp_amount).unwrap();
        
        // Check for level up
        let mut leveled_up = false;
        while self.current_level_xp >= self.next_level_xp {
            self.level_up();
            leveled_up = true;
        }
        
        // Update activity
        self.update_activity(clock);
        
        Ok(leveled_up)
    }
    
    /// Handle level progression
    fn level_up(&mut self) {
        self.current_level_xp -= self.next_level_xp;
        self.level += 1;
        
        // Calculate next level XP requirement (exponential growth)
        self.next_level_xp = self.calculate_next_level_xp_requirement();
    }
    
    /// Calculate XP requirement for next level
    fn calculate_next_level_xp_requirement(&self) -> u64 {
        // Exponential growth: base * level^1.5
        let base = 100.0;
        let level_f64 = self.level as f64;
        (base * level_f64.powf(1.5)) as u64
    }
    
    /// Reset daily XP counter if needed
    fn reset_daily_xp_if_needed(&mut self, clock: &Clock) {
        let current_day = clock.unix_timestamp / 86400; // Seconds in a day
        let reset_day = self.daily_xp_reset / 86400;
        
        if current_day > reset_day {
            self.daily_xp = 0;
            self.daily_xp_reset = clock.unix_timestamp;
        }
    }
    
    /// Update daily login streak
    pub fn update_daily_login(&mut self, clock: &Clock) -> Result<u32> {
        let current_day = clock.unix_timestamp / 86400;
        let last_login_day = self.last_daily_login / 86400;
        
        self.last_daily_login = clock.unix_timestamp;
        
        if current_day == last_login_day + 1 {
            // Consecutive day
            self.daily_streak += 1;
            if self.daily_streak > self.longest_streak {
                self.longest_streak = self.daily_streak;
            }
        } else if current_day > last_login_day + 1 {
            // Streak broken
            self.daily_streak = 1;
        }
        // If same day, don't change streak
        
        self.update_activity(clock);
        Ok(self.daily_streak)
    }
    
    /// Get current mining multiplier based on level, staking, etc.
    pub fn get_mining_multiplier(&self) -> u32 {
        let mut multiplier = 10000; // Base 1.0x multiplier (stored as 10000)
        
        // XP Level bonus: 1.0x + (level / 100)
        multiplier += (self.level * 100) as u32;
        
        // Staking bonus
        if self.staked_fin > 0 {
            let staking_bonus = match self.staked_fin {
                100..=499 => 2000,        // +20%
                500..=999 => 3500,        // +35%  
                1000..=4999 => 5000,      // +50%
                5000..=9999 => 7500,      // +75%
                _ => 10000,               // +100%
            };
            multiplier += staking_bonus;
        }
        
        // RP tier bonus
        let rp_bonus = match self.rp_tier {
            1 => 2000,  // Connector: +20%
            2 => 5000,  // Influencer: +50%
            3 => 10000, // Leader: +100%
            4 => 20000, // Ambassador: +200%
            _ => 0,     // Explorer: +0%
        };
        multiplier += rp_bonus;
        
        // Daily streak bonus
        let streak_bonus = std::cmp::min(self.daily_streak * 50, 1500) as u32; // Max +15%
        multiplier += streak_bonus;
        
        // Apply regression for large holders
        if self.total_fin_mined > 1000 {
            let regression_factor = self.calculate_regression_factor();
            multiplier = (multiplier as u64 * regression_factor / 10000) as u32;
        }
        
        multiplier
    }
    
    /// Calculate exponential regression factor for anti-whale mechanism
    fn calculate_regression_factor(&self) -> u64 {
        // e^(-0.001 * total_holdings) approximated for integer math
        let holdings = self.total_fin_mined;
        let factor = if holdings > 10000 {
            100 // Very small factor for whales
        } else if holdings > 5000 {
            500
        } else if holdings > 1000 {
            2000
        } else {
            8000
        };
        
        std::cmp::max(factor, 100) // Minimum 1% of original rate
    }
    
    /// Calculate pending mining rewards
    pub fn calculate_pending_mining(&self, clock: &Clock) -> u64 {
        let time_diff = clock.unix_timestamp - self.last_mining_claim;
        let hours_passed = time_diff / 3600; // Convert seconds to hours
        
        if hours_passed <= 0 {
            return 0;
        }
        
        let base_rate = self.mining_rate;
        let multiplier = self.get_mining_multiplier();
        
        // Apply active card effects
        let card_multiplier = self.get_active_card_multiplier(clock);
        
        let pending = (base_rate as u128 * hours_passed as u128 * multiplier as u128 * card_multiplier as u128) / (10000 * 10000) as u128;
        
        std::cmp::min(pending as u64, u64::MAX)
    }
    
    /// Get multiplier from active special cards
    fn get_active_card_multiplier(&self, clock: &Clock) -> u32 {
        let mut multiplier = 10000; // Base 1.0x
        
        for effect in &self.active_card_effects {
            // Check if effect is still active
            if clock.unix_timestamp <= effect.started_at + effect.duration as i64 {
                if effect.stackable {
                    multiplier += effect.multiplier as u32;
                } else {
                    multiplier = std::cmp::max(multiplier, 10000 + effect.multiplier as u32);
                }
            }
        }
        
        multiplier
    }
    
    /// Claim pending mining rewards
    pub fn claim_mining_rewards(&mut self, clock: &Clock) -> Result<u64> {
        let pending = self.calculate_pending_mining(clock);
        
        if pending > 0 {
            self.fin_balance = self.fin_balance.checked_add(pending).unwrap();
            self.total_fin_mined = self.total_fin_mined.checked_add(pending).unwrap();
            self.last_mining_claim = clock.unix_timestamp;
            
            // Clean up expired card effects
            self.clean_expired_card_effects(clock);
            
            self.update_activity(clock);
        }
        
        Ok(pending)
    }
    
    /// Remove expired card effects
    fn clean_expired_card_effects(&mut self, clock: &Clock) {
        self.active_card_effects.retain(|effect| {
            clock.unix_timestamp <= effect.started_at + effect.duration as i64
        });
    }
    
    /// Add special card effect
    pub fn add_card_effect(&mut self, effect: CardEffect) -> Result<()> {
        // Check if we can stack this effect
        if effect.stackable {
            self.active_card_effects.push(effect);
        } else {
            // Remove existing effects of same type and add new one
            self.active_card_effects.retain(|e| e.effect_type != effect.effect_type);
            self.active_card_effects.push(effect);
        }
        
        Ok(())
    }
    
    /// Update KYC status
    pub fn set_kyc_verified(&mut self, verified: bool, clock: &Clock) -> Result<()> {
        self.kyc_verified = verified;
        if verified {
            self.kyc_verified_at = Some(clock.unix_timestamp);
            // KYC users get 20% mining bonus
            self.mining_rate = (self.mining_rate as u128 * 12000 / 10000) as u64;
        }
        self.update_activity(clock);
        Ok(())
    }
    
    /// Update referral statistics
    pub fn update_referral_stats(&mut self, direct_referrals: u32, total_network: u32, active_referrals: u32) -> Result<()> {
        self.direct_referrals = direct_referrals;
        self.total_network_size = total_network;
        self.active_referrals = active_referrals;
        
        // Update RP tier based on network size and quality
        self.update_rp_tier();
        
        Ok(())
    }
    
    /// Update RP tier based on network metrics
    fn update_rp_tier(&mut self) {
        let network_quality = self.network_quality_score as f64 / 10000.0;
        let effective_network = (self.active_referrals as f64 * network_quality) as u32;
        
        self.rp_tier = match effective_network {
            0..=9 => 0,      // Explorer
            10..=24 => 1,    // Connector  
            25..=49 => 2,    // Influencer
            50..=99 => 3,    // Leader
            _ => 4,          // Ambassador
        };
    }
    
    /// Update social activity statistics
    pub fn update_social_stats(&mut self, posts: u64, comments: u64, likes_given: u64, likes_received: u64, shares: u64, viral_count: u32) -> Result<()> {
        self.total_posts = posts;
        self.total_comments = comments;
        self.total_likes_given = likes_given;
        self.total_likes_received = likes_received;
        self.total_shares = shares;
        self.viral_content_count = viral_count;
        
        Ok(())
    }
    
    /// Check if user can perform action based on restrictions
    pub fn can_perform_action(&self, action_type: u8) -> bool {
        match self.restriction_level {
            0 => true,  // No restrictions
            1 => true,  // Warning only
            2 => {      // Limited access
                match action_type {
                    0 => true,  // Basic actions allowed
                    1 => false, // Mining limited
                    2 => false, // Referrals limited
                    _ => true,
                }
            }
            3 => false, // Suspended
            _ => false,
        }
    }
    
    /// Update bot probability and human verification scores
    pub fn update_verification_scores(&mut self, bot_probability: f64, human_score: f64) -> Result<()> {
        self.bot_probability = (bot_probability * 10000.0) as u32;
        self.human_verification_score = (human_score * 10000.0) as u32;
        
        // Auto-restrict if bot probability is too high
        if bot_probability > 0.8 {
            self.restriction_level = std::cmp::max(self.restriction_level, 2);
        } else if bot_probability > 0.6 {
            self.restriction_level = std::cmp::max(self.restriction_level, 1);
        }
        
        Ok(())
    }
    
    /// Get user's effective reputation score
    pub fn get_reputation_score(&self) -> u32 {
        let mut score = 5000; // Base 0.5 reputation
        
        // KYC bonus
        if self.kyc_verified {
            score += 2000; // +0.2
        }
        
        // Level bonus
        score += std::cmp::min(self.level * 10, 2000) as u32; // Up to +0.2
        
        // Network quality bonus
        score += (self.network_quality_score / 10) as u32; // Up to +1.0
        
        // Subtract for suspicious activities
        score = score.saturating_sub(self.suspicious_activities * 100);
        
        // Subtract for high bot probability
        score = score.saturating_sub(self.bot_probability / 2);
        
        std::cmp::min(score, 10000) // Cap at 1.0
    }
}
