"""
Finova Network - Bot Detection Pattern Detector
Advanced pattern recognition system for identifying suspicious user behaviors
"""

import numpy as np
import pandas as pd
from typing import Dict, List, Tuple, Optional, Any
from dataclasses import dataclass
from datetime import datetime, timedelta
import json
import hashlib
import logging
from sklearn.ensemble import IsolationForest
from sklearn.preprocessing import StandardScaler
from collections import defaultdict, deque
import asyncio
import redis
from enum import Enum

# Configure logging
logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)

class SuspicionLevel(Enum):
    CLEAN = 0
    LOW = 1
    MEDIUM = 2
    HIGH = 3
    CRITICAL = 4

@dataclass
class UserActivity:
    user_id: str
    timestamp: datetime
    activity_type: str
    platform: str
    device_id: str
    ip_address: str
    session_id: str
    content_hash: Optional[str] = None
    engagement_time: float = 0.0
    geolocation: Optional[Dict[str, float]] = None

@dataclass
class PatternScore:
    pattern_type: str
    score: float
    evidence: List[str]
    severity: SuspicionLevel
    confidence: float

class TemporalPatternDetector:
    """Detects suspicious temporal patterns in user activity"""
    
    def __init__(self, window_hours: int = 24):
        self.window_hours = window_hours
        self.activity_buffer = defaultdict(deque)
        
    def analyze_temporal_patterns(self, activities: List[UserActivity]) -> PatternScore:
        """Analyze temporal patterns for bot-like behavior"""
        if len(activities) < 5:
            return PatternScore("temporal", 0.0, [], SuspicionLevel.CLEAN, 0.5)
        
        # Convert to timestamps
        timestamps = [act.timestamp for act in activities]
        time_diffs = [(timestamps[i+1] - timestamps[i]).total_seconds() 
                      for i in range(len(timestamps)-1)]
        
        evidence = []
        score = 0.0
        
        # Check for extremely regular intervals (bot-like)
        if time_diffs:
            cv = np.std(time_diffs) / np.mean(time_diffs) if np.mean(time_diffs) > 0 else 1
            if cv < 0.1:  # Very regular pattern
                score += 0.4
                evidence.append(f"Extremely regular intervals (CV: {cv:.3f})")
        
        # Check for burst patterns
        burst_threshold = 300  # 5 minutes
        burst_count = sum(1 for diff in time_diffs if diff < burst_threshold)
        if burst_count > len(time_diffs) * 0.7:
            score += 0.3
            evidence.append(f"Burst activity pattern ({burst_count} bursts)")
        
        # Check for 24/7 activity (no sleep pattern)
        hours = [ts.hour for ts in timestamps]
        unique_hours = len(set(hours))
        if unique_hours > 20 and len(timestamps) > 50:
            score += 0.3
            evidence.append(f"No natural sleep pattern ({unique_hours}/24 hours)")
        
        severity = self._calculate_severity(score)
        confidence = min(len(activities) / 100, 1.0)
        
        return PatternScore("temporal", score, evidence, severity, confidence)

