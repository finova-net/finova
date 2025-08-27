"""
Finova Network Content Analyzer Models Module
===========================================

This module provides enterprise-grade AI models for content quality assessment,
originality detection, engagement prediction, and brand safety checking.

Author: Finova Network Development Team
Version: 3.0
License: MIT
"""

import logging
import asyncio
from typing import Dict, List, Optional, Union, Tuple, Any
from enum import Enum
from dataclasses import dataclass
from concurrent.futures import ThreadPoolExecutor
import hashlib
import time

# Core ML/AI imports
import numpy as np
import torch
import torch.nn as nn
from transformers import (
    AutoTokenizer, AutoModel, AutoModelForSequenceClassification,
    pipeline, BertTokenizer, BertForSequenceClassification
)
from sentence_transformers import SentenceTransformer
import cv2
import librosa
from PIL import Image
import torchvision.transforms as transforms

# Finova-specific imports
from .quality_classifier import QualityClassifier
from .originality_detector import OriginalityDetector
from .engagement_predictor import EngagementPredictor
from .brand_safety_checker import BrandSafetyChecker

# Configure logging
logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)

class ContentType(Enum):
    """Content type enumeration for model selection"""
    TEXT = "text"
    IMAGE = "image" 
    VIDEO = "video"
    AUDIO = "audio"
    MIXED = "mixed"

class Platform(Enum):
    """Social media platform enumeration"""
    INSTAGRAM = "instagram"
    TIKTOK = "tiktok"
    YOUTUBE = "youtube"
    FACEBOOK = "facebook"
    TWITTER = "twitter"
    LINKEDIN = "linkedin"

class QualityScore(Enum):
    """Content quality score levels"""
    POOR = 0.5
    BELOW_AVERAGE = 0.7
    AVERAGE = 1.0
    GOOD = 1.3
    EXCELLENT = 1.6
    EXCEPTIONAL = 2.0

@dataclass
class ContentAnalysisResult:
    """Standardized result format for all content analysis"""
    content_id: str
    platform: Platform
    content_type: ContentType
    quality_score: float
    originality_score: float
    engagement_prediction: float
    brand_safety_score: float
    human_generated_probability: float
    detailed_metrics: Dict[str, Any]
    processing_time: float
    model_version: str
    timestamp: float

class ModelManager:
    """Central manager for all AI models with caching and optimization"""
    
    def __init__(self):
        self._models = {}
        self._tokenizers = {}
        self._device = torch.device("cuda" if torch.cuda.is_available() else "cpu")
        self._executor = ThreadPoolExecutor(max_workers=4)
        self._cache = {}
        self._cache_ttl = 3600  # 1 hour cache
        
        logger.info(f"ModelManager initialized on device: {self._device}")
        
    async def initialize_models(self):
        """Initialize all required models asynchronously"""
        try:
            await asyncio.gather(
                self._load_quality_models(),
                self._load_originality_models(),
                self._load_engagement_models(),
                self._load_safety_models()
            )
            logger.info("All models initialized successfully")
        except Exception as e:
            logger.error(f"Model initialization failed: {e}")
            raise

    async def _load_quality_models(self):
        """Load quality assessment models"""
        # Text quality model
        self._models['text_quality'] = AutoModelForSequenceClassification.from_pretrained(
            'microsoft/DialoGPT-medium'
        ).to(self._device)
        self._tokenizers['text_quality'] = AutoTokenizer.from_pretrained(
            'microsoft/DialoGPT-medium'
        )
        
        # Image quality model (ResNet-based)
        self._models['image_quality'] = torch.hub.load(
            'pytorch/vision:v0.10.0', 'resnet50', pretrained=True
        ).to(self._device)
        
        # Video quality model
        self._models['video_quality'] = torch.hub.load(
            'facebookresearch/pytorchvideo', 'slowfast_r50', pretrained=True
        ).to(self._device)

    async def _load_originality_models(self):
        """Load originality detection models"""
        # Text similarity model
        self._models['text_similarity'] = SentenceTransformer(
            'all-MiniLM-L6-v2'
        )
        
        # Image similarity model
        self._models['image_similarity'] = torch.hub.load(
            'pytorch/vision:v0.10.0', 'mobilenet_v2', pretrained=True
        ).to(self._device)

    async def _load_engagement_models(self):
        """Load engagement prediction models"""
        # Multi-platform engagement predictor
        self._models['engagement_predictor'] = AutoModelForSequenceClassification.from_pretrained(
            'cardiffnlp/twitter-roberta-base-sentiment-latest'
        ).to(self._device)
        self._tokenizers['engagement_predictor'] = AutoTokenizer.from_pretrained(
            'cardiffnlp/twitter-roberta-base-sentiment-latest'
        )

    async def _load_safety_models(self):
        """Load brand safety models"""
        # Content moderation model
        self._models['safety_classifier'] = pipeline(
            "text-classification",
            model="unitary/toxic-bert",
            device=0 if torch.cuda.is_available() else -1
        )
        
        # NSFW image detection
        self._models['nsfw_detector'] = AutoModelForSequenceClassification.from_pretrained(
            'Falconsai/nsfw_image_detection'
        ).to(self._device)

    def get_model(self, model_name: str):
        """Get model by name with error handling"""
        if model_name not in self._models:
            raise ValueError(f"Model '{model_name}' not found")
        return self._models[model_name]

    def get_tokenizer(self, tokenizer_name: str):
        """Get tokenizer by name with error handling"""
        if tokenizer_name not in self._tokenizers:
            raise ValueError(f"Tokenizer '{tokenizer_name}' not found")
        return self._tokenizers[tokenizer_name]

