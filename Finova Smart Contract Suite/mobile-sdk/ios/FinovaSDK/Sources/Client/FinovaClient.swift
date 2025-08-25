// finova-net/finova/mobile-sdk/ios/FinovaSDK/Sources/Client/FinovaClient.swift

import Foundation
import Combine
import CryptoKit
import Network
import LocalAuthentication

// MARK: - Core Types and Models
public struct FinovaConfig {
    let apiBaseURL: String
    let wsBaseURL: String
    let solanaRPC: String
    let environment: Environment
    let apiKey: String
    
    public enum Environment {
        case development, staging, production
    }
}

public struct User {
    let id: String
    let walletAddress: String
    let xpLevel: Int
    let rpTier: RPTier
    let totalFIN: Double
    let stakedFIN: Double
    let miningRate: Double
    let kycStatus: KYCStatus
    let profile: UserProfile
}

public struct UserProfile {
    let username: String
    let email: String
    let avatar: String?
    let bio: String?
    let socialConnections: [SocialConnection]
    let achievements: [Achievement]
}

public struct SocialConnection {
    let platform: SocialPlatform
    let username: String
    let isVerified: Bool
    let followers: Int
    let engagementRate: Double
}

public enum SocialPlatform: String, CaseIterable {
    case instagram, tiktok, youtube, facebook, twitter, linkedin
    
    var multiplier: Double {
        switch self {
        case .tiktok: return 1.3
        case .instagram: return 1.2
        case .youtube: return 1.4
        case .twitter: return 1.2
        case .facebook: return 1.1
        case .linkedin: return 1.0
        }
    }
}

public struct Mining {
    let currentRate: Double
    let totalMined: Double
    let lastClaimTime: Date
    let nextClaimTime: Date
    let multipliers: MiningMultipliers
    let phase: MiningPhase
}

public struct MiningMultipliers {
    let pioneerBonus: Double
    let referralBonus: Double
    let xpBonus: Double
    let rpBonus: Double
    let stakingBonus: Double
    let qualityBonus: Double
}

public enum MiningPhase: String {
    case pioneer = "Pioneer"
    case growth = "Growth" 
    case maturity = "Maturity"
    case stability = "Stability"
}

public struct XPActivity {
    let type: ActivityType
    let platform: SocialPlatform
    let baseXP: Int
    let qualityScore: Double
    let timestamp: Date
    let content: ActivityContent?
}

public enum ActivityType: String {
    case post, comment, like, share, follow, login, quest, viral, referral
}

public struct ActivityContent {
    let text: String?
    let imageURL: String?
    let videoURL: String?
    let hashtags: [String]
    let mentions: [String]
}

public enum RPTier: String, CaseIterable {
    case explorer, connector, influencer, leader, ambassador
    
    var miningBonus: Double {
        switch self {
        case .explorer: return 0.0
        case .connector: return 0.2
        case .influencer: return 0.5
        case .leader: return 1.0
        case .ambassador: return 2.0
        }
    }
    
    var referralBonus: Double {
        switch self {
        case .explorer: return 0.1
        case .connector: return 0.15
        case .influencer: return 0.2
        case .leader: return 0.25
        case .ambassador: return 0.3
        }
    }
}

public struct ReferralNetwork {
    let totalReferrals: Int
    let activeReferrals: Int
    let totalRP: Int
    let tier: RPTier
    let networkSize: NetworkSize
    let qualityScore: Double
}

public struct NetworkSize {
    let level1: Int
    let level2: Int
    let level3: Int
    let totalNetworkValue: Double
}

public struct NFTCard {
    let id: String
    let name: String
    let type: CardType
    let rarity: CardRarity
    let effect: CardEffect
    let duration: TimeInterval?
    let price: Double
    let isActive: Bool
}

public enum CardType: String {
    case mining, xp, referral, special
}

public enum CardRarity: String {
    case common, uncommon, rare, epic, legendary
    
    var multiplier: Double {
        switch self {
        case .common: return 1.0
        case .uncommon: return 1.05
        case .rare: return 1.1
        case .epic: return 1.2
        case .legendary: return 1.35
        }
    }
}

public struct CardEffect {
    let miningBoost: Double?
    let xpBoost: Double?
    let rpBoost: Double?
    let description: String
}

public struct Achievement {
    let id: String
    let name: String
    let description: String
    let xpReward: Int
    let finReward: Double
    let nftReward: String?
    let unlockedAt: Date
}

public enum KYCStatus: String {
    case pending, verified, rejected, notStarted
}

// MARK: - Error Handling
public enum FinovaError: Error, LocalizedError {
    case networkError(String)
    case authenticationFailed
    case invalidCredentials
    case kycRequired
    case insufficientBalance
    case rateLimitExceeded
    case serverError(Int, String)
    case biometricFailed
    case walletConnectionFailed
    case invalidTransaction
    case cardAlreadyActive
    case cardNotOwned
    case miningCooldown
    case invalidReferralCode
    case accountSuspended
    
