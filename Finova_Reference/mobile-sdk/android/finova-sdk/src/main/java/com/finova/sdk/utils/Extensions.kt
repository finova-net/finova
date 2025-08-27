package com.finova.sdk.utils

import android.content.Context
import android.content.SharedPreferences
import android.graphics.Bitmap
import android.graphics.Canvas
import android.graphics.Paint
import android.graphics.Rect
import android.net.ConnectivityManager
import android.net.NetworkCapabilities
import android.os.Build
import android.security.keystore.KeyGenParameterSpec
import android.security.keystore.KeyProperties
import android.text.format.DateUtils
import android.util.Base64
import android.util.Log
import android.view.View
import android.widget.Toast
import androidx.biometric.BiometricManager
import androidx.biometric.BiometricPrompt
import androidx.core.content.ContextCompat
import androidx.fragment.app.FragmentActivity
import androidx.lifecycle.lifecycleScope
import kotlinx.coroutines.*
import java.math.BigDecimal
import java.math.RoundingMode
import java.security.KeyStore
import java.security.MessageDigest
import java.text.DecimalFormat
import java.text.NumberFormat
import java.text.SimpleDateFormat
import java.util.*
import java.util.concurrent.Executor
import javax.crypto.Cipher
import javax.crypto.KeyGenerator
import javax.crypto.SecretKey
import javax.crypto.spec.IvParameterSpec
import kotlin.math.*

/**
 * Finova Network Android SDK Extensions
 * Enterprise-grade utility extensions for the Finova social-fi super app
 * 
 * Features:
 * - Crypto & blockchain utilities
 * - Social media integration helpers
 * - Mining calculations
 * - XP/RP system utilities
 * - Security & encryption
 * - UI/UX enhancements
 * - Network & performance optimization
 */

// ================================================================================================
// FINOVA CORE EXTENSIONS
// ================================================================================================

/**
 * Finova mining rate calculation with exponential regression
 */
fun calculateMiningRate(
    baseRate: Double,
    userCount: Long,
    referralCount: Int,
    holdings: Double,
    isKycVerified: Boolean
): Double {
    val pioneerBonus = maxOf(1.0, 2.0 - (userCount.toDouble() / 1_000_000))
    val referralBonus = 1.0 + (referralCount * 0.1)
    val securityBonus = if (isKycVerified) 1.2 else 0.8
    val regressionFactor = exp(-0.001 * holdings)
    
    return baseRate * pioneerBonus * referralBonus * securityBonus * regressionFactor
}

/**
 * XP calculation with quality multipliers
 */
fun calculateXP(
    baseXp: Int,
    platformMultiplier: Double,
    qualityScore: Double,
    streakBonus: Double,
    userLevel: Int
): Int {
    val levelProgression = exp(-0.01 * userLevel)
    val totalXp = baseXp * platformMultiplier * qualityScore * streakBonus * levelProgression
    return maxOf(1, totalXp.roundToInt())
}

/**
 * RP (Referral Points) calculation with network effects
 */
fun calculateRP(
    directReferrals: Int,
    indirectNetwork: Int,
    networkQuality: Double,
    totalNetworkSize: Int
): Int {
    val directRp = directReferrals * 100 * networkQuality
    val indirectRp = (indirectNetwork * 0.3) + (totalNetworkSize * 0.1)
    val qualityBonus = networkQuality * 10
    val regressionFactor = exp(-0.0001 * totalNetworkSize * networkQuality)
    
    return ((directRp + indirectRp) * qualityBonus * regressionFactor).roundToInt()
}

/**
 * Token amount formatting for Finova ecosystem
 */
fun Double.toFinToken(decimals: Int = 6): String {
    val formatter = DecimalFormat().apply {
        maximumFractionDigits = decimals
        minimumFractionDigits = 0
        isGroupingUsed = true
    }
    return "${formatter.format(this)} \$FIN"
}

