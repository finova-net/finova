// finova-net/finova/client/typescript/src/accounts/user.ts

import { 
  PublicKey, 
  Connection, 
  SystemProgram,
  LAMPORTS_PER_SOL,
  Transaction,
  TransactionInstruction,
  AccountInfo
} from '@solana/web3.js';
import { 
  TOKEN_PROGRAM_ID, 
  getAssociatedTokenAddress,
  createAssociatedTokenAccountInstruction,
  getAccount,
  TokenAccountNotFoundError,
  TokenInvalidAccountOwnerError
} from '@solana/spl-token';
import { BN } from '@coral-xyz/anchor';
import { FinovaClient } from '../client';
import { 
  UserAccount, 
  MiningStats, 
  XPStats, 
  ReferralStats,
  StakingInfo,
  UserLevel,
  RPTier,
  NetworkQuality,
  UserActivityData,
  SocialPlatform,
  ContentQuality,
  MiningPhase,
  SecurityStatus
} from '../types/user';
import { FINOVA_CONSTANTS } from '../constants';

/**
 * Enterprise-grade User Account Manager for Finova Network
 * Handles integrated XP, RP, and mining systems with exponential regression
 * 
 * Features:
 * - Triple reward system integration (XP, RP, $FIN)
 * - Exponential regression calculations
 * - AI-powered quality assessment
 * - Anti-bot protection
 * - Real-time network effects
 * - Cross-platform social integration
 */
export class UserAccountClient {
  private connection: Connection;
  private client: FinovaClient;
  private programId: PublicKey;
  private mintAddress: PublicKey;
  
  // Cache for performance optimization
  private userCache = new Map<string, UserAccount>();
  private lastCacheUpdate = new Map<string, number>();
  private readonly CACHE_TTL = 30000; // 30 seconds

  constructor(
    connection: Connection,
    client: FinovaClient,
    programId: PublicKey,
    mintAddress: PublicKey
  ) {
    this.connection = connection;
    this.client = client;
    this.programId = programId;
    this.mintAddress = mintAddress;
  }

  /**
   * Initialize a new user account with comprehensive setup
   * Implements Finova's integrated reward system from genesis
   */
  async initializeUser(
    userWallet: PublicKey,
    referralCode?: string,
    kycData?: {
      documentHash: string;
      biometricHash: string;
      verification: boolean;
    }
  ): Promise<{ 
    signature: string; 
    userAccount: PublicKey;
    tokenAccount: PublicKey;
    initialStats: UserAccount;
  }> {
    try {
      // Generate user account PDA
      const [userAccountPDA] = PublicKey.findProgramAddressSync(
        [Buffer.from('user'), userWallet.toBuffer()],
        this.programId
      );

      // Create associated token account for $FIN
      const tokenAccount = await getAssociatedTokenAddress(
        this.mintAddress,
        userWallet,
        false,
        TOKEN_PROGRAM_ID
      );

      // Build comprehensive initialization instruction
      const initInstruction = await this.buildInitializeUserInstruction(
        userWallet,
        userAccountPDA,
        tokenAccount,
        referralCode,
        kycData
      );

      // Create transaction with all required accounts
      const transaction = new Transaction().add(initInstruction);
      
      // Add token account creation if needed
      try {
        await getAccount(this.connection, tokenAccount);
      } catch (error) {
        if (error instanceof TokenAccountNotFoundError) {
          const createTokenAccountIx = createAssociatedTokenAccountInstruction(
            userWallet,
            tokenAccount,
            userWallet,
            this.mintAddress
          );
          transaction.instructions.unshift(createTokenAccountIx);
        }
      }

      // Send and confirm transaction
      const signature = await this.client.sendAndConfirmTransaction(transaction);

      // Fetch initial user stats
      const initialStats = await this.getUserAccount(userWallet);

      return {
        signature,
        userAccount: userAccountPDA,
        tokenAccount,
        initialStats
      };

    } catch (error) {
      console.error('User initialization failed:', error);
      throw new Error(`Failed to initialize user: ${error.message}`);
    }
  }

  /**
   * Get comprehensive user account data with integrated calculations
   * Implements all three reward systems (XP, RP, Mining)
   */
  async getUserAccount(userWallet: PublicKey, forceRefresh = false): Promise<UserAccount> {
    const cacheKey = userWallet.toString();
    
    // Check cache first (unless force refresh)
    if (!forceRefresh && this.isValidCache(cacheKey)) {
      return this.userCache.get(cacheKey)!;
    }

    try {
      const [userAccountPDA] = PublicKey.findProgramAddressSync(
        [Buffer.from('user'), userWallet.toBuffer()],
        this.programId
      );

      // Fetch raw account data
      const accountInfo = await this.connection.getAccountInfo(userAccountPDA);
      if (!accountInfo || !accountInfo.data) {
        throw new Error('User account not found');
      }

      // Deserialize user account data
      const rawUserData = this.deserializeUserAccount(accountInfo.data);
      
      // Calculate real-time integrated stats
      const integratedStats = await this.calculateIntegratedStats(rawUserData);
      
      // Get current network state for regression calculations
      const networkState = await this.getNetworkState();
      
      // Build comprehensive user account object
      const userAccount: UserAccount = {
        // Basic Info
        publicKey: userWallet,
        accountAddress: userAccountPDA,
        isInitialized: true,
        createdAt: rawUserData.createdAt,
        lastActive: rawUserData.lastActive,
        
        // Security & Verification
        securityStatus: await this.calculateSecurityStatus(rawUserData),
        kycVerified: rawUserData.kycVerified,
        humanProbability: rawUserData.humanProbability,
        suspiciousScore: rawUserData.suspiciousScore,
        
        // Mining System
        mining: await this.calculateMiningStats(rawUserData, networkState),
        
        // Experience Points System
        xp: await this.calculateXPStats(rawUserData),
        
        // Referral Points System  
        referral: await this.calculateReferralStats(rawUserData),
        
        // Token Holdings & Staking
        tokenBalance: await this.getTokenBalance(userWallet),
        stakingInfo: await this.getStakingInfo(userWallet),
        
        // Social Integration
        connectedPlatforms: rawUserData.connectedPlatforms,
        socialStats: rawUserData.socialStats,
        
        // Network Effects
        networkQuality: this.calculateNetworkQuality(rawUserData),
        totalNetworkSize: rawUserData.totalNetworkSize,
        
        // Performance Metrics
        lifetimeEarnings: rawUserData.lifetimeEarnings,
        currentStreak: rawUserData.currentStreak,
        maxStreak: rawUserData.maxStreak,
        
        // Computed Properties
        overallScore: integratedStats.overallScore,
        projectedDailyEarnings: integratedStats.projectedDailyEarnings,
        networkRank: integratedStats.networkRank,
        
        // Regression Factors
        regressionFactors: {
          mining: integratedStats.miningRegression,
          xp: integratedStats.xpRegression,
          referral: integratedStats.referralRegression
        }
      };

      // Update cache
      this.updateCache(cacheKey, userAccount);
      
      return userAccount;

    } catch (error) {
      console.error('Failed to fetch user account:', error);
      throw new Error(`User account fetch failed: ${error.message}`);
    }
  }

