# Finova NFT Program

## Overview

The Finova NFT Program is a comprehensive Solana-based smart contract that powers Finova Network's NFT ecosystem, including Special Cards, Profile Badges, Achievement NFTs, and the integrated marketplace. Built with Anchor framework and designed for maximum security, scalability, and integration with the broader Finova ecosystem.

## Features

### Core NFT Functionality
- **Metaplex Integration**: Full compatibility with Metaplex NFT standards
- **Collection Management**: Hierarchical collection structure with verified creators
- **Dynamic Metadata**: Updatable metadata with versioning system
- **Batch Operations**: Gas-efficient batch minting and transfers

### Special Cards System
- **Mining Boost Cards**: Temporary mining rate multipliers
- **XP Accelerator Cards**: Experience point amplifiers  
- **Referral Power Cards**: Network effect enhancers
- **Synergy System**: Card combination bonuses
- **Usage Tracking**: Single-use and duration-based cards

### Profile Badge NFTs
- **Tier System**: Bronze → Silver → Gold → Platinum → Diamond → Mythic
- **Evolution Mechanism**: Upgradeable badges through achievements
- **Permanent Benefits**: Ongoing mining and XP multipliers
- **Status Display**: Visual representation of user achievements

### Achievement NFTs
- **Milestone Rewards**: First 1000 users, viral creators, network builders
- **Lifetime Benefits**: Permanent bonuses and exclusive features
- **Rarity System**: Common to Legendary rarity tiers
- **Historical Records**: Immutable achievement timestamps

### Integrated Marketplace
- **Direct Trading**: Peer-to-peer NFT exchanges
- **Auction System**: English and Dutch auction mechanisms
- **Fee Structure**: Configurable marketplace fees with revenue sharing
- **Royalty System**: Creator royalties on secondary sales

## Architecture

### Program Structure
```
programs/finova-nft/src/
├── lib.rs                    # Program entry point and module declarations
├── instructions/             # All instruction handlers
│   ├── mod.rs               # Module exports
│   ├── create_collection.rs # Collection creation logic
│   ├── mint_nft.rs         # NFT minting functionality  
│   ├── update_metadata.rs  # Metadata update operations
│   ├── transfer_nft.rs     # Transfer and ownership logic
│   ├── burn_nft.rs         # NFT burning functionality
│   ├── use_special_card.rs # Special card usage logic
│   └── marketplace.rs      # Marketplace operations
├── state/                   # Account state structures
│   ├── mod.rs              # State module exports
│   ├── collection.rs       # Collection account structure
│   ├── nft_metadata.rs     # NFT metadata structure
│   ├── special_card.rs     # Special card state
│   └── marketplace.rs      # Marketplace state
├── events/                  # Program events
│   ├── mod.rs              # Event module exports
│   ├── mint.rs             # Minting events
│   ├── transfer.rs         # Transfer events
│   └── use_card.rs         # Card usage events
├── constants.rs             # Program constants
├── errors.rs               # Custom error definitions
└── utils.rs                # Utility functions
```

### Key Components

#### 1. Collection Management
- **Verified Collections**: Integration with Metaplex collection standards
- **Creator Verification**: Multi-signature creator verification system
- **Collection Metadata**: Rich metadata with IPFS integration
- **Access Control**: Role-based permissions for collection management

#### 2. NFT Lifecycle
- **Minting Process**: Secure minting with metadata validation
- **Ownership Transfer**: Safe transfer mechanisms with hooks
- **Metadata Updates**: Versioned metadata with change tracking
- **Burning Mechanism**: Secure NFT destruction with event logging

#### 3. Special Cards Integration
- **Card Types**: Mining, XP, Referral, and Utility cards
- **Usage Mechanics**: Single-use, duration-based, and permanent effects
- **Synergy Calculations**: Complex bonus calculations for card combinations
- **Integration Points**: Hooks to core Finova programs

#### 4. Marketplace Features
- **Listing Management**: Create, update, and cancel listings
- **Bidding System**: Support for auctions and direct purchases
- **Fee Distribution**: Automatic fee splitting between platform and creators
- **Escrow System**: Secure transaction handling

## Data Structures

### Collection Account
```rust
#[account]
pub struct Collection {
    pub authority: Pubkey,           // Collection authority
    pub mint: Pubkey,                // Collection mint address
    pub metadata: Pubkey,            // Metadata account
    pub master_edition: Pubkey,      // Master edition account
    pub symbol: String,              // Collection symbol
    pub name: String,                // Collection name
    pub uri: String,                 // Metadata URI
    pub seller_fee_basis_points: u16, // Royalty percentage
    pub creators: Vec<Creator>,      // Verified creators
    pub collection_type: CollectionType, // Type classification
    pub max_supply: Option<u64>,     // Maximum supply limit
    pub current_supply: u64,         // Current minted count
    pub is_mutable: bool,           // Metadata mutability
    pub created_at: i64,            // Creation timestamp
    pub updated_at: i64,            // Last update timestamp
}
```

