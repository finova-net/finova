import { Injectable, Logger, BadRequestException, InternalServerErrorException } from '@nestjs/common';
import { ConfigService } from '@nestjs/config';
import { InjectRedis } from '@nestjs-modules/ioredis';
import { Redis } from 'ioredis';
import { PrismaService } from '../database/prisma.service';
import { HttpService } from '@nestjs/axios';
import { firstValueFrom } from 'rxjs';
import * as crypto from 'crypto';
import * as tf from '@tensorflow/tfjs-node';
import { 
  ContentAnalysisRequest, 
  ContentAnalysisResponse, 
  QualityScore,
  ContentType,
  PlatformType,
  AntiSpamResult,
  ViralPotentialScore,
  BrandSafetyResult
} from '../types/ai.types';

@Injectable()
export class AIQualityService {
  private readonly logger = new Logger(AIQualityService.name);
  private nlpModel: tf.LayersModel | null = null;
  private imageModel: tf.LayersModel | null = null;
  private spamDetectionModel: tf.LayersModel | null = null;
  private viralPredictionModel: tf.LayersModel | null = null;

  private readonly QUALITY_WEIGHTS = {
    originality: 0.25,
    engagement_potential: 0.30,
    platform_relevance: 0.15,
    brand_safety: 0.20,
    human_generated: 0.10
  };

  private readonly CACHE_TTL = 3600; // 1 hour
  private readonly MIN_QUALITY_SCORE = 0.5;
  private readonly MAX_QUALITY_SCORE = 2.0;

  constructor(
    private readonly configService: ConfigService,
    private readonly prisma: PrismaService,
    private readonly httpService: HttpService,
    @InjectRedis() private readonly redis: Redis,
  ) {
    this.initializeModels();
  }

  async analyzeContentQuality(request: ContentAnalysisRequest): Promise<ContentAnalysisResponse> {
    const startTime = Date.now();
    
    try {
      // Generate cache key
      const cacheKey = this.generateCacheKey(request);
      const cached = await this.getCachedResult(cacheKey);
      if (cached) {
        this.logger.debug(`Cache hit for content analysis: ${cacheKey}`);
        return cached;
      }

      // Pre-validation checks
      await this.validateRequest(request);

      // Parallel analysis execution
      const [
        originalityScore,
        engagementScore,
        platformRelevanceScore,
        brandSafetyResult,
        humanGeneratedScore,
        antiSpamResult,
        viralPotential
      ] = await Promise.all([
        this.analyzeOriginality(request.content, request.contentType),
        this.predictEngagement(request.content, request.platform, request.contentType),
        this.checkPlatformRelevance(request.content, request.platform, request.contentType),
        this.checkBrandSafety(request.content, request.contentType),
        this.detectHumanGenerated(request.content, request.contentType),
        this.performAntiSpamCheck(request),
        this.calculateViralPotential(request.content, request.platform, request.contentType)
      ]);

      // Calculate weighted quality score
      const qualityScore = this.calculateWeightedQualityScore({
        originality: originalityScore,
        engagement_potential: engagementScore,
        platform_relevance: platformRelevanceScore,
        brand_safety: brandSafetyResult.score,
        human_generated: humanGeneratedScore
      });

      // Apply anti-spam penalty
      const finalScore = antiSpamResult.isSpam 
        ? Math.max(0.1, qualityScore * 0.3) 
        : qualityScore;

      const response: ContentAnalysisResponse = {
        requestId: request.requestId,
        userId: request.userId,
        qualityScore: {
          overall: this.clampScore(finalScore),
          originality: originalityScore,
          engagement: engagementScore,
          platformRelevance: platformRelevanceScore,
          brandSafety: brandSafetyResult.score,
          humanGenerated: humanGeneratedScore
        },
        viralPotential,
        brandSafety: brandSafetyResult,
        antiSpam: antiSpamResult,
        processingTime: Date.now() - startTime,
        timestamp: new Date(),
        recommendations: this.generateRecommendations(qualityScore, antiSpamResult, brandSafetyResult)
      };

      // Cache result
      await this.cacheResult(cacheKey, response);

      // Store in database for analytics
      await this.storeAnalysisResult(response);

      this.logger.log(`Content analysis completed for user ${request.userId}: score ${finalScore.toFixed(3)}`);
      
      return response;

    } catch (error) {
      this.logger.error(`Content analysis failed: ${error.message}`, error.stack);
      throw new InternalServerErrorException('Content analysis failed');
    }
  }

