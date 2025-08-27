import { Router } from 'express';
import { body, param, query, validationResult } from 'express-validator';
import { UserController } from '../controllers/user.controller';
import { authMiddleware } from '../middleware/auth.middleware';
import { kycMiddleware } from '../middleware/kyc.middleware';
import { rateLimitMiddleware } from '../middleware/rate-limit.middleware';
import { validationMiddleware } from '../middleware/validation.middleware';

const router = Router();
const userController = new UserController();

// Rate limiting configurations
const standardLimit = rateLimitMiddleware({ windowMs: 60 * 1000, max: 100 }); // 100 req/min
const strictLimit = rateLimitMiddleware({ windowMs: 60 * 1000, max: 20 });   // 20 req/min

// User Registration & Authentication
router.post('/register',
  standardLimit,
  [
    body('email').isEmail().normalizeEmail(),
    body('password').isLength({ min: 8 }).matches(/^(?=.*[a-z])(?=.*[A-Z])(?=.*\d)(?=.*[@$!%*?&])/),
    body('username').isAlphanumeric().isLength({ min: 3, max: 20 }),
    body('referralCode').optional().isAlphanumeric(),
    body('biometricData').optional().isObject(),
    body('deviceFingerprint').isString().isLength({ min: 10 })
  ],
  validationMiddleware,
  userController.register
);

router.post('/login',
  standardLimit,
  [
    body('email').isEmail().normalizeEmail(),
    body('password').isString(),
    body('biometricData').optional().isObject(),
    body('deviceFingerprint').isString()
  ],
  validationMiddleware,
  userController.login
);

router.post('/logout',
  authMiddleware,
  userController.logout
);

router.post('/refresh-token',
  strictLimit,
  [body('refreshToken').isJWT()],
  validationMiddleware,
  userController.refreshToken
);

// Profile Management
router.get('/profile',
  authMiddleware,
  userController.getProfile
);

router.put('/profile',
  authMiddleware,
  [
    body('firstName').optional().isString().isLength({ min: 1, max: 50 }),
    body('lastName').optional().isString().isLength({ min: 1, max: 50 }),
    body('bio').optional().isString().isLength({ max: 500 }),
    body('avatar').optional().isURL(),
    body('timezone').optional().isString()
  ],
  validationMiddleware,
  userController.updateProfile
);

router.get('/dashboard',
  authMiddleware,
  userController.getDashboard
);

// KYC Management
router.post('/kyc/submit',
  authMiddleware,
  strictLimit,
  [
    body('idType').isIn(['passport', 'nationalId', 'drivingLicense']),
    body('idNumber').isString().isLength({ min: 5, max: 30 }),
    body('fullName').isString().isLength({ min: 2, max: 100 }),
    body('dateOfBirth').isISO8601().toDate(),
    body('address').isObject(),
    body('selfieImage').isString(), // Base64 encoded
    body('idFrontImage').isString(),
    body('idBackImage').optional().isString(),
    body('biometricData').isObject()
  ],
  validationMiddleware,
  userController.submitKYC
);

router.get('/kyc/status',
  authMiddleware,
  userController.getKYCStatus
);

// Mining Operations
router.post('/mining/start',
  authMiddleware,
  kycMiddleware,
  [body('deviceInfo').isObject()],
  validationMiddleware,
  userController.startMining
);

router.post('/mining/stop',
  authMiddleware,
  userController.stopMining
);

router.get('/mining/status',
  authMiddleware,
  userController.getMiningStatus
);

router.post('/mining/claim-rewards',
  authMiddleware,
  kycMiddleware,
  strictLimit,
  userController.claimMiningRewards
);

router.get('/mining/history',
  authMiddleware,
  [
    query('page').optional().isInt({ min: 1 }).toInt(),
    query('limit').optional().isInt({ min: 1, max: 100 }).toInt(),
    query('startDate').optional().isISO8601().toDate(),
    query('endDate').optional().isISO8601().toDate()
  ],
  validationMiddleware,
  userController.getMiningHistory
);

// XP System
router.get('/xp/stats',
  authMiddleware,
  userController.getXPStats
);

