package com.finova.sdk.models

import android.os.Parcelable
import androidx.annotation.Keep
import com.google.gson.annotations.SerializedName
import kotlinx.parcelize.Parcelize
import java.math.BigDecimal
import java.math.RoundingMode
import java.time.LocalDateTime
import java.time.ZoneId
import kotlin.math.*

/**
 * Mining model representing the complete mining system for Finova Network
 * Implements Pi Network-inspired exponential regression with XP/RP integration
 * 
 * @author Finova Network Team
 * @version 3.0
 * @since 2025-07-25
 */
@Keep
@Parcelize
data class Mining(
    @SerializedName("user_id")
    val userId: String,
    
    @SerializedName("session_id")
    val sessionId: String? = null,
    
    @SerializedName("current_rate")
    val currentRate: Double = 0.0,
    
    @SerializedName("base_rate")
    val baseRate: Double = 0.05,
    
    @SerializedName("total_mined")
    val totalMined: BigDecimal = BigDecimal.ZERO,
    
    @SerializedName("daily_mined")
    val dailyMined: BigDecimal = BigDecimal.ZERO,
    
    @SerializedName("daily_cap")
    val dailyCap: BigDecimal = BigDecimal("10.0"),
    
    @SerializedName("mining_phase")
    val miningPhase: MiningPhase = MiningPhase.FINIZEN,
    
    @SerializedName("is_active")
    val isActive: Boolean = false,
    
    @SerializedName("is_kyc_verified")
    val isKycVerified: Boolean = false,
    
    @SerializedName("xp_level")
    val xpLevel: Int = 1,
    
    @SerializedName("rp_tier")
    val rpTier: ReferralTier = ReferralTier.EXPLORER,
    
    @SerializedName("active_referrals")
    val activeReferrals: Int = 0,
    
    @SerializedName("total_holdings")
    val totalHoldings: BigDecimal = BigDecimal.ZERO,
    
    @SerializedName("stake_amount")
    val stakeAmount: BigDecimal = BigDecimal.ZERO,
    
    @SerializedName("quality_score")
    val qualityScore: Double = 1.0,
    
    @SerializedName("streak_days")
    val streakDays: Int = 0,
    
    @SerializedName("last_mining_time")
    val lastMiningTime: Long = 0L,
    
    @SerializedName("next_mining_time")
    val nextMiningTime: Long = 0L,
    
    @SerializedName("active_cards")
    val activeCards: List<ActiveCard> = emptyList(),
    
    @SerializedName("network_total_users")
    val networkTotalUsers: Long = 0L,
    
    @SerializedName("multipliers")
    val multipliers: MiningMultipliers = MiningMultipliers(),
    
    @SerializedName("session_stats")
    val sessionStats: SessionStats = SessionStats(),
    
    @SerializedName("created_at")
    val createdAt: Long = System.currentTimeMillis(),
    
    @SerializedName("updated_at")
    val updatedAt: Long = System.currentTimeMillis()
) : Parcelable {

    /**
     * Mining phases based on Pi Network model with Finova enhancements
     */
    enum class MiningPhase(
        val displayName: String,
        val userThreshold: Long,
        val baseRate: Double,
        val finizenBonus: Double,
        val maxDaily: Double
    ) {
        FINIZEN("Finizen Phase", 100_000L, 0.1, 2.0, 4.8),
        GROWTH("Growth Phase", 1_000_000L, 0.05, 1.5, 1.8),
        MATURITY("Maturity Phase", 10_000_000L, 0.025, 1.2, 0.72),
        STABILITY("Stability Phase", Long.MAX_VALUE, 0.01, 1.0, 0.24);

        companion object {
            fun getCurrentPhase(totalUsers: Long): MiningPhase {
                return values().first { totalUsers <= it.userThreshold }
            }
        }
    }

    /**
     * Referral tier system for RP integration
     */
    enum class ReferralTier(
        val displayName: String,
        val rpRange: IntRange,
        val miningBonus: Double,
        val referralBonus: Double,
        val networkCap: Int
    ) {
        EXPLORER("Explorer", 0..999, 0.0, 0.10, 10),
        CONNECTOR("Connector", 1000..4999, 0.20, 0.15, 25),
        INFLUENCER("Influencer", 5000..14999, 0.50, 0.20, 50),
        LEADER("Leader", 15000..49999, 1.00, 0.25, 100),
        AMBASSADOR("Ambassador", 50000..Int.MAX_VALUE, 2.00, 0.30, Int.MAX_VALUE);

        companion object {
            fun getTierFromRP(rp: Int): ReferralTier {
                return values().first { rp in it.rpRange }
            }
        }
    }

    /**
     * Active special card effects
     */
    @Keep
    @Parcelize
    data class ActiveCard(
        @SerializedName("card_id")
        val cardId: String,
        
        @SerializedName("card_name")
        val cardName: String,
        
        @SerializedName("effect_type")
        val effectType: CardEffectType,
        
        @SerializedName("multiplier")
        val multiplier: Double,
        
        @SerializedName("duration_hours")
        val durationHours: Int,
        
        @SerializedName("activated_at")
        val activatedAt: Long,
        
        @SerializedName("expires_at")
        val expiresAt: Long,
        
        @SerializedName("is_active")
        val isActive: Boolean = true
    ) : Parcelable {
        
        val isExpired: Boolean
            get() = System.currentTimeMillis() > expiresAt
            
        val remainingHours: Double
            get() = maxOf(0.0, (expiresAt - System.currentTimeMillis()) / 3600000.0)
    }

    enum class CardEffectType {
        MINING_BOOST,
        XP_BOOST,
        RP_BOOST,
        QUALITY_BOOST,
        STREAK_PROTECTION
    }

    /**
     * Comprehensive mining multipliers
     */
    @Keep
    @Parcelize
    data class MiningMultipliers(
        @SerializedName("finizen_bonus")
        val finizenBonus: Double = 1.0,
        
        @SerializedName("referral_bonus")
        val referralBonus: Double = 1.0,
        
        @SerializedName("security_bonus")
        val securityBonus: Double = 1.0,
        
        @SerializedName("xp_multiplier")
        val xpMultiplier: Double = 1.0,
        
        @SerializedName("rp_multiplier")
        val rpMultiplier: Double = 1.0,
        
        @SerializedName("staking_bonus")
        val stakingBonus: Double = 1.0,
        
        @SerializedName("quality_multiplier")
        val qualityMultiplier: Double = 1.0,
        
        @SerializedName("card_multiplier")
        val cardMultiplier: Double = 1.0,
        
        @SerializedName("streak_bonus")
        val streakBonus: Double = 1.0,
        
        @SerializedName("regression_factor")
        val regressionFactor: Double = 1.0,
        
        @SerializedName("total_multiplier")
        val totalMultiplier: Double = 1.0
    ) : Parcelable

    /**
     * Mining session statistics
     */
    @Keep
    @Parcelize
    data class SessionStats(
        @SerializedName("session_start")
        val sessionStart: Long = 0L,
        
        @SerializedName("session_duration")
        val sessionDuration: Long = 0L,
        
        @SerializedName("tokens_earned_this_session")
        val tokensEarnedThisSession: BigDecimal = BigDecimal.ZERO,
        
        @SerializedName("activities_completed")
        val activitiesCompleted: Int = 0,
        
        @SerializedName("xp_gained_this_session")
        val xpGainedThisSession: Int = 0,
        
        @SerializedName("rp_gained_this_session")
        val rpGainedThisSession: Int = 0,
        
        @SerializedName("avg_quality_score")
        val avgQualityScore: Double = 1.0
    ) : Parcelable

    // Computed Properties

    /**
     * Calculate current hourly mining rate based on Pi Network formula
     * with Finova enhancements
     */
    fun calculateHourlyRate(): Double {
        val phase = MiningPhase.getCurrentPhase(networkTotalUsers)
        val baseRate = phase.baseRate
        
        // Finizen bonus (pioneer bonus)
        val finizenBonus = maxOf(1.0, phase.finizenBonus - (networkTotalUsers / 1_000_000.0))
        
        // Referral network bonus
        val referralBonus = 1.0 + (activeReferrals * 0.1)
        
        // Security bonus for KYC
        val securityBonus = if (isKycVerified) 1.2 else 0.8
        
        // XP level multiplier
        val xpMultiplier = 1.0 + (xpLevel / 100.0) * 0.5
        
        // RP tier bonus
        val rpMultiplier = 1.0 + rpTier.miningBonus
        
        // Staking bonus
        val stakingMultiplier = when {
            stakeAmount >= BigDecimal("10000") -> 2.0
            stakeAmount >= BigDecimal("5000") -> 1.75
            stakeAmount >= BigDecimal("1000") -> 1.5
            stakeAmount >= BigDecimal("500") -> 1.35
            stakeAmount >= BigDecimal("100") -> 1.2
            else -> 1.0
        }
        
        // Quality score impact
        val qualityMultiplier = qualityScore.coerceIn(0.5, 2.0)
        
        // Streak bonus
        val streakMultiplier = 1.0 + (streakDays / 30.0) * 0.5
        
        // Active cards multiplier
        val cardMultiplier = activeCards
            .filter { !it.isExpired && it.effectType == CardEffectType.MINING_BOOST }
            .fold(1.0) { acc, card -> acc * card.multiplier }
        
        // Anti-whale exponential regression
        val regressionFactor = exp(-0.001 * totalHoldings.toDouble())
        
        return baseRate * finizenBonus * referralBonus * securityBonus * 
               xpMultiplier * rpMultiplier * stakingMultiplier * 
               qualityMultiplier * streakMultiplier * cardMultiplier * regressionFactor
    }

    /**
     * Calculate projected daily earnings
     */
    fun calculateDailyEarnings(): BigDecimal {
        val hourlyRate = calculateHourlyRate()
        val theoreticalDaily = BigDecimal(hourlyRate * 24)
        return theoreticalDaily.min(dailyCap).setScale(6, RoundingMode.HALF_UP)
    }

    /**
     * Get remaining daily capacity
     */
    fun getRemainingDailyCapacity(): BigDecimal {
        return (dailyCap - dailyMined).max(BigDecimal.ZERO)
    }

    /**
     * Check if user can mine (not at daily cap)
     */
    fun canMine(): Boolean {
        return isActive && dailyMined < dailyCap && !isAtHoldingsLimit()
    }

    /**
     * Check if user is at holdings limit (anti-whale mechanism)
     */
    private fun isAtHoldingsLimit(): Boolean {
        val limit = when (rpTier) {
            ReferralTier.EXPLORER -> BigDecimal("1000")
            ReferralTier.CONNECTOR -> BigDecimal("5000")
            ReferralTier.INFLUENCER -> BigDecimal("25000")
            ReferralTier.LEADER -> BigDecimal("100000")
            ReferralTier.AMBASSADOR -> BigDecimal("1000000")
        }
        return totalHoldings >= limit
    }

    /**
     * Get time until next mining session (cooldown mechanism)
     */
    fun getTimeUntilNextMining(): Long {
        return maxOf(0L, nextMiningTime - System.currentTimeMillis())
    }

    /**
     * Calculate mining efficiency score
     */
    fun calculateEfficiencyScore(): Double {
        val actualRate = if (sessionStats.sessionDuration > 0) {
            sessionStats.tokensEarnedThisSession.toDouble() / (sessionStats.sessionDuration / 3600000.0)
        } else 0.0
        
        val theoreticalRate = calculateHourlyRate()
        return if (theoreticalRate > 0) actualRate / theoreticalRate else 0.0
    }

    /**
     * Get detailed multiplier breakdown for transparency
     */
    fun getMultiplierBreakdown(): MiningMultipliers {
        val phase = MiningPhase.getCurrentPhase(networkTotalUsers)
        
        val finizenBonus = maxOf(1.0, phase.finizenBonus - (networkTotalUsers / 1_000_000.0))
        val referralBonus = 1.0 + (activeReferrals * 0.1)
        val securityBonus = if (isKycVerified) 1.2 else 0.8
        val xpMultiplier = 1.0 + (xpLevel / 100.0) * 0.5
        val rpMultiplier = 1.0 + rpTier.miningBonus
        val stakingBonus = when {
            stakeAmount >= BigDecimal("10000") -> 2.0
            stakeAmount >= BigDecimal("5000") -> 1.75
            stakeAmount >= BigDecimal("1000") -> 1.5
            stakeAmount >= BigDecimal("500") -> 1.35
            stakeAmount >= BigDecimal("100") -> 1.2
            else -> 1.0
        }
        val qualityMultiplier = qualityScore.coerceIn(0.5, 2.0)
        val cardMultiplier = activeCards
            .filter { !it.isExpired && it.effectType == CardEffectType.MINING_BOOST }
            .fold(1.0) { acc, card -> acc * card.multiplier }
        val streakBonus = 1.0 + (streakDays / 30.0) * 0.5
        val regressionFactor = exp(-0.001 * totalHoldings.toDouble())
        
        val totalMultiplier = finizenBonus * referralBonus * securityBonus * 
                            xpMultiplier * rpMultiplier * stakingBonus * 
                            qualityMultiplier * cardMultiplier * streakBonus * regressionFactor
        
        return MiningMultipliers(
            finizenBonus = finizenBonus,
            referralBonus = referralBonus,
            securityBonus = securityBonus,
            xpMultiplier = xpMultiplier,
            rpMultiplier = rpMultiplier,
            stakingBonus = stakingBonus,
            qualityMultiplier = qualityMultiplier,
            cardMultiplier = cardMultiplier,
            streakBonus = streakBonus,
            regressionFactor = regressionFactor,
            totalMultiplier = totalMultiplier
        )
    }

    /**
     * Validate mining state for security
     */
    fun isValidMiningState(): Boolean {
        return userId.isNotBlank() &&
               currentRate >= 0 &&
               totalMined >= BigDecimal.ZERO &&
               dailyMined >= BigDecimal.ZERO &&
               dailyMined <= dailyCap &&
               xpLevel > 0 &&
               activeReferrals >= 0 &&
               totalHoldings >= BigDecimal.ZERO &&
               qualityScore in 0.0..5.0
    }

    /**
     * Get mining status summary
     */
    fun getMiningStatusSummary(): String {
        return when {
            !isActive -> "Mining Inactive"
            !canMine() && isAtHoldingsLimit() -> "Holdings Limit Reached"
            !canMine() && dailyMined >= dailyCap -> "Daily Cap Reached"
            getTimeUntilNextMining() > 0 -> "Cooldown Active"
            else -> "Ready to Mine"
        }
    }

    // Security and validation methods
    
    /**
     * Generate mining proof for blockchain verification
     */
    fun generateMiningProof(): String {
        val timestamp = System.currentTimeMillis()
        val data = "$userId:$currentRate:$timestamp:${qualityScore}:$xpLevel"
        return data.hashCode().toString(16) // Simplified hash - use proper cryptographic hash in production
    }

    /**
     * Check for suspicious mining patterns
     */
    fun detectSuspiciousActivity(): List<String> {
        val issues = mutableListOf<String>()
        
        // Check for impossibly high rates
        if (currentRate > miningPhase.maxDaily / 24.0 * 2) {
            issues.add("Suspiciously high mining rate")
        }
        
        // Check for rapid level progression
        val expectedMaxLevel = (System.currentTimeMillis() - createdAt) / (86400000L) // days since creation
        if (xpLevel > expectedMaxLevel * 5) {
            issues.add("Unusually rapid XP progression")
        }
        
        // Check for perfect quality scores (likely bot)
        if (qualityScore == 2.0 && sessionStats.activitiesCompleted > 100) {
            issues.add("Consistent maximum quality score")
        }
        
        return issues
    }

    // Companion object for utility functions
    companion object {
        const val VERSION = "3.0"
        const val MIN_MINING_INTERVAL = 3600000L // 1 hour in milliseconds
        const val MAX_DAILY_CAP = 15.0 // Maximum possible daily earnings
        
        /**
         * Create a new mining instance for a user
         */
        fun createNewMining(
            userId: String,
            isKycVerified: Boolean = false,
            networkTotalUsers: Long = 0L
        ): Mining {
            val phase = MiningPhase.getCurrentPhase(networkTotalUsers)
            return Mining(
                userId = userId,
                baseRate = phase.baseRate,
                dailyCap = BigDecimal(phase.maxDaily),
                miningPhase = phase,
                isKycVerified = isKycVerified,
                networkTotalUsers = networkTotalUsers,
                nextMiningTime = System.currentTimeMillis() + MIN_MINING_INTERVAL
            )
        }
    }
}
