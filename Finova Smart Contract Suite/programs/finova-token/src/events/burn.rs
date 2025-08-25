// programs/finova-token/src/events/burn.rs

use anchor_lang::prelude::*;

/// Event emitted when tokens are burned
#[event]
pub struct TokenBurned {
    /// The mint address of the burned tokens
    pub mint: Pubkey,
    /// The authority that initiated the burn
    pub authority: Pubkey,
    /// The token account from which tokens were burned
    pub from: Pubkey,
    /// The amount of tokens burned (in smallest unit)
    pub amount: u64,
    /// The remaining supply after burn
    pub remaining_supply: u64,
    /// The reason for burning (encoded as u8)
    /// 0 = User initiated, 1 = Transaction fee, 2 = NFT usage, 3 = Whale tax, 4 = Anti-bot penalty
    pub burn_reason: u8,
    /// Unix timestamp when the burn occurred
    pub timestamp: i64,
    /// Block slot when the burn occurred
    pub slot: u64,
}

/// Event emitted when a batch burn operation is completed
#[event]
pub struct BatchTokenBurned {
    /// The mint address of the burned tokens
    pub mint: Pubkey,
    /// The authority that initiated the batch burn
    pub authority: Pubkey,
    /// The number of accounts involved in the batch burn
    pub account_count: u32,
    /// The total amount of tokens burned across all accounts
    pub total_amount: u64,
    /// The remaining supply after batch burn
    pub remaining_supply: u64,
    /// The reason for burning (encoded as u8)
    pub burn_reason: u8,
    /// Unix timestamp when the batch burn completed
    pub timestamp: i64,
    /// Block slot when the batch burn completed
    pub slot: u64,
}

/// Event emitted when automatic burn is triggered
#[event]
pub struct AutoBurnTriggered {
    /// The mint address of the tokens being auto-burned
    pub mint: Pubkey,
    /// The trigger condition that caused the auto burn
    /// 0 = Daily transaction volume threshold, 1 = Weekly whale tax, 2 = Monthly supply adjustment
    pub trigger_type: u8,
    /// The amount scheduled for auto burn
    pub scheduled_amount: u64,
    /// The current total supply before auto burn
    pub current_supply: u64,
    /// The target supply after auto burn
    pub target_supply: u64,
    /// Unix timestamp when auto burn was triggered
    pub timestamp: i64,
    /// The execution delay in seconds (when the burn will actually occur)
    pub execution_delay: i64,
}

/// Event emitted when burn rate is updated
#[event]
pub struct BurnRateUpdated {
    /// The mint address affected by the rate change
    pub mint: Pubkey,
    /// The authority that updated the burn rate
    pub authority: Pubkey,
    /// The previous burn rate (basis points, e.g., 100 = 1%)
    pub old_rate: u16,
    /// The new burn rate (basis points)
    pub new_rate: u16,
    /// The category of burn rate being updated
    /// 0 = Transaction fee, 1 = Whale tax, 2 = Anti-bot penalty, 3 = NFT usage
    pub rate_category: u8,
    /// Unix timestamp when the rate was updated
    pub timestamp: i64,
    /// Block slot when the rate was updated
    pub slot: u64,
}

/// Event emitted when deflationary mechanism is activated
#[event]
pub struct DeflationaryMechanismActivated {
    /// The mint address subject to deflationary pressure
    pub mint: Pubkey,
    /// The mechanism type activated
    /// 0 = Progressive burn increase, 1 = Supply cap reduction, 2 = Velocity-based burn
    pub mechanism_type: u8,
    /// The intensity of the mechanism (0-100, representing percentage)
    pub intensity: u8,
    /// The expected burn rate increase (basis points)
    pub burn_rate_increase: u16,
    /// The duration of the mechanism in seconds
    pub duration: i64,
    /// Current circulating supply
    pub circulating_supply: u64,
    /// Target supply reduction
    pub target_reduction: u64,
    /// Unix timestamp when mechanism was activated
    pub timestamp: i64,
}

