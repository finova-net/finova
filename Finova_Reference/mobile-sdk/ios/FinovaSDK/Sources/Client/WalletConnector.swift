// finova-net/finova/mobile-sdk/ios/FinovaSDK/Sources/Client/WalletConnector.swift

import Foundation
import Combine
import CryptoKit
import LocalAuthentication

// MARK: - Wallet Types
public enum WalletType: String, CaseIterable {
    case phantom = "phantom"
    case solflare = "solflare"
    case backpack = "backpack"
    case glow = "glow"
    case native = "finova_native"
}

// MARK: - Connection Status
public enum WalletConnectionStatus {
    case disconnected
    case connecting
    case connected(publicKey: String)
    case error(WalletError)
}

// MARK: - Wallet Errors
public enum WalletError: Error, LocalizedError {
    case walletNotInstalled(WalletType)
    case connectionFailed(String)
    case transactionFailed(String)
    case userRejected
    case insufficientFunds
    case networkError(String)
    case invalidAddress
    case signatureFailed
    case biometricAuthFailed
    
    public var errorDescription: String? {
        switch self {
        case .walletNotInstalled(let type):
            return "Wallet \(type.rawValue) is not installed"
        case .connectionFailed(let msg):
            return "Connection failed: \(msg)"
        case .transactionFailed(let msg):
            return "Transaction failed: \(msg)"
        case .userRejected:
            return "User rejected the request"
        case .insufficientFunds:
            return "Insufficient funds for transaction"
        case .networkError(let msg):
            return "Network error: \(msg)"
        case .invalidAddress:
            return "Invalid wallet address"
        case .signatureFailed:
            return "Failed to sign transaction"
        case .biometricAuthFailed:
            return "Biometric authentication failed"
        }
    }
}

// MARK: - Transaction Request
public struct TransactionRequest {
    public let to: String
    public let amount: UInt64
    public let tokenMint: String?
    public let instruction: String?
    public let priority: TransactionPriority
    
    public init(to: String, amount: UInt64, tokenMint: String? = nil, 
               instruction: String? = nil, priority: TransactionPriority = .normal) {
        self.to = to
        self.amount = amount
        self.tokenMint = tokenMint
        self.instruction = instruction
        self.priority = priority
    }
}

public enum TransactionPriority {
    case low
    case normal  
    case high
    case urgent
    
    var computeUnitPrice: UInt64 {
        switch self {
        case .low: return 10_000
        case .normal: return 50_000
        case .high: return 100_000
        case .urgent: return 200_000
        }
    }
}

// MARK: - Transaction Response
public struct TransactionResponse {
    public let signature: String
    public let status: TransactionStatus
    public let blockHeight: UInt64?
    public let fee: UInt64?
}

public enum TransactionStatus {
    case pending
    case confirmed
    case failed(String)
    case finalized
}

// MARK: - Wallet Info
public struct WalletInfo {
    public let type: WalletType
    public let publicKey: String
    public let balance: UInt64
    public let tokenBalances: [String: UInt64]
    public let isKYCVerified: Bool
    public let xpLevel: Int
    public let rpTier: String
    public let miningRate: Double
}

// MARK: - Main Wallet Connector
public class WalletConnector: ObservableObject {
    
    // MARK: - Published Properties
    @Published public private(set) var connectionStatus: WalletConnectionStatus = .disconnected
    @Published public private(set) var connectedWallet: WalletInfo?
    @Published public private(set) var isLoading = false
    
    // MARK: - Private Properties
    private var cancellables = Set<AnyCancellable>()
    private let finovaAPI: FinovaAPIClient
    private let secureStorage: SecureStorage
    private let biometricAuth: BiometricAuthenticator
    private let urlSchemeHandler: URLSchemeHandler
    
