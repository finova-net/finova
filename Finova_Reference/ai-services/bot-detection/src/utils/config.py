"""
Finova Network - Bot Detection Configuration
Enterprise-grade configuration management for AI-powered bot detection system
"""

import os
import logging
from dataclasses import dataclass, field
from typing import Dict, List, Optional, Tuple
from enum import Enum
import json
from pathlib import Path


class DetectionLevel(Enum):
    """Bot detection security levels"""
    LOW = "low"
    MEDIUM = "medium" 
    HIGH = "high"
    PARANOID = "paranoid"


class ModelType(Enum):
    """Available AI model types"""
    BEHAVIOR_ANALYZER = "behavior_analyzer"
    PATTERN_DETECTOR = "pattern_detector"
    NETWORK_ANALYZER = "network_analyzer"
    HUMAN_PROBABILITY = "human_probability"


@dataclass
class ModelConfig:
    """Configuration for individual ML models"""
    model_path: str
    model_version: str
    confidence_threshold: float = 0.7
    max_inference_time: int = 100  # milliseconds
    batch_size: int = 32
    use_gpu: bool = True
    warm_up: bool = True
    fallback_enabled: bool = True


@dataclass
class FeatureConfig:
    """Feature extraction configuration"""
    temporal_window: int = 3600  # seconds
    behavioral_patterns: List[str] = field(default_factory=lambda: [
        "click_intervals", "session_duration", "typing_speed",
        "mouse_movements", "scroll_patterns", "focus_changes"
    ])
    network_features: List[str] = field(default_factory=lambda: [
        "connection_clustering", "referral_patterns", "social_graph",
        "geographic_distribution", "device_fingerprints"
    ])
    device_features: List[str] = field(default_factory=lambda: [
        "browser_fingerprint", "screen_resolution", "timezone",
        "language_settings", "hardware_specs", "sensor_data"
    ])


@dataclass
class ThresholdConfig:
    """Detection threshold configuration"""
    human_probability_min: float = 0.6
    suspicious_activity_threshold: float = 0.8
    bot_confirmation_threshold: float = 0.95
    false_positive_tolerance: float = 0.02
    adaptive_learning: bool = True
    threshold_decay: float = 0.95  # daily


@dataclass
class PenaltyConfig:
    """Economic penalty configuration for bots"""
    mining_penalty_base: float = 0.85  # 15% reduction
    mining_penalty_max: float = 0.01   # 99% reduction
    xp_penalty_base: float = 0.90      # 10% reduction
    xp_penalty_max: float = 0.10       # 90% reduction
    rp_penalty_base: float = 0.92      # 8% reduction
    rp_penalty_max: float = 0.05       # 95% reduction
    progressive_scaling: float = 1.5


@dataclass
class SecurityConfig:
    """Security and privacy settings"""
    encrypt_user_data: bool = True
    anonymize_logs: bool = True
    gdpr_compliant: bool = True
    data_retention_days: int = 90
    audit_trail: bool = True
    rate_limit_per_minute: int = 1000
    max_concurrent_checks: int = 500


