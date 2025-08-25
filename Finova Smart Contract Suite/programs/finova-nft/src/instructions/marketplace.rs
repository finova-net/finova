// programs/finova-nft/src/instructions/marketplace.rs

use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    metadata::{
        create_metadata_accounts_v3, CreateMetadataAccountsV3, Metadata,
        mpl_token_metadata::types::{DataV2, Creator, Collection},
    },
    token::{self, Mint, Token, TokenAccount, Transfer},
};
use std::collections::BTreeMap;

use crate::{
    constants::*,
    errors::*,
    events::*,
    state::*,
    utils::*,
};

/// Initialize the NFT Marketplace
/// Creates global marketplace configuration and fee structure
#[derive(Accounts)]
#[instruction(params: InitializeMarketplaceParams)]
pub struct InitializeMarketplace<'info> {
    #[account(
        init,
        payer = authority,
        space = Marketplace::LEN,
        seeds = [MARKETPLACE_SEED],
        bump
    )]
    pub marketplace: Account<'info, Marketplace>,

    #[account(
        init,
        payer = authority,
        space = MarketplaceFeeConfig::LEN,
        seeds = [MARKETPLACE_FEE_SEED],
        bump
    )]
    pub fee_config: Account<'info, MarketplaceFeeConfig>,

    #[account(
        init,
        payer = authority,
        space = MarketplaceStats::LEN,
        seeds = [MARKETPLACE_STATS_SEED],
        bump
    )]
    pub marketplace_stats: Account<'info, MarketplaceStats>,

    #[account(mut)]
    pub authority: Signer<'info>,

    /// Treasury account for collecting fees
    /// CHECK: This is safe as we only store the pubkey
    pub treasury: AccountInfo<'info>,

    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct InitializeMarketplaceParams {
    pub trading_fee_bps: u16,        // Trading fee in basis points (100 = 1%)
    pub listing_fee_lamports: u64,   // Fixed listing fee in lamports
    pub royalty_fee_bps: u16,        // Creator royalty fee in basis points
    pub max_royalty_bps: u16,        // Maximum allowed royalty (1000 = 10%)
    pub min_price_lamports: u64,     // Minimum listing price
    pub max_listings_per_user: u32,  // Maximum active listings per user
}

pub fn initialize_marketplace(
    ctx: Context<InitializeMarketplace>,
    params: InitializeMarketplaceParams,
) -> Result<()> {
    let marketplace = &mut ctx.accounts.marketplace;
    let fee_config = &mut ctx.accounts.fee_config;
    let marketplace_stats = &mut ctx.accounts.marketplace_stats;

    // Validate parameters
    require!(
        params.trading_fee_bps <= MAX_TRADING_FEE_BPS,
        MarketplaceError::TradingFeeTooHigh
    );
    require!(
        params.royalty_fee_bps <= MAX_ROYALTY_FEE_BPS,
        MarketplaceError::RoyaltyFeeTooHigh
    );
    require!(
        params.max_royalty_bps <= MAX_ROYALTY_FEE_BPS,
        MarketplaceError::MaxRoyaltyTooHigh
    );
    require!(
        params.min_price_lamports >= MIN_LISTING_PRICE,
        MarketplaceError::MinPriceTooLow
    );

    // Initialize marketplace
    marketplace.authority = ctx.accounts.authority.key();
    marketplace.treasury = ctx.accounts.treasury.key();
    marketplace.is_active = true;
    marketplace.created_at = Clock::get()?.unix_timestamp;
    marketplace.updated_at = Clock::get()?.unix_timestamp;
    marketplace.bump = ctx.bumps.marketplace;

    // Initialize fee configuration
    fee_config.trading_fee_bps = params.trading_fee_bps;
    fee_config.listing_fee_lamports = params.listing_fee_lamports;
    fee_config.royalty_fee_bps = params.royalty_fee_bps;
    fee_config.max_royalty_bps = params.max_royalty_bps;
    fee_config.min_price_lamports = params.min_price_lamports;
    fee_config.max_listings_per_user = params.max_listings_per_user;
    fee_config.bump = ctx.bumps.fee_config;

    // Initialize marketplace statistics
    marketplace_stats.total_listings = 0;
    marketplace_stats.total_sales = 0;
    marketplace_stats.total_volume_sol = 0;
    marketplace_stats.total_volume_fin = 0;
    marketplace_stats.active_listings = 0;
    marketplace_stats.total_users = 0;
    marketplace_stats.bump = ctx.bumps.marketplace_stats;

    emit!(MarketplaceInitialized {
        marketplace: marketplace.key(),
        authority: ctx.accounts.authority.key(),
        treasury: ctx.accounts.treasury.key(),
        trading_fee_bps: params.trading_fee_bps,
        timestamp: Clock::get()?.unix_timestamp,
    });

    Ok(())
}

/// List NFT for sale in the marketplace
#[derive(Accounts)]
#[instruction(listing_id: u64, params: ListNftParams)]
pub struct ListNft<'info> {
    #[account(
        seeds = [MARKETPLACE_SEED],
        bump = marketplace.bump,
        constraint = marketplace.is_active @ MarketplaceError::MarketplaceInactive
    )]
    pub marketplace: Account<'info, Marketplace>,

    #[account(
        seeds = [MARKETPLACE_FEE_SEED],
        bump = fee_config.bump
    )]
    pub fee_config: Account<'info, MarketplaceFeeConfig>,

    #[account(
        init,
        payer = seller,
        space = NftListing::LEN,
        seeds = [LISTING_SEED, seller.key().as_ref(), &listing_id.to_le_bytes()],
        bump
    )]
    pub listing: Account<'info, NftListing>,

    #[account(
        init_if_needed,
        payer = seller,
        space = UserMarketplaceProfile::LEN,
        seeds = [USER_PROFILE_SEED, seller.key().as_ref()],
        bump
    )]
    pub user_profile: Account<'info, UserMarketplaceProfile>,

    pub nft_mint: Account<'info, Mint>,

    #[account(
        mut,
        associated_token::mint = nft_mint,
        associated_token::authority = seller,
        constraint = seller_nft_account.amount == 1 @ MarketplaceError::InsufficientNftBalance
    )]
    pub seller_nft_account: Account<'info, TokenAccount>,

    #[account(
        init,
        payer = seller,
        associated_token::mint = nft_mint,
        associated_token::authority = listing,
    )]
    pub escrow_nft_account: Account<'info, TokenAccount>,

    /// NFT metadata account
    /// CHECK: This is validated by Metaplex
    #[account(
        mut,
        seeds = [
            b"metadata",
            metadata_program.key().as_ref(),
            nft_mint.key().as_ref()
        ],
        bump,
        seeds::program = metadata_program.key()
    )]
    pub nft_metadata: UncheckedAccount<'info>,

    #[account(mut)]
    pub seller: Signer<'info>,

    /// Treasury account for collecting listing fees
    #[account(
        mut,
        constraint = treasury.key() == marketplace.treasury @ MarketplaceError::InvalidTreasury
    )]
    pub treasury: SystemAccount<'info>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub metadata_program: Program<'info, Metadata>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct ListNftParams {
    pub price_sol: Option<u64>,      // Price in SOL (lamports)
    pub price_fin: Option<u64>,      // Price in $FIN tokens
    pub currency: ListingCurrency,   // Accepted currency
    pub duration_hours: u32,         // Listing duration in hours
    pub is_auction: bool,            // Whether this is an auction
    pub reserve_price: Option<u64>,  // Reserve price for auctions
    pub royalty_bps: u16,           // Creator royalty percentage
    pub special_card_boost: Option<u16>, // Special card boost if applicable
}

pub fn list_nft(
    ctx: Context<ListNft>,
    listing_id: u64,
    params: ListNftParams,
) -> Result<()> {
    let listing = &mut ctx.accounts.listing;
    let user_profile = &mut ctx.accounts.user_profile;
    let fee_config = &ctx.accounts.fee_config;

    // Validate listing parameters
    validate_listing_params(&params, fee_config)?;

    // Check user listing limits
    require!(
        user_profile.active_listings < fee_config.max_listings_per_user,
        MarketplaceError::TooManyActiveListings
    );

    // Determine final price based on currency
    let (final_price_sol, final_price_fin) = match params.currency {
        ListingCurrency::Sol => {
            require!(
                params.price_sol.is_some(),
                MarketplaceError::MissingPrice
            );
            let price = params.price_sol.unwrap();
            require!(
                price >= fee_config.min_price_lamports,
                MarketplaceError::PriceTooLow
            );
            (Some(price), None)
        },
        ListingCurrency::Fin => {
            require!(
                params.price_fin.is_some(),
                MarketplaceError::MissingPrice
            );
            (None, params.price_fin)
        },
        ListingCurrency::Both => {
            require!(
                params.price_sol.is_some() && params.price_fin.is_some(),
                MarketplaceError::MissingPrice
            );
            let price_sol = params.price_sol.unwrap();
            require!(
                price_sol >= fee_config.min_price_lamports,
                MarketplaceError::PriceTooLow
            );
            (Some(price_sol), params.price_fin)
        },
    };

    // Calculate expiry time
    let current_time = Clock::get()?.unix_timestamp;
    let expiry_time = current_time + (params.duration_hours as i64 * 3600);

    // Transfer NFT to escrow
    let transfer_ctx = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        Transfer {
            from: ctx.accounts.seller_nft_account.to_account_info(),
            to: ctx.accounts.escrow_nft_account.to_account_info(),
            authority: ctx.accounts.seller.to_account_info(),
        },
    );
    token::transfer(transfer_ctx, 1)?;

    // Collect listing fee
    if fee_config.listing_fee_lamports > 0 {
        let ix = anchor_lang::system_program::Transfer {
            from: ctx.accounts.seller.to_account_info(),
            to: ctx.accounts.treasury.to_account_info(),
        };
        let cpi_ctx = CpiContext::new(ctx.accounts.system_program.to_account_info(), ix);
        anchor_lang::system_program::transfer(cpi_ctx, fee_config.listing_fee_lamports)?;
    }

    // Initialize listing
    listing.listing_id = listing_id;
    listing.seller = ctx.accounts.seller.key();
    listing.nft_mint = ctx.accounts.nft_mint.key();
    listing.price_sol = final_price_sol;
    listing.price_fin = final_price_fin;
    listing.currency = params.currency;
    listing.is_auction = params.is_auction;
    listing.reserve_price = params.reserve_price;
    listing.royalty_bps = params.royalty_bps;
    listing.status = ListingStatus::Active;
    listing.created_at = current_time;
    listing.expires_at = expiry_time;
    listing.special_card_boost = params.special_card_boost;
    listing.bump = ctx.bumps.listing;

    // Update user profile
    if user_profile.user == Pubkey::default() {
        user_profile.user = ctx.accounts.seller.key();
        user_profile.created_at = current_time;
        user_profile.bump = ctx.bumps.user_profile;
    }
    user_profile.active_listings += 1;
    user_profile.total_listings += 1;
    user_profile.updated_at = current_time;

    emit!(NftListed {
        listing_id,
        seller: ctx.accounts.seller.key(),
        nft_mint: ctx.accounts.nft_mint.key(),
        price_sol: final_price_sol,
        price_fin: final_price_fin,
        currency: params.currency,
        is_auction: params.is_auction,
        expires_at: expiry_time,
        timestamp: current_time,
    });

    Ok(())
}

