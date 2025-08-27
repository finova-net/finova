import { 
  Connection, 
  PublicKey, 
  Transaction, 
  SystemProgram,
  LAMPORTS_PER_SOL,
  sendAndConfirmTransaction,
  Keypair,
  TransactionSignature,
  AccountInfo,
  ParsedAccountData,
  GetProgramAccountsFilter
} from '@solana/web3.js';
import { 
  TOKEN_PROGRAM_ID,
  ASSOCIATED_TOKEN_PROGRAM_ID,
  getAssociatedTokenAddress,
  createAssociatedTokenAccountInstruction,
  createMintToInstruction,
  createBurnInstruction,
  createTransferInstruction,
  getAccount,
  getMint,
  TokenAccount,
  Mint
} from '@solana/spl-token';
import { Program, AnchorProvider, Wallet, web3, BN, IdlAccounts } from '@coral-xyz/anchor';
import { 
  createCreateMetadataAccountV3Instruction,
  PROGRAM_ID as METADATA_PROGRAM_ID,
  createUpdateMetadataAccountV2Instruction
} from '@metaplex-foundation/mpl-token-metadata';
import NodeWallet from '@coral-xyz/anchor/dist/cjs/nodewallet';
import { 
  Logger 
} from '../utils/logger';
import { 
  BLOCKCHAIN_CONFIG,
  PROGRAM_IDS,
  TOKEN_DECIMALS,
  NETWORK_CONSTANTS 
} from '../config/blockchain';
import { 
  MiningData,
  UserProfile,
  StakeAccount,
  RewardPool,
  NFTMetadata,
  TransactionResult,
  BlockchainError,
  MiningCalculation,
  XPData,
  ReferralData
} from '../types/blockchain.types';

interface FinovaCoreProgram {
  mining: any;
  staking: any;
  referral: any;
  xp: any;
  rewards: any;
}

interface FinovaTokenProgram {
  token: any;
  staking: any;
}

interface FinovaNFTProgram {
  nft: any;
  marketplace: any;
}

export class BlockchainService {
  private connection: Connection;
  private provider: AnchorProvider;
  private logger: Logger;
  private coreProgram: Program<FinovaCoreProgram> | null = null;
  private tokenProgram: Program<FinovaTokenProgram> | null = null;
  private nftProgram: Program<FinovaNFTProgram> | null = null;
  private wallet: NodeWallet;

  constructor() {
    this.logger = new Logger('BlockchainService');
    this.connection = new Connection(
      BLOCKCHAIN_CONFIG.RPC_ENDPOINT,
      { 
        commitment: 'confirmed',
        confirmTransactionInitialTimeout: 60000
      }
    );
    
    // Initialize with dummy wallet for read operations
    this.wallet = new NodeWallet(Keypair.generate());
    this.provider = new AnchorProvider(
      this.connection, 
      this.wallet, 
      { commitment: 'confirmed' }
    );
  }

  // ==================== INITIALIZATION ====================

  async initialize(): Promise<void> {
    try {
      await this.loadPrograms();
      await this.validateConnection();
      this.logger.info('Blockchain service initialized successfully');
    } catch (error) {
      this.logger.error('Failed to initialize blockchain service:', error);
      throw new BlockchainError('INITIALIZATION_FAILED', error.message);
    }
  }

  private async loadPrograms(): Promise<void> {
    try {
      // Load IDLs and initialize programs
      // In production, these would be loaded from actual program IDLs
      this.logger.info('Programs loaded successfully');
    } catch (error) {
      throw new BlockchainError('PROGRAM_LOAD_FAILED', error.message);
    }
  }

  private async validateConnection(): Promise<void> {
    const version = await this.connection.getVersion();
    if (!version) {
      throw new BlockchainError('CONNECTION_FAILED', 'Unable to connect to Solana network');
    }
  }

  // ==================== MINING OPERATIONS ====================

