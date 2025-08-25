import { Injectable, UnauthorizedException, BadRequestException, ConflictException } from '@nestjs/common';
import { JwtService } from '@nestjs/jwt';
import { InjectRepository } from '@nestjs/typeorm';
import { Repository } from 'typeorm';
import { ConfigService } from '@nestjs/config';
import * as bcrypt from 'bcryptjs';
import * as crypto from 'crypto';
import { User } from '../models/User.model';
import { UserService } from './user.service';
import { BlockchainService } from './blockchain.service';
import { AntiBot } from './anti-bot.service';
import { NotificationService } from './notification.service';
import { Connection } from '@solana/web3.js';
import * as face from 'face-api.js';

export interface AuthTokens {
  accessToken: string;
  refreshToken: string;
  user: Partial<User>;
}

export interface KYCData {
  firstName: string;
  lastName: string;
  dateOfBirth: string;
  nationality: string;
  idNumber: string;
  phoneNumber: string;
  address: string;
  selfieImage: string; // base64
  idFrontImage: string; // base64
  idBackImage: string; // base64
  biometricData?: string;
}

export interface SocialAuthData {
  provider: 'instagram' | 'tiktok' | 'youtube' | 'facebook' | 'twitter';
  accessToken: string;
  profile: any;
  followersCount?: number;
}

@Injectable()
export class AuthService {
  private readonly connection: Connection;
  
  constructor(
    @InjectRepository(User)
    private userRepository: Repository<User>,
    private jwtService: JwtService,
    private configService: ConfigService,
    private userService: UserService,
    private blockchainService: BlockchainService,
    private antiBotService: AntiBot,
    private notificationService: NotificationService,
  ) {
    this.connection = new Connection(
      this.configService.get<string>('SOLANA_RPC_URL'),
      'confirmed'
    );
  }

  // Register with email/password
  async register(
    email: string,
    password: string,
    referralCode?: string,
    deviceFingerprint?: string
  ): Promise<AuthTokens> {
    // Check if user exists
    const existingUser = await this.userRepository.findOne({ where: { email } });
    if (existingUser) {
      throw new ConflictException('User already exists');
    }

    // Validate referral code
    let referrer: User | null = null;
    if (referralCode) {
      referrer = await this.userRepository.findOne({ where: { referralCode } });
      if (!referrer) {
        throw new BadRequestException('Invalid referral code');
      }
    }

    // Anti-bot check
    const botScore = await this.antiBotService.analyzeRegistration({
      email,
      deviceFingerprint,
      timestamp: new Date(),
      referralCode
    });

    if (botScore > 0.7) {
      throw new BadRequestException('Registration blocked - suspicious activity');
    }

    // Hash password
    const hashedPassword = await bcrypt.hash(password, 12);

    // Generate unique referral code
    const newReferralCode = await this.generateUniqueReferralCode();

    // Create Solana wallet
    const wallet = await this.blockchainService.createWallet();

    // Create user
    const user = this.userRepository.create({
      email,
      password: hashedPassword,
      referralCode: newReferralCode,
      referredBy: referrer?.id,
      walletAddress: wallet.publicKey,
      walletPrivateKey: this.encryptPrivateKey(wallet.privateKey),
      deviceFingerprint,
      botScore,
      isActive: true,
      level: 1,
      totalXP: 0,
      totalRP: 0,
      totalMining: 0,
      miningRate: this.calculateInitialMiningRate(),
      lastMiningClaim: new Date(),
      createdAt: new Date(),
    });

    const savedUser = await this.userRepository.save(user);

    // Process referral
    if (referrer) {
      await this.processReferral(referrer, savedUser);
    }

    // Initialize mining
    await this.blockchainService.initializeUserMining(savedUser.walletAddress);

    // Send welcome notification
    await this.notificationService.sendWelcomeEmail(savedUser.email);

    return this.generateTokens(savedUser);
  }

