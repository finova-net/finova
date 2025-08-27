// finova-net/finova/mobile-sdk/ios/FinovaSDK/Sources/Services/NFTService.swift

import Foundation
import Combine
import CryptoKit

// MARK: - NFT Models
public struct FinovaNFT: Codable, Identifiable {
    public let id: String
    public let tokenAddress: String
    public let collectionId: String
    public let name: String
    public let description: String
    public let imageUrl: String
    public let animationUrl: String?
    public let attributes: [NFTAttribute]
    public let rarity: NFTRarity
    public let category: NFTCategory
    public let owner: String
    public let creator: String
    public let price: Decimal?
    public let isListed: Bool
    public let createdAt: Date
    public let metadata: NFTMetadata
    
    public var isSpecialCard: Bool {
        return category == .specialCard
    }
}

public struct NFTAttribute: Codable {
    public let traitType: String
    public let value: String
    public let displayType: String?
    public let maxValue: Int?
}

public struct NFTMetadata: Codable {
    public let cardType: SpecialCardType?
    public let effect: CardEffect?
    public let duration: TimeInterval?
    public let boostPercentage: Double?
    public let stackable: Bool
    public let usageCount: Int?
    public let maxUsage: Int?
    public let synergies: [String]
}

public struct CardEffect: Codable {
    public let type: EffectType
    public let target: TargetType
    public let multiplier: Double
    public let duration: TimeInterval
    public let cooldown: TimeInterval?
}

public enum NFTRarity: String, Codable, CaseIterable {
    case common = "common"
    case uncommon = "uncommon"
    case rare = "rare"
    case epic = "epic"
    case legendary = "legendary"
    case mythic = "mythic"
    
    public var multiplier: Double {
        switch self {
        case .common: return 1.0
        case .uncommon: return 1.05
        case .rare: return 1.10
        case .epic: return 1.20
        case .legendary: return 1.35
        case .mythic: return 1.50
        }
    }
}

public enum NFTCategory: String, Codable {
    case specialCard = "special_card"
    case profileBadge = "profile_badge"
    case achievement = "achievement"
    case collectible = "collectible"
    case utility = "utility"
}

public enum SpecialCardType: String, Codable {
    case doubleMining = "double_mining"
    case tripleMining = "triple_mining"
    case miningFrenzy = "mining_frenzy"
    case eternalMiner = "eternal_miner"
    case xpDouble = "xp_double"
    case streakSaver = "streak_saver"
    case levelRush = "level_rush"
    case xpMagnet = "xp_magnet"
    case referralBoost = "referral_boost"
    case networkAmplifier = "network_amplifier"
    case ambassadorPass = "ambassador_pass"
    case networkKing = "network_king"
}

public enum EffectType: String, Codable {
    case miningBoost = "mining_boost"
    case xpBoost = "xp_boost"
    case referralBoost = "referral_boost"
    case stakingBoost = "staking_boost"
    case qualityBoost = "quality_boost"
}

public enum TargetType: String, Codable {
    case mining = "mining"
    case xp = "xp"
    case referral = "referral"
    case staking = "staking"
    case all = "all"
}

// MARK: - Marketplace Models
public struct MarketplaceListing: Codable, Identifiable {
    public let id: String
    public let nftId: String
    public let seller: String
    public let price: Decimal
    public let currency: String
    public let status: ListingStatus
    public let createdAt: Date
    public let expiresAt: Date?
    public let nft: FinovaNFT
}

public enum ListingStatus: String, Codable {
    case active = "active"
    case sold = "sold"
    case cancelled = "cancelled"
    case expired = "expired"
}

public struct PurchaseResult: Codable {
    public let success: Bool
    public let transactionHash: String?
    public let nft: FinovaNFT?
    public let error: String?
}

// MARK: - Request/Response Models
public struct CreateListingRequest: Codable {
    public let nftId: String
    public let price: Decimal
    public let currency: String
    public let duration: TimeInterval?
}

