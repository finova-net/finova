"""
Finova Network - AI Bot Detection: Network Features Analyzer
Enterprise-grade network pattern analysis for detecting bot networks and referral farming.
"""

import numpy as np
import pandas as pd
from typing import Dict, List, Tuple, Optional, Set
from dataclasses import dataclass
from datetime import datetime, timedelta
import networkx as nx
from collections import defaultdict, Counter
import hashlib
import logging
from concurrent.futures import ThreadPoolExecutor
import redis
import asyncio
from enum import Enum

# Configure logging
logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)

class SuspicionLevel(Enum):
    """Suspicion levels for network patterns"""
    LOW = "low"
    MEDIUM = "medium" 
    HIGH = "high"
    CRITICAL = "critical"

@dataclass
class NetworkNode:
    """Represents a user node in the referral network"""
    user_id: str
    registration_time: datetime
    kyc_status: bool
    total_referrals: int
    active_referrals: int
    mining_rate: float
    xp_level: int
    rp_tier: str
    last_activity: datetime
    device_fingerprint: str
    ip_address_hash: str
    
@dataclass
class NetworkEdge:
    """Represents a referral relationship between users"""
    referrer_id: str
    referee_id: str
    referral_time: datetime
    activation_status: bool
    quality_score: float

@dataclass
class NetworkCluster:
    """Represents a detected cluster of related users"""
    cluster_id: str
    node_ids: List[str]
    suspicion_level: SuspicionLevel
    confidence_score: float
    detected_patterns: List[str]
    creation_time: datetime

