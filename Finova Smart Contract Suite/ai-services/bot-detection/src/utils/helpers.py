"""
Finova Network - Bot Detection Helper Utilities
Advanced helper functions for bot detection, pattern analysis, and fraud prevention
Author: Finova Development Team
Version: 3.0.0
Last Updated: July 2025
"""

import hashlib
import hmac
import json
import time
import random
import math
import statistics
import re
from datetime import datetime, timedelta
from typing import Dict, List, Tuple, Optional, Any, Union
from dataclasses import dataclass, asdict
from collections import defaultdict, deque
import numpy as np
import pandas as pd
from scipy import stats
from sklearn.preprocessing import StandardScaler
from sklearn.cluster import DBSCAN
import ipaddress
import user_agents
import logging
import asyncio
from functools import wraps, lru_cache
import redis
import aioredis
from cryptography.fernet import Fernet
import base64

# Configure logging
logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)

@dataclass
class UserBehaviorMetrics:
    """Comprehensive user behavior metrics for bot detection"""
    user_id: str
    session_id: str
    timestamp: float
    click_intervals: List[float]
    mouse_movements: List[Tuple[int, int, float]]
    keyboard_patterns: List[Dict[str, Any]]
    scroll_behavior: List[Dict[str, float]]
    page_focus_times: List[float]
    network_timing: Dict[str, float]
    device_fingerprint: Dict[str, Any]
    geolocation: Optional[Dict[str, float]] = None
    user_agent: Optional[str] = None
    ip_address: Optional[str] = None
    
class SecurityHelper:
    """Security utilities for bot detection and fraud prevention"""
    
    def __init__(self, encryption_key: Optional[str] = None):
        self.encryption_key = encryption_key or Fernet.generate_key()
        self.cipher_suite = Fernet(self.encryption_key)
        
    def hash_data(self, data: str, salt: Optional[str] = None) -> str:
        """Secure hash function with optional salt"""
        if salt is None:
            salt = str(time.time())
        
        combined = f"{data}{salt}".encode('utf-8')
        return hashlib.sha256(combined).hexdigest()
    
    def create_hmac(self, data: str, secret_key: str) -> str:
        """Create HMAC for data integrity verification"""
        return hmac.new(
            secret_key.encode('utf-8'),
            data.encode('utf-8'),
            hashlib.sha256
        ).hexdigest()
    
    def encrypt_sensitive_data(self, data: str) -> str:
        """Encrypt sensitive user data"""
        try:
            encrypted_data = self.cipher_suite.encrypt(data.encode('utf-8'))
            return base64.urlsafe_b64encode(encrypted_data).decode('utf-8')
        except Exception as e:
            logger.error(f"Encryption failed: {e}")
            return ""
    
    def decrypt_sensitive_data(self, encrypted_data: str) -> str:
        """Decrypt sensitive user data"""
        try:
            decoded_data = base64.urlsafe_b64decode(encrypted_data.encode('utf-8'))
            decrypted_data = self.cipher_suite.decrypt(decoded_data)
            return decrypted_data.decode('utf-8')
        except Exception as e:
            logger.error(f"Decryption failed: {e}")
            return ""