### NFT Metadata
```rust
#[account]
pub struct FinovaNftMetadata {
    pub mint: Pubkey,               // NFT mint address
    pub collection: Option<Pubkey>, // Parent collection
    pub name: String,               // NFT name
    pub symbol: String,             // NFT symbol
    pub uri: String,                // Metadata URI
    pub seller_fee_basis_points: u16, // Royalty percentage
    pub creators: Vec<Creator>,     // Creator list
    pub nft_type: NftType,         // NFT classification
    pub attributes: Vec<Attribute>, // NFT attributes
    pub rarity: Rarity,            // Rarity classification
    pub special_properties: Option<SpecialProperties>, // Special functionality
    pub usage_count: u32,          // Usage tracking
    pub max_usage: Option<u32>,    // Usage limit
    pub expires_at: Option<i64>,   // Expiration timestamp
    pub created_at: i64,           // Creation timestamp
    pub updated_at: i64,           // Last update timestamp
    pub version: u32,              // Metadata version
}
```

### Special Card Properties
```rust
#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct SpecialProperties {
    pub card_type: SpecialCardType,
    pub effect_value: u64,         // Effect magnitude
    pub duration: Option<i64>,     // Effect duration
    pub synergy_group: u8,         // Synergy classification
    pub stackable: bool,           // Can stack with others
    pub transferable: bool,        // Can be transferred
    pub consumable: bool,          // Single use flag
    pub requirements: Vec<Requirement>, // Usage requirements
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub enum SpecialCardType {
    // Mining Boost Cards
    DoubleMining,      // +100% mining rate
    TripleMining,      // +200% mining rate  
    MiningFrenzy,      // +500% mining rate
    EternalMiner,      // +50% mining rate (long duration)
    
    // XP Accelerator Cards
    XpDouble,          // +100% XP from all activities
    StreakSaver,       // Maintain XP streak
    LevelRush,         // Instant +500 XP
    XpMagnet,          // +300% XP for viral content
    
    // Referral Power Cards
    ReferralBoost,     // +50% referral rewards
    NetworkAmplifier,  // +2 levels to RP tier
    AmbassadorPass,    // Unlock Ambassador benefits
    NetworkKing,       // +100% from entire network
    
    // Profile Badges
    BronzeBadge,       // Bronze tier badge
    SilverBadge,       // Silver tier badge
    GoldBadge,         // Gold tier badge
    PlatinumBadge,     // Platinum tier badge
    DiamondBadge,      // Diamond tier badge
    MythicBadge,       // Mythic tier badge
    
    // Achievement NFTs
    FinizenPioneer,    // First 1000 users
    ContentKing,       // Viral creator
    Ambassador,        // Network builder
    DiamondHands,      // Whale staker
}
```

### Marketplace Listing
```rust
#[account]
pub struct MarketplaceListing {
    pub seller: Pubkey,            // Seller's public key
    pub nft_mint: Pubkey,          // NFT mint address
    pub price: u64,                // Listing price in lamports
    pub currency: Currency,        // Payment currency
    pub listing_type: ListingType, // Sale or auction
    pub created_at: i64,           // Creation timestamp
    pub expires_at: Option<i64>,   // Expiration timestamp
    pub is_active: bool,           // Listing status
    pub highest_bid: Option<Bid>,  // Current highest bid
    pub min_bid_increment: u64,    // Minimum bid increment
    pub reserve_price: Option<u64>, // Reserve price for auctions
    pub buy_now_price: Option<u64>, // Instant buy price
}
```

## Instructions

### 1. Create Collection
```rust
#[derive(Accounts)]
pub struct CreateCollection<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    
    #[account(
        init,
        payer = authority,
        space = Collection::SIZE,
        seeds = [b"collection", authority.key().as_ref(), name.as_bytes()],
        bump
    )]
    pub collection: Account<'info, Collection>,
    
    #[account(mut)]
    pub mint: Signer<'info>,
    
    /// CHECK: Validated by Metaplex
    #[account(mut)]
    pub metadata: UncheckedAccount<'info>,
    
    /// CHECK: Validated by Metaplex  
    #[account(mut)]
    pub master_edition: UncheckedAccount<'info>,
    
    #[account(mut)]
    pub payer: Signer<'info>,
    
    pub rent: Sysvar<'info, Rent>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    /// CHECK: Metaplex program
    pub token_metadata_program: UncheckedAccount<'info>,
}
```

### 2. Mint NFT
```rust
#[derive(Accounts)]
pub struct MintNft<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    
    #[account(
        init,
        payer = payer,
        space = FinovaNftMetadata::SIZE,
        seeds = [b"nft_metadata", mint.key().as_ref()],
        bump
    )]
    pub nft_metadata: Account<'info, FinovaNftMetadata>,
    
    #[account(mut)]
    pub mint: Signer<'info>,
    
    #[account(
        init,
        payer = payer,
        associated_token::mint = mint,
        associated_token::authority = recipient,
    )]
    pub token_account: Account<'info, TokenAccount>,
    
    /// CHECK: Can be any valid pubkey
    pub recipient: UncheckedAccount<'info>,
    
    #[account(
        has_one = authority,
        constraint = collection.current_supply < collection.max_supply.unwrap_or(u64::MAX)
    )]
    pub collection: Account<'info, Collection>,
    
    /// CHECK: Validated by Metaplex
    #[account(mut)]
    pub metadata: UncheckedAccount<'info>,
    
    /// CHECK: Validated by Metaplex
    #[account(mut)]
    pub master_edition: UncheckedAccount<'info>,
    
    #[account(mut)]
    pub payer: Signer<'info>,
    
    pub rent: Sysvar<'info, Rent>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    /// CHECK: Metaplex program
    pub token_metadata_program: UncheckedAccount<'info>,
}
```

