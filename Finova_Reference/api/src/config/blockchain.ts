import { Connection, PublicKey, Keypair, LAMPORTS_PER_SOL } from '@solana/web3.js';
import { Program, AnchorProvider, Wallet, BN } from '@project-serum/anchor';
import { TOKEN_PROGRAM_ID, ASSOCIATED_TOKEN_PROGRAM_ID } from '@solana/spl-token';
import NodeWallet from '@project-serum/anchor/dist/cjs/nodewallet';
import * as anchor from '@project-serum/anchor';
import bs58 from 'bs58';

// Environment configuration
interface BlockchainConfig {
  network: 'devnet' | 'testnet' | 'mainnet-beta';
  rpcEndpoint: string;
  wsEndpoint: string;
  commitment: 'processed' | 'confirmed' | 'finalized';
  skipPreflight: boolean;
  maxRetries: number;
  confirmTransactionInitialTimeout: number;
}

interface ProgramAddresses {
  finovaCore: string;
  finovaToken: string;
  finova_nft: string;
  finovaDefi: string;
  finovaBridge: string;
  finovaOracle: string;
}

interface TokenMints {
  FIN: string;      // Primary utility token
  sFIN: string;     // Staked FIN
  USDfin: string;   // Synthetic stablecoin
  sUSDfin: string;  // Staked USDfin
}

class FinovaBlockchainConfig {
  private static instance: FinovaBlockchainConfig;
  private connection: Connection;
  private provider: AnchorProvider;
  private config: BlockchainConfig;
  private programAddresses: ProgramAddresses;
  private tokenMints: TokenMints;
  private programs: Map<string, Program> = new Map();

  private constructor() {
    this.loadConfiguration();
    this.initializeConnection();
    this.initializeProvider();
  }

  public static getInstance(): FinovaBlockchainConfig {
    if (!FinovaBlockchainConfig.instance) {
      FinovaBlockchainConfig.instance = new FinovaBlockchainConfig();
    }
    return FinovaBlockchainConfig.instance;
  }

  private loadConfiguration(): void {
    const env = process.env.NODE_ENV || 'development';
    
    // Network configuration based on environment
    const networkConfigs = {
      development: {
        network: 'devnet' as const,
        rpcEndpoint: process.env.SOLANA_DEVNET_RPC || 'https://api.devnet.solana.com',
        wsEndpoint: process.env.SOLANA_DEVNET_WS || 'wss://api.devnet.solana.com',
      },
      staging: {
        network: 'testnet' as const,
        rpcEndpoint: process.env.SOLANA_TESTNET_RPC || 'https://api.testnet.solana.com',
        wsEndpoint: process.env.SOLANA_TESTNET_WS || 'wss://api.testnet.solana.com',
      },
      production: {
        network: 'mainnet-beta' as const,
        rpcEndpoint: process.env.SOLANA_MAINNET_RPC || 'https://api.mainnet-beta.solana.com',
        wsEndpoint: process.env.SOLANA_MAINNET_WS || 'wss://api.mainnet-beta.solana.com',
      }
    };

    this.config = {
      ...networkConfigs[env] || networkConfigs.development,
      commitment: (process.env.SOLANA_COMMITMENT as any) || 'confirmed',
      skipPreflight: process.env.SOLANA_SKIP_PREFLIGHT === 'true',
      maxRetries: parseInt(process.env.SOLANA_MAX_RETRIES || '3'),
      confirmTransactionInitialTimeout: parseInt(process.env.SOLANA_CONFIRM_TIMEOUT || '60000'),
    };

    // Program addresses configuration
    this.programAddresses = {
      finovaCore: process.env.FINOVA_CORE_PROGRAM_ID || 'FinovaCoreProgram1111111111111111111111111',
      finovaToken: process.env.FINOVA_TOKEN_PROGRAM_ID || 'FinovaTokenProgram111111111111111111111111',
      finova_nft: process.env.FINOVA_NFT_PROGRAM_ID || 'FinovaNFTProgram1111111111111111111111111',
      finovaDefi: process.env.FINOVA_DEFI_PROGRAM_ID || 'FinovaDefiProgram111111111111111111111111',
      finovaBridge: process.env.FINOVA_BRIDGE_PROGRAM_ID || 'FinovaBridgeProgram11111111111111111111111',
      finovaOracle: process.env.FINOVA_ORACLE_PROGRAM_ID || 'FinovaOracleProgram11111111111111111111111',
    };

    // Token mint addresses
    this.tokenMints = {
      FIN: process.env.FIN_TOKEN_MINT || 'FinTokenMint111111111111111111111111111111',
      sFIN: process.env.SFIN_TOKEN_MINT || 'sFinTokenMint11111111111111111111111111111',
      USDfin: process.env.USDFIN_TOKEN_MINT || 'USDfinTokenMint1111111111111111111111111111',
      sUSDfin: process.env.SUSDFIN_TOKEN_MINT || 'sUSDfinTokenMint111111111111111111111111111',
    };
  }

