// programs/finova-nft/src/state/marketplace.rs

use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, TokenAccount};
use crate::constants::*;
use crate::errors::FinovaNftError;

/// Marketplace configuration and global settings
#[account]
#[derive(Default)]
pub struct Marketplace {
    /// Authority that can update marketplace settings
    pub authority: Pubkey,
    /// Treasury account for collecting fees
    pub treasury: Pubkey,
    /// Fee basis points (e.g., 250 = 2.5%)
    pub fee_basis_points: u16,
    /// Royalty basis points for creators (e.g., 500 = 5%)
    pub royalty_basis_points: u16,
    /// Whether the marketplace is paused
    pub is_paused: bool,
    /// Total volume traded (in lamports)
    pub total_volume: u64,
    /// Total number of trades
    pub total_trades: u64,
    /// Total fees collected
    pub total_fees: u64,
    /// Total royalties paid
    pub total_royalties: u64,
    /// Minimum listing price (in lamports)
    pub min_listing_price: u64,
    /// Maximum listing price (in lamports)
    pub max_listing_price: u64,
    /// Supported payment tokens count
    pub supported_tokens_count: u8,
    /// Reserved space for future upgrades
    pub reserved: [u8; 64],
    /// Bump seed for PDA
    pub bump: u8,
}

impl Marketplace {
    pub const SPACE: usize = 8 + // discriminator
        32 + // authority
        32 + // treasury
        2 + // fee_basis_points
        2 + // royalty_basis_points
        1 + // is_paused
        8 + // total_volume
        8 + // total_trades
        8 + // total_fees
        8 + // total_royalties
        8 + // min_listing_price
        8 + // max_listing_price
        1 + // supported_tokens_count
        64 + // reserved
        1; // bump

    /// Initialize marketplace with default settings
    pub fn initialize(
        &mut self,
        authority: Pubkey,
        treasury: Pubkey,
        bump: u8,
    ) -> Result<()> {
        self.authority = authority;
        self.treasury = treasury;
        self.fee_basis_points = DEFAULT_MARKETPLACE_FEE_BPS;
        self.royalty_basis_points = DEFAULT_ROYALTY_BPS;
        self.is_paused = false;
        self.total_volume = 0;
        self.total_trades = 0;
        self.total_fees = 0;
        self.total_royalties = 0;
        self.min_listing_price = MIN_LISTING_PRICE;
        self.max_listing_price = MAX_LISTING_PRICE;
        self.supported_tokens_count = 1; // SOL by default
        self.bump = bump;
        Ok(())
    }

    /// Update marketplace settings
    pub fn update_settings(
        &mut self,
        fee_basis_points: Option<u16>,
        royalty_basis_points: Option<u16>,
        min_listing_price: Option<u64>,
        max_listing_price: Option<u64>,
    ) -> Result<()> {
        if let Some(fee_bps) = fee_basis_points {
            require!(fee_bps <= MAX_FEE_BASIS_POINTS, FinovaNftError::InvalidFeeRate);
            self.fee_basis_points = fee_bps;
        }

        if let Some(royalty_bps) = royalty_basis_points {
            require!(royalty_bps <= MAX_ROYALTY_BASIS_POINTS, FinovaNftError::InvalidRoyaltyRate);
            self.royalty_basis_points = royalty_bps;
        }

        if let Some(min_price) = min_listing_price {
            require!(min_price > 0, FinovaNftError::InvalidPrice);
            require!(min_price < self.max_listing_price, FinovaNftError::InvalidPriceRange);
            self.min_listing_price = min_price;
        }

        if let Some(max_price) = max_listing_price {
            require!(max_price > self.min_listing_price, FinovaNftError::InvalidPriceRange);
            self.max_listing_price = max_price;
        }

        Ok(())
    }

    /// Record a completed trade
    pub fn record_trade(&mut self, price: u64, fee: u64, royalty: u64) -> Result<()> {
        self.total_volume = self.total_volume.checked_add(price)
            .ok_or(FinovaNftError::MathOverflow)?;
        self.total_trades = self.total_trades.checked_add(1)
            .ok_or(FinovaNftError::MathOverflow)?;
        self.total_fees = self.total_fees.checked_add(fee)
            .ok_or(FinovaNftError::MathOverflow)?;
        self.total_royalties = self.total_royalties.checked_add(royalty)
            .ok_or(FinovaNftError::MathOverflow)?;
        Ok(())
    }