public struct UseSpecialCardRequest: Codable {
    public let cardId: String
    public let targetActivity: String?
    public let timestamp: Date
}

public struct UseSpecialCardResponse: Codable {
    public let success: Bool
    public let effectApplied: CardEffect?
    public let newMiningRate: Double?
    public let newXPMultiplier: Double?
    public let newRPMultiplier: Double?
    public let duration: TimeInterval?
    public let message: String?
}

// MARK: - NFT Service Protocol
public protocol NFTServiceProtocol {
    func getUserNFTs(userId: String) -> AnyPublisher<[FinovaNFT], Error>
    func getNFTDetails(tokenAddress: String) -> AnyPublisher<FinovaNFT, Error>
    func getMarketplaceListings(category: NFTCategory?, rarity: NFTRarity?, sortBy: String?, page: Int, limit: Int) -> AnyPublisher<[MarketplaceListing], Error>
    func purchaseNFT(listingId: String) -> AnyPublisher<PurchaseResult, Error>
    func createListing(request: CreateListingRequest) -> AnyPublisher<MarketplaceListing, Error>
    func cancelListing(listingId: String) -> AnyPublisher<Bool, Error>
    func useSpecialCard(request: UseSpecialCardRequest) -> AnyPublisher<UseSpecialCardResponse, Error>
    func getActiveEffects(userId: String) -> AnyPublisher<[CardEffect], Error>
    func calculateSynergyBonus(activeCards: [FinovaNFT]) -> Double
}

// MARK: - NFT Service Implementation
public class NFTService: NFTServiceProtocol {
    private let client: FinovaClient
    private let cache = NSCache<NSString, AnyObject>()
    private let cacheQueue = DispatchQueue(label: "nft.cache", qos: .utility)
    private var cancellables = Set<AnyCancellable>()
    
    // Cache configuration
    private let cacheExpiration: TimeInterval = 300 // 5 minutes
    private struct CacheEntry {
        let data: AnyObject
        let timestamp: Date
        
        var isExpired: Bool {
            Date().timeIntervalSince(timestamp) > 300
        }
    }
    
    public init(client: FinovaClient) {
        self.client = client
        setupCache()
    }
    
    private func setupCache() {
        cache.countLimit = 1000
        cache.totalCostLimit = 50 * 1024 * 1024 // 50MB
    }
    
    // MARK: - Public Methods
    
    public func getUserNFTs(userId: String) -> AnyPublisher<[FinovaNFT], Error> {
        let cacheKey = "user_nfts_\(userId)"
        
        // Check cache first
        if let cachedEntry = getCachedEntry(for: cacheKey) as? CacheEntry,
           !cachedEntry.isExpired,
           let nfts = cachedEntry.data as? [FinovaNFT] {
            return Just(nfts)
                .setFailureType(to: Error.self)
                .eraseToAnyPublisher()
        }
        
        return client.request(
            endpoint: "/api/v1/users/\(userId)/nfts",
            method: .GET,
            parameters: nil
        )
        .tryMap { [weak self] data in
            let response = try JSONDecoder().decode(APIResponse<[FinovaNFT]>.self, from: data)
            guard response.success, let nfts = response.data else {
                throw FinovaError.apiError(response.message ?? "Failed to fetch NFTs")
            }
            
            // Cache the result
            self?.setCacheEntry(for: cacheKey, data: nfts as AnyObject)
            return nfts
        }
        .receive(on: DispatchQueue.main)
        .eraseToAnyPublisher()
    }
    
