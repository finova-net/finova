// programs/finova-nft/src/instructions/mint_nft.rs

use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    metadata::{
        create_master_edition_v3, create_metadata_accounts_v3, CreateMasterEditionV3,
        CreateMetadataAccountsV3, Metadata,
    },
    token::{mint_to, Mint, MintTo, Token, TokenAccount},
};
use mpl_token_metadata::{
    pda::{find_master_edition_account, find_metadata_account},
    state::{DataV2, Creator},
};

use crate::{
    constants::*,
    errors::*,
    state::{Collection, NFTMetadata, SpecialCard},
    utils::*,
};

#[derive(Accounts)]
#[instruction(
    name: String,
    symbol: String,
    uri: String,
    card_type: u8,
    rarity: u8,
    effect_type: u8,
    effect_value: u64,
    duration: i64,
    collection_id: String
)]
pub struct MintNFT<'info> {
    #[account(
        init,
        payer = payer,
        mint::decimals = 0,
        mint::authority = mint_authority.key(),
        mint::freeze_authority = mint_authority.key(),
    )]
    pub mint: Account<'info, Mint>,

    #[account(
        init_if_needed,
        payer = payer,
        seeds = [b"collection", collection_id.as_bytes()],
        bump,
        space = Collection::SPACE
    )]
    pub collection: Account<'info, Collection>,

    #[account(
        init,
        payer = payer,
        seeds = [b"nft_metadata", mint.key().as_ref()],
        bump,
        space = NFTMetadata::SPACE
    )]
    pub nft_metadata: Account<'info, NFTMetadata>,

    #[account(
        init_if_needed,
        payer = payer,
        seeds = [b"special_card", mint.key().as_ref()],
        bump,
        space = SpecialCard::SPACE
    )]
    pub special_card: Account<'info, SpecialCard>,

    #[account(
        init_if_needed,
        payer = payer,
        associated_token::mint = mint,
        associated_token::authority = recipient
    )]
    pub token_account: Account<'info, TokenAccount>,

    /// CHECK: This is not dangerous because we don't read or write from this account
    #[account(
        mut,
        seeds = [b"metadata", token_metadata_program.key().as_ref(), mint.key().as_ref()],
        bump,
        seeds::program = token_metadata_program.key()
    )]
    pub metadata: UncheckedAccount<'info>,

    /// CHECK: This is not dangerous because we don't read or write from this account
    #[account(
        mut,
        seeds = [
            b"metadata", 
            token_metadata_program.key().as_ref(), 
            mint.key().as_ref(),
            b"edition"
        ],
        bump,
        seeds::program = token_metadata_program.key()
    )]
    pub master_edition: UncheckedAccount<'info>,

    #[account(mut)]
    pub payer: Signer<'info>,

    /// CHECK: Mint authority for the NFT
    pub mint_authority: Signer<'info>,

    /// CHECK: Recipient of the NFT
    pub recipient: AccountInfo<'info>,

    pub rent: Sysvar<'info, Rent>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    /// CHECK: This is not dangerous because it's the official Metaplex program
    pub token_metadata_program: Program<'info, Metadata>,
}

