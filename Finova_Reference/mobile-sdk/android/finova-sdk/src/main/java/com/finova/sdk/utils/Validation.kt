package com.finova.sdk.utils

import android.util.Patterns
import java.math.BigDecimal
import java.security.MessageDigest
import java.util.regex.Pattern
import kotlin.math.exp

/**
 * Finova Network Android SDK - Validation Utilities
 * 
 * Comprehensive validation utilities for all Finova Network operations including:
 * - User input validation
 * - Mining calculations validation
 * - XP system validation
 * - Referral network validation
 * - Token economics validation
 * - Security and anti-bot validation
 * 
 * @version 1.0.0
 * @since 2025-07-26
 */
object Validation {
    
    // Validation patterns
    private val USERNAME_PATTERN = Pattern.compile("^[a-zA-Z0-9_]{3,20}$")
    private val REFERRAL_CODE_PATTERN = Pattern.compile("^FIN[A-Z0-9]{6}$")
    private val WALLET_ADDRESS_PATTERN = Pattern.compile("^[1-9A-HJ-NP-Za-km-z]{32,44}$")
    private val PHONE_INDONESIA_PATTERN = Pattern.compile("^(\\+62|62|0)[2-9][0-9]{7,12}$")
    
    // Mining constants
    private const val MAX_DAILY_MINING = 15.0
    private const val MIN_MINING_RATE = 0.001
    private const val MAX_MINING_RATE = 0.1
    private const val REGRESSION_FACTOR = 0.001
    
    // XP constants
    private const val MAX_DAILY_XP = 10000
    private const val MIN_XP_PER_ACTIVITY = 1
    private const val MAX_XP_PER_ACTIVITY = 1000
    private const val LEVEL_PROGRESSION_FACTOR = 0.01
    
    // RP constants
    private const val MAX_REFERRAL_NETWORK = 1000
    private const val MIN_NETWORK_QUALITY = 0.1
    private const val MAX_NETWORK_QUALITY = 1.0
    
    // Token constants
    private val MAX_TOKEN_AMOUNT = BigDecimal("100000000000") // 100B tokens
    private val MIN_TOKEN_AMOUNT = BigDecimal("0.000001")
    
    /**
     * User Input Validation
     */
    
    /**
     * Validates username format
     */
    fun validateUsername(username: String?): ValidationResult {
        return when {
            username.isNullOrBlank() -> ValidationResult.error("Username cannot be empty")
            username.length < 3 -> ValidationResult.error("Username must be at least 3 characters")
            username.length > 20 -> ValidationResult.error("Username cannot exceed 20 characters")
            !USERNAME_PATTERN.matcher(username).matches() -> 
                ValidationResult.error("Username can only contain letters, numbers, and underscores")
            isReservedUsername(username) -> ValidationResult.error("Username is reserved")
            else -> ValidationResult.success("Valid username")
        }
    }
    
    /**
     * Validates email format
     */
    fun validateEmail(email: String?): ValidationResult {
        return when {
            email.isNullOrBlank() -> ValidationResult.error("Email cannot be empty")
            !Patterns.EMAIL_ADDRESS.matcher(email).matches() -> 
                ValidationResult.error("Invalid email format")
            email.length > 254 -> ValidationResult.error("Email too long")
            else -> ValidationResult.success("Valid email")
        }
    }
    
    /**
     * Validates Indonesian phone number
     */
    fun validatePhoneNumber(phone: String?): ValidationResult {
        return when {
            phone.isNullOrBlank() -> ValidationResult.error("Phone number cannot be empty")
            !PHONE_INDONESIA_PATTERN.matcher(phone.replace(" ", "").replace("-", "")).matches() ->
                ValidationResult.error("Invalid Indonesian phone number format")
            else -> ValidationResult.success("Valid phone number")
        }
    }
    
    /**
     * Validates password strength
     */
    fun validatePassword(password: String?): ValidationResult {
        return when {
            password.isNullOrBlank() -> ValidationResult.error("Password cannot be empty")
            password.length < 8 -> ValidationResult.error("Password must be at least 8 characters")
            password.length > 128 -> ValidationResult.error("Password too long")
            !password.any { it.isUpperCase() } -> ValidationResult.error("Password must contain uppercase letter")
            !password.any { it.isLowerCase() } -> ValidationResult.error("Password must contain lowercase letter")
            !password.any { it.isDigit() } -> ValidationResult.error("Password must contain a number")
            !password.any { "!@#$%^&*()_+-=[]{}|;:,.<>?".contains(it) } ->
                ValidationResult.error("Password must contain special character")
            else -> ValidationResult.success("Strong password")
        }
    }
    