/**
 * Convert to Indonesian Rupiah format
 */
fun Double.toIDRFormat(): String {
    val formatter = NumberFormat.getCurrencyInstance(Locale("id", "ID"))
    return formatter.format(this)
}

// ================================================================================================
// SECURITY & ENCRYPTION EXTENSIONS
// ================================================================================================

/**
 * Secure key generation for Finova SDK
 */
fun Context.generateFinovaSecretKey(alias: String): SecretKey? {
    return try {
        val keyGenerator = KeyGenerator.getInstance(KeyProperties.KEY_ALGORITHM_AES, "AndroidKeyStore")
        val keyGenParameterSpec = KeyGenParameterSpec.Builder(
            alias,
            KeyProperties.PURPOSE_ENCRYPT or KeyProperties.PURPOSE_DECRYPT
        )
            .setBlockModes(KeyProperties.BLOCK_MODE_CBC)
            .setEncryptionPaddings(KeyProperties.ENCRYPTION_PADDING_PKCS7)
            .setUserAuthenticationRequired(true)
            .setUserAuthenticationValidityDurationSeconds(300) // 5 minutes
            .build()
        
        keyGenerator.init(keyGenParameterSpec)
        keyGenerator.generateKey()
    } catch (e: Exception) {
        Log.e("FinovaSDK", "Failed to generate secret key", e)
        null
    }
}

/**
 * Encrypt sensitive data using Android Keystore
 */
fun String.encryptWithFinovaKey(context: Context, keyAlias: String): String? {
    return try {
        val keyStore = KeyStore.getInstance("AndroidKeyStore")
        keyStore.load(null)
        
        val secretKey = keyStore.getKey(keyAlias, null) as SecretKey
        val cipher = Cipher.getInstance("AES/CBC/PKCS7Padding")
        cipher.init(Cipher.ENCRYPT_MODE, secretKey)
        
        val iv = cipher.iv
        val encryptedData = cipher.doFinal(this.toByteArray())
        
        // Combine IV and encrypted data
        val combined = iv + encryptedData
        Base64.encodeToString(combined, Base64.DEFAULT)
    } catch (e: Exception) {
        Log.e("FinovaSDK", "Encryption failed", e)
        null
    }
}

/**
 * Decrypt data using Android Keystore
 */
fun String.decryptWithFinovaKey(context: Context, keyAlias: String): String? {
    return try {
        val keyStore = KeyStore.getInstance("AndroidKeyStore")
        keyStore.load(null)
        
        val secretKey = keyStore.getKey(keyAlias, null) as SecretKey
        val combined = Base64.decode(this, Base64.DEFAULT)
        
        // Extract IV and encrypted data
        val iv = combined.sliceArray(0..15)
        val encryptedData = combined.sliceArray(16 until combined.size)
        
        val cipher = Cipher.getInstance("AES/CBC/PKCS7Padding")
        cipher.init(Cipher.DECRYPT_MODE, secretKey, IvParameterSpec(iv))
        
        String(cipher.doFinal(encryptedData))
    } catch (e: Exception) {
        Log.e("FinovaSDK", "Decryption failed", e)
        null
    }
}

/**
 * Generate SHA-256 hash for content verification
 */
fun String.sha256(): String {
    val digest = MessageDigest.getInstance("SHA-256")
    val hash = digest.digest(this.toByteArray())
    return hash.joinToString("") { "%02x".format(it) }
}

/**
 * Biometric authentication helper
 */
