// programs/finova-core/src/instructions/anti_bot.rs

use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount, Mint};
use std::collections::HashMap;
use crate::state::*;
use crate::errors::*;
use crate::constants::*;
use crate::utils::*;

/// Initialize Anti-Bot System
#[derive(Accounts)]
pub struct InitializeAntiBotSystem<'info> {
    #[account(
        init,
        payer = authority,
        space = AntiBotConfig::SPACE,
        seeds = [ANTI_BOT_CONFIG_SEED],
        bump
    )]
    pub anti_bot_config: Account<'info, AntiBotConfig>,
    
    #[account(mut)]
    pub authority: Signer<'info>,
    
    pub system_program: Program<'info, System>,
}

/// Update Human Probability Score
#[derive(Accounts)]
pub struct UpdateHumanProbability<'info> {
    #[account(
        mut,
        seeds = [USER_SEED, user_account.key().as_ref()],
        bump = user_account.bump
    )]
    pub user_account: Account<'info, UserAccount>,
    
    #[account(
        seeds = [ANTI_BOT_CONFIG_SEED],
        bump = anti_bot_config.bump
    )]
    pub anti_bot_config: Account<'info, AntiBotConfig>,
    
    #[account(mut)]
    pub authority: Signer<'info>,
}

/// Submit Proof of Humanity
#[derive(Accounts)]
pub struct SubmitProofOfHumanity<'info> {
    #[account(
        mut,
        seeds = [USER_SEED, user_account.key().as_ref()],
        bump = user_account.bump
    )]
    pub user_account: Account<'info, UserAccount>,
    
    #[account(
        init,
        payer = user,
        space = ProofOfHumanity::SPACE,
        seeds = [POH_SEED, user.key().as_ref()],
        bump
    )]
    pub proof_of_humanity: Account<'info, ProofOfHumanity>,
    
    #[account(
        seeds = [ANTI_BOT_CONFIG_SEED],
        bump = anti_bot_config.bump
    )]
    pub anti_bot_config: Account<'info, AntiBotConfig>,
    
    #[account(mut)]
    pub user: Signer<'info>,
    
    pub system_program: Program<'info, System>,
}

/// Report Suspicious Activity
#[derive(Accounts)]
pub struct ReportSuspiciousActivity<'info> {
    #[account(
        mut,
        seeds = [USER_SEED, reported_user.key().as_ref()],
        bump = reported_user.bump
    )]
    pub reported_user: Account<'info, UserAccount>,
    
    #[account(
        init,
        payer = reporter,
        space = SuspiciousActivityReport::SPACE,
        seeds = [SUSPICIOUS_REPORT_SEED, reported_user.key().as_ref(), &[suspicious_report_count]],
        bump
    )]
    pub suspicious_report: Account<'info, SuspiciousActivityReport>,
    
    #[account(
        seeds = [USER_SEED, reporter.key().as_ref()],
        bump = reporter_account.bump
    )]
    pub reporter_account: Account<'info, UserAccount>,
    
    #[account(
        seeds = [ANTI_BOT_CONFIG_SEED],
        bump = anti_bot_config.bump
    )]
    pub anti_bot_config: Account<'info, AntiBotConfig>,
    
    #[account(mut)]
    pub reporter: Signer<'info>,
    
    pub system_program: Program<'info, System>,
}

/// Update Bot Detection Parameters
#[derive(Accounts)]
pub struct UpdateBotDetectionParams<'info> {
    #[account(
        mut,
        seeds = [ANTI_BOT_CONFIG_SEED],
        bump = anti_bot_config.bump
    )]
    pub anti_bot_config: Account<'info, AntiBotConfig>,
    
    #[account(mut)]
    pub authority: Signer<'info>,
}

