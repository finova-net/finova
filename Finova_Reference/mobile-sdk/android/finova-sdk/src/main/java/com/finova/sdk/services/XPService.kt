package com.finova.sdk.services

import android.content.Context
import com.finova.sdk.models.XP
import com.finova.sdk.models.User
import com.finova.sdk.utils.Constants
import com.finova.sdk.utils.Validation
import com.finova.sdk.client.FinovaClient
import kotlinx.coroutines.*
import kotlinx.serialization.Serializable
import kotlinx.serialization.json.Json
import java.math.BigDecimal
import java.math.MathContext
import java.math.RoundingMode
import kotlin.math.*

/**
 * XPService - Manages Experience Points system for Finova Network
 * Implements Hamster Kombat-inspired progression with exponential regression
 */
class XPService(
    private val context: Context,
    private val client: FinovaClient
) {
    private val scope = CoroutineScope(Dispatchers.IO + SupervisorJob())
    private val json = Json { ignoreUnknownKeys = true }
    
    companion object {
        private const val TAG = "XPService"
        private const val XP_CACHE_KEY = "finova_xp_cache"
        private const val LEVEL_CACHE_KEY = "finova_level_cache"
        
        // XP Constants from whitepaper
        private const val MAX_LEVEL = 200
        private const val LEVEL_PROGRESSION_FACTOR = 0.01
        private const val VIRAL_THRESHOLD = 1000
        private const val MAX_QUALITY_MULTIPLIER = 2.0
        private const val MIN_QUALITY_MULTIPLIER = 0.5
    }
    
    @Serializable
    data class XPActivity(
        val type: ActivityType,
        val platform: Platform,
        val content: String,
        val metadata: Map<String, String> = emptyMap(),
        val timestamp: Long = System.currentTimeMillis()
    )
    
    @Serializable
    enum class ActivityType(
        val baseXP: Int,
        val dailyLimit: Int,
        val description: String
    ) {
        ORIGINAL_POST(50, Int.MAX_VALUE, "Original text post"),
        PHOTO_POST(75, 20, "Photo/image post"),
        VIDEO_CONTENT(150, 10, "Video content"),
        STORY_STATUS(25, 50, "Story/status update"),
        MEANINGFUL_COMMENT(25, 100, "Quality comment"),
        LIKE_REACT(5, 200, "Like/reaction"),
        SHARE_REPOST(15, 50, "Share/repost"),
        FOLLOW_SUBSCRIBE(20, 25, "Follow/subscribe"),
        DAILY_LOGIN(10, 1, "Daily login bonus"),
        DAILY_QUEST(100, 3, "Complete daily quest"),
        MILESTONE(500, Int.MAX_VALUE, "Achievement milestone"),
        VIRAL_CONTENT(1000, Int.MAX_VALUE, "Viral content (1K+ views)")
    }
    
    @Serializable
    enum class Platform(val multiplier: Double) {
        TIKTOK(1.3),
        INSTAGRAM(1.2),
        YOUTUBE(1.4),
        FACEBOOK(1.1),
        TWITTER_X(1.2),
        APP(1.0),
        OTHER(1.0)
    }
    
    @Serializable
    data class XPLevel(
        val level: Int,
        val tier: String,
        val badge: String,
        val xpRequired: Int,
        val miningMultiplier: Double,
        val dailyCapFIN: Double,
        val specialUnlocks: List<String>
    )
    
    @Serializable
    data class XPCalculationResult(
        val baseXP: Int,
        val platformMultiplier: Double,
        val qualityScore: Double,
        val streakBonus: Double,
        val levelProgression: Double,
        val finalXP: Int,
        val reasonBreakdown: Map<String, String>
    )
    
    private val levelTiers = listOf(
        // Bronze (1-10)
        (1..10).map { level ->
            XPLevel(
                level = level,
                tier = "Bronze",
                badge = "Bronze ${toRoman(level)}",
                xpRequired = (level * 100) - 1,
                miningMultiplier = 1.0 + (level * 0.02),
                dailyCapFIN = 0.5 + (level * 0.15),
                specialUnlocks = listOf("Basic features")
            )
        },
        // Silver (11-25)
        (11..25).map { level ->
            XPLevel(
                level = level,
                tier = "Silver",
                badge = "Silver ${toRoman(level - 10)}",
                xpRequired = 1000 + ((level - 10) * 250) - 1,
                miningMultiplier = 1.3 + ((level - 10) * 0.03),
                dailyCapFIN = 2.0 + ((level - 10) * 0.13),
                specialUnlocks = listOf("Special cards access")
            )
        },
        // Gold (26-50)
        (26..50).map { level ->
            XPLevel(
                level = level,
                tier = "Gold",
                badge = "Gold ${toRoman(level - 25)}",
                xpRequired = 5000 + ((level - 25) * 600) - 1,
                miningMultiplier = 1.9 + ((level - 25) * 0.024),
                dailyCapFIN = 4.0 + ((level - 25) * 0.08),
                specialUnlocks = listOf("Guild leadership")
            )
        },
        // Platinum (51-75)
        (51..75).map { level ->
            XPLevel(
                level = level,
                tier = "Platinum",
                badge = "Platinum ${toRoman(level - 50)}",
                xpRequired = 20000 + ((level - 50) * 1200) - 1,
                miningMultiplier = 2.6 + ((level - 50) * 0.024),
                dailyCapFIN = 6.0 + ((level - 50) * 0.08),
                specialUnlocks = listOf("Creator monetization")
            )
        },
        // Diamond (76-100)
        (76..100).map { level ->
            XPLevel(
                level = level,
                tier = "Diamond",
                badge = "Diamond ${toRoman(level - 75)}",
                xpRequired = 50000 + ((level - 75) * 2000) - 1,
                miningMultiplier = 3.3 + ((level - 75) * 0.028),
                dailyCapFIN = 8.0 + ((level - 75) * 0.08),
                specialUnlocks = listOf("Exclusive events")
            )
        },
        // Mythic (101+)
        (101..MAX_LEVEL).map { level ->
            XPLevel(
                level = level,
                tier = "Mythic",
                badge = "Mythic ${toRoman(level - 100)}",
                xpRequired = 100000 + ((level - 100) * 5000) - 1,
                miningMultiplier = 4.1 + ((level - 100) * 0.009),
                dailyCapFIN = 10.0 + ((level - 100) * 0.05),
                specialUnlocks = listOf("DAO governance")
            )
        }
    ).flatten()
    
    /**
     * Calculate XP for given activity with all multipliers
     */
    suspend fun calculateXP(
        activity: XPActivity,
        user: User
    ): Result<XPCalculationResult> = withContext(Dispatchers.IO) {
        try {
            // Input validation
            if (!Validation.isValidActivity(activity)) {
                return@withContext Result.failure(
                    IllegalArgumentException("Invalid activity data")
                )
            }
            
            // Check daily limits
            val todayCount = getTodayActivityCount(activity.type, user.id)
            if (todayCount >= activity.type.dailyLimit) {
                return@withContext Result.failure(
                    IllegalStateException("Daily limit exceeded for ${activity.type}")
                )
            }
            
            // Base XP calculation
            val baseXP = activity.type.baseXP
            
            // Platform multiplier
            val platformMultiplier = activity.platform.multiplier
            
            // Quality score from AI analysis
            val qualityScore = analyzeContentQuality(activity)
            
            // Streak bonus calculation
            val streakBonus = calculateStreakBonus(user.streakDays)
            
            // Level progression factor (exponential regression)
            val levelProgression = exp(-LEVEL_PROGRESSION_FACTOR * user.currentLevel)
            
            // Handle viral content special case
            val finalBaseXP = if (activity.type == ActivityType.VIRAL_CONTENT) {
                val views = activity.metadata["views"]?.toIntOrNull() ?: 0
                if (views >= VIRAL_THRESHOLD) baseXP else 0
            } else baseXP
            
            // Calculate final XP
            val finalXP = (finalBaseXP * platformMultiplier * qualityScore * 
                          streakBonus * levelProgression).roundToInt()
            
            // Create breakdown for transparency
            val reasonBreakdown = mapOf(
                "base_xp" to "$finalBaseXP XP",
                "platform_bonus" to "${(platformMultiplier * 100 - 100).roundToInt()}%",
                "quality_score" to "${(qualityScore * 100).roundToInt()}%",
                "streak_bonus" to "${(streakBonus * 100 - 100).roundToInt()}%",
                "level_factor" to "${(levelProgression * 100).roundToInt()}%",
                "final_xp" to "$finalXP XP"
            )
            
            val result = XPCalculationResult(
                baseXP = finalBaseXP,
                platformMultiplier = platformMultiplier,
                qualityScore = qualityScore,
                streakBonus = streakBonus,
                levelProgression = levelProgression,
                finalXP = finalXP,
                reasonBreakdown = reasonBreakdown
            )
            
            Result.success(result)
            
        } catch (e: Exception) {
            Result.failure(e)
        }
    }
    
    /**
     * Award XP to user and handle level progression
     */
    suspend fun awardXP(
        userId: String,
        activity: XPActivity,
        calculationResult: XPCalculationResult
    ): Result<XP> = withContext(Dispatchers.IO) {
        try {
            val user = client.getUser(userId).getOrThrow()
            val currentXP = user.totalXP
            val newTotalXP = currentXP + calculationResult.finalXP
            
            // Check for level up
            val currentLevel = getLevelFromXP(currentXP)
            val newLevel = getLevelFromXP(newTotalXP)
            val leveledUp = newLevel > currentLevel
            
            // Create XP record
            val xpRecord = XP(
                id = generateXPId(),
                userId = userId,
                activityType = activity.type.name,
                platform = activity.platform.name,
                baseXP = calculationResult.baseXP,
                multipliedXP = calculationResult.finalXP,
                multipliers = calculationResult.reasonBreakdown,
                timestamp = System.currentTimeMillis()
            )
            
            // Save to blockchain and backend
            val saveResult = client.saveXPRecord(xpRecord)
            if (saveResult.isFailure) {
                return@withContext Result.failure(saveResult.exceptionOrNull()!!)
            }
            
            // Update user level if needed
            if (leveledUp) {
                handleLevelUp(userId, currentLevel, newLevel)
            }
            
            // Update local cache
            updateXPCache(userId, newTotalXP)
            updateActivityCount(activity.type, userId)
            
            Result.success(xpRecord)
            
        } catch (e: Exception) {
            Result.failure(e)
        }
    }
    
    /**
     * Get current level information for user
     */
    suspend fun getCurrentLevel(userId: String): Result<XPLevel> = withContext(Dispatchers.IO) {
        try {
            val user = client.getUser(userId).getOrThrow()
            val level = getLevelFromXP(user.totalXP)
            val levelInfo = levelTiers.find { it.level == level }
                ?: levelTiers.last() // Fallback to max level
            
            Result.success(levelInfo)
            
        } catch (e: Exception) {
            Result.failure(e)
        }
    }
    
    /**
     * Get XP progress to next level
     */
    suspend fun getXPProgress(userId: String): Result<Map<String, Any>> = withContext(Dispatchers.IO) {
        try {
            val user = client.getUser(userId).getOrThrow()
            val currentLevel = getLevelFromXP(user.totalXP)
            val currentLevelInfo = levelTiers.find { it.level == currentLevel }!!
            val nextLevelInfo = levelTiers.find { it.level == currentLevel + 1 }
            
            val currentLevelXP = if (currentLevel > 1) {
                levelTiers.find { it.level == currentLevel - 1 }?.xpRequired ?: 0
            } else 0
            
            val xpInCurrentLevel = user.totalXP - currentLevelXP
            val xpForNextLevel = if (nextLevelInfo != null) {
                nextLevelInfo.xpRequired - currentLevelXP
            } else 0
            
            val progress = if (xpForNextLevel > 0) {
                (xpInCurrentLevel.toDouble() / xpForNextLevel * 100).roundToInt()
            } else 100
            
            val result = mapOf(
                "current_level" to currentLevel,
                "current_tier" to currentLevelInfo.tier,
                "current_badge" to currentLevelInfo.badge,
                "total_xp" to user.totalXP,
                "xp_in_level" to xpInCurrentLevel,
                "xp_for_next" to maxOf(0, xpForNextLevel - xpInCurrentLevel),
                "progress_percent" to progress,
                "mining_multiplier" to currentLevelInfo.miningMultiplier,
                "next_level_info" to nextLevelInfo
            )
            
            Result.success(result)
            
        } catch (e: Exception) {
            Result.failure(e)
        }
    }
    
    /**
     * Get user's XP leaderboard position
     */
    suspend fun getLeaderboardPosition(userId: String): Result<Map<String, Any>> {
        return try {
            val position = client.getXPLeaderboardPosition(userId).getOrThrow()
            Result.success(mapOf(
                "global_rank" to position.globalRank,
                "tier_rank" to position.tierRank,
                "guild_rank" to position.guildRank,
                "percentile" to position.percentile
            ))
        } catch (e: Exception) {
            Result.failure(e)
        }
    }
    
    // Private helper methods
    
    private fun getLevelFromXP(totalXP: Int): Int {
        return levelTiers.findLast { it.xpRequired <= totalXP }?.level ?: 1
    }
    
    private suspend fun analyzeContentQuality(activity: XPActivity): Double {
        return try {
            // Call AI quality analysis service
            val qualityResult = client.analyzeContentQuality(activity.content, activity.type.name)
            qualityResult.getOrElse { 1.0 }.coerceIn(MIN_QUALITY_MULTIPLIER, MAX_QUALITY_MULTIPLIER)
        } catch (e: Exception) {
            1.0 // Default to neutral if analysis fails
        }
    }
    
    private fun calculateStreakBonus(streakDays: Int): Double {
        return when {
            streakDays >= 30 -> 3.0  // Max streak bonus
            streakDays >= 14 -> 2.5
            streakDays >= 7 -> 2.0
            streakDays >= 3 -> 1.5
            streakDays >= 1 -> 1.2
            else -> 1.0
        }
    }
    
    private suspend fun getTodayActivityCount(type: ActivityType, userId: String): Int {
        return try {
            client.getDailyActivityCount(userId, type.name).getOrElse { 0 }
        } catch (e: Exception) {
            0
        }
    }
    
    private suspend fun handleLevelUp(userId: String, oldLevel: Int, newLevel: Int) {
        try {
            // Trigger level up events
            val levelUpReward = (newLevel - oldLevel) * 100 // Bonus XP for leveling up
            
            // Notify user of level up
            client.triggerLevelUpNotification(userId, oldLevel, newLevel)
            
            // Update mining multipliers
            client.updateMiningMultiplier(userId, getLevelMiningMultiplier(newLevel))
            
        } catch (e: Exception) {
            // Log error but don't fail the main operation
        }
    }
    
    private fun getLevelMiningMultiplier(level: Int): Double {
        return levelTiers.find { it.level == level }?.miningMultiplier ?: 1.0
    }
    
    private fun updateXPCache(userId: String, totalXP: Int) {
        val prefs = context.getSharedPreferences(Constants.PREFS_NAME, Context.MODE_PRIVATE)
        prefs.edit().putInt("${XP_CACHE_KEY}_$userId", totalXP).apply()
    }
    
    private fun updateActivityCount(type: ActivityType, userId: String) {
        val prefs = context.getSharedPreferences(Constants.PREFS_NAME, Context.MODE_PRIVATE)
        val today = System.currentTimeMillis() / (24 * 60 * 60 * 1000) // Days since epoch
        val key = "${type.name}_${userId}_$today"
        val current = prefs.getInt(key, 0)
        prefs.edit().putInt(key, current + 1).apply()
    }
    
    private fun generateXPId(): String {
        return "xp_${System.currentTimeMillis()}_${(1000..9999).random()}"
    }
    
    private fun toRoman(number: Int): String {
        val values = intArrayOf(1000, 900, 500, 400, 100, 90, 50, 40, 10, 9, 5, 4, 1)
        val romanSymbols = arrayOf("M", "CM", "D", "CD", "C", "XC", "L", "XL", "X", "IX", "V", "IV", "I")
        
        val result = StringBuilder()
        var num = number
        
        for (i in values.indices) {
            while (num >= values[i]) {
                result.append(romanSymbols[i])
                num -= values[i]
            }
        }
        return result.toString()
    }
    
    /**
     * Get comprehensive XP statistics for user
     */
    suspend fun getXPStatistics(userId: String): Result<Map<String, Any>> = withContext(Dispatchers.IO) {
        try {
            val stats = client.getXPStatistics(userId).getOrThrow()
            Result.success(stats)
        } catch (e: Exception) {
            Result.failure(e)
        }
    }
    
    /**
     * Clean up service resources
     */
    fun cleanup() {
        scope.cancel()
    }
}
