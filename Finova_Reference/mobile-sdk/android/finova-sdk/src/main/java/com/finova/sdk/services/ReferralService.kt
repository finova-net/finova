package com.finova.sdk.services

import android.content.Context
import android.util.Log
import com.finova.sdk.client.FinovaClient
import com.finova.sdk.models.Referral
import com.finova.sdk.models.ReferralStats
import com.finova.sdk.models.RPTier
import com.finova.sdk.utils.Constants
import com.finova.sdk.utils.Validation
import kotlinx.coroutines.*
import kotlinx.coroutines.flow.*
import java.security.SecureRandom
import java.util.*
import kotlin.math.*

/**
 * Enterprise-grade Referral Service for Finova Network Android SDK
 * Handles referral network management, RP calculations, and network effects
 */
class ReferralService(
    private val context: Context,
    private val finovaClient: FinovaClient
) {
    private val scope = CoroutineScope(Dispatchers.IO + SupervisorJob())
    private val _referralStats = MutableStateFlow<ReferralStats?>(null)
    private val _rpTier = MutableStateFlow<RPTier?>(null)
    
    val referralStats: StateFlow<ReferralStats?> = _referralStats.asStateFlow()
    val rpTier: StateFlow<RPTier?> = _rpTier.asStateFlow()
    
    companion object {
        private const val TAG = "ReferralService"
        private const val MAX_REFERRAL_CODE_LENGTH = 12
        private const val MIN_REFERRAL_CODE_LENGTH = 6
        private const val RP_UPDATE_INTERVAL = 30000L // 30 seconds
        private const val NETWORK_REGRESSION_CONSTANT = 0.0001
    }

    /**
     * Generate unique referral code with entropy validation
     */
    suspend fun generateReferralCode(userId: String): Result<String> = withContext(Dispatchers.IO) {
        try {
            val random = SecureRandom()
            val characters = "ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789"
            val baseCode = StringBuilder()
            
            // Generate base code
            repeat(8) {
                baseCode.append(characters[random.nextInt(characters.length)])
            }
            
            // Add user identifier hash
            val userHash = userId.hashCode().toString(16).uppercase().take(4)
            val finalCode = "${baseCode}$userHash"
            
            // Validate uniqueness via API
            val isUnique = finovaClient.validateReferralCodeUniqueness(finalCode)
            if (!isUnique) {
                return@withContext generateReferralCode(userId) // Retry with new code
            }
            
            Log.d(TAG, "Generated referral code: $finalCode")
            Result.success(finalCode)
        } catch (e: Exception) {
            Log.e(TAG, "Failed to generate referral code", e)
            Result.failure(e)
        }
    }

    /**
     * Register referral relationship with validation
     */
    suspend fun useReferralCode(referralCode: String): Result<Boolean> = withContext(Dispatchers.IO) {
        try {
            if (!Validation.isValidReferralCode(referralCode)) {
                return@withContext Result.failure(IllegalArgumentException("Invalid referral code format"))
            }
            
            val currentUser = finovaClient.getCurrentUser()
                ?: return@withContext Result.failure(IllegalStateException("User not authenticated"))
            
            // Prevent self-referral
            if (referralCode == currentUser.referralCode) {
                return@withContext Result.failure(IllegalArgumentException("Cannot use own referral code"))
            }
            
            // Check if user already has referrer
            if (currentUser.referredBy != null) {
                return@withContext Result.failure(IllegalStateException("User already has a referrer"))
            }
            
            val result = finovaClient.registerReferral(currentUser.id, referralCode)
            if (result.isSuccess) {
                // Award initial RP to both parties
                awardRegistrationRP(referralCode, currentUser.id)
                updateReferralStats()
            }
            
            result
        } catch (e: Exception) {
            Log.e(TAG, "Failed to use referral code", e)
            Result.failure(e)
        }
    }

    /**
     * Calculate Referral Points with network effects and regression
     */
    suspend fun calculateRP(userId: String): Result<Double> = withContext(Dispatchers.IO) {
        try {
            val referralNetwork = finovaClient.getReferralNetwork(userId)
                ?: return@withContext Result.failure(IllegalStateException("Could not fetch referral network"))
            
            val directRP = calculateDirectReferralPoints(referralNetwork.directReferrals)
            val networkRP = calculateNetworkPoints(referralNetwork.indirectReferrals)
            val qualityBonus = calculateNetworkQuality(referralNetwork)
            val regressionFactor = calculateNetworkRegression(referralNetwork)
            
            val totalRP = (directRP + networkRP) * qualityBonus * regressionFactor
            
            Log.d(TAG, "RP Calculation - Direct: $directRP, Network: $networkRP, Quality: $qualityBonus, Regression: $regressionFactor, Total: $totalRP")
            
            Result.success(totalRP)
        } catch (e: Exception) {
            Log.e(TAG, "Failed to calculate RP", e)
            Result.failure(e)
        }
    }

    /**
     * Get current RP tier with benefits
     */
    suspend fun getRPTier(rpValue: Double): RPTier {
        return when {
            rpValue >= 50000 -> RPTier.AMBASSADOR
            rpValue >= 15000 -> RPTier.LEADER
            rpValue >= 5000 -> RPTier.INFLUENCER
            rpValue >= 1000 -> RPTier.CONNECTOR
            else -> RPTier.EXPLORER
        }
    }

    /**
     * Calculate mining bonus from RP tier
     */
    fun getMiningBonus(tier: RPTier): Double {
        return when (tier) {
            RPTier.EXPLORER -> 0.0
            RPTier.CONNECTOR -> 0.2
            RPTier.INFLUENCER -> 0.5
            RPTier.LEADER -> 1.0
            RPTier.AMBASSADOR -> 2.0
        }
    }

    /**
     * Get referral percentage bonus based on tier
     */
    fun getReferralBonus(tier: RPTier): Map<String, Double> {
        return when (tier) {
            RPTier.EXPLORER -> mapOf("L1" to 0.10)
            RPTier.CONNECTOR -> mapOf("L1" to 0.15, "L2" to 0.05)
            RPTier.INFLUENCER -> mapOf("L1" to 0.20, "L2" to 0.08, "L3" to 0.03)
            RPTier.LEADER -> mapOf("L1" to 0.25, "L2" to 0.10, "L3" to 0.05)
            RPTier.AMBASSADOR -> mapOf("L1" to 0.30, "L2" to 0.15, "L3" to 0.08)
        }
    }

    /**
     * Track referral activity and update RP in real-time
     */
    fun startReferralTracking(userId: String) {
        scope.launch {
            while (true) {
                try {
                    val rpResult = calculateRP(userId)
                    if (rpResult.isSuccess) {
                        val rpValue = rpResult.getOrThrow()
                        val tier = getRPTier(rpValue)
                        
                        _rpTier.value = tier
                        updateReferralStats()
                    }
                } catch (e: Exception) {
                    Log.e(TAG, "Error during referral tracking", e)
                }
                delay(RP_UPDATE_INTERVAL)
            }
        }
    }

    /**
     * Award RP for referral registration
     */
    private suspend fun awardRegistrationRP(referrerCode: String, newUserId: String) {
        try {
            val referrer = finovaClient.getUserByReferralCode(referrerCode)
            if (referrer != null) {
                // Award RP to referrer
                finovaClient.awardRP(referrer.id, 50.0, "referral_signup")
                
                // Award RP to new user
                finovaClient.awardRP(newUserId, 25.0, "used_referral_code")
                
                Log.d(TAG, "Awarded registration RP - Referrer: ${referrer.id}, New User: $newUserId")
            }
        } catch (e: Exception) {
            Log.e(TAG, "Failed to award registration RP", e)
        }
    }

    /**
     * Calculate direct referral points
     */
    private fun calculateDirectReferralPoints(directReferrals: List<Referral>): Double {
        return directReferrals.sumOf { referral ->
            val activityScore = referral.averageActivity * 100.0
            val retentionBonus = if (referral.isActive) 1.0 else 0.5
            val timeDecay = calculateTimeDecay(referral.joinDate)
            
            activityScore * retentionBonus * timeDecay
        }
    }

    /**
     * Calculate network points from indirect referrals
     */
    private fun calculateNetworkPoints(indirectReferrals: Map<Int, List<Referral>>): Double {
        var totalPoints = 0.0
        
        indirectReferrals.forEach { (level, referrals) ->
            val levelMultiplier = when (level) {
                2 -> 0.3
                3 -> 0.1
                else -> 0.0
            }
            
            val levelPoints = referrals.sumOf { referral ->
                referral.averageActivity * 50.0 * levelMultiplier
            }
            
            totalPoints += levelPoints
        }
        
        return totalPoints
    }

    /**
     * Calculate network quality score
     */
    private fun calculateNetworkQuality(network: ReferralNetwork): Double {
        val totalReferrals = network.directReferrals.size + network.indirectReferrals.values.flatten().size
        if (totalReferrals == 0) return 1.0
        
        val activeReferrals = network.directReferrals.count { it.isActive } + 
                             network.indirectReferrals.values.flatten().count { it.isActive }
        
        val activityRate = activeReferrals.toDouble() / totalReferrals
        val averageLevel = network.directReferrals.map { it.level }.average()
        val retentionRate = network.directReferrals.count { 
            System.currentTimeMillis() - it.joinDate < 30 * 24 * 60 * 60 * 1000 // Active in last 30 days
        }.toDouble() / network.directReferrals.size
        
        return activityRate * (1 + averageLevel / 100) * retentionRate
    }

    /**
     * Calculate exponential regression factor
     */
    private fun calculateNetworkRegression(network: ReferralNetwork): Double {
        val totalNetworkSize = network.directReferrals.size + network.indirectReferrals.values.flatten().size
        val qualityScore = calculateNetworkQuality(network)
        
        return exp(-NETWORK_REGRESSION_CONSTANT * totalNetworkSize * qualityScore)
    }

    /**
     * Calculate time decay factor
     */
    private fun calculateTimeDecay(joinDate: Long): Double {
        val daysSinceJoin = (System.currentTimeMillis() - joinDate) / (24 * 60 * 60 * 1000)
        return max(0.1, 1.0 - (daysSinceJoin * 0.001)) // Gradual decay over time
    }

    /**
     * Update referral statistics
     */
    private suspend fun updateReferralStats() {
        try {
            val currentUser = finovaClient.getCurrentUser() ?: return
            val stats = finovaClient.getReferralStats(currentUser.id)
            
            _referralStats.value = stats
        } catch (e: Exception) {
            Log.e(TAG, "Failed to update referral stats", e)
        }
    }

    /**
     * Validate referral network for suspicious patterns
     */
    suspend fun validateReferralNetwork(userId: String): Result<Boolean> = withContext(Dispatchers.IO) {
        try {
            val network = finovaClient.getReferralNetwork(userId)
                ?: return@withContext Result.failure(IllegalStateException("Could not fetch network"))
            
            // Check for circular references
            val hasCircularRefs = detectCircularReferences(network)
            if (hasCircularRefs) {
                Log.w(TAG, "Circular references detected in referral network")
                return@withContext Result.success(false)
            }
            
            // Check for suspicious patterns
            val hasSuspiciousPatterns = detectSuspiciousPatterns(network)
            if (hasSuspiciousPatterns) {
                Log.w(TAG, "Suspicious patterns detected in referral network")
                return@withContext Result.success(false)
            }
            
            Result.success(true)
        } catch (e: Exception) {
            Log.e(TAG, "Failed to validate referral network", e)
            Result.failure(e)
        }
    }

    /**
     * Detect circular references in referral network
     */
    private fun detectCircularReferences(network: ReferralNetwork): Boolean {
        val visited = mutableSetOf<String>()
        val path = mutableSetOf<String>()
        
        fun dfs(userId: String): Boolean {
            if (path.contains(userId)) return true // Circular reference found
            if (visited.contains(userId)) return false
            
            visited.add(userId)
            path.add(userId)
            
            val referrals = network.directReferrals.filter { it.referrerId == userId }
            for (referral in referrals) {
                if (dfs(referral.userId)) return true
            }
            
            path.remove(userId)
            return false
        }
        
        return network.directReferrals.any { dfs(it.userId) }
    }

    /**
     * Detect suspicious patterns (bot networks, fake referrals)
     */
    private fun detectSuspiciousPatterns(network: ReferralNetwork): Boolean {
        // Check for same-day mass registrations
        val registrationDates = network.directReferrals.map { it.joinDate }
        val sameDayRegistrations = registrationDates.groupBy { 
            it / (24 * 60 * 60 * 1000) 
        }.values.maxOfOrNull { it.size } ?: 0
        
        if (sameDayRegistrations > 10) return true
        
        // Check for identical activity patterns
        val activityPatterns = network.directReferrals.map { it.averageActivity }
        val uniquePatterns = activityPatterns.toSet().size
        val totalReferrals = activityPatterns.size
        
        // If more than 80% have identical activity, suspicious
        if (totalReferrals > 5 && uniquePatterns.toDouble() / totalReferrals < 0.2) return true
        
        return false
    }

    /**
     * Get referral leaderboard
     */
    suspend fun getReferralLeaderboard(limit: Int = 100): Result<List<ReferralLeader>> = withContext(Dispatchers.IO) {
        try {
            val leaderboard = finovaClient.getReferralLeaderboard(limit)
            Result.success(leaderboard)
        } catch (e: Exception) {
            Log.e(TAG, "Failed to get referral leaderboard", e)
            Result.failure(e)
        }
    }

    /**
     * Clean up resources
     */
    fun cleanup() {
        scope.cancel()
    }

    // Data classes for referral network structure
    data class ReferralNetwork(
        val userId: String,
        val directReferrals: List<Referral>,
        val indirectReferrals: Map<Int, List<Referral>>
    )

    data class ReferralLeader(
        val userId: String,
        val username: String,
        val referralCount: Int,
        val totalRP: Double,
        val tier: RPTier,
        val rank: Int
    )
}
