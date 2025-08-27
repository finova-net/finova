"""
Finova Network - AI Content Analyzer Configuration
Enterprise-grade configuration management for content quality assessment
"""

import os
from typing import Dict, Any, List, Optional
from dataclasses import dataclass
from enum import Enum
import logging
from pathlib import Path


class Environment(Enum):
    """Environment types for configuration"""
    DEVELOPMENT = "development"
    STAGING = "staging"
    PRODUCTION = "production"


class ContentType(Enum):
    """Supported content types for analysis"""
    TEXT = "text"
    IMAGE = "image"
    VIDEO = "video"
    AUDIO = "audio"
    MIXED = "mixed"


class Platform(Enum):
    """Supported social media platforms"""
    INSTAGRAM = "instagram"
    TIKTOK = "tiktok"
    YOUTUBE = "youtube"
    FACEBOOK = "facebook"
    TWITTER_X = "twitter"
    UNKNOWN = "unknown"


@dataclass
class ModelConfig:
    """Configuration for ML models"""
    name: str
    version: str
    path: str
    threshold: float
    enabled: bool = True
    batch_size: int = 32
    max_tokens: int = 512


@dataclass
class QualityWeights:
    """Quality assessment weights for different factors"""
    originality: float = 0.25
    engagement_potential: float = 0.20
    platform_relevance: float = 0.15
    brand_safety: float = 0.20
    human_generated: float = 0.20


