// finova-net/finova/client/typescript/src/accounts/mining.ts

/**
 * Finova Network - Mining Accounts Client
 * TypeScript SDK for interacting with Finova Core mining smart contracts
 * 
 * @version 1.0.0
 * @author Finova Network Team
 * @license MIT
 */

import {
  PublicKey,
  Connection,
  AccountInfo,
  GetProgramAccountsFilter,
  DataSizeFilter,
  MemcmpFilter,
  Commitment,
  RpcResponseAndContext,
} from '@solana/web3.js';
import { Program, BN, IdlAccounts, Address } from '@project-serum/anchor';
import { FinovaCore } from '../types/finova_core';
import { calculateMiningRate, calculateXPMultiplier, calculateRPValue } from '../utils/calculations';
import { validatePublicKey, validateMiningParams } from '../utils/validation';
import { 
  FINOVA_PROGRAM_ID, 
  MINING_ACCOUNT_SIZE, 
  USER_ACCOUNT_SIZE,
  NETWORK_ACCOUNT_SIZE,
  MAX_MINING_RATE,
  MIN_MINING_RATE,
  REGRESSION_FACTOR 
} from '../constants';

// Type definitions for mining-related accounts
export type MiningAccount = IdlAccounts<FinovaCore>['miningAccount'];
export type UserAccount = IdlAccounts<FinovaCore>['userAccount'];
export type NetworkAccount = IdlAccounts<FinovaCore>['networkAccount'];

// Mining state interfaces
export interface MiningState {
  user: PublicKey;
  baseRate: BN;
  lastMined: BN;
  totalMined: BN;
  currentStreak: number;
  bonusMultiplier: BN;
  isActive: boolean;
  phase: MiningPhase;
  regression: BN;
  qualityScore: BN;
  securityBonus: BN;
  referralBonus: BN;
  xpMultiplier: BN;
  rpMultiplier: BN;
  stakingBonus: BN;
  cardBoosts: CardBoost[];
  lastActivityTime: BN;
  totalHours: BN;
  bump: number;
}

export interface UserMiningData {
  userAccount: UserAccount;
  miningAccount: MiningAccount;
  networkAccount?: NetworkAccount;
  effectiveRate: number;
  dailyLimit: number;
  currentPhase: MiningPhase;
  bonuses: MiningBonuses;
  penalties: MiningPenalties;
  projectedEarnings: ProjectedEarnings;
}

export interface MiningBonuses {
  pioneerBonus: number;
  referralBonus: number;
  securityBonus: number;
  xpBonus: number;
  rpBonus: number;
  stakingBonus: number;
  cardBoosts: number;
  qualityMultiplier: number;
  streakBonus: number;
}

export interface MiningPenalties {
  regressionFactor: number;
  whaleDecay: number;
  inactivityPenalty: number;
  qualityReduction: number;
}

export interface ProjectedEarnings {
  hourlyRate: number;
  dailyEstimate: number;
  weeklyEstimate: number;
  monthlyEstimate: number;
}

export interface CardBoost {
  cardType: CardType;
  multiplier: BN;
  duration: BN;
  startTime: BN;
  isActive: boolean;
}

export enum MiningPhase {
  Finizen = 0,
  Growth = 1,
  Maturity = 2,
  Stability = 3,
}

export enum CardType {
  DoubleMining = 0,
  TripleMining = 1,
  MiningFrenzy = 2,
  EternalMiner = 3,
  XPDouble = 4,
  StreakSaver = 5,
  LevelRush = 6,
  XPMagnet = 7,
  ReferralBoost = 8,
  NetworkAmplifier = 9,
  AmbassadorPass = 10,
  NetworkKing = 11,
}

// Error classes
export class MiningAccountError extends Error {
  constructor(message: string, public code?: string) {
    super(message);
    this.name = 'MiningAccountError';
  }
}

export class ValidationError extends MiningAccountError {
  constructor(message: string) {
    super(message, 'VALIDATION_ERROR');
  }
}

export class NetworkError extends MiningAccountError {
  constructor(message: string) {
    super(message, 'NETWORK_ERROR');
  }
}

/**
 * Main class for managing mining-related accounts and calculations
 */