/// Validate User Behavior Pattern
#[derive(Accounts)]
pub struct ValidateUserBehavior<'info> {
    #[account(
        mut,
        seeds = [USER_SEED, user_account.key().as_ref()],
        bump = user_account.bump
    )]
    pub user_account: Account<'info, UserAccount>,
    
    #[account(
        mut,
        seeds = [BEHAVIOR_PATTERN_SEED, user_account.key().as_ref()],
        bump = behavior_pattern.bump
    )]
    pub behavior_pattern: Account<'info, BehaviorPattern>,
    
    #[account(
        seeds = [ANTI_BOT_CONFIG_SEED],
        bump = anti_bot_config.bump
    )]
    pub anti_bot_config: Account<'info, AntiBotConfig>,
    
    #[account(mut)]
    pub validator: Signer<'info>,
}

/// Initialize Anti-Bot System
pub fn initialize_anti_bot_system(
    ctx: Context<InitializeAntiBotSystem>,
    detection_threshold: u16,      // Minimum human probability required (e.g., 80 = 80%)
    suspicious_threshold: u16,     // Threshold for suspicious activity
    penalty_multiplier: u16,       // Penalty multiplier for suspicious users
    cooling_period: i64,           // Cooling period in seconds
) -> Result<()> {
    let anti_bot_config = &mut ctx.accounts.anti_bot_config;
    
    require!(detection_threshold <= 100, FinovaError::InvalidParameter);
    require!(suspicious_threshold <= 100, FinovaError::InvalidParameter);
    require!(penalty_multiplier <= 1000, FinovaError::InvalidParameter);
    require!(cooling_period > 0, FinovaError::InvalidParameter);
    
    anti_bot_config.authority = ctx.accounts.authority.key();
    anti_bot_config.detection_threshold = detection_threshold;
    anti_bot_config.suspicious_threshold = suspicious_threshold;
    anti_bot_config.penalty_multiplier = penalty_multiplier;
    anti_bot_config.cooling_period = cooling_period;
    anti_bot_config.total_reports = 0;
    anti_bot_config.verified_humans = 0;
    anti_bot_config.suspected_bots = 0;
    anti_bot_config.bump = *ctx.bumps.get("anti_bot_config").unwrap();
    anti_bot_config.created_at = Clock::get()?.unix_timestamp;
    anti_bot_config.updated_at = Clock::get()?.unix_timestamp;
    
    msg!("Anti-Bot System initialized with threshold: {}%", detection_threshold);
    
    Ok(())
}

/// Update Human Probability Score based on AI analysis
pub fn update_human_probability(
    ctx: Context<UpdateHumanProbability>,
    biometric_score: u16,          // 0-100 based on biometric analysis
    behavioral_score: u16,         // 0-100 based on behavioral patterns
    social_graph_score: u16,       // 0-100 based on social connections
    device_score: u16,             // 0-100 based on device fingerprinting
    interaction_score: u16,        // 0-100 based on interaction quality
) -> Result<()> {
    let user_account = &mut ctx.accounts.user_account;
    let anti_bot_config = &ctx.accounts.anti_bot_config;
    let current_time = Clock::get()?.unix_timestamp;
    
    // Validate scores
    require!(biometric_score <= 100, FinovaError::InvalidParameter);
    require!(behavioral_score <= 100, FinovaError::InvalidParameter);
    require!(social_graph_score <= 100, FinovaError::InvalidParameter);
    require!(device_score <= 100, FinovaError::InvalidParameter);
    require!(interaction_score <= 100, FinovaError::InvalidParameter);
    
    // Calculate weighted human probability score
    let weights = HumanProbabilityWeights {
        biometric: 25,      // 25%
        behavioral: 30,     // 30%
        social_graph: 20,   // 20%
        device: 15,         // 15%
        interaction: 10,    // 10%
    };
    
    let weighted_score = (
        (biometric_score as u32 * weights.biometric) +
        (behavioral_score as u32 * weights.behavioral) +
        (social_graph_score as u32 * weights.social_graph) +
        (device_score as u32 * weights.device) +
        (interaction_score as u32 * weights.interaction)
    ) / 100;
    
    // Apply temporal smoothing to prevent sudden changes
    let previous_score = user_account.human_probability_score;
    let smoothing_factor = if previous_score > 0 { 70 } else { 100 }; // 70% new, 30% old
    
    let final_score = if previous_score > 0 {
        ((weighted_score * smoothing_factor) + (previous_score as u32 * (100 - smoothing_factor))) / 100
    } else {
        weighted_score
    };
    
    user_account.human_probability_score = final_score as u16;
    user_account.last_probability_update = current_time;
    
    // Update status based on thresholds
    if final_score >= anti_bot_config.detection_threshold as u32 {
        user_account.is_verified_human = true;
        user_account.bot_flags = 0;
    } else if final_score <= anti_bot_config.suspicious_threshold as u32 {
        user_account.bot_flags += 1;
        user_account.is_verified_human = false;
        
        // Apply cooling period for suspicious users
        if user_account.bot_flags >= 3 {
            user_account.cooling_period_end = current_time + anti_bot_config.cooling_period;
        }
    }
    
    msg!(
        "Human probability updated for user {}: {}% (flags: {})",
        user_account.owner,
        final_score,
        user_account.bot_flags
    );
    
    Ok(())
}