    public var errorDescription: String? {
        switch self {
        case .networkError(let message): return "Network error: \(message)"
        case .authenticationFailed: return "Authentication failed"
        case .invalidCredentials: return "Invalid credentials"
        case .kycRequired: return "KYC verification required"
        case .insufficientBalance: return "Insufficient balance"
        case .rateLimitExceeded: return "Rate limit exceeded"
        case .serverError(let code, let message): return "Server error \(code): \(message)"
        case .biometricFailed: return "Biometric authentication failed"
        case .walletConnectionFailed: return "Wallet connection failed"
        case .invalidTransaction: return "Invalid transaction"
        case .cardAlreadyActive: return "Card already active"
        case .cardNotOwned: return "Card not owned"
        case .miningCooldown: return "Mining in cooldown period"
        case .invalidReferralCode: return "Invalid referral code"
        case .accountSuspended: return "Account suspended"
        }
    }
}

// MARK: - Main FinovaClient Class
@MainActor
public class FinovaClient: ObservableObject {
    // MARK: - Published Properties
    @Published public private(set) var isAuthenticated = false
    @Published public private(set) var currentUser: User?
    @Published public private(set) var mining: Mining?
    @Published public private(set) var referralNetwork: ReferralNetwork?
    @Published public private(set) var ownedCards: [NFTCard] = []
    @Published public private(set) var isLoading = false
    @Published public private(set) var error: FinovaError?
    
    // MARK: - Private Properties
    private let config: FinovaConfig
    private let session: URLSession
    private let webSocketTask: URLSessionWebSocketTask?
    private var cancellables = Set<AnyCancellable>()
    private let keychain = KeychainManager()
    private let biometric = BiometricManager()
    private let calculator = RewardCalculator()
    private let monitor = NetworkMonitor()
    
    // MARK: - Constants
    private struct Constants {
        static let tokenKey = "finova_auth_token"
        static let userKey = "finova_user_data"
        static let refreshTokenKey = "finova_refresh_token"
        static let biometricKey = "finova_biometric_key"
        static let maxRetries = 3
        static let timeoutInterval: TimeInterval = 30
        static let miningInterval: TimeInterval = 3600 // 1 hour
    }
    
    // MARK: - Initialization
    public init(config: FinovaConfig) {
        self.config = config
        
        let configuration = URLSessionConfiguration.default
        configuration.timeoutIntervalForRequest = Constants.timeoutInterval
        configuration.requestCachePolicy = .reloadIgnoringLocalCacheData
        
        self.session = URLSession(configuration: configuration)
        self.webSocketTask = session.webSocketTask(with: URL(string: config.wsBaseURL)!)
        
        setupNetworkMonitoring()
        loadPersistedState()
        setupMiningTimer()
    }
    
    deinit {
        webSocketTask?.cancel()
        cancellables.removeAll()
    }
    
    // MARK: - Authentication Methods
    
    public func authenticateWithBiometrics() async throws {
        do {
            let isAvailable = await biometric.isBiometricAvailable()
            guard isAvailable else {
                throw FinovaError.biometricFailed
            }
            
            let success = await biometric.authenticate(reason: "Authenticate to access your Finova account")
            guard success else {
                throw FinovaError.biometricFailed
            }
            
            if let token = keychain.get(Constants.tokenKey) {
                try await validateAndSetAuthToken(token)
            } else {
                throw FinovaError.authenticationFailed
            }
        } catch {
            await MainActor.run {
                self.error = error as? FinovaError ?? .biometricFailed
            }
            throw error
        }
    }
    
    public func signIn(email: String, password: String, enableBiometric: Bool = true) async throws {
        isLoading = true
        error = nil
        
        do {
            let request = AuthRequest(email: email, password: password, deviceInfo: getDeviceInfo())
            let response: AuthResponse = try await performRequest(.post, "/auth/signin", body: request)
            
            // Store tokens securely
            keychain.set(response.accessToken, for: Constants.tokenKey)
            keychain.set(response.refreshToken, for: Constants.refreshTokenKey)
            
            if enableBiometric && await biometric.isBiometricAvailable() {
                let biometricKey = generateBiometricKey()
                keychain.set(biometricKey, for: Constants.biometricKey)
            }
            
            // Set authenticated state
            await MainActor.run {
                self.isAuthenticated = true
                self.currentUser = response.user
                self.isLoading = false
            }
            
            // Load initial data
            await loadUserData()
            connectWebSocket()
            
        } catch {
            await MainActor.run {
                self.isLoading = false
                self.error = error as? FinovaError ?? .authenticationFailed
            }
            throw error
        }
    }
    
    public func signUp(email: String, password: String, username: String, referralCode: String? = nil) async throws {
        isLoading = true
        error = nil
        
        do {
            let request = SignUpRequest(
                email: email,
                password: password,
                username: username,
                referralCode: referralCode,
                deviceInfo: getDeviceInfo()
            )
            
            let response: AuthResponse = try await performRequest(.post, "/auth/signup", body: request)
            
            keychain.set(response.accessToken, for: Constants.tokenKey)
            keychain.set(response.refreshToken, for: Constants.refreshTokenKey)
            
            await MainActor.run {
                self.isAuthenticated = true
                self.currentUser = response.user
                self.isLoading = false
            }
            
            await loadUserData()
            connectWebSocket()
            
        } catch {
            await MainActor.run {
                self.isLoading = false
                self.error = error as? FinovaError ?? .authenticationFailed
            }
            throw error
        }
    }
    
