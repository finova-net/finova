import { Request, Response, NextFunction } from 'express';
import bcrypt from 'bcryptjs';
import jwt from 'jsonwebtoken';
import { v4 as uuidv4 } from 'uuid';
import speakeasy from 'speakeasy';
import QRCode from 'qrcode';
import { User, KYCData, DeviceFingerprint, BiometricData } from '../models/User.model';
import { userService } from '../services/user.service';
import { authService } from '../services/auth.service';
import { antiBotService } from '../services/anti-bot.service';
import { blockchainService } from '../services/blockchain.service';
import { notificationService } from '../services/notification.service';
import { kycService } from '../services/kyc.service';
import { logger } from '../utils/logger';
import { ApiResponse, AuthTokens, RegisterRequest, LoginRequest } from '../types/api.types';
import { validateEmail, validatePassword, validateReferralCode } from '../utils/validation';
import { encryptSensitiveData, decryptSensitiveData } from '../utils/encryption';
import { calculateHumanProbability } from '../utils/biometric-analysis';
import { generateDeviceFingerprint } from '../utils/device-fingerprint';

export class AuthController {
  /**
   * User Registration with Multi-Layer Security
   * Implements KYC, biometric verification, and anti-bot measures
   */
  public async register(req: Request, res: Response, next: NextFunction): Promise<void> {
    try {
      const {
        email,
        password,
        confirmPassword,
        referralCode,
        firstName,
        lastName,
        country,
        phoneNumber,
        biometricData,
        deviceInfo,
        socialProfiles,
        acceptedTerms,
        acceptedPrivacy
      }: RegisterRequest = req.body;

      // Input validation
      if (!validateEmail(email)) {
        res.status(400).json({
          success: false,
          message: 'Invalid email format',
          error: 'INVALID_EMAIL'
        });
        return;
      }

      if (!validatePassword(password)) {
        res.status(400).json({
          success: false,
          message: 'Password must be at least 8 characters with uppercase, lowercase, number and special character',
          error: 'WEAK_PASSWORD'
        });
        return;
      }

      if (password !== confirmPassword) {
        res.status(400).json({
          success: false,
          message: 'Passwords do not match',
          error: 'PASSWORD_MISMATCH'
        });
        return;
      }

      if (!acceptedTerms || !acceptedPrivacy) {
        res.status(400).json({
          success: false,
          message: 'Must accept terms and privacy policy',
          error: 'TERMS_NOT_ACCEPTED'
        });
        return;
      }

      // Check if user already exists
      const existingUser = await userService.findByEmail(email);
      if (existingUser) {
        res.status(409).json({
          success: false,
          message: 'Email already registered',
          error: 'EMAIL_EXISTS'
        });
        return;
      }

      // Anti-bot verification
      const deviceFingerprint = generateDeviceFingerprint(deviceInfo, req);
      const humanProbability = await calculateHumanProbability({
        biometricData,
        deviceFingerprint,
        behavioralPatterns: req.body.behavioralData,
        ipAddress: req.ip,
        userAgent: req.headers['user-agent']
      });

      if (humanProbability < 0.7) {
        logger.warn('Potential bot registration attempt', {
          email: email.replace(/(.{2}).*(@.*)/, '$1***$2'),
          humanProbability,
          ip: req.ip,
          userAgent: req.headers['user-agent']
        });

        res.status(403).json({
          success: false,
          message: 'Registration failed security validation',
          error: 'SECURITY_VALIDATION_FAILED'
        });
        return;
      }

      // Validate referral code if provided
      let referrer = null;
      if (referralCode) {
        if (!validateReferralCode(referralCode)) {
          res.status(400).json({
            success: false,
            message: 'Invalid referral code format',
            error: 'INVALID_REFERRAL_CODE'
          });
          return;
        }

        referrer = await userService.findByReferralCode(referralCode);
        if (!referrer) {
          res.status(400).json({
            success: false,
            message: 'Referral code not found',
            error: 'REFERRAL_NOT_FOUND'
          });
          return;
        }
      }

      // Hash password
      const saltRounds = 12;
      const hashedPassword = await bcrypt.hash(password, saltRounds);

      // Generate 2FA secret
      const twoFactorSecret = speakeasy.generateSecret({
        name: `Finova Network (${email})`,
        issuer: 'Finova Network'
      });

      // Create user with encrypted sensitive data
      const userId = uuidv4();
      const userReferralCode = await userService.generateUniqueReferralCode();

      const userData = {
        id: userId,
        email,
        password: hashedPassword,
        firstName: encryptSensitiveData(firstName),
        lastName: encryptSensitiveData(lastName),
        country,
        phoneNumber: phoneNumber ? encryptSensitiveData(phoneNumber) : null,
        referralCode: userReferralCode,
        referredBy: referrer?.id || null,
        twoFactorSecret: encryptSensitiveData(twoFactorSecret.base32),
        humanProbability,
        deviceFingerprint: encryptSensitiveData(JSON.stringify(deviceFingerprint)),
        socialProfiles: socialProfiles ? encryptSensitiveData(JSON.stringify(socialProfiles)) : null,
        acceptedTermsAt: new Date(),
        acceptedPrivacyAt: new Date(),
        registrationIp: req.ip,
        isEmailVerified: false,
        isPhoneVerified: false,
        isKYCVerified: false,
        isTwoFactorEnabled: false,
        accountStatus: 'PENDING_VERIFICATION',
        createdAt: new Date(),
        updatedAt: new Date()
      };

      // Store biometric data securely
      if (biometricData) {
        const encryptedBiometric = {
          userId,
          faceTemplate: encryptSensitiveData(JSON.stringify(biometricData.faceTemplate)),
          voicePrint: biometricData.voicePrint ? encryptSensitiveData(JSON.stringify(biometricData.voicePrint)) : null,
          fingerprintHash: biometricData.fingerprint ? encryptSensitiveData(biometricData.fingerprint) : null,
          createdAt: new Date()
        };
        await userService.storeBiometricData(encryptedBiometric);
      }

      // Create user in database
      const newUser = await userService.create(userData);

      // Create blockchain wallet
      const walletInfo = await blockchainService.createUserWallet(userId);
      await userService.updateWalletInfo(userId, walletInfo);

      // Process referral bonus if applicable
      if (referrer) {
        await authService.processReferralRegistration(referrer.id, userId);
        
        // Award referral points to referrer
        await userService.addReferralPoints(referrer.id, 50, 'REGISTRATION');
        
        // Log referral activity
        logger.info('Referral registration processed', {
          referrerId: referrer.id,
          newUserId: userId,
          referralCode
        });
      }

      // Send verification email
      const emailVerificationToken = await authService.generateEmailVerificationToken(userId);
      await notificationService.sendEmailVerification(email, emailVerificationToken);

      // Generate 2FA QR code
      const qrCodeUrl = await QRCode.toDataURL(twoFactorSecret.otpauth_url);

      // Generate initial JWT tokens for immediate app access
      const tokens = await authService.generateTokens(newUser);

      // Log successful registration
      logger.info('User registered successfully', {
        userId,
        email: email.replace(/(.{2}).*(@.*)/, '$1***$2'),
        referralCode: userReferralCode,
        hasReferrer: !!referrer,
        humanProbability
      });

      // Prepare response (exclude sensitive data)
      const response: ApiResponse = {
        success: true,
        message: 'Registration successful. Please verify your email to activate mining.',
        data: {
          user: {
            id: newUser.id,
            email: newUser.email,
            firstName: decryptSensitiveData(newUser.firstName),
            lastName: decryptSensitiveData(newUser.lastName),
            country: newUser.country,
            referralCode: newUser.referralCode,
            accountStatus: newUser.accountStatus,
            isEmailVerified: newUser.isEmailVerified,
            createdAt: newUser.createdAt,
            walletAddress: walletInfo.publicKey
          },
          tokens,
          twoFactor: {
            secret: twoFactorSecret.base32,
            qrCode: qrCodeUrl,
            backupCodes: await authService.generateBackupCodes(userId)
          }
        }
      };

      res.status(201).json(response);

    } catch (error) {
      logger.error('Registration error:', error);
      next(error);
    }
  }

