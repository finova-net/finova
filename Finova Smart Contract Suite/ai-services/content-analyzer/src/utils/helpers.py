"""
Finova Network - AI Content Analyzer Helper Utilities
Enterprise-grade helper functions for content analysis, quality scoring, and anti-bot detection.

Author: Finova Network Development Team
Version: 3.0.0
Last Updated: July 2025
"""

import hashlib
import re
import json
import time
import math
import logging
import asyncio
from typing import Dict, List, Optional, Tuple, Any, Union
from datetime import datetime, timedelta
from collections import defaultdict, Counter
from functools import lru_cache, wraps
from dataclasses import dataclass
import numpy as np
import pandas as pd
from PIL import Image, ImageStat
import cv2
import nltk
from nltk.corpus import stopwords
from nltk.tokenize import word_tokenize, sent_tokenize
from nltk.sentiment import SentimentIntensityAnalyzer
import textstat
from transformers import pipeline, AutoTokenizer
import torch
from scipy.spatial.distance import cosine
from sklearn.feature_extraction.text import TfidfVectorizer
from sklearn.metrics.pairwise import cosine_similarity
import requests
from urllib.parse import urlparse
import base64
from cryptography.fernet import Fernet
import redis
from pymongo import MongoClient
import os
from dotenv import load_dotenv

# Load environment variables
load_dotenv()

# Configure logging
logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)

# Redis connection for caching
redis_client = redis.Redis(
    host=os.getenv('REDIS_HOST', 'localhost'),
    port=int(os.getenv('REDIS_PORT', 6379)),
    db=0,
    decode_responses=True
)

# MongoDB connection for data storage
mongo_client = MongoClient(os.getenv('MONGODB_URI', 'mongodb://localhost:27017/'))
db = mongo_client.finova_ai

# Download required NLTK data
try:
    nltk.data.find('tokenizers/punkt')
    nltk.data.find('corpora/stopwords')
    nltk.data.find('vader_lexicon')
except LookupError:
    nltk.download('punkt')
    nltk.download('stopwords')
    nltk.download('vader_lexicon')

@dataclass
class ContentMetrics:
    """Data class for content analysis metrics"""
    originality_score: float
    quality_score: float
    engagement_potential: float
    platform_relevance: float
    brand_safety_score: float
    human_probability: float
    sentiment_score: float
    readability_score: float
    uniqueness_hash: str
    analysis_timestamp: datetime

@dataclass
class UserBehaviorProfile:
    """Data class for user behavior analysis"""
    user_id: str
    activity_pattern: Dict[str, float]
    content_consistency: float
    posting_rhythm: Dict[str, int]
    device_fingerprint: str
    interaction_quality: float
    network_authenticity: float
    human_score: float

class SecurityManager:
    """Security utilities for data protection and validation"""
    
    def __init__(self):
        self.cipher_suite = Fernet(os.getenv('ENCRYPTION_KEY', Fernet.generate_key()))
        self.salt = os.getenv('SECURITY_SALT', 'finova_salt_2025').encode()
    
    def hash_content(self, content: str) -> str:
        """Generate secure hash for content uniqueness"""
        content_bytes = content.encode('utf-8')
        return hashlib.sha256(content_bytes + self.salt).hexdigest()
    
    def encrypt_sensitive_data(self, data: str) -> str:
        """Encrypt sensitive user data"""
        return self.cipher_suite.encrypt(data.encode()).decode()
    
    def decrypt_sensitive_data(self, encrypted_data: str) -> str:
        """Decrypt sensitive user data"""
        return self.cipher_suite.decrypt(encrypted_data.encode()).decode()
    
    @staticmethod
    def sanitize_input(text: str) -> str:
        """Sanitize input text to prevent injection attacks"""
        # Remove potential SQL injection patterns
        sql_patterns = [
            r"(\b(SELECT|INSERT|UPDATE|DELETE|DROP|CREATE|ALTER)\b)",
            r"(--|\#|\/\*|\*\/)",
            r"(\bUNION\b.*\bSELECT\b)"
        ]
        
        for pattern in sql_patterns:
            text = re.sub(pattern, '', text, flags=re.IGNORECASE)
        
        # Remove script tags and potential XSS
        text = re.sub(r'<script\b[^<]*(?:(?!<\/script>)<[^<]*)*<\/script>', '', text, flags=re.IGNORECASE)
        text = re.sub(r'javascript:', '', text, flags=re.IGNORECASE)
        
        return text.strip()

class CacheManager:
    """Redis-based caching for performance optimization"""
    
    def __init__(self):
        self.default_ttl = 3600  # 1 hour
        self.long_ttl = 86400    # 24 hours
    
    def get(self, key: str) -> Optional[Any]:
        """Get cached data"""
        try:
            data = redis_client.get(f"finova:ai:{key}")
            return json.loads(data) if data else None
        except Exception as e:
            logger.error(f"Cache get error: {e}")
            return None
    
    def set(self, key: str, value: Any, ttl: int = None) -> bool:
        """Set cached data with TTL"""
        try:
            ttl = ttl or self.default_ttl
            redis_client.setex(f"finova:ai:{key}", ttl, json.dumps(value))
            return True
        except Exception as e:
            logger.error(f"Cache set error: {e}")
            return False
    
    def cache_decorator(self, ttl: int = None):
        """Decorator for caching function results"""
        def decorator(func):
            @wraps(func)
            def wrapper(*args, **kwargs):
                # Create cache key from function name and arguments
                key_data = f"{func.__name__}:{str(args)}:{str(sorted(kwargs.items()))}"
                cache_key = hashlib.md5(key_data.encode()).hexdigest()
                
                # Try to get from cache
                cached_result = self.get(cache_key)
                if cached_result is not None:
                    return cached_result
                
                # Execute function and cache result
                result = func(*args, **kwargs)
                self.set(cache_key, result, ttl or self.default_ttl)
                return result
            return wrapper
        return decorator

class TextAnalyzer:
    """Advanced text analysis utilities"""
    
    def __init__(self):
        self.sentiment_analyzer = SentimentIntensityAnalyzer()
        self.stop_words = set(stopwords.words('english'))
        self.tfidf_vectorizer = TfidfVectorizer(max_features=5000, stop_words='english')
        
        # Load AI detection model (placeholder for actual model)
        self.ai_detector = None
        try:
            # self.ai_detector = pipeline("text-classification", model="ai-content-detector")
            pass
        except Exception as e:
            logger.warning(f"AI detector model not loaded: {e}")
    
    @lru_cache(maxsize=1000)
    def analyze_readability(self, text: str) -> Dict[str, float]:
        """Analyze text readability using multiple metrics"""
        if not text.strip():
            return {
                'flesch_kincaid': 0.0,
                'flesch_reading_ease': 0.0,
                'gunning_fog': 0.0,
                'automated_readability': 0.0,
                'avg_score': 0.0
            }
        
        try:
            scores = {
                'flesch_kincaid': textstat.flesch_kincaid_grade(text),
                'flesch_reading_ease': textstat.flesch_reading_ease(text),
                'gunning_fog': textstat.gunning_fog(text),
                'automated_readability': textstat.automated_readability_index(text)
            }
            
            # Normalize flesch_reading_ease (0-100) to 0-20 scale
            scores['flesch_reading_ease'] = scores['flesch_reading_ease'] / 5
            
            # Calculate average
            scores['avg_score'] = sum(scores.values()) / len(scores)
            
            return scores
        except Exception as e:
            logger.error(f"Readability analysis error: {e}")
            return {'flesch_kincaid': 10.0, 'flesch_reading_ease': 12.0, 
                   'gunning_fog': 12.0, 'automated_readability': 10.0, 'avg_score': 11.0}
    
    def analyze_sentiment(self, text: str) -> Dict[str, float]:
        """Analyze sentiment with detailed scores"""
        if not text.strip():
            return {'compound': 0.0, 'positive': 0.0, 'neutral': 1.0, 'negative': 0.0}
        
        try:
            scores = self.sentiment_analyzer.polarity_scores(text)
            return scores
        except Exception as e:
            logger.error(f"Sentiment analysis error: {e}")
            return {'compound': 0.0, 'positive': 0.0, 'neutral': 1.0, 'negative': 0.0}
    
    def extract_keywords(self, text: str, max_keywords: int = 10) -> List[Tuple[str, float]]:
        """Extract keywords with TF-IDF scoring"""
        if not text.strip():
            return []
        
        try:
            # Tokenize and clean
            words = word_tokenize(text.lower())
            words = [word for word in words if word.isalpha() and word not in self.stop_words]
            
            if not words:
                return []
            
            # Calculate TF-IDF-like scores manually for single document
            word_freq = Counter(words)
            total_words = len(words)
            
            # Calculate TF scores
            tf_scores = {word: freq/total_words for word, freq in word_freq.items()}
            
            # Sort by frequency and return top keywords
            keywords = sorted(tf_scores.items(), key=lambda x: x[1], reverse=True)
            return keywords[:max_keywords]
        
        except Exception as e:
            logger.error(f"Keyword extraction error: {e}")
            return []
    
    def detect_ai_generated(self, text: str) -> float:
        """Detect if content is AI-generated (0.0 = human, 1.0 = AI)"""
        if not text.strip():
            return 0.5
        
        # Heuristic-based detection (replace with actual AI model)
        try:
            # Check for AI patterns
            ai_indicators = 0.0
            
            # Very uniform sentence lengths
            sentences = sent_tokenize(text)
            if len(sentences) > 3:
                lengths = [len(s.split()) for s in sentences]
                length_variance = np.var(lengths) if len(lengths) > 1 else 100
                if length_variance < 5:  # Very uniform
                    ai_indicators += 0.2
            
            # Repetitive patterns
            words = text.lower().split()
            if len(words) > 10:
                unique_ratio = len(set(words)) / len(words)
                if unique_ratio < 0.5:  # High repetition
                    ai_indicators += 0.3
            
            # Perfect grammar indicators (simplified)
            punctuation_ratio = sum(1 for c in text if c in '.,!?') / len(text)
            if 0.02 < punctuation_ratio < 0.08:  # Typical AI range
                ai_indicators += 0.1
            
            # Lack of personal pronouns
            personal_pronouns = ['i', 'me', 'my', 'mine', 'myself']
            personal_count = sum(1 for word in words if word in personal_pronouns)
            if personal_count == 0 and len(words) > 20:
                ai_indicators += 0.2
            
            return min(ai_indicators, 1.0)
            
        except Exception as e:
            logger.error(f"AI detection error: {e}")
            return 0.5
    
    def calculate_originality(self, text: str, user_id: str) -> float:
        """Calculate content originality against user's history and global database"""
        if not text.strip():
            return 0.0
        
        try:
            content_hash = SecurityManager().hash_content(text)
            
            # Check against user's previous content
            user_content = db.user_content.find({'user_id': user_id}, {'content_hash': 1})
            user_hashes = [doc['content_hash'] for doc in user_content]
            
            if content_hash in user_hashes:
                return 0.0  # Exact duplicate
            
            # Check similarity with recent content
            recent_content = db.user_content.find(
                {'user_id': user_id, 'created_at': {'$gte': datetime.now() - timedelta(days=30)}},
                {'text': 1}
            ).limit(10)
            
            similarities = []
            for doc in recent_content:
                if 'text' in doc:
                    similarity = self.calculate_text_similarity(text, doc['text'])
                    similarities.append(similarity)
            
            if similarities:
                max_similarity = max(similarities)
                originality = 1.0 - max_similarity
            else:
                originality = 1.0
            
            # Store content for future comparison
            db.user_content.insert_one({
                'user_id': user_id,
                'content_hash': content_hash,
                'text': text[:500],  # Store first 500 chars for similarity check
                'created_at': datetime.now()
            })
            
            return max(0.0, min(1.0, originality))
            
        except Exception as e:
            logger.error(f"Originality calculation error: {e}")
            return 0.5

    @staticmethod
    def calculate_text_similarity(text1: str, text2: str) -> float:
        """Calculate similarity between two texts using cosine similarity"""
        try:
            vectorizer = TfidfVectorizer(stop_words='english')
            tfidf_matrix = vectorizer.fit_transform([text1, text2])
            similarity_matrix = cosine_similarity(tfidf_matrix)
            return similarity_matrix[0][1]
        except Exception:
            return 0.0

