import { Injectable, Logger } from '@nestjs/common';
import { InjectRepository } from '@nestjs/typeorm';
import { Repository } from 'typeorm';
import { Redis } from 'ioredis';
import * as crypto from 'crypto';
import * as tf from '@tensorflow/tfjs-node';

// Types & Interfaces
interface UserBehaviorData {
  userId: string;
  sessionId: string;
  timestamps: number[];
  clickPatterns: number[];
  deviceFingerprint: string;
  ipAddress: string;
  userAgent: string;
  biometricConsistency: number;
  contentPatterns: any[];
  networkConnections: string[];
  geolocation?: { lat: number; lng: number };
}

interface HumanProbabilityResult {
  score: number;
  confidence: number;
  riskFactors: string[];
  recommendations: string[];
  breakdown: {
    biometric: number;
    behavioral: number;
    network: number;
    device: number;
    temporal: number;
    content: number;
  };
}

interface SuspiciousActivity {
  type: 'CLICK_SPEED' | 'SESSION_PATTERN' | 'NETWORK_ABUSE' | 'CONTENT_SPAM' | 'DEVICE_FARM';
  severity: 'LOW' | 'MEDIUM' | 'HIGH' | 'CRITICAL';
  confidence: number;
  evidence: any;
}

@Injectable()
export class AntiBotService {
  private readonly logger = new Logger(AntiBotService.name);
  private behaviorModel: tf.LayersModel;
  private readonly HUMAN_THRESHOLD = 0.7;
  private readonly CACHE_TTL = 3600; // 1 hour

  constructor(
    @InjectRepository('UserBehavior') private behaviorRepo: Repository<any>,
    @InjectRepository('SuspiciousActivity') private suspiciousRepo: Repository<any>,
    private redis: Redis
  ) {
    this.initializeModels();
  }

  async initializeModels(): Promise<void> {
    try {
      this.behaviorModel = await tf.loadLayersModel('file://models/behavior_classifier.json');
      this.logger.log('AI behavior model loaded successfully');
    } catch (error) {
      this.logger.error('Failed to load AI model:', error);
    }
  }

  /**
   * Main human probability calculator with multi-factor analysis
   */
  async calculateHumanProbability(userData: UserBehaviorData): Promise<HumanProbabilityResult> {
    const cacheKey = `human_score:${userData.userId}:${userData.sessionId}`;
    const cached = await this.redis.get(cacheKey);
    
    if (cached) {
      return JSON.parse(cached);
    }

    const factors = {
      biometric: await this.analyzeBiometricConsistency(userData),
      behavioral: await this.detectBehavioralPatterns(userData),
      network: await this.validateSocialGraph(userData),
      device: await this.checkDeviceAuthenticity(userData),
      temporal: await this.analyzeTemporalPatterns(userData),
      content: await this.measureContentQuality(userData)
    };

    const weights = {
      biometric: 0.25,
      behavioral: 0.20,
      network: 0.15,
      device: 0.15,
      temporal: 0.15,
      content: 0.10
    };

    const weightedScore = Object.keys(factors).reduce((sum, key) => 
      sum + factors[key] * weights[key], 0
    );

    const result: HumanProbabilityResult = {
      score: Math.max(0.1, Math.min(1.0, weightedScore)),
      confidence: this.calculateConfidence(factors),
      riskFactors: this.identifyRiskFactors(factors),
      recommendations: this.generateRecommendations(factors),
      breakdown: factors
    };

    await this.redis.setex(cacheKey, this.CACHE_TTL, JSON.stringify(result));
    return result;
  }

  /**
   * Biometric consistency analysis using facial recognition patterns
   */
  private async analyzeBiometricConsistency(userData: UserBehaviorData): Promise<number> {
    try {
      const historicalBiometrics = await this.behaviorRepo.find({
        where: { userId: userData.userId },
        select: ['biometricHash', 'createdAt'],
        order: { createdAt: 'DESC' },
        take: 10
      });

      if (historicalBiometrics.length < 2) return 0.5;

      let consistencyScore = 0;
      for (let i = 1; i < historicalBiometrics.length; i++) {
        const similarity = this.calculateBiometricSimilarity(
          historicalBiometrics[0].biometricHash,
          historicalBiometrics[i].biometricHash
        );
        consistencyScore += similarity;
      }

      return Math.min(1.0, consistencyScore / (historicalBiometrics.length - 1));
    } catch (error) {
      this.logger.error('Biometric analysis failed:', error);
      return 0.5;
    }
  }

