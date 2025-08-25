"""
Finova Network - Bot Detection API Service
Enterprise-grade bot detection and anti-fraud system
"""

from flask import Flask, request, jsonify, g
from flask_cors import CORS
from flask_limiter import Limiter
from flask_limiter.util import get_remote_address
import redis
import logging
import os
from datetime import datetime, timedelta
import jwt
import hashlib
import numpy as np
from functools import wraps
from typing import Dict, Any, Optional
import asyncio
from concurrent.futures import ThreadPoolExecutor

# Import Finova bot detection models
from ..models.behavior_analyzer import BehaviorAnalyzer
from ..models.pattern_detector import PatternDetector
from ..models.network_analyzer import NetworkAnalyzer
from ..models.human_probability import HumanProbabilityCalculator

# Import feature extractors
from ..features.temporal_features import TemporalFeatureExtractor
from ..features.behavioral_features import BehavioralFeatureExtractor
from ..features.network_features import NetworkFeatureExtractor
from ..features.device_features import DeviceFeatureExtractor

# Import utilities
from ..utils.config import Config
from ..utils.helpers import SecurityHelper, ValidationHelper

# Configure logging
logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)

# Initialize Redis connection
redis_client = redis.Redis(
    host=os.getenv('REDIS_HOST', 'localhost'),
    port=int(os.getenv('REDIS_PORT', 6379)),
    password=os.getenv('REDIS_PASSWORD'),
    decode_responses=True
)

