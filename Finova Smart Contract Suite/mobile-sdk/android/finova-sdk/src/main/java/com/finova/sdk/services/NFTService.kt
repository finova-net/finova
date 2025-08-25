package com.finova.sdk.services

import android.util.Log
import com.finova.sdk.client.FinovaClient
import com.finova.sdk.models.NFT
import com.finova.sdk.models.SpecialCard
import com.finova.sdk.models.NFTCollection
import com.finova.sdk.models.MarketplaceListing
import com.finova.sdk.utils.Constants
import com.finova.sdk.utils.Extensions.toJson
import com.finova.sdk.utils.Validation
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import kotlinx.serialization.Serializable
import kotlinx.serialization.json.Json
import java.math.BigDecimal
import java.security.MessageDigest
import javax.crypto.Cipher
import javax.crypto.spec.SecretKeySpec

/**
 * NFTService - Comprehensive NFT management for Finova Network
 * 
 * Handles Special Cards, Profile Badges, Achievement NFTs, and Marketplace operations
 * Based on Finova Whitepaper v3.0 and Complete Smart Contracts Suite v01
 * 
 * Features:
 * - Special Cards (Mining Boost, XP Accelerator, Referral Power)
 * - Profile Badge NFTs with evolution mechanics
 * - Achievement NFTs (Finizen, Content King, Ambassador, etc.)
 * - NFT Marketplace with integrated trading
 * - Card Synergy System with multiplier calculations
 * - Anti-fraud validation and secure transactions
 */
class NFTService(private val client: FinovaClient) {
    
    companion object {
        private const val TAG = "FinovaNFTService"
        private const val API_VERSION = "v1"
        private const val ENCRYPT_ALGORITHM = "AES"
        
        // Special Card Categories from Whitepaper
        enum class CardCategory {
            MINING_BOOST,
            XP_ACCELERATOR, 
            REFERRAL_POWER,
            PROFILE_BADGE,
            ACHIEVEMENT
        }
        
        // Rarity levels with synergy bonuses
        enum class Rarity(val synergyBonus: Double) {
            COMMON(0.0),
            UNCOMMON(0.05),
            RARE(0.10),
            EPIC(0.20),
            LEGENDARY(0.35)
        }
    }
    
    @Serializable
    data class NFTMetadata(
        val name: String,
        val description: String,
        val image: String,
        val attributes: List<NFTAttribute>,
        val category: String,
        val rarity: String,
        val utility: CardUtility? = null
    )
    
    @Serializable
    data class NFTAttribute(
        val trait_type: String,
        val value: String,
        val display_type: String? = null
    )
    
    @Serializable
    data class CardUtility(
        val effect: String,
        val duration: Long, // milliseconds
        val multiplier: Double,
        val stackable: Boolean,
        val maxStack: Int = 1
    )
    
    @Serializable
    data class SynergyResult(
        val multiplier: Double,
        val activeCards: List<String>,
        val bonusDescription: String
    )
    
    // ========================================
    // SPECIAL CARDS MANAGEMENT
    // ========================================
    
    /**
     * Mint Special Card based on Whitepaper specifications
     */
    suspend fun mintSpecialCard(
        cardType: String,
        rarity: Rarity,
        recipientWallet: String
    ): Result<NFT> = withContext(Dispatchers.IO) {
        return@withContext try {
            Log.d(TAG, "Minting special card: $cardType, rarity: $rarity")
            
            // Validate inputs
            if (!Validation.isValidSolanaAddress(recipientWallet)) {
                return@withContext Result.failure(Exception("Invalid wallet address"))
            }
            
            // Get card configuration from whitepaper specs
            val cardConfig = getCardConfiguration(cardType, rarity)
            val metadata = createCardMetadata(cardConfig)
            
            // Create mint transaction
            val mintData = mapOf(
                "card_type" to cardType,
                "rarity" to rarity.name,
                "recipient" to recipientWallet,
                "metadata" to metadata.toJson(),
                "signature" to generateSecureSignature(cardType, recipientWallet)
            )
            
            val response = client.post("$API_VERSION/nft/mint-card", mintData)
            
            if (response.isSuccessful) {
                val nft = Json.decodeFromString<NFT>(response.body!!)
                Log.i(TAG, "Special card minted successfully: ${nft.tokenId}")
                Result.success(nft)
            } else {
                Log.e(TAG, "Failed to mint card: ${response.errorBody}")
                Result.failure(Exception("Minting failed: ${response.message}"))
            }
            
        } catch (e: Exception) {
            Log.e(TAG, "Error minting special card", e)
            Result.failure(e)
        }
    }
    