class BehaviorAnalyzer:
    """Advanced behavioral pattern analysis for bot detection"""
    
    def __init__(self):
        self.human_click_intervals = (50, 2000)  # Human click intervals in ms
        self.human_typing_speed = (200, 800)    # Human typing speed in ms per char
        self.suspicious_patterns = []
        
    def analyze_click_patterns(self, click_intervals: List[float]) -> Dict[str, float]:
        """Analyze click intervals for bot-like patterns"""
        if len(click_intervals) < 3:
            return {"confidence": 0.5, "human_probability": 0.5}
        
        # Statistical analysis
        mean_interval = statistics.mean(click_intervals)
        std_dev = statistics.stdev(click_intervals) if len(click_intervals) > 1 else 0
        coefficient_of_variation = std_dev / mean_interval if mean_interval > 0 else 0
        
        # Human-like variance check
        human_variance_score = min(1.0, coefficient_of_variation / 0.3)
        
        # Interval distribution analysis
        regular_intervals = sum(1 for i in click_intervals if abs(i - mean_interval) < 10)
        regularity_ratio = regular_intervals / len(click_intervals)
        
        # Bot indicators
        too_regular = regularity_ratio > 0.8  # Too consistent
        too_fast = mean_interval < self.human_click_intervals[0]
        too_slow = mean_interval > self.human_click_intervals[1]
        
        # Calculate human probability
        human_probability = 1.0
        if too_regular:
            human_probability *= 0.2
        if too_fast or too_slow:
            human_probability *= 0.3
        
        human_probability *= human_variance_score
        
        return {
            "mean_interval": mean_interval,
            "std_deviation": std_dev,
            "coefficient_of_variation": coefficient_of_variation,
            "regularity_ratio": regularity_ratio,
            "human_probability": max(0.1, min(1.0, human_probability)),
            "confidence": 0.8 if len(click_intervals) > 10 else 0.5
        }
    
    def analyze_mouse_movements(self, movements: List[Tuple[int, int, float]]) -> Dict[str, float]:
        """Analyze mouse movement patterns"""
        if len(movements) < 5:
            return {"confidence": 0.3, "human_probability": 0.5}
        
        # Extract coordinates and timestamps
        x_coords = [m[0] for m in movements]
        y_coords = [m[1] for m in movements]
        timestamps = [m[2] for m in movements]
        
        # Calculate velocities and accelerations
        velocities = []
        accelerations = []
        
        for i in range(1, len(movements)):
            dx = x_coords[i] - x_coords[i-1]
            dy = y_coords[i] - y_coords[i-1]
            dt = timestamps[i] - timestamps[i-1]
            
            if dt > 0:
                velocity = math.sqrt(dx*dx + dy*dy) / dt
                velocities.append(velocity)
                
                if i > 1 and len(velocities) > 1:
                    dv = velocities[-1] - velocities[-2]
                    acceleration = dv / dt
                    accelerations.append(acceleration)
        
        # Human movement characteristics
        if not velocities:
            return {"confidence": 0.2, "human_probability": 0.5}
        
        avg_velocity = statistics.mean(velocities)
        velocity_variance = statistics.variance(velocities) if len(velocities) > 1 else 0
        
        # Curve analysis (human movements are rarely perfectly straight)
        total_distance = sum(math.sqrt((x_coords[i+1] - x_coords[i])**2 + 
                                     (y_coords[i+1] - y_coords[i])**2) 
                           for i in range(len(movements)-1))
        
        straight_distance = math.sqrt((x_coords[-1] - x_coords[0])**2 + 
                                    (y_coords[-1] - y_coords[0])**2)
        
        curvature_ratio = total_distance / straight_distance if straight_distance > 0 else 1
        
        # Bot indicators
        too_straight = curvature_ratio < 1.1  # Too direct
        too_fast = avg_velocity > 3000  # Unrealistic speed
        no_variance = velocity_variance < 100  # Too consistent
        
        human_probability = 1.0
        if too_straight:
            human_probability *= 0.4
        if too_fast:
            human_probability *= 0.2
        if no_variance:
            human_probability *= 0.3
        
        return {
            "average_velocity": avg_velocity,
            "velocity_variance": velocity_variance,
            "curvature_ratio": curvature_ratio,
            "human_probability": max(0.1, min(1.0, human_probability)),
            "confidence": 0.7 if len(movements) > 20 else 0.4
        }
    
    def analyze_typing_patterns(self, keyboard_events: List[Dict[str, Any]]) -> Dict[str, float]:
        """Analyze keyboard typing patterns"""
        if len(keyboard_events) < 5:
            return {"confidence": 0.3, "human_probability": 0.5}
        
        # Extract timing data
        key_intervals = []
        dwell_times = []
        
        for i, event in enumerate(keyboard_events):
            if event.get('type') == 'keydown' and i > 0:
                prev_event = keyboard_events[i-1]
                if prev_event.get('type') == 'keydown':
                    interval = event['timestamp'] - prev_event['timestamp']
                    key_intervals.append(interval)
            
            if event.get('type') == 'keyup':
                # Find corresponding keydown
                for j in range(i-1, -1, -1):
                    if (keyboard_events[j].get('type') == 'keydown' and 
                        keyboard_events[j].get('key') == event.get('key')):
                        dwell_time = event['timestamp'] - keyboard_events[j]['timestamp']
                        dwell_times.append(dwell_time)
                        break
        
        if not key_intervals:
            return {"confidence": 0.2, "human_probability": 0.5}
        
        # Statistical analysis
        avg_interval = statistics.mean(key_intervals)
        interval_variance = statistics.variance(key_intervals) if len(key_intervals) > 1 else 0
        
        avg_dwell = statistics.mean(dwell_times) if dwell_times else 100
        dwell_variance = statistics.variance(dwell_times) if len(dwell_times) > 1 else 0
        
        # Human typing characteristics
        typing_speed = 1000 / avg_interval if avg_interval > 0 else 0  # chars per second
        
        # Bot indicators
        too_fast = typing_speed > 15  # Unrealistic typing speed
        too_regular = interval_variance < 100  # Too consistent
        weird_dwell = avg_dwell < 50 or avg_dwell > 500  # Unrealistic key press duration
        
        human_probability = 1.0
        if too_fast:
            human_probability *= 0.2
        if too_regular:
            human_probability *= 0.3
        if weird_dwell:
            human_probability *= 0.5
        
        return {
            "typing_speed": typing_speed,
            "average_interval": avg_interval,
            "interval_variance": interval_variance,
            "average_dwell_time": avg_dwell,
            "human_probability": max(0.1, min(1.0, human_probability)),
            "confidence": 0.8 if len(key_intervals) > 20 else 0.5
        }

