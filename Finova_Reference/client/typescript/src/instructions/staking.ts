// client/typescript/src/instructions/staking.ts

/**
 * Finova Network - Staking Instructions
 * TypeScript SDK for Staking Operations
 * 
 * This module provides comprehensive staking functionality including:
 * - Token staking with tier calculations
 * - Unstaking with cooldown periods
 * - Reward claiming with multipliers
 * - Staking tier management
 * - Integration with XP and RP systems
 * 
 * @version 1.0.0
 * @author Finova Network Team
 */

import {
  PublicKey,
  Transaction,
  TransactionInstruction,
  SystemProgram,
  SYSVAR_RENT_PUBKEY,
  SYSVAR_CLOCK_PUBKEY,
} from '@solana/web3.js';
import {
  TOKEN_PROGRAM_ID,
  ASSOCIATED_TOKEN_PROGRAM_ID,
  getAssociatedTokenAddress,
  createAssociatedTokenAccountInstruction,
} from '@solana/spl-token';
import { BN } from '@coral-xyz/anchor';
import { 
  FINOVA_CORE_PROGRAM_ID, 
  FINOVA_TOKEN_PROGRAM_ID,
  STAKING_SEED,
  USER_SEED,
  ACTIVE_EFFECTS_SEED
} from '../constants';

/**
 * Staking tier definitions based on stake amount
 */
export enum StakingTier {
  BASIC = 0,     // 100-499 $FIN
  PREMIUM = 1,   // 500-999 $FIN
  VIP = 2,       // 1,000-4,999 $FIN
  GUILD_MASTER = 3, // 5,000-9,999 $FIN
  DAO_MEMBER = 4    // 10,000+ $FIN
}

/**
 * Staking tier benefits configuration
 */
export interface StakingTierBenefits {
  tier: StakingTier;
  minAmount: BN;
  maxAmount: BN;
  apyRate: number;
  miningBoost: number;
  xpMultiplier: number;
  rpBonus: number;
  features: string[];
}

/**
 * Staking tier benefits lookup table
 */
export const STAKING_TIER_BENEFITS: Record<StakingTier, StakingTierBenefits> = {
  [StakingTier.BASIC]: {
    tier: StakingTier.BASIC,
    minAmount: new BN(100_000_000), // 100 $FIN (8 decimals)
    maxAmount: new BN(499_000_000), // 499 $FIN
    apyRate: 8.0,
    miningBoost: 20,
    xpMultiplier: 10,
    rpBonus: 5,
    features: ['Basic staking rewards', 'Priority support']
  },
  [StakingTier.PREMIUM]: {
    tier: StakingTier.PREMIUM,
    minAmount: new BN(500_000_000), // 500 $FIN
    maxAmount: new BN(999_000_000), // 999 $FIN
    apyRate: 10.0,
    miningBoost: 35,
    xpMultiplier: 20,
    rpBonus: 10,
    features: ['Premium badge', 'VIP support', 'Exclusive events']
  },
  [StakingTier.VIP]: {
    tier: StakingTier.VIP,
    minAmount: new BN(1000_000_000), // 1,000 $FIN
    maxAmount: new BN(4999_000_000), // 4,999 $FIN
    apyRate: 12.0,
    miningBoost: 50,
    xpMultiplier: 30,
    rpBonus: 20,
    features: ['VIP features', 'Guild access', 'Creator tools']
  },
  [StakingTier.GUILD_MASTER]: {
    tier: StakingTier.GUILD_MASTER,
    minAmount: new BN(5000_000_000), // 5,000 $FIN
    maxAmount: new BN(9999_000_000), // 9,999 $FIN
    apyRate: 14.0,
    miningBoost: 75,
    xpMultiplier: 50,
    rpBonus: 35,
    features: ['Guild master privileges', 'Advanced analytics', 'Beta features']
  },
  [StakingTier.DAO_MEMBER]: {
    tier: StakingTier.DAO_MEMBER,
    minAmount: new BN(10000_000_000), // 10,000 $FIN
    maxAmount: new BN(Number.MAX_SAFE_INTEGER),
    apyRate: 15.0,
    miningBoost: 100,
    xpMultiplier: 75,
    rpBonus: 50,
    features: ['DAO governance', 'Maximum benefits', 'Exclusive access']
  }
};

