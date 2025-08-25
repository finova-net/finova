import { Request, Response } from 'express';
import { Connection, PublicKey, Transaction, SystemProgram, LAMPORTS_PER_SOL } from '@solana/web3.js';
import { createCreateMetadataAccountV3Instruction, PROGRAM_ID as TOKEN_METADATA_PROGRAM_ID } from '@metaplex-foundation/mpl-token-metadata';
import { createMint, createAssociatedTokenAccount, mintTo, getAssociatedTokenAddress } from '@solana/spl-token';
import { NFT } from '../models/NFT.model';
import { User } from '../models/User.model';
import { blockchainService } from '../services/blockchain.service';
import { aiQualityService } from '../services/ai-quality.service';
import { xpService } from '../services/xp.service';
import { miningService } from '../services/mining.service';
import { logger } from '../utils/logger';
import { validateNFTMetadata, calculateNFTPrice, generateSpecialCardAttributes } from '../utils/validation';
import { redisClient } from '../config/redis';

export class NFTController {
  private connection: Connection;
  
  constructor() {
    this.connection = new Connection(process.env.SOLANA_RPC_URL || 'https://api.devnet.solana.com');
  }

  // Create NFT Collection
  public createCollection = async (req: Request, res: Response): Promise<void> => {
    try {
      const { name, description, image, royalty, maxSupply } = req.body;
      const userId = req.user.id;

      // Validate user permissions
      const user = await User.findById(userId).populate('xpLevel stakingTier');
      if (!user || user.xpLevel < 26) { // Gold tier required
        res.status(403).json({
          success: false,
          message: 'Gold tier (Level 26+) required to create collections'
        });
        return;
      }

      // Validate metadata
      const validationResult = validateNFTMetadata({ name, description, image });
      if (!validationResult.isValid) {
        res.status(400).json({
          success: false,
          message: 'Invalid metadata',
          errors: validationResult.errors
        });
        return;
      }

      // Create collection on Solana
      const collectionMint = await this.createCollectionMint(
        user.walletAddress,
        name,
        description,
        image,
        royalty
      );

      // Save collection to database
      const collection = new NFT({
        type: 'collection',
        name,
        description,
        image,
        mintAddress: collectionMint.toString(),
        creatorId: userId,
        royalty,
        maxSupply,
        currentSupply: 0,
        isActive: true,
        attributes: {
          collectionType: 'user_created',
          createdAt: new Date(),
          network: 'solana'
        }
      });

      await collection.save();

      // Award XP for collection creation
      await xpService.awardXP(userId, {
        activity: 'create_collection',
        baseXP: 1000,
        qualityMultiplier: 1.5,
        platform: 'finova'
      });

      logger.info(`Collection created: ${collectionMint.toString()} by user ${userId}`);

      res.status(201).json({
        success: true,
        data: {
          collection: {
            id: collection._id,
            mintAddress: collectionMint.toString(),
            name,
            description,
            image,
            royalty,
            maxSupply
          }
        },
        message: 'Collection created successfully'
      });

    } catch (error) {
      logger.error('Create collection error:', error);
      res.status(500).json({
        success: false,
        message: 'Internal server error'
      });
    }
  };

