// finova-net/finova/mobile-sdk/ios/FinovaSDK/Sources/Models/User.swift

import Foundation
import CryptoKit

// MARK: - User Model
@objc public class User: NSObject, Codable, ObservableObject {
    
    // MARK: - Core Properties
    @Published public var id: String
    @Published public var walletAddress: String
    @Published public var email: String?
    @Published public var username: String
    @Published public var displayName: String?
    @Published public var profileImageURL: String?
    @Published public var bio: String?
    
    // MARK: - KYC & Security
    @Published public var kycStatus: KYCStatus
    @Published public var isVerified: Bool
    @Published public var securityLevel: SecurityLevel
    @Published public var lastLoginAt: Date?
    @Published public var deviceFingerprint: String?
    @Published public var humanProbabilityScore: Double // 0.0 - 1.0
    
    // MARK: - XP System
    @Published public var currentXP: Int64
    @Published public var currentLevel: Int
    @Published public var xpProgress: Double // Progress to next level (0.0 - 1.0)
    @Published public var xpTier: XPTier
    @Published public var dailyXPGained: Int64
    @Published public var totalXPEarned: Int64
    @Published public var streakDays: Int
    @Published public var longestStreak: Int
    
    // MARK: - Referral Points System
    @Published public var currentRP: Int64
    @Published public var rpTier: RPTier
    @Published public var referralCode: String
    @Published public var referralLink: String
    @Published public var directReferrals: Int
    @Published public var totalNetworkSize: Int
    @Published public var activeReferrals: Int
    @Published public var networkQualityScore: Double
    @Published public var referralEarnings: Decimal
    
    // MARK: - Mining System
    @Published public var totalMinedFIN: Decimal
    @Published public var currentMiningRate: Double // FIN per hour
    @Published public var isMining: Bool
    @Published public var miningStartTime: Date?
    @Published public var lastMiningClaim: Date?
    @Published public var miningPhase: MiningPhase
    @Published public var pioneerBonus: Double
    @Published public var miningMultiplier: Double
    
    // MARK: - Token Holdings
    @Published public var finBalance: Decimal
    @Published public var sFinBalance: Decimal // Staked FIN
    @Published public var usdFinBalance: Decimal
    @Published public var sUsdFinBalance: Decimal // Staked USDfin
    @Published public var pendingRewards: Decimal
    
    // MARK: - Staking Information
    @Published public var stakingTier: StakingTier
    @Published public var totalStaked: Decimal
    @Published public var stakingRewards: Decimal
    @Published public var stakingAPY: Double
    @Published public var stakingDuration: TimeInterval
    @Published public var canUnstake: Bool
    
    // MARK: - NFT & Cards
    @Published public var ownedNFTs: [NFTCard]
    @Published public var activeCards: [SpecialCard]
    @Published public var profileBadges: [ProfileBadge]
    @Published public var achievementNFTs: [AchievementNFT]
    
    // MARK: - Guild & Social
    @Published public var guildId: String?
    @Published public var guildRole: GuildRole?
    @Published public var connectedPlatforms: [SocialPlatform]
    @Published public var socialStats: SocialStats
    
    // MARK: - Activity & Analytics
    @Published public var lastActiveAt: Date
    @Published public var dailyActivityScore: Double
    @Published public var weeklyActivityScore: Double
    @Published public var monthlyActivityScore: Double
    @Published public var totalSessions: Int64
    @Published public var averageSessionDuration: TimeInterval
    
    // MARK: - Timestamps
    @Published public var createdAt: Date
    @Published public var updatedAt: Date
    
    // MARK: - Computed Properties
    public var nextLevelXP: Int64 {
        XPCalculator.shared.getRequiredXPForLevel(currentLevel + 1)
    }
    
