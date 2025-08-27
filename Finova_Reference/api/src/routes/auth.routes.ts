import { Router } from 'express';
import rateLimit from 'express-rate-limit';
import { body, param, query } from 'express-validator';
import { AuthController } from '../controllers/auth.controller';
import { authMiddleware } from '../middleware/auth.middleware';
import { kycMiddleware } from '../middleware/kyc.middleware';
import { validationMiddleware } from '../middleware/validation.middleware';
import { rateLimitMiddleware } from '../middleware/rate-limit.middleware';

const router = Router();
const authController = new AuthController();

// Rate limiting configurations
const strictRateLimit = rateLimit({
  windowMs: 15 * 60 * 1000, // 15 minutes
  max: 5, // 5 attempts per window
  message: { error: 'Too many attempts, please try again later' },
  standardHeaders: true,
  legacyHeaders: false
});

const moderateRateLimit = rateLimit({
  windowMs: 5 * 60 * 1000, // 5 minutes
  max: 10, // 10 attempts per window
  message: { error: 'Rate limit exceeded' },
  standardHeaders: true,
  legacyHeaders: false
});

const generalRateLimit = rateLimit({
  windowMs: 1 * 60 * 1000, // 1 minute
  max: 30, // 30 requests per minute
  message: { error: 'Too many requests' },
  standardHeaders: true,
  legacyHeaders: false
});

// Validation schemas
const registerValidation = [
  body('email')
    .isEmail()
    .normalizeEmail()
    .withMessage('Valid email is required'),
  body('password')
    .isLength({ min: 8, max: 128 })
    .matches(/^(?=.*[a-z])(?=.*[A-Z])(?=.*\d)(?=.*[@$!%*?&])[A-Za-z\d@$!%*?&]/)
    .withMessage('Password must contain at least 8 characters with uppercase, lowercase, number and special character'),
  body('username')
    .isLength({ min: 3, max: 30 })
    .matches(/^[a-zA-Z0-9_-]+$/)
    .withMessage('Username must be 3-30 characters, alphanumeric, underscore or dash only'),
  body('referralCode')
    .optional()
    .isLength({ min: 6, max: 12 })
    .matches(/^[A-Z0-9]+$/)
    .withMessage('Invalid referral code format'),
  body('deviceFingerprint')
    .isLength({ min: 32, max: 256 })
    .withMessage('Device fingerprint required'),
  body('acceptTerms')
    .equals('true')
    .withMessage('Terms acceptance required'),
  body('acceptPrivacy')
    .equals('true')
    .withMessage('Privacy policy acceptance required')
];

const loginValidation = [
  body('identifier')
    .isLength({ min: 3, max: 254 })
    .withMessage('Email or username required'),
  body('password')
    .isLength({ min: 1, max: 128 })
    .withMessage('Password required'),
  body('deviceFingerprint')
    .isLength({ min: 32, max: 256 })
    .withMessage('Device fingerprint required'),
  body('rememberMe')
    .optional()
    .isBoolean()
    .withMessage('Remember me must be boolean')
];

