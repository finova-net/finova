import crypto from 'crypto';
import bcrypt from 'bcryptjs';
import jwt from 'jsonwebtoken';
import { promisify } from 'util';

// Constants for encryption
const ALGORITHM = 'aes-256-gcm';
const KEY_LENGTH = 32;
const IV_LENGTH = 16;
const SALT_ROUNDS = 12;
const TAG_LENGTH = 16;

// Interface definitions
interface EncryptedData {
  encrypted: string;
  iv: string;
  tag: string;
  salt?: string;
}

interface JWTPayload {
  userId: string;
  walletAddress?: string;
  kycStatus: boolean;
  xpLevel: number;
  rpTier: number;
  finBalance: string;
  sessionId: string;
  deviceFingerprint: string;
  iat?: number;
  exp?: number;
}

interface BiometricData {
  template: string;
  quality: number;
  timestamp: number;
}

/**
 * Enterprise-grade encryption utility class for Finova Network
 * Handles user data, financial transactions, biometric templates, and session management
 */
export class EncryptionUtils {
  private static instance: EncryptionUtils;
  private masterKey: Buffer;
  private jwtSecret: string;
  private jwtRefreshSecret: string;

  private constructor() {
    // Initialize encryption keys from environment
    this.masterKey = this.deriveMasterKey(
      process.env.ENCRYPTION_MASTER_KEY || 'finova-default-key-change-in-production'
    );
    this.jwtSecret = process.env.JWT_SECRET || 'finova-jwt-secret-change-in-production';
    this.jwtRefreshSecret = process.env.JWT_REFRESH_SECRET || 'finova-refresh-secret-change-in-production';
  }

  public static getInstance(): EncryptionUtils {
    if (!EncryptionUtils.instance) {
      EncryptionUtils.instance = new EncryptionUtils();
    }
    return EncryptionUtils.instance;
  }

  /**
   * Derive master key from passphrase using PBKDF2
   */
  private deriveMasterKey(passphrase: string): Buffer {
    const salt = crypto.createHash('sha256').update('finova-salt').digest();
    return crypto.pbkdf2Sync(passphrase, salt, 100000, KEY_LENGTH, 'sha512');
  }

  /**
   * Generate cryptographically secure random key
   */
  public generateSecureKey(length: number = KEY_LENGTH): Buffer {
    return crypto.randomBytes(length);
  }

  /**
   * Encrypt sensitive user data (KYC info, wallet keys, etc.)
   */
  public encryptUserData(plaintext: string, userKey?: Buffer): EncryptedData {
    try {
      const key = userKey || this.masterKey;
      const iv = crypto.randomBytes(IV_LENGTH);
      const cipher = crypto.createCipher(ALGORITHM, key);
      
      cipher.setAAD(Buffer.from('finova-user-data'));
      
      let encrypted = cipher.update(plaintext, 'utf8', 'hex');
      encrypted += cipher.final('hex');
      
      const tag = cipher.getAuthTag();

      return {
        encrypted,
        iv: iv.toString('hex'),
        tag: tag.toString('hex')
      };
    } catch (error) {
      throw new Error(`Encryption failed: ${error.message}`);
    }
  }

  /**
   * Decrypt sensitive user data
   */
  public decryptUserData(encryptedData: EncryptedData, userKey?: Buffer): string {
    try {
      const key = userKey || this.masterKey;
      const decipher = crypto.createDecipher(ALGORITHM, key);
      
      decipher.setAuthTag(Buffer.from(encryptedData.tag, 'hex'));
      decipher.setAAD(Buffer.from('finova-user-data'));
      
      let decrypted = decipher.update(encryptedData.encrypted, 'hex', 'utf8');
      decrypted += decipher.final('utf8');
      
      return decrypted;
    } catch (error) {
      throw new Error(`Decryption failed: ${error.message}`);
    }
  }

