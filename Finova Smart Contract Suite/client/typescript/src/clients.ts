// client/typescript/src/clients.ts

import {
  Connection,
  PublicKey,
  Transaction,
  TransactionInstruction,
  Keypair,
  SystemProgram,
  SYSVAR_RENT_PUBKEY,
  SYSVAR_CLOCK_PUBKEY,
  sendAndConfirmTransaction,
  ConfirmOptions,
  Commitment,
} from '@solana/web3.js';
import {
  TOKEN_PROGRAM_ID,
  ASSOCIATED_TOKEN_PROGRAM_ID,
  getAssociatedTokenAddress,
  createAssociatedTokenAccountInstruction,
  getAccount,
  TokenAccountNotFoundError,
  TokenInvalidAccountOwnerError,
} from '@solana/spl-token';
import * as anchor from '@coral-xyz/anchor';
import { Program, AnchorProvider, BN, web3 } from '@coral-xyz/anchor';
import {
  FinovaCore,
  FinovaToken,
  FinovaNft,
  FinovaDefi,
  FinovaOracle,
  FinovaBridge,
} from '../types';
import {
  UserState,
  XPState,
  ReferralState,
  StakingState,
  ActiveEffectsState,
  GuildState,
  NetworkState,
  ProposalState,
  VoteRecord,
  MiningRewards,
  XPActivity,
  ReferralBonus,
  StakingReward,
  NFTMetadata,
  SpecialCard,
  ClientConfig,
  TransactionResult,
  MiningStats,
  NetworkStats,
} from '../types';
import { PROGRAM_IDS, FINOVA_CONSTANTS } from '../constants';

export class FinovaClient {
  public connection: Connection;
  public provider: AnchorProvider;
  public coreProgram: Program<FinovaCore>;
  public tokenProgram: Program<FinovaToken>;
  public nftProgram: Program<FinovaNft>;
  public defiProgram: Program<FinovaDefi>;
  public oracleProgram: Program<FinovaOracle>;
  public bridgeProgram: Program<FinovaBridge>;
  public wallet: anchor.Wallet;
  public config: ClientConfig;

  constructor(
    connection: Connection,
    wallet: anchor.Wallet,
    config: ClientConfig = {}
  ) {
    this.connection = connection;
    this.wallet = wallet;
    this.config = {
      commitment: 'confirmed',
      preflightCommitment: 'confirmed',
      skipPreflight: false,
      ...config,
    };

    this.provider = new AnchorProvider(
      connection,
      wallet,
      {
        commitment: this.config.commitment as Commitment,
        preflightCommitment: this.config.preflightCommitment as Commitment,
        skipPreflight: this.config.skipPreflight,
      }
    );

    // Initialize programs
    this.initializePrograms();
  }

  private initializePrograms(): void {
    // Note: In a real implementation, you would load IDLs from files or network
    // For this example, we'll assume the IDLs are available
    try {
      this.coreProgram = new Program(
        {} as any, // IDL would be loaded here
        PROGRAM_IDS.FINOVA_CORE,
        this.provider
      );

      this.tokenProgram = new Program(
        {} as any, // IDL would be loaded here
        PROGRAM_IDS.FINOVA_TOKEN,
        this.provider
      );

      this.nftProgram = new Program(
        {} as any, // IDL would be loaded here
        PROGRAM_IDS.FINOVA_NFT,
        this.provider
      );

      this.defiProgram = new Program(
        {} as any, // IDL would be loaded here
        PROGRAM_IDS.FINOVA_DEFI,
        this.provider
      );

      this.oracleProgram = new Program(
        {} as any, // IDL would be loaded here
        PROGRAM_IDS.FINOVA_ORACLE,
        this.provider
      );

      this.bridgeProgram = new Program(
        {} as any, // IDL would be loaded here
        PROGRAM_IDS.FINOVA_BRIDGE,
        this.provider
      );
    } catch (error) {
      console.error('Failed to initialize programs:', error);
      throw new Error('Program initialization failed');
    }
  }

  // ============================================================================
  // Account Management
  // ============================================================================

  /**
   * Initialize a new user account
   */
  async initializeUser(
    referralCode?: string
  ): Promise<TransactionResult> {
    try {
      const userPubkey = this.wallet.publicKey;
      const [userStatePDA] = this.getUserStatePDA(userPubkey);
      const [xpStatePDA] = this.getXPStatePDA(userPubkey);
      const [referralStatePDA] = this.getReferralStatePDA(userPubkey);
      const [stakingStatePDA] = this.getStakingStatePDA(userPubkey);
      const [activeEffectsPDA] = this.getActiveEffectsPDA(userPubkey);

      const instruction = await this.coreProgram.methods
        .initializeUser(referralCode || null)
        .accounts({
          user: userPubkey,
          userState: userStatePDA,
          xpState: xpStatePDA,
          referralState: referralStatePDA,
          stakingState: stakingStatePDA,
          activeEffects: activeEffectsPDA,
          systemProgram: SystemProgram.programId,
          rent: SYSVAR_RENT_PUBKEY,
        })
        .instruction();

      const transaction = new Transaction().add(instruction);
      const signature = await this.sendTransaction(transaction);

      return {
        success: true,
        signature,
        accounts: {
          userState: userStatePDA,
          xpState: xpStatePDA,
          referralState: referralStatePDA,
          stakingState: stakingStatePDA,
          activeEffects: activeEffectsPDA,
        },
      };
    } catch (error) {
      console.error('Failed to initialize user:', error);
      return {
        success: false,
        error: error instanceof Error ? error.message : 'Unknown error',
      };
    }
  }

  /**
   * Get user state account
   */
  async getUserState(userPubkey?: PublicKey): Promise<UserState | null> {
    try {
      const targetUser = userPubkey || this.wallet.publicKey;
      const [userStatePDA] = this.getUserStatePDA(targetUser);
      
      const userState = await this.coreProgram.account.userState.fetch(userStatePDA);
      return userState as UserState;
    } catch (error) {
      console.error('Failed to fetch user state:', error);
      return null;
    }
  }

