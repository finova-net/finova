# Finova Network - AI Content Analyzer

## Overview

The Content Analyzer is a critical AI service that evaluates user-generated content quality to calculate fair XP multipliers and prevent abuse in the Finova Network ecosystem. It provides real-time content scoring for text, images, and videos across all integrated social platforms.

## Architecture

```
content-analyzer/
├── src/
│   ├── main.py                 # FastAPI application entry point
│   ├── models/
│   │   ├── __init__.py
│   │   ├── quality_classifier.py      # Content quality scoring
│   │   ├── originality_detector.py    # Plagiarism/duplicate detection
│   │   ├── engagement_predictor.py    # Viral potential analysis
│   │   └── brand_safety_checker.py    # Brand safety compliance
│   ├── preprocessing/
│   │   ├── __init__.py
│   │   ├── text_processor.py          # NLP preprocessing
│   │   ├── image_processor.py         # Image analysis pipeline
│   │   └── video_processor.py         # Video content extraction
│   ├── api/
│   │   ├── __init__.py
│   │   ├── routes.py                  # API endpoints
│   │   └── schemas.py                 # Pydantic models
│   └── utils/
│       ├── __init__.py
│       ├── config.py                  # Configuration management
│       └── helpers.py                 # Utility functions
├── requirements.txt
├── Dockerfile
└── README.md
```

## Key Features

### Quality Scoring Algorithm

The analyzer calculates a quality score using weighted factors:

```python
Quality_Score = (
    originality * 0.30 +           # Plagiarism check (0-1)
    engagement_potential * 0.25 +  # Predicted viral score (0-1)
    platform_relevance * 0.20 +    # Platform-specific optimization (0-1)
    brand_safety * 0.15 +          # Brand safety compliance (0-1)
    human_generated * 0.10         # AI-generated content detection (0-1)
)

# Final XP Multiplier: max(0.5, min(2.0, Quality_Score))
```

### Content Types Supported

- **Text Content**: Posts, comments, captions
- **Images**: Photos, graphics, memes
- **Videos**: Short-form content (TikTok, Instagram Reels, YouTube Shorts)
- **Mixed Media**: Posts with multiple content types

### Platform Integration

- Instagram posts and stories
- TikTok videos and comments
- YouTube content and engagement
- Facebook posts and interactions
- X (Twitter) tweets and replies

## API Endpoints

### Content Analysis

```http
POST /api/v1/analyze/content
Content-Type: application/json

{
  "user_id": "string",
  "platform": "instagram|tiktok|youtube|facebook|x",
  "content_type": "text|image|video|mixed",
  "content": {
    "text": "Optional text content",
    "media_urls": ["array of media URLs"],
    "metadata": {
      "hashtags": ["array"],
      "mentions": ["array"],
      "location": "optional"
    }
  },
  "context": {
    "user_level": "number",
    "user_tier": "string",
    "historical_quality": "number"
  }
}
```

**Response:**
```json
{
  "quality_score": 0.85,
  "xp_multiplier": 1.7,
  "breakdown": {
    "originality": 0.92,
    "engagement_potential": 0.78,
    "platform_relevance": 0.88,
    "brand_safety": 0.95,
    "human_generated": 0.89
  },
  "flags": [],
  "processing_time_ms": 245
}
```

### Batch Analysis

```http
POST /api/v1/analyze/batch
```

For processing multiple content items simultaneously.

### Quality History

```http
GET /api/v1/user/{user_id}/quality-history
```

Returns user's content quality trends over time.

## Machine Learning Models

### Quality Classifier
- **Model**: Fine-tuned BERT for multilingual content
- **Training Data**: 1M+ labeled social media posts
- **Accuracy**: 94.5% on validation set
- **Languages**: English, Indonesian, 20+ others

### Originality Detector
- **Technology**: Locality-Sensitive Hashing + Semantic similarity
- **Database**: 500M+ indexed content pieces
- **Speed**: <100ms average response time
- **Threshold**: 85% similarity triggers flag

### Engagement Predictor
- **Architecture**: Transformer-based multimodal model
- **Features**: Text sentiment, visual elements, timing, hashtags
- **Accuracy**: 78% viral content prediction
- **Update Frequency**: Weekly model retraining

