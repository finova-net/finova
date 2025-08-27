// programs/finova-nft/src/lib.rs

use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Mint, Transfer};
use anchor_spl::associated_token::AssociatedToken;

declare_id!("FinovaNFT1111111111111111111111111111111111");

pub mod constants;
pub mod errors;
pub mod utils;
pub mod state;
pub mod instructions;
pub mod events;

use constants::*;
use errors::*;
use state::*;
use instructions::*;
use events::*;

#[program]
pub mod finova_nft {
    use super::*;

    /// Initialize the NFT program with collection authority
    pub fn initialize_program(
        ctx: Context<InitializeProgram>,
        authority: Pubkey,
    ) -> Result<()> {
        instructions::initialize_program::handler(ctx, authority)
    }

    /// Create a new NFT collection
    pub fn create_collection(
        ctx: Context<CreateCollection>,
        collection_id: String,
        name: String,
        symbol: String,
        uri: String,
        max_supply: Option<u64>,
        royalty_basis_points: u16,
    ) -> Result<()> {
        instructions::create_collection::handler(
            ctx,
            collection_id,
            name,
            symbol,
            uri,
            max_supply,
            royalty_basis_points,
        )
    }

    /// Mint a new NFT
    pub fn mint_nft(
        ctx: Context<MintNft>,
        token_id: String,
        name: String,
        symbol: String,
        uri: String,
        collection_id: Option<String>,
        special_card_type: Option<u8>,
        rarity: u8,
        attributes: Vec<NftAttribute>,
    ) -> Result<()> {
        instructions::mint_nft::handler(
            ctx,
            token_id,
            name,
            symbol,
            uri,
            collection_id,
            special_card_type,
            rarity,
            attributes,
        )
    }

    /// Update NFT metadata
    pub fn update_metadata(
        ctx: Context<UpdateMetadata>,
        name: Option<String>,
        symbol: Option<String>,
        uri: Option<String>,
        attributes: Option<Vec<NftAttribute>>,
    ) -> Result<()> {
        instructions::update_metadata::handler(ctx, name, symbol, uri, attributes)
    }

    /// Transfer NFT to another user
    pub fn transfer_nft(
        ctx: Context<TransferNft>,
        amount: u64,
    ) -> Result<()> {
        instructions::transfer_nft::handler(ctx, amount)
    }

    /// Burn an NFT
    pub fn burn_nft(
        ctx: Context<BurnNft>,
        amount: u64,
    ) -> Result<()> {
        instructions::burn_nft::handler(ctx, amount)
    }

    /// Use a special card (single-use NFTs)
    pub fn use_special_card(
        ctx: Context<UseSpecialCard>,
    ) -> Result<()> {
        instructions::use_special_card::handler(ctx)
    }

    /// List NFT on marketplace
    pub fn list_nft(
        ctx: Context<ListNft>,
        price: u64,
        currency_mint: Pubkey,
        duration: i64,
    ) -> Result<()> {
        instructions::marketplace::list_nft_handler(ctx, price, currency_mint, duration)
    }

    /// Update NFT listing
    pub fn update_listing(
        ctx: Context<UpdateListing>,
        new_price: Option<u64>,
        new_duration: Option<i64>,
    ) -> Result<()> {
        instructions::marketplace::update_listing_handler(ctx, new_price, new_duration)
    }

    /// Cancel NFT listing
    pub fn cancel_listing(
        ctx: Context<CancelListing>,
    ) -> Result<()> {
        instructions::marketplace::cancel_listing_handler(ctx)
    }

    /// Purchase NFT from marketplace
    pub fn purchase_nft(
        ctx: Context<PurchaseNft>,
    ) -> Result<()> {
        instructions::marketplace::purchase_nft_handler(ctx)
    }

    /// Make an offer on NFT
    pub fn make_offer(
        ctx: Context<MakeOffer>,
        amount: u64,
        currency_mint: Pubkey,
        expiry: i64,
    ) -> Result<()> {
        instructions::marketplace::make_offer_handler(ctx, amount, currency_mint, expiry)
    }