export class MiningAccounts {
  private program: Program<FinovaCore>;
  private connection: Connection;
  private commitment: Commitment;

  constructor(
    program: Program<FinovaCore>,
    connection: Connection,
    commitment: Commitment = 'confirmed'
  ) {
    this.program = program;
    this.connection = connection;
    this.commitment = commitment;
  }

  /**
   * Get mining account for a specific user
   * @param userPublicKey - The user's public key
   * @returns Mining account data or null if not found
   */
  async getMiningAccount(userPublicKey: PublicKey): Promise<MiningAccount | null> {
    try {
      validatePublicKey(userPublicKey);
      
      const [miningPDA] = await this.findMiningAccountPDA(userPublicKey);
      const accountInfo = await this.connection.getAccountInfo(miningPDA, this.commitment);
      
      if (!accountInfo) {
        return null;
      }

      return this.program.coder.accounts.decode('miningAccount', accountInfo.data);
    } catch (error) {
      throw new MiningAccountError(`Failed to get mining account: ${error.message}`);
    }
  }

  /**
   * Get user account data
   * @param userPublicKey - The user's public key
   * @returns User account data or null if not found
   */
  async getUserAccount(userPublicKey: PublicKey): Promise<UserAccount | null> {
    try {
      validatePublicKey(userPublicKey);
      
      const [userPDA] = await this.findUserAccountPDA(userPublicKey);
      const accountInfo = await this.connection.getAccountInfo(userPDA, this.commitment);
      
      if (!accountInfo) {
        return null;
      }

      return this.program.coder.accounts.decode('userAccount', accountInfo.data);
    } catch (error) {
      throw new MiningAccountError(`Failed to get user account: ${error.message}`);
    }
  }

  /**
   * Get network account data
   * @returns Network account data or null if not found
   */
  async getNetworkAccount(): Promise<NetworkAccount | null> {
    try {
      const [networkPDA] = await this.findNetworkAccountPDA();
      const accountInfo = await this.connection.getAccountInfo(networkPDA, this.commitment);
      
      if (!accountInfo) {
        return null;
      }

      return this.program.coder.accounts.decode('networkAccount', accountInfo.data);
    } catch (error) {
      throw new MiningAccountError(`Failed to get network account: ${error.message}`);
    }
  }

  /**
   * Get comprehensive mining data for a user
   * @param userPublicKey - The user's public key
   * @returns Complete mining data including calculations
   */
  async getUserMiningData(userPublicKey: PublicKey): Promise<UserMiningData | null> {
    try {
      validatePublicKey(userPublicKey);

      const [userAccount, miningAccount, networkAccount] = await Promise.all([
        this.getUserAccount(userPublicKey),
        this.getMiningAccount(userPublicKey),
        this.getNetworkAccount(),
      ]);

      if (!userAccount || !miningAccount) {
        return null;
      }

      // Calculate effective mining rate and bonuses
      const bonuses = this.calculateMiningBonuses(userAccount, miningAccount, networkAccount);
      const penalties = this.calculateMiningPenalties(userAccount, miningAccount, networkAccount);
      const effectiveRate = this.calculateEffectiveMiningRate(miningAccount, bonuses, penalties);
      const dailyLimit = this.calculateDailyLimit(userAccount, miningAccount);
      const currentPhase = this.getCurrentMiningPhase(networkAccount);
      const projectedEarnings = this.calculateProjectedEarnings(effectiveRate, dailyLimit);

      return {
        userAccount,
        miningAccount,
        networkAccount,
        effectiveRate,
        dailyLimit,
        currentPhase,
        bonuses,
        penalties,
        projectedEarnings,
      };
    } catch (error) {
      throw new MiningAccountError(`Failed to get user mining data: ${error.message}`);
    }
  }

