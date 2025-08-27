// finova-net/finova/client/typescript/src/accounts/nft.ts

import {
  Connection,
  PublicKey,
  AccountInfo,
  GetProgramAccountsFilter,
} from '@solana/web3.js';
import { BN } from '@coral-xyz/anchor';
import { TOKEN_PROGRAM_ID, ASSOCIATED_TOKEN_PROGRAM_ID } from '@solana/spl-token';

// NFT Account Types
export interface NftMetadata {
  name: string;
  symbol: string;
  uri: string;
  sellerFeeBasisPoints: number;
  creators: Array<{
    address: PublicKey;
    verified: boolean;
    share: number;
  }>;
  collection?: {
    verified: boolean;
    key: PublicKey;
  };
  uses?: {
    useMethod: number;
    remaining: BN;
    total: BN;
  };
}

export interface CollectionAccount {
  authority: PublicKey;
  name: string;
  symbol: string;
  uri: string;
  totalSupply: BN;
  maxSupply: BN;
  verified: boolean;
  creators: Array<{
    address: PublicKey;
    verified: boolean;
    share: number;
  }>;
  createdAt: BN;
  updatedAt: BN;
}

export interface SpecialCardAccount {
  owner: PublicKey;
  cardType: SpecialCardType;
  rarity: CardRarity;
  effect: CardEffect;
  usesRemaining: BN;
  totalUses: BN;
  expiresAt?: BN;
  mintedAt: BN;
  lastUsedAt?: BN;
  transferable: boolean;
  metadata: NftMetadata;
}

export interface MarketplaceListingAccount {
  seller: PublicKey;
  mint: PublicKey;
  price: BN;
  currency: PublicKey; // $FIN or $USDfin
  listedAt: BN;
  active: boolean;
  royaltyBps: number;
  escrowAccount: PublicKey;
}

// Enums
export enum SpecialCardType {
  MiningBoost = 'MiningBoost',
  XpAccelerator = 'XpAccelerator',
  ReferralPower = 'ReferralPower',
  ProfileBadge = 'ProfileBadge',
  Achievement = 'Achievement',
}

export enum CardRarity {
  Common = 'Common',
  Uncommon = 'Uncommon',
  Rare = 'Rare',
  Epic = 'Epic',
  Legendary = 'Legendary',
}

export interface CardEffect {
  type: 'mining' | 'xp' | 'referral' | 'passive';
  multiplier: number;
  duration?: number; // seconds, undefined for permanent
  targetActivity?: string[];
}

// NFT Account Manager Class
export class NftAccountManager {
  constructor(
    private connection: Connection,
    private programId: PublicKey,
    private tokenProgramId: PublicKey = TOKEN_PROGRAM_ID
  ) {}

  // Collection Account Methods
  async getCollectionAccount(collectionMint: PublicKey): Promise<CollectionAccount | null> {
    try {
      const [collectionPda] = PublicKey.findProgramAddressSync(
        [Buffer.from('collection'), collectionMint.toBuffer()],
        this.programId
      );

      const account = await this.connection.getAccountInfo(collectionPda);
      if (!account) return null;

      return this.deserializeCollectionAccount(account.data);
    } catch (error) {
      console.error('Error fetching collection account:', error);
      return null;
    }
  }

  async getAllCollections(): Promise<CollectionAccount[]> {
    try {
      const filters: GetProgramAccountsFilter[] = [
        { dataSize: 512 }, // Approximate collection account size
        {
          memcmp: {
            offset: 0,
            bytes: Buffer.from('collection').toString('base64'),
          },
        },
      ];

      const accounts = await this.connection.getProgramAccounts(
        this.programId,
        { filters }
      );

      return accounts
        .map(({ account }) => this.deserializeCollectionAccount(account.data))
        .filter(Boolean) as CollectionAccount[];
    } catch (error) {
      console.error('Error fetching all collections:', error);
      return [];
    }
  }

