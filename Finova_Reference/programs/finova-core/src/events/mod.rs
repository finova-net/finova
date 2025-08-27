// programs/finova-core/src/events/mod.rs

use anchor_lang::prelude::*;

// Re-export all event modules
pub mod mining;
pub mod xp;
pub mod referral;
pub mod governance;
pub mod staking;
pub mod guild;
pub mod rewards;
pub mod anti_bot;

pub use mining::*;
pub use xp::*;
pub use referral::*;
pub use governance::*;
pub use staking::*;
pub use guild::*;
pub use rewards::*;
pub use anti_bot::*;

/// Core system events for the Finova Network
/// These events provide comprehensive logging and monitoring capabilities
/// for all major system operations as outlined in the whitepaper

/// User lifecycle events
#[event]
pub struct UserRegistered {
    pub user: Pubkey,
    pub referrer: Option<Pubkey>,
    pub timestamp: i64,
    pub registration_method: String,
    pub initial_mining_rate: u64,
}

#[event]
pub struct UserKycVerified {
    pub user: Pubkey,
    pub kyc_provider: String,
    pub verification_level: u8,
    pub timestamp: i64,
    pub security_bonus_activated: bool,
}

#[event]
pub struct UserProfileUpdated {
    pub user: Pubkey,
    pub field_updated: String,
    pub old_value: String,
    pub new_value: String,
    pub timestamp: i64,
}

#[event]
pub struct UserSuspended {
    pub user: Pubkey,
    pub reason: String,
    pub suspension_type: u8, // 1: temporary, 2: permanent
    pub duration: Option<i64>,
    pub timestamp: i64,
    pub admin: Pubkey,
}

#[event]
pub struct UserReinstated {
    pub user: Pubkey,
    pub reason: String,
    pub timestamp: i64,
    pub admin: Pubkey,
}

/// Network growth and phase transition events
#[event]
pub struct NetworkPhaseTransition {
    pub from_phase: u8,
    pub to_phase: u8,
    pub total_users: u64,
    pub new_base_mining_rate: u64,
    pub finizen_bonus: u64,
    pub timestamp: i64,
}

#[event]
pub struct MilestoneReached {
    pub milestone_type: String, // "user_count", "total_mined", "network_value"
    pub milestone_value: u64,
    pub total_users: u64,
    pub total_fin_mined: u64,
    pub timestamp: i64,
}

/// Token economics events
#[event]
pub struct TokenMinted {
    pub recipient: Pubkey,
    pub amount: u64,
    pub mint_type: String, // "mining", "referral", "xp_bonus", "special_event"
    pub multiplier_applied: u64,
    pub timestamp: i64,
}

#[event]
pub struct TokenBurned {
    pub user: Pubkey,
    pub amount: u64,
    pub burn_reason: String, // "nft_usage", "transaction_fee", "whale_tax"
    pub timestamp: i64,
}

#[event]
pub struct TokenTransfer {
    pub from: Pubkey,
    pub to: Pubkey,
    pub amount: u64,
    pub transfer_type: String, // "p2p", "marketplace", "guild", "staking"
    pub fee_amount: u64,
    pub timestamp: i64,
}

/// Special events and bonuses
#[event]
pub struct SpecialEventStarted {
    pub event_id: String,
    pub event_type: String, // "double_mining", "xp_boost", "referral_bonus"
    pub multiplier: u64,
    pub duration: i64,
    pub start_time: i64,
    pub end_time: i64,
    pub eligible_users: Option<u64>,
}

#[event]
pub struct SpecialEventEnded {
    pub event_id: String,
    pub total_participants: u64,
    pub total_rewards_distributed: u64,
    pub timestamp: i64,
}

#[event]
pub struct BonusRewardDistributed {
    pub user: Pubkey,
    pub bonus_type: String, // "streak", "milestone", "loyalty", "special_event"
    pub amount: u64,
    pub multiplier: u64,
    pub timestamp: i64,
}

/// Quality assessment and AI events
#[event]
pub struct ContentQualityAssessed {
    pub user: Pubkey,
    pub content_id: String,
    pub platform: String,
    pub quality_score: u64, // 50-200 (0.5x - 2.0x multiplier * 100)
    pub originality_score: u64,
    pub engagement_prediction: u64,
    pub ai_confidence: u64,
    pub timestamp: i64,
}

