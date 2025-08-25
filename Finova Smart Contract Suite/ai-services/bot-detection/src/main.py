#!/usr/bin/env python3
"""
Finova Network Bot Detection Service
Enterprise-grade anti-bot system with multi-layer protection
"""

import asyncio
import logging
import os
import sys
import time
from datetime import datetime, timedelta
from typing import Dict, List, Optional, Tuple, Any
from dataclasses import dataclass, asdict
from contextlib import asynccontextmanager
import hashlib
import json
import numpy as np
import pandas as pd
from fastapi import FastAPI, HTTPException, Depends, BackgroundTasks
from fastapi.middleware.cors import CORSMiddleware
from fastapi.middleware.trustedhost import TrustedHostMiddleware
from fastapi.security import HTTPBearer, HTTPAuthorizationCredentials
from pydantic import BaseModel, Field, validator
from redis.asyncio import Redis
from sqlalchemy.ext.asyncio import AsyncSession, create_async_engine
from sqlalchemy.orm import sessionmaker
import jwt
from sklearn.ensemble import IsolationForest
from sklearn.preprocessing import StandardScaler
import joblib

# Import local modules
from models.behavior_analyzer import BehaviorAnalyzer
from models.pattern_detector import PatternDetector
from models.network_analyzer import NetworkAnalyzer
from models.human_probability import HumanProbabilityCalculator
from features.temporal_features import TemporalFeatureExtractor
from features.behavioral_features import BehavioralFeatureExtractor
from features.network_features import NetworkFeatureExtractor
from features.device_features import DeviceFeatureExtractor
from api.routes import router as api_router
from utils.config import Config
from utils.helpers import SecurityHelpers, CacheManager

# Configure logging
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(name)s - %(levelname)s - %(message)s'
)
logger = logging.getLogger(__name__)

# Security configuration
security = HTTPBearer()

@dataclass
class UserAnalysisResult:
    """Comprehensive user analysis result"""
    user_id: str
    human_probability: float
    risk_score: float
    behavior_score: float
    network_score: float
    device_score: float
    temporal_score: float
    quality_score: float
    final_recommendation: str
    confidence_level: float
    analysis_timestamp: datetime
    detailed_flags: Dict[str, Any]
    mitigation_actions: List[str]

@dataclass
class MiningPenalty:
    """Mining penalty calculation result"""
    user_id: str
    base_penalty: float
    behavior_penalty: float
    network_penalty: float
    quality_penalty: float
    total_penalty: float
    penalty_duration: int  # hours
    justification: str

class UserActivityRequest(BaseModel):
    """Request model for user activity analysis"""
    user_id: str = Field(..., description="Unique user identifier")
    session_id: str = Field(..., description="Current session ID")
    activity_type: str = Field(..., description="Type of activity")
    platform: str = Field(..., description="Social media platform")
    content_data: Optional[Dict] = Field(None, description="Content metadata")
    device_info: Dict = Field(..., description="Device fingerprint data")
    timestamp: datetime = Field(default_factory=datetime.utcnow)
    
    @validator('activity_type')
    def validate_activity_type(cls, v):
        allowed_types = [
            'login', 'post', 'comment', 'like', 'share', 'follow',
            'mining_claim', 'xp_action', 'referral_action', 'nft_interaction'
        ]
        if v not in allowed_types:
            raise ValueError(f'Activity type must be one of {allowed_types}')
        return v

class BotDetectionResponse(BaseModel):
    """Response model for bot detection analysis"""
    user_id: str
    is_human_probable: bool
    human_probability: float
    risk_level: str
    mining_penalty: float
    xp_penalty: float
    rp_penalty: float
    recommendations: List[str]
    next_analysis_time: datetime

class FinovaBotDetectionService:
    """Main bot detection service for Finova Network"""
    
    def __init__(self):
        self.config = Config()
        self.redis: Optional[Redis] = None
        self.db_engine = None
        self.db_session = None
        
        # Initialize AI models
        self.behavior_analyzer = BehaviorAnalyzer()
        self.pattern_detector = PatternDetector()
        self.network_analyzer = NetworkAnalyzer()
        self.human_probability_calc = HumanProbabilityCalculator()
        
        # Initialize feature extractors
        self.temporal_extractor = TemporalFeatureExtractor()
        self.behavioral_extractor = BehavioralFeatureExtractor()
        self.network_extractor = NetworkFeatureExtractor()
        self.device_extractor = DeviceFeatureExtractor()
        
        # Initialize utilities
        self.security_helpers = SecurityHelpers()
        self.cache_manager = CacheManager()
        
        # Load ML models
        self._load_ml_models()
        
        # Performance metrics
        self.analysis_count = 0
        self.bot_detection_count = 0
        self.false_positive_count = 0
        
    def _load_ml_models(self):
        """Load pre-trained machine learning models"""
        try:
            model_path = self.config.ML_MODELS_PATH
            self.isolation_forest = joblib.load(f"{model_path}/isolation_forest.pkl")
            self.scaler = joblib.load(f"{model_path}/feature_scaler.pkl")
            logger.info("ML models loaded successfully")
        except Exception as e:
            logger.error(f"Server startup failed: {e}")
        sys.exit(1)
    finally:
        await bot_detection_service.shutdown()

# Additional helper methods for the service
async def _fetch_user_info(session: AsyncSession, user_id: str) -> Optional[Dict[str, Any]]:
    """Fetch basic user information from database"""
    try:
        # This would be actual database query in production
        # Simulated for example
        query = """
        SELECT user_id, created_at, kyc_verified, reputation_score, 
               total_activities, last_login, suspicious_flags
        FROM users WHERE user_id = :user_id
        """
        
        # Placeholder for actual database implementation
        return {
            'user_id': user_id,
            'account_age_days': 30,  # Calculated from created_at
            'kyc_status': True,
            'reputation_score': 1.0,
            'total_activities': 150,
            'last_login': datetime.utcnow().isoformat(),
            'suspicious_flags': []
        }
        
    except Exception as e:
        logger.error(f"Failed to fetch user info: {e}")
        return None

async def _fetch_activity_history(
    session: AsyncSession, 
    user_id: str, 
    days: int = 30
) -> List[Dict[str, Any]]:
    """Fetch user activity history"""
    try:
        # Simulated activity data - would be actual database query
        return [
            {
                'activity_id': f"act_{i}",
                'activity_type': 'post',
                'platform': 'instagram',
                'timestamp': (datetime.utcnow() - timedelta(days=i)).isoformat(),
                'content': {'text': f'Sample content {i}'},
                'engagement': {'likes': i * 5, 'comments': i * 2}
            }
            for i in range(min(days, 100))
        ]
        
    except Exception as e:
        logger.error(f"Failed to fetch activity history: {e}")
        return []

async def _fetch_network_data(session: AsyncSession, user_id: str) -> List[Dict[str, Any]]:
    """Fetch user network connection data"""
    try:
        # Simulated network data
        return [
            {
                'connection_id': f"conn_{i}",
                'connected_user_id': f"user_{i}",
                'connection_type': 'referral',
                'connection_date': (datetime.utcnow() - timedelta(days=i*7)).isoformat(),
                'is_active': i < 10,
                'is_mutual': i % 3 == 0,
                'is_verified': i % 4 == 0,
                'source_platform': ['instagram', 'tiktok', 'youtube'][i % 3]
            }
            for i in range(20)
        ]
        
    except Exception as e:
        logger.error(f"Failed to fetch network data: {e}")
        return []