class FinovaAIConfig:
    """Main configuration class for Finova AI Content Analyzer"""
    
    def __init__(self, env: Environment = None):
        self.env = env or Environment(os.getenv('FINOVA_ENV', 'development'))
        self._load_config()
        self._setup_logging()
    
    def _load_config(self):
        """Load configuration based on environment"""
        
        # Base configuration
        self.APP_NAME = "finova-content-analyzer"
        self.VERSION = "1.0.0"
        self.DEBUG = self.env == Environment.DEVELOPMENT
        
        # API Configuration
        self.API_HOST = os.getenv('API_HOST', '0.0.0.0')
        self.API_PORT = int(os.getenv('API_PORT', 8001))
        self.API_PREFIX = "/api/v1"
        self.API_TITLE = "Finova AI Content Analyzer"
        self.API_DESCRIPTION = "AI-powered content quality assessment for social mining"
        
        # Database Configuration
        self.DATABASE_URL = os.getenv(
            'DATABASE_URL',
            'postgresql://finova:finova123@localhost:5432/finova_ai'
        )
        self.REDIS_URL = os.getenv('REDIS_URL', 'redis://localhost:6379/0')
        
        # Model Configurations
        self.MODELS = {
            'quality_classifier': ModelConfig(
                name="quality_classifier",
                version="1.2.0",
                path=os.getenv('QUALITY_MODEL_PATH', './models/quality_v1_2.onnx'),
                threshold=0.7,
                batch_size=64,
                max_tokens=512
            ),
            'originality_detector': ModelConfig(
                name="originality_detector",
                version="1.1.0",
                path=os.getenv('ORIGINALITY_MODEL_PATH', './models/originality_v1_1.onnx'),
                threshold=0.8,
                batch_size=32,
                max_tokens=1024
            ),
            'engagement_predictor': ModelConfig(
                name="engagement_predictor",
                version="1.0.0",
                path=os.getenv('ENGAGEMENT_MODEL_PATH', './models/engagement_v1_0.onnx'),
                threshold=0.6,
                batch_size=128,
                max_tokens=256
            ),
            'brand_safety_checker': ModelConfig(
                name="brand_safety_checker",
                version="2.0.0",
                path=os.getenv('SAFETY_MODEL_PATH', './models/safety_v2_0.onnx'),
                threshold=0.9,
                batch_size=64,
                max_tokens=512
            ),
            'ai_detector': ModelConfig(
                name="ai_detector",
                version="1.3.0",
                path=os.getenv('AI_DETECTOR_PATH', './models/ai_detector_v1_3.onnx'),
                threshold=0.75,
                batch_size=32,
                max_tokens=2048
            )
        }
        
        # Quality Assessment Configuration
        self.QUALITY_WEIGHTS = QualityWeights(
            originality=float(os.getenv('WEIGHT_ORIGINALITY', 0.25)),
            engagement_potential=float(os.getenv('WEIGHT_ENGAGEMENT', 0.20)),
            platform_relevance=float(os.getenv('WEIGHT_RELEVANCE', 0.15)),
            brand_safety=float(os.getenv('WEIGHT_SAFETY', 0.20)),
            human_generated=float(os.getenv('WEIGHT_HUMAN', 0.20))
        )
        
        # Content Processing Configuration
        self.MAX_CONTENT_LENGTH = int(os.getenv('MAX_CONTENT_LENGTH', 10000))
        self.MAX_IMAGE_SIZE = int(os.getenv('MAX_IMAGE_SIZE', 10 * 1024 * 1024))  # 10MB
        self.MAX_VIDEO_SIZE = int(os.getenv('MAX_VIDEO_SIZE', 100 * 1024 * 1024))  # 100MB
        self.SUPPORTED_IMAGE_FORMATS = ['jpg', 'jpeg', 'png', 'webp', 'gif']
        self.SUPPORTED_VIDEO_FORMATS = ['mp4', 'webm', 'mov', 'avi', 'mkv']
        
        # Platform-specific configurations
        self.PLATFORM_CONFIG = {
            Platform.INSTAGRAM: {
                'optimal_hashtag_count': (5, 15),
                'optimal_length': (50, 300),
                'engagement_multiplier': 1.2,
                'quality_threshold': 0.7
            },
            Platform.TIKTOK: {
                'optimal_hashtag_count': (3, 8),
                'optimal_length': (20, 150),
                'engagement_multiplier': 1.3,
                'quality_threshold': 0.6
            },
            Platform.YOUTUBE: {
                'optimal_hashtag_count': (3, 10),
                'optimal_length': (100, 1000),
                'engagement_multiplier': 1.4,
                'quality_threshold': 0.8
            },
            Platform.FACEBOOK: {
                'optimal_hashtag_count': (1, 5),
                'optimal_length': (80, 500),
                'engagement_multiplier': 1.1,
                'quality_threshold': 0.7
            },
            Platform.TWITTER_X: {
                'optimal_hashtag_count': (1, 4),
                'optimal_length': (10, 280),
                'engagement_multiplier': 1.2,
                'quality_threshold': 0.6
            }
        }
        
        # Security Configuration
        self.API_KEY_HEADER = "X-Finova-API-Key"
        self.API_KEYS = os.getenv('API_KEYS', '').split(',') if os.getenv('API_KEYS') else []
        self.RATE_LIMIT_PER_MINUTE = int(os.getenv('RATE_LIMIT_PER_MINUTE', 100))
        self.RATE_LIMIT_PER_HOUR = int(os.getenv('RATE_LIMIT_PER_HOUR', 1000))
        
        # Cache Configuration
        self.CACHE_TTL = int(os.getenv('CACHE_TTL', 3600))  # 1 hour
        self.CACHE_MAX_SIZE = int(os.getenv('CACHE_MAX_SIZE', 10000))
        
        # Monitoring Configuration
        self.METRICS_ENABLED = os.getenv('METRICS_ENABLED', 'true').lower() == 'true'
        self.METRICS_PORT = int(os.getenv('METRICS_PORT', 8002))
        self.LOG_LEVEL = os.getenv('LOG_LEVEL', 'INFO')
        self.SENTRY_DSN = os.getenv('SENTRY_DSN')
        
        # AI Model Performance Thresholds
        self.PERFORMANCE_THRESHOLDS = {
            'min_accuracy': float(os.getenv('MIN_ACCURACY', 0.85)),
            'max_inference_time': float(os.getenv('MAX_INFERENCE_TIME', 2.0)),
            'max_memory_usage': int(os.getenv('MAX_MEMORY_MB', 2048)),
            'alert_threshold': float(os.getenv('ALERT_THRESHOLD', 0.1))  # 10% performance drop
        }
        
        # Blockchain Integration
        self.SOLANA_RPC_URL = os.getenv('SOLANA_RPC_URL', 'https://api.devnet.solana.com')
        self.FINOVA_PROGRAM_ID = os.getenv('FINOVA_PROGRAM_ID', '')
        self.WALLET_PRIVATE_KEY = os.getenv('WALLET_PRIVATE_KEY', '')
        
        # Feature Flags
        self.FEATURE_FLAGS = {
            'advanced_ai_detection': os.getenv('FEATURE_ADVANCED_AI', 'true').lower() == 'true',
            'real_time_analysis': os.getenv('FEATURE_REALTIME', 'true').lower() == 'true',
            'batch_processing': os.getenv('FEATURE_BATCH', 'true').lower() == 'true',
            'experimental_models': os.getenv('FEATURE_EXPERIMENTAL', 'false').lower() == 'true',
            'detailed_metrics': os.getenv('FEATURE_DETAILED_METRICS', 'true').lower() == 'true'
        }
        
        # Environment-specific overrides
        if self.env == Environment.PRODUCTION:
            self._apply_production_config()
        elif self.env == Environment.STAGING:
            self._apply_staging_config()
        else:
            self._apply_development_config()
    
    def _apply_production_config(self):
        """Apply production-specific configurations"""
        self.DEBUG = False
        self.LOG_LEVEL = "WARNING"
        self.RATE_LIMIT_PER_MINUTE = 50
        self.RATE_LIMIT_PER_HOUR = 500
        
        # Stricter thresholds for production
        for model_name, model in self.MODELS.items():
            if model_name == 'brand_safety_checker':
                model.threshold = 0.95
            elif model_name == 'ai_detector':
                model.threshold = 0.8
        
        self.PERFORMANCE_THRESHOLDS['min_accuracy'] = 0.9
        self.PERFORMANCE_THRESHOLDS['max_inference_time'] = 1.5
    
    def _apply_staging_config(self):
        """Apply staging-specific configurations"""
        self.DEBUG = False
        self.LOG_LEVEL = "INFO"
        self.RATE_LIMIT_PER_MINUTE = 200
        self.FEATURE_FLAGS['experimental_models'] = True
    
    def _apply_development_config(self):
        """Apply development-specific configurations"""
        self.DEBUG = True
        self.LOG_LEVEL = "DEBUG"
        self.RATE_LIMIT_PER_MINUTE = 1000
        self.RATE_LIMIT_PER_HOUR = 10000
        self.FEATURE_FLAGS['experimental_models'] = True
        self.FEATURE_FLAGS['detailed_metrics'] = True
    
    def _setup_logging(self):
        """Setup logging configuration"""
        logging.basicConfig(
            level=getattr(logging, self.LOG_LEVEL),
            format='%(asctime)s - %(name)s - %(levelname)s - %(message)s',
            handlers=[
                logging.StreamHandler(),
                logging.FileHandler(f'logs/{self.APP_NAME}.log', encoding='utf-8')
            ] if Path('logs').exists() else [logging.StreamHandler()]
        )
    
    def get_model_config(self, model_name: str) -> Optional[ModelConfig]:
        """Get configuration for specific model"""
        return self.MODELS.get(model_name)
    
    def get_platform_config(self, platform: Platform) -> Dict[str, Any]:
        """Get platform-specific configuration"""
        return self.PLATFORM_CONFIG.get(platform, self.PLATFORM_CONFIG[Platform.INSTAGRAM])
    
    def is_feature_enabled(self, feature_name: str) -> bool:
        """Check if a feature flag is enabled"""
        return self.FEATURE_FLAGS.get(feature_name, False)
    
    def validate_config(self) -> List[str]:
        """Validate configuration and return list of issues"""
        issues = []
        
        # Validate model paths
        for model_name, model_config in self.MODELS.items():
            if not os.path.exists(model_config.path):
                issues.append(f"Model file not found: {model_config.path}")
        
        # Validate API keys in production
        if self.env == Environment.PRODUCTION and not self.API_KEYS:
            issues.append("API keys not configured for production")
        
        # Validate quality weights sum to 1.0
        weights_sum = sum([
            self.QUALITY_WEIGHTS.originality,
            self.QUALITY_WEIGHTS.engagement_potential,
            self.QUALITY_WEIGHTS.platform_relevance,
            self.QUALITY_WEIGHTS.brand_safety,
            self.QUALITY_WEIGHTS.human_generated
        ])
        if abs(weights_sum - 1.0) > 0.01:
            issues.append(f"Quality weights sum to {weights_sum}, should be 1.0")
        
        # Validate database URL
        if not self.DATABASE_URL.startswith(('postgresql://', 'sqlite://')):
            issues.append("Invalid database URL format")
        
        return issues
    
    def get_quality_multiplier(self, quality_scores: Dict[str, float]) -> float:
        """Calculate quality multiplier based on scores and weights"""
        if not quality_scores:
            return 1.0
        
        weighted_score = (
            quality_scores.get('originality', 0.5) * self.QUALITY_WEIGHTS.originality +
            quality_scores.get('engagement_potential', 0.5) * self.QUALITY_WEIGHTS.engagement_potential +
            quality_scores.get('platform_relevance', 0.5) * self.QUALITY_WEIGHTS.platform_relevance +
            quality_scores.get('brand_safety', 0.5) * self.QUALITY_WEIGHTS.brand_safety +
            quality_scores.get('human_generated', 0.5) * self.QUALITY_WEIGHTS.human_generated
        )
        
        # Clamp between 0.5x and 2.0x as per whitepaper
        return max(0.5, min(2.0, weighted_score))
    
    def to_dict(self) -> Dict[str, Any]:
        """Convert configuration to dictionary"""
        return {
            'app_name': self.APP_NAME,
            'version': self.VERSION,
            'environment': self.env.value,
            'debug': self.DEBUG,
            'api': {
                'host': self.API_HOST,
                'port': self.API_PORT,
                'prefix': self.API_PREFIX
            },
            'models': {name: {
                'name': config.name,
                'version': config.version,
                'threshold': config.threshold,
                'enabled': config.enabled
            } for name, config in self.MODELS.items()},
            'quality_weights': {
                'originality': self.QUALITY_WEIGHTS.originality,
                'engagement_potential': self.QUALITY_WEIGHTS.engagement_potential,
                'platform_relevance': self.QUALITY_WEIGHTS.platform_relevance,
                'brand_safety': self.QUALITY_WEIGHTS.brand_safety,
                'human_generated': self.QUALITY_WEIGHTS.human_generated
            },
            'feature_flags': self.FEATURE_FLAGS,
            'rate_limits': {
                'per_minute': self.RATE_LIMIT_PER_MINUTE,
                'per_hour': self.RATE_LIMIT_PER_HOUR
            }
        }


# Global configuration instance
config = FinovaAIConfig()


# Configuration validation on import
if __name__ == "__main__":
    issues = config.validate_config()
    if issues:
        print("Configuration issues found:")
        for issue in issues:
            print(f"  - {issue}")
    else:
        print("Configuration validated successfully!")
        print(f"Environment: {config.env.value}")
        print(f"Models loaded: {len(config.MODELS)}")
        print(f"Feature flags: {sum(config.FEATURE_FLAGS.values())} enabled")