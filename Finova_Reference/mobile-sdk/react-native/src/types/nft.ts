/**
 * Finova Network Mobile SDK - NFT & Special Cards Types
 *
 * @version 2.0.0
 * @author Finova Network Team
 * @description Defines structures for NFTs, Special Cards, collections,
 * and the marketplace within the Finova ecosystem.
 */

import { BaseEntity } from './';
import { MultiplierType } from './mining';

// MARK: - Core NFT & Collection Types

export enum NFTRarity {
  COMMON = 'common',
  UNCOMMON = 'uncommon',
  RARE = 'rare',
  EPIC = 'epic',
  LEGENDARY = 'legendary',
  MYTHIC = 'mythic'
}

export enum NFTCategory {
  SPECIAL_CARD = 'special_card',
  PROFILE_BADGE = 'profile_badge',
  ACHIEVEMENT = 'achievement',
  COLLECTIBLE = 'collectible',
  UTILITY = 'utility'
}

export interface NFT extends BaseEntity {
  tokenId: string;
  mintAddress: string;
  collectionId: string;
  name: string;
  description: string;
  imageUrl: string;
  animationUrl?: string;
  attributes: NFTAttribute[];
  rarity: NFTRarity;
  rarityRank?: number;
  ownerAddress: string;
  isStaked: boolean;
}

export interface NFTAttribute {
  trait_type: string;
  value: string | number;
  display_type?: 'number' | 'date' | 'boost_percentage';
}

export interface NFTCollection extends BaseEntity {
  name: string;
  symbol: string;
  description: string;
  imageUrl: string;
  creatorAddress: string;
  totalSupply: number;
  floorPrice: number; // in FIN
  volume24h: number; // in FIN
  isVerified: boolean;
}

// MARK: - Special Cards & Utility

export interface SpecialCard extends NFT {
  category: 'mining' | 'xp' | 'referral';
  utility: CardUtility;
  synergy?: CardSynergy;
}

export interface CardUtility {
  effectType: MultiplierType;
  multiplier: number;
  duration: number; // in seconds, 0 for permanent
  uses: number | 'unlimited';
  remainingUses?: number;
  cooldown: number; // in seconds
  lastUsedAt?: Date;
  isActive: boolean;
}

export interface CardSynergy {
  requiredCardIds: string[];
  bonusEffect: {
    description: string;
    multiplier: number;
  };
}

// MARK: - Marketplace Types

export interface MarketplaceListing extends BaseEntity {
  nft: NFT;
  sellerAddress: string;
  price: number; // in FIN
  currency: 'FIN' | 'SOL' | 'USDC';
  status: 'active' | 'sold' | 'cancelled';
  listedAt: Date;
  expiresAt?: Date;
}

export interface MarketplaceSale extends BaseEntity {
  nft: NFT;
  sellerAddress: string;
  buyerAddress: string;
  price: number;
  currency: 'FIN' | 'SOL' | 'USDC';
  soldAt: Date;
  transactionSignature: string;
}

export interface MarketplaceStats {
  totalVolume: number;
  sales24h: number;
  topCollections: { collectionId: string; volume: number }[];
  trendingNFTs: { nftId: string; views: number }[];
}

export interface NFTPortfolio {
  totalValue: number; // in FIN
  nftCount: number;
  collectionCount: number;
  activeBoosts: CardUtility[];
}