/**
 * Calculate staking tier based on stake amount
 */
export function calculateStakingTier(stakeAmount: BN): StakingTier {
  for (const [tier, benefits] of Object.entries(STAKING_TIER_BENEFITS)) {
    if (stakeAmount.gte(benefits.minAmount) && stakeAmount.lte(benefits.maxAmount)) {
      return parseInt(tier) as StakingTier;
    }
  }
  return StakingTier.BASIC; // Default fallback
}

/**
 * Calculate expected APY rewards
 */
export function calculateStakingRewards(
  stakeAmount: BN,
  durationDays: number,
  tier?: StakingTier
): BN {
  const stakingTier = tier || calculateStakingTier(stakeAmount);
  const benefits = STAKING_TIER_BENEFITS[stakingTier];
  
  // Annual rewards = stakeAmount * (APY / 100)
  const annualRewards = stakeAmount.mul(new BN(benefits.apyRate * 100)).div(new BN(10000));
  
  // Daily rewards = annualRewards / 365
  const dailyRewards = annualRewards.div(new BN(365));
  
  // Total rewards = dailyRewards * duration
  return dailyRewards.mul(new BN(durationDays));
}

/**
 * Calculate loyalty bonus based on staking duration
 */
export function calculateLoyaltyBonus(stakingDurationMonths: number): number {
  // 5% bonus per month, capped at 50%
  return Math.min(stakingDurationMonths * 5, 50);
}

/**
 * Calculate total staking multiplier including all bonuses
 */
export function calculateTotalStakingMultiplier(
  tier: StakingTier,
  loyaltyMonths: number,
  activityScore: number
): number {
  const benefits = STAKING_TIER_BENEFITS[tier];
  const loyaltyBonus = calculateLoyaltyBonus(loyaltyMonths);
  const activityBonus = Math.min(activityScore * 10, 100); // Max 100% activity bonus
  
  // Base multiplier + loyalty + activity bonuses
  return 100 + benefits.miningBoost + loyaltyBonus + activityBonus;
}

/**
 * Derive staking account PDA
 */
export function deriveStakingAddress(
  user: PublicKey,
  programId: PublicKey = FINOVA_CORE_PROGRAM_ID
): [PublicKey, number] {
  return PublicKey.findProgramAddressSync(
    [Buffer.from(STAKING_SEED), user.toBuffer()],
    programId
  );
}

/**
 * Derive user state account PDA
 */
export function deriveUserAddress(
  user: PublicKey,
  programId: PublicKey = FINOVA_CORE_PROGRAM_ID
): [PublicKey, number] {
  return PublicKey.findProgramAddressSync(
    [Buffer.from(USER_SEED), user.toBuffer()],
    programId
  );
}

/**
 * Derive active effects account PDA (for staking bonuses)
 */
export function deriveActiveEffectsAddress(
  user: PublicKey,
  programId: PublicKey = FINOVA_CORE_PROGRAM_ID
): [PublicKey, number] {
  return PublicKey.findProgramAddressSync(
    [Buffer.from(ACTIVE_EFFECTS_SEED), user.toBuffer()],
    programId
  );
}

/**
 * Create stake tokens instruction
 */
