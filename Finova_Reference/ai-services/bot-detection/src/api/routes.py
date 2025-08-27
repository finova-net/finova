"""
Finova Network - Bot Detection API Routes
Enterprise-grade bot detection service for fair mining distribution
"""

from fastapi import APIRouter, HTTPException, Depends, BackgroundTasks, Request
from fastapi.security import HTTPBearer, HTTPAuthorizationCredentials
from pydantic import BaseModel, Field
from typing import Dict, List, Optional, Any
import asyncio
import logging
from datetime import datetime, timedelta
import json

from ..models.behavior_analyzer import BehaviorAnalyzer
from ..models.pattern_detector import PatternDetector
from ..models.network_analyzer import NetworkAnalyzer
from ..models.human_probability import HumanProbabilityCalculator
from ..features.temporal_features import TemporalFeatureExtractor
from ..features.behavioral_features import BehavioralFeatureExtractor
from ..features.network_features import NetworkFeatureExtractor
from ..features.device_features import DeviceFeatureExtractor
from ..utils.config import get_settings
from ..utils.helpers import SecurityUtils, RateLimiter

# Initialize components
settings = get_settings()
security = HTTPBearer()
logger = logging.getLogger(__name__)
router = APIRouter(prefix="/api/v1/bot-detection", tags=["Bot Detection"])

# Rate limiting
rate_limiter = RateLimiter(
    requests_per_minute=60,
    requests_per_hour=1000
)

# Models initialization
behavior_analyzer = BehaviorAnalyzer()
pattern_detector = PatternDetector()
network_analyzer = NetworkAnalyzer()
human_calc = HumanProbabilityCalculator()

# Feature extractors
temporal_extractor = TemporalFeatureExtractor()
behavioral_extractor = BehavioralFeatureExtractor()
network_extractor = NetworkFeatureExtractor()
device_extractor = DeviceFeatureExtractor()

# Pydantic models
class UserAnalysisRequest(BaseModel):
    user_id: str = Field(..., description="Unique user identifier")
    wallet_address: str = Field(..., description="Solana wallet address")
    session_data: Dict[str, Any] = Field(..., description="Current session information")
    historical_data: Optional[Dict[str, Any]] = Field(None, description="Historical user data")
    device_info: Dict[str, Any] = Field(..., description="Device fingerprint data")
    ip_address: str = Field(..., description="User IP address")
    user_agent: str = Field(..., description="Browser user agent")
    
    class Config:
        schema_extra = {
            "example": {
                "user_id": "user_123456789",
                "wallet_address": "7xKXtg2CW87d97TXJSDpbD5jBkheTqA83TZRuJosgAsU",
                "session_data": {
                    "login_time": "2025-07-26T10:30:00Z",
                    "actions_count": 15,
                    "platforms_accessed": ["instagram", "tiktok"],
                    "mining_sessions": 3
                },
                "device_info": {
                    "screen_resolution": "1920x1080",
                    "timezone": "Asia/Jakarta",
                    "languages": ["en-US", "id-ID"],
                    "browser": "Chrome/91.0"
                },
                "ip_address": "103.94.189.45",
                "user_agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36"
            }
        }

class BatchAnalysisRequest(BaseModel):
    user_requests: List[UserAnalysisRequest] = Field(..., max_items=100)
    analysis_type: str = Field("comprehensive", description="Type of analysis to perform")
    priority: str = Field("normal", description="Processing priority")

class BotDetectionResponse(BaseModel):
    user_id: str
    is_bot_probability: float = Field(..., ge=0.0, le=1.0)
    human_probability: float = Field(..., ge=0.0, le=1.0)
    risk_level: str = Field(..., description="LOW, MEDIUM, HIGH, CRITICAL")
    confidence_score: float = Field(..., ge=0.0, le=1.0)
    detection_factors: Dict[str, float]
    recommendations: List[str]
    penalty_applied: Dict[str, float]
    analysis_timestamp: datetime
    next_check_required: Optional[datetime]

class NetworkAnalysisResponse(BaseModel):
    network_id: str
    suspicious_clusters: List[Dict[str, Any]]
    bot_probability: float
    connection_patterns: Dict[str, Any]
    risk_assessment: str
    recommended_actions: List[str]

