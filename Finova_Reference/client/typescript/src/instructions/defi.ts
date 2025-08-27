// client/typescript/src/instructions/defi.ts

/**
 * Finova Network - DeFi Instructions Module
 * Enterprise-grade TypeScript SDK for DeFi operations
 * 
 * @version 1.0.0
 * @author Finova Network Team
 * @license MIT
 * 
 * This module provides comprehensive DeFi instruction builders for:
 * - Automated Market Maker (AMM) operations
 * - Liquidity provision and management
 * - Yield farming and staking rewards
 * - Flash loans and arbitrage
 * - Cross-program integration with finova-core
 */

import {
  Connection,
  PublicKey,
  Transaction,
  TransactionInstruction,
  SystemProgram,
  SYSVAR_RENT_PUBKEY,
  SYSVAR_CLOCK_PUBKEY,
  AccountMeta,
  Keypair,
  LAMPORTS_PER_SOL
} from '@solana/web3.js';
import {
  TOKEN_PROGRAM_ID,
  ASSOCIATED_TOKEN_PROGRAM_ID,
  getAssociatedTokenAddress,
  createAssociatedTokenAccountInstruction
} from '@solana/spl-token';
import { BN } from '@project-serum/anchor';
import { Buffer } from 'buffer';

// ============================================================================
// Constants & Configuration
// ============================================================================

export const FINOVA_DEFI_PROGRAM_ID = new PublicKey('FinovaDeFi11111111111111111111111111111111');
export const FINOVA_CORE_PROGRAM_ID = new PublicKey('FinovaCore11111111111111111111111111111111');
export const FINOVA_TOKEN_PROGRAM_ID = new PublicKey('FinovaToken1111111111111111111111111111111');

// DeFi Pool Configuration
export const POOL_SEED = 'pool';
export const LIQUIDITY_SEED = 'liquidity';
export const FARM_SEED = 'farm';
export const VAULT_SEED = 'vault';
export const ORACLE_SEED = 'oracle';

// Fee constants (in basis points)
export const DEFAULT_TRADE_FEE_BPS = 30; // 0.3%
export const DEFAULT_PROTOCOL_FEE_BPS = 5; // 0.05%
export const LP_FEE_BPS = 25; // 0.25%

// Curve constants
export const CURVE_CONSTANT = new BN(1000000);
export const MIN_LIQUIDITY = new BN(1000);
export const MAX_SLIPPAGE_BPS = 500; // 5%

// ============================================================================
// Types & Interfaces
// ============================================================================

export interface PoolInfo {
  address: PublicKey;
  tokenAMint: PublicKey;
  tokenBMint: PublicKey;
  tokenAReserve: PublicKey;
  tokenBReserve: PublicKey;
  lpMint: PublicKey;
  feeRecipient: PublicKey;
  tradeFee: number;
  protocolFee: number;
  curveType: CurveType;
}

export interface LiquidityPosition {
  address: PublicKey;
  pool: PublicKey;
  owner: PublicKey;
  lpTokens: BN;
  rewardDebt: BN;
  lastUpdateSlot: BN;
}

export interface FarmInfo {
  address: PublicKey;
  pool: PublicKey;
  rewardMint: PublicKey;
  rewardVault: PublicKey;
  rewardPerSlot: BN;
  totalLiquidity: BN;
  accRewardPerShare: BN;
  lastRewardSlot: BN;
}

export interface YieldFarmPosition {
  address: PublicKey;
  farm: PublicKey;
  owner: PublicKey;
  stakedAmount: BN;
  rewardDebt: BN;
  pendingRewards: BN;
}

export interface FlashLoanInfo {
  address: PublicKey;
  borrower: PublicKey;
  amount: BN;
  fee: BN;
  expiry: BN;
  repaid: boolean;
}

export enum CurveType {
  ConstantProduct = 0,
  Stable = 1,
  Concentrated = 2
}

export enum PoolStatus {
  Active = 0,
  Paused = 1,
  Deprecated = 2
}

// ============================================================================
// Instruction Data Structures
// ============================================================================

export interface CreatePoolData {
  tradeFee: number;
  protocolFee: number;
  curveType: CurveType;
  initialTokenAAmount: BN;
  initialTokenBAmount: BN;
}

export interface AddLiquidityData {
  tokenAAmount: BN;
  tokenBAmount: BN;
  minLpTokens: BN;
  slippageBps: number;
}

export interface RemoveLiquidityData {
  lpTokens: BN;
  minTokenAAmount: BN;
  minTokenBAmount: BN;
}

export interface SwapData {
  amountIn: BN;
  minAmountOut: BN;
  slippageBps: number;
  isTokenAToB: boolean;
}

export interface YieldFarmData {
  rewardPerSlot: BN;
  startSlot: BN;
  endSlot: BN;
}

export interface FlashLoanData {
  amount: BN;
  fee: BN;
  instructions: TransactionInstruction[];
}

// ============================================================================
// PDA Derivation Utilities
// ============================================================================

export class PDAUtils {
  /**
   * Derive pool PDA address
   */
  static async getPoolAddress(
    tokenAMint: PublicKey,
    tokenBMint: PublicKey,
    programId: PublicKey = FINOVA_DEFI_PROGRAM_ID
  ): Promise<[PublicKey, number]> {
    const [tokenA, tokenB] = tokenAMint.toBuffer().compare(tokenBMint.toBuffer()) < 0 
      ? [tokenAMint, tokenBMint] 
      : [tokenBMint, tokenAMint];
    
    return PublicKey.findProgramAddress(
      [
        Buffer.from(POOL_SEED),
        tokenA.toBuffer(),
        tokenB.toBuffer()
      ],
      programId
    );
  }

