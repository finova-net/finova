import { NativeModules, NativeEventEmitter, Platform } from 'react-native';
import AsyncStorage from '@react-native-async-storage/async-storage';

// Native Module Interface
const { FinovaReactNative } = NativeModules;
const eventEmitter = new NativeEventEmitter(FinovaReactNative);

// Types
export interface FinovaConfig {
  apiUrl: string;
  wsUrl: string;
  solanaRpc: string;
  environment: 'development' | 'staging' | 'production';
  enableBiometric?: boolean;
  enableAnalytics?: boolean;
}

export interface User {
  id: string;
  publicKey: string;
  xpLevel: number;
  rpTier: 'Explorer' | 'Connector' | 'Influencer' | 'Leader' | 'Ambassador';
  miningRate: number;
  totalFIN: number;
  stakedFIN: number;
  isKYCVerified: boolean;
  referralCode: string;
  networkSize: number;
}

export interface MiningStats {
  currentRate: number;
  totalMined: number;
  phase: 'Finizen' | 'Growth' | 'Maturity' | 'Stability';
  isActive: boolean;
  nextClaim: Date;
  dailyLimit: number;
  earnedToday: number;
}

export interface XPActivity {
  type: 'post' | 'comment' | 'like' | 'share' | 'login' | 'quest' | 'viral';
  platform: 'instagram' | 'tiktok' | 'youtube' | 'facebook' | 'twitter' | 'app';
  content?: string;
  baseXP: number;
  multiplier: number;
  qualityScore?: number;
}

export interface RPNetwork {
  directReferrals: number;
  activeReferrals: number;
  totalNetworkSize: number;
  networkQuality: number;
  currentRP: number;
  tierMultiplier: number;
}

export interface SpecialCard {
  id: string;
  type: 'mining' | 'xp' | 'referral';
  rarity: 'Common' | 'Uncommon' | 'Rare' | 'Epic' | 'Legendary';
  effect: string;
  duration: number;
  price: number;
  isOwned: boolean;
  isActive: boolean;
}

// Main SDK Class
export class FinovaSDK {
  private config: FinovaConfig;
  private user: User | null = null;
  private apiKey: string | null = null;
  private socket: WebSocket | null = null;
  private miningInterval: NodeJS.Timeout | null = null;
  private listeners: Map<string, Function[]> = new Map();

  constructor(config: FinovaConfig) {
    this.config = config;
    this.initializeNativeModule();
    this.setupEventListeners();
  }

  // Initialization
  private async initializeNativeModule(): Promise<void> {
    try {
      await FinovaReactNative.initialize(this.config);
    } catch (error) {
      console.error('Failed to initialize native module:', error);
    }
  }

  private setupEventListeners(): void {
    eventEmitter.addListener('MiningUpdate', (data) => {
      this.emit('miningUpdate', data);
    });

    eventEmitter.addListener('XPGained', (data) => {
      this.emit('xpGained', data);
    });

    eventEmitter.addListener('RPUpdated', (data) => {
      this.emit('rpUpdated', data);
    });

    eventEmitter.addListener('BiometricResult', (data) => {
      this.emit('biometricResult', data);
    });
  }

  // Authentication & User Management
  public async initializeUser(referralCode?: string): Promise<User> {
    try {
      const stored = await AsyncStorage.getItem('finova_user');
      if (stored) {
        this.user = JSON.parse(stored);
        await this.authenticateUser();
        return this.user!;
      }

      // Create new user
      const walletData = await this.generateWallet();
      const userData = await this.registerUser(walletData, referralCode);
      
      this.user = userData;
      await AsyncStorage.setItem('finova_user', JSON.stringify(userData));
      
      this.startMiningSession();
      return userData;
    } catch (error) {
      throw new Error(`User initialization failed: ${error}`);
    }
  }

  private async generateWallet(): Promise<any> {
    return await FinovaReactNative.generateWallet();
  }

  private async registerUser(walletData: any, referralCode?: string): Promise<User> {
    const response = await this.apiCall('/user/register', {
      method: 'POST',
      body: JSON.stringify({
        publicKey: walletData.publicKey,
        referralCode,
        platform: Platform.OS,
        deviceId: await this.getDeviceId()
      })
    });

    if (!response.success) {
      throw new Error(response.error || 'Registration failed');
    }

    return response.user;
  }

