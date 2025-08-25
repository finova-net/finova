# finova-net/finova/client/python/setup.py

#!/usr/bin/env python3
"""
Finova Network Python SDK Setup
Enterprise-grade Social-Fi mining client for Web3 integration
"""

import os
import sys
import platform
from pathlib import Path
from setuptools import setup, find_packages, Extension
from setuptools.command.build_ext import build_ext
from setuptools.command.install import install
from setuptools.command.develop import develop

# Project metadata
PACKAGE_NAME = "finova-network"
VERSION = "1.0.0"
DESCRIPTION = "Finova Network: Engage & Earn - Social-Fi Super App Python SDK"
LONG_DESCRIPTION = """
Finova Network Python SDK - The Next Generation Social-Fi Super App

Finova Network represents the ultimate convergence of social media, gaming mechanics, 
and cryptocurrency mining into a unified Super App ecosystem. This Python SDK provides 
comprehensive integration for:

🎯 Core Features:
- Integrated Triple Reward System (XP + RP + $FIN Mining)
- Exponential Regression Mining (Pi Network-inspired)
- Hamster Kombat-style Gamification
- Real-world IDR E-wallet Integration
- AI-powered Anti-bot Protection

📱 Social Platform Integration:
- Instagram, TikTok, YouTube, Facebook, Twitter/X
- Automated content quality analysis
- Viral content detection and rewards
- Cross-platform engagement tracking

⛏️ Mining & Rewards:
- Exponential regression fair distribution
- Network effect amplification through referrals
- XP-based level progression with mining multipliers
- Special NFT cards for enhanced rewards

🔐 Enterprise Security:
- Multi-layer bot detection
- Biometric KYC verification
- Hardware security modules (HSM)
- Formal verification of smart contracts

Built on Solana blockchain with 400ms blocks and 50K+ TPS capacity.
"""

AUTHOR = "Finova Network Team"
AUTHOR_EMAIL = "dev@finova.network"
URL = "https://github.com/finova-network/finova-contracts"
LICENSE = "MIT"
KEYWORDS = [
    "finova", "social-fi", "web3", "solana", "mining", "crypto", 
    "social-media", "nft", "defi", "gamification", "rewards",
    "blockchain", "python-sdk", "api-client"
]

CLASSIFIERS = [
    "Development Status :: 4 - Beta",
    "Intended Audience :: Developers",
    "Intended Audience :: Financial and Insurance Industry",
    "License :: OSI Approved :: MIT License",
    "Operating System :: OS Independent",
    "Programming Language :: Python",
    "Programming Language :: Python :: 3",
    "Programming Language :: Python :: 3.8",
    "Programming Language :: Python :: 3.9",
    "Programming Language :: Python :: 3.10",
    "Programming Language :: Python :: 3.11",
    "Programming Language :: Python :: 3.12",
    "Topic :: Software Development :: Libraries :: Python Modules",
    "Topic :: Internet :: WWW/HTTP :: Dynamic Content",
    "Topic :: Office/Business :: Financial",
    "Topic :: Security :: Cryptography",
    "Topic :: Games/Entertainment",
    "Framework :: AsyncIO",
]

# Dependencies with version pinning for security and stability
INSTALL_REQUIRES = [
    # Core Solana & Web3 dependencies
    "solana>=0.30.2,<1.0.0",
    "solders>=0.18.1,<1.0.0",
    "anchorpy>=0.19.1,<1.0.0",
    "construct>=2.10.68,<3.0.0",
    
    # HTTP & API clients
    "httpx>=0.25.0,<1.0.0",
    "aiohttp>=3.8.5,<4.0.0",
    "requests>=2.31.0,<3.0.0",
    "websockets>=11.0.3,<12.0.0",
    
    # Cryptography & Security
    "cryptography>=41.0.4,<42.0.0",
    "pynacl>=1.5.0,<2.0.0",
    "ed25519>=1.5,<2.0",
    "ecdsa>=0.18.0,<1.0.0",
    
    # Data handling & validation
    "pydantic>=2.4.0,<3.0.0",
    "marshmallow>=3.20.0,<4.0.0",
    "jsonschema>=4.19.0,<5.0.0",
    "python-dotenv>=1.0.0,<2.0.0",
    
    # Async & concurrency
    "asyncio>=3.4.3",
    "aiofiles>=23.2.1,<24.0.0",
    "asyncio-throttle>=1.0.2,<2.0.0",
    
    # Utilities
    "click>=8.1.7,<9.0.0",
    "rich>=13.5.2,<14.0.0",
    "tqdm>=4.66.0,<5.0.0",
    "python-dateutil>=2.8.2,<3.0.0",
    "pytz>=2023.3",
    
    # Social media integrations
    "instagrapi>=1.19.0,<2.0.0",
    "TikTokApi>=5.3.0,<6.0.0",
    "google-api-python-client>=2.100.0,<3.0.0",
    "tweepy>=4.14.0,<5.0.0",
    
    # AI & Machine Learning
    "scikit-learn>=1.3.0,<2.0.0",
    "numpy>=1.24.3,<2.0.0",
    "pandas>=2.0.3,<3.0.0",
    "transformers>=4.33.0,<5.0.0",
    
    # Image & Media processing
    "Pillow>=10.0.0,<11.0.0",
    "opencv-python>=4.8.0.74,<5.0.0",
    "moviepy>=1.0.3,<2.0.0",
    
    # Database & caching
    "redis>=4.6.0,<5.0.0",
    "sqlalchemy>=2.0.20,<3.0.0",
    "alembic>=1.12.0,<2.0.0",
]