  /**
   * Advanced behavioral pattern detection using ML
   */
  private async detectBehavioralPatterns(userData: UserBehaviorData): Promise<number> {
    if (!this.behaviorModel || userData.timestamps.length < 10) return 0.5;

    try {
      // Analyze click intervals for human-like variance
      const clickIntervals = this.calculateClickIntervals(userData.timestamps);
      const intervalVariance = this.calculateVariance(clickIntervals);
      
      // Human clicking has natural variance (200-2000ms typical)
      const clickScore = this.scoreClickPatterns(intervalVariance);
      
      // Analyze session patterns
      const sessionScore = await this.analyzeSessionPatterns(userData);
      
      // Mouse movement entropy (if available)
      const movementScore = this.analyzeMouseMovement(userData.clickPatterns);
      
      // Combine behavioral signals
      const behaviorInput = tf.tensor2d([[
        clickScore,
        sessionScore,
        movementScore,
        intervalVariance / 1000, // normalize
        userData.timestamps.length / 100 // activity level
      ]]);

      const prediction = this.behaviorModel.predict(behaviorInput) as tf.Tensor;
      const humanScore = await prediction.data();
      
      behaviorInput.dispose();
      prediction.dispose();
      
      return humanScore[0];
    } catch (error) {
      this.logger.error('Behavioral analysis failed:', error);
      return 0.5;
    }
  }

  /**
   * Social graph validation for authentic connections
   */
  private async validateSocialGraph(userData: UserBehaviorData): Promise<number> {
    try {
      const networkConnections = userData.networkConnections || [];
      
      if (networkConnections.length === 0) return 0.3;
      
      // Check for circular referral patterns (bot networks)
      const circularPatterns = await this.detectCircularReferrals(userData.userId);
      if (circularPatterns > 0.3) return 0.2;
      
      // Analyze connection diversity
      const diversityScore = await this.calculateNetworkDiversity(networkConnections);
      
      // Check mutual connections authenticity
      const mutualityScore = await this.analyzeMutualConnections(networkConnections);
      
      // Account age vs connection ratio
      const accountAge = await this.getAccountAge(userData.userId);
      const ageConnectionRatio = Math.min(1.0, accountAge / (networkConnections.length * 7)); // 1 week per connection
      
      return (diversityScore * 0.4 + mutualityScore * 0.4 + ageConnectionRatio * 0.2);
    } catch (error) {
      this.logger.error('Network validation failed:', error);
      return 0.5;
    }
  }

  /**
   * Device authenticity verification
   */
  private async checkDeviceAuthenticity(userData: UserBehaviorData): Promise<number> {
    try {
      const deviceHash = crypto.createHash('sha256')
        .update(`${userData.deviceFingerprint}${userData.userAgent}`)
        .digest('hex');

      // Check for device farm patterns
      const deviceUsage = await this.redis.get(`device:${deviceHash}`);
      const usageCount = deviceUsage ? parseInt(deviceUsage) : 0;
      
      await this.redis.incr(`device:${deviceHash}`);
      await this.redis.expire(`device:${deviceHash}`, 86400); // 24 hours

      // Single device used by many accounts = suspicious
      if (usageCount > 10) return 0.1;
      if (usageCount > 5) return 0.3;
      
      // Analyze user agent consistency
      const uaScore = this.analyzeUserAgent(userData.userAgent);
      
      // Check geolocation consistency
      const geoScore = await this.analyzeGeolocation(userData);
      
      return Math.min(1.0, (uaScore * 0.5 + geoScore * 0.3 + (usageCount <= 2 ? 1.0 : 0.5) * 0.2));
    } catch (error) {
      this.logger.error('Device authenticity check failed:', error);
      return 0.5;
    }
  }

  /**
   * Circadian rhythm and temporal pattern analysis
   */
  private async analyzeTemporalPatterns(userData: UserBehaviorData): Promise<number> {
    try {
      const activities = await this.behaviorRepo.find({
        where: { userId: userData.userId },
        select: ['timestamp'],
        order: { timestamp: 'DESC' },
        take: 168 // 1 week of hourly data
      });

      if (activities.length < 24) return 0.5;

      // Extract hours from timestamps
      const hours = activities.map(a => new Date(a.timestamp).getHours());
      
      // Human activity follows circadian patterns (more active 8-22, less 23-7)
      const circadianScore = this.calculateCircadianCompliance(hours);
      
      // Check for 24/7 activity (suspicious)
      const continuousActivity = this.detectContinuousActivity(activities);
      
      // Weekend vs weekday patterns
      const weeklyPattern = this.analyzeWeeklyPatterns(activities);
      
      return (circadianScore * 0.5 + (1 - continuousActivity) * 0.3 + weeklyPattern * 0.2);
    } catch (error) {
      this.logger.error('Temporal analysis failed:', error);
      return 0.5;
    }
  }