    /// Accept an offer
    pub fn accept_offer(
        ctx: Context<AcceptOffer>,
    ) -> Result<()> {
        instructions::marketplace::accept_offer_handler(ctx)
    }

    /// Cancel an offer
    pub fn cancel_offer(
        ctx: Context<CancelOffer>,
    ) -> Result<()> {
        instructions::marketplace::cancel_offer_handler(ctx)
    }

    /// Create an auction
    pub fn create_auction(
        ctx: Context<CreateAuction>,
        starting_price: u64,
        reserve_price: Option<u64>,
        currency_mint: Pubkey,
        duration: i64,
    ) -> Result<()> {
        instructions::marketplace::create_auction_handler(
            ctx,
            starting_price,
            reserve_price,
            currency_mint,
            duration,
        )
    }

    /// Place bid in auction
    pub fn place_bid(
        ctx: Context<PlaceBid>,
        amount: u64,
    ) -> Result<()> {
        instructions::marketplace::place_bid_handler(ctx, amount)
    }

    /// Finalize auction
    pub fn finalize_auction(
        ctx: Context<FinalizeAuction>,
    ) -> Result<()> {
        instructions::marketplace::finalize_auction_handler(ctx)
    }

    /// Claim auction proceeds
    pub fn claim_auction_proceeds(
        ctx: Context<ClaimAuctionProceeds>,
    ) -> Result<()> {
        instructions::marketplace::claim_auction_proceeds_handler(ctx)
    }
}

// Program configuration and initialization contexts
#[derive(Accounts)]
pub struct InitializeProgram<'info> {
    #[account(
        init,
        payer = authority,
        space = ProgramConfig::LEN,
        seeds = [PROGRAM_CONFIG_SEED],
        bump
    )]
    pub program_config: Account<'info, ProgramConfig>,
    
    #[account(mut)]
    pub authority: Signer<'info>,
    
    pub system_program: Program<'info, System>,
}

// Collection creation contexts
#[derive(Accounts)]
#[instruction(collection_id: String)]
pub struct CreateCollection<'info> {
    #[account(
        init,
        payer = creator,
        space = Collection::LEN,
        seeds = [COLLECTION_SEED, collection_id.as_bytes()],
        bump
    )]
    pub collection: Account<'info, Collection>,
    
    #[account(
        seeds = [PROGRAM_CONFIG_SEED],
        bump
    )]
    pub program_config: Account<'info, ProgramConfig>,
    
    #[account(mut)]
    pub creator: Signer<'info>,
    
    pub system_program: Program<'info, System>,
}

// NFT minting contexts
#[derive(Accounts)]
#[instruction(token_id: String)]
pub struct MintNft<'info> {
    #[account(
        init,
        payer = minter,
        mint::decimals = 0,
        mint::authority = nft_metadata,
        mint::freeze_authority = nft_metadata,
    )]
    pub mint: Account<'info, Mint>,
    
    #[account(
        init,
        payer = minter,
        space = NftMetadata::LEN,
        seeds = [NFT_METADATA_SEED, mint.key().as_ref()],
        bump
    )]
    pub nft_metadata: Account<'info, NftMetadata>,
    
    #[account(
        init,
        payer = minter,
        associated_token::mint = mint,
        associated_token::authority = minter,
    )]
    pub token_account: Account<'info, TokenAccount>,
    
    #[account(
        seeds = [COLLECTION_SEED, collection.collection_id.as_bytes()],
        bump,
        constraint = collection.is_active @ NftError::CollectionInactive
    )]
    pub collection: Option<Account<'info, Collection>>,
    
    #[account(mut)]
    pub minter: Signer<'info>,
    
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

