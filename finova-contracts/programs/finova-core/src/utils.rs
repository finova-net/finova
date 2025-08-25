//! Utility functions for the Finova Core program.

use anchor_lang::prelude::*;
use anchor_lang::solana_program::hash;

/// Generates a unique referral code based on the user's ID.
///
/// This creates a simple, human-readable-enough code by taking the first 8 bytes
/// of the hash of the user ID and base58 encoding it.
pub fn generate_referral_code(user_id: u64) -> [u8; 16] {
    let mut code_bytes = [0u8; 16];
    let user_id_bytes = user_id.to_le_bytes();
    let hash_result = hash::hash(&user_id_bytes).to_bytes();

    // Use the first 16 bytes of the hash for the code
    code_bytes.copy_from_slice(&hash_result[0..16]);

    code_bytes
}
