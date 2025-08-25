// finova-net/finova/client/rust/src/accounts.rs

use anchor_lang::prelude::*;
use solana_sdk::pubkey::Pubkey;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// Re-export common types
pub use anchor_lang::solana_program::system_program;
pub use anchor_spl::token::{Token, TokenAccount, Mint};

/// Core user account structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserAccount {
    pub authority: Pubkey,
    pub mining_rate: u64,
    pub total_mined: u64,
    pub last_mining_time: i64,
    pub xp_level: u32,
    pub xp_points: u64,
    pub rp_tier: u8,
    pub rp_points: u64,
    pub referral_count: u32,
    pub active_referrals: u32,
    pub kyc_verified: bool,
    pub human_score: u16, // 0-1000 (0.000-1.000)
    pub total_holdings: u64,
    pub streak_days: u16,
    pub quality_score: u16,
    pub guild_id: Option<Pubkey>,
    pub created_at: i64,
    pub updated_at: i64,
    pub status: u8, // 0: Active, 1: Suspended, 2: Banned
}

impl UserAccount {
    pub const LEN: usize = 8 + // discriminator
        32 + // authority
        8 + // mining_rate
        8 + // total_mined
        8 + // last_mining_time
        4 + // xp_level
        8 + // xp_points
        1 + // rp_tier
        8 + // rp_points
        4 + // referral_count
        4 + // active_referrals
        1 + // kyc_verified
        2 + // human_score
        8 + // total_holdings
        2 + // streak_days
        2 + // quality_score
        33 + // guild_id (Option<Pubkey>)
        8 + // created_at
        8 + // updated_at
        1; // status
}

/// Mining session account
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MiningAccount {
    pub user: Pubkey,
    pub session_id: u64,
    pub base_rate: u64,
    pub pioneer_bonus: u16, // basis points
    pub referral_bonus: u16, // basis points
    pub security_bonus: u16, // basis points
    pub regression_factor: u16, // basis points
    pub session_start: i64,
    pub session_end: i64,
    pub tokens_earned: u64,
    pub is_active: bool,
    pub phase: u8, // 1-4 (Finizen, Growth, Maturity, Stability)
}

impl MiningAccount {
    pub const LEN: usize = 8 + // discriminator
        32 + // user
        8 + // session_id
        8 + // base_rate
        2 + // pioneer_bonus
        2 + // referral_bonus
        2 + // security_bonus
        2 + // regression_factor
        8 + // session_start
        8 + // session_end
        8 + // tokens_earned
        1 + // is_active
        1; // phase
}

/// Staking account for liquid staking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StakingAccount {
    pub user: Pubkey,
    pub staked_amount: u64,
    pub sfin_amount: u64, // sFIN tokens received
    pub stake_timestamp: i64,
    pub last_reward_claim: i64,
    pub accumulated_rewards: u64,
    pub tier: u8, // 0-4 (100-499, 500-999, 1K-4.9K, 5K-9.9K, 10K+)
    pub loyalty_bonus: u16, // basis points
    pub activity_multiplier: u16, // basis points
    pub auto_compound: bool,
}

impl StakingAccount {
    pub const LEN: usize = 8 + // discriminator
        32 + // user
        8 + // staked_amount
        8 + // sfin_amount
        8 + // stake_timestamp
        8 + // last_reward_claim
        8 + // accumulated_rewards
        1 + // tier
        2 + // loyalty_bonus
        2 + // activity_multiplier
        1; // auto_compound
}

/// Referral network account
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReferralAccount {
    pub user: Pubkey,
    pub referrer: Option<Pubkey>,
    pub referral_code: String, // Max 32 chars
    pub direct_referrals: Vec<Pubkey>, // Max 1000
    pub network_size: u32,
    pub network_quality: u16, // 0-1000
    pub total_earned_rp: u64,
    pub tier: u8, // 0-4 (Explorer, Connector, Influencer, Leader, Ambassador)
    pub bonus_multiplier: u16, // basis points
    pub created_at: i64,
}

