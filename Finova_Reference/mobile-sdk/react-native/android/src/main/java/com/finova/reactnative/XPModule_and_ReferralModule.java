package com.finova.reactnative;

import com.facebook.react.bridge.*;
import com.facebook.react.modules.core.DeviceEventManagerModule;
import android.content.SharedPreferences;
import android.content.Context;
import android.util.Log;
import java.util.concurrent.ExecutorService;
import java.util.concurrent.Executors;

/**
 * XP Module - Experience Points system
 */
@ReactModule(name = XPModule.NAME)
public class XPModule extends ReactContextBaseJavaModule {
    public static final String NAME = "FinovaXP";
    private static final String TAG = "XPModule";
    private final ReactApplicationContext reactContext;
    private final SharedPreferences prefs;
    private final ExecutorService executor;

    public XPModule(ReactApplicationContext reactContext) {
        super(reactContext);
        this.reactContext = reactContext;
        this.prefs = reactContext.getSharedPreferences("finova_xp", Context.MODE_PRIVATE);
        this.executor = Executors.newFixedThreadPool(2);
    }

    @Override
    public String getName() {
        return NAME;
    }

    @ReactMethod
    public void addXP(ReadableMap activity, Promise promise) {
        executor.execute(() -> {
            try {
                String activityType = activity.getString("type");
                String platform = activity.hasKey("platform") ? activity.getString("platform") : "app";
                double qualityScore = activity.hasKey("qualityScore") ? activity.getDouble("qualityScore") : 1.0;
                int currentLevel = getCurrentLevel();
                int streakDays = getStreakDays();

                int baseXP = getBaseXP(activityType);
                double platformMultiplier = getPlatformMultiplier(platform);
                double streakBonus = 1.0 + Math.min(streakDays * 0.05, 2.0);
                double levelProgression = Math.exp(-0.01 * currentLevel);

                int earnedXP = (int)(baseXP * platformMultiplier * qualityScore * streakBonus * levelProgression);
                
                int totalXP = getTotalXP() + earnedXP;
                int newLevel = calculateLevel(totalXP);
                
                SharedPreferences.Editor editor = prefs.edit();
                editor.putInt("total_xp", totalXP);
                editor.putInt("current_level", newLevel);
                editor.putLong("last_activity", System.currentTimeMillis());
                editor.apply();

                WritableMap response = Arguments.createMap();
                response.putInt("earnedXP", earnedXP);
                response.putInt("totalXP", totalXP);
                response.putInt("currentLevel", newLevel);
                response.putInt("previousLevel", currentLevel);
                response.putBoolean("levelUp", newLevel > currentLevel);

                if (newLevel > currentLevel) {
                    sendEvent("onLevelUp", response);
                }

                promise.resolve(response);
                sendEvent("onXPEarned", response);

            } catch (Exception e) {
                Log.e(TAG, "Failed to add XP", e);
                promise.reject("XP_ADD_FAILED", e.getMessage());
            }
        });
    }

    @ReactMethod
    public void getXPStatus(Promise promise) {
        try {
            int totalXP = getTotalXP();
            int currentLevel = getCurrentLevel();
            int xpForNextLevel = getXPForNextLevel(currentLevel);
            int xpProgress = totalXP - getXPForLevel(currentLevel);
            String tier = getLevelTier(currentLevel);
            
            WritableMap status = Arguments.createMap();
            status.putInt("totalXP", totalXP);
            status.putInt("currentLevel", currentLevel);
            status.putInt("xpForNextLevel", xpForNextLevel);
            status.putInt("xpProgress", xpProgress);
            status.putString("tier", tier);
            status.putInt("streakDays", getStreakDays());
            status.putDouble("miningMultiplier", getMiningMultiplier(currentLevel));
            
            promise.resolve(status);
        } catch (Exception e) {
            promise.reject("XP_STATUS_FAILED", e.getMessage());
        }
    }

    private int getBaseXP(String activityType) {
        switch (activityType) {
            case "post": return 50;
            case "photo": return 75;
            case "video": return 150;
            case "comment": return 25;
            case "like": return 5;
            case "share": return 15;
            case "follow": return 20;
            case "login": return 10;
            case "quest": return 100;
            case "viral": return 1000;
            default: return 10;
        }
    }

    private double getPlatformMultiplier(String platform) {
        switch (platform.toLowerCase()) {
            case "tiktok": return 1.3;
            case "youtube": return 1.4;
            case "instagram": return 1.2;
            case "facebook": return 1.1;
            case "twitter":
            case "x": return 1.2;
            default: return 1.0;
        }
    }

