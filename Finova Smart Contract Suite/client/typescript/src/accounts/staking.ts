// finova-net/finova/client/typescript/src/accounts/staking.ts

/**
 * Finova Network - Staking Accounts Module
 * 
 * Enterprise-grade TypeScript client for managing staking-related accounts
 * Supports liquid staking, enhanced rewards, and multi-tier staking system
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
  MemcmpFilter
} from '@solana/web3.js';
import { BN } from '@project-serum/anchor';
import { TOKEN_PROGRAM_ID, ASSOCIATED_TOKEN_PROGRAM_ID } from '@solana/spl-token';

// Internal imports
import { FinovaClient } from '../client';
import { StakeAccount, StakePool, RewardPool, StakingTier } from '../types';
import { STAKING_PROGRAM_ID, FIN_TOKEN_MINT, SFIN_TOKEN_MINT } from '../constants';

/**
 * Staking-related account sizes for filtering
 */
export const ACCOUNT_SIZES = {
  STAKE_ACCOUNT: 264,      // User stake account
  STAKE_POOL: 520,         // Global stake pool
  REWARD_POOL: 200,        // Reward distribution pool
  STAKING_CONFIG: 128      // Staking configuration
} as const;

/**
 * Staking account discriminators (first 8 bytes)
 */
export const ACCOUNT_DISCRIMINATORS = {
  STAKE_ACCOUNT: Buffer.from([0x9A, 0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC, 0xDE]),
  STAKE_POOL: Buffer.from([0x1A, 0x23, 0x45, 0x67, 0x89, 0xAB, 0xCD, 0xEF]),
  REWARD_POOL: Buffer.from([0x2B, 0x34, 0x56, 0x78, 0x9A, 0xBC, 0xDE, 0xF0]),
  STAKING_CONFIG: Buffer.from([0x3C, 0x45, 0x67, 0x89, 0xAB, 0xCD, 0xEF, 0x01])
} as const;

/**
 * Staking tiers with their respective benefits
 */
export const STAKING_TIERS = {
  TIER_1: { min: 100, max: 499, name: 'Bronze', miningBoost: 1.2, xpMultiplier: 1.1, rpBonus: 1.05 },
  TIER_2: { min: 500, max: 999, name: 'Silver', miningBoost: 1.35, xpMultiplier: 1.2, rpBonus: 1.1 },
  TIER_3: { min: 1000, max: 4999, name: 'Gold', miningBoost: 1.5, xpMultiplier: 1.3, rpBonus: 1.2 },
  TIER_4: { min: 5000, max: 9999, name: 'Platinum', miningBoost: 1.75, xpMultiplier: 1.5, rpBonus: 1.35 },
  TIER_5: { min: 10000, max: Number.MAX_SAFE_INTEGER, name: 'Diamond', miningBoost: 2.0, xpMultiplier: 1.75, rpBonus: 1.5 }
} as const;

/**
 * Interface for decoded stake account data
 */
export interface DecodedStakeAccount {
  discriminator: Buffer;
  owner: PublicKey;
  stakePool: PublicKey;
  stakedAmount: BN;
  sFinAmount: BN;
  stakingTier: number;
  stakeTimestamp: BN;
  lastRewardClaim: BN;
  loyaltyBonus: number;
  activityMultiplier: number;
  totalRewardsEarned: BN;
  pendingRewards: BN;
  isActive: boolean;
  lockupPeriod: BN;
  earlyWithdrawalPenalty: number;
  xpLevelBonus: number;
  rpTierBonus: number;
  qualityScore: number;
  networkEffectMultiplier: number;
}

/**
 * Interface for decoded stake pool data
 */
export interface DecodedStakePool {
  discriminator: Buffer;
  authority: PublicKey;
  finTokenMint: PublicKey;
  sFinTokenMint: PublicKey;
  finTokenVault: PublicKey;
  sFinTokenVault: PublicKey;
  totalStaked: BN;
  totalSFinSupply: BN;
  exchangeRate: BN; // sFIN to FIN ratio (scaled by 1e9)
  baseApy: number;
  rewardRate: BN;
  lastUpdateSlot: BN;
  emergencyPause: boolean;
  minimumStake: BN;
  maximumStake: BN;
  stakingFee: number; // basis points
  unstakingFee: number; // basis points
  lockupPeriods: number[]; // Available lockup periods in days
  tierBonuses: number[]; // Multipliers for each tier
  qualityThresholds: number[]; // Quality score requirements
}

/**
 * Interface for decoded reward pool data
 */
export interface DecodedRewardPool {
  discriminator: Buffer;
  authority: PublicKey;
  rewardTokenMint: PublicKey;
  rewardTokenVault: PublicKey;
  totalRewards: BN;
  distributedRewards: BN;
  rewardRate: BN; // Rewards per second
  lastDistribution: BN;
  distributionPeriod: BN;
  stakingPoolAddress: PublicKey;
  isActive: boolean;
}

/**
 * Staking accounts management class
 */
export class StakingAccounts {
  private client: FinovaClient;
  private connection: Connection;

  constructor(client: FinovaClient) {
    this.client = client;
    this.connection = client.connection;
  }

  /**
   * Get stake account address for a user
   */
  getStakeAccountAddress(userPublicKey: PublicKey, stakePool?: PublicKey): PublicKey {
    const pool = stakePool || this.getDefaultStakePoolAddress();
    
    const [stakeAccountPda] = PublicKey.findProgramAddressSync(
      [
        Buffer.from('stake_account'),
        userPublicKey.toBuffer(),
        pool.toBuffer()
      ],
      STAKING_PROGRAM_ID
    );

    return stakeAccountPda;
  }

