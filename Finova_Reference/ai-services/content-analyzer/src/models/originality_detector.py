"""
Finova Network - AI-Powered Content Originality Detector
Enterprise-grade implementation for detecting content originality and preventing AI-generated spam

Author: Finova Network Team
Version: 1.0.0
License: MIT
"""

import hashlib
import json
import logging
import numpy as np
import pickle
import re
import sqlite3
import time
from dataclasses import dataclass
from datetime import datetime, timedelta
from difflib import SequenceMatcher
from pathlib import Path
from typing import Dict, List, Optional, Tuple, Union

import torch
import torch.nn as nn
import torch.nn.functional as F
from sentence_transformers import SentenceTransformer
from sklearn.feature_extraction.text import TfidfVectorizer
from sklearn.metrics.pairwise import cosine_similarity
from transformers import (
    AutoModel, AutoTokenizer, pipeline,
    RobertaForSequenceClassification, RobertaTokenizer
)

# Configure logging
logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)

@dataclass
class OriginalityResult:
    """Container for originality detection results"""
    is_original: bool
    confidence_score: float
    similarity_score: float
    ai_generated_probability: float
    duplicate_sources: List[str]
    plagiarism_matches: List[Dict]
    quality_metrics: Dict[str, float]
    processing_time: float

class ContentFingerprint:
    """Generate unique fingerprints for content deduplication"""
    
    def __init__(self):
        self.shingle_size = 5
        self.hash_size = 64
    
    def generate_shingles(self, text: str) -> List[str]:
        """Generate overlapping n-grams from text"""
        text = re.sub(r'\s+', ' ', text.strip().lower())
        words = text.split()
        return [' '.join(words[i:i+self.shingle_size]) 
                for i in range(len(words) - self.shingle_size + 1)]
    
    def minhash_signature(self, shingles: List[str]) -> List[int]:
        """Generate MinHash signature for similarity detection"""
        if not shingles:
            return [0] * self.hash_size
        
        signature = []
        for i in range(self.hash_size):
            min_hash = float('inf')
            for shingle in shingles:
                hash_val = hash(f"{shingle}_{i}") % (2**32)
                min_hash = min(min_hash, hash_val)
            signature.append(min_hash)
        return signature
    
    def jaccard_similarity(self, sig1: List[int], sig2: List[int]) -> float:
        """Calculate Jaccard similarity between MinHash signatures"""
        if len(sig1) != len(sig2):
            return 0.0
        matches = sum(1 for a, b in zip(sig1, sig2) if a == b)
        return matches / len(sig1)