    public func getNFTDetails(tokenAddress: String) -> AnyPublisher<FinovaNFT, Error> {
        let cacheKey = "nft_details_\(tokenAddress)"
        
        if let cachedEntry = getCachedEntry(for: cacheKey) as? CacheEntry,
           !cachedEntry.isExpired,
           let nft = cachedEntry.data as? FinovaNFT {
            return Just(nft)
                .setFailureType(to: Error.self)
                .eraseToAnyPublisher()
        }
        
        return client.request(
            endpoint: "/api/v1/nfts/\(tokenAddress)",
            method: .GET,
            parameters: nil
        )
        .tryMap { [weak self] data in
            let response = try JSONDecoder().decode(APIResponse<FinovaNFT>.self, from: data)
            guard response.success, let nft = response.data else {
                throw FinovaError.apiError(response.message ?? "Failed to fetch NFT details")
            }
            
            self?.setCacheEntry(for: cacheKey, data: nft as AnyObject)
            return nft
        }
        .receive(on: DispatchQueue.main)
        .eraseToAnyPublisher()
    }
    
    public func getMarketplaceListings(
        category: NFTCategory? = nil,
        rarity: NFTRarity? = nil,
        sortBy: String? = nil,
        page: Int = 1,
        limit: Int = 20
    ) -> AnyPublisher<[MarketplaceListing], Error> {
        
        var parameters: [String: Any] = [
            "page": page,
            "limit": limit
        ]
        
        if let category = category {
            parameters["category"] = category.rawValue
        }
        if let rarity = rarity {
            parameters["rarity"] = rarity.rawValue
        }
        if let sortBy = sortBy {
            parameters["sort"] = sortBy
        }
        
        return client.request(
            endpoint: "/api/v1/marketplace/listings",
            method: .GET,
            parameters: parameters
        )
        .tryMap { data in
            let response = try JSONDecoder().decode(APIResponse<[MarketplaceListing]>.self, from: data)
            guard response.success, let listings = response.data else {
                throw FinovaError.apiError(response.message ?? "Failed to fetch marketplace listings")
            }
            return listings
        }
        .receive(on: DispatchQueue.main)
        .eraseToAnyPublisher()
    }
    
    public func purchaseNFT(listingId: String) -> AnyPublisher<PurchaseResult, Error> {
        let parameters = ["listingId": listingId]
        
        return client.request(
            endpoint: "/api/v1/marketplace/purchase",
            method: .POST,
            parameters: parameters
        )
        .tryMap { [weak self] data in
            let response = try JSONDecoder().decode(APIResponse<PurchaseResult>.self, from: data)
            guard response.success, let result = response.data else {
                throw FinovaError.apiError(response.message ?? "Purchase failed")
            }
            
            // Clear user NFTs cache on successful purchase
            if result.success, let userId = self?.client.currentUserId {
                self?.clearCachedEntry(for: "user_nfts_\(userId)")
            }
            
            return result
        }
        .receive(on: DispatchQueue.main)
        .eraseToAnyPublisher()
    }
    
    public func createListing(request: CreateListingRequest) -> AnyPublisher<MarketplaceListing, Error> {
        return client.request(
            endpoint: "/api/v1/marketplace/listings",
            method: .POST,
            parameters: try! JSONSerialization.jsonObject(with: JSONEncoder().encode(request)) as? [String: Any]
        )
        .tryMap { [weak self] data in
            let response = try JSONDecoder().decode(APIResponse<MarketplaceListing>.self, from: data)
            guard response.success, let listing = response.data else {
                throw FinovaError.apiError(response.message ?? "Failed to create listing")
            }
            
            // Clear user NFTs cache
            if let userId = self?.client.currentUserId {
                self?.clearCachedEntry(for: "user_nfts_\(userId)")
            }
            
            return listing
        }
        .receive(on: DispatchQueue.main)
        .eraseToAnyPublisher()
    }
    
    public func cancelListing(listingId: String) -> AnyPublisher<Bool, Error> {
        return client.request(
            endpoint: "/api/v1/marketplace/listings/\(listingId)/cancel",
            method: .POST,
            parameters: nil
        )
        .tryMap { data in
            let response = try JSONDecoder().decode(APIResponse<Bool>.self, from: data)
            guard response.success, let success = response.data else {
                throw FinovaError.apiError(response.message ?? "Failed to cancel listing")
            }
            return success
        }
        .receive(on: DispatchQueue.main)
        .eraseToAnyPublisher()
    }
    
