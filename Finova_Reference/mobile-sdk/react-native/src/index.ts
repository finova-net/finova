/**
 * Finova Network React Native SDK
 * Main entry point for the complete Social-Fi Super App SDK
 * Version: 3.0.0
 * Built for enterprise-grade React Native applications
 */

import { NativeModules, NativeEventEmitter, Platform } from 'react-native';
import type {
  FinovaConfig,
  User,
  MiningStatus,
  XPActivity,
  ReferralData,
  NFTCard,
  StakingInfo,
  GuildInfo,
  TransactionResult,
  WebSocketEvent,
  BiometricVerification,
  QualityScore,
  NetworkStats
} from './types';

const { FinovaReactNative } = NativeModules;
const eventEmitter = new NativeEventEmitter(FinovaReactNative);

export interface FinovaSDKConfig {
  apiKey: string;
  environment: 'development' | 'staging' | 'production';
  solanaRpcUrl: string;
  enableBiometrics?: boolean;
  enableWebSocket?: boolean;
  enableAIQuality?: boolean;
  walletIntegration?: {
    ovo?: boolean;
    gopay?: boolean;
    dana?: boolean;
    shopeepay?: boolean;
  };
}

export class FinovaSDK {
  private config: FinovaSDKConfig;
  private isInitialized: boolean = false;
  private currentUser: User | null = null;
  private wsConnection: WebSocket | null = null;
  private eventListeners: Map<string, Function[]> = new Map();

  constructor(config: FinovaSDKConfig) {
    this.config = config;
  }

  // ============ INITIALIZATION ============
  async initialize(): Promise<boolean> {
    try {
      if (this.isInitialized) return true;

      // Initialize native module
      const result = await FinovaReactNative.initialize({
        apiKey: this.config.apiKey,
        environment: this.config.environment,
        solanaRpcUrl: this.config.solanaRpcUrl,
        enableBiometrics: this.config.enableBiometrics || false,
        platform: Platform.OS
      });

      if (result.success) {
        this.isInitialized = true;
        
        // Setup WebSocket if enabled
        if (this.config.enableWebSocket) {
          await this.setupWebSocket();
        }

        // Setup event listeners
        this.setupNativeEventListeners();
        
        return true;
      }
      
      throw new Error(`SDK initialization failed: ${result.error}`);
    } catch (error) {
      console.error('FinovaSDK initialization error:', error);
      return false;
    }
  }

  // ============ USER AUTHENTICATION ============
  async authenticateUser(phoneNumber: string, biometricData?: BiometricVerification): Promise<User> {
    this.ensureInitialized();
    
    try {
      const authData = {
        phoneNumber,
        biometricData,
        deviceInfo: await this.getDeviceInfo()
      };

      const result = await FinovaReactNative.authenticateUser(authData);
      
      if (result.success) {
        this.currentUser = result.user;
        this.emit('userAuthenticated', result.user);
        return result.user;
      }
      
      throw new Error(`Authentication failed: ${result.error}`);
    } catch (error) {
      console.error('User authentication error:', error);
      throw error;
    }
  }

  async performKYC(kycData: {
    idNumber: string;
    fullName: string;
    dateOfBirth: string;
    address: string;
    selfieImage: string;
    idCardImage: string;
  }): Promise<boolean> {
    this.ensureInitialized();
    
    try {
      const result = await FinovaReactNative.performKYC({
        ...kycData,
        userId: this.currentUser?.id
      });

      if (result.success && this.currentUser) {
        this.currentUser.isKYCVerified = true;
        this.emit('kycCompleted', this.currentUser);
      }

      return result.success;
    } catch (error) {
      console.error('KYC verification error:', error);
      return false;
    }
  }

  // ============ MINING SYSTEM ============
  async startMining(): Promise<MiningStatus> {
    this.ensureInitialized();
    
    try {
      const result = await FinovaReactNative.startMining({
        userId: this.currentUser?.id
      });

      if (result.success) {
        this.emit('miningStarted', result.miningStatus);
        return result.miningStatus;
      }
      
      throw new Error(`Mining start failed: ${result.error}`);
    } catch (error) {
      console.error('Mining start error:', error);
      throw error;
    }
  }