const kycValidation = [
  body('personalInfo.firstName')
    .isLength({ min: 1, max: 50 })
    .matches(/^[a-zA-Z\s-']+$/)
    .withMessage('Valid first name required'),
  body('personalInfo.lastName')
    .isLength({ min: 1, max: 50 })
    .matches(/^[a-zA-Z\s-']+$/)
    .withMessage('Valid last name required'),
  body('personalInfo.dateOfBirth')
    .isISO8601()
    .withMessage('Valid date of birth required'),
  body('personalInfo.nationality')
    .isLength({ min: 2, max: 3 })
    .matches(/^[A-Z]+$/)
    .withMessage('Valid nationality code required'),
  body('personalInfo.phoneNumber')
    .matches(/^\+[1-9]\d{1,14}$/)
    .withMessage('Valid international phone number required'),
  body('documents.idType')
    .isIn(['passport', 'national_id', 'drivers_license'])
    .withMessage('Valid ID type required'),
  body('documents.idNumber')
    .isLength({ min: 5, max: 30 })
    .matches(/^[A-Z0-9]+$/)
    .withMessage('Valid ID number required'),
  body('documents.frontImage')
    .isBase64()
    .withMessage('Valid front image required'),
  body('documents.backImage')
    .optional()
    .isBase64()
    .withMessage('Valid back image required'),
  body('biometric.selfieImage')
    .isBase64()
    .withMessage('Valid selfie required'),
  body('biometric.livenessData')
    .isObject()
    .withMessage('Liveness data required'),
  body('address.street')
    .isLength({ min: 5, max: 100 })
    .withMessage('Valid street address required'),
  body('address.city')
    .isLength({ min: 2, max: 50 })
    .withMessage('Valid city required'),
  body('address.state')
    .isLength({ min: 2, max: 50 })
    .withMessage('Valid state/province required'),
  body('address.postalCode')
    .isLength({ min: 3, max: 10 })
    .withMessage('Valid postal code required'),
  body('address.country')
    .isLength({ min: 2, max: 3 })
    .matches(/^[A-Z]+$/)
    .withMessage('Valid country code required')
];

const mfaValidation = [
  body('method')
    .isIn(['sms', 'email', 'totp', 'biometric'])
    .withMessage('Valid MFA method required'),
  body('code')
    .optional()
    .isLength({ min: 4, max: 8 })
    .matches(/^\d+$/)
    .withMessage('Valid verification code required'),
  body('biometricData')
    .optional()
    .isBase64()
    .withMessage('Valid biometric data required')
];

// Auth Routes

/**
 * @route   POST /api/auth/register
 * @desc    Register new user with advanced validation
 * @access  Public
 */
router.post('/register',
  strictRateLimit,
  registerValidation,
  validationMiddleware,
  authController.register
);

/**
 * @route   POST /api/auth/login
 * @desc    User login with device fingerprinting
 * @access  Public
 */
router.post('/login',
  strictRateLimit,
  loginValidation,
  validationMiddleware,
  authController.login
);

/**
 * @route   POST /api/auth/logout
 * @desc    Logout user and invalidate tokens
 * @access  Private
 */
router.post('/logout',
  generalRateLimit,
  authMiddleware,
  authController.logout
);

/**
 * @route   POST /api/auth/refresh
 * @desc    Refresh JWT tokens
 * @access  Public (requires refresh token)
 */
router.post('/refresh',
  moderateRateLimit,
  body('refreshToken').isJWT().withMessage('Valid refresh token required'),
  validationMiddleware,
  authController.refreshTokens
);

/**
 * @route   POST /api/auth/verify-email
 * @desc    Verify email with token
 * @access  Public
 */
router.post('/verify-email',
  moderateRateLimit,
  body('token').isLength({ min: 32, max: 256 }).withMessage('Valid verification token required'),
  validationMiddleware,
  authController.verifyEmail
);

/**
 * @route   POST /api/auth/resend-verification
 * @desc    Resend email verification
 * @access  Private
 */
router.post('/resend-verification',
  moderateRateLimit,
  authMiddleware,
  authController.resendEmailVerification
);

/**
 * @route   POST /api/auth/forgot-password
 * @desc    Request password reset
 * @access  Public
 */
router.post('/forgot-password',
  moderateRateLimit,
  body('email').isEmail().normalizeEmail().withMessage('Valid email required'),
  validationMiddleware,
  authController.forgotPassword
);

/**
 * @route   POST /api/auth/reset-password
 * @desc    Reset password with token
 * @access  Public
 */
router.post('/reset-password',
  strictRateLimit,
  body('token').isLength({ min: 32, max: 256 }).withMessage('Valid reset token required'),
  body('password')
    .isLength({ min: 8, max: 128 })
    .matches(/^(?=.*[a-z])(?=.*[A-Z])(?=.*\d)(?=.*[@$!%*?&])[A-Za-z\d@$!%*?&]/)
    .withMessage('Password must meet security requirements'),
  validationMiddleware,
  authController.resetPassword
);

/**
 * @route   POST /api/auth/change-password
 * @desc    Change password for authenticated user
 * @access  Private
 */
router.post('/change-password',
  moderateRateLimit,
  authMiddleware,
  body('currentPassword').isLength({ min: 1, max: 128 }).withMessage('Current password required'),
  body('newPassword')
    .isLength({ min: 8, max: 128 })
    .matches(/^(?=.*[a-z])(?=.*[A-Z])(?=.*\d)(?=.*[@$!%*?&])[A-Za-z\d@$!%*?&]/)
    .withMessage('New password must meet security requirements'),
  validationMiddleware,
  authController.changePassword
);

// KYC Routes

/**
 * @route   POST /api/auth/kyc/submit
 * @desc    Submit KYC verification documents
 * @access  Private
 */
router.post('/kyc/submit',
  moderateRateLimit,
  authMiddleware,
  kycValidation,
  validationMiddleware,
  authController.submitKYC
);

/**
 * @route   GET /api/auth/kyc/status
 * @desc    Get KYC verification status
 * @access  Private
 */
router.get('/kyc/status',
  generalRateLimit,
  authMiddleware,
  authController.getKYCStatus
);

/**
 * @route   POST /api/auth/kyc/resubmit
 * @desc    Resubmit KYC after rejection
 * @access  Private
 */
router.post('/kyc/resubmit',
  moderateRateLimit,
  authMiddleware,
  kycMiddleware.checkResubmissionEligibility,
  kycValidation,
  validationMiddleware,
  authController.resubmitKYC
);

/**
 * @route   GET /api/auth/kyc/requirements/:country
 * @desc    Get KYC requirements for specific country
 * @access  Public
 */
router.get('/kyc/requirements/:country',
  generalRateLimit,
  param('country').isLength({ min: 2, max: 3 }).matches(/^[A-Z]+$/).withMessage('Valid country code required'),
  validationMiddleware,
  authController.getKYCRequirements
);

// MFA Routes

/**
 * @route   POST /api/auth/mfa/setup
 * @desc    Setup multi-factor authentication
 * @access  Private
 */
router.post('/mfa/setup',
  moderateRateLimit,
  authMiddleware,
  body('method').isIn(['sms', 'email', 'totp', 'biometric']).withMessage('Valid MFA method required'),
  body('phoneNumber').optional().matches(/^\+[1-9]\d{1,14}$/).withMessage('Valid phone number required for SMS'),
  validationMiddleware,
  authController.setupMFA
);

/**
 * @route   POST /api/auth/mfa/verify
 * @desc    Verify MFA during login
 * @access  Public (requires MFA token)
 */
router.post('/mfa/verify',
  strictRateLimit,
  body('mfaToken').isJWT().withMessage('Valid MFA token required'),
  mfaValidation,
  validationMiddleware,
  authController.verifyMFA
);

/**
 * @route   POST /api/auth/mfa/disable
 * @desc    Disable MFA (requires password confirmation)
 * @access  Private
 */
router.post('/mfa/disable',
  moderateRateLimit,
  authMiddleware,
  body('password').isLength({ min: 1, max: 128 }).withMessage('Password required'),
  body('method').isIn(['sms', 'email', 'totp', 'biometric']).withMessage('Valid MFA method required'),
  validationMiddleware,
  authController.disableMFA
);

/**
 * @route   GET /api/auth/mfa/backup-codes
 * @desc    Generate backup codes for MFA
 * @access  Private
 */
router.get('/mfa/backup-codes',
  moderateRateLimit,
  authMiddleware,
  authController.generateBackupCodes
);

/**
 * @route   POST /api/auth/mfa/backup-codes/verify
 * @desc    Verify backup code
 * @access  Public (requires MFA token)
 */
router.post('/mfa/backup-codes/verify',
  strictRateLimit,
  body('mfaToken').isJWT().withMessage('Valid MFA token required'),
  body('backupCode').isLength({ min: 8, max: 12 }).withMessage('Valid backup code required'),
  validationMiddleware,
  authController.verifyBackupCode
);

// Biometric Authentication Routes

/**
 * @route   POST /api/auth/biometric/register
 * @desc    Register biometric authentication
 * @access  Private
 */
router.post('/biometric/register',
  moderateRateLimit,
  authMiddleware,
  body('biometricType').isIn(['fingerprint', 'face', 'voice']).withMessage('Valid biometric type required'),
  body('biometricData').isBase64().withMessage('Valid biometric data required'),
  body('deviceInfo').isObject().withMessage('Device info required'),
  validationMiddleware,
  authController.registerBiometric
);

/**
 * @route   POST /api/auth/biometric/authenticate
 * @desc    Authenticate using biometrics
 * @access  Public
 */
router.post('/biometric/authenticate',
  strictRateLimit,
  body('userId').isUUID().withMessage('Valid user ID required'),
  body('biometricType').isIn(['fingerprint', 'face', 'voice']).withMessage('Valid biometric type required'),
  body('biometricData').isBase64().withMessage('Valid biometric data required'),
  body('deviceFingerprint').isLength({ min: 32, max: 256 }).withMessage('Device fingerprint required'),
  validationMiddleware,
  authController.biometricAuthenticate
);

/**
 * @route   DELETE /api/auth/biometric/:type
 * @desc    Remove biometric authentication
 * @access  Private
 */
router.delete('/biometric/:type',
  moderateRateLimit,
  authMiddleware,
  param('type').isIn(['fingerprint', 'face', 'voice']).withMessage('Valid biometric type required'),
  body('password').isLength({ min: 1, max: 128 }).withMessage('Password required for removal'),
  validationMiddleware,
  authController.removeBiometric
);

// Session Management Routes

/**
 * @route   GET /api/auth/sessions
 * @desc    Get all active sessions
 * @access  Private
 */
router.get('/sessions',
  generalRateLimit,
  authMiddleware,
  authController.getActiveSessions
);

/**
 * @route   DELETE /api/auth/sessions/:sessionId
 * @desc    Revoke specific session
 * @access  Private
 */
router.delete('/sessions/:sessionId',
  moderateRateLimit,
  authMiddleware,
  param('sessionId').isUUID().withMessage('Valid session ID required'),
  validationMiddleware,
  authController.revokeSession
);

/**
 * @route   DELETE /api/auth/sessions/all
 * @desc    Revoke all sessions except current
 * @access  Private
 */
router.delete('/sessions/all',
  moderateRateLimit,
  authMiddleware,
  body('password').isLength({ min: 1, max: 128 }).withMessage('Password required'),
  validationMiddleware,
  authController.revokeAllSessions
);

// Security Routes

/**
 * @route   GET /api/auth/security/login-history
 * @desc    Get login history and security events
 * @access  Private
 */
router.get('/security/login-history',
  generalRateLimit,
  authMiddleware,
  query('limit').optional().isInt({ min: 1, max: 100 }).withMessage('Limit must be 1-100'),
  query('offset').optional().isInt({ min: 0 }).withMessage('Offset must be non-negative'),
  validationMiddleware,
  authController.getLoginHistory
);

/**
 * @route   POST /api/auth/security/report-suspicious
 * @desc    Report suspicious activity
 * @access  Private
 */
router.post('/security/report-suspicious',
  moderateRateLimit,
  authMiddleware,
  body('activityType').isIn(['unauthorized_login', 'unusual_location', 'suspicious_behavior', 'other']).withMessage('Valid activity type required'),
  body('description').isLength({ min: 10, max: 500 }).withMessage('Description required (10-500 characters)'),
  body('timestamp').isISO8601().withMessage('Valid timestamp required'),
  validationMiddleware,
  authController.reportSuspiciousActivity
);

/**
 * @route   GET /api/auth/security/trust-score
 * @desc    Get user trust score and security metrics
 * @access  Private
 */
router.get('/security/trust-score',
  generalRateLimit,
  authMiddleware,
  authController.getTrustScore
);

/**
 * @route   POST /api/auth/security/lock-account
 * @desc    Temporarily lock account for security
 * @access  Private
 */
router.post('/security/lock-account',
  strictRateLimit,
  authMiddleware,
  body('reason').isIn(['suspicious_activity', 'user_request', 'security_breach']).withMessage('Valid lock reason required'),
  body('password').isLength({ min: 1, max: 128 }).withMessage('Password required'),
  validationMiddleware,
  authController.lockAccount
);

// Admin Routes (for support/admin access)

/**
 * @route   GET /api/auth/admin/users/:userId/profile
 * @desc    Get user profile for admin review
 * @access  Admin
 */
router.get('/admin/users/:userId/profile',
  generalRateLimit,
  authMiddleware,
  param('userId').isUUID().withMessage('Valid user ID required'),
  validationMiddleware,
  authController.getUserProfileForAdmin
);

/**
 * @route   POST /api/auth/admin/users/:userId/actions
 * @desc    Perform admin actions on user account
 * @access  Admin
 */
router.post('/admin/users/:userId/actions',
  moderateRateLimit,
  authMiddleware,
  param('userId').isUUID().withMessage('Valid user ID required'),
  body('action').isIn(['suspend', 'unsuspend', 'ban', 'unban', 'force_kyc_review', 'reset_mfa']).withMessage('Valid action required'),
  body('reason').isLength({ min: 10, max: 500 }).withMessage('Reason required (10-500 characters)'),
  validationMiddleware,
  authController.performAdminAction
);

// Profile Management Routes

/**
 * @route   GET /api/auth/profile
 * @desc    Get user profile
 * @access  Private
 */
router.get('/profile',
  generalRateLimit,
  authMiddleware,
  authController.getProfile
);

/**
 * @route   PUT /api/auth/profile
 * @desc    Update user profile
 * @access  Private
 */
router.put('/profile',
  moderateRateLimit,
  authMiddleware,
  body('username').optional().isLength({ min: 3, max: 30 }).matches(/^[a-zA-Z0-9_-]+$/),
  body('email').optional().isEmail().normalizeEmail(),
  body('displayName').optional().isLength({ min: 1, max: 50 }),
  body('bio').optional().isLength({ max: 500 }),
  body('avatar').optional().isURL(),
  body('timezone').optional().isIn(require('moment-timezone').tz.names()),
  body('language').optional().isIn(['en', 'id', 'zh', 'ja', 'ko', 'th', 'vi', 'ms']),
  validationMiddleware,
  authController.updateProfile
);

/**
 * @route   POST /api/auth/profile/deactivate
 * @desc    Deactivate user account
 * @access  Private
 */
router.post('/profile/deactivate',
  strictRateLimit,
  authMiddleware,
  body('password').isLength({ min: 1, max: 128 }).withMessage('Password required'),
  body('reason').optional().isLength({ max: 500 }),
  validationMiddleware,
  authController.deactivateAccount
);

export default router;
