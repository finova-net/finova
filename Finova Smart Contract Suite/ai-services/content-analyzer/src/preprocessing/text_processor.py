"""
Finova Network - AI Content Analyzer Text Processor
Enterprise-grade text preprocessing for social media content analysis
Supports XP calculation, quality scoring, and anti-bot detection
"""

import re
import string
import unicodedata
import hashlib
from typing import Dict, List, Tuple, Optional, Any, Set
from dataclasses import dataclass
from datetime import datetime
import asyncio
import aiofiles
import spacy
import emoji
import langdetect
from langdetect.lang_detect_exception import LangDetectException
import nltk
from nltk.corpus import stopwords
from nltk.tokenize import word_tokenize, sent_tokenize
from nltk.stem import WordNetLemmatizer, PorterStemmer
from nltk.sentiment import SentimentIntensityAnalyzer
from transformers import pipeline, AutoTokenizer, AutoModel
import torch
from sklearn.feature_extraction.text import TfidfVectorizer
from sklearn.metrics.pairwise import cosine_similarity
import numpy as np
import logging
from functools import lru_cache
import asyncpg
import redis.asyncio as redis
from cryptography.fernet import Fernet

# Download required NLTK data
try:
    nltk.download('punkt', quiet=True)
    nltk.download('stopwords', quiet=True)
    nltk.download('wordnet', quiet=True)
    nltk.download('vader_lexicon', quiet=True)
    nltk.download('averaged_perceptron_tagger', quiet=True)
except:
    pass

@dataclass
class ProcessingResult:
    """Results from text processing operations"""
    original_text: str
    cleaned_text: str
    tokens: List[str]
    lemmatized_tokens: List[str]
    sentences: List[str]
    language: str
    quality_score: float
    originality_score: float
    engagement_potential: float
    platform_relevance: Dict[str, float]
    sentiment_analysis: Dict[str, float]
    readability_score: float
    spam_probability: float
    ai_generated_probability: float
    content_features: Dict[str, Any]
    processing_metadata: Dict[str, Any]

@dataclass
class ContentMetrics:
    """Comprehensive content analysis metrics"""
    word_count: int
    char_count: int
    sentence_count: int
    avg_sentence_length: float
    unique_word_ratio: float
    emoji_count: int
    hashtag_count: int
    mention_count: int
    url_count: int
    profanity_score: float
    complexity_score: float

