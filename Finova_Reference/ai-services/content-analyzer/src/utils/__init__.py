"""
Finova Network - AI Content Analyzer Utilities Package
======================================================

Enterprise-grade utilities for content quality assessment, bot detection,
and engagement prediction in the Finova Network ecosystem.

This module provides core utility functions for:
- Content quality scoring (0.5x - 2.0x multiplier)
- Platform-specific analysis
- Anti-bot detection mechanisms
- XP calculation support
- Mining rate adjustments

Version: 3.0
Author: Finova Network Team
License: MIT
"""

import os
import sys
import json
import hashlib
import logging
import asyncio
import time
import re
from typing import Dict, List, Any, Optional, Union, Tuple
from datetime import datetime, timedelta
from functools import wraps, lru_cache
from dataclasses import dataclass, asdict
from enum import Enum
import unicodedata

# Third-party imports
import numpy as np
import pandas as pd
from PIL import Image
import cv2
import requests
from cryptography.fernet import Fernet
import jwt
import redis
from sqlalchemy import create_engine
from pymongo import MongoClient

# Logging configuration
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(name)s - %(levelname)s - %(message)s',
    handlers=[
        logging.FileHandler('/var/log/finova/content_analyzer.log'),
        logging.StreamHandler(sys.stdout)
    ]
)

logger = logging.getLogger(__name__)

# Constants and Configuration
class Config:
    """Configuration constants for the content analyzer"""
    
    # Quality score ranges
    MIN_QUALITY_SCORE = 0.5
    MAX_QUALITY_SCORE = 2.0
    
    # Platform multipliers
    PLATFORM_MULTIPLIERS = {
        'tiktok': 1.3,
        'instagram': 1.2,
        'youtube': 1.4,
        'facebook': 1.1,
        'twitter': 1.2,
        'x': 1.2,
        'finova': 1.0
    }
    
    # XP base values
    XP_BASE_VALUES = {
        'text_post': 50,
        'image_post': 75,
        'video_post': 150,
        'story': 25,
        'comment': 25,
        'like': 5,
        'share': 15,
        'follow': 20
    }
    
    # Mining phase configurations
    MINING_PHASES = {
        1: {'users': 100000, 'rate': 0.1, 'finizen_bonus': 2.0},
        2: {'users': 1000000, 'rate': 0.05, 'finizen_bonus': 1.5},
        3: {'users': 10000000, 'rate': 0.025, 'finizen_bonus': 1.2},
        4: {'users': float('inf'), 'rate': 0.01, 'finizen_bonus': 1.0}
    }
    
    # Cache settings
    CACHE_TTL = 3600  # 1 hour
    REDIS_URL = os.getenv('REDIS_URL', 'redis://localhost:6379/0')
    
    # Security settings
    JWT_SECRET = os.getenv('JWT_SECRET', 'finova-super-secret-key')
    ENCRYPTION_KEY = os.getenv('ENCRYPTION_KEY', Fernet.generate_key())

class ContentType(Enum):
    """Enumeration for content types"""
    TEXT = "text"
    IMAGE = "image"
    VIDEO = "video"
    STORY = "story"
    COMMENT = "comment"
    MIXED = "mixed"

class Platform(Enum):
    """Enumeration for social media platforms"""
    TIKTOK = "tiktok"
    INSTAGRAM = "instagram" 
    YOUTUBE = "youtube"
    FACEBOOK = "facebook"
    TWITTER = "twitter"
    X = "x"
    FINOVA = "finova"

@dataclass
class ContentAnalysisResult:
    """Data class for content analysis results"""
    content_id: str
    user_id: str
    platform: str
    content_type: str
    quality_score: float
    originality_score: float
    engagement_potential: float
    brand_safety_score: float
    human_probability: float
    xp_multiplier: float
    mining_boost: float
    timestamp: datetime
    metadata: Dict[str, Any]

@dataclass 
class UserMetrics:
    """Data class for user metrics and statistics"""
    user_id: str
    xp_level: int
    total_xp: int
    rp_tier: str
    total_rp: int
    mining_rate: float
    staking_amount: float
    referral_count: int
    activity_streak: int
    quality_average: float
    last_activity: datetime