### 3. Use Special Card
```rust
#[derive(Accounts)]
pub struct UseSpecialCard<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    
    #[account(
        mut,
        seeds = [b"nft_metadata", nft_mint.key().as_ref()],
        bump,
        constraint = nft_metadata.special_properties.is_some() @ FinovaNftError::NotSpecialCard,
        constraint = nft_metadata.usage_count < nft_metadata.max_usage.unwrap_or(u32::MAX) @ FinovaNftError::ExceededMaxUsage,
    )]
    pub nft_metadata: Account<'info, FinovaNftMetadata>,
    
    pub nft_mint: Account<'info, Mint>,
    
    #[account(
        associated_token::mint = nft_mint,
        associated_token::authority = user,
        constraint = user_token_account.amount >= 1 @ FinovaNftError::InsufficientBalance,
    )]
    pub user_token_account: Account<'info, TokenAccount>,
    
    /// CHECK: This will be validated by the core program
    #[account(mut)]
    pub finova_core_program: UncheckedAccount<'info>,
    
    /// CHECK: User account in core program
    #[account(mut)]
    pub user_account: UncheckedAccount<'info>,
    
    pub system_program: Program<'info, System>,
}
```

### 4. Marketplace Operations
```rust
#[derive(Accounts)]
pub struct CreateListing<'info> {
    #[account(mut)]
    pub seller: Signer<'info>,
    
    #[account(
        init,
        payer = seller,
        space = MarketplaceListing::SIZE,
        seeds = [b"listing", nft_mint.key().as_ref()],
        bump
    )]
    pub listing: Account<'info, MarketplaceListing>,
    
    pub nft_mint: Account<'info, Mint>,
    
    #[account(
        mut,
        associated_token::mint = nft_mint,
        associated_token::authority = seller,
        constraint = seller_token_account.amount >= 1 @ FinovaNftError::InsufficientBalance,
    )]
    pub seller_token_account: Account<'info, TokenAccount>,
    
    #[account(
        init,
        payer = seller,
        associated_token::mint = nft_mint,
        associated_token::authority = marketplace_authority,
    )]
    pub escrow_token_account: Account<'info, TokenAccount>,
    
    /// CHECK: PDA authority for escrow
    #[account(
        seeds = [b"marketplace_authority"],
        bump
    )]
    pub marketplace_authority: UncheckedAccount<'info>,
    
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}
```

## Events

### NFT Events
```rust
#[event]
pub struct NftMinted {
    pub mint: Pubkey,
    pub recipient: Pubkey,
    pub collection: Option<Pubkey>,
    pub nft_type: NftType,
    pub rarity: Rarity,
    pub timestamp: i64,
}

#[event]
pub struct NftTransferred {
    pub mint: Pubkey,
    pub from: Pubkey,
    pub to: Pubkey,
    pub timestamp: i64,
}

#[event]
pub struct NftBurned {
    pub mint: Pubkey,
    pub owner: Pubkey,
    pub timestamp: i64,
}
```

### Special Card Events
```rust
#[event]
pub struct SpecialCardUsed {
    pub mint: Pubkey,
    pub user: Pubkey,
    pub card_type: SpecialCardType,
    pub effect_value: u64,
    pub duration: Option<i64>,
    pub usage_count: u32,
    pub timestamp: i64,
}

#[event]
pub struct CardSynergyActivated {
    pub user: Pubkey,
    pub cards: Vec<Pubkey>,
    pub synergy_bonus: u64,
    pub timestamp: i64,
}
```

### Marketplace Events
```rust
#[event]
pub struct ListingCreated {
    pub seller: Pubkey,
    pub nft_mint: Pubkey,
    pub price: u64,
    pub listing_type: ListingType,
    pub timestamp: i64,
}

#[event]
pub struct ListingSold {
    pub seller: Pubkey,
    pub buyer: Pubkey,
    pub nft_mint: Pubkey,
    pub price: u64,
    pub platform_fee: u64,
    pub creator_royalty: u64,
    pub timestamp: i64,
}

#[event]
pub struct BidPlaced {
    pub bidder: Pubkey,
    pub nft_mint: Pubkey,
    pub bid_amount: u64,
    pub timestamp: i64,
}
```

## Constants