class ContentAnalyzer:
    """Main content analyzer orchestrating all models"""
    
    def __init__(self):
        self.model_manager = ModelManager()
        self.quality_classifier = None
        self.originality_detector = None
        self.engagement_predictor = None
        self.brand_safety_checker = None
        
    async def initialize(self):
        """Initialize all components"""
        await self.model_manager.initialize_models()
        
        self.quality_classifier = QualityClassifier(self.model_manager)
        self.originality_detector = OriginalityDetector(self.model_manager)
        self.engagement_predictor = EngagementPredictor(self.model_manager)
        self.brand_safety_checker = BrandSafetyChecker(self.model_manager)
        
        logger.info("ContentAnalyzer initialized successfully")

    async def analyze_content(
        self, 
        content: Union[str, bytes, Image.Image],
        content_type: ContentType,
        platform: Platform,
        user_context: Optional[Dict] = None
    ) -> ContentAnalysisResult:
        """
        Comprehensive content analysis with XP/RP integration
        
        Args:
            content: Content to analyze (text, image, video, etc.)
            content_type: Type of content
            platform: Social media platform
            user_context: Additional user context for scoring
            
        Returns:
            ContentAnalysisResult with comprehensive metrics
        """
        start_time = time.time()
        content_id = self._generate_content_id(content)
        
        try:
            # Run all analyses in parallel
            quality_task = self._analyze_quality(content, content_type, platform)
            originality_task = self._analyze_originality(content, content_type)
            engagement_task = self._predict_engagement(content, content_type, platform)
            safety_task = self._check_brand_safety(content, content_type)
            human_task = self._detect_human_generated(content, content_type)
            
            # Await all results
            quality_score, originality_score, engagement_score, safety_score, human_prob = await asyncio.gather(
                quality_task, originality_task, engagement_task, safety_task, human_task
            )
            
            # Calculate final integrated score for Finova rewards
            final_score = self._calculate_finova_score(
                quality_score, originality_score, engagement_score, 
                safety_score, human_prob, platform, user_context
            )
            
            processing_time = time.time() - start_time
            
            result = ContentAnalysisResult(
                content_id=content_id,
                platform=platform,
                content_type=content_type,
                quality_score=final_score,
                originality_score=originality_score,
                engagement_prediction=engagement_score,
                brand_safety_score=safety_score,
                human_generated_probability=human_prob,
                detailed_metrics={
                    'raw_quality': quality_score,
                    'platform_bonus': self._get_platform_bonus(platform),
                    'content_type_bonus': self._get_content_type_bonus(content_type),
                    'user_level_bonus': user_context.get('xp_level', 1) if user_context else 1,
                    'network_quality_bonus': user_context.get('rp_tier', 1) if user_context else 1
                },
                processing_time=processing_time,
                model_version="3.0",
                timestamp=time.time()
            )
            
            logger.info(f"Content analysis completed in {processing_time:.2f}s")
            return result
            
        except Exception as e:
            logger.error(f"Content analysis failed: {e}")
            raise

    async def _analyze_quality(self, content, content_type: ContentType, platform: Platform) -> float:
        """Analyze content quality using appropriate model"""
        return await self.quality_classifier.analyze(content, content_type, platform)

    async def _analyze_originality(self, content, content_type: ContentType) -> float:
        """Check content originality"""
        return await self.originality_detector.check_originality(content, content_type)

    async def _predict_engagement(self, content, content_type: ContentType, platform: Platform) -> float:
        """Predict engagement potential"""
        return await self.engagement_predictor.predict(content, content_type, platform)

    async def _check_brand_safety(self, content, content_type: ContentType) -> float:
        """Check brand safety compliance"""
        return await self.brand_safety_checker.check_safety(content, content_type)

    async def _detect_human_generated(self, content, content_type: ContentType) -> float:
        """Detect if content is human-generated"""
        # Implement AI detection logic
        if content_type == ContentType.TEXT:
            # Check for AI writing patterns
            ai_indicators = [
                'as an ai', 'i am an artificial', 'i cannot', 'i don\'t have personal',
                'as a language model', 'i\'m sorry, but i can\'t'
            ]
            content_lower = str(content).lower()
            ai_score = sum(1 for indicator in ai_indicators if indicator in content_lower)
            human_probability = max(0.1, 1.0 - (ai_score * 0.2))
            return human_probability
        
        return 0.9  # Default high human probability for non-text content

    def _calculate_finova_score(
        self, 
        quality: float, 
        originality: float, 
        engagement: float,
        safety: float, 
        human_prob: float, 
        platform: Platform, 
        user_context: Optional[Dict]
    ) -> float:
        """
        Calculate final Finova score integrating XP/RP mechanics
        Formula: Base_Score × Platform_Bonus × User_Bonuses × Safety_Gate
        """
        # Base score from content metrics
        base_score = (quality * 0.4 + originality * 0.3 + engagement * 0.2 + human_prob * 0.1)
        
        # Platform-specific bonuses (from whitepaper)
        platform_bonuses = {
            Platform.TIKTOK: 1.3,
            Platform.INSTAGRAM: 1.2,
            Platform.YOUTUBE: 1.4,
            Platform.FACEBOOK: 1.1,
            Platform.TWITTER: 1.2,
            Platform.LINKEDIN: 1.0
        }
        
        platform_bonus = platform_bonuses.get(platform, 1.0)
        
        # User context bonuses
        user_level_bonus = 1.0
        rp_tier_bonus = 1.0
        
        if user_context:
            # XP level bonus (1.0x to 1.5x based on level)
            user_level = user_context.get('xp_level', 1)
            user_level_bonus = min(1.5, 1.0 + (user_level * 0.01))
            
            # RP tier bonus (1.0x to 1.3x based on referral tier)
            rp_tier = user_context.get('rp_tier', 1)
            rp_tier_bonus = min(1.3, 1.0 + (rp_tier * 0.05))
        
        # Safety gate (content must pass safety check)
        safety_gate = 1.0 if safety > 0.7 else 0.5
        
        # Final calculation
        final_score = base_score * platform_bonus * user_level_bonus * rp_tier_bonus * safety_gate
        
        # Clamp to valid range (0.5x to 2.0x as per whitepaper)
        return max(0.5, min(2.0, final_score))

    def _get_platform_bonus(self, platform: Platform) -> float:
        """Get platform-specific bonus multiplier"""
        bonuses = {
            Platform.TIKTOK: 1.3,
            Platform.INSTAGRAM: 1.2,
            Platform.YOUTUBE: 1.4,
            Platform.FACEBOOK: 1.1,
            Platform.TWITTER: 1.2,
            Platform.LINKEDIN: 1.0
        }
        return bonuses.get(platform, 1.0)

    def _get_content_type_bonus(self, content_type: ContentType) -> float:
        """Get content type bonus multiplier"""
        bonuses = {
            ContentType.VIDEO: 1.5,
            ContentType.IMAGE: 1.2,
            ContentType.TEXT: 1.0,
            ContentType.AUDIO: 1.1,
            ContentType.MIXED: 1.3
        }
        return bonuses.get(content_type, 1.0)

    def _generate_content_id(self, content) -> str:
        """Generate unique content ID for caching and tracking"""
        content_str = str(content)[:1000]  # Limit length for hashing
        return hashlib.md5(content_str.encode()).hexdigest()

