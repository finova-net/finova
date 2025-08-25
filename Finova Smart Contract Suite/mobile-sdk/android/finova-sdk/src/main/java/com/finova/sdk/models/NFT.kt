package com.finova.sdk.models

import kotlinx.serialization.Serializable
import java.math.BigDecimal
import java.time.Instant
import java.util.*

/**
 * NFT Model for Finova Network Android SDK
 * Represents various NFT types including Special Cards, Profile Badges, and Achievement NFTs
 */
@Serializable
data class NFT(
    val id: String,
    val mintAddress: String,
    val collectionId: String,
    val tokenId: Long,
    val name: String,
    val description: String,
    val imageUrl: String,
    val animationUrl: String? = null,
    val externalUrl: String? = null,
    val type: NFTType,
    val category: NFTCategory,
    val rarity: NFTRarity,
    val attributes: List<NFTAttribute>,
    val metadata: NFTMetadata,
    val owner: String,
    val creator: String,
    val price: BigDecimal? = null,
    val currency: String = "FIN",
    val isListed: Boolean = false,
    val isUsed: Boolean = false,
    val usageCount: Int = 0,
    val maxUsage: Int? = null,
    val createdAt: Instant,
    val updatedAt: Instant,
    val expiryDate: Instant? = null,
    val utility: NFTUtility? = null
) {
    val isExpired: Boolean
        get() = expiryDate?.isBefore(Instant.now()) ?: false
    
    val isUsable: Boolean
        get() = !isExpired && !isUsed && (maxUsage == null || usageCount < maxUsage)
    
    val remainingUses: Int?
        get() = maxUsage?.let { max -> (max - usageCount).coerceAtLeast(0) }
}

@Serializable
enum class NFTType {
    SPECIAL_CARD,
    PROFILE_BADGE,
    ACHIEVEMENT_NFT,
    COLLECTIBLE,
    UTILITY_NFT
}

@Serializable
enum class NFTCategory {
    // Special Cards
    MINING_BOOST,
    XP_ACCELERATOR,
    REFERRAL_POWER,
    
    // Profile Badges
    LEVEL_BADGE,
    TIER_BADGE,
    ACHIEVEMENT_BADGE,
    
    // Collectibles
    LIMITED_EDITION,
    SEASONAL,
    EVENT_EXCLUSIVE,
    
    // Utility
    ACCESS_PASS,
    MULTIPLIER,
    UNLOCK_FEATURE
}

@Serializable
enum class NFTRarity {
    COMMON(1.0, "#9CA3AF"),
    UNCOMMON(1.05, "#10B981"),
    RARE(1.10, "#3B82F6"),
    EPIC(1.20, "#8B5CF6"),
    LEGENDARY(1.35, "#F59E0B"),
    MYTHIC(1.50, "#EF4444");
    
    val multiplier: Double
    val colorHex: String
    
    constructor(multiplier: Double, colorHex: String) {
        this.multiplier = multiplier
        this.colorHex = colorHex
    }
}

@Serializable
data class NFTAttribute(
    val traitType: String,
    val value: String,
    val displayType: String? = null,
    val maxValue: String? = null
)

@Serializable
data class NFTMetadata(
    val version: String = "1.0",
    val standard: String = "Metaplex",
    val collection: CollectionInfo,
    val royalty: RoyaltyInfo,
    val properties: Map<String, Any> = emptyMap(),
    val files: List<FileInfo> = emptyList()
)

@Serializable
data class CollectionInfo(
    val name: String,
    val family: String,
    val verified: Boolean = false
)

@Serializable
data class RoyaltyInfo(
    val sellerFeeBasisPoints: Int,
    val creators: List<CreatorInfo>
)

@Serializable
data class CreatorInfo(
    val address: String,
    val share: Int,
    val verified: Boolean = false
)

@Serializable
data class FileInfo(
    val uri: String,
    val type: String,
    val cdn: Boolean = true
)

@Serializable
data class NFTUtility(
    val effects: List<UtilityEffect>,
    val duration: UtilityDuration? = null,
    val stackable: Boolean = false,
    val restrictions: List<String> = emptyList()
)