```rust
// Program constants
pub const COLLECTION_SEED: &[u8] = b"collection";
pub const NFT_METADATA_SEED: &[u8] = b"nft_metadata";
pub const LISTING_SEED: &[u8] = b"listing";
pub const MARKETPLACE_AUTHORITY_SEED: &[u8] = b"marketplace_authority";

// Size constants
pub const COLLECTION_SIZE: usize = 8 + // discriminator
    32 + // authority
    32 + // mint
    32 + // metadata
    32 + // master_edition
    32 + // symbol (String)
    64 + // name (String)
    200 + // uri (String)
    2 + // seller_fee_basis_points
    4 + (34 * 5) + // creators (Vec<Creator>, max 5)
    1 + // collection_type
    1 + 8 + // max_supply (Option<u64>)
    8 + // current_supply
    1 + // is_mutable
    8 + // created_at
    8 + // updated_at
    64; // padding

pub const NFT_METADATA_SIZE: usize = 8 + // discriminator
    32 + // mint
    1 + 32 + // collection (Option<Pubkey>)
    64 + // name (String)
    32 + // symbol (String)
    200 + // uri (String)
    2 + // seller_fee_basis_points
    4 + (34 * 5) + // creators (Vec<Creator>, max 5)
    1 + // nft_type
    4 + (64 * 10) + // attributes (Vec<Attribute>, max 10)
    1 + // rarity
    1 + 200 + // special_properties (Option<SpecialProperties>)
    4 + // usage_count
    1 + 4 + // max_usage (Option<u32>)
    1 + 8 + // expires_at (Option<i64>)
    8 + // created_at
    8 + // updated_at
    4 + // version
    128; // padding

// Marketplace constants
pub const MARKETPLACE_FEE_BPS: u16 = 250; // 2.5%
pub const MAX_CREATOR_ROYALTY_BPS: u16 = 1000; // 10%
pub const MIN_BID_INCREMENT_BPS: u16 = 500; // 5%

// Special card constants
pub const MAX_CARD_SYNERGY: u8 = 5;
pub const SYNERGY_BONUS_BPS: u16 = 1500; // 15% bonus per synergy level
```

## Error Codes

```rust
#[error_code]
pub enum FinovaNftError {
    #[msg("Invalid collection authority")]
    InvalidCollectionAuthority,
    
    #[msg("Collection supply limit exceeded")]
    SupplyLimitExceeded,
    
    #[msg("NFT not found")]
    NftNotFound,
    
    #[msg("Insufficient balance")]
    InsufficientBalance,
    
    #[msg("Not a special card")]
    NotSpecialCard,
    
    #[msg("Card usage limit exceeded")]
    ExceededMaxUsage,
    
    #[msg("Card has expired")]
    CardExpired,
    
    #[msg("Requirements not met")]
    RequirementsNotMet,
    
    #[msg("Invalid card combination")]
    InvalidCardCombination,
    
    #[msg("Marketplace listing not found")]
    ListingNotFound,
    
    #[msg("Listing has expired")]
    ListingExpired,
    
    #[msg("Invalid bid amount")]
    InvalidBidAmount,
    
    #[msg("Auction not ended")]
    AuctionNotEnded,
    
    #[msg("Reserve price not met")]
    ReservePriceNotMet,
    
    #[msg("Invalid marketplace fee")]
    InvalidMarketplaceFee,
    
    #[msg("Invalid creator royalty")]
    InvalidCreatorRoyalty,
    
    #[msg("Metadata update not allowed")]
    MetadataUpdateNotAllowed,
    
    #[msg("Invalid NFT type")]
    InvalidNftType,
    
    #[msg("Invalid special properties")]
    InvalidSpecialProperties,
    
    #[msg("Math overflow")]
    MathOverflow,
}
```

## Utility Functions

```rust
// Card synergy calculations
pub fn calculate_synergy_bonus(active_cards: &[SpecialCardType]) -> Result<u64> {
    let mut synergy_groups: std::collections::HashMap<u8, u32> = std::collections::HashMap::new();
    
    for card in active_cards {
        let group = get_synergy_group(card);
        *synergy_groups.entry(group).or_insert(0) += 1;
    }
    
    let mut total_bonus = 0u64;
    for (_, count) in synergy_groups {
        if count > 1 {
            let synergy_level = std::cmp::min(count - 1, MAX_CARD_SYNERGY as u32);
            total_bonus = total_bonus
                .checked_add(synergy_level as u64 * SYNERGY_BONUS_BPS as u64)
                .ok_or(FinovaNftError::MathOverflow)?;
        }
    }
    
    Ok(total_bonus)
}

// Rarity calculation
pub fn calculate_rarity(attributes: &[Attribute], total_supply: u64) -> Rarity {
    let mut rarity_score = 0f64;
    
    for attribute in attributes {
        if let Some(frequency) = attribute.frequency {
            rarity_score += 1.0 / (frequency as f64 / total_supply as f64);
        }
    }
    
    match rarity_score {
        x if x < 10.0 => Rarity::Common,
        x if x < 25.0 => Rarity::Uncommon,
        x if x < 50.0 => Rarity::Rare,
        x if x < 100.0 => Rarity::Epic,
        _ => Rarity::Legendary,
    }
}

// Price calculation utilities
pub fn calculate_marketplace_fee(price: u64) -> Result<u64> {
    price
        .checked_mul(MARKETPLACE_FEE_BPS as u64)
        .and_then(|x| x.checked_div(10000))
        .ok_or(FinovaNftError::MathOverflow)
}

pub fn calculate_creator_royalty(price: u64, royalty_bps: u16) -> Result<u64> {
    price
        .checked_mul(royalty_bps as u64)
        .and_then(|x| x.checked_div(10000))
        .ok_or(FinovaNftError::MathOverflow)
}

// Validation utilities
pub fn validate_special_properties(properties: &SpecialProperties) -> Result<()> {
    match properties.card_type {
        SpecialCardType::DoubleMining => {
            require!(properties.effect_value == 10000, FinovaNftError::InvalidSpecialProperties); // 100%
            require!(properties.duration.is_some(), FinovaNftError::InvalidSpecialProperties);
        },
        SpecialCardType::EternalMiner => {
            require!(properties.effect_value == 5000, FinovaNftError::InvalidSpecialProperties); // 50%
            require!(properties.duration.unwrap_or(0) >= 30 * 24 * 3600, FinovaNftError::InvalidSpecialProperties); // 30 days
        },
        _ => {} // Add validation for other card types
    }
    
    Ok(())
}
```

