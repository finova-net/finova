"""
Finova Network - Device Features Extraction for Bot Detection
Advanced device fingerprinting and hardware authentication system
"""

import asyncio
import hashlib
import json
import logging
import platform
import time
from datetime import datetime, timedelta
from typing import Dict, List, Optional, Tuple, Any
from dataclasses import dataclass, asdict
import numpy as np
from sklearn.ensemble import IsolationForest
import redis
import ipaddress
import requests
from cryptography.hazmat.primitives import hashes
from cryptography.hazmat.primitives.kdf.pbkdf2 import PBKDF2HMAC
import base64

logger = logging.getLogger(__name__)

@dataclass
class DeviceProfile:
    """Complete device profile for fingerprinting"""
    device_id: str
    user_agent: str
    screen_resolution: str
    timezone: str
    language: str
    platform_info: str
    hardware_specs: Dict
    network_info: Dict
    browser_features: Dict
    mobile_features: Dict
    biometric_hash: Optional[str]
    device_trust_score: float
    created_at: datetime
    last_seen: datetime

@dataclass
class DeviceMetrics:
    """Real-time device behavior metrics"""
    cpu_usage_pattern: List[float]
    memory_usage_pattern: List[float]
    network_latency: float
    click_timing_variance: float
    keystroke_dynamics: Dict
    touch_pressure_patterns: List[float]
    gyroscope_data: List[float]
    accelerometer_data: List[float]
    battery_level_changes: List[Tuple[datetime, int]]
    app_switching_frequency: int

