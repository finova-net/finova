import AsyncStorage from '@react-native-async-storage/async-storage';
import { Platform } from 'react-native';

// Log Levels
export enum LogLevel {
  DEBUG = 0,
  INFO = 1,
  WARN = 2,
  ERROR = 3,
  CRITICAL = 4,
}

// Log Categories for Finova Network
export enum LogCategory {
  MINING = 'MINING',
  XP_SYSTEM = 'XP_SYSTEM',
  REFERRAL = 'REFERRAL',
  NFT = 'NFT',
  SOCIAL_INTEGRATION = 'SOCIAL_INTEGRATION',
  AUTHENTICATION = 'AUTHENTICATION',
  API = 'API',
  BLOCKCHAIN = 'BLOCKCHAIN',
  PERFORMANCE = 'PERFORMANCE',
  SECURITY = 'SECURITY',
  USER_BEHAVIOR = 'USER_BEHAVIOR',
  BOT_DETECTION = 'BOT_DETECTION',
  WALLET = 'WALLET',
  GUILD = 'GUILD',
}

// Performance Metrics Interface
interface PerformanceMetrics {
  timestamp: number;
  category: LogCategory;
  action: string;
  duration: number;
  memoryUsage?: number;
  networkLatency?: number;
  cpuUsage?: number;
  batteryLevel?: number;
  connectionType?: string;
}

// Log Entry Interface
interface LogEntry {
  id: string;
  timestamp: number;
  level: LogLevel;
  category: LogCategory;
  message: string;
  data?: any;
  userId?: string;
  sessionId: string;
  platform: string;
  version: string;
  stackTrace?: string;
  performance?: PerformanceMetrics;
  sensitive?: boolean;
}

// Logger Configuration
interface LoggerConfig {
  enableConsoleLogging: boolean;
  enableAsyncStorage: boolean;
  enableRemoteLogging: boolean;
  maxStoredLogs: number;
  logLevel: LogLevel;
  apiEndpoint?: string;
  apiKey?: string;
  enablePerformanceMonitoring: boolean;
  enableCriticalStorage: boolean;
  batchSize: number;
  flushInterval: number;
  enableSensitiveDataMasking: boolean;
}

// Critical Log Storage
interface CriticalLogStorage {
  securityIncidents: LogEntry[];
  errorLogs: LogEntry[];
  performanceIssues: LogEntry[];
}

class FinovaLogger {
  private config: LoggerConfig;
  private logBuffer: LogEntry[] = [];
  private sessionId: string;
  private performanceTrackers: Map<string, number> = new Map();
  private criticalLogs: CriticalLogStorage = {
    securityIncidents: [],
    errorLogs: [],
    performanceIssues: [],
  };
  private flushTimer: NodeJS.Timeout | null = null;

  constructor(config: Partial<LoggerConfig> = {}) {
    this.config = {
      enableConsoleLogging: __DEV__,
      enableAsyncStorage: true,
      enableRemoteLogging: true,
      maxStoredLogs: 1000,
      logLevel: __DEV__ ? LogLevel.DEBUG : LogLevel.INFO,
      enablePerformanceMonitoring: true,
      enableCriticalStorage: true,
      batchSize: 50,
      flushInterval: 30000, // 30 seconds
      enableSensitiveDataMasking: true,
      ...config,
    };

    this.sessionId = this.generateSessionId();
    this.initializeLogger();
  }

