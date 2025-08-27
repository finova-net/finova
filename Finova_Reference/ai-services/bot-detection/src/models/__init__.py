"""
Finova Network - Bot Detection Models Module
Enterprise-grade AI models for detecting bot behavior and ensuring fair distribution

This module provides comprehensive bot detection capabilities through:
- Behavioral pattern analysis
- Human probability scoring  
- Network analysis for suspicious connections
- Real-time fraud detection
"""

import logging
from typing import Dict, List, Optional, Tuple, Union
from dataclasses import dataclass
from enum import Enum
import numpy as np
from datetime import datetime, timedelta

# Import all model classes
from .behavior_analyzer import BehaviorAnalyzer
from .pattern_detector import PatternDetector  
from .network_analyzer import NetworkAnalyzer
from .human_probability import HumanProbabilityCalculator

# Configure logging
logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)

class ThreatLevel(Enum):
    """Threat classification levels"""
    SAFE = "safe"
    LOW = "low" 
    MEDIUM = "medium"
    HIGH = "high"
    CRITICAL = "critical"

class DetectionType(Enum):
    """Types of bot detection"""
    BEHAVIORAL = "behavioral"
    PATTERN = "pattern"
    NETWORK = "network"
    BIOMETRIC = "biometric"
    HYBRID = "hybrid"

@dataclass
class DetectionResult:
    """Standardized detection result"""
    user_id: str
    threat_level: ThreatLevel
    confidence_score: float  # 0.0 - 1.0
    human_probability: float  # 0.0 - 1.0
    detection_types: List[DetectionType]
    risk_factors: Dict[str, float]
    recommendations: List[str]
    timestamp: datetime
    
