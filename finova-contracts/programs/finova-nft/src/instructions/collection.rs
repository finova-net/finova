//! Instruction to create a new NFT collection.

use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{Mint, Token, TokenAccount},
};
use mpl_token_metadata::{
    pda::{find_master_edition_account, find_metadata_account},
    state::DataV2,
    instruction::{create_metadata_accounts_v3, create_master_edition_v3},
};

#[derive(Accounts)]
pub struct CreateCollection<'info> {
    /// The authority creating the collection, who will pay for the transaction.
    #[account(mut)]
    pub authority: Signer<'info>,

    /// The new mint account for the collection NFT.
    #[account(
        init,
        payer = authority,
        mint::decimals = 0,
        mint::authority = authority,
        mint::freeze_authority = authority
    )]
    pub collection_mint: Account<'info, Mint>,

    /// The token account that will hold the single collection NFT.
    #[account(
        init,
        payer = authority,
        associated_token::mint = collection_mint,
        associated_token::authority = authority
    )]
    pub token_account: Account<'info, TokenAccount>,

    /// The metadata account for the collection NFT.
    /// CHECK: This will be created and owned by the Token Metadata program.
    #[account(
        mut,
        address = find_metadata_account(&collection_mint.key()).0
    )]
    pub metadata_account: UncheckedAccount<'info>,

    /// The master edition account for the collection NFT.
    /// CHECK: This will be created and owned by the Token Metadata program.
    #[account(
        mut,
        address = find_master_edition_account(&collection_mint.key()).0
    )]
    pub master_edition_account: UncheckedAccount<'info>,

    // --- Programs ---
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
    /// CHECK: The address is verified in the CPI call.
    pub token_metadata_program: UncheckedAccount<'info>,
}


/// # Handler for the `create_collection` instruction
pub fn create_handler(ctx: Context<CreateCollection>, name: String, symbol: String, uri: String) -> Result<()> {
    // --- Create Metadata Account ---
    let cpi_context = CpiContext::new(
        ctx.accounts.token_metadata_program.to_account_info(),
        create_metadata_accounts_v3 {
            metadata: ctx.accounts.metadata_account.to_account_info(),
            mint: ctx.accounts.collection_mint.to_account_info(),
            mint_authority: ctx.accounts.authority.to_account_info(),
            payer: ctx.accounts.authority.to_account_info(),
            update_authority: ctx.accounts.authority.to_account_info(),
            system_program: ctx.accounts.system_program.to_account_info(),
            rent: ctx.accounts.rent.to_account_info(),
        },
    );

    let data_v2 = DataV2 {
        name,
        symbol,
        uri,
        seller_fee_basis_points: 0,
        creators: None,
        collection: None,
        uses: None,
    };

    create_metadata_accounts_v3(cpi_context, data_v2, false, true, None)?;

    // --- Create Master Edition Account ---
    let cpi_context = CpiContext::new(
        ctx.accounts.token_metadata_program.to_account_info(),
        create_master_edition_v3 {
            edition: ctx.accounts.master_edition_account.to_account_info(),
            mint: ctx.accounts.collection_mint.to_account_info(),
            update_authority: ctx.accounts.authority.to_account_info(),
            mint_authority: ctx.accounts.authority.to_account_info(),
            payer: ctx.accounts.authority.to_account_info(),
            metadata: ctx.accounts.metadata_account.to_account_info(),
            token_program: ctx.accounts.token_program.to_account_info(),
            system_program: ctx.accounts.system_program.to_account_info(),
            rent: ctx.accounts.rent.to_account_info(),
        },
    );

    create_master_edition_v3(cpi_context, Some(0))?;

    msg!("Collection NFT created successfully: {}", ctx.accounts.collection_mint.key());
    Ok(())
}
