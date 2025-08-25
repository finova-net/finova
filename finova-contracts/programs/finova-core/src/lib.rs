//! Finova Core Program
//!
//! This program is the central hub for the Finova Network's on-chain operations.
//! It manages user accounts, mining, XP, referrals, staking, guilds, and governance.

use anchor_lang::prelude::*;

// Declare modules for organization
pub mod constants;
pub mod errors;
pub mod events;
pub mod instructions;
pub mod state;
pub mod utils;

// Bring modules into scope
use instructions::*;

// The unique on-chain address of the program.
declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[program]
pub mod finova_core {
    use super::*;

    // -----------------
    // Core Instructions
    // -----------------

    /// Initializes the core state of the Finova Network. This should be called only once.
    ///
    /// # Arguments
    ///
    /// * `ctx` - The context for this instruction.
    /// * `admin` - The public key of the initial administrator.
    /// * `fin_token_mint` - The public key of the $FIN token mint.
    pub fn initialize_network(ctx: Context<InitializeNetwork>, admin: Pubkey, fin_token_mint: Pubkey) -> Result<()> {
        instructions::initialize::handler(ctx, admin, fin_token_mint)
    }

    /// Initializes a new user's set of accounts.
    ///
    /// # Arguments
    ///
    /// * `ctx` - The context for this instruction.
    /// * `referral_code` - The referral code of the user who referred this new user (optional).
    pub fn initialize_user(ctx: Context<InitializeUser>, referral_code: Option<String>) -> Result<()> {
        instructions::user::initialize_handler(ctx, referral_code)
    }

    // -----------------
    // Mining Instructions
    // -----------------

    /// Starts a new mining session for the user.
    pub fn start_mining(ctx: Context<StartMining>) -> Result<()> {
        instructions::mining::start_handler(ctx)
    }

    /// Claims pending mining rewards for the user.
    pub fn claim_rewards(ctx: Context<ClaimRewards>) -> Result<()> {
        instructions::mining::claim_handler(ctx)
    }

    // -----------------
    // XP & Social Instructions
    // -----------------

    /// Updates a user's Experience Points (XP) based on a social activity.
    ///
    /// # Arguments
    ///
    /// * `ctx` - The context for this instruction.
    /// * `xp_amount` - The amount of XP to grant.
    /// * `activity_id` - A unique identifier for the social activity to prevent duplicate processing.
    pub fn grant_xp(ctx: Context<GrantXp>, xp_amount: u64, activity_id: u64) -> Result<()> {
        instructions::xp::grant_handler(ctx, xp_amount, activity_id)
    }

    /// Updates a user's daily login streak.
    pub fn update_streak(ctx: Context<UpdateStreak>) -> Result<()> {
        instructions::xp::streak_handler(ctx)
    }

    // -----------------
    // Referral Instructions
    // -----------------

    /// Processes a new referral, linking a referee to a referrer.
    pub fn process_referral(ctx: Context<ProcessReferral>) -> Result<()> {
        instructions::referral::process_handler(ctx)
    }

    // -----------------
    // Staking Instructions
    // -----------------

    /// Stakes $FIN tokens for a user.
    ///
    /// # Arguments
    ///
    /// * `ctx` - The context for this instruction.
    /// * `amount` - The amount of $FIN tokens to stake.
    pub fn stake(ctx: Context<Stake>, amount: u64) -> Result<()> {
        instructions::staking::stake_handler(ctx, amount)
    }

    /// Unstakes $FIN tokens for a user.
    pub fn unstake(ctx: Context<Unstake>, amount: u64) -> Result<()> {
        instructions::staking::unstake_handler(ctx, amount)
    }

    // -----------------
    // Guild Instructions
    // -----------------

    /// Creates a new guild.
    ///
    /// # Arguments
    ///
    /// * `ctx` - The context for this instruction.
    /// * `name` - The name of the guild.
    /// * `handle` - A unique handle for the guild.
    /// * `metadata_uri` - URI pointing to the guild's metadata (logo, etc.).
    pub fn create_guild(ctx: Context<CreateGuild>, name: String, handle: String, metadata_uri: String) -> Result<()> {
        instructions::guild::create_handler(ctx, name, handle, metadata_uri)
    }

    /// Allows a user to join a guild.
    pub fn join_guild(ctx: Context<JoinGuild>) -> Result<()> {
        instructions::guild::join_handler(ctx)
    }

    /// Allows a user to leave their current guild.
    pub fn leave_guild(ctx: Context<LeaveGuild>) -> Result<()> {
        instructions::guild::leave_handler(ctx)
    }

    // -----------------
    // Governance Instructions (Future Implementation)
    // -----------------

    // pub fn submit_proposal(ctx: Context<SubmitProposal>, ...) -> Result<()> { ... }
    // pub fn vote_on_proposal(ctx: Context<VoteOnProposal>, ...) -> Result<()> { ... }
    // pub fn execute_proposal(ctx: Context<ExecuteProposal>, ...) -> Result<()> { ... }

    // -----------------
    // Admin Instructions
    // -----------------

    /// Pauses or unpauses the entire program.
    pub fn set_paused(ctx: Context<SetPaused>, paused: bool) -> Result<()> {
        instructions::admin::set_paused_handler(ctx, paused)
    }

    /// Updates the network configuration.
    pub fn update_network_config(ctx: Context<UpdateNetworkConfig>, config: NetworkConfig) -> Result<()> {
        instructions::admin::update_network_config_handler(ctx, config)
    }
}
