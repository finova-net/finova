// programs/finova-bridge/src/utils.rs

use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount, Transfer};
use solana_program::{
    hash::{hash, Hash},
    keccak::{hashv, Hasher},
    pubkey::Pubkey,
    system_instruction,
};
use std::collections::HashMap;

use crate::{constants::*, errors::FinovaBridgeError};

/// Bridge utility functions for cross-chain operations
pub struct BridgeUtils;

impl BridgeUtils {
    /// Generate bridge transaction hash
    pub fn generate_bridge_hash(
        source_chain: u16,
        dest_chain: u16,
        token_mint: &Pubkey,
        amount: u64,
        recipient: &Pubkey,
        nonce: u64,
    ) -> Result<[u8; 32]> {
        let mut hasher = Hasher::default();
        
        hasher.hash(&source_chain.to_le_bytes());
        hasher.hash(&dest_chain.to_le_bytes());
        hasher.hash(&token_mint.to_bytes());
        hasher.hash(&amount.to_le_bytes());
        hasher.hash(&recipient.to_bytes());
        hasher.hash(&nonce.to_le_bytes());
        
        Ok(hasher.result().to_bytes())
    }

    /// Validate bridge transaction parameters
    pub fn validate_bridge_params(
        source_chain: u16,
        dest_chain: u16,
        amount: u64,
    ) -> Result<()> {
        // Validate chain IDs
        if !Self::is_supported_chain(source_chain) {
            return Err(FinovaBridgeError::UnsupportedChain.into());
        }
        
        if !Self::is_supported_chain(dest_chain) {
            return Err(FinovaBridgeError::UnsupportedChain.into());
        }
        
        if source_chain == dest_chain {
            return Err(FinovaBridgeError::SameChainBridge.into());
        }
        
        // Validate amount
        if amount == 0 {
            return Err(FinovaBridgeError::ZeroAmount.into());
        }
        
        if amount < MIN_BRIDGE_AMOUNT {
            return Err(FinovaBridgeError::AmountTooSmall.into());
        }
        
        if amount > MAX_BRIDGE_AMOUNT {
            return Err(FinovaBridgeError::AmountTooLarge.into());
        }
        
        Ok(())
    }

    /// Check if chain ID is supported
    pub fn is_supported_chain(chain_id: u16) -> bool {
        matches!(
            chain_id,
            SOLANA_CHAIN_ID
                | ETHEREUM_CHAIN_ID
                | BSC_CHAIN_ID
                | POLYGON_CHAIN_ID
                | AVALANCHE_CHAIN_ID
                | ARBITRUM_CHAIN_ID
                | OPTIMISM_CHAIN_ID
        )
    }

    /// Calculate bridge fee based on amount and destination chain
    pub fn calculate_bridge_fee(amount: u64, dest_chain: u16) -> Result<u64> {
        let base_fee = match dest_chain {
            ETHEREUM_CHAIN_ID => BASE_ETHEREUM_FEE,
            BSC_CHAIN_ID => BASE_BSC_FEE,
            POLYGON_CHAIN_ID => BASE_POLYGON_FEE,
            AVALANCHE_CHAIN_ID => BASE_AVALANCHE_FEE,
            ARBITRUM_CHAIN_ID => BASE_ARBITRUM_FEE,
            OPTIMISM_CHAIN_ID => BASE_OPTIMISM_FEE,
            _ => return Err(FinovaBridgeError::UnsupportedChain.into()),
        };

        // Calculate percentage fee
        let percentage_fee = amount
            .checked_mul(BRIDGE_FEE_BPS as u64)
            .ok_or(FinovaBridgeError::MathOverflow)?
            .checked_div(10000)
            .ok_or(FinovaBridgeError::MathOverflow)?;

        // Return maximum of base fee and percentage fee
        Ok(std::cmp::max(base_fee, percentage_fee))
    }

