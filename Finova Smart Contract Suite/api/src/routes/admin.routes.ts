import { Router } from 'express';
import { body, param, query } from 'express-validator';
import rateLimit from 'express-rate-limit';
import {
  authMiddleware,
  adminMiddleware,
  superAdminMiddleware,
  validationMiddleware,
  rateLimitMiddleware
} from '../middleware';
import {
  AdminController,
  UserManagementController,
  MiningManagementController,
  NFTManagementController,
  AnalyticsController,
  SecurityController,
  SystemController
} from '../controllers';

const router = Router();

// Rate limiting for admin routes
const adminRateLimit = rateLimit({
  windowMs: 15 * 60 * 1000, // 15 minutes
  max: 100, // limit each IP to 100 requests per windowMs
  message: 'Too many admin requests from this IP',
  standardHeaders: true,
  legacyHeaders: false,
});

const strictAdminRateLimit = rateLimit({
  windowMs: 60 * 1000, // 1 minute
  max: 10, // limit to 10 requests per minute for sensitive operations
  message: 'Rate limit exceeded for sensitive admin operations',
});

// Apply rate limiting to all admin routes
router.use(adminRateLimit);

// Authentication middleware for all admin routes
router.use(authMiddleware);

// =====================================
// USER MANAGEMENT ROUTES
// =====================================

// Get all users with pagination and filtering
router.get('/users',
  adminMiddleware,
  [
    query('page').optional().isInt({ min: 1 }).withMessage('Page must be a positive integer'),
    query('limit').optional().isInt({ min: 1, max: 100 }).withMessage('Limit must be between 1-100'),
    query('status').optional().isIn(['active', 'suspended', 'banned', 'pending']),
    query('kyc_status').optional().isIn(['pending', 'approved', 'rejected']),
    query('search').optional().isLength({ min: 1, max: 100 }),
    query('sort_by').optional().isIn(['created_at', 'last_active', 'total_fin', 'xp_level', 'rp_tier']),
    query('sort_order').optional().isIn(['asc', 'desc'])
  ],
  validationMiddleware,
  UserManagementController.getAllUsers
);

// Get user details
router.get('/users/:userId',
  adminMiddleware,
  [
    param('userId').isUUID().withMessage('Invalid user ID')
  ],
  validationMiddleware,
  UserManagementController.getUserDetails
);

// Update user status
router.patch('/users/:userId/status',
  adminMiddleware,
  strictAdminRateLimit,
  [
    param('userId').isUUID().withMessage('Invalid user ID'),
    body('status').isIn(['active', 'suspended', 'banned']).withMessage('Invalid status'),
    body('reason').isLength({ min: 10, max: 500 }).withMessage('Reason must be 10-500 characters'),
    body('admin_notes').optional().isLength({ max: 1000 })
  ],
  validationMiddleware,
  UserManagementController.updateUserStatus
);

// Approve/Reject KYC
router.patch('/users/:userId/kyc',
  adminMiddleware,
  strictAdminRateLimit,
  [
    param('userId').isUUID().withMessage('Invalid user ID'),
    body('action').isIn(['approve', 'reject']).withMessage('Action must be approve or reject'),
    body('reason').if(body('action').equals('reject'))
      .isLength({ min: 10, max: 500 }).withMessage('Rejection reason required'),
    body('admin_notes').optional().isLength({ max: 1000 })
  ],
  validationMiddleware,
  UserManagementController.processKYC
);

// Manual XP adjustment
router.post('/users/:userId/xp/adjust',
  superAdminMiddleware,
  strictAdminRateLimit,
  [
    param('userId').isUUID().withMessage('Invalid user ID'),
    body('amount').isInt({ min: -100000, max: 100000 }).withMessage('XP adjustment must be between -100k and 100k'),
    body('reason').isLength({ min: 10, max: 500 }).withMessage('Reason required'),
    body('type').isIn(['bonus', 'penalty', 'correction']).withMessage('Invalid adjustment type')
  ],
  validationMiddleware,
  UserManagementController.adjustUserXP
);

