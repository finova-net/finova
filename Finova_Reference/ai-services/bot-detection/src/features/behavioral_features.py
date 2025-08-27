"""
Finova Network - Behavioral Features Analysis
Advanced bot detection through behavioral pattern analysis
"""

import numpy as np
import pandas as pd
from typing import Dict, List, Tuple, Optional, Any
from datetime import datetime, timedelta
import math
from dataclasses import dataclass, field
from scipy import stats
from sklearn.preprocessing import StandardScaler
import json
import logging

logger = logging.getLogger(__name__)

@dataclass
class BehavioralMetrics:
    """Core behavioral metrics for user analysis"""
    user_id: str
    session_count: int = 0
    total_actions: int = 0
    avg_session_duration: float = 0.0
    action_frequency: float = 0.0
    click_intervals: List[float] = field(default_factory=list)
    session_gaps: List[float] = field(default_factory=list)
    activity_entropy: float = 0.0
    circadian_score: float = 0.0
    variance_coefficient: float = 0.0
    human_probability: float = 0.5

@dataclass
class ActivityPattern:
    """User activity pattern analysis"""
    timestamp: datetime
    action_type: str
    platform: str
    session_id: str
    duration: float
    content_quality: float
    interaction_depth: int

class BehavioralFeatureExtractor:
    """Extract behavioral features for bot detection"""
    
    def __init__(self, config: Optional[Dict] = None):
        self.config = config or self._default_config()
        self.scaler = StandardScaler()
        self.feature_weights = self._initialize_weights()
        
    def _default_config(self) -> Dict:
        """Default configuration for behavioral analysis"""
        return {
            'min_sessions_required': 5,
            'analysis_window_days': 30,
            'click_speed_threshold': 0.1,  # seconds
            'suspicious_frequency_threshold': 10,  # actions per minute
            'entropy_bins': 24,  # for circadian analysis
            'variance_threshold': 0.3,
            'human_score_threshold': 0.7,
            'bot_score_threshold': 0.3
        }
    
    def _initialize_weights(self) -> Dict[str, float]:
        """Feature importance weights for final scoring"""
        return {
            'click_speed_variance': 0.25,
            'session_pattern_regularity': 0.20,
            'activity_entropy': 0.15,
            'circadian_rhythm': 0.15,
            'interaction_depth': 0.10,
            'content_quality_consistency': 0.10,
            'platform_switching_pattern': 0.05
        }

    def extract_features(self, user_id: str, activities: List[ActivityPattern]) -> BehavioralMetrics:
        """Extract comprehensive behavioral features for a user"""
        try:
            if len(activities) < self.config['min_sessions_required']:
                logger.warning(f"Insufficient data for user {user_id}: {len(activities)} activities")
                return BehavioralMetrics(user_id=user_id)
            
            # Filter recent activities
            cutoff_date = datetime.now() - timedelta(days=self.config['analysis_window_days'])
            recent_activities = [a for a in activities if a.timestamp >= cutoff_date]
            
            if not recent_activities:
                return BehavioralMetrics(user_id=user_id)
            
            # Extract core metrics
            metrics = BehavioralMetrics(user_id=user_id)
            
            # Basic statistics
            metrics.total_actions = len(recent_activities)
            metrics.session_count = len(set(a.session_id for a in recent_activities))
            
            # Calculate detailed features
            metrics.click_intervals = self._calculate_click_intervals(recent_activities)
            metrics.session_gaps = self._calculate_session_gaps(recent_activities)
            metrics.avg_session_duration = self._calculate_avg_session_duration(recent_activities)
            metrics.action_frequency = self._calculate_action_frequency(recent_activities)
            metrics.activity_entropy = self._calculate_activity_entropy(recent_activities)
            metrics.circadian_score = self._calculate_circadian_score(recent_activities)
            metrics.variance_coefficient = self._calculate_variance_coefficient(recent_activities)
            
            # Calculate final human probability
            metrics.human_probability = self._calculate_human_probability(metrics, recent_activities)
            
            return metrics
            
        except Exception as e:
            logger.error(f"Error extracting features for user {user_id}: {e}")
            return BehavioralMetrics(user_id=user_id)

    def _calculate_click_intervals(self, activities: List[ActivityPattern]) -> List[float]:
        """Calculate time intervals between consecutive actions"""
        if len(activities) < 2:
            return []
        
        sorted_activities = sorted(activities, key=lambda x: x.timestamp)
        intervals = []
        
        for i in range(1, len(sorted_activities)):
            interval = (sorted_activities[i].timestamp - sorted_activities[i-1].timestamp).total_seconds()
            intervals.append(interval)
        
        return intervals

    def _calculate_session_gaps(self, activities: List[ActivityPattern]) -> List[float]:
        """Calculate gaps between sessions"""
        sessions = {}
        for activity in activities:
            if activity.session_id not in sessions:
                sessions[activity.session_id] = []
            sessions[activity.session_id].append(activity.timestamp)
        
        # Get session end times
        session_ends = []
        for session_activities in sessions.values():
            session_ends.append(max(session_activities))
        
        session_ends.sort()
        
        gaps = []
        for i in range(1, len(session_ends)):
            gap = (session_ends[i] - session_ends[i-1]).total_seconds() / 3600  # hours
            gaps.append(gap)
        
        return gaps

    def _calculate_avg_session_duration(self, activities: List[ActivityPattern]) -> float:
        """Calculate average session duration"""
        sessions = {}
        for activity in activities:
            if activity.session_id not in sessions:
                sessions[activity.session_id] = []
            sessions[activity.session_id].append(activity.timestamp)
        
        durations = []
        for session_times in sessions.values():
            if len(session_times) > 1:
                duration = (max(session_times) - min(session_times)).total_seconds() / 60  # minutes
                durations.append(duration)
        
        return np.mean(durations) if durations else 0.0

    def _calculate_action_frequency(self, activities: List[ActivityPattern]) -> float:
        """Calculate actions per minute over active periods"""
        if not activities:
            return 0.0
        
        sorted_activities = sorted(activities, key=lambda x: x.timestamp)
        total_time = (sorted_activities[-1].timestamp - sorted_activities[0].timestamp).total_seconds() / 60
        
        return len(activities) / max(total_time, 1.0)

    def _calculate_activity_entropy(self, activities: List[ActivityPattern]) -> float:
        """Calculate Shannon entropy of hourly activity distribution"""
        if not activities:
            return 0.0
        
        # Create hourly bins
        hours = [activity.timestamp.hour for activity in activities]
        hour_counts = np.zeros(24)
        
        for hour in hours:
            hour_counts[hour] += 1
        
        # Normalize to probabilities
        probabilities = hour_counts / np.sum(hour_counts)
        probabilities = probabilities[probabilities > 0]  # Remove zeros
        
        # Calculate entropy
        entropy = -np.sum(probabilities * np.log2(probabilities))
        return entropy / np.log2(24)  # Normalize to 0-1 range

    def _calculate_circadian_score(self, activities: List[ActivityPattern]) -> float:
        """Calculate how well activity follows human circadian patterns"""
        if not activities:
            return 0.0
        
        # Human activity typically peaks around 10 AM, 2 PM, and 8 PM
        # and is lowest from 2 AM to 6 AM
        expected_pattern = np.array([
            0.2, 0.1, 0.05, 0.05, 0.1, 0.2, 0.4, 0.6,  # 0-7 AM
            0.8, 0.9, 1.0, 0.9, 0.8, 0.9, 1.0, 0.9,   # 8-15 (8 AM-3 PM)
            0.8, 0.7, 0.6, 0.8, 0.9, 0.7, 0.5, 0.3    # 16-23 (4 PM-11 PM)
        ])
        
        # Actual activity distribution
        hours = [activity.timestamp.hour for activity in activities]
        hour_counts = np.zeros(24)
        
        for hour in hours:
            hour_counts[hour] += 1
        
        # Normalize
        if np.sum(hour_counts) > 0:
            actual_pattern = hour_counts / np.sum(hour_counts)
        else:
            actual_pattern = np.zeros(24)
        
        # Calculate correlation with expected human pattern
        if np.std(actual_pattern) > 0 and np.std(expected_pattern) > 0:
            correlation = np.corrcoef(actual_pattern, expected_pattern)[0, 1]
            return max(0, correlation)  # Only positive correlations
        
        return 0.0

    def _calculate_variance_coefficient(self, activities: List[ActivityPattern]) -> float:
        """Calculate coefficient of variation in action timing"""
        intervals = self._calculate_click_intervals(activities)
        
        if len(intervals) < 2:
            return 0.0
        
        mean_interval = np.mean(intervals)
        std_interval = np.std(intervals)
        
        if mean_interval == 0:
            return 0.0
        
        return std_interval / mean_interval

    def _calculate_human_probability(self, metrics: BehavioralMetrics, activities: List[ActivityPattern]) -> float:
        """Calculate overall human probability score"""
        features = self._extract_advanced_features(metrics, activities)
        
        # Weighted scoring
        human_score = 0.0
        total_weight = 0.0
        
        for feature_name, weight in self.feature_weights.items():
            if feature_name in features:
                human_score += features[feature_name] * weight
                total_weight += weight
        
        if total_weight > 0:
            human_score /= total_weight
        
        # Apply sigmoid normalization
        normalized_score = 1 / (1 + np.exp(-5 * (human_score - 0.5)))
        
        return max(0.1, min(0.9, normalized_score))

    def _extract_advanced_features(self, metrics: BehavioralMetrics, activities: List[ActivityPattern]) -> Dict[str, float]:
        """Extract advanced behavioral features"""
        features = {}
        
        # Click speed variance (humans have natural variation)
        if metrics.click_intervals:
            interval_variance = np.var(metrics.click_intervals)
            features['click_speed_variance'] = min(1.0, interval_variance / 10.0)
        else:
            features['click_speed_variance'] = 0.0
        
        # Session pattern regularity (bots often have rigid patterns)
        if metrics.session_gaps:
            gap_regularity = 1.0 - (np.std(metrics.session_gaps) / (np.mean(metrics.session_gaps) + 1e-6))
            features['session_pattern_regularity'] = 1.0 - max(0.0, min(1.0, gap_regularity))
        else:
            features['session_pattern_regularity'] = 0.5
        
        # Activity entropy (humans have varied activity)
        features['activity_entropy'] = metrics.activity_entropy
        
        # Circadian rhythm alignment
        features['circadian_rhythm'] = metrics.circadian_score
        
        # Interaction depth analysis
        avg_depth = np.mean([a.interaction_depth for a in activities]) if activities else 0
        features['interaction_depth'] = min(1.0, avg_depth / 5.0)
        
        # Content quality consistency
        quality_scores = [a.content_quality for a in activities if a.content_quality > 0]
        if quality_scores:
            quality_variance = np.var(quality_scores)
            features['content_quality_consistency'] = min(1.0, quality_variance)
        else:
            features['content_quality_consistency'] = 0.0
        
        # Platform switching pattern
        platforms = [a.platform for a in activities]
        platform_switches = sum(1 for i in range(1, len(platforms)) 
                               if platforms[i] != platforms[i-1])
        switch_rate = platform_switches / max(len(platforms), 1)
        features['platform_switching_pattern'] = min(1.0, switch_rate * 2)
        
        return features

    def analyze_suspicious_patterns(self, metrics: BehavioralMetrics) -> Dict[str, Any]:
        """Analyze specific suspicious patterns"""
        suspicions = {
            'is_suspicious': False,
            'risk_level': 'low',
            'suspicious_indicators': [],
            'confidence': 0.0
        }
        
        indicators = []
        risk_score = 0.0
        
        # Check click speed consistency (too regular = suspicious)
        if metrics.click_intervals:
            interval_std = np.std(metrics.click_intervals)
            if interval_std < self.config['click_speed_threshold']:
                indicators.append('extremely_consistent_click_timing')
                risk_score += 0.3
        
        # Check frequency (too high = suspicious)
        if metrics.action_frequency > self.config['suspicious_frequency_threshold']:
            indicators.append('abnormally_high_activity_frequency')
            risk_score += 0.25
        
        # Check variance coefficient (too low = bot-like)
        if metrics.variance_coefficient < self.config['variance_threshold']:
            indicators.append('low_behavioral_variance')
            risk_score += 0.2
        
        # Check circadian pattern (no pattern = suspicious)
        if metrics.circadian_score < 0.3:
            indicators.append('non_human_activity_pattern')
            risk_score += 0.15
        
        # Check entropy (too uniform = suspicious)
        if metrics.activity_entropy < 0.3:
            indicators.append('uniform_activity_distribution')
            risk_score += 0.1
        
        # Determine risk level
        if risk_score >= 0.7:
            suspicions['risk_level'] = 'high'
            suspicions['is_suspicious'] = True
        elif risk_score >= 0.4:
            suspicions['risk_level'] = 'medium'
            suspicions['is_suspicious'] = True
        elif risk_score >= 0.2:
            suspicions['risk_level'] = 'low'
        
        suspicions['suspicious_indicators'] = indicators
        suspicions['confidence'] = min(1.0, risk_score)
        
        return suspicions

    def get_feature_explanation(self, metrics: BehavioralMetrics) -> Dict[str, str]:
        """Provide human-readable explanations of features"""
        explanations = {}
        
        explanations['human_probability'] = f"Overall human likelihood: {metrics.human_probability:.2%}"
        
        if metrics.click_intervals:
            avg_interval = np.mean(metrics.click_intervals)
            explanations['click_timing'] = f"Average action interval: {avg_interval:.2f}s"
        
        explanations['activity_pattern'] = f"Activity entropy: {metrics.activity_entropy:.2f} (higher = more varied)"
        explanations['circadian_alignment'] = f"Human rhythm match: {metrics.circadian_score:.2%}"
        explanations['session_behavior'] = f"Average session: {metrics.avg_session_duration:.1f} minutes"
        explanations['frequency'] = f"Actions per minute: {metrics.action_frequency:.2f}"
        
        return explanations

    def batch_analyze_users(self, user_activities: Dict[str, List[ActivityPattern]]) -> Dict[str, Dict]:
        """Analyze multiple users in batch"""
        results = {}
        
        for user_id, activities in user_activities.items():
            try:
                metrics = self.extract_features(user_id, activities)
                suspicions = self.analyze_suspicious_patterns(metrics)
                explanations = self.get_feature_explanation(metrics)
                
                results[user_id] = {
                    'metrics': metrics,
                    'suspicions': suspicions,
                    'explanations': explanations,
                    'timestamp': datetime.now().isoformat()
                }
                
            except Exception as e:
                logger.error(f"Error analyzing user {user_id}: {e}")
                results[user_id] = {
                    'error': str(e),
                    'timestamp': datetime.now().isoformat()
                }
        
        return results

