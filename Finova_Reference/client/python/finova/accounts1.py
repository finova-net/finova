# finova-net/finova/client/python/finova/accounts.py

"""
Finova Network Python Client SDK - Accounts Module (Part 1)
Enterprise-grade account management for Finova ecosystem
Supports: Mining, XP, RP, Staking, NFTs, Guilds, Anti-bot, Quality assessment
"""

import asyncio
import json
import math
import time
from dataclasses import dataclass, field
from datetime import datetime, timedelta, timezone
from decimal import Decimal
from enum import Enum
from typing import Dict, List, Optional, Union, Any, Tuple
from solana.publickey import PublicKey
from solana.keypair import Keypair
from solana.rpc.async_api import AsyncClient
from solana.transaction import Transaction
from solders.signature import Signature
import hashlib
import hmac
import base64
from cryptography.hazmat.primitives import hashes
from cryptography.hazmat.primitives.kdf.pbkdf2 import PBKDF2HMAC

# Core Constants
FIN_DECIMALS = 9
BASE_MINING_RATE = 0.05
MAX_REFERRAL_DEPTH = 3
XP_LEVEL_CAP = 1000
RP_TIER_CAP = 5
QUALITY_SCORE_MIN = 0.5
QUALITY_SCORE_MAX = 2.0
REGRESSION_FACTOR = 0.001
NETWORK_REGRESSION = 0.0001

# Enums
class UserTier(Enum):
    EXPLORER = 0
    CONNECTOR = 1
    INFLUENCER = 2
    LEADER = 3
    AMBASSADOR = 4

class XPLevel(Enum):
    BRONZE = (1, 10)
    SILVER = (11, 25)
    GOLD = (26, 50)
    PLATINUM = (51, 75)
    DIAMOND = (76, 100)
    MYTHIC = (101, 1000)

class MiningPhase(Enum):
    FINIZEN = 1
    GROWTH = 2
    MATURITY = 3
    STABILITY = 4

class ActivityType(Enum):
    POST = "post"
    COMMENT = "comment"
    LIKE = "like"
    SHARE = "share"
    FOLLOW = "follow"
    VIRAL = "viral"
    LOGIN = "login"
    QUEST = "quest"

class Platform(Enum):
    INSTAGRAM = ("instagram", 1.2)
    TIKTOK = ("tiktok", 1.3)
    YOUTUBE = ("youtube", 1.4)
    FACEBOOK = ("facebook", 1.1)
    TWITTER = ("twitter", 1.2)
    APP = ("app", 1.0)

class CardType(Enum):
    MINING_BOOST = "mining_boost"
    XP_ACCELERATOR = "xp_accelerator"
    REFERRAL_POWER = "referral_power"
    SPECIAL_EVENT = "special_event"

class CardRarity(Enum):
    COMMON = (1, 0.0)
    UNCOMMON = (2, 0.05)
    RARE = (3, 0.10)
    EPIC = (4, 0.20)
    LEGENDARY = (5, 0.35)

# Core Data Structures
@dataclass
class UserProfile:
    """Core user profile with all account data"""
    public_key: str
    user_id: str
    username: str
    email: Optional[str] = None
    phone: Optional[str] = None
    
    # KYC & Security
    kyc_verified: bool = False
    kyc_level: int = 0
    human_score: float = 1.0
    security_score: float = 1.0
    suspicious_activity_count: int = 0
    last_security_check: Optional[datetime] = None
    
    # Account Status
    is_active: bool = True
    is_banned: bool = False
    ban_reason: Optional[str] = None
    created_at: datetime = field(default_factory=lambda: datetime.now(timezone.utc))
    last_active: datetime = field(default_factory=lambda: datetime.now(timezone.utc))
    
    # Progression
    current_level: int = 1
    total_xp: int = 0
    current_tier: UserTier = UserTier.EXPLORER
    total_rp: int = 0
    
    # Holdings
    fin_balance: Decimal = Decimal('0')
    sfin_balance: Decimal = Decimal('0')
    usdfin_balance: Decimal = Decimal('0')
    susdfin_balance: Decimal = Decimal('0')
    
    # Statistics
    total_posts: int = 0
    total_comments: int = 0
    total_likes_given: int = 0
    total_likes_received: int = 0
    total_shares: int = 0
    total_followers: int = 0
    total_following: int = 0
    
    # Referral Data
    referral_code: str = ""
    referred_by: Optional[str] = None
    direct_referrals: List[str] = field(default_factory=list)
    total_network_size: int = 0
    active_referrals_30d: int = 0
    
    # Guild Data
    guild_id: Optional[str] = None
    guild_role: Optional[str] = None
    guild_joined_at: Optional[datetime] = None
    
    # Device & Security
    device_fingerprints: List[str] = field(default_factory=list)
    ip_addresses: List[str] = field(default_factory=list)
    login_sessions: List[Dict] = field(default_factory=list)