// Bulk user operations
router.post('/users/bulk-action',
  superAdminMiddleware,
  strictAdminRateLimit,
  [
    body('user_ids').isArray({ min: 1, max: 100 }).withMessage('Must select 1-100 users'),
    body('user_ids.*').isUUID().withMessage('Invalid user ID format'),
    body('action').isIn(['suspend', 'activate', 'ban', 'kyc_reset']).withMessage('Invalid bulk action'),
    body('reason').isLength({ min: 10, max: 500 }).withMessage('Reason required for bulk action')
  ],
  validationMiddleware,
  UserManagementController.bulkUserAction
);

// =====================================
// MINING MANAGEMENT ROUTES
// =====================================

// Get mining statistics
router.get('/mining/stats',
  adminMiddleware,
  [
    query('period').optional().isIn(['24h', '7d', '30d', '90d', '1y']),
    query('breakdown').optional().isIn(['hourly', 'daily', 'weekly', 'monthly'])
  ],
  validationMiddleware,
  MiningManagementController.getMiningStats
);

// Update mining parameters
router.patch('/mining/parameters',
  superAdminMiddleware,
  strictAdminRateLimit,
  [
    body('base_rate').optional().isFloat({ min: 0.001, max: 1.0 }).withMessage('Base rate must be 0.001-1.0'),
    body('pioneer_bonus').optional().isFloat({ min: 1.0, max: 3.0 }).withMessage('Pioneer bonus must be 1.0-3.0'),
    body('regression_factor').optional().isFloat({ min: 0.0001, max: 0.01 }).withMessage('Invalid regression factor'),
    body('daily_cap_multiplier').optional().isFloat({ min: 1.0, max: 5.0 }).withMessage('Daily cap multiplier must be 1.0-5.0'),
    body('effective_date').optional().isISO8601().withMessage('Invalid effective date')
  ],
  validationMiddleware,
  MiningManagementController.updateMiningParameters
);

// Manual mining adjustment
router.post('/mining/adjust/:userId',
  superAdminMiddleware,
  strictAdminRateLimit,
  [
    param('userId').isUUID().withMessage('Invalid user ID'),
    body('amount').isFloat({ min: -10000, max: 10000 }).withMessage('Amount must be between -10k and 10k FIN'),
    body('reason').isLength({ min: 10, max: 500 }).withMessage('Reason required'),
    body('type').isIn(['bonus', 'penalty', 'correction', 'compensation']).withMessage('Invalid adjustment type')
  ],
  validationMiddleware,
  MiningManagementController.adjustUserMining
);

// Pause/Resume mining globally
router.post('/mining/global-control',
  superAdminMiddleware,
  strictAdminRateLimit,
  [
    body('action').isIn(['pause', 'resume', 'emergency_stop']).withMessage('Invalid mining control action'),
    body('reason').isLength({ min: 10, max: 500 }).withMessage('Reason required'),
    body('duration').if(body('action').equals('pause'))
      .isInt({ min: 1, max: 168 }).withMessage('Pause duration must be 1-168 hours')
  ],
  validationMiddleware,
  MiningManagementController.globalMiningControl
);

// Get mining anomalies
router.get('/mining/anomalies',
  adminMiddleware,
  [
    query('severity').optional().isIn(['low', 'medium', 'high', 'critical']),
    query('status').optional().isIn(['open', 'investigating', 'resolved']),
    query('limit').optional().isInt({ min: 1, max: 100 })
  ],
  validationMiddleware,
  MiningManagementController.getMiningAnomalies
);

// =====================================
// NFT MANAGEMENT ROUTES
// =====================================

// Get NFT collection stats
router.get('/nft/stats',
  adminMiddleware,
  NFTManagementController.getNFTStats
);

