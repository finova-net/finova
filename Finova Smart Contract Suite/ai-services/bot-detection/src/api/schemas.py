"""
Finova Network - Bot Detection API Schemas
Enterprise-grade Pydantic schemas for bot detection service
Version: 1.0.0
"""

from typing import List, Dict, Optional, Any, Union
from datetime import datetime, timedelta
from enum import Enum
from pydantic import BaseModel, Field, validator, root_validator
from decimal import Decimal
import re

# ==================== ENUMS ====================

class RiskLevel(str, Enum):
    """Risk level classification"""
    MINIMAL = "minimal"
    LOW = "low"
    MEDIUM = "medium"
    HIGH = "high"
    CRITICAL = "critical"

class DetectionType(str, Enum):
    """Type of bot detection analysis"""
    BEHAVIORAL = "behavioral"
    TEMPORAL = "temporal"
    NETWORK = "network"
    DEVICE = "device"
    BIOMETRIC = "biometric"
    CONTENT = "content"

class ActionType(str, Enum):
    """User action types"""
    LOGIN = "login"
    MINING = "mining"
    SOCIAL_POST = "social_post"
    LIKE = "like"
    COMMENT = "comment"
    SHARE = "share"
    REFERRAL = "referral"
    NFT_TRADE = "nft_trade"
    STAKE = "stake"

class DeviceType(str, Enum):
    """Device types"""
    MOBILE = "mobile"
    DESKTOP = "desktop"
    TABLET = "tablet"
    UNKNOWN = "unknown"

class PlatformType(str, Enum):
    """Social media platforms"""
    INSTAGRAM = "instagram"
    TIKTOK = "tiktok"
    YOUTUBE = "youtube"
    FACEBOOK = "facebook"
    TWITTER_X = "twitter_x"
    FINOVA_APP = "finova_app"

# ==================== BASE MODELS ====================

class BaseTimestamp(BaseModel):
    """Base model with timestamp fields"""
    created_at: datetime = Field(default_factory=datetime.utcnow)
    updated_at: Optional[datetime] = None

class UserIdentifier(BaseModel):
    """User identification fields"""
    user_id: str = Field(..., min_length=1, max_length=64)
    wallet_address: Optional[str] = Field(None, regex=r"^[A-Za-z0-9]{32,44}$")
    device_id: Optional[str] = Field(None, min_length=16, max_length=128)
    session_id: Optional[str] = Field(None, min_length=16, max_length=128)

# ==================== DEVICE & BIOMETRIC MODELS ====================

class DeviceFingerprint(BaseModel):
    """Device fingerprinting data"""
    device_type: DeviceType
    os_name: str = Field(..., max_length=50)
    os_version: str = Field(..., max_length=20)
    browser_name: Optional[str] = Field(None, max_length=50)
    browser_version: Optional[str] = Field(None, max_length=20)
    screen_resolution: str = Field(..., regex=r"^\d+x\d+$")
    timezone: str = Field(..., max_length=50)
    language: str = Field(..., max_length=10)
    user_agent: str = Field(..., max_length=500)
    hardware_concurrency: Optional[int] = Field(None, ge=1, le=64)
    memory_gb: Optional[float] = Field(None, ge=0.5, le=1024)
    touch_support: bool = False
    webgl_vendor: Optional[str] = Field(None, max_length=100)
    canvas_fingerprint: Optional[str] = Field(None, min_length=32, max_length=128)
    audio_fingerprint: Optional[str] = Field(None, min_length=32, max_length=128)

class BiometricData(BaseModel):
    """Biometric analysis data"""
    selfie_hash: Optional[str] = Field(None, min_length=64, max_length=64)
    face_encoding: Optional[List[float]] = Field(None, min_items=128, max_items=512)
    liveness_score: Optional[float] = Field(None, ge=0.0, le=1.0)
    face_quality_score: Optional[float] = Field(None, ge=0.0, le=1.0)
    face_uniqueness_score: Optional[float] = Field(None, ge=0.0, le=1.0)
    voice_print: Optional[List[float]] = Field(None, min_items=64, max_items=256)
    keystroke_patterns: Optional[List[Dict[str, float]]] = None
    mouse_movement_entropy: Optional[float] = Field(None, ge=0.0, le=10.0)
    touch_pressure_variance: Optional[float] = Field(None, ge=0.0, le=1.0)

