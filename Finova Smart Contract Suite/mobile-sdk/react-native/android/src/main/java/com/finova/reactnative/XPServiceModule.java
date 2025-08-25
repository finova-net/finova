package com.finova.reactnative;

import android.util.Log;

import com.facebook.react.bridge.Arguments;
import com.facebook.react.bridge.Callback;
import com.facebook.react.bridge.Promise;
import com.facebook.react.bridge.ReactApplicationContext;
import com.facebook.react.bridge.ReactContextBaseJavaModule;
import com.facebook.react.bridge.ReactMethod;
import com.facebook.react.bridge.ReadableMap;
import com.facebook.react.bridge.WritableArray;
import com.facebook.react.bridge.WritableMap;
import com.facebook.react.modules.core.DeviceEventManagerModule;

import org.json.JSONException;
import org.json.JSONObject;

import java.io.IOException;
import java.security.MessageDigest;
import java.security.NoSuchAlgorithmException;
import java.util.ArrayList;
import java.util.HashMap;
import java.util.List;
import java.util.Map;
import java.util.concurrent.ExecutorService;
import java.util.concurrent.Executors;

import javax.annotation.Nullable;

import okhttp3.*;

/**
 * Finova Network XP Service Module for React Native Android
 * Handles Experience Points system with gamification mechanics
 * 
 * Features:
 * - XP calculation with exponential regression
 * - Activity tracking and validation
 * - Level progression system
 * - Mining multiplier integration
 * - Quality score assessment
 * - Real-time XP events
 * 
 * @version 1.0.0
 * @author Finova Development Team
 */
public class XPServiceModule extends ReactContextBaseJavaModule {

    private static final String TAG = "FinovaXPService";
    private static final String MODULE_NAME = "FinovaXPService";
    
    // XP System Constants
    private static final int MAX_DAILY_XP = 10000;
    private static final double LEVEL_PROGRESSION_DECAY = 0.01;
    private static final int XP_STREAK_MAX_DAYS = 365;
    
    // Activity Types
    private static final String ACTIVITY_POST = "post";
    private static final String ACTIVITY_COMMENT = "comment";
    private static final String ACTIVITY_LIKE = "like";
    private static final String ACTIVITY_SHARE = "share";
    private static final String ACTIVITY_LOGIN = "login";
    private static final String ACTIVITY_VIRAL = "viral";
    
    // Level Tiers
    private static final Map<String, Integer[]> LEVEL_TIERS = new HashMap<String, Integer[]>() {{
        put("BRONZE", new Integer[]{1, 10});
        put("SILVER", new Integer[]{11, 25});
        put("GOLD", new Integer[]{26, 50});
        put("PLATINUM", new Integer[]{51, 75});
        put("DIAMOND", new Integer[]{76, 100});
        put("MYTHIC", new Integer[]{101, Integer.MAX_VALUE});
    }};
    
    private final ReactApplicationContext reactContext;
    private final ExecutorService executor;
    private final OkHttpClient httpClient;
    private final XPCalculator xpCalculator;
    private final ActivityTracker activityTracker;
    
    public XPServiceModule(ReactApplicationContext reactContext) {
        super(reactContext);
        this.reactContext = reactContext;
        this.executor = Executors.newFixedThreadPool(3);
        this.httpClient = new OkHttpClient.Builder()
                .connectTimeout(10, java.util.concurrent.TimeUnit.SECONDS)
                .readTimeout(30, java.util.concurrent.TimeUnit.SECONDS)
                .build();
        this.xpCalculator = new XPCalculator();
        this.activityTracker = new ActivityTracker();
        
        Log.d(TAG, "XPServiceModule initialized");
    }
    
    @Override
    public String getName() {
        return MODULE_NAME;
    }
    
    @Override
    public Map<String, Object> getConstants() {
        final Map<String, Object> constants = new HashMap<>();
        constants.put("ACTIVITY_TYPES", getActivityTypes());
        constants.put("LEVEL_TIERS", LEVEL_TIERS);
        constants.put("MAX_DAILY_XP", MAX_DAILY_XP);
        return constants;
    }
    
