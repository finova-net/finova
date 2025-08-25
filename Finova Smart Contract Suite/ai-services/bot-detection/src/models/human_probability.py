"""
Finova Network - AI Bot Detection: Human Probability Model
Enterprise-grade implementation for detecting genuine human users vs bots
"""

import numpy as np
import pandas as pd
from typing import Dict, List, Optional, Tuple, Any
from dataclasses import dataclass, field
from datetime import datetime, timedelta
import hashlib
import json
import logging
from scipy import stats
from sklearn.ensemble import IsolationForest
from sklearn.preprocessing import StandardScaler
import asyncio
import redis
from cryptography.fernet import Fernet


# Configure logging
logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)


@dataclass
class UserBehaviorData:
    """Comprehensive user behavior data structure"""
    user_id: str
    session_id: str
    timestamp: datetime
    
    # Biometric consistency data
    selfie_embeddings: List[float] = field(default_factory=list)
    face_landmarks: Dict[str, float] = field(default_factory=dict)
    biometric_variance: float = 0.0
    
    # Behavioral patterns
    click_intervals: List[float] = field(default_factory=list)
    typing_patterns: List[float] = field(default_factory=list)
    scroll_velocity: List[float] = field(default_factory=list)
    session_duration: float = 0.0
    activity_breaks: List[float] = field(default_factory=list)
    
    # Social graph data
    referral_network: Dict[str, Any] = field(default_factory=dict)
    connection_quality: float = 0.0
    interaction_authenticity: float = 0.0
    
    # Device fingerprint
    device_fingerprint: Dict[str, str] = field(default_factory=dict)
    location_consistency: float = 0.0
    device_stability: float = 0.0
    
    # Content interaction quality
    content_originality: float = 0.0
    engagement_patterns: Dict[str, float] = field(default_factory=dict)
    content_quality_scores: List[float] = field(default_factory=list)


class BiometricAnalyzer:
    """Analyzes biometric consistency for human verification"""
    
    def __init__(self):
        self.face_model_threshold = 0.85
        self.variance_threshold = 0.3
        
    def analyze_selfie_consistency(self, user_data: UserBehaviorData) -> float:
        """Analyze facial recognition consistency across sessions"""
        try:
            if len(user_data.selfie_embeddings) < 2:
                return 0.5  # Insufficient data
            
            # Calculate embedding similarity
            embeddings = np.array(user_data.selfie_embeddings).reshape(-1, 512)  # Assuming 512-dim embeddings
            similarities = []
            
            for i in range(len(embeddings)):
                for j in range(i + 1, len(embeddings)):
                    cosine_sim = np.dot(embeddings[i], embeddings[j]) / (
                        np.linalg.norm(embeddings[i]) * np.linalg.norm(embeddings[j])
                    )
                    similarities.append(cosine_sim)
            
            avg_similarity = np.mean(similarities)
            consistency_score = min(1.0, avg_similarity / self.face_model_threshold)
            
            # Penalty for high variance (possible face swapping)
            if user_data.biometric_variance > self.variance_threshold:
                consistency_score *= 0.6
            
            return max(0.1, consistency_score)
            
        except Exception as e:
            logger.error(f"Biometric analysis error: {e}")
            return 0.3