  /**
   * Get default stake pool address
   */
  getDefaultStakePoolAddress(): PublicKey {
    const [stakePoolPda] = PublicKey.findProgramAddressSync(
      [Buffer.from('stake_pool'), FIN_TOKEN_MINT.toBuffer()],
      STAKING_PROGRAM_ID
    );

    return stakePoolPda;
  }

  /**
   * Get reward pool address for a stake pool
   */
  getRewardPoolAddress(stakePool?: PublicKey): PublicKey {
    const pool = stakePool || this.getDefaultStakePoolAddress();
    
    const [rewardPoolPda] = PublicKey.findProgramAddressSync(
      [Buffer.from('reward_pool'), pool.toBuffer()],
      STAKING_PROGRAM_ID
    );

    return rewardPoolPda;
  }

  /**
   * Fetch and decode stake account data
   */
  async getStakeAccount(
    userPublicKey: PublicKey, 
    stakePool?: PublicKey
  ): Promise<DecodedStakeAccount | null> {
    try {
      const stakeAccountAddress = this.getStakeAccountAddress(userPublicKey, stakePool);
      const accountInfo = await this.connection.getAccountInfo(stakeAccountAddress);

      if (!accountInfo || !accountInfo.data) {
        return null;
      }

      return this.decodeStakeAccount(accountInfo.data);
    } catch (error) {
      console.error('Error fetching stake account:', error);
      throw new Error(`Failed to fetch stake account: ${error.message}`);
    }
  }

  /**
   * Fetch and decode stake pool data
   */
  async getStakePool(stakePoolAddress?: PublicKey): Promise<DecodedStakePool | null> {
    try {
      const poolAddress = stakePoolAddress || this.getDefaultStakePoolAddress();
      const accountInfo = await this.connection.getAccountInfo(poolAddress);

      if (!accountInfo || !accountInfo.data) {
        return null;
      }

      return this.decodeStakePool(accountInfo.data);
    } catch (error) {
      console.error('Error fetching stake pool:', error);
      throw new Error(`Failed to fetch stake pool: ${error.message}`);
    }
  }

  /**
   * Fetch and decode reward pool data
   */
  async getRewardPool(stakePool?: PublicKey): Promise<DecodedRewardPool | null> {
    try {
      const rewardPoolAddress = this.getRewardPoolAddress(stakePool);
      const accountInfo = await this.connection.getAccountInfo(rewardPoolAddress);

      if (!accountInfo || !accountInfo.data) {
        return null;
      }

      return this.decodeRewardPool(accountInfo.data);
    } catch (error) {
      console.error('Error fetching reward pool:', error);
      throw new Error(`Failed to fetch reward pool: ${error.message}`);
    }
  }

  /**
   * Get all stake accounts for a user across different pools
   */
  async getAllUserStakeAccounts(userPublicKey: PublicKey): Promise<DecodedStakeAccount[]> {
    try {
      const filters: GetProgramAccountsFilter[] = [
        {
          dataSize: ACCOUNT_SIZES.STAKE_ACCOUNT
        } as DataSizeFilter,
        {
          memcmp: {
            offset: 8, // Skip discriminator
            bytes: userPublicKey.toBase58()
          }
        } as MemcmpFilter
      ];

      const accounts = await this.connection.getProgramAccounts(
        STAKING_PROGRAM_ID,
        { filters }
      );

      return accounts
        .map(account => this.decodeStakeAccount(account.account.data))
        .filter(account => account !== null) as DecodedStakeAccount[];
    } catch (error) {
      console.error('Error fetching user stake accounts:', error);
      throw new Error(`Failed to fetch user stake accounts: ${error.message}`);
    }
  }

  /**
   * Get all active stake pools
   */
  async getAllStakePools(): Promise<Array<{ address: PublicKey; data: DecodedStakePool }>> {
    try {
      const filters: GetProgramAccountsFilter[] = [
        {
          dataSize: ACCOUNT_SIZES.STAKE_POOL
        } as DataSizeFilter,
        {
          memcmp: {
            offset: 0,
            bytes: ACCOUNT_DISCRIMINATORS.STAKE_POOL.toString('base64')
          }
        } as MemcmpFilter
      ];

      const accounts = await this.connection.getProgramAccounts(
        STAKING_PROGRAM_ID,
        { filters }
      );

      return accounts
        .map(account => ({
          address: account.pubkey,
          data: this.decodeStakePool(account.account.data)
        }))
        .filter(pool => pool.data !== null) as Array<{ address: PublicKey; data: DecodedStakePool }>;
    } catch (error) {
      console.error('Error fetching stake pools:', error);
      throw new Error(`Failed to fetch stake pools: ${error.message}`);
    }
  }

