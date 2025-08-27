"""
Finova Network - Behavior Analyzer
Enterprise-grade bot detection through behavioral pattern analysis
"""

import numpy as np
import pandas as pd
from datetime import datetime, timedelta
from typing import Dict, List, Optional, Tuple, Any
from dataclasses import dataclass
from scipy import stats
from sklearn.ensemble import IsolationForest
from sklearn.preprocessing import StandardScaler
import asyncio
import logging

@dataclass
class BehaviorMetrics:
    """Core behavioral metrics for analysis"""
    user_id: str
    session_duration: float
    click_intervals: List[float]
    typing_patterns: List[float]
    mouse_movements: List[Tuple[float, float]]
    scroll_patterns: List[float]
    activity_timestamps: List[datetime]
    content_quality_scores: List[float]
    social_interactions: List[Dict[str, Any]]
    device_fingerprint: Dict[str, str]

@dataclass
class BehaviorScore:
    """Behavior analysis results"""
    human_probability: float
    risk_level: str
    confidence: float
    anomaly_flags: List[str]
    behavioral_patterns: Dict[str, float]
    recommendation: str

class TemporalAnalyzer:
    """Analyzes temporal patterns in user behavior"""
    
    def __init__(self):
        self.logger = logging.getLogger(__name__)
    
    def analyze_circadian_rhythm(self, timestamps: List[datetime]) -> float:
        """Detect natural human activity patterns"""
        if len(timestamps) < 10:
            return 0.5
            
        hours = [ts.hour for ts in timestamps]
        
        # Expected human activity distribution (higher during day, lower at night)
        expected_distribution = np.array([
            0.02, 0.01, 0.01, 0.01, 0.02, 0.03,  # 0-5 AM
            0.06, 0.08, 0.09, 0.08, 0.07, 0.06,  # 6-11 AM  
            0.07, 0.08, 0.09, 0.08, 0.07, 0.06,  # 12-5 PM
            0.05, 0.04, 0.03, 0.03, 0.02, 0.02   # 6-11 PM
        ])
        
        actual_distribution = np.histogram(hours, bins=24, range=(0, 24))[0]
        actual_distribution = actual_distribution / np.sum(actual_distribution)
        
        # Calculate similarity to human pattern
        similarity = 1 - np.sum(np.abs(expected_distribution - actual_distribution)) / 2
        return max(0.1, min(1.0, similarity))
    
    def analyze_session_patterns(self, metrics: BehaviorMetrics) -> float:
        """Analyze session duration and frequency patterns"""
        if not metrics.activity_timestamps:
            return 0.3
            
        # Group activities by day
        daily_sessions = {}
        for ts in metrics.activity_timestamps:
            date_key = ts.date()
            if date_key not in daily_sessions:
                daily_sessions[date_key] = []
            daily_sessions[date_key].append(ts)
        
        session_durations = []
        session_gaps = []
        
        for date, sessions in daily_sessions.items():
            if len(sessions) > 1:
                sessions.sort()
                # Calculate session duration
                duration = (sessions[-1] - sessions[0]).total_seconds() / 3600
                session_durations.append(duration)
                
                # Calculate gaps between activities
                for i in range(1, len(sessions)):
                    gap = (sessions[i] - sessions[i-1]).total_seconds() / 60
                    session_gaps.append(gap)
        
        if not session_durations:
            return 0.3
            
        # Human-like patterns: varied session durations, natural breaks
        duration_variance = np.var(session_durations)
        avg_gap = np.mean(session_gaps) if session_gaps else 0
        
        # Score based on variance and natural breaks
        duration_score = min(1.0, duration_variance / 4.0)  # Higher variance = more human
        gap_score = min(1.0, max(0.1, avg_gap / 30.0))  # Natural breaks expected
        
        return (duration_score + gap_score) / 2

