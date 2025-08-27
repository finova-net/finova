# finova-net/finova/client/python/finova/types.py

"""
Finova Network Python Client - Type Definitions
Enterprise-grade type system for the Finova social mining ecosystem
"""

from __future__ import annotations
from typing import Dict, List, Optional, Union, Any, Literal, TypedDict, Generic, TypeVar
from dataclasses import dataclass, field
from datetime import datetime, timedelta
from decimal import Decimal
from enum import Enum, IntEnum
import uuid

# Type Variables
T = TypeVar('T')
K = TypeVar('K') 
V = TypeVar('V')

# ============================================================================
# CORE BLOCKCHAIN TYPES
# ============================================================================

@dataclass(frozen=True)
class PublicKey:
    """Solana public key representation"""
    key: str
    
    def __post_init__(self):
        if len(self.key) != 44:  # Base58 encoded public key length
            raise ValueError("Invalid public key format")

@dataclass(frozen=True)
class TransactionSignature:
    """Solana transaction signature"""
    signature: str
    
    def __post_init__(self):
        if len(self.signature) not in [86, 87, 88]:  # Base58 signature lengths
            raise ValueError("Invalid transaction signature format")

@dataclass
class AccountMeta:
    """Solana account metadata"""
    pubkey: PublicKey
    is_signer: bool
    is_writable: bool

# ============================================================================
# USER MANAGEMENT TYPES
# ============================================================================

class UserStatus(Enum):
    """User account status"""
    PENDING = "pending"
    ACTIVE = "active"
    SUSPENDED = "suspended"
    BANNED = "banned"
    KYC_REQUIRED = "kyc_required"

class KYCStatus(Enum):
    """KYC verification status"""
    NOT_STARTED = "not_started"
    PENDING = "pending"
    APPROVED = "approved"
    REJECTED = "rejected"
    EXPIRED = "expired"

@dataclass
class UserProfile:
    """Core user profile data"""
    user_id: str
    wallet_address: PublicKey
    username: str
    email: Optional[str]
    phone: Optional[str]
    created_at: datetime
    updated_at: datetime
    status: UserStatus
    kyc_status: KYCStatus
    referral_code: str
    referred_by: Optional[str] = None
    last_active: Optional[datetime] = None
    human_verification_score: float = 0.0  # 0.0 - 1.0
    
    def is_verified(self) -> bool:
        return self.kyc_status == KYCStatus.APPROVED

@dataclass
class BiometricData:
    """Biometric verification data"""
    user_id: str
    face_encoding: bytes
    device_fingerprint: str
    verification_timestamp: datetime
    confidence_score: float  # 0.0 - 1.0
    is_verified: bool = False

# ============================================================================
# MINING SYSTEM TYPES
# ============================================================================

class MiningPhase(IntEnum):
    """Mining phases based on network growth"""
    FINIZEN = 1      # 0-100K users
    GROWTH = 2       # 100K-1M users  
    MATURITY = 3     # 1M-10M users
    STABILITY = 4    # 10M+ users

@dataclass
class MiningRate:
    """Dynamic mining rate calculation"""
    base_rate: Decimal
    finizen_bonus: float
    referral_bonus: float
    security_bonus: float
    regression_factor: float
    phase: MiningPhase
    effective_rate: Decimal = field(init=False)
    
    def __post_init__(self):
        self.effective_rate = (
            self.base_rate * 
            self.finizen_bonus * 
            self.referral_bonus * 
            self.security_bonus * 
            self.regression_factor
        )

@dataclass
class MiningSession:
    """Individual mining session data"""
    session_id: str
    user_id: str
    start_time: datetime
    duration: timedelta
    base_mined: Decimal
    bonuses_applied: Dict[str, float]
    total_mined: Decimal
    transaction_signature: Optional[TransactionSignature] = None
    is_active: bool = True

@dataclass
class MiningStats:
    """User mining statistics"""
    user_id: str
    total_mined: Decimal
    current_rate: Decimal
    daily_mined: Decimal
    weekly_mined: Decimal
    monthly_mined: Decimal
    mining_streak: int
    last_mining_session: Optional[datetime]
    total_sessions: int
    regression_factor: float

# ============================================================================
# EXPERIENCE POINTS (XP) SYSTEM
# ============================================================================