  async stopMining(): Promise<MiningStatus> {
    this.ensureInitialized();
    
    try {
      const result = await FinovaReactNative.stopMining({
        userId: this.currentUser?.id
      });

      if (result.success) {
        this.emit('miningStopped', result.miningStatus);
        return result.miningStatus;
      }
      
      throw new Error(`Mining stop failed: ${result.error}`);
    } catch (error) {
      console.error('Mining stop error:', error);
      throw error;
    }
  }

  async getMiningStatus(): Promise<MiningStatus> {
    this.ensureInitialized();
    
    try {
      const result = await FinovaReactNative.getMiningStatus({
        userId: this.currentUser?.id
      });

      return result.miningStatus;
    } catch (error) {
      console.error('Get mining status error:', error);
      throw error;
    }
  }

  // Calculate current mining rate based on all factors
  calculateMiningRate(): number {
    if (!this.currentUser) return 0;

    const baseRate = this.getBaseMiningRate();
    const pioneerBonus = this.calculatePioneerBonus();
    const referralBonus = this.calculateReferralBonus();
    const securityBonus = this.currentUser.isKYCVerified ? 1.2 : 0.8;
    const xpMultiplier = this.calculateXPMultiplier();
    const regressionFactor = Math.exp(-0.001 * (this.currentUser.totalFIN || 0));

    return baseRate * pioneerBonus * referralBonus * securityBonus * xpMultiplier * regressionFactor;
  }

  // ============ XP SYSTEM ============
  async recordXPActivity(activity: XPActivity): Promise<{ xpGained: number; newLevel: number }> {
    this.ensureInitialized();
    
    try {
      // Calculate XP with quality analysis
      const qualityScore = this.config.enableAIQuality 
        ? await this.analyzeContentQuality(activity.content)
        : { score: 1.0, factors: {} };

      const xpData = {
        ...activity,
        userId: this.currentUser?.id,
        qualityScore: qualityScore.score,
        timestamp: Date.now()
      };

      const result = await FinovaReactNative.recordXPActivity(xpData);
      
      if (result.success) {
        this.emit('xpGained', {
          activity,
          xpGained: result.xpGained,
          newLevel: result.newLevel,
          qualityScore
        });

        // Update current user data
        if (this.currentUser) {
          this.currentUser.xpLevel = result.newLevel;
          this.currentUser.totalXP = (this.currentUser.totalXP || 0) + result.xpGained;
        }

        return {
          xpGained: result.xpGained,
          newLevel: result.newLevel
        };
      }
      
      throw new Error(`XP recording failed: ${result.error}`);
    } catch (error) {
      console.error('XP activity recording error:', error);
      throw error;
    }
  }

  async getXPLeaderboard(timeframe: 'daily' | 'weekly' | 'monthly' | 'all'): Promise<User[]> {
    this.ensureInitialized();
    
    try {
      const result = await FinovaReactNative.getXPLeaderboard({ timeframe });
      return result.leaderboard || [];
    } catch (error) {
      console.error('Get XP leaderboard error:', error);
      return [];
    }
  }

  // ============ REFERRAL SYSTEM ============
  async generateReferralCode(): Promise<string> {
    this.ensureInitialized();
    
    try {
      const result = await FinovaReactNative.generateReferralCode({
        userId: this.currentUser?.id
      });

      return result.referralCode;
    } catch (error) {
      console.error('Generate referral code error:', error);
      throw error;
    }
  }

  async useReferralCode(referralCode: string): Promise<boolean> {
    this.ensureInitialized();
    
    try {
      const result = await FinovaReactNative.useReferralCode({
        userId: this.currentUser?.id,
        referralCode
      });

      if (result.success) {
        this.emit('referralUsed', { referralCode, bonus: result.bonus });
      }

      return result.success;
    } catch (error) {
      console.error('Use referral code error:', error);
      return false;
    }
  }

  async getReferralData(): Promise<ReferralData> {
    this.ensureInitialized();
    
    try {
      const result = await FinovaReactNative.getReferralData({
        userId: this.currentUser?.id
      });

      return result.referralData;
    } catch (error) {
      console.error('Get referral data error:', error);
      throw error;
    }
  }

