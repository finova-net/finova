// programs/finova-nft/src/instructions/create_collection.rs

use anchor_lang::prelude::*;
use anchor_spl::{
    token::{Mint, Token, TokenAccount},
    metadata::{
        create_master_edition_v3, create_metadata_accounts_v3,
        mpl_token_metadata::types::{CollectionDetails, Creator, DataV2, Uses},
        CreateMasterEditionV3, CreateMetadataAccountsV3, Metadata,
        MetadataAccount, MasterEdition,
    },
    associated_token::AssociatedToken,
};

use crate::{
    constants::*,
    errors::FinovaNftError,
    state::{Collection, CollectionType, CollectionTier},
    utils::*,
};

#[derive(Accounts)]
#[instruction(
    collection_id: String,
    collection_type: CollectionType,
    collection_tier: CollectionTier
)]
pub struct CreateCollection<'info> {
    /// Collection state account
    #[account(
        init,
        payer = authority,
        space = Collection::LEN,
        seeds = [
            COLLECTION_SEED,
            collection_id.as_bytes(),
            authority.key().as_ref()
        ],
        bump
    )]
    pub collection: Account<'info, Collection>,

    /// Collection mint account
    #[account(
        init,
        payer = authority,
        mint::decimals = 0,
        mint::authority = collection_mint_authority,
        mint::freeze_authority = collection_mint_authority,
        seeds = [
            COLLECTION_MINT_SEED,
            collection_id.as_bytes(),
            authority.key().as_ref()
        ],
        bump
    )]
    pub collection_mint: Account<'info, Mint>,

    /// Collection mint authority PDA
    /// CHECK: This is a PDA used as mint authority
    #[account(
        seeds = [
            COLLECTION_MINT_AUTHORITY_SEED,
            collection_id.as_bytes(),
            authority.key().as_ref()
        ],
        bump
    )]
    pub collection_mint_authority: UncheckedAccount<'info>,

    /// Collection metadata account
    /// CHECK: This account is initialized by the metadata program
    #[account(mut)]
    pub collection_metadata: UncheckedAccount<'info>,

    /// Collection master edition account
    /// CHECK: This account is initialized by the metadata program
    #[account(mut)]
    pub collection_master_edition: UncheckedAccount<'info>,

    /// Collection token account (for authority)
    #[account(
        init,
        payer = authority,
        associated_token::mint = collection_mint,
        associated_token::authority = authority
    )]
    pub collection_token_account: Account<'info, TokenAccount>,

    /// Authority creating the collection (must be authorized creator)
    #[account(
        mut,
        constraint = is_authorized_creator(&authority.key()) @ FinovaNftError::UnauthorizedCreator
    )]
    pub authority: Signer<'info>,

    /// System program
    pub system_program: Program<'info, System>,

    /// Token program
    pub token_program: Program<'info, Token>,

    /// Associated token program
    pub associated_token_program: Program<'info, AssociatedToken>,

    /// Metadata program
    pub metadata_program: Program<'info, Metadata>,

    /// Rent sysvar
    pub rent: Sysvar<'info, Rent>,
}

impl<'info> CreateCollection<'info> {
    pub fn create_collection_cpi_context(&self) -> CpiContext<'_, '_, '_, 'info, CreateMetadataAccountsV3<'info>> {
        let cpi_accounts = CreateMetadataAccountsV3 {
            metadata: self.collection_metadata.to_account_info(),
            mint: self.collection_mint.to_account_info(),
            mint_authority: self.collection_mint_authority.to_account_info(),
            update_authority: self.collection_mint_authority.to_account_info(),
            payer: self.authority.to_account_info(),
            system_program: self.system_program.to_account_info(),
            rent: self.rent.to_account_info(),
        };
        CpiContext::new(self.metadata_program.to_account_info(), cpi_accounts)
    }

    pub fn create_master_edition_cpi_context(&self) -> CpiContext<'_, '_, '_, 'info, CreateMasterEditionV3<'info>> {
        let cpi_accounts = CreateMasterEditionV3 {
            edition: self.collection_master_edition.to_account_info(),
            mint: self.collection_mint.to_account_info(),
            update_authority: self.collection_mint_authority.to_account_info(),
            mint_authority: self.collection_mint_authority.to_account_info(),
            payer: self.authority.to_account_info(),
            metadata: self.collection_metadata.to_account_info(),
            token_program: self.token_program.to_account_info(),
            system_program: self.system_program.to_account_info(),
            rent: self.rent.to_account_info(),
        };
        CpiContext::new(self.metadata_program.to_account_info(), cpi_accounts)
    }
}