// Metadata update contexts
#[derive(Accounts)]
pub struct UpdateMetadata<'info> {
    #[account(
        mut,
        seeds = [NFT_METADATA_SEED, mint.key().as_ref()],
        bump,
        constraint = nft_metadata.creator == authority.key() @ NftError::UnauthorizedUpdate
    )]
    pub nft_metadata: Account<'info, NftMetadata>,
    
    pub mint: Account<'info, Mint>,
    pub authority: Signer<'info>,
}

// Transfer contexts
#[derive(Accounts)]
pub struct TransferNft<'info> {
    #[account(
        mut,
        seeds = [NFT_METADATA_SEED, mint.key().as_ref()],
        bump
    )]
    pub nft_metadata: Account<'info, NftMetadata>,
    
    pub mint: Account<'info, Mint>,
    
    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = from,
    )]
    pub from_token_account: Account<'info, TokenAccount>,
    
    #[account(
        init_if_needed,
        payer = from,
        associated_token::mint = mint,
        associated_token::authority = to,
    )]
    pub to_token_account: Account<'info, TokenAccount>,
    
    #[account(mut)]
    pub from: Signer<'info>,
    
    /// CHECK: This is the recipient address
    pub to: AccountInfo<'info>,
    
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

// Burn contexts
#[derive(Accounts)]
pub struct BurnNft<'info> {
    #[account(
        mut,
        seeds = [NFT_METADATA_SEED, mint.key().as_ref()],
        bump
    )]
    pub nft_metadata: Account<'info, NftMetadata>,
    
    #[account(mut)]
    pub mint: Account<'info, Mint>,
    
    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = owner,
    )]
    pub token_account: Account<'info, TokenAccount>,
    
    #[account(mut)]
    pub owner: Signer<'info>,
    
    pub token_program: Program<'info, Token>,
}

// Special card usage contexts
#[derive(Accounts)]
pub struct UseSpecialCard<'info> {
    #[account(
        mut,
        seeds = [NFT_METADATA_SEED, mint.key().as_ref()],
        bump,
        constraint = nft_metadata.special_card_type.is_some() @ NftError::NotSpecialCard,
        constraint = !nft_metadata.is_used @ NftError::CardAlreadyUsed
    )]
    pub nft_metadata: Account<'info, NftMetadata>,
    
    #[account(mut)]
    pub mint: Account<'info, Mint>,
    
    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = user,
        constraint = token_account.amount > 0 @ NftError::InsufficientBalance
    )]
    pub token_account: Account<'info, TokenAccount>,
    
    #[account(mut)]
    pub user: Signer<'info>,
    
    pub token_program: Program<'info, Token>,
}

// Marketplace listing contexts
#[derive(Accounts)]
pub struct ListNft<'info> {
    #[account(
        init,
        payer = seller,
        space = MarketplaceListing::LEN,
        seeds = [LISTING_SEED, mint.key().as_ref(), seller.key().as_ref()],
        bump
    )]
    pub listing: Account<'info, MarketplaceListing>,
    
    #[account(
        seeds = [NFT_METADATA_SEED, mint.key().as_ref()],
        bump
    )]
    pub nft_metadata: Account<'info, NftMetadata>,
    
    pub mint: Account<'info, Mint>,
    
    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = seller,
        constraint = seller_token_account.amount > 0 @ NftError::InsufficientBalance
    )]
    pub seller_token_account: Account<'info, TokenAccount>,
    
    #[account(
        init,
        payer = seller,
        associated_token::mint = mint,
        associated_token::authority = escrow_account,
    )]
    pub escrow_token_account: Account<'info, TokenAccount>,
    
    /// CHECK: This is a PDA for escrow
    #[account(
        seeds = [ESCROW_SEED, listing.key().as_ref()],
        bump
    )]
    pub escrow_account: AccountInfo<'info>,
    
    #[account(mut)]
    pub seller: Signer<'info>,
    
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct UpdateListing<'info> {
    #[account(
        mut,
        seeds = [LISTING_SEED, mint.key().as_ref(), seller.key().as_ref()],
        bump,
        constraint = listing.seller == seller.key() @ NftError::UnauthorizedSeller
    )]
    pub listing: Account<'info, MarketplaceListing>,
    
    pub mint: Account<'info, Mint>,
    pub seller: Signer<'info>,
}

