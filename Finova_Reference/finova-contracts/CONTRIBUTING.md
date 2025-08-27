# Contributing to Finova Network

Welcome to the Finova Network project! We're excited that you're interested in contributing to the next generation social-fi super app. This document provides comprehensive guidelines for contributing to our codebase.

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [Development Environment Setup](#development-environment-setup)
- [Project Structure](#project-structure)
- [Development Workflow](#development-workflow)
- [Coding Standards](#coding-standards)
- [Testing Requirements](#testing-requirements)
- [Security Guidelines](#security-guidelines)
- [Smart Contract Development](#smart-contract-development)
- [API Development](#api-development)
- [Mobile Development](#mobile-development)
- [Documentation](#documentation)
- [Submission Process](#submission-process)
- [Review Process](#review-process)
- [Release Process](#release-process)

## Code of Conduct

By participating in this project, you agree to abide by our [Code of Conduct](CODE_OF_CONDUCT.md). Please read it before contributing.

## Getting Started

### Prerequisites

Before contributing, ensure you have the following installed:

```bash
# Core Development Tools
- Rust 1.70+
- Node.js 18+
- Python 3.9+
- Git 2.30+

# Blockchain Development
- Solana CLI 1.16+
- Anchor Framework 0.28+
- SPL Token CLI

# Mobile Development (if contributing to mobile)
- Xcode 14+ (iOS)
- Android Studio 2022.3+ (Android)
- React Native CLI 0.72+

# Database & Infrastructure
- PostgreSQL 14+
- Redis 6+
- Docker 20.10+
- Kubernetes 1.24+ (optional)
```

### Initial Setup

1. **Fork the Repository**
   ```bash
   git clone https://github.com/your-username/finova-contracts.git
   cd finova-contracts
   ```

2. **Install Dependencies**
   ```bash
   # Install all workspace dependencies
   yarn install
   
   # Install Rust dependencies
   cargo build
   
   # Setup development environment
   ./scripts/development/setup-dev-environment.sh
   ```

3. **Environment Configuration**
   ```bash
   # Copy environment template
   cp .env.example .env
   
   # Configure your local settings
   nano .env
   ```

## Development Environment Setup

### Local Blockchain Setup

```bash
# Start local Solana validator
solana-test-validator --reset

# Deploy programs to local
anchor build
anchor deploy --provider.cluster localnet

# Verify deployment
anchor test
```

### Database Setup

```bash
# Start PostgreSQL and Redis
docker-compose up -d postgres redis

# Run database migrations
./scripts/migration/migrate-database.sh

# Seed test data
./scripts/development/seed-test-data.sh
```

### AI Services Setup

```bash
# Start AI services
docker-compose up -d ai-services

# Verify AI services
curl http://localhost:8080/health
```

## Project Structure

Our project follows a modular monorepo structure:

```
finova-contracts/
├── programs/           # Solana smart contracts
├── client/            # SDK implementations
├── api/               # Backend API services
├── mobile-sdk/        # Mobile SDKs
├── ai-services/       # AI/ML services
├── tests/             # Comprehensive test suite
├── docs/              # Documentation
├── scripts/           # Automation scripts
└── infrastructure/    # DevOps configurations
```

## Development Workflow

### Branch Naming Convention

```bash
# Feature branches
feature/mining-algorithm-optimization
feature/nft-marketplace-integration
feature/mobile-sdk-ios

# Bug fixes
bugfix/memory-leak-mining-service
bugfix/xp-calculation-overflow

# Hot fixes
hotfix/critical-security-patch
hotfix/mainnet-deployment-fix

# Documentation
docs/api-endpoint-documentation
docs/smart-contract-architecture
```

### Commit Message Format

We follow the [Conventional Commits](https://www.conventionalcommits.org/) specification:

```bash
<type>[optional scope]: <description>

[optional body]

[optional footer(s)]
```

**Types:**
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `style`: Code style changes (formatting, etc.)
- `refactor`: Code refactoring
- `test`: Adding or updating tests
- `chore`: Maintenance tasks
- `security`: Security improvements
- `perf`: Performance improvements

**Examples:**
```bash
feat(mining): implement exponential regression algorithm

- Add Pi Network-inspired mining rate calculation
- Include anti-whale mechanisms
- Implement proper mathematical safeguards

Closes #123

fix(api): resolve memory leak in websocket connections

- Fix event listener cleanup
- Add proper connection pooling
- Implement automatic reconnection

security(smart-contracts): add reentrancy protection

- Implement checks-effects-interactions pattern
- Add ReentrancyGuard to critical functions
- Update security tests

Breaking Change: API endpoint /v1/mining now requires authentication
```

## Coding Standards

### Rust (Smart Contracts)

```rust
// Use descriptive names and proper error handling
pub fn calculate_mining_rate(
    user: &UserAccount,
    network_state: &NetworkState,
) -> Result<u64> {
    // Validate inputs
    require!(user.is_initialized, ErrorCode::UserNotInitialized);
    require!(network_state.is_active, ErrorCode::NetworkNotActive);
    
    // Calculate with overflow protection
    let base_rate = network_state.base_mining_rate;
    let user_multiplier = calculate_user_multiplier(user)?;
    
    base_rate
        .checked_mul(user_multiplier)
        .ok_or(ErrorCode::MathOverflow)
}

// Always use proper error types
#[error_code]
pub enum ErrorCode {
    #[msg("User account not initialized")]
    UserNotInitialized,
    #[msg("Mathematical overflow occurred")]
    MathOverflow,
    #[msg("Network is not active")]
    NetworkNotActive,
}
```

### TypeScript (API/Client)

```typescript
// Use strict typing and proper error handling
interface MiningRateCalculation {
  baseRate: number;
  userMultiplier: number;
  finalRate: number;
  timestamp: Date;
}

class MiningService {
  async calculateUserMiningRate(
    userId: string,
    options?: MiningOptions
  ): Promise<Result<MiningRateCalculation, MiningError>> {
    try {
      // Validate input
      if (!this.isValidUserId(userId)) {
        return Err(new MiningError('Invalid user ID'));
      }
      
      // Fetch user data with timeout
      const user = await this.userService.getUser(userId, {
        timeout: 5000
      });
      
      if (!user) {
        return Err(new MiningError('User not found'));
      }
      
      // Calculate rate
      const calculation = await this.performCalculation(user, options);
      
      return Ok(calculation);
    } catch (error) {
      this.logger.error('Mining rate calculation failed', {
        userId,
        error: error.message,
        stack: error.stack
      });
      
      return Err(new MiningError('Calculation failed'));
    }
  }
  
  private isValidUserId(userId: string): boolean {
    return typeof userId === 'string' && 
           userId.length > 0 && 
           /^[a-zA-Z0-9_-]+$/.test(userId);
  }
}
```

### Python (AI Services)

```python
from typing import Optional, Dict, Any, List
import logging
from dataclasses import dataclass
from abc import ABC, abstractmethod

@dataclass
class ContentQualityResult:
    score: float
    confidence: float
    factors: Dict[str, float]
    recommendations: List[str]

class ContentAnalyzer(ABC):
    """Abstract base class for content analysis"""
    
    def __init__(self, config: Dict[str, Any]):
        self.config = config
        self.logger = logging.getLogger(self.__class__.__name__)
    
    @abstractmethod
    async def analyze_content(
        self, 
        content: str, 
        metadata: Optional[Dict[str, Any]] = None
    ) -> ContentQualityResult:
        """Analyze content quality and return score"""
        pass
    
    def _validate_content(self, content: str) -> bool:
        """Validate content input"""
        if not isinstance(content, str):
            return False
        if len(content.strip()) == 0:
            return False
        if len(content) > self.config.get('max_content_length', 10000):
            return False
        return True

class AIQualityAnalyzer(ContentAnalyzer):
    """AI-powered content quality analyzer"""
    
    async def analyze_content(
        self, 
        content: str, 
        metadata: Optional[Dict[str, Any]] = None
    ) -> ContentQualityResult:
        if not self._validate_content(content):
            raise ValueError("Invalid content provided")
        
        try:
            # Perform AI analysis
            factors = await self._analyze_factors(content, metadata)
            score = self._calculate_composite_score(factors)
            confidence = self._calculate_confidence(factors)
            recommendations = self._generate_recommendations(factors)
            
            return ContentQualityResult(
                score=score,
                confidence=confidence,
                factors=factors,
                recommendations=recommendations
            )
        except Exception as e:
            self.logger.error(f"Content analysis failed: {e}")
            raise
```

## Testing Requirements

### Test Coverage Standards

- **Smart Contracts**: 95% line coverage minimum
- **API Services**: 90% line coverage minimum
- **Client SDKs**: 85% line coverage minimum
- **Mobile SDKs**: 80% line coverage minimum

### Testing Types Required

1. **Unit Tests**
   ```bash
   # Rust smart contracts
   cargo test --package finova-core
   
   # TypeScript API
   yarn test:unit --coverage
   
   # Python AI services
   pytest tests/unit/ --cov=src --cov-report=html
   ```

2. **Integration Tests**
   ```bash
   # Cross-program interactions
   anchor test --skip-deploy
   
   # API integration
   yarn test:integration
   
   # End-to-end flows
   yarn test:e2e
   ```

3. **Security Tests**
   ```bash
   # Smart contract security
   ./scripts/test/run-security-tests.sh
   
   # API penetration testing
   yarn test:security
   
   # Dependency vulnerability scanning
   yarn audit --audit-level moderate
   ```

### Test Examples

```rust
// Smart contract unit test
#[tokio::test]
async fn test_mining_rate_calculation() {
    let mut context = program_test().start_with_context().await;
    
    // Setup test data
    let user = create_test_user(&mut context).await?;
    let network_state = create_test_network_state(&mut context).await?;
    
    // Execute function
    let result = calculate_mining_rate(&user, &network_state)?;
    
    // Verify results
    assert!(result > 0);
    assert!(result <= network_state.max_mining_rate);
    
    // Test edge cases
    let empty_user = UserAccount::default();
    let error_result = calculate_mining_rate(&empty_user, &network_state);
    assert!(error_result.is_err());
}
```

```typescript
// API integration test
describe('Mining API', () => {
  let app: Application;
  let testUser: TestUser;
  
  beforeAll(async () => {
    app = await createTestApp();
    testUser = await createTestUser();
  });
  
  afterAll(async () => {
    await cleanupTestData();
    await app.close();
  });
  
  it('should calculate mining rate correctly', async () => {
    const response = await request(app)
      .get(`/api/v1/mining/rate/${testUser.id}`)
      .set('Authorization', `Bearer ${testUser.token}`)
      .expect(200);
    
    expect(response.body).toMatchObject({
      baseRate: expect.any(Number),
      userMultiplier: expect.any(Number),
      finalRate: expect.any(Number),
      timestamp: expect.any(String)
    });
    
    expect(response.body.finalRate).toBeGreaterThan(0);
  });
  
  it('should handle rate limiting', async () => {
    // Make multiple rapid requests
    const promises = Array(100).fill().map(() => 
      request(app)
        .get(`/api/v1/mining/rate/${testUser.id}`)
        .set('Authorization', `Bearer ${testUser.token}`)
    );
    
    const responses = await Promise.all(promises);
    const rateLimited = responses.filter(r => r.status === 429);
    
    expect(rateLimited.length).toBeGreaterThan(0);
  });
});
```

## Security Guidelines

### Smart Contract Security

1. **Reentrancy Protection**
   ```rust
   use anchor_lang::prelude::*;
   
   #[derive(Accounts)]
   pub struct SecureWithdraw<'info> {
       #[account(mut, has_one = authority)]
       pub user_account: Account<'info, UserAccount>,
       #[account(mut)]
       pub authority: Signer<'info>,
       pub system_program: Program<'info, System>,
   }
   
   pub fn secure_withdraw(ctx: Context<SecureWithdraw>, amount: u64) -> Result<()> {
       let user_account = &mut ctx.accounts.user_account;
       
       // Checks
       require!(amount > 0, ErrorCode::InvalidAmount);
       require!(user_account.balance >= amount, ErrorCode::InsufficientBalance);
       
       // Effects
       user_account.balance = user_account.balance.checked_sub(amount)
           .ok_or(ErrorCode::MathOverflow)?;
       
       // Interactions (external calls last)
       invoke_signed(
           &system_instruction::transfer(
               &ctx.accounts.user_account.to_account_info().key,
               &ctx.accounts.authority.key,
               amount,
           ),
           &[
               ctx.accounts.user_account.to_account_info(),
               ctx.accounts.authority.to_account_info(),
               ctx.accounts.system_program.to_account_info(),
           ],
           &[]
       )?;
       
       Ok(())
   }
   ```

2. **Input Validation**
   ```rust
   pub fn validate_mining_parameters(
       base_rate: u64,
       user_multiplier: u64,
       network_factor: u64,
   ) -> Result<()> {
       // Range validation
       require!(base_rate <= MAX_BASE_RATE, ErrorCode::BaseRateTooHigh);
       require!(user_multiplier <= MAX_USER_MULTIPLIER, ErrorCode::MultiplierTooHigh);
       require!(network_factor <= MAX_NETWORK_FACTOR, ErrorCode::NetworkFactorTooHigh);
       
       // Overflow protection
       let _result = base_rate
           .checked_mul(user_multiplier)
           .and_then(|x| x.checked_mul(network_factor))
           .ok_or(ErrorCode::CalculationOverflow)?;
       
       Ok(())
   }
   ```

### API Security

```typescript
// Authentication middleware
export const authenticateToken = async (
  req: Request,
  res: Response,
  next: NextFunction
): Promise<void> => {
  try {
    const authHeader = req.headers.authorization;
    const token = authHeader?.split(' ')[1];
    
    if (!token) {
      res.status(401).json({ error: 'Access token required' });
      return;
    }
    
    // Verify JWT token
    const decoded = jwt.verify(token, process.env.JWT_SECRET!) as JWTPayload;
    
    // Check token expiration
    if (decoded.exp < Date.now() / 1000) {
      res.status(401).json({ error: 'Token expired' });
      return;
    }
    
    // Attach user to request
    req.user = await userService.getUserById(decoded.userId);
    
    if (!req.user) {
      res.status(401).json({ error: 'Invalid token' });
      return;
    }
    
    next();
  } catch (error) {
    logger.error('Authentication failed', { error: error.message });
    res.status(401).json({ error: 'Invalid token' });
  }
};

// Rate limiting
export const createRateLimiter = (options: RateLimitOptions) => {
  return rateLimit({
    windowMs: options.windowMs,
    max: options.max,
    message: { error: 'Too many requests, please try again later' },
    standardHeaders: true,
    legacyHeaders: false,
    handler: (req, res) => {
      logger.warn('Rate limit exceeded', {
        ip: req.ip,
        userAgent: req.get('User-Agent'),
        endpoint: req.path
      });
      
      res.status(429).json({
        error: 'Too many requests',
        retryAfter: Math.ceil(options.windowMs / 1000)
      });
    }
  });
};
```

## Smart Contract Development

### Program Architecture

```rust
// lib.rs - Main program entry point
use anchor_lang::prelude::*;

declare_id!("FinovaNetworkProgram1111111111111111111111111");

pub mod instructions;
pub mod state;
pub mod events;
pub mod errors;
pub mod constants;
pub mod utils;

use instructions::*;

#[program]
pub mod finova_core {
    use super::*;
    
    pub fn initialize_network(
        ctx: Context<InitializeNetwork>,
        params: NetworkParams,
    ) -> Result<()> {
        instructions::initialize::initialize_network(ctx, params)
    }
    
    pub fn initialize_user(
        ctx: Context<InitializeUser>,
        referral_code: Option<String>,
    ) -> Result<()> {
        instructions::initialize::initialize_user(ctx, referral_code)
    }
    
    pub fn claim_mining_rewards(
        ctx: Context<ClaimMiningRewards>,
    ) -> Result<()> {
        instructions::mining::claim_mining_rewards(ctx)
    }
    
    pub fn update_xp(
        ctx: Context<UpdateXP>,
        activity_type: ActivityType,
        amount: u64,
    ) -> Result<()> {
        instructions::xp::update_xp(ctx, activity_type, amount)
    }
}
```

### State Account Design

```rust
// state/user.rs
use anchor_lang::prelude::*;

#[account]
pub struct UserState {
    /// User's public key
    pub authority: Pubkey,
    /// Current user level
    pub level: u16,
    /// Total XP earned
    pub total_xp: u64,
    /// Current XP in level
    pub current_level_xp: u64,
    /// Total mining rewards claimed
    pub total_mined: u64,
    /// Last mining claim timestamp
    pub last_claim: i64,
    /// User's referral code
    pub referral_code: String,
    /// Referrer's public key (if any)
    pub referrer: Option<Pubkey>,
    /// KYC verification status
    pub kyc_verified: bool,
    /// Account creation timestamp
    pub created_at: i64,
    /// Last activity timestamp
    pub last_activity: i64,
    /// Reserved space for future upgrades
    pub reserved: [u8; 64],
}

impl UserState {
    pub const LEN: usize = 8 + // discriminator
        32 + // authority
        2 + // level
        8 + // total_xp
        8 + // current_level_xp
        8 + // total_mined
        8 + // last_claim
        (4 + 32) + // referral_code (max 32 chars)
        (1 + 32) + // optional referrer
        1 + // kyc_verified
        8 + // created_at
        8 + // last_activity
        64; // reserved
    
    pub fn is_eligible_for_mining(&self) -> bool {
        self.kyc_verified && 
        Clock::get().unwrap().unix_timestamp - self.last_activity < 86400 // 24 hours
    }
    
    pub fn calculate_level_from_xp(total_xp: u64) -> u16 {
        // Exponential level progression
        let level = (total_xp as f64 / 1000.0).sqrt() as u16;
        std::cmp::min(level, 100) // Max level 100
    }
}
```

## API Development

### Controller Structure

```typescript
// controllers/mining.controller.ts
import { Request, Response } from 'express';
import { MiningService } from '../services/mining.service';
import { validateMiningRequest } from '../middleware/validation.middleware';
import { authenticateToken } from '../middleware/auth.middleware';
import { rateLimit } from '../middleware/rate-limit.middleware';

export class MiningController {
  constructor(private miningService: MiningService) {}
  
  @Get('/rate/:userId')
  @UseMiddleware([authenticateToken, rateLimit({ max: 10, windowMs: 60000 })])
  async getMiningRate(req: Request, res: Response): Promise<void> {
    try {
      const { userId } = req.params;
      
      // Validate user ID format
      if (!this.isValidUserId(userId)) {
        res.status(400).json({ error: 'Invalid user ID format' });
        return;
      }
      
      // Check user permissions
      if (req.user.id !== userId && !req.user.isAdmin) {
        res.status(403).json({ error: 'Access denied' });
        return;
      }
      
      const result = await this.miningService.calculateMiningRate(userId);
      
      if (result.isErr()) {
        res.status(400).json({ error: result.error.message });
        return;
      }
      
      res.json({
        success: true,
        data: result.value,
        timestamp: new Date().toISOString()
      });
      
    } catch (error) {
      this.logger.error('Get mining rate failed', {
        userId: req.params.userId,
        error: error.message,
        stack: error.stack
      });
      
      res.status(500).json({ error: 'Internal server error' });
    }
  }
  
  @Post('/claim')
  @UseMiddleware([authenticateToken, validateMiningRequest])
  async claimRewards(req: Request, res: Response): Promise<void> {
    try {
      const userId = req.user.id;
      
      const result = await this.miningService.claimMiningRewards(userId);
      
      if (result.isErr()) {
        res.status(400).json({ error: result.error.message });
        return;
      }
      
      // Emit real-time update
      this.websocketService.emitToUser(userId, 'mining:claimed', {
        amount: result.value.amount,
        newBalance: result.value.newBalance,
        timestamp: result.value.timestamp
      });
      
      res.json({
        success: true,
        data: result.value
      });
      
    } catch (error) {
      this.logger.error('Claim rewards failed', {
        userId: req.user.id,
        error: error.message
      });
      
      res.status(500).json({ error: 'Internal server error' });
    }
  }
  
  private isValidUserId(userId: string): boolean {
    return /^[a-zA-Z0-9_-]{1,50}$/.test(userId);
  }
}
```

## Mobile Development

### iOS SDK Structure

```swift
// Sources/FinovaSDK/Client/FinovaClient.swift
import Foundation
import Combine

public class FinovaClient {
    private let baseURL: URL
    private let apiKey: String
    private let session: URLSession
    private var cancellables = Set<AnyCancellable>()
    
    public init(baseURL: URL, apiKey: String) {
        self.baseURL = baseURL
        self.apiKey = apiKey
        
        let config = URLSessionConfiguration.default
        config.timeoutIntervalForRequest = 30
        config.timeoutIntervalForResource = 60
        self.session = URLSession(configuration: config)
    }
    
    // MARK: - Mining Services
    
    public func getMiningRate(for userId: String) -> AnyPublisher<MiningRateResponse, FinovaError> {
        let endpoint = "/api/v1/mining/rate/\(userId)"
        
        return makeRequest(endpoint: endpoint, method: .GET)
            .decode(type: APIResponse<MiningRateResponse>.self, decoder: JSONDecoder())
            .map(\.data)
            .mapError { error in
                if let finovaError = error as? FinovaError {
                    return finovaError
                }
                return FinovaError.networkError(error.localizedDescription)
            }
            .eraseToAnyPublisher()
    }
    
    public func claimMiningRewards() -> AnyPublisher<ClaimRewardsResponse, FinovaError> {
        let endpoint = "/api/v1/mining/claim"
        
        return makeRequest(endpoint: endpoint, method: .POST)
            .decode(type: APIResponse<ClaimRewardsResponse>.self, decoder: JSONDecoder())
            .map(\.data)
            .mapError { error in
                if let finovaError = error as? FinovaError {
                    return finovaError
                }
                return FinovaError.networkError(error.localizedDescription)
            }
            .eraseToAnyPublisher()
    }
    
    // MARK: - Private Methods
    
    private func makeRequest(
        endpoint: String,
        method: HTTPMethod,
        body: Data? = nil
    ) -> AnyPublisher<Data, Error> {
        let url = baseURL.appendingPathComponent(endpoint)
        var request = URLRequest(url: url)
        request.httpMethod = method.rawValue
        request.setValue("application/json", forHTTPHeaderField: "Content-Type")
        request.setValue("Bearer \(apiKey)", forHTTPHeaderField: "Authorization")
        
        if let body = body {
            request.httpBody = body
        }
        
        return session.dataTaskPublisher(for: request)
            .map(\.data)
            .eraseToAnyPublisher()
    }
}

// MARK: - Models

public struct MiningRateResponse: Codable {
    public let baseRate: Double
    public let userMultiplier: Double
    public let finalRate: Double
    public let timestamp: Date
}

public struct ClaimRewardsResponse: Codable {
    public let amount: Double
    public let newBalance: Double
    public let timestamp: Date
    public let transactionHash: String
}

public enum FinovaError: Error, LocalizedError {
    case networkError(String)
    case invalidResponse
    case authenticationRequired
    case rateLimitExceeded
    
    public var errorDescription: String? {
        switch self {
        case .networkError(let message):
            return "Network error: \(message)"
        case .invalidResponse:
            return "Invalid response from server"
        case .authenticationRequired:
            return "Authentication required"
        case .rateLimitExceeded:
            return "Rate limit exceeded"
        }
    }
}

private enum HTTPMethod: String {
    case GET = "GET"
    case POST = "POST"
    case PUT = "PUT"
    case DELETE = "DELETE"
}
```

## Documentation

### API Documentation Standards

All API endpoints must be documented using OpenAPI 3.0 specification:

```yaml
# docs/api/mining-endpoints.yaml
openapi: 3.0.0
info:
  title: Finova Network API
  description: The ultimate social-fi super app API
  version: 1.0.0
  contact:
    name: Finova Network Team
    email: developers@finova.network

servers:
  - url: https://api.finova.network/v1
    description: Production server
  - url: https://api-staging.finova.network/v1
    description: Staging server

paths:
  /mining/rate/{userId}:
    get:
      summary: Get user mining rate
      description: Calculate current mining rate for a specific user
      tags:
        - Mining
      parameters:
        - in: path
          name: userId
          required: true
          schema:
            type: string
            pattern: '^[a-zA-Z0-9_-]{1,50}$'
          description: User ID
      responses:
        '200':
          description: Mining rate calculated successfully
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/MiningRateResponse'
        '400':
          description: Invalid request
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/ErrorResponse'
        '401':
          description: Unauthorized
        '403':
          description: Forbidden
        '429':
          description: Rate limit exceeded

components:
  schemas:
    MiningRateResponse:
      type: object
      properties:
        success:
          type: boolean
          example: true
        data:
          type: object
          properties:
            baseRate:
              type: number
              format: double
              description: Base mining rate in FIN/hour
              example: 0.05
            userMultiplier:
              type: number
              format: double
              description: User-specific multiplier
              example: 1.25
            finalRate:
              type: number
              format: double
              description: Final mining rate after all calculations
              example: 0.0625
            timestamp:
              type: string
              format: date-time
              description: Calculation timestamp
              example: "2025-07-29T10:30:00Z"
```

## Submission Process

### Pre-submission Checklist

Before submitting your contribution, ensure:

- [ ] All tests pass locally
- [ ] Code coverage meets requirements
- [ ] No security vulnerabilities detected
- [ ] Documentation is updated
- [ ] Commit messages follow conventions
- [ ] Branch is up to date with main
- [ ] No merge conflicts exist

### Pull Request Process

1. **Create Feature Branch**
   ```bash
   git checkout -b feature/your-feature-name
   git push -u origin feature/your-feature-name
   ```

2. **Make Changes and Test**
   ```bash
   # Make your changes
   # Run tests
   yarn test:all
   
   # Run security scan
   yarn audit
   
   # Check code quality
   yarn lint --fix
   ```

3. **Submit Pull Request**
   - Use our PR template
   - Provide clear description
   - Link related issues
   - Add reviewers
   - Ensure CI passes

### Pull Request Template

```markdown
## Description
Brief description of changes made.

## Type of Change
- [ ] Bug fix (non-breaking change which fixes an issue)
- [ ] New feature (non-breaking change which adds functionality)
- [ ] Breaking change (fix or feature that would cause existing functionality to not work as expected)
- [ ] Documentation update
- [ ] Security improvement
- [ ] Performance improvement
- [ ] Code refactoring

## Related Issues
Closes #issue_number

## Testing
- [ ] Unit tests added/updated
- [ ] Integration tests added/updated
- [ ] End-to-end tests added/updated
- [ ] Security tests added/updated
- [ ] All tests pass locally

## Security Considerations
- [ ] No sensitive data exposed
- [ ] Input validation implemented
- [ ] Proper error handling
- [ ] Authentication/authorization verified
- [ ] SQL injection protection (if applicable)
- [ ] XSS protection (if applicable)

## Performance Impact
- [ ] No performance degradation
- [ ] Performance benchmarks included
- [ ] Memory usage optimized
- [ ] Database queries optimized

## Documentation
- [ ] Code comments updated
- [ ] API documentation updated
- [ ] User documentation updated
- [ ] README updated (if needed)

## Deployment Notes
Special deployment considerations, if any.

## Screenshots/Videos
If applicable, add screenshots or videos to demonstrate changes.

## Checklist
- [ ] My code follows the project's coding standards
- [ ] I have performed a self-review of my code
- [ ] I have commented my code, particularly in hard-to-understand areas
- [ ] I have made corresponding changes to the documentation
- [ ] My changes generate no new warnings
- [ ] I have added tests that prove my fix is effective or that my feature works
- [ ] New and existing unit tests pass locally with my changes
- [ ] Any dependent changes have been merged and published
```

## Review Process

### Review Criteria

All contributions are reviewed based on:

1. **Code Quality**
   - Follows project coding standards
   - Proper error handling
   - Efficient algorithms
   - Clean, readable code

2. **Security**
   - No security vulnerabilities
   - Proper input validation
   - Safe handling of sensitive data
   - Secure authentication/authorization

3. **Testing**
   - Adequate test coverage
   - Tests are meaningful and comprehensive
   - Edge cases covered
   - Performance tests included where relevant

4. **Documentation**
   - Code is well-documented
   - API changes documented
   - User-facing changes documented
   - Technical decisions explained

### Review Timeline

- **Initial Review**: Within 48 hours
- **Follow-up Reviews**: Within 24 hours
- **Final Approval**: Within 72 hours (for standard PRs)
- **Security Reviews**: May take up to 1 week

### Review Roles

- **Code Reviewers**: Focus on code quality, logic, and best practices
- **Security Reviewers**: Focus on security implications and vulnerabilities
- **Architecture Reviewers**: Focus on system design and architectural decisions
- **Product Reviewers**: Focus on user experience and feature completeness

## Release Process

### Version Numbering

We follow [Semantic Versioning](https://semver.org/):

- **MAJOR** version: Incompatible API changes
- **MINOR** version: Backward-compatible functionality additions
- **PATCH** version: Backward-compatible bug fixes

Examples:
- `1.0.0` → `1.0.1` (bug fix)
- `1.0.1` → `1.1.0` (new feature)
- `1.1.0` → `2.0.0` (breaking change)

### Release Branches

```bash
# Create release branch
git checkout -b release/v1.2.0 develop

# Update version numbers
./scripts/release/update-version.sh 1.2.0

# Final testing
./scripts/test/run-all-tests.sh

# Merge to main
git checkout main
git merge --no-ff release/v1.2.0

# Tag release
git tag -a v1.2.0 -m "Release version 1.2.0"

# Merge back to develop
git checkout develop
git merge --no-ff release/v1.2.0
```

### Release Checklist

- [ ] All tests pass on all supported platforms
- [ ] Security audit completed
- [ ] Performance benchmarks meet requirements
- [ ] Documentation updated
- [ ] Migration scripts tested (if applicable)
- [ ] Rollback plan prepared
- [ ] Monitoring alerts configured
- [ ] Release notes prepared

### Hotfix Process

For critical security fixes:

```bash
# Create hotfix branch from main
git checkout -b hotfix/security-fix-v1.2.1 main

# Make critical fix
# Test thoroughly
./scripts/test/run-security-tests.sh

# Merge to main and develop immediately
git checkout main
git merge --no-ff hotfix/security-fix-v1.2.1
git tag -a v1.2.1 -m "Hotfix: Critical security patch"

git checkout develop
git merge --no-ff hotfix/security-fix-v1.2.1
```

## Development Best Practices

### Error Handling

Always implement comprehensive error handling:

```rust
// Rust - Smart Contracts
pub fn safe_calculation(a: u64, b: u64) -> Result<u64> {
    // Input validation
    require!(a > 0, ErrorCode::InvalidInputA);
    require!(b > 0, ErrorCode::InvalidInputB);
    require!(a <= MAX_VALUE, ErrorCode::InputATooLarge);
    require!(b <= MAX_VALUE, ErrorCode::InputBTooLarge);
    
    // Safe arithmetic
    let result = a.checked_mul(b)
        .ok_or(ErrorCode::MultiplicationOverflow)?
        .checked_div(SCALING_FACTOR)
        .ok_or(ErrorCode::DivisionError)?;
    
    // Output validation
    require!(result <= MAX_RESULT, ErrorCode::ResultTooLarge);
    
    Ok(result)
}
```

```typescript
// TypeScript - API Services
export class SafeService {
  async performOperation(input: OperationInput): Promise<Result<OperationResult, ServiceError>> {
    try {
      // Input validation
      const validation = this.validateInput(input);
      if (validation.isErr()) {
        return Err(new ValidationError(validation.error));
      }
      
      // Business logic with error handling
      const result = await this.executeOperation(input);
      
      // Output validation
      if (!this.validateOutput(result)) {
        return Err(new OutputValidationError('Invalid result'));
      }
      
      return Ok(result);
      
    } catch (error) {
      this.logger.error('Operation failed', {
        input: this.sanitizeInput(input),
        error: error.message,
        stack: error.stack
      });
      
      if (error instanceof DatabaseError) {
        return Err(new ServiceError('Database operation failed'));
      }
      
      if (error instanceof NetworkError) {
        return Err(new ServiceError('Network operation failed'));
      }
      
      return Err(new ServiceError('Unknown error occurred'));
    }
  }
}
```

### Logging Standards

Implement structured logging throughout the application:

```typescript
// Centralized logger configuration
import winston from 'winston';

export const logger = winston.createLogger({
  level: process.env.LOG_LEVEL || 'info',
  format: winston.format.combine(
    winston.format.timestamp(),
    winston.format.errors({ stack: true }),
    winston.format.json()
  ),
  defaultMeta: { 
    service: 'finova-api',
    version: process.env.APP_VERSION
  },
  transports: [
    new winston.transports.File({ 
      filename: 'logs/error.log', 
      level: 'error' 
    }),
    new winston.transports.File({ 
      filename: 'logs/combined.log' 
    }),
    new winston.transports.Console({
      format: winston.format.simple()
    })
  ]
});

// Usage in services
export class MiningService {
  async claimRewards(userId: string): Promise<Result<ClaimResult, ServiceError>> {
    const startTime = Date.now();
    
    logger.info('Mining rewards claim started', {
      userId,
      timestamp: new Date().toISOString()
    });
    
    try {
      const result = await this.processRewardClaim(userId);
      
      const duration = Date.now() - startTime;
      logger.info('Mining rewards claim completed', {
        userId,
        amount: result.amount,
        duration,
        success: true
      });
      
      return Ok(result);
      
    } catch (error) {
      const duration = Date.now() - startTime;
      logger.error('Mining rewards claim failed', {
        userId,
        duration,
        error: error.message,
        stack: error.stack,
        success: false
      });
      
      return Err(new ServiceError('Claim failed'));
    }
  }
}
```

### Performance Guidelines

#### Database Optimization

```sql
-- Always use proper indexing
CREATE INDEX CONCURRENTLY idx_users_last_activity 
ON users (last_activity) 
WHERE kyc_verified = true;

CREATE INDEX CONCURRENTLY idx_mining_claims_user_date 
ON mining_claims (user_id, created_at DESC);

-- Use partial indexes for common queries
CREATE INDEX CONCURRENTLY idx_active_users 
ON users (id) 
WHERE last_activity > NOW() - INTERVAL '7 days';
```

```typescript
// Use connection pooling and query optimization
export class DatabaseService {
  private pool: Pool;
  
  constructor() {
    this.pool = new Pool({
      host: process.env.DB_HOST,
      port: parseInt(process.env.DB_PORT || '5432'),
      database: process.env.DB_NAME,
      user: process.env.DB_USER,
      password: process.env.DB_PASSWORD,
      max: 20, // Maximum pool size
      idleTimeoutMillis: 30000,
      connectionTimeoutMillis: 2000,
    });
  }
  
  async getUserMiningData(userId: string): Promise<UserMiningData> {
    // Use prepared statements to prevent SQL injection and improve performance
    const query = `
      SELECT 
        u.id,
        u.level,
        u.total_xp,
        u.last_claim,
        u.kyc_verified,
        COALESCE(mc.total_claimed, 0) as total_claimed
      FROM users u
      LEFT JOIN (
        SELECT 
          user_id, 
          SUM(amount) as total_claimed
        FROM mining_claims 
        WHERE user_id = $1
      ) mc ON u.id = mc.user_id
      WHERE u.id = $1
    `;
    
    const result = await this.pool.query(query, [userId]);
    
    if (result.rows.length === 0) {
      throw new Error('User not found');
    }
    
    return result.rows[0] as UserMiningData;
  }
}
```

#### Caching Strategy

```typescript
// Redis caching for frequently accessed data
export class CacheService {
  private redis: Redis;
  
  constructor() {
    this.redis = new Redis({
      host: process.env.REDIS_HOST,
      port: parseInt(process.env.REDIS_PORT || '6379'),
      retryDelayOnFailover: 100,
      maxRetriesPerRequest: 3,
    });
  }
  
  async getMiningRate(userId: string): Promise<MiningRateData | null> {
    try {
      const cached = await this.redis.get(`mining:rate:${userId}`);
      
      if (cached) {
        const data = JSON.parse(cached) as MiningRateData;
        
        // Check if cache is still valid (5 minutes)
        if (Date.now() - data.calculatedAt < 300000) {
          return data;
        }
      }
      
      return null;
    } catch (error) {
      logger.warn('Cache read failed', { userId, error: error.message });
      return null;
    }
  }
  
  async setMiningRate(userId: string, data: MiningRateData): Promise<void> {
    try {
      await this.redis.setex(
        `mining:rate:${userId}`,
        300, // 5 minutes TTL
        JSON.stringify({
          ...data,
          calculatedAt: Date.now()
        })
      );
    } catch (error) {
      logger.warn('Cache write failed', { userId, error: error.message });
      // Don't throw - caching failure shouldn't break the request
    }
  }
}
```

## Security Requirements

### Input Validation

All inputs must be validated at multiple layers:

```typescript
// Schema validation using Joi
import Joi from 'joi';

export const miningClaimSchema = Joi.object({
  userId: Joi.string()
    .pattern(/^[a-zA-Z0-9_-]{1,50}$/)
    .required()
    .messages({
      'string.pattern.base': 'User ID contains invalid characters',
      'any.required': 'User ID is required'
    }),
  
  timestamp: Joi.date()
    .iso()
    .max('now')
    .min(new Date(Date.now() - 3600000)) // Max 1 hour old
    .required(),
  
  signature: Joi.string()
    .base64()
    .length(88) // Base64 encoded signature length
    .required()
});

// Validation middleware
export const validateMiningClaim = (
  req: Request, 
  res: Response, 
  next: NextFunction
): void => {
  const { error, value } = miningClaimSchema.validate(req.body);
  
  if (error) {
    logger.warn('Validation failed', {
      error: error.details[0].message,
      path: error.details[0].path,
      ip: req.ip
    });
    
    res.status(400).json({
      error: 'Validation failed',
      details: error.details[0].message
    });
    return;
  }
  
  req.body = value; // Use sanitized values
  next();
};
```

### Authentication & Authorization

```typescript
// JWT token management
export class AuthService {
  private readonly jwtSecret: string;
  private readonly refreshSecret: string;
  
  constructor() {
    this.jwtSecret = process.env.JWT_SECRET!;
    this.refreshSecret = process.env.JWT_REFRESH_SECRET!;
    
    if (!this.jwtSecret || !this.refreshSecret) {
      throw new Error('JWT secrets not configured');
    }
  }
  
  generateTokens(user: User): TokenPair {
    const payload = {
      userId: user.id,
      email: user.email,
      role: user.role,
      permissions: user.permissions
    };
    
    const accessToken = jwt.sign(payload, this.jwtSecret, {
      expiresIn: '15m',
      issuer: 'finova-network',
      audience: 'finova-api'
    });
    
    const refreshToken = jwt.sign(
      { userId: user.id },
      this.refreshSecret,
      {
        expiresIn: '7d',
        issuer: 'finova-network',
        audience: 'finova-api'
      }
    );
    
    return { accessToken, refreshToken };
  }
  
  async verifyAccessToken(token: string): Promise<JWTPayload> {
    try {
      const payload = jwt.verify(token, this.jwtSecret, {
        issuer: 'finova-network',
        audience: 'finova-api'
      }) as JWTPayload;
      
      // Additional verification - check if user still exists and is active
      const user = await this.userService.getUserById(payload.userId);
      
      if (!user || !user.isActive) {
        throw new Error('User not found or inactive');
      }
      
      return payload;
    } catch (error) {
      throw new AuthenticationError('Invalid token');
    }
  }
}
```

## Contributing to AI Services

### Content Quality Analysis

```python
# ai-services/content-analyzer/src/models/quality_classifier.py
import numpy as np
import torch
import torch.nn as nn
from transformers import AutoTokenizer, AutoModel
from typing import Dict, List, Tuple, Optional
import asyncio
from dataclasses import dataclass

@dataclass
class QualityMetrics:
    originality_score: float
    engagement_potential: float
    toxicity_score: float
    readability_score: float
    relevance_score: float
    overall_quality: float

class ContentQualityClassifier(nn.Module):
    """Advanced content quality classifier using transformer models"""
    
    def __init__(self, model_name: str = "distilbert-base-uncased"):
        super().__init__()
        
        self.tokenizer = AutoTokenizer.from_pretrained(model_name)
        self.transformer = AutoModel.from_pretrained(model_name)
        
        # Quality assessment heads
        self.originality_head = nn.Linear(768, 1)
        self.engagement_head = nn.Linear(768, 1)
        self.toxicity_head = nn.Linear(768, 1)
        self.readability_head = nn.Linear(768, 1)
        self.relevance_head = nn.Linear(768, 1)
        
        # Overall quality aggregator
        self.quality_aggregator = nn.Sequential(
            nn.Linear(5, 32),
            nn.ReLU(),
            nn.Dropout(0.1),
            nn.Linear(32, 16),
            nn.ReLU(),
            nn.Linear(16, 1),
            nn.Sigmoid()
        )
        
        self.dropout = nn.Dropout(0.1)
        
    def forward(self, input_ids: torch.Tensor, attention_mask: torch.Tensor) -> Dict[str, torch.Tensor]:
        # Get transformer outputs
        outputs = self.transformer(input_ids=input_ids, attention_mask=attention_mask)
        pooled_output = outputs.pooler_output
        pooled_output = self.dropout(pooled_output)
        
        # Calculate individual metrics
        originality = torch.sigmoid(self.originality_head(pooled_output))
        engagement = torch.sigmoid(self.engagement_head(pooled_output))
        toxicity = torch.sigmoid(self.toxicity_head(pooled_output))
        readability = torch.sigmoid(self.readability_head(pooled_output))
        relevance = torch.sigmoid(self.relevance_head(pooled_output))
        
        # Aggregate for overall quality
        metric_vector = torch.cat([originality, engagement, 1-toxicity, readability, relevance], dim=1)
        overall_quality = self.quality_aggregator(metric_vector)
        
        return {
            'originality': originality,
            'engagement': engagement,
            'toxicity': toxicity,
            'readability': readability,
            'relevance': relevance,
            'overall_quality': overall_quality
        }
    
    async def analyze_content(self, content: str, metadata: Optional[Dict] = None) -> QualityMetrics:
        """Analyze content quality asynchronously"""
        
        # Preprocess content
        content = self._preprocess_content(content)
        
        # Tokenize
        inputs = self.tokenizer(
            content,
            max_length=512,
            padding=True,
            truncation=True,
            return_tensors='pt'
        )
        
        # Run inference
        with torch.no_grad():
            outputs = self.forward(inputs['input_ids'], inputs['attention_mask'])
        
        # Extract scores
        return QualityMetrics(
            originality_score=float(outputs['originality'].item()),
            engagement_potential=float(outputs['engagement'].item()),
            toxicity_score=float(outputs['toxicity'].item()),
            readability_score=float(outputs['readability'].item()),
            relevance_score=float(outputs['relevance'].item()),
            overall_quality=float(outputs['overall_quality'].item())
        )
    
    def _preprocess_content(self, content: str) -> str:
        """Preprocess content for analysis"""
        # Remove excessive whitespace
        content = ' '.join(content.split())
        
        # Remove URLs
        import re
        content = re.sub(r'http[s]?://(?:[a-zA-Z]|[0-9]|[$-_@.&+]|[!*\\(\\),]|(?:%[0-9a-fA-F][0-9a-fA-F]))+', '', content)
        
        # Remove mentions and hashtags for analysis (but keep them for context)
        content = re.sub(r'@\w+', '[MENTION]', content)
        content = re.sub(r'#\w+', '[HASHTAG]', content)
        
        return content.strip()
```

### Bot Detection System

```python
# ai-services/bot-detection/src/models/behavior_analyzer.py
import numpy as np
from typing import Dict, List, Tuple, Optional
from dataclasses import dataclass
from sklearn.ensemble import IsolationForest
from sklearn.preprocessing import StandardScaler
import joblib

@dataclass
class BehaviorFeatures:
    temporal_patterns: Dict[str, float]
    interaction_patterns: Dict[str, float]
    content_patterns: Dict[str, float]
    network_patterns: Dict[str, float]
    device_patterns: Dict[str, float]

@dataclass
class BotDetectionResult:
    human_probability: float
    bot_probability: float
    risk_score: float
    anomaly_flags: List[str]
    confidence: float

class BehaviorAnalyzer:
    """Advanced bot detection using behavioral analysis"""
    
    def __init__(self, model_path: Optional[str] = None):
        self.scaler = StandardScaler()
        self.anomaly_detector = IsolationForest(
            contamination=0.1,
            random_state=42,
            n_estimators=100
        )
        
        if model_path:
            self.load_model(model_path)
        
        # Feature extractors
        self.temporal_extractor = TemporalFeatureExtractor()
        self.interaction_extractor = InteractionFeatureExtractor()
        self.content_extractor = ContentFeatureExtractor()
        self.network_extractor = NetworkFeatureExtractor()
        self.device_extractor = DeviceFeatureExtractor()
    
    async def analyze_user_behavior(
        self, 
        user_id: str, 
        activity_history: List[Dict],
        time_window_hours: int = 24
    ) -> BotDetectionResult:
        """Analyze user behavior for bot detection"""
        
        try:
            # Extract features from different dimensions
            features = await self._extract_all_features(user_id, activity_history, time_window_hours)
            
            # Convert to feature vector
            feature_vector = self._features_to_vector(features)
            
            # Scale features
            scaled_features = self.scaler.transform([feature_vector])
            
            # Detect anomalies
            anomaly_score = self.anomaly_detector.decision_function(scaled_features)[0]
            is_anomaly = self.anomaly_detector.predict(scaled_features)[0] == -1
            
            # Calculate probabilities
            human_probability = self._calculate_human_probability(features, anomaly_score)
            bot_probability = 1.0 - human_probability
            
            # Generate risk score
            risk_score = self._calculate_risk_score(features, anomaly_score, is_anomaly)
            
            # Identify specific anomaly flags
            anomaly_flags = self._identify_anomaly_flags(features, anomaly_score)
            
            # Calculate confidence
            confidence = self._calculate_confidence(features, anomaly_score)
            
            return BotDetectionResult(
                human_probability=human_probability,
                bot_probability=bot_probability,
                risk_score=risk_score,
                anomaly_flags=anomaly_flags,
                confidence=confidence
            )
            
        except Exception as e:
            # Log error and return safe default
            logger.error(f"Bot detection analysis failed for user {user_id}: {e}")
            
            return BotDetectionResult(
                human_probability=0.5,  # Neutral when analysis fails
                bot_probability=0.5,
                risk_score=0.5,
                anomaly_flags=['analysis_failed'],
                confidence=0.0
            )
    
    async def _extract_all_features(
        self, 
        user_id: str, 
        activity_history: List[Dict],
        time_window_hours: int
    ) -> BehaviorFeatures:
        """Extract all behavioral features"""
        
        # Extract features from different analyzers
        temporal_features = await self.temporal_extractor.extract(activity_history, time_window_hours)
        interaction_features = await self.interaction_extractor.extract(activity_history)
        content_features = await self.content_extractor.extract(activity_history)
        network_features = await self.network_extractor.extract(user_id, activity_history)
        device_features = await self.device_extractor.extract(activity_history)
        
        return BehaviorFeatures(
            temporal_patterns=temporal_features,
            interaction_patterns=interaction_features,
            content_patterns=content_features,
            network_patterns=network_features,
            device_patterns=device_features
        )
    
    def _calculate_human_probability(self, features: BehaviorFeatures, anomaly_score: float) -> float:
        """Calculate probability that user is human"""
        
        # Base probability from anomaly score
        # Anomaly scores range from -1 (anomalous) to 1 (normal)
        base_prob = (anomaly_score + 1) / 2
        
        # Adjust based on specific behavioral indicators
        human_indicators = []
        
        # Temporal patterns (humans show circadian rhythms)
        if features.temporal_patterns.get('circadian_rhythm_strength', 0) > 0.7:
            human_indicators.append(0.8)
        
        # Natural variation in activity
        if 0.3 < features.temporal_patterns.get('activity_variance', 0) < 0.8:
            human_indicators.append(0.7)
        
        # Content quality and variety
        if features.content_patterns.get('content_diversity', 0) > 0.6:
            human_indicators.append(0.6)
        
        # Natural interaction patterns
        if features.interaction_patterns.get('response_time_variance', 0) > 0.4:
            human_indicators.append(0.7)
        
        # Device consistency (humans typically use 1-3 devices)
        device_count = features.device_patterns.get('unique_devices', 0)
        if 1 <= device_count <= 3:
            human_indicators.append(0.6)
        
        # Combine indicators
        if human_indicators:
            indicator_weight = np.mean(human_indicators)
            final_prob = 0.7 * base_prob + 0.3 * indicator_weight
        else:
            final_prob = base_prob
        
        return np.clip(final_prob, 0.0, 1.0)
    
    def _calculate_risk_score(
        self, 
        features: BehaviorFeatures, 
        anomaly_score: float, 
        is_anomaly: bool
    ) -> float:
        """Calculate overall risk score"""
        
        risk_factors = []
        
        # Anomaly detection result
        if is_anomaly:
            risk_factors.append(0.8)
        else:
            risk_factors.append(0.2)
        
        # Extremely regular patterns (bot-like)
        if features.temporal_patterns.get('regularity_score', 0) > 0.9:
            risk_factors.append(0.9)
        
        # Suspicious interaction speeds
        avg_response_time = features.interaction_patterns.get('avg_response_time', 1000)
        if avg_response_time < 100:  # Less than 100ms average
            risk_factors.append(0.8)
        
        # Low content quality
        if features.content_patterns.get('avg_quality_score', 0.5) < 0.3:
            risk_factors.append(0.7)
        
        # Suspicious network patterns
        if features.network_patterns.get('suspicious_connections', 0) > 0.5:
            risk_factors.append(0.9)
        
        # Too many devices (potential device farm)
        if features.device_patterns.get('unique_devices', 0) > 10:
            risk_factors.append(0.8)
        
        return np.mean(risk_factors)
    
    def _identify_anomaly_flags(self, features: BehaviorFeatures, anomaly_score: float) -> List[str]:
        """Identify specific anomaly flags"""
        
        flags = []
        
        # Temporal anomalies
        if features.temporal_patterns.get('regularity_score', 0) > 0.95:
            flags.append('extremely_regular_timing')
        
        if features.temporal_patterns.get('night_activity_ratio', 0) > 0.8:
            flags.append('excessive_night_activity')
        
        # Interaction anomalies
        if features.interaction_patterns.get('avg_response_time', 1000) < 50:
            flags.append('superhuman_response_speed')
        
        if features.interaction_patterns.get('click_precision', 0) > 0.98:
            flags.append('perfect_click_precision')
        
        # Content anomalies
        if features.content_patterns.get('repetition_score', 0) > 0.8:
            flags.append('highly_repetitive_content')
        
        if features.content_patterns.get('template_similarity', 0) > 0.9:
            flags.append('template_based_content')
        
        # Network anomalies
        if features.network_patterns.get('proxy_probability', 0) > 0.8:
            flags.append('likely_proxy_usage')
        
        if features.device_patterns.get('unique_devices', 0) > 20:
            flags.append('device_farm_pattern')
        
        return flags
    
    def save_model(self, path: str):
        """Save trained model"""
        model_data = {
            'scaler': self.scaler,
            'anomaly_detector': self.anomaly_detector
        }
        joblib.dump(model_data, path)
    
    def load_model(self, path: str):
        """Load trained model"""
        model_data = joblib.load(path)
        self.scaler = model_data['scaler']
        self.anomaly_detector = model_data['anomaly_detector']

# Feature extractors
class TemporalFeatureExtractor:
    """Extract temporal behavioral features"""
    
    async def extract(self, activity_history: List[Dict], time_window_hours: int) -> Dict[str, float]:
        timestamps = [activity['timestamp'] for activity in activity_history]
        
        if len(timestamps) < 2:
            return self._default_features()
        
        # Convert to numpy array for analysis
        timestamps = np.array([pd.Timestamp(ts).timestamp() for ts in timestamps])
        
        features = {}
        
        # Calculate time intervals between activities
        intervals = np.diff(timestamps)
        
        # Regularity score (lower variance = more regular)
        if len(intervals) > 1:
            features['regularity_score'] = 1.0 / (1.0 + np.std(intervals))
        else:
            features['regularity_score'] = 0.5
        
        # Activity variance
        features['activity_variance'] = np.std(intervals) if len(intervals) > 1 else 0.0
        
        # Circadian rhythm analysis
        hours = [pd.Timestamp(ts).hour for ts in timestamps]
        hour_distribution = np.bincount(hours, minlength=24) / len(hours)
        
        # Calculate circadian rhythm strength
        expected_pattern = self._generate_circadian_pattern()
        features['circadian_rhythm_strength'] = 1.0 - np.sum(np.abs(hour_distribution - expected_pattern)) / 2.0
        
        # Night activity ratio (11 PM - 6 AM)
        night_hours = [h for h in hours if h >= 23 or h <= 6]
        features['night_activity_ratio'] = len(night_hours) / len(hours)
        
        # Weekend vs weekday pattern
        weekdays = [pd.Timestamp(ts).weekday() for ts in timestamps]
        weekend_count = sum(1 for d in weekdays if d >= 5)
        features['weekend_activity_ratio'] = weekend_count / len(weekdays)
        
        return features
    
    def _generate_circadian_pattern(self) -> np.ndarray:
        """Generate expected human circadian activity pattern"""
        # Peak activity during day hours, low activity at night
        pattern = np.array([
            0.01, 0.01, 0.01, 0.01, 0.01, 0.02,  # 0-5 AM (very low)
            0.03, 0.05, 0.07, 0.08, 0.09, 0.09,  # 6-11 AM (increasing)
            0.08, 0.07, 0.08, 0.09, 0.08, 0.07,  # 12-5 PM (peak)
            0.06, 0.05, 0.04, 0.03, 0.02, 0.01   # 6-11 PM (decreasing)
        ])
        return pattern / np.sum(pattern)
    
    def _default_features(self) -> Dict[str, float]:
        return {
            'regularity_score': 0.5,
            'activity_variance': 0.0,
            'circadian_rhythm_strength': 0.5,
            'night_activity_ratio': 0.0,
            'weekend_activity_ratio': 0.0
        }

class InteractionFeatureExtractor:
    """Extract interaction behavioral features"""
    
    async def extract(self, activity_history: List[Dict]) -> Dict[str, float]:
        if not activity_history:
            return self._default_features()
        
        features = {}
        
        # Response time analysis
        response_times = []
        click_positions = []
        
        for activity in activity_history:
            if 'response_time' in activity:
                response_times.append(activity['response_time'])
            if 'click_position' in activity:
                click_positions.append(activity['click_position'])
        
        # Average response time
        features['avg_response_time'] = np.mean(response_times) if response_times else 1000.0
        
        # Response time variance (humans show natural variation)
        features['response_time_variance'] = np.std(response_times) if len(response_times) > 1 else 0.0
        
        # Click precision analysis
        if click_positions:
            precisions = [self._calculate_click_precision(pos) for pos in click_positions]
            features['click_precision'] = np.mean(precisions)
        else:
            features['click_precision'] = 0.5
        
        # Interaction consistency
        interaction_types = [activity.get('type', 'unknown') for activity in activity_history]
        type_counts = {}
        for t in interaction_types:
            type_counts[t] = type_counts.get(t, 0) + 1
        
        # Calculate interaction diversity
        if len(type_counts) > 1:
            entropy = -sum((count/len(interaction_types)) * np.log2(count/len(interaction_types)) 
                          for count in type_counts.values())
            features['interaction_diversity'] = entropy / np.log2(len(type_counts))
        else:
            features['interaction_diversity'] = 0.0
        
        return features
    
    def _calculate_click_precision(self, position: Dict) -> float:
        """Calculate how precisely a click hit its target"""
        target_center = position.get('target_center', (0, 0))
        actual_click = position.get('actual_click', (0, 0))
        target_size = position.get('target_size', (50, 50))
        
        # Calculate distance from center
        distance = np.sqrt((target_center[0] - actual_click[0])**2 + 
                          (target_center[1] - actual_click[1])**2)
        
        # Normalize by target size
        normalized_distance = distance / (max(target_size) / 2)
        
        # Convert to precision score (0 = perfect, 1 = completely missed)
        precision = max(0.0, 1.0 - normalized_distance)
        return precision
    
    def _default_features(self) -> Dict[str, float]:
        return {
            'avg_response_time': 1000.0,
            'response_time_variance': 0.0,
            'click_precision': 0.5,
            'interaction_diversity': 0.0
        }
```

## Support and Community

### Getting Help

If you need help while contributing:

1. **Documentation**: Check our comprehensive docs at `/docs/`
2. **GitHub Issues**: Search existing issues or create new ones
3. **Discussions**: Use GitHub Discussions for questions
4. **Discord**: Join our development Discord server
5. **Email**: Contact developers@finova.network for sensitive issues

### Community Guidelines

- Be respectful and inclusive
- Help newcomers get started
- Share knowledge and best practices
- Provide constructive feedback
- Report security issues responsibly

### Recognition

Contributors are recognized through:

- **Hall of Fame**: Top contributors featured in README
- **Contributor Badges**: GitHub profile badges
- **Special Access**: Early access to new features
- **Swag**: Finova Network merchandise for significant contributions
- **Token Rewards**: $FIN tokens for major contributions (when mainnet launches)

## Tools and Resources

### Recommended Development Tools

#### Code Editors
- **VS Code** with extensions:
  - Rust Analyzer
  - Solana Extension Pack
  - TypeScript Hero
  - Python Extension Pack
  - GitLens

#### Testing Tools
- **Postman**: API testing
- **Insomnia**: Alternative API client
- **k6**: Load testing
- **OWASP ZAP**: Security testing

#### Debugging Tools
- **Solana Explorer**: Transaction debugging
- **Chrome DevTools**: Frontend debugging
- **PostgreSQL Admin**: Database inspection
- **Redis Commander**: Cache inspection

### Useful Commands

```bash
# Development workflow
make setup          # Setup development environment
make build          # Build all components
make test           # Run all tests
make lint           # Run linters
make format         # Format code
make clean          # Clean build artifacts

# Blockchain specific
make anchor-build   # Build Solana programs
make anchor-test    # Test Solana programs
make anchor-deploy  # Deploy to configured network

# Services
make api-start      # Start API server
make ai-start       # Start AI services
make db-migrate     # Run database migrations
make db-seed        # Seed test data

# Quality assurance
make security-scan  # Run security scans
make performance-test # Run performance tests
make coverage       # Generate coverage reports
make docs           # Generate documentation
```

### Environment Setup Script

```bash
#!/bin/bash
# scripts/development/setup-dev-environment.sh

set -e

echo "🚀 Setting up Finova Network development environment..."

# Check prerequisites
check_prereqs() {
    echo "📋 Checking prerequisites..."
    
    # Check Rust
    if ! command -v rustc &> /dev/null; then
        echo "❌ Rust not found. Please install from https://rustup.rs/"
        exit 1
    fi
    
    # Check Node.js
    if ! command -v node &> /dev/null; then
        echo "❌ Node.js not found. Please install Node.js 18+"
        exit 1
    fi
    
    # Check Solana CLI
    if ! command -v solana &> /dev/null; then
        echo "❌ Solana CLI not found. Installing..."
        sh -c "$(curl -sSfL https://release.solana.com/stable/install)"
        export PATH="$HOME/.local/share/solana/install/active_release/bin:$PATH"
    fi
    
    # Check Anchor
    if ! command -v anchor &> /dev/null; then
        echo "❌ Anchor not found. Installing..."
        npm install -g @coral-xyz/anchor-cli
    fi
    
    echo "✅ Prerequisites check complete"
}

# Setup Solana environment
setup_solana() {
    echo "⚙️ Setting up Solana environment..."
    
    # Set to devnet
    solana config set --url devnet
    
    # Generate keypair if doesn't exist
    if [ ! -f ~/.config/solana/id.json ]; then
        solana-keygen new --no-bip39-passphrase
    fi
    
    # Request airdrop
    solana airdrop 2
    
    echo "✅ Solana environment setup complete"
}

# Install dependencies
install_deps() {
    echo "📦 Installing dependencies..."
    
    # Yarn workspaces
    yarn install
    
    # Rust dependencies
    cargo build
    
    # Python dependencies (for AI services)
    if command -v pip3 &> /dev/null; then
        pip3 install -r ai-services/requirements.txt
    fi
    
    echo "✅ Dependencies installed"
}

# Setup database
setup_database() {
    echo "🗄️ Setting up database..."
    
    # Start PostgreSQL and Redis with Docker
    docker-compose up -d postgres redis
    
    # Wait for services to be ready
    sleep 10
    
    # Run migrations
    ./scripts/migration/migrate-database.sh
    
    # Seed test data
    ./scripts/development/seed-test-data.sh
    
    echo "✅ Database setup complete"
}

# Setup configuration
setup_config() {
    echo "⚙️ Setting up configuration..."
    
    # Copy environment template
    if [ ! -f .env ]; then
        cp .env.example .env
        echo "📝 Please edit .env file with your configuration"
    fi
    
    echo "✅ Configuration setup complete"
}

# Main setup flow
main() {
    check_prereqs
    setup_solana
    install_deps
    setup_database
    setup_config
    
    echo "🎉 Development environment setup complete!"
    echo ""
    echo "Next steps:"
    echo "1. Edit .env file with your configuration"
    echo "2. Run 'make build' to build all components"
    echo "3. Run 'make test' to verify everything works"
    echo "4. Start developing! 🚀"
}

main "$@"
```

## Continuous Integration

Our CI/CD pipeline ensures code quality and security:

### GitHub Actions Workflow

```yaml
# .github/workflows/ci.yml
name: Continuous Integration

on:
  push:
    branches: [ main, develop ]
  pull_request:
    branches: [ main, develop ]

env:
  CARGO_TERM_COLOR: always
  SOLANA_VERSION: 1.16.0
  ANCHOR_VERSION: 0.28.0

jobs:
  lint:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      
      - name: Setup Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          components: rustfmt, clippy
      
      - name: Setup Node.js
        uses: actions/setup-node@v3
        with:
          node-version: '18'
          cache: 'yarn'
      
      - name: Install dependencies
        run: yarn install --frozen-lockfile
      
      - name: Rust format check
        run: cargo fmt --all -- --check
      
      - name: Rust clippy
        run: cargo clippy --all-targets --all-features -- -D warnings
      
      - name: TypeScript lint
        run: yarn lint
      
      - name: Python lint
        run: |
          pip install flake8 black isort
          flake8 ai-services/
          black --check ai-services/
          isort --check-only ai-services/

  test:
    runs-on: ubuntu-latest
    services:
      postgres:
        image: postgres:14
        env:
          POSTGRES_PASSWORD: postgres
        options: >-
          --health-cmd pg_isready
          --health-interval 10s
          --health-timeout 5s
          --health-retries 5
      
      redis:
        image: redis:6
        options: >-
          --health-cmd "redis-cli ping"
          --health-interval 10s
          --health-timeout 5s
          --health-retries 5
    
    steps:
      - uses: actions/checkout@v3
      
      - name: Setup Solana
        uses: solana-labs/setup-solana@v1
        with:
          solana-version: ${{ env.SOLANA_VERSION }}
      
      - name: Setup Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      
      - name: Setup Node.js
        uses: actions/setup-node@v3
        with:
          node-version: '18'
          cache: 'yarn'
      
      - name: Install Anchor
        run: npm install -g @coral-xyz/anchor-cli@${{ env.ANCHOR_VERSION }}
      
      - name: Install dependencies
        run: yarn install --frozen-lockfile
      
      - name: Build Anchor programs
        run: anchor build
      
      - name: Test Anchor programs
        run: anchor test --skip-deploy
      
      - name: Test TypeScript
        run: yarn test --coverage
        env:
          DATABASE_URL: postgresql://postgres:postgres@localhost:5432/finova_test
          REDIS_URL: redis://localhost:6379
      
      - name: Test Python
        run: |
          cd ai-services
          pip install -r requirements.txt
          pip install -r requirements-test.txt
          pytest --cov=src --cov-report=xml
      
      - name: Upload coverage
        uses: codecov/codecov-action@v3
        with:
          files: ./coverage/lcov.info,./ai-services/coverage.xml

  security:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      
      - name: Setup Node.js
        uses: actions/setup-node@v3
        with:
          node-version: '18'
          cache: 'yarn'
      
      - name: Install dependencies
        run: yarn install --frozen-lockfile
      
      - name: Audit npm packages
        run: yarn audit --audit-level moderate
      
      - name: Rust security audit
        uses: actions-rs/audit-check@v1
        with:
          token: ${{ secrets.GITHUB_TOKEN }}
      
      - name: Python security scan
        run: |
          pip install safety bandit
          safety check -r ai-services/requirements.txt
          bandit -r ai-services/ -f json -o bandit-report.json
      
      - name: CodeQL Analysis
        uses: github/codeql-action/analyze@v2
        with:
          languages: javascript, python

  build:
    runs-on: ubuntu-latest
    needs: [lint, test, security]
    steps:
      - uses: actions/checkout@v3
      
      - name: Setup Docker Buildx
        uses: docker/setup-buildx-action@v2
      
      - name: Build API image
        uses: docker/build-push-action@v4
        with:
          context: .
          file: infrastructure/docker/Dockerfile.api
          push: false
          tags: finova-api:latest
          cache-from: type=gha
          cache-to: type=gha,mode=max
      
      - name: Build AI services image
        uses: docker/build-push-action@v4
        with:
          context: ./ai-services
          file: infrastructure/docker/Dockerfile.ai-services
          push: false
          tags: finova-ai:latest
          cache-from: type=gha
          cache-to: type=gha,mode=max
```

## Final Notes

Thank you for contributing to Finova Network! Your contributions help build the future of social-fi and Web3 social media monetization.

### Key Reminders

1. **Security First**: Always prioritize security in your contributions
2. **Test Thoroughly**: Comprehensive testing prevents production issues
3. **Document Everything**: Good documentation helps the entire community
4. **Ask Questions**: Don't hesitate to ask for help or clarification
5. **Stay Updated**: Keep up with project updates and changes

### Contact Information

- **General Questions**: developers@finova.network
- **Security Issues**: security@finova.network
- **Partnership Inquiries**: partnerships@finova.network
- **Community Discord**: [discord.gg/finova-network](https://discord.gg/finova-network)
- **GitHub Discussions**: [github.com/finova-network/finova-contracts/discussions](https://github.com/finova-network/finova-contracts/discussions)

### License

By contributing to Finova Network, you agree that your contributions will be licensed under the same license as the project.

---

*This contributing guide is a living document and will be updated as the project evolves. Last updated: July 29, 2025*
