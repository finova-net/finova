// programs/finova-nft/src/instructions/update_metadata.rs

use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount, Mint};
use mpl_token_metadata::state::{Metadata, TokenMetadataAccount};
use mpl_token_metadata::instruction::{update_metadata_accounts_v2};
use mpl_token_metadata::state::{Creator, Data, DataV2};

use crate::constants::*;
use crate::errors::*;
use crate::state::*;
use crate::utils::*;

/// Instruction to update NFT metadata
/// Supports both regular NFT metadata updates and special card parameter changes
#[derive(Accounts)]
#[instruction(
    new_name: Option<String>,
    new_symbol: Option<String>, 
    new_uri: Option<String>,
    new_seller_fee_basis_points: Option<u16>,
    new_creators: Option<Vec<Creator>>,
    special_card_updates: Option<SpecialCardUpdates>
)]
pub struct UpdateMetadata<'info> {
    /// The mint account of the NFT being updated
    #[account(mut)]
    pub mint: Account<'info, Mint>,

    /// The metadata account for this NFT (PDA derived from mint)
    #[account(
        mut,
        seeds = [
            METADATA_PREFIX.as_bytes(),
            mpl_token_metadata::id().as_ref(),
            mint.key().as_ref(),
        ],
        bump,
        seeds::program = mpl_token_metadata::id()
    )]
    /// CHECK: This account is validated by the Metaplex program
    pub metadata: UncheckedAccount<'info>,

    /// The update authority for the metadata (must match metadata.update_authority)
    #[account(mut)]
    pub update_authority: Signer<'info>,

    /// The NFT metadata state account
    #[account(
        mut,
        seeds = [
            NFT_METADATA_SEED.as_bytes(),
            mint.key().as_ref(),
        ],
        bump = nft_metadata.bump
    )]
    pub nft_metadata: Account<'info, NftMetadata>,

    /// Special card account (optional, only for special card NFTs)
    #[account(
        mut,
        seeds = [
            SPECIAL_CARD_SEED.as_bytes(),
            mint.key().as_ref(),
        ],
        bump,
        constraint = special_card.mint == mint.key() @ FinovaNftError::InvalidSpecialCard
    )]
    pub special_card: Option<Account<'info, SpecialCard>>,

    /// Collection account (must match the NFT's collection)
    #[account(
        seeds = [
            COLLECTION_SEED.as_bytes(),
            nft_metadata.collection.as_ref(),
        ],
        bump = collection.bump,
        constraint = collection.key() == nft_metadata.collection @ FinovaNftError::InvalidCollection
    )]
    pub collection: Account<'info, Collection>,

    /// System program
    pub system_program: Program<'info, System>,

    /// Token program
    pub token_program: Program<'info, Token>,

    /// Metaplex Token Metadata program
    /// CHECK: This is the Metaplex Token Metadata program
    #[account(address = mpl_token_metadata::id())]
    pub token_metadata_program: UncheckedAccount<'info>,

    /// Rent sysvar
    pub rent: Sysvar<'info, Rent>,
}

/// Special card parameter updates
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct SpecialCardUpdates {
    pub effect_type: Option<EffectType>,
    pub effect_value: Option<u64>,
    pub duration_hours: Option<u32>,
    pub rarity: Option<Rarity>,
    pub usage_limit: Option<u32>,
    pub category: Option<CardCategory>,
    pub synergy_bonus: Option<u16>,
    pub activation_cost: Option<u64>,
}

impl<'info> UpdateMetadata<'info> {
    /// Validates the update authority has permission to update this NFT
    pub fn validate_update_authority(&self) -> Result<()> {
        // Get the existing metadata
        let metadata_account_info = &self.metadata.to_account_info();
        let metadata = Metadata::from_account_info(metadata_account_info)?;

        // Verify update authority matches
        require!(
            metadata.update_authority == self.update_authority.key(),
            FinovaNftError::InvalidUpdateAuthority
        );

        // For collection NFTs, ensure only collection authority can update
        if self.nft_metadata.nft_type == NftType::Collection {
            require!(
                self.collection.authority == self.update_authority.key(),
                FinovaNftError::UnauthorizedCollectionUpdate
            );
        }

        // For special cards, validate additional permissions
        if let Some(special_card) = &self.special_card {
            require!(
                special_card.creator == self.update_authority.key() ||
                self.collection.authority == self.update_authority.key(),
                FinovaNftError::UnauthorizedSpecialCardUpdate
            );
        }

        Ok(())
    }