    /// Verify merkle proof for cross-chain validation
    pub fn verify_merkle_proof(
        leaf: &[u8; 32],
        proof: &[[u8; 32]],
        root: &[u8; 32],
    ) -> bool {
        let mut computed_hash = *leaf;
        
        for proof_element in proof {
            if computed_hash <= *proof_element {
                computed_hash = Self::hash_pair(&computed_hash, proof_element);
            } else {
                computed_hash = Self::hash_pair(proof_element, &computed_hash);
            }
        }
        
        computed_hash == *root
    }

    /// Hash two 32-byte arrays together
    fn hash_pair(a: &[u8; 32], b: &[u8; 32]) -> [u8; 32] {
        let mut hasher = Hasher::default();
        hasher.hash(a);
        hasher.hash(b);
        hasher.result().to_bytes()
    }

    /// Generate withdrawal receipt hash
    pub fn generate_withdrawal_receipt(
        bridge_hash: &[u8; 32],
        recipient: &Pubkey,
        amount: u64,
        fee: u64,
        timestamp: i64,
    ) -> [u8; 32] {
        let mut hasher = Hasher::default();
        
        hasher.hash(bridge_hash);
        hasher.hash(&recipient.to_bytes());
        hasher.hash(&amount.to_le_bytes());
        hasher.hash(&fee.to_le_bytes());
        hasher.hash(&timestamp.to_le_bytes());
        
        hasher.result().to_bytes()
    }

    /// Validate signature using secp256k1
    pub fn verify_secp256k1_signature(
        message: &[u8],
        signature: &[u8; 64],
        public_key: &[u8; 64],
    ) -> Result<bool> {
        // This would integrate with secp256k1 library in production
        // For now, we'll do basic validation
        if signature.len() != 64 {
            return Err(FinovaBridgeError::InvalidSignature.into());
        }
        
        if public_key.len() != 64 {
            return Err(FinovaBridgeError::InvalidPublicKey.into());
        }
        
        // In production, use proper secp256k1 verification
        // This is a placeholder implementation
        Ok(true)
    }

    /// Calculate gas price for destination chain
    pub fn calculate_gas_price(dest_chain: u16, priority: u8) -> Result<u64> {
        let base_gas = match dest_chain {
            ETHEREUM_CHAIN_ID => 21000,
            BSC_CHAIN_ID => 21000,
            POLYGON_CHAIN_ID => 21000,
            AVALANCHE_CHAIN_ID => 21000,
            ARBITRUM_CHAIN_ID => 21000,
            OPTIMISM_CHAIN_ID => 21000,
            _ => return Err(FinovaBridgeError::UnsupportedChain.into()),
        };

        let multiplier = match priority {
            0 => 1,     // Slow
            1 => 2,     // Standard
            2 => 3,     // Fast
            _ => return Err(FinovaBridgeError::InvalidPriority.into()),
        };

        Ok(base_gas * multiplier)
    }

    /// Convert amount between different token decimals
    pub fn convert_decimals(
        amount: u64,
        from_decimals: u8,
        to_decimals: u8,
    ) -> Result<u64> {
        if from_decimals == to_decimals {
            return Ok(amount);
        }

        if from_decimals > to_decimals {
            let divisor = 10_u64.pow((from_decimals - to_decimals) as u32);
            Ok(amount / divisor)
        } else {
            let multiplier = 10_u64.pow((to_decimals - from_decimals) as u32);
            amount
                .checked_mul(multiplier)
                .ok_or(FinovaBridgeError::MathOverflow.into())
        }
    }

    /// Generate unique nonce for bridge transaction
    pub fn generate_nonce(
        user: &Pubkey,
        chain_id: u16,
        slot: u64,
    ) -> u64 {
        let mut hasher = Hasher::default();
        
        hasher.hash(&user.to_bytes());
        hasher.hash(&chain_id.to_le_bytes());
        hasher.hash(&slot.to_le_bytes());
        
        let hash_bytes = hasher.result().to_bytes();
        u64::from_le_bytes([
            hash_bytes[0],
            hash_bytes[1],
            hash_bytes[2],
            hash_bytes[3],
            hash_bytes[4],
            hash_bytes[5],
            hash_bytes[6],
            hash_bytes[7],
        ])
    }

