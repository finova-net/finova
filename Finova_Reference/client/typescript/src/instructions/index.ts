// client/typescript/src/instructions/index.ts
/**
 * Finova Network TypeScript SDK - Instructions Module
 * 
 * This module provides instruction builders for all Finova Network smart contract programs.
 * It handles the complex interactions between finova-core, finova-token, finova-nft, and other programs.
 * 
 * @version 1.0.0
 * @author Finova Network Development Team
 */

import { 
  PublicKey, 
  SystemProgram, 
  SYSVAR_RENT_PUBKEY,
  Transaction,
  TransactionInstruction,
  SYSVAR_CLOCK_PUBKEY
} from '@solana/web3.js';
import { 
  TOKEN_PROGRAM_ID, 
  ASSOCIATED_TOKEN_PROGRAM_ID,
  getAssociatedTokenAddress
} from '@solana/spl-token';
import { BN } from '@project-serum/anchor';
import * as anchor from '@project-serum/anchor';

// Import types
import { 
  NetworkParams,
  UserInitParams,
  MiningParams,
  StakingParams,
  XPUpdateParams,
  ReferralParams,
  GuildParams,
  GovernanceParams,
  NFTParams,
  SpecialCardParams,
  RewardClaimParams
} from '../types';

// Import constants
import { 
  FINOVA_CORE_PROGRAM_ID,
  FINOVA_TOKEN_PROGRAM_ID,
  FINOVA_NFT_PROGRAM_ID,
  FINOVA_DEFI_PROGRAM_ID,
  FINOVA_ORACLE_PROGRAM_ID,
  FINOVA_BRIDGE_PROGRAM_ID,
  FIN_TOKEN_MINT,
  NETWORK_STATE_SEED,
  USER_STATE_SEED,
  XP_STATE_SEED,
  REFERRAL_STATE_SEED,
  STAKING_STATE_SEED,
  ACTIVE_EFFECTS_STATE_SEED,
  GUILD_STATE_SEED,
  PROPOSAL_STATE_SEED,
  TOKEN_STATE_SEED
} from '../constants';

/**
 * Core instruction builders for Finova Network
 */
export class FinovaCoreInstructions {
  
  /**
   * Initialize the global network state
   * This is called once during deployment by the admin
   */
  static async initializeNetwork(
    program: anchor.Program,
    admin: PublicKey,
    params: NetworkParams
  ): Promise<TransactionInstruction> {
    const [networkState] = await PublicKey.findProgramAddress(
      [Buffer.from(NETWORK_STATE_SEED)],
      program.programId
    );

    return program.methods
      .initialize(
        params.baseMinimumRate,
        params.pioneerBonusDecayRate,
        params.referralBonusRate,
        params.securityBonusRate,
        params.regressionFactor,
        params.maxDailyRewards,
        params.qualityThreshold
      )
      .accounts({
        networkState,
        admin,
        systemProgram: SystemProgram.programId,
        rent: SYSVAR_RENT_PUBKEY,
      })
      .instruction();
  }

  /**
   * Initialize a new user with all required state accounts
   * Creates UserState, XPState, ReferralState, StakingState, and ActiveEffectsState
   */
  static async initializeUser(
    program: anchor.Program,
    user: PublicKey,
    referrer?: PublicKey
  ): Promise<TransactionInstruction> {
    const [networkState] = await PublicKey.findProgramAddress(
      [Buffer.from(NETWORK_STATE_SEED)],
      program.programId
    );

    const [userState] = await PublicKey.findProgramAddress(
      [Buffer.from(USER_STATE_SEED), user.toBuffer()],
      program.programId
    );

    const [xpState] = await PublicKey.findProgramAddress(
      [Buffer.from(XP_STATE_SEED), user.toBuffer()],
      program.programId
    );

    const [referralState] = await PublicKey.findProgramAddress(
      [Buffer.from(REFERRAL_STATE_SEED), user.toBuffer()],
      program.programId
    );

    const [stakingState] = await PublicKey.findProgramAddress(
      [Buffer.from(STAKING_STATE_SEED), user.toBuffer()],
      program.programId
    );

    const [activeEffectsState] = await PublicKey.findProgramAddress(
      [Buffer.from(ACTIVE_EFFECTS_STATE_SEED), user.toBuffer()],
      program.programId
    );

    // If there's a referrer, get their referral state
    let referrerReferralState = null;
    if (referrer) {
      [referrerReferralState] = await PublicKey.findProgramAddress(
        [Buffer.from(REFERRAL_STATE_SEED), referrer.toBuffer()],
        program.programId
      );
    }

    const accounts: any = {
      networkState,
      userState,
      xpState,
      referralState,
      stakingState,
      activeEffectsState,
      user,
      systemProgram: SystemProgram.programId,
      rent: SYSVAR_RENT_PUBKEY,
    };

    if (referrerReferralState) {
      accounts.referrerReferralState = referrerReferralState;
    }

    return program.methods
      .initializeUser(referrer)
      .accounts(accounts)
      .instruction();
  }