  /**
   * Derive liquidity position PDA address
   */
  static async getLiquidityPositionAddress(
    pool: PublicKey,
    owner: PublicKey,
    programId: PublicKey = FINOVA_DEFI_PROGRAM_ID
  ): Promise<[PublicKey, number]> {
    return PublicKey.findProgramAddress(
      [
        Buffer.from(LIQUIDITY_SEED),
        pool.toBuffer(),
        owner.toBuffer()
      ],
      programId
    );
  }

  /**
   * Derive farm PDA address
   */
  static async getFarmAddress(
    pool: PublicKey,
    rewardMint: PublicKey,
    programId: PublicKey = FINOVA_DEFI_PROGRAM_ID
  ): Promise<[PublicKey, number]> {
    return PublicKey.findProgramAddress(
      [
        Buffer.from(FARM_SEED),
        pool.toBuffer(),
        rewardMint.toBuffer()
      ],
      programId
    );
  }

  /**
   * Derive vault PDA address
   */
  static async getVaultAddress(
    pool: PublicKey,
    mint: PublicKey,
    programId: PublicKey = FINOVA_DEFI_PROGRAM_ID
  ): Promise<[PublicKey, number]> {
    return PublicKey.findProgramAddress(
      [
        Buffer.from(VAULT_SEED),
        pool.toBuffer(),
        mint.toBuffer()
      ],
      programId
    );
  }
}

// ============================================================================
// Math Utilities for AMM Calculations
// ============================================================================

export class AMMUtils {
  /**
   * Calculate constant product AMM output
   */
  static calculateSwapOutput(
    amountIn: BN,
    reserveIn: BN,
    reserveOut: BN,
    feeBps: number = DEFAULT_TRADE_FEE_BPS
  ): BN {
    if (amountIn.isZero() || reserveIn.isZero() || reserveOut.isZero()) {
      return new BN(0);
    }

    const feeAmount = amountIn.mul(new BN(feeBps)).div(new BN(10000));
    const amountInAfterFee = amountIn.sub(feeAmount);
    
    const numerator = amountInAfterFee.mul(reserveOut);
    const denominator = reserveIn.add(amountInAfterFee);
    
    return numerator.div(denominator);
  }

  /**
   * Calculate required input for desired output
   */
  static calculateSwapInput(
    amountOut: BN,
    reserveIn: BN,
    reserveOut: BN,
    feeBps: number = DEFAULT_TRADE_FEE_BPS
  ): BN {
    if (amountOut.isZero() || reserveIn.isZero() || reserveOut.isZero()) {
      return new BN(0);
    }

    if (amountOut.gte(reserveOut)) {
      throw new Error('Insufficient liquidity');
    }

    const numerator = reserveIn.mul(amountOut);
    const denominator = reserveOut.sub(amountOut);
    const amountInBeforeFee = numerator.div(denominator);
    
    const feeMultiplier = new BN(10000).sub(new BN(feeBps));
    return amountInBeforeFee.mul(new BN(10000)).div(feeMultiplier);
  }

  /**
   * Calculate LP tokens to mint for liquidity addition
   */
  static calculateLPTokensToMint(
    tokenAAmount: BN,
    tokenBAmount: BN,
    reserveA: BN,
    reserveB: BN,
    totalSupply: BN
  ): BN {
    if (totalSupply.isZero()) {
      // Initial liquidity
      return tokenAAmount.mul(tokenBAmount).sqrt().sub(MIN_LIQUIDITY);
    }

    const lpFromA = tokenAAmount.mul(totalSupply).div(reserveA);
    const lpFromB = tokenBAmount.mul(totalSupply).div(reserveB);
    
    return BN.min(lpFromA, lpFromB);
  }

  /**
   * Calculate tokens to receive for LP burn
   */
  static calculateTokensFromLP(
    lpAmount: BN,
    reserveA: BN,
    reserveB: BN,
    totalSupply: BN
  ): [BN, BN] {
    const tokenAAmount = lpAmount.mul(reserveA).div(totalSupply);
    const tokenBAmount = lpAmount.mul(reserveB).div(totalSupply);
    
    return [tokenAAmount, tokenBAmount];
  }

  /**
   * Calculate price impact for a swap
   */
  static calculatePriceImpact(
    amountIn: BN,
    reserveIn: BN,
    reserveOut: BN,
    amountOut: BN
  ): number {
    const priceBefore = reserveOut.mul(new BN(1e9)).div(reserveIn);
    const newReserveIn = reserveIn.add(amountIn);
    const newReserveOut = reserveOut.sub(amountOut);
    const priceAfter = newReserveOut.mul(new BN(1e9)).div(newReserveIn);
    
    const impact = priceBefore.sub(priceAfter).mul(new BN(10000)).div(priceBefore);
    return impact.toNumber();
  }
}

// ============================================================================
// Core DeFi Instruction Builders
// ============================================================================

export class DeFiInstructions {
  