  /**
   * User Login with Multi-Factor Authentication
   */
  public async login(req: Request, res: Response, next: NextFunction): Promise<void> {
    try {
      const {
        email,
        password,
        twoFactorCode,
        biometricVerification,
        deviceInfo,
        rememberDevice
      }: LoginRequest = req.body;

      // Input validation
      if (!validateEmail(email)) {
        res.status(400).json({
          success: false,
          message: 'Invalid email format',
          error: 'INVALID_EMAIL'
        });
        return;
      }

      // Find user
      const user = await userService.findByEmail(email);
      if (!user) {
        // Rate limiting for failed attempts
        await antiBotService.recordFailedLogin(req.ip, email);
        
        res.status(401).json({
          success: false,
          message: 'Invalid credentials',
          error: 'INVALID_CREDENTIALS'
        });
        return;
      }

      // Check account status
      if (user.accountStatus === 'SUSPENDED') {
        res.status(403).json({
          success: false,
          message: 'Account suspended. Contact support.',
          error: 'ACCOUNT_SUSPENDED'
        });
        return;
      }

      // Verify password
      const isPasswordValid = await bcrypt.compare(password, user.password);
      if (!isPasswordValid) {
        await antiBotService.recordFailedLogin(req.ip, email);
        
        res.status(401).json({
          success: false,
          message: 'Invalid credentials',
          error: 'INVALID_CREDENTIALS'
        });
        return;
      }

      // Device fingerprint verification
      const currentDeviceFingerprint = generateDeviceFingerprint(deviceInfo, req);
      const storedFingerprint = JSON.parse(decryptSensitiveData(user.deviceFingerprint));
      
      const deviceTrusted = await authService.isDeviceTrusted(user.id, currentDeviceFingerprint);

      // 2FA verification (if enabled)
      if (user.isTwoFactorEnabled) {
        if (!twoFactorCode) {
          res.status(200).json({
            success: false,
            message: '2FA code required',
            error: 'TWO_FACTOR_REQUIRED',
            data: { requiresTwoFactor: true }
          });
          return;
        }

        const twoFactorSecret = decryptSensitiveData(user.twoFactorSecret);
        const isValidTwoFactor = speakeasy.totp.verify({
          secret: twoFactorSecret,
          encoding: 'base32',
          token: twoFactorCode,
          window: 2
        });

        if (!isValidTwoFactor) {
          res.status(401).json({
            success: false,
            message: 'Invalid 2FA code',
            error: 'INVALID_TWO_FACTOR'
          });
          return;
        }
      }

      // Biometric verification for high-security accounts
      if (user.requiresBiometricAuth && biometricVerification) {
        const storedBiometric = await userService.getBiometricData(user.id);
        if (storedBiometric) {
          const biometricValid = await authService.verifyBiometric(
            biometricVerification,
            storedBiometric
          );
          
          if (!biometricValid) {
            res.status(401).json({
              success: false,
              message: 'Biometric verification failed',
              error: 'BIOMETRIC_VERIFICATION_FAILED'
            });
            return;
          }
        }
      }

      // Generate new device fingerprint if not trusted
      if (!deviceTrusted && !rememberDevice) {
        // Send device verification notification
        await notificationService.sendDeviceVerification(user.email, {
          device: currentDeviceFingerprint.device,
          location: currentDeviceFingerprint.location,
          timestamp: new Date()
        });
      }

      // Update last login and device info
      await userService.updateLastLogin(user.id, {
        lastLoginAt: new Date(),
        lastLoginIp: req.ip,
        lastLoginDevice: currentDeviceFingerprint.device,
        deviceFingerprint: rememberDevice ? 
          encryptSensitiveData(JSON.stringify(currentDeviceFingerprint)) : 
          user.deviceFingerprint
      });

      // Generate JWT tokens
      const tokens = await authService.generateTokens(user);

      // Store refresh token
      await authService.storeRefreshToken(user.id, tokens.refreshToken);

      // Log successful login
      logger.info('User logged in successfully', {
        userId: user.id,
        email: email.replace(/(.{2}).*(@.*)/, '$1***$2'),
        deviceTrusted,
        twoFactorUsed: user.isTwoFactorEnabled
      });

      // Get user mining status
      const miningStatus = await userService.getMiningStatus(user.id);
      
      // Get user stats for dashboard
      const userStats = await userService.getUserStats(user.id);

      const response: ApiResponse = {
        success: true,
        message: 'Login successful',
        data: {
          user: {
            id: user.id,
            email: user.email,
            firstName: decryptSensitiveData(user.firstName),
            lastName: decryptSensitiveData(user.lastName),
            country: user.country,
            referralCode: user.referralCode,
            accountStatus: user.accountStatus,
            isEmailVerified: user.isEmailVerified,
            isKYCVerified: user.isKYCVerified,
            currentLevel: user.currentLevel,
            totalXP: user.totalXP,
            totalRP: user.totalRP,
            walletAddress: user.walletAddress,
            lastLoginAt: user.lastLoginAt
          },
          tokens,
          mining: miningStatus,
          stats: userStats,
          deviceTrusted
        }
      };

      res.status(200).json(response);

    } catch (error) {
      logger.error('Login error:', error);
      next(error);
    }
  }