class AIDetectionModel(nn.Module):
    """Neural network for detecting AI-generated content"""
    
    def __init__(self, input_dim: int = 768, hidden_dim: int = 256):
        super().__init__()
        self.dropout = nn.Dropout(0.3)
        self.fc1 = nn.Linear(input_dim, hidden_dim)
        self.fc2 = nn.Linear(hidden_dim, hidden_dim // 2)
        self.fc3 = nn.Linear(hidden_dim // 2, 1)
        self.sigmoid = nn.Sigmoid()
    
    def forward(self, x):
        x = self.dropout(x)
        x = F.relu(self.fc1(x))
        x = self.dropout(x)
        x = F.relu(self.fc2(x))
        x = self.dropout(x)
        x = self.fc3(x)
        return self.sigmoid(x)

class ContentDatabase:
    """SQLite database for storing content fingerprints and metadata"""
    
    def __init__(self, db_path: str = "finova_content.db"):
        self.db_path = db_path
        self.init_database()
    
    def init_database(self):
        """Initialize database with required tables"""
        with sqlite3.connect(self.db_path) as conn:
            conn.execute("""
                CREATE TABLE IF NOT EXISTS content_fingerprints (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    content_hash TEXT UNIQUE NOT NULL,
                    minhash_signature TEXT NOT NULL,
                    user_id TEXT NOT NULL,
                    platform TEXT NOT NULL,
                    content_type TEXT NOT NULL,
                    timestamp DATETIME NOT NULL,
                    embedding BLOB,
                    quality_score REAL,
                    is_original BOOLEAN,
                    INDEX(content_hash),
                    INDEX(user_id),
                    INDEX(timestamp)
                )
            """)
            
            conn.execute("""
                CREATE TABLE IF NOT EXISTS similarity_cache (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    hash1 TEXT NOT NULL,
                    hash2 TEXT NOT NULL,
                    similarity_score REAL NOT NULL,
                    timestamp DATETIME NOT NULL,
                    INDEX(hash1, hash2)
                )
            """)
    
    def store_fingerprint(self, content_hash: str, signature: List[int], 
                         user_id: str, platform: str, content_type: str,
                         embedding: np.ndarray, quality_score: float,
                         is_original: bool) -> bool:
        """Store content fingerprint in database"""
        try:
            with sqlite3.connect(self.db_path) as conn:
                conn.execute("""
                    INSERT OR REPLACE INTO content_fingerprints 
                    (content_hash, minhash_signature, user_id, platform, 
                     content_type, timestamp, embedding, quality_score, is_original)
                    VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
                """, (
                    content_hash, json.dumps(signature), user_id, platform,
                    content_type, datetime.now(), pickle.dumps(embedding),
                    quality_score, is_original
                ))
            return True
        except Exception as e:
            logger.error(f"Error storing fingerprint: {e}")
            return False
    
    def find_similar_content(self, signature: List[int], 
                           threshold: float = 0.7) -> List[Dict]:
        """Find content with similar MinHash signatures"""
        similar_content = []
        
        with sqlite3.connect(self.db_path) as conn:
            cursor = conn.execute("""
                SELECT content_hash, minhash_signature, user_id, platform,
                       timestamp, quality_score
                FROM content_fingerprints
                WHERE timestamp > ?
                ORDER BY timestamp DESC
                LIMIT 10000
            """, (datetime.now() - timedelta(days=30),))
            
            fingerprint_gen = ContentFingerprint()
            
            for row in cursor:
                stored_sig = json.loads(row[1])
                similarity = fingerprint_gen.jaccard_similarity(signature, stored_sig)
                
                if similarity >= threshold:
                    similar_content.append({
                        'content_hash': row[0],
                        'user_id': row[2],
                        'platform': row[3],
                        'timestamp': row[4],
                        'similarity': similarity,
                        'quality_score': row[5]
                    })
        
        return sorted(similar_content, key=lambda x: x['similarity'], reverse=True)

class OriginalityDetector:
    """Main class for content originality detection"""
    
    def __init__(self, model_path: Optional[str] = None, cache_dir: str = "./models"):
        self.cache_dir = Path(cache_dir)
        self.cache_dir.mkdir(exist_ok=True)
        
        # Initialize components
        self.fingerprint_gen = ContentFingerprint()
        self.database = ContentDatabase()
        
        # Load models
        self._load_models()
        
        # Configuration
        self.similarity_threshold = 0.7
        self.ai_probability_threshold = 0.8
        self.min_content_length = 10
        
        logger.info("OriginalityDetector initialized successfully")
    
    def _load_models(self):
        """Load pre-trained models for analysis"""
        try:
            # Sentence transformer for semantic similarity
            self.sentence_model = SentenceTransformer('all-MiniLM-L6-v2')
            
            # AI detection model (using RoBERTa as base)
            model_name = "roberta-base"
            self.ai_tokenizer = RobertaTokenizer.from_pretrained(model_name)
            self.ai_base_model = RobertaForSequenceClassification.from_pretrained(model_name)
            
            # Custom AI detection head
            self.ai_detector = AIDetectionModel()
            
            # TF-IDF for quick similarity checks
            self.tfidf_vectorizer = TfidfVectorizer(
                max_features=10000,
                stop_words='english',
                ngram_range=(1, 3)
            )
            
            # Language model for text quality assessment
            self.quality_pipeline = pipeline(
                "text-classification",
                model="distilbert-base-uncased-finetuned-sst-2-english",
                device=0 if torch.cuda.is_available() else -1
            )
            
            logger.info("All models loaded successfully")
            
        except Exception as e:
            logger.error(f"Error loading models: {e}")
            raise
    
    def preprocess_content(self, content: str) -> str:
        """Clean and normalize content for analysis"""
        if not content or len(content.strip()) < self.min_content_length:
            return ""
        
        # Remove excessive whitespace
        content = re.sub(r'\s+', ' ', content.strip())
        
        # Remove URLs
        content = re.sub(r'http[s]?://(?:[a-zA-Z]|[0-9]|[$-_@.&+]|[!*\\(\\),]|(?:%[0-9a-fA-F][0-9a-fA-F]))+', '', content)
        
        # Remove mentions and hashtags for similarity comparison
        content_for_similarity = re.sub(r'[@#]\w+', '', content)
        
        # Remove excessive punctuation
        content_for_similarity = re.sub(r'[^\w\s.,!?-]', '', content_for_similarity)
        
        return content_for_similarity.strip()
    
    def detect_ai_generated(self, content: str) -> float:
        """Detect if content is AI-generated using multiple signals"""
        try:
            # Tokenize content
            inputs = self.ai_tokenizer(
                content,
                return_tensors="pt",
                truncation=True,
                max_length=512,
                padding=True
            )
            
            # Get embeddings from base model
            with torch.no_grad():
                outputs = self.ai_base_model(**inputs, output_hidden_states=True)
                embeddings = outputs.hidden_states[-1].mean(dim=1)
            
            # Pass through AI detection model
            ai_probability = self.ai_detector(embeddings).item()
            
            # Additional heuristics for AI detection
            heuristic_score = self._calculate_ai_heuristics(content)
            
            # Combine scores
            final_score = (ai_probability * 0.7) + (heuristic_score * 0.3)
            
            return min(max(final_score, 0.0), 1.0)
            
        except Exception as e:
            logger.error(f"Error in AI detection: {e}")
            return 0.5  # Default to uncertain
    
    def _calculate_ai_heuristics(self, content: str) -> float:
        """Calculate heuristic signals for AI-generated content"""
        signals = []
        
        # 1. Repetitive patterns
        words = content.lower().split()
        if len(words) > 10:
            unique_ratio = len(set(words)) / len(words)
            signals.append(1.0 - unique_ratio)  # Higher repetition = more AI-like
        
        # 2. Unnatural sentence structure
        sentences = re.split(r'[.!?]+', content)
        if len(sentences) > 3:
            lengths = [len(s.split()) for s in sentences if s.strip()]
            if lengths:
                length_variance = np.var(lengths) / (np.mean(lengths) + 1)
                signals.append(1.0 / (1.0 + length_variance))  # Low variance = more AI-like
        
        # 3. Generic phrases detection
        generic_phrases = [
            "in conclusion", "it is important to note", "furthermore",
            "moreover", "in addition", "on the other hand", "however",
            "therefore", "as a result", "consequently"
        ]
        generic_count = sum(1 for phrase in generic_phrases if phrase in content.lower())
        if len(words) > 0:
            signals.append(min(generic_count / len(words) * 100, 1.0))
        
        # 4. Perfect grammar (too perfect can indicate AI)
        # This is a simplified heuristic - in production, use a grammar checker
        punctuation_ratio = len(re.findall(r'[.!?,:;]', content)) / max(len(content), 1)
        if punctuation_ratio > 0.05:  # Very high punctuation usage
            signals.append(0.7)
        else:
            signals.append(0.2)
        
        return np.mean(signals) if signals else 0.5
    
    def calculate_semantic_similarity(self, content: str, comparison_texts: List[str]) -> List[float]:
        """Calculate semantic similarity using sentence transformers"""
        if not comparison_texts:
            return []
        
        try:
            # Generate embeddings
            content_embedding = self.sentence_model.encode([content])
            comparison_embeddings = self.sentence_model.encode(comparison_texts)
            
            # Calculate cosine similarities
            similarities = cosine_similarity(content_embedding, comparison_embeddings)[0]
            
            return similarities.tolist()
            
        except Exception as e:
            logger.error(f"Error calculating semantic similarity: {e}")
            return [0.0] * len(comparison_texts)
    
    def assess_content_quality(self, content: str) -> Dict[str, float]:
        """Assess various quality metrics of the content"""
        metrics = {}
        
        try:
            # 1. Length and structure
            words = content.split()
            sentences = re.split(r'[.!?]+', content)
            
            metrics['word_count'] = len(words)
            metrics['sentence_count'] = len([s for s in sentences if s.strip()])
            metrics['avg_sentence_length'] = metrics['word_count'] / max(metrics['sentence_count'], 1)
            
            # 2. Vocabulary diversity
            if len(words) > 0:
                metrics['vocabulary_diversity'] = len(set(words)) / len(words)
            else:
                metrics['vocabulary_diversity'] = 0.0
            
            # 3. Readability (simplified Flesch score)
            avg_sentence_length = metrics['avg_sentence_length']
            syllable_count = sum(self._count_syllables(word) for word in words)
            avg_syllables_per_word = syllable_count / max(len(words), 1)
            
            flesch_score = 206.835 - (1.015 * avg_sentence_length) - (84.6 * avg_syllables_per_word)
            metrics['readability_score'] = max(0, min(100, flesch_score)) / 100
            
            # 4. Sentiment analysis for engagement potential
            try:
                sentiment_result = self.quality_pipeline(content[:512])
                if sentiment_result and len(sentiment_result) > 0:
                    metrics['sentiment_confidence'] = sentiment_result[0]['score']
                else:
                    metrics['sentiment_confidence'] = 0.5
            except:
                metrics['sentiment_confidence'] = 0.5
            
            # 5. Content coherence (simplified)
            sentences_clean = [s.strip() for s in sentences if s.strip()]
            if len(sentences_clean) > 1:
                sentence_similarities = []
                for i in range(len(sentences_clean) - 1):
                    sim = SequenceMatcher(None, sentences_clean[i], sentences_clean[i+1]).ratio()
                    sentence_similarities.append(sim)
                metrics['coherence_score'] = np.mean(sentence_similarities)
            else:
                metrics['coherence_score'] = 1.0
            
            # 6. Overall quality score
            quality_weights = {
                'vocabulary_diversity': 0.25,
                'readability_score': 0.25,
                'sentiment_confidence': 0.20,
                'coherence_score': 0.30
            }
            
            metrics['overall_quality'] = sum(
                metrics[key] * weight for key, weight in quality_weights.items()
            )
            
        except Exception as e:
            logger.error(f"Error assessing content quality: {e}")
            metrics = {key: 0.5 for key in ['vocabulary_diversity', 'readability_score', 
                                          'sentiment_confidence', 'coherence_score', 'overall_quality']}
        
        return metrics
    
    def _count_syllables(self, word: str) -> int:
        """Simple syllable counting heuristic"""
        word = word.lower()
        vowels = "aeiouy"
        syllable_count = 0
        previous_was_vowel = False
        
        for i, char in enumerate(word):
            is_vowel = char in vowels
            if is_vowel and not previous_was_vowel:
                syllable_count += 1
            previous_was_vowel = is_vowel
        
        # Handle silent 'e'
        if word.endswith('e') and syllable_count > 1:
            syllable_count -= 1
        
        return max(1, syllable_count)
    
    def detect_originality(self, content: str, user_id: str, platform: str,
                          content_type: str = "text") -> OriginalityResult:
        """Main method to detect content originality"""
        start_time = time.time()
        
        # Preprocess content
        processed_content = self.preprocess_content(content)
        if not processed_content:
            return OriginalityResult(
                is_original=False,
                confidence_score=0.0,
                similarity_score=1.0,
                ai_generated_probability=0.0,
                duplicate_sources=[],
                plagiarism_matches=[],
                quality_metrics={},
                processing_time=time.time() - start_time
            )
        
        # Generate content fingerprint
        content_hash = hashlib.sha256(processed_content.encode()).hexdigest()
        shingles = self.fingerprint_gen.generate_shingles(processed_content)
        signature = self.fingerprint_gen.minhash_signature(shingles)
        
        # Find similar content
        similar_content = self.database.find_similar_content(
            signature, self.similarity_threshold
        )
        
        # Calculate AI probability
        ai_probability = self.detect_ai_generated(processed_content)
        
        # Assess content quality
        quality_metrics = self.assess_content_quality(processed_content)
        
        # Calculate semantic similarity for high-confidence matches
        plagiarism_matches = []
        max_similarity = 0.0
        
        if similar_content:
            # Get content for semantic analysis
            similar_texts = []
            for match in similar_content[:5]:  # Check top 5 matches
                # In production, retrieve actual content from database
                similar_texts.append(f"Similar content from {match['platform']}")
            
            if similar_texts:
                semantic_similarities = self.calculate_semantic_similarity(
                    processed_content, similar_texts
                )
                
                for i, match in enumerate(similar_content[:5]):
                    if i < len(semantic_similarities):
                        semantic_sim = semantic_similarities[i]
                        combined_sim = (match['similarity'] + semantic_sim) / 2
                        
                        if combined_sim > 0.6:
                            plagiarism_matches.append({
                                'source': match['platform'],
                                'similarity': combined_sim,
                                'user_id': match['user_id'],
                                'timestamp': match['timestamp']
                            })
                        
                        max_similarity = max(max_similarity, combined_sim)
        
        # Determine originality
        is_original = (
            max_similarity < self.similarity_threshold and
            ai_probability < self.ai_probability_threshold and
            quality_metrics.get('overall_quality', 0) > 0.3
        )
        
        # Calculate confidence score
        confidence_factors = [
            1.0 - max_similarity,  # Lower similarity = higher confidence
            1.0 - ai_probability,  # Lower AI probability = higher confidence
            quality_metrics.get('overall_quality', 0.5),  # Higher quality = higher confidence
            min(len(processed_content) / 100, 1.0)  # Longer content = higher confidence
        ]
        confidence_score = np.mean(confidence_factors)
        
        # Store fingerprint for future comparisons
        embedding = self.sentence_model.encode([processed_content])[0]
        self.database.store_fingerprint(
            content_hash, signature, user_id, platform, content_type,
            embedding, quality_metrics.get('overall_quality', 0.5), is_original
        )
        
        processing_time = time.time() - start_time
        
        return OriginalityResult(
            is_original=is_original,
            confidence_score=confidence_score,
            similarity_score=max_similarity,
            ai_generated_probability=ai_probability,
            duplicate_sources=[match['source'] for match in plagiarism_matches],
            plagiarism_matches=plagiarism_matches,
            quality_metrics=quality_metrics,
            processing_time=processing_time
        )
    
    def bulk_analyze_user_content(self, user_id: str, content_list: List[str]) -> Dict:
        """Analyze multiple pieces of content for patterns"""
        results = []
        
        for content in content_list:
            result = self.detect_originality(content, user_id, "bulk_analysis")
            results.append(result)
        
        # Calculate user-level metrics
        if results:
            avg_originality = np.mean([r.confidence_score for r in results])
            avg_ai_probability = np.mean([r.ai_generated_probability for r in results])
            avg_quality = np.mean([r.quality_metrics.get('overall_quality', 0) for r in results])
            
            user_analysis = {
                'total_content': len(results),
                'original_content': sum(1 for r in results if r.is_original),
                'avg_originality_score': avg_originality,
                'avg_ai_probability': avg_ai_probability,
                'avg_quality_score': avg_quality,
                'suspicious_pattern': avg_ai_probability > 0.7 or avg_originality < 0.3,
                'individual_results': results
            }
        else:
            user_analysis = {
                'total_content': 0,
                'original_content': 0,
                'avg_originality_score': 0.0,
                'avg_ai_probability': 0.0,
                'avg_quality_score': 0.0,
                'suspicious_pattern': False,
                'individual_results': []
            }
        
        return user_analysis
    
    def get_user_reputation_score(self, user_id: str, days: int = 30) -> float:
        """Calculate user reputation based on historical content quality"""
        try:
            with sqlite3.connect(self.database.db_path) as conn:
                cursor = conn.execute("""
                    SELECT quality_score, is_original
                    FROM content_fingerprints
                    WHERE user_id = ? AND timestamp > ?
                    ORDER BY timestamp DESC
                """, (user_id, datetime.now() - timedelta(days=days)))
                
                records = cursor.fetchall()
                
                if not records:
                    return 0.5  # Neutral reputation for new users
                
                quality_scores = [r[0] for r in records if r[0] is not None]
                original_content = sum(1 for r in records if r[1])
                
                if quality_scores:
                    avg_quality = np.mean(quality_scores)
                    originality_ratio = original_content / len(records)
                    
                    # Weighted reputation score
                    reputation = (avg_quality * 0.6) + (originality_ratio * 0.4)
                    return min(max(reputation, 0.0), 1.0)
                
                return 0.5
                
        except Exception as e:
            logger.error(f"Error calculating user reputation: {e}")
            return 0.5

# Example usage and testing
if __name__ == "__main__":
    # Initialize detector
    detector = OriginalityDetector()
    
    # Test content
    test_content = """
    Artificial intelligence is revolutionizing the way we interact with technology.
    From smart assistants to autonomous vehicles, AI is becoming increasingly 
    integrated into our daily lives. This transformation brings both opportunities
    and challenges that we must carefully navigate.
    """
    
    # Analyze content
    result = detector.detect_originality(
        content=test_content,
        user_id="test_user_123",
        platform="finova",
        content_type="social_post"
    )
    
    # Print results
    print(f"Content Analysis Results:")
    print(f"Original: {result.is_original}")
    print(f"Confidence: {result.confidence_score:.3f}")
    print(f"AI Probability: {result.ai_generated_probability:.3f}")
    print(f"Similarity Score: {result.similarity_score:.3f}")
    print(f"Quality Score: {result.quality_metrics.get('overall_quality', 0):.3f}")
    print(f"Processing Time: {result.processing_time:.3f}s")
    
    if result.plagiarism_matches:
        print(f"Potential matches found: {len(result.plagiarism_matches)}")
        for match in result.plagiarism_matches:
            print(f"  - {match['source']}: {match['similarity']:.3f}")
            