# Utility functions for model integration
def preprocess_text(text: str) -> str:
    """Standardize text preprocessing across all models"""
    import re
    
    # Remove excessive whitespace
    text = re.sub(r'\s+', ' ', text)
    
    # Remove special characters but keep emoji
    text = re.sub(r'[^\w\s\U0001F600-\U0001F64F\U0001F300-\U0001F5FF\U0001F680-\U0001F6FF\U0001F1E0-\U0001F1FF]', '', text)
    
    # Trim and return
    return text.strip()

def preprocess_image(image: Image.Image) -> torch.Tensor:
    """Standardize image preprocessing for all vision models"""
    transform = transforms.Compose([
        transforms.Resize((224, 224)),
        transforms.ToTensor(),
        transforms.Normalize(mean=[0.485, 0.456, 0.406], std=[0.229, 0.224, 0.225])
    ])
    return transform(image).unsqueeze(0)

async def batch_analyze_content(
    analyzer: ContentAnalyzer,
    content_batch: List[Tuple[Any, ContentType, Platform]],
    batch_size: int = 32
) -> List[ContentAnalysisResult]:
    """Efficiently process multiple content items in batches"""
    results = []
    
    for i in range(0, len(content_batch), batch_size):
        batch = content_batch[i:i + batch_size]
        
        # Process batch in parallel
        tasks = [
            analyzer.analyze_content(content, content_type, platform)
            for content, content_type, platform in batch
        ]
        
        batch_results = await asyncio.gather(*tasks, return_exceptions=True)
        
        # Filter out exceptions and add successful results
        for result in batch_results:
            if not isinstance(result, Exception):
                results.append(result)
            else:
                logger.error(f"Batch processing error: {result}")
    
    return results