impl<'info> MintNFT<'info> {
    pub fn validate_inputs(
        &self,
        name: &str,
        symbol: &str,
        uri: &str,
        card_type: u8,
        rarity: u8,
        effect_type: u8,
        effect_value: u64,
        duration: i64,
    ) -> Result<()> {
        // Validate name length
        require!(
            name.len() >= MIN_NAME_LENGTH && name.len() <= MAX_NAME_LENGTH,
            FinovaNFTError::InvalidNameLength
        );

        // Validate symbol length
        require!(
            symbol.len() >= MIN_SYMBOL_LENGTH && symbol.len() <= MAX_SYMBOL_LENGTH,
            FinovaNFTError::InvalidSymbolLength
        );

        // Validate URI format
        require!(
            uri.len() >= MIN_URI_LENGTH && uri.len() <= MAX_URI_LENGTH,
            FinovaNFTError::InvalidURILength
        );

        require!(
            uri.starts_with("https://") || uri.starts_with("ipfs://"),
            FinovaNFTError::InvalidURIFormat
        );

        // Validate card type
        require!(
            card_type <= MAX_CARD_TYPE,
            FinovaNFTError::InvalidCardType
        );

        // Validate rarity
        require!(
            rarity >= RARITY_COMMON && rarity <= RARITY_LEGENDARY,
            FinovaNFTError::InvalidRarity
        );

        // Validate effect type
        require!(
            effect_type <= MAX_EFFECT_TYPE,
            FinovaNFTError::InvalidEffectType
        );

        // Validate effect value based on rarity
        let max_effect = match rarity {
            RARITY_COMMON => MAX_COMMON_EFFECT,
            RARITY_UNCOMMON => MAX_UNCOMMON_EFFECT,
            RARITY_RARE => MAX_RARE_EFFECT,
            RARITY_EPIC => MAX_EPIC_EFFECT,
            RARITY_LEGENDARY => MAX_LEGENDARY_EFFECT,
            _ => return Err(FinovaNFTError::InvalidRarity.into()),
        };

        require!(
            effect_value <= max_effect,
            FinovaNFTError::EffectValueTooHigh
        );

        // Validate duration
        require!(
            duration >= MIN_DURATION && duration <= MAX_DURATION,
            FinovaNFTError::InvalidDuration
        );

        Ok(())
    }

    pub fn validate_collection(&self, collection_id: &str) -> Result<()> {
        // Validate collection ID format
        require!(
            collection_id.len() >= MIN_COLLECTION_ID_LENGTH && 
            collection_id.len() <= MAX_COLLECTION_ID_LENGTH,
            FinovaNFTError::InvalidCollectionId
        );

        // Check if collection exists and is active
        if self.collection.is_initialized {
            require!(
                self.collection.is_active,
                FinovaNFTError::CollectionNotActive
            );

            require!(
                self.collection.current_supply < self.collection.max_supply,
                FinovaNFTError::CollectionSupplyExceeded
            );
        }

        Ok(())
    }

    pub fn validate_mint_authority(&self) -> Result<()> {
        // Check if mint authority is authorized
        require!(
            self.mint_authority.is_signer,
            FinovaNFTError::UnauthorizedMintAuthority
        );

        // For special cards, only specific authorities can mint
        require!(
            is_authorized_minter(&self.mint_authority.key()),
            FinovaNFTError::UnauthorizedMintAuthority
        );

        Ok(())
    }

    pub fn calculate_rarity_multiplier(rarity: u8) -> Result<u64> {
        match rarity {
            RARITY_COMMON => Ok(COMMON_MULTIPLIER),
            RARITY_UNCOMMON => Ok(UNCOMMON_MULTIPLIER),
            RARITY_RARE => Ok(RARE_MULTIPLIER),
            RARITY_EPIC => Ok(EPIC_MULTIPLIER),
            RARITY_LEGENDARY => Ok(LEGENDARY_MULTIPLIER),
            _ => Err(FinovaNFTError::InvalidRarity.into()),
        }
    }

    pub fn calculate_mint_cost(rarity: u8, card_type: u8) -> Result<u64> {
        let base_cost = match rarity {
            RARITY_COMMON => MINT_COST_COMMON,
            RARITY_UNCOMMON => MINT_COST_UNCOMMON,
            RARITY_RARE => MINT_COST_RARE,
            RARITY_EPIC => MINT_COST_EPIC,
            RARITY_LEGENDARY => MINT_COST_LEGENDARY,
            _ => return Err(FinovaNFTError::InvalidRarity.into()),
        };

        let type_multiplier = match card_type {
            CARD_TYPE_MINING => MINING_CARD_MULTIPLIER,
            CARD_TYPE_XP => XP_CARD_MULTIPLIER,
            CARD_TYPE_REFERRAL => REFERRAL_CARD_MULTIPLIER,
            CARD_TYPE_SPECIAL => SPECIAL_CARD_MULTIPLIER,
            _ => return Err(FinovaNFTError::InvalidCardType.into()),
        };

        Ok(base_cost * type_multiplier / BASIS_POINTS)
    }
}