  // ============ NFT & SPECIAL CARDS ============
  async getUserNFTs(): Promise<NFTCard[]> {
    this.ensureInitialized();
    
    try {
      const result = await FinovaReactNative.getUserNFTs({
        userId: this.currentUser?.id
      });

      return result.nfts || [];
    } catch (error) {
      console.error('Get user NFTs error:', error);
      return [];
    }
  }

  async useSpecialCard(cardId: string): Promise<{ success: boolean; effect: any }> {
    this.ensureInitialized();
    
    try {
      const result = await FinovaReactNative.useSpecialCard({
        userId: this.currentUser?.id,
        cardId
      });

      if (result.success) {
        this.emit('cardUsed', { cardId, effect: result.effect });
      }

      return result;
    } catch (error) {
      console.error('Use special card error:', error);
      throw error;
    }
  }

  async purchaseCard(cardType: string, finAmount: number): Promise<NFTCard> {
    this.ensureInitialized();
    
    try {
      const result = await FinovaReactNative.purchaseCard({
        userId: this.currentUser?.id,
        cardType,
        finAmount
      });

      if (result.success) {
        this.emit('cardPurchased', result.card);
        return result.card;
      }
      
      throw new Error(`Card purchase failed: ${result.error}`);
    } catch (error) {
      console.error('Purchase card error:', error);
      throw error;
    }
  }

  // ============ STAKING SYSTEM ============
  async stakeTokens(amount: number, duration: number): Promise<StakingInfo> {
    this.ensureInitialized();
    
    try {
      const result = await FinovaReactNative.stakeTokens({
        userId: this.currentUser?.id,
        amount,
        duration
      });

      if (result.success) {
        this.emit('tokensStaked', result.stakingInfo);
        return result.stakingInfo;
      }
      
      throw new Error(`Staking failed: ${result.error}`);
    } catch (error) {
      console.error('Stake tokens error:', error);
      throw error;
    }
  }

  async unstakeTokens(stakingId: string): Promise<TransactionResult> {
    this.ensureInitialized();
    
    try {
      const result = await FinovaReactNative.unstakeTokens({
        userId: this.currentUser?.id,
        stakingId
      });

      if (result.success) {
        this.emit('tokensUnstaked', result.transaction);
      }

      return result.transaction;
    } catch (error) {
      console.error('Unstake tokens error:', error);
      throw error;
    }
  }

  async getStakingInfo(): Promise<StakingInfo[]> {
    this.ensureInitialized();
    
    try {
      const result = await FinovaReactNative.getStakingInfo({
        userId: this.currentUser?.id
      });

      return result.stakingInfo || [];
    } catch (error) {
      console.error('Get staking info error:', error);
      return [];
    }
  }

  // ============ GUILD SYSTEM ============
  async createGuild(guildData: {
    name: string;
    description: string;
    isPrivate: boolean;
    maxMembers: number;
  }): Promise<GuildInfo> {
    this.ensureInitialized();
    
    try {
      const result = await FinovaReactNative.createGuild({
        ...guildData,
        founderId: this.currentUser?.id
      });

      if (result.success) {
        this.emit('guildCreated', result.guild);
        return result.guild;
      }
      
      throw new Error(`Guild creation failed: ${result.error}`);
    } catch (error) {
      console.error('Create guild error:', error);
      throw error;
    }
  }

  async joinGuild(guildId: string, inviteCode?: string): Promise<boolean> {
    this.ensureInitialized();
    
    try {
      const result = await FinovaReactNative.joinGuild({
        userId: this.currentUser?.id,
        guildId,
        inviteCode
      });

      if (result.success) {
        this.emit('guildJoined', { guildId });
      }

      return result.success;
    } catch (error) {
      console.error('Join guild error:', error);
      return false;
    }
  }

  // ============ E-WALLET INTEGRATION ============
  async connectWallet(walletType: 'ovo' | 'gopay' | 'dana' | 'shopeepay'): Promise<boolean> {
    this.ensureInitialized();
    
    try {
      const result = await FinovaReactNative.connectWallet({
        userId: this.currentUser?.id,
        walletType
      });

      if (result.success) {
        this.emit('walletConnected', { walletType });
      }

      return result.success;
    } catch (error) {
      console.error('Connect wallet error:', error);
      return false;
    }
  }

