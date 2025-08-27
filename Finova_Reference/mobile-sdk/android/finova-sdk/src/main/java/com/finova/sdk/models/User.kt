package com.finova.sdk.models

import android.os.Parcelable
import kotlinx.parcelize.Parcelize
import kotlinx.serialization.Serializable
import kotlinx.serialization.SerialName
import java.math.BigDecimal
import java.time.Instant
import java.util.*

/**
 * Finova Network User Model
 * Represents a complete user profile with integrated XP, RP, and Mining systems
 * 
 * Based on Finova Network Whitepaper v3.0 and Complete Smart Contracts Suite
 * Enterprise-grade implementation with security, validation, and performance optimization
 */
@Parcelize
@Serializable
data class User(
    @SerialName("id")
    val id: String,
    
    @SerialName("wallet_address")
    val walletAddress: String,
    
    @SerialName("username")
    val username: String,
    
    @SerialName("email")
    val email: String? = null,
    
    @SerialName("phone")
    val phone: String? = null,
    
    @SerialName("profile")
    val profile: UserProfile,
    
    @SerialName("mining")
    val mining: MiningData,
    
    @SerialName("xp")
    val xp: XPData,
    
    @SerialName("referral")
    val referral: ReferralData,
    
    @SerialName("staking")
    val staking: StakingData,
    
    @SerialName("nft")
    val nft: NFTData,
    
    @SerialName("guild")
    val guild: GuildData? = null,
    
    @SerialName("security")
    val security: SecurityData,
    
    @SerialName("preferences")
    val preferences: UserPreferences,
    
    @SerialName("created_at")
    val createdAt: Long,
    
    @SerialName("updated_at")
    val updatedAt: Long
) : Parcelable {

    /**
     * Calculate total user value based on integrated reward system
     * Formula: XP × RP × Mining Rate × Quality Score × Network Effect
     */
    fun calculateTotalValue(): BigDecimal {
        val baseValue = mining.currentRate
        val xpMultiplier = xp.getLevelMultiplier()
        val rpMultiplier = referral.getTierMultiplier()
        val qualityScore = BigDecimal(profile.qualityScore)
        val networkEffect = referral.getNetworkEffect()
        
        return baseValue
            .multiply(xpMultiplier)
            .multiply(rpMultiplier)
            .multiply(qualityScore)
            .multiply(networkEffect)
    }
    
    /**
     * Get current mining rate with all bonuses applied
     */
    fun getCurrentMiningRate(): BigDecimal = mining.calculateCurrentRate(xp, referral, staking)
    
    /**
     * Check if user qualifies for next tier benefits
     */
    fun canUpgradeTier(): Boolean = xp.canLevelUp() || referral.canUpgradeTier()
    
    /**
     * Get comprehensive user statistics
     */
    fun getStats(): UserStats = UserStats(
        totalValue = calculateTotalValue(),
        miningRate = getCurrentMiningRate(),
        networkSize = referral.getTotalNetworkSize(),
        achievements = profile.achievements.size,
        nftCount = nft.totalOwned
    )
}

@Parcelize
@Serializable
data class UserProfile(
    @SerialName("display_name")
    val displayName: String,
    
    @SerialName("avatar_url")
    val avatarUrl: String? = null,
    
    @SerialName("bio")
    val bio: String? = null,
    
    @SerialName("country_code")
    val countryCode: String,
    
    @SerialName("timezone")
    val timezone: String,
    
    @SerialName("language")
    val language: String = "en",
    
    @SerialName("quality_score")
    val qualityScore: Double = 1.0, // 0.5x - 2.0x multiplier
    
    @SerialName("reputation_score")
    val reputationScore: Int = 0,
    
    @SerialName("achievements")
    val achievements: List<Achievement> = emptyList(),
    
    @SerialName("badges")
    val badges: List<Badge> = emptyList(),
    
    @SerialName("social_connections")
    val socialConnections: List<SocialConnection> = emptyList()
) : Parcelable