#[event]
pub struct QualityThresholdUpdated {
    pub old_threshold: u64,
    pub new_threshold: u64,
    pub reason: String,
    pub updated_by: Pubkey,
    pub timestamp: i64,
}

/// Anti-bot and security events
#[event]
pub struct SuspiciousActivityDetected {
    pub user: Pubkey,
    pub activity_type: String,
    pub suspicion_score: u64,
    pub detection_method: String, // "ai_pattern", "behavioral", "network_analysis"
    pub auto_action_taken: String, // "none", "rate_limit", "temporary_suspend"
    pub timestamp: i64,
}

#[event]
pub struct BotDetectionScoreUpdated {
    pub user: Pubkey,
    pub old_score: u64,
    pub new_score: u64,
    pub contributing_factors: Vec<String>,
    pub timestamp: i64,
}

/// Network health and monitoring events
#[event]
pub struct NetworkHealthCheck {
    pub total_active_users: u64,
    pub total_mining_rate: u64,
    pub average_quality_score: u64,
    pub suspicious_activity_count: u64,
    pub network_growth_rate: u64, // percentage * 100
    pub timestamp: i64,
}

#[event]
pub struct SystemMaintenanceScheduled {
    pub maintenance_type: String,
    pub scheduled_start: i64,
    pub estimated_duration: i64,
    pub affected_features: Vec<String>,
    pub scheduled_by: Pubkey,
}

#[event]
pub struct SystemMaintenanceCompleted {
    pub maintenance_type: String,
    pub actual_start: i64,
    pub actual_duration: i64,
    pub changes_made: Vec<String>,
    pub success: bool,
}

/// Emergency and circuit breaker events
#[event]
pub struct CircuitBreakerTriggered {
    pub breaker_type: String, // "mining", "xp", "referral", "global"
    pub trigger_condition: String,
    pub threshold_value: u64,
    pub current_value: u64,
    pub auto_recovery_time: Option<i64>,
    pub timestamp: i64,
}

#[event]
pub struct CircuitBreakerReset {
    pub breaker_type: String,
    pub reset_by: Pubkey,
    pub reset_reason: String,
    pub timestamp: i64,
}

#[event]
pub struct EmergencyPause {
    pub component: String, // "mining", "staking", "all"
    pub reason: String,
    pub paused_by: Pubkey,
    pub estimated_resolution: Option<i64>,
    pub timestamp: i64,
}

#[event]
pub struct EmergencyUnpause {
    pub component: String,
    pub unpaused_by: Pubkey,
    pub resolution_summary: String,
    pub timestamp: i64,
}

/// Configuration and parameter updates
#[event]
pub struct ParameterUpdated {
    pub parameter_name: String,
    pub old_value: String,
    pub new_value: String,
    pub updated_by: Pubkey,
    pub governance_proposal: Option<Pubkey>,
    pub timestamp: i64,
}

#[event]
pub struct FormulaUpdated {
    pub formula_type: String, // "mining", "xp", "referral", "quality"
    pub version: String,
    pub changes: Vec<String>,
    pub updated_by: Pubkey,
    pub effective_date: i64,
    pub timestamp: i64,
}

/// Integration and external service events
#[event]
pub struct SocialPlatformIntegrated {
    pub user: Pubkey,
    pub platform: String,
    pub platform_user_id: String,
    pub verification_status: String,
    pub integration_timestamp: i64,
}

#[event]
pub struct SocialPlatformDisconnected {
    pub user: Pubkey,
    pub platform: String,
    pub reason: String, // "user_request", "platform_error", "security_concern"
    pub timestamp: i64,
}

#[event]
pub struct ExternalApiCall {
    pub service: String,
    pub endpoint: String,
    pub response_status: u16,
    pub response_time_ms: u64,
    pub success: bool,
    pub error_message: Option<String>,
    pub timestamp: i64,
}

/// Reward pool and treasury events
#[event]
pub struct RewardPoolFunded {
    pub amount: u64,
    pub funding_source: String, // "revenue", "treasury", "partnership"
    pub pool_type: String, // "mining", "xp", "referral", "special"
    pub funded_by: Pubkey,
    pub timestamp: i64,
}