# Export all public classes and functions
__all__ = [
    'ContentAnalyzer',
    'ModelManager', 
    'ContentAnalysisResult',
    'ContentType',
    'Platform',
    'QualityScore',
    'QualityClassifier',
    'OriginalityDetector', 
    'EngagementPredictor',
    'BrandSafetyChecker',
    'preprocess_text',
    'preprocess_image',
    'batch_analyze_content'
]

# Version info
__version__ = "3.0.0"
__author__ = "Finova Network Development Team"
__email__ = "dev@finova.network"

# Model configuration for easy updates
MODEL_CONFIG = {
    'text_quality_model': 'microsoft/DialoGPT-medium',
    'image_quality_model': 'resnet50',
    'engagement_model': 'cardiffnlp/twitter-roberta-base-sentiment-latest',
    'safety_model': 'unitary/toxic-bert',
    'similarity_model': 'all-MiniLM-L6-v2',
    'nsfw_model': 'Falconsai/nsfw_image_detection'
}

# Quality thresholds for different content types
QUALITY_THRESHOLDS = {
    ContentType.TEXT: {
        'min_length': 10,
        'max_length': 5000,
        'min_quality_score': 0.6
    },
    ContentType.IMAGE: {
        'min_resolution': (256, 256),
        'max_file_size': 10 * 1024 * 1024,  # 10MB
        'min_quality_score': 0.7
    },
    ContentType.VIDEO: {
        'min_duration': 3,  # seconds
        'max_duration': 600,  # 10 minutes
        'min_quality_score': 0.8
    }
}

# Initialize global analyzer instance (singleton pattern)
_global_analyzer = None

async def get_analyzer() -> ContentAnalyzer:
    """Get global analyzer instance (singleton)"""
    global _global_analyzer
    if _global_analyzer is None:
        _global_analyzer = ContentAnalyzer()
        await _global_analyzer.initialize()
    return _global_analyzer

# Clean up resources on module unload
import atexit

def cleanup_models():
    """Clean up model resources"""
    global _global_analyzer
    if _global_analyzer:
        if hasattr(_global_analyzer.model_manager, '_executor'):
            _global_analyzer.model_manager._executor.shutdown(wait=True)
        logger.info("Content analyzer models cleaned up")

atexit.register(cleanup_models)