// Create special card collection
router.post('/nft/special-cards/collection',
  adminMiddleware,
  [
    body('name').isLength({ min: 3, max: 50 }).withMessage('Collection name must be 3-50 characters'),
    body('description').isLength({ min: 10, max: 500 }).withMessage('Description must be 10-500 characters'),
    body('max_supply').isInt({ min: 1, max: 1000000 }).withMessage('Max supply must be 1-1M'),
    body('card_type').isIn(['mining_boost', 'xp_accelerator', 'referral_power', 'special_event']),
    body('rarity').isIn(['common', 'uncommon', 'rare', 'epic', 'legendary']),
    body('effect_params').isObject().withMessage('Effect parameters must be an object'),
    body('price_fin').isFloat({ min: 1, max: 100000 }).withMessage('Price must be 1-100k FIN')
  ],
  validationMiddleware,
  NFTManagementController.createSpecialCardCollection
);

// Update NFT metadata
router.patch('/nft/:nftId/metadata',
  adminMiddleware,
  [
    param('nftId').isUUID().withMessage('Invalid NFT ID'),
    body('name').optional().isLength({ min: 3, max: 50 }),
    body('description').optional().isLength({ min: 10, max: 500 }),
    body('attributes').optional().isArray(),
    body('image_url').optional().isURL()
  ],
  validationMiddleware,
  NFTManagementController.updateNFTMetadata
);

// Mint special cards to users
router.post('/nft/mint-to-user',
  adminMiddleware,
  strictAdminRateLimit,
  [
    body('user_id').isUUID().withMessage('Invalid user ID'),
    body('collection_id').isUUID().withMessage('Invalid collection ID'),
    body('quantity').isInt({ min: 1, max: 100 }).withMessage('Quantity must be 1-100'),
    body('reason').isLength({ min: 10, max: 500 }).withMessage('Reason required')
  ],
  validationMiddleware,
  NFTManagementController.mintToUser
);

// Freeze/Unfreeze NFT trading
router.patch('/nft/:nftId/trading-status',
  adminMiddleware,
  [
    param('nftId').isUUID().withMessage('Invalid NFT ID'),
    body('action').isIn(['freeze', 'unfreeze']).withMessage('Action must be freeze or unfreeze'),
    body('reason').isLength({ min: 10, max: 500 }).withMessage('Reason required')
  ],
  validationMiddleware,
  NFTManagementController.updateTradingStatus
);

// =====================================
// ANALYTICS & REPORTING ROUTES
// =====================================

// Dashboard overview
router.get('/analytics/dashboard',
  adminMiddleware,
  AnalyticsController.getDashboardOverview
);

// User analytics
router.get('/analytics/users',
  adminMiddleware,
  [
    query('period').optional().isIn(['24h', '7d', '30d', '90d', '1y']),
    query('metrics').optional().isIn(['growth', 'engagement', 'retention', 'revenue']),
    query('breakdown').optional().isIn(['daily', 'weekly', 'monthly'])
  ],
  validationMiddleware,
  AnalyticsController.getUserAnalytics
);

// Revenue analytics
router.get('/analytics/revenue',
  adminMiddleware,
  [
    query('period').optional().isIn(['7d', '30d', '90d', '1y']),
    query('breakdown').optional().isIn(['daily', 'weekly', 'monthly']),
    query('category').optional().isIn(['nft', 'premium', 'partnerships', 'fees'])
  ],
  validationMiddleware,
  AnalyticsController.getRevenueAnalytics
);

// Network effect analytics
router.get('/analytics/network',
  adminMiddleware,
  [
    query('period').optional().isIn(['7d', '30d', '90d']),
    query('depth').optional().isInt({ min: 1, max: 5 }).withMessage('Network depth must be 1-5')
  ],
  validationMiddleware,
  AnalyticsController.getNetworkAnalytics
);

// Export analytics data
router.post('/analytics/export',
  adminMiddleware,
  [
    body('report_type').isIn(['users', 'mining', 'revenue', 'network', 'comprehensive']),
    body('format').isIn(['csv', 'xlsx', 'json']),
    body('date_from').isISO8601().withMessage('Invalid from date'),
    body('date_to').isISO8601().withMessage('Invalid to date'),
    body('filters').optional().isObject()
  ],
  validationMiddleware,
  AnalyticsController.exportAnalytics
);