async def _fetch_mining_history(
    session: AsyncSession, 
    user_id: str, 
    days: int = 30
) -> List[Dict[str, Any]]:
    """Fetch user mining history"""
    try:
        # Simulated mining data
        return [
            {
                'mining_session_id': f"mining_{i}",
                'start_time': (datetime.utcnow() - timedelta(days=i)).isoformat(),
                'duration_minutes': 60 + (i % 30),
                'fin_earned': 0.1 + (i * 0.01),
                'xp_gained': 50 + (i * 5),
                'rp_gained': 10 + (i * 2),
                'quality_score': 0.8 + (i % 20) * 0.01
            }
            for i in range(min(days, 50))
        ]
        
    except Exception as e:
        logger.error(f"Failed to fetch mining history: {e}")
        return []

# Security and utility functions
class SecurityHelpers:
    """Security utility functions"""
    
    @staticmethod
    def hash_user_data(data: str) -> str:
        """Hash sensitive user data"""
        return hashlib.sha256(data.encode()).hexdigest()
    
    @staticmethod
    def encrypt_sensitive_data(data: str, key: str) -> str:
        """Encrypt sensitive data (simplified implementation)"""
        # In production, use proper encryption like Fernet
        return hashlib.sha256((data + key).encode()).hexdigest()
    
    @staticmethod
    def validate_ip_address(ip: str) -> bool:
        """Validate IP address format"""
        try:
            parts = ip.split('.')
            return len(parts) == 4 and all(0 <= int(part) <= 255 for part in parts)
        except (ValueError, AttributeError):
            return False

class CacheManager:
    """Cache management utilities"""
    
    def __init__(self):
        self.cache_ttl = {
            'user_analysis': 24 * 3600,  # 24 hours
            'batch_results': 6 * 3600,   # 6 hours
            'metrics': 1 * 3600,         # 1 hour
            'reputation': 30 * 24 * 3600 # 30 days
        }
    
    async def get_cached_analysis(self, redis: Redis, user_id: str) -> Optional[Dict]:
        """Get cached analysis result"""
        try:
            cached = await redis.get(f"bot_analysis:{user_id}")
            return json.loads(cached) if cached else None
        except Exception:
            return None
    
    async def cache_analysis(
        self, 
        redis: Redis, 
        user_id: str, 
        result: Dict,
        ttl: int = None
    ):
        """Cache analysis result"""
        try:
            ttl = ttl or self.cache_ttl['user_analysis']
            await redis.setex(
                f"bot_analysis:{user_id}",
                ttl,
                json.dumps(result)
            )
        except Exception as e:
            logger.error(f"Failed to cache analysis: {e}")

# Configuration management
class Config:
    """Configuration management for bot detection service"""
    
    def __init__(self):
        # Database configuration
        self.DATABASE_URL = os.getenv(
            'DATABASE_URL',
            'postgresql+asyncpg://user:pass@localhost/finova'
        )
        
        # Redis configuration
        self.REDIS_HOST = os.getenv('REDIS_HOST', 'localhost')
        self.REDIS_PORT = int(os.getenv('REDIS_PORT', 6379))
        self.REDIS_PASSWORD = os.getenv('REDIS_PASSWORD', '')
        
        # JWT configuration
        self.JWT_SECRET = os.getenv('JWT_SECRET', 'finova-super-secret-key')
        self.JWT_ALGORITHM = os.getenv('JWT_ALGORITHM', 'HS256')
        
        # ML Models path
        self.ML_MODELS_PATH = os.getenv('ML_MODELS_PATH', '/app/models')
        
        # Service configuration
        self.LOG_LEVEL = os.getenv('LOG_LEVEL', 'INFO')
        self.MAX_BATCH_SIZE = int(os.getenv('MAX_BATCH_SIZE', 100))
        self.ANALYSIS_TIMEOUT = int(os.getenv('ANALYSIS_TIMEOUT', 30))
        
        # Thresholds
        self.HUMAN_PROBABILITY_THRESHOLD = float(os.getenv('HUMAN_THRESHOLD', 0.6))
        self.HIGH_RISK_THRESHOLD = float(os.getenv('HIGH_RISK_THRESHOLD', 0.8))
        self.BLOCK_THRESHOLD = float(os.getenv('BLOCK_THRESHOLD', 0.9))