    public var miningBoostMultiplier: Double {
        let xpBonus = XPCalculator.shared.getMiningMultiplier(for: xpTier)
        let rpBonus = RPCalculator.shared.getMiningBonus(for: rpTier)
        let stakingBonus = StakingCalculator.shared.getMiningBoost(for: stakingTier)
        return xpBonus * rpBonus * stakingBonus
    }
    
    public var effectiveMiningRate: Double {
        return currentMiningRate * miningBoostMultiplier * pioneerBonus
    }
    
    public var governanceVotingPower: Double {
        let stakedWeight = Double(truncating: sFinBalance as NSNumber)
        let xpWeight = Double(currentLevel) / 100.0
        let rpWeight = Double(rpTier.rawValue) * 0.2
        let activityWeight = min(dailyActivityScore / 100.0, 2.0)
        return stakedWeight * (1 + xpWeight) * (1 + rpWeight) * activityWeight
    }
    
    public var isEligibleForPremium: Bool {
        return xpTier.rawValue >= XPTier.silver.rawValue || 
               finBalance >= 500 || 
               stakingTier.rawValue >= StakingTier.premium.rawValue
    }
    
    // MARK: - Initializers
    public init(id: String = UUID().uuidString,
                walletAddress: String,
                username: String) {
        self.id = id
        self.walletAddress = walletAddress
        self.username = username
        
        // Default values
        self.kycStatus = .pending
        self.isVerified = false
        self.securityLevel = .basic
        self.humanProbabilityScore = 0.5
        
        // XP System defaults
        self.currentXP = 0
        self.currentLevel = 1
        self.xpProgress = 0.0
        self.xpTier = .bronze
        self.dailyXPGained = 0
        self.totalXPEarned = 0
        self.streakDays = 0
        self.longestStreak = 0
        
        // RP System defaults
        self.currentRP = 0
        self.rpTier = .explorer
        self.referralCode = Self.generateReferralCode()
        self.referralLink = "https://finova.network/ref/\(self.referralCode)"
        self.directReferrals = 0
        self.totalNetworkSize = 0
        self.activeReferrals = 0
        self.networkQualityScore = 0.0
        self.referralEarnings = 0
        
        // Mining defaults
        self.totalMinedFIN = 0
        self.currentMiningRate = 0.1 // Phase 1 rate
        self.isMining = false
        self.miningPhase = .pioneer
        self.pioneerBonus = 2.0
        self.miningMultiplier = 1.0
        
        // Token balances
        self.finBalance = 0
        self.sFinBalance = 0
        self.usdFinBalance = 0
        self.sUsdFinBalance = 0
        self.pendingRewards = 0
        
        // Staking
        self.stakingTier = .basic
        self.totalStaked = 0
        self.stakingRewards = 0
        self.stakingAPY = 8.0
        self.stakingDuration = 0
        self.canUnstake = true
        
        // Collections
        self.ownedNFTs = []
        self.activeCards = []
        self.profileBadges = []
        self.achievementNFTs = []
        
        // Social
        self.connectedPlatforms = []
        self.socialStats = SocialStats()
        
        // Activity
        self.lastActiveAt = Date()
        self.dailyActivityScore = 0
        self.weeklyActivityScore = 0
        self.monthlyActivityScore = 0
        self.totalSessions = 0
        self.averageSessionDuration = 0
        
        // Timestamps
        self.createdAt = Date()
        self.updatedAt = Date()
        
        super.init()
    }
    