    public func signOut() async {
        do {
            let _: EmptyResponse = try await performRequest(.post, "/auth/signout")
        } catch {
            // Continue with local signout even if server request fails
        }
        
        // Clear all stored data
        keychain.delete(Constants.tokenKey)
        keychain.delete(Constants.refreshTokenKey)
        keychain.delete(Constants.biometricKey)
        keychain.delete(Constants.userKey)
        
        // Reset state
        await MainActor.run {
            self.isAuthenticated = false
            self.currentUser = nil
            self.mining = nil
            self.referralNetwork = nil
            self.ownedCards = []
            self.error = nil
        }
        
        webSocketTask?.cancel()
    }
    
    // MARK: - Mining Methods
    
    public func startMining() async throws {
        guard isAuthenticated else { throw FinovaError.authenticationFailed }
        guard currentUser?.kycStatus == .verified else { throw FinovaError.kycRequired }
        
        do {
            let request = StartMiningRequest(timestamp: Date())
            let response: MiningResponse = try await performRequest(.post, "/mining/start", body: request)
            
            await MainActor.run {
                self.mining = response.mining
            }
            
        } catch {
            await MainActor.run {
                self.error = error as? FinovaError ?? .networkError("Mining start failed")
            }
            throw error
        }
    }
    
    public func claimMining() async throws -> Double {
        guard isAuthenticated else { throw FinovaError.authenticationFailed }
        guard let mining = mining else { throw FinovaError.miningCooldown }
        guard Date() >= mining.nextClaimTime else { throw FinovaError.miningCooldown }
        
        do {
            let request = ClaimMiningRequest(timestamp: Date())
            let response: ClaimMiningResponse = try await performRequest(.post, "/mining/claim", body: request)
            
            await MainActor.run {
                self.mining = response.mining
                self.currentUser?.updateBalance(response.claimedAmount)
            }
            
            return response.claimedAmount
            
        } catch {
            await MainActor.run {
                self.error = error as? FinovaError ?? .networkError("Mining claim failed")
            }
            throw error
        }
    }
    
    public func calculateMiningRate() -> Double {
        guard let user = currentUser, let mining = mining else { return 0.0 }
        return calculator.calculateMiningRate(user: user, mining: mining)
    }
    
    // MARK: - XP System Methods
    
    public func submitActivity(_ activity: XPActivity) async throws -> Int {
        guard isAuthenticated else { throw FinovaError.authenticationFailed }
        
        do {
            let request = SubmitActivityRequest(activity: activity)
            let response: XPResponse = try await performRequest(.post, "/xp/activity", body: request)
            
            await MainActor.run {
                self.currentUser?.updateXP(response.xpGained)
            }
            
            return response.xpGained
            
        } catch {
            await MainActor.run {
                self.error = error as? FinovaError ?? .networkError("Activity submission failed")
            }
            throw error
        }
    }
    
    public func calculateXPMultiplier(for activity: XPActivity) -> Double {
        guard let user = currentUser else { return 1.0 }
        return calculator.calculateXPMultiplier(activity: activity, user: user)
    }
    
    public func getXPHistory(limit: Int = 50) async throws -> [XPActivity] {
        guard isAuthenticated else { throw FinovaError.authenticationFailed }
        
        do {
            let response: XPHistoryResponse = try await performRequest(.get, "/xp/history?limit=\(limit)")
            return response.activities
        } catch {
            await MainActor.run {
                self.error = error as? FinovaError ?? .networkError("XP history fetch failed")
            }
            throw error
        }
    }
    
    // MARK: - Referral System Methods
    
    public func generateReferralCode() async throws -> String {
        guard isAuthenticated else { throw FinovaError.authenticationFailed }
        
        do {
            let response: ReferralCodeResponse = try await performRequest(.post, "/referral/generate")
            return response.code
        } catch {
            await MainActor.run {
                self.error = error as? FinovaError ?? .networkError("Referral code generation failed")
            }
            throw error
        }
    }
    
    public func getReferralNetwork() async throws -> ReferralNetwork {
        guard isAuthenticated else { throw FinovaError.authenticationFailed }
        
        do {
            let response: ReferralNetworkResponse = try await performRequest(.get, "/referral/network")
            
            await MainActor.run {
                self.referralNetwork = response.network
            }
            
            return response.network
        } catch {
            await MainActor.run {
                self.error = error as? FinovaError ?? .networkError("Referral network fetch failed")
            }
            throw error
        }
    }
    
    public func calculateRPValue() -> Int {
        guard let user = currentUser, let network = referralNetwork else { return 0 }
        return calculator.calculateRPValue(user: user, network: network)
    }
    
