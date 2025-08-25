// programs/finova-nft/src/instructions/mod.rs

use anchor_lang::prelude::*;

pub mod create_collection;
pub mod mint_nft;
pub mod update_metadata;
pub mod transfer_nft;
pub mod burn_nft;
pub mod use_special_card;
pub mod marketplace;

pub use create_collection::*;
pub use mint_nft::*;
pub use update_metadata::*;
pub use transfer_nft::*;
pub use burn_nft::*;
pub use use_special_card::*;
pub use marketplace::*;

// Re-export all instruction structs for easy access
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum FinovaNftInstruction {
    /// Create a new NFT collection
    /// Accounts expected:
    /// 0. `[signer]` Collection authority
    /// 1. `[writable]` Collection account
    /// 2. `[]` Collection mint
    /// 3. `[]` Collection metadata
    /// 4. `[]` Collection master edition
    /// 5. `[]` Token program
    /// 6. `[]` System program
    /// 7. `[]` Rent sysvar
    CreateCollection {
        name: String,
        symbol: String,
        uri: String,
        seller_fee_basis_points: u16,
        is_mutable: bool,
        max_supply: Option<u64>,
        collection_type: u8,
    },

    /// Mint a new NFT from a collection
    /// Accounts expected:
    /// 0. `[signer]` Payer/Authority
    /// 1. `[writable]` NFT mint account
    /// 2. `[writable]` NFT metadata account
    /// 3. `[writable]` NFT master edition account
    /// 4. `[writable]` Token account
    /// 5. `[]` Collection mint
    /// 6. `[]` Collection metadata
    /// 7. `[]` Collection master edition
    /// 8. `[]` Token program
    /// 9. `[]` System program
    /// 10. `[]` Rent sysvar
    MintNft {
        name: String,
        symbol: String,
        uri: String,
        card_type: u8,
        rarity: u8,
        power: u64,
        duration: Option<i64>,
        special_attributes: Vec<u8>,
    },

    /// Update NFT metadata
    /// Accounts expected:
    /// 0. `[signer]` Update authority
    /// 1. `[writable]` NFT metadata account
    /// 2. `[]` NFT mint account
    UpdateMetadata {
        name: Option<String>,
        symbol: Option<String>,
        uri: Option<String>,
        seller_fee_basis_points: Option<u16>,
        creators: Option<Vec<Creator>>,
    },

    /// Transfer NFT to another wallet
    /// Accounts expected:
    /// 0. `[signer]` Current owner
    /// 1. `[writable]` Source token account
    /// 2. `[writable]` Destination token account
    /// 3. `[]` NFT mint account
    /// 4. `[]` Token program
    TransferNft {
        amount: u64,
    },

    /// Burn an NFT (destroy it permanently)
    /// Accounts expected:
    /// 0. `[signer]` Owner/Authority
    /// 1. `[writable]` NFT mint account
    /// 2. `[writable]` Token account
    /// 3. `[writable]` NFT metadata account
    /// 4. `[writable]` Master edition account
    /// 5. `[]` Token program
    /// 6. `[]` System program
    BurnNft,

    /// Use a special card (single-use NFTs)
    /// Accounts expected:
    /// 0. `[signer]` Card owner
    /// 1. `[writable]` Special card account
    /// 2. `[writable]` User account (from finova-core)
    /// 3. `[writable]` Mining account (from finova-core)
    /// 4. `[]` NFT mint account
    /// 5. `[]` Token account
    /// 6. `[]` Finova core program
    UseSpecialCard {
        card_id: Pubkey,
        target_user: Option<Pubkey>,
    },

    /// Create marketplace listing
    /// Accounts expected:
    /// 0. `[signer]` NFT owner
    /// 1. `[writable]` Marketplace listing account
    /// 2. `[writable]` Escrow token account
    /// 3. `[]` NFT mint account
    /// 4. `[]` Source token account
    /// 5. `[]` Token program
    /// 6. `[]` System program
    CreateListing {
        price: u64,
        currency: Pubkey, // Token mint for payment
        listing_type: u8, // 0 = Fixed price, 1 = Auction
        duration: i64,
        minimum_bid: Option<u64>,
    },

    /// Cancel marketplace listing
    /// Accounts expected:
    /// 0. `[signer]` Listing owner
    /// 1. `[writable]` Marketplace listing account
    /// 2. `[writable]` Escrow token account
    /// 3. `[writable]` Owner token account
    /// 4. `[]` Token program
    CancelListing,

    /// Execute marketplace purchase
    /// Accounts expected:
    /// 0. `[signer]` Buyer
    /// 1. `[writable]` Marketplace listing account
    /// 2. `[writable]` Escrow token account
    /// 3. `[writable]` Buyer token account
    /// 4. `[writable]` Seller token account
    /// 5. `[writable]` Buyer payment account
    /// 6. `[writable]` Seller payment account
    /// 7. `[writable]` Fee account
    /// 8. `[]` Token program
    ExecutePurchase {
        amount: u64,
    },

    /// Place bid in auction
    /// Accounts expected:
    /// 0. `[signer]` Bidder
    /// 1. `[writable]` Marketplace listing account
    /// 2. `[writable]` Bid account
    /// 3. `[writable]` Bidder payment account
    /// 4. `[]` Payment token mint
    /// 5. `[]` Token program
    /// 6. `[]` System program
    PlaceBid {
        bid_amount: u64,
    },

    /// Accept bid in auction
    /// Accounts expected:
    /// 0. `[signer]` Auction owner
    /// 1. `[writable]` Marketplace listing account
    /// 2. `[writable]` Winning bid account
    /// 3. `[writable]` Escrow token account
    /// 4. `[writable]` Winner token account
    /// 5. `[writable]` Bidder payment account
    /// 6. `[writable]` Seller payment account
    /// 7. `[writable]` Fee account
    /// 8. `[]` Token program
    AcceptBid,

    /// Withdraw expired bid
    /// Accounts expected:
    /// 0. `[signer]` Bidder
    /// 1. `[writable]` Bid account
    /// 2. `[writable]` Bidder payment account
    /// 3. `[]` Token program
    WithdrawBid,

    /// Update collection settings (only collection authority)
    /// Accounts expected:
    /// 0. `[signer]` Collection authority
    /// 1. `[writable]` Collection account
    UpdateCollection {
        new_authority: Option<Pubkey>,
        royalty_percentage: Option<u16>,
        is_mutable: Option<bool>,
        max_supply: Option<u64>,
    },

    /// Set collection verified status
    /// Accounts expected:
    /// 0. `[signer]` Collection authority
    /// 1. `[writable]` NFT metadata account
    /// 2. `[]` Collection mint
    /// 3. `[]` Collection metadata
    SetCollectionVerified {
        verified: bool,
    },

    /// Create special card template (for game mechanics)
    /// Accounts expected:
    /// 0. `[signer]` Authority
    /// 1. `[writable]` Card template account
    /// 2. `[]` System program
    CreateCardTemplate {
        card_type: u8,
        name: String,
        description: String,
        base_power: u64,
        duration: Option<i64>,
        rarity: u8,
        max_supply: Option<u64>,
        mint_price: u64,
        effects: Vec<CardEffect>,
    },

    /// Update card template
    /// Accounts expected:
    /// 0. `[signer]` Authority
    /// 1. `[writable]` Card template account
    UpdateCardTemplate {
        name: Option<String>,
        description: Option<String>,
        base_power: Option<u64>,
        duration: Option<i64>,
        mint_price: Option<u64>,
        effects: Option<Vec<CardEffect>>,
        is_active: Option<bool>,
    },

    /// Mint special card from template
    /// Accounts expected:
    /// 0. `[signer]` Payer
    /// 1. `[writable]` Card template account
    /// 2. `[writable]` NFT mint account
    /// 3. `[writable]` NFT metadata account
    /// 4. `[writable]` Token account
    /// 5. `[writable]` Payment account
    /// 6. `[writable]` Treasury account
    /// 7. `[]` Token program
    /// 8. `[]` System program
    MintSpecialCard {
        recipient: Pubkey,
    },

    /// Activate card effect (for temporary boosts)
    /// Accounts expected:
    /// 0. `[signer]` Card owner
    /// 1. `[writable]` Special card account
    /// 2. `[writable]` User account (from finova-core)
    /// 3. `[]` NFT mint account
    /// 4. `[]` Token account
    ActivateCardEffect {
        effect_type: u8,
        target_user: Option<Pubkey>,
        duration_override: Option<i64>,
    },

    /// Combine multiple cards (crafting system)
    /// Accounts expected:
    /// 0. `[signer]` Owner
    /// 1. `[writable]` Source card 1 account
    /// 2. `[writable]` Source card 2 account
    /// 3. `[writable]` Result card account
    /// 4. `[writable]` Recipe account
    /// 5. `[]` Multiple NFT mint accounts
    /// 6. `[]` Multiple token accounts
    /// 7. `[]` Token program
    CombineCards {
        recipe_id: u64,
        source_cards: Vec<Pubkey>,
    },

    /// Stake NFT for rewards
    /// Accounts expected:
    /// 0. `[signer]` NFT owner
    /// 1. `[writable]` NFT stake account
    /// 2. `[writable]` Escrow token account
    /// 3. `[writable]` Owner token account
    /// 4. `[]` NFT mint account
    /// 5. `[]` Token program
    /// 6. `[]` System program
    StakeNft {
        stake_duration: i64,
    },

    /// Unstake NFT and claim rewards
    /// Accounts expected:
    /// 0. `[signer]` NFT owner
    /// 1. `[writable]` NFT stake account
    /// 2. `[writable]` Escrow token account
    /// 3. `[writable]` Owner token account
    /// 4. `[writable]` Reward account
    /// 5. `[]` NFT mint account
    /// 6. `[]` Token program
    UnstakeNft,

    /// Create NFT rental listing
    /// Accounts expected:
    /// 0. `[signer]` NFT owner
    /// 1. `[writable]` Rental listing account
    /// 2. `[writable]` Escrow token account
    /// 3. `[]` NFT mint account
    /// 4. `[]` Source token account
    /// 5. `[]` Token program
    /// 6. `[]` System program
    CreateRental {
        daily_rate: u64,
        max_duration: i64,
        currency: Pubkey,
        deposit_required: u64,
    },

    /// Rent an NFT
    /// Accounts expected:
    /// 0. `[signer]` Renter
    /// 1. `[writable]` Rental listing account
    /// 2. `[writable]` Rental agreement account
    /// 3. `[writable]` Renter payment account
    /// 4. `[writable]` Owner payment account
    /// 5. `[writable]` Deposit escrow account
    /// 6. `[]` Token program
    /// 7. `[]` System program
    RentNft {
        duration_days: u32,
    },

    /// Return rented NFT
    /// Accounts expected:
    /// 0. `[signer]` Renter or Owner
    /// 1. `[writable]` Rental agreement account
    /// 2. `[writable]` Escrow token account
    /// 3. `[writable]` Owner token account
    /// 4. `[writable]` Deposit escrow account
    /// 5. `[writable]` Renter deposit account
    /// 6. `[]` Token program
    ReturnRental,
}