#[derive(Accounts)]
pub struct CancelListing<'info> {
    #[account(
        mut,
        seeds = [LISTING_SEED, mint.key().as_ref(), seller.key().as_ref()],
        bump,
        constraint = listing.seller == seller.key() @ NftError::UnauthorizedSeller,
        close = seller
    )]
    pub listing: Account<'info, MarketplaceListing>,
    
    pub mint: Account<'info, Mint>,
    
    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = seller,
    )]
    pub seller_token_account: Account<'info, TokenAccount>,
    
    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = escrow_account,
    )]
    pub escrow_token_account: Account<'info, TokenAccount>,
    
    /// CHECK: This is a PDA for escrow
    #[account(
        seeds = [ESCROW_SEED, listing.key().as_ref()],
        bump
    )]
    pub escrow_account: AccountInfo<'info>,
    
    #[account(mut)]
    pub seller: Signer<'info>,
    
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct PurchaseNft<'info> {
    #[account(
        mut,
        seeds = [LISTING_SEED, mint.key().as_ref(), seller.key().as_ref()],
        bump,
        constraint = listing.is_active @ NftError::ListingInactive,
        constraint = Clock::get()?.unix_timestamp <= listing.expires_at @ NftError::ListingExpired,
        close = seller
    )]
    pub listing: Account<'info, MarketplaceListing>,
    
    pub mint: Account<'info, Mint>,
    
    #[account(
        mut,
        associated_token::mint = listing.currency_mint,
        associated_token::authority = buyer,
    )]
    pub buyer_payment_account: Account<'info, TokenAccount>,
    
    #[account(
        init_if_needed,
        payer = buyer,
        associated_token::mint = mint,
        associated_token::authority = buyer,
    )]
    pub buyer_token_account: Account<'info, TokenAccount>,
    
    #[account(
        mut,
        associated_token::mint = listing.currency_mint,
        associated_token::authority = seller,
    )]
    pub seller_payment_account: Account<'info, TokenAccount>,
    
    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = escrow_account,
    )]
    pub escrow_token_account: Account<'info, TokenAccount>,
    
    /// CHECK: This is a PDA for escrow
    #[account(
        seeds = [ESCROW_SEED, listing.key().as_ref()],
        bump
    )]
    pub escrow_account: AccountInfo<'info>,
    
    /// CHECK: This is the seller account
    #[account(mut)]
    pub seller: AccountInfo<'info>,
    
    #[account(mut)]
    pub buyer: Signer<'info>,
    
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
    pub clock: Sysvar<'info, Clock>,
}

