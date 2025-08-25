// finova-net/finova/mobile-sdk/ios/FinovaSDK/Sources/Services/ReferralService.swift

import Foundation
import Combine

/// Enterprise-grade Referral Service for Finova Network iOS SDK
/// Implements RP (Referral Points) system with exponential regression and network quality scoring
@available(iOS 13.0, *)
public class ReferralService: ObservableObject {
    
    // MARK: - Properties
    
    private let apiClient: FinovaAPIClient
    private let cryptoService: CryptoService
    private var cancellables = Set<AnyCancellable>()
    
    @Published public private(set) var currentRP: Int = 0
    @Published public private(set) var referralTier: ReferralTier = .explorer
    @Published public private(set) var networkStats: NetworkStats?
    @Published public private(set) var isLoading = false
    
    // MARK: - Enums
    
    public enum ReferralTier: String, CaseIterable {
        case explorer = "Explorer"
        case connector = "Connector"
        case influencer = "Influencer"
        case leader = "Leader"
        case ambassador = "Ambassador"
        
        var rpRange: ClosedRange<Int> {
            switch self {
            case .explorer: return 0...999
            case .connector: return 1000...4999
            case .influencer: return 5000...14999
            case .leader: return 15000...49999
            case .ambassador: return 50000...Int.max
            }
        }
        
        var miningBonus: Double {
            switch self {
            case .explorer: return 0.0
            case .connector: return 0.2
            case .influencer: return 0.5
            case .leader: return 1.0
            case .ambassador: return 2.0
            }
        }
        
        var referralBonus: (l1: Double, l2: Double, l3: Double) {
            switch self {
            case .explorer: return (0.10, 0.0, 0.0)
            case .connector: return (0.15, 0.05, 0.0)
            case .influencer: return (0.20, 0.08, 0.03)
            case .leader: return (0.25, 0.10, 0.05)
            case .ambassador: return (0.30, 0.15, 0.08)
            }
        }
        
        var networkCap: Int {
            switch self {
            case .explorer: return 10
            case .connector: return 25
            case .influencer: return 50
            case .leader: return 100
            case .ambassador: return Int.max
            }
        }
    }
    
    // MARK: - Models
    
    public struct NetworkStats: Codable {
        public let totalReferrals: Int
        public let activeReferrals: Int
        public let l2NetworkSize: Int
        public let l3NetworkSize: Int
        public let networkQualityScore: Double
        public let monthlyRPEarned: Int
        public let lifetimeRPEarned: Int
        
        var networkRetentionRate: Double {
            guard totalReferrals > 0 else { return 0.0 }
            return Double(activeReferrals) / Double(totalReferrals)
        }
    }
    
    public struct ReferralReward: Codable {
        public let rpEarned: Int
        public let finEarned: Double
        public let xpBonus: Int
        public let source: String
        public let timestamp: Date
        public let referralLevel: Int
    }
    
    public struct ReferralCode: Codable {
        public let code: String
        public let customCode: String?
        public let usageCount: Int
        public let maxUsage: Int?
        public let isActive: Bool
        public let createdAt: Date
        public let expiresAt: Date?
    }
    
    // MARK: - Initialization
    
    public init(apiClient: FinovaAPIClient, cryptoService: CryptoService) {
        self.apiClient = apiClient
        self.cryptoService = cryptoService
        setupBindings()
    }
    
    // MARK: - Private Setup
    
    private func setupBindings() {
        // Auto-refresh network stats every 5 minutes
        Timer.publish(every: 300, on: .main, in: .common)
            .autoconnect()
            .sink { [weak self] _ in
                Task { await self?.refreshNetworkStats() }
            }
            .store(in: &cancellables)
    }
    
    // MARK: - Public Methods
    
    /// Generate or retrieve user's referral code
    public func getReferralCode() async throws -> ReferralCode {
        isLoading = true
        defer { isLoading = false }
        
        let endpoint = "/api/v1/referral/code"
        return try await apiClient.get(endpoint: endpoint)
    }
    