class ImageAnalyzer:
    """Advanced image analysis utilities"""
    
    @staticmethod
    def analyze_image_quality(image_path: str) -> Dict[str, float]:
        """Analyze image quality metrics"""
        try:
            img = Image.open(image_path)
            
            # Convert to RGB if necessary
            if img.mode != 'RGB':
                img = img.convert('RGB')
            
            # Get image statistics
            stat = ImageStat.Stat(img)
            
            # Calculate quality metrics
            metrics = {
                'brightness': sum(stat.mean) / (3 * 255),  # Normalize to 0-1
                'contrast': sum(stat.stddev) / (3 * 255),  # Normalize to 0-1
                'sharpness': ImageAnalyzer._calculate_sharpness(image_path),
                'noise_level': ImageAnalyzer._calculate_noise(image_path),
                'composition_score': ImageAnalyzer._analyze_composition(img),
                'resolution_score': ImageAnalyzer._calculate_resolution_score(img.size)
            }
            
            # Calculate overall quality
            weights = {
                'brightness': 0.15,
                'contrast': 0.20,
                'sharpness': 0.25,
                'noise_level': 0.15,
                'composition_score': 0.15,
                'resolution_score': 0.10
            }
            
            overall_quality = sum(metrics[key] * weights[key] for key in weights.keys())
            metrics['overall_quality'] = min(1.0, max(0.0, overall_quality))
            
            return metrics
            
        except Exception as e:
            logger.error(f"Image quality analysis error: {e}")
            return {
                'brightness': 0.5, 'contrast': 0.5, 'sharpness': 0.5,
                'noise_level': 0.5, 'composition_score': 0.5,
                'resolution_score': 0.5, 'overall_quality': 0.5
            }
    
    @staticmethod
    def _calculate_sharpness(image_path: str) -> float:
        """Calculate image sharpness using Laplacian variance"""
        try:
            image = cv2.imread(image_path, cv2.IMREAD_GRAYSCALE)
            if image is None:
                return 0.5
            
            laplacian_var = cv2.Laplacian(image, cv2.CV_64F).var()
            # Normalize to 0-1 range (typical values 0-2000)
            return min(1.0, laplacian_var / 1000)
        except Exception:
            return 0.5
    
    @staticmethod
    def _calculate_noise(image_path: str) -> float:
        """Calculate image noise level"""
        try:
            image = cv2.imread(image_path, cv2.IMREAD_GRAYSCALE)
            if image is None:
                return 0.5
            
            # Use standard deviation as noise indicator
            noise_level = np.std(image) / 255.0
            # Invert so higher score means less noise
            return 1.0 - min(1.0, noise_level)
        except Exception:
            return 0.5
    
    @staticmethod
    def _analyze_composition(img: Image.Image) -> float:
        """Analyze image composition using rule of thirds and other principles"""
        try:
            width, height = img.size
            
            # Convert to numpy array for analysis
            img_array = np.array(img)
            
            # Rule of thirds analysis
            third_w, third_h = width // 3, height // 3
            
            # Check for interesting points along rule of thirds lines
            thirds_score = 0.0
            
            # Sample pixels along rule of thirds lines
            vertical_lines = [third_w, 2 * third_w]
            horizontal_lines = [third_h, 2 * third_h]
            
            # Simplified composition analysis
            # In a real implementation, you'd use more sophisticated algorithms
            
            # Check brightness variation along thirds lines
            for x in vertical_lines:
                if x < width:
                    line_var = np.var(img_array[:, x, :])
                    thirds_score += min(1.0, line_var / 10000)
            
            for y in horizontal_lines:
                if y < height:
                    line_var = np.var(img_array[y, :, :])
                    thirds_score += min(1.0, line_var / 10000)
            
            return min(1.0, thirds_score / 4)
            
        except Exception:
            return 0.5
    
    @staticmethod
    def _calculate_resolution_score(size: Tuple[int, int]) -> float:
        """Calculate resolution quality score"""
        width, height = size
        total_pixels = width * height
        
        # Score based on resolution tiers
        if total_pixels >= 8000000:  # 4K+ (8MP+)
            return 1.0
        elif total_pixels >= 2000000:  # HD+ (2MP+)
            return 0.8
        elif total_pixels >= 1000000:  # Standard (1MP+)
            return 0.6
        elif total_pixels >= 500000:   # Low (0.5MP+)
            return 0.4
        else:
            return 0.2

class VideoAnalyzer:
    """Video content analysis utilities"""
    
    @staticmethod
    def analyze_video_quality(video_path: str) -> Dict[str, float]:
        """Analyze video quality and engagement metrics"""
        try:
            cap = cv2.VideoCapture(video_path)
            
            if not cap.isOpened():
                return VideoAnalyzer._default_video_metrics()
            
            # Get video properties
            fps = cap.get(cv2.CAP_PROP_FPS)
            frame_count = int(cap.get(cv2.CAP_PROP_FRAME_COUNT))
            duration = frame_count / fps if fps > 0 else 0
            width = int(cap.get(cv2.CAP_PROP_FRAME_WIDTH))
            height = int(cap.get(cv2.CAP_PROP_FRAME_HEIGHT))
            
            # Sample frames for analysis
            sample_frames = []
            sample_count = min(10, max(1, int(frame_count / 10)))
            
            for i in range(sample_count):
                frame_pos = int(i * frame_count / sample_count)
                cap.set(cv2.CAP_PROP_POS_FRAMES, frame_pos)
                ret, frame = cap.read()
                if ret:
                    sample_frames.append(frame)
            
            cap.release()
            
            if not sample_frames:
                return VideoAnalyzer._default_video_metrics()
            
            # Analyze quality metrics
            metrics = {
                'resolution_score': VideoAnalyzer._calculate_video_resolution_score(width, height),
                'duration_score': VideoAnalyzer._calculate_duration_score(duration),
                'fps_score': VideoAnalyzer._calculate_fps_score(fps),
                'stability_score': VideoAnalyzer._analyze_stability(sample_frames),
                'brightness_consistency': VideoAnalyzer._analyze_brightness_consistency(sample_frames),
                'motion_analysis': VideoAnalyzer._analyze_motion(sample_frames)
            }
            
            # Calculate overall score
            weights = {
                'resolution_score': 0.2,
                'duration_score': 0.15,
                'fps_score': 0.15,
                'stability_score': 0.2,
                'brightness_consistency': 0.15,
                'motion_analysis': 0.15
            }
            
            overall_score = sum(metrics[key] * weights[key] for key in weights.keys())
            metrics['overall_quality'] = min(1.0, max(0.0, overall_score))
            
            return metrics
            
        except Exception as e:
            logger.error(f"Video quality analysis error: {e}")
            return VideoAnalyzer._default_video_metrics()
    
    @staticmethod
    def _default_video_metrics() -> Dict[str, float]:
        """Return default video metrics when analysis fails"""
        return {
            'resolution_score': 0.5,
            'duration_score': 0.5,
            'fps_score': 0.5,
            'stability_score': 0.5,
            'brightness_consistency': 0.5,
            'motion_analysis': 0.5,
            'overall_quality': 0.5
        }
    
    @staticmethod
    def _calculate_video_resolution_score(width: int, height: int) -> float:
        """Calculate video resolution quality score"""
        total_pixels = width * height
        
        if total_pixels >= 8294400:  # 4K (3840x2160)
            return 1.0
        elif total_pixels >= 2073600:  # 1080p (1920x1080)
            return 0.9
        elif total_pixels >= 921600:   # 720p (1280x720)
            return 0.7
        elif total_pixels >= 307200:   # 480p (640x480)
            return 0.5
        else:
            return 0.3
    
    @staticmethod
    def _calculate_duration_score(duration: float) -> float:
        """Calculate score based on video duration (engagement optimization)"""
        if 15 <= duration <= 60:  # Optimal for social media
            return 1.0
        elif 5 <= duration <= 120:  # Good range
            return 0.8
        elif duration <= 300:  # Acceptable
            return 0.6
        else:  # Too long or too short
            return 0.3
    
    @staticmethod
    def _calculate_fps_score(fps: float) -> float:
        """Calculate score based on frame rate"""
        if fps >= 60:
            return 1.0
        elif fps >= 30:
            return 0.9
        elif fps >= 24:
            return 0.7
        elif fps >= 15:
            return 0.5
        else:
            return 0.3
    
    @staticmethod
    def _analyze_stability(frames: List[np.ndarray]) -> float:
        """Analyze video stability (camera shake detection)"""
        if len(frames) < 2:
            return 1.0
        
        try:
            stability_scores = []
            
            for i in range(1, len(frames)):
                # Convert to grayscale
                gray1 = cv2.cvtColor(frames[i-1], cv2.COLOR_BGR2GRAY)
                gray2 = cv2.cvtColor(frames[i], cv2.COLOR_BGR2GRAY)
                
                # Calculate optical flow
                flow = cv2.calcOpticalFlowPyrLK(gray1, gray2, None, None)
                
                # Simple stability metric based on flow magnitude
                if flow[0] is not None and len(flow[0]) > 0:
                    motion_magnitude = np.mean(np.sqrt(flow[0][:, 0]**2 + flow[0][:, 1]**2))
                    stability = max(0.0, 1.0 - (motion_magnitude / 50))  # Normalize
                    stability_scores.append(stability)
            
            return np.mean(stability_scores) if stability_scores else 1.0
            
        except Exception:
            return 0.7  # Default to reasonable stability
    
    @staticmethod
    def _analyze_brightness_consistency(frames: List[np.ndarray]) -> float:
        """Analyze brightness consistency across frames"""
        try:
            brightness_values = []
            
            for frame in frames:
                gray = cv2.cvtColor(frame, cv2.COLOR_BGR2GRAY)
                brightness = np.mean(gray)
                brightness_values.append(brightness)
            
            if len(brightness_values) > 1:
                brightness_var = np.var(brightness_values)
                # Lower variance = better consistency
                consistency = max(0.0, 1.0 - (brightness_var / 10000))
                return consistency
            
            return 1.0
            
        except Exception:
            return 0.7

    @staticmethod
    def _analyze_motion(frames: List[np.ndarray]) -> float:
        """Analyze motion content in video"""
        if len(frames) < 2:
            return 0.5
        
        try:
            motion_scores = []
            
            for i in range(1, len(frames)):
                # Frame difference
                diff = cv2.absdiff(frames[i-1], frames[i])
                gray_diff = cv2.cvtColor(diff, cv2.COLOR_BGR2GRAY)
                motion_amount = np.mean(gray_diff)
                
                # Normalize and score (some motion is good, too much is bad)
                motion_score = min(1.0, motion_amount / 50)
                if motion_score > 0.8:  # Too much motion
                    motion_score = 1.0 - motion_score
                
                motion_scores.append(motion_score)
            
            return np.mean(motion_scores) if motion_scores else 0.5
            
        except Exception:
            return 0.5