class BehavioralPatternDetector:
    """Detects suspicious behavioral patterns"""
    
    def __init__(self):
        self.scaler = StandardScaler()
        self.isolation_forest = IsolationForest(contamination=0.1, random_state=42)
        self.feature_cache = {}
        
    def extract_behavioral_features(self, activities: List[UserActivity]) -> np.ndarray:
        """Extract behavioral features for anomaly detection"""
        features = []
        
        if not activities:
            return np.array([]).reshape(0, -1)
        
        # Activity frequency features
        activity_types = [act.activity_type for act in activities]
        type_counts = pd.Series(activity_types).value_counts()
        
        # Platform distribution
        platforms = [act.platform for act in activities]
        platform_counts = pd.Series(platforms).value_counts()
        
        # Engagement time statistics
        engagement_times = [act.engagement_time for act in activities if act.engagement_time > 0]
        
        # Create feature vector
        feature_vector = [
            len(activities),  # Total activities
            len(set(activity_types)),  # Activity type diversity
            len(set(platforms)),  # Platform diversity
            type_counts.max() / len(activities) if activities else 0,  # Max activity type ratio
            platform_counts.max() / len(activities) if activities else 0,  # Max platform ratio
            np.mean(engagement_times) if engagement_times else 0,  # Avg engagement time
            np.std(engagement_times) if len(engagement_times) > 1 else 0,  # Engagement time variance
            len(set([act.device_id for act in activities])),  # Device diversity
            len(set([act.ip_address for act in activities])),  # IP diversity
        ]
        
        return np.array(feature_vector).reshape(1, -1)
    
    def analyze_behavioral_patterns(self, activities: List[UserActivity]) -> PatternScore:
        """Analyze behavioral patterns for anomalies"""
        if len(activities) < 10:
            return PatternScore("behavioral", 0.0, [], SuspicionLevel.CLEAN, 0.3)
        
        features = self.extract_behavioral_features(activities)
        evidence = []
        score = 0.0
        
        # Check for single-platform dominance
        platforms = [act.platform for act in activities]
        platform_counts = pd.Series(platforms).value_counts()
        max_platform_ratio = platform_counts.max() / len(activities)
        
        if max_platform_ratio > 0.95:
            score += 0.3
            evidence.append(f"Single platform dominance: {max_platform_ratio:.2f}")
        
        # Check for repetitive activity types
        activity_types = [act.activity_type for act in activities]
        type_counts = pd.Series(activity_types).value_counts()
        max_type_ratio = type_counts.max() / len(activities)
        
        if max_type_ratio > 0.9:
            score += 0.25
            evidence.append(f"Repetitive activity type: {max_type_ratio:.2f}")
        
        # Check engagement time patterns
        engagement_times = [act.engagement_time for act in activities if act.engagement_time > 0]
        if engagement_times:
            avg_engagement = np.mean(engagement_times)
            if avg_engagement < 5:  # Less than 5 seconds average
                score += 0.2
                evidence.append(f"Suspiciously low engagement: {avg_engagement:.2f}s")
            elif avg_engagement > 3600:  # More than 1 hour average
                score += 0.15
                evidence.append(f"Unusually high engagement: {avg_engagement:.2f}s")
        
        # Check device/IP consistency
        devices = set([act.device_id for act in activities])
        ips = set([act.ip_address for act in activities])
        
        if len(devices) == 1 and len(ips) > 10:
            score += 0.2
            evidence.append(f"Single device, multiple IPs: {len(ips)}")
        
        severity = self._calculate_severity(score)
        confidence = min(len(activities) / 50, 1.0)
        
        return PatternScore("behavioral", score, evidence, severity, confidence)

class ContentPatternDetector:
    """Detects suspicious content patterns"""
    
    def __init__(self):
        self.content_hashes = defaultdict(int)
        self.similarity_threshold = 0.8
        
    def calculate_content_similarity(self, content1: str, content2: str) -> float:
        """Calculate similarity between two content strings"""
        if not content1 or not content2:
            return 0.0
        
        # Simple Jaccard similarity using character n-grams
        def get_ngrams(text: str, n: int = 3) -> set:
            return set([text[i:i+n] for i in range(len(text)-n+1)])
        
        ngrams1 = get_ngrams(content1.lower())
        ngrams2 = get_ngrams(content2.lower())
        
        if not ngrams1 and not ngrams2:
            return 1.0
        if not ngrams1 or not ngrams2:
            return 0.0
        
        intersection = len(ngrams1.intersection(ngrams2))
        union = len(ngrams1.union(ngrams2))
        
        return intersection / union if union > 0 else 0.0
    
    def analyze_content_patterns(self, activities: List[UserActivity]) -> PatternScore:
        """Analyze content patterns for duplication and spam"""
        if len(activities) < 5:
            return PatternScore("content", 0.0, [], SuspicionLevel.CLEAN, 0.3)
        
        evidence = []
        score = 0.0
        
        # Extract activities with content
        content_activities = [act for act in activities if act.content_hash]
        if not content_activities:
            return PatternScore("content", 0.0, [], SuspicionLevel.CLEAN, 0.2)
        
        # Check for exact duplicates
        content_hashes = [act.content_hash for act in content_activities]
        hash_counts = pd.Series(content_hashes).value_counts()
        duplicate_ratio = (len(content_hashes) - len(hash_counts)) / len(content_hashes)
        
        if duplicate_ratio > 0.3:
            score += 0.4
            evidence.append(f"High duplicate content: {duplicate_ratio:.2f}")
        
        # Check for rapid content generation
        if len(content_activities) > 1:
            time_diffs = []
            for i in range(1, len(content_activities)):
                diff = (content_activities[i].timestamp - content_activities[i-1].timestamp).total_seconds()
                time_diffs.append(diff)
            
            avg_time_diff = np.mean(time_diffs)
            if avg_time_diff < 30:  # Less than 30 seconds between content
                score += 0.3
                evidence.append(f"Rapid content generation: {avg_time_diff:.1f}s avg")
        
        # Check content length patterns (assuming hash represents content complexity)
        content_complexities = [len(act.content_hash) for act in content_activities]
        if content_complexities:
            cv = np.std(content_complexities) / np.mean(content_complexities)
            if cv < 0.1:  # Very similar content complexity
                score += 0.2
                evidence.append(f"Uniform content complexity: CV {cv:.3f}")
        
        severity = self._calculate_severity(score)
        confidence = min(len(content_activities) / 20, 1.0)
        
        return PatternScore("content", score, evidence, severity, confidence)

