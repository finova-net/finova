# Finova Network API

> **The Next Generation Social-Fi Super App Backend**
> 
> Enterprise-grade Node.js/TypeScript API powering the integrated XP + RP + $FIN mining ecosystem

[![TypeScript](https://img.shields.io/badge/TypeScript-5.0+-blue.svg)](https://www.typescriptlang.org/)
[![Node.js](https://img.shields.io/badge/Node.js-18.0+-green.svg)](https://nodejs.org/)
[![Solana](https://img.shields.io/badge/Solana-Web3.js-purple.svg)](https://solana.com/)
[![License](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)

## 🚀 Quick Start

```bash
# Clone and setup
git clone https://github.com/finova-network/finova-contracts.git
cd finova-contracts/api

# Install dependencies
npm install

# Setup environment
cp .env.example .env
# Edit .env with your configuration

# Run database migrations
npm run migrate

# Start development server
npm run dev

# API available at http://localhost:3000
```

## 📋 Table of Contents

- [Architecture Overview](#architecture-overview)
- [Core Features](#core-features)
- [API Endpoints](#api-endpoints)
- [Authentication](#authentication)
- [Mining Engine](#mining-engine)
- [Real-time Features](#real-time-features)
- [Database Schema](#database-schema)
- [Deployment](#deployment)
- [Security](#security)
- [Testing](#testing)

## 🏗️ Architecture Overview

```
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│   Mobile Apps   │    │    Web Frontend  │    │  Admin Panel    │
└─────┬───────────┘    └────────┬─────────┘    └─────────┬───────┘
      │                         │                        │
      └─────────────────────────┼────────────────────────┘
                                │
                    ┌───────────▼────────────┐
                    │     API Gateway        │
                    │   (Rate Limiting)      │
                    └───────────┬────────────┘
                                │
            ┌───────────────────┼───────────────────┐
            │                   │                   │
    ┌───────▼────────┐ ┌────────▼────────┐ ┌───────▼────────┐
    │ Auth Service   │ │  Core Services  │ │ AI Services    │
    │ - JWT + KYC    │ │ - Mining Engine │ │ - Content AI   │
    │ - Biometric    │ │ - XP Calculator │ │ - Bot Detection│
    │ - 2FA          │ │ - RP Manager    │ │ - Quality Score│
    └────────────────┘ └─────────────────┘ └────────────────┘
            │                   │                   │
            └───────────────────┼───────────────────┘
                                │
                    ┌───────────▼────────────┐
                    │    Database Layer      │
                    │ PostgreSQL + Redis     │
                    └───────────┬────────────┘
                                │
                    ┌───────────▼────────────┐
                    │   Solana Blockchain    │
                    │   Smart Contracts      │
                    └────────────────────────┘
```

## ⚡ Core Features

### Integrated Reward System
- **XP Mining**: Gamified social engagement with exponential progression
- **RP Network**: Referral system with network effects and quality scoring
- **$FIN Mining**: Pi Network-inspired mining with exponential regression
- **Quality AI**: Content analysis for fair reward distribution

### Social Media Integration
- **Multi-Platform**: Instagram, TikTok, YouTube, Facebook, X (Twitter)
- **Real-time Sync**: Automated engagement tracking and verification
- **Content Analysis**: AI-powered quality assessment and originality detection

### Advanced Features
- **Anti-Bot Protection**: Multi-layer detection with human verification
- **KYC Integration**: Biometric verification with Indonesian e-wallet support
- **Guild System**: Hamster Kombat-inspired community mechanics
- **NFT Marketplace**: Special cards with utility integration

## 🔗 API Endpoints

### Authentication
```
POST   /api/auth/register           # User registration
POST   /api/auth/login              # User login
POST   /api/auth/refresh            # Token refresh
POST   /api/auth/kyc/submit         # KYC submission
GET    /api/auth/kyc/status         # KYC verification status
POST   /api/auth/2fa/enable         # Enable 2FA
```

### Mining System
```
GET    /api/mining/status           # Current mining status
POST   /api/mining/start            # Start mining session
POST   /api/mining/stop             # Stop mining session
GET    /api/mining/rate             # Current mining rate
GET    /api/mining/history          # Mining history
POST   /api/mining/boost            # Apply mining boost card
```

### XP System
```
GET    /api/xp/profile              # User XP profile
GET    /api/xp/leaderboard          # XP leaderboard
POST   /api/xp/activity             # Log social activity
GET    /api/xp/activities           # Activity history
GET    /api/xp/multipliers          # Current multipliers
```

### Referral System
```
GET    /api/referral/code           # User referral code
POST   /api/referral/apply          # Apply referral code
GET    /api/referral/network        # Referral network stats
GET    /api/referral/earnings       # Referral earnings
GET    /api/referral/leaderboard    # Referral leaderboard
```

### Social Integration
```
POST   /api/social/connect          # Connect social account
GET    /api/social/accounts         # Connected accounts
POST   /api/social/sync             # Manual sync activities
GET    /api/social/activities       # Social activities log
POST   /api/social/webhook          # Platform webhooks
```

### NFT & Marketplace
```
GET    /api/nft/collection          # User NFT collection
POST   /api/nft/purchase            # Purchase NFT/card
POST   /api/nft/use                 # Use special card
GET    /api/nft/marketplace         # Marketplace listings
POST   /api/nft/list                # List NFT for sale
```

## 🔐 Authentication

### JWT + Biometric Flow
```typescript
// Authentication request example
POST /api/auth/login
{
  "email": "user@example.com",
  "password": "securePassword",
  "biometric_hash": "base64_encoded_biometric",
  "device_id": "unique_device_identifier"
}

// Response
{
  "access_token": "jwt_token",
  "refresh_token": "refresh_token",
  "expires_in": 3600,
  "user": {
    "id": "user_id",
    "level": 25,
    "rp_tier": "Influencer",
    "mining_rate": 0.045
  }
}
```

### Security Headers
```
Authorization: Bearer <jwt_token>
X-Device-ID: <device_identifier>
X-Biometric-Hash: <biometric_verification>
X-Request-Signature: <hmac_signature>
```

## ⛏️ Mining Engine

### Core Algorithm Implementation
```typescript
class MiningEngine {
  calculateMiningRate(user: User): number {
    const baseRate = this.getCurrentPhaseRate();
    const pioneerBonus = this.calculatePioneerBonus();
    const referralBonus = this.calculateReferralBonus(user.referrals);
    const securityBonus = user.isKYCVerified ? 1.2 : 0.8;
    const regressionFactor = Math.exp(-0.001 * user.totalHoldings);
    
    return baseRate * pioneerBonus * referralBonus * securityBonus * regressionFactor;
  }

  calculateIntegratedReward(user: User, activity: Activity): RewardResult {
    const miningReward = this.calculateMiningRate(user);
    const xpMultiplier = this.calculateXPMultiplier(activity, user);
    const rpBonus = this.calculateRPBonus(user);
    const qualityScore = this.analyzeContentQuality(activity);
    
    return {
      fin_earned: miningReward * qualityScore,
      xp_gained: xpMultiplier * qualityScore,
      rp_impact: rpBonus * qualityScore,
      total_value: this.calculateTotalValue(miningReward, xpMultiplier, rpBonus, qualityScore)
    };
  }
}
```

### Mining Phases & Rates
```typescript
const MINING_PHASES = {
  PHASE_1: { userLimit: 100000, baseRate: 0.1, pioneerBonus: 2.0 },
  PHASE_2: { userLimit: 1000000, baseRate: 0.05, pioneerBonus: 1.5 },
  PHASE_3: { userLimit: 10000000, baseRate: 0.025, pioneerBonus: 1.2 },
  PHASE_4: { userLimit: Infinity, baseRate: 0.01, pioneerBonus: 1.0 }
};
```

## 🔄 Real-time Features

### WebSocket Events
```typescript
// Client connection
const socket = io('ws://localhost:3000', {
  auth: { token: 'jwt_token' }
});

// Mining updates
socket.on('mining:rate_update', (data) => {
  console.log('New mining rate:', data.rate);
});

// XP notifications
socket.on('xp:level_up', (data) => {
  console.log('Level up!', data.newLevel);
});

// Network updates
socket.on('referral:new_member', (data) => {
  console.log('New referral:', data.user);
});
```

### Event Types
- `mining:rate_update` - Mining rate changes
- `xp:activity_logged` - New XP activity
- `xp:level_up` - Level progression
- `referral:new_member` - Network growth
- `nft:card_used` - Special card activation
- `guild:challenge_update` - Guild activities

## 🗄️ Database Schema

### Core Tables
```sql
-- Users table with integrated stats
CREATE TABLE users (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  email VARCHAR(255) UNIQUE NOT NULL,
  wallet_address VARCHAR(44) UNIQUE,
  kyc_status VARCHAR(20) DEFAULT 'pending',
  biometric_hash TEXT,
  
  -- Mining stats
  total_fin_mined DECIMAL(18,8) DEFAULT 0,
  current_mining_rate DECIMAL(10,8) DEFAULT 0,
  last_mining_claim TIMESTAMP,
  
  -- XP system
  total_xp BIGINT DEFAULT 0,
  current_level INTEGER DEFAULT 1,
  xp_multiplier DECIMAL(4,2) DEFAULT 1.0,
  
  -- RP system
  total_rp BIGINT DEFAULT 0,
  rp_tier VARCHAR(20) DEFAULT 'Explorer',
  referral_code VARCHAR(10) UNIQUE,
  referred_by UUID REFERENCES users(id),
  
  -- Security
  human_probability DECIMAL(3,2) DEFAULT 0.5,
  suspicious_score INTEGER DEFAULT 0,
  
  created_at TIMESTAMP DEFAULT NOW(),
  updated_at TIMESTAMP DEFAULT NOW()
);

-- Activities table for XP tracking
CREATE TABLE activities (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  user_id UUID REFERENCES users(id),
  platform VARCHAR(20) NOT NULL,
  activity_type VARCHAR(30) NOT NULL,
  content_hash VARCHAR(64),
  
  -- Rewards
  xp_gained INTEGER DEFAULT 0,
  fin_earned DECIMAL(10,8) DEFAULT 0,
  quality_score DECIMAL(3,2) DEFAULT 1.0,
  
  -- Metadata
  platform_post_id VARCHAR(255),
  engagement_metrics JSONB,
  ai_analysis_result JSONB,
  
  created_at TIMESTAMP DEFAULT NOW()
);

-- Referral network tracking
CREATE TABLE referral_network (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  referrer_id UUID REFERENCES users(id),
  referee_id UUID REFERENCES users(id),
  level INTEGER NOT NULL, -- 1=direct, 2=L2, 3=L3
  rp_earned BIGINT DEFAULT 0,
  is_active BOOLEAN DEFAULT true,
  
  created_at TIMESTAMP DEFAULT NOW(),
  UNIQUE(referrer_id, referee_id)
);
```

## 🚀 Deployment

### Docker Setup
```yaml
# docker-compose.prod.yml
version: '3.8'
services:
  api:
    build: .
    ports:
      - "3000:3000"
    environment:
      - NODE_ENV=production
      - DATABASE_URL=${DATABASE_URL}
      - REDIS_URL=${REDIS_URL}
      - SOLANA_RPC_URL=${SOLANA_RPC_URL}
      - JWT_SECRET=${JWT_SECRET}
    depends_on:
      - postgres
      - redis

  postgres:
    image: postgres:15
    environment:
      POSTGRES_DB: finova_network
      POSTGRES_USER: ${DB_USER}
      POSTGRES_PASSWORD: ${DB_PASSWORD}
    volumes:
      - postgres_data:/var/lib/postgresql/data

  redis:
    image: redis:7-alpine
    volumes:
      - redis_data:/data

volumes:
  postgres_data:
  redis_data:
```

### Kubernetes Deployment
```bash
# Deploy to Kubernetes
kubectl apply -f infrastructure/kubernetes/

# Scale based on load
kubectl autoscale deployment finova-api --cpu-percent=70 --min=3 --max=20
```

### Environment Variables
```bash
# Core Configuration
NODE_ENV=production
PORT=3000
API_VERSION=v1

# Database
DATABASE_URL=postgresql://user:pass@localhost:5432/finova_network
REDIS_URL=redis://localhost:6379

# Blockchain
SOLANA_RPC_URL=https://api.mainnet-beta.solana.com
SOLANA_PRIVATE_KEY=base58_encoded_key
PROGRAM_ID=FinovaCoreProgram_address

# Authentication
JWT_SECRET=your_super_secret_key
JWT_EXPIRES_IN=1h
REFRESH_TOKEN_EXPIRES_IN=7d

# External APIs
INSTAGRAM_API_KEY=your_instagram_key
TIKTOK_API_KEY=your_tiktok_key
YOUTUBE_API_KEY=your_youtube_key

# AI Services
OPENAI_API_KEY=your_openai_key
CONTENT_ANALYSIS_ENDPOINT=http://ai-services:5000

# KYC Integration
KYC_PROVIDER_URL=https://kyc-provider.com/api
KYC_API_KEY=your_kyc_key

# Monitoring
SENTRY_DSN=your_sentry_dsn
PROMETHEUS_PORT=9090
```

## 🛡️ Security

### Security Measures
- **Rate Limiting**: 100 requests/minute per user
- **Input Validation**: Joi schema validation on all endpoints
- **SQL Injection Prevention**: Parameterized queries with TypeORM
- **XSS Protection**: Helmet.js security headers
- **CSRF Protection**: Double submit cookie pattern
- **DDoS Protection**: Cloudflare integration
- **Data Encryption**: AES-256 for sensitive data

### API Security Headers
```typescript
app.use(helmet({
  contentSecurityPolicy: {
    directives: {
      defaultSrc: ["'self'"],
      scriptSrc: ["'self'", "'unsafe-inline'"],
      styleSrc: ["'self'", "'unsafe-inline'"],
      imgSrc: ["'self'", "data:", "https:"],
    },
  },
  hsts: {
    maxAge: 31536000,
    includeSubDomains: true,
    preload: true
  }
}));
```

## 🧪 Testing

### Test Coverage
- **Unit Tests**: Jest + Supertest (>95% coverage)
- **Integration Tests**: Database and external API mocking
- **Load Tests**: Artillery.js for performance testing
- **Security Tests**: OWASP ZAP automated scanning

### Run Tests
```bash
# Unit tests
npm run test

# Integration tests
npm run test:integration

# Load testing
npm run test:load

# Security testing
npm run test:security

# Generate coverage report
npm run test:coverage
```

### Example Test
```typescript
describe('Mining Engine', () => {
  it('should calculate correct mining rate with exponential regression', async () => {
    const user = await createTestUser({
      totalHoldings: 10000,
      referrals: 25,
      isKYCVerified: true
    });
    
    const rate = miningEngine.calculateMiningRate(user);
    expect(rate).toBeLessThan(0.01); // Regression should reduce rate
    expect(rate).toBeGreaterThan(0); // But still positive
  });
});
```

## 📊 Monitoring & Analytics

### Health Checks
```
GET /health              # Basic health check
GET /health/detailed     # Detailed system status
GET /metrics             # Prometheus metrics
```

### Key Metrics
- **API Performance**: Response times, error rates
- **Mining Statistics**: Total $FIN mined, active miners
- **User Engagement**: XP activities, referral growth
- **System Health**: Database connections, memory usage

### Logging
```typescript
import logger from './utils/logger';

// Structured logging
logger.info('User registered', {
  userId: user.id,
  referralCode: user.referralCode,
  timestamp: new Date().toISOString()
});
```

## 🤝 Contributing

### Development Setup
```bash
# Install dependencies
npm install

# Setup pre-commit hooks
npm run prepare

# Run in development mode
npm run dev

# Format code
npm run format

# Lint code
npm run lint
```

### Coding Standards
- **TypeScript**: Strict mode enabled
- **ESLint**: Airbnb configuration
- **Prettier**: Automatic code formatting
- **Conventional Commits**: Commit message standards

## 📄 License

MIT License - see [LICENSE](LICENSE) file for details.

## 🔗 Links

- **Main Repository**: [github.com/finova-network/finova-contracts](https://github.com/finova-network/finova-contracts)
- **Documentation**: [docs.finova.network](https://docs.finova.network)
- **API Status**: [status.finova.network](https://status.finova.network)
- **Community**: [discord.gg/finova](https://discord.gg/finova)

## 📞 Support

- **Technical Issues**: [GitHub Issues](https://github.com/finova-network/finova-contracts/issues)
- **Business Inquiries**: business@finova.network
- **Security Reports**: security@finova.network

---

**Built with ❤️ by the Finova Network Team**

> Transforming social engagement into measurable value through XP + RP + $FIN integration