fun FragmentActivity.authenticateWithBiometric(
    title: String = "Finova Authentication",
    subtitle: String = "Use your biometric to authenticate",
    onSuccess: () -> Unit,
    onError: (String) -> Unit
) {
    val biometricManager = BiometricManager.from(this)
    
    when (biometricManager.canAuthenticate(BiometricManager.Authenticators.BIOMETRIC_WEAK)) {
        BiometricManager.BIOMETRIC_SUCCESS -> {
            val executor: Executor = ContextCompat.getMainExecutor(this)
            val biometricPrompt = BiometricPrompt(this, executor,
                object : BiometricPrompt.AuthenticationCallback() {
                    override fun onAuthenticationError(errorCode: Int, errString: CharSequence) {
                        super.onAuthenticationError(errorCode, errString)
                        onError(errString.toString())
                    }

                    override fun onAuthenticationSucceeded(result: BiometricPrompt.AuthenticationResult) {
                        super.onAuthenticationSucceeded(result)
                        onSuccess()
                    }

                    override fun onAuthenticationFailed() {
                        super.onAuthenticationFailed()
                        onError("Authentication failed")
                    }
                })

            val promptInfo = BiometricPrompt.PromptInfo.Builder()
                .setTitle(title)
                .setSubtitle(subtitle)
                .setNegativeButtonText("Cancel")
                .build()

            biometricPrompt.authenticate(promptInfo)
        }
        else -> {
            onError("Biometric authentication not available")
        }
    }
}

// ================================================================================================
// SOCIAL MEDIA INTEGRATION EXTENSIONS
// ================================================================================================

/**
 * Social platform detection and configuration
 */
enum class SocialPlatform(val platformName: String, val multiplier: Double, val baseXp: Int) {
    INSTAGRAM("Instagram", 1.2, 75),
    TIKTOK("TikTok", 1.3, 100),
    YOUTUBE("YouTube", 1.4, 150),
    FACEBOOK("Facebook", 1.1, 50),
    TWITTER_X("X (Twitter)", 1.2, 60),
    LINKEDIN("LinkedIn", 1.1, 40),
    UNKNOWN("Unknown", 1.0, 25)
}

/**
 * Content type classification for XP calculation
 */
enum class ContentType(val baseXp: Int, val qualityMultiplier: DoubleRange) {
    TEXT_POST(50, 0.8..1.5),
    IMAGE_POST(75, 0.9..1.8),
    VIDEO_POST(150, 1.0..2.0),
    STORY_STATUS(25, 0.7..1.2),
    COMMENT(25, 0.5..1.5),
    LIKE_REACTION(5, 1.0..1.0),
    SHARE_REPOST(15, 0.8..1.3),
    FOLLOW_SUBSCRIBE(20, 1.0..1.0)
}

/**
 * Quality score calculation for social content
 */
fun calculateContentQuality(
    engagementRate: Double,
    originalityScore: Double,
    platformRelevance: Double,
    brandSafety: Double,
    humanGenerated: Double
): Double {
    val weights = mapOf(
        "engagement" to 0.25,
        "originality" to 0.30,
        "relevance" to 0.20,
        "safety" to 0.15,
        "human" to 0.10
    )
    
    val weightedScore = (engagementRate * weights["engagement"]!!) +
                       (originalityScore * weights["originality"]!!) +
                       (platformRelevance * weights["relevance"]!!) +
                       (brandSafety * weights["safety"]!!) +
                       (humanGenerated * weights["human"]!!)
    
    return maxOf(0.5, minOf(2.0, weightedScore))
}

/**
 * Viral content detection
 */
fun isViralContent(views: Long, likes: Long, shares: Long, platform: SocialPlatform): Boolean {
    val viralThresholds = mapOf(
        SocialPlatform.TIKTOK to mapOf("views" to 10_000L, "likes" to 1_000L, "shares" to 100L),
        SocialPlatform.INSTAGRAM to mapOf("views" to 5_000L, "likes" to 500L, "shares" to 50L),
        SocialPlatform.YOUTUBE to mapOf("views" to 1_000L, "likes" to 100L, "shares" to 10L),
        SocialPlatform.TWITTER_X to mapOf("views" to 25_000L, "likes" to 1_000L, "shares" to 250L)
    )
    
    val thresholds = viralThresholds[platform] ?: return false
    
    return views >= thresholds["views"]!! && 
           likes >= thresholds["likes"]!! && 
           shares >= thresholds["shares"]!!
}

