import { Router } from 'express';
import { body, param, query, validationResult } from 'express-validator';
import { authMiddleware } from '../middleware/auth.middleware';
import { kycMiddleware } from '../middleware/kyc.middleware';
import { rateLimitMiddleware } from '../middleware/rate-limit.middleware';
import { validationMiddleware } from '../middleware/validation.middleware';
import * as nftController from '../controllers/nft.controller';

const router = Router();

// Apply auth middleware to all NFT routes
router.use(authMiddleware);

/**
 * GET /api/nft/collections
 * Get all NFT collections with pagination
 */
router.get('/collections',
  rateLimitMiddleware({ windowMs: 60000, max: 100 }),
  [
    query('page').optional().isInt({ min: 1 }).withMessage('Page must be positive integer'),
    query('limit').optional().isInt({ min: 1, max: 100 }).withMessage('Limit must be 1-100'),
    query('category').optional().isIn(['mining', 'xp', 'referral', 'badge', 'special']).withMessage('Invalid category'),
    query('rarity').optional().isIn(['common', 'uncommon', 'rare', 'epic', 'legendary', 'mythic']).withMessage('Invalid rarity')
  ],
  validationMiddleware,
  nftController.getCollections
);

/**
 * GET /api/nft/collections/:id
 * Get specific collection details
 */
router.get('/collections/:id',
  rateLimitMiddleware({ windowMs: 60000, max: 200 }),
  [
    param('id').isMongoId().withMessage('Invalid collection ID')
  ],
  validationMiddleware,
  nftController.getCollection
);

/**
 * POST /api/nft/collections
 * Create new NFT collection (Admin only)
 */
router.post('/collections',
  rateLimitMiddleware({ windowMs: 3600000, max: 10 }),
  kycMiddleware,
  [
    body('name').notEmpty().isLength({ min: 3, max: 50 }).withMessage('Name must be 3-50 characters'),
    body('symbol').notEmpty().isLength({ min: 2, max: 10 }).withMessage('Symbol must be 2-10 characters'),
    body('description').notEmpty().isLength({ min: 10, max: 500 }).withMessage('Description must be 10-500 characters'),
    body('category').isIn(['mining', 'xp', 'referral', 'badge', 'special']).withMessage('Invalid category'),
    body('maxSupply').isInt({ min: 1 }).withMessage('Max supply must be positive integer'),
    body('royaltyPercent').optional().isFloat({ min: 0, max: 10 }).withMessage('Royalty must be 0-10%'),
    body('basePrice').isFloat({ min: 0 }).withMessage('Base price must be positive')
  ],
  validationMiddleware,
  nftController.createCollection
);

/**
 * GET /api/nft/cards
 * Get available special cards
 */
router.get('/cards',
  rateLimitMiddleware({ windowMs: 60000, max: 150 }),
  [
    query('page').optional().isInt({ min: 1 }).withMessage('Page must be positive integer'),
    query('limit').optional().isInt({ min: 1, max: 50 }).withMessage('Limit must be 1-50'),
    query('type').optional().isIn(['mining', 'xp', 'referral', 'combo']).withMessage('Invalid card type'),
    query('rarity').optional().isIn(['common', 'uncommon', 'rare', 'epic', 'legendary']).withMessage('Invalid rarity'),
    query('available').optional().isBoolean().withMessage('Available must be boolean')
  ],
  validationMiddleware,
  nftController.getSpecialCards
);

/**
 * GET /api/nft/cards/:id
 * Get specific special card details
 */
router.get('/cards/:id',
  rateLimitMiddleware({ windowMs: 60000, max: 200 }),
  [
    param('id').isMongoId().withMessage('Invalid card ID')
  ],
  validationMiddleware,
  nftController.getSpecialCard
);

/**
 * POST /api/nft/cards/purchase
 * Purchase special cards with $FIN
 */
router.post('/cards/purchase',
  rateLimitMiddleware({ windowMs: 60000, max: 20 }),
  kycMiddleware,
  [
    body('cardId').isMongoId().withMessage('Invalid card ID'),
    body('quantity').isInt({ min: 1, max: 10 }).withMessage('Quantity must be 1-10'),
    body('paymentMethod').isIn(['fin', 'sfin', 'usdfin']).withMessage('Invalid payment method'),
    body('walletAddress').notEmpty().withMessage('Wallet address required')
  ],
  validationMiddleware,
  nftController.purchaseSpecialCard
);

/**
 * POST /api/nft/cards/use
 * Use/activate a special card
 */
router.post('/cards/use',
  rateLimitMiddleware({ windowMs: 60000, max: 50 }),
  [
    body('cardId').isMongoId().withMessage('Invalid card ID'),
    body('nftTokenId').notEmpty().withMessage('NFT token ID required'),
    body('targetAction').optional().isIn(['mining', 'xp', 'referral', 'combo']).withMessage('Invalid target action')
  ],
  validationMiddleware,
  nftController.useSpecialCard
);

