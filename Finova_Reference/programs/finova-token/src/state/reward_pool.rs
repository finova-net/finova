// programs/finova-token/src/state/reward_pool.rs

use anchor_lang::prelude::*;
use std::collections::BTreeMap;

/// Reward pool state managing all reward distribution mechanisms
#[account]
#[derive(Default)]
pub struct RewardPool {
    /// Authority managing the reward pool
    pub authority: Pubkey,
    /// Total rewards allocated to the pool
    pub total_allocated: u64,
    /// Total rewards distributed so far
    pub total_distributed: u64,
    /// Total rewards claimed by users
    pub total_claimed: u64,
    /// Current pool balance
    pub current_balance: u64,
    /// Pool configuration
    pub config: RewardPoolConfig,
    /// Distribution phases
    pub phases: Vec<DistributionPhase>,
    /// Current active phase index
    pub current_phase: u8,
    /// Pool statistics
    pub stats: PoolStatistics,
    /// Emergency controls
    pub emergency: EmergencyControls,
    /// Reward multipliers for different activities
    pub multipliers: ActivityMultipliers,
    /// Time-based distribution settings
    pub time_settings: TimeSettings,
    /// Anti-gaming mechanisms
    pub anti_gaming: AntiGamingConfig,
    /// Bump seed for PDA
    pub bump: u8,
}

/// Configuration for the reward pool
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default)]
pub struct RewardPoolConfig {
    /// Minimum rewards per distribution
    pub min_reward_per_distribution: u64,
    /// Maximum rewards per distribution
    pub max_reward_per_distribution: u64,
    /// Distribution frequency in seconds
    pub distribution_frequency: i64,
    /// Staking bonus multiplier (basis points)
    pub staking_bonus_bps: u16,
    /// Referral bonus multiplier (basis points)
    pub referral_bonus_bps: u16,
    /// Quality score impact (basis points)
    pub quality_impact_bps: u16,
    /// Pool sustainability factor
    pub sustainability_factor: u16,
    /// Maximum individual claim per period
    pub max_individual_claim: u64,
    /// Vesting period for large rewards
    pub vesting_period: i64,
    /// Compound interest rate (basis points)
    pub compound_rate_bps: u16,
}

/// Distribution phase configuration
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default)]
pub struct DistributionPhase {
    /// Phase identifier
    pub phase_id: u8,
    /// Phase name
    pub name: String,
    /// Start timestamp
    pub start_time: i64,
    /// End timestamp
    pub end_time: i64,
    /// Base reward rate for this phase
    pub base_rate: u64,
    /// Phase multiplier (basis points)
    pub phase_multiplier: u16,
    /// Total allocation for this phase
    pub phase_allocation: u64,
    /// Amount distributed in this phase
    pub distributed_amount: u64,
    /// Number of participants in this phase
    pub participant_count: u32,
    /// Phase status
    pub status: PhaseStatus,
    /// Special conditions for this phase
    pub special_conditions: SpecialConditions,
}

/// Phase status enumeration
#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq)]
pub enum PhaseStatus {
    Pending,
    Active,
    Paused,
    Completed,
    Emergency,
}

impl Default for PhaseStatus {
    fn default() -> Self {
        PhaseStatus::Pending
    }
}

/// Special conditions for distribution phases
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default)]
pub struct SpecialConditions {
    /// Minimum staking requirement
    pub min_staking_requirement: u64,
    /// Required KYC level
    pub required_kyc_level: u8,
    /// Minimum XP level
    pub min_xp_level: u32,
    /// Geographic restrictions
    pub geo_restrictions: Vec<String>,
    /// Activity requirements
    pub activity_requirements: ActivityRequirements,
    /// Network effect requirements
    pub network_requirements: NetworkRequirements,
}

/// Activity requirements for reward eligibility
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default)]
pub struct ActivityRequirements {
    /// Minimum daily activities
    pub min_daily_activities: u16,
    /// Required platforms for cross-platform bonus
    pub required_platforms: u8,
    /// Minimum content quality score
    pub min_quality_score: u16,
    /// Social engagement threshold
    pub social_engagement_threshold: u32,
    /// Consistency requirement (days)
    pub consistency_days: u16,
}

/// Network requirements for enhanced rewards
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default)]
pub struct NetworkRequirements {
    /// Minimum active referrals
    pub min_active_referrals: u16,
    /// Required network depth levels
    pub required_network_depth: u8,
    /// Network quality threshold
    pub network_quality_threshold: u16,
    /// Minimum network growth rate
    pub min_network_growth_rate: u16,
}

