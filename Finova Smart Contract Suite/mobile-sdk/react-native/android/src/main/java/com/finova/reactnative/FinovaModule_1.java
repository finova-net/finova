package com.finova.reactnative;

import com.facebook.react.bridge.*;
import com.facebook.react.modules.core.DeviceEventManagerModule;
import android.content.Context;
import android.content.SharedPreferences;
import android.util.Log;
import org.json.JSONObject;
import org.json.JSONException;
import java.util.HashMap;
import java.util.Map;
import java.util.concurrent.ExecutorService;
import java.util.concurrent.Executors;

/**
 * Main Finova Module - Core SDK functionality
 */
@ReactModule(name = FinovaModule.NAME)
public class FinovaModule extends ReactContextBaseJavaModule {
    public static final String NAME = "FinovaSDK";
    private static final String TAG = "FinovaModule";
    private final ReactApplicationContext reactContext;
    private final SharedPreferences prefs;
    private final ExecutorService executor;
    private boolean isInitialized = false;
    private String apiKey;
    private String userId;
    private String accessToken;

    public FinovaModule(ReactApplicationContext reactContext) {
        super(reactContext);
        this.reactContext = reactContext;
        this.prefs = reactContext.getSharedPreferences("finova_prefs", Context.MODE_PRIVATE);
        this.executor = Executors.newFixedThreadPool(4);
    }

    @Override
    public String getName() {
        return NAME;
    }

    @Override
    public Map<String, Object> getConstants() {
        Map<String, Object> constants = new HashMap<>();
        constants.put("SDK_VERSION", "1.0.0");
        constants.put("API_VERSION", "v1");
        constants.put("NETWORK", "mainnet");
        constants.put("MIN_ANDROID_VERSION", 21);
        return constants;
    }

    @ReactMethod
    public void initialize(ReadableMap config, Promise promise) {
        try {
            if (isInitialized) {
                promise.resolve(createSuccessResponse("SDK already initialized"));
                return;
            }

            String apiKey = config.getString("apiKey");
            String environment = config.hasKey("environment") ? config.getString("environment") : "production";
            boolean enableLogging = config.hasKey("enableLogging") && config.getBoolean("enableLogging");

            if (apiKey == null || apiKey.isEmpty()) {
                promise.reject("INVALID_API_KEY", "API key is required");
                return;
            }

            this.apiKey = apiKey;
            
            // Store initialization config
            SharedPreferences.Editor editor = prefs.edit();
            editor.putString("api_key", apiKey);
            editor.putString("environment", environment);
            editor.putBoolean("logging_enabled", enableLogging);
            editor.apply();

            // Initialize components
            initializeComponents();
            isInitialized = true;

            if (enableLogging) {
                Log.d(TAG, "Finova SDK initialized successfully");
            }

            WritableMap response = createSuccessResponse("SDK initialized successfully");
            response.putString("sdkVersion", "1.0.0");
            response.putString("environment", environment);
            promise.resolve(response);

        } catch (Exception e) {
            Log.e(TAG, "Failed to initialize SDK", e);
            promise.reject("INIT_FAILED", e.getMessage());
        }
    }

    @ReactMethod
    public void authenticateUser(ReadableMap userData, Promise promise) {
        if (!checkInitialization(promise)) return;

        executor.execute(() -> {
            try {
                String walletAddress = userData.getString("walletAddress");
                String signature = userData.getString("signature");
                String message = userData.getString("message");

                if (walletAddress == null || signature == null || message == null) {
                    promise.reject("INVALID_AUTH_DATA", "Wallet address, signature, and message are required");
                    return;
                }

                // Simulate authentication process
                JSONObject authRequest = new JSONObject();
                authRequest.put("walletAddress", walletAddress);
                authRequest.put("signature", signature);
                authRequest.put("message", message);
                authRequest.put("timestamp", System.currentTimeMillis());

                // Mock API call - replace with actual authentication
                Thread.sleep(1000); // Simulate network delay

                // Generate mock response
                String mockUserId = "user_" + walletAddress.substring(0, 8);
                String mockToken = generateMockToken(walletAddress);

                this.userId = mockUserId;
                this.accessToken = mockToken;

                // Store authentication data
                SharedPreferences.Editor editor = prefs.edit();
                editor.putString("user_id", mockUserId);
                editor.putString("access_token", mockToken);
                editor.putString("wallet_address", walletAddress);
                editor.putLong("auth_timestamp", System.currentTimeMillis());
                editor.apply();

                WritableMap response = createSuccessResponse("Authentication successful");
                response.putString("userId", mockUserId);
                response.putString("accessToken", mockToken);
                response.putString("walletAddress", walletAddress);
                
                promise.resolve(response);

                // Emit authentication event
                sendEvent("onAuthenticationSuccess", response);

            } catch (Exception e) {
                Log.e(TAG, "Authentication failed", e);
                promise.reject("AUTH_FAILED", e.getMessage());
            }
        });
    }