    // MARK: - Initialization
    public init(apiKey: String, environment: FinovaEnvironment = .mainnet) {
        self.finovaAPI = FinovaAPIClient(apiKey: apiKey, environment: environment)
        self.secureStorage = SecureStorage()
        self.biometricAuth = BiometricAuthenticator()
        self.urlSchemeHandler = URLSchemeHandler()
        
        setupURLSchemeHandling()
        loadSavedConnection()
    }
    
    // MARK: - Public Connection Methods
    public func connect(to walletType: WalletType) async throws {
        DispatchQueue.main.async {
            self.isLoading = true
            self.connectionStatus = .connecting
        }
        
        do {
            let publicKey = try await performConnection(to: walletType)
            let walletInfo = try await fetchWalletInfo(publicKey: publicKey, type: walletType)
            
            await saveConnection(publicKey: publicKey, type: walletType)
            
            DispatchQueue.main.async {
                self.connectionStatus = .connected(publicKey: publicKey)
                self.connectedWallet = walletInfo
                self.isLoading = false
            }
            
        } catch {
            DispatchQueue.main.async {
                self.connectionStatus = .error(error as! WalletError)
                self.isLoading = false
            }
            throw error
        }
    }
    
    public func disconnect() async {
        await clearSavedConnection()
        
        DispatchQueue.main.async {
            self.connectionStatus = .disconnected
            self.connectedWallet = nil
        }
    }
    
    // MARK: - Transaction Methods
    public func sendTransaction(_ request: TransactionRequest) async throws -> TransactionResponse {
        guard case .connected(let publicKey) = connectionStatus else {
            throw WalletError.connectionFailed("Wallet not connected")
        }
        
        // Biometric authentication for transactions
        try await biometricAuth.authenticate(reason: "Authenticate to send transaction")
        
        // Build transaction
        let transaction = try await buildTransaction(request, from: publicKey)
        
        // Sign transaction based on wallet type
        let signedTransaction = try await signTransaction(transaction)
        
        // Send to blockchain
        let response = try await finovaAPI.sendTransaction(signedTransaction)
        
        return response
    }
    
    public func signMessage(_ message: String) async throws -> String {
        guard case .connected(_) = connectionStatus else {
            throw WalletError.connectionFailed("Wallet not connected")
        }
        
        try await biometricAuth.authenticate(reason: "Authenticate to sign message")
        
        guard let connectedWallet = connectedWallet else {
            throw WalletError.connectionFailed("No wallet info available")
        }
        
        return try await performMessageSigning(message, walletType: connectedWallet.type)
    }
    
    // MARK: - Utility Methods
    public func refreshWalletInfo() async throws {
        guard case .connected(let publicKey) = connectionStatus,
              let wallet = connectedWallet else {
            throw WalletError.connectionFailed("Wallet not connected")
        }
        
        let updatedInfo = try await fetchWalletInfo(publicKey: publicKey, type: wallet.type)
        
        DispatchQueue.main.async {
            self.connectedWallet = updatedInfo
        }
    }
    
    public func getAvailableWallets() -> [WalletType] {
        return WalletType.allCases.filter { isWalletInstalled($0) }
    }
    
    // MARK: - Private Implementation
    private func performConnection(to walletType: WalletType) async throws -> String {
        switch walletType {
        case .phantom:
            return try await connectPhantom()
        case .solflare:
            return try await connectSolflare()
        case .backpack:
            return try await connectBackpack()
        case .glow:
            return try await connectGlow()
        case .native:
            return try await connectNativeWallet()
        }
    }
    
    private func connectPhantom() async throws -> String {
        guard isWalletInstalled(.phantom) else {
            throw WalletError.walletNotInstalled(.phantom)
        }
        
        let connectURL = URL(string: "phantom://v1/connect?app_url=finova://&redirect_url=finova://phantom-response")!
        
        return try await withCheckedThrowingContinuation { continuation in
            urlSchemeHandler.handleResponse = { [weak self] url in
                if let components = URLComponents(url: url, resolvingAgainstBaseURL: false),
                   let queryItems = components.queryItems {
                    
                    if let publicKey = queryItems.first(where: { $0.name == "public_key" })?.value {
                        continuation.resume(returning: publicKey)
                    } else if queryItems.contains(where: { $0.name == "error" }) {
                        continuation.resume(throwing: WalletError.userRejected)
                    }
                }
            }
            
            UIApplication.shared.open(connectURL) { success in
                if !success {
                    continuation.resume(throwing: WalletError.connectionFailed("Failed to open Phantom"))
                }
            }
        }
    }
    