  /**
   * Encrypt financial transaction data with additional security layers
   */
  public encryptFinancialData(data: any, userWalletKey: string): EncryptedData {
    try {
      const jsonData = JSON.stringify(data);
      const salt = crypto.randomBytes(32);
      const key = crypto.pbkdf2Sync(userWalletKey, salt, 50000, KEY_LENGTH, 'sha512');
      const iv = crypto.randomBytes(IV_LENGTH);
      
      const cipher = crypto.createCipherGCM(ALGORITHM, key, iv);
      cipher.setAAD(Buffer.from('finova-financial-tx'));
      
      let encrypted = cipher.update(jsonData, 'utf8', 'hex');
      encrypted += cipher.final('hex');
      
      const tag = cipher.getAuthTag();

      return {
        encrypted,
        iv: iv.toString('hex'),
        tag: tag.toString('hex'),
        salt: salt.toString('hex')
      };
    } catch (error) {
      throw new Error(`Financial data encryption failed: ${error.message}`);
    }
  }

  /**
   * Decrypt financial transaction data
   */
  public decryptFinancialData(encryptedData: EncryptedData, userWalletKey: string): any {
    try {
      const salt = Buffer.from(encryptedData.salt!, 'hex');
      const key = crypto.pbkdf2Sync(userWalletKey, salt, 50000, KEY_LENGTH, 'sha512');
      const iv = Buffer.from(encryptedData.iv, 'hex');
      
      const decipher = crypto.createDecipherGCM(ALGORITHM, key, iv);
      decipher.setAuthTag(Buffer.from(encryptedData.tag, 'hex'));
      decipher.setAAD(Buffer.from('finova-financial-tx'));
      
      let decrypted = decipher.update(encryptedData.encrypted, 'hex', 'utf8');
      decrypted += decipher.final('utf8');
      
      return JSON.parse(decrypted);
    } catch (error) {
      throw new Error(`Financial data decryption failed: ${error.message}`);
    }
  }

  /**
   * Hash passwords with salt using bcrypt
   */
  public async hashPassword(password: string): Promise<string> {
    try {
      return await bcrypt.hash(password, SALT_ROUNDS);
    } catch (error) {
      throw new Error(`Password hashing failed: ${error.message}`);
    }
  }

  /**
   * Verify password against hash
   */
  public async verifyPassword(password: string, hashedPassword: string): Promise<boolean> {
    try {
      return await bcrypt.compare(password, hashedPassword);
    } catch (error) {
      throw new Error(`Password verification failed: ${error.message}`);
    }
  }

  /**
   * Generate JWT access token with user session data
   */
  public generateAccessToken(payload: JWTPayload, expiresIn: string = '15m'): string {
    try {
      const tokenPayload = {
        ...payload,
        tokenType: 'access',
        timestamp: Date.now()
      };

      return jwt.sign(tokenPayload, this.jwtSecret, {
        expiresIn,
        algorithm: 'HS512',
        issuer: 'finova-network',
        audience: 'finova-users'
      });
    } catch (error) {
      throw new Error(`Access token generation failed: ${error.message}`);
    }
  }

  /**
   * Generate JWT refresh token
   */
  public generateRefreshToken(userId: string, sessionId: string, expiresIn: string = '7d'): string {
    try {
      const payload = {
        userId,
        sessionId,
        tokenType: 'refresh',
        timestamp: Date.now()
      };

      return jwt.sign(payload, this.jwtRefreshSecret, {
        expiresIn,
        algorithm: 'HS512',
        issuer: 'finova-network',
        audience: 'finova-users'
      });
    } catch (error) {
      throw new Error(`Refresh token generation failed: ${error.message}`);
    }
  }

  /**
   * Verify and decode JWT access token
   */
  public verifyAccessToken(token: string): JWTPayload {
    try {
      return jwt.verify(token, this.jwtSecret, {
        algorithms: ['HS512'],
        issuer: 'finova-network',
        audience: 'finova-users'
      }) as JWTPayload;
    } catch (error) {
      throw new Error(`Access token verification failed: ${error.message}`);
    }
  }

  /**
   * Verify refresh token
   */
  public verifyRefreshToken(token: string): any {
    try {
      return jwt.verify(token, this.jwtRefreshSecret, {
        algorithms: ['HS512'],
        issuer: 'finova-network',
        audience: 'finova-users'
      });
    } catch (error) {
      throw new Error(`Refresh token verification failed: ${error.message}`);
    }
  }