#[event]
pub struct RewardPoolDepleted {
    pub pool_type: String,
    pub remaining_amount: u64,
    pub depletion_rate: u64, // tokens per hour
    pub estimated_refill_needed: i64,
    pub timestamp: i64,
}

#[event]
pub struct TreasuryOperation {
    pub operation_type: String, // "deposit", "withdrawal", "transfer", "burn"
    pub amount: u64,
    pub destination: Option<Pubkey>,
    pub purpose: String,
    pub authorized_by: Pubkey,
    pub timestamp: i64,
}

/// Analytics and reporting events
#[event]
pub struct DailyMetricsSnapshot {
    pub date: i64,
    pub total_users: u64,
    pub active_users_24h: u64,
    pub total_fin_mined: u64,
    pub fin_mined_24h: u64,
    pub average_mining_rate: u64,
    pub total_xp_earned: u64,
    pub xp_earned_24h: u64,
    pub referral_signups_24h: u64,
    pub quality_score_average: u64,
}

#[event]
pub struct WeeklyAnalyticsReport {
    pub week_start: i64,
    pub user_growth_rate: u64, // percentage * 100
    pub retention_rate_7d: u64,
    pub churn_rate: u64,
    pub average_session_duration: u64, // minutes
    pub most_active_platform: String,
    pub top_referrer: Option<Pubkey>,
    pub network_health_score: u64,
}

/// Error and exception events
#[event]
pub struct SystemError {
    pub error_code: String,
    pub error_message: String,
    pub component: String,
    pub severity: String, // "low", "medium", "high", "critical"
    pub user_affected: Option<Pubkey>,
    pub stack_trace: Option<String>,
    pub timestamp: i64,
}

#[event]
pub struct RecoveryAction {
    pub error_reference: String,
    pub action_taken: String,
    pub success: bool,
    pub recovery_time_ms: u64,
    pub timestamp: i64,
}

/// Helper functions for event emission
impl UserRegistered {
    pub fn emit_with_context(
        user: Pubkey,
        referrer: Option<Pubkey>,
        registration_method: String,
        initial_mining_rate: u64,
    ) {
        emit!(UserRegistered {
            user,
            referrer,
            timestamp: Clock::get().unwrap().unix_timestamp,
            registration_method,
            initial_mining_rate,
        });
    }
}

impl NetworkPhaseTransition {
    pub fn emit_transition(
        from_phase: u8,
        to_phase: u8,
        total_users: u64,
        new_base_mining_rate: u64,
        finizen_bonus: u64,
    ) {
        emit!(NetworkPhaseTransition {
            from_phase,
            to_phase,
            total_users,
            new_base_mining_rate,
            finizen_bonus,
            timestamp: Clock::get().unwrap().unix_timestamp,
        });
    }
}

impl SuspiciousActivityDetected {
    pub fn emit_detection(
        user: Pubkey,
        activity_type: String,
        suspicion_score: u64,
        detection_method: String,
        auto_action_taken: String,
    ) {
        emit!(SuspiciousActivityDetected {
            user,
            activity_type,
            suspicion_score,
            detection_method,
            auto_action_taken,
            timestamp: Clock::get().unwrap().unix_timestamp,
        });
    }
}

impl SystemError {
    pub fn emit_error(
        error_code: String,
        error_message: String,
        component: String,
        severity: String,
        user_affected: Option<Pubkey>,
    ) {
        emit!(SystemError {
            error_code,
            error_message,
            component,
            severity,
            user_affected,
            stack_trace: None,
            timestamp: Clock::get().unwrap().unix_timestamp,
        });
    }
}

/// Event filtering and categorization utilities
pub struct EventFilter;

impl EventFilter {
    pub fn is_security_event(event_name: &str) -> bool {
        matches!(
            event_name,
            "SuspiciousActivityDetected" 
            | "BotDetectionScoreUpdated" 
            | "UserSuspended" 
            | "CircuitBreakerTriggered"
            | "EmergencyPause"
        )
    }

    pub fn is_financial_event(event_name: &str) -> bool {
        matches!(
            event_name,
            "TokenMinted" 
            | "TokenBurned" 
            | "TokenTransfer" 
            | "RewardPoolFunded"
            | "TreasuryOperation"
        )
    }