class PlatformSpecificAnalyzer:
    """Platform-specific content analysis and optimization"""
    
    PLATFORM_CONFIGS = {
        'instagram': {
            'optimal_hashtags': (5, 15),
            'optimal_length': (50, 2200),
            'image_aspect_ratios': [(1, 1), (4, 5), (9, 16)],
            'video_duration': (3, 60),
            'engagement_factors': ['hashtags', 'mentions', 'location', 'story_tags']
        },
        'tiktok': {
            'optimal_hashtags': (3, 8),
            'optimal_length': (10, 100),
            'video_duration': (15, 180),
            'trending_sounds': True,
            'engagement_factors': ['trending_hashtags', 'effects', 'music', 'duets']
        },
        'youtube': {
            'optimal_title_length': (30, 60),
            'optimal_description': (200, 5000),
            'video_duration': (300, 1200),  # 5-20 minutes optimal
            'engagement_factors': ['thumbnail', 'title_optimization', 'tags', 'end_screens']
        },
        'facebook': {
            'optimal_length': (40, 400),
            'video_duration': (60, 300),
            'engagement_factors': ['reactions', 'shares', 'comments', 'live_video']
        },
        'twitter': {
            'optimal_length': (71, 140),
            'hashtag_limit': 2,
            'engagement_factors': ['retweets', 'replies', 'trends', 'threads']
        }
    }
    
    @classmethod
    def analyze_platform_fit(cls, content: str, platform: str, content_type: str = 'text') -> Dict[str, float]:
        """Analyze how well content fits platform requirements"""
        platform = platform.lower()
        
        if platform not in cls.PLATFORM_CONFIGS:
            return {'platform_score': 0.5, 'optimization_suggestions': []}
        
        config = cls.PLATFORM_CONFIGS[platform]
        scores = {}
        suggestions = []
        
        # Analyze text content
        if content_type == 'text':
            scores.update(cls._analyze_text_fit(content, config, suggestions))
        
        # Calculate overall platform fit
        platform_score = np.mean(list(scores.values())) if scores else 0.5
        
        return {
            'platform_score': platform_score,
            'detailed_scores': scores,
            'optimization_suggestions': suggestions
        }
    
    @classmethod
    def _analyze_text_fit(cls, content: str, config: Dict, suggestions: List[str]) -> Dict[str, float]:
        """Analyze text content fit for platform"""
        scores = {}
        
        # Length analysis
        if 'optimal_length' in config:
            min_len, max_len = config['optimal_length']
            content_len = len(content)
            
            if min_len <= content_len <= max_len:
                scores['length_score'] = 1.0
            elif content_len < min_len:
                scores['length_score'] = content_len / min_len
                suggestions.append(f"Content too short. Add {min_len - content_len} more characters.")
            else:  # content_len > max_len
                scores['length_score'] = max(0.3, max_len / content_len)
                suggestions.append(f"Content too long. Remove {content_len - max_len} characters.")
        
        # Hashtag analysis
        hashtags = re.findall(r'#\w+', content)
        if 'optimal_hashtags' in config:
            min_tags, max_tags = config['optimal_hashtags']
            tag_count = len(hashtags)
            
            if min_tags <= tag_count <= max_tags:
                scores['hashtag_score'] = 1.0
            elif tag_count < min_tags:
                scores['hashtag_score'] = tag_count / min_tags if min_tags > 0 else 0
                suggestions.append(f"Add {min_tags - tag_count} more hashtags.")
            else:
                scores['hashtag_score'] = max(0.3, max_tags / tag_count)
                suggestions.append(f"Remove {tag_count - max_tags} hashtags.")
        
        # Mention analysis
        mentions = re.findall(r'@\w+', content)
        scores['mention_score'] = min(1.0, len(mentions) * 0.2)  # Up to 5 mentions = 1.0
        
        return scores

class BehaviorAnalyzer:
    """User behavior pattern analysis for bot detection"""
    
    def __init__(self):
        self.cache_manager = CacheManager()
    
    def analyze_user_behavior(self, user_id: str, activity_window_hours: int = 168) -> UserBehaviorProfile:
        """Comprehensive user behavior analysis"""
        try:
            # Get user activity from database
            cutoff_time = datetime.now() - timedelta(hours=activity_window_hours)
            activities = db.user_activities.find({
                'user_id': user_id,
                'timestamp': {'$gte': cutoff_time}
            }).sort('timestamp', 1)
            
            activity_list = list(activities)
            
            if not activity_list:
                return self._default_behavior_profile(user_id)
            
            # Analyze patterns
            profile = UserBehaviorProfile(
                user_id=user_id,
                activity_pattern=self._analyze_activity_pattern(activity_list),
                content_consistency=self._analyze_content_consistency(activity_list),
                posting_rhythm=self._analyze_posting_rhythm(activity_list),
                device_fingerprint=self._analyze_device_consistency(activity_list),
                interaction_quality=self._analyze_interaction_quality(activity_list),
                network_authenticity=self._analyze_network_authenticity(user_id),
                human_score=0.0  # Will be calculated
            )
            
            # Calculate overall human probability
            profile.human_score = self._calculate_human_probability(profile)
            
            return profile
            
        except Exception as e:
            logger.error(f"Behavior analysis error for user {user_id}: {e}")
            return self._default_behavior_profile(user_id)
    
    def _default_behavior_profile(self, user_id: str) -> UserBehaviorProfile:
        """Return default behavior profile for new or error cases"""
        return UserBehaviorProfile(
            user_id=user_id,
            activity_pattern={'morning': 0.25, 'afternoon': 0.25, 'evening': 0.25, 'night': 0.25},
            content_consistency=0.5,
            posting_rhythm={'hourly_variance': 60, 'daily_pattern': 'normal'},
            device_fingerprint='unknown',
            interaction_quality=0.5,
            network_authenticity=0.5,
            human_score=0.5
        )
    
    def _analyze_activity_pattern(self, activities: List[Dict]) -> Dict[str, float]:
        """Analyze temporal activity patterns"""
        pattern = {'morning': 0, 'afternoon': 0, 'evening': 0, 'night': 0}
        
        for activity in activities:
            hour = activity['timestamp'].hour
            
            if 6 <= hour < 12:
                pattern['morning'] += 1
            elif 12 <= hour < 18:
                pattern['afternoon'] += 1
            elif 18 <= hour < 24:
                pattern['evening'] += 1
            else:  # 0 <= hour < 6
                pattern['night'] += 1
        
        total = sum(pattern.values())
        if total > 0:
            pattern = {k: v/total for k, v in pattern.items()}
        
        return pattern
    
    def _analyze_content_consistency(self, activities: List[Dict]) -> float:
        """Analyze consistency in content creation patterns"""
        try:
            content_lengths = []
            posting_intervals = []
            content_types = []
            
            for i, activity in enumerate(activities):
                if 'content' in activity:
                    content_lengths.append(len(activity['content']))
                    content_types.append(activity.get('type', 'unknown'))
                
                if i > 0:
                    interval = (activity['timestamp'] - activities[i-1]['timestamp']).total_seconds()
                    posting_intervals.append(interval)
            
            consistency_score = 0.0
            
            # Length consistency (moderate variance is human-like)
            if content_lengths:
                length_variance = np.var(content_lengths)
                optimal_variance = np.mean(content_lengths) * 0.5
                length_consistency = 1.0 - abs(length_variance - optimal_variance) / optimal_variance
                consistency_score += max(0, length_consistency) * 0.4
            
            # Interval consistency (human posting should have natural variance)
            if posting_intervals:
                interval_variance = np.var(posting_intervals)
                mean_interval = np.mean(posting_intervals)
                if mean_interval > 0:
                    coefficient_of_variation = np.sqrt(interval_variance) / mean_interval
                    # Sweet spot for human behavior
                    if 0.3 <= coefficient_of_variation <= 1.5:
                        interval_consistency = 1.0
                    else:
                        interval_consistency = max(0, 1.0 - abs(coefficient_of_variation - 0.9) / 0.9)
                    consistency_score += interval_consistency * 0.6
            
            return min(1.0, max(0.0, consistency_score))
            
        except Exception as e:
            logger.error(f"Content consistency analysis error: {e}")
            return 0.5
    
    def _analyze_posting_rhythm(self, activities: List[Dict]) -> Dict[str, Union[int, str]]:
        """Analyze posting rhythm and detect bot-like patterns"""
        try:
            timestamps = [activity['timestamp'] for activity in activities]
            
            if len(timestamps) < 2:
                return {'hourly_variance': 60, 'daily_pattern': 'insufficient_data'}
            
            # Calculate intervals between posts
            intervals = []
            for i in range(1, len(timestamps)):
                interval_seconds = (timestamps[i] - timestamps[i-1]).total_seconds()
                intervals.append(interval_seconds)
            
            # Hourly variance analysis
            hourly_variance = int(np.var(intervals) / 3600) if intervals else 60
            
            # Daily pattern analysis
            hours = [ts.hour for ts in timestamps]
            hour_distribution = Counter(hours)
            
            # Detect patterns
            if len(set(hours)) <= 3 and len(hours) > 10:
                daily_pattern = 'bot_like'  # Too concentrated
            elif max(hour_distribution.values()) > len(hours) * 0.7:
                daily_pattern = 'suspicious'  # One hour dominates
            else:
                daily_pattern = 'normal'
            
            return {
                'hourly_variance': hourly_variance,
                'daily_pattern': daily_pattern,
                'peak_hour': max(hour_distribution, key=hour_distribution.get) if hour_distribution else 12,
                'activity_spread': len(set(hours))
            }
            
        except Exception as e:
            logger.error(f"Posting rhythm analysis error: {e}")
            return {'hourly_variance': 60, 'daily_pattern': 'error'}
    
    def _analyze_device_consistency(self, activities: List[Dict]) -> str:
        """Analyze device fingerprint consistency"""
        try:
            devices = [activity.get('device_fingerprint', 'unknown') for activity in activities]
            unique_devices = set(devices)
            
            if len(unique_devices) == 1:
                return devices[0]
            elif len(unique_devices) <= 3:  # Reasonable for mobile + desktop
                return 'multiple_consistent'
            else:
                return 'suspicious_multiple'
                
        except Exception:
            return 'unknown'
    
    def _analyze_interaction_quality(self, activities: List[Dict]) -> float:
        """Analyze quality of user interactions"""
        try:
            interaction_scores = []
            
            for activity in activities:
                activity_type = activity.get('type', '')
                content = activity.get('content', '')
                
                # Score based on activity type and content quality
                if activity_type == 'post':
                    score = min(1.0, len(content) / 100)  # Longer posts generally better
                elif activity_type == 'comment':
                    score = min(1.0, len(content) / 50)   # Meaningful comments
                elif activity_type == 'like':
                    score = 0.1  # Low effort
                elif activity_type == 'share':
                    score = 0.3  # Medium effort
                else:
                    score = 0.2  # Default
                
                # Bonus for original content
                if content and len(content) > 20:
                    text_analyzer = TextAnalyzer()
                    originality = text_analyzer.calculate_originality(content, activity['user_id'])
                    score *= (0.5 + originality * 0.5)
                
                interaction_scores.append(score)
            
            return np.mean(interaction_scores) if interaction_scores else 0.5
            
        except Exception as e:
            logger.error(f"Interaction quality analysis error: {e}")
            return 0.5
    
    def _analyze_network_authenticity(self, user_id: str) -> float:
        """Analyze authenticity of user's referral network"""
        try:
            # Get user's referral network
            network_data = db.referral_networks.find_one({'user_id': user_id})
            
            if not network_data:
                return 0.5  # No network data
            
            referrals = network_data.get('referrals', [])
            
            if not referrals:
                return 1.0  # No referrals is fine
            
            authenticity_factors = []
            
            # Check referral registration patterns
            reg_times = [ref.get('registered_at') for ref in referrals if ref.get('registered_at')]
            if len(reg_times) > 1:
                # Check for suspicious clustering
                time_diffs = [(reg_times[i] - reg_times[i-1]).total_seconds() 
                             for i in range(1, len(reg_times))]
                
                # Very small intervals suggest bot creation
                if any(diff < 60 for diff in time_diffs):  # Less than 1 minute
                    authenticity_factors.append(0.2)
                elif any(diff < 300 for diff in time_diffs):  # Less than 5 minutes
                    authenticity_factors.append(0.5)
                else:
                    authenticity_factors.append(1.0)
            
            # Check referral activity diversity
            activity_scores = [ref.get('activity_score', 0) for ref in referrals]
            if activity_scores:
                activity_variance = np.var(activity_scores)
                mean_activity = np.mean(activity_scores)
                
                if mean_activity > 0:
                    diversity_score = min(1.0, activity_variance / mean_activity)
                    authenticity_factors.append(diversity_score)
            
            # Check for circular referrals (A refers B, B refers A)
            user_referrals = {ref['user_id'] for ref in referrals}
            circular_count = 0
            
            for ref_id in user_referrals:
                ref_network = db.referral_networks.find_one({'user_id': ref_id})
                if ref_network:
                    ref_referrals = {r['user_id'] for r in ref_network.get('referrals', [])}
                    if user_id in ref_referrals:
                        circular_count += 1
            
            circular_ratio = circular_count / len(referrals) if referrals else 0
            circular_score = max(0.0, 1.0 - circular_ratio * 2)  # Penalize circular referrals
            authenticity_factors.append(circular_score)
            
            return np.mean(authenticity_factors) if authenticity_factors else 0.5
            
        except Exception as e:
            logger.error(f"Network authenticity analysis error: {e}")
            return 0.5
    
    def _calculate_human_probability(self, profile: UserBehaviorProfile) -> float:
        """Calculate overall human probability score"""
        try:
            factors = {
                'activity_naturalness': self._score_activity_naturalness(profile.activity_pattern),
                'content_consistency': profile.content_consistency,
                'rhythm_humanness': self._score_rhythm_humanness(profile.posting_rhythm),
                'device_consistency': self._score_device_consistency(profile.device_fingerprint),
                'interaction_quality': profile.interaction_quality,
                'network_authenticity': profile.network_authenticity
            }
            
            # Weights for different factors
            weights = {
                'activity_naturalness': 0.20,
                'content_consistency': 0.15,
                'rhythm_humanness': 0.25,
                'device_consistency': 0.10,
                'interaction_quality': 0.20,
                'network_authenticity': 0.10
            }
            
            weighted_score = sum(factors[key] * weights[key] for key in factors.keys())
            
            # Apply sigmoid function for smoother distribution
            human_probability = 1 / (1 + math.exp(-5 * (weighted_score - 0.5)))
            
            return min(1.0, max(0.0, human_probability))
            
        except Exception as e:
            logger.error(f"Human probability calculation error: {e}")
            return 0.5
    
    @staticmethod
    def _score_activity_naturalness(pattern: Dict[str, float]) -> float:
        """Score how natural the activity pattern looks"""
        # Natural human pattern: more active during day, less at night
        ideal_pattern = {'morning': 0.3, 'afternoon': 0.35, 'evening': 0.25, 'night': 0.1}
        
        # Calculate KL divergence (lower = more natural)
        kl_divergence = 0
        for period in ideal_pattern:
            if pattern[period] > 0:
                kl_divergence += ideal_pattern[period] * math.log(ideal_pattern[period] / pattern[period])
        
        # Convert to 0-1 score (lower KL = higher score)
        naturalness = max(0.0, 1.0 - kl_divergence)
        return naturalness
    
    @staticmethod
    def _score_rhythm_humanness(rhythm: Dict[str, Union[int, str]]) -> float:
        """Score posting rhythm humanness"""
        if rhythm['daily_pattern'] == 'bot_like':
            return 0.1
        elif rhythm['daily_pattern'] == 'suspicious':
            return 0.3
        elif rhythm['daily_pattern'] == 'normal':
            base_score = 0.8
        else:
            base_score = 0.5
        
        # Variance bonus (humans have natural variance)
        variance = rhythm['hourly_variance']
        if 30 <= variance <= 120:  # Optimal human variance range
            variance_bonus = 0.2
        elif 10 <= variance <= 240:  # Acceptable range
            variance_bonus = 0.1
        else:
            variance_bonus = 0.0
        
        return min(1.0, base_score + variance_bonus)
    
    @staticmethod
    def _score_device_consistency(device_fingerprint: str) -> float:
        """Score device usage consistency"""
        if device_fingerprint == 'unknown':
            return 0.3
        elif device_fingerprint == 'suspicious_multiple':
            return 0.2
        elif device_fingerprint == 'multiple_consistent':
            return 0.9  # Normal to use multiple devices
        else:
            return 0.8  # Single consistent device