  /**
   * Claim mining rewards with comprehensive calculation
   * Integrates XP, RP, staking bonuses, and active card effects
   */
  static async claimRewards(
    program: anchor.Program,
    tokenProgram: anchor.Program,
    user: PublicKey
  ): Promise<TransactionInstruction> {
    const [networkState] = await PublicKey.findProgramAddress(
      [Buffer.from(NETWORK_STATE_SEED)],
      program.programId
    );

    const [userState] = await PublicKey.findProgramAddress(
      [Buffer.from(USER_STATE_SEED), user.toBuffer()],
      program.programId
    );

    const [xpState] = await PublicKey.findProgramAddress(
      [Buffer.from(XP_STATE_SEED), user.toBuffer()],
      program.programId
    );

    const [referralState] = await PublicKey.findProgramAddress(
      [Buffer.from(REFERRAL_STATE_SEED), user.toBuffer()],
      program.programId
    );

    const [stakingState] = await PublicKey.findProgramAddress(
      [Buffer.from(STAKING_STATE_SEED), user.toBuffer()],
      program.programId
    );

    const [activeEffectsState] = await PublicKey.findProgramAddress(
      [Buffer.from(ACTIVE_EFFECTS_STATE_SEED), user.toBuffer()],
      program.programId
    );

    // Get user's token account
    const userTokenAccount = await getAssociatedTokenAddress(
      FIN_TOKEN_MINT,
      user
    );

    // Get token mint PDA for CPI authority
    const [tokenMintPda] = await PublicKey.findProgramAddress(
      [Buffer.from(TOKEN_STATE_SEED)],
      tokenProgram.programId
    );

    return program.methods
      .claimRewards()
      .accounts({
        networkState,
        userState,
        xpState,
        referralState,
        stakingState,
        activeEffectsState,
        user,
        userTokenAccount,
        finTokenMint: FIN_TOKEN_MINT,
        tokenMintPda,
        tokenProgram: tokenProgram.programId,
        clock: SYSVAR_CLOCK_PUBKEY,
      })
      .instruction();
  }

  /**
   * Update user's XP based on social media activity
   * Includes quality scoring and platform multipliers
   */
  static async updateXP(
    program: anchor.Program,
    user: PublicKey,
    params: XPUpdateParams
  ): Promise<TransactionInstruction> {
    const [networkState] = await PublicKey.findProgramAddress(
      [Buffer.from(NETWORK_STATE_SEED)],
      program.programId
    );

    const [userState] = await PublicKey.findProgramAddress(
      [Buffer.from(USER_STATE_SEED), user.toBuffer()],
      program.programId
    );

    const [xpState] = await PublicKey.findProgramAddress(
      [Buffer.from(XP_STATE_SEED), user.toBuffer()],
      program.programId
    );

    return program.methods
      .updateXp(
        params.activityType,
        params.baseXp,
        params.platformMultiplier,
        params.qualityScore,
        params.contentHash,
        params.engagement
      )
      .accounts({
        networkState,
        userState,
        xpState,
        user,
        clock: SYSVAR_CLOCK_PUBKEY,
      })
      .instruction();
  }

  /**
   * Stake FIN tokens for enhanced rewards
   * Includes tier-based benefits and liquid staking
   */
  static async stakeTokens(
    program: anchor.Program,
    user: PublicKey,
    amount: BN,
    duration: number
  ): Promise<TransactionInstruction> {
    const [userState] = await PublicKey.findProgramAddress(
      [Buffer.from(USER_STATE_SEED), user.toBuffer()],
      program.programId
    );

    const [stakingState] = await PublicKey.findProgramAddress(
      [Buffer.from(STAKING_STATE_SEED), user.toBuffer()],
      program.programId
    );

    const userTokenAccount = await getAssociatedTokenAddress(
      FIN_TOKEN_MINT,
      user
    );

    // Staking pool account (controlled by program)
    const [stakingPool] = await PublicKey.findProgramAddress(
      [Buffer.from("staking_pool")],
      program.programId
    );

    return program.methods
      .stake(amount, duration)
      .accounts({
        userState,
        stakingState,
        user,
        userTokenAccount,
        stakingPool,
        finTokenMint: FIN_TOKEN_MINT,
        tokenProgram: TOKEN_PROGRAM_ID,
        clock: SYSVAR_CLOCK_PUBKEY,
      })
      .instruction();
  }

  /**
   * Update referral network and calculate RP
   * Handles multi-level referral rewards
   */
  static async updateReferral(
    program: anchor.Program,
    user: PublicKey,
    referredUser: PublicKey,
    activityType: number,
    rewardAmount: BN
  ): Promise<TransactionInstruction> {
    const [networkState] = await PublicKey.findProgramAddress(
      [Buffer.from(NETWORK_STATE_SEED)],
      program.programId
    );

    const [userReferralState] = await PublicKey.findProgramAddress(
      [Buffer.from(REFERRAL_STATE_SEED), user.toBuffer()],
      program.programId
    );

    const [referredUserState] = await PublicKey.findProgramAddress(
      [Buffer.from(USER_STATE_SEED), referredUser.toBuffer()],
      program.programId
    );

    return program.methods
      .updateReferral(activityType, rewardAmount)
      .accounts({
        networkState,
        userReferralState,
        referredUserState,
        user,
        referredUser,
        clock: SYSVAR_CLOCK_PUBKEY,
      })
      .instruction();
  }

  /**
   * Create a new guild for community participation
   */
  static async createGuild(
    program: anchor.Program,
    creator: PublicKey,
    params: GuildParams
  ): Promise<TransactionInstruction> {
    const [networkState] = await PublicKey.findProgramAddress(
      [Buffer.from(NETWORK_STATE_SEED)],
      program.programId
    );

    const [guildState] = await PublicKey.findProgramAddress(
      [Buffer.from(GUILD_STATE_SEED), Buffer.from(params.name)],
      program.programId
    );

    const [userState] = await PublicKey.findProgramAddress(
      [Buffer.from(USER_STATE_SEED), creator.toBuffer()],
      program.programId
    );

    return program.methods
      .createGuild(
        params.name,
        params.description,
        params.minLevel,
        params.maxMembers,
        params.isPrivate
      )
      .accounts({
        networkState,
        guildState,
        userState,
        creator,
        systemProgram: SystemProgram.programId,
        rent: SYSVAR_RENT_PUBKEY,
      })
      .instruction();
  }