    private int getTotalXP() {
        return prefs.getInt("total_xp", 0);
    }

    private int getCurrentLevel() {
        return prefs.getInt("current_level", 1);
    }

    private int getStreakDays() {
        return prefs.getInt("streak_days", 0);
    }

    private int calculateLevel(int totalXP) {
        return Math.min(100, (int)(Math.sqrt(totalXP / 10.0) + 1));
    }

    private int getXPForLevel(int level) {
        return (level - 1) * (level - 1) * 10;
    }

    private int getXPForNextLevel(int currentLevel) {
        return getXPForLevel(currentLevel + 1) - getXPForLevel(currentLevel);
    }

    private String getLevelTier(int level) {
        if (level <= 10) return "Bronze";
        if (level <= 25) return "Silver";
        if (level <= 50) return "Gold";
        if (level <= 75) return "Platinum";
        if (level <= 100) return "Diamond";
        return "Mythic";
    }

    private double getMiningMultiplier(int level) {
        return 1.0 + (level * 0.02);
    }

    private void sendEvent(String eventName, WritableMap data) {
        if (reactContext.hasActiveCatalystInstance()) {
            reactContext.getJSModule(DeviceEventManagerModule.RCTDeviceEventEmitter.class)
                .emit(eventName, data);
        }
    }
}

/**
 * Referral Module - Referral Points and network management
 */
@ReactModule(name = ReferralModule.NAME)
public class ReferralModule extends ReactContextBaseJavaModule {
    public static final String NAME = "FinovaReferral";
    private static final String TAG = "ReferralModule";
    private final ReactApplicationContext reactContext;
    private final SharedPreferences prefs;
    private final ExecutorService executor;

    public ReferralModule(ReactApplicationContext reactContext) {
        super(reactContext);
        this.reactContext = reactContext;
        this.prefs = reactContext.getSharedPreferences("finova_referral", Context.MODE_PRIVATE);
        this.executor = Executors.newFixedThreadPool(2);
    }

    @Override
    public String getName() {
        return NAME;
    }

    @ReactMethod
    public void generateReferralCode(Promise promise) {
        executor.execute(() -> {
            try {
                String userId = prefs.getString("user_id", "");
                if (userId.isEmpty()) {
                    promise.reject("NOT_AUTHENTICATED", "User not authenticated");
                    return;
                }

                String referralCode = "FIN" + userId.substring(0, 4).toUpperCase() + 
                    String.valueOf(System.currentTimeMillis()).substring(8);
                
                SharedPreferences.Editor editor = prefs.edit();
                editor.putString("referral_code", referralCode);
                editor.apply();

                WritableMap response = Arguments.createMap();
                response.putString("referralCode", referralCode);
                response.putString("shareUrl", "https://finova.network/join/" + referralCode);
                response.putLong("generatedAt", System.currentTimeMillis());
                
                promise.resolve(response);

            } catch (Exception e) {
                Log.e(TAG, "Failed to generate referral code", e);
                promise.reject("CODE_GENERATION_FAILED", e.getMessage());
            }
        });
    }

    @ReactMethod
    public void useReferralCode(String code, Promise promise) {
        executor.execute(() -> {
            try {
                if (code == null || code.length() < 6) {
                    promise.reject("INVALID_CODE", "Invalid referral code format");
                    return;
                }

                // Mock validation - replace with actual API call
                boolean isValid = validateReferralCode(code);
                if (!isValid) {
                    promise.reject("INVALID_CODE", "Referral code not found or expired");
                    return;
                }

                SharedPreferences.Editor editor = prefs.edit();
                editor.putString("used_referral_code", code);
                editor.putLong("referral_used_at", System.currentTimeMillis());
                editor.apply();

                // Award referral bonuses
                int bonusXP = 100;
                int bonusRP = 50;

                WritableMap response = Arguments.createMap();
                response.putBoolean("success", true);
                response.putString("referralCode", code);
                response.putInt("bonusXP", bonusXP);
                response.putInt("bonusRP", bonusRP);
                response.putString("message", "Referral code applied successfully!");

                promise.resolve(response);
                sendEvent("onReferralCodeUsed", response);

            } catch (Exception e) {
                Log.e(TAG, "Failed to use referral code", e);
                promise.reject("CODE_USE_FAILED", e.getMessage());
            }
        });
    }