# ==================== BEHAVIORAL MODELS ====================

class TemporalPattern(BaseModel):
    """Temporal behavior analysis"""
    action_timestamps: List[datetime] = Field(..., min_items=1, max_items=1000)
    session_duration_seconds: int = Field(..., ge=1, le=86400)
    actions_per_minute: float = Field(..., ge=0.0, le=1000.0)
    peak_activity_hours: List[int] = Field(..., min_items=0, max_items=24)
    weekend_activity_ratio: float = Field(..., ge=0.0, le=1.0)
    night_activity_ratio: float = Field(..., ge=0.0, le=1.0)
    session_gaps_minutes: List[int] = Field(default_factory=list)
    circadian_score: Optional[float] = Field(None, ge=0.0, le=1.0)
    temporal_entropy: Optional[float] = Field(None, ge=0.0, le=10.0)

class BehavioralMetrics(BaseModel):
    """User behavior metrics"""
    click_speed_avg_ms: float = Field(..., ge=50.0, le=5000.0)
    click_speed_variance: float = Field(..., ge=0.0, le=1000.0)
    scroll_speed_avg: float = Field(..., ge=0.0, le=10000.0)
    scroll_pattern_entropy: float = Field(..., ge=0.0, le=10.0)
    text_input_speed_wpm: Optional[float] = Field(None, ge=1.0, le=200.0)
    text_input_rhythm_score: Optional[float] = Field(None, ge=0.0, le=1.0)
    navigation_pattern_score: float = Field(..., ge=0.0, le=1.0)
    error_rate: float = Field(..., ge=0.0, le=1.0)
    retry_frequency: float = Field(..., ge=0.0, le=10.0)
    ui_interaction_diversity: float = Field(..., ge=0.0, le=1.0)

class NetworkBehavior(BaseModel):
    """Network and referral behavior"""
    referral_count: int = Field(..., ge=0, le=10000)
    active_referrals_30d: int = Field(..., ge=0, le=10000)
    referral_success_rate: float = Field(..., ge=0.0, le=1.0)
    network_diversity_score: float = Field(..., ge=0.0, le=1.0)
    circular_referral_score: float = Field(..., ge=0.0, le=1.0)
    referral_timing_entropy: float = Field(..., ge=0.0, le=10.0)
    cross_platform_connections: int = Field(..., ge=0, le=100)
    geographic_spread_km: Optional[float] = Field(None, ge=0.0, le=20000.0)
    referral_quality_score: float = Field(..., ge=0.0, le=1.0)

# ==================== CONTENT ANALYSIS MODELS ====================

class ContentMetrics(BaseModel):
    """Content quality and authenticity metrics"""
    originality_score: float = Field(..., ge=0.0, le=1.0)
    ai_generated_probability: float = Field(..., ge=0.0, le=1.0)
    duplicate_content_ratio: float = Field(..., ge=0.0, le=1.0)
    engagement_authenticity: float = Field(..., ge=0.0, le=1.0)
    content_quality_score: float = Field(..., ge=0.0, le=1.0)
    platform_relevance_score: float = Field(..., ge=0.0, le=1.0)
    brand_safety_score: float = Field(..., ge=0.0, le=1.0)
    sentiment_consistency: Optional[float] = Field(None, ge=-1.0, le=1.0)
    linguistic_complexity: Optional[float] = Field(None, ge=0.0, le=10.0)
    content_type_distribution: Dict[str, float] = Field(default_factory=dict)

class SocialActivity(BaseModel):
    """Social media activity data"""
    platform: PlatformType
    action_type: ActionType
    content_id: Optional[str] = Field(None, max_length=128)
    content_text: Optional[str] = Field(None, max_length=5000)
    media_hashes: Optional[List[str]] = Field(None, max_items=10)
    engagement_count: int = Field(..., ge=0, le=1000000)
    timestamp: datetime
    location_data: Optional[Dict[str, Any]] = None
    hashtags: Optional[List[str]] = Field(None, max_items=50)
    mentions: Optional[List[str]] = Field(None, max_items=100)