    /**
     * Validates referral code format
     */
    fun validateReferralCode(code: String?): ValidationResult {
        return when {
            code.isNullOrBlank() -> ValidationResult.error("Referral code cannot be empty")
            !REFERRAL_CODE_PATTERN.matcher(code).matches() ->
                ValidationResult.error("Invalid referral code format (should be FINXXXXXX)")
            else -> ValidationResult.success("Valid referral code")
        }
    }
    
    /**
     * Mining System Validation
     */
    
    /**
     * Validates mining rate calculation
     */
    fun validateMiningRate(
        baseRate: Double,
        pioneerBonus: Double,
        referralBonus: Double,
        securityBonus: Double,
        totalHoldings: Double
    ): ValidationResult {
        return when {
            baseRate < MIN_MINING_RATE || baseRate > MAX_MINING_RATE ->
                ValidationResult.error("Base mining rate out of bounds")
            pioneerBonus < 1.0 || pioneerBonus > 2.0 ->
                ValidationResult.error("Pioneer bonus out of bounds")
            referralBonus < 1.0 || referralBonus > 3.5 ->
                ValidationResult.error("Referral bonus out of bounds")
            securityBonus < 0.8 || securityBonus > 1.2 ->
                ValidationResult.error("Security bonus out of bounds")
            totalHoldings < 0 -> ValidationResult.error("Total holdings cannot be negative")
            else -> {
                val regressionFactor = exp(-REGRESSION_FACTOR * totalHoldings)
                val finalRate = baseRate * pioneerBonus * referralBonus * securityBonus * regressionFactor
                
                when {
                    finalRate > MAX_DAILY_MINING / 24.0 ->
                        ValidationResult.error("Calculated mining rate exceeds daily limit")
                    finalRate < 0 -> ValidationResult.error("Calculated mining rate is negative")
                    else -> ValidationResult.success("Valid mining rate: $finalRate \$FIN/hour")
                }
            }
        }
    }
    
    /**
     * Validates mining session
     */
    fun validateMiningSession(
        sessionDuration: Long,
        lastMiningTime: Long,
        dailyMined: Double
    ): ValidationResult {
        val currentTime = System.currentTimeMillis()
        val hoursSinceLastMining = (currentTime - lastMiningTime) / (1000 * 60 * 60)
        
        return when {
            sessionDuration > 24 * 60 * 60 * 1000 -> // 24 hours in milliseconds
                ValidationResult.error("Mining session too long")
            hoursSinceLastMining < 0.5 -> // Minimum 30 minutes between claims
                ValidationResult.error("Mining claim too frequent")
            dailyMined >= MAX_DAILY_MINING ->
                ValidationResult.error("Daily mining limit reached")
            else -> ValidationResult.success("Valid mining session")
        }
    }
    
    /**
     * XP System Validation
     */
    
    /**
     * Validates XP calculation
     */
    fun validateXPCalculation(
        baseXP: Int,
        platformMultiplier: Double,
        qualityScore: Double,
        streakBonus: Double,
        currentLevel: Int
    ): ValidationResult {
        return when {
            baseXP < MIN_XP_PER_ACTIVITY || baseXP > MAX_XP_PER_ACTIVITY ->
                ValidationResult.error("Base XP out of bounds")
            platformMultiplier < 1.0 || platformMultiplier > 1.5 ->
                ValidationResult.error("Platform multiplier out of bounds")
            qualityScore < 0.5 || qualityScore > 2.0 ->
                ValidationResult.error("Quality score out of bounds")
            streakBonus < 1.0 || streakBonus > 3.0 ->
                ValidationResult.error("Streak bonus out of bounds")
            currentLevel < 1 || currentLevel > 200 ->
                ValidationResult.error("Current level out of bounds")
            else -> {
                val levelProgression = exp(-LEVEL_PROGRESSION_FACTOR * currentLevel)
                val finalXP = (baseXP * platformMultiplier * qualityScore * streakBonus * levelProgression).toInt()
                
                when {
                    finalXP < 0 -> ValidationResult.error("Calculated XP is negative")
                    finalXP > MAX_XP_PER_ACTIVITY * 2 ->
                        ValidationResult.error("Calculated XP exceeds maximum")
                    else -> ValidationResult.success("Valid XP calculation: $finalXP XP")
                }
            }
        }
    }
    