@dataclass
class MiningAccount:
    """Mining-specific account data"""
    user_id: str
    
    # Current Mining Status
    is_mining: bool = False
    mining_start_time: Optional[datetime] = None
    current_mining_rate: Decimal = Decimal('0')
    total_mined: Decimal = Decimal('0')
    pending_rewards: Decimal = Decimal('0')
    
    # Mining Multipliers
    base_rate_multiplier: float = 1.0
    finizen_bonus: float = 1.0
    referral_bonus: float = 1.0
    security_bonus: float = 1.0
    xp_multiplier: float = 1.0
    rp_multiplier: float = 1.0
    staking_multiplier: float = 1.0
    card_multiplier: float = 1.0
    quality_multiplier: float = 1.0
    
    # Regression Factors
    whale_regression: float = 1.0
    network_regression: float = 1.0
    
    # Mining History
    daily_mining_history: List[Dict] = field(default_factory=list)
    total_mining_sessions: int = 0
    longest_mining_streak: int = 0
    current_mining_streak: int = 0
    
    # Phase Information
    current_phase: MiningPhase = MiningPhase.FINIZEN
    phase_start_time: datetime = field(default_factory=lambda: datetime.now(timezone.utc))
    
    # Activity Boosters
    active_boosters: List[Dict] = field(default_factory=list)
    booster_expiry_times: Dict[str, datetime] = field(default_factory=dict)
    
    # Anti-bot measures
    difficulty_multiplier: float = 1.0
    penalty_factor: float = 1.0
    cooling_period_end: Optional[datetime] = None

@dataclass
class XPAccount:
    """Experience Points account data"""
    user_id: str
    
    # XP Status
    total_xp: int = 0
    current_level: int = 1
    xp_to_next_level: int = 100
    level_progress_percentage: float = 0.0
    
    # XP Breakdown by Activity
    xp_from_posts: int = 0
    xp_from_comments: int = 0
    xp_from_likes: int = 0
    xp_from_shares: int = 0
    xp_from_follows: int = 0
    xp_from_viral_content: int = 0
    xp_from_daily_login: int = 0
    xp_from_quests: int = 0
    xp_from_achievements: int = 0
    
    # XP Breakdown by Platform
    xp_from_instagram: int = 0
    xp_from_tiktok: int = 0
    xp_from_youtube: int = 0
    xp_from_facebook: int = 0
    xp_from_twitter: int = 0
    xp_from_app: int = 0
    
    # Multipliers & Bonuses
    level_multiplier: float = 1.0
    streak_bonus: float = 1.0
    quality_bonus: float = 1.0
    platform_bonus: float = 1.0
    card_bonus: float = 1.0
    
    # Streaks
    daily_login_streak: int = 0
    activity_streak: int = 0
    quality_streak: int = 0
    
    # Level Information
    level_tier: str = "Bronze"
    level_badge: str = "Bronze I"
    mining_multiplier_from_level: float = 1.0
    daily_fin_cap_from_level: Decimal = Decimal('0.5')
    
    # Activity Tracking
    daily_activities: List[Dict] = field(default_factory=list)
    weekly_activities: List[Dict] = field(default_factory=list)
    monthly_activities: List[Dict] = field(default_factory=list)
    
    # Quality Metrics
    average_content_quality: float = 1.0
    content_originality_score: float = 1.0
    engagement_prediction_score: float = 1.0
    brand_safety_score: float = 1.0