/// Update an existing NFT listing
#[derive(Accounts)]
#[instruction(listing_id: u64)]
pub struct UpdateListing<'info> {
    #[account(
        seeds = [MARKETPLACE_SEED],
        bump = marketplace.bump,
        constraint = marketplace.is_active @ MarketplaceError::MarketplaceInactive
    )]
    pub marketplace: Account<'info, Marketplace>,

    #[account(
        seeds = [MARKETPLACE_FEE_SEED],
        bump = fee_config.bump
    )]
    pub fee_config: Account<'info, MarketplaceFeeConfig>,

    #[account(
        mut,
        seeds = [LISTING_SEED, seller.key().as_ref(), &listing_id.to_le_bytes()],
        bump = listing.bump,
        constraint = listing.seller == seller.key() @ MarketplaceError::UnauthorizedSeller,
        constraint = listing.status == ListingStatus::Active @ MarketplaceError::ListingNotActive
    )]
    pub listing: Account<'info, NftListing>,

    #[account(mut)]
    pub seller: Signer<'info>,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct UpdateListingParams {
    pub new_price_sol: Option<u64>,
    pub new_price_fin: Option<u64>,
    pub new_duration_hours: Option<u32>,
    pub new_currency: Option<ListingCurrency>,
}

pub fn update_listing(
    ctx: Context<UpdateListing>,
    listing_id: u64,
    params: UpdateListingParams,
) -> Result<()> {
    let listing = &mut ctx.accounts.listing;
    let fee_config = &ctx.accounts.fee_config;
    let current_time = Clock::get()?.unix_timestamp;

    // Check if listing is still valid
    require!(
        listing.expires_at > current_time,
        MarketplaceError::ListingExpired
    );

    // Update price if provided
    if let Some(new_price_sol) = params.new_price_sol {
        require!(
            new_price_sol >= fee_config.min_price_lamports,
            MarketplaceError::PriceTooLow
        );
        listing.price_sol = Some(new_price_sol);
    }

    if let Some(new_price_fin) = params.new_price_fin {
        listing.price_fin = Some(new_price_fin);
    }

    // Update currency if provided
    if let Some(new_currency) = params.new_currency {
        listing.currency = new_currency;
    }

    // Update duration if provided
    if let Some(new_duration_hours) = params.new_duration_hours {
        listing.expires_at = current_time + (new_duration_hours as i64 * 3600);
    }

    listing.updated_at = Some(current_time);

    emit!(ListingUpdated {
        listing_id,
        seller: ctx.accounts.seller.key(),
        nft_mint: listing.nft_mint,
        new_price_sol: listing.price_sol,
        new_price_fin: listing.price_fin,
        new_currency: listing.currency,
        timestamp: current_time,
    });

    Ok(())
}

/// Cancel an active NFT listing
#[derive(Accounts)]
#[instruction(listing_id: u64)]
pub struct CancelListing<'info> {
    #[account(
        mut,
        seeds = [LISTING_SEED, seller.key().as_ref(), &listing_id.to_le_bytes()],
        bump = listing.bump,
        constraint = listing.seller == seller.key() @ MarketplaceError::UnauthorizedSeller,
        constraint = listing.status == ListingStatus::Active @ MarketplaceError::ListingNotActive
    )]
    pub listing: Account<'info, NftListing>,

    #[account(
        mut,
        seeds = [USER_PROFILE_SEED, seller.key().as_ref()],
        bump = user_profile.bump
    )]
    pub user_profile: Account<'info, UserMarketplaceProfile>,

    #[account(
        mut,
        associated_token::mint = listing.nft_mint,
        associated_token::authority = listing,
    )]
    pub escrow_nft_account: Account<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = listing.nft_mint,
        associated_token::authority = seller,
    )]
    pub seller_nft_account: Account<'info, TokenAccount>,

    #[account(mut)]
    pub seller: Signer<'info>,

    pub token_program: Program<'info, Token>,
}

pub fn cancel_listing(
    ctx: Context<CancelListing>,
    listing_id: u64,
) -> Result<()> {
    let listing = &mut ctx.accounts.listing;
    let user_profile = &mut ctx.accounts.user_profile;

    // Return NFT to seller
    let listing_key = listing.key();
    let listing_seeds = &[
        LISTING_SEED,
        listing.seller.as_ref(),
        &listing_id.to_le_bytes(),
        &[listing.bump],
    ];
    let signer = &[&listing_seeds[..]];

    let transfer_ctx = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        Transfer {
            from: ctx.accounts.escrow_nft_account.to_account_info(),
            to: ctx.accounts.seller_nft_account.to_account_info(),
            authority: listing.to_account_info(),
        },
        signer,
    );
    token::transfer(transfer_ctx, 1)?;

    // Update listing status
    listing.status = ListingStatus::Cancelled;
    listing.updated_at = Some(Clock::get()?.unix_timestamp);

    // Update user profile
    user_profile.active_listings = user_profile.active_listings.saturating_sub(1);
    user_profile.updated_at = Clock::get()?.unix_timestamp;

    emit!(ListingCancelled {
        listing_id,
        seller: ctx.accounts.seller.key(),
        nft_mint: listing.nft_mint,
        timestamp: Clock::get()?.unix_timestamp,
    });

    Ok(())
}

/// Purchase NFT directly (instant buy)
#[derive(Accounts)]
#[instruction(listing_id: u64)]
pub struct PurchaseNft<'info> {
    #[account(
        seeds = [MARKETPLACE_SEED],
        bump = marketplace.bump,
        constraint = marketplace.is_active @ MarketplaceError::MarketplaceInactive
    )]
    pub marketplace: Account<'info, Marketplace>,

    #[account(
        seeds = [MARKETPLACE_FEE_SEED],
        bump = fee_config.bump
    )]
    pub fee_config: Account<'info, MarketplaceFeeConfig>,

    #[account(
        mut,
        seeds = [MARKETPLACE_STATS_SEED],
        bump = marketplace_stats.bump
    )]
    pub marketplace_stats: Account<'info, MarketplaceStats>,

    #[account(
        mut,
        seeds = [LISTING_SEED, listing.seller.as_ref(), &listing_id.to_le_bytes()],
        bump = listing.bump,
        constraint = listing.status == ListingStatus::Active @ MarketplaceError::ListingNotActive
    )]
    pub listing: Account<'info, NftListing>,

    #[account(
        mut,
        seeds = [USER_PROFILE_SEED, listing.seller.as_ref()],
        bump = seller_profile.bump
    )]
    pub seller_profile: Account<'info, UserMarketplaceProfile>,

    #[account(
        init_if_needed,
        payer = buyer,
        space = UserMarketplaceProfile::LEN,
        seeds = [USER_PROFILE_SEED, buyer.key().as_ref()],
        bump
    )]
    pub buyer_profile: Account<'info, UserMarketplaceProfile>,

    #[account(
        mut,
        associated_token::mint = listing.nft_mint,
        associated_token::authority = listing,
        constraint = escrow_nft_account.amount == 1 @ MarketplaceError::NftNotInEscrow
    )]
    pub escrow_nft_account: Account<'info, TokenAccount>,

    #[account(
        init_if_needed,
        payer = buyer,
        associated_token::mint = listing.nft_mint,
        associated_token::authority = buyer,
    )]
    pub buyer_nft_account: Account<'info, TokenAccount>,

    /// Seller's account to receive payment
    #[account(mut, constraint = seller.key() == listing.seller)]
    pub seller: SystemAccount<'info>,

    #[account(mut)]
    pub buyer: Signer<'info>,

    /// Treasury account for collecting fees
    #[account(
        mut,
        constraint = treasury.key() == marketplace.treasury @ MarketplaceError::InvalidTreasury
    )]
    pub treasury: SystemAccount<'info>,

    /// Creator account for royalties (optional)
    /// CHECK: This will be validated if royalties are applicable
    pub creator: UncheckedAccount<'info>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct PurchaseParams {
    pub payment_currency: PaymentCurrency,
    pub special_card_discount: Option<u16>, // Discount from special cards
}