@Serializable
data class UtilityEffect(
    val type: EffectType,
    val value: Double,
    val target: String,
    val operation: OperationType = OperationType.MULTIPLY
)

@Serializable
enum class EffectType {
    MINING_RATE_BOOST,
    XP_MULTIPLIER,
    RP_BONUS,
    STAKING_APY_BOOST,
    TRANSACTION_FEE_DISCOUNT,
    DAILY_LIMIT_INCREASE,
    QUALITY_SCORE_BOOST,
    NETWORK_REGRESSION_REDUCTION
}

@Serializable
enum class OperationType {
    ADD,
    MULTIPLY,
    SET
}

@Serializable
data class UtilityDuration(
    val type: DurationType,
    val value: Long,
    val startTime: Instant? = null
) {
    val isActive: Boolean
        get() = when (type) {
            DurationType.PERMANENT -> true
            DurationType.INSTANT -> false
            else -> {
                val start = startTime ?: return false
                val now = Instant.now()
                val endTime = when (type) {
                    DurationType.MINUTES -> start.plusSeconds(value * 60)
                    DurationType.HOURS -> start.plusSeconds(value * 3600)
                    DurationType.DAYS -> start.plusSeconds(value * 86400)
                    else -> start
                }
                now.isBefore(endTime)
            }
        }
}

@Serializable
enum class DurationType {
    INSTANT,
    MINUTES,
    HOURS,
    DAYS,
    PERMANENT
}

// Special Card Templates
@Serializable
sealed class SpecialCard {
    @Serializable
    data class DoubleMining(
        val nft: NFT,
        val boostPercentage: Double = 100.0,
        val durationHours: Int = 24
    ) : SpecialCard()
    
    @Serializable
    data class TripleMining(
        val nft: NFT,
        val boostPercentage: Double = 200.0,
        val durationHours: Int = 12
    ) : SpecialCard()
    
    @Serializable
    data class MiningFrenzy(
        val nft: NFT,
        val boostPercentage: Double = 500.0,
        val durationHours: Int = 4
    ) : SpecialCard()
    
    @Serializable
    data class XPDouble(
        val nft: NFT,
        val xpMultiplier: Double = 2.0,
        val durationHours: Int = 24
    ) : SpecialCard()
    
    @Serializable
    data class ReferralBoost(
        val nft: NFT,
        val referralBonus: Double = 0.5,
        val durationDays: Int = 7
    ) : SpecialCard()
    
    @Serializable
    data class NetworkAmplifier(
        val nft: NFT,
        val tierBoost: Int = 2,
        val durationHours: Int = 24
    ) : SpecialCard()
}

// Profile Badge System
@Serializable
data class ProfileBadge(
    val nft: NFT,
    val tier: BadgeTier,
    val level: Int,
    val permanentBonuses: List<UtilityEffect>,
    val upgradeRequirements: UpgradeRequirements? = null
)

@Serializable
enum class BadgeTier {
    BRONZE,
    SILVER,
    GOLD,
    PLATINUM,
    DIAMOND,
    MYTHIC
}

@Serializable
data class UpgradeRequirements(
    val requiredXP: Long,
    val requiredFIN: BigDecimal,
    val requiredAchievements: List<String> = emptyList(),
    val timeRequirement: Long? = null
)

// Achievement NFT System
@Serializable
data class AchievementNFT(
    val nft: NFT,
    val achievementType: AchievementType,
    val unlockedAt: Instant,
    val milestone: String,
    val permanentBonus: UtilityEffect? = null
)

@Serializable
enum class AchievementType {
    FIRST_1000_USERS,
    VIRAL_CREATOR,
    NETWORK_BUILDER,
    WHALE_STAKER,
    DAILY_MINER,
    SOCIAL_BUTTERFLY,
    GUILD_MASTER,
    CONTENT_KING
}

// Marketplace Integration
@Serializable
data class MarketplaceListing(
    val id: String,
    val nftId: String,
    val sellerId: String,
    val price: BigDecimal,
    val currency: String,
    val listingType: ListingType,
    val startTime: Instant,
    val endTime: Instant? = null,
    val minBid: BigDecimal? = null,
    val buyNowPrice: BigDecimal? = null,
    val status: ListingStatus,
    val bids: List<Bid> = emptyList()
)