    /// Calculate marketplace fee
    pub fn calculate_fee(&self, price: u64) -> Result<u64> {
        price.checked_mul(self.fee_basis_points as u64)
            .and_then(|x| x.checked_div(10000))
            .ok_or_else(|| FinovaNftError::MathOverflow.into())
    }

    /// Calculate royalty fee
    pub fn calculate_royalty(&self, price: u64) -> Result<u64> {
        price.checked_mul(self.royalty_basis_points as u64)
            .and_then(|x| x.checked_div(10000))
            .ok_or_else(|| FinovaNftError::MathOverflow.into())
    }
}

/// Individual NFT listing on the marketplace
#[account]
#[derive(Default)]
pub struct Listing {
    /// The NFT mint being sold
    pub nft_mint: Pubkey,
    /// Current owner/seller
    pub seller: Pubkey,
    /// Price in lamports or specified token
    pub price: u64,
    /// Payment token mint (System Program for SOL)
    pub payment_token: Pubkey,
    /// When the listing was created
    pub created_at: i64,
    /// When the listing expires (0 = never)
    pub expires_at: i64,
    /// Whether this is an auction (true) or fixed price (false)
    pub is_auction: bool,
    /// Current highest bid (for auctions)
    pub highest_bid: u64,
    /// Current highest bidder (for auctions)
    pub highest_bidder: Pubkey,
    /// Auction end time
    pub auction_end_time: i64,
    /// Reserve price for auctions
    pub reserve_price: u64,
    /// Listing status
    pub status: ListingStatus,
    /// Total number of bids (for auctions)
    pub bid_count: u32,
    /// Listing category/type
    pub category: ListingCategory,
    /// Special card type if applicable
    pub special_card_type: SpecialCardType,
    /// Rarity tier
    pub rarity: RarityTier,
    /// Metadata URI
    pub metadata_uri: String,
    /// Reserved space for future features
    pub reserved: [u8; 32],
    /// Bump seed
    pub bump: u8,
}

impl Listing {
    pub const SPACE: usize = 8 + // discriminator
        32 + // nft_mint
        32 + // seller
        8 + // price
        32 + // payment_token
        8 + // created_at
        8 + // expires_at
        1 + // is_auction
        8 + // highest_bid
        32 + // highest_bidder
        8 + // auction_end_time
        8 + // reserve_price
        1 + // status
        4 + // bid_count
        1 + // category
        1 + // special_card_type
        1 + // rarity
        200 + // metadata_uri (max length)
        32 + // reserved
        1; // bump

    /// Create a new fixed-price listing
    pub fn create_fixed_price(
        &mut self,
        nft_mint: Pubkey,
        seller: Pubkey,
        price: u64,
        payment_token: Pubkey,
        expires_at: i64,
        category: ListingCategory,
        special_card_type: SpecialCardType,
        rarity: RarityTier,
        metadata_uri: String,
        bump: u8,
    ) -> Result<()> {
        let now = Clock::get()?.unix_timestamp;
        
        require!(price > 0, FinovaNftError::InvalidPrice);
        require!(expires_at == 0 || expires_at > now, FinovaNftError::InvalidExpirationTime);
        require!(metadata_uri.len() <= 200, FinovaNftError::MetadataUriTooLong);

        self.nft_mint = nft_mint;
        self.seller = seller;
        self.price = price;
        self.payment_token = payment_token;
        self.created_at = now;
        self.expires_at = expires_at;
        self.is_auction = false;
        self.highest_bid = 0;
        self.highest_bidder = Pubkey::default();
        self.auction_end_time = 0;
        self.reserve_price = 0;
        self.status = ListingStatus::Active;
        self.bid_count = 0;
        self.category = category;
        self.special_card_type = special_card_type;
        self.rarity = rarity;
        self.metadata_uri = metadata_uri;
        self.bump = bump;

        Ok(())
    }