class ActivityType(Enum):
    """Types of social media activities"""
    ORIGINAL_POST = "original_post"
    PHOTO_POST = "photo_post"
    VIDEO_POST = "video_post"
    STORY_POST = "story_post"
    COMMENT = "comment"
    LIKE_REACT = "like_react"
    SHARE_REPOST = "share_repost"
    FOLLOW_SUBSCRIBE = "follow_subscribe"
    DAILY_LOGIN = "daily_login"
    DAILY_QUEST = "daily_quest"
    MILESTONE = "milestone"
    VIRAL_CONTENT = "viral_content"

class SocialPlatform(Enum):
    """Supported social media platforms"""
    INSTAGRAM = "instagram"
    TIKTOK = "tiktok"
    YOUTUBE = "youtube"
    FACEBOOK = "facebook"
    TWITTER_X = "twitter_x"
    FINOVA_APP = "finova_app"

class XPTier(Enum):
    """XP level tiers with badges"""
    BRONZE = "bronze"      # 1-10
    SILVER = "silver"      # 11-25
    GOLD = "gold"          # 26-50
    PLATINUM = "platinum"  # 51-75
    DIAMOND = "diamond"    # 76-100
    MYTHIC = "mythic"      # 101+

@dataclass
class XPActivity:
    """Individual XP-earning activity"""
    activity_id: str
    user_id: str
    activity_type: ActivityType
    platform: SocialPlatform
    content_hash: Optional[str]
    base_xp: int
    quality_score: float  # 0.5 - 2.0
    platform_multiplier: float
    streak_bonus: float
    level_progression: float
    total_xp_gained: int
    timestamp: datetime
    metadata: Dict[str, Any] = field(default_factory=dict)

@dataclass
class XPLevel:
    """XP level information"""
    level: int
    tier: XPTier
    xp_required: int
    xp_current: int
    xp_to_next: int
    mining_multiplier: float
    daily_fin_cap: Decimal
    special_unlocks: List[str]
    badge_name: str

@dataclass
class XPStats:
    """User XP statistics"""
    user_id: str
    total_xp: int
    current_level: XPLevel
    daily_xp: int
    weekly_xp: int
    monthly_xp: int
    streak_days: int
    best_streak: int
    activities_today: int
    last_activity: Optional[datetime]

# ============================================================================
# REFERRAL POINTS (RP) SYSTEM
# ============================================================================

class RPTier(Enum):
    """Referral Points tier system"""
    EXPLORER = "explorer"        # 0-999 RP
    CONNECTOR = "connector"      # 1K-4.9K RP
    INFLUENCER = "influencer"    # 5K-14.9K RP
    LEADER = "leader"           # 15K-49.9K RP
    AMBASSADOR = "ambassador"    # 50K+ RP

@dataclass
class ReferralUser:
    """Referral network user data"""
    user_id: str
    referred_at: datetime
    referral_level: int  # L1, L2, L3, etc.
    is_active: bool
    last_activity: datetime
    total_contributed_rp: Decimal
    kyc_verified: bool

@dataclass
class ReferralNetwork:
    """User's referral network structure"""
    user_id: str
    direct_referrals: List[ReferralUser]  # L1
    indirect_referrals: Dict[int, List[ReferralUser]]  # L2, L3, etc.
    total_network_size: int
    active_network_size: int
    network_quality_score: float  # 0.0 - 1.0
    
    def get_level_referrals(self, level: int) -> List[ReferralUser]:
        if level == 1:
            return self.direct_referrals
        return self.indirect_referrals.get(level, [])

@dataclass
class RPCalculation:
    """Referral Points calculation breakdown"""
    direct_rp: Decimal
    indirect_rp: Decimal
    network_quality_bonus: Decimal
    regression_factor: float
    total_rp: Decimal
    tier: RPTier
    mining_bonus: float
    referral_bonus_percentage: float

@dataclass
class RPStats:
    """User RP statistics"""
    user_id: str
    total_rp: Decimal
    current_tier: RPTier
    network: ReferralNetwork
    calculation: RPCalculation
    daily_rp: Decimal
    weekly_rp: Decimal
    monthly_rp: Decimal

# ============================================================================
# TOKEN ECONOMICS TYPES
# ============================================================================

class TokenType(Enum):
    """Types of tokens in the ecosystem"""
    FIN = "FIN"              # Primary utility token
    SFIN = "sFIN"            # Staked FIN
    USDFIN = "USDfin"        # Synthetic stablecoin
    SUSDFIN = "sUSDfin"      # Staked USDfin

@dataclass
class TokenBalance:
    """Token balance information"""
    token_type: TokenType
    balance: Decimal
    staked_balance: Decimal
    pending_rewards: Decimal
    last_updated: datetime

