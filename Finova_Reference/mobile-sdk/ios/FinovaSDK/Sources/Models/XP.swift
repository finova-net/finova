// finova-net/finova/mobile-sdk/ios/FinovaSDK/Sources/Models/XP.swift

import Foundation
import CryptoKit

// MARK: - XP Activity Types
public enum XPActivityType: String, CaseIterable, Codable {
    case originalPost = "original_post"
    case photoPost = "photo_post"
    case videoContent = "video_content"
    case storyStatus = "story_status"
    case meaningfulComment = "meaningful_comment"
    case likeReact = "like_react"
    case shareRepost = "share_repost"
    case followSubscribe = "follow_subscribe"
    case dailyLogin = "daily_login"
    case dailyQuest = "daily_quest"
    case milestone = "milestone"
    case viralContent = "viral_content"
    
    public var baseXP: Int {
        switch self {
        case .originalPost: return 50
        case .photoPost: return 75
        case .videoContent: return 150
        case .storyStatus: return 25
        case .meaningfulComment: return 25
        case .likeReact: return 5
        case .shareRepost: return 15
        case .followSubscribe: return 20
        case .dailyLogin: return 10
        case .dailyQuest: return 100
        case .milestone: return 500
        case .viralContent: return 1000
        }
    }
    
    public var dailyLimit: Int? {
        switch self {
        case .originalPost: return nil
        case .photoPost: return 20
        case .videoContent: return 10
        case .storyStatus: return 50
        case .meaningfulComment: return 100
        case .likeReact: return 200
        case .shareRepost: return 50
        case .followSubscribe: return 25
        case .dailyLogin: return 1
        case .dailyQuest: return 3
        case .milestone: return nil
        case .viralContent: return nil
        }
    }
}

// MARK: - Social Platform Types
public enum SocialPlatform: String, CaseIterable, Codable {
    case tiktok = "tiktok"
    case instagram = "instagram"
    case youtube = "youtube"
    case facebook = "facebook"
    case twitter = "twitter"
    case finova = "finova"
    
    public var multiplier: Double {
        switch self {
        case .tiktok: return 1.3
        case .instagram: return 1.2
        case .youtube: return 1.4
        case .facebook: return 1.1
        case .twitter: return 1.2
        case .finova: return 1.0
        }
    }
}

// MARK: - XP Badge Tiers
public enum XPBadgeTier: String, CaseIterable, Codable {
    case bronze = "bronze"
    case silver = "silver"
    case gold = "gold"
    case platinum = "platinum"
    case diamond = "diamond"
    case mythic = "mythic"
    
    public var levelRange: ClosedRange<Int> {
        switch self {
        case .bronze: return 1...10
        case .silver: return 11...25
        case .gold: return 26...50
        case .platinum: return 51...75
        case .diamond: return 76...100
        case .mythic: return 101...Int.max
        }
    }
    
    public var miningMultiplier: ClosedRange<Double> {
        switch self {
        case .bronze: return 1.0...1.2
        case .silver: return 1.3...1.8
        case .gold: return 1.9...2.5
        case .platinum: return 2.6...3.2
        case .diamond: return 3.3...4.0
        case .mythic: return 4.1...5.0
        }
    }
    
    public var dailyFinCap: ClosedRange<Double> {
        switch self {
        case .bronze: return 0.5...2.0
        case .silver: return 2.0...4.0
        case .gold: return 4.0...6.0
        case .platinum: return 6.0...8.0
        case .diamond: return 8.0...10.0
        case .mythic: return 10.0...15.0
        }
    }
    
    public var requiredXP: Int {
        switch self {
        case .bronze: return 0
        case .silver: return 1000
        case .gold: return 5000
        case .platinum: return 20000
        case .diamond: return 50000
        case .mythic: return 100000
        }
    }
}

// MARK: - XP Activity Record
public struct XPActivity: Codable, Identifiable, Hashable {
    public let id: UUID
    public let type: XPActivityType
    public let platform: SocialPlatform
    public let timestamp: Date
    public let baseXP: Int
    public let qualityScore: Double
    public let contentHash: String?
    public let metadata: [String: String]
    
    // Calculated properties
    public var platformMultiplier: Double { platform.multiplier }
    public var finalXP: Int {
        return Int(Double(baseXP) * platformMultiplier * qualityScore)
    }
    
