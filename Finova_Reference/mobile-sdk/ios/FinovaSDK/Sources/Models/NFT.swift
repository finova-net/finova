// finova-net/finova/mobile-sdk/ios/FinovaSDK/Sources/Models/NFT.swift

import Foundation
import UIKit

// MARK: - NFT Core Models

/// Main NFT model representing all Finova NFT types
public struct NFT: Codable, Identifiable, Hashable {
    public let id: String
    public let tokenAddress: String
    public let mintAddress: String
    public let collectionId: String
    public let name: String
    public let description: String
    public let imageUrl: URL?
    public let animationUrl: URL?
    public let type: NFTType
    public let rarity: NFTRarity
    public let attributes: [NFTAttribute]
    public let creator: String
    public let owner: String
    public let price: Decimal?
    public let lastSalePrice: Decimal?
    public let marketplaceStatus: MarketplaceStatus
    public let utility: NFTUtility?
    public let createdAt: Date
    public let updatedAt: Date
    public let metadata: NFTMetadata
    
    public init(id: String, tokenAddress: String, mintAddress: String, collectionId: String,
                name: String, description: String, imageUrl: URL?, animationUrl: URL?,
                type: NFTType, rarity: NFTRarity, attributes: [NFTAttribute],
                creator: String, owner: String, price: Decimal?, lastSalePrice: Decimal?,
                marketplaceStatus: MarketplaceStatus, utility: NFTUtility?,
                createdAt: Date, updatedAt: Date, metadata: NFTMetadata) {
        self.id = id
        self.tokenAddress = tokenAddress
        self.mintAddress = mintAddress
        self.collectionId = collectionId
        self.name = name
        self.description = description
        self.imageUrl = imageUrl
        self.animationUrl = animationUrl
        self.type = type
        self.rarity = rarity
        self.attributes = attributes
        self.creator = creator
        self.owner = owner
        self.price = price
        self.lastSalePrice = lastSalePrice
        self.marketplaceStatus = marketplaceStatus
        self.utility = utility
        self.createdAt = createdAt
        self.updatedAt = updatedAt
        self.metadata = metadata
    }
}

// MARK: - NFT Type Definitions

public enum NFTType: String, Codable, CaseIterable {
    case specialCard = "special_card"
    case profileBadge = "profile_badge"
    case achievementBadge = "achievement_badge"
    case guildEmblem = "guild_emblem"
    case collectible = "collectible"
    case utilityItem = "utility_item"
    
    public var displayName: String {
        switch self {
        case .specialCard: return "Special Card"
        case .profileBadge: return "Profile Badge"
        case .achievementBadge: return "Achievement Badge"
        case .guildEmblem: return "Guild Emblem"
        case .collectible: return "Collectible"
        case .utilityItem: return "Utility Item"
        }
    }
    
    public var icon: String {
        switch self {
        case .specialCard: return "🎴"
        case .profileBadge: return "🏆"
        case .achievementBadge: return "🏅"
        case .guildEmblem: return "⚔️"
        case .collectible: return "💎"
        case .utilityItem: return "🔧"
        }
    }
}

public enum NFTRarity: String, Codable, CaseIterable {
    case common = "common"
    case uncommon = "uncommon"
    case rare = "rare"
    case epic = "epic"
    case legendary = "legendary"
    case mythic = "mythic"
    
    public var displayName: String {
        switch self {
        case .common: return "Common"
        case .uncommon: return "Uncommon"
        case .rare: return "Rare"
        case .epic: return "Epic"
        case .legendary: return "Legendary"
        case .mythic: return "Mythic"
        }
    }
    
    public var color: UIColor {
        switch self {
        case .common: return UIColor.systemGray
        case .uncommon: return UIColor.systemGreen
        case .rare: return UIColor.systemBlue
        case .epic: return UIColor.systemPurple
        case .legendary: return UIColor.systemOrange
        case .mythic: return UIColor.systemRed
        }
    }
    