    // MARK: - Codable
    enum CodingKeys: String, CodingKey {
        case id, walletAddress, email, username, displayName, profileImageURL, bio
        case kycStatus, isVerified, securityLevel, lastLoginAt, deviceFingerprint, humanProbabilityScore
        case currentXP, currentLevel, xpProgress, xpTier, dailyXPGained, totalXPEarned, streakDays, longestStreak
        case currentRP, rpTier, referralCode, referralLink, directReferrals, totalNetworkSize, activeReferrals, networkQualityScore, referralEarnings
        case totalMinedFIN, currentMiningRate, isMining, miningStartTime, lastMiningClaim, miningPhase, pioneerBonus, miningMultiplier
        case finBalance, sFinBalance, usdFinBalance, sUsdFinBalance, pendingRewards
        case stakingTier, totalStaked, stakingRewards, stakingAPY, stakingDuration, canUnstake
        case ownedNFTs, activeCards, profileBadges, achievementNFTs
        case guildId, guildRole, connectedPlatforms, socialStats
        case lastActiveAt, dailyActivityScore, weeklyActivityScore, monthlyActivityScore, totalSessions, averageSessionDuration
        case createdAt, updatedAt
    }
    
    // MARK: - User Actions
    @discardableResult
    public func startMining() -> Bool {
        guard !isMining else { return false }
        
        isMining = true
        miningStartTime = Date()
        updatedAt = Date()
        
        // Calculate current mining rate with all bonuses
        currentMiningRate = MiningCalculator.shared.calculateMiningRate(for: self)
        
        return true
    }
    
    public func stopMining() {
        isMining = false
        miningStartTime = nil
        updatedAt = Date()
    }
    
    public func claimMiningRewards() -> Decimal {
        guard let startTime = miningStartTime else { return 0 }
        
        let miningDuration = Date().timeIntervalSince(startTime)
        let hoursMinined = miningDuration / 3600.0
        let rewards = Decimal(effectiveMiningRate * hoursMinined)
        
        totalMinedFIN += rewards
        finBalance += rewards
        pendingRewards = 0
        lastMiningClaim = Date()
        updatedAt = Date()
        
        return rewards
    }
    
    public func addXP(_ amount: Int64, activity: ActivityType) {
        let actualAmount = XPCalculator.shared.calculateXPGain(
            baseXP: amount,
            user: self,
            activity: activity
        )
        
        currentXP += actualAmount
        dailyXPGained += actualAmount
        totalXPEarned += actualAmount
        
        // Check for level up
        let newLevel = XPCalculator.shared.getLevelForXP(currentXP)
        if newLevel > currentLevel {
            levelUp(to: newLevel)
        }
        
        // Update progress
        let currentLevelXP = XPCalculator.shared.getRequiredXPForLevel(currentLevel)
        let nextLevelXP = XPCalculator.shared.getRequiredXPForLevel(currentLevel + 1)
        xpProgress = Double(currentXP - currentLevelXP) / Double(nextLevelXP - currentLevelXP)
        
        updatedAt = Date()
    }
    
    private func levelUp(to newLevel: Int) {
        let oldLevel = currentLevel
        currentLevel = newLevel
        xpTier = XPTier.tierForLevel(newLevel)
        
        // Trigger level up rewards and notifications
        NotificationCenter.default.post(
            name: .userLevelUp,
            object: self,
            userInfo: ["oldLevel": oldLevel, "newLevel": newLevel]
        )
    }
    
    public func addReferralPoints(_ amount: Int64, source: RPSource) {
        let actualAmount = RPCalculator.shared.calculateRPGain(
            baseRP: amount,
            user: self,
            source: source
        )
        
        currentRP += actualAmount
        
        // Update RP tier
        rpTier = RPTier.tierForRP(currentRP)
        
        updatedAt = Date()
    }
    
    public func updateActivityScore(_ score: Double) {
        dailyActivityScore = score
        lastActiveAt = Date()
        updatedAt = Date()
        
        // Update human probability based on activity patterns
        humanProbabilityScore = min(1.0, max(0.1, 
            humanProbabilityScore * 0.9 + score * 0.1
        ))
    }
    
    // MARK: - Utility Methods
    public static func generateReferralCode() -> String {
        let characters = "ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789"
        return String((0..<8).map { _ in characters.randomElement()! })
    }
    
    public func canUseSpecialCard(_ card: SpecialCard) -> Bool {
        return ownedNFTs.contains { $0.id == card.id } &&
               !activeCards.contains { $0.id == card.id }
    }
    