class DeviceFeatureExtractor:
    """
    Enterprise-grade device feature extraction for bot detection
    Combines hardware fingerprinting with behavioral analysis
    """
    
    def __init__(self, redis_client: redis.Redis, config: Dict):
        self.redis = redis_client
        self.config = config
        self.isolation_forest = IsolationForest(
            contamination=0.1, 
            random_state=42,
            n_estimators=100
        )
        self.device_cache = {}
        self.suspicious_patterns = self._load_suspicious_patterns()
        
    def _load_suspicious_patterns(self) -> Dict:
        """Load known bot device patterns"""
        return {
            'vm_indicators': [
                'virtualbox', 'vmware', 'qemu', 'xen', 'hyper-v',
                'parallels', 'vbox', 'kvm'
            ],
            'emulator_indicators': [
                'android_emulator', 'genymotion', 'bluestacks',
                'nox', 'memu', 'ldplayer', 'simulator'
            ],
            'automation_tools': [
                'selenium', 'puppeteer', 'playwright', 'chromedriver',
                'geckodriver', 'appium', 'cypress'
            ],
            'suspicious_resolutions': [
                '800x600', '1024x768', '1280x720',  # Common bot resolutions
                '320x240', '640x480'
            ],
            'bot_user_agents': [
                'headlesschrome', 'phantomjs', 'htmlunit',
                'bot', 'crawler', 'spider', 'scraper'
            ]
        }

    async def extract_device_features(self, request_data: Dict) -> DeviceProfile:
        """
        Extract comprehensive device features from request
        
        Args:
            request_data: Raw request data including headers, device info
            
        Returns:
            DeviceProfile: Complete device fingerprint
        """
        try:
            # Generate unique device ID
            device_id = await self._generate_device_id(request_data)
            
            # Extract basic device info
            basic_info = await self._extract_basic_info(request_data)
            
            # Extract hardware specifications
            hardware_specs = await self._extract_hardware_specs(request_data)
            
            # Extract network information
            network_info = await self._extract_network_info(request_data)
            
            # Extract browser/app features
            browser_features = await self._extract_browser_features(request_data)
            
            # Extract mobile-specific features
            mobile_features = await self._extract_mobile_features(request_data)
            
            # Generate biometric hash if available
            biometric_hash = await self._generate_biometric_hash(request_data)
            
            # Calculate device trust score
            trust_score = await self._calculate_device_trust_score({
                'basic_info': basic_info,
                'hardware_specs': hardware_specs,
                'network_info': network_info,
                'browser_features': browser_features,
                'mobile_features': mobile_features
            })
            
            profile = DeviceProfile(
                device_id=device_id,
                user_agent=basic_info.get('user_agent', ''),
                screen_resolution=basic_info.get('screen_resolution', ''),
                timezone=basic_info.get('timezone', ''),
                language=basic_info.get('language', ''),
                platform_info=basic_info.get('platform_info', ''),
                hardware_specs=hardware_specs,
                network_info=network_info,
                browser_features=browser_features,
                mobile_features=mobile_features,
                biometric_hash=biometric_hash,
                device_trust_score=trust_score,
                created_at=datetime.now(),
                last_seen=datetime.now()
            )
            
            # Cache device profile
            await self._cache_device_profile(profile)
            
            return profile
            
        except Exception as e:
            logger.error(f"Error extracting device features: {e}")
            raise

    async def _generate_device_id(self, request_data: Dict) -> str:
        """Generate unique device identifier"""
        try:
            # Collect identifying information
            identifiers = [
                request_data.get('user_agent', ''),
                request_data.get('screen_resolution', ''),
                request_data.get('timezone', ''),
                request_data.get('language', ''),
                request_data.get('platform', ''),
                str(request_data.get('hardware_concurrency', 0)),
                str(request_data.get('device_memory', 0)),
                request_data.get('canvas_fingerprint', ''),
                request_data.get('webgl_fingerprint', ''),
                request_data.get('audio_fingerprint', ''),
            ]
            
            # Create hash of combined identifiers
            combined = '|'.join(filter(None, identifiers))
            device_hash = hashlib.sha256(combined.encode()).hexdigest()
            
            return f"fin_dev_{device_hash[:16]}"
            
        except Exception as e:
            logger.error(f"Error generating device ID: {e}")
            return f"fin_dev_unknown_{int(time.time())}"

    async def _extract_basic_info(self, request_data: Dict) -> Dict:
        """Extract basic device information"""
        return {
            'user_agent': request_data.get('user_agent', ''),
            'screen_resolution': f"{request_data.get('screen_width', 0)}x{request_data.get('screen_height', 0)}",
            'color_depth': request_data.get('color_depth', 0),
            'pixel_ratio': request_data.get('pixel_ratio', 1.0),
            'timezone': request_data.get('timezone', ''),
            'language': request_data.get('language', ''),
            'platform_info': request_data.get('platform', ''),
            'cookie_enabled': request_data.get('cookie_enabled', True),
            'do_not_track': request_data.get('do_not_track', False),
            'touch_support': request_data.get('touch_support', False)
        }

    async def _extract_hardware_specs(self, request_data: Dict) -> Dict:
        """Extract hardware specifications"""
        return {
            'cpu_cores': request_data.get('hardware_concurrency', 0),
            'device_memory': request_data.get('device_memory', 0),
            'max_touch_points': request_data.get('max_touch_points', 0),
            'gpu_vendor': request_data.get('webgl_vendor', ''),
            'gpu_renderer': request_data.get('webgl_renderer', ''),
            'canvas_fingerprint': request_data.get('canvas_fingerprint', ''),
            'webgl_fingerprint': request_data.get('webgl_fingerprint', ''),
            'audio_fingerprint': request_data.get('audio_fingerprint', ''),
            'battery_level': request_data.get('battery_level'),
            'charging_status': request_data.get('charging_status'),
            'connection_type': request_data.get('connection_type', ''),
            'connection_speed': request_data.get('connection_speed', 0)
        }

    async def _extract_network_info(self, request_data: Dict) -> Dict:
        """Extract network and IP information"""
        ip_address = request_data.get('ip_address', '')
        
        network_info = {
            'ip_address': ip_address,
            'ip_type': self._classify_ip_type(ip_address),
            'headers': request_data.get('headers', {}),
            'connection_type': request_data.get('connection_type', ''),
            'effective_type': request_data.get('effective_connection_type', ''),
            'downlink': request_data.get('downlink', 0),
            'rtt': request_data.get('rtt', 0),
            'save_data': request_data.get('save_data', False)
        }
        
        # Get geolocation info
        if ip_address:
            geo_info = await self._get_ip_geolocation(ip_address)
            network_info.update(geo_info)
            
        return network_info

    def _classify_ip_type(self, ip_address: str) -> str:
        """Classify IP address type (residential, datacenter, mobile, etc.)"""
        try:
            ip = ipaddress.ip_address(ip_address)
            
            # Check for private IPs
            if ip.is_private:
                return 'private'
            elif ip.is_loopback:
                return 'loopback'
            elif ip.is_multicast:
                return 'multicast'
            elif ip.is_reserved:
                return 'reserved'
            else:
                # For public IPs, we'd normally check against
                # datacenter IP ranges, but for now return 'public'
                return 'public'
                
        except ValueError:
            return 'invalid'

    async def _get_ip_geolocation(self, ip_address: str) -> Dict:
        """Get IP geolocation information"""
        try:
            # In production, use a proper IP geolocation service
            # This is a placeholder implementation
            return {
                'country': 'ID',  # Default to Indonesia
                'region': 'JK',   # Jakarta
                'city': 'Jakarta',
                'isp': 'Unknown',
                'organization': 'Unknown',
                'timezone': 'Asia/Jakarta'
            }
        except Exception:
            return {}

    async def _extract_browser_features(self, request_data: Dict) -> Dict:
        """Extract browser-specific features"""
        return {
            'plugins': request_data.get('plugins', []),
            'mime_types': request_data.get('mime_types', []),
            'fonts': request_data.get('fonts', []),
            'local_storage': request_data.get('local_storage_enabled', True),
            'session_storage': request_data.get('session_storage_enabled', True),
            'indexed_db': request_data.get('indexed_db_enabled', True),
            'web_rtc': request_data.get('webrtc_enabled', False),
            'webgl_enabled': request_data.get('webgl_enabled', False),
            'canvas_enabled': request_data.get('canvas_enabled', False),
            'notification_permission': request_data.get('notification_permission', 'default'),
            'geolocation_permission': request_data.get('geolocation_permission', 'default')
        }

    async def _extract_mobile_features(self, request_data: Dict) -> Dict:
        """Extract mobile-specific features"""
        return {
            'device_model': request_data.get('device_model', ''),
            'device_brand': request_data.get('device_brand', ''),
            'os_version': request_data.get('os_version', ''),
            'app_version': request_data.get('app_version', ''),
            'orientation': request_data.get('orientation', ''),
            'accelerometer_available': request_data.get('accelerometer', False),
            'gyroscope_available': request_data.get('gyroscope', False),
            'magnetometer_available': request_data.get('magnetometer', False),
            'proximity_sensor': request_data.get('proximity_sensor', False),
            'ambient_light_sensor': request_data.get('ambient_light', False),
            'vibration_support': request_data.get('vibration', False),
            'nfc_support': request_data.get('nfc', False),
            'bluetooth_support': request_data.get('bluetooth', False)
        }

    async def _generate_biometric_hash(self, request_data: Dict) -> Optional[str]:
        """Generate biometric hash if biometric data available"""
        biometric_data = request_data.get('biometric_data')
        if not biometric_data:
            return None
            
        try:
            # Create salt from device info
            salt_data = f"{request_data.get('device_id', '')}{request_data.get('user_id', '')}"
            salt = hashlib.sha256(salt_data.encode()).digest()[:16]
            
            # Derive key from biometric data
            kdf = PBKDF2HMAC(
                algorithm=hashes.SHA256(),
                length=32,
                salt=salt,
                iterations=100000,
            )
            key = kdf.derive(str(biometric_data).encode())
            
            return base64.b64encode(key).decode()
            
        except Exception as e:
            logger.error(f"Error generating biometric hash: {e}")
            return None

    async def _calculate_device_trust_score(self, features: Dict) -> float:
        """
        Calculate device trust score based on various factors
        Score: 0.0 (highly suspicious) to 1.0 (highly trusted)
        """
        score = 1.0
        penalties = []
        
        # Check for VM/Emulator indicators
        user_agent = features['basic_info'].get('user_agent', '').lower()
        for indicator in self.suspicious_patterns['vm_indicators']:
            if indicator in user_agent:
                penalties.append(('vm_indicator', 0.3))
                break
                
        for indicator in self.suspicious_patterns['emulator_indicators']:
            if indicator in user_agent:
                penalties.append(('emulator_indicator', 0.4))
                break
                
        for indicator in self.suspicious_patterns['automation_tools']:
            if indicator in user_agent:
                penalties.append(('automation_tool', 0.5))
                break
        
        # Check screen resolution
        resolution = features['basic_info'].get('screen_resolution', '')
        if resolution in self.suspicious_patterns['suspicious_resolutions']:
            penalties.append(('suspicious_resolution', 0.2))
        
        # Check hardware consistency
        cpu_cores = features['hardware_specs'].get('cpu_cores', 0)
        device_memory = features['hardware_specs'].get('device_memory', 0)
        
        if cpu_cores == 0 or device_memory == 0:
            penalties.append(('missing_hardware_info', 0.3))
        elif cpu_cores > 32:  # Suspiciously high
            penalties.append(('unrealistic_cpu', 0.2))
        elif device_memory > 128:  # Suspiciously high (128GB+)
            penalties.append(('unrealistic_memory', 0.2))
        
        # Check for consistent timezone/language
        timezone = features['basic_info'].get('timezone', '')
        language = features['basic_info'].get('language', '')
        country = features.get('network_info', {}).get('country', '')
        
        if not self._validate_geo_consistency(timezone, language, country):
            penalties.append(('geo_inconsistency', 0.15))
        
        # Apply penalties
        for penalty_type, penalty_value in penalties:
            score *= (1.0 - penalty_value)
            logger.debug(f"Applied penalty {penalty_type}: {penalty_value}, new score: {score}")
        
        # Ensure score is between 0 and 1
        return max(0.0, min(1.0, score))

    def _validate_geo_consistency(self, timezone: str, language: str, country: str) -> bool:
        """Validate geographic consistency between timezone, language, and IP location"""
        # Simplified validation - in production, use comprehensive geo data
        if country == 'ID':  # Indonesia
            valid_timezones = ['Asia/Jakarta', 'Asia/Makassar', 'Asia/Jayapura']
            valid_languages = ['id', 'id-ID', 'en', 'en-US']
            
            timezone_valid = any(tz in timezone for tz in valid_timezones) if timezone else True
            language_valid = any(lang in language for lang in valid_languages) if language else True
            
            return timezone_valid and language_valid
        
        return True  # Default to valid for other countries

    async def analyze_device_behavior(self, user_id: str, device_id: str, metrics: DeviceMetrics) -> Dict:
        """
        Analyze device behavior patterns for bot detection
        
        Args:
            user_id: User identifier
            device_id: Device identifier
            metrics: Real-time device behavior metrics
            
        Returns:
            Dict: Behavior analysis results
        """
        try:
            analysis = {
                'user_id': user_id,
                'device_id': device_id,
                'timestamp': datetime.now(),
                'behavior_score': 1.0,
                'anomalies': [],
                'risk_factors': []
            }
            
            # Analyze CPU usage patterns
            cpu_anomaly = self._analyze_cpu_patterns(metrics.cpu_usage_pattern)
            if cpu_anomaly:
                analysis['anomalies'].append(cpu_anomaly)
                analysis['behavior_score'] *= 0.8
            
            # Analyze click timing variance
            if metrics.click_timing_variance < 0.01:  # Too consistent
                analysis['risk_factors'].append('suspiciously_consistent_clicking')
                analysis['behavior_score'] *= 0.7
            elif metrics.click_timing_variance > 2.0:  # Too random
                analysis['risk_factors'].append('suspiciously_random_clicking')
                analysis['behavior_score'] *= 0.8
            
            # Analyze keystroke dynamics
            keystroke_analysis = self._analyze_keystroke_dynamics(metrics.keystroke_dynamics)
            if keystroke_analysis['suspicious']:
                analysis['risk_factors'].append('suspicious_keystroke_patterns')
                analysis['behavior_score'] *= keystroke_analysis['penalty']
            
            # Analyze mobile sensor data
            if metrics.gyroscope_data or metrics.accelerometer_data:
                sensor_analysis = self._analyze_sensor_data(
                    metrics.gyroscope_data, 
                    metrics.accelerometer_data
                )
                if sensor_analysis['suspicious']:
                    analysis['risk_factors'].append('suspicious_sensor_data')
                    analysis['behavior_score'] *= sensor_analysis['penalty']
            
            # Store analysis results
            await self._store_behavior_analysis(analysis)
            
            return analysis
            
        except Exception as e:
            logger.error(f"Error analyzing device behavior: {e}")
            raise

    def _analyze_cpu_patterns(self, cpu_pattern: List[float]) -> Optional[Dict]:
        """Analyze CPU usage patterns for anomalies"""
        if not cpu_pattern or len(cpu_pattern) < 10:
            return None
            
        # Calculate statistics
        mean_usage = np.mean(cpu_pattern)
        std_usage = np.std(cpu_pattern)
        
        # Check for suspicious patterns
        if std_usage < 0.01:  # Too stable
            return {
                'type': 'cpu_too_stable',
                'severity': 'medium',
                'details': f'CPU usage too stable: std={std_usage:.4f}'
            }
        elif mean_usage > 95:  # Consistently maxed out
            return {
                'type': 'cpu_maxed_out',
                'severity': 'high',
                'details': f'CPU consistently maxed: mean={mean_usage:.2f}%'
            }
        elif mean_usage < 0.1:  # Suspiciously low
            return {
                'type': 'cpu_too_low',
                'severity': 'medium',
                'details': f'CPU usage suspiciously low: mean={mean_usage:.2f}%'
            }
        
        return None

    def _analyze_keystroke_dynamics(self, keystroke_data: Dict) -> Dict:
        """Analyze keystroke timing patterns"""
        if not keystroke_data:
            return {'suspicious': False, 'penalty': 1.0}
        
        # Extract timing data
        dwell_times = keystroke_data.get('dwell_times', [])
        flight_times = keystroke_data.get('flight_times', [])
        
        if not dwell_times or not flight_times:
            return {'suspicious': False, 'penalty': 1.0}
        
        # Analyze patterns
        dwell_std = np.std(dwell_times)
        flight_std = np.std(flight_times)
        
        # Human typing has natural variance
        if dwell_std < 5 or flight_std < 10:  # Too consistent (ms)
            return {'suspicious': True, 'penalty': 0.6}
        elif dwell_std > 200 or flight_std > 500:  # Too random
            return {'suspicious': True, 'penalty': 0.7}
        
        return {'suspicious': False, 'penalty': 1.0}

    def _analyze_sensor_data(self, gyro_data: List[float], accel_data: List[float]) -> Dict:
        """Analyze mobile sensor data for authenticity"""
        if not gyro_data and not accel_data:
            return {'suspicious': False, 'penalty': 1.0}
        
        # Check for impossible values
        if gyro_data:
            max_gyro = max(abs(x) for x in gyro_data)
            if max_gyro > 2000:  # Beyond typical range (deg/s)
                return {'suspicious': True, 'penalty': 0.5}
        
        if accel_data:
            max_accel = max(abs(x) for x in accel_data)
            if max_accel > 100:  # Beyond Earth gravity + movement
                return {'suspicious': True, 'penalty': 0.5}
        
        # Check for suspiciously perfect patterns
        if gyro_data and np.std(gyro_data) < 0.001:
            return {'suspicious': True, 'penalty': 0.7}
        
        if accel_data and np.std(accel_data) < 0.001:
            return {'suspicious': True, 'penalty': 0.7}
        
        return {'suspicious': False, 'penalty': 1.0}

    async def _cache_device_profile(self, profile: DeviceProfile) -> None:
        """Cache device profile in Redis"""
        try:
            profile_data = asdict(profile)
            # Convert datetime objects to ISO strings for JSON serialization
            profile_data['created_at'] = profile.created_at.isoformat()
            profile_data['last_seen'] = profile.last_seen.isoformat()
            
            await self.redis.setex(
                f"device_profile:{profile.device_id}",
                self.config.get('cache_ttl', 3600),
                json.dumps(profile_data, default=str)
            )
            
        except Exception as e:
            logger.error(f"Error caching device profile: {e}")

    async def _store_behavior_analysis(self, analysis: Dict) -> None:
        """Store behavior analysis results"""
        try:
            # Convert datetime to ISO string
            analysis['timestamp'] = analysis['timestamp'].isoformat()
            
            # Store in Redis with TTL
            key = f"behavior_analysis:{analysis['user_id']}:{analysis['device_id']}:{int(time.time())}"
            await self.redis.setex(
                key,
                self.config.get('behavior_cache_ttl', 86400),  # 24 hours
                json.dumps(analysis, default=str)
            )
            
        except Exception as e:
            logger.error(f"Error storing behavior analysis: {e}")

    async def get_device_history(self, device_id: str, days: int = 30) -> List[Dict]:
        """Get device behavior history"""
        try:
            end_time = datetime.now()
            start_time = end_time - timedelta(days=days)
            
            # Scan for behavior analysis keys
            pattern = f"behavior_analysis:*:{device_id}:*"
            keys = await self.redis.keys(pattern)
            
            history = []
            for key in keys:
                data = await self.redis.get(key)
                if data:
                    analysis = json.loads(data)
                    timestamp = datetime.fromisoformat(analysis['timestamp'])
                    if start_time <= timestamp <= end_time:
                        history.append(analysis)
            
            # Sort by timestamp
            history.sort(key=lambda x: x['timestamp'])
            return history
            
        except Exception as e:
            logger.error(f"Error getting device history: {e}")
            return []

    async def update_device_trust_score(self, device_id: str, score_adjustment: float, reason: str) -> None:
        """Update device trust score based on behavior"""
        try:
            profile_key = f"device_profile:{device_id}"
            profile_data = await self.redis.get(profile_key)
            
            if profile_data:
                profile = json.loads(profile_data)
                old_score = profile['device_trust_score']
                new_score = max(0.0, min(1.0, old_score + score_adjustment))
                
                profile['device_trust_score'] = new_score
                profile['last_seen'] = datetime.now().isoformat()
                
                # Log the change
                logger.info(f"Device {device_id} trust score: {old_score:.3f} -> {new_score:.3f} (reason: {reason})")
                
                # Update cache
                await self.redis.setex(
                    profile_key,
                    self.config.get('cache_ttl', 3600),
                    json.dumps(profile, default=str)
                )
                
        except Exception as e:
            logger.error(f"Error updating device trust score: {e}")