## Integration with Core Program

The NFT program integrates seamlessly with the Finova Core program through Cross-Program Invocations (CPI):

```rust
// Integration with mining system
pub fn apply_card_effect(
    ctx: Context<UseSpecialCard>,
    card_type: SpecialCardType,
    effect_value: u64,
    duration: Option<i64>,
) -> Result<()> {
    // Update card usage
    ctx.accounts.nft_metadata.usage_count += 1;
    ctx.accounts.nft_metadata.updated_at = Clock::get()?.unix_timestamp;
    
    // CPI to core program to apply mining boost
    let cpi_program = ctx.accounts.finova_core_program.to_account_info();
    let cpi_accounts = finova_core::cpi::accounts::ApplyCardBoost {
        user: ctx.accounts.user_account.to_account_info(),
        authority: ctx.accounts.user.to_account_info(),
    };
    let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
    
    finova_core::cpi::apply_card_boost(cpi_ctx, card_type, effect_value, duration)?;
    
    // Emit event
    emit!(SpecialCardUsed {
        mint: ctx.accounts.nft_mint.key(),
        user: ctx.accounts.user.key(),
        card_type,
        effect_value,
        duration,
        usage_count: ctx.accounts.nft_metadata.usage_count,
        timestamp: Clock::get()?.unix_timestamp,
    });
    
    Ok(())
}
```

## Security Features

### Access Control
- **Role-based permissions**: Different access levels for different operations
- **Multi-signature support**: Critical operations require multiple signatures
- **PDA validation**: All PDAs use secure seed patterns
- **Account ownership verification**: Strict ownership validation

### Economic Security
- **Fee validation**: All fees are validated against maximum limits
- **Overflow protection**: All mathematical operations are checked
- **Reentrancy protection**: State changes before external calls
- **Flash loan protection**: Time-based operation restrictions

### Data Integrity
- **Metadata validation**: All metadata follows strict schemas
- **Version control**: Metadata versioning for change tracking
- **Immutable records**: Critical data cannot be modified
- **Event logging**: All operations emit comprehensive events

## Testing

### Unit Tests
```bash
# Run all unit tests
anchor test

# Run specific test file
anchor test --skip-deploy tests/unit/mint_nft.ts

# Run with verbose output
anchor test --skip-deploy -- --verbose
```

### Integration Tests
```bash
# Run integration tests
npm run test:integration

# Run specific integration suite
npm run test:integration -- --grep "marketplace"

# Run with coverage
npm run test:coverage
```

### Security Tests
```bash
# Run security audit tests
npm run test:security

# Run fuzz tests
npm run test:fuzz

# Run overflow tests
npm run test:overflow
```

## Deployment

### Local Development
```bash
# Start local validator
solana-test-validator

# Build and deploy
anchor build
anchor deploy

# Initialize program
anchor run initialize
```

### Testnet Deployment
```bash
# Configure for testnet
solana config set --url https://api.testnet.solana.com

# Deploy to testnet
anchor deploy --provider.cluster testnet

# Verify deployment
anchor verify --provider.cluster testnet
```

### Mainnet Deployment
```bash
# Configure for mainnet
solana config set --url https://api.mainnet-beta.solana.com

# Deploy to mainnet (requires authority)
anchor deploy --provider.cluster mainnet-beta

# Verify and upgrade
anchor verify --provider.cluster mainnet-beta
anchor upgrade --provider.cluster mainnet-beta
```

## API Reference

### Client SDK Usage
```typescript
import { FinovaNftClient } from '@finova/nft-sdk';

// Initialize client
const client = new FinovaNftClient(connection, wallet);

// Create collection
const collection = await client.createCollection({
    name: "Finova Special Cards",
    symbol: "FSC",
    uri: "https://metadata.finova.network/collections/special-cards",
    sellerFeeBasisPoints: 500,
    creators: [{ address: authority.publicKey, verified: true, share: 100 }],
    collectionType: CollectionType.SpecialCards,
    maxSupply: 10000,
});

// Mint special card NFT
const nft = await client.mintNft({
    collection: collection.publicKey,
    name: "Double Mining Card",
    symbol: "DMC",
    uri: "https://metadata.finova.network/cards/double-mining",
    recipient: recipient.publicKey,
    nftType: NftType.SpecialCard,
    specialProperties: {
        cardType: SpecialCardType.DoubleMining,
        effectValue: 10000, // 100%
        duration: 24 * 60 * 60, // 24 hours
        synergyGroup: 1,
        stackable: true,
        transferable: true,
        consumable: false,
        requirements: [],
    },
});

// Use special card
await client.useSpecialCard({
    nftMint: nft.mint,
    user: user.publicKey,
});

// Create marketplace listing
const listing = await client.createListing({
    nftMint: nft.mint,
    price: 1000000000, // 1 SOL
    listingType: ListingType.FixedPrice,
    expiresAt: Date.now() + (7 * 24 * 60 * 60 * 1000), // 7 days
});

// Purchase from marketplace
await client.purchaseNft({
    listing: listing.publicKey,
    buyer: buyer.publicKey,
    paymentAmount: 1000000000,
});
```