# ==================== REQUEST MODELS ====================

class BotDetectionRequest(UserIdentifier, BaseTimestamp):
    """Main bot detection analysis request"""
    analysis_types: List[DetectionType] = Field(..., min_items=1, max_items=6)
    device_fingerprint: Optional[DeviceFingerprint] = None
    biometric_data: Optional[BiometricData] = None
    temporal_pattern: Optional[TemporalPattern] = None
    behavioral_metrics: Optional[BehavioralMetrics] = None
    network_behavior: Optional[NetworkBehavior] = None
    content_metrics: Optional[ContentMetrics] = None
    recent_activities: Optional[List[SocialActivity]] = Field(None, max_items=100)
    ip_address: Optional[str] = Field(None, regex=r"^(?:[0-9]{1,3}\.){3}[0-9]{1,3}$")
    geolocation: Optional[Dict[str, Any]] = None
    context_data: Optional[Dict[str, Any]] = Field(default_factory=dict)

    @validator('analysis_types')
    def validate_analysis_types(cls, v):
        if not v or len(v) == 0:
            raise ValueError("At least one analysis type is required")
        return list(set(v))  # Remove duplicates

class BulkBotDetectionRequest(BaseModel):
    """Bulk bot detection request"""
    requests: List[BotDetectionRequest] = Field(..., min_items=1, max_items=100)
    priority: int = Field(default=5, ge=1, le=10)
    callback_url: Optional[str] = Field(None, regex=r"^https?://.*")

class PatternAnalysisRequest(BaseModel):
    """Pattern analysis for suspicious networks"""
    user_ids: List[str] = Field(..., min_items=2, max_items=1000)
    analysis_depth: int = Field(default=3, ge=1, le=5)
    time_range_hours: int = Field(default=24, ge=1, le=8760)  # Max 1 year
    include_inactive: bool = False

# ==================== RESPONSE MODELS ====================

class DetectionResult(BaseModel):
    """Individual detection result"""
    detection_type: DetectionType
    confidence_score: float = Field(..., ge=0.0, le=1.0)
    risk_level: RiskLevel
    evidence: List[str] = Field(default_factory=list)
    metrics: Dict[str, float] = Field(default_factory=dict)

class BotProbability(BaseModel):
    """Bot probability calculation"""
    overall_score: float = Field(..., ge=0.0, le=1.0, description="0=human, 1=bot")
    human_probability: float = Field(..., ge=0.0, le=1.0)
    bot_probability: float = Field(..., ge=0.0, le=1.0)
    confidence_level: float = Field(..., ge=0.0, le=1.0)
    risk_level: RiskLevel
    
    @root_validator
    def validate_probabilities(cls, values):
        human_prob = values.get('human_probability', 0)
        bot_prob = values.get('bot_probability', 0)
        if abs(human_prob + bot_prob - 1.0) > 0.01:
            raise ValueError("Human and bot probabilities must sum to 1.0")
        return values

class RecommendedActions(BaseModel):
    """Recommended actions based on analysis"""
    immediate_actions: List[str] = Field(default_factory=list)
    monitoring_actions: List[str] = Field(default_factory=list)
    verification_required: bool = False
    kyc_reverification: bool = False
    account_restrictions: List[str] = Field(default_factory=list)
    mining_penalties: Optional[Dict[str, float]] = None

