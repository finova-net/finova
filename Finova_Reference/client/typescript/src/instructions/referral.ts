// client/typescript/src/instructions/referral.ts

import {
  PublicKey,
  TransactionInstruction,
  SystemProgram,
  SYSVAR_RENT_PUBKEY,
  Connection,
  AccountMeta,
} from '@solana/web3.js';
import { Program, BN, web3 } from '@project-serum/anchor';
import {
  TOKEN_PROGRAM_ID,
  ASSOCIATED_TOKEN_PROGRAM_ID,
  getAssociatedTokenAddress,
} from '@solana/spl-token';
import {
  FINOVA_CORE_PROGRAM_ID,
  FINOVA_TOKEN_PROGRAM_ID,
  SEEDS,
  ReferralTier,
  NetworkLevel,
} from '../constants';
import {
  ReferralState,
  UserState,
  NetworkState,
  ReferralBonus,
  NetworkStats,
  ReferralReward,
  InstructionResult,
} from '../types';

/**
 * Referral system instruction builder for Finova Network
 * Manages referral networks, rewards, and tier progression
 */
export class ReferralInstructions {
  constructor(
    private program: Program,
    private connection: Connection
  ) {}

  /**
   * Initialize referral system for a user
   * Creates the initial referral state account
   */
  async initializeReferral(
    user: PublicKey,
    referrer?: PublicKey
  ): Promise<InstructionResult> {
    try {
      // Derive PDAs
      const [userState] = PublicKey.findProgramAddressSync(
        [Buffer.from(SEEDS.USER_STATE), user.toBuffer()],
        FINOVA_CORE_PROGRAM_ID
      );

      const [referralState] = PublicKey.findProgramAddressSync(
        [Buffer.from(SEEDS.REFERRAL_STATE), user.toBuffer()],
        FINOVA_CORE_PROGRAM_ID
      );

      const [networkState] = PublicKey.findProgramAddressSync(
        [Buffer.from(SEEDS.NETWORK_STATE)],
        FINOVA_CORE_PROGRAM_ID
      );

      let referrerState: PublicKey | undefined;
      let referrerReferralState: PublicKey | undefined;

      if (referrer) {
        [referrerState] = PublicKey.findProgramAddressSync(
          [Buffer.from(SEEDS.USER_STATE), referrer.toBuffer()],
          FINOVA_CORE_PROGRAM_ID
        );

        [referrerReferralState] = PublicKey.findProgramAddressSync(
          [Buffer.from(SEEDS.REFERRAL_STATE), referrer.toBuffer()],
          FINOVA_CORE_PROGRAM_ID
        );
      }

      // Build accounts array
      const accounts: AccountMeta[] = [
        { pubkey: user, isSigner: true, isWritable: false },
        { pubkey: userState, isSigner: false, isWritable: false },
        { pubkey: referralState, isSigner: false, isWritable: true },
        { pubkey: networkState, isSigner: false, isWritable: true },
        { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
        { pubkey: SYSVAR_RENT_PUBKEY, isSigner: false, isWritable: false },
      ];

      // Add referrer accounts if provided
      if (referrer && referrerState && referrerReferralState) {
        accounts.push(
          { pubkey: referrer, isSigner: false, isWritable: false },
          { pubkey: referrerState, isSigner: false, isWritable: false },
          { pubkey: referrerReferralState, isSigner: false, isWritable: true }
        );
      }

      const instruction = new TransactionInstruction({
        keys: accounts,
        programId: FINOVA_CORE_PROGRAM_ID,
        data: Buffer.from([
          0, // Initialize referral instruction discriminator
          ...(referrer ? [1] : [0]), // Has referrer flag
        ]),
      });

      return {
        instruction,
        accounts: {
          user,
          userState,
          referralState,
          networkState,
          referrer,
          referrerState,
          referrerReferralState,
        },
        success: true,
      };
    } catch (error) {
      console.error('Error creating initialize referral instruction:', error);
      return {
        instruction: null,
        accounts: {},
        success: false,
        error: error instanceof Error ? error.message : 'Unknown error',
      };
    }
  }

  /**
   * Add a referral to user's network
   * Updates referral counts and calculates bonuses
   */
  async addReferral(
    referrer: PublicKey,
    newReferral: PublicKey,
    level: NetworkLevel = NetworkLevel.L1
  ): Promise<InstructionResult> {
    try {
      // Derive PDAs
      const [referrerState] = PublicKey.findProgramAddressSync(
        [Buffer.from(SEEDS.USER_STATE), referrer.toBuffer()],
        FINOVA_CORE_PROGRAM_ID
      );

      const [referrerReferralState] = PublicKey.findProgramAddressSync(
        [Buffer.from(SEEDS.REFERRAL_STATE), referrer.toBuffer()],
        FINOVA_CORE_PROGRAM_ID
      );

      const [referralState] = PublicKey.findProgramAddressSync(
        [Buffer.from(SEEDS.REFERRAL_STATE), newReferral.toBuffer()],
        FINOVA_CORE_PROGRAM_ID
      );

      const [networkState] = PublicKey.findProgramAddressSync(
        [Buffer.from(SEEDS.NETWORK_STATE)],
        FINOVA_CORE_PROGRAM_ID
      );

      const instruction = new TransactionInstruction({
        keys: [
          { pubkey: referrer, isSigner: true, isWritable: false },
          { pubkey: referrerState, isSigner: false, isWritable: false },
          { pubkey: referrerReferralState, isSigner: false, isWritable: true },
          { pubkey: newReferral, isSigner: false, isWritable: false },
          { pubkey: referralState, isSigner: false, isWritable: true },
          { pubkey: networkState, isSigner: false, isWritable: true },
        ],
        programId: FINOVA_CORE_PROGRAM_ID,
        data: Buffer.from([
          1, // Add referral instruction discriminator
          level, // Network level (L1, L2, L3)
        ]),
      });

      return {
        instruction,
        accounts: {
          referrer,
          referrerState,
          referrerReferralState,
          newReferral,
          referralState,
          networkState,
        },
        success: true,
      };
    } catch (error) {
      console.error('Error creating add referral instruction:', error);
      return {
        instruction: null,
        accounts: {},
        success: false,
        error: error instanceof Error ? error.message : 'Unknown error',
      };
    }
  }

  /**
   * Update referral points based on user activity
   * Calculates and distributes referral rewards
   */
  async updateReferralPoints(
    user: PublicKey,
    activityType: string,
    pointsEarned: number,
    qualityScore: number = 1.0
  ): Promise<InstructionResult> {
    try {
      // Derive PDAs
      const [userState] = PublicKey.findProgramAddressSync(
        [Buffer.from(SEEDS.USER_STATE), user.toBuffer()],
        FINOVA_CORE_PROGRAM_ID
      );

      const [referralState] = PublicKey.findProgramAddressSync(
        [Buffer.from(SEEDS.REFERRAL_STATE), user.toBuffer()],
        FINOVA_CORE_PROGRAM_ID
      );

      const [networkState] = PublicKey.findProgramAddressSync(
        [Buffer.from(SEEDS.NETWORK_STATE)],
        FINOVA_CORE_PROGRAM_ID
      );

      // Get referral state to find referrer
      const referralData = await this.getReferralState(user);
      let referrerAccounts: AccountMeta[] = [];

      if (referralData?.referrer) {
        const [referrerState] = PublicKey.findProgramAddressSync(
          [Buffer.from(SEEDS.USER_STATE), referralData.referrer.toBuffer()],
          FINOVA_CORE_PROGRAM_ID
        );

        const [referrerReferralState] = PublicKey.findProgramAddressSync(
          [Buffer.from(SEEDS.REFERRAL_STATE), referralData.referrer.toBuffer()],
          FINOVA_CORE_PROGRAM_ID
        );

        referrerAccounts = [
          { pubkey: referralData.referrer, isSigner: false, isWritable: false },
          { pubkey: referrerState, isSigner: false, isWritable: false },
          { pubkey: referrerReferralState, isSigner: false, isWritable: true },
        ];
      }

      // Encode activity type and data
      const activityBuffer = Buffer.alloc(32);
      activityBuffer.write(activityType.slice(0, 31), 0, 'utf8');

      const pointsBuffer = Buffer.alloc(8);
      pointsBuffer.writeBigUInt64LE(BigInt(Math.floor(pointsEarned * 1000)), 0); // Scale by 1000 for precision

      const qualityBuffer = Buffer.alloc(8);
      qualityBuffer.writeBigUInt64LE(BigInt(Math.floor(qualityScore * 1000)), 0); // Scale by 1000 for precision

      const instruction = new TransactionInstruction({
        keys: [
          { pubkey: user, isSigner: true, isWritable: false },
          { pubkey: userState, isSigner: false, isWritable: true },
          { pubkey: referralState, isSigner: false, isWritable: true },
          { pubkey: networkState, isSigner: false, isWritable: true },
          ...referrerAccounts,
        ],
        programId: FINOVA_CORE_PROGRAM_ID,
        data: Buffer.concat([
          Buffer.from([2]), // Update RP instruction discriminator
          activityBuffer,
          pointsBuffer,
          qualityBuffer,
        ]),
      });

      return {
        instruction,
        accounts: {
          user,
          userState,
          referralState,
          networkState,
          referrer: referralData?.referrer,
        },
        success: true,
      };
    } catch (error) {
      console.error('Error creating update referral points instruction:', error);
      return {
        instruction: null,
        accounts: {},
        success: false,
        error: error instanceof Error ? error.message : 'Unknown error',
      };
    }
  }

  /**
   * Claim accumulated referral rewards
   * Distributes $FIN tokens based on referral network activity
   */
  async claimReferralRewards(user: PublicKey): Promise<InstructionResult> {
    try {
      // Derive PDAs
      const [userState] = PublicKey.findProgramAddressSync(
        [Buffer.from(SEEDS.USER_STATE), user.toBuffer()],
        FINOVA_CORE_PROGRAM_ID
      );

      const [referralState] = PublicKey.findProgramAddressSync(
        [Buffer.from(SEEDS.REFERRAL_STATE), user.toBuffer()],
        FINOVA_CORE_PROGRAM_ID
      );

      const [networkState] = PublicKey.findProgramAddressSync(
        [Buffer.from(SEEDS.NETWORK_STATE)],
        FINOVA_CORE_PROGRAM_ID
      );

      // Token accounts
      const [finMint] = PublicKey.findProgramAddressSync(
        [Buffer.from(SEEDS.FIN_MINT)],
        FINOVA_TOKEN_PROGRAM_ID
      );

      const userTokenAccount = await getAssociatedTokenAddress(
        finMint,
        user,
        false,
        TOKEN_PROGRAM_ID,
        ASSOCIATED_TOKEN_PROGRAM_ID
      );

      // Core program authority for CPI
      const [coreAuthority] = PublicKey.findProgramAddressSync(
        [Buffer.from(SEEDS.CORE_AUTHORITY)],
        FINOVA_CORE_PROGRAM_ID
      );

      const instruction = new TransactionInstruction({
        keys: [
          { pubkey: user, isSigner: true, isWritable: false },
          { pubkey: userState, isSigner: false, isWritable: true },
          { pubkey: referralState, isSigner: false, isWritable: true },
          { pubkey: networkState, isSigner: false, isWritable: false },
          { pubkey: finMint, isSigner: false, isWritable: true },
          { pubkey: userTokenAccount, isSigner: false, isWritable: true },
          { pubkey: coreAuthority, isSigner: false, isWritable: false },
          { pubkey: FINOVA_TOKEN_PROGRAM_ID, isSigner: false, isWritable: false },
          { pubkey: TOKEN_PROGRAM_ID, isSigner: false, isWritable: false },
          { pubkey: ASSOCIATED_TOKEN_PROGRAM_ID, isSigner: false, isWritable: false },
          { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
        ],
        programId: FINOVA_CORE_PROGRAM_ID,
        data: Buffer.from([3]), // Claim referral rewards instruction discriminator
      });

      return {
        instruction,
        accounts: {
          user,
          userState,
          referralState,
          networkState,
          finMint,
          userTokenAccount,
          coreAuthority,
        },
        success: true,
      };
    } catch (error) {
      console.error('Error creating claim referral rewards instruction:', error);
      return {
        instruction: null,
        accounts: {},
        success: false,
        error: error instanceof Error ? error.message : 'Unknown error',
      };
    }
  }

  /**
   * Upgrade referral tier based on network performance
   * Updates tier and unlocks new benefits
   */
  async upgradeReferralTier(user: PublicKey): Promise<InstructionResult> {
    try {
      // Derive PDAs
      const [userState] = PublicKey.findProgramAddressSync(
        [Buffer.from(SEEDS.USER_STATE), user.toBuffer()],
        FINOVA_CORE_PROGRAM_ID
      );

      const [referralState] = PublicKey.findProgramAddressSync(
        [Buffer.from(SEEDS.REFERRAL_STATE), user.toBuffer()],
        FINOVA_CORE_PROGRAM_ID
      );

      const [networkState] = PublicKey.findProgramAddressSync(
        [Buffer.from(SEEDS.NETWORK_STATE)],
        FINOVA_CORE_PROGRAM_ID
      );

      const instruction = new TransactionInstruction({
        keys: [
          { pubkey: user, isSigner: true, isWritable: false },
          { pubkey: userState, isSigner: false, isWritable: true },
          { pubkey: referralState, isSigner: false, isWritable: true },
          { pubkey: networkState, isSigner: false, isWritable: false },
        ],
        programId: FINOVA_CORE_PROGRAM_ID,
        data: Buffer.from([4]), // Upgrade tier instruction discriminator
      });

      return {
        instruction,
        accounts: {
          user,
          userState,
          referralState,
          networkState,
        },
        success: true,
      };
    } catch (error) {
      console.error('Error creating upgrade referral tier instruction:', error);
      return {
        instruction: null,
        accounts: {},
        success: false,
        error: error instanceof Error ? error.message : 'Unknown error',
      };
    }
  }

  /**
   * Generate referral code for user
   * Creates unique code for sharing and tracking
   */
  async generateReferralCode(
    user: PublicKey,
    customCode?: string
  ): Promise<InstructionResult> {
    try {
      // Derive PDAs
      const [userState] = PublicKey.findProgramAddressSync(
        [Buffer.from(SEEDS.USER_STATE), user.toBuffer()],
        FINOVA_CORE_PROGRAM_ID
      );

      const [referralState] = PublicKey.findProgramAddressSync(
        [Buffer.from(SEEDS.REFERRAL_STATE), user.toBuffer()],
        FINOVA_CORE_PROGRAM_ID
      );

      // Encode custom code if provided
      const codeBuffer = Buffer.alloc(16);
      if (customCode) {
        codeBuffer.write(customCode.slice(0, 15), 0, 'utf8');
      }

      const instruction = new TransactionInstruction({
        keys: [
          { pubkey: user, isSigner: true, isWritable: false },
          { pubkey: userState, isSigner: false, isWritable: false },
          { pubkey: referralState, isSigner: false, isWritable: true },
        ],
        programId: FINOVA_CORE_PROGRAM_ID,
        data: Buffer.concat([
          Buffer.from([5]), // Generate code instruction discriminator
          Buffer.from([customCode ? 1 : 0]), // Has custom code flag
          codeBuffer,
        ]),
      });

      return {
        instruction,
        accounts: {
          user,
          userState,
          referralState,
        },
        success: true,
      };
    } catch (error) {
      console.error('Error creating generate referral code instruction:', error);
      return {
        instruction: null,
        accounts: {},
        success: false,
        error: error instanceof Error ? error.message : 'Unknown error',
      };
    }
  }

  // Utility methods for referral data fetching

  /**
   * Fetch referral state for a user
   */
  async getReferralState(user: PublicKey): Promise<ReferralState | null> {
    try {
      const [referralState] = PublicKey.findProgramAddressSync(
        [Buffer.from(SEEDS.REFERRAL_STATE), user.toBuffer()],
        FINOVA_CORE_PROGRAM_ID
      );

      const accountInfo = await this.connection.getAccountInfo(referralState);
      if (!accountInfo) return null;

      // Parse referral state data (simplified version)
      return this.parseReferralState(accountInfo.data);
    } catch (error) {
      console.error('Error fetching referral state:', error);
      return null;
    }
  }

  /**
   * Calculate referral bonus for a user
   */
  async calculateReferralBonus(
    user: PublicKey,
    baseAmount: number
  ): Promise<ReferralBonus> {
    try {
      const referralState = await this.getReferralState(user);
      if (!referralState) {
        return {
          baseAmount,
          bonusMultiplier: 1.0,
          bonusAmount: 0,
          totalAmount: baseAmount,
          tier: ReferralTier.Explorer,
        };
      }

      // Calculate bonus based on tier and network size
      const multiplier = this.getTierMultiplier(referralState.tier);
      const networkBonus = Math.min(referralState.activeReferrals * 0.01, 0.5); // Max 50% bonus
      const totalMultiplier = multiplier + networkBonus;
      const bonusAmount = baseAmount * (totalMultiplier - 1.0);

      return {
        baseAmount,
        bonusMultiplier: totalMultiplier,
        bonusAmount,
        totalAmount: baseAmount + bonusAmount,
        tier: referralState.tier,
        networkSize: referralState.activeReferrals,
      };
    } catch (error) {
      console.error('Error calculating referral bonus:', error);
      return {
        baseAmount,
        bonusMultiplier: 1.0,
        bonusAmount: 0,
        totalAmount: baseAmount,
        tier: ReferralTier.Explorer,
      };
    }
  }

  /**
   * Get network statistics for a user
   */
  async getNetworkStats(user: PublicKey): Promise<NetworkStats | null> {
    try {
      const referralState = await this.getReferralState(user);
      if (!referralState) return null;

      return {
        totalReferrals: referralState.totalReferrals,
        activeReferrals: referralState.activeReferrals,
        l1Referrals: referralState.l1Referrals,
        l2Referrals: referralState.l2Referrals,
        l3Referrals: referralState.l3Referrals,
        totalReferralRewards: referralState.totalReferralRewards,
        currentTier: referralState.tier,
        tierProgress: this.calculateTierProgress(referralState),
        networkQuality: referralState.networkQualityScore,
      };
    } catch (error) {
      console.error('Error fetching network stats:', error);
      return null;
    }
  }

  // Private utility methods

  private parseReferralState(data: Buffer): ReferralState {
    // Simplified parsing - in real implementation, use proper borsh deserialization
    let offset = 8; // Skip discriminator

    const referrer = data.subarray(offset, offset + 32);
    offset += 32;

    const referralCode = data.subarray(offset, offset + 16).toString('utf8').replace(/\0/g, '');
    offset += 16;

    const totalReferrals = data.readUInt32LE(offset);
    offset += 4;

    const activeReferrals = data.readUInt32LE(offset);
    offset += 4;

    const l1Referrals = data.readUInt32LE(offset);
    offset += 4;

    const l2Referrals = data.readUInt32LE(offset);
    offset += 4;

    const l3Referrals = data.readUInt32LE(offset);
    offset += 4;

    const totalReferralPoints = Number(data.readBigUInt64LE(offset));
    offset += 8;

    const totalReferralRewards = Number(data.readBigUInt64LE(offset));
    offset += 8;

    const tier = data.readUInt8(offset);
    offset += 1;

    const networkQualityScore = data.readFloatLE(offset);
    offset += 4;

    return {
      referrer: new PublicKey(referrer),
      referralCode,
      totalReferrals,
      activeReferrals,
      l1Referrals,
      l2Referrals,
      l3Referrals,
      totalReferralPoints,
      totalReferralRewards,
      tier: tier as ReferralTier,
      networkQualityScore,
      lastUpdated: Date.now(),
    };
  }

  private getTierMultiplier(tier: ReferralTier): number {
    switch (tier) {
      case ReferralTier.Explorer:
        return 1.0;
      case ReferralTier.Connector:
        return 1.2;
      case ReferralTier.Influencer:
        return 1.5;
      case ReferralTier.Leader:
        return 2.0;
      case ReferralTier.Ambassador:
        return 3.0;
      default:
        return 1.0;
    }
  }

  private calculateTierProgress(referralState: ReferralState): number {
    const tierRequirements = {
      [ReferralTier.Explorer]: { rp: 0, referrals: 0 },
      [ReferralTier.Connector]: { rp: 1000, referrals: 10 },
      [ReferralTier.Influencer]: { rp: 5000, referrals: 25 },
      [ReferralTier.Leader]: { rp: 15000, referrals: 50 },
      [ReferralTier.Ambassador]: { rp: 50000, referrals: 100 },
    };

    const currentTier = referralState.tier;
    const nextTier = Math.min(currentTier + 1, ReferralTier.Ambassador);
    
    if (nextTier === currentTier) return 100; // Already at max tier

    const nextRequirement = tierRequirements[nextTier];
    const rpProgress = Math.min(referralState.totalReferralPoints / nextRequirement.rp, 1);
    const referralProgress = Math.min(referralState.activeReferrals / nextRequirement.referrals, 1);

    return Math.min(rpProgress, referralProgress) * 100;
  }
}

/**
 * Helper function to create referral instructions
 */
export function createReferralInstructions(
  program: Program,
  connection: Connection
): ReferralInstructions {
  return new ReferralInstructions(program, connection);
}

/**
 * Validate referral code format
 */
export function validateReferralCode(code: string): boolean {
  // Referral code must be 3-15 characters, alphanumeric
  const regex = /^[a-zA-Z0-9]{3,15}$/;
  return regex.test(code);
}

/**
 * Generate random referral code
 */
export function generateRandomReferralCode(length: number = 8): string {
  const chars = 'ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789';
  let result = '';
  for (let i = 0; i < length; i++) {
    result += chars.charAt(Math.floor(Math.random() * chars.length));
  }
  return result;
}

/**
 * Calculate network value based on referral structure
 */
export function calculateNetworkValue(
  l1Count: number,
  l2Count: number,
  l3Count: number,
  avgActivity: number = 1.0
): number {
  const l1Value = l1Count * 1.0 * avgActivity;
  const l2Value = l2Count * 0.3 * avgActivity;
  const l3Value = l3Count * 0.1 * avgActivity;
  
  return l1Value + l2Value + l3Value;
}

export default ReferralInstructions;