  /**
   * Join an existing guild
   */
  static async joinGuild(
    program: anchor.Program,
    user: PublicKey,
    guildName: string
  ): Promise<TransactionInstruction> {
    const [guildState] = await PublicKey.findProgramAddress(
      [Buffer.from(GUILD_STATE_SEED), Buffer.from(guildName)],
      program.programId
    );

    const [userState] = await PublicKey.findProgramAddress(
      [Buffer.from(USER_STATE_SEED), user.toBuffer()],
      program.programId
    );

    const [xpState] = await PublicKey.findProgramAddress(
      [Buffer.from(XP_STATE_SEED), user.toBuffer()],
      program.programId
    );

    return program.methods
      .joinGuild(guildName)
      .accounts({
        guildState,
        userState,
        xpState,
        user,
      })
      .instruction();
  }

  /**
   * Create a governance proposal
   */
  static async createProposal(
    program: anchor.Program,
    proposer: PublicKey,
    params: GovernanceParams
  ): Promise<TransactionInstruction> {
    const [networkState] = await PublicKey.findProgramAddress(
      [Buffer.from(NETWORK_STATE_SEED)],
      program.programId
    );

    const [proposalState] = await PublicKey.findProgramAddress(
      [Buffer.from(PROPOSAL_STATE_SEED), Buffer.from(params.title)],
      program.programId
    );

    const [stakingState] = await PublicKey.findProgramAddress(
      [Buffer.from(STAKING_STATE_SEED), proposer.toBuffer()],
      program.programId
    );

    return program.methods
      .createProposal(
        params.title,
        params.description,
        params.proposalType,
        params.executionDelay,
        params.votingPeriod
      )
      .accounts({
        networkState,
        proposalState,
        stakingState,
        proposer,
        systemProgram: SystemProgram.programId,
        rent: SYSVAR_RENT_PUBKEY,
        clock: SYSVAR_CLOCK_PUBKEY,
      })
      .instruction();
  }

  /**
   * Vote on a governance proposal
   */
  static async vote(
    program: anchor.Program,
    voter: PublicKey,
    proposalTitle: string,
    voteType: number,
    weight: BN
  ): Promise<TransactionInstruction> {
    const [proposalState] = await PublicKey.findProgramAddress(
      [Buffer.from(PROPOSAL_STATE_SEED), Buffer.from(proposalTitle)],
      program.programId
    );

    const [stakingState] = await PublicKey.findProgramAddress(
      [Buffer.from(STAKING_STATE_SEED), voter.toBuffer()],
      program.programId
    );

    const [voteRecord] = await PublicKey.findProgramAddress(
      [
        Buffer.from("vote_record"),
        proposalState.toBuffer(),
        voter.toBuffer()
      ],
      program.programId
    );

    return program.methods
      .vote(proposalTitle, voteType, weight)
      .accounts({
        proposalState,
        stakingState,
        voteRecord,
        voter,
        systemProgram: SystemProgram.programId,
        rent: SYSVAR_RENT_PUBKEY,
        clock: SYSVAR_CLOCK_PUBKEY,
      })
      .instruction();
  }

  /**
   * Endpoint for NFT program to apply special card effects
   * This is called via CPI from finova-nft
   */
  static async useCard(
    program: anchor.Program,
    user: PublicKey,
    effectType: number,
    multiplier: number,
    duration: BN
  ): Promise<TransactionInstruction> {
    const [userState] = await PublicKey.findProgramAddress(
      [Buffer.from(USER_STATE_SEED), user.toBuffer()],
      program.programId
    );

    const [activeEffectsState] = await PublicKey.findProgramAddress(
      [Buffer.from(ACTIVE_EFFECTS_STATE_SEED), user.toBuffer()],
      program.programId
    );

    return program.methods
      .useCard(effectType, multiplier, duration)
      .accounts({
        userState,
        activeEffectsState,
        user,
        clock: SYSVAR_CLOCK_PUBKEY,
      })
      .instruction();
  }
}

/**
 * Token instruction builders for Finova Network
 */
export class FinovaTokenInstructions {
  
  /**
   * Initialize the FIN token mint
   * Called once during deployment
   */
  static async initializeMint(
    program: anchor.Program,
    admin: PublicKey
  ): Promise<TransactionInstruction> {
    const [tokenState] = await PublicKey.findProgramAddress(
      [Buffer.from(TOKEN_STATE_SEED)],
      program.programId
    );

    return program.methods
      .initialize()
      .accounts({
        tokenState,
        finTokenMint: FIN_TOKEN_MINT,
        admin,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
        rent: SYSVAR_RENT_PUBKEY,
      })
      .instruction();
  }