# Development dependencies
EXTRAS_REQUIRE = {
    "dev": [
        "pytest>=7.4.0,<8.0.0",
        "pytest-asyncio>=0.21.1,<1.0.0",
        "pytest-cov>=4.1.0,<5.0.0",
        "pytest-mock>=3.11.1,<4.0.0",
        "pytest-xdist>=3.3.1,<4.0.0",
        "black>=23.7.0,<24.0.0",
        "isort>=5.12.0,<6.0.0",
        "flake8>=6.0.0,<7.0.0",
        "mypy>=1.5.1,<2.0.0",
        "bandit>=1.7.5,<2.0.0",
        "safety>=2.3.4,<3.0.0",
        "pre-commit>=3.3.3,<4.0.0",
    ],
    "docs": [
        "sphinx>=7.1.2,<8.0.0",
        "sphinx-rtd-theme>=1.3.0,<2.0.0",
        "sphinx-autodoc-typehints>=1.24.0,<2.0.0",
        "myst-parser>=2.0.0,<3.0.0",
    ],
    "testing": [
        "factory-boy>=3.3.0,<4.0.0",
        "faker>=19.3.0,<20.0.0",
        "responses>=0.23.3,<1.0.0",
        "aioresponses>=0.7.4,<1.0.0",
        "pytest-benchmark>=4.0.0,<5.0.0",
    ],
    "monitoring": [
        "prometheus-client>=0.17.1,<1.0.0",
        "sentry-sdk>=1.29.2,<2.0.0",
        "structlog>=23.1.0,<24.0.0",
        "opentelemetry-api>=1.19.0,<2.0.0",
    ],
    "social": [
        "facebook-sdk>=3.1.0,<4.0.0",
        "python-telegram-bot>=20.4,<21.0",
        "discord.py>=2.3.2,<3.0.0",
        "slack-sdk>=3.21.3,<4.0.0",
    ]
}

# Add 'all' option for complete installation
EXTRAS_REQUIRE["all"] = list(set(sum(EXTRAS_REQUIRE.values(), [])))

# Entry points for CLI tools
ENTRY_POINTS = {
    "console_scripts": [
        "finova=finova.cli:main",
        "finova-mine=finova.cli:mine_command",
        "finova-wallet=finova.cli:wallet_command",
        "finova-social=finova.cli:social_command",
        "finova-nft=finova.cli:nft_command",
        "finova-stats=finova.cli:stats_command",
        "finova-guild=finova.cli:guild_command",
    ]
}

class CustomBuildExt(build_ext):
    """Custom build extension for platform-specific optimizations"""
    
    def run(self):
        # Add platform-specific compilation flags
        if platform.system() == "Darwin":  # macOS
            os.environ["CFLAGS"] = "-O3 -march=native"
        elif platform.system() == "Linux":
            os.environ["CFLAGS"] = "-O3 -march=native -fPIC"
        elif platform.system() == "Windows":
            os.environ["CFLAGS"] = "/O2"
            
        super().run()

class CustomInstall(install):
    """Custom installation with post-install setup"""
    
    def run(self):
        super().run()
        self._post_install()
    
    def _post_install(self):
        """Post-installation configuration"""
        try:
            # Create default config directory
            config_dir = Path.home() / ".finova"
            config_dir.mkdir(exist_ok=True)
            
            # Create default configuration file
            config_file = config_dir / "config.yaml"
            if not config_file.exists():
                default_config = """
# Finova Network Configuration
network:
  cluster: "mainnet-beta"  # mainnet-beta, testnet, devnet
  rpc_url: "https://api.mainnet-beta.solana.com"
  commitment: "confirmed"

mining:
  auto_start: false
  check_interval: 300  # seconds
  quality_threshold: 0.7

social:
  platforms:
    instagram: false
    tiktok: false
    youtube: false
    facebook: false
    twitter: false
  
  content:
    auto_analyze: true
    quality_filter: true
    spam_detection: true

security:
  encryption_enabled: true
  biometric_verification: false
  hardware_security: false

logging:
  level: "INFO"
  file: "~/.finova/logs/finova.log"
  max_size: "10MB"
  backup_count: 5
"""
                config_file.write_text(default_config)
            
            print(f"✅ Finova Network SDK installed successfully!")
            print(f"📁 Configuration directory: {config_dir}")
            print(f"🔧 Edit config: {config_file}")
            print(f"🚀 Get started: finova --help")
            
        except Exception as e:
            print(f"⚠️ Post-install setup warning: {e}")

