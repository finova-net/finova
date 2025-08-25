// programs/finova-nft/src/instructions/transfer_nft.rs

use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    metadata::{
        mpl_token_metadata::{
            instructions::{TransferV1CpiBuilder, UpdateV1CpiBuilder},
            types::{TransferArgs, UpdateArgs, Data, DataV2},
        },
        Metadata,
        MetadataAccount,
    },
    token::{Mint, Token, TokenAccount, Transfer},
};
use crate::{
    constants::*,
    errors::*,
    state::*,
    utils::*,
};

/// Transfer NFT from one user to another with proper validation and marketplace integration
#[derive(Accounts)]
#[instruction(transfer_args: TransferNftArgs)]
pub struct TransferNft<'info> {
    /// The authority initiating the transfer (current owner or approved delegate)
    #[account(mut)]
    pub authority: Signer<'info>,

    /// The user receiving the NFT
    /// CHECK: This account is validated through the associated token account
    pub recipient: UncheckedAccount<'info>,

    /// Current owner of the NFT (must match authority if direct transfer)
    #[account(
        mut,
        constraint = current_owner.key() == nft_metadata.owner @ FinovaNftError::UnauthorizedOwner
    )]
    pub current_owner: SystemAccount<'info>,

    /// The NFT mint account
    #[account(
        mut,
        constraint = nft_mint.supply == 1 @ FinovaNftError::InvalidNftMint,
        constraint = nft_mint.decimals == 0 @ FinovaNftError::InvalidNftMint
    )]
    pub nft_mint: Account<'info, Mint>,

    /// Current owner's token account for the NFT
    #[account(
        mut,
        associated_token::mint = nft_mint,
        associated_token::authority = current_owner,
        constraint = owner_token_account.amount == 1 @ FinovaNftError::NftNotOwned
    )]
    pub owner_token_account: Account<'info, TokenAccount>,

    /// Recipient's token account for the NFT (will be created if doesn't exist)
    #[account(
        init_if_needed,
        payer = authority,
        associated_token::mint = nft_mint,
        associated_token::authority = recipient
    )]
    pub recipient_token_account: Account<'info, TokenAccount>,

    /// NFT metadata account
    #[account(
        mut,
        seeds = [
            METADATA_SEED.as_bytes(),
            mpl_token_metadata::ID.as_ref(),
            nft_mint.key().as_ref()
        ],
        bump,
        seed_program = mpl_token_metadata::ID
    )]
    pub nft_metadata: Account<'info, MetadataAccount>,

    /// Finova NFT metadata state
    #[account(
        mut,
        seeds = [
            NFT_METADATA_SEED.as_bytes(),
            nft_mint.key().as_ref()
        ],
        bump = nft_finova_metadata.bump,
        constraint = nft_finova_metadata.mint == nft_mint.key() @ FinovaNftError::InvalidMetadata
    )]
    pub nft_finova_metadata: Account<'info, NftMetadata>,

    /// Collection metadata (if part of a collection)
    #[account(
        mut,
        seeds = [
            COLLECTION_SEED.as_bytes(),
            collection.creator.as_ref(),
            collection.symbol.as_bytes()
        ],
        bump = collection.bump,
        constraint = collection.key() == nft_finova_metadata.collection @ FinovaNftError::InvalidCollection
    )]
    pub collection: Account<'info, Collection>,

    /// Special card state (if this NFT is a special card)
    #[account(
        mut,
        seeds = [
            SPECIAL_CARD_SEED.as_bytes(),
            nft_mint.key().as_ref()
        ],
        bump,
        constraint = special_card.mint == nft_mint.key() @ FinovaNftError::InvalidSpecialCard
    )]
    pub special_card: Option<Account<'info, SpecialCard>>,

    /// Marketplace listing (if being sold through marketplace)
    #[account(
        mut,
        seeds = [
            MARKETPLACE_LISTING_SEED.as_bytes(),
            nft_mint.key().as_ref()
        ],
        bump,
        constraint = marketplace_listing.nft_mint == nft_mint.key() @ FinovaNftError::InvalidListing
    )]
    pub marketplace_listing: Option<Account<'info, MarketplaceListing>>,

    /// Token program
    pub token_program: Program<'info, Token>,

    /// Associated token program
    pub associated_token_program: Program<'info, AssociatedToken>,

    /// Metadata program
    /// CHECK: This is the official Metaplex metadata program
    #[account(address = mpl_token_metadata::ID)]
    pub metadata_program: UncheckedAccount<'info>,

    /// System program
    pub system_program: Program<'info, System>,

    /// Sysvar rent
    pub rent: Sysvar<'info, Rent>,

    /// Sysvar instructions
    /// CHECK: This account is validated by the Metaplex program
    pub sysvar_instructions: UncheckedAccount<'info>,
}