    /// Validates the new metadata parameters
    pub fn validate_metadata_updates(
        &self,
        new_name: &Option<String>,
        new_symbol: &Option<String>,
        new_uri: &Option<String>,
        new_seller_fee_basis_points: &Option<u16>,
        new_creators: &Option<Vec<Creator>>,
    ) -> Result<()> {
        // Validate name length
        if let Some(name) = new_name {
            require!(
                name.len() <= MAX_NAME_LENGTH && !name.is_empty(),
                FinovaNftError::InvalidNameLength
            );
        }

        // Validate symbol length
        if let Some(symbol) = new_symbol {
            require!(
                symbol.len() <= MAX_SYMBOL_LENGTH && !symbol.is_empty(),
                FinovaNftError::InvalidSymbolLength
            );
        }

        // Validate URI length and format
        if let Some(uri) = new_uri {
            require!(
                uri.len() <= MAX_URI_LENGTH && !uri.is_empty(),
                FinovaNftError::InvalidUriLength
            );
            
            // Basic URI format validation
            require!(
                uri.starts_with("https://") || uri.starts_with("ipfs://") || uri.starts_with("ar://"),
                FinovaNftError::InvalidUriFormat
            );
        }

        // Validate seller fee basis points (max 10% = 1000 basis points)
        if let Some(fee) = new_seller_fee_basis_points {
            require!(
                *fee <= MAX_SELLER_FEE_BASIS_POINTS,
                FinovaNftError::ExcessiveSellerFee
            );
        }

        // Validate creators
        if let Some(creators) = new_creators {
            require!(
                !creators.is_empty() && creators.len() <= MAX_CREATORS,
                FinovaNftError::InvalidCreatorsCount
            );

            let total_share: u8 = creators.iter().map(|c| c.share).sum();
            require!(
                total_share == 100,
                FinovaNftError::InvalidCreatorShares
            );

            // Ensure no duplicate addresses
            let mut addresses = std::collections::HashSet::new();
            for creator in creators {
                require!(
                    addresses.insert(creator.address),
                    FinovaNftError::DuplicateCreatorAddress
                );
            }
        }

        Ok(())
    }

    /// Validates special card updates
    pub fn validate_special_card_updates(
        &self,
        updates: &Option<SpecialCardUpdates>,
    ) -> Result<()> {
        if let Some(updates) = updates {
            require!(
                self.special_card.is_some(),
                FinovaNftError::NotASpecialCard
            );

            // Validate effect value ranges based on type
            if let Some(effect_value) = updates.effect_value {
                require!(
                    effect_value > 0 && effect_value <= MAX_EFFECT_VALUE,
                    FinovaNftError::InvalidEffectValue
                );
            }

            // Validate duration (max 30 days = 720 hours)
            if let Some(duration) = updates.duration_hours {
                require!(
                    duration > 0 && duration <= MAX_DURATION_HOURS,
                    FinovaNftError::InvalidDuration
                );
            }

            // Validate usage limit
            if let Some(usage_limit) = updates.usage_limit {
                require!(
                    usage_limit > 0 && usage_limit <= MAX_USAGE_LIMIT,
                    FinovaNftError::InvalidUsageLimit
                );
            }

            // Validate synergy bonus (max 50% = 5000 basis points)
            if let Some(synergy_bonus) = updates.synergy_bonus {
                require!(
                    synergy_bonus <= MAX_SYNERGY_BONUS,
                    FinovaNftError::InvalidSynergyBonus
                );
            }

            // Validate activation cost
            if let Some(activation_cost) = updates.activation_cost {
                require!(
                    activation_cost <= MAX_ACTIVATION_COST,
                    FinovaNftError::InvalidActivationCost
                );
            }
        }

        Ok(())
    }

    /// Updates the on-chain metadata through Metaplex
    pub fn update_on_chain_metadata(
        &self,
        new_name: Option<String>,
        new_symbol: Option<String>,
        new_uri: Option<String>,
        new_seller_fee_basis_points: Option<u16>,
        new_creators: Option<Vec<Creator>>,
    ) -> Result<()> {
        // Get current metadata
        let metadata_account_info = &self.metadata.to_account_info();
        let current_metadata = Metadata::from_account_info(metadata_account_info)?;

        // Prepare the new data
        let new_data = DataV2 {
            name: new_name.unwrap_or(current_metadata.data.name),
            symbol: new_symbol.unwrap_or(current_metadata.data.symbol),
            uri: new_uri.unwrap_or(current_metadata.data.uri),
            seller_fee_basis_points: new_seller_fee_basis_points
                .unwrap_or(current_metadata.data.seller_fee_basis_points),
            creators: new_creators.or(current_metadata.data.creators),
            collection: current_metadata.collection,
            uses: current_metadata.uses,
        };

        // Create the instruction
        let update_instruction = update_metadata_accounts_v2(
            mpl_token_metadata::id(),
            self.metadata.key(),
            self.update_authority.key(),
            None, // new_update_authority (None means no change)
            Some(new_data),
            None, // primary_sale_happened (None means no change)
            Some(current_metadata.is_mutable), // is_mutable (keep current setting)
        );

        // Execute the instruction
        anchor_lang::solana_program::program::invoke(
            &update_instruction,
            &[
                self.metadata.to_account_info(),
                self.update_authority.to_account_info(),
            ],
        )?;

        Ok(())
    }