@Parcelize
@Serializable
data class MiningData(
    @SerialName("total_mined")
    val totalMined: String, // BigDecimal as String for precision
    
    @SerialName("current_rate")
    val currentRate: String, // Current $FIN per hour
    
    @SerialName("base_rate")
    val baseRate: String, // Base mining rate without bonuses
    
    @SerialName("last_claim")
    val lastClaim: Long,
    
    @SerialName("streak_days")
    val streakDays: Int = 0,
    
    @SerialName("phase")
    val phase: MiningPhase,
    
    @SerialName("bonuses")
    val bonuses: List<MiningBonus> = emptyList(),
    
    @SerialName("daily_cap")
    val dailyCap: String,
    
    @SerialName("daily_mined")
    val dailyMined: String = "0"
) : Parcelable {
    
    val currentRateBD: BigDecimal get() = BigDecimal(currentRate)
    val totalMinedBD: BigDecimal get() = BigDecimal(totalMined)
    
    /**
     * Calculate current mining rate with all multipliers
     * Formula: Base_Rate × Finizen_Bonus × Referral_Bonus × Security_Bonus × Regression_Factor
     */
    fun calculateCurrentRate(xp: XPData, referral: ReferralData, staking: StakingData): BigDecimal {
        val base = BigDecimal(baseRate)
        val finizenBonus = phase.getFinizedBonus()
        val referralBonus = referral.getReferralBonus()
        val securityBonus = BigDecimal("1.2") // Assuming KYC verified
        val regressionFactor = calculateRegressionFactor()
        val stakingBonus = staking.getMiningBonus()
        val xpBonus = xp.getMiningMultiplier()
        
        return base
            .multiply(finizenBonus)
            .multiply(referralBonus)
            .multiply(securityBonus)
            .multiply(regressionFactor)
            .multiply(stakingBonus)
            .multiply(xpBonus)
    }
    
    private fun calculateRegressionFactor(): BigDecimal {
        // Exponential regression: e^(-0.001 × Total_Holdings)
        val holdings = totalMinedBD
        val exponent = holdings.multiply(BigDecimal("-0.001"))
        return BigDecimal(Math.exp(exponent.toDouble()))
    }
    
    fun canClaim(): Boolean = System.currentTimeMillis() - lastClaim >= 3600000 // 1 hour
    
    fun getTimeToNextClaim(): Long = maxOf(0, 3600000 - (System.currentTimeMillis() - lastClaim))
}

@Parcelize
@Serializable
data class XPData(
    @SerialName("total_xp")
    val totalXP: Long,
    
    @SerialName("current_level")
    val currentLevel: Int,
    
    @SerialName("level_xp")
    val levelXP: Long, // XP in current level
    
    @SerialName("next_level_xp")
    val nextLevelXP: Long, // XP needed for next level
    
    @SerialName("badge_tier")
    val badgeTier: BadgeTier,
    
    @SerialName("daily_xp")
    val dailyXP: Long = 0,
    
    @SerialName("weekly_xp")
    val weeklyXP: Long = 0,
    
    @SerialName("streak_days")
    val streakDays: Int = 0,
    
    @SerialName("last_activity")
    val lastActivity: Long,
    
    @SerialName("multipliers")
    val multipliers: List<XPMultiplier> = emptyList()
) : Parcelable {
    
    /**
     * Get XP level multiplier for mining
     * Formula: 1.0x + (Level / 100)
     */
    fun getLevelMultiplier(): BigDecimal = 
        BigDecimal.ONE.add(BigDecimal(currentLevel).divide(BigDecimal("100")))
    
    /**
     * Get mining multiplier from XP level
     * Based on badge tier progression from whitepaper
     */
    fun getMiningMultiplier(): BigDecimal = when (badgeTier) {
        BadgeTier.BRONZE -> BigDecimal("1.0").add(BigDecimal(currentLevel * 0.02))
        BadgeTier.SILVER -> BigDecimal("1.3").add(BigDecimal((currentLevel - 10) * 0.05))
        BadgeTier.GOLD -> BigDecimal("1.9").add(BigDecimal((currentLevel - 25) * 0.024))
        BadgeTier.PLATINUM -> BigDecimal("2.6").add(BigDecimal((currentLevel - 50) * 0.024))
        BadgeTier.DIAMOND -> BigDecimal("3.3").add(BigDecimal((currentLevel - 75) * 0.028))
        BadgeTier.MYTHIC -> BigDecimal("4.1").add(BigDecimal((currentLevel - 100) * 0.018))
    }
    
    fun canLevelUp(): Boolean = levelXP >= nextLevelXP
    
    fun getProgressPercent(): Float = (levelXP.toFloat() / nextLevelXP.toFloat() * 100f)
    
    fun getStreakBonus(): BigDecimal = BigDecimal.ONE.add(
        BigDecimal(minOf(streakDays, 30)).multiply(BigDecimal("0.05"))
    )
}