class BotDetectionResponse(BaseTimestamp):
    """Main bot detection response"""
    request_id: str = Field(..., min_length=16, max_length=64)
    user_id: str = Field(..., min_length=1, max_length=64)
    analysis_completed: bool = True
    processing_time_ms: int = Field(..., ge=0, le=60000)
    
    # Core results
    bot_probability: BotProbability
    detection_results: List[DetectionResult] = Field(default_factory=list)
    recommended_actions: RecommendedActions
    
    # Detailed analysis
    detailed_scores: Dict[str, float] = Field(default_factory=dict)
    risk_factors: List[str] = Field(default_factory=list)
    positive_signals: List[str] = Field(default_factory=list)
    
    # Context
    analysis_version: str = Field(default="1.0.0")
    model_versions: Dict[str, str] = Field(default_factory=dict)
    expires_at: datetime = Field(default_factory=lambda: datetime.utcnow() + timedelta(hours=24))

class BulkBotDetectionResponse(BaseTimestamp):
    """Bulk bot detection response"""
    batch_id: str = Field(..., min_length=16, max_length=64)
    total_requests: int = Field(..., ge=1, le=100)
    completed_requests: int = Field(..., ge=0, le=100)
    failed_requests: int = Field(..., ge=0, le=100)
    results: List[BotDetectionResponse] = Field(default_factory=list)
    processing_time_ms: int = Field(..., ge=0, le=300000)  # Max 5 minutes
    batch_summary: Dict[str, Any] = Field(default_factory=dict)

class NetworkAnalysisResult(BaseModel):
    """Network pattern analysis result"""
    network_id: str = Field(..., min_length=16, max_length=64)
    suspicious_clusters: List[Dict[str, Any]] = Field(default_factory=list)
    circular_references: List[List[str]] = Field(default_factory=list)
    coordinated_behavior_score: float = Field(..., ge=0.0, le=1.0)
    network_authenticity_score: float = Field(..., ge=0.0, le=1.0)
    recommended_investigations: List[str] = Field(default_factory=list)

# ==================== MONITORING MODELS ====================

class AlertRule(BaseModel):
    """Alert rule configuration"""
    rule_id: str = Field(..., min_length=1, max_length=64)
    rule_name: str = Field(..., min_length=1, max_length=128)
    detection_type: DetectionType
    threshold_score: float = Field(..., ge=0.0, le=1.0)
    risk_level: RiskLevel
    enabled: bool = True
    notification_channels: List[str] = Field(default_factory=list)

class BotAlert(BaseTimestamp):
    """Bot detection alert"""
    alert_id: str = Field(..., min_length=16, max_length=64)
    rule_id: str = Field(..., min_length=1, max_length=64)
    user_id: str = Field(..., min_length=1, max_length=64)
    triggered_score: float = Field(..., ge=0.0, le=1.0)
    severity: RiskLevel
    alert_message: str = Field(..., min_length=1, max_length=500)
    evidence_summary: List[str] = Field(default_factory=list)
    auto_actions_taken: List[str] = Field(default_factory=list)
    requires_manual_review: bool = False
    acknowledged: bool = False
    acknowledged_by: Optional[str] = None
    acknowledged_at: Optional[datetime] = None

# ==================== ANALYTICS MODELS ====================

class DetectionMetrics(BaseModel):
    """Detection performance metrics"""
    time_period: str = Field(..., regex=r"^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}Z$")
    total_analyses: int = Field(..., ge=0)
    bot_detections: int = Field(..., ge=0)
    human_confirmations: int = Field(..., ge=0)
    false_positive_rate: float = Field(..., ge=0.0, le=1.0)
    false_negative_rate: float = Field(..., ge=0.0, le=1.0)
    accuracy: float = Field(..., ge=0.0, le=1.0)
    precision: float = Field(..., ge=0.0, le=1.0)
    recall: float = Field(..., ge=0.0, le=1.0)
    f1_score: float = Field(..., ge=0.0, le=1.0)
    avg_processing_time_ms: float = Field(..., ge=0.0)

# ==================== ERROR MODELS ====================

class ValidationError(BaseModel):
    """Validation error details"""
    field: str
    message: str
    invalid_value: Optional[Any] = None

class BotDetectionError(BaseModel):
    """Bot detection error response"""
    error_code: str = Field(..., min_length=1, max_length=32)
    error_message: str = Field(..., min_length=1, max_length=500)
    error_details: Optional[str] = None
    validation_errors: Optional[List[ValidationError]] = None
    request_id: Optional[str] = None
    timestamp: datetime = Field(default_factory=datetime.utcnow)
    retry_after_seconds: Optional[int] = Field(None, ge=1, le=3600)

