// finova-net/finova/mobile-sdk/ios/FinovaSDK/Sources/Models/Mining.swift

import Foundation
import CryptoKit

// MARK: - Mining Phase Enumeration
public enum MiningPhase: Int, CaseIterable, Codable {
    case finizen = 1    // 0-100K users
    case growth = 2     // 100K-1M users
    case maturity = 3   // 1M-10M users
    case stability = 4  // 10M+ users
    
    public var baseRate: Double {
        switch self {
        case .finizen: return 0.1
        case .growth: return 0.05
        case .maturity: return 0.025
        case .stability: return 0.01
        }
    }
    
    public var finizenBonus: Double {
        switch self {
        case .finizen: return 2.0
        case .growth: return 1.5
        case .maturity: return 1.2
        case .stability: return 1.0
        }
    }
    
    public var maxDailyFIN: Double {
        switch self {
        case .finizen: return 4.8
        case .growth: return 1.8
        case .maturity: return 0.72
        case .stability: return 0.24
        }
    }
}

// MARK: - Mining Boost Types
public enum MiningBoostType: String, CaseIterable, Codable {
    case dailyPost = "daily_social_post"
    case dailyQuest = "complete_daily_quest"
    case referralKYC = "referral_kyc_success"
    case specialCard = "use_special_card"
    case guildParticipation = "guild_participation"
    
    public var boostPercentage: Double {
        switch self {
        case .dailyPost: return 0.20
        case .dailyQuest: return 0.50
        case .referralKYC: return 1.00
        case .specialCard: return 2.00
        case .guildParticipation: return 0.30
        }
    }
    
    public var duration: TimeInterval {
        switch self {
        case .dailyPost: return 24 * 3600
        case .dailyQuest: return 12 * 3600
        case .referralKYC: return 48 * 3600
        case .specialCard: return 4 * 3600 // Variable, default 4h
        case .guildParticipation: return 12 * 3600 // Event duration
        }
    }
    
    public var isStackable: Bool {
        switch self {
        case .dailyPost: return true
        case .dailyQuest: return false
        case .referralKYC: return true
        case .specialCard: return true
        case .guildParticipation: return true
        }
    }
    
    public var maxStack: Int {
        switch self {
        case .dailyPost: return 3
        case .dailyQuest: return 1
        case .referralKYC: return 5
        case .specialCard: return 10
        case .guildParticipation: return 3
        }
    }
}

// MARK: - Mining Boost Model
public struct MiningBoost: Codable, Identifiable {
    public let id: String
    public let type: MiningBoostType
    public let boostPercentage: Double
    public let activatedAt: Date
    public let expiresAt: Date
    public let stackCount: Int
    public let isActive: Bool
    
    public init(type: MiningBoostType, stackCount: Int = 1, customDuration: TimeInterval? = nil) {
        self.id = UUID().uuidString
        self.type = type
        self.boostPercentage = type.boostPercentage
        self.activatedAt = Date()
        self.expiresAt = Date().addingTimeInterval(customDuration ?? type.duration)
        self.stackCount = min(stackCount, type.maxStack)
        self.isActive = true
    }
    
    public var totalBoostMultiplier: Double {
        guard isActive && Date() < expiresAt else { return 1.0 }
        let baseMultiplier = 1.0 + boostPercentage
        return type.isStackable ? pow(baseMultiplier, Double(stackCount)) : baseMultiplier
    }
    
    public var remainingTime: TimeInterval {
        return max(0, expiresAt.timeIntervalSince(Date()))
    }
}

// MARK: - Mining Statistics
public struct MiningStatistics: Codable {
    public let totalMined: Double
    public let todayMined: Double
    public let currentHourlyRate: Double
    public let averageHourlyRate: Double
    public let miningStreak: Int
    public let totalMiningHours: Double
    public let lastMiningSession: Date?
    public let projectedDailyEarnings: Double
    public let lifetimeRank: Int
    public let networkContribution: Double
    
    public init(totalMined: Double = 0, todayMined: Double = 0, currentHourlyRate: Double = 0,
                averageHourlyRate: Double = 0, miningStreak: Int = 0, totalMiningHours: Double = 0,
                lastMiningSession: Date? = nil, projectedDailyEarnings: Double = 0,
                lifetimeRank: Int = 0, networkContribution: Double = 0) {
        self.totalMined = totalMined
        self.todayMined = todayMined
        self.currentHourlyRate = currentHourlyRate
        self.averageHourlyRate = averageHourlyRate
        self.miningStreak = miningStreak
        self.totalMiningHours = totalMiningHours
        self.lastMiningSession = lastMiningSession
        self.projectedDailyEarnings = projectedDailyEarnings
        self.lifetimeRank = lifetimeRank
        self.networkContribution = networkContribution
    }
}

// MARK: - Main Mining Model
public struct Mining: Codable, Identifiable {
    public let id: String
    public let userId: String
    public let walletAddress: String
    