    // MARK: - NFT & Cards Methods
    
    public func getOwnedCards() async throws -> [NFTCard] {
        guard isAuthenticated else { throw FinovaError.authenticationFailed }
        
        do {
            let response: NFTCardsResponse = try await performRequest(.get, "/nft/cards/owned")
            
            await MainActor.run {
                self.ownedCards = response.cards
            }
            
            return response.cards
        } catch {
            await MainActor.run {
                self.error = error as? FinovaError ?? .networkError("Cards fetch failed")
            }
            throw error
        }
    }
    
    public func useCard(_ cardId: String) async throws {
        guard isAuthenticated else { throw FinovaError.authenticationFailed }
        guard ownedCards.contains(where: { $0.id == cardId && !$0.isActive }) else {
            throw FinovaError.cardNotOwned
        }
        
        do {
            let request = UseCardRequest(cardId: cardId, timestamp: Date())
            let response: UseCardResponse = try await performRequest(.post, "/nft/cards/use", body: request)
            
            await MainActor.run {
                if let index = self.ownedCards.firstIndex(where: { $0.id == cardId }) {
                    self.ownedCards[index] = response.card
                }
                self.mining = response.updatedMining
            }
            
        } catch {
            await MainActor.run {
                self.error = error as? FinovaError ?? .networkError("Card use failed")
            }
            throw error
        }
    }
    
    public func purchaseCard(cardType: CardType, rarity: CardRarity) async throws -> NFTCard {
        guard isAuthenticated else { throw FinovaError.authenticationFailed }
        
        do {
            let request = PurchaseCardRequest(type: cardType, rarity: rarity)
            let response: PurchaseCardResponse = try await performRequest(.post, "/nft/cards/purchase", body: request)
            
            await MainActor.run {
                self.ownedCards.append(response.card)
                self.currentUser?.updateBalance(-response.cost)
            }
            
            return response.card
            
        } catch {
            await MainActor.run {
                self.error = error as? FinovaError ?? .networkError("Card purchase failed")
            }
            throw error
        }
    }
    
    // MARK: - Staking Methods
    
    public func stakeTokens(amount: Double) async throws {
        guard isAuthenticated else { throw FinovaError.authenticationFailed }
        guard let user = currentUser, user.totalFIN >= amount else {
            throw FinovaError.insufficientBalance
        }
        
        do {
            let request = StakeRequest(amount: amount)
            let response: StakeResponse = try await performRequest(.post, "/staking/stake", body: request)
            
            await MainActor.run {
                self.currentUser?.updateStaking(response.totalStaked)
                self.currentUser?.updateBalance(-amount)
                self.mining = response.updatedMining
            }
            
        } catch {
            await MainActor.run {
                self.error = error as? FinovaError ?? .networkError("Staking failed")
            }
            throw error
        }
    }
    
    public func unstakeTokens(amount: Double) async throws {
        guard isAuthenticated else { throw FinovaError.authenticationFailed }
        guard let user = currentUser, user.stakedFIN >= amount else {
            throw FinovaError.insufficientBalance
        }
        
        do {
            let request = UnstakeRequest(amount: amount)
            let response: UnstakeResponse = try await performRequest(.post, "/staking/unstake", body: request)
            
            await MainActor.run {
                self.currentUser?.updateStaking(response.totalStaked)
                self.currentUser?.updateBalance(amount)
                self.mining = response.updatedMining
            }
            
        } catch {
            await MainActor.run {
                self.error = error as? FinovaError ?? .networkError("Unstaking failed")
            }
            throw error
        }
    }
    
    // MARK: - Social Integration Methods
    
    public func connectSocialPlatform(_ platform: SocialPlatform, accessToken: String) async throws {
        guard isAuthenticated else { throw FinovaError.authenticationFailed }
        
        do {
            let request = SocialConnectRequest(platform: platform, accessToken: accessToken)
            let response: SocialConnectResponse = try await performRequest(.post, "/social/connect", body: request)
            
            await MainActor.run {
                self.currentUser?.addSocialConnection(response.connection)
            }
            
        } catch {
            await MainActor.run {
                self.error = error as? FinovaError ?? .networkError("Social platform connection failed")
            }
            throw error
        }
    }
    
    public func disconnectSocialPlatform(_ platform: SocialPlatform) async throws {
        guard isAuthenticated else { throw FinovaError.authenticationFailed }
        
        do {
            let request = SocialDisconnectRequest(platform: platform)
            let _: EmptyResponse = try await performRequest(.post, "/social/disconnect", body: request)
            
            await MainActor.run {
                self.currentUser?.removeSocialConnection(platform)
            }
            
        } catch {
            await MainActor.run {
                self.error = error as? FinovaError ?? .networkError("Social platform disconnection failed")
            }
            throw error
        }
    }
    
    // MARK: - KYC Methods
    
