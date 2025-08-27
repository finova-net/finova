"""
Finova Network - AI Bot Detection: Network Analyzer
Enterprise-grade network analysis for detecting bot networks and referral farming.

This module implements advanced graph analysis and network pattern detection
to identify suspicious user networks, circular referral schemes, and coordinated inauthentic behavior.
"""

import numpy as np
import networkx as nx
from typing import Dict, List, Tuple, Set, Optional, Any
from dataclasses import dataclass
from datetime import datetime, timedelta
import logging
from concurrent.futures import ThreadPoolExecutor
import asyncio
from scipy.stats import chi2_contingency, kstest
from sklearn.cluster import DBSCAN
from sklearn.preprocessing import StandardScaler
import redis
import hashlib

# Configure logging
logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)

@dataclass
class UserNetworkProfile:
    """Profile data for network analysis"""
    user_id: str
    registration_date: datetime
    referrals_count: int
    active_referrals: int
    network_depth: int
    total_network_size: int
    avg_referral_activity: float
    network_diversity_score: float
    temporal_patterns: List[float]
    geographical_distribution: Dict[str, int]
    device_fingerprints: Set[str]
    ip_addresses: Set[str]
    behavioral_similarity: float

@dataclass
class NetworkSuspicionResult:
    """Result of network suspicion analysis"""
    user_id: str
    suspicion_score: float
    risk_factors: Dict[str, float]
    network_cluster: Optional[str]
    recommendations: List[str]
    confidence: float

