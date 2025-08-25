# finova-net/finova/client/python/finova/utils.py

"""
Finova Network Python Client SDK - Utilities Module
Enterprise-grade utility functions for Finova Network integration
Version: 3.0
"""

import hashlib
import hmac
import json
import time
import math
import base64
import secrets
from typing import Dict, List, Optional, Union, Tuple, Any
from decimal import Decimal, ROUND_HALF_UP
from datetime import datetime, timedelta, timezone
import re
from dataclasses import dataclass
from enum import Enum


class PlatformType(Enum):
    """Social media platform types with multipliers"""
    INSTAGRAM = ("instagram", 1.2)
    TIKTOK = ("tiktok", 1.3)
    YOUTUBE = ("youtube", 1.4)
    FACEBOOK = ("facebook", 1.1)
    TWITTER_X = ("twitter_x", 1.2)
    APP_NATIVE = ("app_native", 1.0)

    def __init__(self, platform_id: str, multiplier: float):
        self.platform_id = platform_id
        self.multiplier = multiplier


class ActivityType(Enum):
    """User activity types with base XP values"""
    ORIGINAL_POST = ("original_post", 50)
    PHOTO_POST = ("photo_post", 75)
    VIDEO_POST = ("video_post", 150)
    STORY_POST = ("story_post", 25)
    COMMENT = ("comment", 25)
    LIKE_REACT = ("like_react", 5)
    SHARE_REPOST = ("share_repost", 15)
    FOLLOW_SUBSCRIBE = ("follow_subscribe", 20)
    DAILY_LOGIN = ("daily_login", 10)
    DAILY_QUEST = ("daily_quest", 100)
    MILESTONE = ("milestone", 500)
    VIRAL_CONTENT = ("viral_content", 1000)

    def __init__(self, activity_id: str, base_xp: int):
        self.activity_id = activity_id
        self.base_xp = base_xp


@dataclass
class MiningConfig:
    """Mining configuration constants"""
    BASE_RATE_PHASE_1 = 0.1  # FIN/hour
    BASE_RATE_PHASE_2 = 0.05
    BASE_RATE_PHASE_3 = 0.025
    BASE_RATE_PHASE_4 = 0.01
    
    PHASE_1_USERS = 100_000
    PHASE_2_USERS = 1_000_000
    PHASE_3_USERS = 10_000_000
    
    PIONEER_BONUS_MAX = 2.0
    REFERRAL_BONUS_RATE = 0.1
    KYC_SECURITY_BONUS = 1.2
    NON_KYC_PENALTY = 0.8
    REGRESSION_COEFFICIENT = 0.001