    public func submitKYC(documents: [KYCDocument]) async throws {
        guard isAuthenticated else { throw FinovaError.authenticationFailed }
        
        do {
            let request = KYCSubmissionRequest(documents: documents)
            let response: KYCResponse = try await performRequest(.post, "/kyc/submit", body: request)
            
            await MainActor.run {
                self.currentUser?.updateKYCStatus(response.status)
            }
            
        } catch {
            await MainActor.run {
                self.error = error as? FinovaError ?? .networkError("KYC submission failed")
            }
            throw error
        }
    }
    
    // MARK: - Utility Methods
    
    public func refreshUserData() async throws {
        await loadUserData()
    }
    
    public func getNetworkStats() async throws -> NetworkStats {
        let response: NetworkStatsResponse = try await performRequest(.get, "/network/stats")
        return response.stats
    }
    
    public func getLeaderboard(type: LeaderboardType, limit: Int = 100) async throws -> [LeaderboardEntry] {
        let response: LeaderboardResponse = try await performRequest(.get, "/leaderboard/\(type.rawValue)?limit=\(limit)")
        return response.entries
    }
    
    // MARK: - Private Implementation
    
    private func performRequest<T: Codable>(_ method: HTTPMethod, _ path: String, body: Encodable? = nil) async throws -> T {
        var request = URLRequest(url: URL(string: config.apiBaseURL + path)!)
        request.httpMethod = method.rawValue
        request.setValue("application/json", forHTTPHeaderField: "Content-Type")
        request.setValue("Bearer \(keychain.get(Constants.tokenKey) ?? "")", forHTTPHeaderField: "Authorization")
        request.setValue(config.apiKey, forHTTPHeaderField: "X-API-Key")
        
        if let body = body {
            request.httpBody = try JSONEncoder().encode(body)
        }
        
        let (data, response) = try await session.data(for: request)
        
        guard let httpResponse = response as? HTTPURLResponse else {
            throw FinovaError.networkError("Invalid response")
        }
        
        switch httpResponse.statusCode {
        case 200...299:
            if T.self == EmptyResponse.self {
                return EmptyResponse() as! T
            }
            return try JSONDecoder().decode(T.self, from: data)
        case 401:
            // Try to refresh token
            if await refreshTokenIfNeeded() {
                return try await performRequest(method, path, body: body)
            }
            throw FinovaError.authenticationFailed
        case 429:
            throw FinovaError.rateLimitExceeded
        default:
            let errorMessage = String(data: data, encoding: .utf8) ?? "Unknown error"
            throw FinovaError.serverError(httpResponse.statusCode, errorMessage)
        }
    }
    
    private func refreshTokenIfNeeded() async -> Bool {
        guard let refreshToken = keychain.get(Constants.refreshTokenKey) else { return false }
        
        do {
            let request = RefreshTokenRequest(refreshToken: refreshToken)
            let response: AuthResponse = try await performRequest(.post, "/auth/refresh", body: request)
            
            keychain.set(response.accessToken, for: Constants.tokenKey)
            keychain.set(response.refreshToken, for: Constants.refreshTokenKey)
            
            return true
        } catch {
            return false
        }
    }
    
    private func loadUserData() async {
        do {
            let userResponse: UserResponse = try await performRequest(.get, "/user/profile")
            let miningResponse: MiningResponse = try await performRequest(.get, "/mining/status")
            let referralResponse: ReferralNetworkResponse = try await performRequest(.get, "/referral/network")
            let cardsResponse: NFTCardsResponse = try await performRequest(.get, "/nft/cards/owned")
            
            await MainActor.run {
                self.currentUser = userResponse.user
                self.mining = miningResponse.mining
                self.referralNetwork = referralResponse.network
                self.ownedCards = cardsResponse.cards
            }
        } catch {
            await MainActor.run {
                self.error = error as? FinovaError ?? .networkError("Failed to load user data")
            }
        }
    }
    
    private func connectWebSocket() {
        webSocketTask?.resume()
        listenForWebSocketMessages()
    }
    
    private func listenForWebSocketMessages() {
        webSocketTask?.receive { [weak self] result in
            switch result {
            case .success(let message):
                Task {
                    await self?.handleWebSocketMessage(message)
                    self?.listenForWebSocketMessages()
                }
            case .failure(let error):
                print("WebSocket error: \(error)")
            }
        }
    }
    
    private func handleWebSocketMessage(_ message: URLSessionWebSocketTask.Message) async {
        switch message {
        case .string(let text):
            if let data = text.data(using: .utf8),
               let wsMessage = try? JSONDecoder().decode(WebSocketMessage.self, from: data) {
                await processWebSocketMessage(wsMessage)
            }
        case .data(let data):
            if let wsMessage = try? JSONDecoder().decode(WebSocketMessage.self, from: data) {
                await processWebSocketMessage(wsMessage)
            }
        @unknown default:
            break
        }
    }
    
