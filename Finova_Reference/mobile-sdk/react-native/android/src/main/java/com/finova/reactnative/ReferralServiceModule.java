package com.finova.reactnative;

import androidx.annotation.NonNull;
import com.facebook.react.bridge.*;
import com.facebook.react.modules.core.DeviceEventManagerModule;
import com.facebook.react.bridge.ReactApplicationContext;
import com.facebook.react.bridge.ReactContextBaseJavaModule;
import com.facebook.react.bridge.ReactMethod;
import com.facebook.react.bridge.Promise;
import com.facebook.react.bridge.WritableMap;
import com.facebook.react.bridge.Arguments;
import com.facebook.react.bridge.ReadableMap;

import org.json.JSONObject;
import org.json.JSONException;

import java.util.*;
import java.util.concurrent.*;
import java.security.MessageDigest;
import java.security.NoSuchAlgorithmException;
import java.text.SimpleDateFormat;

import okhttp3.*;
import okio.IOException;

public class ReferralServiceModule extends ReactContextBaseJavaModule {
    
    private static final String MODULE_NAME = "FinovaReferralService";
    private static final String BASE_API_URL = "https://api.finova.network";
    private static final int MAX_NETWORK_DEPTH = 3;
    private static final double REGRESSION_COEFFICIENT = 0.0001;
    
    private final ReactApplicationContext reactContext;
    private final OkHttpClient httpClient;
    private final ExecutorService executorService;
    private String currentUserId;
    private String authToken;
    
    // RP Tier thresholds
    private static final int[] RP_TIER_THRESHOLDS = {0, 1000, 5000, 15000, 50000};
    private static final String[] RP_TIER_NAMES = {"Explorer", "Connector", "Influencer", "Leader", "Ambassador"};
    private static final double[] RP_MINING_BONUSES = {0.0, 0.2, 0.5, 1.0, 2.0};
    private static final double[] RP_REFERRAL_BONUSES = {0.1, 0.15, 0.2, 0.25, 0.3};

    public ReferralServiceModule(ReactApplicationContext reactContext) {
        super(reactContext);
        this.reactContext = reactContext;
        this.httpClient = new OkHttpClient.Builder()
            .connectTimeout(30, TimeUnit.SECONDS)
            .readTimeout(30, TimeUnit.SECONDS)
            .writeTimeout(30, TimeUnit.SECONDS)
            .build();
        this.executorService = Executors.newCachedThreadPool();
    }

    @NonNull
    @Override
    public String getName() {
        return MODULE_NAME;
    }

    // Initialize referral service
    @ReactMethod
    public void initialize(String userId, String token, Promise promise) {
        try {
            this.currentUserId = userId;
            this.authToken = token;
            
            WritableMap result = Arguments.createMap();
            result.putString("status", "initialized");
            result.putString("userId", userId);
            result.putDouble("timestamp", System.currentTimeMillis());
            
            promise.resolve(result);
        } catch (Exception e) {
            promise.reject("INIT_ERROR", e.getMessage(), e);
        }
    }

    // Generate referral code
    @ReactMethod
    public void generateReferralCode(String customCode, Promise promise) {
        executorService.execute(() -> {
            try {
                String referralCode = customCode != null && !customCode.isEmpty() ? 
                    customCode : generateUniqueCode();
                
                JSONObject requestBody = new JSONObject();
                requestBody.put("userId", currentUserId);
                requestBody.put("referralCode", referralCode);
                requestBody.put("timestamp", System.currentTimeMillis());

                Request request = new Request.Builder()
                    .url(BASE_API_URL + "/referral/generate-code")
                    .addHeader("Authorization", "Bearer " + authToken)
                    .post(RequestBody.create(requestBody.toString(), MediaType.parse("application/json")))
                    .build();

                Response response = httpClient.newCall(request).execute();
                if (response.isSuccessful()) {
                    JSONObject responseData = new JSONObject(response.body().string());
                    
                    WritableMap result = Arguments.createMap();
                    result.putString("referralCode", responseData.getString("referralCode"));
                    result.putString("referralLink", responseData.getString("referralLink"));
                    result.putDouble("expiresAt", responseData.optDouble("expiresAt", 0));
                    result.putBoolean("isCustom", customCode != null && !customCode.isEmpty());
                    
                    promise.resolve(result);
                } else {
                    promise.reject("API_ERROR", "Failed to generate referral code: " + response.code());
                }
                
            } catch (Exception e) {
                promise.reject("GENERATE_ERROR", e.getMessage(), e);
            }
        });
    }