  async calculateMiningRate(
    userId: string,
    userLevel: number,
    referralTier: number,
    totalHoldings: number,
    isKYCVerified: boolean
  ): Promise<MiningCalculation> {
    try {
      const networkData = await this.getNetworkStats();
      
      // Base mining calculation (Pi Network inspired)
      const baseRate = this.getCurrentPhaseRate(networkData.totalUsers);
      const pioneerBonus = Math.max(1.0, 2.0 - (networkData.totalUsers / 1000000));
      const referralBonus = this.calculateReferralBonus(referralTier);
      const securityBonus = isKYCVerified ? 1.2 : 0.8;
      const xpMultiplier = 1.0 + (userLevel / 100);
      
      // Exponential regression to prevent whale dominance
      const regressionFactor = Math.exp(-0.001 * totalHoldings);
      
      const hourlyRate = baseRate * pioneerBonus * referralBonus * securityBonus * xpMultiplier * regressionFactor;
      const dailyRate = hourlyRate * 24;
      
      return {
        hourlyRate,
        dailyRate,
        baseRate,
        pioneerBonus,
        referralBonus,
        securityBonus,
        xpMultiplier,
        regressionFactor,
        phase: this.getCurrentPhase(networkData.totalUsers)
      };
    } catch (error) {
      this.logger.error('Mining calculation failed:', error);
      throw new BlockchainError('MINING_CALCULATION_FAILED', error.message);
    }
  }

  async startMining(userWallet: PublicKey): Promise<TransactionResult> {
    try {
      const userAccount = await this.getUserAccount(userWallet);
      if (!userAccount) {
        throw new BlockchainError('USER_NOT_FOUND', 'User account not initialized');
      }

      // Create mining transaction
      const transaction = new Transaction();
      
      // Add mining instruction (would use actual program instruction)
      const instruction = SystemProgram.transfer({
        fromPubkey: userWallet,
        toPubkey: userWallet, // Placeholder
        lamports: 0
      });
      
      transaction.add(instruction);
      
      const signature = await this.sendTransaction(transaction, userWallet);
      
      return {
        success: true,
        signature,
        message: 'Mining started successfully'
      };
    } catch (error) {
      this.logger.error('Mining start failed:', error);
      throw new BlockchainError('MINING_START_FAILED', error.message);
    }
  }

  async claimMiningRewards(userWallet: PublicKey): Promise<TransactionResult> {
    try {
      const pendingRewards = await this.getPendingRewards(userWallet);
      
      if (pendingRewards <= 0) {
        return {
          success: false,
          message: 'No rewards to claim'
        };
      }

      // Create claim transaction
      const transaction = new Transaction();
      const rewardAmount = new BN(pendingRewards * TOKEN_DECIMALS.FIN);
      
      // Mint rewards to user (would use actual program instruction)
      const userTokenAccount = await getAssociatedTokenAddress(
        new PublicKey(PROGRAM_IDS.FIN_TOKEN),
        userWallet
      );
      
      const mintInstruction = createMintToInstruction(
        new PublicKey(PROGRAM_IDS.FIN_TOKEN),
        userTokenAccount,
        new PublicKey(PROGRAM_IDS.MINT_AUTHORITY),
        rewardAmount.toNumber()
      );
      
      transaction.add(mintInstruction);
      
      const signature = await this.sendTransaction(transaction, userWallet);
      
      return {
        success: true,
        signature,
        message: `Claimed ${pendingRewards} $FIN tokens`,
        amount: pendingRewards
      };
    } catch (error) {
      this.logger.error('Reward claim failed:', error);
      throw new BlockchainError('REWARD_CLAIM_FAILED', error.message);
    }
  }

  // ==================== XP SYSTEM ====================

  async updateXP(
    userWallet: PublicKey,
    activityType: string,
    baseXP: number,
    qualityScore: number,
    platformMultiplier: number
  ): Promise<TransactionResult> {
    try {
      const userAccount = await this.getUserAccount(userWallet);
      const currentLevel = userAccount?.level || 1;
      
      // Calculate XP with regression
      const levelProgression = Math.exp(-0.01 * currentLevel);
      const xpGained = Math.floor(baseXP * qualityScore * platformMultiplier * levelProgression);
      
      // Create XP update transaction
      const transaction = new Transaction();
      
      // Would use actual program instruction to update XP
      const instruction = SystemProgram.transfer({
        fromPubkey: userWallet,
        toPubkey: userWallet,
        lamports: 0
      });
      
      transaction.add(instruction);
      
      const signature = await this.sendTransaction(transaction, userWallet);
      
      return {
        success: true,
        signature,
        message: `Gained ${xpGained} XP from ${activityType}`,
        amount: xpGained
      };
    } catch (error) {
      this.logger.error('XP update failed:', error);
      throw new BlockchainError('XP_UPDATE_FAILED', error.message);
    }
  }