  async withdrawToWallet(amount: number, walletType: string): Promise<TransactionResult> {
    this.ensureInitialized();
    
    try {
      const result = await FinovaReactNative.withdrawToWallet({
        userId: this.currentUser?.id,
        amount,
        walletType
      });

      if (result.success) {
        this.emit('withdrawalCompleted', result.transaction);
      }

      return result.transaction;
    } catch (error) {
      console.error('Wallet withdrawal error:', error);
      throw error;
    }
  }

  // ============ SOCIAL INTEGRATION ============
  async connectSocialAccount(platform: string, accessToken: string): Promise<boolean> {
    this.ensureInitialized();
    
    try {
      const result = await FinovaReactNative.connectSocialAccount({
        userId: this.currentUser?.id,
        platform,
        accessToken
      });

      if (result.success) {
        this.emit('socialAccountConnected', { platform });
      }

      return result.success;
    } catch (error) {
      console.error('Connect social account error:', error);
      return false;
    }
  }

  async syncSocialActivity(): Promise<XPActivity[]> {
    this.ensureInitialized();
    
    try {
      const result = await FinovaReactNative.syncSocialActivity({
        userId: this.currentUser?.id
      });

      if (result.success && result.activities) {
        // Process each activity for XP
        for (const activity of result.activities) {
          await this.recordXPActivity(activity);
        }
      }

      return result.activities || [];
    } catch (error) {
      console.error('Sync social activity error:', error);
      return [];
    }
  }

  // ============ ANALYTICS & INSIGHTS ============
  async getUserAnalytics(timeframe: string): Promise<any> {
    this.ensureInitialized();
    
    try {
      const result = await FinovaReactNative.getUserAnalytics({
        userId: this.currentUser?.id,
        timeframe
      });

      return result.analytics;
    } catch (error) {
      console.error('Get user analytics error:', error);
      return {};
    }
  }

  async getNetworkStats(): Promise<NetworkStats> {
    this.ensureInitialized();
    
    try {
      const result = await FinovaReactNative.getNetworkStats();
      return result.networkStats;
    } catch (error) {
      console.error('Get network stats error:', error);
      throw error;
    }
  }

  // ============ UTILITY FUNCTIONS ============
  private ensureInitialized(): void {
    if (!this.isInitialized) {
      throw new Error('FinovaSDK must be initialized before use');
    }
  }

  private async getDeviceInfo(): Promise<any> {
    return {
      platform: Platform.OS,
      version: Platform.Version,
      model: await FinovaReactNative.getDeviceModel(),
      timestamp: Date.now()
    };
  }

  private getBaseMiningRate(): number {
    // Phase-based mining rates
    const phases = {
      1: { users: 100000, rate: 0.1 },
      2: { users: 1000000, rate: 0.05 },
      3: { users: 10000000, rate: 0.025 },
      4: { users: Infinity, rate: 0.01 }
    };

    // This would come from network stats in real implementation
    const totalUsers = 50000; // Placeholder
    
    for (const phase of Object.values(phases)) {
      if (totalUsers < phase.users) {
        return phase.rate;
      }
    }
    
    return 0.01; // Phase 4 default
  }

  private calculatePioneerBonus(): number {
    const totalUsers = 50000; // This should come from network stats
    return Math.max(1.0, 2.0 - (totalUsers / 1000000));
  }

  private calculateReferralBonus(): number {
    const activeReferrals = this.currentUser?.activeReferrals || 0;
    return 1 + (activeReferrals * 0.1);
  }

  private calculateXPMultiplier(): number {
    if (!this.currentUser) return 1.0;
    const level = this.currentUser.xpLevel || 1;
    return 1.0 + (level / 100) * 4.0; // Max 5.0x at level 100
  }

  private async analyzeContentQuality(content: string): Promise<QualityScore> {
    if (!this.config.enableAIQuality) {
      return { score: 1.0, factors: {} };
    }

    try {
      const result = await FinovaReactNative.analyzeContentQuality({ content });
      return result.qualityScore;
    } catch (error) {
      console.error('Content quality analysis error:', error);
      return { score: 1.0, factors: {} };
    }
  }

