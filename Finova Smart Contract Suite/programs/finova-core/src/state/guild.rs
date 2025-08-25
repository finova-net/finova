// programs/finova-core/src/state/guild.rs

use anchor_lang::prelude::*;
use crate::constants::*;

#[account]
#[derive(Default)]
pub struct Guild {
    /// Guild unique identifier
    pub guild_id: u64,
    /// Guild name (max 32 characters)
    pub name: [u8; 32],
    /// Guild description (max 128 characters)
    pub description: [u8; 128],
    /// Guild master's public key
    pub guild_master: Pubkey,
    /// Guild officers (max 5)
    pub officers: Vec<Pubkey>,
    /// Guild members list
    pub members: Vec<Pubkey>,
    /// Maximum number of members allowed
    pub max_members: u32,
    /// Current member count
    pub member_count: u32,
    /// Guild creation timestamp
    pub created_at: i64,
    /// Guild level (1-100)
    pub level: u8,
    /// Guild experience points
    pub experience: u64,
    /// Guild treasury balance in $FIN
    pub treasury_balance: u64,
    /// Guild total mining rewards earned
    pub total_mining_rewards: u64,
    /// Guild competitions won count
    pub competitions_won: u32,
    /// Guild current season rank
    pub current_rank: u32,
    /// Guild is active flag
    pub is_active: bool,
    /// Guild requirements for joining
    pub join_requirements: GuildRequirements,
    /// Guild statistics
    pub stats: GuildStats,
    /// Guild settings and configuration
    pub settings: GuildSettings,
    /// Reserved space for future upgrades
    pub reserved: [u8; 128],
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default)]
pub struct GuildRequirements {
    /// Minimum XP level required to join
    pub min_xp_level: u8,
    /// Minimum RP tier required to join
    pub min_rp_tier: u8,
    /// Minimum $FIN holdings required
    pub min_fin_holdings: u64,
    /// Minimum daily activity score
    pub min_daily_activity: u32,
    /// KYC verification required
    pub kyc_required: bool,
    /// Invitation only flag
    pub invitation_only: bool,
    /// Geographic restrictions (country codes)
    pub geo_restrictions: Vec<u16>,
    /// Premium membership required
    pub premium_required: bool,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default)]
pub struct GuildStats {
    /// Total XP earned by all members
    pub total_xp_earned: u64,
    /// Total RP earned by all members
    pub total_rp_earned: u64,
    /// Total mining rewards earned
    pub total_mining_earned: u64,
    /// Total NFTs owned by guild members
    pub total_nfts_owned: u32,
    /// Average member level
    pub average_member_level: u16,
    /// Guild activity score (0-100)
    pub activity_score: u8,
    /// Daily active members count
    pub daily_active_members: u32,
    /// Weekly active members count
    pub weekly_active_members: u32,
    /// Monthly active members count
    pub monthly_active_members: u32,
    /// Guild retention rate (percentage)
    pub retention_rate: u8,
    /// Average session duration in minutes
    pub avg_session_duration: u32,
    /// Total social media posts by members
    pub total_social_posts: u64,
    /// Total referrals made by guild
    pub total_referrals: u32,
    /// Guild quality score (0-100)
    pub quality_score: u8,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default)]
pub struct GuildSettings {
    /// Auto-accept new members based on requirements
    pub auto_accept_members: bool,
    /// Allow members to invite others
    pub members_can_invite: bool,
    /// Share mining rewards with treasury
    pub share_mining_rewards: bool,
    /// Treasury contribution percentage (0-100)
    pub treasury_contribution_rate: u8,
    /// Enable guild challenges
    pub enable_challenges: bool,
    /// Enable guild competitions
    pub enable_competitions: bool,
    /// Allow external partnerships
    pub allow_partnerships: bool,
    /// Guild privacy level (0=public, 1=private, 2=secret)
    pub privacy_level: u8,
    /// Enable guild messaging
    pub enable_messaging: bool,
    /// Enable guild events
    pub enable_events: bool,
    /// Enable guild marketplace
    pub enable_marketplace: bool,
    /// Enable cross-guild interactions
    pub enable_cross_guild: bool,
}