impl ReferralAccount {
    pub const LEN: usize = 8 + // discriminator
        32 + // user
        33 + // referrer (Option<Pubkey>)
        36 + // referral_code (String with length)
        4 + (32 * 1000) + // direct_referrals (Vec<Pubkey>)
        4 + // network_size
        2 + // network_quality
        8 + // total_earned_rp
        1 + // tier
        2 + // bonus_multiplier
        8; // created_at
}

/// NFT metadata account
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NftAccount {
    pub mint: Pubkey,
    pub owner: Pubkey,
    pub card_type: u8, // 0: Mining, 1: XP, 2: Referral, 3: Badge
    pub rarity: u8, // 0: Common, 1: Uncommon, 2: Rare, 3: Epic, 4: Legendary
    pub effect_value: u16, // Effect strength in basis points
    pub duration: u32, // Duration in seconds, 0 for permanent
    pub uses_remaining: u16, // For consumable cards
    pub is_active: bool,
    pub activation_time: i64,
    pub metadata_uri: String, // Max 200 chars
    pub created_at: i64,
}

impl NftAccount {
    pub const LEN: usize = 8 + // discriminator
        32 + // mint
        32 + // owner
        1 + // card_type
        1 + // rarity
        2 + // effect_value
        4 + // duration
        2 + // uses_remaining
        1 + // is_active
        8 + // activation_time
        204 + // metadata_uri (String with length)
        8; // created_at
}

/// Guild account
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuildAccount {
    pub id: Pubkey,
    pub name: String, // Max 50 chars
    pub master: Pubkey,
    pub officers: Vec<Pubkey>, // Max 5
    pub members: Vec<Pubkey>, // Max 50
    pub member_count: u32,
    pub total_xp: u64,
    pub guild_level: u32,
    pub treasury: u64,
    pub competition_wins: u32,
    pub created_at: i64,
    pub is_active: bool,
}

impl GuildAccount {
    pub const LEN: usize = 8 + // discriminator
        32 + // id
        54 + // name (String with length)
        32 + // master
        4 + (32 * 5) + // officers (Vec<Pubkey>)
        4 + (32 * 50) + // members (Vec<Pubkey>)
        4 + // member_count
        8 + // total_xp
        4 + // guild_level
        8 + // treasury
        4 + // competition_wins
        8 + // created_at
        1; // is_active
}

/// XP activity tracking account
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XpAccount {
    pub user: Pubkey,
    pub current_level: u32,
    pub total_xp: u64,
    pub daily_xp: u64,
    pub last_activity: i64,
    pub streak_days: u16,
    pub activities_today: u16,
    pub platform_bonuses: HashMap<String, u16>, // Platform -> bonus in bp
    pub achievements: Vec<u32>, // Achievement IDs
    pub multiplier_expires: i64,
    pub current_multiplier: u16, // basis points
}

impl XpAccount {
    pub const LEN: usize = 8 + // discriminator
        32 + // user
        4 + // current_level
        8 + // total_xp
        8 + // daily_xp
        8 + // last_activity
        2 + // streak_days
        2 + // activities_today
        500 + // platform_bonuses (estimated)
        4 + (4 * 100) + // achievements (Vec<u32>)
        8 + // multiplier_expires
        2; // current_multiplier
}

/// Reward pool account
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RewardPoolAccount {
    pub pool_id: u8,
    pub total_rewards: u64,
    pub distributed_rewards: u64,
    pub weekly_allocation: u64,
    pub last_distribution: i64,
    pub pool_type: u8, // 0: Mining, 1: Staking, 2: XP, 3: RP, 4: Events
    pub is_active: bool,
    pub distribution_rate: u64, // Per second
}

impl RewardPoolAccount {
    pub const LEN: usize = 8 + // discriminator
        1 + // pool_id
        8 + // total_rewards
        8 + // distributed_rewards
        8 + // weekly_allocation
        8 + // last_distribution
        1 + // pool_type
        1 + // is_active
        8; // distribution_rate
}