// Card effect types for special cards
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub struct CardEffect {
    pub effect_type: u8,     // 0=Mining boost, 1=XP boost, 2=RP boost, etc.
    pub magnitude: u64,      // Effect strength (percentage or absolute)
    pub duration: i64,       // Effect duration in seconds
    pub target: u8,          // 0=Self, 1=Referrals, 2=Guild, 3=Global
    pub conditions: Vec<u8>, // Required conditions for effect
}

// Creator information for NFT metadata
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub struct Creator {
    pub address: Pubkey,
    pub verified: bool,
    pub share: u8,
}

// Instruction validation helpers
impl FinovaNftInstruction {
    pub fn validate(&self) -> Result<()> {
        match self {
            FinovaNftInstruction::CreateCollection {
                name,
                symbol,
                seller_fee_basis_points,
                ..
            } => {
                require!(!name.is_empty(), crate::errors::FinovaNftError::InvalidName);
                require!(!symbol.is_empty(), crate::errors::FinovaNftError::InvalidSymbol);
                require!(
                    *seller_fee_basis_points <= 10000,
                    crate::errors::FinovaNftError::InvalidRoyalty
                );
            }
            FinovaNftInstruction::MintNft {
                name,
                symbol,
                power,
                rarity,
                ..
            } => {
                require!(!name.is_empty(), crate::errors::FinovaNftError::InvalidName);
                require!(!symbol.is_empty(), crate::errors::FinovaNftError::InvalidSymbol);
                require!(*power > 0, crate::errors::FinovaNftError::InvalidPower);
                require!(*rarity <= 5, crate::errors::FinovaNftError::InvalidRarity);
            }
            FinovaNftInstruction::CreateListing {
                price, duration, ..
            } => {
                require!(*price > 0, crate::errors::FinovaNftError::InvalidPrice);
                require!(*duration > 0, crate::errors::FinovaNftError::InvalidDuration);
            }
            FinovaNftInstruction::PlaceBid { bid_amount } => {
                require!(
                    *bid_amount > 0,
                    crate::errors::FinovaNftError::InvalidBidAmount
                );
            }
            FinovaNftInstruction::CreateCardTemplate {
                name,
                base_power,
                rarity,
                mint_price,
                effects,
                ..
            } => {
                require!(!name.is_empty(), crate::errors::FinovaNftError::InvalidName);
                require!(*base_power > 0, crate::errors::FinovaNftError::InvalidPower);
                require!(*rarity > 0 && *rarity <= 5, crate::errors::FinovaNftError::InvalidRarity);
                require!(*mint_price > 0, crate::errors::FinovaNftError::InvalidPrice);
                require!(
                    effects.len() <= 10,
                    crate::errors::FinovaNftError::TooManyEffects
                );
            }
            FinovaNftInstruction::CombineCards { source_cards, .. } => {
                require!(
                    source_cards.len() >= 2 && source_cards.len() <= 5,
                    crate::errors::FinovaNftError::InvalidCombination
                );
            }
            FinovaNftInstruction::CreateRental {
                daily_rate,
                max_duration,
                deposit_required,
                ..
            } => {
                require!(*daily_rate > 0, crate::errors::FinovaNftError::InvalidPrice);
                require!(*max_duration > 0, crate::errors::FinovaNftError::InvalidDuration);
                require!(
                    *deposit_required >= 0,
                    crate::errors::FinovaNftError::InvalidDeposit
                );
            }
            FinovaNftInstruction::RentNft { duration_days } => {
                require!(
                    *duration_days > 0 && *duration_days <= 365,
                    crate::errors::FinovaNftError::InvalidDuration
                );
            }
            _ => {} // Other instructions have validation in their respective handlers
        }
        Ok(())
    }