pub fn purchase_nft(
    ctx: Context<PurchaseNft>,
    listing_id: u64,
    params: PurchaseParams,
) -> Result<()> {
    let listing = &mut ctx.accounts.listing;
    let fee_config = &ctx.accounts.fee_config;
    let marketplace_stats = &mut ctx.accounts.marketplace_stats;
    let seller_profile = &mut ctx.accounts.seller_profile;
    let buyer_profile = &mut ctx.accounts.buyer_profile;
    let current_time = Clock::get()?.unix_timestamp;

    // Check if listing is still valid
    require!(
        listing.expires_at > current_time,
        MarketplaceError::ListingExpired
    );

    // Validate payment currency matches listing
    let purchase_price = match params.payment_currency {
        PaymentCurrency::Sol => {
            require!(
                listing.currency == ListingCurrency::Sol || listing.currency == ListingCurrency::Both,
                MarketplaceError::InvalidPaymentCurrency
            );
            listing.price_sol.ok_or(MarketplaceError::PriceNotSet)?
        },
        PaymentCurrency::Fin => {
            require!(
                listing.currency == ListingCurrency::Fin || listing.currency == ListingCurrency::Both,
                MarketplaceError::InvalidPaymentCurrency
            );
            listing.price_fin.ok_or(MarketplaceError::PriceNotSet)?
        },
    };

    // Apply special card discount if applicable
    let final_price = if let Some(discount_bps) = params.special_card_discount {
        let discount_amount = (purchase_price as u128 * discount_bps as u128 / 10000) as u64;
        purchase_price.saturating_sub(discount_amount)
    } else {
        purchase_price
    };

    // Calculate fees
    let trading_fee = (final_price as u128 * fee_config.trading_fee_bps as u128 / 10000) as u64;
    let royalty_fee = if listing.royalty_bps > 0 {
        (final_price as u128 * listing.royalty_bps as u128 / 10000) as u64
    } else {
        0
    };

    let seller_proceeds = final_price.saturating_sub(trading_fee).saturating_sub(royalty_fee);

    // Transfer payment based on currency
    match params.payment_currency {
        PaymentCurrency::Sol => {
            // Transfer SOL to seller
            let ix = anchor_lang::system_program::Transfer {
                from: ctx.accounts.buyer.to_account_info(),
                to: ctx.accounts.seller.to_account_info(),
            };
            let cpi_ctx = CpiContext::new(ctx.accounts.system_program.to_account_info(), ix);
            anchor_lang::system_program::transfer(cpi_ctx, seller_proceeds)?;

            // Transfer trading fee to treasury
            if trading_fee > 0 {
                let fee_ix = anchor_lang::system_program::Transfer {
                    from: ctx.accounts.buyer.to_account_info(),
                    to: ctx.accounts.treasury.to_account_info(),
                };
                let fee_cpi_ctx = CpiContext::new(ctx.accounts.system_program.to_account_info(), fee_ix);
                anchor_lang::system_program::transfer(fee_cpi_ctx, trading_fee)?;
            }

            // Transfer royalty to creator if applicable
            if royalty_fee > 0 && ctx.accounts.creator.key() != Pubkey::default() {
                let royalty_ix = anchor_lang::system_program::Transfer {
                    from: ctx.accounts.buyer.to_account_info(),
                    to: ctx.accounts.creator.to_account_info(),
                };
                let royalty_cpi_ctx = CpiContext::new(ctx.accounts.system_program.to_account_info(), royalty_ix);
                anchor_lang::system_program::transfer(royalty_cpi_ctx, royalty_fee)?;
            }

            marketplace_stats.total_volume_sol += final_price;
        },
        PaymentCurrency::Fin => {
            // For $FIN payments, we would need additional token transfer logic
            // This would require $FIN token accounts and transfer instructions
            marketplace_stats.total_volume_fin += final_price;
        },
    }

    // Transfer NFT to buyer
    let listing_key = listing.key();
    let listing_seeds = &[
        LISTING_SEED,
        listing.seller.as_ref(),
        &listing_id.to_le_bytes(),
        &[listing.bump],
    ];
    let signer = &[&listing_seeds[..]];

    let transfer_ctx = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        Transfer {
            from: ctx.accounts.escrow_nft_account.to_account_info(),
            to: ctx.accounts.buyer_nft_account.to_account_info(),
            authority: listing.to_account_info(),
        },
        signer,
    );
    token::transfer(transfer_ctx, 1)?;

    // Update listing status
    listing.status = ListingStatus::Sold;
    listing.buyer = Some(ctx.accounts.buyer.key());
    listing.sold_price = Some(final_price);
    listing.sold_at = Some(current_time);
    listing.updated_at = Some(current_time);

    // Update seller profile
    seller_profile.active_listings = seller_profile.active_listings.saturating_sub(1);
    seller_profile.total_sales += 1;
    seller_profile.total_volume += final_price;
    seller_profile.updated_at = current_time;

    // Update buyer profile
    if buyer_profile.user == Pubkey::default() {
        buyer_profile.user = ctx.accounts.buyer.key();
        buyer_profile.created_at = current_time;
        buyer_profile.bump = ctx.bumps.buyer_profile;
    }
    buyer_profile.total_purchases += 1;
    buyer_profile.total_spent += final_price;
    buyer_profile.updated_at = current_time;

    // Update marketplace statistics
    marketplace_stats.total_sales += 1;
    marketplace_stats.active_listings = marketplace_stats.active_listings.saturating_sub(1);

    emit!(NftSold {
        listing_id,
        seller: listing.seller,
        buyer: ctx.accounts.buyer.key(),
        nft_mint: listing.nft_mint,
        sale_price: final_price,
        currency: params.payment_currency,
        trading_fee,
        royalty_fee,
        timestamp: current_time,
    });

    Ok(())
}

/// Place bid on an auction listing
#[derive(Accounts)]
#[instruction(listing_id: u64, bid_amount: u64)]
pub struct PlaceBid<'info> {
    #[account(
        seeds = [MARKETPLACE_SEED],
        bump = marketplace.bump,
        constraint = marketplace.is_active @ MarketplaceError::MarketplaceInactive
    )]
    pub marketplace: Account<'info, Marketplace>,

    #[account(
        mut,
        seeds = [LISTING_SEED, listing.seller.as_ref(), &listing_id.to_le_bytes()],
        bump = listing.bump,
        constraint = listing.status == ListingStatus::Active @ MarketplaceError::ListingNotActive,
        constraint = listing.is_auction @ MarketplaceError::NotAuction
    )]
    pub listing: Account<'info, NftListing>,

    #[account(
        init,
        payer = bidder,
        space = Bid::LEN,
        seeds = [BID_SEED, listing.key().as_ref(), bidder.key().as_ref()],
        bump
    )]
    pub bid: Account<'info, Bid>,

    #[account(
        init_if_needed,
        payer = bidder,
        space = UserMarketplaceProfile::LEN,
        seeds = [USER_PROFILE_SEED, bidder.key().as_ref()],
        bump
    )]
    pub bidder_profile: Account<'info, UserMarketplaceProfile>,

    /// Escrow account to hold bid funds
    #[account(
        init,
        payer = bidder,
        space = 0,
        seeds = [BID_ESCROW_SEED, listing.key().as_ref(), bidder.key().as_ref()],
        bump
    )]
    pub bid_escrow: SystemAccount<'info>,

    #[account(mut)]
    pub bidder: Signer<'info>,

    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

pub fn place_bid(
    ctx: Context<PlaceBid>,
    listing_id: u64,
    bid_amount: u64,
) -> Result<()> {
    let listing = &mut ctx.accounts.listing;
    let bid = &mut ctx.accounts.bid;
    let bidder_profile = &mut ctx.accounts.bidder_profile;
    let current_time = Clock::get()?.unix_timestamp;

    // Check if listing is still valid
    require!(
        listing.expires_at > current_time,
        MarketplaceError::ListingExpired
    );

    // Validate bid amount
    if let Some(reserve_price) = listing.reserve_price {
        require!(
            bid_amount >= reserve_price,
            MarketplaceError::BidBelowReserve
        );
    }

    // Check if bid is higher than current highest bid
    if let Some(current_highest) = listing.highest_bid {
        require!(
            bid_amount > current_highest,
            MarketplaceError::BidTooLow
        );
    }

    // Transfer bid amount to escrow
    let ix = anchor_lang::system_program::Transfer {
        from: ctx.accounts.bidder.to_account_info(),
        to: ctx.accounts.bid_escrow.to_account_info(),
    };
    let cpi_ctx = CpiContext::new(ctx.accounts.system_program.to_account_info(), ix);
    anchor_lang::system_program::transfer(cpi_ctx, bid_amount)?;

    // Initialize bid
    bid.listing = listing.key();
    bid.bidder = ctx.accounts.bidder.key();
    bid.amount = bid_amount;
    bid.timestamp = current_time;
    bid.is_active = true;
    bid.bump = ctx.bumps.bid;

    // Update listing with new highest bid
    listing.highest_bid = Some(bid_amount);
    listing.highest_bidder = Some(ctx.accounts.bidder.key());
    listing.total_bids += 1;
    listing.updated_at = Some(current_time);

    // Update bidder profile
    if bidder_profile.user == Pubkey::default() {
        bidder_profile.user = ctx.accounts.bidder.key();
        bidder_profile.created_at = current_time;
        bidder_profile.bump = ctx.bumps.bidder_profile;
    }
    bidder_profile.total_bids += 1;
    bidder_profile.updated_at = current_time;

    emit!(BidPlaced {
        listing_id,
        bidder: ctx.accounts.bidder.key(),
        bid_amount,
        nft_mint: listing.nft_mint,
        timestamp: current_time,
    });

    Ok(())
}

/// Accept bid on auction (seller action)
#[derive(Accounts)]
#[instruction(listing_id: u64)]
pub struct AcceptBid<'info> {
    #[account(
        seeds = [MARKETPLACE_SEED],
        bump = marketplace.bump,
        constraint = marketplace.is_active @ MarketplaceError::MarketplaceInactive
    )]
    pub marketplace: Account<'info, Marketplace>,

    #[account(
        seeds = [MARKETPLACE_FEE_SEED],
        bump = fee_config.bump
    )]
    pub fee_config: Account<'info, MarketplaceFeeConfig>,

    #[account(
        mut,
        seeds = [MARKETPLACE_STATS_SEED],
        bump = marketplace_stats.bump
    )]
    pub marketplace_stats: Account<'info, MarketplaceStats>,

    #[account(
        mut,
        seeds = [LISTING_SEED, seller.key().as_ref(), &listing_id.to_le_bytes()],
        bump = listing.bump,
        constraint = listing.seller == seller.key() @ MarketplaceError::UnauthorizedSeller,
        constraint = listing.status == ListingStatus::Active @ MarketplaceError::ListingNotActive,
        constraint = listing.is_auction @ MarketplaceError::NotAuction
    )]
    pub listing: Account<'info, NftListing>,

    #[account(
        mut,
        seeds = [BID_SEED, listing.key().as_ref(), listing.highest_bidder.unwrap().as_ref()],
        bump = winning_bid.bump,
        constraint = winning_bid.is_active @ MarketplaceError::BidNotActive
    )]
    pub winning_bid: Account<'info, Bid>,

    #[account(
        mut,
        seeds = [USER_PROFILE_SEED, seller.key().as_ref()],
        bump = seller_profile.bump
    )]
    pub seller_profile: Account<'info, UserMarketplaceProfile>,

    #[account(
        mut,
        seeds = [USER_PROFILE_SEED, winning_bid.bidder.as_ref()],
        bump = buyer_profile.bump
    )]
    pub buyer_profile: Account<'info, UserMarketplaceProfile>,

    #[account(
        mut,
        associated_token::mint = listing.nft_mint,
        associated_token::authority = listing,
        constraint = escrow_nft_account.amount == 1 @ MarketplaceError::NftNotInEscrow
    )]
    pub escrow_nft_account: Account<'info, TokenAccount>,

    #[account(
        init_if_needed,
        payer = seller,
        associated_token::mint = listing.nft_mint,
        associated_token::authority = winning_bid.bidder,
    )]
    pub buyer_nft_account: Account<'info, TokenAccount>,

    /// Bid escrow account holding the winning bid funds
    #[account(
        mut,
        seeds = [BID_ESCROW_SEED, listing.key().as_ref(), winning_bid.bidder.as_ref()],
        bump
    )]
    pub bid_escrow: SystemAccount<'info>,

    /// Seller's account to receive payment
    #[account(mut)]
    pub seller: Signer<'info>,

    /// Treasury account for collecting fees
    #[account(
        mut,
        constraint = treasury.key() == marketplace.treasury @ MarketplaceError::InvalidTreasury
    )]
    pub treasury: SystemAccount<'info>,

    /// Creator account for royalties (optional)
    /// CHECK: This will be validated if royalties are applicable
    pub creator: UncheckedAccount<'info>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