class QualityScorer:
    """Comprehensive content quality scoring system"""
    
    def __init__(self):
        self.text_analyzer = TextAnalyzer()
        self.image_analyzer = ImageAnalyzer()
        self.video_analyzer = VideoAnalyzer()
        self.platform_analyzer = PlatformSpecificAnalyzer()
        self.behavior_analyzer = BehaviorAnalyzer()
        self.cache_manager = CacheManager()
    
    @CacheManager().cache_decorator(ttl=1800)  # 30 minutes cache
    def calculate_comprehensive_quality_score(
        self, 
        content: str, 
        user_id: str, 
        platform: str,
        content_type: str = 'text',
        media_path: Optional[str] = None
    ) -> ContentMetrics:
        """Calculate comprehensive quality score for content"""
        try:
            # Initialize metrics
            metrics = {
                'originality_score': 0.0,
                'quality_score': 0.0,
                'engagement_potential': 0.0,
                'platform_relevance': 0.0,
                'brand_safety_score': 0.0,
                'human_probability': 0.0,
                'sentiment_score': 0.0,
                'readability_score': 0.0
            }
            
            # Text analysis
            if content.strip():
                metrics['originality_score'] = self.text_analyzer.calculate_originality(content, user_id)
                metrics['sentiment_score'] = self.text_analyzer.analyze_sentiment(content)['compound']
                readability = self.text_analyzer.analyze_readability(content)
                metrics['readability_score'] = self._normalize_readability_score(readability['avg_score'])
                metrics['human_probability'] = 1.0 - self.text_analyzer.detect_ai_generated(content)
                metrics['brand_safety_score'] = self._analyze_brand_safety(content)
            
            # Platform-specific analysis
            platform_analysis = self.platform_analyzer.analyze_platform_fit(content, platform, content_type)
            metrics['platform_relevance'] = platform_analysis['platform_score']
            
            # Media analysis
            if media_path and content_type in ['image', 'video']:
                media_quality = self._analyze_media_quality(media_path, content_type)
                metrics['quality_score'] = media_quality
            else:
                metrics['quality_score'] = self._calculate_text_quality(content)
            
            # Engagement potential
            metrics['engagement_potential'] = self._predict_engagement_potential(
                content, platform, metrics['quality_score'], metrics['sentiment_score']
            )
            
            # User behavior factor
            behavior_profile = self.behavior_analyzer.analyze_user_behavior(user_id)
            user_credibility = behavior_profile.human_score
            
            # Apply user credibility to all scores
            for key in metrics:
                if key != 'human_probability':
                    metrics[key] *= (0.5 + user_credibility * 0.5)
            
            # Create final metrics object
            final_metrics = ContentMetrics(
                originality_score=metrics['originality_score'],
                quality_score=metrics['quality_score'],
                engagement_potential=metrics['engagement_potential'],
                platform_relevance=metrics['platform_relevance'],
                brand_safety_score=metrics['brand_safety_score'],
                human_probability=user_credibility,
                sentiment_score=metrics['sentiment_score'],
                readability_score=metrics['readability_score'],
                uniqueness_hash=SecurityManager().hash_content(content),
                analysis_timestamp=datetime.now()
            )
            
            # Store analysis results
            self._store_analysis_results(user_id, final_metrics, platform)
            
            return final_metrics
            
        except Exception as e:
            logger.error(f"Quality score calculation error: {e}")
            return self._default_content_metrics()
    
    def _normalize_readability_score(self, avg_readability: float) -> float:
        """Normalize readability score to 0-1 range"""
        # Optimal readability is around grade 8-12
        if 8 <= avg_readability <= 12:
            return 1.0
        elif 6 <= avg_readability <= 16:
            return 0.8
        elif 4 <= avg_readability <= 20:
            return 0.6
        else:
            return 0.4
    
    def _analyze_brand_safety(self, content: str) -> float:
        """Analyze brand safety score (1.0 = safe, 0.0 = unsafe)"""
        try:
            content_lower = content.lower()
            
            # Define unsafe content patterns
            unsafe_patterns = [
                r'\b(hate|kill|death|murder|violence)\b',
                r'\b(drug|cocaine|heroin|marijuana)\b',
                r'\b(scam|fraud|ponzi|pyramid)\b',
                r'\b(porn|sex|xxx|adult)\b',
                r'\b(terrorist|bomb|weapon|gun)\b'
            ]
            
            safety_score = 1.0
            
            for pattern in unsafe_patterns:
                matches = len(re.findall(pattern, content_lower))
                safety_score -= matches * 0.2
            
            # Check for excessive profanity
            profanity_words = ['fuck', 'shit', 'damn', 'hell', 'ass', 'bitch']
            profanity_count = sum(1 for word in profanity_words if word in content_lower)
            safety_score -= min(0.3, profanity_count * 0.1)
            
            return max(0.0, min(1.0, safety_score))
            
        except Exception as e:
            logger.error(f"Brand safety analysis error: {e}")
            return 0.7  # Conservative default
    
    def _analyze_media_quality(self, media_path: str, content_type: str) -> float:
        """Analyze media quality based on type"""
        try:
            if content_type == 'image':
                quality_metrics = self.image_analyzer.analyze_image_quality(media_path)
                return quality_metrics['overall_quality']
            elif content_type == 'video':
                quality_metrics = self.video_analyzer.analyze_video_quality(media_path)
                return quality_metrics['overall_quality']
            else:
                return 0.5
        except Exception as e:
            logger.error(f"Media quality analysis error: {e}")
            return 0.5
    
    def _calculate_text_quality(self, text: str) -> float:
        """Calculate text content quality"""
        if not text.strip():
            return 0.0
        
        try:
            quality_factors = []
            
            # Length factor (optimal range)
            length = len(text)
            if 50 <= length <= 1000:
                length_score = 1.0
            elif 20 <= length <= 2000:
                length_score = 0.8
            elif 10 <= length <= 5000:
                length_score = 0.6
            else:
                length_score = 0.3
            quality_factors.append(length_score)
            
            # Vocabulary richness
            words = text.lower().split()
            unique_words = set(words)
            vocabulary_richness = len(unique_words) / len(words) if words else 0
            quality_factors.append(min(1.0, vocabulary_richness * 2))
            
            # Sentence structure variety
            sentences = sent_tokenize(text)
            if len(sentences) > 1:
                sentence_lengths = [len(s.split()) for s in sentences]
                sentence_variance = np.var(sentence_lengths)
                structure_score = min(1.0, sentence_variance / 20)
                quality_factors.append(structure_score)
            
            # Grammar and punctuation (simplified)
            punctuation_ratio = sum(1 for c in text if c in '.,!?;:') / len(text)
            grammar_score = min(1.0, punctuation_ratio * 50)
            quality_factors.append(grammar_score)
            
            return np.mean(quality_factors)
            
        except Exception as e:
            logger.error(f"Text quality calculation error: {e}")
            return 0.5
    
    def _predict_engagement_potential(
        self, 
        content: str, 
        platform: str, 
        quality_score: float, 
        sentiment_score: float
    ) -> float:
        """Predict engagement potential based on content and platform"""
        try:
            engagement_factors = []
            
            # Base quality contribution
            engagement_factors.append(quality_score * 0.4)
            
            # Sentiment contribution (slightly positive content performs well)
            if 0.1 <= sentiment_score <= 0.8:
                sentiment_contribution = 1.0
            elif -0.2 <= sentiment_score <= 0.9:
                sentiment_contribution = 0.8
            else:
                sentiment_contribution = 0.5
            engagement_factors.append(sentiment_contribution * 0.2)
            
            # Platform-specific factors
            platform_boost = self._get_platform_engagement_boost(content, platform)
            engagement_factors.append(platform_boost * 0.3)
            
            # Content type bonus
            content_type_bonus = self._analyze_content_type_engagement(content)
            engagement_factors.append(content_type_bonus * 0.1)
            
            return min(1.0, max(0.0, sum(engagement_factors)))
            
        except Exception as e:
            logger.error(f"Engagement prediction error: {e}")
            return 0.5
    
    def _get_platform_engagement_boost(self, content: str, platform: str) -> float:
        """Get platform-specific engagement boost"""
        platform_patterns = {
            'instagram': [r'#\w+', r'@\w+', r'📸|🌟|💫|✨'],  # Hashtags, mentions, emojis
            'tiktok': [r'#\w+', r'🎵|🎶|💃|🕺', r'\b(viral|trending|fyp)\b'],
            'youtube': [r'\b(subscribe|like|comment|share)\b', r'👍|👎|🔔'],
            'facebook': [r'\b(friends|family|share)\b', r'❤️|👍|😂|😮|😢|😡'],
            'twitter': [r'#\w+', r'@\w+', r'\b(thread|viral|breaking)\b', r'🔥|💯|🚀']
        }
        
        if platform not in platform_patterns:
            return 0.5
        
        boost_score = 0.5
        patterns = platform_patterns[platform]
        
        for pattern in patterns:
            matches = len(re.findall(pattern, content, re.IGNORECASE))
            boost_score += min(0.15, matches * 0.05)
        
        return min(1.0, boost_score)
    
    def _analyze_content_type_engagement(self, content: str) -> float:
        """Analyze content type for engagement potential"""
        try:
            engagement_score = 0.5
            
            # Question content (encourages responses)
            if '?' in content:
                question_count = content.count('?')
                engagement_score += min(0.3, question_count * 0.1)
            
            # Call-to-action phrases
            cta_patterns = [
                r'\b(comment|share|like|follow|subscribe)\b',
                r'\b(what do you think|your thoughts|tell me|let me know)\b'
            ]
            
            for pattern in cta_patterns:
                matches = len(re.findall(pattern, content, re.IGNORECASE))
                engagement_score += min(0.2, matches * 0.1)
            
            # Controversy/discussion triggers (moderate amount is good)
            controversial_words = ['debate', 'opinion', 'controversial', 'unpopular', 'agree', 'disagree']
            controversy_count = sum(1 for word in controversial_words if word in content.lower())
            if 1 <= controversy_count <= 3:
                engagement_score += 0.15
            
            return min(1.0, engagement_score)
            
        except Exception:
            return 0.5
    
    def _store_analysis_results(self, user_id: str, metrics: ContentMetrics, platform: str):
        """Store analysis results for future reference and learning"""
        try:
            analysis_doc = {
                'user_id': user_id,
                'platform': platform,
                'metrics': {
                    'originality_score': metrics.originality_score,
                    'quality_score': metrics.quality_score,
                    'engagement_potential': metrics.engagement_potential,
                    'platform_relevance': metrics.platform_relevance,
                    'brand_safety_score': metrics.brand_safety_score,
                    'human_probability': metrics.human_probability,
                    'sentiment_score': metrics.sentiment_score,
                    'readability_score': metrics.readability_score
                },
                'content_hash': metrics.uniqueness_hash,
                'analysis_timestamp': metrics.analysis_timestamp,
                'created_at': datetime.now()
            }
            
            db.content_analysis.insert_one(analysis_doc)
            
        except Exception as e:
            logger.error(f"Error storing analysis results: {e}")
    
    def _default_content_metrics(self) -> ContentMetrics:
        """Return default metrics when analysis fails"""
        return ContentMetrics(
            originality_score=0.5,
            quality_score=0.5,
            engagement_potential=0.5,
            platform_relevance=0.5,
            brand_safety_score=0.7,
            human_probability=0.5,
            sentiment_score=0.0,
            readability_score=0.5,
            uniqueness_hash='default',
            analysis_timestamp=datetime.now()
        )