class NetworkAnalyzer:
    """Network-based bot detection and fraud analysis"""
    
    def __init__(self):
        self.suspicious_networks = set()
        self.known_bot_ips = set()
        self.vpn_ranges = []  # IP ranges known to be VPN/proxy
        
    def analyze_ip_reputation(self, ip_address: str) -> Dict[str, Any]:
        """Analyze IP address reputation and characteristics"""
        try:
            ip_obj = ipaddress.ip_address(ip_address)
            
            analysis = {
                "ip_address": ip_address,
                "is_private": ip_obj.is_private,
                "is_multicast": ip_obj.is_multicast,
                "is_reserved": ip_obj.is_reserved,
                "is_loopback": ip_obj.is_loopback,
                "risk_score": 0.0,
                "reputation": "unknown"
            }
            
            # Check against known bot IPs
            if ip_address in self.known_bot_ips:
                analysis["risk_score"] = 0.9
                analysis["reputation"] = "known_bot"
            
            # Check for suspicious patterns
            if ip_obj.is_private or ip_obj.is_loopback:
                analysis["risk_score"] = 0.1
                analysis["reputation"] = "local"
            elif self._is_datacenter_ip(ip_address):
                analysis["risk_score"] = 0.7
                analysis["reputation"] = "datacenter"
            elif self._is_vpn_ip(ip_address):
                analysis["risk_score"] = 0.5
                analysis["reputation"] = "vpn_proxy"
            
            return analysis
            
        except ValueError:
            return {
                "ip_address": ip_address,
                "error": "invalid_ip",
                "risk_score": 1.0,
                "reputation": "invalid"
            }
    
    def _is_datacenter_ip(self, ip_address: str) -> bool:
        """Check if IP belongs to a datacenter (simplified implementation)"""
        # This would normally check against ASN databases
        datacenter_patterns = [
            r"^23\..*",      # Linode
            r"^104\..*",     # DigitalOcean
            r"^159\.65\..*", # DigitalOcean
            r"^165\.227\..*" # DigitalOcean
        ]
        
        return any(re.match(pattern, ip_address) for pattern in datacenter_patterns)
    
    def _is_vpn_ip(self, ip_address: str) -> bool:
        """Check if IP belongs to VPN/proxy service"""
        # Simplified VPN detection
        vpn_patterns = [
            r"^185\..*",     # Common VPN ranges
            r"^91\..*",      # Common VPN ranges
        ]
        
        return any(re.match(pattern, ip_address) for pattern in vpn_patterns)
    
    def analyze_user_agent(self, user_agent_string: str) -> Dict[str, Any]:
        """Analyze user agent for bot indicators"""
        if not user_agent_string:
            return {
                "risk_score": 0.8,
                "is_bot": True,
                "reason": "missing_user_agent"
            }
        
        try:
            ua = user_agents.parse(user_agent_string)
            
            analysis = {
                "browser": ua.browser.family,
                "browser_version": ua.browser.version_string,
                "os": ua.os.family,
                "os_version": ua.os.version_string,
                "device": ua.device.family,
                "is_mobile": ua.is_mobile,
                "is_tablet": ua.is_tablet,
                "is_pc": ua.is_pc,
                "is_bot": ua.is_bot,
                "risk_score": 0.0
            }
            
            # Bot indicators
            if ua.is_bot:
                analysis["risk_score"] = 0.9
                analysis["bot_reason"] = "identified_as_bot"
            elif "headless" in user_agent_string.lower():
                analysis["risk_score"] = 0.8
                analysis["bot_reason"] = "headless_browser"
            elif "selenium" in user_agent_string.lower():
                analysis["risk_score"] = 0.9
                analysis["bot_reason"] = "automation_tool"
            elif "phantom" in user_agent_string.lower():
                analysis["risk_score"] = 0.9
                analysis["bot_reason"] = "phantom_browser"
            
            # Unusual patterns
            if not ua.browser.family or ua.browser.family == "Other":
                analysis["risk_score"] += 0.3
            
            if not ua.os.family or ua.os.family == "Other":
                analysis["risk_score"] += 0.3
            
            analysis["risk_score"] = min(1.0, analysis["risk_score"])
            
            return analysis
            
        except Exception as e:
            logger.error(f"User agent parsing failed: {e}")
            return {
                "error": "parsing_failed",
                "risk_score": 0.6,
                "is_bot": False
            }

class TemporalAnalyzer:
    """Time-based pattern analysis for bot detection"""
    
    def __init__(self):
        self.activity_windows = defaultdict(list)
        self.circadian_patterns = {}
        
    def analyze_activity_patterns(self, user_id: str, activities: List[Dict[str, Any]]) -> Dict[str, float]:
        """Analyze temporal activity patterns for bot detection"""
        if len(activities) < 10:
            return {"confidence": 0.3, "human_probability": 0.5}
        
        # Extract timestamps
        timestamps = [activity['timestamp'] for activity in activities]
        timestamps.sort()
        
        # Convert to datetime objects
        datetimes = [datetime.fromtimestamp(ts) for ts in timestamps]
        
        # Analyze time intervals
        intervals = [(datetimes[i+1] - datetimes[i]).total_seconds() 
                    for i in range(len(datetimes)-1)]
        
        # Statistical analysis
        if not intervals:
            return {"confidence": 0.2, "human_probability": 0.5}
        
        avg_interval = statistics.mean(intervals)
        interval_variance = statistics.variance(intervals) if len(intervals) > 1 else 0
        
        # Circadian rhythm analysis
        hours = [dt.hour for dt in datetimes]
        hour_distribution = np.histogram(hours, bins=24, range=(0, 24))[0]
        
        # Human activity patterns (more active during day, less at night)
        night_activity = sum(hour_distribution[22:24]) + sum(hour_distribution[0:6])
        day_activity = sum(hour_distribution[6:22])
        night_ratio = night_activity / (night_activity + day_activity) if (night_activity + day_activity) > 0 else 0
        
        # Bot indicators
        too_regular = interval_variance < 3600  # Too consistent (1 hour variance)
        no_sleep = night_ratio > 0.4  # Too much night activity
        constant_activity = len(set(hours)) > 20  # Active almost all hours
        
        # Weekend vs weekday analysis
        weekdays = [dt.weekday() for dt in datetimes]
        weekday_count = sum(1 for wd in weekdays if wd < 5)
        weekend_count = sum(1 for wd in weekdays if wd >= 5)
        
        # Humans typically less active on weekends
        weekend_ratio = weekend_count / len(weekdays) if weekdays else 0
        unusual_weekend_pattern = weekend_ratio > 0.4
        
        human_probability = 1.0
        if too_regular:
            human_probability *= 0.3
        if no_sleep:
            human_probability *= 0.4
        if constant_activity:
            human_probability *= 0.2
        if unusual_weekend_pattern:
            human_probability *= 0.6
        
        return {
            "average_interval": avg_interval,
            "interval_variance": interval_variance,
            "night_activity_ratio": night_ratio,
            "weekend_activity_ratio": weekend_ratio,
            "unique_hours_active": len(set(hours)),
            "human_probability": max(0.1, min(1.0, human_probability)),
            "confidence": 0.8 if len(activities) > 50 else 0.5
        }
    
    def detect_burst_patterns(self, timestamps: List[float], window_size: int = 3600) -> Dict[str, Any]:
        """Detect suspicious burst patterns in activity"""
        if len(timestamps) < 5:
            return {"burst_detected": False, "confidence": 0.3}
        
        timestamps.sort()
        
        # Count activities in sliding windows
        burst_counts = []
        for i, ts in enumerate(timestamps):
            window_start = ts
            window_end = ts + window_size
            
            count = sum(1 for t in timestamps[i:] if window_start <= t <= window_end)
            burst_counts.append(count)
        
        if not burst_counts:
            return {"burst_detected": False, "confidence": 0.2}
        
        max_burst = max(burst_counts)
        avg_activity = statistics.mean(burst_counts)
        
        # Detect unusual bursts (more than 3x average activity)
        burst_threshold = max(10, avg_activity * 3)
        burst_detected = max_burst > burst_threshold
        
        return {
            "burst_detected": burst_detected,
            "max_burst_count": max_burst,
            "average_activity": avg_activity,
            "burst_threshold": burst_threshold,
            "burst_ratio": max_burst / avg_activity if avg_activity > 0 else 0,
            "confidence": 0.7 if len(timestamps) > 20 else 0.4
        }