    public func useSpecialCard(request: UseSpecialCardRequest) -> AnyPublisher<UseSpecialCardResponse, Error> {
        return client.request(
            endpoint: "/api/v1/nfts/use-card",
            method: .POST,
            parameters: try! JSONSerialization.jsonObject(with: JSONEncoder().encode(request)) as? [String: Any]
        )
        .tryMap { [weak self] data in
            let response = try JSONDecoder().decode(APIResponse<UseSpecialCardResponse>.self, from: data)
            guard response.success, let result = response.data else {
                throw FinovaError.apiError(response.message ?? "Failed to use special card")
            }
            
            // Clear relevant caches
            if let userId = self?.client.currentUserId {
                self?.clearCachedEntry(for: "user_nfts_\(userId)")
                self?.clearCachedEntry(for: "active_effects_\(userId)")
            }
            
            return result
        }
        .receive(on: DispatchQueue.main)
        .eraseToAnyPublisher()
    }
    
    public func getActiveEffects(userId: String) -> AnyPublisher<[CardEffect], Error> {
        let cacheKey = "active_effects_\(userId)"
        
        if let cachedEntry = getCachedEntry(for: cacheKey) as? CacheEntry,
           !cachedEntry.isExpired,
           let effects = cachedEntry.data as? [CardEffect] {
            return Just(effects)
                .setFailureType(to: Error.self)
                .eraseToAnyPublisher()
        }
        
        return client.request(
            endpoint: "/api/v1/users/\(userId)/active-effects",
            method: .GET,
            parameters: nil
        )
        .tryMap { [weak self] data in
            let response = try JSONDecoder().decode(APIResponse<[CardEffect]>.self, from: data)
            guard response.success, let effects = response.data else {
                throw FinovaError.apiError(response.message ?? "Failed to fetch active effects")
            }
            
            self?.setCacheEntry(for: cacheKey, data: effects as AnyObject)
            return effects
        }
        .receive(on: DispatchQueue.main)
        .eraseToAnyPublisher()
    }
    
    public func calculateSynergyBonus(activeCards: [FinovaNFT]) -> Double {
        guard !activeCards.isEmpty else { return 1.0 }
        
        var synergyMultiplier = 1.0
        let cardCount = activeCards.count
        let rarityBonus = activeCards.map { $0.rarity.multiplier }.max() ?? 1.0
        
        // Base synergy bonus for multiple cards
        synergyMultiplier += Double(cardCount) * 0.1
        
        // Rarity bonus
        synergyMultiplier += (rarityBonus - 1.0)
        
        // Category matching bonus
        let categories = Set(activeCards.map { $0.category })
        if categories.count == 1 {
            // Same category bonus
            synergyMultiplier += 0.15
        } else if categories.count >= 3 {
            // Diverse category bonus
            synergyMultiplier += 0.30
        }
        
        // Special card type synergies
        let cardTypes = activeCards.compactMap { $0.metadata.cardType }
        if cardTypes.contains(.doubleMining) && cardTypes.contains(.xpDouble) {
            synergyMultiplier += 0.25 // Mining + XP synergy
        }
        if cardTypes.contains(.referralBoost) && cardTypes.contains(.networkAmplifier) {
            synergyMultiplier += 0.20 // Referral synergy
        }
        
        return min(synergyMultiplier, 3.0) // Cap at 3x
    }
    
    // MARK: - Helper Methods
    
    private func getCachedEntry(for key: String) -> AnyObject? {
        return cacheQueue.sync {
            return cache.object(forKey: NSString(string: key))
        }
    }
    
    private func setCacheEntry(for key: String, data: AnyObject) {
        cacheQueue.async { [weak self] in
            let entry = CacheEntry(data: data, timestamp: Date())
            self?.cache.setObject(entry as AnyObject, forKey: NSString(string: key))
        }
    }
    
