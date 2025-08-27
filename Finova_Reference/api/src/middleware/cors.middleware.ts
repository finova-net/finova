import { Request, Response, NextFunction } from 'express';
import cors from 'cors';

/**
 * Finova Network CORS Middleware
 * Enterprise-grade CORS configuration for multi-platform integration
 * Supports mobile apps, web clients, and social platform webhooks
 */

interface FinovaCorsOptions {
  origin: string | string[] | boolean | RegExp | Function;
  methods: string[];
  allowedHeaders: string[];
  exposedHeaders: string[];
  credentials: boolean;
  maxAge: number;
  preflightContinue: boolean;
  optionsSuccessStatus: number;
}

class FinovaCorsManager {
  private static instance: FinovaCorsManager;
  private corsOptions: FinovaCorsOptions;
  
  private constructor() {
    this.corsOptions = this.initializeCorsConfig();
  }

  public static getInstance(): FinovaCorsManager {
    if (!FinovaCorsManager.instance) {
      FinovaCorsManager.instance = new FinovaCorsManager();
    }
    return FinovaCorsManager.instance;
  }

  private initializeCorsConfig(): FinovaCorsOptions {
    const allowedOrigins = this.getAllowedOrigins();
    
    return {
      origin: (origin, callback) => {
        // Allow requests with no origin (mobile apps, Postman, etc.)
        if (!origin) return callback(null, true);
        
        // Check against whitelist
        if (this.isOriginAllowed(origin, allowedOrigins)) {
          return callback(null, true);
        }
        
        // Reject unauthorized origins
        const msg = `The CORS policy for this site does not allow access from the specified Origin: ${origin}`;
        return callback(new Error(msg), false);
      },
      methods: [
        'GET', 'POST', 'PUT', 'PATCH', 'DELETE', 'OPTIONS', 'HEAD'
      ],
      allowedHeaders: [
        // Standard headers
        'Origin',
        'X-Requested-With',
        'Content-Type',
        'Accept',
        'Authorization',
        
        // Finova-specific headers
        'X-Finova-API-Key',
        'X-Finova-User-ID',
        'X-Finova-Session-Token',
        'X-Finova-Device-ID',
        'X-Finova-Platform',
        'X-Finova-Version',
        'X-Finova-Timestamp',
        'X-Finova-Nonce',
        'X-Finova-Signature',
        
        // Social platform headers
        'X-Instagram-Webhook-Signature',
        'X-TikTok-Signature',
        'X-YouTube-Signature',
        'X-Facebook-Signature',
        'X-Twitter-Webhook-Signature',
        
        // Mobile app headers
        'X-Device-Platform',
        'X-App-Version',
        'X-Device-UUID',
        'X-Push-Token',
        
        // Security headers
        'X-CSRF-Token',
        'X-Request-ID',
        'X-Correlation-ID',
        'X-Rate-Limit-Remaining',
        
        // WebSocket upgrade headers
        'Upgrade',
        'Connection',
        'Sec-WebSocket-Key',
        'Sec-WebSocket-Version',
        'Sec-WebSocket-Protocol'
      ],
      exposedHeaders: [
        // Finova response headers
        'X-Finova-Mining-Rate',
        'X-Finova-XP-Balance',
        'X-Finova-RP-Balance',
        'X-Finova-Level',
        'X-Finova-Streak',
        
        // Rate limiting
        'X-RateLimit-Limit',
        'X-RateLimit-Remaining',
        'X-RateLimit-Reset',
        'X-RateLimit-RetryAfter',
        
        // Pagination
        'X-Total-Count',
        'X-Page-Count',
        'X-Current-Page',
        'X-Per-Page',
        
        // Cache control
        'Cache-Control',
        'ETag',
        'Last-Modified',
        
        // Request tracking
        'X-Request-ID',
        'X-Response-Time',
        'X-API-Version'
      ],
      credentials: true,
      maxAge: 86400, // 24 hours
      preflightContinue: false,
      optionsSuccessStatus: 200
    };
  }

  private getAllowedOrigins(): string[] {
    const baseOrigins = [
      // Production domains
      'https://finova.network',
      'https://app.finova.network',
      'https://api.finova.network',
      'https://admin.finova.network',
      'https://dashboard.finova.network',
      
      // Mobile app schemes
      'finova://',
      'finovaapp://',
      
      // CDN and static assets
      'https://cdn.finova.network',
      'https://assets.finova.network',
      'https://img.finova.network'
    ];

    // Environment-specific origins
    const envOrigins = this.getEnvironmentOrigins();
    
    // Development origins (only in non-production)
    const devOrigins = process.env.NODE_ENV === 'production' ? [] : [
      'http://localhost:3000',
      'http://localhost:3001',
      'http://localhost:3002',
      'http://localhost:5173', // Vite
      'http://localhost:8080', // Webpack dev server
      'http://127.0.0.1:3000',
      'http://192.168.1.100:3000', // Local network testing
      /^http:\/\/localhost:\d{4}$/, // Dynamic ports
      /^http:\/\/127\.0\.0\.1:\d{4}$/ // Dynamic ports
    ];

    return [...baseOrigins, ...envOrigins, ...devOrigins];
  }