/// Pool statistics tracking
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default)]
pub struct PoolStatistics {
    /// Total unique recipients
    pub unique_recipients: u32,
    /// Average reward per recipient
    pub avg_reward_per_recipient: u64,
    /// Peak distribution amount
    pub peak_distribution: u64,
    /// Peak distribution timestamp
    pub peak_distribution_time: i64,
    /// Total transactions processed
    pub total_transactions: u64,
    /// Failed distribution attempts
    pub failed_distributions: u32,
    /// Pool utilization rate (basis points)
    pub utilization_rate: u16,
    /// Distribution efficiency metrics
    pub efficiency_metrics: EfficiencyMetrics,
    /// Historical performance data
    pub performance_history: Vec<PerformanceSnapshot>,
}

/// Efficiency metrics for pool operations
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default)]
pub struct EfficiencyMetrics {
    /// Average distribution processing time (ms)
    pub avg_processing_time: u32,
    /// Success rate (basis points)
    pub success_rate: u16,
    /// Cost per distribution (lamports)
    pub cost_per_distribution: u64,
    /// Gas efficiency score
    pub gas_efficiency_score: u16,
    /// System load impact score
    pub load_impact_score: u16,
}

/// Performance snapshot for historical tracking
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default)]
pub struct PerformanceSnapshot {
    /// Snapshot timestamp
    pub timestamp: i64,
    /// Total distributed at this point
    pub total_distributed: u64,
    /// Active participants count
    pub active_participants: u32,
    /// Average reward amount
    pub avg_reward: u64,
    /// Pool health score
    pub health_score: u16,
    /// Sustainability indicator
    pub sustainability_score: u16,
}

/// Emergency controls for pool management
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default)]
pub struct EmergencyControls {
    /// Emergency pause status
    pub is_paused: bool,
    /// Emergency pause timestamp
    pub pause_timestamp: i64,
    /// Authorized emergency operators
    pub emergency_operators: Vec<Pubkey>,
    /// Circuit breaker thresholds
    pub circuit_breaker: CircuitBreakerConfig,
    /// Recovery procedures
    pub recovery_config: RecoveryConfig,
    /// Incident tracking
    pub incident_log: Vec<IncidentRecord>,
}

/// Circuit breaker configuration
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default)]
pub struct CircuitBreakerConfig {
    /// Maximum distribution per hour
    pub max_hourly_distribution: u64,
    /// Maximum failed attempts before pause
    pub max_failed_attempts: u16,
    /// Anomaly detection threshold
    pub anomaly_threshold: u16,
    /// Auto-recovery timeout
    pub auto_recovery_timeout: i64,
    /// Alert thresholds
    pub alert_thresholds: AlertThresholds,
}

/// Alert threshold configuration
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default)]
pub struct AlertThresholds {
    /// Low balance alert (percentage)
    pub low_balance_alert: u8,
    /// High utilization alert (percentage)
    pub high_utilization_alert: u8,
    /// Unusual activity alert threshold
    pub unusual_activity_threshold: u16,
    /// Performance degradation threshold
    pub performance_degradation_threshold: u16,
}

/// Recovery configuration
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default)]
pub struct RecoveryConfig {
    /// Gradual recovery enabled
    pub gradual_recovery_enabled: bool,
    /// Recovery phase duration
    pub recovery_phase_duration: i64,
    /// Recovery rate multiplier
    pub recovery_rate_multiplier: u16,
    /// Post-recovery monitoring period
    pub monitoring_period: i64,
    /// Recovery validation criteria
    pub validation_criteria: ValidationCriteria,
}

/// Validation criteria for recovery
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default)]
pub struct ValidationCriteria {
    /// Minimum system stability period
    pub min_stability_period: i64,
    /// Required success rate for validation
    pub required_success_rate: u16,
    /// Maximum allowed error rate
    pub max_error_rate: u16,
    /// Performance benchmark threshold
    pub performance_benchmark: u16,
}

/// Incident record for tracking issues
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default)]
pub struct IncidentRecord {
    /// Incident ID
    pub incident_id: u64,
    /// Incident timestamp
    pub timestamp: i64,
    /// Incident type
    pub incident_type: IncidentType,
    /// Severity level
    pub severity: SeverityLevel,
    /// Description of the incident
    pub description: String,
    /// Resolution status
    pub resolution_status: ResolutionStatus,
    /// Impact assessment
    pub impact: ImpactAssessment,
}