class FinovaUtils:
    """Core utility functions for Finova Network calculations"""
    
    @staticmethod
    def generate_referral_code(user_id: str, prefix: str = "FIN") -> str:
        """Generate unique referral code"""
        timestamp = str(int(time.time()))
        data = f"{user_id}:{timestamp}"
        hash_digest = hashlib.sha256(data.encode()).hexdigest()[:8]
        return f"{prefix}{hash_digest.upper()}"
    
    @staticmethod
    def validate_referral_code(code: str) -> bool:
        """Validate referral code format"""
        pattern = r"^FIN[A-F0-9]{8}$"
        return bool(re.match(pattern, code))
    
    @staticmethod
    def calculate_mining_rate(
        total_users: int,
        user_holdings: float,
        active_referrals: int,
        is_kyc_verified: bool
    ) -> float:
        """Calculate user's mining rate using exponential regression"""
        config = MiningConfig()
        
        # Determine base rate by phase
        if total_users <= config.PHASE_1_USERS:
            base_rate = config.BASE_RATE_PHASE_1
        elif total_users <= config.PHASE_2_USERS:
            base_rate = config.BASE_RATE_PHASE_2
        elif total_users <= config.PHASE_3_USERS:
            base_rate = config.BASE_RATE_PHASE_3
        else:
            base_rate = config.BASE_RATE_PHASE_4
        
        # Pioneer bonus
        pioneer_bonus = max(1.0, config.PIONEER_BONUS_MAX - (total_users / 1_000_000))
        
        # Referral bonus
        referral_bonus = 1 + (active_referrals * config.REFERRAL_BONUS_RATE)
        
        # Security bonus
        security_bonus = config.KYC_SECURITY_BONUS if is_kyc_verified else config.NON_KYC_PENALTY
        
        # Exponential regression (anti-whale)
        regression_factor = math.exp(-config.REGRESSION_COEFFICIENT * user_holdings)
        
        return base_rate * pioneer_bonus * referral_bonus * security_bonus * regression_factor
    
    @staticmethod
    def calculate_xp_gain(
        activity_type: ActivityType,
        platform: PlatformType,
        quality_score: float,
        streak_days: int,
        user_level: int
    ) -> int:
        """Calculate XP gain for user activity"""
        base_xp = activity_type.base_xp
        platform_multiplier = platform.multiplier
        
        # Quality score (0.5x - 2.0x)
        quality_multiplier = max(0.5, min(2.0, quality_score))
        
        # Streak bonus (1.0x - 3.0x)
        streak_bonus = min(3.0, 1.0 + (streak_days * 0.1))
        
        # Level progression (exponential decay)
        level_progression = math.exp(-0.01 * user_level)
        
        total_xp = (base_xp * platform_multiplier * quality_multiplier * 
                   streak_bonus * level_progression)
        
        return int(round(total_xp))
    
    @staticmethod
    def calculate_rp_value(
        direct_referrals: List[Dict],
        network_data: Dict,
        network_quality_score: float
    ) -> float:
        """Calculate Referral Points value"""
        # Direct referral points
        direct_rp = sum(
            ref.get('activity_score', 0) * ref.get('level', 1) * 
            ref.get('retention_factor', 1.0) for ref in direct_referrals
        )
        
        # Indirect network points
        l2_activity = network_data.get('l2_activity', 0)
        l3_activity = network_data.get('l3_activity', 0)
        indirect_rp = (l2_activity * 0.3) + (l3_activity * 0.1)
        
        # Network quality bonus
        total_network_size = network_data.get('total_size', 1)
        network_diversity = network_data.get('diversity_score', 1.0)
        average_level = network_data.get('average_level', 1)
        retention_rate = network_data.get('retention_rate', 0.5)
        
        quality_bonus = network_diversity * average_level * retention_rate
        
        # Exponential regression for large networks
        regression_factor = math.exp(-0.0001 * total_network_size * network_quality_score)
        
        return (direct_rp + indirect_rp) * quality_bonus * regression_factor
    
    @staticmethod
    def get_xp_level_info(total_xp: int) -> Dict[str, Any]:
        """Get user level information from total XP"""
        level_thresholds = [
            (0, 10, "Bronze", 1.0, 1.2),
            (11, 25, "Silver", 1.3, 1.8),
            (26, 50, "Gold", 1.9, 2.5),
            (51, 75, "Platinum", 2.6, 3.2),
            (76, 100, "Diamond", 3.3, 4.0),
            (101, float('inf'), "Mythic", 4.1, 5.0)
        ]
        
        # Calculate level from XP
        if total_xp < 1000:
            level = total_xp // 100 + 1
        elif total_xp < 5000:
            level = 10 + (total_xp - 1000) // 250 + 1
        elif total_xp < 20000:
            level = 25 + (total_xp - 5000) // 600 + 1
        elif total_xp < 50000:
            level = 50 + (total_xp - 20000) // 1200 + 1
        elif total_xp < 100000:
            level = 75 + (total_xp - 50000) // 2000 + 1
        else:
            level = 100 + (total_xp - 100000) // 5000 + 1
        
        # Find tier information
        for min_level, max_level, tier, min_mult, max_mult in level_thresholds:
            if min_level <= level <= max_level:
                tier_progress = (level - min_level) / (max_level - min_level) if max_level != float('inf') else 0
                mining_multiplier = min_mult + (max_mult - min_mult) * tier_progress
                
                return {
                    "level": level,
                    "tier": tier,
                    "mining_multiplier": round(mining_multiplier, 2),
                    "tier_progress": round(tier_progress, 2),
                    "xp_for_next": FinovaUtils._calculate_xp_for_next_level(level),
                    "daily_fin_cap": FinovaUtils._calculate_daily_fin_cap(tier)
                }
        
        return {"level": level, "tier": "Unknown", "mining_multiplier": 1.0}
    
    @staticmethod
    def _calculate_xp_for_next_level(current_level: int) -> int:
        """Calculate XP needed for next level"""
        if current_level <= 10:
            return (current_level * 100) - (current_level - 1) * 100
        elif current_level <= 25:
            return 250
        elif current_level <= 50:
            return 600
        elif current_level <= 75:
            return 1200
        elif current_level <= 100:
            return 2000
        else:
            return 5000
    
    @staticmethod
    def _calculate_daily_fin_cap(tier: str) -> float:
        """Calculate daily FIN earning cap by tier"""
        caps = {
            "Bronze": 2.0,
            "Silver": 4.0,
            "Gold": 6.0,
            "Platinum": 8.0,
            "Diamond": 10.0,
            "Mythic": 15.0
        }
        return caps.get(tier, 1.0)
    
    @staticmethod
    def get_rp_tier_info(total_rp: float) -> Dict[str, Any]:
        """Get RP tier information"""
        tiers = [
            (0, 999, "Explorer", 0, 10, ["Basic referral link"]),
            (1000, 4999, "Connector", 20, 25, ["Custom referral code"]),
            (5000, 14999, "Influencer", 50, 50, ["Referral analytics"]),
            (15000, 49999, "Leader", 100, 100, ["Exclusive events"]),
            (50000, float('inf'), "Ambassador", 200, -1, ["DAO governance"])
        ]
        
        for min_rp, max_rp, tier, mining_bonus, referral_cap, benefits in tiers:
            if min_rp <= total_rp <= max_rp:
                return {
                    "tier": tier,
                    "mining_bonus": mining_bonus,
                    "referral_cap": referral_cap if referral_cap != -1 else "Unlimited",
                    "benefits": benefits,
                    "progress": (total_rp - min_rp) / (max_rp - min_rp) if max_rp != float('inf') else 1.0
                }
        
        return {"tier": "Unknown", "mining_bonus": 0}
    
    @staticmethod
    def calculate_staking_rewards(
        staked_amount: float,
        staking_duration_days: int,
        user_level: int,
        rp_tier: str,
        daily_activity_score: float
    ) -> Dict[str, float]:
        """Calculate staking rewards and multipliers"""
        # Base APY by stake amount
        if staked_amount < 500:
            base_apy = 0.08  # 8%
        elif staked_amount < 1000:
            base_apy = 0.10  # 10%
        elif staked_amount < 5000:
            base_apy = 0.12  # 12%
        elif staked_amount < 10000:
            base_apy = 0.14  # 14%
        else:
            base_apy = 0.15  # 15%
        
        # Multiplier bonuses
        xp_level_bonus = 1.0 + (user_level / 100)
        
        rp_tier_bonuses = {
            "Explorer": 1.0, "Connector": 1.2, "Influencer": 1.4,
            "Leader": 1.6, "Ambassador": 2.0
        }
        rp_tier_bonus = rp_tier_bonuses.get(rp_tier, 1.0)
        
        loyalty_bonus = 1.0 + (staking_duration_days / 30 * 0.05)  # 5% per month
        activity_bonus = 1.0 + (daily_activity_score * 0.1)
        
        total_multiplier = xp_level_bonus * rp_tier_bonus * loyalty_bonus * activity_bonus
        effective_apy = base_apy * total_multiplier
        
        daily_rewards = (staked_amount * effective_apy) / 365
        
        return {
            "base_apy": base_apy,
            "effective_apy": min(effective_apy, 0.50),  # Cap at 50%
            "daily_rewards": daily_rewards,
            "xp_level_bonus": xp_level_bonus,
            "rp_tier_bonus": rp_tier_bonus,
            "loyalty_bonus": loyalty_bonus,
            "activity_bonus": activity_bonus
        }
    
    @staticmethod
    def validate_content_quality(content: str, content_type: str) -> float:
        """Basic content quality validation (placeholder for AI integration)"""
        if not content or len(content.strip()) < 10:
            return 0.5
        
        # Basic quality metrics
        word_count = len(content.split())
        char_count = len(content)
        
        # Length score
        if content_type == "comment":
            optimal_length = 50
        elif content_type == "post":
            optimal_length = 200
        else:
            optimal_length = 100
        
        length_score = min(1.0, char_count / optimal_length)
        
        # Engagement potential (simple heuristics)
        engagement_indicators = ['?', '!', '#', '@', 'http', 'what', 'how', 'why']
        engagement_score = min(1.0, sum(1 for indicator in engagement_indicators if indicator in content.lower()) * 0.2)
        
        # Combine scores
        quality_score = (length_score * 0.4) + (engagement_score * 0.6) + 0.5  # Base 0.5
        
        return max(0.5, min(2.0, quality_score))
    
    @staticmethod
    def calculate_card_synergy(active_cards: List[Dict]) -> float:
        """Calculate NFT card synergy multiplier"""
        if not active_cards:
            return 1.0
        
        card_count = len(active_cards)
        
        # Rarity bonuses
        rarity_bonuses = {
            "common": 0.0,
            "uncommon": 0.05,
            "rare": 0.10,
            "epic": 0.20,
            "legendary": 0.35
        }
        
        rarity_bonus = sum(rarity_bonuses.get(card.get('rarity', 'common'), 0) for card in active_cards)
        
        # Type matching bonus
        card_types = [card.get('type') for card in active_cards]
        unique_types = set(card_types)
        
        if len(unique_types) == 3:  # All three categories
            type_bonus = 0.30
        elif len(card_types) > 1 and len(unique_types) == 1:  # Same category
            type_bonus = 0.15
        else:
            type_bonus = 0.0
        
        synergy_multiplier = 1.0 + (card_count * 0.1) + rarity_bonus + type_bonus
        
        return min(synergy_multiplier, 3.0)  # Cap at 3x
    
    @staticmethod
    def format_fin_amount(amount: float, decimals: int = 4) -> str:
        """Format FIN amount with proper decimals"""
        decimal_amount = Decimal(str(amount))
        formatted = decimal_amount.quantize(
            Decimal('0.' + '0' * decimals),
            rounding=ROUND_HALF_UP
        )
        return f"{formatted} FIN"
    
    @staticmethod
    def format_time_remaining(seconds: int) -> str:
        """Format time remaining in human-readable format"""
        if seconds <= 0:
            return "Expired"
        
        days = seconds // 86400
        hours = (seconds % 86400) // 3600
        minutes = (seconds % 3600) // 60
        
        if days > 0:
            return f"{days}d {hours}h {minutes}m"
        elif hours > 0:
            return f"{hours}h {minutes}m"
        else:
            return f"{minutes}m"
    
    @staticmethod
    def create_signature(data: Dict, secret_key: str) -> str:
        """Create HMAC signature for API requests"""
        json_data = json.dumps(data, sort_keys=True, separators=(',', ':'))
        signature = hmac.new(
            secret_key.encode(),
            json_data.encode(),
            hashlib.sha256
        ).hexdigest()
        return signature
    
    @staticmethod
    def verify_signature(data: Dict, signature: str, secret_key: str) -> bool:
        """Verify HMAC signature"""
        expected_signature = FinovaUtils.create_signature(data, secret_key)
        return hmac.compare_digest(signature, expected_signature)
    
    @staticmethod
    def generate_nonce() -> str:
        """Generate cryptographically secure nonce"""
        return secrets.token_hex(16)
    
    @staticmethod
    def calculate_network_health(network_data: Dict) -> Dict[str, float]:
        """Calculate network health metrics"""
        total_users = network_data.get('total_users', 0)
        active_users = network_data.get('active_users', 0)
        retention_rate = network_data.get('retention_rate', 0)
        average_activity = network_data.get('average_activity', 0)
        
        if total_users == 0:
            return {"health_score": 0.0, "activity_score": 0.0, "retention_score": 0.0}
        
        activity_score = min(1.0, active_users / total_users)
        retention_score = min(1.0, retention_rate)
        engagement_score = min(1.0, average_activity / 100)  # Normalize to 100
        
        health_score = (activity_score * 0.4) + (retention_score * 0.4) + (engagement_score * 0.2)
        
        return {
            "health_score": round(health_score, 3),
            "activity_score": round(activity_score, 3),
            "retention_score": round(retention_score, 3),
            "engagement_score": round(engagement_score, 3)
        }
    
    @staticmethod
    def validate_wallet_address(address: str, network: str = "solana") -> bool:
        """Validate blockchain wallet address"""
        if network.lower() == "solana":
            # Solana addresses are base58 encoded, 44 characters
            pattern = r"^[1-9A-HJ-NP-Za-km-z]{44}$"
            return bool(re.match(pattern, address))
        
        return False
    
    @staticmethod
    def calculate_anti_bot_score(user_behavior: Dict) -> float:
        """Calculate anti-bot detection score"""
        factors = {
            'click_speed_variance': user_behavior.get('click_variance', 0.5),
            'session_patterns': user_behavior.get('session_naturalness', 0.5),
            'content_originality': user_behavior.get('content_uniqueness', 0.5),
            'social_connections': user_behavior.get('real_connections', 0.5),
            'device_consistency': user_behavior.get('device_stability', 0.5)
        }
        
        weights = {
            'click_speed_variance': 0.2,
            'session_patterns': 0.25,
            'content_originality': 0.25,
            'social_connections': 0.2,
            'device_consistency': 0.1
        }
        
        weighted_score = sum(factors[key] * weights[key] for key in factors)
        
        return max(0.1, min(1.0, weighted_score))