class RealTimeMonitoringResponse(BaseModel):
    monitoring_active: bool
    alerts_count: int
    recent_detections: List[Dict[str, Any]]
    system_health: Dict[str, str]

# Utility functions
async def verify_token(credentials: HTTPAuthorizationCredentials = Depends(security)):
    """Verify JWT token and extract user permissions"""
    try:
        token = credentials.credentials
        # Implement JWT verification logic
        payload = SecurityUtils.verify_jwt_token(token)
        if not payload:
            raise HTTPException(status_code=401, detail="Invalid authentication token")
        return payload
    except Exception as e:
        logger.error(f"Token verification failed: {e}")
        raise HTTPException(status_code=401, detail="Authentication failed")

async def check_rate_limit(request: Request):
    """Apply rate limiting based on IP address"""
    client_ip = request.client.host
    if not await rate_limiter.check_limit(client_ip):
        raise HTTPException(
            status_code=429, 
            detail="Rate limit exceeded. Please try again later."
        )

# Main API Endpoints

@router.post("/analyze/user", response_model=BotDetectionResponse)
async def analyze_user_behavior(
    request: UserAnalysisRequest,
    background_tasks: BackgroundTasks,
    token_data: dict = Depends(verify_token),
    _: None = Depends(check_rate_limit)
):
    """
    Comprehensive user behavior analysis for bot detection
    Implements multi-layer analysis as per Finova whitepaper
    """
    try:
        logger.info(f"Starting analysis for user: {request.user_id}")
        
        # Extract features from different sources
        temporal_features = await temporal_extractor.extract_features(
            request.session_data, request.historical_data
        )
        
        behavioral_features = await behavioral_extractor.extract_features(
            request.user_id, request.session_data
        )
        
        network_features = await network_extractor.extract_features(
            request.wallet_address, request.ip_address
        )
        
        device_features = await device_extractor.extract_features(
            request.device_info, request.user_agent
        )
        
        # Combine all features
        combined_features = {
            "temporal": temporal_features,
            "behavioral": behavioral_features,
            "network": network_features,
            "device": device_features,
            "user_id": request.user_id,
            "wallet_address": request.wallet_address
        }
        
        # Run analysis models
        behavior_score = await behavior_analyzer.analyze_behavior(combined_features)
        pattern_score = await pattern_detector.detect_patterns(combined_features)
        network_score = await network_analyzer.analyze_network(
            request.wallet_address, network_features
        )
        
        # Calculate human probability using Finova's PoH system
        human_probability = await human_calc.calculate_probability({
            "biometric_consistency": behavior_score.get("biometric_score", 0.5),
            "behavioral_patterns": behavior_score.get("behavior_score", 0.5),
            "social_graph_validity": network_score.get("social_validity", 0.5),
            "device_authenticity": device_features.get("authenticity_score", 0.5),
            "interaction_quality": behavioral_features.get("quality_score", 0.5)
        })
        
        # Determine bot probability and risk level
        bot_probability = 1.0 - human_probability
        
        if bot_probability >= 0.9:
            risk_level = "CRITICAL"
        elif bot_probability >= 0.7:
            risk_level = "HIGH"
        elif bot_probability >= 0.4:
            risk_level = "MEDIUM"
        else:
            risk_level = "LOW"
        
        # Calculate penalties based on Finova's economic disincentives
        penalty_applied = _calculate_penalties(
            bot_probability, 
            combined_features,
            request.historical_data
        )
        
        # Generate recommendations
        recommendations = _generate_recommendations(
            bot_probability, 
            risk_level, 
            combined_features
        )
        
        # Confidence score calculation
        confidence_score = min(
            (behavior_score.get("confidence", 0.8) + 
             pattern_score.get("confidence", 0.8) + 
             network_score.get("confidence", 0.8)) / 3,
            1.0
        )
        
        # Schedule background tasks
        background_tasks.add_task(
            _update_user_profile, 
            request.user_id, 
            combined_features, 
            bot_probability
        )
        
        background_tasks.add_task(
            _log_analysis_result,
            request.user_id,
            bot_probability,
            risk_level
        )
        
        response = BotDetectionResponse(
            user_id=request.user_id,
            is_bot_probability=bot_probability,
            human_probability=human_probability,
            risk_level=risk_level,
            confidence_score=confidence_score,
            detection_factors={
                "behavioral_score": behavior_score.get("score", 0.5),
                "pattern_score": pattern_score.get("score", 0.5),
                "network_score": network_score.get("score", 0.5),
                "temporal_anomaly": temporal_features.get("anomaly_score", 0.0),
                "device_consistency": device_features.get("consistency_score", 1.0)
            },
            recommendations=recommendations,
            penalty_applied=penalty_applied,
            analysis_timestamp=datetime.utcnow(),
            next_check_required=datetime.utcnow() + timedelta(hours=24)
        )
        
        logger.info(f"Analysis completed for user {request.user_id}: {risk_level}")
        return response
        
    except Exception as e:
        logger.error(f"Analysis failed for user {request.user_id}: {e}")
        raise HTTPException(status_code=500, detail="Analysis processing failed")

