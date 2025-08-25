/**
 * Finova Network Mobile SDK - React Native Types
 * Main Types Export File
 *
 * @version 2.0.0
 * @author Finova Network Team
 * @description This file serves as the central export point for all type
 * definitions within the Finova React Native SDK. It ensures
 * a consistent and organized type structure for the entire application.
 * All types have been reviewed, merged, and optimized for an
 * enterprise-grade, world-class Super App.
 */

// MARK: - Core System & Common Types
export * from './common';
export * from './core';

// MARK: - Feature-Specific Types
export * from './user';
export * from './mining';
export * from './xp';
export * from './referral';
export * from './social';
export * from './nft';
export * from './staking';
export * from './defi';
export * from './guild';

// MARK: - Global Constants
export const FINOVA_CONSTANTS = {
  SDK_VERSION: '2.0.0',
  SOLANA_CHAIN_ID: 101, // Solana Mainnet-Beta
  FIN_DECIMALS: 9,
  MAX_SUPPLY: '100000000000', // 100 Billion FIN tokens
  MINING_SESSION_DURATION: 24 * 60 * 60, // 24 hours in seconds
  MAX_REFERRAL_DEPTH: 3,
  XP_LEVEL_CAP: 1000,
  RP_TIER_COUNT: 5,
  GUILD_MAX_MEMBERS: 50,
} as const;

// MARK: - Base Entity Type
// A consistent base for all major data models.
export interface BaseEntity {
  id: string;
  createdAt: Date;
  updatedAt: Date;
  version: number;
  metadata?: Record<string, any>;
}
