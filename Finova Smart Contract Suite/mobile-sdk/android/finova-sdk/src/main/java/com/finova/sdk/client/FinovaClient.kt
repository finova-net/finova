package com.finova.sdk.client

import android.content.Context
import android.util.Log
import androidx.lifecycle.LiveData
import androidx.lifecycle.MutableLiveData
import com.finova.sdk.models.*
import com.finova.sdk.services.*
import com.finova.sdk.utils.Constants
import com.finova.sdk.utils.Validation
import com.finova.sdk.utils.Extensions.toJson
import com.finova.sdk.utils.Extensions.fromJson
import kotlinx.coroutines.*
import okhttp3.*
import okhttp3.logging.HttpLoggingInterceptor
import retrofit2.Retrofit
import retrofit2.converter.gson.GsonConverterFactory
import java.math.BigDecimal
import java.util.concurrent.TimeUnit
import javax.crypto.Cipher
import javax.crypto.KeyGenerator
import javax.crypto.spec.SecretKeySpec
import kotlin.math.exp
import kotlin.math.pow

/**
 * Main Finova SDK client for Android applications
 * Integrates mining, XP, RP, staking, NFT, and social features
 */
class FinovaClient private constructor(
    private val context: Context,
    private val config: FinovaConfig
) {
    companion object {
        private const val TAG = "FinovaClient"
        private const val PREFS_NAME = "finova_sdk_prefs"
        private const val KEY_USER_DATA = "user_data"
        private const val KEY_AUTH_TOKEN = "auth_token"
        private const val KEY_REFRESH_TOKEN = "refresh_token"
        
        @Volatile
        private var INSTANCE: FinovaClient? = null
        
        fun initialize(context: Context, config: FinovaConfig): FinovaClient {
            return INSTANCE ?: synchronized(this) {
                INSTANCE ?: FinovaClient(context.applicationContext, config).also {
                    INSTANCE = it
                    it.initializeServices()
                }
            }
        }
        
        fun getInstance(): FinovaClient {
            return INSTANCE ?: throw IllegalStateException("FinovaClient not initialized")
        }
    }

    // Core Services
    private val miningService: MiningService by lazy { MiningService(this) }
    private val xpService: XPService by lazy { XPService(this) }
    private val referralService: ReferralService by lazy { ReferralService(this) }
    private val nftService: NFTService by lazy { NFTService(this) }
    private val walletConnector: WalletConnector by lazy { WalletConnector(context, this) }
    private val transactionManager: TransactionManager by lazy { TransactionManager(this) }

    // Network & Storage
    private val okHttpClient: OkHttpClient
    private val retrofit: Retrofit
    private val sharedPrefs = context.getSharedPreferences(PREFS_NAME, Context.MODE_PRIVATE)
    private val coroutineScope = CoroutineScope(Dispatchers.IO + SupervisorJob())

    // Live Data Observables
    private val _userState = MutableLiveData<User>()
    val userState: LiveData<User> = _userState

    private val _miningState = MutableLiveData<MiningState>()
    val miningState: LiveData<MiningState> = _miningState

    private val _connectionState = MutableLiveData<ConnectionState>()
    val connectionState: LiveData<ConnectionState> = _connectionState

    // Internal State
    private var currentUser: User? = null
    private var authToken: String? = null
    private var isInitialized = false
    private var miningJob: Job? = null

    init {
        // Configure OkHttp with security and monitoring
        okHttpClient = OkHttpClient.Builder()
            .addInterceptor(AuthInterceptor())
            .addInterceptor(SecurityInterceptor())
            .addInterceptor(createLoggingInterceptor())
            .connectTimeout(30, TimeUnit.SECONDS)
            .readTimeout(60, TimeUnit.SECONDS)
            .writeTimeout(60, TimeUnit.SECONDS)
            .retryOnConnectionFailure(true)
            .build()

        // Configure Retrofit
        retrofit = Retrofit.Builder()
            .baseUrl(config.apiBaseUrl)
            .client(okHttpClient)
            .addConverterFactory(GsonConverterFactory.create())
            .build()

        Log.d(TAG, "FinovaClient initialized with config: ${config.environment}")
    }

    private fun initializeServices() {
        coroutineScope.launch {
            try {
                loadStoredUserData()
                if (authToken != null) {
                    validateSession()
                }
                _connectionState.postValue(ConnectionState.CONNECTED)
                isInitialized = true
                Log.d(TAG, "Services initialized successfully")
            } catch (e: Exception) {
                Log.e(TAG, "Failed to initialize services", e)
                _connectionState.postValue(ConnectionState.ERROR)
            }
        }
    }

    // ==================== AUTHENTICATION ====================

    suspend fun login(credentials: LoginCredentials): Result<AuthResponse> {
        return try {
            val request = LoginRequest(
                email = credentials.email,
                password = encryptPassword(credentials.password),
                deviceInfo = getDeviceInfo(),
                biometricHash = credentials.biometricHash
            )

            val response = retrofit.create(AuthService::class.java).login(request)
            
            if (response.isSuccessful && response.body() != null) {
                val authResponse = response.body()!!
                handleAuthSuccess(authResponse)
                Result.success(authResponse)
            } else {
                Result.failure(Exception("Login failed: ${response.message()}"))
            }
        } catch (e: Exception) {
            Log.e(TAG, "Login error", e)
            Result.failure(e)
        }
    }

    suspend fun register(userData: RegisterRequest): Result<AuthResponse> {
        return try {
            val enrichedRequest = userData.copy(
                deviceInfo = getDeviceInfo(),
                referralCode = extractReferralCode(userData.referralCode),
                kycData = userData.kycData?.copy(
                    timestamp = System.currentTimeMillis()
                )
            )

            val response = retrofit.create(AuthService::class.java).register(enrichedRequest)
            
            if (response.isSuccessful && response.body() != null) {
                val authResponse = response.body()!!
                handleAuthSuccess(authResponse)
                Result.success(authResponse)
            } else {
                Result.failure(Exception("Registration failed: ${response.message()}"))
            }
        } catch (e: Exception) {
            Log.e(TAG, "Registration error", e)
            Result.failure(e)
        }
    }

    private fun handleAuthSuccess(authResponse: AuthResponse) {
        authToken = authResponse.accessToken
        currentUser = authResponse.user
        
        // Store securely
        sharedPrefs.edit()
            .putString(KEY_AUTH_TOKEN, authResponse.accessToken)
            .putString(KEY_REFRESH_TOKEN, authResponse.refreshToken)
            .putString(KEY_USER_DATA, authResponse.user.toJson())
            .apply()

        _userState.postValue(authResponse.user)
        
        // Start mining if user is KYC verified
        if (authResponse.user.isKycVerified) {
            startMining()
        }
    }

    // ==================== MINING SYSTEM ====================

    suspend fun startMining(): Result<MiningSession> {
        return try {
            val user = currentUser ?: return Result.failure(Exception("User not logged in"))
            
            if (!user.isKycVerified) {
                return Result.failure(Exception("KYC verification required for mining"))
            }

            val miningRate = calculateMiningRate(user)
            val session = MiningSession(
                userId = user.id,
                startTime = System.currentTimeMillis(),
                currentRate = miningRate,
                totalMined = BigDecimal.ZERO,
                isActive = true
            )

            // Start mining coroutine
            miningJob = coroutineScope.launch {
                while (session.isActive && isActive) {
                    delay(Constants.MINING_UPDATE_INTERVAL)
                    updateMiningProgress(session)
                }
            }

            _miningState.postValue(MiningState.ACTIVE(session))
            Result.success(session)
        } catch (e: Exception) {
            Log.e(TAG, "Failed to start mining", e)
            Result.failure(e)
        }
    }

    private fun calculateMiningRate(user: User): BigDecimal {
        val baseRate = getCurrentPhaseRate()
        val pioneerBonus = calculatePioneerBonus()
        val referralBonus = calculateReferralBonus(user.referrals)
        val securityBonus = if (user.isKycVerified) 1.2 else 0.8
        val regressionFactor = exp(-0.001 * user.totalHoldings.toDouble())
        val xpMultiplier = calculateXPMultiplier(user.xpLevel)
        val rpMultiplier = calculateRPMultiplier(user.rpTier)
        val stakingBonus = calculateStakingBonus(user.stakedAmount)

        return baseRate
            .multiply(BigDecimal(pioneerBonus))
            .multiply(BigDecimal(referralBonus))
            .multiply(BigDecimal(securityBonus))
            .multiply(BigDecimal(regressionFactor))
            .multiply(BigDecimal(xpMultiplier))
            .multiply(BigDecimal(rpMultiplier))
            .multiply(BigDecimal(stakingBonus))
    }

    private fun getCurrentPhaseRate(): BigDecimal {
        return when (config.totalUsers) {
            in 0..100000 -> BigDecimal("0.1")      // Phase 1: Pioneer
            in 100001..1000000 -> BigDecimal("0.05")   // Phase 2: Growth
            in 1000001..10000000 -> BigDecimal("0.025") // Phase 3: Maturity
            else -> BigDecimal("0.01")              // Phase 4: Stability
        }
    }

    private fun calculatePioneerBonus(): Double {
        return maxOf(1.0, 2.0 - (config.totalUsers / 1000000.0))
    }

    private fun calculateReferralBonus(referrals: List<Referral>): Double {
        val activeReferrals = referrals.count { it.isActive }
        return 1.0 + (activeReferrals * 0.1)
    }

    private fun calculateXPMultiplier(xpLevel: Int): Double {
        return when (xpLevel) {
            in 1..10 -> 1.0 + (xpLevel * 0.02)     // 1.0x - 1.2x
            in 11..25 -> 1.3 + ((xpLevel - 10) * 0.03) // 1.3x - 1.8x
            in 26..50 -> 1.9 + ((xpLevel - 25) * 0.024) // 1.9x - 2.5x
            in 51..75 -> 2.6 + ((xpLevel - 50) * 0.024) // 2.6x - 3.2x
            in 76..100 -> 3.3 + ((xpLevel - 75) * 0.028) // 3.3x - 4.0x
            else -> 4.1 + ((xpLevel - 100) * 0.009)      // 4.1x - 5.0x max
        }.coerceAtMost(5.0)
    }

    private fun calculateRPMultiplier(rpTier: RPTier): Double {
        return when (rpTier) {
            RPTier.EXPLORER -> 1.0
            RPTier.CONNECTOR -> 1.2
            RPTier.INFLUENCER -> 1.5
            RPTier.LEADER -> 2.0
            RPTier.AMBASSADOR -> 3.0
        }
    }

    private fun calculateStakingBonus(stakedAmount: BigDecimal): Double {
        return when {
            stakedAmount >= BigDecimal("10000") -> 2.0  // 100% bonus
            stakedAmount >= BigDecimal("5000") -> 1.75   // 75% bonus
            stakedAmount >= BigDecimal("1000") -> 1.5    // 50% bonus
            stakedAmount >= BigDecimal("500") -> 1.35    // 35% bonus
            stakedAmount >= BigDecimal("100") -> 1.2     // 20% bonus
            else -> 1.0                                  // No bonus
        }
    }

    private suspend fun updateMiningProgress(session: MiningSession) {
        val currentTime = System.currentTimeMillis()
        val elapsedHours = (currentTime - session.startTime) / (1000 * 60 * 60.0)
        val minedAmount = session.currentRate.multiply(BigDecimal(elapsedHours))
        
        session.totalMined = minedAmount
        _miningState.postValue(MiningState.ACTIVE(session))

        // Sync with backend every 5 minutes
        if ((currentTime - session.lastSync) > 300000) {
            syncMiningProgress(session)
            session.lastSync = currentTime
        }
    }

    suspend fun stopMining(): Result<MiningReward> {
        return try {
            miningJob?.cancel()
            
            val currentState = _miningState.value
            if (currentState is MiningState.ACTIVE) {
                val reward = finalizeMiningSession(currentState.session)
                _miningState.postValue(MiningState.COMPLETED(reward))
                Result.success(reward)
            } else {
                Result.failure(Exception("No active mining session"))
            }
        } catch (e: Exception) {
            Log.e(TAG, "Failed to stop mining", e)
            Result.failure(e)
        }
    }

    // ==================== XP SYSTEM ====================

    suspend fun earnXP(activity: XPActivity): Result<XPReward> {
        return try {
            val user = currentUser ?: return Result.failure(Exception("User not logged in"))
            
            val baseXP = getBaseXP(activity.type)
            val platformMultiplier = getPlatformMultiplier(activity.platform)
            val qualityScore = analyzeContentQuality(activity.content)
            val streakBonus = calculateStreakBonus(user.streakDays)
            val levelProgression = exp(-0.01 * user.xpLevel)

            val earnedXP = (baseXP * platformMultiplier * qualityScore * streakBonus * levelProgression).toInt()
            
            val xpReward = XPReward(
                amount = earnedXP,
                activity = activity,
                multipliers = mapOf(
                    "platform" to platformMultiplier,
                    "quality" to qualityScore,
                    "streak" to streakBonus,
                    "level" to levelProgression
                ),
                timestamp = System.currentTimeMillis()
            )

            // Update user XP and check for level up
            user.totalXP += earnedXP
            val newLevel = calculateXPLevel(user.totalXP)
            val leveledUp = newLevel > user.xpLevel
            
            if (leveledUp) {
                user.xpLevel = newLevel
                handleLevelUp(user, newLevel)
            }

            currentUser = user
            _userState.postValue(user)

            // Sync with backend
            syncXPUpdate(xpReward)

            Result.success(xpReward)
        } catch (e: Exception) {
            Log.e(TAG, "Failed to earn XP", e)
            Result.failure(e)
        }
    }

    private fun getBaseXP(activityType: XPActivityType): Int {
        return when (activityType) {
            XPActivityType.ORIGINAL_POST -> 50
            XPActivityType.PHOTO_POST -> 75
            XPActivityType.VIDEO_POST -> 150
            XPActivityType.STORY_POST -> 25
            XPActivityType.MEANINGFUL_COMMENT -> 25
            XPActivityType.LIKE_REACT -> 5
            XPActivityType.SHARE_REPOST -> 15
            XPActivityType.FOLLOW_SUBSCRIBE -> 20
            XPActivityType.DAILY_LOGIN -> 10
            XPActivityType.COMPLETE_QUEST -> 100
            XPActivityType.ACHIEVE_MILESTONE -> 500
            XPActivityType.VIRAL_CONTENT -> 1000
        }
    }

    private fun getPlatformMultiplier(platform: SocialPlatform): Double {
        return when (platform) {
            SocialPlatform.TIKTOK -> 1.3
            SocialPlatform.YOUTUBE -> 1.4
            SocialPlatform.INSTAGRAM -> 1.2
            SocialPlatform.TWITTER_X -> 1.2
            SocialPlatform.FACEBOOK -> 1.1
            SocialPlatform.APP -> 1.0
        }
    }

    private suspend fun analyzeContentQuality(content: String?): Double {
        if (content.isNullOrBlank()) return 1.0

        return try {
            val request = ContentAnalysisRequest(
                content = content,
                analysisType = listOf("originality", "engagement", "quality")
            )
            
            val response = retrofit.create(AIService::class.java).analyzeContent(request)
            if (response.isSuccessful) {
                response.body()?.qualityScore ?: 1.0
            } else {
                1.0 // Default to neutral score on API failure
            }
        } catch (e: Exception) {
            Log.w(TAG, "Content analysis failed, using default score", e)
            1.0
        }
    }

    private fun calculateStreakBonus(streakDays: Int): Double {
        return minOf(3.0, 1.0 + (streakDays * 0.05)) // Max 3x bonus at 40 days
    }

    private fun calculateXPLevel(totalXP: Int): Int {
        return when {
            totalXP < 1000 -> totalXP / 100 + 1        // Levels 1-10
            totalXP < 5000 -> (totalXP - 1000) / 267 + 11  // Levels 11-25
            totalXP < 20000 -> (totalXP - 5000) / 600 + 26 // Levels 26-50
            totalXP < 50000 -> (totalXP - 20000) / 1200 + 51 // Levels 51-75
            totalXP < 100000 -> (totalXP - 50000) / 2000 + 76 // Levels 76-100
            else -> (totalXP - 100000) / 5000 + 101     // 101+
        }
    }

    // ==================== REFERRAL SYSTEM ====================

    suspend fun createReferralCode(customCode: String? = null): Result<ReferralCode> {
        return try {
            val user = currentUser ?: return Result.failure(Exception("User not logged in"))
            
            val request = CreateReferralRequest(
                userId = user.id,
                customCode = customCode,
                tier = user.rpTier
            )

            val response = retrofit.create(ReferralService::class.java).createReferralCode(request)
            
            if (response.isSuccessful && response.body() != null) {
                Result.success(response.body()!!)
            } else {
                Result.failure(Exception("Failed to create referral code: ${response.message()}"))
            }
        } catch (e: Exception) {
            Log.e(TAG, "Failed to create referral code", e)
            Result.failure(e)
        }
    }

    suspend fun getReferralNetwork(): Result<ReferralNetwork> {
        return try {
            val user = currentUser ?: return Result.failure(Exception("User not logged in"))
            
            val response = retrofit.create(ReferralService::class.java).getReferralNetwork(user.id)
            
            if (response.isSuccessful && response.body() != null) {
                Result.success(response.body()!!)
            } else {
                Result.failure(Exception("Failed to get referral network: ${response.message()}"))
            }
        } catch (e: Exception) {
            Log.e(TAG, "Failed to get referral network", e)
            Result.failure(e)
        }
    }

    suspend fun calculateRPValue(userId: String): Result<RPCalculation> {
        return try {
            val network = getReferralNetwork().getOrThrow()
            
            val directRP = calculateDirectReferralPoints(network.directReferrals)
            val indirectRP = calculateIndirectNetworkPoints(network)
            val qualityBonus = calculateNetworkQuality(network)
            val regressionFactor = exp(-0.0001 * network.totalNetworkSize * network.qualityScore)
            
            val totalRP = ((directRP + indirectRP) * qualityBonus * regressionFactor).toInt()
            
            val calculation = RPCalculation(
                directPoints = directRP,
                indirectPoints = indirectRP,
                qualityBonus = qualityBonus,
                regressionFactor = regressionFactor,
                totalRP = totalRP,
                tier = calculateRPTier(totalRP),
                networkSize = network.totalNetworkSize
            )
            
            Result.success(calculation)
        } catch (e: Exception) {
            Log.e(TAG, "Failed to calculate RP value", e)
            Result.failure(e)
        }
    }

    // ==================== NFT & SPECIAL CARDS ====================

    suspend fun purchaseSpecialCard(cardType: SpecialCardType, quantity: Int = 1): Result<PurchaseResult> {
        return try {
            val user = currentUser ?: return Result.failure(Exception("User not logged in"))
            
            val cardPrice = getCardPrice(cardType)
            val totalCost = cardPrice.multiply(BigDecimal(quantity))
            
            if (user.finBalance < totalCost) {
                return Result.failure(Exception("Insufficient FIN balance"))
            }

            val request = PurchaseCardRequest(
                userId = user.id,
                cardType = cardType,
                quantity = quantity,
                totalCost = totalCost
            )

            val response = retrofit.create(NFTService::class.java).purchaseSpecialCard(request)
            
            if (response.isSuccessful && response.body() != null) {
                val result = response.body()!!
                
                // Update user balance
                user.finBalance = user.finBalance.subtract(totalCost)
                currentUser = user
                _userState.postValue(user)
                
                Result.success(result)
            } else {
                Result.failure(Exception("Failed to purchase card: ${response.message()}"))
            }
        } catch (e: Exception) {
            Log.e(TAG, "Failed to purchase special card", e)
            Result.failure(e)
        }
    }

    suspend fun useSpecialCard(cardId: String): Result<CardEffect> {
        return try {
            val user = currentUser ?: return Result.failure(Exception("User not logged in"))
            
            val request = UseCardRequest(
                userId = user.id,
                cardId = cardId,
                timestamp = System.currentTimeMillis()
            )

            val response = retrofit.create(NFTService::class.java).useSpecialCard(request)
            
            if (response.isSuccessful && response.body() != null) {
                val effect = response.body()!!
                applyCardEffect(effect)
                Result.success(effect)
            } else {
                Result.failure(Exception("Failed to use card: ${response.message()}"))
            }
        } catch (e: Exception) {
            Log.e(TAG, "Failed to use special card", e)
            Result.failure(e)
        }
    }

    // ==================== STAKING SYSTEM ====================

    suspend fun stakeTokens(amount: BigDecimal, duration: StakingDuration): Result<StakingPosition> {
        return try {
            val user = currentUser ?: return Result.failure(Exception("User not logged in"))
            
            if (user.finBalance < amount) {
                return Result.failure(Exception("Insufficient FIN balance"))
            }

            val request = StakeRequest(
                userId = user.id,
                amount = amount,
                duration = duration,
                currentLevel = user.xpLevel,
                currentRPTier = user.rpTier
            )

            val response = retrofit.create(StakingService::class.java).stakeTokens(request)
            
            if (response.isSuccessful && response.body() != null) {
                val position = response.body()!!
                
                // Update user balances
                user.finBalance = user.finBalance.subtract(amount)
                user.stakedAmount = user.stakedAmount.add(amount)
                currentUser = user
                _userState.postValue(user)
                
                Result.success(position)
            } else {
                Result.failure(Exception("Failed to stake tokens: ${response.message()}"))
            }
        } catch (e: Exception) {
            Log.e(TAG, "Failed to stake tokens", e)
            Result.failure(e)
        }
    }

    // ==================== UTILITY METHODS ====================

    private fun loadStoredUserData() {
        authToken = sharedPrefs.getString(KEY_AUTH_TOKEN, null)
        val userData = sharedPrefs.getString(KEY_USER_DATA, null)
        if (userData != null) {
            currentUser = userData.fromJson<User>()
            currentUser?.let { _userState.postValue(it) }
        }
    }

    private suspend fun validateSession(): Boolean {
        return try {
            val response = retrofit.create(AuthService::class.java).validateSession()
            response.isSuccessful
        } catch (e: Exception) {
            false
        }
    }

    private fun encryptPassword(password: String): String {
        // Implement proper password encryption
        return password // Simplified for demo
    }

    private fun getDeviceInfo(): DeviceInfo {
        return DeviceInfo(
            deviceId = android.provider.Settings.Secure.getString(
                context.contentResolver,
                android.provider.Settings.Secure.ANDROID_ID
            ),
            deviceModel = "${android.os.Build.MANUFACTURER} ${android.os.Build.MODEL}",
            osVersion = android.os.Build.VERSION.RELEASE,
            appVersion = getAppVersion(),
            timestamp = System.currentTimeMillis()
        )
    }

    private fun getAppVersion(): String {
        return try {
            val pInfo = context.packageManager.getPackageInfo(context.packageName, 0)
            pInfo.versionName ?: "1.0.0"
        } catch (e: Exception) {
            "1.0.0"
        }
    }

    private fun createLoggingInterceptor(): HttpLoggingInterceptor {
        return HttpLoggingInterceptor { message ->
            Log.d("$TAG-Network", message)
        }.apply {
            level = if (config.isDebug) {
                HttpLoggingInterceptor.Level.BODY
            } else {
                HttpLoggingInterceptor.Level.NONE
            }
        }
    }

    // ==================== INTERCEPTORS ====================

    private inner class AuthInterceptor : Interceptor {
        override fun intercept(chain: Interceptor.Chain): Response {
            val originalRequest = chain.request()
            val builder = originalRequest.newBuilder()

            authToken?.let { token ->
                builder.addHeader("Authorization", "Bearer $token")
            }

            builder.addHeader("Content-Type", "application/json")
            builder.addHeader("User-Agent", "FinovaSDK-Android/${getAppVersion()}")

            return chain.proceed(builder.build())
        }
    }

    private inner class SecurityInterceptor : Interceptor {
        override fun intercept(chain: Interceptor.Chain): Response {
            val request = chain.request()
            
            // Add security headers
            val secureRequest = request.newBuilder()
                .addHeader("X-API-Version", Constants.API_VERSION)
                .addHeader("X-Client-Type", "android")
                .addHeader("X-Request-ID", generateRequestId())
                .build()

            return chain.proceed(secureRequest)
        }
        
        private fun generateRequestId(): String {
            return "${System.currentTimeMillis()}-${(Math.random() * 10000).toInt()}"
        }
    }

    // ==================== CLEANUP ====================

    fun destroy() {
        coroutineScope.cancel()
        miningJob?.cancel()
        INSTANCE = null
        Log.d(TAG, "FinovaClient destroyed")
    }
}