@dataclass
class TokenomicsData:
    """Overall tokenomics information"""
    total_supply: Dict[TokenType, Decimal]
    circulating_supply: Dict[TokenType, Decimal]
    staked_supply: Dict[TokenType, Decimal]
    burn_rate: Dict[TokenType, Decimal]
    mint_rate: Dict[TokenType, Decimal]
    current_phase: MiningPhase
    total_users: int
    active_miners: int

# ============================================================================
# STAKING SYSTEM TYPES
# ============================================================================

class StakingTier(Enum):
    """Staking tiers based on amount"""
    BASIC = "basic"          # 100-499 FIN
    PREMIUM = "premium"      # 500-999 FIN
    VIP = "vip"             # 1K-4.9K FIN
    ELITE = "elite"         # 5K-9.9K FIN
    LEGENDARY = "legendary"  # 10K+ FIN

@dataclass
class StakingPosition:
    """Individual staking position"""
    position_id: str
    user_id: str
    staked_amount: Decimal
    stake_date: datetime
    tier: StakingTier
    base_apy: float
    multiplier_effects: Dict[str, float]
    effective_apy: float
    pending_rewards: Decimal
    total_earned: Decimal
    lock_period: Optional[timedelta] = None
    unlock_date: Optional[datetime] = None

@dataclass
class StakingRewards:
    """Staking rewards calculation"""
    base_rewards: Decimal
    xp_level_bonus: Decimal
    rp_tier_bonus: Decimal
    loyalty_bonus: Decimal
    activity_bonus: Decimal
    total_rewards: Decimal

@dataclass
class StakingStats:
    """User staking statistics"""
    user_id: str
    positions: List[StakingPosition]
    total_staked: Decimal
    current_tier: StakingTier
    total_rewards_earned: Decimal
    average_apy: float
    staking_duration: timedelta

# ============================================================================
# NFT & SPECIAL CARDS TYPES
# ============================================================================

class CardRarity(Enum):
    """NFT card rarity levels"""
    COMMON = "common"
    UNCOMMON = "uncommon"
    RARE = "rare"
    EPIC = "epic"
    LEGENDARY = "legendary"

class CardCategory(Enum):
    """Special card categories"""
    MINING_BOOST = "mining_boost"
    XP_ACCELERATOR = "xp_accelerator"
    REFERRAL_POWER = "referral_power"
    PROFILE_BADGE = "profile_badge"
    ACHIEVEMENT = "achievement"

@dataclass
class CardEffect:
    """Card effect definition"""
    effect_type: str
    multiplier: float
    duration: Optional[timedelta]
    max_uses: Optional[int]
    conditions: Dict[str, Any] = field(default_factory=dict)

@dataclass
class SpecialCard:
    """Special card NFT data"""
    card_id: str
    name: str
    description: str
    category: CardCategory
    rarity: CardRarity
    effect: CardEffect
    price_fin: Decimal
    image_uri: str
    metadata_uri: str
    mint_address: PublicKey
    total_supply: int
    current_supply: int
    created_at: datetime

@dataclass
class UserCard:
    """User-owned card instance"""
    instance_id: str
    user_id: str
    card: SpecialCard
    acquired_at: datetime
    uses_remaining: Optional[int]
    is_active: bool
    activation_time: Optional[datetime]
    expiry_time: Optional[datetime]

@dataclass
class CardSynergy:
    """Card synergy effects"""
    active_cards: List[UserCard]
    synergy_multiplier: float
    rarity_bonus: float
    type_match_bonus: float
    total_multiplier: float

# ============================================================================
# GUILD SYSTEM TYPES
# ============================================================================

class GuildRole(Enum):
    """Guild member roles"""
    MEMBER = "member"
    OFFICER = "officer"
    LEADER = "leader"
    MASTER = "master"

class GuildCompetitionType(Enum):
    """Types of guild competitions"""
    DAILY_CHALLENGE = "daily_challenge"
    WEEKLY_WAR = "weekly_war"
    MONTHLY_CHAMPIONSHIP = "monthly_championship"
    SEASONAL_LEAGUE = "seasonal_league"

@dataclass
class GuildMember:
    """Guild member information"""
    user_id: str
    username: str
    role: GuildRole
    joined_at: datetime
    contribution_score: int
    last_active: datetime
    xp_level: int
    mining_rate: Decimal

@dataclass
class Guild:
    """Guild information"""
    guild_id: str
    name: str
    description: str
    master_id: str
    created_at: datetime
    member_count: int
    max_members: int
    members: List[GuildMember]
    total_power: int
    guild_level: int
    treasury_balance: Decimal
    current_competitions: List[str]