class BehavioralAnalyzer:
    """Analyzes human behavioral patterns"""
    
    def __init__(self):
        self.human_click_mean = 0.3  # seconds
        self.human_click_std = 0.15
        self.circadian_weight = 0.2
        
    def detect_human_rhythms(self, user_data: UserBehaviorData) -> float:
        """Detect natural human behavioral rhythms"""
        try:
            score = 0.0
            
            # 1. Click interval analysis
            if user_data.click_intervals:
                click_score = self._analyze_click_patterns(user_data.click_intervals)
                score += click_score * 0.3
            
            # 2. Typing rhythm analysis
            if user_data.typing_patterns:
                typing_score = self._analyze_typing_patterns(user_data.typing_patterns)
                score += typing_score * 0.25
            
            # 3. Session pattern analysis
            session_score = self._analyze_session_patterns(user_data)
            score += session_score * 0.25
            
            # 4. Activity break patterns
            if user_data.activity_breaks:
                break_score = self._analyze_break_patterns(user_data.activity_breaks)
                score += break_score * 0.2
            
            return max(0.1, min(1.0, score))
            
        except Exception as e:
            logger.error(f"Behavioral analysis error: {e}")
            return 0.4
    
    def _analyze_click_patterns(self, click_intervals: List[float]) -> float:
        """Analyze naturalness of click timing"""
        if len(click_intervals) < 5:
            return 0.5
        
        # Convert to numpy array
        intervals = np.array(click_intervals)
        
        # Calculate statistics
        mean_interval = np.mean(intervals)
        std_interval = np.std(intervals)
        
        # Human clicks have natural variance
        if std_interval < 0.05:  # Too consistent = bot
            return 0.2
        
        # Check if distribution matches human patterns
        expected_mean = self.human_click_mean
        expected_std = self.human_click_std
        
        mean_score = 1.0 - abs(mean_interval - expected_mean) / expected_mean
        std_score = 1.0 - abs(std_interval - expected_std) / expected_std
        
        return max(0.1, (mean_score + std_score) / 2)
    
    def _analyze_typing_patterns(self, typing_patterns: List[float]) -> float:
        """Analyze typing rhythm naturalness"""
        if len(typing_patterns) < 10:
            return 0.5
        
        patterns = np.array(typing_patterns)
        
        # Check for human-like variance in typing speed
        cv = np.std(patterns) / np.mean(patterns)  # Coefficient of variation
        
        # Human typing has moderate variance (0.2-0.8)
        if 0.2 <= cv <= 0.8:
            return 0.8
        elif cv < 0.2:  # Too consistent
            return 0.3
        else:  # Too erratic
            return 0.5
    
    def _analyze_session_patterns(self, user_data: UserBehaviorData) -> float:
        """Analyze session duration and patterns"""
        session_duration = user_data.session_duration
        
        # Human sessions typically 2-45 minutes
        if 120 <= session_duration <= 2700:  # 2-45 minutes in seconds
            return 0.9
        elif session_duration < 60:  # Too short
            return 0.4
        elif session_duration > 7200:  # Too long (>2 hours)
            return 0.6
        else:
            return 0.7
    
    def _analyze_break_patterns(self, activity_breaks: List[float]) -> float:
        """Analyze natural break patterns"""
        if len(activity_breaks) < 3:
            return 0.5
        
        breaks = np.array(activity_breaks)
        
        # Humans take irregular breaks
        variance = np.var(breaks)
        
        # Good variance indicates human behavior
        if 10 <= variance <= 1000:  # Reasonable variance in seconds
            return 0.8
        else:
            return 0.4


class SocialGraphAnalyzer:
    """Analyzes social connection authenticity"""
    
    def __init__(self):
        self.min_connections = 3
        self.quality_threshold = 0.6
        
    def validate_real_connections(self, user_data: UserBehaviorData) -> float:
        """Validate authenticity of social connections"""
        try:
            network = user_data.referral_network
            
            if not network or len(network.get('connections', [])) < self.min_connections:
                return 0.3
            
            # Calculate network authenticity score
            authenticity_score = 0.0
            
            # 1. Connection diversity
            diversity_score = self._calculate_connection_diversity(network)
            authenticity_score += diversity_score * 0.3
            
            # 2. Interaction quality
            interaction_score = user_data.interaction_authenticity
            authenticity_score += interaction_score * 0.4
            
            # 3. Network growth pattern
            growth_score = self._analyze_network_growth(network)
            authenticity_score += growth_score * 0.3
            
            return max(0.1, min(1.0, authenticity_score))
            
        except Exception as e:
            logger.error(f"Social graph analysis error: {e}")
            return 0.4
    
    def _calculate_connection_diversity(self, network: Dict[str, Any]) -> float:
        """Calculate diversity of connections"""
        connections = network.get('connections', [])
        
        if len(connections) < 3:
            return 0.2
        
        # Analyze connection metadata diversity
        locations = set()
        join_times = []
        
        for conn in connections:
            if 'location' in conn:
                locations.add(conn['location'])
            if 'join_time' in conn:
                join_times.append(conn['join_time'])
        
        # Diversity score based on location spread and join time distribution
        location_diversity = min(1.0, len(locations) / len(connections))
        
        # Time diversity (connections joined at different times)
        if len(join_times) > 1:
            time_variance = np.var(join_times)
            time_diversity = min(1.0, time_variance / 86400)  # Normalize by day
        else:
            time_diversity = 0.1
        
        return (location_diversity + time_diversity) / 2
    
    def _analyze_network_growth(self, network: Dict[str, Any]) -> float:
        """Analyze natural vs artificial network growth"""
        growth_pattern = network.get('growth_history', [])
        
        if len(growth_pattern) < 2:
            return 0.5
        
        # Natural growth is gradual, not sudden spikes
        growth_rates = []
        for i in range(1, len(growth_pattern)):
            rate = growth_pattern[i] - growth_pattern[i-1]
            growth_rates.append(rate)
        
        # Check for suspicious spikes
        if max(growth_rates) > np.mean(growth_rates) * 5:
            return 0.3  # Suspicious spike
        
        return 0.8  # Natural growth