    public var dropRate: Double {
        switch self {
        case .common: return 0.60
        case .uncommon: return 0.25
        case .rare: return 0.10
        case .epic: return 0.04
        case .legendary: return 0.009
        case .mythic: return 0.001
        }
    }
}

public enum MarketplaceStatus: String, Codable {
    case notListed = "not_listed"
    case listed = "listed"
    case sold = "sold"
    case auction = "auction"
    case reserved = "reserved"
}

// MARK: - Special Cards (Hamster Kombat Inspired)

public struct SpecialCard: Codable {
    public let cardId: String
    public let name: String
    public let category: CardCategory
    public let effect: CardEffect
    public let duration: TimeInterval
    public let usageCount: Int
    public let maxUsage: Int
    public let isConsumed: Bool
    public let synergyTags: [String]
    public let flavorText: String
    
    public enum CardCategory: String, Codable {
        case miningBoost = "mining_boost"
        case xpAccelerator = "xp_accelerator"
        case referralPower = "referral_power"
        case qualityEnhancer = "quality_enhancer"
        case networkAmplifier = "network_amplifier"
        case specialEvent = "special_event"
    }
    
    public struct CardEffect: Codable {
        public let type: EffectType
        public let value: Double
        public let targetScope: TargetScope
        public let stackable: Bool
        public let diminishingReturns: Bool
        
        public enum EffectType: String, Codable {
            case miningRateMultiplier = "mining_rate_multiplier"
            case xpMultiplier = "xp_multiplier"
            case referralBonusMultiplier = "referral_bonus_multiplier"
            case qualityScoreBoost = "quality_score_boost"
            case networkEffectBoost = "network_effect_boost"
            case instantReward = "instant_reward"
            case streakProtection = "streak_protection"
            case levelBoost = "level_boost"
        }
        
        public enum TargetScope: String, Codable {
            case self = "self"
            case referrals = "referrals"
            case guild = "guild"
            case network = "network"
        }
    }
    
    public var isActive: Bool {
        return !isConsumed && usageCount < maxUsage
    }
    
    public var remainingUses: Int {
        return max(0, maxUsage - usageCount)
    }
}

// MARK: - Profile Badges

public struct ProfileBadge: Codable {
    public let badgeId: String
    public let tier: BadgeTier
    public let level: Int
    public let experience: Int
    public let nextLevelExp: Int
    public let permanentBonuses: [PermanentBonus]
    public let isEvolutionary: Bool
    public let evolutionRequirements: [EvolutionRequirement]?
    
    public enum BadgeTier: String, Codable, CaseIterable {
        case bronze = "bronze"
        case silver = "silver"
        case gold = "gold"
        case platinum = "platinum"
        case diamond = "diamond"
        case mythic = "mythic"
        
        public var roman: String {
            switch self {
            case .bronze: return "I-X"
            case .silver: return "I-XV"
            case .gold: return "I-XXV"
            case .platinum: return "I-XXV"
            case .diamond: return "I-XXV"
            case .mythic: return "I+"
            }
        }
        
        public var miningMultiplier: (min: Double, max: Double) {
            switch self {
            case .bronze: return (1.0, 1.2)
            case .silver: return (1.3, 1.8)
            case .gold: return (1.9, 2.5)
            case .platinum: return (2.6, 3.2)
            case .diamond: return (3.3, 4.0)
            case .mythic: return (4.1, 5.0)
            }
        }
    }
    
    public struct PermanentBonus: Codable {
        public let type: String
        public let value: Double
        public let description: String
    }
    
    public struct EvolutionRequirement: Codable {
        public let type: String
        public let target: Int
        public let current: Int
        public let description: String
        
        public var isCompleted: Bool {
            return current >= target
        }
        
        public var progress: Double {
            return min(1.0, Double(current) / Double(target))
        }
    }
    
    public var canEvolve: Bool {
        guard isEvolutionary, let requirements = evolutionRequirements else { return false }
        return requirements.allSatisfy { $0.isCompleted }
    }
}

// MARK: - Achievement System