class NetworkAnalyzer:
    """Referral network analysis for RP system optimization"""
    
    @staticmethod
    def calculate_network_quality_score(user_id: str) -> Dict[str, float]:
        """Calculate network quality metrics for RP system"""
        try:
            # Get network data
            network_data = db.referral_networks.find_one({'user_id': user_id})
            
            if not network_data:
                return {
                    'network_size': 0,
                    'active_ratio': 0.0,
                    'quality_score': 0.0,
                    'diversity_score': 0.0,
                    'retention_rate': 0.0,
                    'growth_velocity': 0.0,
                    'overall_network_score': 0.0
                }
            
            referrals = network_data.get('referrals', [])
            
            if not referrals:
                return NetworkAnalyzer._default_network_metrics()
            
            # Calculate metrics
            network_size = len(referrals)
            
            # Active ratio (active in last 30 days)
            cutoff_date = datetime.now() - timedelta(days=30)
            active_referrals = [r for r in referrals if r.get('last_active', cutoff_date) >= cutoff_date]
            active_ratio = len(active_referrals) / network_size
            
            # Quality score (based on referral activity levels)
            activity_scores = [r.get('activity_score', 0) for r in referrals]
            avg_activity = np.mean(activity_scores) if activity_scores else 0
            quality_score = min(1.0, avg_activity / 100)  # Normalize to 0-1
            
            # Diversity score (geographic and temporal)
            diversity_score = NetworkAnalyzer._calculate_network_diversity(referrals)
            
            # Retention rate (30-day retention)
            retention_rate = NetworkAnalyzer._calculate_retention_rate(referrals)
            
            # Growth velocity (referrals per week trend)
            growth_velocity = NetworkAnalyzer._calculate_growth_velocity(referrals)
            
            # Overall network score
            weights = {
                'active_ratio': 0.25,
                'quality_score': 0.25,
                'diversity_score': 0.20,
                'retention_rate': 0.20,
                'growth_velocity': 0.10
            }
            
            overall_score = (
                active_ratio * weights['active_ratio'] +
                quality_score * weights['quality_score'] +
                diversity_score * weights['diversity_score'] +
                retention_rate * weights['retention_rate'] +
                growth_velocity * weights['growth_velocity']
            )
            
            return {
                'network_size': network_size,
                'active_ratio': active_ratio,
                'quality_score': quality_score,
                'diversity_score': diversity_score,
                'retention_rate': retention_rate,
                'growth_velocity': growth_velocity,
                'overall_network_score': overall_score
            }
            
        except Exception as e:
            logger.error(f"Network quality analysis error: {e}")
            return NetworkAnalyzer._default_network_metrics()
    
    @staticmethod
    def _default_network_metrics() -> Dict[str, float]:
        """Default network metrics for error cases"""
        return {
            'network_size': 0,
            'active_ratio': 0.0,
            'quality_score': 0.0,
            'diversity_score': 0.0,
            'retention_rate': 0.0,
            'growth_velocity': 0.0,
            'overall_network_score': 0.0
        }
    
    @staticmethod
    def _calculate_network_diversity(referrals: List[Dict]) -> float:
        """Calculate network diversity score"""
        try:
            if len(referrals) < 2:
                return 1.0
            
            # Geographic diversity
            countries = [r.get('country', 'unknown') for r in referrals]
            unique_countries = len(set(countries))
            geo_diversity = min(1.0, unique_countries / min(10, len(referrals)))
            
            # Temporal diversity (registration spread)
            reg_times = [r.get('registered_at') for r in referrals if r.get('registered_at')]
            if len(reg_times) > 1:
                reg_times.sort()
                time_span = (reg_times[-1] - reg_times[0]).total_seconds()
                temporal_diversity = min(1.0, time_span / (30 * 24 * 3600))  # 30 days max
            else:
                temporal_diversity = 0.5
            
            # Platform diversity
            platforms = []
            for r in referrals:
                user_platforms = r.get('active_platforms', [])
                platforms.extend(user_platforms)
            
            unique_platforms = len(set(platforms))
            platform_diversity = min(1.0, unique_platforms / 5)  # Up to 5 platforms
            
            return (geo_diversity + temporal_diversity + platform_diversity) / 3
            
        except Exception:
            return 0.5
    
    @staticmethod
    def _calculate_retention_rate(referrals: List[Dict]) -> float:
        """Calculate 30-day retention rate"""
        try:
            if not referrals:
                return 0.0
            
            cutoff_date = datetime.now() - timedelta(days=30)
            eligible_referrals = [r for r in referrals if r.get('registered_at', datetime.now()) <= cutoff_date]
            
            if not eligible_referrals:
                return 1.0  # All referrals are too new
            
            retained_referrals = [r for r in eligible_referrals 
                                if r.get('last_active', datetime.min) >= cutoff_date]
            
            return len(retained_referrals) / len(eligible_referrals)
            
        except Exception:
            return 0.5
    
    @staticmethod
    def _calculate_growth_velocity(referrals: List[Dict]) -> float:
        """Calculate network growth velocity"""
        try:
            if len(referrals) < 2:
                return 0.0
            
            # Get registration times
            reg_times = [r.get('registered_at') for r in referrals if r.get('registered_at')]
            reg_times.sort()
            
            # Calculate weekly growth rate
            first_week = reg_times[0]
            current_time = datetime.now()
            weeks_elapsed = (current_time - first_week).total_seconds() / (7 * 24 * 3600)
            
            if weeks_elapsed > 0:
                growth_rate = len(referrals) / weeks_elapsed
                # Normalize to 0-1 scale (10 referrals/week = 1.0)
                return min(1.0, growth_rate / 10)
            
            return 0.0
            
        except Exception:
            return 0.0

class RewardCalculator:
    """Calculate integrated rewards (XP, RP, $FIN) based on quality scores"""
    
    # Finova Network reward constants from whitepaper
    BASE_MINING_RATE = 0.05  # $FIN per hour
    XP_BASE_VALUES = {
        'post': 50,
        'comment': 25,
        'like': 5,
        'share': 15,
        'video': 150,
        'image': 75,
        'story': 25
    }
    
    PLATFORM_MULTIPLIERS = {
        'tiktok': 1.3,
        'youtube': 1.4,
        'instagram': 1.2,
        'facebook': 1.1,
        'twitter': 1.2
    }
    
    @staticmethod
    def calculate_integrated_rewards(
        user_id: str,
        activity_type: str,
        platform: str,
        quality_metrics: ContentMetrics,
        user_level: int = 1,
        rp_tier: int = 0,
        streak_days: int = 0
    ) -> Dict[str, float]:
        """Calculate integrated XP, RP, and $FIN rewards"""
        try:
            # XP Calculation
            base_xp = RewardCalculator.XP_BASE_VALUES.get(activity_type, 25)
            platform_multiplier = RewardCalculator.PLATFORM_MULTIPLIERS.get(platform, 1.0)
            quality_multiplier = (quality_metrics.quality_score + quality_metrics.originality_score) / 2
            streak_bonus = min(3.0, 1.0 + (streak_days * 0.1))
            level_progression = math.exp(-0.01 * user_level)
            
            xp_reward = base_xp * platform_multiplier * quality_multiplier * streak_bonus * level_progression
            
            # Mining Rate Calculation
            base_mining = RewardCalculator.BASE_MINING_RATE
            xp_level_bonus = 1.0 + (user_level / 100)
            rp_tier_bonus = 1.0 + (rp_tier * 0.2)
            quality_bonus = quality_metrics.quality_score
            
            mining_rate = base_mining * xp_level_bonus * rp_tier_bonus * quality_bonus
            
            # RP Calculation (for referral benefits)
            rp_base = 10  # Base RP for activity
            network_quality_bonus = quality_metrics.human_probability
            engagement_bonus = quality_metrics.engagement_potential
            
            rp_reward = rp_base * network_quality_bonus * engagement_bonus
            
            # Apply brand safety penalty
            safety_penalty = quality_metrics.brand_safety_score
            
            return {
                'xp_reward': xp_reward * safety_penalty,
                'mining_rate_boost': mining_rate * safety_penalty,
                'rp_reward': rp_reward * safety_penalty,
                'quality_multiplier': quality_multiplier,
                'safety_penalty': safety_penalty,
                'total_value_score': (xp_reward + mining_rate * 24 + rp_reward) * safety_penalty
            }
            
        except Exception as e:
            logger.error(f"Reward calculation error: {e}")
            return {
                'xp_reward': 0.0,
                'mining_rate_boost': 0.0,
                'rp_reward': 0.0,
                'quality_multiplier': 0.0,
                'safety_penalty': 0.7,
                'total_value_score': 0.0
            }

