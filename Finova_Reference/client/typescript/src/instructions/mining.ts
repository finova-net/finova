// client/typescript/src/instructions/mining.ts

import {
  Connection,
  PublicKey,
  TransactionInstruction,
  SystemProgram,
  SYSVAR_RENT_PUBKEY,
  SYSVAR_CLOCK_PUBKEY,
} from '@solana/web3.js';
import { TOKEN_PROGRAM_ID, ASSOCIATED_TOKEN_PROGRAM_ID } from '@solana/spl-token';
import { Program, Provider, BN, IdlAccounts } from '@project-serum/anchor';
import { 
  FINOVA_CORE_PROGRAM_ID, 
  FINOVA_TOKEN_PROGRAM_ID,
  FINOVA_NFT_PROGRAM_ID,
  FIN_MINT_ADDRESS,
  NETWORK_SEED,
  USER_SEED,
  XP_SEED,
  REFERRAL_SEED,
  STAKING_SEED,
  ACTIVE_EFFECTS_SEED
} from '../constants';
import {
  MiningRewardsParams,
  MiningBoostParams,
  DailyLoginParams,
  QualityActivityParams,
  NetworkPhase,
  MiningCalculation,
  UserMiningState,
  ActiveEffect
} from '../types';

/**
 * Finova Mining Instructions
 * Handles all mining-related blockchain operations including:
 * - Claiming mining rewards
 * - Applying mining boosts
 * - Daily login bonuses
 * - Quality activity rewards
 * - Mining rate calculations
 */
export class MiningInstructions {
  constructor(
    private connection: Connection,
    private program: Program,
    private provider: Provider
  ) {}

  /**
   * Claims accumulated mining rewards for a user
   * Implements the core mining formula with XP, RP, and quality multipliers
   */
  async claimMiningRewards(params: MiningRewardsParams): Promise<TransactionInstruction> {
    const {
      user,
      forceCalculation = false,
      includeBoosts = true
    } = params;

    // Derive all necessary PDAs
    const [networkState] = await PublicKey.findProgramAddress(
      [Buffer.from(NETWORK_SEED)],
      FINOVA_CORE_PROGRAM_ID
    );

    const [userState] = await PublicKey.findProgramAddress(
      [Buffer.from(USER_SEED), user.toBuffer()],
      FINOVA_CORE_PROGRAM_ID
    );

    const [xpState] = await PublicKey.findProgramAddress(
      [Buffer.from(XP_SEED), user.toBuffer()],
      FINOVA_CORE_PROGRAM_ID
    );

    const [referralState] = await PublicKey.findProgramAddress(
      [Buffer.from(REFERRAL_SEED), user.toBuffer()],
      FINOVA_CORE_PROGRAM_ID
    );

    const [stakingState] = await PublicKey.findProgramAddress(
      [Buffer.from(STAKING_SEED), user.toBuffer()],
      FINOVA_CORE_PROGRAM_ID
    );

    const [activeEffectsState] = await PublicKey.findProgramAddress(
      [Buffer.from(ACTIVE_EFFECTS_SEED), user.toBuffer()],
      FINOVA_CORE_PROGRAM_ID
    );

    // Get user's FIN token account
    const [userTokenAccount] = await PublicKey.findProgramAddress(
      [user.toBuffer(), TOKEN_PROGRAM_ID.toBuffer(), FIN_MINT_ADDRESS.toBuffer()],
      ASSOCIATED_TOKEN_PROGRAM_ID
    );

    // Get token mint authority (controlled by finova-token program)
    const [mintAuthority] = await PublicKey.findProgramAddress(
      [Buffer.from("mint_authority")],
      FINOVA_TOKEN_PROGRAM_ID
    );

    const accounts = {
      user,
      networkState,
      userState,
      xpState,
      referralState,
      stakingState,
      activeEffectsState,
      userTokenAccount,
      finMint: FIN_MINT_ADDRESS,
      mintAuthority,
      finovaTokenProgram: FINOVA_TOKEN_PROGRAM_ID,
      tokenProgram: TOKEN_PROGRAM_ID,
      associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
      systemProgram: SystemProgram.programId,
      rent: SYSVAR_RENT_PUBKEY,
      clock: SYSVAR_CLOCK_PUBKEY,
    };

    const instruction = await this.program.methods
      .claimMiningRewards(forceCalculation, includeBoosts)
      .accounts(accounts)
      .instruction();

    return instruction;
  }