  // Mint Special Cards (Mining boost cards, XP cards, etc.)
  public mintSpecialCard = async (req: Request, res: Response): Promise<void> => {
    try {
      const { cardType, quantity = 1 } = req.body;
      const userId = req.user.id;

      const user = await User.findById(userId).populate('finBalance stakingTier');
      if (!user) {
        res.status(404).json({
          success: false,
          message: 'User not found'
        });
        return;
      }

      // Get card configuration
      const cardConfig = this.getSpecialCardConfig(cardType);
      if (!cardConfig) {
        res.status(400).json({
          success: false,
          message: 'Invalid card type'
        });
        return;
      }

      // Calculate total price with user discounts
      const totalPrice = this.calculateCardPrice(cardConfig, quantity, user);

      // Check user balance
      if (user.finBalance < totalPrice) {
        res.status(400).json({
          success: false,
          message: 'Insufficient $FIN balance',
          required: totalPrice,
          balance: user.finBalance
        });
        return;
      }

      // Mint cards
      const mintedCards = [];
      for (let i = 0; i < quantity; i++) {
        const cardMint = await this.mintCard(user.walletAddress, cardConfig);
        
        const card = new NFT({
          type: 'special_card',
          name: cardConfig.name,
          description: cardConfig.description,
          image: cardConfig.image,
          mintAddress: cardMint.toString(),
          ownerId: userId,
          attributes: {
            ...cardConfig.attributes,
            mintedAt: new Date(),
            serialNumber: await this.getNextSerialNumber(cardType)
          },
          rarity: cardConfig.rarity,
          isUsed: false,
          cardType: cardType
        });

        await card.save();
        mintedCards.push(card);
      }

      // Deduct payment
      user.finBalance -= totalPrice;
      await user.save();

      // Burn $FIN tokens
      await blockchainService.burnTokens(totalPrice);

      // Award XP for NFT purchase
      await xpService.awardXP(userId, {
        activity: 'nft_purchase',
        baseXP: 50 * quantity,
        qualityMultiplier: 1.0,
        platform: 'finova'
      });

      logger.info(`Special cards minted: ${quantity}x ${cardType} for user ${userId}`);

      res.status(201).json({
        success: true,
        data: {
          cards: mintedCards.map(card => ({
            id: card._id,
            mintAddress: card.mintAddress,
            name: card.name,
            rarity: card.rarity,
            attributes: card.attributes
          })),
          totalCost: totalPrice,
          newBalance: user.finBalance
        },
        message: `${quantity} special card(s) minted successfully`
      });

    } catch (error) {
      logger.error('Mint special card error:', error);
      res.status(500).json({
        success: false,
        message: 'Internal server error'
      });
    }
  };

  // Use Special Card
  public useSpecialCard = async (req: Request, res: Response): Promise<void> => {
    try {
      const { cardId } = req.params;
      const userId = req.user.id;

      const card = await NFT.findById(cardId);
      if (!card || card.ownerId.toString() !== userId) {
        res.status(404).json({
          success: false,
          message: 'Card not found or not owned'
        });
        return;
      }

      if (card.isUsed) {
        res.status(400).json({
          success: false,
          message: 'Card already used'
        });
        return;
      }

      // Apply card effects
      const effect = await this.applyCardEffect(userId, card);
      
      // Mark card as used
      card.isUsed = true;
      card.usedAt = new Date();
      await card.save();

      // Burn the NFT (single-use cards)
      await blockchainService.burnNFT(card.mintAddress);

      // Award XP for card usage
      await xpService.awardXP(userId, {
        activity: 'use_special_card',
        baseXP: 25,
        qualityMultiplier: 1.0,
        platform: 'finova'
      });

      logger.info(`Special card used: ${cardId} by user ${userId}`);

      res.status(200).json({
        success: true,
        data: {
          effect,
          cardName: card.name,
          usedAt: card.usedAt
        },
        message: 'Special card activated successfully'
      });

    } catch (error) {
      logger.error('Use special card error:', error);
      res.status(500).json({
        success: false,
        message: 'Internal server error'
      });
    }
  };

  // Get User's NFT Collection
  public getUserNFTs = async (req: Request, res: Response): Promise<void> => {
    try {
      const userId = req.user.id;
      const { type, page = 1, limit = 20 } = req.query;

      const filter: any = { ownerId: userId };
      if (type) filter.type = type;

      const nfts = await NFT.find(filter)
        .populate('creatorId', 'username profileImage')
        .sort({ createdAt: -1 })
        .limit(Number(limit) * Number(page))
        .skip((Number(page) - 1) * Number(limit));

      const total = await NFT.countDocuments(filter);

      // Get active card effects
      const activeEffects = await this.getActiveCardEffects(userId);

      res.status(200).json({
        success: true,
        data: {
          nfts: nfts.map(nft => ({
            id: nft._id,
            type: nft.type,
            name: nft.name,
            description: nft.description,
            image: nft.image,
            mintAddress: nft.mintAddress,
            rarity: nft.rarity,
            attributes: nft.attributes,
            isUsed: nft.isUsed,
            usedAt: nft.usedAt,
            creator: nft.creatorId
          })),
          activeEffects,
          pagination: {
            page: Number(page),
            limit: Number(limit),
            total,
            totalPages: Math.ceil(total / Number(limit))
          }
        }
      });

    } catch (error) {
      logger.error('Get user NFTs error:', error);
      res.status(500).json({
        success: false,
        message: 'Internal server error'
      });
    }
  };