/// Arguments for NFT transfer
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct TransferNftArgs {
    /// Transfer type (direct, marketplace sale, etc.)
    pub transfer_type: TransferType,
    /// Sale price (if marketplace transfer)
    pub sale_price: Option<u64>,
    /// Additional metadata to update
    pub update_metadata: Option<UpdateMetadataArgs>,
    /// Transfer approval signature (for delegated transfers)
    pub approval_signature: Option<[u8; 64]>,
}

/// Type of NFT transfer
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum TransferType {
    /// Direct transfer between users
    Direct,
    /// Marketplace sale
    MarketplaceSale,
    /// Guild reward distribution
    GuildReward,
    /// Admin transfer (emergency cases)
    AdminTransfer,
    /// Burn transfer (destroying NFT)
    Burn,
}

/// Metadata update arguments
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct UpdateMetadataArgs {
    pub name: Option<String>,
    pub symbol: Option<String>,
    pub uri: Option<String>,
    pub seller_fee_basis_points: Option<u16>,
}

impl<'info> TransferNft<'info> {
    /// Validate the transfer request
    pub fn validate_transfer(&self, args: &TransferNftArgs) -> Result<()> {
        // Check if NFT is transferable
        if !self.nft_finova_metadata.is_transferable {
            return Err(FinovaNftError::NftNotTransferable.into());
        }

        // Validate transfer type specific requirements
        match args.transfer_type {
            TransferType::Direct => {
                self.validate_direct_transfer()?;
            },
            TransferType::MarketplaceSale => {
                self.validate_marketplace_sale(args.sale_price)?;
            },
            TransferType::GuildReward => {
                self.validate_guild_reward()?;
            },
            TransferType::AdminTransfer => {
                self.validate_admin_transfer()?;
            },
            TransferType::Burn => {
                self.validate_burn_transfer()?;
            },
        }

        // Check if special card has usage restrictions
        if let Some(special_card) = &self.special_card {
            if special_card.is_consumed {
                return Err(FinovaNftError::SpecialCardAlreadyUsed.into());
            }

            // Check if special card can be transferred
            if !special_card.is_transferable {
                return Err(FinovaNftError::SpecialCardNotTransferable.into());
            }
        }

        // Validate collection transfer restrictions
        if self.collection.has_transfer_restrictions {
            self.validate_collection_transfer_rules()?;
        }

        Ok(())
    }

    /// Validate direct transfer
    fn validate_direct_transfer(&self) -> Result<()> {
        // Ensure authority is the owner or approved delegate
        if self.authority.key() != self.current_owner.key() {
            // Check if authority has approval to transfer
            if !self.check_transfer_approval()? {
                return Err(FinovaNftError::UnauthorizedTransfer.into());
            }
        }

        // Check cooldown period for frequent transfers
        let current_time = Clock::get()?.unix_timestamp;
        if current_time - self.nft_finova_metadata.last_transfer_time < TRANSFER_COOLDOWN_SECONDS {
            return Err(FinovaNftError::TransferCooldownActive.into());
        }

        Ok(())
    }

    /// Validate marketplace sale
    fn validate_marketplace_sale(&self, sale_price: Option<u64>) -> Result<()> {
        let marketplace_listing = self.marketplace_listing
            .as_ref()
            .ok_or(FinovaNftError::NoMarketplaceListing)?;

        // Validate sale price matches listing
        let expected_price = sale_price.ok_or(FinovaNftError::MissingSalePrice)?;
        if expected_price != marketplace_listing.price {
            return Err(FinovaNftError::InvalidSalePrice.into());
        }

        // Check if listing is still active
        let current_time = Clock::get()?.unix_timestamp;
        if marketplace_listing.expires_at < current_time {
            return Err(FinovaNftError::ListingExpired.into());
        }

        // Validate buyer meets any requirements
        if let Some(buyer_requirements) = &marketplace_listing.buyer_requirements {
            self.validate_buyer_requirements(buyer_requirements)?;
        }

        Ok(())
    }