/// Incident type enumeration
#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq)]
pub enum IncidentType {
    SystemFailure,
    SecurityBreach,
    PerformanceDegradation,
    DataCorruption,
    NetworkIssue,
    UserError,
    ConfigurationError,
    ExternalDependency,
}

impl Default for IncidentType {
    fn default() -> Self {
        IncidentType::SystemFailure
    }
}

/// Severity level enumeration
#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq)]
pub enum SeverityLevel {
    Critical,
    High,
    Medium,
    Low,
    Info,
}

impl Default for SeverityLevel {
    fn default() -> Self {
        SeverityLevel::Low
    }
}

/// Resolution status enumeration
#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq)]
pub enum ResolutionStatus {
    Open,
    InProgress,
    Resolved,
    Closed,
    Escalated,
}

impl Default for ResolutionStatus {
    fn default() -> Self {
        ResolutionStatus::Open
    }
}

/// Impact assessment for incidents
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default)]
pub struct ImpactAssessment {
    /// Number of affected users
    pub affected_users: u32,
    /// Financial impact (in tokens)
    pub financial_impact: u64,
    /// Service downtime (in seconds)
    pub downtime_duration: i64,
    /// Reputation impact score
    pub reputation_impact: u16,
    /// Recovery time estimate
    pub recovery_time_estimate: i64,
}

/// Activity multipliers for different reward types
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default)]
pub struct ActivityMultipliers {
    /// Mining activity multiplier
    pub mining_multiplier: u16,
    /// Social engagement multiplier
    pub social_multiplier: u16,
    /// Referral activity multiplier
    pub referral_multiplier: u16,
    /// Staking loyalty multiplier
    pub staking_multiplier: u16,
    /// Content creation multiplier
    pub content_multiplier: u16,
    /// Community participation multiplier
    pub community_multiplier: u16,
    /// Special event multipliers
    pub event_multipliers: BTreeMap<String, u16>,
    /// Time-based multipliers
    pub time_multipliers: TimeMultipliers,
}

/// Time-based multiplier configuration
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default)]
pub struct TimeMultipliers {
    /// Early bird bonus multiplier
    pub early_bird_multiplier: u16,
    /// Peak hours multiplier
    pub peak_hours_multiplier: u16,
    /// Weekend bonus multiplier
    pub weekend_multiplier: u16,
    /// Holiday special multiplier
    pub holiday_multiplier: u16,
    /// Anniversary bonus multiplier
    pub anniversary_multiplier: u16,
}

/// Time settings for reward distribution
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default)]
pub struct TimeSettings {
    /// Pool creation timestamp
    pub creation_time: i64,
    /// Last distribution timestamp
    pub last_distribution: i64,
    /// Next scheduled distribution
    pub next_distribution: i64,
    /// Distribution window duration
    pub distribution_window: i64,
    /// Timezone offset for calculations
    pub timezone_offset: i32,
    /// Daylight saving adjustment
    pub dst_adjustment: bool,
    /// Business hours configuration
    pub business_hours: BusinessHours,
    /// Maintenance windows
    pub maintenance_windows: Vec<MaintenanceWindow>,
}

/// Business hours configuration
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default)]
pub struct BusinessHours {
    /// Start hour (24-hour format)
    pub start_hour: u8,
    /// End hour (24-hour format)
    pub end_hour: u8,
    /// Operating days (bit mask)
    pub operating_days: u8,
    /// Special business hour rules
    pub special_rules: Vec<SpecialHourRule>,
}

/// Special hour rule for business operations
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default)]
pub struct SpecialHourRule {
    /// Rule name
    pub name: String,
    /// Start date
    pub start_date: i64,
    /// End date
    pub end_date: i64,
    /// Modified start hour
    pub modified_start_hour: u8,
    /// Modified end hour
    pub modified_end_hour: u8,
    /// Rule priority
    pub priority: u8,
}

/// Maintenance window configuration
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default)]
pub struct MaintenanceWindow {
    /// Window ID
    pub window_id: u32,
    /// Start timestamp
    pub start_time: i64,
    /// End timestamp
    pub end_time: i64,
    /// Maintenance type
    pub maintenance_type: MaintenanceType,
    /// Impact level
    pub impact_level: ImpactLevel,
    /// Description
    pub description: String,
    /// Notification sent
    pub notification_sent: bool,
}

