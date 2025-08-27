// mobile-sdk/react-native/src/utils/validation.ts
import { ERROR_CODES } from './constants';

export interface ValidationResult {
  isValid: boolean;
  errors: string[];
  errorCode?: string;
}

export class FinovaValidation {
  
  // User Input Validation
  static validateEmail(email: string): ValidationResult {
    const emailRegex = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;
    const isValid = emailRegex.test(email) && email.length <= 254;
    
    return {
      isValid,
      errors: isValid ? [] : ['Invalid email format'],
      errorCode: isValid ? undefined : ERROR_CODES.VALIDATION_FORMAT_ERROR
    };
  }

  static validatePassword(password: string): ValidationResult {
    const errors: string[] = [];
    
    if (password.length < 8) errors.push('Password must be at least 8 characters');
    if (password.length > 128) errors.push('Password must be less than 128 characters');
    if (!/[A-Z]/.test(password)) errors.push('Password must contain uppercase letter');
    if (!/[a-z]/.test(password)) errors.push('Password must contain lowercase letter');
    if (!/\d/.test(password)) errors.push('Password must contain number');
    if (!/[!@#$%^&*(),.?":{}|<>]/.test(password)) errors.push('Password must contain special character');
    
    return {
      isValid: errors.length === 0,
      errors,
      errorCode: errors.length > 0 ? ERROR_CODES.VALIDATION_FORMAT_ERROR : undefined
    };
  }

  static validatePhoneNumber(phone: string, countryCode: string = 'ID'): ValidationResult {
    const cleanPhone = phone.replace(/\D/g, '');
    
    let isValid = false;
    if (countryCode === 'ID') {
      // Indonesian phone validation (08xxx or +628xxx)
      isValid = /^(08|628)\d{8,11}$/.test(cleanPhone) || /^(\+628)\d{8,11}$/.test(phone);
    } else {
      // Generic international format
      isValid = cleanPhone.length >= 7 && cleanPhone.length <= 15;
    }
    
    return {
      isValid,
      errors: isValid ? [] : ['Invalid phone number format'],
      errorCode: isValid ? undefined : ERROR_CODES.VALIDATION_FORMAT_ERROR
    };
  }

  static validateUsername(username: string): ValidationResult {
    const errors: string[] = [];
    
    if (username.length < 3) errors.push('Username must be at least 3 characters');
    if (username.length > 30) errors.push('Username must be less than 30 characters');
    if (!/^[a-zA-Z0-9_]+$/.test(username)) errors.push('Username can only contain letters, numbers, and underscores');
    if (/^[_]/.test(username) || /[_]$/.test(username)) errors.push('Username cannot start or end with underscore');
    if (/_{2,}/.test(username)) errors.push('Username cannot contain consecutive underscores');
    
    return {
      isValid: errors.length === 0,
      errors,
      errorCode: errors.length > 0 ? ERROR_CODES.VALIDATION_FORMAT_ERROR : undefined
    };
  }

  // Referral Code Validation
  static validateReferralCode(code: string): ValidationResult {
    const isValid = /^[A-Z0-9]{6,12}$/.test(code);
    
    return {
      isValid,
      errors: isValid ? [] : ['Invalid referral code format'],
      errorCode: isValid ? undefined : ERROR_CODES.VALIDATION_FORMAT_ERROR
    };
  }

  // Social Platform Validation
  static validateSocialHandle(handle: string, platform: string): ValidationResult {
    let pattern: RegExp;
    let maxLength: number;
    
    switch (platform.toLowerCase()) {
      case 'instagram':
        pattern = /^[a-zA-Z0-9_.]+$/;
        maxLength = 30;
        break;
      case 'tiktok':
        pattern = /^[a-zA-Z0-9_.]+$/;
        maxLength = 24;
        break;
      case 'youtube':
        pattern = /^[a-zA-Z0-9_-]+$/;
        maxLength = 20;
        break;
      case 'twitter':
      case 'x':
        pattern = /^[a-zA-Z0-9_]+$/;
        maxLength = 15;
        break;
      case 'facebook':
        pattern = /^[a-zA-Z0-9.]+$/;
        maxLength = 50;
        break;
      default:
        pattern = /^[a-zA-Z0-9_.@-]+$/;
        maxLength = 50;
    }
    
    const isValid = pattern.test(handle) && handle.length <= maxLength && handle.length >= 1;
    
    return {
      isValid,
      errors: isValid ? [] : [`Invalid ${platform} handle format`],
      errorCode: isValid ? undefined : ERROR_CODES.VALIDATION_FORMAT_ERROR
    };
  }

  // Content Validation
  static validateContentText(content: string, minLength: number = 1, maxLength: number = 2000): ValidationResult {
    const errors: string[] = [];
    const trimmedContent = content.trim();
    
    if (trimmedContent.length < minLength) errors.push(`Content must be at least ${minLength} characters`);
    if (trimmedContent.length > maxLength) errors.push(`Content must be less than ${maxLength} characters`);
    if (this.containsProfanity(trimmedContent)) errors.push('Content contains inappropriate language');
    if (this.containsSpam(trimmedContent)) errors.push('Content appears to be spam');
    
    return {
      isValid: errors.length === 0,
      errors,
      errorCode: errors.length > 0 ? ERROR_CODES.VALIDATION_FORMAT_ERROR : undefined
    };
  }

  private static containsProfanity(text: string): boolean {
    // Basic profanity filter - in production, use a comprehensive list
    const profanityList = ['spam', 'scam', 'fake', 'bot', 'cheat'];
    const lowerText = text.toLowerCase();
    return profanityList.some(word => lowerText.includes(word));
  }

  private static containsSpam(text: string): boolean {
    // Basic spam detection
    const spamIndicators = [
      /(.)\1{4,}/, // Repeated characters
      /^[A-Z\s!]{20,}$/, // All caps
      /(https?:\/\/[^\s]+){3,}/, // Multiple URLs
      /(\$|USD|EUR|BTC|ETH|\d+\s?(dollar|euro))/gi // Financial terms
    ];
    
    return spamIndicators.some(pattern => pattern.test(text));
  }

  // Blockchain Validation
  static validateSolanaAddress(address: string): ValidationResult {
    const isValid = /^[1-9A-HJ-NP-Za-km-z]{32,44}$/.test(address);
    
    return {
      isValid,
      errors: isValid ? [] : ['Invalid Solana address format'],
      errorCode: isValid ? undefined : ERROR_CODES.VALIDATION_FORMAT_ERROR
    };
  }

  static validateTokenAmount(amount: string, maxDecimals: number = 9): ValidationResult {
    const errors: string[] = [];
    const numAmount = parseFloat(amount);
    
    if (isNaN(numAmount)) errors.push('Amount must be a valid number');
    if (numAmount <= 0) errors.push('Amount must be greater than zero');
    if (numAmount > 1000000000) errors.push('Amount exceeds maximum limit');
    
    const decimalPlaces = (amount.split('.')[1] || '').length;
    if (decimalPlaces > maxDecimals) errors.push(`Amount cannot have more than ${maxDecimals} decimal places`);
    
    return {
      isValid: errors.length === 0,
      errors,
      errorCode: errors.length > 0 ? ERROR_CODES.VALIDATION_FORMAT_ERROR : undefined
    };
  }

  // KYC Document Validation
  static validateKYCDocument(documentType: string, documentNumber: string, country: string = 'ID'): ValidationResult {
    let pattern: RegExp;
    let name: string;
    
    if (country === 'ID') {
      switch (documentType.toUpperCase()) {
        case 'KTP':
        case 'ID_CARD':
          pattern = /^\d{16}$/;
          name = 'KTP number';
          break;
        case 'PASSPORT':
          pattern = /^[A-Z]\d{7}$/;
          name = 'Passport number';
          break;
        case 'DRIVER_LICENSE':
          pattern = /^\d{12}$/;
          name = 'Driver license number';
          break;
        default:
          return {
            isValid: false,
            errors: ['Unsupported document type'],
            errorCode: ERROR_CODES.VALIDATION_FORMAT_ERROR
          };
      }
    } else {
      // Generic validation for international documents
      pattern = /^[A-Z0-9]{6,20}$/;
      name = 'Document number';
    }
    
    const isValid = pattern.test(documentNumber);
    
    return {
      isValid,
      errors: isValid ? [] : [`Invalid ${name} format`],
      errorCode: isValid ? undefined : ERROR_CODES.VALIDATION_FORMAT_ERROR
    };
  }

  // Transaction Validation
  static validateTransactionData(data: any): ValidationResult {
    const errors: string[] = [];
    
    if (!data.to) errors.push('Recipient address is required');
    if (!data.amount) errors.push('Amount is required');
    if (!data.token) errors.push('Token type is required');
    
    if (data.to && !this.validateSolanaAddress(data.to).isValid) {
      errors.push('Invalid recipient address');
    }
    
    if (data.amount && !this.validateTokenAmount(data.amount.toString()).isValid) {
      errors.push('Invalid amount');
    }
    
    return {
      isValid: errors.length === 0,
      errors,
      errorCode: errors.length > 0 ? ERROR_CODES.VALIDATION_FORMAT_ERROR : undefined
    };
  }

  // Security Validation
  static validateApiKey(apiKey: string): ValidationResult {
    const isValid = /^[A-Za-z0-9+/]{32,}={0,2}$/.test(apiKey) && apiKey.length >= 32;
    
    return {
      isValid,
      errors: isValid ? [] : ['Invalid API key format'],
      errorCode: isValid ? undefined : ERROR_CODES.VALIDATION_FORMAT_ERROR
    };
  }

  static validateJWT(token: string): ValidationResult {
    const parts = token.split('.');
    const isValid = parts.length === 3 && parts.every(part => /^[A-Za-z0-9_-]+$/.test(part));
    
    return {
      isValid,
      errors: isValid ? [] : ['Invalid JWT token format'],
      errorCode: isValid ? undefined : ERROR_CODES.VALIDATION_FORMAT_ERROR
    };
  }

  // Batch Validation
  static validateBatch(validations: Array<() => ValidationResult>): ValidationResult {
    const results = validations.map(validation => validation());
    const allErrors = results.reduce((acc, result) => [...acc, ...result.errors], [] as string[]);
    const firstError = results.find(result => !result.isValid);
    
    return {
      isValid: allErrors.length === 0,
      errors: allErrors,
      errorCode: firstError?.errorCode
    };
  }
}

// mobile-sdk/react-native/src/utils/formatting.ts
import { FINOVA_CONSTANTS } from './constants';

export class FinovaFormatting {
  
  // Token Formatting
  static formatFinAmount(amount: number, decimals: number = 4, showSymbol: boolean = true): string {
    if (amount === 0) return showSymbol ? '0.0000 $FIN' : '0.0000';
    
    const formatted = amount.toFixed(decimals);
    const withCommas = this.addCommas(formatted);
    
    return showSymbol ? `${withCommas} $FIN` : withCommas;
  }

  static formatUSDAmount(amount: number, decimals: number = 2): string {
    const formatted = amount.toFixed(decimals);
    return `${this.addCommas(formatted)}`;
  }

  static formatPercentage(value: number, decimals: number = 2): string {
    return `${(value * 100).toFixed(decimals)}%`;
  }

  private static addCommas(numStr: string): string {
    const parts = numStr.split('.');
    parts[0] = parts[0].replace(/\B(?=(\d{3})+(?!\d))/g, ',');
    return parts.join('.');
  }

  // Time Formatting
  static formatDuration(seconds: number): string {
    if (seconds < 60) return `${Math.floor(seconds)}s`;
    if (seconds < 3600) return `${Math.floor(seconds / 60)}m ${Math.floor(seconds % 60)}s`;
    if (seconds < 86400) {
      const hours = Math.floor(seconds / 3600);
      const minutes = Math.floor((seconds % 3600) / 60);
      return `${hours}h ${minutes}m`;
    }
    
    const days = Math.floor(seconds / 86400);
    const hours = Math.floor((seconds % 86400) / 3600);
    return `${days}d ${hours}h`;
  }

  static formatTimeAgo(timestamp: number): string {
    const now = Date.now();
    const diff = Math.floor((now - timestamp) / 1000);
    
    if (diff < 60) return 'just now';
    if (diff < 3600) return `${Math.floor(diff / 60)}m ago`;
    if (diff < 86400) return `${Math.floor(diff / 3600)}h ago`;
    if (diff < 2592000) return `${Math.floor(diff / 86400)}d ago`;
    if (diff < 31536000) return `${Math.floor(diff / 2592000)}mo ago`;
    return `${Math.floor(diff / 31536000)}y ago`;
  }

  static formatCountdown(targetTimestamp: number): string {
    const now = Date.now();
    const diff = Math.floor((targetTimestamp - now) / 1000);
    
    if (diff <= 0) return 'Expired';
    return this.formatDuration(diff);
  }

  // User Stats Formatting
  static formatXPLevel(level: number, showBadge: boolean = true): string {
    const badges = ['Bronze', 'Silver', 'Gold', 'Platinum', 'Diamond', 'Mythic'];
    let badge = 'Bronze';
    
    if (level <= 10) badge = 'Bronze';
    else if (level <= 25) badge = 'Silver';
    else if (level <= 50) badge = 'Gold';
    else if (level <= 75) badge = 'Platinum';
    else if (level <= 100) badge = 'Diamond';
    else badge = 'Mythic';
    
    return showBadge ? `Level ${level} (${badge})` : `Level ${level}`;
  }

  static formatRPTier(tier: number): string {
    const tiers = ['Explorer', 'Connector', 'Influencer', 'Leader', 'Ambassador'];
    return tiers[tier] || 'Unknown';
  }

  static formatNetworkSize(size: number): string {
    if (size < 1000) return size.toString();
    if (size < 1000000) return `${(size / 1000).toFixed(1)}K`;
    return `${(size / 1000000).toFixed(1)}M`;
  }

  // Mining Stats Formatting
  static formatMiningRate(rate: number): string {
    if (rate < 0.001) return `${(rate * 1000000).toFixed(2)}μ $FIN/h`;
    if (rate < 1) return `${(rate * 1000).toFixed(2)}m $FIN/h`;
    return `${rate.toFixed(4)} $FIN/h`;
  }

  static formatHashRate(rate: number): string {
    if (rate < 1000) return `${rate.toFixed(2)} H/s`;
    if (rate < 1000000) return `${(rate / 1000).toFixed(2)} KH/s`;
    if (rate < 1000000000) return `${(rate / 1000000).toFixed(2)} MH/s`;
    return `${(rate / 1000000000).toFixed(2)} GH/s`;
  }

  // Social Media Formatting
  static formatSocialStats(count: number): string {
    if (count < 1000) return count.toString();
    if (count < 1000000) return `${(count / 1000).toFixed(1)}K`;
    if (count < 1000000000) return `${(count / 1000000).toFixed(1)}M`;
    return `${(count / 1000000000).toFixed(1)}B`;
  }

  static formatEngagementRate(rate: number): string {
    return `${(rate * 100).toFixed(2)}%`;
  }

  // Address Formatting
  static formatAddress(address: string, startChars: number = 6, endChars: number = 4): string {
    if (address.length <= startChars + endChars) return address;
    return `${address.substring(0, startChars)}...${address.substring(address.length - endChars)}`;
  }

  static formatTransactionHash(hash: string): string {
    return this.formatAddress(hash, 8, 8);
  }

  // File Size Formatting
  static formatFileSize(bytes: number): string {
    if (bytes === 0) return '0 B';
    
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    
    return `${parseFloat((bytes / Math.pow(k, i)).toFixed(2))} ${sizes[i]}`;
  }

  // Quality Score Formatting
  static formatQualityScore(score: number): { text: string; color: string; emoji: string } {
    if (score >= 1.8) return { text: 'Excellent', color: '#22c55e', emoji: '🏆' };
    if (score >= 1.5) return { text: 'Great', color: '#3b82f6', emoji: '⭐' };
    if (score >= 1.2) return { text: 'Good', color: '#eab308', emoji: '👍' };
    if (score >= 0.8) return { text: 'Fair', color: '#f97316', emoji: '👌' };
    return { text: 'Poor', color: '#ef4444', emoji: '👎' };
  }

  // Price Change Formatting
  static formatPriceChange(change: number): { text: string; color: string; prefix: string } {
    const prefix = change >= 0 ? '+' : '';
    const color = change >= 0 ? '#22c55e' : '#ef4444';
    const text = `${prefix}${this.formatPercentage(Math.abs(change))}`;
    
    return { text, color, prefix };
  }

  // Card Rarity Formatting
  static formatCardRarity(rarity: string): { text: string; color: string; gradient: string[] } {
    const rarityData = {
      COMMON: { text: 'Common', color: '#9ca3af', gradient: ['#9ca3af', '#6b7280'] },
      UNCOMMON: { text: 'Uncommon', color: '#22c55e', gradient: ['#22c55e', '#16a34a'] },
      RARE: { text: 'Rare', color: '#3b82f6', gradient: ['#3b82f6', '#2563eb'] },
      EPIC: { text: 'Epic', color: '#8b5cf6', gradient: ['#8b5cf6', '#7c3aed'] },
      LEGENDARY: { text: 'Legendary', color: '#f59e0b', gradient: ['#f59e0b', '#d97706'] }
    };
    
    return rarityData[rarity as keyof typeof rarityData] || rarityData.COMMON;
  }

  // Validation Message Formatting
  static formatValidationErrors(errors: string[]): string {
    if (errors.length === 0) return '';
    if (errors.length === 1) return errors[0];
    return errors.map((error, index) => `${index + 1}. ${error}`).join('\n');
  }

  // Number Input Formatting
  static formatNumberInput(value: string, decimals: number = 9): string {
    // Remove non-numeric characters except decimal point
    let cleaned = value.replace(/[^0-9.]/g, '');
    
    // Ensure only one decimal point
    const parts = cleaned.split('.');
    if (parts.length > 2) {
      cleaned = parts[0] + '.' + parts.slice(1).join('');
    }
    
    // Limit decimal places
    if (parts.length === 2 && parts[1].length > decimals) {
      cleaned = parts[0] + '.' + parts[1].substring(0, decimals);
    }
    
    return cleaned;
  }

  // Progress Formatting
  static formatProgress(current: number, total: number): { percentage: number; text: string } {
    const percentage = Math.min(100, (current / total) * 100);
    const text = `${this.formatNetworkSize(current)} / ${this.formatNetworkSize(total)}`;
    
    return { percentage, text };
  }

  // Achievement Formatting
  static formatAchievement(type: string, milestone: number): string {
    const types = {
      MINING: '⛏️ Mining Milestone',
      XP: '⭐ XP Achievement', 
      REFERRAL: '🤝 Network Builder',
      SOCIAL: '📱 Social Star',
      STAKING: '💎 Diamond Hands',
      NFT: '🎨 Collector'
    };
    
    return `${types[type as keyof typeof types] || type}: ${this.formatNetworkSize(milestone)}`;
  }
}

// mobile-sdk/react-native/src/utils/crypto.ts
import CryptoJS from 'crypto-js';

export class FinovaCrypto {
  
  // Encryption/Decryption
  static encrypt(data: string, key: string): string {
    try {
      return CryptoJS.AES.encrypt(data, key).toString();
    } catch (error) {
      throw new Error('Encryption failed');
    }
  }

  static decrypt(encryptedData: string, key: string): string {
    try {
      const bytes = CryptoJS.AES.decrypt(encryptedData, key);
      return bytes.toString(CryptoJS.enc.Utf8);
    } catch (error) {
      throw new Error('Decryption failed');
    }
  }

  // Hashing
  static sha256(data: string): string {
    return CryptoJS.SHA256(data).toString();
  }

  static md5(data: string): string {
    return CryptoJS.MD5(data).toString();
  }

  // HMAC
  static hmacSha256(data: string, key: string): string {
    return CryptoJS.HmacSHA256(data, key).toString();
  }

  // Random Generation
  static generateRandomString(length: number = 32): string {
    const chars = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789';
    let result = '';
    for (let i = 0; i < length; i++) {
      result += chars.charAt(Math.floor(Math.random() * chars.length));
    }
    return result;
  }

  static generateUUID(): string {
    return 'xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx'.replace(/[xy]/g, (c) => {
      const r = Math.random() * 16 | 0;
      const v = c === 'x' ? r : (r & 0x3 | 0x8);
      return v.toString(16);
    });
  }

  // Base64 Encoding/Decoding
  static base64Encode(data: string): string {
    return CryptoJS.enc.Base64.stringify(CryptoJS.enc.Utf8.parse(data));
  }

  static base64Decode(encodedData: string): string {
    return CryptoJS.enc.Base64.parse(encodedData).toString(CryptoJS.enc.Utf8);
  }

  // Key Derivation
  static deriveKey(password: string, salt: string, iterations: number = 10000): string {
    return CryptoJS.PBKDF2(password, salt, {
      keySize: 256/32,
      iterations
    }).toString();
  }

  // Digital Signature Simulation
  static signMessage(message: string, privateKey: string): string {
    return this.hmacSha256(message, privateKey);
  }

  static verifySignature(message: string, signature: string, publicKey: string): boolean {
    const expectedSignature = this.hmacSha256(message, publicKey);
    return signature === expectedSignature;
  }
}
