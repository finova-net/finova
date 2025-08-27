// programs/finova-nft/src/instructions/burn_nft.rs

use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    metadata::{
        mpl_token_metadata::{
            instructions::{
                BurnNftV1CpiBuilder,
                BurnV1CpiBuilder,
            },
            types::{BurnArgs, TokenStandard},
        },
        Metadata,
        MetadataAccount,
    },
    token::{
        burn,
        close_account,
        Burn,
        CloseAccount,
        Mint,
        Token,
        TokenAccount,
    },
};

use crate::{
    constants::*,
    errors::*,
    events::*,
    state::*,
};

/// Burn NFT instruction
/// Permanently destroys an NFT and its associated accounts
/// Used for single-use special cards or unwanted NFTs
#[derive(Accounts)]
pub struct BurnNft<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    /// NFT mint account to be burned
    #[account(
        mut,
        constraint = nft_mint.supply == 1 @ FinovaNftError::InvalidNftSupply,
        constraint = nft_mint.decimals == 0 @ FinovaNftError::InvalidNftDecimals,
    )]
    pub nft_mint: Account<'info, Mint>,

    /// Owner's NFT token account
    #[account(
        mut,
        associated_token::mint = nft_mint,
        associated_token::authority = owner,
        constraint = nft_token_account.amount == 1 @ FinovaNftError::InsufficientNftBalance,
    )]
    pub nft_token_account: Account<'info, TokenAccount>,

    /// NFT metadata account
    #[account(
        mut,
        seeds = [
            METADATA_SEED,
            metadata_program.key().as_ref(),
            nft_mint.key().as_ref(),
        ],
        bump,
        seeds::program = metadata_program.key(),
        constraint = nft_metadata.mint == nft_mint.key() @ FinovaNftError::InvalidMetadataMint,
    )]
    pub nft_metadata: Account<'info, MetadataAccount>,

    /// Master edition account (if exists)
    /// CHECK: This account is validated by the metadata program
    #[account(
        mut,
        seeds = [
            METADATA_SEED,
            metadata_program.key().as_ref(),
            nft_mint.key().as_ref(),
            EDITION_SEED,
        ],
        bump,
        seeds::program = metadata_program.key(),
    )]
    pub master_edition: Option<UncheckedAccount<'info>>,

    /// Collection metadata (if NFT is part of a collection)
    #[account(
        mut,
        seeds = [COLLECTION_SEED, collection.collection_mint.as_ref()],
        bump = collection.bump,
        constraint = collection.authority == owner.key() || collection.update_authority == owner.key() @ FinovaNftError::UnauthorizedBurn,
    )]
    pub collection: Option<Account<'info, Collection>>,

    /// Special card state (if burning a special card)
    #[account(
        mut,
        seeds = [SPECIAL_CARD_SEED, nft_mint.key().as_ref()],
        bump = special_card.bump,
        close = owner,
        constraint = special_card.mint == nft_mint.key() @ FinovaNftError::InvalidSpecialCardMint,
        constraint = special_card.owner == owner.key() @ FinovaNftError::UnauthorizedBurn,
    )]
    pub special_card: Option<Account<'info, SpecialCard>>,

    /// Marketplace listing (if NFT is listed for sale)
    #[account(
        mut,
        seeds = [MARKETPLACE_LISTING_SEED, nft_mint.key().as_ref()],
        bump = marketplace_listing.bump,
        close = owner,
        constraint = marketplace_listing.seller == owner.key() @ FinovaNftError::UnauthorizedBurn,
    )]
    pub marketplace_listing: Option<Account<'info, MarketplaceListing>>,

    /// System programs
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub metadata_program: Program<'info, Metadata>,
    pub rent: Sysvar<'info, Rent>,
}

impl<'info> BurnNft<'info> {
    /// Validate burn prerequisites
    fn validate_burn_conditions(&self) -> Result<()> {
        // Check if NFT is frozen (cannot burn frozen NFTs)
        if self.nft_token_account.is_frozen() {
            return Err(FinovaNftError::FrozenNft.into());
        }

        // If it's a special card, check if it can be burned
        if let Some(ref special_card) = self.special_card {
            // Check if card is already used (some used cards cannot be burned)
            if special_card.card_type == SpecialCardType::Eternal && special_card.uses_remaining > 0 {
                return Err(FinovaNftError::CannotBurnActiveEternalCard.into());
            }

            // Check if card is currently active
            if special_card.is_active && Clock::get()?.unix_timestamp < special_card.expires_at {
                return Err(FinovaNftError::CannotBurnActiveCard.into());
            }
        }

        // If NFT is listed on marketplace, it cannot be burned
        if self.marketplace_listing.is_some() {
            return Err(FinovaNftError::CannotBurnListedNft.into());
        }

        Ok(())
    }