class FinovaValidator:
    """Validation utilities for Finova Network"""
    
    @staticmethod
    def validate_user_data(user_data: Dict) -> Tuple[bool, List[str]]:
        """Validate user registration data"""
        errors = []
        
        # Required fields
        required_fields = ['username', 'email', 'wallet_address']
        for field in required_fields:
            if field not in user_data or not user_data[field]:
                errors.append(f"Missing required field: {field}")
        
        # Username validation
        if 'username' in user_data:
            username = user_data['username']
            if len(username) < 3 or len(username) > 20:
                errors.append("Username must be 3-20 characters")
            if not re.match(r"^[a-zA-Z0-9_]+$", username):
                errors.append("Username can only contain letters, numbers, and underscores")
        
        # Email validation
        if 'email' in user_data:
            email = user_data['email']
            email_pattern = r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$"
            if not re.match(email_pattern, email):
                errors.append("Invalid email format")
        
        # Wallet address validation
        if 'wallet_address' in user_data:
            if not FinovaUtils.validate_wallet_address(user_data['wallet_address']):
                errors.append("Invalid wallet address")
        
        return len(errors) == 0, errors
    
    @staticmethod
    def validate_activity_data(activity_data: Dict) -> Tuple[bool, List[str]]:
        """Validate social media activity data"""
        errors = []
        
        required_fields = ['activity_type', 'platform', 'content']
        for field in required_fields:
            if field not in activity_data:
                errors.append(f"Missing required field: {field}")
        
        # Validate activity type
        if 'activity_type' in activity_data:
            valid_types = [t.activity_id for t in ActivityType]
            if activity_data['activity_type'] not in valid_types:
                errors.append(f"Invalid activity type: {activity_data['activity_type']}")
        
        # Validate platform
        if 'platform' in activity_data:
            valid_platforms = [p.platform_id for p in PlatformType]
            if activity_data['platform'] not in valid_platforms:
                errors.append(f"Invalid platform: {activity_data['platform']}")
        
        # Content validation
        if 'content' in activity_data:
            content = activity_data['content']
            if len(content) > 10000:  # 10K character limit
                errors.append("Content exceeds maximum length (10,000 characters)")
        
        return len(errors) == 0, errors


