// finova-net/finova/mobile-sdk/ios/FinovaSDK/Sources/Client/TransactionManager.swift

import Foundation
import Combine
import CryptoKit

// MARK: - Transaction Types
public enum TransactionType: String, CaseIterable {
    case mining = "mining"
    case xpClaim = "xp_claim"
    case rpClaim = "rp_claim"
    case staking = "staking"
    case unstaking = "unstaking"
    case nftMint = "nft_mint"
    case nftTransfer = "nft_transfer"
    case cardUse = "card_use"
    case governance = "governance"
}

// MARK: - Transaction Status
public enum TransactionStatus {
    case pending
    case confirming
    case confirmed
    case failed(Error)
}

// MARK: - Transaction Models
public struct Transaction: Codable, Identifiable {
    public let id: String
    public let type: TransactionType
    public let amount: Double?
    public let fromAddress: String?
    public let toAddress: String?
    public let signature: String?
    public let blockHeight: UInt64?
    public let status: String
    public let createdAt: Date
    public let confirmedAt: Date?
    public let gasUsed: UInt64?
    public let metadata: [String: Any]?
    
    private enum CodingKeys: String, CodingKey {
        case id, type, amount, fromAddress, toAddress, signature
        case blockHeight, status, createdAt, confirmedAt, gasUsed
    }
}

public struct TransactionRequest {
    public let type: TransactionType
    public let amount: Double?
    public let recipientAddress: String?
    public let metadata: [String: Any]?
    public let priority: TransactionPriority
    
    public init(type: TransactionType, amount: Double? = nil, 
                recipientAddress: String? = nil, metadata: [String: Any]? = nil,
                priority: TransactionPriority = .normal) {
        self.type = type
        self.amount = amount
        self.recipientAddress = recipientAddress
        self.metadata = metadata
        self.priority = priority
    }
}

public enum TransactionPriority: String {
    case low = "low"
    case normal = "normal"
    case high = "high"
    case urgent = "urgent"
}

// MARK: - Transaction Manager
@MainActor
public class TransactionManager: ObservableObject {
    
    // MARK: - Published Properties
    @Published public private(set) var pendingTransactions: [Transaction] = []
    @Published public private(set) var transactionHistory: [Transaction] = []
    @Published public private(set) var isProcessing = false
    
    // MARK: - Private Properties
    private let networkService: NetworkService
    private let keyManager: KeyManager
    private let cacheManager: CacheManager
    private var cancellables = Set<AnyCancellable>()
    private let transactionQueue = DispatchQueue(label: "finova.transaction.queue", qos: .userInitiated)
    
    // MARK: - Configuration
    private struct Config {
        static let maxRetries = 3
        static let confirmationTimeout: TimeInterval = 120
        static let batchSize = 10
        static let cacheExpiry: TimeInterval = 3600 // 1 hour
    }
    
    // MARK: - Initialization
    public init(networkService: NetworkService, keyManager: KeyManager, cacheManager: CacheManager) {
        self.networkService = networkService
        self.keyManager = keyManager
        self.cacheManager = cacheManager
        setupTransactionMonitoring()
        loadCachedTransactions()
    }
    
    // MARK: - Public Methods
    
    /// Submit a new transaction
    public func submitTransaction(_ request: TransactionRequest) async throws -> Transaction {
        isProcessing = true
        defer { isProcessing = false }
        
        do {
            // Validate request
            try validateTransactionRequest(request)
            
            // Create transaction payload
            let payload = try await createTransactionPayload(request)
            
            // Sign transaction
            let signedPayload = try await signTransaction(payload)
            
            // Submit to blockchain
            let transaction = try await networkService.submitTransaction(signedPayload)
            
            // Add to pending transactions
            await addPendingTransaction(transaction)
            
            // Start monitoring
            monitorTransaction(transaction.id)
            
            return transaction
            
        } catch {
            throw TransactionError.submissionFailed(error.localizedDescription)
        }
    }
    
    /// Claim mining rewards
    public func claimMiningRewards() async throws -> Transaction {
        let request = TransactionRequest(
            type: .mining,
            priority: .normal
        )
        return try await submitTransaction(request)
    }
    
    /// Claim XP rewards
    public func claimXPRewards(amount: Double) async throws -> Transaction {
        let request = TransactionRequest(
            type: .xpClaim,
            amount: amount,
            metadata: ["timestamp": Date().timeIntervalSince1970],
            priority: .normal
        )
        return try await submitTransaction(request)
    }
    
    /// Stake tokens
    public func stakeTokens(amount: Double) async throws -> Transaction {
        let request = TransactionRequest(
            type: .staking,
            amount: amount,
            priority: .high
        )
        return try await submitTransaction(request)
    }
    
    /// Use special card NFT
    public func useSpecialCard(cardId: String, metadata: [String: Any]) async throws -> Transaction {
        let request = TransactionRequest(
            type: .cardUse,
            metadata: ["cardId": cardId].merging(metadata) { _, new in new },
            priority: .urgent
        )
        return try await submitTransaction(request)
    }
    
