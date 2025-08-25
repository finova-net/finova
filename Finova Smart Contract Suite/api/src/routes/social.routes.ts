import { Router } from 'express';
import { 
  authMiddleware, 
  kycMiddleware, 
  rateLimitMiddleware, 
  validationMiddleware 
} from '../middleware';
import { socialController } from '../controllers/social.controller';
import { body, param, query } from 'express-validator';

const router = Router();

// Rate limits for social operations
const socialRateLimit = rateLimitMiddleware({
  windowMs: 15 * 60 * 1000, // 15 minutes
  max: 100, // 100 requests per window
  message: 'Too many social requests, try again later'
});

const postRateLimit = rateLimitMiddleware({
  windowMs: 60 * 60 * 1000, // 1 hour
  max: 20, // 20 posts per hour
  message: 'Post limit exceeded, try again later'
});

// Validation schemas
const connectPlatformValidation = [
  body('platform').isIn(['instagram', 'tiktok', 'youtube', 'facebook', 'twitter']),
  body('accessToken').isString().isLength({ min: 10 }),
  body('platformUserId').isString().isLength({ min: 1 }),
  body('platformUsername').isString().isLength({ min: 1, max: 100 })
];

const submitContentValidation = [
  body('platform').isIn(['instagram', 'tiktok', 'youtube', 'facebook', 'twitter']),
  body('contentType').isIn(['post', 'story', 'video', 'comment', 'like', 'share']),
  body('contentUrl').isURL(),
  body('contentText').optional().isString().isLength({ max: 5000 }),
  body('mediaUrls').optional().isArray({ max: 10 }),
  body('hashtags').optional().isArray({ max: 30 }),
  body('mentions').optional().isArray({ max: 20 })
];

const engagementValidation = [
  body('contentId').isString().isLength({ min: 1 }),
  body('engagementType').isIn(['like', 'comment', 'share', 'follow']),
  body('targetUserId').optional().isString(),
  body('commentText').optional().isString().isLength({ max: 1000 })
];

// SOCIAL PLATFORM INTEGRATION ROUTES

// Connect social media platform
router.post('/connect-platform', 
  authMiddleware,
  kycMiddleware,
  socialRateLimit,
  connectPlatformValidation,
  validationMiddleware,
  socialController.connectPlatform
);

// Disconnect platform
router.delete('/disconnect-platform/:platform',
  authMiddleware,
  param('platform').isIn(['instagram', 'tiktok', 'youtube', 'facebook', 'twitter']),
  validationMiddleware,
  socialController.disconnectPlatform
);

// Get connected platforms
router.get('/connected-platforms',
  authMiddleware,
  socialController.getConnectedPlatforms
);

// Verify platform connection
router.post('/verify-platform/:platform',
  authMiddleware,
  param('platform').isIn(['instagram', 'tiktok', 'youtube', 'facebook', 'twitter']),
  validationMiddleware,
  socialController.verifyPlatformConnection
);

// CONTENT SUBMISSION & XP EARNING

// Submit content for XP calculation
router.post('/submit-content',
  authMiddleware,
  kycMiddleware,
  postRateLimit,
  submitContentValidation,
  validationMiddleware,
  socialController.submitContent
);

// Bulk content sync (for catching up missed content)
router.post('/sync-content',
  authMiddleware,
  kycMiddleware,
  rateLimitMiddleware({ windowMs: 60 * 60 * 1000, max: 5 }),
  body('platform').isIn(['instagram', 'tiktok', 'youtube', 'facebook', 'twitter']),
  body('syncPeriod').isIn(['24h', '7d', '30d']),
  validationMiddleware,
  socialController.syncContent
);

// Submit engagement activity
router.post('/submit-engagement',
  authMiddleware,
  kycMiddleware,
  socialRateLimit,
  engagementValidation,
  validationMiddleware,
  socialController.submitEngagement
);