    // Apply referral code
    @ReactMethod
    public void applyReferralCode(String referralCode, Promise promise) {
        executorService.execute(() -> {
            try {
                JSONObject requestBody = new JSONObject();
                requestBody.put("userId", currentUserId);
                requestBody.put("referralCode", referralCode);
                requestBody.put("timestamp", System.currentTimeMillis());

                Request request = new Request.Builder()
                    .url(BASE_API_URL + "/referral/apply-code")
                    .addHeader("Authorization", "Bearer " + authToken)
                    .post(RequestBody.create(requestBody.toString(), MediaType.parse("application/json")))
                    .build();

                Response response = httpClient.newCall(request).execute();
                if (response.isSuccessful()) {
                    JSONObject responseData = new JSONObject(response.body().string());
                    
                    WritableMap result = Arguments.createMap();
                    result.putBoolean("success", responseData.getBoolean("success"));
                    result.putString("referrerId", responseData.optString("referrerId", ""));
                    result.putDouble("bonusRP", responseData.optDouble("bonusRP", 0));
                    result.putDouble("bonusFIN", responseData.optDouble("bonusFIN", 0));
                    result.putString("tierUpgrade", responseData.optString("tierUpgrade", ""));
                    
                    if (responseData.getBoolean("success")) {
                        emitEvent("referralApplied", result);
                    }
                    
                    promise.resolve(result);
                } else {
                    promise.reject("API_ERROR", "Failed to apply referral code: " + response.code());
                }
                
            } catch (Exception e) {
                promise.reject("APPLY_ERROR", e.getMessage(), e);
            }
        });
    }

    // Get referral statistics
    @ReactMethod
    public void getReferralStats(Promise promise) {
        executorService.execute(() -> {
            try {
                Request request = new Request.Builder()
                    .url(BASE_API_URL + "/referral/stats/" + currentUserId)
                    .addHeader("Authorization", "Bearer " + authToken)
                    .get()
                    .build();

                Response response = httpClient.newCall(request).execute();
                if (response.isSuccessful()) {
                    JSONObject responseData = new JSONObject(response.body().string());
                    
                    WritableMap result = Arguments.createMap();
                    result.putDouble("totalRP", responseData.optDouble("totalRP", 0));
                    result.putString("currentTier", responseData.optString("currentTier", "Explorer"));
                    result.putDouble("tierProgress", responseData.optDouble("tierProgress", 0));
                    result.putDouble("nextTierThreshold", responseData.optDouble("nextTierThreshold", 1000));
                    result.putInt("totalReferrals", responseData.optInt("totalReferrals", 0));
                    result.putInt("activeReferrals", responseData.optInt("activeReferrals", 0));
                    result.putInt("networkSize", responseData.optInt("networkSize", 0));
                    result.putDouble("networkQuality", responseData.optDouble("networkQuality", 0));
                    result.putDouble("dailyEarnings", responseData.optDouble("dailyEarnings", 0));
                    result.putDouble("monthlyEarnings", responseData.optDouble("monthlyEarnings", 0));
                    result.putDouble("miningBonus", responseData.optDouble("miningBonus", 0));
                    
                    promise.resolve(result);
                } else {
                    promise.reject("API_ERROR", "Failed to get referral stats: " + response.code());
                }
                
            } catch (Exception e) {
                promise.reject("STATS_ERROR", e.getMessage(), e);
            }
        });
    }