/// Creates a new NFT collection with specified parameters
/// 
/// # Arguments
/// * `ctx` - The instruction context
/// * `collection_id` - Unique identifier for the collection
/// * `name` - Collection name (max 32 chars)
/// * `symbol` - Collection symbol (max 10 chars)  
/// * `uri` - Metadata URI pointing to off-chain data
/// * `description` - Collection description (max 200 chars)
/// * `collection_type` - Type of collection (SpecialCards, ProfileBadges, Achievements)
/// * `collection_tier` - Tier level (Common, Uncommon, Rare, Epic, Legendary, Mythic)
/// * `max_supply` - Maximum number of NFTs in collection (0 = unlimited)
/// * `is_mutable` - Whether metadata can be updated after creation
/// * `creator_fee_basis_points` - Creator royalty fee (0-10000 basis points)
pub fn create_collection(
    ctx: Context<CreateCollection>,
    collection_id: String,
    name: String,
    symbol: String,
    uri: String,
    description: String,
    collection_type: CollectionType,
    collection_tier: CollectionTier,
    max_supply: u64,
    is_mutable: bool,
    creator_fee_basis_points: u16,
) -> Result<()> {
    // Validate input parameters
    require!(
        collection_id.len() <= MAX_COLLECTION_ID_LENGTH,
        FinovaNftError::CollectionIdTooLong
    );
    require!(
        name.len() <= MAX_NAME_LENGTH,
        FinovaNftError::NameTooLong
    );
    require!(
        symbol.len() <= MAX_SYMBOL_LENGTH,
        FinovaNftError::SymbolTooLong
    );
    require!(
        uri.len() <= MAX_URI_LENGTH,
        FinovaNftError::UriTooLong
    );
    require!(
        description.len() <= MAX_DESCRIPTION_LENGTH,
        FinovaNftError::DescriptionTooLong
    );
    require!(
        creator_fee_basis_points <= MAX_CREATOR_FEE_BASIS_POINTS,
        FinovaNftError::CreatorFeeTooHigh
    );

    // Validate collection type and tier compatibility
    validate_collection_type_tier_compatibility(&collection_type, &collection_tier)?;

    let collection = &mut ctx.accounts.collection;
    let clock = Clock::get()?;

    // Initialize collection state
    collection.collection_id = collection_id.clone();
    collection.authority = ctx.accounts.authority.key();
    collection.mint = ctx.accounts.collection_mint.key();
    collection.name = name.clone();
    collection.symbol = symbol.clone();
    collection.uri = uri.clone();
    collection.description = description;
    collection.collection_type = collection_type;
    collection.collection_tier = collection_tier;
    collection.max_supply = max_supply;
    collection.current_supply = 0;
    collection.is_mutable = is_mutable;
    collection.creator_fee_basis_points = creator_fee_basis_points;
    collection.is_verified = false;
    collection.created_at = clock.unix_timestamp;
    collection.updated_at = clock.unix_timestamp;
    collection.mint_authority_bump = ctx.bumps.collection_mint_authority;
    collection.collection_bump = ctx.bumps.collection;

    // Calculate collection multipliers based on type and tier  
    let (utility_multiplier, rarity_multiplier) = calculate_collection_multipliers(
        &collection_type,
        &collection_tier
    );
    collection.utility_multiplier = utility_multiplier;
    collection.rarity_multiplier = rarity_multiplier;

    // Create metadata for the collection
    let creator = Creator {
        address: ctx.accounts.authority.key(),
        verified: true,
        share: 100,
    };

    let collection_details = if max_supply > 0 {
        Some(CollectionDetails::V1 { size: max_supply })
    } else {
        None
    };

    let data = DataV2 {
        name: name.clone(),
        symbol: symbol.clone(),
        uri: uri.clone(),
        seller_fee_basis_points: creator_fee_basis_points,
        creators: Some(vec![creator]),
        collection: None,
        uses: None,
    };

    // Get PDA seeds for signing
    let collection_id_bytes = collection_id.as_bytes();
    let authority_key = ctx.accounts.authority.key();
    let seeds = &[
        COLLECTION_MINT_AUTHORITY_SEED,
        collection_id_bytes,
        authority_key.as_ref(),
        &[collection.mint_authority_bump],
    ];
    let signer_seeds = &[&seeds[..]];

    // Create metadata account
    create_metadata_accounts_v3(
        ctx.accounts
            .create_collection_cpi_context()
            .with_signer(signer_seeds),
        data,
        is_mutable,
        true, // update_authority_is_signer
        collection_details,
    )?;

    // Create master edition (makes it an NFT)
    create_master_edition_v3(
        ctx.accounts
            .create_master_edition_cpi_context()
            .with_signer(signer_seeds),
        Some(max_supply),
    )?;

    // Mint the collection NFT to the authority
    anchor_spl::token::mint_to(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            anchor_spl::token::MintTo {
                mint: ctx.accounts.collection_mint.to_account_info(),
                to: ctx.accounts.collection_token_account.to_account_info(),
                authority: ctx.accounts.collection_mint_authority.to_account_info(),
            },
            signer_seeds,
        ),
        1,
    )?;

    // Emit collection creation event
    emit!(CollectionCreatedEvent {
        collection_id: collection_id.clone(),
        authority: ctx.accounts.authority.key(),
        mint: ctx.accounts.collection_mint.key(),
        name: name,
        symbol: symbol,
        collection_type: collection_type,
        collection_tier: collection_tier,
        max_supply,
        utility_multiplier,
        rarity_multiplier,
        timestamp: clock.unix_timestamp,
    });

    msg!("Collection created successfully: {}", collection_id);
    Ok(())
}