  /**
   * Refresh JWT Access Token
   */
  public async refreshToken(req: Request, res: Response, next: NextFunction): Promise<void> {
    try {
      const { refreshToken } = req.body;

      if (!refreshToken) {
        res.status(400).json({
          success: false,
          message: 'Refresh token required',
          error: 'REFRESH_TOKEN_REQUIRED'
        });
        return;
      }

      // Verify refresh token
      const decoded = jwt.verify(refreshToken, process.env.JWT_REFRESH_SECRET!) as any;
      const userId = decoded.userId;

      // Check if refresh token exists in database
      const storedToken = await authService.getRefreshToken(userId, refreshToken);
      if (!storedToken || storedToken.expiresAt < new Date()) {
        res.status(401).json({
          success: false,
          message: 'Invalid or expired refresh token',
          error: 'INVALID_REFRESH_TOKEN'
        });
        return;
      }

      // Get user
      const user = await userService.findById(userId);
      if (!user || user.accountStatus === 'SUSPENDED') {
        res.status(401).json({
          success: false,
          message: 'User not found or suspended',
          error: 'USER_NOT_FOUND'
        });
        return;
      }

      // Generate new tokens
      const newTokens = await authService.generateTokens(user);

      // Update refresh token in database
      await authService.updateRefreshToken(userId, refreshToken, newTokens.refreshToken);

      res.status(200).json({
        success: true,
        message: 'Token refreshed successfully',
        data: { tokens: newTokens }
      });

    } catch (error) {
      if (error.name === 'TokenExpiredError' || error.name === 'JsonWebTokenError') {
        res.status(401).json({
          success: false,
          message: 'Invalid refresh token',
          error: 'INVALID_REFRESH_TOKEN'
        });
        return;
      }
      
      logger.error('Token refresh error:', error);
      next(error);
    }
  }

