// finova-net/finova/mobile-sdk/ios/FinovaSDK/Sources/Services/XPService.swift

import Foundation
import Combine
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
    case achieveMilestone = "achieve_milestone"
    case viralContent = "viral_content"
    
    var baseXP: Int {
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
        case .achieveMilestone: return 500
        case .viralContent: return 1000
        }
    }
    
    var dailyLimit: Int? {
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
        case .achieveMilestone: return nil
        case .viralContent: return nil
        }
    }
}

// MARK: - Social Platforms
public enum SocialPlatform: String, CaseIterable, Codable {
    case instagram = "instagram"
    case tiktok = "tiktok"
    case youtube = "youtube"
    case facebook = "facebook"
    case twitter = "twitter"
    case app = "app"
    
    var multiplier: Double {
        switch self {
        case .tiktok: return 1.3
        case .instagram: return 1.2
        case .youtube: return 1.4
        case .twitter: return 1.2
        case .facebook: return 1.1
        case .app: return 1.0
        }
    }
}

// MARK: - XP Level Tiers
public enum XPTier: String, CaseIterable, Codable {
    case bronze = "bronze"
    case silver = "silver"
    case gold = "gold"
    case platinum = "platinum"
    case diamond = "diamond"
    case mythic = "mythic"
    
    var levelRange: ClosedRange<Int> {
        switch self {
        case .bronze: return 1...10
        case .silver: return 11...25
        case .gold: return 26...50
        case .platinum: return 51...75
        case .diamond: return 76...100
        case .mythic: return 101...Int.max
        }
    }
    
    var miningMultiplierRange: ClosedRange<Double> {
        switch self {
        case .bronze: return 1.0...1.2
        case .silver: return 1.3...1.8
        case .gold: return 1.9...2.5
        case .platinum: return 2.6...3.2
        case .diamond: return 3.3...4.0
        case .mythic: return 4.1...5.0
        }
    }
    
    var dailyFINCapRange: ClosedRange<Double> {
        switch self {
        case .bronze: return 0.5...2.0
        case .silver: return 2.0...4.0
        case .gold: return 4.0...6.0
        case .platinum: return 6.0...8.0
        case .diamond: return 8.0...10.0
        case .mythic: return 10.0...15.0
        }
    }
}

// MARK: - XP Activity Model
public struct XPActivity: Codable, Identifiable {
    public let id: String
    public let userId: String
    public let activityType: XPActivityType
    public let platform: SocialPlatform
    public let contentId: String?
    public let qualityScore: Double
    public let viewCount: Int?
    public let engagementCount: Int?
    public let baseXP: Int
    public let finalXP: Int
    public let multipliers: XPMultipliers
    public let timestamp: Date
    public let isVerified: Bool
    public let metadata: [String: String]?
    
    public init(
        id: String = UUID().uuidString,
        userId: String,
        activityType: XPActivityType,
        platform: SocialPlatform,
        contentId: String? = nil,
        qualityScore: Double = 1.0,
        viewCount: Int? = nil,
        engagementCount: Int? = nil,
        metadata: [String: String]? = nil
    ) {
        self.id = id
        self.userId = userId
        self.activityType = activityType
        self.platform = platform
        self.contentId = contentId
        self.qualityScore = max(0.5, min(2.0, qualityScore))
        self.viewCount = viewCount
        self.engagementCount = engagementCount
        self.baseXP = activityType.baseXP
        self.multipliers = XPMultipliers()
        self.finalXP = 0 // Will be calculated
        self.timestamp = Date()
        self.isVerified = false
        self.metadata = metadata
    }
}

// MARK: - XP Multipliers
public struct XPMultipliers: Codable {
    public let platformMultiplier: Double
    public let qualityMultiplier: Double
    public let streakMultiplier: Double
    public let levelProgressionMultiplier: Double
    public let cardMultiplier: Double
    public let viralMultiplier: Double
    