    public init(
        type: XPActivityType,
        platform: SocialPlatform,
        qualityScore: Double = 1.0,
        contentHash: String? = nil,
        metadata: [String: String] = [:]
    ) {
        self.id = UUID()
        self.type = type
        self.platform = platform
        self.timestamp = Date()
        self.baseXP = type.baseXP
        self.qualityScore = max(0.5, min(2.0, qualityScore))
        self.contentHash = contentHash
        self.metadata = metadata
    }
}

// MARK: - XP Statistics
public struct XPStatistics: Codable {
    public let totalXP: Int
    public let dailyXP: Int
    public let weeklyXP: Int
    public let monthlyXP: Int
    public let currentLevel: Int
    public let currentTier: XPBadgeTier
    public let progressToNextLevel: Double
    public let streak: Int
    public let activities: [XPActivityType: Int]
    public let platforms: [SocialPlatform: Int]
    
    public init(
        totalXP: Int = 0,
        dailyXP: Int = 0,
        weeklyXP: Int = 0,
        monthlyXP: Int = 0,
        currentLevel: Int = 1,
        streak: Int = 0,
        activities: [XPActivityType: Int] = [:],
        platforms: [SocialPlatform: Int] = [:]
    ) {
        self.totalXP = totalXP
        self.dailyXP = dailyXP
        self.weeklyXP = weeklyXP
        self.monthlyXP = monthlyXP
        self.currentLevel = currentLevel
        self.currentTier = XPBadgeTier.allCases.first { $0.levelRange.contains(currentLevel) } ?? .bronze
        self.progressToNextLevel = XPManager.calculateProgressToNextLevel(currentXP: totalXP, currentLevel: currentLevel)
        self.streak = streak
        self.activities = activities
        self.platforms = platforms
    }
}

// MARK: - XP Manager
public class XPManager: ObservableObject {
    @Published public private(set) var statistics = XPStatistics()
    @Published public private(set) var recentActivities: [XPActivity] = []
    
    private let userDefaults = UserDefaults.standard
    private let calendar = Calendar.current
    
    // MARK: - Constants
    private struct Constants {
        static let xpStatisticsKey = "finova_xp_statistics"
        static let recentActivitiesKey = "finova_recent_activities"
        static let lastActivityDate = "finova_last_activity_date"
        static let maxRecentActivities = 100
    }
    
    public init() {
        loadStoredData()
        updateDailyStats()
    }
    
    // MARK: - Public Methods
    
    /// Calculate XP gain for a specific activity
    public func calculateXP(
        activity: XPActivityType,
        platform: SocialPlatform,
        qualityScore: Double = 1.0,
        streakBonus: Double? = nil,
        levelProgression: Double? = nil
    ) -> Int {
        let baseXP = Double(activity.baseXP)
        let platformMult = platform.multiplier
        let quality = max(0.5, min(2.0, qualityScore))
        let streak = streakBonus ?? calculateStreakBonus()
        let progression = levelProgression ?? calculateLevelProgression()
        
        let finalXP = baseXP * platformMult * quality * streak * progression
        return max(1, Int(finalXP))
    }
    
    /// Add XP activity and update statistics
    public func addActivity(_ activity: XPActivity) {
        // Validate daily limits
        guard canPerformActivity(activity.type) else {
            print("Daily limit reached for activity: \(activity.type)")
            return
        }
        
        // Add to recent activities
        recentActivities.insert(activity, at: 0)
        if recentActivities.count > Constants.maxRecentActivities {
            recentActivities.removeLast()
        }
        
        // Update statistics
        updateStatistics(with: activity)
        
        // Save data
        saveData()
        
        // Update last activity date for streak calculation
        userDefaults.set(Date(), forKey: Constants.lastActivityDate)
    }
    
    /// Check if user can perform specific activity (daily limits)
    public func canPerformActivity(_ type: XPActivityType) -> Bool {
        guard let limit = type.dailyLimit else { return true }
        
        let todayActivities = getTodayActivities()
        let activityCount = todayActivities.filter { $0.type == type }.count
        
        return activityCount < limit
    }
    
    /// Get remaining activities for today
    public func getRemainingActivities(_ type: XPActivityType) -> Int? {
        guard let limit = type.dailyLimit else { return nil }
        
        let todayActivities = getTodayActivities()
        let activityCount = todayActivities.filter { $0.type == type }.count
        
        return max(0, limit - activityCount)
    }
    