@Serializable
enum class ListingType {
    FIXED_PRICE,
    AUCTION,
    DUTCH_AUCTION
}

@Serializable
enum class ListingStatus {
    ACTIVE,
    SOLD,
    CANCELLED,
    EXPIRED
}

@Serializable
data class Bid(
    val bidderId: String,
    val amount: BigDecimal,
    val timestamp: Instant,
    val txHash: String? = null
)

// Usage Tracking
@Serializable
data class NFTUsageHistory(
    val nftId: String,
    val usages: List<UsageRecord>,
    val totalUsageCount: Int,
    val lastUsedAt: Instant? = null
)

@Serializable
data class UsageRecord(
    val id: String,
    val usedAt: Instant,
    val userId: String,
    val usageType: String,
    val effects: List<UtilityEffect>,
    val duration: UtilityDuration,
    val txHash: String? = null,
    val context: Map<String, Any> = emptyMap()
)

// Factory Methods for Common NFT Types
object NFTFactory {
    fun createMiningBoostCard(
        name: String,
        boostPercentage: Double,
        durationHours: Int,
        rarity: NFTRarity,
        price: BigDecimal
    ): NFT = NFT(
        id = UUID.randomUUID().toString(),
        mintAddress = "",
        collectionId = "finova-special-cards",
        tokenId = 0L,
        name = name,
        description = "Boost mining rate by ${boostPercentage}% for ${durationHours} hours",
        imageUrl = "",
        type = NFTType.SPECIAL_CARD,
        category = NFTCategory.MINING_BOOST,
        rarity = rarity,
        attributes = listOf(
            NFTAttribute("Boost Percentage", boostPercentage.toString()),
            NFTAttribute("Duration", "${durationHours} hours"),
            NFTAttribute("Category", "Mining Boost")
        ),
        metadata = NFTMetadata(
            collection = CollectionInfo("Finova Special Cards", "FinovaCards"),
            royalty = RoyaltyInfo(500, emptyList())
        ),
        owner = "",
        creator = "finova-network",
        price = price,
        createdAt = Instant.now(),
        updatedAt = Instant.now(),
        utility = NFTUtility(
            effects = listOf(
                UtilityEffect(
                    type = EffectType.MINING_RATE_BOOST,
                    value = boostPercentage / 100.0 + 1.0,
                    target = "mining_rate"
                )
            ),
            duration = UtilityDuration(DurationType.HOURS, durationHours.toLong()),
            stackable = false
        )
    )
    
    fun createProfileBadge(
        tier: BadgeTier,
        level: Int,
        permanentBonuses: List<UtilityEffect>
    ): NFT = NFT(
        id = UUID.randomUUID().toString(),
        mintAddress = "",
        collectionId = "finova-profile-badges",
        tokenId = 0L,
        name = "${tier.name} Badge Level $level",
        description = "Profile badge showing ${tier.name} tier achievement",
        imageUrl = "",
        type = NFTType.PROFILE_BADGE,
        category = NFTCategory.LEVEL_BADGE,
        rarity = when (tier) {
            BadgeTier.BRONZE -> NFTRarity.COMMON
            BadgeTier.SILVER -> NFTRarity.UNCOMMON
            BadgeTier.GOLD -> NFTRarity.RARE
            BadgeTier.PLATINUM -> NFTRarity.EPIC
            BadgeTier.DIAMOND -> NFTRarity.LEGENDARY
            BadgeTier.MYTHIC -> NFTRarity.MYTHIC
        },
        attributes = listOf(
            NFTAttribute("Tier", tier.name),
            NFTAttribute("Level", level.toString()),
            NFTAttribute("Type", "Profile Badge")
        ),
        metadata = NFTMetadata(
            collection = CollectionInfo("Finova Profile Badges", "FinovaBadges"),
            royalty = RoyaltyInfo(0, emptyList())
        ),
        owner = "",
        creator = "finova-network",
        createdAt = Instant.now(),
        updatedAt = Instant.now(),
        utility = NFTUtility(
            effects = permanentBonuses,
            duration = UtilityDuration(DurationType.PERMANENT, 0L),
            stackable = true
        )
    )
}