pub fn handler(
    ctx: Context<MintNFT>,
    name: String,
    symbol: String,
    uri: String,
    card_type: u8,
    rarity: u8,
    effect_type: u8,
    effect_value: u64,
    duration: i64,
    collection_id: String,
    creator_royalty: u16,
) -> Result<()> {
    let accounts = &ctx.accounts;

    // Validate all inputs
    accounts.validate_inputs(
        &name,
        &symbol,
        &uri,
        card_type,
        rarity,
        effect_type,
        effect_value,
        duration,
    )?;

    accounts.validate_collection(&collection_id)?;
    accounts.validate_mint_authority()?;

    // Calculate mint cost and validate payment
    let mint_cost = MintNFT::calculate_mint_cost(rarity, card_type)?;
    let rarity_multiplier = MintNFT::calculate_rarity_multiplier(rarity)?;

    // Get current timestamp
    let clock = Clock::get()?;
    let current_timestamp = clock.unix_timestamp;

    // Initialize or update collection
    if !accounts.collection.is_initialized {
        accounts.collection.collection_id = collection_id.clone();
        accounts.collection.name = format!("Finova {} Collection", collection_id);
        accounts.collection.symbol = "FINC".to_string();
        accounts.collection.description = "Finova Network Special Cards Collection".to_string();
        accounts.collection.image = "https://finova.network/collection.png".to_string();
        accounts.collection.external_url = "https://finova.network".to_string();
        accounts.collection.creator = accounts.mint_authority.key();
        accounts.collection.max_supply = DEFAULT_COLLECTION_MAX_SUPPLY;
        accounts.collection.current_supply = 0;
        accounts.collection.royalty_percentage = creator_royalty;
        accounts.collection.is_active = true;
        accounts.collection.is_initialized = true;
        accounts.collection.created_at = current_timestamp;
        accounts.collection.updated_at = current_timestamp;
        accounts.collection.bump = ctx.bumps.collection;
    }

    // Initialize NFT metadata
    accounts.nft_metadata.mint = accounts.mint.key();
    accounts.nft_metadata.name = name.clone();
    accounts.nft_metadata.symbol = symbol.clone();
    accounts.nft_metadata.uri = uri.clone();
    accounts.nft_metadata.description = generate_card_description(card_type, rarity, effect_type)?;
    accounts.nft_metadata.image = generate_card_image_url(card_type, rarity)?;
    accounts.nft_metadata.external_url = format!("https://finova.network/nft/{}", accounts.mint.key());
    accounts.nft_metadata.collection = accounts.collection.key();
    accounts.nft_metadata.attributes = generate_card_attributes(
        card_type,
        rarity,
        effect_type,
        effect_value,
        duration,
    )?;
    accounts.nft_metadata.creators = vec![Creator {
        address: accounts.mint_authority.key(),
        verified: true,
        share: 100,
    }];
    accounts.nft_metadata.seller_fee_basis_points = creator_royalty;
    accounts.nft_metadata.is_mutable = true;
    accounts.nft_metadata.primary_sale_happened = false;
    accounts.nft_metadata.is_edition = false;
    accounts.nft_metadata.created_at = current_timestamp;
    accounts.nft_metadata.updated_at = current_timestamp;
    accounts.nft_metadata.bump = ctx.bumps.nft_metadata;

    // Initialize special card if applicable
    if card_type != CARD_TYPE_COLLECTIBLE {
        accounts.special_card.mint = accounts.mint.key();
        accounts.special_card.card_type = card_type;
        accounts.special_card.rarity = rarity;
        accounts.special_card.effect_type = effect_type;
        accounts.special_card.effect_value = effect_value;
        accounts.special_card.duration = duration;
        accounts.special_card.uses_remaining = if card_type == CARD_TYPE_SINGLE_USE { 1 } else { u32::MAX };
        accounts.special_card.is_active = true;
        accounts.special_card.is_tradeable = true;
        accounts.special_card.mint_cost = mint_cost;
        accounts.special_card.rarity_multiplier = rarity_multiplier;
        accounts.special_card.collection_id = collection_id.clone();
        accounts.special_card.original_owner = accounts.recipient.key();
        accounts.special_card.current_owner = accounts.recipient.key();
        accounts.special_card.created_at = current_timestamp;
        accounts.special_card.last_used_at = 0;
        accounts.special_card.bump = ctx.bumps.special_card;
    }

    // Create metadata account using Metaplex
    let creator_accounts = vec![Creator {
        address: accounts.mint_authority.key(),
        verified: true,
        share: 100,
    }];

    let data_v2 = DataV2 {
        name: name.clone(),
        symbol: symbol.clone(),
        uri: uri.clone(),
        seller_fee_basis_points: creator_royalty,
        creators: Some(creator_accounts),
        collection: None, // We'll set this separately if needed
        uses: None,
    };

    let metadata_ctx = CpiContext::new(
        accounts.token_metadata_program.to_account_info(),
        CreateMetadataAccountsV3 {
            metadata: accounts.metadata.to_account_info(),
            mint: accounts.mint.to_account_info(),
            mint_authority: accounts.mint_authority.to_account_info(),
            payer: accounts.payer.to_account_info(),
            update_authority: accounts.mint_authority.to_account_info(),
            system_program: accounts.system_program.to_account_info(),
            rent: accounts.rent.to_account_info(),
        },
    );

    create_metadata_accounts_v3(
        metadata_ctx,
        data_v2,
        true, // is_mutable
        true, // update_authority_is_signer
        None, // collection_details
    )?;

    // Create master edition
    let master_edition_ctx = CpiContext::new(
        accounts.token_metadata_program.to_account_info(),
        CreateMasterEditionV3 {
            edition: accounts.master_edition.to_account_info(),
            mint: accounts.mint.to_account_info(),
            update_authority: accounts.mint_authority.to_account_info(),
            mint_authority: accounts.mint_authority.to_account_info(),
            payer: accounts.payer.to_account_info(),
            metadata: accounts.metadata.to_account_info(),
            token_program: accounts.token_program.to_account_info(),
            system_program: accounts.system_program.to_account_info(),
            rent: accounts.rent.to_account_info(),
        },
    );

    create_master_edition_v3(
        master_edition_ctx,
        Some(0), // max_supply (0 means unlimited editions)
    )?;

    // Mint the token to recipient
    let mint_ctx = CpiContext::new(
        accounts.token_program.to_account_info(),
        MintTo {
            mint: accounts.mint.to_account_info(),
            to: accounts.token_account.to_account_info(),
            authority: accounts.mint_authority.to_account_info(),
        },
    );

    mint_to(mint_ctx, 1)?;

    // Update collection supply
    accounts.collection.current_supply = accounts
        .collection
        .current_supply
        .checked_add(1)
        .ok_or(FinovaNFTError::MathOverflow)?;
    accounts.collection.updated_at = current_timestamp;

    // Emit mint event
    emit!(NFTMinted {
        mint: accounts.mint.key(),
        recipient: accounts.recipient.key(),
        collection: accounts.collection.key(),
        name: name,
        symbol: symbol,
        uri: uri,
        card_type,
        rarity,
        effect_type,
        effect_value,
        duration,
        mint_cost,
        rarity_multiplier,
        timestamp: current_timestamp,
    });

    msg!(
        "Successfully minted NFT: {} to recipient: {}",
        accounts.mint.key(),
        accounts.recipient.key()
    );

    Ok(())
}