  private async authenticateUser(): Promise<void> {
    if (!this.user) throw new Error('No user to authenticate');

    const signature = await this.signMessage('finova_auth');
    const response = await this.apiCall('/auth/verify', {
      method: 'POST',
      body: JSON.stringify({
        publicKey: this.user.publicKey,
        signature,
        timestamp: Date.now()
      })
    });

    if (response.success) {
      this.apiKey = response.token;
      this.connectWebSocket();
      this.startMiningSession();
    }
  }

  public async performKYC(selfieData: string, idData: string): Promise<boolean> {
    try {
      const biometricHash = await FinovaReactNative.processBiometric(selfieData);
      
      const response = await this.apiCall('/kyc/submit', {
        method: 'POST',
        body: JSON.stringify({
          biometricHash,
          idData,
          userId: this.user?.id
        })
      });

      if (response.success && this.user) {
        this.user.isKYCVerified = true;
        await AsyncStorage.setItem('finova_user', JSON.stringify(this.user));
        this.emit('kycCompleted', { verified: true });
      }

      return response.success;
    } catch (error) {
      console.error('KYC process failed:', error);
      return false;
    }
  }

  // Mining System
  public async startMining(): Promise<MiningStats> {
    if (!this.user) throw new Error('User not initialized');

    const stats = await this.getMiningStats();
    
    if (!stats.isActive) {
      await this.apiCall('/mining/start', { method: 'POST' });
      this.startMiningSession();
    }

    return stats;
  }

  public async stopMining(): Promise<void> {
    await this.apiCall('/mining/stop', { method: 'POST' });
    if (this.miningInterval) {
      clearInterval(this.miningInterval);
      this.miningInterval = null;
    }
  }

  public async getMiningStats(): Promise<MiningStats> {
    const response = await this.apiCall('/mining/stats', { method: 'GET' });
    return response.stats;
  }

  public async claimMiningRewards(): Promise<number> {
    const response = await this.apiCall('/mining/claim', { method: 'POST' });
    
    if (response.success && this.user) {
      const claimed = response.amount;
      this.user.totalFIN += claimed;
      await AsyncStorage.setItem('finova_user', JSON.stringify(this.user));
      this.emit('rewardsClaimed', { amount: claimed, total: this.user.totalFIN });
      return claimed;
    }
    
    throw new Error(response.error || 'Claim failed');
  }

  private startMiningSession(): void {
    if (this.miningInterval) return;

    this.miningInterval = setInterval(async () => {
      try {
        const stats = await this.getMiningStats();
        this.emit('miningUpdate', stats);
        
        // Auto-claim when ready
        if (stats.earnedToday > 0 && stats.nextClaim <= new Date()) {
          await this.claimMiningRewards();
        }
      } catch (error) {
        console.error('Mining session error:', error);
      }
    }, 60000); // Check every minute
  }

  // XP System
  public async submitActivity(activity: Omit<XPActivity, 'baseXP' | 'multiplier'>): Promise<{ xpGained: number; newLevel?: number }> {
    try {
      const response = await this.apiCall('/xp/submit', {
        method: 'POST',
        body: JSON.stringify({
          ...activity,
          userId: this.user?.id,
          timestamp: Date.now()
        })
      });

      if (response.success && this.user) {
        const { xpGained, newLevel, totalXP } = response;
        
        if (newLevel && newLevel > this.user.xpLevel) {
          this.user.xpLevel = newLevel;
          await AsyncStorage.setItem('finova_user', JSON.stringify(this.user));
          this.emit('levelUp', { oldLevel: this.user.xpLevel, newLevel, totalXP });
        }

        this.emit('xpGained', { amount: xpGained, activity: activity.type, platform: activity.platform });
        return { xpGained, newLevel };
      }

      throw new Error(response.error || 'Activity submission failed');
    } catch (error) {
      console.error('XP submission error:', error);
      throw error;
    }
  }

  public async getXPHistory(limit: number = 50): Promise<any[]> {
    const response = await this.apiCall(`/xp/history?limit=${limit}`, { method: 'GET' });
    return response.history || [];
  }

  // Referral System  
  public async getReferralStats(): Promise<RPNetwork> {
    const response = await this.apiCall('/referral/stats', { method: 'GET' });
    return response.network;
  }

