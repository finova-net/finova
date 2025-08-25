"""
Finova Network - Brand Safety Checker
Advanced AI-powered content safety analysis for social media mining rewards

This module provides comprehensive brand safety analysis for user-generated content
across multiple social platforms, ensuring advertiser-friendly content classification.
"""

import logging
import re
import hashlib
import asyncio
from typing import Dict, List, Optional, Tuple, Any, Union
from dataclasses import dataclass, asdict
from enum import Enum
import numpy as np
from datetime import datetime, timedelta
import cv2
import torch
import torch.nn as nn
from transformers import (
    AutoTokenizer, AutoModelForSequenceClassification,
    pipeline, AutoProcessor, AutoModel
)
from PIL import Image
import aiohttp
import asyncpg
import redis.asyncio as redis
from pydantic import BaseModel, Field
import joblib
from sklearn.feature_extraction.text import TfidfVectorizer
from sklearn.linear_model import LogisticRegression

# Configure logging
logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)

class SafetyLevel(Enum):
    """Content safety levels for brand safety classification"""
    SAFE = "safe"
    MODERATE = "moderate" 
    UNSAFE = "unsafe"
    BLOCKED = "blocked"

class ContentType(Enum):
    """Types of content that can be analyzed"""
    TEXT = "text"
    IMAGE = "image"
    VIDEO = "video"
    AUDIO = "audio"
    MIXED = "mixed"

class SafetyCategory(Enum):
    """Brand safety risk categories"""
    HATE_SPEECH = "hate_speech"
    VIOLENCE = "violence"
    ADULT_CONTENT = "adult_content"
    DRUGS_ALCOHOL = "drugs_alcohol"
    GAMBLING = "gambling"
    POLITICAL = "political"
    MISINFORMATION = "misinformation"
    SPAM = "spam"
    COPYRIGHT = "copyright"
    PROFANITY = "profanity"
    HARASSMENT = "harassment"
    SELF_HARM = "self_harm"

@dataclass
class SafetyResult:
    """Result of brand safety analysis"""
    content_id: str
    safety_level: SafetyLevel
    confidence_score: float
    risk_categories: List[SafetyCategory]
    category_scores: Dict[str, float]
    platform_compatibility: Dict[str, bool]
    advertiser_friendly: bool
    monetization_eligible: bool
    explanation: str
    processing_time: float
    metadata: Dict[str, Any]

class BrandSafetyConfig(BaseModel):
    """Configuration for brand safety checker"""
    models_path: str = "/models/brand_safety"
    redis_url: str = "redis://localhost:6379"
    db_url: str = "postgresql://user:pass@localhost/finova"
    
    # Model configurations
    text_model: str = "unitary/toxic-bert"
    image_model: str = "microsoft/DiT-large"
    multimodal_model: str = "openai/clip-vit-base-patch32"
    
    # Safety thresholds
    safe_threshold: float = 0.8
    moderate_threshold: float = 0.5
    unsafe_threshold: float = 0.2
    
    # Platform-specific settings
    platform_rules: Dict[str, Dict[str, float]] = Field(default_factory=lambda: {
        "instagram": {"adult_content": 0.3, "violence": 0.2, "profanity": 0.4},
        "tiktok": {"adult_content": 0.2, "violence": 0.1, "profanity": 0.3},
        "youtube": {"adult_content": 0.4, "violence": 0.3, "profanity": 0.5},
        "facebook": {"adult_content": 0.3, "violence": 0.2, "profanity": 0.4},
        "twitter": {"adult_content": 0.5, "violence": 0.4, "profanity": 0.6}
    })
    
    # Performance settings
    cache_ttl: int = 3600
    batch_size: int = 32
    max_workers: int = 4