    /// Validate bridge transaction timeout
    pub fn is_transaction_expired(
        created_at: i64,
        current_time: i64,
        timeout_seconds: i64,
    ) -> bool {
        current_time > created_at + timeout_seconds
    }

    /// Calculate confirmation requirements based on amount
    pub fn required_confirmations(amount: u64) -> u8 {
        if amount >= HIGH_VALUE_THRESHOLD {
            HIGH_VALUE_CONFIRMATIONS
        } else if amount >= MEDIUM_VALUE_THRESHOLD {
            MEDIUM_VALUE_CONFIRMATIONS
        } else {
            LOW_VALUE_CONFIRMATIONS
        }
    }

    /// Generate bridge PDA seeds
    pub fn bridge_seeds(
        bridge_id: &[u8],
        user: &Pubkey,
    ) -> [&[u8]; 3] {
        [
            BRIDGE_SEED,
            bridge_id,
            user.as_ref(),
        ]
    }

    /// Generate locked tokens PDA seeds
    pub fn locked_tokens_seeds(
        mint: &Pubkey,
        user: &Pubkey,
    ) -> [&[u8]; 3] {
        [
            LOCKED_TOKENS_SEED,
            mint.as_ref(),
            user.as_ref(),
        ]
    }

    /// Validate validator signature threshold
    pub fn validate_validator_threshold(
        signatures: &[bool],
        required_threshold: u8,
    ) -> Result<bool> {
        let signature_count = signatures.iter().filter(|&&sig| sig).count() as u8;
        
        if signature_count >= required_threshold {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Calculate time-based penalty for delayed processing
    pub fn calculate_delay_penalty(
        created_at: i64,
        processed_at: i64,
        base_amount: u64,
    ) -> u64 {
        let delay_seconds = processed_at - created_at;
        
        if delay_seconds <= BRIDGE_TIMEOUT_SECONDS {
            return 0;
        }
        
        let delay_hours = (delay_seconds - BRIDGE_TIMEOUT_SECONDS) / 3600;
        let penalty_rate = std::cmp::min(delay_hours as u64 * DELAY_PENALTY_RATE, MAX_DELAY_PENALTY);
        
        base_amount
            .checked_mul(penalty_rate)
            .unwrap_or(0)
            .checked_div(10000)
            .unwrap_or(0)
    }

    /// Format bridge transaction ID
    pub fn format_bridge_id(
        source_chain: u16,
        dest_chain: u16,
        nonce: u64,
    ) -> String {
        format!("{:04x}{:04x}{:016x}", source_chain, dest_chain, nonce)
    }

    /// Parse bridge transaction ID
    pub fn parse_bridge_id(bridge_id: &str) -> Result<(u16, u16, u64)> {
        if bridge_id.len() != 24 {
            return Err(FinovaBridgeError::InvalidBridgeId.into());
        }
        
        let source_chain = u16::from_str_radix(&bridge_id[0..4], 16)
            .map_err(|_| FinovaBridgeError::InvalidBridgeId)?;
        
        let dest_chain = u16::from_str_radix(&bridge_id[4..8], 16)
            .map_err(|_| FinovaBridgeError::InvalidBridgeId)?;
        
        let nonce = u64::from_str_radix(&bridge_id[8..24], 16)
            .map_err(|_| FinovaBridgeError::InvalidBridgeId)?;
        
        Ok((source_chain, dest_chain, nonce))
    }

    /// Check if user has sufficient balance for bridge operation
    pub fn check_sufficient_balance(
        user_balance: u64,
        bridge_amount: u64,
        bridge_fee: u64,
    ) -> Result<()> {
        let total_required = bridge_amount
            .checked_add(bridge_fee)
            .ok_or(FinovaBridgeError::MathOverflow)?;
        
        if user_balance < total_required {
            return Err(FinovaBridgeError::InsufficientBalance.into());
        }
        
        Ok(())
    }

    /// Calculate dynamic fee based on network congestion
    pub fn calculate_dynamic_fee(
        base_fee: u64,
        congestion_multiplier: u64,
    ) -> Result<u64> {
        if congestion_multiplier == 0 {
            return Ok(base_fee);
        }
        
        base_fee
            .checked_mul(congestion_multiplier)
            .ok_or(FinovaBridgeError::MathOverflow.into())
    }

    /// Validate bridge operation timing
    pub fn validate_bridge_timing(
        last_bridge_time: i64,
        current_time: i64,
        min_interval: i64,
    ) -> Result<()> {
        if current_time < last_bridge_time + min_interval {
            return Err(FinovaBridgeError::BridgeTooFrequent.into());
        }
        
        Ok(())
    }

    /// Generate emergency pause hash
    pub fn generate_emergency_hash(
        authority: &Pubkey,
        reason: &str,
        timestamp: i64,
    ) -> [u8; 32] {
        let mut hasher = Hasher::default();
        
        hasher.hash(&authority.to_bytes());
        hasher.hash(reason.as_bytes());
        hasher.hash(&timestamp.to_le_bytes());
        
        hasher.result().to_bytes()
    }

    /// Validate cross-chain message format
    pub fn validate_cross_chain_message(
        message: &[u8],
        expected_format_version: u8,
    ) -> Result<()> {
        if message.len() < MIN_MESSAGE_LENGTH {
            return Err(FinovaBridgeError::InvalidMessageFormat.into());
        }
        
        if message[0] != expected_format_version {
            return Err(FinovaBridgeError::UnsupportedMessageVersion.into());
        }
        
        // Validate message checksum (last 4 bytes)
        let message_body = &message[..message.len() - 4];
        let provided_checksum = u32::from_le_bytes([
            message[message.len() - 4],
            message[message.len() - 3],
            message[message.len() - 2],
            message[message.len() - 1],
        ]);
        
        let calculated_checksum = Self::calculate_checksum(message_body);
        
        if provided_checksum != calculated_checksum {
            return Err(FinovaBridgeError::InvalidMessageChecksum.into());
        }
        
        Ok(())
    }

    /// Calculate message checksum
    fn calculate_checksum(data: &[u8]) -> u32 {
        let mut checksum: u32 = 0;
        
        for chunk in data.chunks(4) {
            let mut bytes = [0u8; 4];
            bytes[..chunk.len()].copy_from_slice(chunk);
            checksum = checksum.wrapping_add(u32::from_le_bytes(bytes));
        }
        
        checksum
    }

    /// Get chain name from ID
    pub fn get_chain_name(chain_id: u16) -> &'static str {
        match chain_id {
            SOLANA_CHAIN_ID => "Solana",
            ETHEREUM_CHAIN_ID => "Ethereum",
            BSC_CHAIN_ID => "BSC",
            POLYGON_CHAIN_ID => "Polygon",
            AVALANCHE_CHAIN_ID => "Avalanche",
            ARBITRUM_CHAIN_ID => "Arbitrum",
            OPTIMISM_CHAIN_ID => "Optimism",
            _ => "Unknown",
        }
    }

    /// Create token transfer instruction
    pub fn create_token_transfer<'info>(
        token_program: &Program<'info, Token>,
        from: &Account<'info, TokenAccount>,
        to: &Account<'info, TokenAccount>,
        authority: &AccountInfo<'info>,
        amount: u64,
    ) -> Result<()> {
        let cpi_accounts = Transfer {
            from: from.to_account_info(),
            to: to.to_account_info(),
            authority: authority.clone(),
        };
        
        let cpi_program = token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        
        token::transfer(cpi_ctx, amount)
    }