@dataclass
class RPAccount:
    """Referral Points account data"""
    user_id: str
    
    # RP Status
    total_rp: int = 0
    current_tier: UserTier = UserTier.EXPLORER
    tier_progress_percentage: float = 0.0
    rp_to_next_tier: int = 1000
    
    # Network Structure
    direct_referrals: List[str] = field(default_factory=list)
    l2_network: List[str] = field(default_factory=list)
    l3_network: List[str] = field(default_factory=list)
    total_network_size: int = 0
    
    # Network Quality Metrics
    active_referrals_7d: int = 0
    active_referrals_30d: int = 0
    network_retention_rate: float = 0.0
    network_quality_score: float = 0.0
    network_diversity_score: float = 0.0
    
    # RP Sources
    rp_from_registrations: int = 0
    rp_from_kyc_completions: int = 0
    rp_from_first_mining: int = 0
    rp_from_daily_activity: int = 0
    rp_from_achievements: int = 0
    rp_from_network_bonuses: int = 0
    
    # Bonuses & Multipliers
    tier_multiplier: float = 1.0
    network_effect_bonus: float = 1.0
    quality_bonus_multiplier: float = 1.0
    activity_bonus_multiplier: float = 1.0
    
    # Tier Benefits
    mining_bonus_percentage: float = 0.0
    referral_commission_percentage: float = 10.0
    network_cap: int = 10
    special_benefits: List[str] = field(default_factory=list)
    
    # Network Analytics
    network_growth_rate: float = 0.0
    network_churn_rate: float = 0.0
    average_referral_level: float = 1.0
    network_lifetime_value: Decimal = Decimal('0')
    
    # Regression Factors
    network_regression_factor: float = 1.0
    quality_regression_factor: float = 1.0

@dataclass
class StakingAccount:
    """Staking account data"""
    user_id: str
    
    # Staking Status
    total_staked_fin: Decimal = Decimal('0')
    total_staked_usdfin: Decimal = Decimal('0')
    sfin_balance: Decimal = Decimal('0')
    susdfin_balance: Decimal = Decimal('0')
    
    # Staking Positions
    fin_positions: List[Dict] = field(default_factory=list)
    usdfin_positions: List[Dict] = field(default_factory=list)
    
    # Rewards
    pending_fin_rewards: Decimal = Decimal('0')
    pending_usdfin_rewards: Decimal = Decimal('0')
    total_rewards_claimed: Decimal = Decimal('0')
    last_reward_claim: Optional[datetime] = None
    
    # APY & Multipliers
    current_fin_apy: float = 8.0
    current_usdfin_apy: float = 4.0
    staking_tier_multiplier: float = 1.0
    loyalty_multiplier: float = 1.0
    activity_multiplier: float = 1.0
    
    # Staking Tier
    staking_tier: str = "Basic"
    tier_benefits: List[str] = field(default_factory=list)
    mining_boost_percentage: float = 0.0
    xp_boost_percentage: float = 0.0
    rp_boost_percentage: float = 0.0
    
    # Lock Periods
    average_lock_period: int = 0  # days
    locked_amounts: Dict[str, Decimal] = field(default_factory=dict)
    unlock_schedule: List[Dict] = field(default_factory=list)
    
    # Statistics
    total_staking_sessions: int = 0
    longest_staking_period: int = 0  # days
    total_compounding_events: int = 0
    staking_start_date: Optional[datetime] = None