public struct AchievementBadge: Codable {
    public let achievementId: String
    public let category: AchievementCategory
    public let milestone: String
    public let unlockedAt: Date
    public let specialRewards: [SpecialReward]
    public let isRare: Bool
    public let globalRank: Int?
    
    public enum AchievementCategory: String, Codable {
        case pioneer = "pioneer"
        case creator = "creator"
        case socialButterfly = "social_butterfly"
        case networkBuilder = "network_builder"
        case miningExpert = "mining_expert"
        case collector = "collector"
        case guildMaster = "guild_master"
        case whaleWatcher = "whale_watcher"
    }
    
    public struct SpecialReward: Codable {
        public let type: RewardType
        public let value: String
        public let isPermanent: Bool
        
        public enum RewardType: String, Codable {
            case miningBonus = "mining_bonus"
            case xpBonus = "xp_bonus"
            case rpBonus = "rp_bonus"
            case exclusiveAccess = "exclusive_access"
            case customization = "customization"
            case title = "title"
        }
    }
}

// MARK: - NFT Attributes

public struct NFTAttribute: Codable, Hashable {
    public let traitType: String
    public let value: AttributeValue
    public let displayType: DisplayType?
    public let maxValue: Double?
    
    public enum AttributeValue: Codable, Hashable {
        case string(String)
        case number(Double)
        case boolean(Bool)
        case date(Date)
        
        public init(from decoder: Decoder) throws {
            let container = try decoder.singleValueContainer()
            
            if let stringValue = try? container.decode(String.self) {
                // Try to parse as date first
                let formatter = ISO8601DateFormatter()
                if let date = formatter.date(from: stringValue) {
                    self = .date(date)
                } else {
                    self = .string(stringValue)
                }
            } else if let numberValue = try? container.decode(Double.self) {
                self = .number(numberValue)
            } else if let boolValue = try? container.decode(Bool.self) {
                self = .boolean(boolValue)
            } else {
                throw DecodingError.typeMismatch(AttributeValue.self, DecodingError.Context(codingPath: decoder.codingPath, debugDescription: "Invalid attribute value"))
            }
        }
        
        public func encode(to encoder: Encoder) throws {
            var container = encoder.singleValueContainer()
            switch self {
            case .string(let value):
                try container.encode(value)
            case .number(let value):
                try container.encode(value)
            case .boolean(let value):
                try container.encode(value)
            case .date(let value):
                let formatter = ISO8601DateFormatter()
                try container.encode(formatter.string(from: value))
            }
        }
        
        public var displayString: String {
            switch self {
            case .string(let value): return value
            case .number(let value): return String(format: "%.2f", value)
            case .boolean(let value): return value ? "Yes" : "No"
            case .date(let value):
                let formatter = DateFormatter()
                formatter.dateStyle = .medium
                return formatter.string(from: value)
            }
        }
    }
    
    public enum DisplayType: String, Codable {
        case number = "number"
        case percentage = "percentage"
        case boostNumber = "boost_number"
        case boostPercentage = "boost_percentage"
        case date = "date"
    }
}

// MARK: - NFT Utility System

public struct NFTUtility: Codable {
    public let isUsable: Bool
    public let usageType: UsageType
    public let cooldownPeriod: TimeInterval?
    public let lastUsed: Date?
    public let remainingUses: Int?
    public let maxUses: Int?
    public let utilityData: [String: Any]
    
    public enum UsageType: String, Codable {
        case consumable = "consumable"
        case reusable = "reusable"
        case permanent = "permanent"
        case timedEffect = "timed_effect"
    }
    
    private enum CodingKeys: String, CodingKey {
        case isUsable, usageType, cooldownPeriod, lastUsed, remainingUses, maxUses, utilityData
    }
    