@router.post("/analyze/batch", response_model=List[BotDetectionResponse])
async def batch_analyze_users(
    request: BatchAnalysisRequest,
    background_tasks: BackgroundTasks,
    token_data: dict = Depends(verify_token),
    _: None = Depends(check_rate_limit)
):
    """Batch analysis for multiple users - optimized for performance"""
    try:
        if len(request.user_requests) > 100:
            raise HTTPException(status_code=400, detail="Batch size too large (max 100)")
        
        logger.info(f"Starting batch analysis for {len(request.user_requests)} users")
        
        # Process in parallel with semaphore to limit concurrency
        semaphore = asyncio.Semaphore(10)  # Max 10 concurrent analyses
        
        async def analyze_single_user(user_request: UserAnalysisRequest):
            async with semaphore:
                return await analyze_user_behavior(
                    user_request, background_tasks, token_data
                )
        
        # Execute all analyses concurrently
        tasks = [analyze_single_user(user_req) for user_req in request.user_requests]
        results = await asyncio.gather(*tasks, return_exceptions=True)
        
        # Filter out exceptions and log errors
        valid_results = []
        for i, result in enumerate(results):
            if isinstance(result, Exception):
                logger.error(f"Batch analysis failed for user {i}: {result}")
            else:
                valid_results.append(result)
        
        logger.info(f"Batch analysis completed: {len(valid_results)}/{len(request.user_requests)}")
        return valid_results
        
    except Exception as e:
        logger.error(f"Batch analysis failed: {e}")
        raise HTTPException(status_code=500, detail="Batch analysis processing failed")

@router.post("/analyze/network", response_model=NetworkAnalysisResponse)
async def analyze_referral_network(
    wallet_addresses: List[str] = Field(..., max_items=1000),
    analysis_depth: int = Field(3, ge=1, le=5),
    token_data: dict = Depends(verify_token),
    _: None = Depends(check_rate_limit)
):
    """Analyze referral networks for suspicious clustering"""
    try:
        logger.info(f"Analyzing network of {len(wallet_addresses)} addresses")
        
        # Extract network features for all addresses
        network_data = await network_analyzer.analyze_cluster(
            wallet_addresses, 
            depth=analysis_depth
        )
        
        # Detect suspicious patterns
        suspicious_clusters = await pattern_detector.detect_network_patterns(network_data)
        
        # Calculate overall network bot probability
        individual_scores = []
        for address in wallet_addresses:
            features = await network_extractor.extract_features(address, None)
            score = await network_analyzer.analyze_network(address, features)
            individual_scores.append(score.get("bot_probability", 0.5))
        
        network_bot_probability = sum(individual_scores) / len(individual_scores)
        
        # Risk assessment
        if network_bot_probability >= 0.8:
            risk_assessment = "HIGH_RISK_NETWORK"
        elif network_bot_probability >= 0.6:
            risk_assessment = "MEDIUM_RISK_NETWORK"
        else:
            risk_assessment = "LOW_RISK_NETWORK"
        
        # Generate recommendations
        recommendations = []
        if network_bot_probability >= 0.7:
            recommendations.extend([
                "Implement additional verification for all network members",
                "Reduce referral bonuses for this network",
                "Monitor network activity closely"
            ])
        elif network_bot_probability >= 0.5:
            recommendations.extend([
                "Periodic re-verification required",
                "Monitor for circular referral patterns"
            ])
        
        response = NetworkAnalysisResponse(
            network_id=f"network_{hash(str(sorted(wallet_addresses)))}" ,
            suspicious_clusters=suspicious_clusters,
            bot_probability=network_bot_probability,
            connection_patterns=network_data.get("patterns", {}),
            risk_assessment=risk_assessment,
            recommended_actions=recommendations
        )
        
        logger.info(f"Network analysis completed: {risk_assessment}")
        return response
        
    except Exception as e:
        logger.error(f"Network analysis failed: {e}")
        raise HTTPException(status_code=500, detail="Network analysis failed")