export async function createStakeInstruction(
  user: PublicKey,
  amount: BN,
  programId: PublicKey = FINOVA_CORE_PROGRAM_ID
): Promise<TransactionInstruction> {
  const [stakingAccount] = deriveStakingAddress(user, programId);
  const [userAccount] = deriveUserAddress(user, programId);
  const [activeEffectsAccount] = deriveActiveEffectsAddress(user, programId);
  
  // Get user's FIN token account
  const userTokenAccount = await getAssociatedTokenAddress(
    new PublicKey("FINTokenMintAddress"), // Replace with actual FIN mint
    user
  );
  
  // Get staking vault token account
  const stakingVault = await getAssociatedTokenAddress(
    new PublicKey("FINTokenMintAddress"), // Replace with actual FIN mint
    stakingAccount,
    true // Allow PDA owner
  );

  const accounts = {
    user,
    userAccount,
    stakingAccount,
    activeEffectsAccount,
    userTokenAccount,
    stakingVault,
    tokenProgram: TOKEN_PROGRAM_ID,
    associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
    systemProgram: SystemProgram.programId,
    rent: SYSVAR_RENT_PUBKEY,
    clock: SYSVAR_CLOCK_PUBKEY,
  };

  const data = Buffer.alloc(16);
  data.writeUInt32LE(4, 0); // Instruction discriminator for stake
  data.writeBigUInt64LE(BigInt(amount.toString()), 8);

  return new TransactionInstruction({
    keys: [
      { pubkey: accounts.user, isSigner: true, isWritable: false },
      { pubkey: accounts.userAccount, isSigner: false, isWritable: true },
      { pubkey: accounts.stakingAccount, isSigner: false, isWritable: true },
      { pubkey: accounts.activeEffectsAccount, isSigner: false, isWritable: true },
      { pubkey: accounts.userTokenAccount, isSigner: false, isWritable: true },
      { pubkey: accounts.stakingVault, isSigner: false, isWritable: true },
      { pubkey: accounts.tokenProgram, isSigner: false, isWritable: false },
      { pubkey: accounts.associatedTokenProgram, isSigner: false, isWritable: false },
      { pubkey: accounts.systemProgram, isSigner: false, isWritable: false },
      { pubkey: accounts.rent, isSigner: false, isWritable: false },
      { pubkey: accounts.clock, isSigner: false, isWritable: false },
    ],
    programId,
    data,
  });
}

/**
 * Create unstake tokens instruction
 */
export async function createUnstakeInstruction(
  user: PublicKey,
  amount: BN,
  programId: PublicKey = FINOVA_CORE_PROGRAM_ID
): Promise<TransactionInstruction> {
  const [stakingAccount] = deriveStakingAddress(user, programId);
  const [userAccount] = deriveUserAddress(user, programId);
  const [activeEffectsAccount] = deriveActiveEffectsAddress(user, programId);
  
  // Get user's FIN token account
  const userTokenAccount = await getAssociatedTokenAddress(
    new PublicKey("FINTokenMintAddress"), // Replace with actual FIN mint
    user
  );
  
  // Get staking vault token account
  const stakingVault = await getAssociatedTokenAddress(
    new PublicKey("FINTokenMintAddress"), // Replace with actual FIN mint
    stakingAccount,
    true // Allow PDA owner
  );

  const accounts = {
    user,
    userAccount,
    stakingAccount,
    activeEffectsAccount,
    userTokenAccount,
    stakingVault,
    tokenProgram: TOKEN_PROGRAM_ID,
    systemProgram: SystemProgram.programId,
    clock: SYSVAR_CLOCK_PUBKEY,
  };

  const data = Buffer.alloc(16);
  data.writeUInt32LE(5, 0); // Instruction discriminator for unstake
  data.writeBigUInt64LE(BigInt(amount.toString()), 8);

  return new TransactionInstruction({
    keys: [
      { pubkey: accounts.user, isSigner: true, isWritable: false },
      { pubkey: accounts.userAccount, isSigner: false, isWritable: true },
      { pubkey: accounts.stakingAccount, isSigner: false, isWritable: true },
      { pubkey: accounts.activeEffectsAccount, isSigner: false, isWritable: true },
      { pubkey: accounts.userTokenAccount, isSigner: false, isWritable: true },
      { pubkey: accounts.stakingVault, isSigner: false, isWritable: true },
      { pubkey: accounts.tokenProgram, isSigner: false, isWritable: false },
      { pubkey: accounts.systemProgram, isSigner: false, isWritable: false },
      { pubkey: accounts.clock, isSigner: false, isWritable: false },
    ],
    programId,
    data,
  });
}