    public init(from decoder: Decoder) throws {
        let container = try decoder.container(keyedBy: CodingKeys.self)
        isUsable = try container.decode(Bool.self, forKey: .isUsable)
        usageType = try container.decode(UsageType.self, forKey: .usageType)
        cooldownPeriod = try container.decodeIfPresent(TimeInterval.self, forKey: .cooldownPeriod)
        lastUsed = try container.decodeIfPresent(Date.self, forKey: .lastUsed)
        remainingUses = try container.decodeIfPresent(Int.self, forKey: .remainingUses)
        maxUses = try container.decodeIfPresent(Int.self, forKey: .maxUses)
        
        // Decode utilityData as [String: Any]
        if let data = try container.decodeIfPresent(Data.self, forKey: .utilityData) {
            utilityData = (try JSONSerialization.jsonObject(with: data) as? [String: Any]) ?? [:]
        } else {
            utilityData = [:]
        }
    }
    
    public func encode(to encoder: Encoder) throws {
        var container = encoder.container(keyedBy: CodingKeys.self)
        try container.encode(isUsable, forKey: .isUsable)
        try container.encode(usageType, forKey: .usageType)
        try container.encodeIfPresent(cooldownPeriod, forKey: .cooldownPeriod)
        try container.encodeIfPresent(lastUsed, forKey: .lastUsed)
        try container.encodeIfPresent(remainingUses, forKey: .remainingUses)
        try container.encodeIfPresent(maxUses, forKey: .maxUses)
        
        // Encode utilityData
        let data = try JSONSerialization.data(withJSONObject: utilityData)
        try container.encode(data, forKey: .utilityData)
    }
    
    public var canUse: Bool {
        guard isUsable else { return false }
        
        // Check usage limits
        if let remaining = remainingUses, remaining <= 0 { return false }
        
        // Check cooldown
        if let cooldown = cooldownPeriod,
           let lastUse = lastUsed,
           Date().timeIntervalSince(lastUse) < cooldown {
            return false
        }
        
        return true
    }
    
    public var nextUsableDate: Date? {
        guard let cooldown = cooldownPeriod, let lastUse = lastUsed else { return nil }
        return lastUse.addingTimeInterval(cooldown)
    }
}

// MARK: - NFT Metadata

public struct NFTMetadata: Codable {
    public let version: String
    public let standard: String
    public let external_url: URL?
    public let background_color: String?
    public let youtube_url: URL?
    public let properties: NFTProperties?
    public let collection: NFTCollection?
    
    public struct NFTProperties: Codable {
        public let files: [NFTFile]?
        public let category: String?
        public let creators: [NFTCreator]?
    }
    
    public struct NFTFile: Codable {
        public let uri: URL
        public let type: String
        public let cdn: Bool?
    }
    
    public struct NFTCreator: Codable {
        public let address: String
        public let share: Int
        public let verified: Bool?
    }
    
    public struct NFTCollection: Codable {
        public let name: String
        public let family: String?
        public let verified: Bool?
    }
}

// MARK: - NFT Operations

public struct NFTTransfer: Codable {
    public let id: String
    public let nftId: String
    public let fromAddress: String
    public let toAddress: String
    public let transactionHash: String
    public let blockNumber: Int
    public let timestamp: Date
    public let gasUsed: Int
    public let gasPriceSOL: Decimal
    public let status: TransferStatus
    
    public enum TransferStatus: String, Codable {
        case pending = "pending"
        case confirmed = "confirmed"
        case failed = "failed"
    }
}

public struct NFTSale: Codable {
    public let id: String
    public let nftId: String
    public let seller: String
    public let buyer: String
    public let price: Decimal
    public let currency: String
    public let marketplaceFee: Decimal
    public let royaltyFee: Decimal
    public let timestamp: Date
    public let transactionHash: String
}

public struct NFTListing: Codable {
    public let id: String
    public let nftId: String
    public let seller: String
    public let price: Decimal
    public let currency: String
    public let listingType: ListingType
    public let startTime: Date
    public let endTime: Date?
    public let isActive: Bool
    public let reservePrice: Decimal?
    
    public enum ListingType: String, Codable {
        case fixedPrice = "fixed_price"
        case auction = "auction"
        case dutchAuction = "dutch_auction"
    }
}

