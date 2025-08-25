"""
Finova Network - Content Quality Classifier
AI-powered content analysis for XP multiplier calculation and anti-bot detection
"""

import asyncio
import hashlib
import json
import logging
import numpy as np
import re
import time
from datetime import datetime, timedelta
from dataclasses import dataclass
from typing import Dict, List, Optional, Tuple, Union
from enum import Enum

import torch
import torch.nn as nn
from transformers import (
    AutoTokenizer, AutoModel, pipeline,
    BertTokenizer, BertModel
)
from sklearn.feature_extraction.text import TfidfVectorizer
from sklearn.metrics.pairwise import cosine_similarity
import cv2
from PIL import Image
import imagehash
import spacy

# Configure logging
logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)

class ContentType(Enum):
    TEXT = "text"
    IMAGE = "image"
    VIDEO = "video"
    MIXED = "mixed"

class Platform(Enum):
    INSTAGRAM = "instagram"
    TIKTOK = "tiktok"
    YOUTUBE = "youtube"
    FACEBOOK = "facebook"
    TWITTER_X = "twitter_x"
    FINOVA_APP = "finova_app"

@dataclass
class QualityScore:
    originality: float  # 0.0 - 1.0
    engagement_potential: float  # 0.0 - 1.0
    platform_relevance: float  # 0.0 - 1.0
    brand_safety: float  # 0.0 - 1.0
    human_generated: float  # 0.0 - 1.0
    technical_quality: float  # 0.0 - 1.0
    overall_score: float  # Final multiplier: 0.5x - 2.0x

@dataclass
class ContentMetadata:
    content_id: str
    user_id: str
    platform: Platform
    content_type: ContentType
    timestamp: datetime
    text_content: Optional[str] = None
    image_urls: Optional[List[str]] = None
    video_url: Optional[str] = None
    hashtags: Optional[List[str]] = None
    mentions: Optional[List[str]] = None
    engagement_data: Optional[Dict] = None

