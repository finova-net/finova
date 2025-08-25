// client/typescript/src/instructions/nft.ts

/**
 * Finova Network - NFT Instructions SDK
 * TypeScript client for interacting with finova-nft program
 * 
 * This module provides comprehensive functionality for:
 * - NFT collection and metadata management
 * - Special card system with CPI integration
 * - Marketplace operations (listing, buying, auctions)
 * - Cross-program invocations with finova-core
 * 
 * @version 1.0.0
 * @author Finova Network Team
 * @license MIT
 */

import {
  Connection,
  PublicKey,
  Transaction,
  TransactionInstruction,
  SystemProgram,
  SYSVAR_RENT_PUBKEY,
  SYSVAR_CLOCK_PUBKEY,
  AccountMeta,
  Keypair,
  SendOptions,
  ConfirmOptions,
} from '@solana/web3.js';
import {
  TOKEN_PROGRAM_ID,
  ASSOCIATED_TOKEN_PROGRAM_ID,
  getAssociatedTokenAddress,
  createAssociatedTokenAccountInstruction,
} from '@solana/spl-token';
import { BN } from '@coral-xyz/anchor';
import * as borsh from 'borsh';

// Program IDs (these would be set after deployment)
export const FINOVA_NFT_PROGRAM_ID = new PublicKey('FinovaNFTProgram11111111111111111111111111111');
export const FINOVA_CORE_PROGRAM_ID = new PublicKey('FinovaCoreProgram1111111111111111111111111111');
export const METAPLEX_PROGRAM_ID = new PublicKey('metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s');

// Constants
export const COLLECTION_SEED = 'collection';
export const NFT_METADATA_SEED = 'nft_metadata';
export const SPECIAL_CARD_SEED = 'special_card';
export const MARKETPLACE_SEED = 'marketplace';
export const LISTING_SEED = 'listing';
export const AUCTION_SEED = 'auction';
export const BID_SEED = 'bid';

// Enums
export enum SpecialCardType {
  MiningBoost = 0,
  XPAccelerator = 1,
  ReferralPower = 2,
  GuildBuff = 3,
  QualityEnhancer = 4,
}

export enum SpecialCardRarity {
  Common = 0,
  Uncommon = 1,
  Rare = 2,
  Epic = 3,
  Legendary = 4,
  Mythic = 5,
}

export enum ListingType {
  FixedPrice = 0,
  Auction = 1,
  DutchAuction = 2,
}

export enum AuctionStatus {
  Active = 0,
  Completed = 1,
  Cancelled = 2,
  Expired = 3,
}

// Type definitions
export interface CollectionData {
  authority: PublicKey;
  name: string;
  symbol: string;
  description: string;
  image: string;
  totalSupply: BN;
  maxSupply: BN;
  royalty: number; // basis points (100 = 1%)
  isSpecialCard: boolean;
  createdAt: BN;
  updatedAt: BN;
}

export interface NFTMetadata {
  collection: PublicKey;
  mint: PublicKey;
  owner: PublicKey;
  tokenId: BN;
  name: string;
  description: string;
  image: string;
  attributes: NFTAttribute[];
  isSpecialCard: boolean;
  specialCardData?: SpecialCardData;
  createdAt: BN;
  updatedAt: BN;
}

export interface NFTAttribute {
  traitType: string;
  value: string;
  displayType?: string;
  maxValue?: number;
}

export interface SpecialCardData {
  cardType: SpecialCardType;
  rarity: SpecialCardRarity;
  effect: CardEffect;
  price: BN;
  isUsed: boolean;
  usedAt?: BN;
}

export interface CardEffect {
  multiplier: number; // 100 = 1.0x, 200 = 2.0x
  duration: BN; // duration in seconds
  effectType: number; // 0=mining, 1=xp, 2=referral, etc.
  additionalData?: Uint8Array;
}

export interface MarketplaceListing {
  id: BN;
  seller: PublicKey;
  nftMint: PublicKey;
  listingType: ListingType;
  price: BN;
  startTime: BN;
  endTime?: BN;
  isActive: boolean;
  royaltyFee: number;
  platformFee: number;
  createdAt: BN;
  updatedAt: BN;
}

export interface AuctionData {
  listing: PublicKey;
  highestBidder?: PublicKey;
  highestBid: BN;
  reservePrice: BN;
  bidIncrement: BN;
  totalBids: number;
  status: AuctionStatus;
  extendedTime: BN;
}

export interface BidData {
  auction: PublicKey;
  bidder: PublicKey;
  amount: BN;
  timestamp: BN;
  isRefunded: boolean;
}

// Schema definitions for Borsh serialization
const CollectionSchema = new Map([
  [CollectionData, {
    kind: 'struct',
    fields: [
      ['authority', [32]],
      ['name', 'string'],
      ['symbol', 'string'], 
      ['description', 'string'],
      ['image', 'string'],
      ['totalSupply', 'u64'],
      ['maxSupply', 'u64'],
      ['royalty', 'u16'],
      ['isSpecialCard', 'u8'],
      ['createdAt', 'i64'],
      ['updatedAt', 'i64'],
    ]
  }]
]);

// Helper functions
export function findCollectionPDA(
  name: string,
  authority: PublicKey,
  programId: PublicKey = FINOVA_NFT_PROGRAM_ID
): [PublicKey, number] {
  return PublicKey.findProgramAddressSync(
    [
      Buffer.from(COLLECTION_SEED),
      Buffer.from(name),
      authority.toBuffer(),
    ],
    programId
  );
}

export function findNFTMetadataPDA(
  mint: PublicKey,
  programId: PublicKey = FINOVA_NFT_PROGRAM_ID
): [PublicKey, number] {
  return PublicKey.findProgramAddressSync(
    [
      Buffer.from(NFT_METADATA_SEED),
      mint.toBuffer(),
    ],
    programId
  );
}

export function findSpecialCardPDA(
  mint: PublicKey,
  programId: PublicKey = FINOVA_NFT_PROGRAM_ID
): [PublicKey, number] {
  return PublicKey.findProgramAddressSync(
    [
      Buffer.from(SPECIAL_CARD_SEED),
      mint.toBuffer(),
    ],
    programId
  );
}