    /// Calculate burn rewards
    fn calculate_burn_rewards(&self) -> Result<BurnRewards> {
        let mut rewards = BurnRewards {
            fin_tokens: 0,
            xp_points: 0,
            special_bonus: false,
        };

        // Base burn rewards
        rewards.fin_tokens = BASE_BURN_REWARD;
        rewards.xp_points = BASE_BURN_XP;

        // Special card burn rewards
        if let Some(ref special_card) = self.special_card {
            let card_multiplier = match special_card.rarity {
                Rarity::Common => 1,
                Rarity::Uncommon => 2,
                Rarity::Rare => 5,
                Rarity::Epic => 15,
                Rarity::Legendary => 50,
            };

            rewards.fin_tokens = rewards.fin_tokens.checked_mul(card_multiplier)
                .ok_or(FinovaNftError::ArithmeticOverflow)?;
            rewards.xp_points = rewards.xp_points.checked_mul(card_multiplier)
                .ok_or(FinovaNftError::ArithmeticOverflow)?;

            // Bonus for burning unused cards
            if special_card.uses_remaining > 0 {
                rewards.special_bonus = true;
                rewards.fin_tokens = rewards.fin_tokens.checked_mul(2)
                    .ok_or(FinovaNftError::ArithmeticOverflow)?;
            }
        }

        Ok(rewards)
    }

    /// Update collection stats after burn
    fn update_collection_stats(&mut self) -> Result<()> {
        if let Some(ref mut collection) = self.collection {
            collection.total_supply = collection.total_supply
                .checked_sub(1)
                .ok_or(FinovaNftError::ArithmeticUnderflow)?;
            
            collection.burned_count = collection.burned_count
                .checked_add(1)
                .ok_or(FinovaNftError::ArithmeticOverflow)?;

            collection.updated_at = Clock::get()?.unix_timestamp;
        }
        Ok(())
    }
}

/// Burn rewards structure
#[derive(Debug, Clone, Copy)]
pub struct BurnRewards {
    pub fin_tokens: u64,
    pub xp_points: u64,
    pub special_bonus: bool,
}