/// Event emitted when burn fails due to insufficient balance
#[event]
pub struct BurnFailed {
    /// The mint address where burn failed
    pub mint: Pubkey,
    /// The authority that attempted the burn
    pub authority: Pubkey,
    /// The token account that had insufficient balance
    pub from: Pubkey,
    /// The amount attempted to burn
    pub attempted_amount: u64,
    /// The actual available balance
    pub available_balance: u64,
    /// The reason code for failure
    /// 0 = Insufficient balance, 1 = Account frozen, 2 = Mint frozen, 3 = Authority mismatch
    pub failure_reason: u8,
    /// Unix timestamp when burn failed
    pub timestamp: i64,
}

/// Event emitted when burn quota is exceeded
#[event]
pub struct BurnQuotaExceeded {
    /// The mint address where quota was exceeded
    pub mint: Pubkey,
    /// The authority that exceeded the quota
    pub authority: Pubkey,
    /// The attempted burn amount
    pub attempted_amount: u64,
    /// The remaining quota for the period
    pub remaining_quota: u64,
    /// The quota reset timestamp
    pub quota_reset_time: i64,
    /// The quota period type (0 = Daily, 1 = Weekly, 2 = Monthly)
    pub quota_period: u8,
    /// Unix timestamp when quota was exceeded
    pub timestamp: i64,
}

/// Event emitted when emergency burn is executed
#[event]
pub struct EmergencyBurnExecuted {
    /// The mint address of emergency burned tokens
    pub mint: Pubkey,
    /// The emergency authority that executed the burn
    pub emergency_authority: Pubkey,
    /// The reason for emergency burn (encoded string hash)
    pub reason_hash: [u8; 32],
    /// The amount of tokens emergency burned
    pub amount: u64,
    /// The accounts affected by emergency burn
    pub affected_accounts: u32,
    /// The governance proposal ID that authorized this burn (if applicable)
    pub governance_proposal_id: Option<u64>,
    /// Unix timestamp when emergency burn was executed
    pub timestamp: i64,
    /// Block slot when emergency burn was executed
    pub slot: u64,
}

/// Event emitted when burn statistics are updated
#[event]
pub struct BurnStatisticsUpdated {
    /// The mint address for which statistics are updated
    pub mint: Pubkey,
    /// Total tokens burned in the last 24 hours
    pub daily_burn: u64,
    /// Total tokens burned in the last 7 days
    pub weekly_burn: u64,
    /// Total tokens burned in the last 30 days
    pub monthly_burn: u64,
    /// Total tokens burned since inception
    pub total_burn: u64,
    /// Current effective burn rate (annualized percentage)
    pub effective_burn_rate: u16,
    /// The most significant burn reason in the period
    pub primary_burn_reason: u8,
    /// Unix timestamp of the statistics update
    pub timestamp: i64,
}

/// Event emitted when burn threshold is reached
#[event]
pub struct BurnThresholdReached {
    /// The mint address that reached the threshold
    pub mint: Pubkey,
    /// The threshold type that was reached
    /// 0 = Daily burn limit, 1 = Supply reduction target, 2 = Deflationary milestone
    pub threshold_type: u8,
    /// The threshold value that was reached
    pub threshold_value: u64,
    /// The actual value that triggered the threshold
    pub trigger_value: u64,
    /// The action taken as a result
    /// 0 = Rate adjustment, 1 = Mechanism activation, 2 = Alert only, 3 = Emergency stop
    pub action_taken: u8,
    /// Additional data related to the action (context-dependent)
    pub action_data: u64,
    /// Unix timestamp when threshold was reached
    pub timestamp: i64,
}