    /**
     * Validates XP level progression
     */
    fun validateXPLevel(currentXP: Int, currentLevel: Int): ValidationResult {
        val expectedLevel = calculateLevelFromXP(currentXP)
        
        return when {
            currentXP < 0 -> ValidationResult.error("Current XP cannot be negative")
            currentLevel < 1 -> ValidationResult.error("Current level must be at least 1")
            abs(expectedLevel - currentLevel) > 1 ->
                ValidationResult.error("Level and XP mismatch detected")
            else -> ValidationResult.success("Valid XP level")
        }
    }
    
    /**
     * Referral Network Validation
     */
    
    /**
     * Validates referral network structure
     */
    fun validateReferralNetwork(
        directReferrals: Int,
        l2Network: Int,
        l3Network: Int,
        activeReferrals: Int,
        networkQuality: Double
    ): ValidationResult {
        return when {
            directReferrals < 0 -> ValidationResult.error("Direct referrals cannot be negative")
            l2Network < 0 -> ValidationResult.error("L2 network cannot be negative")
            l3Network < 0 -> ValidationResult.error("L3 network cannot be negative")
            activeReferrals > directReferrals ->
                ValidationResult.error("Active referrals cannot exceed total referrals")
            networkQuality < MIN_NETWORK_QUALITY || networkQuality > MAX_NETWORK_QUALITY ->
                ValidationResult.error("Network quality out of bounds")
            directReferrals > MAX_REFERRAL_NETWORK ->
                ValidationResult.error("Direct referrals exceed maximum limit")
            l2Network > directReferrals * 50 ->
                ValidationResult.error("L2 network suspiciously large")
            l3Network > l2Network * 20 ->
                ValidationResult.error("L3 network suspiciously large")
            else -> ValidationResult.success("Valid referral network")
        }
    }
    
    /**
     * Validates RP calculation
     */
    fun validateRPCalculation(
        directRP: Double,
        networkRP: Double,
        qualityBonus: Double,
        totalNetworkSize: Int,
        networkQualityScore: Double
    ): ValidationResult {
        return when {
            directRP < 0 -> ValidationResult.error("Direct RP cannot be negative")
            networkRP < 0 -> ValidationResult.error("Network RP cannot be negative")
            qualityBonus < 0.1 || qualityBonus > 50.0 ->
                ValidationResult.error("Quality bonus out of bounds")
            totalNetworkSize < 0 -> ValidationResult.error("Total network size cannot be negative")
            networkQualityScore < MIN_NETWORK_QUALITY || networkQualityScore > MAX_NETWORK_QUALITY ->
                ValidationResult.error("Network quality score out of bounds")
            else -> {
                val regressionFactor = exp(-0.0001 * totalNetworkSize * networkQualityScore)
                val finalRP = (directRP + networkRP) * qualityBonus * regressionFactor
                
                when {
                    finalRP < 0 -> ValidationResult.error("Calculated RP is negative")
                    finalRP > 1000000 -> ValidationResult.error("Calculated RP suspiciously high")
                    else -> ValidationResult.success("Valid RP calculation: $finalRP RP")
                }
            }
        }
    }
    
    /**
     * Token Economics Validation
     */
    
    /**
     * Validates token amount
     */
    fun validateTokenAmount(amount: BigDecimal?): ValidationResult {
        return when {
            amount == null -> ValidationResult.error("Token amount cannot be null")
            amount < MIN_TOKEN_AMOUNT -> ValidationResult.error("Token amount too small")
            amount > MAX_TOKEN_AMOUNT -> ValidationResult.error("Token amount exceeds maximum supply")
            amount.scale() > 6 -> ValidationResult.error("Token amount has too many decimal places")
            else -> ValidationResult.success("Valid token amount")
        }
    }
    
    /**
     * Validates wallet address
     */
    fun validateWalletAddress(address: String?): ValidationResult {
        return when {
            address.isNullOrBlank() -> ValidationResult.error("Wallet address cannot be empty")
            !WALLET_ADDRESS_PATTERN.matcher(address).matches() ->
                ValidationResult.error("Invalid Solana wallet address format")
            address.length < 32 || address.length > 44 ->
                ValidationResult.error("Wallet address length invalid")
            else -> ValidationResult.success("Valid wallet address")
        }
    }
    
