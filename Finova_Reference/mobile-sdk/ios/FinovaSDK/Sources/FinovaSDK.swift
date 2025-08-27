// finova-net/finova/mobile-sdk/ios/FinovaSDK/Sources/FinovaSDK.swift

import Foundation
import Combine
import CryptoKit
import LocalAuthentication
import Network

// MARK: - Core SDK Class
@objc public class FinovaSDK: NSObject {
    public static let shared = FinovaSDK()
    
    // MARK: - Properties
    private let client: FinovaClient
    private let walletConnector: WalletConnector
    private let transactionManager: TransactionManager
    private let miningService: MiningService
    private let xpService: XPService
    private let referralService: ReferralService
    private let nftService: NFTService
    
    private var cancellables = Set<AnyCancellable>()
    private let networkMonitor = NWPathMonitor()
    
    // Configuration
    public private(set) var config: FinovaConfig?
    public private(set) var isInitialized = false
    
    // MARK: - Initialization
    private override init() {
        self.client = FinovaClient()
        self.walletConnector = WalletConnector()
        self.transactionManager = TransactionManager()
        self.miningService = MiningService()
        self.xpService = XPService()
        self.referralService = ReferralService()
        self.nftService = NFTService()
        
        super.init()
        setupNetworkMonitoring()
    }
    
    // MARK: - Public Methods
    public func initialize(
        with config: FinovaConfig,
        completion: @escaping (Result<Void, FinovaError>) -> Void
    ) {
        guard !isInitialized else {
            completion(.failure(.alreadyInitialized))
            return
        }
        
        self.config = config
        
        // Initialize all services
        Task {
            do {
                try await client.configure(with: config)
                try await setupServices()
                
                await MainActor.run {
                    self.isInitialized = true
                    completion(.success(()))
                }
            } catch {
                await MainActor.run {
                    completion(.failure(.initializationFailed(error)))
                }
            }
        }
    }
    
    // MARK: - Authentication
    public func authenticateUser(
        credentials: UserCredentials,
        completion: @escaping (Result<User, FinovaError>) -> Void
    ) {
        guard isInitialized else {
            completion(.failure(.notInitialized))
            return
        }
        
        Task {
            do {
                let user = try await client.authenticate(credentials: credentials)
                try await walletConnector.connectWallet(for: user)
                
                await MainActor.run {
                    completion(.success(user))
                }
            } catch {
                await MainActor.run {
                    completion(.failure(.authenticationFailed(error)))
                }
            }
        }
    }
    
    // MARK: - Biometric Authentication
    public func authenticateWithBiometrics(
        completion: @escaping (Result<User, FinovaError>) -> Void
    ) {
        let context = LAContext()
        var error: NSError?
        
        guard context.canEvaluatePolicy(.deviceOwnerAuthenticationWithBiometrics, error: &error) else {
            completion(.failure(.biometricNotAvailable))
            return
        }
        
        context.evaluatePolicy(
            .deviceOwnerAuthenticationWithBiometrics,
            localizedReason: "Authenticate to access Finova Network"
        ) { [weak self] success, error in
            DispatchQueue.main.async {
                if success {
                    self?.loadStoredUser { result in
                        completion(result)
                    }
                } else {
                    completion(.failure(.biometricAuthenticationFailed))
                }
            }
        }
    }
    
    // MARK: - Mining Operations
    public func startMining(completion: @escaping (Result<MiningSession, FinovaError>) -> Void) {
        guard isInitialized else {
            completion(.failure(.notInitialized))
            return
        }
        
        miningService.startMining { result in
            DispatchQueue.main.async {
                completion(result)
            }
        }
    }
    
    public func stopMining(completion: @escaping (Result<MiningRewards, FinovaError>) -> Void) {
        miningService.stopMining { result in
            DispatchQueue.main.async {
                completion(result)
            }
        }
    }
    
    public func getMiningStatus(completion: @escaping (Result<MiningStatus, FinovaError>) -> Void) {
        miningService.getStatus { result in
            DispatchQueue.main.async {
                completion(result)
            }
        }
    }
    