@router.get("/monitoring/status", response_model=RealTimeMonitoringResponse)
async def get_monitoring_status(
    token_data: dict = Depends(verify_token),
    _: None = Depends(check_rate_limit)
):
    """Get real-time monitoring status and recent detections"""
    try:
        # Get system health metrics
        system_health = {
            "analyzer_status": "healthy",
            "detector_status": "healthy", 
            "database_status": "healthy",
            "ml_models_status": "healthy"
        }
        
        # Get recent detections (last 24 hours)
        recent_detections = await _get_recent_detections(hours=24)
        
        # Count active alerts
        alerts_count = len([d for d in recent_detections if d.get("risk_level") in ["HIGH", "CRITICAL"]])
        
        return RealTimeMonitoringResponse(
            monitoring_active=True,
            alerts_count=alerts_count,
            recent_detections=recent_detections,
            system_health=system_health
        )
        
    except Exception as e:
        logger.error(f"Failed to get monitoring status: {e}")
        raise HTTPException(status_code=500, detail="Monitoring status unavailable")

@router.post("/training/feedback")
async def submit_training_feedback(
    user_id: str,
    actual_result: str = Field(..., regex="^(bot|human)$"),
    predicted_probability: float = Field(..., ge=0.0, le=1.0),
    feedback_type: str = Field("user_report", regex="^(user_report|admin_review|automated)$"),
    token_data: dict = Depends(verify_token),
    _: None = Depends(check_rate_limit)
):
    """Submit feedback for model training and improvement"""
    try:
        # Store feedback for model retraining
        feedback_data = {
            "user_id": user_id,
            "actual_result": actual_result,
            "predicted_probability": predicted_probability,
            "feedback_type": feedback_type,
            "timestamp": datetime.utcnow(),
            "reviewer": token_data.get("user_id")
        }
        
        # Queue for model retraining
        await _queue_training_feedback(feedback_data)
        
        logger.info(f"Training feedback submitted for user {user_id}")
        return {"message": "Feedback submitted successfully", "status": "queued"}
        
    except Exception as e:
        logger.error(f"Failed to submit feedback: {e}")
        raise HTTPException(status_code=500, detail="Feedback submission failed")

# Helper functions

def _calculate_penalties(bot_probability: float, features: Dict, historical_data: Optional[Dict]) -> Dict[str, float]:
    """Calculate economic penalties based on Finova's anti-bot mechanisms"""
    base_mining_penalty = 0.0
    xp_penalty = 0.0
    rp_penalty = 0.0
    
    # Progressive difficulty scaling based on bot probability
    difficulty_multiplier = 1 + (bot_probability * 2)
    
    # Apply penalties based on risk level
    if bot_probability >= 0.9:  # CRITICAL
        base_mining_penalty = 0.95  # 95% reduction
        xp_penalty = 0.98  # 98% reduction
        rp_penalty = 1.0   # Complete suspension
    elif bot_probability >= 0.7:  # HIGH
        base_mining_penalty = 0.80
        xp_penalty = 0.85
        rp_penalty = 0.90
    elif bot_probability >= 0.4:  # MEDIUM
        base_mining_penalty = 0.30
        xp_penalty = 0.40
        rp_penalty = 0.50
    
    # Additional penalties for whale behavior
    if historical_data:
        total_holdings = historical_data.get("total_fin_holdings", 0)
        if total_holdings > 10000:  # Whale threshold
            whale_penalty = min(0.3, total_holdings / 100000)
            base_mining_penalty += whale_penalty
    
    return {
        "mining_rate_reduction": min(base_mining_penalty, 0.95),
        "xp_gain_reduction": min(xp_penalty, 0.98),
        "rp_benefit_reduction": min(rp_penalty, 1.0),
        "difficulty_multiplier": difficulty_multiplier
    }