  /**
   * Applies a mining boost from special cards or promotional events
   */
  async applyMiningBoost(params: MiningBoostParams): Promise<TransactionInstruction> {
    const {
      user,
      boostType,
      multiplier,
      duration,
      cardId = null
    } = params;

    const [userState] = await PublicKey.findProgramAddress(
      [Buffer.from(USER_SEED), user.toBuffer()],
      FINOVA_CORE_PROGRAM_ID
    );

    const [activeEffectsState] = await PublicKey.findProgramAddress(
      [Buffer.from(ACTIVE_EFFECTS_SEED), user.toBuffer()],
      FINOVA_CORE_PROGRAM_ID
    );

    let accounts: any = {
      user,
      userState,
      activeEffectsState,
      systemProgram: SystemProgram.programId,
      rent: SYSVAR_RENT_PUBKEY,
      clock: SYSVAR_CLOCK_PUBKEY,
    };

    // If boost comes from NFT card, include NFT program accounts
    if (cardId) {
      const [cardState] = await PublicKey.findProgramAddress(
        [Buffer.from("card"), cardId.toBuffer()],
        FINOVA_NFT_PROGRAM_ID
      );

      accounts = {
        ...accounts,
        cardState,
        finovaNftProgram: FINOVA_NFT_PROGRAM_ID,
      };
    }

    const instruction = await this.program.methods
      .applyMiningBoost(
        { [boostType]: {} }, // Convert string to enum variant
        new BN(multiplier * 100), // Convert to basis points
        new BN(duration),
        cardId
      )
      .accounts(accounts)
      .instruction();

    return instruction;
  }

  /**
   * Records daily login and applies streak bonuses
   */
  async recordDailyLogin(params: DailyLoginParams): Promise<TransactionInstruction> {
    const {
      user,
      platform = "app",
      deviceFingerprint = null
    } = params;

    const [userState] = await PublicKey.findProgramAddress(
      [Buffer.from(USER_SEED), user.toBuffer()],
      FINOVA_CORE_PROGRAM_ID
    );

    const [xpState] = await PublicKey.findProgramAddress(
      [Buffer.from(XP_SEED), user.toBuffer()],
      FINOVA_CORE_PROGRAM_ID
    );

    const [activeEffectsState] = await PublicKey.findProgramAddress(
      [Buffer.from(ACTIVE_EFFECTS_SEED), user.toBuffer()],
      FINOVA_CORE_PROGRAM_ID
    );

    const accounts = {
      user,
      userState,
      xpState,
      activeEffectsState,
      systemProgram: SystemProgram.programId,
      clock: SYSVAR_CLOCK_PUBKEY,
    };

    const platformBytes = Buffer.from(platform.padEnd(16, '\0'));
    const fingerprintBytes = deviceFingerprint 
      ? Buffer.from(deviceFingerprint.padEnd(32, '\0'))
      : Buffer.alloc(32);

    const instruction = await this.program.methods
      .recordDailyLogin(
        Array.from(platformBytes),
        Array.from(fingerprintBytes)
      )
      .accounts(accounts)
      .instruction();

    return instruction;
  }

  /**
   * Records quality social media activity for mining bonuses
   */
  async recordQualityActivity(params: QualityActivityParams): Promise<TransactionInstruction> {
    const {
      user,
      activityType,
      platform,
      contentHash,
      qualityScore,
      engagementMetrics,
      viralBonus = false
    } = params;

    const [userState] = await PublicKey.findProgramAddress(
      [Buffer.from(USER_SEED), user.toBuffer()],
      FINOVA_CORE_PROGRAM_ID
    );

    const [xpState] = await PublicKey.findProgramAddress(
      [Buffer.from(XP_SEED), user.toBuffer()],
      FINOVA_CORE_PROGRAM_ID
    );

    const [referralState] = await PublicKey.findProgramAddress(
      [Buffer.from(REFERRAL_SEED), user.toBuffer()],
      FINOVA_CORE_PROGRAM_ID
    );

    const accounts = {
      user,
      userState,
      xpState,
      referralState,
      systemProgram: SystemProgram.programId,
      clock: SYSVAR_CLOCK_PUBKEY,
    };

    const activityTypeEnum = { [activityType]: {} };
    const platformEnum = { [platform]: {} };
    
    const instruction = await this.program.methods
      .recordQualityActivity(
        activityTypeEnum,
        platformEnum,
        Array.from(Buffer.from(contentHash.padEnd(32, '\0'))),
        new BN(qualityScore * 100), // Convert to basis points
        {
          likes: new BN(engagementMetrics.likes || 0),
          comments: new BN(engagementMetrics.comments || 0),
          shares: new BN(engagementMetrics.shares || 0),
          views: new BN(engagementMetrics.views || 0),
        },
        viralBonus
      )
      .accounts(accounts)
      .instruction();

    return instruction;
  }

