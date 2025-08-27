package com.finova.example

import android.app.Application
import android.app.NotificationChannel
import android.app.NotificationManager
import android.content.Context
import android.os.Build
import androidx.hilt.work.HiltWorkerFactory
import androidx.work.Configuration
import androidx.work.WorkManager
import com.finova.sdk.FinovaSDK
import com.finova.sdk.config.FinovaConfig
import com.finova.sdk.config.Environment
import com.google.firebase.FirebaseApp
import com.google.firebase.crashlytics.FirebaseCrashlytics
import dagger.hilt.android.HiltAndroidApp
import timber.log.Timber
import javax.inject.Inject

@HiltAndroidApp
class FinovaApplication : Application(), Configuration.Provider {

    @Inject
    lateinit var workerFactory: HiltWorkerFactory

    @Inject
    lateinit var finovaSDK: FinovaSDK

    override fun onCreate() {
        super.onCreate()
        
        // Initialize logging
        initializeLogging()
        
        // Initialize Firebase
        initializeFirebase()
        
        // Initialize Finova SDK
        initializeFinovaSDK()
        
        // Initialize WorkManager
        initializeWorkManager()
        
        // Create notification channels
        createNotificationChannels()
        
        // Set up crash reporting
        setupCrashReporting()
        
        Timber.d("FinovaApplication initialized successfully")
    }

    private fun initializeLogging() {
        if (BuildConfig.DEBUG) {
            Timber.plant(object : Timber.DebugTree() {
                override fun createStackElementTag(element: StackTraceElement): String? {
                    return "Finova-${super.createStackElementTag(element)}"
                }
            })
        } else {
            // Plant a release tree that logs only warnings and errors
            Timber.plant(object : Timber.Tree() {
                override fun log(priority: Int, tag: String?, message: String, t: Throwable?) {
                    if (priority >= android.util.Log.WARN) {
                        // Log to crashlytics or other crash reporting service
                        FirebaseCrashlytics.getInstance().log("$tag: $message")
                        t?.let { FirebaseCrashlytics.getInstance().recordException(it) }
                    }
                }
            })
        }
    }

    private fun initializeFirebase() {
        try {
            FirebaseApp.initializeApp(this)
            Timber.d("Firebase initialized successfully")
        } catch (e: Exception) {
            Timber.e(e, "Failed to initialize Firebase")
        }
    }

    private fun initializeFinovaSDK() {
        try {
            val environment = when (BuildConfig.FINOVA_ENVIRONMENT) {
                "mainnet" -> Environment.MAINNET
                "testnet" -> Environment.TESTNET
                "devnet" -> Environment.DEVNET
                "staging" -> Environment.STAGING
                else -> Environment.TESTNET
            }

            val config = FinovaConfig.Builder()
                .setApiKey(BuildConfig.FINOVA_API_KEY)
                .setEnvironment(environment)
                .setApiBaseUrl(BuildConfig.API_BASE_URL)
                .setWebSocketUrl(BuildConfig.WEBSOCKET_URL)
                .setSolanaRpcUrl(BuildConfig.SOLANA_RPC_URL)
                .setDebugMode(BuildConfig.DEBUG_MODE)
                .setLogLevel(BuildConfig.LOG_LEVEL)
                .build()

            finovaSDK.initialize(this, config)
            Timber.d("Finova SDK initialized with environment: ${environment.name}")
        } catch (e: Exception) {
            Timber.e(e, "Failed to initialize Finova SDK")
            FirebaseCrashlytics.getInstance().recordException(e)
        }
    }

    private fun initializeWorkManager() {
        try {
            WorkManager.initialize(this, workManagerConfiguration)
            Timber.d("WorkManager initialized successfully")
        } catch (e: Exception) {
            Timber.e(e, "Failed to initialize WorkManager")
        }
    }

    override val workManagerConfiguration: Configuration
        get() = Configuration.Builder()
            .setWorkerFactory(workerFactory)
            .setMinimumLoggingLevel(
                if (BuildConfig.DEBUG) android.util.Log.DEBUG 
                else android.util.Log.ERROR
            )
            .build()