    // MARK: - XP System
    public func gainXP(
        activity: XPActivity,
        completion: @escaping (Result<XPResult, FinovaError>) -> Void
    ) {
        xpService.gainXP(activity: activity) { result in
            DispatchQueue.main.async {
                completion(result)
            }
        }
    }
    
    public func getXPLevel(completion: @escaping (Result<XPLevel, FinovaError>) -> Void) {
        xpService.getCurrentLevel { result in
            DispatchQueue.main.async {
                completion(result)
            }
        }
    }
    
    // MARK: - Referral System
    public func generateReferralCode(completion: @escaping (Result<String, FinovaError>) -> Void) {
        referralService.generateReferralCode { result in
            DispatchQueue.main.async {
                completion(result)
            }
        }
    }
    
    public func useReferralCode(
        _ code: String,
        completion: @escaping (Result<ReferralResult, FinovaError>) -> Void
    ) {
        referralService.useReferralCode(code) { result in
            DispatchQueue.main.async {
                completion(result)
            }
        }
    }
    
    public func getReferralNetwork(completion: @escaping (Result<ReferralNetwork, FinovaError>) -> Void) {
        referralService.getNetwork { result in
            DispatchQueue.main.async {
                completion(result)
            }
        }
    }
    
    // MARK: - NFT Operations
    public func getNFTCollection(completion: @escaping (Result<[NFTItem], FinovaError>) -> Void) {
        nftService.getUserNFTs { result in
            DispatchQueue.main.async {
                completion(result)
            }
        }
    }
    
    public func useSpecialCard(
        cardId: String,
        completion: @escaping (Result<CardUsageResult, FinovaError>) -> Void
    ) {
        nftService.useSpecialCard(cardId: cardId) { result in
            DispatchQueue.main.async {
                completion(result)
            }
        }
    }
    
    // MARK: - Staking Operations
    public func stakeTokens(
        amount: Double,
        completion: @escaping (Result<StakingResult, FinovaError>) -> Void
    ) {
        Task {
            do {
                let result = try await transactionManager.stakeTokens(amount: amount)
                await MainActor.run {
                    completion(.success(result))
                }
            } catch {
                await MainActor.run {
                    completion(.failure(.stakingFailed(error)))
                }
            }
        }
    }
    
    public func unstakeTokens(
        amount: Double,
        completion: @escaping (Result<UnstakingResult, FinovaError>) -> Void
    ) {
        Task {
            do {
                let result = try await transactionManager.unstakeTokens(amount: amount)
                await MainActor.run {
                    completion(.success(result))
                }
            } catch {
                await MainActor.run {
                    completion(.failure(.unstakingFailed(error)))
                }
            }
        }
    }
    
    // MARK: - Wallet Operations
    public func getWalletBalance(completion: @escaping (Result<WalletBalance, FinovaError>) -> Void) {
        walletConnector.getBalance { result in
            DispatchQueue.main.async {
                completion(result)
            }
        }
    }
    
    public func sendTokens(
        to address: String,
        amount: Double,
        completion: @escaping (Result<TransactionResult, FinovaError>) -> Void
    ) {
        Task {
            do {
                let result = try await transactionManager.sendTokens(to: address, amount: amount)
                await MainActor.run {
                    completion(.success(result))
                }
            } catch {
                await MainActor.run {
                    completion(.failure(.transactionFailed(error)))
                }
            }
        }
    }
    
    // MARK: - Social Media Integration
    public func connectSocialPlatform(
        platform: SocialPlatform,
        credentials: SocialCredentials,
        completion: @escaping (Result<Void, FinovaError>) -> Void
    ) {
        Task {
            do {
                try await client.connectSocialPlatform(platform: platform, credentials: credentials)
                await MainActor.run {
                    completion(.success(()))
                }
            } catch {
                await MainActor.run {
                    completion(.failure(.socialConnectionFailed(error)))
                }
            }
        }
    }
    
    public func syncSocialActivity(completion: @escaping (Result<SocialSyncResult, FinovaError>) -> Void) {
        Task {
            do {
                let result = try await client.syncSocialActivity()
                await MainActor.run {
                    completion(.success(result))
                }
            } catch {
                await MainActor.run {
                    completion(.failure(.socialSyncFailed(error)))
                }
            }
        }
    }
    
