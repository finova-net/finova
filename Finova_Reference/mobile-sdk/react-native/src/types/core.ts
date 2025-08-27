/**
 * Finova Network Mobile SDK - Core Types
 *
 * @version 2.0.0
 * @author Finova Network Team
 * @description Core type definitions for the Finova Network, including blockchain,
 * authentication, and system-level interfaces.
 */

import { PublicKey } from '@solana/web3.js';
import { RateLimitInfo } from './user'; // Assuming RateLimitInfo is in user.ts

// MARK: - Network & Environment

export enum NetworkType {
  MAINNET = 'mainnet-beta',
  TESTNET = 'testnet',
  DEVNET = 'devnet',
  LOCALNET = 'localnet'
}

export enum MiningPhase {
  FINIZEN = 1,    // 0-100K users
  GROWTH = 2,     // 100K-1M users
  MATURITY = 3,   // 1M-10M users
  STABILITY = 4   // 10M+ users
}

export interface EnvironmentConfig {
  network: NetworkType;
  apiBaseUrl: string;
  wsUrl: string;
  solanaRpcUrl: string;
  ipfsGateway: string;
  cdnUrl: string;
  debug: boolean;
  version: string;
}

export interface FeatureFlags {
  mining: boolean;
  staking: boolean;
  nftMarketplace: boolean;
  guilds: boolean;
  socialIntegration: boolean;
  crossChainBridge: boolean;
  aiQualityCheck: boolean;
  biometricAuth: boolean;
  [key: string]: boolean;
}

// MARK: - Blockchain & Wallet

export interface WalletInfo {
  address: string;
  publicKey: PublicKey;
  balance: string;
  isConnected: boolean;
  walletType: 'phantom' | 'solflare' | 'coinbase' | 'backpack' | 'internal';
  network: NetworkType;
}

export interface TokenBalance {
  symbol: string;
  amount: string;
  decimals: number;
  uiAmount: number;
  usdValue?: string;
  icon?: string;
}

export interface TokenInfo {
  mint: string;
  symbol: string;
  name: string;
  decimals: number;
  totalSupply: string;
  icon?: string;
  isNative: boolean;
}

// MARK: - Transaction Types

export interface TransactionData {
  id: string;
  type: TransactionType;
  from: string;
  to: string;
  amount: string;
  token: string;
  fee: string;
  status: TransactionStatus;
  timestamp: number;
  blockHeight?: number;
  signature?: string;
  memo?: string;
}

export enum TransactionType {
  MINING = 'mining',
  STAKING = 'staking',
  UNSTAKING = 'unstaking',
  TRANSFER = 'transfer',
  SWAP = 'swap',
  NFT_MINT = 'nft_mint',
  NFT_TRANSFER = 'nft_transfer',
  REWARD_CLAIM = 'reward_claim',
  CARD_USE = 'card_use'
}

export enum TransactionStatus {
  PENDING = 'pending',
  PROCESSING = 'processing',
  CONFIRMED = 'confirmed',
  FAILED = 'failed',
  CANCELLED = 'cancelled'
}

// MARK: - Device & Location

export interface DeviceInfo {
  id: string;
  platform: 'ios' | 'android';
  version: string;
  model: string;
  manufacturer: string;
  isRooted: boolean;
  isEmulator: boolean;
  fingerprint: string;
  registeredAt: number;
}

export interface LocationData {
  country: string;
  region: string;
  city: string;
  timezone: string;
  coordinates?: {
    latitude: number;
    longitude: number;
  };
}

// MARK: - Cache & Storage

export interface CacheData<T = any> {
  key: string;
  data: T;
  timestamp: number;
  expiresAt: number;
  version: string;
}

export interface StorageConfig {
  encrypted: boolean;
  persistent: boolean;
  maxSize: number; // in MB
  ttl: number; // in seconds
}