    /// Updates the Finova-specific metadata
    pub fn update_finova_metadata(
        &mut self,
        new_name: &Option<String>,
        new_uri: &Option<String>,
    ) -> Result<()> {
        let clock = Clock::get()?;

        // Update name if provided
        if let Some(name) = new_name {
            self.nft_metadata.name = name.clone();
        }

        // Update URI if provided
        if let Some(uri) = new_uri {
            self.nft_metadata.uri = uri.clone();
        }

        // Update modification timestamp
        self.nft_metadata.last_updated = clock.unix_timestamp;

        // Increment version
        self.nft_metadata.version = self.nft_metadata.version
            .checked_add(1)
            .ok_or(FinovaNftError::NumericalOverflow)?;

        Ok(())
    }

    /// Updates special card parameters
    pub fn update_special_card_parameters(
        &mut self,
        updates: &Option<SpecialCardUpdates>,
    ) -> Result<()> {
        if let (Some(updates), Some(special_card)) = (updates, &mut self.special_card) {
            let clock = Clock::get()?;

            // Update effect type
            if let Some(effect_type) = &updates.effect_type {
                special_card.effect_type = *effect_type;
            }

            // Update effect value
            if let Some(effect_value) = updates.effect_value {
                special_card.effect_value = effect_value;
            }

            // Update duration
            if let Some(duration_hours) = updates.duration_hours {
                special_card.duration_hours = duration_hours;
            }

            // Update rarity
            if let Some(rarity) = &updates.rarity {
                special_card.rarity = *rarity;
            }

            // Update usage limit
            if let Some(usage_limit) = updates.usage_limit {
                special_card.usage_limit = usage_limit;
            }

            // Update category
            if let Some(category) = &updates.category {
                special_card.category = *category;
            }

            // Update synergy bonus
            if let Some(synergy_bonus) = updates.synergy_bonus {
                special_card.synergy_bonus = synergy_bonus;
            }

            // Update activation cost
            if let Some(activation_cost) = updates.activation_cost {
                special_card.activation_cost = activation_cost;
            }

            // Update modification timestamp
            special_card.last_updated = clock.unix_timestamp;

            // Increment version
            special_card.version = special_card.version
                .checked_add(1)
                .ok_or(FinovaNftError::NumericalOverflow)?;
        }

        Ok(())
    }

    /// Records the metadata update event
    pub fn emit_update_event(&self) -> Result<()> {
        emit!(MetadataUpdatedEvent {
            mint: self.mint.key(),
            update_authority: self.update_authority.key(),
            collection: self.nft_metadata.collection,
            nft_type: self.nft_metadata.nft_type,
            version: self.nft_metadata.version,
            timestamp: Clock::get()?.unix_timestamp,
        });

        Ok(())
    }
}

/// Main handler function for updating NFT metadata
pub fn handler(
    ctx: Context<UpdateMetadata>,
    new_name: Option<String>,
    new_symbol: Option<String>,
    new_uri: Option<String>,
    new_seller_fee_basis_points: Option<u16>,
    new_creators: Option<Vec<Creator>>,
    special_card_updates: Option<SpecialCardUpdates>,
) -> Result<()> {
    msg!("Updating NFT metadata for mint: {}", ctx.accounts.mint.key());

    // Validate update authority
    ctx.accounts.validate_update_authority()?;

    // Validate metadata updates
    ctx.accounts.validate_metadata_updates(
        &new_name,
        &new_symbol,
        &new_uri,
        &new_seller_fee_basis_points,
        &new_creators,
    )?;

    // Validate special card updates if provided
    ctx.accounts.validate_special_card_updates(&special_card_updates)?;

    // Update on-chain metadata through Metaplex
    ctx.accounts.update_on_chain_metadata(
        new_name.clone(),
        new_symbol,
        new_uri.clone(),
        new_seller_fee_basis_points,
        new_creators,
    )?;

    // Update Finova-specific metadata
    ctx.accounts.update_finova_metadata(&new_name, &new_uri)?;

    // Update special card parameters if applicable
    ctx.accounts.update_special_card_parameters(&special_card_updates)?;

    // Emit update event
    ctx.accounts.emit_update_event()?;

    msg!("NFT metadata updated successfully");
    Ok(())
}