    /// Create a new auction listing
    pub fn create_auction(
        &mut self,
        nft_mint: Pubkey,
        seller: Pubkey,
        starting_price: u64,
        reserve_price: u64,
        payment_token: Pubkey,
        auction_duration: i64,
        category: ListingCategory,
        special_card_type: SpecialCardType,
        rarity: RarityTier,
        metadata_uri: String,
        bump: u8,
    ) -> Result<()> {
        let now = Clock::get()?.unix_timestamp;
        
        require!(starting_price > 0, FinovaNftError::InvalidPrice);
        require!(reserve_price >= starting_price, FinovaNftError::InvalidReservePrice);
        require!(auction_duration > 0, FinovaNftError::InvalidAuctionDuration);
        require!(metadata_uri.len() <= 200, FinovaNftError::MetadataUriTooLong);

        self.nft_mint = nft_mint;
        self.seller = seller;
        self.price = starting_price;
        self.payment_token = payment_token;
        self.created_at = now;
        self.expires_at = 0;
        self.is_auction = true;
        self.highest_bid = 0;
        self.highest_bidder = Pubkey::default();
        self.auction_end_time = now.checked_add(auction_duration)
            .ok_or(FinovaNftError::MathOverflow)?;
        self.reserve_price = reserve_price;
        self.status = ListingStatus::Active;
        self.bid_count = 0;
        self.category = category;
        self.special_card_type = special_card_type;
        self.rarity = rarity;
        self.metadata_uri = metadata_uri;
        self.bump = bump;

        Ok(())
    }

    /// Place a bid on an auction
    pub fn place_bid(&mut self, bidder: Pubkey, bid_amount: u64) -> Result<()> {
        require!(self.is_auction, FinovaNftError::NotAnAuction);
        require!(self.status == ListingStatus::Active, FinovaNftError::ListingNotActive);
        
        let now = Clock::get()?.unix_timestamp;
        require!(now < self.auction_end_time, FinovaNftError::AuctionEnded);
        require!(bid_amount > self.highest_bid, FinovaNftError::BidTooLow);
        require!(bid_amount >= self.price, FinovaNftError::BidBelowStartingPrice);

        self.highest_bid = bid_amount;
        self.highest_bidder = bidder;
        self.bid_count = self.bid_count.checked_add(1)
            .ok_or(FinovaNftError::MathOverflow)?;

        // Extend auction if bid is placed in last 10 minutes
        let time_left = self.auction_end_time.checked_sub(now)
            .ok_or(FinovaNftError::MathOverflow)?;
        
        if time_left < AUCTION_EXTENSION_THRESHOLD {
            self.auction_end_time = now.checked_add(AUCTION_EXTENSION_TIME)
                .ok_or(FinovaNftError::MathOverflow)?;
        }

        Ok(())
    }

    /// Cancel the listing
    pub fn cancel(&mut self, canceller: Pubkey) -> Result<()> {
        require!(canceller == self.seller, FinovaNftError::UnauthorizedCancellation);
        require!(self.status == ListingStatus::Active, FinovaNftError::ListingNotActive);
        
        if self.is_auction {
            require!(self.highest_bid == 0, FinovaNftError::CannotCancelAuctionWithBids);
        }

        self.status = ListingStatus::Cancelled;
        Ok(())
    }

    /// Complete the sale
    pub fn complete_sale(&mut self, buyer: Pubkey) -> Result<()> {
        require!(self.status == ListingStatus::Active, FinovaNftError::ListingNotActive);

        if self.is_auction {
            let now = Clock::get()?.unix_timestamp;
            require!(now >= self.auction_end_time, FinovaNftError::AuctionNotEnded);
            require!(self.highest_bid >= self.reserve_price, FinovaNftError::ReservePriceNotMet);
            require!(buyer == self.highest_bidder, FinovaNftError::UnauthorizedBuyer);
        }

        // Check if fixed-price listing has expired
        if !self.is_auction && self.expires_at > 0 {
            let now = Clock::get()?.unix_timestamp;
            require!(now < self.expires_at, FinovaNftError::ListingExpired);
        }

        self.status = ListingStatus::Sold;
        Ok(())
    }