    /// Validate guild reward transfer
    fn validate_guild_reward(&self) -> Result<()> {
        // Ensure this is a guild-related NFT or reward
        if self.nft_finova_metadata.nft_type != NftType::GuildReward {
            return Err(FinovaNftError::InvalidGuildReward.into());
        }

        // Additional guild-specific validations can be added here
        // For example, checking guild membership, achievement requirements, etc.

        Ok(())
    }

    /// Validate admin transfer
    fn validate_admin_transfer(&self) -> Result<()> {
        // Check if authority has admin privileges
        if !self.collection.admin_authorities.contains(&self.authority.key()) {
            return Err(FinovaNftError::UnauthorizedAdmin.into());
        }

        // Log admin transfer for audit purposes
        msg!("Admin transfer initiated by: {}", self.authority.key());

        Ok(())
    }

    /// Validate burn transfer
    fn validate_burn_transfer(&self) -> Result<()> {
        // Ensure NFT is burnable
        if !self.nft_finova_metadata.is_burnable {
            return Err(FinovaNftError::NftNotBurnable.into());
        }

        // Special cards might have burn restrictions
        if let Some(special_card) = &self.special_card {
            if special_card.card_type == SpecialCardType::Legendary && 
               !special_card.is_consumed {
                return Err(FinovaNftError::LegendaryCardBurnRestricted.into());
            }
        }

        Ok(())
    }

    /// Check if authority has transfer approval
    fn check_transfer_approval(&self) -> Result<bool> {
        // Implementation would check on-chain approval mechanisms
        // For now, returning false as placeholder
        Ok(false)
    }

    /// Validate collection transfer rules
    fn validate_collection_transfer_rules(&self) -> Result<()> {
        // Check if recipient meets collection requirements
        if let Some(requirements) = &self.collection.transfer_requirements {
            // Validate minimum XP level
            if let Some(min_xp) = requirements.min_xp_level {
                // This would need to fetch recipient's XP from core program
                // For now, we'll assume it's valid
                msg!("Validating recipient XP level: {}", min_xp);
            }

            // Validate minimum stake amount
            if let Some(min_stake) = requirements.min_stake_amount {
                msg!("Validating recipient stake amount: {}", min_stake);
            }

            // Validate guild membership if required
            if requirements.requires_guild_membership {
                msg!("Validating recipient guild membership");
            }
        }

        Ok(())
    }

    /// Validate buyer requirements for marketplace sales
    fn validate_buyer_requirements(&self, _requirements: &BuyerRequirements) -> Result<()> {
        // Implementation would validate buyer meets specific requirements
        // such as minimum level, stake amount, guild membership, etc.
        Ok(())
    }

    /// Execute the NFT transfer
    pub fn execute_transfer(&mut self, args: TransferNftArgs) -> Result<()> {
        let current_time = Clock::get()?.unix_timestamp;

        // Handle marketplace sale payment processing
        if args.transfer_type == TransferType::MarketplaceSale {
            self.process_marketplace_payment(&args)?;
        }

        // Perform the actual token transfer
        self.transfer_nft_token()?;

        // Update NFT metadata
        self.update_nft_metadata(&args, current_time)?;

        // Update collection statistics
        self.update_collection_stats(&args.transfer_type)?;

        // Handle special card transfer logic
        if let Some(special_card) = &mut self.special_card {
            self.handle_special_card_transfer(special_card, &args.transfer_type)?;
        }

        // Close marketplace listing if applicable
        if args.transfer_type == TransferType::MarketplaceSale {
            self.close_marketplace_listing()?;
        }

        // Emit transfer event
        self.emit_transfer_event(&args, current_time)?;

        Ok(())
    }

    /// Transfer the actual NFT token
    fn transfer_nft_token(&self) -> Result<()> {
        let transfer_instruction = Transfer {
            from: self.owner_token_account.to_account_info(),
            to: self.recipient_token_account.to_account_info(),
            authority: self.current_owner.to_account_info(),
        };

        let cpi_ctx = CpiContext::new(
            self.token_program.to_account_info(),
            transfer_instruction,
        );

        anchor_spl::token::transfer(cpi_ctx, 1)?;

        Ok(())
    }