/// Network statistics account
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkAccount {
    pub total_users: u64,
    pub active_miners: u64,
    pub total_mined: u64,
    pub current_phase: u8,
    pub phase_start_time: i64,
    pub base_mining_rate: u64,
    pub pioneer_multiplier: u16, // basis points
    pub last_updated: i64,
    pub daily_new_users: u32,
    pub kyc_verified_users: u64,
}

impl NetworkAccount {
    pub const LEN: usize = 8 + // discriminator
        8 + // total_users
        8 + // active_miners
        8 + // total_mined
        1 + // current_phase
        8 + // phase_start_time
        8 + // base_mining_rate
        2 + // pioneer_multiplier
        8 + // last_updated
        4 + // daily_new_users
        8; // kyc_verified_users
}

/// Oracle price feed account
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OracleAccount {
    pub feed_id: String, // Max 32 chars
    pub price: u64,
    pub confidence: u64,
    pub last_updated: i64,
    pub update_authority: Pubkey,
    pub is_active: bool,
    pub decimals: u8,
}

impl OracleAccount {
    pub const LEN: usize = 8 + // discriminator
        36 + // feed_id (String with length)
        8 + // price
        8 + // confidence
        8 + // last_updated
        32 + // update_authority
        1 + // is_active
        1; // decimals
}

/// Anti-bot detection account
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AntiBotAccount {
    pub user: Pubkey,
    pub human_probability: u16, // 0-1000
    pub behavioral_score: u16,
    pub device_fingerprint: String, // Max 100 chars
    pub last_verification: i64,
    pub verification_count: u32,
    pub suspicious_activities: u16,
    pub penalty_factor: u16, // basis points
    pub is_verified: bool,
}

impl AntiBotAccount {
    pub const LEN: usize = 8 + // discriminator
        32 + // user
        2 + // human_probability
        2 + // behavioral_score
        104 + // device_fingerprint (String with length)
        8 + // last_verification
        4 + // verification_count
        2 + // suspicious_activities
        2 + // penalty_factor
        1; // is_verified
}

/// Account helper functions
pub mod helpers {
    use super::*;
    use anchor_lang::prelude::*;

    /// Find user account PDA
    pub fn find_user_account(authority: &Pubkey, program_id: &Pubkey) -> (Pubkey, u8) {
        Pubkey::find_program_address(&[b"user", authority.as_ref()], program_id)
    }

    /// Find mining account PDA
    pub fn find_mining_account(user: &Pubkey, session_id: u64, program_id: &Pubkey) -> (Pubkey, u8) {
        Pubkey::find_program_address(
            &[b"mining", user.as_ref(), &session_id.to_le_bytes()],
            program_id,
        )
    }

    /// Find staking account PDA
    pub fn find_staking_account(user: &Pubkey, program_id: &Pubkey) -> (Pubkey, u8) {
        Pubkey::find_program_address(&[b"staking", user.as_ref()], program_id)
    }

    /// Find referral account PDA
    pub fn find_referral_account(user: &Pubkey, program_id: &Pubkey) -> (Pubkey, u8) {
        Pubkey::find_program_address(&[b"referral", user.as_ref()], program_id)
    }

    /// Find NFT account PDA
    pub fn find_nft_account(mint: &Pubkey, program_id: &Pubkey) -> (Pubkey, u8) {
        Pubkey::find_program_address(&[b"nft", mint.as_ref()], program_id)
    }

    /// Find guild account PDA
    pub fn find_guild_account(guild_id: &Pubkey, program_id: &Pubkey) -> (Pubkey, u8) {
        Pubkey::find_program_address(&[b"guild", guild_id.as_ref()], program_id)
    }

    /// Find XP account PDA
    pub fn find_xp_account(user: &Pubkey, program_id: &Pubkey) -> (Pubkey, u8) {
        Pubkey::find_program_address(&[b"xp", user.as_ref()], program_id)
    }

    /// Find network account PDA
    pub fn find_network_account(program_id: &Pubkey) -> (Pubkey, u8) {
        Pubkey::find_program_address(&[b"network"], program_id)
    }

