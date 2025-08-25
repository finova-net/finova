# finova-net/finova/client/rust/README.md

# Finova Network - Rust Client SDK

[![Crates.io](https://img.shields.io/crates/v/finova-client)](https://crates.io/crates/finova-client)
[![Documentation](https://docs.rs/finova-client/badge.svg)](https://docs.rs/finova-client)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

High-performance Rust client for the Finova Network - the next-generation Social-Fi Super App that integrates XP, RP, and $FIN mining systems.

## Quick Start

```toml
[dependencies]
finova-client = "0.1.0"
solana-client = "1.16"
anchor-client = "0.28"
tokio = { version = "1.0", features = ["full"] }
```

```rust
use finova_client::{FinovaClient, Config, Network};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = Config::new(Network::Devnet, "your-rpc-url");
    let client = FinovaClient::new(config).await?;
    
    // Start mining
    let mining_result = client.start_mining().await?;
    println!("Mining started: {}", mining_result.transaction_id);
    
    // Get user stats
    let stats = client.get_user_stats().await?;
    println!("XP: {}, RP: {}, $FIN: {}", stats.xp, stats.rp, stats.fin_balance);
    
    Ok(())
}
```

## Core Features

### 🚀 Mining System
- **Exponential Regression Mining**: Pi Network-inspired fair distribution
- **Real-time Rate Calculation**: Dynamic mining rates based on network growth
- **Anti-Bot Protection**: Advanced pattern detection and human verification

### 🎮 XP System (Hamster Kombat-inspired)
- **Multi-Platform Integration**: Instagram, TikTok, YouTube, Facebook, X
- **Quality-Based Rewards**: AI-powered content analysis
- **Level Progression**: 100+ levels with exponential XP requirements

### 🌐 Referral Points (RP)
- **Network Effect Amplification**: Up to 3-level referral rewards
- **Quality Network Building**: Rewards for active, engaged referrals
- **Ambassador Program**: Elite status for top referrers

### 🎴 NFT & Special Cards
- **Mining Boost Cards**: Temporary mining rate multipliers
- **XP Accelerators**: Enhanced experience point earning
- **Referral Power Cards**: Network effect amplifiers

## Installation & Setup

### Prerequisites
```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install Solana CLI
sh -c "$(curl -sSfL https://release.solana.com/v1.16.0/install)"
```

### Configuration
```rust
use finova_client::Config;

let config = Config {
    network: Network::Mainnet,
    rpc_url: "https://api.mainnet-beta.solana.com".to_string(),
    commitment: CommitmentConfig::confirmed(),
    timeout: Duration::from_secs(30),
};
```

## API Reference

### Core Client Methods

#### Mining Operations
```rust
// Start mining session
client.start_mining().await?;

// Stop mining session  
client.stop_mining().await?;

// Get current mining rate
let rate = client.get_mining_rate().await?;

// Claim mined tokens
let claim_result = client.claim_mining_rewards().await?;
```

#### XP System
```rust
// Add social media activity
let activity = SocialActivity {
    platform: Platform::TikTok,
    content_type: ContentType::Video,
    engagement_metrics: EngagementMetrics::new(likes, comments, shares),
    quality_score: 0.85, // AI-analyzed quality
};
client.add_xp_activity(activity).await?;

// Get XP statistics
let xp_stats = client.get_xp_stats().await?;
```

#### Referral System
```rust
// Create referral code
let referral_code = client.create_referral_code().await?;

// Process referral signup
client.process_referral_signup("REF123").await?;

// Get referral network stats
let network_stats = client.get_referral_network().await?;
```

#### NFT & Special Cards
```rust
// Use special card
let card_id = "mining_boost_24h";
client.use_special_card(card_id).await?;

// Get user NFTs
let nfts = client.get_user_nfts().await?;

// Purchase NFT from marketplace
client.purchase_nft(nft_id, price).await?;
```

### Data Types

#### User Statistics
```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct UserStats {
    pub user_id: String,
    pub xp_level: u32,
    pub total_xp: u64,
    pub rp_tier: RpTier,
    pub total_rp: u64,
    pub fin_balance: u64,
    pub mining_rate: f64,
    pub network_size: u32,
    pub active_referrals: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum RpTier {
    Explorer,     // 0-999 RP
    Connector,    // 1,000-4,999 RP
    Influencer,   // 5,000-14,999 RP
    Leader,       // 15,000-49,999 RP
    Ambassador,   // 50,000+ RP
}
```

#### Mining Calculations
```rust
#[derive(Debug)]
pub struct MiningRate {
    pub base_rate: f64,
    pub pioneer_bonus: f64,
    pub referral_bonus: f64,
    pub xp_multiplier: f64,
    pub rp_multiplier: f64,
    pub quality_multiplier: f64,
    pub regression_factor: f64,
    pub final_rate: f64,
}

impl MiningRate {
    pub fn calculate(&self) -> f64 {
        self.base_rate 
            * self.pioneer_bonus 
            * self.referral_bonus 
            * self.xp_multiplier 
            * self.rp_multiplier 
            * self.quality_multiplier 
            * self.regression_factor
    }
}
```

## Advanced Usage

### Real-time Mining Monitor
```rust
use finova_client::events::MiningEventStream;

let mut event_stream = client.subscribe_mining_events().await?;

while let Some(event) = event_stream.next().await {
    match event {
        MiningEvent::RateUpdate { new_rate, .. } => {
            println!("Mining rate updated: {:.6} $FIN/hour", new_rate);
        }
        MiningEvent::RewardClaimed { amount, .. } => {
            println!("Claimed: {:.6} $FIN", amount);
        }
        MiningEvent::LevelUp { new_level, .. } => {
            println!("Level up! Now level {}", new_level);
        }
    }
}
```

### Batch Operations
```rust
use finova_client::batch::BatchBuilder;

let batch = BatchBuilder::new()
    .add_xp_activity(activity1)
    .add_xp_activity(activity2)
    .use_special_card("xp_double_24h")
    .claim_mining_rewards()
    .build();

let results = client.execute_batch(batch).await?;
```

### Custom Network Configuration
```rust
let config = Config::builder()
    .network(Network::Custom {
        rpc_url: "https://your-custom-rpc.com".to_string(),
        program_ids: ProgramIds {
            finova_core: "FinCore11111111111111111111111111111111".parse()?,
            finova_token: "FinToken1111111111111111111111111111111".parse()?,
            finova_nft: "FinNFT111111111111111111111111111111111".parse()?,
        },
    })
    .commitment(CommitmentConfig::finalized())
    .timeout(Duration::from_secs(60))
    .retry_policy(RetryPolicy::exponential_backoff(3))
    .build();
```

## Error Handling

```rust
use finova_client::error::{FinovaError, ErrorKind};

match client.start_mining().await {
    Ok(result) => println!("Mining started: {}", result.transaction_id),
    Err(FinovaError::InsufficientBalance { required, available }) => {
        eprintln!("Need {:.6} $FIN, have {:.6}", required, available);
    }
    Err(FinovaError::RateLimited { retry_after }) => {
        eprintln!("Rate limited, retry after {} seconds", retry_after);
    }
    Err(FinovaError::BotDetected { confidence }) => {
        eprintln!("Bot detection triggered (confidence: {:.2})", confidence);
    }
    Err(e) => eprintln!("Error: {}", e),
}
```

## Integration Examples

### Web Server Integration (Axum)
```rust
use axum::{Json, extract::State};

#[derive(Clone)]
struct AppState {
    finova_client: Arc<FinovaClient>,
}

async fn get_user_stats(
    State(state): State<AppState>,
    Json(req): Json<GetStatsRequest>,
) -> Result<Json<UserStats>, AppException> {
    let stats = state.finova_client
        .get_user_stats_by_id(&req.user_id)
        .await?;
    
    Ok(Json(stats))
}
```

### Background Mining Service
```rust
use tokio::time::{interval, Duration};

async fn mining_service(client: Arc<FinovaClient>) -> anyhow::Result<()> {
    let mut interval = interval(Duration::from_secs(3600)); // Every hour
    
    loop {
        interval.tick().await;
        
        // Auto-claim rewards for all users
        let active_miners = client.get_active_miners().await?;
        
        for miner in active_miners {
            if let Err(e) = client.claim_mining_rewards_for(&miner.user_id).await {
                log::warn!("Failed to claim for {}: {}", miner.user_id, e);
            }
        }
    }
}
```

## Security & Best Practices

### Wallet Integration
```rust
use solana_sdk::signer::Signer;

// Use hardware wallet for production
let keypair = read_keypair_file("~/.config/solana/id.json")?;
let client = FinovaClient::with_signer(config, Arc::new(keypair)).await?;

// Or use environment variable
let private_key = std::env::var("FINOVA_PRIVATE_KEY")?;
let keypair = Keypair::from_base58_string(&private_key);
```

### Rate Limiting
```rust
use finova_client::middleware::RateLimiter;

let rate_limiter = RateLimiter::new(100, Duration::from_secs(60)); // 100 req/min
let client = FinovaClient::with_middleware(config, rate_limiter).await?;
```

### Logging
```rust
use tracing_subscriber;

tracing_subscriber::fmt()
    .with_max_level(tracing::Level::INFO)
    .init();

// Client automatically logs important events
let client = FinovaClient::with_logging(config, true).await?;
```

## Performance Optimization

### Connection Pooling
```rust
let config = Config::builder()
    .connection_pool_size(10)
    .max_concurrent_requests(50)
    .build();
```

### Caching
```rust
use finova_client::cache::MemoryCache;

let cache = MemoryCache::with_ttl(Duration::from_secs(300)); // 5min TTL
let client = FinovaClient::with_cache(config, cache).await?;
```

## Testing

### Unit Tests
```bash
cargo test --lib
```

### Integration Tests
```bash
# Requires local Solana test validator
solana-test-validator &
cargo test --test integration_tests
```

### Mock Client for Testing
```rust
use finova_client::mock::MockFinovaClient;

let mock_client = MockFinovaClient::new()
    .with_user_stats(UserStats::default())
    .with_mining_rate(0.05)
    .build();
```

## Troubleshooting

### Common Issues

**Connection Timeouts**
```rust
let config = Config::builder()
    .timeout(Duration::from_secs(120))
    .retry_policy(RetryPolicy::fixed_delay(3, Duration::from_secs(2)))
    .build();
```

**RPC Rate Limits**
```rust
// Use multiple RPC endpoints
let config = Config::with_rpc_pool(vec![
    "https://api.mainnet-beta.solana.com",
    "https://solana-api.projectserum.com",
    "https://rpc.ankr.com/solana",
]);
```

**Memory Usage**
```rust
// Optimize for memory-constrained environments
let config = Config::builder()
    .memory_efficient(true)
    .max_cache_size(1024 * 1024) // 1MB cache
    .build();
```

## Examples

Check the [`examples/`](examples/) directory for complete examples:

- [`basic_mining.rs`](examples/basic_mining.rs) - Simple mining operations
- [`social_integration.rs`](examples/social_integration.rs) - XP system integration
- [`referral_network.rs`](examples/referral_network.rs) - Building referral networks
- [`nft_trading.rs`](examples/nft_trading.rs) - NFT marketplace operations
- [`websocket_events.rs`](examples/websocket_events.rs) - Real-time event handling

## Contributing

1. Fork the repository
2. Create feature branch (`git checkout -b feature/amazing-feature`)
3. Commit changes (`git commit -m 'Add amazing feature'`)
4. Push to branch (`git push origin feature/amazing-feature`)
5. Open Pull Request

### Development Setup
```bash
git clone https://github.com/finova-network/finova-contracts
cd finova-contracts/client/rust
cargo build
cargo test
```

## Documentation

- [API Documentation](https://docs.rs/finova-client)
- [Whitepaper](../../docs/whitepaper/)
- [Technical Architecture](../../docs/architecture/)
- [Integration Guides](../../docs/integration/)

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Support

- **Discord**: [Join our community](https://discord.gg/finova)
- **Telegram**: [Finova Network Official](https://t.me/finova_network)
- **Email**: dev@finova.network
- **Documentation**: [docs.finova.network](https://docs.finova.network)

---

**Finova Network** - Where Every Interaction Has Value 🚀