class DeviceAnalyzer:
    """Device fingerprinting and analysis for bot detection"""
    
    def __init__(self):
        self.known_devices = {}
        self.suspicious_fingerprints = set()
        
    def analyze_device_fingerprint(self, fingerprint: Dict[str, Any]) -> Dict[str, float]:
        """Analyze device fingerprint for bot indicators"""
        if not fingerprint:
            return {"risk_score": 0.8, "confidence": 0.3}
        
        risk_indicators = []
        
        # Screen resolution analysis
        screen_width = fingerprint.get('screen_width', 0)
        screen_height = fingerprint.get('screen_height', 0)
        
        if screen_width == 0 or screen_height == 0:
            risk_indicators.append("missing_screen_info")
        elif self._is_unusual_resolution(screen_width, screen_height):
            risk_indicators.append("unusual_resolution")
        
        # Timezone analysis
        timezone_offset = fingerprint.get('timezone_offset')
        if timezone_offset is None:
            risk_indicators.append("missing_timezone")
        
        # Plugin/extension analysis
        plugins = fingerprint.get('plugins', [])
        if not plugins:
            risk_indicators.append("no_plugins")
        elif self._has_automation_plugins(plugins):
            risk_indicators.append("automation_plugins")
        
        # Canvas fingerprinting
        canvas_hash = fingerprint.get('canvas_hash')
        if not canvas_hash:
            risk_indicators.append("missing_canvas")
        elif canvas_hash in self.suspicious_fingerprints:
            risk_indicators.append("known_bot_canvas")
        
        # WebGL analysis
        webgl_vendor = fingerprint.get('webgl_vendor', '')
        webgl_renderer = fingerprint.get('webgl_renderer', '')
        
        if not webgl_vendor or not webgl_renderer:
            risk_indicators.append("missing_webgl")
        elif 'headless' in webgl_renderer.lower():
            risk_indicators.append("headless_webgl")
        
        # Language and locale consistency
        languages = fingerprint.get('languages', [])
        locale = fingerprint.get('locale', '')
        
        if not languages:
            risk_indicators.append("missing_languages")
        elif not self._is_consistent_locale(languages, locale):
            risk_indicators.append("inconsistent_locale")
        
        # Calculate risk score
        risk_score = min(1.0, len(risk_indicators) * 0.15)
        
        return {
            "risk_score": risk_score,
            "risk_indicators": risk_indicators,
            "fingerprint_hash": self._hash_fingerprint(fingerprint),
            "confidence": 0.8 if len(fingerprint) > 10 else 0.4
        }
    
    def _is_unusual_resolution(self, width: int, height: int) -> bool:
        """Check if screen resolution is unusual (bot indicator)"""
        # Common resolutions
        common_resolutions = {
            (1920, 1080), (1366, 768), (1536, 864), (1440, 900),
            (1680, 1050), (1280, 1024), (1024, 768), (1280, 800)
        }
        
        return (width, height) not in common_resolutions and width * height < 500000
    
    def _has_automation_plugins(self, plugins: List[str]) -> bool:
        """Check for automation-related plugins"""
        automation_keywords = ['selenium', 'webdriver', 'phantom', 'headless', 'chromedriver']
        plugin_text = ' '.join(plugins).lower()
        
        return any(keyword in plugin_text for keyword in automation_keywords)
    
    def _is_consistent_locale(self, languages: List[str], locale: str) -> bool:
        """Check if languages and locale are consistent"""
        if not languages or not locale:
            return False
        
        primary_lang = languages[0][:2] if languages else ""
        locale_lang = locale[:2] if locale else ""
        
        return primary_lang == locale_lang
    
    def _hash_fingerprint(self, fingerprint: Dict[str, Any]) -> str:
        """Create hash of device fingerprint"""
        # Sort keys for consistent hashing
        sorted_items = sorted(fingerprint.items())
        fingerprint_str = json.dumps(sorted_items, sort_keys=True)
        
        return hashlib.sha256(fingerprint_str.encode('utf-8')).hexdigest()