    public init(
        platformMultiplier: Double = 1.0,
        qualityMultiplier: Double = 1.0,
        streakMultiplier: Double = 1.0,
        levelProgressionMultiplier: Double = 1.0,
        cardMultiplier: Double = 1.0,
        viralMultiplier: Double = 1.0
    ) {
        self.platformMultiplier = platformMultiplier
        self.qualityMultiplier = qualityMultiplier
        self.streakMultiplier = streakMultiplier
        self.levelProgressionMultiplier = levelProgressionMultiplier
        self.cardMultiplier = cardMultiplier
        self.viralMultiplier = viralMultiplier
    }
}

// MARK: - User XP Profile
public struct UserXPProfile: Codable {
    public let userId: String
    public let totalXP: Int
    public let currentLevel: Int
    public let currentLevelXP: Int
    public let nextLevelXP: Int
    public let progressToNextLevel: Double
    public let tier: XPTier
    public let badge: String
    public let streakDays: Int
    public let lastActivityDate: Date
    public let todayActivities: [XPActivityType: Int]
    public let miningMultiplier: Double
    public let specialUnlocks: [String]
    public let achievements: [String]
    
    public var isStreakActive: Bool {
        Calendar.current.isDateInToday(lastActivityDate) ||
        Calendar.current.isDateInYesterday(lastActivityDate)
    }
}

// MARK: - XP Service Errors
public enum XPServiceError: LocalizedError {
    case invalidActivity
    case dailyLimitExceeded(XPActivityType, Int)
    case userNotFound
    case networkError(Error)
    case invalidQualityScore
    case rateLimitExceeded
    case authenticationRequired
    case serverError(String)
    
    public var errorDescription: String? {
        switch self {
        case .invalidActivity:
            return "Invalid XP activity type"
        case .dailyLimitExceeded(let type, let limit):
            return "Daily limit exceeded for \(type.rawValue): \(limit)"
        case .userNotFound:
            return "User profile not found"
        case .networkError(let error):
            return "Network error: \(error.localizedDescription)"
        case .invalidQualityScore:
            return "Quality score must be between 0.5 and 2.0"
        case .rateLimitExceeded:
            return "Rate limit exceeded. Please try again later"
        case .authenticationRequired:
            return "Authentication required"
        case .serverError(let message):
            return "Server error: \(message)"
        }
    }
}

// MARK: - XP Service Protocol
public protocol XPServiceProtocol {
    func submitActivity(_ activity: XPActivity) async throws -> XPActivity
    func getUserProfile(_ userId: String) async throws -> UserXPProfile
    func getDailyActivities(_ userId: String) async throws -> [XPActivity]
    func getLeaderboard(limit: Int) async throws -> [UserXPProfile]
    func calculateXP(for activity: XPActivity, userProfile: UserXPProfile) -> Int
    func checkDailyLimits(_ userId: String, activityType: XPActivityType) async throws -> Bool
}

// MARK: - Main XP Service Implementation
@MainActor
public final class XPService: ObservableObject, XPServiceProtocol {
    
    // MARK: - Published Properties
    @Published public private(set) var userProfile: UserXPProfile?
    @Published public private(set) var todayActivities: [XPActivity] = []
    @Published public private(set) var isLoading = false
    @Published public private(set) var error: XPServiceError?
    
    // MARK: - Private Properties
    private let networkManager: NetworkManager
    private let authService: AuthService
    private let cacheManager: CacheManager
    private let analyticsService: AnalyticsService
    private var cancellables = Set<AnyCancellable>()
    
    // MARK: - Constants
    private struct Constants {
        static let baseURL = "https://api.finova.network/v1/xp"
        static let cacheExpiry: TimeInterval = 300 // 5 minutes
        static let maxRetries = 3
        static let rateLimitWindow: TimeInterval = 60
        static let maxRequestsPerMinute = 100
    }
    
    // MARK: - Rate Limiting
    private var requestTimestamps: [Date] = []
    private let rateLimitQueue = DispatchQueue(label: "com.finova.ratelimit", qos: .utility)
    
    // MARK: - Initialization
    public init(
        networkManager: NetworkManager = .shared,
        authService: AuthService = .shared,
        cacheManager: CacheManager = .shared,
        analyticsService: AnalyticsService = .shared
    ) {
        self.networkManager = networkManager
        self.authService = authService
        self.cacheManager = cacheManager
        self.analyticsService = analyticsService
        
        setupBindings()
    }
    