  private async analyzeOriginality(content: string, contentType: ContentType): Promise<number> {
    try {
      // Check against database of known content
      const contentHash = crypto.createHash('sha256').update(content).digest('hex');
      const existingContent = await this.redis.get(`content:${contentHash}`);
      
      if (existingContent) {
        return 0.2; // Very low originality for duplicate content
      }

      // NLP-based similarity check
      if (this.nlpModel && contentType === 'text') {
        const embedding = await this.generateTextEmbedding(content);
        const similarity = await this.checkSimilarityDatabase(embedding);
        
        if (similarity > 0.9) return 0.3;
        if (similarity > 0.7) return 0.6;
        if (similarity > 0.5) return 0.8;
      }

      // Unique content patterns
      const uniquenessScore = this.calculateTextUniqueness(content);
      
      // Store content hash for future checks
      await this.redis.setex(`content:${contentHash}`, 86400 * 7, Date.now().toString());

      return Math.min(1.0, uniquenessScore);

    } catch (error) {
      this.logger.warn(`Originality analysis failed: ${error.message}`);
      return 0.7; // Default moderate score
    }
  }

  private async predictEngagement(content: string, platform: PlatformType, contentType: ContentType): Promise<number> {
    try {
      let baseScore = 0.5;

      // Platform-specific engagement patterns
      const platformMultipliers = {
        'instagram': contentType === 'image' ? 1.2 : 0.9,
        'tiktok': contentType === 'video' ? 1.3 : 0.7,
        'youtube': contentType === 'video' ? 1.4 : 0.8,
        'facebook': contentType === 'text' ? 1.1 : 1.0,
        'twitter': contentType === 'text' ? 1.2 : 0.9
      };

      baseScore *= platformMultipliers[platform] || 1.0;

      // Content quality indicators
      if (contentType === 'text') {
        const wordCount = content.split(' ').length;
        const sentenceVariety = this.analyzeSentenceVariety(content);
        const emotionalTone = this.analyzeEmotionalTone(content);
        const hashtags = (content.match(/#\w+/g) || []).length;

        baseScore *= this.calculateTextEngagementMultiplier(wordCount, sentenceVariety, emotionalTone, hashtags);
      }

      // Time-based trending factors
      const trendingTopics = await this.getTrendingTopics(platform);
      if (this.containsTrendingElements(content, trendingTopics)) {
        baseScore *= 1.3;
      }

      // ML model prediction if available
      if (this.viralPredictionModel) {
        const mlScore = await this.runViralPrediction(content, platform, contentType);
        baseScore = (baseScore * 0.7) + (mlScore * 0.3); // Weighted combination
      }

      return Math.min(1.0, baseScore);

    } catch (error) {
      this.logger.warn(`Engagement prediction failed: ${error.message}`);
      return 0.5;
    }
  }

  private async checkPlatformRelevance(content: string, platform: PlatformType, contentType: ContentType): Promise<number> {
    const platformRules = {
      'instagram': {
        'image': 1.0,
        'video': 0.9,
        'text': 0.6
      },
      'tiktok': {
        'video': 1.0,
        'image': 0.4,
        'text': 0.3
      },
      'youtube': {
        'video': 1.0,
        'image': 0.5,
        'text': 0.4
      },
      'facebook': {
        'text': 1.0,
        'image': 0.9,
        'video': 0.8
      },
      'twitter': {
        'text': 1.0,
        'image': 0.7,
        'video': 0.8
      }
    };

    let relevanceScore = platformRules[platform]?.[contentType] || 0.5;

    // Content length appropriateness
    if (contentType === 'text') {
      const length = content.length;
      const optimalLengths = {
        'twitter': 280,
        'instagram': 2200,
        'facebook': 500,
        'tiktok': 100,
        'youtube': 1000
      };

      const optimal = optimalLengths[platform] || 500;
      const lengthRatio = Math.min(length, optimal) / optimal;
      relevanceScore *= (0.5 + lengthRatio * 0.5);
    }

    return Math.min(1.0, relevanceScore);
  }

  private async checkBrandSafety(content: string, contentType: ContentType): Promise<BrandSafetyResult> {
    const result: BrandSafetyResult = {
      score: 1.0,
      isSafe: true,
      violations: [],
      categories: []
    };

    try {
      // Prohibited content patterns
      const violations = [];
      const prohibitedPatterns = [
        { pattern: /hate|discrimination|racism/i, category: 'hate_speech', severity: 0.9 },
        { pattern: /violence|weapon|harm/i, category: 'violence', severity: 0.8 },
        { pattern: /drugs|alcohol|gambling/i, category: 'substance_abuse', severity: 0.6 },
        { pattern: /adult|nsfw|explicit/i, category: 'adult_content', severity: 0.9 },
        { pattern: /scam|fraud|fake/i, category: 'deception', severity: 0.7 }
      ];

      for (const rule of prohibitedPatterns) {
        if (rule.pattern.test(content)) {
          violations.push({
            type: rule.category,
            severity: rule.severity,
            confidence: 0.8
          });
          result.score *= (1 - rule.severity * 0.5);
        }
      }

      // External brand safety API check
      if (violations.length === 0) {
        const externalCheck = await this.performExternalBrandSafetyCheck(content);
        if (!externalCheck.safe) {
          violations.push(...externalCheck.violations);
          result.score *= 0.3;
        }
      }

      result.violations = violations;
      result.isSafe = violations.length === 0;
      result.categories = violations.map(v => v.type);

    } catch (error) {
      this.logger.warn(`Brand safety check failed: ${error.message}`);
      result.score = 0.5; // Conservative default
    }

    return result;
  }

  private async detectHumanGenerated(content: string, contentType: ContentType): Promise<number> {
    try {
      if (contentType !== 'text') return 1.0; // Assume human for non-text content

      // Pattern-based AI detection
      const aiIndicators = [
        /as an ai|i'm an ai|artificial intelligence/i,
        /generated by|created by ai|ai-generated/i,
        /here are some|here's a list|in conclusion/i,
        /(certainly|absolutely|definitely).{0,20}(here|this|that)/i
      ];

      let aiScore = 0;
      for (const pattern of aiIndicators) {
        if (pattern.test(content)) aiScore += 0.3;
      }

      // Statistical analysis
      const avgSentenceLength = this.calculateAverageSentenceLength(content);
      const vocabularyDiversity = this.calculateVocabularyDiversity(content);
      const punctuationVariety = this.analyzePunctuationVariety(content);

      // AI content tends to be more uniform
      if (avgSentenceLength > 20 && vocabularyDiversity < 0.3) aiScore += 0.2;
      if (punctuationVariety < 0.4) aiScore += 0.1;

      // ML model prediction if available
      if (this.nlpModel) {
        const mlAiScore = await this.runAIDetectionModel(content);
        aiScore = (aiScore * 0.6) + (mlAiScore * 0.4);
      }

      return Math.max(0.1, 1.0 - Math.min(1.0, aiScore));

    } catch (error) {
      this.logger.warn(`Human detection failed: ${error.message}`);
      return 0.8; // Assume mostly human
    }
  }

  private async performAntiSpamCheck(request: ContentAnalysisRequest): Promise<AntiSpamResult> {
    const result: AntiSpamResult = {
      isSpam: false,
      confidence: 0,
      reasons: []
    };

    try {
      const { userId, content, timestamp } = request;

      // Rate limiting check
      const userKey = `spam:${userId}:${new Date().toDateString()}`;
      const dailyCount = await this.redis.incr(userKey);
      await this.redis.expire(userKey, 86400);

      if (dailyCount > 100) {
        result.isSpam = true;
        result.confidence = 0.9;
        result.reasons.push('excessive_daily_posts');
        return result;
      }

      // Content similarity check
      const recentPosts = await this.redis.lrange(`user:${userId}:recent`, 0, 9);
      const contentHash = crypto.createHash('md5').update(content).digest('hex');
      
      if (recentPosts.includes(contentHash)) {
        result.isSpam = true;
        result.confidence = 0.8;
        result.reasons.push('duplicate_content');
        return result;
      }

      // Store recent content
      await this.redis.lpush(`user:${userId}:recent`, contentHash);
      await this.redis.ltrim(`user:${userId}:recent`, 0, 9);
      await this.redis.expire(`user:${userId}:recent`, 3600);

      // Spam pattern detection
      const spamPatterns = [
        { pattern: /click here|buy now|limited time/i, weight: 0.3 },
        { pattern: /\b(free|win|earn).{0,20}(money|cash|$)/i, weight: 0.4 },
        { pattern: /([A-Z]{2,}.*){3,}/i, weight: 0.2 }, // Excessive caps
        { pattern: /(.{1,3})\1{5,}/i, weight: 0.5 }, // Repetitive characters
      ];

      let spamScore = 0;
      const triggeredPatterns = [];

      for (const spam of spamPatterns) {
        if (spam.pattern.test(content)) {
          spamScore += spam.weight;
          triggeredPatterns.push(spam.pattern.source);
        }
      }

      if (spamScore > 0.5) {
        result.isSpam = true;
        result.confidence = Math.min(0.95, spamScore);
        result.reasons.push('spam_patterns');
      }

      // ML spam detection
      if (this.spamDetectionModel && spamScore < 0.5) {
        const mlSpamScore = await this.runSpamDetectionModel(content);
        if (mlSpamScore > 0.7) {
          result.isSpam = true;
          result.confidence = mlSpamScore;
          result.reasons.push('ml_detection');
        }
      }

    } catch (error) {
      this.logger.warn(`Anti-spam check failed: ${error.message}`);
    }

    return result;
  }

  private async calculateViralPotential(content: string, platform: PlatformType, contentType: ContentType): Promise<ViralPotentialScore> {
    let score = 0.1;
    const factors = [];

    try {
      // Engagement elements
      if (contentType === 'text') {
        const emojis = (content.match(/[\u{1F600}-\u{1F64F}]|[\u{1F300}-\u{1F5FF}]|[\u{1F680}-\u{1F6FF}]|[\u{1F1E0}-\u{1F1FF}]/gu) || []).length;
        const hashtags = (content.match(/#\w+/g) || []).length;
        const mentions = (content.match(/@\w+/g) || []).length;
        const questions = (content.match(/\?/g) || []).length;

        if (emojis > 0) { score += 0.1; factors.push('emoji_usage'); }
        if (hashtags >= 2 && hashtags <= 5) { score += 0.2; factors.push('optimal_hashtags'); }
        if (mentions > 0) { score += 0.1; factors.push('user_mentions'); }
        if (questions > 0) { score += 0.15; factors.push('engagement_questions'); }
      }

      // Trending topic alignment
      const trendingTopics = await this.getTrendingTopics(platform);
      if (this.containsTrendingElements(content, trendingTopics)) {
        score += 0.3;
        factors.push('trending_alignment');
      }

      // Time of posting (peak hours boost)
      const hour = new Date().getHours();
      const peakHours = platform === 'instagram' ? [18, 19, 20] : [12, 17, 19];
      if (peakHours.includes(hour)) {
        score += 0.1;
        factors.push('peak_timing');
      }

      // Content quality indicators
      const qualityIndicators = this.analyzeContentQualityIndicators(content, contentType);
      score += qualityIndicators.score * 0.2;
      factors.push(...qualityIndicators.factors);

      return {
        score: Math.min(1.0, score),
        factors,
        confidence: 0.7
      };

    } catch (error) {
      this.logger.warn(`Viral potential calculation failed: ${error.message}`);
      return { score: 0.3, factors: [], confidence: 0.3 };
    }
  }

  // Helper Methods
  private calculateWeightedQualityScore(scores: Record<string, number>): number {
    let weightedSum = 0;
    let totalWeight = 0;

    for (const [key, weight] of Object.entries(this.QUALITY_WEIGHTS)) {
      if (scores[key] !== undefined) {
        weightedSum += scores[key] * weight;
        totalWeight += weight;
      }
    }

    return totalWeight > 0 ? weightedSum / totalWeight : 0.5;
  }

  private clampScore(score: number): number {
    return Math.max(this.MIN_QUALITY_SCORE, Math.min(this.MAX_QUALITY_SCORE, score));
  }

  private generateCacheKey(request: ContentAnalysisRequest): string {
    const contentHash = crypto.createHash('md5')
      .update(JSON.stringify({
        content: request.content,
        platform: request.platform,
        contentType: request.contentType
      }))
      .digest('hex');
    
    return `ai:quality:${contentHash}`;
  }

  private async getCachedResult(key: string): Promise<ContentAnalysisResponse | null> {
    try {
      const cached = await this.redis.get(key);
      return cached ? JSON.parse(cached) : null;
    } catch {
      return null;
    }
  }

  private async cacheResult(key: string, result: ContentAnalysisResponse): Promise<void> {
    try {
      await this.redis.setex(key, this.CACHE_TTL, JSON.stringify(result));
    } catch (error) {
      this.logger.warn(`Cache storage failed: ${error.message}`);
    }
  }

  private async validateRequest(request: ContentAnalysisRequest): Promise<void> {
    if (!request.content || request.content.trim().length === 0) {
      throw new BadRequestException('Content cannot be empty');
    }

    if (request.content.length > 10000) {
      throw new BadRequestException('Content too long (max 10000 characters)');
    }

    if (!['text', 'image', 'video'].includes(request.contentType)) {
      throw new BadRequestException('Invalid content type');
    }
  }

  private async initializeModels(): Promise<void> {
    try {
      // Initialize TensorFlow models (mock implementation)
      this.logger.log('Initializing AI models...');
      
      // In production, load actual model files
      // this.nlpModel = await tf.loadLayersModel('path/to/nlp-model');
      // this.spamDetectionModel = await tf.loadLayersModel('path/to/spam-model');
      
      this.logger.log('AI models initialized successfully');
    } catch (error) {
      this.logger.error(`Model initialization failed: ${error.message}`);
    }
  }

  // Additional utility methods would be implemented here
  private calculateTextUniqueness(content: string): number {
    const words = content.toLowerCase().split(/\s+/);
    const uniqueWords = new Set(words);
    return Math.min(1.0, uniqueWords.size / Math.max(words.length, 1));
  }

  private analyzeSentenceVariety(content: string): number {
    const sentences = content.split(/[.!?]+/).filter(s => s.trim().length > 0);
    const lengths = sentences.map(s => s.split(' ').length);
    const avgLength = lengths.reduce((a, b) => a + b, 0) / lengths.length;
    const variance = lengths.reduce((sum, len) => sum + Math.pow(len - avgLength, 2), 0) / lengths.length;
    return Math.min(1.0, Math.sqrt(variance) / avgLength);
  }

  private analyzeEmotionalTone(content: string): number {
    const positiveWords = ['good', 'great', 'amazing', 'love', 'awesome', 'excellent'];
    const negativeWords = ['bad', 'terrible', 'hate', 'awful', 'horrible', 'worst'];
    
    const words = content.toLowerCase().split(/\s+/);
    const positive = words.filter(w => positiveWords.includes(w)).length;
    const negative = words.filter(w => negativeWords.includes(w)).length;
    
    return Math.max(0.1, 0.5 + (positive - negative) / Math.max(words.length, 1));
  }

  private calculateTextEngagementMultiplier(wordCount: number, variety: number, tone: number, hashtags: number): number {
    let multiplier = 1.0;
    
    // Optimal word count ranges
    if (wordCount >= 20 && wordCount <= 150) multiplier *= 1.2;
    else if (wordCount < 10 || wordCount > 300) multiplier *= 0.8;
    
    // Sentence variety bonus
    multiplier *= (0.8 + variety * 0.4);
    
    // Emotional tone impact
    multiplier *= tone;
    
    // Hashtag optimization
    if (hashtags >= 2 && hashtags <= 5) multiplier *= 1.1;
    else if (hashtags > 10) multiplier *= 0.7;
    
    return multiplier;
  }

  private async getTrendingTopics(platform: PlatformType): Promise<string[]> {
    // Mock implementation - in production, integrate with platform APIs
    return ['technology', 'crypto', 'ai', 'social', 'fintech'];
  }

  private containsTrendingElements(content: string, trending: string[]): boolean {
    const lowerContent = content.toLowerCase();
    return trending.some(topic => lowerContent.includes(topic));
  }

  private calculateAverageSentenceLength(content: string): number {
    const sentences = content.split(/[.!?]+/).filter(s => s.trim().length > 0);
    if (sentences.length === 0) return 0;
    
    const totalLength = sentences.reduce((sum, sentence) => sum + sentence.split(' ').length, 0);
    return totalLength / sentences.length;
  }

  private calculateVocabularyDiversity(content: string): number {
    const words = content.toLowerCase().split(/\s+/).filter(w => w.length > 2);
    if (words.length === 0) return 0;
    
    const uniqueWords = new Set(words);
    return uniqueWords.size / words.length;
  }

  private analyzePunctuationVariety(content: string): number {
    const punctuation = content.match(/[.!?,:;-]/g) || [];
    const uniquePunct = new Set(punctuation);
    return Math.min(1.0, uniquePunct.size / 6); // Normalize to common punctuation types
  }

  private analyzeContentQualityIndicators(content: string, contentType: ContentType): { score: number; factors: string[] } {
    const factors = [];
    let score = 0;

    if (contentType === 'text') {
      if (content.length >= 50) { score += 0.1; factors.push('adequate_length'); }
      if (/[.!?]/.test(content)) { score += 0.1; factors.push('proper_punctuation'); }
      if (!/(.)\1{3,}/.test(content)) { score += 0.1; factors.push('no_spam_chars'); }
    }

    return { score, factors };
  }

  private async performExternalBrandSafetyCheck(content: string): Promise<{ safe: boolean; violations: any[] }> {
    // Mock implementation - integrate with brand safety APIs
    return { safe: true, violations: [] };
  }

  private async generateTextEmbedding(content: string): Promise<number[]> {
    // Mock implementation - use actual NLP model
    return new Array(128).fill(0).map(() => Math.random());
  }

  private async checkSimilarityDatabase(embedding: number[]): Promise<number> {
    // Mock implementation - check against stored embeddings
    return Math.random() * 0.3; // Low similarity by default
  }

  private async runViralPrediction(content: string, platform: PlatformType, contentType: ContentType): Promise<number> {
    // Mock ML model prediction
    return Math.random() * 0.5 + 0.3;
  }

  private async runAIDetectionModel(content: string): Promise<number> {
    // Mock AI detection model
    return Math.random() * 0.3; // Low AI probability by default
  }

  private async runSpamDetectionModel(content: string): Promise<number> {
    // Mock spam detection model
    return Math.random() * 0.2; // Low spam probability by default
  }

  private generateRecommendations(qualityScore: number, antiSpam: AntiSpamResult, brandSafety: BrandSafetyResult): string[] {
    const recommendations = [];

    if (qualityScore < 0.7) {
      recommendations.push('Consider adding more engaging elements like emojis or questions');
    }
    
    if (antiSpam.isSpam) {
      recommendations.push('Avoid repetitive content and spam-like patterns');
    }
    
    if (!brandSafety.isSafe) {
      recommendations.push('Review content for brand safety compliance');
    }

    return recommendations;
  }

  private async storeAnalysisResult(result: ContentAnalysisResponse): Promise<void> {
    try {
      await this.prisma.contentAnalysis.create({
        data: {
          requestId: result.requestId,
          userId: result.userId,
          qualityScore: result.qualityScore.overall,
          viralPotential: result.viralPotential.score,
          isBrandSafe: result.brandSafety.isSafe,
          isSpam: result.antiSpam.isSpam,
          processingTime: result.processingTime,
          timestamp: result.timestamp
        }
      });
    } catch (error) {
      this.logger.warn(`Failed to store analysis result: ${error.message}`);
    }
  }
}