    /// Check if listing is active and valid
    pub fn is_valid(&self) -> bool {
        if self.status != ListingStatus::Active {
            return false;
        }

        let now = Clock::get().map(|c| c.unix_timestamp).unwrap_or(0);

        if self.is_auction {
            now < self.auction_end_time
        } else {
            self.expires_at == 0 || now < self.expires_at
        }
    }

    /// Get the current effective price
    pub fn get_current_price(&self) -> u64 {
        if self.is_auction && self.highest_bid > 0 {
            self.highest_bid
        } else {
            self.price
        }
    }
}

/// Bid record for auction tracking
#[account]
#[derive(Default)]
pub struct Bid {
    /// The listing this bid is for
    pub listing: Pubkey,
    /// The bidder
    pub bidder: Pubkey,
    /// Bid amount
    pub amount: u64,
    /// When the bid was placed
    pub created_at: i64,
    /// Whether this bid is still active
    pub is_active: bool,
    /// Refund status
    pub refund_status: RefundStatus,
    /// Transaction signature when bid was placed
    pub transaction_signature: String,
    /// Reserved space
    pub reserved: [u8; 32],
    /// Bump seed
    pub bump: u8,
}

impl Bid {
    pub const SPACE: usize = 8 + // discriminator
        32 + // listing
        32 + // bidder
        8 + // amount
        8 + // created_at
        1 + // is_active
        1 + // refund_status
        88 + // transaction_signature (base58 signature length)
        32 + // reserved
        1; // bump

    pub fn initialize(
        &mut self,
        listing: Pubkey,
        bidder: Pubkey,
        amount: u64,
        transaction_signature: String,
        bump: u8,
    ) -> Result<()> {
        let now = Clock::get()?.unix_timestamp;
        
        self.listing = listing;
        self.bidder = bidder;
        self.amount = amount;
        self.created_at = now;
        self.is_active = true;
        self.refund_status = RefundStatus::Pending;
        self.transaction_signature = transaction_signature;
        self.bump = bump;

        Ok(())
    }

    pub fn mark_refunded(&mut self) -> Result<()> {
        require!(self.is_active, FinovaNftError::BidNotActive);
        self.is_active = false;
        self.refund_status = RefundStatus::Completed;
        Ok(())
    }
}

/// Trade history record
#[account]
#[derive(Default)]
pub struct TradeHistory {
    /// The NFT that was traded
    pub nft_mint: Pubkey,
    /// Seller address
    pub seller: Pubkey,
    /// Buyer address
    pub buyer: Pubkey,
    /// Sale price
    pub price: u64,
    /// Payment token used
    pub payment_token: Pubkey,
    /// Marketplace fee paid
    pub marketplace_fee: u64,
    /// Royalty fee paid
    pub royalty_fee: u64,
    /// Transaction timestamp
    pub traded_at: i64,
    /// Whether it was an auction sale
    pub was_auction: bool,
    /// Number of bids (if auction)
    pub bid_count: u32,
    /// Transaction signature
    pub transaction_signature: String,
    /// NFT category
    pub category: ListingCategory,
    /// Special card type
    pub special_card_type: SpecialCardType,
    /// Rarity tier
    pub rarity: RarityTier,
    /// Reserved space
    pub reserved: [u8; 32],
    /// Bump seed
    pub bump: u8,
}

impl TradeHistory {
    pub const SPACE: usize = 8 + // discriminator
        32 + // nft_mint
        32 + // seller
        32 + // buyer
        8 + // price
        32 + // payment_token
        8 + // marketplace_fee
        8 + // royalty_fee
        8 + // traded_at
        1 + // was_auction
        4 + // bid_count
        88 + // transaction_signature
        1 + // category
        1 + // special_card_type
        1 + // rarity
        32 + // reserved
        1; // bump
}