class InteractionAnalyzer:
    """Analyzes user interaction patterns"""
    
    def analyze_click_patterns(self, click_intervals: List[float]) -> float:
        """Detect human-like clicking patterns"""
        if len(click_intervals) < 5:
            return 0.5
            
        # Human clicks have natural variance
        mean_interval = np.mean(click_intervals)
        std_interval = np.std(click_intervals)
        
        # Bots typically have very consistent intervals
        if std_interval < 0.01:  # Too consistent
            return 0.2
            
        # Human-like range: 0.5-3 seconds with good variance
        if 0.5 <= mean_interval <= 3.0 and std_interval > 0.1:
            return 0.9
        elif mean_interval > 3.0:  # Slow but human-like
            return 0.7
        else:  # Too fast
            return 0.3
    
    def analyze_typing_patterns(self, typing_patterns: List[float]) -> float:
        """Analyze typing rhythm and patterns"""
        if len(typing_patterns) < 10:
            return 0.5
            
        # Human typing has characteristic patterns
        intervals = np.array(typing_patterns)
        
        # Check for natural variations
        cv = np.std(intervals) / np.mean(intervals) if np.mean(intervals) > 0 else 0
        
        # Human typing coefficient of variation typically 0.3-0.8
        if 0.3 <= cv <= 0.8:
            return 0.9
        elif cv < 0.1:  # Too consistent (bot-like)
            return 0.2
        else:
            return 0.6
    
    def analyze_mouse_movements(self, movements: List[Tuple[float, float]]) -> float:
        """Analyze mouse movement naturalness"""
        if len(movements) < 20:
            return 0.5
            
        # Calculate movement vectors
        vectors = []
        for i in range(1, len(movements)):
            dx = movements[i][0] - movements[i-1][0]
            dy = movements[i][1] - movements[i-1][1]
            distance = np.sqrt(dx**2 + dy**2)
            if distance > 0:
                vectors.append(distance)
        
        if not vectors:
            return 0.3
            
        # Human mouse movements show natural curves and variations
        movement_variance = np.var(vectors)
        
        # Straight lines (bot-like) vs curved movements (human-like)
        direction_changes = 0
        for i in range(2, len(movements)):
            v1 = np.array([movements[i-1][0] - movements[i-2][0], 
                          movements[i-1][1] - movements[i-2][1]])
            v2 = np.array([movements[i][0] - movements[i-1][0], 
                          movements[i][1] - movements[i-1][1]])
            
            if np.linalg.norm(v1) > 0 and np.linalg.norm(v2) > 0:
                angle = np.arccos(np.clip(np.dot(v1, v2) / 
                                        (np.linalg.norm(v1) * np.linalg.norm(v2)), 
                                        -1.0, 1.0))
                if angle > np.pi / 4:  # Significant direction change
                    direction_changes += 1
        
        change_ratio = direction_changes / len(movements)
        
        # Score based on variance and direction changes
        variance_score = min(1.0, movement_variance / 1000.0)
        change_score = min(1.0, change_ratio * 10)
        
        return (variance_score + change_score) / 2