// Get content history and XP breakdown
router.get('/content-history',
  authMiddleware,
  query('platform').optional().isIn(['instagram', 'tiktok', 'youtube', 'facebook', 'twitter']),
  query('contentType').optional().isIn(['post', 'story', 'video', 'comment', 'like', 'share']),
  query('dateFrom').optional().isISO8601(),
  query('dateTo').optional().isISO8601(),
  query('page').optional().isInt({ min: 1 }),
  query('limit').optional().isInt({ min: 1, max: 100 }),
  validationMiddleware,
  socialController.getContentHistory
);

// VIRAL CONTENT & QUALITY ANALYSIS

// Check content viral status
router.get('/viral-content',
  authMiddleware,
  query('platform').optional().isIn(['instagram', 'tiktok', 'youtube', 'facebook', 'twitter']),
  query('minViews').optional().isInt({ min: 1000 }),
  query('dateFrom').optional().isISO8601(),
  query('dateTo').optional().isISO8601(),
  validationMiddleware,
  socialController.getViralContent
);

// Request content quality re-analysis
router.post('/reanalyze-content/:contentId',
  authMiddleware,
  rateLimitMiddleware({ windowMs: 60 * 60 * 1000, max: 10 }),
  param('contentId').isString(),
  validationMiddleware,
  socialController.reanalyzeContent
);

// Get quality score breakdown
router.get('/quality-analysis/:contentId',
  authMiddleware,
  param('contentId').isString(),
  validationMiddleware,
  socialController.getQualityAnalysis
);

// SOCIAL CHALLENGES & CAMPAIGNS

// Get available social challenges
router.get('/challenges',
  authMiddleware,
  query('platform').optional().isIn(['instagram', 'tiktok', 'youtube', 'facebook', 'twitter']),
  query('difficulty').optional().isIn(['easy', 'medium', 'hard', 'expert']),
  query('status').optional().isIn(['active', 'upcoming', 'completed']),
  validationMiddleware,
  socialController.getChallenges
);

// Join social challenge
router.post('/challenges/:challengeId/join',
  authMiddleware,
  kycMiddleware,
  param('challengeId').isString(),
  validationMiddleware,
  socialController.joinChallenge
);

// Submit challenge entry
router.post('/challenges/:challengeId/submit',
  authMiddleware,
  param('challengeId').isString(),
  body('contentUrl').isURL(),
  body('description').optional().isString().isLength({ max: 1000 }),
  validationMiddleware,
  socialController.submitChallengeEntry
);

// Get challenge leaderboard
router.get('/challenges/:challengeId/leaderboard',
  authMiddleware,
  param('challengeId').isString(),
  query('page').optional().isInt({ min: 1 }),
  query('limit').optional().isInt({ min: 1, max: 100 }),
  validationMiddleware,
  socialController.getChallengeLeaderboard
);

// SOCIAL ANALYTICS & INSIGHTS

// Get social analytics dashboard
router.get('/analytics/dashboard',
  authMiddleware,
  query('period').optional().isIn(['24h', '7d', '30d', '90d']),
  query('platforms').optional().custom((value) => {
    if (typeof value === 'string') {
      const platforms = value.split(',');
      return platforms.every(p => ['instagram', 'tiktok', 'youtube', 'facebook', 'twitter'].includes(p));
    }
    return false;
  }),
  validationMiddleware,
  socialController.getAnalyticsDashboard
);

// Get XP breakdown by platform/activity
router.get('/analytics/xp-breakdown',
  authMiddleware,
  query('period').optional().isIn(['24h', '7d', '30d', '90d']),
  query('groupBy').optional().isIn(['platform', 'contentType', 'date']),
  validationMiddleware,
  socialController.getXpBreakdown
);

// Get engagement trends
router.get('/analytics/engagement-trends',
  authMiddleware,
  query('period').optional().isIn(['7d', '30d', '90d']),
  query('platform').optional().isIn(['instagram', 'tiktok', 'youtube', 'facebook', 'twitter']),
  validationMiddleware,
  socialController.getEngagementTrends
);