  /**
   * Mint rewards to user (called via CPI from finova-core)
   * This function is permissioned and can only be called by finova-core
   */
  static async mintRewards(
    program: anchor.Program,
    recipient: PublicKey,
    amount: BN
  ): Promise<TransactionInstruction> {
    const [tokenState] = await PublicKey.findProgramAddress(
      [Buffer.from(TOKEN_STATE_SEED)],
      program.programId
    );

    const recipientTokenAccount = await getAssociatedTokenAddress(
      FIN_TOKEN_MINT,
      recipient
    );

    return program.methods
      .mintRewards(amount)
      .accounts({
        tokenState,
        finTokenMint: FIN_TOKEN_MINT,
        recipientTokenAccount,
        recipient,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .instruction();
  }

  /**
   * Burn tokens for deflationary mechanisms
   */
  static async burnTokens(
    program: anchor.Program,
    owner: PublicKey,
    amount: BN
  ): Promise<TransactionInstruction> {
    const [tokenState] = await PublicKey.findProgramAddress(
      [Buffer.from(TOKEN_STATE_SEED)],
      program.programId
    );

    const ownerTokenAccount = await getAssociatedTokenAddress(
      FIN_TOKEN_MINT,
      owner
    );

    return program.methods
      .burnTokens(amount)
      .accounts({
        tokenState,
        finTokenMint: FIN_TOKEN_MINT,
        ownerTokenAccount,
        owner,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .instruction();
  }
}

/**
 * NFT instruction builders for Finova Network
 */
export class FinovaNFTInstructions {

  /**
   * Create an NFT collection
   */
  static async createCollection(
    program: anchor.Program,
    creator: PublicKey,
    params: NFTParams
  ): Promise<TransactionInstruction> {
    const [collectionState] = await PublicKey.findProgramAddress(
      [Buffer.from("collection"), Buffer.from(params.symbol)],
      program.programId
    );

    return program.methods
      .createCollection(
        params.name,
        params.symbol,
        params.description,
        params.image,
        params.maxSupply
      )
      .accounts({
        collectionState,
        creator,
        systemProgram: SystemProgram.programId,
        rent: SYSVAR_RENT_PUBKEY,
      })
      .instruction();
  }

  /**
   * Mint a special card NFT
   */
  static async mintSpecialCard(
    program: anchor.Program,
    recipient: PublicKey,
    params: SpecialCardParams
  ): Promise<TransactionInstruction> {
    const [collectionState] = await PublicKey.findProgramAddress(
      [Buffer.from("collection"), Buffer.from("FINOVA_CARDS")],
      program.programId
    );

    const [cardState] = await PublicKey.findProgramAddress(
      [
        Buffer.from("special_card"),
        Buffer.from(params.cardType.toString()),
        recipient.toBuffer()
      ],
      program.programId
    );

    return program.methods
      .mintSpecialCard(
        params.cardType,
        params.effectType,
        params.multiplier,
        params.duration,
        params.rarity,
        params.metadata
      )
      .accounts({
        collectionState,
        cardState,
        recipient,
        systemProgram: SystemProgram.programId,
        rent: SYSVAR_RENT_PUBKEY,
      })
      .instruction();
  }

  /**
   * Use a special card and apply its effects
   * Makes CPI call to finova-core
   */
  static async useSpecialCard(
    program: anchor.Program,
    coreProgram: anchor.Program,
    user: PublicKey,
    cardType: number
  ): Promise<TransactionInstruction> {
    const [cardState] = await PublicKey.findProgramAddress(
      [
        Buffer.from("special_card"),
        Buffer.from(cardType.toString()),
        user.toBuffer()
      ],
      program.programId
    );

    // Core program accounts for CPI
    const [userState] = await PublicKey.findProgramAddress(
      [Buffer.from(USER_STATE_SEED), user.toBuffer()],
      coreProgram.programId
    );

    const [activeEffectsState] = await PublicKey.findProgramAddress(
      [Buffer.from(ACTIVE_EFFECTS_STATE_SEED), user.toBuffer()],
      coreProgram.programId
    );

    return program.methods
      .useSpecialCard(cardType)
      .accounts({
        cardState,
        userState,
        activeEffectsState,
        user,
        coreProgram: coreProgram.programId,
        clock: SYSVAR_CLOCK_PUBKEY,
      })
      .instruction();
  }

  /**
   * List NFT on marketplace
   */
  static async listNFT(
    program: anchor.Program,
    seller: PublicKey,
    nftMint: PublicKey,
    price: BN
  ): Promise<TransactionInstruction> {
    const [marketplaceListing] = await PublicKey.findProgramAddress(
      [Buffer.from("marketplace"), nftMint.toBuffer()],
      program.programId
    );

    const sellerNftAccount = await getAssociatedTokenAddress(
      nftMint,
      seller
    );

    const [escrowAccount] = await PublicKey.findProgramAddress(
      [Buffer.from("escrow"), nftMint.toBuffer()],
      program.programId
    );

    return program.methods
      .listNft(price)
      .accounts({
        marketplaceListing,
        sellerNftAccount,
        escrowAccount,
        nftMint,
        seller,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
        rent: SYSVAR_RENT_PUBKEY,
      })
      .instruction();
  }

  /**
   * Buy NFT from marketplace
   */
  static async buyNFT(
    program: anchor.Program,
    buyer: PublicKey,
    nftMint: PublicKey
  ): Promise<TransactionInstruction> {
    const [marketplaceListing] = await PublicKey.findProgramAddress(
      [Buffer.from("marketplace"), nftMint.toBuffer()],
      program.programId
    );

    const buyerNftAccount = await getAssociatedTokenAddress(
      nftMint,
      buyer
    );

    const buyerTokenAccount = await getAssociatedTokenAddress(
      FIN_TOKEN_MINT,
      buyer
    );

    const [escrowAccount] = await PublicKey.findProgramAddress(
      [Buffer.from("escrow"), nftMint.toBuffer()],
      program.programId
    );

    return program.methods
      .buyNft()
      .accounts({
        marketplaceListing,
        buyerNftAccount,
        buyerTokenAccount,
        escrowAccount,
        nftMint,
        finTokenMint: FIN_TOKEN_MINT,
        buyer,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
        rent: SYSVAR_RENT_PUBKEY,
      })
      .instruction();
  }
}

/**
 * DeFi instruction builders (AMM functionality)
 */
export class FinovaDeFiInstructions {

  /**
   * Create a liquidity pool
   */
  static async createPool(
    program: anchor.Program,
    creator: PublicKey,
    tokenAMint: PublicKey,
    tokenBMint: PublicKey,
    fee: number
  ): Promise<TransactionInstruction> {
    const [poolState] = await PublicKey.findProgramAddress(
      [
        Buffer.from("pool"),
        tokenAMint.toBuffer(),
        tokenBMint.toBuffer()
      ],
      program.programId
    );

    return program.methods
      .createPool(fee)
      .accounts({
        poolState,
        tokenAMint,
        tokenBMint,
        creator,
        systemProgram: SystemProgram.programId,
        rent: SYSVAR_RENT_PUBKEY,
      })
      .instruction();
  }

  /**
   * Add liquidity to pool
   */
  static async addLiquidity(
    program: anchor.Program,
    provider: PublicKey,
    tokenAMint: PublicKey,
    tokenBMint: PublicKey,
    amountA: BN,
    amountB: BN
  ): Promise<TransactionInstruction> {
    const [poolState] = await PublicKey.findProgramAddress(
      [
        Buffer.from("pool"),
        tokenAMint.toBuffer(),
        tokenBMint.toBuffer()
      ],
      program.programId
    );

    const providerTokenAAccount = await getAssociatedTokenAddress(
      tokenAMint,
      provider
    );

    const providerTokenBAccount = await getAssociatedTokenAddress(
      tokenBMint,
      provider
    );

    return program.methods
      .addLiquidity(amountA, amountB)
      .accounts({
        poolState,
        providerTokenAAccount,
        providerTokenBAccount,
        tokenAMint,
        tokenBMint,
        provider,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .instruction();
  }

  /**
   * Swap tokens in pool
   */
  static async swap(
    program: anchor.Program,
    trader: PublicKey,
    tokenInMint: PublicKey,
    tokenOutMint: PublicKey,
    amountIn: BN,
    minimumAmountOut: BN
  ): Promise<TransactionInstruction> {
    const [poolState] = await PublicKey.findProgramAddress(
      [
        Buffer.from("pool"),
        tokenInMint.toBuffer(),
        tokenOutMint.toBuffer()
      ],
      program.programId
    );

    const traderTokenInAccount = await getAssociatedTokenAddress(
      tokenInMint,
      trader
    );

    const traderTokenOutAccount = await getAssociatedTokenAddress(
      tokenOutMint,
      trader
    );

    return program.methods
      .swap(amountIn, minimumAmountOut)
      .accounts({
        poolState,
        traderTokenInAccount,
        traderTokenOutAccount,
        tokenInMint,
        tokenOutMint,
        trader,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .instruction();
  }
}

/**
 * Oracle instruction builders
 */
export class FinovaOracleInstructions {

  /**
   * Initialize price feed
   */
  static async initializeFeed(
    program: anchor.Program,
    admin: PublicKey,
    symbol: string
  ): Promise<TransactionInstruction> {
    const [priceFeed] = await PublicKey.findProgramAddress(
      [Buffer.from("price_feed"), Buffer.from(symbol)],
      program.programId
    );

    return program.methods
      .initializeFeed(symbol)
      .accounts({
        priceFeed,
        admin,
        systemProgram: SystemProgram.programId,
        rent: SYSVAR_RENT_PUBKEY,
      })
      .instruction();
  }

  /**
   * Update price feed (admin only for mock implementation)
   */
  static async updatePrice(
    program: anchor.Program,
    admin: PublicKey,
    symbol: string,
    price: BN,
    confidence: BN
  ): Promise<TransactionInstruction> {
    const [priceFeed] = await PublicKey.findProgramAddress(
      [Buffer.from("price_feed"), Buffer.from(symbol)],
      program.programId
    );

    return program.methods
      .updatePrice(symbol, price, confidence)
      .accounts({
        priceFeed,
        admin,
        clock: SYSVAR_CLOCK_PUBKEY,
      })
      .instruction();
  }
}

/**
 * Bridge instruction builders
 */
export class FinovaBridgeInstructions {

  /**
   * Initialize bridge configuration
   */
  static async initializeBridge(
    program: anchor.Program,
    admin: PublicKey,
    targetChain: string,
    minimumAmount: BN
  ): Promise<TransactionInstruction> {
    const [bridgeConfig] = await PublicKey.findProgramAddress(
      [Buffer.from("bridge_config"), Buffer.from(targetChain)],
      program.programId
    );

    return program.methods
      .initializeBridge(targetChain, minimumAmount)
      .accounts({
        bridgeConfig,
        admin,
        systemProgram: SystemProgram.programId,
        rent: SYSVAR_RENT_PUBKEY,
      })
      .instruction();
  }

  /**
   * Lock tokens for cross-chain transfer
   */
  static async lockTokens(
    program: anchor.Program,
    user: PublicKey,
    amount: BN,
    targetChain: string,
    targetAddress: string
  ): Promise<TransactionInstruction> {
    const [bridgeConfig] = await PublicKey.findProgramAddress(
      [Buffer.from("bridge_config"), Buffer.from(targetChain)],
      program.programId
    );

    const [lockedTokens] = await PublicKey.findProgramAddress(
      [
        Buffer.from("locked_tokens"),
        user.toBuffer(),
        Buffer.from(targetChain)
      ],
      program.programId
    );

    const userTokenAccount = await getAssociatedTokenAddress(
      FIN_TOKEN_MINT,
      user
    );

    const [vaultAccount] = await PublicKey.findProgramAddress(
      [Buffer.from("vault"), Buffer.from(targetChain)],
      program.programId
    );

    return program.methods
      .lockTokens(amount, targetChain, targetAddress)
      .accounts({
        bridgeConfig,
        lockedTokens,
        userTokenAccount,
        vaultAccount,
        finTokenMint: FIN_TOKEN_MINT,
        user,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
        rent: SYSVAR_RENT_PUBKEY,
        clock: SYSVAR_CLOCK_PUBKEY,
      })
      .instruction();
  }

  /**
   * Unlock tokens from cross-chain transfer (admin only for mock)
   */
  static async unlockTokens(
    program: anchor.Program,
    admin: PublicKey,
    recipient: PublicKey,
    amount: BN,
    targetChain: string,
    proof: Buffer
  ): Promise<TransactionInstruction> {
    const [bridgeConfig] = await PublicKey.findProgramAddress(
      [Buffer.from("bridge_config"), Buffer.from(targetChain)],
      program.programId
    );

    const recipientTokenAccount = await getAssociatedTokenAddress(
      FIN_TOKEN_MINT,
      recipient
    );

    const [vaultAccount] = await PublicKey.findProgramAddress(
      [Buffer.from("vault"), Buffer.from(targetChain)],
      program.programId
    );

    return program.methods
      .unlockTokens(amount, targetChain, proof)
      .accounts({
        bridgeConfig,
        recipientTokenAccount,
        vaultAccount,
        finTokenMint: FIN_TOKEN_MINT,
        admin,
        recipient,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .instruction();
  }
}

/**
 * Utility functions for instruction building
 */
export class InstructionUtils {

  /**
   * Create associated token account if it doesn't exist
   */
  static async createAssociatedTokenAccountIfNeeded(
    mint: PublicKey,
    owner: PublicKey,
    payer: PublicKey
  ): Promise<TransactionInstruction | null> {
    try {
      const associatedTokenAddress = await getAssociatedTokenAddress(mint, owner);
      
      // In a real implementation, you would check if the account exists
      // For now, we'll assume it needs to be created
      return TOKEN_PROGRAM_ID.createAssociatedTokenAccountInstruction(
        payer,
        associatedTokenAddress,
        owner,
        mint
      );
    } catch (error) {
      console.error('Error creating associated token account instruction:', error);
      return null;
    }
  }

  /**
   * Build transaction with multiple instructions
   */
  static async buildTransaction(
    instructions: TransactionInstruction[],
    feePayer: PublicKey,
    recentBlockhash: string
  ): Promise<Transaction> {
    const transaction = new Transaction({
      feePayer,
      recentBlockhash,
    });

    instructions.forEach(instruction => {
      transaction.add(instruction);
    });

    return transaction;
  }

  /**
   * Calculate program derived address
   */
  static async findProgramAddress(
    seeds: (Buffer | Uint8Array)[],
    programId: PublicKey
  ): Promise<[PublicKey, number]> {
    return await PublicKey.findProgramAddress(seeds, programId);
  }

  /**
   * Get multiple accounts efficiently
   */
  static async getMultipleAccounts(
    connection: any,
    publicKeys: PublicKey[]
  ): Promise<any[]> {
    try {
      const accountInfos = await connection.getMultipleAccountsInfo(publicKeys);
      return accountInfos;
    } catch (error) {
      console.error('Error fetching multiple accounts:', error);
      throw error;
    }
  }

  /**
   * Validate instruction parameters
   */
  static validateAmount(amount: BN): boolean {
    return amount.gt(new BN(0)) && amount.lt(new BN('18446744073709551615')); // u64 max
  }

  static validatePublicKey(publicKey: PublicKey | null | undefined): boolean {
    return publicKey !== null && publicKey !== undefined && PublicKey.isOnCurve(publicKey);
  }

  static validateString(str: string, maxLength: number = 100): boolean {
    return typeof str === 'string' && str.length > 0 && str.length <= maxLength;
  }

  /**
   * Calculate mining rewards based on current parameters
   */
  static calculateMiningRewards(
    baseRate: number,
    pioneerBonus: number,
    referralBonus: number,
    securityBonus: number,
    regressionFactor: number,
    xpMultiplier: number,
    rpMultiplier: number,
    stakingBonus: number,
    cardEffects: number[]
  ): BN {
    let totalMultiplier = pioneerBonus * referralBonus * securityBonus * regressionFactor;
    totalMultiplier *= xpMultiplier * rpMultiplier * stakingBonus;
    
    // Apply card effects
    cardEffects.forEach(effect => {
      totalMultiplier *= effect;
    });

    const finalReward = baseRate * totalMultiplier;
    return new BN(Math.floor(finalReward * 1e9)); // Convert to lamports
  }

  /**
   * Calculate XP multiplier based on level
   */
  static calculateXPMultiplier(level: number): number {
    if (level <= 10) return 1.0 + (level * 0.02); // Bronze: 1.0x - 1.2x
    if (level <= 25) return 1.3 + ((level - 10) * 0.033); // Silver: 1.3x - 1.8x
    if (level <= 50) return 1.9 + ((level - 25) * 0.024); // Gold: 1.9x - 2.5x
    if (level <= 75) return 2.6 + ((level - 50) * 0.024); // Platinum: 2.6x - 3.2x
    if (level <= 100) return 3.3 + ((level - 75) * 0.028); // Diamond: 3.3x - 4.0x
    return 4.1 + ((level - 100) * 0.018); // Mythic: 4.1x - 5.0x (capped)
  }

  /**
   * Calculate RP tier multiplier
   */
  static calculateRPMultiplier(rpPoints: number): number {
    if (rpPoints < 1000) return 1.0; // Explorer
    if (rpPoints < 5000) return 1.2; // Connector
    if (rpPoints < 15000) return 1.5; // Influencer
    if (rpPoints < 50000) return 2.0; // Leader
    return 3.0; // Ambassador
  }

  /**
   * Calculate staking bonus based on amount and duration
   */
  static calculateStakingBonus(stakedAmount: BN, stakingTier: number): number {
    const baseBonus = 1.0;
    const tierMultipliers = [1.0, 1.2, 1.35, 1.5, 1.75, 2.0]; // Based on staking tiers
    
    if (stakingTier >= 0 && stakingTier < tierMultipliers.length) {
      return baseBonus * tierMultipliers[stakingTier];
    }
    
    return baseBonus;
  }

  /**
   * Get current mining phase based on total users
   */
  static getCurrentMiningPhase(totalUsers: number): {
    phase: number;
    baseRate: number;
    pioneerBonus: number;
    maxDaily: number;
  } {
    if (totalUsers < 100000) {
      return { phase: 1, baseRate: 0.1, pioneerBonus: 2.0, maxDaily: 4.8 };
    } else if (totalUsers < 1000000) {
      return { phase: 2, baseRate: 0.05, pioneerBonus: 1.5, maxDaily: 1.8 };
    } else if (totalUsers < 10000000) {
      return { phase: 3, baseRate: 0.025, pioneerBonus: 1.2, maxDaily: 0.72 };
    } else {
      return { phase: 4, baseRate: 0.01, pioneerBonus: 1.0, maxDaily: 0.24 };
    }
  }

  /**
   * Calculate quality score from engagement metrics
   */
  static calculateQualityScore(
    likes: number,
    comments: number,
    shares: number,
    views: number,
    contentLength: number
  ): number {
    // Engagement rate calculation
    const totalEngagement = likes + comments + shares;
    const engagementRate = views > 0 ? totalEngagement / views : 0;
    
    // Content quality factors
    let qualityScore = 1.0;
    
    // Engagement rate bonus (0.8x - 1.5x)
    if (engagementRate > 0.1) qualityScore *= 1.5;
    else if (engagementRate > 0.05) qualityScore *= 1.3;
    else if (engagementRate > 0.02) qualityScore *= 1.1;
    else if (engagementRate < 0.01) qualityScore *= 0.8;
    
    // Content length bonus (encourage meaningful content)
    if (contentLength > 500) qualityScore *= 1.2;
    else if (contentLength > 100) qualityScore *= 1.1;
    else if (contentLength < 50) qualityScore *= 0.9;
    
    // Cap the quality score
    return Math.min(Math.max(qualityScore, 0.5), 2.0);
  }

  /**
   * Calculate network regression factor to prevent whale dominance
   */
  static calculateRegressionFactor(totalHoldings: BN): number {
    const holdings = totalHoldings.toNumber();
    const factor = Math.exp(-0.001 * holdings);
    return Math.max(factor, 0.1); // Minimum 10% of base rate
  }

  /**
   * Format amount for display
   */
  static formatAmount(amount: BN, decimals: number = 9): string {
    const divisor = new BN(10).pow(new BN(decimals));
    const wholePart = amount.div(divisor);
    const fractionalPart = amount.mod(divisor);
    
    const fractionalStr = fractionalPart.toString().padStart(decimals, '0');
    const trimmedFractional = fractionalStr.replace(/0+$/, '');
    
    if (trimmedFractional.length === 0) {
      return wholePart.toString();
    }
    
    return `${wholePart.toString()}.${trimmedFractional}`;
  }

  /**
   * Parse amount from string
   */
  static parseAmount(amount: string, decimals: number = 9): BN {
    const [wholePart, fractionalPart = ''] = amount.split('.');
    const paddedFractional = fractionalPart.padEnd(decimals, '0').slice(0, decimals);
    const wholePartBN = new BN(wholePart || '0');
    const fractionalPartBN = new BN(paddedFractional || '0');
    const multiplier = new BN(10).pow(new BN(decimals));
    
    return wholePartBN.mul(multiplier).add(fractionalPartBN);
  }

  /**
   * Generate referral code
   */
  static generateReferralCode(publicKey: PublicKey): string {
    const hash = publicKey.toString().slice(-8).toUpperCase();
    return `FIN${hash}`;
  }

  /**
   * Validate referral code format
   */
  static validateReferralCode(code: string): boolean {
    const pattern = /^FIN[A-Z0-9]{8}$/;
    return pattern.test(code);
  }

  /**
   * Calculate guild bonus based on participation
   */
  static calculateGuildBonus(
    guildLevel: number,
    memberContribution: number,
    guildSize: number
  ): number {
    const levelBonus = 1.0 + (guildLevel * 0.05); // 5% per guild level
    const contributionBonus = 1.0 + Math.min(memberContribution / 1000, 0.5); // Up to 50% for high contribution
    const sizeBonus = 1.0 + Math.min(guildSize / 100, 0.3); // Up to 30% for large guilds
    
    return levelBonus * contributionBonus * sizeBonus;
  }

  /**
   * Calculate time-based bonuses (streaks, etc.)
   */
  static calculateTimeBonus(streakDays: number, lastActivity: Date): number {
    const now = new Date();
    const hoursSinceActivity = (now.getTime() - lastActivity.getTime()) / (1000 * 60 * 60);
    
    // Streak bonus (up to 3x for 30+ days)
    const streakBonus = Math.min(1.0 + (streakDays * 0.067), 3.0);
    
    // Activity freshness (penalty for inactive users)
    let freshnessMultiplier = 1.0;
    if (hoursSinceActivity > 48) freshnessMultiplier = 0.5;
    else if (hoursSinceActivity > 24) freshnessMultiplier = 0.8;
    
    return streakBonus * freshnessMultiplier;
  }

  /**
   * Estimate gas fees for instruction
   */
  static estimateGasFee(instructionType: string): number {
    const baseFees: { [key: string]: number } = {
      'initialize_user': 0.01,
      'claim_rewards': 0.005,
      'update_xp': 0.003,
      'stake_tokens': 0.005,
      'use_special_card': 0.004,
      'create_guild': 0.008,
      'mint_nft': 0.006,
      'swap_tokens': 0.007,
    };
    
    return baseFees[instructionType] || 0.005; // Default 0.005 SOL
  }

  /**
   * Check if instruction requires KYC verification
   */
  static requiresKYC(instructionType: string): boolean {
    const kycRequired = [
      'claim_rewards',
      'stake_tokens',
      'create_guild',
      'create_proposal',
      'bridge_tokens'
    ];
    
    return kycRequired.includes(instructionType);
  }

  /**
   * Get instruction cooldown period
   */
  static getInstructionCooldown(instructionType: string): number {
    const cooldowns: { [key: string]: number } = {
      'claim_rewards': 3600, // 1 hour
      'update_xp': 60, // 1 minute
      'use_special_card': 300, // 5 minutes
      'create_proposal': 86400, // 24 hours
    };
    
    return cooldowns[instructionType] || 0;
  }
}

/**
 * Complex transaction builders for common user flows
 */
export class TransactionBuilders {

  /**
   * Complete user onboarding transaction
   * Initializes user and creates associated token account
   */
  static async buildOnboardingTransaction(
    coreProgram: anchor.Program,
    tokenProgram: anchor.Program,
    user: PublicKey,
    referrer?: PublicKey
  ): Promise<Transaction> {
    const instructions: TransactionInstruction[] = [];

    // Create associated token account for FIN tokens
    const createATAInstruction = await InstructionUtils.createAssociatedTokenAccountIfNeeded(
      FIN_TOKEN_MINT,
      user,
      user
    );
    if (createATAInstruction) {
      instructions.push(createATAInstruction);
    }

    // Initialize user with all state accounts
    const initUserInstruction = await FinovaCoreInstructions.initializeUser(
      coreProgram,
      user,
      referrer
    );
    instructions.push(initUserInstruction);

    // Get recent blockhash (this would be done by the client)
    const recentBlockhash = 'PLACEHOLDER_BLOCKHASH';

    return InstructionUtils.buildTransaction(instructions, user, recentBlockhash);
  }

  /**
   * Daily mining claim with XP update
   */
  static async buildDailyMiningTransaction(
    coreProgram: anchor.Program,
    tokenProgram: anchor.Program,
    user: PublicKey,
    xpParams: XPUpdateParams
  ): Promise<Transaction> {
    const instructions: TransactionInstruction[] = [];

    // Update XP first
    const updateXPInstruction = await FinovaCoreInstructions.updateXP(
      coreProgram,
      user,
      xpParams
    );
    instructions.push(updateXPInstruction);

    // Claim mining rewards
    const claimRewardsInstruction = await FinovaCoreInstructions.claimRewards(
      coreProgram,
      tokenProgram,
      user
    );
    instructions.push(claimRewardsInstruction);

    const recentBlockhash = 'PLACEHOLDER_BLOCKHASH';
    return InstructionUtils.buildTransaction(instructions, user, recentBlockhash);
  }

  /**
   * NFT card usage with immediate rewards claim
   */
  static async buildCardUsageTransaction(
    coreProgram: anchor.Program,
    tokenProgram: anchor.Program,
    nftProgram: anchor.Program,
    user: PublicKey,
    cardType: number
  ): Promise<Transaction> {
    const instructions: TransactionInstruction[] = [];

    // Use special card (applies effects via CPI)
    const useCardInstruction = await FinovaNFTInstructions.useSpecialCard(
      nftProgram,
      coreProgram,
      user,
      cardType
    );
    instructions.push(useCardInstruction);

    // Claim enhanced rewards immediately
    const claimRewardsInstruction = await FinovaCoreInstructions.claimRewards(
      coreProgram,
      tokenProgram,
      user
    );
    instructions.push(claimRewardsInstruction);

    const recentBlockhash = 'PLACEHOLDER_BLOCKHASH';
    return InstructionUtils.buildTransaction(instructions, user, recentBlockhash);
  }

  /**
   * Complete guild participation flow
   */
  static async buildGuildParticipationTransaction(
    coreProgram: anchor.Program,
    creator: PublicKey,
    guildParams: GuildParams,
    participants: PublicKey[]
  ): Promise<Transaction> {
    const instructions: TransactionInstruction[] = [];

    // Create guild
    const createGuildInstruction = await FinovaCoreInstructions.createGuild(
      coreProgram,
      creator,
      guildParams
    );
    instructions.push(createGuildInstruction);

    // Add participants
    for (const participant of participants.slice(0, 5)) { // Limit to 5 per transaction
      const joinGuildInstruction = await FinovaCoreInstructions.joinGuild(
        coreProgram,
        participant,
        guildParams.name
      );
      instructions.push(joinGuildInstruction);
    }

    const recentBlockhash = 'PLACEHOLDER_BLOCKHASH';
    return InstructionUtils.buildTransaction(instructions, creator, recentBlockhash);
  }

  /**
   * DeFi liquidity provision with staking
   */
  static async buildLiquidityProvisionTransaction(
    coreProgram: anchor.Program,
    defiProgram: anchor.Program,
    provider: PublicKey,
    tokenAMint: PublicKey,
    tokenBMint: PublicKey,
    amountA: BN,
    amountB: BN,
    stakingAmount: BN
  ): Promise<Transaction> {
    const instructions: TransactionInstruction[] = [];

    // Add liquidity to DeFi pool
    const addLiquidityInstruction = await FinovaDeFiInstructions.addLiquidity(
      defiProgram,
      provider,
      tokenAMint,
      tokenBMint,
      amountA,
      amountB
    );
    instructions.push(addLiquidityInstruction);

    // Stake tokens for additional rewards
    const stakeInstruction = await FinovaCoreInstructions.stakeTokens(
      coreProgram,
      provider,
      stakingAmount,
      30 // 30 days
    );
    instructions.push(stakeInstruction);

    const recentBlockhash = 'PLACEHOLDER_BLOCKHASH';
    return InstructionUtils.buildTransaction(instructions, provider, recentBlockhash);
  }
}

// Export all instruction classes and utilities
export {
  InstructionUtils,
  TransactionBuilders
};

// Export default object with all instruction builders
export default {
  Core: FinovaCoreInstructions,
  Token: FinovaTokenInstructions,
  NFT: FinovaNFTInstructions,
  DeFi: FinovaDeFiInstructions,
  Oracle: FinovaOracleInstructions,
  Bridge: FinovaBridgeInstructions,
  Utils: InstructionUtils,
  Builders: TransactionBuilders
};