class NetworkPatternDetector:
    """Detects suspicious network-level patterns"""
    
    def __init__(self):
        self.ip_activity = defaultdict(list)
        self.device_activity = defaultdict(list)
        
    def analyze_network_patterns(self, activities: List[UserActivity]) -> PatternScore:
        """Analyze network-level suspicious patterns"""
        if len(activities) < 3:
            return PatternScore("network", 0.0, [], SuspicionLevel.CLEAN, 0.2)
        
        evidence = []
        score = 0.0
        
        # Group by IP and device
        ip_groups = defaultdict(list)
        device_groups = defaultdict(list)
        
        for act in activities:
            ip_groups[act.ip_address].append(act)
            device_groups[act.device_id].append(act)
        
        # Check for suspicious IP patterns
        if len(ip_groups) > 1:
            # Check for rapid IP switching
            sorted_activities = sorted(activities, key=lambda x: x.timestamp)
            ip_switches = 0
            for i in range(1, len(sorted_activities)):
                if sorted_activities[i].ip_address != sorted_activities[i-1].ip_address:
                    ip_switches += 1
            
            switch_ratio = ip_switches / len(activities)
            if switch_ratio > 0.5:
                score += 0.3
                evidence.append(f"Frequent IP switching: {switch_ratio:.2f}")
        
        # Check for geolocation inconsistencies
        geolocations = [act.geolocation for act in activities if act.geolocation]
        if len(geolocations) > 2:
            # Calculate maximum distance between geolocations
            max_distance = 0
            for i in range(len(geolocations)):
                for j in range(i+1, len(geolocations)):
                    lat1, lon1 = geolocations[i].get('lat', 0), geolocations[i].get('lon', 0)
                    lat2, lon2 = geolocations[j].get('lat', 0), geolocations[j].get('lon', 0)
                    
                    # Haversine distance calculation (simplified)
                    dlat = abs(lat2 - lat1)
                    dlon = abs(lon2 - lon1)
                    distance = (dlat ** 2 + dlon ** 2) ** 0.5 * 111  # Approximate km
                    max_distance = max(max_distance, distance)
            
            if max_distance > 1000:  # More than 1000km difference
                score += 0.25
                evidence.append(f"Large geolocation variance: {max_distance:.0f}km")
        
        # Check for device fingerprint inconsistencies
        if len(device_groups) > 3:  # Multiple devices for same user
            score += 0.15
            evidence.append(f"Multiple devices: {len(device_groups)}")
        
        severity = self._calculate_severity(score)
        confidence = min(len(activities) / 30, 1.0)
        
        return PatternScore("network", score, evidence, severity, confidence)