/// Submit Proof of Humanity verification
pub fn submit_proof_of_humanity(
    ctx: Context<SubmitProofOfHumanity>,
    verification_type: u8,         // 1=KYC, 2=Biometric, 3=Social, 4=Device
    verification_data_hash: [u8; 32], // Hash of verification data
    metadata: String,              // Additional metadata
) -> Result<()> {
    let user_account = &mut ctx.accounts.user_account;
    let proof_of_humanity = &mut ctx.accounts.proof_of_humanity;
    let current_time = Clock::get()?.unix_timestamp;
    
    require!(verification_type >= 1 && verification_type <= 4, FinovaError::InvalidParameter);
    require!(metadata.len() <= MAX_METADATA_LENGTH, FinovaError::MetadataTooLong);
    
    proof_of_humanity.user = ctx.accounts.user.key();
    proof_of_humanity.verification_type = verification_type;
    proof_of_humanity.verification_data_hash = verification_data_hash;
    proof_of_humanity.metadata = metadata;
    proof_of_humanity.submitted_at = current_time;
    proof_of_humanity.status = 0; // 0=Pending, 1=Approved, 2=Rejected
    proof_of_humanity.verified_by = Pubkey::default();
    proof_of_humanity.verified_at = 0;
    proof_of_humanity.attempts = 1;
    proof_of_humanity.bump = *ctx.bumps.get("proof_of_humanity").unwrap();
    
    // Update user account
    user_account.poh_submissions += 1;
    user_account.last_poh_submission = current_time;
    
    // Bonus for KYC verification
    if verification_type == 1 {
        user_account.is_kyc_verified = true;
        user_account.human_probability_score = std::cmp::max(
            user_account.human_probability_score,
            MINIMUM_KYC_HUMAN_SCORE
        );
    }
    
    msg!(
        "Proof of Humanity submitted by user {}: type {}",
        ctx.accounts.user.key(),
        verification_type
    );
    
    Ok(())
}