  private initializeConnection(): void {
    this.connection = new Connection(
      this.config.rpcEndpoint,
      {
        commitment: this.config.commitment,
        wsEndpoint: this.config.wsEndpoint,
        confirmTransactionInitialTimeout: this.config.confirmTransactionInitialTimeout,
      }
    );

    // Connection health monitoring
    this.monitorConnection();
  }

  private initializeProvider(): void {
    let wallet: Wallet;

    if (process.env.SOLANA_PRIVATE_KEY) {
      const privateKeyBytes = bs58.decode(process.env.SOLANA_PRIVATE_KEY);
      const keypair = Keypair.fromSecretKey(privateKeyBytes);
      wallet = new NodeWallet(keypair);
    } else {
      // Generate ephemeral wallet for testing
      const keypair = Keypair.generate();
      wallet = new NodeWallet(keypair);
      console.warn('Using generated keypair. Set SOLANA_PRIVATE_KEY for production.');
    }

    this.provider = new AnchorProvider(
      this.connection,
      wallet,
      {
        commitment: this.config.commitment,
        skipPreflight: this.config.skipPreflight,
        maxRetries: this.config.maxRetries,
      }
    );

    anchor.setProvider(this.provider);
  }

  // Connection monitoring and health checks
  private async monitorConnection(): Promise<void> {
    try {
      const version = await this.connection.getVersion();
      console.log(`Connected to Solana ${this.config.network}:`, version);
      
      // Monitor connection every 30 seconds
      setInterval(async () => {
        try {
          await this.connection.getSlot();
        } catch (error) {
          console.error('Solana connection health check failed:', error);
          this.reconnect();
        }
      }, 30000);
    } catch (error) {
      console.error('Initial connection failed:', error);
      throw new Error(`Failed to connect to Solana ${this.config.network}`);
    }
  }

  private async reconnect(): Promise<void> {
    console.log('Attempting to reconnect to Solana...');
    this.initializeConnection();
  }

  // Program management
  async loadProgram(programName: keyof ProgramAddresses): Promise<Program> {
    if (this.programs.has(programName)) {
      return this.programs.get(programName)!;
    }

    try {
      const programId = new PublicKey(this.programAddresses[programName]);
      const idl = await Program.fetchIdl(programId, this.provider);
      
      if (!idl) {
        throw new Error(`IDL not found for program: ${programName}`);
      }

      const program = new Program(idl, programId, this.provider);
      this.programs.set(programName, program);
      
      console.log(`Loaded program: ${programName} at ${programId.toString()}`);
      return program;
    } catch (error) {
      console.error(`Failed to load program ${programName}:`, error);
      throw error;
    }
  }

  // Mining calculations integration
  async calculateMiningRewards(userPublicKey: PublicKey): Promise<{
    baseRate: number;
    pioneerBonus: number;
    referralBonus: number;
    securityBonus: number;
    regressionFactor: number;
    totalRate: number;
  }> {
    try {
      const coreProgram = await this.loadProgram('finovaCore');
      const [userStatePda] = PublicKey.findProgramAddressSync(
        [Buffer.from('user'), userPublicKey.toBuffer()],
        coreProgram.programId
      );

      const userState = await coreProgram.account.userState.fetch(userStatePda);
      const networkStats = await coreProgram.account.networkState.fetch(
        this.getNetworkStatePda()
      );

      // Calculate mining components based on whitepaper formulas
      const baseRate = this.getCurrentPhaseRate(networkStats.totalUsers);
      const pioneerBonus = Math.max(1.0, 2.0 - (networkStats.totalUsers / 1000000));
      const referralBonus = 1 + (userState.activeReferrals * 0.1);
      const securityBonus = userState.isKycVerified ? 1.2 : 0.8;
      const regressionFactor = Math.exp(-0.001 * userState.totalHoldings.toNumber());

      const totalRate = baseRate * pioneerBonus * referralBonus * securityBonus * regressionFactor;

      return {
        baseRate,
        pioneerBonus,
        referralBonus,
        securityBonus,
        regressionFactor,
        totalRate,
      };
    } catch (error) {
      console.error('Error calculating mining rewards:', error);
      throw error;
    }
  }