    // Get referral network tree
    @ReactMethod
    public void getReferralNetwork(int depth, Promise promise) {
        executorService.execute(() -> {
            try {
                int safeDepth = Math.min(depth, MAX_NETWORK_DEPTH);
                
                Request request = new Request.Builder()
                    .url(BASE_API_URL + "/referral/network/" + currentUserId + "?depth=" + safeDepth)
                    .addHeader("Authorization", "Bearer " + authToken)
                    .get()
                    .build();

                Response response = httpClient.newCall(request).execute();
                if (response.isSuccessful()) {
                    JSONObject responseData = new JSONObject(response.body().string());
                    
                    WritableMap result = Arguments.createMap();
                    result.putMap("networkTree", parseNetworkTree(responseData.optJSONObject("networkTree")));
                    result.putInt("totalNodes", responseData.optInt("totalNodes", 0));
                    result.putDouble("networkValue", responseData.optDouble("networkValue", 0));
                    result.putDouble("networkQualityScore", responseData.optDouble("networkQualityScore", 0));
                    
                    promise.resolve(result);
                } else {
                    promise.reject("API_ERROR", "Failed to get referral network: " + response.code());
                }
                
            } catch (Exception e) {
                promise.reject("NETWORK_ERROR", e.getMessage(), e);
            }
        });
    }

    // Calculate RP value with regression
    @ReactMethod
    public void calculateRPValue(ReadableMap networkData, Promise promise) {
        try {
            int totalNetworkSize = networkData.getInt("totalNetworkSize");
            double networkQuality = networkData.getDouble("networkQualityScore");
            int activeUsers = networkData.getInt("activeUsers");
            double avgActivityLevel = networkData.getDouble("avgActivityLevel");
            
            // Calculate direct RP
            double directRP = networkData.getDouble("directReferralPoints");
            
            // Calculate indirect network points
            double indirectRP = networkData.getDouble("level2Points") * 0.3 + 
                               networkData.getDouble("level3Points") * 0.1;
            
            // Network quality bonus
            double diversityScore = networkData.getDouble("networkDiversity");
            double avgReferralLevel = networkData.getDouble("avgReferralLevel");
            double retentionRate = networkData.getDouble("retentionRate");
            
            double qualityBonus = diversityScore * avgReferralLevel * retentionRate;
            
            // Calculate regression factor
            double regressionFactor = Math.exp(-REGRESSION_COEFFICIENT * totalNetworkSize * networkQuality);
            
            // Final RP calculation
            double finalRP = (directRP + indirectRP) * qualityBonus * regressionFactor;
            
            // Determine tier
            String tierName = determineTier(finalRP);
            int tierIndex = getTierIndex(tierName);
            
            WritableMap result = Arguments.createMap();
            result.putDouble("totalRP", finalRP);
            result.putDouble("directRP", directRP);
            result.putDouble("indirectRP", indirectRP);
            result.putDouble("qualityBonus", qualityBonus);
            result.putDouble("regressionFactor", regressionFactor);
            result.putString("tierName", tierName);
            result.putInt("tierIndex", tierIndex);
            result.putDouble("miningBonus", RP_MINING_BONUSES[tierIndex]);
            result.putDouble("referralBonus", RP_REFERRAL_BONUSES[tierIndex]);
            result.putDouble("nextTierThreshold", getNextTierThreshold(tierIndex));
            
            promise.resolve(result);
            
        } catch (Exception e) {
            promise.reject("CALCULATION_ERROR", e.getMessage(), e);
        }
    }

    // Track referral activity
    @ReactMethod
    public void trackReferralActivity(String activityType, ReadableMap activityData, Promise promise) {
        executorService.execute(() -> {
            try {
                JSONObject requestBody = new JSONObject();
                requestBody.put("userId", currentUserId);
                requestBody.put("activityType", activityType);
                requestBody.put("activityData", convertReadableMapToJSONObject(activityData));
                requestBody.put("timestamp", System.currentTimeMillis());

                Request request = new Request.Builder()
                    .url(BASE_API_URL + "/referral/track-activity")
                    .addHeader("Authorization", "Bearer " + authToken)
                    .post(RequestBody.create(requestBody.toString(), MediaType.parse("application/json")))
                    .build();

                Response response = httpClient.newCall(request).execute();
                if (response.isSuccessful()) {
                    JSONObject responseData = new JSONObject(response.body().string());
                    
                    WritableMap result = Arguments.createMap();
                    result.putBoolean("tracked", true);
                    result.putDouble("rpEarned", responseData.optDouble("rpEarned", 0));
                    result.putDouble("referralBonus", responseData.optDouble("referralBonus", 0));
                    result.putBoolean("tierChanged", responseData.optBoolean("tierChanged", false));
                    result.putString("newTier", responseData.optString("newTier", ""));
                    
                    if (responseData.optBoolean("tierChanged", false)) {
                        emitEvent("tierUpgrade", result);
                    }
                    
                    promise.resolve(result);
                } else {
                    promise.reject("API_ERROR", "Failed to track activity: " + response.code());
                }
                
            } catch (Exception e) {
                promise.reject("TRACK_ERROR", e.getMessage(), e);
            }
        });
    }

