import { Pool, PoolConfig, PoolClient } from 'pg';
import { createClient, RedisClientType } from 'redis';
import { Logger } from '../utils/logger';
import { config } from './index';

interface DatabaseConfig {
  postgres: PoolConfig;
  redis: {
    url: string;
    password?: string;
    retryDelayOnFailover: number;
    maxRetriesPerRequest: number;
  };
  mongodb?: {
    uri: string;
    dbName: string;
  };
}

interface DatabaseConnections {
  postgres: Pool;
  redis: RedisClientType;
  mongodb?: any; // MongoDB client if needed for analytics
}

class DatabaseManager {
  private static instance: DatabaseManager;
  private connections: DatabaseConnections | null = null;
  private logger = new Logger('DatabaseManager');
  private isShuttingDown = false;

  private constructor() {}

  static getInstance(): DatabaseManager {
    if (!DatabaseManager.instance) {
      DatabaseManager.instance = new DatabaseManager();
    }
    return DatabaseManager.instance;
  }

  async initialize(): Promise<DatabaseConnections> {
    if (this.connections) {
      return this.connections;
    }

    try {
      this.logger.info('Initializing database connections...');
      
      // Initialize PostgreSQL connection
      const postgresPool = await this.initializePostgreSQL();
      
      // Initialize Redis connection
      const redisClient = await this.initializeRedis();
      
      // Initialize MongoDB for analytics (optional)
      const mongoClient = config.database.mongodb?.uri ? 
        await this.initializeMongoDB() : undefined;

      this.connections = {
        postgres: postgresPool,
        redis: redisClient,
        mongodb: mongoClient
      };

      // Setup graceful shutdown
      this.setupGracefulShutdown();
      
      this.logger.info('All database connections initialized successfully');
      return this.connections;

    } catch (error) {
      this.logger.error('Failed to initialize database connections:', error);
      throw error;
    }
  }

  private async initializePostgreSQL(): Promise<Pool> {
    const dbConfig = this.getPostgreSQLConfig();
    
    const pool = new Pool({
      ...dbConfig,
      // Connection pool settings for enterprise-grade performance
      max: 20, // Maximum pool size
      min: 5,  // Minimum pool connections
      idleTimeoutMillis: 30000, // 30 seconds
      connectionTimeoutMillis: 10000, // 10 seconds
      
      // SSL configuration for production
      ssl: config.env === 'production' ? {
        rejectUnauthorized: false,
        ca: config.database.postgres.ssl?.ca,
        cert: config.database.postgres.ssl?.cert,
        key: config.database.postgres.ssl?.key
      } : false,
      
      // Query timeout
      query_timeout: 60000, // 60 seconds
      statement_timeout: 60000 // 60 seconds
    });

    // Test connection
    const client = await pool.connect();
    try {
      await client.query('SELECT NOW()');
      this.logger.info('PostgreSQL connection established');
    } finally {
      client.release();
    }

    // Setup connection error handling
    pool.on('error', (err) => {
      this.logger.error('PostgreSQL pool error:', err);
    });

    pool.on('connect', () => {
      this.logger.debug('New PostgreSQL connection established');
    });

    pool.on('remove', () => {
      this.logger.debug('PostgreSQL connection removed from pool');
    });

    return pool;
  }

  private async initializeRedis(): Promise<RedisClientType> {
    const redisConfig = config.database.redis;
    
    const client = createClient({
      url: redisConfig.url,
      password: redisConfig.password,
      socket: {
        reconnectStrategy: (retries) => {
          if (retries > 10) {
            this.logger.error('Redis connection failed after 10 retries');
            return new Error('Redis connection failed');
          }
          return Math.min(retries * 50, 500);
        },
        connectTimeout: 10000, // 10 seconds
        commandTimeout: 5000   // 5 seconds
      },
      // Redis cluster support if needed
      ...(redisConfig.cluster && {
        cluster: {
          enableReadyCheck: true,
          redisOptions: {
            password: redisConfig.password
          }
        }
      })
    });

    // Setup Redis event handlers
    client.on('error', (err) => {
      this.logger.error('Redis connection error:', err);
    });

    client.on('connect', () => {
      this.logger.info('Redis connection established');
    });

    client.on('reconnecting', () => {
      this.logger.info('Redis reconnecting...');
    });

    client.on('ready', () => {
      this.logger.info('Redis client ready');
    });

    // Connect to Redis
    await client.connect();
    
    // Test Redis connection
    await client.ping();
    this.logger.info('Redis connection verified');

    return client;
  }