/// Validates that collection type and tier are compatible
fn validate_collection_type_tier_compatibility(
    collection_type: &CollectionType,
    collection_tier: &CollectionTier,
) -> Result<()> {
    match collection_type {
        CollectionType::SpecialCards => {
            // Special cards can be any tier
            Ok(())
        }
        CollectionType::ProfileBadges => {
            // Profile badges are typically Common to Rare
            match collection_tier {
                CollectionTier::Common | CollectionTier::Uncommon | CollectionTier::Rare => Ok(()),
                _ => Err(FinovaNftError::InvalidCollectionTierForType.into()),
            }
        }
        CollectionType::Achievements => {
            // Achievement NFTs can be any tier based on difficulty
            Ok(())
        }
        CollectionType::GuildAssets => {
            // Guild assets are typically Uncommon to Epic
            match collection_tier {
                CollectionTier::Uncommon | CollectionTier::Rare | CollectionTier::Epic => Ok(()),
                _ => Err(FinovaNftError::InvalidCollectionTierForType.into()),
            }
        }
        CollectionType::SeasonalRewards => {
            // Seasonal rewards can be Rare to Mythic
            match collection_tier {
                CollectionTier::Rare | CollectionTier::Epic | CollectionTier::Legendary | CollectionTier::Mythic => Ok(()),
                _ => Err(FinovaNftError::InvalidCollectionTierForType.into()),
            }
        }
    }
}