    private func connectSolflare() async throws -> String {
        guard isWalletInstalled(.solflare) else {
            throw WalletError.walletNotInstalled(.solflare)
        }
        
        let connectURL = URL(string: "solflare://connect?callback=finova://solflare-response")!
        
        return try await withCheckedThrowingContinuation { continuation in
            urlSchemeHandler.handleResponse = { url in
                if let publicKey = self.extractPublicKeyFromURL(url) {
                    continuation.resume(returning: publicKey)
                } else {
                    continuation.resume(throwing: WalletError.connectionFailed("Invalid response"))
                }
            }
            
            UIApplication.shared.open(connectURL) { success in
                if !success {
                    continuation.resume(throwing: WalletError.connectionFailed("Failed to open Solflare"))
                }
            }
        }
    }
    
    private func connectBackpack() async throws -> String {
        guard isWalletInstalled(.backpack) else {
            throw WalletError.walletNotInstalled(.backpack)
        }
        
        // Backpack connection implementation
        let connectURL = URL(string: "backpack://connect?origin=finova&callback=finova://backpack-response")!
        
        return try await withCheckedThrowingContinuation { continuation in
            urlSchemeHandler.handleResponse = { url in
                if let publicKey = self.extractPublicKeyFromURL(url) {
                    continuation.resume(returning: publicKey)
                } else {
                    continuation.resume(throwing: WalletError.connectionFailed("Invalid response"))
                }
            }
            
            UIApplication.shared.open(connectURL) { success in
                if !success {
                    continuation.resume(throwing: WalletError.connectionFailed("Failed to open Backpack"))
                }
            }
        }
    }
    
    private func connectGlow() async throws -> String {
        guard isWalletInstalled(.glow) else {
            throw WalletError.walletNotInstalled(.glow)
        }
        
        // Glow connection implementation
        let connectURL = URL(string: "glow://connect?redirect=finova://glow-response")!
        
        return try await withCheckedThrowingContinuation { continuation in
            urlSchemeHandler.handleResponse = { url in
                if let publicKey = self.extractPublicKeyFromURL(url) {
                    continuation.resume(returning: publicKey)
                } else {
                    continuation.resume(throwing: WalletError.connectionFailed("Invalid response"))
                }
            }
            
            UIApplication.shared.open(connectURL) { success in
                if !success {
                    continuation.resume(throwing: WalletError.connectionFailed("Failed to open Glow"))
                }
            }
        }
    }
    
    private func connectNativeWallet() async throws -> String {
        // Generate or retrieve native wallet keypair
        if let existingKey = try secureStorage.retrievePrivateKey() {
            return try derivePublicKey(from: existingKey)
        } else {
            let keypair = try generateNewKeypair()
            try secureStorage.storePrivateKey(keypair.privateKey)
            return keypair.publicKey
        }
    }
    
    private func fetchWalletInfo(publicKey: String, type: WalletType) async throws -> WalletInfo {
        // Fetch wallet information from Finova API
        let balanceInfo = try await finovaAPI.getWalletBalance(publicKey: publicKey)
        let userProfile = try await finovaAPI.getUserProfile(publicKey: publicKey)
        
        return WalletInfo(
            type: type,
            publicKey: publicKey,
            balance: balanceInfo.balance,
            tokenBalances: balanceInfo.tokenBalances,
            isKYCVerified: userProfile.isKYCVerified,
            xpLevel: userProfile.xpLevel,
            rpTier: userProfile.rpTier,
            miningRate: userProfile.miningRate
        )
    }
    