  /**
   * Get XP state account
   */
  async getXPState(userPubkey?: PublicKey): Promise<XPState | null> {
    try {
      const targetUser = userPubkey || this.wallet.publicKey;
      const [xpStatePDA] = this.getXPStatePDA(targetUser);
      
      const xpState = await this.coreProgram.account.xpState.fetch(xpStatePDA);
      return xpState as XPState;
    } catch (error) {
      console.error('Failed to fetch XP state:', error);
      return null;
    }
  }

  /**
   * Get referral state account
   */
  async getReferralState(userPubkey?: PublicKey): Promise<ReferralState | null> {
    try {
      const targetUser = userPubkey || this.wallet.publicKey;
      const [referralStatePDA] = this.getReferralStatePDA(targetUser);
      
      const referralState = await this.coreProgram.account.referralState.fetch(referralStatePDA);
      return referralState as ReferralState;
    } catch (error) {
      console.error('Failed to fetch referral state:', error);
      return null;
    }
  }

  /**
   * Get staking state account
   */
  async getStakingState(userPubkey?: PublicKey): Promise<StakingState | null> {
    try {
      const targetUser = userPubkey || this.wallet.publicKey;
      const [stakingStatePDA] = this.getStakingStatePDA(targetUser);
      
      const stakingState = await this.coreProgram.account.stakingState.fetch(stakingStatePDA);
      return stakingState as StakingState;
    } catch (error) {
      console.error('Failed to fetch staking state:', error);
      return null;
    }
  }

  /**
   * Get active effects state account
   */
  async getActiveEffectsState(userPubkey?: PublicKey): Promise<ActiveEffectsState | null> {
    try {
      const targetUser = userPubkey || this.wallet.publicKey;
      const [activeEffectsPDA] = this.getActiveEffectsPDA(targetUser);
      
      const activeEffects = await this.coreProgram.account.activeEffectsState.fetch(activeEffectsPDA);
      return activeEffects as ActiveEffectsState;
    } catch (error) {
      console.error('Failed to fetch active effects state:', error);
      return null;
    }
  }

  /**
   * Get network state
   */
  async getNetworkState(): Promise<NetworkState | null> {
    try {
      const [networkStatePDA] = this.getNetworkStatePDA();
      const networkState = await this.coreProgram.account.networkState.fetch(networkStatePDA);
      return networkState as NetworkState;
    } catch (error) {
      console.error('Failed to fetch network state:', error);
      return null;
    }
  }

  // ============================================================================
  // Mining Operations
  // ============================================================================

  /**
   * Claim mining rewards
   */
  async claimMiningRewards(): Promise<TransactionResult> {
    try {
      const userPubkey = this.wallet.publicKey;
      const [userStatePDA] = this.getUserStatePDA(userPubkey);
      const [xpStatePDA] = this.getXPStatePDA(userPubkey);
      const [referralStatePDA] = this.getReferralStatePDA(userPubkey);
      const [stakingStatePDA] = this.getStakingStatePDA(userPubkey);
      const [activeEffectsPDA] = this.getActiveEffectsPDA(userPubkey);
      const [networkStatePDA] = this.getNetworkStatePDA();

      // Get user's associated token account for $FIN
      const userTokenAccount = await getAssociatedTokenAddress(
        FINOVA_CONSTANTS.FIN_MINT,
        userPubkey
      );

      const instruction = await this.coreProgram.methods
        .claimRewards()
        .accounts({
          user: userPubkey,
          userState: userStatePDA,
          xpState: xpStatePDA,
          referralState: referralStatePDA,
          stakingState: stakingStatePDA,
          activeEffects: activeEffectsPDA,
          networkState: networkStatePDA,
          userTokenAccount,
          finMint: FINOVA_CONSTANTS.FIN_MINT,
          tokenProgram: TOKEN_PROGRAM_ID,
          clock: SYSVAR_CLOCK_PUBKEY,
        })
        .remainingAccounts([
          {
            pubkey: PROGRAM_IDS.FINOVA_TOKEN,
            isWritable: false,
            isSigner: false,
          },
        ])
        .instruction();

      const transaction = new Transaction().add(instruction);
      const signature = await this.sendTransaction(transaction);

      return {
        success: true,
        signature,
      };
    } catch (error) {
      console.error('Failed to claim mining rewards:', error);
      return {
        success: false,
        error: error instanceof Error ? error.message : 'Unknown error',
      };
    }
  }

  /**
   * Calculate current mining rate for user
   */
  async calculateMiningRate(userPubkey?: PublicKey): Promise<number> {
    try {
      const targetUser = userPubkey || this.wallet.publicKey;
      const userState = await this.getUserState(targetUser);
      const xpState = await this.getXPState(targetUser);
      const referralState = await this.getReferralState(targetUser);
      const stakingState = await this.getStakingState(targetUser);
      const activeEffects = await this.getActiveEffectsState(targetUser);
      const networkState = await this.getNetworkState();

      if (!userState || !xpState || !referralState || !stakingState || !networkState) {
        throw new Error('Failed to fetch required account data');
      }

      // Implement mining rate calculation based on whitepaper formula
      const baseRate = this.calculateBaseRate(networkState);
      const pioneerBonus = this.calculatePioneerBonus(networkState);
      const referralBonus = this.calculateReferralBonus(referralState);
      const securityBonus = userState.isKycVerified ? 1.2 : 0.8;
      const regressionFactor = Math.exp(-0.001 * userState.totalHoldings.toNumber());
      const xpMultiplier = this.calculateXPMultiplier(xpState);
      const stakingMultiplier = this.calculateStakingMultiplier(stakingState);
      const effectsMultiplier = this.calculateEffectsMultiplier(activeEffects);

      const miningRate = baseRate * pioneerBonus * referralBonus * securityBonus * 
                        regressionFactor * xpMultiplier * stakingMultiplier * effectsMultiplier;

      return miningRate;
    } catch (error) {
      console.error('Failed to calculate mining rate:', error);
      return 0;
    }
  }