/**
 * Create claim staking rewards instruction
 */
export async function createClaimStakingRewardsInstruction(
  user: PublicKey,
  programId: PublicKey = FINOVA_CORE_PROGRAM_ID
): Promise<TransactionInstruction> {
  const [stakingAccount] = deriveStakingAddress(user, programId);
  const [userAccount] = deriveUserAddress(user, programId);
  const [activeEffectsAccount] = deriveActiveEffectsAddress(user, programId);
  
  // Get user's FIN token account
  const userTokenAccount = await getAssociatedTokenAddress(
    new PublicKey("FINTokenMintAddress"), // Replace with actual FIN mint
    user
  );

  const accounts = {
    user,
    userAccount,
    stakingAccount,
    activeEffectsAccount,
    userTokenAccount,
    finovaTokenProgram: FINOVA_TOKEN_PROGRAM_ID,
    tokenProgram: TOKEN_PROGRAM_ID,
    systemProgram: SystemProgram.programId,
    clock: SYSVAR_CLOCK_PUBKEY,
  };

  const data = Buffer.alloc(8);
  data.writeUInt32LE(6, 0); // Instruction discriminator for claim_staking_rewards

  return new TransactionInstruction({
    keys: [
      { pubkey: accounts.user, isSigner: true, isWritable: false },
      { pubkey: accounts.userAccount, isSigner: false, isWritable: true },
      { pubkey: accounts.stakingAccount, isSigner: false, isWritable: true },
      { pubkey: accounts.activeEffectsAccount, isSigner: false, isWritable: false },
      { pubkey: accounts.userTokenAccount, isSigner: false, isWritable: true },
      { pubkey: accounts.finovaTokenProgram, isSigner: false, isWritable: false },
      { pubkey: accounts.tokenProgram, isSigner: false, isWritable: false },
      { pubkey: accounts.systemProgram, isSigner: false, isWritable: false },
      { pubkey: accounts.clock, isSigner: false, isWritable: false },
    ],
    programId,
    data,
  });
}

/**
 * Create compound staking instruction (restake rewards)
 */
export async function createCompoundStakingInstruction(
  user: PublicKey,
  programId: PublicKey = FINOVA_CORE_PROGRAM_ID
): Promise<TransactionInstruction> {
  const [stakingAccount] = deriveStakingAddress(user, programId);
  const [userAccount] = deriveUserAddress(user, programId);
  const [activeEffectsAccount] = deriveActiveEffectsAddress(user, programId);

  const accounts = {
    user,
    userAccount,
    stakingAccount,
    activeEffectsAccount,
    systemProgram: SystemProgram.programId,
    clock: SYSVAR_CLOCK_PUBKEY,
  };

  const data = Buffer.alloc(8);
  data.writeUInt32LE(7, 0); // Instruction discriminator for compound_staking

  return new TransactionInstruction({
    keys: [
      { pubkey: accounts.user, isSigner: true, isWritable: false },
      { pubkey: accounts.userAccount, isSigner: false, isWritable: true },
      { pubkey: accounts.stakingAccount, isSigner: false, isWritable: true },
      { pubkey: accounts.activeEffectsAccount, isSigner: false, isWritable: true },
      { pubkey: accounts.systemProgram, isSigner: false, isWritable: false },
      { pubkey: accounts.clock, isSigner: false, isWritable: false },
    ],
    programId,
    data,
  });
}

/**
 * Create emergency unstake instruction (with penalty)
 */