class FinovaBotDetectionConfig:
    """Main configuration class for Finova bot detection system"""
    
    def __init__(self, env: str = None):
        """Initialize configuration based on environment"""
        self.env = env or os.getenv('FINOVA_ENV', 'development')
        self.config_dir = Path(__file__).parent.parent.parent / 'config'
        
        # Core settings
        self.detection_level = DetectionLevel(os.getenv('DETECTION_LEVEL', 'high'))
        self.debug = os.getenv('DEBUG', 'false').lower() == 'true'
        self.version = "1.0.0"
        
        # Load environment-specific config
        self._load_environment_config()
        
        # Initialize configurations
        self.models = self._init_model_configs()
        self.features = self._init_feature_config()
        self.thresholds = self._init_threshold_config()
        self.penalties = self._init_penalty_config()
        self.security = self._init_security_config()
        
        # API Configuration
        self.api = self._init_api_config()
        
        # Database Configuration
        self.database = self._init_database_config()
        
        # Redis Configuration for caching
        self.redis = self._init_redis_config()
        
        # Monitoring Configuration
        self.monitoring = self._init_monitoring_config()
        
        # Logging Configuration
        self._init_logging()

    def _load_environment_config(self):
        """Load environment-specific configuration file"""
        config_file = self.config_dir / f"{self.env}.json"
        if config_file.exists():
            with open(config_file, 'r') as f:
                self.env_config = json.load(f)
        else:
            self.env_config = {}

    def _init_model_configs(self) -> Dict[ModelType, ModelConfig]:
        """Initialize ML model configurations"""
        base_path = os.getenv('MODEL_PATH', '/app/models')
        
        return {
            ModelType.BEHAVIOR_ANALYZER: ModelConfig(
                model_path=f"{base_path}/behavior_analyzer_v2.1.onnx",
                model_version="2.1",
                confidence_threshold=0.75,
                max_inference_time=80,
                batch_size=64 if self.env == 'production' else 16
            ),
            ModelType.PATTERN_DETECTOR: ModelConfig(
                model_path=f"{base_path}/pattern_detector_v1.8.onnx",
                model_version="1.8", 
                confidence_threshold=0.70,
                max_inference_time=60,
                batch_size=128 if self.env == 'production' else 32
            ),
            ModelType.NETWORK_ANALYZER: ModelConfig(
                model_path=f"{base_path}/network_analyzer_v1.5.onnx",
                model_version="1.5",
                confidence_threshold=0.80,
                max_inference_time=120,
                batch_size=32 if self.env == 'production' else 8
            ),
            ModelType.HUMAN_PROBABILITY: ModelConfig(
                model_path=f"{base_path}/human_probability_v3.0.onnx",
                model_version="3.0",
                confidence_threshold=0.65,
                max_inference_time=40,
                batch_size=256 if self.env == 'production' else 64
            )
        }

    def _init_feature_config(self) -> FeatureConfig:
        """Initialize feature extraction configuration"""
        if self.detection_level == DetectionLevel.PARANOID:
            temporal_window = 7200  # 2 hours
        elif self.detection_level == DetectionLevel.HIGH:
            temporal_window = 3600  # 1 hour
        else:
            temporal_window = 1800  # 30 minutes
            
        return FeatureConfig(temporal_window=temporal_window)

    def _init_threshold_config(self) -> ThresholdConfig:
        """Initialize detection threshold configuration"""
        if self.detection_level == DetectionLevel.PARANOID:
            return ThresholdConfig(
                human_probability_min=0.8,
                suspicious_activity_threshold=0.6,
                bot_confirmation_threshold=0.9,
                false_positive_tolerance=0.001
            )
        elif self.detection_level == DetectionLevel.HIGH:
            return ThresholdConfig(
                human_probability_min=0.7,
                suspicious_activity_threshold=0.7,
                bot_confirmation_threshold=0.92,
                false_positive_tolerance=0.01
            )
        elif self.detection_level == DetectionLevel.MEDIUM:
            return ThresholdConfig(
                human_probability_min=0.6,
                suspicious_activity_threshold=0.8,
                bot_confirmation_threshold=0.95,
                false_positive_tolerance=0.02
            )
        else:  # LOW
            return ThresholdConfig(
                human_probability_min=0.5,
                suspicious_activity_threshold=0.85,
                bot_confirmation_threshold=0.97,
                false_positive_tolerance=0.05
            )

    def _init_penalty_config(self) -> PenaltyConfig:
        """Initialize economic penalty configuration"""
        return PenaltyConfig(
            progressive_scaling=2.0 if self.detection_level == DetectionLevel.PARANOID else 1.5
        )

    def _init_security_config(self) -> SecurityConfig:
        """Initialize security configuration"""
        return SecurityConfig(
            rate_limit_per_minute=500 if self.env == 'production' else 1000,
            max_concurrent_checks=1000 if self.env == 'production' else 500,
            data_retention_days=30 if self.env == 'production' else 90
        )

    def _init_api_config(self) -> Dict:
        """Initialize API configuration"""
        return {
            'host': os.getenv('API_HOST', '0.0.0.0'),
            'port': int(os.getenv('API_PORT', 8083)),
            'workers': int(os.getenv('WORKERS', 4)),
            'timeout': int(os.getenv('TIMEOUT', 30)),
            'max_request_size': int(os.getenv('MAX_REQUEST_SIZE', 16777216)),  # 16MB
            'cors_origins': os.getenv('CORS_ORIGINS', '*').split(','),
            'api_key_required': os.getenv('API_KEY_REQUIRED', 'true').lower() == 'true',
            'rate_limit': {
                'requests_per_minute': int(os.getenv('RATE_LIMIT_RPM', 1000)),
                'burst_size': int(os.getenv('RATE_LIMIT_BURST', 100))
            }
        }

    def _init_database_config(self) -> Dict:
        """Initialize database configuration"""
        return {
            'host': os.getenv('DB_HOST', 'localhost'),
            'port': int(os.getenv('DB_PORT', 5432)),
            'database': os.getenv('DB_NAME', 'finova_bot_detection'),
            'username': os.getenv('DB_USER', 'finova'),
            'password': os.getenv('DB_PASSWORD', 'secure_password'),
            'pool_size': int(os.getenv('DB_POOL_SIZE', 20)),
            'max_overflow': int(os.getenv('DB_MAX_OVERFLOW', 30)),
            'pool_timeout': int(os.getenv('DB_POOL_TIMEOUT', 30)),
            'pool_recycle': int(os.getenv('DB_POOL_RECYCLE', 3600)),
            'echo': self.debug,
            'ssl_mode': os.getenv('DB_SSL_MODE', 'prefer')
        }

    def _init_redis_config(self) -> Dict:
        """Initialize Redis caching configuration"""
        return {
            'host': os.getenv('REDIS_HOST', 'localhost'),
            'port': int(os.getenv('REDIS_PORT', 6379)),
            'db': int(os.getenv('REDIS_DB', 2)),
            'password': os.getenv('REDIS_PASSWORD'),
            'socket_timeout': int(os.getenv('REDIS_SOCKET_TIMEOUT', 5)),
            'socket_connect_timeout': int(os.getenv('REDIS_CONNECT_TIMEOUT', 5)),
            'socket_keepalive': True,
            'socket_keepalive_options': {},
            'max_connections': int(os.getenv('REDIS_MAX_CONNECTIONS', 50)),
            'retry_on_timeout': True,
            'decode_responses': True,
            'prefix': f'finova:bot_detection:{self.env}',
            'default_ttl': int(os.getenv('REDIS_DEFAULT_TTL', 3600))
        }

    def _init_monitoring_config(self) -> Dict:
        """Initialize monitoring and metrics configuration"""
        return {
            'enabled': os.getenv('MONITORING_ENABLED', 'true').lower() == 'true',
            'prometheus_port': int(os.getenv('PROMETHEUS_PORT', 8084)),
            'metrics_prefix': 'finova_bot_detection',
            'alert_thresholds': {
                'error_rate': float(os.getenv('ALERT_ERROR_RATE', 0.05)),
                'response_time_p99': int(os.getenv('ALERT_RESPONSE_TIME', 1000)),
                'false_positive_rate': float(os.getenv('ALERT_FALSE_POSITIVE', 0.03)),
                'model_accuracy': float(os.getenv('ALERT_MODEL_ACCURACY', 0.85))
            },
            'health_check': {
                'interval': int(os.getenv('HEALTH_CHECK_INTERVAL', 60)),
                'timeout': int(os.getenv('HEALTH_CHECK_TIMEOUT', 10))
            }
        }

    def _init_logging(self):
        """Initialize logging configuration"""
        log_level = os.getenv('LOG_LEVEL', 'INFO' if self.env == 'production' else 'DEBUG')
        log_format = os.getenv('LOG_FORMAT', 'json' if self.env == 'production' else 'text')
        
        if log_format == 'json':
            formatter = logging.Formatter(
                '{"timestamp": "%(asctime)s", "level": "%(levelname)s", '
                '"module": "%(name)s", "message": "%(message)s", "env": "' + self.env + '"}'
            )
        else:
            formatter = logging.Formatter(
                '%(asctime)s - %(name)s - %(levelname)s - %(message)s'
            )
        
        handler = logging.StreamHandler()
        handler.setFormatter(formatter)
        
        logger = logging.getLogger('finova_bot_detection')
        logger.setLevel(getattr(logging, log_level))
        logger.addHandler(handler)
        
        self.logger = logger

    def get_model_config(self, model_type: ModelType) -> ModelConfig:
        """Get configuration for specific model type"""
        return self.models.get(model_type)

    def get_detection_weights(self) -> Dict[str, float]:
        """Get detection algorithm weights based on security level"""
        if self.detection_level == DetectionLevel.PARANOID:
            return {
                'behavior_analysis': 0.35,
                'pattern_detection': 0.30,
                'network_analysis': 0.25,
                'device_fingerprint': 0.10
            }
        elif self.detection_level == DetectionLevel.HIGH:
            return {
                'behavior_analysis': 0.30,
                'pattern_detection': 0.25,
                'network_analysis': 0.25,
                'device_fingerprint': 0.20
            }
        else:
            return {
                'behavior_analysis': 0.25,
                'pattern_detection': 0.25,
                'network_analysis': 0.25,
                'device_fingerprint': 0.25
            }

    def calculate_penalty_multiplier(self, suspicious_score: float, 
                                   total_earned_fin: float) -> Tuple[float, float, float]:
        """Calculate mining, XP, and RP penalty multipliers"""
        base_penalty = min(suspicious_score, 0.99)
        wealth_penalty = min(total_earned_fin / 100000, 0.5)  # Max 50% additional penalty
        
        combined_penalty = min(base_penalty + wealth_penalty, 0.99)
        scaling_factor = self.penalties.progressive_scaling
        
        mining_penalty = max(
            self.penalties.mining_penalty_max,
            self.penalties.mining_penalty_base * (1 - combined_penalty * scaling_factor)
        )
        
        xp_penalty = max(
            self.penalties.xp_penalty_max,
            self.penalties.xp_penalty_base * (1 - combined_penalty * scaling_factor * 0.8)
        )
        
        rp_penalty = max(
            self.penalties.rp_penalty_max,
            self.penalties.rp_penalty_base * (1 - combined_penalty * scaling_factor * 0.9)
        )
        
        return mining_penalty, xp_penalty, rp_penalty

    def is_production(self) -> bool:
        """Check if running in production environment"""
        return self.env == 'production'

    def is_development(self) -> bool:
        """Check if running in development environment"""
        return self.env == 'development'

    def get_cache_key(self, user_id: str, cache_type: str) -> str:
        """Generate standardized cache key"""
        return f"{self.redis['prefix']}:{cache_type}:{user_id}"

    def validate_configuration(self) -> List[str]:
        """Validate configuration and return list of issues"""
        issues = []
        
        # Validate model paths
        for model_type, config in self.models.items():
            if not os.path.exists(config.model_path):
                issues.append(f"Model file not found: {config.model_path}")
        
        # Validate thresholds
        if self.thresholds.human_probability_min >= self.thresholds.bot_confirmation_threshold:
            issues.append("Human probability min should be less than bot confirmation threshold")
        
        # Validate database connection
        if not self.database['host'] or not self.database['database']:
            issues.append("Database configuration incomplete")
        
        return issues