  /**
   * Get mining statistics
   */
  async getMiningStats(userPubkey?: PublicKey): Promise<MiningStats> {
    try {
      const targetUser = userPubkey || this.wallet.publicKey;
      const userState = await this.getUserState(targetUser);
      const currentRate = await this.calculateMiningRate(targetUser);

      if (!userState) {
        throw new Error('User state not found');
      }

      return {
        currentRate,
        totalMined: userState.totalMined.toNumber(),
        lastClaimTime: userState.lastClaimTime.toNumber(),
        dailyEstimate: currentRate * 24,
        weeklyEstimate: currentRate * 24 * 7,
        monthlyEstimate: currentRate * 24 * 30,
        pendingRewards: userState.pendingRewards.toNumber(),
      };
    } catch (error) {
      console.error('Failed to get mining stats:', error);
      return {
        currentRate: 0,
        totalMined: 0,
        lastClaimTime: 0,
        dailyEstimate: 0,
        weeklyEstimate: 0,
        monthlyEstimate: 0,
        pendingRewards: 0,
      };
    }
  }

  // ============================================================================
  // XP System Operations
  // ============================================================================

  /**
   * Update XP for user activity
   */
  async updateXP(
    activityType: string,
    platform: string,
    contentData: any,
    qualityScore?: number
  ): Promise<TransactionResult> {
    try {
      const userPubkey = this.wallet.publicKey;
      const [userStatePDA] = this.getUserStatePDA(userPubkey);
      const [xpStatePDA] = this.getXPStatePDA(userPubkey);
      const [activeEffectsPDA] = this.getActiveEffectsPDA(userPubkey);

      const instruction = await this.coreProgram.methods
        .updateXp(activityType, platform, contentData, qualityScore || null)
        .accounts({
          user: userPubkey,
          userState: userStatePDA,
          xpState: xpStatePDA,
          activeEffects: activeEffectsPDA,
          clock: SYSVAR_CLOCK_PUBKEY,
        })
        .instruction();

      const transaction = new Transaction().add(instruction);
      const signature = await this.sendTransaction(transaction);

      return {
        success: true,
        signature,
      };
    } catch (error) {
      console.error('Failed to update XP:', error);
      return {
        success: false,
        error: error instanceof Error ? error.message : 'Unknown error',
      };
    }
  }

  /**
   * Get XP activity history
   */
  async getXPHistory(userPubkey?: PublicKey, limit: number = 100): Promise<XPActivity[]> {
    try {
      const targetUser = userPubkey || this.wallet.publicKey;
      // This would typically query event logs or a separate indexing service
      // For now, we'll return empty array as placeholder
      return [];
    } catch (error) {
      console.error('Failed to get XP history:', error);
      return [];
    }
  }

  // ============================================================================
  // Referral System Operations
  // ============================================================================

  /**
   * Process referral signup
   */
  async processReferralSignup(referrerPubkey: PublicKey): Promise<TransactionResult> {
    try {
      const userPubkey = this.wallet.publicKey;
      const [userReferralPDA] = this.getReferralStatePDA(userPubkey);
      const [referrerReferralPDA] = this.getReferralStatePDA(referrerPubkey);

      const instruction = await this.coreProgram.methods
        .processReferralSignup()
        .accounts({
          user: userPubkey,
          referrer: referrerPubkey,
          userReferralState: userReferralPDA,
          referrerReferralState: referrerReferralPDA,
          clock: SYSVAR_CLOCK_PUBKEY,
        })
        .instruction();

      const transaction = new Transaction().add(instruction);
      const signature = await this.sendTransaction(transaction);

      return {
        success: true,
        signature,
      };
    } catch (error) {
      console.error('Failed to process referral signup:', error);
      return {
        success: false,
        error: error instanceof Error ? error.message : 'Unknown error',
      };
    }
  }

  /**
   * Get referral network
   */
  async getReferralNetwork(userPubkey?: PublicKey): Promise<{
    directReferrals: PublicKey[];
    level2Referrals: PublicKey[];
    level3Referrals: PublicKey[];
    totalNetworkSize: number;
  }> {
    try {
      const targetUser = userPubkey || this.wallet.publicKey;
      const referralState = await this.getReferralState(targetUser);

      if (!referralState) {
        return {
          directReferrals: [],
          level2Referrals: [],
          level3Referrals: [],
          totalNetworkSize: 0,
        };
      }

      return {
        directReferrals: referralState.directReferrals,
        level2Referrals: referralState.level2Referrals,
        level3Referrals: referralState.level3Referrals,
        totalNetworkSize: referralState.totalNetworkSize,
      };
    } catch (error) {
      console.error('Failed to get referral network:', error);
      return {
        directReferrals: [],
        level2Referrals: [],
        level3Referrals: [],
        totalNetworkSize: 0,
      };
    }
  }

  // ============================================================================
  // Staking Operations
  // ============================================================================

  /**
   * Stake tokens
   */
  async stakeTokens(amount: number): Promise<TransactionResult> {
    try {
      const userPubkey = this.wallet.publicKey;
      const [userStatePDA] = this.getUserStatePDA(userPubkey);
      const [stakingStatePDA] = this.getStakingStatePDA(userPubkey);

      const userTokenAccount = await getAssociatedTokenAddress(
        FINOVA_CONSTANTS.FIN_MINT,
        userPubkey
      );

      const instruction = await this.coreProgram.methods
        .stake(new BN(amount * FINOVA_CONSTANTS.TOKEN_DECIMALS))
        .accounts({
          user: userPubkey,
          userState: userStatePDA,
          stakingState: stakingStatePDA,
          userTokenAccount,
          finMint: FINOVA_CONSTANTS.FIN_MINT,
          tokenProgram: TOKEN_PROGRAM_ID,
          clock: SYSVAR_CLOCK_PUBKEY,
        })
        .instruction();

      const transaction = new Transaction().add(instruction);
      const signature = await this.sendTransaction(transaction);

      return {
        success: true,
        signature,
      };
    } catch (error) {
      console.error('Failed to stake tokens:', error);
      return {
        success: false,
        error: error instanceof Error ? error.message : 'Unknown error',
      };
    }
  }