  /**
   * Encrypt biometric template data for KYC
   */
  public encryptBiometricData(biometricData: BiometricData, userId: string): EncryptedData {
    try {
      const userKey = crypto.createHash('sha256').update(`${userId}-biometric-key`).digest();
      const jsonData = JSON.stringify(biometricData);
      const iv = crypto.randomBytes(IV_LENGTH);
      
      const cipher = crypto.createCipherGCM(ALGORITHM, userKey, iv);
      cipher.setAAD(Buffer.from('finova-biometric-template'));
      
      let encrypted = cipher.update(jsonData, 'utf8', 'hex');
      encrypted += cipher.final('hex');
      
      const tag = cipher.getAuthTag();

      return {
        encrypted,
        iv: iv.toString('hex'),
        tag: tag.toString('hex')
      };
    } catch (error) {
      throw new Error(`Biometric encryption failed: ${error.message}`);
    }
  }

  /**
   * Decrypt biometric template data
   */
  public decryptBiometricData(encryptedData: EncryptedData, userId: string): BiometricData {
    try {
      const userKey = crypto.createHash('sha256').update(`${userId}-biometric-key`).digest();
      const iv = Buffer.from(encryptedData.iv, 'hex');
      
      const decipher = crypto.createDecipherGCM(ALGORITHM, userKey, iv);
      decipher.setAuthTag(Buffer.from(encryptedData.tag, 'hex'));
      decipher.setAAD(Buffer.from('finova-biometric-template'));
      
      let decrypted = decipher.update(encryptedData.encrypted, 'hex', 'utf8');
      decrypted += decipher.final('utf8');
      
      return JSON.parse(decrypted);
    } catch (error) {
      throw new Error(`Biometric decryption failed: ${error.message}`);
    }
  }

  /**
   * Generate device fingerprint hash
   */
  public generateDeviceFingerprint(deviceInfo: any): string {
    const deviceString = JSON.stringify({
      userAgent: deviceInfo.userAgent,
      screenResolution: deviceInfo.screenResolution,
      timezone: deviceInfo.timezone,
      language: deviceInfo.language,
      platform: deviceInfo.platform,
      hardwareConcurrency: deviceInfo.hardwareConcurrency
    });

    return crypto.createHash('sha256').update(deviceString).digest('hex');
  }

  /**
   * Create secure session ID
   */
  public generateSessionId(): string {
    return crypto.randomBytes(32).toString('hex');
  }

  /**
   * Generate referral code with entropy
   */
  public generateReferralCode(userId: string): string {
    const entropy = crypto.randomBytes(4).toString('hex');
    const userHash = crypto.createHash('md5').update(userId).digest('hex').slice(0, 4);
    return `FIN${userHash}${entropy}`.toUpperCase();
  }

  /**
   * Hash sensitive query parameters for logging
   */
  public hashForLogging(sensitiveData: string): string {
    return crypto.createHash('sha256').update(sensitiveData).digest('hex').slice(0, 16);
  }

  /**
   * Generate API key for third-party integrations
   */
  public generateAPIKey(userId: string, scope: string): string {
    const timestamp = Date.now().toString();
    const entropy = crypto.randomBytes(16).toString('hex');
    const payload = `${userId}:${scope}:${timestamp}:${entropy}`;
    
    const hmac = crypto.createHmac('sha256', this.masterKey);
    hmac.update(payload);
    const signature = hmac.digest('hex');
    
    return Buffer.from(`${payload}:${signature}`).toString('base64url');
  }