  // Login with email/password
  async login(
    email: string,
    password: string,
    deviceFingerprint?: string
  ): Promise<AuthTokens> {
    const user = await this.userRepository.findOne({ 
      where: { email },
      relations: ['referrer', 'referrals']
    });

    if (!user) {
      throw new UnauthorizedException('Invalid credentials');
    }

    // Check password
    const isPasswordValid = await bcrypt.compare(password, user.password);
    if (!isPasswordValid) {
      throw new UnauthorizedException('Invalid credentials');
    }

    // Anti-bot check
    const botScore = await this.antiBotService.analyzeLogin({
      userId: user.id,
      deviceFingerprint,
      timestamp: new Date(),
      ipAddress: this.getCurrentIP()
    });

    if (botScore > 0.8) {
      throw new UnauthorizedException('Login blocked - suspicious activity');
    }

    // Update last login
    user.lastLoginAt = new Date();
    user.deviceFingerprint = deviceFingerprint;
    user.botScore = Math.max(user.botScore - 0.1, botScore); // Gradually reduce if legitimate
    
    await this.userRepository.save(user);

    return this.generateTokens(user);
  }

  // Social media authentication
  async socialAuth(socialData: SocialAuthData, referralCode?: string): Promise<AuthTokens> {
    // Verify social token
    const profileData = await this.verifySocialToken(socialData.provider, socialData.accessToken);
    
    const socialId = profileData.id;
    const email = profileData.email || `${socialId}@${socialData.provider}.finova.net`;

    let user = await this.userRepository.findOne({
      where: [
        { [`${socialData.provider}Id`]: socialId },
        { email }
      ]
    });

    if (!user) {
      // Create new user from social profile
      const referrer = referralCode ? 
        await this.userRepository.findOne({ where: { referralCode } }) : null;

      const wallet = await this.blockchainService.createWallet();
      const newReferralCode = await this.generateUniqueReferralCode();

      user = this.userRepository.create({
        email,
        [`${socialData.provider}Id`]: socialId,
        [`${socialData.provider}Token`]: socialData.accessToken,
        [`${socialData.provider}Profile`]: profileData,
        followersCount: socialData.followersCount || 0,
        referralCode: newReferralCode,
        referredBy: referrer?.id,
        walletAddress: wallet.publicKey,
        walletPrivateKey: this.encryptPrivateKey(wallet.privateKey),
        isActive: true,
        level: 1,
        totalXP: 50, // Bonus for social signup
        totalRP: 0,
        totalMining: 0,
        miningRate: this.calculateInitialMiningRate(),
        lastMiningClaim: new Date(),
        createdAt: new Date(),
      });

      user = await this.userRepository.save(user);

      // Process referral
      if (referrer) {
        await this.processReferral(referrer, user);
      }

      // Initialize blockchain mining
      await this.blockchainService.initializeUserMining(user.walletAddress);
    } else {
      // Update existing user's social data
      user[`${socialData.provider}Token`] = socialData.accessToken;
      user[`${socialData.provider}Profile`] = profileData;
      user.followersCount = socialData.followersCount || user.followersCount;
      user.lastLoginAt = new Date();
      
      await this.userRepository.save(user);
    }

    return this.generateTokens(user);
  }

  // KYC submission
  async submitKYC(userId: string, kycData: KYCData): Promise<{ success: boolean; message: string }> {
    const user = await this.userRepository.findOne({ where: { id: userId } });
    if (!user) {
      throw new BadRequestException('User not found');
    }

    if (user.kycStatus === 'APPROVED') {
      throw new BadRequestException('KYC already approved');
    }

    // Validate images and extract data
    const validation = await this.validateKYCData(kycData);
    if (!validation.isValid) {
      throw new BadRequestException(`KYC validation failed: ${validation.errors.join(', ')}`);
    }

    // Biometric verification
    const biometricValid = await this.verifyBiometrics(kycData.selfieImage, kycData.biometricData);
    if (!biometricValid) {
      throw new BadRequestException('Biometric verification failed');
    }

    // Store encrypted KYC data
    user.kycData = this.encryptKYCData(kycData);
    user.kycStatus = 'PENDING';
    user.kycSubmittedAt = new Date();
    user.firstName = kycData.firstName;
    user.lastName = kycData.lastName;
    user.phoneNumber = kycData.phoneNumber;
    user.nationality = kycData.nationality;

    await this.userRepository.save(user);

    // Trigger KYC review process
    await this.triggerKYCReview(user.id);

    return {
      success: true,
      message: 'KYC submitted successfully. Review typically takes 1-3 business days.'
    };
  }

