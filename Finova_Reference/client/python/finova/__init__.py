# finova-net/finova/client/python/finova/__init__.py

"""
Finova Network Python SDK

A comprehensive Python client for interacting with the Finova Network ecosystem.
Supports mining, XP system, referral points, NFTs, staking, and DeFi operations.

Version: 1.0.0
Author: Finova Network Team
License: MIT
"""

__version__ = "1.0.0"
__author__ = "Finova Network Team"
__email__ = "dev@finova.network"
__license__ = "MIT"

# Core imports
from .client import FinovaClient
from .accounts import (
    UserAccount,
    MiningAccount, 
    StakingAccount,
    NFTAccount,
    ReferralAccount
)
from .instructions import (
    MiningInstructions,
    StakingInstructions,
    XPInstructions,
    ReferralInstructions,
    NFTInstructions,
    DeFiInstructions
)
from .types import (
    MiningData,
    XPActivity,
    ReferralData,
    StakingInfo,
    NFTMetadata,
    UserProfile,
    NetworkStats,
    RewardCalculation
)
from .utils import (
    calculate_mining_rate,
    calculate_xp_multiplier,
    calculate_rp_value,
    validate_activity,
    format_token_amount,
    encrypt_sensitive_data,
    verify_signature
)

# Constants
from .constants import (
    FINOVA_PROGRAM_ID,
    TOKEN_PROGRAM_ID,
    NFT_PROGRAM_ID,
    DEFI_PROGRAM_ID,
    BRIDGE_PROGRAM_ID,
    ORACLE_PROGRAM_ID,
    
    # Network configurations
    SOLANA_MAINNET_RPC,
    SOLANA_DEVNET_RPC,
    SOLANA_TESTNET_RPC,
    
    # Mining constants
    BASE_MINING_RATE,
    MAX_DAILY_MINING,
    PIONEER_BONUS_THRESHOLD,
    REGRESSION_FACTOR,
    
    # XP system constants
    XP_LEVELS,
    XP_MULTIPLIERS,
    QUALITY_SCORE_RANGE,
    PLATFORM_MULTIPLIERS,
    
    # Referral constants
    RP_TIERS,
    REFERRAL_BONUSES,
    NETWORK_CAPS,
    
    # Token economics
    TOTAL_SUPPLY,
    DECIMALS,
    STAKING_APYS,
    
    # API endpoints
    API_BASE_URL,
    WS_BASE_URL,
    AI_SERVICES_URL
)

# Exception classes
class FinovaError(Exception):
    """Base exception for Finova SDK"""
    pass

class FinovaAPIError(FinovaError):
    """API-related errors"""
    def __init__(self, message, status_code=None, response_data=None):
        super().__init__(message)
        self.status_code = status_code
        self.response_data = response_data

class FinovaValidationError(FinovaError):
    """Validation-related errors"""
    pass

class FinovaNetworkError(FinovaError):
    """Network-related errors"""
    pass

class FinovaSecurityError(FinovaError):
    """Security-related errors"""
    pass

# Main SDK factory function
def create_client(
    rpc_url=None,
    api_key=None,
    private_key=None,
    network="mainnet",
    timeout=30,
    max_retries=3,
    enable_logging=True,
    user_agent=None
):
    """
    Create a Finova Network client instance.
    
    Args:
        rpc_url (str, optional): Solana RPC endpoint URL
        api_key (str, optional): Finova API key for enhanced features
        private_key (str, optional): Wallet private key for transactions
        network (str): Network to connect to ('mainnet', 'devnet', 'testnet')
        timeout (int): Request timeout in seconds
        max_retries (int): Maximum number of retry attempts
        enable_logging (bool): Enable SDK logging
        user_agent (str, optional): Custom user agent string
        
    Returns:
        FinovaClient: Configured client instance
        
    Example:
        >>> from finova import create_client
        >>> client = create_client(
        ...     api_key="your_api_key",
        ...     private_key="your_private_key",
        ...     network="mainnet"
        ... )
        >>> user_data = client.get_user_profile("user_wallet_address")
        >>> mining_rate = client.calculate_current_mining_rate(user_data)
    """
    return FinovaClient(
        rpc_url=rpc_url,
        api_key=api_key,
        private_key=private_key,
        network=network,
        timeout=timeout,
        max_retries=max_retries,
        enable_logging=enable_logging,
        user_agent=user_agent or f"finova-python-sdk/{__version__}"
    )

# Convenience functions for common operations
def quick_mining_calculation(
    user_level=1,
    referral_count=0,
    total_holdings=0,
    kyc_verified=False,
    network_size=50000
):
    """
    Quick mining rate calculation without full client setup.
    
    Args:
        user_level (int): User's XP level
        referral_count (int): Number of active referrals
        total_holdings (float): User's total $FIN holdings
        kyc_verified (bool): KYC verification status
        network_size (int): Total network size
        
    Returns:
        dict: Mining calculation results
    """
    from .utils import calculate_mining_rate
    
    return calculate_mining_rate(
        user_level=user_level,
        referral_count=referral_count,
        total_holdings=total_holdings,
        kyc_verified=kyc_verified,
        network_size=network_size
    )