  // Special Card Account Methods
  async getSpecialCardAccount(mint: PublicKey): Promise<SpecialCardAccount | null> {
    try {
      const [cardPda] = PublicKey.findProgramAddressSync(
        [Buffer.from('special_card'), mint.toBuffer()],
        this.programId
      );

      const account = await this.connection.getAccountInfo(cardPda);
      if (!account) return null;

      return this.deserializeSpecialCardAccount(account.data);
    } catch (error) {
      console.error('Error fetching special card account:', error);
      return null;
    }
  }

  async getUserSpecialCards(owner: PublicKey): Promise<SpecialCardAccount[]> {
    try {
      const filters: GetProgramAccountsFilter[] = [
        { dataSize: 1024 }, // Approximate special card account size
        {
          memcmp: {
            offset: 8, // Skip discriminator
            bytes: owner.toString(),
          },
        },
      ];

      const accounts = await this.connection.getProgramAccounts(
        this.programId,
        { filters }
      );

      return accounts
        .map(({ account }) => this.deserializeSpecialCardAccount(account.data))
        .filter(Boolean) as SpecialCardAccount[];
    } catch (error) {
      console.error('Error fetching user special cards:', error);
      return [];
    }
  }

  async getActiveSpecialCards(owner: PublicKey): Promise<SpecialCardAccount[]> {
    const cards = await this.getUserSpecialCards(owner);
    const now = Math.floor(Date.now() / 1000);

    return cards.filter(card => {
      // Check if card has uses remaining
      if (card.usesRemaining.lten(0)) return false;
      
      // Check if card is not expired
      if (card.expiresAt && card.expiresAt.lten(now)) return false;
      
      return true;
    });
  }

  // Marketplace Methods
  async getMarketplaceListing(mint: PublicKey): Promise<MarketplaceListingAccount | null> {
    try {
      const [listingPda] = PublicKey.findProgramAddressSync(
        [Buffer.from('listing'), mint.toBuffer()],
        this.programId
      );

      const account = await this.connection.getAccountInfo(listingPda);
      if (!account) return null;

      return this.deserializeMarketplaceListingAccount(account.data);
    } catch (error) {
      console.error('Error fetching marketplace listing:', error);
      return null;
    }
  }

  async getAllActiveListings(): Promise<MarketplaceListingAccount[]> {
    try {
      const filters: GetProgramAccountsFilter[] = [
        { dataSize: 256 }, // Approximate listing account size
        {
          memcmp: {
            offset: 0,
            bytes: Buffer.from('listing').toString('base64'),
          },
        },
      ];

      const accounts = await this.connection.getProgramAccounts(
        this.programId,
        { filters }
      );

      const listings = accounts
        .map(({ account }) => this.deserializeMarketplaceListingAccount(account.data))
        .filter(Boolean) as MarketplaceListingAccount[];

      return listings.filter(listing => listing.active);
    } catch (error) {
      console.error('Error fetching active listings:', error);
      return [];
    }
  }

  async getUserListings(seller: PublicKey): Promise<MarketplaceListingAccount[]> {
    try {
      const filters: GetProgramAccountsFilter[] = [
        { dataSize: 256 },
        {
          memcmp: {
            offset: 8, // Skip discriminator
            bytes: seller.toString(),
          },
        },
      ];

      const accounts = await this.connection.getProgramAccounts(
        this.programId,
        { filters }
      );

      return accounts
        .map(({ account }) => this.deserializeMarketplaceListingAccount(account.data))
        .filter(Boolean) as MarketplaceListingAccount[];
    } catch (error) {
      console.error('Error fetching user listings:', error);
      return [];
    }
  }

