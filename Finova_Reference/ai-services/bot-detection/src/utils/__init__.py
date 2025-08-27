"""
Finova Network Bot Detection Utils Module
=========================================

Enterprise-grade utilities for bot detection AI service.
Provides configuration management, helper functions, and shared utilities.

Author: Finova Network Team
Version: 1.0.0
License: MIT
"""

import os
import json
import logging
import hashlib
import secrets
from datetime import datetime, timedelta
from typing import Dict, List, Optional, Any, Union, Tuple
from dataclasses import dataclass
from pathlib import Path
import numpy as np
import pandas as pd
from cryptography.fernet import Fernet
from redis import Redis
import jwt

# Version info
__version__ = "1.0.0"
__author__ = "Finova Network Team"

# Export main utilities
__all__ = [
    # Configuration
    'Config',
    'BotDetectionConfig',
    'load_config',
    'get_env_config',
    
    # Cryptography utilities
    'SecurityUtils',
    'encrypt_data',
    'decrypt_data',
    'generate_token',
    'verify_token',
    
    # Data processing utilities
    'DataProcessor',
    'normalize_features',
    'calculate_scores',
    'aggregate_metrics',
    
    # Cache utilities
    'CacheManager',
    'get_cached_result',
    'set_cache_result',
    'invalidate_cache',
    
    # Validation utilities
    'ValidationUtils',
    'validate_user_data',
    'sanitize_input',
    'check_rate_limits',
    
    # Mathematical utilities
    'MathUtils',
    'calculate_entropy',
    'compute_behavioral_score',
    'exponential_regression',
    
    # Logging utilities
    'setup_logger',
    'log_detection_event',
    'audit_log',
    
    # Constants
    'DETECTION_THRESHOLDS',
    'MODEL_CONFIGS',
    'FEATURE_WEIGHTS'
]

# =============================================================================
# Configuration Classes
# =============================================================================

@dataclass
class BotDetectionConfig:
    """Bot detection configuration parameters."""
    
    # Model parameters
    human_threshold: float = 0.7
    bot_threshold: float = 0.3
    suspicious_threshold: float = 0.5
    
    # Feature weights
    temporal_weight: float = 0.25
    behavioral_weight: float = 0.30
    network_weight: float = 0.20
    device_weight: float = 0.25
    
    # Analysis windows
    short_window_hours: int = 1
    medium_window_hours: int = 24
    long_window_hours: int = 168  # 7 days
    
    # Rate limiting
    max_requests_per_minute: int = 60
    max_requests_per_hour: int = 1000
    
    # Cache settings
    cache_ttl_seconds: int = 300  # 5 minutes
    batch_size: int = 100
    
    # Security
    encryption_enabled: bool = True
    audit_logging: bool = True
    
    def __post_init__(self):
        """Validate configuration after initialization."""
        if self.human_threshold <= self.bot_threshold:
            raise ValueError("Human threshold must be > bot threshold")
        
        total_weight = (self.temporal_weight + self.behavioral_weight + 
                       self.network_weight + self.device_weight)
        if abs(total_weight - 1.0) > 0.01:
            raise ValueError(f"Feature weights must sum to 1.0, got {total_weight}")

class Config:
    """Main configuration manager."""
    
    def __init__(self, config_path: Optional[str] = None):
        self.config_path = config_path or os.getenv('BOT_DETECTION_CONFIG', 'config.json')
        self._config = self._load_config()
        self.bot_detection = BotDetectionConfig(**self._config.get('bot_detection', {}))
    
    def _load_config(self) -> Dict[str, Any]:
        """Load configuration from file or environment."""
        if os.path.exists(self.config_path):
            with open(self.config_path, 'r') as f:
                return json.load(f)
        
        # Fallback to environment variables
        return {
            'redis': {
                'host': os.getenv('REDIS_HOST', 'localhost'),
                'port': int(os.getenv('REDIS_PORT', 6379)),
                'db': int(os.getenv('REDIS_DB', 0)),
                'password': os.getenv('REDIS_PASSWORD')
            },
            'database': {
                'url': os.getenv('DATABASE_URL'),
                'pool_size': int(os.getenv('DB_POOL_SIZE', 10))
            },
            'ml_models': {
                'model_path': os.getenv('MODEL_PATH', './models'),
                'device': os.getenv('ML_DEVICE', 'cpu'),
                'batch_size': int(os.getenv('ML_BATCH_SIZE', 32))
            }
        }
    
    @property
    def redis_config(self) -> Dict[str, Any]:
        return self._config.get('redis', {})
    
    @property 
    def database_config(self) -> Dict[str, Any]:
        return self._config.get('database', {})
    
    @property
    def ml_config(self) -> Dict[str, Any]:
        return self._config.get('ml_models', {})