    /**
     * Validates staking parameters
     */
    fun validateStaking(
        stakeAmount: BigDecimal,
        currentStaked: BigDecimal,
        maxStakeLimit: BigDecimal,
        minStakeAmount: BigDecimal
    ): ValidationResult {
        return when {
            stakeAmount < minStakeAmount ->
                ValidationResult.error("Stake amount below minimum")
            stakeAmount + currentStaked > maxStakeLimit ->
                ValidationResult.error("Total stake would exceed limit")
            stakeAmount <= BigDecimal.ZERO ->
                ValidationResult.error("Stake amount must be positive")
            else -> ValidationResult.success("Valid staking parameters")
        }
    }
    
    /**
     * Security Validation
     */
    
    /**
     * Validates human probability score
     */
    fun validateHumanScore(score: Double): ValidationResult {
        return when {
            score < 0.0 || score > 1.0 -> ValidationResult.error("Human score out of bounds")
            score < 0.3 -> ValidationResult.warning("Low human probability score")
            score < 0.7 -> ValidationResult.warning("Moderate human probability score")
            else -> ValidationResult.success("High human probability score")
        }
    }
    
    /**
     * Validates activity pattern for bot detection
     */
    fun validateActivityPattern(
        activityTimes: List<Long>,
        activityTypes: List<String>,
        sessionDurations: List<Long>
    ): ValidationResult {
        return when {
            activityTimes.isEmpty() -> ValidationResult.error("No activity data")
            activityTypes.size != activityTimes.size ->
                ValidationResult.error("Activity data mismatch")
            sessionDurations.size != activityTimes.size ->
                ValidationResult.error("Session data mismatch")
            hasRoboticPattern(activityTimes, sessionDurations) ->
                ValidationResult.warning("Suspicious robotic activity pattern detected")
            else -> ValidationResult.success("Natural activity pattern")
        }
    }
    
    /**
     * Content Quality Validation
     */
    
    /**
     * Validates content for quality scoring
     */
    fun validateContent(
        content: String?,
        contentType: String,
        platform: String
    ): ValidationResult {
        return when {
            content.isNullOrBlank() -> ValidationResult.error("Content cannot be empty")
            content.length < getMinContentLength(contentType) ->
                ValidationResult.error("Content too short for quality assessment")
            content.length > getMaxContentLength(contentType) ->
                ValidationResult.error("Content exceeds platform limits")
            containsProfanity(content) -> ValidationResult.error("Content contains inappropriate language")
            isSpam(content) -> ValidationResult.error("Content detected as spam")
            !isSupportedPlatform(platform) -> ValidationResult.error("Unsupported platform")
            else -> ValidationResult.success("Content ready for quality assessment")
        }
    }
    
    /**
     * NFT & Special Cards Validation
     */
    
    /**
     * Validates NFT metadata
     */
    fun validateNFTMetadata(
        name: String?,
        description: String?,
        imageUrl: String?,
        attributes: Map<String, Any>?
    ): ValidationResult {
        return when {
            name.isNullOrBlank() -> ValidationResult.error("NFT name cannot be empty")
            name.length > 50 -> ValidationResult.error("NFT name too long")
            description.isNullOrBlank() -> ValidationResult.error("NFT description cannot be empty")
            description.length > 200 -> ValidationResult.error("NFT description too long")
            imageUrl.isNullOrBlank() -> ValidationResult.error("NFT image URL required")
            !Patterns.WEB_URL.matcher(imageUrl).matches() ->
                ValidationResult.error("Invalid NFT image URL")
            attributes.isNullOrEmpty() -> ValidationResult.error("NFT attributes required")
            else -> ValidationResult.success("Valid NFT metadata")
        }
    }
    
    /**
     * Validates special card usage
     */
    fun validateSpecialCardUsage(
        cardType: String,
        cardRarity: String,
        currentBoosts: List<String>,
        userLevel: Int
    ): ValidationResult {
        return when {
            !isSupportedCardType(cardType) -> ValidationResult.error("Unsupported card type")
            !isSupportedCardRarity(cardRarity) -> ValidationResult.error("Invalid card rarity")
            currentBoosts.size >= getMaxActiveBoosts(userLevel) ->
                ValidationResult.error("Maximum active boosts reached")
            currentBoosts.contains(cardType) && !isStackableCard(cardType) ->
                ValidationResult.error("Card type already active and not stackable")
            else -> ValidationResult.success("Valid special card usage")
        }
    }
    