// =====================================
// SECURITY & FRAUD DETECTION ROUTES
// =====================================

// Get security alerts
router.get('/security/alerts',
  adminMiddleware,
  [
    query('severity').optional().isIn(['low', 'medium', 'high', 'critical']),
    query('status').optional().isIn(['open', 'investigating', 'resolved', 'false_positive']),
    query('category').optional().isIn(['bot_detection', 'fraud', 'suspicious_activity', 'system']),
    query('limit').optional().isInt({ min: 1, max: 100 })
  ],
  validationMiddleware,
  SecurityController.getSecurityAlerts
);

// Update alert status
router.patch('/security/alerts/:alertId',
  adminMiddleware,
  [
    param('alertId').isUUID().withMessage('Invalid alert ID'),
    body('status').isIn(['investigating', 'resolved', 'false_positive']),
    body('admin_notes').optional().isLength({ max: 1000 }),
    body('resolution_action').optional().isLength({ max: 500 })
  ],
  validationMiddleware,
  SecurityController.updateAlertStatus
);

// Bot detection results
router.get('/security/bot-detection',
  adminMiddleware,
  [
    query('confidence_min').optional().isFloat({ min: 0, max: 1 }),
    query('status').optional().isIn(['flagged', 'reviewed', 'confirmed', 'false_positive']),
    query('period').optional().isIn(['24h', '7d', '30d'])
  ],
  validationMiddleware,
  SecurityController.getBotDetectionResults
);

// Manual bot review
router.post('/security/bot-review/:userId',
  adminMiddleware,
  strictAdminRateLimit,
  [
    param('userId').isUUID().withMessage('Invalid user ID'),
    body('verdict').isIn(['human', 'bot', 'suspicious']).withMessage('Invalid verdict'),
    body('confidence').isFloat({ min: 0, max: 1 }).withMessage('Confidence must be 0-1'),
    body('evidence').optional().isArray(),
    body('admin_notes').optional().isLength({ max: 1000 })
  ],
  validationMiddleware,
  SecurityController.manualBotReview
);

// Fraud pattern analysis
router.get('/security/fraud-patterns',
  adminMiddleware,
  [
    query('pattern_type').optional().isIn(['referral_abuse', 'mining_exploit', 'account_farming']),
    query('severity').optional().isIn(['low', 'medium', 'high']),
    query('limit').optional().isInt({ min: 1, max: 50 })
  ],
  validationMiddleware,
  SecurityController.getFraudPatterns
);

// IP and device management
router.get('/security/ip-analysis',
  adminMiddleware,
  [
    query('ip_address').optional().isIP(),
    query('risk_level').optional().isIn(['low', 'medium', 'high', 'critical']),
    query('action_taken').optional().isIn(['none', 'warning', 'restricted', 'banned'])
  ],
  validationMiddleware,
  SecurityController.getIPAnalysis
);

router.patch('/security/ip/:ipAddress/status',
  adminMiddleware,
  [
    param('ipAddress').isIP().withMessage('Invalid IP address'),
    body('action').isIn(['whitelist', 'blacklist', 'restrict', 'monitor']),
    body('reason').isLength({ min: 10, max: 500 }),
    body('duration').optional().isInt({ min: 1, max: 8760 }) // max 1 year
  ],
  validationMiddleware,
  SecurityController.updateIPStatus
);

// =====================================
// SYSTEM MANAGEMENT ROUTES
// =====================================

// System health check
router.get('/system/health',
  adminMiddleware,
  SystemController.getSystemHealth
);

// System configuration
router.get('/system/config',
  superAdminMiddleware,
  SystemController.getSystemConfig
);

router.patch('/system/config',
  superAdminMiddleware,
  strictAdminRateLimit,
  [
    body('category').isIn(['mining', 'xp', 'rp', 'nft', 'security', 'general']),
    body('key').isLength({ min: 1, max: 100 }),
    body('value').exists().withMessage('Configuration value required'),
    body('environment').optional().isIn(['development', 'staging', 'production'])
  ],
  validationMiddleware,
  SystemController.updateSystemConfig
);