    private fun createNotificationChannels() {
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
            val notificationManager = getSystemService(Context.NOTIFICATION_SERVICE) as NotificationManager

            // Mining notifications channel
            val miningChannel = NotificationChannel(
                MINING_CHANNEL_ID,
                "Mining Notifications",
                NotificationManager.IMPORTANCE_DEFAULT
            ).apply {
                description = "Notifications about mining status and rewards"
                enableVibration(true)
                setSound(null, null) // Use default sound
            }

            // XP and achievements channel
            val xpChannel = NotificationChannel(
                XP_CHANNEL_ID,
                "XP & Achievements",
                NotificationManager.IMPORTANCE_DEFAULT
            ).apply {
                description = "Notifications about XP gains and achievements"
                enableVibration(true)
            }

            // Referral notifications channel
            val referralChannel = NotificationChannel(
                REFERRAL_CHANNEL_ID,
                "Referral Updates",
                NotificationManager.IMPORTANCE_DEFAULT
            ).apply {
                description = "Notifications about referral network activity"
                enableVibration(false)
            }

            // NFT and marketplace channel
            val nftChannel = NotificationChannel(
                NFT_CHANNEL_ID,
                "NFT & Marketplace",
                NotificationManager.IMPORTANCE_DEFAULT
            ).apply {
                description = "Notifications about NFT drops and marketplace activity"
                enableVibration(true)
            }

            // System and security channel
            val securityChannel = NotificationChannel(
                SECURITY_CHANNEL_ID,
                "Security & System",
                NotificationManager.IMPORTANCE_HIGH
            ).apply {
                description = "Important security and system notifications"
                enableVibration(true)
                enableLights(true)
            }

            // General announcements channel
            val generalChannel = NotificationChannel(
                GENERAL_CHANNEL_ID,
                "General Announcements",
                NotificationManager.IMPORTANCE_LOW
            ).apply {
                description = "General announcements and updates"
                enableVibration(false)
            }

            // Register all channels
            notificationManager.createNotificationChannels(listOf(
                miningChannel,
                xpChannel,
                referralChannel,
                nftChannel,
                securityChannel,
                generalChannel
            ))

            Timber.d("Notification channels created successfully")
        }
    }

    private fun setupCrashReporting() {
        try {
            FirebaseCrashlytics.getInstance().apply {
                setCrashlyticsCollectionEnabled(!BuildConfig.DEBUG)
                setCustomKey("environment", BuildConfig.FINOVA_ENVIRONMENT)
                setCustomKey("version_name", BuildConfig.VERSION_NAME)
                setCustomKey("version_code", BuildConfig.VERSION_CODE)
                setUserId("anonymous") // Will be updated when user logs in
            }
            Timber.d("Crash reporting setup completed")
        } catch (e: Exception) {
            Timber.e(e, "Failed to setup crash reporting")
        }
    }

    override fun onTerminate() {
        super.onTerminate()
        try {
            finovaSDK.cleanup()
            Timber.d("Application terminated, cleanup completed")
        } catch (e: Exception) {
            Timber.e(e, "Error during application cleanup")
        }
    }

    override fun onLowMemory() {
        super.onLowMemory()
        Timber.w("Application received low memory warning")
        try {
            // Trigger garbage collection
            System.gc()
            
            // Clear unnecessary caches
            finovaSDK.clearCaches()
            
            FirebaseCrashlytics.getInstance().log("Low memory condition triggered")
        } catch (e: Exception) {
            Timber.e(e, "Error handling low memory condition")
        }
    }

    override fun onTrimMemory(level: Int) {
        super.onTrimMemory(level)
        Timber.d("Memory trim requested with level: $level")
        
        try {
            when (level) {
                TRIM_MEMORY_UI_HIDDEN -> {
                    // App UI is no longer visible, trim UI-related resources
                    finovaSDK.trimUIResources()
                }
                TRIM_MEMORY_BACKGROUND -> {
                    // App is in background, trim non-essential resources
                    finovaSDK.trimBackgroundResources()
                }
                TRIM_MEMORY_MODERATE -> {
                    // System is moderately low on memory
                    finovaSDK.trimModerateResources()
                }
                TRIM_MEMORY_COMPLETE -> {
                    // System is very low on memory, trim everything possible
                    finovaSDK.trimAllResources()
                }
            }
        } catch (e: Exception) {
            Timber.e(e, "Error during memory trimming")
        }
    }

    companion object {
        // Notification channel IDs
        const val MINING_CHANNEL_ID = "mining_notifications"
        const val XP_CHANNEL_ID = "xp_notifications"
        const val REFERRAL_CHANNEL_ID = "referral_notifications"
        const val NFT_CHANNEL_ID = "nft_notifications"
        const val SECURITY_CHANNEL_ID = "security_notifications"
        const val GENERAL_CHANNEL_ID = "general_notifications"
        
        // Default notification channel (required by Firebase)
        const val DEFAULT_NOTIFICATION_CHANNEL_ID = "default_notifications"
        
        // Shared preferences keys
        const val PREFS_NAME = "finova_prefs"
        const val PREF_FIRST_LAUNCH = "first_launch"
        const val PREF_USER_ONBOARDED = "user_onboarded"
        const val PREF_MINING_ENABLED = "mining_enabled"
        const val PREF_NOTIFICATIONS_ENABLED = "notifications_enabled"
        const val PREF_BIOMETRIC_ENABLED = "biometric_enabled"
        
        // Work manager tags
        const val WORK_TAG_MINING = "mining_work"
        const val WORK_TAG_SYNC = "sync_work"
        const val WORK_TAG_BACKUP = "backup_work"
        const val WORK_TAG_CLEANUP = "cleanup_work"
        
        // Request codes
        const val REQUEST_CODE_BIOMETRIC = 1001
        const val REQUEST_CODE_CAMERA = 1002
        const val REQUEST_CODE_STORAGE = 1003
        const val REQUEST_CODE_NOTIFICATIONS = 1004
    }
}