class APIResponseManager:
    """Manage API responses and error handling"""
    
    @staticmethod
    def create_success_response(data: Any, message: str = "Success") -> Dict[str, Any]:
        """Create standardized success response"""
        return {
            'success': True,
            'message': message,
            'data': data,
            'timestamp': datetime.now().isoformat(),
            'version': '3.0.0'
        }
    
    @staticmethod
    def create_error_response(error_code: str, message: str, details: Optional[Dict] = None) -> Dict[str, Any]:
        """Create standardized error response"""
        return {
            'success': False,
            'error_code': error_code,
            'message': message,
            'details': details or {},
            'timestamp': datetime.now().isoformat(),
            'version': '3.0.0'
        }
    
    @staticmethod
    def validate_request_data(data: Dict, required_fields: List[str]) -> Tuple[bool, Optional[str]]:
        """Validate request data has required fields"""
        try:
            for field in required_fields:
                if field not in data:
                    return False, f"Missing required field: {field}"
                
                if data[field] is None or (isinstance(data[field], str) and not data[field].strip()):
                    return False, f"Field '{field}' cannot be empty"
            
            return True, None
            
        except Exception as e:
            return False, f"Validation error: {str(e)}"

class PerformanceMonitor:
    """Monitor and optimize AI service performance"""
    
    def __init__(self):
        self.metrics = defaultdict(list)
        self.start_time = time.time()
    
    def record_execution_time(self, function_name: str, execution_time: float):
        """Record function execution time"""
        self.metrics[f"{function_name}_execution_time"].append(execution_time)
        
        # Keep only last 1000 records per function
        if len(self.metrics[f"{function_name}_execution_time"]) > 1000:
            self.metrics[f"{function_name}_execution_time"] = \
                self.metrics[f"{function_name}_execution_time"][-1000:]
    
    def get_performance_stats(self) -> Dict[str, Any]:
        """Get performance statistics"""
        stats = {}
        
        for metric_name, values in self.metrics.items():
            if values:
                stats[metric_name] = {
                    'count': len(values),
                    'mean': np.mean(values),
                    'median': np.median(values),
                    'std': np.std(values),
                    'min': np.min(values),
                    'max': np.max(values),
                    'p95': np.percentile(values, 95),
                    'p99': np.percentile(values, 99)
                }
        
        stats['uptime_seconds'] = time.time() - self.start_time
        return stats
    
    def performance_decorator(self, function_name: str = None):
        """Decorator to automatically track function performance"""
        def decorator(func):
            @wraps(func)
            def wrapper(*args, **kwargs):
                start_time = time.time()
                try:
                    result = func(*args, **kwargs)
                    return result
                finally:
                    execution_time = time.time() - start_time
                    name = function_name or func.__name__
                    self.record_execution_time(name, execution_time)
            return wrapper
        return decorator

class DataProcessor:
    """Data processing utilities for AI analysis"""
    
    @staticmethod
    def clean_text_content(text: str) -> str:
        """Clean and normalize text content"""
        if not text:
            return ""
        
        # Remove extra whitespace
        text = ' '.join(text.split())
        
        # Remove or replace special characters
        text = re.sub(r'[^\w\s.,!?@#$%&*()_+=\-\[\]{}|\\:";\'<>?/~`]', '', text)
        
        # Normalize URLs
        text = re.sub(r'http[s]?://(?:[a-zA-Z]|[0-9]|[$-_@.&+]|[!*\\(\\),]|(?:%[0-9a-fA-F][0-9a-fA-F]))+', 
                     '[URL]', text)
        
        # Normalize mentions and hashtags
        text = re.sub(r'@\w+', '[MENTION]', text)
        text = re.sub(r'#\w+', '[HASHTAG]', text)
        
        return text.strip()
    
    @staticmethod
    def extract_metadata(content: Dict[str, Any]) -> Dict[str, Any]:
        """Extract metadata from content object"""
        metadata = {
            'content_length': len(content.get('text', '')),
            'has_media': bool(content.get('media_url')),
            'media_type': content.get('media_type', 'none'),
            'platform': content.get('platform', 'unknown'),
            'timestamp': content.get('created_at', datetime.now()),
            'language': DataProcessor._detect_language(content.get('text', '')),
            'hashtag_count': len(re.findall(r'#\w+', content.get('text', ''))),
            'mention_count': len(re.findall(r'@\w+', content.get('text', ''))),
            'url_count': len(re.findall(r'http[s]?://\S+', content.get('text', '')))
        }
        
        return metadata
    
    @staticmethod
    def _detect_language(text: str) -> str:
        """Simple language detection (placeholder for actual implementation)"""
        # This is a simplified implementation
        # In production, use langdetect or similar library
        
        if not text.strip():
            return 'unknown'
        
        # Check for common Indonesian words
        indonesian_words = ['dan', 'yang', 'di', 'ke', 'dari', 'untuk', 'ini', 'itu', 'dengan', 'tidak']
        english_words = ['the', 'and', 'or', 'but', 'in', 'on', 'at', 'to', 'for', 'of']
        
        text_lower = text.lower()
        
        id_count = sum(1 for word in indonesian_words if word in text_lower)
        en_count = sum(1 for word in english_words if word in text_lower)
        
        if id_count > en_count:
            return 'id'
        elif en_count > 0:
            return 'en'
        else:
            return 'unknown'
    
    @staticmethod
    def batch_process_content(content_list: List[Dict], batch_size: int = 10) -> List[Dict]:
        """Process multiple content items in batches for efficiency"""
        results = []
        
        for i in range(0, len(content_list), batch_size):
            batch = content_list[i:i + batch_size]
            batch_results = []
            
            for content_item in batch:
                try:
                    # Process each item
                    processed_item = {
                        'id': content_item.get('id'),
                        'user_id': content_item.get('user_id'),
                        'processed_at': datetime.now(),
                        'metadata': DataProcessor.extract_metadata(content_item),
                        'cleaned_text': DataProcessor.clean_text_content(content_item.get('text', ''))
                    }
                    batch_results.append(processed_item)
                    
                except Exception as e:
                    logger.error(f"Batch processing error for item {content_item.get('id')}: {e}")
                    batch_results.append({
                        'id': content_item.get('id'),
                        'error': str(e),
                        'processed_at': datetime.now()
                    })
            
            results.extend(batch_results)
            
            # Small delay between batches to prevent overload
            if i + batch_size < len(content_list):
                time.sleep(0.1)
        
        return results

class TrendAnalyzer:
    """Analyze content trends and viral potential"""
    
    @staticmethod
    def analyze_trending_potential(content: str, platform: str, user_engagement_history: Dict) -> float:
        """Analyze potential for content to trend"""
        try:
            trend_score = 0.0
            
            # Check for trending keywords/hashtags
            trending_keywords = TrendAnalyzer._get_trending_keywords(platform)
            content_lower = content.lower()
            
            for keyword, weight in trending_keywords.items():
                if keyword in content_lower:
                    trend_score += weight
            
            # User engagement history factor
            avg_engagement = user_engagement_history.get('avg_engagement', 0)
            engagement_boost = min(0.3, avg_engagement / 1000)  # Normalize
            trend_score += engagement_boost
            
            # Content timing factor (posting during peak hours)
            current_hour = datetime.now().hour
            peak_hours = TrendAnalyzer._get_platform_peak_hours(platform)
            
            if current_hour in peak_hours:
                trend_score += 0.2
            
            # Content freshness (recent topics trend better)
            freshness_score = TrendAnalyzer._analyze_content_freshness(content)
            trend_score += freshness_score * 0.3
            
            return min(1.0, max(0.0, trend_score))
            
        except Exception as e:
            logger.error(f"Trending potential analysis error: {e}")
            return 0.5
    
    @staticmethod
    def _get_trending_keywords(platform: str) -> Dict[str, float]:
        """Get current trending keywords for platform (mock implementation)"""
        # In production, this would connect to platform APIs or trend services
        trending_keywords = {
            'viral': 0.3,
            'trending': 0.25,
            'fyp': 0.2,
            'breaking': 0.3,
            'exclusive': 0.15,
            'challenge': 0.2,
            'tutorial': 0.15,
            'review': 0.1
        }
        
        return trending_keywords
    
    @staticmethod
    def _get_platform_peak_hours(platform: str) -> List[int]:
        """Get peak engagement hours for each platform"""
        peak_hours = {
            'instagram': [11, 12, 13, 17, 18, 19, 20, 21],
            'tiktok': [9, 12, 19, 20, 21],
            'youtube': [14, 15, 16, 20, 21, 22],
            'facebook': [13, 15, 20, 21],
            'twitter': [8, 9, 12, 17, 18, 19]
        }
        
        return peak_hours.get(platform, [12, 18, 20])
    
    @staticmethod
    def _analyze_content_freshness(content: str) -> float:
        """Analyze how fresh/current the content topics are"""
        # Simplified implementation - check for time-sensitive words
        fresh_indicators = [
            'today', 'now', 'current', 'latest', 'new', 'breaking',
            'update', 'recent', 'this week', 'just', 'happening'
        ]
        
        content_lower = content.lower()
        freshness_count = sum(1 for indicator in fresh_indicators if indicator in content_lower)
        
        return min(1.0, freshness_count * 0.3)

# Global instances for reuse
security_manager = SecurityManager()
cache_manager = CacheManager()
performance_monitor = PerformanceMonitor()

# Utility functions for external use
def analyze_content_comprehensive(
    content: str,
    user_id: str,
    platform: str,
    content_type: str = 'text',
    media_path: Optional[str] = None
) -> Dict[str, Any]:
    """Main entry point for comprehensive content analysis"""
    try:
        quality_scorer = QualityScorer()
        
        # Get quality metrics
        metrics = quality_scorer.calculate_comprehensive_quality_score(
            content, user_id, platform, content_type, media_path
        )
        
        # Get user data for reward calculation
        user_data = db.users.find_one({'user_id': user_id})
        user_level = user_data.get('xp_level', 1) if user_data else 1
        rp_tier = user_data.get('rp_tier', 0) if user_data else 0
        streak_days = user_data.get('streak_days', 0) if user_data else 0
        
        # Calculate rewards
        rewards = RewardCalculator.calculate_integrated_rewards(
            user_id=user_id,
            activity_type=content_type,
            platform=platform,
            quality_metrics=metrics,
            user_level=user_level,
            rp_tier=rp_tier,
            streak_days=streak_days
        )
        
        # Analyze trending potential
        engagement_history = user_data.get('engagement_history', {}) if user_data else {}
        trending_potential = TrendAnalyzer.analyze_trending_potential(
            content, platform, engagement_history
        )
        
        return APIResponseManager.create_success_response({
            'quality_metrics': {
                'originality_score': metrics.originality_score,
                'quality_score': metrics.quality_score,
                'engagement_potential': metrics.engagement_potential,
                'platform_relevance': metrics.platform_relevance,
                'brand_safety_score': metrics.brand_safety_score,
                'human_probability': metrics.human_probability,
                'sentiment_score': metrics.sentiment_score,
                'readability_score': metrics.readability_score
            },
            'reward_calculation': rewards,
            'trending_potential': trending_potential,
            'recommendations': generate_improvement_recommendations(metrics, platform),
            'content_hash': metrics.uniqueness_hash
        })
        
    except Exception as e:
        logger.error(f"Comprehensive analysis error: {e}")
        return APIResponseManager.create_error_response(
            'ANALYSIS_ERROR',
            'Failed to analyze content',
            {'error': str(e)}
        )