  /**
   * Unstake tokens
   */
  async unstakeTokens(amount: number): Promise<TransactionResult> {
    try {
      const userPubkey = this.wallet.publicKey;
      const [userStatePDA] = this.getUserStatePDA(userPubkey);
      const [stakingStatePDA] = this.getStakingStatePDA(userPubkey);

      const userTokenAccount = await getAssociatedTokenAddress(
        FINOVA_CONSTANTS.FIN_MINT,
        userPubkey
      );

      const instruction = await this.coreProgram.methods
        .unstake(new BN(amount * FINOVA_CONSTANTS.TOKEN_DECIMALS))
        .accounts({
          user: userPubkey,
          userState: userStatePDA,
          stakingState: stakingStatePDA,
          userTokenAccount,
          finMint: FINOVA_CONSTANTS.FIN_MINT,
          tokenProgram: TOKEN_PROGRAM_ID,
          clock: SYSVAR_CLOCK_PUBKEY,
        })
        .instruction();

      const transaction = new Transaction().add(instruction);
      const signature = await this.sendTransaction(transaction);

      return {
        success: true,
        signature,
      };
    } catch (error) {
      console.error('Failed to unstake tokens:', error);
      return {
        success: false,
        error: error instanceof Error ? error.message : 'Unknown error',
      };
    }
  }

  // ============================================================================
  // NFT Operations
  // ============================================================================

  /**
   * Use special card
   */
  async useSpecialCard(cardMint: PublicKey): Promise<TransactionResult> {
    try {
      const userPubkey = this.wallet.publicKey;
      const [activeEffectsPDA] = this.getActiveEffectsPDA(userPubkey);

      const userTokenAccount = await getAssociatedTokenAddress(
        cardMint,
        userPubkey
      );

      const instruction = await this.nftProgram.methods
        .useSpecialCard()
        .accounts({
          user: userPubkey,
          cardMint,
          userTokenAccount,
          activeEffects: activeEffectsPDA,
          coreProgram: PROGRAM_IDS.FINOVA_CORE,
          tokenProgram: TOKEN_PROGRAM_ID,
          clock: SYSVAR_CLOCK_PUBKEY,
        })
        .instruction();

      const transaction = new Transaction().add(instruction);
      const signature = await this.sendTransaction(transaction);

      return {
        success: true,
        signature,
      };
    } catch (error) {
      console.error('Failed to use special card:', error);
      return {
        success: false,
        error: error instanceof Error ? error.message : 'Unknown error',
      };
    }
  }

  /**
   * Get user's NFT collection
   */
  async getUserNFTs(userPubkey?: PublicKey): Promise<NFTMetadata[]> {
    try {
      const targetUser = userPubkey || this.wallet.publicKey;
      // This would typically query the token accounts and NFT metadata
      // For now, we'll return empty array as placeholder
      return [];
    } catch (error) {
      console.error('Failed to get user NFTs:', error);
      return [];
    }
  }

  // ============================================================================
  // Guild Operations
  // ============================================================================

  /**
   * Create guild
   */
  async createGuild(name: string, description: string): Promise<TransactionResult> {
    try {
      const userPubkey = this.wallet.publicKey;
      const guildKeypair = Keypair.generate();
      const [guildStatePDA] = this.getGuildStatePDA(guildKeypair.publicKey);

      const instruction = await this.coreProgram.methods
        .createGuild(name, description)
        .accounts({
          creator: userPubkey,
          guild: guildKeypair.publicKey,
          guildState: guildStatePDA,
          systemProgram: SystemProgram.programId,
          rent: SYSVAR_RENT_PUBKEY,
        })
        .signers([guildKeypair])
        .instruction();

      const transaction = new Transaction().add(instruction);
      transaction.partialSign(guildKeypair);
      const signature = await this.sendTransaction(transaction);

      return {
        success: true,
        signature,
        accounts: {
          guild: guildKeypair.publicKey,
          guildState: guildStatePDA,
        },
      };
    } catch (error) {
      console.error('Failed to create guild:', error);
      return {
        success: false,
        error: error instanceof Error ? error.message : 'Unknown error',
      };
    }
  }

  /**
   * Join guild
   */
  async joinGuild(guildPubkey: PublicKey): Promise<TransactionResult> {
    try {
      const userPubkey = this.wallet.publicKey;
      const [userStatePDA] = this.getUserStatePDA(userPubkey);
      const [guildStatePDA] = this.getGuildStatePDA(guildPubkey);

      const instruction = await this.coreProgram.methods
        .joinGuild()
        .accounts({
          user: userPubkey,
          userState: userStatePDA,
          guild: guildPubkey,
          guildState: guildStatePDA,
        })
        .instruction();

      const transaction = new Transaction().add(instruction);
      const signature = await this.sendTransaction(transaction);

      return {
        success: true,
        signature,
      };
    } catch (error) {
      console.error('Failed to join guild:', error);
      return {
        success: false,
        error: error instanceof Error ? error.message : 'Unknown error',
      };
    }
  }

  // ============================================================================
  // Utility Methods
  // ============================================================================

  /**
   * Send transaction with proper error handling
   */
  private async sendTransaction(transaction: Transaction): Promise<string> {
    try {
      const signature = await sendAndConfirmTransaction(
        this.connection,
        transaction,
        [this.wallet.payer],
        {
          commitment: this.config.commitment as Commitment,
          preflightCommitment: this.config.preflightCommitment as Commitment,
          skipPreflight: this.config.skipPreflight,
        }
      );
      return signature;
    } catch (error) {
      console.error('Transaction failed:', error);
      throw error;
    }
  }