pub fn accept_bid(
    ctx: Context<AcceptBid>,
    listing_id: u64,
) -> Result<()> {
    let listing = &mut ctx.accounts.listing;
    let winning_bid = &mut ctx.accounts.winning_bid;
    let fee_config = &ctx.accounts.fee_config;
    let marketplace_stats = &mut ctx.accounts.marketplace_stats;
    let seller_profile = &mut ctx.accounts.seller_profile;
    let buyer_profile = &mut ctx.accounts.buyer_profile;
    let current_time = Clock::get()?.unix_timestamp;

    let winning_amount = winning_bid.amount;

    // Calculate fees
    let trading_fee = (winning_amount as u128 * fee_config.trading_fee_bps as u128 / 10000) as u64;
    let royalty_fee = if listing.royalty_bps > 0 {
        (winning_amount as u128 * listing.royalty_bps as u128 / 10000) as u64
    } else {
        0
    };

    let seller_proceeds = winning_amount.saturating_sub(trading_fee).saturating_sub(royalty_fee);

    // Transfer funds from bid escrow to seller
    **ctx.accounts.bid_escrow.to_account_info().try_borrow_mut_lamports()? -= seller_proceeds;
    **ctx.accounts.seller.to_account_info().try_borrow_mut_lamports()? += seller_proceeds;

    // Transfer trading fee to treasury
    if trading_fee > 0 {
        **ctx.accounts.bid_escrow.to_account_info().try_borrow_mut_lamports()? -= trading_fee;
        **ctx.accounts.treasury.to_account_info().try_borrow_mut_lamports()? += trading_fee;
    }

    // Transfer royalty to creator if applicable
    if royalty_fee > 0 && ctx.accounts.creator.key() != Pubkey::default() {
        **ctx.accounts.bid_escrow.to_account_info().try_borrow_mut_lamports()? -= royalty_fee;
        **ctx.accounts.creator.to_account_info().try_borrow_mut_lamports()? += royalty_fee;
    }

    // Transfer NFT to buyer
    let listing_key = listing.key();
    let listing_seeds = &[
        LISTING_SEED,
        listing.seller.as_ref(),
        &listing_id.to_le_bytes(),
        &[listing.bump],
    ];
    let signer = &[&listing_seeds[..]];

    let transfer_ctx = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        Transfer {
            from: ctx.accounts.escrow_nft_account.to_account_info(),
            to: ctx.accounts.buyer_nft_account.to_account_info(),
            authority: listing.to_account_info(),
        },
        signer,
    );
    token::transfer(transfer_ctx, 1)?;

    // Update listing status
    listing.status = ListingStatus::Sold;
    listing.buyer = Some(winning_bid.bidder);
    listing.sold_price = Some(winning_amount);
    listing.sold_at = Some(current_time);
    listing.updated_at = Some(current_time);

    // Update winning bid status
    winning_bid.is_active = false;

    // Update seller profile
    seller_profile.active_listings = seller_profile.active_listings.saturating_sub(1);
    seller_profile.total_sales += 1;
    seller_profile.total_volume += winning_amount;
    seller_profile.updated_at = current_time;

    // Update buyer profile
    buyer_profile.total_purchases += 1;
    buyer_profile.total_spent += winning_amount;
    buyer_profile.updated_at = current_time;

    // Update marketplace statistics
    marketplace_stats.total_sales += 1;
    marketplace_stats.total_volume_sol += winning_amount;
    marketplace_stats.active_listings = marketplace_stats.active_listings.saturating_sub(1);

    emit!(BidAccepted {
        listing_id,
        seller: ctx.accounts.seller.key(),
        buyer: winning_bid.bidder,
        nft_mint: listing.nft_mint,
        winning_bid: winning_amount,
        trading_fee,
        royalty_fee,
        timestamp: current_time,
    });

    Ok(())
}

/// Make an offer on any NFT (even not listed)
#[derive(Accounts)]
#[instruction(offer_amount: u64, duration_hours: u32)]
pub struct MakeOffer<'info> {
    #[account(
        seeds = [MARKETPLACE_SEED],
        bump = marketplace.bump,
        constraint = marketplace.is_active @ MarketplaceError::MarketplaceInactive
    )]
    pub marketplace: Account<'info, Marketplace>,

    #[account(
        init,
        payer = offerer,
        space = Offer::LEN,
        seeds = [OFFER_SEED, nft_mint.key().as_ref(), offerer.key().as_ref()],
        bump
    )]
    pub offer: Account<'info, Offer>,

    #[account(
        init_if_needed,
        payer = offerer,
        space = UserMarketplaceProfile::LEN,
        seeds = [USER_PROFILE_SEED, offerer.key().as_ref()],
        bump
    )]
    pub offerer_profile: Account<'info, UserMarketplaceProfile>,

    pub nft_mint: Account<'info, Mint>,

    /// Escrow account to hold offer funds
    #[account(
        init,
        payer = offerer,
        space = 0,
        seeds = [OFFER_ESCROW_SEED, nft_mint.key().as_ref(), offerer.key().as_ref()],
        bump
    )]
    pub offer_escrow: SystemAccount<'info>,

    #[account(mut)]
    pub offerer: Signer<'info>,

    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

pub fn make_offer(
    ctx: Context<MakeOffer>,
    offer_amount: u64,
    duration_hours: u32,
) -> Result<()> {
    let offer = &mut ctx.accounts.offer;
    let offerer_profile = &mut ctx.accounts.offerer_profile;
    let current_time = Clock::get()?.unix_timestamp;

    // Validate offer parameters
    require!(
        offer_amount > 0,
        MarketplaceError::InvalidOfferAmount
    );
    require!(
        duration_hours >= MIN_OFFER_DURATION_HOURS && duration_hours <= MAX_OFFER_DURATION_HOURS,
        MarketplaceError::InvalidOfferDuration
    );

    // Transfer offer amount to escrow
    let ix = anchor_lang::system_program::Transfer {
        from: ctx.accounts.offerer.to_account_info(),
        to: ctx.accounts.offer_escrow.to_account_info(),
    };
    let cpi_ctx = CpiContext::new(ctx.accounts.system_program.to_account_info(), ix);
    anchor_lang::system_program::transfer(cpi_ctx, offer_amount)?;

    // Calculate expiry time
    let expiry_time = current_time + (duration_hours as i64 * 3600);

    // Initialize offer
    offer.nft_mint = ctx.accounts.nft_mint.key();
    offer.offerer = ctx.accounts.offerer.key();
    offer.amount = offer_amount;
    offer.created_at = current_time;
    offer.expires_at = expiry_time;
    offer.status = OfferStatus::Active;
    offer.bump = ctx.bumps.offer;

    // Update offerer profile
    if offerer_profile.user == Pubkey::default() {
        offerer_profile.user = ctx.accounts.offerer.key();
        offerer_profile.created_at = current_time;
        offerer_profile.bump = ctx.bumps.offerer_profile;
    }
    offerer_profile.total_offers += 1;
    offerer_profile.updated_at = current_time;

    emit!(OfferMade {
        nft_mint: ctx.accounts.nft_mint.key(),
        offerer: ctx.accounts.offerer.key(),
        offer_amount,
        expires_at: expiry_time,
        timestamp: current_time,
    });

    Ok(())
}

/// Accept an offer (NFT owner action)
#[derive(Accounts)]
#[instruction()]
pub struct AcceptOffer<'info> {
    #[account(
        seeds = [MARKETPLACE_SEED],
        bump = marketplace.bump,
        constraint = marketplace.is_active @ MarketplaceError::MarketplaceInactive
    )]
    pub marketplace: Account<'info, Marketplace>,

    #[account(
        seeds = [MARKETPLACE_FEE_SEED],
        bump = fee_config.bump
    )]
    pub fee_config: Account<'info, MarketplaceFeeConfig>,

    #[account(
        mut,
        seeds = [MARKETPLACE_STATS_SEED],
        bump = marketplace_stats.bump
    )]
    pub marketplace_stats: Account<'info, MarketplaceStats>,

    #[account(
        mut,
        seeds = [OFFER_SEED, nft_mint.key().as_ref(), offer.offerer.as_ref()],
        bump = offer.bump,
        constraint = offer.status == OfferStatus::Active @ MarketplaceError::OfferNotActive
    )]
    pub offer: Account<'info, Offer>,

    #[account(
        init_if_needed,
        payer = nft_owner,
        space = UserMarketplaceProfile::LEN,
        seeds = [USER_PROFILE_SEED, nft_owner.key().as_ref()],
        bump
    )]
    pub seller_profile: Account<'info, UserMarketplaceProfile>,

    #[account(
        mut,
        seeds = [USER_PROFILE_SEED, offer.offerer.as_ref()],
        bump = buyer_profile.bump
    )]
    pub buyer_profile: Account<'info, UserMarketplaceProfile>,

    pub nft_mint: Account<'info, Mint>,

    #[account(
        mut,
        associated_token::mint = nft_mint,
        associated_token::authority = nft_owner,
        constraint = owner_nft_account.amount == 1 @ MarketplaceError::InsufficientNftBalance
    )]
    pub owner_nft_account: Account<'info, TokenAccount>,

    #[account(
        init_if_needed,
        payer = nft_owner,
        associated_token::mint = nft_mint,
        associated_token::authority = offer.offerer,
    )]
    pub buyer_nft_account: Account<'info, TokenAccount>,

    /// Offer escrow account holding the offer funds
    #[account(
        mut,
        seeds = [OFFER_ESCROW_SEED, nft_mint.key().as_ref(), offer.offerer.as_ref()],
        bump
    )]
    pub offer_escrow: SystemAccount<'info>,

    /// NFT owner/seller
    #[account(mut)]
    pub nft_owner: Signer<'info>,

    /// Treasury account for collecting fees
    #[account(
        mut,
        constraint = treasury.key() == marketplace.treasury @ MarketplaceError::InvalidTreasury
    )]
    pub treasury: SystemAccount<'info>,

    /// Creator account for royalties (optional)
    /// CHECK: This will be validated if royalties are applicable
    pub creator: UncheckedAccount<'info>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

pub fn accept_offer(ctx: Context<AcceptOffer>) -> Result<()> {
    let offer = &mut ctx.accounts.offer;
    let fee_config = &ctx.accounts.fee_config;
    let marketplace_stats = &mut ctx.accounts.marketplace_stats;
    let seller_profile = &mut ctx.accounts.seller_profile;
    let buyer_profile = &mut ctx.accounts.buyer_profile;
    let current_time = Clock::get()?.unix_timestamp;

    // Check if offer is still valid
    require!(
        offer.expires_at > current_time,
        MarketplaceError::OfferExpired
    );

    let offer_amount = offer.amount;

    // Calculate fees (assuming 5% royalty for simplicity - should be from metadata)
    let trading_fee = (offer_amount as u128 * fee_config.trading_fee_bps as u128 / 10000) as u64;
    let royalty_fee = (offer_amount as u128 * 500u128 / 10000) as u64; // 5% royalty
    let seller_proceeds = offer_amount.saturating_sub(trading_fee).saturating_sub(royalty_fee);

    // Transfer funds from offer escrow to seller
    **ctx.accounts.offer_escrow.to_account_info().try_borrow_mut_lamports()? -= seller_proceeds;
    **ctx.accounts.nft_owner.to_account_info().try_borrow_mut_lamports()? += seller_proceeds;

    // Transfer trading fee to treasury
    if trading_fee > 0 {
        **ctx.accounts.offer_escrow.to_account_info().try_borrow_mut_lamports()? -= trading_fee;
        **ctx.accounts.treasury.to_account_info().try_borrow_mut_lamports()? += trading_fee;
    }

    // Transfer royalty to creator if applicable
    if royalty_fee > 0 && ctx.accounts.creator.key() != Pubkey::default() {
        **ctx.accounts.offer_escrow.to_account_info().try_borrow_mut_lamports()? -= royalty_fee;
        **ctx.accounts.creator.to_account_info().try_borrow_mut_lamports()? += royalty_fee;
    }

    // Transfer NFT to buyer
    let transfer_ctx = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        Transfer {
            from: ctx.accounts.owner_nft_account.to_account_info(),
            to: ctx.accounts.buyer_nft_account.to_account_info(),
            authority: ctx.accounts.nft_owner.to_account_info(),
        },
    );
    token::transfer(transfer_ctx, 1)?;

    // Update offer status
    offer.status = OfferStatus::Accepted;
    offer.accepted_at = Some(current_time);

    // Update seller profile
    if seller_profile.user == Pubkey::default() {
        seller_profile.user = ctx.accounts.nft_owner.key();
        seller_profile.created_at = current_time;
        seller_profile.bump = ctx.bumps.seller_profile;
    }
    seller_profile.total_sales += 1;
    seller_profile.total_volume += offer_amount;
    seller_profile.updated_at = current_time;

    // Update buyer profile
    buyer_profile.total_purchases += 1;
    buyer_profile.total_spent += offer_amount;
    buyer_profile.updated_at = current_time;

    // Update marketplace statistics
    marketplace_stats.total_sales += 1;
    marketplace_stats.total_volume_sol += offer_amount;

    emit!(OfferAccepted {
        nft_mint: ctx.accounts.nft_mint.key(),
        seller: ctx.accounts.nft_owner.key(),
        buyer: offer.offerer,
        offer_amount,
        trading_fee,
        royalty_fee,
        timestamp: current_time,
    });

    Ok(())
}