  /**
   * Calculate comprehensive mining statistics with Pi Network-inspired mechanics
   * Implements exponential regression and network effects
   */
  private async calculateMiningStats(
    rawUserData: any, 
    networkState: any
  ): Promise<MiningStats> {
    // Determine current mining phase
    const currentPhase = this.getCurrentMiningPhase(networkState.totalUsers);
    
    // Base mining rate calculation
    const baseRate = this.getBaseMiningRate(currentPhase);
    
    // Pioneer (Finizen) bonus calculation
    const pioneerBonus = Math.max(1.0, 2.0 - (networkState.totalUsers / 1_000_000));
    
    // Referral network bonus
    const referralBonus = 1 + (rawUserData.activeReferrals * 0.1);
    
    // Security bonus (KYC verification)
    const securityBonus = rawUserData.kycVerified ? 1.2 : 0.8;
    
    // Exponential regression factor (anti-whale mechanism)
    const regressionFactor = Math.exp(-0.001 * rawUserData.totalHoldings);
    
    // XP level mining multiplier
    const xpMultiplier = this.getXPMiningMultiplier(rawUserData.xpLevel);
    
    // RP tier mining bonus
    const rpMultiplier = this.getRPMiningMultiplier(rawUserData.rpTier);
    
    // Quality score from AI assessment
    const qualityMultiplier = rawUserData.averageQualityScore || 1.0;
    
    // Activity-based bonus
    const activityBonus = this.calculateActivityBonus(rawUserData);
    
    // Calculate final hourly mining rate
    const hourlyRate = baseRate * pioneerBonus * referralBonus * securityBonus * 
                      regressionFactor * xpMultiplier * rpMultiplier * 
                      qualityMultiplier * activityBonus;
    
    // Apply daily cap based on user level
    const dailyCap = this.getDailyMiningCap(rawUserData.xpLevel);
    const cappedHourlyRate = Math.min(hourlyRate, dailyCap / 24);
    
    // Calculate time until next mining reward
    const lastMiningTime = rawUserData.lastMiningClaimed || rawUserData.lastActive;
    const timeSinceLastMining = Date.now() - lastMiningTime;
    const timeUntilNextReward = Math.max(0, 3600000 - timeSinceLastMining); // 1 hour in ms
    
    // Calculate pending rewards
    const hoursElapsed = Math.min(24, timeSinceLastMining / 3600000); // Max 24 hours
    const pendingRewards = cappedHourlyRate * hoursElapsed;

    return {
      isActive: rawUserData.miningActive,
      currentPhase,
      hourlyRate: cappedHourlyRate,
      dailyRate: cappedHourlyRate * 24,
      dailyCap,
      pendingRewards,
      totalMined: rawUserData.totalMined,
      lifetimeMined: rawUserData.lifetimeMined,
      lastClaimedAt: rawUserData.lastMiningClaimed,
      timeUntilNextReward,
      
      // Bonus breakdown for transparency
      bonusBreakdown: {
        baseRate,
        pioneerBonus,
        referralBonus,
        securityBonus,
        xpMultiplier,
        rpMultiplier,
        qualityMultiplier,
        activityBonus,
        regressionFactor
      },
      
      // Performance metrics
      efficiency: this.calculateMiningEfficiency(rawUserData),
      rank: rawUserData.miningRank || 0,
      nextMilestone: this.getNextMiningMilestone(rawUserData.totalMined)
    };
  }

  /**
   * Calculate XP statistics with Hamster Kombat-inspired mechanics
   * Implements level progression and activity tracking
   */
  private async calculateXPStats(rawUserData: any): Promise<XPStats> {
    const currentXP = rawUserData.totalXP || 0;
    const currentLevel = this.calculateLevelFromXP(currentXP);
    const currentTier = this.getXPTier(currentLevel);
    
    // Calculate XP requirements for next level
    const nextLevelXP = this.getXPRequiredForLevel(currentLevel + 1);
    const currentLevelXP = this.getXPRequiredForLevel(currentLevel);
    const progressToNext = currentXP - currentLevelXP;
    const requiredForNext = nextLevelXP - currentXP;
    
    // Daily XP statistics
    const dailyXP = rawUserData.dailyXP || 0;
    const weeklyXP = rawUserData.weeklyXP || 0;
    const monthlyXP = rawUserData.monthlyXP || 0;
    
    // Activity breakdown
    const activityBreakdown = {
      posts: rawUserData.xpFromPosts || 0,
      comments: rawUserData.xpFromComments || 0,
      likes: rawUserData.xpFromLikes || 0,
      shares: rawUserData.xpFromShares || 0,
      viral: rawUserData.xpFromViral || 0,
      quests: rawUserData.xpFromQuests || 0,
      referrals: rawUserData.xpFromReferrals || 0
    };
    
    // Streak calculations
    const currentStreak = rawUserData.xpStreak || 0;
    const maxStreak = rawUserData.maxXpStreak || 0;
    const streakMultiplier = this.getStreakMultiplier(currentStreak);
    
    // Platform-specific XP
    const platformBreakdown: Record<SocialPlatform, number> = {
      [SocialPlatform.Instagram]: rawUserData.xpInstagram || 0,
      [SocialPlatform.TikTok]: rawUserData.xpTikTok || 0,
      [SocialPlatform.YouTube]: rawUserData.xpYoutube || 0,
      [SocialPlatform.Facebook]: rawUserData.xpFacebook || 0,
      [SocialPlatform.Twitter]: rawUserData.xpTwitter || 0
    };

    return {
      total: currentXP,
      level: currentLevel,
      tier: currentTier,
      
      // Progression
      currentLevelXP: progressToNext,
      nextLevelXP: requiredForNext,
      progressPercentage: (progressToNext / (nextLevelXP - currentLevelXP)) * 100,
      
      // Time-based stats
      daily: dailyXP,
      weekly: weeklyXP,
      monthly: monthlyXP,
      
      // Activity breakdown
      activityBreakdown,
      platformBreakdown,
      
      // Streaks and multipliers
      currentStreak,
      maxStreak,
      streakMultiplier,
      
      // Quality metrics
      averageQualityScore: rawUserData.averageQualityScore || 1.0,
      viralContentCount: rawUserData.viralContentCount || 0,
      
      // Benefits unlocked
      miningMultiplier: this.getXPMiningMultiplier(currentLevel),
      dailyFinCap: this.getDailyMiningCap(currentLevel),
      specialFeatures: this.getXPSpecialFeatures(currentLevel),
      
      // Performance
      rank: rawUserData.xpRank || 0,
      percentile: rawUserData.xpPercentile || 0
    };
  }