class PatternDetector:
    """Main pattern detector orchestrating all detection methods"""
    
    def __init__(self, redis_client: Optional[redis.Redis] = None):
        self.temporal_detector = TemporalPatternDetector()
        self.behavioral_detector = BehavioralPatternDetector()
        self.content_detector = ContentPatternDetector()
        self.network_detector = NetworkPatternDetector()
        self.redis_client = redis_client
        
        # Pattern weights for final scoring
        self.pattern_weights = {
            'temporal': 0.25,
            'behavioral': 0.30,
            'content': 0.25,
            'network': 0.20
        }
        
    def _calculate_severity(self, score: float) -> SuspicionLevel:
        """Convert numeric score to severity level"""
        if score >= 0.8:
            return SuspicionLevel.CRITICAL
        elif score >= 0.6:
            return SuspicionLevel.HIGH
        elif score >= 0.4:
            return SuspicionLevel.MEDIUM
        elif score >= 0.2:
            return SuspicionLevel.LOW
        else:
            return SuspicionLevel.CLEAN
    
    def _cache_results(self, user_id: str, results: Dict[str, Any], ttl: int = 3600):
        """Cache detection results in Redis"""
        if self.redis_client:
            try:
                cache_key = f"pattern_detection:{user_id}"
                self.redis_client.setex(cache_key, ttl, json.dumps(results, default=str))
            except Exception as e:
                logger.warning(f"Failed to cache results: {e}")
    
    def _get_cached_results(self, user_id: str) -> Optional[Dict[str, Any]]:
        """Get cached detection results"""
        if self.redis_client:
            try:
                cache_key = f"pattern_detection:{user_id}"
                cached_data = self.redis_client.get(cache_key)
                if cached_data:
                    return json.loads(cached_data)
            except Exception as e:
                logger.warning(f"Failed to get cached results: {e}")
        return None
    
    async def detect_patterns(self, user_id: str, activities: List[UserActivity], 
                            use_cache: bool = True) -> Dict[str, Any]:
        """Main pattern detection method"""
        
        # Check cache first
        if use_cache:
            cached_results = self._get_cached_results(user_id)
            if cached_results:
                logger.info(f"Returning cached results for user {user_id}")
                return cached_results
        
        logger.info(f"Analyzing patterns for user {user_id} with {len(activities)} activities")
        
        # Run all detectors
        detectors = [
            ("temporal", self.temporal_detector.analyze_temporal_patterns),
            ("behavioral", self.behavioral_detector.analyze_behavioral_patterns),
            ("content", self.content_detector.analyze_content_patterns),
            ("network", self.network_detector.analyze_network_patterns)
        ]
        
        pattern_scores = {}
        total_weighted_score = 0.0
        total_weight = 0.0
        all_evidence = []
        
        for pattern_type, detector_func in detectors:
            try:
                pattern_score = detector_func(activities)
                pattern_scores[pattern_type] = {
                    'score': pattern_score.score,
                    'evidence': pattern_score.evidence,
                    'severity': pattern_score.severity.name,
                    'confidence': pattern_score.confidence
                }
                
                # Calculate weighted contribution
                weight = self.pattern_weights[pattern_type] * pattern_score.confidence
                total_weighted_score += pattern_score.score * weight
                total_weight += weight
                all_evidence.extend(pattern_score.evidence)
                
            except Exception as e:
                logger.error(f"Error in {pattern_type} detection: {e}")
                pattern_scores[pattern_type] = {
                    'score': 0.0,
                    'evidence': [f"Detection error: {str(e)}"],
                    'severity': SuspicionLevel.CLEAN.name,
                    'confidence': 0.0
                }
        
        # Calculate final composite score
        final_score = total_weighted_score / total_weight if total_weight > 0 else 0.0
        final_severity = self._calculate_severity(final_score)
        
        # Generate recommendations
        recommendations = self._generate_recommendations(pattern_scores, final_score)
        
        results = {
            'user_id': user_id,
            'timestamp': datetime.utcnow().isoformat(),
            'final_score': round(final_score, 3),
            'severity': final_severity.name,
            'pattern_scores': pattern_scores,
            'evidence': all_evidence,
            'recommendations': recommendations,
            'activity_count': len(activities),
            'analysis_version': '1.0.0'
        }
        
        # Cache results
        if use_cache:
            self._cache_results(user_id, results)
        
        logger.info(f"Pattern analysis complete for user {user_id}: {final_severity.name} ({final_score:.3f})")
        return results
    
    def _generate_recommendations(self, pattern_scores: Dict[str, Any], final_score: float) -> List[str]:
        """Generate actionable recommendations based on detection results"""
        recommendations = []
        
        if final_score >= 0.8:
            recommendations.append("IMMEDIATE ACTION: Suspend account pending investigation")
            recommendations.append("Require additional verification before re-enabling")
        elif final_score >= 0.6:
            recommendations.append("Apply enhanced monitoring and rate limiting")
            recommendations.append("Require human verification for high-value activities")
        elif final_score >= 0.4:
            recommendations.append("Increase monitoring frequency")
            recommendations.append("Apply minor rate limiting as precaution")
        elif final_score >= 0.2:
            recommendations.append("Flag for periodic review")
            recommendations.append("Monitor for pattern evolution")
        else:
            recommendations.append("No immediate action required")
            recommendations.append("Continue standard monitoring")
        
        # Add specific recommendations based on pattern types
        for pattern_type, scores in pattern_scores.items():
            if scores['score'] > 0.6:
                if pattern_type == 'temporal':
                    recommendations.append("Implement CAPTCHA challenges during suspected bot hours")
                elif pattern_type == 'behavioral':
                    recommendations.append("Require behavioral biometric verification")
                elif pattern_type == 'content':
                    recommendations.append("Apply content originality verification")
                elif pattern_type == 'network':
                    recommendations.append("Restrict access from suspicious network patterns")
        
        return recommendations
    
    def get_detection_statistics(self) -> Dict[str, Any]:
        """Get statistics about detection performance"""
        stats = {
            'detector_versions': {
                'temporal': '1.0.0',
                'behavioral': '1.0.0', 
                'content': '1.0.0',
                'network': '1.0.0'
            },
            'pattern_weights': self.pattern_weights,
            'severity_thresholds': {
                'CLEAN': '0.0-0.2',
                'LOW': '0.2-0.4', 
                'MEDIUM': '0.4-0.6',
                'HIGH': '0.6-0.8',
                'CRITICAL': '0.8-1.0'
            }
        }
        return stats