  // Refresh tokens
  async refreshTokens(refreshToken: string): Promise<AuthTokens> {
    try {
      const payload = this.jwtService.verify(refreshToken, {
        secret: this.configService.get<string>('JWT_REFRESH_SECRET'),
      });

      const user = await this.userRepository.findOne({ 
        where: { id: payload.sub },
        relations: ['referrer', 'referrals']
      });

      if (!user || user.refreshTokenHash !== this.hashToken(refreshToken)) {
        throw new UnauthorizedException('Invalid refresh token');
      }

      return this.generateTokens(user);
    } catch (error) {
      throw new UnauthorizedException('Invalid refresh token');
    }
  }

  // Logout
  async logout(userId: string): Promise<void> {
    await this.userRepository.update(userId, { refreshTokenHash: null });
  }

  // Validate JWT token
  async validateUser(payload: any): Promise<User> {
    const user = await this.userRepository.findOne({ 
      where: { id: payload.sub },
      relations: ['referrer', 'referrals']
    });
    
    if (!user || !user.isActive) {
      throw new UnauthorizedException('User not found or inactive');
    }

    // Update last activity
    user.lastActivityAt = new Date();
    await this.userRepository.save(user);

    return user;
  }

  // Password reset request
  async requestPasswordReset(email: string): Promise<void> {
    const user = await this.userRepository.findOne({ where: { email } });
    if (!user) {
      return; // Don't reveal if email exists
    }

    const resetToken = crypto.randomBytes(32).toString('hex');
    const resetTokenHash = crypto.createHash('sha256').update(resetToken).digest('hex');

    user.passwordResetToken = resetTokenHash;
    user.passwordResetExpires = new Date(Date.now() + 15 * 60 * 1000); // 15 minutes
    
    await this.userRepository.save(user);
    await this.notificationService.sendPasswordResetEmail(email, resetToken);
  }

  // Reset password
  async resetPassword(token: string, newPassword: string): Promise<void> {
    const resetTokenHash = crypto.createHash('sha256').update(token).digest('hex');
    
    const user = await this.userRepository.findOne({
      where: {
        passwordResetToken: resetTokenHash,
      },
    });

    if (!user || user.passwordResetExpires < new Date()) {
      throw new BadRequestException('Invalid or expired reset token');
    }

    user.password = await bcrypt.hash(newPassword, 12);
    user.passwordResetToken = null;
    user.passwordResetExpires = null;
    
    await this.userRepository.save(user);
  }

  // Private helper methods
  private async generateTokens(user: User): Promise<AuthTokens> {
    const payload = {
      sub: user.id,
      email: user.email,
      level: user.level,
      kycStatus: user.kycStatus,
    };

    const accessToken = this.jwtService.sign(payload, {
      secret: this.configService.get<string>('JWT_SECRET'),
      expiresIn: '15m',
    });

    const refreshToken = this.jwtService.sign(payload, {
      secret: this.configService.get<string>('JWT_REFRESH_SECRET'),
      expiresIn: '7d',
    });

    // Store refresh token hash
    user.refreshTokenHash = this.hashToken(refreshToken);
    await this.userRepository.save(user);

    return {
      accessToken,
      refreshToken,
      user: {
        id: user.id,
        email: user.email,
        level: user.level,
        totalXP: user.totalXP,
        totalRP: user.totalRP,
        miningRate: user.miningRate,
        kycStatus: user.kycStatus,
        walletAddress: user.walletAddress,
        referralCode: user.referralCode,
      },
    };
  }

  private async generateUniqueReferralCode(): Promise<string> {
    let code: string;
    let exists = true;
    
    while (exists) {
      code = crypto.randomBytes(4).toString('hex').toUpperCase();
      const existing = await this.userRepository.findOne({ where: { referralCode: code } });
      exists = !!existing;
    }
    
    return code;
  }

  private calculateInitialMiningRate(): number {
    // Pi Network-inspired mining rate calculation
    const totalUsers = 50000; // This would be dynamic
    const baseRate = 0.1;
    const pioneerBonus = Math.max(1.0, 2.0 - (totalUsers / 1000000));
    
    return baseRate * pioneerBonus;
  }