# Entry point
if __name__ == "__main__":
    try:
        # Set up logging
        logging.basicConfig(
            level=getattr(logging, Config().LOG_LEVEL),
            format='%(asctime)s - %(name)s - %(levelname)s - [%(filename)s:%(lineno)d] - %(message)s'
        )
        
        # Run the service
        asyncio.run(main())
        
    except KeyboardInterrupt:
        logger.info("Service interrupted by user")
    except Exception as e:
        logger.error(f"Service failed to start: {e}")
        sys.exit(1)f"Failed to load ML models: {e}")
            # Initialize with default models
            self.isolation_forest = IsolationForest(contamination=0.1, random_state=42)
            self.scaler = StandardScaler()
    
    async def initialize_connections(self):
        """Initialize database and Redis connections"""
        try:
            # Redis connection
            self.redis = Redis(
                host=self.config.REDIS_HOST,
                port=self.config.REDIS_PORT,
                password=self.config.REDIS_PASSWORD,
                decode_responses=True
            )
            await self.redis.ping()
            logger.info("Redis connection established")
            
            # Database connection
            self.db_engine = create_async_engine(
                self.config.DATABASE_URL,
                echo=False,
                pool_size=20,
                max_overflow=30
            )
            self.db_session = sessionmaker(
                self.db_engine, 
                class_=AsyncSession, 
                expire_on_commit=False
            )
            logger.info("Database connection established")
            
        except Exception as e:
            logger.error(f"Failed to initialize connections: {e}")
            raise

    async def analyze_user_activity(
        self, 
        request: UserActivityRequest,
        background_tasks: BackgroundTasks
    ) -> UserAnalysisResult:
        """
        Comprehensive user activity analysis for bot detection
        Implements the multi-layer protection system from whitepaper
        """
        start_time = time.time()
        
        try:
            # Extract user behavior data
            user_data = await self._gather_user_data(request.user_id)
            
            # Extract features from current activity
            features = await self._extract_comprehensive_features(request, user_data)
            
            # Run multi-model analysis
            analysis_results = await self._run_multi_model_analysis(features)
            
            # Calculate human probability using ensemble method
            human_prob = await self._calculate_human_probability(
                request, user_data, analysis_results
            )
            
            # Generate detailed analysis result
            result = await self._generate_analysis_result(
                request, human_prob, analysis_results, user_data
            )
            
            # Cache results and schedule updates
            background_tasks.add_task(
                self._cache_analysis_result, 
                result, 
                request.user_id
            )
            
            # Update user reputation score
            background_tasks.add_task(
                self._update_user_reputation,
                request.user_id,
                result
            )
            
            # Log performance metrics
            analysis_time = time.time() - start_time
            self.analysis_count += 1
            logger.info(
                f"Analysis completed for user {request.user_id} in {analysis_time:.3f}s"
            )
            
            return result
            
        except Exception as e:
            logger.error(f"Analysis failed for user {request.user_id}: {e}")
            # Return safe default result
            return await self._generate_safe_default_result(request.user_id)

    async def _gather_user_data(self, user_id: str) -> Dict[str, Any]:
        """Gather comprehensive user data from multiple sources"""
        user_data = {
            'activity_history': [],
            'network_connections': [],
            'device_history': [],
            'mining_history': [],
            'xp_history': [],
            'rp_history': [],
            'reputation_score': 1.0,
            'account_age_days': 0,
            'kyc_status': False,
            'suspicious_flags': []
        }
        
        try:
            # Check cache first
            cached_data = await self.redis.get(f"user_data:{user_id}")
            if cached_data:
                base_data = json.loads(cached_data)
                user_data.update(base_data)
            
            # Fetch recent activity (last 30 days)
            async with self.db_session() as session:
                # Get user basic info
                user_info = await self._fetch_user_info(session, user_id)
                if user_info:
                    user_data.update(user_info)
                
                # Get activity patterns
                user_data['activity_history'] = await self._fetch_activity_history(
                    session, user_id, days=30
                )
                
                # Get network data
                user_data['network_connections'] = await self._fetch_network_data(
                    session, user_id
                )
                
                # Get mining patterns
                user_data['mining_history'] = await self._fetch_mining_history(
                    session, user_id, days=30
                )
                
        except Exception as e:
            logger.error(f"Error gathering user data for {user_id}: {e}")
            
        return user_data

    async def _extract_comprehensive_features(
        self, 
        request: UserActivityRequest, 
        user_data: Dict[str, Any]
    ) -> Dict[str, float]:
        """Extract features for ML analysis"""
        features = {}
        
        # Temporal features
        temporal_features = self.temporal_extractor.extract_features(
            request, user_data['activity_history']
        )
        features.update(temporal_features)
        
        # Behavioral features
        behavioral_features = self.behavioral_extractor.extract_features(
            request, user_data['activity_history']
        )
        features.update(behavioral_features)
        
        # Network features
        network_features = self.network_extractor.extract_features(
            user_data['network_connections']
        )
        features.update(network_features)
        
        # Device features
        device_features = self.device_extractor.extract_features(
            request.device_info, user_data['device_history']
        )
        features.update(device_features)
        
        return features

    async def _run_multi_model_analysis(self, features: Dict[str, float]) -> Dict[str, float]:
        """Run ensemble of ML models for bot detection"""
        results = {}
        
        try:
            # Prepare feature vector
            feature_vector = np.array(list(features.values())).reshape(1, -1)
            scaled_features = self.scaler.transform(feature_vector)
            
            # Isolation Forest for anomaly detection
            anomaly_score = self.isolation_forest.decision_function(scaled_features)[0]
            results['anomaly_score'] = float(anomaly_score)
            
            # Behavior analysis
            behavior_score = await self.behavior_analyzer.analyze(features)
            results['behavior_score'] = behavior_score
            
            # Pattern detection
            pattern_score = await self.pattern_detector.detect_patterns(features)
            results['pattern_score'] = pattern_score
            
            # Network analysis
            network_score = await self.network_analyzer.analyze_network(features)
            results['network_score'] = network_score
            
        except Exception as e:
            logger.error(f"ML analysis failed: {e}")
            # Return neutral scores on failure
            results = {
                'anomaly_score': 0.0,
                'behavior_score': 0.5,
                'pattern_score': 0.5,
                'network_score': 0.5
            }
            
        return results

    async def _calculate_human_probability(
        self,
        request: UserActivityRequest,
        user_data: Dict[str, Any],
        analysis_results: Dict[str, float]
    ) -> float:
        """
        Calculate human probability using the whitepaper formula
        Implements Proof-of-Humanity (PoH) integration
        """
        factors = {
            'biometric_consistency': await self._analyze_biometric_patterns(user_data),
            'behavioral_patterns': await self._detect_human_rhythms(user_data),
            'social_graph_validity': await self._validate_real_connections(user_data),
            'device_authenticity': await self._check_device_fingerprint(request.device_info),
            'interaction_quality': await self._measure_content_uniqueness(request, user_data)
        }
        
        # Weights based on Finova's PoH algorithm
        weights = {
            'biometric_consistency': 0.25,
            'behavioral_patterns': 0.20,
            'social_graph_validity': 0.20,
            'device_authenticity': 0.15,
            'interaction_quality': 0.20
        }
        
        weighted_score = sum(factors[key] * weights[key] for key in factors)
        
        # Apply ML model corrections
        ml_correction = (
            analysis_results['behavior_score'] * 0.3 +
            analysis_results['pattern_score'] * 0.3 +
            analysis_results['network_score'] * 0.4
        )
        
        # Combine scores with safety bounds
        final_score = (weighted_score * 0.7) + (ml_correction * 0.3)
        return max(0.1, min(1.0, final_score))

    async def _calculate_mining_penalties(
        self, 
        user_id: str, 
        human_probability: float,
        analysis_results: Dict[str, float]
    ) -> MiningPenalty:
        """
        Calculate mining penalties based on whitepaper formulas
        """
        # Base penalty calculation
        base_penalty = max(0.0, 1.0 - human_probability)
        
        # Behavior-based penalty
        behavior_penalty = max(0.0, 1.0 - analysis_results['behavior_score'])
        
        # Network-based penalty
        network_penalty = max(0.0, 1.0 - analysis_results['network_score'])
        
        # Quality-based penalty
        quality_penalty = max(0.0, 1.0 - analysis_results.get('quality_score', 0.5))
        
        # Total penalty with exponential scaling
        total_penalty = min(0.95, (
            base_penalty * 0.4 +
            behavior_penalty * 0.25 +
            network_penalty * 0.20 +
            quality_penalty * 0.15
        ))
        
        # Penalty duration (hours)
        if total_penalty > 0.8:
            duration = 168  # 7 days for severe violations
        elif total_penalty > 0.6:
            duration = 72   # 3 days for moderate violations
        elif total_penalty > 0.3:
            duration = 24   # 1 day for minor violations
        else:
            duration = 0    # No penalty
        
        justification = self._generate_penalty_justification(
            human_probability, analysis_results
        )
        
        return MiningPenalty(
            user_id=user_id,
            base_penalty=base_penalty,
            behavior_penalty=behavior_penalty,
            network_penalty=network_penalty,
            quality_penalty=quality_penalty,
            total_penalty=total_penalty,
            penalty_duration=duration,
            justification=justification
        )

    async def _generate_analysis_result(
        self,
        request: UserActivityRequest,
        human_probability: float,
        analysis_results: Dict[str, float],
        user_data: Dict[str, Any]
    ) -> UserAnalysisResult:
        """Generate comprehensive analysis result"""
        
        # Calculate risk score
        risk_score = 1.0 - human_probability
        
        # Determine recommendation
        if human_probability >= 0.8:
            recommendation = "ALLOW"
        elif human_probability >= 0.6:
            recommendation = "MONITOR"
        elif human_probability >= 0.4:
            recommendation = "RESTRICT"
        else:
            recommendation = "BLOCK"
        
        # Calculate confidence level
        score_variance = np.var(list(analysis_results.values()))
        confidence_level = max(0.5, 1.0 - (score_variance * 2))
        
        # Generate detailed flags
        detailed_flags = await self._generate_detailed_flags(
            request, analysis_results, user_data
        )
        
        # Generate mitigation actions
        mitigation_actions = await self._generate_mitigation_actions(
            recommendation, detailed_flags
        )
        
        return UserAnalysisResult(
            user_id=request.user_id,
            human_probability=human_probability,
            risk_score=risk_score,
            behavior_score=analysis_results.get('behavior_score', 0.5),
            network_score=analysis_results.get('network_score', 0.5),
            device_score=analysis_results.get('device_score', 0.5),
            temporal_score=analysis_results.get('temporal_score', 0.5),
            quality_score=analysis_results.get('quality_score', 0.5),
            final_recommendation=recommendation,
            confidence_level=confidence_level,
            analysis_timestamp=datetime.utcnow(),
            detailed_flags=detailed_flags,
            mitigation_actions=mitigation_actions
        )

    async def _analyze_biometric_patterns(self, user_data: Dict[str, Any]) -> float:
        """Analyze biometric consistency patterns"""
        try:
            # Simulate biometric analysis (would integrate with actual biometric data)
            kyc_score = 1.0 if user_data.get('kyc_status') else 0.3
            
            # Check for consistent selfie patterns in KYC data
            selfie_consistency = user_data.get('selfie_consistency_score', 0.8)
            
            # Analyze device biometric usage patterns
            biometric_usage = user_data.get('biometric_usage_frequency', 0.5)
            
            return (kyc_score * 0.5) + (selfie_consistency * 0.3) + (biometric_usage * 0.2)
            
        except Exception as e:
            logger.error(f"Biometric analysis failed: {e}")
            return 0.5

    async def _detect_human_rhythms(self, user_data: Dict[str, Any]) -> float:
        """Detect natural human behavioral rhythms"""
        try:
            activity_history = user_data.get('activity_history', [])
            if not activity_history:
                return 0.5
            
            # Convert to DataFrame for analysis
            df = pd.DataFrame(activity_history)
            df['timestamp'] = pd.to_datetime(df['timestamp'])
            df['hour'] = df['timestamp'].dt.hour
            df['day_of_week'] = df['timestamp'].dt.dayofweek
            
            # Analyze circadian rhythm patterns
            hourly_activity = df.groupby('hour').size()
            daily_activity = df.groupby('day_of_week').size()
            
            # Check for natural variance (humans have irregular patterns)
            hourly_variance = hourly_activity.var() / hourly_activity.mean()
            daily_variance = daily_activity.var() / daily_activity.mean()
            
            # Natural sleep patterns (low activity 1-6 AM)
            night_activity = hourly_activity[1:7].sum() / hourly_activity.sum()
            sleep_pattern_score = max(0.0, 1.0 - (night_activity * 4))
            
            # Activity gaps (humans take breaks)
            activity_gaps = self._analyze_activity_gaps(df)
            gap_score = min(1.0, activity_gaps / 10)  # Normalize gaps
            
            # Combine rhythm scores
            rhythm_score = (
                min(1.0, hourly_variance / 2) * 0.3 +
                min(1.0, daily_variance / 2) * 0.2 +
                sleep_pattern_score * 0.3 +
                gap_score * 0.2
            )
            
            return max(0.1, min(1.0, rhythm_score))
            
        except Exception as e:
            logger.error(f"Human rhythm detection failed: {e}")
            return 0.5

    async def _validate_real_connections(self, user_data: Dict[str, Any]) -> float:
        """Validate authenticity of social connections"""
        try:
            connections = user_data.get('network_connections', [])
            if not connections:
                return 0.3  # New users get benefit of doubt
            
            # Analyze connection patterns
            total_connections = len(connections)
            active_connections = len([c for c in connections if c.get('is_active')])
            
            # Connection quality metrics
            mutual_connections = len([c for c in connections if c.get('is_mutual')])
            verified_connections = len([c for c in connections if c.get('is_verified')])
            
            # Calculate scores
            activity_ratio = active_connections / max(1, total_connections)
            mutual_ratio = mutual_connections / max(1, total_connections)
            verified_ratio = verified_connections / max(1, total_connections)
            
            # Connection growth pattern (natural vs suspicious)
            connection_timeline = [c.get('connection_date') for c in connections]
            growth_pattern_score = self._analyze_connection_growth_pattern(connection_timeline)
            
            # Network diversity (not all from same source)
            source_diversity = len(set(c.get('source_platform') for c in connections))
            diversity_score = min(1.0, source_diversity / 5)  # Normalize to max 5 platforms
            
            # Final validation score
            validation_score = (
                activity_ratio * 0.25 +
                mutual_ratio * 0.20 +
                verified_ratio * 0.20 +
                growth_pattern_score * 0.20 +
                diversity_score * 0.15
            )
            
            return max(0.1, min(1.0, validation_score))
            
        except Exception as e:
            logger.error(f"Connection validation failed: {e}")
            return 0.5

    async def _check_device_fingerprint(self, device_info: Dict[str, Any]) -> float:
        """Check device authenticity and consistency"""
        try:
            # Generate device fingerprint
            fingerprint_data = {
                'user_agent': device_info.get('user_agent', ''),
                'screen_resolution': device_info.get('screen_resolution', ''),
                'timezone': device_info.get('timezone', ''),
                'language': device_info.get('language', ''),
                'platform': device_info.get('platform', ''),
                'hardware_specs': device_info.get('hardware_specs', {})
            }
            
            # Create consistent fingerprint hash
            fingerprint_string = json.dumps(fingerprint_data, sort_keys=True)
            fingerprint_hash = hashlib.sha256(fingerprint_string.encode()).hexdigest()
            
            # Check for device consistency over time
            device_history = await self.redis.lrange(
                f"device_history:{fingerprint_hash}", 0, -1
            )
            
            # Analyze device authenticity factors
            authenticity_factors = {
                'hardware_consistency': self._check_hardware_consistency(device_info),
                'browser_authenticity': self._check_browser_authenticity(device_info),
                'os_authenticity': self._check_os_authenticity(device_info),
                'sensor_data': self._check_sensor_authenticity(device_info),
                'network_authenticity': self._check_network_authenticity(device_info)
            }
            
            # Calculate weighted authenticity score
            weights = {
                'hardware_consistency': 0.25,
                'browser_authenticity': 0.20,
                'os_authenticity': 0.20,
                'sensor_data': 0.20,
                'network_authenticity': 0.15
            }
            
            authenticity_score = sum(
                authenticity_factors[key] * weights[key] 
                for key in authenticity_factors
            )
            
            # Store device fingerprint for future analysis
            await self.redis.lpush(
                f"device_history:{fingerprint_hash}",
                json.dumps({
                    'timestamp': datetime.utcnow().isoformat(),
                    'authenticity_score': authenticity_score
                })
            )
            await self.redis.ltrim(f"device_history:{fingerprint_hash}", 0, 99)
            
            return max(0.1, min(1.0, authenticity_score))
            
        except Exception as e:
            logger.error(f"Device fingerprint check failed: {e}")
            return 0.5

    async def _measure_content_uniqueness(
        self, 
        request: UserActivityRequest, 
        user_data: Dict[str, Any]
    ) -> float:
        """Measure content uniqueness and quality"""
        try:
            content_data = request.content_data or {}
            content_text = content_data.get('text', '')
            
            if not content_text:
                return 0.7  # Neutral score for non-text content
            
            # Check against previous user content
            user_content_history = [
                activity.get('content', {}).get('text', '')
                for activity in user_data.get('activity_history', [])
                if activity.get('content', {}).get('text')
            ]
            
            # Calculate similarity scores
            similarity_scores = []
            for historical_content in user_content_history[-50:]:  # Last 50 pieces
                similarity = self._calculate_text_similarity(content_text, historical_content)
                similarity_scores.append(similarity)
            
            # Uniqueness metrics
            max_similarity = max(similarity_scores) if similarity_scores else 0.0
            avg_similarity = np.mean(similarity_scores) if similarity_scores else 0.0
            
            # Content quality metrics
            quality_metrics = {
                'length_appropriateness': self._check_content_length(content_text),
                'language_quality': self._check_language_quality(content_text),
                'spam_indicators': self._check_spam_indicators(content_text),
                'originality': max(0.0, 1.0 - max_similarity),
                'diversity': max(0.0, 1.0 - avg_similarity)
            }
            
            # Calculate weighted quality score
            quality_weights = {
                'length_appropriateness': 0.15,
                'language_quality': 0.20,
                'spam_indicators': 0.25,
                'originality': 0.25,
                'diversity': 0.15
            }
            
            quality_score = sum(
                quality_metrics[key] * quality_weights[key]
                for key in quality_metrics
            )
            
            return max(0.1, min(1.0, quality_score))
            
        except Exception as e:
            logger.error(f"Content uniqueness measurement failed: {e}")
            return 0.5

    def _calculate_text_similarity(self, text1: str, text2: str) -> float:
        """Calculate similarity between two text strings"""
        try:
            # Simple Jaccard similarity for demonstration
            # In production, would use more sophisticated NLP models
            words1 = set(text1.lower().split())
            words2 = set(text2.lower().split())
            
            if not words1 and not words2:
                return 1.0
            if not words1 or not words2:
                return 0.0
                
            intersection = len(words1.intersection(words2))
            union = len(words1.union(words2))
            
            return intersection / union if union > 0 else 0.0
            
        except Exception:
            return 0.0

    def _check_hardware_consistency(self, device_info: Dict[str, Any]) -> float:
        """Check hardware specification consistency"""
        try:
            hardware = device_info.get('hardware_specs', {})
            
            # Check for reasonable hardware combinations
            cpu_cores = hardware.get('cpu_cores', 4)
            memory_gb = hardware.get('memory_gb', 8)
            gpu_memory = hardware.get('gpu_memory_mb', 2048)
            
            # Reasonable hardware ratios
            memory_cpu_ratio = memory_gb / max(1, cpu_cores)
            
            # Consistency checks
            consistency_score = 1.0
            
            # Unrealistic combinations
            if memory_cpu_ratio > 8 or memory_cpu_ratio < 1:
                consistency_score -= 0.3
            
            if cpu_cores > 32 or cpu_cores < 1:
                consistency_score -= 0.2
                
            if gpu_memory > 24576 or gpu_memory < 512:  # 24GB max, 512MB min
                consistency_score -= 0.2
            
            return max(0.1, consistency_score)
            
        except Exception:
            return 0.5

    def _check_browser_authenticity(self, device_info: Dict[str, Any]) -> float:
        """Check browser authenticity indicators"""
        try:
            user_agent = device_info.get('user_agent', '')
            
            # Check for common bot indicators
            bot_indicators = [
                'bot', 'crawler', 'spider', 'scraper', 'automated',
                'headless', 'phantom', 'selenium', 'puppeteer'
            ]
            
            bot_score = sum(1 for indicator in bot_indicators if indicator in user_agent.lower())
            
            # Check for realistic browser versions
            version_authenticity = self._check_browser_version_authenticity(user_agent)
            
            # Check for standard browser features
            has_javascript = device_info.get('javascript_enabled', True)
            has_cookies = device_info.get('cookies_enabled', True)
            has_local_storage = device_info.get('local_storage_enabled', True)
            
            feature_score = (
                (1.0 if has_javascript else 0.0) +
                (1.0 if has_cookies else 0.0) +
                (1.0 if has_local_storage else 0.0)
            ) / 3.0
            
            # Final browser authenticity score
            authenticity_score = (
                max(0.0, 1.0 - (bot_score * 0.3)) * 0.4 +
                version_authenticity * 0.3 +
                feature_score * 0.3
            )
            
            return max(0.1, min(1.0, authenticity_score))
            
        except Exception:
            return 0.5

    def _check_browser_version_authenticity(self, user_agent: str) -> float:
        """Check if browser version is realistic and current"""
        try:
            # Simplified browser version checking
            current_year = datetime.now().year
            
            # Check for outdated browsers (potential bot indicator)
            if 'Chrome/9' in user_agent or 'Firefox/3' in user_agent:
                return 0.2  # Very old browsers
            elif 'Chrome/10' in user_agent or 'Firefox/4' in user_agent:
                return 0.4  # Old browsers
            else:
                return 0.9  # Modern browsers
                
        except Exception:
            return 0.5

    def _check_os_authenticity(self, device_info: Dict[str, Any]) -> float:
        """Check operating system authenticity"""
        try:
            os_info = device_info.get('os_info', {})
            os_name = os_info.get('name', '').lower()
            os_version = os_info.get('version', '')
            
            # Check for realistic OS combinations
            authenticity_score = 1.0
            
            # Check for suspicious OS indicators
            if 'bot' in os_name or 'headless' in os_name:
                authenticity_score -= 0.5
            
            # Check version reasonableness
            if not os_version or len(os_version) < 3:
                authenticity_score -= 0.2
            
            # Platform consistency
            user_agent = device_info.get('user_agent', '')
            if 'Windows' in user_agent and 'mac' in os_name:
                authenticity_score -= 0.4
            
            return max(0.1, authenticity_score)
            
        except Exception:
            return 0.5

    def _check_sensor_authenticity(self, device_info: Dict[str, Any]) -> float:
        """Check device sensor data for authenticity"""
        try:
            sensors = device_info.get('sensors', {})
            
            # Check for presence of common mobile sensors
            has_accelerometer = sensors.get('accelerometer', False)
            has_gyroscope = sensors.get('gyroscope', False)
            has_magnetometer = sensors.get('magnetometer', False)
            has_gps = sensors.get('gps', False)
            
            # Mobile devices should have these sensors
            is_mobile = 'mobile' in device_info.get('platform', '').lower()
            
            if is_mobile:
                sensor_score = (
                    (1.0 if has_accelerometer else 0.0) +
                    (1.0 if has_gyroscope else 0.0) +
                    (0.5 if has_magnetometer else 0.0) +
                    (0.5 if has_gps else 0.0)
                ) / 3.0
            else:
                # Desktop devices - different expectations
                sensor_score = 0.8  # Neutral score for desktop
            
            return max(0.1, min(1.0, sensor_score))
            
        except Exception:
            return 0.5

    def _check_network_authenticity(self, device_info: Dict[str, Any]) -> float:
        """Check network connection authenticity"""
        try:
            network_info = device_info.get('network', {})
            
            # Check connection type
            connection_type = network_info.get('type', 'unknown')
            ip_address = network_info.get('ip_address', '')
            
            # Check for VPN/proxy indicators
            is_vpn = network_info.get('is_vpn', False)
            is_proxy = network_info.get('is_proxy', False)
            is_tor = network_info.get('is_tor', False)
            
            # Geographic consistency
            declared_country = network_info.get('country', '')
            timezone_country = device_info.get('timezone_country', '')
            
            # Calculate network authenticity
            authenticity_score = 1.0
            
            if is_vpn:
                authenticity_score -= 0.3
            if is_proxy:
                authenticity_score -= 0.4
            if is_tor:
                authenticity_score -= 0.5
            
            # Geographic inconsistency
            if declared_country != timezone_country and both_present:
                authenticity_score -= 0.2
            
            return max(0.1, authenticity_score)
            
        except Exception:
            return 0.5

    def _analyze_activity_gaps(self, activity_df: pd.DataFrame) -> int:
        """Analyze natural activity gaps in user behavior"""
        try:
            # Sort by timestamp
            df_sorted = activity_df.sort_values('timestamp')
            
            # Calculate time differences between activities
            time_diffs = df_sorted['timestamp'].diff()
            
            # Count significant gaps (>2 hours)
            significant_gaps = (time_diffs > timedelta(hours=2)).sum()
            
            return int(significant_gaps)
            
        except Exception:
            return 5  # Default reasonable gap count

    def _analyze_connection_growth_pattern(self, connection_dates: List[str]) -> float:
        """Analyze if connection growth pattern is natural"""
        try:
            if not connection_dates:
                return 0.5
            
            # Convert to datetime objects
            dates = [datetime.fromisoformat(date) for date in connection_dates if date]
            dates.sort()
            
            if len(dates) < 2:
                return 0.8  # Single connection is natural
            
            # Calculate time intervals between connections
            intervals = [(dates[i] - dates[i-1]).total_seconds() / 3600 
                        for i in range(1, len(dates))]  # Convert to hours
            
            # Natural growth should have variance
            if len(intervals) > 1:
                interval_variance = np.var(intervals)
                mean_interval = np.mean(intervals)
                
                # Coefficient of variation
                cv = interval_variance / max(1, mean_interval)
                
                # Natural patterns have moderate variance
                if cv < 0.1:  # Too uniform (bot-like)
                    return 0.3
                elif cv > 5.0:  # Too random
                    return 0.6
                else:
                    return 0.9  # Natural variance
            
            return 0.7
            
        except Exception:
            return 0.5

    def _check_content_length(self, content: str) -> float:
        """Check if content length is appropriate"""
        length = len(content.strip())
        
        if length == 0:
            return 0.3
        elif length < 5:  # Too short
            return 0.4
        elif length > 5000:  # Suspiciously long
            return 0.6
        elif 20 <= length <= 500:  # Optimal range
            return 1.0
        else:
            return 0.8

    def _check_language_quality(self, content: str) -> float:
        """Check language quality and naturalness"""
        try:
            # Simple quality checks
            words = content.split()
            
            if not words:
                return 0.3
            
            # Check for reasonable word variety
            unique_words = len(set(words))
            word_variety = unique_words / len(words)
            
            # Check for excessive repetition
            repetition_score = 1.0 - max(0, (len(words) - unique_words * 2) / len(words))
            
            # Check for proper capitalization
            capitalization_score = self._check_capitalization(content)
            
            # Combine scores
            quality_score = (
                min(1.0, word_variety * 2) * 0.4 +
                repetition_score * 0.3 +
                capitalization_score * 0.3
            )
            
            return max(0.1, quality_score)
            
        except Exception:
            return 0.5

    def _check_spam_indicators(self, content: str) -> float:
        """Check for spam indicators in content"""
        try:
            content_lower = content.lower()
            
            # Common spam indicators
            spam_words = [
                'click here', 'buy now', 'free money', 'guaranteed',
                'make money fast', 'no risk', 'limited time',
                'act now', 'urgent', 'congratulations you won'
            ]
            
            # URL spam patterns
            url_count = content.count('http')
            excessive_urls = url_count > 3
            
            # Excessive punctuation
            exclamation_count = content.count('!')
            excessive_punctuation = exclamation_count > 5
            
            # Calculate spam score
            spam_indicators = (
                sum(1 for word in spam_words if word in content_lower) +
                (1 if excessive_urls else 0) +
                (1 if excessive_punctuation else 0)
            )
            
            # Convert to quality score (inverted)
            spam_score = max(0.0, 1.0 - (spam_indicators * 0.2))
            
            return spam_score
            
        except Exception:
            return 0.5

    def _check_capitalization(self, content: str) -> float:
        """Check for natural capitalization patterns"""
        try:
            if not content:
                return 0.5
            
            # Check sentence beginnings
            sentences = content.split('.')
            proper_caps = sum(1 for s in sentences if s.strip() and s.strip()[0].isupper())
            
            # Check for all caps (spam indicator)
            all_caps_ratio = sum(1 for c in content if c.isupper()) / max(1, len(content))
            
            if all_caps_ratio > 0.7:  # Mostly caps
                return 0.2
            elif all_caps_ratio < 0.05:  # No caps
                return 0.3
            else:
                return 0.9
                
        except Exception:
            return 0.5

    async def _generate_detailed_flags(
        self,
        request: UserActivityRequest,
        analysis_results: Dict[str, float],
        user_data: Dict[str, Any]
    ) -> Dict[str, Any]:
        """Generate detailed analysis flags"""
        flags = {
            'behavioral_anomalies': [],
            'network_red_flags': [],
            'device_inconsistencies': [],
            'content_quality_issues': [],
            'temporal_irregularities': []
        }
        
        # Behavioral flags
        if analysis_results.get('behavior_score', 0.5) < 0.4:
            flags['behavioral_anomalies'].extend([
                'Unusual activity patterns detected',
                'Inconsistent interaction timing',
                'Mechanical behavior indicators'
            ])
        
        # Network flags
        if analysis_results.get('network_score', 0.5) < 0.4:
            flags['network_red_flags'].extend([
                'Suspicious referral patterns',
                'Potential bot network connections',
                'Unnatural network growth'
            ])
        
        # Device flags
        device_score = analysis_results.get('device_score', 0.5)
        if device_score < 0.4:
            flags['device_inconsistencies'].extend([
                'Inconsistent device fingerprinting',
                'Potential emulation detected',
                'Hardware specification anomalies'
            ])
        
        # Content quality flags
        quality_score = analysis_results.get('quality_score', 0.5)
        if quality_score < 0.4:
            flags['content_quality_issues'].extend([
                'Low content originality',
                'Repetitive content patterns',
                'Potential automated content generation'
            ])
        
        # Temporal flags
        temporal_score = analysis_results.get('temporal_score', 0.5)
        if temporal_score < 0.4:
            flags['temporal_irregularities'].extend([
                'Unnatural timing patterns',
                'Missing circadian rhythm indicators',
                'Mechanical session patterns'
            ])
        
        return flags

    async def _generate_mitigation_actions(
        self,
        recommendation: str,
        detailed_flags: Dict[str, Any]
    ) -> List[str]:
        """Generate specific mitigation actions based on analysis"""
        actions = []
        
        if recommendation == "BLOCK":
            actions.extend([
                "Suspend mining activities",
                "Freeze XP and RP accumulation",
                "Require enhanced verification",
                "Flag for manual review"
            ])
        elif recommendation == "RESTRICT":
            actions.extend([
                "Apply 70% mining penalty",
                "Reduce XP gain by 50%",
                "Limit RP network effects",
                "Increase monitoring frequency"
            ])
        elif recommendation == "MONITOR":
            actions.extend([
                "Apply 30% mining penalty",
                "Enhanced activity logging",
                "Weekly behavior analysis",
                "Require additional verification steps"
            ])
        
        # Add specific actions based on flags
        if detailed_flags['behavioral_anomalies']:
            actions.append("Implement behavioral challenge tests")
        
        if detailed_flags['network_red_flags']:
            actions.append("Audit referral network connections")
        
        if detailed_flags['device_inconsistencies']:
            actions.append("Require device re-verification")
        
        if detailed_flags['content_quality_issues']:
            actions.append("Implement content quality filtering")
        
        return list(set(actions))  # Remove duplicates

    def _generate_penalty_justification(
        self,
        human_probability: float,
        analysis_results: Dict[str, float]
    ) -> str:
        """Generate human-readable penalty justification"""
        if human_probability >= 0.8:
            return "No penalty - user demonstrates authentic human behavior"
        
        issues = []
        
        if human_probability < 0.4:
            issues.append("Low human probability score")
        
        if analysis_results.get('behavior_score', 0.5) < 0.4:
            issues.append("Suspicious behavioral patterns")
        
        if analysis_results.get('network_score', 0.5) < 0.4:
            issues.append("Questionable network connections")
        
        if analysis_results.get('quality_score', 0.5) < 0.4:
            issues.append("Low content quality indicators")
        
        if not issues:
            return "Moderate confidence penalty applied as precaution"
        
        return f"Penalty applied due to: {', '.join(issues)}"

    async def _cache_analysis_result(
        self,
        result: UserAnalysisResult,
        user_id: str
    ):
        """Cache analysis result for future reference"""
        try:
            # Store in Redis with TTL
            cache_key = f"bot_analysis:{user_id}"
            cache_data = asdict(result)
            
            # Convert datetime to string for JSON serialization
            cache_data['analysis_timestamp'] = result.analysis_timestamp.isoformat()
            
            await self.redis.setex(
                cache_key,
                timedelta(hours=24),  # Cache for 24 hours
                json.dumps(cache_data)
            )
            
            # Store in analysis history
            history_key = f"analysis_history:{user_id}"
            await self.redis.lpush(history_key, json.dumps(cache_data))
            await self.redis.ltrim(history_key, 0, 99)  # Keep last 100 analyses
            
        except Exception as e:
            logger.error(f"Failed to cache analysis result: {e}")

    async def _update_user_reputation(
        self,
        user_id: str,
        result: UserAnalysisResult
    ):
        """Update user reputation score based on analysis"""
        try:
            current_reputation = await self.redis.get(f"reputation:{user_id}")
            current_reputation = float(current_reputation) if current_reputation else 1.0
            
            # Calculate reputation adjustment
            reputation_delta = (result.human_probability - 0.5) * 0.1
            new_reputation = max(0.1, min(2.0, current_reputation + reputation_delta))
            
            # Store updated reputation
            await self.redis.setex(
                f"reputation:{user_id}",
                timedelta(days=30),
                str(new_reputation)
            )
            
            # Log significant reputation changes
            if abs(reputation_delta) > 0.05:
                logger.info(
                    f"Reputation updated for user {user_id}: "
                    f"{current_reputation:.3f} -> {new_reputation:.3f}"
                )
                
        except Exception as e:
            logger.error(f"Failed to update user reputation: {e}")

    async def _generate_safe_default_result(self, user_id: str) -> UserAnalysisResult:
        """Generate safe default result when analysis fails"""
        return UserAnalysisResult(
            user_id=user_id,
            human_probability=0.5,
            risk_score=0.5,
            behavior_score=0.5,
            network_score=0.5,
            device_score=0.5,
            temporal_score=0.5,
            quality_score=0.5,
            final_recommendation="MONITOR",
            confidence_level=0.3,
            analysis_timestamp=datetime.utcnow(),
            detailed_flags={'system_error': ['Analysis system temporarily unavailable']},
            mitigation_actions=['Retry analysis in 1 hour', 'Apply conservative restrictions']
        )

    async def get_user_bot_score(self, user_id: str) -> float:
        """Get current bot probability score for user"""
        try:
            cached_result = await self.redis.get(f"bot_analysis:{user_id}")
            if cached_result:
                data = json.loads(cached_result)
                return 1.0 - data.get('human_probability', 0.5)
            
            return 0.5  # Default moderate risk
            
        except Exception as e:
            logger.error(f"Failed to get bot score for {user_id}: {e}")
            return 0.5

    async def batch_analyze_users(self, user_ids: List[str]) -> Dict[str, float]:
        """Batch analysis for multiple users (performance optimization)"""
        results = {}
        
        try:
            # Gather batch data
            batch_data = await asyncio.gather(*[
                self._gather_user_data(user_id) for user_id in user_ids
            ])
            
            # Process in parallel
            analysis_tasks = []
            for i, user_id in enumerate(user_ids):
                if i < len(batch_data):
                    task = self._quick_analysis(user_id, batch_data[i])
                    analysis_tasks.append(task)
            
            batch_results = await asyncio.gather(*analysis_tasks)
            
            # Compile results
            for i, user_id in enumerate(user_ids):
                if i < len(batch_results):
                    results[user_id] = batch_results[i]
                else:
                    results[user_id] = 0.5  # Default score
                    
        except Exception as e:
            logger.error(f"Batch analysis failed: {e}")
            # Return default scores for all users
            results = {user_id: 0.5 for user_id in user_ids}
        
        return results

    async def _quick_analysis(self, user_id: str, user_data: Dict[str, Any]) -> float:
        """Quick analysis for batch processing"""
        try:
            # Simplified analysis for performance
            activity_count = len(user_data.get('activity_history', []))
            account_age = user_data.get('account_age_days', 0)
            kyc_status = user_data.get('kyc_status', False)
            
            # Basic heuristics
            activity_score = min(1.0, activity_count / 100)  # Normalize activity
            age_score = min(1.0, account_age / 30)  # Normalize account age
            kyc_score = 1.0 if kyc_status else 0.5
            
            # Quick human probability estimate
            quick_score = (
                activity_score * 0.4 +
                age_score * 0.3 +
                kyc_score * 0.3
            )
            
            return max(0.1, min(1.0, quick_score))
            
        except Exception:
            return 0.5

    async def health_check(self) -> Dict[str, Any]:
        """System health check endpoint"""
        try:
            # Test Redis connection
            redis_status = await self.redis.ping()
            
            # Test database connection
            async with self.db_session() as session:
                await session.execute("SELECT 1")
                db_status = True
            
            # Check ML model status
            ml_status = hasattr(self, 'isolation_forest') and hasattr(self, 'scaler')
            
            # Performance metrics
            uptime = time.time() - self.start_time if hasattr(self, 'start_time') else 0
            
            return {
                'status': 'healthy',
                'redis_connected': redis_status,
                'database_connected': db_status,
                'ml_models_loaded': ml_status,
                'uptime_seconds': uptime,
                'total_analyses': self.analysis_count,
                'bot_detections': self.bot_detection_count,
                'false_positives': self.false_positive_count,
                'timestamp': datetime.utcnow().isoformat()
            }
            
        except Exception as e:
            logger.error(f"Health check failed: {e}")
            return {
                'status': 'unhealthy',
                'error': str(e),
                'timestamp': datetime.utcnow().isoformat()
            }

    async def shutdown(self):
        """Graceful shutdown procedure"""
        try:
            logger.info("Shutting down bot detection service...")
            
            if self.redis:
                await self.redis.close()
            
            if self.db_engine:
                await self.db_engine.dispose()
            
            logger.info("Bot detection service shutdown complete")
            
        except Exception as e:
            logger.error(f"Error during shutdown: {e}")