    /**
     * Calculate XP gain for user activity
     */
    @ReactMethod
    public void calculateXP(ReadableMap activityData, Promise promise) {
        executor.execute(() -> {
            try {
                String activityType = activityData.getString("type");
                String platform = activityData.hasKey("platform") ? activityData.getString("platform") : "app";
                String content = activityData.hasKey("content") ? activityData.getString("content") : "";
                int userLevel = activityData.hasKey("userLevel") ? activityData.getInt("userLevel") : 1;
                int streakDays = activityData.hasKey("streakDays") ? activityData.getInt("streakDays") : 1;
                double qualityScore = activityData.hasKey("qualityScore") ? activityData.getDouble("qualityScore") : 1.0;
                
                XPCalculationResult result = xpCalculator.calculateXPGain(
                    activityType, platform, content, userLevel, streakDays, qualityScore
                );
                
                WritableMap response = Arguments.createMap();
                response.putInt("baseXP", result.baseXP);
                response.putDouble("platformMultiplier", result.platformMultiplier);
                response.putDouble("qualityScore", result.qualityScore);
                response.putDouble("streakBonus", result.streakBonus);
                response.putDouble("levelProgression", result.levelProgression);
                response.putInt("totalXP", result.totalXP);
                response.putMap("breakdown", result.getBreakdownMap());
                
                promise.resolve(response);
                
                // Emit XP gain event
                emitXPEvent("xpGained", response);
                
            } catch (Exception e) {
                Log.e(TAG, "Error calculating XP: " + e.getMessage(), e);
                promise.reject("XP_CALCULATION_ERROR", e.getMessage());
            }
        });
    }
    
    /**
     * Get user's current XP level and progress
     */
    @ReactMethod
    public void getUserLevel(int totalXP, Promise promise) {
        executor.execute(() -> {
            try {
                UserLevel userLevel = xpCalculator.calculateUserLevel(totalXP);
                
                WritableMap response = Arguments.createMap();
                response.putInt("level", userLevel.level);
                response.putString("tier", userLevel.tier);
                response.putInt("currentLevelXP", userLevel.currentLevelXP);
                response.putInt("nextLevelXP", userLevel.nextLevelXP);
                response.putInt("progressXP", userLevel.progressXP);
                response.putDouble("progressPercentage", userLevel.progressPercentage);
                response.putDouble("miningMultiplier", userLevel.miningMultiplier);
                
                promise.resolve(response);
                
            } catch (Exception e) {
                Log.e(TAG, "Error getting user level: " + e.getMessage(), e);
                promise.reject("USER_LEVEL_ERROR", e.getMessage());
            }
        });
    }
    
    /**
     * Track user activity for XP validation
     */
    @ReactMethod
    public void trackActivity(ReadableMap activityData, Promise promise) {
        executor.execute(() -> {
            try {
                String userId = activityData.getString("userId");
                String activityType = activityData.getString("type");
                String platform = activityData.getString("platform");
                long timestamp = activityData.hasKey("timestamp") ? 
                    (long) activityData.getDouble("timestamp") : System.currentTimeMillis();
                
                // Validate activity limits
                boolean isValid = activityTracker.validateActivity(userId, activityType, platform, timestamp);
                
                if (!isValid) {
                    promise.reject("ACTIVITY_LIMIT_EXCEEDED", "Daily activity limit exceeded for " + activityType);
                    return;
                }
                
                // Track activity
                activityTracker.trackActivity(userId, activityType, platform, timestamp);
                
                WritableMap response = Arguments.createMap();
                response.putBoolean("success", true);
                response.putInt("remainingActivities", activityTracker.getRemainingActivities(userId, activityType));
                
                promise.resolve(response);
                
            } catch (Exception e) {
                Log.e(TAG, "Error tracking activity: " + e.getMessage(), e);
                promise.reject("ACTIVITY_TRACKING_ERROR", e.getMessage());
            }
        });
    }
    
    /**
     * Get XP leaderboard data
     */
    @ReactMethod
    public void getLeaderboard(String timeframe, int limit, Promise promise) {
        executor.execute(() -> {
            try {
                // This would typically call your backend API
                WritableArray leaderboard = Arguments.createArray();
                
                // Mock data - replace with actual API call
                for (int i = 0; i < Math.min(limit, 10); i++) {
                    WritableMap entry = Arguments.createMap();
                    entry.putInt("rank", i + 1);
                    entry.putString("username", "User" + (i + 1));
                    entry.putInt("totalXP", 10000 - (i * 500));
                    entry.putInt("level", 25 - (i * 2));
                    entry.putString("tier", "GOLD");
                    leaderboard.pushMap(entry);
                }
                
                promise.resolve(leaderboard);
                
            } catch (Exception e) {
                Log.e(TAG, "Error getting leaderboard: " + e.getMessage(), e);
                promise.reject("LEADERBOARD_ERROR", e.getMessage());
            }
        });
    }
    
