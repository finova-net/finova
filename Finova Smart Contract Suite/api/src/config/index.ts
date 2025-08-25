import { z } from 'zod';
import dotenv from 'dotenv';

// Load environment variables
dotenv.config();

// Environment validation schema
const envSchema = z.object({
  // Server Configuration
  NODE_ENV: z.enum(['development', 'staging', 'testnet', 'mainnet', 'production']).default('development'),
  PORT: z.string().transform(val => parseInt(val, 10)).default('3000'),
  API_VERSION: z.string().default('v1'),
  
  // Database Configuration
  DATABASE_URL: z.string(),
  DATABASE_HOST: z.string().default('localhost'),
  DATABASE_PORT: z.string().transform(val => parseInt(val, 10)).default('5432'),
  DATABASE_NAME: z.string().default('finova_network'),
  DATABASE_USER: z.string(),
  DATABASE_PASSWORD: z.string(),
  DATABASE_SSL: z.string().transform(val => val === 'true').default('false'),
  DATABASE_POOL_MIN: z.string().transform(val => parseInt(val, 10)).default('2'),
  DATABASE_POOL_MAX: z.string().transform(val => parseInt(val, 10)).default('10'),
  
  // Redis Configuration
  REDIS_URL: z.string().optional(),
  REDIS_HOST: z.string().default('localhost'),
  REDIS_PORT: z.string().transform(val => parseInt(val, 10)).default('6379'),
  REDIS_PASSWORD: z.string().optional(),
  REDIS_DB: z.string().transform(val => parseInt(val, 10)).default('0'),
  REDIS_TTL: z.string().transform(val => parseInt(val, 10)).default('3600'),
  
  // Blockchain Configuration
  SOLANA_RPC_URL: z.string(),
  SOLANA_WSS_URL: z.string(),
  SOLANA_NETWORK: z.enum(['devnet', 'testnet', 'mainnet-beta']).default('devnet'),
  SOLANA_COMMITMENT: z.enum(['processed', 'confirmed', 'finalized']).default('confirmed'),
  
  // Program IDs
  FINOVA_CORE_PROGRAM_ID: z.string(),
  FINOVA_TOKEN_PROGRAM_ID: z.string(),
  FINOVA_NFT_PROGRAM_ID: z.string(),
  FINOVA_DEFI_PROGRAM_ID: z.string(),
  FINOVA_BRIDGE_PROGRAM_ID: z.string(),
  FINOVA_ORACLE_PROGRAM_ID: z.string(),
  
  // JWT Configuration
  JWT_SECRET: z.string().min(32),
  JWT_EXPIRES_IN: z.string().default('24h'),
  JWT_REFRESH_EXPIRES_IN: z.string().default('7d'),
  JWT_ISSUER: z.string().default('finova-network'),
  JWT_AUDIENCE: z.string().default('finova-users'),
  
  // Encryption
  ENCRYPTION_KEY: z.string().min(32),
  ENCRYPTION_ALGORITHM: z.string().default('aes-256-gcm'),
  HASH_SALT_ROUNDS: z.string().transform(val => parseInt(val, 10)).default('12'),
  
  // Rate Limiting
  RATE_LIMIT_WINDOW_MS: z.string().transform(val => parseInt(val, 10)).default('900000'), // 15 minutes
  RATE_LIMIT_MAX_REQUESTS: z.string().transform(val => parseInt(val, 10)).default('100'),
  RATE_LIMIT_SKIP_SUCCESSFUL_REQUESTS: z.string().transform(val => val === 'true').default('false'),
  
  // CORS Configuration
  CORS_ORIGIN: z.string().optional(),
  CORS_METHODS: z.string().default('GET,HEAD,PUT,PATCH,POST,DELETE'),
  CORS_ALLOWED_HEADERS: z.string().default('Content-Type,Authorization,X-Requested-With'),
  
  // KYC Configuration
  KYC_PROVIDER: z.enum(['jumio', 'onfido', 'sumsub']).default('sumsub'),
  KYC_API_URL: z.string(),
  KYC_API_KEY: z.string(),
  KYC_SECRET_KEY: z.string(),
  KYC_WEBHOOK_SECRET: z.string(),
  
  // Social Media Integration
  INSTAGRAM_CLIENT_ID: z.string().optional(),
  INSTAGRAM_CLIENT_SECRET: z.string().optional(),
  TIKTOK_CLIENT_ID: z.string().optional(),
  TIKTOK_CLIENT_SECRET: z.string().optional(),
  YOUTUBE_API_KEY: z.string().optional(),
  FACEBOOK_APP_ID: z.string().optional(),
  FACEBOOK_APP_SECRET: z.string().optional(),
  TWITTER_API_KEY: z.string().optional(),
  TWITTER_API_SECRET: z.string().optional(),
  
  // Payment Gateway Configuration
  OVO_MERCHANT_ID: z.string().optional(),
  OVO_API_KEY: z.string().optional(),
  GOPAY_MERCHANT_ID: z.string().optional(),
  GOPAY_API_KEY: z.string().optional(),
  DANA_MERCHANT_ID: z.string().optional(),
  DANA_API_KEY: z.string().optional(),
  SHOPEEPAY_MERCHANT_ID: z.string().optional(),
  SHOPEEPAY_API_KEY: z.string().optional(),
  
  // AI Services Configuration
  AI_CONTENT_ANALYZER_URL: z.string(),
  AI_BOT_DETECTION_URL: z.string(),
  AI_RECOMMENDATION_URL: z.string(),
  AI_ANALYTICS_URL: z.string(),
  AI_SERVICE_API_KEY: z.string(),
  
  // Email Configuration
  SMTP_HOST: z.string().optional(),
  SMTP_PORT: z.string().transform(val => parseInt(val, 10)).default('587'),
  SMTP_USER: z.string().optional(),
  SMTP_PASS: z.string().optional(),
  EMAIL_FROM: z.string().default('noreply@finova.network'),
  
  // AWS Configuration
  AWS_REGION: z.string().default('ap-southeast-1'),
  AWS_ACCESS_KEY_ID: z.string().optional(),
  AWS_SECRET_ACCESS_KEY: z.string().optional(),
  AWS_S3_BUCKET: z.string().optional(),
  AWS_CLOUDFRONT_DOMAIN: z.string().optional(),
  
  // Monitoring & Logging
  LOG_LEVEL: z.enum(['error', 'warn', 'info', 'debug']).default('info'),
  SENTRY_DSN: z.string().optional(),
  NEW_RELIC_LICENSE_KEY: z.string().optional(),
  PROMETHEUS_ENABLED: z.string().transform(val => val === 'true').default('false'),
  
  // WebSocket Configuration
  WS_PORT: z.string().transform(val => parseInt(val, 10)).default('3001'),
  WS_PING_INTERVAL: z.string().transform(val => parseInt(val, 10)).default('25000'),
  WS_PONG_TIMEOUT: z.string().transform(val => parseInt(val, 10)).default('5000'),
  
  // Mining Configuration
  MINING_BASE_RATE: z.string().transform(val => parseFloat(val)).default('0.05'),
  MINING_PHASE_THRESHOLD_1: z.string().transform(val => parseInt(val, 10)).default('100000'),
  MINING_PHASE_THRESHOLD_2: z.string().transform(val => parseInt(val, 10)).default('1000000'),
  MINING_PHASE_THRESHOLD_3: z.string().transform(val => parseInt(val, 10)).default('10000000'),
  MINING_REGRESSION_FACTOR: z.string().transform(val => parseFloat(val)).default('0.001'),
  
  // XP System Configuration
  XP_LEVEL_PROGRESSION_FACTOR: z.string().transform(val => parseFloat(val)).default('0.01'),
  XP_DAILY_STREAK_MAX: z.string().transform(val => parseInt(val, 10)).default('30'),
  XP_QUALITY_SCORE_MIN: z.string().transform(val => parseFloat(val)).default('0.5'),
  XP_QUALITY_SCORE_MAX: z.string().transform(val => parseFloat(val)).default('2.0'),
  
  // RP System Configuration
  RP_NETWORK_REGRESSION_FACTOR: z.string().transform(val => parseFloat(val)).default('0.0001'),
  RP_TIER_THRESHOLDS: z.string().default('1000,5000,15000,50000'),
  RP_MAX_REFERRAL_DEPTH: z.string().transform(val => parseInt(val, 10)).default('3'),
  
  // Security Configuration
  SECURITY_MAX_LOGIN_ATTEMPTS: z.string().transform(val => parseInt(val, 10)).default('5'),
  SECURITY_LOCKOUT_DURATION: z.string().transform(val => parseInt(val, 10)).default('900'), // 15 minutes
  SECURITY_SESSION_TIMEOUT: z.string().transform(val => parseInt(val, 10)).default('1800'), // 30 minutes
  SECURITY_BIOMETRIC_ENABLED: z.string().transform(val => val === 'true').default('true'),
  
  // Feature Flags
  FEATURE_NFT_MARKETPLACE: z.string().transform(val => val === 'true').default('true'),
  FEATURE_GUILD_SYSTEM: z.string().transform(val => val === 'true').default('true'),
  FEATURE_DEFI_INTEGRATION: z.string().transform(val => val === 'true').default('false'),
  FEATURE_BRIDGE_ENABLED: z.string().transform(val => val === 'true').default('false'),
  FEATURE_GOVERNANCE: z.string().transform(val => val === 'true').default('false'),
});