export function findMarketplacePDA(
  programId: PublicKey = FINOVA_NFT_PROGRAM_ID
): [PublicKey, number] {
  return PublicKey.findProgramAddressSync(
    [Buffer.from(MARKETPLACE_SEED)],
    programId
  );
}

export function findListingPDA(
  seller: PublicKey,
  mint: PublicKey,
  programId: PublicKey = FINOVA_NFT_PROGRAM_ID
): [PublicKey, number] {
  return PublicKey.findProgramAddressSync(
    [
      Buffer.from(LISTING_SEED),
      seller.toBuffer(),
      mint.toBuffer(),
    ],
    programId
  );
}

export function findAuctionPDA(
  listing: PublicKey,
  programId: PublicKey = FINOVA_NFT_PROGRAM_ID
): [PublicKey, number] {
  return PublicKey.findProgramAddressSync(
    [
      Buffer.from(AUCTION_SEED),
      listing.toBuffer(),
    ],
    programId
  );
}

export function findBidPDA(
  auction: PublicKey,
  bidder: PublicKey,
  programId: PublicKey = FINOVA_NFT_PROGRAM_ID
): [PublicKey, number] {
  return PublicKey.findProgramAddressSync(
    [
      Buffer.from(BID_SEED),
      auction.toBuffer(),
      bidder.toBuffer(),
    ],
  programId
  );
}

/**
 * Main NFT Instructions class
 * Provides methods for all NFT-related operations
 */
export class FinovaNFTInstructions {
  constructor(
    private connection: Connection,
    private programId: PublicKey = FINOVA_NFT_PROGRAM_ID
  ) {}