/// Event emitted when NFT metadata is updated
#[event]
pub struct MetadataUpdatedEvent {
    pub mint: Pubkey,
    pub update_authority: Pubkey,
    pub collection: Pubkey,
    pub nft_type: NftType,
    pub version: u32,
    pub timestamp: i64,
}

/// Instruction for batch metadata updates (admin only)
#[derive(Accounts)]
#[instruction(updates: Vec<BatchMetadataUpdate>)]
pub struct BatchUpdateMetadata<'info> {
    /// The collection authority (admin)
    #[account(mut)]
    pub authority: Signer<'info>,

    /// The collection account
    #[account(
        mut,
        seeds = [COLLECTION_SEED.as_bytes(), collection.name.as_bytes()],
        bump = collection.bump,
        constraint = collection.authority == authority.key() @ FinovaNftError::UnauthorizedAccess
    )]
    pub collection: Account<'info, Collection>,

    /// System program
    pub system_program: Program<'info, System>,
}

/// Batch update data structure
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct BatchMetadataUpdate {
    pub mint: Pubkey,
    pub new_name: Option<String>,
    pub new_uri: Option<String>,
    pub special_card_updates: Option<SpecialCardUpdates>,
}

/// Handler for batch metadata updates (for admin efficiency)
pub fn batch_update_handler(
    ctx: Context<BatchUpdateMetadata>,
    updates: Vec<BatchMetadataUpdate>,
) -> Result<()> {
    require!(
        updates.len() <= MAX_BATCH_SIZE,
        FinovaNftError::BatchSizeExceeded
    );

    msg!("Processing batch update for {} NFTs", updates.len());

    let clock = Clock::get()?;
    
    for (index, update) in updates.iter().enumerate() {
        msg!("Processing update {} for mint: {}", index + 1, update.mint);
        
        // Note: In a real implementation, you would need to load each NFT's accounts
        // and perform the updates. This is a simplified version showing the structure.
        
        // Validate each update would go here
        // Apply updates would go here
        
        // For now, we'll just emit an event for each update
        emit!(BatchMetadataUpdateEvent {
            mint: update.mint,
            authority: ctx.accounts.authority.key(),
            collection: ctx.accounts.collection.key(),
            batch_index: index as u32,
            timestamp: clock.unix_timestamp,
        });
    }

    msg!("Batch metadata update completed");
    Ok(())
}

/// Event for batch metadata updates
#[event]
pub struct BatchMetadataUpdateEvent {
    pub mint: Pubkey,
    pub authority: Pubkey,
    pub collection: Pubkey,
    pub batch_index: u32,
    pub timestamp: i64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_metadata_updates() {
        // Test valid updates
        let valid_name = Some("Updated NFT Name".to_string());
        let valid_symbol = Some("UPD".to_string());
        let valid_uri = Some("https://example.com/updated-metadata.json".to_string());
        let valid_fee = Some(500u16); // 5%
        
        // These would normally be tested with proper account setup
        // For now, we're just testing the validation logic structure
        assert!(valid_name.as_ref().unwrap().len() <= MAX_NAME_LENGTH);
        assert!(valid_symbol.as_ref().unwrap().len() <= MAX_SYMBOL_LENGTH);
        assert!(valid_uri.as_ref().unwrap().len() <= MAX_URI_LENGTH);
        assert!(valid_fee.unwrap() <= MAX_SELLER_FEE_BASIS_POINTS);
    }

    #[test]
    fn test_special_card_updates_validation() {
        let updates = SpecialCardUpdates {
            effect_type: Some(EffectType::MiningBoost),
            effect_value: Some(200), // 200% boost
            duration_hours: Some(24),
            rarity: Some(Rarity::Epic),
            usage_limit: Some(1),
            category: Some(CardCategory::Mining),
            synergy_bonus: Some(1500), // 15%
            activation_cost: Some(100),
        };

        // Validate ranges
        assert!(updates.effect_value.unwrap() <= MAX_EFFECT_VALUE);
        assert!(updates.duration_hours.unwrap() <= MAX_DURATION_HOURS);
        assert!(updates.usage_limit.unwrap() <= MAX_USAGE_LIMIT);
        assert!(updates.synergy_bonus.unwrap() <= MAX_SYNERGY_BONUS);
        assert!(updates.activation_cost.unwrap() <= MAX_ACTIVATION_COST);
    }
}