  /**
   * Calculate staking rewards for a user
   */
  async calculatePendingRewards(
    userPublicKey: PublicKey,
    stakePool?: PublicKey
  ): Promise<{
    baseRewards: BN;
    loyaltyBonus: BN;
    activityBonus: BN;
    tierBonus: BN;
    totalRewards: BN;
    projectedApy: number;
  }> {
    try {
      const stakeAccount = await this.getStakeAccount(userPublicKey, stakePool);
      const pool = await this.getStakePool(stakePool);

      if (!stakeAccount || !pool) {
        throw new Error('Stake account or pool not found');
      }

      const currentSlot = await this.connection.getSlot();
      const timeSinceLastClaim = new BN(currentSlot).sub(new BN(stakeAccount.lastRewardClaim));
      
      // Base rewards calculation
      const baseRewards = stakeAccount.stakedAmount
        .mul(pool.rewardRate)
        .mul(timeSinceLastClaim)
        .div(new BN(1000000)); // Scale factor

      // Loyalty bonus (increases over time)
      const stakingDuration = new BN(currentSlot).sub(stakeAccount.stakeTimestamp);
      const loyaltyMultiplier = Math.min(1.5, 1 + stakingDuration.toNumber() / 31536000); // Max 1.5x after 1 year
      const loyaltyBonus = baseRewards.mul(new BN(Math.floor((loyaltyMultiplier - 1) * 1000))).div(new BN(1000));

      // Activity bonus based on XP and RP
      const activityMultiplier = stakeAccount.activityMultiplier;
      const activityBonus = baseRewards.mul(new BN(Math.floor((activityMultiplier - 1) * 1000))).div(new BN(1000));

      // Tier bonus
      const tierMultiplier = STAKING_TIERS[`TIER_${stakeAccount.stakingTier}`]?.miningBoost || 1.0;
      const tierBonus = baseRewards.mul(new BN(Math.floor((tierMultiplier - 1) * 1000))).div(new BN(1000));

      const totalRewards = baseRewards.add(loyaltyBonus).add(activityBonus).add(tierBonus);

      // Calculate projected APY
      const yearlyRewards = totalRewards.mul(new BN(31536000)).div(timeSinceLastClaim);
      const projectedApy = yearlyRewards.mul(new BN(100)).div(stakeAccount.stakedAmount).toNumber();

      return {
        baseRewards,
        loyaltyBonus,
        activityBonus,
        tierBonus,
        totalRewards,
        projectedApy
      };
    } catch (error) {
      console.error('Error calculating pending rewards:', error);
      throw new Error(`Failed to calculate pending rewards: ${error.message}`);
    }
  }

  /**
   * Get staking tier for an amount
   */
  getStakingTier(amount: number): { tier: number; name: string; benefits: any } {
    for (const [key, tier] of Object.entries(STAKING_TIERS)) {
      if (amount >= tier.min && amount <= tier.max) {
        return {
          tier: parseInt(key.replace('TIER_', '')),
          name: tier.name,
          benefits: {
            miningBoost: tier.miningBoost,
            xpMultiplier: tier.xpMultiplier,
            rpBonus: tier.rpBonus
          }
        };
      }
    }
    return {
      tier: 1,
      name: 'Bronze',
      benefits: STAKING_TIERS.TIER_1
    };
  }

  /**
   * Calculate sFIN to FIN exchange rate
   */
  async getExchangeRate(stakePool?: PublicKey): Promise<number> {
    try {
      const pool = await this.getStakePool(stakePool);
      if (!pool) {
        throw new Error('Stake pool not found');
      }

      // Exchange rate is stored as scaled by 1e9
      return pool.exchangeRate.toNumber() / 1e9;
    } catch (error) {
      console.error('Error getting exchange rate:', error);
      throw new Error(`Failed to get exchange rate: ${error.message}`);
    }
  }

  /**
   * Get staking statistics
   */
  async getStakingStats(): Promise<{
    totalStaked: BN;
    totalStakers: number;
    averageStake: BN;
    totalRewardsDistributed: BN;
    currentApy: number;
    tierDistribution: Record<string, number>;
  }> {
    try {
      const pools = await this.getAllStakePools();
      const allStakeAccounts = [];

      // Fetch all stake accounts
      const filters: GetProgramAccountsFilter[] = [
        {
          dataSize: ACCOUNT_SIZES.STAKE_ACCOUNT
        } as DataSizeFilter
      ];

      const accounts = await this.connection.getProgramAccounts(
        STAKING_PROGRAM_ID,
        { filters }
      );

      for (const account of accounts) {
        const decoded = this.decodeStakeAccount(account.account.data);
        if (decoded && decoded.isActive) {
          allStakeAccounts.push(decoded);
        }
      }

      // Calculate statistics
      const totalStaked = allStakeAccounts.reduce((sum, account) => sum.add(account.stakedAmount), new BN(0));
      const totalStakers = allStakeAccounts.length;
      const averageStake = totalStakers > 0 ? totalStaked.div(new BN(totalStakers)) : new BN(0);
      const totalRewardsDistributed = allStakeAccounts.reduce((sum, account) => sum.add(account.totalRewardsEarned), new BN(0));

      // Calculate weighted average APY
      let totalWeightedApy = 0;
      let totalWeight = 0;
      for (const pool of pools) {
        const weight = pool.data.totalStaked.toNumber();
        totalWeightedApy += pool.data.baseApy * weight;
        totalWeight += weight;
      }
      const currentApy = totalWeight > 0 ? totalWeightedApy / totalWeight : 0;

      // Calculate tier distribution
      const tierDistribution: Record<string, number> = {};
      for (let i = 1; i <= 5; i++) {
        tierDistribution[`tier_${i}`] = allStakeAccounts.filter(account => account.stakingTier === i).length;
      }

      return {
        totalStaked,
        totalStakers,
        averageStake,
        totalRewardsDistributed,
        currentApy,
        tierDistribution
      };
    } catch (error) {
      console.error('Error getting staking stats:', error);
      throw new Error(`Failed to get staking stats: ${error.message}`);
    }
  }