/// Helper functions for validation and calculations
fn validate_listing_params(
    params: &ListNftParams,
    fee_config: &MarketplaceFeeConfig,
) -> Result<()> {
    // Validate duration
    require!(
        params.duration_hours >= MIN_LISTING_DURATION_HOURS && 
        params.duration_hours <= MAX_LISTING_DURATION_HOURS,
        MarketplaceError::InvalidListingDuration
    );

    // Validate royalty percentage
    require!(
        params.royalty_bps <= fee_config.max_royalty_bps,
        MarketplaceError::RoyaltyFeeTooHigh
    );

    // Validate auction parameters
    if params.is_auction {
        if let Some(reserve_price) = params.reserve_price {
            if let Some(price_sol) = params.price_sol {
                require!(
                    reserve_price <= price_sol,
                    MarketplaceError::ReservePriceTooHigh
                );
            }
        }
    }

    Ok(())
}


// =================================================
// === Additional logic from marketplace2.rs ===
// =================================================

// programs/finova-nft/src/instructions/marketplace2.rs

use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Mint, Transfer};
use anchor_spl::associated_token::AssociatedToken;
use std::collections::HashMap;

use crate::state::*;
use crate::errors::*;
use crate::events::*;

/// Collection Management: Create a new NFT collection
#[derive(Accounts)]
#[instruction(name: String, symbol: String)]
pub struct CreateCollection<'info> {
    #[account(mut)]
    pub creator: Signer<'info>,

    #[account(
        init,
        payer = creator,
        space = Collection::LEN,
        seeds = [b"collection", creator.key().as_ref(), name.as_bytes()],
        bump
    )]
    pub collection: Account<'info, Collection>,

    #[account(
        init,
        payer = creator,
        mint::decimals = 0,
        mint::authority = collection,
        mint::freeze_authority = collection,
    )]
    pub collection_mint: Account<'info, Mint>,

    #[account(
        init,
        payer = creator,
        associated_token::mint = collection_mint,
        associated_token::authority = creator,
    )]
    pub creator_token_account: Account<'info, TokenAccount>,

    #[account(
        mut,
        seeds = [b"marketplace"],
        bump = marketplace.bump
    )]
    pub marketplace: Account<'info, Marketplace>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

pub fn create_collection(
    ctx: Context<CreateCollection>,
    name: String,
    symbol: String,
    description: String,
    image: String,
    external_url: String,
    creator_fee_basis_points: u16,
    max_supply: Option<u64>,
) -> Result<()> {
    let collection = &mut ctx.accounts.collection;
    let marketplace = &mut ctx.accounts.marketplace;
    let creator = &ctx.accounts.creator;
    let clock = Clock::get()?;

    // Validate inputs
    require!(name.len() <= 50, MarketplaceError::NameTooLong);
    require!(symbol.len() <= 10, MarketplaceError::SymbolTooLong);
    require!(description.len() <= 500, MarketplaceError::DescriptionTooLong);
    require!(creator_fee_basis_points <= 1000, MarketplaceError::CreatorFeeTooHigh); // Max 10%

    // Initialize collection
    collection.creator = creator.key();
    collection.name = name.clone();
    collection.symbol = symbol.clone();
    collection.description = description;
    collection.image = image;
    collection.external_url = external_url;
    collection.creator_fee_basis_points = creator_fee_basis_points;
    collection.max_supply = max_supply;
    collection.current_supply = 0;
    collection.mint = ctx.accounts.collection_mint.key();
    collection.verified = false; // Admin verification required
    collection.created_at = clock.unix_timestamp;
    collection.bump = *ctx.bumps.get("collection").unwrap();

    // Mint collection NFT to creator
    let seeds = &[
        b"collection",
        creator.key().as_ref(),
        name.as_bytes(),
        &[collection.bump],
    ];
    let signer_seeds = &[&seeds[..]];

    token::mint_to(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            token::MintTo {
                mint: ctx.accounts.collection_mint.to_account_info(),
                to: ctx.accounts.creator_token_account.to_account_info(),
                authority: collection.to_account_info(),
            },
            signer_seeds,
        ),
        1,
    )?;

    // Update marketplace statistics
    marketplace.total_collections += 1;

    emit!(CollectionCreated {
        collection: collection.key(),
        creator: creator.key(),
        name: name,
        symbol: symbol,
        creator_fee_basis_points,
        max_supply,
        timestamp: clock.unix_timestamp,
    });

    Ok(())
}

/// Verify Collection: Admin function to verify collections
#[derive(Accounts)]
pub struct VerifyCollection<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    #[account(
        mut,
        seeds = [b"collection", collection.creator.as_ref(), collection.name.as_bytes()],
        bump = collection.bump
    )]
    pub collection: Account<'info, Collection>,

    #[account(
        mut,
        seeds = [b"marketplace"],
        bump = marketplace.bump,
        has_one = admin @ MarketplaceError::UnauthorizedAdmin
    )]
    pub marketplace: Account<'info, Marketplace>,
}

pub fn verify_collection(ctx: Context<VerifyCollection>, verified: bool) -> Result<()> {
    let collection = &mut ctx.accounts.collection;
    let marketplace = &mut ctx.accounts.marketplace;
    let clock = Clock::get()?;

    collection.verified = verified;

    if verified {
        marketplace.verified_collections += 1;
    } else if collection.verified {
        marketplace.verified_collections = marketplace.verified_collections.saturating_sub(1);
    }

    emit!(CollectionVerified {
        collection: collection.key(),
        verified,
        admin: ctx.accounts.admin.key(),
        timestamp: clock.unix_timestamp,
    });

    Ok(())
}

/// Batch List NFTs: List multiple NFTs at once
#[derive(Accounts)]
pub struct BatchListNFTs<'info> {
    #[account(mut)]
    pub seller: Signer<'info>,

    #[account(
        mut,
        seeds = [b"marketplace"],
        bump = marketplace.bump
    )]
    pub marketplace: Account<'info, Marketplace>,

    #[account(
        mut,
        seeds = [b"user_profile", seller.key().as_ref()],
        bump = user_profile.bump
    )]
    pub user_profile: Account<'info, UserProfile>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct BatchListingData {
    pub nft_mint: Pubkey,
    pub price: u64,
    pub currency: Currency,
    pub listing_type: ListingType,
    pub duration: i64,
}

pub fn batch_list_nfts(
    ctx: Context<BatchListNFTs>,
    listings: Vec<BatchListingData>,
) -> Result<()> {
    let seller = &ctx.accounts.seller;
    let marketplace = &mut ctx.accounts.marketplace;
    let user_profile = &mut ctx.accounts.user_profile;
    let clock = Clock::get()?;

    require!(listings.len() <= 50, MarketplaceError::BatchSizeTooLarge);
    require!(!listings.is_empty(), MarketplaceError::EmptyBatch);

    let mut successful_listings = 0u32;
    let mut total_value = 0u64;

    for listing_data in listings.iter() {
        // Validate each listing
        require!(listing_data.price > 0, MarketplaceError::InvalidPrice);
        require!(listing_data.duration > 0, MarketplaceError::InvalidDuration);
        require!(
            listing_data.duration <= marketplace.max_listing_duration,
            MarketplaceError::ListingDurationTooLong
        );

        // In a real implementation, you would create individual listing accounts here
        // For brevity, we're just tracking the batch operation
        successful_listings += 1;
        
        // Convert price to SOL equivalent for statistics
        let sol_equivalent = match listing_data.currency {
            Currency::Sol => listing_data.price,
            Currency::Fin => {
                // Assume 1 FIN = 0.1 SOL for calculation (would be from oracle in reality)
                listing_data.price / 10
            }
        };
        total_value += sol_equivalent;
    }

    // Update marketplace statistics
    marketplace.total_listings += successful_listings as u64;
    marketplace.total_volume += total_value;

    // Update user profile
    user_profile.listings_created += successful_listings as u64;
    user_profile.total_volume += total_value;

    emit!(BatchListingCreated {
        seller: seller.key(),
        count: successful_listings,
        total_value,
        timestamp: clock.unix_timestamp,
    });

    Ok(())
}

/// Batch Purchase: Buy multiple NFTs at once
#[derive(Accounts)]
pub struct BatchPurchase<'info> {
    #[account(mut)]
    pub buyer: Signer<'info>,

    #[account(
        mut,
        seeds = [b"marketplace"],
        bump = marketplace.bump
    )]
    pub marketplace: Account<'info, Marketplace>,

    #[account(
        mut,
        seeds = [b"user_profile", buyer.key().as_ref()],
        bump = user_profile.bump
    )]
    pub user_profile: Account<'info, UserProfile>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
}

pub fn batch_purchase(
    ctx: Context<BatchPurchase>,
    listing_keys: Vec<Pubkey>,
    max_total_cost: u64,
) -> Result<()> {
    let buyer = &ctx.accounts.buyer;
    let marketplace = &mut ctx.accounts.marketplace;
    let user_profile = &mut ctx.accounts.user_profile;
    let clock = Clock::get()?;

    require!(listing_keys.len() <= 20, MarketplaceError::BatchSizeTooLarge);
    require!(!listing_keys.is_empty(), MarketplaceError::EmptyBatch);

    let mut total_cost = 0u64;
    let mut successful_purchases = 0u32;

    // In a real implementation, you would iterate through each listing
    // and execute the purchase logic from marketplace1.rs
    for _listing_key in listing_keys.iter() {
        // Simulate purchase cost calculation
        let estimated_cost = 1_000_000_000; // 1 SOL placeholder
        
        if total_cost + estimated_cost <= max_total_cost {
            total_cost += estimated_cost;
            successful_purchases += 1;
        } else {
            break; // Stop if we exceed max budget
        }
    }

    require!(successful_purchases > 0, MarketplaceError::InsufficientFunds);

    // Update marketplace statistics
    marketplace.total_sales += successful_purchases as u64;
    marketplace.total_volume += total_cost;

    // Update user profile
    user_profile.purchases_made += successful_purchases as u64;
    user_profile.total_spent += total_cost;

    emit!(BatchPurchaseCompleted {
        buyer: buyer.key(),
        count: successful_purchases,
        total_cost,
        timestamp: clock.unix_timestamp,
    });

    Ok(())
}