    /// Create custom referral code (for Connector tier and above)
    public func createCustomReferralCode(_ code: String) async throws -> ReferralCode {
        guard referralTier != .explorer else {
            throw ReferralError.insufficientTier
        }
        
        isLoading = true
        defer { isLoading = false }
        
        let request = ["customCode": code]
        let endpoint = "/api/v1/referral/code/custom"
        return try await apiClient.post(endpoint: endpoint, body: request)
    }
    
    /// Submit referral code during registration
    public func submitReferralCode(_ code: String) async throws -> Bool {
        isLoading = true
        defer { isLoading = false }
        
        let request = ["referralCode": code]
        let endpoint = "/api/v1/referral/submit"
        let response: [String: Bool] = try await apiClient.post(endpoint: endpoint, body: request)
        
        if response["success"] == true {
            await refreshNetworkStats()
            return true
        }
        return false
    }
    
    /// Get referral network tree
    public func getReferralNetwork() async throws -> ReferralNetwork {
        isLoading = true
        defer { isLoading = false }
        
        let endpoint = "/api/v1/referral/network"
        return try await apiClient.get(endpoint: endpoint)
    }
    
    /// Calculate RP value with exponential regression
    public func calculateRPValue(networkStats: NetworkStats) -> Int {
        let directRP = calculateDirectReferralPoints(activeReferrals: networkStats.activeReferrals)
        let networkRP = calculateNetworkPoints(
            l2Size: networkStats.l2NetworkSize, 
            l3Size: networkStats.l3NetworkSize
        )
        let qualityBonus = calculateNetworkQualityBonus(stats: networkStats)
        let regressionFactor = calculateRegressionFactor(stats: networkStats)
        
        return Int((Double(directRP + networkRP) * qualityBonus * regressionFactor))
    }
    
    /// Get referral rewards history
    public func getReferralRewards(limit: Int = 50, offset: Int = 0) async throws -> [ReferralReward] {
        let endpoint = "/api/v1/referral/rewards?limit=\(limit)&offset=\(offset)"
        let response: [String: [ReferralReward]] = try await apiClient.get(endpoint: endpoint)
        return response["rewards"] ?? []
    }
    
    /// Refresh network statistics
    @MainActor
    public func refreshNetworkStats() async {
        do {
            let endpoint = "/api/v1/referral/stats"
            let stats: NetworkStats = try await apiClient.get(endpoint: endpoint)
            
            self.networkStats = stats
            self.currentRP = calculateRPValue(networkStats: stats)
            self.referralTier = determineTier(rp: currentRP)
            
        } catch {
            print("Failed to refresh network stats: \(error)")
        }
    }
    
    /// Calculate mining multiplier based on RP tier
    public func getMiningMultiplier() -> Double {
        return 1.0 + referralTier.miningBonus
    }
    
    /// Get referral bonus percentages for current tier
    public func getReferralBonuses() -> (l1: Double, l2: Double, l3: Double) {
        return referralTier.referralBonus
    }
    
    // MARK: - Private Calculations
    
    private func calculateDirectReferralPoints(activeReferrals: Int) -> Int {
        // Base RP calculation: Active referrals * average activity * retention
        return activeReferrals * 100 // Simplified base calculation
    }
    
    private func calculateNetworkPoints(l2Size: Int, l3Size: Int) -> Int {
        let l2Points = Double(l2Size) * 0.3 * 50 // 30% of L2 activity
        let l3Points = Double(l3Size) * 0.1 * 25 // 10% of L3 activity
        return Int(l2Points + l3Points)
    }
    
    private func calculateNetworkQualityBonus(stats: NetworkStats) -> Double {
        let diversityScore = min(1.0, Double(stats.totalReferrals) / 100.0)
        let retentionScore = stats.networkRetentionRate
        let qualityScore = stats.networkQualityScore
        
        return 1.0 + (diversityScore * retentionScore * qualityScore * 0.5)
    }
    
    private func calculateRegressionFactor(stats: NetworkStats) -> Double {
        let totalNetworkSize = Double(stats.totalReferrals + stats.l2NetworkSize + stats.l3NetworkSize)
        let qualityScore = stats.networkQualityScore
        
        // Exponential regression: e^(-0.0001 × Total_Network_Size × Network_Quality_Score)
        return exp(-0.0001 * totalNetworkSize * qualityScore)
    }
    