class FinovaQualityClassifier:
    """
    Enterprise-grade content quality classifier for Finova Network
    Implements multi-modal analysis with anti-gaming mechanisms
    """
    
    def __init__(self, config_path: str = None):
        self.config = self._load_config(config_path)
        self.weights = self.config.get('quality_weights', {
            'originality': 0.25,
            'engagement_potential': 0.20,
            'platform_relevance': 0.15,
            'brand_safety': 0.20,
            'human_generated': 0.15,
            'technical_quality': 0.05
        })
        
        # Initialize models
        self._initialize_models()
        
        # Content cache for duplicate detection
        self.content_cache = {}
        self.similarity_threshold = 0.85
        
        # Anti-gaming mechanisms
        self.user_history = {}
        self.spam_patterns = self._load_spam_patterns()
        
    def _load_config(self, config_path: str) -> Dict:
        """Load configuration from file or use defaults"""
        default_config = {
            'models': {
                'text_model': 'distilbert-base-uncased',
                'sentiment_model': 'cardiffnlp/twitter-roberta-base-sentiment-latest',
                'toxicity_model': 'unitary/toxic-bert'
            },
            'thresholds': {
                'originality_min': 0.3,
                'brand_safety_min': 0.7,
                'human_probability_min': 0.6
            },
            'quality_weights': {
                'originality': 0.25,
                'engagement_potential': 0.20,
                'platform_relevance': 0.15,
                'brand_safety': 0.20,
                'human_generated': 0.15,
                'technical_quality': 0.05
            }
        }
        
        if config_path:
            try:
                with open(config_path, 'r') as f:
                    config = json.load(f)
                return {**default_config, **config}
            except Exception as e:
                logger.warning(f"Failed to load config: {e}. Using defaults.")
        
        return default_config
    
    def _initialize_models(self):
        """Initialize all AI models"""
        try:
            # Text analysis models
            self.tokenizer = AutoTokenizer.from_pretrained(
                self.config['models']['text_model']
            )
            self.text_model = AutoModel.from_pretrained(
                self.config['models']['text_model']
            )
            
            # Sentiment analysis
            self.sentiment_analyzer = pipeline(
                "sentiment-analysis",
                model=self.config['models']['sentiment_model'],
                tokenizer=self.config['models']['sentiment_model']
            )
            
            # Toxicity detection
            self.toxicity_analyzer = pipeline(
                "text-classification",
                model=self.config['models']['toxicity_model']
            )
            
            # NLP processor
            self.nlp = spacy.load("en_core_web_sm")
            
            # TF-IDF for similarity detection
            self.tfidf_vectorizer = TfidfVectorizer(
                max_features=1000,
                stop_words='english',
                ngram_range=(1, 3)
            )
            
            logger.info("All models initialized successfully")
            
        except Exception as e:
            logger.error(f"Model initialization failed: {e}")
            raise
    
    def _load_spam_patterns(self) -> List[str]:
        """Load common spam patterns"""
        return [
            r'click here',
            r'free money',
            r'guaranteed profit',
            r'easy money',
            r'get rich quick',
            r'follow for follow',
            r'like for like',
            r'sub for sub',
            r'dm me',
            r'check my bio',
            r'link in bio',
            r'\$\d+.*day',  # $100/day patterns
            r'crypto.*pump',
            r'moon.*rocket'
        ]
    
    async def analyze_content_quality(
        self, 
        content: ContentMetadata
    ) -> QualityScore:
        """
        Main function to analyze content quality
        Returns QualityScore with overall multiplier (0.5x - 2.0x)
        """
        try:
            # Check cache first
            content_hash = self._generate_content_hash(content)
            if content_hash in self.content_cache:
                cached_result = self.content_cache[content_hash]
                if self._is_cache_valid(cached_result['timestamp']):
                    return cached_result['score']
            
            # Run parallel analysis
            analysis_tasks = [
                self._analyze_originality(content),
                self._analyze_engagement_potential(content),
                self._analyze_platform_relevance(content),
                self._analyze_brand_safety(content),
                self._analyze_human_generated(content),
                self._analyze_technical_quality(content)
            ]
            
            results = await asyncio.gather(*analysis_tasks)
            
            originality, engagement, platform_rel, brand_safety, human_gen, tech_quality = results
            
            # Calculate weighted overall score
            overall_score = (
                originality * self.weights['originality'] +
                engagement * self.weights['engagement_potential'] +
                platform_rel * self.weights['platform_relevance'] +
                brand_safety * self.weights['brand_safety'] +
                human_gen * self.weights['human_generated'] +
                tech_quality * self.weights['technical_quality']
            )
            
            # Apply anti-gaming adjustments
            overall_score = self._apply_anti_gaming_adjustments(
                overall_score, content
            )
            
            # Convert to multiplier (0.5x - 2.0x)
            multiplier = max(0.5, min(2.0, 0.5 + (overall_score * 1.5)))
            
            quality_score = QualityScore(
                originality=originality,
                engagement_potential=engagement,
                platform_relevance=platform_rel,
                brand_safety=brand_safety,
                human_generated=human_gen,
                technical_quality=tech_quality,
                overall_score=multiplier
            )
            
            # Cache result
            self.content_cache[content_hash] = {
                'score': quality_score,
                'timestamp': datetime.now()
            }
            
            # Update user history
            self._update_user_history(content, quality_score)
            
            return quality_score
            
        except Exception as e:
            logger.error(f"Quality analysis failed: {e}")
            # Return neutral score on error
            return QualityScore(
                originality=0.7,
                engagement_potential=0.7,
                platform_relevance=0.7,
                brand_safety=0.7,
                human_generated=0.7,
                technical_quality=0.7,
                overall_score=1.0
            )
    
    async def _analyze_originality(self, content: ContentMetadata) -> float:
        """Detect duplicate/similar content"""
        if not content.text_content:
            return 0.8  # Neutral for non-text content
        
        text = content.text_content.lower().strip()
        
        # Check exact duplicates
        text_hash = hashlib.md5(text.encode()).hexdigest()
        if text_hash in self.content_cache:
            return 0.1  # Severe penalty for exact duplicates
        
        # Check semantic similarity
        try:
            # Get text embedding
            inputs = self.tokenizer(
                text, 
                return_tensors="pt", 
                truncation=True, 
                max_length=512
            )
            
            with torch.no_grad():
                outputs = self.text_model(**inputs)
                embedding = outputs.last_hidden_state.mean(dim=1)
            
            # Compare with recent content
            max_similarity = 0.0
            for cached_hash, cached_data in list(self.content_cache.items())[-100:]:
                if 'embedding' in cached_data:
                    similarity = torch.cosine_similarity(
                        embedding, 
                        cached_data['embedding'], 
                        dim=1
                    ).item()
                    max_similarity = max(max_similarity, similarity)
            
            # Convert similarity to originality score
            if max_similarity > self.similarity_threshold:
                return max(0.1, 1.0 - max_similarity)
            
            return max(0.3, 1.0 - (max_similarity * 0.7))
            
        except Exception as e:
            logger.warning(f"Originality analysis failed: {e}")
            return 0.7
    
    async def _analyze_engagement_potential(self, content: ContentMetadata) -> float:
        """Predict content engagement potential"""
        score = 0.5  # Base score
        
        if content.text_content:
            text = content.text_content
            
            # Length analysis
            word_count = len(text.split())
            if 10 <= word_count <= 150:  # Optimal length
                score += 0.2
            elif word_count < 5:  # Too short
                score -= 0.2
            
            # Hashtag analysis
            if content.hashtags:
                hashtag_count = len(content.hashtags)
                if 3 <= hashtag_count <= 10:  # Optimal hashtag count
                    score += 0.15
                elif hashtag_count > 15:  # Hashtag spam
                    score -= 0.3
            
            # Sentiment analysis
            try:
                sentiment = self.sentiment_analyzer(text[:500])[0]
                if sentiment['label'] in ['POSITIVE', 'positive']:
                    score += 0.1 * sentiment['score']
                elif sentiment['label'] in ['NEGATIVE', 'negative']:
                    score -= 0.05 * sentiment['score']
            except Exception:
                pass
            
            # Question/call-to-action detection
            if any(marker in text.lower() for marker in ['?', 'what do you think', 'comment below', 'share your']):
                score += 0.1
            
            # URL detection (often lower engagement)
            if re.search(r'http[s]?://|www\.', text):
                score -= 0.1
        
        # Platform-specific adjustments
        platform_bonuses = {
            Platform.TIKTOK: 0.1,  # Video-first platform
            Platform.INSTAGRAM: 0.05,
            Platform.YOUTUBE: 0.08,
            Platform.TWITTER_X: 0.0,
            Platform.FACEBOOK: -0.05
        }
        
        score += platform_bonuses.get(content.platform, 0.0)
        
        return max(0.0, min(1.0, score))
    
    async def _analyze_platform_relevance(self, content: ContentMetadata) -> float:
        """Analyze content relevance to specific platform"""
        platform_requirements = {
            Platform.TIKTOK: {
                'video_preferred': True,
                'short_form': True,
                'trending_hashtags': True,
                'young_audience': True
            },
            Platform.INSTAGRAM: {
                'visual_preferred': True,
                'aesthetic_quality': True,
                'stories_format': True,
                'lifestyle_content': True
            },
            Platform.YOUTUBE: {
                'long_form': True,
                'educational': True,
                'thumbnail_quality': True,
                'series_content': True
            },
            Platform.TWITTER_X: {
                'short_text': True,
                'news_relevant': True,
                'real_time': True,
                'discussion_starter': True
            },
            Platform.FACEBOOK: {
                'community_focused': True,
                'longer_text': True,
                'family_friendly': True,
                'event_sharing': True
            }
        }
        
        requirements = platform_requirements.get(content.platform, {})
        score = 0.5
        
        if content.text_content:
            text = content.text_content
            word_count = len(text.split())
            
            # Platform-specific length preferences
            if content.platform == Platform.TWITTER_X and word_count <= 50:
                score += 0.2
            elif content.platform == Platform.FACEBOOK and 50 <= word_count <= 200:
                score += 0.2
            elif content.platform == Platform.INSTAGRAM and 20 <= word_count <= 100:
                score += 0.2
            
            # Content type preferences
            if content.platform == Platform.TIKTOK and content.content_type == ContentType.VIDEO:
                score += 0.3
            elif content.platform == Platform.INSTAGRAM and content.content_type in [ContentType.IMAGE, ContentType.VIDEO]:
                score += 0.2
            elif content.platform == Platform.YOUTUBE and content.content_type == ContentType.VIDEO:
                score += 0.3
        
        return max(0.0, min(1.0, score))
    
    async def _analyze_brand_safety(self, content: ContentMetadata) -> float:
        """Analyze brand safety and content appropriateness"""
        if not content.text_content:
            return 0.8  # Assume safe for non-text content
        
        text = content.text_content
        score = 1.0
        
        try:
            # Toxicity detection
            toxicity_result = self.toxicity_analyzer(text[:500])
            if toxicity_result[0]['label'] == 'TOXIC':
                toxicity_score = toxicity_result[0]['score']
                score -= toxicity_score * 0.8
            
            # Spam pattern detection
            spam_score = 0
            for pattern in self.spam_patterns:
                if re.search(pattern, text.lower()):
                    spam_score += 0.1
            
            score -= min(spam_score, 0.6)
            
            # Excessive capitalization
            caps_ratio = sum(1 for c in text if c.isupper()) / max(len(text), 1)
            if caps_ratio > 0.3:
                score -= 0.2
            
            # Excessive punctuation
            punct_ratio = sum(1 for c in text if c in '!?.,;') / max(len(text), 1)
            if punct_ratio > 0.1:
                score -= 0.1
            
            # Adult content keywords
            adult_keywords = [
                'sex', 'porn', 'nude', 'xxx', 'adult', 'nsfw',
                'gambling', 'casino', 'bet', 'poker'
            ]
            adult_count = sum(1 for keyword in adult_keywords if keyword in text.lower())
            if adult_count > 0:
                score -= adult_count * 0.3
            
        except Exception as e:
            logger.warning(f"Brand safety analysis failed: {e}")
            score = 0.7  # Conservative score on error
        
        return max(0.0, min(1.0, score))
    
    async def _analyze_human_generated(self, content: ContentMetadata) -> float:
        """Detect AI-generated vs human-generated content"""
        if not content.text_content:
            return 0.8  # Assume human for non-text
        
        text = content.text_content
        human_score = 0.5
        
        # Linguistic patterns analysis
        doc = self.nlp(text)
        
        # Natural language markers
        if len(doc.ents) > 0:  # Named entities
            human_score += 0.1
        
        # Sentence complexity variation
        sentence_lengths = [len(sent.text.split()) for sent in doc.sents]
        if len(sentence_lengths) > 1:
            length_variance = np.var(sentence_lengths)
            if length_variance > 5:  # Natural variation
                human_score += 0.15
        
        # Personal pronouns usage
        personal_pronouns = ['i', 'me', 'my', 'we', 'us', 'our']
        pronoun_count = sum(1 for token in doc if token.text.lower() in personal_pronouns)
        if pronoun_count > 0:
            human_score += 0.1
        
        # Emotional expressions
        emotion_markers = ['!', '?', 'wow', 'amazing', 'love', 'hate', 'feel']
        emotion_count = sum(1 for marker in emotion_markers if marker in text.lower())
        if emotion_count > 0:
            human_score += 0.1
        
        # Typos and informal language (indicators of human writing)
        informal_markers = ['lol', 'omg', 'btw', 'tbh', 'imo', 'gonna', 'wanna']
        informal_count = sum(1 for marker in informal_markers if marker in text.lower())
        if informal_count > 0:
            human_score += 0.1
        
        # AI-like patterns (very structured, repetitive)
        if self._detect_ai_patterns(text):
            human_score -= 0.3
        
        return max(0.0, min(1.0, human_score))
    
    def _detect_ai_patterns(self, text: str) -> bool:
        """Detect patterns common in AI-generated text"""
        ai_patterns = [
            r'in conclusion',
            r'furthermore',
            r'additionally',
            r'it is important to note',
            r'it should be mentioned',
            r'as an ai',
            r'as mentioned earlier',
            r'to summarize'
        ]
        
        pattern_count = sum(1 for pattern in ai_patterns if re.search(pattern, text.lower()))
        return pattern_count >= 2
    
    async def _analyze_technical_quality(self, content: ContentMetadata) -> float:
        """Analyze technical quality of content"""
        score = 0.7  # Base score
        
        if content.text_content:
            text = content.text_content
            
            # Grammar and spelling (basic check)
            doc = self.nlp(text)
            error_count = 0
            
            for token in doc:
                if token.is_alpha and not token.is_stop:
                    # Simple spell check using word frequency
                    if len(token.text) > 3 and token.text.lower() not in ['crypto', 'blockchain', 'finova']:
                        # Could integrate with proper spell checker
                        pass
            
            # Proper capitalization
            sentences = text.split('.')
            properly_capitalized = sum(1 for s in sentences if s.strip() and s.strip()[0].isupper())
            if properly_capitalized / max(len(sentences), 1) > 0.7:
                score += 0.1
            
            # Reasonable punctuation
            if text.count('!') <= 3 and text.count('?') <= 2:
                score += 0.1
        
        # Image/video quality would be analyzed here
        if content.image_urls or content.video_url:
            # Placeholder for image/video quality analysis
            score += 0.1
        
        return max(0.0, min(1.0, score))
    
    def _apply_anti_gaming_adjustments(
        self, 
        base_score: float, 
        content: ContentMetadata
    ) -> float:
        """Apply anti-gaming mechanisms"""
        adjusted_score = base_score
        
        # Check user history for patterns
        user_id = content.user_id
        if user_id in self.user_history:
            history = self.user_history[user_id]
            
            # Frequency check (posting too often)
            recent_posts = [
                post for post in history 
                if (datetime.now() - post['timestamp']).total_seconds() < 3600  # Last hour
            ]
            
            if len(recent_posts) > 10:  # More than 10 posts per hour
                adjusted_score *= 0.5
            elif len(recent_posts) > 5:
                adjusted_score *= 0.8
            
            # Quality consistency check
            if len(history) >= 5:
                recent_scores = [post['quality_score'] for post in history[-5:]]
                score_variance = np.var(recent_scores)
                
                # Very consistent scores might indicate gaming
                if score_variance < 0.01:
                    adjusted_score *= 0.7
            
            # Similar content check
            similar_content_count = 0
            for post in history[-10:]:  # Last 10 posts
                if content.text_content and post.get('text_content'):
                    # Simple similarity check
                    common_words = set(content.text_content.lower().split()) & set(post['text_content'].lower().split())
                    if len(common_words) / max(len(content.text_content.split()), 1) > 0.7:
                        similar_content_count += 1
            
            if similar_content_count > 3:
                adjusted_score *= 0.6
        
        return max(0.1, min(1.0, adjusted_score))
    
    def _update_user_history(
        self, 
        content: ContentMetadata, 
        quality_score: QualityScore
    ):
        """Update user history for anti-gaming tracking"""
        user_id = content.user_id
        
        if user_id not in self.user_history:
            self.user_history[user_id] = []
        
        # Add current content to history
        self.user_history[user_id].append({
            'timestamp': content.timestamp,
            'content_type': content.content_type.value,
            'platform': content.platform.value,
            'text_content': content.text_content,
            'quality_score': quality_score.overall_score
        })
        
        # Keep only last 50 entries per user
        self.user_history[user_id] = self.user_history[user_id][-50:]
    
    def _generate_content_hash(self, content: ContentMetadata) -> str:
        """Generate unique hash for content"""
        content_str = f"{content.user_id}:{content.platform.value}:{content.text_content or ''}:{content.timestamp.isoformat()}"
        return hashlib.sha256(content_str.encode()).hexdigest()
    
    def _is_cache_valid(self, timestamp: datetime, ttl_hours: int = 24) -> bool:
        """Check if cached result is still valid"""
        return (datetime.now() - timestamp).total_seconds() < ttl_hours * 3600
    
    def get_user_quality_metrics(self, user_id: str) -> Dict:
        """Get aggregated quality metrics for a user"""
        if user_id not in self.user_history:
            return {'average_quality': 0.7, 'total_posts': 0, 'quality_trend': 'stable'}
        
        history = self.user_history[user_id]
        scores = [post['quality_score'] for post in history]
        
        return {
            'average_quality': np.mean(scores),
            'total_posts': len(history),
            'quality_trend': 'improving' if len(scores) > 5 and np.mean(scores[-5:]) > np.mean(scores[:-5]) else 'stable',
            'consistency': 1.0 - np.var(scores) if len(scores) > 1 else 1.0
        }
    
    async def batch_analyze(self, contents: List[ContentMetadata]) -> List[QualityScore]:
        """Analyze multiple contents in batch for efficiency"""
        tasks = [self.analyze_content_quality(content) for content in contents]
        return await asyncio.gather(*tasks)