## Performance Optimization

### Account Size Optimization
- **Packed structures**: Efficient memory usage with packed data
- **Optional fields**: Use Option<T> for optional data
- **String optimization**: Fixed-size strings where possible
- **Vector limits**: Reasonable limits on dynamic arrays

### Compute Optimization
- **Batch operations**: Group related operations
- **Minimal CPIs**: Only necessary cross-program invocations
- **Lazy loading**: Load only required account data
- **Efficient algorithms**: Optimized mathematical operations

### Storage Optimization
- **Data compression**: Compress large metadata fields
- **Reference patterns**: Use references instead of duplicating data
- **Cleanup routines**: Remove obsolete data automatically
- **Archive strategies**: Move old data to cheaper storage

## Monitoring and Analytics

### Key Metrics
```rust
// Program metrics structure
#[account]
pub struct ProgramMetrics {
    pub total_collections: u64,
    pub total_nfts_minted: u64,
    pub total_special_cards_used: u64,
    pub total_marketplace_volume: u64,
    pub active_listings: u64,
    pub total_fees_collected: u64,
    pub last_updated: i64,
}
```

### Event Tracking
- **Minting events**: Track NFT creation patterns
- **Usage events**: Monitor special card utilization
- **Trading events**: Analyze marketplace activity
- **User behavior**: Track engagement patterns

### Performance Monitoring
```typescript
// Client-side metrics collection
class FinovaNftMetrics {
    async trackMintingLatency(operation: () => Promise<any>) {
        const start = performance.now();
        const result = await operation();
        const latency = performance.now() - start;
        
        this.reportMetric('nft_mint_latency', latency);
        return result;
    }
    
    async trackMarketplaceVolume(price: number, currency: string) {
        this.reportMetric('marketplace_volume', price, { currency });
    }
    
    async trackCardUsage(cardType: SpecialCardType) {
        this.reportMetric('special_card_usage', 1, { cardType });
    }
}
```

## Advanced Features

### Fractional Ownership
```rust
// Fractional NFT support
#[account]
pub struct FractionalNft {
    pub parent_nft: Pubkey,
    pub total_shares: u64,
    pub share_mint: Pubkey,
    pub vault: Pubkey,
    pub curator: Pubkey,
    pub buyout_price: Option<u64>,
    pub created_at: i64,
}

pub fn fractionalize_nft(
    ctx: Context<FractionalizeNft>,
    total_shares: u64,
    share_price: u64,
) -> Result<()> {
    // Implementation for fractional ownership
    Ok(())
}
```

### Rental System
```rust
// NFT rental functionality
#[account]
pub struct NftRental {
    pub nft_mint: Pubkey,
    pub owner: Pubkey,
    pub renter: Pubkey,
    pub rental_price: u64,
    pub rental_duration: i64,
    pub start_time: i64,
    pub end_time: i64,
    pub collateral: u64,
    pub is_active: bool,
}

pub fn create_rental_listing(
    ctx: Context<CreateRentalListing>,
    rental_price: u64,
    rental_duration: i64,
    collateral: u64,
) -> Result<()> {
    // Implementation for NFT rentals
    Ok(())
}
```

### Dynamic NFTs
```rust
// Dynamic NFT properties that change over time
#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct DynamicProperties {
    pub level: u32,
    pub experience: u64,
    pub evolution_stage: u8,
    pub last_interaction: i64,
    pub upgrade_requirements: Vec<Requirement>,
}

pub fn evolve_nft(
    ctx: Context<EvolveNft>,
    new_stage: u8,
) -> Result<()> {
    // Implementation for NFT evolution
    Ok(())
}
```

## Cross-Chain Integration

### Wormhole Bridge Support
```rust
// Cross-chain NFT transfer preparation
pub fn prepare_cross_chain_transfer(
    ctx: Context<PrepareCrossChainTransfer>,
    target_chain: u16,
    recipient: [u8; 32],
) -> Result<()> {
    // Lock NFT for cross-chain transfer
    let nft_metadata = &mut ctx.accounts.nft_metadata;
    nft_metadata.is_locked = true;
    nft_metadata.lock_reason = LockReason::CrossChainTransfer;
    nft_metadata.lock_expiry = Clock::get()?.unix_timestamp + 3600; // 1 hour
    
    // Emit cross-chain event
    emit!(CrossChainTransferInitiated {
        nft_mint: ctx.accounts.nft_mint.key(),
        owner: ctx.accounts.owner.key(),
        target_chain,
        recipient,
        timestamp: Clock::get()?.unix_timestamp,
    });
    
    Ok(())
}
```

### Multi-Chain Metadata
```rust
// Cross-chain metadata synchronization
#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct CrossChainMetadata {
    pub original_chain: u16,
    pub original_address: [u8; 32],
    pub bridge_timestamp: i64,
    pub sync_status: SyncStatus,
    pub verification_hash: [u8; 32],
}
```

## Governance Integration