/// Report suspicious activity
pub fn report_suspicious_activity(
    ctx: Context<ReportSuspiciousActivity>,
    activity_type: u8,             // 1=Bot behavior, 2=Spam, 3=Fake engagement, 4=Network abuse
    evidence_hash: [u8; 32],       // Hash of evidence data
    description: String,           // Description of suspicious activity
    severity: u8,                  // 1=Low, 2=Medium, 3=High, 4=Critical
    suspicious_report_count: u8,   // Current report count for this user
) -> Result<()> {
    let reported_user = &mut ctx.accounts.reported_user;
    let suspicious_report = &mut ctx.accounts.suspicious_report;
    let reporter_account = &ctx.accounts.reporter_account;
    let current_time = Clock::get()?.unix_timestamp;
    
    require!(activity_type >= 1 && activity_type <= 4, FinovaError::InvalidParameter);
    require!(severity >= 1 && severity <= 4, FinovaError::InvalidParameter);
    require!(description.len() <= MAX_DESCRIPTION_LENGTH, FinovaError::DescriptionTooLong);
    require!(reporter_account.is_verified_human, FinovaError::ReporterNotVerified);
    
    // Prevent self-reporting
    require!(
        ctx.accounts.reporter.key() != reported_user.owner,
        FinovaError::CannotReportSelf
    );
    
    // Check reporter's reputation
    require!(
        reporter_account.human_probability_score >= MINIMUM_REPORTER_SCORE,
        FinovaError::InsufficientReporterScore
    );
    
    suspicious_report.reported_user = reported_user.owner;
    suspicious_report.reporter = ctx.accounts.reporter.key();
    suspicious_report.activity_type = activity_type;
    suspicious_report.evidence_hash = evidence_hash;
    suspicious_report.description = description;
    suspicious_report.severity = severity;
    suspicious_report.reported_at = current_time;
    suspicious_report.status = 0; // 0=Pending, 1=Confirmed, 2=Dismissed
    suspicious_report.reviewed_by = Pubkey::default();
    suspicious_report.reviewed_at = 0;
    suspicious_report.bump = *ctx.bumps.get("suspicious_report").unwrap();
    
    // Update reported user's flags
    reported_user.suspicious_reports += 1;
    reported_user.bot_flags += severity as u32;
    
    // Apply immediate penalties for high severity reports
    if severity >= 3 {
        reported_user.human_probability_score = std::cmp::max(
            reported_user.human_probability_score.saturating_sub(severity as u16 * 5),
            0
        );
        
        // Apply cooling period for critical reports
        if severity == 4 {
            reported_user.cooling_period_end = current_time + (24 * 60 * 60); // 24 hours
        }
    }
    
    // Auto-suspend if too many reports
    if reported_user.suspicious_reports >= MAX_SUSPICIOUS_REPORTS {
        reported_user.is_suspended = true;
        reported_user.suspension_end = current_time + (7 * 24 * 60 * 60); // 7 days
    }
    
    msg!(
        "Suspicious activity reported: user {} reported by {} (type: {}, severity: {})",
        reported_user.owner,
        ctx.accounts.reporter.key(),
        activity_type,
        severity
    );
    
    Ok(())
}

/// Update bot detection parameters (admin only)
pub fn update_bot_detection_params(
    ctx: Context<UpdateBotDetectionParams>,
    new_detection_threshold: Option<u16>,
    new_suspicious_threshold: Option<u16>,
    new_penalty_multiplier: Option<u16>,
    new_cooling_period: Option<i64>,
) -> Result<()> {
    let anti_bot_config = &mut ctx.accounts.anti_bot_config;
    
    require!(
        ctx.accounts.authority.key() == anti_bot_config.authority,
        FinovaError::Unauthorized
    );
    
    if let Some(threshold) = new_detection_threshold {
        require!(threshold <= 100, FinovaError::InvalidParameter);
        anti_bot_config.detection_threshold = threshold;
    }
    
    if let Some(threshold) = new_suspicious_threshold {
        require!(threshold <= 100, FinovaError::InvalidParameter);
        anti_bot_config.suspicious_threshold = threshold;
    }
    
    if let Some(multiplier) = new_penalty_multiplier {
        require!(multiplier <= 1000, FinovaError::InvalidParameter);
        anti_bot_config.penalty_multiplier = multiplier;
    }
    
    if let Some(period) = new_cooling_period {
        require!(period > 0, FinovaError::InvalidParameter);
        anti_bot_config.cooling_period = period;
    }
    
    anti_bot_config.updated_at = Clock::get()?.unix_timestamp;
    
    msg!("Bot detection parameters updated by authority");
    
    Ok(())
}

