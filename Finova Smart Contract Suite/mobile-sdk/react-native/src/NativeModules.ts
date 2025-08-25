import { NativeModules, Platform } from 'react-native';

// Native Module Interfaces
interface FinovaWalletModule {
  // Wallet Operations
  generateWallet(): Promise<{
    publicKey: string;
    privateKey: string;
    mnemonic: string;
  }>;
  
  importWallet(mnemonic: string): Promise<{
    publicKey: string;
    address: string;
  }>;
  
  getBalance(address: string): Promise<{
    fin: number;
    sFin: number;
    usdFin: number;
  }>;
  
  signTransaction(transaction: string, privateKey: string): Promise<string>;
  
  // Biometric Authentication
  authenticateWithBiometric(): Promise<boolean>;
  isBiometricAvailable(): Promise<boolean>;
}

interface FinovaMiningModule {
  // Mining Operations
  startMining(userId: string): Promise<{
    success: boolean;
    miningRate: number;
    sessionId: string;
  }>;
  
  stopMining(sessionId: string): Promise<boolean>;
  
  getMiningStatus(userId: string): Promise<{
    isActive: boolean;
    currentRate: number;
    totalEarned: number;
    hoursRemaining: number;
  }>;
  
  claimRewards(userId: string): Promise<{
    amount: number;
    transactionHash: string;
  }>;
  
  // Mining Calculations
  calculateMiningRate(userStats: {
    holdings: number;
    referrals: number;
    xpLevel: number;
    rpTier: number;
    isKycVerified: boolean;
  }): Promise<number>;
}

interface FinovaXPModule {
  // XP Operations
  recordActivity(activity: {
    type: 'post' | 'comment' | 'like' | 'share' | 'login';
    platform: 'instagram' | 'tiktok' | 'youtube' | 'facebook' | 'x';
    content?: string;
    engagement?: number;
  }): Promise<{
    xpGained: number;
    newLevel: number;
    multiplier: number;
  }>;
  
  getXPStatus(userId: string): Promise<{
    currentXP: number;
    level: number;
    nextLevelXP: number;
    miningBonus: number;
    streak: number;
  }>;
  
  validateContent(content: string, type: string): Promise<{
    qualityScore: number;
    isOriginal: boolean;
    engagementPrediction: number;
  }>;
}

interface FinovaReferralModule {
  // Referral Operations
  generateReferralCode(userId: string): Promise<string>;
  
  processReferral(referralCode: string, newUserId: string): Promise<{
    success: boolean;
    rpGained: number;
    bonusUnlocked: boolean;
  }>;
  
  getReferralStats(userId: string): Promise<{
    totalReferrals: number;
    activeReferrals: number;
    rpPoints: number;
    tier: string;
    networkSize: number;
    earnings: number;
  }>;
  
  getReferralNetwork(userId: string, depth: number): Promise<{
    level1: Array<{ id: string; activity: number; earnings: number }>;
    level2: Array<{ id: string; activity: number; earnings: number }>;
    level3: Array<{ id: string; activity: number; earnings: number }>;
  }>;
}

interface FinovaNFTModule {
  // NFT Operations
  mintSpecialCard(cardType: string, userId: string): Promise<{
    tokenId: string;
    metadata: object;
    transactionHash: string;
  }>;
  
  useSpecialCard(tokenId: string, userId: string): Promise<{
    effectApplied: boolean;
    duration: number;
    multiplier: number;
  }>;
  
  getUserNFTs(userId: string): Promise<Array<{
    tokenId: string;
    cardType: string;
    rarity: string;
    isUsed: boolean;
    metadata: object;
  }>>;
  
  transferNFT(tokenId: string, toAddress: string): Promise<string>;
}

interface FinovaSecurityModule {
  // Security & Anti-Bot
  generateDeviceFingerprint(): Promise<string>;
  
  validateHumanBehavior(behaviorData: {
    clickPatterns: number[];
    sessionDuration: number;
    interactionTiming: number[];
    scrollPatterns: number[];
  }): Promise<{
    humanProbability: number;
    riskScore: number;
    isBot: boolean;
  }>;
  
  encryptSensitiveData(data: string): Promise<string>;
  decryptSensitiveData(encryptedData: string): Promise<string>;
  
  // KYC Operations
  captureKYCData(): Promise<{
    documentImage: string;
    selfieImage: string;
    livenessProof: string;
  }>;
  
  submitKYC(kycData: object): Promise<{
    submissionId: string;
    status: 'pending' | 'approved' | 'rejected';
  }>;
}

interface FinovaStakingModule {
  // Staking Operations
  stakeTokens(amount: number, duration: number): Promise<{
    stakingId: string;
    apr: number;
    maturityDate: number;
    sFINReceived: number;
  }>;
  
  unstakeTokens(stakingId: string): Promise<{
    amountReturned: number;
    rewardsEarned: number;
    penalty: number;
  }>;
  
  getStakingStatus(userId: string): Promise<{
    totalStaked: number;
    totalRewards: number;
    activeStakes: Array<{
      id: string;
      amount: number;
      apr: number;
      maturity: number;
    }>;
  }>;
  
  claimStakingRewards(stakingId: string): Promise<number>;
}

interface FinovaNotificationModule {
  // Push Notifications
  registerForNotifications(deviceToken: string): Promise<boolean>;
  
  subscribeToChannel(channel: 'mining' | 'rewards' | 'social' | 'nft'): Promise<boolean>;
  