  /**
   * User Logout
   */
  public async logout(req: Request, res: Response, next: NextFunction): Promise<void> {
    try {
      const { refreshToken } = req.body;
      const userId = req.user?.id;

      if (refreshToken && userId) {
        // Remove refresh token from database
        await authService.removeRefreshToken(userId, refreshToken);
      }

      // Blacklist access token
      const accessToken = req.headers.authorization?.split(' ')[1];
      if (accessToken) {
        await authService.blacklistToken(accessToken);
      }

      logger.info('User logged out', { userId });

      res.status(200).json({
        success: true,
        message: 'Logged out successfully'
      });

    } catch (error) {
      logger.error('Logout error:', error);
      next(error);
    }
  }

  /**
   * Email Verification
   */
  public async verifyEmail(req: Request, res: Response, next: NextFunction): Promise<void> {
    try {
      const { token } = req.query;

      if (!token) {
        res.status(400).json({
          success: false,
          message: 'Verification token required',
          error: 'TOKEN_REQUIRED'
        });
        return;
      }

      // Verify email verification token
      const decoded = jwt.verify(token as string, process.env.EMAIL_VERIFICATION_SECRET!) as any;
      const userId = decoded.userId;

      const user = await userService.findById(userId);
      if (!user) {
        res.status(404).json({
          success: false,
          message: 'User not found',
          error: 'USER_NOT_FOUND'
        });
        return;
      }

      if (user.isEmailVerified) {
        res.status(400).json({
          success: false,
          message: 'Email already verified',
          error: 'EMAIL_ALREADY_VERIFIED'
        });
        return;
      }

      // Update user email verification status
      await userService.updateEmailVerification(userId, true);

      // Award XP for email verification
      await userService.addXP(userId, 100, 'EMAIL_VERIFICATION');

      // Activate mining if KYC is also complete
      if (user.isKYCVerified) {
        await userService.activateMining(userId);
      }

      logger.info('Email verified successfully', { userId });

      res.status(200).json({
        success: true,
        message: 'Email verified successfully',
        data: {
          canStartMining: user.isKYCVerified
        }
      });

    } catch (error) {
      if (error.name === 'TokenExpiredError') {
        res.status(400).json({
          success: false,
          message: 'Verification token expired',
          error: 'TOKEN_EXPIRED'
        });
        return;
      }

      logger.error('Email verification error:', error);
      next(error);
    }
  }

  /**
   * Resend Email Verification
   */
  public async resendVerification(req: Request, res: Response, next: NextFunction): Promise<void> {
    try {
      const { email } = req.body;

      const user = await userService.findByEmail(email);
      if (!user) {
        res.status(404).json({
          success: false,
          message: 'User not found',
          error: 'USER_NOT_FOUND'
        });
        return;
      }

      if (user.isEmailVerified) {
        res.status(400).json({
          success: false,
          message: 'Email already verified',
          error: 'EMAIL_ALREADY_VERIFIED'
        });
        return;
      }

      // Check rate limiting
      const canResend = await authService.checkEmailResendLimit(user.id);
      if (!canResend) {
        res.status(429).json({
          success: false,
          message: 'Too many verification emails sent. Please wait.',
          error: 'RATE_LIMITED'
        });
        return;
      }

      // Generate new verification token
      const token = await authService.generateEmailVerificationToken(user.id);
      await notificationService.sendEmailVerification(email, token);

      res.status(200).json({
        success: true,
        message: 'Verification email sent'
      });

    } catch (error) {
      logger.error('Resend verification error:', error);
      next(error);
    }
  }

  /**
   * Forgot Password
   */
  public async forgotPassword(req: Request, res: Response, next: NextFunction): Promise<void> {
    try {
      const { email } = req.body;

      const user = await userService.findByEmail(email);
      if (!user) {
        // Don't reveal if email exists
        res.status(200).json({
          success: true,
          message: 'If the email exists, a reset link has been sent'
        });
        return;
      }

      // Check rate limiting
      const canReset = await authService.checkPasswordResetLimit(user.id);
      if (!canReset) {
        res.status(429).json({
          success: false,
          message: 'Too many reset attempts. Please wait.',
          error: 'RATE_LIMITED'
        });
        return;
      }

      // Generate reset token
      const resetToken = await authService.generatePasswordResetToken(user.id);
      await notificationService.sendPasswordReset(email, resetToken);

      logger.info('Password reset requested', {
        userId: user.id,
        email: email.replace(/(.{2}).*(@.*)/, '$1***$2')
      });

      res.status(200).json({
        success: true,
        message: 'If the email exists, a reset link has been sent'
      });

    } catch (error) {
      logger.error('Forgot password error:', error);
      next(error);
    }
  }