    private func buildTransaction(_ request: TransactionRequest, from publicKey: String) async throws -> String {
        // Build Solana transaction
        let instruction = try await finovaAPI.buildTransactionInstruction(
            from: publicKey,
            to: request.to,
            amount: request.amount,
            tokenMint: request.tokenMint,
            computeUnitPrice: request.priority.computeUnitPrice
        )
        
        return instruction
    }
    
    private func signTransaction(_ transaction: String) async throws -> String {
        guard let wallet = connectedWallet else {
            throw WalletError.connectionFailed("No wallet connected")
        }
        
        switch wallet.type {
        case .phantom:
            return try await signWithPhantom(transaction)
        case .solflare:
            return try await signWithSolflare(transaction)
        case .backpack:
            return try await signWithBackpack(transaction)
        case .glow:
            return try await signWithGlow(transaction)
        case .native:
            return try await signWithNativeWallet(transaction)
        }
    }
    
    private func signWithPhantom(_ transaction: String) async throws -> String {
        let signURL = URL(string: "phantom://v1/signTransaction?transaction=\(transaction)&redirect_url=finova://phantom-sign-response")!
        
        return try await withCheckedThrowingContinuation { continuation in
            urlSchemeHandler.handleResponse = { url in
                if let signature = self.extractSignatureFromURL(url) {
                    continuation.resume(returning: signature)
                } else {
                    continuation.resume(throwing: WalletError.signatureFailed)
                }
            }
            
            UIApplication.shared.open(signURL) { success in
                if !success {
                    continuation.resume(throwing: WalletError.signatureFailed)
                }
            }
        }
    }
    
    private func signWithSolflare(_ transaction: String) async throws -> String {
        let signURL = URL(string: "solflare://sign?transaction=\(transaction)&callback=finova://solflare-sign-response")!
        
        return try await withCheckedThrowingContinuation { continuation in
            urlSchemeHandler.handleResponse = { url in
                if let signature = self.extractSignatureFromURL(url) {
                    continuation.resume(returning: signature)
                } else {
                    continuation.resume(throwing: WalletError.signatureFailed)
                }
            }
            
            UIApplication.shared.open(signURL) { success in
                if !success {
                    continuation.resume(throwing: WalletError.signatureFailed)
                }
            }
        }
    }
    
    private func signWithBackpack(_ transaction: String) async throws -> String {
        let signURL = URL(string: "backpack://sign?tx=\(transaction)&callback=finova://backpack-sign-response")!
        
        return try await withCheckedThrowingContinuation { continuation in
            urlSchemeHandler.handleResponse = { url in
                if let signature = self.extractSignatureFromURL(url) {
                    continuation.resume(returning: signature)
                } else {
                    continuation.resume(throwing: WalletError.signatureFailed)
                }
            }
            
            UIApplication.shared.open(signURL) { success in
                if !success {
                    continuation.resume(throwing: WalletError.signatureFailed)
                }
            }
        }
    }
    
    private func signWithGlow(_ transaction: String) async throws -> String {
        let signURL = URL(string: "glow://sign?transaction=\(transaction)&redirect=finova://glow-sign-response")!
        
        return try await withCheckedThrowingContinuation { continuation in
            urlSchemeHandler.handleResponse = { url in
                if let signature = self.extractSignatureFromURL(url) {
                    continuation.resume(returning: signature)
                } else {
                    continuation.resume(throwing: WalletError.signatureFailed)
                }
            }
            
            UIApplication.shared.open(signURL) { success in
                if !success {
                    continuation.resume(throwing: WalletError.signatureFailed)
                }
            }
        }
    }
    
    private func signWithNativeWallet(_ transaction: String) async throws -> String {
        guard let privateKey = try secureStorage.retrievePrivateKey() else {
            throw WalletError.signatureFailed
        }
        
        // Sign transaction with native wallet private key
        return try signTransactionWithPrivateKey(transaction: transaction, privateKey: privateKey)
    }
    