  /**
   * Get network statistics
   */
  async getNetworkStats(): Promise<NetworkStats> {
    try {
      const networkState = await this.getNetworkState();
      if (!networkState) {
        throw new Error('Network state not found');
      }

      return {
        totalUsers: networkState.totalUsers.toNumber(),
        totalMiners: networkState.totalMiners.toNumber(),
        totalSupply: networkState.totalSupply.toNumber(),
        currentPhase: networkState.currentPhase,
        baseRate: networkState.baseRate,
        avgMiningRate: networkState.avgMiningRate,
        totalStaked: networkState.totalStaked.toNumber(),
        totalGuilds: networkState.totalGuilds.toNumber(),
      };
    } catch (error) {
      console.error('Failed to get network stats:', error);
      return {
        totalUsers: 0,
        totalMiners: 0,
        totalSupply: 0,
        currentPhase: 0,
        baseRate: 0,
        avgMiningRate: 0,
        totalStaked: 0,
        totalGuilds: 0,
      };
    }
  }

  // ============================================================================
  // PDA Helper Methods
  // ============================================================================

  getUserStatePDA(userPubkey: PublicKey): [PublicKey, number] {
    return PublicKey.findProgramAddressSync(
      [Buffer.from('user_state'), userPubkey.toBuffer()],
      PROGRAM_IDS.FINOVA_CORE
    );
  }

  getXPStatePDA(userPubkey: PublicKey): [PublicKey, number] {
    return PublicKey.findProgramAddressSync(
      [Buffer.from('xp_state'), userPubkey.toBuffer()],
      PROGRAM_IDS.FINOVA_CORE
    );
  }

  getReferralStatePDA(userPubkey: PublicKey): [PublicKey, number] {
    return PublicKey.findProgramAddressSync(
      [Buffer.from('referral_state'), userPubkey.toBuffer()],
      PROGRAM_IDS.FINOVA_CORE
    );
  }

  getStakingStatePDA(userPubkey: PublicKey): [PublicKey, number] {
    return PublicKey.findProgramAddressSync(
      [Buffer.from('staking_state'), userPubkey.toBuffer()],
      PROGRAM_IDS.FINOVA_CORE
    );
  }

  getActiveEffectsPDA(userPubkey: PublicKey): [PublicKey, number] {
    return PublicKey.findProgramAddressSync(
      [Buffer.from('active_effects'), userPubkey.toBuffer()],
      PROGRAM_IDS.FINOVA_CORE
    );
  }

  getGuildStatePDA(guildPubkey: PublicKey): [PublicKey, number] {
    return PublicKey.findProgramAddressSync(
      [Buffer.from('guild_state'), guildPubkey.toBuffer()],
      PROGRAM_IDS.FINOVA_CORE
    );
  }

  getNetworkStatePDA(): [PublicKey, number] {
    return PublicKey.findProgramAddressSync(
      [Buffer.from('network_state')],
      PROGRAM_IDS.FINOVA_CORE
    );
  }

  getProposalPDA(proposalId: number): [PublicKey, number] {
    return PublicKey.findProgramAddressSync(
      [Buffer.from('proposal'), Buffer.from(proposalId.toString())],
      PROGRAM_IDS.FINOVA_CORE
    );
  }

  getVoteRecordPDA(proposalId: number, voterPubkey: PublicKey): [PublicKey, number] {
    return PublicKey.findProgramAddressSync(
      [
        Buffer.from('vote_record'),
        Buffer.from(proposalId.toString()),
        voterPubkey.toBuffer(),
      ],
      PROGRAM_IDS.FINOVA_CORE
    );
  }

  // ============================================================================
  // Calculation Helper Methods
  // ============================================================================

  private calculateBaseRate(networkState: NetworkState): number {
    const totalUsers = networkState.totalUsers.toNumber();
    
    // Phase calculation based on user count (from whitepaper)
    if (totalUsers < 100000) {
      return 0.1; // Phase 1: Finizen
    } else if (totalUsers < 1000000) {
      return 0.05; // Phase 2: Growth
    } else if (totalUsers < 10000000) {
      return 0.025; // Phase 3: Maturity
    } else {
      return 0.01; // Phase 4: Stability
    }
  }

  private calculatePioneerBonus(networkState: NetworkState): number {
    const totalUsers = networkState.totalUsers.toNumber();
    return Math.max(1.0, 2.0 - (totalUsers / 1000000));
  }

  private calculateReferralBonus(referralState: ReferralState): number {
    const activeReferrals = referralState.activeReferrals;
    return 1 + (activeReferrals * 0.1);
  }

  private calculateXPMultiplier(xpState: XPState): number {
    const level = xpState.currentLevel;
    return 1.0 + (level / 100); // 1% bonus per level
  }

  private calculateStakingMultiplier(stakingState: StakingState): number {
    const stakedAmount = stakingState.stakedAmount.toNumber();
    
    if (stakedAmount >= 10000 * FINOVA_CONSTANTS.TOKEN_DECIMALS) {
      return 2.0; // 100% bonus for 10K+ stake
    } else if (stakedAmount >= 5000 * FINOVA_CONSTANTS.TOKEN_DECIMALS) {
      return 1.75; // 75% bonus for 5K+ stake
    } else if (stakedAmount >= 1000 * FINOVA_CONSTANTS.TOKEN_DECIMALS) {
      return 1.5; // 50% bonus for 1K+ stake
    } else if (stakedAmount >= 500 * FINOVA_CONSTANTS.TOKEN_DECIMALS) {
      return 1.35; // 35% bonus for 500+ stake
    } else if (stakedAmount >= 100 * FINOVA_CONSTANTS.TOKEN_DECIMALS) {
      return 1.2; // 20% bonus for 100+ stake
    } else {
      return 1.0; // No bonus
    }
  }

  private calculateEffectsMultiplier(activeEffects: ActiveEffectsState): number {
    let multiplier = 1.0;
    const currentTime = Date.now() / 1000;

    // Check for active mining boost effects
    for (const effect of activeEffects.activeEffects) {
      if (effect.endTime.toNumber() > currentTime) {
        switch (effect.effectType) {
          case 'MiningBoost':
            multiplier *= effect.multiplier;
            break;
          case 'XPBoost':
            // XP boost indirectly affects mining through level multiplier
            multiplier *= 1.1;
            break;
          default:
            break;
        }
      }
    }

    return multiplier;
  }