// ==================== DATA CLASSES ====================

data class FinovaConfig(
    val apiBaseUrl: String,
    val environment: Environment,
    val totalUsers: Int,
    val isDebug: Boolean = false
)

enum class Environment {
    DEVELOPMENT, STAGING, PRODUCTION
}

enum class ConnectionState {
    CONNECTING, CONNECTED, DISCONNECTED, ERROR
}

sealed class MiningState {
    object IDLE : MiningState()
    data class ACTIVE(val session: MiningSession) : MiningState()
    data class COMPLETED(val reward: MiningReward) : MiningState()
    data class ERROR(val error: String) : MiningState()
}

enum class RPTier {
    EXPLORER, CONNECTOR, INFLUENCER, LEADER, AMBASSADOR
}

enum class XPActivityType {
    ORIGINAL_POST, PHOTO_POST, VIDEO_POST, STORY_POST,
    MEANINGFUL_COMMENT, LIKE_REACT, SHARE_REPOST, FOLLOW_SUBSCRIBE,
    DAILY_LOGIN, COMPLETE_QUEST, ACHIEVE_MILESTONE, VIRAL_CONTENT
}

enum class SocialPlatform {
    TIKTOK, YOUTUBE, INSTAGRAM, TWITTER_X, FACEBOOK, APP
}

enum class SpecialCardType {
    DOUBLE_MINING, TRIPLE_MINING, MINING_FRENZY, ETERNAL_MINER,
    XP_DOUBLE, STREAK_SAVER, LEVEL_RUSH, XP_MAGNET,
    REFERRAL_BOOST, NETWORK_AMPLIFIER, AMBASSADOR_PASS, NETWORK_KING
}

enum class StakingDuration(val days: Int, val multiplier: Double) {
    FLEXIBLE(0, 1.0),
    WEEK_1(7, 1.1),
    WEEK_2(14, 1.15),
    MONTH_1(30, 1.25),
    MONTH_3(90, 1.4),
    MONTH_6(180, 1.6),
    YEAR_1(365, 2.0)
}