/// Calculates utility and rarity multipliers based on collection type and tier
fn calculate_collection_multipliers(
    collection_type: &CollectionType,
    collection_tier: &CollectionTier,
) -> (u16, u16) {
    // Base multipliers by tier (in basis points, 10000 = 1.0x)
    let tier_multiplier = match collection_tier {
        CollectionTier::Common => 10000,     // 1.0x
        CollectionTier::Uncommon => 12000,   // 1.2x
        CollectionTier::Rare => 15000,       // 1.5x
        CollectionTier::Epic => 20000,       // 2.0x
        CollectionTier::Legendary => 30000,  // 3.0x
        CollectionTier::Mythic => 50000,     // 5.0x
    };

    // Type-specific utility multipliers
    let utility_multiplier = match collection_type {
        CollectionType::SpecialCards => {
            // Special cards have the highest utility for mining/XP
            match collection_tier {
                CollectionTier::Common => 15000,     // 1.5x
                CollectionTier::Uncommon => 20000,   // 2.0x
                CollectionTier::Rare => 30000,       // 3.0x
                CollectionTier::Epic => 50000,       // 5.0x
                CollectionTier::Legendary => 100000, // 10.0x
                CollectionTier::Mythic => 200000,    // 20.0x
            }
        }
        CollectionType::ProfileBadges => {
            // Profile badges provide moderate ongoing utility
            match collection_tier {
                CollectionTier::Common => 11000,     // 1.1x
                CollectionTier::Uncommon => 13000,   // 1.3x
                CollectionTier::Rare => 16000,       // 1.6x
                _ => 10000, // Should not reach here due to validation
            }
        }
        CollectionType::Achievements => {
            // Achievement NFTs provide prestige but lower utility
            match collection_tier {
                CollectionTier::Common => 10500,     // 1.05x
                CollectionTier::Uncommon => 11500,   // 1.15x
                CollectionTier::Rare => 13000,       // 1.3x
                CollectionTier::Epic => 18000,       // 1.8x
                CollectionTier::Legendary => 25000,  // 2.5x
                CollectionTier::Mythic => 40000,     // 4.0x
            }
        }
        CollectionType::GuildAssets => {
            // Guild assets provide community-focused utility
            match collection_tier {
                CollectionTier::Uncommon => 14000,   // 1.4x
                CollectionTier::Rare => 18000,       // 1.8x
                CollectionTier::Epic => 25000,       // 2.5x
                _ => 10000, // Should not reach here due to validation
            }
        }
        CollectionType::SeasonalRewards => {
            // Seasonal rewards have time-limited high utility
            match collection_tier {
                CollectionTier::Rare => 22000,       // 2.2x
                CollectionTier::Epic => 35000,       // 3.5x
                CollectionTier::Legendary => 60000,  // 6.0x
                CollectionTier::Mythic => 150000,    // 15.0x
                _ => 10000, // Should not reach here due to validation
            }
        }
    };

    // Rarity multiplier affects drop rates and marketplace value
    let rarity_multiplier = tier_multiplier;

    (utility_multiplier, rarity_multiplier)
}

/// Checks if an account is authorized to create collections
fn is_authorized_creator(authority: &Pubkey) -> bool {
    // For now, allow any authority to create collections
    // In production, this would check against a whitelist or admin list
    // Could also check for minimum staking requirements, etc.
    true
}

#[event]
pub struct CollectionCreatedEvent {
    pub collection_id: String,
    pub authority: Pubkey,
    pub mint: Pubkey,
    pub name: String,
    pub symbol: String,
    pub collection_type: CollectionType,
    pub collection_tier: CollectionTier,
    pub max_supply: u64,
    pub utility_multiplier: u16,
    pub rarity_multiplier: u16,
    pub timestamp: i64,
}

/// Additional helper functions for collection management

/// Updates collection verification status (admin only)
pub fn verify_collection(collection: &mut Collection, verified: bool) -> Result<()> {
    collection.is_verified = verified;
    collection.updated_at = Clock::get()?.unix_timestamp;
    Ok(())
}

/// Increments the current supply when a new NFT is minted
pub fn increment_collection_supply(collection: &mut Collection) -> Result<()> {
    require!(
        collection.max_supply == 0 || collection.current_supply < collection.max_supply,
        FinovaNftError::CollectionSupplyExceeded
    );
    
    collection.current_supply += 1;
    collection.updated_at = Clock::get()?.unix_timestamp;
    Ok(())
}

/// Checks if collection has reached maximum supply
pub fn is_collection_supply_available(collection: &Collection) -> bool {
    collection.max_supply == 0 || collection.current_supply < collection.max_supply
}

/// Gets the current mint cost for the collection based on supply and tier
pub fn get_collection_mint_cost(
    collection: &Collection,
    base_cost: u64,
) -> u64 {
    let tier_multiplier = match collection.collection_tier {
        CollectionTier::Common => 100,      // 1.0x
        CollectionTier::Uncommon => 200,    // 2.0x
        CollectionTier::Rare => 500,        // 5.0x
        CollectionTier::Epic => 1000,       // 10.0x
        CollectionTier::Legendary => 2500,  // 25.0x
        CollectionTier::Mythic => 10000,    // 100.0x
    };

    // Apply supply scarcity multiplier
    let supply_multiplier = if collection.max_supply > 0 {
        let remaining = collection.max_supply - collection.current_supply;
        let scarcity_factor = (collection.max_supply * 100) / std::cmp::max(remaining, 1);
        std::cmp::min(scarcity_factor, 1000) // Cap at 10x
    } else {
        100 // No scarcity for unlimited supply
    };

    (base_cost * tier_multiplier * supply_multiplier) / 10000
}