class NetworkFeaturesAnalyzer:
    """
    Advanced network analysis for bot detection in Finova Network.
    Analyzes referral patterns, user clusters, and network topology.
    """
    
    def __init__(self, redis_client: Optional[redis.Redis] = None):
        """Initialize the network features analyzer"""
        self.redis_client = redis_client or redis.Redis(
            host='localhost', port=6379, db=0, decode_responses=True
        )
        self.graph = nx.DiGraph()
        self.suspicious_clusters: Dict[str, NetworkCluster] = {}
        
        # Network analysis thresholds
        self.thresholds = {
            'max_referrals_per_day': 50,
            'min_activation_rate': 0.3,
            'max_similar_device_ratio': 0.7,
            'min_time_between_referrals': 300,  # seconds
            'max_cluster_density': 0.8,
            'min_diversity_score': 0.4,
            'suspicious_pattern_threshold': 0.75
        }
        
        # Pattern weights for suspicion calculation
        self.pattern_weights = {
            'circular_referrals': 0.9,
            'burst_registrations': 0.8,
            'similar_devices': 0.85,
            'geographical_clustering': 0.7,
            'timing_patterns': 0.75,
            'low_quality_network': 0.8,
            'artificial_activity': 0.9
        }

    def analyze_user_network_features(self, user_id: str) -> Dict[str, float]:
        """
        Analyze network-based features for a specific user.
        Returns comprehensive network metrics for bot detection.
        """
        try:
            features = {}
            
            # Get user's network data
            user_data = self._get_user_network_data(user_id)
            if not user_data:
                return self._get_default_features()
            
            # 1. Referral Network Analysis
            referral_features = self._analyze_referral_patterns(user_id, user_data)
            features.update(referral_features)
            
            # 2. Clustering Analysis
            cluster_features = self._analyze_user_clustering(user_id, user_data)
            features.update(cluster_features)
            
            # 3. Graph Topology Features
            topology_features = self._analyze_network_topology(user_id)
            features.update(topology_features)
            
            # 4. Temporal Pattern Analysis
            temporal_features = self._analyze_temporal_patterns(user_id, user_data)
            features.update(temporal_features)
            
            # 5. Device and IP Correlation
            correlation_features = self._analyze_device_ip_correlation(user_id, user_data)
            features.update(correlation_features)
            
            # 6. Quality and Engagement Metrics
            quality_features = self._analyze_network_quality(user_id, user_data)
            features.update(quality_features)
            
            # Cache results for performance
            self._cache_features(user_id, features)
            
            logger.info(f"Network features analyzed for user {user_id}: {len(features)} features")
            return features
            
        except Exception as e:
            logger.error(f"Error analyzing network features for {user_id}: {e}")
            return self._get_default_features()

    def _analyze_referral_patterns(self, user_id: str, user_data: Dict) -> Dict[str, float]:
        """Analyze referral-specific patterns for bot detection"""
        features = {}
        
        referrals = user_data.get('referrals', [])
        if not referrals:
            return {
                'referral_count': 0.0,
                'referral_activation_rate': 0.0,
                'referral_burst_score': 0.0,
                'referral_time_variance': 0.0,
                'circular_referral_score': 0.0
            }
        
        # Basic referral metrics
        features['referral_count'] = min(len(referrals), 100) / 100.0
        
        # Activation rate analysis
        active_referrals = sum(1 for r in referrals if r.get('active', False))
        features['referral_activation_rate'] = active_referrals / len(referrals) if referrals else 0.0
        
        # Burst detection (many referrals in short time)
        referral_times = [datetime.fromisoformat(r['created_at']) for r in referrals]
        features['referral_burst_score'] = self._calculate_burst_score(referral_times)
        
        # Time variance analysis
        if len(referral_times) > 1:
            time_diffs = [(referral_times[i+1] - referral_times[i]).total_seconds() 
                         for i in range(len(referral_times)-1)]
            features['referral_time_variance'] = np.std(time_diffs) / max(np.mean(time_diffs), 1.0)
        else:
            features['referral_time_variance'] = 0.0
        
        # Circular referral detection
        features['circular_referral_score'] = self._detect_circular_referrals(user_id, referrals)
        
        return features

    def _analyze_user_clustering(self, user_id: str, user_data: Dict) -> Dict[str, float]:
        """Analyze clustering patterns that indicate bot networks"""
        features = {}
        
        # Get user's connected network
        connected_users = self._get_connected_users(user_id, depth=3)
        
        if len(connected_users) < 2:
            return {
                'cluster_density': 0.0,
                'cluster_size_score': 0.0,
                'device_similarity_score': 0.0,
                'ip_clustering_score': 0.0,
                'registration_clustering_score': 0.0
            }
        
        # Calculate cluster density
        cluster_graph = self.graph.subgraph(connected_users)
        if cluster_graph.number_of_nodes() > 1:
            features['cluster_density'] = nx.density(cluster_graph)
        else:
            features['cluster_density'] = 0.0
        
        # Cluster size normalization
        features['cluster_size_score'] = min(len(connected_users), 50) / 50.0
        
        # Device fingerprint similarity
        features['device_similarity_score'] = self._calculate_device_similarity(connected_users)
        
        # IP address clustering
        features['ip_clustering_score'] = self._calculate_ip_clustering(connected_users)
        
        # Registration time clustering
        features['registration_clustering_score'] = self._calculate_registration_clustering(connected_users)
        
        return features

    def _analyze_network_topology(self, user_id: str) -> Dict[str, float]:
        """Analyze graph topology features"""
        features = {}
        
        try:
            if user_id not in self.graph.nodes():
                return {
                    'betweenness_centrality': 0.0,
                    'closeness_centrality': 0.0,
                    'degree_centrality': 0.0,
                    'eigenvector_centrality': 0.0,
                    'pagerank_score': 0.0
                }
            
            # Centrality measures
            betweenness = nx.betweenness_centrality(self.graph)
            closeness = nx.closeness_centrality(self.graph)
            degree = nx.degree_centrality(self.graph)
            
            features['betweenness_centrality'] = betweenness.get(user_id, 0.0)
            features['closeness_centrality'] = closeness.get(user_id, 0.0)
            features['degree_centrality'] = degree.get(user_id, 0.0)
            
            # Eigenvector centrality (more robust for bot detection)
            try:
                eigenvector = nx.eigenvector_centrality(self.graph, max_iter=1000)
                features['eigenvector_centrality'] = eigenvector.get(user_id, 0.0)
            except:
                features['eigenvector_centrality'] = 0.0
            
            # PageRank score
            pagerank = nx.pagerank(self.graph)
            features['pagerank_score'] = pagerank.get(user_id, 0.0)
            
        except Exception as e:
            logger.warning(f"Topology analysis failed for {user_id}: {e}")
            features = {k: 0.0 for k in ['betweenness_centrality', 'closeness_centrality', 
                                       'degree_centrality', 'eigenvector_centrality', 'pagerank_score']}
        
        return features

    def _analyze_temporal_patterns(self, user_id: str, user_data: Dict) -> Dict[str, float]:
        """Analyze temporal patterns in user activity and network growth"""
        features = {}
        
        # Activity timing analysis
        activities = user_data.get('activities', [])
        if not activities:
            return {
                'activity_time_entropy': 0.0,
                'weekend_activity_ratio': 0.0,
                'night_activity_ratio': 0.0,
                'activity_periodicity_score': 0.0,
                'burst_activity_score': 0.0
            }
        
        activity_times = [datetime.fromisoformat(a['timestamp']) for a in activities]
        
        # Time entropy (human activity has high entropy, bots have low entropy)
        hour_counts = Counter([dt.hour for dt in activity_times])
        total_activities = len(activity_times)
        hour_probs = [count / total_activities for count in hour_counts.values()]
        features['activity_time_entropy'] = -sum(p * np.log2(p) for p in hour_probs if p > 0)
        
        # Weekend vs weekday activity
        weekend_activities = sum(1 for dt in activity_times if dt.weekday() >= 5)
        features['weekend_activity_ratio'] = weekend_activities / total_activities
        
        # Night activity (potential bot indicator)
        night_activities = sum(1 for dt in activity_times if dt.hour < 6 or dt.hour > 22)
        features['night_activity_ratio'] = night_activities / total_activities
        
        # Activity periodicity (bots often have regular patterns)
        features['activity_periodicity_score'] = self._calculate_periodicity_score(activity_times)
        
        # Burst activity detection
        features['burst_activity_score'] = self._calculate_burst_score(activity_times)
        
        return features

    def _analyze_device_ip_correlation(self, user_id: str, user_data: Dict) -> Dict[str, float]:
        """Analyze device and IP correlation patterns"""
        features = {}
        
        # Get device and IP data
        sessions = user_data.get('sessions', [])
        if not sessions:
            return {
                'device_consistency_score': 1.0,
                'ip_diversity_score': 1.0,
                'device_sharing_score': 0.0,
                'suspicious_ip_score': 0.0,
                'geolocation_consistency_score': 1.0
            }
        
        devices = [s.get('device_fingerprint') for s in sessions if s.get('device_fingerprint')]
        ips = [s.get('ip_hash') for s in sessions if s.get('ip_hash')]
        
        # Device consistency
        if devices:
            device_counts = Counter(devices)
            most_common_device_ratio = device_counts.most_common(1)[0][1] / len(devices)
            features['device_consistency_score'] = most_common_device_ratio
        else:
            features['device_consistency_score'] = 1.0
        
        # IP diversity
        if ips:
            unique_ips = len(set(ips))
            features['ip_diversity_score'] = min(unique_ips / len(ips), 1.0)
        else:
            features['ip_diversity_score'] = 1.0
        
        # Device sharing detection
        features['device_sharing_score'] = self._calculate_device_sharing_score(user_id, devices)
        
        # Suspicious IP detection
        features['suspicious_ip_score'] = self._calculate_suspicious_ip_score(ips)
        
        # Geolocation consistency
        features['geolocation_consistency_score'] = self._calculate_geo_consistency_score(sessions)
        
        return features

    def _analyze_network_quality(self, user_id: str, user_data: Dict) -> Dict[str, float]:
        """Analyze the quality of user's referral network"""
        features = {}
        
        referrals = user_data.get('referrals', [])
        if not referrals:
            return {
                'network_diversity_score': 0.0,
                'network_engagement_score': 0.0,
                'network_retention_score': 0.0,
                'network_quality_score': 0.0,
                'artificial_growth_score': 0.0
            }
        
        # Network diversity (different levels, platforms, etc.)
        referral_levels = [r.get('xp_level', 0) for r in referrals]
        referral_platforms = [r.get('primary_platform', 'unknown') for r in referrals]
        
        level_diversity = len(set(referral_levels)) / len(referrals) if referrals else 0.0
        platform_diversity = len(set(referral_platforms)) / len(referrals) if referrals else 0.0
        features['network_diversity_score'] = (level_diversity + platform_diversity) / 2.0
        
        # Network engagement
        avg_engagement = np.mean([r.get('engagement_score', 0.0) for r in referrals])
        features['network_engagement_score'] = min(avg_engagement / 100.0, 1.0)
        
        # Network retention
        active_referrals = sum(1 for r in referrals if r.get('last_active_days', 999) <= 7)
        features['network_retention_score'] = active_referrals / len(referrals) if referrals else 0.0
        
        # Overall network quality
        quality_scores = [r.get('quality_score', 0.0) for r in referrals]
        features['network_quality_score'] = np.mean(quality_scores) if quality_scores else 0.0
        
        # Artificial growth detection
        features['artificial_growth_score'] = self._detect_artificial_growth(user_id, referrals)
        
        return features

    def detect_suspicious_clusters(self, min_cluster_size: int = 5) -> List[NetworkCluster]:
        """Detect suspicious clusters in the entire network"""
        try:
            suspicious_clusters = []
            
            # Use community detection algorithms
            communities = self._detect_communities()
            
            for i, community in enumerate(communities):
                if len(community) < min_cluster_size:
                    continue
                
                cluster_features = self._analyze_cluster_features(community)
                suspicion_score = self._calculate_cluster_suspicion_score(cluster_features)
                
                if suspicion_score > self.thresholds['suspicious_pattern_threshold']:
                    suspicion_level = self._determine_suspicion_level(suspicion_score)
                    detected_patterns = self._identify_cluster_patterns(cluster_features)
                    
                    cluster = NetworkCluster(
                        cluster_id=f"cluster_{i}_{datetime.now().timestamp()}",
                        node_ids=list(community),
                        suspicion_level=suspicion_level,
                        confidence_score=suspicion_score,
                        detected_patterns=detected_patterns,
                        creation_time=datetime.now()
                    )
                    
                    suspicious_clusters.append(cluster)
                    self.suspicious_clusters[cluster.cluster_id] = cluster
            
            logger.info(f"Detected {len(suspicious_clusters)} suspicious clusters")
            return suspicious_clusters
            
        except Exception as e:
            logger.error(f"Error detecting suspicious clusters: {e}")
            return []

    def _get_user_network_data(self, user_id: str) -> Optional[Dict]:
        """Retrieve user network data from cache or database"""
        try:
            # Try cache first
            cached_data = self.redis_client.get(f"network_data:{user_id}")
            if cached_data:
                return eval(cached_data)  # In production, use proper JSON parsing
            
            # Fallback to database query (mock implementation)
            return self._fetch_user_data_from_db(user_id)
            
        except Exception as e:
            logger.error(f"Error fetching user network data: {e}")
            return None

    def _calculate_burst_score(self, timestamps: List[datetime]) -> float:
        """Calculate burst activity score"""
        if len(timestamps) < 3:
            return 0.0
        
        # Sort timestamps
        sorted_times = sorted(timestamps)
        
        # Calculate intervals
        intervals = [(sorted_times[i+1] - sorted_times[i]).total_seconds() 
                    for i in range(len(sorted_times)-1)]
        
        # Find short intervals (< 5 minutes)
        short_intervals = sum(1 for interval in intervals if interval < 300)
        burst_score = short_intervals / len(intervals)
        
        return min(burst_score * 2.0, 1.0)  # Amplify and cap at 1.0

    def _detect_circular_referrals(self, user_id: str, referrals: List[Dict]) -> float:
        """Detect circular referral patterns"""
        try:
            referral_ids = [r['user_id'] for r in referrals]
            
            # Check if any referrals also refer back to this user
            circular_count = 0
            for ref_id in referral_ids:
                ref_referrals = self._get_user_referrals(ref_id)
                if user_id in [rr.get('user_id') for rr in ref_referrals]:
                    circular_count += 1
            
            return circular_count / len(referrals) if referrals else 0.0
            
        except Exception:
            return 0.0

    def _get_connected_users(self, user_id: str, depth: int = 2) -> Set[str]:
        """Get users connected within specified depth"""
        if user_id not in self.graph.nodes():
            return {user_id}
        
        connected = {user_id}
        current_level = {user_id}
        
        for _ in range(depth):
            next_level = set()
            for node in current_level:
                # Add neighbors (both directions)
                next_level.update(self.graph.successors(node))
                next_level.update(self.graph.predecessors(node))
            
            next_level -= connected
            connected.update(next_level)
            current_level = next_level
            
            if not current_level:
                break
        
        return connected

    def _calculate_device_similarity(self, user_ids: List[str]) -> float:
        """Calculate device fingerprint similarity among users"""
        try:
            device_fingerprints = []
            for uid in user_ids:
                user_data = self._get_user_network_data(uid)
                if user_data and 'device_fingerprint' in user_data:
                    device_fingerprints.append(user_data['device_fingerprint'])
            
            if len(device_fingerprints) < 2:
                return 0.0
            
            # Calculate similarity using Jaccard similarity on device features
            similarity_scores = []
            for i in range(len(device_fingerprints)):
                for j in range(i+1, len(device_fingerprints)):
                    similarity = self._jaccard_similarity(device_fingerprints[i], device_fingerprints[j])
                    similarity_scores.append(similarity)
            
            return np.mean(similarity_scores) if similarity_scores else 0.0
            
        except Exception:
            return 0.0

    def _jaccard_similarity(self, fp1: str, fp2: str) -> float:
        """Calculate Jaccard similarity between two device fingerprints"""
        # Simple implementation - in production, parse actual fingerprint features
        set1 = set(fp1)
        set2 = set(fp2)
        
        intersection = len(set1.intersection(set2))
        union = len(set1.union(set2))
        
        return intersection / union if union > 0 else 0.0

    def _get_default_features(self) -> Dict[str, float]:
        """Return default feature values for error cases"""
        return {
            'referral_count': 0.0,
            'referral_activation_rate': 0.0,
            'referral_burst_score': 0.0,
            'referral_time_variance': 0.0,
            'circular_referral_score': 0.0,
            'cluster_density': 0.0,
            'cluster_size_score': 0.0,
            'device_similarity_score': 0.0,
            'ip_clustering_score': 0.0,
            'registration_clustering_score': 0.0,
            'betweenness_centrality': 0.0,
            'closeness_centrality': 0.0,
            'degree_centrality': 0.0,
            'eigenvector_centrality': 0.0,
            'pagerank_score': 0.0,
            'activity_time_entropy': 5.0,  # High entropy = human-like
            'weekend_activity_ratio': 0.3,
            'night_activity_ratio': 0.1,
            'activity_periodicity_score': 0.0,
            'burst_activity_score': 0.0,
            'device_consistency_score': 1.0,
            'ip_diversity_score': 1.0,
            'device_sharing_score': 0.0,
            'suspicious_ip_score': 0.0,
            'geolocation_consistency_score': 1.0,
            'network_diversity_score': 1.0,
            'network_engagement_score': 0.5,
            'network_retention_score': 0.7,
            'network_quality_score': 0.5,
            'artificial_growth_score': 0.0
        }

    def _cache_features(self, user_id: str, features: Dict[str, float]) -> None:
        """Cache computed features for performance"""
        try:
            cache_key = f"network_features:{user_id}"
            self.redis_client.setex(cache_key, 3600, str(features))  # Cache for 1 hour
        except Exception as e:
            logger.warning(f"Failed to cache features for {user_id}: {e}")

    # Additional helper methods would be implemented here...
    # (Implementation details for other methods follow similar patterns)