router.post('/xp/track-activity',
  authMiddleware,
  [
    body('platform').isIn(['instagram', 'tiktok', 'youtube', 'facebook', 'twitter-x']),
    body('activityType').isIn(['post', 'comment', 'like', 'share', 'follow', 'story']),
    body('contentId').optional().isString(),
    body('contentUrl').optional().isURL(),
    body('engagement').optional().isObject(),
    body('metadata').optional().isObject()
  ],
  validationMiddleware,
  userController.trackXPActivity
);

router.get('/xp/level-progress',
  authMiddleware,
  userController.getXPLevelProgress
);

router.get('/xp/activities',
  authMiddleware,
  [
    query('page').optional().isInt({ min: 1 }).toInt(),
    query('limit').optional().isInt({ min: 1, max: 50 }).toInt(),
    query('platform').optional().isIn(['instagram', 'tiktok', 'youtube', 'facebook', 'twitter-x'])
  ],
  validationMiddleware,
  userController.getXPActivities
);

// Referral System
router.get('/referral/code',
  authMiddleware,
  userController.getReferralCode
);

router.post('/referral/generate-custom-code',
  authMiddleware,
  [body('customCode').isAlphanumeric().isLength({ min: 6, max: 12 })],
  validationMiddleware,
  userController.generateCustomReferralCode
);

router.get('/referral/stats',
  authMiddleware,
  userController.getReferralStats
);

router.get('/referral/network',
  authMiddleware,
  [
    query('level').optional().isInt({ min: 1, max: 3 }).toInt(),
    query('page').optional().isInt({ min: 1 }).toInt(),
    query('limit').optional().isInt({ min: 1, max: 100 }).toInt()
  ],
  validationMiddleware,
  userController.getReferralNetwork
);

router.get('/referral/earnings',
  authMiddleware,
  [
    query('period').optional().isIn(['daily', 'weekly', 'monthly', 'all']),
    query('page').optional().isInt({ min: 1 }).toInt(),
    query('limit').optional().isInt({ min: 1, max: 100 }).toInt()
  ],
  validationMiddleware,
  userController.getReferralEarnings
);

// Staking Operations
router.post('/staking/stake',
  authMiddleware,
  kycMiddleware,
  [
    body('amount').isNumeric().custom((value) => {
      if (parseFloat(value) < 100) throw new Error('Minimum stake is 100 $FIN');
      return true;
    }),
    body('duration').optional().isIn(['flexible', '30d', '90d', '180d', '365d'])
  ],
  validationMiddleware,
  userController.stakeTokens
);

router.post('/staking/unstake',
  authMiddleware,
  [
    body('stakeId').isUUID(),
    body('amount').optional().isNumeric()
  ],
  validationMiddleware,
  userController.unstakeTokens
);

router.get('/staking/positions',
  authMiddleware,
  userController.getStakingPositions
);

router.post('/staking/claim-rewards',
  authMiddleware,
  strictLimit,
  userController.claimStakingRewards
);

// NFT & Special Cards
router.get('/nft/collection',
  authMiddleware,
  [
    query('category').optional().isIn(['mining', 'xp', 'referral', 'achievement']),
    query('rarity').optional().isIn(['common', 'uncommon', 'rare', 'epic', 'legendary']),
    query('page').optional().isInt({ min: 1 }).toInt(),
    query('limit').optional().isInt({ min: 1, max: 50 }).toInt()
  ],
  validationMiddleware,
  userController.getNFTCollection
);

router.post('/nft/use-special-card',
  authMiddleware,
  [
    body('cardId').isUUID(),
    body('targetActivity').optional().isString()
  ],
  validationMiddleware,
  userController.useSpecialCard
);

router.get('/nft/marketplace',
  authMiddleware,
  [
    query('category').optional().isIn(['mining', 'xp', 'referral', 'achievement']),
    query('minPrice').optional().isNumeric(),
    query('maxPrice').optional().isNumeric(),
    query('sortBy').optional().isIn(['price', 'rarity', 'newest']),
    query('page').optional().isInt({ min: 1 }).toInt(),
    query('limit').optional().isInt({ min: 1, max: 50 }).toInt()
  ],
  validationMiddleware,
  userController.getNFTMarketplace
);

router.post('/nft/purchase',
  authMiddleware,
  kycMiddleware,
  [
    body('nftId').isUUID(),
    body('paymentMethod').isIn(['fin', 'usdFin'])
  ],
  validationMiddleware,
  userController.purchaseNFT
);