  // NFT Marketplace - List for Sale
  public listForSale = async (req: Request, res: Response): Promise<void> => {
    try {
      const { nftId } = req.params;
      const { price, currency = 'FIN' } = req.body;
      const userId = req.user.id;

      const nft = await NFT.findById(nftId);
      if (!nft || nft.ownerId.toString() !== userId) {
        res.status(404).json({
          success: false,
          message: 'NFT not found or not owned'
        });
        return;
      }

      if (nft.type === 'special_card' && nft.isUsed) {
        res.status(400).json({
          success: false,
          message: 'Used cards cannot be sold'
        });
        return;
      }

      // Update NFT listing
      nft.isListed = true;
      nft.listPrice = price;
      nft.listCurrency = currency;
      nft.listedAt = new Date();
      await nft.save();

      // Cache listing for quick marketplace queries
      await redisClient.setex(
        `marketplace:${nftId}`,
        3600, // 1 hour cache
        JSON.stringify({
          id: nft._id,
          name: nft.name,
          price,
          currency,
          image: nft.image,
          rarity: nft.rarity,
          listedAt: nft.listedAt
        })
      );

      logger.info(`NFT listed: ${nftId} for ${price} ${currency} by user ${userId}`);

      res.status(200).json({
        success: true,
        data: {
          nft: {
            id: nft._id,
            name: nft.name,
            price,
            currency,
            listedAt: nft.listedAt
          }
        },
        message: 'NFT listed successfully'
      });

    } catch (error) {
      logger.error('List NFT error:', error);
      res.status(500).json({
        success: false,
        message: 'Internal server error'
      });
    }
  };

  // NFT Marketplace - Purchase
  public purchaseNFT = async (req: Request, res: Response): Promise<void> => {
    try {
      const { nftId } = req.params;
      const userId = req.user.id;

      const nft = await NFT.findById(nftId).populate('ownerId');
      if (!nft || !nft.isListed) {
        res.status(404).json({
          success: false,
          message: 'NFT not available for purchase'
        });
        return;
      }

      if (nft.ownerId._id.toString() === userId) {
        res.status(400).json({
          success: false,
          message: 'Cannot purchase your own NFT'
        });
        return;
      }

      const buyer = await User.findById(userId);
      const seller = nft.ownerId;

      // Check buyer balance
      if (buyer.finBalance < nft.listPrice) {
        res.status(400).json({
          success: false,
          message: 'Insufficient balance',
          required: nft.listPrice,
          balance: buyer.finBalance
        });
        return;
      }

      // Calculate fees
      const marketplaceFee = nft.listPrice * 0.025; // 2.5% marketplace fee
      const royaltyFee = nft.royalty ? (nft.listPrice * nft.royalty / 100) : 0;
      const sellerReceives = nft.listPrice - marketplaceFee - royaltyFee;

      // Process transaction
      buyer.finBalance -= nft.listPrice;
      seller.finBalance += sellerReceives;

      // Update NFT ownership
      nft.ownerId = buyer._id;
      nft.isListed = false;
      nft.listPrice = null;
      nft.listCurrency = null;
      nft.listedAt = null;
      nft.lastSalePrice = nft.listPrice;
      nft.lastSaleDate = new Date();

      await Promise.all([buyer.save(), seller.save(), nft.save()]);

      // Transfer NFT on blockchain
      await blockchainService.transferNFT(
        nft.mintAddress,
        seller.walletAddress,
        buyer.walletAddress
      );

      // Remove from marketplace cache
      await redisClient.del(`marketplace:${nftId}`);

      // Award XP to both parties
      await Promise.all([
        xpService.awardXP(buyer._id, {
          activity: 'nft_purchase',
          baseXP: 75,
          qualityMultiplier: 1.0,
          platform: 'finova'
        }),
        xpService.awardXP(seller._id, {
          activity: 'nft_sale',
          baseXP: 100,
          qualityMultiplier: 1.0,
          platform: 'finova'
        })
      ]);

      logger.info(`NFT purchased: ${nftId} by ${userId} from ${seller._id} for ${nft.listPrice} FIN`);

      res.status(200).json({
        success: true,
        data: {
          transaction: {
            nftId,
            price: nft.listPrice,
            marketplaceFee,
            royaltyFee,
            sellerReceives,
            purchasedAt: new Date()
          },
          newBalance: buyer.finBalance
        },
        message: 'NFT purchased successfully'
      });

    } catch (error) {
      logger.error('Purchase NFT error:', error);
      res.status(500).json({
        success: false,
        message: 'Internal server error'
      });
    }
  };

