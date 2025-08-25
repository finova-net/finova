// programs/finova-token/src/instructions/mod.rs

use anchor_lang::prelude::*;

pub mod initialize_mint;
pub mod mint_tokens;
pub mod burn_tokens;
pub mod stake_tokens;
pub mod unstake_tokens;
pub mod claim_rewards;

pub use initialize_mint::*;
pub use mint_tokens::*;
pub use burn_tokens::*;
pub use stake_tokens::*;
pub use unstake_tokens::*;
pub use claim_rewards::*;

/// Comprehensive instruction handler for all token operations
/// Integrates with Finova's XP, RP, and mining systems for unified rewards
#[derive(Accounts)]
pub struct TokenInstructionContext<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    
    #[account(mut)]
    pub user: SystemAccount<'info>,
    
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub rent: Sysvar<'info, Rent>,
}

/// Enhanced token operations with integrated reward calculations
pub struct TokenOperations;

impl TokenOperations {
    /// Calculate integrated mining rewards based on token operations
    /// Formula: Base_Reward × Token_Amount_Factor × Activity_Multiplier × Quality_Score
    pub fn calculate_mining_boost(
        token_amount: u64,
        operation_type: TokenOperationType,
        user_xp_level: u32,
        user_rp_tier: u8,
        quality_score: f64,
    ) -> Result<u64> {
        let base_reward = match operation_type {
            TokenOperationType::Stake => 100, // Base mining boost for staking
            TokenOperationType::Mint => 50,   // Base mining boost for earning tokens
            TokenOperationType::Burn => 200,  // Higher boost for deflationary actions
            TokenOperationType::Transfer => 25, // Minimal boost for transfers
        };

        // Token amount factor with logarithmic scaling to prevent whale dominance
        let amount_factor = if token_amount > 0 {
            ((token_amount as f64).ln() / 10.0).min(5.0).max(1.0)
        } else {
            1.0
        };

        // XP level multiplier (1.0x to 5.0x based on level)
        let xp_multiplier = 1.0 + (user_xp_level as f64 / 100.0).min(4.0);

        // RP tier multiplier (1.0x to 3.0x based on tier)
        let rp_multiplier = 1.0 + (user_rp_tier as f64 * 0.2).min(2.0);

        // Quality score (0.5x to 2.0x)
        let quality_multiplier = quality_score.max(0.5).min(2.0);

        let final_reward = (base_reward as f64 
            * amount_factor 
            * xp_multiplier 
            * rp_multiplier 
            * quality_multiplier) as u64;

        Ok(final_reward)
    }

    /// Calculate staking rewards with compound interest and loyalty bonuses
    /// Enhanced formula integrating XP and RP systems
    pub fn calculate_staking_rewards(
        staked_amount: u64,
        staking_duration_days: u64,
        base_apy: f64,
        user_xp_level: u32,
        user_rp_tier: u8,
        loyalty_multiplier: f64,
        activity_score: f64,
    ) -> Result<u64> {
        // Base APY calculation (8-15% based on tier)
        let enhanced_apy = base_apy 
            + (user_xp_level as f64 / 1000.0) // +0.1% per 10 XP levels
            + (user_rp_tier as f64 * 0.005); // +0.5% per RP tier

        // Loyalty bonus (up to +5% APY for long-term stakers)
        let loyalty_bonus = (staking_duration_days as f64 / 365.0 * 0.05).min(0.05);

        // Activity bonus (up to +2% APY for active users)
        let activity_bonus = (activity_score * 0.02).min(0.02);

        // Total effective APY
        let total_apy = enhanced_apy + loyalty_bonus + activity_bonus;

        // Daily reward calculation with compound interest
        let daily_rate = total_apy / 365.0;
        let compound_factor = (1.0 + daily_rate).powf(staking_duration_days as f64);
        
        let total_rewards = (staked_amount as f64 * (compound_factor - 1.0)) as u64;

        // Apply loyalty multiplier for additional bonuses
        let final_rewards = (total_rewards as f64 * loyalty_multiplier) as u64;

        Ok(final_rewards)
    }

