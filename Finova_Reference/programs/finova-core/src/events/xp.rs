// programs/finova-core/src/events/xp.rs

use anchor_lang::prelude::*;

/// Event emitted when user gains XP
#[event]
pub struct XpGained {
    /// User's public key
    pub user: Pubkey,
    /// Activity type that generated XP
    pub activity_type: ActivityType,
    /// Platform where activity occurred
    pub platform: Platform,
    /// Base XP earned before multipliers
    pub base_xp: u64,
    /// Final XP after all multipliers
    pub final_xp: u64,
    /// XP multiplier applied
    pub multiplier: u64, // Stored as basis points (10000 = 1.0x)
    /// Quality score of the content/activity
    pub quality_score: u64, // Stored as basis points
    /// User's level before XP gain
    pub previous_level: u32,
    /// User's level after XP gain
    pub new_level: u32,
    /// Total XP after this gain
    pub total_xp: u64,
    /// Streak bonus applied
    pub streak_bonus: u64, // Stored as basis points
    /// Timestamp of the activity
    pub timestamp: i64,
    /// Content hash for verification
    pub content_hash: [u8; 32],
    /// External platform post ID
    pub external_post_id: String,
}

/// Event emitted when user levels up
#[event]
pub struct LevelUp {
    /// User's public key
    pub user: Pubkey,
    /// Previous level
    pub from_level: u32,
    /// New level achieved
    pub to_level: u32,
    /// Total XP at level up
    pub total_xp: u64,
    /// Mining multiplier unlocked at new level
    pub mining_multiplier: u64, // Stored as basis points
    /// Badge tier achieved
    pub badge_tier: BadgeTier,
    /// Bonus rewards for leveling up
    pub bonus_fin_reward: u64,
    /// Special privileges unlocked
    pub privileges_unlocked: Vec<Privilege>,
    /// Timestamp of level up
    pub timestamp: i64,
    /// Achievement NFT minted (if applicable)
    pub achievement_nft: Option<Pubkey>,
}

/// Event emitted when XP streak is updated
#[event]
pub struct StreakUpdated {
    /// User's public key
    pub user: Pubkey,
    /// Previous streak count
    pub previous_streak: u32,
    /// New streak count
    pub new_streak: u32,
    /// Streak type (daily, weekly, etc.)
    pub streak_type: StreakType,
    /// Bonus multiplier for current streak
    pub streak_multiplier: u64, // Stored as basis points
    /// Next milestone streak count
    pub next_milestone: u32,
    /// Reward for reaching milestone (if any)
    pub milestone_reward: Option<u64>,
    /// Last activity timestamp
    pub last_activity: i64,
    /// Current timestamp
    pub timestamp: i64,
}

/// Event emitted when streak is broken
#[event]
pub struct StreakBroken {
    /// User's public key
    pub user: Pubkey,
    /// Streak count that was lost
    pub lost_streak: u32,
    /// Streak type that was broken
    pub streak_type: StreakType,
    /// Last activity timestamp
    pub last_activity: i64,
    /// Grace period expiry (if applicable)
    pub grace_period_expiry: Option<i64>,
    /// Streak saver card used (if any)
    pub streak_saver_used: Option<Pubkey>,
    /// Current timestamp
    pub timestamp: i64,
}

/// Event emitted when viral content bonus is awarded
#[event]
pub struct ViralContentBonus {
    /// User's public key
    pub user: Pubkey,
    /// Content that went viral
    pub content_hash: [u8; 32],
    /// Platform where content went viral
    pub platform: Platform,
    /// External post ID
    pub external_post_id: String,
    /// View/engagement count that triggered viral status
    pub engagement_count: u64,
    /// Viral threshold that was exceeded
    pub viral_threshold: u64,
    /// Base XP for the content
    pub base_xp: u64,
    /// Viral multiplier applied
    pub viral_multiplier: u64, // Stored as basis points
    /// Total bonus XP awarded
    pub bonus_xp: u64,
    /// Bonus FIN tokens awarded
    pub bonus_fin: u64,
    /// Special NFT minted (if applicable)
    pub viral_nft: Option<Pubkey>,
    /// Timestamp when viral status was achieved
    pub timestamp: i64,
}