  /**
   * Content quality and uniqueness measurement
   */
  private async measureContentQuality(userData: UserBehaviorData): Promise<number> {
    try {
      if (!userData.contentPatterns || userData.contentPatterns.length === 0) return 0.5;

      let qualityScores = [];
      
      for (const content of userData.contentPatterns) {
        // Check content uniqueness
        const uniquenessScore = await this.checkContentUniqueness(content);
        
        // Analyze content complexity
        const complexityScore = this.analyzeContentComplexity(content);
        
        // Check for spam patterns
        const spamScore = 1 - this.detectSpamPatterns(content);
        
        qualityScores.push(uniquenessScore * 0.4 + complexityScore * 0.3 + spamScore * 0.3);
      }

      return qualityScores.reduce((a, b) => a + b, 0) / qualityScores.length;
    } catch (error) {
      this.logger.error('Content quality analysis failed:', error);
      return 0.5;
    }
  }

  /**
   * Progressive difficulty and penalty system
   */
  async calculateProgressive​Penalties(userId: string, totalEarned: number, suspiciousScore: number): Promise<{
    miningPenalty: number;
    xpPenalty: number;
    rpPenalty: number;
  }> {
    const difficultyMultiplier = 1 + (totalEarned / 1000) + (suspiciousScore * 2);
    
    return {
      miningPenalty: Math.min(0.95, difficultyMultiplier * 0.1),
      xpPenalty: Math.min(0.90, difficultyMultiplier * 0.05),
      rpPenalty: Math.min(0.92, difficultyMultiplier * 0.08)
    };
  }

  /**
   * Real-time suspicious activity detection
   */
  async detectSuspiciousActivity(userData: UserBehaviorData): Promise<SuspiciousActivity[]> {
    const activities: SuspiciousActivity[] = [];

    // Click speed analysis
    if (userData.timestamps.length >= 5) {
      const avgInterval = this.calculateAverageInterval(userData.timestamps);
      if (avgInterval < 100) { // Less than 100ms between clicks
        activities.push({
          type: 'CLICK_SPEED',
          severity: 'HIGH',
          confidence: 0.9,
          evidence: { averageInterval: avgInterval }
        });
      }
    }

    // Session pattern detection
    const sessionDuration = userData.timestamps[userData.timestamps.length - 1] - userData.timestamps[0];
    if (sessionDuration > 86400000 && userData.timestamps.length > 1000) { // 24+ hours, 1000+ actions
      activities.push({
        type: 'SESSION_PATTERN',
        severity: 'CRITICAL',
        confidence: 0.95,
        evidence: { duration: sessionDuration, actions: userData.timestamps.length }
      });
    }

    // Network abuse detection
    const networkScore = await this.validateSocialGraph(userData);
    if (networkScore < 0.2) {
      activities.push({
        type: 'NETWORK_ABUSE',
        severity: 'HIGH',
        confidence: 1 - networkScore,
        evidence: { networkScore }
      });
    }

    return activities;
  }

  /**
   * Bulk user analysis for network-wide bot detection
   */
  async performNetworkAnalysis(): Promise<{
    suspiciousUsers: string[];
    botNetworks: string[][];
    recommendations: string[];
  }> {
    try {
      const recentUsers = await this.behaviorRepo
        .createQueryBuilder('ub')
        .select(['ub.userId', 'ub.deviceFingerprint', 'ub.ipAddress'])
        .where('ub.createdAt > :since', { since: new Date(Date.now() - 86400000) })
        .getRawMany();

      // Group by IP address to detect farms
      const ipGroups = this.groupBy(recentUsers, 'ipAddress');
      const deviceGroups = this.groupBy(recentUsers, 'deviceFingerprint');

      const suspiciousUsers = [];
      const botNetworks = [];

      // Detect IP-based farms
      for (const [ip, users] of Object.entries(ipGroups)) {
        if (users.length > 20) { // More than 20 users from same IP
          suspiciousUsers.push(...users.map(u => u.userId));
          botNetworks.push(users.map(u => u.userId));
        }
      }

      // Detect device farms
      for (const [device, users] of Object.entries(deviceGroups)) {
        if (users.length > 5) { // More than 5 users from same device
          suspiciousUsers.push(...users.map(u => u.userId));
        }
      }

      return {
        suspiciousUsers: [...new Set(suspiciousUsers)],
        botNetworks,
        recommendations: this.generateNetworkRecommendations(botNetworks.length)
      };
    } catch (error) {
      this.logger.error('Network analysis failed:', error);
      return { suspiciousUsers: [], botNetworks: [], recommendations: [] };
    }
  }

  // Helper Methods
  private calculateClickIntervals(timestamps: number[]): number[] {
    const intervals = [];
    for (let i = 1; i < timestamps.length; i++) {
      intervals.push(timestamps[i] - timestamps[i - 1]);
    }
    return intervals;
  }

  private calculateVariance(numbers: number[]): number {
    const mean = numbers.reduce((a, b) => a + b, 0) / numbers.length;
    const squaredDiffs = numbers.map(n => Math.pow(n - mean, 2));
    return squaredDiffs.reduce((a, b) => a + b, 0) / numbers.length;
  }