    // MARK: - Real-time Updates
    public func subscribeToUpdates() -> AnyPublisher<FinovaUpdate, Never> {
        client.updatesPublisher
            .receive(on: DispatchQueue.main)
            .eraseToAnyPublisher()
    }
    
    // MARK: - Private Methods
    private func setupServices() async throws {
        try await miningService.configure(client: client)
        try await xpService.configure(client: client)
        try await referralService.configure(client: client)
        try await nftService.configure(client: client)
        try await transactionManager.configure(client: client, walletConnector: walletConnector)
    }
    
    private func setupNetworkMonitoring() {
        networkMonitor.pathUpdateHandler = { [weak self] path in
            if path.status == .satisfied {
                self?.handleNetworkReconnection()
            }
        }
        let queue = DispatchQueue(label: "NetworkMonitor")
        networkMonitor.start(queue: queue)
    }
    
    private func handleNetworkReconnection() {
        // Reconnect and sync when network is available
        Task {
            try? await client.reconnect()
        }
    }
    
    private func loadStoredUser(completion: @escaping (Result<User, FinovaError>) -> Void) {
        // Implementation for loading stored user credentials
        Task {
            do {
                let user = try await client.loadStoredUser()
                completion(.success(user))
            } catch {
                completion(.failure(.userNotFound))
            }
        }
    }
}

// MARK: - Configuration
public struct FinovaConfig {
    public let apiBaseURL: String
    public let blockchainRPC: String
    public let environment: Environment
    public let apiKey: String
    public let enableBiometric: Bool
    public let enableNotifications: Bool
    
    public enum Environment {
        case development
        case staging
        case production
    }
    
    public init(
        apiBaseURL: String,
        blockchainRPC: String,
        environment: Environment,
        apiKey: String,
        enableBiometric: Bool = true,
        enableNotifications: Bool = true
    ) {
        self.apiBaseURL = apiBaseURL
        self.blockchainRPC = blockchainRPC
        self.environment = environment
        self.apiKey = apiKey
        self.enableBiometric = enableBiometric
        self.enableNotifications = enableNotifications
    }
}

// MARK: - Error Types
public enum FinovaError: Error, LocalizedError {
    case notInitialized
    case alreadyInitialized
    case initializationFailed(Error)
    case authenticationFailed(Error)
    case biometricNotAvailable
    case biometricAuthenticationFailed
    case miningFailed(Error)
    case stakingFailed(Error)
    case unstakingFailed(Error)
    case transactionFailed(Error)
    case socialConnectionFailed(Error)
    case socialSyncFailed(Error)
    case networkError(Error)
    case userNotFound
    case invalidInput
    case insufficientBalance
    case rateLimitExceeded
    case serverError(Int, String)
    
    public var errorDescription: String? {
        switch self {
        case .notInitialized:
            return "SDK not initialized. Call initialize() first."
        case .alreadyInitialized:
            return "SDK already initialized."
        case .initializationFailed(let error):
            return "Initialization failed: \(error.localizedDescription)"
        case .authenticationFailed(let error):
            return "Authentication failed: \(error.localizedDescription)"
        case .biometricNotAvailable:
            return "Biometric authentication not available on this device."
        case .biometricAuthenticationFailed:
            return "Biometric authentication failed."
        case .miningFailed(let error):
            return "Mining operation failed: \(error.localizedDescription)"
        case .stakingFailed(let error):
            return "Staking failed: \(error.localizedDescription)"
        case .unstakingFailed(let error):
            return "Unstaking failed: \(error.localizedDescription)"
        case .transactionFailed(let error):
            return "Transaction failed: \(error.localizedDescription)"
        case .socialConnectionFailed(let error):
            return "Social platform connection failed: \(error.localizedDescription)"
        case .socialSyncFailed(let error):
            return "Social activity sync failed: \(error.localizedDescription)"
        case .networkError(let error):
            return "Network error: \(error.localizedDescription)"
        case .userNotFound:
            return "User not found."
        case .invalidInput:
            return "Invalid input provided."
        case .insufficientBalance:
            return "Insufficient balance."
        case .rateLimitExceeded:
            return "Rate limit exceeded. Please try again later."
        case .serverError(let code, let message):
            return "Server error (\(code)): \(message)"
        }
    }
}