  /**
   * Calculate referral statistics with network effect analysis
   * Implements RP tier system and network quality assessment
   */
  private async calculateReferralStats(rawUserData: any): Promise<ReferralStats> {
    const totalRP = rawUserData.totalRP || 0;
    const rpTier = this.getRPTier(totalRP);
    
    // Direct referral calculations
    const directReferrals = rawUserData.directReferrals || 0;
    const activeReferrals = rawUserData.activeReferrals || 0; // Active in last 30 days
    const retentionRate = directReferrals > 0 ? activeReferrals / directReferrals : 0;
    
    // Network calculations (multi-level)
    const level2Network = rawUserData.level2Network || 0;
    const level3Network = rawUserData.level3Network || 0;
    const totalNetworkSize = directReferrals + level2Network + level3Network;
    
    // Network quality assessment
    const networkQuality = this.calculateNetworkQuality(rawUserData);
    
    // RP earning breakdown
    const rpBreakdown = {
      registration: rawUserData.rpFromRegistration || 0,
      kyc: rawUserData.rpFromKyc || 0,
      activity: rawUserData.rpFromActivity || 0,
      networking: rawUserData.rpFromNetworking || 0,
      achievements: rawUserData.rpFromAchievements || 0
    };
    
    // Daily/weekly RP from network activity
    const dailyNetworkRP = this.calculateDailyNetworkRP(rawUserData);
    const weeklyNetworkRP = rawUserData.weeklyNetworkRP || 0;
    
    // Regression factor for large networks
    const regressionFactor = Math.exp(-0.0001 * totalNetworkSize * networkQuality.score);
    
    // Benefits calculation
    const tierBenefits = this.getRPTierBenefits(rpTier);
    const miningBonus = tierBenefits.miningBonus;
    const referralBonus = tierBenefits.referralBonus;

    return {
      total: totalRP,
      tier: rpTier,
      
      // Network size
      directReferrals,
      activeReferrals,
      level2Network,
      level3Network,
      totalNetworkSize,
      
      // Network quality
      networkQuality,
      retentionRate,
      averageReferralLevel: rawUserData.averageReferralLevel || 1,
      
      // Earnings
      dailyRP: dailyNetworkRP,
      weeklyRP: weeklyNetworkRP,
      monthlyRP: rawUserData.monthlyNetworkRP || 0,
      
      // Breakdown
      rpBreakdown,
      
      // Performance
      regressionFactor,
      effectiveRP: totalRP * regressionFactor,
      
      // Benefits
      miningBonus,
      referralBonus,
      networkCap: tierBenefits.networkCap,
      specialFeatures: tierBenefits.specialFeatures,
      
      // Rankings
      rank: rawUserData.rpRank || 0,
      percentile: rawUserData.rpPercentile || 0,
      
      // Growth metrics
      monthlyGrowthRate: rawUserData.monthlyNetworkGrowth || 0,
      projectedMonthlyRP: this.calculateProjectedMonthlyRP(rawUserData)
    };
  }

  /**
   * Get comprehensive token balance information
   */
  private async getTokenBalance(userWallet: PublicKey): Promise<{
    fin: number;
    sFin: number;
    usdFin: number;
    sUsdFin: number;
    totalValue: number;
  }> {
    try {
      // Get $FIN balance
      const finTokenAccount = await getAssociatedTokenAddress(
        this.mintAddress,
        userWallet
      );
      
      let finBalance = 0;
      try {
        const finAccount = await getAccount(this.connection, finTokenAccount);
        finBalance = Number(finAccount.amount) / Math.pow(10, 9); // Assuming 9 decimals
      } catch (error) {
        // Account doesn't exist yet
      }
      
      // TODO: Get other token balances when implemented
      const sFinBalance = 0; // Staked FIN
      const usdFinBalance = 0; // USD-pegged stablecoin
      const sUsdFinBalance = 0; // Staked USD-FIN
      
      return {
        fin: finBalance,
        sFin: sFinBalance,
        usdFin: usdFinBalance,
        sUsdFin: sUsdFinBalance,
        totalValue: finBalance + sFinBalance + usdFinBalance + sUsdFinBalance
      };
    } catch (error) {
      console.error('Error fetching token balance:', error);
      return {
        fin: 0,
        sFin: 0,
        usdFin: 0,
        sUsdFin: 0,
        totalValue: 0
      };
    }
  }

  /**
   * Get staking information and benefits
   */
  private async getStakingInfo(userWallet: PublicKey): Promise<StakingInfo> {
    // TODO: Implement staking contract interaction
    return {
      isStaking: false,
      stakedAmount: 0,
      stakingTier: 0,
      aprRate: 0,
      pendingRewards: 0,
      lockupEndDate: null,
      loyaltyMultiplier: 1.0,
      benefits: {
        miningBoost: 0,
        xpMultiplier: 0,
        rpBonus: 0,
        specialFeatures: []
      }
    };
  }