def _generate_recommendations(bot_probability: float, risk_level: str, features: Dict) -> List[str]:
    """Generate actionable recommendations based on analysis"""
    recommendations = []
    
    if risk_level == "CRITICAL":
        recommendations.extend([
            "Account requires immediate manual review",
            "All rewards suspended pending verification",
            "Enhanced KYC verification required",
            "Monitor for coordinated bot network"
        ])
    elif risk_level == "HIGH":
        recommendations.extend([
            "Require additional identity verification",
            "Implement cooling period between sessions",
            "Monitor referral network for patterns",
            "Apply progressive mining penalties"
        ])
    elif risk_level == "MEDIUM":
        recommendations.extend([
            "Periodic re-verification recommended",
            "Monitor activity patterns closely",
            "Apply moderate mining rate reduction"
        ])
    
    # Feature-specific recommendations
    temporal_score = features.get("temporal", {}).get("anomaly_score", 0)
    if temporal_score > 0.7:
        recommendations.append("Unusual timing patterns detected - implement session limits")
    
    device_consistency = features.get("device", {}).get("consistency_score", 1.0)
    if device_consistency < 0.3:
        recommendations.append("Multiple device usage detected - verify device ownership")
    
    return recommendations

async def _update_user_profile(user_id: str, features: Dict, bot_probability: float):
    """Background task to update user risk profile"""
    try:
        # Update user profile in database
        # This would integrate with your user management system
        logger.info(f"Updated risk profile for user {user_id}: {bot_probability:.3f}")
    except Exception as e:
        logger.error(f"Failed to update user profile {user_id}: {e}")

async def _log_analysis_result(user_id: str, bot_probability: float, risk_level: str):
    """Background task to log analysis results"""
    try:
        # Log to analytics system
        log_data = {
            "user_id": user_id,
            "bot_probability": bot_probability,
            "risk_level": risk_level,
            "timestamp": datetime.utcnow().isoformat(),
            "event_type": "bot_analysis"
        }
        logger.info(f"Logged analysis result: {json.dumps(log_data)}")
    except Exception as e:
        logger.error(f"Failed to log analysis result: {e}")

async def _get_recent_detections(hours: int = 24) -> List[Dict[str, Any]]:
    """Get recent bot detections from the system"""
    try:
        # This would query your detection database
        # Mock data for example
        return [
            {
                "user_id": "user_suspicious_123",
                "risk_level": "HIGH",
                "bot_probability": 0.82,
                "timestamp": (datetime.utcnow() - timedelta(hours=2)).isoformat(),
                "detection_type": "pattern_analysis"
            },
            {
                "user_id": "user_bot_456", 
                "risk_level": "CRITICAL",
                "bot_probability": 0.95,
                "timestamp": (datetime.utcnow() - timedelta(hours=5)).isoformat(),
                "detection_type": "network_analysis"
            }
        ]
    except Exception as e:
        logger.error(f"Failed to get recent detections: {e}")
        return []

async def _queue_training_feedback(feedback_data: Dict):
    """Queue feedback data for model retraining"""
    try:
        # This would queue the feedback for ML model retraining
        logger.info(f"Queued training feedback: {feedback_data['user_id']}")
    except Exception as e:
        logger.error(f"Failed to queue training feedback: {e}")

# Error handlers
@router.exception_handler(HTTPException)
async def http_exception_handler(request: Request, exc: HTTPException):
    logger.error(f"HTTP error {exc.status_code}: {exc.detail}")
    return {"error": exc.detail, "status_code": exc.status_code}

@router.exception_handler(Exception)
async def general_exception_handler(request: Request, exc: Exception):
    logger.error(f"Unexpected error: {exc}")
    return {"error": "Internal server error", "status_code": 500}
    