class BrandSafetyChecker:
    """
    Advanced brand safety checker for Finova Network content analysis
    
    Features:
    - Multi-modal content analysis (text, image, video)
    - Platform-specific safety rules
    - Real-time processing with caching
    - Explainable AI results
    - Integration with XP/RP/Mining systems
    """
    
    def __init__(self, config: BrandSafetyConfig):
        self.config = config
        self.redis_client = None
        self.db_pool = None
        
        # Initialize AI models
        self._load_models()
        
        # Pre-compiled regex patterns for fast text analysis
        self._compile_patterns()
        
        # Platform advertiser guidelines
        self._load_platform_guidelines()
        
        logger.info("BrandSafetyChecker initialized successfully")
    
    def _load_models(self):
        """Load and initialize AI models for safety analysis"""
        try:
            # Text safety model
            self.text_tokenizer = AutoTokenizer.from_pretrained(self.config.text_model)
            self.text_model = AutoModelForSequenceClassification.from_pretrained(
                self.config.text_model
            )
            
            # Image safety pipeline
            self.image_processor = AutoProcessor.from_pretrained(self.config.image_model)
            self.image_model = AutoModel.from_pretrained(self.config.image_model)
            
            # Multimodal CLIP model
            self.clip_processor = AutoProcessor.from_pretrained(self.config.multimodal_model)
            self.clip_model = AutoModel.from_pretrained(self.config.multimodal_model)
            
            # Custom trained models for specific categories
            self._load_custom_models()
            
            logger.info("All AI models loaded successfully")
            
        except Exception as e:
            logger.error(f"Error loading models: {e}")
            raise
    
    def _load_custom_models(self):
        """Load custom trained models for Finova-specific safety rules"""
        try:
            # Indonesian language specific model
            self.indonesian_safety_model = joblib.load(
                f"{self.config.models_path}/indonesian_safety_classifier.joblib"
            )
            
            # Crypto/Web3 context model
            self.crypto_safety_model = joblib.load(
                f"{self.config.models_path}/crypto_safety_classifier.joblib"
            )
            
            # Social mining fraud detection
            self.fraud_detection_model = joblib.load(
                f"{self.config.models_path}/fraud_detection_classifier.joblib"
            )
            
            logger.info("Custom models loaded successfully")
            
        except Exception as e:
            logger.warning(f"Custom models not found, using defaults: {e}")
            self._initialize_fallback_models()
    
    def _initialize_fallback_models(self):
        """Initialize simple fallback models when custom models unavailable"""
        # Simple TF-IDF + Logistic Regression fallback
        self.fallback_vectorizer = TfidfVectorizer(max_features=10000, stop_words='english')
        self.fallback_classifier = LogisticRegression()
        
        # Pre-trained with basic safety categories
        self._train_fallback_models()
    
    def _train_fallback_models(self):
        """Train basic fallback models with minimal data"""
        # Basic training data for fallback
        safe_samples = [
            "Great product review", "Beautiful sunset photo", "Educational content",
            "Travel experience sharing", "Cooking tutorial", "Fitness motivation"
        ]
        unsafe_samples = [
            "Hate speech content", "Violence promotion", "Adult content",
            "Drug promotion", "Gambling advertisement", "Spam content"
        ]
        
        X = safe_samples + unsafe_samples
        y = [1] * len(safe_samples) + [0] * len(unsafe_samples)
        
        X_vectors = self.fallback_vectorizer.fit_transform(X)
        self.fallback_classifier.fit(X_vectors, y)
    
    def _compile_patterns(self):
        """Compile regex patterns for fast text analysis"""
        self.patterns = {
            SafetyCategory.PROFANITY: re.compile(
                r'\b(fuck|shit|damn|bitch|asshole|crap)\b', re.IGNORECASE
            ),
            SafetyCategory.HATE_SPEECH: re.compile(
                r'\b(hate|kill|murder|nazi|terrorist)\b', re.IGNORECASE
            ),
            SafetyCategory.ADULT_CONTENT: re.compile(
                r'\b(sex|porn|nude|adult|xxx|nsfw)\b', re.IGNORECASE
            ),
            SafetyCategory.DRUGS_ALCOHOL: re.compile(
                r'\b(drugs|cocaine|marijuana|alcohol|drunk|weed)\b', re.IGNORECASE
            ),
            SafetyCategory.GAMBLING: re.compile(
                r'\b(casino|bet|gambling|poker|lottery|jackpot)\b', re.IGNORECASE
            ),
            SafetyCategory.SPAM: re.compile(
                r'\b(buy now|click here|free money|earn fast|get rich)\b', re.IGNORECASE
            )
        }
    
    def _load_platform_guidelines(self):
        """Load platform-specific advertiser guidelines"""
        self.platform_guidelines = {
            "instagram": {
                "max_profanity_score": 0.3,
                "adult_content_strict": True,
                "violence_tolerance": 0.2,
                "political_content": False
            },
            "tiktok": {
                "max_profanity_score": 0.2,
                "adult_content_strict": True,
                "violence_tolerance": 0.1,
                "political_content": False
            },
            "youtube": {
                "max_profanity_score": 0.5,
                "adult_content_strict": False,
                "violence_tolerance": 0.4,
                "political_content": True
            },
            "facebook": {
                "max_profanity_score": 0.3,
                "adult_content_strict": True,
                "violence_tolerance": 0.2,
                "political_content": True
            },
            "twitter": {
                "max_profanity_score": 0.6,
                "adult_content_strict": False,
                "violence_tolerance": 0.5,
                "political_content": True
            }
        }
    
    async def initialize_connections(self):
        """Initialize database and cache connections"""
        try:
            # Initialize Redis connection
            self.redis_client = redis.from_url(self.config.redis_url)
            await self.redis_client.ping()
            
            # Initialize PostgreSQL connection pool
            self.db_pool = await asyncpg.create_pool(self.config.db_url)
            
            logger.info("Database connections initialized")
            
        except Exception as e:
            logger.error(f"Failed to initialize connections: {e}")
            raise
    
    async def analyze_content(
        self,
        content: Union[str, bytes, Dict[str, Any]],
        content_type: ContentType,
        platform: str = "general",
        user_id: Optional[str] = None,
        context: Optional[Dict[str, Any]] = None
    ) -> SafetyResult:
        """
        Comprehensive brand safety analysis of content
        
        Args:
            content: Content to analyze (text, image bytes, or mixed)
            content_type: Type of content being analyzed
            platform: Target social media platform
            user_id: User ID for personalized analysis
            context: Additional context for analysis
            
        Returns:
            SafetyResult with comprehensive safety assessment
        """
        start_time = datetime.now()
        content_id = self._generate_content_id(content, content_type)
        
        try:
            # Check cache first
            cached_result = await self._get_cached_result(content_id)
            if cached_result:
                logger.info(f"Cache hit for content_id: {content_id}")
                return cached_result
            
            # Perform safety analysis based on content type
            if content_type == ContentType.TEXT:
                result = await self._analyze_text_content(content, platform, context)
            elif content_type == ContentType.IMAGE:
                result = await self._analyze_image_content(content, platform, context)
            elif content_type == ContentType.VIDEO:
                result = await self._analyze_video_content(content, platform, context)
            elif content_type == ContentType.MIXED:
                result = await self._analyze_mixed_content(content, platform, context)
            else:
                raise ValueError(f"Unsupported content type: {content_type}")
            
            # Add metadata
            processing_time = (datetime.now() - start_time).total_seconds()
            result.content_id = content_id
            result.processing_time = processing_time
            result.metadata = {
                "platform": platform,
                "user_id": user_id,
                "analysis_timestamp": datetime.now().isoformat(),
                "model_versions": self._get_model_versions(),
                "content_hash": self._hash_content(content)
            }
            
            # Cache result
            await self._cache_result(content_id, result)
            
            # Log result for monitoring
            await self._log_analysis_result(result, user_id)
            
            return result
            
        except Exception as e:
            logger.error(f"Error analyzing content {content_id}: {e}")
            # Return safe fallback result
            return self._create_fallback_result(content_id, str(e))
    
    async def _analyze_text_content(
        self,
        text: str,
        platform: str,
        context: Optional[Dict[str, Any]]
    ) -> SafetyResult:
        """Analyze text content for brand safety"""
        
        # Basic text preprocessing
        processed_text = self._preprocess_text(text)
        
        # Multi-model analysis
        safety_scores = {}
        
        # 1. Transformer-based toxic content detection
        toxic_scores = await self._analyze_text_toxicity(processed_text)
        safety_scores.update(toxic_scores)
        
        # 2. Pattern-based analysis
        pattern_scores = self._analyze_text_patterns(processed_text)
        safety_scores.update(pattern_scores)
        
        # 3. Custom model analysis (Indonesian, crypto-specific)
        custom_scores = await self._analyze_text_custom(processed_text, context)
        safety_scores.update(custom_scores)
        
        # 4. Platform-specific rules
        platform_scores = self._apply_platform_rules(safety_scores, platform)
        
        # Aggregate results
        overall_safety = self._calculate_overall_safety(platform_scores)
        
        return SafetyResult(
            content_id="",  # Will be set by caller
            safety_level=overall_safety["level"],
            confidence_score=overall_safety["confidence"],
            risk_categories=overall_safety["risk_categories"],
            category_scores=platform_scores,
            platform_compatibility=self._check_platform_compatibility(platform_scores),
            advertiser_friendly=overall_safety["advertiser_friendly"],
            monetization_eligible=overall_safety["monetization_eligible"],
            explanation=overall_safety["explanation"],
            processing_time=0.0,  # Will be set by caller
            metadata={}  # Will be set by caller
        )
    
    async def _analyze_text_toxicity(self, text: str) -> Dict[str, float]:
        """Use transformer model to detect toxic content"""
        try:
            inputs = self.text_tokenizer(
                text, 
                return_tensors="pt", 
                truncation=True, 
                padding=True, 
                max_length=512
            )
            
            with torch.no_grad():
                outputs = self.text_model(**inputs)
                predictions = torch.softmax(outputs.logits, dim=-1)
            
            # Convert to safety scores
            toxic_prob = predictions[0][1].item()  # Assuming binary classification
            
            return {
                "toxicity": toxic_prob,
                "hate_speech": toxic_prob * 0.8,  # Correlation-based estimation
                "harassment": toxic_prob * 0.6,
                "profanity": toxic_prob * 0.4
            }
            
        except Exception as e:
            logger.error(f"Error in toxicity analysis: {e}")
            return {"toxicity": 0.0, "hate_speech": 0.0, "harassment": 0.0, "profanity": 0.0}
    
    def _analyze_text_patterns(self, text: str) -> Dict[str, float]:
        """Pattern-based safety analysis for fast detection"""
        scores = {}
        
        for category, pattern in self.patterns.items():
            matches = pattern.findall(text)
            # Normalize score based on text length and match count
            score = min(len(matches) / max(len(text.split()) / 10, 1), 1.0)
            scores[category.value] = score
        
        return scores
    
    async def _analyze_text_custom(
        self, 
        text: str, 
        context: Optional[Dict[str, Any]]
    ) -> Dict[str, float]:
        """Custom model analysis for Indonesian and crypto content"""
        scores = {}
        
        try:
            # Indonesian-specific analysis
            if hasattr(self, 'indonesian_safety_model'):
                indo_features = self.fallback_vectorizer.transform([text])
                indo_score = self.indonesian_safety_model.predict_proba(indo_features)[0][0]
                scores["indonesian_safety"] = 1.0 - indo_score
            
            # Crypto/Web3 context analysis
            if hasattr(self, 'crypto_safety_model') and context and context.get("crypto_related"):
                crypto_features = self.fallback_vectorizer.transform([text])
                crypto_score = self.crypto_safety_model.predict_proba(crypto_features)[0][0]
                scores["crypto_safety"] = 1.0 - crypto_score
            
            # Fraud detection for mining-related content
            if hasattr(self, 'fraud_detection_model'):
                fraud_features = self.fallback_vectorizer.transform([text])
                fraud_score = self.fraud_detection_model.predict_proba(fraud_features)[0][0]
                scores["fraud_risk"] = fraud_score
            
        except Exception as e:
            logger.error(f"Error in custom analysis: {e}")
        
        return scores
    
    async def _analyze_image_content(
        self,
        image_data: bytes,
        platform: str,
        context: Optional[Dict[str, Any]]
    ) -> SafetyResult:
        """Analyze image content for brand safety"""
        
        try:
            # Convert bytes to PIL Image
            image = Image.open(io.BytesIO(image_data))
            
            # Multi-model image analysis
            safety_scores = {}
            
            # 1. NSFW detection
            nsfw_scores = await self._analyze_image_nsfw(image)
            safety_scores.update(nsfw_scores)
            
            # 2. Violence detection
            violence_scores = await self._analyze_image_violence(image)
            safety_scores.update(violence_scores)
            
            # 3. Brand logo detection
            brand_scores = await self._analyze_image_brands(image)
            safety_scores.update(brand_scores)
            
            # 4. Text extraction and analysis
            if text_in_image := self._extract_text_from_image(image):
                text_scores = await self._analyze_text_content(text_in_image, platform, context)
                safety_scores.update(text_scores.category_scores)
            
            # Apply platform rules
            platform_scores = self._apply_platform_rules(safety_scores, platform)
            overall_safety = self._calculate_overall_safety(platform_scores)
            
            return SafetyResult(
                content_id="",
                safety_level=overall_safety["level"],
                confidence_score=overall_safety["confidence"],
                risk_categories=overall_safety["risk_categories"],
                category_scores=platform_scores,
                platform_compatibility=self._check_platform_compatibility(platform_scores),
                advertiser_friendly=overall_safety["advertiser_friendly"],
                monetization_eligible=overall_safety["monetization_eligible"],
                explanation=overall_safety["explanation"],
                processing_time=0.0,
                metadata={}
            )
            
        except Exception as e:
            logger.error(f"Error analyzing image: {e}")
            return self._create_fallback_result("", str(e))
    
    def _apply_platform_rules(
        self, 
        safety_scores: Dict[str, float], 
        platform: str
    ) -> Dict[str, float]:
        """Apply platform-specific safety rules"""
        
        if platform not in self.platform_guidelines:
            return safety_scores
        
        guidelines = self.platform_guidelines[platform]
        adjusted_scores = safety_scores.copy()
        
        # Apply platform-specific thresholds
        for category, score in safety_scores.items():
            if category in guidelines:
                threshold = guidelines[category]
                # Amplify scores that exceed platform thresholds
                if score > threshold:
                    adjusted_scores[category] = min(score * 1.5, 1.0)
        
        return adjusted_scores
    
    def _calculate_overall_safety(self, scores: Dict[str, float]) -> Dict[str, Any]:
        """Calculate overall safety assessment from category scores"""
        
        # Weighted aggregation of safety scores
        weights = {
            "hate_speech": 3.0,
            "violence": 2.5,
            "adult_content": 2.0,
            "harassment": 2.0,
            "self_harm": 3.0,
            "drugs_alcohol": 1.5,
            "gambling": 1.5,
            "profanity": 1.0,
            "spam": 1.0,
            "misinformation": 2.0
        }
        
        weighted_score = 0.0
        total_weight = 0.0
        risk_categories = []
        
        for category, score in scores.items():
            weight = weights.get(category, 1.0)
            weighted_score += score * weight
            total_weight += weight
            
            # Flag high-risk categories
            if score > self.config.moderate_threshold:
                try:
                    risk_categories.append(SafetyCategory(category))
                except ValueError:
                    # Skip invalid categories
                    pass
        
        overall_score = weighted_score / max(total_weight, 1.0) if total_weight > 0 else 0.0
        
        # Determine safety level
        if overall_score >= self.config.safe_threshold:
            level = SafetyLevel.SAFE
        elif overall_score >= self.config.moderate_threshold:
            level = SafetyLevel.MODERATE
        elif overall_score >= self.config.unsafe_threshold:
            level = SafetyLevel.UNSAFE
        else:
            level = SafetyLevel.BLOCKED
        
        # Calculate confidence based on score consistency
        score_variance = np.var(list(scores.values())) if scores else 0
        confidence = max(0.5, 1.0 - score_variance)
        
        # Advertiser and monetization eligibility
        advertiser_friendly = level in [SafetyLevel.SAFE, SafetyLevel.MODERATE]
        monetization_eligible = level == SafetyLevel.SAFE and overall_score >= 0.9
        
        # Generate explanation
        explanation = self._generate_explanation(level, risk_categories, overall_score)
        
        return {
            "level": level,
            "confidence": confidence,
            "risk_categories": risk_categories,
            "advertiser_friendly": advertiser_friendly,
            "monetization_eligible": monetization_eligible,
            "explanation": explanation
        }
    
    def _generate_explanation(
        self, 
        level: SafetyLevel, 
        risk_categories: List[SafetyCategory], 
        score: float
    ) -> str:
        """Generate human-readable explanation of safety assessment"""
        
        if level == SafetyLevel.SAFE:
            return f"Content is brand-safe with high confidence (score: {score:.2f}). Suitable for all advertisers and monetization."
        
        elif level == SafetyLevel.MODERATE:
            risks = ", ".join([cat.value.replace("_", " ") for cat in risk_categories[:3]])
            return f"Content has moderate risk (score: {score:.2f}). Detected risks: {risks}. Limited advertiser appeal."
        
        elif level == SafetyLevel.UNSAFE:
            risks = ", ".join([cat.value.replace("_", " ") for cat in risk_categories[:3]])
            return f"Content is unsafe for brands (score: {score:.2f}). Major risks: {risks}. Not suitable for advertising."
        
        else:  # BLOCKED
            risks = ", ".join([cat.value.replace("_", " ") for cat in risk_categories[:3]])
            return f"Content violates safety policies (score: {score:.2f}). Violations: {risks}. Blocked from monetization."
    
    def _check_platform_compatibility(self, scores: Dict[str, float]) -> Dict[str, bool]:
        """Check compatibility with different social media platforms"""
        compatibility = {}
        
        for platform, guidelines in self.platform_guidelines.items():
            is_compatible = True
            
            for category, threshold in guidelines.items():
                if category in scores and scores[category] > threshold:
                    is_compatible = False
                    break
            
            compatibility[platform] = is_compatible
        
        return compatibility
    
    async def _get_cached_result(self, content_id: str) -> Optional[SafetyResult]:
        """Retrieve cached safety result"""
        try:
            if not self.redis_client:
                return None
            
            cached_data = await self.redis_client.get(f"safety:{content_id}")
            if cached_data:
                data = json.loads(cached_data)
                return SafetyResult(**data)
            
        except Exception as e:
            logger.error(f"Cache retrieval error: {e}")
        
        return None
    
    async def _cache_result(self, content_id: str, result: SafetyResult):
        """Cache safety analysis result"""
        try:
            if not self.redis_client:
                return
            
            cache_data = asdict(result)
            # Convert enums to strings for JSON serialization
            cache_data["safety_level"] = result.safety_level.value
            cache_data["risk_categories"] = [cat.value for cat in result.risk_categories]
            
            await self.redis_client.setex(
                f"safety:{content_id}",
                self.config.cache_ttl,
                json.dumps(cache_data, default=str)
            )
            
        except Exception as e:
            logger.error(f"Cache storage error: {e}")
    
    def _generate_content_id(self, content: Any, content_type: ContentType) -> str:
        """Generate unique content ID for caching"""
        if isinstance(content, str):
            content_hash = hashlib.sha256(content.encode()).hexdigest()
        elif isinstance(content, bytes):
            content_hash = hashlib.sha256(content).hexdigest()
        else:
            content_hash = hashlib.sha256(str(content).encode()).hexdigest()
        
        return f"{content_type.value}_{content_hash[:16]}"
    
    def _hash_content(self, content: Any) -> str:
        """Generate content hash for integrity checking"""
        if isinstance(content, str):
            return hashlib.sha256(content.encode()).hexdigest()
        elif isinstance(content, bytes):
            return hashlib.sha256(content).hexdigest()
        else:
            return hashlib.sha256(str(content).encode()).hexdigest()
    
    def _preprocess_text(self, text: str) -> str:
        """Preprocess text for analysis"""
        # Basic cleaning
        text = re.sub(r'http[s]?://(?:[a-zA-Z]|[0-9]|[$-_@.&+]|[!*\\(\\),]|(?:%[0-9a-fA-F][0-9a-fA-F]))+', '', text)
        text = re.sub(r'@\w+', '', text)  # Remove mentions
        text = re.sub(r'#\w+', '', text)  # Remove hashtags
        text = re.sub(r'\s+', ' ', text).strip()  # Normalize whitespace
        
        return text
    
    def _get_model_versions(self) -> Dict[str, str]:
        """Get current model versions for metadata"""
        return {
            "text_model": self.config.text_model,
            "image_model": self.config.image_model,
            "multimodal_model": self.config.multimodal_model,
            "version": "1.0.0"
        }
    
    def _create_fallback_result(self, content_id: str, error: str) -> SafetyResult:
        """Create safe fallback result when analysis fails"""
        return SafetyResult(
            content_id=content_id,
            safety_level=SafetyLevel.MODERATE,
            confidence_score=0.5,
            risk_categories=[],
            category_scores={},
            platform_compatibility={platform: False for platform in self.platform_guidelines.keys()},
            advertiser_friendly=False,
            monetization_eligible=False,
            explanation=f"Analysis failed: {error}. Marked as moderate risk for safety.",
            processing_time=0.0,
            metadata={"error": error, "fallback": True}
        )
    
    async def _log_analysis_result(self, result: SafetyResult, user_id: Optional[str]):
        """Log analysis result for monitoring and improvement"""
        try:
            if not self.db_pool:
                return
            
            async with self.db_pool.acquire() as conn:
                await conn.execute("""
                    INSERT INTO content_safety_logs 
                    (content_id, user_id, safety_level, confidence_score, 
                     risk_categories, processing_time, created_at)
                    VALUES ($1, $2, $3, $4, $5, $6, $7)
                """, 
                result.content_id,
                user_id,
                result.safety_level.value,
                result.confidence_score,
                [cat.value for cat in result.risk_categories],
                result.processing_time,
                datetime.now()
                )
        except Exception as e:
            logger.error(f"Failed to log analysis result: {e}")
    
    async def get_user_safety_stats(self, user_id: str) -> Dict[str, Any]:
        """Get user's content safety statistics"""
        try:
            if not self.db_pool:
                return {}
            
            async with self.db_pool.acquire() as conn:
                stats = await conn.fetchrow("""
                    SELECT 
                        COUNT(*) as total_content,
                        AVG(confidence_score) as avg_confidence,
                        COUNT(CASE WHEN safety_level = 'safe' THEN 1 END) as safe_content,
                        COUNT(CASE WHEN safety_level = 'moderate' THEN 1 END) as moderate_content,
                        COUNT(CASE WHEN safety_level = 'unsafe' THEN 1 END) as unsafe_content,
                        COUNT(CASE WHEN safety_level = 'blocked' THEN 1 END) as blocked_content
                    FROM content_safety_logs 
                    WHERE user_id = $1 AND created_at > NOW() - INTERVAL '30 days'
                """, user_id)
                
                return {
                    "total_content": stats["total_content"],
                    "avg_confidence": float(stats["avg_confidence"] or 0),
                    "safety_distribution": {
                        "safe": stats["safe_content"],
                        "moderate": stats["moderate_content"], 
                        "unsafe": stats["unsafe_content"],
                        "blocked": stats["blocked_content"]
                    },
                    "safety_score": (stats["safe_content"] * 100 / max(stats["total_content"], 1))
                }
        except Exception as e:
            logger.error(f"Failed to get user safety stats: {e}")
            return {}
    
    async def analyze_image_nsfw(self, image: Image.Image) -> Dict[str, float]:
        """Analyze image for NSFW content"""
        try:
            # Preprocess image
            inputs = self.image_processor(images=image, return_tensors="pt")
            
            with torch.no_grad():
                outputs = self.image_model(**inputs)
                # Extract features for NSFW classification
                features = outputs.last_hidden_state.mean(dim=1)
                
            # Simple NSFW classification (placeholder - replace with actual model)
            nsfw_score = torch.sigmoid(features).mean().item()
            
            return {
                "adult_content": nsfw_score,
                "suggestive": nsfw_score * 0.7,
                "nudity": nsfw_score * 0.8
            }
            
        except Exception as e:
            logger.error(f"NSFW analysis error: {e}")
            return {"adult_content": 0.0, "suggestive": 0.0, "nudity": 0.0}
    
    async def analyze_image_violence(self, image: Image.Image) -> Dict[str, float]:
        """Analyze image for violent content"""
        try:
            # Convert to OpenCV format
            cv_image = cv2.cvtColor(np.array(image), cv2.COLOR_RGB2BGR)
            
            # Simple violence detection using edge/color analysis
            gray = cv2.cvtColor(cv_image, cv2.COLOR_BGR2GRAY)
            edges = cv2.Canny(gray, 50, 150)
            edge_density = np.sum(edges > 0) / edges.size
            
            # Red color detection (blood indicator)
            hsv = cv2.cvtColor(cv_image, cv2.COLOR_BGR2HSV)
            red_mask = cv2.inRange(hsv, (0, 50, 50), (10, 255, 255))
            red_ratio = np.sum(red_mask > 0) / red_mask.size
            
            violence_score = min((edge_density * 2 + red_ratio * 3) / 5, 1.0)
            
            return {
                "violence": violence_score,
                "weapons": violence_score * 0.6,
                "blood": red_ratio
            }
            
        except Exception as e:
            logger.error(f"Violence analysis error: {e}")
            return {"violence": 0.0, "weapons": 0.0, "blood": 0.0}
    
    async def analyze_image_brands(self, image: Image.Image) -> Dict[str, float]:
        """Analyze image for brand logos and copyright content"""
        try:
            # CLIP-based brand detection
            brand_queries = [
                "corporate logo", "brand symbol", "trademark", 
                "copyrighted material", "product placement"
            ]
            
            inputs = self.clip_processor(
                text=brand_queries, 
                images=image, 
                return_tensors="pt", 
                padding=True
            )
            
            with torch.no_grad():
                outputs = self.clip_model(**inputs)
                logits_per_image = outputs.logits_per_image
                probs = logits_per_image.softmax(dim=-1)
            
            brand_score = probs[0].max().item()
            
            return {
                "copyright": brand_score,
                "trademark": brand_score * 0.8,
                "brand_content": brand_score * 0.9
            }
            
        except Exception as e:
            logger.error(f"Brand analysis error: {e}")
            return {"copyright": 0.0, "trademark": 0.0, "brand_content": 0.0}
    
    def extract_text_from_image(self, image: Image.Image) -> str:
        """Extract text from image using OCR"""
        try:
            import pytesseract
            text = pytesseract.image_to_string(image)
            return text.strip()
        except Exception as e:
            logger.error(f"Text extraction error: {e}")
            return ""
    
    async def analyze_video_content(
        self,
        video_data: bytes,
        platform: str,
        context: Optional[Dict[str, Any]]
    ) -> SafetyResult:
        """Analyze video content for brand safety"""
        try:
            # Extract frames for analysis
            frames = self._extract_video_frames(video_data)
            
            # Analyze each frame
            frame_results = []
            for frame in frames[:10]:  # Limit to 10 frames for performance
                frame_result = await self._analyze_image_content(
                    self._image_to_bytes(frame), platform, context
                )
                frame_results.append(frame_result)
            
            # Extract audio for analysis
            audio_text = self._extract_audio_text(video_data)
            if audio_text:
                audio_result = await self._analyze_text_content(audio_text, platform, context)
                frame_results.append(audio_result)
            
            # Aggregate results
            aggregated_scores = self._aggregate_video_results(frame_results)
            overall_safety = self._calculate_overall_safety(aggregated_scores)
            
            return SafetyResult(
                content_id="",
                safety_level=overall_safety["level"],
                confidence_score=overall_safety["confidence"],
                risk_categories=overall_safety["risk_categories"],
                category_scores=aggregated_scores,
                platform_compatibility=self._check_platform_compatibility(aggregated_scores),
                advertiser_friendly=overall_safety["advertiser_friendly"],
                monetization_eligible=overall_safety["monetization_eligible"],
                explanation=overall_safety["explanation"],
                processing_time=0.0,
                metadata={"frame_count": len(frames), "has_audio": bool(audio_text)}
            )
            
        except Exception as e:
            logger.error(f"Video analysis error: {e}")
            return self._create_fallback_result("", str(e))
    
    def _extract_video_frames(self, video_data: bytes) -> List[Image.Image]:
        """Extract frames from video for analysis"""
        try:
            import tempfile
            import os
            
            # Save video to temporary file
            with tempfile.NamedTemporaryFile(suffix='.mp4', delete=False) as temp_file:
                temp_file.write(video_data)
                temp_path = temp_file.name
            
            cap = cv2.VideoCapture(temp_path)
            frames = []
            frame_count = 0
            
            while cap.isOpened() and frame_count < 30:  # Max 30 frames
                ret, frame = cap.read()
                if not ret:
                    break
                
                # Convert frame to PIL Image
                rgb_frame = cv2.cvtColor(frame, cv2.COLOR_BGR2RGB)
                pil_frame = Image.fromarray(rgb_frame)
                frames.append(pil_frame)
                
                # Skip frames for performance
                for _ in range(10):
                    cap.read()
                
                frame_count += 1
            
            cap.release()
            os.unlink(temp_path)
            
            return frames
            
        except Exception as e:
            logger.error(f"Frame extraction error: {e}")
            return []
    
    def _extract_audio_text(self, video_data: bytes) -> str:
        """Extract and transcribe audio from video"""
        try:
            # Placeholder for audio transcription
            # In production, use services like OpenAI Whisper or Google Speech-to-Text
            return ""
        except Exception as e:
            logger.error(f"Audio extraction error: {e}")
            return ""
    
    def _image_to_bytes(self, image: Image.Image) -> bytes:
        """Convert PIL Image to bytes"""
        import io
        buffer = io.BytesIO()
        image.save(buffer, format='JPEG')
        return buffer.getvalue()
    
    def _aggregate_video_results(self, results: List[SafetyResult]) -> Dict[str, float]:
        """Aggregate safety results from multiple video frames"""
        if not results:
            return {}
        
        # Collect all category scores
        all_scores = {}
        for result in results:
            for category, score in result.category_scores.items():
                if category not in all_scores:
                    all_scores[category] = []
                all_scores[category].append(score)
        
        # Calculate aggregated scores (max for safety, mean for others)
        aggregated = {}
        safety_categories = ["hate_speech", "violence", "adult_content", "harassment"]
        
        for category, scores in all_scores.items():
            if category in safety_categories:
                # Use max for safety-critical categories
                aggregated[category] = max(scores)
            else:
                # Use mean for other categories
                aggregated[category] = sum(scores) / len(scores)
        
        return aggregated
    
    async def analyze_mixed_content(
        self,
        content: Dict[str, Any],
        platform: str,
        context: Optional[Dict[str, Any]]
    ) -> SafetyResult:
        """Analyze mixed content (text + images + video)"""
        try:
            results = []
            
            # Analyze text component
            if "text" in content:
                text_result = await self._analyze_text_content(
                    content["text"], platform, context
                )
                results.append(text_result)
            
            # Analyze image components
            if "images" in content:
                for image_data in content["images"]:
                    image_result = await self._analyze_image_content(
                        image_data, platform, context
                    )
                    results.append(image_result)
            
            # Analyze video components
            if "videos" in content:
                for video_data in content["videos"]:
                    video_result = await self._analyze_video_content(
                        video_data, platform, context
                    )
                    results.append(video_result)
            
            # Aggregate all results
            aggregated_scores = {}
            for result in results:
                for category, score in result.category_scores.items():
                    if category not in aggregated_scores:
                        aggregated_scores[category] = []
                    aggregated_scores[category].append(score)
            
            # Calculate final scores
            final_scores = {}
            for category, scores in aggregated_scores.items():
                final_scores[category] = max(scores)  # Use worst case
            
            overall_safety = self._calculate_overall_safety(final_scores)
            
            return SafetyResult(
                content_id="",
                safety_level=overall_safety["level"],
                confidence_score=overall_safety["confidence"],
                risk_categories=overall_safety["risk_categories"],
                category_scores=final_scores,
                platform_compatibility=self._check_platform_compatibility(final_scores),
                advertiser_friendly=overall_safety["advertiser_friendly"],
                monetization_eligible=overall_safety["monetization_eligible"],
                explanation=overall_safety["explanation"],
                processing_time=0.0,
                metadata={"component_count": len(results)}
            )
            
        except Exception as e:
            logger.error(f"Mixed content analysis error: {e}")
            return self._create_fallback_result("", str(e))
    
    async def batch_analyze(
        self,
        content_list: List[Dict[str, Any]],
        platform: str = "general"
    ) -> List[SafetyResult]:
        """Batch analyze multiple content items"""
        semaphore = asyncio.Semaphore(self.config.max_workers)
        
        async def analyze_single(content_item):
            async with semaphore:
                return await self.analyze_content(
                    content_item["content"],
                    ContentType(content_item["type"]),
                    platform,
                    content_item.get("user_id"),
                    content_item.get("context")
                )
        
        tasks = [analyze_single(item) for item in content_list]
        results = await asyncio.gather(*tasks, return_exceptions=True)
        
        # Handle exceptions
        processed_results = []
        for i, result in enumerate(results):
            if isinstance(result, Exception):
                processed_results.append(
                    self._create_fallback_result(f"batch_{i}", str(result))
                )
            else:
                processed_results.append(result)
        
        return processed_results
    
    async def update_user_reputation(self, user_id: str, safety_result: SafetyResult):
        """Update user reputation based on content safety"""
        try:
            if not self.db_pool:
                return
            
            # Calculate reputation impact
            reputation_delta = self._calculate_reputation_delta(safety_result)
            
            async with self.db_pool.acquire() as conn:
                await conn.execute("""
                    INSERT INTO user_safety_reputation (user_id, reputation_score, last_updated)
                    VALUES ($1, $2, $3)
                    ON CONFLICT (user_id) 
                    DO UPDATE SET 
                        reputation_score = user_safety_reputation.reputation_score + $2,
                        last_updated = $3
                """, user_id, reputation_delta, datetime.now())
                
        except Exception as e:
            logger.error(f"Failed to update user reputation: {e}")
    
    def _calculate_reputation_delta(self, result: SafetyResult) -> float:
        """Calculate reputation change based on safety result"""
        base_delta = {
            SafetyLevel.SAFE: 1.0,
            SafetyLevel.MODERATE: 0.0,
            SafetyLevel.UNSAFE: -2.0,
            SafetyLevel.BLOCKED: -5.0
        }
        
        delta = base_delta[result.safety_level]
        
        # Apply confidence multiplier
        delta *= result.confidence_score
        
        # Apply quality bonus/penalty
        if result.monetization_eligible:
            delta += 0.5
        
        return delta
    
    async def close(self):
        """Clean up resources"""
        try:
            if self.redis_client:
                await self.redis_client.close()
            
            if self.db_pool:
                await self.db_pool.close()
                
            logger.info("BrandSafetyChecker resources cleaned up")
            
        except Exception as e:
            logger.error(f"Error during cleanup: {e}")