/// Maintenance type enumeration
#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq)]
pub enum MaintenanceType {
    Scheduled,
    Emergency,
    Upgrade,
    Security,
    Performance,
}

impl Default for MaintenanceType {
    fn default() -> Self {
        MaintenanceType::Scheduled
    }
}

/// Impact level enumeration
#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq)]
pub enum ImpactLevel {
    NoImpact,
    LowImpact,
    MediumImpact,
    HighImpact,
    ServiceUnavailable,
}

impl Default for ImpactLevel {
    fn default() -> Self {
        ImpactLevel::NoImpact
    }
}

/// Anti-gaming configuration
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default)]
pub struct AntiGamingConfig {
    /// Maximum rewards per user per day
    pub max_daily_rewards_per_user: u64,
    /// Minimum time between claims
    pub min_claim_interval: i64,
    /// Behavior analysis enabled
    pub behavior_analysis_enabled: bool,
    /// Suspicious activity threshold
    pub suspicious_activity_threshold: u16,
    /// Automatic penalty system
    pub auto_penalty_system: AutoPenaltyConfig,
    /// Whitelist for trusted users
    pub trusted_user_whitelist: Vec<Pubkey>,
    /// Reputation system integration
    pub reputation_integration: ReputationIntegration,
}

/// Automatic penalty system configuration
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default)]
pub struct AutoPenaltyConfig {
    /// Penalty enabled
    pub enabled: bool,
    /// Warning threshold
    pub warning_threshold: u16,
    /// Temporary ban threshold
    pub temp_ban_threshold: u16,
    /// Permanent ban threshold
    pub permanent_ban_threshold: u16,
    /// Penalty reduction rate
    pub penalty_reduction_rate: u16,
    /// Appeal process enabled
    pub appeal_process_enabled: bool,
}

/// Reputation system integration
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default)]
pub struct ReputationIntegration {
    /// Minimum reputation score for rewards
    pub min_reputation_score: u32,
    /// Reputation bonus multiplier
    pub reputation_bonus_multiplier: u16,
    /// Reputation decay rate
    pub reputation_decay_rate: u16,
    /// Community voting weight
    pub community_voting_weight: u16,
}

impl RewardPool {
    /// Size of the RewardPool account in bytes
    pub const SIZE: usize = 8 + // discriminator
        32 + // authority
        8 + // total_allocated
        8 + // total_distributed
        8 + // total_claimed
        8 + // current_balance
        std::mem::size_of::<RewardPoolConfig>() +
        4 + (std::mem::size_of::<DistributionPhase>() * 10) + // phases (max 10)
        1 + // current_phase
        std::mem::size_of::<PoolStatistics>() +
        std::mem::size_of::<EmergencyControls>() +
        std::mem::size_of::<ActivityMultipliers>() +
        std::mem::size_of::<TimeSettings>() +
        std::mem::size_of::<AntiGamingConfig>() +
        1 + // bump
        1000; // buffer for future expansions and dynamic content

    /// Initialize a new reward pool
    pub fn initialize(
        &mut self,
        authority: Pubkey,
        initial_allocation: u64,
        config: RewardPoolConfig,
        bump: u8,
    ) -> Result<()> {
        self.authority = authority;
        self.total_allocated = initial_allocation;
        self.current_balance = initial_allocation;
        self.config = config;
        self.bump = bump;
        
        // Initialize with default phase
        let initial_phase = DistributionPhase {
            phase_id: 0,
            name: "Genesis Phase".to_string(),
            start_time: Clock::get()?.unix_timestamp,
            end_time: Clock::get()?.unix_timestamp + 86400 * 30, // 30 days
            base_rate: 1000, // 0.001 FIN per hour
            phase_multiplier: 10000, // 100% (basis points)
            phase_allocation: initial_allocation / 4, // 25% for first phase
            ..Default::default()
        };
        
        self.phases.push(initial_phase);
        self.current_phase = 0;
        
        // Initialize time settings
        self.time_settings.creation_time = Clock::get()?.unix_timestamp;
        self.time_settings.next_distribution = Clock::get()?.unix_timestamp + 3600; // 1 hour
        
        // Initialize default multipliers
        self.multipliers = ActivityMultipliers {
            mining_multiplier: 10000, // 100%
            social_multiplier: 8000,  // 80%
            referral_multiplier: 12000, // 120%
            staking_multiplier: 15000, // 150%
            content_multiplier: 9000, // 90%
            community_multiplier: 11000, // 110%
            ..Default::default()
        };
        
        Ok(())
    }