  /**
   * Decode stake account data from buffer
   */
  private decodeStakeAccount(data: Buffer): DecodedStakeAccount | null {
    try {
      if (data.length < ACCOUNT_SIZES.STAKE_ACCOUNT) {
        return null;
      }

      let offset = 0;

      const discriminator = data.slice(offset, offset + 8);
      offset += 8;

      const owner = new PublicKey(data.slice(offset, offset + 32));
      offset += 32;

      const stakePool = new PublicKey(data.slice(offset, offset + 32));
      offset += 32;

      const stakedAmount = new BN(data.slice(offset, offset + 8), 'le');
      offset += 8;

      const sFinAmount = new BN(data.slice(offset, offset + 8), 'le');
      offset += 8;

      const stakingTier = data.readUInt8(offset);
      offset += 1;

      const stakeTimestamp = new BN(data.slice(offset, offset + 8), 'le');
      offset += 8;

      const lastRewardClaim = new BN(data.slice(offset, offset + 8), 'le');
      offset += 8;

      const loyaltyBonus = data.readFloatLE(offset);
      offset += 4;

      const activityMultiplier = data.readFloatLE(offset);
      offset += 4;

      const totalRewardsEarned = new BN(data.slice(offset, offset + 8), 'le');
      offset += 8;

      const pendingRewards = new BN(data.slice(offset, offset + 8), 'le');
      offset += 8;

      const isActive = Boolean(data.readUInt8(offset));
      offset += 1;

      const lockupPeriod = new BN(data.slice(offset, offset + 8), 'le');
      offset += 8;

      const earlyWithdrawalPenalty = data.readFloatLE(offset);
      offset += 4;

      const xpLevelBonus = data.readFloatLE(offset);
      offset += 4;

      const rpTierBonus = data.readFloatLE(offset);
      offset += 4;

      const qualityScore = data.readFloatLE(offset);
      offset += 4;

      const networkEffectMultiplier = data.readFloatLE(offset);

      return {
        discriminator,
        owner,
        stakePool,
        stakedAmount,
        sFinAmount,
        stakingTier,
        stakeTimestamp,
        lastRewardClaim,
        loyaltyBonus,
        activityMultiplier,
        totalRewardsEarned,
        pendingRewards,
        isActive,
        lockupPeriod,
        earlyWithdrawalPenalty,
        xpLevelBonus,
        rpTierBonus,
        qualityScore,
        networkEffectMultiplier
      };
    } catch (error) {
      console.error('Error decoding stake account:', error);
      return null;
    }
  }

  /**
   * Decode stake pool data from buffer
   */
  private decodeStakePool(data: Buffer): DecodedStakePool | null {
    try {
      if (data.length < ACCOUNT_SIZES.STAKE_POOL) {
        return null;
      }

      let offset = 0;

      const discriminator = data.slice(offset, offset + 8);
      offset += 8;

      const authority = new PublicKey(data.slice(offset, offset + 32));
      offset += 32;

      const finTokenMint = new PublicKey(data.slice(offset, offset + 32));
      offset += 32;

      const sFinTokenMint = new PublicKey(data.slice(offset, offset + 32));
      offset += 32;

      const finTokenVault = new PublicKey(data.slice(offset, offset + 32));
      offset += 32;

      const sFinTokenVault = new PublicKey(data.slice(offset, offset + 32));
      offset += 32;

      const totalStaked = new BN(data.slice(offset, offset + 8), 'le');
      offset += 8;

      const totalSFinSupply = new BN(data.slice(offset, offset + 8), 'le');
      offset += 8;

      const exchangeRate = new BN(data.slice(offset, offset + 8), 'le');
      offset += 8;

      const baseApy = data.readFloatLE(offset);
      offset += 4;

      const rewardRate = new BN(data.slice(offset, offset + 8), 'le');
      offset += 8;

      const lastUpdateSlot = new BN(data.slice(offset, offset + 8), 'le');
      offset += 8;

      const emergencyPause = Boolean(data.readUInt8(offset));
      offset += 1;

      const minimumStake = new BN(data.slice(offset, offset + 8), 'le');
      offset += 8;

      const maximumStake = new BN(data.slice(offset, offset + 8), 'le');
      offset += 8;

      const stakingFee = data.readUInt16LE(offset);
      offset += 2;

      const unstakingFee = data.readUInt16LE(offset);
      offset += 2;

      // Read arrays
      const lockupPeriodsLength = data.readUInt8(offset);
      offset += 1;
      const lockupPeriods: number[] = [];
      for (let i = 0; i < lockupPeriodsLength; i++) {
        lockupPeriods.push(data.readUInt32LE(offset));
        offset += 4;
      }

      const tierBonusesLength = data.readUInt8(offset);
      offset += 1;
      const tierBonuses: number[] = [];
      for (let i = 0; i < tierBonusesLength; i++) {
        tierBonuses.push(data.readFloatLE(offset));
        offset += 4;
      }

      const qualityThresholdsLength = data.readUInt8(offset);
      offset += 1;
      const qualityThresholds: number[] = [];
      for (let i = 0; i < qualityThresholdsLength; i++) {
        qualityThresholds.push(data.readFloatLE(offset));
        offset += 4;
      }

      return {
        discriminator,
        authority,
        finTokenMint,
        sFinTokenMint,
        finTokenVault,
        sFinTokenVault,
        totalStaked,
        totalSFinSupply,
        exchangeRate,
        baseApy,
        rewardRate,
        lastUpdateSlot,
        emergencyPause,
        minimumStake,
        maximumStake,
        stakingFee,
        unstakingFee,
        lockupPeriods,
        tierBonuses,
        qualityThresholds
      };
    } catch (error) {
      console.error('Error decoding stake pool:', error);
      return null;
    }
  }