/// Event emitted when XP multiplier card is used
#[event]
pub struct XpMultiplierCardUsed {
    /// User's public key
    pub user: Pubkey,
    /// Card NFT that was consumed
    pub card_nft: Pubkey,
    /// Card type used
    pub card_type: XpCardType,
    /// Multiplier effect
    pub multiplier: u64, // Stored as basis points
    /// Duration of effect (in seconds)
    pub duration: i64,
    /// Effect expiry timestamp
    pub expiry: i64,
    /// Activities the multiplier applies to
    pub applicable_activities: Vec<ActivityType>,
    /// Timestamp when card was used
    pub timestamp: i64,
}

/// Event emitted when XP penalty is applied
#[event]
pub struct XpPenaltyApplied {
    /// User's public key
    pub user: Pubkey,
    /// Reason for penalty
    pub penalty_reason: PenaltyReason,
    /// XP amount penalized
    pub penalty_amount: u64,
    /// Penalty multiplier (reduction factor)
    pub penalty_multiplier: u64, // Stored as basis points (e.g., 5000 = 0.5x)
    /// Duration of penalty (in seconds)
    pub penalty_duration: i64,
    /// Penalty expiry timestamp
    pub penalty_expiry: i64,
    /// Activity that triggered penalty
    pub triggering_activity: Option<ActivityType>,
    /// Content hash (if applicable)
    pub content_hash: Option<[u8; 32]>,
    /// Admin who applied penalty (if manual)
    pub admin: Option<Pubkey>,
    /// Timestamp when penalty was applied
    pub timestamp: i64,
    /// Appeal deadline
    pub appeal_deadline: i64,
}

/// Event emitted when XP is restored after penalty reversal
#[event]
pub struct XpRestored {
    /// User's public key
    pub user: Pubkey,
    /// Original penalty event
    pub penalty_reference: [u8; 32],
    /// XP amount restored
    pub restored_amount: u64,
    /// Reason for restoration
    pub restoration_reason: RestorationReason,
    /// Admin who authorized restoration
    pub admin: Pubkey,
    /// Additional compensation XP
    pub compensation_xp: Option<u64>,
    /// Timestamp of restoration
    pub timestamp: i64,
}

/// Event emitted when content quality is reassessed
#[event]
pub struct ContentQualityReassessed {
    /// User's public key
    pub user: Pubkey,
    /// Content being reassessed
    pub content_hash: [u8; 32],
    /// Platform of the content
    pub platform: Platform,
    /// Original quality score
    pub original_quality_score: u64,
    /// New quality score after reassessment
    pub new_quality_score: u64,
    /// Original XP awarded
    pub original_xp: u64,
    /// New XP amount (can be higher or lower)
    pub new_xp: u64,
    /// XP adjustment (positive or negative)
    pub xp_adjustment: i64,
    /// Reason for reassessment
    pub reassessment_reason: ReassessmentReason,
    /// AI model version used
    pub ai_model_version: String,
    /// Manual reviewer (if applicable)
    pub manual_reviewer: Option<Pubkey>,
    /// Timestamp of reassessment
    pub timestamp: i64,
}

/// Event emitted when XP boost period starts
#[event]
pub struct XpBoostActivated {
    /// User's public key (None for global boost)
    pub user: Option<Pubkey>,
    /// Guild affected (if guild-specific boost)
    pub guild: Option<Pubkey>,
    /// Type of boost
    pub boost_type: BoostType,
    /// Boost multiplier
    pub boost_multiplier: u64, // Stored as basis points
    /// Boost duration (in seconds)
    pub duration: i64,
    /// Boost expiry timestamp
    pub expiry: i64,
    /// Activities affected by boost
    pub affected_activities: Vec<ActivityType>,
    /// Platforms affected by boost
    pub affected_platforms: Vec<Platform>,
    /// Trigger event (quest completion, event, etc.)
    pub trigger: BoostTrigger,
    /// Timestamp when boost was activated
    pub timestamp: i64,
}

/// Event emitted when XP boost period ends
#[event]
pub struct XpBoostExpired {
    /// User's public key (None for global boost)
    pub user: Option<Pubkey>,
    /// Guild affected (if guild-specific boost)
    pub guild: Option<Pubkey>,
    /// Type of boost that expired
    pub boost_type: BoostType,
    /// Original boost multiplier
    pub boost_multiplier: u64,
    /// Total XP gained during boost period
    pub total_xp_gained: u64,
    /// Bonus XP from the boost
    pub bonus_xp_from_boost: u64,
    /// Activities performed during boost
    pub activities_count: u32,
    /// Timestamp when boost expired
    pub timestamp: i64,
}