/// User's marketplace profile and statistics
#[account]
#[derive(Default)]
pub struct UserProfile {
    /// User's wallet address
    pub user: Pubkey,
    /// Total number of NFTs sold
    pub nfts_sold: u32,
    /// Total number of NFTs bought
    pub nfts_bought: u32,
    /// Total volume sold (in SOL)
    pub total_sold_volume: u64,
    /// Total volume bought (in SOL)
    pub total_bought_volume: u64,
    /// Total fees paid as buyer
    pub total_fees_paid: u64,
    /// Total royalties earned as creator
    pub total_royalties_earned: u64,
    /// Average sale price
    pub average_sale_price: u64,
    /// Average purchase price
    pub average_purchase_price: u64,
    /// User reputation score (0-100)
    pub reputation_score: u8,
    /// Number of successful trades
    pub successful_trades: u32,
    /// Number of cancelled listings
    pub cancelled_listings: u32,
    /// First trade timestamp
    pub first_trade_at: i64,
    /// Last trade timestamp
    pub last_trade_at: i64,
    /// User tier based on activity
    pub user_tier: UserTier,
    /// Special badges earned
    pub badges: Vec<Badge>,
    /// Reserved space
    pub reserved: [u8; 64],
    /// Bump seed
    pub bump: u8,
}

impl UserProfile {
    pub const SPACE: usize = 8 + // discriminator
        32 + // user
        4 + // nfts_sold
        4 + // nfts_bought
        8 + // total_sold_volume
        8 + // total_bought_volume
        8 + // total_fees_paid
        8 + // total_royalties_earned
        8 + // average_sale_price
        8 + // average_purchase_price
        1 + // reputation_score
        4 + // successful_trades
        4 + // cancelled_listings
        8 + // first_trade_at
        8 + // last_trade_at
        1 + // user_tier
        4 + 10 * 1 + // badges (max 10 badges)
        64 + // reserved
        1; // bump

    pub fn update_after_sale(
        &mut self,
        price: u64,
        marketplace_fee: u64,
        royalty_fee: u64,
        is_seller: bool,
    ) -> Result<()> {
        let now = Clock::get()?.unix_timestamp;
        
        if is_seller {
            self.nfts_sold = self.nfts_sold.checked_add(1)
                .ok_or(FinovaNftError::MathOverflow)?;
            self.total_sold_volume = self.total_sold_volume.checked_add(price)
                .ok_or(FinovaNftError::MathOverflow)?;
            self.total_royalties_earned = self.total_royalties_earned.checked_add(royalty_fee)
                .ok_or(FinovaNftError::MathOverflow)?;
                
            if self.nfts_sold > 0 {
                self.average_sale_price = self.total_sold_volume / self.nfts_sold as u64;
            }
        } else {
            self.nfts_bought = self.nfts_bought.checked_add(1)
                .ok_or(FinovaNftError::MathOverflow)?;
            self.total_bought_volume = self.total_bought_volume.checked_add(price)
                .ok_or(FinovaNftError::MathOverflow)?;
            self.total_fees_paid = self.total_fees_paid.checked_add(marketplace_fee)
                .ok_or(FinovaNftError::MathOverflow)?;
                
            if self.nfts_bought > 0 {
                self.average_purchase_price = self.total_bought_volume / self.nfts_bought as u64;
            }
        }

        self.successful_trades = self.successful_trades.checked_add(1)
            .ok_or(FinovaNftError::MathOverflow)?;
        
        if self.first_trade_at == 0 {
            self.first_trade_at = now;
        }
        self.last_trade_at = now;

        // Update user tier based on activity
        self.update_user_tier()?;
        
        // Update reputation score
        self.update_reputation_score()?;

        Ok(())
    }

    fn update_user_tier(&mut self) -> Result<()> {
        let total_trades = self.successful_trades;
        let total_volume = self.total_sold_volume + self.total_bought_volume;

        self.user_tier = match (total_trades, total_volume) {
            (0..=5, _) => UserTier::Bronze,
            (6..=20, 0..=10_000_000_000) => UserTier::Silver, // < 10 SOL
            (21..=50, 10_000_000_001..=100_000_000_000) => UserTier::Gold, // 10-100 SOL
            (51..=100, 100_000_000_001..=1_000_000_000_000) => UserTier::Platinum, // 100-1000 SOL
            _ => UserTier::Diamond,
        };

        Ok(())
    }