# Global service instance
bot_detection_service = FinovaBotDetectionService()

@asynccontextmanager
async def lifespan(app: FastAPI):
    """Application lifespan management"""
    # Startup
    bot_detection_service.start_time = time.time()
    await bot_detection_service.initialize_connections()
    logger.info("Finova Bot Detection Service started successfully")
    
    yield
    
    # Shutdown
    await bot_detection_service.shutdown()

# FastAPI application
app = FastAPI(
    title="Finova Network Bot Detection Service",
    description="Enterprise-grade anti-bot protection for Finova Network",
    version="1.0.0",
    lifespan=lifespan
)

# Add middleware
app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],  # Configure properly for production
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)

app.add_middleware(
    TrustedHostMiddleware,
    allowed_hosts=["*"]  # Configure properly for production
)

# Include API routes
app.include_router(api_router, prefix="/api/v1")

@app.post("/analyze", response_model=BotDetectionResponse)
async def analyze_user_activity(
    request: UserActivityRequest,
    background_tasks: BackgroundTasks,
    credentials: HTTPAuthorizationCredentials = Depends(security)
):
    """
    Main endpoint for user activity analysis
    Implements the comprehensive bot detection from Finova whitepaper
    """
    try:
        # Verify JWT token
        token_data = jwt.decode(
            credentials.credentials,
            bot_detection_service.config.JWT_SECRET,
            algorithms=["HS256"]
        )
        
        # Run comprehensive analysis
        analysis_result = await bot_detection_service.analyze_user_activity(
            request, background_tasks
        )
        
        # Calculate penalties using whitepaper formulas
        mining_penalty = await bot_detection_service._calculate_mining_penalties(
            request.user_id,
            analysis_result.human_probability,
            {
                'behavior_score': analysis_result.behavior_score,
                'network_score': analysis_result.network_score,
                'quality_score': analysis_result.quality_score
            }
        )
        
        # Determine next analysis time
        if analysis_result.final_recommendation == "BLOCK":
            next_analysis = datetime.utcnow() + timedelta(days=7)
        elif analysis_result.final_recommendation == "RESTRICT":
            next_analysis = datetime.utcnow() + timedelta(days=1)
        else:
            next_analysis = datetime.utcnow() + timedelta(hours=6)
        
        return BotDetectionResponse(
            user_id=request.user_id,
            is_human_probable=analysis_result.human_probability >= 0.6,
            human_probability=analysis_result.human_probability,
            risk_level=analysis_result.final_recommendation,
            mining_penalty=mining_penalty.total_penalty,
            xp_penalty=mining_penalty.total_penalty * 0.8,  # Slightly less for XP
            rp_penalty=mining_penalty.total_penalty * 1.2,  # More for RP (network effects)
            recommendations=analysis_result.mitigation_actions,
            next_analysis_time=next_analysis
        )
        
    except jwt.InvalidTokenError:
        raise HTTPException(status_code=401, detail="Invalid authentication token")
    except Exception as e:
        logger.error(f"Analysis endpoint failed: {e}")
        raise HTTPException(status_code=500, detail="Internal analysis error")