    /// Get transaction by ID
    public func getTransaction(id: String) async throws -> Transaction? {
        // Check cache first
        if let cached = getCachedTransaction(id) {
            return cached
        }
        
        // Fetch from network
        let transaction = try await networkService.getTransaction(id: id)
        
        // Cache result
        if let transaction = transaction {
            cacheTransaction(transaction)
        }
        
        return transaction
    }
    
    /// Get transaction history with pagination
    public func getTransactionHistory(page: Int = 0, limit: Int = 20) async throws -> [Transaction] {
        let cacheKey = "tx_history_\(page)_\(limit)"
        
        // Check cache
        if let cached: [Transaction] = cacheManager.get(key: cacheKey) {
            return cached
        }
        
        // Fetch from network
        let transactions = try await networkService.getTransactionHistory(
            page: page,
            limit: limit
        )
        
        // Cache results
        cacheManager.set(key: cacheKey, value: transactions, expiry: Config.cacheExpiry)
        
        DispatchQueue.main.async {
            if page == 0 {
                self.transactionHistory = transactions
            } else {
                self.transactionHistory.append(contentsOf: transactions)
            }
        }
        
        return transactions
    }
    
    /// Cancel pending transaction
    public func cancelTransaction(id: String) async throws {
        guard let transaction = pendingTransactions.first(where: { $0.id == id }) else {
            throw TransactionError.transactionNotFound
        }
        
        try await networkService.cancelTransaction(id: id)
        
        await MainActor.run {
            pendingTransactions.removeAll { $0.id == id }
        }
    }
    
    /// Retry failed transaction
    public func retryTransaction(id: String) async throws -> Transaction {
        guard let transaction = transactionHistory.first(where: { $0.id == id }) else {
            throw TransactionError.transactionNotFound
        }
        
        let request = TransactionRequest(
            type: transaction.type,
            amount: transaction.amount,
            recipientAddress: transaction.toAddress,
            metadata: transaction.metadata,
            priority: .normal
        )
        
        return try await submitTransaction(request)
    }
    
    // MARK: - Private Methods
    
    private func setupTransactionMonitoring() {
        // Monitor network connection
        NetworkMonitor.shared.$isConnected
            .sink { [weak self] isConnected in
                if isConnected {
                    Task {
                        await self?.syncPendingTransactions()
                    }
                }
            }
            .store(in: &cancellables)
        
        // Periodic sync
        Timer.publish(every: 30, on: .main, in: .common)
            .autoconnect()
            .sink { [weak self] _ in
                Task {
                    await self?.syncPendingTransactions()
                }
            }
            .store(in: &cancellables)
    }
    
    private func validateTransactionRequest(_ request: TransactionRequest) throws {
        // Validate amount
        if let amount = request.amount {
            guard amount > 0 else {
                throw TransactionError.invalidAmount
            }
        }
        
        // Validate recipient for transfers
        if request.type == .nftTransfer {
            guard request.recipientAddress != nil else {
                throw TransactionError.missingRecipient
            }
        }
        
        // Validate staking amount
        if request.type == .staking {
            guard let amount = request.amount, amount >= 100 else {
                throw TransactionError.minimumStakeNotMet
            }
        }
    }
    
    private func createTransactionPayload(_ request: TransactionRequest) async throws -> [String: Any] {
        let userAddress = try await keyManager.getUserAddress()
        let nonce = try await networkService.getNonce(address: userAddress)
        
        var payload: [String: Any] = [
            "type": request.type.rawValue,
            "from": userAddress,
            "nonce": nonce,
            "timestamp": Date().timeIntervalSince1970,
            "priority": request.priority.rawValue
        ]
        
        if let amount = request.amount {
            payload["amount"] = amount
        }
        
        if let recipient = request.recipientAddress {
            payload["to"] = recipient
        }
        
        if let metadata = request.metadata {
            payload["metadata"] = metadata
        }
        
        return payload
    }
    
    private func signTransaction(_ payload: [String: Any]) async throws -> [String: Any] {
        let payloadData = try JSONSerialization.data(withJSONObject: payload)
        let signature = try await keyManager.signData(payloadData)
        
        var signedPayload = payload
        signedPayload["signature"] = signature
        
        return signedPayload
    }
    
    private func addPendingTransaction(_ transaction: Transaction) async {
        await MainActor.run {
            pendingTransactions.append(transaction)
            // Cache pending transactions
            cacheManager.set(key: "pending_tx", value: pendingTransactions)
        }
    }
    