    public func activateSpecialCard(_ card: SpecialCard) -> Bool {
        guard canUseSpecialCard(card) else { return false }
        
        activeCards.append(card)
        card.activatedAt = Date()
        updatedAt = Date()
        
        return true
    }
    
    public func hasAchievement(_ achievementId: String) -> Bool {
        return achievementNFTs.contains { $0.achievementId == achievementId }
    }
    
    public func isEligibleForStakingTier(_ tier: StakingTier) -> Bool {
        return totalStaked >= tier.minimumStake
    }
}

// MARK: - Supporting Enums and Structs

public enum KYCStatus: String, Codable, CaseIterable {
    case pending = "pending"
    case inProgress = "in_progress"
    case approved = "approved"
    case rejected = "rejected"
    case expired = "expired"
}

public enum SecurityLevel: String, Codable, CaseIterable {
    case basic = "basic"
    case enhanced = "enhanced"
    case premium = "premium"
    case enterprise = "enterprise"
    
    public var bonus: Double {
        switch self {
        case .basic: return 0.8
        case .enhanced: return 1.0
        case .premium: return 1.2
        case .enterprise: return 1.5
        }
    }
}

public enum XPTier: Int, Codable, CaseIterable {
    case bronze = 1
    case silver = 2
    case gold = 3
    case platinum = 4
    case diamond = 5
    case mythic = 6
    
    public static func tierForLevel(_ level: Int) -> XPTier {
        switch level {
        case 1...10: return .bronze
        case 11...25: return .silver
        case 26...50: return .gold
        case 51...75: return .platinum
        case 76...100: return .diamond
        default: return .mythic
        }
    }
    
    public var miningMultiplier: Double {
        switch self {
        case .bronze: return 1.0
        case .silver: return 1.3
        case .gold: return 1.9
        case .platinum: return 2.6
        case .diamond: return 3.3
        case .mythic: return 4.1
        }
    }
}

public enum RPTier: Int, Codable, CaseIterable {
    case explorer = 0
    case connector = 1
    case influencer = 2
    case leader = 3
    case ambassador = 4
    
    public static func tierForRP(_ rp: Int64) -> RPTier {
        switch rp {
        case 0...999: return .explorer
        case 1000...4999: return .connector
        case 5000...14999: return .influencer
        case 15000...49999: return .leader
        default: return .ambassador
        }
    }
    
    public var miningBonus: Double {
        switch self {
        case .explorer: return 0.0
        case .connector: return 0.2
        case .influencer: return 0.5
        case .leader: return 1.0
        case .ambassador: return 2.0
        }
    }
}

public enum MiningPhase: String, Codable, CaseIterable {
    case pioneer = "pioneer"
    case growth = "growth"
    case maturity = "maturity"
    case stability = "stability"
    
    public var baseRate: Double {
        switch self {
        case .pioneer: return 0.1
        case .growth: return 0.05
        case .maturity: return 0.025
        case .stability: return 0.01
        }
    }
}

public enum StakingTier: Int, Codable, CaseIterable {
    case basic = 0
    case premium = 1
    case vip = 2
    case elite = 3
    case legendary = 4
    
    public var minimumStake: Decimal {
        switch self {
        case .basic: return 100
        case .premium: return 500
        case .vip: return 1000
        case .elite: return 5000
        case .legendary: return 10000
        }
    }
    
    public var apy: Double {
        switch self {
        case .basic: return 8.0
        case .premium: return 10.0
        case .vip: return 12.0
        case .elite: return 14.0
        case .legendary: return 15.0
        }
    }
}

public enum GuildRole: String, Codable, CaseIterable {
    case member = "member"
    case officer = "officer"
    case leader = "leader"
    case master = "master"
}

