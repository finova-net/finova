/**
 * Finova Network Mobile SDK - Staking System Types
 *
 * @version 2.0.0
 * @author Finova Network Team
 * @description Defines structures for staking pools, user positions, rewards,
 * and liquid staking within the Finova DeFi ecosystem.
 */

import { BaseEntity, TokenInfo } from './';
import { MultiplierType } from './mining';

// MARK: - Staking Pools

export interface StakingPool extends BaseEntity {
  name: string;
  token: TokenInfo;
  rewardToken: TokenInfo;
  totalStaked: number;
  totalStakedUSD: number;
  apy: number;
  minStakeAmount: number;
  lockupPeriods: LockupPeriod[];
  features: ('auto-compound' | 'liquid-staking' | 'nft-boost')[];
  isActive: boolean;
}

export interface LockupPeriod {
  duration: number; // in seconds
  apyMultiplier: number;
  earlyUnstakePenalty: number; // percentage
  label: string;
}

// MARK: - User Staking Position

export interface StakingPosition extends BaseEntity {
  poolId: string;
  userId: string;
  stakedAmount: number;
  stakedValueUSD: number;
  shares: number; // Represents ownership share in the pool
  stakedAt: Date;
  lockupEndsAt?: Date;
  rewards: StakingRewards;
  status: 'active' | 'unstaking' | 'withdrawn';
  autoCompounding: boolean;
}

export interface StakingRewards {
  earned: number;
  pending: number;
  claimed: number;
  lastClaimAt: Date;
  projectedDaily: number;
}

// MARK: - Staking Tiers & Multipliers

export interface StakingTier {
  name: string;
  minAmount: number;
  benefits: {
    apyBonus: number;
    miningBoost: number;
    governanceWeight: number;
  };
}

export interface StakingMultiplier {
  type: MultiplierType;
  value: number;
  source: string; // e.g., 'XP Level 50', 'Legendary NFT'
}

// MARK: - Liquid Staking (sFIN)

export interface LiquidStakingInfo {
  sFINBalance: number;
  underlyingFIN: number;
  exchangeRate: number;
  lastUpdate: Date;
  unstakeQueue: UnstakeRequest[];
}

export interface UnstakeRequest extends BaseEntity {
  sFINAmount: number;
  finAmount: number;
  requestedAt: Date;
  availableAt: Date;
  status: 'pending' | 'available' | 'claimed';
}