  /**
   * Create a new NFT collection
   */
  async createCollection(
    authority: PublicKey,
    name: string,
    symbol: string,
    description: string,
    image: string,
    maxSupply: number,
    royalty: number,
    isSpecialCard: boolean = false
  ): Promise<TransactionInstruction> {
    const [collectionPDA] = findCollectionPDA(name, authority, this.programId);
    
    // Create collection mint
    const collectionMint = Keypair.generate();

    const data = Buffer.alloc(1024);
    let offset = 0;

    // Instruction discriminator (8 bytes)
    data.writeUInt8(0, offset); // create_collection instruction
    offset += 1;

    // Serialize parameters
    const nameBuffer = Buffer.from(name, 'utf8');
    data.writeUInt32LE(nameBuffer.length, offset);
    offset += 4;
    nameBuffer.copy(data, offset);
    offset += nameBuffer.length;

    const symbolBuffer = Buffer.from(symbol, 'utf8');
    data.writeUInt32LE(symbolBuffer.length, offset);
    offset += 4;
    symbolBuffer.copy(data, offset);
    offset += symbolBuffer.length;

    const descBuffer = Buffer.from(description, 'utf8');
    data.writeUInt32LE(descBuffer.length, offset);
    offset += 4;
    descBuffer.copy(data, offset);
    offset += descBuffer.length;

    const imageBuffer = Buffer.from(image, 'utf8');
    data.writeUInt32LE(imageBuffer.length, offset);
    offset += 4;
    imageBuffer.copy(data, offset);
    offset += imageBuffer.length;

    // Max supply (8 bytes)
    const maxSupplyBN = new BN(maxSupply);
    maxSupplyBN.toArrayLike(Buffer, 'le', 8).copy(data, offset);
    offset += 8;

    // Royalty (2 bytes)
    data.writeUInt16LE(royalty, offset);
    offset += 2;

    // Is special card (1 byte)
    data.writeUInt8(isSpecialCard ? 1 : 0, offset);
    offset += 1;

    const finalData = data.slice(0, offset);

    const accounts: AccountMeta[] = [
      { pubkey: authority, isSigner: true, isWritable: true },
      { pubkey: collectionPDA, isSigner: false, isWritable: true },
      { pubkey: collectionMint.publicKey, isSigner: true, isWritable: true },
      { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
      { pubkey: TOKEN_PROGRAM_ID, isSigner: false, isWritable: false },
      { pubkey: SYSVAR_RENT_PUBKEY, isSigner: false, isWritable: false },
    ];

    return new TransactionInstruction({
      keys: accounts,
      programId: this.programId,
      data: finalData,
    });
  }

  /**
   * Mint a new NFT to a collection
   */
  async mintNFT(
    authority: PublicKey,
    collection: PublicKey,
    recipient: PublicKey,
    name: string,
    description: string,
    image: string,
    attributes: NFTAttribute[],
    isSpecialCard: boolean = false,
    specialCardData?: SpecialCardData
  ): Promise<TransactionInstruction> {
    const nftMint = Keypair.generate();
    const [nftMetadataPDA] = findNFTMetadataPDA(nftMint.publicKey, this.programId);
    const recipientTokenAccount = await getAssociatedTokenAddress(
      nftMint.publicKey,
      recipient
    );

    const data = Buffer.alloc(2048);
    let offset = 0;

    // Instruction discriminator
    data.writeUInt8(1, offset); // mint_nft instruction
    offset += 1;

    // Serialize NFT data
    const nameBuffer = Buffer.from(name, 'utf8');
    data.writeUInt32LE(nameBuffer.length, offset);
    offset += 4;
    nameBuffer.copy(data, offset);
    offset += nameBuffer.length;

    const descBuffer = Buffer.from(description, 'utf8');
    data.writeUInt32LE(descBuffer.length, offset);
    offset += 4;
    descBuffer.copy(data, offset);
    offset += descBuffer.length;

    const imageBuffer = Buffer.from(image, 'utf8');
    data.writeUInt32LE(imageBuffer.length, offset);
    offset += 4;
    imageBuffer.copy(data, offset);
    offset += imageBuffer.length;

    // Attributes
    data.writeUInt32LE(attributes.length, offset);
    offset += 4;
    
    for (const attr of attributes) {
      const traitBuffer = Buffer.from(attr.traitType, 'utf8');
      data.writeUInt32LE(traitBuffer.length, offset);
      offset += 4;
      traitBuffer.copy(data, offset);
      offset += traitBuffer.length;

      const valueBuffer = Buffer.from(attr.value, 'utf8');
      data.writeUInt32LE(valueBuffer.length, offset);
      offset += 4;
      valueBuffer.copy(data, offset);
      offset += valueBuffer.length;
    }

    // Is special card
    data.writeUInt8(isSpecialCard ? 1 : 0, offset);
    offset += 1;

    // Special card data if applicable
    if (isSpecialCard && specialCardData) {
      data.writeUInt8(specialCardData.cardType, offset);
      offset += 1;
      data.writeUInt8(specialCardData.rarity, offset);
      offset += 1;
      data.writeUInt32LE(specialCardData.effect.multiplier, offset);
      offset += 4;
      specialCardData.effect.duration.toArrayLike(Buffer, 'le', 8).copy(data, offset);
      offset += 8;
      data.writeUInt8(specialCardData.effect.effectType, offset);
      offset += 1;
      specialCardData.price.toArrayLike(Buffer, 'le', 8).copy(data, offset);
      offset += 8;
    }

    const finalData = data.slice(0, offset);

    const accounts: AccountMeta[] = [
      { pubkey: authority, isSigner: true, isWritable: true },
      { pubkey: collection, isSigner: false, isWritable: true },
      { pubkey: nftMint.publicKey, isSigner: true, isWritable: true },
      { pubkey: nftMetadataPDA, isSigner: false, isWritable: true },
      { pubkey: recipient, isSigner: false, isWritable: false },
      { pubkey: recipientTokenAccount, isSigner: false, isWritable: true },
      { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
      { pubkey: TOKEN_PROGRAM_ID, isSigner: false, isWritable: false },
      { pubkey: ASSOCIATED_TOKEN_PROGRAM_ID, isSigner: false, isWritable: false },
      { pubkey: SYSVAR_RENT_PUBKEY, isSigner: false, isWritable: false },
    ];

    // Add special card PDA if needed
    if (isSpecialCard) {
      const [specialCardPDA] = findSpecialCardPDA(nftMint.publicKey, this.programId);
      accounts.push({ pubkey: specialCardPDA, isSigner: false, isWritable: true });
    }

    return new TransactionInstruction({
      keys: accounts,
      programId: this.programId,
      data: finalData,
    });
  }

  /**
   * Use a special card (triggers CPI to finova-core)
   * This is the key integration point with the core program
   */
  async useSpecialCard(
    user: PublicKey,
    cardMint: PublicKey,
    userTokenAccount: PublicKey
  ): Promise<TransactionInstruction> {
    const [specialCardPDA] = findSpecialCardPDA(cardMint, this.programId);
    const [nftMetadataPDA] = findNFTMetadataPDA(cardMint, this.programId);

    // Core program accounts for CPI
    const [userStatePDA] = PublicKey.findProgramAddressSync(
      [Buffer.from('user'), user.toBuffer()],
      FINOVA_CORE_PROGRAM_ID
    );
    const [activeEffectsPDA] = PublicKey.findProgramAddressSync(
      [Buffer.from('active_effects'), user.toBuffer()],
      FINOVA_CORE_PROGRAM_ID
    );

    const data = Buffer.alloc(32);
    data.writeUInt8(2, 0); // use_special_card instruction

    const accounts: AccountMeta[] = [
      { pubkey: user, isSigner: true, isWritable: true },
      { pubkey: cardMint, isSigner: false, isWritable: true },
      { pubkey: userTokenAccount, isSigner: false, isWritable: true },
      { pubkey: specialCardPDA, isSigner: false, isWritable: true },
      { pubkey: nftMetadataPDA, isSigner: false, isWritable: true },
      
      // Core program accounts for CPI
      { pubkey: FINOVA_CORE_PROGRAM_ID, isSigner: false, isWritable: false },
      { pubkey: userStatePDA, isSigner: false, isWritable: true },
      { pubkey: activeEffectsPDA, isSigner: false, isWritable: true },
      
      { pubkey: TOKEN_PROGRAM_ID, isSigner: false, isWritable: false },
      { pubkey: SYSVAR_CLOCK_PUBKEY, isSigner: false, isWritable: false },
    ];

    return new TransactionInstruction({
      keys: accounts,
      programId: this.programId,
      data: data.slice(0, 1),
    });
  }

  /**
   * Transfer NFT between users
   */
  async transferNFT(
    from: PublicKey,
    to: PublicKey,
    mint: PublicKey
  ): Promise<TransactionInstruction> {
    const fromTokenAccount = await getAssociatedTokenAddress(mint, from);
    const toTokenAccount = await getAssociatedTokenAddress(mint, to);
    const [nftMetadataPDA] = findNFTMetadataPDA(mint, this.programId);

    const data = Buffer.alloc(32);
    data.writeUInt8(3, 0); // transfer_nft instruction

    const accounts: AccountMeta[] = [
      { pubkey: from, isSigner: true, isWritable: true },
      { pubkey: to, isSigner: false, isWritable: false },
      { pubkey: mint, isSigner: false, isWritable: false },
      { pubkey: fromTokenAccount, isSigner: false, isWritable: true },
      { pubkey: toTokenAccount, isSigner: false, isWritable: true },
      { pubkey: nftMetadataPDA, isSigner: false, isWritable: true },
      { pubkey: TOKEN_PROGRAM_ID, isSigner: false, isWritable: false },
      { pubkey: ASSOCIATED_TOKEN_PROGRAM_ID, isSigner: false, isWritable: false },
      { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
    ];

    return new TransactionInstruction({
      keys: accounts,
      programId: this.programId,
      data: data.slice(0, 1),
    });
  }

  /**
   * Burn an NFT (permanently destroy)
   */
  async burnNFT(
    owner: PublicKey,
    mint: PublicKey
  ): Promise<TransactionInstruction> {
    const ownerTokenAccount = await getAssociatedTokenAddress(mint, owner);
    const [nftMetadataPDA] = findNFTMetadataPDA(mint, this.programId);
    const [specialCardPDA] = findSpecialCardPDA(mint, this.programId);

    const data = Buffer.alloc(32);
    data.writeUInt8(4, 0); // burn_nft instruction

    const accounts: AccountMeta[] = [
      { pubkey: owner, isSigner: true, isWritable: true },
      { pubkey: mint, isSigner: false, isWritable: true },
      { pubkey: ownerTokenAccount, isSigner: false, isWritable: true },
      { pubkey: nftMetadataPDA, isSigner: false, isWritable: true },
      { pubkey: specialCardPDA, isSigner: false, isWritable: true },
      { pubkey: TOKEN_PROGRAM_ID, isSigner: false, isWritable: false },
    ];

    return new TransactionInstruction({
      keys: accounts,
      programId: this.programId,
      data: data.slice(0, 1),
    });
  }

  /**
   * Initialize the global marketplace
   */
  async initializeMarketplace(
    authority: PublicKey,
    platformFee: number, // basis points
    maxRoyalty: number // basis points
  ): Promise<TransactionInstruction> {
    const [marketplacePDA] = findMarketplacePDA(this.programId);

    const data = Buffer.alloc(32);
    let offset = 0;
    data.writeUInt8(5, offset); // initialize_marketplace instruction
    offset += 1;
    data.writeUInt16LE(platformFee, offset);
    offset += 2;
    data.writeUInt16LE(maxRoyalty, offset);
    offset += 2;

    const accounts: AccountMeta[] = [
      { pubkey: authority, isSigner: true, isWritable: true },
      { pubkey: marketplacePDA, isSigner: false, isWritable: true },
      { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
      { pubkey: SYSVAR_RENT_PUBKEY, isSigner: false, isWritable: false },
    ];

    return new TransactionInstruction({
      keys: accounts,
      programId: this.programId,
      data: data.slice(0, offset),
    });
  }

  /**
   * List an NFT for sale on the marketplace
   */
  async listNFT(
    seller: PublicKey,
    mint: PublicKey,
    price: BN,
    listingType: ListingType,
    duration?: number // seconds
  ): Promise<TransactionInstruction> {
    const [marketplacePDA] = findMarketplacePDA(this.programId);
    const [listingPDA] = findListingPDA(seller, mint, this.programId);
    const [nftMetadataPDA] = findNFTMetadataPDA(mint, this.programId);
    const sellerTokenAccount = await getAssociatedTokenAddress(mint, seller);

    const data = Buffer.alloc(64);
    let offset = 0;
    data.writeUInt8(6, offset); // list_nft instruction
    offset += 1;
    price.toArrayLike(Buffer, 'le', 8).copy(data, offset);
    offset += 8;
    data.writeUInt8(listingType, offset);
    offset += 1;
    
    if (duration) {
      data.writeUInt8(1, offset); // has duration
      offset += 1;
      data.writeUInt32LE(duration, offset);
      offset += 4;
    } else {
      data.writeUInt8(0, offset); // no duration
      offset += 1;
    }

    const accounts: AccountMeta[] = [
      { pubkey: seller, isSigner: true, isWritable: true },
      { pubkey: mint, isSigner: false, isWritable: false },
      { pubkey: sellerTokenAccount, isSigner: false, isWritable: true },
      { pubkey: marketplacePDA, isSigner: false, isWritable: false },
      { pubkey: listingPDA, isSigner: false, isWritable: true },
      { pubkey: nftMetadataPDA, isSigner: false, isWritable: false },
      { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
      { pubkey: SYSVAR_RENT_PUBKEY, isSigner: false, isWritable: false },
      { pubkey: SYSVAR_CLOCK_PUBKEY, isSigner: false, isWritable: false },
    ];

    return new TransactionInstruction({
      keys: accounts,
      programId: this.programId,
      data: data.slice(0, offset),
    });
  }

  /**
   * Buy an NFT from a fixed-price listing
   */
  async buyNFT(
    buyer: PublicKey,
    seller: PublicKey,
    mint: PublicKey,
    price: BN
  ): Promise<TransactionInstruction> {
    const [marketplacePDA] = findMarketplacePDA(this.programId);
    const [listingPDA] = findListingPDA(seller, mint, this.programId);
    const [nftMetadataPDA] = findNFTMetadataPDA(mint, this.programId);
    const sellerTokenAccount = await getAssociatedTokenAddress(mint, seller);
    const buyerTokenAccount = await getAssociatedTokenAddress(mint, buyer);

    const data = Buffer.alloc(32);
    let offset = 0;
    data.writeUInt8(7, offset); // buy_nft instruction
    offset += 1;
    price.toArrayLike(Buffer, 'le', 8).copy(data, offset);
    offset += 8;

    const accounts: AccountMeta[] = [
      { pubkey: buyer, isSigner: true, isWritable: true },
      { pubkey: seller, isSigner: false, isWritable: true },
      { pubkey: mint, isSigner: false, isWritable: false },
      { pubkey: sellerTokenAccount, isSigner: false, isWritable: true },
      { pubkey: buyerTokenAccount, isSigner: false, isWritable: true },
      { pubkey: marketplacePDA, isSigner: false, isWritable: true },
      { pubkey: listingPDA, isSigner: false, isWritable: true },
      { pubkey: nftMetadataPDA, isSigner: false, isWritable: false },
      { pubkey: TOKEN_PROGRAM_ID, isSigner: false, isWritable: false },
      { pubkey: ASSOCIATED_TOKEN_PROGRAM_ID, isSigner: false, isWritable: false },
      { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
      { pubkey: SYSVAR_CLOCK_PUBKEY, isSigner: false, isWritable: false },
    ];

    return new TransactionInstruction({
      keys: accounts,
      programId: this.programId,
      data: data.slice(0, offset),
    });
  }

  /**
   * Cancel an active listing
   */
  async cancelListing(
    seller: PublicKey,
    mint: PublicKey
  ): Promise<TransactionInstruction> {
    const [listingPDA] = findListingPDA(seller, mint, this.programId);
    const sellerTokenAccount = await getAssociatedTokenAddress(mint, seller);

    const data = Buffer.alloc(32);
    data.writeUInt8(8, 0); // cancel_listing instruction

    const accounts: AccountMeta[] = [
      { pubkey: seller, isSigner: true, isWritable: true },
      { pubkey: mint, isSigner: false, isWritable: false },
      { pubkey: sellerTokenAccount, isSigner: false, isWritable: true },
      { pubkey: listingPDA, isSigner: false, isWritable: true },
      { pubkey: SYSVAR_CLOCK_PUBKEY, isSigner: false, isWritable: false },
    ];

    return new TransactionInstruction({
      keys: accounts,
      programId: this.programId,
      data: data.slice(0, 1),
    });
  }

  /**
   * Place a bid on an auction
   */
  async placeBid(
    bidder: PublicKey,
    seller: PublicKey,
    mint: PublicKey,
    bidAmount: BN
  ): Promise<TransactionInstruction> {
    const [listingPDA] = findListingPDA(seller, mint, this.programId);
    const [auctionPDA] = findAuctionPDA(listingPDA, this.programId);
    const [bidPDA] = findBidPDA(auctionPDA, bidder, this.programId);

    const data = Buffer.alloc(32);
    let offset = 0;
    data.writeUInt8(9, offset); // place_bid instruction
    offset += 1;
    bidAmount.toArrayLike(Buffer, 'le', 8).copy(data, offset);
    offset += 8;

    const accounts: AccountMeta[] = [
      { pubkey: bidder, isSigner: true, isWritable: true },
      { pubkey: seller, isSigner: false, isWritable: false },
      { pubkey: mint, isSigner: false, isWritable: false },
      { pubkey: listingPDA, isSigner: false, isWritable: false },
      { pubkey: auctionPDA, isSigner: false, isWritable: true },
      { pubkey: bidPDA, isSigner: false, isWritable: true },
      { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
      { pubkey: SYSVAR_RENT_PUBKEY, isSigner: false, isWritable: false },
      { pubkey: SYSVAR_CLOCK_PUBKEY, isSigner: false, isWritable: false },
    ];

    return new TransactionInstruction({
      keys: accounts,
      programId: this.programId,
      data: data.slice(0, offset),
    });
  }

  /**
   * Finalize an auction (claim winning bid)
   */
  async finalizeAuction(
    seller: PublicKey,
    mint: PublicKey,
    winner?: PublicKey
  ): Promise<TransactionInstruction> {
    const [listingPDA] = findListingPDA(seller, mint, this.programId);
    const [auctionPDA] = findAuctionPDA(listingPDA, this.programId);
    const [nftMetadataPDA] = findNFTMetadataPDA(mint, this.programId);
    const sellerTokenAccount = await getAssociatedTokenAddress(mint, seller);

    const accounts: AccountMeta[] = [
      { pubkey: seller, isSigner: true, isWritable: true },
      { pubkey: mint, isSigner: false, isWritable: false },
      { pubkey: sellerTokenAccount, isSigner: false, isWritable: true },
      { pubkey: listingPDA, isSigner: false, isWritable: true },
      { pubkey: auctionPDA, isSigner: false, isWritable: true },
      { pubkey: nftMetadataPDA, isSigner: false, isWritable: true },
      { pubkey: SYSVAR_CLOCK_PUBKEY, isSigner: false, isWritable: false },
    ];

    // Add winner accounts if specified
    if (winner) {
      const winnerTokenAccount = await getAssociatedTokenAddress(mint, winner);
      const [winnerBidPDA] = findBidPDA(auctionPDA, winner, this.programId);
      
      accounts.push(
        { pubkey: winner, isSigner: false, isWritable: true },
        { pubkey: winnerTokenAccount, isSigner: false, isWritable: true },
        { pubkey: winnerBidPDA, isSigner: false, isWritable: true }
      );
    }

    accounts.push(
      { pubkey: TOKEN_PROGRAM_ID, isSigner: false, isWritable: false },
      { pubkey: ASSOCIATED_TOKEN_PROGRAM_ID, isSigner: false, isWritable: false }
    );

    const data = Buffer.alloc(32);
    data.writeUInt8(10, 0); // finalize_auction instruction

    return new TransactionInstruction({
      keys: accounts,
      programId: this.programId,
      data: data.slice(0, 1),
    });
  }

  /**
   * Update NFT metadata (only collection authority)
   */
  async updateMetadata(
    authority: PublicKey,
    mint: PublicKey,
    newName?: string,
    newDescription?: string,
    newImage?: string,
    newAttributes?: NFTAttribute[]
  ): Promise<TransactionInstruction> {
    const [nftMetadataPDA] = findNFTMetadataPDA(mint, this.programId);

    const data = Buffer.alloc(1024);
    let offset = 0;
    data.writeUInt8(11, offset); // update_metadata instruction
    offset += 1;

    // Update flags
    const hasNewName = newName !== undefined;
    const hasNewDescription = newDescription !== undefined;
    const hasNewImage = newImage !== undefined;
    const hasNewAttributes = newAttributes !== undefined;

    data.writeUInt8((hasNewName ? 1 : 0) | (hasNewDescription ? 2 : 0) | (hasNewImage ? 4 : 0) | (hasNewAttributes ? 8 : 0), offset);
    offset += 1;

    if (newName) {
      const nameBuffer = Buffer.from(newName, 'utf8');
      data.writeUInt32LE(nameBuffer.length, offset);
      offset += 4;
      nameBuffer.copy(data, offset);
      offset += nameBuffer.length;
    }

    if (newDescription) {
      const descBuffer = Buffer.from(newDescription, 'utf8');
      data.writeUInt32LE(descBuffer.length, offset);
      offset += 4;
      descBuffer.copy(data, offset);
      offset += descBuffer.length;
    }

    if (newImage) {
      const imageBuffer = Buffer.from(newImage, 'utf8');
      data.writeUInt32LE(imageBuffer.length, offset);
      offset += 4;
      imageBuffer.copy(data, offset);
      offset += imageBuffer.length;
    }

    if (newAttributes) {
      data.writeUInt32LE(newAttributes.length, offset);
      offset += 4;
      
      for (const attr of newAttributes) {
        const traitBuffer = Buffer.from(attr.traitType, 'utf8');
        data.writeUInt32LE(traitBuffer.length, offset);
        offset += 4;
        traitBuffer.copy(data, offset);
        offset += traitBuffer.length;

        const valueBuffer = Buffer.from(attr.value, 'utf8');
        data.writeUInt32LE(valueBuffer.length, offset);
        offset += 4;
        valueBuffer.copy(data, offset);
        offset += valueBuffer.length;
      }
    }

    const accounts: AccountMeta[] = [
      { pubkey: authority, isSigner: true, isWritable: false },
      { pubkey: mint, isSigner: false, isWritable: false },
      { pubkey: nftMetadataPDA, isSigner: false, isWritable: true },
      { pubkey: SYSVAR_CLOCK_PUBKEY, isSigner: false, isWritable: false },
    ];

    return new TransactionInstruction({
      keys: accounts,
      programId: this.programId,
      data: data.slice(0, offset),
    });
  }

  // Account fetching methods

  /**
   * Fetch collection data
   */
  async getCollection(collectionPDA: PublicKey): Promise<CollectionData | null> {
    try {
      const accountInfo = await this.connection.getAccountInfo(collectionPDA);
      if (!accountInfo) return null;

      // Skip discriminator (8 bytes) and deserialize
      const data = accountInfo.data.slice(8);
      return this.deserializeCollection(data);
    } catch (error) {
      console.error('Error fetching collection:', error);
      return null;
    }
  }

  /**
   * Fetch NFT metadata
   */
  async getNFTMetadata(mint: PublicKey): Promise<NFTMetadata | null> {
    try {
      const [nftMetadataPDA] = findNFTMetadataPDA(mint, this.programId);
      const accountInfo = await this.connection.getAccountInfo(nftMetadataPDA);
      if (!accountInfo) return null;

      const data = accountInfo.data.slice(8);
      return this.deserializeNFTMetadata(data);
    } catch (error) {
      console.error('Error fetching NFT metadata:', error);
      return null;
    }
  }

  /**
   * Fetch special card data
   */
  async getSpecialCard(mint: PublicKey): Promise<SpecialCardData | null> {
    try {
      const [specialCardPDA] = findSpecialCardPDA(mint, this.programId);
      const accountInfo = await this.connection.getAccountInfo(specialCardPDA);
      if (!accountInfo) return null;

      const data = accountInfo.data.slice(8);
      return this.deserializeSpecialCard(data);
    } catch (error) {
      console.error('Error fetching special card:', error);
      return null;
    }
  }

  /**
   * Fetch marketplace data
   */
  async getMarketplace(): Promise<any | null> {
    try {
      const [marketplacePDA] = findMarketplacePDA(this.programId);
      const accountInfo = await this.connection.getAccountInfo(marketplacePDA);
      if (!accountInfo) return null;

      const data = accountInfo.data.slice(8);
      return this.deserializeMarketplace(data);
    } catch (error) {
      console.error('Error fetching marketplace:', error);
      return null;
    }
  }

  /**
   * Fetch listing data
   */
  async getListing(seller: PublicKey, mint: PublicKey): Promise<MarketplaceListing | null> {
    try {
      const [listingPDA] = findListingPDA(seller, mint, this.programId);
      const accountInfo = await this.connection.getAccountInfo(listingPDA);
      if (!accountInfo) return null;

      const data = accountInfo.data.slice(8);
      return this.deserializeListing(data);
    } catch (error) {
      console.error('Error fetching listing:', error);
      return null;
    }
  }

  /**
   * Fetch auction data
   */
  async getAuction(listing: PublicKey): Promise<AuctionData | null> {
    try {
      const [auctionPDA] = findAuctionPDA(listing, this.programId);
      const accountInfo = await this.connection.getAccountInfo(auctionPDA);
      if (!accountInfo) return null;

      const data = accountInfo.data.slice(8);
      return this.deserializeAuction(data);
    } catch (error) {
      console.error('Error fetching auction:', error);
      return null;
    }
  }

  /**
   * Get all NFTs owned by a user
   */
  async getUserNFTs(owner: PublicKey): Promise<PublicKey[]> {
    try {
      const tokenAccounts = await this.connection.getParsedTokenAccountsByOwner(
        owner,
        { programId: TOKEN_PROGRAM_ID }
      );

      const nftMints: PublicKey[] = [];
      for (const account of tokenAccounts.value) {
        const tokenAmount = account.account.data.parsed.info.tokenAmount;
        if (tokenAmount.uiAmount === 1 && tokenAmount.decimals === 0) {
          // This is likely an NFT (amount = 1, decimals = 0)
          const mint = new PublicKey(account.account.data.parsed.info.mint);
          
          // Verify it's a Finova NFT by checking metadata PDA
          const [metadataPDA] = findNFTMetadataPDA(mint, this.programId);
          const metadataAccount = await this.connection.getAccountInfo(metadataPDA);
          
          if (metadataAccount) {
            nftMints.push(mint);
          }
        }
      }

      return nftMints;
    } catch (error) {
      console.error('Error fetching user NFTs:', error);
      return [];
    }
  }

  /**
   * Get all active listings in marketplace
   */
  async getActiveListings(limit: number = 50): Promise<MarketplaceListing[]> {
    try {
      const accounts = await this.connection.getProgramAccounts(this.programId, {
        filters: [
          { dataSize: 200 }, // Approximate size of listing account
          { memcmp: { offset: 8, bytes: Buffer.from([1]).toString('base64') } }, // Active flag
        ],
      });

      const listings: MarketplaceListing[] = [];
      for (const account of accounts.slice(0, limit)) {
        try {
          const listing = this.deserializeListing(account.account.data.slice(8));
          if (listing && listing.isActive) {
            listings.push(listing);
          }
        } catch (error) {
          console.warn('Failed to deserialize listing:', error);
        }
      }

      return listings.sort((a, b) => b.createdAt.sub(a.createdAt).toNumber());
    } catch (error) {
      console.error('Error fetching active listings:', error);
      return [];
    }
  }

  /**
   * Get collections by authority
   */
  async getCollectionsByAuthority(authority: PublicKey): Promise<PublicKey[]> {
    try {
      const accounts = await this.connection.getProgramAccounts(this.programId, {
        filters: [
          { dataSize: 300 }, // Approximate size of collection account
          { memcmp: { offset: 8, bytes: authority.toBase58() } }, // Authority filter
        ],
      });

      return accounts.map(account => account.pubkey);
    } catch (error) {
      console.error('Error fetching collections by authority:', error);
      return [];
    }
  }

  // Utility methods for building complete transactions

  /**
   * Build a complete mint NFT transaction with all required instructions
   */
  async buildMintNFTTransaction(
    authority: Keypair,
    collection: PublicKey,
    recipient: PublicKey,
    nftData: {
      name: string;
      description: string;
      image: string;
      attributes: NFTAttribute[];
      isSpecialCard?: boolean;
      specialCardData?: SpecialCardData;
    }
  ): Promise<{ transaction: Transaction; nftMint: Keypair }> {
    const nftMint = Keypair.generate();
    const recipientTokenAccount = await getAssociatedTokenAddress(
      nftMint.publicKey,
      recipient
    );

    const transaction = new Transaction();

    // Create associated token account if it doesn't exist
    const recipientAccountInfo = await this.connection.getAccountInfo(recipientTokenAccount);
    if (!recipientAccountInfo) {
      transaction.add(
        createAssociatedTokenAccountInstruction(
          authority.publicKey,
          recipientTokenAccount,
          recipient,
          nftMint.publicKey
        )
      );
    }

    // Add mint instruction
    transaction.add(
      await this.mintNFT(
        authority.publicKey,
        collection,
        recipient,
        nftData.name,
        nftData.description,
        nftData.image,
        nftData.attributes,
        nftData.isSpecialCard,
        nftData.specialCardData
      )
    );

    // Set recent blockhash and fee payer
    const { blockhash } = await this.connection.getLatestBlockhash();
    transaction.recentBlockhash = blockhash;
    transaction.feePayer = authority.publicKey;

    return { transaction, nftMint };
  }

  /**
   * Build complete marketplace listing transaction
   */
  async buildListNFTTransaction(
    seller: Keypair,
    mint: PublicKey,
    price: BN,
    listingType: ListingType,
    duration?: number
  ): Promise<Transaction> {
    const transaction = new Transaction();

    // Add list instruction
    transaction.add(
      await this.listNFT(
        seller.publicKey,
        mint,
        price,
        listingType,
        duration
      )
    );

    // Set recent blockhash and fee payer
    const { blockhash } = await this.connection.getLatestBlockhash();
    transaction.recentBlockhash = blockhash;
    transaction.feePayer = seller.publicKey;

    return transaction;
  }

  /**
   * Build complete NFT purchase transaction
   */
  async buildBuyNFTTransaction(
    buyer: Keypair,
    seller: PublicKey,
    mint: PublicKey,
    price: BN
  ): Promise<Transaction> {
    const transaction = new Transaction();
    
    const buyerTokenAccount = await getAssociatedTokenAddress(mint, buyer.publicKey);

    // Create buyer's associated token account if needed
    const buyerAccountInfo = await this.connection.getAccountInfo(buyerTokenAccount);
    if (!buyerAccountInfo) {
      transaction.add(
        createAssociatedTokenAccountInstruction(
          buyer.publicKey,
          buyerTokenAccount,
          buyer.publicKey,
          mint
        )
      );
    }

    // Add buy instruction
    transaction.add(
      await this.buyNFT(
        buyer.publicKey,
        seller,
        mint,
        price
      )
    );

    // Set recent blockhash and fee payer
    const { blockhash } = await this.connection.getLatestBlockhash();
    transaction.recentBlockhash = blockhash;
    transaction.feePayer = buyer.publicKey;

    return transaction;
  }

  // Deserialization helper methods
  private deserializeCollection(data: Buffer): CollectionData {
    let offset = 0;
    
    const authority = new PublicKey(data.slice(offset, offset + 32));
    offset += 32;
    
    const nameLength = data.readUInt32LE(offset);
    offset += 4;
    const name = data.slice(offset, offset + nameLength).toString('utf8');
    offset += nameLength;
    
    const symbolLength = data.readUInt32LE(offset);
    offset += 4;
    const symbol = data.slice(offset, offset + symbolLength).toString('utf8');
    offset += symbolLength;
    
    const descLength = data.readUInt32LE(offset);
    offset += 4;
    const description = data.slice(offset, offset + descLength).toString('utf8');
    offset += descLength;
    
    const imageLength = data.readUInt32LE(offset);
    offset += 4;
    const image = data.slice(offset, offset + imageLength).toString('utf8');
    offset += imageLength;
    
    const totalSupply = new BN(data.slice(offset, offset + 8), 'le');
    offset += 8;
    const maxSupply = new BN(data.slice(offset, offset + 8), 'le');
    offset += 8;
    
    const royalty = data.readUInt16LE(offset);
    offset += 2;
    const isSpecialCard = data.readUInt8(offset) === 1;
    offset += 1;
    
    const createdAt = new BN(data.slice(offset, offset + 8), 'le');
    offset += 8;
    const updatedAt = new BN(data.slice(offset, offset + 8), 'le');

    return {
      authority,
      name,
      symbol,
      description,
      image,
      totalSupply,
      maxSupply,
      royalty,
      isSpecialCard,
      createdAt,
      updatedAt,
    };
  }

  private deserializeNFTMetadata(data: Buffer): NFTMetadata {
    let offset = 0;
    
    const collection = new PublicKey(data.slice(offset, offset + 32));
    offset += 32;
    const mint = new PublicKey(data.slice(offset, offset + 32));
    offset += 32;
    const owner = new PublicKey(data.slice(offset, offset + 32));
    offset += 32;
    
    const tokenId = new BN(data.slice(offset, offset + 8), 'le');
    offset += 8;
    
    const nameLength = data.readUInt32LE(offset);
    offset += 4;
    const name = data.slice(offset, offset + nameLength).toString('utf8');
    offset += nameLength;
    
    const descLength = data.readUInt32LE(offset);
    offset += 4;
    const description = data.slice(offset, offset + descLength).toString('utf8');
    offset += descLength;
    
    const imageLength = data.readUInt32LE(offset);
    offset += 4;
    const image = data.slice(offset, offset + imageLength).toString('utf8');
    offset += imageLength;
    
    // Attributes
    const attributeCount = data.readUInt32LE(offset);
    offset += 4;
    const attributes: NFTAttribute[] = [];
    
    for (let i = 0; i < attributeCount; i++) {
      const traitLength = data.readUInt32LE(offset);
      offset += 4;
      const traitType = data.slice(offset, offset + traitLength).toString('utf8');
      offset += traitLength;
      
      const valueLength = data.readUInt32LE(offset);
      offset += 4;
      const value = data.slice(offset, offset + valueLength).toString('utf8');
      offset += valueLength;
      
      attributes.push({ traitType, value });
    }
    
    const isSpecialCard = data.readUInt8(offset) === 1;
    offset += 1;
    
    let specialCardData: SpecialCardData | undefined;
    if (isSpecialCard) {
      // Deserialize special card data if present
      specialCardData = this.deserializeSpecialCardData(data.slice(offset));
    }
    
    const createdAt = new BN(data.slice(-16, -8), 'le');
    const updatedAt = new BN(data.slice(-8), 'le');

    return {
      collection,
      mint,
      owner,
      tokenId,
      name,
      description,
      image,
      attributes,
      isSpecialCard,
      specialCardData,
      createdAt,
      updatedAt,
    };
  }

  private deserializeSpecialCard(data: Buffer): SpecialCardData {
    let offset = 0;
    
    const cardType = data.readUInt8(offset) as SpecialCardType;
    offset += 1;
    const rarity = data.readUInt8(offset) as SpecialCardRarity;
    offset += 1;
    
    const multiplier = data.readUInt32LE(offset);
    offset += 4;
    const duration = new BN(data.slice(offset, offset + 8), 'le');
    offset += 8;
    const effectType = data.readUInt8(offset);
    offset += 1;
    
    const price = new BN(data.slice(offset, offset + 8), 'le');
    offset += 8;
    const isUsed = data.readUInt8(offset) === 1;
    offset += 1;
    
    let usedAt: BN | undefined;
    if (isUsed) {
      usedAt = new BN(data.slice(offset, offset + 8), 'le');
    }

    return {
      cardType,
      rarity,
      effect: {
        multiplier,
        duration,
        effectType,
      },
      price,
      isUsed,
      usedAt,
    };
  }

  private deserializeSpecialCardData(data: Buffer): SpecialCardData {
    return this.deserializeSpecialCard(data);
  }

  private deserializeMarketplace(data: Buffer): any {
    let offset = 0;
    
    const authority = new PublicKey(data.slice(offset, offset + 32));
    offset += 32;
    const platformFee = data.readUInt16LE(offset);
    offset += 2;
    const maxRoyalty = data.readUInt16LE(offset);
    offset += 2;
    const totalVolume = new BN(data.slice(offset, offset + 8), 'le');
    offset += 8;
    const totalSales = new BN(data.slice(offset, offset + 8), 'le');

    return {
      authority,
      platformFee,
      maxRoyalty,
      totalVolume,
      totalSales,
    };
  }

  private deserializeListing(data: Buffer): MarketplaceListing {
    let offset = 0;
    
    const id = new BN(data.slice(offset, offset + 8), 'le');
    offset += 8;
    const seller = new PublicKey(data.slice(offset, offset + 32));
    offset += 32;
    const nftMint = new PublicKey(data.slice(offset, offset + 32));
    offset += 32;
    const listingType = data.readUInt8(offset) as ListingType;
    offset += 1;
    const price = new BN(data.slice(offset, offset + 8), 'le');
    offset += 8;
    const startTime = new BN(data.slice(offset, offset + 8), 'le');
    offset += 8;
    
    const hasEndTime = data.readUInt8(offset) === 1;
    offset += 1;
    let endTime: BN | undefined;
    if (hasEndTime) {
      endTime = new BN(data.slice(offset, offset + 8), 'le');
      offset += 8;
    }
    
    const isActive = data.readUInt8(offset) === 1;
    offset += 1;
    const royaltyFee = data.readUInt16LE(offset);
    offset += 2;
    const platformFee = data.readUInt16LE(offset);
    offset += 2;
    const createdAt = new BN(data.slice(offset, offset + 8), 'le');
    offset += 8;
    const updatedAt = new BN(data.slice(offset, offset + 8), 'le');

    return {
      id,
      seller,
      nftMint,
      listingType,
      price,
      startTime,
      endTime,
      isActive,
      royaltyFee,
      platformFee,
      createdAt,
      updatedAt,
    };
  }

  private deserializeAuction(data: Buffer): AuctionData {
    let offset = 0;
    
    const listing = new PublicKey(data.slice(offset, offset + 32));
    offset += 32;
    
    const hasHighestBidder = data.readUInt8(offset) === 1;
    offset += 1;
    let highestBidder: PublicKey | undefined;
    if (hasHighestBidder) {
      highestBidder = new PublicKey(data.slice(offset, offset + 32));
      offset += 32;
    }
    
    const highestBid = new BN(data.slice(offset, offset + 8), 'le');
    offset += 8;
    const reservePrice = new BN(data.slice(offset, offset + 8), 'le');
    offset += 8;
    const bidIncrement = new BN(data.slice(offset, offset + 8), 'le');
    offset += 8;
    const totalBids = data.readUInt32LE(offset);
    offset += 4;
    const status = data.readUInt8(offset) as AuctionStatus;
    offset += 1;
    const extendedTime = new BN(data.slice(offset, offset + 8), 'le');

    return {
      listing,
      highestBidder,
      highestBid,
      reservePrice,
      bidIncrement,
      totalBids,
      status,
      extendedTime,
    };
  }

  // Transaction execution helpers
  
  /**
   * Send and confirm a transaction with retry logic
   */
  async sendAndConfirmTransaction(
    transaction: Transaction,
    signers: Keypair[],
    options?: SendOptions & ConfirmOptions
  ): Promise<string> {
    const maxRetries = 3;
    let attempt = 0;

    while (attempt < maxRetries) {
      try {
        const signature = await this.connection.sendTransaction(
          transaction,
          signers,
          options
        );

        const confirmation = await this.connection.confirmTransaction(
          signature,
          options?.commitment || 'confirmed'
        );

        if (confirmation.value.err) {
          throw new Error(`Transaction failed: ${confirmation.value.err}`);
        }

        return signature;
      } catch (error) {
        attempt++;
        if (attempt >= maxRetries) {
          throw error;
        }
        
        // Wait before retry with exponential backoff
        await new Promise(resolve => setTimeout(resolve, 1000 * Math.pow(2, attempt)));
        
        // Update blockhash for retry
        const { blockhash } = await this.connection.getLatestBlockhash();
        transaction.recentBlockhash = blockhash;
      }
    }

    throw new Error('Transaction failed after maximum retries');
  }

  /**
   * Estimate transaction fees
   */
  async estimateTransactionFee(transaction: Transaction): Promise<number> {
    try {
      const { feeCalculator } = await this.connection.getRecentBlockhash();
      return feeCalculator.lamportsPerSignature * transaction.signatures.length;
    } catch (error) {
      console.error('Error estimating transaction fee:', error);
      return 5000; // Default fallback fee
    }
  }
}

// Export all types and functions for easy import
export {
  FinovaNFTInstructions as default,
  type CollectionData,
  type NFTMetadata,
  type NFTAttribute,
  type SpecialCardData,
  type CardEffect,
  type MarketplaceListing,
  type AuctionData,
  type BidData,
};