# Example usage and testing functions
def create_sample_activities(user_id: str, pattern_type: str = "normal") -> List[UserActivity]:
    """Create sample activities for testing different patterns"""
    base_time = datetime.utcnow() - timedelta(days=1)
    activities = []
    
    if pattern_type == "bot":
        # Create bot-like pattern: very regular intervals, same platform
        for i in range(20):
            activities.append(UserActivity(
                user_id=user_id,
                timestamp=base_time + timedelta(minutes=i*15),  # Every 15 minutes exactly
                activity_type="post",
                platform="instagram",
                device_id="device_123",
                ip_address="192.168.1.1",
                session_id=f"session_{i//5}",
                content_hash=f"hash_{i%3}",  # Repetitive content
                engagement_time=3.0  # Very low engagement
            ))
    else:
        # Create normal human-like pattern
        import random
        platforms = ["instagram", "tiktok", "youtube", "facebook"]
        activities_types = ["post", "comment", "like", "share"]
        
        for i in range(15):
            # Random intervals with some clustering
            minutes_offset = sum([random.randint(5, 180) for _ in range(i+1)])
            activities.append(UserActivity(
                user_id=user_id,
                timestamp=base_time + timedelta(minutes=minutes_offset),
                activity_type=random.choice(activities_types),
                platform=random.choice(platforms),
                device_id=random.choice(["device_1", "device_2"]),
                ip_address=random.choice(["192.168.1.1", "192.168.1.2"]),
                session_id=f"session_{random.randint(1, 5)}",
                content_hash=f"hash_{random.randint(1, 10)}" if random.random() > 0.3 else None,
                engagement_time=random.uniform(10, 300)
            ))
    
    return sorted(activities, key=lambda x: x.timestamp)

async def main():
    """Example usage of PatternDetector"""
    # Initialize detector
    detector = PatternDetector()
    
    # Test with normal user
    normal_activities = create_sample_activities("user_normal", "normal")
    normal_results = await detector.detect_patterns("user_normal", normal_activities)
    print("Normal User Results:")
    print(f"Score: {normal_results['final_score']}, Severity: {normal_results['severity']}")
    print("Evidence:", normal_results['evidence'][:3])  # Show first 3 pieces of evidence
    print()
    
    # Test with suspicious user
    bot_activities = create_sample_activities("user_bot", "bot")
    bot_results = await detector.detect_patterns("user_bot", bot_activities)
    print("Bot User Results:")
    print(f"Score: {bot_results['final_score']}, Severity: {bot_results['severity']}")
    print("Evidence:", bot_results['evidence'][:3])
    print("Recommendations:", bot_results['recommendations'][:2])

if __name__ == "__main__":
    asyncio.run(main())
    