@Parcelize
@Serializable
data class ReferralData(
    @SerialName("referral_code")
    val referralCode: String,
    
    @SerialName("referred_by")
    val referredBy: String? = null,
    
    @SerialName("total_rp")
    val totalRP: Long,
    
    @SerialName("tier")
    val tier: RPTier,
    
    @SerialName("direct_referrals")
    val directReferrals: Int,
    
    @SerialName("active_referrals")
    val activeReferrals: Int,
    
    @SerialName("network_size")
    val networkSize: NetworkSize,
    
    @SerialName("quality_score")
    val qualityScore: Double, // Network quality (active/total ratio)
    
    @SerialName("earnings_shared")
    val earningsShared: String, // Total earnings shared with referrers
    
    @SerialName("level_2_count")
    val level2Count: Int = 0,
    
    @SerialName("level_3_count")
    val level3Count: Int = 0
) : Parcelable {
    
    /**
     * Get referral tier multiplier for mining
     */
    fun getTierMultiplier(): BigDecimal = when (tier) {
        RPTier.EXPLORER -> BigDecimal("1.0")
        RPTier.CONNECTOR -> BigDecimal("1.2")
        RPTier.INFLUENCER -> BigDecimal("1.5")
        RPTier.LEADER -> BigDecimal("2.0")
        RPTier.AMBASSADOR -> BigDecimal("3.0")
    }
    
    /**
     * Get referral bonus for mining calculation
     * Formula: 1 + (Active_Referrals × 0.1)
     */
    fun getReferralBonus(): BigDecimal = 
        BigDecimal.ONE.add(BigDecimal(activeReferrals).multiply(BigDecimal("0.1")))
    
    /**
     * Calculate network effect multiplier
     */
    fun getNetworkEffect(): BigDecimal {
        val networkQuality = BigDecimal(qualityScore)
        val sizeMultiplier = BigDecimal(Math.log(networkSize.total + 1.0))
        return BigDecimal.ONE.add(networkQuality.multiply(sizeMultiplier).multiply(BigDecimal("0.1")))
    }
    
    fun getTotalNetworkSize(): Int = networkSize.total
    
    fun canUpgradeTier(): Boolean = when (tier) {
        RPTier.EXPLORER -> totalRP >= 1000
        RPTier.CONNECTOR -> totalRP >= 5000
        RPTier.INFLUENCER -> totalRP >= 15000
        RPTier.LEADER -> totalRP >= 50000
        RPTier.AMBASSADOR -> false
    }
}