  // Token Account Methods
  async getUserNftTokenAccounts(owner: PublicKey): Promise<Array<{
    mint: PublicKey;
    tokenAccount: PublicKey;
    amount: BN;
  }>> {
    try {
      const tokenAccounts = await this.connection.getTokenAccountsByOwner(
        owner,
        { programId: this.tokenProgramId }
      );

      const nftAccounts = [];

      for (const { pubkey, account } of tokenAccounts.value) {
        const data = account.data;
        if (data.length < 165) continue; // SPL token account size

        // Parse token account data
        const mint = new PublicKey(data.slice(0, 32));
        const amount = new BN(data.slice(64, 72), 'le');

        // Check if it's an NFT (amount = 1 and decimals = 0)
        if (amount.eq(new BN(1))) {
          const mintInfo = await this.connection.getAccountInfo(mint);
          if (mintInfo && mintInfo.data.length >= 82) {
            const decimals = mintInfo.data[44];
            if (decimals === 0) {
              nftAccounts.push({
                mint,
                tokenAccount: pubkey,
                amount,
              });
            }
          }
        }
      }

      return nftAccounts;
    } catch (error) {
      console.error('Error fetching user NFT token accounts:', error);
      return [];
    }
  }

  async getAssociatedTokenAccount(owner: PublicKey, mint: PublicKey): Promise<PublicKey> {
    const [ata] = PublicKey.findProgramAddressSync(
      [owner.toBuffer(), this.tokenProgramId.toBuffer(), mint.toBuffer()],
      ASSOCIATED_TOKEN_PROGRAM_ID
    );
    return ata;
  }

  // Utility Methods for Card Effects
  calculateCardEffects(cards: SpecialCardAccount[]): {
    miningMultiplier: number;
    xpMultiplier: number;
    referralMultiplier: number;
    synergyBonus: number;
  } {
    let miningMultiplier = 1.0;
    let xpMultiplier = 1.0;
    let referralMultiplier = 1.0;
    
    const activeCards = cards.filter(card => this.isCardActive(card));
    const cardTypes = new Set<SpecialCardType>();
    let rarityBonus = 0;

    for (const card of activeCards) {
      cardTypes.add(card.cardType);
      
      switch (card.effect.type) {
        case 'mining':
          miningMultiplier += (card.effect.multiplier - 1);
          break;
        case 'xp':
          xpMultiplier += (card.effect.multiplier - 1);
          break;
        case 'referral':
          referralMultiplier += (card.effect.multiplier - 1);
          break;
      }

      // Add rarity bonus
      switch (card.rarity) {
        case CardRarity.Uncommon: rarityBonus += 0.05; break;
        case CardRarity.Rare: rarityBonus += 0.1; break;
        case CardRarity.Epic: rarityBonus += 0.2; break;
        case CardRarity.Legendary: rarityBonus += 0.35; break;
      }
    }

    // Calculate synergy bonus
    let synergyBonus = 1.0 + (activeCards.length * 0.1) + rarityBonus;
    
    // Same category bonus
    if (cardTypes.size === 1 && activeCards.length > 1) {
      synergyBonus += 0.15;
    }
    
    // All categories active bonus
    if (cardTypes.size >= 3) {
      synergyBonus += 0.3;
    }

    return {
      miningMultiplier,
      xpMultiplier,
      referralMultiplier,
      synergyBonus,
    };
  }

  private isCardActive(card: SpecialCardAccount): boolean {
    const now = Math.floor(Date.now() / 1000);
    
    // Check uses remaining
    if (card.usesRemaining.lten(0)) return false;
    
    // Check expiration
    if (card.expiresAt && card.expiresAt.lten(now)) return false;
    
    return true;
  }

  // Deserialization methods
  private deserializeCollectionAccount(data: Buffer): CollectionAccount | null {
    try {
      // Simplified deserialization - in production, use proper Borsh deserialization
      let offset = 8; // Skip discriminator
      
      const authority = new PublicKey(data.slice(offset, offset + 32));
      offset += 32;
      
      const nameLen = data.readUInt32LE(offset);
      offset += 4;
      const name = data.slice(offset, offset + nameLen).toString('utf8');
      offset += nameLen;
      
      const symbolLen = data.readUInt32LE(offset);
      offset += 4;
      const symbol = data.slice(offset, offset + symbolLen).toString('utf8');
      offset += symbolLen;
      
      const uriLen = data.readUInt32LE(offset);
      offset += 4;
      const uri = data.slice(offset, offset + uriLen).toString('utf8');
      offset += uriLen;
      
      const totalSupply = new BN(data.slice(offset, offset + 8), 'le');
      offset += 8;
      const maxSupply = new BN(data.slice(offset, offset + 8), 'le');
      offset += 8;
      
      const verified = data[offset] === 1;
      offset += 1;
      
      const createdAt = new BN(data.slice(offset, offset + 8), 'le');
      offset += 8;
      const updatedAt = new BN(data.slice(offset, offset + 8), 'le');

      return {
        authority,
        name,
        symbol,
        uri,
        totalSupply,
        maxSupply,
        verified,
        creators: [], // Simplified - would parse creators array
        createdAt,
        updatedAt,
      };
    } catch (error) {
      console.error('Error deserializing collection account:', error);
      return null;
    }
  }