class FinovaAccountManager:
    """Core account management class for Finova ecosystem"""
    
    def __init__(self, rpc_client: AsyncClient, program_id: str):
        self.rpc = rpc_client
        self.program_id = PublicKey(program_id)
        self._cache = {}
        self._cache_ttl = 300  # 5 minutes
        
    # User Profile Management
    async def get_user_profile(self, user_id: str) -> Optional[UserProfile]:
        """Retrieve complete user profile"""
        cache_key = f"profile_{user_id}"
        if self._is_cached(cache_key):
            return self._cache[cache_key]['data']
            
        try:
            # Fetch from blockchain
            user_pda = self._derive_user_pda(user_id)
            account_info = await self.rpc.get_account_info(user_pda)
            
            if not account_info.value:
                return None
                
            # Decode account data
            profile_data = self._decode_user_account(account_info.value.data)
            profile = UserProfile(**profile_data)
            
            # Cache result
            self._cache[cache_key] = {
                'data': profile,
                'timestamp': time.time()
            }
            
            return profile
            
        except Exception as e:
            print(f"Error fetching user profile: {e}")
            return None
    
    async def create_user_profile(self, 
                                keypair: Keypair,
                                username: str,
                                email: Optional[str] = None,
                                referral_code: Optional[str] = None) -> Tuple[bool, str]:
        """Create new user profile"""
        try:
            user_id = str(keypair.public_key)
            
            # Generate unique referral code
            unique_referral_code = self._generate_referral_code(username)
            
            # Derive PDAs
            user_pda = self._derive_user_pda(user_id)
            mining_pda = self._derive_mining_pda(user_id)
            xp_pda = self._derive_xp_pda(user_id)
            rp_pda = self._derive_rp_pda(user_id)
            
            # Build transaction
            tx = Transaction()
            
            # Add create user instruction
            create_user_ix = self._build_create_user_instruction(
                user_pda=user_pda,
                mining_pda=mining_pda,
                xp_pda=xp_pda,
                rp_pda=rp_pda,
                user_keypair=keypair,
                username=username,
                email=email,
                referral_code=unique_referral_code,
                referred_by=referral_code
            )
            tx.add(create_user_ix)
            
            # Sign and send
            tx.sign(keypair)
            result = await self.rpc.send_transaction(tx)
            
            if result.value:
                # Clear cache
                self._clear_user_cache(user_id)
                return True, str(result.value)
            else:
                return False, "Transaction failed"
                
        except Exception as e:
            return False, f"Error creating profile: {e}"
    
    async def update_user_profile(self,
                                keypair: Keypair,
                                updates: Dict[str, Any]) -> Tuple[bool, str]:
        """Update user profile data"""
        try:
            user_id = str(keypair.public_key)
            user_pda = self._derive_user_pda(user_id)
            
            # Build update instruction
            update_ix = self._build_update_user_instruction(
                user_pda=user_pda,
                user_keypair=keypair,
                updates=updates
            )
            
            tx = Transaction()
            tx.add(update_ix)
            tx.sign(keypair)
            
            result = await self.rpc.send_transaction(tx)
            
            if result.value:
                self._clear_user_cache(user_id)
                return True, str(result.value)
            else:
                return False, "Update failed"
                
        except Exception as e:
            return False, f"Error updating profile: {e}"
    
    # Mining Account Management
    async def get_mining_account(self, user_id: str) -> Optional[MiningAccount]:
        """Get mining account data"""
        cache_key = f"mining_{user_id}"
        if self._is_cached(cache_key):
            return self._cache[cache_key]['data']
            
        try:
            mining_pda = self._derive_mining_pda(user_id)
            account_info = await self.rpc.get_account_info(mining_pda)
            
            if not account_info.value:
                return None
                
            mining_data = self._decode_mining_account(account_info.value.data)
            mining_account = MiningAccount(**mining_data)
            
            self._cache[cache_key] = {
                'data': mining_account,
                'timestamp': time.time()
            }
            
            return mining_account
            
        except Exception as e:
            print(f"Error fetching mining account: {e}")
            return None
    
    async def start_mining(self, keypair: Keypair) -> Tuple[bool, str]:
        """Start mining session"""
        try:
            user_id = str(keypair.public_key)
            user_pda = self._derive_user_pda(user_id)
            mining_pda = self._derive_mining_pda(user_id)
            
            # Calculate current mining rate
            mining_rate = await self._calculate_mining_rate(user_id)
            
            # Build mining instruction
            start_mining_ix = self._build_start_mining_instruction(
                user_pda=user_pda,
                mining_pda=mining_pda,
                user_keypair=keypair,
                mining_rate=mining_rate
            )
            
            tx = Transaction()
            tx.add(start_mining_ix)
            tx.sign(keypair)
            
            result = await self.rpc.send_transaction(tx)
            
            if result.value:
                self._clear_user_cache(user_id)
                return True, str(result.value)
            else:
                return False, "Mining start failed"
                
        except Exception as e:
            return False, f"Error starting mining: {e}"
    
    async def claim_mining_rewards(self, keypair: Keypair) -> Tuple[bool, str, Decimal]:
        """Claim pending mining rewards"""
        try:
            user_id = str(keypair.public_key)
            mining_account = await self.get_mining_account(user_id)
            
            if not mining_account or mining_account.pending_rewards <= 0:
                return False, "No rewards to claim", Decimal('0')
            
            user_pda = self._derive_user_pda(user_id)
            mining_pda = self._derive_mining_pda(user_id)
            token_account = self._derive_token_account(user_id)
            
            # Build claim instruction
            claim_ix = self._build_claim_rewards_instruction(
                user_pda=user_pda,
                mining_pda=mining_pda,
                token_account=token_account,
                user_keypair=keypair,
                amount=mining_account.pending_rewards
            )
            
            tx = Transaction()
            tx.add(claim_ix)
            tx.sign(keypair)
            
            result = await self.rpc.send_transaction(tx)
            
            if result.value:
                self._clear_user_cache(user_id)
                return True, str(result.value), mining_account.pending_rewards
            else:
                return False, "Claim failed", Decimal('0')
                
        except Exception as e:
            return False, f"Error claiming rewards: {e}", Decimal('0')
    
    # XP Account Management
    async def get_xp_account(self, user_id: str) -> Optional[XPAccount]:
        """Get XP account data"""
        cache_key = f"xp_{user_id}"
        if self._is_cached(cache_key):
            return self._cache[cache_key]['data']
            
        try:
            xp_pda = self._derive_xp_pda(user_id)
            account_info = await self.rpc.get_account_info(xp_pda)
            
            if not account_info.value:
                return None
                
            xp_data = self._decode_xp_account(account_info.value.data)
            xp_account = XPAccount(**xp_data)
            
            self._cache[cache_key] = {
                'data': xp_account,
                'timestamp': time.time()
            }
            
            return xp_account
            
        except Exception as e:
            print(f"Error fetching XP account: {e}")
            return None
    
    async def add_xp_activity(self,
                            keypair: Keypair,
                            activity_type: ActivityType,
                            platform: Platform,
                            content_data: Dict[str, Any]) -> Tuple[bool, str, int]:
        """Add XP-earning activity"""
        try:
            user_id = str(keypair.public_key)
            
            # Calculate XP gain
            xp_gained = await self._calculate_xp_gain(
                user_id, activity_type, platform, content_data
            )
            
            if xp_gained <= 0:
                return False, "No XP gained", 0
            
            user_pda = self._derive_user_pda(user_id)
            xp_pda = self._derive_xp_pda(user_id)
            
            # Build XP instruction
            add_xp_ix = self._build_add_xp_instruction(
                user_pda=user_pda,
                xp_pda=xp_pda,
                user_keypair=keypair,
                activity_type=activity_type.value,
                platform=platform.value[0],
                xp_amount=xp_gained,
                content_data=content_data
            )
            
            tx = Transaction()
            tx.add(add_xp_ix)
            tx.sign(keypair)
            
            result = await self.rpc.send_transaction(tx)
            
            if result.value:
                self._clear_user_cache(user_id)
                return True, str(result.value), xp_gained
            else:
                return False, "XP addition failed", 0
                
        except Exception as e:
            return False, f"Error adding XP: {e}", 0
    
    # Utility Methods
    def _derive_user_pda(self, user_id: str) -> PublicKey:
        """Derive user account PDA"""
        return PublicKey.find_program_address(
            [b"user", user_id.encode()], self.program_id
        )[0]
    
    def _derive_mining_pda(self, user_id: str) -> PublicKey:
        """Derive mining account PDA"""
        return PublicKey.find_program_address(
            [b"mining", user_id.encode()], self.program_id
        )[0]
    
    def _derive_xp_pda(self, user_id: str) -> PublicKey:
        """Derive XP account PDA"""
        return PublicKey.find_program_address(
            [b"xp", user_id.encode()], self.program_id
        )[0]
    
    def _derive_rp_pda(self, user_id: str) -> PublicKey:
        """Derive RP account PDA"""
        return PublicKey.find_program_address(
            [b"rp", user_id.encode()], self.program_id
        )[0]
    
    def _derive_token_account(self, user_id: str) -> PublicKey:
        """Derive token account PDA"""
        return PublicKey.find_program_address(
            [b"token", user_id.encode()], self.program_id
        )[0]
    
    def _generate_referral_code(self, username: str) -> str:
        """Generate unique referral code"""
        timestamp = str(int(time.time()))
        raw_data = f"{username}_{timestamp}".encode()
        hash_digest = hashlib.sha256(raw_data).hexdigest()
        return f"FIN_{hash_digest[:8].upper()}"
    
    def _is_cached(self, key: str) -> bool:
        """Check if data is cached and valid"""
        if key not in self._cache:
            return False
        return (time.time() - self._cache[key]['timestamp']) < self._cache_ttl
    
    def _clear_user_cache(self, user_id: str):
        """Clear all cached data for user"""
        keys_to_remove = []
        for key in self._cache:
            if user_id in key:
                keys_to_remove.append(key)
        for key in keys_to_remove:
            del self._cache[key]
            