    /// Calculate current mining multiplier based on XP level
    public func getMiningMultiplier() -> Double {
        let tier = statistics.currentTier
        let level = statistics.currentLevel
        let range = tier.miningMultiplier
        
        // Calculate position within tier
        let tierProgress = Double(level - tier.levelRange.lowerBound) / Double(tier.levelRange.count)
        return range.lowerBound + (range.upperBound - range.lowerBound) * tierProgress
    }
    
    /// Get daily FIN cap based on current level
    public func getDailyFinCap() -> Double {
        let tier = statistics.currentTier
        let level = statistics.currentLevel
        let range = tier.dailyFinCap
        
        let tierProgress = Double(level - tier.levelRange.lowerBound) / Double(tier.levelRange.count)
        return range.lowerBound + (range.upperBound - range.lowerBound) * tierProgress
    }
    
    // MARK: - Private Methods
    
    private func updateStatistics(with activity: XPActivity) {
        let newTotalXP = statistics.totalXP + activity.finalXP
        let newLevel = calculateLevel(from: newTotalXP)
        
        // Update activity counts
        var activities = statistics.activities
        activities[activity.type, default: 0] += 1
        
        var platforms = statistics.platforms
        platforms[activity.platform, default: 0] += 1
        
        // Calculate time-based XP
        let dailyXP = calculateDailyXP() + activity.finalXP
        let weeklyXP = calculateWeeklyXP() + activity.finalXP
        let monthlyXP = calculateMonthlyXP() + activity.finalXP
        
        statistics = XPStatistics(
            totalXP: newTotalXP,
            dailyXP: dailyXP,
            weeklyXP: weeklyXP,
            monthlyXP: monthlyXP,
            currentLevel: newLevel,
            streak: calculateCurrentStreak(),
            activities: activities,
            platforms: platforms
        )
    }
    
    private func calculateLevel(from totalXP: Int) -> Int {
        // Level formula: Level = floor(sqrt(totalXP / 100)) + 1
        return Int(sqrt(Double(totalXP) / 100.0)) + 1
    }
    
    private func calculateStreakBonus() -> Double {
        let streak = calculateCurrentStreak()
        return min(3.0, 1.0 + Double(streak) * 0.1)
    }
    
    private func calculateLevelProgression() -> Double {
        return exp(-0.01 * Double(statistics.currentLevel))
    }
    
    private func calculateCurrentStreak() -> Int {
        guard let lastActivity = userDefaults.object(forKey: Constants.lastActivityDate) as? Date else {
            return 0
        }
        
        let daysBetween = calendar.dateComponents([.day], from: lastActivity, to: Date()).day ?? 0
        
        if daysBetween <= 1 {
            return statistics.streak + (daysBetween == 1 ? 1 : 0)
        } else {
            return 0 // Streak broken
        }
    }
    
    public static func calculateProgressToNextLevel(currentXP: Int, currentLevel: Int) -> Double {
        let currentLevelXP = (currentLevel - 1) * (currentLevel - 1) * 100
        let nextLevelXP = currentLevel * currentLevel * 100
        
        let progress = Double(currentXP - currentLevelXP) / Double(nextLevelXP - currentLevelXP)
        return max(0.0, min(1.0, progress))
    }
    
    private func getTodayActivities() -> [XPActivity] {
        let today = calendar.startOfDay(for: Date())
        let tomorrow = calendar.date(byAdding: .day, value: 1, to: today)!
        
        return recentActivities.filter { activity in
            activity.timestamp >= today && activity.timestamp < tomorrow
        }
    }
    
    private func calculateDailyXP() -> Int {
        return getTodayActivities().reduce(0) { $0 + $1.finalXP }
    }
    
    private func calculateWeeklyXP() -> Int {
        let weekAgo = calendar.date(byAdding: .day, value: -7, to: Date())!
        return getActivitiesSince(weekAgo).reduce(0) { $0 + $1.finalXP }
    }
    
    private func calculateMonthlyXP() -> Int {
        let monthAgo = calendar.date(byAdding: .day, value: -30, to: Date())!
        return getActivitiesSince(monthAgo).reduce(0) { $0 + $1.finalXP }
    }
    
    private func getActivitiesSince(_ date: Date) -> [XPActivity] {
        return recentActivities.filter { $0.timestamp >= date }
    }
    