class DeviceAnalyzer:
    """Analyzes device fingerprint authenticity"""
    
    def __init__(self):
        self.stability_threshold = 0.8
        
    def check_device_fingerprint(self, user_data: UserBehaviorData) -> float:
        """Check device fingerprint consistency"""
        try:
            fingerprint = user_data.device_fingerprint
            
            if not fingerprint:
                return 0.2
            
            score = 0.0
            
            # 1. Device stability
            stability = user_data.device_stability
            score += min(1.0, stability / self.stability_threshold) * 0.4
            
            # 2. Location consistency
            location_score = user_data.location_consistency
            score += location_score * 0.3
            
            # 3. Hardware authenticity
            hardware_score = self._check_hardware_authenticity(fingerprint)
            score += hardware_score * 0.3
            
            return max(0.1, score)
            
        except Exception as e:
            logger.error(f"Device analysis error: {e}")
            return 0.4
    
    def _check_hardware_authenticity(self, fingerprint: Dict[str, str]) -> float:
        """Check if hardware fingerprint looks authentic"""
        # Check for common emulator/bot signatures
        suspicious_patterns = [
            'android_sdk',
            'bluestacks',
            'nox',
            'headless',
            'phantom'
        ]
        
        device_info = str(fingerprint).lower()
        
        for pattern in suspicious_patterns:
            if pattern in device_info:
                return 0.2
        
        # Check for reasonable hardware specs
        if 'cpu' in fingerprint and 'memory' in fingerprint:
            return 0.8
        
        return 0.6


class ContentQualityAnalyzer:
    """Analyzes content interaction quality"""
    
    def __init__(self):
        self.originality_threshold = 0.7
        
    def measure_content_uniqueness(self, user_data: UserBehaviorData) -> float:
        """Measure uniqueness and quality of user content"""
        try:
            originality = user_data.content_originality
            quality_scores = user_data.content_quality_scores
            engagement = user_data.engagement_patterns
            
            if not quality_scores:
                return 0.4
            
            # Calculate overall content quality
            avg_quality = np.mean(quality_scores)
            
            # Bonus for originality
            originality_bonus = 1.0 if originality >= self.originality_threshold else 0.7
            
            # Engagement authenticity
            engagement_score = engagement.get('authenticity', 0.5)
            
            final_score = avg_quality * originality_bonus * engagement_score
            
            return max(0.1, min(1.0, final_score))
            
        except Exception as e:
            logger.error(f"Content quality analysis error: {e}")
            return 0.4