class NetworkAnalyzer:
    """
    Advanced network analyzer for detecting bot networks and referral farming.
    
    Implements multiple detection algorithms:
    - Graph-based anomaly detection
    - Temporal pattern analysis
    - Clustering-based bot detection
    - Referral network quality assessment
    """
    
    def __init__(self, redis_client: redis.Redis):
        self.redis_client = redis_client
        self.graph = nx.DiGraph()
        self.scaler = StandardScaler()
        
        # Detection thresholds (configurable)
        self.config = {
            'max_referrals_per_hour': 10,
            'max_network_similarity': 0.8,
            'min_network_diversity': 0.3,
            'suspicious_cluster_size': 50,
            'temporal_variance_threshold': 0.1,
            'ip_sharing_threshold': 5,
            'device_sharing_threshold': 3
        }
        
        # Cache for expensive computations
        self.cache_ttl = 3600  # 1 hour
    
    async def analyze_user_network(self, user_profile: UserNetworkProfile) -> NetworkSuspicionResult:
        """
        Comprehensive network analysis for a specific user.
        
        Args:
            user_profile: User's network profile data
            
        Returns:
            NetworkSuspicionResult with detailed analysis
        """
        try:
            # Parallel analysis of different aspects
            tasks = [
                self._analyze_referral_patterns(user_profile),
                self._analyze_network_structure(user_profile),
                self._analyze_temporal_patterns(user_profile),
                self._analyze_geographical_distribution(user_profile),
                self._analyze_device_fingerprints(user_profile)
            ]
            
            results = await asyncio.gather(*tasks)
            
            # Combine results with weighted scoring
            risk_factors = {}
            total_suspicion = 0.0
            weights = [0.3, 0.25, 0.2, 0.15, 0.1]  # Importance weights
            
            for i, result in enumerate(results):
                risk_factors.update(result)
                total_suspicion += sum(result.values()) * weights[i]
            
            # Network clustering analysis
            cluster_id = await self._identify_network_cluster(user_profile)
            
            # Generate recommendations
            recommendations = self._generate_recommendations(risk_factors, total_suspicion)
            
            # Calculate confidence based on data completeness
            confidence = self._calculate_confidence(user_profile, risk_factors)
            
            return NetworkSuspicionResult(
                user_id=user_profile.user_id,
                suspicion_score=min(total_suspicion, 1.0),
                risk_factors=risk_factors,
                network_cluster=cluster_id,
                recommendations=recommendations,
                confidence=confidence
            )
            
        except Exception as e:
            logger.error(f"Error analyzing user network {user_profile.user_id}: {e}")
            return NetworkSuspicionResult(
                user_id=user_profile.user_id,
                suspicion_score=0.5,  # Default moderate suspicion on error
                risk_factors={'analysis_error': 1.0},
                network_cluster=None,
                recommendations=['Manual review required due to analysis error'],
                confidence=0.0
            )
    
    async def _analyze_referral_patterns(self, profile: UserNetworkProfile) -> Dict[str, float]:
        """Analyze referral acquisition patterns for anomalies"""
        risk_factors = {}
        
        # Referral velocity analysis
        days_since_registration = (datetime.now() - profile.registration_date).days
        if days_since_registration > 0:
            referral_velocity = profile.referrals_count / days_since_registration
            if referral_velocity > self.config['max_referrals_per_hour'] * 24:
                risk_factors['high_referral_velocity'] = min(referral_velocity / 100, 1.0)
        
        # Network depth vs breadth ratio
        if profile.referrals_count > 0:
            depth_breadth_ratio = profile.network_depth / profile.referrals_count
            if depth_breadth_ratio > 3:  # Suspiciously deep narrow networks
                risk_factors['suspicious_network_shape'] = min(depth_breadth_ratio / 10, 1.0)
        
        # Active vs inactive referrals ratio
        if profile.referrals_count > 0:
            activity_ratio = profile.active_referrals / profile.referrals_count
            if activity_ratio < 0.1 and profile.referrals_count > 10:
                risk_factors['low_referral_activity'] = 1.0 - activity_ratio
        
        return risk_factors
    
    async def _analyze_network_structure(self, profile: UserNetworkProfile) -> Dict[str, float]:
        """Analyze network graph structure for bot patterns"""
        risk_factors = {}
        
        # Network diversity score
        if profile.network_diversity_score < self.config['min_network_diversity']:
            risk_factors['low_network_diversity'] = 1.0 - profile.network_diversity_score
        
        # Behavioral similarity (high similarity = potential bots)
        if profile.behavioral_similarity > self.config['max_network_similarity']:
            risk_factors['high_behavioral_similarity'] = profile.behavioral_similarity
        
        # Network size vs quality
        expected_quality = self._calculate_expected_network_quality(profile.total_network_size)
        if profile.avg_referral_activity < expected_quality * 0.5:
            risk_factors['poor_network_quality'] = 1.0 - (profile.avg_referral_activity / expected_quality)
        
        return risk_factors
    
    async def _analyze_temporal_patterns(self, profile: UserNetworkProfile) -> Dict[str, float]:
        """Analyze temporal patterns for bot-like behavior"""
        risk_factors = {}
        
        if not profile.temporal_patterns:
            return risk_factors
        
        # Calculate temporal variance (bots often have low variance)
        temporal_variance = np.var(profile.temporal_patterns)
        if temporal_variance < self.config['temporal_variance_threshold']:
            risk_factors['low_temporal_variance'] = 1.0 - (temporal_variance / self.config['temporal_variance_threshold'])
        
        # Detect unnatural periodicity
        fft_result = np.fft.fft(profile.temporal_patterns)
        dominant_frequency = np.argmax(np.abs(fft_result))
        if dominant_frequency > 0:
            periodicity_strength = np.abs(fft_result[dominant_frequency]) / len(profile.temporal_patterns)
            if periodicity_strength > 0.5:  # Strong artificial periodicity
                risk_factors['artificial_periodicity'] = periodicity_strength
        
        return risk_factors
    
    async def _analyze_geographical_distribution(self, profile: UserNetworkProfile) -> Dict[str, float]:
        """Analyze geographical patterns for anomalies"""
        risk_factors = {}
        
        if not profile.geographical_distribution:
            return risk_factors
        
        total_users = sum(profile.geographical_distribution.values())
        if total_users == 0:
            return risk_factors
        
        # Calculate geographical concentration
        max_concentration = max(profile.geographical_distribution.values()) / total_users
        if max_concentration > 0.8 and total_users > 10:
            risk_factors['geographical_concentration'] = max_concentration
        
        # Check for impossible geographical spread
        unique_locations = len(profile.geographical_distribution)
        if unique_locations > total_users * 0.5 and total_users > 20:  # Too many locations
            risk_factors['impossible_geographic_spread'] = min(unique_locations / total_users, 1.0)
        
        return risk_factors
    
    async def _analyze_device_fingerprints(self, profile: UserNetworkProfile) -> Dict[str, float]:
        """Analyze device and IP patterns for sharing"""
        risk_factors = {}
        
        # Device fingerprint sharing
        if len(profile.device_fingerprints) > 0:
            device_sharing_ratio = profile.total_network_size / len(profile.device_fingerprints)
            if device_sharing_ratio > self.config['device_sharing_threshold']:
                risk_factors['device_sharing'] = min(device_sharing_ratio / 10, 1.0)
        
        # IP address sharing
        if len(profile.ip_addresses) > 0:
            ip_sharing_ratio = profile.total_network_size / len(profile.ip_addresses)
            if ip_sharing_ratio > self.config['ip_sharing_threshold']:
                risk_factors['ip_sharing'] = min(ip_sharing_ratio / 20, 1.0)
        
        return risk_factors
    
    async def _identify_network_cluster(self, profile: UserNetworkProfile) -> Optional[str]:
        """Identify if user belongs to a suspicious cluster"""
        try:
            # Create feature vector for clustering
            features = [
                profile.referrals_count,
                profile.network_depth,
                profile.avg_referral_activity,
                profile.network_diversity_score,
                profile.behavioral_similarity,
                len(profile.device_fingerprints),
                len(profile.ip_addresses)
            ]
            
            # Normalize features
            features_normalized = self.scaler.fit_transform([features])
            
            # Generate cluster ID based on profile characteristics
            cluster_hash = hashlib.sha256(str(sorted(features)).encode()).hexdigest()[:8]
            
            # Cache cluster information
            await self._cache_cluster_info(cluster_hash, profile)
            
            return cluster_hash
            
        except Exception as e:
            logger.error(f"Error in cluster identification: {e}")
            return None
    
    def _calculate_expected_network_quality(self, network_size: int) -> float:
        """Calculate expected network quality based on size"""
        # Exponential decay similar to Finova's regression model
        base_quality = 0.8
        decay_factor = 0.001
        return base_quality * np.exp(-decay_factor * network_size)
    
    def _generate_recommendations(self, risk_factors: Dict[str, float], total_suspicion: float) -> List[str]:
        """Generate actionable recommendations based on risk factors"""
        recommendations = []
        
        if total_suspicion > 0.8:
            recommendations.append("URGENT: Suspend account pending manual review")
        elif total_suspicion > 0.6:
            recommendations.append("Require additional verification")
            
        if 'high_referral_velocity' in risk_factors:
            recommendations.append("Implement referral rate limiting")
            
        if 'device_sharing' in risk_factors or 'ip_sharing' in risk_factors:
            recommendations.append("Flag for device/IP investigation")
            
        if 'low_network_diversity' in risk_factors:
            recommendations.append("Monitor for coordinated behavior")
            
        if 'artificial_periodicity' in risk_factors:
            recommendations.append("Investigate for automated behavior")
            
        if not recommendations:
            recommendations.append("Continue monitoring with standard protocols")
            
        return recommendations
    
    def _calculate_confidence(self, profile: UserNetworkProfile, risk_factors: Dict[str, float]) -> float:
        """Calculate confidence score based on data completeness and consistency"""
        data_completeness_factors = [
            1.0 if profile.temporal_patterns else 0.0,
            1.0 if profile.geographical_distribution else 0.0,
            1.0 if profile.device_fingerprints else 0.0,
            1.0 if profile.ip_addresses else 0.0,
            1.0 if profile.total_network_size > 0 else 0.0
        ]
        
        data_completeness = np.mean(data_completeness_factors)
        
        # Higher confidence if we have more risk factors (more evidence)
        evidence_strength = min(len(risk_factors) / 5, 1.0)
        
        # Time factor (older accounts have more reliable data)
        days_since_registration = (datetime.now() - profile.registration_date).days
        time_factor = min(days_since_registration / 30, 1.0)  # Max confidence after 30 days
        
        return (data_completeness + evidence_strength + time_factor) / 3
    
    async def _cache_cluster_info(self, cluster_id: str, profile: UserNetworkProfile):
        """Cache cluster information for pattern recognition"""
        try:
            cache_key = f"network_cluster:{cluster_id}"
            cluster_data = {
                'users': [profile.user_id],
                'avg_referrals': profile.referrals_count,
                'avg_network_size': profile.total_network_size,
                'last_updated': datetime.now().isoformat()
            }
            
            # Try to get existing data and merge
            existing_data = await self._get_cached_data(cache_key)
            if existing_data:
                existing_data['users'].append(profile.user_id)
                existing_data['avg_referrals'] = (existing_data['avg_referrals'] + profile.referrals_count) / 2
                existing_data['avg_network_size'] = (existing_data['avg_network_size'] + profile.total_network_size) / 2
                cluster_data = existing_data
            
            self.redis_client.setex(cache_key, self.cache_ttl, str(cluster_data))
            
        except Exception as e:
            logger.error(f"Error caching cluster info: {e}")
    
    async def _get_cached_data(self, key: str) -> Optional[Dict]:
        """Get cached data with error handling"""
        try:
            data = self.redis_client.get(key)
            return eval(data.decode()) if data else None
        except Exception as e:
            logger.error(f"Error retrieving cached data: {e}")
            return None
    
    async def analyze_network_clusters(self, user_profiles: List[UserNetworkProfile]) -> Dict[str, List[str]]:
        """
        Analyze multiple users to identify bot clusters.
        
        Args:
            user_profiles: List of user profiles to analyze
            
        Returns:
            Dictionary mapping cluster IDs to lists of suspicious user IDs
        """
        suspicious_clusters = {}
        
        try:
            # Extract features for clustering
            features = []
            user_ids = []
            
            for profile in user_profiles:
                feature_vector = [
                    profile.referrals_count,
                    profile.network_depth,
                    profile.avg_referral_activity,
                    profile.network_diversity_score,
                    profile.behavioral_similarity,
                    len(profile.device_fingerprints) if profile.device_fingerprints else 0,
                    len(profile.ip_addresses) if profile.ip_addresses else 0,
                    profile.total_network_size
                ]
                features.append(feature_vector)
                user_ids.append(profile.user_id)
            
            if len(features) < 2:
                return suspicious_clusters
            
            # Normalize features
            features_normalized = self.scaler.fit_transform(features)
            
            # Apply DBSCAN clustering
            clustering = DBSCAN(eps=0.3, min_samples=5).fit(features_normalized)
            labels = clustering.labels_
            
            # Identify suspicious clusters
            for i, label in enumerate(labels):
                if label != -1:  # Not noise
                    cluster_id = f"cluster_{label}"
                    if cluster_id not in suspicious_clusters:
                        suspicious_clusters[cluster_id] = []
                    suspicious_clusters[cluster_id].append(user_ids[i])
            
            # Filter clusters by size and suspicion criteria
            filtered_clusters = {}
            for cluster_id, user_list in suspicious_clusters.items():
                if len(user_list) >= 5:  # Minimum cluster size for investigation
                    # Calculate cluster suspicion metrics
                    cluster_profiles = [p for p in user_profiles if p.user_id in user_list]
                    cluster_suspicion = await self._calculate_cluster_suspicion(cluster_profiles)
                    
                    if cluster_suspicion > 0.5:
                        filtered_clusters[cluster_id] = user_list
            
            return filtered_clusters
            
        except Exception as e:
            logger.error(f"Error in network cluster analysis: {e}")
            return {}
    
    async def _calculate_cluster_suspicion(self, cluster_profiles: List[UserNetworkProfile]) -> float:
        """Calculate overall suspicion score for a cluster"""
        if not cluster_profiles:
            return 0.0
        
        # Calculate cluster-level metrics
        avg_behavioral_similarity = np.mean([p.behavioral_similarity for p in cluster_profiles])
        avg_network_diversity = np.mean([p.network_diversity_score for p in cluster_profiles])
        registration_time_variance = np.var([p.registration_date.timestamp() for p in cluster_profiles])
        
        # Suspicious if high similarity, low diversity, similar registration times
        suspicion_factors = [
            avg_behavioral_similarity,  # High similarity is suspicious
            1.0 - avg_network_diversity,  # Low diversity is suspicious
            1.0 - min(registration_time_variance / 86400, 1.0)  # Low time variance is suspicious
        ]
        
        return np.mean(suspicion_factors)