class FinovaTextProcessor:
    """
    Enterprise-grade text processor for Finova Network content analysis.
    Handles preprocessing, quality assessment, and feature extraction
    for social media content across multiple platforms.
    """
    
    def __init__(self, config: Dict[str, Any]):
        self.config = config
        self.logger = self._setup_logging()
        
        # Initialize models and tools
        self.nlp = None
        self.sentiment_analyzer = None
        self.ai_detector = None
        self.tokenizer = None
        self.lemmatizer = WordNetLemmatizer()
        self.stemmer = PorterStemmer()
        self.tfidf = TfidfVectorizer(max_features=5000, stop_words='english')
        
        # Cache and database connections
        self.redis_client = None
        self.db_pool = None
        
        # Content quality models
        self.quality_model = None
        self.engagement_model = None
        self.originality_model = None
        
        # Security and encryption
        self.cipher_suite = None
        
        # Platform-specific configurations
        self.platform_configs = {
            'instagram': {
                'max_length': 2200,
                'optimal_hashtags': 11,
                'emoji_weight': 1.2
            },
            'tiktok': {
                'max_length': 150,
                'optimal_hashtags': 5,
                'emoji_weight': 1.5
            },
            'youtube': {
                'max_length': 5000,
                'optimal_hashtags': 15,
                'emoji_weight': 0.8
            },
            'facebook': {
                'max_length': 63206,
                'optimal_hashtags': 3,
                'emoji_weight': 1.0
            },
            'twitter': {
                'max_length': 280,
                'optimal_hashtags': 2,
                'emoji_weight': 1.1
            }
        }
        
        # Initialize async components
        asyncio.create_task(self._initialize_async_components())
    
    def _setup_logging(self) -> logging.Logger:
        """Setup comprehensive logging system"""
        logger = logging.getLogger('finova_text_processor')
        logger.setLevel(logging.INFO)
        
        if not logger.handlers:
            handler = logging.StreamHandler()
            formatter = logging.Formatter(
                '%(asctime)s - %(name)s - %(levelname)s - %(message)s'
            )
            handler.setFormatter(formatter)
            logger.addHandler(handler)
        
        return logger
    
    async def _initialize_async_components(self):
        """Initialize async components and models"""
        try:
            # Load NLP models
            self.nlp = spacy.load("en_core_web_sm")
            self.sentiment_analyzer = SentimentIntensityAnalyzer()
            
            # Load AI detection model
            self.ai_detector = pipeline(
                "text-classification",
                model="roberta-base-openai-detector",
                device=0 if torch.cuda.is_available() else -1
            )
            
            # Initialize tokenizer
            self.tokenizer = AutoTokenizer.from_pretrained("distilbert-base-uncased")
            
            # Setup Redis connection
            if self.config.get('redis_url'):
                self.redis_client = redis.from_url(
                    self.config['redis_url'],
                    encoding="utf-8",
                    decode_responses=True
                )
            
            # Setup database connection
            if self.config.get('database_url'):
                self.db_pool = await asyncpg.create_pool(
                    self.config['database_url'],
                    min_size=5,
                    max_size=20
                )
            
            # Initialize encryption
            if self.config.get('encryption_key'):
                self.cipher_suite = Fernet(self.config['encryption_key'].encode())
            
            self.logger.info("Text processor initialization completed successfully")
            
        except Exception as e:
            self.logger.error(f"Failed to initialize text processor: {str(e)}")
            raise
    
    async def process_content(
        self,
        text: str,
        platform: str = "general",
        user_id: str = None,
        content_type: str = "post",
        additional_context: Dict[str, Any] = None
    ) -> ProcessingResult:
        """
        Main content processing pipeline that returns comprehensive analysis
        """
        start_time = datetime.utcnow()
        
        try:
            # Input validation
            if not text or not isinstance(text, str):
                raise ValueError("Invalid text input")
            
            if len(text) > 100000:  # Safety limit
                raise ValueError("Text too long for processing")
            
            # Check cache first
            cache_key = self._generate_cache_key(text, platform, content_type)
            cached_result = await self._get_cached_result(cache_key)
            if cached_result:
                self.logger.info(f"Retrieved cached result for content hash: {cache_key[:8]}")
                return cached_result
            
            # Core preprocessing
            cleaned_text = await self._clean_text(text)
            tokens = await self._tokenize_text(cleaned_text)
            lemmatized_tokens = await self._lemmatize_tokens(tokens)
            sentences = await self._extract_sentences(cleaned_text)
            
            # Language detection
            language = await self._detect_language(cleaned_text)
            
            # Content metrics
            metrics = await self._calculate_content_metrics(text, cleaned_text)
            
            # Quality analysis
            quality_score = await self._assess_quality(
                cleaned_text, tokens, metrics, platform
            )
            
            # Originality detection
            originality_score = await self._check_originality(
                cleaned_text, user_id
            )
            
            # Engagement prediction
            engagement_potential = await self._predict_engagement(
                text, platform, metrics
            )
            
            # Platform relevance
            platform_relevance = await self._calculate_platform_relevance(
                text, metrics
            )
            
            # Sentiment analysis
            sentiment_analysis = await self._analyze_sentiment(cleaned_text)
            
            # Readability assessment
            readability_score = await self._calculate_readability(
                cleaned_text, sentences
            )
            
            # Spam detection
            spam_probability = await self._detect_spam(
                text, cleaned_text, metrics, user_id
            )
            
            # AI-generated content detection
            ai_generated_probability = await self._detect_ai_content(cleaned_text)
            
            # Extract advanced features
            content_features = await self._extract_content_features(
                text, cleaned_text, tokens, platform
            )
            
            # Processing metadata
            processing_time = (datetime.utcnow() - start_time).total_seconds()
            processing_metadata = {
                'processing_time_seconds': processing_time,
                'processor_version': '1.0.0',
                'timestamp': start_time.isoformat(),
                'platform': platform,
                'content_type': content_type,
                'user_id_hash': hashlib.sha256(user_id.encode()).hexdigest()[:16] if user_id else None
            }
            
            # Create result object
            result = ProcessingResult(
                original_text=text,
                cleaned_text=cleaned_text,
                tokens=tokens,
                lemmatized_tokens=lemmatized_tokens,
                sentences=sentences,
                language=language,
                quality_score=quality_score,
                originality_score=originality_score,
                engagement_potential=engagement_potential,
                platform_relevance=platform_relevance,
                sentiment_analysis=sentiment_analysis,
                readability_score=readability_score,
                spam_probability=spam_probability,
                ai_generated_probability=ai_generated_probability,
                content_features=content_features,
                processing_metadata=processing_metadata
            )
            
            # Cache result
            await self._cache_result(cache_key, result)
            
            # Store analytics data
            await self._store_analytics_data(result, user_id)
            
            self.logger.info(
                f"Content processed successfully in {processing_time:.3f}s - "
                f"Quality: {quality_score:.3f}, Originality: {originality_score:.3f}"
            )
            
            return result
            
        except Exception as e:
            self.logger.error(f"Content processing failed: {str(e)}")
            raise
    
    async def _clean_text(self, text: str) -> str:
        """Comprehensive text cleaning and normalization"""
        # Unicode normalization
        text = unicodedata.normalize('NFKD', text)
        
        # Remove zero-width characters
        text = re.sub(r'[\u200b-\u200f\ufeff]', '', text)
        
        # Preserve emojis and special characters for analysis
        # but normalize whitespace
        text = re.sub(r'\s+', ' ', text)
        
        # Remove excessive punctuation but preserve meaning
        text = re.sub(r'([.!?]){3,}', r'\1\1\1', text)
        
        # Clean up URLs while preserving the fact they exist
        url_pattern = r'http[s]?://(?:[a-zA-Z]|[0-9]|[$-_@.&+]|[!*\\(\\),]|(?:%[0-9a-fA-F][0-9a-fA-F]))+'
        text = re.sub(url_pattern, ' [URL] ', text)
        
        # Clean up email addresses
        email_pattern = r'\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Z|a-z]{2,}\b'
        text = re.sub(email_pattern, ' [EMAIL] ', text)
        
        # Normalize mentions and hashtags for better processing
        text = re.sub(r'@(\w+)', r' [MENTION:\1] ', text)
        text = re.sub(r'#(\w+)', r' [HASHTAG:\1] ', text)
        
        # Strip and normalize spaces
        text = text.strip()
        text = re.sub(r'\s+', ' ', text)
        
        return text
    
    async def _tokenize_text(self, text: str) -> List[str]:
        """Advanced tokenization with context preservation"""
        try:
            # Use spaCy for sophisticated tokenization
            doc = self.nlp(text)
            
            # Extract tokens while preserving important features
            tokens = []
            for token in doc:
                if not token.is_space:
                    # Preserve special tokens
                    if token.text.startswith('[') and token.text.endswith(']'):
                        tokens.append(token.text)
                    elif not token.is_punct or token.text in ['!', '?', '.']:
                        tokens.append(token.lemma_.lower())
            
            return tokens
            
        except Exception as e:
            self.logger.warning(f"spaCy tokenization failed, falling back to NLTK: {str(e)}")
            # Fallback to NLTK
            return word_tokenize(text.lower())
    
    async def _lemmatize_tokens(self, tokens: List[str]) -> List[str]:
        """Advanced lemmatization with POS tagging"""
        try:
            # Use spaCy for better lemmatization
            text = ' '.join(tokens)
            doc = self.nlp(text)
            
            lemmatized = []
            for token in doc:
                if not token.is_space and not token.is_punct:
                    lemmatized.append(token.lemma_.lower())
            
            return lemmatized
            
        except Exception as e:
            self.logger.warning(f"spaCy lemmatization failed, using NLTK: {str(e)}")
            return [self.lemmatizer.lemmatize(token) for token in tokens]
    
    async def _extract_sentences(self, text: str) -> List[str]:
        """Extract sentences with proper handling of social media text"""
        try:
            # Use spaCy for better sentence segmentation
            doc = self.nlp(text)
            sentences = [sent.text.strip() for sent in doc.sents if sent.text.strip()]
            
            # Fallback to NLTK if spaCy fails
            if not sentences:
                sentences = sent_tokenize(text)
            
            return sentences
            
        except Exception as e:
            self.logger.warning(f"Sentence extraction failed: {str(e)}")
            return [text]  # Return original text as single sentence
    
    async def _detect_language(self, text: str) -> str:
        """Detect content language with confidence scoring"""
        try:
            # Clean text for language detection
            clean_text = re.sub(r'\[.*?\]', '', text)  # Remove special tokens
            clean_text = emoji.demojize(clean_text)  # Convert emojis to text
            clean_text = re.sub(r'[^\w\s]', ' ', clean_text)  # Remove punctuation
            
            if len(clean_text.strip()) < 10:
                return 'unknown'
            
            detected_lang = langdetect.detect(clean_text)
            return detected_lang
            
        except LangDetectException:
            return 'unknown'
        except Exception as e:
            self.logger.warning(f"Language detection failed: {str(e)}")
            return 'unknown'
    
    async def _calculate_content_metrics(
        self, 
        original_text: str, 
        cleaned_text: str
    ) -> ContentMetrics:
        """Calculate comprehensive content metrics"""
        
        # Basic counts
        word_count = len(cleaned_text.split())
        char_count = len(original_text)
        
        # Sentence analysis
        sentences = await self._extract_sentences(cleaned_text)
        sentence_count = len(sentences)
        avg_sentence_length = word_count / max(sentence_count, 1)
        
        # Unique word ratio
        words = cleaned_text.lower().split()
        unique_words = set(words)
        unique_word_ratio = len(unique_words) / max(word_count, 1)
        
        # Social media elements
        emoji_count = len(emoji.emoji_count(original_text))
        hashtag_count = len(re.findall(r'#\w+', original_text))
        mention_count = len(re.findall(r'@\w+', original_text))
        url_count = len(re.findall(r'http[s]?://[^\s]+', original_text))
        
        # Profanity score (simplified - in production use a comprehensive filter)
        profanity_words = {'damn', 'hell', 'crap'}  # Placeholder
        profanity_score = sum(1 for word in words if word.lower() in profanity_words) / max(word_count, 1)
        
        # Complexity score based on various factors
        complexity_score = (
            min(avg_sentence_length / 20, 1.0) * 0.3 +
            min(unique_word_ratio * 2, 1.0) * 0.3 +
            min(len([w for w in words if len(w) > 6]) / max(word_count, 1) * 3, 1.0) * 0.4
        )
        
        return ContentMetrics(
            word_count=word_count,
            char_count=char_count,
            sentence_count=sentence_count,
            avg_sentence_length=avg_sentence_length,
            unique_word_ratio=unique_word_ratio,
            emoji_count=emoji_count,
            hashtag_count=hashtag_count,
            mention_count=mention_count,
            url_count=url_count,
            profanity_score=profanity_score,
            complexity_score=complexity_score
        )
    
    async def _assess_quality(
        self,
        text: str,
        tokens: List[str],
        metrics: ContentMetrics,
        platform: str
    ) -> float:
        """Assess content quality using multiple factors"""
        
        scores = []
        
        # Length appropriateness for platform
        platform_config = self.platform_configs.get(platform, self.platform_configs['instagram'])
        optimal_length = platform_config['max_length'] * 0.7  # 70% of max is optimal
        length_score = 1.0 - abs(metrics.word_count * 5 - optimal_length) / optimal_length
        length_score = max(0.2, min(1.0, length_score))
        scores.append(length_score * 0.15)
        
        # Vocabulary diversity
        diversity_score = min(metrics.unique_word_ratio * 2, 1.0)
        scores.append(diversity_score * 0.20)
        
        # Content complexity (not too simple, not too complex)
        complexity_score = 1.0 - abs(metrics.complexity_score - 0.6)  # Optimal complexity around 0.6
        scores.append(complexity_score * 0.15)
        
        # Grammar and readability (simplified)
        grammar_score = 1.0 - min(metrics.profanity_score * 2, 0.5)  # Penalize profanity
        scores.append(grammar_score * 0.20)
        
        # Engagement elements
        engagement_score = 0.5  # Base score
        if metrics.emoji_count > 0:
            engagement_score += 0.1
        if metrics.hashtag_count > 0:
            engagement_score += 0.1
        if 1 <= metrics.hashtag_count <= platform_config['optimal_hashtags']:
            engagement_score += 0.2
        if metrics.sentence_count > 1:
            engagement_score += 0.1
        engagement_score = min(engagement_score, 1.0)
        scores.append(engagement_score * 0.20)
        
        # Spam indicators (inverse)
        spam_indicators = 0
        if metrics.emoji_count > metrics.word_count * 0.3:  # Too many emojis
            spam_indicators += 0.2
        if metrics.hashtag_count > 10:  # Too many hashtags
            spam_indicators += 0.3
        if len(set(tokens)) < len(tokens) * 0.5:  # Too repetitive
            spam_indicators += 0.3
        
        spam_score = 1.0 - min(spam_indicators, 0.8)
        scores.append(spam_score * 0.10)
        
        # Calculate weighted average
        final_score = sum(scores)
        return max(0.1, min(1.0, final_score))
    
    async def _check_originality(self, text: str, user_id: str = None) -> float:
        """Check content originality against existing content"""
        try:
            # Create content hash for duplicate detection
            content_hash = hashlib.sha256(text.encode()).hexdigest()
            
            # Check against cached content hashes
            if self.redis_client:
                is_duplicate = await self.redis_client.exists(f"content_hash:{content_hash}")
                if is_duplicate:
                    return 0.1  # Very low originality for exact duplicates
                
                # Store this content hash
                await self.redis_client.setex(
                    f"content_hash:{content_hash}",
                    3600 * 24 * 7,  # 7 days
                    user_id or "anonymous"
                )
            
            # Check semantic similarity (simplified version)
            # In production, this would use vector embeddings and similarity search
            similarity_score = await self._calculate_semantic_similarity(text, user_id)
            
            # Return originality score (1.0 = completely original, 0.0 = duplicate)
            originality_score = 1.0 - similarity_score
            return max(0.1, min(1.0, originality_score))
            
        except Exception as e:
            self.logger.warning(f"Originality check failed: {str(e)}")
            return 0.8  # Default to high originality if check fails
    
    async def _calculate_semantic_similarity(self, text: str, user_id: str = None) -> float:
        """Calculate semantic similarity to existing content (simplified)"""
        try:
            # This is a simplified version - production would use sophisticated embeddings
            
            # Get recent content from user or general pool
            if self.db_pool and user_id:
                async with self.db_pool.acquire() as conn:
                    recent_content = await conn.fetch(
                        """
                        SELECT content_text FROM content_analysis 
                        WHERE user_id = $1 AND created_at > NOW() - INTERVAL '7 days'
                        ORDER BY created_at DESC LIMIT 10
                        """,
                        user_id
                    )
                    
                    if recent_content:
                        # Simple TF-IDF based similarity
                        texts = [text] + [row['content_text'] for row in recent_content]
                        try:
                            tfidf_matrix = self.tfidf.fit_transform(texts)
                            similarities = cosine_similarity(tfidf_matrix[0:1], tfidf_matrix[1:])
                            max_similarity = np.max(similarities) if similarities.size > 0 else 0.0
                            return float(max_similarity)
                        except:
                            return 0.0
            
            return 0.0  # No similar content found
            
        except Exception as e:
            self.logger.warning(f"Semantic similarity calculation failed: {str(e)}")
            return 0.0
    
    async def _predict_engagement(
        self, 
        text: str, 
        platform: str, 
        metrics: ContentMetrics
    ) -> float:
        """Predict engagement potential based on content features"""
        
        engagement_factors = []
        
        # Platform-specific factors
        platform_config = self.platform_configs.get(platform, self.platform_configs['instagram'])
        
        # Length optimization
        optimal_length = platform_config['max_length'] * 0.6
        length_factor = 1.0 - abs(metrics.char_count - optimal_length) / optimal_length
        engagement_factors.append(max(0.2, min(1.0, length_factor)) * 0.2)
        
        # Emoji usage
        emoji_factor = min(metrics.emoji_count * platform_config['emoji_weight'] / 3, 1.0)
        engagement_factors.append(emoji_factor * 0.15)
        
        # Hashtag optimization
        optimal_hashtags = platform_config['optimal_hashtags']
        if metrics.hashtag_count == 0:
            hashtag_factor = 0.3
        elif metrics.hashtag_count <= optimal_hashtags:
            hashtag_factor = 0.8 + (metrics.hashtag_count / optimal_hashtags) * 0.2
        else:
            hashtag_factor = max(0.2, 1.0 - (metrics.hashtag_count - optimal_hashtags) * 0.1)
        engagement_factors.append(hashtag_factor * 0.15)
        
        # Question/call-to-action detection
        cta_indicators = ['?', 'what do you think', 'comment below', 'share your', 'tag someone']
        cta_factor = 0.5
        text_lower = text.lower()
        for indicator in cta_indicators:
            if indicator in text_lower:
                cta_factor += 0.1
        engagement_factors.append(min(cta_factor, 1.0) * 0.2)
        
        # Readability and accessibility
        readability_factor = 1.0 - abs(metrics.complexity_score - 0.5)  # Optimal complexity
        engagement_factors.append(readability_factor * 0.15)
        
        # Trending topic detection (simplified)
        trending_keywords = ['trending', 'viral', 'challenge', 'new', 'exclusive']
        trending_factor = 0.5
        for keyword in trending_keywords:
            if keyword in text_lower:
                trending_factor += 0.1
        engagement_factors.append(min(trending_factor, 1.0) * 0.15)
        
        # Calculate final engagement score
        final_score = sum(engagement_factors)
        return max(0.1, min(1.0, final_score))
    
    async def _calculate_platform_relevance(
        self, 
        text: str, 
        metrics: ContentMetrics
    ) -> Dict[str, float]:
        """Calculate relevance scores for each platform"""
        
        relevance_scores = {}
        
        for platform, config in self.platform_configs.items():
            score_factors = []
            
            # Length appropriateness
            if metrics.char_count <= config['max_length']:
                length_score = 1.0 - (metrics.char_count / config['max_length']) * 0.2
            else:
                length_score = max(0.1, 1.0 - (metrics.char_count - config['max_length']) / config['max_length'])
            score_factors.append(length_score * 0.3)
            
            # Hashtag optimization
            hashtag_score = 1.0 - abs(metrics.hashtag_count - config['optimal_hashtags']) / max(config['optimal_hashtags'], 1)
            score_factors.append(max(0.2, hashtag_score) * 0.25)
            
            # Emoji appropriateness
            emoji_score = min(metrics.emoji_count * config['emoji_weight'] / 5, 1.0)
            score_factors.append(emoji_score * 0.2)
            
            # Platform-specific content patterns
            platform_score = 0.5  # Base score
            text_lower = text.lower()
            
            if platform == 'tiktok':
                if any(word in text_lower for word in ['dance', 'music', 'trend', 'viral', 'challenge']):
                    platform_score += 0.3
                if metrics.emoji_count > 2:
                    platform_score += 0.2
            elif platform == 'instagram':
                if any(word in text_lower for word in ['photo', 'picture', 'beautiful', 'aesthetic']):
                    platform_score += 0.3
                if 5 <= metrics.hashtag_count <= 15:
                    platform_score += 0.2
            elif platform == 'youtube':
                if any(word in text_lower for word in ['video', 'tutorial', 'review', 'subscribe']):
                    platform_score += 0.3
                if metrics.word_count > 50:
                    platform_score += 0.2
            elif platform == 'twitter':
                if metrics.char_count <= 280:
                    platform_score += 0.3
                if any(word in text_lower for word in ['breaking', 'news', 'update']):
                    platform_score += 0.2
            
            score_factors.append(min(platform_score, 1.0) * 0.25)
            
            # Calculate final platform relevance
            relevance_scores[platform] = max(0.1, min(1.0, sum(score_factors)))
        
        return relevance_scores
    
    async def _analyze_sentiment(self, text: str) -> Dict[str, float]:
        """Comprehensive sentiment analysis"""
        try:
            # Use VADER sentiment analyzer
            sentiment_scores = self.sentiment_analyzer.polarity_scores(text)
            
            # Additional emotional indicators
            emotion_keywords = {
                'joy': ['happy', 'joy', 'excited', 'amazing', 'wonderful', 'love'],
                'anger': ['angry', 'hate', 'furious', 'mad', 'annoyed'],
                'sadness': ['sad', 'depressed', 'crying', 'heartbroken', 'disappointed'],
                'fear': ['scared', 'afraid', 'worried', 'anxious', 'nervous'],
                'surprise': ['wow', 'amazing', 'incredible', 'unbelievable', 'shocking']
            }
            
            text_lower = text.lower()
            emotion_scores = {}
            
            for emotion, keywords in emotion_keywords.items():
                score = sum(1 for keyword in keywords if keyword in text_lower)
                emotion_scores[emotion] = min(score / 5, 1.0)  # Normalize to 0-1
            
            return {
                'positive': sentiment_scores['pos'],
                'negative': sentiment_scores['neg'],
                'neutral': sentiment_scores['neu'],
                'compound': sentiment_scores['compound'],
                **emotion_scores
            }
            
        except Exception as e:
            self.logger.warning(f"Sentiment analysis failed: {str(e)}")
            return {
                'positive': 0.5,
                'negative': 0.1,
                'neutral': 0.4,
                'compound': 0.0,
                'joy': 0.0,
                'anger': 0.0,
                'sadness': 0.0,
                'fear': 0.0,
                'surprise': 0.0
            }
    
    async def _calculate_readability(self, text: str, sentences: List[str]) -> float:
        """Calculate content readability using multiple metrics"""
        try:
            if not text or not sentences:
                return 0.5
            
            # Basic readability factors
            words = text.split()
            word_count = len(words)
            sentence_count = len(sentences)
            
            if word_count == 0 or sentence_count == 0:
                return 0.5
            
            # Average sentence length
            avg_sentence_length = word_count / sentence_count
            
            # Average word length
            avg_word_length = sum(len(word) for word in words) / word_count
            
            # Simplified Flesch Reading Ease approximation
            # Formula: 206.835 - (1.015 × ASL) - (84.6 × ASW)
            # ASL = Average Sentence Length, ASW = Average Syllables per Word
            
            # Estimate syllables (simplified)
            def count_syllables(word):
                word = word.lower()
                vowels = 'aeiouy'
                syllable_count = 0
                prev_char_was_vowel = False
                
                for char in word:
                    if char in vowels:
                        if not prev_char_was_vowel:
                            syllable_count += 1
                        prev_char_was_vowel = True
                    else:
                        prev_char_was_vowel = False
                
                if word.endswith('e'):
                    syllable_count -= 1
                
                return max(1, syllable_count)
            
            avg_syllables = sum(count_syllables(word) for word in words) / word_count
            
            # Flesch Reading Ease
            flesch_score = 206.835 - (1.015 * avg_sentence_length) - (84.6 * avg_syllables)
            
            # Normalize to 0-1 scale (90-100 = very easy, 0-30 = very difficult)
            readability_score = max(0.0, min(1.0, flesch_score / 100))
            
            # Adjust for social media context (shorter is often better)
            if avg_sentence_length < 15:  # Short sentences are good for social media
                readability_score += 0.1
            
            if avg_word_length < 5:  # Simple words are good
                readability_score += 0.1
            
            return max(0.1, min(1.0, readability_score))
            
        except Exception as e:
            self.logger.warning(f"Readability calculation failed: {str(e)}")
            return 0.5
    
    async def _detect_spam(
        self, 
        original_text: str, 
        cleaned_text: str, 
        metrics: ContentMetrics, 
        user_id: str = None
    ) -> float:
        """Detect spam probability using multiple indicators"""
        try:
            spam_indicators = []
            
            # Excessive repetition
            words = cleaned_text.lower().split()
            if words:
                unique_ratio = len(set(words)) / len(words)
                repetition_score = 1.0 - unique_ratio
                spam_indicators.append(repetition_score * 0.25)
            
            # Excessive capitalization
            caps_ratio = sum(1 for c in original_text if c.isupper()) / max(len(original_text), 1)
            caps_score = min(caps_ratio * 3, 1.0)  # Penalize excessive caps
            spam_indicators.append(caps_score * 0.15)
            
            # Excessive punctuation
            punct_ratio = sum(1 for c in original_text if c in string.punctuation) / max(len(original_text), 1)
            punct_score = min(punct_ratio * 5, 1.0)
            spam_indicators.append(punct_score * 0.15)
            
            # Excessive emojis
            emoji_ratio = metrics.emoji_count / max(metrics.word_count, 1)
            emoji_score = min(emoji_ratio * 2, 1.0)
            spam_indicators.append(emoji_score * 0.15)
            
            # Excessive hashtags
            hashtag_score = min(metrics.hashtag_count / 10, 1.0)
            spam_indicators.append(hashtag_score * 0.15)
            
            # Suspicious patterns
            pattern_score = 0.0
            spam_patterns = [
                r'(.)\1{4,}',  # Repeated characters
                r'[!]{3,}',    # Multiple exclamations
                r'[\$€£¥]{2,}', # Multiple currency symbols
                r'[0-9]{10,}',  # Long number sequences
            ]
            
            for pattern in spam_patterns:
                if re.search(pattern, original_text):
                    pattern_score += 0.2
            
            spam_indicators.append(min(pattern_score, 1.0) * 0.15)
            
            # Calculate final spam probability
            spam_probability = sum(spam_indicators)
            return max(0.0, min(1.0, spam_probability))
            
        except Exception as e:
            self.logger.warning(f"Spam detection failed: {str(e)}")
            return 0.1  # Low spam probability if detection fails
    
    async def _detect_ai_content(self, text: str) -> float:
        """Detect AI-generated content probability"""
        try:
            if not self.ai_detector or len(text.strip()) < 50:
                return 0.1  # Default low probability for short text
            
            # Use the AI detection model
            result = self.ai_detector(text)
            
            if isinstance(result, list) and len(result) > 0:
                # Extract probability from model output
                ai_label_score = 0.0
                for item in result:
                    if item.get('label', '').upper() in ['AI', 'MACHINE', 'ARTIFICIAL']:
                        ai_label_score = max(ai_label_score, item.get('score', 0.0))
                
                return min(max(ai_label_score, 0.0), 1.0)
            
            # Fallback heuristics for AI detection
            ai_indicators = []
            
            # Perfect grammar with repetitive structure
            sentences = await self._extract_sentences(text)
            if len(sentences) > 2:
                avg_length_variance = np.std([len(s.split()) for s in sentences])
                if avg_length_variance < 2.0:  # Very uniform sentence lengths
                    ai_indicators.append(0.3)
            
            # Lack of personal pronouns and emotions
            personal_pronouns = ['i', 'me', 'my', 'myself', 'we', 'us', 'our']
            text_lower = text.lower()
            pronoun_count = sum(1 for pronoun in personal_pronouns if pronoun in text_lower.split())
            if pronoun_count == 0 and len(text.split()) > 20:
                ai_indicators.append(0.2)
            
            # Overly formal or structured language
            formal_indicators = ['furthermore', 'moreover', 'additionally', 'consequently', 'therefore']
            formal_count = sum(1 for indicator in formal_indicators if indicator in text_lower)
            if formal_count > 2:
                ai_indicators.append(0.2)
            
            return min(sum(ai_indicators), 0.8)  # Cap at 0.8 for heuristics
            
        except Exception as e:
            self.logger.warning(f"AI content detection failed: {str(e)}")
            return 0.1
    
    async def _extract_content_features(
        self, 
        original_text: str, 
        cleaned_text: str, 
        tokens: List[str], 
        platform: str
    ) -> Dict[str, Any]:
        """Extract comprehensive content features for ML models"""
        try:
            features = {}
            
            # Basic linguistic features
            features['text_length'] = len(original_text)
            features['word_count'] = len(tokens)
            features['avg_word_length'] = sum(len(token) for token in tokens) / max(len(tokens), 1)
            features['sentence_count'] = len(await self._extract_sentences(cleaned_text))
            
            # Lexical diversity
            unique_tokens = set(tokens)
            features['lexical_diversity'] = len(unique_tokens) / max(len(tokens), 1)
            features['hapax_legomena'] = sum(1 for token in unique_tokens if tokens.count(token) == 1)
            
            # Part-of-speech features (simplified)
            try:
                doc = self.nlp(cleaned_text)
                pos_counts = {}
                for token in doc:
                    pos = token.pos_
                    pos_counts[pos] = pos_counts.get(pos, 0) + 1
                
                total_tokens = sum(pos_counts.values())
                features['noun_ratio'] = pos_counts.get('NOUN', 0) / max(total_tokens, 1)
                features['verb_ratio'] = pos_counts.get('VERB', 0) / max(total_tokens, 1)
                features['adj_ratio'] = pos_counts.get('ADJ', 0) / max(total_tokens, 1)
                features['adv_ratio'] = pos_counts.get('ADV', 0) / max(total_tokens, 1)
            except:
                features.update({
                    'noun_ratio': 0.3,
                    'verb_ratio': 0.2,
                    'adj_ratio': 0.1,
                    'adv_ratio': 0.05
                })
            
            # Social media specific features
            features['emoji_count'] = len(emoji.emoji_count(original_text))
            features['hashtag_count'] = len(re.findall(r'#\w+', original_text))
            features['mention_count'] = len(re.findall(r'@\w+', original_text))
            features['url_count'] = len(re.findall(r'http[s]?://[^\s]+', original_text))
            
            # Engagement indicators
            features['has_question'] = '?' in original_text
            features['has_exclamation'] = '!' in original_text
            features['call_to_action'] = any(
                cta in original_text.lower() 
                for cta in ['comment', 'share', 'like', 'follow', 'subscribe', 'tag']
            )
            
            # Content type indicators
            features['has_numbers'] = bool(re.search(r'\d', original_text))
            features['has_caps'] = any(c.isupper() for c in original_text)
            features['caps_ratio'] = sum(1 for c in original_text if c.isupper()) / max(len(original_text), 1)
            
            # Platform optimization features
            platform_config = self.platform_configs.get(platform, {})
            if platform_config:
                features['length_appropriateness'] = 1.0 - abs(
                    len(original_text) - platform_config.get('max_length', 1000) * 0.6
                ) / platform_config.get('max_length', 1000)
                features['hashtag_appropriateness'] = 1.0 - abs(
                    features['hashtag_count'] - platform_config.get('optimal_hashtags', 5)
                ) / max(platform_config.get('optimal_hashtags', 5), 1)
            
            # Temporal features
            features['timestamp'] = datetime.utcnow().isoformat()
            features['hour_of_day'] = datetime.utcnow().hour
            features['day_of_week'] = datetime.utcnow().weekday()
            
            # Normalize all numeric features to [0, 1] range where applicable
            for key, value in features.items():
                if isinstance(value, (int, float)) and key.endswith('_ratio'):
                    features[key] = max(0.0, min(1.0, float(value)))
            
            return features
            
        except Exception as e:
            self.logger.error(f"Feature extraction failed: {str(e)}")
            return {}
    
    def _generate_cache_key(self, text: str, platform: str, content_type: str) -> str:
        """Generate cache key for processed content"""
        content_hash = hashlib.sha256(f"{text}:{platform}:{content_type}".encode()).hexdigest()
        return f"processed_content:{content_hash}"
    
    async def _get_cached_result(self, cache_key: str) -> Optional[ProcessingResult]:
        """Retrieve cached processing result"""
        try:
            if not self.redis_client:
                return None
            
            cached_data = await self.redis_client.get(cache_key)
            if cached_data:
                # In production, implement proper serialization/deserialization
                # For now, return None to force fresh processing
                pass
            
            return None
            
        except Exception as e:
            self.logger.warning(f"Cache retrieval failed: {str(e)}")
            return None
    
    async def _cache_result(self, cache_key: str, result: ProcessingResult):
        """Cache processing result"""
        try:
            if not self.redis_client:
                return
            
            # In production, implement proper serialization
            # For now, cache basic metrics only
            cache_data = {
                'quality_score': result.quality_score,
                'originality_score': result.originality_score,
                'engagement_potential': result.engagement_potential,
                'spam_probability': result.spam_probability,
                'cached_at': datetime.utcnow().isoformat()
            }
            
            await self.redis_client.setex(
                cache_key,
                3600,  # 1 hour cache
                str(cache_data)
            )
            
        except Exception as e:
            self.logger.warning(f"Caching failed: {str(e)}")
    
    async def _store_analytics_data(self, result: ProcessingResult, user_id: str = None):
        """Store processing results for analytics"""
        try:
            if not self.db_pool:
                return
            
            async with self.db_pool.acquire() as conn:
                # Encrypt sensitive content if encryption is available
                content_to_store = result.original_text
                if self.cipher_suite and len(content_to_store) < 1000:
                    try:
                        content_to_store = self.cipher_suite.encrypt(
                            content_to_store.encode()
                        ).decode()
                    except:
                        pass  # Store unencrypted if encryption fails
                
                await conn.execute("""
                    INSERT INTO content_analysis (
                        user_id, content_text, content_hash, language,
                        quality_score, originality_score, engagement_potential,
                        spam_probability, ai_generated_probability,
                        word_count, char_count, emoji_count, hashtag_count,
                        sentiment_compound, platform_relevance,
                        processing_time, created_at
                    ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17)
                """,
                    user_id,
                    content_to_store,
                    hashlib.sha256(result.original_text.encode()).hexdigest(),
                    result.language,
                    result.quality_score,
                    result.originality_score,
                    result.engagement_potential,
                    result.spam_probability,
                    result.ai_generated_probability,
                    result.content_features.get('word_count', 0),
                    result.content_features.get('text_length', 0),
                    result.content_features.get('emoji_count', 0),
                    result.content_features.get('hashtag_count', 0),
                    result.sentiment_analysis.get('compound', 0.0),
                    str(result.platform_relevance),
                    result.processing_metadata.get('processing_time_seconds', 0.0),
                    datetime.utcnow()
                )
                
        except Exception as e:
            self.logger.warning(f"Analytics storage failed: {str(e)}")
    
    async def batch_process_content(
        self, 
        content_list: List[Dict[str, Any]], 
        max_concurrent: int = 10
    ) -> List[ProcessingResult]:
        """Process multiple content items concurrently"""
        
        async def process_single(content_data):
            try:
                return await self.process_content(
                    text=content_data['text'],
                    platform=content_data.get('platform', 'general'),
                    user_id=content_data.get('user_id'),
                    content_type=content_data.get('content_type', 'post'),
                    additional_context=content_data.get('context')
                )
            except Exception as e:
                self.logger.error(f"Batch processing item failed: {str(e)}")
                return None
        
        # Process in batches to avoid overwhelming the system
        semaphore = asyncio.Semaphore(max_concurrent)
        
        async def process_with_semaphore(content_data):
            async with semaphore:
                return await process_single(content_data)
        
        tasks = [process_with_semaphore(content) for content in content_list]
        results = await asyncio.gather(*tasks, return_exceptions=True)
        
        # Filter out None results and exceptions
        valid_results = [
            result for result in results 
            if isinstance(result, ProcessingResult)
        ]
        
        self.logger.info(
            f"Batch processed {len(valid_results)}/{len(content_list)} items successfully"
        )
        
        return valid_results
    
    async def get_content_insights(self, user_id: str, days: int = 30) -> Dict[str, Any]:
        """Get content insights for a user over specified period"""
        try:
            if not self.db_pool:
                return {}
            
            async with self.db_pool.acquire() as conn:
                # Get user's content analytics
                results = await conn.fetch("""
                    SELECT 
                        quality_score, originality_score, engagement_potential,
                        spam_probability, word_count, emoji_count, hashtag_count,
                        sentiment_compound, language, created_at
                    FROM content_analysis 
                    WHERE user_id = $1 AND created_at > NOW() - INTERVAL '%s days'
                    ORDER BY created_at DESC
                """, user_id, days)
                
                if not results:
                    return {}
                
                # Calculate insights
                insights = {
                    'total_posts': len(results),
                    'avg_quality_score': np.mean([r['quality_score'] for r in results]),
                    'avg_originality_score': np.mean([r['originality_score'] for r in results]),
                    'avg_engagement_potential': np.mean([r['engagement_potential'] for r in results]),
                    'spam_rate': np.mean([r['spam_probability'] for r in results]),
                    'avg_word_count': np.mean([r['word_count'] for r in results]),
                    'avg_emoji_usage': np.mean([r['emoji_count'] for r in results]),
                    'avg_hashtag_usage': np.mean([r['hashtag_count'] for r in results]),
                    'sentiment_trend': np.mean([r['sentiment_compound'] for r in results]),
                    'languages_used': list(set(r['language'] for r in results)),
                    'improvement_score': self._calculate_improvement_score(results),
                    'recommendations': self._generate_recommendations(results)
                }
                
                return insights
                
        except Exception as e:
            self.logger.error(f"Content insights generation failed: {str(e)}")
            return {}
    
    def _calculate_improvement_score(self, results: List[Any]) -> float:
        """Calculate user's content improvement over time"""
        if len(results) < 5:
            return 0.0
        
        try:
            # Sort by date (newest first)
            sorted_results = sorted(results, key=lambda x: x['created_at'])
            
            # Compare first half vs second half
            mid_point = len(sorted_results) // 2
            early_scores = [r['quality_score'] for r in sorted_results[:mid_point]]
            recent_scores = [r['quality_score'] for r in sorted_results[mid_point:]]
            
            early_avg = np.mean(early_scores)
            recent_avg = np.mean(recent_scores)
            
            improvement = (recent_avg - early_avg) / max(early_avg, 0.1)
            return max(-1.0, min(1.0, improvement))  # Clamp between -1 and 1
            
        except Exception:
            return 0.0
    
    def _generate_recommendations(self, results: List[Any]) -> List[str]:
        """Generate personalized content recommendations"""
        recommendations = []
        
        try:
            if not results:
                return ["Start creating content to get personalized recommendations!"]
            
            avg_quality = np.mean([r['quality_score'] for r in results])
            avg_engagement = np.mean([r['engagement_potential'] for r in results])
            avg_originality = np.mean([r['originality_score'] for r in results])
            avg_emoji = np.mean([r['emoji_count'] for r in results])
            avg_hashtags = np.mean([r['hashtag_count'] for r in results])
            
            # Quality recommendations
            if avg_quality < 0.6:
                recommendations.append(
                    "Focus on improving content quality by using proper grammar and structure"
                )
            
            # Engagement recommendations
            if avg_engagement < 0.5:
                recommendations.append(
                    "Increase engagement by asking questions and using call-to-action phrases"
                )
            
            # Originality recommendations
            if avg_originality < 0.7:
                recommendations.append(
                    "Try to create more original content rather than repeating similar themes"
                )
            
            # Emoji recommendations
            if avg_emoji < 1:
                recommendations.append(
                    "Consider adding emojis to make your content more expressive and engaging"
                )
            elif avg_emoji > 5:
                recommendations.append(
                    "Try reducing emoji usage for better readability"
                )
            
            # Hashtag recommendations
            if avg_hashtags < 2:
                recommendations.append(
                    "Use more relevant hashtags to increase discoverability"
                )
            elif avg_hashtags > 10:
                recommendations.append(
                    "Reduce hashtag count and focus on the most relevant ones"
                )
            
            if not recommendations:
                recommendations.append(
                    "Great job! Your content quality is excellent. Keep up the good work!"
                )
            
            return recommendations
            
        except Exception as e:
            self.logger.warning(f"Recommendation generation failed: {str(e)}")
            return ["Unable to generate recommendations at this time"]
    
    async def cleanup_resources(self):
        """Clean up resources and connections"""
        try:
            if self.redis_client:
                await self.redis_client.close()
            
            if self.db_pool:
                await self.db_pool.close()
            
            self.logger.info("Text processor resources cleaned up successfully")
            
        except Exception as e:
            self.logger.error(f"Resource cleanup failed: {str(e)}")