  /**
   * Decode reward pool data from buffer
   */
  private decodeRewardPool(data: Buffer): DecodedRewardPool | null {
    try {
      if (data.length < ACCOUNT_SIZES.REWARD_POOL) {
        return null;
      }

      let offset = 0;

      const discriminator = data.slice(offset, offset + 8);
      offset += 8;

      const authority = new PublicKey(data.slice(offset, offset + 32));
      offset += 32;

      const rewardTokenMint = new PublicKey(data.slice(offset, offset + 32));
      offset += 32;

      const rewardTokenVault = new PublicKey(data.slice(offset, offset + 32));
      offset += 32;

      const totalRewards = new BN(data.slice(offset, offset + 8), 'le');
      offset += 8;

      const distributedRewards = new BN(data.slice(offset, offset + 8), 'le');
      offset += 8;

      const rewardRate = new BN(data.slice(offset, offset + 8), 'le');
      offset += 8;

      const lastDistribution = new BN(data.slice(offset, offset + 8), 'le');
      offset += 8;

      const distributionPeriod = new BN(data.slice(offset, offset + 8), 'le');
      offset += 8;

      const stakingPoolAddress = new PublicKey(data.slice(offset, offset + 32));
      offset += 32;

      const isActive = Boolean(data.readUInt8(offset));

      return {
        discriminator,
        authority,
        rewardTokenMint,
        rewardTokenVault,
        totalRewards,
        distributedRewards,
        rewardRate,
        lastDistribution,
        distributionPeriod,
        stakingPoolAddress,
        isActive
      };
    } catch (error) {
      console.error('Error decoding reward pool:', error);
      return null;
    }
  }

  /**
   * Subscribe to stake account changes
   */
  subscribeToStakeAccount(
    userPublicKey: PublicKey,
    callback: (account: DecodedStakeAccount | null) => void,
    stakePool?: PublicKey
  ): number {
    const stakeAccountAddress = this.getStakeAccountAddress(userPublicKey, stakePool);
    
    return this.connection.onAccountChange(
      stakeAccountAddress,
      (accountInfo) => {
        if (accountInfo.data) {
          const decoded = this.decodeStakeAccount(accountInfo.data);
          callback(decoded);
        } else {
          callback(null);
        }
      },
      'confirmed'
    );
  }

  /**
   * Subscribe to stake pool changes
   */
  subscribeToStakePool(
    callback: (pool: DecodedStakePool | null) => void,
    stakePool?: PublicKey
  ): number {
    const poolAddress = stakePool || this.getDefaultStakePoolAddress();
    
    return this.connection.onAccountChange(
      poolAddress,
      (accountInfo) => {
        if (accountInfo.data) {
          const decoded = this.decodeStakePool(accountInfo.data);
          callback(decoded);
        } else {
          callback(null);
        }
      },
      'confirmed'
    );
  }

  /**
   * Unsubscribe from account changes
   */
  async unsubscribe(subscriptionId: number): Promise<void> {
    await this.connection.removeAccountChangeListener(subscriptionId);
  }

  /**
   * Batch fetch multiple stake accounts
   */
  async batchGetStakeAccounts(
    userPublicKeys: PublicKey[],
    stakePool?: PublicKey
  ): Promise<(DecodedStakeAccount | null)[]> {
    try {
      const addresses = userPublicKeys.map(pubkey => 
        this.getStakeAccountAddress(pubkey, stakePool)
      );

      const accountInfos = await this.connection.getMultipleAccountsInfo(addresses);

      return accountInfos.map(accountInfo => {
        if (!accountInfo || !accountInfo.data) {
          return null;
        }
        return this.decodeStakeAccount(accountInfo.data);
      });
    } catch (error) {
      console.error('Error batch fetching stake accounts:', error);
      throw new Error(`Failed to batch fetch stake accounts: ${error.message}`);
    }
  }

  /**
   * Get historical staking data for analytics
   */
  async getHistoricalStakingData(
    userPublicKey: PublicKey,
    fromSlot?: number,
    toSlot?: number
  ): Promise<{
    stakingHistory: Array<{
      slot: number;
      timestamp: number;
      stakedAmount: BN;
      rewards: BN;
      tier: number;
      apy: number;
    }>;
    totalRewards: BN;
    averageApy: number;
    stakingDuration: number;
  }> {
    try {
      const stakeAccount = await this.getStakeAccount(userPublicKey);
      if (!stakeAccount) {
        throw new Error('Stake account not found');
      }

      const currentSlot = await this.connection.getSlot();
      const startSlot = fromSlot || stakeAccount.stakeTimestamp.toNumber();
      const endSlot = toSlot || currentSlot;

      // This would typically query indexed historical data
      // For now, we'll simulate with current data
      const stakingHistory = [{
        slot: currentSlot,
        timestamp: Date.now(),
        stakedAmount: stakeAccount.stakedAmount,
        rewards: stakeAccount.totalRewardsEarned,
        tier: stakeAccount.stakingTier,
        apy: 12.5 // This would be calculated from historical data
      }];

      return {
        stakingHistory,
        totalRewards: stakeAccount.totalRewardsEarned,
        averageApy: 12.5,
        stakingDuration: currentSlot - stakeAccount.stakeTimestamp.toNumber()
      };
    } catch (error) {
      console.error('Error getting historical staking data:', error);
      throw new Error(`Failed to get historical staking data: ${error.message}`);
    }
  }

