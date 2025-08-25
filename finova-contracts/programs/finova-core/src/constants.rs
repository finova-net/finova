//! Constants used in the Finova Core program.

use anchor_lang::prelude::*;

// --- Seeds ---
#[constant]
pub const NETWORK_STATE_SEED: &[u8] = b"network_state";

#[constant]
pub const REWARD_POOL_SEED: &[u8] = b"reward_pool";

#[constant]
pub const USER_ACCOUNT_SEED: &[u8] = b"user";

#[constant]
pub const USER_PROFILE_SEED: &[u8] = b"user_profile";

#[constant]
pub const MINING_ACCOUNT_SEED: &[u8] = b"mining";

#[constant]
pub const XP_ACCOUNT_SEED: &[u8] = b"xp";

#[constant]
pub const STAKING_ACCOUNT_SEED: &[u8] = b"staking";

#[constant]
pub const STAKING_VAULT_META_SEED: &[u8] = b"staking_vault_meta";

#[constant]
pub const STAKING_VAULT_TOKENS_SEED: &[u8] = b"staking_vault_tokens";

#[constant]
pub const GUILD_SEED: &[u8] = b"guild";

// --- Game Mechanics ---
#[constant]
pub const MAX_GUILD_MEMBERS: u32 = 50;

#[constant]
pub const XP_PER_LEVEL_BASE: f64 = 100.0;

#[constant]
pub const XP_PER_LEVEL_EXPONENT: f64 = 1.5;

#[constant]
pub const ONE_DAY_IN_SECONDS: i64 = 86_400;