  private async initializeMongoDB(): Promise<any> {
    // MongoDB connection for analytics data (optional)
    try {
      const { MongoClient } = require('mongodb');
      const mongoConfig = config.database.mongodb!;
      
      const client = new MongoClient(mongoConfig.uri, {
        maxPoolSize: 10,
        serverSelectionTimeoutMS: 5000,
        socketTimeoutMS: 45000,
      });

      await client.connect();
      await client.db(mongoConfig.dbName).admin().ping();
      
      this.logger.info('MongoDB connection established');
      return client;
    } catch (error) {
      this.logger.warn('MongoDB connection failed (optional):', error);
      return null;
    }
  }

  private getPostgreSQLConfig(): PoolConfig {
    const dbConfig = config.database.postgres;
    
    return {
      host: dbConfig.host,
      port: dbConfig.port,
      database: dbConfig.database,
      user: dbConfig.user,
      password: dbConfig.password,
      
      // Enhanced configuration for Finova Network requirements
      application_name: 'finova-api',
      
      // Connection pool configuration
      max: parseInt(process.env.DB_POOL_MAX || '20'),
      min: parseInt(process.env.DB_POOL_MIN || '5'),
      
      // Timeout configurations
      connectionTimeoutMillis: 10000,
      idleTimeoutMillis: 30000,
      
      // Additional PostgreSQL-specific settings
      options: '-c timezone=UTC',
      
      // Performance optimizations
      keepAlive: true,
      keepAliveInitialDelayMillis: 10000,
    };
  }

  // Transaction management for mining operations
  async executeTransaction<T>(
    operation: (client: PoolClient) => Promise<T>
  ): Promise<T> {
    if (!this.connections) {
      throw new Error('Database not initialized');
    }

    const client = await this.connections.postgres.connect();
    
    try {
      await client.query('BEGIN');
      const result = await operation(client);
      await client.query('COMMIT');
      return result;
    } catch (error) {
      await client.query('ROLLBACK');
      this.logger.error('Transaction failed:', error);
      throw error;
    } finally {
      client.release();
    }
  }

  // Batch operations for XP/RP calculations
  async executeBatchQuery(
    queries: Array<{ text: string; values?: any[] }>
  ): Promise<any[]> {
    if (!this.connections) {
      throw new Error('Database not initialized');
    }

    const results: any[] = [];
    const client = await this.connections.postgres.connect();
    
    try {
      await client.query('BEGIN');
      
      for (const query of queries) {
        const result = await client.query(query.text, query.values);
        results.push(result);
      }
      
      await client.query('COMMIT');
      return results;
    } catch (error) {
      await client.query('ROLLBACK');
      this.logger.error('Batch query failed:', error);
      throw error;
    } finally {
      client.release();
    }
  }

  // Cache management for frequent queries
  async getCachedData<T>(
    key: string,
    fetchFunction: () => Promise<T>,
    ttl: number = 300 // 5 minutes default
  ): Promise<T> {
    if (!this.connections) {
      throw new Error('Database not initialized');
    }

    try {
      // Try to get from cache first
      const cachedData = await this.connections.redis.get(key);
      if (cachedData) {
        return JSON.parse(cachedData);
      }

      // If not in cache, fetch data
      const data = await fetchFunction();
      
      // Store in cache
      await this.connections.redis.setEx(key, ttl, JSON.stringify(data));
      
      return data;
    } catch (error) {
      this.logger.error('Cache operation failed:', error);
      // Fallback to direct fetch if cache fails
      return await fetchFunction();
    }
  }

