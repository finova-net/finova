//! Finova NFT Program
//!
//! This program handles the creation and management of Finova-specific NFTs,
//! such as profile badges, achievements, and special cards.

use anchor_lang::prelude::*;

pub mod instructions;
pub mod state;
pub mod errors;

use instructions::*;

declare_id!("FinovaNftProgram11111111111111111111111111");

#[program]
pub mod finova_nft {
    use super::*;

    /// Creates a new NFT collection.
    ///
    /// # Arguments
    ///
    /// * `ctx` - The context for this instruction.
    /// * `name` - The name of the collection.
    /// * `symbol` - The symbol for the collection's tokens.
    /// * `uri` - The URI for the collection's metadata.
    pub fn create_collection(ctx: Context<CreateCollection>, name: String, symbol: String, uri: String) -> Result<()> {
        instructions::collection::create_handler(ctx, name, symbol, uri)
    }

    /// Mints a new NFT into a collection.
    ///
    /// # Arguments
    ///
    /// * `ctx` - The context for this instruction.
    /// * `name` - The name of the NFT.
    /// * `symbol` - The symbol for the NFT.
    /// * `uri` - The URI for the NFT's metadata.
    pub fn mint_nft(ctx: Context<MintNft>, name: String, symbol: String, uri: String) -> Result<()> {
        instructions::mint::mint_handler(ctx, name, symbol, uri)
    }
}