  /**
   * Calculates current mining rate for a user (view function)
   */
  async calculateMiningRate(user: PublicKey): Promise<MiningCalculation> {
    try {
      // Fetch all relevant state accounts
      const [networkState] = await PublicKey.findProgramAddress(
        [Buffer.from(NETWORK_SEED)],
        FINOVA_CORE_PROGRAM_ID
      );

      const [userState] = await PublicKey.findProgramAddress(
        [Buffer.from(USER_SEED), user.toBuffer()],
        FINOVA_CORE_PROGRAM_ID
      );

      const [xpState] = await PublicKey.findProgramAddress(
        [Buffer.from(XP_SEED), user.toBuffer()],
        FINOVA_CORE_PROGRAM_ID
      );

      const [referralState] = await PublicKey.findProgramAddress(
        [Buffer.from(REFERRAL_SEED), user.toBuffer()],
        FINOVA_CORE_PROGRAM_ID
      );

      const [stakingState] = await PublicKey.findProgramAddress(
        [Buffer.from(STAKING_SEED), user.toBuffer()],
        FINOVA_CORE_PROGRAM_ID
      );

      const [activeEffectsState] = await PublicKey.findProgramAddress(
        [Buffer.from(ACTIVE_EFFECTS_SEED), user.toBuffer()],
        FINOVA_CORE_PROGRAM_ID
      );

      // Fetch account data
      const networkData = await this.program.account.networkState.fetch(networkState);
      const userData = await this.program.account.userState.fetch(userState);
      const xpData = await this.program.account.xpState.fetch(xpState);
      const referralData = await this.program.account.referralState.fetch(referralState);
      const stakingData = await this.program.account.stakingState.fetch(stakingState);
      const effectsData = await this.program.account.activeEffectsState.fetch(activeEffectsState);

      // Implement mining calculation logic based on whitepaper formulas
      const baseRate = this.calculateBaseRate(networkData);
      const pioneerBonus = this.calculatePioneerBonus(networkData);
      const xpMultiplier = this.calculateXPMultiplier(xpData);
      const referralBonus = this.calculateReferralBonus(referralData);
      const stakingBonus = this.calculateStakingBonus(stakingData);
      const securityBonus = userData.isKycVerified ? 1.2 : 0.8;
      const regressionFactor = this.calculateRegressionFactor(userData.totalEarned);
      const activeEffectsMultiplier = this.calculateActiveEffects(effectsData);

      const hourlyRate = baseRate * 
                        pioneerBonus * 
                        xpMultiplier * 
                        referralBonus * 
                        stakingBonus * 
                        securityBonus * 
                        regressionFactor * 
                        activeEffectsMultiplier;

      return {
        baseRate,
        pioneerBonus,
        xpMultiplier,
        referralBonus,
        stakingBonus,
        securityBonus,
        regressionFactor,
        activeEffectsMultiplier,
        hourlyRate,
        dailyRate: hourlyRate * 24,
        phase: this.getNetworkPhase(networkData),
        lastCalculation: Date.now(),
        pendingRewards: userData.pendingMiningRewards.toNumber() / 1e9, // Convert from lamports
      };
    } catch (error) {
      throw new Error(`Failed to calculate mining rate: ${error.message}`);
    }
  }

  /**
   * Gets current user mining state
   */
  async getUserMiningState(user: PublicKey): Promise<UserMiningState> {
    try {
      const [userState] = await PublicKey.findProgramAddress(
        [Buffer.from(USER_SEED), user.toBuffer()],
        FINOVA_CORE_PROGRAM_ID
      );

      const [activeEffectsState] = await PublicKey.findProgramAddress(
        [Buffer.from(ACTIVE_EFFECTS_SEED), user.toBuffer()],
        FINOVA_CORE_PROGRAM_ID
      );

      const userData = await this.program.account.userState.fetch(userState);
      const effectsData = await this.program.account.activeEffectsState.fetch(activeEffectsState);

      const activeEffects: ActiveEffect[] = effectsData.effects.map((effect: any) => ({
        effectType: Object.keys(effect.effectType)[0],
        multiplier: effect.multiplier / 100, // Convert from basis points
        expiresAt: effect.expiresAt.toNumber() * 1000, // Convert to milliseconds
        source: effect.source || "unknown",
        isActive: effect.expiresAt.toNumber() * 1000 > Date.now()
      }));

      return {
        user: user.toString(),
        totalEarned: userData.totalEarned.toNumber() / 1e9,
        pendingRewards: userData.pendingMiningRewards.toNumber() / 1e9,
        lastClaimTime: userData.lastMiningClaim.toNumber() * 1000,
        miningStartTime: userData.miningStartTime.toNumber() * 1000,
        dailyLoginStreak: userData.dailyLoginStreak,
        lastLoginTime: userData.lastLoginTime.toNumber() * 1000,
        isKycVerified: userData.isKycVerified,
        activeEffects,
        currentPhase: await this.getCurrentNetworkPhase(),
        estimatedHourlyRate: (await this.calculateMiningRate(user)).hourlyRate
      };
    } catch (error) {
      throw new Error(`Failed to get user mining state: ${error.message}`);
    }
  }