# =============================================================================
# Security Utilities
# =============================================================================

class SecurityUtils:
    """Cryptographic and security utility functions."""
    
    def __init__(self):
        self._key = self._get_or_create_key()
        self._cipher = Fernet(self._key)
    
    def _get_or_create_key(self) -> bytes:
        """Get existing key or create new one."""
        key_path = Path('.secret_key')
        if key_path.exists():
            return key_path.read_bytes()
        
        key = Fernet.generate_key()
        key_path.write_bytes(key)
        return key
    
    def encrypt_data(self, data: str) -> str:
        """Encrypt sensitive data."""
        return self._cipher.encrypt(data.encode()).decode()
    
    def decrypt_data(self, encrypted_data: str) -> str:
        """Decrypt sensitive data."""
        return self._cipher.decrypt(encrypted_data.encode()).decode()
    
    def generate_token(self, payload: Dict[str, Any], 
                      expires_hours: int = 24) -> str:
        """Generate JWT token."""
        payload['exp'] = datetime.utcnow() + timedelta(hours=expires_hours)
        payload['iat'] = datetime.utcnow()
        
        secret = os.getenv('JWT_SECRET', secrets.token_urlsafe(32))
        return jwt.encode(payload, secret, algorithm='HS256')
    
    def verify_token(self, token: str) -> Optional[Dict[str, Any]]:
        """Verify and decode JWT token."""
        try:
            secret = os.getenv('JWT_SECRET', secrets.token_urlsafe(32))
            return jwt.decode(token, secret, algorithms=['HS256'])
        except jwt.InvalidTokenError:
            return None
    
    @staticmethod
    def hash_user_id(user_id: str) -> str:
        """Create privacy-preserving hash of user ID."""
        salt = os.getenv('USER_HASH_SALT', 'finova_default_salt').encode()
        return hashlib.pbkdf2_hmac('sha256', user_id.encode(), salt, 100000).hex()

# =============================================================================
# Data Processing Utilities  
# =============================================================================

class DataProcessor:
    """Data processing and feature engineering utilities."""
    
    @staticmethod
    def normalize_features(features: Dict[str, float]) -> Dict[str, float]:
        """Normalize feature values to [0, 1] range."""
        normalized = {}
        
        # Define normalization ranges for different feature types
        ranges = {
            'click_speed': (0.1, 2.0),  # clicks per second
            'session_duration': (60, 7200),  # 1 minute to 2 hours
            'activity_variance': (0, 1),  # already normalized
            'network_size': (0, 1000),  # max reasonable network size
            'device_consistency': (0, 1)  # already normalized
        }
        
        for key, value in features.items():
            if key in ranges:
                min_val, max_val = ranges[key]
                normalized[key] = max(0, min(1, (value - min_val) / (max_val - min_val)))
            else:
                normalized[key] = max(0, min(1, value))
        
        return normalized
    
    @staticmethod
    def calculate_scores(features: Dict[str, float], 
                        weights: Dict[str, float]) -> float:
        """Calculate weighted score from features."""
        score = 0.0
        total_weight = 0.0
        
        for feature, value in features.items():
            if feature in weights:
                score += value * weights[feature]
                total_weight += weights[feature]
        
        return score / total_weight if total_weight > 0 else 0.5
    
    @staticmethod
    def aggregate_metrics(metrics_list: List[Dict[str, float]]) -> Dict[str, float]:
        """Aggregate multiple metric dictionaries."""
        if not metrics_list:
            return {}
        
        aggregated = {}
        keys = metrics_list[0].keys()
        
        for key in keys:
            values = [m.get(key, 0) for m in metrics_list]
            aggregated[f'{key}_mean'] = np.mean(values)
            aggregated[f'{key}_std'] = np.std(values)
            aggregated[f'{key}_min'] = np.min(values)
            aggregated[f'{key}_max'] = np.max(values)
        
        return aggregated