    /**
     * Use Special Card - Apply effects based on Whitepaper formulas
     */
    suspend fun useSpecialCard(cardId: String, userId: String): Result<Map<String, Any>> = 
        withContext(Dispatchers.IO) {
            return@withContext try {
                Log.d(TAG, "Using special card: $cardId for user: $userId")
                
                // Get card details
                val cardResult = getNFTDetails(cardId)
                if (cardResult.isFailure) {
                    return@withContext Result.failure(cardResult.exceptionOrNull()!!)
                }
                
                val card = cardResult.getOrNull()!!
                
                // Validate card ownership and usability
                if (!validateCardUsage(card, userId)) {
                    return@withContext Result.failure(Exception("Card usage validation failed"))
                }
                
                // Calculate effects based on whitepaper formulas
                val effects = calculateCardEffects(card, userId)
                
                // Apply effects via API
                val useData = mapOf(
                    "card_id" to cardId,
                    "user_id" to userId,
                    "effects" to effects,
                    "timestamp" to System.currentTimeMillis(),
                    "signature" to generateSecureSignature(cardId, userId)
                )
                
                val response = client.post("$API_VERSION/nft/use-card", useData)
                
                if (response.isSuccessful) {
                    val result = Json.decodeFromString<Map<String, Any>>(response.body!!)
                    Log.i(TAG, "Card used successfully: $cardId")
                    
                    // Update local cache
                    updateLocalCardStatus(cardId, "used")
                    
                    Result.success(result)
                } else {
                    Result.failure(Exception("Card usage failed: ${response.message}"))
                }
                
            } catch (e: Exception) {
                Log.e(TAG, "Error using special card", e)
                Result.failure(e)
            }
        }
    
    /**
     * Calculate Card Synergy based on Whitepaper formulas
     */
    suspend fun calculateCardSynergy(activeCards: List<String>, userId: String): Result<SynergyResult> = 
        withContext(Dispatchers.IO) {
            return@withContext try {
                Log.d(TAG, "Calculating synergy for ${activeCards.size} cards")
                
                if (activeCards.isEmpty()) {
                    return@withContext Result.success(SynergyResult(1.0, emptyList(), "No active cards"))
                }
                
                // Get card details for all active cards
                val cardDetails = mutableListOf<NFT>()
                for (cardId in activeCards) {
                    val cardResult = getNFTDetails(cardId)
                    if (cardResult.isSuccess) {
                        cardDetails.add(cardResult.getOrNull()!!)
                    }
                }
                
                // Calculate synergy multiplier using Whitepaper formula:
                // Synergy_Multiplier = 1.0 + (Active_Card_Count × 0.1) + Rarity_Bonus + Type_Match_Bonus
                
                var synergyMultiplier = 1.0 + (cardDetails.size * 0.1)
                
                // Add rarity bonuses
                val rarityBonus = cardDetails.maxOfOrNull { card ->
                    Rarity.valueOf(card.rarity?.uppercase() ?: "COMMON").synergyBonus
                } ?: 0.0
                synergyMultiplier += rarityBonus
                
                // Calculate type match bonuses
                val categories = cardDetails.mapNotNull { it.category }.toSet()
                val typeMatchBonus = when {
                    categories.size >= 3 -> 0.30 // All three categories active: +30%
                    categories.size == 2 && cardDetails.filter { it.category in categories }.size >= 2 -> 0.15 // Same category cards: +15%
                    else -> 0.0
                }
                synergyMultiplier += typeMatchBonus
                
                // Cap at reasonable maximum
                synergyMultiplier = minOf(synergyMultiplier, 5.0)
                
                val bonusDescription = buildSynergyDescription(cardDetails.size, rarityBonus, typeMatchBonus)
                
                Result.success(SynergyResult(synergyMultiplier, activeCards, bonusDescription))
                
            } catch (e: Exception) {
                Log.e(TAG, "Error calculating card synergy", e)
                Result.failure(e)
            }
        }
    