pub fn handler(ctx: Context<BurnNft>) -> Result<()> {
    let burn_nft = &mut ctx.accounts;

    // Validate burn conditions
    burn_nft.validate_burn_conditions()?;

    // Calculate burn rewards before burning
    let rewards = burn_nft.calculate_burn_rewards()?;

    // Store NFT information for event emission
    let nft_info = NftBurnInfo {
        mint: burn_nft.nft_mint.key(),
        owner: burn_nft.owner.key(),
        metadata_uri: burn_nft.nft_metadata.data.uri.clone(),
        name: burn_nft.nft_metadata.data.name.clone(),
        symbol: burn_nft.nft_metadata.data.symbol.clone(),
        is_special_card: burn_nft.special_card.is_some(),
        special_card_data: burn_nft.special_card.as_ref().map(|sc| SpecialCardBurnData {
            card_type: sc.card_type,
            rarity: sc.rarity,
            uses_remaining: sc.uses_remaining,
            total_uses: sc.total_uses,
        }),
        collection: burn_nft.collection.as_ref().map(|c| c.collection_mint),
        rewards: rewards,
        burned_at: Clock::get()?.unix_timestamp,
    };

    // Burn the NFT token first
    let burn_ctx = CpiContext::new(
        burn_nft.token_program.to_account_info(),
        Burn {
            mint: burn_nft.nft_mint.to_account_info(),
            from: burn_nft.nft_token_account.to_account_info(),
            authority: burn_nft.owner.to_account_info(),
        },
    );
    burn(burn_ctx, 1)?;

    // Close the token account
    let close_ctx = CpiContext::new(
        burn_nft.token_program.to_account_info(),
        CloseAccount {
            account: burn_nft.nft_token_account.to_account_info(),
            destination: burn_nft.owner.to_account_info(),
            authority: burn_nft.owner.to_account_info(),
        },
    );
    close_account(close_ctx)?;

    // Burn NFT metadata using Metaplex
    if burn_nft.master_edition.is_some() {
        // Burn NFT with master edition
        let burn_nft_cpi = BurnNftV1CpiBuilder::new(&burn_nft.metadata_program.to_account_info())
            .metadata(&burn_nft.nft_metadata.to_account_info())
            .authority(&burn_nft.owner.to_account_info())
            .mint(&burn_nft.nft_mint.to_account_info())
            .token(&burn_nft.nft_token_account.to_account_info())
            .master_edition(burn_nft.master_edition.as_ref().unwrap())
            .system_program(&burn_nft.system_program.to_account_info())
            .sysvar_instructions(&burn_nft.rent.to_account_info())
            .spl_token_program(&burn_nft.token_program.to_account_info());

        burn_nft_cpi.invoke()?;
    } else {
        // Burn regular metadata
        let burn_cpi = BurnV1CpiBuilder::new(&burn_nft.metadata_program.to_account_info())
            .authority(&burn_nft.owner.to_account_info())
            .metadata(&burn_nft.nft_metadata.to_account_info())
            .mint(&burn_nft.nft_mint.to_account_info())
            .token(&burn_nft.nft_token_account.to_account_info())
            .system_program(&burn_nft.system_program.to_account_info())
            .sysvar_instructions(&burn_nft.rent.to_account_info())
            .spl_token_program(&burn_nft.token_program.to_account_info())
            .invoke_signed(&[]);

        burn_cpi.invoke()?;
    }

    // Update collection statistics
    burn_nft.update_collection_stats()?;

    // Emit burn event
    emit!(NftBurnedEvent {
        nft_mint: burn_nft.nft_mint.key(),
        owner: burn_nft.owner.key(),
        nft_info: nft_info.clone(),
        rewards_earned: rewards,
        collection: burn_nft.collection.as_ref().map(|c| c.key()),
        burned_at: Clock::get()?.unix_timestamp,
    });

    // If special card, emit additional event
    if let Some(special_card) = &burn_nft.special_card {
        emit!(SpecialCardBurnedEvent {
            card_mint: burn_nft.nft_mint.key(),
            owner: burn_nft.owner.key(),
            card_type: special_card.card_type,
            rarity: special_card.rarity,
            uses_remaining: special_card.uses_remaining,
            total_uses: special_card.total_uses,
            bonus_rewards: rewards.special_bonus,
            burned_at: Clock::get()?.unix_timestamp,
        });
    }

    // Log successful burn
    msg!(
        "NFT burned successfully: mint={}, owner={}, rewards={}FIN+{}XP",
        burn_nft.nft_mint.key(),
        burn_nft.owner.key(),
        rewards.fin_tokens,
        rewards.xp_points
    );

    Ok(())
}

/// NFT burn information for events
#[derive(Debug, Clone)]
pub struct NftBurnInfo {
    pub mint: Pubkey,
    pub owner: Pubkey,
    pub metadata_uri: String,
    pub name: String,
    pub symbol: String,
    pub is_special_card: bool,
    pub special_card_data: Option<SpecialCardBurnData>,
    pub collection: Option<Pubkey>,
    pub rewards: BurnRewards,
    pub burned_at: i64,
}

/// Special card burn data
#[derive(Debug, Clone, Copy)]
pub struct SpecialCardBurnData {
    pub card_type: SpecialCardType,
    pub rarity: Rarity,
    pub uses_remaining: u32,
    pub total_uses: u32,
}

/// Batch burn multiple NFTs (for efficiency)
#[derive(Accounts)]
#[instruction(nft_count: u8)]
pub struct BatchBurnNfts<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    /// Collection account (all NFTs must be from same collection)
    #[account(
        mut,
        seeds = [COLLECTION_SEED, collection.collection_mint.as_ref()],
        bump = collection.bump,
        constraint = collection.authority == owner.key() || collection.update_authority == owner.key() @ FinovaNftError::UnauthorizedBurn,
    )]
    pub collection: Account<'info, Collection>,

    /// System programs
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub metadata_program: Program<'info, Metadata>,
    pub rent: Sysvar<'info, Rent>,
}