/// Event emitted when daily quest is completed
#[event]
pub struct DailyQuestCompleted {
    /// User's public key
    pub user: Pubkey,
    /// Quest ID that was completed
    pub quest_id: String,
    /// Quest type
    pub quest_type: QuestType,
    /// XP reward for completion
    pub xp_reward: u64,
    /// FIN token reward
    pub fin_reward: u64,
    /// Bonus multiplier if part of streak
    pub streak_multiplier: u64,
    /// Activities completed for quest
    pub completed_activities: Vec<ActivityType>,
    /// Quest completion timestamp
    pub completion_time: i64,
    /// Total quests completed today
    pub daily_quests_completed: u32,
    /// Special reward for completing all daily quests
    pub perfect_day_bonus: Option<u64>,
}

/// Event emitted when user participates in XP competition
#[event]
pub struct XpCompetitionEntry {
    /// User's public key
    pub user: Pubkey,
    /// Competition ID
    pub competition_id: String,
    /// Competition type
    pub competition_type: CompetitionType,
    /// User's current rank in competition
    pub current_rank: u32,
    /// XP earned in this competition period
    pub competition_xp: u64,
    /// Total participants in competition
    pub total_participants: u32,
    /// Competition start time
    pub competition_start: i64,
    /// Competition end time
    pub competition_end: i64,
    /// Prize pool (in FIN tokens)
    pub prize_pool: u64,
    /// Current timestamp
    pub timestamp: i64,
}

/// Types of activities that can generate XP
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum ActivityType {
    /// Original text post
    OriginalPost,
    /// Photo/image post
    PhotoPost,
    /// Video content
    VideoPost,
    /// Story/status update
    StoryPost,
    /// Meaningful comment
    Comment,
    /// Like/reaction
    Like,
    /// Share/repost
    Share,
    /// Follow/subscribe
    Follow,
    /// Daily login
    DailyLogin,
    /// Quest completion
    QuestCompletion,
    /// Milestone achievement
    Milestone,
    /// Viral content creation
    ViralContent,
    /// Guild participation
    GuildActivity,
    /// Tournament participation
    Tournament,
    /// Content curation
    ContentCuration,
    /// Community moderation
    Moderation,
    /// Educational content
    Education,
    /// User onboarding help
    Mentoring,
}

/// Social media platforms
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum Platform {
    Instagram,
    TikTok,
    YouTube,
    Facebook,
    TwitterX,
    LinkedIn,
    Discord,
    Telegram,
    WhatsApp,
    Snapchat,
    /// Native Finova app
    FinovaApp,
}

/// Badge tiers based on XP levels
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum BadgeTier {
    Bronze,
    Silver,
    Gold,
    Platinum,
    Diamond,
    Mythic,
}

/// Special privileges unlocked by leveling up
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum Privilege {
    /// Access to special cards
    SpecialCardsAccess,
    /// Guild leadership abilities
    GuildLeadership,
    /// Creator monetization features
    CreatorMonetization,
    /// Exclusive events access
    ExclusiveEvents,
    /// DAO governance participation
    DaoGovernance,
    /// Priority customer support
    PrioritySupport,
    /// Custom referral codes
    CustomReferralCodes,
    /// Analytics dashboard
    AnalyticsDashboard,
    /// Beta features access
    BetaAccess,
    /// Ambassador status
    AmbassadorStatus,
}

/// Types of streaks tracked
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum StreakType {
    /// Daily login streak
    DailyLogin,
    /// Daily posting streak
    DailyPosting,
    /// Daily quest completion
    DailyQuests,
    /// Weekly activity streak
    WeeklyActivity,
    /// Monthly milestone streak
    MonthlyMilestone,
}

/// Types of XP multiplier cards
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum XpCardType {
    /// Double XP for 24 hours
    XpDouble,
    /// Triple XP for 12 hours
    XpTriple,
    /// 5x XP for 4 hours
    XpFrenzy,
    /// Maintain streak even if inactive
    StreakSaver,
    /// Instant level boost
    LevelRush,
    /// Viral content XP magnet
    XpMagnet,
}

/// Reasons for XP penalties
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum PenaltyReason {
    /// Low quality content
    LowQuality,
    /// Suspected bot activity
    BotActivity,
    /// Spam content
    Spam,
    /// Inappropriate content
    Inappropriate,
    /// Terms of service violation
    ToSViolation,
    /// Coordinated inauthentic behavior
    CoordinatedBehavior,
    /// Copyright violation
    Copyright,
    /// False information
    Misinformation,
    /// Community guidelines violation
    CommunityViolation,
}