class NetworkFeaturesBatchProcessor:
    """Batch processor for analyzing multiple users efficiently"""
    
    def __init__(self, analyzer: NetworkFeaturesAnalyzer, max_workers: int = 10):
        self.analyzer = analyzer
        self.max_workers = max_workers
    
    async def process_users_batch(self, user_ids: List[str]) -> Dict[str, Dict[str, float]]:
        """Process multiple users in parallel"""
        results = {}
        
        with ThreadPoolExecutor(max_workers=self.max_workers) as executor:
            futures = {
                executor.submit(self.analyzer.analyze_user_network_features, uid): uid 
                for uid in user_ids
            }
            
            for future in futures:
                user_id = futures[future]
                try:
                    features = future.result(timeout=30)
                    results[user_id] = features
                except Exception as e:
                    logger.error(f"Failed to process user {user_id}: {e}")
                    results[user_id] = self.analyzer._get_default_features()
        
        return results

# Example usage and testing
if __name__ == "__main__":
    # Initialize analyzer
    analyzer = NetworkFeaturesAnalyzer()
    
    # Analyze single user
    user_features = analyzer.analyze_user_network_features("user_12345")
    print(f"Network features for user: {user_features}")
    
    # Detect suspicious clusters
    suspicious_clusters = analyzer.detect_suspicious_clusters()
    print(f"Found {len(suspicious_clusters)} suspicious clusters")
    
    # Batch processing example
    batch_processor = NetworkFeaturesBatchProcessor(analyzer)
    batch_results = asyncio.run(
        batch_processor.process_users_batch(["user_1", "user_2", "user_3"])
    )
    print(f"Batch processing completed: {len(batch_results)} users analyzed")
    