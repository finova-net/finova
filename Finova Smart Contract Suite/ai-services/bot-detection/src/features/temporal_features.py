"""
Finova Network - Temporal Features Analysis for Bot Detection
AI-powered temporal pattern analysis to distinguish human vs bot behavior

Features:
- Circadian rhythm analysis
- Activity pattern recognition
- Session timing analysis
- Burst detection
- Consistency scoring
- Entropy calculations
"""

import numpy as np
import pandas as pd
from typing import Dict, List, Tuple, Optional, Any
from datetime import datetime, timedelta
from scipy import stats
from scipy.fft import fft
from sklearn.preprocessing import StandardScaler
import logging
from dataclasses import dataclass
from enum import Enum
import asyncio
import json

# Configure logging
logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)

class ActivityType(Enum):
    """Activity types for temporal analysis"""
    LOGIN = "login"
    POST_CREATION = "post_creation"
    COMMENT = "comment"
    LIKE = "like"
    SHARE = "share"
    MINING_CLAIM = "mining_claim"
    XP_ACTIVITY = "xp_activity"
    REFERRAL_ACTION = "referral_action"
    NFT_INTERACTION = "nft_interaction"
    LOGOUT = "logout"

@dataclass
class TemporalWindow:
    """Temporal window configuration"""
    duration_hours: int
    min_activities: int
    weight: float

@dataclass
class ActivityEvent:
    """Individual activity event"""
    user_id: str
    timestamp: datetime
    activity_type: ActivityType
    platform: str
    metadata: Dict[str, Any]
    ip_address: Optional[str] = None
    user_agent: Optional[str] = None

@dataclass
class TemporalFeatures:
    """Temporal features for bot detection"""
    user_id: str
    
    # Circadian features
    circadian_regularity: float
    peak_activity_hour: int
    activity_variance: float
    timezone_consistency: float
    
    # Pattern features  
    burst_frequency: float
    session_regularity: float
    inter_activity_variance: float
    activity_entropy: float
    
    # Timing features
    avg_session_duration: float
    click_speed_variance: float
    response_time_consistency: float
    weekend_pattern_score: float
    
    # Anomaly scores
    temporal_anomaly_score: float
    human_likelihood: float
    bot_probability: float
    
    # Metadata
    analysis_period_days: int
    total_activities: int
    confidence_score: float