  // ============================================================================
  // Token Account Management
  // ============================================================================

  /**
   * Ensure user has associated token account for FIN token
   */
  async ensureTokenAccount(mint: PublicKey, owner?: PublicKey): Promise<PublicKey> {
    try {
      const ownerPubkey = owner || this.wallet.publicKey;
      const associatedTokenAccount = await getAssociatedTokenAddress(mint, ownerPubkey);

      try {
        await getAccount(this.connection, associatedTokenAccount);
        return associatedTokenAccount;
      } catch (error) {
        if (error instanceof TokenAccountNotFoundError || error instanceof TokenInvalidAccountOwnerError) {
          // Create the associated token account
          const createAccountIx = createAssociatedTokenAccountInstruction(
            this.wallet.publicKey, // payer
            associatedTokenAccount,
            ownerPubkey, // owner
            mint
          );

          const transaction = new Transaction().add(createAccountIx);
          await this.sendTransaction(transaction);
          
          return associatedTokenAccount;
        }
        throw error;
      }
    } catch (error) {
      console.error('Failed to ensure token account:', error);
      throw error;
    }
  }

  /**
   * Get token balance for user
   */
  async getTokenBalance(mint: PublicKey, owner?: PublicKey): Promise<number> {
    try {
      const ownerPubkey = owner || this.wallet.publicKey;
      const associatedTokenAccount = await getAssociatedTokenAddress(mint, ownerPubkey);
      
      try {
        const account = await getAccount(this.connection, associatedTokenAccount);
        return Number(account.amount) / FINOVA_CONSTANTS.TOKEN_DECIMALS;
      } catch (error) {
        if (error instanceof TokenAccountNotFoundError) {
          return 0;
        }
        throw error;
      }
    } catch (error) {
      console.error('Failed to get token balance:', error);
      return 0;
    }
  }

  // ============================================================================
  // Event Listening
  // ============================================================================

  /**
   * Listen for mining events
   */
  onMiningEvent(callback: (event: any) => void): number {
    return this.coreProgram.addEventListener('MiningRewardsClaimed', callback);
  }

  /**
   * Listen for XP events
   */
  onXPEvent(callback: (event: any) => void): number {
    return this.coreProgram.addEventListener('XPUpdated', callback);
  }

  /**
   * Listen for referral events
   */
  onReferralEvent(callback: (event: any) => void): number {
    return this.coreProgram.addEventListener('ReferralProcessed', callback);
  }

  /**
   * Listen for staking events
   */
  onStakingEvent(callback: (event: any) => void): number {
    return this.coreProgram.addEventListener('TokensStaked', callback);
  }

  /**
   * Remove event listener
   */
  removeEventListener(listenerId: number): void {
    this.coreProgram.removeEventListener(listenerId);
  }

  // ============================================================================
  // Governance Operations
  // ============================================================================

  /**
   * Create governance proposal
   */
  async createProposal(
    title: string,
    description: string,
    proposalType: string,
    data: any
  ): Promise<TransactionResult> {
    try {
      const userPubkey = this.wallet.publicKey;
      const [userStatePDA] = this.getUserStatePDA(userPubkey);
      const [stakingStatePDA] = this.getStakingStatePDA(userPubkey);
      
      // Generate unique proposal ID (in practice, this would be managed by the program)
      const proposalId = Math.floor(Math.random() * 1000000);
      const [proposalPDA] = this.getProposalPDA(proposalId);

      const instruction = await this.coreProgram.methods
        .createProposal(proposalId, title, description, proposalType, data)
        .accounts({
          proposer: userPubkey,
          userState: userStatePDA,
          stakingState: stakingStatePDA,
          proposal: proposalPDA,
          systemProgram: SystemProgram.programId,
          rent: SYSVAR_RENT_PUBKEY,
          clock: SYSVAR_CLOCK_PUBKEY,
        })
        .instruction();

      const transaction = new Transaction().add(instruction);
      const signature = await this.sendTransaction(transaction);

      return {
        success: true,
        signature,
        accounts: {
          proposal: proposalPDA,
        },
        data: {
          proposalId,
        },
      };
    } catch (error) {
      console.error('Failed to create proposal:', error);
      return {
        success: false,
        error: error instanceof Error ? error.message : 'Unknown error',
      };
    }
  }

  /**
   * Vote on proposal
   */
  async voteOnProposal(
    proposalId: number,
    vote: boolean,
    votingPower?: number
  ): Promise<TransactionResult> {
    try {
      const userPubkey = this.wallet.publicKey;
      const [userStatePDA] = this.getUserStatePDA(userPubkey);
      const [stakingStatePDA] = this.getStakingStatePDA(userPubkey);
      const [proposalPDA] = this.getProposalPDA(proposalId);
      const [voteRecordPDA] = this.getVoteRecordPDA(proposalId, userPubkey);

      const instruction = await this.coreProgram.methods
        .vote(proposalId, vote, votingPower || null)
        .accounts({
          voter: userPubkey,
          userState: userStatePDA,
          stakingState: stakingStatePDA,
          proposal: proposalPDA,
          voteRecord: voteRecordPDA,
          systemProgram: SystemProgram.programId,
          rent: SYSVAR_RENT_PUBKEY,
          clock: SYSVAR_CLOCK_PUBKEY,
        })
        .instruction();

      const transaction = new Transaction().add(instruction);
      const signature = await this.sendTransaction(transaction);

      return {
        success: true,
        signature,
      };
    } catch (error) {
      console.error('Failed to vote on proposal:', error);
      return {
        success: false,
        error: error instanceof Error ? error.message : 'Unknown error',
      };
    }
  }

  /**
   * Get proposal details
   */
  async getProposal(proposalId: number): Promise<ProposalState | null> {
    try {
      const [proposalPDA] = this.getProposalPDA(proposalId);
      const proposal = await this.coreProgram.account.proposalState.fetch(proposalPDA);
      return proposal as ProposalState;
    } catch (error) {
      console.error('Failed to fetch proposal:', error);
      return null;
    }
  }