  /**
   * Reset Password
   */
  public async resetPassword(req: Request, res: Response, next: NextFunction): Promise<void> {
    try {
      const { token, newPassword, confirmPassword } = req.body;

      if (!validatePassword(newPassword)) {
        res.status(400).json({
          success: false,
          message: 'Password must be at least 8 characters with uppercase, lowercase, number and special character',
          error: 'WEAK_PASSWORD'
        });
        return;
      }

      if (newPassword !== confirmPassword) {
        res.status(400).json({
          success: false,
          message: 'Passwords do not match',
          error: 'PASSWORD_MISMATCH'
        });
        return;
      }

      // Verify reset token
      const decoded = jwt.verify(token, process.env.PASSWORD_RESET_SECRET!) as any;
      const userId = decoded.userId;

      const user = await userService.findById(userId);
      if (!user) {
        res.status(404).json({
          success: false,
          message: 'Invalid reset token',
          error: 'INVALID_TOKEN'
        });
        return;
      }

      // Hash new password
      const hashedPassword = await bcrypt.hash(newPassword, 12);

      // Update password
      await userService.updatePassword(userId, hashedPassword);

      // Invalidate all existing refresh tokens for security
      await authService.revokeAllRefreshTokens(userId);

      logger.info('Password reset successful', { userId });

      res.status(200).json({
        success: true,
        message: 'Password reset successfully'
      });

    } catch (error) {
      if (error.name === 'TokenExpiredError') {
        res.status(400).json({
          success: false,
          message: 'Reset token expired',
          error: 'TOKEN_EXPIRED'
        });
        return;
      }

      logger.error('Reset password error:', error);
      next(error);
    }
  }

  /**
   * Enable Two-Factor Authentication
   */
  public async enableTwoFactor(req: Request, res: Response, next: NextFunction): Promise<void> {
    try {
      const userId = req.user?.id;
      const { token } = req.body;

      const user = await userService.findById(userId);
      if (!user) {
        res.status(404).json({
          success: false,
          message: 'User not found',
          error: 'USER_NOT_FOUND'
        });
        return;
      }

      const twoFactorSecret = decryptSensitiveData(user.twoFactorSecret);
      
      // Verify the provided token
      const isValid = speakeasy.totp.verify({
        secret: twoFactorSecret,
        encoding: 'base32',
        token: token,
        window: 2
      });

      if (!isValid) {
        res.status(400).json({
          success: false,
          message: 'Invalid 2FA code',
          error: 'INVALID_TWO_FACTOR'
        });
        return;
      }

      // Enable 2FA
      await userService.updateTwoFactorStatus(userId, true);

      // Generate backup codes
      const backupCodes = await authService.generateBackupCodes(userId);

      logger.info('2FA enabled', { userId });

      res.status(200).json({
        success: true,
        message: '2FA enabled successfully',
        data: { backupCodes }
      });

    } catch (error) {
      logger.error('Enable 2FA error:', error);
      next(error);
    }
  }

  /**
   * Disable Two-Factor Authentication
   */
  public async disableTwoFactor(req: Request, res: Response, next: NextFunction): Promise<void> {
    try {
      const userId = req.user?.id;
      const { password, token } = req.body;

      const user = await userService.findById(userId);
      if (!user) {
        res.status(404).json({
          success: false,
          message: 'User not found',
          error: 'USER_NOT_FOUND'
        });
        return;
      }

      // Verify password
      const isPasswordValid = await bcrypt.compare(password, user.password);
      if (!isPasswordValid) {
        res.status(401).json({
          success: false,
          message: 'Invalid password',
          error: 'INVALID_PASSWORD'
        });
        return;
      }

      // Verify 2FA token
      const twoFactorSecret = decryptSensitiveData(user.twoFactorSecret);
      const isValidToken = speakeasy.totp.verify({
        secret: twoFactorSecret,
        encoding: 'base32',
        token: token,
        window: 2
      });

      if (!isValidToken) {
        res.status(400).json({
          success: false,
          message: 'Invalid 2FA code',
          error: 'INVALID_TWO_FACTOR'
        });
        return;
      }

      // Disable 2FA
      await userService.updateTwoFactorStatus(userId, false);

      // Remove backup codes
      await authService.removeBackupCodes(userId);

      logger.info('2FA disabled', { userId });

      res.status(200).json({
        success: true,
        message: '2FA disabled successfully'
      });

    } catch (error) {
      logger.error('Disable 2FA error:', error);
      next(error);
    }
  }