@Parcelize
@Serializable
data class StakingData(
    @SerialName("total_staked")
    val totalStaked: String,
    
    @SerialName("staking_tier")
    val stakingTier: StakingTier,
    
    @SerialName("sfin_balance")
    val sfinBalance: String, // Staked $FIN tokens
    
    @SerialName("rewards_earned")
    val rewardsEarned: String,
    
    @SerialName("apy")
    val apy: Double,
    
    @SerialName("staking_duration")
    val stakingDuration: Long, // Days staked
    
    @SerialName("last_reward_claim")
    val lastRewardClaim: Long,
    
    @SerialName("loyalty_bonus")
    val loyaltyBonus: Double = 0.0
) : Parcelable {
    
    val totalStakedBD: BigDecimal get() = BigDecimal(totalStaked)
    
    /**
     * Get mining bonus from staking
     */
    fun getMiningBonus(): BigDecimal = when (stakingTier) {
        StakingTier.NONE -> BigDecimal.ONE
        StakingTier.BASIC -> BigDecimal("1.2")
        StakingTier.PREMIUM -> BigDecimal("1.35")
        StakingTier.VIP -> BigDecimal("1.5")
        StakingTier.ELITE -> BigDecimal("1.75")
        StakingTier.LEGENDARY -> BigDecimal("2.0")
    }
    
    fun getLoyaltyBonus(): BigDecimal = 
        BigDecimal.ONE.add(BigDecimal(stakingDuration).multiply(BigDecimal("0.05")))
    
    fun canClaimRewards(): Boolean = 
        System.currentTimeMillis() - lastRewardClaim >= 86400000 // 24 hours
}

@Parcelize
@Serializable
data class NFTData(
    @SerialName("total_owned")
    val totalOwned: Int,
    
    @SerialName("special_cards")
    val specialCards: List<SpecialCard> = emptyList(),
    
    @SerialName("profile_badges")
    val profileBadges: List<ProfileBadge> = emptyList(),
    
    @SerialName("achievements")
    val achievements: List<AchievementNFT> = emptyList(),
    
    @SerialName("marketplace_stats")
    val marketplaceStats: MarketplaceStats
) : Parcelable

@Parcelize
@Serializable
data class GuildData(
    @SerialName("guild_id")
    val guildId: String,
    
    @SerialName("guild_name")
    val guildName: String,
    
    @SerialName("role")
    val role: GuildRole,
    
    @SerialName("contribution_points")
    val contributionPoints: Long,
    
    @SerialName("joined_at")
    val joinedAt: Long,
    
    @SerialName("current_season")
    val currentSeason: GuildSeason
) : Parcelable

@Parcelize
@Serializable
data class SecurityData(
    @SerialName("kyc_status")
    val kycStatus: KYCStatus,
    
    @SerialName("kyc_level")
    val kycLevel: Int = 0,
    
    @SerialName("biometric_verified")
    val biometricVerified: Boolean = false,
    
    @SerialName("human_probability")
    val humanProbability: Double = 0.5, // 0.0 - 1.0
    
    @SerialName("suspicious_activity_score")
    val suspiciousActivityScore: Double = 0.0,
    
    @SerialName("last_security_check")
    val lastSecurityCheck: Long,
    
    @SerialName("device_trust_score")
    val deviceTrustScore: Double = 0.5
) : Parcelable {
    
    fun getSecurityMultiplier(): BigDecimal = when {
        kycStatus == KYCStatus.VERIFIED && biometricVerified -> BigDecimal("1.2")
        kycStatus == KYCStatus.VERIFIED -> BigDecimal("1.1")
        else -> BigDecimal("0.8")
    }
    
    fun isHighRisk(): Boolean = 
        suspiciousActivityScore > 0.7 || humanProbability < 0.3
}

@Parcelize
@Serializable
data class UserPreferences(
    @SerialName("notifications")
    val notifications: NotificationSettings,
    
    @SerialName("privacy")
    val privacy: PrivacySettings,
    
    @SerialName("display")
    val display: DisplaySettings
) : Parcelable

// Supporting Data Classes
@Parcelize
@Serializable
data class NetworkSize(
    @SerialName("level_1")
    val level1: Int,
    
    @SerialName("level_2") 
    val level2: Int,
    
    @SerialName("level_3")
    val level3: Int
) : Parcelable {
    val total: Int get() = level1 + level2 + level3
}