  private generateSessionId(): string {
    return `${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
  }

  private async initializeLogger(): Promise<void> {
    try {
      // Load existing logs from storage
      await this.loadStoredLogs();
      
      // Start periodic flush
      this.startPeriodicFlush();
      
      // Log initialization
      this.info(LogCategory.API, 'Finova Logger initialized', {
        sessionId: this.sessionId,
        config: this.sanitizeConfig(),
      });
    } catch (error) {
      console.error('Failed to initialize Finova Logger:', error);
    }
  }

  private sanitizeConfig(): Partial<LoggerConfig> {
    const { apiKey, ...safeConfig } = this.config;
    return {
      ...safeConfig,
      apiKey: apiKey ? '***MASKED***' : undefined,
    };
  }

  private startPeriodicFlush(): void {
    if (this.flushTimer) {
      clearInterval(this.flushTimer);
    }

    this.flushTimer = setInterval(() => {
      this.flushLogs();
    }, this.config.flushInterval);
  }

  private async loadStoredLogs(): Promise<void> {
    try {
      const storedLogs = await AsyncStorage.getItem('@finova_logs');
      if (storedLogs) {
        this.logBuffer = JSON.parse(storedLogs);
        // Keep only recent logs
        this.logBuffer = this.logBuffer.slice(-this.config.maxStoredLogs);
      }

      const criticalLogs = await AsyncStorage.getItem('@finova_critical_logs');
      if (criticalLogs) {
        this.criticalLogs = JSON.parse(criticalLogs);
      }
    } catch (error) {
      console.error('Error loading stored logs:', error);
    }
  }

  private createLogEntry(
    level: LogLevel,
    category: LogCategory,
    message: string,
    data?: any,
    userId?: string,
    sensitive?: boolean
  ): LogEntry {
    const entry: LogEntry = {
      id: `${Date.now()}_${Math.random().toString(36).substr(2, 6)}`,
      timestamp: Date.now(),
      level,
      category,
      message: this.config.enableSensitiveDataMasking && sensitive 
        ? this.maskSensitiveData(message) 
        : message,
      data: this.config.enableSensitiveDataMasking && sensitive 
        ? this.maskSensitiveData(data) 
        : data,
      userId,
      sessionId: this.sessionId,
      platform: Platform.OS,
      version: '1.0.0', // Should come from app config
      sensitive,
    };

    // Add stack trace for errors
    if (level >= LogLevel.ERROR) {
      entry.stackTrace = new Error().stack;
    }

    return entry;
  }

  private maskSensitiveData(data: any): any {
    if (typeof data === 'string') {
      return data.replace(/(\b\w{2})\w+(\w{2}\b)/g, '$1***$2');
    }
    
    if (typeof data === 'object' && data !== null) {
      const masked = { ...data };
      const sensitiveKeys = ['password', 'token', 'privateKey', 'secret', 'apiKey', 'pin'];
      
      Object.keys(masked).forEach(key => {
        if (sensitiveKeys.some(sk => key.toLowerCase().includes(sk))) {
          masked[key] = '***MASKED***';
        }
      });
      
      return masked;
    }
    
    return data;
  }

  private shouldLog(level: LogLevel): boolean {
    return level >= this.config.logLevel;
  }

  private async addLogEntry(entry: LogEntry): Promise<void> {
    if (!this.shouldLog(entry.level)) return;

    // Add to buffer
    this.logBuffer.push(entry);

    // Store critical logs separately
    if (entry.level >= LogLevel.ERROR || entry.category === LogCategory.SECURITY) {
      await this.storeCriticalLog(entry);
    }

    // Console logging
    if (this.config.enableConsoleLogging) {
      this.logToConsole(entry);
    }

    // Auto-flush if buffer is full
    if (this.logBuffer.length >= this.config.batchSize) {
      await this.flushLogs();
    }

    // Store in AsyncStorage
    if (this.config.enableAsyncStorage) {
      await this.storeLogsLocal();
    }
  }

  private async storeCriticalLog(entry: LogEntry): Promise<void> {
    try {
      if (entry.category === LogCategory.SECURITY || entry.category === LogCategory.BOT_DETECTION) {
        this.criticalLogs.securityIncidents.push(entry);
        this.criticalLogs.securityIncidents = this.criticalLogs.securityIncidents.slice(-100);
      }

      if (entry.level >= LogLevel.ERROR) {
        this.criticalLogs.errorLogs.push(entry);
        this.criticalLogs.errorLogs = this.criticalLogs.errorLogs.slice(-100);
      }

      if (entry.category === LogCategory.PERFORMANCE) {
        this.criticalLogs.performanceIssues.push(entry);
        this.criticalLogs.performanceIssues = this.criticalLogs.performanceIssues.slice(-100);
      }

      await AsyncStorage.setItem('@finova_critical_logs', JSON.stringify(this.criticalLogs));
    } catch (error) {
      console.error('Error storing critical log:', error);
    }
  }

  private logToConsole(entry: LogEntry): void {
    const timestamp = new Date(entry.timestamp).toISOString();
    const logMessage = `[${timestamp}] [${LogLevel[entry.level]}] [${entry.category}] ${entry.message}`;
    
    switch (entry.level) {
      case LogLevel.DEBUG:
        console.log(`🔍 ${logMessage}`, entry.data);
        break;
      case LogLevel.INFO:
        console.info(`ℹ️ ${logMessage}`, entry.data);
        break;
      case LogLevel.WARN:
        console.warn(`⚠️ ${logMessage}`, entry.data);
        break;
      case LogLevel.ERROR:
        console.error(`❌ ${logMessage}`, entry.data);
        break;
      case LogLevel.CRITICAL:
        console.error(`🚨 ${logMessage}`, entry.data);
        break;
    }
  }

  private async storeLogsLocal(): Promise<void> {
    try {
      const logsToStore = this.logBuffer.slice(-this.config.maxStoredLogs);
      await AsyncStorage.setItem('@finova_logs', JSON.stringify(logsToStore));
    } catch (error) {
      console.error('Error storing logs locally:', error);
    }
  }

  private async flushLogs(): Promise<void> {
    if (this.logBuffer.length === 0 || !this.config.enableRemoteLogging) return;

    try {
      const logsToSend = [...this.logBuffer];
      this.logBuffer = [];

      if (this.config.apiEndpoint) {
        await this.sendLogsToServer(logsToSend);
      }
    } catch (error) {
      console.error('Error flushing logs:', error);
      // Restore logs to buffer if sending failed
      this.logBuffer.unshift(...this.logBuffer);
    }
  }

  private async sendLogsToServer(logs: LogEntry[]): Promise<void> {
    if (!this.config.apiEndpoint || !this.config.apiKey) return;

    const response = await fetch(`${this.config.apiEndpoint}/logs`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        'Authorization': `Bearer ${this.config.apiKey}`,
        'X-Session-ID': this.sessionId,
      },
      body: JSON.stringify({ logs }),
    });

    if (!response.ok) {
      throw new Error(`HTTP ${response.status}: ${response.statusText}`);
    }
  }

  // Performance Monitoring
  startPerformanceTracking(action: string): string {
    const trackerId = `${action}_${Date.now()}_${Math.random().toString(36).substr(2, 4)}`;
    this.performanceTrackers.set(trackerId, performance.now());
    return trackerId;
  }

  endPerformanceTracking(trackerId: string, category: LogCategory, additionalData?: any): void {
    const startTime = this.performanceTrackers.get(trackerId);
    if (!startTime) return;

    const duration = performance.now() - startTime;
    this.performanceTrackers.delete(trackerId);

    const metrics: PerformanceMetrics = {
      timestamp: Date.now(),
      category,
      action: trackerId.split('_')[0],
      duration,
      ...additionalData,
    };

    this.info(LogCategory.PERFORMANCE, `Performance: ${metrics.action}`, {
      duration: `${duration.toFixed(2)}ms`,
      metrics,
    });

    // Flag slow operations
    if (duration > 1000) {
      this.warn(LogCategory.PERFORMANCE, `Slow operation detected: ${metrics.action}`, metrics);
    }
  }

  // Finova-specific logging methods
  logMiningActivity(userId: string, action: string, data: any): void {
    this.info(LogCategory.MINING, `Mining: ${action}`, {
      userId,
      ...data,
    });
  }

  logXPGain(userId: string, activity: string, xpGained: number, multipliers: any): void {
    this.info(LogCategory.XP_SYSTEM, `XP gained: ${xpGained}`, {
      userId,
      activity,
      xpGained,
      multipliers,
    });
  }

  logReferralActivity(userId: string, action: string, referralData: any): void {
    this.info(LogCategory.REFERRAL, `Referral: ${action}`, {
      userId,
      ...referralData,
    });
  }

  logSocialIntegration(platform: string, action: string, data: any): void {
    this.info(LogCategory.SOCIAL_INTEGRATION, `${platform}: ${action}`, data);
  }

  logSecurityEvent(event: string, data: any, userId?: string): void {
    this.warn(LogCategory.SECURITY, `Security event: ${event}`, {
      userId,
      ...data,
    });
  }

  logBotDetection(userId: string, suspiciousActivity: string, score: number): void {
    this.warn(LogCategory.BOT_DETECTION, `Bot detection: ${suspiciousActivity}`, {
      userId,
      suspiciousActivity,
      botScore: score,
    });
  }

  logWalletOperation(userId: string, operation: string, amount?: number): void {
    this.info(LogCategory.WALLET, `Wallet: ${operation}`, {
      userId,
      operation,
      amount: amount ? `${amount} FIN` : undefined,
    }, false, true); // Mark as sensitive
  }

  // Public logging methods
  debug(category: LogCategory, message: string, data?: any): void {
    const entry = this.createLogEntry(LogLevel.DEBUG, category, message, data);
    this.addLogEntry(entry);
  }

  info(category: LogCategory, message: string, data?: any, userId?: string): void {
    const entry = this.createLogEntry(LogLevel.INFO, category, message, data, userId);
    this.addLogEntry(entry);
  }

  warn(category: LogCategory, message: string, data?: any, userId?: string): void {
    const entry = this.createLogEntry(LogLevel.WARN, category, message, data, userId);
    this.addLogEntry(entry);
  }

  error(category: LogCategory, message: string, error?: Error, userId?: string): void {
    const data = error ? {
      name: error.name,
      message: error.message,
      stack: error.stack,
    } : undefined;
    
    const entry = this.createLogEntry(LogLevel.ERROR, category, message, data, userId);
    this.addLogEntry(entry);
  }

  critical(category: LogCategory, message: string, data?: any, userId?: string): void {
    const entry = this.createLogEntry(LogLevel.CRITICAL, category, message, data, userId);
    this.addLogEntry(entry);
  }

  // Utility methods
  async getLogs(category?: LogCategory, level?: LogLevel): Promise<LogEntry[]> {
    let logs = [...this.logBuffer];
    
    if (category) {
      logs = logs.filter(log => log.category === category);
    }
    
    if (level !== undefined) {
      logs = logs.filter(log => log.level >= level);
    }
    
    return logs.sort((a, b) => b.timestamp - a.timestamp);
  }

  async getCriticalLogs(): Promise<CriticalLogStorage> {
    return { ...this.criticalLogs };
  }

  async clearLogs(): Promise<void> {
    this.logBuffer = [];
    this.criticalLogs = {
      securityIncidents: [],
      errorLogs: [],
      performanceIssues: [],
    };
    
    await AsyncStorage.multiRemove(['@finova_logs', '@finova_critical_logs']);
    this.info(LogCategory.API, 'Logs cleared');
  }

  async exportLogs(): Promise<string> {
    const exportData = {
      sessionId: this.sessionId,
      exportedAt: new Date().toISOString(),
      logs: this.logBuffer,
      criticalLogs: this.criticalLogs,
    };
    
    return JSON.stringify(exportData, null, 2);
  }

  // Cleanup
  destroy(): void {
    if (this.flushTimer) {
      clearInterval(this.flushTimer);
    }
    
    this.flushLogs(); // Final flush
  }
}

// Singleton instance
let loggerInstance: FinovaLogger | null = null;

export const initializeLogger = (config?: Partial<LoggerConfig>): FinovaLogger => {
  if (!loggerInstance) {
    loggerInstance = new FinovaLogger(config);
  }
  return loggerInstance;
};

export const getLogger = (): FinovaLogger => {
  if (!loggerInstance) {
    loggerInstance = new FinovaLogger();
  }
  return loggerInstance;
};

// Export types
export type { LoggerConfig, LogEntry, PerformanceMetrics, CriticalLogStorage };
export { FinovaLogger };