  public async generateReferralLink(): Promise<string> {
    if (!this.user?.referralCode) throw new Error('No referral code available');
    
    const response = await this.apiCall('/referral/link', {
      method: 'POST',
      body: JSON.stringify({ code: this.user.referralCode })
    });

    return response.link;
  }

  public async getReferralRewards(): Promise<{ total: number; pending: number; claimed: number }> {
    const response = await this.apiCall('/referral/rewards', { method: 'GET' });
    return response.rewards;
  }

  // NFT & Special Cards
  public async getAvailableCards(): Promise<SpecialCard[]> {
    const response = await this.apiCall('/nft/cards/available', { method: 'GET' });
    return response.cards || [];
  }

  public async purchaseCard(cardId: string): Promise<boolean> {
    const response = await this.apiCall('/nft/cards/purchase', {
      method: 'POST',
      body: JSON.stringify({ cardId, userId: this.user?.id })
    });

    if (response.success) {
      this.emit('cardPurchased', { cardId, transaction: response.transaction });
    }

    return response.success;
  }

  public async useCard(cardId: string): Promise<boolean> {
    const response = await this.apiCall('/nft/cards/use', {
      method: 'POST',
      body: JSON.stringify({ cardId, userId: this.user?.id })
    });

    if (response.success) {
      this.emit('cardActivated', { cardId, effect: response.effect, duration: response.duration });
    }

    return response.success;
  }

  public async getOwnedCards(): Promise<SpecialCard[]> {
    const response = await this.apiCall('/nft/cards/owned', { method: 'GET' });
    return response.cards || [];
  }

  // Social Integration
  public async connectSocialPlatform(platform: string, credentials: any): Promise<boolean> {
    const response = await this.apiCall('/social/connect', {
      method: 'POST',
      body: JSON.stringify({
        platform,
        credentials,
        userId: this.user?.id
      })
    });

    if (response.success) {
      this.emit('platformConnected', { platform, status: 'connected' });
    }

    return response.success;
  }

  public async disconnectSocialPlatform(platform: string): Promise<boolean> {
    const response = await this.apiCall('/social/disconnect', {
      method: 'POST',
      body: JSON.stringify({ platform, userId: this.user?.id })
    });

    return response.success;
  }

  public async getConnectedPlatforms(): Promise<string[]> {
    const response = await this.apiCall('/social/connected', { method: 'GET' });
    return response.platforms || [];
  }

  // Staking
  public async stakeTokens(amount: number): Promise<{ success: boolean; txHash?: string }> {
    if (!this.user || this.user.totalFIN < amount) {
      throw new Error('Insufficient balance');
    }

    const response = await this.apiCall('/staking/stake', {
      method: 'POST',
      body: JSON.stringify({ amount, userId: this.user.id })
    });

    if (response.success && this.user) {
      this.user.totalFIN -= amount;
      this.user.stakedFIN += amount;
      await AsyncStorage.setItem('finova_user', JSON.stringify(this.user));
      this.emit('tokensStaked', { amount, total: this.user.stakedFIN });
    }

    return response;
  }

  public async unstakeTokens(amount: number): Promise<{ success: boolean; txHash?: string }> {
    const response = await this.apiCall('/staking/unstake', {
      method: 'POST',
      body: JSON.stringify({ amount, userId: this.user?.id })
    });

    if (response.success && this.user) {
      this.user.stakedFIN -= amount;
      this.user.totalFIN += amount;
      await AsyncStorage.setItem('finova_user', JSON.stringify(this.user));
      this.emit('tokensUnstaked', { amount, remaining: this.user.stakedFIN });
    }

    return response;
  }

  // WebSocket Connection
  private connectWebSocket(): void {
    if (!this.apiKey) return;

    const wsUrl = `${this.config.wsUrl}?token=${this.apiKey}`;
    this.socket = new WebSocket(wsUrl);

    this.socket.onopen = () => {
      this.emit('connected', { status: 'connected' });
      this.socket?.send(JSON.stringify({ type: 'subscribe', channels: ['mining', 'xp', 'referral'] }));
    };

    this.socket.onmessage = (event) => {
      try {
        const data = JSON.parse(event.data);
        this.handleWebSocketMessage(data);
      } catch (error) {
        console.error('WebSocket message parse error:', error);
      }
    };

    this.socket.onclose = () => {
      this.emit('disconnected', { status: 'disconnected' });
      // Attempt reconnection after 5 seconds
      setTimeout(() => this.connectWebSocket(), 5000);
    };

    this.socket.onerror = (error) => {
      console.error('WebSocket error:', error);
      this.emit('error', { error: 'WebSocket connection failed' });
    };
  }