  /**
   * Get all mining accounts with optional filters
   * @param filters - Optional filters for querying accounts
   * @returns Array of mining accounts
   */
  async getAllMiningAccounts(filters?: {
    isActive?: boolean;
    minTotalMined?: BN;
    maxTotalMined?: BN;
    phase?: MiningPhase;
  }): Promise<{ publicKey: PublicKey; account: MiningAccount }[]> {
    try {
      const programAccountsFilters: GetProgramAccountsFilter[] = [
        {
          dataSize: MINING_ACCOUNT_SIZE,
        } as DataSizeFilter,
      ];

      // Add specific filters if provided
      if (filters?.isActive !== undefined) {
        programAccountsFilters.push({
          memcmp: {
            offset: 40, // Adjust offset based on account structure
            bytes: filters.isActive ? '01' : '00',
          },
        } as MemcmpFilter);
      }

      const accounts = await this.connection.getProgramAccounts(
        FINOVA_PROGRAM_ID,
        {
          filters: programAccountsFilters,
          commitment: this.commitment,
        }
      );

      const miningAccounts = accounts
        .map(({ pubkey, account }) => ({
          publicKey: pubkey,
          account: this.program.coder.accounts.decode('miningAccount', account.data) as MiningAccount,
        }))
        .filter(({ account }) => {
          // Apply additional filters
          if (filters?.minTotalMined && account.totalMined.lt(filters.minTotalMined)) {
            return false;
          }
          if (filters?.maxTotalMined && account.totalMined.gt(filters.maxTotalMined)) {
            return false;
          }
          if (filters?.phase !== undefined && account.phase !== filters.phase) {
            return false;
          }
          return true;
        });

      return miningAccounts;
    } catch (error) {
      throw new MiningAccountError(`Failed to get all mining accounts: ${error.message}`);
    }
  }

  /**
   * Get mining leaderboard
   * @param limit - Number of top miners to return
   * @returns Array of top miners sorted by total mined
   */
  async getMiningLeaderboard(limit: number = 100): Promise<{
    publicKey: PublicKey;
    account: MiningAccount;
    rank: number;
    totalMined: number;
    effectiveRate: number;
  }[]> {
    try {
      const allAccounts = await this.getAllMiningAccounts({ isActive: true });
      
      const sortedAccounts = allAccounts
        .sort((a, b) => b.account.totalMined.cmp(a.account.totalMined))
        .slice(0, limit)
        .map((item, index) => ({
          ...item,
          rank: index + 1,
          totalMined: item.account.totalMined.toNumber() / 1e9, // Convert to FIN units
          effectiveRate: item.account.baseRate.toNumber() / 1e9, // Convert to FIN/hour
        }));

      return sortedAccounts;
    } catch (error) {
      throw new MiningAccountError(`Failed to get mining leaderboard: ${error.message}`);
    }
  }

  /**
   * Get mining statistics for the network
   * @returns Network-wide mining statistics
   */
  async getMiningStatistics(): Promise<{
    totalUsers: number;
    activeMiners: number;
    totalMined: BN;
    averageRate: number;
    currentPhase: MiningPhase;
    dailyVolume: BN;
    topMinerRate: number;
  }> {
    try {
      const [networkAccount, allAccounts] = await Promise.all([
        this.getNetworkAccount(),
        this.getAllMiningAccounts(),
      ]);

      const activeAccounts = allAccounts.filter(({ account }) => account.isActive);
      const totalMined = allAccounts.reduce((sum, { account }) => sum.add(account.totalMined), new BN(0));
      const averageRate = activeAccounts.length > 0 
        ? activeAccounts.reduce((sum, { account }) => sum + account.baseRate.toNumber(), 0) / activeAccounts.length
        : 0;
      
      const topMinerRate = activeAccounts.length > 0
        ? Math.max(...activeAccounts.map(({ account }) => account.baseRate.toNumber()))
        : 0;

      const currentPhase = this.getCurrentMiningPhase(networkAccount);
      
      // Calculate daily volume (last 24 hours of mining)
      const oneDayAgo = new BN(Date.now() / 1000 - 24 * 60 * 60);
      const recentMiners = activeAccounts.filter(({ account }) => 
        account.lastMined.gt(oneDayAgo)
      );
      const dailyVolume = recentMiners.reduce((sum, { account }) => 
        sum.add(account.baseRate.muln(24)), new BN(0)
      );

      return {
        totalUsers: allAccounts.length,
        activeMiners: activeAccounts.length,
        totalMined,
        averageRate: averageRate / 1e9, // Convert to FIN/hour
        currentPhase,
        dailyVolume,
        topMinerRate: topMinerRate / 1e9, // Convert to FIN/hour
      };
    } catch (error) {
      throw new MiningAccountError(`Failed to get mining statistics: ${error.message}`);
    }
  }