    // MARK: - Setup
    private func setupBindings() {
        authService.$currentUser
            .compactMap { $0?.id }
            .removeDuplicates()
            .sink { [weak self] userId in
                Task {
                    await self?.loadUserProfile(userId)
                }
            }
            .store(in: &cancellables)
    }
    
    // MARK: - Public Methods
    
    /// Submit XP activity with comprehensive validation and calculation
    public func submitActivity(_ activity: XPActivity) async throws -> XPActivity {
        guard let currentUser = authService.currentUser else {
            throw XPServiceError.authenticationRequired
        }
        
        try await checkRateLimit()
        
        // Validate activity
        try validateActivity(activity)
        
        // Check daily limits
        let canSubmit = try await checkDailyLimits(currentUser.id, activityType: activity.activityType)
        if !canSubmit {
            if let limit = activity.activityType.dailyLimit {
                throw XPServiceError.dailyLimitExceeded(activity.activityType, limit)
            }
        }
        
        // Get current user profile for calculations
        let profile = try await getUserProfile(currentUser.id)
        
        // Calculate final XP with all multipliers
        let finalActivity = calculateActivityXP(activity, userProfile: profile)
        
        // Submit to server
        let submittedActivity = try await submitActivityToServer(finalActivity)
        
        // Update local cache
        await updateLocalState(with: submittedActivity)
        
        // Track analytics
        trackActivityAnalytics(submittedActivity)
        
        return submittedActivity
    }
    
    /// Get comprehensive user XP profile
    public func getUserProfile(_ userId: String) async throws -> UserXPProfile {
        // Check cache first
        if let cachedProfile = cacheManager.getUserXPProfile(userId),
           !isCacheExpired(cachedProfile) {
            return cachedProfile
        }
        
        let profile = try await fetchUserProfileFromServer(userId)
        cacheManager.cacheUserXPProfile(profile)
        
        if userId == authService.currentUser?.id {
            userProfile = profile
        }
        
        return profile
    }
    
    /// Get today's activities for user
    public func getDailyActivities(_ userId: String) async throws -> [XPActivity] {
        let cacheKey = "daily_activities_\(userId)_\(todayDateString())"
        
        if let cached: [XPActivity] = cacheManager.getCachedData(for: cacheKey) {
            return cached
        }
        
        let activities = try await fetchDailyActivitiesFromServer(userId)
        cacheManager.cacheData(activities, for: cacheKey, expiry: Constants.cacheExpiry)
        
        if userId == authService.currentUser?.id {
            todayActivities = activities
        }
        
        return activities
    }
    
    /// Get XP leaderboard
    public func getLeaderboard(limit: Int = 100) async throws -> [UserXPProfile] {
        let cacheKey = "leaderboard_\(limit)"
        
        if let cached: [UserXPProfile] = cacheManager.getCachedData(for: cacheKey) {
            return cached
        }
        
        let leaderboard = try await fetchLeaderboardFromServer(limit: limit)
        cacheManager.cacheData(leaderboard, for: cacheKey, expiry: Constants.cacheExpiry)
        
        return leaderboard
    }
    
    /// Calculate XP for activity with all multipliers (Whitepaper Formula)
    public func calculateXP(for activity: XPActivity, userProfile: UserXPProfile) -> Int {
        let baseXP = Double(activity.activityType.baseXP)
        let platformMultiplier = activity.platform.multiplier
        let qualityScore = activity.qualityScore
        let streakBonus = calculateStreakBonus(userProfile.streakDays)
        let levelProgression = calculateLevelProgression(userProfile.currentLevel)
        let cardMultiplier = calculateActiveCardMultiplier(for: userProfile.userId)
        let viralMultiplier = calculateViralMultiplier(activity)
        
        let finalXP = baseXP *
                     platformMultiplier *
                     qualityScore *
                     streakBonus *
                     levelProgression *
                     cardMultiplier *
                     viralMultiplier
        
        return max(1, Int(finalXP.rounded()))
    }
    