    /// Calculate token burn benefits with deflationary incentives
    /// Rewards users for contributing to token scarcity
    pub fn calculate_burn_incentives(
        burn_amount: u64,
        total_supply: u64,
        user_holdings: u64,
        burn_pool_balance: u64,
    ) -> Result<(u64, u64)> {
        // Burn ratio relative to total supply
        let burn_ratio = burn_amount as f64 / total_supply as f64;

        // Base incentive (1% of burned amount returned as bonus)
        let base_incentive = burn_amount / 100;

        // Scarcity bonus (higher rewards for larger burns relative to supply)
        let scarcity_multiplier = if burn_ratio > 0.001 { // > 0.1% of supply
            2.0
        } else if burn_ratio > 0.0001 { // > 0.01% of supply
            1.5
        } else {
            1.0
        };

        // Holder loyalty bonus (rewards for burning portion of holdings)
        let holder_ratio = burn_amount as f64 / user_holdings as f64;
        let loyalty_bonus = if holder_ratio > 0.1 { // Burning >10% of holdings
            burn_amount / 20 // 5% bonus
        } else if holder_ratio > 0.05 { // Burning >5% of holdings
            burn_amount / 50 // 2% bonus
        } else {
            0
        };

        // Pool distribution (rewards from burn incentive pool)
        let pool_reward = if burn_pool_balance > 0 {
            let pool_ratio = burn_amount as f64 / total_supply as f64;
            ((burn_pool_balance as f64 * pool_ratio).min(burn_pool_balance as f64 / 100.0)) as u64
        } else {
            0
        };

        let total_incentive = ((base_incentive as f64 * scarcity_multiplier) as u64)
            .saturating_add(loyalty_bonus)
            .saturating_add(pool_reward);

        // Return (incentive_amount, mining_boost_duration_hours)
        let mining_boost_hours = (burn_ratio * 1000000.0) as u64; // Scale boost duration
        Ok((total_incentive, mining_boost_hours.min(168))) // Max 1 week boost
    }

    /// Validate token operation against anti-bot measures
    /// Implements comprehensive fraud detection for token transactions
    pub fn validate_token_operation(
        user_pubkey: &Pubkey,
        operation_type: TokenOperationType,
        amount: u64,
        user_activity_score: f64,
        transaction_frequency: u32,
        device_fingerprint: Option<String>,
        network_quality_score: Option<f64>,
    ) -> Result<ValidationResult> {
        let mut risk_score = 0.0;
        let mut validation_flags = Vec::new();

        // Amount validation (detect suspiciously round numbers)
        if amount % 1000000 == 0 && amount > 1000000 { // Exact millions
            risk_score += 0.2;
            validation_flags.push("round_amount".to_string());
        }

        // Transaction frequency analysis (detect bot-like patterns)
        if transaction_frequency > 100 { // >100 transactions per hour
            risk_score += 0.4;
            validation_flags.push("high_frequency".to_string());
        } else if transaction_frequency > 50 {
            risk_score += 0.2;
            validation_flags.push("moderate_frequency".to_string());
        }

        // Activity score validation (low activity suggests bot)
        if user_activity_score < 0.3 {
            risk_score += 0.3;
            validation_flags.push("low_activity".to_string());
        }

        // Device fingerprint analysis
        if let Some(fingerprint) = device_fingerprint {
            if fingerprint.len() < 20 { // Suspiciously short fingerprint
                risk_score += 0.2;
                validation_flags.push("weak_fingerprint".to_string());
            }
        } else {
            risk_score += 0.1;
            validation_flags.push("no_fingerprint".to_string());
        }

        // Network quality analysis (referral network quality)
        if let Some(network_quality) = network_quality_score {
            if network_quality < 0.4 {
                risk_score += 0.2;
                validation_flags.push("poor_network_quality".to_string());
            }
        }

        // Operation-specific validations
        match operation_type {
            TokenOperationType::Stake => {
                // Staking operations should have lower risk tolerance
                if amount > 1000000000 { // >1B tokens (whale detection)
                    risk_score += 0.3;
                    validation_flags.push("whale_staking".to_string());
                }
            },
            TokenOperationType::Burn => {
                // Burn operations are generally positive for ecosystem
                risk_score *= 0.8; // Reduce risk score for burns
            },
            TokenOperationType::Transfer => {
                // Transfers to new addresses increase risk
                risk_score += 0.1;
                validation_flags.push("transfer_operation".to_string());
            },
            _ => {}
        }

        // Determine validation result
        let result = if risk_score >= 0.8 {
            ValidationResult::Rejected
        } else if risk_score >= 0.5 {
            ValidationResult::RequiresManualReview
        } else if risk_score >= 0.3 {
            ValidationResult::RequiresAdditionalVerification
        } else {
            ValidationResult::Approved
        };

        msg!("Token operation validation - Risk Score: {}, Flags: {:?}", risk_score, validation_flags);

        Ok(result)
    }