# Utility Functions
class SecurityUtils:
    """Security and encryption utilities"""
    
    @staticmethod
    def encrypt_sensitive_data(data: str) -> str:
        """Encrypt sensitive user data"""
        try:
            f = Fernet(Config.ENCRYPTION_KEY)
            return f.encrypt(data.encode()).decode()
        except Exception as e:
            logger.error(f"Encryption failed: {e}")
            raise

    @staticmethod
    def decrypt_sensitive_data(encrypted_data: str) -> str:
        """Decrypt sensitive user data"""
        try:
            f = Fernet(Config.ENCRYPTION_KEY)
            return f.decrypt(encrypted_data.encode()).decode()
        except Exception as e:
            logger.error(f"Decryption failed: {e}")
            raise

    @staticmethod
    def generate_content_hash(content: str) -> str:
        """Generate SHA-256 hash for content uniqueness"""
        return hashlib.sha256(content.encode('utf-8')).hexdigest()

    @staticmethod
    def validate_jwt_token(token: str) -> Dict[str, Any]:
        """Validate and decode JWT token"""
        try:
            payload = jwt.decode(token, Config.JWT_SECRET, algorithms=['HS256'])
            return payload
        except jwt.InvalidTokenError as e:
            logger.warning(f"Invalid JWT token: {e}")
            return {}

class CacheUtils:
    """Redis caching utilities"""
    
    def __init__(self):
        self.redis_client = redis.from_url(Config.REDIS_URL)
    
    def get_cached_result(self, key: str) -> Optional[Dict[str, Any]]:
        """Get cached analysis result"""
        try:
            cached = self.redis_client.get(key)
            if cached:
                return json.loads(cached)
            return None
        except Exception as e:
            logger.error(f"Cache retrieval failed: {e}")
            return None
    
    def cache_result(self, key: str, data: Dict[str, Any], ttl: int = Config.CACHE_TTL):
        """Cache analysis result"""
        try:
            self.redis_client.setex(key, ttl, json.dumps(data, default=str))
        except Exception as e:
            logger.error(f"Cache storage failed: {e}")

class TextAnalysisUtils:
    """Text content analysis utilities"""
    
    @staticmethod
    def clean_text(text: str) -> str:
        """Clean and normalize text content"""
        if not text:
            return ""
        
        # Remove excessive whitespace
        text = re.sub(r'\s+', ' ', text.strip())
        
        # Normalize unicode
        text = unicodedata.normalize('NFKC', text)
        
        # Remove excessive punctuation
        text = re.sub(r'[!]{2,}', '!', text)
        text = re.sub(r'[?]{2,}', '?', text)
        
        return text
    
    @staticmethod
    def extract_features(text: str) -> Dict[str, float]:
        """Extract text features for quality assessment"""
        if not text:
            return {'length': 0, 'word_count': 0, 'unique_ratio': 0}
        
        cleaned_text = TextAnalysisUtils.clean_text(text)
        words = cleaned_text.split()
        
        features = {
            'length': len(cleaned_text),
            'word_count': len(words),
            'unique_ratio': len(set(words)) / max(len(words), 1),
            'avg_word_length': np.mean([len(word) for word in words]) if words else 0,
            'sentence_count': len(re.split(r'[.!?]+', cleaned_text)),
            'exclamation_ratio': cleaned_text.count('!') / max(len(cleaned_text), 1),
            'question_ratio': cleaned_text.count('?') / max(len(cleaned_text), 1),
            'caps_ratio': sum(1 for c in cleaned_text if c.isupper()) / max(len(cleaned_text), 1)
        }
        
        return features