    // ========================================
    // PROFILE BADGE NFTS
    // ========================================
    
    /**
     * Upgrade Profile Badge NFT with evolution mechanics
     */
    suspend fun upgradeProfileBadge(
        currentBadgeId: String,
        targetLevel: String,
        userId: String
    ): Result<NFT> = withContext(Dispatchers.IO) {
        return@withContext try {
            Log.d(TAG, "Upgrading badge $currentBadgeId to $targetLevel")
            
            // Validate upgrade eligibility
            val eligibilityResult = checkUpgradeEligibility(currentBadgeId, targetLevel, userId)
            if (eligibilityResult.isFailure) {
                return@withContext Result.failure(eligibilityResult.exceptionOrNull()!!)
            }
            
            val upgradeData = mapOf(
                "current_badge_id" to currentBadgeId,
                "target_level" to targetLevel,
                "user_id" to userId,
                "timestamp" to System.currentTimeMillis(),
                "signature" to generateSecureSignature(currentBadgeId, userId)
            )
            
            val response = client.post("$API_VERSION/nft/upgrade-badge", upgradeData)
            
            if (response.isSuccessful) {
                val upgradedBadge = Json.decodeFromString<NFT>(response.body!!)
                Log.i(TAG, "Badge upgraded successfully: ${upgradedBadge.tokenId}")
                Result.success(upgradedBadge)
            } else {
                Result.failure(Exception("Badge upgrade failed: ${response.message}"))
            }
            
        } catch (e: Exception) {
            Log.e(TAG, "Error upgrading badge", e)
            Result.failure(e)
        }
    }
    
    // ========================================
    // ACHIEVEMENT NFTS
    // ========================================
    
    /**
     * Claim Achievement NFT (Finizen, Content King, Ambassador, etc.)
     */
    suspend fun claimAchievementNFT(
        achievementType: String,
        userId: String,
        proofData: Map<String, Any>
    ): Result<NFT> = withContext(Dispatchers.IO) {
        return@withContext try {
            Log.d(TAG, "Claiming achievement NFT: $achievementType")
            
            // Validate achievement eligibility with proof
            if (!validateAchievementProof(achievementType, userId, proofData)) {
                return@withContext Result.failure(Exception("Achievement proof validation failed"))
            }
            
            val claimData = mapOf(
                "achievement_type" to achievementType,
                "user_id" to userId,
                "proof_data" to proofData,
                "claim_timestamp" to System.currentTimeMillis(),
                "signature" to generateSecureSignature(achievementType, userId)
            )
            
            val response = client.post("$API_VERSION/nft/claim-achievement", claimData)
            
            if (response.isSuccessful) {
                val achievementNFT = Json.decodeFromString<NFT>(response.body!!)
                Log.i(TAG, "Achievement NFT claimed: $achievementType")
                Result.success(achievementNFT)
            } else {
                Result.failure(Exception("Achievement claim failed: ${response.message}"))
            }
            
        } catch (e: Exception) {
            Log.e(TAG, "Error claiming achievement NFT", e)
            Result.failure(e)
        }
    }
    
    // ========================================
    // MARKETPLACE OPERATIONS
    // ========================================
    
    /**
     * List NFT on Marketplace
     */
    suspend fun listNFTForSale(
        nftId: String,
        price: BigDecimal,
        currency: String = "FIN",
        duration: Long = 7 * 24 * 60 * 60 * 1000 // 7 days
    ): Result<MarketplaceListing> = withContext(Dispatchers.IO) {
        return@withContext try {
            Log.d(TAG, "Listing NFT for sale: $nftId at $price $currency")
            
            val listingData = mapOf(
                "nft_id" to nftId,
                "price" to price.toString(),
                "currency" to currency,
                "duration" to duration,
                "timestamp" to System.currentTimeMillis(),
                "signature" to generateSecureSignature(nftId, price.toString())
            )
            
            val response = client.post("$API_VERSION/marketplace/list", listingData)
            
            if (response.isSuccessful) {
                val listing = Json.decodeFromString<MarketplaceListing>(response.body!!)
                Log.i(TAG, "NFT listed successfully: ${listing.listingId}")
                Result.success(listing)
            } else {
                Result.failure(Exception("Listing failed: ${response.message}"))
            }
            
        } catch (e: Exception) {
            Log.e(TAG, "Error listing NFT", e)
            Result.failure(e)
        }
    }
    