# Global configuration instance
config = FinovaBotDetectionConfig()

# Export commonly used configurations
MODEL_CONFIGS = config.models
FEATURE_CONFIG = config.features
THRESHOLD_CONFIG = config.thresholds
PENALTY_CONFIG = config.penalties
SECURITY_CONFIG = config.security

# Helper functions for common operations
def get_human_probability_threshold() -> float:
    """Get current human probability threshold"""
    return config.thresholds.human_probability_min

def get_bot_confirmation_threshold() -> float:
    """Get current bot confirmation threshold"""
    return config.thresholds.bot_confirmation_threshold

def is_suspicious(probability: float) -> bool:
    """Check if probability indicates suspicious activity"""
    return probability >= config.thresholds.suspicious_activity_threshold

def is_likely_bot(probability: float) -> bool:
    """Check if probability indicates likely bot"""
    return probability >= config.thresholds.bot_confirmation_threshold

def calculate_economic_penalties(suspicious_score: float, total_fin: float) -> Dict[str, float]:
    """Calculate all economic penalties for a user"""
    mining, xp, rp = config.calculate_penalty_multiplier(suspicious_score, total_fin)
    return {
        'mining_multiplier': mining,
        'xp_multiplier': xp,
        'rp_multiplier': rp,
        'suspicious_score': suspicious_score
    }

if __name__ == "__main__":
    # Configuration validation and testing
    issues = config.validate_configuration()
    if issues:
        print("Configuration Issues Found:")
        for issue in issues:
            print(f"  - {issue}")
    else:
        print(f"Configuration validated successfully for {config.env} environment")
        print(f"Detection Level: {config.detection_level.value}")
        print(f"Models configured: {len(config.models)}")
        