// Guild System
router.get('/guild/search',
  authMiddleware,
  [
    query('name').optional().isString(),
    query('category').optional().isString(),
    query('minMembers').optional().isInt({ min: 1 }).toInt(),
    query('maxMembers').optional().isInt({ min: 1 }).toInt(),
    query('page').optional().isInt({ min: 1 }).toInt(),
    query('limit').optional().isInt({ min: 1, max: 50 }).toInt()
  ],
  validationMiddleware,
  userController.searchGuilds
);

router.post('/guild/join',
  authMiddleware,
  [body('guildId').isUUID()],
  validationMiddleware,
  userController.joinGuild
);

router.post('/guild/leave',
  authMiddleware,
  userController.leaveGuild
);

router.get('/guild/current',
  authMiddleware,
  userController.getCurrentGuild
);

// Wallet & Transactions
router.get('/wallet/balance',
  authMiddleware,
  userController.getWalletBalance
);

router.get('/wallet/transactions',
  authMiddleware,
  [
    query('type').optional().isIn(['mining', 'staking', 'referral', 'nft', 'guild']),
    query('status').optional().isIn(['pending', 'completed', 'failed']),
    query('page').optional().isInt({ min: 1 }).toInt(),
    query('limit').optional().isInt({ min: 1, max: 100 }).toInt(),
    query('startDate').optional().isISO8601().toDate(),
    query('endDate').optional().isISO8601().toDate()
  ],
  validationMiddleware,
  userController.getTransactionHistory
);

router.post('/wallet/transfer',
  authMiddleware,
  kycMiddleware,
  strictLimit,
  [
    body('recipientAddress').isString().isLength({ min: 32, max: 44 }),
    body('amount').isNumeric().custom((value) => {
      if (parseFloat(value) <= 0) throw new Error('Amount must be positive');
      return true;
    }),
    body('tokenType').isIn(['fin', 'usdFin']),
    body('memo').optional().isString().isLength({ max: 100 })
  ],
  validationMiddleware,
  userController.transferTokens
);

// E-wallet Integration
router.post('/ewallet/connect',
  authMiddleware,
  kycMiddleware,
  [
    body('provider').isIn(['ovo', 'gopay', 'dana', 'shopeepay']),
    body('phoneNumber').isMobilePhone('id-ID'),
    body('verificationCode').isNumeric().isLength({ min: 4, max: 6 })
  ],
  validationMiddleware,
  userController.connectEWallet
);

router.post('/ewallet/withdraw',
  authMiddleware,
  kycMiddleware,
  strictLimit,
  [
    body('provider').isIn(['ovo', 'gopay', 'dana', 'shopeepay']),
    body('amount').isNumeric().custom((value) => {
      const amt = parseFloat(value);
      if (amt < 50000) throw new Error('Minimum withdrawal is IDR 50,000');
      if (amt > 10000000) throw new Error('Maximum withdrawal is IDR 10,000,000');
      return true;
    }),
    body('pin').isNumeric().isLength({ min: 6, max: 6 })
  ],
  validationMiddleware,
  userController.withdrawToEWallet
);

router.get('/ewallet/history',
  authMiddleware,
  [
    query('provider').optional().isIn(['ovo', 'gopay', 'dana', 'shopeepay']),
    query('type').optional().isIn(['withdrawal', 'deposit']),
    query('page').optional().isInt({ min: 1 }).toInt(),
    query('limit').optional().isInt({ min: 1, max: 50 }).toInt()
  ],
  validationMiddleware,
  userController.getEWalletHistory
);

// Social Media Integrations
router.post('/social/connect',
  authMiddleware,
  [
    body('platform').isIn(['instagram', 'tiktok', 'youtube', 'facebook', 'twitter-x']),
    body('accessToken').isString(),
    body('refreshToken').optional().isString(),
    body('platformUserId').isString(),
    body('username').isString()
  ],
  validationMiddleware,
  userController.connectSocialPlatform
);

router.post('/social/disconnect',
  authMiddleware,
  [body('platform').isIn(['instagram', 'tiktok', 'youtube', 'facebook', 'twitter-x'])],
  validationMiddleware,
  userController.disconnectSocialPlatform
);

router.get('/social/connected',
  authMiddleware,
  userController.getConnectedPlatforms
);