  // XP system integration
  async calculateXpMultiplier(
    activityType: string,
    platform: string,
    userLevel: number,
    contentQuality: number,
    streakDays: number
  ): Promise<number> {
    const baseXp = this.getBaseXp(activityType);
    const platformMultiplier = this.getPlatformMultiplier(platform);
    const qualityScore = Math.max(0.5, Math.min(2.0, contentQuality));
    const streakBonus = Math.min(3.0, 1.0 + (streakDays * 0.1));
    const levelProgression = Math.exp(-0.01 * userLevel);

    return baseXp * platformMultiplier * qualityScore * streakBonus * levelProgression;
  }

  // RP (Referral Points) calculations
  async calculateRpValue(userPublicKey: PublicKey): Promise<{
    directRp: number;
    networkRp: number;
    qualityBonus: number;
    totalRp: number;
  }> {
    try {
      const coreProgram = await this.loadProgram('finovaCore');
      const [referralStatePda] = PublicKey.findProgramAddressSync(
        [Buffer.from('referral'), userPublicKey.toBuffer()],
        coreProgram.programId
      );

      const referralState = await coreProgram.account.referralState.fetch(referralStatePda);
      
      const directRp = referralState.directReferrals.reduce((sum: number, ref: any) => 
        sum + (ref.activity * ref.level * this.getTimeDecay(ref.lastActive)), 0
      );
      
      const networkRp = 
        referralState.level2Network * 0.3 + 
        referralState.level3Network * 0.1;
      
      const qualityBonus = 
        referralState.networkDiversity * 
        referralState.avgReferralLevel * 
        referralState.retentionRate;

      const totalRp = (directRp + networkRp) * qualityBonus;

      return { directRp, networkRp, qualityBonus, totalRp };
    } catch (error) {
      console.error('Error calculating RP value:', error);
      throw error;
    }
  }

  // Token operations
  async getTokenBalance(walletAddress: PublicKey, tokenType: keyof TokenMints): Promise<number> {
    try {
      const mintAddress = new PublicKey(this.tokenMints[tokenType]);
      const tokenAccount = await this.connection.getTokenAccountsByOwner(walletAddress, {
        mint: mintAddress,
      });

      if (tokenAccount.value.length === 0) {
        return 0;
      }

      const balance = await this.connection.getTokenAccountBalance(
        tokenAccount.value[0].pubkey
      );
      
      return parseFloat(balance.value.uiAmountString || '0');
    } catch (error) {
      console.error(`Error getting ${tokenType} balance:`, error);
      return 0;
    }
  }