/// Reasons for XP restoration
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum RestorationReason {
    /// Appeal approved
    AppealApproved,
    /// System error correction
    SystemError,
    /// False positive detection
    FalsePositive,
    /// Policy change retroactive
    PolicyChange,
    /// Manual admin override
    AdminOverride,
}

/// Reasons for content quality reassessment
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum ReassessmentReason {
    /// User appeal
    UserAppeal,
    /// AI model update
    ModelUpdate,
    /// Manual review request
    ManualReview,
    /// Community flagging
    CommunityFlag,
    /// Quality audit
    QualityAudit,
    /// Viral content review
    ViralReview,
}

/// Types of XP boosts
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum BoostType {
    /// Personal boost for individual user
    Personal,
    /// Guild-wide boost
    Guild,
    /// Platform-wide boost event
    Global,
    /// Newbie boost for new users
    Newbie,
    /// Creator boost for content creators
    Creator,
    /// Weekend special boost
    Weekend,
    /// Holiday event boost
    Holiday,
    /// Competition boost
    Competition,
}

/// Events that can trigger XP boosts
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum BoostTrigger {
    /// Quest completion
    QuestCompletion,
    /// Special event
    SpecialEvent,
    /// Achievement unlock
    Achievement,
    /// Guild milestone
    GuildMilestone,
    /// Platform celebration
    PlatformEvent,
    /// User milestone
    UserMilestone,
    /// Card activation
    CardActivation,
    /// Admin activation
    AdminActivation,
}

/// Types of daily quests
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum QuestType {
    /// Post content on specific platform
    PostContent,
    /// Engage with others' content
    EngageContent,
    /// Refer new users
    ReferUsers,
    /// Reach XP target
    XpTarget,
    /// Social media variety
    PlatformDiversity,
    /// Quality content creation
    QualityCreation,
    /// Community interaction
    CommunityInteraction,
    /// Educational content
    EducationalContent,
}

/// Types of XP competitions
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum CompetitionType {
    /// Daily XP leaderboard
    DailyLeaderboard,
    /// Weekly XP challenge
    WeeklyChallenge,
    /// Monthly marathon
    MonthlyMarathon,
    /// Creator contest
    CreatorContest,
    /// Platform-specific contest
    PlatformContest,
    /// Guild vs Guild battle
    GuildBattle,
    /// Newbie competition
    NewbieCompetition,
    /// Quality content contest
    QualityContest,
}

impl ActivityType {
    /// Get base XP for activity type
    pub fn base_xp(&self) -> u64 {
        match self {
            ActivityType::OriginalPost => 50,
            ActivityType::PhotoPost => 75,
            ActivityType::VideoPost => 150,
            ActivityType::StoryPost => 25,
            ActivityType::Comment => 25,
            ActivityType::Like => 5,
            ActivityType::Share => 15,
            ActivityType::Follow => 20,
            ActivityType::DailyLogin => 10,
            ActivityType::QuestCompletion => 100,
            ActivityType::Milestone => 500,
            ActivityType::ViralContent => 1000,
            ActivityType::GuildActivity => 30,
            ActivityType::Tournament => 200,
            ActivityType::ContentCuration => 40,
            ActivityType::Moderation => 60,
            ActivityType::Education => 80,
            ActivityType::Mentoring => 100,
        }
    }

    /// Get daily limit for activity type
    pub fn daily_limit(&self) -> Option<u32> {
        match self {
            ActivityType::PhotoPost => Some(20),
            ActivityType::VideoPost => Some(10),
            ActivityType::StoryPost => Some(50),
            ActivityType::Comment => Some(100),
            ActivityType::Like => Some(200),
            ActivityType::Share => Some(50),
            ActivityType::Follow => Some(25),
            ActivityType::DailyLogin => Some(1),
            ActivityType::QuestCompletion => Some(3),
            _ => None, // No limit
        }
    }
}

impl Platform {
    /// Get platform-specific multiplier
    pub fn multiplier(&self) -> u64 {
        match self {
            Platform::TikTok => 13000,      // 1.3x
            Platform::Instagram => 12000,   // 1.2x
            Platform::YouTube => 14000,     // 1.4x
            Platform::TwitterX => 12000,    // 1.2x
            Platform::Facebook => 11000,    // 1.1x
            Platform::LinkedIn => 11500,    // 1.15x
            Platform::Discord => 10500,     // 1.05x
            Platform::Telegram => 10500,    // 1.05x
            Platform::WhatsApp => 10000,    // 1.0x
            Platform::Snapchat => 11000,    // 1.1x
            Platform::FinovaApp => 15000,   // 1.5x (highest for native app)
        }
    }
}