def quick_xp_calculation(
    activity_type="post",
    platform="instagram", 
    content_quality=1.0,
    user_level=1,
    streak_days=0
):
    """
    Quick XP calculation for activities.
    
    Args:
        activity_type (str): Type of activity ('post', 'comment', 'share', etc.)
        platform (str): Social media platform
        content_quality (float): AI-assessed quality score (0.5-2.0)
        user_level (int): User's current level
        streak_days (int): Current streak length
        
    Returns:
        dict: XP calculation results
    """
    from .utils import calculate_xp_multiplier
    
    return calculate_xp_multiplier(
        activity_type=activity_type,
        platform=platform,
        content_quality=content_quality,
        user_level=user_level,
        streak_days=streak_days
    )

def quick_rp_calculation(
    direct_referrals=0,
    l2_network_size=0,
    l3_network_size=0,
    network_quality=0.8,
    total_network_size=None
):
    """
    Quick Referral Points calculation.
    
    Args:
        direct_referrals (int): Number of direct referrals
        l2_network_size (int): Level 2 network size
        l3_network_size (int): Level 3 network size
        network_quality (float): Network quality score (0.0-1.0)
        total_network_size (int, optional): Total network size for regression
        
    Returns:
        dict: RP calculation results
    """
    from .utils import calculate_rp_value
    
    return calculate_rp_value(
        direct_referrals=direct_referrals,
        l2_network_size=l2_network_size,
        l3_network_size=l3_network_size,
        network_quality=network_quality,
        total_network_size=total_network_size
    )

# Utility functions for token formatting
def format_fin_amount(amount, decimals=6):
    """Format $FIN token amount for display."""
    from .utils import format_token_amount
    return format_token_amount(amount, decimals)

def parse_fin_amount(amount_str):
    """Parse $FIN amount string to float."""
    try:
        return float(amount_str.replace(',', '').replace(' $FIN', '').replace('$FIN', ''))
    except (ValueError, AttributeError):
        raise FinovaValidationError(f"Invalid $FIN amount format: {amount_str}")

# SDK health check
def sdk_health_check():
    """
    Perform SDK health check and return status information.
    
    Returns:
        dict: Health check results
    """
    import sys
    import platform
    from datetime import datetime
    
    try:
        # Test imports
        from solana.rpc.api import Client
        from solana.keypair import Keypair
        solana_available = True
    except ImportError:
        solana_available = False
    
    try:
        import requests
        requests_available = True
    except ImportError:
        requests_available = False
    
    return {
        'sdk_version': __version__,
        'python_version': sys.version,
        'platform': platform.platform(),
        'timestamp': datetime.utcnow().isoformat(),
        'dependencies': {
            'solana': solana_available,
            'requests': requests_available
        },
        'status': 'healthy' if solana_available and requests_available else 'degraded'
    }

# Export all public APIs
__all__ = [
    # Core client
    'FinovaClient',
    'create_client',
    
    # Account types
    'UserAccount',
    'MiningAccount', 
    'StakingAccount',
    'NFTAccount',
    'ReferralAccount',
    
    # Instruction builders
    'MiningInstructions',
    'StakingInstructions', 
    'XPInstructions',
    'ReferralInstructions',
    'NFTInstructions',
    'DeFiInstructions',
    
    # Data types
    'MiningData',
    'XPActivity',
    'ReferralData', 
    'StakingInfo',
    'NFTMetadata',
    'UserProfile',
    'NetworkStats',
    'RewardCalculation',
    
    # Utility functions
    'calculate_mining_rate',
    'calculate_xp_multiplier',
    'calculate_rp_value',
    'validate_activity',
    'format_token_amount',
    'encrypt_sensitive_data',
    'verify_signature',
    
    # Quick calculation functions
    'quick_mining_calculation',
    'quick_xp_calculation', 
    'quick_rp_calculation',
    
    # Token utilities
    'format_fin_amount',
    'parse_fin_amount',
    
    # Health check
    'sdk_health_check',
    
    # Exceptions
    'FinovaError',
    'FinovaAPIError',
    'FinovaValidationError', 
    'FinovaNetworkError',
    'FinovaSecurityError',
    
    # Constants (re-exported for convenience)
    'FINOVA_PROGRAM_ID',
    'BASE_MINING_RATE',
    'XP_LEVELS',
    'RP_TIERS',
    'TOTAL_SUPPLY',
    'API_BASE_URL',
    
    # Version info
    '__version__',
    '__author__',
    '__license__'
]

# SDK initialization logging
import logging
logger = logging.getLogger(__name__)
logger.info(f"Finova Network Python SDK v{__version__} initialized")

# Optional: Auto-configure logging if not already configured
if not logging.getLogger().handlers:
    logging.basicConfig(
        level=logging.INFO,
        format='%(asctime)s - %(name)s - %(levelname)s - %(message)s',
        datefmt='%Y-%m-%d %H:%M:%S'
    )
    