    // Get leaderboard
    @ReactMethod
    public void getLeaderboard(String period, int limit, Promise promise) {
        executorService.execute(() -> {
            try {
                Request request = new Request.Builder()
                    .url(BASE_API_URL + "/referral/leaderboard?period=" + period + "&limit=" + limit)
                    .addHeader("Authorization", "Bearer " + authToken)
                    .get()
                    .build();

                Response response = httpClient.newCall(request).execute();
                if (response.isSuccessful()) {
                    JSONObject responseData = new JSONObject(response.body().string());
                    
                    WritableMap result = Arguments.createMap();
                    result.putArray("leaderboard", parseLeaderboard(responseData.getJSONArray("leaderboard")));
                    result.putInt("userRank", responseData.optInt("userRank", 0));
                    result.putString("period", period);
                    result.putDouble("timestamp", System.currentTimeMillis());
                    
                    promise.resolve(result);
                } else {
                    promise.reject("API_ERROR", "Failed to get leaderboard: " + response.code());
                }
                
            } catch (Exception e) {
                promise.reject("LEADERBOARD_ERROR", e.getMessage(), e);
            }
        });
    }

    // Claim referral rewards
    @ReactMethod
    public void claimReferralRewards(Promise promise) {
        executorService.execute(() -> {
            try {
                JSONObject requestBody = new JSONObject();
                requestBody.put("userId", currentUserId);
                requestBody.put("timestamp", System.currentTimeMillis());

                Request request = new Request.Builder()
                    .url(BASE_API_URL + "/referral/claim-rewards")
                    .addHeader("Authorization", "Bearer " + authToken)
                    .post(RequestBody.create(requestBody.toString(), MediaType.parse("application/json")))
                    .build();

                Response response = httpClient.newCall(request).execute();
                if (response.isSuccessful()) {
                    JSONObject responseData = new JSONObject(response.body().string());
                    
                    WritableMap result = Arguments.createMap();
                    result.putBoolean("success", responseData.getBoolean("success"));
                    result.putDouble("finClaimed", responseData.optDouble("finClaimed", 0));
                    result.putDouble("rpClaimed", responseData.optDouble("rpClaimed", 0));
                    result.putString("transactionHash", responseData.optString("transactionHash", ""));
                    result.putDouble("nextClaimTime", responseData.optDouble("nextClaimTime", 0));
                    
                    if (responseData.getBoolean("success")) {
                        emitEvent("rewardsClaimed", result);
                    }
                    
                    promise.resolve(result);
                } else {
                    promise.reject("API_ERROR", "Failed to claim rewards: " + response.code());
                }
                
            } catch (Exception e) {
                promise.reject("CLAIM_ERROR", e.getMessage(), e);
            }
        });
    }

    // Private helper methods
    private String generateUniqueCode() {
        String timestamp = String.valueOf(System.currentTimeMillis());
        String userId = currentUserId != null ? currentUserId : "anonymous";
        String input = userId + timestamp + UUID.randomUUID().toString();
        
        try {
            MessageDigest md = MessageDigest.getInstance("SHA-256");
            byte[] hash = md.digest(input.getBytes());
            StringBuilder hexString = new StringBuilder();
            
            for (byte b : hash) {
                String hex = Integer.toHexString(0xff & b);
                if (hex.length() == 1) hexString.append('0');
                hexString.append(hex);
            }
            
            return "FIN" + hexString.toString().substring(0, 8).toUpperCase();
        } catch (NoSuchAlgorithmException e) {
            return "FIN" + UUID.randomUUID().toString().replace("-", "").substring(0, 8).toUpperCase();
        }
    }