fn generate_card_description(card_type: u8, rarity: u8, effect_type: u8) -> Result<String> {
    let type_name = match card_type {
        CARD_TYPE_MINING => "Mining Boost",
        CARD_TYPE_XP => "XP Accelerator",
        CARD_TYPE_REFERRAL => "Referral Power",
        CARD_TYPE_SPECIAL => "Special Effect",
        CARD_TYPE_COLLECTIBLE => "Collectible",
        _ => return Err(FinovaNFTError::InvalidCardType.into()),
    };

    let rarity_name = match rarity {
        RARITY_COMMON => "Common",
        RARITY_UNCOMMON => "Uncommon",
        RARITY_RARE => "Rare",
        RARITY_EPIC => "Epic",
        RARITY_LEGENDARY => "Legendary",
        _ => return Err(FinovaNFTError::InvalidRarity.into()),
    };

    let effect_name = match effect_type {
        EFFECT_TYPE_MINING_BOOST => "increases mining rate",
        EFFECT_TYPE_XP_MULTIPLIER => "multiplies XP gains",
        EFFECT_TYPE_REFERRAL_BONUS => "enhances referral rewards",
        EFFECT_TYPE_STREAK_PROTECTION => "protects activity streaks",
        EFFECT_TYPE_QUALITY_BOOST => "improves content quality scores",
        _ => "provides special benefits",
    };

    Ok(format!(
        "A {} {} card that {} for Finova Network users. This special card provides enhanced benefits within the Finova ecosystem.",
        rarity_name,
        type_name,
        effect_name
    ))
}