    private func processWebSocketMessage(_ message: WebSocketMessage) async {
        await MainActor.run {
            switch message.type {
            case "mining_update":
                if let mining = message.data as? Mining {
                    self.mining = mining
                }
            case "xp_gained":
                if let xpData = message.data as? XPUpdate {
                    self.currentUser?.updateXP(xpData.amount)
                }
            case "referral_bonus":
                if let rpData = message.data as? RPUpdate {
                    self.referralNetwork?.updateRP(rpData.amount)
                }
            case "achievement_unlocked":
                if let achievement = message.data as? Achievement {
                    self.currentUser?.addAchievement(achievement)
                }
            default:
                break
            }
        }
    }
    
    private func setupNetworkMonitoring() {
        monitor.pathUpdateHandler = { [weak self] path in
            if path.status != .satisfied {
                Task {
                    await MainActor.run {
                        self?.error = .networkError("No internet connection")
                    }
                }
            }
        }
        monitor.start(queue: DispatchQueue.global())
    }
    
    private func setupMiningTimer() {
        Timer.publish(every: Constants.miningInterval, on: .main, in: .common)
            .autoconnect()
            .sink { [weak self] _ in
                Task {
                    try? await self?.updateMiningStatus()
                }
            }
            .store(in: &cancellables)
    }
    
    private func updateMiningStatus() async throws {
        guard isAuthenticated else { return }
        
        do {
            let response: MiningResponse = try await performRequest(.get, "/mining/status")
            await MainActor.run {
                self.mining = response.mining
            }
        } catch {
            // Silently fail for background updates
        }
    }
    
    private func loadPersistedState() {
        if let token = keychain.get(Constants.tokenKey),
           let userData = keychain.get(Constants.userKey),
           let user = try? JSONDecoder().decode(User.self, from: userData.data(using: .utf8) ?? Data()) {
            Task {
                await MainActor.run {
                    self.isAuthenticated = true
                    self.currentUser = user
                }
                await loadUserData()
            }
        }
    }
    
    private func validateAndSetAuthToken(_ token: String) async throws {
        let request = ValidateTokenRequest(token: token)
        let response: UserResponse = try await performRequest(.post, "/auth/validate", body: request)
        
        await MainActor.run {
            self.isAuthenticated = true
            self.currentUser = response.user
        }
        
        await loadUserData()
        connectWebSocket()
    }
    
    private func getDeviceInfo() -> DeviceInfo {
        return DeviceInfo(
            deviceId: UIDevice.current.identifierForVendor?.uuidString ?? UUID().uuidString,
            model: UIDevice.current.model,
            systemVersion: UIDevice.current.systemVersion,
            appVersion: Bundle.main.infoDictionary?["CFBundleShortVersionString"] as? String ?? "1.0"
        )
    }
    
    private func generateBiometricKey() -> String {
        return UUID().uuidString + "-" + Date().timeIntervalSince1970.description
    }
}

// MARK: - Supporting Classes

class KeychainManager {
    func set(_ value: String, for key: String) {
        let data = value.data(using: .utf8)!
        let query: [String: Any] = [
            kSecClass as String: kSecClassGenericPassword,
            kSecAttrAccount as String: key,
            kSecValueData as String: data,
            kSecAttrAccessible as String: kSecAttrAccessibleWhenUnlockedThisDeviceOnly
        ]
        
        SecItemDelete(query as CFDictionary)
        SecItemAdd(query as CFDictionary, nil)
    }
    
    func get(_ key: String) -> String? {
        let query: [String: Any] = [
            kSecClass as String: kSecClassGenericPassword,
            kSecAttrAccount as String: key,
            kSecReturnData as String: true,
            kSecMatchLimit as String: kSecMatchLimitOne
        ]
        
        var result: AnyObject?
        let status = SecItemCopyMatching(query as CFDictionary, &result)
        
        if status == errSecSuccess,
           let data = result as? Data,
           let string = String(data: data, encoding: .utf8) {
            return string
        }
        
        return nil
    }
    
    func delete(_ key: String) {
        let query: [String: Any] = [
            kSecClass as String: kSecClassGenericPassword,
            kSecAttrAccount as String: key
        ]
        
        SecItemDelete(query as CFDictionary)
    }
}

class BiometricManager {
    private let context = LAContext()
    
    func isBiometricAvailable() async -> Bool {
        return await withCheckedContinuation { continuation in
            DispatchQueue.global().async {
                var error: NSError?
                let available = self.context.canEvaluatePolicy(.deviceOwnerAuthenticationWithBiometrics, error: &error)
                continuation.resume(returning: available)
            }
        }
    }
    
    func authenticate(reason: String) async -> Bool {
        return await withCheckedContinuation { continuation in
            context.evaluatePolicy(.deviceOwnerAuthenticationWithBiometrics, localizedReason: reason) { success, error in
                continuation.resume(returning: success)
            }
        }
    }
}

class RewardCalculator {
    func calculateMiningRate(user: User, mining: Mining) -> Double {
        let baseRate = getPhaseBaseRate(mining.phase)
        let pioneerBonus = calculatePioneerBonus(user)
        let referralBonus = calculateReferralBonus(user)
        let xpBonus = calculateXPBonus(user)
        let rpBonus = calculateRPBonus(user)
        let stakingBonus = calculateStakingBonus(user)
        let regressionFactor = calculateRegressionFactor(user)
        
        return baseRate * pioneerBonus * referralBonus * xpBonus * rpBonus * stakingBonus * regressionFactor
    }
    