/// Validate user behavior patterns
pub fn validate_user_behavior(
    ctx: Context<ValidateUserBehavior>,
    click_patterns: Vec<u32>,      // Array of click intervals in milliseconds
    session_durations: Vec<u32>,   // Array of session durations
    activity_timestamps: Vec<i64>, // Array of activity timestamps
    interaction_qualities: Vec<u16>, // Array of interaction quality scores
) -> Result<()> {
    let user_account = &mut ctx.accounts.user_account;
    let behavior_pattern = &mut ctx.accounts.behavior_pattern;
    let anti_bot_config = &ctx.accounts.anti_bot_config;
    let current_time = Clock::get()?.unix_timestamp;
    
    require!(click_patterns.len() <= MAX_PATTERN_SAMPLES, FinovaError::TooManyPatterns);
    require!(session_durations.len() <= MAX_PATTERN_SAMPLES, FinovaError::TooManyPatterns);
    require!(activity_timestamps.len() <= MAX_PATTERN_SAMPLES, FinovaError::TooManyPatterns);
    
    // Analyze click patterns
    let click_variance = calculate_variance(&click_patterns);
    let click_human_score = if click_variance > MIN_HUMAN_CLICK_VARIANCE {
        std::cmp::min(
            ((click_variance - MIN_HUMAN_CLICK_VARIANCE) * 100 / MAX_HUMAN_CLICK_VARIANCE) as u16,
            100
        )
    } else {
        0
    };
    
    // Analyze session patterns
    let avg_session_duration = if !session_durations.is_empty() {
        session_durations.iter().sum::<u32>() / session_durations.len() as u32
    } else {
        0
    };
    
    let session_human_score = if avg_session_duration >= MIN_HUMAN_SESSION_DURATION &&
                                avg_session_duration <= MAX_HUMAN_SESSION_DURATION {
        100
    } else {
        50
    };
    
    // Analyze temporal patterns
    let temporal_human_score = if activity_timestamps.len() >= 2 {
        let intervals: Vec<i64> = activity_timestamps.windows(2)
            .map(|w| w[1] - w[0])
            .collect();
        
        let avg_interval = intervals.iter().sum::<i64>() / intervals.len() as i64;
        
        if avg_interval >= MIN_HUMAN_ACTIVITY_INTERVAL &&
           avg_interval <= MAX_HUMAN_ACTIVITY_INTERVAL {
            100
        } else {
            30
        }
    } else {
        50
    };
    
    // Calculate composite behavior score
    let behavior_score = (
        (click_human_score as u32 * 30) +
        (session_human_score as u32 * 35) +
        (temporal_human_score as u32 * 35)
    ) / 100;
    
    // Update behavior pattern
    behavior_pattern.user = user_account.owner;
    behavior_pattern.click_variance = click_variance;
    behavior_pattern.avg_session_duration = avg_session_duration;
    behavior_pattern.behavior_score = behavior_score as u16;
    behavior_pattern.pattern_count += 1;
    behavior_pattern.last_analysis = current_time;
    behavior_pattern.updated_at = current_time;
    
    // Update user's overall human probability
    let current_prob = user_account.human_probability_score as u32;
    let new_prob = (current_prob * 70 + behavior_score * 30) / 100; // Weighted average
    
    user_account.human_probability_score = new_prob as u16;
    user_account.last_behavior_analysis = current_time;
    
    // Apply penalties if behavior is too robotic
    if behavior_score < anti_bot_config.suspicious_threshold as u32 {
        user_account.bot_flags += 1;
        
        if user_account.bot_flags >= MAX_BOT_FLAGS {
            user_account.cooling_period_end = current_time + anti_bot_config.cooling_period;
        }
    }
    
    msg!(
        "Behavior validated for user {}: score {}% (patterns: {})",
        user_account.owner,
        behavior_score,
        behavior_pattern.pattern_count
    );
    
    Ok(())
}

// Helper function to calculate variance
fn calculate_variance(data: &[u32]) -> u32 {
    if data.len() < 2 {
        return 0;
    }
    
    let mean = data.iter().sum::<u32>() / data.len() as u32;
    let variance = data.iter()
        .map(|&x| {
            let diff = if x > mean { x - mean } else { mean - x };
            diff * diff
        })
        .sum::<u32>() / data.len() as u32;
    
    variance
}