    /// Validate bridge configuration
    pub fn validate_bridge_config(
        min_amount: u64,
        max_amount: u64,
        fee_bps: u16,
        timeout_seconds: i64,
    ) -> Result<()> {
        if min_amount >= max_amount {
            return Err(FinovaBridgeError::InvalidConfiguration.into());
        }
        
        if fee_bps > MAX_FEE_BPS {
            return Err(FinovaBridgeError::FeeTooHigh.into());
        }
        
        if timeout_seconds < MIN_TIMEOUT_SECONDS || timeout_seconds > MAX_TIMEOUT_SECONDS {
            return Err(FinovaBridgeError::InvalidTimeout.into());
        }
        
        Ok(())
    }
}

/// Bridge transaction priority levels
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BridgePriority {
    Slow = 0,
    Standard = 1,
    Fast = 2,
}

impl BridgePriority {
    pub fn from_u8(value: u8) -> Result<Self> {
        match value {
            0 => Ok(BridgePriority::Slow),
            1 => Ok(BridgePriority::Standard),
            2 => Ok(BridgePriority::Fast),
            _ => Err(FinovaBridgeError::InvalidPriority.into()),
        }
    }
    
    pub fn gas_multiplier(&self) -> u64 {
        match self {
            BridgePriority::Slow => 1,
            BridgePriority::Standard => 2,
            BridgePriority::Fast => 3,
        }
    }
}