  unsubscribeFromChannel(channel: string): Promise<boolean>;
  
  // In-App Notifications
  getNotifications(userId: string, limit: number): Promise<Array<{
    id: string;
    type: string;
    title: string;
    message: string;
    timestamp: number;
    isRead: boolean;
    data?: object;
  }>>;
  
  markNotificationRead(notificationId: string): Promise<boolean>;
}

// Native Module Instances
const { 
  FinovaWallet,
  FinovaMining,
  FinovaXP,
  FinovaReferral,
  FinovaNFT,
  FinovaSecurity,
  FinovaStaking,
  FinovaNotification
} = NativeModules;

// Type-safe exports with fallbacks
export const FinovaWalletModule: FinovaWalletModule = FinovaWallet || {
  generateWallet: () => Promise.reject(new Error('Wallet module not available')),
  importWallet: () => Promise.reject(new Error('Wallet module not available')),
  getBalance: () => Promise.reject(new Error('Wallet module not available')),
  signTransaction: () => Promise.reject(new Error('Wallet module not available')),
  authenticateWithBiometric: () => Promise.reject(new Error('Biometric not available')),
  isBiometricAvailable: () => Promise.resolve(false),
};

export const FinovaMiningModule: FinovaMiningModule = FinovaMining || {
  startMining: () => Promise.reject(new Error('Mining module not available')),
  stopMining: () => Promise.reject(new Error('Mining module not available')),
  getMiningStatus: () => Promise.reject(new Error('Mining module not available')),
  claimRewards: () => Promise.reject(new Error('Mining module not available')),
  calculateMiningRate: () => Promise.resolve(0),
};

export const FinovaXPModule: FinovaXPModule = FinovaXP || {
  recordActivity: () => Promise.reject(new Error('XP module not available')),
  getXPStatus: () => Promise.reject(new Error('XP module not available')),
  validateContent: () => Promise.resolve({ qualityScore: 1, isOriginal: true, engagementPrediction: 0 }),
};

export const FinovaReferralModule: FinovaReferralModule = FinovaReferral || {
  generateReferralCode: () => Promise.reject(new Error('Referral module not available')),
  processReferral: () => Promise.reject(new Error('Referral module not available')),
  getReferralStats: () => Promise.reject(new Error('Referral module not available')),
  getReferralNetwork: () => Promise.reject(new Error('Referral module not available')),
};

export const FinovaNFTModule: FinovaNFTModule = FinovaNFT || {
  mintSpecialCard: () => Promise.reject(new Error('NFT module not available')),
  useSpecialCard: () => Promise.reject(new Error('NFT module not available')),
  getUserNFTs: () => Promise.resolve([]),
  transferNFT: () => Promise.reject(new Error('NFT module not available')),
};

export const FinovaSecurityModule: FinovaSecurityModule = FinovaSecurity || {
  generateDeviceFingerprint: () => Promise.resolve('mock-fingerprint'),
  validateHumanBehavior: () => Promise.resolve({ humanProbability: 1, riskScore: 0, isBot: false }),
  encryptSensitiveData: (data: string) => Promise.resolve(data),
  decryptSensitiveData: (data: string) => Promise.resolve(data),
  captureKYCData: () => Promise.reject(new Error('KYC capture not available')),
  submitKYC: () => Promise.reject(new Error('KYC submission not available')),
};

export const FinovaStakingModule: FinovaStakingModule = FinovaStaking || {
  stakeTokens: () => Promise.reject(new Error('Staking module not available')),
  unstakeTokens: () => Promise.reject(new Error('Staking module not available')),
  getStakingStatus: () => Promise.reject(new Error('Staking module not available')),
  claimStakingRewards: () => Promise.reject(new Error('Staking module not available')),
};

export const FinovaNotificationModule: FinovaNotificationModule = FinovaNotification || {
  registerForNotifications: () => Promise.resolve(false),
  subscribeToChannel: () => Promise.resolve(false),
  unsubscribeFromChannel: () => Promise.resolve(false),
  getNotifications: () => Promise.resolve([]),
  markNotificationRead: () => Promise.resolve(false),
};

// Platform-specific checks
export const isNativeModuleAvailable = (moduleName: string): boolean => {
  const modules = NativeModules;
  return modules[moduleName] !== undefined;
};

// Error handling wrapper
export const safeNativeCall = async <T>(
  moduleCall: () => Promise<T>,
  fallback: T,
  errorMessage?: string
): Promise<T> => {
  try {
    return await moduleCall();
  } catch (error) {
    console.warn(errorMessage || 'Native module call failed:', error);
    return fallback;
  }
};

// Module availability checks
export const ModuleAvailability = {
  wallet: isNativeModuleAvailable('FinovaWallet'),
  mining: isNativeModuleAvailable('FinovaMining'),
  xp: isNativeModuleAvailable('FinovaXP'),
  referral: isNativeModuleAvailable('FinovaReferral'),
  nft: isNativeModuleAvailable('FinovaNFT'),
  security: isNativeModuleAvailable('FinovaSecurity'),
  staking: isNativeModuleAvailable('FinovaStaking'),
  notification: isNativeModuleAvailable('FinovaNotification'),
};

// Export all modules as default
export default {
  FinovaWalletModule,
  FinovaMiningModule,
  FinovaXPModule,
  FinovaReferralModule,
  FinovaNFTModule,
  FinovaSecurityModule,
  FinovaStakingModule,
  FinovaNotificationModule,
  ModuleAvailability,
  safeNativeCall,
};