  async mintTokens(
    recipient: PublicKey,
    tokenType: keyof TokenMints,
    amount: number
  ): Promise<string> {
    try {
      const tokenProgram = await this.loadProgram('finovaToken');
      const mintAddress = new PublicKey(this.tokenMints[tokenType]);
      
      const transaction = await tokenProgram.methods
        .mintTokens(new BN(amount * LAMPORTS_PER_SOL))
        .accounts({
          mint: mintAddress,
          recipient,
          authority: this.provider.wallet.publicKey,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .rpc();

      console.log(`Minted ${amount} ${tokenType} to ${recipient.toString()}`);
      return transaction;
    } catch (error) {
      console.error(`Error minting ${tokenType}:`, error);
      throw error;
    }
  }

  // NFT operations
  async mintNft(
    recipient: PublicKey,
    cardType: string,
    rarity: string,
    metadata: any
  ): Promise<string> {
    try {
      const nftProgram = await this.loadProgram('finova_nft');
      
      const transaction = await nftProgram.methods
        .mintNft(cardType, rarity, metadata)
        .accounts({
          recipient,
          authority: this.provider.wallet.publicKey,
        })
        .rpc();

      console.log(`Minted NFT ${cardType} to ${recipient.toString()}`);
      return transaction;
    } catch (error) {
      console.error('Error minting NFT:', error);
      throw error;
    }
  }

  // Staking operations
  async stakeTokens(amount: number, stakingPeriod: number): Promise<string> {
    try {
      const tokenProgram = await this.loadProgram('finovaToken');
      
      const transaction = await tokenProgram.methods
        .stakeTokens(new BN(amount * LAMPORTS_PER_SOL), new BN(stakingPeriod))
        .accounts({
          staker: this.provider.wallet.publicKey,
          finMint: new PublicKey(this.tokenMints.FIN),
          sfinMint: new PublicKey(this.tokenMints.sFIN),
        })
        .rpc();

      console.log(`Staked ${amount} FIN for ${stakingPeriod} days`);
      return transaction;
    } catch (error) {
      console.error('Error staking tokens:', error);
      throw error;
    }
  }

  // Bridge operations for cross-chain
  async initializeBridge(targetChain: string): Promise<string> {
    try {
      const bridgeProgram = await this.loadProgram('finovaBridge');
      
      const transaction = await bridgeProgram.methods
        .initializeBridge(targetChain)
        .accounts({
          authority: this.provider.wallet.publicKey,
        })
        .rpc();

      console.log(`Initialized bridge to ${targetChain}`);
      return transaction;
    } catch (error) {
      console.error('Error initializing bridge:', error);
      throw error;
    }
  }

  // Oracle price feeds
  async getPriceFeeds(): Promise<Record<string, number>> {
    try {
      const oracleProgram = await this.loadProgram('finovaOracle');
      const priceFeeds = await oracleProgram.account.priceFeed.all();
      
      const prices: Record<string, number> = {};
      priceFeeds.forEach((feed) => {
        prices[feed.account.symbol] = feed.account.price.toNumber() / 1e8; // Assuming 8 decimals
      });

      return prices;
    } catch (error) {
      console.error('Error fetching price feeds:', error);
      return {};
    }
  }

  // Utility methods
  private getCurrentPhaseRate(totalUsers: number): number {
    if (totalUsers < 100000) return 0.1; // Phase 1: Pioneer
    if (totalUsers < 1000000) return 0.05; // Phase 2: Growth  
    if (totalUsers < 10000000) return 0.025; // Phase 3: Maturity
    return 0.01; // Phase 4: Stability
  }

  private getBaseXp(activityType: string): number {
    const baseXpMap: Record<string, number> = {
      'text_post': 50,
      'photo_post': 75,
      'video_post': 150,
      'story': 25,
      'comment': 25,
      'like': 5,
      'share': 15,
      'follow': 20,
      'daily_login': 10,
      'daily_quest': 100,
      'viral_content': 1000,
    };
    return baseXpMap[activityType] || 10;
  }

  private getPlatformMultiplier(platform: string): number {
    const multipliers: Record<string, number> = {
      'tiktok': 1.3,
      'instagram': 1.2,
      'youtube': 1.4,
      'facebook': 1.1,
      'twitter': 1.2,
      'x': 1.2,
      'default': 1.0,
    };
    return multipliers[platform.toLowerCase()] || multipliers.default;
  }

  private getTimeDecay(lastActive: number): number {
    const daysSinceActive = (Date.now() - lastActive) / (1000 * 60 * 60 * 24);
    return Math.exp(-daysSinceActive * 0.01); // 1% decay per day
  }

  private getNetworkStatePda(): PublicKey {
    const [pda] = PublicKey.findProgramAddressSync(
      [Buffer.from('network_state')],
      new PublicKey(this.programAddresses.finovaCore)
    );
    return pda;
  }

  // Getters
  public getConnection(): Connection {
    return this.connection;
  }

  public getProvider(): AnchorProvider {
    return this.provider;
  }

  public getConfig(): BlockchainConfig {
    return this.config;
  }

  public getProgramAddresses(): ProgramAddresses {
    return this.programAddresses;
  }

  public getTokenMints(): TokenMints {
    return this.tokenMints;
  }

  public async healthCheck(): Promise<{
    connected: boolean;
    network: string;
    slot: number;
    blockTime: number | null;
    version: any;
  }> {
    try {
      const slot = await this.connection.getSlot();
      const blockTime = await this.connection.getBlockTime(slot);
      const version = await this.connection.getVersion();

      return {
        connected: true,
        network: this.config.network,
        slot,
        blockTime,
        version,
      };
    } catch (error) {
      return {
        connected: false,
        network: this.config.network,
        slot: 0,
        blockTime: null,
        version: null,
      };
    }
  }
}

// Export singleton instance
export const blockchainConfig = FinovaBlockchainConfig.getInstance();
export default blockchainConfig;

// Type exports
export type {
  BlockchainConfig,
  ProgramAddresses,
  TokenMints,
};

export {
  FinovaBlockchainConfig,
};
