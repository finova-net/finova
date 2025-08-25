//! Instruction to mint a new NFT into a collection.

use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{mint_to, Mint, MintTo, Token, TokenAccount},
};
use mpl_token_metadata::{
    pda::{find_master_edition_account, find_metadata_account},
    state::DataV2,
    instruction::{create_metadata_accounts_v3, set_and_verify_sized_collection_item},
};

#[derive(Accounts)]
pub struct MintNft<'info> {
    /// The authority minting the NFT, who will pay for the transaction.
    #[account(mut)]
    pub authority: Signer<'info>,

    /// The new mint account for the NFT.
    #[account(
        init,
        payer = authority,
        mint::decimals = 0,
        mint::authority = authority,
        mint::freeze_authority = authority
    )]
    pub nft_mint: Account<'info, Mint>,

    /// The recipient's associated token account for the new NFT.
    #[account(
        init,
        payer = authority,
        associated_token::mint = nft_mint,
        associated_token::authority = recipient
    )]
    pub token_account: Account<'info, TokenAccount>,

    /// The recipient of the NFT.
    /// CHECK: This is the wallet that will receive the NFT.
    pub recipient: UncheckedAccount<'info>,

    /// The metadata account for the new NFT.
    /// CHECK: This will be created and owned by the Token Metadata program.
    #[account(
        mut,
        address = find_metadata_account(&nft_mint.key()).0
    )]
    pub metadata_account: UncheckedAccount<'info>,

    /// The master edition account for the new NFT.
    /// CHECK: This will be created and owned by the Token Metadata program.
    #[account(
        mut,
        address = find_master_edition_account(&nft_mint.key()).0
    )]
    pub master_edition_account: UncheckedAccount<'info>,

    /// The collection's mint account.
    pub collection_mint: Account<'info, Mint>,

    /// The collection's metadata account.
    #[account(mut)]
    pub collection_metadata: UncheckedAccount<'info>,

    /// The collection's master edition account.
    #[account(mut)]
    pub collection_master_edition: UncheckedAccount<'info>,

    // --- Programs ---
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
    /// CHECK: The address is verified in the CPI call.
    pub token_metadata_program: UncheckedAccount<'info>,
}


/// # Handler for the `mint_nft` instruction
pub fn mint_handler(ctx: Context<MintNft>, name: String, symbol: String, uri: String) -> Result<()> {
    // --- Mint one token to the recipient's ATA ---
    mint_to(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            MintTo {
                mint: ctx.accounts.nft_mint.to_account_info(),
                to: ctx.accounts.token_account.to_account_info(),
                authority: ctx.accounts.authority.to_account_info(),
            },
        ),
        1,
    )?;

    // --- Create Metadata Account for the new NFT ---
    let cpi_context = CpiContext::new(
        ctx.accounts.token_metadata_program.to_account_info(),
        create_metadata_accounts_v3 {
            metadata: ctx.accounts.metadata_account.to_account_info(),
            mint: ctx.accounts.nft_mint.to_account_info(),
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
        collection: Some(mpl_token_metadata::state::Collection {
            verified: false, // It will be verified in the next step
            key: ctx.accounts.collection_mint.key(),
        }),
        uses: None,
    };

    create_metadata_accounts_v3(cpi_context, data_v2, false, true, None)?;

    // --- Verify the new NFT as part of the collection ---
    let cpi_context = CpiContext::new(
        ctx.accounts.token_metadata_program.to_account_info(),
        set_and_verify_sized_collection_item {
            metadata: ctx.accounts.metadata_account.to_account_info(),
            collection_authority: ctx.accounts.authority.to_account_info(),
            payer: ctx.accounts.authority.to_account_info(),
            update_authority: ctx.accounts.authority.to_account_info(),
            collection_mint: ctx.accounts.collection_mint.to_account_info(),
            collection: ctx.accounts.collection_metadata.to_account_info(),
            collection_master_edition_account: ctx.accounts.collection_master_edition.to_account_info(),
            collection_authority_record: None,
        },
    );

    set_and_verify_sized_collection_item(cpi_context)?;

    msg!("NFT {} minted into collection {}.", ctx.accounts.nft_mint.key(), ctx.accounts.collection_mint.key());
    Ok(())
}