# =============================================================================
# Cache Management
# =============================================================================

class CacheManager:
    """Redis-based caching utilities."""
    
    def __init__(self, redis_config: Dict[str, Any]):
        self.redis_client = Redis(**redis_config)
        self.default_ttl = 300  # 5 minutes
    
    def get_cached_result(self, cache_key: str) -> Optional[Dict[str, Any]]:
        """Retrieve cached result."""
        try:
            cached_data = self.redis_client.get(cache_key)
            return json.loads(cached_data) if cached_data else None
        except Exception as e:
            logging.warning(f"Cache retrieval failed: {e}")
            return None
    
    def set_cache_result(self, cache_key: str, data: Dict[str, Any], 
                        ttl: Optional[int] = None) -> bool:
        """Cache result with TTL."""
        try:
            ttl = ttl or self.default_ttl
            self.redis_client.setex(
                cache_key, 
                ttl, 
                json.dumps(data, default=str)
            )
            return True
        except Exception as e:
            logging.warning(f"Cache storage failed: {e}")
            return False
    
    def invalidate_cache(self, pattern: str) -> int:
        """Invalidate cache entries matching pattern."""
        try:
            keys = self.redis_client.keys(pattern)
            return self.redis_client.delete(*keys) if keys else 0
        except Exception as e:
            logging.warning(f"Cache invalidation failed: {e}")
            return 0
    
    def generate_cache_key(self, user_id: str, analysis_type: str, 
                          timestamp: Optional[datetime] = None) -> str:
        """Generate standardized cache key."""
        timestamp = timestamp or datetime.utcnow()
        hour_key = timestamp.strftime('%Y%m%d%H')
        return f"bot_detection:{analysis_type}:{user_id}:{hour_key}"

# =============================================================================
# Validation Utilities
# =============================================================================

class ValidationUtils:
    """Input validation and sanitization utilities."""
    
    @staticmethod
    def validate_user_data(user_data: Dict[str, Any]) -> Tuple[bool, List[str]]:
        """Validate user data structure and content."""
        errors = []
        required_fields = ['user_id', 'timestamp', 'activity_type']
        
        # Check required fields
        for field in required_fields:
            if field not in user_data:
                errors.append(f"Missing required field: {field}")
        
        # Validate user_id
        if 'user_id' in user_data:
            user_id = user_data['user_id']
            if not isinstance(user_id, str) or len(user_id) < 3:
                errors.append("user_id must be string with length >= 3")
        
        # Validate timestamp
        if 'timestamp' in user_data:
            try:
                if isinstance(user_data['timestamp'], str):
                    datetime.fromisoformat(user_data['timestamp'])
            except (ValueError, TypeError):
                errors.append("Invalid timestamp format")
        
        return len(errors) == 0, errors
    
    @staticmethod
    def sanitize_input(data: Any) -> Any:
        """Sanitize input data to prevent injection attacks."""
        if isinstance(data, str):
            # Remove potentially dangerous characters
            dangerous_chars = ['<', '>', '&', '"', "'", ';', '(', ')']
            sanitized = data
            for char in dangerous_chars:
                sanitized = sanitized.replace(char, '')
            return sanitized.strip()
        
        elif isinstance(data, dict):
            return {k: ValidationUtils.sanitize_input(v) for k, v in data.items()}
        
        elif isinstance(data, list):
            return [ValidationUtils.sanitize_input(item) for item in data]
        
        return data
    
    @staticmethod
    def check_rate_limits(user_id: str, redis_client: Redis, 
                         max_per_minute: int = 60) -> Tuple[bool, int]:
        """Check if user is within rate limits."""
        key = f"rate_limit:{user_id}:{datetime.utcnow().strftime('%Y%m%d%H%M')}"
        current_count = redis_client.incr(key)
        
        if current_count == 1:
            redis_client.expire(key, 60)  # Expire after 1 minute
        
        remaining = max(0, max_per_minute - current_count)
        return current_count <= max_per_minute, remaining

# =============================================================================
# Mathematical Utilities
# =============================================================================