  // Get Marketplace Listings
  public getMarketplace = async (req: Request, res: Response): Promise<void> => {
    try {
      const {
        type,
        rarity,
        minPrice,
        maxPrice,
        sortBy = 'listedAt',
        sortOrder = 'desc',
        page = 1,
        limit = 20
      } = req.query;

      const filter: any = { isListed: true };
      if (type) filter.type = type;
      if (rarity) filter.rarity = rarity;
      if (minPrice || maxPrice) {
        filter.listPrice = {};
        if (minPrice) filter.listPrice.$gte = Number(minPrice);
        if (maxPrice) filter.listPrice.$lte = Number(maxPrice);
      }

      const sortOptions: any = {};
      sortOptions[sortBy as string] = sortOrder === 'desc' ? -1 : 1;

      const listings = await NFT.find(filter)
        .populate('ownerId', 'username profileImage xpLevel')
        .populate('creatorId', 'username profileImage')
        .sort(sortOptions)
        .limit(Number(limit) * Number(page))
        .skip((Number(page) - 1) * Number(limit));

      const total = await NFT.countDocuments(filter);

      // Get trending NFTs (cached)
      const trendingKey = 'marketplace:trending';
      let trending = await redisClient.get(trendingKey);
      if (!trending) {
        const trendingNFTs = await this.calculateTrendingNFTs();
        await redisClient.setex(trendingKey, 300, JSON.stringify(trendingNFTs)); // 5min cache
        trending = JSON.stringify(trendingNFTs);
      }

      res.status(200).json({
        success: true,
        data: {
          listings: listings.map(nft => ({
            id: nft._id,
            type: nft.type,
            name: nft.name,
            description: nft.description,
            image: nft.image,
            price: nft.listPrice,
            currency: nft.listCurrency,
            rarity: nft.rarity,
            attributes: nft.attributes,
            listedAt: nft.listedAt,
            owner: nft.ownerId,
            creator: nft.creatorId,
            lastSalePrice: nft.lastSalePrice
          })),
          trending: JSON.parse(trending),
          pagination: {
            page: Number(page),
            limit: Number(limit),
            total,
            totalPages: Math.ceil(total / Number(limit))
          }
        }
      });

    } catch (error) {
      logger.error('Get marketplace error:', error);
      res.status(500).json({
        success: false,
        message: 'Internal server error'
      });
    }
  };

  // Private helper methods
  private async createCollectionMint(
    creatorAddress: string,
    name: string,
    description: string,
    image: string,
    royalty: number
  ): Promise<PublicKey> {
    const creatorPubkey = new PublicKey(creatorAddress);
    
    // Create mint
    const mint = await createMint(
      this.connection,
      creatorPubkey as any,
      creatorPubkey,
      creatorPubkey,
      0 // 0 decimals for NFT
    );

    // Create metadata
    const metadataAddress = await this.createMetadata(
      mint,
      creatorPubkey,
      name,
      description,
      image,
      royalty
    );

    return mint;
  }

  private async mintCard(
    ownerAddress: string,
    cardConfig: any
  ): Promise<PublicKey> {
    const ownerPubkey = new PublicKey(ownerAddress);
    
    const mint = await createMint(
      this.connection,
      ownerPubkey as any,
      ownerPubkey,
      ownerPubkey,
      0
    );

    // Create associated token account
    const tokenAccount = await createAssociatedTokenAccount(
      this.connection,
      ownerPubkey as any,
      mint,
      ownerPubkey
    );

    // Mint the NFT
    await mintTo(
      this.connection,
      ownerPubkey as any,
      mint,
      tokenAccount,
      ownerPubkey,
      1
    );

    // Create metadata
    await this.createMetadata(
      mint,
      ownerPubkey,
      cardConfig.name,
      cardConfig.description,
      cardConfig.image,
      0
    );

    return mint;
  }

  private async createMetadata(
    mint: PublicKey,
    creator: PublicKey,
    name: string,
    description: string,
    image: string,
    royalty: number
  ): Promise<PublicKey> {
    // This is a simplified version - actual implementation would use Metaplex SDK
    // to create proper NFT metadata
    return mint; // Placeholder
  }