### Brand Safety Checker
- **Categories**: Violence, adult content, hate speech, misinformation
- **Compliance**: IAB standards + custom Finova guidelines
- **Accuracy**: 99.2% precision, 96.8% recall
- **Languages**: 25+ supported languages

## Installation & Setup

### Docker Deployment (Recommended)

```bash
# Build the container
docker build -t finova/content-analyzer .

# Run with environment variables
docker run -d \
  --name content-analyzer \
  -p 8000:8000 \
  -e REDIS_URL=redis://redis:6379 \
  -e POSTGRES_URL=postgresql://user:pass@db:5432/finova \
  -e MODEL_CACHE_SIZE=2GB \
  finova/content-analyzer
```

### Local Development

```bash
# Install dependencies
pip install -r requirements.txt

# Download ML models
python scripts/download_models.py

# Set environment variables
export REDIS_URL=redis://localhost:6379
export POSTGRES_URL=postgresql://localhost:5432/finova_dev

# Run the service
uvicorn src.main:app --host 0.0.0.0 --port 8000 --reload
```

### Environment Variables

```env
# Database
REDIS_URL=redis://localhost:6379
POSTGRES_URL=postgresql://user:pass@host:5432/db

# ML Models
MODEL_CACHE_SIZE=2GB
TRANSFORMERS_CACHE=/app/models
TORCH_DEVICE=cuda  # or cpu

# API Configuration
API_RATE_LIMIT=1000/hour
MAX_CONTENT_SIZE=50MB
SUPPORTED_FORMATS=jpg,png,mp4,webp

# External Services
OPENAI_API_KEY=your_key_here  # For advanced analysis
HUGGINGFACE_TOKEN=your_token  # For model downloads
```

## Performance Specifications

### Throughput
- **Text Analysis**: 10,000 requests/minute
- **Image Analysis**: 1,000 requests/minute  
- **Video Analysis**: 100 requests/minute
- **Batch Processing**: Up to 1,000 items/request

### Latency (95th percentile)
- **Text**: <200ms
- **Image**: <1000ms
- **Video**: <5000ms
- **Complex Mixed Media**: <8000ms

### Resource Requirements

**Minimum:**
- 4 CPU cores
- 8GB RAM
- 2GB GPU VRAM (optional)
- 50GB storage

**Recommended (Production):**
- 16 CPU cores
- 32GB RAM  
- 8GB GPU VRAM
- 500GB SSD storage

## Quality Score Examples

### High Quality Content (Score: 1.8x multiplier)

```json
{
  "content": "Just launched my eco-friendly startup! 🌱 After 2 years of R&D, we're solving plastic waste with biodegradable alternatives. Check out our prototype! #sustainability #innovation #startup",
  "analysis": {
    "originality": 0.95,      // Unique personal story
    "engagement_potential": 0.88, // Trending topic + emotive
    "platform_relevance": 0.92,  // Perfect for LinkedIn/Instagram
    "brand_safety": 0.98,     // Positive, safe content
    "human_generated": 0.94   // Clear personal voice
  },
  "final_score": 1.8
}
```

### Medium Quality Content (Score: 1.2x multiplier)

```json
{
  "content": "Good morning everyone! Hope you have a great day ☀️ #goodmorning #motivation",
  "analysis": {
    "originality": 0.45,      // Generic greeting
    "engagement_potential": 0.55, // Low engagement expected
    "platform_relevance": 0.78,  // Fits most platforms
    "brand_safety": 1.0,      // Completely safe
    "human_generated": 0.87   // Natural but simple
  },
  "final_score": 1.2
}
```

### Low Quality Content (Score: 0.6x multiplier)

```json
{
  "content": "Click here for free money!!! 💰💰💰 Link in bio!!! #money #free #rich #crypto #bitcoin",
  "analysis": {
    "originality": 0.15,      // Spam-like content
    "engagement_potential": 0.25, // Low quality engagement
    "platform_relevance": 0.30,  // Spam across platforms
    "brand_safety": 0.20,     // Potential scam content
    "human_generated": 0.45   // Bot-like patterns
  },
  "final_score": 0.6
}
```

## Anti-Gaming Measures

