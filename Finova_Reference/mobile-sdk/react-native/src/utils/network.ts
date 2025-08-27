/**
 * Finova Network - React Native SDK Network Utils
 * Advanced networking with rate limiting, auth, retry logic, WebSocket, file upload
 * Enterprise-grade implementation for Social-Fi Super App
 * 
 * @version 1.0.0
 * @author Finova Network Team
 * @license MIT
 */

import { Platform } from 'react-native';

// ============================================================================
// TYPES & INTERFACES
// ============================================================================

export interface NetworkConfig {
  baseURL: string;
  apiVersion: string;
  timeout: number;
  retryAttempts: number;
  retryDelay: number;
  rateLimitWindow: number;
  rateLimitMax: number;
  enableWebSocket: boolean;
  wsReconnectDelay: number;
  maxFileSize: number;
  chunkSize: number;
}

export interface AuthTokens {
  accessToken: string;
  refreshToken: string;
  expiresAt: number;
}

export interface RequestConfig {
  method: 'GET' | 'POST' | 'PUT' | 'DELETE' | 'PATCH';
  endpoint: string;
  data?: any;
  headers?: Record<string, string>;
  requiresAuth?: boolean;
  skipRateLimit?: boolean;
  timeout?: number;
  retries?: number;
}

export interface UploadConfig {
  file: {
    uri: string;
    name: string;
    type: string;
    size?: number;
  };
  endpoint: string;
  onProgress?: (progress: number) => void;
  metadata?: Record<string, any>;
}

export interface WebSocketConfig {
  endpoint: string;
  protocols?: string[];
  reconnect?: boolean;
  heartbeatInterval?: number;
}

export interface RateLimitEntry {
  count: number;
  resetTime: number;
}

export interface NetworkResponse<T = any> {
  success: boolean;
  data?: T;
  error?: string;
  statusCode?: number;
  headers?: Record<string, string>;
}

// ============================================================================
// RATE LIMITER
// ============================================================================

class RateLimiter {
  private limits: Map<string, RateLimitEntry> = new Map();
  
  constructor(
    private maxRequests: number = 100,
    private windowMs: number = 60000 // 1 minute
  ) {}

  canMakeRequest(identifier: string): boolean {
    const now = Date.now();
    const entry = this.limits.get(identifier);

    if (!entry || now >= entry.resetTime) {
      this.limits.set(identifier, {
        count: 1,
        resetTime: now + this.windowMs
      });
      return true;
    }

    if (entry.count >= this.maxRequests) {
      return false;
    }

    entry.count++;
    return true;
  }

  getRemainingRequests(identifier: string): number {
    const entry = this.limits.get(identifier);
    if (!entry || Date.now() >= entry.resetTime) {
      return this.maxRequests;
    }
    return Math.max(0, this.maxRequests - entry.count);
  }

  getResetTime(identifier: string): number {
    const entry = this.limits.get(identifier);
    return entry?.resetTime || Date.now();
  }

  cleanup(): void {
    const now = Date.now();
    for (const [key, entry] of this.limits.entries()) {
      if (now >= entry.resetTime) {
        this.limits.delete(key);
      }
    }
  }
}

// ============================================================================
// WEBSOCKET MANAGER
// ============================================================================

class WebSocketManager {
  private ws: WebSocket | null = null;
  private reconnectAttempts = 0;
  private maxReconnectAttempts = 5;
  private heartbeatInterval: NodeJS.Timeout | null = null;
  private reconnectTimeout: NodeJS.Timeout | null = null;
  private eventListeners: Map<string, Function[]> = new Map();

  constructor(
    private url: string,
    private config: WebSocketConfig = {}
  ) {}