  async getUserXPData(userWallet: PublicKey): Promise<XPData> {
    try {
      const userAccount = await this.getUserAccount(userWallet);
      
      if (!userAccount) {
        return {
          totalXP: 0,
          currentLevel: 1,
          xpToNextLevel: 1000,
          miningMultiplier: 1.0,
          tier: 'Bronze I'
        };
      }

      const totalXP = userAccount.totalXP || 0;
      const currentLevel = this.calculateLevel(totalXP);
      const xpToNextLevel = this.calculateXPToNextLevel(currentLevel);
      const miningMultiplier = this.calculateXPMiningMultiplier(currentLevel);
      const tier = this.getXPTier(currentLevel);

      return {
        totalXP,
        currentLevel,
        xpToNextLevel,
        miningMultiplier,
        tier
      };
    } catch (error) {
      this.logger.error('Failed to get XP data:', error);
      throw new BlockchainError('XP_DATA_FETCH_FAILED', error.message);
    }
  }

  // ==================== REFERRAL SYSTEM ====================

  async processReferralReward(
    referrerWallet: PublicKey,
    refereeWallet: PublicKey,
    activityType: string,
    baseReward: number
  ): Promise<TransactionResult> {
    try {
      const referrerData = await this.getReferralData(referrerWallet);
      const referralTier = this.calculateReferralTier(referrerData.totalRP);
      const rewardMultiplier = this.getReferralRewardMultiplier(referralTier);
      
      const rpReward = Math.floor(baseReward * rewardMultiplier);
      
      // Create referral reward transaction
      const transaction = new Transaction();
      
      // Would use actual program instruction to update RP
      const instruction = SystemProgram.transfer({
        fromPubkey: referrerWallet,
        toPubkey: referrerWallet,
        lamports: 0
      });
      
      transaction.add(instruction);
      
      const signature = await this.sendTransaction(transaction, referrerWallet);
      
      return {
        success: true,
        signature,
        message: `Referral reward: ${rpReward} RP`,
        amount: rpReward
      };
    } catch (error) {
      this.logger.error('Referral reward processing failed:', error);
      throw new BlockchainError('REFERRAL_REWARD_FAILED', error.message);
    }
  }

  async getReferralData(userWallet: PublicKey): Promise<ReferralData> {
    try {
      // Fetch referral data from blockchain
      const referralAccount = await this.connection.getAccountInfo(userWallet);
      
      if (!referralAccount) {
        return {
          totalRP: 0,
          tier: 'Explorer',
          directReferrals: 0,
          activeReferrals: 0,
          networkSize: 0,
          miningBonus: 0
        };
      }

      // Parse referral data (would deserialize actual account data)
      return {
        totalRP: 0,
        tier: 'Explorer',
        directReferrals: 0,
        activeReferrals: 0,
        networkSize: 0,
        miningBonus: 0
      };
    } catch (error) {
      this.logger.error('Failed to get referral data:', error);
      throw new BlockchainError('REFERRAL_DATA_FETCH_FAILED', error.message);
    }
  }

  // ==================== STAKING OPERATIONS ====================