    /// Update NFT metadata after transfer
    fn update_nft_metadata(&mut self, args: &TransferNftArgs, current_time: i64) -> Result<()> {
        // Update Finova-specific metadata
        self.nft_finova_metadata.owner = self.recipient.key();
        self.nft_finova_metadata.last_transfer_time = current_time;
        self.nft_finova_metadata.transfer_count += 1;

        // Update transfer history
        self.nft_finova_metadata.transfer_history.push(TransferRecord {
            from: self.current_owner.key(),
            to: self.recipient.key(),
            transfer_type: args.transfer_type.clone(),
            timestamp: current_time,
            price: args.sale_price,
        });

        // Keep only last 10 transfer records to save space
        if self.nft_finova_metadata.transfer_history.len() > 10 {
            self.nft_finova_metadata.transfer_history.remove(0);
        }

        // Update Metaplex metadata if requested
        if let Some(update_args) = &args.update_metadata {
            self.update_metaplex_metadata(update_args)?;
        }

        Ok(())
    }

    /// Update Metaplex metadata
    fn update_metaplex_metadata(&self, update_args: &UpdateMetadataArgs) -> Result<()> {
        if update_args.name.is_some() || 
           update_args.symbol.is_some() || 
           update_args.uri.is_some() {
            
            let current_data = &self.nft_metadata.data;
            
            let new_data = DataV2 {
                name: update_args.name.clone().unwrap_or(current_data.name.clone()),
                symbol: update_args.symbol.clone().unwrap_or(current_data.symbol.clone()),
                uri: update_args.uri.clone().unwrap_or(current_data.uri.clone()),
                seller_fee_basis_points: update_args.seller_fee_basis_points
                    .unwrap_or(current_data.seller_fee_basis_points),
                creators: current_data.creators.clone(),
                collection: current_data.collection.clone(),
                uses: current_data.uses.clone(),
            };

            let update_instruction = UpdateV1CpiBuilder::new(&self.metadata_program)
                .metadata(&self.nft_metadata.to_account_info())
                .authority(&self.current_owner.to_account_info())
                .data(Data::V2(new_data))
                .invoke()?;
        }

        Ok(())
    }

    /// Update collection statistics
    fn update_collection_stats(&mut self, transfer_type: &TransferType) -> Result<()> {
        self.collection.transfer_count += 1;
        self.collection.last_activity = Clock::get()?.unix_timestamp;

        match transfer_type {
            TransferType::MarketplaceSale => {
                self.collection.marketplace_sales += 1;
            },
            TransferType::Direct => {
                self.collection.direct_transfers += 1;
            },
            TransferType::GuildReward => {
                self.collection.guild_distributions += 1;
            },
            _ => {}
        }

        Ok(())
    }

    /// Handle special card transfer logic
    fn handle_special_card_transfer(
        &self,
        special_card: &mut Account<SpecialCard>,
        transfer_type: &TransferType,
    ) -> Result<()> {
        // Update special card owner
        special_card.current_owner = self.recipient.key();
        special_card.transfer_count += 1;

        // Reset usage if it's a new owner (for certain card types)
        match special_card.card_type {
            SpecialCardType::SingleUse => {
                // Single use cards retain their consumed state
            },
            SpecialCardType::MultiUse => {
                // Multi-use cards might reset usage count for new owner
                if matches!(transfer_type, TransferType::MarketplaceSale | TransferType::Direct) {
                    special_card.usage_count = 0;
                    special_card.last_used = 0;
                }
            },
            SpecialCardType::Permanent => {
                // Permanent cards transfer with all their state
            },
            _ => {}
        }

        Ok(())
    }

    /// Process marketplace sale payment
    fn process_marketplace_payment(&self, args: &TransferNftArgs) -> Result<()> {
        let marketplace_listing = self.marketplace_listing
            .as_ref()
            .ok_or(FinovaNftError::NoMarketplaceListing)?;

        let sale_price = args.sale_price.ok_or(FinovaNftError::MissingSalePrice)?;

        // Calculate fees
        let marketplace_fee = (sale_price * MARKETPLACE_FEE_BASIS_POINTS as u64) / 10000;
        let creator_royalty = (sale_price * self.nft_metadata.data.seller_fee_basis_points as u64) / 10000;
        let seller_proceeds = sale_price - marketplace_fee - creator_royalty;

        // Transfer payments (this would typically involve SOL/token transfers)
        msg!("Processing marketplace payment:");
        msg!("Sale price: {}", sale_price);
        msg!("Marketplace fee: {}", marketplace_fee);
        msg!("Creator royalty: {}", creator_royalty);
        msg!("Seller proceeds: {}", seller_proceeds);

        // Actual payment transfers would be implemented here
        
        Ok(())
    }