#[account]
#[derive(Default)]
pub struct GuildMembership {
    /// Member's public key
    pub member: Pubkey,
    /// Guild public key
    pub guild: Pubkey,
    /// Membership role
    pub role: GuildRole,
    /// Join timestamp
    pub joined_at: i64,
    /// Member's contribution to guild
    pub contribution_score: u64,
    /// Member's guild-specific XP
    pub guild_xp: u64,
    /// Member's guild-specific level
    pub guild_level: u8,
    /// Total mining rewards contributed to guild
    pub mining_contributed: u64,
    /// Total referrals made for guild
    pub referrals_made: u32,
    /// Member's guild-specific achievements
    pub achievements: Vec<u32>,
    /// Member's participation in guild events
    pub events_participated: u32,
    /// Member's last activity timestamp
    pub last_active: i64,
    /// Member's status in guild
    pub status: MembershipStatus,
    /// Member's guild-specific settings
    pub settings: MemberSettings,
    /// Reserved space for future upgrades
    pub reserved: [u8; 64],
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default)]
pub enum GuildRole {
    #[default]
    Member,
    Officer,
    CoLeader,
    Leader,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default)]
pub enum MembershipStatus {
    #[default]
    Active,
    Inactive,
    Suspended,
    Banned,
    PendingApproval,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default)]
pub struct MemberSettings {
    /// Receive guild notifications
    pub notifications_enabled: bool,
    /// Share mining rewards with guild
    pub share_mining_rewards: bool,
    /// Participate in guild challenges
    pub participate_challenges: bool,
    /// Participate in guild competitions
    pub participate_competitions: bool,
    /// Allow guild messaging
    pub allow_messaging: bool,
    /// Share profile with guild members
    pub share_profile: bool,
    /// Auto-join guild events
    pub auto_join_events: bool,
    /// Allow guild partnership invitations
    pub allow_partnerships: bool,
}