### DAO Voting with NFTs
```rust
// NFT-based governance voting
pub fn cast_governance_vote(
    ctx: Context<CastGovernanceVote>,
    proposal_id: u64,
    vote: VoteChoice,
) -> Result<()> {
    // Calculate voting power based on NFT holdings
    let voting_power = calculate_nft_voting_power(&ctx.accounts.user_nfts)?;
    
    // Record vote with weight
    emit!(GovernanceVotecast {
        voter: ctx.accounts.voter.key(),
        proposal_id,
        vote,
        voting_power,
        timestamp: Clock::get()?.unix_timestamp,
    });
    
    Ok(())
}

fn calculate_nft_voting_power(nfts: &[Account<FinovaNftMetadata>]) -> Result<u64> {
    let mut total_power = 0u64;
    
    for nft in nfts {
        let power = match nft.nft_type {
            NftType::ProfileBadge => match nft.rarity {
                Rarity::Legendary => 1000,
                Rarity::Epic => 500,
                Rarity::Rare => 100,
                Rarity::Uncommon => 50,
                Rarity::Common => 10,
            },
            NftType::Achievement => 200,
            NftType::SpecialCard => 0, // Cards don't vote
        };
        
        total_power = total_power.checked_add(power).ok_or(FinovaNftError::MathOverflow)?;
    }
    
    Ok(total_power)
}
```

## Mobile SDK Integration

### React Native Example
```typescript
// React Native integration
import { FinovaNftMobile } from '@finova/nft-mobile-sdk';

const FinovaCardScreen = () => {
    const [cards, setCards] = useState([]);
    const { wallet } = useWallet();
    
    useEffect(() => {
        loadUserCards();
    }, [wallet]);
    
    const loadUserCards = async () => {
        try {
            const client = new FinovaNftMobile(wallet);
            const userCards = await client.getUserSpecialCards();
            setCards(userCards);
        } catch (error) {
            console.error('Failed to load cards:', error);
        }
    };
    
    const useCard = async (cardMint: string) => {
        try {
            const client = new FinovaNftMobile(wallet);
            await client.useSpecialCard(cardMint);
            
            // Refresh cards list
            await loadUserCards();
            
            // Show success notification
            showNotification('Card used successfully!');
        } catch (error) {
            showError('Failed to use card');
        }
    };
    
    return (
        <ScrollView>
            {cards.map(card => (
                <CardComponent
                    key={card.mint}
                    card={card}
                    onUse={() => useCard(card.mint)}
                />
            ))}
        </ScrollView>
    );
};
```

### iOS Swift Integration
```swift
// iOS SDK integration
import FinovaNftSDK

class CardViewController: UIViewController {
    @IBOutlet weak var cardsCollectionView: UICollectionView!
    
    private let nftClient = FinovaNftClient()
    private var specialCards: [SpecialCard] = []
    
    override func viewDidLoad() {
        super.viewDidLoad()
        loadSpecialCards()
    }
    
    private func loadSpecialCards() {
        Task {
            do {
                specialCards = try await nftClient.getSpecialCards()
                await MainActor.run {
                    cardsCollectionView.reloadData()
                }
            } catch {
                print("Failed to load special cards: \(error)")
            }
        }
    }
    
    @IBAction func useCardTapped(_ sender: UIButton) {
        let cardIndex = sender.tag
        let card = specialCards[cardIndex]
        
        Task {
            do {
                try await nftClient.useSpecialCard(mint: card.mint)
                await MainActor.run {
                    showSuccessAlert("Card used successfully!")
                    loadSpecialCards() // Refresh list
                }
            } catch {
                await MainActor.run {
                    showErrorAlert("Failed to use card")
                }
            }
        }
    }
}
```

## Web3 Integration Examples

### MetaMask Integration
```typescript
// Web3 wallet integration
import { Connection, PublicKey } from '@solana/web3.js';
import { AnchorProvider, Program } from '@project-serum/anchor';

class FinovaNftWeb3 {
    constructor(private connection: Connection, private wallet: any) {}
    
    async connectWallet() {
        if (window.solana) {
            try {
                const response = await window.solana.connect();
                return new PublicKey(response.publicKey.toString());
            } catch (error) {
                throw new Error('Failed to connect wallet');
            }
        } else {
            throw new Error('Solana wallet not found');
        }
    }
    
    async getProgram() {
        const provider = new AnchorProvider(this.connection, this.wallet, {});
        return new Program(IDL, PROGRAM_ID, provider);
    }
    
    async mintSpecialCard(params: MintSpecialCardParams) {
        const program = await this.getProgram();
        
        return await program.methods
            .mintNft(params)
            .accounts({
                authority: this.wallet.publicKey,
                // ... other accounts
            })
            .rpc();
    }
}
```

## Error Handling and Recovery

### Comprehensive Error Recovery
```rust
// Error recovery mechanisms
pub fn recover_failed_transaction(
    ctx: Context<RecoverFailedTransaction>,
    transaction_id: [u8; 32],
) -> Result<()> {
    let recovery_account = &mut ctx.accounts.recovery_account;
    
    // Validate recovery conditions
    require!(
        Clock::get()?.unix_timestamp > recovery_account.failure_timestamp + 3600,
        FinovaNftError::RecoveryTooEarly
    );
    
    // Restore account state
    match recovery_account.operation_type {
        OperationType::Mint => {
            // Refund minting fees
            **ctx.accounts.payer.to_account_info().try_borrow_mut_lamports()? +=
                recovery_account.refund_amount;
        },
        OperationType::Transfer => {
            // Restore NFT ownership
            // Implementation details...
        },
        OperationType::Marketplace => {
            // Restore marketplace state
            // Implementation details...
        },
    }
    
    recovery_account.is_recovered = true;
    recovery_account.recovery_timestamp = Clock::get()?.unix_timestamp;
    
    emit!(TransactionRecovered {
        transaction_id,
        user: ctx.accounts.user.key(),
        operation_type: recovery_account.operation_type,
        timestamp: Clock::get()?.unix_timestamp,
    });
    
    Ok(())
}
```