@app.get("/user/{user_id}/bot-score")
async def get_user_bot_score(
    user_id: str,
    credentials: HTTPAuthorizationCredentials = Depends(security)
):
    """Get current bot probability score for specific user"""
    try:
        # Verify token
        jwt.decode(
            credentials.credentials,
            bot_detection_service.config.JWT_SECRET,
            algorithms=["HS256"]
        )
        
        bot_score = await bot_detection_service.get_user_bot_score(user_id)
        
        return {
            'user_id': user_id,
            'bot_probability': bot_score,
            'human_probability': 1.0 - bot_score,
            'timestamp': datetime.utcnow().isoformat()
        }
        
    except jwt.InvalidTokenError:
        raise HTTPException(status_code=401, detail="Invalid authentication token")
    except Exception as e:
        logger.error(f"Bot score retrieval failed for {user_id}: {e}")
        raise HTTPException(status_code=500, detail="Failed to retrieve bot score")

@app.post("/batch-analyze")
async def batch_analyze_users(
    user_ids: List[str],
    credentials: HTTPAuthorizationCredentials = Depends(security)
):
    """Batch analysis endpoint for multiple users"""
    try:
        # Verify token
        jwt.decode(
            credentials.credentials,
            bot_detection_service.config.JWT_SECRET,
            algorithms=["HS256"]
        )
        
        # Limit batch size for performance
        if len(user_ids) > 100:
            raise HTTPException(
                status_code=400, 
                detail="Batch size limited to 100 users"
            )
        
        results = await bot_detection_service.batch_analyze_users(user_ids)
        
        return {
            'batch_id': hashlib.md5(str(user_ids).encode()).hexdigest()[:8],
            'user_count': len(user_ids),
            'results': results,
            'timestamp': datetime.utcnow().isoformat()
        }
        
    except jwt.InvalidTokenError:
        raise HTTPException(status_code=401, detail="Invalid authentication token")
    except Exception as e:
        logger.error(f"Batch analysis failed: {e}")
        raise HTTPException(status_code=500, detail="Batch analysis error")

