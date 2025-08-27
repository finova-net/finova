package com.finova.sdk.models

import kotlinx.serialization.Serializable
import kotlinx.serialization.SerialName
import java.math.BigDecimal
import java.time.Instant
import kotlin.math.*

/**
 * XP (Experience Points) Model for Finova Network Android SDK
 * Implements Hamster Kombat-inspired gamification mechanics
 * with exponential progression and integrated reward calculations
 */
@Serializable
data class XP(
    @SerialName("user_id")
    val userId: String,
    
    @SerialName("current_xp")
    val currentXP: Long = 0L,
    
    @SerialName("current_level")
    val currentLevel: Int = 1,
    
    @SerialName("total_lifetime_xp")
    val totalLifetimeXP: Long = 0L,
    
    @SerialName("daily_xp_gained")
    val dailyXPGained: Long = 0L,
    
    @SerialName("weekly_xp_gained")
    val weeklyXPGained: Long = 0L,
    
    @SerialName("current_streak")
    val currentStreak: Int = 0,
    
    @SerialName("longest_streak")
    val longestStreak: Int = 0,
    
    @SerialName("last_activity")
    val lastActivity: String = Instant.now().toString(),
    
    @SerialName("badge_tier")
    val badgeTier: BadgeTier = BadgeTier.BRONZE_I,
    
    @SerialName("mining_multiplier")
    val miningMultiplier: Double = 1.0,
    
    @SerialName("daily_fin_cap")
    val dailyFinCap: Double = 0.5,
    
    @SerialName("activity_breakdown")
    val activityBreakdown: Map<String, ActivityStats> = emptyMap(),
    
    @SerialName("achievements")
    val achievements: List<Achievement> = emptyList(),
    
    @SerialName("created_at")
    val createdAt: String = Instant.now().toString(),
    
    @SerialName("updated_at")
    val updatedAt: String = Instant.now().toString()
) {
    
    /**
     * Badge Tier System with Mining Multipliers
     */
    @Serializable
    enum class BadgeTier(
        val displayName: String,
        val minXP: Long,
        val maxXP: Long,
        val miningMultiplierMin: Double,
        val miningMultiplierMax: Double,
        val dailyCapMin: Double,
        val dailyCapMax: Double,
        val specialUnlocks: List<String>
    ) {
        // Bronze Tier (Levels 1-10)
        BRONZE_I("Bronze I", 0, 99, 1.0, 1.02, 0.5, 0.7, listOf("Basic features")),
        BRONZE_V("Bronze V", 400, 499, 1.08, 1.12, 1.2, 1.4, listOf("Basic features")),
        BRONZE_X("Bronze X", 900, 999, 1.16, 1.2, 1.8, 2.0, listOf("Basic features")),
        
        // Silver Tier (Levels 11-25) 
        SILVER_I("Silver I", 1000, 1499, 1.3, 1.4, 2.0, 2.4, listOf("Special cards access")),
        SILVER_VIII("Silver VIII", 2500, 2999, 1.5, 1.6, 3.0, 3.4, listOf("Special cards access", "Premium badge")),
        SILVER_XV("Silver XV", 4500, 4999, 1.7, 1.8, 3.8, 4.0, listOf("Special cards access", "Priority support")),
        
        // Gold Tier (Levels 26-50)
        GOLD_I("Gold I", 5000, 7999, 1.9, 2.1, 4.0, 4.8, listOf("Guild leadership")),
        GOLD_XIII("Gold XIII", 12000, 14999, 2.2, 2.4, 5.2, 5.6, listOf("Guild leadership", "Advanced features")),
        GOLD_XXV("Gold XXV", 18000, 19999, 2.4, 2.5, 5.8, 6.0, listOf("Guild leadership", "Creator tools")),
        
        // Platinum Tier (Levels 51-75)
        PLATINUM_I("Platinum I", 20000, 24999, 2.6, 2.8, 6.0, 6.8, listOf("Creator monetization")),
        PLATINUM_XIII("Platinum XIII", 32000, 36999, 2.9, 3.1, 7.2, 7.6, listOf("Creator monetization", "VIP features")),
        PLATINUM_XXV("Platinum XXV", 46000, 49999, 3.1, 3.2, 7.8, 8.0, listOf("Creator monetization", "Exclusive events")),
        
        // Diamond Tier (Levels 76-100)
        DIAMOND_I("Diamond I", 50000, 59999, 3.3, 3.5, 8.0, 8.8, listOf("Exclusive events")),
        DIAMOND_XIII("Diamond XIII", 72000, 79999, 3.6, 3.8, 9.2, 9.6, listOf("Exclusive events", "Elite status")),
        DIAMOND_XXV("Diamond XXV", 92000, 99999, 3.8, 4.0, 9.8, 10.0, listOf("Exclusive events", "Master privileges")),
        
        // Mythic Tier (Levels 101+)
        MYTHIC_I("Mythic I", 100000, 149999, 4.1, 4.3, 10.0, 11.0, listOf("DAO governance")),
        MYTHIC_V("Mythic V", 200000, 299999, 4.4, 4.6, 12.0, 13.0, listOf("DAO governance", "Council member")),
        MYTHIC_X("Mythic X", 500000, Long.MAX_VALUE, 4.7, 5.0, 14.0, 15.0, listOf("DAO governance", "Legendary status"));
        
        companion object {
            fun getBadgeTierByXP(xp: Long): BadgeTier {
                return values().lastOrNull { xp >= it.minXP } ?: BRONZE_I
            }
            
            fun getBadgeTierByLevel(level: Int): BadgeTier {
                return when {
                    level <= 10 -> when {
                        level <= 3 -> BRONZE_I
                        level <= 7 -> BRONZE_V
                        else -> BRONZE_X
                    }
                    level <= 25 -> when {
                        level <= 15 -> SILVER_I
                        level <= 20 -> SILVER_VIII
                        else -> SILVER_XV
                    }
                    level <= 50 -> when {
                        level <= 35 -> GOLD_I
                        level <= 42 -> GOLD_XIII
                        else -> GOLD_XXV
                    }
                    level <= 75 -> when {
                        level <= 60 -> PLATINUM_I
                        level <= 67 -> PLATINUM_XIII
                        else -> PLATINUM_XXV
                    }
                    level <= 100 -> when {
                        level <= 85 -> DIAMOND_I
                        level <= 92 -> DIAMOND_XIII
                        else -> DIAMOND_XXV
                    }
                    else -> when {
                        level <= 110 -> MYTHIC_I
                        level <= 125 -> MYTHIC_V
                        else -> MYTHIC_X
                    }
                }
            }
        }
    }
    
    /**
     * Activity Statistics per platform/type
     */
    @Serializable
    data class ActivityStats(
        @SerialName("platform")
        val platform: String,
        
        @SerialName("activity_type")
        val activityType: String,
        
        @SerialName("total_count")
        val totalCount: Long = 0L,
        
        @SerialName("daily_count")
        val dailyCount: Int = 0,
        
        @SerialName("total_xp_earned")
        val totalXPEarned: Long = 0L,
        
        @SerialName("average_quality_score")
        val averageQualityScore: Double = 1.0,
        
        @SerialName("last_performed")
        val lastPerformed: String = Instant.now().toString()
    )
    
    /**
     * Achievement System
     */
    @Serializable
    data class Achievement(
        @SerialName("id")
        val id: String,
        
        @SerialName("name")
        val name: String,
        
        @SerialName("description")
        val description: String,
        
        @SerialName("category")
        val category: String,
        
        @SerialName("xp_reward")
        val xpReward: Long,
        
        @SerialName("fin_reward")
        val finReward: Double,
        
        @SerialName("nft_reward")
        val nftReward: String?,
        
        @SerialName("unlocked_at")
        val unlockedAt: String,
        
        @SerialName("is_rare")
        val isRare: Boolean = false
    )
    
    /**
     * Activity Types with Base XP Values
     */
    enum class ActivityType(
        val baseXP: Long,
        val dailyLimit: Int,
        val platformBonus: Map<String, Double>
    ) {
        ORIGINAL_POST(50, Int.MAX_VALUE, mapOf(
            "tiktok" to 1.3, "instagram" to 1.2, "youtube" to 1.4, "x" to 1.2, "facebook" to 1.1
        )),
        
        PHOTO_POST(75, 20, mapOf(
            "instagram" to 1.3, "tiktok" to 1.2, "facebook" to 1.1
        )),
        
        VIDEO_CONTENT(150, 10, mapOf(
            "youtube" to 1.4, "tiktok" to 1.3, "instagram" to 1.2
        )),
        
        STORY_STATUS(25, 50, mapOf(
            "instagram" to 1.1, "facebook" to 1.0
        )),
        
        MEANINGFUL_COMMENT(25, 100, mapOf(
            "x" to 1.2, "youtube" to 1.1
        )),
        
        LIKE_REACT(5, 200, emptyMap()),
        
        SHARE_REPOST(15, 50, mapOf(
            "facebook" to 1.1, "x" to 1.1
        )),
        
        FOLLOW_SUBSCRIBE(20, 25, emptyMap()),
        
        DAILY_LOGIN(10, 1, emptyMap()),
        
        DAILY_QUEST(100, 3, emptyMap()),
        
        VIRAL_CONTENT(1000, Int.MAX_VALUE, mapOf(
            "tiktok" to 1.5, "instagram" to 1.4, "youtube" to 1.6, "x" to 1.3
        )),
        
        ACHIEVEMENT_UNLOCK(500, Int.MAX_VALUE, emptyMap())
    }
    
    /**
     * Calculate XP gain for an activity
     * Formula: XP_Gained = Base_XP × Platform_Multiplier × Quality_Score × Streak_Bonus × Level_Progression
     */
    fun calculateXPGain(
        activityType: ActivityType,
        platform: String,
        qualityScore: Double = 1.0,
        streakBonus: Double = 1.0
    ): Long {
        val baseXP = activityType.baseXP
        val platformMultiplier = activityType.platformBonus[platform.lowercase()] ?: 1.0
        val levelProgression = exp(-0.01 * currentLevel)
        val qualityScoreClamped = qualityScore.coerceIn(0.5, 2.0)
        val streakBonusClamped = streakBonus.coerceIn(1.0, 3.0)
        
        val finalXP = baseXP * platformMultiplier * qualityScoreClamped * 
                     streakBonusClamped * levelProgression
        
        return finalXP.roundToLong()
    }
    
    /**
     * Calculate current level based on XP
     * Using exponential progression formula
     */
    fun calculateLevel(): Int {
        if (currentXP == 0L) return 1
        
        // Level formula: Level = floor(log2(XP/100 + 1)) + 1
        val level = floor(ln((currentXP / 100.0) + 1.0) / ln(2.0)).toInt() + 1
        return level.coerceAtLeast(1)
    }
    
    /**
     * Calculate XP required for next level
     */
    fun getXPRequiredForNextLevel(): Long {
        val nextLevel = currentLevel + 1
        val requiredXP = ((2.0.pow(nextLevel - 1) - 1) * 100).toLong()
        return (requiredXP - currentXP).coerceAtLeast(0)
    }
    
    /**
     * Calculate XP progress percentage to next level
     */
    fun getProgressToNextLevel(): Double {
        val currentLevelXP = ((2.0.pow(currentLevel - 1) - 1) * 100).toLong()
        val nextLevelXP = ((2.0.pow(currentLevel) - 1) * 100).toLong()
        
        if (nextLevelXP <= currentLevelXP) return 1.0
        
        val progress = (currentXP - currentLevelXP).toDouble() / (nextLevelXP - currentLevelXP)
        return progress.coerceIn(0.0, 1.0)
    }
    
    /**
     * Calculate mining multiplier based on current level and tier
     */
    fun calculateMiningMultiplier(): Double {
        val tier = BadgeTier.getBadgeTierByLevel(currentLevel)
        val progressInTier = getProgressInTier()
        
        return tier.miningMultiplierMin + 
               (tier.miningMultiplierMax - tier.miningMultiplierMin) * progressInTier
    }
    
    /**
     * Calculate daily FIN cap based on current level and tier
     */
    fun calculateDailyFinCap(): Double {
        val tier = BadgeTier.getBadgeTierByLevel(currentLevel)
        val progressInTier = getProgressInTier()
        
        return tier.dailyCapMin + 
               (tier.dailyCapMax - tier.dailyCapMin) * progressInTier
    }
    
    /**
     * Get progress within current tier (0.0 to 1.0)
     */
    private fun getProgressInTier(): Double {
        val tier = BadgeTier.getBadgeTierByLevel(currentLevel)
        if (tier.maxXP == Long.MAX_VALUE) return 1.0
        
        val progress = (currentXP - tier.minXP).toDouble() / (tier.maxXP - tier.minXP)
        return progress.coerceIn(0.0, 1.0)
    }
    
    /**
     * Calculate streak bonus multiplier
     */
    fun calculateStreakBonus(): Double {
        return when {
            currentStreak >= 30 -> 3.0  // 30+ days: 3x
            currentStreak >= 14 -> 2.5  // 14+ days: 2.5x
            currentStreak >= 7 -> 2.0   // 7+ days: 2x
            currentStreak >= 3 -> 1.5   // 3+ days: 1.5x
            else -> 1.0                 // < 3 days: 1x
        }
    }
    
    /**
     * Check if user can perform activity (daily limits)
     */
    fun canPerformActivity(activityType: ActivityType, platform: String): Boolean {
        val todayStats = activityBreakdown["${platform}_${activityType.name}"]
        val dailyCount = todayStats?.dailyCount ?: 0
        return dailyCount < activityType.dailyLimit
    }
    
    /**
     * Get remaining activities for the day
     */
    fun getRemainingActivities(activityType: ActivityType, platform: String): Int {
        val todayStats = activityBreakdown["${platform}_${activityType.name}"]
        val dailyCount = todayStats?.dailyCount ?: 0
        return (activityType.dailyLimit - dailyCount).coerceAtLeast(0)
    }
    
    /**
     * Get all available special unlocks for current tier
     */
    fun getAvailableUnlocks(): List<String> {
        val tier = BadgeTier.getBadgeTierByLevel(currentLevel)
        val previousTiers = BadgeTier.values().filter { it.ordinal <= tier.ordinal }
        return previousTiers.flatMap { it.specialUnlocks }.distinct()
    }
    
    /**
     * Check if specific feature is unlocked
     */
    fun isFeatureUnlocked(feature: String): Boolean {
        return getAvailableUnlocks().contains(feature)
    }
    
    /**
     * Calculate total XP earned from all activities today
     */
    fun getTodayTotalXP(): Long {
        return activityBreakdown.values.sumOf { it.totalXPEarned }
    }
    
    /**
     * Get top performing platform by XP
     */
    fun getTopPlatform(): String? {
        return activityBreakdown.values
            .groupBy { it.platform }
            .mapValues { it.value.sumOf { stat -> stat.totalXPEarned } }
            .maxByOrNull { it.value }?.key
    }
    
    /**
     * Copy with updated values (immutable update)
     */
    fun addXP(
        xpGained: Long,
        platform: String,
        activityType: ActivityType,
        qualityScore: Double = 1.0
    ): XP {
        val newCurrentXP = currentXP + xpGained
        val newLevel = calculateLevel()
        val newTotalLifetimeXP = totalLifetimeXP + xpGained
        val newDailyXP = dailyXPGained + xpGained
        val newWeeklyXP = weeklyXPGained + xpGained
        
        val activityKey = "${platform}_${activityType.name}"
        val currentStats = activityBreakdown[activityKey] ?: ActivityStats(
            platform = platform,
            activityType = activityType.name
        )
        
        val updatedStats = currentStats.copy(
            totalCount = currentStats.totalCount + 1,
            dailyCount = currentStats.dailyCount + 1,
            totalXPEarned = currentStats.totalXPEarned + xpGained,
            averageQualityScore = (currentStats.averageQualityScore * currentStats.totalCount + qualityScore) / (currentStats.totalCount + 1),
            lastPerformed = Instant.now().toString()
        )
        
        val newActivityBreakdown = activityBreakdown.toMutableMap()
        newActivityBreakdown[activityKey] = updatedStats
        
        return copy(
            currentXP = newCurrentXP,
            currentLevel = newLevel,
            totalLifetimeXP = newTotalLifetimeXP,
            dailyXPGained = newDailyXP,
            weeklyXPGained = newWeeklyXP,
            lastActivity = Instant.now().toString(),
            badgeTier = BadgeTier.getBadgeTierByLevel(newLevel),
            miningMultiplier = calculateMiningMultiplier(),
            dailyFinCap = calculateDailyFinCap(),
            activityBreakdown = newActivityBreakdown,
            updatedAt = Instant.now().toString()
        )
    }
    
    companion object {
        /**
         * Create new XP instance for user
         */
        fun createForUser(userId: String): XP {
            return XP(
                userId = userId,
                currentXP = 0L,
                currentLevel = 1,
                badgeTier = BadgeTier.BRONZE_I,
                miningMultiplier = 1.0,
                dailyFinCap = 0.5
            )
        }
        
        /**
         * XP requirements for each level (first 50 levels)
         */
        val LEVEL_REQUIREMENTS = (1..50).associateWith { level ->
            ((2.0.pow(level - 1) - 1) * 100).toLong()
        }
        
        /**
         * Quality score ranges
         */
        const val MIN_QUALITY_SCORE = 0.5
        const val MAX_QUALITY_SCORE = 2.0
        const val DEFAULT_QUALITY_SCORE = 1.0
        
        /**
         * Streak bonus thresholds
         */
        const val STREAK_BONUS_TIER_1 = 3   // 1.5x
        const val STREAK_BONUS_TIER_2 = 7   // 2.0x
        const val STREAK_BONUS_TIER_3 = 14  // 2.5x
        const val STREAK_BONUS_TIER_4 = 30  // 3.0x
    }
}