// Get competitor analysis (premium feature)
router.get('/analytics/competitor-analysis',
  authMiddleware,
  kycMiddleware,
  rateLimitMiddleware({ windowMs: 60 * 60 * 1000, max: 10 }),
  query('competitors').isString(), // comma-separated usernames
  query('platform').isIn(['instagram', 'tiktok', 'youtube', 'facebook', 'twitter']),
  query('metrics').optional().isString(), // comma-separated metrics
  validationMiddleware,
  socialController.getCompetitorAnalysis
);

// SOCIAL FEED & DISCOVERY

// Get social feed from Finova users
router.get('/feed',
  authMiddleware,
  query('feedType').optional().isIn(['following', 'trending', 'recommended', 'guild']),
  query('platform').optional().isIn(['instagram', 'tiktok', 'youtube', 'facebook', 'twitter']),
  query('page').optional().isInt({ min: 1 }),
  query('limit').optional().isInt({ min: 1, max: 50 }),
  validationMiddleware,
  socialController.getSocialFeed
);

// Follow/Unfollow Finova user
router.post('/follow/:userId',
  authMiddleware,
  kycMiddleware,
  param('userId').isString(),
  validationMiddleware,
  socialController.followUser
);

router.delete('/follow/:userId',
  authMiddleware,
  param('userId').isString(),
  validationMiddleware,
  socialController.unfollowUser
);

// Get user's social profile
router.get('/profile/:userId',
  authMiddleware,
  param('userId').isString(),
  validationMiddleware,
  socialController.getUserSocialProfile
);

// CONTENT SCHEDULING & AUTOMATION

// Schedule content post
router.post('/schedule-content',
  authMiddleware,
  kycMiddleware,
  rateLimitMiddleware({ windowMs: 60 * 60 * 1000, max: 20 }),
  body('platform').isIn(['instagram', 'tiktok', 'youtube', 'facebook', 'twitter']),
  body('contentType').isIn(['post', 'story', 'video']),
  body('content').isString().isLength({ max: 5000 }),
  body('mediaUrls').optional().isArray({ max: 10 }),
  body('scheduledTime').isISO8601(),
  body('hashtags').optional().isArray({ max: 30 }),
  validationMiddleware,
  socialController.scheduleContent
);

// Get scheduled content
router.get('/scheduled-content',
  authMiddleware,
  query('platform').optional().isIn(['instagram', 'tiktok', 'youtube', 'facebook', 'twitter']),
  query('status').optional().isIn(['pending', 'posted', 'failed']),
  validationMiddleware,
  socialController.getScheduledContent
);

// Cancel scheduled content
router.delete('/scheduled-content/:scheduleId',
  authMiddleware,
  param('scheduleId').isString(),
  validationMiddleware,
  socialController.cancelScheduledContent
);

// SOCIAL AUTOMATION SETTINGS

// Get automation settings
router.get('/automation-settings',
  authMiddleware,
  socialController.getAutomationSettings
);

// Update automation settings
router.put('/automation-settings',
  authMiddleware,
  body('autoSync').optional().isBoolean(),
  body('autoHashtags').optional().isBoolean(),
  body('qualityFilters').optional().isObject(),
  body('notifications').optional().isObject(),
  validationMiddleware,
  socialController.updateAutomationSettings
);

// SOCIAL REWARDS & BONUSES

// Claim daily social bonus
router.post('/claim-daily-bonus',
  authMiddleware,
  kycMiddleware,
  rateLimitMiddleware({ windowMs: 24 * 60 * 60 * 1000, max: 1 }),
  socialController.claimDailyBonus
);

// Get available social rewards
router.get('/available-rewards',
  authMiddleware,
  query('category').optional().isIn(['daily', 'weekly', 'monthly', 'achievement']),
  validationMiddleware,
  socialController.getAvailableRewards
);

// Claim achievement reward
router.post('/claim-reward/:rewardId',
  authMiddleware,
  kycMiddleware,
  param('rewardId').isString(),
  validationMiddleware,
  socialController.claimReward
);

