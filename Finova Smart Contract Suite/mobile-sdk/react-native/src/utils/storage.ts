// mobile-sdk/react-native/src/utils/storage.ts
import AsyncStorage from '@react-native-async-storage/async-storage';
import { FinovaCrypto } from './crypto';

export interface StorageOptions {
  encrypted?: boolean;
  expirationMs?: number;
}

interface StoredItem {
  data: any;
  timestamp: number;
  expirationMs?: number;
  encrypted: boolean;
}

export class FinovaStorage {
  private static encryptionKey: string | null = null;

  static async initialize(encryptionKey?: string): Promise<void> {
    if (encryptionKey) {
      this.encryptionKey = encryptionKey;
    } else {
      // Generate or retrieve encryption key
      let key = await AsyncStorage.getItem('@finova:encryption_key');
      if (!key) {
        key = FinovaCrypto.generateRandomString(32);
        await AsyncStorage.setItem('@finova:encryption_key', key);
      }
      this.encryptionKey = key;
    }
  }

  // Basic Storage Operations
  static async setItem(key: string, value: any, options: StorageOptions = {}): Promise<void> {
    try {
      const item: StoredItem = {
        data: value,
        timestamp: Date.now(),
        expirationMs: options.expirationMs,
        encrypted: options.encrypted || false
      };

      let serialized = JSON.stringify(item);
      
      if (options.encrypted && this.encryptionKey) {
        serialized = FinovaCrypto.encrypt(serialized, this.encryptionKey);
      }

      await AsyncStorage.setItem(`@finova:${key}`, serialized);
    } catch (error) {
      throw new Error(`Failed to store item: ${error}`);
    }
  }

  static async getItem<T>(key: string): Promise<T | null> {
    try {
      const stored = await AsyncStorage.getItem(`@finova:${key}`);
      if (!stored) return null;

      let item: StoredItem;
      
      try {
        // Try parsing as regular JSON first
        item = JSON.parse(stored);
      } catch {
        // If parsing fails, try decrypting first
        if (this.encryptionKey) {
          const decrypted = FinovaCrypto.decrypt(stored, this.encryptionKey);
          item = JSON.parse(decrypted);
        } else {
          throw new Error('Encrypted data found but no encryption key available');
        }
      }

      // Check expiration
      if (item.expirationMs && Date.now() - item.timestamp > item.expirationMs) {
        await this.removeItem(key);
        return null;
      }

      return item.data;
    } catch (error) {
      console.warn(`Failed to retrieve item ${key}:`, error);
      return null;
    }
  }

  static async removeItem(key: string): Promise<void> {
    try {
      await AsyncStorage.removeItem(`@finova:${key}`);
    } catch (error) {
      throw new Error(`Failed to remove item: ${error}`);
    }
  }

  static async clear(): Promise<void> {
    try {
      const allKeys = await AsyncStorage.getAllKeys();
      const finovaKeys = allKeys.filter(key => key.startsWith('@finova:'));
      await AsyncStorage.multiRemove(finovaKeys);
    } catch (error) {
      throw new Error(`Failed to clear storage: ${error}`);
    }
  }

  // Specialized Storage Methods
  static async setUserData(userData: any): Promise<void> {
    await this.setItem('user_data', userData, { encrypted: true });
  }

  static async getUserData(): Promise<any> {
    return await this.getItem('user_data');
  }

  static async setAuthToken(token: string, expirationMs: number = 3600000): Promise<void> {
    await this.setItem('auth_token', token, { encrypted: true, expirationMs });
  }

  static async getAuthToken(): Promise<string | null> {
    return await this.getItem('auth_token');
  }

  static async setRefreshToken(token: string, expirationMs: number = 2592000000): Promise<void> {
    await this.setItem('refresh_token', token, { encrypted: true, expirationMs });
  }

  static async getRefreshToken(): Promise<string | null> {
    return await this.getItem('refresh_token');
  }

  static async setMiningState(state: any): Promise<void> {
    await this.setItem('mining_state', state, { encrypted: false });
  }

  static async getMiningState(): Promise<any> {
    return await this.getItem('mining_state');
  }

  static async setXPData(xpData: any): Promise<void> {
    await this.setItem('xp_data', xpData, { encrypted: false });
  }

  static async getXPData(): Promise<any> {
    return await this.getItem('xp_data');
  }