// Validate environment variables
const env = envSchema.parse(process.env);

// Configuration object
export const config = {
  // Server
  server: {
    nodeEnv: env.NODE_ENV,
    port: env.PORT,
    apiVersion: env.API_VERSION,
    baseUrl: `http://localhost:${env.PORT}`,
  },

  // Database
  database: {
    url: env.DATABASE_URL,
    host: env.DATABASE_HOST,
    port: env.DATABASE_PORT,
    name: env.DATABASE_NAME,
    user: env.DATABASE_USER,
    password: env.DATABASE_PASSWORD,
    ssl: env.DATABASE_SSL,
    pool: {
      min: env.DATABASE_POOL_MIN,
      max: env.DATABASE_POOL_MAX,
    },
  },

  // Redis
  redis: {
    url: env.REDIS_URL || `redis://${env.REDIS_HOST}:${env.REDIS_PORT}`,
    host: env.REDIS_HOST,
    port: env.REDIS_PORT,
    password: env.REDIS_PASSWORD,
    db: env.REDIS_DB,
    ttl: env.REDIS_TTL,
  },

  // Blockchain
  blockchain: {
    solana: {
      rpcUrl: env.SOLANA_RPC_URL,
      wssUrl: env.SOLANA_WSS_URL,
      network: env.SOLANA_NETWORK,
      commitment: env.SOLANA_COMMITMENT,
      programs: {
        core: env.FINOVA_CORE_PROGRAM_ID,
        token: env.FINOVA_TOKEN_PROGRAM_ID,
        nft: env.FINOVA_NFT_PROGRAM_ID,
        defi: env.FINOVA_DEFI_PROGRAM_ID,
        bridge: env.FINOVA_BRIDGE_PROGRAM_ID,
        oracle: env.FINOVA_ORACLE_PROGRAM_ID,
      },
    },
  },

  // Authentication
  auth: {
    jwt: {
      secret: env.JWT_SECRET,
      expiresIn: env.JWT_EXPIRES_IN,
      refreshExpiresIn: env.JWT_REFRESH_EXPIRES_IN,
      issuer: env.JWT_ISSUER,
      audience: env.JWT_AUDIENCE,
    },
    encryption: {
      key: env.ENCRYPTION_KEY,
      algorithm: env.ENCRYPTION_ALGORITHM,
      saltRounds: env.HASH_SALT_ROUNDS,
    },
  },

  // Rate Limiting
  rateLimit: {
    windowMs: env.RATE_LIMIT_WINDOW_MS,
    maxRequests: env.RATE_LIMIT_MAX_REQUESTS,
    skipSuccessfulRequests: env.RATE_LIMIT_SKIP_SUCCESSFUL_REQUESTS,
  },

  // CORS
  cors: {
    origin: env.CORS_ORIGIN?.split(',') || true,
    methods: env.CORS_METHODS.split(','),
    allowedHeaders: env.CORS_ALLOWED_HEADERS.split(','),
    credentials: true,
  },

  // KYC
  kyc: {
    provider: env.KYC_PROVIDER,
    apiUrl: env.KYC_API_URL,
    apiKey: env.KYC_API_KEY,
    secretKey: env.KYC_SECRET_KEY,
    webhookSecret: env.KYC_WEBHOOK_SECRET,
  },

  // Social Media
  socialMedia: {
    instagram: {
      clientId: env.INSTAGRAM_CLIENT_ID,
      clientSecret: env.INSTAGRAM_CLIENT_SECRET,
    },
    tiktok: {
      clientId: env.TIKTOK_CLIENT_ID,
      clientSecret: env.TIKTOK_CLIENT_SECRET,
    },
    youtube: {
      apiKey: env.YOUTUBE_API_KEY,
    },
    facebook: {
      appId: env.FACEBOOK_APP_ID,
      appSecret: env.FACEBOOK_APP_SECRET,
    },
    twitter: {
      apiKey: env.TWITTER_API_KEY,
      apiSecret: env.TWITTER_API_SECRET,
    },
  },

  // Payment Gateways
  payments: {
    ovo: {
      merchantId: env.OVO_MERCHANT_ID,
      apiKey: env.OVO_API_KEY,
    },
    gopay: {
      merchantId: env.GOPAY_MERCHANT_ID,
      apiKey: env.GOPAY_API_KEY,
    },
    dana: {
      merchantId: env.DANA_MERCHANT_ID,
      apiKey: env.DANA_API_KEY,
    },
    shopeepay: {
      merchantId: env.SHOPEEPAY_MERCHANT_ID,
      apiKey: env.SHOPEEPAY_API_KEY,
    },
  },

  // AI Services
  ai: {
    baseUrl: env.AI_CONTENT_ANALYZER_URL,
    services: {
      contentAnalyzer: env.AI_CONTENT_ANALYZER_URL,
      botDetection: env.AI_BOT_DETECTION_URL,
      recommendation: env.AI_RECOMMENDATION_URL,
      analytics: env.AI_ANALYTICS_URL,
    },
    apiKey: env.AI_SERVICE_API_KEY,
  },

  // Email
  email: {
    smtp: {
      host: env.SMTP_HOST,
      port: env.SMTP_PORT,
      user: env.SMTP_USER,
      pass: env.SMTP_PASS,
    },
    from: env.EMAIL_FROM,
  },

  // AWS
  aws: {
    region: env.AWS_REGION,
    accessKeyId: env.AWS_ACCESS_KEY_ID,
    secretAccessKey: env.AWS_SECRET_ACCESS_KEY,
    s3: {
      bucket: env.AWS_S3_BUCKET,
      cloudfrontDomain: env.AWS_CLOUDFRONT_DOMAIN,
    },
  },

  // Monitoring
  monitoring: {
    logLevel: env.LOG_LEVEL,
    sentry: {
      dsn: env.SENTRY_DSN,
    },
    newRelic: {
      licenseKey: env.NEW_RELIC_LICENSE_KEY,
    },
    prometheus: {
      enabled: env.PROMETHEUS_ENABLED,
    },
  },

  // WebSocket
  websocket: {
    port: env.WS_PORT,
    pingInterval: env.WS_PING_INTERVAL,
    pongTimeout: env.WS_PONG_TIMEOUT,
  },

  // Mining System
  mining: {
    baseRate: env.MINING_BASE_RATE,
    phases: {
      finizen: {
        threshold: env.MINING_PHASE_THRESHOLD_1,
        rate: 0.1,
        bonus: 2.0,
      },
      growth: {
        threshold: env.MINING_PHASE_THRESHOLD_2,
        rate: 0.05,
        bonus: 1.5,
      },
      maturity: {
        threshold: env.MINING_PHASE_THRESHOLD_3,
        rate: 0.025,
        bonus: 1.2,
      },
      stability: {
        rate: 0.01,
        bonus: 1.0,
      },
    },
    regressionFactor: env.MINING_REGRESSION_FACTOR,
  },

  // XP System
  xp: {
    levelProgressionFactor: env.XP_LEVEL_PROGRESSION_FACTOR,
    dailyStreakMax: env.XP_DAILY_STREAK_MAX,
    qualityScore: {
      min: env.XP_QUALITY_SCORE_MIN,
      max: env.XP_QUALITY_SCORE_MAX,
    },
    activities: {
      originalPost: 50,
      photoPost: 75,
      videoPost: 150,
      storyPost: 25,
      meaningfulComment: 25,
      like: 5,
      share: 15,
      follow: 20,
      dailyLogin: 10,
      dailyQuest: 100,
      milestone: 500,
      viralContent: 1000,
    },
    platformMultipliers: {
      tiktok: 1.3,
      instagram: 1.2,
      youtube: 1.4,
      facebook: 1.1,
      twitter: 1.2,
    },
  },

  // RP System
  rp: {
    networkRegressionFactor: env.RP_NETWORK_REGRESSION_FACTOR,
    tiers: {
      explorer: { min: 0, max: 999, bonus: 0, referralBonus: 10 },
      connector: { min: 1000, max: 4999, bonus: 20, referralBonus: 15 },
      influencer: { min: 5000, max: 14999, bonus: 50, referralBonus: 20 },
      leader: { min: 15000, max: 49999, bonus: 100, referralBonus: 25 },
      ambassador: { min: 50000, max: Infinity, bonus: 200, referralBonus: 30 },
    },
    maxReferralDepth: env.RP_MAX_REFERRAL_DEPTH,
    networkBonuses: {
      10: { rp: 500, multiplier: 0.5 },
      25: { rp: 1500, multiplier: 1.0 },
      50: { rp: 5000, multiplier: 1.5 },
      100: { rp: 15000, multiplier: 2.0 },
    },
  },

  // Security
  security: {
    maxLoginAttempts: env.SECURITY_MAX_LOGIN_ATTEMPTS,
    lockoutDuration: env.SECURITY_LOCKOUT_DURATION,
    sessionTimeout: env.SECURITY_SESSION_TIMEOUT,
    biometricEnabled: env.SECURITY_BIOMETRIC_ENABLED,
  },

  // Feature Flags
  features: {
    nftMarketplace: env.FEATURE_NFT_MARKETPLACE,
    guildSystem: env.FEATURE_GUILD_SYSTEM,
    defiIntegration: env.FEATURE_DEFI_INTEGRATION,
    bridgeEnabled: env.FEATURE_BRIDGE_ENABLED,
    governance: env.FEATURE_GOVERNANCE,
  },

  // Constants
  constants: {
    maxDailyMining: 24, // hours
    kycRequiredForMining: true,
    minWithdrawalAmount: 10, // $FIN
    maxTransactionAmount: 100000, // $FIN
    supportedLanguages: ['en', 'id', 'ms', 'th', 'vi', 'zh'],
    supportedCurrencies: ['USD', 'IDR', 'SGD', 'THB', 'VND', 'CNY'],
  },
} as const;

// Type definitions
export type Config = typeof config;
export type Environment = typeof env.NODE_ENV;
export type BlockchainNetwork = typeof env.SOLANA_NETWORK;

// Helper functions
export const isProduction = () => config.server.nodeEnv === 'production';
export const isDevelopment = () => config.server.nodeEnv === 'development';
export const isTestnet = () => config.server.nodeEnv === 'testnet';

export const getBlockchainConfig = () => config.blockchain.solana;
export const getMiningConfig = () => config.mining;
export const getXPConfig = () => config.xp;
export const getRPConfig = () => config.rp;

// Export default
export default config;
