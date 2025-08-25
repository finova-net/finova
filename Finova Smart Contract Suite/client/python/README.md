# Finova Network Python SDK

[![Version](https://img.shields.io/badge/version-1.0.0-blue.svg)](https://pypi.org/project/finova-sdk/)
[![Python](https://img.shields.io/badge/python-3.8+-green.svg)](https://python.org)
[![License](https://img.shields.io/badge/license-MIT-yellow.svg)](LICENSE)
[![Build](https://img.shields.io/badge/build-passing-brightgreen.svg)](https://github.com/finova-network/finova-contracts)

Official Python SDK for the Finova Network Social-Fi Super App. Mine $FIN tokens, earn XP, build referral networks, and engage with the complete Finova ecosystem through a simple, powerful Python interface.

## 🚀 Quick Start

```bash
pip install finova-sdk
```

```python
from finova import FinovaClient
import asyncio

async def main():
    # Initialize client
    client = FinovaClient(
        rpc_url="https://api.mainnet-beta.solana.com",
        private_key="your_private_key_here"
    )
    
    # Start mining $FIN tokens
    mining_result = await client.mining.start_mining()
    print(f"Mining started! Rate: {mining_result.rate} $FIN/hour")
    
    # Post content and earn XP
    xp_result = await client.xp.post_content(
        platform="instagram",
        content_type="image",
        content_url="https://example.com/post",
        description="My amazing content!"
    )
    print(f"Earned {xp_result.xp_gained} XP!")
    
    # Build referral network
    referral = await client.referral.create_referral_code()
    print(f"Your referral code: {referral.code}")

if __name__ == "__main__":
    asyncio.run(main())
```

## 📋 Table of Contents

- [Installation](#installation)
- [Authentication](#authentication)
- [Core Features](#core-features)
  - [Mining System](#mining-system)
  - [XP System](#xp-system)
  - [Referral System](#referral-system)
  - [NFT & Special Cards](#nft--special-cards)
  - [Staking](#staking)
  - [Social Integration](#social-integration)
- [API Reference](#api-reference)
- [Examples](#examples)
- [Error Handling](#error-handling)
- [Configuration](#configuration)
- [Contributing](#contributing)

## 🔧 Installation

### Prerequisites

- Python 3.8 or higher
- Solana CLI tools (optional, for advanced features)
- Active Finova Network account

### Install from PyPI

```bash
pip install finova-sdk
```

### Install from Source

```bash
git clone https://github.com/finova-network/finova-contracts.git
cd finova-contracts/client/python
pip install -e .
```

### Development Installation

```bash
git clone https://github.com/finova-network/finova-contracts.git
cd finova-contracts/client/python
pip install -e ".[dev]"
```

## 🔐 Authentication

### Environment Setup

Create a `.env` file:

```env
FINOVA_RPC_URL=https://api.mainnet-beta.solana.com
FINOVA_PRIVATE_KEY=your_solana_private_key
FINOVA_API_KEY=your_finova_api_key
FINOVA_ENVIRONMENT=mainnet
```

### Client Initialization

```python
from finova import FinovaClient
from finova.auth import WalletAuth, APIAuth

# Method 1: Wallet Authentication
client = FinovaClient(
    auth=WalletAuth(
        private_key="your_private_key",
        rpc_url="https://api.mainnet-beta.solana.com"
    )
)

# Method 2: API Key Authentication
client = FinovaClient(
    auth=APIAuth(
        api_key="your_api_key",
        base_url="https://api.finova.network"
    )
)

# Method 3: Environment Variables
client = FinovaClient.from_env()
```

## 🌟 Core Features

### Mining System

The mining system implements Pi Network-inspired exponential regression with Finova-specific enhancements.

#### Start Mining

```python
# Start mining with automatic optimization
mining_session = await client.mining.start_mining(
    optimize_for="balanced",  # "speed", "efficiency", "balanced"
    auto_claim=True
)

print(f"Mining Rate: {mining_session.hourly_rate} $FIN/hour")
print(f"Phase: {mining_session.phase}")
print(f"Finizen Bonus: {mining_session.finizen_bonus}x")
```

#### Monitor Mining Progress

```python
# Get current mining status
status = await client.mining.get_status()
print(f"Total Mined: {status.total_mined} $FIN")
print(f"Pending Rewards: {status.pending_rewards} $FIN")
print(f"Next Claim: {status.next_claim_time}")

# Get detailed mining analytics
analytics = await client.mining.get_analytics(period="7d")
print(f"Average Rate: {analytics.avg_hourly_rate} $FIN/hour")
print(f"Best Day: {analytics.best_day_rate} $FIN/day")
print(f"Efficiency Score: {analytics.efficiency_score}")
```

#### Advanced Mining Features

```python
# Apply special mining cards
await client.mining.use_special_card(
    card_id="double_mining_24h",
    duration_hours=24
)

# Optimize mining based on activity patterns
optimization = await client.mining.optimize_schedule(
    activity_pattern="night_owl",  # "early_bird", "consistent", "random"
    timezone="Asia/Jakarta"
)

# Set mining goals and tracking
await client.mining.set_goals(
    daily_target=10.0,  # $FIN per day
    monthly_target=300.0,
    auto_adjust=True
)
```

### XP System

Hamster Kombat-inspired gamified progression system with exponential scaling.

#### Earn XP from Social Activities

```python
# Post content across platforms
platforms = ["instagram", "tiktok", "youtube", "facebook", "twitter"]

for platform in platforms:
    xp_result = await client.xp.post_content(
        platform=platform,
        content_type="video",
        content_data={
            "title": "My Finova Journey",
            "description": "Earning while engaging!",
            "tags": ["finova", "socialfi", "crypto"],
            "url": f"https://{platform}.com/my_post"
        }
    )
    
    print(f"{platform}: +{xp_result.xp_gained} XP")
    print(f"Quality Score: {xp_result.quality_score}")
    print(f"Current Level: {xp_result.current_level}")
```

#### Track XP Progress and Levels

```python
# Get comprehensive XP status
xp_status = await client.xp.get_status()
print(f"Current XP: {xp_status.total_xp:,}")
print(f"Level: {xp_status.level} ({xp_status.level_name})")
print(f"Progress to Next: {xp_status.progress_percent}%")
print(f"Mining Multiplier: {xp_status.mining_multiplier}x")

# Get XP leaderboard position
leaderboard = await client.xp.get_leaderboard(
    timeframe="weekly",
    category="content_creators"
)
print(f"Your Rank: #{leaderboard.user_rank}")
print(f"Top 10: {leaderboard.top_users}")
```

#### XP Optimization Strategies

```python
# Get personalized XP recommendations
recommendations = await client.xp.get_recommendations()
for rec in recommendations:
    print(f"Activity: {rec.activity}")
    print(f"Potential XP: {rec.estimated_xp}")
    print(f"Time Investment: {rec.time_required}")
    print(f"Success Probability: {rec.success_rate}%")

# Analyze XP earning patterns
analysis = await client.xp.analyze_patterns(period="30d")
print(f"Best Platform: {analysis.top_platform}")
print(f"Peak Hours: {analysis.peak_hours}")
print(f"Content Type Performance: {analysis.content_performance}")
```

### Referral System

Build exponential value through authentic network growth with anti-abuse mechanisms.

#### Create and Manage Referrals

```python
# Generate custom referral code
referral_code = await client.referral.create_code(
    custom_code="FINOVA2025",  # Optional custom code
    max_uses=100,
    expiry_days=30
)
print(f"Referral Code: {referral_code.code}")
print(f"Referral Link: {referral_code.share_url}")

# Track referral performance
referral_stats = await client.referral.get_stats()
print(f"Total Referrals: {referral_stats.total_count}")
print(f"Active Referrals: {referral_stats.active_count}")
print(f"RP Tier: {referral_stats.rp_tier}")
print(f"Network Bonus: {referral_stats.network_multiplier}x")
```

#### Network Analysis and Optimization

```python
# Analyze referral network health
network_analysis = await client.referral.analyze_network()
print(f"Network Quality Score: {network_analysis.quality_score}")
print(f"L1 Network: {network_analysis.l1_size} users")
print(f"L2 Network: {network_analysis.l2_size} users")
print(f"L3 Network: {network_analysis.l3_size} users")

# Get network growth recommendations
growth_tips = await client.referral.get_growth_tips()
for tip in growth_tips:
    print(f"Strategy: {tip.strategy}")
    print(f"Expected Impact: {tip.impact_score}")
    print(f"Implementation: {tip.steps}")
```

#### Advanced Referral Features

```python
# Set up referral campaigns
campaign = await client.referral.create_campaign(
    name="Holiday Special",
    bonus_multiplier=1.5,
    start_date="2025-12-01",
    end_date="2025-12-31",
    target_audience="content_creators"
)

# Track campaign performance
campaign_stats = await client.referral.get_campaign_stats(campaign.id)
print(f"Campaign Signups: {campaign_stats.signups}")
print(f"Conversion Rate: {campaign_stats.conversion_rate}%")
print(f"Bonus RP Earned: {campaign_stats.bonus_rp}")
```

### NFT & Special Cards

Hamster Kombat-inspired card system with functional utility and collectible value.

#### Browse and Purchase NFTs

```python
# Browse available special cards
available_cards = await client.nft.browse_cards(
    category="mining_boost",
    rarity=["rare", "epic"],
    price_range=(50, 500)  # $FIN price range
)

for card in available_cards:
    print(f"Card: {card.name}")
    print(f"Effect: {card.effect_description}")
    print(f"Duration: {card.duration}")
    print(f"Price: {card.price} $FIN")

# Purchase special card
purchase = await client.nft.purchase_card(
    card_id="triple_mining_12h",
    quantity=1,
    use_staked_discount=True
)
print(f"Purchase successful! Transaction: {purchase.transaction_id}")
```

#### Use Special Cards

```python
# Activate special cards for bonuses
activation = await client.nft.use_card(
    card_id="xp_double_24h",
    activation_time="immediate"  # or schedule for later
)
print(f"Card activated! Bonus active until: {activation.expires_at}")

# Check active card effects
active_effects = await client.nft.get_active_effects()
for effect in active_effects:
    print(f"Effect: {effect.type}")
    print(f"Multiplier: {effect.multiplier}x")
    print(f"Remaining: {effect.time_remaining}")
```

#### NFT Portfolio Management

```python
# View your NFT collection
collection = await client.nft.get_collection()
print(f"Total NFTs: {len(collection.items)}")
print(f"Portfolio Value: {collection.total_value} $FIN")

# Get collection analytics
analytics = await client.nft.get_portfolio_analytics()
print(f"ROI: {analytics.roi_percentage}%")
print(f"Best Performing: {analytics.top_performer}")
print(f"Usage Efficiency: {analytics.usage_efficiency}%")
```

### Staking

Liquid staking with auto-compounding and enhanced rewards integration.

#### Stake $FIN Tokens

```python
# Start staking with flexible options
staking_position = await client.staking.stake(
    amount=1000.0,  # $FIN amount
    duration="flexible",  # "30d", "90d", "180d", "365d", "flexible"
    auto_compound=True,
    stake_type="enhanced"  # includes XP/RP multipliers
)

print(f"Staked: {staking_position.amount} $FIN")
print(f"APY: {staking_position.apy}%")
print(f"Rewards Multiplier: {staking_position.multiplier}x")
```

#### Monitor Staking Rewards

```python
# Get staking status and rewards
staking_status = await client.staking.get_status()
print(f"Total Staked: {staking_status.total_staked} $FIN")
print(f"Pending Rewards: {staking_status.pending_rewards} $FIN")
print(f"Current APY: {staking_status.current_apy}%")

# Calculate optimal staking strategy
optimization = await client.staking.optimize_strategy(
    available_balance=5000.0,
    risk_tolerance="moderate",
    time_horizon="1_year"
)
print(f"Recommended Split: {optimization.recommendations}")
```

### Social Integration

Seamless connection with major social media platforms for automated XP and reward tracking.

#### Connect Social Accounts

```python
# Connect social media accounts
platforms_to_connect = ["instagram", "tiktok", "youtube"]

for platform in platforms_to_connect:
    auth_url = await client.social.get_auth_url(platform)
    print(f"Connect {platform}: {auth_url}")
    
    # After user authorization, complete connection
    connection = await client.social.complete_auth(
        platform=platform,
        auth_code="received_auth_code"
    )
    print(f"{platform} connected! Status: {connection.status}")
```

#### Automated Content Tracking

```python
# Enable automated content tracking
tracking_config = await client.social.configure_tracking(
    platforms=["instagram", "tiktok"],
    auto_claim_xp=True,
    quality_threshold=0.7,
    notification_settings={
        "new_content": True,
        "xp_earned": True,
        "milestones": True
    }
)

# Get social activity summary
activity_summary = await client.social.get_activity_summary(period="7d")
print(f"Posts Tracked: {activity_summary.posts_count}")
print(f"Total XP Earned: {activity_summary.total_xp}")
print(f"Best Performing Post: {activity_summary.top_post}")
```

## 📚 API Reference

### Core Client Methods

```python
class FinovaClient:
    def __init__(self, auth: AuthProvider, config: Optional[Config] = None)
    
    @property
    def mining(self) -> MiningService
    @property
    def xp(self) -> XPService
    @property
    def referral(self) -> ReferralService
    @property
    def nft(self) -> NFTService
    @property
    def staking(self) -> StakingService
    @property
    def social(self) -> SocialService
    @property
    def user(self) -> UserService
    @property
    def analytics(self) -> AnalyticsService
```

### Service Classes

#### MiningService

```python
class MiningService:
    async def start_mining(self, **options) -> MiningSession
    async def stop_mining(self) -> bool
    async def get_status(self) -> MiningStatus
    async def claim_rewards(self) -> ClaimResult
    async def get_analytics(self, period: str) -> MiningAnalytics
    async def use_special_card(self, card_id: str, **options) -> bool
    async def optimize_schedule(self, **preferences) -> OptimizationResult
```

#### XPService

```python
class XPService:
    async def post_content(self, platform: str, **content_data) -> XPResult
    async def get_status(self) -> XPStatus
    async def get_leaderboard(self, **filters) -> Leaderboard
    async def get_recommendations(self) -> List[XPRecommendation]
    async def analyze_patterns(self, period: str) -> XPAnalysis
```

#### ReferralService

```python
class ReferralService:
    async def create_code(self, **options) -> ReferralCode
    async def get_stats(self) -> ReferralStats
    async def analyze_network(self) -> NetworkAnalysis
    async def create_campaign(self, **campaign_data) -> Campaign
    async def get_growth_tips(self) -> List[GrowthTip]
```

## 🔥 Examples

### Complete Mining Bot

```python
import asyncio
from finova import FinovaClient
from finova.utils import calculate_optimal_timing

class FinovaMiningBot:
    def __init__(self, client: FinovaClient):
        self.client = client
        self.running = False
    
    async def start(self):
        self.running = True
        await self._mining_loop()
    
    async def stop(self):
        self.running = False
    
    async def _mining_loop(self):
        while self.running:
            try:
                # Check mining status
                status = await self.client.mining.get_status()
                
                # Auto-claim rewards when threshold reached
                if status.pending_rewards >= 10.0:
                    await self.client.mining.claim_rewards()
                    print(f"Claimed {status.pending_rewards} $FIN")
                
                # Optimize mining rate
                if status.efficiency < 0.8:
                    await self.client.mining.optimize_schedule()
                
                # Use special cards when beneficial
                available_cards = await self.client.nft.get_available_cards()
                best_card = self._select_best_card(available_cards, status)
                if best_card:
                    await self.client.nft.use_card(best_card.id)
                
                # Wait for next optimization cycle
                await asyncio.sleep(3600)  # 1 hour
                
            except Exception as e:
                print(f"Mining bot error: {e}")
                await asyncio.sleep(300)  # 5 minutes on error
    
    def _select_best_card(self, cards, mining_status):
        # Implement card selection logic
        pass

# Usage
async def main():
    client = FinovaClient.from_env()
    bot = FinovaMiningBot(client)
    await bot.start()

if __name__ == "__main__":
    asyncio.run(main())
```

### Social Media Automation

```python
from finova import FinovaClient
from finova.social import ContentOptimizer

async def automate_social_posting():
    client = FinovaClient.from_env()
    optimizer = ContentOptimizer(client)
    
    # Generate optimized content schedule
    schedule = await optimizer.generate_schedule(
        platforms=["instagram", "tiktok", "youtube"],
        content_types=["image", "video", "story"],
        frequency="daily",
        optimization_goal="max_xp"
    )
    
    for post in schedule.posts:
        # Post content automatically
        result = await client.xp.post_content(
            platform=post.platform,
            content_type=post.type,
            content_data=post.data,
            scheduled_time=post.optimal_time
        )
        
        print(f"Posted to {post.platform}: +{result.xp_gained} XP")
        
        # Track performance
        await optimizer.track_performance(post.id, result)
```

### Advanced Analytics Dashboard

```python
import pandas as pd
import matplotlib.pyplot as plt
from finova import FinovaClient

class FinovaAnalyticsDashboard:
    def __init__(self, client: FinovaClient):
        self.client = client
    
    async def generate_report(self, period="30d"):
        # Collect all data sources
        mining_data = await self.client.mining.get_analytics(period)
        xp_data = await self.client.xp.analyze_patterns(period)
        referral_data = await self.client.referral.analyze_network()
        staking_data = await self.client.staking.get_status()
        
        # Generate comprehensive report
        report = {
            "mining_efficiency": mining_data.efficiency_score,
            "total_earnings": mining_data.total_earned,
            "xp_growth_rate": xp_data.growth_rate,
            "network_quality": referral_data.quality_score,
            "staking_apy": staking_data.current_apy,
            "roi_percentage": self._calculate_total_roi(mining_data, staking_data)
        }
        
        return report
    
    def _calculate_total_roi(self, mining_data, staking_data):
        # Implement ROI calculation
        pass
    
    def visualize_performance(self, data):
        # Create performance charts
        pass
```

## ⚠️ Error Handling

The SDK implements comprehensive error handling with specific exception types:

```python
from finova.exceptions import (
    FinovaAPIError,
    InsufficientBalanceError,
    RateLimitError,
    InvalidParameterError,
    NetworkError
)

async def safe_mining_operation():
    try:
        result = await client.mining.start_mining()
        return result
    except InsufficientBalanceError as e:
        print(f"Insufficient balance: {e.required_balance} $FIN needed")
    except RateLimitError as e:
        print(f"Rate limited. Retry after: {e.retry_after} seconds")
    except NetworkError as e:
        print(f"Network error: {e.message}")
        # Implement retry logic
    except FinovaAPIError as e:
        print(f"API error: {e.error_code} - {e.message}")
```

## ⚙️ Configuration

### Environment Variables

```bash
# Required
FINOVA_RPC_URL=https://api.mainnet-beta.solana.com
FINOVA_PRIVATE_KEY=your_private_key

# Optional
FINOVA_API_KEY=your_api_key
FINOVA_ENVIRONMENT=mainnet  # mainnet, testnet, devnet
FINOVA_TIMEOUT=30
FINOVA_RETRY_ATTEMPTS=3
FINOVA_CACHE_TTL=300

# Social Media API Keys
INSTAGRAM_CLIENT_ID=your_instagram_client_id
TIKTOK_CLIENT_KEY=your_tiktok_client_key
YOUTUBE_API_KEY=your_youtube_api_key
```

### Advanced Configuration

```python
from finova import Config, FinovaClient

config = Config(
    rpc_url="https://api.mainnet-beta.solana.com",
    timeout=30,
    retry_attempts=3,
    cache_settings={
        "enabled": True,
        "ttl": 300,
        "max_size": 1000
    },
    logging_level="INFO",
    rate_limiting={
        "requests_per_minute": 60,
        "burst_allowance": 10
    }
)

client = FinovaClient.from_config(config)
```

## 🧪 Testing

Run the test suite:

```bash
# Run all tests
pytest

# Run specific test categories
pytest tests/test_mining.py
pytest tests/test_xp.py
pytest tests/test_referral.py

# Run with coverage
pytest --cov=finova tests/

# Run integration tests (requires testnet setup)
pytest tests/integration/ --testnet
```

### Mock Testing

```python
from finova.testing import MockFinovaClient

async def test_mining_logic():
    client = MockFinovaClient()
    
    # Configure mock responses
    client.mining.mock_start_mining(
        return_value={"rate": 0.1, "phase": "growth"}
    )
    
    # Test your logic
    result = await client.mining.start_mining()
    assert result.rate == 0.1
```

## 🤝 Contributing

We welcome contributions! Please see our [Contributing Guide](CONTRIBUTING.md) for details.

### Development Setup

```bash
git clone https://github.com/finova-network/finova-contracts.git
cd finova-contracts/client/python
python -m venv venv
source venv/bin/activate  # On Windows: venv\Scripts\activate
pip install -e ".[dev]"
```

### Code Style

We use `black`, `isort`, and `flake8`:

```bash
black finova/
isort finova/
flake8 finova/
```

## 📝 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🆘 Support

- **Documentation**: [https://docs.finova.network](https://docs.finova.network)
- **Discord**: [https://discord.gg/finova](https://discord.gg/finova)
- **Telegram**: [https://t.me/finova_network](https://t.me/finova_network)
- **Email**: support@finova.network

## 🏆 Contributors

Thanks to all contributors who have helped build this SDK!

---

**Start building with Finova Network today and monetize every social interaction! 🚀**