    private func monitorTransaction(_ transactionId: String) {
        Task {
            var retryCount = 0
            
            while retryCount < Config.maxRetries {
                do {
                    guard let transaction = try await getTransaction(id: transactionId) else {
                        throw TransactionError.transactionNotFound
                    }
                    
                    await MainActor.run {
                        updateTransactionStatus(transaction)
                    }
                    
                    // If confirmed or failed, stop monitoring
                    if transaction.status == "confirmed" || transaction.status == "failed" {
                        break
                    }
                    
                    // Wait before next check
                    try await Task.sleep(nanoseconds: 5_000_000_000) // 5 seconds
                    
                } catch {
                    retryCount += 1
                    if retryCount >= Config.maxRetries {
                        await MainActor.run {
                            handleTransactionError(transactionId, error)
                        }
                        break
                    }
                    
                    // Exponential backoff
                    let delay = UInt64(pow(2.0, Double(retryCount)) * 1_000_000_000)
                    try await Task.sleep(nanoseconds: delay)
                }
            }
        }
    }
    
    private func updateTransactionStatus(_ transaction: Transaction) {
        // Remove from pending if confirmed or failed
        if transaction.status == "confirmed" || transaction.status == "failed" {
            pendingTransactions.removeAll { $0.id == transaction.id }
        }
        
        // Update in history
        if let index = transactionHistory.firstIndex(where: { $0.id == transaction.id }) {
            transactionHistory[index] = transaction
        } else {
            transactionHistory.insert(transaction, at: 0)
        }
        
        // Cache updates
        cacheManager.set(key: "pending_tx", value: pendingTransactions)
        cacheManager.set(key: "tx_history_0_20", value: Array(transactionHistory.prefix(20)))
    }
    
    private func handleTransactionError(_ transactionId: String, _ error: Error) {
        // Remove from pending
        pendingTransactions.removeAll { $0.id == transactionId }
        
        // Log error
        print("Transaction \(transactionId) failed: \(error.localizedDescription)")
        
        // Notify user through notification system
        NotificationCenter.default.post(
            name: .transactionFailed,
            object: nil,
            userInfo: ["transactionId": transactionId, "error": error]
        )
    }
    
    private func syncPendingTransactions() async {
        for transaction in pendingTransactions {
            monitorTransaction(transaction.id)
        }
    }
    
    private func loadCachedTransactions() {
        // Load pending transactions
        if let cached: [Transaction] = cacheManager.get(key: "pending_tx") {
            pendingTransactions = cached
        }
        
        // Load transaction history
        if let cached: [Transaction] = cacheManager.get(key: "tx_history_0_20") {
            transactionHistory = cached
        }
    }
    
    private func getCachedTransaction(_ id: String) -> Transaction? {
        return cacheManager.get(key: "tx_\(id)")
    }
    
    private func cacheTransaction(_ transaction: Transaction) {
        cacheManager.set(key: "tx_\(transaction.id)", value: transaction, expiry: Config.cacheExpiry)
    }
}

// MARK: - Transaction Errors
public enum TransactionError: LocalizedError {
    case invalidAmount
    case missingRecipient
    case minimumStakeNotMet
    case transactionNotFound
    case submissionFailed(String)
    case signingFailed
    case networkError
    case insufficientFunds
    case rateLimitExceeded
    
    public var errorDescription: String? {
        switch self {
        case .invalidAmount:
            return "Invalid transaction amount"
        case .missingRecipient:
            return "Recipient address is required"
        case .minimumStakeNotMet:
            return "Minimum stake amount is 100 FIN"
        case .transactionNotFound:
            return "Transaction not found"
        case .submissionFailed(let reason):
            return "Transaction submission failed: \(reason)"
        case .signingFailed:
            return "Failed to sign transaction"
        case .networkError:
            return "Network error occurred"
        case .insufficientFunds:
            return "Insufficient funds for transaction"
        case .rateLimitExceeded:
            return "Rate limit exceeded, please try again later"
        }
    }
}

// MARK: - Notification Names
extension Notification.Name {
    static let transactionFailed = Notification.Name("transactionFailed")
    static let transactionConfirmed = Notification.Name("transactionConfirmed")
}

// MARK: - Network Monitor (Placeholder)
class NetworkMonitor: ObservableObject {
    static let shared = NetworkMonitor()
    @Published var isConnected = true
    
    private init() {}
}

// MARK: - Supporting Services (Interfaces)
protocol NetworkService {
    func submitTransaction(_ payload: [String: Any]) async throws -> Transaction
    func getTransaction(id: String) async throws -> Transaction?
    func getTransactionHistory(page: Int, limit: Int) async throws -> [Transaction]
    func cancelTransaction(id: String) async throws
    func getNonce(address: String) async throws -> UInt64
}

protocol KeyManager {
    func getUserAddress() async throws -> String
    func signData(_ data: Data) async throws -> String
}

protocol CacheManager {
    func get<T: Codable>(key: String) -> T?
    func set<T: Codable>(key: String, value: T, expiry: TimeInterval?)
}

// MARK: - Extensions
extension CacheManager {
    func set<T: Codable>(key: String, value: T) {
        set(key: key, value: value, expiry: nil)
    }
}
