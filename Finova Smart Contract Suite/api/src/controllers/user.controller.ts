import { Request, Response } from 'express';
import { UserService } from '../services/user.service';
import { MiningService } from '../services/mining.service';
import { XPService } from '../services/xp.service';
import { ReferralService } from '../services/referral.service';
import { AntiBotService } from '../services/anti-bot.service';
import { AIQualityService } from '../services/ai-quality.service';
import { BlockchainService } from '../services/blockchain.service';
import { NotificationService } from '../services/notification.service';
import { validateRequest, handleError } from '../utils/validation';
import { logger } from '../utils/logger';
import { calculateMiningRate, calculateXPMultiplier, calculateRPValue } from '../utils/calculations';

interface AuthenticatedRequest extends Request {
  user: {
    id: string;
    publicKey: string;
    kycVerified: boolean;
    humanScore: number;
  };
}

export class UserController {
  private userService: UserService;
  private miningService: MiningService;
  private xpService: XPService;
  private referralService: ReferralService;
  private antiBotService: AntiBotService;
  private aiQualityService: AIQualityService;
  private blockchainService: BlockchainService;
  private notificationService: NotificationService;

  constructor() {
    this.userService = new UserService();
    this.miningService = new MiningService();
    this.xpService = new XPService();
    this.referralService = new ReferralService();
    this.antiBotService = new AntiBotService();
    this.aiQualityService = new AIQualityService();
    this.blockchainService = new BlockchainService();
    this.notificationService = new NotificationService();
  }

  /**
   * Get user profile with integrated XP, RP, and Mining data
   */
  public getProfile = async (req: AuthenticatedRequest, res: Response): Promise<void> => {
    try {
      const { id: userId } = req.user;
      
      // Parallel fetch of user data
      const [
        userProfile,
        miningStats,
        xpData,
        referralData,
        stakingInfo,
        nftCollection
      ] = await Promise.all([
        this.userService.getUserById(userId),
        this.miningService.getUserMiningStats(userId),
        this.xpService.getUserXPData(userId),
        this.referralService.getUserReferralData(userId),
        this.userService.getStakingInfo(userId),
        this.userService.getUserNFTs(userId)
      ]);

      if (!userProfile) {
        res.status(404).json({ error: 'User not found' });
        return;
      }

      // Calculate current mining rate with all multipliers
      const currentMiningRate = calculateMiningRate({
        baseRate: miningStats.baseRate,
        pioneerBonus: miningStats.pioneerBonus,
        referralBonus: referralData.bonus,
        securityBonus: userProfile.kycVerified ? 1.2 : 0.8,
        totalHoldings: userProfile.totalFinBalance,
        xpLevel: xpData.currentLevel,
        rpTier: referralData.tier,
        stakingMultiplier: stakingInfo.multiplier || 1.0
      });

      // Calculate XP and RP values
      const xpMultiplier = calculateXPMultiplier(xpData.currentLevel);
      const rpValue = calculateRPValue(referralData);

      const response = {
        user: {
          id: userProfile.id,
          username: userProfile.username,
          email: userProfile.email,
          publicKey: userProfile.publicKey,
          kycVerified: userProfile.kycVerified,
          humanScore: userProfile.humanScore,
          createdAt: userProfile.createdAt,
          lastActive: userProfile.lastActive,
          avatar: userProfile.avatar,
          badges: userProfile.badges
        },
        mining: {
          totalMined: miningStats.totalMined,
          currentRate: currentMiningRate,
          lastClaim: miningStats.lastClaim,
          pendingRewards: miningStats.pendingRewards,
          phase: miningStats.currentPhase,
          dailyCap: miningStats.dailyCap,
          canMine: miningStats.isActive && userProfile.humanScore > 0.7
        },
        xp: {
          currentXP: xpData.currentXP,
          level: xpData.currentLevel,
          nextLevelXP: xpData.nextLevelXP,
          progressPercent: xpData.progressPercent,
          multiplier: xpMultiplier,
          streak: xpData.dailyStreak,
          badges: xpData.badges,
          recentActivities: xpData.recentActivities
        },
        referral: {
          code: referralData.code,
          tier: referralData.tier,
          totalRP: rpValue,
          directReferrals: referralData.directCount,
          networkSize: referralData.networkSize,
          activeReferrals: referralData.activeCount,
          networkQuality: referralData.qualityScore,
          earnings: referralData.totalEarnings
        },
        staking: {
          stakedAmount: stakingInfo.stakedAmount,
          sFINBalance: stakingInfo.sFINBalance,
          apy: stakingInfo.currentAPY,
          pendingRewards: stakingInfo.pendingRewards,
          tier: stakingInfo.tier,
          multiplier: stakingInfo.multiplier
        },
        nfts: {
          total: nftCollection.length,
          activeCards: nftCollection.filter(nft => nft.active),
          collections: nftCollection.reduce((acc, nft) => {
            acc[nft.collection] = (acc[nft.collection] || 0) + 1;
            return acc;
          }, {} as Record<string, number>)
        },
        balances: {
          FIN: userProfile.totalFinBalance,
          sFIN: stakingInfo.sFINBalance,
          USDfin: userProfile.usdFinBalance,
          sUSDfin: userProfile.sUsdFinBalance
        }
      };

      res.json(response);
      logger.info(`Profile fetched for user ${userId}`);

    } catch (error) {
      handleError(error, res, 'Failed to fetch user profile');
    }
  };