class BotDetectionOrchestrator:
    """
    Main orchestrator for all bot detection models
    Coordinates between different detection systems for comprehensive analysis
    """
    
    def __init__(self, config: Optional[Dict] = None):
        self.config = config or self._default_config()
        
        # Initialize all detection models
        self.behavior_analyzer = BehaviorAnalyzer(self.config.get('behavior', {}))
        self.pattern_detector = PatternDetector(self.config.get('pattern', {}))
        self.network_analyzer = NetworkAnalyzer(self.config.get('network', {}))
        self.human_calculator = HumanProbabilityCalculator(self.config.get('human', {}))
        
        # Detection thresholds
        self.thresholds = {
            ThreatLevel.LOW: 0.3,
            ThreatLevel.MEDIUM: 0.5, 
            ThreatLevel.HIGH: 0.7,
            ThreatLevel.CRITICAL: 0.9
        }
        
        logger.info("BotDetectionOrchestrator initialized successfully")
    
    def analyze_user(self, user_data: Dict) -> DetectionResult:
        """
        Comprehensive user analysis using all detection models
        
        Args:
            user_data: Complete user data including activity, network, biometrics
            
        Returns:
            DetectionResult with comprehensive threat assessment
        """
        try:
            # Run all detection models
            behavior_score = self.behavior_analyzer.analyze(user_data)
            pattern_score = self.pattern_detector.detect_patterns(user_data)
            network_score = self.network_analyzer.analyze_network(user_data)
            human_prob = self.human_calculator.calculate_probability(user_data)
            
            # Weighted ensemble scoring
            composite_score = self._calculate_composite_score(
                behavior_score, pattern_score, network_score, human_prob
            )
            
            # Determine threat level
            threat_level = self._determine_threat_level(composite_score)
            
            # Extract risk factors
            risk_factors = self._extract_risk_factors(
                user_data, behavior_score, pattern_score, network_score
            )
            
            # Generate recommendations
            recommendations = self._generate_recommendations(threat_level, risk_factors)
            
            return DetectionResult(
                user_id=user_data.get('user_id', 'unknown'),
                threat_level=threat_level,
                confidence_score=composite_score,
                human_probability=human_prob,
                detection_types=self._get_active_detection_types(user_data),
                risk_factors=risk_factors,
                recommendations=recommendations,
                timestamp=datetime.utcnow()
            )
            
        except Exception as e:
            logger.error(f"Error analyzing user {user_data.get('user_id')}: {str(e)}")
            return self._create_error_result(user_data.get('user_id', 'unknown'))
    
    def _calculate_composite_score(self, behavior: float, pattern: float, 
                                 network: float, human: float) -> float:
        """Calculate weighted composite threat score"""
        weights = {
            'behavior': 0.3,
            'pattern': 0.25, 
            'network': 0.25,
            'human': 0.2
        }
        
        # Invert human probability for threat scoring
        human_threat = 1.0 - human
        
        composite = (
            behavior * weights['behavior'] +
            pattern * weights['pattern'] + 
            network * weights['network'] +
            human_threat * weights['human']
        )
        
        return min(max(composite, 0.0), 1.0)
    
    def _determine_threat_level(self, score: float) -> ThreatLevel:
        """Determine threat level based on composite score"""
        if score >= self.thresholds[ThreatLevel.CRITICAL]:
            return ThreatLevel.CRITICAL
        elif score >= self.thresholds[ThreatLevel.HIGH]:
            return ThreatLevel.HIGH
        elif score >= self.thresholds[ThreatLevel.MEDIUM]:
            return ThreatLevel.MEDIUM
        elif score >= self.thresholds[ThreatLevel.LOW]:
            return ThreatLevel.LOW
        else:
            return ThreatLevel.SAFE
    
    def _extract_risk_factors(self, user_data: Dict, behavior: float, 
                            pattern: float, network: float) -> Dict[str, float]:
        """Extract detailed risk factors for transparency"""
        return {
            'behavioral_anomalies': behavior,
            'pattern_consistency': pattern,
            'network_suspicious': network,
            'activity_volume': self._analyze_activity_volume(user_data),
            'temporal_patterns': self._analyze_temporal_patterns(user_data),
            'device_fingerprint': self._analyze_device_patterns(user_data),
            'social_graph_validity': self._analyze_social_connections(user_data)
        }
    
    def _generate_recommendations(self, threat_level: ThreatLevel, 
                                risk_factors: Dict[str, float]) -> List[str]:
        """Generate actionable recommendations based on threat assessment"""
        recommendations = []
        
        if threat_level == ThreatLevel.CRITICAL:
            recommendations.extend([
                "Immediate account suspension required",
                "Manual review by security team",
                "Block all mining activities",
                "Investigate related accounts"
            ])
        elif threat_level == ThreatLevel.HIGH:
            recommendations.extend([
                "Reduce mining rate by 80%",
                "Require additional verification",
                "Monitor all activities closely",
                "Flag for manual review"
            ])
        elif threat_level == ThreatLevel.MEDIUM:
            recommendations.extend([
                "Reduce mining rate by 50%",
                "Increase verification requirements",
                "Monitor for pattern changes"
            ])
        elif threat_level == ThreatLevel.LOW:
            recommendations.extend([
                "Reduce mining rate by 20%",
                "Standard monitoring protocol"
            ])
        
        # Add specific recommendations based on risk factors
        if risk_factors.get('activity_volume', 0) > 0.8:
            recommendations.append("Implement activity cooldown periods")
        
        if risk_factors.get('network_suspicious', 0) > 0.7:
            recommendations.append("Investigate referral network for coordinated behavior")
            
        return recommendations
    
    def _get_active_detection_types(self, user_data: Dict) -> List[DetectionType]:
        """Determine which detection types are active"""
        active_types = [DetectionType.BEHAVIORAL, DetectionType.PATTERN]
        
        if user_data.get('referral_network'):
            active_types.append(DetectionType.NETWORK)
        
        if user_data.get('biometric_data'):
            active_types.append(DetectionType.BIOMETRIC)
            
        return active_types
    
    def _analyze_activity_volume(self, user_data: Dict) -> float:
        """Analyze if activity volume is suspiciously high"""
        daily_activities = user_data.get('daily_activity_count', 0)
        
        # Human baseline: 10-100 activities per day
        if daily_activities > 500:
            return 1.0  # Definitely suspicious
        elif daily_activities > 200:
            return 0.8  # Very suspicious
        elif daily_activities > 100:
            return 0.5  # Moderately suspicious
        else:
            return 0.0  # Normal range
    
    def _analyze_temporal_patterns(self, user_data: Dict) -> float:
        """Analyze temporal patterns for bot-like behavior"""
        activity_times = user_data.get('activity_timestamps', [])
        
        if len(activity_times) < 10:
            return 0.0
        
        # Check for too-regular patterns (bot characteristic)
        intervals = [activity_times[i] - activity_times[i-1] 
                    for i in range(1, len(activity_times))]
        
        # Calculate coefficient of variation
        if not intervals:
            return 0.0
            
        mean_interval = np.mean(intervals)
        std_interval = np.std(intervals)
        
        if mean_interval == 0:
            return 1.0  # Extremely suspicious
        
        cv = std_interval / mean_interval
        
        # Low coefficient of variation indicates regular patterns (bot-like)
        if cv < 0.1:
            return 0.9
        elif cv < 0.3:
            return 0.6
        elif cv < 0.5:
            return 0.3
        else:
            return 0.0
    
    def _analyze_device_patterns(self, user_data: Dict) -> float:
        """Analyze device fingerprint patterns"""
        device_data = user_data.get('device_fingerprint', {})
        
        risk_score = 0.0
        
        # Check for headless browser indicators
        if device_data.get('webdriver_detected', False):
            risk_score += 0.4
        
        # Check for automation frameworks
        if device_data.get('automation_detected', False):
            risk_score += 0.5
        
        # Check for suspicious user agent
        user_agent = device_data.get('user_agent', '')
        if any(bot_indicator in user_agent.lower() 
               for bot_indicator in ['headless', 'phantom', 'selenium', 'bot']):
            risk_score += 0.3
        
        return min(risk_score, 1.0)
    
    def _analyze_social_connections(self, user_data: Dict) -> float:
        """Analyze social graph validity"""
        connections = user_data.get('social_connections', [])
        
        if len(connections) == 0:
            return 0.5  # No social proof
        
        # Check for suspicious connection patterns
        suspicious_indicators = 0
        
        # Check for too many connections in short time
        recent_connections = [c for c in connections 
                            if (datetime.utcnow() - c.get('created_at', datetime.min)).days < 7]
        
        if len(recent_connections) > 50:
            suspicious_indicators += 1
        
        # Check for reciprocal bot networks
        reciprocal_rate = sum(1 for c in connections if c.get('reciprocal', False)) / len(connections)
        if reciprocal_rate > 0.9:
            suspicious_indicators += 1
        
        return min(suspicious_indicators * 0.4, 1.0)
    
    def _create_error_result(self, user_id: str) -> DetectionResult:
        """Create error result when analysis fails"""
        return DetectionResult(
            user_id=user_id,
            threat_level=ThreatLevel.MEDIUM,  # Conservative approach
            confidence_score=0.0,
            human_probability=0.5,
            detection_types=[],
            risk_factors={},
            recommendations=["Manual review required due to analysis error"],
            timestamp=datetime.utcnow()
        )
    
    def _default_config(self) -> Dict:
        """Default configuration for bot detection"""
        return {
            'behavior': {
                'analysis_window_hours': 24,
                'min_activities_for_analysis': 5,
                'anomaly_threshold': 0.7
            },
            'pattern': {
                'sequence_length': 10,
                'similarity_threshold': 0.85,
                'update_frequency': 3600  # 1 hour
            },
            'network': {
                'max_depth': 3,
                'suspicious_cluster_size': 10,
                'connection_velocity_threshold': 50
            },
            'human': {
                'biometric_weight': 0.4,
                'behavioral_weight': 0.3,
                'social_weight': 0.2,
                'device_weight': 0.1
            }
        }