  // ============ WEBSOCKET & REAL-TIME ============
  private async setupWebSocket(): Promise<void> {
    if (!this.config.enableWebSocket) return;

    try {
      const wsUrl = `wss://api.finova.network/ws?token=${this.config.apiKey}`;
      this.wsConnection = new WebSocket(wsUrl);

      this.wsConnection.onopen = () => {
        console.log('Finova WebSocket connected');
        this.emit('wsConnected', {});
      };

      this.wsConnection.onmessage = (event) => {
        try {
          const data: WebSocketEvent = JSON.parse(event.data);
          this.handleWebSocketEvent(data);
        } catch (error) {
          console.error('WebSocket message parsing error:', error);
        }
      };

      this.wsConnection.onerror = (error) => {
        console.error('WebSocket error:', error);
        this.emit('wsError', error);
      };

      this.wsConnection.onclose = () => {
        console.log('WebSocket connection closed');
        this.emit('wsDisconnected', {});
        
        // Attempt reconnection after 5 seconds
        setTimeout(() => this.setupWebSocket(), 5000);
      };
    } catch (error) {
      console.error('WebSocket setup error:', error);
    }
  }

  private handleWebSocketEvent(event: WebSocketEvent): void {
    switch (event.type) {
      case 'miningUpdate':
        this.emit('miningUpdate', event.data);
        break;
      case 'xpGained':
        this.emit('realTimeXP', event.data);
        break;
      case 'referralJoined':
        this.emit('referralUpdate', event.data);
        break;
      case 'cardReceived':
        this.emit('cardReceived', event.data);
        break;
      case 'guildNotification':
        this.emit('guildUpdate', event.data);
        break;
      default:
        this.emit('unknownEvent', event);
    }
  }

  // ============ EVENT SYSTEM ============
  private setupNativeEventListeners(): void {
    // Listen to native events from iOS/Android
    eventEmitter.addListener('FinovaEvent', (event) => {
      this.emit(event.type, event.data);
    });
  }

  on(eventName: string, callback: Function): void {
    if (!this.eventListeners.has(eventName)) {
      this.eventListeners.set(eventName, []);
    }
    this.eventListeners.get(eventName)?.push(callback);
  }

  off(eventName: string, callback?: Function): void {
    if (!callback) {
      this.eventListeners.delete(eventName);
      return;
    }
    
    const listeners = this.eventListeners.get(eventName);
    if (listeners) {
      const index = listeners.indexOf(callback);
      if (index > -1) {
        listeners.splice(index, 1);
      }
    }
  }

  private emit(eventName: string, data: any): void {
    const listeners = this.eventListeners.get(eventName);
    if (listeners) {
      listeners.forEach(callback => {
        try {
          callback(data);
        } catch (error) {
          console.error(`Error in event listener for ${eventName}:`, error);
        }
      });
    }
  }

  // ============ CLEANUP ============
  destroy(): void {
    // Close WebSocket connection
    if (this.wsConnection) {
      this.wsConnection.close();
      this.wsConnection = null;
    }

    // Clear all event listeners
    this.eventListeners.clear();

    // Reset state
    this.isInitialized = false;
    this.currentUser = null;

    // Call native cleanup
    if (FinovaReactNative?.cleanup) {
      FinovaReactNative.cleanup();
    }
  }

  // ============ GETTERS ============
  get user(): User | null {
    return this.currentUser;
  }

  get isConnected(): boolean {
    return this.isInitialized && this.currentUser !== null;
  }

  get miningRate(): number {
    return this.calculateMiningRate();
  }
}

// Export types and utilities
export * from './types';
export * from './utils';

// Export singleton instance creator
let sdkInstance: FinovaSDK | null = null;

export const createFinovaSDK = (config: FinovaSDKConfig): FinovaSDK => {
  if (sdkInstance) {
    sdkInstance.destroy();
  }
  
  sdkInstance = new FinovaSDK(config);
  return sdkInstance;
};

export const getFinovaSDK = (): FinovaSDK | null => {
  return sdkInstance;
};

export default FinovaSDK;