  /**
   * Update user profile with validation
   */
  public updateProfile = async (req: AuthenticatedRequest, res: Response): Promise<void> => {
    try {
      const { id: userId } = req.user;
      const updates = validateRequest(req.body, [
        'username', 'avatar', 'bio', 'socialLinks'
      ]);

      // Anti-bot validation
      const humanScore = await this.antiBotService.validateUserActivity(userId, req.body);
      if (humanScore < 0.7) {
        res.status(403).json({ 
          error: 'Profile update blocked',
          reason: 'Suspicious activity detected'
        });
        return;
      }

      const updatedUser = await this.userService.updateUser(userId, {
        ...updates,
        humanScore,
        lastActive: new Date()
      });

      // Award XP for profile update
      await this.xpService.awardXP(userId, 'profile_update', 25);

      res.json({ 
        message: 'Profile updated successfully',
        user: updatedUser
      });

      logger.info(`Profile updated for user ${userId}`);

    } catch (error) {
      handleError(error, res, 'Failed to update profile');
    }
  };

  /**
   * Start mining session with comprehensive validation
   */
  public startMining = async (req: AuthenticatedRequest, res: Response): Promise<void> => {
    try {
      const { id: userId, kycVerified, humanScore } = req.user;

      // Validation checks
      if (!kycVerified) {
        res.status(403).json({ error: 'KYC verification required to mine' });
        return;
      }

      if (humanScore < 0.7) {
        res.status(403).json({ error: 'Mining blocked due to suspicious activity' });
        return;
      }

      // Check if user can start mining
      const canMine = await this.miningService.canStartMining(userId);
      if (!canMine.allowed) {
        res.status(400).json({ 
          error: 'Cannot start mining',
          reason: canMine.reason,
          nextAvailable: canMine.nextAvailable
        });
        return;
      }

      // Get current rates and multipliers
      const [xpData, referralData, stakingInfo] = await Promise.all([
        this.xpService.getUserXPData(userId),
        this.referralService.getUserReferralData(userId),
        this.userService.getStakingInfo(userId)
      ]);

      // Start mining session
      const miningSession = await this.miningService.startMining(userId, {
        xpLevel: xpData.currentLevel,
        rpTier: referralData.tier,
        stakingMultiplier: stakingInfo.multiplier || 1.0,
        humanScore
      });

      // Award XP for starting mining
      await this.xpService.awardXP(userId, 'start_mining', 10);

      res.json({
        message: 'Mining started successfully',
        session: {
          sessionId: miningSession.id,
          startTime: miningSession.startTime,
          miningRate: miningSession.currentRate,
          estimatedDaily: miningSession.estimatedDaily,
          phase: miningSession.phase,
          multipliers: miningSession.multipliers
        }
      });

      logger.info(`Mining started for user ${userId} at rate ${miningSession.currentRate}`);

    } catch (error) {
      handleError(error, res, 'Failed to start mining');
    }
  };