router.post('/social/sync-activity',
  authMiddleware,
  [
    body('platform').isIn(['instagram', 'tiktok', 'youtube', 'facebook', 'twitter-x']),
    body('forceSync').optional().isBoolean()
  ],
  validationMiddleware,
  userController.syncSocialActivity
);

// Analytics & Statistics
router.get('/analytics/summary',
  authMiddleware,
  [query('period').optional().isIn(['7d', '30d', '90d', '1y'])],
  validationMiddleware,
  userController.getAnalyticsSummary
);

router.get('/analytics/mining-performance',
  authMiddleware,
  [
    query('period').optional().isIn(['7d', '30d', '90d']),
    query('groupBy').optional().isIn(['day', 'week', 'month'])
  ],
  validationMiddleware,
  userController.getMiningPerformance
);

router.get('/analytics/xp-trends',
  authMiddleware,
  [
    query('period').optional().isIn(['7d', '30d', '90d']),
    query('platform').optional().isIn(['instagram', 'tiktok', 'youtube', 'facebook', 'twitter-x'])
  ],
  validationMiddleware,
  userController.getXPTrends
);

router.get('/analytics/network-growth',
  authMiddleware,
  [query('period').optional().isIn(['7d', '30d', '90d'])],
  validationMiddleware,
  userController.getNetworkGrowth
);

// Achievements & Milestones
router.get('/achievements',
  authMiddleware,
  [
    query('category').optional().isIn(['mining', 'xp', 'referral', 'social', 'special']),
    query('status').optional().isIn(['completed', 'in-progress', 'locked'])
  ],
  validationMiddleware,
  userController.getAchievements
);

router.post('/achievements/claim',
  authMiddleware,
  [body('achievementId').isUUID()],
  validationMiddleware,
  userController.claimAchievement
);

// Notifications & Settings
router.get('/notifications',
  authMiddleware,
  [
    query('type').optional().isIn(['mining', 'xp', 'referral', 'system', 'achievement']),
    query('unread').optional().isBoolean(),
    query('page').optional().isInt({ min: 1 }).toInt(),
    query('limit').optional().isInt({ min: 1, max: 50 }).toInt()
  ],
  validationMiddleware,
  userController.getNotifications
);

router.post('/notifications/mark-read',
  authMiddleware,
  [body('notificationIds').isArray()],
  validationMiddleware,
  userController.markNotificationsRead
);

router.get('/settings',
  authMiddleware,
  userController.getSettings
);

router.put('/settings',
  authMiddleware,
  [
    body('notifications').optional().isObject(),
    body('privacy').optional().isObject(),
    body('mining').optional().isObject(),
    body('display').optional().isObject()
  ],
  validationMiddleware,
  userController.updateSettings
);

// Security & Account Management
router.post('/security/change-password',
  authMiddleware,
  [
    body('currentPassword').isString(),
    body('newPassword').isLength({ min: 8 }).matches(/^(?=.*[a-z])(?=.*[A-Z])(?=.*\d)(?=.*[@$!%*?&])/),
    body('confirmPassword').custom((value, { req }) => {
      if (value !== req.body.newPassword) throw new Error('Passwords do not match');
      return true;
    })
  ],
  validationMiddleware,
  userController.changePassword
);

router.post('/security/enable-2fa',
  authMiddleware,
  [
    body('method').isIn(['sms', 'email', 'authenticator']),
    body('verificationCode').isNumeric().isLength({ min: 6, max: 6 })
  ],
  validationMiddleware,
  userController.enable2FA
);

router.post('/security/disable-2fa',
  authMiddleware,
  [body('verificationCode').isNumeric().isLength({ min: 6, max: 6 })],
  validationMiddleware,
  userController.disable2FA
);

router.delete('/account',
  authMiddleware,
  [
    body('password').isString(),
    body('reason').optional().isString().isLength({ max: 500 }),
    body('confirmation').equals('DELETE_MY_ACCOUNT')
  ],
  validationMiddleware,
  userController.deleteAccount
);

// Error handling middleware
router.use((error: any, req: any, res: any, next: any) => {
  const errors = validationResult(req);
  if (!errors.isEmpty()) {
    return res.status(400).json({
      success: false,
      message: 'Validation failed',
      errors: errors.array()
    });
  }
  next(error);
});

export default router;