    /// Check if user can perform activity (daily limits)
    public func checkDailyLimits(_ userId: String, activityType: XPActivityType) async throws -> Bool {
        guard let dailyLimit = activityType.dailyLimit else { return true }
        
        let todayActivities = try await getDailyActivities(userId)
        let count = todayActivities.filter { $0.activityType == activityType }.count
        
        return count < dailyLimit
    }
    
    // MARK: - Private Calculation Methods
    
    private func calculateActivityXP(_ activity: XPActivity, userProfile: UserXPProfile) -> XPActivity {
        let finalXP = calculateXP(for: activity, userProfile: userProfile)
        
        let multipliers = XPMultipliers(
            platformMultiplier: activity.platform.multiplier,
            qualityMultiplier: activity.qualityScore,
            streakMultiplier: calculateStreakBonus(userProfile.streakDays),
            levelProgressionMultiplier: calculateLevelProgression(userProfile.currentLevel),
            cardMultiplier: calculateActiveCardMultiplier(for: userProfile.userId),
            viralMultiplier: calculateViralMultiplier(activity)
        )
        
        var updatedActivity = activity
        updatedActivity = XPActivity(
            id: activity.id,
            userId: activity.userId,
            activityType: activity.activityType,
            platform: activity.platform,
            contentId: activity.contentId,
            qualityScore: activity.qualityScore,
            viewCount: activity.viewCount,
            engagementCount: activity.engagementCount,
            metadata: activity.metadata
        )
        
        return updatedActivity
    }
    
    /// Calculate streak bonus multiplier (Whitepaper: 1.0x - 3.0x)
    private func calculateStreakBonus(_ streakDays: Int) -> Double {
        let maxStreak = 30.0
        let maxBonus = 3.0
        let normalizedStreak = min(Double(streakDays), maxStreak) / maxStreak
        return 1.0 + (normalizedStreak * (maxBonus - 1.0))
    }
    
    /// Calculate level progression multiplier (Whitepaper: e^(-0.01 × Current_Level))
    private func calculateLevelProgression(_ level: Int) -> Double {
        return exp(-0.01 * Double(level))
    }
    
    /// Calculate active card multiplier
    private func calculateActiveCardMultiplier(for userId: String) -> Double {
        // This would integrate with NFTService to check active XP boost cards
        // For now, return base multiplier
        return 1.0
    }
    
    /// Calculate viral content multiplier
    private func calculateViralMultiplier(_ activity: XPActivity) -> Double {
        guard let viewCount = activity.viewCount else { return 1.0 }
        
        switch viewCount {
        case 1000..<5000: return 1.5
        case 5000..<10000: return 2.0
        case 10000..<50000: return 2.5
        case 50000...: return 3.0
        default: return 1.0
        }
    }
    
    // MARK: - Network Methods
    
    private func submitActivityToServer(_ activity: XPActivity) async throws -> XPActivity {
        isLoading = true
        defer { isLoading = false }
        
        let endpoint = "\(Constants.baseURL)/activities"
        let request = try await networkManager.request(
            url: endpoint,
            method: .POST,
            body: activity,
            headers: await authService.getAuthHeaders()
        )
        
        return try await networkManager.perform(request)
    }
    
    private func fetchUserProfileFromServer(_ userId: String) async throws -> UserXPProfile {
        isLoading = true
        defer { isLoading = false }
        
        let endpoint = "\(Constants.baseURL)/users/\(userId)/profile"
        let request = try await networkManager.request(
            url: endpoint,
            method: .GET,
            headers: await authService.getAuthHeaders()
        )
        
        return try await networkManager.perform(request)
    }
    
    private func fetchDailyActivitiesFromServer(_ userId: String) async throws -> [XPActivity] {
        let endpoint = "\(Constants.baseURL)/users/\(userId)/activities/today"
        let request = try await networkManager.request(
            url: endpoint,
            method: .GET,
            headers: await authService.getAuthHeaders()
        )
        
        return try await networkManager.perform(request)
    }
    
    private func fetchLeaderboardFromServer(limit: Int) async throws -> [UserXPProfile] {
        let endpoint = "\(Constants.baseURL)/leaderboard?limit=\(limit)"
        let request = try await networkManager.request(
            url: endpoint,
            method: .GET,
            headers: await authService.getAuthHeaders()
        )
        
        return try await networkManager.perform(request)
    }
    