  static async setRPData(rpData: any): Promise<void> {
    await this.setItem('rp_data', rpData, { encrypted: false });
  }

  static async getRPData(): Promise<any> {
    return await this.getItem('rp_data');
  }

  static async setAppSettings(settings: any): Promise<void> {
    await this.setItem('app_settings', settings, { encrypted: false });
  }

  static async getAppSettings(): Promise<any> {
    const defaultSettings = {
      notifications: true,
      biometricAuth: false,
      darkMode: false,
      language: 'en',
      currency: 'USD'
    };
    
    const stored = await this.getItem('app_settings');
    return { ...defaultSettings, ...stored };
  }

  static async setCachedData(key: string, data: any, expirationMs: number = 300000): Promise<void> {
    await this.setItem(`cache_${key}`, data, { encrypted: false, expirationMs });
  }

  static async getCachedData(key: string): Promise<any> {
    return await this.getItem(`cache_${key}`);
  }

  // Batch Operations
  static async setBatch(items: Array<{key: string, value: any, options?: StorageOptions}>): Promise<void> {
    await Promise.all(items.map(item => this.setItem(item.key, item.value, item.options)));
  }

  static async getBatch<T>(keys: string[]): Promise<Array<{key: string, value: T | null}>> {
    const results = await Promise.all(keys.map(async key => ({
      key,
      value: await this.getItem<T>(key)
    })));
    return results;
  }
}

// mobile-sdk/react-native/src/utils/network.ts
import { FINOVA_CONSTANTS, ERROR_CODES } from './constants';
import { FinovaStorage } from './storage';

export interface NetworkRequest {
  url: string;
  method?: 'GET' | 'POST' | 'PUT' | 'DELETE' | 'PATCH';
  headers?: Record<string, string>;
  body?: any;
  timeout?: number;
  retries?: number;
  requiresAuth?: boolean;
}

export interface NetworkResponse<T = any> {
  data: T;
  status: number;
  headers: Record<string, string>;
  success: boolean;
  error?: string;
  errorCode?: string;
}

export class FinovaNetwork {
  private static baseURL: string = FINOVA_CONSTANTS.FINOVA_API_BASE;
  private static defaultTimeout: number = 30000;
  private static rateLimitTracker: Map<string, number[]> = new Map();

  static setBaseURL(url: string): void {
    this.baseURL = url;
  }

  static async request<T = any>(config: NetworkRequest): Promise<NetworkResponse<T>> {
    const {
      url,
      method = 'GET',
      headers = {},
      body,
      timeout = this.defaultTimeout,
      retries = 3,
      requiresAuth = false
    } = config;

    // Check rate limiting
    if (!this.checkRateLimit(url)) {
      throw new Error('Rate limit exceeded');
    }

    // Add authentication if required
    if (requiresAuth) {
      const token = await FinovaStorage.getAuthToken();
      if (token) {
        headers['Authorization'] = `Bearer ${token}`;
      }
    }

    // Add default headers
    headers['Content-Type'] = headers['Content-Type'] || 'application/json';
    headers['User-Agent'] = 'FinovaSDK/1.0';

    const fullURL = url.startsWith('http') ? url : `${this.baseURL}${url}`;

    for (let attempt = 0; attempt <= retries; attempt++) {
      try {
        const controller = new AbortController();
        const timeoutId = setTimeout(() => controller.abort(), timeout);

        const response = await fetch(fullURL, {
          method,
          headers,
          body: body ? JSON.stringify(body) : undefined,
          signal: controller.signal
        });

        clearTimeout(timeoutId);

        const responseData = await response.json();

        if (response.ok) {
          return {
            data: responseData,
            status: response.status,
            headers: this.parseHeaders(response.headers),
            success: true
          };
        } else {
          // Handle specific error cases
          if (response.status === 401) {
            await this.handleAuthError();
          }

          return {
            data: responseData,
            status: response.status,
            headers: this.parseHeaders(response.headers),
            success: false,
            error: responseData.message || 'Request failed',
            errorCode: responseData.code || this.getErrorCodeFromStatus(response.status)
          };
        }
      } catch (error: any) {
        if (attempt === retries) {
          return {
            data: null as T,
            status: 0,
            headers: {},
            success: false,
            error: error.message || 'Network request failed',
            errorCode: this.getErrorCodeFromError(error)
          };
        }

        // Exponential backoff
        await this.delay(Math.pow(2, attempt) * 1000);
      }
    }

    throw new Error('Max retries exceeded');
  }