class SocialGraphAnalyzer:
    """Social network graph analysis for bot detection"""
    
    def __init__(self):
        self.connection_graphs = defaultdict(set)
        self.suspicious_clusters = set()
        
    def analyze_referral_network(self, user_id: str, referral_data: Dict[str, Any]) -> Dict[str, float]:
        """Analyze referral network for bot farm indicators"""
        direct_referrals = referral_data.get('direct_referrals', [])
        referral_activities = referral_data.get('referral_activities', [])
        
        if len(direct_referrals) < 3:
            return {"confidence": 0.3, "bot_farm_probability": 0.1}
        
        # Analyze referral timing patterns
        registration_times = [ref.get('registration_time', 0) for ref in direct_referrals]
        registration_times.sort()
        
        # Check for suspicious timing clusters
        time_clusters = self._detect_time_clusters(registration_times)
        
        # Analyze activity correlation
        activity_correlation = self._calculate_activity_correlation(referral_activities)
        
        # Check for identical behavior patterns
        behavior_similarity = self._analyze_behavior_similarity(direct_referrals)
        
        # Geographic clustering analysis
        geo_clustering = self._analyze_geographic_clustering(direct_referrals)
        
        # Calculate bot farm probability
        bot_farm_indicators = []
        
        if len(time_clusters) > len(direct_referrals) * 0.3:
            bot_farm_indicators.append("suspicious_timing")
        
        if activity_correlation > 0.8:
            bot_farm_indicators.append("correlated_activity")
        
        if behavior_similarity > 0.7:
            bot_farm_indicators.append("similar_behavior")
        
        if geo_clustering > 0.8:
            bot_farm_indicators.append("geographic_clustering")
        
        bot_farm_probability = min(1.0, len(bot_farm_indicators) * 0.25)
        
        return {
            "bot_farm_probability": bot_farm_probability,
            "bot_farm_indicators": bot_farm_indicators,
            "time_cluster_count": len(time_clusters),
            "activity_correlation": activity_correlation,
            "behavior_similarity": behavior_similarity,
            "geographic_clustering": geo_clustering,
            "confidence": 0.8 if len(direct_referrals) > 10 else 0.5
        }
    
    def _detect_time_clusters(self, timestamps: List[float], window: int = 3600) -> List[List[float]]:
        """Detect clusters of registrations within time windows"""
        if len(timestamps) < 2:
            return []
        
        clusters = []
        current_cluster = [timestamps[0]]
        
        for i in range(1, len(timestamps)):
            if timestamps[i] - timestamps[i-1] <= window:
                current_cluster.append(timestamps[i])
            else:
                if len(current_cluster) > 2:  # Minimum cluster size
                    clusters.append(current_cluster)
                current_cluster = [timestamps[i]]
        
        if len(current_cluster) > 2:
            clusters.append(current_cluster)
        
        return clusters
    
    def _calculate_activity_correlation(self, activities: List[Dict[str, Any]]) -> float:
        """Calculate correlation between referral activities"""
        if len(activities) < 2:
            return 0.0
        
        # Group activities by user
        user_activities = defaultdict(list)
        for activity in activities:
            user_id = activity.get('user_id')
            if user_id:
                user_activities[user_id].append(activity['timestamp'])
        
        if len(user_activities) < 2:
            return 0.0
        
        # Calculate correlation coefficients
        correlations = []
        users = list(user_activities.keys())
        
        for i in range(len(users)):
            for j in range(i+1, len(users)):
                user1_times = user_activities[users[i]]
                user2_times = user_activities[users[j]]
                
                # Simplified correlation calculation
                if len(user1_times) > 1 and len(user2_times) > 1:
                    try:
                        correlation = np.corrcoef(user1_times[:min(len(user1_times), len(user2_times))],
                                               user2_times[:min(len(user1_times), len(user2_times))])[0, 1]
                        if not np.isnan(correlation):
                            correlations.append(abs(correlation))
                    except Exception:
                        continue
        
        return statistics.mean(correlations) if correlations else 0.0
    
    def _analyze_behavior_similarity(self, referrals: List[Dict[str, Any]]) -> float:
        """Analyze similarity in behavior patterns"""
        if len(referrals) < 2:
            return 0.0
        
        # Extract behavior features
        behavior_vectors = []
        for referral in referrals:
            vector = [
                referral.get('activity_frequency', 0),
                referral.get('avg_session_duration', 0),
                referral.get('platform_diversity', 0),
                referral.get('content_quality_score', 0),
                referral.get('social_engagement_rate', 0)
            ]
            behavior_vectors.append(vector)
        
        # Calculate pairwise similarities
        similarities = []
        for i in range(len(behavior_vectors)):
            for j in range(i+1, len(behavior_vectors)):
                try:
                    # Cosine similarity
                    vec1 = np.array(behavior_vectors[i])
                    vec2 = np.array(behavior_vectors[j])
                    
                    dot_product = np.dot(vec1, vec2)
                    magnitude1 = np.linalg.norm(vec1)
                    magnitude2 = np.linalg.norm(vec2)
                    
                    if magnitude1 > 0 and magnitude2 > 0:
                        similarity = dot_product / (magnitude1 * magnitude2)
                        similarities.append(similarity)
                except Exception:
                    continue
        
        return statistics.mean(similarities) if similarities else 0.0
    
    def _analyze_geographic_clustering(self, referrals: List[Dict[str, Any]]) -> float:
        """Analyze geographic clustering of referrals"""
        locations = []
        for referral in referrals:
            lat = referral.get('latitude')
            lon = referral.get('longitude')
            if lat is not None and lon is not None:
                locations.append((lat, lon))
        
        if len(locations) < 3:
            return 0.0
        
        # Calculate pairwise distances
        distances = []
        for i in range(len(locations)):
            for j in range(i+1, len(locations)):
                dist = self._haversine_distance(locations[i], locations[j])
                distances.append(dist)
        
        if not distances:
            return 0.0
        
        # Check for clustering (many short distances)
        avg_distance = statistics.mean(distances)
        short_distances = sum(1 for d in distances if d < avg_distance * 0.1)
        clustering_ratio = short_distances / len(distances)
        
        return clustering_ratio
    
    def _haversine_distance(self, coord1: Tuple[float, float], coord2: Tuple[float, float]) -> float:
        """Calculate distance between two coordinates in kilometers"""
        lat1, lon1 = coord1
        lat2, lon2 = coord2
        
        # Convert to radians
        lat1, lon1, lat2, lon2 = map(math.radians, [lat1, lon1, lat2, lon2])
        
        # Haversine formula
        dlat = lat2 - lat1
        dlon = lon2 - lon1
        a = math.sin(dlat/2)**2 + math.cos(lat1) * math.cos(lat2) * math.sin(dlon/2)**2
        c = 2 * math.asin(math.sqrt(a))
        
        # Earth radius in kilometers
        r = 6371
        
        return c * r