    // MARK: - Validation Methods
    
    private func validateActivity(_ activity: XPActivity) throws {
        guard activity.qualityScore >= 0.5 && activity.qualityScore <= 2.0 else {
            throw XPServiceError.invalidQualityScore
        }
        
        guard !activity.userId.isEmpty else {
            throw XPServiceError.invalidActivity
        }
        
        // Additional validation rules
        if activity.activityType == .viralContent {
            guard let viewCount = activity.viewCount, viewCount >= 1000 else {
                throw XPServiceError.invalidActivity
            }
        }
    }
    
    // MARK: - Rate Limiting
    
    private func checkRateLimit() async throws {
        try await rateLimitQueue.asyncExecute {
            let now = Date()
            let windowStart = now.addingTimeInterval(-Constants.rateLimitWindow)
            
            // Remove old timestamps
            self.requestTimestamps = self.requestTimestamps.filter { $0 > windowStart }
            
            // Check if we've exceeded the limit
            if self.requestTimestamps.count >= Constants.maxRequestsPerMinute {
                throw XPServiceError.rateLimitExceeded
            }
            
            // Add current timestamp
            self.requestTimestamps.append(now)
        }
    }
    
    // MARK: - Helper Methods
    
    private func loadUserProfile(_ userId: String) async {
        do {
            userProfile = try await getUserProfile(userId)
            todayActivities = try await getDailyActivities(userId)
        } catch {
            self.error = error as? XPServiceError ?? .networkError(error)
        }
    }
    
    private func updateLocalState(with activity: XPActivity) async {
        // Update today's activities
        if Calendar.current.isDateInToday(activity.timestamp) {
            todayActivities.append(activity)
        }
        
        // Refresh user profile to get updated XP totals
        if let userId = authService.currentUser?.id, userId == activity.userId {
            do {
                userProfile = try await getUserProfile(userId)
            } catch {
                self.error = error as? XPServiceError ?? .networkError(error)
            }
        }
    }
    
    private func trackActivityAnalytics(_ activity: XPActivity) {
        analyticsService.track("xp_activity_submitted", parameters: [
            "activity_type": activity.activityType.rawValue,
            "platform": activity.platform.rawValue,
            "base_xp": activity.baseXP,
            "final_xp": activity.finalXP,
            "quality_score": activity.qualityScore
        ])
    }
    
    private func isCacheExpired(_ profile: UserXPProfile) -> Bool {
        // Simple cache expiry logic - in production, you'd store cache timestamps
        return false
    }
    
    private func todayDateString() -> String {
        let formatter = DateFormatter()
        formatter.dateFormat = "yyyy-MM-dd"
        return formatter.string(from: Date())
    }
}

// MARK: - Extensions

extension DispatchQueue {
    func asyncExecute<T>(_ work: @escaping () throws -> T) async throws -> T {
        return try await withCheckedThrowingContinuation { continuation in
            self.async {
                do {
                    let result = try work()
                    continuation.resume(returning: result)
                } catch {
                    continuation.resume(throwing: error)
                }
            }
        }
    }
}

// MARK: - Convenience Extensions

extension XPService {
    
    /// Quick submit for common activities
    public func submitPost(platform: SocialPlatform, contentId: String? = nil, viewCount: Int? = nil) async throws {
        guard let userId = authService.currentUser?.id else {
            throw XPServiceError.authenticationRequired
        }
        
        let activityType: XPActivityType = viewCount != nil && viewCount! >= 1000 ? .viralContent : .originalPost
        
        let activity = XPActivity(
            userId: userId,
            activityType: activityType,
            platform: platform,
            contentId: contentId,
            viewCount: viewCount
        )
        
        _ = try await submitActivity(activity)
    }
    
    /// Quick daily login
    public func recordDailyLogin() async throws {
        guard let userId = authService.currentUser?.id else {
            throw XPServiceError.authenticationRequired
        }
        
        let activity = XPActivity(
            userId: userId,
            activityType: .dailyLogin,
            platform: .app
        )
        
        _ = try await submitActivity(activity)
    }
}