#[account]
#[derive(Default)]
pub struct GuildChallenge {
    /// Challenge unique identifier
    pub challenge_id: u64,
    /// Guild hosting the challenge
    pub guild: Pubkey,
    /// Challenge name
    pub name: [u8; 64],
    /// Challenge description
    pub description: [u8; 256],
    /// Challenge type
    pub challenge_type: ChallengeType,
    /// Challenge start time
    pub start_time: i64,
    /// Challenge end time
    pub end_time: i64,
    /// Challenge target/goal
    pub target: u64,
    /// Current progress towards target
    pub current_progress: u64,
    /// Participating members
    pub participants: Vec<Pubkey>,
    /// Challenge rewards pool
    pub rewards_pool: u64,
    /// Challenge status
    pub status: ChallengeStatus,
    /// Winner(s) of the challenge
    pub winners: Vec<Pubkey>,
    /// Challenge requirements
    pub requirements: ChallengeRequirements,
    /// Challenge leaderboard
    pub leaderboard: Vec<LeaderboardEntry>,
    /// Reserved space for future upgrades
    pub reserved: [u8; 128],
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub enum ChallengeType {
    MiningRewards,
    XPGain,
    ReferralCount,
    SocialPosts,
    NFTCollection,
    TradingVolume,
    StakingAmount,
    CommunityEngagement,
}

impl Default for ChallengeType {
    fn default() -> Self {
        ChallengeType::MiningRewards
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub enum ChallengeStatus {
    Pending,
    Active,
    Completed,
    Cancelled,
    Expired,
}

impl Default for ChallengeStatus {
    fn default() -> Self {
        ChallengeStatus::Pending
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default)]
pub struct ChallengeRequirements {
    /// Minimum level to participate
    pub min_level: u8,
    /// Minimum RP tier to participate
    pub min_rp_tier: u8,
    /// Entry fee in $FIN
    pub entry_fee: u64,
    /// Maximum participants allowed
    pub max_participants: u32,
    /// KYC verification required
    pub kyc_required: bool,
    /// Guild membership required
    pub guild_membership_required: bool,
    /// Minimum guild level required
    pub min_guild_level: u8,
    /// Premium membership required
    pub premium_required: bool,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default)]
pub struct LeaderboardEntry {
    /// Participant's public key
    pub participant: Pubkey,
    /// Participant's score in challenge
    pub score: u64,
    /// Participant's rank
    pub rank: u32,
    /// Timestamp of last score update
    pub last_updated: i64,
    /// Additional metadata
    pub metadata: [u8; 32],
}

#[account]
#[derive(Default)]
pub struct GuildTournament {
    /// Tournament unique identifier
    pub tournament_id: u64,
    /// Tournament name
    pub name: [u8; 64],
    /// Tournament description
    pub description: [u8; 256],
    /// Tournament type
    pub tournament_type: TournamentType,
    /// Organizer guild (can be null for system tournaments)
    pub organizer: Option<Pubkey>,
    /// Participating guilds
    pub participating_guilds: Vec<Pubkey>,
    /// Tournament start time
    pub start_time: i64,
    /// Tournament end time
    pub end_time: i64,
    /// Registration deadline
    pub registration_deadline: i64,
    /// Tournament rules and conditions
    pub rules: TournamentRules,
    /// Prize pool distribution
    pub prize_pool: TournamentPrizePool,
    /// Tournament status
    pub status: TournamentStatus,
    /// Current tournament round
    pub current_round: u8,
    /// Total number of rounds
    pub total_rounds: u8,
    /// Tournament brackets/matchups
    pub brackets: Vec<TournamentMatch>,
    /// Tournament leaderboard
    pub leaderboard: Vec<GuildLeaderboardEntry>,
    /// Tournament statistics
    pub stats: TournamentStats,
    /// Reserved space for future upgrades
    pub reserved: [u8; 128],
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub enum TournamentType {
    SingleElimination,
    DoubleElimination,
    RoundRobin,
    Swiss,
    KingOfTheHill,
    TeamBattle,
    SeasonLeague,
}

impl Default for TournamentType {
    fn default() -> Self {
        TournamentType::SingleElimination
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub enum TournamentStatus {
    Registration,
    Active,
    Completed,
    Cancelled,
    Suspended,
}

impl Default for TournamentStatus {
    fn default() -> Self {
        TournamentStatus::Registration
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default)]
pub struct TournamentRules {
    /// Minimum guild level to participate
    pub min_guild_level: u8,
    /// Minimum member count required
    pub min_members: u32,
    /// Maximum member count allowed
    pub max_members: u32,
    /// Entry fee per guild
    pub entry_fee: u64,
    /// Scoring method
    pub scoring_method: ScoringMethod,
    /// Match duration in hours
    pub match_duration: u32,
    /// Allow substitutions
    pub allow_substitutions: bool,
    /// Maximum substitutions per match
    pub max_substitutions: u8,
    /// Tie-breaker rules
    pub tie_breaker: TieBreakerMethod,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub enum ScoringMethod {
    TotalPoints,
    AveragePerMember,
    BestPerformers,
    CombinedMetrics,
}

impl Default for ScoringMethod {
    fn default() -> Self {
        ScoringMethod::TotalPoints
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub enum TieBreakerMethod {
    HighestIndividualScore,
    MostActiveMembers,
    BestQualityScore,
    RandomSelection,
}

impl Default for TieBreakerMethod {
    fn default() -> Self {
        TieBreakerMethod::HighestIndividualScore
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default)]
pub struct TournamentPrizePool {
    /// Total prize pool in $FIN
    pub total_pool: u64,
    /// First place prize percentage
    pub first_place_pct: u8,
    /// Second place prize percentage
    pub second_place_pct: u8,
    /// Third place prize percentage
    pub third_place_pct: u8,
    /// Participation prize percentage
    pub participation_pct: u8,
    /// Special achievement prizes
    pub special_prizes: Vec<SpecialPrize>,
    /// NFT rewards
    pub nft_rewards: Vec<Pubkey>,
    /// Bonus multipliers for different achievements
    pub bonus_multipliers: BonusMultipliers,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default)]
pub struct SpecialPrize {
    /// Prize name/description
    pub name: [u8; 32],
    /// Prize value in $FIN
    pub value: u64,
    /// Prize criteria
    pub criteria: [u8; 64],
    /// Prize type
    pub prize_type: PrizeType,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub enum PrizeType {
    Token,
    NFT,
    SpecialCard,
    Achievement,
    Title,
}

impl Default for PrizeType {
    fn default() -> Self {
        PrizeType::Token
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default)]
pub struct BonusMultipliers {
    /// MVP player bonus
    pub mvp_bonus: u16,
    /// Perfect participation bonus
    pub perfect_participation_bonus: u16,
    /// Comeback victory bonus
    pub comeback_bonus: u16,
    /// Dominant victory bonus
    pub dominant_victory_bonus: u16,
    /// First tournament bonus
    pub first_tournament_bonus: u16,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default)]
pub struct TournamentMatch {
    /// Match unique identifier
    pub match_id: u64,
    /// Guild A
    pub guild_a: Pubkey,
    /// Guild B
    pub guild_b: Pubkey,
    /// Match start time
    pub start_time: i64,
    /// Match end time
    pub end_time: i64,
    /// Guild A score
    pub score_a: u64,
    /// Guild B score
    pub score_b: u64,
    /// Match winner
    pub winner: Option<Pubkey>,
    /// Match status
    pub status: MatchStatus,
    /// Match round
    pub round: u8,
    /// Match type (group, quarter, semi, final, etc.)
    pub match_type: MatchType,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub enum MatchStatus {
    Scheduled,
    InProgress,
    Completed,
    Forfeited,
    Disputed,
}

impl Default for MatchStatus {
    fn default() -> Self {
        MatchStatus::Scheduled
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub enum MatchType {
    Group,
    Playoff,
    QuarterFinal,
    SemiFinal,
    Final,
    ThirdPlace,
}

impl Default for MatchType {
    fn default() -> Self {
        MatchType::Group
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default)]
pub struct GuildLeaderboardEntry {
    /// Guild public key
    pub guild: Pubkey,
    /// Guild's tournament score
    pub score: u64,
    /// Guild's current rank
    pub rank: u32,
    /// Matches played
    pub matches_played: u32,
    /// Matches won
    pub matches_won: u32,
    /// Matches lost
    pub matches_lost: u32,
    /// Points differential
    pub points_differential: i64,
    /// Additional tournament-specific metrics
    pub tournament_metrics: TournamentMetrics,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default)]
pub struct TournamentMetrics {
    /// Total XP earned during tournament
    pub total_xp: u64,
    /// Total mining rewards during tournament
    pub total_mining: u64,
    /// Total referrals made during tournament
    pub total_referrals: u32,
    /// Average member participation rate
    pub participation_rate: u8,
    /// Quality score during tournament
    pub quality_score: u8,
    /// Special achievements unlocked
    pub achievements: Vec<u32>,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default)]
pub struct TournamentStats {
    /// Total participants
    pub total_participants: u32,
    /// Total matches played
    pub total_matches: u32,
    /// Total prize pool distributed
    pub total_prizes_distributed: u64,
    /// Average match score
    pub average_match_score: u64,
    /// Highest individual performance
    pub highest_individual_score: u64,
    /// Most active guild
    pub most_active_guild: Option<Pubkey>,
    /// Tournament duration in hours
    pub duration_hours: u32,
    /// Viewer/spectator count
    pub spectator_count: u32,
}

impl Guild {
    pub const LEN: usize = 8 + // discriminator
        8 + // guild_id
        32 + // name
        128 + // description
        32 + // guild_master
        4 + (32 * MAX_GUILD_OFFICERS) + // officers
        4 + (32 * MAX_GUILD_MEMBERS) + // members
        4 + // max_members
        4 + // member_count
        8 + // created_at
        1 + // level
        8 + // experience
        8 + // treasury_balance
        8 + // total_mining_rewards
        4 + // competitions_won
        4 + // current_rank
        1 + // is_active
        std::mem::size_of::<GuildRequirements>() +
        std::mem::size_of::<GuildStats>() +
        std::mem::size_of::<GuildSettings>() +
        128; // reserved

    /// Calculate guild mining bonus multiplier
    pub fn calculate_mining_multiplier(&self) -> u64 {
        let base_multiplier = 100; // 1.0x in basis points
        let level_bonus = (self.level as u64) * 2; // 0.02x per level
        let activity_bonus = (self.stats.activity_score as u64) / 2; // up to 0.5x
        let member_bonus = (self.member_count as u64).min(50) * 1; // up to 0.5x
        
        base_multiplier + level_bonus + activity_bonus + member_bonus
    }

    /// Calculate guild XP bonus multiplier
    pub fn calculate_xp_multiplier(&self) -> u64 {
        let base_multiplier = 100; // 1.0x in basis points
        let level_bonus = (self.level as u64) * 3; // 0.03x per level
        let quality_bonus = (self.stats.quality_score as u64) / 4; // up to 0.25x
        let retention_bonus = (self.stats.retention_rate as u64) / 4; // up to 0.25x
        
        base_multiplier + level_bonus + quality_bonus + retention_bonus
    }

    /// Check if user meets guild requirements
    pub fn meets_requirements(&self, user_level: u8, user_rp_tier: u8, user_holdings: u64, user_kyc: bool) -> bool {
        if user_level < self.join_requirements.min_xp_level {
            return false;
        }
        if user_rp_tier < self.join_requirements.min_rp_tier {
            return false;
        }
        if user_holdings < self.join_requirements.min_fin_holdings {
            return false;
        }
        if self.join_requirements.kyc_required && !user_kyc {
            return false;
        }
        true
    }

    /// Update guild statistics
    pub fn update_stats(&mut self, member_activity: &[u64], current_time: i64) {
        // Calculate average member level
        if !member_activity.is_empty() {
            self.stats.average_member_level = (member_activity.iter().sum::<u64>() / member_activity.len() as u64) as u16;
        }

        // Update activity score based on member engagement
        let active_members = member_activity.iter().filter(|&&activity| activity > 0).count();
        self.stats.activity_score = ((active_members * 100) / self.member_count.max(1) as usize) as u8;
        self.stats.daily_active_members = active_members as u32;

        // Update quality score based on various factors
        self.stats.quality_score = self.calculate_quality_score();
    }

    /// Calculate guild quality score
    fn calculate_quality_score(&self) -> u8 {
        let mut score = 0u32;
        
        // Activity component (40%)
        score += (self.stats.activity_score as u32 * 40) / 100;
        
        // Retention component (30%)
        score += (self.stats.retention_rate as u32 * 30) / 100;
        
        // Level component (20%)
        let level_score = (self.level as u32 * 100) / MAX_GUILD_LEVEL as u32;
        score += (level_score * 20) / 100;
        
        // Member count component (10%)
        let member_score = (self.member_count * 100) / self.max_members;
        score += (member_score * 10) / 100;
        
        score.min(100) as u8
    }

    /// Check if guild can level up
    pub fn can_level_up(&self) -> bool {
        if self.level >= MAX_GUILD_LEVEL {
            return false;
        }
        
        let required_xp = self.calculate_required_xp_for_next_level();
        self.experience >= required_xp
    }

    /// Calculate required XP for next level
    pub fn calculate_required_xp_for_next_level(&self) -> u64 {
        let next_level = self.level + 1;
        (next_level as u64).pow(2) * 1000 // Quadratic scaling
    }
}

impl GuildMembership {
    pub const LEN: usize = 8 + // discriminator
        32 + // member
        32 + // guild
        std::mem::size_of::<GuildRole>() +
        8 + // joined_at
        8 + // contribution_score
        8 + // guild_xp
        1 + // guild_level
        8 + // mining_contributed
        4 + // referrals_made
        4 + (4 * MAX_MEMBER_ACHIEVEMENTS) + // achievements
        4 + // events_participated
        8 + // last_active
        std::mem::size_of::<MembershipStatus>() +
        std::mem::size_of::<MemberSettings>() +
        64; // reserved

    /// Calculate member's contribution score
    pub fn calculate_contribution_score(&mut self, mining_rewards: u64, xp_gained: u64, referrals: u32) {
        self.contribution_score = (mining_rewards / 1000) + (xp_gained / 100) + (referrals as u64 * 50);
    }

    /// Check if member is active
    pub fn is_active(&self, current_time: i64) -> bool {
        matches!(self.status, MembershipStatus::Active) && 
        (current_time - self.last_active) < MEMBER_INACTIVE_THRESHOLD
    }

    /// Get member's role permissions
    pub fn get_permissions(&self) -> Vec<GuildPermission> {
        match self.role {
            GuildRole::Member => vec![
                GuildPermission::ParticipateEvents,
                GuildPermission::ViewGuildInfo,
                GuildPermission::UseGuildChat,
            ],
            GuildRole::Officer => vec![
                GuildPermission::ParticipateEvents,
                GuildPermission::ViewGuildInfo,
                GuildPermission::UseGuildChat,
                GuildPermission::InviteMembers,
                GuildPermission::KickMembers,
                GuildPermission::CreateChallenges,
            ],
            GuildRole::CoLeader => vec![
                GuildPermission::ParticipateEvents,
                GuildPermission::ViewGuildInfo,
                GuildPermission::UseGuildChat,
                GuildPermission::InviteMembers,
                GuildPermission::KickMembers,
                GuildPermission::CreateChallenges,
                GuildPermission::ManageSettings,
                GuildPermission::ManageTreasury,
            ],
            GuildRole::Leader => vec![
                GuildPermission::ParticipateEvents,
                GuildPermission::ViewGuildInfo,
                GuildPermission::UseGuildChat,
                GuildPermission::InviteMembers,
                GuildPermission::KickMembers,
                GuildPermission::CreateChallenges,
                GuildPermission::ManageSettings,
                GuildPermission::ManageTreasury,
                GuildPermission::DisbandGuild,
                GuildPermission::TransferLeadership,
            ],
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum GuildPermission {
    ParticipateEvents,
    ViewGuildInfo,
    UseGuildChat,
    InviteMembers,
    KickMembers,
    CreateChallenges,
    ManageSettings,
    ManageTreasury,
    DisbandGuild,
    TransferLeadership,
}

impl GuildChallenge {
    pub const LEN: usize = 8 + // discriminator
        8 + // challenge_id
        32 + // guild
        64 + // name
        256 + // description
        std::mem::size_of::<ChallengeType>() +
        8 + // start_time
        8 + // end_time
        8 + // target
        8 + // current_progress
        4 + (32 * MAX_CHALLENGE_PARTICIPANTS) + // participants
        8 + // rewards_pool
        std::mem::size_of::<ChallengeStatus>() +
        4 + (32 * MAX_CHALLENGE_WINNERS) + // winners
        std::mem::size_of::<ChallengeRequirements>() +
        4 + (std::mem::size_of::<LeaderboardEntry>() * MAX_LEADERBOARD_ENTRIES) + // leaderboard
        128; // reserved

    /// Check if challenge is active
    pub fn is_active(&self, current_time: i64) -> bool {
        matches!(self.status, ChallengeStatus::Active) &&
        current_time >= self.start_time &&
        current_time <= self.end_time
    }

    /// Update challenge progress
    pub fn update_progress(&mut self, participant: Pubkey, score: u64, current_time: i64) {
        // Update leaderboard
        if let Some(entry) = self.leaderboard.iter_mut().find(|e| e.participant == participant) {
            entry.score = score;
            entry.last_updated = current_time;
        } else if self.leaderboard.len() < MAX_LEADERBOARD_ENTRIES {
            self.leaderboard.push(LeaderboardEntry {
                participant,
                score,
                rank: 0, // Will be calculated when sorting
                last_updated: current_time,
                metadata: [0; 32],
            });
        }

        // Sort leaderboard by score
        self.leaderboard.sort_by(|a, b| b.score.cmp(&a.score));
        
        // Update ranks
        for (i, entry) in self.leaderboard.iter_mut().enumerate() {
            entry.rank = (i + 1) as u32;
        }

        // Update total progress
        self.current_progress = self.leaderboard.iter().map(|e| e.score).sum();
    }

    /// Check if challenge is completed
    pub fn is_completed(&self) -> bool {
        self.current_progress >= self.target || 
        matches!(self.status, ChallengeStatus::Completed)
    }

    /// Get challenge winners
    pub fn get_winners(&self, winner_count: usize) -> Vec<Pubkey> {
        self.leaderboard
            .iter()
            .take(winner_count)
            .map(|entry| entry.participant)
            .collect()
    }
}

// Constants for guild system
pub const MAX_GUILD_MEMBERS: usize = 100;
pub const MAX_GUILD_OFFICERS: usize = 5;
pub const MAX_GUILD_LEVEL: u8 = 100;
pub const MAX_CHALLENGE_PARTICIPANTS: usize = 1000;
pub const MAX_CHALLENGE_WINNERS: usize = 10;
pub const MAX_LEADERBOARD_ENTRIES: usize = 100;
pub const MAX_MEMBER_ACHIEVEMENTS: usize = 50;
pub const MEMBER_INACTIVE_THRESHOLD: i64 = 30 * 24 * 60 * 60; // 30 days in seconds