def create_sample_activities() -> List[ActivityPattern]:
    """Create sample activities for testing"""
    activities = []
    base_time = datetime.now() - timedelta(days=7)
    
    # Simulate human-like activity patterns
    for day in range(7):
        daily_start = base_time + timedelta(days=day)
        
        # Morning session (9-11 AM)
        morning_start = daily_start.replace(hour=9, minute=0, second=0)
        for i in range(np.random.poisson(15)):  # Variable activity
            activity_time = morning_start + timedelta(minutes=np.random.exponential(8))
            if activity_time.hour < 11:
                activities.append(ActivityPattern(
                    timestamp=activity_time,
                    action_type='post',
                    platform='instagram',
                    session_id=f"session_{day}_morning",
                    duration=np.random.normal(2.5, 1.0),
                    content_quality=np.random.beta(2, 1),
                    interaction_depth=np.random.randint(1, 6)
                ))
        
        # Evening session (7-9 PM)
        evening_start = daily_start.replace(hour=19, minute=0, second=0)
        for i in range(np.random.poisson(20)):
            activity_time = evening_start + timedelta(minutes=np.random.exponential(6))
            if activity_time.hour < 21:
                activities.append(ActivityPattern(
                    timestamp=activity_time,
                    action_type='comment',
                    platform='tiktok',
                    session_id=f"session_{day}_evening",
                    duration=np.random.normal(1.5, 0.5),
                    content_quality=np.random.beta(1.5, 1.5),
                    interaction_depth=np.random.randint(1, 4)
                ))
    
    return activities

# Example usage and testing
if __name__ == "__main__":
    # Initialize feature extractor
    extractor = BehavioralFeatureExtractor()
    
    # Create sample data
    sample_activities = create_sample_activities()
    
    # Extract features
    metrics = extractor.extract_features("test_user_001", sample_activities)
    
    # Analyze for suspicious patterns
    suspicions = extractor.analyze_suspicious_patterns(metrics)
    
    # Get explanations
    explanations = extractor.get_feature_explanation(metrics)
    
    # Print results
    print("Behavioral Analysis Results:")
    print(f"User ID: {metrics.user_id}")
    print(f"Human Probability: {metrics.human_probability:.2%}")
    print(f"Risk Level: {suspicions['risk_level']}")
    print(f"Is Suspicious: {suspicions['is_suspicious']}")
    print("\nFeature Explanations:")
    for key, explanation in explanations.items():
        print(f"  {key}: {explanation}")
    
    if suspicions['suspicious_indicators']:
        print(f"\nSuspicious Indicators: {', '.join(suspicions['suspicious_indicators'])}")
        