/**
 * GET /api/nft/user/inventory
 * Get user's NFT inventory
 */
router.get('/user/inventory',
  rateLimitMiddleware({ windowMs: 60000, max: 100 }),
  [
    query('page').optional().isInt({ min: 1 }).withMessage('Page must be positive integer'),
    query('limit').optional().isInt({ min: 1, max: 100 }).withMessage('Limit must be 1-100'),
    query('category').optional().isIn(['mining', 'xp', 'referral', 'badge', 'special']).withMessage('Invalid category'),
    query('status').optional().isIn(['active', 'used', 'expired', 'available']).withMessage('Invalid status')
  ],
  validationMiddleware,
  nftController.getUserInventory
);

/**
 * GET /api/nft/user/badges
 * Get user's achievement badges
 */
router.get('/user/badges',
  rateLimitMiddleware({ windowMs: 60000, max: 100 }),
  nftController.getUserBadges
);

/**
 * POST /api/nft/badges/claim
 * Claim achievement badge
 */
router.post('/badges/claim',
  rateLimitMiddleware({ windowMs: 60000, max: 20 }),
  [
    body('achievementId').isMongoId().withMessage('Invalid achievement ID'),
    body('proofData').optional().isObject().withMessage('Proof data must be object')
  ],
  validationMiddleware,
  nftController.claimBadge
);

/**
 * GET /api/nft/marketplace
 * Get marketplace listings
 */
router.get('/marketplace',
  rateLimitMiddleware({ windowMs: 60000, max: 150 }),
  [
    query('page').optional().isInt({ min: 1 }).withMessage('Page must be positive integer'),
    query('limit').optional().isInt({ min: 1, max: 50 }).withMessage('Limit must be 1-50'),
    query('category').optional().isIn(['mining', 'xp', 'referral', 'badge', 'special']).withMessage('Invalid category'),
    query('minPrice').optional().isFloat({ min: 0 }).withMessage('Min price must be positive'),
    query('maxPrice').optional().isFloat({ min: 0 }).withMessage('Max price must be positive'),
    query('rarity').optional().isIn(['common', 'uncommon', 'rare', 'epic', 'legendary', 'mythic']).withMessage('Invalid rarity'),
    query('sortBy').optional().isIn(['price', 'rarity', 'created', 'name']).withMessage('Invalid sort field'),
    query('sortOrder').optional().isIn(['asc', 'desc']).withMessage('Invalid sort order')
  ],
  validationMiddleware,
  nftController.getMarketplaceListings
);

/**
 * POST /api/nft/marketplace/list
 * List NFT for sale on marketplace
 */
router.post('/marketplace/list',
  rateLimitMiddleware({ windowMs: 3600000, max: 50 }),
  kycMiddleware,
  [
    body('nftTokenId').notEmpty().withMessage('NFT token ID required'),
    body('price').isFloat({ min: 0.01 }).withMessage('Price must be at least 0.01 FIN'),
    body('currency').isIn(['fin', 'usdfin', 'sol']).withMessage('Invalid currency'),
    body('duration').optional().isInt({ min: 1, max: 30 }).withMessage('Duration must be 1-30 days'),
    body('autoRenew').optional().isBoolean().withMessage('Auto renew must be boolean')
  ],
  validationMiddleware,
  nftController.listOnMarketplace
);

/**
 * POST /api/nft/marketplace/buy
 * Purchase NFT from marketplace
 */
router.post('/marketplace/buy',
  rateLimitMiddleware({ windowMs: 60000, max: 20 }),
  kycMiddleware,
  [
    body('listingId').isMongoId().withMessage('Invalid listing ID'),
    body('paymentMethod').isIn(['fin', 'sfin', 'usdfin', 'sol']).withMessage('Invalid payment method'),
    body('buyerWallet').notEmpty().withMessage('Buyer wallet address required')
  ],
  validationMiddleware,
  nftController.buyFromMarketplace
);

/**
 * DELETE /api/nft/marketplace/unlist/:listingId
 * Remove NFT listing from marketplace
 */
router.delete('/marketplace/unlist/:listingId',
  rateLimitMiddleware({ windowMs: 60000, max: 30 }),
  [
    param('listingId').isMongoId().withMessage('Invalid listing ID')
  ],
  validationMiddleware,
  nftController.unlistFromMarketplace
);

/**
 * POST /api/nft/transfer
 * Transfer NFT to another user
 */
router.post('/transfer',
  rateLimitMiddleware({ windowMs: 3600000, max: 20 }),
  kycMiddleware,
  [
    body('nftTokenId').notEmpty().withMessage('NFT token ID required'),
    body('recipientAddress').notEmpty().withMessage('Recipient address required'),
    body('transferFee').optional().isFloat({ min: 0 }).withMessage('Transfer fee must be positive'),
    body('memo').optional().isLength({ max: 200 }).withMessage('Memo must be under 200 characters')
  ],
  validationMiddleware,
  nftController.transferNFT
);