class ContentQualityAnalyzer:
    """Analyzes content quality patterns"""
    
    def analyze_content_diversity(self, quality_scores: List[float], 
                                 social_interactions: List[Dict[str, Any]]) -> float:
        """Analyze content quality and diversity patterns"""
        if not quality_scores:
            return 0.5
            
        # Human content typically shows variety in quality
        quality_variance = np.var(quality_scores)
        avg_quality = np.mean(quality_scores)
        
        # Extract content types from interactions
        content_types = set()
        for interaction in social_interactions:
            content_type = interaction.get('content_type', 'unknown')
            content_types.add(content_type)
        
        diversity_score = min(1.0, len(content_types) / 5.0)  # Max 5 types
        quality_score = min(1.0, avg_quality)
        variance_score = min(1.0, quality_variance * 2)  # Humans show variance
        
        return (diversity_score + quality_score + variance_score) / 3
    
    def detect_spam_patterns(self, social_interactions: List[Dict[str, Any]]) -> float:
        """Detect spam-like content patterns"""
        if not social_interactions:
            return 0.5
            
        # Analyze posting frequency and content similarity
        post_timestamps = []
        content_lengths = []
        
        for interaction in social_interactions:
            if interaction.get('action') == 'post':
                timestamp = interaction.get('timestamp')
                content = interaction.get('content', '')
                
                if timestamp:
                    post_timestamps.append(timestamp)
                content_lengths.append(len(content))
        
        if len(post_timestamps) < 2:
            return 0.7
            
        # Check for excessive posting frequency
        time_diffs = []
        for i in range(1, len(post_timestamps)):
            diff = (post_timestamps[i] - post_timestamps[i-1]).total_seconds()
            time_diffs.append(diff)
        
        avg_interval = np.mean(time_diffs)
        
        # Spam indicators
        if avg_interval < 60:  # Posts every minute
            return 0.1
        elif avg_interval < 300:  # Posts every 5 minutes
            return 0.3
        
        # Content length analysis
        length_variance = np.var(content_lengths) if content_lengths else 0
        
        # Bots often post very similar length content
        if length_variance < 10 and len(content_lengths) > 5:
            return 0.3
        
        return 0.8

class AnomalyDetector:
    """Machine learning based anomaly detection"""
    
    def __init__(self):
        self.isolation_forest = IsolationForest(contamination=0.1, random_state=42)
        self.scaler = StandardScaler()
        self.is_trained = False
    
    def extract_features(self, metrics: BehaviorMetrics) -> np.ndarray:
        """Extract numerical features for ML analysis"""
        features = []
        
        # Temporal features
        if metrics.activity_timestamps:
            hour_variance = np.var([ts.hour for ts in metrics.activity_timestamps])
            features.append(hour_variance)
            
            # Activity frequency
            total_hours = (metrics.activity_timestamps[-1] - 
                          metrics.activity_timestamps[0]).total_seconds() / 3600
            frequency = len(metrics.activity_timestamps) / max(1, total_hours)
            features.append(frequency)
        else:
            features.extend([0, 0])
        
        # Interaction features
        features.append(np.mean(metrics.click_intervals) if metrics.click_intervals else 0)
        features.append(np.std(metrics.click_intervals) if metrics.click_intervals else 0)
        features.append(np.mean(metrics.typing_patterns) if metrics.typing_patterns else 0)
        features.append(np.std(metrics.typing_patterns) if metrics.typing_patterns else 0)
        
        # Content features
        features.append(np.mean(metrics.content_quality_scores) if metrics.content_quality_scores else 0)
        features.append(len(metrics.social_interactions))
        
        # Session features
        features.append(metrics.session_duration)
        
        return np.array(features).reshape(1, -1)
    
    def train(self, training_metrics: List[BehaviorMetrics]):
        """Train the anomaly detection model"""
        features_list = []
        for metrics in training_metrics:
            features = self.extract_features(metrics)
            features_list.append(features.flatten())
        
        if features_list:
            X = np.array(features_list)
            X_scaled = self.scaler.fit_transform(X)
            self.isolation_forest.fit(X_scaled)
            self.is_trained = True
    
    def detect_anomaly(self, metrics: BehaviorMetrics) -> Tuple[float, List[str]]:
        """Detect behavioral anomalies"""
        if not self.is_trained:
            return 0.5, ["Model not trained"]
            
        features = self.extract_features(metrics)
        features_scaled = self.scaler.transform(features)
        
        # Get anomaly score (-1 for outliers, 1 for inliers)
        anomaly_score = self.isolation_forest.decision_function(features_scaled)[0]
        is_outlier = self.isolation_forest.predict(features_scaled)[0] == -1
        
        # Convert to probability (0-1, where 1 is normal)
        probability = (anomaly_score + 1) / 2
        
        flags = []
        if is_outlier:
            flags.append("Statistical outlier detected")
        if probability < 0.3:
            flags.append("Highly anomalous behavior")
        elif probability < 0.6:
            flags.append("Potentially suspicious behavior")
            
        return probability, flags