# Factory function for easy initialization
async def create_brand_safety_checker(config: Optional[BrandSafetyConfig] = None) -> BrandSafetyChecker:
    """Factory function to create and initialize BrandSafetyChecker"""
    if config is None:
        config = BrandSafetyConfig()
    
    checker = BrandSafetyChecker(config)
    await checker.initialize_connections()
    return checker

# Example usage and integration
if __name__ == "__main__":
    import asyncio
    import json
    
    async def main():
        # Initialize the brand safety checker
        config = BrandSafetyConfig(
            redis_url="redis://localhost:6379",
            db_url="postgresql://user:pass@localhost/finova"
        )
        
        checker = await create_brand_safety_checker(config)
        
        # Example text analysis
        text_content = "This is a great product! I love using it every day. #finova #mining"
        result = await checker.analyze_content(
            content=text_content,
            content_type=ContentType.TEXT,
            platform="instagram",
            user_id="user123",
            context={"crypto_related": True}
        )
        
        print(f"Safety Level: {result.safety_level.value}")
        print(f"Confidence: {result.confidence_score:.2f}")
        print(f"Advertiser Friendly: {result.advertiser_friendly}")
        print(f"Monetization Eligible: {result.monetization_eligible}")
        print(f"Explanation: {result.explanation}")
        
        # Get user safety stats
        stats = await checker.get_user_safety_stats("user123")
        print(f"User Safety Stats: {json.dumps(stats, indent=2)}")
        
        # Clean up
        await checker.close()
    
    # Run the example
    asyncio.run(main())
    