    private func performMessageSigning(_ message: String, walletType: WalletType) async throws -> String {
        switch walletType {
        case .phantom:
            return try await signMessageWithPhantom(message)
        case .solflare:
            return try await signMessageWithSolflare(message)
        case .backpack:
            return try await signMessageWithBackpack(message)
        case .glow:
            return try await signMessageWithGlow(message)
        case .native:
            return try await signMessageWithNativeWallet(message)
        }
    }
    
    private func signMessageWithPhantom(_ message: String) async throws -> String {
        let encodedMessage = message.data(using: .utf8)?.base64EncodedString() ?? ""
        let signURL = URL(string: "phantom://v1/signMessage?message=\(encodedMessage)&redirect_url=finova://phantom-message-response")!
        
        return try await withCheckedThrowingContinuation { continuation in
            urlSchemeHandler.handleResponse = { url in
                if let signature = self.extractSignatureFromURL(url) {
                    continuation.resume(returning: signature)
                } else {
                    continuation.resume(throwing: WalletError.signatureFailed)
                }
            }
            
            UIApplication.shared.open(signURL) { success in
                if !success {
                    continuation.resume(throwing: WalletError.signatureFailed)
                }
            }
        }
    }
    
    private func signMessageWithSolflare(_ message: String) async throws -> String {
        let encodedMessage = message.data(using: .utf8)?.base64EncodedString() ?? ""
        let signURL = URL(string: "solflare://signMessage?message=\(encodedMessage)&callback=finova://solflare-message-response")!
        
        return try await withCheckedThrowingContinuation { continuation in
            urlSchemeHandler.handleResponse = { url in
                if let signature = self.extractSignatureFromURL(url) {
                    continuation.resume(returning: signature)
                } else {
                    continuation.resume(throwing: WalletError.signatureFailed)
                }
            }
            
            UIApplication.shared.open(signURL) { success in
                if !success {
                    continuation.resume(throwing: WalletError.signatureFailed)
                }
            }
        }
    }
    
    private func signMessageWithBackpack(_ message: String) async throws -> String {
        let encodedMessage = message.data(using: .utf8)?.base64EncodedString() ?? ""
        let signURL = URL(string: "backpack://signMessage?msg=\(encodedMessage)&callback=finova://backpack-message-response")!
        
        return try await withCheckedThrowingContinuation { continuation in
            urlSchemeHandler.handleResponse = { url in
                if let signature = self.extractSignatureFromURL(url) {
                    continuation.resume(returning: signature)
                } else {
                    continuation.resume(throwing: WalletError.signatureFailed)
                }
            }
            
            UIApplication.shared.open(signURL) { success in
                if !success {
                    continuation.resume(throwing: WalletError.signatureFailed)
                }
            }
        }
    }
    
    private func signMessageWithGlow(_ message: String) async throws -> String {
        let encodedMessage = message.data(using: .utf8)?.base64EncodedString() ?? ""
        let signURL = URL(string: "glow://signMessage?message=\(encodedMessage)&redirect=finova://glow-message-response")!
        
        return try await withCheckedThrowingContinuation { continuation in
            urlSchemeHandler.handleResponse = { url in
                if let signature = self.extractSignatureFromURL(url) {
                    continuation.resume(returning: signature)
                } else {
                    continuation.resume(throwing: WalletError.signatureFailed)
                }
            }
            
            UIApplication.shared.open(signURL) { success in
                if !success {
                    continuation.resume(throwing: WalletError.signatureFailed)
                }
            }
        }
    }
    
    private func signMessageWithNativeWallet(_ message: String) async throws -> String {
        guard let privateKey = try secureStorage.retrievePrivateKey() else {
            throw WalletError.signatureFailed
        }
        
        return try signMessageWithPrivateKey(message: message, privateKey: privateKey)
    }
    