/// Event emitted when burn mechanism is paused or resumed
#[event]
pub struct BurnMechanismStatusChanged {
    /// The mint address affected by the status change
    pub mint: Pubkey,
    /// The authority that changed the status
    pub authority: Pubkey,
    /// The previous status (true = active, false = paused)
    pub previous_status: bool,
    /// The new status (true = active, false = paused)
    pub new_status: bool,
    /// The reason for status change (encoded as u8)
    /// 0 = Manual admin action, 1 = Emergency pause, 2 = Governance decision, 3 = Automatic trigger
    pub reason: u8,
    /// The duration of the status change (0 = permanent, >0 = temporary in seconds)
    pub duration: i64,
    /// Unix timestamp when status changed
    pub timestamp: i64,
}

/// Event emitted for burn transaction fee calculation
#[event]
pub struct BurnFeeCalculated {
    /// The mint address where fee was calculated
    pub mint: Pubkey,
    /// The original transaction amount
    pub transaction_amount: u64,
    /// The calculated burn fee amount
    pub burn_fee: u64,
    /// The fee rate used for calculation (basis points)
    pub fee_rate: u16,
    /// The transaction type that triggered the fee
    /// 0 = Transfer, 1 = Swap, 2 = Staking, 3 = NFT purchase, 4 = DeFi interaction
    pub transaction_type: u8,
    /// The account that will pay the burn fee
    pub fee_payer: Pubkey,
    /// Unix timestamp when fee was calculated
    pub timestamp: i64,
}

/// Event emitted when burn schedule is created or updated
#[event]
pub struct BurnScheduleUpdated {
    /// The mint address for the burn schedule
    pub mint: Pubkey,
    /// The authority that updated the schedule
    pub authority: Pubkey,
    /// The schedule ID (for tracking multiple schedules)
    pub schedule_id: u32,
    /// The frequency of scheduled burns (in seconds)
    pub frequency: i64,
    /// The amount to burn per schedule execution
    pub amount_per_burn: u64,
    /// The start time of the schedule
    pub start_time: i64,
    /// The end time of the schedule (0 = indefinite)
    pub end_time: i64,
    /// Whether the schedule is active
    pub is_active: bool,
    /// Unix timestamp when schedule was updated
    pub timestamp: i64,
}

/// Event emitted when cross-program burn is initiated
#[event]
pub struct CrossProgramBurnInitiated {
    /// The mint address of tokens being burned
    pub mint: Pubkey,
    /// The calling program that initiated the burn
    pub calling_program: Pubkey,
    /// The cross-program invocation authority
    pub cpi_authority: Pubkey,
    /// The amount being burned via CPI
    pub amount: u64,
    /// The source account for the burn
    pub source_account: Pubkey,
    /// The burn reason provided by calling program
    pub external_reason: u8,
    /// Additional context data from calling program
    pub context_data: [u8; 32],
    /// Unix timestamp when CPI burn was initiated
    pub timestamp: i64,
}

impl TokenBurned {
    /// Create a new TokenBurned event
    pub fn new(
        mint: Pubkey,
        authority: Pubkey,
        from: Pubkey,
        amount: u64,
        remaining_supply: u64,
        burn_reason: u8,
    ) -> Self {
        let clock = Clock::get().unwrap();
        Self {
            mint,
            authority,
            from,
            amount,
            remaining_supply,
            burn_reason,
            timestamp: clock.unix_timestamp,
            slot: clock.slot,
        }
    }

    /// Get human-readable burn reason
    pub fn burn_reason_str(&self) -> &'static str {
        match self.burn_reason {
            0 => "User Initiated",
            1 => "Transaction Fee",
            2 => "NFT Usage",
            3 => "Whale Tax",
            4 => "Anti-bot Penalty",
            _ => "Unknown",
        }
    }

    /// Calculate burn percentage of total supply before burn
    pub fn burn_percentage(&self) -> f64 {
        let total_before_burn = self.remaining_supply + self.amount;
        if total_before_burn == 0 {
            0.0
        } else {
            (self.amount as f64 / total_before_burn as f64) * 100.0
        }
    }
}