def generate_improvement_recommendations(metrics: ContentMetrics, platform: str) -> List[str]:
    """Generate actionable recommendations for content improvement"""
    recommendations = []
    
    try:
        # Quality recommendations
        if metrics.quality_score < 0.6:
            recommendations.append("Improve content quality: Add more detailed information and engaging elements")
        
        if metrics.originality_score < 0.7:
            recommendations.append("Increase originality: Create more unique content different from your previous posts")
        
        if metrics.engagement_potential < 0.6:
            recommendations.append("Boost engagement: Add questions, calls-to-action, or trending hashtags")
        
        if metrics.platform_relevance < 0.7:
            recommendations.append(f"Optimize for {platform}: Follow platform-specific best practices")
        
        if metrics.readability_score < 0.6:
            recommendations.append("Improve readability: Use shorter sentences and simpler words")
        
        if metrics.brand_safety_score < 0.8:
            recommendations.append("Review content for brand safety: Avoid controversial or inappropriate language")
        
        # Platform-specific recommendations
        platform_tips = {
            'instagram': ["Add relevant hashtags (5-15)", "Include location tags", "Use high-quality images"],
            'tiktok': ["Use trending sounds", "Add trending hashtags", "Keep videos 15-60 seconds"],
            'youtube': ["Optimize title length (30-60 chars)", "Add detailed description", "Use custom thumbnail"],
            'facebook': ["Encourage comments and shares", "Post during peak hours", "Use engaging visuals"],
            'twitter': ["Keep tweets concise", "Use relevant hashtags (1-2)", "Engage with trending topics"]
        }
        
        if platform in platform_tips:
            recommendations.extend(platform_tips[platform][:2])  # Add top 2 tips
        
        return recommendations[:5]  # Limit to 5 recommendations
        
    except Exception as e:
        logger.error(f"Recommendation generation error: {e}")
        return ["Focus on creating original, high-quality content for your audience"]

def calculate_anti_bot_score(user_id: str) -> Dict[str, Any]:
    """Calculate comprehensive anti-bot score for user"""
    try:
        behavior_analyzer = BehaviorAnalyzer()
        profile = behavior_analyzer.analyze_user_behavior(user_id)
        
        # Additional checks
        account_age_days = (datetime.now() - db.users.find_one({'user_id': user_id}, {'created_at': 1})['created_at']).days
        
        # Account age factor (new accounts more suspicious)
        age_factor = min(1.0, account_age_days / 30)  # Full trust after 30 days
        
        # Network analysis
        network_metrics = NetworkAnalyzer.calculate_network_quality_score(user_id)
        network_authenticity = network_metrics['overall_network_score']
        
        # Combined anti-bot score
        final_score = (
            profile.human_score * 0.5 +
            age_factor * 0.2 +
            network_authenticity * 0.2 +
            profile.interaction_quality * 0.1
        )
        
        return {
            'anti_bot_score': final_score,
            'risk_level': 'low' if final_score >= 0.8 else 'medium' if final_score >= 0.5 else 'high',
            'behavior_profile': {
                'human_probability': profile.human_score,
                'activity_naturalness': profile.activity_pattern,
                'content_consistency': profile.content_consistency,
                'network_authenticity': network_authenticity
            },
            'account_factors': {
                'age_days': account_age_days,
                'age_factor': age_factor
            },
            'recommended_actions': generate_anti_bot_recommendations(final_score, profile)
        }
        
    except Exception as e:
        logger.error(f"Anti-bot score calculation error: {e}")
        return {
            'anti_bot_score': 0.5,
            'risk_level': 'medium',
            'error': str(e)
        }

def generate_anti_bot_recommendations(score: float, profile: UserBehaviorProfile) -> List[str]:
    """Generate recommendations based on anti-bot analysis"""
    recommendations = []
    
    if score < 0.3:
        recommendations.extend([
            "Account flagged for manual review",
            "Complete additional KYC verification",
            "Reduce posting frequency to natural levels"
        ])
    elif score < 0.6:
        recommendations.extend([
            "Vary posting times and content types",
            "Increase interaction quality",
            "Build authentic referral network"
        ])
    elif score < 0.8:
        recommendations.extend([
            "Continue building authentic engagement",
            "Maintain consistent quality content"
        ])
    else:
        recommendations.append("Excellent human behavior patterns detected")
    
    return recommendations

def health_check() -> Dict[str, Any]:
    """System health check for AI services"""
    try:
        health_status = {
            'status': 'healthy',
            'timestamp': datetime.now().isoformat(),
            'services': {
                'redis': test_redis_connection(),
                'mongodb': test_mongodb_connection(),
                'ai_models': test_ai_models(),
                'performance': performance_monitor.get_performance_stats()
            }
        }
        
        # Check if any service is down
        services_status = [status for status in health_status['services'].values() 
                          if isinstance(status, dict) and status.get('status') == 'down']
        
        if services_status:
            health_status['status'] = 'degraded'
        
        return health_status
        
    except Exception as e:
        return {
            'status': 'error',
            'error': str(e),
            'timestamp': datetime.now().isoformat()
        }

def test_redis_connection() -> Dict[str, str]:
    """Test Redis connection"""
    try:
        redis_client.ping()
        return {'status': 'up', 'service': 'redis'}
    except Exception as e:
        return {'status': 'down', 'service': 'redis', 'error': str(e)}

def test_mongodb_connection() -> Dict[str, str]:
    """Test MongoDB connection"""
    try:
        mongo_client.admin.command('ping')
        return {'status': 'up', 'service': 'mongodb'}
    except Exception as e:
        return {'status': 'down', 'service': 'mongodb', 'error': str(e)}

def test_ai_models() -> Dict[str, str]:
    """Test AI models availability"""
    try:
        # Test basic text analysis
        test_text = "This is a test message for AI model validation."
        analyzer = TextAnalyzer()
        sentiment = analyzer.analyze_sentiment(test_text)
        
        if 'compound' in sentiment:
            return {'status': 'up', 'service': 'ai_models'}
        else:
            return {'status': 'degraded', 'service': 'ai_models', 'issue': 'partial_functionality'}
            
    except Exception as e:
        return {'status': 'down', 'service': 'ai_models', 'error': str(e)}

# Advanced Utility Functions

class EconomicCalculator:
    """Calculate economic impacts and token flow"""
    
    @staticmethod
    def calculate_mining_regression(user_holdings: float, total_network_size: int) -> float:
        """Calculate exponential regression factor for mining"""
        # Based on whitepaper formula: e^(-0.001 × User_Total_Holdings)
        regression_factor = math.exp(-0.001 * user_holdings)
        
        # Additional network size factor
        network_factor = max(0.1, 2.0 - (total_network_size / 1_000_000))
        
        return regression_factor * network_factor
    
    @staticmethod
    def calculate_network_bonus(active_referrals: int, referral_quality: float) -> float:
        """Calculate referral network bonus"""
        # Base bonus from active referrals
        base_bonus = 1 + (active_referrals * 0.1)
        
        # Quality multiplier
        quality_multiplier = 0.5 + (referral_quality * 0.5)
        
        # Cap at 3.5x total bonus
        return min(3.5, base_bonus * quality_multiplier)
    
    @staticmethod
    def predict_token_value_impact(
        daily_active_users: int,
        content_quality_avg: float,
        network_growth_rate: float
    ) -> Dict[str, float]:
        """Predict token value impact from platform metrics"""
        try:
            # Base value from user activity
            activity_value = daily_active_users * 0.001  # $0.001 per DAU
            
            # Quality premium
            quality_premium = content_quality_avg * 0.5
            
            # Network effect (Metcalfe's law approximation)
            network_value = math.sqrt(daily_active_users) * 0.01
            
            # Growth momentum
            growth_momentum = min(2.0, network_growth_rate * 10)
            
            predicted_impact = {
                'base_activity_value': activity_value,
                'quality_premium': quality_premium,
                'network_effect_value': network_value,
                'growth_momentum': growth_momentum,
                'total_predicted_impact': activity_value + quality_premium + network_value + growth_momentum
            }
            
            return predicted_impact
            
        except Exception as e:
            logger.error(f"Token value prediction error: {e}")
            return {'total_predicted_impact': 0.0}

class ContentModerationHelper:
    """Content moderation and compliance utilities"""
    
    PROHIBITED_CONTENT = {
        'violence': [
            'kill', 'murder', 'death', 'violence', 'harm', 'hurt', 'blood',
            'weapon', 'gun', 'knife', 'bomb', 'terrorist', 'attack'
        ],
        'hate_speech': [
            'hate', 'racism', 'nazi', 'supremacist', 'discrimination',
            'bigot', 'xenophobia', 'homophobia', 'transphobia'
        ],
        'adult_content': [
            'porn', 'sex', 'nude', 'naked', 'adult', 'xxx', 'erotic',
            'sexual', 'masturbate', 'orgasm', 'prostitute'
        ],
        'financial_scam': [
            'ponzi', 'pyramid', 'scam', 'fraud', 'fake', 'steal',
            'cheat', 'deceive', 'trick', 'con', 'ripoff'
        ],
        'drugs': [
            'cocaine', 'heroin', 'marijuana', 'weed', 'drug', 'dealer',
            'methamphetamine', 'ecstasy', 'lsd', 'cannabis'
        ]
    }
    
    @staticmethod
    def moderate_content(content: str) -> Dict[str, Any]:
        """Comprehensive content moderation"""
        try:
            content_lower = content.lower()
            violations = []
            severity_score = 0.0
            
            for category, keywords in ContentModerationHelper.PROHIBITED_CONTENT.items():
                category_violations = []
                
                for keyword in keywords:
                    if keyword in content_lower:
                        category_violations.append(keyword)
                        severity_score += 0.1
                
                if category_violations:
                    violations.append({
                        'category': category,
                        'keywords_found': category_violations,
                        'severity': len(category_violations) * 0.1
                    })
            
            # Calculate overall moderation score
            moderation_score = max(0.0, 1.0 - severity_score)
            
            # Determine action
            if severity_score >= 0.8:
                action = 'block'
            elif severity_score >= 0.4:
                action = 'review'
            elif severity_score >= 0.2:
                action = 'warn'
            else:
                action = 'approve'
            
            return {
                'moderation_score': moderation_score,
                'action': action,
                'violations': violations,
                'severity_score': severity_score,
                'safe_for_platform': moderation_score >= 0.6
            }
            
        except Exception as e:
            logger.error(f"Content moderation error: {e}")
            return {
                'moderation_score': 0.5,
                'action': 'review',
                'violations': [],
                'severity_score': 0.5,
                'safe_for_platform': False,
                'error': str(e)
            }