// Offer contexts
#[derive(Accounts)]
pub struct MakeOffer<'info> {
    #[account(
        init,
        payer = bidder,
        space = MarketplaceOffer::LEN,
        seeds = [OFFER_SEED, mint.key().as_ref(), bidder.key().as_ref()],
        bump
    )]
    pub offer: Account<'info, MarketplaceOffer>,
    
    pub mint: Account<'info, Mint>,
    
    #[account(
        mut,
        associated_token::mint = currency_mint,
        associated_token::authority = bidder,
    )]
    pub bidder_payment_account: Account<'info, TokenAccount>,
    
    #[account(
        init,
        payer = bidder,
        associated_token::mint = currency_mint,
        associated_token::authority = offer_escrow,
    )]
    pub offer_escrow_account: Account<'info, TokenAccount>,
    
    /// CHECK: This is a PDA for offer escrow
    #[account(
        seeds = [OFFER_ESCROW_SEED, offer.key().as_ref()],
        bump
    )]
    pub offer_escrow: AccountInfo<'info>,
    
    pub currency_mint: Account<'info, Mint>,
    
    #[account(mut)]
    pub bidder: Signer<'info>,
    
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct AcceptOffer<'info> {
    #[account(
        mut,
        seeds = [OFFER_SEED, mint.key().as_ref(), bidder.key().as_ref()],
        bump,
        constraint = offer.is_active @ NftError::OfferInactive,
        constraint = Clock::get()?.unix_timestamp <= offer.expires_at @ NftError::OfferExpired,
        close = bidder
    )]
    pub offer: Account<'info, MarketplaceOffer>,
    
    pub mint: Account<'info, Mint>,
    
    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = seller,
        constraint = seller_token_account.amount > 0 @ NftError::InsufficientBalance
    )]
    pub seller_token_account: Account<'info, TokenAccount>,
    
    #[account(
        init_if_needed,
        payer = seller,
        associated_token::mint = mint,
        associated_token::authority = bidder,
    )]
    pub bidder_token_account: Account<'info, TokenAccount>,
    
    #[account(
        init_if_needed,
        payer = seller,
        associated_token::mint = offer.currency_mint,
        associated_token::authority = seller,
    )]
    pub seller_payment_account: Account<'info, TokenAccount>,
    
    #[account(
        mut,
        associated_token::mint = offer.currency_mint,
        associated_token::authority = offer_escrow,
    )]
    pub offer_escrow_account: Account<'info, TokenAccount>,
    
    /// CHECK: This is a PDA for offer escrow
    #[account(
        seeds = [OFFER_ESCROW_SEED, offer.key().as_ref()],
        bump
    )]
    pub offer_escrow: AccountInfo<'info>,
    
    /// CHECK: This is the bidder account
    #[account(mut)]
    pub bidder: AccountInfo<'info>,
    
    #[account(mut)]
    pub seller: Signer<'info>,
    
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
    pub clock: Sysvar<'info, Clock>,
}

#[derive(Accounts)]
pub struct CancelOffer<'info> {
    #[account(
        mut,
        seeds = [OFFER_SEED, mint.key().as_ref(), bidder.key().as_ref()],
        bump,
        constraint = offer.bidder == bidder.key() @ NftError::UnauthorizedBidder,
        close = bidder
    )]
    pub offer: Account<'info, MarketplaceOffer>,
    
    pub mint: Account<'info, Mint>,
    
    #[account(
        mut,
        associated_token::mint = offer.currency_mint,
        associated_token::authority = bidder,
    )]
    pub bidder_payment_account: Account<'info, TokenAccount>,
    
    #[account(
        mut,
        associated_token::mint = offer.currency_mint,
        associated_token::authority = offer_escrow,
    )]
    pub offer_escrow_account: Account<'info, TokenAccount>,
    
    /// CHECK: This is a PDA for offer escrow
    #[account(
        seeds = [OFFER_ESCROW_SEED, offer.key().as_ref()],
        bump
    )]
    pub offer_escrow: AccountInfo<'info>,
    
    #[account(mut)]
    pub bidder: Signer<'info>,
    
    pub token_program: Program<'info, Token>,
}