  /**
   * Get all active proposals
   */
  async getActiveProposals(): Promise<ProposalState[]> {
    try {
      const proposals = await this.coreProgram.account.proposalState.all();
      return proposals
        .map(p => p.account as ProposalState)
        .filter(p => p.status === 'Active');
    } catch (error) {
      console.error('Failed to fetch active proposals:', error);
      return [];
    }
  }

  // ============================================================================
  // Batch Operations
  // ============================================================================

  /**
   * Execute multiple operations in a single transaction
   */
  async batchExecute(operations: Array<{
    type: string;
    params: any;
  }>): Promise<TransactionResult> {
    try {
      const transaction = new Transaction();
      
      for (const operation of operations) {
        let instruction: TransactionInstruction;
        
        switch (operation.type) {
          case 'claimRewards':
            instruction = await this.buildClaimRewardsInstruction();
            break;
          case 'updateXP':
            instruction = await this.buildUpdateXPInstruction(operation.params);
            break;
          case 'stake':
            instruction = await this.buildStakeInstruction(operation.params);
            break;
          default:
            throw new Error(`Unknown operation type: ${operation.type}`);
        }
        
        transaction.add(instruction);
      }

      const signature = await this.sendTransaction(transaction);

      return {
        success: true,
        signature,
      };
    } catch (error) {
      console.error('Failed to execute batch operations:', error);
      return {
        success: false,
        error: error instanceof Error ? error.message : 'Unknown error',
      };
    }
  }

  // ============================================================================
  // Instruction Builders (Private)
  // ============================================================================

  private async buildClaimRewardsInstruction(): Promise<TransactionInstruction> {
    const userPubkey = this.wallet.publicKey;
    const [userStatePDA] = this.getUserStatePDA(userPubkey);
    const [xpStatePDA] = this.getXPStatePDA(userPubkey);
    const [referralStatePDA] = this.getReferralStatePDA(userPubkey);
    const [stakingStatePDA] = this.getStakingStatePDA(userPubkey);
    const [activeEffectsPDA] = this.getActiveEffectsPDA(userPubkey);
    const [networkStatePDA] = this.getNetworkStatePDA();

    const userTokenAccount = await getAssociatedTokenAddress(
      FINOVA_CONSTANTS.FIN_MINT,
      userPubkey
    );

    return await this.coreProgram.methods
      .claimRewards()
      .accounts({
        user: userPubkey,
        userState: userStatePDA,
        xpState: xpStatePDA,
        referralState: referralStatePDA,
        stakingState: stakingStatePDA,
        activeEffects: activeEffectsPDA,
        networkState: networkStatePDA,
        userTokenAccount,
        finMint: FINOVA_CONSTANTS.FIN_MINT,
        tokenProgram: TOKEN_PROGRAM_ID,
        clock: SYSVAR_CLOCK_PUBKEY,
      })
      .instruction();
  }

  private async buildUpdateXPInstruction(params: {
    activityType: string;
    platform: string;
    contentData: any;
    qualityScore?: number;
  }): Promise<TransactionInstruction> {
    const userPubkey = this.wallet.publicKey;
    const [userStatePDA] = this.getUserStatePDA(userPubkey);
    const [xpStatePDA] = this.getXPStatePDA(userPubkey);
    const [activeEffectsPDA] = this.getActiveEffectsPDA(userPubkey);

    return await this.coreProgram.methods
      .updateXp(
        params.activityType,
        params.platform,
        params.contentData,
        params.qualityScore || null
      )
      .accounts({
        user: userPubkey,
        userState: userStatePDA,
        xpState: xpStatePDA,
        activeEffects: activeEffectsPDA,
        clock: SYSVAR_CLOCK_PUBKEY,
      })
      .instruction();
  }

  private async buildStakeInstruction(params: {
    amount: number;
  }): Promise<TransactionInstruction> {
    const userPubkey = this.wallet.publicKey;
    const [userStatePDA] = this.getUserStatePDA(userPubkey);
    const [stakingStatePDA] = this.getStakingStatePDA(userPubkey);

    const userTokenAccount = await getAssociatedTokenAddress(
      FINOVA_CONSTANTS.FIN_MINT,
      userPubkey
    );

    return await this.coreProgram.methods
      .stake(new BN(params.amount * FINOVA_CONSTANTS.TOKEN_DECIMALS))
      .accounts({
        user: userPubkey,
        userState: userStatePDA,
        stakingState: stakingStatePDA,
        userTokenAccount,
        finMint: FINOVA_CONSTANTS.FIN_MINT,
        tokenProgram: TOKEN_PROGRAM_ID,
        clock: SYSVAR_CLOCK_PUBKEY,
      })
      .instruction();
  }

  // ============================================================================
  // Advanced Analytics
  // ============================================================================

  /**
   * Get comprehensive user analytics
   */
  async getUserAnalytics(userPubkey?: PublicKey): Promise<{
    miningStats: MiningStats;
    xpProgress: {
      currentLevel: number;
      currentXP: number;
      nextLevelXP: number;
      progress: number;
    };
    referralMetrics: {
      directReferrals: number;
      totalNetwork: number;
      referralEarnings: number;
      rpTier: string;
    };
    stakingInfo: {
      stakedAmount: number;
      stakingTier: string;
      stakingRewards: number;
      stakingMultiplier: number;
    };
    nftCollection: {
      totalNFTs: number;
      specialCards: number;
      activeEffects: number;
    };
  }> {
    try {
      const targetUser = userPubkey || this.wallet.publicKey;
      
      // Get all user states
      const userState = await this.getUserState(targetUser);
      const xpState = await this.getXPState(targetUser);
      const referralState = await this.getReferralState(targetUser);
      const stakingState = await this.getStakingState(targetUser);
      const activeEffects = await this.getActiveEffectsState(targetUser);
      
      // Get mining stats
      const miningStats = await this.getMiningStats(targetUser);
      
      // Calculate XP progress
      const xpProgress = this.calculateXPProgress(xpState);
      
      // Calculate referral metrics
      const referralMetrics = this.calculateReferralMetrics(referralState);
      
      // Calculate staking info
      const stakingInfo = this.calculateStakingInfo(stakingState);
      
      // Get NFT collection info
      const nftCollection = await this.getNFTCollectionInfo(targetUser);

      return {
        miningStats,
        xpProgress,
        referralMetrics,
        stakingInfo,
        nftCollection,
      };
    } catch (error) {
      console.error('Failed to get user analytics:', error);
      throw error;
    }
  }