class FinovaConstants:
    """Constants and configuration values"""
    
    # API Configuration
    API_VERSION = "v3.0"
    DEFAULT_TIMEOUT = 30
    MAX_RETRIES = 3
    
    # Mining Configuration
    MINING_CONFIG = MiningConfig()
    
    # XP Configuration
    MAX_DAILY_XP = 10000
    STREAK_MAX_BONUS = 3.0
    LEVEL_PROGRESSION_DECAY = 0.01
    
    # RP Configuration
    MAX_REFERRAL_DEPTH = 3
    NETWORK_REGRESSION_COEFFICIENT = 0.0001
    
    # Staking Configuration
    MIN_STAKE_AMOUNT = 100
    MAX_STAKE_APY = 0.50  # 50%
    LOYALTY_BONUS_RATE = 0.05  # 5% per month
    
    # Security Configuration
    MIN_PASSWORD_LENGTH = 8
    MAX_LOGIN_ATTEMPTS = 5
    SESSION_TIMEOUT = 3600  # 1 hour
    
    # Platform Limits
    DAILY_ACTIVITY_LIMITS = {
        'original_post': float('inf'),
        'photo_post': 20,
        'video_post': 10,
        'story_post': 50,
        'comment': 100,
        'like_react': 200,
        'share_repost': 50,
        'follow_subscribe': 25
    }
    
    # Quality Score Ranges
    MIN_QUALITY_SCORE = 0.5
    MAX_QUALITY_SCORE = 2.0
    
    # Network Health Weights
    NETWORK_HEALTH_WEIGHTS = {
        'activity': 0.4,
        'retention': 0.4,
        'engagement': 0.2
    }


# Export main classes and functions
__all__ = [
    'FinovaUtils',
    'FinovaValidator', 
    'FinovaConstants',
    'PlatformType',
    'ActivityType',
    'MiningConfig'
]