    // MARK: - Helper Methods
    private func isWalletInstalled(_ walletType: WalletType) -> Bool {
        let scheme: String
        switch walletType {
        case .phantom: scheme = "phantom://"
        case .solflare: scheme = "solflare://"
        case .backpack: scheme = "backpack://"
        case .glow: scheme = "glow://"
        case .native: return true
        }
        
        guard let url = URL(string: scheme) else { return false }
        return UIApplication.shared.canOpenURL(url)
    }
    
    private func extractPublicKeyFromURL(_ url: URL) -> String? {
        guard let components = URLComponents(url: url, resolvingAgainstBaseURL: false),
              let queryItems = components.queryItems else { return nil }
        
        return queryItems.first(where: { 
            $0.name == "public_key" || $0.name == "publicKey" || $0.name == "pubkey" 
        })?.value
    }
    
    private func extractSignatureFromURL(_ url: URL) -> String? {
        guard let components = URLComponents(url: url, resolvingAgainstBaseURL: false),
              let queryItems = components.queryItems else { return nil }
        
        return queryItems.first(where: { 
            $0.name == "signature" || $0.name == "sig" || $0.name == "signed_transaction" 
        })?.value
    }
    
    private func setupURLSchemeHandling() {
        NotificationCenter.default.addObserver(
            forName: .finovaURLSchemeResponse,
            object: nil,
            queue: .main
        ) { [weak self] notification in
            if let url = notification.userInfo?["url"] as? URL {
                self?.urlSchemeHandler.handleResponse?(url)
            }
        }
    }
    
    private func loadSavedConnection() {
        if let savedConnection = secureStorage.retrieveSavedConnection() {
            Task {
                try await connect(to: savedConnection.walletType)
            }
        }
    }
    
    private func saveConnection(publicKey: String, type: WalletType) async {
        let connection = SavedConnection(publicKey: publicKey, walletType: type, timestamp: Date())
        try? secureStorage.saveSavedConnection(connection)
    }
    
    private func clearSavedConnection() async {
        secureStorage.clearSavedConnection()
    }
    
    private func generateNewKeypair() throws -> (publicKey: String, privateKey: Data) {
        let privateKey = SymmetricKey(size: .bits256)
        let privateKeyData = privateKey.withUnsafeBytes { Data($0) }
        let publicKey = try derivePublicKey(from: privateKeyData)
        
        return (publicKey: publicKey, privateKey: privateKeyData)
    }
    
    private func derivePublicKey(from privateKey: Data) throws -> String {
        // Derive public key from private key (simplified Ed25519 implementation)
        let hash = SHA256.hash(data: privateKey)
        return Data(hash).base58EncodedString
    }
    
    private func signTransactionWithPrivateKey(transaction: String, privateKey: Data) throws -> String {
        // Sign transaction with Ed25519 (simplified implementation)
        guard let transactionData = Data(base64Encoded: transaction) else {
            throw WalletError.signatureFailed
        }
        
        let signature = try signData(transactionData, with: privateKey)
        return signature.base64EncodedString()
    }
    
    private func signMessageWithPrivateKey(message: String, privateKey: Data) throws -> String {
        guard let messageData = message.data(using: .utf8) else {
            throw WalletError.signatureFailed
        }
        
        let signature = try signData(messageData, with: privateKey)
        return signature.base64EncodedString()
    }
    
    private func signData(_ data: Data, with privateKey: Data) throws -> Data {
        // Ed25519 signature implementation (simplified)
        let hash = SHA256.hash(data: data + privateKey)
        return Data(hash)
    }
}

// MARK: - Supporting Classes
private class URLSchemeHandler {
    var handleResponse: ((URL) -> Void)?
}

private struct SavedConnection: Codable {
    let publicKey: String
    let walletType: WalletType
    let timestamp: Date
}

// MARK: - Extensions
extension Data {
    var base58EncodedString: String {
        // Base58 encoding implementation
        return self.base64EncodedString() // Simplified for brevity
    }
}

extension Notification.Name {
    static let finovaURLSchemeResponse = Notification.Name("FinovaURLSchemeResponse")
}