  private getSpecialCardConfig(cardType: string): any {
    const configs = {
      'double_mining': {
        name: 'Double Mining Card',
        description: 'Double your mining rate for 24 hours',
        image: 'https://finova.network/cards/double-mining.png',
        rarity: 'common',
        basePrice: 50,
        attributes: {
          effect: 'mining_boost',
          multiplier: 2.0,
          duration: 24 * 60 * 60 * 1000, // 24 hours in ms
          category: 'mining'
        }
      },
      'triple_mining': {
        name: 'Triple Mining Card',
        description: 'Triple your mining rate for 12 hours',
        image: 'https://finova.network/cards/triple-mining.png',
        rarity: 'rare',
        basePrice: 150,
        attributes: {
          effect: 'mining_boost',
          multiplier: 3.0,
          duration: 12 * 60 * 60 * 1000,
          category: 'mining'
        }
      },
      'xp_double': {
        name: 'XP Doubler Card',
        description: 'Double XP from all activities for 24 hours',
        image: 'https://finova.network/cards/xp-double.png',
        rarity: 'common',
        basePrice: 40,
        attributes: {
          effect: 'xp_boost',
          multiplier: 2.0,
          duration: 24 * 60 * 60 * 1000,
          category: 'experience'
        }
      },
      'referral_boost': {
        name: 'Referral Boost Card',
        description: 'Increase referral rewards by 50% for 7 days',
        image: 'https://finova.network/cards/referral-boost.png',
        rarity: 'uncommon',
        basePrice: 60,
        attributes: {
          effect: 'referral_boost',
          multiplier: 1.5,
          duration: 7 * 24 * 60 * 60 * 1000,
          category: 'referral'
        }
      }
    };

    return configs[cardType] || null;
  }

  private calculateCardPrice(cardConfig: any, quantity: number, user: any): number {
    let basePrice = cardConfig.basePrice * quantity;
    
    // Apply staking discounts
    const stakingDiscount = user.stakingTier?.discount || 0;
    basePrice *= (1 - stakingDiscount);
    
    // Apply XP level discounts
    if (user.xpLevel >= 50) basePrice *= 0.9; // 10% discount for Gold+
    if (user.xpLevel >= 75) basePrice *= 0.85; // Additional 5% for Platinum+
    
    return Math.floor(basePrice);
  }

  private async applyCardEffect(userId: string, card: any): Promise<any> {
    const effect = {
      type: card.attributes.effect,
      multiplier: card.attributes.multiplier,
      duration: card.attributes.duration,
      startTime: Date.now(),
      endTime: Date.now() + card.attributes.duration
    };

    // Store active effect in Redis for fast access
    await redisClient.setex(
      `card_effect:${userId}:${card.attributes.effect}:${Date.now()}`,
      Math.floor(card.attributes.duration / 1000),
      JSON.stringify(effect)
    );

    // Apply immediate effects based on card type
    switch (card.attributes.effect) {
      case 'mining_boost':
        await miningService.applyTemporaryBoost(userId, {
          multiplier: card.attributes.multiplier,
          duration: card.attributes.duration
        });
        break;
      
      case 'xp_boost':
        await xpService.applyTemporaryBoost(userId, {
          multiplier: card.attributes.multiplier,
          duration: card.attributes.duration
        });
        break;
    }

    return effect;
  }

  private async getActiveCardEffects(userId: string): Promise<any[]> {
    const pattern = `card_effect:${userId}:*`;
    const keys = await redisClient.keys(pattern);
    
    const effects = [];
    for (const key of keys) {
      const effectData = await redisClient.get(key);
      if (effectData) {
        const effect = JSON.parse(effectData);
        if (effect.endTime > Date.now()) {
          effects.push(effect);
        }
      }
    }
    
    return effects;
  }

  private async getNextSerialNumber(cardType: string): Promise<number> {
    const key = `serial:${cardType}`;
    return await redisClient.incr(key);
  }

  private async calculateTrendingNFTs(): Promise<any[]> {
    // Calculate trending based on recent sales volume, views, etc.
    const trending = await NFT.aggregate([
      { $match: { isListed: true, lastSaleDate: { $gte: new Date(Date.now() - 7 * 24 * 60 * 60 * 1000) } } },
      { $group: { _id: '$_id', salesCount: { $sum: 1 }, avgPrice: { $avg: '$lastSalePrice' } } },
      { $sort: { salesCount: -1, avgPrice: -1 } },
      { $limit: 10 }
    ]);

    return trending;
  }
}
