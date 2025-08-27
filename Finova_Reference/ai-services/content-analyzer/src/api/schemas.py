"""
Finova Network - Content Analyzer API Schemas
Enterprise-grade Pydantic schemas for content analysis services
Supporting XP calculation, quality scoring, and anti-bot detection
"""

from typing import Optional, List, Dict, Any, Union
from pydantic import BaseModel, Field, validator, root_validator
from enum import Enum
from datetime import datetime
import re


class ContentType(str, Enum):
    """Content types supported by the analyzer"""
    TEXT = "text"
    IMAGE = "image"
    VIDEO = "video"
    AUDIO = "audio"
    MIXED = "mixed"


class Platform(str, Enum):
    """Social media platforms integrated with Finova"""
    INSTAGRAM = "instagram"
    TIKTOK = "tiktok"
    YOUTUBE = "youtube"
    FACEBOOK = "facebook"
    TWITTER_X = "twitter_x"
    FINOVA_APP = "finova_app"


class QualityTier(str, Enum):
    """Quality assessment tiers"""
    LOW = "low"           # 0.5x - 0.7x multiplier
    AVERAGE = "average"   # 0.8x - 1.2x multiplier
    HIGH = "high"         # 1.3x - 1.7x multiplier
    PREMIUM = "premium"   # 1.8x - 2.0x multiplier


class RiskLevel(str, Enum):
    """Content risk levels for brand safety"""
    SAFE = "safe"
    LOW_RISK = "low_risk"
    MEDIUM_RISK = "medium_risk"
    HIGH_RISK = "high_risk"
    UNSAFE = "unsafe"


# Base Models
class UserContext(BaseModel):
    """User context for personalized analysis"""
    user_id: str = Field(..., description="Unique user identifier")
    xp_level: int = Field(ge=1, le=200, description="User's current XP level")
    rp_tier: str = Field(..., description="User's RP tier (Explorer/Connector/etc)")
    total_fin_holdings: float = Field(ge=0, description="Total $FIN holdings")
    streak_days: int = Field(ge=0, description="Current activity streak")
    is_kyc_verified: bool = Field(default=False, description="KYC verification status")
    guild_membership: Optional[str] = Field(None, description="Guild ID if member")
    creation_date: datetime = Field(..., description="Account creation date")
    
    @validator('rp_tier')
    def validate_rp_tier(cls, v):
        valid_tiers = ['Explorer', 'Connector', 'Influencer', 'Leader', 'Ambassador']
        if v not in valid_tiers:
            raise ValueError(f'RP tier must be one of: {valid_tiers}')
        return v


class ContentMetadata(BaseModel):
    """Content metadata and platform-specific information"""
    content_id: str = Field(..., description="Unique content identifier")
    platform: Platform = Field(..., description="Source platform")
    content_type: ContentType = Field(..., description="Type of content")
    language: str = Field(default="en", description="Content language (ISO 639-1)")
    hashtags: List[str] = Field(default=[], description="Content hashtags")
    mentions: List[str] = Field(default=[], description="User mentions")
    engagement_metrics: Dict[str, int] = Field(default={}, description="Likes, shares, comments")
    creation_timestamp: datetime = Field(..., description="Content creation time")
    external_url: Optional[str] = Field(None, description="External link if any")
    duration_seconds: Optional[int] = Field(None, description="Video/audio duration")
    
    @validator('hashtags', 'mentions', pre=True)
    def clean_social_tags(cls, v):
        if isinstance(v, list):
            return [tag.strip().lower().replace('#', '').replace('@', '') for tag in v]
        return []
    
    @validator('engagement_metrics')
    def validate_engagement(cls, v):
        allowed_keys = ['likes', 'shares', 'comments', 'views', 'reactions']
        return {k: max(0, int(v.get(k, 0))) for k in allowed_keys if k in v}


# Request Schemas
class ContentAnalysisRequest(BaseModel):
    """Main request schema for content analysis"""
    content_text: Optional[str] = Field(None, max_length=10000, description="Text content")
    image_urls: List[str] = Field(default=[], description="Image URLs for analysis")
    video_url: Optional[str] = Field(None, description="Video URL for analysis")
    audio_url: Optional[str] = Field(None, description="Audio URL for analysis")
    metadata: ContentMetadata = Field(..., description="Content metadata")
    user_context: UserContext = Field(..., description="User context information")
    analysis_options: Dict[str, bool] = Field(
        default={
            "quality_check": True,
            "originality_check": True,
            "engagement_prediction": True,
            "brand_safety_check": True,
            "ai_detection": True,
            "sentiment_analysis": True
        },
        description="Analysis options to enable/disable"
    )
    
    @root_validator
    def validate_content_provided(cls, values):
        content_fields = ['content_text', 'image_urls', 'video_url', 'audio_url']
        if not any(values.get(field) for field in content_fields):
            raise ValueError('At least one content field must be provided')
        return values
    
    @validator('content_text')
    def validate_text_content(cls, v):
        if v and len(v.strip()) < 3:
            raise ValueError('Text content must be at least 3 characters')
        return v.strip() if v else None
    
    @validator('image_urls', 'video_url', 'audio_url')
    def validate_urls(cls, v):
        if isinstance(v, list):
            for url in v:
                if not re.match(r'^https?://', url):
                    raise ValueError(f'Invalid URL format: {url}')
            return v[:10]  # Limit to 10 images max
        elif isinstance(v, str) and v:
            if not re.match(r'^https?://', v):
                raise ValueError(f'Invalid URL format: {v}')
        return v