  /**
   * Initialize KYC Process
   */
  public async initiateKYC(req: Request, res: Response, next: NextFunction): Promise<void> {
    try {
      const userId = req.user?.id;
      const {
        documentType,
        documentNumber,
        dateOfBirth,
        address,
        city,
        state,
        zipCode,
        occupation
      } = req.body;

      const user = await userService.findById(userId);
      if (!user) {
        res.status(404).json({
          success: false,
          message: 'User not found',
          error: 'USER_NOT_FOUND'
        });
        return;
      }

      if (user.isKYCVerified) {
        res.status(400).json({
          success: false,
          message: 'KYC already verified',
          error: 'KYC_ALREADY_VERIFIED'
        });
        return;
      }

      // Initialize KYC session
      const kycSession = await kycService.initializeKYC(userId, {
        documentType,
        documentNumber: encryptSensitiveData(documentNumber),
        dateOfBirth,
        address: encryptSensitiveData(address),
        city,
        state,
        zipCode,
        occupation
      });

      logger.info('KYC process initiated', { userId, kycSessionId: kycSession.id });

      res.status(200).json({
        success: true,
        message: 'KYC process initiated',
        data: {
          sessionId: kycSession.id,
          uploadUrl: kycSession.uploadUrl,
          requiredDocuments: kycSession.requiredDocuments,
          expiresAt: kycSession.expiresAt
        }
      });

    } catch (error) {
      logger.error('KYC initiation error:', error);
      next(error);
    }
  }

  /**
   * Submit KYC Documents
   */
  public async submitKYCDocuments(req: Request, res: Response, next: NextFunction): Promise<void> {
    try {
      const userId = req.user?.id;
      const { sessionId } = req.params;
      const files = req.files as Express.Multer.File[];

      if (!files || files.length === 0) {
        res.status(400).json({
          success: false,
          message: 'Documents required',
          error: 'DOCUMENTS_REQUIRED'
        });
        return;
      }

      // Verify KYC session
      const kycSession = await kycService.getKYCSession(sessionId);
      if (!kycSession || kycSession.userId !== userId) {
        res.status(404).json({
          success: false,
          message: 'Invalid KYC session',
          error: 'INVALID_KYC_SESSION'
        });
        return;
      }

      // Process and validate documents
      const processedDocuments = await kycService.processDocuments(sessionId, files);

      // Run AI-powered document verification
      const verificationResults = await kycService.verifyDocuments(processedDocuments);

      // Update KYC session with results
      await kycService.updateKYCSession(sessionId, {
        status: 'UNDER_REVIEW',
        documents: processedDocuments,
        verificationResults,
        submittedAt: new Date()
      });

      // If auto-verification passed, mark as verified
      if (verificationResults.autoVerified && verificationResults.confidence > 0.95) {
        await this.approveKYC(userId, sessionId);
      } else {
        // Queue for manual review
        await kycService.queueForManualReview(sessionId);
      }

      logger.info('KYC documents submitted', { 
        userId, 
        sessionId, 
        documentsCount: files.length,
        autoVerified: verificationResults.autoVerified
      });

      res.status(200).json({
        success: true,
        message: 'Documents submitted successfully',
        data: {
          status: verificationResults.autoVerified ? 'VERIFIED' : 'UNDER_REVIEW',
          estimatedReviewTime: verificationResults.autoVerified ? null : '24-48 hours',
          verificationResults
        }
      });

    } catch (error) {
      logger.error('KYC document submission error:', error);
      next(error);
    }
  }

  /**
   * Get KYC Status
   */
  public async getKYCStatus(req: Request, res: Response, next: NextFunction): Promise<void> {
    try {
      const userId = req.user?.id;

      const kycData = await kycService.getKYCStatus(userId);

      res.status(200).json({
        success: true,
        data: {
          isVerified: kycData?.isVerified || false,
          status: kycData?.status || 'NOT_STARTED',
          submittedAt: kycData?.submittedAt,
          verifiedAt: kycData?.verifiedAt,
          rejectionReason: kycData?.rejectionReason,
          canResubmit: kycData?.canResubmit || false
        }
      });

    } catch (error) {
      logger.error('Get KYC status error:', error);
      next(error);
    }
  }

  /**
   * Verify Biometric Data
   */
  public async verifyBiometric(req: Request, res: Response, next: NextFunction): Promise<void> {
    try {
      const userId = req.user?.id;
      const { biometricData, action } = req.body;

      const user = await userService.findById(userId);
      if (!user) {
        res.status(404).json({
          success: false,
          message: 'User not found',
          error: 'USER_NOT_FOUND'
        });
        return;
      }

      // Get stored biometric data
      const storedBiometric = await userService.getBiometricData(userId);
      if (!storedBiometric) {
        res.status(400).json({
          success: false,
          message: 'No biometric data on file',
          error: 'NO_BIOMETRIC_DATA'
        });
        return;
      }

      // Verify biometric data
      const isValid = await authService.verifyBiometric(biometricData, storedBiometric);

      if (isValid) {
        // Update human probability score
        const newHumanProbability = Math.min(1.0, user.humanProbability + 0.05);
        await userService.updateHumanProbability(userId, newHumanProbability);

        // Log successful biometric verification
        logger.info('Biometric verification successful', { userId, action });

        res.status(200).json({
          success: true,
          message: 'Biometric verification successful',
          data: { verified: true }
        });
      } else {
        // Log failed biometric verification
        logger.warn('Biometric verification failed', { userId, action });

        res.status(401).json({
          success: false,
          message: 'Biometric verification failed',
          error: 'BIOMETRIC_VERIFICATION_FAILED'
        });
      }

    } catch (error) {
      logger.error('Biometric verification error:', error);
      next(error);
    }
  }