  // Convenience methods
  static async get<T>(url: string, requiresAuth: boolean = false): Promise<NetworkResponse<T>> {
    return this.request<T>({ url, method: 'GET', requiresAuth });
  }

  static async post<T>(url: string, body: any, requiresAuth: boolean = true): Promise<NetworkResponse<T>> {
    return this.request<T>({ url, method: 'POST', body, requiresAuth });
  }

  static async put<T>(url: string, body: any, requiresAuth: boolean = true): Promise<NetworkResponse<T>> {
    return this.request<T>({ url, method: 'PUT', body, requiresAuth });
  }

  static async delete<T>(url: string, requiresAuth: boolean = true): Promise<NetworkResponse<T>> {
    return this.request<T>({ url, method: 'DELETE', requiresAuth });
  }

  // Rate limiting
  private static checkRateLimit(url: string): boolean {
    const now = Date.now();
    const windowMs = 60000; // 1 minute
    const maxRequests = FINOVA_CONSTANTS.SECURITY.API_RATE_LIMIT;

    if (!this.rateLimitTracker.has(url)) {
      this.rateLimitTracker.set(url, []);
    }

    const requests = this.rateLimitTracker.get(url)!;
    
    // Remove old requests outside the window
    const validRequests = requests.filter(timestamp => now - timestamp < windowMs);
    
    if (validRequests.length >= maxRequests) {
      return false;
    }

    // Add current request
    validRequests.push(now);
    this.rateLimitTracker.set(url, validRequests);
    
    return true;
  }

  // Authentication handling
  private static async handleAuthError(): Promise<void> {
    const refreshToken = await FinovaStorage.getRefreshToken();
    
    if (refreshToken) {
      try {
        const response = await this.request({
          url: '/auth/refresh',
          method: 'POST',
          body: { refreshToken },
          requiresAuth: false
        });

        if (response.success) {
          await FinovaStorage.setAuthToken(response.data.accessToken);
          await FinovaStorage.setRefreshToken(response.data.refreshToken);
        } else {
          await this.clearAuthData();
        }
      } catch {
        await this.clearAuthData();
      }
    } else {
      await this.clearAuthData();
    }
  }

  private static async clearAuthData(): Promise<void> {
    await FinovaStorage.removeItem('auth_token');
    await FinovaStorage.removeItem('refresh_token');
    await FinovaStorage.removeItem('user_data');
  }

  // Helper methods
  private static parseHeaders(headers: Headers): Record<string, string> {
    const result: Record<string, string> = {};
    headers.forEach((value, key) => {
      result[key] = value;
    });
    return result;
  }

  private static getErrorCodeFromStatus(status: number): string {
    switch (status) {
      case 400: return ERROR_CODES.VALIDATION_INVALID_INPUT;
      case 401: return ERROR_CODES.AUTH_TOKEN_EXPIRED;
      case 403: return ERROR_CODES.AUTH_KYC_REQUIRED;
      case 429: return ERROR_CODES.NETWORK_RATE_LIMITED;
      case 500: return ERROR_CODES.NETWORK_SERVER_ERROR;
      default: return ERROR_CODES.NETWORK_CONNECTION_FAILED;
    }
  }

  private static getErrorCodeFromError(error: Error): string {
    if (error.name === 'AbortError') return ERROR_CODES.NETWORK_TIMEOUT;
    if (error.message.includes('network')) return ERROR_CODES.NETWORK_CONNECTION_FAILED;
    return ERROR_CODES.NETWORK_SERVER_ERROR;
  }

  private static delay(ms: number): Promise<void> {
    return new Promise(resolve => setTimeout(resolve, ms));
  }

  // WebSocket connection
  static createWebSocket(endpoint: string = FINOVA_CONSTANTS.WS_ENDPOINT): WebSocket {
    const ws = new WebSocket(endpoint);
    
    ws.onopen = () => {
      console.log('WebSocket connected');
    };

    ws.onclose = (event) => {
      console.log('WebSocket disconnected:', event.code, event.reason);
    };

    ws.onerror = (error) => {
      console.error('WebSocket error:', error);
    };

    return ws;
  }

