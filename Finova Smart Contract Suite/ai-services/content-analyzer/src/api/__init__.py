"""
Finova Network - Content Analyzer API Module
Enterprise-grade content quality assessment and analysis system
"""

from .routes import (
    ContentAnalysisRoutes,
    QualityAnalysisRoutes, 
    OriginalityRoutes,
    EngagementRoutes,
    BrandSafetyRoutes,
    BatchAnalysisRoutes,
    MetricsRoutes
)
from .schemas import (
    ContentAnalysisRequest,
    ContentAnalysisResponse,
    QualityScoreResponse,
    OriginalityCheckResponse,
    EngagementPredictionResponse,
    BrandSafetyResponse,
    BatchAnalysisRequest,
    BatchAnalysisResponse,
    AnalysisMetricsResponse,
    ErrorResponse,
    HealthCheckResponse
)
from .middleware import (
    AuthenticationMiddleware,
    RateLimitMiddleware,
    ValidationMiddleware,
    CacheMiddleware,
    LoggingMiddleware,
    ErrorHandlerMiddleware
)
from .validators import (
    ContentValidator,
    PlatformValidator,
    UserValidator,
    AnalysisValidator
)
from .serializers import (
    ContentSerializer,
    AnalysisResultSerializer,
    MetricsSerializer,
    ErrorSerializer
)

__version__ = "3.0.0"
__author__ = "Finova Network Team"
__email__ = "dev@finova.network"

# API Configuration
API_CONFIG = {
    "version": "v3",
    "title": "Finova Content Analyzer API",
    "description": "Advanced AI-powered content quality assessment system",
    "prefix": "/api/v3/content-analyzer",
    "rate_limits": {
        "default": "1000/hour",
        "premium": "5000/hour", 
        "enterprise": "unlimited"
    },
    "cache_ttl": {
        "quality_analysis": 3600,  # 1 hour
        "originality_check": 7200,  # 2 hours
        "engagement_prediction": 1800,  # 30 minutes
        "brand_safety": 86400  # 24 hours
    }
}

# Supported Content Types
SUPPORTED_CONTENT_TYPES = {
    "text": ["plain", "markdown", "html"],
    "image": ["jpeg", "png", "webp", "gif"],
    "video": ["mp4", "webm", "mov", "avi"],
    "audio": ["mp3", "wav", "ogg", "m4a"]
}

# Platform-specific configurations
PLATFORM_CONFIGS = {
    "instagram": {
        "max_text_length": 2200,
        "optimal_hashtags": 11,
        "image_aspect_ratios": ["1:1", "4:5", "9:16"],
        "quality_weights": {
            "visual": 0.6,
            "engagement": 0.3,
            "text": 0.1
        }
    },
    "tiktok": {
        "max_text_length": 150,
        "optimal_duration": [15, 60],
        "trending_sounds": True,
        "quality_weights": {
            "engagement": 0.5,
            "visual": 0.4,
            "trend_alignment": 0.1
        }
    },
    "youtube": {
        "max_title_length": 100,
        "optimal_description": 250,
        "thumbnail_importance": 0.8,
        "quality_weights": {
            "retention": 0.4,
            "engagement": 0.3,
            "visual": 0.2,
            "text": 0.1
        }
    },
    "facebook": {
        "max_text_length": 63206,
        "optimal_text": 80,
        "link_preview": True,
        "quality_weights": {
            "engagement": 0.4,
            "text": 0.3,
            "visual": 0.2,
            "shareability": 0.1
        }
    },
    "twitter": {
        "max_text_length": 280,
        "thread_support": True,
        "media_limit": 4,
        "quality_weights": {
            "engagement": 0.5,
            "text": 0.4,
            "timing": 0.1
        }
    }
}

# Quality scoring thresholds
QUALITY_THRESHOLDS = {
    "excellent": 0.9,
    "good": 0.7,
    "fair": 0.5,
    "poor": 0.3,
    "spam": 0.1
}

# Analysis models configuration
MODEL_CONFIGS = {
    "quality_classifier": {
        "model_name": "finova_quality_v3",
        "confidence_threshold": 0.85,
        "batch_size": 32,
        "max_tokens": 512
    },
    "originality_detector": {
        "model_name": "finova_originality_v2", 
        "similarity_threshold": 0.75,
        "hash_length": 256,
        "database_size": 10000000
    },
    "engagement_predictor": {
        "model_name": "finova_engagement_v3",
        "prediction_window": 168,  # 7 days in hours
        "feature_count": 128,
        "accuracy_target": 0.82
    },
    "brand_safety_checker": {
        "model_name": "finova_safety_v2",
        "risk_categories": [
            "adult_content",
            "violence", 
            "hate_speech",
            "misinformation",
            "copyright_violation",
            "spam"
        ],
        "severity_levels": ["low", "medium", "high", "critical"]
    }
}

# Error codes and messages
ERROR_CODES = {
    1000: "Invalid content format",
    1001: "Content too large",
    1002: "Unsupported content type",
    1003: "Missing required parameters",
    1004: "Rate limit exceeded",
    1005: "Analysis timeout",
    1006: "Model unavailable",
    1007: "Insufficient quality data",
    1008: "Platform not supported",
    1009: "User not authorized",
    1010: "Service temporarily unavailable"
}

# Success response templates  
SUCCESS_RESPONSES = {
    "analysis_complete": {
        "status": "success",
        "message": "Content analysis completed successfully"
    },
    "batch_complete": {
        "status": "success", 
        "message": "Batch analysis completed successfully"
    },
    "cache_hit": {
        "status": "success",
        "message": "Results retrieved from cache"
    }
}

# Export all public components
__all__ = [
    # Routes
    "ContentAnalysisRoutes",
    "QualityAnalysisRoutes",
    "OriginalityRoutes", 
    "EngagementRoutes",
    "BrandSafetyRoutes",
    "BatchAnalysisRoutes",
    "MetricsRoutes",
    
    # Schemas
    "ContentAnalysisRequest",
    "ContentAnalysisResponse",
    "QualityScoreResponse",
    "OriginalityCheckResponse",
    "EngagementPredictionResponse", 
    "BrandSafetyResponse",
    "BatchAnalysisRequest",
    "BatchAnalysisResponse",
    "AnalysisMetricsResponse",
    "ErrorResponse",
    "HealthCheckResponse",
    
    # Middleware
    "AuthenticationMiddleware",
    "RateLimitMiddleware",
    "ValidationMiddleware",
    "CacheMiddleware", 
    "LoggingMiddleware",
    "ErrorHandlerMiddleware",
    
    # Validators
    "ContentValidator",
    "PlatformValidator",
    "UserValidator", 
    "AnalysisValidator",
    
    # Serializers
    "ContentSerializer",
    "AnalysisResultSerializer",
    "MetricsSerializer",
    "ErrorSerializer",
    
    # Configuration
    "API_CONFIG",
    "SUPPORTED_CONTENT_TYPES",
    "PLATFORM_CONFIGS",
    "QUALITY_THRESHOLDS",
    "MODEL_CONFIGS",
    "ERROR_CODES",
    "SUCCESS_RESPONSES"
]