  /**
   * Calculate mining bonuses for a user
   * @param userAccount - User account data
   * @param miningAccount - Mining account data
   * @param networkAccount - Network account data
   * @returns Mining bonuses breakdown
   */
  private calculateMiningBonuses(
    userAccount: UserAccount,
    miningAccount: MiningAccount,
    networkAccount?: NetworkAccount
  ): MiningBonuses {
    const now = new BN(Date.now() / 1000);
    
    // Pioneer bonus (Finizen bonus)
    const totalUsers = networkAccount?.totalUsers.toNumber() || 0;
    const pioneerBonus = Math.max(1.0, 2.0 - (totalUsers / 1_000_000));

    // Security bonus (KYC verification)
    const securityBonus = userAccount.isKycVerified ? 1.2 : 0.8;

    // XP level bonus
    const xpBonus = 1.0 + (userAccount.xpLevel / 100);

    // RP tier bonus
    const rpBonus = 1.0 + (userAccount.rpTier * 0.2);

    // Staking bonus
    const stakingBonus = userAccount.stakedAmount.gt(new BN(0)) 
      ? 1.0 + (userAccount.stakedAmount.toNumber() / 10000) // 0.01% per FIN staked
      : 1.0;

    // Card boosts (sum of active card multipliers)
    const cardBoosts = miningAccount.cardBoosts
      .filter(boost => boost.isActive && boost.startTime.add(boost.duration).gt(now))
      .reduce((sum, boost) => sum + boost.multiplier.toNumber() / 1e9, 0);

    // Quality multiplier from AI analysis
    const qualityMultiplier = miningAccount.qualityScore.toNumber() / 1e9;

    // Streak bonus
    const streakBonus = 1.0 + (miningAccount.currentStreak * 0.05); // 5% per day streak

    // Referral network bonus
    const referralBonus = 1.0 + (userAccount.activeReferrals * 0.1);

    return {
      pioneerBonus,
      referralBonus,
      securityBonus,
      xpBonus,
      rpBonus,
      stakingBonus,
      cardBoosts,
      qualityMultiplier,
      streakBonus,
    };
  }

  /**
   * Calculate mining penalties for a user
   * @param userAccount - User account data
   * @param miningAccount - Mining account data
   * @param networkAccount - Network account data
   * @returns Mining penalties breakdown
   */
  private calculateMiningPenalties(
    userAccount: UserAccount,
    miningAccount: MiningAccount,
    networkAccount?: NetworkAccount
  ): MiningPenalties {
    // Exponential regression factor (anti-whale mechanism)
    const totalHoldings = userAccount.totalFin.toNumber();
    const regressionFactor = Math.exp(-0.001 * totalHoldings);

    // Whale decay for large holders
    const whaleDecay = totalHoldings > 100000 
      ? Math.exp(-0.0001 * (totalHoldings - 100000))
      : 1.0;

    // Inactivity penalty
    const now = Date.now() / 1000;
    const lastActivity = miningAccount.lastActivityTime.toNumber();
    const daysSinceActivity = (now - lastActivity) / (24 * 60 * 60);
    const inactivityPenalty = daysSinceActivity > 7 
      ? Math.max(0.5, 1.0 - (daysSinceActivity - 7) * 0.1)
      : 1.0;

    // Quality reduction for low-quality content
    const qualityReduction = userAccount.suspiciousScore > 0.5 
      ? Math.max(0.1, 1.0 - userAccount.suspiciousScore)
      : 1.0;

    return {
      regressionFactor,
      whaleDecay,
      inactivityPenalty,
      qualityReduction,
    };
  }