/// Dutch Auction: Create a Dutch auction listing
#[derive(Accounts)]
#[instruction(nft_mint: Pubkey)]
pub struct CreateDutchAuction<'info> {
    #[account(mut)]
    pub seller: Signer<'info>,

    #[account(
        init,
        payer = seller,
        space = DutchAuction::LEN,
        seeds = [b"dutch_auction", nft_mint.as_ref()],
        bump
    )]
    pub dutch_auction: Account<'info, DutchAuction>,

    #[account(
        mut,
        seeds = [b"marketplace"],
        bump = marketplace.bump
    )]
    pub marketplace: Account<'info, Marketplace>,

    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

pub fn create_dutch_auction(
    ctx: Context<CreateDutchAuction>,
    nft_mint: Pubkey,
    starting_price: u64,
    ending_price: u64,
    duration: i64,
    currency: Currency,
    price_drop_interval: i64,
) -> Result<()> {
    let dutch_auction = &mut ctx.accounts.dutch_auction;
    let marketplace = &mut ctx.accounts.marketplace;
    let seller = &ctx.accounts.seller;
    let clock = Clock::get()?;

    // Validation
    require!(starting_price > ending_price, MarketplaceError::InvalidPriceRange);
    require!(duration > 0, MarketplaceError::InvalidDuration);
    require!(price_drop_interval > 0, MarketplaceError::InvalidPriceDropInterval);
    require!(
        duration <= marketplace.max_listing_duration,
        MarketplaceError::ListingDurationTooLong
    );

    dutch_auction.seller = seller.key();
    dutch_auction.nft_mint = nft_mint;
    dutch_auction.starting_price = starting_price;
    dutch_auction.ending_price = ending_price;
    dutch_auction.current_price = starting_price;
    dutch_auction.currency = currency;
    dutch_auction.start_time = clock.unix_timestamp;
    dutch_auction.end_time = clock.unix_timestamp + duration;
    dutch_auction.price_drop_interval = price_drop_interval;
    dutch_auction.is_active = true;
    dutch_auction.bump = *ctx.bumps.get("dutch_auction").unwrap();

    // Calculate price drop per interval
    let total_intervals = duration / price_drop_interval;
    dutch_auction.price_drop_per_interval = (starting_price - ending_price) / total_intervals as u64;

    marketplace.total_dutch_auctions += 1;

    emit!(DutchAuctionCreated {
        dutch_auction: dutch_auction.key(),
        seller: seller.key(),
        nft_mint,
        starting_price,
        ending_price,
        duration,
        timestamp: clock.unix_timestamp,
    });

    Ok(())
}

/// Buy from Dutch Auction
#[derive(Accounts)]
#[instruction(nft_mint: Pubkey)]
pub struct BuyDutchAuction<'info> {
    #[account(mut)]
    pub buyer: Signer<'info>,

    /// CHECK: Seller account for payment
    #[account(mut)]
    pub seller: AccountInfo<'info>,

    #[account(
        mut,
        seeds = [b"dutch_auction", nft_mint.as_ref()],
        bump = dutch_auction.bump,
        constraint = dutch_auction.is_active @ MarketplaceError::AuctionNotActive
    )]
    pub dutch_auction: Account<'info, DutchAuction>,

    #[account(
        mut,
        seeds = [b"marketplace"],
        bump = marketplace.bump
    )]
    pub marketplace: Account<'info, Marketplace>,

    pub system_program: Program<'info, System>,
}

pub fn buy_dutch_auction(ctx: Context<BuyDutchAuction>, nft_mint: Pubkey) -> Result<()> {
    let dutch_auction = &mut ctx.accounts.dutch_auction;
    let marketplace = &mut ctx.accounts.marketplace;
    let buyer = &ctx.accounts.buyer;
    let seller = &ctx.accounts.seller;
    let clock = Clock::get()?;

    // Check if auction is still active
    require!(clock.unix_timestamp <= dutch_auction.end_time, MarketplaceError::AuctionExpired);

    // Calculate current price based on time elapsed
    let time_elapsed = clock.unix_timestamp - dutch_auction.start_time;
    let intervals_passed = time_elapsed / dutch_auction.price_drop_interval;
    
    let price_reduction = (intervals_passed as u64) * dutch_auction.price_drop_per_interval;
    dutch_auction.current_price = std::cmp::max(
        dutch_auction.starting_price.saturating_sub(price_reduction),
        dutch_auction.ending_price
    );

    let final_price = dutch_auction.current_price;

    // Calculate fees
    let trading_fee = (final_price * marketplace.trading_fee_basis_points as u64) / 10000;
    let seller_receives = final_price - trading_fee;

    // Transfer payment from buyer to seller
    match dutch_auction.currency {
        Currency::Sol => {
            // Transfer SOL
            **buyer.to_account_info().try_borrow_mut_lamports()? -= final_price;
            **seller.try_borrow_mut_lamports()? += seller_receives;
            **marketplace.to_account_info().try_borrow_mut_lamports()? += trading_fee;
        }
        Currency::Fin => {
            // Would implement FIN token transfer in real implementation
            msg!("FIN token transfer not implemented in this example");
        }
    }

    // Mark auction as completed
    dutch_auction.is_active = false;

    // Update marketplace statistics
    marketplace.total_sales += 1;
    marketplace.total_volume += final_price;

    emit!(DutchAuctionCompleted {
        dutch_auction: dutch_auction.key(),
        buyer: buyer.key(),
        seller: dutch_auction.seller,
        nft_mint,
        final_price,
        timestamp: clock.unix_timestamp,
    });

    Ok(())
}

/// Reserve Price Auction: Create auction with reserve price
#[derive(Accounts)]
#[instruction(nft_mint: Pubkey)]
pub struct CreateReserveAuction<'info> {
    #[account(mut)]
    pub seller: Signer<'info>,

    #[account(
        init,
        payer = seller,
        space = ReserveAuction::LEN,
        seeds = [b"reserve_auction", nft_mint.as_ref()],
        bump
    )]
    pub reserve_auction: Account<'info, ReserveAuction>,

    #[account(
        mut,
        seeds = [b"marketplace"],
        bump = marketplace.bump
    )]
    pub marketplace: Account<'info, Marketplace>,

    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

pub fn create_reserve_auction(
    ctx: Context<CreateReserveAuction>,
    nft_mint: Pubkey,
    starting_bid: u64,
    reserve_price: u64,
    duration: i64,
    currency: Currency,
) -> Result<()> {
    let reserve_auction = &mut ctx.accounts.reserve_auction;
    let marketplace = &mut ctx.accounts.marketplace;
    let seller = &ctx.accounts.seller;
    let clock = Clock::get()?;

    // Validation
    require!(reserve_price >= starting_bid, MarketplaceError::ReservePriceTooLow);
    require!(duration > 0, MarketplaceError::InvalidDuration);
    require!(
        duration <= marketplace.max_listing_duration,
        MarketplaceError::ListingDurationTooLong
    );

    reserve_auction.seller = seller.key();
    reserve_auction.nft_mint = nft_mint;
    reserve_auction.starting_bid = starting_bid;
    reserve_auction.reserve_price = reserve_price;
    reserve_auction.current_highest_bid = 0;
    reserve_auction.highest_bidder = None;
    reserve_auction.currency = currency;
    reserve_auction.start_time = clock.unix_timestamp;
    reserve_auction.end_time = clock.unix_timestamp + duration;
    reserve_auction.is_active = true;
    reserve_auction.reserve_met = false;
    reserve_auction.bump = *ctx.bumps.get("reserve_auction").unwrap();

    marketplace.total_reserve_auctions += 1;

    emit!(ReserveAuctionCreated {
        reserve_auction: reserve_auction.key(),
        seller: seller.key(),
        nft_mint,
        starting_bid,
        reserve_price,
        duration,
        timestamp: clock.unix_timestamp,
    });

    Ok(())
}

/// Update Marketplace Settings: Admin function to update marketplace parameters
#[derive(Accounts)]
pub struct UpdateMarketplaceSettings<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    #[account(
        mut,
        seeds = [b"marketplace"],
        bump = marketplace.bump,
        has_one = admin @ MarketplaceError::UnauthorizedAdmin
    )]
    pub marketplace: Account<'info, Marketplace>,
}

pub fn update_marketplace_settings(
    ctx: Context<UpdateMarketplaceSettings>,
    new_trading_fee: Option<u16>,
    new_max_listing_duration: Option<i64>,
    new_min_bid_increment: Option<u64>,
    new_treasury: Option<Pubkey>,
) -> Result<()> {
    let marketplace = &mut ctx.accounts.marketplace;
    let clock = Clock::get()?;

    if let Some(fee) = new_trading_fee {
        require!(fee <= 1000, MarketplaceError::TradingFeeTooHigh); // Max 10%
        marketplace.trading_fee_basis_points = fee;
    }

    if let Some(duration) = new_max_listing_duration {
        require!(duration > 0, MarketplaceError::InvalidDuration);
        require!(duration <= 365 * 24 * 60 * 60, MarketplaceError::ListingDurationTooLong); // Max 1 year
        marketplace.max_listing_duration = duration;
    }

    if let Some(increment) = new_min_bid_increment {
        require!(increment > 0, MarketplaceError::InvalidBidIncrement);
        marketplace.min_bid_increment = increment;
    }

    if let Some(treasury) = new_treasury {
        marketplace.treasury = treasury;
    }

    emit!(MarketplaceSettingsUpdated {
        admin: ctx.accounts.admin.key(),
        trading_fee: marketplace.trading_fee_basis_points,
        max_listing_duration: marketplace.max_listing_duration,
        min_bid_increment: marketplace.min_bid_increment,
        treasury: marketplace.treasury,
        timestamp: clock.unix_timestamp,
    });

    Ok(())
}

/// Emergency Pause: Admin function to pause marketplace operations
#[derive(Accounts)]
pub struct EmergencyPause<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    #[account(
        mut,
        seeds = [b"marketplace"],
        bump = marketplace.bump,
        has_one = admin @ MarketplaceError::UnauthorizedAdmin
    )]
    pub marketplace: Account<'info, Marketplace>,
}