class TemporalFeaturesExtractor:
    """Extract temporal features for bot detection"""
    
    def __init__(self, config: Dict[str, Any]):
        self.config = config
        self.scaler = StandardScaler()
        
        # Temporal windows for analysis
        self.windows = [
            TemporalWindow(1, 5, 0.3),    # 1 hour window
            TemporalWindow(6, 10, 0.4),   # 6 hour window  
            TemporalWindow(24, 20, 0.5),  # Daily window
            TemporalWindow(168, 50, 0.6)  # Weekly window
        ]
        
        # Human baseline patterns
        self.human_baselines = {
            'circadian_peak_hours': [19, 20, 21],  # 7-9 PM typical peak
            'normal_session_duration': (15, 120),  # 15 min - 2 hours
            'human_click_speed': (0.5, 3.0),      # 0.5-3 seconds between clicks
            'weekend_activity_ratio': (0.7, 1.3)   # Weekend vs weekday ratio
        }

    async def extract_features(self, user_id: str, activities: List[ActivityEvent], 
                             analysis_days: int = 30) -> TemporalFeatures:
        """Extract comprehensive temporal features for a user"""
        try:
            if not activities or len(activities) < 10:
                return self._create_default_features(user_id, analysis_days, len(activities))
            
            # Sort activities by timestamp
            activities.sort(key=lambda x: x.timestamp)
            
            # Extract feature components
            circadian_features = await self._extract_circadian_features(activities)
            pattern_features = await self._extract_pattern_features(activities)
            timing_features = await self._extract_timing_features(activities)
            anomaly_scores = await self._calculate_anomaly_scores(activities)
            
            # Calculate human likelihood
            human_likelihood = self._calculate_human_likelihood(
                circadian_features, pattern_features, timing_features, anomaly_scores
            )
            
            return TemporalFeatures(
                user_id=user_id,
                circadian_regularity=circadian_features['regularity'],
                peak_activity_hour=circadian_features['peak_hour'],
                activity_variance=circadian_features['variance'],
                timezone_consistency=circadian_features['timezone_consistency'],
                burst_frequency=pattern_features['burst_frequency'],
                session_regularity=pattern_features['session_regularity'],
                inter_activity_variance=pattern_features['inter_activity_variance'],
                activity_entropy=pattern_features['entropy'],
                avg_session_duration=timing_features['avg_session_duration'],
                click_speed_variance=timing_features['click_speed_variance'],
                response_time_consistency=timing_features['response_consistency'],
                weekend_pattern_score=timing_features['weekend_pattern'],
                temporal_anomaly_score=anomaly_scores['temporal_anomaly'],
                human_likelihood=human_likelihood,
                bot_probability=1.0 - human_likelihood,
                analysis_period_days=analysis_days,
                total_activities=len(activities),
                confidence_score=self._calculate_confidence_score(activities, analysis_days)
            )
            
        except Exception as e:
            logger.error(f"Error extracting temporal features for user {user_id}: {str(e)}")
            return self._create_default_features(user_id, analysis_days, len(activities))

    async def _extract_circadian_features(self, activities: List[ActivityEvent]) -> Dict[str, float]:
        """Extract circadian rhythm features"""
        try:
            # Convert to hour-of-day distribution
            hours = [activity.timestamp.hour for activity in activities]
            hour_counts = np.bincount(hours, minlength=24)
            
            # Calculate circadian regularity using FFT
            fft_result = np.abs(fft(hour_counts))
            circadian_power = fft_result[1]  # 24-hour cycle
            total_power = np.sum(fft_result[1:12])  # Avoid DC component
            regularity = circadian_power / (total_power + 1e-8)
            
            # Peak activity hour
            peak_hour = np.argmax(hour_counts)
            
            # Activity variance across hours
            variance = np.var(hour_counts) / (np.mean(hour_counts) + 1e-8)
            
            # Timezone consistency (check for sudden shifts)
            timezone_consistency = self._calculate_timezone_consistency(activities)
            
            return {
                'regularity': float(regularity),
                'peak_hour': int(peak_hour),
                'variance': float(variance),
                'timezone_consistency': timezone_consistency
            }
            
        except Exception as e:
            logger.error(f"Error in circadian feature extraction: {str(e)}")
            return {'regularity': 0.5, 'peak_hour': 12, 'variance': 1.0, 'timezone_consistency': 0.5}

    async def _extract_pattern_features(self, activities: List[ActivityEvent]) -> Dict[str, float]:
        """Extract activity pattern features"""
        try:
            timestamps = [activity.timestamp for activity in activities]
            
            # Calculate inter-activity intervals
            intervals = []
            for i in range(1, len(timestamps)):
                interval = (timestamps[i] - timestamps[i-1]).total_seconds()
                intervals.append(interval)
            
            if not intervals:
                return {'burst_frequency': 0.0, 'session_regularity': 0.0, 
                       'inter_activity_variance': 1.0, 'entropy': 0.0}
            
            # Burst frequency (activities within 5 seconds)
            burst_count = sum(1 for interval in intervals if interval < 5)
            burst_frequency = burst_count / len(intervals)
            
            # Session regularity (coefficient of variation of session gaps)
            session_gaps = self._identify_session_gaps(timestamps)
            if session_gaps:
                cv = np.std(session_gaps) / (np.mean(session_gaps) + 1e-8)
                session_regularity = 1.0 / (1.0 + cv)
            else:
                session_regularity = 0.5
            
            # Inter-activity variance
            inter_activity_variance = np.var(intervals) / (np.mean(intervals) + 1e-8)
            
            # Activity entropy (Shannon entropy of activity types)
            activity_types = [activity.activity_type.value for activity in activities]
            entropy = self._calculate_entropy(activity_types)
            
            return {
                'burst_frequency': float(burst_frequency),
                'session_regularity': float(session_regularity),
                'inter_activity_variance': float(inter_activity_variance),
                'entropy': float(entropy)
            }
            
        except Exception as e:
            logger.error(f"Error in pattern feature extraction: {str(e)}")
            return {'burst_frequency': 0.0, 'session_regularity': 0.5, 
                   'inter_activity_variance': 1.0, 'entropy': 1.0}

    async def _extract_timing_features(self, activities: List[ActivityEvent]) -> Dict[str, float]:
        """Extract timing-related features"""
        try:
            timestamps = [activity.timestamp for activity in activities]
            
            # Session analysis
            sessions = self._group_into_sessions(timestamps)
            session_durations = [(session[-1] - session[0]).total_seconds() / 60 
                               for session in sessions if len(session) > 1]
            
            avg_session_duration = np.mean(session_durations) if session_durations else 0
            
            # Click speed variance (time between consecutive activities)
            click_speeds = []
            for i in range(1, len(timestamps)):
                speed = (timestamps[i] - timestamps[i-1]).total_seconds()
                if speed < 300:  # Within 5 minutes
                    click_speeds.append(speed)
            
            click_speed_variance = np.var(click_speeds) if click_speeds else 0
            
            # Response time consistency
            response_consistency = self._calculate_response_consistency(activities)
            
            # Weekend pattern analysis
            weekend_pattern = self._analyze_weekend_pattern(timestamps)
            
            return {
                'avg_session_duration': float(avg_session_duration),
                'click_speed_variance': float(click_speed_variance),
                'response_consistency': float(response_consistency),
                'weekend_pattern': float(weekend_pattern)
            }
            
        except Exception as e:
            logger.error(f"Error in timing feature extraction: {str(e)}")
            return {'avg_session_duration': 30.0, 'click_speed_variance': 1.0,
                   'response_consistency': 0.5, 'weekend_pattern': 0.8}

    async def _calculate_anomaly_scores(self, activities: List[ActivityEvent]) -> Dict[str, float]:
        """Calculate temporal anomaly scores"""
        try:
            timestamps = [activity.timestamp for activity in activities]
            
            # Time-based anomaly detection
            temporal_anomalies = []
            
            # Check for unusual timing patterns
            for window in self.windows:
                window_anomalies = self._detect_window_anomalies(timestamps, window)
                temporal_anomalies.extend(window_anomalies)
            
            # Overall temporal anomaly score
            temporal_anomaly_score = np.mean(temporal_anomalies) if temporal_anomalies else 0.0
            
            return {
                'temporal_anomaly': float(temporal_anomaly_score)
            }
            
        except Exception as e:
            logger.error(f"Error calculating anomaly scores: {str(e)}")
            return {'temporal_anomaly': 0.3}

    def _calculate_human_likelihood(self, circadian_features: Dict, pattern_features: Dict,
                                  timing_features: Dict, anomaly_scores: Dict) -> float:
        """Calculate overall human likelihood score"""
        try:
            scores = []
            weights = []
            
            # Circadian score (higher regularity = more human)
            circadian_score = min(circadian_features['regularity'] * 2, 1.0)
            scores.append(circadian_score)
            weights.append(0.25)
            
            # Pattern score (moderate burst frequency is human)
            burst_freq = pattern_features['burst_frequency']
            pattern_score = 1.0 - abs(burst_freq - 0.1)  # Optimal around 10%
            scores.append(max(0, pattern_score))
            weights.append(0.20)
            
            # Session regularity (humans have some regularity but not perfect)
            session_score = pattern_features['session_regularity']
            if session_score > 0.9:  # Too regular = suspicious
                session_score = 0.3
            scores.append(session_score)
            weights.append(0.15)
            
            # Timing score (human-like session durations)
            timing_score = self._score_session_duration(timing_features['avg_session_duration'])
            scores.append(timing_score)
            weights.append(0.20)
            
            # Entropy score (humans have diverse activities)
            entropy = pattern_features['entropy']
            entropy_score = min(entropy / 2.0, 1.0)  # Normalize entropy
            scores.append(entropy_score)
            weights.append(0.10)
            
            # Anomaly penalty
            anomaly_penalty = 1.0 - anomaly_scores['temporal_anomaly']
            scores.append(anomaly_penalty)
            weights.append(0.10)
            
            # Weighted average
            weighted_score = sum(s * w for s, w in zip(scores, weights)) / sum(weights)
            
            return max(0.0, min(1.0, weighted_score))
            
        except Exception as e:
            logger.error(f"Error calculating human likelihood: {str(e)}")
            return 0.5

    def _calculate_timezone_consistency(self, activities: List[ActivityEvent]) -> float:
        """Check for consistent timezone usage"""
        try:
            # Group activities by day and check peak hours
            daily_peaks = {}
            
            for activity in activities:
                day_key = activity.timestamp.date()
                hour = activity.timestamp.hour
                
                if day_key not in daily_peaks:
                    daily_peaks[day_key] = []
                daily_peaks[day_key].append(hour)
            
            # Calculate peak hour for each day
            daily_peak_hours = []
            for day, hours in daily_peaks.items():
                if len(hours) >= 3:  # Minimum activities per day
                    hour_counts = np.bincount(hours, minlength=24)
                    peak_hour = np.argmax(hour_counts)
                    daily_peak_hours.append(peak_hour)
            
            if len(daily_peak_hours) < 2:
                return 0.5  # Not enough data
            
            # Calculate consistency (lower variance = more consistent)
            variance = np.var(daily_peak_hours)
            consistency = max(0.0, 1.0 - variance / 24.0)
            
            return consistency
            
        except Exception:
            return 0.5

    def _identify_session_gaps(self, timestamps: List[datetime]) -> List[float]:
        """Identify gaps between sessions"""
        gaps = []
        current_session_end = None
        
        for i, timestamp in enumerate(timestamps):
            if current_session_end is None:
                current_session_end = timestamp
                continue
            
            # Gap of more than 30 minutes indicates new session
            time_diff = (timestamp - current_session_end).total_seconds()
            if time_diff > 1800:  # 30 minutes
                gaps.append(time_diff)
                current_session_end = timestamp
            else:
                current_session_end = timestamp
        
        return gaps

    def _calculate_entropy(self, items: List[str]) -> float:
        """Calculate Shannon entropy of items"""
        try:
            if not items:
                return 0.0
            
            # Count frequencies
            freq_dict = {}
            for item in items:
                freq_dict[item] = freq_dict.get(item, 0) + 1
            
            # Calculate probabilities
            total = len(items)
            entropy = 0.0
            
            for count in freq_dict.values():
                prob = count / total
                entropy -= prob * np.log2(prob + 1e-8)
            
            return entropy
            
        except Exception:
            return 1.0

    def _group_into_sessions(self, timestamps: List[datetime], 
                           session_gap_minutes: int = 30) -> List[List[datetime]]:
        """Group timestamps into sessions based on gaps"""
        if not timestamps:
            return []
        
        sessions = []
        current_session = [timestamps[0]]
        
        for i in range(1, len(timestamps)):
            time_gap = (timestamps[i] - timestamps[i-1]).total_seconds() / 60
            
            if time_gap <= session_gap_minutes:
                current_session.append(timestamps[i])
            else:
                sessions.append(current_session)
                current_session = [timestamps[i]]
        
        if current_session:
            sessions.append(current_session)
        
        return sessions

    def _calculate_response_consistency(self, activities: List[ActivityEvent]) -> float:
        """Calculate consistency in response times"""
        try:
            response_times = []
            
            # Look for comment/like responses to posts
            for i in range(1, len(activities)):
                prev_activity = activities[i-1]
                curr_activity = activities[i]
                
                # If current activity is a response to previous
                if (curr_activity.activity_type in [ActivityType.COMMENT, ActivityType.LIKE] and
                    prev_activity.activity_type == ActivityType.POST_CREATION):
                    
                    response_time = (curr_activity.timestamp - prev_activity.timestamp).total_seconds()
                    if response_time < 300:  # Within 5 minutes
                        response_times.append(response_time)
            
            if not response_times:
                return 0.5
            
            # Calculate coefficient of variation
            cv = np.std(response_times) / (np.mean(response_times) + 1e-8)
            consistency = max(0.0, 1.0 - cv / 2.0)  # Normalize
            
            return consistency
            
        except Exception:
            return 0.5

    def _analyze_weekend_pattern(self, timestamps: List[datetime]) -> float:
        """Analyze weekend vs weekday activity patterns"""
        try:
            weekday_count = 0
            weekend_count = 0
            
            for timestamp in timestamps:
                if timestamp.weekday() >= 5:  # Saturday = 5, Sunday = 6
                    weekend_count += 1
                else:
                    weekday_count += 1
            
            if weekday_count == 0:
                return 0.5  # No weekday data
            
            # Calculate weekend/weekday ratio
            ratio = weekend_count / weekday_count if weekday_count > 0 else 0
            
            # Score based on human-like pattern (typically less active on weekends)
            baseline_min, baseline_max = self.human_baselines['weekend_activity_ratio']
            
            if baseline_min <= ratio <= baseline_max:
                score = 1.0
            else:
                # Distance from acceptable range
                if ratio < baseline_min:
                    score = ratio / baseline_min
                else:
                    score = baseline_max / ratio
                score = max(0.0, score)
            
            return score
            
        except Exception:
            return 0.5

    def _detect_window_anomalies(self, timestamps: List[datetime], 
                                window: TemporalWindow) -> List[float]:
        """Detect anomalies within a temporal window"""
        try:
            anomalies = []
            window_size = timedelta(hours=window.duration_hours)
            
            # Sliding window analysis
            for i in range(len(timestamps)):
                window_start = timestamps[i]
                window_end = window_start + window_size
                
                # Count activities in window
                activities_in_window = sum(1 for ts in timestamps 
                                         if window_start <= ts <= window_end)
                
                # Check for suspicious patterns
                if activities_in_window > window.min_activities * 3:
                    # Too many activities - possible bot burst
                    anomaly_score = min(1.0, activities_in_window / (window.min_activities * 3))
                    anomalies.append(anomaly_score)
                elif activities_in_window > 0 and activities_in_window < window.min_activities / 3:
                    # Too few activities - possible automated pattern
                    anomaly_score = 0.3
                    anomalies.append(anomaly_score)
            
            return anomalies
            
        except Exception:
            return [0.0]

    def _score_session_duration(self, avg_duration: float) -> float:
        """Score session duration for human-likeness"""
        min_human, max_human = self.human_baselines['normal_session_duration']
        
        if min_human <= avg_duration <= max_human:
            return 1.0
        elif avg_duration < min_human:
            return max(0.0, avg_duration / min_human)
        else:
            # Very long sessions are suspicious
            return max(0.0, max_human / avg_duration)

    def _calculate_confidence_score(self, activities: List[ActivityEvent], 
                                  analysis_days: int) -> float:
        """Calculate confidence in the analysis"""
        try:
            # Base confidence on data quantity and quality
            activity_count_score = min(1.0, len(activities) / 100)  # More activities = higher confidence
            time_span_score = min(1.0, analysis_days / 30)  # Longer analysis = higher confidence
            
            # Diversity of activities
            activity_types = set(activity.activity_type for activity in activities)
            diversity_score = min(1.0, len(activity_types) / len(ActivityType))
            
            # Overall confidence
            confidence = (activity_count_score + time_span_score + diversity_score) / 3
            return max(0.1, confidence)  # Minimum 10% confidence
            
        except Exception:
            return 0.5

    def _create_default_features(self, user_id: str, analysis_days: int, 
                               activity_count: int) -> TemporalFeatures:
        """Create default features for users with insufficient data"""
        return TemporalFeatures(
            user_id=user_id,
            circadian_regularity=0.5,
            peak_activity_hour=20,  # Default to 8 PM
            activity_variance=1.0,
            timezone_consistency=0.5,
            burst_frequency=0.1,
            session_regularity=0.5,
            inter_activity_variance=1.0,
            activity_entropy=1.0,
            avg_session_duration=30.0,
            click_speed_variance=1.0,
            response_time_consistency=0.5,
            weekend_pattern_score=0.8,
            temporal_anomaly_score=0.3,
            human_likelihood=0.5,  # Neutral when no data
            bot_probability=0.5,
            analysis_period_days=analysis_days,
            total_activities=activity_count,
            confidence_score=0.1  # Low confidence due to insufficient data
        )