export async function createEmergencyUnstakeInstruction(
  user: PublicKey,
  programId: PublicKey = FINOVA_CORE_PROGRAM_ID
): Promise<TransactionInstruction> {
  const [stakingAccount] = deriveStakingAddress(user, programId);
  const [userAccount] = deriveUserAddress(user, programId);
  const [activeEffectsAccount] = deriveActiveEffectsAddress(user, programId);
  
  // Get user's FIN token account
  const userTokenAccount = await getAssociatedTokenAddress(
    new PublicKey("FINTokenMintAddress"), // Replace with actual FIN mint
    user
  );
  
  // Get staking vault token account
  const stakingVault = await getAssociatedTokenAddress(
    new PublicKey("FINTokenMintAddress"), // Replace with actual FIN mint
    stakingAccount,
    true // Allow PDA owner
  );

  const accounts = {
    user,
    userAccount,
    stakingAccount,
    activeEffectsAccount,
    userTokenAccount,
    stakingVault,
    tokenProgram: TOKEN_PROGRAM_ID,
    systemProgram: SystemProgram.programId,
    clock: SYSVAR_CLOCK_PUBKEY,
  };

  const data = Buffer.alloc(8);
  data.writeUInt32LE(8, 0); // Instruction discriminator for emergency_unstake

  return new TransactionInstruction({
    keys: [
      { pubkey: accounts.user, isSigner: true, isWritable: false },
      { pubkey: accounts.userAccount, isSigner: false, isWritable: true },
      { pubkey: accounts.stakingAccount, isSigner: false, isWritable: true },
      { pubkey: accounts.activeEffectsAccount, isSigner: false, isWritable: true },
      { pubkey: accounts.userTokenAccount, isSigner: false, isWritable: true },
      { pubkey: accounts.stakingVault, isSigner: false, isWritable: true },
      { pubkey: accounts.tokenProgram, isSigner: false, isWritable: false },
      { pubkey: accounts.systemProgram, isSigner: false, isWritable: false },
      { pubkey: accounts.clock, isSigner: false, isWritable: false },
    ],
    programId,
    data,
  });
}

/**
 * High-level staking transaction builders
 */

/**
 * Build stake transaction
 */
export async function buildStakeTransaction(
  user: PublicKey,
  amount: BN,
  programId: PublicKey = FINOVA_CORE_PROGRAM_ID
): Promise<Transaction> {
  const transaction = new Transaction();
  
  // Add stake instruction
  const stakeInstruction = await createStakeInstruction(user, amount, programId);
  transaction.add(stakeInstruction);
  
  return transaction;
}

/**
 * Build unstake transaction
 */
export async function buildUnstakeTransaction(
  user: PublicKey,
  amount: BN,
  programId: PublicKey = FINOVA_CORE_PROGRAM_ID
): Promise<Transaction> {
  const transaction = new Transaction();
  
  // Add unstake instruction
  const unstakeInstruction = await createUnstakeInstruction(user, amount, programId);
  transaction.add(unstakeInstruction);
  
  return transaction;
}

/**
 * Build claim rewards transaction
 */
export async function buildClaimRewardsTransaction(
  user: PublicKey,
  programId: PublicKey = FINOVA_CORE_PROGRAM_ID
): Promise<Transaction> {
  const transaction = new Transaction();
  
  // Add claim rewards instruction
  const claimInstruction = await createClaimStakingRewardsInstruction(user, programId);
  transaction.add(claimInstruction);
  
  return transaction;
}

/**
 * Build compound staking transaction
 */
export async function buildCompoundTransaction(
  user: PublicKey,
  programId: PublicKey = FINOVA_CORE_PROGRAM_ID
): Promise<Transaction> {
  const transaction = new Transaction();
  
  // Add compound instruction
  const compoundInstruction = await createCompoundStakingInstruction(user, programId);
  transaction.add(compoundInstruction);
  
  return transaction;
}

/**
 * Build emergency unstake transaction
 */
export async function buildEmergencyUnstakeTransaction(
  user: PublicKey,
  programId: PublicKey = FINOVA_CORE_PROGRAM_ID
): Promise<Transaction> {
  const transaction = new Transaction();
  
  // Add emergency unstake instruction
  const emergencyInstruction = await createEmergencyUnstakeInstruction(user, programId);
  transaction.add(emergencyInstruction);
  
  return transaction;
}