    /**
     * Sync XP data with backend
     */
    @ReactMethod
    public void syncXPData(String userId, ReadableMap xpData, Promise promise) {
        executor.execute(() -> {
            try {
                JSONObject jsonData = new JSONObject();
                jsonData.put("userId", userId);
                jsonData.put("totalXP", xpData.getInt("totalXP"));
                jsonData.put("level", xpData.getInt("level"));
                jsonData.put("dailyXP", xpData.getInt("dailyXP"));
                jsonData.put("streakDays", xpData.getInt("streakDays"));
                jsonData.put("lastActivity", xpData.getDouble("lastActivity"));
                
                RequestBody body = RequestBody.create(
                    jsonData.toString(),
                    MediaType.get("application/json; charset=utf-8")
                );
                
                Request request = new Request.Builder()
                    .url("https://api.finova.network/v1/xp/sync")
                    .post(body)
                    .addHeader("Authorization", "Bearer " + getAuthToken())
                    .build();
                
                Response response = httpClient.newCall(request).execute();
                
                if (response.isSuccessful()) {
                    promise.resolve(true);
                } else {
                    promise.reject("SYNC_ERROR", "Failed to sync XP data: " + response.message());
                }
                
            } catch (Exception e) {
                Log.e(TAG, "Error syncing XP data: " + e.getMessage(), e);
                promise.reject("SYNC_ERROR", e.getMessage());
            }
        });
    }
    
    /**
     * Get daily quest progress
     */
    @ReactMethod
    public void getDailyQuests(String userId, Promise promise) {
        executor.execute(() -> {
            try {
                WritableArray quests = Arguments.createArray();
                
                // Mock daily quests - replace with actual data
                String[] questTypes = {"post_content", "engage_social", "refer_friend", "daily_login"};
                String[] questTitles = {"Create Original Content", "Engage on Social Media", "Invite a Friend", "Daily Check-in"};
                int[] questXP = {100, 50, 200, 25};
                
                for (int i = 0; i < questTypes.length; i++) {
                    WritableMap quest = Arguments.createMap();
                    quest.putString("id", questTypes[i]);
                    quest.putString("title", questTitles[i]);
                    quest.putString("description", "Complete this quest to earn " + questXP[i] + " XP");
                    quest.putInt("xpReward", questXP[i]);
                    quest.putInt("progress", 0);
                    quest.putInt("target", 1);
                    quest.putBoolean("completed", false);
                    quests.pushMap(quest);
                }
                
                promise.resolve(quests);
                
            } catch (Exception e) {
                Log.e(TAG, "Error getting daily quests: " + e.getMessage(), e);
                promise.reject("DAILY_QUESTS_ERROR", e.getMessage());
            }
        });
    }
    
    // Helper Methods
    
    private Map<String, Object> getActivityTypes() {
        Map<String, Object> activities = new HashMap<>();
        activities.put("POST", ACTIVITY_POST);
        activities.put("COMMENT", ACTIVITY_COMMENT);
        activities.put("LIKE", ACTIVITY_LIKE);
        activities.put("SHARE", ACTIVITY_SHARE);
        activities.put("LOGIN", ACTIVITY_LOGIN);
        activities.put("VIRAL", ACTIVITY_VIRAL);
        return activities;
    }
    
    private void emitXPEvent(String eventName, WritableMap data) {
        reactContext
            .getJSModule(DeviceEventManagerModule.RCTDeviceEventEmitter.class)
            .emit(eventName, data);
    }
    
    private String getAuthToken() {
        // Get authentication token from storage or context
        return "mock_token_for_development";
    }
    
    // Inner Classes
    
    /**
     * XP Calculator handles all XP-related calculations
     */
    private static class XPCalculator {
        
        private final Map<String, Integer> baseXPValues = new HashMap<String, Integer>() {{
            put(ACTIVITY_POST, 50);
            put(ACTIVITY_COMMENT, 25);
            put(ACTIVITY_LIKE, 5);
            put(ACTIVITY_SHARE, 15);
            put(ACTIVITY_LOGIN, 10);
            put(ACTIVITY_VIRAL, 1000);
        }};
        
        private final Map<String, Double> platformMultipliers = new HashMap<String, Double>() {{
            put("tiktok", 1.3);
            put("instagram", 1.2);
            put("youtube", 1.4);
            put("facebook", 1.1);
            put("twitter", 1.2);
            put("app", 1.0);
        }};
        
        public XPCalculationResult calculateXPGain(String activityType, String platform, 
                                                  String content, int userLevel, 
                                                  int streakDays, double qualityScore) {
            
            int baseXP = baseXPValues.getOrDefault(activityType, 0);
            double platformMultiplier = platformMultipliers.getOrDefault(platform.toLowerCase(), 1.0);
            double streakBonus = Math.min(1.0 + (streakDays * 0.1), 3.0);
            double levelProgression = Math.exp(-LEVEL_PROGRESSION_DECAY * userLevel);
            
            int totalXP = (int) Math.round(baseXP * platformMultiplier * qualityScore * 
                                         streakBonus * levelProgression);
            
            return new XPCalculationResult(baseXP, platformMultiplier, qualityScore, 
                                         streakBonus, levelProgression, totalXP);
        }
        