pub fn emergency_pause(ctx: Context<EmergencyPause>, paused: bool) -> Result<()> {
    let marketplace = &mut ctx.accounts.marketplace;
    let clock = Clock::get()?;

    marketplace.is_paused = paused;

    emit!(MarketplacePaused {
        admin: ctx.accounts.admin.key(),
        paused,
        timestamp: clock.unix_timestamp,
    });

    Ok(())
}

/// Get Marketplace Analytics: View function for marketplace statistics
#[derive(Accounts)]
pub struct GetMarketplaceAnalytics<'info> {
    #[account(
        seeds = [b"marketplace"],
        bump = marketplace.bump
    )]
    pub marketplace: Account<'info, Marketplace>,
}

pub fn get_marketplace_analytics(ctx: Context<GetMarketplaceAnalytics>) -> Result<MarketplaceAnalytics> {
    let marketplace = &ctx.accounts.marketplace;
    
    Ok(MarketplaceAnalytics {
        total_listings: marketplace.total_listings,
        total_sales: marketplace.total_sales,
        total_volume: marketplace.total_volume,
        total_users: marketplace.total_users,
        total_collections: marketplace.total_collections,
        verified_collections: marketplace.verified_collections,
        total_dutch_auctions: marketplace.total_dutch_auctions,
        total_reserve_auctions: marketplace.total_reserve_auctions,
        average_sale_price: if marketplace.total_sales > 0 {
            marketplace.total_volume / marketplace.total_sales
        } else {
            0
        },
        trading_fee_basis_points: marketplace.trading_fee_basis_points,
        is_paused: marketplace.is_paused,
    })
}

/// Creator Dashboard: Get creator-specific analytics
#[derive(Accounts)]
pub struct GetCreatorAnalytics<'info> {
    pub creator: Signer<'info>,

    #[account(
        seeds = [b"user_profile", creator.key().as_ref()],
        bump = user_profile.bump
    )]
    pub user_profile: Account<'info, UserProfile>,
}

pub fn get_creator_analytics(ctx: Context<GetCreatorAnalytics>) -> Result<CreatorAnalytics> {
    let user_profile = &ctx.accounts.user_profile;
    
    Ok(CreatorAnalytics {
        listings_created: user_profile.listings_created,
        items_sold: user_profile.items_sold,
        total_earnings: user_profile.total_earnings,
        purchases_made: user_profile.purchases_made,
        total_spent: user_profile.total_spent,
        reputation_score: user_profile.reputation_score,
        verified_creator: user_profile.verified_creator,
        total_volume: user_profile.total_volume,
        success_rate: if user_profile.listings_created > 0 {
            (user_profile.items_sold * 100) / user_profile.listings_created
        } else {
            0
        },
    })
}

/// Integration Helper: Get listing data for external systems
#[derive(Accounts)]
pub struct GetListingData<'info> {
    #[account(
        seeds = [b"listing", listing.nft_mint.as_ref()],
        bump = listing.bump
    )]
    pub listing: Account<'info, Listing>,
}

pub fn get_listing_data(ctx: Context<GetListingData>) -> Result<ListingData> {
    let listing = &ctx.accounts.listing;
    let clock = Clock::get()?;
    
    Ok(ListingData {
        seller: listing.seller,
        nft_mint: listing.nft_mint,
        price: listing.price,
        currency: listing.currency.clone(),
        listing_type: listing.listing_type.clone(),
        created_at: listing.created_at,
        expires_at: listing.expires_at,
        is_active: listing.is_active && clock.unix_timestamp <= listing.expires_at,
        highest_bid: listing.highest_bid,
        highest_bidder: listing.highest_bidder,
        special_card_boost: listing.special_card_boost,
    })
}

/// Bulk Operations: Update multiple listing prices
#[derive(Accounts)]
pub struct BulkUpdatePrices<'info> {
    #[account(mut)]
    pub seller: Signer<'info>,

    #[account(
        seeds = [b"marketplace"],
        bump = marketplace.bump
    )]
    pub marketplace: Account<'info, Marketplace>,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct PriceUpdate {
    pub listing_key: Pubkey,
    pub new_price: u64,
}

pub fn bulk_update_prices(
    ctx: Context<BulkUpdatePrices>,
    updates: Vec<PriceUpdate>,
) -> Result<()> {
    let seller = &ctx.accounts.seller;
    let marketplace = &ctx.accounts.marketplace;
    let clock = Clock::get()?;

    require!(!marketplace.is_paused, MarketplaceError::MarketplacePaused);
    require!(updates.len() <= 50, MarketplaceError::BatchSizeTooLarge);
    require!(!updates.is_empty(), MarketplaceError::EmptyBatch);

    let mut successful_updates = 0u32;

    for update in updates.iter() {
        require!(update.new_price > 0, MarketplaceError::InvalidPrice);
        
        // In real implementation, you would load each listing account and update
        // Here we just track successful operations
        successful_updates += 1;
    }

    emit!(BulkPriceUpdate {
        seller: seller.key(),
        count: successful_updates,
        timestamp: clock.unix_timestamp,
    });

    Ok(())
}

/// Feature Toggle: Admin function to enable/disable marketplace features
#[derive(Accounts)]
pub struct ToggleFeature<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    #[account(
        mut,
        seeds = [b"marketplace"],
        bump = marketplace.bump,
        has_one = admin @ MarketplaceError::UnauthorizedAdmin
    )]
    pub marketplace: Account<'info, Marketplace>,
}

pub fn toggle_feature(
    ctx: Context<ToggleFeature>,
    feature: MarketplaceFeature,
    enabled: bool,
) -> Result<()> {
    let marketplace = &mut ctx.accounts.marketplace;
    let clock = Clock::get()?;

    match feature {
        MarketplaceFeature::Auctions => {
            marketplace.features.auctions_enabled = enabled;
        }
        MarketplaceFeature::Offers => {
            marketplace.features.offers_enabled = enabled;
        }
        MarketplaceFeature::DutchAuctions => {
            marketplace.features.dutch_auctions_enabled = enabled;
        }
        MarketplaceFeature::ReserveAuctions => {
            marketplace.features.reserve_auctions_enabled = enabled;
        }
        MarketplaceFeature::BatchOperations => {
            marketplace.features.batch_operations_enabled = enabled;
        }
        MarketplaceFeature::SpecialCards => {
            marketplace.features.special_cards_enabled = enabled;
        }
    }

    emit!(FeatureToggled {
        admin: ctx.accounts.admin.key(),
        feature,
        enabled,
        timestamp: clock.unix_timestamp,
    });

    Ok(())
}

/// Advanced Search: Get filtered listings
pub fn get_filtered_listings(
    price_min: Option<u64>,
    price_max: Option<u64>,
    currency: Option<Currency>,
    listing_type: Option<ListingType>,
    collection: Option<Pubkey>,
    verified_only: bool,
) -> Result<Vec<Pubkey>> {
    // In a real implementation, this would query the blockchain state
    // and return filtered listing pubkeys based on the criteria
    // For now, we return an empty vector as a placeholder
    Ok(vec![])
}

/// Royalty Distribution: Calculate and distribute creator royalties
#[derive(Accounts)]
pub struct DistributeRoyalties<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    /// CHECK: Creator account to receive royalties
    #[account(mut)]
    pub creator: AccountInfo<'info>,

    #[account(
        seeds = [b"collection", collection.creator.as_ref(), collection.name.as_bytes()],
        bump = collection.bump
    )]
    pub collection: Account<'info, Collection>,

    #[account(
        mut,
        seeds = [b"marketplace"],
        bump = marketplace.bump
    )]
    pub marketplace: Account<'info, Marketplace>,

    pub system_program: Program<'info, System>,
}

pub fn distribute_royalties(
    ctx: Context<DistributeRoyalties>,
    sale_amount: u64,
) -> Result<()> {
    let collection = &ctx.accounts.collection;
    let marketplace = &mut ctx.accounts.marketplace;
    let creator = &ctx.accounts.creator;
    let payer = &ctx.accounts.payer;
    let clock = Clock::get()?;

    // Calculate royalty amount
    let royalty_amount = (sale_amount * collection.creator_fee_basis_points as u64) / 10000;
    
    require!(royalty_amount > 0, MarketplaceError::NoRoyaltyToPay);

    // Transfer royalty to creator
    **payer.to_account_info().try_borrow_mut_lamports()? -= royalty_amount;
    **creator.try_borrow_mut_lamports()? += royalty_amount;

    // Update marketplace statistics
    marketplace.total_royalties_paid += royalty_amount;

    emit!(RoyaltiesDistributed {
        collection: collection.key(),
        creator: collection.creator,
        amount: royalty_amount,
        sale_amount,
        timestamp: clock.unix_timestamp,
    });

    Ok(())
}

/// Reputation System: Update user reputation based on trading activity
#[derive(Accounts)]
pub struct UpdateReputation<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        mut,
        seeds = [b"user_profile", user.as_ref()],
        bump = user_profile.bump
    )]
    pub user_profile: Account<'info, UserProfile>,

    #[account(
        seeds = [b"marketplace"],
        bump = marketplace.bump
    )]
    pub marketplace: Account<'info, Marketplace>,
}

pub fn update_reputation(
    ctx: Context<UpdateReputation>,
    user: Pubkey,
    action: ReputationAction,
    impact: i32,
) -> Result<()> {
    let user_profile = &mut ctx.accounts.user_profile;
    let clock = Clock::get()?;

    // Calculate reputation change based on action type
    let reputation_change = match action {
        ReputationAction::SuccessfulSale => impact.max(0),
        ReputationAction::SuccessfulPurchase => (impact / 2).max(0),
        ReputationAction::CancelledListing => -impact.abs() / 4,
        ReputationAction::DisputeResolved => impact,
        ReputationAction::DisputeLost => -impact.abs(),
        ReputationAction::VerifiedCreator => 100,
        ReputationAction::CommunityReport => -impact.abs() / 2,
    };

    // Update reputation with bounds checking
    let new_reputation = (user_profile.reputation_score as i32 + reputation_change)
        .max(0)
        .min(1000) as u32;
    
    user_profile.reputation_score = new_reputation;
    user_profile.last_activity = clock.unix_timestamp;

    emit!(ReputationUpdated {
        user,
        action,
        old_score: (new_reputation as i32 - reputation_change).max(0) as u32,
        new_score: new_reputation,
        change: reputation_change,
        timestamp: clock.unix_timestamp,
    });

    Ok(())
}

/// Special Card Integration: Apply special card effects to transactions
#[derive(Accounts)]
pub struct ApplySpecialCard<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        mut,
        constraint = special_card.owner == user.key() @ MarketplaceError::UnauthorizedCardUse,
        constraint = special_card.uses_remaining > 0 @ MarketplaceError::CardExhausted,
        constraint = special_card.expires_at > Clock::get()?.unix_timestamp @ MarketplaceError::CardExpired
    )]
    pub special_card: Account<'info, SpecialCard>,

    #[account(
        mut,
        seeds = [b"user_profile", user.key().as_ref()],
        bump = user_profile.bump
    )]
    pub user_profile: Account<'info, UserProfile>,
}