# Convenience functions for external use
def analyze_user_for_bot_behavior(user_data: Dict, config: Optional[Dict] = None) -> DetectionResult:
    """
    Convenience function for single user analysis
    
    Args:
        user_data: User data dictionary
        config: Optional configuration override
        
    Returns:
        DetectionResult with threat assessment
    """
    orchestrator = BotDetectionOrchestrator(config)
    return orchestrator.analyze_user(user_data)

def batch_analyze_users(users_data: List[Dict], config: Optional[Dict] = None) -> List[DetectionResult]:
    """
    Batch analysis for multiple users
    
    Args:
        users_data: List of user data dictionaries
        config: Optional configuration override
        
    Returns:
        List of DetectionResult objects
    """
    orchestrator = BotDetectionOrchestrator(config)
    results = []
    
    for user_data in users_data:
        try:
            result = orchestrator.analyze_user(user_data)
            results.append(result)
        except Exception as e:
            logger.error(f"Error analyzing user {user_data.get('user_id')}: {str(e)}")
            results.append(orchestrator._create_error_result(user_data.get('user_id', 'unknown')))
    
    return results

def get_threat_level_penalties() -> Dict[ThreatLevel, Dict[str, float]]:
    """
    Get mining and reward penalties for each threat level
    
    Returns:
        Dictionary mapping threat levels to penalty multipliers
    """
    return {
        ThreatLevel.SAFE: {
            'mining_penalty': 0.0,
            'xp_penalty': 0.0,
            'rp_penalty': 0.0
        },
        ThreatLevel.LOW: {
            'mining_penalty': 0.2,  # 20% reduction
            'xp_penalty': 0.1,      # 10% reduction  
            'rp_penalty': 0.1       # 10% reduction
        },
        ThreatLevel.MEDIUM: {
            'mining_penalty': 0.5,  # 50% reduction
            'xp_penalty': 0.3,      # 30% reduction
            'rp_penalty': 0.4       # 40% reduction
        },
        ThreatLevel.HIGH: {
            'mining_penalty': 0.8,  # 80% reduction
            'xp_penalty': 0.7,      # 70% reduction
            'rp_penalty': 0.8       # 80% reduction
        },
        ThreatLevel.CRITICAL: {
            'mining_penalty': 1.0,  # 100% reduction (blocked)
            'xp_penalty': 1.0,      # 100% reduction (blocked)
            'rp_penalty': 1.0       # 100% reduction (blocked)
        }
    }