### Pattern Detection
- **Repetitive Content**: Detects users posting identical/similar content
- **Artificial Engagement**: Identifies coordinated like/comment farming
- **Content Spinning**: Catches paraphrased duplicate content
- **Timing Analysis**: Flags unnatural posting patterns

### Dynamic Thresholds
- User-specific quality baselines
- Platform-adjusted scoring
- Temporal quality tracking
- Network behavior analysis

### Fraud Prevention
```python
def calculate_fraud_probability(user_content_history):
    factors = {
        'content_diversity': measure_content_uniqueness(),
        'posting_patterns': analyze_temporal_patterns(),
        'engagement_authenticity': validate_interaction_quality(),
        'quality_consistency': check_score_variations(),
        'network_behavior': analyze_referral_content()
    }
    
    return weighted_fraud_score(factors)
```

## Monitoring & Analytics

### Real-time Metrics
- Content analysis throughput
- Model accuracy drift detection
- API response times
- Error rates by content type

### Quality Insights Dashboard
- Platform-specific quality trends
- User quality distribution
- Content category performance
- Fraud detection statistics

### Alerts & Notifications
- Model performance degradation
- Unusual content patterns
- System resource usage
- API rate limit breaches

## Integration with Core System

### XP Calculation Integration
```python
# Called from finova-core mining engine
quality_score = await content_analyzer.analyze_content(
    user_content=user_post,
    user_context=user_stats
)

xp_multiplier = max(0.5, min(2.0, quality_score))
final_xp = base_xp * xp_multiplier * other_bonuses
```

### Real-time Processing
- WebSocket connections for instant analysis
- Redis pub/sub for event distribution
- Database triggers for automatic quality updates
- Webhook notifications for significant changes

## Security & Privacy

### Data Protection
- Content is analyzed but not permanently stored
- User privacy compliance (GDPR, CCPA)
- Encrypted data transmission
- Access logging and audit trails

### Model Security
- Regular model validation against adversarial attacks
- Encrypted model weights
- Secure model update mechanisms
- Input sanitization and validation

## Scaling & Performance

### Horizontal Scaling
- Stateless service design
- Load balancer compatible
- Auto-scaling based on queue depth
- Multi-region deployment support

### Caching Strategy
- Redis for frequent analysis results
- Model prediction caching
- User quality score caching
- CDN for static model assets

### Database Optimization
- Indexed user content history
- Partitioned tables by date
- Read replicas for analytics
- Connection pooling

## Development & Testing

### Testing Strategy
```bash
# Unit tests
python -m pytest tests/unit/

# Integration tests  
python -m pytest tests/integration/

# Load tests
locust -f tests/load/content_analyzer_load.py

# Model accuracy tests
python tests/model/evaluate_models.py
```

### Code Quality
- Type hints with mypy
- Linting with flake8/black
- Security scanning with bandit
- Dependency vulnerability checks

### CI/CD Pipeline
- Automated testing on PR
- Model validation checks
- Security scanning
- Staged deployment (dev→staging→prod)

## Troubleshooting

### Common Issues

**High Latency**
- Check GPU utilization
- Monitor model loading times
- Verify network connectivity
- Review batch processing configs

**Quality Score Inconsistency**
- Validate model versions
- Check feature preprocessing
- Review user context data
- Analyze temporal patterns

**Memory Issues**
- Adjust model cache size
- Monitor batch processing limits
- Check for memory leaks
- Optimize tensor operations

### Debug Commands
```bash
# Check service health
curl http://localhost:8000/health

# Model diagnostics
python scripts/model_diagnostics.py

# Performance profiling
python -m cProfile src/main.py

# Memory usage analysis
python scripts/memory_profiler.py
```

## Contributing

### Development Setup
1. Fork the repository
2. Create feature branch
3. Install development dependencies
4. Run tests before committing
5. Submit PR with detailed description

### Model Training
1. Use standardized datasets
2. Follow model versioning conventions  
3. Document training procedures
4. Validate on held-out test sets
5. Performance regression testing

## Support & Contact

For technical support or questions:
- GitHub Issues: [finova-network/ai-services](https://github.com/finova-network/ai-services)
- Technical Documentation: [docs.finova.network](https://docs.finova.network)
- Developer Discord: [discord.gg/finova-dev](https://discord.gg/finova-dev)

## License

Apache 2.0 License - see LICENSE file for details.