class TemporalBotDetector:
    """Main temporal bot detection class"""
    
    def __init__(self, config: Dict[str, Any]):
        self.config = config
        self.extractor = TemporalFeaturesExtractor(config)
        
        # Thresholds for bot detection
        self.bot_threshold = config.get('bot_threshold', 0.3)
        self.human_threshold = config.get('human_threshold', 0.7)
        
    async def analyze_user(self, user_id: str, activities: List[ActivityEvent]) -> Dict[str, Any]:
        """Analyze a user's temporal patterns for bot detection"""
        try:
            features = await self.extractor.extract_features(user_id, activities)
            
            # Determine classification
            if features.human_likelihood >= self.human_threshold:
                classification = "HUMAN"
                risk_level = "LOW"
            elif features.human_likelihood <= self.bot_threshold:
                classification = "BOT"
                risk_level = "HIGH"
            else:
                classification = "SUSPICIOUS"
                risk_level = "MEDIUM"
            
            return {
                'user_id': user_id,
                'classification': classification,
                'risk_level': risk_level,
                'human_likelihood': features.human_likelihood,
                'bot_probability': features.bot_probability,
                'confidence_score': features.confidence_score,
                'temporal_features': features,
                'recommendations': self._generate_recommendations(features),
                'timestamp': datetime.utcnow().isoformat()
            }
            
        except Exception as e:
            logger.error(f"Error analyzing user {user_id}: {str(e)}")
            return {
                'user_id': user_id,
                'classification': 'ERROR',
                'risk_level': 'UNKNOWN',
                'error': str(e),
                'timestamp': datetime.utcnow().isoformat()
            }
    
    def _generate_recommendations(self, features: TemporalFeatures) -> List[str]:
        """Generate recommendations based on temporal analysis"""
        recommendations = []
        
        if features.burst_frequency > 0.5:
            recommendations.append("HIGH_BURST_ACTIVITY: Monitor for automated behavior")
        
        if features.circadian_regularity < 0.2:
            recommendations.append("IRREGULAR_TIMING: No clear circadian pattern")
        
        if features.session_regularity > 0.95:
            recommendations.append("TOO_REGULAR: Suspiciously consistent patterns")
        
        if features.temporal_anomaly_score > 0.7:
            recommendations.append("TEMPORAL_ANOMALIES: Unusual timing patterns detected")
        
        if features.confidence_score < 0.3:
            recommendations.append("INSUFFICIENT_DATA: Need more activity data for reliable analysis")
        
        if not recommendations:
            recommendations.append("NORMAL_PATTERNS: Activity patterns appear human-like")
        
        return recommendations