    pub fn get_instruction_name(&self) -> &'static str {
        match self {
            FinovaNftInstruction::CreateCollection { .. } => "CreateCollection",
            FinovaNftInstruction::MintNft { .. } => "MintNft",
            FinovaNftInstruction::UpdateMetadata { .. } => "UpdateMetadata",
            FinovaNftInstruction::TransferNft { .. } => "TransferNft",
            FinovaNftInstruction::BurnNft => "BurnNft",
            FinovaNftInstruction::UseSpecialCard { .. } => "UseSpecialCard",
            FinovaNftInstruction::CreateListing { .. } => "CreateListing",
            FinovaNftInstruction::CancelListing => "CancelListing",
            FinovaNftInstruction::ExecutePurchase { .. } => "ExecutePurchase",
            FinovaNftInstruction::PlaceBid { .. } => "PlaceBid",
            FinovaNftInstruction::AcceptBid => "AcceptBid",
            FinovaNftInstruction::WithdrawBid => "WithdrawBid",
            FinovaNftInstruction::UpdateCollection { .. } => "UpdateCollection",
            FinovaNftInstruction::SetCollectionVerified { .. } => "SetCollectionVerified",
            FinovaNftInstruction::CreateCardTemplate { .. } => "CreateCardTemplate",
            FinovaNftInstruction::UpdateCardTemplate { .. } => "UpdateCardTemplate",
            FinovaNftInstruction::MintSpecialCard { .. } => "MintSpecialCard",
            FinovaNftInstruction::ActivateCardEffect { .. } => "ActivateCardEffect",
            FinovaNftInstruction::CombineCards { .. } => "CombineCards",
            FinovaNftInstruction::StakeNft { .. } => "StakeNft",
            FinovaNftInstruction::UnstakeNft => "UnstakeNft",
            FinovaNftInstruction::CreateRental { .. } => "CreateRental",
            FinovaNftInstruction::RentNft { .. } => "RentNft",
            FinovaNftInstruction::ReturnRental => "ReturnRental",
        }
    }

    pub fn requires_authority(&self) -> bool {
        matches!(
            self,
            FinovaNftInstruction::CreateCollection { .. }
                | FinovaNftInstruction::UpdateCollection { .. }
                | FinovaNftInstruction::SetCollectionVerified { .. }
                | FinovaNftInstruction::CreateCardTemplate { .. }
                | FinovaNftInstruction::UpdateCardTemplate { .. }
        )
    }

    pub fn affects_token_balance(&self) -> bool {
        matches!(
            self,
            FinovaNftInstruction::MintNft { .. }
                | FinovaNftInstruction::BurnNft
                | FinovaNftInstruction::TransferNft { .. }
                | FinovaNftInstruction::ExecutePurchase { .. }
                | FinovaNftInstruction::AcceptBid
                | FinovaNftInstruction::StakeNft { .. }
                | FinovaNftInstruction::UnstakeNft
                | FinovaNftInstruction::RentNft { .. }
                | FinovaNftInstruction::ReturnRental
        )
    }

    pub fn requires_payment(&self) -> bool {
        matches!(
            self,
            FinovaNftInstruction::MintSpecialCard { .. }
                | FinovaNftInstruction::ExecutePurchase { .. }
                | FinovaNftInstruction::PlaceBid { .. }
                | FinovaNftInstruction::RentNft { .. }
        )
    }
}