// MARK: - NFT Collection Model

public struct NFTCollection: Codable, Identifiable {
    public let id: String
    public let name: String
    public let description: String
    public let imageUrl: URL?
    public let bannerUrl: URL?
    public let creator: String
    public let totalSupply: Int
    public let mintedSupply: Int
    public let floorPrice: Decimal?
    public let volume24h: Decimal?
    public let volumeTotal: Decimal?
    public let isVerified: Bool
    public let category: String
    public let blockchain: String
    public let contractAddress: String
    public let royalty: Double
    public let createdAt: Date
    public let stats: CollectionStats
    
    public struct CollectionStats: Codable {
        public let owners: Int
        public let sales24h: Int
        public let salesTotal: Int
        public let averagePrice: Decimal?
        public let marketCap: Decimal?
        public let listedCount: Int
        public let listedPercentage: Double
    }
    
    public var mintedPercentage: Double {
        guard totalSupply > 0 else { return 0 }
        return Double(mintedSupply) / Double(totalSupply) * 100
    }
}

// MARK: - NFT Cache Model

public struct NFTCache: Codable {
    public let nft: NFT
    public let cachedAt: Date
    public let expiresAt: Date
    
    public var isExpired: Bool {
        return Date() > expiresAt
    }
}

// MARK: - NFT Filter & Search

public struct NFTFilter {
    public var collections: [String] = []
    public var types: [NFTType] = []
    public var rarities: [NFTRarity] = []
    public var priceRange: ClosedRange<Decimal>?
    public var isListed: Bool?
    public var attributes: [String: [String]] = [:]
    public var sortBy: SortOption = .dateCreated
    public var sortOrder: SortOrder = .descending
    
    public enum SortOption: String, CaseIterable {
        case dateCreated = "date_created"
        case price = "price"
        case rarity = "rarity"
        case name = "name"
        case lastSale = "last_sale"
    }
    
    public enum SortOrder: String {
        case ascending = "asc"
        case descending = "desc"
    }
}

// MARK: - Extension Methods

extension NFT {
    /// Calculate the estimated USD value based on current market data
    public func estimatedUSDValue(exchangeRate: Decimal = 0.02) -> Decimal {
        return (price ?? lastSalePrice ?? 0) * exchangeRate
    }
    
    /// Check if NFT has specific utility
    public func hasUtility(_ type: NFTUtility.UsageType) -> Bool {
        return utility?.usageType == type
    }
    
    /// Get display-friendly rarity percentage
    public var rarityDisplayText: String {
        let percentage = rarity.dropRate * 100
        return String(format: "%.1f%% (%@)", percentage, rarity.displayName)
    }
    
    /// Check if NFT is currently usable
    public var isCurrentlyUsable: Bool {
        return utility?.canUse ?? false
    }
}

extension SpecialCard {
    /// Calculate synergy bonus with other active cards
    public func calculateSynergyBonus(with otherCards: [SpecialCard]) -> Double {
        let matchingTags = synergyTags.filter { tag in
            otherCards.contains { $0.synergyTags.contains(tag) }
        }
        
        let synergyMultiplier = 1.0 + (Double(matchingTags.count) * 0.1)
        let rarityBonus = effect.stackable ? 1.2 : 1.0
        
        return synergyMultiplier * rarityBonus
    }
}

extension ProfileBadge {
    /// Calculate current mining multiplier based on level and tier
    public var currentMiningMultiplier: Double {
        let tierRange = tier.miningMultiplier
        let levelProgress = Double(level) / 25.0 // Assuming 25 levels per tier
        return tierRange.min + (tierRange.max - tierRange.min) * min(1.0, levelProgress)
    }
    
    /// Get experience needed for next level
    public var expToNextLevel: Int {
        return nextLevelExp - experience
    }
    
    /// Get level progress percentage
    public var levelProgress: Double {
        guard nextLevelExp > 0 else { return 1.0 }
        return Double(experience) / Double(nextLevelExp)
    }
}