### Client-Side Error Handling
```typescript
// Robust error handling in client
class ErrorHandler {
    static async handleNftOperation<T>(
        operation: () => Promise<T>,
        retryCount: number = 3
    ): Promise<T> {
        let lastError: Error;
        
        for (let i = 0; i < retryCount; i++) {
            try {
                return await operation();
            } catch (error) {
                lastError = error as Error;
                
                if (this.isRetryableError(error)) {
                    await this.delay(Math.pow(2, i) * 1000); // Exponential backoff
                    continue;
                } else {
                    throw error; // Non-retryable error
                }
            }
        }
        
        throw lastError!;
    }
    
    private static isRetryableError(error: any): boolean {
        const retryableErrors = [
            'Network request failed',
            'Transaction simulation failed',
            'Blockhash not found',
        ];
        
        return retryableErrors.some(msg => 
            error.message?.includes(msg)
        );
    }
    
    private static delay(ms: number): Promise<void> {
        return new Promise(resolve => setTimeout(resolve, ms));
    }
}
```

## Deployment Checklist

### Pre-Deployment Validation
- [ ] All smart contracts compiled without warnings
- [ ] Comprehensive test suite passes (100% coverage)
- [ ] Security audit completed and approved
- [ ] Gas optimization validated
- [ ] Integration tests with other Finova programs
- [ ] Mobile SDK compatibility verified
- [ ] Documentation updated and reviewed

### Deployment Steps
1. **Environment Setup**
   ```bash
   # Set network configuration
   solana config set --url https://api.mainnet-beta.solana.com
   
   # Verify wallet balance
   solana balance
   
   # Build optimized program
   anchor build --verifiable
   ```

2. **Security Verification**
   ```bash
   # Run security tests
   npm run test:security
   
   # Verify program hash
   anchor verify --provider.cluster mainnet-beta
   ```

3. **Gradual Rollout**
   ```bash
   # Deploy to mainnet
   anchor deploy --provider.cluster mainnet-beta
   
   # Initialize with conservative limits
   anchor run initialize --provider.cluster mainnet-beta
   ```

4. **Post-Deployment Monitoring**
   ```bash
   # Monitor program logs
   solana logs --url mainnet-beta <PROGRAM_ID>
   
   # Track metrics
   npm run monitor:production
   ```

### Rollback Procedures
```rust
// Emergency pause functionality
pub fn emergency_pause(ctx: Context<EmergencyPause>) -> Result<()> {
    require!(
        ctx.accounts.authority.key() == EMERGENCY_AUTHORITY,
        FinovaNftError::UnauthorizedEmergencyAction
    );
    
    let program_state = &mut ctx.accounts.program_state;
    program_state.is_paused = true;
    program_state.pause_reason = "Emergency maintenance".to_string();
    program_state.paused_at = Clock::get()?.unix_timestamp;
    
    emit!(ProgramPaused {
        authority: ctx.accounts.authority.key(),
        reason: program_state.pause_reason.clone(),
        timestamp: program_state.paused_at,
    });
    
    Ok(())
}
```

## Contributing

### Development Setup
```bash
# Clone repository
git clone https://github.com/finova-network/finova-contracts.git
cd finova-contracts

# Install dependencies
npm install
cargo install --version 0.28.0 anchor-cli

# Setup development environment
cp .env.example .env
anchor build
anchor deploy --provider.cluster localnet
```

### Code Standards
- **Rust Formatting**: Use `rustfmt` with project configuration
- **TypeScript**: Follow ESLint rules and Prettier formatting
- **Documentation**: All public functions must have documentation
- **Testing**: Minimum 95% test coverage required
- **Security**: All PRs must pass security scans

### Pull Request Process
1. Fork the repository
2. Create feature branch (`git checkout -b feature/amazing-feature`)
3. Commit changes (`git commit -m 'Add amazing feature'`)
4. Push to branch (`git push origin feature/amazing-feature`)
5. Open Pull Request with detailed description
6. Pass all CI/CD checks
7. Get code review approval
8. Merge to main branch

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Support

### Documentation Resources
- [Anchor Framework Documentation](https://anchor-lang.com/)
- [Solana Documentation](https://docs.solana.com/)
- [Metaplex Documentation](https://docs.metaplex.com/)

### Community Support
- **Discord**: Join our [Discord server](https://discord.gg/finova)
- **Telegram**: [Finova Network Community](https://t.me/finovanetwork)
- **GitHub Discussions**: Use GitHub Discussions for questions
- **Email Support**: technical-support@finova.network

### Professional Support
For enterprise integration and custom development:
- **Email**: enterprise@finova.network
- **Calendar**: [Schedule consultation](https://calendly.com/finova-enterprise)

---

**Built with ❤️ by the Finova Network Team**

*Empowering the future of Social-Fi through innovative NFT solutions*