    // Mining State
    public let isActive: Bool
    public let isPaused: Bool
    public let lastActiveAt: Date?
    public let sessionStartTime: Date?
    
    // Core Mining Data
    public let currentPhase: MiningPhase
    public let baseHourlyRate: Double
    public let totalNetworkUsers: Int
    public let userHoldings: Double
    public let activeReferrals: Int
    public let isKYCVerified: Bool
    
    // Calculated Rates
    public let finizenBonus: Double
    public let referralBonus: Double
    public let securityBonus: Double
    public let regressionFactor: Double
    public let finalHourlyRate: Double
    
    // Boosts and Multipliers
    public let activeBoosts: [MiningBoost]
    public let xpLevelMultiplier: Double
    public let rpTierMultiplier: Double
    public let qualityScore: Double
    
    // Statistics
    public let statistics: MiningStatistics
    
    // Timestamps
    public let createdAt: Date
    public let updatedAt: Date
    
    public init(userId: String, walletAddress: String, totalNetworkUsers: Int,
                userHoldings: Double, activeReferrals: Int, isKYCVerified: Bool,
                xpLevel: Int, rpTier: Int, qualityScore: Double = 1.0,
                activeBoosts: [MiningBoost] = []) {
        
        self.id = UUID().uuidString
        self.userId = userId
        self.walletAddress = walletAddress
        
        // State
        self.isActive = true
        self.isPaused = false
        self.lastActiveAt = Date()
        self.sessionStartTime = Date()
        
        // Phase determination
        self.currentPhase = Self.determinePhase(totalUsers: totalNetworkUsers)
        self.totalNetworkUsers = totalNetworkUsers
        self.userHoldings = userHoldings
        self.activeReferrals = activeReferrals
        self.isKYCVerified = isKYCVerified
        
        // Base calculations
        self.baseHourlyRate = currentPhase.baseRate
        
        // Bonus calculations
        self.finizenBonus = max(1.0, currentPhase.finizenBonus - (Double(totalNetworkUsers) / 1_000_000))
        self.referralBonus = 1.0 + (Double(activeReferrals) * 0.1)
        self.securityBonus = isKYCVerified ? 1.2 : 0.8
        self.regressionFactor = exp(-0.001 * userHoldings)
        
        // Multipliers
        self.xpLevelMultiplier = 1.0 + (Double(xpLevel) / 100.0)
        self.rpTierMultiplier = 1.0 + (Double(rpTier) * 0.2)
        self.qualityScore = qualityScore
        self.activeBoosts = activeBoosts
        
        // Final rate calculation
        let boostMultiplier = activeBoosts.reduce(1.0) { $0 * $1.totalBoostMultiplier }
        self.finalHourlyRate = baseHourlyRate * finizenBonus * referralBonus * 
                               securityBonus * regressionFactor * xpLevelMultiplier * 
                               rpTierMultiplier * qualityScore * boostMultiplier
        
        // Initialize statistics
        self.statistics = MiningStatistics(currentHourlyRate: finalHourlyRate,
                                         projectedDailyEarnings: min(finalHourlyRate * 24, currentPhase.maxDailyFIN))
        
        // Timestamps
        self.createdAt = Date()
        self.updatedAt = Date()
    }
    
    // MARK: - Phase Determination
    private static func determinePhase(totalUsers: Int) -> MiningPhase {
        switch totalUsers {
        case 0..<100_000: return .finizen
        case 100_000..<1_000_000: return .growth
        case 1_000_000..<10_000_000: return .maturity
        default: return .stability
        }
    }
    
    // MARK: - Mining Calculations
    public func calculateHourlyEarnings() -> Double {
        guard isActive && !isPaused else { return 0.0 }
        return min(finalHourlyRate, currentPhase.maxDailyFIN / 24.0)
    }
    
    public func calculateDailyEarnings() -> Double {
        let hourlyRate = calculateHourlyEarnings()
        return min(hourlyRate * 24, currentPhase.maxDailyFIN)
    }
    
    public func calculateProjectedWeeklyEarnings() -> Double {
        return calculateDailyEarnings() * 7
    }
    
    public func calculateProjectedMonthlyEarnings() -> Double {
        return calculateDailyEarnings() * 30
    }
    
    // MARK: - Boost Management
    public func addBoost(_ boostType: MiningBoostType, stackCount: Int = 1, customDuration: TimeInterval? = nil) -> Mining {
        let newBoost = MiningBoost(type: boostType, stackCount: stackCount, customDuration: customDuration)
        var updatedBoosts = activeBoosts.filter { $0.type != boostType || boostType.isStackable }
        updatedBoosts.append(newBoost)
        
        return Mining(userId: userId, walletAddress: walletAddress, totalNetworkUsers: totalNetworkUsers,
                     userHoldings: userHoldings, activeReferrals: activeReferrals, isKYCVerified: isKYCVerified,
                     xpLevel: Int(xpLevelMultiplier * 100 - 100), rpTier: Int((rpTierMultiplier - 1.0) / 0.2),
                     qualityScore: qualityScore, activeBoosts: updatedBoosts)
    }
    