@dataclass
class GuildCompetition:
    """Guild competition data"""
    competition_id: str
    name: str
    competition_type: GuildCompetitionType
    start_time: datetime
    end_time: datetime
    participating_guilds: List[str]
    rewards: Dict[str, Decimal]
    leaderboard: List[Dict[str, Any]]
    is_active: bool

# ============================================================================
# ANTI-BOT & SECURITY TYPES
# ============================================================================

class SecurityLevel(Enum):
    """Security verification levels"""
    NONE = "none"
    BASIC = "basic"
    MODERATE = "moderate"  
    HIGH = "high"
    MAXIMUM = "maximum"

@dataclass
class BehaviorPattern:
    """User behavior analysis data"""
    user_id: str
    click_patterns: Dict[str, float]
    session_patterns: Dict[str, float]
    temporal_patterns: Dict[str, float]
    content_patterns: Dict[str, float]
    network_patterns: Dict[str, float]
    human_probability: float  # 0.0 - 1.0
    risk_score: float  # 0.0 - 1.0
    last_analysis: datetime

@dataclass
class SecurityCheck:
    """Security verification result"""
    user_id: str
    check_type: str
    passed: bool
    confidence: float
    details: Dict[str, Any]
    timestamp: datetime
    expires_at: Optional[datetime]

@dataclass
class AntiBot:
    """Anti-bot system data"""
    user_id: str
    behavior_pattern: BehaviorPattern
    security_checks: List[SecurityCheck]
    current_level: SecurityLevel
    verification_required: bool
    penalty_factor: float  # Mining reduction factor
    last_verification: Optional[datetime]

# ============================================================================
# GOVERNANCE & DAO TYPES
# ============================================================================

class ProposalType(Enum):
    """Types of governance proposals"""
    PARAMETER_CHANGE = "parameter_change"
    FEATURE_ADDITION = "feature_addition"
    TREASURY_ALLOCATION = "treasury_allocation"
    COMMUNITY_INITIATIVE = "community_initiative"
    EMERGENCY_ACTION = "emergency_action"

class ProposalStatus(Enum):
    """Governance proposal status"""
    DRAFT = "draft"
    ACTIVE = "active"
    PASSED = "passed"
    REJECTED = "rejected"
    EXECUTED = "executed"
    CANCELLED = "cancelled"

@dataclass
class VotingPower:
    """User's voting power calculation"""
    user_id: str
    staked_sfin: Decimal
    xp_level_multiplier: float
    rp_reputation_score: float
    activity_weight: float
    total_voting_power: Decimal

@dataclass
class GovernanceProposal:
    """DAO governance proposal"""
    proposal_id: str
    title: str
    description: str
    proposal_type: ProposalType
    proposer_id: str
    created_at: datetime
    voting_start: datetime
    voting_end: datetime
    status: ProposalStatus
    votes_for: Decimal
    votes_against: Decimal
    total_voting_power: Decimal
    execution_data: Optional[Dict[str, Any]]

# ============================================================================
# API RESPONSE TYPES
# ============================================================================

class APIStatus(Enum):
    """API response status"""
    SUCCESS = "success"
    ERROR = "error"
    WARNING = "warning"

@dataclass
class APIResponse(Generic[T]):
    """Generic API response wrapper"""
    status: APIStatus
    data: Optional[T]
    message: str
    timestamp: datetime
    request_id: str
    errors: List[str] = field(default_factory=list)

# Specific API response types
class UserResponse(TypedDict):
    user: UserProfile
    mining_stats: MiningStats
    xp_stats: XPStats
    rp_stats: RPStats
    token_balances: List[TokenBalance]

class ActivityResponse(TypedDict):
    activity: XPActivity
    xp_gained: int
    mining_boost: float
    new_level: Optional[XPLevel]

class MiningResponse(TypedDict):
    session: MiningSession
    total_mined: Decimal
    current_rate: Decimal
    bonuses: Dict[str, float]

class StakingResponse(TypedDict):
    position: StakingPosition
    rewards: StakingRewards
    new_tier: Optional[StakingTier]

# ============================================================================
# EVENT TYPES
# ============================================================================

class EventType(Enum):
    """System event types"""
    USER_REGISTERED = "user_registered"
    USER_VERIFIED = "user_verified"
    MINING_SESSION_STARTED = "mining_session_started"
    MINING_SESSION_ENDED = "mining_session_ended"
    XP_GAINED = "xp_gained"
    LEVEL_UP = "level_up"
    RP_EARNED = "rp_earned"
    TIER_UPGRADED = "tier_upgraded"
    CARD_ACQUIRED = "card_acquired"
    CARD_USED = "card_used"
    STAKING_POSITION_CREATED = "staking_position_created"
    REWARDS_CLAIMED = "rewards_claimed"
    GUILD_JOINED = "guild_joined"
    PROPOSAL_CREATED = "proposal_created"
    VOTE_CAST = "vote_cast"