  private handleWebSocketMessage(data: any): void {
    switch (data.type) {
      case 'mining_update':
        this.emit('miningUpdate', data.payload);
        break;
      case 'xp_gained':
        this.emit('xpGained', data.payload);
        break;
      case 'rp_updated':
        this.emit('rpUpdated', data.payload);
        break;
      case 'level_up':
        this.emit('levelUp', data.payload);
        break;
      case 'referral_success':
        this.emit('referralSuccess', data.payload);
        break;
      default:
        console.log('Unknown WebSocket message type:', data.type);
    }
  }

  // Utility Methods
  private async apiCall(endpoint: string, options: RequestInit): Promise<any> {
    const url = `${this.config.apiUrl}${endpoint}`;
    const headers = {
      'Content-Type': 'application/json',
      ...(this.apiKey && { 'Authorization': `Bearer ${this.apiKey}` }),
      ...options.headers
    };

    try {
      const response = await fetch(url, { ...options, headers });
      const data = await response.json();

      if (!response.ok) {
        throw new Error(data.error || `HTTP ${response.status}`);
      }

      return data;
    } catch (error) {
      console.error(`API call failed for ${endpoint}:`, error);
      throw error;
    }
  }

  private async signMessage(message: string): Promise<string> {
    return await FinovaReactNative.signMessage(message);
  }

  private async getDeviceId(): Promise<string> {
    return await FinovaReactNative.getDeviceId();
  }

  // Event System
  public on(event: string, callback: Function): void {
    if (!this.listeners.has(event)) {
      this.listeners.set(event, []);
    }
    this.listeners.get(event)!.push(callback);
  }

  public off(event: string, callback: Function): void {
    const listeners = this.listeners.get(event);
    if (listeners) {
      const index = listeners.indexOf(callback);
      if (index > -1) {
        listeners.splice(index, 1);
      }
    }
  }

  private emit(event: string, data: any): void {
    const listeners = this.listeners.get(event);
    if (listeners) {
      listeners.forEach(callback => callback(data));
    }
  }

  // Cleanup
  public destroy(): void {
    if (this.socket) {
      this.socket.close();
      this.socket = null;
    }

    if (this.miningInterval) {
      clearInterval(this.miningInterval);
      this.miningInterval = null;
    }

    eventEmitter.removeAllListeners();
    this.listeners.clear();
    this.user = null;
    this.apiKey = null;
  }

  // Getters
  public getCurrentUser(): User | null {
    return this.user;
  }

  public isAuthenticated(): boolean {
    return this.apiKey !== null && this.user !== null;
  }

  public getConfig(): FinovaConfig {
    return { ...this.config };
  }
}

// Default export
export default FinovaSDK;

// Additional utility functions
export const FinovaUtils = {
  calculateMiningRate: (baseRate: number, xpLevel: number, rpTier: string, stakingAmount: number): number => {
    const xpMultiplier = 1 + (xpLevel / 100);
    const rpMultiplier = getRPMultiplier(rpTier);
    const stakingMultiplier = Math.min(2.0, 1 + (stakingAmount / 10000));
    
    return baseRate * xpMultiplier * rpMultiplier * stakingMultiplier;
  },

  formatFIN: (amount: number): string => {
    return new Intl.NumberFormat('en-US', {
      minimumFractionDigits: 0,
      maximumFractionDigits: 6
    }).format(amount);
  },

  formatXP: (xp: number): string => {
    return new Intl.NumberFormat('en-US').format(xp);
  },

  getXPLevelFromTotal: (totalXP: number): number => {
    return Math.floor(Math.sqrt(totalXP / 100)) + 1;
  },

  getXPRequiredForLevel: (level: number): number => {
    return Math.pow(level - 1, 2) * 100;
  }
};

function getRPMultiplier(rpTier: string): number {
  const multipliers = {
    'Explorer': 1.0,
    'Connector': 1.2,
    'Influencer': 1.5,
    'Leader': 2.0,
    'Ambassador': 3.0
  };
  return multipliers[rpTier as keyof typeof multipliers] || 1.0;
}