class ContentQualityAnalyzer:
    """Content quality analysis for detecting low-effort or generated content"""
    
    def __init__(self):
        self.spam_patterns = [
            r'(buy|sale|discount|offer|deal|free|win|earn|money|cash|prize)',
            r'(http[s]?://(?:[a-zA-Z]|[0-9]|[$-_@.&+]|[!*\\(\\),]|(?:%[0-9a-fA-F][0-9a-fA-F]))+)',
            r'([A-Z]{3,})',  # Excessive caps
            r'(.)\1{4,}',    # Repeated characters
        ]
        
    def analyze_content_quality(self, content: str, metadata: Dict[str, Any] = None) -> Dict[str, float]:
        """Analyze content quality and detect spam/bot-generated content"""
        if not content:
            return {"quality_score": 0.0, "spam_probability": 1.0, "confidence": 0.9}
        
        analysis = {
            "content_length": len(content),
            "word_count": len(content.split()),
            "unique_words": len(set(content.lower().split())),
            "spam_patterns": 0,
            "readability_score": 0.0,
            "originality_score": 1.0,
            "engagement_potential": 0.5
        }
        
        # Spam pattern detection
        spam_matches = 0
        for pattern in self.spam_patterns:
            matches = re.findall(pattern, content, re.IGNORECASE)
            spam_matches += len(matches)
        
        analysis["spam_patterns"] = spam_matches
        
        # Basic readability (simplified Flesch score)
        sentences = content.count('.') + content.count('!') + content.count('?')
        if sentences > 0 and analysis["word_count"] > 0:
            avg_sentence_length = analysis["word_count"] / sentences
            syllable_count = self._estimate_syllables(content)
            avg_syllables_per_word = syllable_count / analysis["word_count"]
            
            # Simplified Flesch Reading Ease
            readability = 206.835 - (1.015 * avg_sentence_length) - (84.6 * avg_syllables_per_word)
            analysis["readability_score"] = max(0, min(100, readability)) / 100
        
        # Vocabulary diversity
        if analysis["word_count"] > 0:
            vocab_diversity = analysis["unique_words"] / analysis["word_count"]
            analysis["vocabulary_diversity"] = vocab_diversity
        else:
            analysis["vocabulary_diversity"] = 0
        
        # Bot indicators
        bot_indicators = []
        
        if spam_matches > 2:
            bot_indicators.append("spam_patterns")
        
        if analysis["vocabulary_diversity"] < 0.3:
            bot_indicators.append("low_vocabulary")
        
        if len(content) < 10:
            bot_indicators.append("too_short")
        
        if re.search(r'(.)\1{5,}', content):  # Excessive repetition
            bot_indicators.append("excessive_repetition")
        
        if content.count(' ') == 0 and len(content) > 50:  # No spaces in long text
            bot_indicators.append("no_spaces")
        
        # Calculate quality and spam scores
        quality_score = 1.0
        if bot_indicators:
            quality_score -= len(bot_indicators) * 0.2
        
        quality_score = max(0.0, min(1.0, quality_score))
        spam_probability = 1.0 - quality_score
        
        analysis.update({
            "quality_score": quality_score,
            "spam_probability": spam_probability,
            "bot_indicators": bot_indicators,
            "confidence": 0.8 if len(content) > 100 else 0.5
        })
        
        return analysis
    
    def _estimate_syllables(self, text: str) -> int:
        """Estimate syllable count (simplified)"""
        vowels = 'aeiouyAEIOUY'
        syllables = 0
        prev_was_vowel = False
        
        for char in text:
            if char in vowels:
                if not prev_was_vowel:
                    syllables += 1
                prev_was_vowel = True
            else:
                prev_was_vowel = False
        
        # Adjust for silent e
        if text.endswith('e') or text.endswith('E'):
            syllables -= 1
        
        return max(1, syllables)