// Auction contexts
#[derive(Accounts)]
pub struct CreateAuction<'info> {
    #[account(
        init,
        payer = seller,
        space = MarketplaceAuction::LEN,
        seeds = [AUCTION_SEED, mint.key().as_ref(), seller.key().as_ref()],
        bump
    )]
    pub auction: Account<'info, MarketplaceAuction>,
    
    pub mint: Account<'info, Mint>,
    
    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = seller,
        constraint = seller_token_account.amount > 0 @ NftError::InsufficientBalance
    )]
    pub seller_token_account: Account<'info, TokenAccount>,
    
    #[account(
        init,
        payer = seller,
        associated_token::mint = mint,
        associated_token::authority = auction_escrow,
    )]
    pub auction_escrow_token_account: Account<'info, TokenAccount>,
    
    /// CHECK: This is a PDA for auction escrow
    #[account(
        seeds = [AUCTION_ESCROW_SEED, auction.key().as_ref()],
        bump
    )]
    pub auction_escrow: AccountInfo<'info>,
    
    #[account(mut)]
    pub seller: Signer<'info>,
    
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct PlaceBid<'info> {
    #[account(
        mut,
        seeds = [AUCTION_SEED, mint.key().as_ref(), seller.key().as_ref()],
        bump,
        constraint = auction.is_active @ NftError::AuctionInactive,
        constraint = Clock::get()?.unix_timestamp <= auction.ends_at @ NftError::AuctionEnded
    )]
    pub auction: Account<'info, MarketplaceAuction>,
    
    pub mint: Account<'info, Mint>,
    
    #[account(
        mut,
        associated_token::mint = auction.currency_mint,
        associated_token::authority = bidder,
    )]
    pub bidder_payment_account: Account<'info, TokenAccount>,
    
    #[account(
        init_if_needed,
        payer = bidder,
        associated_token::mint = auction.currency_mint,
        associated_token::authority = bid_escrow,
    )]
    pub bid_escrow_account: Account<'info, TokenAccount>,
    
    /// CHECK: This is a PDA for bid escrow
    #[account(
        seeds = [BID_ESCROW_SEED, auction.key().as_ref(), bidder.key().as_ref()],
        bump
    )]
    pub bid_escrow: AccountInfo<'info>,
    
    /// CHECK: This is the seller account
    pub seller: AccountInfo<'info>,
    
    #[account(mut)]
    pub bidder: Signer<'info>,
    
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
    pub clock: Sysvar<'info, Clock>,
}

#[derive(Accounts)]
pub struct FinalizeAuction<'info> {
    #[account(
        mut,
        seeds = [AUCTION_SEED, mint.key().as_ref(), seller.key().as_ref()],
        bump,
        constraint = !auction.is_active || Clock::get()?.unix_timestamp > auction.ends_at @ NftError::AuctionStillActive
    )]
    pub auction: Account<'info, MarketplaceAuction>,
    
    pub mint: Account<'info, Mint>,
    
    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = auction_escrow,
    )]
    pub auction_escrow_token_account: Account<'info, TokenAccount>,
    
    /// CHECK: This is a PDA for auction escrow
    #[account(
        seeds = [AUCTION_ESCROW_SEED, auction.key().as_ref()],
        bump
    )]
    pub auction_escrow: AccountInfo<'info>,
    
    /// CHECK: This can be any account that calls finalize
    pub finalizer: Signer<'info>,
    
    pub token_program: Program<'info, Token>,
    pub clock: Sysvar<'info, Clock>,
}

#[derive(Accounts)]
pub struct ClaimAuctionProceeds<'info> {
    #[account(
        mut,
        seeds = [AUCTION_SEED, mint.key().as_ref(), seller.key().as_ref()],
        bump,
        constraint = auction.is_finalized @ NftError::AuctionNotFinalized,
        constraint = auction.seller == seller.key() @ NftError::UnauthorizedSeller
    )]
    pub auction: Account<'info, MarketplaceAuction>,
    
    pub mint: Account<'info, Mint>,
    
    #[account(
        init_if_needed,
        payer = seller,
        associated_token::mint = auction.currency_mint,
        associated_token::authority = seller,
    )]
    pub seller_payment_account: Account<'info, TokenAccount>,
    
    #[account(
        mut,
        associated_token::mint = auction.currency_mint,
        associated_token::authority = bid_escrow,
    )]
    pub winning_bid_escrow_account: Account<'info, TokenAccount>,
    
    /// CHECK: This is a PDA for the winning bid escrow
    #[account(
        seeds = [BID_ESCROW_SEED, auction.key().as_ref(), auction.highest_bidder.as_ref()],
        bump
    )]
    pub bid_escrow: AccountInfo<'info>,
    
    #[account(mut)]
    pub seller: Signer<'info>,
    
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}