class HumanProbabilityModel:
    """Main model for calculating human probability"""
    
    def __init__(self, redis_client: Optional[redis.Redis] = None):
        # Initialize analyzers
        self.biometric_analyzer = BiometricAnalyzer()
        self.behavioral_analyzer = BehavioralAnalyzer()
        self.social_analyzer = SocialGraphAnalyzer()
        self.device_analyzer = DeviceAnalyzer()
        self.content_analyzer = ContentQualityAnalyzer()
        
        # Redis for caching
        self.redis_client = redis_client
        
        # Model weights (sum to 1.0)
        self.weights = {
            'biometric_consistency': 0.25,
            'behavioral_patterns': 0.25,
            'social_graph_validity': 0.20,
            'device_authenticity': 0.15,
            'interaction_quality': 0.15
        }
        
        # Anomaly detection model
        self.anomaly_detector = IsolationForest(contamination=0.1, random_state=42)
        self.scaler = StandardScaler()
        
        # Historical data for model training
        self.historical_data = []
        
    def calculate_human_probability(self, user_data: UserBehaviorData) -> Dict[str, float]:
        """Calculate comprehensive human probability score"""
        try:
            # Check cache first
            cache_key = f"human_prob:{user_data.user_id}:{user_data.session_id}"
            
            if self.redis_client:
                cached_result = self.redis_client.get(cache_key)
                if cached_result:
                    return json.loads(cached_result)
            
            # Calculate individual factor scores
            factors = {}
            
            # 1. Biometric consistency
            factors['biometric_consistency'] = self.biometric_analyzer.analyze_selfie_consistency(user_data)
            
            # 2. Behavioral patterns
            factors['behavioral_patterns'] = self.behavioral_analyzer.detect_human_rhythms(user_data)
            
            # 3. Social graph validity
            factors['social_graph_validity'] = self.social_analyzer.validate_real_connections(user_data)
            
            # 4. Device authenticity
            factors['device_authenticity'] = self.device_analyzer.check_device_fingerprint(user_data)
            
            # 5. Interaction quality
            factors['interaction_quality'] = self.content_analyzer.measure_content_uniqueness(user_data)
            
            # Calculate weighted score
            weighted_score = sum(factors[key] * self.weights[key] for key in factors)
            
            # Apply anomaly detection
            anomaly_adjustment = self._apply_anomaly_detection(factors)
            
            # Final score with bounds
            final_score = max(0.1, min(1.0, weighted_score * anomaly_adjustment))
            
            # Prepare result
            result = {
                'human_probability': final_score,
                'confidence': self._calculate_confidence(factors),
                'factors': factors,
                'risk_level': self._determine_risk_level(final_score),
                'anomaly_score': anomaly_adjustment,
                'timestamp': datetime.now().isoformat()
            }
            
            # Cache result
            if self.redis_client:
                self.redis_client.setex(
                    cache_key,
                    timedelta(minutes=15),
                    json.dumps(result, default=str)
                )
            
            # Store for model training
            self._update_historical_data(user_data, result)
            
            return result
            
        except Exception as e:
            logger.error(f"Human probability calculation error: {e}")
            return {
                'human_probability': 0.5,
                'confidence': 0.0,
                'factors': {},
                'risk_level': 'unknown',
                'error': str(e)
            }
    
    def _apply_anomaly_detection(self, factors: Dict[str, float]) -> float:
        """Apply anomaly detection to factor scores"""
        try:
            factor_vector = np.array(list(factors.values())).reshape(1, -1)
            
            if len(self.historical_data) > 100:
                # Use trained anomaly detector
                anomaly_score = self.anomaly_detector.decision_function(factor_vector)[0]
                # Convert to multiplier (normal = 1.0, anomalous = 0.5-0.8)
                return max(0.5, min(1.0, (anomaly_score + 1) / 2))
            else:
                return 1.0  # Not enough data for anomaly detection
                
        except Exception as e:
            logger.warning(f"Anomaly detection failed: {e}")
            return 1.0
    
    def _calculate_confidence(self, factors: Dict[str, float]) -> float:
        """Calculate confidence in the human probability score"""
        # Confidence based on factor consistency and data availability
        factor_variance = np.var(list(factors.values()))
        
        # Lower variance = higher confidence
        confidence = max(0.1, 1.0 - factor_variance)
        
        # Adjust for data availability
        non_zero_factors = sum(1 for score in factors.values() if score > 0.1)
        completeness = non_zero_factors / len(factors)
        
        return confidence * completeness
    
    def _determine_risk_level(self, probability: float) -> str:
        """Determine risk level based on human probability"""
        if probability >= 0.8:
            return 'low'
        elif probability >= 0.6:
            return 'medium'
        elif probability >= 0.4:
            return 'high'
        else:
            return 'critical'
    
    def _update_historical_data(self, user_data: UserBehaviorData, result: Dict[str, float]):
        """Update historical data for model improvement"""
        try:
            data_point = {
                'factors': result['factors'],
                'human_probability': result['human_probability'],
                'timestamp': datetime.now()
            }
            
            self.historical_data.append(data_point)
            
            # Limit historical data size
            if len(self.historical_data) > 10000:
                self.historical_data = self.historical_data[-8000:]  # Keep recent 8000
            
            # Retrain anomaly detector periodically
            if len(self.historical_data) % 1000 == 0:
                self._retrain_anomaly_detector()
                
        except Exception as e:
            logger.warning(f"Historical data update failed: {e}")
    
    def _retrain_anomaly_detector(self):
        """Retrain anomaly detection model with new data"""
        try:
            if len(self.historical_data) < 100:
                return
            
            # Prepare training data
            factor_matrix = []
            for data_point in self.historical_data[-5000:]:  # Use recent 5000 samples
                factor_vector = list(data_point['factors'].values())
                factor_matrix.append(factor_vector)
            
            factor_matrix = np.array(factor_matrix)
            
            # Scale data
            factor_matrix_scaled = self.scaler.fit_transform(factor_matrix)
            
            # Train anomaly detector
            self.anomaly_detector.fit(factor_matrix_scaled)
            
            logger.info(f"Anomaly detector retrained with {len(factor_matrix)} samples")
            
        except Exception as e:
            logger.error(f"Anomaly detector retraining failed: {e}")
    
    async def batch_calculate_probabilities(self, user_data_list: List[UserBehaviorData]) -> List[Dict[str, float]]:
        """Calculate human probabilities for multiple users efficiently"""
        results = []
        
        # Process in batches for better performance
        batch_size = 50
        for i in range(0, len(user_data_list), batch_size):
            batch = user_data_list[i:i + batch_size]
            
            # Create tasks for concurrent processing
            tasks = [
                asyncio.create_task(self._async_calculate_single(user_data))
                for user_data in batch
            ]
            
            # Wait for batch completion
            batch_results = await asyncio.gather(*tasks, return_exceptions=True)
            
            # Handle results and exceptions
            for result in batch_results:
                if isinstance(result, Exception):
                    logger.error(f"Batch calculation error: {result}")
                    results.append({'human_probability': 0.5, 'error': str(result)})
                else:
                    results.append(result)
        
        return results
    
    async def _async_calculate_single(self, user_data: UserBehaviorData) -> Dict[str, float]:
        """Async wrapper for single probability calculation"""
        return self.calculate_human_probability(user_data)
    
    def get_model_statistics(self) -> Dict[str, Any]:
        """Get model performance statistics"""
        if not self.historical_data:
            return {'message': 'No historical data available'}
        
        recent_data = self.historical_data[-1000:]  # Last 1000 samples
        probabilities = [d['human_probability'] for d in recent_data]
        
        return {
            'total_samples': len(self.historical_data),
            'recent_samples': len(recent_data),
            'mean_probability': np.mean(probabilities),
            'std_probability': np.std(probabilities),
            'risk_distribution': {
                'low': sum(1 for p in probabilities if p >= 0.8),
                'medium': sum(1 for p in probabilities if 0.6 <= p < 0.8),
                'high': sum(1 for p in probabilities if 0.4 <= p < 0.6),
                'critical': sum(1 for p in probabilities if p < 0.4)
            },
            'model_weights': self.weights,
            'last_updated': datetime.now().isoformat()
        }