    private func clearCachedEntry(for key: String) {
        cacheQueue.async { [weak self] in
            self?.cache.removeObject(forKey: NSString(string: key))
        }
    }
    
    // MARK: - Card Effect Utilities
    
    public func getCardEffectDescription(_ effect: CardEffect) -> String {
        let multiplierText = effect.multiplier > 1 ? "+\(Int((effect.multiplier - 1) * 100))%" : "\(Int(effect.multiplier * 100))%"
        let durationText = formatDuration(effect.duration)
        
        switch effect.type {
        case .miningBoost:
            return "Mining Rate \(multiplierText) for \(durationText)"
        case .xpBoost:
            return "XP Gain \(multiplierText) for \(durationText)"
        case .referralBoost:
            return "Referral Rewards \(multiplierText) for \(durationText)"
        case .stakingBoost:
            return "Staking APY \(multiplierText) for \(durationText)"
        case .qualityBoost:
            return "Content Quality Score \(multiplierText) for \(durationText)"
        }
    }
    
    private func formatDuration(_ duration: TimeInterval) -> String {
        let hours = Int(duration / 3600)
        let days = hours / 24
        
        if days > 0 {
            return "\(days) day\(days > 1 ? "s" : "")"
        } else if hours > 0 {
            return "\(hours) hour\(hours > 1 ? "s" : "")"
        } else {
            let minutes = Int(duration / 60)
            return "\(minutes) minute\(minutes > 1 ? "s" : "")"
        }
    }
    
    public func isCardUsable(_ nft: FinovaNFT) -> Bool {
        guard nft.isSpecialCard,
              let metadata = nft.metadata as NFTMetadata?,
              let maxUsage = metadata.maxUsage,
              let usageCount = metadata.usageCount else {
            return false
        }
        
        return usageCount < maxUsage
    }
    
    public func getRemainingUses(_ nft: FinovaNFT) -> Int? {
        guard let metadata = nft.metadata as NFTMetadata?,
              let maxUsage = metadata.maxUsage,
              let usageCount = metadata.usageCount else {
            return nil
        }
        
        return max(0, maxUsage - usageCount)
    }
}

// MARK: - Extensions

extension NFTService {
    public func searchNFTs(query: String, filters: [String: Any] = [:]) -> AnyPublisher<[FinovaNFT], Error> {
        var parameters = filters
        parameters["q"] = query
        
        return client.request(
            endpoint: "/api/v1/nfts/search",
            method: .GET,
            parameters: parameters
        )
        .tryMap { data in
            let response = try JSONDecoder().decode(APIResponse<[FinovaNFT]>.self, from: data)
            guard response.success, let nfts = response.data else {
                throw FinovaError.apiError(response.message ?? "Search failed")
            }
            return nfts
        }
        .receive(on: DispatchQueue.main)
        .eraseToAnyPublisher()
    }
    
    public func getCollectionStats(collectionId: String) -> AnyPublisher<CollectionStats, Error> {
        return client.request(
            endpoint: "/api/v1/collections/\(collectionId)/stats",
            method: .GET,
            parameters: nil
        )
        .tryMap { data in
            let response = try JSONDecoder().decode(APIResponse<CollectionStats>.self, from: data)
            guard response.success, let stats = response.data else {
                throw FinovaError.apiError(response.message ?? "Failed to fetch collection stats")
            }
            return stats
        }
        .receive(on: DispatchQueue.main)
        .eraseToAnyPublisher()
    }
}

// MARK: - Supporting Models

public struct CollectionStats: Codable {
    public let totalSupply: Int
    public let floorPrice: Decimal?
    public let volume24h: Decimal
    public let owners: Int
    public let listings: Int
}

// MARK: - API Response Model
private struct APIResponse<T: Codable>: Codable {
    let success: Bool
    let data: T?
    let message: String?
    let error: String?
}