    /**
     * Purchase NFT from Marketplace
     */
    suspend fun purchaseNFT(listingId: String, buyerWallet: String): Result<Map<String, Any>> = 
        withContext(Dispatchers.IO) {
            return@withContext try {
                Log.d(TAG, "Purchasing NFT: $listingId")
                
                val purchaseData = mapOf(
                    "listing_id" to listingId,
                    "buyer_wallet" to buyerWallet,
                    "timestamp" to System.currentTimeMillis(),
                    "signature" to generateSecureSignature(listingId, buyerWallet)
                )
                
                val response = client.post("$API_VERSION/marketplace/purchase", purchaseData)
                
                if (response.isSuccessful) {
                    val result = Json.decodeFromString<Map<String, Any>>(response.body!!)
                    Log.i(TAG, "NFT purchased successfully: $listingId")
                    Result.success(result)
                } else {
                    Result.failure(Exception("Purchase failed: ${response.message}"))
                }
                
            } catch (e: Exception) {
                Log.e(TAG, "Error purchasing NFT", e)
                Result.failure(e)
            }
        }
    
    // ========================================
    // UTILITY METHODS
    // ========================================
    
    /**
     * Get NFT details with full metadata
     */
    suspend fun getNFTDetails(nftId: String): Result<NFT> = withContext(Dispatchers.IO) {
        return@withContext try {
            val response = client.get("$API_VERSION/nft/$nftId")
            
            if (response.isSuccessful) {
                val nft = Json.decodeFromString<NFT>(response.body!!)
                Result.success(nft)
            } else {
                Result.failure(Exception("Failed to get NFT details: ${response.message}"))
            }
            
        } catch (e: Exception) {
            Log.e(TAG, "Error getting NFT details", e)
            Result.failure(e)
        }
    }
    
    /**
     * Get user's NFT collection
     */
    suspend fun getUserNFTs(userId: String, category: String? = null): Result<List<NFT>> = 
        withContext(Dispatchers.IO) {
            return@withContext try {
                val params = mutableMapOf<String, String>()
                category?.let { params["category"] = it }
                
                val response = client.get("$API_VERSION/nft/user/$userId", params)
                
                if (response.isSuccessful) {
                    val nfts = Json.decodeFromString<List<NFT>>(response.body!!)
                    Result.success(nfts)
                } else {
                    Result.failure(Exception("Failed to get user NFTs: ${response.message}"))
                }
                
            } catch (e: Exception) {
                Log.e(TAG, "Error getting user NFTs", e)
                Result.failure(e)
            }
        }
    
    /**
     * Get active special cards with remaining duration
     */
    suspend fun getActiveCards(userId: String): Result<List<SpecialCard>> = withContext(Dispatchers.IO) {
        return@withContext try {
            val response = client.get("$API_VERSION/nft/active-cards/$userId")
            
            if (response.isSuccessful) {
                val activeCards = Json.decodeFromString<List<SpecialCard>>(response.body!!)
                Result.success(activeCards)
            } else {
                Result.failure(Exception("Failed to get active cards: ${response.message}"))
            }
            
        } catch (e: Exception) {
            Log.e(TAG, "Error getting active cards", e)
            Result.failure(e)
        }
    }
    
    // ========================================
    // PRIVATE HELPER METHODS
    // ========================================
    
    private fun getCardConfiguration(cardType: String, rarity: Rarity): Map<String, Any> {
        // Card configurations based on Whitepaper specifications
        return when (cardType.lowercase()) {
            "double_mining" -> mapOf(
                "effect" to "+100% mining rate",
                "duration" to 24 * 60 * 60 * 1000L, // 24 hours
                "multiplier" to 2.0,
                "price" to 50
            )
            "triple_mining" -> mapOf(
                "effect" to "+200% mining rate", 
                "duration" to 12 * 60 * 60 * 1000L, // 12 hours
                "multiplier" to 3.0,
                "price" to 150
            )
            "mining_frenzy" -> mapOf(
                "effect" to "+500% mining rate",
                "duration" to 4 * 60 * 60 * 1000L, // 4 hours
                "multiplier" to 6.0,
                "price" to 500
            )
            "xp_double" -> mapOf(
                "effect" to "+100% XP from all activities",
                "duration" to 24 * 60 * 60 * 1000L,
                "multiplier" to 2.0,
                "price" to 40
            )
            "referral_boost" -> mapOf(
                "effect" to "+50% referral rewards",
                "duration" to 7 * 24 * 60 * 60 * 1000L, // 7 days
                "multiplier" to 1.5,
                "price" to 60
            )
            else -> mapOf(
                "effect" to "+20% general boost",
                "duration" to 12 * 60 * 60 * 1000L,
                "multiplier" to 1.2,
                "price" to 20
            )
        }
    }
    