  /**
   * Claim mining rewards with security validation
   */
  public claimMining = async (req: AuthenticatedRequest, res: Response): Promise<void> => {
    try {
      const { id: userId, publicKey } = req.user;

      // Validate claim eligibility
      const pendingRewards = await this.miningService.getPendingRewards(userId);
      if (pendingRewards <= 0) {
        res.status(400).json({ error: 'No rewards to claim' });
        return;
      }

      // Anti-bot validation
      const isValidClaim = await this.antiBotService.validateClaim(userId);
      if (!isValidClaim) {
        res.status(403).json({ error: 'Claim blocked due to suspicious activity' });
        return;
      }

      // Execute blockchain transaction
      const transaction = await this.blockchainService.mintTokens(publicKey, pendingRewards);
      
      // Update mining records
      const claimResult = await this.miningService.processClaim(userId, pendingRewards, transaction.signature);

      // Award XP for claiming
      await this.xpService.awardXP(userId, 'claim_mining', 20);

      // Update referral rewards
      await this.referralService.distributeReferralRewards(userId, pendingRewards);

      res.json({
        message: 'Rewards claimed successfully',
        claimed: pendingRewards,
        transactionSignature: transaction.signature,
        newBalance: claimResult.newBalance,
        totalMined: claimResult.totalMined
      });

      // Send notification
      await this.notificationService.sendRewardNotification(userId, pendingRewards);

      logger.info(`Mining rewards claimed: ${pendingRewards} FIN for user ${userId}`);

    } catch (error) {
      handleError(error, res, 'Failed to claim mining rewards');
    }
  };

  /**
   * Submit social media activity for XP and mining boost
   */
  public submitActivity = async (req: AuthenticatedRequest, res: Response): Promise<void> => {
    try {
      const { id: userId } = req.user;
      const { platform, activityType, content, url, metadata } = validateRequest(req.body, [
        'platform', 'activityType', 'content'
      ]);

      // AI quality assessment
      const qualityScore = await this.aiQualityService.analyzeContent({
        content,
        platform,
        type: activityType,
        url,
        metadata
      });

      if (qualityScore < 0.5) {
        res.status(400).json({ 
          error: 'Content quality too low',
          score: qualityScore,
          threshold: 0.5
        });
        return;
      }

      // Calculate XP reward
      const xpReward = calculateXPMultiplier(activityType, platform, qualityScore);
      
      // Award XP
      const xpResult = await this.xpService.awardXP(userId, activityType, xpReward, {
        platform,
        qualityScore,
        content: content.substring(0, 100) // Store preview
      });

      // Apply mining boost if applicable
      let miningBoost = null;
      if (qualityScore > 1.2) {
        miningBoost = await this.miningService.applyActivityBoost(userId, activityType, qualityScore);
      }

      // Update referral network
      await this.referralService.updateNetworkActivity(userId, xpReward);

      res.json({
        message: 'Activity submitted successfully',
        xpAwarded: xpReward,
        qualityScore,
        newXP: xpResult.newTotal,
        levelUp: xpResult.levelUp,
        miningBoost: miningBoost ? {
          multiplier: miningBoost.multiplier,
          duration: miningBoost.duration
        } : null
      });

      logger.info(`Activity submitted: ${activityType} on ${platform} by user ${userId}, XP: ${xpReward}`);

    } catch (error) {
      handleError(error, res, 'Failed to submit activity');
    }
  };