class CustomDevelop(develop):
    """Custom development installation"""
    
    def run(self):
        super().run()
        self._setup_dev_environment()
    
    def _setup_dev_environment(self):
        """Setup development environment"""
        try:
            # Install pre-commit hooks
            os.system("pre-commit install")
            print("✅ Development environment configured!")
            print("🔨 Pre-commit hooks installed")
            print("🧪 Run tests: pytest")
            print("🎨 Format code: black . && isort .")
            
        except Exception as e:
            print(f"⚠️ Dev setup warning: {e}")

def read_file(filename):
    """Read file content safely"""
    try:
        with open(filename, "r", encoding="utf-8") as f:
            return f.read()
    except FileNotFoundError:
        return ""

def get_version():
    """Get version from __init__.py or fallback"""
    init_file = Path("finova") / "__init__.py"
    if init_file.exists():
        content = init_file.read_text()
        for line in content.split("\n"):
            if line.startswith("__version__"):
                return line.split("=")[1].strip().strip('"').strip("'")
    return VERSION

# Platform-specific requirements
if platform.system() == "Windows":
    INSTALL_REQUIRES.append("pywin32>=306")
elif platform.system() == "Darwin":
    INSTALL_REQUIRES.append("pyobjc-core>=9.2")

# Python version check
if sys.version_info < (3, 8):
    sys.exit("❌ Python 3.8 or higher is required")

# Main setup configuration
setup(
    name=PACKAGE_NAME,
    version=get_version(),
    description=DESCRIPTION,
    long_description=LONG_DESCRIPTION,
    long_description_content_type="text/markdown",
    author=AUTHOR,
    author_email=AUTHOR_EMAIL,
    url=URL,
    license=LICENSE,
    
    # Package discovery
    packages=find_packages(exclude=["tests", "tests.*", "docs", "examples"]),
    package_dir={"finova": "finova"},
    
    # Include additional files
    include_package_data=True,
    package_data={
        "finova": [
            "config/*.yaml",
            "config/*.json",
            "templates/*.json",
            "schemas/*.json",
            "*.md",
            "*.txt",
        ]
    },
    
    # Dependencies
    install_requires=INSTALL_REQUIRES,
    extras_require=EXTRAS_REQUIRE,
    
    # Python requirements
    python_requires=">=3.8,<4.0",
    
    # Entry points
    entry_points=ENTRY_POINTS,
    
    # Custom commands
    cmdclass={
        "build_ext": CustomBuildExt,
        "install": CustomInstall,
        "develop": CustomDevelop,
    },
    
    # Metadata
    keywords=KEYWORDS,
    classifiers=CLASSIFIERS,
    
    # Additional metadata
    project_urls={
        "Documentation": "https://docs.finova.network",
        "Source": "https://github.com/finova-network/finova-contracts",
        "Tracker": "https://github.com/finova-network/finova-contracts/issues",
        "Funding": "https://github.com/sponsors/finova-network",
        "Twitter": "https://twitter.com/FinovaNetwork",
        "Discord": "https://discord.gg/finova",
        "Telegram": "https://t.me/finovanetwork",
    },
    
    # Security and distribution
    zip_safe=False,
    platforms=["any"],
    
    # Optional C extensions for performance
    ext_modules=[],
    
    # Test suite
    test_suite="tests",
    tests_require=EXTRAS_REQUIRE["testing"],
    
    # Distribution options
    options={
        "build": {
            "build_base": "build",
        },
        "egg_info": {
            "egg_base": ".",
        },
    },
)

# Post-setup information
if __name__ == "__main__":
    print("""
🎉 Finova Network Python SDK Setup Complete!

📚 Quick Start Guide:
   1. Configure: finova config init
   2. Connect wallet: finova wallet connect
   3. Start mining: finova mine start
   4. Check stats: finova stats overview

🔗 Important Links:
   • Documentation: https://docs.finova.network
   • GitHub: https://github.com/finova-network/finova-contracts
   • Discord: https://discord.gg/finova
   • Whitepaper: https://whitepaper.finova.network

⚡ Ready to Engage & Earn with Finova Network!
""")