// ================================================================================================
// UI/UX ENHANCEMENT EXTENSIONS
// ================================================================================================

/**
 * Finova-themed toast messages
 */
fun Context.showFinovaToast(message: String, isSuccess: Boolean = true) {
    val icon = if (isSuccess) "✅" else "❌"
    Toast.makeText(this, "$icon $message", Toast.LENGTH_SHORT).show()
}

/**
 * Level badge generation
 */
fun generateLevelBadge(level: Int, context: Context): Bitmap? {
    return try {
        val size = 120
        val bitmap = Bitmap.createBitmap(size, size, Bitmap.Config.ARGB_8888)
        val canvas = Canvas(bitmap)
        
        // Background circle
        val paint = Paint().apply {
            isAntiAlias = true
            color = when {
                level < 11 -> 0xFF8D4004.toInt() // Bronze
                level < 26 -> 0xFFC0C0C0.toInt() // Silver
                level < 51 -> 0xFFFFD700.toInt() // Gold
                level < 76 -> 0xFFE5E4E2.toInt() // Platinum
                level < 101 -> 0xFFB9F2FF.toInt() // Diamond
                else -> 0xFFFF6B6B.toInt() // Mythic
            }
        }
        canvas.drawCircle(size / 2f, size / 2f, size / 2f - 5, paint)
        
        // Level text
        paint.apply {
            color = 0xFF000000.toInt()
            textAlign = Paint.Align.CENTER
            textSize = 24f
        }
        
        val textBounds = Rect()
        val text = level.toString()
        paint.getTextBounds(text, 0, text.length, textBounds)
        
        canvas.drawText(
            text,
            size / 2f,
            size / 2f + textBounds.height() / 2f,
            paint
        )
        
        bitmap
    } catch (e: Exception) {
        Log.e("FinovaSDK", "Failed to generate level badge", e)
        null
    }
}

/**
 * Animated progress bar for mining
 */
fun View.animateMiningProgress(
    currentAmount: Double,
    targetAmount: Double,
    duration: Long = 1000L
) {
    val animator = android.animation.ValueAnimator.ofFloat(0f, 1f)
    animator.duration = duration
    animator.addUpdateListener { animation ->
        val progress = animation.animatedValue as Float
        val currentValue = currentAmount + (targetAmount - currentAmount) * progress
        
        // Update UI based on progress
        alpha = 0.7f + (0.3f * progress)
        scaleX = 0.95f + (0.05f * progress)
        scaleY = 0.95f + (0.05f * progress)
    }
    animator.start()
}

// ================================================================================================
// NETWORK & PERFORMANCE EXTENSIONS
// ================================================================================================

/**
 * Network connectivity check
 */
fun Context.isNetworkAvailable(): Boolean {
    val connectivityManager = getSystemService(Context.CONNECTIVITY_SERVICE) as ConnectivityManager
    
    return if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.M) {
        val network = connectivityManager.activeNetwork ?: return false
        val capabilities = connectivityManager.getNetworkCapabilities(network) ?: return false
        capabilities.hasTransport(NetworkCapabilities.TRANSPORT_WIFI) ||
        capabilities.hasTransport(NetworkCapabilities.TRANSPORT_CELLULAR) ||
        capabilities.hasTransport(NetworkCapabilities.TRANSPORT_ETHERNET)
    } else {
        @Suppress("DEPRECATION")
        val networkInfo = connectivityManager.activeNetworkInfo
        networkInfo?.isConnected == true
    }
}

/**
 * Retry mechanism with exponential backoff
 */