  // File upload
  static async uploadFile(file: File, endpoint: string = '/upload'): Promise<NetworkResponse<{url: string}>> {
    const formData = new FormData();
    formData.append('file', file);

    const token = await FinovaStorage.getAuthToken();
    const headers: Record<string, string> = {};
    
    if (token) {
      headers['Authorization'] = `Bearer ${token}`;
    }

    try {
      const response = await fetch(`${this.baseURL}${endpoint}`, {
        method: 'POST',
        headers,
        body: formData
      });

      const data = await response.json();

      return {
        data,
        status: response.status,
        headers: this.parseHeaders(response.headers),
        success: response.ok,
        error: response.ok ? undefined : data.message,
        errorCode: response.ok ? undefined : data.code
      };
    } catch (error: any) {
      return {
        data: null as any,
        status: 0,
        headers: {},
        success: false,
        error: error.message,
        errorCode: ERROR_CODES.NETWORK_CONNECTION_FAILED
      };
    }
  }
}

// mobile-sdk/react-native/src/utils/biometric.ts
import ReactNativeBiometrics, { BiometryTypes } from 'react-native-biometrics';
import { FinovaStorage } from './storage';

export interface BiometricConfig {
  promptMessage?: string;
  cancelButtonText?: string;
  fallbackEnabled?: boolean;
}

export interface BiometricResult {
  success: boolean;
  error?: string;
  signature?: string;
}

export class FinovaBiometric {
  private static rnBiometrics = new ReactNativeBiometrics({
    allowDeviceCredentials: true
  });

  // Check biometric availability
  static async isBiometricAvailable(): Promise<{
    available: boolean;
    biometryType?: BiometryTypes;
    error?: string;
  }> {
    try {
      const { available, biometryType, error } = await this.rnBiometrics.isSensorAvailable();
      return { available, biometryType, error };
    } catch (error: any) {
      return { available: false, error: error.message };
    }
  }

  // Create biometric keys
  static async createKeys(): Promise<{ publicKey?: string; success: boolean; error?: string }> {
    try {
      const { publicKey } = await this.rnBiometrics.createKeys();
      await FinovaStorage.setItem('biometric_public_key', publicKey, { encrypted: true });
      return { publicKey, success: true };
    } catch (error: any) {
      return { success: false, error: error.message };
    }
  }

  // Delete biometric keys
  static async deleteKeys(): Promise<{ success: boolean; error?: string }> {
    try {
      const { keysDeleted } = await this.rnBiometrics.deleteKeys();
      if (keysDeleted) {
        await FinovaStorage.removeItem('biometric_public_key');
      }
      return { success: keysDeleted };
    } catch (error: any) {
      return { success: false, error: error.message };
    }
  }

  // Biometric authentication
  static async authenticate(config: BiometricConfig = {}): Promise<BiometricResult> {
    const {
      promptMessage = 'Authenticate with biometrics',
      cancelButtonText = 'Cancel',
      fallbackEnabled = true
    } = config;

    try {
      const { success, error } = await this.rnBiometrics.simplePrompt({
        promptMessage,
        cancelButtonText,
        fallbackEnabled
      });

      return { success, error };
    } catch (error: any) {
      return { success: false, error: error.message };
    }
  }

  // Biometric signature
  static async createSignature(payload: string, config: BiometricConfig = {}): Promise<BiometricResult> {
    const {
      promptMessage = 'Sign transaction with biometrics',
      cancelButtonText = 'Cancel'
    } = config;

    try {
      const { success, signature, error } = await this.rnBiometrics.createSignature({
        promptMessage,
        cancelButtonText,
        payload
      });

      return { success, signature, error };
    } catch (error: any) {
      return { success: false, error: error.message };
    }
  }

  // Verify signature
  static async verifySignature(signature: string, payload: string): Promise<boolean> {
    try {
      const publicKey = await FinovaStorage.getItem('biometric_public_key');
      if (!publicKey) return false;

      // In a real implementation, you would verify the signature against the public key
      // For this example, we'll simulate verification
      return signature.length > 0 && payload.length > 0;
    } catch {
      return false;
    }
  }