  connect(): Promise<void> {
    return new Promise((resolve, reject) => {
      try {
        this.ws = new WebSocket(this.url, this.config.protocols);
        
        this.ws.onopen = (event) => {
          console.log('[Finova WS] Connected');
          this.reconnectAttempts = 0;
          this.startHeartbeat();
          this.emit('connected', event);
          resolve();
        };

        this.ws.onmessage = (event) => {
          try {
            const data = JSON.parse(event.data);
            this.handleMessage(data);
          } catch (error) {
            console.error('[Finova WS] Message parse error:', error);
          }
        };

        this.ws.onclose = (event) => {
          console.log('[Finova WS] Disconnected:', event.code, event.reason);
          this.cleanup();
          this.emit('disconnected', event);
          
          if (this.config.reconnect && this.reconnectAttempts < this.maxReconnectAttempts) {
            this.scheduleReconnect();
          }
        };

        this.ws.onerror = (error) => {
          console.error('[Finova WS] Error:', error);
          this.emit('error', error);
          reject(error);
        };

      } catch (error) {
        reject(error);
      }
    });
  }

  disconnect(): void {
    this.config.reconnect = false;
    this.cleanup();
    if (this.ws) {
      this.ws.close(1000, 'Client disconnect');
      this.ws = null;
    }
  }

  send(data: any): boolean {
    if (this.ws?.readyState === WebSocket.OPEN) {
      try {
        this.ws.send(JSON.stringify(data));
        return true;
      } catch (error) {
        console.error('[Finova WS] Send error:', error);
        return false;
      }
    }
    return false;
  }

  on(event: string, callback: Function): void {
    if (!this.eventListeners.has(event)) {
      this.eventListeners.set(event, []);
    }
    this.eventListeners.get(event)!.push(callback);
  }

  off(event: string, callback?: Function): void {
    if (!callback) {
      this.eventListeners.delete(event);
      return;
    }
    
    const listeners = this.eventListeners.get(event);
    if (listeners) {
      const index = listeners.indexOf(callback);
      if (index > -1) {
        listeners.splice(index, 1);
      }
    }
  }

  private emit(event: string, data?: any): void {
    const listeners = this.eventListeners.get(event);
    if (listeners) {
      listeners.forEach(callback => callback(data));
    }
  }

  private handleMessage(data: any): void {
    // Handle different message types
    switch (data.type) {
      case 'mining_update':
        this.emit('mining_update', data.payload);
        break;
      case 'xp_gained':
        this.emit('xp_gained', data.payload);
        break;
      case 'referral_bonus':
        this.emit('referral_bonus', data.payload);
        break;
      case 'notification':
        this.emit('notification', data.payload);
        break;
      case 'heartbeat':
        this.send({ type: 'heartbeat_response' });
        break;
      default:
        this.emit('message', data);
    }
  }

  private startHeartbeat(): void {
    if (this.heartbeatInterval) {
      clearInterval(this.heartbeatInterval);
    }
    
    const interval = this.config.heartbeatInterval || 30000;
    this.heartbeatInterval = setInterval(() => {
      if (this.ws?.readyState === WebSocket.OPEN) {
        this.send({ type: 'heartbeat', timestamp: Date.now() });
      }
    }, interval);
  }

  private scheduleReconnect(): void {
    this.reconnectAttempts++;
    const delay = Math.min(1000 * Math.pow(2, this.reconnectAttempts), 30000);
    
    console.log(`[Finova WS] Reconnecting in ${delay}ms (attempt ${this.reconnectAttempts})`);
    
    this.reconnectTimeout = setTimeout(() => {
      this.connect().catch(error => {
        console.error('[Finova WS] Reconnect failed:', error);
      });
    }, delay);
  }

  private cleanup(): void {
    if (this.heartbeatInterval) {
      clearInterval(this.heartbeatInterval);
      this.heartbeatInterval = null;
    }
    
    if (this.reconnectTimeout) {
      clearTimeout(this.reconnectTimeout);
      this.reconnectTimeout = null;
    }
  }

  get isConnected(): boolean {
    return this.ws?.readyState === WebSocket.OPEN;
  }
}

// ============================================================================
// MAIN NETWORK CLIENT
// ============================================================================

export class FinovaNetworkClient {
  private config: NetworkConfig;
  private rateLimiter: RateLimiter;
  private wsManager: WebSocketManager | null = null;
  private authTokens: AuthTokens | null = null;
  private refreshPromise: Promise<boolean> | null = null;