impl BatchTokenBurned {
    /// Create a new BatchTokenBurned event
    pub fn new(
        mint: Pubkey,
        authority: Pubkey,
        account_count: u32,
        total_amount: u64,
        remaining_supply: u64,
        burn_reason: u8,
    ) -> Self {
        let clock = Clock::get().unwrap();
        Self {
            mint,
            authority,
            account_count,
            total_amount,
            remaining_supply,
            burn_reason,
            timestamp: clock.unix_timestamp,
            slot: clock.slot,
        }
    }

    /// Calculate average burn per account
    pub fn average_burn_per_account(&self) -> u64 {
        if self.account_count == 0 {
            0
        } else {
            self.total_amount / self.account_count as u64
        }
    }
}

impl AutoBurnTriggered {
    /// Create a new AutoBurnTriggered event
    pub fn new(
        mint: Pubkey,
        trigger_type: u8,
        scheduled_amount: u64,
        current_supply: u64,
        target_supply: u64,
        execution_delay: i64,
    ) -> Self {
        let clock = Clock::get().unwrap();
        Self {
            mint,
            trigger_type,
            scheduled_amount,
            current_supply,
            target_supply,
            timestamp: clock.unix_timestamp,
            execution_delay,
        }
    }

    /// Get the execution timestamp
    pub fn execution_time(&self) -> i64 {
        self.timestamp + self.execution_delay
    }

    /// Calculate the supply reduction percentage
    pub fn reduction_percentage(&self) -> f64 {
        if self.current_supply == 0 {
            0.0
        } else {
            (self.scheduled_amount as f64 / self.current_supply as f64) * 100.0
        }
    }
}

impl BurnRateUpdated {
    /// Create a new BurnRateUpdated event
    pub fn new(
        mint: Pubkey,
        authority: Pubkey,
        old_rate: u16,
        new_rate: u16,
        rate_category: u8,
    ) -> Self {
        let clock = Clock::get().unwrap();
        Self {
            mint,
            authority,
            old_rate,
            new_rate,
            rate_category,
            timestamp: clock.unix_timestamp,
            slot: clock.slot,
        }
    }

    /// Calculate the rate change percentage
    pub fn rate_change_percentage(&self) -> f64 {
        if self.old_rate == 0 {
            if self.new_rate == 0 { 0.0 } else { 100.0 }
        } else {
            ((self.new_rate as f64 - self.old_rate as f64) / self.old_rate as f64) * 100.0
        }
    }

    /// Get human-readable rate category
    pub fn rate_category_str(&self) -> &'static str {
        match self.rate_category {
            0 => "Transaction Fee",
            1 => "Whale Tax",
            2 => "Anti-bot Penalty",
            3 => "NFT Usage",
            _ => "Unknown",
        }
    }
}

impl DeflationaryMechanismActivated {
    /// Create a new DeflationaryMechanismActivated event
    pub fn new(
        mint: Pubkey,
        mechanism_type: u8,
        intensity: u8,
        burn_rate_increase: u16,
        duration: i64,
        circulating_supply: u64,
        target_reduction: u64,
    ) -> Self {
        let clock = Clock::get().unwrap();
        Self {
            mint,
            mechanism_type,
            intensity,
            burn_rate_increase,
            duration,
            circulating_supply,
            target_reduction,
            timestamp: clock.unix_timestamp,
        }
    }

    /// Get the mechanism end time
    pub fn end_time(&self) -> i64 {
        self.timestamp + self.duration
    }

    /// Calculate target reduction percentage
    pub fn target_reduction_percentage(&self) -> f64 {
        if self.circulating_supply == 0 {
            0.0
        } else {
            (self.target_reduction as f64 / self.circulating_supply as f64) * 100.0
        }
    }
}