  /**
   * Calculate effective mining rate considering all bonuses and penalties
   * @param miningAccount - Mining account data
   * @param bonuses - Mining bonuses
   * @param penalties - Mining penalties
   * @returns Effective mining rate per hour
   */
  private calculateEffectiveMiningRate(
    miningAccount: MiningAccount,
    bonuses: MiningBonuses,
    penalties: MiningPenalties
  ): number {
    const baseRate = miningAccount.baseRate.toNumber() / 1e9; // Convert to FIN

    const totalBonus = bonuses.pioneerBonus * 
                      bonuses.referralBonus * 
                      bonuses.securityBonus * 
                      bonuses.xpBonus * 
                      bonuses.rpBonus * 
                      bonuses.stakingBonus * 
                      (1 + bonuses.cardBoosts) * 
                      bonuses.qualityMultiplier * 
                      bonuses.streakBonus;

    const totalPenalty = penalties.regressionFactor * 
                        penalties.whaleDecay * 
                        penalties.inactivityPenalty * 
                        penalties.qualityReduction;

    const effectiveRate = baseRate * totalBonus * totalPenalty;

    // Clamp to min/max rates
    return Math.max(MIN_MINING_RATE, Math.min(MAX_MINING_RATE, effectiveRate));
  }

  /**
   * Calculate daily mining limit for a user
   * @param userAccount - User account data
   * @param miningAccount - Mining account data
   * @returns Daily mining limit in FIN
   */
  private calculateDailyLimit(userAccount: UserAccount, miningAccount: MiningAccount): number {
    const baseLimit = 10.0; // Base daily limit in FIN
    const levelMultiplier = 1.0 + (userAccount.xpLevel * 0.02); // 2% increase per level
    const tierMultiplier = 1.0 + (userAccount.rpTier * 0.1); // 10% increase per RP tier
    
    return baseLimit * levelMultiplier * tierMultiplier;
  }

  /**
   * Get current mining phase based on network data
   * @param networkAccount - Network account data
   * @returns Current mining phase
   */
  private getCurrentMiningPhase(networkAccount?: NetworkAccount): MiningPhase {
    if (!networkAccount) {
      return MiningPhase.Finizen;
    }

    const totalUsers = networkAccount.totalUsers.toNumber();
    
    if (totalUsers < 100_000) {
      return MiningPhase.Finizen;
    } else if (totalUsers < 1_000_000) {
      return MiningPhase.Growth;
    } else if (totalUsers < 10_000_000) {
      return MiningPhase.Maturity;
    } else {
      return MiningPhase.Stability;
    }
  }

  /**
   * Calculate projected earnings based on current rate
   * @param hourlyRate - Current hourly mining rate
   * @param dailyLimit - Daily mining limit
   * @returns Projected earnings breakdown
   */
  private calculateProjectedEarnings(hourlyRate: number, dailyLimit: number): ProjectedEarnings {
    const dailyEstimate = Math.min(hourlyRate * 24, dailyLimit);
    const weeklyEstimate = dailyEstimate * 7;
    const monthlyEstimate = dailyEstimate * 30;

    return {
      hourlyRate,
      dailyEstimate,
      weeklyEstimate,
      monthlyEstimate,
    };
  }

  /**
   * Find mining account PDA for a user
   * @param userPublicKey - User's public key
   * @returns Mining account PDA and bump
   */
  async findMiningAccountPDA(userPublicKey: PublicKey): Promise<[PublicKey, number]> {
    return PublicKey.findProgramAddressSync(
      [
        Buffer.from('mining'),
        userPublicKey.toBuffer(),
      ],
      FINOVA_PROGRAM_ID
    );
  }

  /**
   * Find user account PDA
   * @param userPublicKey - User's public key
   * @returns User account PDA and bump
   */
  async findUserAccountPDA(userPublicKey: PublicKey): Promise<[PublicKey, number]> {
    return PublicKey.findProgramAddressSync(
      [
        Buffer.from('user'),
        userPublicKey.toBuffer(),
      ],
      FINOVA_PROGRAM_ID
    );
  }

  /**
   * Find network account PDA
   * @returns Network account PDA and bump
   */
  async findNetworkAccountPDA(): Promise<[PublicKey, number]> {
    return PublicKey.findProgramAddressSync(
      [Buffer.from('network')],
      FINOVA_PROGRAM_ID
    );
  }