  // Health check for all database connections
  async healthCheck(): Promise<{
    postgres: boolean;
    redis: boolean;
    mongodb?: boolean;
  }> {
    const health = {
      postgres: false,
      redis: false,
      mongodb: undefined as boolean | undefined
    };

    try {
      if (this.connections?.postgres) {
        const client = await this.connections.postgres.connect();
        await client.query('SELECT 1');
        client.release();
        health.postgres = true;
      }
    } catch (error) {
      this.logger.error('PostgreSQL health check failed:', error);
    }

    try {
      if (this.connections?.redis) {
        await this.connections.redis.ping();
        health.redis = true;
      }
    } catch (error) {
      this.logger.error('Redis health check failed:', error);
    }

    try {
      if (this.connections?.mongodb) {
        await this.connections.mongodb.db().admin().ping();
        health.mongodb = true;
      }
    } catch (error) {
      this.logger.error('MongoDB health check failed:', error);
      health.mongodb = false;
    }

    return health;
  }

  private setupGracefulShutdown(): void {
    const shutdown = async (signal: string) => {
      if (this.isShuttingDown) return;
      this.isShuttingDown = true;

      this.logger.info(`Received ${signal}, shutting down gracefully...`);
      
      try {
        // Close all connections
        if (this.connections?.postgres) {
          await this.connections.postgres.end();
          this.logger.info('PostgreSQL connections closed');
        }

        if (this.connections?.redis) {
          await this.connections.redis.quit();
          this.logger.info('Redis connection closed');
        }

        if (this.connections?.mongodb) {
          await this.connections.mongodb.close();
          this.logger.info('MongoDB connection closed');
        }

        this.logger.info('Database shutdown completed');
        process.exit(0);
      } catch (error) {
        this.logger.error('Error during shutdown:', error);
        process.exit(1);
      }
    };

    process.on('SIGTERM', () => shutdown('SIGTERM'));
    process.on('SIGINT', () => shutdown('SIGINT'));
    process.on('SIGUSR2', () => shutdown('SIGUSR2')); // For nodemon
  }

  // Specific methods for Finova Network operations
  
  // Mining-related database operations
  async updateUserMiningState(
    userId: string,
    miningRate: number,
    lastClaimTime: Date
  ): Promise<void> {
    if (!this.connections) {
      throw new Error('Database not initialized');
    }

    await this.executeTransaction(async (client) => {
      await client.query(
        'UPDATE users SET mining_rate = $1, last_claim_time = $2, updated_at = NOW() WHERE id = $3',
        [miningRate, lastClaimTime, userId]
      );

      // Update mining statistics
      await client.query(
        'INSERT INTO mining_stats (user_id, rate, timestamp) VALUES ($1, $2, $3)',
        [userId, miningRate, new Date()]
      );
    });
  }

  // XP system database operations
  async addUserXP(
    userId: string,
    xpGained: number,
    activityType: string,
    qualityScore: number
  ): Promise<{ newLevel: number; totalXP: number }> {
    if (!this.connections) {
      throw new Error('Database not initialized');
    }

    return await this.executeTransaction(async (client) => {
      // Update user XP
      const userResult = await client.query(
        'UPDATE users SET total_xp = total_xp + $1, updated_at = NOW() WHERE id = $2 RETURNING total_xp',
        [xpGained, userId]
      );

      const totalXP = userResult.rows[0].total_xp;
      const newLevel = Math.floor(Math.sqrt(totalXP / 100)) + 1;

      // Update user level if changed
      await client.query(
        'UPDATE users SET level = $1 WHERE id = $2 AND level < $1',
        [newLevel, userId]
      );

      // Log XP activity
      await client.query(
        'INSERT INTO xp_activities (user_id, xp_gained, activity_type, quality_score, timestamp) VALUES ($1, $2, $3, $4, $5)',
        [userId, xpGained, activityType, qualityScore, new Date()]
      );

      return { newLevel, totalXP };
    });
  }