    /// Close marketplace listing after successful sale
    fn close_marketplace_listing(&mut self) -> Result<()> {
        if let Some(marketplace_listing) = &mut self.marketplace_listing {
            marketplace_listing.is_active = false;
            marketplace_listing.sold_at = Some(Clock::get()?.unix_timestamp);
            marketplace_listing.buyer = Some(self.recipient.key());
        }

        Ok(())
    }

    /// Emit transfer event
    fn emit_transfer_event(&self, args: &TransferNftArgs, timestamp: i64) -> Result<()> {
        emit!(NftTransferEvent {
            nft_mint: self.nft_mint.key(),
            from: self.current_owner.key(),
            to: self.recipient.key(),
            transfer_type: args.transfer_type.clone(),
            sale_price: args.sale_price,
            collection: self.collection.key(),
            timestamp,
            transaction_signature: self.sysvar_instructions.key(), // Placeholder
        });

        Ok(())
    }
}

/// Transfer NFT instruction handler
pub fn handler(ctx: Context<TransferNft>, args: TransferNftArgs) -> Result<()> {
    msg!("Transferring NFT: {}", ctx.accounts.nft_mint.key());
    msg!("From: {} To: {}", ctx.accounts.current_owner.key(), ctx.accounts.recipient.key());
    msg!("Transfer type: {:?}", args.transfer_type);

    // Validate the transfer
    ctx.accounts.validate_transfer(&args)?;

    // Execute the transfer
    ctx.accounts.execute_transfer(args)?;

    msg!("NFT transfer completed successfully");

    Ok(())
}

/// NFT Transfer Event
#[event]
pub struct NftTransferEvent {
    pub nft_mint: Pubkey,
    pub from: Pubkey,
    pub to: Pubkey,
    pub transfer_type: TransferType,
    pub sale_price: Option<u64>,
    pub collection: Pubkey,
    pub timestamp: i64,
    pub transaction_signature: Pubkey,
}

/// Transfer record for history tracking
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct TransferRecord {
    pub from: Pubkey,
    pub to: Pubkey,
    pub transfer_type: TransferType,
    pub timestamp: i64,
    pub price: Option<u64>,
}

/// Buyer requirements for marketplace listings
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct BuyerRequirements {
    pub min_xp_level: Option<u32>,
    pub min_stake_amount: Option<u64>,
    pub required_guild_membership: Option<Pubkey>,
    pub whitelist_only: bool,
    pub blacklist_check: bool,
}

/// Collection transfer requirements
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct CollectionTransferRequirements {
    pub min_xp_level: Option<u32>,
    pub min_stake_amount: Option<u64>,
    pub requires_guild_membership: bool,
    pub cooldown_period: Option<i64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transfer_type_serialization() {
        let transfer_type = TransferType::MarketplaceSale;
        let serialized = transfer_type.try_to_vec().unwrap();
        let deserialized = TransferType::try_from_slice(&serialized).unwrap();
        assert_eq!(transfer_type, deserialized);
    }

    #[test]
    fn test_transfer_args_validation() {
        let args = TransferNftArgs {
            transfer_type: TransferType::Direct,
            sale_price: None,
            update_metadata: None,
            approval_signature: None,
        };

        // Direct transfers shouldn't have sale price
        assert!(args.sale_price.is_none());
    }

    #[test]
    fn test_marketplace_transfer_args() {
        let args = TransferNftArgs {
            transfer_type: TransferType::MarketplaceSale,
            sale_price: Some(1000000), // 1 SOL in lamports
            update_metadata: None,
            approval_signature: None,
        };

        // Marketplace transfers should have sale price
        assert!(args.sale_price.is_some());
        assert_eq!(args.sale_price.unwrap(), 1000000);
    }

    #[test]
    fn test_update_metadata_args() {
        let update_args = UpdateMetadataArgs {
            name: Some("Updated NFT Name".to_string()),
            symbol: Some("UPD".to_string()),
            uri: Some("https://updated-uri.com".to_string()),
            seller_fee_basis_points: Some(500), // 5%
        };

        assert!(update_args.name.is_some());
        assert_eq!(update_args.seller_fee_basis_points.unwrap(), 500);
    }
}