    func calculateXPMultiplier(activity: XPActivity, user: User) -> Double {
        let baseXP = Double(activity.baseXP)
        let platformMultiplier = activity.platform.multiplier
        let qualityScore = activity.qualityScore
        let streakBonus = calculateStreakBonus(user)
        let levelProgression = exp(-0.01 * Double(user.xpLevel))
        
        return baseXP * platformMultiplier * qualityScore * streakBonus * levelProgression
    }
    
    func calculateRPValue(user: User, network: ReferralNetwork) -> Int {
        let directRP = network.activeReferrals * 100
        let networkRP = network.networkSize.level2 * 30 + network.networkSize.level3 * 10
        let qualityBonus = network.qualityScore
        let regressionFactor = exp(-0.0001 * Double(network.totalReferrals) * qualityBonus)
        
        return Int(Double(directRP + networkRP) * qualityBonus * regressionFactor)
    }
    
    private func getPhaseBaseRate(_ phase: MiningPhase) -> Double {
        switch phase {
        case .pioneer: return 0.1
        case .growth: return 0.05
        case .maturity: return 0.025
        case .stability: return 0.01
        }
    }
    
    private func calculatePioneerBonus(_ user: User) -> Double {
        // Simplified calculation - would integrate with actual network size
        return max(1.0, 2.0 - (50000.0 / 1000000.0)) // Example with 50K users
    }
    
    private func calculateReferralBonus(_ user: User) -> Double {
        let activeReferrals = user.profile.socialConnections.count // Simplified
        return 1.0 + (Double(activeReferrals) * 0.1)
    }
    
    private func calculateXPBonus(_ user: User) -> Double {
        return 1.0 + (Double(user.xpLevel) / 100.0)
    }
    
    private func calculateRPBonus(_ user: User) -> Double {
        return 1.0 + user.rpTier.miningBonus
    }
    
    private func calculateStakingBonus(_ user: User) -> Double {
        if user.stakedFIN >= 10000 { return 2.0 }
        if user.stakedFIN >= 5000 { return 1.75 }
        if user.stakedFIN >= 1000 { return 1.5 }
        if user.stakedFIN >= 500 { return 1.35 }
        if user.stakedFIN >= 100 { return 1.2 }
        return 1.0
    }
    
    private func calculateRegressionFactor(_ user: User) -> Double {
        return exp(-0.001 * user.totalFIN)
    }
    
    private func calculateStreakBonus(_ user: User) -> Double {
        // Would be calculated based on user's streak data
        return 1.5 // Simplified
    }
}

// MARK: - Network Classes

class NetworkMonitor {
    private let monitor = NWPathMonitor()
    var pathUpdateHandler: ((NWPath) -> Void)?
    
    func start(queue: DispatchQueue) {
        monitor.pathUpdateHandler = pathUpdateHandler
        monitor.start(queue: queue)
    }
    
    func stop() {
        monitor.cancel()
    }
}

// MARK: - Request/Response Models

struct AuthRequest: Codable {
    let email: String
    let password: String
    let deviceInfo: DeviceInfo
}

struct SignUpRequest: Codable {
    let email: String
    let password: String
    let username: String
    let referralCode: String?
    let deviceInfo: DeviceInfo
}

struct RefreshTokenRequest: Codable {
    let refreshToken: String
}

struct ValidateTokenRequest: Codable {
    let token: String
}

struct StartMiningRequest: Codable {
    let timestamp: Date
}

struct ClaimMiningRequest: Codable {
    let timestamp: Date
}

struct SubmitActivityRequest: Codable {
    let activity: XPActivity
}

struct UseCardRequest: Codable {
    let cardId: String
    let timestamp: Date
}

struct PurchaseCardRequest: Codable {
    let type: CardType
    let rarity: CardRarity
}

struct StakeRequest: Codable {
    let amount: Double
}

struct UnstakeRequest: Codable {
    let amount: Double
}

struct SocialConnectRequest: Codable {
    let platform: SocialPlatform
    let accessToken: String
}

struct SocialDisconnectRequest: Codable {
    let platform: SocialPlatform
}

struct KYCSubmissionRequest: Codable {
    let documents: [KYCDocument]
}

struct AuthResponse: Codable {
    let accessToken: String
    let refreshToken: String
    let user: User
}

struct UserResponse: Codable {
    let user: User
}

struct MiningResponse: Codable {
    let mining: Mining
}

struct ClaimMiningResponse: Codable {
    let claimedAmount: Double
    let mining: Mining
}

struct XPResponse: Codable {
    let xpGained: Int
    let currentLevel: Int
    let totalXP: Int
}

struct XPHistoryResponse: Codable {
    let activities: [XPActivity]
}

struct ReferralCodeResponse: Codable {
    let code: String
}