    private func determineTier(rp: Int) -> ReferralTier {
        for tier in ReferralTier.allCases.reversed() {
            if tier.rpRange.contains(rp) {
                return tier
            }
        }
        return .explorer
    }
    
    // MARK: - Validation & Security
    
    private func validateReferralCode(_ code: String) -> Bool {
        // Referral code validation rules
        let codeRegex = "^[A-Z0-9]{6,12}$"
        let predicate = NSPredicate(format: "SELF MATCHES %@", codeRegex)
        return predicate.evaluate(with: code.uppercased())
    }
    
    private func generateSecureReferralCode() -> String {
        let characters = "ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789"
        return String((0..<8).map { _ in characters.randomElement()! })
    }
}

// MARK: - Supporting Models

public struct ReferralNetwork: Codable {
    public let directReferrals: [ReferralUser]
    public let l2Network: [ReferralUser]
    public let l3Network: [ReferralUser]
    public let totalEarnings: ReferralEarnings
}

public struct ReferralUser: Codable, Identifiable {
    public let id: UUID
    public let username: String
    public let level: Int
    public let isActive: Bool
    public let joinedAt: Date
    public let lastActiveAt: Date
    public let totalRP: Int
    public let monthlyRP: Int
}

public struct ReferralEarnings: Codable {
    public let totalRP: Int
    public let totalFIN: Double
    public let monthlyRP: Int
    public let monthlyFIN: Double
    public let lifetimeEarnings: Double
}

// MARK: - Error Handling

public enum ReferralError: LocalizedError {
    case invalidReferralCode
    case referralCodeExpired
    case referralCodeUsageLimitExceeded
    case cannotReferSelf
    case alreadyReferred
    case insufficientTier
    case networkLimitExceeded
    case apiError(String)
    
    public var errorDescription: String? {
        switch self {
        case .invalidReferralCode:
            return "Invalid referral code format"
        case .referralCodeExpired:
            return "Referral code has expired"
        case .referralCodeUsageLimitExceeded:
            return "Referral code usage limit exceeded"
        case .cannotReferSelf:
            return "Cannot use your own referral code"
        case .alreadyReferred:
            return "User already has a referrer"
        case .insufficientTier:
            return "Insufficient referral tier for this action"
        case .networkLimitExceeded:
            return "Network size limit exceeded for current tier"
        case .apiError(let message):
            return "API Error: \(message)"
        }
    }
}

// MARK: - Extensions

extension ReferralService {
    
    /// Calculate potential earnings for user based on network activity
    public func calculatePotentialEarnings(networkActivity: Double) -> ReferralEarnings {
        guard let stats = networkStats else {
            return ReferralEarnings(totalRP: 0, totalFIN: 0, monthlyRP: 0, monthlyFIN: 0, lifetimeEarnings: 0)
        }
        
        let bonuses = getReferralBonuses()
        let miningMultiplier = getMiningMultiplier()
        
        let monthlyRP = Int(networkActivity * Double(stats.activeReferrals) * bonuses.l1 * 30)
        let monthlyFIN = networkActivity * Double(stats.activeReferrals) * bonuses.l1 * miningMultiplier * 0.05 * 30
        
        return ReferralEarnings(
            totalRP: stats.lifetimeRPEarned,
            totalFIN: 0, // Would be calculated from API
            monthlyRP: monthlyRP,
            monthlyFIN: monthlyFIN,
            lifetimeEarnings: 0 // Would be calculated from API
        )
    }
    
    /// Get tier upgrade requirements
    public func getTierUpgradeRequirements() -> (nextTier: ReferralTier?, rpNeeded: Int) {
        let allTiers = ReferralTier.allCases
        guard let currentIndex = allTiers.firstIndex(of: referralTier),
              currentIndex < allTiers.count - 1 else {
            return (nil, 0)
        }
        
        let nextTier = allTiers[currentIndex + 1]
        let rpNeeded = nextTier.rpRange.lowerBound - currentRP
        
        return (nextTier, max(0, rpNeeded))
    }
}