pub fn apply_special_card(
    ctx: Context<ApplySpecialCard>,
    transaction_type: TransactionType,
    base_amount: u64,
) -> Result<CardEffect> {
    let special_card = &mut ctx.accounts.special_card;
    let user_profile = &mut ctx.accounts.user_profile;
    let clock = Clock::get()?;

    let effect = match special_card.card_type {
        SpecialCardType::TradingFeeDiscount => {
            if matches!(transaction_type, TransactionType::Purchase | TransactionType::Sale) {
                CardEffect {
                    discount_percentage: special_card.effect_value,
                    bonus_xp: 0,
                    fee_reduction: (base_amount * special_card.effect_value as u64) / 10000,
                }
            } else {
                return Err(MarketplaceError::InvalidCardUsage.into());
            }
        }
        SpecialCardType::XpBoost => {
            CardEffect {
                discount_percentage: 0,
                bonus_xp: special_card.effect_value as u64,
                fee_reduction: 0,
            }
        }
        SpecialCardType::RoyaltyBoost => {
            if matches!(transaction_type, TransactionType::Sale) {
                CardEffect {
                    discount_percentage: 0,
                    bonus_xp: 25,
                    fee_reduction: 0,
                }
            } else {
                return Err(MarketplaceError::InvalidCardUsage.into());
            }
        }
        SpecialCardType::ListingBoost => {
            if matches!(transaction_type, TransactionType::Listing) {
                CardEffect {
                    discount_percentage: 0,
                    bonus_xp: 50,
                    fee_reduction: 0,
                }
            } else {
                return Err(MarketplaceError::InvalidCardUsage.into());
            }
        }
    };

    // Consume card usage
    special_card.uses_remaining -= 1;
    special_card.last_used = clock.unix_timestamp;

    // Update user profile with card usage
    user_profile.special_cards_used += 1;

    emit!(SpecialCardUsed {
        user: ctx.accounts.user.key(),
        card: special_card.key(),
        card_type: special_card.card_type.clone(),
        effect_value: special_card.effect_value,
        uses_remaining: special_card.uses_remaining,
        timestamp: clock.unix_timestamp,
    });

    Ok(effect)
}

/// Price Oracle Integration: Get current market prices
#[derive(Accounts)]
pub struct GetMarketPrice<'info> {
    #[account(
        seeds = [b"marketplace"],
        bump = marketplace.bump
    )]
    pub marketplace: Account<'info, Marketplace>,

    /// CHECK: Oracle account for price data
    pub price_oracle: AccountInfo<'info>,
}

pub fn get_market_price(
    ctx: Context<GetMarketPrice>,
    currency_pair: CurrencyPair,
) -> Result<PriceData> {
    let marketplace = &ctx.accounts.marketplace;
    let clock = Clock::get()?;

    // In a real implementation, this would fetch from Chainlink or Pyth oracle
    let mock_price = match currency_pair {
        CurrencyPair::FinSol => PriceData {
            price: 100_000_000, // 0.1 SOL per FIN
            confidence: 95,
            last_updated: clock.unix_timestamp,
            valid: true,
        },
        CurrencyPair::SolUsdc => PriceData {
            price: 100_000_000_000, // $100 per SOL
            confidence: 99,
            last_updated: clock.unix_timestamp,
            valid: true,
        },
    };

    Ok(mock_price)
}

/// Analytics Aggregator: Calculate marketplace metrics
pub fn calculate_marketplace_metrics(
    total_volume: u64,
    total_sales: u64,
    total_listings: u64,
    time_period: i64,
) -> MarketplaceMetrics {
    MarketplaceMetrics {
        volume_24h: total_volume, // Simplified - would need time-based filtering
        sales_24h: total_sales,
        average_price: if total_sales > 0 {
            total_volume / total_sales
        } else {
            0
        },
        listings_to_sales_ratio: if total_sales > 0 {
            (total_listings * 100) / total_sales
        } else {
            0
        },
        velocity: if time_period > 0 {
            total_sales / (time_period / 86400) as u64 // Sales per day
        } else {
            0
        },
    }
}

/// Collection Analytics: Get collection-specific metrics
#[derive(Accounts)]
pub struct GetCollectionMetrics<'info> {
    #[account(
        seeds = [b"collection", collection.creator.as_ref(), collection.name.as_bytes()],
        bump = collection.bump
    )]
    pub collection: Account<'info, Collection>,
}

pub fn get_collection_metrics(ctx: Context<GetCollectionMetrics>) -> Result<CollectionMetrics> {
    let collection = &ctx.accounts.collection;
    
    // In a real implementation, these metrics would be calculated from
    // aggregated transaction data. For now, we provide mock data.
    Ok(CollectionMetrics {
        floor_price: 1_000_000_000, // 1 SOL
        total_volume: 100_000_000_000, // 100 SOL
        total_sales: 50,
        unique_holders: 25,
        listed_count: 10,
        average_hold_time: 86400 * 30, // 30 days
        creator_earnings: 5_000_000_000, // 5 SOL in royalties
    })
}

/// Trending Collections: Get trending collections based on volume/activity
pub fn get_trending_collections(
    time_period: TrendingPeriod,
    limit: u8,
) -> Result<Vec<TrendingCollection>> {
    // Mock trending data - in reality would query and sort by metrics
    let trending = vec![
        TrendingCollection {
            collection: Pubkey::default(), // Would be actual collection pubkey
            volume_change: 150, // 150% increase
            sales_count: 25,
            floor_price_change: 120, // 20% increase
        },
        TrendingCollection {
            collection: Pubkey::default(),
            volume_change: 130,
            sales_count: 18,
            floor_price_change: 110,
        },
    ];

    Ok(trending.into_iter().take(limit as usize).collect())
}

/// User Activity Feed: Get user's recent marketplace activities
#[derive(Accounts)]
pub struct GetUserActivity<'info> {
    pub user: Signer<'info>,

    #[account(
        seeds = [b"user_profile", user.key().as_ref()],
        bump = user_profile.bump
    )]
    pub user_profile: Account<'info, UserProfile>,
}

pub fn get_user_activity(
    ctx: Context<GetUserActivity>,
    limit: u8,
) -> Result<Vec<UserActivity>> {
    let user = &ctx.accounts.user;
    let clock = Clock::get()?;

    // Mock activity data - in reality would query transaction history
    let activities = vec![
        UserActivity {
            activity_type: ActivityType::Purchase,
            nft_mint: Pubkey::default(),
            amount: 2_000_000_000, // 2 SOL
            timestamp: clock.unix_timestamp - 3600, // 1 hour ago
            counterparty: Some(Pubkey::default()),
        },
        UserActivity {
            activity_type: ActivityType::Listing,
            nft_mint: Pubkey::default(),
            amount: 1_500_000_000, // 1.5 SOL
            timestamp: clock.unix_timestamp - 7200, // 2 hours ago
            counterparty: None,
        },
    ];

    Ok(activities.into_iter().take(limit as usize).collect())
}

/// Dispute Resolution: Handle marketplace disputes
#[derive(Accounts)]
pub struct ResolveDispute<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    #[account(
        mut,
        seeds = [b"dispute", dispute.transaction_id.as_ref()],
        bump = dispute.bump
    )]
    pub dispute: Account<'info, Dispute>,

    #[account(
        mut,
        seeds = [b"marketplace"],
        bump = marketplace.bump,
        has_one = admin @ MarketplaceError::UnauthorizedAdmin
    )]
    pub marketplace: Account<'info, Marketplace>,
}

pub fn resolve_dispute(
    ctx: Context<ResolveDispute>,
    resolution: DisputeResolution,
    refund_buyer: bool,
    refund_amount: u64,
) -> Result<()> {
    let dispute = &mut ctx.accounts.dispute;
    let marketplace = &mut ctx.accounts.marketplace;
    let clock = Clock::get()?;

    require!(dispute.status == DisputeStatus::Open, MarketplaceError::DisputeAlreadyResolved);

    dispute.status = DisputeStatus::Resolved;
    dispute.resolution = Some(resolution.clone());
    dispute.resolved_at = Some(clock.unix_timestamp);
    dispute.resolved_by = Some(ctx.accounts.admin.key());

    if refund_buyer && refund_amount > 0 {
        // In real implementation, would transfer funds back to buyer
        dispute.refund_amount = Some(refund_amount);
    }

    marketplace.total_disputes_resolved += 1;

    emit!(DisputeResolved {
        dispute: dispute.key(),
        transaction_id: dispute.transaction_id,
        resolution,
        refund_buyer,
        refund_amount,
        resolved_by: ctx.accounts.admin.key(),
        timestamp: clock.unix_timestamp,
    });

    Ok(())
}

/// Marketplace Health Check: System status verification
pub fn marketplace_health_check(
    marketplace: &Account<Marketplace>,
) -> Result<HealthStatus> {
    let clock = Clock::get()?;
    
    let status = HealthStatus {
        is_operational: !marketplace.is_paused,
        total_users: marketplace.total_users,
        total_listings: marketplace.total_listings,
        total_volume_24h: marketplace.total_volume, // Simplified
        last_sale_timestamp: clock.unix_timestamp, // Mock data
        system_load: 75, // Mock system load percentage
        average_response_time: 150, // Mock response time in ms
    };

    Ok(status)
}

/// Data Export: Export marketplace data for analytics
pub fn export_marketplace_data(
    start_time: i64,
    end_time: i64,
    data_type: ExportDataType,
) -> Result<ExportResult> {
    require!(start_time < end_time, MarketplaceError::InvalidTimeRange);
    require!(
        end_time - start_time <= 86400 * 30, // Max 30 days
        MarketplaceError::ExportRangeTooLarge
    );

    let export_id = Clock::get()?.unix_timestamp;
    
    Ok(ExportResult {
        export_id: export_id.to_string(),
        data_type,
        start_time,
        end_time,
        status: ExportStatus::Processing,
        download_url: None,
        estimated_completion: Clock::get()?.unix_timestamp + 300, // 5 minutes
    })
}

/// Integration Testing Helper: Create test data
#[cfg(feature = "test")]
pub fn create_test_marketplace_data(
    marketplace: &mut Account<Marketplace>,
    num_users: u32,
    num_listings: u32,
) -> Result<()> {
    // Only available in test builds
    marketplace.total_users = num_users as u64;
    marketplace.total_listings = num_listings as u64;
    marketplace.total_volume = (num_listings as u64) * 1_000_000_000; // Mock volume
    marketplace.total_sales = num_listings as u64 / 2; // 50% sell rate
    
    Ok(())
}

/// Migration Helper: Upgrade marketplace data structures
pub fn migrate_marketplace_data(
    marketplace: &mut Account<Marketplace>,
    migration_version: u8,
) -> Result<()> {
    match migration_version {
        1 => {
            // Migration logic for version 1
            marketplace.total_dutch_auctions = 0;
            marketplace.total_reserve_auctions = 0;
        }
        2 => {
            // Migration logic for version 2
            marketplace.total_royalties_paid = 0;
            marketplace.verified_collections = 0;
        }
        _ => return Err(MarketplaceError::UnsupportedMigration.into()),
    }

    Ok(())
}