class MathUtils:
    """Mathematical utility functions for bot detection."""
    
    @staticmethod
    def calculate_entropy(values: List[float]) -> float:
        """Calculate Shannon entropy of values."""
        if not values:
            return 0.0
        
        # Create histogram
        hist, _ = np.histogram(values, bins=10, density=True)
        hist = hist[hist > 0]  # Remove zero bins
        
        # Calculate entropy
        entropy = -np.sum(hist * np.log2(hist))
        return entropy
    
    @staticmethod
    def compute_behavioral_score(activities: List[Dict[str, Any]]) -> float:
        """Compute behavioral consistency score."""
        if len(activities) < 2:
            return 0.5  # Neutral score for insufficient data
        
        # Analyze temporal patterns
        timestamps = [a.get('timestamp', datetime.utcnow()) for a in activities]
        intervals = []
        
        for i in range(1, len(timestamps)):
            if isinstance(timestamps[i], str):
                timestamps[i] = datetime.fromisoformat(timestamps[i])
            if isinstance(timestamps[i-1], str):
                timestamps[i-1] = datetime.fromisoformat(timestamps[i-1])
            
            interval = (timestamps[i] - timestamps[i-1]).total_seconds()
            intervals.append(interval)
        
        # Calculate variance in intervals (human behavior is variable)
        if intervals:
            variance = np.var(intervals)
            normalized_variance = min(1.0, variance / 3600)  # Normalize by hour
            return normalized_variance
        
        return 0.5
    
    @staticmethod
    def exponential_regression(total_activity: int, user_score: float) -> float:
        """Apply exponential regression to prevent gaming."""
        # Finova Network's anti-whale mechanism
        regression_factor = np.exp(-0.001 * total_activity * (1 - user_score))
        return max(0.1, min(1.0, regression_factor))  # Clamp between 0.1 and 1.0
    
    @staticmethod
    def weighted_average(values: List[float], weights: List[float]) -> float:
        """Calculate weighted average with validation."""
        if len(values) != len(weights) or not values:
            return 0.0
        
        total_weight = sum(weights)
        if total_weight == 0:
            return 0.0
        
        return sum(v * w for v, w in zip(values, weights)) / total_weight

# =============================================================================
# Logging Utilities
# =============================================================================

def setup_logger(name: str, level: str = 'INFO') -> logging.Logger:
    """Set up structured logger for bot detection service."""
    logger = logging.getLogger(name)
    logger.setLevel(getattr(logging, level.upper()))
    
    if not logger.handlers:
        handler = logging.StreamHandler()
        formatter = logging.Formatter(
            '%(asctime)s - %(name)s - %(levelname)s - %(message)s'
        )
        handler.setFormatter(formatter)
        logger.addHandler(handler)
    
    return logger

def log_detection_event(logger: logging.Logger, user_id: str, 
                       detection_result: Dict[str, Any]) -> None:
    """Log bot detection event with structured data."""
    log_data = {
        'event_type': 'bot_detection',
        'user_id_hash': SecurityUtils.hash_user_id(user_id),
        'human_probability': detection_result.get('human_probability', 0),
        'classification': detection_result.get('classification', 'unknown'),
        'timestamp': datetime.utcnow().isoformat(),
        'features': detection_result.get('feature_scores', {})
    }
    
    logger.info(f"Detection event: {json.dumps(log_data)}")

def audit_log(action: str, user_id: str, details: Dict[str, Any]) -> None:
    """Create audit log entry for compliance."""
    audit_logger = logging.getLogger('audit')
    
    audit_entry = {
        'action': action,
        'user_id_hash': SecurityUtils.hash_user_id(user_id),
        'timestamp': datetime.utcnow().isoformat(),
        'details': details,
        'service': 'bot_detection'
    }
    
    audit_logger.info(json.dumps(audit_entry))

# =============================================================================
# Constants and Configuration
# =============================================================================

# Detection thresholds used across the system
DETECTION_THRESHOLDS = {
    'human': 0.7,
    'suspicious': 0.5,
    'bot': 0.3,
    'high_confidence': 0.85,
    'low_confidence': 0.15
}

# Model configuration parameters
MODEL_CONFIGS = {
    'temporal_analyzer': {
        'window_sizes': [60, 3600, 86400],  # 1 min, 1 hour, 1 day
        'min_samples': 10,
        'max_samples': 1000
    },
    'behavioral_analyzer': {
        'feature_count': 15,
        'normalization_method': 'min_max',
        'outlier_threshold': 2.0
    },
    'network_analyzer': {
        'max_depth': 3,
        'min_connections': 2,
        'trust_propagation': 0.8
    }
}