  /**
   * Get User Security Settings
   */
  public async getSecuritySettings(req: Request, res: Response, next: NextFunction): Promise<void> {
    try {
      const userId = req.user?.id;

      const user = await userService.findById(userId);
      if (!user) {
        res.status(404).json({
          success: false,
          message: 'User not found',
          error: 'USER_NOT_FOUND'
        });
        return;
      }

      const trustedDevices = await authService.getTrustedDevices(userId);
      const loginHistory = await authService.getLoginHistory(userId, 10);
      const backupCodes = await authService.getBackupCodesStatus(userId);

      res.status(200).json({
        success: true,
        data: {
          twoFactorEnabled: user.isTwoFactorEnabled,
          biometricEnabled: user.requiresBiometricAuth,
          emailVerified: user.isEmailVerified,
          phoneVerified: user.isPhoneVerified,
          kycVerified: user.isKYCVerified,
          trustedDevices: trustedDevices.length,
          hasBackupCodes: backupCodes.hasActiveCodes,
          recentLogins: loginHistory,
          securityScore: await authService.calculateSecurityScore(user)
        }
      });

    } catch (error) {
      logger.error('Get security settings error:', error);
      next(error);
    }
  }

  /**
   * Update Security Settings
   */
  public async updateSecuritySettings(req: Request, res: Response, next: NextFunction): Promise<void> {
    try {
      const userId = req.user?.id;
      const { 
        requireBiometric,
        loginNotifications,
        sensitiveActionNotifications,
        deviceTrustExpiry
      } = req.body;

      const updates: any = {};

      if (typeof requireBiometric === 'boolean') {
        updates.requiresBiometricAuth = requireBiometric;
      }

      if (typeof loginNotifications === 'boolean') {
        updates.loginNotificationsEnabled = loginNotifications;
      }

      if (typeof sensitiveActionNotifications === 'boolean') {
        updates.sensitiveActionNotificationsEnabled = sensitiveActionNotifications;
      }

      if (deviceTrustExpiry && [7, 30, 90, 365].includes(deviceTrustExpiry)) {
        updates.deviceTrustExpiryDays = deviceTrustExpiry;
      }

      await userService.updateSecuritySettings(userId, updates);

      logger.info('Security settings updated', { userId, updates });

      res.status(200).json({
        success: true,
        message: 'Security settings updated successfully'
      });

    } catch (error) {
      logger.error('Update security settings error:', error);
      next(error);
    }
  }

  /**
   * Revoke Device Access
   */
  public async revokeDevice(req: Request, res: Response, next: NextFunction): Promise<void> {
    try {
      const userId = req.user?.id;
      const { deviceId } = req.params;

      await authService.revokeTrustedDevice(userId, deviceId);

      logger.info('Device access revoked', { userId, deviceId });

      res.status(200).json({
        success: true,
        message: 'Device access revoked successfully'
      });

    } catch (error) {
      logger.error('Revoke device error:', error);
      next(error);
    }
  }

  /**
   * Generate New Backup Codes
   */
  public async generateBackupCodes(req: Request, res: Response, next: NextFunction): Promise<void> {
    try {
      const userId = req.user?.id;
      const { password } = req.body;

      const user = await userService.findById(userId);
      if (!user) {
        res.status(404).json({
          success: false,
          message: 'User not found',
          error: 'USER_NOT_FOUND'
        });
        return;
      }

      // Verify password for security
      const isPasswordValid = await bcrypt.compare(password, user.password);
      if (!isPasswordValid) {
        res.status(401).json({
          success: false,
          message: 'Invalid password',
          error: 'INVALID_PASSWORD'
        });
        return;
      }

      // Generate new backup codes
      const backupCodes = await authService.generateBackupCodes(userId);

      logger.info('Backup codes regenerated', { userId });

      res.status(200).json({
        success: true,
        message: 'New backup codes generated',
        data: { backupCodes }
      });

    } catch (error) {
      logger.error('Generate backup codes error:', error);
      next(error);
    }
  }