  private getEnvironmentOrigins(): string[] {
    const env = process.env.NODE_ENV || 'development';
    
    switch (env) {
      case 'development':
        return [
          'https://dev.finova.network',
          'https://dev-api.finova.network'
        ];
      case 'staging':
        return [
          'https://staging.finova.network',
          'https://staging-api.finova.network'
        ];
      case 'testnet':
        return [
          'https://testnet.finova.network',
          'https://testnet-api.finova.network'
        ];
      case 'production':
      default:
        return [];
    }
  }

  private isOriginAllowed(origin: string, allowedOrigins: (string | RegExp)[]): boolean {
    return allowedOrigins.some(allowed => {
      if (typeof allowed === 'string') {
        return origin === allowed;
      }
      if (allowed instanceof RegExp) {
        return allowed.test(origin);
      }
      return false;
    });
  }

  public getCorsMiddleware() {
    return cors(this.corsOptions);
  }

  public getSecurityCorsMiddleware() {
    // Enhanced CORS for sensitive endpoints
    const securityOptions = {
      ...this.corsOptions,
      origin: (origin: string | undefined, callback: Function) => {
        // More restrictive for sensitive endpoints
        if (!origin && process.env.NODE_ENV === 'production') {
          return callback(new Error('Origin required for security endpoints'), false);
        }
        
        const allowedSecurityOrigins = [
          'https://app.finova.network',
          'https://admin.finova.network'
        ];
        
        if (origin && allowedSecurityOrigins.includes(origin)) {
          return callback(null, true);
        }
        
        return callback(new Error('Unauthorized origin for security endpoint'), false);
      },
      credentials: true,
      maxAge: 3600 // 1 hour for security endpoints
    };
    
    return cors(securityOptions);
  }

  public getWebhookCorsMiddleware() {
    // Specific CORS for social platform webhooks
    const webhookOptions = {
      ...this.corsOptions,
      origin: (origin: string | undefined, callback: Function) => {
        const webhookOrigins = [
          // Instagram/Facebook
          'https://graph.facebook.com',
          'https://www.facebook.com',
          
          // TikTok
          'https://www.tiktok.com',
          'https://tiktok.com',
          
          // YouTube
          'https://www.youtube.com',
          'https://youtube.com',
          'https://pubsubhubbub.appspot.com',
          
          // Twitter/X
          'https://api.twitter.com',
          'https://twitter.com',
          
          // Internal services
          'https://api.finova.network'
        ];
        
        if (!origin || webhookOrigins.includes(origin)) {
          return callback(null, true);
        }
        
        return callback(null, false);
      },
      methods: ['POST', 'GET', 'HEAD'],
      credentials: false // Webhooks typically don't need credentials
    };
    
    return cors(webhookOptions);
  }
}

// Middleware factory functions
export const createCorsMiddleware = () => {
  const corsManager = FinovaCorsManager.getInstance();
  return corsManager.getCorsMiddleware();
};

export const createSecurityCorsMiddleware = () => {
  const corsManager = FinovaCorsManager.getInstance();
  return corsManager.getSecurityCorsMiddleware();
};

export const createWebhookCorsMiddleware = () => {
  const corsManager = FinovaCorsManager.getInstance();
  return corsManager.getWebhookCorsMiddleware();
};

// Custom CORS middleware for different routes
export const dynamicCorsMiddleware = (req: Request, res: Response, next: NextFunction) => {
  const corsManager = FinovaCorsManager.getInstance();
  
  // Route-specific CORS logic
  if (req.path.startsWith('/api/webhook/')) {
    return corsManager.getWebhookCorsMiddleware()(req, res, next);
  }
  
  if (req.path.startsWith('/api/admin/') || req.path.startsWith('/api/auth/security/')) {
    return corsManager.getSecurityCorsMiddleware()(req, res, next);
  }
  
  // Default CORS for general API routes
  return corsManager.getCorsMiddleware()(req, res, next);
};

// CORS preflight handler for complex requests
export const corsPreflightHandler = (req: Request, res: Response, next: NextFunction) => {
  if (req.method === 'OPTIONS') {
    // Handle preflight request
    res.header('Access-Control-Allow-Origin', req.headers.origin || '*');
    res.header('Access-Control-Allow-Credentials', 'true');
    res.header('Access-Control-Allow-Methods', 'GET,HEAD,PUT,PATCH,POST,DELETE,OPTIONS');
    res.header('Access-Control-Allow-Headers', req.headers['access-control-request-headers']);
    res.header('Access-Control-Max-Age', '86400');
    
    // Add Finova-specific preflight headers
    res.header('X-Finova-API-Version', process.env.API_VERSION || '1.0.0');
    res.header('X-Finova-Service', 'cors-preflight');
    
    return res.sendStatus(200);
  }
  next();
};

// CORS error handler
export const corsErrorHandler = (err: Error, req: Request, res: Response, next: NextFunction) => {
  if (err.message.includes('CORS')) {
    return res.status(403).json({
      error: 'CORS_POLICY_VIOLATION',
      message: 'Cross-Origin Request Blocked',
      details: process.env.NODE_ENV === 'development' ? err.message : 'Origin not allowed',
      timestamp: new Date().toISOString(),
      requestId: req.headers['x-request-id'] || 'unknown'
    });
  }
  next(err);
};

// Main CORS middleware (default export)
const corsMiddleware = createCorsMiddleware();

export default corsMiddleware;