suspend fun <T> retryWithExponentialBackoff(
    maxRetries: Int = 3,
    initialDelayMs: Long = 1000,
    maxDelayMs: Long = 10000,
    factor: Double = 2.0,
    block: suspend () -> T
): T {
    var currentDelay = initialDelayMs
    repeat(maxRetries - 1) { attempt ->
        try {
            return block()
        } catch (e: Exception) {
            Log.w("FinovaSDK", "Attempt ${attempt + 1} failed, retrying in ${currentDelay}ms", e)
            delay(currentDelay)
            currentDelay = minOf(currentDelay * factor.toLong(), maxDelayMs)
        }
    }
    return block() // Last attempt
}

// ================================================================================================
// DATA PERSISTENCE EXTENSIONS
// ================================================================================================

/**
 * Secure SharedPreferences for Finova data
 */
fun Context.getFinovaPrefs(): SharedPreferences {
    return getSharedPreferences("finova_secure_prefs", Context.MODE_PRIVATE)
}

/**
 * Store encrypted user data
 */
fun SharedPreferences.putSecureString(key: String, value: String, context: Context) {
    val encrypted = value.encryptWithFinovaKey(context, "finova_user_key")
    encrypted?.let { edit().putString(key, it).apply() }
}

/**
 * Retrieve decrypted user data
 */
fun SharedPreferences.getSecureString(key: String, context: Context, defaultValue: String = ""): String {
    val encrypted = getString(key, null) ?: return defaultValue
    return encrypted.decryptWithFinovaKey(context, "finova_user_key") ?: defaultValue
}

// ================================================================================================
// TIME & DATE UTILITIES
// ================================================================================================

/**
 * Mining streak calculation
 */
fun calculateMiningStreak(lastMiningTime: Long): Int {
    val now = System.currentTimeMillis()
    val timeDiff = now - lastMiningTime
    val daysDiff = timeDiff / (24 * 60 * 60 * 1000)
    
    return when {
        daysDiff <= 1 -> 1 // Same day or yesterday
        timeDiff < 48 * 60 * 60 * 1000 -> daysDiff.toInt() // Within 48 hours
        else -> 0 // Streak broken
    }
}

/**
 * Format time remaining for mining
 */
fun Long.formatTimeRemaining(): String {
    val hours = this / 3600
    val minutes = (this % 3600) / 60
    val seconds = this % 60
    
    return when {
        hours > 0 -> String.format("%02d:%02d:%02d", hours, minutes, seconds)
        minutes > 0 -> String.format("%02d:%02d", minutes, seconds)
        else -> "${seconds}s"
    }
}

/**
 * Human-readable time ago
 */
fun Long.timeAgo(context: Context): String {
    return DateUtils.getRelativeTimeSpanString(
        this,
        System.currentTimeMillis(),
        DateUtils.MINUTE_IN_MILLIS,
        DateUtils.FORMAT_ABBREV_RELATIVE
    ).toString()
}

// ================================================================================================
// VALIDATION UTILITIES
// ================================================================================================

/**
 * Validate Solana wallet address
 */
fun String.isValidSolanaAddress(): Boolean {
    if (length !in 32..44) return false
    return matches(Regex("^[1-9A-HJ-NP-Za-km-z]+$"))
}

/**
 * Validate referral code format
 */
fun String.isValidReferralCode(): Boolean {
    return matches(Regex("^FIN[0-9A-Z]{6}$"))
}

/**
 * Content quality validation
 */
fun String.validateContentQuality(): Pair<Boolean, String> {
    return when {
        length < 10 -> false to "Content too short"
        length > 5000 -> false to "Content too long"
        contains(Regex("[^\\w\\s.,!?@#\$%^&*()\\-+={}\\[\\]:;\"'<>\\/|\\\\~`]")) -> 
            false to "Contains invalid characters"
        split("\\s+".toRegex()).distinct().size < length / 20 -> 
            false to "Content appears repetitive"
        else -> true to "Content quality acceptable"
    }
}

// ================================================================================================
// DEBUGGING & LOGGING EXTENSIONS
// ================================================================================================