    private func updateDailyStats() {
        // Reset daily stats if new day
        guard let lastUpdate = userDefaults.object(forKey: "last_daily_update") as? Date else {
            userDefaults.set(Date(), forKey: "last_daily_update")
            return
        }
        
        if !calendar.isDate(lastUpdate, inSameDayAs: Date()) {
            // New day - recalculate daily stats
            let dailyXP = calculateDailyXP()
            statistics = XPStatistics(
                totalXP: statistics.totalXP,
                dailyXP: dailyXP,
                weeklyXP: statistics.weeklyXP,
                monthlyXP: statistics.monthlyXP,
                currentLevel: statistics.currentLevel,
                streak: calculateCurrentStreak(),
                activities: statistics.activities,
                platforms: statistics.platforms
            )
            userDefaults.set(Date(), forKey: "last_daily_update")
        }
    }
    
    // MARK: - Data Persistence
    
    private func saveData() {
        do {
            let statisticsData = try JSONEncoder().encode(statistics)
            userDefaults.set(statisticsData, forKey: Constants.xpStatisticsKey)
            
            let activitiesData = try JSONEncoder().encode(recentActivities)
            userDefaults.set(activitiesData, forKey: Constants.recentActivitiesKey)
        } catch {
            print("Failed to save XP data: \(error)")
        }
    }
    
    private func loadStoredData() {
        // Load statistics
        if let statisticsData = userDefaults.data(forKey: Constants.xpStatisticsKey) {
            do {
                statistics = try JSONDecoder().decode(XPStatistics.self, from: statisticsData)
            } catch {
                print("Failed to decode XP statistics: \(error)")
            }
        }
        
        // Load recent activities
        if let activitiesData = userDefaults.data(forKey: Constants.recentActivitiesKey) {
            do {
                recentActivities = try JSONDecoder().decode([XPActivity].self, from: activitiesData)
            } catch {
                print("Failed to decode XP activities: \(error)")
            }
        }
    }
    
    // MARK: - Utility Methods
    
    /// Generate content hash for duplicate detection
    public static func generateContentHash(_ content: String) -> String {
        let data = Data(content.utf8)
        let hash = SHA256.hash(data: data)
        return hash.map { String(format: "%02hhx", $0) }.joined()
    }
    
    /// Check if content is duplicate based on hash
    public func isDuplicateContent(_ hash: String) -> Bool {
        return recentActivities.contains { $0.contentHash == hash }
    }
    
    /// Get XP breakdown by platform for analytics
    public func getXPBreakdown() -> [String: Any] {
        return [
            "total_xp": statistics.totalXP,
            "current_level": statistics.currentLevel,
            "current_tier": statistics.currentTier.rawValue,
            "mining_multiplier": getMiningMultiplier(),
            "daily_fin_cap": getDailyFinCap(),
            "streak": statistics.streak,
            "activities_breakdown": statistics.activities.mapKeys { $0.rawValue },
            "platforms_breakdown": statistics.platforms.mapKeys { $0.rawValue }
        ]
    }
}

// MARK: - Extensions

extension Dictionary {
    func mapKeys<T>(_ transform: (Key) -> T) -> [T: Value] {
        var result: [T: Value] = [:]
        for (key, value) in self {
            result[transform(key)] = value
        }
        return result
    }
}

// MARK: - XP Calculation Helper
public struct XPCalculationHelper {
    
    /// Calculate XP with all bonuses and multipliers
    public static func calculateFinalXP(
        activity: XPActivityType,
        platform: SocialPlatform,
        qualityScore: Double,
        userLevel: Int,
        streakDays: Int
    ) -> Int {
        let baseXP = Double(activity.baseXP)
        let platformMultiplier = platform.multiplier
        let quality = max(0.5, min(2.0, qualityScore))
        let streakBonus = min(3.0, 1.0 + Double(streakDays) * 0.1)
        let levelProgression = exp(-0.01 * Double(userLevel))
        
        let finalXP = baseXP * platformMultiplier * quality * streakBonus * levelProgression
        return max(1, Int(finalXP))
    }
    
    /// Calculate XP required for specific level
    public static func xpRequiredForLevel(_ level: Int) -> Int {
        return (level - 1) * (level - 1) * 100
    }
    
    /// Get tier from level
    public static func getTierFromLevel(_ level: Int) -> XPBadgeTier {
        return XPBadgeTier.allCases.first { $0.levelRange.contains(level) } ?? .bronze
    }
}