  /**
   * Create a new AMM pool
   */
  static async createPool(
    connection: Connection,
    payer: PublicKey,
    tokenAMint: PublicKey,
    tokenBMint: PublicKey,
    data: CreatePoolData,
    programId: PublicKey = FINOVA_DEFI_PROGRAM_ID
  ): Promise<TransactionInstruction> {
    const [poolAddress] = await PDAUtils.getPoolAddress(tokenAMint, tokenBMint, programId);
    const [tokenAVault] = await PDAUtils.getVaultAddress(poolAddress, tokenAMint, programId);
    const [tokenBVault] = await PDAUtils.getVaultAddress(poolAddress, tokenBMint, programId);
    
    // Create LP mint
    const lpMint = Keypair.generate();
    
    const instructionData = Buffer.concat([
      Buffer.from([0]), // CreatePool instruction discriminator
      Buffer.from(new Uint16Array([data.tradeFee]).buffer),
      Buffer.from(new Uint16Array([data.protocolFee]).buffer),
      Buffer.from([data.curveType]),
      data.initialTokenAAmount.toArrayLike(Buffer, 'le', 8),
      data.initialTokenBAmount.toArrayLike(Buffer, 'le', 8)
    ]);

    const accounts: AccountMeta[] = [
      { pubkey: payer, isSigner: true, isWritable: true },
      { pubkey: poolAddress, isSigner: false, isWritable: true },
      { pubkey: tokenAMint, isSigner: false, isWritable: false },
      { pubkey: tokenBMint, isSigner: false, isWritable: false },
      { pubkey: tokenAVault, isSigner: false, isWritable: true },
      { pubkey: tokenBVault, isSigner: false, isWritable: true },
      { pubkey: lpMint.publicKey, isSigner: true, isWritable: true },
      { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
      { pubkey: TOKEN_PROGRAM_ID, isSigner: false, isWritable: false },
      { pubkey: SYSVAR_RENT_PUBKEY, isSigner: false, isWritable: false }
    ];

    return new TransactionInstruction({
      keys: accounts,
      programId,
      data: instructionData
    });
  }

  /**
   * Add liquidity to an existing pool
   */
  static async addLiquidity(
    connection: Connection,
    user: PublicKey,
    poolAddress: PublicKey,
    tokenAMint: PublicKey,
    tokenBMint: PublicKey,
    data: AddLiquidityData,
    programId: PublicKey = FINOVA_DEFI_PROGRAM_ID
  ): Promise<TransactionInstruction> {
    const userTokenAAccount = await getAssociatedTokenAddress(tokenAMint, user);
    const userTokenBAccount = await getAssociatedTokenAddress(tokenBMint, user);
    const [liquidityPosition] = await PDAUtils.getLiquidityPositionAddress(poolAddress, user, programId);
    const [tokenAVault] = await PDAUtils.getVaultAddress(poolAddress, tokenAMint, programId);
    const [tokenBVault] = await PDAUtils.getVaultAddress(poolAddress, tokenBMint, programId);

    const instructionData = Buffer.concat([
      Buffer.from([1]), // AddLiquidity instruction discriminator
      data.tokenAAmount.toArrayLike(Buffer, 'le', 8),
      data.tokenBAmount.toArrayLike(Buffer, 'le', 8),
      data.minLpTokens.toArrayLike(Buffer, 'le', 8),
      Buffer.from(new Uint16Array([data.slippageBps]).buffer)
    ]);

    const accounts: AccountMeta[] = [
      { pubkey: user, isSigner: true, isWritable: true },
      { pubkey: poolAddress, isSigner: false, isWritable: true },
      { pubkey: liquidityPosition, isSigner: false, isWritable: true },
      { pubkey: userTokenAAccount, isSigner: false, isWritable: true },
      { pubkey: userTokenBAccount, isSigner: false, isWritable: true },
      { pubkey: tokenAVault, isSigner: false, isWritable: true },
      { pubkey: tokenBVault, isSigner: false, isWritable: true },
      { pubkey: TOKEN_PROGRAM_ID, isSigner: false, isWritable: false },
      { pubkey: SYSVAR_CLOCK_PUBKEY, isSigner: false, isWritable: false }
    ];

    return new TransactionInstruction({
      keys: accounts,
      programId,
      data: instructionData
    });
  }

  /**
   * Remove liquidity from a pool
   */
  static async removeLiquidity(
    connection: Connection,
    user: PublicKey,
    poolAddress: PublicKey,
    tokenAMint: PublicKey,
    tokenBMint: PublicKey,
    data: RemoveLiquidityData,
    programId: PublicKey = FINOVA_DEFI_PROGRAM_ID
  ): Promise<TransactionInstruction> {
    const userTokenAAccount = await getAssociatedTokenAddress(tokenAMint, user);
    const userTokenBAccount = await getAssociatedTokenAddress(tokenBMint, user);
    const [liquidityPosition] = await PDAUtils.getLiquidityPositionAddress(poolAddress, user, programId);
    const [tokenAVault] = await PDAUtils.getVaultAddress(poolAddress, tokenAMint, programId);
    const [tokenBVault] = await PDAUtils.getVaultAddress(poolAddress, tokenBMint, programId);

    const instructionData = Buffer.concat([
      Buffer.from([2]), // RemoveLiquidity instruction discriminator
      data.lpTokens.toArrayLike(Buffer, 'le', 8),
      data.minTokenAAmount.toArrayLike(Buffer, 'le', 8),
      data.minTokenBAmount.toArrayLike(Buffer, 'le', 8)
    ]);

    const accounts: AccountMeta[] = [
      { pubkey: user, isSigner: true, isWritable: true },
      { pubkey: poolAddress, isSigner: false, isWritable: true },
      { pubkey: liquidityPosition, isSigner: false, isWritable: true },
      { pubkey: userTokenAAccount, isSigner: false, isWritable: true },
      { pubkey: userTokenBAccount, isSigner: false, isWritable: true },
      { pubkey: tokenAVault, isSigner: false, isWritable: true },
      { pubkey: tokenBVault, isSigner: false, isWritable: true },
      { pubkey: TOKEN_PROGRAM_ID, isSigner: false, isWritable: false }
    ];

    return new TransactionInstruction({
      keys: accounts,
      programId,
      data: instructionData
    });
  }

  /**
   * Execute a token swap
   */
  static async swap(
    connection: Connection,
    user: PublicKey,
    poolAddress: PublicKey,
    tokenAMint: PublicKey,
    tokenBMint: PublicKey,
    data: SwapData,
    programId: PublicKey = FINOVA_DEFI_PROGRAM_ID
  ): Promise<TransactionInstruction> {
    const [inputMint, outputMint] = data.isTokenAToB ? [tokenAMint, tokenBMint] : [tokenBMint, tokenAMint];
    const userInputAccount = await getAssociatedTokenAddress(inputMint, user);
    const userOutputAccount = await getAssociatedTokenAddress(outputMint, user);
    const [inputVault] = await PDAUtils.getVaultAddress(poolAddress, inputMint, programId);
    const [outputVault] = await PDAUtils.getVaultAddress(poolAddress, outputMint, programId);

    const instructionData = Buffer.concat([
      Buffer.from([3]), // Swap instruction discriminator
      data.amountIn.toArrayLike(Buffer, 'le', 8),
      data.minAmountOut.toArrayLike(Buffer, 'le', 8),
      Buffer.from(new Uint16Array([data.slippageBps]).buffer),
      Buffer.from([data.isTokenAToB ? 1 : 0])
    ]);

    const accounts: AccountMeta[] = [
      { pubkey: user, isSigner: true, isWritable: true },
      { pubkey: poolAddress, isSigner: false, isWritable: true },
      { pubkey: userInputAccount, isSigner: false, isWritable: true },
      { pubkey: userOutputAccount, isSigner: false, isWritable: true },
      { pubkey: inputVault, isSigner: false, isWritable: true },
      { pubkey: outputVault, isSigner: false, isWritable: true },
      { pubkey: TOKEN_PROGRAM_ID, isSigner: false, isWritable: false },
      { pubkey: SYSVAR_CLOCK_PUBKEY, isSigner: false, isWritable: false }
    ];

    return new TransactionInstruction({
      keys: accounts,
      programId,
      data: instructionData
    });
  }

  /**
   * Create a yield farm for a pool
   */
  static async createYieldFarm(
    connection: Connection,
    authority: PublicKey,
    poolAddress: PublicKey,
    rewardMint: PublicKey,
    data: YieldFarmData,
    programId: PublicKey = FINOVA_DEFI_PROGRAM_ID
  ): Promise<TransactionInstruction> {
    const [farmAddress] = await PDAUtils.getFarmAddress(poolAddress, rewardMint, programId);
    const [rewardVault] = await PDAUtils.getVaultAddress(farmAddress, rewardMint, programId);

    const instructionData = Buffer.concat([
      Buffer.from([4]), // CreateYieldFarm instruction discriminator
      data.rewardPerSlot.toArrayLike(Buffer, 'le', 8),
      data.startSlot.toArrayLike(Buffer, 'le', 8),
      data.endSlot.toArrayLike(Buffer, 'le', 8)
    ]);

    const accounts: AccountMeta[] = [
      { pubkey: authority, isSigner: true, isWritable: true },
      { pubkey: farmAddress, isSigner: false, isWritable: true },
      { pubkey: poolAddress, isSigner: false, isWritable: false },
      { pubkey: rewardMint, isSigner: false, isWritable: false },
      { pubkey: rewardVault, isSigner: false, isWritable: true },
      { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
      { pubkey: TOKEN_PROGRAM_ID, isSigner: false, isWritable: false },
      { pubkey: SYSVAR_RENT_PUBKEY, isSigner: false, isWritable: false }
    ];

    return new TransactionInstruction({
      keys: accounts,
      programId,
      data: instructionData
    });
  }

  /**
   * Stake LP tokens in a yield farm
   */
  static async stakeLPTokens(
    connection: Connection,
    user: PublicKey,
    farmAddress: PublicKey,
    poolAddress: PublicKey,
    lpAmount: BN,
    programId: PublicKey = FINOVA_DEFI_PROGRAM_ID
  ): Promise<TransactionInstruction> {
    const [yieldPosition] = await PDAUtils.getLiquidityPositionAddress(farmAddress, user, programId);
    const [liquidityPosition] = await PDAUtils.getLiquidityPositionAddress(poolAddress, user, programId);

    const instructionData = Buffer.concat([
      Buffer.from([5]), // StakeLPTokens instruction discriminator
      lpAmount.toArrayLike(Buffer, 'le', 8)
    ]);

    const accounts: AccountMeta[] = [
      { pubkey: user, isSigner: true, isWritable: true },
      { pubkey: farmAddress, isSigner: false, isWritable: true },
      { pubkey: yieldPosition, isSigner: false, isWritable: true },
      { pubkey: liquidityPosition, isSigner: false, isWritable: true },
      { pubkey: SYSVAR_CLOCK_PUBKEY, isSigner: false, isWritable: false }
    ];

    return new TransactionInstruction({
      keys: accounts,
      programId,
      data: instructionData
    });
  }

  /**
   * Harvest yield farm rewards
   */
  static async harvestRewards(
    connection: Connection,
    user: PublicKey,
    farmAddress: PublicKey,
    rewardMint: PublicKey,
    programId: PublicKey = FINOVA_DEFI_PROGRAM_ID
  ): Promise<TransactionInstruction> {
    const [yieldPosition] = await PDAUtils.getLiquidityPositionAddress(farmAddress, user, programId);
    const [rewardVault] = await PDAUtils.getVaultAddress(farmAddress, rewardMint, programId);
    const userRewardAccount = await getAssociatedTokenAddress(rewardMint, user);

    const instructionData = Buffer.concat([
      Buffer.from([6]) // HarvestRewards instruction discriminator
    ]);

    const accounts: AccountMeta[] = [
      { pubkey: user, isSigner: true, isWritable: true },
      { pubkey: farmAddress, isSigner: false, isWritable: true },
      { pubkey: yieldPosition, isSigner: false, isWritable: true },
      { pubkey: rewardVault, isSigner: false, isWritable: true },
      { pubkey: userRewardAccount, isSigner: false, isWritable: true },
      { pubkey: TOKEN_PROGRAM_ID, isSigner: false, isWritable: false },
      { pubkey: SYSVAR_CLOCK_PUBKEY, isSigner: false, isWritable: false }
    ];

    return new TransactionInstruction({
      keys: accounts,
      programId,
      data: instructionData
    });
  }

  /**
   * Execute a flash loan
   */
  static async flashLoan(
    connection: Connection,
    borrower: PublicKey,
    poolAddress: PublicKey,
    tokenMint: PublicKey,
    data: FlashLoanData,
    programId: PublicKey = FINOVA_DEFI_PROGRAM_ID
  ): Promise<TransactionInstruction> {
    const [vault] = await PDAUtils.getVaultAddress(poolAddress, tokenMint, programId);
    const borrowerTokenAccount = await getAssociatedTokenAddress(tokenMint, borrower);

    const instructionData = Buffer.concat([
      Buffer.from([7]), // FlashLoan instruction discriminator
      data.amount.toArrayLike(Buffer, 'le', 8),
      data.fee.toArrayLike(Buffer, 'le', 8),
      Buffer.from(new Uint16Array([data.instructions.length]).buffer)
    ]);

    const accounts: AccountMeta[] = [
      { pubkey: borrower, isSigner: true, isWritable: true },
      { pubkey: poolAddress, isSigner: false, isWritable: true },
      { pubkey: vault, isSigner: false, isWritable: true },
      { pubkey: borrowerTokenAccount, isSigner: false, isWritable: true },
      { pubkey: TOKEN_PROGRAM_ID, isSigner: false, isWritable: false },
      { pubkey: SYSVAR_CLOCK_PUBKEY, isSigner: false, isWritable: false }
    ];

    return new TransactionInstruction({
      keys: accounts,
      programId,
      data: instructionData
    });
  }

  /**
   * Update pool parameters (admin only)
   */
  static async updatePoolParameters(
    connection: Connection,
    authority: PublicKey,
    poolAddress: PublicKey,
    newTradeFee?: number,
    newProtocolFee?: number,
    newStatus?: PoolStatus,
    programId: PublicKey = FINOVA_DEFI_PROGRAM_ID
  ): Promise<TransactionInstruction> {
    const instructionData = Buffer.concat([
      Buffer.from([8]), // UpdatePoolParameters instruction discriminator
      Buffer.from([newTradeFee !== undefined ? 1 : 0]),
      newTradeFee !== undefined ? Buffer.from(new Uint16Array([newTradeFee]).buffer) : Buffer.alloc(2),
      Buffer.from([newProtocolFee !== undefined ? 1 : 0]),
      newProtocolFee !== undefined ? Buffer.from(new Uint16Array([newProtocolFee]).buffer) : Buffer.alloc(2),
      Buffer.from([newStatus !== undefined ? 1 : 0]),
      newStatus !== undefined ? Buffer.from([newStatus]) : Buffer.alloc(1)
    ]);

    const accounts: AccountMeta[] = [
      { pubkey: authority, isSigner: true, isWritable: true },
      { pubkey: poolAddress, isSigner: false, isWritable: true }
    ];

    return new TransactionInstruction({
      keys: accounts,
      programId,
      data: instructionData
    });
  }

  /**
   * Pause/unpause pool operations (emergency)
   */
  static async emergencyPause(
    connection: Connection,
    authority: PublicKey,
    poolAddress: PublicKey,
    pause: boolean,
    programId: PublicKey = FINOVA_DEFI_PROGRAM_ID
  ): Promise<TransactionInstruction> {
    const instructionData = Buffer.concat([
      Buffer.from([9]), // EmergencyPause instruction discriminator
      Buffer.from([pause ? 1 : 0])
    ]);

    const accounts: AccountMeta[] = [
      { pubkey: authority, isSigner: true, isWritable: true },
      { pubkey: poolAddress, isSigner: false, isWritable: true },
      { pubkey: SYSVAR_CLOCK_PUBKEY, isSigner: false, isWritable: false }
    ];

    return new TransactionInstruction({
      keys: accounts,
      programId,
      data: instructionData
    });
  }
}

// ============================================================================
// Cross-Program Integration with Finova Core
// ============================================================================

export class FinovaCoreIntegration {
  
  /**
   * Integrate DeFi activity with XP system
   */
  static async reportDeFiActivity(
    connection: Connection,
    user: PublicKey,
    activityType: DeFiActivityType,
    amount: BN,
    coreProgram: PublicKey = FINOVA_CORE_PROGRAM_ID
  ): Promise<TransactionInstruction> {
    const instructionData = Buffer.concat([
      Buffer.from([255]), // ReportDeFiActivity CPI instruction discriminator
      Buffer.from([activityType]),
      amount.toArrayLike(Buffer, 'le', 8)
    ]);

    const accounts: AccountMeta[] = [
      { pubkey: user, isSigner: true, isWritable: true },
      { pubkey: coreProgram, isSigner: false, isWritable: false }
    ];

    return new TransactionInstruction({
      keys: accounts,
      programId: FINOVA_DEFI_PROGRAM_ID,
      data: instructionData
    });
  }

  /**
   * Apply DeFi bonuses based on user's XP level
   */
  static async applyXPBonus(
    connection: Connection,
    user: PublicKey,
    poolAddress: PublicKey,
    baseAmount: BN,
    coreProgram: PublicKey = FINOVA_CORE_PROGRAM_ID
  ): Promise<BN> {
    // This would typically fetch user's XP state from finova-core
    // and calculate bonus multiplier
    try {
      // Mock implementation - in production, this would fetch actual XP data
      const xpLevel = 25; // Would be fetched from finova-core
      const bonusMultiplier = Math.min(1 + (xpLevel * 0.01), 2.0); // Max 2x bonus
      
      return baseAmount.mul(new BN(Math.floor(bonusMultiplier * 100))).div(new BN(100));
    } catch (error) {
      console.warn('Failed to apply XP bonus, using base amount:', error);
      return baseAmount;
    }
  }

  /**
   * Trigger mining rewards for DeFi participation
   */
  static async triggerMiningBonus(
    connection: Connection,
    user: PublicKey,
    activityValue: BN,
    coreProgram: PublicKey = FINOVA_CORE_PROGRAM_ID
  ): Promise<TransactionInstruction> {
    const instructionData = Buffer.concat([
      Buffer.from([254]), // TriggerMiningBonus CPI instruction discriminator
      activityValue.toArrayLike(Buffer, 'le', 8)
    ]);

    const accounts: AccountMeta[] = [
      { pubkey: user, isSigner: true, isWritable: true },
      { pubkey: coreProgram, isSigner: false, isWritable: false }
    ];

    return new TransactionInstruction({
      keys: accounts,
      programId: FINOVA_DEFI_PROGRAM_ID,
      data: instructionData
    });
  }
}

// ============================================================================
// Advanced DeFi Features
// ============================================================================

export enum DeFiActivityType {
  AddLiquidity = 0,
  RemoveLiquidity = 1,
  Swap = 2,
  YieldFarm = 3,
  FlashLoan = 4
}

export class AdvancedDeFiOperations {
  
  /**
   * Create concentrated liquidity position
   */
  static async createConcentratedPosition(
    connection: Connection,
    user: PublicKey,
    poolAddress: PublicKey,
    lowerTick: number,
    upperTick: number,
    liquidity: BN,
    programId: PublicKey = FINOVA_DEFI_PROGRAM_ID
  ): Promise<TransactionInstruction> {
    const [positionAddress] = await PublicKey.findProgramAddress(
      [
        Buffer.from('concentrated_position'),
        poolAddress.toBuffer(),
        user.toBuffer(),
        Buffer.from(new Int32Array([lowerTick]).buffer),
        Buffer.from(new Int32Array([upperTick]).buffer)
      ],
      programId
    );

    const instructionData = Buffer.concat([
      Buffer.from([10]), // CreateConcentratedPosition instruction discriminator
      Buffer.from(new Int32Array([lowerTick]).buffer),
      Buffer.from(new Int32Array([upperTick]).buffer),
      liquidity.toArrayLike(Buffer, 'le', 16)
    ]);

    const accounts: AccountMeta[] = [
      { pubkey: user, isSigner: true, isWritable: true },
      { pubkey: poolAddress, isSigner: false, isWritable: true },
      { pubkey: positionAddress, isSigner: false, isWritable: true },
      { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
      { pubkey: SYSVAR_RENT_PUBKEY, isSigner: false, isWritable: false }
    ];

    return new TransactionInstruction({
      keys: accounts,
      programId,
      data: instructionData
    });
  }

  /**
   * Execute multi-hop swap across multiple pools
   */
  static async multiHopSwap(
    connection: Connection,
    user: PublicKey,
    swapPath: PublicKey[],
    amountIn: BN,
    minAmountOut: BN,
    programId: PublicKey = FINOVA_DEFI_PROGRAM_ID
  ): Promise<TransactionInstruction> {
    if (swapPath.length < 2) {
      throw new Error('Swap path must contain at least 2 pools');
    }

    const instructionData = Buffer.concat([
      Buffer.from([11]), // MultiHopSwap instruction discriminator
      amountIn.toArrayLike(Buffer, 'le', 8),
      minAmountOut.toArrayLike(Buffer, 'le', 8),
      Buffer.from([swapPath.length])
    ]);

    const accounts: AccountMeta[] = [
      { pubkey: user, isSigner: true, isWritable: true },
      ...swapPath.map(pool => ({ pubkey: pool, isSigner: false, isWritable: true })),
      { pubkey: TOKEN_PROGRAM_ID, isSigner: false, isWritable: false }
    ];

    return new TransactionInstruction({
      keys: accounts,
      programId,
      data: instructionData
    });
  }

  /**
   * Create limit order
   */
  static async createLimitOrder(
    connection: Connection,
    user: PublicKey,
    poolAddress: PublicKey,
    inputMint: PublicKey,
    outputMint: PublicKey,
    inputAmount: BN,
    minOutputAmount: BN,
    expiry: BN,
    programId: PublicKey = FINOVA_DEFI_PROGRAM_ID
  ): Promise<TransactionInstruction> {
    const [orderAddress] = await PublicKey.findProgramAddress(
      [
        Buffer.from('limit_order'),
        user.toBuffer(),
        poolAddress.toBuffer(),
        Buffer.from(Date.now().toString())
      ],
      programId
    );

    const instructionData = Buffer.concat([
      Buffer.from([12]), // CreateLimitOrder instruction discriminator
      inputAmount.toArrayLike(Buffer, 'le', 8),
      minOutputAmount.toArrayLike(Buffer, 'le', 8),
      expiry.toArrayLike(Buffer, 'le', 8)
    ]);

    const accounts: AccountMeta[] = [
      { pubkey: user, isSigner: true, isWritable: true },
      { pubkey: orderAddress, isSigner: false, isWritable: true },
      { pubkey: poolAddress, isSigner: false, isWritable: false },
      { pubkey: inputMint, isSigner: false, isWritable: false },
      { pubkey: outputMint, isSigner: false, isWritable: false },
      { pubkey: SystemProgram.programId, isSigner: false, isWritable: false }
    ];

    return new TransactionInstruction({
      keys: accounts,
      programId,
      data: instructionData
    });
  }

  /**
   * Execute arbitrage between pools
   */
  static async executeArbitrage(
    connection: Connection,
    arbitrageur: PublicKey,
    poolA: PublicKey,
    poolB: PublicKey,
    tokenMint: PublicKey,
    amount: BN,
    programId: PublicKey = FINOVA_DEFI_PROGRAM_ID
  ): Promise<TransactionInstruction> {
    const instructionData = Buffer.concat([
      Buffer.from([13]), // ExecuteArbitrage instruction discriminator
      amount.toArrayLike(Buffer, 'le', 8)
    ]);

    const accounts: AccountMeta[] = [
      { pubkey: arbitrageur, isSigner: true, isWritable: true },
      { pubkey: poolA, isSigner: false, isWritable: true },
      { pubkey: poolB, isSigner: false, isWritable: true },
      { pubkey: tokenMint, isSigner: false, isWritable: false },
      { pubkey: TOKEN_PROGRAM_ID, isSigner: false, isWritable: false }
    ];

    return new TransactionInstruction({
      keys: accounts,
      programId,
      data: instructionData
    });
  }
}

// ============================================================================
// Oracle Integration
// ============================================================================

export class OracleIntegration {
  
  /**
   * Update price feed from external oracle
   */
  static async updatePriceFeed(
    connection: Connection,
    authority: PublicKey,
    poolAddress: PublicKey,
    newPrice: BN,
    confidence: BN,
    oracleProgram: PublicKey,
    programId: PublicKey = FINOVA_DEFI_PROGRAM_ID
  ): Promise<TransactionInstruction> {
    const [oracleAddress] = await PublicKey.findProgramAddress(
      [
        Buffer.from(ORACLE_SEED),
        poolAddress.toBuffer()
      ],
      oracleProgram
    );

    const instructionData = Buffer.concat([
      Buffer.from([14]), // UpdatePriceFeed instruction discriminator
      newPrice.toArrayLike(Buffer, 'le', 8),
      confidence.toArrayLike(Buffer, 'le', 8)
    ]);

    const accounts: AccountMeta[] = [
      { pubkey: authority, isSigner: true, isWritable: true },
      { pubkey: poolAddress, isSigner: false, isWritable: true },
      { pubkey: oracleAddress, isSigner: false, isWritable: true },
      { pubkey: oracleProgram, isSigner: false, isWritable: false },
      { pubkey: SYSVAR_CLOCK_PUBKEY, isSigner: false, isWritable: false }
    ];

    return new TransactionInstruction({
      keys: accounts,
      programId,
      data: instructionData
    });
  }

  /**
   * Fetch current price from oracle
   */
  static async getCurrentPrice(
    connection: Connection,
    poolAddress: PublicKey,
    oracleProgram: PublicKey
  ): Promise<{ price: BN; confidence: BN; lastUpdate: BN }> {
    try {
      const [oracleAddress] = await PublicKey.findProgramAddress(
        [
          Buffer.from(ORACLE_SEED),
          poolAddress.toBuffer()
        ],
        oracleProgram
      );

      const oracleAccount = await connection.getAccountInfo(oracleAddress);
      if (!oracleAccount?.data) {
        throw new Error('Oracle account not found');
      }

      // Parse oracle data (implementation depends on oracle format)
      const price = new BN(oracleAccount.data.slice(8, 16), 'le');
      const confidence = new BN(oracleAccount.data.slice(16, 24), 'le');
      const lastUpdate = new BN(oracleAccount.data.slice(24, 32), 'le');

      return { price, confidence, lastUpdate };
    } catch (error) {
      console.error('Failed to fetch oracle price:', error);
      throw error;
    }
  }
}

// ============================================================================
// Risk Management & Security
// ============================================================================

export class RiskManagement {
  
  /**
   * Calculate maximum safe swap amount based on slippage tolerance
   */
  static calculateMaxSafeSwapAmount(
    reserveIn: BN,
    reserveOut: BN,
    maxSlippageBps: number = MAX_SLIPPAGE_BPS
  ): BN {
    // Using constant product formula to find max amount for given slippage
    const slippageRatio = new BN(maxSlippageBps).div(new BN(10000));
    const maxPriceImpact = reserveOut.mul(slippageRatio);
    
    // Simplified calculation - in production use more sophisticated models
    return reserveIn.mul(slippageRatio);
  }

  /**
   * Validate pool health metrics
   */
  static validatePoolHealth(poolInfo: PoolInfo): { isHealthy: boolean; warnings: string[] } {
    const warnings: string[] = [];
    let isHealthy = true;

    // Check for sufficient liquidity
    // Note: This would need actual reserve data in production
    
    // Check fee parameters
    if (poolInfo.tradeFee > 1000) { // 10%
      warnings.push('Trade fee unusually high');
      isHealthy = false;
    }

    if (poolInfo.protocolFee > 100) { // 1%
      warnings.push('Protocol fee unusually high');
    }

    return { isHealthy, warnings };
  }

  /**
   * Calculate impermanent loss for LP position
   */
  static calculateImpermanentLoss(
    initialPriceRatio: number,
    currentPriceRatio: number
  ): number {
    const ratio = currentPriceRatio / initialPriceRatio;
    const hodlValue = (1 + ratio) / 2;
    const lpValue = Math.sqrt(ratio);
    
    return ((lpValue / hodlValue) - 1) * 100; // Return as percentage
  }

  /**
   * Estimate gas costs for complex operations
   */
  static estimateGasCosts(operationType: DeFiActivityType): number {
    const baseCosts = {
      [DeFiActivityType.AddLiquidity]: 150000,
      [DeFiActivityType.RemoveLiquidity]: 120000,
      [DeFiActivityType.Swap]: 100000,
      [DeFiActivityType.YieldFarm]: 180000,
      [DeFiActivityType.FlashLoan]: 200000
    };

    return baseCosts[operationType] || 100000;
  }
}

// ============================================================================
// Transaction Builders & Helpers
// ============================================================================

export class TransactionBuilders {
  
  /**
   * Build complete add liquidity transaction with all required instructions
   */
  static async buildAddLiquidityTransaction(
    connection: Connection,
    user: PublicKey,
    poolAddress: PublicKey,
    tokenAMint: PublicKey,
    tokenBMint: PublicKey,
    data: AddLiquidityData,
    programId: PublicKey = FINOVA_DEFI_PROGRAM_ID
  ): Promise<Transaction> {
    const transaction = new Transaction();
    
    // Check if user has associated token accounts
    const userTokenAAccount = await getAssociatedTokenAddress(tokenAMint, user);
    const userTokenBAccount = await getAssociatedTokenAddress(tokenBMint, user);
    
    try {
      await connection.getAccountInfo(userTokenAAccount);
    } catch {
      // Create associated token account if it doesn't exist
      transaction.add(
        createAssociatedTokenAccountInstruction(
          user,
          userTokenAAccount,
          user,
          tokenAMint
        )
      );
    }

    try {
      await connection.getAccountInfo(userTokenBAccount);
    } catch {
      transaction.add(
        createAssociatedTokenAccountInstruction(
          user,
          userTokenBAccount,
          user,
          tokenBMint
        )
      );
    }

    // Add the main instruction
    const addLiquidityIx = await DeFiInstructions.addLiquidity(
      connection,
      user,
      poolAddress,
      tokenAMint,
      tokenBMint,
      data,
      programId
    );
    transaction.add(addLiquidityIx);

    // Add XP reporting instruction
    const reportActivityIx = await FinovaCoreIntegration.reportDeFiActivity(
      connection,
      user,
      DeFiActivityType.AddLiquidity,
      data.tokenAAmount.add(data.tokenBAmount)
    );
    transaction.add(reportActivityIx);

    return transaction;
  }

  /**
   * Build swap transaction with optimal routing
   */
  static async buildOptimalSwapTransaction(
    connection: Connection,
    user: PublicKey,
    inputMint: PublicKey,
    outputMint: PublicKey,
    amountIn: BN,
    minAmountOut: BN,
    maxSlippageBps: number = 100,
    programId: PublicKey = FINOVA_DEFI_PROGRAM_ID
  ): Promise<Transaction> {
    const transaction = new Transaction();

    // Find optimal route (simplified - in production use route optimization)
    const [poolAddress] = await PDAUtils.getPoolAddress(inputMint, outputMint, programId);
    
    const swapData: SwapData = {
      amountIn,
      minAmountOut,
      slippageBps: maxSlippageBps,
      isTokenAToB: inputMint.toBuffer().compare(outputMint.toBuffer()) < 0
    };

    const swapIx = await DeFiInstructions.swap(
      connection,
      user,
      poolAddress,
      inputMint,
      outputMint,
      swapData,
      programId
    );
    transaction.add(swapIx);

    // Add activity reporting
    const reportActivityIx = await FinovaCoreIntegration.reportDeFiActivity(
      connection,
      user,
      DeFiActivityType.Swap,
      amountIn
    );
    transaction.add(reportActivityIx);

    return transaction;
  }

  /**
   * Build flash loan arbitrage transaction
   */
  static async buildFlashLoanArbitrageTransaction(
    connection: Connection,
    arbitrageur: PublicKey,
    poolA: PublicKey,
    poolB: PublicKey,
    tokenMint: PublicKey,
    loanAmount: BN,
    programId: PublicKey = FINOVA_DEFI_PROGRAM_ID
  ): Promise<Transaction> {
    const transaction = new Transaction();

    // Calculate expected profit
    // This would involve price checking and route optimization in production

    const flashLoanData: FlashLoanData = {
      amount: loanAmount,
      fee: loanAmount.mul(new BN(5)).div(new BN(10000)), // 0.05% fee
      instructions: [] // Would contain arbitrage instructions
    };

    const flashLoanIx = await DeFiInstructions.flashLoan(
      connection,
      arbitrageur,
      poolA,
      tokenMint,
      flashLoanData,
      programId
    );
    transaction.add(flashLoanIx);

    return transaction;
  }
}

// ============================================================================
// Export All Components
// ============================================================================

export {
  DeFiInstructions,
  FinovaCoreIntegration,
  AdvancedDeFiOperations,
  OracleIntegration,
  RiskManagement,
  TransactionBuilders,
  PDAUtils,
  AMMUtils
};

export default {
  DeFiInstructions,
  FinovaCoreIntegration,
  AdvancedDeFiOperations,
  OracleIntegration,
  RiskManagement,
  TransactionBuilders,
  PDAUtils,
  AMMUtils,
  FINOVA_DEFI_PROGRAM_ID,
  FINOVA_CORE_PROGRAM_ID,
  FINOVA_TOKEN_PROGRAM_ID
};

/**
 * Usage Examples:
 * 
 * // Create a new AMM pool
 * const createPoolIx = await DeFiInstructions.createPool(
 *   connection,
 *   payer.publicKey,
 *   tokenAMint,
 *   tokenBMint,
 *   {
 *     tradeFee: 30,
 *     protocolFee: 5,
 *     curveType: CurveType.ConstantProduct,
 *     initialTokenAAmount: new BN(1000000),
 *     initialTokenBAmount: new BN(1000000)
 *   }
 * );
 * 
 * // Add liquidity to existing pool
 * const addLiquidityTx = await TransactionBuilders.buildAddLiquidityTransaction(
 *   connection,
 *   user.publicKey,
 *   poolAddress,
 *   tokenAMint,
 *   tokenBMint,
 *   {
 *     tokenAAmount: new BN(10000),
 *     tokenBAmount: new BN(10000),
 *     minLpTokens: new BN(9900),
 *     slippageBps: 100
 *   }
 * );
 * 
 * // Execute optimal swap
 * const swapTx = await TransactionBuilders.buildOptimalSwapTransaction(
 *   connection,
 *   user.publicKey,
 *   inputMint,
 *   outputMint,
 *   new BN(1000),
 *   new BN(950),
 *   50 // 0.5% max slippage
 * );
 * 
 * // Calculate price impact
 * const priceImpact = AMMUtils.calculatePriceImpact(
 *   swapAmount,
 *   reserveIn,
 *   reserveOut,
 *   outputAmount
 * );
 * 
 * // Check pool health
 * const healthCheck = RiskManagement.validatePoolHealth(poolInfo);
 */
 