/**
 * Debug logging with Finova prefix
 */
fun Any.logDebug(message: String) {
    if (BuildConfig.DEBUG) {
        Log.d("FinovaSDK-${this::class.java.simpleName}", message)
    }
}

/**
 * Error logging with context
 */
fun Any.logError(message: String, throwable: Throwable? = null) {
    Log.e("FinovaSDK-${this::class.java.simpleName}", message, throwable)
}

/**
 * Performance monitoring
 */
inline fun <T> measureTimeMillisWithLog(tag: String, block: () -> T): T {
    val startTime = System.currentTimeMillis()
    val result = block()
    val endTime = System.currentTimeMillis()
    Log.d("FinovaSDK-Performance", "$tag took ${endTime - startTime}ms")
    return result
}

// ================================================================================================
// COROUTINES & ASYNC EXTENSIONS
// ================================================================================================

/**
 * Safe async execution with error handling
 */
fun CoroutineScope.safeLaunch(
    onError: (Throwable) -> Unit = {},
    block: suspend CoroutineScope.() -> Unit
): Job {
    return launch {
        try {
            block()
        } catch (e: Exception) {
            Log.e("FinovaSDK", "Coroutine execution failed", e)
            onError(e)
        }
    }
}

/**
 * Debounced function execution
 */
class Debouncer(private val delayMs: Long) {
    private var job: Job? = null
    
    fun submit(scope: CoroutineScope, action: suspend () -> Unit) {
        job?.cancel()
        job = scope.launch {
            delay(delayMs)
            action()
        }
    }
}

// ================================================================================================
// MATHEMATICAL UTILITIES
// ================================================================================================

/**
 * Safe division with default value
 */
fun Double.safeDivide(divisor: Double, defaultValue: Double = 0.0): Double {
    return if (divisor != 0.0) this / divisor else defaultValue
}

/**
 * Percentage calculation
 */
fun Double.percentageOf(total: Double): Double {
    return if (total != 0.0) (this / total) * 100 else 0.0
}

/**
 * Round to specific decimal places
 */
fun Double.roundTo(decimals: Int): Double {
    return BigDecimal(this).setScale(decimals, RoundingMode.HALF_UP).toDouble()
}

/**
 * Convert to compact number format (1.2K, 1.5M, etc.)
 */
fun Long.toCompactFormat(): String {
    return when {
        this >= 1_000_000_000 -> "${(this / 1_000_000_000.0).roundTo(1)}B"
        this >= 1_000_000 -> "${(this / 1_000_000.0).roundTo(1)}M"
        this >= 1_000 -> "${(this / 1_000.0).roundTo(1)}K"
        else -> toString()
    }
}

// ================================================================================================
// CONSTANTS
// ================================================================================================

object FinovaConstants {
    const val SDK_VERSION = "1.0.0"
    const val API_BASE_URL = "https://api.finova.network/v1/"
    const val WS_BASE_URL = "wss://ws.finova.network/v1/"
    
    // Mining constants
    const val BASE_MINING_RATE = 0.05
    const val MAX_DAILY_MINING = 24.0
    const val MINING_INTERVAL_HOURS = 1
    
    // XP constants
    const val MAX_DAILY_XP = 10000
    const val LEVEL_UP_THRESHOLD = 1000
    
    // RP constants
    const val MAX_REFERRALS_PER_USER = 100
    const val REFERRAL_REWARD_PERCENTAGE = 0.1
    
    // Security constants
    const val SESSION_TIMEOUT_MINUTES = 30
    const val MAX_LOGIN_ATTEMPTS = 5
    const val KEYSTORE_ALIAS = "finova_master_key"
    
    // Network constants
    const val REQUEST_TIMEOUT_SECONDS = 30L
    const val MAX_RETRY_ATTEMPTS = 3
    const val EXPONENTIAL_BACKOFF_BASE = 2.0
}