        public UserLevel calculateUserLevel(int totalXP) {
            int level = 1;
            int currentLevelXP = 0;
            int nextLevelXP = 100;
            
            // Calculate level based on exponential progression
            while (totalXP >= nextLevelXP && level < 200) {
                level++;
                currentLevelXP = nextLevelXP;
                nextLevelXP = (int) (100 * Math.pow(1.1, level - 1));
            }
            
            String tier = getTierForLevel(level);
            double miningMultiplier = 1.0 + (level * 0.01);
            int progressXP = totalXP - currentLevelXP;
            double progressPercentage = (double) progressXP / (nextLevelXP - currentLevelXP) * 100;
            
            return new UserLevel(level, tier, currentLevelXP, nextLevelXP, 
                               progressXP, progressPercentage, miningMultiplier);
        }
        
        private String getTierForLevel(int level) {
            for (Map.Entry<String, Integer[]> entry : LEVEL_TIERS.entrySet()) {
                if (level >= entry.getValue()[0] && level <= entry.getValue()[1]) {
                    return entry.getKey();
                }
            }
            return "BRONZE";
        }
    }
    
    /**
     * Activity Tracker manages daily limits and validation
     */
    private static class ActivityTracker {
        
        private final Map<String, Map<String, Integer>> dailyActivityCount = new HashMap<>();
        private final Map<String, Integer> activityLimits = new HashMap<String, Integer>() {{
            put(ACTIVITY_POST, 20);
            put(ACTIVITY_COMMENT, 100);
            put(ACTIVITY_LIKE, 200);
            put(ACTIVITY_SHARE, 50);
            put(ACTIVITY_LOGIN, 1);
        }};
        
        public boolean validateActivity(String userId, String activityType, 
                                      String platform, long timestamp) {
            String key = userId + "_" + getCurrentDay(timestamp);
            Map<String, Integer> userActivities = dailyActivityCount.getOrDefault(key, new HashMap<>());
            
            int currentCount = userActivities.getOrDefault(activityType, 0);
            int limit = activityLimits.getOrDefault(activityType, Integer.MAX_VALUE);
            
            return currentCount < limit;
        }
        
        public void trackActivity(String userId, String activityType, 
                                String platform, long timestamp) {
            String key = userId + "_" + getCurrentDay(timestamp);
            Map<String, Integer> userActivities = dailyActivityCount.getOrDefault(key, new HashMap<>());
            
            int currentCount = userActivities.getOrDefault(activityType, 0);
            userActivities.put(activityType, currentCount + 1);
            dailyActivityCount.put(key, userActivities);
        }
        
        public int getRemainingActivities(String userId, String activityType) {
            String key = userId + "_" + getCurrentDay(System.currentTimeMillis());
            Map<String, Integer> userActivities = dailyActivityCount.getOrDefault(key, new HashMap<>());
            
            int currentCount = userActivities.getOrDefault(activityType, 0);
            int limit = activityLimits.getOrDefault(activityType, Integer.MAX_VALUE);
            
            return Math.max(0, limit - currentCount);
        }
        
        private String getCurrentDay(long timestamp) {
            return String.valueOf(timestamp / (24 * 60 * 60 * 1000));
        }
    }
    
    /**
     * XP Calculation Result data class
     */
    private static class XPCalculationResult {
        public final int baseXP;
        public final double platformMultiplier;
        public final double qualityScore;
        public final double streakBonus;
        public final double levelProgression;
        public final int totalXP;
        
        public XPCalculationResult(int baseXP, double platformMultiplier, double qualityScore,
                                 double streakBonus, double levelProgression, int totalXP) {
            this.baseXP = baseXP;
            this.platformMultiplier = platformMultiplier;
            this.qualityScore = qualityScore;
            this.streakBonus = streakBonus;
            this.levelProgression = levelProgression;
            this.totalXP = totalXP;
        }
        
        public WritableMap getBreakdownMap() {
            WritableMap breakdown = Arguments.createMap();
            breakdown.putInt("baseXP", baseXP);
            breakdown.putDouble("platformMultiplier", platformMultiplier);
            breakdown.putDouble("qualityScore", qualityScore);
            breakdown.putDouble("streakBonus", streakBonus);
            breakdown.putDouble("levelProgression", levelProgression);
            return breakdown;
        }
    }
    
    /**
     * User Level data class
     */
    private static class UserLevel {
        public final int level;
        public final String tier;
        public final int currentLevelXP;
        public final int nextLevelXP;
        public final int progressXP;
        public final double progressPercentage;
        public final double miningMultiplier;
        
        public UserLevel(int level, String tier, int currentLevelXP, int nextLevelXP,
                        int progressXP, double progressPercentage, double miningMultiplier) {
            this.level = level;
            this.tier = tier;
            this.currentLevelXP = currentLevelXP;
            this.nextLevelXP = nextLevelXP;
            this.progressXP = progressXP;
            this.progressPercentage = progressPercentage;
            this.miningMultiplier = miningMultiplier;
        }
    }
}