  /**
   * Verify API key integrity
   */
  public verifyAPIKey(apiKey: string): { userId: string; scope: string; timestamp: number; valid: boolean } {
    try {
      const decoded = Buffer.from(apiKey, 'base64url').toString('utf8');
      const parts = decoded.split(':');
      
      if (parts.length !== 5) {
        return { userId: '', scope: '', timestamp: 0, valid: false };
      }

      const [userId, scope, timestamp, entropy, signature] = parts;
      const payload = `${userId}:${scope}:${timestamp}:${entropy}`;
      
      const hmac = crypto.createHmac('sha256', this.masterKey);
      hmac.update(payload);
      const expectedSignature = hmac.digest('hex');
      
      const valid = crypto.timingSafeEqual(
        Buffer.from(signature, 'hex'),
        Buffer.from(expectedSignature, 'hex')
      );

      return {
        userId,
        scope,
        timestamp: parseInt(timestamp),
        valid
      };
    } catch (error) {
      return { userId: '', scope: '', timestamp: 0, valid: false };
    }
  }

  /**
   * Encrypt mining reward data with anti-tamper protection
   */
  public encryptMiningReward(rewardData: any, userWallet: string): EncryptedData {
    try {
      const timestamp = Date.now();
      const dataWithTimestamp = { ...rewardData, timestamp, nonce: crypto.randomBytes(8).toString('hex') };
      const jsonData = JSON.stringify(dataWithTimestamp);
      
      const key = crypto.createHash('sha256').update(`${userWallet}-mining-rewards`).digest();
      const iv = crypto.randomBytes(IV_LENGTH);
      
      const cipher = crypto.createCipherGCM(ALGORITHM, key, iv);
      cipher.setAAD(Buffer.from('finova-mining-rewards'));
      
      let encrypted = cipher.update(jsonData, 'utf8', 'hex');
      encrypted += cipher.final('hex');
      
      const tag = cipher.getAuthTag();

      return {
        encrypted,
        iv: iv.toString('hex'),
        tag: tag.toString('hex')
      };
    } catch (error) {
      throw new Error(`Mining reward encryption failed: ${error.message}`);
    }
  }

  /**
   * Create message authentication code (MAC) for data integrity
   */
  public createMAC(data: string, key?: Buffer): string {
    const macKey = key || this.masterKey;
    return crypto.createHmac('sha256', macKey).update(data).digest('hex');
  }

  /**
   * Verify message authentication code
   */
  public verifyMAC(data: string, mac: string, key?: Buffer): boolean {
    const macKey = key || this.masterKey;
    const expectedMAC = crypto.createHmac('sha256', macKey).update(data).digest('hex');
    
    return crypto.timingSafeEqual(
      Buffer.from(mac, 'hex'),
      Buffer.from(expectedMAC, 'hex')
    );
  }

  /**
   * Secure random number generation for fair distribution
   */
  public generateSecureRandom(min: number, max: number): number {
    const range = max - min + 1;
    const bytesNeeded = Math.ceil(Math.log2(range) / 8);
    const maxValidValue = Math.floor(256 ** bytesNeeded / range) * range - 1;
    
    let randomValue;
    do {
      const randomBytes = crypto.randomBytes(bytesNeeded);
      randomValue = randomBytes.readUIntBE(0, bytesNeeded);
    } while (randomValue > maxValidValue);
    
    return min + (randomValue % range);
  }

  /**
   * Clean up sensitive data from memory (best effort)
   */
  public secureWipe(buffer: Buffer): void {
    if (buffer && buffer.length > 0) {
      crypto.randomFillSync(buffer);
      buffer.fill(0);
    }
  }
}

// Export singleton instance
export const encryption = EncryptionUtils.getInstance();

// Export types
export type { EncryptedData, JWTPayload, BiometricData };

// Utility functions for common operations
export const hashPasswordAsync = async (password: string): Promise<string> => {
  return encryption.hashPassword(password);
};

export const verifyPasswordAsync = async (password: string, hash: string): Promise<boolean> => {
  return encryption.verifyPassword(password, hash);
};

export const createSecureToken = (payload: JWTPayload, expiresIn?: string): string => {
  return encryption.generateAccessToken(payload, expiresIn);
};

export const verifySecureToken = (token: string): JWTPayload => {
  return encryption.verifyAccessToken(token);
};

export const encryptSensitiveData = (data: string, key?: Buffer): EncryptedData => {
  return encryption.encryptUserData(data, key);
};

export const decryptSensitiveData = (encryptedData: EncryptedData, key?: Buffer): string => {
  return encryption.decryptUserData(encryptedData, key);
};