// Database maintenance
router.post('/system/maintenance/:action',
  superAdminMiddleware,
  strictAdminRateLimit,
  [
    param('action').isIn(['vacuum', 'reindex', 'analyze', 'cleanup']),
    body('confirm').equals('true').withMessage('Confirmation required for maintenance operations')
  ],
  validationMiddleware,
  SystemController.performMaintenance
);

// Feature flags management
router.get('/system/feature-flags',
  adminMiddleware,
  SystemController.getFeatureFlags
);

router.patch('/system/feature-flags/:flagName',
  superAdminMiddleware,
  [
    param('flagName').isLength({ min: 1, max: 50 }),
    body('enabled').isBoolean(),
    body('rollout_percentage').optional().isInt({ min: 0, max: 100 }),
    body('target_groups').optional().isArray()
  ],
  validationMiddleware,
  SystemController.updateFeatureFlag
);

// API rate limit management
router.get('/system/rate-limits',
  adminMiddleware,
  SystemController.getRateLimits
);

router.patch('/system/rate-limits',
  superAdminMiddleware,
  [
    body('endpoint').isLength({ min: 1, max: 100 }),
    body('requests_per_minute').isInt({ min: 1, max: 10000 }),
    body('burst_limit').optional().isInt({ min: 1, max: 1000 })
  ],
  validationMiddleware,
  SystemController.updateRateLimit
);

// =====================================
// CONTENT MODERATION ROUTES
// =====================================

// Get flagged content
router.get('/moderation/flagged-content',
  adminMiddleware,
  [
    query('platform').optional().isIn(['instagram', 'tiktok', 'youtube', 'facebook', 'twitter']),
    query('flag_reason').optional().isIn(['spam', 'inappropriate', 'copyright', 'fraud', 'ai_generated']),
    query('status').optional().isIn(['pending', 'approved', 'rejected']),
    query('severity').optional().isIn(['low', 'medium', 'high'])
  ],
  validationMiddleware,
  AdminController.getFlaggedContent
);

// Review flagged content
router.patch('/moderation/content/:contentId/review',
  adminMiddleware,
  [
    param('contentId').isUUID().withMessage('Invalid content ID'),
    body('action').isIn(['approve', 'reject', 'flag_for_review']),
    body('reason').isLength({ min: 10, max: 500 }),
    body('quality_score_override').optional().isFloat({ min: 0.1, max: 2.0 })
  ],
  validationMiddleware,
  AdminController.reviewFlaggedContent
);

// Content quality trends
router.get('/moderation/quality-trends',
  adminMiddleware,
  [
    query('period').optional().isIn(['7d', '30d', '90d']),
    query('platform').optional().isIn(['instagram', 'tiktok', 'youtube', 'facebook', 'twitter'])
  ],
  validationMiddleware,
  AdminController.getContentQualityTrends
);

// =====================================
// PARTNERSHIP & BUSINESS ROUTES
// =====================================

// Brand partnership management
router.get('/partnerships/brands',
  adminMiddleware,
  AdminController.getBrandPartnerships
);

router.post('/partnerships/brands',
  adminMiddleware,
  [
    body('brand_name').isLength({ min: 2, max: 100 }),
    body('contact_email').isEmail(),
    body('partnership_type').isIn(['advertising', 'content', 'event', 'integration']),
    body('revenue_share').isFloat({ min: 0, max: 0.5 }),
    body('contract_duration').isInt({ min: 30, max: 365 })
  ],
  validationMiddleware,
  AdminController.createBrandPartnership
);

// Campaign management
router.get('/campaigns',
  adminMiddleware,
  AdminController.getCampaigns
);