  /**
   * Calculate integrated statistics across all three systems
   */
  private async calculateIntegratedStats(rawUserData: any): Promise<{
    overallScore: number;
    projectedDailyEarnings: number;
    networkRank: number;
    miningRegression: number;
    xpRegression: number;
    referralRegression: number;
  }> {
    // XP regression based on level
    const xpRegression = Math.exp(-0.01 * rawUserData.xpLevel);
    
    // Mining regression based on holdings
    const miningRegression = Math.exp(-0.001 * rawUserData.totalHoldings);
    
    // Referral regression based on network size and quality
    const networkQuality = this.calculateNetworkQuality(rawUserData);
    const referralRegression = Math.exp(-0.0001 * rawUserData.totalNetworkSize * networkQuality.score);
    
    // Overall score calculation (weighted combination)
    const xpScore = rawUserData.totalXP * 0.001; // Normalize XP
    const rpScore = rawUserData.totalRP * 0.01; // Normalize RP
    const miningScore = rawUserData.totalMined * 1.0; // Mining contribution
    const qualityScore = rawUserData.averageQualityScore || 1.0;
    
    const overallScore = (xpScore * 0.3 + rpScore * 0.3 + miningScore * 0.4) * qualityScore;
    
    // Projected daily earnings (integrated calculation)
    const baseDaily = await this.calculateProjectedDailyEarnings(rawUserData);
    
    return {
      overallScore,
      projectedDailyEarnings: baseDaily,
      networkRank: rawUserData.overallRank || 0,
      miningRegression,
      xpRegression,
      referralRegression
    };
  }

  /**
   * Helper method to calculate network quality
   */
  private calculateNetworkQuality(rawUserData: any): NetworkQuality {
    const totalReferrals = rawUserData.directReferrals || 0;
    const activeReferrals = rawUserData.activeReferrals || 0;
    const averageLevel = rawUserData.averageReferralLevel || 1;
    const diversityScore = rawUserData.networkDiversityScore || 0.5;
    
    if (totalReferrals === 0) {
      return {
        score: 0.5,
        grade: 'C',
        metrics: {
          retention: 0,
          activity: 0,
          diversity: 0.5,
          growth: 0
        }
      };
    }
    
    const retention = activeReferrals / totalReferrals;
    const activity = Math.min(1.0, averageLevel / 25); // Normalized to max level 25
    const growth = Math.min(1.0, rawUserData.monthlyNetworkGrowth || 0);
    
    const score = (retention * 0.4 + activity * 0.3 + diversityScore * 0.2 + growth * 0.1);
    
    let grade: 'S' | 'A' | 'B' | 'C' | 'D' | 'F';
    if (score >= 0.95) grade = 'S';
    else if (score >= 0.85) grade = 'A';
    else if (score >= 0.75) grade = 'B';
    else if (score >= 0.60) grade = 'C';
    else if (score >= 0.40) grade = 'D';
    else grade = 'F';
    
    return {
      score,
      grade,
      metrics: {
        retention,
        activity,
        diversity: diversityScore,
        growth
      }
    };
  }