# Example usage and configuration
async def main():
    """Example usage of the FinovaTextProcessor"""
    
    # Configuration
    config = {
        'redis_url': 'redis://localhost:6379',
        'database_url': 'postgresql://user:pass@localhost/finova',
        'encryption_key': 'your-encryption-key-here',
        'log_level': 'INFO'
    }
    
    # Initialize processor
    processor = FinovaTextProcessor(config)
    
    # Wait for initialization
    await asyncio.sleep(2)
    
    # Example content processing
    sample_content = """
    Just discovered this amazing new coffee shop in downtown! ☕️✨ 
    The atmosphere is perfect for working and the baristas are so friendly. 
    Has anyone else been there? What's your favorite order? 
    #coffee #downtown #workfriendly #newdiscovery
    """
    
    try:
        result = await processor.process_content(
            text=sample_content,
            platform="instagram",
            user_id="user123",
            content_type="post"
        )
        
        print(f"Quality Score: {result.quality_score:.3f}")
        print(f"Engagement Potential: {result.engagement_potential:.3f}")
        print(f"Originality Score: {result.originality_score:.3f}")
        print(f"Platform Relevance: {result.platform_relevance}")
        
    except Exception as e:
        print(f"Processing failed: {str(e)}")
    
    finally:
        await processor.cleanup_resources()

if __name__ == "__main__":
    asyncio.run(main())
    