class BehaviorAnalyzer:
    """Main behavior analysis orchestrator"""
    
    def __init__(self):
        self.temporal_analyzer = TemporalAnalyzer()
        self.interaction_analyzer = InteractionAnalyzer()
        self.content_analyzer = ContentQualityAnalyzer()
        self.anomaly_detector = AnomalyDetector()
        self.logger = logging.getLogger(__name__)
        
        # Weights for different analysis components
        self.weights = {
            'temporal': 0.25,
            'interaction': 0.30,
            'content': 0.25,
            'anomaly': 0.20
        }
    
    async def analyze_behavior(self, metrics: BehaviorMetrics) -> BehaviorScore:
        """Comprehensive behavior analysis"""
        try:
            # Perform all analyses
            temporal_score = self.temporal_analyzer.analyze_circadian_rhythm(
                metrics.activity_timestamps
            )
            session_score = self.temporal_analyzer.analyze_session_patterns(metrics)
            
            click_score = self.interaction_analyzer.analyze_click_patterns(
                metrics.click_intervals
            )
            typing_score = self.interaction_analyzer.analyze_typing_patterns(
                metrics.typing_patterns
            )
            mouse_score = self.interaction_analyzer.analyze_mouse_movements(
                metrics.mouse_movements
            )
            
            content_score = self.content_analyzer.analyze_content_diversity(
                metrics.content_quality_scores, metrics.social_interactions
            )
            spam_score = self.content_analyzer.detect_spam_patterns(
                metrics.social_interactions
            )
            
            anomaly_score, anomaly_flags = self.anomaly_detector.detect_anomaly(metrics)
            
            # Calculate weighted scores
            temporal_component = (temporal_score + session_score) / 2 * self.weights['temporal']
            interaction_component = (click_score + typing_score + mouse_score) / 3 * self.weights['interaction']
            content_component = (content_score + spam_score) / 2 * self.weights['content']
            anomaly_component = anomaly_score * self.weights['anomaly']
            
            # Final human probability
            human_probability = temporal_component + interaction_component + content_component + anomaly_component
            
            # Determine risk level and confidence
            risk_level, confidence = self._calculate_risk_level(human_probability)
            
            # Generate behavioral patterns summary
            behavioral_patterns = {
                'temporal_naturalness': (temporal_score + session_score) / 2,
                'interaction_humanness': (click_score + typing_score + mouse_score) / 3,
                'content_authenticity': (content_score + spam_score) / 2,
                'statistical_normalcy': anomaly_score
            }
            
            # Generate recommendation
            recommendation = self._generate_recommendation(
                human_probability, anomaly_flags, behavioral_patterns
            )
            
            return BehaviorScore(
                human_probability=human_probability,
                risk_level=risk_level,
                confidence=confidence,
                anomaly_flags=anomaly_flags,
                behavioral_patterns=behavioral_patterns,
                recommendation=recommendation
            )
            
        except Exception as e:
            self.logger.error(f"Behavior analysis failed for user {metrics.user_id}: {str(e)}")
            return BehaviorScore(
                human_probability=0.5,
                risk_level="UNKNOWN",
                confidence=0.0,
                anomaly_flags=["Analysis failed"],
                behavioral_patterns={},
                recommendation="Manual review required"
            )
    
    def _calculate_risk_level(self, probability: float) -> Tuple[str, float]:
        """Calculate risk level and confidence from probability"""
        # Confidence based on how far from 0.5 (uncertain)
        confidence = abs(probability - 0.5) * 2
        
        if probability >= 0.8:
            return "LOW", confidence
        elif probability >= 0.6:
            return "MEDIUM", confidence
        elif probability >= 0.4:
            return "HIGH", confidence
        else:
            return "CRITICAL", confidence
    
    def _generate_recommendation(self, probability: float, flags: List[str], 
                                patterns: Dict[str, float]) -> str:
        """Generate actionable recommendation based on analysis"""
        if probability >= 0.8:
            return "User behavior appears authentic. Continue normal operations."
        elif probability >= 0.6:
            return "Minor anomalies detected. Enable enhanced monitoring."
        elif probability >= 0.4:
            return "Suspicious behavior detected. Require additional verification."
        else:
            return "High probability of bot behavior. Restrict access and require manual review."
    
    async def batch_analyze(self, metrics_list: List[BehaviorMetrics]) -> List[BehaviorScore]:
        """Analyze multiple users in batch"""
        tasks = [self.analyze_behavior(metrics) for metrics in metrics_list]
        return await asyncio.gather(*tasks)
    
    def train_model(self, training_data: List[BehaviorMetrics]):
        """Train the anomaly detection model with historical data"""
        self.anomaly_detector.train(training_data)
        self.logger.info(f"Trained anomaly detector with {len(training_data)} samples")