class QualityScorer:
    """Content quality scoring engine"""
    
    def __init__(self):
        self.cache = CacheUtils()
    
    def calculate_quality_score(self, content: str, content_type: ContentType, 
                              platform: Platform, user_metrics: UserMetrics) -> float:
        """Calculate comprehensive quality score (0.5 - 2.0 range)"""
        
        # Check cache first
        cache_key = f"quality:{SecurityUtils.generate_content_hash(content)}"
        cached_result = self.cache.get_cached_result(cache_key)
        if cached_result:
            return cached_result['quality_score']
        
        try:
            # Base quality components
            originality_score = self._calculate_originality(content)
            engagement_score = self._predict_engagement(content, content_type, platform)
            brand_safety_score = self._check_brand_safety(content)
            platform_relevance = self._check_platform_relevance(content, platform)
            user_reputation = self._calculate_user_reputation(user_metrics)
            
            # Weighted quality calculation
            weights = {
                'originality': 0.25,
                'engagement': 0.3,
                'brand_safety': 0.2,
                'platform_relevance': 0.15,
                'user_reputation': 0.1
            }
            
            weighted_score = (
                originality_score * weights['originality'] +
                engagement_score * weights['engagement'] +
                brand_safety_score * weights['brand_safety'] +
                platform_relevance * weights['platform_relevance'] +
                user_reputation * weights['user_reputation']
            )
            
            # Clamp to valid range
            final_score = max(Config.MIN_QUALITY_SCORE, 
                            min(Config.MAX_QUALITY_SCORE, weighted_score))
            
            # Cache result
            result = {'quality_score': final_score, 'timestamp': time.time()}
            self.cache.cache_result(cache_key, result)
            
            return final_score
            
        except Exception as e:
            logger.error(f"Quality scoring failed: {e}")
            return 1.0  # Default neutral score
    
    def _calculate_originality(self, content: str) -> float:
        """Calculate content originality (0.0 - 2.0)"""
        if not content:
            return 0.5
        
        features = TextAnalysisUtils.extract_features(content)
        
        # Originality factors
        uniqueness = features['unique_ratio']
        complexity = min(features['avg_word_length'] / 6.0, 1.0)
        length_factor = min(features['word_count'] / 50.0, 1.0)
        
        originality = (uniqueness * 0.4 + complexity * 0.3 + length_factor * 0.3) * 2.0
        
        return max(0.5, min(2.0, originality))
    
    def _predict_engagement(self, content: str, content_type: ContentType, 
                          platform: Platform) -> float:
        """Predict engagement potential (0.0 - 2.0)"""
        if not content:
            return 0.5
        
        features = TextAnalysisUtils.extract_features(content)
        
        # Engagement prediction factors
        optimal_length = self._get_optimal_length(platform, content_type)
        length_score = 1.0 - abs(features['word_count'] - optimal_length) / optimal_length
        
        # Question and exclamation engagement boost
        interaction_boost = min((features['question_ratio'] + features['exclamation_ratio']) * 2, 0.5)
        
        # Platform-specific factors
        platform_factor = Config.PLATFORM_MULTIPLIERS.get(platform.value, 1.0)
        
        engagement = (length_score + interaction_boost) * platform_factor
        
        return max(0.5, min(2.0, engagement))
    
    def _check_brand_safety(self, content: str) -> float:
        """Check brand safety score (0.0 - 2.0)"""
        if not content:
            return 1.0
        
        # Simplified brand safety keywords
        unsafe_keywords = [
            'hate', 'violence', 'explicit', 'illegal', 'scam', 'fraud',
            'discrimination', 'harassment', 'abuse', 'threat'
        ]
        
        content_lower = content.lower()
        unsafe_count = sum(1 for keyword in unsafe_keywords if keyword in content_lower)
        
        # Penalty for unsafe content
        safety_score = max(0.1, 2.0 - (unsafe_count * 0.5))
        
        return safety_score
    
    def _check_platform_relevance(self, content: str, platform: Platform) -> float:
        """Check platform relevance (0.5 - 2.0)"""
        # Platform-specific content preferences
        platform_keywords = {
            Platform.TIKTOK: ['dance', 'music', 'viral', 'trend', 'challenge'],
            Platform.INSTAGRAM: ['photo', 'aesthetic', 'lifestyle', 'fashion', 'art'],
            Platform.YOUTUBE: ['video', 'tutorial', 'review', 'vlog', 'subscribe'],
            Platform.TWITTER: ['breaking', 'news', 'opinion', 'thread', 'discussion'],
            Platform.FACEBOOK: ['family', 'friends', 'community', 'event', 'share']
        }
        
        keywords = platform_keywords.get(platform, [])
        if not keywords:
            return 1.0
        
        content_lower = content.lower()
        relevance_count = sum(1 for keyword in keywords if keyword in content_lower)
        
        relevance_score = 1.0 + (relevance_count * 0.2)
        
        return min(2.0, relevance_score)
    
    def _calculate_user_reputation(self, user_metrics: UserMetrics) -> float:
        """Calculate user reputation factor (0.5 - 2.0)"""
        try:
            # Reputation factors
            level_factor = min(user_metrics.xp_level / 50.0, 1.0)
            quality_factor = user_metrics.quality_average
            streak_factor = min(user_metrics.activity_streak / 30.0, 0.5)
            
            reputation = level_factor + quality_factor + streak_factor
            
            return max(0.5, min(2.0, reputation))
        except:
            return 1.0
    
    def _get_optimal_length(self, platform: Platform, content_type: ContentType) -> int:
        """Get optimal content length for platform"""
        optimal_lengths = {
            (Platform.TWITTER, ContentType.TEXT): 25,
            (Platform.INSTAGRAM, ContentType.TEXT): 40,
            (Platform.TIKTOK, ContentType.TEXT): 15,
            (Platform.FACEBOOK, ContentType.TEXT): 60,
            (Platform.YOUTUBE, ContentType.TEXT): 80
        }
        
        return optimal_lengths.get((platform, content_type), 50)