    fn update_reputation_score(&mut self) -> Result<()> {
        let total_actions = self.successful_trades + self.cancelled_listings;
        if total_actions == 0 {
            self.reputation_score = 50; // Default score
            return Ok();
        }

        let success_rate = (self.successful_trades * 100) / total_actions;
        let volume_bonus = std::cmp::min(10, (self.total_sold_volume / 1_000_000_000) as u8); // 1 point per SOL, max 10
        
        self.reputation_score = std::cmp::min(100, 
            (success_rate as u8).saturating_add(volume_bonus)
        );

        Ok(())
    }
}

/// Listing status enumeration
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq)]
pub enum ListingStatus {
    Active,
    Sold,
    Cancelled,
    Expired,
}

impl Default for ListingStatus {
    fn default() -> Self {
        ListingStatus::Active
    }
}

/// Listing category enumeration
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq)]
pub enum ListingCategory {
    SpecialCard,
    ProfileBadge,
    Achievement,
    Collectible,
    Utility,
}

impl Default for ListingCategory {
    fn default() -> Self {
        ListingCategory::Collectible
    }
}

/// Special card type enumeration
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq)]
pub enum SpecialCardType {
    None,
    DoubleMining,
    TripleMining,
    MiningFrenzy,
    EternalMiner,
    XpDouble,
    StreakSaver,
    LevelRush,
    XpMagnet,
    ReferralBoost,
    NetworkAmplifier,
    AmbassadorPass,
    NetworkKing,
}

impl Default for SpecialCardType {
    fn default() -> Self {
        SpecialCardType::None
    }
}

/// Rarity tier enumeration
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq)]
pub enum RarityTier {
    Common,
    Uncommon,
    Rare,
    Epic,
    Legendary,
    Mythic,
}

impl Default for RarityTier {
    fn default() -> Self {
        RarityTier::Common
    }
}

/// Refund status enumeration
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq)]
pub enum RefundStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
}

impl Default for RefundStatus {
    fn default() -> Self {
        RefundStatus::Pending
    }
}

/// User tier enumeration
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq)]
pub enum UserTier {
    Bronze,
    Silver,
    Gold,
    Platinum,
    Diamond,
}

impl Default for UserTier {
    fn default() -> Self {
        UserTier::Bronze
    }
}

/// Badge enumeration
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq)]
pub enum Badge {
    FirstSale,
    FirstPurchase,
    HighVolume,
    FrequentTrader,
    RareCollector,
    EarlyAdopter,
    TrustedSeller,
    PowerBuyer,
}

impl Default for Badge {
    fn default() -> Self {
        Badge::FirstSale
    }
}

/// Marketplace analytics and metrics
#[account]
#[derive(Default)]
pub struct MarketplaceAnalytics {
    /// Daily trading volume
    pub daily_volume: u64,
    /// Weekly trading volume
    pub weekly_volume: u64,
    /// Monthly trading volume
    pub monthly_volume: u64,
    /// Daily trade count
    pub daily_trades: u32,
    /// Weekly trade count
    pub weekly_trades: u32,
    /// Monthly trade count
    pub monthly_trades: u32,
    /// Average sale price (last 24h)
    pub daily_avg_price: u64,
    /// Most expensive sale (last 24h)
    pub daily_highest_sale: u64,
    /// Cheapest sale (last 24h)
    pub daily_lowest_sale: u64,
    /// Most active collection (by volume)
    pub top_collection: Pubkey,
    /// Top collection volume
    pub top_collection_volume: u64,
    /// Last update timestamp
    pub last_updated: i64,
    /// Reserved space
    pub reserved: [u8; 64],
    /// Bump seed
    pub bump: u8,
}

impl MarketplaceAnalytics {
    pub const SPACE: usize = 8 + // discriminator
        8 + // daily_volume
        8 + // weekly_volume
        8 + // monthly_volume
        4 + // daily_trades
        4 + // weekly_trades
        4 + // monthly_trades
        8 + // daily_avg_price
        8 + // daily_highest_sale
        8 + // daily_lowest_sale
        32 + // top_collection
        8 + // top_collection_volume
        8 + // last_updated
        64 + // reserved
        1; // bump
}