  /**
   * Get staking leaderboard
   */
  async getStakingLeaderboard(limit: number = 100): Promise<Array<{
    rank: number;
    owner: PublicKey;
    stakedAmount: BN;
    tier: number;
    totalRewards: BN;
    apy: number;
  }>> {
    try {
      const filters: GetProgramAccountsFilter[] = [
        {
          dataSize: ACCOUNT_SIZES.STAKE_ACCOUNT
        } as DataSizeFilter
      ];

      const accounts = await this.connection.getProgramAccounts(
        STAKING_PROGRAM_ID,
        { filters }
      );

      const stakeAccounts = accounts
        .map(account => this.decodeStakeAccount(account.account.data))
        .filter(account => account !== null && account.isActive) as DecodedStakeAccount[];

      // Sort by staked amount
      stakeAccounts.sort((a, b) => b.stakedAmount.cmp(a.stakedAmount));

      return stakeAccounts.slice(0, limit).map((account, index) => ({
        rank: index + 1,
        owner: account.owner,
        stakedAmount: account.stakedAmount,
        tier: account.stakingTier,
        totalRewards: account.totalRewardsEarned,
        apy: this.calculateApyFromAccount(account)
      }));
    } catch (error) {
      console.error('Error getting staking leaderboard:', error);
      throw new Error(`Failed to get staking leaderboard: ${error.message}`);
    }
  }

  /**
   * Validate stake account data integrity
   */
  async validateStakeAccount(userPublicKey: PublicKey): Promise<{
    isValid: boolean;
    issues: string[];
    recommendations: string[];
  }> {
    try {
      const stakeAccount = await this.getStakeAccount(userPublicKey);
      const stakePool = await this.getStakePool();

      const issues: string[] = [];
      const recommendations: string[] = [];

      if (!stakeAccount) {
        issues.push('Stake account not found');
        return { isValid: false, issues, recommendations };
      }

      if (!stakePool) {
        issues.push('Stake pool not found');
        return { isValid: false, issues, recommendations };
      }

      // Validate staked amount
      if (stakeAccount.stakedAmount.lt(stakePool.minimumStake)) {
        issues.push(`Staked amount below minimum: ${stakeAccount.stakedAmount.toString()} < ${stakePool.minimumStake.toString()}`);
        recommendations.push('Increase stake to meet minimum requirements');
      }

      if (stakeAccount.stakedAmount.gt(stakePool.maximumStake)) {
        issues.push(`Staked amount above maximum: ${stakeAccount.stakedAmount.toString()} > ${stakePool.maximumStake.toString()}`);
        recommendations.push('Consider splitting stake across multiple accounts');
      }

      // Validate tier consistency
      const expectedTier = this.getStakingTier(stakeAccount.stakedAmount.toNumber() / 1e9);
      if (stakeAccount.stakingTier !== expectedTier.tier) {
        issues.push(`Tier mismatch: expected ${expectedTier.tier}, got ${stakeAccount.stakingTier}`);
        recommendations.push('Account may need tier update');
      }

      // Validate timestamps
      const currentSlot = await this.connection.getSlot();
      if (stakeAccount.stakeTimestamp.gt(new BN(currentSlot))) {
        issues.push('Invalid stake timestamp (future date)');
      }

      if (stakeAccount.lastRewardClaim.gt(new BN(currentSlot))) {
        issues.push('Invalid last reward claim timestamp (future date)');
      }

      // Check for suspicious activity patterns
      if (stakeAccount.activityMultiplier > 3.0) {
        issues.push('Unusually high activity multiplier');
        recommendations.push('Activity patterns may be flagged for review');
      }

      return {
        isValid: issues.length === 0,
        issues,
        recommendations
      };
    } catch (error) {
      console.error('Error validating stake account:', error);
      throw new Error(`Failed to validate stake account: ${error.message}`);
    }
  }

  /**
   * Get optimal staking strategy recommendations
   */
  async getStakingRecommendations(
    userPublicKey: PublicKey,
    availableAmount: BN
  ): Promise<{
    currentTier: number;
    recommendedAmount: BN;
    targetTier: number;
    expectedApy: number;
    projectedRewards: BN;
    timeToNextTier: number;
    strategies: Array<{
      name: string;
      description: string;
      amount: BN;
      expectedReturn: BN;
      riskLevel: 'low' | 'medium' | 'high';
    }>;
  }> {
    try {
      const stakeAccount = await this.getStakeAccount(userPublicKey);
      const currentStaked = stakeAccount?.stakedAmount || new BN(0);
      const totalAvailable = currentStaked.add(availableAmount);

      const currentTier = this.getStakingTier(currentStaked.toNumber() / 1e9);
      const targetTier = this.getStakingTier(totalAvailable.toNumber() / 1e9);

      // Calculate expected APY with new amount
      const baseApy = 12; // Base APY percentage
      const tierMultiplier = targetTier.benefits.miningBoost;
      const expectedApy = baseApy * tierMultiplier;

      // Project annual rewards
      const projectedRewards = totalAvailable
        .mul(new BN(Math.floor(expectedApy * 100)))
        .div(new BN(10000)); // Convert percentage to basis points

      // Calculate time to next tier
      const nextTierThreshold = this.getNextTierThreshold(targetTier.tier);
      const amountToNextTier = nextTierThreshold ? new BN(nextTierThreshold * 1e9).sub(totalAvailable) : new BN(0);
      const timeToNextTier = amountToNextTier.gt(new BN(0)) 
        ? Math.ceil(amountToNextTier.div(projectedRewards.div(new BN(365))).toNumber())
        : 0;

      // Generate strategies
      const strategies = [
        {
          name: 'Conservative Growth',
          description: 'Stake minimum amount for next tier with low risk',
          amount: nextTierThreshold ? new BN(nextTierThreshold * 1e9).sub(currentStaked) : availableAmount,
          expectedReturn: projectedRewards.div(new BN(2)),
          riskLevel: 'low' as const
        },
        {
          name: 'Balanced Approach',
          description: 'Stake 70% of available amount for balanced returns',
          amount: availableAmount.mul(new BN(70)).div(new BN(100)),
          expectedReturn: projectedRewards.mul(new BN(70)).div(new BN(100)),
          riskLevel: 'medium' as const
        },
        {
          name: 'Maximum Rewards',
          description: 'Stake all available amount for maximum returns',
          amount: availableAmount,
          expectedReturn: projectedRewards,
          riskLevel: 'high' as const
        }
      ];

      return {
        currentTier: currentTier.tier,
        recommendedAmount: strategies[1].amount, // Balanced approach
        targetTier: targetTier.tier,
        expectedApy,
        projectedRewards,
        timeToNextTier,
        strategies
      };
    } catch (error) {
      console.error('Error getting staking recommendations:', error);
      throw new Error(`Failed to get staking recommendations: ${error.message}`);
    }
  }