// SOCIAL MARKETPLACE INTEGRATION

// List user-generated content for sale
router.post('/marketplace/list-content',
  authMiddleware,
  kycMiddleware,
  body('contentId').isString(),
  body('price').isFloat({ min: 0 }),
  body('currency').isIn(['FIN', 'USDC', 'SOL']),
  body('licenseType').isIn(['single', 'commercial', 'exclusive']),
  body('description').optional().isString().isLength({ max: 1000 }),
  validationMiddleware,
  socialController.listContentForSale
);

// Purchase content license
router.post('/marketplace/purchase/:listingId',
  authMiddleware,
  kycMiddleware,
  param('listingId').isString(),
  body('paymentMethod').isIn(['FIN', 'USDC', 'SOL']),
  validationMiddleware,
  socialController.purchaseContentLicense
);

// Get marketplace listings
router.get('/marketplace/listings',
  authMiddleware,
  query('contentType').optional().isIn(['post', 'video', 'image']),
  query('platform').optional().isIn(['instagram', 'tiktok', 'youtube', 'facebook', 'twitter']),
  query('priceMin').optional().isFloat({ min: 0 }),
  query('priceMax').optional().isFloat({ min: 0 }),
  query('currency').optional().isIn(['FIN', 'USDC', 'SOL']),
  query('sortBy').optional().isIn(['price', 'date', 'popularity']),
  query('page').optional().isInt({ min: 1 }),
  query('limit').optional().isInt({ min: 1, max: 50 }),
  validationMiddleware,
  socialController.getMarketplaceListings
);

// SOCIAL REPUTATION & TRUST

// Get user reputation score
router.get('/reputation/:userId',
  authMiddleware,
  param('userId').isString(),
  validationMiddleware,
  socialController.getUserReputation
);

// Report inappropriate content
router.post('/report-content',
  authMiddleware,
  body('contentId').isString(),
  body('reason').isIn(['spam', 'inappropriate', 'plagiarism', 'fake', 'harassment']),
  body('description').optional().isString().isLength({ max: 1000 }),
  validationMiddleware,
  socialController.reportContent
);

// Vote on content quality (community moderation)
router.post('/vote-content-quality',
  authMiddleware,
  kycMiddleware,
  rateLimitMiddleware({ windowMs: 60 * 60 * 1000, max: 50 }),
  body('contentId').isString(),
  body('qualityScore').isInt({ min: 1, max: 5 }),
  body('feedback').optional().isString().isLength({ max: 500 }),
  validationMiddleware,
  socialController.voteContentQuality
);

// ADMIN & MODERATION ROUTES (restricted)

// Get flagged content (admin only)
router.get('/admin/flagged-content',
  authMiddleware,
  // adminMiddleware, // to be implemented
  query('severity').optional().isIn(['low', 'medium', 'high', 'critical']),
  query('status').optional().isIn(['pending', 'reviewed', 'resolved']),
  query('page').optional().isInt({ min: 1 }),
  query('limit').optional().isInt({ min: 1, max: 100 }),
  validationMiddleware,
  socialController.getFlaggedContent
);

// Review flagged content (admin only)
router.post('/admin/review-content/:contentId',
  authMiddleware,
  // adminMiddleware,
  param('contentId').isString(),
  body('action').isIn(['approve', 'reject', 'warn', 'ban']),
  body('reason').isString().isLength({ min: 1, max: 1000 }),
  validationMiddleware,
  socialController.reviewFlaggedContent
);

// Bulk content moderation (admin only)
router.post('/admin/bulk-moderate',
  authMiddleware,
  // adminMiddleware,
  rateLimitMiddleware({ windowMs: 60 * 60 * 1000, max: 10 }),
  body('contentIds').isArray({ min: 1, max: 100 }),
  body('action').isIn(['approve', 'reject', 'warn']),
  body('reason').isString(),
  validationMiddleware,
  socialController.bulkModerateContent
);

export default router;