struct ReferralNetworkResponse: Codable {
    let network: ReferralNetwork
}

struct NFTCardsResponse: Codable {
    let cards: [NFTCard]
}

struct UseCardResponse: Codable {
    let card: NFTCard
    let updatedMining: Mining
}

struct PurchaseCardResponse: Codable {
    let card: NFTCard
    let cost: Double
}

struct StakeResponse: Codable {
    let totalStaked: Double
    let updatedMining: Mining
}

struct UnstakeResponse: Codable {
    let totalStaked: Double
    let updatedMining: Mining
}

struct SocialConnectResponse: Codable {
    let connection: SocialConnection
}

struct KYCResponse: Codable {
    let status: KYCStatus
}

struct NetworkStatsResponse: Codable {
    let stats: NetworkStats
}

struct LeaderboardResponse: Codable {
    let entries: [LeaderboardEntry]
}

struct EmptyResponse: Codable {
    init() {}
}

// MARK: - Supporting Models

struct DeviceInfo: Codable {
    let deviceId: String
    let model: String
    let systemVersion: String
    let appVersion: String
}

struct KYCDocument: Codable {
    let type: DocumentType
    let imageData: Data
    let metadata: DocumentMetadata
    
    enum DocumentType: String, Codable {
        case passport, drivingLicense, nationalId, selfie
    }
}

struct DocumentMetadata: Codable {
    let fileName: String
    let fileSize: Int
    let timestamp: Date
}

struct NetworkStats: Codable {
    let totalUsers: Int
    let activeMiners: Int
    let totalFINMined: Double
    let currentPhase: MiningPhase
}

struct LeaderboardEntry: Codable {
    let rank: Int
    let username: String
    let value: Double
    let avatar: String?
}

enum LeaderboardType: String {
    case xp, mining, referrals, netWorth = "net_worth"
}

struct WebSocketMessage: Codable {
    let type: String
    let data: AnyCodable
}

struct XPUpdate: Codable {
    let amount: Int
    let source: String
}

struct RPUpdate: Codable {
    let amount: Int
    let source: String
}

// MARK: - User Extensions

extension User {
    mutating func updateBalance(_ amount: Double) {
        // Implementation would update totalFIN
    }
    
    mutating func updateXP(_ amount: Int) {
        // Implementation would update XP and potentially level
    }
    
    mutating func updateStaking(_ amount: Double) {
        // Implementation would update stakedFIN
    }
    
    mutating func updateKYCStatus(_ status: KYCStatus) {
        // Implementation would update KYC status
    }
    
    mutating func addSocialConnection(_ connection: SocialConnection) {
        // Implementation would add social connection
    }
    
    mutating func removeSocialConnection(_ platform: SocialPlatform) {
        // Implementation would remove social connection
    }
    
    mutating func addAchievement(_ achievement: Achievement) {
        // Implementation would add achievement
    }
}

extension ReferralNetwork {
    mutating func updateRP(_ amount: Int) {
        // Implementation would update RP
    }
}

// MARK: - HTTP Method Enum

enum HTTPMethod: String {
    case get = "GET"
    case post = "POST"
    case put = "PUT"
    case delete = "DELETE"
    case patch = "PATCH"
}

// MARK: - AnyCodable Helper

struct AnyCodable: Codable {
    let value: Any
    
    init<T>(_ value: T?) {
        self.value = value ?? ()
    }
}

extension AnyCodable {
    init(from decoder: Decoder) throws {
        let container = try decoder.singleValueContainer()
        
        if container.decodeNil() {
            self.init(())
        } else if let bool = try? container.decode(Bool.self) {
            self.init(bool)
        } else if let int = try? container.decode(Int.self) {
            self.init(int)
        } else if let double = try? container.decode(Double.self) {
            self.init(double)
        } else if let string = try? container.decode(String.self) {
            self.init(string)
        } else if let array = try? container.decode([AnyCodable].self) {
            self.init(array.map { $0.value })
        } else if let dictionary = try? container.decode([String: AnyCodable].self) {
            self.init(dictionary.mapValues { $0.value })
        } else {
            throw DecodingError.dataCorruptedError(in: container, debugDescription: "AnyCodable value cannot be decoded")
        }
    }
    
    func encode(to encoder: Encoder) throws {
        var container = encoder.singleValueContainer()
        
        switch value {
        case is Void:
            try container.encodeNil()
        case let bool as Bool:
            try container.encode(bool)
        case let int as Int:
            try container.encode(int)
        case let double as Double:
            try container.encode(double)
        case let string as String:
            try container.encode(string)
        case let array as [Any]:
            let anyArray = array.map { AnyCodable($0) }
            try container.encode(anyArray)
        case let dictionary as [String: Any]:
            let anyDictionary = dictionary.mapValues { AnyCodable($0) }
            try container.encode(anyDictionary)
        default:
            let context = EncodingError.Context(codingPath: container.codingPath, debugDescription: "AnyCodable value cannot be encoded")
            throw EncodingError.invalidValue(value, context)
        }
    }
}