# Factory function for easy initialization
def create_device_feature_extractor(redis_client: redis.Redis, config: Dict) -> DeviceFeatureExtractor:
    """Create and configure device feature extractor"""
    return DeviceFeatureExtractor(redis_client, config)

# Example usage and testing
if __name__ == "__main__":
    import asyncio
    
    # Mock Redis client for testing
    class MockRedis:
        def __init__(self):
            self.data = {}
        
        async def setex(self, key, ttl, value):
            self.data[key] = value
        
        async def get(self, key):
            return self.data.get(key)
        
        async def keys(self, pattern):
            return [k for k in self.data.keys() if pattern.replace('*', '') in k]
    
    async def test_device_features():
        config = {
            'cache_ttl': 3600,
            'behavior_cache_ttl': 86400
        }
        
        extractor = DeviceFeatureExtractor(MockRedis(), config)
        
        # Test device feature extraction
        sample_request = {
            'user_agent': 'Mozilla/5.0 (Linux; Android 10; SM-G975F) AppleWebKit/537.36',
            'screen_width': 1440,
            'screen_height': 3040,
            'timezone': 'Asia/Jakarta',
            'language': 'id-ID',
            'platform': 'Android',
            'hardware_concurrency': 8,
            'device_memory': 8,
            'ip_address': '203.142.74.1'
        }
        
        profile = await extractor.extract_device_features(sample_request)
        print(f"Device ID: {profile.device_id}")
        print(f"Trust Score: {profile.device_trust_score:.3f}")
        print(f"Platform: {profile.platform_info}")
        
        # Test behavior analysis
        metrics = DeviceMetrics(
            cpu_usage_pattern=[12.5, 15.2, 11.8, 13.9, 14.1],
            memory_usage_pattern=[45.2, 46.1, 44.8, 45.9],
            network_latency=45.0,
            click_timing_variance=0.15,
            keystroke_dynamics={'dwell_times': [80, 75, 90, 85], 'flight_times': [120, 115, 125]},
            touch_pressure_patterns=[0.8, 0.9, 0.7, 0.85],
            gyroscope_data=[0.1, -0.2, 0.15, 0.05],
            accelerometer_data=[9.8, 9.7, 9.9, 9.8],
            battery_level_changes=[(datetime.now(), 85)],
            app_switching_frequency=5
        )
        
        behavior_analysis = await extractor.analyze_device_behavior(
            'user_123', profile.device_id, metrics
        )
        print(f"Behavior Score: {behavior_analysis['behavior_score']:.3f}")
        print(f"Risk Factors: {behavior_analysis['risk_factors']}")
        
    # Run test
    asyncio.run(test_device_features())
    