class MiningCalculator:
    """Mining rate calculation utilities"""
    
    @staticmethod
    def calculate_mining_rate(user_metrics: UserMetrics, total_users: int, 
                            quality_multiplier: float) -> float:
        """Calculate final mining rate with all bonuses"""
        try:
            # Determine mining phase
            phase = MiningCalculator._get_mining_phase(total_users)
            base_rate = Config.MINING_PHASES[phase]['rate']
            finizen_bonus = Config.MINING_PHASES[phase]['finizen_bonus']
            
            # Calculate bonuses
            referral_bonus = 1 + (user_metrics.referral_count * 0.1)
            security_bonus = 1.2  # Assuming KYC verified
            xp_bonus = 1 + (user_metrics.xp_level / 100.0)
            staking_bonus = 1 + (user_metrics.staking_amount / 10000.0)
            
            # Regression factor (anti-whale mechanism)
            regression_factor = np.exp(-0.001 * user_metrics.staking_amount)
            
            final_rate = (base_rate * finizen_bonus * referral_bonus * 
                         security_bonus * xp_bonus * staking_bonus * 
                         quality_multiplier * regression_factor)
            
            return max(0.001, final_rate)  # Minimum rate
            
        except Exception as e:
            logger.error(f"Mining calculation failed: {e}")
            return 0.01  # Default rate
    
    @staticmethod
    def _get_mining_phase(total_users: int) -> int:
        """Determine current mining phase"""
        for phase, config in Config.MINING_PHASES.items():
            if total_users <= config['users']:
                return phase
        return 4  # Final phase

class XPCalculator:
    """XP calculation utilities"""
    
    @staticmethod
    def calculate_xp_gain(activity_type: str, platform: Platform, 
                         quality_score: float, user_metrics: UserMetrics) -> int:
        """Calculate XP gain for activity"""
        try:
            # Base XP
            base_xp = Config.XP_BASE_VALUES.get(activity_type, 10)
            
            # Platform multiplier
            platform_multiplier = Config.PLATFORM_MULTIPLIERS.get(platform.value, 1.0)
            
            # Streak bonus
            streak_bonus = min(1 + (user_metrics.activity_streak * 0.05), 3.0)
            
            # Level progression (exponential decay)
            level_progression = np.exp(-0.01 * user_metrics.xp_level)
            
            final_xp = int(base_xp * platform_multiplier * quality_score * 
                          streak_bonus * level_progression)
            
            return max(1, final_xp)  # Minimum 1 XP
            
        except Exception as e:
            logger.error(f"XP calculation failed: {e}")
            return 1

# Performance monitoring decorator
def monitor_performance(func):
    """Decorator for monitoring function performance"""
    @wraps(func)
    def wrapper(*args, **kwargs):
        start_time = time.time()
        try:
            result = func(*args, **kwargs)
            execution_time = time.time() - start_time
            logger.info(f"{func.__name__} executed in {execution_time:.4f}s")
            return result
        except Exception as e:
            execution_time = time.time() - start_time
            logger.error(f"{func.__name__} failed after {execution_time:.4f}s: {e}")
            raise
    return wrapper

# Rate limiting decorator
def rate_limit(max_calls: int, time_window: int):
    """Rate limiting decorator"""
    def decorator(func):
        calls = {}
        
        @wraps(func)
        def wrapper(*args, **kwargs):
            now = time.time()
            func_name = func.__name__
            
            if func_name not in calls:
                calls[func_name] = []
            
            # Clean old calls
            calls[func_name] = [call_time for call_time in calls[func_name] 
                              if now - call_time < time_window]
            
            if len(calls[func_name]) >= max_calls:
                raise Exception(f"Rate limit exceeded for {func_name}")
            
            calls[func_name].append(now)
            return func(*args, **kwargs)
        
        return wrapper
    return decorator

# Main utility classes and functions export
__all__ = [
    'Config',
    'ContentType',
    'Platform', 
    'ContentAnalysisResult',
    'UserMetrics',
    'SecurityUtils',
    'CacheUtils',
    'TextAnalysisUtils',
    'QualityScorer',
    'MiningCalculator',
    'XPCalculator',
    'monitor_performance',
    'rate_limit',
    'logger'
]

# Initialize global instances
cache_utils = CacheUtils()
quality_scorer = QualityScorer()

logger.info("Finova Network Content Analyzer Utils initialized successfully")