# Example usage and testing
async def main():
    """Example usage of the FinovaQualityClassifier"""
    classifier = FinovaQualityClassifier()
    
    # Example content
    sample_content = ContentMetadata(
        content_id="test_001",
        user_id="user_123",
        platform=Platform.INSTAGRAM,
        content_type=ContentType.TEXT,
        timestamp=datetime.now(),
        text_content="Just discovered this amazing crypto project! The future of social media is here. #crypto #blockchain #finova #socialmedia",
        hashtags=["crypto", "blockchain", "finova", "socialmedia"]
    )
    
    # Analyze quality
    quality_score = await classifier.analyze_content_quality(sample_content)
    
    print(f"Quality Analysis Results:")
    print(f"Originality: {quality_score.originality:.2f}")
    print(f"Engagement Potential: {quality_score.engagement_potential:.2f}")
    print(f"Platform Relevance: {quality_score.platform_relevance:.2f}")
    print(f"Brand Safety: {quality_score.brand_safety:.2f}")
    print(f"Human Generated: {quality_score.human_generated:.2f}")
    print(f"Technical Quality: {quality_score.technical_quality:.2f}")
    print(f"Overall Score (XP Multiplier): {quality_score.overall_score:.2f}x")

if __name__ == "__main__":
    asyncio.run(main())
    