    /// Calculate gas optimization for batch operations
    /// Reduces transaction costs for multiple token operations
    pub fn optimize_batch_operations(
        operations: &[BatchTokenOperation],
        max_batch_size: usize,
    ) -> Result<Vec<Vec<BatchTokenOperation>>> {
        let mut optimized_batches = Vec::new();
        let mut current_batch = Vec::new();
        let mut current_compute_units = 0u32;

        const MAX_COMPUTE_UNITS: u32 = 1400000; // Solana limit minus buffer
        const BASE_OPERATION_COST: u32 = 10000;

        for operation in operations {
            let operation_cost = match operation.operation_type {
                TokenOperationType::Mint => BASE_OPERATION_COST,
                TokenOperationType::Burn => BASE_OPERATION_COST * 2, // More expensive
                TokenOperationType::Stake => BASE_OPERATION_COST * 3, // Most expensive
                TokenOperationType::Transfer => BASE_OPERATION_COST / 2, // Cheapest
            };

            // Check if adding this operation exceeds limits
            if current_batch.len() >= max_batch_size 
                || current_compute_units + operation_cost > MAX_COMPUTE_UNITS {
                
                if !current_batch.is_empty() {
                    optimized_batches.push(current_batch.clone());
                    current_batch.clear();
                    current_compute_units = 0;
                }
            }

            current_batch.push(operation.clone());
            current_compute_units += operation_cost;
        }

        // Add remaining operations
        if !current_batch.is_empty() {
            optimized_batches.push(current_batch);
        }

        Ok(optimized_batches)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum TokenOperationType {
    Mint,
    Burn,
    Stake,
    Transfer,
}

#[derive(Clone, Debug)]
pub struct BatchTokenOperation {
    pub user: Pubkey,
    pub operation_type: TokenOperationType,
    pub amount: u64,
    pub metadata: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ValidationResult {
    Approved,
    RequiresAdditionalVerification,
    RequiresManualReview,
    Rejected,
}

/// Enhanced error handling for token operations
#[derive(Debug)]
pub enum TokenOperationError {
    InsufficientFunds,
    InvalidAmount,
    UnauthorizedOperation,
    ExcessiveTransactionFrequency,
    SuspiciousActivity,
    ValidationFailed,
    ComputeBudgetExceeded,
    NetworkQualityTooLow,
    DeviceFingerprintMismatch,
    BotDetected,
}

impl From<TokenOperationError> for ProgramError {
    fn from(error: TokenOperationError) -> Self {
        match error {
            TokenOperationError::InsufficientFunds => ProgramError::InsufficientFunds,
            TokenOperationError::InvalidAmount => ProgramError::InvalidArgument,
            TokenOperationError::UnauthorizedOperation => ProgramError::MissingRequiredSignature,
            _ => ProgramError::Custom(9000 + error as u32),
        }
    }
}

/// Token metrics tracking for analytics and optimization
#[derive(Clone, Debug, Default)]
pub struct TokenMetrics {
    pub total_minted: u64,
    pub total_burned: u64,
    pub total_staked: u64,
    pub total_transfers: u64,
    pub unique_holders: u32,
    pub average_holding_time: u64,
    pub burn_incentives_distributed: u64,
    pub staking_rewards_distributed: u64,
    pub mining_boosts_granted: u64,
    pub bot_operations_blocked: u32,
    pub validation_failures: u32,
    pub gas_optimized_operations: u64,
}

impl TokenMetrics {
    pub fn update_operation_metrics(
        &mut self,
        operation_type: TokenOperationType,
        amount: u64,
        validation_result: ValidationResult,
    ) {
        match operation_type {
            TokenOperationType::Mint => self.total_minted = self.total_minted.saturating_add(amount),
            TokenOperationType::Burn => self.total_burned = self.total_burned.saturating_add(amount),
            TokenOperationType::Stake => self.total_staked = self.total_staked.saturating_add(amount),
            TokenOperationType::Transfer => self.total_transfers = self.total_transfers.saturating_add(1),
        }

        match validation_result {
            ValidationResult::Rejected => self.bot_operations_blocked = self.bot_operations_blocked.saturating_add(1),
            ValidationResult::RequiresManualReview | 
            ValidationResult::RequiresAdditionalVerification => {
                self.validation_failures = self.validation_failures.saturating_add(1)
            },
            _ => {}
        }
    }

    pub fn calculate_token_health_score(&self) -> f64 {
        let burn_ratio = if self.total_minted > 0 {
            self.total_burned as f64 / self.total_minted as f64
        } else {
            0.0
        };

        let stake_ratio = if self.total_minted > 0 {
            self.total_staked as f64 / self.total_minted as f64
        } else {
            0.0
        };

        let security_score = if self.total_transfers > 0 {
            1.0 - (self.bot_operations_blocked as f64 / self.total_transfers as f64)
        } else {
            1.0
        };

        // Health score formula: weighted average of key metrics
        let health_score = (burn_ratio * 0.3) // Deflationary pressure
            + (stake_ratio * 0.4) // Token lock-up
            + (security_score * 0.3); // Security effectiveness

        health_score.min(1.0).max(0.0)
    }
}

/// Integration utilities for connecting with Finova Core systems
pub struct CoreIntegration;

impl CoreIntegration {
    /// Sync token operations with user XP system
    pub fn sync_with_xp_system(
        user_pubkey: &Pubkey,
        operation_type: TokenOperationType,
        amount: u64,
    ) -> Result<u32> {
        let xp_gain = match operation_type {
            TokenOperationType::Stake => (amount / 1000).min(100), // 1 XP per 1000 tokens staked, max 100
            TokenOperationType::Burn => (amount / 500).min(200),   // 1 XP per 500 tokens burned, max 200
            TokenOperationType::Mint => (amount / 2000).min(50),   // 1 XP per 2000 tokens earned, max 50
            TokenOperationType::Transfer => 5, // Fixed 5 XP for transfers
        } as u32;

        msg!("XP gained from token operation: {} XP for user: {}", xp_gain, user_pubkey);
        Ok(xp_gain)
    }

    /// Sync token operations with referral point system
    pub fn sync_with_rp_system(
        user_pubkey: &Pubkey,
        operation_type: TokenOperationType,
        amount: u64,
        referral_network_size: u32,
    ) -> Result<u32> {
        if referral_network_size == 0 {
            return Ok(0);
        }

        let rp_gain = match operation_type {
            TokenOperationType::Stake => {
                // Referrers get 5% of staker's RP value
                let base_rp = (amount / 10000).min(50) as u32; // 1 RP per 10k tokens
                (base_rp * referral_network_size / 20).min(100) // Distributed across network
            },
            TokenOperationType::Burn => {
                // Higher RP rewards for burns (deflationary benefit)
                let base_rp = (amount / 5000).min(100) as u32;
                (base_rp * referral_network_size / 15).min(150)
            },
            _ => 0,
        };

        msg!("RP gained from token operation: {} RP distributed to network of user: {}", rp_gain, user_pubkey);
        Ok(rp_gain)
    }

    /// Sync token operations with mining rate adjustments
    pub fn sync_with_mining_system(
        user_pubkey: &Pubkey,
        operation_type: TokenOperationType,
        amount: u64,
        current_mining_rate: f64,
    ) -> Result<f64> {
        let mining_boost = match operation_type {
            TokenOperationType::Stake => {
                // Staking boosts mining rate by up to 100%
                let boost_factor = (amount as f64 / 1000000.0).min(1.0); // Max boost at 1M tokens
                current_mining_rate * (1.0 + boost_factor)
            },
            TokenOperationType::Burn => {
                // Burning provides temporary 200% mining boost
                current_mining_rate * 3.0 // 3x mining rate temporarily
            },
            _ => current_mining_rate,
        };

        msg!("Mining rate adjusted from {} to {} for user: {}", current_mining_rate, mining_boost, user_pubkey);
        Ok(mining_boost)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mining_boost_calculation() {
        let boost = TokenOperations::calculate_mining_boost(
            1000000, // 1M tokens
            TokenOperationType::Stake,
            50, // Level 50 user
            3,  // RP tier 3
            1.5, // Good quality score
        ).unwrap();

        assert!(boost > 100); // Should be more than base reward
        assert!(boost < 10000); // Should be reasonable
    }

    #[test]
    fn test_staking_rewards_calculation() {
        let rewards = TokenOperations::calculate_staking_rewards(
            1000000, // 1M tokens staked
            365,     // 1 year
            0.12,    // 12% base APY
            75,      // High XP level
            4,       // High RP tier
            1.2,     // 20% loyalty multiplier
            0.8,     // High activity score
        ).unwrap();

        assert!(rewards > 120000); // Should be more than 12% base
        assert!(rewards < 200000); // Should be reasonable with bonuses
    }

    #[test]
    fn test_burn_incentives() {
        let (incentive, boost_hours) = TokenOperations::calculate_burn_incentives(
            100000,   // 100k tokens burned
            10000000, // 10M total supply
            500000,   // User has 500k tokens
            1000000,  // 1M in burn pool
        ).unwrap();

        assert!(incentive > 1000); // Should get some incentive
        assert!(boost_hours > 0);  // Should get mining boost
        assert!(boost_hours <= 168); // Max 1 week boost
    }

    #[test]
    fn test_operation_validation() {
        let result = TokenOperations::validate_token_operation(
            &Pubkey::new_unique(),
            TokenOperationType::Stake,
            1000000,
            0.8, // Good activity score
            10,  // Normal frequency
            Some("valid_fingerprint_string".to_string()),
            Some(0.7), // Good network quality
        ).unwrap();

        assert_eq!(result, ValidationResult::Approved);
    }

    #[test]
    fn test_bot_detection() {
        let result = TokenOperations::validate_token_operation(
            &Pubkey::new_unique(),
            TokenOperationType::Transfer,
            1000000000, // Exact billion (suspicious)
            0.1, // Low activity score
            150, // High frequency
            None, // No fingerprint
            Some(0.2), // Poor network quality
        ).unwrap();

        assert!(result != ValidationResult::Approved);
    }

    #[test]
    fn test_batch_optimization() {
        let operations = vec![
            BatchTokenOperation {
                user: Pubkey::new_unique(),
                operation_type: TokenOperationType::Mint,
                amount: 1000,
                metadata: None,
            };
            100 // 100 operations
        ];

        let batches = TokenOperations::optimize_batch_operations(&operations, 50).unwrap();
        assert!(batches.len() >= 2); // Should split into multiple batches
        assert!(batches.iter().all(|batch| batch.len() <= 50)); // Respect batch size limit
    }

    #[test]
    fn test_token_health_score() {
        let mut metrics = TokenMetrics::default();
        metrics.total_minted = 1000000;
        metrics.total_burned = 100000; // 10% burned
        metrics.total_staked = 500000;  // 50% staked
        metrics.total_transfers = 1000;
        metrics.bot_operations_blocked = 50; // 5% bot operations

        let health_score = metrics.calculate_token_health_score();
        assert!(health_score > 0.7); // Should be healthy
        assert!(health_score <= 1.0);
    }

    #[test]
    fn test_core_integration_xp_sync() {
        let xp_gain = CoreIntegration::sync_with_xp_system(
            &Pubkey::new_unique(),
            TokenOperationType::Stake,
            50000, // 50k tokens
        ).unwrap();

        assert_eq!(xp_gain, 50); // 50k / 1000 = 50 XP
    }

    #[test]
    fn test_core_integration_mining_sync() {
        let new_rate = CoreIntegration::sync_with_mining_system(
            &Pubkey::new_unique(),
            TokenOperationType::Burn,
            100000, // 100k tokens burned
            0.05,   // Current rate
        ).unwrap();

        assert_eq!(new_rate, 0.15); // Should be 3x boost (0.05 * 3)
    }
}