  async stakeTokens(
    userWallet: PublicKey,
    amount: number,
    duration: number
  ): Promise<TransactionResult> {
    try {
      const userTokenAccount = await getAssociatedTokenAddress(
        new PublicKey(PROGRAM_IDS.FIN_TOKEN),
        userWallet
      );

      const tokenBalance = await this.getTokenBalance(userWallet, PROGRAM_IDS.FIN_TOKEN);
      
      if (tokenBalance < amount) {
        throw new BlockchainError('INSUFFICIENT_BALANCE', 'Insufficient token balance');
      }

      const transaction = new Transaction();
      const stakeAmount = new BN(amount * TOKEN_DECIMALS.FIN);
      
      // Create staking instruction (would use actual program)
      const instruction = SystemProgram.transfer({
        fromPubkey: userWallet,
        toPubkey: new PublicKey(PROGRAM_IDS.STAKING_POOL),
        lamports: stakeAmount.toNumber()
      });
      
      transaction.add(instruction);
      
      const signature = await this.sendTransaction(transaction, userWallet);
      
      return {
        success: true,
        signature,
        message: `Staked ${amount} $FIN tokens`,
        amount
      };
    } catch (error) {
      this.logger.error('Token staking failed:', error);
      throw new BlockchainError('STAKING_FAILED', error.message);
    }
  }

  async unstakeTokens(userWallet: PublicKey, stakeId: string): Promise<TransactionResult> {
    try {
      const stakeAccount = await this.getStakeAccount(userWallet, stakeId);
      
      if (!stakeAccount) {
        throw new BlockchainError('STAKE_NOT_FOUND', 'Stake account not found');
      }

      if (stakeAccount.lockEndTime > Date.now()) {
        throw new BlockchainError('STAKE_LOCKED', 'Tokens are still locked');
      }

      const transaction = new Transaction();
      
      // Create unstaking instruction
      const instruction = SystemProgram.transfer({
        fromPubkey: new PublicKey(PROGRAM_IDS.STAKING_POOL),
        toPubkey: userWallet,
        lamports: stakeAccount.amount
      });
      
      transaction.add(instruction);
      
      const signature = await this.sendTransaction(transaction, userWallet);
      
      return {
        success: true,
        signature,
        message: `Unstaked ${stakeAccount.amount / TOKEN_DECIMALS.FIN} $FIN tokens`,
        amount: stakeAccount.amount / TOKEN_DECIMALS.FIN
      };
    } catch (error) {
      this.logger.error('Token unstaking failed:', error);
      throw new BlockchainError('UNSTAKING_FAILED', error.message);
    }
  }

  // ==================== NFT OPERATIONS ====================

  async mintSpecialCard(
    userWallet: PublicKey,
    cardType: string,
    metadata: NFTMetadata
  ): Promise<TransactionResult> {
    try {
      const transaction = new Transaction();
      const mintKeypair = Keypair.generate();
      
      // Create mint account
      const createMintInstruction = SystemProgram.createAccount({
        fromPubkey: userWallet,
        newAccountPubkey: mintKeypair.publicKey,
        lamports: await this.connection.getMinimumBalanceForRentExemption(82),
        space: 82,
        programId: TOKEN_PROGRAM_ID
      });
      
      transaction.add(createMintInstruction);
      
      // Create associated token account
      const userTokenAccount = await getAssociatedTokenAddress(
        mintKeypair.publicKey,
        userWallet
      );
      
      const createATAInstruction = createAssociatedTokenAccountInstruction(
        userWallet,
        userTokenAccount,
        userWallet,
        mintKeypair.publicKey
      );
      
      transaction.add(createATAInstruction);
      
      // Mint NFT
      const mintInstruction = createMintToInstruction(
        mintKeypair.publicKey,
        userTokenAccount,
        userWallet,
        1
      );
      
      transaction.add(mintInstruction);
      
      const signature = await this.sendTransaction(transaction, userWallet);
      
      return {
        success: true,
        signature,
        message: `Minted ${cardType} special card`,
        nftMint: mintKeypair.publicKey.toString()
      };
    } catch (error) {
      this.logger.error('NFT minting failed:', error);
      throw new BlockchainError('NFT_MINT_FAILED', error.message);
    }
  }