@app.get("/health")
async def health_check():
    """Service health check endpoint"""
    return await bot_detection_service.health_check()

@app.get("/metrics")
async def get_metrics(credentials: HTTPAuthorizationCredentials = Depends(security)):
    """Get service performance metrics"""
    try:
        # Verify admin token
        token_data = jwt.decode(
            credentials.credentials,
            bot_detection_service.config.JWT_SECRET,
            algorithms=["HS256"]
        )
        
        if not token_data.get('is_admin'):
            raise HTTPException(status_code=403, detail="Admin access required")
        
        uptime = time.time() - bot_detection_service.start_time
        
        return {
            'service_uptime_hours': uptime / 3600,
            'total_analyses': bot_detection_service.analysis_count,
            'bot_detections': bot_detection_service.bot_detection_count,
            'false_positives': bot_detection_service.false_positive_count,
            'detection_rate': (
                bot_detection_service.bot_detection_count / 
                max(1, bot_detection_service.analysis_count)
            ),
            'system_load': await bot_detection_service._get_system_load(),
            'timestamp': datetime.utcnow().isoformat()
        }
        
    except jwt.InvalidTokenError:
        raise HTTPException(status_code=401, detail="Invalid authentication token")
    except Exception as e:
        logger.error(f"Metrics retrieval failed: {e}")
        raise HTTPException(status_code=500, detail="Failed to retrieve metrics")

async def main():
    """Main entry point for standalone execution"""
    import uvicorn
    
    logger.info("Starting Finova Bot Detection Service...")
    
    # Configuration
    host = os.getenv('HOST', '0.0.0.0')
    port = int(os.getenv('PORT', 8000))
    workers = int(os.getenv('WORKERS', 1))
    
    # Production configuration
    config = uvicorn.Config(
        app=app,
        host=host,
        port=port,
        workers=workers,
        loop="asyncio",
        log_level="info",
        access_log=True,
        use_colors=True,
        reload=os.getenv('ENVIRONMENT') == 'development'
    )
    
    server = uvicorn.Server(config)
    
    try:
        await server.serve()
    except KeyboardInterrupt:
        logger.info("Service stopped by user")
    except Exception as e:
        logger.error(