# Usage example and utility functions
async def analyze_user_temporal_patterns(user_id: str, user_activities: List[Dict]) -> Dict[str, Any]:
    """
    Convenience function to analyze user temporal patterns
    
    Args:
        user_id: User identifier
        user_activities: List of user activities with timestamps
    
    Returns:
        Analysis results with bot detection scores
    """
    
    # Convert activities to ActivityEvent objects
    activities = []
    for activity in user_activities:
        try:
            event = ActivityEvent(
                user_id=user_id,
                timestamp=datetime.fromisoformat(activity['timestamp']),
                activity_type=ActivityType(activity['type']),
                platform=activity.get('platform', 'unknown'),
                metadata=activity.get('metadata', {}),
                ip_address=activity.get('ip_address'),
                user_agent=activity.get('user_agent')
            )
            activities.append(event)
        except (ValueError, KeyError) as e:
            logger.warning(f"Skipping invalid activity: {e}")
            continue
    
    # Initialize detector
    config = {
        'bot_threshold': 0.3,
        'human_threshold': 0.7,
        'analysis_days': 30
    }
    
    detector = TemporalBotDetector(config)
    return await detector.analyze_user(user_id, activities)

if __name__ == "__main__":
    # Test the temporal features extractor
    import asyncio
    
    async def test_temporal_analysis():
        # Sample test data
        test_activities = [
            {
                'timestamp': '2025-07-25T08:00:00',
                'type': 'login',
                'platform': 'mobile',
                'metadata': {'session_id': 'test1'}
            },
            {
                'timestamp': '2025-07-25T08:05:00',
                'type': 'post_creation',
                'platform': 'instagram',
                'metadata': {'content_type': 'image'}
            },
            {
                'timestamp': '2025-07-25T08:10:00',
                'type': 'mining_claim',
                'platform': 'mobile',
                'metadata': {'amount': 0.1}
            }
        ]
        
        result = await analyze_user_temporal_patterns('test_user_123', test_activities)
        print(json.dumps(result, indent=2, default=str))
    
    asyncio.run(test_temporal_analysis())
    