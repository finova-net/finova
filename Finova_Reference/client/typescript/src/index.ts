// client/typescript/src/index.ts

/**
 * Finova Network TypeScript SDK
 * Official SDK for interacting with Finova Network Smart Contracts
 */

export * from './client';
export * from './instructions';
export * from './accounts';
export * from './types';
export * from './utils';
export * from './constants';

// Re-export commonly used types from dependencies
export { Connection, PublicKey, Keypair, Transaction } from '@solana/web3.js';
export { Program, AnchorProvider, Wallet } from '@coral-xyz/anchor';
export { BN } from 'bn.js';