  // RP (Referral Points) system operations
  async updateReferralNetwork(
    userId: string,
    referrerId: string,
    rpGained: number
  ): Promise<void> {
    if (!this.connections) {
      throw new Error('Database not initialized');
    }

    await this.executeTransaction(async (client) => {
      // Update referrer's RP
      await client.query(
        'UPDATE users SET referral_points = referral_points + $1 WHERE id = $2',
        [rpGained, referrerId]
      );

      // Update referral relationship
      await client.query(
        'UPDATE referrals SET total_rp_earned = total_rp_earned + $1, updated_at = NOW() WHERE referrer_id = $2 AND referred_id = $3',
        [rpGained, referrerId, userId]
      );

      // Log RP activity
      await client.query(
        'INSERT INTO rp_activities (referrer_id, referred_id, rp_gained, timestamp) VALUES ($1, $2, $3, $4)',
        [referrerId, userId, rpGained, new Date()]
      );
    });
  }

  getConnections(): DatabaseConnections | null {
    return this.connections;
  }
}

// Export singleton instance
export const database = DatabaseManager.getInstance();

// Export the database configuration for use in other modules
export const databaseConfig: DatabaseConfig = {
  postgres: {
    host: config.database.postgres.host,
    port: config.database.postgres.port,
    database: config.database.postgres.database,
    user: config.database.postgres.user,
    password: config.database.postgres.password,
  },
  redis: {
    url: config.database.redis.url,
    password: config.database.redis.password,
    retryDelayOnFailover: 100,
    maxRetriesPerRequest: 3,
  },
  ...(config.database.mongodb && {
    mongodb: {
      uri: config.database.mongodb.uri,
      dbName: config.database.mongodb.dbName,
    }
  })
};

// Connection initialization helper
export async function initializeDatabase(): Promise<DatabaseConnections> {
  return await database.initialize();
}

// Health check endpoint helper
export async function checkDatabaseHealth() {
  return await database.healthCheck();
}

// Export types for use in other modules
export type { DatabaseConnections, DatabaseConfig };

// Database utility functions for Finova-specific operations
export const DatabaseUtils = {
  // Calculate mining regression factor
  calculateMiningRegression: async (userId: string): Promise<number> => {
    const connections = database.getConnections();
    if (!connections) throw new Error('Database not initialized');

    const result = await connections.postgres.query(
      'SELECT total_fin_holdings FROM users WHERE id = $1',
      [userId]
    );

    const holdings = result.rows[0]?.total_fin_holdings || 0;
    return Math.exp(-0.001 * holdings);
  },

  // Get user's network quality score for RP calculations
  getUserNetworkQuality: async (userId: string): Promise<number> => {
    const connections = database.getConnections();
    if (!connections) throw new Error('Database not initialized');

    const result = await connections.postgres.query(`
      SELECT 
        COUNT(*) as total_referrals,
        COUNT(CASE WHEN last_activity_at > NOW() - INTERVAL '30 days' THEN 1 END) as active_referrals
      FROM referrals r
      JOIN users u ON r.referred_id = u.id
      WHERE r.referrer_id = $1
    `, [userId]);

    const { total_referrals, active_referrals } = result.rows[0];
    return total_referrals > 0 ? active_referrals / total_referrals : 0;
  },

  // Batch update mining rates for all active users
  batchUpdateMiningRates: async (): Promise<void> => {
    const connections = database.getConnections();
    if (!connections) throw new Error('Database not initialized');

    await connections.postgres.query(`
      UPDATE users 
      SET mining_rate = calculate_mining_rate(id),
          updated_at = NOW()
      WHERE is_mining = true 
      AND last_activity_at > NOW() - INTERVAL '24 hours'
    `);
  }
};

export default database;