// MARK: - Data Models
public struct User: Codable {
    public let id: String
    public let walletAddress: String
    public let username: String
    public let email: String
    public let level: Int
    public let xp: Int
    public let referralPoints: Int
    public let isKYCVerified: Bool
    public let createdAt: Date
    public let lastActiveAt: Date
}

public struct UserCredentials {
    public let username: String
    public let password: String
    public let referralCode: String?
    
    public init(username: String, password: String, referralCode: String? = nil) {
        self.username = username
        self.password = password
        self.referralCode = referralCode
    }
}

public struct MiningSession: Codable {
    public let sessionId: String
    public let startTime: Date
    public let currentRate: Double
    public let bonusMultiplier: Double
    public let estimatedDailyReward: Double
}

public struct MiningStatus: Codable {
    public let isActive: Bool
    public let currentSession: MiningSession?
    public let totalEarned: Double
    public let dailyEarned: Double
    public let nextRewardTime: Date?
}

public struct MiningRewards: Codable {
    public let sessionId: String
    public let finEarned: Double
    public let xpGained: Int
    public let rpGained: Int
    public let duration: TimeInterval
}

public struct XPActivity {
    public let type: ActivityType
    public let platform: SocialPlatform
    public let content: String
    public let metadata: [String: Any]
    
    public enum ActivityType: String, CaseIterable {
        case post = "post"
        case comment = "comment"
        case like = "like"
        case share = "share"
        case follow = "follow"
        case story = "story"
        case video = "video"
        case login = "login"
        case quest = "quest"
    }
    
    public init(type: ActivityType, platform: SocialPlatform, content: String, metadata: [String: Any] = [:]) {
        self.type = type
        self.platform = platform
        self.content = content
        self.metadata = metadata
    }
}

public struct XPResult: Codable {
    public let xpGained: Int
    public let totalXP: Int
    public let newLevel: Int
    public let leveledUp: Bool
    public let qualityScore: Double
    public let multipliers: XPMultipliers
}

public struct XPMultipliers: Codable {
    public let platform: Double
    public let quality: Double
    public let streak: Double
    public let level: Double
}

public struct XPLevel: Codable {
    public let level: Int
    public let currentXP: Int
    public let xpToNext: Int
    public let tier: String
    public let miningMultiplier: Double
    public let dailyCapIncrease: Double
}

public struct ReferralResult: Codable {
    public let success: Bool
    public let rpGained: Int
    public let bonusUnlocked: String?
    public let referrerReward: Int
}

public struct ReferralNetwork: Codable {
    public let totalReferrals: Int
    public let activeReferrals: Int
    public let totalRP: Int
    public let tier: String
    public let networkMultiplier: Double
    public let levels: [ReferralLevel]
}

public struct ReferralLevel: Codable {
    public let level: Int
    public let count: Int
    public let activeCount: Int
    public let contribution: Double
}

public struct NFTItem: Codable {
    public let id: String
    public let name: String
    public let type: NFTType
    public let rarity: Rarity
    public let effect: String
    public let duration: TimeInterval?
    public let isUsable: Bool
    public let imageURL: String
    
    public enum NFTType: String, CaseIterable {
        case miningBoost = "mining_boost"
        case xpAccelerator = "xp_accelerator"
        case referralPower = "referral_power"
        case profileBadge = "profile_badge"
        case achievement = "achievement"
    }
    
    public enum Rarity: String, CaseIterable {
        case common = "common"
        case uncommon = "uncommon"
        case rare = "rare"
        case epic = "epic"
        case legendary = "legendary"
    }
}

public struct CardUsageResult: Codable {
    public let success: Bool
    public let effectApplied: String
    public let duration: TimeInterval?
    public let newMultipliers: CardMultipliers
}

public struct CardMultipliers: Codable {
    public let mining: Double
    public let xp: Double
    public let referral: Double
    public let expiresAt: Date?
}