router.post('/campaigns',
  adminMiddleware,
  [
    body('name').isLength({ min: 3, max: 100 }),
    body('description').isLength({ min: 10, max: 1000 }),
    body('campaign_type').isIn(['mining_boost', 'xp_bonus', 'special_event', 'nft_drop']),
    body('start_date').isISO8601(),
    body('end_date').isISO8601(),
    body('target_audience').isObject(),
    body('reward_pool').isFloat({ min: 100, max: 1000000 })
  ],
  validationMiddleware,
  AdminController.createCampaign
);

// =====================================
// BACKUP & RECOVERY ROUTES
// =====================================

// Database backup status
router.get('/backup/status',
  superAdminMiddleware,
  SystemController.getBackupStatus
);

// Initiate manual backup
router.post('/backup/manual',
  superAdminMiddleware,
  strictAdminRateLimit,
  [
    body('backup_type').isIn(['full', 'incremental', 'user_data', 'blockchain_data']),
    body('compress').optional().isBoolean()
  ],
  validationMiddleware,
  SystemController.initiateBackup
);

// Recovery operations
router.post('/recovery/validate',
  superAdminMiddleware,
  [
    body('backup_id').isUUID(),
    body('recovery_type').isIn(['full', 'partial', 'test'])
  ],
  validationMiddleware,
  SystemController.validateRecovery
);

// =====================================
// AUDIT & COMPLIANCE ROUTES
// =====================================

// Audit logs
router.get('/audit/logs',
  adminMiddleware,
  [
    query('action_type').optional().isIn(['user_action', 'admin_action', 'system_event', 'security_event']),
    query('admin_id').optional().isUUID(),
    query('user_id').optional().isUUID(),
    query('date_from').optional().isISO8601(),
    query('date_to').optional().isISO8601(),
    query('limit').optional().isInt({ min: 1, max: 1000 })
  ],
  validationMiddleware,
  AdminController.getAuditLogs
);

// Compliance reports
router.get('/compliance/reports',
  superAdminMiddleware,
  [
    query('report_type').optional().isIn(['kyc_summary', 'transaction_summary', 'user_activity', 'financial']),
    query('period').optional().isIn(['monthly', 'quarterly', 'yearly'])
  ],
  validationMiddleware,
  AdminController.getComplianceReports
);

// Generate compliance export
router.post('/compliance/export',
  superAdminMiddleware,
  [
    body('report_type').isIn(['kyc_records', 'financial_transactions', 'user_data', 'security_incidents']),
    body('date_from').isISO8601(),
    body('date_to').isISO8601(),
    body('format').isIn(['csv', 'xlsx', 'pdf']),
    body('include_sensitive').optional().isBoolean()
  ],
  validationMiddleware,
  AdminController.exportComplianceData
);

// =====================================
// EMERGENCY OPERATIONS ROUTES
// =====================================

// Emergency system shutdown
router.post('/emergency/shutdown',
  superAdminMiddleware,
  strictAdminRateLimit,
  [
    body('reason').isLength({ min: 20, max: 1000 }),
    body('estimated_duration').isInt({ min: 5, max: 1440 }),
    body('affected_services').isArray({ min: 1 }),
    body('confirm_shutdown').equals('EMERGENCY_SHUTDOWN_CONFIRMED')
  ],
  validationMiddleware,
  SystemController.emergencyShutdown
);

// Emergency user actions
router.post('/emergency/users/:userId/freeze',
  adminMiddleware,
  strictAdminRateLimit,
  [
    param('userId').isUUID(),
    body('reason').isLength({ min: 20, max: 1000 }),
    body('freeze_assets').isBoolean(),
    body('notify_user').isBoolean()
  ],
  validationMiddleware,
  AdminController.emergencyFreezeUser
);

// Emergency mining halt
router.post('/emergency/mining/halt',
  superAdminMiddleware,
  strictAdminRateLimit,
  [
    body('reason').isLength({ min: 20, max: 1000 }),
    body('affected_users').optional().isArray(),
    body('halt_duration').isInt({ min: 1, max: 72 })
  ],
  validationMiddleware,
  MiningManagementController.emergencyMiningHalt
);

export default router;