  async useSpecialCard(
    userWallet: PublicKey,
    nftMint: string,
    cardType: string
  ): Promise<TransactionResult> {
    try {
      const userTokenAccount = await getAssociatedTokenAddress(
        new PublicKey(nftMint),
        userWallet
      );
      
      // Verify ownership
      const tokenAccount = await getAccount(this.connection, userTokenAccount);
      if (tokenAccount.amount < 1) {
        throw new BlockchainError('NFT_NOT_OWNED', 'User does not own this NFT');
      }

      const transaction = new Transaction();
      
      // Burn the NFT (single use)
      const burnInstruction = createBurnInstruction(
        userTokenAccount,
        new PublicKey(nftMint),
        userWallet,
        1
      );
      
      transaction.add(burnInstruction);
      
      const signature = await this.sendTransaction(transaction, userWallet);
      
      // Apply card effects (would be handled by program)
      await this.applyCardEffects(userWallet, cardType);
      
      return {
        success: true,
        signature,
        message: `Used ${cardType} special card`,
        cardType
      };
    } catch (error) {
      this.logger.error('Card usage failed:', error);
      throw new BlockchainError('CARD_USE_FAILED', error.message);
    }
  }

  // ==================== UTILITY METHODS ====================

  private getCurrentPhaseRate(totalUsers: number): number {
    if (totalUsers < 100000) return 0.1;        // Phase 1: Pioneer
    if (totalUsers < 1000000) return 0.05;      // Phase 2: Growth
    if (totalUsers < 10000000) return 0.025;    // Phase 3: Maturity
    return 0.01;                                 // Phase 4: Stability
  }

  private getCurrentPhase(totalUsers: number): string {
    if (totalUsers < 100000) return 'Pioneer';
    if (totalUsers < 1000000) return 'Growth';
    if (totalUsers < 10000000) return 'Maturity';
    return 'Stability';
  }

  private calculateReferralBonus(referralTier: number): number {
    const tierMultipliers = [1.0, 1.2, 1.5, 2.0, 3.0]; // Explorer to Ambassador
    return tierMultipliers[referralTier] || 1.0;
  }

  private calculateLevel(totalXP: number): number {
    // XP requirements: 1000, 2000, 4000, 8000, etc.
    return Math.floor(Math.log2(totalXP / 500 + 1)) + 1;
  }

  private calculateXPToNextLevel(currentLevel: number): number {
    const nextLevelXP = Math.pow(2, currentLevel - 1) * 1000;
    const currentLevelXP = Math.pow(2, currentLevel - 2) * 1000;
    return nextLevelXP - currentLevelXP;
  }

  private calculateXPMiningMultiplier(level: number): number {
    if (level <= 10) return 1.0 + (level - 1) * 0.02;    // 1.0x - 1.18x
    if (level <= 25) return 1.3 + (level - 11) * 0.033;  // 1.3x - 1.8x
    if (level <= 50) return 1.9 + (level - 26) * 0.024;  // 1.9x - 2.5x
    if (level <= 75) return 2.6 + (level - 51) * 0.024;  // 2.6x - 3.2x
    if (level <= 100) return 3.3 + (level - 76) * 0.028; // 3.3x - 4.0x
    return 4.1 + (level - 101) * 0.009;                   // 4.1x - 5.0x
  }

  private getXPTier(level: number): string {
    if (level <= 10) return `Bronze ${['I','II','III','IV','V','VI','VII','VIII','IX','X'][level-1]}`;
    if (level <= 25) return `Silver ${['I','II','III','IV','V','VI','VII','VIII','IX','X','XI','XII','XIII','XIV','XV'][level-11]}`;
    if (level <= 50) return `Gold ${['I','II','III','IV','V','VI','VII','VIII','IX','X','XI','XII','XIII','XIV','XV','XVI','XVII','XVIII','XIX','XX','XXI','XXII','XXIII','XXIV','XXV'][level-26]}`;
    if (level <= 75) return `Platinum ${['I','II','III','IV','V','VI','VII','VIII','IX','X','XI','XII','XIII','XIV','XV','XVI','XVII','XVIII','XIX','XX','XXI','XXII','XXIII','XXIV','XXV'][level-51]}`;
    if (level <= 100) return `Diamond ${['I','II','III','IV','V','VI','VII','VIII','IX','X','XI','XII','XIII','XIV','XV','XVI','XVII','XVIII','XIX','XX','XXI','XXII','XXIII','XXIV','XXV'][level-76]}`;
    return `Mythic ${level - 100}`;
  }