    /// Add rewards to the pool
    pub fn add_rewards(&mut self, amount: u64) -> Result<()> {
        self.total_allocated = self.total_allocated
            .checked_add(amount)
            .ok_or(ErrorCode::ArithmeticOverflow)?;
        
        self.current_balance = self.current_balance
            .checked_add(amount)
            .ok_or(ErrorCode::ArithmeticOverflow)?;
        
        Ok(())
    }

    /// Calculate reward amount for a user
    pub fn calculate_reward(
        &self,
        base_amount: u64,
        staking_multiplier: u16,
        referral_multiplier: u16,
        quality_score: u16,
        activity_type: ActivityType,
    ) -> Result<u64> {
        let mut reward = base_amount;
        
        // Apply activity-specific multiplier
        let activity_multiplier = match activity_type {
            ActivityType::Mining => self.multipliers.mining_multiplier,
            ActivityType::Social => self.multipliers.social_multiplier,
            ActivityType::Referral => self.multipliers.referral_multiplier,
            ActivityType::Staking => self.multipliers.staking_multiplier,
            ActivityType::Content => self.multipliers.content_multiplier,
            ActivityType::Community => self.multipliers.community_multiplier,
        };
        
        reward = reward
            .checked_mul(activity_multiplier as u64)
            .ok_or(ErrorCode::ArithmeticOverflow)?
            .checked_div(10000)
            .ok_or(ErrorCode::ArithmeticOverflow)?;
        
        // Apply staking bonus
        if staking_multiplier > 10000 {
            reward = reward
                .checked_mul(staking_multiplier as u64)
                .ok_or(ErrorCode::ArithmeticOverflow)?
                .checked_div(10000)
                .ok_or(ErrorCode::ArithmeticOverflow)?;
        }
        
        // Apply referral bonus
        if referral_multiplier > 10000 {
            reward = reward
                .checked_mul(referral_multiplier as u64)
                .ok_or(ErrorCode::ArithmeticOverflow)?
                .checked_div(10000)
                .ok_or(ErrorCode::ArithmeticOverflow)?;
        }
        
        // Apply quality score impact
        reward = reward
            .checked_mul(quality_score as u64)
            .ok_or(ErrorCode::ArithmeticOverflow)?
            .checked_div(10000)
            .ok_or(ErrorCode::ArithmeticOverflow)?;
        
        // Apply phase multiplier
        if let Some(current_phase) = self.phases.get(self.current_phase as usize) {
            reward = reward
                .checked_mul(current_phase.phase_multiplier as u64)
                .ok_or(ErrorCode::ArithmeticOverflow)?
                .checked_div(10000)
                .ok_or(ErrorCode::ArithmeticOverflow)?;
        }
        
        // Ensure reward doesn't exceed maximum
        if reward > self.config.max_reward_per_distribution {
            reward = self.config.max_reward_per_distribution;
        }
        
        // Ensure reward meets minimum
        if reward < self.config.min_reward_per_distribution {
            reward = self.config.min_reward_per_distribution;
        }
        
        Ok(reward)
    }

    /// Distribute rewards to a user
    pub fn distribute_reward(&mut self, user: Pubkey, amount: u64) -> Result<()> {
        // Check if pool has sufficient balance
        require!(
            self.current_balance >= amount,
            ErrorCode::InsufficientPoolBalance
        );
        
        // Check if pool is not paused
        require!(
            !self.emergency.is_paused,
            ErrorCode::PoolPaused
        );
        
        // Update balances
        self.current_balance = self.current_balance
            .checked_sub(amount)
            .ok_or(ErrorCode::ArithmeticUnderflow)?;
        
        self.total_distributed = self.total_distributed
            .checked_add(amount)
            .ok_or(ErrorCode::ArithmeticOverflow)?;
        
        // Update statistics
        self.stats.total_transactions = self.stats.total_transactions
            .checked_add(1)
            .ok_or(ErrorCode::ArithmeticOverflow)?;
        
        if amount > self.stats.peak_distribution {
            self.stats.peak_distribution = amount;
            self.stats.peak_distribution_time = Clock::get()?.unix_timestamp;
        }
        
        // Update phase statistics
        if let Some(current_phase) = self.phases.get_mut(self.current_phase as usize) {
            current_phase.distributed_amount = current_phase.distributed_amount
                .checked_add(amount)
                .ok_or(ErrorCode::ArithmeticOverflow)?;
        }
        
        self.time_settings.last_distribution = Clock::get()?.unix_timestamp;
        
        Ok(())
    }