    public func removeExpiredBoosts() -> Mining {
        let activeBoosts = self.activeBoosts.filter { $0.remainingTime > 0 }
        
        return Mining(userId: userId, walletAddress: walletAddress, totalNetworkUsers: totalNetworkUsers,
                     userHoldings: userHoldings, activeReferrals: activeReferrals, isKYCVerified: isKYCVerified,
                     xpLevel: Int(xpLevelMultiplier * 100 - 100), rpTier: Int((rpTierMultiplier - 1.0) / 0.2),
                     qualityScore: qualityScore, activeBoosts: activeBoosts)
    }
    
    // MARK: - Session Management
    public func startMiningSession() -> Mining {
        var updated = self
        // Create new instance with updated session time
        return Mining(userId: userId, walletAddress: walletAddress, totalNetworkUsers: totalNetworkUsers,
                     userHoldings: userHoldings, activeReferrals: activeReferrals, isKYCVerified: isKYCVerified,
                     xpLevel: Int(xpLevelMultiplier * 100 - 100), rpTier: Int((rpTierMultiplier - 1.0) / 0.2),
                     qualityScore: qualityScore, activeBoosts: activeBoosts)
    }
    
    public func pauseMining() -> Mining {
        // Return paused version
        return self // Implementation would create paused state
    }
    
    public func resumeMining() -> Mining {
        // Return resumed version
        return self // Implementation would create resumed state
    }
    
    // MARK: - Validation
    public func validateMiningEligibility() -> (isEligible: Bool, reason: String?) {
        if !isKYCVerified && currentPhase == .stability {
            return (false, "KYC verification required for current mining phase")
        }
        
        if userHoldings > 1_000_000 {
            return (false, "Holdings exceed maximum mining threshold")
        }
        
        if let lastActive = lastActiveAt, Date().timeIntervalSince(lastActive) > 24 * 3600 {
            return (false, "Mining session expired. Please restart.")
        }
        
        return (true, nil)
    }
    
    // MARK: - Utility Methods
    public var isEligibleForMining: Bool {
        return validateMiningEligibility().isEligible
    }
    
    public var nextPhaseUsersRequired: Int {
        switch currentPhase {
        case .finizen: return 100_000 - totalNetworkUsers
        case .growth: return 1_000_000 - totalNetworkUsers
        case .maturity: return 10_000_000 - totalNetworkUsers
        case .stability: return 0
        }
    }
    
    public var phaseProgressPercentage: Double {
        switch currentPhase {
        case .finizen: return Double(totalNetworkUsers) / 100_000.0
        case .growth: return Double(totalNetworkUsers - 100_000) / 900_000.0
        case .maturity: return Double(totalNetworkUsers - 1_000_000) / 9_000_000.0
        case .stability: return 1.0
        }
    }
    
    // MARK: - Debugging and Monitoring
    public var debugDescription: String {
        return """
        Mining Debug Info:
        - User ID: \(userId)
        - Phase: \(currentPhase) (\(totalNetworkUsers) users)
        - Base Rate: \(baseHourlyRate) FIN/hour
        - Final Rate: \(finalHourlyRate) FIN/hour
        - Bonuses: Finizen(\(finizenBonus)x), Referral(\(referralBonus)x), Security(\(securityBonus)x)
        - Regression: \(regressionFactor)x
        - Active Boosts: \(activeBoosts.count)
        - Daily Cap: \(currentPhase.maxDailyFIN) FIN
        - Eligible: \(isEligibleForMining)
        """
    }
}

// MARK: - Extensions
extension Mining {
    public static let mock = Mining(
        userId: "user_123",
        walletAddress: "0x742d35Cc6659C458814b3A4Ff7AE85F3c1BDe075",
        totalNetworkUsers: 50_000,
        userHoldings: 1_000,
        activeReferrals: 5,
        isKYCVerified: true,
        xpLevel: 25,
        rpTier: 2,
        qualityScore: 1.5,
        activeBoosts: [
            MiningBoost(type: .dailyPost, stackCount: 2),
            MiningBoost(type: .dailyQuest)
        ]
    )
}

// MARK: - Hash and Equatable for Security
extension Mining: Hashable {
    public func hash(into hasher: inout Hasher) {
        hasher.combine(id)
        hasher.combine(userId)
        hasher.combine(walletAddress)
        hasher.combine(finalHourlyRate)
        hasher.combine(updatedAt)
    }
    
    public static func == (lhs: Mining, rhs: Mining) -> Bool {
        return lhs.id == rhs.id &&
               lhs.userId == rhs.userId &&
               lhs.finalHourlyRate == rhs.finalHourlyRate &&
               lhs.updatedAt == rhs.updatedAt
    }
}