  private calculateReferralTier(totalRP: number): number {
    if (totalRP < 1000) return 0;      // Explorer
    if (totalRP < 5000) return 1;      // Connector
    if (totalRP < 15000) return 2;     // Influencer
    if (totalRP < 50000) return 3;     // Leader
    return 4;                          // Ambassador
  }

  private getReferralRewardMultiplier(tier: number): number {
    const multipliers = [0.1, 0.15, 0.2, 0.25, 0.3]; // 10% to 30%
    return multipliers[tier] || 0.1;
  }

  async getTokenBalance(userWallet: PublicKey, tokenMint: string): Promise<number> {
    try {
      const tokenAccount = await getAssociatedTokenAddress(
        new PublicKey(tokenMint),
        userWallet
      );
      
      const account = await getAccount(this.connection, tokenAccount);
      return Number(account.amount) / TOKEN_DECIMALS.FIN;
    } catch (error) {
      return 0;
    }
  }

  async getUserAccount(userWallet: PublicKey): Promise<UserProfile | null> {
    try {
      const accountInfo = await this.connection.getAccountInfo(userWallet);
      if (!accountInfo) return null;
      
      // Would deserialize actual account data
      return {
        wallet: userWallet.toString(),
        totalXP: 0,
        level: 1,
        totalRP: 0,
        referralTier: 0,
        totalHoldings: 0,
        isKYCVerified: false,
        lastMiningClaim: Date.now(),
        stakingAccounts: []
      };
    } catch (error) {
      return null;
    }
  }

  private async getNetworkStats(): Promise<{ totalUsers: number; totalSupply: number }> {
    // Would fetch from program state
    return {
      totalUsers: 50000,  // Placeholder
      totalSupply: 5000000
    };
  }

  private async getPendingRewards(userWallet: PublicKey): Promise<number> {
    // Calculate pending rewards based on last claim time and mining rate
    const userAccount = await this.getUserAccount(userWallet);
    if (!userAccount) return 0;
    
    const timeSinceLastClaim = Date.now() - userAccount.lastMiningClaim;
    const hoursElapsed = timeSinceLastClaim / (1000 * 60 * 60);
    
    const miningCalc = await this.calculateMiningRate(
      userAccount.wallet,
      userAccount.level,
      userAccount.referralTier,
      userAccount.totalHoldings,
      userAccount.isKYCVerified
    );
    
    return Math.min(hoursElapsed * miningCalc.hourlyRate, miningCalc.dailyRate);
  }

  private async getStakeAccount(userWallet: PublicKey, stakeId: string): Promise<StakeAccount | null> {
    // Would fetch from actual stake program
    return null;
  }

  private async applyCardEffects(userWallet: PublicKey, cardType: string): Promise<void> {
    // Apply card effects to user account
    this.logger.info(`Applied ${cardType} effects for ${userWallet.toString()}`);
  }

  private async sendTransaction(transaction: Transaction, payer: PublicKey): Promise<TransactionSignature> {
    try {
      transaction.feePayer = payer;
      transaction.recentBlockhash = (await this.connection.getLatestBlockhash()).blockhash;
      
      // In production, would be signed by actual user wallet
      const signature = await this.connection.sendTransaction(transaction, [this.wallet.payer], {
        skipPreflight: false,
        preflightCommitment: 'confirmed'
      });
      
      await this.connection.confirmTransaction(signature, 'confirmed');
      return signature;
    } catch (error) {
      this.logger.error('Transaction failed:', error);
      throw new BlockchainError('TRANSACTION_FAILED', error.message);
    }
  }

  // ==================== HEALTH CHECK ====================

  async healthCheck(): Promise<{ status: string; blockHeight: number; networkVersion: string }> {
    try {
      const blockHeight = await this.connection.getBlockHeight();
      const version = await this.connection.getVersion();
      
      return {
        status: 'healthy',
        blockHeight,
        networkVersion: version['solana-core']
      };
    } catch (error) {
      return {
        status: 'unhealthy',
        blockHeight: 0,
        networkVersion: 'unknown'
      };
    }
  }
}