class FinovaBotDetectionAPI:
    """Enterprise-grade bot detection API for Finova Network"""
    
    def __init__(self):
        self.app = Flask(__name__)
        self.app.config.from_object(Config)
        
        # Enable CORS with security headers
        CORS(self.app, origins=Config.ALLOWED_ORIGINS)
        
        # Rate limiting
        self.limiter = Limiter(
            app=self.app,
            key_func=get_remote_address,
            storage_uri=f"redis://{os.getenv('REDIS_HOST', 'localhost')}:6379"
        )
        
        # Initialize ML models
        self.behavior_analyzer = BehaviorAnalyzer()
        self.pattern_detector = PatternDetector()
        self.network_analyzer = NetworkAnalyzer()
        self.human_calc = HumanProbabilityCalculator()
        
        # Initialize feature extractors
        self.temporal_extractor = TemporalFeatureExtractor()
        self.behavioral_extractor = BehavioralFeatureExtractor()
        self.network_extractor = NetworkFeatureExtractor()
        self.device_extractor = DeviceFeatureExtractor()
        
        # Thread pool for async processing
        self.executor = ThreadPoolExecutor(max_workers=10)
        
        self._register_routes()
        self._setup_error_handlers()
        
    def _register_routes(self):
        """Register all API endpoints"""
        
        @self.app.before_request
        def before_request():
            """Security checks and request preprocessing"""
            g.start_time = datetime.utcnow()
            
            # Rate limiting check
            if not self._check_rate_limit():
                return jsonify({'error': 'Rate limit exceeded'}), 429
            
            # Authentication for protected endpoints
            if request.endpoint in ['analyze_user', 'bulk_analysis', 'update_model']:
                if not self._authenticate_request():
                    return jsonify({'error': 'Unauthorized'}), 401
        
        @self.app.after_request
        def after_request(response):
            """Add security headers and logging"""
            response.headers['X-Content-Type-Options'] = 'nosniff'
            response.headers['X-Frame-Options'] = 'DENY'
            response.headers['X-XSS-Protection'] = '1; mode=block'
            
            # Log request
            duration = (datetime.utcnow() - g.start_time).total_seconds()
            logger.info(f"Request: {request.method} {request.path} - {response.status_code} - {duration:.3f}s")
            
            return response
        
        # Health check endpoint
        @self.app.route('/health', methods=['GET'])
        def health_check():
            """Health check endpoint"""
            return jsonify({
                'status': 'healthy',
                'timestamp': datetime.utcnow().isoformat(),
                'version': '1.0.0',
                'models_loaded': True
            })
        
        # Main bot detection endpoint
        @self.app.route('/analyze', methods=['POST'])
        @self.limiter.limit("100 per minute")
        def analyze_user():
            """Analyze user for bot probability"""
            try:
                data = request.get_json()
                if not data or 'user_id' not in data:
                    return jsonify({'error': 'Missing user_id'}), 400
                
                result = self._analyze_user_comprehensive(data)
                return jsonify(result)
                
            except Exception as e:
                logger.error(f"Analysis error: {str(e)}")
                return jsonify({'error': 'Analysis failed'}), 500
        
        # Bulk analysis endpoint
        @self.app.route('/bulk-analyze', methods=['POST'])
        @self.limiter.limit("10 per minute")
        def bulk_analysis():
            """Bulk analyze multiple users"""
            try:
                data = request.get_json()
                if not data or 'users' not in data:
                    return jsonify({'error': 'Missing users array'}), 400
                
                results = self._process_bulk_analysis(data['users'])
                return jsonify({'results': results})
                
            except Exception as e:
                logger.error(f"Bulk analysis error: {str(e)}")
                return jsonify({'error': 'Bulk analysis failed'}), 500
        
        # Real-time risk assessment
        @self.app.route('/risk-assessment', methods=['POST'])
        @self.limiter.limit("500 per minute")
        def risk_assessment():
            """Real-time risk assessment for actions"""
            try:
                data = request.get_json()
                risk_score = self._calculate_action_risk(data)
                
                return jsonify({
                    'risk_score': risk_score,
                    'action_allowed': risk_score < 0.7,
                    'recommended_action': self._get_risk_recommendation(risk_score)
                })
                
            except Exception as e:
                logger.error(f"Risk assessment error: {str(e)}")
                return jsonify({'error': 'Risk assessment failed'}), 500
        
        # Pattern reporting endpoint
        @self.app.route('/report-pattern', methods=['POST'])
        @self.limiter.limit("50 per minute")
        def report_pattern():
            """Report suspicious pattern detection"""
            try:
                data = request.get_json()
                pattern_id = self._process_pattern_report(data)
                
                return jsonify({
                    'pattern_id': pattern_id,
                    'status': 'recorded',
                    'investigation_started': True
                })
                
            except Exception as e:
                logger.error(f"Pattern reporting error: {str(e)}")
                return jsonify({'error': 'Pattern reporting failed'}), 500
    
    def _analyze_user_comprehensive(self, data: Dict[str, Any]) -> Dict[str, Any]:
        """Comprehensive user analysis using all detection models"""
        user_id = data['user_id']
        
        # Extract features from all dimensions
        temporal_features = self.temporal_extractor.extract_features(data)
        behavioral_features = self.behavioral_extractor.extract_features(data)
        network_features = self.network_extractor.extract_features(data)
        device_features = self.device_extractor.extract_features(data)
        
        # Run analysis models
        behavior_score = self.behavior_analyzer.analyze(behavioral_features)
        pattern_score = self.pattern_detector.detect_patterns(temporal_features)
        network_score = self.network_analyzer.analyze_network(network_features)
        
        # Calculate human probability
        human_probability = self.human_calc.calculate_probability({
            'temporal': temporal_features,
            'behavioral': behavioral_features,
            'network': network_features,
            'device': device_features
        })
        
        # Weighted final score (based on Finova whitepaper specs)
        weights = {
            'behavior': 0.30,
            'pattern': 0.25,
            'network': 0.25,
            'human_prob': 0.20
        }
        
        final_score = (
            behavior_score * weights['behavior'] +
            pattern_score * weights['pattern'] +
            network_score * weights['network'] +
            (1 - human_probability) * weights['human_prob']
        )
        
        # Risk classification
        risk_level = self._classify_risk(final_score)
        
        # Cache result
        self._cache_analysis_result(user_id, {
            'bot_probability': final_score,
            'human_probability': human_probability,
            'risk_level': risk_level,
            'timestamp': datetime.utcnow().isoformat()
        })
        
        return {
            'user_id': user_id,
            'bot_probability': round(final_score, 4),
            'human_probability': round(human_probability, 4),
            'risk_level': risk_level,
            'confidence': self._calculate_confidence(temporal_features, behavioral_features),
            'flags': self._get_risk_flags(behavior_score, pattern_score, network_score),
            'recommendations': self._generate_recommendations(final_score, risk_level),
            'analysis_timestamp': datetime.utcnow().isoformat()
        }
    
    def _process_bulk_analysis(self, users: list) -> list:
        """Process multiple users in parallel"""
        futures = []
        
        for user_data in users[:50]:  # Limit bulk size
            future = self.executor.submit(self._analyze_user_comprehensive, user_data)
            futures.append(future)
        
        results = []
        for future in futures:
            try:
                result = future.result(timeout=30)
                results.append(result)
            except Exception as e:
                logger.error(f"Bulk analysis individual error: {str(e)}")
                results.append({'error': 'Analysis failed for user'})
        
        return results
    
    def _calculate_action_risk(self, data: Dict[str, Any]) -> float:
        """Calculate risk score for specific actions (mining, XP gain, referral)"""
        action_type = data.get('action_type', 'unknown')
        user_context = data.get('user_context', {})
        
        # Base risk scores by action type (from Finova anti-bot specs)
        base_risks = {
            'mining': 0.3,
            'xp_gain': 0.2,
            'referral_action': 0.4,
            'social_post': 0.1,
            'nft_purchase': 0.5
        }
        
        base_risk = base_risks.get(action_type, 0.5)
        
        # Contextual modifiers
        modifiers = []
        
        # Frequency modifier
        if data.get('recent_action_count', 0) > 10:
            modifiers.append(0.3)
        
        # Time pattern modifier
        if self._detect_suspicious_timing(data.get('action_history', [])):
            modifiers.append(0.4)
        
        # Device consistency modifier
        if not self._check_device_consistency(data.get('device_fingerprint')):
            modifiers.append(0.2)
        
        # Apply modifiers
        final_risk = base_risk + sum(modifiers)
        return min(1.0, final_risk)
    
    def _classify_risk(self, score: float) -> str:
        """Classify risk level based on bot probability"""
        if score < 0.2:
            return 'low'
        elif score < 0.5:
            return 'medium'
        elif score < 0.8:
            return 'high'
        else:
            return 'critical'
    
    def _get_risk_flags(self, behavior_score: float, pattern_score: float, network_score: float) -> list:
        """Generate specific risk flags"""
        flags = []
        
        if behavior_score > 0.7:
            flags.append('suspicious_behavior_patterns')
        if pattern_score > 0.7:
            flags.append('automated_timing_detected')
        if network_score > 0.7:
            flags.append('suspicious_network_activity')
        
        return flags
    
    def _generate_recommendations(self, score: float, risk_level: str) -> list:
        """Generate actionable recommendations"""
        recommendations = []
        
        if risk_level == 'critical':
            recommendations.extend([
                'immediate_account_suspension',
                'manual_verification_required',
                'investigate_network_connections'
            ])
        elif risk_level == 'high':
            recommendations.extend([
                'additional_verification_required',
                'reduce_mining_rewards',
                'monitor_closely'
            ])
        elif risk_level == 'medium':
            recommendations.extend([
                'enhanced_monitoring',
                'periodic_verification'
            ])
        
        return recommendations
    
    def _cache_analysis_result(self, user_id: str, result: Dict[str, Any]):
        """Cache analysis result in Redis"""
        cache_key = f"bot_analysis:{user_id}"
        redis_client.setex(cache_key, 3600, str(result))  # Cache for 1 hour
    
    def _authenticate_request(self) -> bool:
        """Authenticate API requests using JWT"""
        auth_header = request.headers.get('Authorization', '')
        if not auth_header.startswith('Bearer '):
            return False
        
        token = auth_header.split(' ')[1]
        try:
            jwt.decode(token, Config.JWT_SECRET, algorithms=['HS256'])
            return True
        except jwt.InvalidTokenError:
            return False
    
    def _check_rate_limit(self) -> bool:
        """Custom rate limiting logic"""
        client_ip = get_remote_address()
        key = f"rate_limit:{client_ip}"
        
        current_requests = redis_client.get(key)
        if current_requests and int(current_requests) > 1000:  # 1000 requests per hour
            return False
        
        redis_client.incr(key)
        redis_client.expire(key, 3600)
        return True
    
    def _setup_error_handlers(self):
        """Setup custom error handlers"""
        
        @self.app.errorhandler(404)
        def not_found(error):
            return jsonify({'error': 'Endpoint not found'}), 404
        
        @self.app.errorhandler(500)
        def internal_error(error):
            logger.error(f"Internal error: {str(error)}")
            return jsonify({'error': 'Internal server error'}), 500
    
    def _detect_suspicious_timing(self, action_history: list) -> bool:
        """Detect suspicious timing patterns"""
        if len(action_history) < 5:
            return False
        
        intervals = []
        for i in range(1, len(action_history)):
            prev_time = datetime.fromisoformat(action_history[i-1]['timestamp'])
            curr_time = datetime.fromisoformat(action_history[i]['timestamp'])
            intervals.append((curr_time - prev_time).total_seconds())
        
        # Check for too regular patterns (bot-like)
        if len(set(intervals)) < 3 and len(intervals) > 10:
            return True
        
        return False
    
    def _check_device_consistency(self, device_fingerprint: str) -> bool:
        """Check device fingerprint consistency"""
        if not device_fingerprint:
            return False
        
        # Simple consistency check - in production, use more sophisticated methods
        cached_fingerprint = redis_client.get(f"device:{hashlib.sha256(device_fingerprint.encode()).hexdigest()}")
        return cached_fingerprint is not None
    
    def _calculate_confidence(self, temporal_features: dict, behavioral_features: dict) -> float:
        """Calculate confidence score for the analysis"""
        data_quality = len(temporal_features.get('activities', [])) / 100.0
        feature_completeness = len([f for f in behavioral_features.values() if f is not None]) / len(behavioral_features)
        
        return min(1.0, (data_quality + feature_completeness) / 2.0)
    
    def _get_risk_recommendation(self, risk_score: float) -> str:
        """Get recommendation based on risk score"""
        if risk_score < 0.3:
            return 'allow_normal_operations'
        elif risk_score < 0.7:
            return 'enhanced_monitoring'
        else:
            return 'require_additional_verification'
    
    def _process_pattern_report(self, data: Dict[str, Any]) -> str:
        """Process pattern reports from the community"""
        pattern_id = hashlib.sha256(f"{data}{datetime.utcnow()}".encode()).hexdigest()[:12]
        
        # Store pattern report
        redis_client.setex(
            f"pattern_report:{pattern_id}",
            86400,  # 24 hours
            str(data)
        )
        
        return pattern_id

# Global API instance
bot_detection_api = FinovaBotDetectionAPI()
app = bot_detection_api.app

# Export for external use
__all__ = ['app', 'bot_detection_api', 'FinovaBotDetectionAPI']