  private deserializeSpecialCardAccount(data: Buffer): SpecialCardAccount | null {
    try {
      let offset = 8; // Skip discriminator
      
      const owner = new PublicKey(data.slice(offset, offset + 32));
      offset += 32;
      
      const cardType = Object.values(SpecialCardType)[data[offset]];
      offset += 1;
      
      const rarity = Object.values(CardRarity)[data[offset]];
      offset += 1;
      
      const effectType = ['mining', 'xp', 'referral', 'passive'][data[offset]];
      offset += 1;
      
      const multiplier = data.readFloatLE(offset);
      offset += 4;
      
      const hasDuration = data[offset] === 1;
      offset += 1;
      
      let duration: number | undefined;
      if (hasDuration) {
        duration = data.readUInt32LE(offset);
        offset += 4;
      }
      
      const usesRemaining = new BN(data.slice(offset, offset + 8), 'le');
      offset += 8;
      const totalUses = new BN(data.slice(offset, offset + 8), 'le');
      offset += 8;
      
      const hasExpiry = data[offset] === 1;
      offset += 1;
      
      let expiresAt: BN | undefined;
      if (hasExpiry) {
        expiresAt = new BN(data.slice(offset, offset + 8), 'le');
        offset += 8;
      }
      
      const mintedAt = new BN(data.slice(offset, offset + 8), 'le');
      offset += 8;
      
      const hasLastUsed = data[offset] === 1;
      offset += 1;
      
      let lastUsedAt: BN | undefined;
      if (hasLastUsed) {
        lastUsedAt = new BN(data.slice(offset, offset + 8), 'le');
        offset += 8;
      }
      
      const transferable = data[offset] === 1;

      return {
        owner,
        cardType,
        rarity,
        effect: {
          type: effectType as any,
          multiplier,
          duration,
          targetActivity: [], // Simplified
        },
        usesRemaining,
        totalUses,
        expiresAt,
        mintedAt,
        lastUsedAt,
        transferable,
        metadata: {
          name: '',
          symbol: '',
          uri: '',
          sellerFeeBasisPoints: 0,
          creators: [],
        }, // Simplified
      };
    } catch (error) {
      console.error('Error deserializing special card account:', error);
      return null;
    }
  }

  private deserializeMarketplaceListingAccount(data: Buffer): MarketplaceListingAccount | null {
    try {
      let offset = 8; // Skip discriminator
      
      const seller = new PublicKey(data.slice(offset, offset + 32));
      offset += 32;
      
      const mint = new PublicKey(data.slice(offset, offset + 32));
      offset += 32;
      
      const price = new BN(data.slice(offset, offset + 8), 'le');
      offset += 8;
      
      const currency = new PublicKey(data.slice(offset, offset + 32));
      offset += 32;
      
      const listedAt = new BN(data.slice(offset, offset + 8), 'le');
      offset += 8;
      
      const active = data[offset] === 1;
      offset += 1;
      
      const royaltyBps = data.readUInt16LE(offset);
      offset += 2;
      
      const escrowAccount = new PublicKey(data.slice(offset, offset + 32));

      return {
        seller,
        mint,
        price,
        currency,
        listedAt,
        active,
        royaltyBps,
        escrowAccount,
      };
    } catch (error) {
      console.error('Error deserializing marketplace listing account:', error);
      return null;
    }
  }
}