def calculate_human_probability_fast(user_data: Dict) -> float:
    """
    Fast human probability calculation for real-time use
    
    Args:
        user_data: Minimal user data for quick analysis
        
    Returns:
        Human probability score (0.0 - 1.0)
    """
    calculator = HumanProbabilityCalculator()
    return calculator.quick_assessment(user_data)

# Export main classes and functions
__all__ = [
    'BotDetectionOrchestrator',
    'BehaviorAnalyzer', 
    'PatternDetector',
    'NetworkAnalyzer',
    'HumanProbabilityCalculator',
    'DetectionResult',
    'ThreatLevel',
    'DetectionType',
    'analyze_user_for_bot_behavior',
    'batch_analyze_users', 
    'get_threat_level_penalties',
    'calculate_human_probability_fast'
]

# Version info
__version__ = "1.0.0"
__author__ = "Finova Network Team"
__description__ = "Enterprise-grade bot detection for fair token distribution"

# Initialize global orchestrator for module-level access
_global_orchestrator = None

def get_global_orchestrator(config: Optional[Dict] = None) -> BotDetectionOrchestrator:
    """Get or create global orchestrator instance"""
    global _global_orchestrator
    if _global_orchestrator is None:
        _global_orchestrator = BotDetectionOrchestrator(config)
    return _global_orchestrator

def reset_global_orchestrator():
    """Reset global orchestrator (useful for testing)"""
    global _global_orchestrator
    _global_orchestrator = None

# Configuration validation
def validate_config(config: Dict) -> bool:
    """
    Validate bot detection configuration
    
    Args:
        config: Configuration dictionary
        
    Returns:
        True if valid, False otherwise
    """
    required_sections = ['behavior', 'pattern', 'network', 'human']
    
    for section in required_sections:
        if section not in config:
            logger.warning(f"Missing required config section: {section}")
            return False
    
    # Validate threshold values
    behavior_config = config.get('behavior', {})
    if behavior_config.get('anomaly_threshold', 0) > 1.0:
        logger.warning("Anomaly threshold must be <= 1.0")
        return False
    
    return True

# Performance monitoring
class PerformanceMonitor:
    """Monitor bot detection performance metrics"""
    
    def __init__(self):
        self.metrics = {
            'total_analyses': 0,
            'threat_detections': 0,
            'false_positives': 0,
            'analysis_time_avg': 0.0,
            'last_reset': datetime.utcnow()
        }
    
    def record_analysis(self, result: DetectionResult, analysis_time: float):
        """Record analysis metrics"""
        self.metrics['total_analyses'] += 1
        self.metrics['analysis_time_avg'] = (
            (self.metrics['analysis_time_avg'] * (self.metrics['total_analyses'] - 1) + analysis_time) /
            self.metrics['total_analyses']
        )
        
        if result.threat_level != ThreatLevel.SAFE:
            self.metrics['threat_detections'] += 1
    
    def get_detection_rate(self) -> float:
        """Get current threat detection rate"""
        if self.metrics['total_analyses'] == 0:
            return 0.0
        return self.metrics['threat_detections'] / self.metrics['total_analyses']
    
    def reset_metrics(self):
        """Reset performance metrics"""
        self.metrics = {
            'total_analyses': 0,
            'threat_detections': 0, 
            'false_positives': 0,
            'analysis_time_avg': 0.0,
            'last_reset': datetime.utcnow()
        }

# Global performance monitor
performance_monitor = PerformanceMonitor()

logger.info("Finova Bot Detection Models module loaded successfully")