  constructor(config: Partial<NetworkConfig> = {}) {
    this.config = {
      baseURL: 'https://api.finova.network',
      apiVersion: 'v1',
      timeout: 30000,
      retryAttempts: 3,
      retryDelay: 1000,
      rateLimitWindow: 60000,
      rateLimitMax: 100,
      enableWebSocket: true,
      wsReconnectDelay: 5000,
      maxFileSize: 10 * 1024 * 1024, // 10MB
      chunkSize: 1024 * 1024, // 1MB
      ...config
    };

    this.rateLimiter = new RateLimiter(
      this.config.rateLimitMax,
      this.config.rateLimitWindow
    );

    if (this.config.enableWebSocket) {
      this.initWebSocket();
    }

    // Cleanup rate limiter periodically
    setInterval(() => this.rateLimiter.cleanup(), 5 * 60 * 1000);
  }

  // ============================================================================
  // AUTHENTICATION
  // ============================================================================

  setAuthTokens(tokens: AuthTokens): void {
    this.authTokens = tokens;
  }

  clearAuthTokens(): void {
    this.authTokens = null;
  }

  private async refreshAccessToken(): Promise<boolean> {
    if (this.refreshPromise) {
      return this.refreshPromise;
    }

    if (!this.authTokens?.refreshToken) {
      return false;
    }

    this.refreshPromise = this.performTokenRefresh();
    const result = await this.refreshPromise;
    this.refreshPromise = null;
    return result;
  }

  private async performTokenRefresh(): Promise<boolean> {
    try {
      const response = await fetch(`${this.config.baseURL}/${this.config.apiVersion}/auth/refresh`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          'User-Agent': this.getUserAgent(),
        },
        body: JSON.stringify({
          refreshToken: this.authTokens!.refreshToken
        }),
        timeout: this.config.timeout
      });

      if (response.ok) {
        const data = await response.json();
        this.authTokens = {
          accessToken: data.accessToken,
          refreshToken: data.refreshToken || this.authTokens!.refreshToken,
          expiresAt: Date.now() + (data.expiresIn * 1000)
        };
        return true;
      }
      