  /**
   * Change Password
   */
  public async changePassword(req: Request, res: Response, next: NextFunction): Promise<void> {
    try {
      const userId = req.user?.id;
      const { currentPassword, newPassword, confirmPassword } = req.body;

      if (!validatePassword(newPassword)) {
        res.status(400).json({
          success: false,
          message: 'Password must be at least 8 characters with uppercase, lowercase, number and special character',
          error: 'WEAK_PASSWORD'
        });
        return;
      }

      if (newPassword !== confirmPassword) {
        res.status(400).json({
          success: false,
          message: 'Passwords do not match',
          error: 'PASSWORD_MISMATCH'
        });
        return;
      }

      const user = await userService.findById(userId);
      if (!user) {
        res.status(404).json({
          success: false,
          message: 'User not found',
          error: 'USER_NOT_FOUND'
        });
        return;
      }

      // Verify current password
      const isCurrentPasswordValid = await bcrypt.compare(currentPassword, user.password);
      if (!isCurrentPasswordValid) {
        res.status(401).json({
          success: false,
          message: 'Current password is incorrect',
          error: 'INVALID_CURRENT_PASSWORD'
        });
        return;
      }

      // Check if new password is different from current
      const isSamePassword = await bcrypt.compare(newPassword, user.password);
      if (isSamePassword) {
        res.status(400).json({
          success: false,
          message: 'New password must be different from current password',
          error: 'SAME_PASSWORD'
        });
        return;
      }

      // Hash new password
      const hashedPassword = await bcrypt.hash(newPassword, 12);

      // Update password
      await userService.updatePassword(userId, hashedPassword);

      // Revoke all refresh tokens except current session
      const currentRefreshToken = req.body.currentRefreshToken;
      await authService.revokeAllRefreshTokensExcept(userId, currentRefreshToken);

      // Send password change notification
      await notificationService.sendPasswordChangeNotification(user.email);

      logger.info('Password changed successfully', { userId });

      res.status(200).json({
        success: true,
        message: 'Password changed successfully'
      });

    } catch (error) {
      logger.error('Change password error:', error);
      next(error);
    }
  }

  /**
   * Delete Account (with security checks)
   */
  public async deleteAccount(req: Request, res: Response, next: NextFunction): Promise<void> {
    try {
      const userId = req.user?.id;
      const { password, confirmDeletion, reason } = req.body;

      if (confirmDeletion !== 'DELETE_MY_ACCOUNT') {
        res.status(400).json({
          success: false,
          message: 'Account deletion not confirmed',
          error: 'DELETION_NOT_CONFIRMED'
        });
        return;
      }

      const user = await userService.findById(userId);
      if (!user) {
        res.status(404).json({
          success: false,
          message: 'User not found',
          error: 'USER_NOT_FOUND'
        });
        return;
      }

      // Verify password
      const isPasswordValid = await bcrypt.compare(password, user.password);
      if (!isPasswordValid) {
        res.status(401).json({
          success: false,
          message: 'Invalid password',
          error: 'INVALID_PASSWORD'
        });
        return;
      }

      // Check if user has significant assets (prevent accidental deletion)
      const userStats = await userService.getUserStats(userId);
      if (userStats.totalFIN > 1000 || userStats.nftCount > 10) {
        res.status(400).json({
          success: false,
          message: 'Cannot delete account with significant assets. Please withdraw or transfer first.',
          error: 'HAS_SIGNIFICANT_ASSETS'
        });
        return;
      }

      // Process account deletion
      await userService.deleteAccount(userId, reason);

      // Send deletion confirmation email
      await notificationService.sendAccountDeletionConfirmation(user.email);

      logger.info('Account deleted', { 
        userId, 
        email: user.email.replace(/(.{2}).*(@.*)/, '$1***$2'), 
        reason 
      });

      res.status(200).json({
        success: true,
        message: 'Account deleted successfully'
      });

    } catch (error) {
      logger.error('Delete account error:', error);
      next(error);
    }
  }

  // Private helper methods

  /**
   * Approve KYC after verification
   */
  private async approveKYC(userId: string, kycSessionId: string): Promise<void> {
    try {
      // Update user KYC status
      await userService.updateKYCStatus(userId, true);

      // Award KYC completion XP and RP
      await userService.addXP(userId, 500, 'KYC_VERIFICATION');
      await userService.addReferralPoints(userId, 100, 'KYC_COMPLETION');

      // Activate mining if email is also verified
      const user = await userService.findById(userId);
      if (user?.isEmailVerified) {
        await userService.activateMining(userId);
      }

      // Update KYC session
      await kycService.updateKYCSession(kycSessionId, {
        status: 'VERIFIED',
        verifiedAt: new Date()
      });

      // Process referral KYC bonus if user was referred
      if (user?.referredBy) {
        await userService.addReferralPoints(user.referredBy, 200, 'REFERRAL_KYC_SUCCESS');
        await userService.addXP(user.referredBy, 100, 'REFERRAL_KYC_SUCCESS');
      }

      // Send KYC approval notification
      await notificationService.sendKYCApproval(user!.email);

      logger.info('KYC approved', { userId, kycSessionId });

    } catch (error) {
      logger.error('KYC approval error:', error);
      throw error;
    }
  }
}