// Helper structures
#[derive(Debug, Clone)]
pub struct HumanProbabilityWeights {
    pub biometric: u32,
    pub behavioral: u32,
    pub social_graph: u32,
    pub device: u32,
    pub interaction: u32,
}

// State structures for anti-bot system
#[account]
pub struct AntiBotConfig {
    pub authority: Pubkey,
    pub detection_threshold: u16,
    pub suspicious_threshold: u16,
    pub penalty_multiplier: u16,
    pub cooling_period: i64,
    pub total_reports: u64,
    pub verified_humans: u64,
    pub suspected_bots: u64,
    pub bump: u8,
    pub created_at: i64,
    pub updated_at: i64,
}

impl AntiBotConfig {
    pub const SPACE: usize = 8 + 32 + 2 + 2 + 2 + 8 + 8 + 8 + 8 + 1 + 8 + 8;
}

#[account]
pub struct ProofOfHumanity {
    pub user: Pubkey,
    pub verification_type: u8,
    pub verification_data_hash: [u8; 32],
    pub metadata: String,
    pub submitted_at: i64,
    pub status: u8,
    pub verified_by: Pubkey,
    pub verified_at: i64,
    pub attempts: u16,
    pub bump: u8,
}

impl ProofOfHumanity {
    pub const SPACE: usize = 8 + 32 + 1 + 32 + 256 + 8 + 1 + 32 + 8 + 2 + 1;
}

#[account]
pub struct SuspiciousActivityReport {
    pub reported_user: Pubkey,
    pub reporter: Pubkey,
    pub activity_type: u8,
    pub evidence_hash: [u8; 32],
    pub description: String,
    pub severity: u8,
    pub reported_at: i64,
    pub status: u8,
    pub reviewed_by: Pubkey,
    pub reviewed_at: i64,
    pub bump: u8,
}

impl SuspiciousActivityReport {
    pub const SPACE: usize = 8 + 32 + 32 + 1 + 32 + 512 + 1 + 8 + 1 + 32 + 8 + 1;
}

#[account]
pub struct BehaviorPattern {
    pub user: Pubkey,
    pub click_variance: u32,
    pub avg_session_duration: u32,
    pub behavior_score: u16,
    pub pattern_count: u32,
    pub last_analysis: i64,
    pub updated_at: i64,
    pub bump: u8,
}

impl BehaviorPattern {
    pub const SPACE: usize = 8 + 32 + 4 + 4 + 2 + 4 + 8 + 8 + 1;
}

// Constants for anti-bot system
pub const ANTI_BOT_CONFIG_SEED: &[u8] = b"anti_bot_config";
pub const POH_SEED: &[u8] = b"proof_of_humanity";
pub const SUSPICIOUS_REPORT_SEED: &[u8] = b"suspicious_report";
pub const BEHAVIOR_PATTERN_SEED: &[u8] = b"behavior_pattern";

pub const MAX_METADATA_LENGTH: usize = 256;
pub const MAX_DESCRIPTION_LENGTH: usize = 512;
pub const MAX_SUSPICIOUS_REPORTS: u32 = 5;
pub const MAX_BOT_FLAGS: u32 = 10;
pub const MAX_PATTERN_SAMPLES: usize = 100;

pub const MINIMUM_KYC_HUMAN_SCORE: u16 = 80;
pub const MINIMUM_REPORTER_SCORE: u16 = 70;

pub const MIN_HUMAN_CLICK_VARIANCE: u32 = 1000;
pub const MAX_HUMAN_CLICK_VARIANCE: u32 = 50000;
pub const MIN_HUMAN_SESSION_DURATION: u32 = 30000;  // 30 seconds
pub const MAX_HUMAN_SESSION_DURATION: u32 = 3600000; // 1 hour
pub const MIN_HUMAN_ACTIVITY_INTERVAL: i64 = 5;     // 5 seconds
pub const MAX_HUMAN_ACTIVITY_INTERVAL: i64 = 86400; // 24 hours