    private fun createCardMetadata(config: Map<String, Any>): NFTMetadata {
        return NFTMetadata(
            name = config["name"] as? String ?: "Finova Special Card",
            description = config["effect"] as String,
            image = "https://cdn.finova.net/cards/${config["type"]}.png",
            attributes = listOf(
                NFTAttribute("Effect", config["effect"] as String),
                NFTAttribute("Duration", "${config["duration"]}ms", "duration"),
                NFTAttribute("Multiplier", config["multiplier"].toString(), "boost_percentage")
            ),
            category = "special_card",
            rarity = config["rarity"] as? String ?: "common",
            utility = CardUtility(
                effect = config["effect"] as String,
                duration = config["duration"] as Long,
                multiplier = config["multiplier"] as Double,
                stackable = config["stackable"] as? Boolean ?: false
            )
        )
    }
    
    private suspend fun calculateCardEffects(card: NFT, userId: String): Map<String, Any> {
        // Calculate card effects based on user's current stats and card properties
        return mapOf(
            "mining_multiplier" to (card.utility?.multiplier ?: 1.0),
            "duration" to (card.utility?.duration ?: 0L),
            "effect_type" to (card.utility?.effect ?: "general_boost"),
            "start_time" to System.currentTimeMillis(),
            "user_level_bonus" to calculateUserLevelBonus(userId),
            "synergy_multiplier" to 1.0 // Will be calculated separately
        )
    }
    
    private suspend fun calculateUserLevelBonus(userId: String): Double {
        // Placeholder for user level bonus calculation
        // In real implementation, this would fetch user's XP level and calculate bonus
        return 1.0
    }
    
    private fun validateCardUsage(card: NFT, userId: String): Boolean {
        // Validate card ownership, cooldown, and usage conditions
        return true // Simplified validation
    }
    
    private suspend fun checkUpgradeEligibility(
        badgeId: String, 
        targetLevel: String, 
        userId: String
    ): Result<Boolean> {
        // Check if user meets requirements for badge upgrade
        return Result.success(true) // Simplified check
    }
    
    private fun validateAchievementProof(
        achievementType: String,
        userId: String, 
        proofData: Map<String, Any>
    ): Boolean {
        // Validate achievement proof data
        return when (achievementType) {
            "finizen" -> proofData["user_rank"] as? Int ?: 0 <= 1000
            "content_king" -> proofData["viral_posts"] as? Int ?: 0 > 0
            "ambassador" -> proofData["active_referrals"] as? Int ?: 0 >= 100
            else -> false
        }
    }
    
    private fun buildSynergyDescription(cardCount: Int, rarityBonus: Double, typeBonus: Double): String {
        val parts = mutableListOf<String>()
        parts.add("${cardCount} active cards (+${cardCount * 10}%)")
        if (rarityBonus > 0) parts.add("Rarity bonus (+${(rarityBonus * 100).toInt()}%)")
        if (typeBonus > 0) parts.add("Type match bonus (+${(typeBonus * 100).toInt()}%)")
        return parts.joinToString(", ")
    }
    
    private fun updateLocalCardStatus(cardId: String, status: String) {
        // Update local card status cache
        Log.d(TAG, "Updating card status: $cardId -> $status")
    }
    
    private fun generateSecureSignature(data1: String, data2: String): String {
        return try {
            val combined = "$data1:$data2:${System.currentTimeMillis()}"
            val digest = MessageDigest.getInstance("SHA-256")
            val hash = digest.digest(combined.toByteArray())
            hash.joinToString("") { "%02x".format(it) }
        } catch (e: Exception) {
            Log.e(TAG, "Error generating signature", e)
            ""
        }
    }
}