# Example usage and testing
if __name__ == "__main__":
    # Initialize model
    model = HumanProbabilityModel()
    
    # Create test data
    test_user_data = UserBehaviorData(
        user_id="user_12345",
        session_id="session_67890",
        timestamp=datetime.now(),
        selfie_embeddings=[0.1] * 512,  # Mock embedding
        biometric_variance=0.15,
        click_intervals=[0.25, 0.32, 0.28, 0.35, 0.30],
        typing_patterns=[0.15, 0.18, 0.12, 0.20, 0.16],
        session_duration=1800,  # 30 minutes
        activity_breaks=[300, 450, 600],  # 5, 7.5, 10 minute breaks
        referral_network={
            'connections': [
                {'location': 'Jakarta', 'join_time': 1627849200},
                {'location': 'Surabaya', 'join_time': 1627935600},
                {'location': 'Bandung', 'join_time': 1628022000}
            ],
            'growth_history': [0, 1, 2, 3]
        },
        connection_quality=0.8,
        interaction_authenticity=0.7,
        device_fingerprint={'cpu': 'ARM64', 'memory': '8GB'},
        location_consistency=0.9,
        device_stability=0.85,
        content_originality=0.75,
        engagement_patterns={'authenticity': 0.8},
        content_quality_scores=[0.7, 0.8, 0.75, 0.85]
    )
    
    # Calculate human probability
    result = model.calculate_human_probability(test_user_data)
    
    print("Human Probability Analysis Result:")
    print(json.dumps(result, indent=2, default=str))
    
    # Get model statistics
    stats = model.get_model_statistics()
    print("\nModel Statistics:")
    print(json.dumps(stats, indent=2, default=str))