    /// Emergency pause the pool
    pub fn emergency_pause(&mut self, operator: Pubkey) -> Result<()> {
        // Check if operator is authorized
        require!(
            self.emergency.emergency_operators.contains(&operator) || 
            operator == self.authority,
            ErrorCode::UnauthorizedEmergencyAction
        );
        
        self.emergency.is_paused = true;
        self.emergency.pause_timestamp = Clock::get()?.unix_timestamp;
        
        // Log incident
        let incident = IncidentRecord {
            incident_id: self.emergency.incident_log.len() as u64,
            timestamp: Clock::get()?.unix_timestamp,
            incident_type: IncidentType::SystemFailure,
            severity: SeverityLevel::Critical,
            description: "Emergency pause activated".to_string(),
            resolution_status: ResolutionStatus::Open,
            impact: ImpactAssessment {
                affected_users: self.stats.unique_recipients,
                financial_impact: 0,
                downtime_duration: 0,
                reputation_impact: 8000, // High impact
                recovery_time_estimate: 3600, // 1 hour estimate
            },
        };
        
        self.emergency.incident_log.push(incident);
        
        Ok(())
    }

    /// Resume pool operations
    pub fn resume_operations(&mut self, operator: Pubkey) -> Result<()> {
        require!(
            operator == self.authority,
            ErrorCode::UnauthorizedOperation
        );
        
        self.emergency.is_paused = false;
        
        // Update recovery settings if gradual recovery is enabled
        if self.emergency.recovery_config.gradual_recovery_enabled {
            self.time_settings.next_distribution = Clock::get()?.unix_timestamp +
                self.emergency.recovery_config.recovery_phase_duration;
        }
        
        Ok(())
    }

    /// Check pool health and sustainability
    pub fn check_pool_health(&self) -> PoolHealthStatus {
        let utilization_rate = if self.total_allocated > 0 {
            ((self.total_distributed * 10000) / self.total_allocated) as u16
        } else {
            0
        };
        
        let balance_ratio = if self.total_allocated > 0 {
            ((self.current_balance * 10000) / self.total_allocated) as u16
        } else {
            0
        };
        
        if balance_ratio < 1000 { // Less than 10%
            PoolHealthStatus::Critical
        } else if balance_ratio < 2500 { // Less than 25%
            PoolHealthStatus::Low
        } else if utilization_rate > 8000 { // More than 80% utilized
            PoolHealthStatus::Medium
        } else {
            PoolHealthStatus::Healthy
        }
    }
}

/// Activity type enumeration for reward calculation
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy)]
pub enum ActivityType {
    Mining,
    Social,
    Referral,
    Staking,
    Content,
    Community,
}

/// Pool health status enumeration
#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq)]
pub enum PoolHealthStatus {
    Healthy,
    Medium,
    Low,
    Critical,
}

/// Custom error codes for reward pool operations
#[error_code]
pub enum ErrorCode {
    #[msg("Arithmetic overflow occurred")]
    ArithmeticOverflow,
    #[msg("Arithmetic underflow occurred")]
    ArithmeticUnderflow,
    #[msg("Insufficient pool balance")]
    InsufficientPoolBalance,
    #[msg("Pool is currently paused")]
    PoolPaused,
    #[msg("Unauthorized emergency action")]
    UnauthorizedEmergencyAction,
    #[msg("Unauthorized operation")]
    UnauthorizedOperation,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reward_calculation() {
        let pool = RewardPool::default();
        let result = pool.calculate_reward(
            1000,
            12000, // 120% staking multiplier
            11000, // 110% referral multiplier
            15000, // 150% quality score
            ActivityType::Mining,
        );
        
        // Expected: 1000 * 1.0 * 1.2 * 1.1 * 1.5 * 1.0 = 1980
        assert!(result.is_ok());
    }

    #[test]
    fn test_pool_health_check() {
        let mut pool = RewardPool::default();
        pool.total_allocated = 10000;
        pool.current_balance = 8000;
        pool.total_distributed = 2000;
        
        let health = pool.check_pool_health();
        assert_eq!(health, PoolHealthStatus::Healthy);
    }
}