# Utility functions for integration
def create_user_network_profile(user_data: Dict[str, Any]) -> UserNetworkProfile:
    """Create UserNetworkProfile from raw user data"""
    return UserNetworkProfile(
        user_id=user_data.get('user_id', ''),
        registration_date=datetime.fromisoformat(user_data.get('registration_date', datetime.now().isoformat())),
        referrals_count=user_data.get('referrals_count', 0),
        active_referrals=user_data.get('active_referrals', 0),
        network_depth=user_data.get('network_depth', 0),
        total_network_size=user_data.get('total_network_size', 0),
        avg_referral_activity=user_data.get('avg_referral_activity', 0.0),
        network_diversity_score=user_data.get('network_diversity_score', 0.0),
        temporal_patterns=user_data.get('temporal_patterns', []),
        geographical_distribution=user_data.get('geographical_distribution', {}),
        device_fingerprints=set(user_data.get('device_fingerprints', [])),
        ip_addresses=set(user_data.get('ip_addresses', [])),
        behavioral_similarity=user_data.get('behavioral_similarity', 0.0)
    )

# Example usage and testing
if __name__ == "__main__":
    import asyncio
    
    async def test_network_analyzer():
        """Test the network analyzer with sample data"""
        # Mock Redis client for testing
        redis_client = redis.Redis(host='localhost', port=6379, db=0, decode_responses=True)
        
        analyzer = NetworkAnalyzer(redis_client)
        
        # Create test profile
        test_profile = UserNetworkProfile(
            user_id="test_user_001",
            registration_date=datetime.now() - timedelta(days=30),
            referrals_count=50,
            active_referrals=5,
            network_depth=3,
            total_network_size=150,
            avg_referral_activity=0.2,
            network_diversity_score=0.25,
            temporal_patterns=[0.1, 0.1, 0.1, 0.1, 0.1] * 10,
            geographical_distribution={'ID': 140, 'MY': 8, 'SG': 2},
            device_fingerprints={'device_001', 'device_002'},
            ip_addresses={'192.168.1.1', '192.168.1.2', '192.168.1.3'},
            behavioral_similarity=0.85
        )
        
        # Analyze the network
        result = await analyzer.analyze_user_network(test_profile)
        
        print(f"Analysis Result for {result.user_id}:")
        print(f"Suspicion Score: {result.suspicion_score:.3f}")
        print(f"Confidence: {result.confidence:.3f}")
        print(f"Risk Factors: {result.risk_factors}")
        print(f"Recommendations: {result.recommendations}")
        
        return result
    
    # Run test
    asyncio.run(test_network_analyzer())
    