class BulkAnalysisRequest(BaseModel):
    """Request schema for bulk content analysis"""
    contents: List[ContentAnalysisRequest] = Field(
        ..., min_items=1, max_items=50, 
        description="List of content analysis requests"
    )
    batch_id: Optional[str] = Field(None, description="Batch identifier for tracking")
    priority: int = Field(default=1, ge=1, le=5, description="Processing priority (1=lowest, 5=highest)")


# Response Schemas
class QualityScore(BaseModel):
    """Content quality assessment results"""
    overall_score: float = Field(..., ge=0.5, le=2.0, description="Overall quality multiplier")
    originality_score: float = Field(..., ge=0, le=1, description="Content originality (0-1)")
    engagement_potential: float = Field(..., ge=0, le=1, description="Predicted engagement score")
    platform_relevance: float = Field(..., ge=0, le=1, description="Platform-specific relevance")
    brand_safety_score: float = Field(..., ge=0, le=1, description="Brand safety assessment")
    human_generated_score: float = Field(..., ge=0, le=1, description="Human vs AI detection")
    quality_tier: QualityTier = Field(..., description="Quality tier classification")
    
    # Detailed breakdown
    linguistic_quality: float = Field(..., ge=0, le=1, description="Grammar, vocabulary, style")
    visual_quality: Optional[float] = Field(None, ge=0, le=1, description="Image/video quality")
    audio_quality: Optional[float] = Field(None, ge=0, le=1, description="Audio quality")
    content_depth: float = Field(..., ge=0, le=1, description="Content informativeness")
    uniqueness_score: float = Field(..., ge=0, le=1, description="Content uniqueness")


class EngagementPrediction(BaseModel):
    """Engagement prediction results"""
    predicted_likes: int = Field(..., ge=0, description="Predicted like count")
    predicted_shares: int = Field(..., ge=0, description="Predicted share count")
    predicted_comments: int = Field(..., ge=0, description="Predicted comment count")
    predicted_views: int = Field(..., ge=0, description="Predicted view count")
    viral_probability: float = Field(..., ge=0, le=1, description="Probability of going viral")
    engagement_rate: float = Field(..., ge=0, description="Predicted engagement rate")
    reach_potential: str = Field(..., description="Reach category (low/medium/high/viral)")
    
    confidence_interval: Dict[str, float] = Field(
        default={}, description="Confidence intervals for predictions"
    )


class BrandSafetyResult(BaseModel):
    """Brand safety assessment results"""
    overall_risk: RiskLevel = Field(..., description="Overall risk assessment")
    risk_score: float = Field(..., ge=0, le=1, description="Risk score (0=safe, 1=unsafe)")
    
    # Risk categories
    violence_risk: float = Field(..., ge=0, le=1, description="Violence content risk")
    adult_content_risk: float = Field(..., ge=0, le=1, description="Adult content risk")
    hate_speech_risk: float = Field(..., ge=0, le=1, description="Hate speech risk")
    misinformation_risk: float = Field(..., ge=0, le=1, description="Misinformation risk")
    spam_risk: float = Field(..., ge=0, le=1, description="Spam content risk")
    
    flagged_keywords: List[str] = Field(default=[], description="Flagged keywords found")
    content_warnings: List[str] = Field(default=[], description="Content warnings")
    recommendations: List[str] = Field(default=[], description="Safety recommendations")


class SentimentAnalysis(BaseModel):
    """Sentiment analysis results"""
    overall_sentiment: str = Field(..., description="Overall sentiment (positive/negative/neutral)")
    sentiment_score: float = Field(..., ge=-1, le=1, description="Sentiment score (-1 to 1)")
    confidence: float = Field(..., ge=0, le=1, description="Prediction confidence")
    
    emotions: Dict[str, float] = Field(
        default={}, description="Emotion breakdown (joy, anger, fear, etc.)"
    )
    topics: List[Dict[str, Any]] = Field(
        default=[], description="Detected topics and their sentiments"
    )


class XPCalculation(BaseModel):
    """XP calculation based on content analysis"""
    base_xp: int = Field(..., ge=0, description="Base XP for activity type")
    platform_multiplier: float = Field(..., ge=1.0, le=1.4, description="Platform-specific multiplier")
    quality_multiplier: float = Field(..., ge=0.5, le=2.0, description="Quality-based multiplier")
    streak_bonus: float = Field(..., ge=1.0, le=3.0, description="Streak bonus multiplier")
    level_progression: float = Field(..., ge=0.1, le=1.0, description="Level progression factor")
    
    final_xp: int = Field(..., ge=0, description="Final calculated XP")
    bonus_explanation: List[str] = Field(default=[], description="Bonus explanations")


