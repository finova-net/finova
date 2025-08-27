/**
 * Finova Network Mobile SDK - DeFi Types
 *
 * @version 2.0.0
 * @author Finova Network Team
 * @description Contains all type definitions related to DeFi features like
 * Staking, Liquidity Pools, Yield Farming, and Flash Loans.
 */

import { BaseEntity, TransactionStatus, TokenInfo } from './';

// MARK: - Staking

export interface StakingPool extends BaseEntity {
  name: string;
  token: TokenInfo;
  rewardToken: TokenInfo;
  apr: number;
  apy: number;
  tvl: number; // Total Value Locked in USD
  minStake: number;
  maxStake?: number;
  lockupPeriod: number; // in days
  earlyExitPenalty: number; // percentage
  status: 'active' | 'paused' | 'ended';
  features: StakingFeature[];
}

export type StakingFeature = 'auto-compound' | 'nft-boost' | 'flexible-term' | 'liquid-staking';

export interface StakingPosition extends BaseEntity {
  userId: string;
  poolId: string;
  stakedAmount: number;
  stakedValueUSD: number;
  rewardAmount: number;
  stakedAt: Date;
  unlocksAt?: Date;
  apy: number;
  rewardsEarned: number;
  status: 'staking' | 'unstaking' | 'withdrawn';
}

// MARK: - Liquidity Pools & Swaps

export interface LiquidityPool extends BaseEntity {
  name: string;
  symbol: string;
  tokenA: TokenInfo;
  tokenB: TokenInfo;
  reserveA: number;
  reserveB: number;
  totalLiquidityUSD: number;
  volume24h: number;
  fees24h: number;
  apr: number;
  status: 'active' | 'paused' | 'deprecated';
}

export interface LiquidityPosition extends BaseEntity {
  userId: string;
  poolId: string;
  lpTokens: number;
  tokenAAmount: number;
  tokenBAmount: number;
  shareOfPool: number;
  valueUSD: number;
  impermanentLoss?: number;
  feesEarned: number;
  addedAt: Date;
}

export interface SwapTransaction extends BaseEntity {
  userId: string;
  poolId: string;
  tokenIn: TokenInfo;
  tokenOut: TokenInfo;
  amountIn: number;
  amountOut: number;
  priceImpact: number;
  fee: number;
  slippage: number;
  route: SwapRoute[];
  txHash: string;
  status: TransactionStatus;
  timestamp: Date;
}

export interface SwapRoute {
  poolId: string;
  tokenInSymbol: string;
  tokenOutSymbol: string;
}

// MARK: - Yield Farming

export interface YieldFarm extends BaseEntity {
  name: string;
  description: string;
  lpToken: TokenInfo; // The token to stake (usually an LP token)
  rewardTokens: TokenInfo[];
  totalStakedUSD: number;
  apr: number;
  apy: number;
  multiplier: number;
  status: 'active' | 'ended' | 'paused';
  participants: number;
}

export interface FarmPosition extends BaseEntity {
  userId: string;
  farmId: string;
  stakedAmount: number; // Amount of LP tokens
  stakedValueUSD: number;
  rewards: FarmReward[];
  stakedAt: Date;
  lastHarvestAt: Date;
}

export interface FarmReward {
  token: TokenInfo;
  amount: number;
  valueUSD: number;
}

// MARK: - Advanced DeFi

export interface FlashLoan extends BaseEntity {
  userId: string;
  asset: TokenInfo;
  amount: number;
  fee: number;
  premium: number;
  purpose: 'arbitrage' | 'liquidation' | 'refinancing';
  profit: number;
  txHash: string;
  status: TransactionStatus;
}

export interface PriceOracle {
  asset: string;
  price: number;
  confidence: number;
  lastUpdate: Date;
  sources: PriceSource[];
  isValid: boolean;
}

export interface PriceSource {
  name: string; // e.g., 'Pyth', 'Chainlink', 'DEX'
  price: number;
  weight: number;
  lastUpdate: Date;
  status: 'active' | 'stale' | 'failed';
}

export interface DeFiPortfolio {
  totalValueUSD: number;
  liquidityPositions: LiquidityPosition[];
  farmPositions: FarmPosition[];
  stakedPositions: StakingPosition[];
  walletTokens: TokenBalance[];
  dailyPnL: number; // Profit and Loss
  totalPnL: number;
  overallApr: number;
  riskScore: number; // 1-100
}

export interface TokenBalance {
  token: TokenInfo;
  balance: number;
  valueUSD: number;
  allocation: number; // percentage
}