    pub fn is_user_activity_event(event_name: &str) -> bool {
        matches!(
            event_name,
            "MiningStarted" 
            | "MiningCompleted" 
            | "XpEarned" 
            | "ReferralRegistered"
            | "ContentQualityAssessed"
        )
    }

    pub fn is_system_health_event(event_name: &str) -> bool {
        matches!(
            event_name,
            "NetworkHealthCheck" 
            | "SystemError" 
            | "RecoveryAction"
            | "ExternalApiCall"
        )
    }

    pub fn requires_immediate_attention(event_name: &str) -> bool {
        matches!(
            event_name,
            "CircuitBreakerTriggered" 
            | "EmergencyPause" 
            | "SystemError"
            | "RewardPoolDepleted"
        )
    }
}

/// Event analytics and aggregation helpers
pub struct EventAnalytics;

impl EventAnalytics {
    pub fn calculate_event_frequency(events: &[String], time_window: i64) -> u64 {
        // Implementation would analyze event frequency within time window
        events.len() as u64
    }

    pub fn detect_event_anomalies(events: &[String], baseline: u64) -> bool {
        // Implementation would detect unusual event patterns
        events.len() as u64 > baseline * 2
    }

    pub fn generate_event_summary(events: &[String]) -> String {
        format!("Processed {} events", events.len())
    }
}

/// Constants for event categorization
pub mod event_constants {
    // Event severity levels
    pub const SEVERITY_LOW: &str = "low";
    pub const SEVERITY_MEDIUM: &str = "medium";
    pub const SEVERITY_HIGH: &str = "high";
    pub const SEVERITY_CRITICAL: &str = "critical";

    // Event categories
    pub const CATEGORY_SECURITY: &str = "security";
    pub const CATEGORY_FINANCIAL: &str = "financial";
    pub const CATEGORY_USER_ACTIVITY: &str = "user_activity";
    pub const CATEGORY_SYSTEM_HEALTH: &str = "system_health";
    pub const CATEGORY_GOVERNANCE: &str = "governance";

    // Auto-actions for security events
    pub const ACTION_NONE: &str = "none";
    pub const ACTION_RATE_LIMIT: &str = "rate_limit";
    pub const ACTION_TEMPORARY_SUSPEND: &str = "temporary_suspend";
    pub const ACTION_PERMANENT_SUSPEND: &str = "permanent_suspend";
    pub const ACTION_REQUIRE_REVERIFICATION: &str = "require_reverification";

    // Network phases
    pub const PHASE_FINIZEN: u8 = 1;
    pub const PHASE_GROWTH: u8 = 2;
    pub const PHASE_MATURITY: u8 = 3;
    pub const PHASE_STABILITY: u8 = 4;

    // Registration methods
    pub const REGISTRATION_DIRECT: &str = "direct";
    pub const REGISTRATION_REFERRAL: &str = "referral";
    pub const REGISTRATION_SOCIAL: &str = "social";
    pub const REGISTRATION_MIGRATION: &str = "migration";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_filter_security() {
        assert!(EventFilter::is_security_event("SuspiciousActivityDetected"));
        assert!(EventFilter::is_security_event("UserSuspended"));
        assert!(!EventFilter::is_security_event("TokenMinted"));
    }

    #[test]
    fn test_event_filter_financial() {
        assert!(EventFilter::is_financial_event("TokenMinted"));
        assert!(EventFilter::is_financial_event("TokenBurned"));
        assert!(!EventFilter::is_financial_event("UserRegistered"));
    }

    #[test]
    fn test_immediate_attention_required() {
        assert!(EventFilter::requires_immediate_attention("CircuitBreakerTriggered"));
        assert!(EventFilter::requires_immediate_attention("EmergencyPause"));
        assert!(!EventFilter::requires_immediate_attention("UserRegistered"));
    }

    #[test]
    fn test_event_analytics() {
        let events = vec!["Event1".to_string(), "Event2".to_string()];
        let frequency = EventAnalytics::calculate_event_frequency(&events, 3600);
        assert_eq!(frequency, 2);

        let has_anomalies = EventAnalytics::detect_event_anomalies(&events, 5);
        assert!(!has_anomalies);
    }
}