  private scoreClickPatterns(variance: number): number {
    // Human clicking has variance between 10,000-1,000,000 ms²
    if (variance < 1000) return 0.1; // Too consistent (bot)
    if (variance > 2000000) return 0.2; // Too random (bot)
    return Math.min(1.0, variance / 100000);
  }

  private calculateBiometricSimilarity(hash1: string, hash2: string): number {
    // Simplified similarity calculation - in production use proper face recognition
    let matches = 0;
    for (let i = 0; i < Math.min(hash1.length, hash2.length); i++) {
      if (hash1[i] === hash2[i]) matches++;
    }
    return matches / Math.max(hash1.length, hash2.length);
  }

  private calculateConfidence(factors: any): number {
    const scores = Object.values(factors) as number[];
    const variance = this.calculateVariance(scores);
    return Math.max(0.1, 1 - variance); // Higher variance = lower confidence
  }

  private identifyRiskFactors(factors: any): string[] {
    const risks = [];
    if (factors.biometric < 0.3) risks.push('Inconsistent biometric data');
    if (factors.behavioral < 0.3) risks.push('Non-human behavioral patterns');
    if (factors.network < 0.3) risks.push('Suspicious network connections');
    if (factors.device < 0.3) risks.push('Device authenticity concerns');
    if (factors.temporal < 0.3) risks.push('Unnatural activity timing');
    if (factors.content < 0.3) risks.push('Low quality or spam content');
    return risks;
  }

  private generateRecommendations(factors: any): string[] {
    const recommendations = [];
    if (factors.biometric < 0.5) recommendations.push('Require additional biometric verification');
    if (factors.behavioral < 0.5) recommendations.push('Implement CAPTCHA challenges');
    if (factors.network < 0.5) recommendations.push('Limit referral rewards');
    if (factors.device < 0.5) recommendations.push('Restrict device access');
    return recommendations;
  }

  private groupBy(array: any[], key: string): Record<string, any[]> {
    return array.reduce((groups, item) => {
      const value = item[key];
      groups[value] = groups[value] || [];
      groups[value].push(item);
      return groups;
    }, {});
  }

  // Additional helper methods would be implemented here...
  private async analyzeSessionPatterns(userData: UserBehaviorData): Promise<number> {
    // Implementation for session pattern analysis
    return 0.8;
  }

  private analyzeMouseMovement(patterns: number[]): number {
    // Implementation for mouse movement analysis
    return 0.7;
  }

  private async detectCircularReferrals(userId: string): Promise<number> {
    // Implementation for circular referral detection
    return 0.1;
  }

  private async calculateNetworkDiversity(connections: string[]): Promise<number> {
    // Implementation for network diversity calculation
    return 0.8;
  }

  private async analyzeMutualConnections(connections: string[]): Promise<number> {
    // Implementation for mutual connection analysis
    return 0.7;
  }

  private async getAccountAge(userId: string): Promise<number> {
    // Implementation for account age calculation
    return 30; // days
  }

  private analyzeUserAgent(userAgent: string): number {
    // Implementation for user agent analysis
    return 0.9;
  }

  private async analyzeGeolocation(userData: UserBehaviorData): Promise<number> {
    // Implementation for geolocation analysis
    return 0.8;
  }

  private calculateCircadianCompliance(hours: number[]): number {
    // Implementation for circadian pattern analysis
    const nightHours = hours.filter(h => h >= 23 || h <= 6).length;
    return Math.max(0.1, 1 - (nightHours / hours.length));
  }

  private detectContinuousActivity(activities: any[]): number {
    // Implementation for continuous activity detection
    return 0.2;
  }

  private analyzeWeeklyPatterns(activities: any[]): number {
    // Implementation for weekly pattern analysis
    return 0.8;
  }

  private async checkContentUniqueness(content: any): Promise<number> {
    // Implementation for content uniqueness check
    return 0.9;
  }

  private analyzeContentComplexity(content: any): number {
    // Implementation for content complexity analysis
    return 0.8;
  }

  private detectSpamPatterns(content: any): number {
    // Implementation for spam detection
    return 0.1;
  }

  private calculateAverageInterval(timestamps: number[]): number {
    if (timestamps.length < 2) return 1000;
    const intervals = this.calculateClickIntervals(timestamps);
    return intervals.reduce((a, b) => a + b, 0) / intervals.length;
  }

  private generateNetworkRecommendations(botNetworkCount: number): string[] {
    const recommendations = [];
    if (botNetworkCount > 10) recommendations.push('Implement stricter KYC requirements');
    if (botNetworkCount > 5) recommendations.push('Increase device verification');
    if (botNetworkCount > 0) recommendations.push('Monitor suspicious IP ranges');
    return recommendations;
  }
}