  /**
   * Get user dashboard data with real-time stats
   */
  public getDashboard = async (req: AuthenticatedRequest, res: Response): Promise<void> => {
    try {
      const { id: userId } = req.user;

      const [
        miningStats,
        xpProgress,
        referralStats,
        stakingRewards,
        recentActivities,
        leaderboard,
        notifications
      ] = await Promise.all([
        this.miningService.getDashboardStats(userId),
        this.xpService.getProgressData(userId),
        this.referralService.getDashboardStats(userId),
        this.userService.getStakingRewards(userId),
        this.userService.getRecentActivities(userId, 10),
        this.userService.getUserLeaderboardPosition(userId),
        this.notificationService.getUnreadNotifications(userId, 5)
      ]);

      const dashboard = {
        mining: {
          currentRate: miningStats.currentRate,
          pendingRewards: miningStats.pendingRewards,
          totalMined: miningStats.totalMined,
          dailyProgress: miningStats.dailyProgress,
          isActive: miningStats.isActive,
          timeToNextClaim: miningStats.timeToNextClaim,
          phase: miningStats.phase
        },
        xp: {
          currentLevel: xpProgress.level,
          currentXP: xpProgress.xp,
          nextLevelXP: xpProgress.nextLevelXP,
          progressPercent: xpProgress.progressPercent,
          dailyXP: xpProgress.dailyXP,
          streak: xpProgress.streak,
          multiplier: xpProgress.multiplier
        },
        referral: {
          totalRP: referralStats.totalRP,
          tier: referralStats.tier,
          activeReferrals: referralStats.activeReferrals,
          networkSize: referralStats.networkSize,
          monthlyEarnings: referralStats.monthlyEarnings,
          qualityScore: referralStats.qualityScore
        },
        staking: {
          totalStaked: stakingRewards.totalStaked,
          pendingRewards: stakingRewards.pendingRewards,
          apy: stakingRewards.apy,
          tier: stakingRewards.tier,
          multiplier: stakingRewards.multiplier
        },
        leaderboard: {
          miningRank: leaderboard.miningRank,
          xpRank: leaderboard.xpRank,
          referralRank: leaderboard.referralRank,
          overallRank: leaderboard.overallRank
        },
        recentActivities,
        notifications,
        summary: {
          totalEarnings: miningStats.totalMined + stakingRewards.totalEarned + referralStats.totalEarnings,
          dailyEarnings: miningStats.dailyEarnings + stakingRewards.dailyRewards + referralStats.dailyEarnings,
          networkValue: referralStats.networkValue,
          achievements: xpProgress.recentAchievements
        }
      };

      res.json(dashboard);
      logger.info(`Dashboard data fetched for user ${userId}`);

    } catch (error) {
      handleError(error, res, 'Failed to fetch dashboard data');
    }
  };

  /**
   * Get referral network tree with pagination
   */
  public getReferralNetwork = async (req: AuthenticatedRequest, res: Response): Promise<void> => {
    try {
      const { id: userId } = req.user;
      const { page = 1, limit = 20, level = 1 } = req.query;

      const networkData = await this.referralService.getNetworkTree(userId, {
        page: Number(page),
        limit: Number(limit),
        maxLevel: Math.min(Number(level), 3) // Limit to 3 levels for performance
      });

      res.json({
        network: networkData.tree,
        stats: {
          totalSize: networkData.totalSize,
          activeUsers: networkData.activeUsers,
          qualityScore: networkData.qualityScore,
          totalEarnings: networkData.totalEarnings
        },
        pagination: {
          page: Number(page),
          limit: Number(limit),
          total: networkData.totalSize,
          hasMore: networkData.hasMore
        }
      });

    } catch (error) {
      handleError(error, res, 'Failed to fetch referral network');
    }
  };

  /**
   * Use special NFT card
   */
  public useSpecialCard = async (req: AuthenticatedRequest, res: Response): Promise<void> => {
    try {
      const { id: userId } = req.user;
      const { cardId } = validateRequest(req.body, ['cardId']);

      // Validate card ownership and availability
      const card = await this.userService.getUserNFT(userId, cardId);
      if (!card) {
        res.status(404).json({ error: 'Card not found or not owned' });
        return;
      }

      if (!card.canUse) {
        res.status(400).json({ error: 'Card cannot be used at this time' });
        return;
      }

      // Apply card effects
      const cardEffect = await this.userService.useSpecialCard(userId, cardId);

      res.json({
        message: 'Special card used successfully',
        effect: {
          type: cardEffect.type,
          multiplier: cardEffect.multiplier,
          duration: cardEffect.duration,
          expiresAt: cardEffect.expiresAt
        },
        cardBurned: card.singleUse
      });

      // Award XP for card usage
      await this.xpService.awardXP(userId, 'use_special_card', 50);

      logger.info(`Special card ${cardId} used by user ${userId}`);

    } catch (error) {
      handleError(error, res, 'Failed to use special card');
    }
  };

  /**
   * Get user achievements and milestones
   */
  public getAchievements = async (req: AuthenticatedRequest, res: Response): Promise<void> => {
    try {
      const { id: userId } = req.user;

      const achievements = await this.userService.getUserAchievements(userId);

      res.json({
        completed: achievements.completed,
        progress: achievements.inProgress,
        available: achievements.available,
        stats: {
          totalCompleted: achievements.completed.length,
          totalXPFromAchievements: achievements.totalXP,
          totalRewardsFromAchievements: achievements.totalRewards,
          completionRate: achievements.completionRate
        }
      });

    } catch (error) {
      handleError(error, res, 'Failed to fetch achievements');
    }
  };
}