  private calculateXPProgress(xpState: XPState | null): {
    currentLevel: number;
    currentXP: number;
    nextLevelXP: number;
    progress: number;
  } {
    if (!xpState) {
      return {
        currentLevel: 0,
        currentXP: 0,
        nextLevelXP: 100,
        progress: 0,
      };
    }

    const currentLevel = xpState.currentLevel;
    const currentXP = xpState.totalXP.toNumber();
    const nextLevelXP = this.calculateXPRequiredForLevel(currentLevel + 1);
    const currentLevelXP = this.calculateXPRequiredForLevel(currentLevel);
    const progress = ((currentXP - currentLevelXP) / (nextLevelXP - currentLevelXP)) * 100;

    return {
      currentLevel,
      currentXP,
      nextLevelXP,
      progress: Math.min(100, Math.max(0, progress)),
    };
  }

  private calculateXPRequiredForLevel(level: number): number {
    // Exponential XP requirement based on level
    return Math.floor(100 * Math.pow(1.2, level));
  }

  private calculateReferralMetrics(referralState: ReferralState | null): {
    directReferrals: number;
    totalNetwork: number;
    referralEarnings: number;
    rpTier: string;
  } {
    if (!referralState) {
      return {
        directReferrals: 0,
        totalNetwork: 0,
        referralEarnings: 0,
        rpTier: 'Explorer',
      };
    }

    const totalRP = referralState.totalRP.toNumber();
    const rpTier = this.calculateRPTier(totalRP);

    return {
      directReferrals: referralState.directReferrals.length,
      totalNetwork: referralState.totalNetworkSize,
      referralEarnings: referralState.totalReferralEarnings.toNumber(),
      rpTier,
    };
  }

  private calculateRPTier(totalRP: number): string {
    if (totalRP >= 50000) return 'Ambassador';
    if (totalRP >= 15000) return 'Leader';
    if (totalRP >= 5000) return 'Influencer';
    if (totalRP >= 1000) return 'Connector';
    return 'Explorer';
  }

  private calculateStakingInfo(stakingState: StakingState | null): {
    stakedAmount: number;
    stakingTier: string;
    stakingRewards: number;
    stakingMultiplier: number;
  } {
    if (!stakingState) {
      return {
        stakedAmount: 0,
        stakingTier: 'None',
        stakingRewards: 0,
        stakingMultiplier: 1.0,
      };
    }

    const stakedAmount = stakingState.stakedAmount.toNumber() / FINOVA_CONSTANTS.TOKEN_DECIMALS;
    const stakingTier = this.calculateStakingTier(stakedAmount);
    const stakingMultiplier = this.calculateStakingMultiplier(stakingState);

    return {
      stakedAmount,
      stakingTier,
      stakingRewards: stakingState.pendingRewards.toNumber() / FINOVA_CONSTANTS.TOKEN_DECIMALS,
      stakingMultiplier,
    };
  }

  private calculateStakingTier(stakedAmount: number): string {
    if (stakedAmount >= 10000) return 'Diamond';
    if (stakedAmount >= 5000) return 'Platinum';
    if (stakedAmount >= 1000) return 'Gold';
    if (stakedAmount >= 500) return 'Silver';
    if (stakedAmount >= 100) return 'Bronze';
    return 'None';
  }

  private async getNFTCollectionInfo(userPubkey: PublicKey): Promise<{
    totalNFTs: number;
    specialCards: number;
    activeEffects: number;
  }> {
    try {
      const nfts = await this.getUserNFTs(userPubkey);
      const activeEffects = await this.getActiveEffectsState(userPubkey);
      
      const specialCards = nfts.filter(nft => nft.type === 'SpecialCard').length;
      const activeEffectsCount = activeEffects ? activeEffects.activeEffects.length : 0;

      return {
        totalNFTs: nfts.length,
        specialCards,
        activeEffects: activeEffectsCount,
      };
    } catch (error) {
      console.error('Failed to get NFT collection info:', error);
      return {
        totalNFTs: 0,
        specialCards: 0,
        activeEffects: 0,
      };
    }
  }

  // ============================================================================
  // Connection Management
  // ============================================================================

  /**
   * Update connection
   */
  updateConnection(connection: Connection): void {
    this.connection = connection;
    this.provider = new AnchorProvider(
      connection,
      this.wallet,
      {
        commitment: this.config.commitment as Commitment,
        preflightCommitment: this.config.preflightCommitment as Commitment,
        skipPreflight: this.config.skipPreflight,
      }
    );
    this.initializePrograms();
  }

  /**
   * Update wallet
   */
  updateWallet(wallet: anchor.Wallet): void {
    this.wallet = wallet;
    this.provider = new AnchorProvider(
      this.connection,
      wallet,
      {
        commitment: this.config.commitment as Commitment,
        preflightCommitment: this.config.preflightCommitment as Commitment,
        skipPreflight: this.config.skipPreflight,
      }
    );
    this.initializePrograms();
  }

  /**
   * Check connection health
   */
  async checkConnection(): Promise<boolean> {
    try {
      const version = await this.connection.getVersion();
      return !!version;
    } catch (error) {
      console.error('Connection check failed:', error);
      return false;
    }
  }

  // ============================================================================
  // Cleanup
  // ============================================================================

  /**
   * Cleanup resources and event listeners
   */
  cleanup(): void {
    // Remove all event listeners
    try {
      this.coreProgram.removeEventListener();
      this.tokenProgram.removeEventListener();
      this.nftProgram.removeEventListener();
    } catch (error) {
      console.error('Error during cleanup:', error);
    }
  }
}