    private String determineTier(double rpValue) {
        for (int i = RP_TIER_THRESHOLDS.length - 1; i >= 0; i--) {
            if (rpValue >= RP_TIER_THRESHOLDS[i]) {
                return RP_TIER_NAMES[i];
            }
        }
        return RP_TIER_NAMES[0];
    }

    private int getTierIndex(String tierName) {
        for (int i = 0; i < RP_TIER_NAMES.length; i++) {
            if (RP_TIER_NAMES[i].equals(tierName)) {
                return i;
            }
        }
        return 0;
    }

    private double getNextTierThreshold(int currentTierIndex) {
        if (currentTierIndex < RP_TIER_THRESHOLDS.length - 1) {
            return RP_TIER_THRESHOLDS[currentTierIndex + 1];
        }
        return RP_TIER_THRESHOLDS[RP_TIER_THRESHOLDS.length - 1];
    }

    private WritableMap parseNetworkTree(JSONObject networkTree) throws JSONException {
        WritableMap result = Arguments.createMap();
        if (networkTree != null) {
            result.putString("userId", networkTree.optString("userId", ""));
            result.putString("username", networkTree.optString("username", ""));
            result.putInt("level", networkTree.optInt("level", 0));
            result.putDouble("rpContribution", networkTree.optDouble("rpContribution", 0));
            result.putBoolean("isActive", networkTree.optBoolean("isActive", false));
            result.putDouble("joinDate", networkTree.optDouble("joinDate", 0));
            
            if (networkTree.has("children")) {
                WritableArray children = Arguments.createArray();
                // Add children parsing logic here
                result.putArray("children", children);
            }
        }
        return result;
    }

    private WritableArray parseLeaderboard(org.json.JSONArray leaderboardArray) throws JSONException {
        WritableArray result = Arguments.createArray();
        
        for (int i = 0; i < leaderboardArray.length(); i++) {
            JSONObject entry = leaderboardArray.getJSONObject(i);
            WritableMap leaderEntry = Arguments.createMap();
            
            leaderEntry.putInt("rank", entry.optInt("rank", 0));
            leaderEntry.putString("userId", entry.optString("userId", ""));
            leaderEntry.putString("username", entry.optString("username", ""));
            leaderEntry.putDouble("rpValue", entry.optDouble("rpValue", 0));
            leaderEntry.putString("tier", entry.optString("tier", ""));
            leaderEntry.putInt("networkSize", entry.optInt("networkSize", 0));
            leaderEntry.putBoolean("isCurrentUser", entry.optString("userId", "").equals(currentUserId));
            
            result.pushMap(leaderEntry);
        }
        
        return result;
    }

    private JSONObject convertReadableMapToJSONObject(ReadableMap readableMap) throws JSONException {
        JSONObject jsonObject = new JSONObject();
        ReadableMapKeySetIterator iterator = readableMap.keySetIterator();
        
        while (iterator.hasNextKey()) {
            String key = iterator.nextKey();
            switch (readableMap.getType(key)) {
                case Null:
                    jsonObject.put(key, JSONObject.NULL);
                    break;
                case Boolean:
                    jsonObject.put(key, readableMap.getBoolean(key));
                    break;
                case Number:
                    jsonObject.put(key, readableMap.getDouble(key));
                    break;
                case String:
                    jsonObject.put(key, readableMap.getString(key));
                    break;
                case Map:
                    jsonObject.put(key, convertReadableMapToJSONObject(readableMap.getMap(key)));
                    break;
                case Array:
                    // Handle arrays if needed
                    break;
            }
        }
        return jsonObject;
    }

    private void emitEvent(String eventName, WritableMap params) {
        getReactApplicationContext()
            .getJSModule(DeviceEventManagerModule.RCTDeviceEventEmitter.class)
            .emit(eventName, params);
    }

    @Override
    public void onCatalystInstanceDestroy() {
        super.onCatalystInstanceDestroy();
        if (executorService != null && !executorService.isShutdown()) {
            executorService.shutdown();
        }
    }
}