@Parcelize
@Serializable
data class UserStats(
    @SerialName("total_value")
    val totalValue: BigDecimal,
    
    @SerialName("mining_rate")
    val miningRate: BigDecimal,
    
    @SerialName("network_size")
    val networkSize: Int,
    
    @SerialName("achievements")
    val achievements: Int,
    
    @SerialName("nft_count")
    val nftCount: Int
) : Parcelable

// Enums
@Serializable
enum class MiningPhase {
    @SerialName("finizen") FINIZEN,
    @SerialName("growth") GROWTH,
    @SerialName("maturity") MATURITY,
    @SerialName("stability") STABILITY;
    
    fun getFinizedBonus(): BigDecimal = when (this) {
        FINIZEN -> BigDecimal("2.0")
        GROWTH -> BigDecimal("1.5")
        MATURITY -> BigDecimal("1.2")
        STABILITY -> BigDecimal("1.0")
    }
}

@Serializable
enum class BadgeTier {
    @SerialName("bronze") BRONZE,
    @SerialName("silver") SILVER,
    @SerialName("gold") GOLD,
    @SerialName("platinum") PLATINUM,
    @SerialName("diamond") DIAMOND,
    @SerialName("mythic") MYTHIC
}

@Serializable
enum class RPTier {
    @SerialName("explorer") EXPLORER,
    @SerialName("connector") CONNECTOR,
    @SerialName("influencer") INFLUENCER,
    @SerialName("leader") LEADER,
    @SerialName("ambassador") AMBASSADOR
}

@Serializable
enum class StakingTier {
    @SerialName("none") NONE,
    @SerialName("basic") BASIC,
    @SerialName("premium") PREMIUM,
    @SerialName("vip") VIP,
    @SerialName("elite") ELITE,
    @SerialName("legendary") LEGENDARY
}

@Serializable
enum class KYCStatus {
    @SerialName("pending") PENDING,
    @SerialName("under_review") UNDER_REVIEW,
    @SerialName("verified") VERIFIED,
    @SerialName("rejected") REJECTED
}

@Serializable
enum class GuildRole {
    @SerialName("member") MEMBER,
    @SerialName("officer") OFFICER,
    @SerialName("master") MASTER
}

// Additional supporting classes would be defined in separate files
@Parcelize @Serializable data class Achievement(val id: String, val name: String, val description: String) : Parcelable
@Parcelize @Serializable data class Badge(val id: String, val type: String, val rarity: String) : Parcelable
@Parcelize @Serializable data class SocialConnection(val platform: String, val connected: Boolean) : Parcelable
@Parcelize @Serializable data class MiningBonus(val type: String, val multiplier: Double, val expires: Long) : Parcelable
@Parcelize @Serializable data class XPMultiplier(val source: String, val value: Double, val expires: Long) : Parcelable
@Parcelize @Serializable data class SpecialCard(val id: String, val type: String, val rarity: String) : Parcelable
@Parcelize @Serializable data class ProfileBadge(val id: String, val name: String, val imageUrl: String) : Parcelable
@Parcelize @Serializable data class AchievementNFT(val id: String, val achievement: String, val mintedAt: Long) : Parcelable
@Parcelize @Serializable data class MarketplaceStats(val totalSales: Int, val totalPurchases: Int) : Parcelable
@Parcelize @Serializable data class GuildSeason(val season: Int, val rank: Int, val points: Long) : Parcelable
@Parcelize @Serializable data class NotificationSettings(val mining: Boolean, val social: Boolean, val guild: Boolean) : Parcelable
@Parcelize @Serializable data class PrivacySettings(val publicProfile: Boolean, val showStats: Boolean) : Parcelable
@Parcelize @Serializable data class DisplaySettings(val theme: String, val language: String) : Parcelable