impl EmergencyBurnExecuted {
    /// Create a new EmergencyBurnExecuted event
    pub fn new(
        mint: Pubkey,
        emergency_authority: Pubkey,
        reason_hash: [u8; 32],
        amount: u64,
        affected_accounts: u32,
        governance_proposal_id: Option<u64>,
    ) -> Self {
        let clock = Clock::get().unwrap();
        Self {
            mint,
            emergency_authority,
            reason_hash,
            amount,
            affected_accounts,
            governance_proposal_id,
            timestamp: clock.unix_timestamp,
            slot: clock.slot,
        }
    }

    /// Calculate average burn per affected account
    pub fn average_burn_per_account(&self) -> u64 {
        if self.affected_accounts == 0 {
            0
        } else {
            self.amount / self.affected_accounts as u64
        }
    }
}

// Helper functions for burn event analysis

/// Calculate total burn impact across multiple events
pub fn calculate_total_burn_impact(events: &[TokenBurned]) -> u64 {
    events.iter().map(|e| e.amount).sum()
}

/// Find the most common burn reason in a set of events
pub fn find_primary_burn_reason(events: &[TokenBurned]) -> u8 {
    let mut reason_counts = [0u32; 8]; // Support up to 8 burn reasons
    
    for event in events {
        if (event.burn_reason as usize) < reason_counts.len() {
            reason_counts[event.burn_reason as usize] += 1;
        }
    }
    
    reason_counts
        .iter()
        .enumerate()
        .max_by_key(|(_, &count)| count)
        .map(|(reason, _)| reason as u8)
        .unwrap_or(0)
}

/// Calculate burn velocity (burns per unit time)
pub fn calculate_burn_velocity(events: &[TokenBurned], time_window: i64) -> f64 {
    if events.is_empty() || time_window == 0 {
        return 0.0;
    }
    
    let total_amount: u64 = events.iter().map(|e| e.amount).sum();
    total_amount as f64 / time_window as f64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_burned_creation() {
        let mint = Pubkey::new_unique();
        let authority = Pubkey::new_unique();
        let from = Pubkey::new_unique();
        
        let event = TokenBurned::new(mint, authority, from, 1000, 999000, 1);
        
        assert_eq!(event.mint, mint);
        assert_eq!(event.authority, authority);
        assert_eq!(event.from, from);
        assert_eq!(event.amount, 1000);
        assert_eq!(event.remaining_supply, 999000);
        assert_eq!(event.burn_reason, 1);
    }

    #[test]
    fn test_burn_percentage_calculation() {
        let event = TokenBurned {
            mint: Pubkey::new_unique(),
            authority: Pubkey::new_unique(),
            from: Pubkey::new_unique(),
            amount: 1000,
            remaining_supply: 99000,
            burn_reason: 0,
            timestamp: 0,
            slot: 0,
        };
        
        let percentage = event.burn_percentage();
        assert!((percentage - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_burn_reason_string() {
        let event = TokenBurned {
            mint: Pubkey::new_unique(),
            authority: Pubkey::new_unique(),
            from: Pubkey::new_unique(),
            amount: 1000,
            remaining_supply: 99000,
            burn_reason: 2,
            timestamp: 0,
            slot: 0,
        };
        
        assert_eq!(event.burn_reason_str(), "NFT Usage");
    }

    #[test]
    fn test_batch_burn_average_calculation() {
        let event = BatchTokenBurned::new(
            Pubkey::new_unique(),
            Pubkey::new_unique(),
            10,
            1000,
            99000,
            0,
        );
        
        assert_eq!(event.average_burn_per_account(), 100);
    }

    #[test]
    fn test_rate_change_percentage() {
        let event = BurnRateUpdated::new(
            Pubkey::new_unique(),
            Pubkey::new_unique(),
            100,
            150,
            0,
        );
        
        let change = event.rate_change_percentage();
        assert!((change - 50.0).abs() < f64::EPSILON);
    }
}