  /**
   * Get compound interest projections
   */
  calculateCompoundProjections(
    principal: BN,
    apy: number,
    compoundFrequency: number = 365, // Daily compounding
    years: number = 5
  ): Array<{
    year: number;
    amount: BN;
    interest: BN;
    totalReturn: BN;
  }> {
    const projections = [];
    const rate = apy / 100;
    
    for (let year = 1; year <= years; year++) {
      const compoundAmount = principal.mul(
        new BN(Math.floor(Math.pow(1 + rate / compoundFrequency, compoundFrequency * year) * 1e9))
      ).div(new BN(1e9));
      
      const interest = compoundAmount.sub(principal);
      const totalReturn = interest.mul(new BN(100)).div(principal);

      projections.push({
        year,
        amount: compoundAmount,
        interest,
        totalReturn
      });
    }

    return projections;
  }

  /**
   * Calculate break-even analysis for staking
   */
  calculateBreakEvenAnalysis(
    stakeAmount: BN,
    stakingFee: number,
    unstakingFee: number,
    apy: number
  ): {
    breakEvenDays: number;
    totalFees: BN;
    dailyRewards: BN;
    profitableAfterDays: number;
  } {
    const totalFeesPercent = (stakingFee + unstakingFee) / 10000; // Convert basis points to percentage
    const totalFees = stakeAmount.mul(new BN(Math.floor(totalFeesPercent * 1000))).div(new BN(1000));
    
    const annualRewards = stakeAmount.mul(new BN(Math.floor(apy * 100))).div(new BN(10000));
    const dailyRewards = annualRewards.div(new BN(365));
    
    const breakEvenDays = totalFees.div(dailyRewards).toNumber();
    const profitableAfterDays = Math.ceil(breakEvenDays * 1.1); // 10% buffer

    return {
      breakEvenDays,
      totalFees,
      dailyRewards,
      profitableAfterDays
    };
  }

  /**
   * Helper method to calculate APY from account data
   */
  private calculateApyFromAccount(account: DecodedStakeAccount): number {
    const currentSlot = Date.now() / 1000; // Approximate
    const stakingDuration = currentSlot - account.stakeTimestamp.toNumber();
    const annualizedDuration = stakingDuration / (365 * 24 * 3600);
    
    if (annualizedDuration <= 0) return 0;
    
    const totalReturn = account.totalRewardsEarned.toNumber() / account.stakedAmount.toNumber();
    return (totalReturn / annualizedDuration) * 100;
  }

  /**
   * Helper method to get next tier threshold
   */
  private getNextTierThreshold(currentTier: number): number | null {
    const tiers = Object.values(STAKING_TIERS);
    const nextTier = tiers.find(tier => tier.min > STAKING_TIERS[`TIER_${currentTier}`].max);
    return nextTier ? nextTier.min : null;
  }

  /**
   * Get staking health score
   */
  async getStakingHealthScore(userPublicKey: PublicKey): Promise<{
    score: number;
    factors: {
      diversification: number;
      activityLevel: number;
      loyaltyDuration: number;
      tierOptimization: number;
      riskManagement: number;
    };
    recommendations: string[];
  }> {
    try {
      const stakeAccounts = await this.getAllUserStakeAccounts(userPublicKey);
      const recommendations: string[] = [];

      if (stakeAccounts.length === 0) {
        return {
          score: 0,
          factors: {
            diversification: 0,
            activityLevel: 0,
            loyaltyDuration: 0,
            tierOptimization: 0,
            riskManagement: 0
          },
          recommendations: ['Start staking to begin earning rewards']
        };
      }

      // Calculate diversification score (0-100)
      const diversification = Math.min(100, stakeAccounts.length * 25);
      if (diversification < 75) {
        recommendations.push('Consider diversifying across multiple pools');
      }

      // Calculate activity level score (0-100)
      const avgActivityMultiplier = stakeAccounts.reduce((sum, acc) => sum + acc.activityMultiplier, 0) / stakeAccounts.length;
      const activityLevel = Math.min(100, avgActivityMultiplier * 50);
      if (activityLevel < 60) {
        recommendations.push('Increase social media activity to boost rewards');
      }

      // Calculate loyalty duration score (0-100)
      const currentSlot = await this.connection.getSlot();
      const avgStakingDuration = stakeAccounts.reduce((sum, acc) => sum + (currentSlot - acc.stakeTimestamp.toNumber()), 0) / stakeAccounts.length;
      const loyaltyDuration = Math.min(100, (avgStakingDuration / (365 * 24 * 3600 / 0.4)) * 100); // Assuming 0.4s per slot
      if (loyaltyDuration < 50) {
        recommendations.push('Longer staking periods increase loyalty bonuses');
      }

      // Calculate tier optimization score (0-100)
      const totalStaked = stakeAccounts.reduce((sum, acc) => sum.add(acc.stakedAmount), new BN(0));
      const currentTier = this.getStakingTier(totalStaked.toNumber() / 1e9);
      const tierOptimization = Math.min(100, currentTier.tier * 20);
      if (tierOptimization < 80) {
        recommendations.push('Consider increasing stake amount for higher tier benefits');
      }

      // Calculate risk management score (0-100)
      const maxStakeRatio = Math.max(...stakeAccounts.map(acc => acc.stakedAmount.toNumber())) / totalStaked.toNumber();
      const riskManagement = Math.max(0, 100 - (maxStakeRatio * 100));
      if (riskManagement < 70) {
        recommendations.push('Avoid concentrating too much stake in one pool');
      }

      // Calculate overall score
      const factors = {
        diversification,
        activityLevel,
        loyaltyDuration,
        tierOptimization,
        riskManagement
      };

      const score = (diversification + activityLevel + loyaltyDuration + tierOptimization + riskManagement) / 5;

      return {
        score,
        factors,
        recommendations
      };
    } catch (error) {
      console.error('Error calculating staking health score:', error);
      throw new Error(`Failed to calculate staking health score: ${error.message}`);
    }
  }