  private async processReferral(referrer: User, newUser: User): Promise<void> {
    // Award referral points
    referrer.totalRP += 50;
    referrer.activeReferrals = (referrer.activeReferrals || 0) + 1;
    
    // Bonus XP for both users
    referrer.totalXP += 100;
    newUser.totalXP += 25;
    
    await this.userRepository.save([referrer, newUser]);
    
    // Send notification
    await this.notificationService.sendReferralNotification(referrer.email, newUser.email);
  }

  private async verifySocialToken(provider: string, token: string): Promise<any> {
    // Implement social token verification for each platform
    const apis = {
      instagram: 'https://graph.instagram.com/me',
      facebook: 'https://graph.facebook.com/me',
      youtube: 'https://www.googleapis.com/oauth2/v1/userinfo',
      tiktok: 'https://open-api.tiktok.com/platform/oauth/connect/',
      twitter: 'https://api.twitter.com/1.1/account/verify_credentials.json'
    };
    
    const response = await fetch(`${apis[provider]}?access_token=${token}`);
    if (!response.ok) {
      throw new UnauthorizedException('Invalid social token');
    }
    
    return response.json();
  }

  private async validateKYCData(kycData: KYCData): Promise<{ isValid: boolean; errors: string[] }> {
    const errors: string[] = [];

    // Validate required fields
    if (!kycData.firstName || !kycData.lastName) {
      errors.push('Name is required');
    }
    
    if (!kycData.idNumber || kycData.idNumber.length < 10) {
      errors.push('Valid ID number is required');
    }
    
    // Validate images
    if (!this.isValidBase64Image(kycData.selfieImage)) {
      errors.push('Valid selfie image is required');
    }
    
    if (!this.isValidBase64Image(kycData.idFrontImage)) {
      errors.push('Valid ID front image is required');
    }

    // Age validation
    const birthDate = new Date(kycData.dateOfBirth);
    const age = this.calculateAge(birthDate);
    if (age < 18) {
      errors.push('Must be 18 years or older');
    }

    return {
      isValid: errors.length === 0,
      errors
    };
  }

  private async verifyBiometrics(selfieImage: string, biometricData?: string): Promise<boolean> {
    // Implement facial recognition and liveness detection
    try {
      // This would integrate with face-api.js or similar
      const buffer = Buffer.from(selfieImage.split(',')[1], 'base64');
      // Perform face detection and liveness check
      return true; // Simplified for now
    } catch (error) {
      return false;
    }
  }

  private encryptPrivateKey(privateKey: string): string {
    const cipher = crypto.createCipher('aes-256-cbc', this.configService.get<string>('ENCRYPTION_KEY'));
    let encrypted = cipher.update(privateKey, 'utf8', 'hex');
    encrypted += cipher.final('hex');
    return encrypted;
  }

  private encryptKYCData(kycData: KYCData): string {
    const cipher = crypto.createCipher('aes-256-cbc', this.configService.get<string>('KYC_ENCRYPTION_KEY'));
    let encrypted = cipher.update(JSON.stringify(kycData), 'utf8', 'hex');
    encrypted += cipher.final('hex');
    return encrypted;
  }

  private hashToken(token: string): string {
    return crypto.createHash('sha256').update(token).digest('hex');
  }

  private isValidBase64Image(base64String: string): boolean {
    try {
      const matches = base64String.match(/^data:image\/([a-zA-Z]*);base64,([^\"]*)/);
      return matches !== null && matches.length === 3;
    } catch (error) {
      return false;
    }
  }

  private calculateAge(birthDate: Date): number {
    const today = new Date();
    let age = today.getFullYear() - birthDate.getFullYear();
    const monthDiff = today.getMonth() - birthDate.getMonth();
    
    if (monthDiff < 0 || (monthDiff === 0 && today.getDate() < birthDate.getDate())) {
      age--;
    }
    
    return age;
  }

  private async triggerKYCReview(userId: string): Promise<void> {
    // Queue KYC for manual review
    // This would integrate with admin dashboard
    console.log(`KYC review queued for user: ${userId}`);
  }

  private getCurrentIP(): string {
    // This would be injected from request context
    return '127.0.0.1';
  }
}
