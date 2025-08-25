//! Error types for the Finova Core program.

use anchor_lang::prelude::*;

#[error_code]
pub enum FinovaError {
    #[msg("Invalid amount specified. Amount must be greater than zero.")]
    InvalidAmount,

    #[msg("The user is already in a guild.")]
    AlreadyInGuild,

    #[msg("The user is not in a guild.")]
    NotInGuild,

    #[msg("The user does not have a sufficient level to perform this action.")]
    InsufficientLevel,

    #[msg("The specified guild is not active.")]
    GuildNotActive,

    #[msg("The specified guild is full.")]
    GuildFull,

    #[msg("A guild leader cannot leave their own guild. It must be dissolved.")]
    LeaderCannotLeaveGuild,

    #[msg("This social media activity has already been processed.")]
    DuplicateActivity,

    #[msg("Mining session is already active.")]
    MiningAlreadyActive,

    #[msg("Mining session is not currently active.")]
    MiningNotActive,

    #[msg("There are no rewards to claim at this time.")]
    NoRewardsToClaim,

    #[msg("The network rewards pool is currently empty.")]
    InsufficientRewardsPool,

    #[msg("The user has an insufficient staked amount for this transaction.")]
    InsufficientStakedAmount,
}