# Example usage and testing
if __name__ == "__main__":
    import asyncio
    from datetime import datetime, timedelta
    
    async def test_behavior_analyzer():
        # Create sample behavior data
        now = datetime.now()
        
        # Human-like behavior
        human_metrics = BehaviorMetrics(
            user_id="human_user_123",
            session_duration=45.5,
            click_intervals=[1.2, 0.8, 2.1, 1.5, 0.9, 3.2, 1.1],
            typing_patterns=[0.15, 0.22, 0.18, 0.35, 0.12, 0.28, 0.19],
            mouse_movements=[(100, 150), (105, 148), (112, 145), (118, 140), 
                           (125, 138), (135, 142), (145, 155)],
            scroll_patterns=[2.1, 1.8, 2.5, 1.9, 2.2],
            activity_timestamps=[now - timedelta(hours=i) for i in range(10, 0, -1)],
            content_quality_scores=[0.7, 0.8, 0.6, 0.9, 0.5, 0.8],
            social_interactions=[
                {"action": "post", "content_type": "text", "timestamp": now - timedelta(hours=2)},
                {"action": "comment", "content_type": "text", "timestamp": now - timedelta(hours=1)},
            ],
            device_fingerprint={"screen_resolution": "1920x1080", "user_agent": "Chrome/91.0"}
        )
        
        # Bot-like behavior
        bot_metrics = BehaviorMetrics(
            user_id="bot_user_456",
            session_duration=120.0,
            click_intervals=[0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5],
            typing_patterns=[0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1],
            mouse_movements=[(0, 0), (10, 0), (20, 0), (30, 0), (40, 0)],
            scroll_patterns=[1.0, 1.0, 1.0, 1.0, 1.0],
            activity_timestamps=[now - timedelta(seconds=i*30) for i in range(20)],
            content_quality_scores=[0.3, 0.3, 0.3, 0.3, 0.3],
            social_interactions=[
                {"action": "post", "content_type": "text", "timestamp": now - timedelta(minutes=i)} 
                for i in range(10)
            ],
            device_fingerprint={"screen_resolution": "1024x768", "user_agent": "Bot/1.0"}
        )
        
        # Initialize analyzer
        analyzer = BehaviorAnalyzer()
        
        # Analyze behaviors
        human_result = await analyzer.analyze_behavior(human_metrics)
        bot_result = await analyzer.analyze_behavior(bot_metrics)
        
        print("=== Human User Analysis ===")
        print(f"Human Probability: {human_result.human_probability:.2f}")
        print(f"Risk Level: {human_result.risk_level}")
        print(f"Confidence: {human_result.confidence:.2f}")
        print(f"Recommendation: {human_result.recommendation}")
        print()
        
        print("=== Bot User Analysis ===")
        print(f"Human Probability: {bot_result.human_probability:.2f}")
        print(f"Risk Level: {bot_result.risk_level}")
        print(f"Confidence: {bot_result.confidence:.2f}")
        print(f"Recommendation: {bot_result.recommendation}")
        print(f"Anomaly Flags: {bot_result.anomaly_flags}")
    
    # Run the test
    asyncio.run(test_behavior_analyzer
    