    @ReactMethod
    public void getReferralStats(Promise promise) {
        executor.execute(() -> {
            try {
                WritableMap stats = Arguments.createMap();
                stats.putInt("totalRP", getTotalRP());
                stats.putInt("directReferrals", getDirectReferrals());
                stats.putInt("activeReferrals", getActiveReferrals());
                stats.putInt("networkSize", getNetworkSize());
                stats.putString("tier", getRPTier());
                stats.putDouble("networkQuality", getNetworkQuality());
                stats.putDouble("miningBonus", getMiningBonus());
                stats.putString("referralCode", getReferralCode());

                WritableArray referralHistory = Arguments.createArray();
                // Mock referral history
                for (int i = 0; i < 5; i++) {
                    WritableMap referral = Arguments.createMap();
                    referral.putString("userId", "user_" + i);
                    referral.putLong("joinedAt", System.currentTimeMillis() - (i * 24 * 60 * 60 * 1000L));
                    referral.putBoolean("active", i < 3);
                    referral.putInt("level", 10 + i * 5);
                    referralHistory.pushMap(referral);
                }
                stats.putArray("recentReferrals", referralHistory);

                promise.resolve(stats);

            } catch (Exception e) {
                Log.e(TAG, "Failed to get referral stats", e);
                promise.reject("STATS_FETCH_FAILED", e.getMessage());
            }
        });
    }

    @ReactMethod
    public void calculateRPValue(ReadableMap networkData, Promise promise) {
        executor.execute(() -> {
            try {
                int directReferrals = networkData.getInt("directReferrals");
                int l2Network = networkData.hasKey("l2Network") ? networkData.getInt("l2Network") : 0;
                int l3Network = networkData.hasKey("l3Network") ? networkData.getInt("l3Network") : 0;
                double networkQuality = networkData.hasKey("networkQuality") ? 
                    networkData.getDouble("networkQuality") : 0.5;

                // RP calculation based on whitepaper formula
                double directRP = directReferrals * 100 * networkQuality;
                double indirectRP = (l2Network * 50 * 0.3) + (l3Network * 25 * 0.1);
                double qualityBonus = networkQuality * 15 * 0.85;
                double totalRP = (directRP + indirectRP) * qualityBonus;

                // Apply regression factor
                int totalNetworkSize = directReferrals + l2Network + l3Network;
                double regressionFactor = Math.exp(-0.0001 * totalNetworkSize * networkQuality);
                double finalRP = totalRP * regressionFactor;

                WritableMap result = Arguments.createMap();
                result.putDouble("directRP", directRP);
                result.putDouble("indirectRP", indirectRP);
                result.putDouble("qualityBonus", qualityBonus);
                result.putDouble("totalRP", totalRP);
                result.putDouble("regressionFactor", regressionFactor);
                result.putDouble("finalRP", finalRP);
                result.putString("tier", calculateRPTier(finalRP));

                promise.resolve(result);

            } catch (Exception e) {
                Log.e(TAG, "Failed to calculate RP value", e);
                promise.reject("RP_CALCULATION_FAILED", e.getMessage());
            }
        });
    }

    private boolean validateReferralCode(String code) {
        // Mock validation - implement actual API validation
        return code.startsWith("FIN") && code.length() >= 8;
    }

    private int getTotalRP() {
        return prefs.getInt("total_rp", 0);
    }

    private int getDirectReferrals() {
        return prefs.getInt("direct_referrals", 0);
    }

    private int getActiveReferrals() {
        return prefs.getInt("active_referrals", 0);
    }

    private int getNetworkSize() {
        return prefs.getInt("network_size", 0);
    }

    private String getRPTier() {
        int totalRP = getTotalRP();
        if (totalRP < 1000) return "Explorer";
        if (totalRP < 5000) return "Connector";
        if (totalRP < 15000) return "Influencer";
        if (totalRP < 50000) return "Leader";
        return "Ambassador";
    }

    private String calculateRPTier(double rp) {
        if (rp < 1000) return "Explorer";
        if (rp < 5000) return "Connector";
        if (rp < 15000) return "Influencer";
        if (rp < 50000) return "Leader";
        return "Ambassador";
    }

    private double getNetworkQuality() {
        int active = getActiveReferrals();
        int total = getDirectReferrals();
        return total > 0 ? (double)active / total : 0.0;
    }

    private double getMiningBonus() {
        String tier = getRPTier();
        switch (tier) {
            case "Connector": return 0.2;
            case "Influencer": return 0.5;
            case "Leader": return 1.0;
            case "Ambassador": return 2.0;
            default: return 0.0;
        }
    }

    private String getReferralCode() {
        return prefs.getString("referral_code", "");
    }

    private void sendEvent(String eventName, WritableMap data) {
        if (reactContext.hasActiveCatalystInstance()) {
            reactContext.getJSModule(DeviceEventManagerModule.RCTDeviceEventEmitter.class)
                .emit(eventName, data);
        }
    }
}