    @ReactMethod
    public void getUserProfile(Promise promise) {
        if (!checkInitialization(promise) || !checkAuthentication(promise)) return;

        executor.execute(() -> {
            try {
                // Mock user profile data - replace with actual API call
                WritableMap profile = Arguments.createMap();
                profile.putString("userId", userId);
                profile.putString("walletAddress", prefs.getString("wallet_address", ""));
                profile.putInt("level", 15);
                profile.putInt("xp", 7500);
                profile.putInt("rp", 2300);
                profile.putDouble("finBalance", 145.75);
                profile.putDouble("sfinBalance", 89.25);
                profile.putInt("miningRate", 120);
                profile.putString("tier", "Silver");
                profile.putInt("referralCount", 8);
                profile.putBoolean("kycVerified", true);
                profile.putLong("joinDate", System.currentTimeMillis() - (30L * 24 * 60 * 60 * 1000));

                WritableArray badges = Arguments.createArray();
                badges.pushString("early_adopter");
                badges.pushString("content_creator");
                profile.putArray("badges", badges);

                WritableMap stats = Arguments.createMap();
                stats.putInt("totalPosts", 156);
                stats.putInt("totalLikes", 2341);
                stats.putInt("streakDays", 12);
                stats.putDouble("qualityScore", 0.87);
                profile.putMap("stats", stats);

                promise.resolve(profile);

            } catch (Exception e) {
                Log.e(TAG, "Failed to get user profile", e);
                promise.reject("PROFILE_FETCH_FAILED", e.getMessage());
            }
        });
    }

    @ReactMethod
    public void updateUserProfile(ReadableMap updates, Promise promise) {
        if (!checkInitialization(promise) || !checkAuthentication(promise)) return;

        executor.execute(() -> {
            try {
                // Mock profile update - replace with actual API call
                Thread.sleep(500);

                WritableMap response = createSuccessResponse("Profile updated successfully");
                promise.resolve(response);

                // Emit profile update event
                sendEvent("onProfileUpdated", updates.toHashMap());

            } catch (Exception e) {
                Log.e(TAG, "Failed to update profile", e);
                promise.reject("PROFILE_UPDATE_FAILED", e.getMessage());
            }
        });
    }

    @ReactMethod
    public void logout(Promise promise) {
        try {
            // Clear stored data
            SharedPreferences.Editor editor = prefs.edit();
            editor.remove("user_id");
            editor.remove("access_token");
            editor.remove("wallet_address");
            editor.remove("auth_timestamp");
            editor.apply();

            this.userId = null;
            this.accessToken = null;

            WritableMap response = createSuccessResponse("Logged out successfully");
            promise.resolve(response);

            // Emit logout event
            sendEvent("onLogout", null);

        } catch (Exception e) {
            Log.e(TAG, "Failed to logout", e);
            promise.reject("LOGOUT_FAILED", e.getMessage());
        }
    }

    @ReactMethod
    public void getAppInfo(Promise promise) {
        try {
            WritableMap info = Arguments.createMap();
            info.putString("version", "1.0.0");
            info.putString("buildNumber", "100");
            info.putString("environment", prefs.getString("environment", "production"));
            info.putBoolean("isInitialized", isInitialized);
            info.putBoolean("isAuthenticated", userId != null);
            info.putLong("timestamp", System.currentTimeMillis());

            promise.resolve(info);
        } catch (Exception e) {
            promise.reject("INFO_FETCH_FAILED", e.getMessage());
        }
    }

    // Helper methods
    private void initializeComponents() {
        // Initialize SDK components
        Log.d(TAG, "Initializing Finova SDK components");
    }

    private boolean checkInitialization(Promise promise) {
        if (!isInitialized) {
            promise.reject("NOT_INITIALIZED", "SDK not initialized. Call initialize() first.");
            return false;
        }
        return true;
    }

    private boolean checkAuthentication(Promise promise) {
        if (userId == null || accessToken == null) {
            promise.reject("NOT_AUTHENTICATED", "User not authenticated. Call authenticateUser() first.");
            return false;
        }
        return true;
    }

    private WritableMap createSuccessResponse(String message) {
        WritableMap response = Arguments.createMap();
        response.putBoolean("success", true);
        response.putString("message", message);
        response.putLong("timestamp", System.currentTimeMillis());
        return response;
    }

    private String generateMockToken(String walletAddress) {
        return "ft_" + walletAddress.substring(0, 8) + "_" + System.currentTimeMillis();
    }

    private void sendEvent(String eventName, Object data) {
        if (reactContext.hasActiveCatalystInstance()) {
            reactContext
                .getJSModule(DeviceEventManagerModule.RCTDeviceEventEmitter.class)
                .emit(eventName, data);
        }
    }

    @Override
    public void onCatalystInstanceDestroy() {
        super.onCatalystInstanceDestroy();
        if (executor != null && !executor.isShutdown()) {
            executor.shutdown();
        }
    }
}