class FinovaScoreCalculator:
    """Main score calculation for Finova bot detection system"""
    
    def __init__(self):
        self.behavior_analyzer = BehaviorAnalyzer()
        self.network_analyzer = NetworkAnalyzer()
        self.temporal_analyzer = TemporalAnalyzer()
        self.device_analyzer = DeviceAnalyzer()
        self.social_graph_analyzer = SocialGraphAnalyzer()
        self.content_analyzer = ContentQualityAnalyzer()
        
        # Weights for different analysis components
        self.weights = {
            "behavior": 0.25,
            "network": 0.15,
            "temporal": 0.20,
            "device": 0.15,
            "social_graph": 0.15,
            "content_quality": 0.10
        }
    
    def calculate_human_probability(self, user_data: UserBehaviorMetrics, 
                                   additional_data: Dict[str, Any] = None) -> Dict[str, Any]:
        """Calculate comprehensive human probability score"""
        if additional_data is None:
            additional_data = {}
        
        scores = {}
        confidences = {}
        
        # Behavior analysis
        if user_data.click_intervals:
            click_analysis = self.behavior_analyzer.analyze_click_patterns(user_data.click_intervals)
            scores["behavior_clicks"] = click_analysis["human_probability"]
            confidences["behavior_clicks"] = click_analysis["confidence"]
        
        if user_data.mouse_movements:
            mouse_analysis = self.behavior_analyzer.analyze_mouse_movements(user_data.mouse_movements)
            scores["behavior_mouse"] = mouse_analysis["human_probability"]
            confidences["behavior_mouse"] = mouse_analysis["confidence"]
        
        if user_data.keyboard_patterns:
            typing_analysis = self.behavior_analyzer.analyze_typing_patterns(user_data.keyboard_patterns)
            scores["behavior_typing"] = typing_analysis["human_probability"]
            confidences["behavior_typing"] = typing_analysis["confidence"]
        
        # Network analysis
        if user_data.ip_address:
            ip_analysis = self.network_analyzer.analyze_ip_reputation(user_data.ip_address)
            scores["network_ip"] = 1.0 - ip_analysis["risk_score"]
            confidences["network_ip"] = 0.8
        
        if user_data.user_agent:
            ua_analysis = self.network_analyzer.analyze_user_agent(user_data.user_agent)
            scores["network_ua"] = 1.0 - ua_analysis["risk_score"]
            confidences["network_ua"] = 0.7
        
        # Temporal analysis
        activities = additional_data.get("activities", [])
        if activities:
            temporal_analysis = self.temporal_analyzer.analyze_activity_patterns(
                user_data.user_id, activities
            )
            scores["temporal"] = temporal_analysis["human_probability"]
            confidences["temporal"] = temporal_analysis["confidence"]
        
        # Device analysis
        if user_data.device_fingerprint:
            device_analysis = self.device_analyzer.analyze_device_fingerprint(
                user_data.device_fingerprint
            )
            scores["device"] = 1.0 - device_analysis["risk_score"]
            confidences["device"] = device_analysis["confidence"]
        
        # Social graph analysis
        referral_data = additional_data.get("referral_data")
        if referral_data:
            social_analysis = self.social_graph_analyzer.analyze_referral_network(
                user_data.user_id, referral_data
            )
            scores["social_graph"] = 1.0 - social_analysis["bot_farm_probability"]
            confidences["social_graph"] = social_analysis["confidence"]
        
        # Content quality analysis
        content_samples = additional_data.get("content_samples", [])
        if content_samples:
            content_scores = []
            for content in content_samples:
                content_analysis = self.content_analyzer.analyze_content_quality(content)
                content_scores.append(content_analysis["quality_score"])
            
            if content_scores:
                scores["content_quality"] = statistics.mean(content_scores)
                confidences["content_quality"] = 0.7
        
        # Calculate weighted final score
        final_score = 0.0
        total_weight = 0.0
        
        for category, score in scores.items():
            base_category = category.split('_')[0]  # Get base category name
            weight = self.weights.get(base_category, 0.1)
            confidence = confidences.get(category, 0.5)
            
            # Weight by confidence
            effective_weight = weight * confidence
            final_score += score * effective_weight
            total_weight += effective_weight
        
        if total_weight > 0:
            final_score /= total_weight
        else:
            final_score = 0.5  # Default neutral score
        
        # Calculate overall confidence
        overall_confidence = statistics.mean(confidences.values()) if confidences else 0.3
        
        # Risk categorization
        if final_score >= 0.8:
            risk_level = "low"
        elif final_score >= 0.6:
            risk_level = "medium"
        elif final_score >= 0.4:
            risk_level = "high"
        else:
            risk_level = "critical"
        
        return {
            "human_probability": final_score,
            "risk_level": risk_level,
            "confidence": overall_confidence,
            "component_scores": scores,
            "component_confidences": confidences,
            "analysis_timestamp": time.time(),
            "user_id": user_data.user_id,
            "session_id": user_data.session_id
        }