  /**
   * Export staking data for external analysis
   */
  async exportStakingData(
    userPublicKey: PublicKey,
    format: 'json' | 'csv' = 'json'
  ): Promise<string> {
    try {
      const stakeAccounts = await this.getAllUserStakeAccounts(userPublicKey);
      const stakingStats = await this.getStakingStats();
      const healthScore = await this.getStakingHealthScore(userPublicKey);

      const exportData = {
        user: userPublicKey.toString(),
        exportTimestamp: new Date().toISOString(),
        stakeAccounts: stakeAccounts.map(account => ({
          ...account,
          owner: account.owner.toString(),
          stakePool: account.stakePool.toString(),
          stakedAmount: account.stakedAmount.toString(),
          sFinAmount: account.sFinAmount.toString(),
          totalRewardsEarned: account.totalRewardsEarned.toString(),
          pendingRewards: account.pendingRewards.toString()
        })),
        overallStats: {
          ...stakingStats,
          totalStaked: stakingStats.totalStaked.toString(),
          averageStake: stakingStats.averageStake.toString(),
          totalRewardsDistributed: stakingStats.totalRewardsDistributed.toString()
        },
        healthScore
      };

      if (format === 'json') {
        return JSON.stringify(exportData, null, 2);
      } else {
        // Convert to CSV format
        const csvLines = ['Field,Value'];
        csvLines.push(`User,${exportData.user}`);
        csvLines.push(`Export Time,${exportData.exportTimestamp}`);
        csvLines.push(`Total Stake Accounts,${exportData.stakeAccounts.length}`);
        csvLines.push(`Health Score,${exportData.healthScore.score}`);
        
        return csvLines.join('\n');
      }
    } catch (error) {
      console.error('Error exporting staking data:', error);
      throw new Error(`Failed to export staking data: ${error.message}`);
    }
  }
}

/**
 * Export the StakingAccounts class and related types
 */
export default StakingAccounts;

/**
 * Utility functions for staking calculations
 */
export const StakingUtils = {
  /**
   * Calculate optimal stake amount for target tier
   */
  calculateOptimalStakeForTier(targetTier: number): BN {
    const tier = STAKING_TIERS[`TIER_${targetTier}`];
    if (!tier) {
      throw new Error(`Invalid tier: ${targetTier}`);
    }
    return new BN(tier.min * 1e9); // Convert to lamports
  },

  /**
   * Format staking amount for display
   */
  formatStakeAmount(amount: BN, decimals: number = 9): string {
    const divisor = new BN(10).pow(new BN(decimals));
    const wholeAmount = amount.div(divisor);
    const fractionalAmount = amount.mod(divisor);
    
    if (fractionalAmount.isZero()) {
      return wholeAmount.toString();
    }
    
    const fractionalStr = fractionalAmount.toString().padStart(decimals, '0');
    return `${wholeAmount.toString()}.${fractionalStr.replace(/0+$/, '')}`;
  },

  /**
   * Parse staking amount from string
   */
  parseStakeAmount(amountStr: string, decimals: number = 9): BN {
    const parts = amountStr.split('.');
    const wholePart = new BN(parts[0] || '0');
    const fractionalPart = parts[1] ? new BN(parts[1].padEnd(decimals, '0').slice(0, decimals)) : new BN(0);
    
    const divisor = new BN(10).pow(new BN(decimals));
    return wholePart.mul(divisor).add(fractionalPart);
  },

  /**
   * Calculate compound annual growth rate (CAGR)
   */
  calculateCAGR(initialAmount: BN, finalAmount: BN, years: number): number {
    const initial = initialAmount.toNumber();
    const final = finalAmount.toNumber();
    return (Math.pow(final / initial, 1 / years) - 1) * 100;
  },

  /**
   * Validate staking parameters
   */
  validateStakingParams(amount: BN, minStake: BN, maxStake: BN): { isValid: boolean; error?: string } {
    if (amount.lt(minStake)) {
      return { isValid: false, error: `Amount below minimum stake: ${minStake.toString()}` };
    }
    if (amount.gt(maxStake)) {
      return { isValid: false, error: `Amount above maximum stake: ${maxStake.toString()}` };
    }
    return { isValid: true };
  }
};