  // Biometric login
  static async biometricLogin(): Promise<{ success: boolean; token?: string; error?: string }> {
    try {
      const authResult = await this.authenticate({
        promptMessage: 'Login with biometrics'
      });

      if (!authResult.success) {
        return { success: false, error: authResult.error };
      }

      // Generate login payload
      const timestamp = Date.now().toString();
      const signatureResult = await this.createSignature(timestamp, {
        promptMessage: 'Confirm biometric login'
      });

      if (!signatureResult.success) {
        return { success: false, error: signatureResult.error };
      }

      // In a real implementation, send signature to server for verification
      // For now, we'll simulate successful login
      const mockToken = 'biometric_login_token_' + timestamp;
      
      return { success: true, token: mockToken };
    } catch (error: any) {
      return { success: false, error: error.message };
    }
  }

  // Enable biometric authentication
  static async enableBiometricAuth(): Promise<{ success: boolean; error?: string }> {
    try {
      const availability = await this.isBiometricAvailable();
      if (!availability.available) {
        return { success: false, error: 'Biometric authentication not available' };
      }

      const keyResult = await this.createKeys();
      if (!keyResult.success) {
        return { success: false, error: keyResult.error };
      }

      await FinovaStorage.setItem('biometric_enabled', true, { encrypted: false });
      return { success: true };
    } catch (error: any) {
      return { success: false, error: error.message };
    }
  }

  // Disable biometric authentication
  static async disableBiometricAuth(): Promise<{ success: boolean; error?: string }> {
    try {
      await this.deleteKeys();
      await FinovaStorage.setItem('biometric_enabled', false, { encrypted: false });
      return { success: true };
    } catch (error: any) {
      return { success: false, error: error.message };
    }
  }

  // Check if biometric is enabled
  static async isBiometricEnabled(): Promise<boolean> {
    try {
      const enabled = await FinovaStorage.getItem('biometric_enabled');
      return enabled === true;
    } catch {
      return false;
    }
  }

  // Biometric transaction signing
  static async signTransaction(transactionData: any): Promise<BiometricResult> {
    try {
      const payload = JSON.stringify(transactionData);
      const result = await this.createSignature(payload, {
        promptMessage: 'Sign transaction with biometrics'
      });

      return result;
    } catch (error: any) {
      return { success: false, error: error.message };
    }
  }

  // Get biometric info
  static async getBiometricInfo(): Promise<{
    available: boolean;
    enabled: boolean;
    biometryType?: BiometryTypes;
    hasKeys: boolean;
  }> {
    const availability = await this.isBiometricAvailable();
    const enabled = await this.isBiometricEnabled();
    const publicKey = await FinovaStorage.getItem('biometric_public_key');

    return {
      available: availability.available,
      enabled,
      biometryType: availability.biometryType,
      hasKeys: !!publicKey
    };
  }
}

// mobile-sdk/react-native/src/utils/logger.ts
import { FinovaStorage } from './storage';

export enum LogLevel {
  DEBUG = 0,
  INFO = 1,
  WARN = 2,
  ERROR = 3
}

export interface LogEntry {
  level: LogLevel;
  message: string;
  timestamp: number;
  category?: string;
  data?: any;
  userId?: string;
  sessionId?: string;
}

export class FinovaLogger {
  private static logLevel: LogLevel = __DEV__ ? LogLevel.DEBUG : LogLevel.WARN;
  private static logs: LogEntry[] = [];
  private static maxLogs: number = 1000;
  private static sessionId: string = this.generateSessionId();

  static setLogLevel(level: LogLevel): void {
    this.logLevel = level;
  }

  static generateSessionId(): string {
    return Date.now().toString(36) + Math.random().toString(36).substr(2);
  }

  private static shouldLog(level: LogLevel): boolean {
    return level >= this.logLevel;
  }

  private static async log(level: LogLevel, message: string, category?: string, data?: any): Promise<void> {
    if (!this.shouldLog(level)) return;

    const entry: LogEntry = {
      level,
      message,
      timestamp: Date.now(),
      category,
      data,
      sessionId: this.sessionId
    };

    // Add to memory logs
    this.logs.push(entry);
    if (this.logs.length > this.maxLogs) {
      this.logs.shift();
    }

    // Console output in development
    if (__DEV__) {
      const prefix = `[${LogLevel[level]}] ${category ? `[${category}] ` : ''}`;
      switch (level) {
        case LogLevel.DEBUG:
          console.debug(prefix + message, data);
          break;
        case LogLevel.INFO:
          console.info(prefix + message, data);
          break;
        case LogLevel.WARN:
          console.warn(prefix + message, data);
          break;
        case LogLevel.ERROR:
          console.error(prefix + message, data);
          break;
      }
    }

    // Store critical logs
    if (level >= LogLevel.ERROR) {
      await this.storeCriticalLog(entry);
    }
  }