public struct WalletBalance: Codable {
    public let fin: Double
    public let sFin: Double
    public let usdFin: Double
    public let sUsdFin: Double
    public let sol: Double
}

public struct StakingResult: Codable {
    public let transactionId: String
    public let amountStaked: Double
    public let newStakingTier: String
    public let newMultiplier: Double
    public let estimatedAPY: Double
}

public struct UnstakingResult: Codable {
    public let transactionId: String
    public let amountUnstaked: Double
    public let penalty: Double
    public let netAmount: Double
    public let unlockTime: Date?
}

public struct TransactionResult: Codable {
    public let transactionId: String
    public let fromAddress: String
    public let toAddress: String
    public let amount: Double
    public let fee: Double
    public let status: TransactionStatus
    public let timestamp: Date
    
    public enum TransactionStatus: String, CaseIterable {
        case pending = "pending"
        case confirmed = "confirmed"
        case failed = "failed"
    }
}

public enum SocialPlatform: String, CaseIterable {
    case instagram = "instagram"
    case tiktok = "tiktok"
    case youtube = "youtube"
    case facebook = "facebook"
    case twitter = "twitter"
    case linkedin = "linkedin"
}

public struct SocialCredentials {
    public let accessToken: String
    public let refreshToken: String?
    public let expiresAt: Date?
    
    public init(accessToken: String, refreshToken: String? = nil, expiresAt: Date? = nil) {
        self.accessToken = accessToken
        self.refreshToken = refreshToken
        self.expiresAt = expiresAt
    }
}

public struct SocialSyncResult: Codable {
    public let activitiesSynced: Int
    public let xpGained: Int
    public let finEarned: Double
    public let newFollowers: Int
    public let viralContent: [String]
}

public enum FinovaUpdate {
    case miningStatusChanged(MiningStatus)
    case xpGained(XPResult)
    case referralReward(ReferralResult)
    case balanceUpdated(WalletBalance)
    case socialActivitySynced(SocialSyncResult)
    case notificationReceived(String)
}

// MARK: - Extensions
extension FinovaSDK {
    // Convenience methods for common operations
    public func quickStart(
        username: String,
        password: String,
        completion: @escaping (Result<User, FinovaError>) -> Void
    ) {
        let credentials = UserCredentials(username: username, password: password)
        authenticateUser(credentials: credentials) { [weak self] result in
            switch result {
            case .success(let user):
                self?.startMining { miningResult in
                    switch miningResult {
                    case .success:
                        completion(.success(user))
                    case .failure(let error):
                        completion(.failure(error))
                    }
                }
            case .failure(let error):
                completion(.failure(error))
            }
        }
    }
    
    public func getDashboardData(completion: @escaping (Result<DashboardData, FinovaError>) -> Void) {
        let group = DispatchGroup()
        var dashboardData = DashboardData()
        var hasError: FinovaError?
        
        group.enter()
        getMiningStatus { result in
            switch result {
            case .success(let status):
                dashboardData.miningStatus = status
            case .failure(let error):
                hasError = error
            }
            group.leave()
        }
        
        group.enter()
        getXPLevel { result in
            switch result {
            case .success(let level):
                dashboardData.xpLevel = level
            case .failure(let error):
                if hasError == nil { hasError = error }
            }
            group.leave()
        }
        
        group.enter()
        getReferralNetwork { result in
            switch result {
            case .success(let network):
                dashboardData.referralNetwork = network
            case .failure(let error):
                if hasError == nil { hasError = error }
            }
            group.leave()
        }
        
        group.enter()
        getWalletBalance { result in
            switch result {
            case .success(let balance):
                dashboardData.walletBalance = balance
            case .failure(let error):
                if hasError == nil { hasError = error }
            }
            group.leave()
        }
        
        group.notify(queue: .main) {
            if let error = hasError {
                completion(.failure(error))
            } else {
                completion(.success(dashboardData))
            }
        }
    }
}

public struct DashboardData {
    public var miningStatus: MiningStatus?
    public var xpLevel: XPLevel?
    public var referralNetwork: ReferralNetwork?
    public var walletBalance: WalletBalance?
    
    public init() {}
}