    /**
     * Utility Functions
     */
    
    private fun isReservedUsername(username: String): Boolean {
        val reserved = listOf("admin", "finova", "support", "system", "bot", "api", "root", "test")
        return reserved.any { username.lowercase().contains(it) }
    }
    
    private fun calculateLevelFromXP(xp: Int): Int {
        return when {
            xp < 1000 -> (xp / 100) + 1
            xp < 5000 -> ((xp - 1000) / 200) + 11
            xp < 20000 -> ((xp - 5000) / 600) + 26
            xp < 50000 -> ((xp - 20000) / 1200) + 51
            xp < 100000 -> ((xp - 50000) / 2000) + 76
            else -> ((xp - 100000) / 5000) + 101
        }.coerceAtMost(200)
    }
    
    private fun hasRoboticPattern(activityTimes: List<Long>, sessionDurations: List<Long>): Boolean {
        if (activityTimes.size < 3) return false
        
        // Check for too regular intervals
        val intervals = activityTimes.zipWithNext { a, b -> b - a }
        val avgInterval = intervals.average()
        val variance = intervals.map { (it - avgInterval) * (it - avgInterval) }.average()
        val coefficient = variance / (avgInterval * avgInterval)
        
        // Check for too consistent session durations
        val avgSession = sessionDurations.average()
        val sessionVariance = sessionDurations.map { (it - avgSession) * (it - avgSession) }.average()
        val sessionCoefficient = sessionVariance / (avgSession * avgSession)
        
        return coefficient < 0.01 || sessionCoefficient < 0.01
    }
    
    private fun getMinContentLength(contentType: String): Int = when (contentType) {
        "text_post" -> 10
        "comment" -> 5
        "caption" -> 5
        else -> 1
    }
    
    private fun getMaxContentLength(contentType: String): Int = when (contentType) {
        "text_post" -> 2200
        "comment" -> 500
        "caption" -> 2200
        else -> 1000
    }
    
    private fun containsProfanity(content: String): Boolean {
        val profanityWords = listOf("spam", "scam", "fake", "bot") // Simplified check
        return profanityWords.any { content.lowercase().contains(it) }
    }
    
    private fun isSpam(content: String): Boolean {
        // Simple spam detection - in production this would use ML models
        return content.count { it == '!' } > 5 ||
               content.uppercase().length > content.length * 0.7 ||
               content.contains(Regex("http[s]?://[^\\s]{10,}"))
    }
    
    private fun isSupportedPlatform(platform: String): Boolean {
        return listOf("instagram", "tiktok", "youtube", "facebook", "twitter", "x").contains(platform.lowercase())
    }
    
    private fun isSupportedCardType(cardType: String): Boolean {
        return listOf("mining_boost", "xp_accelerator", "referral_power", "quality_enhancer")
            .contains(cardType.lowercase())
    }
    
    private fun isSupportedCardRarity(rarity: String): Boolean {
        return listOf("common", "uncommon", "rare", "epic", "legendary").contains(rarity.lowercase())
    }
    
    private fun getMaxActiveBoosts(userLevel: Int): Int = when {
        userLevel < 10 -> 1
        userLevel < 25 -> 2
        userLevel < 50 -> 3
        userLevel < 100 -> 4
        else -> 5
    }
    
    private fun isStackableCard(cardType: String): Boolean {
        return listOf("mining_boost", "xp_accelerator").contains(cardType.lowercase())
    }
    
    private fun abs(value: Int): Int = if (value < 0) -value else value
}

/**
 * Validation result data class
 */
data class ValidationResult(
    val isValid: Boolean,
    val message: String,
    val severity: Severity
) {
    enum class Severity {
        SUCCESS, WARNING, ERROR
    }
    
    companion object {
        fun success(message: String) = ValidationResult(true, message, Severity.SUCCESS)
        fun warning(message: String) = ValidationResult(true, message, Severity.WARNING)
        fun error(message: String) = ValidationResult(false, message, Severity.ERROR)
    }
    
    val isSuccess: Boolean get() = isValid && severity == Severity.SUCCESS
    val isWarning: Boolean get() = isValid && severity == Severity.WARNING
    val isError: Boolean get() = !isValid && severity == Severity.ERROR
}