/**
 * Utility functions for staking calculations
 */

/**
 * Calculate minimum staking period in days
 */
export function getMinimumStakingPeriod(tier: StakingTier): number {
  const minimumPeriods = {
    [StakingTier.BASIC]: 7,        // 7 days
    [StakingTier.PREMIUM]: 14,     // 14 days
    [StakingTier.VIP]: 30,         // 30 days
    [StakingTier.GUILD_MASTER]: 60, // 60 days
    [StakingTier.DAO_MEMBER]: 90   // 90 days
  };
  
  return minimumPeriods[tier];
}

/**
 * Calculate early unstaking penalty
 */
export function calculateEarlyUnstakingPenalty(
  stakedAmount: BN,
  stakingDays: number,
  tier: StakingTier
): BN {
  const minimumDays = getMinimumStakingPeriod(tier);
  
  if (stakingDays >= minimumDays) {
    return new BN(0); // No penalty
  }
  
  // Penalty = 5% to 20% based on how early
  const penaltyRate = Math.min(20, 5 + (minimumDays - stakingDays));
  return stakedAmount.mul(new BN(penaltyRate)).div(new BN(100));
}

/**
 * Calculate cooldown period for unstaking
 */
export function calculateUnstakingCooldown(tier: StakingTier): number {
  const cooldownPeriods = {
    [StakingTier.BASIC]: 1,        // 1 day
    [StakingTier.PREMIUM]: 2,      // 2 days
    [StakingTier.VIP]: 3,          // 3 days
    [StakingTier.GUILD_MASTER]: 5, // 5 days
    [StakingTier.DAO_MEMBER]: 7    // 7 days
  };
  
  return cooldownPeriods[tier];
}

/**
 * Validate staking amount
 */
export function validateStakingAmount(amount: BN): {
  isValid: boolean;
  error?: string;
  suggestedTier?: StakingTier;
} {
  if (amount.lte(new BN(0))) {
    return {
      isValid: false,
      error: 'Staking amount must be greater than 0'
    };
  }
  
  const minStakeAmount = new BN(100_000_000); // 100 $FIN minimum
  if (amount.lt(minStakeAmount)) {
    return {
      isValid: false,
      error: 'Minimum staking amount is 100 $FIN',
      suggestedTier: StakingTier.BASIC
    };
  }
  
  const tier = calculateStakingTier(amount);
  return {
    isValid: true,
    suggestedTier: tier
  };
}

/**
 * Get staking tier name
 */
export function getStakingTierName(tier: StakingTier): string {
  const names = {
    [StakingTier.BASIC]: 'Basic',
    [StakingTier.PREMIUM]: 'Premium',
    [StakingTier.VIP]: 'VIP',
    [StakingTier.GUILD_MASTER]: 'Guild Master',
    [StakingTier.DAO_MEMBER]: 'DAO Member'
  };
  
  return names[tier];
}

/**
 * Export all staking-related types and functions
 */
export * from './types';

export default {
  // Enums
  StakingTier,
  
  // Constants
  STAKING_TIER_BENEFITS,
  
  // Calculation functions
  calculateStakingTier,
  calculateStakingRewards,
  calculateLoyaltyBonus,
  calculateTotalStakingMultiplier,
  calculateEarlyUnstakingPenalty,
  calculateUnstakingCooldown,
  
  // Address derivation
  deriveStakingAddress,
  deriveUserAddress,
  deriveActiveEffectsAddress,
  
  // Instruction builders
  createStakeInstruction,
  createUnstakeInstruction,
  createClaimStakingRewardsInstruction,
  createCompoundStakingInstruction,
  createEmergencyUnstakeInstruction,
  
  // Transaction builders
  buildStakeTransaction,
  buildUnstakeTransaction,
  buildClaimRewardsTransaction,
  buildCompoundTransaction,
  buildEmergencyUnstakeTransaction,
  
  // Utility functions
  getMinimumStakingPeriod,
  validateStakingAmount,
  getStakingTierName,
};