    /// Find oracle account PDA
    pub fn find_oracle_account(feed_id: &str, program_id: &Pubkey) -> (Pubkey, u8) {
        Pubkey::find_program_address(&[b"oracle", feed_id.as_bytes()], program_id)
    }

    /// Find anti-bot account PDA
    pub fn find_antibot_account(user: &Pubkey, program_id: &Pubkey) -> (Pubkey, u8) {
        Pubkey::find_program_address(&[b"antibot", user.as_ref()], program_id)
    }
}

/// Account validation utilities
pub mod validation {
    use super::*;

    /// Validate user account constraints
    pub fn validate_user_account(account: &UserAccount) -> Result<(), &'static str> {
        if account.human_score > 1000 {
            return Err("Invalid human score");
        }
        if account.rp_tier > 4 {
            return Err("Invalid RP tier");
        }
        if account.status > 2 {
            return Err("Invalid status");
        }
        Ok(())
    }

    /// Validate mining account constraints
    pub fn validate_mining_account(account: &MiningAccount) -> Result<(), &'static str> {
        if account.phase == 0 || account.phase > 4 {
            return Err("Invalid mining phase");
        }
        if account.session_start >= account.session_end && account.is_active {
            return Err("Invalid session timing");
        }
        Ok(())
    }

    /// Validate staking tier
    pub fn validate_staking_tier(staked_amount: u64, tier: u8) -> Result<(), &'static str> {
        let expected_tier = match staked_amount {
            100..=499 => 0,
            500..=999 => 1,
            1000..=4999 => 2,
            5000..=9999 => 3,
            10000.. => 4,
            _ => return Err("Below minimum staking amount"),
        };
        
        if tier != expected_tier {
            return Err("Incorrect staking tier");
        }
        Ok(())
    }

    /// Validate RP tier based on points
    pub fn validate_rp_tier(rp_points: u64, tier: u8) -> Result<(), &'static str> {
        let expected_tier = match rp_points {
            0..=999 => 0,        // Explorer
            1000..=4999 => 1,    // Connector
            5000..=14999 => 2,   // Influencer
            15000..=49999 => 3,  // Leader
            50000.. => 4,        // Ambassador
        };
        
        if tier != expected_tier {
            return Err("Incorrect RP tier");
        }
        Ok(())
    }
}

/// Account event types for logging and monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AccountEvent {
    UserCreated { user: Pubkey, timestamp: i64 },
    MiningStarted { user: Pubkey, session_id: u64, rate: u64 },
    StakingDeposit { user: Pubkey, amount: u64, new_tier: u8 },
    ReferralSuccess { referrer: Pubkey, referee: Pubkey },
    NftMinted { mint: Pubkey, owner: Pubkey, card_type: u8 },
    XpGained { user: Pubkey, amount: u64, new_level: u32 },
    GuildCreated { guild_id: Pubkey, master: Pubkey },
    SuspiciousActivity { user: Pubkey, activity_type: String },
}

/// Constants for account calculations
pub mod constants {
    pub const SECONDS_PER_HOUR: i64 = 3600;
    pub const SECONDS_PER_DAY: i64 = 86400;
    pub const BASIS_POINTS_SCALE: u16 = 10000;
    pub const MAX_REFERRALS: usize = 1000;
    pub const MAX_GUILD_MEMBERS: usize = 50;
    pub const MAX_GUILD_OFFICERS: usize = 5;
    pub const HUMAN_SCORE_SCALE: u16 = 1000;
    pub const DEFAULT_MINING_RATE: u64 = 50_000; // 0.05 FIN in microFIN
    
    // Phase thresholds
    pub const PHASE_2_THRESHOLD: u64 = 100_000;   // 100K users
    pub const PHASE_3_THRESHOLD: u64 = 1_000_000; // 1M users
    pub const PHASE_4_THRESHOLD: u64 = 10_000_000; // 10M users
    
    // XP level requirements (exponential)
    pub const XP_LEVEL_BASE: u64 = 100;
    pub const XP_LEVEL_MULTIPLIER: f64 = 1.5;
}