      // Refresh failed, clear tokens
      this.authTokens = null;
      return false;
      
    } catch (error) {
      console.error('[Finova Network] Token refresh error:', error);
      this.authTokens = null;
      return false;
    }
  }

  private isTokenExpired(): boolean {
    if (!this.authTokens) return true;
    // Check if token expires in next 5 minutes
    return this.authTokens.expiresAt <= Date.now() + (5 * 60 * 1000);
  }

  // ============================================================================
  // HTTP REQUESTS
  // ============================================================================

  async request<T = any>(config: RequestConfig): Promise<NetworkResponse<T>> {
    const identifier = `${config.method}:${config.endpoint}`;
    
    // Rate limiting check
    if (!config.skipRateLimit && !this.rateLimiter.canMakeRequest(identifier)) {
      return {
        success: false,
        error: 'Rate limit exceeded',
        statusCode: 429
      };
    }

    // Token refresh if needed
    if (config.requiresAuth && this.isTokenExpired()) {
      const refreshed = await this.refreshAccessToken();
      if (!refreshed) {
        return {
          success: false,
          error: 'Authentication required',
          statusCode: 401
        };
      }
    }

    const retries = config.retries ?? this.config.retryAttempts;
    let lastError: any;

    for (let attempt = 0; attempt <= retries; attempt++) {
      try {
        const response = await this.performRequest(config);
        
        if (response.success || response.statusCode !== 500) {
          return response;
        }
        
        lastError = response.error;
        
      } catch (error) {
        lastError = error;
        console.error(`[Finova Network] Request attempt ${attempt + 1} failed:`, error);
      }

      // Wait before retry (exponential backoff)
      if (attempt < retries) {
        const delay = this.config.retryDelay * Math.pow(2, attempt);
        await this.sleep(delay);
      }
    }

    return {
      success: false,
      error: lastError?.message || 'Request failed after retries',
      statusCode: 500
    };
  }

  private async performRequest<T>(config: RequestConfig): Promise<NetworkResponse<T>> {
    const url = `${this.config.baseURL}/${this.config.apiVersion}${config.endpoint}`;
    const headers: Record<string, string> = {
      'Content-Type': 'application/json',
      'User-Agent': this.getUserAgent(),
      'X-Platform': Platform.OS,
      'X-App-Version': '1.0.0',
      ...config.headers
    };

    if (config.requiresAuth && this.authTokens?.accessToken) {
      headers['Authorization'] = `Bearer ${this.authTokens.accessToken}`;
    }

    try {
      const response = await fetch(url, {
        method: config.method,
        headers,
        body: config.data ? JSON.stringify(config.data) : undefined,
        timeout: config.timeout || this.config.timeout
      });

      const responseHeaders: Record<string, string> = {};
      response.headers.forEach((value, key) => {
        responseHeaders[key] = value;
      });

      let data: any;
      const contentType = response.headers.get('content-type');
      
      if (contentType?.includes('application/json')) {
        data = await response.json();
      } else {
        data = await response.text();
      }

      return {
        success: response.ok,
        data: response.ok ? data : undefined,
        error: response.ok ? undefined : data?.message || data || 'Request failed',
        statusCode: response.status,
        headers: responseHeaders
      };

    } catch (error: any) {
      return {
        success: false,
        error: error.message || 'Network error',
        statusCode: 0
      };
    }
  }

  // ============================================================================
  // FILE UPLOAD
  // ============================================================================

  async uploadFile(config: UploadConfig): Promise<NetworkResponse> {
    const { file, endpoint, onProgress, metadata } = config;

    // Validate file size
    if (file.size && file.size > this.config.maxFileSize) {
      return {
        success: false,
        error: `File too large. Maximum size: ${this.config.maxFileSize / (1024 * 1024)}MB`
      };
    }

    try {
      const formData = new FormData();
      formData.append('file', {
        uri: file.uri,
        name: file.name,
        type: file.type
      } as any);

      if (metadata) {
        Object.entries(metadata).forEach(([key, value]) => {
          formData.append(key, String(value));
        });
      }

      const url = `${this.config.baseURL}/${this.config.apiVersion}${endpoint}`;
      const headers: Record<string, string> = {
        'Content-Type': 'multipart/form-data',
        'User-Agent': this.getUserAgent()
      };

      if (this.authTokens?.accessToken) {
        headers['Authorization'] = `Bearer ${this.authTokens.accessToken}`;
      }

      // Use XMLHttpRequest for upload progress
      return new Promise((resolve) => {
        const xhr = new XMLHttpRequest();

        xhr.upload.onprogress = (event) => {
          if (event.lengthComputable && onProgress) {
            const progress = (event.loaded / event.total) * 100;
            onProgress(Math.round(progress));
          }
        };

        xhr.onload = () => {
          try {
            const data = JSON.parse(xhr.responseText);
            resolve({
              success: xhr.status >= 200 && xhr.status < 300,
              data: xhr.status >= 200 && xhr.status < 300 ? data : undefined,
              error: xhr.status >= 200 && xhr.status < 300 ? undefined : data?.message || 'Upload failed',
              statusCode: xhr.status
            });
          } catch (error) {
            resolve({
              success: false,
              error: 'Invalid response format',
              statusCode: xhr.status
            });
          }
        };

        xhr.onerror = () => {
          resolve({
            success: false,
            error: 'Upload failed',
            statusCode: 0
          });
        };

        xhr.open('POST', url);
        Object.entries(headers).forEach(([key, value]) => {
          if (key !== 'Content-Type') { // Let browser set content-type for FormData
            xhr.setRequestHeader(key, value);
          }
        });
        
        xhr.send(formData);
      });

    } catch (error: any) {
      return {
        success: false,
        error: error.message || 'Upload error'
      };
    }
  }

  // ============================================================================
  // WEBSOCKET METHODS
  // ============================================================================

  private initWebSocket(): void {
    const wsUrl = this.config.baseURL.replace('https://', 'wss://').replace('http://', 'ws://');
    this.wsManager = new WebSocketManager(`${wsUrl}/ws/${this.config.apiVersion}`, {
      reconnect: true,
      heartbeatInterval: 30000
    });
  }

  async connectWebSocket(): Promise<void> {
    if (!this.wsManager) {
      throw new Error('WebSocket not initialized');
    }
    return this.wsManager.connect();
  }

  disconnectWebSocket(): void {
    this.wsManager?.disconnect();
  }

  sendWebSocketMessage(data: any): boolean {
    return this.wsManager?.send(data) || false;
  }

  onWebSocketEvent(event: string, callback: Function): void {
    this.wsManager?.on(event, callback);
  }

  offWebSocketEvent(event: string, callback?: Function): void {
    this.wsManager?.off(event, callback);
  }

  get isWebSocketConnected(): boolean {
    return this.wsManager?.isConnected || false;
  }

  // ============================================================================
  // UTILITY METHODS
  // ============================================================================

  private getUserAgent(): string {
    return `FinovaApp/1.0.0 (${Platform.OS} ${Platform.Version}) ReactNative`;
  }

  private sleep(ms: number): Promise<void> {
    return new Promise(resolve => setTimeout(resolve, ms));
  }

  getRateLimitStatus(): { remaining: number; resetTime: number } {
    const identifier = 'general';
    return {
      remaining: this.rateLimiter.getRemainingRequests(identifier),
      resetTime: this.rateLimiter.getResetTime(identifier)
    };
  }

  // ============================================================================
  // FINOVA-SPECIFIC METHODS
  // ============================================================================

  // Mining-related requests
  async getMiningStatus(): Promise<NetworkResponse> {
    return this.request({
      method: 'GET',
      endpoint: '/mining/status',
      requiresAuth: true
    });
  }

  async startMining(): Promise<NetworkResponse> {
    return this.request({
      method: 'POST',
      endpoint: '/mining/start',
      requiresAuth: true
    });
  }

  async claimMiningRewards(): Promise<NetworkResponse> {
    return this.request({
      method: 'POST',
      endpoint: '/mining/claim',
      requiresAuth: true
    });
  }

  // XP System requests
  async submitActivity(activity: any): Promise<NetworkResponse> {
    return this.request({
      method: 'POST',
      endpoint: '/xp/activity',
      data: activity,
      requiresAuth: true
    });
  }

  async getXPStatus(): Promise<NetworkResponse> {
    return this.request({
      method: 'GET',
      endpoint: '/xp/status',
      requiresAuth: true
    });
  }

  // Referral System requests
  async getReferralStats(): Promise<NetworkResponse> {
    return this.request({
      method: 'GET',
      endpoint: '/referral/stats',
      requiresAuth: true
    });
  }

  async generateReferralCode(): Promise<NetworkResponse> {
    return this.request({
      method: 'POST',
      endpoint: '/referral/generate-code',
      requiresAuth: true
    });
  }

  // NFT and Special Cards
  async getUserNFTs(): Promise<NetworkResponse> {
    return this.request({
      method: 'GET',
      endpoint: '/nft/user-collection',
      requiresAuth: true
    });
  }

  async useSpecialCard(cardId: string): Promise<NetworkResponse> {
    return this.request({
      method: 'POST',
      endpoint: '/nft/use-card',
      data: { cardId },
      requiresAuth: true
    });
  }

  // Social Integration
  async connectSocialAccount(platform: string, token: string): Promise<NetworkResponse> {
    return this.request({
      method: 'POST',
      endpoint: '/social/connect',
      data: { platform, token },
      requiresAuth: true
    });
  }

  async uploadContent(content: UploadConfig): Promise<NetworkResponse> {
    return this.uploadFile({
      ...content,
      endpoint: '/social/upload-content'
    });
  }

  // User Profile
  async getUserProfile(): Promise<NetworkResponse> {
    return this.request({
      method: 'GET',
      endpoint: '/user/profile',
      requiresAuth: true
    });
  }

  async updateUserProfile(profileData: any): Promise<NetworkResponse> {
    return this.request({
      method: 'PUT',
      endpoint: '/user/profile',
      data: profileData,
      requiresAuth: true
    });
  }
}

// ============================================================================
// SINGLETON INSTANCE
// ============================================================================

let networkInstance: FinovaNetworkClient | null = null;

export const createNetworkClient = (config?: Partial<NetworkConfig>): FinovaNetworkClient => {
  if (!networkInstance) {
    networkInstance = new FinovaNetworkClient(config);
  }
  return networkInstance;
};

export const getNetworkClient = (): FinovaNetworkClient => {
  if (!networkInstance) {
    networkInstance = new FinovaNetworkClient();
  }
  return networkInstance;
};

// Export default instance
export default getNetworkClient();