class ContentAnalysisResponse(BaseModel):
    """Complete content analysis response"""
    analysis_id: str = Field(..., description="Unique analysis identifier")
    content_id: str = Field(..., description="Content identifier from request")
    timestamp: datetime = Field(default_factory=datetime.utcnow, description="Analysis timestamp")
    processing_time_ms: int = Field(..., ge=0, description="Processing time in milliseconds")
    
    # Analysis results
    quality_score: QualityScore = Field(..., description="Quality assessment results")
    engagement_prediction: EngagementPrediction = Field(..., description="Engagement predictions")
    brand_safety: BrandSafetyResult = Field(..., description="Brand safety assessment")
    sentiment_analysis: SentimentAnalysis = Field(..., description="Sentiment analysis")
    xp_calculation: XPCalculation = Field(..., description="XP calculation results")
    
    # Additional metadata
    analysis_version: str = Field(default="1.0", description="Analysis model version")
    flags: List[str] = Field(default=[], description="Special flags or warnings")
    recommendations: List[str] = Field(default=[], description="Content improvement recommendations")


class BulkAnalysisResponse(BaseModel):
    """Bulk analysis response"""
    batch_id: str = Field(..., description="Batch identifier")
    total_processed: int = Field(..., ge=0, description="Total items processed")
    successful_analyses: int = Field(..., ge=0, description="Successful analyses count")
    failed_analyses: int = Field(..., ge=0, description="Failed analyses count")
    processing_time_seconds: float = Field(..., ge=0, description="Total processing time")
    
    results: List[ContentAnalysisResponse] = Field(
        default=[], description="Individual analysis results"
    )
    errors: List[Dict[str, Any]] = Field(
        default=[], description="Error details for failed analyses"
    )


# Error Schemas
class AnalysisError(BaseModel):
    """Error response schema"""
    error_code: str = Field(..., description="Error code")
    error_message: str = Field(..., description="Human-readable error message")
    error_details: Dict[str, Any] = Field(default={}, description="Additional error details")
    timestamp: datetime = Field(default_factory=datetime.utcnow)
    request_id: Optional[str] = Field(None, description="Request identifier for tracking")


# Health Check Schemas
class HealthCheckResponse(BaseModel):
    """Health check response"""
    status: str = Field(..., description="Service status")
    timestamp: datetime = Field(default_factory=datetime.utcnow)
    version: str = Field(..., description="Service version")
    models_loaded: Dict[str, bool] = Field(
        default={}, description="Status of loaded AI models"
    )
    queue_size: int = Field(..., ge=0, description="Current processing queue size")
    avg_processing_time_ms: float = Field(..., ge=0, description="Average processing time")


# Configuration Schemas
class AnalysisConfig(BaseModel):
    """Configuration for analysis parameters"""
    quality_threshold: float = Field(default=0.5, ge=0, le=1, description="Minimum quality threshold")
    engagement_threshold: int = Field(default=100, ge=0, description="Minimum engagement for viral")
    brand_safety_threshold: float = Field(default=0.8, ge=0, le=1, description="Brand safety threshold")
    ai_detection_threshold: float = Field(default=0.7, ge=0, le=1, description="AI detection threshold")
    
    # XP calculation parameters
    base_xp_rates: Dict[str, int] = Field(
        default={
            "text_post": 50,
            "image_post": 75,
            "video_post": 150,
            "comment": 25,
            "story": 25
        },
        description="Base XP rates for different activities"
    )
    
    platform_multipliers: Dict[str, float] = Field(
        default={
            "instagram": 1.2,
            "tiktok": 1.3,
            "youtube": 1.4,
            "facebook": 1.1,
            "twitter_x": 1.2,
            "finova_app": 1.0
        },
        description="Platform-specific XP multipliers"
    )


# Webhook Schemas
class WebhookPayload(BaseModel):
    """Webhook notification payload"""
    event_type: str = Field(..., description="Event type (analysis_complete, error, etc.)")
    analysis_id: str = Field(..., description="Analysis identifier")
    content_id: str = Field(..., description="Content identifier")
    user_id: str = Field(..., description="User identifier")
    timestamp: datetime = Field(default_factory=datetime.utcnow)
    data: Dict[str, Any] = Field(default={}, description="Event-specific data")


# Export all schemas
__all__ = [
    # Enums
    'ContentType', 'Platform', 'QualityTier', 'RiskLevel',
    
    # Base Models
    'UserContext', 'ContentMetadata',
    
    # Request Schemas
    'ContentAnalysisRequest', 'BulkAnalysisRequest',
    
    # Response Components
    'QualityScore', 'EngagementPrediction', 'BrandSafetyResult', 
    'SentimentAnalysis', 'XPCalculation',
    
    # Main Response Schemas
    'ContentAnalysisResponse', 'BulkAnalysisResponse',
    
    # Error and Health Schemas
    'AnalysisError', 'HealthCheckResponse',
    
    # Configuration Schemas
    'AnalysisConfig', 'WebhookPayload'
]