# Utility decorators and helper functions
def rate_limit(max_calls: int, window_seconds: int = 3600):
    """Rate limiting decorator for API calls"""
    calls = defaultdict(deque)
    
    def decorator(func):
        @wraps(func)
        async def wrapper(*args, **kwargs):
            now = time.time()
            client_id = kwargs.get('client_id', 'anonymous')
            
            # Clean old calls
            while calls[client_id] and calls[client_id][0] < now - window_seconds:
                calls[client_id].popleft()
            
            # Check rate limit
            if len(calls[client_id]) >= max_calls:
                raise Exception(f"Rate limit exceeded: {max_calls} calls per {window_seconds}s")
            
            calls[client_id].append(now)
            return await func(*args, **kwargs)
        
        return wrapper
    return decorator

@lru_cache(maxsize=1000)
def get_country_from_ip(ip_address: str) -> str:
    """Get country code from IP address (cached)"""
    # This would normally use a GeoIP database
    # Simplified implementation for demo
    if ip_address.startswith('192.168.') or ip_address.startswith('10.') or ip_address.startswith('127.'):
        return "LOCAL"
    
    # Mock country detection based on IP ranges
    ip_parts = ip_address.split('.')
    if len(ip_parts) == 4:
        first_octet = int(ip_parts[0])
        if 1 <= first_octet <= 50:
            return "US"
        elif 51 <= first_octet <= 100:
            return "EU"
        elif 101 <= first_octet <= 150:
            return "ASIA"
        else:
            return "OTHER"
    
    return "UNKNOWN"

async def redis_cache_get(redis_client: aioredis.Redis, key: str) -> Optional[Dict[str, Any]]:
    """Get data from Redis cache"""
    try:
        data = await redis_client.get(key)
        if data:
            return json.loads(data)
    except Exception as e:
        logger.error(f"Redis get error: {e}")
    
    return None

async def redis_cache_set(redis_client: aioredis.Redis, key: str, data: Dict[str, Any], 
                         expire_seconds: int = 3600) -> bool:
    """Set data in Redis cache"""
    try:
        await redis_client.setex(key, expire_seconds, json.dumps(data))
        return True
    except Exception as e:
        logger.error(f"Redis set error: {e}")
        return False

def create_user_risk_key(user_id: str, analysis_type: str = "general") -> str:
    """Create standardized cache key for user risk data"""
    return f"finova:bot_detection:{analysis_type}:{user_id}"

def sanitize_input(data: str, max_length: int = 10000) -> str:
    """Sanitize input data for security"""
    if not isinstance(data, str):
        data = str(data)
    
    # Truncate if too long
    if len(data) > max_length:
        data = data[:max_length]
    
    # Remove potential script tags
    data = re.sub(r'<script.*?</script>', '', data, flags=re.IGNORECASE | re.DOTALL)
    
    # Remove other dangerous patterns
    dangerous_patterns = [
        r'javascript:', r'data:', r'vbscript:', r'on\w+='
    ]
    
    for pattern in dangerous_patterns:
        data = re.sub(pattern, '', data, flags=re.IGNORECASE)
    
    return data.strip()

def validate_user_metrics(metrics: UserBehaviorMetrics) -> List[str]:
    """Validate user behavior metrics for completeness and correctness"""
    errors = []
    
    if not metrics.user_id:
        errors.append("Missing user_id")
    
    if not metrics.session_id:
        errors.append("Missing session_id")
    
    if metrics.timestamp <= 0:
        errors.append("Invalid timestamp")
    
    if metrics.click_intervals and any(interval < 0 for interval in metrics.click_intervals):
        errors.append("Negative click intervals")
    
    if metrics.mouse_movements:
        for movement in metrics.mouse_movements:
            if len(movement) != 3:
                errors.append("Invalid mouse movement format")
                break
            if movement[2] < 0:  # timestamp
                errors.append("Invalid mouse movement timestamp")
                break
    
    return errors

# Configuration and constants
class BotDetectionConfig:
    """Configuration constants for bot detection"""
    
    # Scoring thresholds
    HUMAN_PROBABILITY_THRESHOLD = 0.6
    HIGH_RISK_THRESHOLD = 0.4
    CRITICAL_RISK_THRESHOLD = 0.2
    
    # Analysis parameters
    MIN_ACTIVITIES_FOR_TEMPORAL = 10
    MIN_REFERRALS_FOR_SOCIAL = 3
    MIN_CONTENT_SAMPLES = 2
    
    # Cache settings
    CACHE_EXPIRY_SECONDS = 3600  # 1 hour
    MAX_CACHE_ENTRIES = 10000
    
    # Rate limiting
    MAX_ANALYSIS_CALLS_PER_HOUR = 1000
    MAX_ANALYSIS_CALLS_PER_MINUTE = 50
    
    # Security
    MAX_INPUT_LENGTH = 50000
    ALLOWED_FILE_TYPES = ['json', 'csv', 'txt']
    
    # Model parameters
    BEHAVIOR_WEIGHT = 0.25
    NETWORK_WEIGHT = 0.15
    TEMPORAL_WEIGHT = 0.20
    DEVICE_WEIGHT = 0.15
    SOCIAL_GRAPH_WEIGHT = 0.15
    CONTENT_QUALITY_WEIGHT = 0.10

# Export main classes and functions
__all__ = [
    'UserBehaviorMetrics',
    'SecurityHelper',
    'BehaviorAnalyzer',
    'NetworkAnalyzer',
    'TemporalAnalyzer',
    'DeviceAnalyzer',
    'SocialGraphAnalyzer',
    'ContentQualityAnalyzer',
    'FinovaScoreCalculator',
    'BotDetectionConfig',
    'rate_limit',
    'get_country_from_ip',
    'redis_cache_get',
    'redis_cache_set',
    'create_user_risk_key',
    'sanitize_input',
    'validate_user_metrics'
]