  // Private helper methods for calculations

  private calculateBaseRate(networkData: any): number {
    const totalUsers = networkData.totalUsers.toNumber();
    
    if (totalUsers < 100000) return 0.1; // Phase 1: Pioneer
    if (totalUsers < 1000000) return 0.05; // Phase 2: Growth
    if (totalUsers < 10000000) return 0.025; // Phase 3: Maturity
    return 0.01; // Phase 4: Stability
  }

  private calculatePioneerBonus(networkData: any): number {
    const totalUsers = networkData.totalUsers.toNumber();
    return Math.max(1.0, 2.0 - (totalUsers / 1000000));
  }

  private calculateXPMultiplier(xpData: any): number {
    const level = xpData.currentLevel;
    return 1.0 + (level / 100) * 4.0; // Scale to max 5.0x at level 100
  }

  private calculateReferralBonus(referralData: any): number {
    const activeReferrals = referralData.activeReferrals.toNumber();
    return Math.min(1.0 + (activeReferrals * 0.1), 3.0); // Cap at 3.0x
  }

  private calculateStakingBonus(stakingData: any): number {
    const stakingTier = stakingData.tier;
    const tierMultipliers = {
      0: 1.0,  // No staking
      1: 1.2,  // Bronze
      2: 1.35, // Silver
      3: 1.5,  // Gold
      4: 1.75, // Platinum
      5: 2.0   // Diamond
    };
    return tierMultipliers[stakingTier] || 1.0;
  }

  private calculateRegressionFactor(totalEarned: BN): number {
    const earned = totalEarned.toNumber() / 1e9; // Convert from lamports
    return Math.exp(-0.001 * earned);
  }

  private calculateActiveEffects(effectsData: any): number {
    let multiplier = 1.0;
    const currentTime = Date.now() / 1000;

    for (const effect of effectsData.effects) {
      if (effect.expiresAt.toNumber() > currentTime) {
        multiplier *= (effect.multiplier / 100); // Convert from basis points
      }
    }

    return multiplier;
  }

  private getNetworkPhase(networkData: any): NetworkPhase {
    const totalUsers = networkData.totalUsers.toNumber();
    
    if (totalUsers < 100000) return 'pioneer';
    if (totalUsers < 1000000) return 'growth';
    if (totalUsers < 10000000) return 'maturity';
    return 'stability';
  }

  private async getCurrentNetworkPhase(): Promise<NetworkPhase> {
    try {
      const [networkState] = await PublicKey.findProgramAddress(
        [Buffer.from(NETWORK_SEED)],
        FINOVA_CORE_PROGRAM_ID
      );

      const networkData = await this.program.account.networkState.fetch(networkState);
      return this.getNetworkPhase(networkData);
    } catch (error) {
      console.warn('Failed to get network phase, defaulting to pioneer:', error);
      return 'pioneer';
    }
  }

  /**
   * Estimates mining rewards for a given time period
   */
  async estimateRewards(
    user: PublicKey, 
    hours: number = 24
  ): Promise<{ estimated: number; breakdown: MiningCalculation }> {
    const calculation = await this.calculateMiningRate(user);
    const estimated = calculation.hourlyRate * hours;

    return {
      estimated,
      breakdown: calculation
    };
  }

  /**
   * Gets mining leaderboard position for a user
   */
  async getLeaderboardPosition(user: PublicKey): Promise<{
    position: number;
    totalUsers: number;
    percentile: number;
  }> {
    try {
      // This would typically require an indexed query or off-chain calculation
      // For now, return estimated position based on total earned
      const userState = await this.getUserMiningState(user);
      const totalEarned = userState.totalEarned;

      // Placeholder calculation - in production, this would query all users
      const estimatedPosition = Math.max(1, Math.floor(1000 - (totalEarned * 10)));
      const totalUsers = 10000; // Get from network state in production
      const percentile = ((totalUsers - estimatedPosition) / totalUsers) * 100;

      return {
        position: estimatedPosition,
        totalUsers,
        percentile: Math.round(percentile * 100) / 100
      };
    } catch (error) {
      throw new Error(`Failed to get leaderboard position: ${error.message}`);
    }
  }
}