@dataclass
class SystemEvent:
    """System event data"""
    event_id: str
    event_type: EventType
    user_id: Optional[str]
    data: Dict[str, Any]
    timestamp: datetime
    block_height: Optional[int]
    transaction_signature: Optional[TransactionSignature]

# ============================================================================
# UTILITY TYPES
# ============================================================================

@dataclass
class PaginationParams:
    """Pagination parameters"""
    page: int = 1
    limit: int = 20
    offset: int = field(init=False)
    
    def __post_init__(self):
        self.offset = (self.page - 1) * self.limit

@dataclass
class PaginatedResponse(Generic[T]):
    """Paginated response wrapper"""
    items: List[T]
    total: int
    page: int
    limit: int
    has_next: bool
    has_prev: bool

@dataclass
class FilterParams:
    """Generic filter parameters"""
    start_date: Optional[datetime] = None
    end_date: Optional[datetime] = None
    status: Optional[str] = None
    category: Optional[str] = None
    min_amount: Optional[Decimal] = None
    max_amount: Optional[Decimal] = None
    additional_filters: Dict[str, Any] = field(default_factory=dict)

# ============================================================================
# CONFIGURATION TYPES
# ============================================================================

@dataclass
class NetworkConfig:
    """Blockchain network configuration"""
    cluster_url: str
    commitment: str
    program_ids: Dict[str, PublicKey]
    token_addresses: Dict[TokenType, PublicKey]

@dataclass
class ClientConfig:
    """Client configuration"""
    api_base_url: str
    websocket_url: str
    network: NetworkConfig
    timeout: int = 30
    max_retries: int = 3
    retry_delay: float = 1.0

# ============================================================================
# ERROR TYPES
# ============================================================================

class FinovaError(Exception):
    """Base Finova client error"""
    def __init__(self, message: str, code: Optional[str] = None):
        self.message = message
        self.code = code
        super().__init__(message)

class NetworkError(FinovaError):
    """Network-related errors"""
    pass

class ValidationError(FinovaError):
    """Data validation errors"""
    pass

class AuthenticationError(FinovaError):
    """Authentication-related errors"""
    pass

class InsufficientFundsError(FinovaError):
    """Insufficient balance errors"""
    pass

class RateLimitError(FinovaError):
    """Rate limiting errors"""
    pass

# Export all types for easy importing
__all__ = [
    # Core types
    'PublicKey', 'TransactionSignature', 'AccountMeta',
    
    # User types
    'UserStatus', 'KYCStatus', 'UserProfile', 'BiometricData',
    
    # Mining types
    'MiningPhase', 'MiningRate', 'MiningSession', 'MiningStats',
    
    # XP types
    'ActivityType', 'SocialPlatform', 'XPTier', 'XPActivity', 'XPLevel', 'XPStats',
    
    # RP types
    'RPTier', 'ReferralUser', 'ReferralNetwork', 'RPCalculation', 'RPStats',
    
    # Token types
    'TokenType', 'TokenBalance', 'TokenomicsData',
    
    # Staking types
    'StakingTier', 'StakingPosition', 'StakingRewards', 'StakingStats',
    
    # NFT types
    'CardRarity', 'CardCategory', 'CardEffect', 'SpecialCard', 'UserCard', 'CardSynergy',
    
    # Guild types
    'GuildRole', 'GuildCompetitionType', 'GuildMember', 'Guild', 'GuildCompetition',
    
    # Security types
    'SecurityLevel', 'BehaviorPattern', 'SecurityCheck', 'AntiBot',
    
    # Governance types
    'ProposalType', 'ProposalStatus', 'VotingPower', 'GovernanceProposal',
    
    # API types
    'APIStatus', 'APIResponse', 'UserResponse', 'ActivityResponse', 'MiningResponse', 'StakingResponse',
    
    # Event types
    'EventType', 'SystemEvent',
    
    # Utility types
    'PaginationParams', 'PaginatedResponse', 'FilterParams',
    
    # Config types
    'NetworkConfig', 'ClientConfig',
    
    # Error types
    'FinovaError', 'NetworkError', 'ValidationError', 'AuthenticationError', 
    'InsufficientFundsError', 'RateLimitError'
]