// Instruction data size calculations for rent calculation
impl FinovaNftInstruction {
    pub fn get_max_size(&self) -> usize {
        match self {
            FinovaNftInstruction::CreateCollection { .. } => 300,
            FinovaNftInstruction::MintNft { .. } => 400,
            FinovaNftInstruction::UpdateMetadata { .. } => 350,
            FinovaNftInstruction::CreateListing { .. } => 150,
            FinovaNftInstruction::CreateCardTemplate { .. } => 500,
            FinovaNftInstruction::CombineCards { source_cards, .. } => 100 + (source_cards.len() * 32),
            FinovaNftInstruction::CreateRental { .. } => 200,
            _ => 100, // Default size for simpler instructions
        }
    }
}

// Helper functions for instruction building
pub fn create_collection_ix(
    authority: Pubkey,
    collection: Pubkey,
    collection_mint: Pubkey,
    name: String,
    symbol: String,
    uri: String,
    seller_fee_basis_points: u16,
    is_mutable: bool,
    max_supply: Option<u64>,
    collection_type: u8,
) -> anchor_lang::solana_program::instruction::Instruction {
    let data = FinovaNftInstruction::CreateCollection {
        name,
        symbol,
        uri,
        seller_fee_basis_points,
        is_mutable,
        max_supply,
        collection_type,
    };

    anchor_lang::solana_program::instruction::Instruction {
        program_id: crate::ID,
        accounts: vec![
            anchor_lang::solana_program::instruction::AccountMeta::new(authority, true),
            anchor_lang::solana_program::instruction::AccountMeta::new(collection, false),
            anchor_lang::solana_program::instruction::AccountMeta::new_readonly(collection_mint, false),
        ],
        data: data.try_to_vec().unwrap(),
    }
}

pub fn mint_nft_ix(
    authority: Pubkey,
    nft_mint: Pubkey,
    token_account: Pubkey,
    collection_mint: Pubkey,
    name: String,
    symbol: String,
    uri: String,
    card_type: u8,
    rarity: u8,
    power: u64,
    duration: Option<i64>,
    special_attributes: Vec<u8>,
) -> anchor_lang::solana_program::instruction::Instruction {
    let data = FinovaNftInstruction::MintNft {
        name,
        symbol,
        uri,
        card_type,
        rarity,
        power,
        duration,
        special_attributes,
    };

    anchor_lang::solana_program::instruction::Instruction {
        program_id: crate::ID,
        accounts: vec![
            anchor_lang::solana_program::instruction::AccountMeta::new(authority, true),
            anchor_lang::solana_program::instruction::AccountMeta::new(nft_mint, false),
            anchor_lang::solana_program::instruction::AccountMeta::new(token_account, false),
            anchor_lang::solana_program::instruction::AccountMeta::new_readonly(collection_mint, false),
        ],
        data: data.try_to_vec().unwrap(),
    }
}

// Additional instruction builders can be added here for common use cases