/// Bridge transaction status
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BridgeStatus {
    Pending = 0,
    Confirmed = 1,
    Completed = 2,
    Failed = 3,
    Expired = 4,
}

impl BridgeStatus {
    pub fn from_u8(value: u8) -> Result<Self> {
        match value {
            0 => Ok(BridgeStatus::Pending),
            1 => Ok(BridgeStatus::Confirmed),
            2 => Ok(BridgeStatus::Completed),
            3 => Ok(BridgeStatus::Failed),
            4 => Ok(BridgeStatus::Expired),
            _ => Err(FinovaBridgeError::InvalidStatus.into()),
        }
    }
}

/// Network congestion levels
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CongestionLevel {
    Low = 1,
    Medium = 2,
    High = 3,
    Critical = 5,
}

impl CongestionLevel {
    pub fn from_pending_count(pending_count: u64) -> Self {
        match pending_count {
            0..=100 => CongestionLevel::Low,
            101..=500 => CongestionLevel::Medium,
            501..=1000 => CongestionLevel::High,
            _ => CongestionLevel::Critical,
        }
    }
    
    pub fn fee_multiplier(&self) -> u64 {
        *self as u64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bridge_fee_calculation() {
        let amount = 1000000; // 1 FIN
        let fee = BridgeUtils::calculate_bridge_fee(amount, ETHEREUM_CHAIN_ID).unwrap();
        
        assert!(fee > 0);
        assert!(fee >= BASE_ETHEREUM_FEE);
    }

    #[test]
    fn test_nonce_generation() {
        let user = Pubkey::new_unique();
        let nonce1 = BridgeUtils::generate_nonce(&user, ETHEREUM_CHAIN_ID, 100);
        let nonce2 = BridgeUtils::generate_nonce(&user, ETHEREUM_CHAIN_ID, 101);
        
        assert_ne!(nonce1, nonce2);
    }

    #[test]
    fn test_bridge_id_format_parse() {
        let source_chain = SOLANA_CHAIN_ID;
        let dest_chain = ETHEREUM_CHAIN_ID;
        let nonce = 12345;
        
        let bridge_id = BridgeUtils::format_bridge_id(source_chain, dest_chain, nonce);
        let (parsed_source, parsed_dest, parsed_nonce) = 
            BridgeUtils::parse_bridge_id(&bridge_id).unwrap();
        
        assert_eq!(parsed_source, source_chain);
        assert_eq!(parsed_dest, dest_chain);
        assert_eq!(parsed_nonce, nonce);
    }

    #[test]
    fn test_decimal_conversion() {
        let amount = 1000000; // 1 token with 6 decimals
        let converted = BridgeUtils::convert_decimals(amount, 6, 18).unwrap();
        
        assert_eq!(converted, 1000000000000000000); // 1 token with 18 decimals
    }

    #[test]
    fn test_validator_threshold() {
        let signatures = vec![true, true, false, true, false];
        let result = BridgeUtils::validate_validator_threshold(&signatures, 3).unwrap();
        
        assert!(result);
    }
}