# Feature importance weights
FEATURE_WEIGHTS = {
    'temporal_consistency': 0.25,
    'behavioral_patterns': 0.30,
    'network_authenticity': 0.20,
    'device_consistency': 0.15,
    'content_quality': 0.10
}

# =============================================================================
# Utility Functions
# =============================================================================

def load_config(config_path: Optional[str] = None) -> Config:
    """Load and return configuration instance."""
    return Config(config_path)

def get_env_config(key: str, default: Any = None) -> Any:
    """Get environment configuration with type conversion."""
    value = os.getenv(key, default)
    
    # Try to convert to appropriate type
    if isinstance(default, bool):
        return str(value).lower() in ('true', '1', 'yes', 'on')
    elif isinstance(default, int):
        try:
            return int(value)
        except (ValueError, TypeError):
            return default
    elif isinstance(default, float):
        try:
            return float(value)
        except (ValueError, TypeError):
            return default
    
    return value

def encrypt_data(data: str) -> str:
    """Convenience function for data encryption."""
    return SecurityUtils().encrypt_data(data)

def decrypt_data(encrypted_data: str) -> str:
    """Convenience function for data decryption."""
    return SecurityUtils().decrypt_data(encrypted_data)

def generate_token(payload: Dict[str, Any]) -> str:
    """Convenience function for token generation."""
    return SecurityUtils().generate_token(payload)

def verify_token(token: str) -> Optional[Dict[str, Any]]:
    """Convenience function for token verification."""
    return SecurityUtils().verify_token(token)

def normalize_features(features: Dict[str, float]) -> Dict[str, float]:
    """Convenience function for feature normalization."""
    return DataProcessor.normalize_features(features)

def calculate_scores(features: Dict[str, float], 
                    weights: Dict[str, float]) -> float:
    """Convenience function for score calculation."""
    return DataProcessor.calculate_scores(features, weights)

def aggregate_metrics(metrics_list: List[Dict[str, float]]) -> Dict[str, float]:
    """Convenience function for metrics aggregation."""
    return DataProcessor.aggregate_metrics(metrics_list)

def get_cached_result(cache_key: str, redis_config: Dict[str, Any]) -> Optional[Dict[str, Any]]:
    """Convenience function for cache retrieval."""
    cache_manager = CacheManager(redis_config)
    return cache_manager.get_cached_result(cache_key)

def set_cache_result(cache_key: str, data: Dict[str, Any], 
                    redis_config: Dict[str, Any]) -> bool:
    """Convenience function for cache storage."""
    cache_manager = CacheManager(redis_config)
    return cache_manager.set_cache_result(cache_key, data)

def invalidate_cache(pattern: str, redis_config: Dict[str, Any]) -> int:
    """Convenience function for cache invalidation."""
    cache_manager = CacheManager(redis_config)
    return cache_manager.invalidate_cache(pattern)

def validate_user_data(user_data: Dict[str, Any]) -> Tuple[bool, List[str]]:
    """Convenience function for user data validation."""
    return ValidationUtils.validate_user_data(user_data)

def sanitize_input(data: Any) -> Any:
    """Convenience function for input sanitization."""
    return ValidationUtils.sanitize_input(data)

def check_rate_limits(user_id: str, redis_client: Redis) -> Tuple[bool, int]:
    """Convenience function for rate limit checking."""
    return ValidationUtils.check_rate_limits(user_id, redis_client)

def calculate_entropy(values: List[float]) -> float:
    """Convenience function for entropy calculation."""
    return MathUtils.calculate_entropy(values)

def compute_behavioral_score(activities: List[Dict[str, Any]]) -> float:
    """Convenience function for behavioral score computation."""
    return MathUtils.compute_behavioral_score(activities)

def exponential_regression(total_activity: int, user_score: float) -> float:
    """Convenience function for exponential regression."""
    return MathUtils.exponential_regression(total_activity, user_score)

# =============================================================================
# Module Initialization
# =============================================================================

# Initialize default logger
_default_logger = setup_logger(__name__)

# Load default configuration
try:
    _default_config = load_config()
    _default_logger.info(f"Bot detection utils initialized successfully (v{__version__})")
except Exception as e:
    _default_logger.error(f"Failed to initialize bot detection utils: {e}")
    _default_config = None

# Export default instances for convenience
default_config = _default_config
default_logger = _default_logger