  /**
   * Subscribe to mining account changes
   * @param userPublicKey - User's public key
   * @param callback - Callback function for account updates
   * @returns Subscription ID
   */
  async subscribeMiningAccount(
    userPublicKey: PublicKey,
    callback: (accountInfo: AccountInfo<Buffer> | null, context: RpcResponseAndContext<AccountInfo<Buffer> | null>) => void
  ): Promise<number> {
    try {
      const [miningPDA] = await this.findMiningAccountPDA(userPublicKey);
      
      return this.connection.onAccountChange(
        miningPDA,
        callback,
        this.commitment
      );
    } catch (error) {
      throw new MiningAccountError(`Failed to subscribe to mining account: ${error.message}`);
    }
  }

  /**
   * Unsubscribe from account changes
   * @param subscriptionId - Subscription ID to remove
   */
  async unsubscribeAccount(subscriptionId: number): Promise<void> {
    try {
      await this.connection.removeAccountChangeListener(subscriptionId);
    } catch (error) {
      throw new MiningAccountError(`Failed to unsubscribe from account: ${error.message}`);
    }
  }

  /**
   * Validate mining account data integrity
   * @param miningAccount - Mining account to validate
   * @returns Validation result
   */
  validateMiningAccount(miningAccount: MiningAccount): {
    isValid: boolean;
    errors: string[];
  } {
    const errors: string[] = [];

    // Check basic constraints
    if (miningAccount.baseRate.isNeg()) {
      errors.push('Base rate cannot be negative');
    }

    if (miningAccount.totalMined.isNeg()) {
      errors.push('Total mined cannot be negative');
    }

    if (miningAccount.currentStreak < 0) {
      errors.push('Current streak cannot be negative');
    }

    if (miningAccount.bonusMultiplier.isNeg()) {
      errors.push('Bonus multiplier cannot be negative');
    }

    // Check rate limits
    const rateInFin = miningAccount.baseRate.toNumber() / 1e9;
    if (rateInFin > MAX_MINING_RATE) {
      errors.push(`Mining rate ${rateInFin} exceeds maximum ${MAX_MINING_RATE}`);
    }

    if (rateInFin < MIN_MINING_RATE && miningAccount.isActive) {
      errors.push(`Active mining rate ${rateInFin} below minimum ${MIN_MINING_RATE}`);
    }

    // Validate card boosts
    const now = new BN(Date.now() / 1000);
    miningAccount.cardBoosts.forEach((boost, index) => {
      if (boost.isActive && boost.startTime.add(boost.duration).lt(now)) {
        errors.push(`Card boost ${index} is marked active but has expired`);
      }
      
      if (boost.multiplier.isNeg()) {
        errors.push(`Card boost ${index} has negative multiplier`);
      }
    });

    return {
      isValid: errors.length === 0,
      errors,
    };
  }

  /**
   * Get mining history for a user (requires additional indexing service)
   * @param userPublicKey - User's public key
   * @param limit - Number of records to return
   * @returns Mining history array
   */
  async getMiningHistory(userPublicKey: PublicKey, limit: number = 100): Promise<{
    timestamp: number;
    amount: number;
    rate: number;
    bonuses: MiningBonuses;
    txSignature?: string;
  }[]> {
    // This would typically require an indexing service or transaction history API
    // For now, return empty array as placeholder
    console.warn('getMiningHistory requires indexing service implementation');
    return [];
  }

  /**
   * Estimate gas costs for mining operations
   * @returns Estimated costs in SOL
   */
  async estimateGasCosts(): Promise<{
    initializeMining: number;
    claimRewards: number;
    updateBoosts: number;
    stake: number;
  }> {
    try {
      const recentBlockhash = await this.connection.getLatestBlockhash();
      const rentExemption = await this.connection.getMinimumBalanceForRentExemption(MINING_ACCOUNT_SIZE);
      
      // These are estimates based on typical instruction costs
      return {
        initializeMining: (rentExemption + 5000) / 1e9, // Rent + transaction fee
        claimRewards: 5000 / 1e9, // Basic transaction fee
        updateBoosts: 7500 / 1e9, // Slightly higher for complex operations
        stake: 10000 / 1e9, // Token transfer + account updates
      };
    } catch (error) {
      throw new MiningAccountError(`Failed to estimate gas costs: ${error.message}`);
    }
  }
}