fn generate_card_image_url(card_type: u8, rarity: u8) -> Result<String> {
    let type_folder = match card_type {
        CARD_TYPE_MINING => "mining",
        CARD_TYPE_XP => "xp",
        CARD_TYPE_REFERRAL => "referral",
        CARD_TYPE_SPECIAL => "special",
        CARD_TYPE_COLLECTIBLE => "collectible",
        _ => return Err(FinovaNFTError::InvalidCardType.into()),
    };

    let rarity_name = match rarity {
        RARITY_COMMON => "common",
        RARITY_UNCOMMON => "uncommon",
        RARITY_RARE => "rare",
        RARITY_EPIC => "epic",
        RARITY_LEGENDARY => "legendary",
        _ => return Err(FinovaNFTError::InvalidRarity.into()),
    };

    Ok(format!(
        "https://assets.finova.network/cards/{}/{}.png",
        type_folder,
        rarity_name
    ))
}

fn generate_card_attributes(
    card_type: u8,
    rarity: u8,
    effect_type: u8,
    effect_value: u64,
    duration: i64,
) -> Result<String> {
    let attributes = serde_json::json!([
        {
            "trait_type": "Card Type",
            "value": match card_type {
                CARD_TYPE_MINING => "Mining Boost",
                CARD_TYPE_XP => "XP Accelerator",
                CARD_TYPE_REFERRAL => "Referral Power",
                CARD_TYPE_SPECIAL => "Special Effect",
                CARD_TYPE_COLLECTIBLE => "Collectible",
                _ => "Unknown"
            }
        },
        {
            "trait_type": "Rarity",
            "value": match rarity {
                RARITY_COMMON => "Common",
                RARITY_UNCOMMON => "Uncommon",
                RARITY_RARE => "Rare",
                RARITY_EPIC => "Epic",
                RARITY_LEGENDARY => "Legendary",
                _ => "Unknown"
            }
        },
        {
            "trait_type": "Effect Type",
            "value": match effect_type {
                EFFECT_TYPE_MINING_BOOST => "Mining Boost",
                EFFECT_TYPE_XP_MULTIPLIER => "XP Multiplier",
                EFFECT_TYPE_REFERRAL_BONUS => "Referral Bonus",
                EFFECT_TYPE_STREAK_PROTECTION => "Streak Protection",
                EFFECT_TYPE_QUALITY_BOOST => "Quality Boost",
                _ => "Unknown"
            }
        },
        {
            "trait_type": "Effect Value",
            "value": effect_value
        },
        {
            "trait_type": "Duration (hours)",
            "value": duration / 3600
        },
        {
            "trait_type": "Network",
            "value": "Finova"
        }
    ]);

    serde_json::to_string(&attributes)
        .map_err(|_| FinovaNFTError::SerializationError.into())
}

#[event]
pub struct NFTMinted {
    pub mint: Pubkey,
    pub recipient: Pubkey,
    pub collection: Pubkey,
    pub name: String,
    pub symbol: String,
    pub uri: String,
    pub card_type: u8,
    pub rarity: u8,
    pub effect_type: u8,
    pub effect_value: u64,
    pub duration: i64,
    pub mint_cost: u64,
    pub rarity_multiplier: u64,
    pub timestamp: i64,
}