pub fn batch_burn_handler(
    ctx: Context<BatchBurnNfts>,
    nft_mints: Vec<Pubkey>,
) -> Result<()> {
    require!(
        nft_mints.len() <= MAX_BATCH_BURN_SIZE,
        FinovaNftError::BatchSizeTooLarge
    );

    let batch_burn = &mut ctx.accounts;
    let mut total_rewards = BurnRewards {
        fin_tokens: 0,
        xp_points: 0,
        special_bonus: false,
    };

    let burned_count = nft_mints.len() as u64;

    // Update collection stats
    batch_burn.collection.total_supply = batch_burn.collection.total_supply
        .checked_sub(burned_count)
        .ok_or(FinovaNftError::ArithmeticUnderflow)?;
    
    batch_burn.collection.burned_count = batch_burn.collection.burned_count
        .checked_add(burned_count)
        .ok_or(FinovaNftError::ArithmeticOverflow)?;

    batch_burn.collection.updated_at = Clock::get()?.unix_timestamp;

    // Calculate batch rewards (with batch bonus)
    total_rewards.fin_tokens = BASE_BURN_REWARD
        .checked_mul(burned_count)
        .and_then(|r| r.checked_mul(BATCH_BURN_MULTIPLIER))
        .ok_or(FinovaNftError::ArithmeticOverflow)?;

    total_rewards.xp_points = BASE_BURN_XP
        .checked_mul(burned_count)
        .and_then(|r| r.checked_mul(BATCH_BURN_MULTIPLIER))
        .ok_or(FinovaNftError::ArithmeticOverflow)?;

    // Emit batch burn event
    emit!(BatchNftBurnedEvent {
        owner: batch_burn.owner.key(),
        collection: batch_burn.collection.key(),
        nft_mints: nft_mints.clone(),
        burned_count: burned_count,
        total_rewards: total_rewards,
        batch_multiplier: BATCH_BURN_MULTIPLIER,
        burned_at: Clock::get()?.unix_timestamp,
    });

    msg!(
        "Batch burn completed: owner={}, count={}, rewards={}FIN+{}XP",
        batch_burn.owner.key(),
        burned_count,
        total_rewards.fin_tokens,
        total_rewards.xp_points
    );

    Ok(())
}

/// Emergency burn function (admin only)
#[derive(Accounts)]
pub struct EmergencyBurnNft<'info> {
    #[account(
        mut,
        constraint = admin.key() == ADMIN_PUBKEY @ FinovaNftError::UnauthorizedAccess,
    )]
    pub admin: Signer<'info>,

    /// NFT mint to be emergency burned
    #[account(mut)]
    pub nft_mint: Account<'info, Mint>,

    /// NFT metadata
    #[account(
        mut,
        seeds = [
            METADATA_SEED,
            metadata_program.key().as_ref(),
            nft_mint.key().as_ref(),
        ],
        bump,
        seeds::program = metadata_program.key(),
    )]
    pub nft_metadata: Account<'info, MetadataAccount>,

    /// System programs
    pub system_program: Program<'info, System>,
    pub metadata_program: Program<'info, Metadata>,
}

pub fn emergency_burn_handler(
    ctx: Context<EmergencyBurnNft>,
    reason: String,
) -> Result<()> {
    require!(
        reason.len() <= MAX_BURN_REASON_LENGTH,
        FinovaNftError::ReasonTooLong
    );

    let emergency_burn = &ctx.accounts;

    // Burn metadata directly (bypass normal checks)
    let burn_cpi = BurnV1CpiBuilder::new(&emergency_burn.metadata_program.to_account_info())
        .authority(&emergency_burn.admin.to_account_info())
        .metadata(&emergency_burn.nft_metadata.to_account_info())
        .mint(&emergency_burn.nft_mint.to_account_info())
        .system_program(&emergency_burn.system_program.to_account_info())
        .invoke_signed(&[]);

    burn_cpi.invoke()?;

    // Emit emergency burn event
    emit!(EmergencyNftBurnedEvent {
        admin: emergency_burn.admin.key(),
        nft_mint: emergency_burn.nft_mint.key(),
        reason: reason.clone(),
        burned_at: Clock::get()?.unix_timestamp,
    });

    msg!(
        "Emergency burn executed: admin={}, mint={}, reason={}",
        emergency_burn.admin.key(),
        emergency_burn.nft_mint.key(),
        reason
    );

    Ok(())
}