/**
 * GET /api/nft/stats
 * Get NFT marketplace and collection statistics
 */
router.get('/stats',
  rateLimitMiddleware({ windowMs: 60000, max: 50 }),
  nftController.getNFTStats
);

/**
 * GET /api/nft/user/activity
 * Get user's NFT transaction history
 */
router.get('/user/activity',
  rateLimitMiddleware({ windowMs: 60000, max: 100 }),
  [
    query('page').optional().isInt({ min: 1 }).withMessage('Page must be positive integer'),
    query('limit').optional().isInt({ min: 1, max: 100 }).withMessage('Limit must be 1-100'),
    query('type').optional().isIn(['mint', 'purchase', 'sale', 'transfer', 'use']).withMessage('Invalid activity type'),
    query('dateFrom').optional().isISO8601().withMessage('Invalid date format'),
    query('dateTo').optional().isISO8601().withMessage('Invalid date format')
  ],
  validationMiddleware,
  nftController.getUserNFTActivity
);

/**
 * POST /api/nft/metadata/update
 * Update NFT metadata (for upgradeable NFTs)
 */
router.post('/metadata/update',
  rateLimitMiddleware({ windowMs: 3600000, max: 10 }),
  kycMiddleware,
  [
    body('nftTokenId').notEmpty().withMessage('NFT token ID required'),
    body('updateType').isIn(['upgrade', 'evolve', 'customize']).withMessage('Invalid update type'),
    body('newMetadata').isObject().withMessage('New metadata must be object'),
    body('burnRequirement').optional().isArray().withMessage('Burn requirement must be array')
  ],
  validationMiddleware,
  nftController.updateNFTMetadata
);

/**
 * GET /api/nft/collections/:id/mint-status
 * Check minting availability for collection
 */
router.get('/collections/:id/mint-status',
  rateLimitMiddleware({ windowMs: 60000, max: 200 }),
  [
    param('id').isMongoId().withMessage('Invalid collection ID')
  ],
  validationMiddleware,
  nftController.getCollectionMintStatus
);

/**
 * POST /api/nft/whitelist/register
 * Register for NFT whitelist
 */
router.post('/whitelist/register',
  rateLimitMiddleware({ windowMs: 3600000, max: 5 }),
  kycMiddleware,
  [
    body('collectionId').isMongoId().withMessage('Invalid collection ID'),
    body('whitelistType').isIn(['early_access', 'discount', 'guaranteed']).withMessage('Invalid whitelist type'),
    body('referralCode').optional().isLength({ min: 6, max: 20 }).withMessage('Invalid referral code')
  ],
  validationMiddleware,
  nftController.registerForWhitelist
);

/**
 * GET /api/nft/combos/available
 * Get available card combinations and bonuses
 */
router.get('/combos/available',
  rateLimitMiddleware({ windowMs: 60000, max: 100 }),
  nftController.getAvailableCombos
);

/**
 * POST /api/nft/combos/activate
 * Activate card combination for bonus effects
 */
router.post('/combos/activate',
  rateLimitMiddleware({ windowMs: 60000, max: 10 }),
  [
    body('cardIds').isArray({ min: 2, max: 5 }).withMessage('Combo requires 2-5 cards'),
    body('cardIds.*').isMongoId().withMessage('Invalid card ID in combo'),
    body('comboType').isIn(['mining_boost', 'xp_multiplier', 'rp_amplifier', 'special_event']).withMessage('Invalid combo type')
  ],
  validationMiddleware,
  nftController.activateCombo
);

/**
 * GET /api/nft/leaderboard
 * Get NFT collection and trading leaderboards
 */
router.get('/leaderboard',
  rateLimitMiddleware({ windowMs: 60000, max: 50 }),
  [
    query('type').isIn(['collectors', 'traders', 'creators', 'card_users']).withMessage('Invalid leaderboard type'),
    query('period').optional().isIn(['daily', 'weekly', 'monthly', 'all_time']).withMessage('Invalid period'),
    query('limit').optional().isInt({ min: 1, max: 100 }).withMessage('Limit must be 1-100')
  ],
  validationMiddleware,
  nftController.getNFTLeaderboard
);

/**
 * Error handling for invalid routes
 */
router.use('*', (req, res) => {
  res.status(404).json({
    success: false,
    error: 'NFT endpoint not found',
    availableEndpoints: [
      'GET /collections', 'POST /collections',
      'GET /cards', 'POST /cards/purchase', 'POST /cards/use',
      'GET /user/inventory', 'GET /user/badges', 'POST /badges/claim',
      'GET /marketplace', 'POST /marketplace/list', 'POST /marketplace/buy',
      'POST /transfer', 'GET /stats', 'GET /user/activity'
    ]
  });
});

export default router;