  /**
   * Build initialization instruction for new user
   */
  private async buildInitializeUserInstruction(
    userWallet: PublicKey,
    userAccountPDA: PublicKey,
    tokenAccount: PublicKey,
    referralCode?: string,
    kycData?: any
  ): Promise<TransactionInstruction> {
    // This would integrate with your Anchor program
    // For now, returning a placeholder instruction
    
    const data = Buffer.alloc(256); // Placeholder for instruction data
    let offset = 0;
    
    // Write instruction discriminator
    data.writeUInt8(0, offset); // Initialize user discriminator
    offset += 1;
    
    // Write referral code if provided
    if (referralCode) {
      const codeBuffer = Buffer.from(referralCode, 'utf8');
      data.writeUInt32LE(codeBuffer.length, offset);
      offset += 4;
      codeBuffer.copy(data, offset);
      offset += codeBuffer.length;
    } else {
      data.writeUInt32LE(0, offset);
      offset += 4;
    }
    
    // Write KYC data if provided
    if (kycData) {
      data.writeUInt8(1, offset); // KYC present flag
      offset += 1;
      // Add KYC data serialization here
    } else {
      data.writeUInt8(0, offset);
      offset += 1;
    }
    
    return new TransactionInstruction({
      keys: [
        { pubkey: userWallet, isSigner: true, isWritable: true },
        { pubkey: userAccountPDA, isSigner: false, isWritable: true },
        { pubkey: tokenAccount, isSigner: false, isWritable: true },
        { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
        { pubkey: TOKEN_PROGRAM_ID, isSigner: false, isWritable: false }
      ],
      programId: this.programId,
      data: data.slice(0, offset)
    });
  }

  /**
   * Deserialize user account data from blockchain
   */
  private deserializeUserAccount(data: Buffer): any {
    // This would integrate with your Anchor IDL
    // For now, returning a mock structure
    return {
      createdAt: Date.now(),
      lastActive: Date.now(),
      kycVerified: false,
      humanProbability: 0.85,
      suspiciousScore: 0.1,
      totalXP: 0,
      xpLevel: 1,
      totalRP: 0,
      rpTier: 0,
      totalMined: 0,
      totalHoldings: 0,
      directReferrals: 0,
      activeReferrals: 0,
      totalNetworkSize: 0,
      connectedPlatforms: [],
      socialStats: {},
      lifetimeEarnings: 0,
      currentStreak: 0,
      maxStreak: 0,
      averageQualityScore: 1.0,
      miningActive: true
    };
  }

  /**
   * Cache management methods
   */
  private isValidCache(key: string): boolean {
    const lastUpdate = this.lastCacheUpdate.get(key);
    return lastUpdate ? (Date.now() - lastUpdate) < this.CACHE_TTL : false;
  }

  private updateCache(key: string, data: UserAccount): void {
    this.userCache.set(key, data);
    this.lastCacheUpdate.set(key, Date.now());
  }

  /**
   * Get current network state for calculations
   */
  private async getNetworkState(): Promise<any> {
    // This would fetch from your network stats account
    return {
      totalUsers: 50000, // Mock data
      totalMiners: 25000,
      totalSupply: 1000000,
      currentPhase: MiningPhase.Finizen
    };
  }

  /**
   * Utility methods for calculations
   */
  private getCurrentMiningPhase(totalUsers: number): MiningPhase {
    if (totalUsers < 100000) return MiningPhase.Finizen;
    if (totalUsers < 1000000) return MiningPhase.Growth;
    if (totalUsers < 10000000) return MiningPhase.Maturity;
    return MiningPhase.Stability;
  }

  private getBaseMiningRate(phase: MiningPhase): number {
    switch (phase) {
      case MiningPhase.Finizen: return 0.1;
      case MiningPhase.Growth: return 0.05;
      case MiningPhase.Maturity: return 0.025;
      case MiningPhase.Stability: return 0.01;
      default: return 0.01;
    }
  }

  private calculateLevelFromXP(xp: number): number {
    // Progressive XP requirements: Level n requires n^2 * 100 XP
    return Math.floor(Math.sqrt(xp / 100)) + 1;
  }

  private getXPRequiredForLevel(level: number): number {
    return (level - 1) * (level - 1) * 100;
  }

  private getXPTier(level: number): UserLevel {
    if (level <= 10) return UserLevel.Bronze;
    if (level <= 25) return UserLevel.Silver;
    if (level <= 50) return UserLevel.Gold;
    if (level <= 75) return UserLevel.Platinum;
    if (level <= 100) return UserLevel.Diamond;
    return UserLevel.Mythic;
  }

  private getRPTier(rp: number): RPTier {
    if (rp < 1000) return RPTier.Explorer;
    if (rp < 5000) return RPTier.Connector;
    if (rp < 15000) return RPTier.Influencer;
    if (rp < 50000) return RPTier.Leader;
    return RPTier.Ambassador;
  }

  private getXPMiningMultiplier(level: number): number {
    const tier = this.getXPTier(level);
    switch (tier) {
      case UserLevel.Bronze: return 1.0 + (level * 0.02); // 1.0x - 1.2x
      case UserLevel.Silver: return 1.3 + ((level - 10) * 0.033); // 1.3x - 1.8x
      case UserLevel.Gold: return 1.9 + ((level - 25) * 0.024); // 1.9x - 2.5x
      case UserLevel.Platinum: return 2.6 + ((level - 50) * 0.024); // 2.6x - 3.2x
      case UserLevel.Diamond: return 3.3 + ((level - 75) * 0.028); // 3.3x - 4.0x
      case UserLevel.Mythic: return 4.1 + ((level - 100) * 0.045); // 4.1x - 5.0x (capped)
      default: return 1.0;
    }
  }

  private getRPMiningMultiplier(tier: RPTier): number {
    switch (tier) {
      case RPTier.Explorer: return 1.0;
      case RPTier.Connector: return 1.2;
      case RPTier.Influencer: return 1.5;
      case RPTier.Leader: return 2.0;
      case RPTier.Ambassador: return 3.0;
      default: return 1.0;
    }
  }

  private getDailyMiningCap(level: number): number {
    const tier = this.getXPTier(level);
    switch (tier) {
      case UserLevel.Bronze: return 0.5 + (level * 0.15); // 0.5-2.0 $FIN
      case UserLevel.Silver: return 2.0 + ((level - 10) * 0.133); // 2.0-4.0 $FIN
      case UserLevel.Gold: return 4.0 + ((level - 25) * 0.08); // 4.0-6.0 $FIN
      case UserLevel.Platinum: return 6.0 + ((level - 50) * 0.08); // 6.0-8.0 $FIN
      case UserLevel.Diamond: return 8.0 + ((level - 75) * 0.08); // 8.0-10.0 $FIN
      case UserLevel.Mythic: return 10.0 + ((level - 100) * 0.25); // 10.0-15.0 $FIN
      default: return 0.5;
    }
  }

  private getStreakMultiplier(streak: number): number {
    if (streak <= 0) return 1.0;
    if (streak <= 7) return 1.0 + (streak * 0.05); // Up to 1.35x
    if (streak <= 30) return 1.35 + ((streak - 7) * 0.02); // Up to 1.81x
    if (streak <= 90) return 1.81 + ((streak - 30) * 0.01); // Up to 2.41x
    return Math.min(3.0, 2.41 + ((streak - 90) * 0.005)); // Cap at 3.0x
  }

  private calculateActivityBonus(rawUserData: any): number {
    const baseActivity = 1.0;
    const dailyPosts = rawUserData.dailyPosts || 0;
    const weeklyEngagement = rawUserData.weeklyEngagement || 0;
    const platformDiversity = rawUserData.connectedPlatforms?.length || 1;
    
    // Bonus for consistent posting (max +20%)
    const postingBonus = Math.min(0.2, dailyPosts * 0.05);
    
    // Bonus for high engagement (max +15%)
    const engagementBonus = Math.min(0.15, weeklyEngagement * 0.001);
    
    // Bonus for platform diversity (max +10%)
    const diversityBonus = Math.min(0.1, (platformDiversity - 1) * 0.025);
    
    return baseActivity + postingBonus + engagementBonus + diversityBonus;
  }

  private calculateMiningEfficiency(rawUserData: any): number {
    const timeActive = Date.now() - rawUserData.createdAt;
    const daysActive = Math.max(1, timeActive / (24 * 60 * 60 * 1000));
    const avgDailyMining = rawUserData.totalMined / daysActive;
    
    // Efficiency based on daily average vs theoretical maximum
    const theoreticalMax = this.getDailyMiningCap(rawUserData.xpLevel);
    return Math.min(1.0, avgDailyMining / theoreticalMax);
  }

  private getNextMiningMilestone(totalMined: number): {
    target: number;
    reward: string;
    progress: number;
  } {
    const milestones = [
      { target: 100, reward: "First Century Badge + 10% Mining Boost (24h)" },
      { target: 500, reward: "Mining Veteran Badge + Rare NFT Card" },
      { target: 1000, reward: "Mining Master Badge + 25% Mining Boost (48h)" },
      { target: 5000, reward: "Mining Legend Badge + Epic NFT Collection" },
      { target: 10000, reward: "Mining Emperor Badge + Lifetime 5% Boost" }
    ];
    
    const nextMilestone = milestones.find(m => m.target > totalMined);
    if (!nextMilestone) {
      return {
        target: 50000,
        reward: "Ultimate Mining God Badge + Custom NFT",
        progress: Math.min(100, (totalMined / 50000) * 100)
      };
    }
    
    const prevTarget = milestones[milestones.indexOf(nextMilestone) - 1]?.target || 0;
    const progress = ((totalMined - prevTarget) / (nextMilestone.target - prevTarget)) * 100;
    
    return {
      target: nextMilestone.target,
      reward: nextMilestone.reward,
      progress: Math.max(0, progress)
    };
  }

  private getXPSpecialFeatures(level: number): string[] {
    const features: string[] = [];
    const tier = this.getXPTier(level);
    
    switch (tier) {
      case UserLevel.Bronze:
        features.push("Basic Mining", "Social Integration", "Daily Rewards");
        break;
      case UserLevel.Silver:
        features.push("Special Cards Access", "Premium Badge", "Enhanced Support");
        break;
      case UserLevel.Gold:
        features.push("Guild Leadership", "Advanced Analytics", "VIP Events");
        break;
      case UserLevel.Platinum:
        features.push("Creator Monetization", "Exclusive Partnerships", "Custom Referral Codes");
        break;
      case UserLevel.Diamond:
        features.push("Exclusive Events", "Priority Features", "Beta Access");
        break;
      case UserLevel.Mythic:
        features.push("DAO Governance", "Maximum Benefits", "Legendary Status");
        break;
    }
    
    return features;
  }

  private getRPTierBenefits(tier: RPTier): {
    miningBonus: number;
    referralBonus: number;
    networkCap: number;
    specialFeatures: string[];
  } {
    switch (tier) {
      case RPTier.Explorer:
        return {
          miningBonus: 0,
          referralBonus: 0.1, // 10% of L1
          networkCap: 10,
          specialFeatures: ["Basic Referral Link"]
        };
      case RPTier.Connector:
        return {
          miningBonus: 0.2, // +20%
          referralBonus: 0.15, // 15% of L1, 5% of L2
          networkCap: 25,
          specialFeatures: ["Custom Referral Code", "Basic Analytics"]
        };
      case RPTier.Influencer:
        return {
          miningBonus: 0.5, // +50%
          referralBonus: 0.20, // 20% of L1, 8% of L2, 3% of L3
          networkCap: 50,
          specialFeatures: ["Referral Analytics", "Performance Dashboard", "Special Events"]
        };
      case RPTier.Leader:
        return {
          miningBonus: 1.0, // +100%
          referralBonus: 0.25, // 25% of L1, 10% of L2, 5% of L3
          networkCap: 100,
          specialFeatures: ["Exclusive Events", "Priority Support", "Advanced Tools"]
        };
      case RPTier.Ambassador:
        return {
          miningBonus: 2.0, // +200%
          referralBonus: 0.30, // 30% of L1, 15% of L2, 8% of L3
          networkCap: -1, // Unlimited
          specialFeatures: ["DAO Governance", "Custom Events", "Ambassador Program", "Revenue Share"]
        };
      default:
        return {
          miningBonus: 0,
          referralBonus: 0,
          networkCap: 0,
          specialFeatures: []
        };
    }
  }

  private calculateDailyNetworkRP(rawUserData: any): number {
    const activeReferrals = rawUserData.activeReferrals || 0;
    const avgReferralActivity = rawUserData.avgReferralActivity || 0;
    const tier = this.getRPTier(rawUserData.totalRP || 0);
    const tierBenefits = this.getRPTierBenefits(tier);
    
    // Base RP from active referral network
    const baseDaily = activeReferrals * avgReferralActivity * tierBenefits.referralBonus;
    
    // Network quality multiplier
    const networkQuality = this.calculateNetworkQuality(rawUserData);
    
    return baseDaily * networkQuality.score;
  }

  private calculateProjectedMonthlyRP(rawUserData: any): number {
    const currentDailyRP = this.calculateDailyNetworkRP(rawUserData);
    const growthRate = rawUserData.monthlyNetworkGrowth || 0;
    
    // Project growth over 30 days
    let projectedMonthly = 0;
    for (let day = 1; day <= 30; day++) {
      const dailyGrowthFactor = 1 + (growthRate / 30); // Distribute growth over month
      const projectedDailyRP = currentDailyRP * Math.pow(dailyGrowthFactor, day);
      projectedMonthly += projectedDailyRP;
    }
    
    return projectedMonthly;
  }

  private async calculateProjectedDailyEarnings(rawUserData: any): Promise<number> {
    // Calculate base mining earnings
    const networkState = await this.getNetworkState();
    const miningStats = await this.calculateMiningStats(rawUserData, networkState);
    const baseMining = miningStats.dailyRate;
    
    // Add XP-based bonuses
    const xpBonus = baseMining * 0.2 * this.getXPMiningMultiplier(rawUserData.xpLevel);
    
    // Add RP-based bonuses
    const rpBonus = baseMining * 0.3 * this.getRPMiningMultiplier(this.getRPTier(rawUserData.totalRP || 0));
    
    // Add staking rewards (when implemented)
    const stakingRewards = 0; // TODO: Implement staking calculations
    
    // Add network activity rewards
    const networkActivityRewards = this.calculateDailyNetworkRP(rawUserData) * 0.1; // Convert RP to $FIN equivalent
    
    return baseMining + xpBonus + rpBonus + stakingRewards + networkActivityRewards;
  }

  private calculateSecurityStatus(rawUserData: any): SecurityStatus {
    const kycVerified = rawUserData.kycVerified || false;
    const humanProbability = rawUserData.humanProbability || 0.5;
    const suspiciousScore = rawUserData.suspiciousScore || 0;
    const accountAge = Date.now() - rawUserData.createdAt;
    const daysSinceCreation = accountAge / (24 * 60 * 60 * 1000);
    
    let riskLevel: 'LOW' | 'MEDIUM' | 'HIGH' | 'CRITICAL';
    let trustScore = 0;
    
    // Base trust from KYC
    if (kycVerified) trustScore += 40;
    
    // Trust from human probability
    trustScore += humanProbability * 30;
    
    // Trust from account age (max 20 points for 30+ days)
    trustScore += Math.min(20, daysSinceCreation * 0.67);
    
    // Subtract for suspicious activity
    trustScore -= suspiciousScore * 50;
    
    // Network trust (if user has successful referrals)
    const networkTrust = Math.min(10, (rawUserData.activeReferrals || 0) * 2);
    trustScore += networkTrust;
    
    // Determine risk level
    if (trustScore >= 80) riskLevel = 'LOW';
    else if (trustScore >= 60) riskLevel = 'MEDIUM';
    else if (trustScore >= 40) riskLevel = 'HIGH';
    else riskLevel = 'CRITICAL';
    
    return {
      kycStatus: kycVerified ? 'VERIFIED' : 'PENDING',
      trustScore: Math.max(0, Math.min(100, trustScore)),
      riskLevel,
      humanProbability,
      suspiciousScore,
      securityFlags: this.getSecurityFlags(rawUserData),
      lastSecurityCheck: rawUserData.lastSecurityCheck || rawUserData.createdAt
    };
  }

  private getSecurityFlags(rawUserData: any): string[] {
    const flags: string[] = [];
    
    if (!rawUserData.kycVerified) flags.push('KYC_PENDING');
    if (rawUserData.humanProbability < 0.7) flags.push('LOW_HUMAN_PROBABILITY');
    if (rawUserData.suspiciousScore > 0.3) flags.push('SUSPICIOUS_ACTIVITY');
    if (rawUserData.multipleDevices) flags.push('MULTIPLE_DEVICES');
    if (rawUserData.vpnUsage) flags.push('VPN_USAGE');
    if (rawUserData.rapidActivity) flags.push('RAPID_ACTIVITY_PATTERN');
    if (rawUserData.circularReferrals) flags.push('CIRCULAR_REFERRALS');
    
    return flags;
  }

  /**
   * Update user activity data
   */
  async updateUserActivity(
    userWallet: PublicKey,
    activity: UserActivityData
  ): Promise<string> {
    try {
      const [userAccountPDA] = PublicKey.findProgramAddressSync(
        [Buffer.from('user'), userWallet.toBuffer()],
        this.programId
      );

      // Build update activity instruction
      const updateInstruction = await this.buildUpdateActivityInstruction(
        userWallet,
        userAccountPDA,
        activity
      );

      const transaction = new Transaction().add(updateInstruction);
      const signature = await this.client.sendAndConfirmTransaction(transaction);

      // Invalidate cache to force refresh
      this.userCache.delete(userWallet.toString());
      this.lastCacheUpdate.delete(userWallet.toString());

      return signature;
    } catch (error) {
      console.error('Failed to update user activity:', error);
      throw new Error(`Activity update failed: ${error.message}`);
    }
  }

  /**
   * Claim pending mining rewards
   */
  async claimMiningRewards(userWallet: PublicKey): Promise<{
    signature: string;
    claimedAmount: number;
    newBalance: number;
  }> {
    try {
      const userAccount = await this.getUserAccount(userWallet);
      const pendingRewards = userAccount.mining.pendingRewards;

      if (pendingRewards <= 0) {
        throw new Error('No pending rewards to claim');
      }

      const [userAccountPDA] = PublicKey.findProgramAddressSync(
        [Buffer.from('user'), userWallet.toBuffer()],
        this.programId
      );

      const tokenAccount = await getAssociatedTokenAddress(
        this.mintAddress,
        userWallet
      );

      // Build claim instruction
      const claimInstruction = await this.buildClaimMiningInstruction(
        userWallet,
        userAccountPDA,
        tokenAccount,
        pendingRewards
      );

      const transaction = new Transaction().add(claimInstruction);
      const signature = await this.client.sendAndConfirmTransaction(transaction);

      // Get updated balance
      const updatedBalance = await this.getTokenBalance(userWallet);

      // Invalidate cache
      this.userCache.delete(userWallet.toString());
      this.lastCacheUpdate.delete(userWallet.toString());

      return {
        signature,
        claimedAmount: pendingRewards,
        newBalance: updatedBalance.fin
      };
    } catch (error) {
      console.error('Failed to claim mining rewards:', error);
      throw new Error(`Mining claim failed: ${error.message}`);
    }
  }

  /**
   * Connect social media platform
   */
  async connectSocialPlatform(
    userWallet: PublicKey,
    platform: SocialPlatform,
    platformData: {
      username: string;
      userId: string;
      accessToken: string;
      profileUrl: string;
    }
  ): Promise<string> {
    try {
      const [userAccountPDA] = PublicKey.findProgramAddressSync(
        [Buffer.from('user'), userWallet.toBuffer()],
        this.programId
      );

      // Build connect platform instruction
      const connectInstruction = await this.buildConnectPlatformInstruction(
        userWallet,
        userAccountPDA,
        platform,
        platformData
      );

      const transaction = new Transaction().add(connectInstruction);
      const signature = await this.client.sendAndConfirmTransaction(transaction);

      // Invalidate cache
      this.userCache.delete(userWallet.toString());
      this.lastCacheUpdate.delete(userWallet.toString());

      return signature;
    } catch (error) {
      console.error('Failed to connect social platform:', error);
      throw new Error(`Platform connection failed: ${error.message}`);
    }
  }

  /**
   * Get user's referral network analysis
   */
  async getReferralNetworkAnalysis(userWallet: PublicKey): Promise<{
    networkMap: any[];
    growthMetrics: any;
    qualityAnalysis: any;
    earnings: any;
  }> {
    const userAccount = await this.getUserAccount(userWallet);
    
    // This would fetch detailed network data
    return {
      networkMap: [], // Tree structure of referral network
      growthMetrics: {
        weeklyGrowth: userAccount.referral.monthlyGrowthRate / 4,
        monthlyGrowth: userAccount.referral.monthlyGrowthRate,
        retentionRate: userAccount.referral.retentionRate,
        conversionRate: userAccount.referral.activeReferrals / Math.max(1, userAccount.referral.directReferrals)
      },
      qualityAnalysis: userAccount.referral.networkQuality,
      earnings: {
        daily: userAccount.referral.dailyRP,
        weekly: userAccount.referral.weeklyRP,
        monthly: userAccount.referral.projectedMonthlyRP,
        lifetime: userAccount.referral.total
      }
    };
  }

  /**
   * Build helper instructions (these would integrate with your Anchor program)
   */
  private async buildUpdateActivityInstruction(
    userWallet: PublicKey,
    userAccountPDA: PublicKey,
    activity: UserActivityData
  ): Promise<TransactionInstruction> {
    // Placeholder instruction builder
    const data = Buffer.alloc(128);
    let offset = 0;
    
    data.writeUInt8(1, offset); // Update activity discriminator
    offset += 1;
    
    // Serialize activity data
    data.writeUInt32LE(activity.platform, offset);
    offset += 4;
    data.writeUInt32LE(activity.activityType, offset);
    offset += 4;
    data.writeDoubleLE(activity.qualityScore || 1.0, offset);
    offset += 8;
    
    return new TransactionInstruction({
      keys: [
        { pubkey: userWallet, isSigner: true, isWritable: false },
        { pubkey: userAccountPDA, isSigner: false, isWritable: true }
      ],
      programId: this.programId,
      data: data.slice(0, offset)
    });
  }

  private async buildClaimMiningInstruction(
    userWallet: PublicKey,
    userAccountPDA: PublicKey,
    tokenAccount: PublicKey,
    amount: number
  ): Promise<TransactionInstruction> {
    const data = Buffer.alloc(16);
    let offset = 0;
    
    data.writeUInt8(2, offset); // Claim mining discriminator
    offset += 1;
    
    // Amount to claim (as lamports)
    const lamports = new BN(amount * Math.pow(10, 9));
    lamports.toArrayLike(Buffer, 'le', 8).copy(data, offset);
    offset += 8;
    
    return new TransactionInstruction({
      keys: [
        { pubkey: userWallet, isSigner: true, isWritable: false },
        { pubkey: userAccountPDA, isSigner: false, isWritable: true },
        { pubkey: tokenAccount, isSigner: false, isWritable: true },
        { pubkey: this.mintAddress, isSigner: false, isWritable: true },
        { pubkey: TOKEN_PROGRAM_ID, isSigner: false, isWritable: false }
      ],
      programId: this.programId,
      data: data.slice(0, offset)
    });
  }

  private async buildConnectPlatformInstruction(
    userWallet: PublicKey,
    userAccountPDA: PublicKey,
    platform: SocialPlatform,
    platformData: any
  ): Promise<TransactionInstruction> {
    const data = Buffer.alloc(256);
    let offset = 0;
    
    data.writeUInt8(3, offset); // Connect platform discriminator
    offset += 1;
    
    data.writeUInt32LE(platform, offset);
    offset += 4;
    
    // Platform data (simplified)
    const usernameBuffer = Buffer.from(platformData.username, 'utf8');
    data.writeUInt32LE(usernameBuffer.length, offset);
    offset += 4;
    usernameBuffer.copy(data, offset);
    offset += usernameBuffer.length;
    
    return new TransactionInstruction({
      keys: [
        { pubkey: userWallet, isSigner: true, isWritable: false },
        { pubkey: userAccountPDA, isSigner: false, isWritable: true }
      ],
      programId: this.programId,
      data: data.slice(0, offset)
    });
  }

  /**
   * Batch operations for efficiency
   */
  async batchGetUserAccounts(userWallets: PublicKey[]): Promise<Map<string, UserAccount>> {
    const results = new Map<string, UserAccount>();
    
    // Process in batches to avoid RPC limits
    const batchSize = 10;
    for (let i = 0; i < userWallets.length; i += batchSize) {
      const batch = userWallets.slice(i, i + batchSize);
      const promises = batch.map(wallet => 
        this.getUserAccount(wallet).catch(error => {
          console.warn(`Failed to fetch user ${wallet.toString()}:`, error);
          return null;
        })
      );
      
      const batchResults = await Promise.all(promises);
      batchResults.forEach((result, index) => {
        if (result) {
          results.set(batch[index].toString(), result);
        }
      });
    }
    
    return results;
  }

  /**
   * Real-time event listeners for user account changes
   */
  subscribeToUserUpdates(
    userWallet: PublicKey,
    callback: (account: UserAccount) => void
  ): number {
    const [userAccountPDA] = PublicKey.findProgramAddressSync(
      [Buffer.from('user'), userWallet.toBuffer()],
      this.programId
    );

    return this.connection.onAccountChange(
      userAccountPDA,
      async (accountInfo) => {
        try {
          // Invalidate cache and fetch fresh data
          const cacheKey = userWallet.toString();
          this.userCache.delete(cacheKey);
          this.lastCacheUpdate.delete(cacheKey);
          
          const updatedAccount = await this.getUserAccount(userWallet, true);
          callback(updatedAccount);
        } catch (error) {
          console.error('Error processing user account update:', error);
        }
      },
      'confirmed'
    );
  }

  /**
   * Unsubscribe from account changes
   */
  async unsubscribeFromUserUpdates(subscriptionId: number): Promise<void> {
    await this.connection.removeAccountChangeListener(subscriptionId);
  }

  /**
   * Export user data for analytics or backup
   */
  async exportUserData(userWallet: PublicKey): Promise<{
    account: UserAccount;
    networkAnalysis: any;
    performanceMetrics: any;
    exportTimestamp: number;
  }> {
    const account = await this.getUserAccount(userWallet);
    const networkAnalysis = await this.getReferralNetworkAnalysis(userWallet);
    
    const performanceMetrics = {
      miningEfficiency: account.mining.efficiency,
      xpGrowthRate: account.xp.weekly / Math.max(1, account.xp.daily * 7),
      networkGrowthRate: account.referral.monthlyGrowthRate,
      overallPerformance: account.overallScore,
      rankingPercentiles: {
        mining: 100 - account.mining.rank,
        xp: 100 - account.xp.rank,
        referral: 100 - account.referral.rank
      }
    };
    
    return {
      account,
      networkAnalysis,
      performanceMetrics,
      exportTimestamp: Date.now()
    };
  }
}
