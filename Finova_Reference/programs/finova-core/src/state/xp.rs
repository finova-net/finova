// programs/finova-core/src/state/xp.rs

use anchor_lang::prelude::*;
use crate::constants::*;

#[account]
#[derive(Default)]
pub struct XPAccount {
    /// User's public key
    pub user: Pubkey,
    /// Current XP level (1-101+)
    pub level: u16,
    /// Current XP points
    pub current_xp: u64,
    /// Total XP earned all time
    pub total_xp_earned: u64,
    /// XP earned today
    pub daily_xp: u64,
    /// Daily XP reset timestamp
    pub daily_reset_time: i64,
    /// Current streak of daily activities
    pub daily_streak: u32,
    /// Best streak achieved
    pub best_streak: u32,
    /// Streak start date
    pub streak_start_date: i64,
    /// Last activity timestamp
    pub last_activity: i64,
    /// XP multipliers currently active
    pub active_multipliers: Vec<XPMultiplier>,
    /// Platform-specific XP tracking
    pub platform_xp: PlatformXP,
    /// XP achievements unlocked
    pub achievements: Vec<u32>,
    /// XP statistics
    pub stats: XPStats,
    /// Badge tier (Bronze, Silver, Gold, etc.)
    pub badge_tier: BadgeTier,
    /// Special XP bonuses
    pub special_bonuses: SpecialBonuses,
    /// Reserved space for future upgrades
    pub reserved: [u8; 128],
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default)]
pub struct XPMultiplier {
    /// Multiplier type
    pub multiplier_type: MultiplierType,
    /// Multiplier value (in basis points, 100 = 1.0x)
    pub value: u16,
    /// Start time of multiplier
    pub start_time: i64,
    /// Duration in seconds
    pub duration: i64,
    /// Source of multiplier (NFT, achievement, event, etc.)
    pub source: MultiplierSource,
    /// Multiplier is stackable
    pub stackable: bool,
    /// Maximum stack count
    pub max_stacks: u8,
    /// Current stack count
    pub current_stacks: u8,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub enum MultiplierType {
    Global,
    Platform,
    Activity,
    Quality,
    Streak,
    Event,
    Achievement,
    NFT,
    Guild,
    Staking,
}

impl Default for MultiplierType {
    fn default() -> Self {
        MultiplierType::Global
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub enum MultiplierSource {
    BaseSystem,
    NFTCard,
    Achievement,
    Event,
    Guild,
    Staking,
    Referral,
    Premium,
    Special,
}

impl Default for MultiplierSource {
    fn default() -> Self {
        MultiplierSource::BaseSystem
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default)]
pub struct PlatformXP {
    /// Instagram XP and stats
    pub instagram: PlatformStats,
    /// TikTok XP and stats
    pub tiktok: PlatformStats,
    /// YouTube XP and stats
    pub youtube: PlatformStats,
    /// Facebook XP and stats
    pub facebook: PlatformStats,
    /// X (Twitter) XP and stats
    pub twitter_x: PlatformStats,
    /// LinkedIn XP and stats
    pub linkedin: PlatformStats,
    /// Discord XP and stats
    pub discord: PlatformStats,
    /// Telegram XP and stats
    pub telegram: PlatformStats,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default)]
pub struct PlatformStats {
    /// Total XP earned on this platform
    pub total_xp: u64,
    /// Posts made on this platform
    pub posts_count: u32,
    /// Comments made on this platform
    pub comments_count: u32,
    /// Likes/reactions given
    pub likes_given: u32,
    /// Shares/reposts made
    pub shares_count: u32,
    /// Followers gained through Finova
    pub followers_gained: u32,
    /// Viral content count (>1K views/engagement)
    pub viral_content_count: u32,
    /// Average engagement rate
    pub avg_engagement_rate: u32,
    /// Quality score on this platform
    pub quality_score: u8,
    /// Platform-specific level
    pub platform_level: u16,
    /// Last activity on platform
    pub last_activity: i64,
    /// Platform multiplier bonus
    pub platform_bonus: u16,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default)]
pub struct XPStats {
    /// XP earned per day (last 30 days)
    pub daily_xp_history: [u32; 30],
    /// Average XP per day
    pub avg_daily_xp: u32,
    /// Best single day XP
    pub best_day_xp: u32,
    /// Total activities completed
    pub total_activities: u64,
    /// Content creation XP percentage
    pub content_creation_pct: u8,
    /// Engagement XP percentage
    pub engagement_pct: u8,
    /// Social XP percentage
    pub social_pct: u8,
    /// Special activity XP percentage
    pub special_activity_pct: u8,
    /// Time spent in app (minutes)
    pub total_time_minutes: u64,
    /// Sessions completed
    pub total_sessions: u32,
    /// Average session duration
    pub avg_session_duration: u32,
    /// Level up count
    pub level_ups: u32,
    /// XP lost due to inactivity
    pub xp_lost_inactivity: u64,
    /// XP gained from referrals
    pub xp_from_referrals: u64,
    /// XP gained from guild activities
    pub xp_from_guild: u64,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub enum BadgeTier {
    Bronze,
    Silver,
    Gold,
    Platinum,
    Diamond,
    Mythic,
    Legendary,
}

impl Default for BadgeTier {
    fn default() -> Self {
        BadgeTier::Bronze
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default)]
pub struct SpecialBonuses {
    /// First week bonus active
    pub first_week_bonus: bool,
    /// Weekend bonus multiplier
    pub weekend_bonus: u16,
    /// Holiday bonus multiplier
    pub holiday_bonus: u16,
    /// Birthday bonus (user's birthday)
    pub birthday_bonus: u16,
    /// Anniversary bonus (account creation)
    pub anniversary_bonus: u16,
    /// Comeback bonus (after inactivity)
    pub comeback_bonus: u16,
    /// Perfect week bonus (7 days streak)
    pub perfect_week_bonus: u16,
    /// Community milestone bonus
    pub milestone_bonus: u16,
}

#[account]
#[derive(Default)]
pub struct XPActivity {
    /// Activity unique identifier
    pub activity_id: u64,
    /// User who performed the activity
    pub user: Pubkey,
    /// Activity type
    pub activity_type: ActivityType,
    /// Platform where activity occurred
    pub platform: Platform,
    /// Timestamp of activity
    pub timestamp: i64,
    /// Base XP for this activity
    pub base_xp: u32,
    /// Actual XP awarded (after multipliers)
    pub awarded_xp: u32,
    /// Quality score of the activity
    pub quality_score: u8,
    /// Platform multiplier applied
    pub platform_multiplier: u16,
    /// Streak multiplier applied
    pub streak_multiplier: u16,
    /// Other multipliers applied
    pub other_multipliers: Vec<XPMultiplier>,
    /// Activity metadata
    pub metadata: ActivityMetadata,
    /// Activity verification status
    pub verification_status: VerificationStatus,
    /// Activity engagement metrics
    pub engagement_metrics: EngagementMetrics,
    /// Reserved space for future upgrades
    pub reserved: [u8; 64],
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub enum ActivityType {
    // Content Creation
    TextPost,
    ImagePost,
    VideoPost,
    Story,
    Reel,
    Live,
    
    // Engagement
    Comment,
    Like,
    Share,
    Follow,
    Subscribe,
    
    // Special Activities
    DailyLogin,
    CompleteQuest,
    AchieveMilestone,
    ReferralSuccess,
    GuildActivity,
    
    // Social Integration
    ConnectPlatform,
    ShareFinova,
    InviteFriend,
    ReviewApp,
    
    // Premium Activities
    PurchaseNFT,
    UseSpecialCard,
    StakeTokens,
    ParticipateEvent,
}

impl Default for ActivityType {
    fn default() -> Self {
        ActivityType::TextPost
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub enum Platform {
    Instagram,
    TikTok,
    YouTube,
    Facebook,
    TwitterX,
    LinkedIn,
    Discord,
    Telegram,
    FinovaApp,
    Other,
}

impl Default for Platform {
    fn default() -> Self {
        Platform::FinovaApp
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default)]
pub struct ActivityMetadata {
    /// Content hash (for verification)
    pub content_hash: [u8; 32],
    /// External platform post ID
    pub external_post_id: [u8; 64],
    /// Content type (text, image, video)
    pub content_type: ContentType,
    /// Content length/duration
    pub content_length: u32,
    /// Hashtags used
    pub hashtags: Vec<String>,
    /// Mentions made
    pub mentions: Vec<String>,
    /// Location tagged
    pub location: Option<String>,
    /// Language of content
    pub language: [u8; 8],
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub enum ContentType {
    Text,
    Image,
    Video,
    Audio,
    Mixed,
    Link,
}

impl Default for ContentType {
    fn default() -> Self {
        ContentType::Text
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub enum VerificationStatus {
    Pending,
    Verified,
    Failed,
    Disputed,
    ManualReview,
}

impl Default for VerificationStatus {
    fn default() ->

{
        VerificationStatus::Pending
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default)]
pub struct EngagementMetrics {
    /// Views/impressions
    pub views: u32,
    /// Likes/reactions
    pub likes: u32,
    /// Comments received
    pub comments: u32,
    /// Shares/reposts
    pub shares: u32,
    /// Click-through rate (if applicable)
    pub ctr: u16,
    /// Engagement rate
    pub engagement_rate: u16,
    /// Time spent viewing (seconds)
    pub view_duration: u32,
    /// Unique viewers
    pub unique_viewers: u32,
}

#[account]
#[derive(Default)]
pub struct XPLevelConfig {
    /// Level number
    pub level: u16,
    /// XP required to reach this level
    pub xp_required: u64,
    /// XP required for next level
    pub xp_to_next: u64,
    /// Rewards unlocked at this level
    pub rewards: LevelRewards,
    /// Features unlocked at this level
    pub features_unlocked: Vec<Feature>,
    /// Mining multiplier at this level
    pub mining_multiplier: u16,
    /// Special privileges
    pub privileges: Vec<Privilege>,
    /// Level tier
    pub tier: BadgeTier,
    /// Reserved space for future upgrades
    pub reserved: [u8; 64],
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default)]
pub struct LevelRewards {
    /// $FIN tokens reward
    pub fin_tokens: u64,
    /// Special NFT reward
    pub nft_reward: Option<Pubkey>,
    /// Special card reward
    pub special_card: Option<u32>,
    /// Title/badge reward
    pub title_reward: Option<String>,
    /// Achievement unlock
    pub achievement_unlock: Vec<u32>,
    /// Bonus XP multiplier
    pub bonus_multiplier: u16,
    /// Access to exclusive features
    pub exclusive_features: Vec<Feature>,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub enum Feature {
    BasicMining,
    EnhancedMining,
    NFTMarketplace,
    SpecialCards,
    GuildCreation,
    GuildLeadership,
    TournamentEntry,
    PremiumSupport,
    CustomProfile,
    AdvancedAnalytics,
    APIAccess,
    BetaFeatures,
}

impl Default for Feature {
    fn default() -> Self {
        Feature::BasicMining
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub enum Privilege {
    PrioritySupport,
    EarlyAccess,
    ExclusiveEvents,
    CustomBadge,
    ProfileVerification,
    EnhancedLimits,
    SpecialChannels,
    ModeratorRights,
    CommunityVoting,
    InfluencerProgram,
}

impl Default for Privilege {
    fn default() -> Self {
        Privilege::PrioritySupport
    }
}

#[account]
#[derive(Default)]
pub struct XPLeaderboard {
    /// Leaderboard type (daily, weekly, monthly, all-time)
    pub leaderboard_type: LeaderboardType,
    /// Leaderboard entries
    pub entries: Vec<LeaderboardEntry>,
    /// Last update timestamp
    pub last_updated: i64,
    /// Current season/period
    pub current_period: u32,
    /// Total participants
    pub total_participants: u32,
    /// Leaderboard rewards pool
    pub rewards_pool: u64,
    /// Top performers rewards
    pub top_rewards: Vec<LeaderboardReward>,
    /// Leaderboard statistics
    pub stats: LeaderboardStats,
    /// Reserved space for future upgrades
    pub reserved: [u8; 128],
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub enum LeaderboardType {
    Daily,
    Weekly,
    Monthly,
    Seasonal,
    AllTime,
    Guild,
    Platform,
    Activity,
}

impl Default for LeaderboardType {
    fn default() -> Self {
        LeaderboardType::Daily
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default)]
pub struct LeaderboardEntry {
    /// User's public key
    pub user: Pubkey,
    /// User's score for this leaderboard
    pub score: u64,
    /// User's current rank
    pub rank: u32,
    /// User's previous rank
    pub previous_rank: u32,
    /// Rank change
    pub rank_change: i32,
    /// User's level
    pub level: u16,
    /// User's badge tier
    pub badge_tier: BadgeTier,
    /// Additional stats for this leaderboard
    pub additional_stats: Vec<u32>,
    /// Last update timestamp
    pub last_updated: i64,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default)]
pub struct LeaderboardReward {
    /// Rank range (e.g., 1-3 for top 3)
    pub rank_start: u32,
    pub rank_end: u32,
    /// Reward in $FIN tokens
    pub fin_reward: u64,
    /// Special NFT reward
    pub nft_reward: Option<Pubkey>,
    /// Special card reward
    pub card_reward: Option<u32>,
    /// Achievement reward
    pub achievement_reward: Option<u32>,
    /// Title reward
    pub title_reward: Option<String>,
    /// Multiplier bonus reward
    pub multiplier_reward: Option<u16>,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default)]
pub struct LeaderboardStats {
    /// Average score
    pub average_score: u64,
    /// Highest score
    pub highest_score: u64,
    /// Score distribution
    pub score_distribution: [u32; 10], // 10 percentile buckets
    /// Most active user
    pub most_active_user: Option<Pubkey>,
    /// Biggest climber (rank improvement)
    pub biggest_climber: Option<Pubkey>,
    /// Most consistent performer
    pub most_consistent: Option<Pubkey>,
    /// Competition level (how close scores are)
    pub competition_level: u8,
}

impl XPAccount {
    pub const LEN: usize = 8 + // discriminator
        32 + // user
        2 + // level
        8 + // current_xp
        8 + // total_xp_earned
        8 + // daily_xp
        8 + // daily_reset_time
        4 + // daily_streak
        4 + // best_streak
        8 + // streak_start_date
        8 + // last_activity
        4 + (std::mem::size_of::<XPMultiplier>() * MAX_ACTIVE_MULTIPLIERS) + // active_multipliers
        std::mem::size_of::<PlatformXP>() +
        4 + (4 * MAX_XP_ACHIEVEMENTS) + // achievements
        std::mem::size_of::<XPStats>() +
        std::mem::size_of::<BadgeTier>() +
        std::mem::size_of::<SpecialBonuses>() +
        128; // reserved

    /// Calculate current level based on XP
    pub fn calculate_level(&self) -> u16 {
        // Exponential XP curve: XP_required = level^2 * 100
        let mut level = 1u16;
        let mut total_xp_needed = 0u64;
        
        loop {
            let xp_for_this_level = (level as u64).pow(2) * 100;
            if total_xp_needed + xp_for_this_level > self.current_xp {
                break;
            }
            total_xp_needed += xp_for_this_level;
            level += 1;
            
            if level > MAX_XP_LEVEL {
                break;
            }
        }
        
        level.saturating_sub(1).max(1)
    }

    /// Calculate XP needed for next level
    pub fn xp_needed_for_next_level(&self) -> u64 {
        let next_level = self.level + 1;
        let xp_for_next_level = (next_level as u64).pow(2) * 100;
        let current_level_total_xp = (1..=self.level).map(|l| (l as u64).pow(2) * 100).sum::<u64>();
        
        current_level_total_xp + xp_for_next_level - self.current_xp
    }

    /// Calculate total active multiplier
    pub fn calculate_total_multiplier(&self, current_time: i64, activity_type: &ActivityType, platform: &Platform) -> u16 {
        let mut total_multiplier = 100u16; // Base 1.0x
        
        // Apply active multipliers
        for multiplier in &self.active_multipliers {
            if current_time >= multiplier.start_time && 
               current_time <= (multiplier.start_time + multiplier.duration) {
                match multiplier.multiplier_type {
                    MultiplierType::Global => {
                        total_multiplier = total_multiplier.saturating_add(multiplier.value - 100);
                    },
                    MultiplierType::Platform => {
                        if self.is_platform_match(platform, &multiplier) {
                            total_multiplier = total_multiplier.saturating_add(multiplier.value - 100);
                        }
                    },
                    MultiplierType::Activity => {
                        if self.is_activity_match(activity_type, &multiplier) {
                            total_multiplier = total_multiplier.saturating_add(multiplier.value - 100);
                        }
                    },
                    MultiplierType::Streak => {
                        let streak_bonus = self.calculate_streak_multiplier();
                        total_multiplier = total_multiplier.saturating_add(streak_bonus);
                    },
                    _ => {
                        total_multiplier = total_multiplier.saturating_add(multiplier.value - 100);
                    }
                }
            }
        }
        
        // Apply level-based multiplier
        let level_multiplier = (self.level / 10) * 5; // 0.5% per 10 levels
        total_multiplier = total_multiplier.saturating_add(level_multiplier);
        
        // Apply badge tier multiplier
        let badge_multiplier = match self.badge_tier {
            BadgeTier::Bronze => 0,
            BadgeTier::Silver => 10,
            BadgeTier::Gold => 25,
            BadgeTier::Platinum => 50,
            BadgeTier::Diamond => 75,
            BadgeTier::Mythic => 100,
            BadgeTier::Legendary => 150,
        };
        total_multiplier = total_multiplier.saturating_add(badge_multiplier);
        
        total_multiplier.min(500) // Cap at 5.0x
    }

    /// Calculate streak multiplier
    pub fn calculate_streak_multiplier(&self) -> u16 {
        match self.daily_streak {
            0..=6 => 0,
            7..=13 => 10,  // 0.1x bonus for 1 week
            14..=29 => 25, // 0.25x bonus for 2 weeks
            30..=59 => 50, // 0.5x bonus for 1 month
            60..=89 => 75, // 0.75x bonus for 2 months
            90..=179 => 100, // 1.0x bonus for 3 months
            180..=364 => 125, // 1.25x bonus for 6 months
            _ => 150, // 1.5x bonus for 1+ year
        }
    }

    /// Update daily XP and streak
    pub fn update_daily_activity(&mut self, xp_gained: u32, current_time: i64) {
        let current_day = current_time / (24 * 60 * 60);
        let reset_day = self.daily_reset_time / (24 * 60 * 60);
        
        // Check if it's a new day
        if current_day > reset_day {
            // Reset daily XP
            self.daily_xp = 0;
            self.daily_reset_time = current_day * (24 * 60 * 60);
            
            // Check streak
            if current_day == reset_day + 1 {
                // Consecutive day - maintain streak
                self.daily_streak += 1;
            } else {
                // Missed day(s) - reset streak
                if self.daily_streak > self.best_streak {
                    self.best_streak = self.daily_streak;
                }
                self.daily_streak = 1;
                self.streak_start_date = current_time;
            }
        }
        
        // Add XP to daily total
        self.daily_xp = self.daily_xp.saturating_add(xp_gained as u64);
        self.last_activity = current_time;
        
        // Update daily XP history
        self.update_xp_history(xp_gained);
    }

    /// Update XP history tracking
    fn update_xp_history(&mut self, xp_gained: u32) {
        // Shift array left and add new value
        for i in 0..29 {
            self.stats.daily_xp_history[i] = self.stats.daily_xp_history[i + 1];
        }
        self.stats.daily_xp_history[29] = xp_gained;
        
        // Recalculate average
        let total: u32 = self.stats.daily_xp_history.iter().sum();
        self.stats.avg_daily_xp = total / 30;
        
        // Update best day
        if xp_gained > self.stats.best_day_xp {
            self.stats.best_day_xp = xp_gained;
        }
    }

    /// Check if platform matches multiplier
    fn is_platform_match(&self, platform: &Platform, multiplier: &XPMultiplier) -> bool {
        // This would be implemented based on multiplier metadata
        // For now, return true for all platform multipliers
        true
    }

    /// Check if activity matches multiplier
    fn is_activity_match(&self, activity_type: &ActivityType, multiplier: &XPMultiplier) -> bool {
        // This would be implemented based on multiplier metadata
        // For now, return true for all activity multipliers
        true
    }

    /// Add XP multiplier
    pub fn add_multiplier(&mut self, multiplier: XPMultiplier) -> Result<(), &'static str> {
        if self.active_multipliers.len() >= MAX_ACTIVE_MULTIPLIERS {
            return Err("Maximum multipliers reached");
        }
        
        // Check if multiplier is stackable
        if !multiplier.stackable {
            // Remove existing non-stackable multipliers of same type
            self.active_multipliers.retain(|m| {
                m.multiplier_type != multiplier.multiplier_type || m.stackable
            });
        }
        
        self.active_multipliers.push(multiplier);
        Ok(())
    }

    /// Remove expired multipliers
    pub fn remove_expired_multipliers(&mut self, current_time: i64) {
        self.active_multipliers.retain(|multiplier| {
            current_time < (multiplier.start_time + multiplier.duration)
        });
    }

    /// Update badge tier based on level and achievements
    pub fn update_badge_tier(&mut self) {
        self.badge_tier = match self.level {
            1..=10 => BadgeTier::Bronze,
            11..=25 => BadgeTier::Silver,
            26..=50 => BadgeTier::Gold,
            51..=75 => BadgeTier::Platinum,
            76..=100 => BadgeTier::Diamond,
            101..=150 => BadgeTier::Mythic,
            _ => BadgeTier::Legendary,
        };
    }

    /// Calculate quality score based on activities
    pub fn calculate_quality_score(&self) -> u8 {
        let mut score = 50u32; // Base score
        
        // Factor in platform diversity
        let active_platforms = self.count_active_platforms();
        score += (active_platforms * 5).min(25);
        
        // Factor in streak
        if self.daily_streak >= 7 {
            score += 10;
        }
        if self.daily_streak >= 30 {
            score += 15;
        }
        
        // Factor in engagement vs creation ratio
        let creation_ratio = self.stats.content_creation_pct;
        let engagement_ratio = self.stats.engagement_pct;
        
        if creation_ratio >= 30 && engagement_ratio >= 30 {
            score += 10; // Balanced activity
        }
        
        score.min(100) as u8
    }

    /// Count active platforms
    fn count_active_platforms(&self) -> u32 {
        let mut count = 0;
        
        if self.platform_xp.instagram.total_xp > 0 { count += 1; }
        if self.platform_xp.tiktok.total_xp > 0 { count += 1; }
        if self.platform_xp.youtube.total_xp > 0 { count += 1; }
        if self.platform_xp.facebook.total_xp > 0 { count += 1; }
        if self.platform_xp.twitter_x.total_xp > 0 { count += 1; }
        if self.platform_xp.linkedin.total_xp > 0 { count += 1; }
        if self.platform_xp.discord.total_xp > 0 { count += 1; }
        if self.platform_xp.telegram.total_xp > 0 { count += 1; }
        
        count
    }
}

impl XPActivity {
    pub const LEN: usize = 8 + // discriminator
        8 + // activity_id
        32 + // user
        std::mem::size_of::<ActivityType>() +
        std::mem::size_of::<Platform>() +
        8 + // timestamp
        4 + // base_xp
        4 + // awarded_xp
        1 + // quality_score
        2 + // platform_multiplier
        2 + // streak_multiplier
        4 + (std::mem::size_of::<XPMultiplier>() * MAX_ACTIVITY_MULTIPLIERS) + // other_multipliers
        std::mem::size_of::<ActivityMetadata>() +
        std::mem::size_of::<VerificationStatus>() +
        std::mem::size_of::<EngagementMetrics>() +
        64; // reserved

    /// Verify activity authenticity
    pub fn verify_activity(&mut self, verification_data: &[u8]) -> bool {
        // Implement verification logic based on platform and activity type
        // This would integrate with social media APIs
        self.verification_status = VerificationStatus::Verified;
        true
    }

    /// Calculate final XP with all multipliers
    pub fn calculate_final_xp(&self) -> u32 {
        let mut final_xp = self.base_xp as u64;
        
        // Apply platform multiplier
        final_xp = (final_xp * self.platform_multiplier as u64) / 100;
        
        // Apply streak multiplier
        final_xp = (final_xp * self.streak_multiplier as u64) / 100;
        
        // Apply other multipliers
        for multiplier in &self.other_multipliers {
            final_xp = (final_xp * multiplier.value as u64) / 100;
        }
        
        // Apply quality score modifier
        let quality_modifier = 50 + (self.quality_score as u64 / 2); // 0.5x to 1.0x based on quality
        final_xp = (final_xp * quality_modifier) / 100;
        
        final_xp.min(u32::MAX as u64) as u32
    }

    /// Update engagement metrics
    pub fn update_engagement(&mut self, views: u32, likes: u32, comments: u32, shares: u32) {
        self.engagement_metrics.views = views;
        self.engagement_metrics.likes = likes;
        self.engagement_metrics.comments = comments;
        self.engagement_metrics.shares = shares;
        
        // Calculate engagement rate
        if views > 0 {
            let total_engagement = likes + comments + shares;
            self.engagement_metrics.engagement_rate = ((total_engagement * 10000) / views) as u16; // Basis points
        }
    }
}

// Constants for XP system
pub const MAX_XP_LEVEL: u16 = 1000;
pub const MAX_ACTIVE_MULTIPLIERS: usize = 20;
pub const MAX_ACTIVITY_MULTIPLIERS: usize = 10;
pub const MAX_XP_ACHIEVEMENTS: usize = 100;
pub const DAILY_XP_CAP: u64 = 10000;
pub const STREAK_XP_BONUS_CAP: u16 = 200; // 2.0x max
pub const QUALITY_SCORE_MIN: u8 = 10;
pub const QUALITY_SCORE_MAX: u8 = 100;