  static debug(message: string, category?: string, data?: any): void {
    this.log(LogLevel.DEBUG, message, category, data);
  }

  static info(message: string, category?: string, data?: any): void {
    this.log(LogLevel.INFO, message, category, data);
  }

  static warn(message: string, category?: string, data?: any): void {
    this.log(LogLevel.WARN, message, category, data);
  }

  static error(message: string, category?: string, data?: any): void {
    this.log(LogLevel.ERROR, message, category, data);
  }

  // Specialized logging methods
  static apiRequest(method: string, url: string, status?: number, duration?: number): void {
    this.info(`${method} ${url} ${status ? `- ${status}` : ''} ${duration ? `(${duration}ms)` : ''}`, 'API');
  }

  static apiError(method: string, url: string, error: string, status?: number): void {
    this.error(`${method} ${url} - ${error}`, 'API', { status });
  }

  static miningActivity(action: string, data?: any): void {
    this.info(`Mining: ${action}`, 'MINING', data);
  }

  static xpActivity(action: string, xpGained?: number, data?: any): void {
    this.info(`XP: ${action}${xpGained ? ` (+${xpGained} XP)` : ''}`, 'XP', data);
  }

  static rpActivity(action: string, rpGained?: number, data?: any): void {
    this.info(`RP: ${action}${rpGained ? ` (+${rpGained} RP)` : ''}`, 'RP', data);
  }

  static socialActivity(platform: string, action: string, data?: any): void {
    this.info(`${platform}: ${action}`, 'SOCIAL', data);
  }

  static securityEvent(event: string, data?: any): void {
    this.warn(`Security: ${event}`, 'SECURITY', data);
  }

  static biometricActivity(action: string, success: boolean, error?: string): void {
    const message = `Biometric ${action}: ${success ? 'SUCCESS' : 'FAILED'}${error ? ` - ${error}` : ''}`;
    if (success) {
      this.info(message, 'BIOMETRIC');
    } else {
      this.warn(message, 'BIOMETRIC');
    }
  }

  // Log management
  static getLogs(category?: string, level?: LogLevel, limit?: number): LogEntry[] {
    let filteredLogs = this.logs;

    if (category) {
      filteredLogs = filteredLogs.filter(log => log.category === category);
    }

    if (level !== undefined) {
      filteredLogs = filteredLogs.filter(log => log.level >= level);
    }

    if (limit) {
      filteredLogs = filteredLogs.slice(-limit);
    }

    return filteredLogs;
  }

  static clearLogs(): void {
    this.logs = [];
  }

  private static async storeCriticalLog(entry: LogEntry): Promise<void> {
    try {
      const criticalLogs = await FinovaStorage.getItem('critical_logs') || [];
      criticalLogs.push(entry);
      
      // Keep only last 100 critical logs
      if (criticalLogs.length > 100) {
        criticalLogs.splice(0, criticalLogs.length - 100);
      }

      await FinovaStorage.setItem('critical_logs', criticalLogs, { encrypted: false });
    } catch (error) {
      console.error('Failed to store critical log:', error);
    }
  }

  static async getCriticalLogs(): Promise<LogEntry[]> {
    try {
      return await FinovaStorage.getItem('critical_logs') || [];
    } catch {
      return [];
    }
  }

  static async clearCriticalLogs(): Promise<void> {
    try {
      await FinovaStorage.removeItem('critical_logs');
    } catch (error) {
      console.error('Failed to clear critical logs:', error);
    }
  }

  // Performance logging
  static startTimer(label: string): () => void {
    const start = Date.now();
    return () => {
      const duration = Date.now() - start;
      this.debug(`Timer: ${label} completed in ${duration}ms`, 'PERFORMANCE');
    };
  }

  // Export logs for debugging
  static exportLogs(): string {
    const logs = this.logs.map(log => ({
      timestamp: new Date(log.timestamp).toISOString(),
      level: LogLevel[log.level],
      category: log.category,
      message: log.message,
      data: log.data
    }));

    return JSON.stringify(logs, null, 2);
  }
}