# ==================== WEBHOOK MODELS ====================

class WebhookEvent(BaseModel):
    """Webhook event data"""
    event_id: str = Field(..., min_length=16, max_length=64)
    event_type: str = Field(..., min_length=1, max_length=64)
    user_id: str = Field(..., min_length=1, max_length=64)
    event_data: Dict[str, Any] = Field(default_factory=dict)
    timestamp: datetime = Field(default_factory=datetime.utcnow)
    signature: Optional[str] = Field(None, min_length=64, max_length=128)

class WebhookDelivery(BaseModel):
    """Webhook delivery status"""
    delivery_id: str = Field(..., min_length=16, max_length=64)
    webhook_url: str = Field(..., regex=r"^https?://.*")
    event: WebhookEvent
    delivery_status: str = Field(..., regex=r"^(pending|success|failed|retrying)$")
    response_code: Optional[int] = Field(None, ge=100, le=599)
    response_body: Optional[str] = Field(None, max_length=1000)
    attempts: int = Field(default=0, ge=0, le=10)
    next_retry_at: Optional[datetime] = None
    delivered_at: Optional[datetime] = None

# ==================== CONFIGURATION MODELS ====================

class ModelConfiguration(BaseModel):
    """ML model configuration"""
    model_name: str = Field(..., min_length=1, max_length=64)
    model_version: str = Field(..., regex=r"^\d+\.\d+\.\d+$")
    model_path: str = Field(..., min_length=1, max_length=256)
    confidence_threshold: float = Field(..., ge=0.0, le=1.0)
    feature_weights: Dict[str, float] = Field(default_factory=dict)
    preprocessing_config: Dict[str, Any] = Field(default_factory=dict)
    postprocessing_config: Dict[str, Any] = Field(default_factory=dict)
    enabled: bool = True
    last_trained: Optional[datetime] = None
    performance_metrics: Optional[DetectionMetrics] = None

# ==================== HEALTH CHECK MODELS ====================

class HealthStatus(BaseModel):
    """Service health status"""
    service_name: str = "bot-detection-service"
    version: str = Field(..., regex=r"^\d+\.\d+\.\d+$")
    status: str = Field(..., regex=r"^(healthy|degraded|unhealthy)$")
    timestamp: datetime = Field(default_factory=datetime.utcnow)
    uptime_seconds: int = Field(..., ge=0)
    dependencies: Dict[str, str] = Field(default_factory=dict)
    performance_metrics: Dict[str, float] = Field(default_factory=dict)
    error_rate_1h: float = Field(..., ge=0.0, le=1.0)
    avg_response_time_ms: float = Field(..., ge=0.0)
    active_connections: int = Field(..., ge=0)

# ==================== EXPORT MODELS ====================

__all__ = [
    # Enums
    "RiskLevel", "DetectionType", "ActionType", "DeviceType", "PlatformType",
    
    # Base Models
    "BaseTimestamp", "UserIdentifier",
    
    # Device & Biometric
    "DeviceFingerprint", "BiometricData",
    
    # Behavioral
    "TemporalPattern", "BehavioralMetrics", "NetworkBehavior",
    
    # Content
    "ContentMetrics", "SocialActivity",
    
    # Requests
    "BotDetectionRequest", "BulkBotDetectionRequest", "PatternAnalysisRequest",
    
    # Responses
    "DetectionResult", "BotProbability", "RecommendedActions",
    "BotDetectionResponse", "BulkBotDetectionResponse", "NetworkAnalysisResult",
    
    # Monitoring
    "AlertRule", "BotAlert",
    
    # Analytics
    "DetectionMetrics",
    
    # Errors
    "ValidationError", "BotDetectionError",
    
    # Webhooks
    "WebhookEvent", "WebhookDelivery",
    
    # Configuration
    "ModelConfiguration",
    
    # Health
    "HealthStatus"
]