class AdvancedAnalytics:
    """Advanced analytics for business intelligence"""
    
    @staticmethod
    def calculate_user_lifetime_value(user_id: str) -> Dict[str, float]:
        """Calculate predicted user lifetime value"""
        try:
            # Get user data
            user_data = db.users.find_one({'user_id': user_id})
            if not user_data:
                return {'ltv': 0.0, 'confidence': 0.0}
            
            # Historical activity value
            activity_history = db.user_activities.find({'user_id': user_id}).sort('timestamp', -1)
            activities = list(activity_history.limit(1000))  # Last 1000 activities
            
            if not activities:
                return {'ltv': 0.0, 'confidence': 0.0}
            
            # Calculate metrics
            account_age_days = (datetime.now() - user_data.get('created_at', datetime.now())).days
            avg_daily_activity = len(activities) / max(1, account_age_days)
            
            # Content quality average
            quality_scores = [a.get('quality_score', 0.5) for a in activities if 'quality_score' in a]
            avg_quality = np.mean(quality_scores) if quality_scores else 0.5
            
            # Referral network value
            network_metrics = NetworkAnalyzer.calculate_network_quality_score(user_id)
            network_value = network_metrics['overall_network_score']
            
            # LTV Calculation
            # Base value from activity frequency
            activity_value = avg_daily_activity * 365 * 0.01  # $0.01 per activity annually
            
            # Quality premium
            quality_premium = avg_quality * activity_value * 2
            
            # Network effect value
            network_effect_value = network_value * activity_value * 3
            
            # Retention probability (based on engagement patterns)
            retention_prob = min(1.0, avg_daily_activity / 5 + avg_quality)
            
            # Final LTV
            ltv = (activity_value + quality_premium + network_effect_value) * retention_prob
            
            # Confidence based on data availability
            confidence = min(1.0, len(activities) / 100 + account_age_days / 30)
            
            return {
                'ltv': ltv,
                'confidence': confidence,
                'components': {
                    'activity_value': activity_value,
                    'quality_premium': quality_premium,
                    'network_effect_value': network_effect_value,
                    'retention_probability': retention_prob
                }
            }
            
        except Exception as e:
            logger.error(f"LTV calculation error: {e}")
            return {'ltv': 0.0, 'confidence': 0.0, 'error': str(e)}
    
    @staticmethod
    def analyze_platform_performance(platform: str, days: int = 30) -> Dict[str, Any]:
        """Analyze platform-specific performance metrics"""
        try:
            cutoff_date = datetime.now() - timedelta(days=days)
            
            # Get platform activities
            activities = db.user_activities.find({
                'platform': platform,
                'timestamp': {'$gte': cutoff_date}
            })
            
            activities_list = list(activities)
            
            if not activities_list:
                return {'platform': platform, 'performance_score': 0.0}
            
            # Calculate metrics
            total_activities = len(activities_list)
            unique_users = len(set(a['user_id'] for a in activities_list))
            
            # Quality metrics
            quality_scores = [a.get('quality_score', 0.5) for a in activities_list]
            avg_quality = np.mean(quality_scores)
            
            # Engagement metrics
            engagement_scores = [a.get('engagement_score', 0.5) for a in activities_list]
            avg_engagement = np.mean(engagement_scores)
            
            # Growth metrics
            daily_counts = defaultdict(int)
            for activity in activities_list:
                day = activity['timestamp'].date()
                daily_counts[day] += 1
            
            growth_trend = AdvancedAnalytics._calculate_growth_trend(daily_counts)
            
            # Platform performance score
            performance_score = (
                min(1.0, total_activities / 1000) * 0.3 +  # Activity volume
                avg_quality * 0.3 +                        # Content quality
                avg_engagement * 0.2 +                     # Engagement level
                min(1.0, unique_users / 500) * 0.1 +       # User diversity
                growth_trend * 0.1                         # Growth trend
            )
            
            return {
                'platform': platform,
                'performance_score': performance_score,
                'metrics': {
                    'total_activities': total_activities,
                    'unique_users': unique_users,
                    'avg_quality': avg_quality,
                    'avg_engagement': avg_engagement,
                    'growth_trend': growth_trend,
                    'activities_per_user': total_activities / unique_users if unique_users > 0 else 0
                }
            }
            
        except Exception as e:
            logger.error(f"Platform performance analysis error: {e}")
            return {'platform': platform, 'performance_score': 0.0, 'error': str(e)}
    
    @staticmethod
    def _calculate_growth_trend(daily_counts: Dict) -> float:
        """Calculate growth trend from daily activity counts"""
        try:
            if len(daily_counts) < 7:
                return 0.5
            
            # Convert to time series
            dates = sorted(daily_counts.keys())
            counts = [daily_counts[date] for date in dates]
            
            # Simple linear regression for trend
            n = len(counts)
            if n < 2:
                return 0.5
            
            x = list(range(n))
            xy = sum(x[i] * counts[i] for i in range(n))
            x_sum = sum(x)
            y_sum = sum(counts)
            x_sq_sum = sum(xi ** 2 for xi in x)
            
            if n * x_sq_sum - x_sum ** 2 != 0:
                slope = (n * xy - x_sum * y_sum) / (n * x_sq_sum - x_sum ** 2)
                # Normalize slope to 0-1 range
                normalized_slope = max(0.0, min(1.0, (slope + 10) / 20))
                return normalized_slope
            
            return 0.5
            
        except Exception:
            return 0.5

# Async utilities for high-performance processing
class AsyncProcessor:
    """Asynchronous processing utilities for scalability"""
    
    @staticmethod
    async def process_content_batch_async(content_batch: List[Dict]) -> List[Dict]:
        """Process content batch asynchronously"""
        semaphore = asyncio.Semaphore(10)  # Limit concurrent processing
        
        async def process_single_content(content_item):
            async with semaphore:
                try:
                    # Simulate async processing
                    await asyncio.sleep(0.1)  # Placeholder for actual AI processing
                    
                    result = analyze_content_comprehensive(
                        content=content_item.get('text', ''),
                        user_id=content_item.get('user_id', ''),
                        platform=content_item.get('platform', ''),
                        content_type=content_item.get('type', 'text')
                    )
                    
                    return {
                        'id': content_item.get('id'),
                        'processing_result': result,
                        'processed_at': datetime.now().isoformat()
                    }
                    
                except Exception as e:
                    logger.error(f"Async processing error for {content_item.get('id')}: {e}")
                    return {
                        'id': content_item.get('id'),
                        'error': str(e),
                        'processed_at': datetime.now().isoformat()
                    }
        
        # Process all content items concurrently
        tasks = [process_single_content(item) for item in content_batch]
        results = await asyncio.gather(*tasks, return_exceptions=True)
        
        # Filter out exceptions and return valid results
        valid_results = [r for r in results if not isinstance(r, Exception)]
        return valid_results

# Configuration and Constants
class AIConfig:
    """AI service configuration constants"""
    
    # Quality scoring thresholds
    QUALITY_THRESHOLDS = {
        'excellent': 0.9,
        'good': 0.7,
        'average': 0.5,
        'poor': 0.3,
        'very_poor': 0.1
    }
    
    # Anti-bot thresholds
    ANTI_BOT_THRESHOLDS = {
        'definitely_human': 0.9,
        'likely_human': 0.7,
        'uncertain': 0.5,
        'likely_bot': 0.3,
        'definitely_bot': 0.1
    }
    
    # Platform-specific configurations
    PLATFORM_LIMITS = {
        'instagram': {'max_hashtags': 30, 'max_length': 2200},
        'tiktok': {'max_hashtags': 10, 'max_length': 100},
        'youtube': {'max_tags': 15, 'max_description': 5000},
        'facebook': {'max_length': 63206},
        'twitter': {'max_length': 280, 'max_hashtags': 2}
    }
    
    # AI model configurations
    MODEL_CONFIGS = {
        'content_quality': {
            'threshold_excellent': 0.85,
            'threshold_good': 0.65,
            'threshold_poor': 0.35
        },
        'bot_detection': {
            'threshold_human': 0.75,
            'threshold_suspicious': 0.45,
            'threshold_bot': 0.25
        },
        'sentiment_analysis': {
            'positive_threshold': 0.1,
            'negative_threshold': -0.1,
            'neutral_range': (-0.1, 0.1)
        }
    }

# Export all utility classes and functions
__all__ = [
    'SecurityManager',
    'CacheManager', 
    'TextAnalyzer',
    'ImageAnalyzer',
    'VideoAnalyzer',
    'PlatformSpecificAnalyzer',
    'BehaviorAnalyzer',
    'QualityScorer',
    'NetworkAnalyzer',
    'RewardCalculator',
    'APIResponseManager',
    'PerformanceMonitor',
    'DataProcessor',
    'TrendAnalyzer',
    'EconomicCalculator',
    'ContentModerationHelper',
    'AdvancedAnalytics',
    'AsyncProcessor',
    'AIConfig',
    'ContentMetrics',
    'UserBehaviorProfile',
    'analyze_content_comprehensive',
    'generate_improvement_recommendations',
    'calculate_anti_bot_score',
    'generate_anti_bot_recommendations',
    'health_check',
    'test_redis_connection',
    'test_mongodb_connection',
    'test_ai_models'
]

# Initialize global components
def initialize_ai_helpers():
    """Initialize AI helper components"""
    try:
        logger.info("Initializing Finova AI Helper utilities...")
        
        # Test connections
        redis_status = test_redis_connection()
        mongo_status = test_mongodb_connection()
        ai_status = test_ai_models()
        
        logger.info(f"Redis: {redis_status['status']}")
        logger.info(f"MongoDB: {mongo_status['status']}")
        logger.info(f"AI Models: {ai_status['status']}")
        
        # Initialize collections indexes for performance
        db.user_activities.create_index([('user_id', 1), ('timestamp', -1)])
        db.content_analysis.create_index([('user_id', 1), ('analysis_timestamp', -1)])
        db.referral_networks.create_index([('user_id', 1)])
        db.user_content.create_index([('user_id', 1), ('content_hash', 1)])
        
        logger.info("Finova AI Helper utilities initialized successfully")
        return True
        
    except Exception as e:
        logger.error(f"Initialization error: {e}")
        return False

# Performance optimization utilities
def optimize_database_queries():
    """Optimize database queries for better performance"""
    try:
        # Create compound indexes for common query patterns
        db.user_activities.create_index([
            ('user_id', 1), 
            ('platform', 1), 
            ('timestamp', -1)
        ])
        
        db.content_analysis.create_index([
            ('user_id', 1),
            ('platform', 1),
            ('metrics.quality_score', -1)
        ])
        
        # Cleanup old data (older than 1 year)
        cutoff_date = datetime.now() - timedelta(days=365)
        
        db.user_activities.delete_many({'timestamp': {'$lt': cutoff_date}})
        db.content_analysis.delete_many({'analysis_timestamp': {'$lt': cutoff_date}})
        
        logger.info("Database optimization completed")
        return True
        
    except Exception as e:
        logger.error(f"Database optimization error: {e}")
        return False

# Error handling utilities
def handle_ai_service_error(error: Exception, context: str) -> Dict[str, Any]:
    """Standardized error handling for AI services"""
    error_response = {
        'error_type': type(error).__name__,
        'error_message': str(error),
        'context': context,
        'timestamp': datetime.now().isoformat(),
        'suggested_action': 'retry_with_backoff'
    }
    
    # Log error for monitoring
    logger.error(f"AI Service Error in {context}: {error}")
    
    # Store error for analysis
    try:
        db.error_logs.insert_one({
            **error_response,
            'stack_trace': str(error.__traceback__) if error.__traceback__ else None
        })
    except Exception:
        pass  # Don't let logging errors crash the service
    
    return error_response

# Main execution guard
if __name__ == "__main__":
    # Initialize the helper utilities
    success = initialize_ai_helpers()
    
    if success:
        print("✅ Finova AI Helper utilities ready!")
        print("🚀 All systems operational for enterprise-grade content analysis")
        
        # Run health check
        health = health_check()
        print(f"📊 System Health: {health['status']}")
        
        # Display performance stats
        stats = performance_monitor.get_performance_stats()
        print(f"⚡ Uptime: {stats.get('uptime_seconds', 0):.2f} seconds")
        
    else:
        print("❌ Failed to initialize AI Helper utilities")
        print("🔧 Please check configuration and dependencies")
        