public enum ActivityType: String, Codable, CaseIterable {
    case post = "post"
    case comment = "comment"
    case like = "like"
    case share = "share"
    case follow = "follow"
    case login = "login"
    case quest = "quest"
    case viral = "viral"
}

public enum RPSource: String, Codable, CaseIterable {
    case referralSignup = "referral_signup"
    case referralKYC = "referral_kyc"
    case referralActivity = "referral_activity"
    case networkBonus = "network_bonus"
}

// MARK: - Supporting Models

public struct SocialPlatform: Codable {
    public let id: String
    public let name: String
    public let username: String?
    public let isConnected: Bool
    public let lastSync: Date?
    public let followerCount: Int
    public let engagementRate: Double
}

public struct SocialStats: Codable {
    public var totalPosts: Int64 = 0
    public var totalLikes: Int64 = 0
    public var totalComments: Int64 = 0
    public var totalShares: Int64 = 0
    public var totalFollowers: Int64 = 0
    public var engagementRate: Double = 0.0
    public var viralPosts: Int = 0
}

public struct NFTCard: Codable, Identifiable {
    public let id: String
    public let tokenId: String
    public let name: String
    public let description: String
    public let imageURL: String
    public let rarity: CardRarity
    public let category: CardCategory
    public let attributes: [String: Any]
    public let mintedAt: Date
    
    private enum CodingKeys: String, CodingKey {
        case id, tokenId, name, description, imageURL, rarity, category, mintedAt
    }
}

public struct SpecialCard: Codable, Identifiable {
    public let id: String
    public let cardId: String
    public let name: String
    public let effect: CardEffect
    public let duration: TimeInterval
    public let multiplier: Double
    public var activatedAt: Date?
    public var expiresAt: Date?
    
    public var isActive: Bool {
        guard let activated = activatedAt else { return false }
        return Date().timeIntervalSince(activated) < duration
    }
}

public struct ProfileBadge: Codable, Identifiable {
    public let id: String
    public let name: String
    public let description: String
    public let imageURL: String
    public let rarity: BadgeRarity
    public let earnedAt: Date
    public let bonus: Double
}

public struct AchievementNFT: Codable, Identifiable {
    public let id: String
    public let achievementId: String
    public let tokenId: String
    public let name: String
    public let description: String
    public let imageURL: String
    public let unlockedAt: Date
    public let bonus: AchievementBonus
}

public enum CardRarity: String, Codable, CaseIterable {
    case common = "common"
    case uncommon = "uncommon"
    case rare = "rare"
    case epic = "epic"
    case legendary = "legendary"
}

public enum CardCategory: String, Codable, CaseIterable {
    case mining = "mining"
    case xp = "xp"
    case referral = "referral"
    case social = "social"
    case special = "special"
}

public enum CardEffect: String, Codable, CaseIterable {
    case miningBoost = "mining_boost"
    case xpMultiplier = "xp_multiplier"
    case referralBonus = "referral_bonus"
    case streakSaver = "streak_saver"
    case networkAmplifier = "network_amplifier"
}

public enum BadgeRarity: String, Codable, CaseIterable {
    case bronze = "bronze"
    case silver = "silver"
    case gold = "gold"
    case platinum = "platinum"
    case diamond = "diamond"
}

public struct AchievementBonus: Codable {
    public let type: BonusType
    public let value: Double
    public let isPermanent: Bool
}

public enum BonusType: String, Codable, CaseIterable {
    case miningRate = "mining_rate"
    case xpGain = "xp_gain"
    case referralBonus = "referral_bonus"
    case stakingReward = "staking_reward"
}

// MARK: - Notification Names
extension Notification.Name {
    public static let userLevelUp = Notification.Name("userLevelUp")
    public static let userTierUp = Notification.Name("userTierUp")
    public static let miningRewardsClaimed = Notification.Name("miningRewardsClaimed")
    public static let specialCardActivated = Notification.Name("specialCardActivated")
}
