package com.finova.reactnative;

import com.facebook.react.bridge.*;
import com.facebook.react.modules.core.DeviceEventManagerModule;
import android.content.Context;
import android.content.SharedPreferences;
import android.util.Log;
import org.json.JSONObject;
import java.util.concurrent.ExecutorService;
import java.util.concurrent.Executors;
import java.util.concurrent.ScheduledExecutorService;
import java.util.concurrent.TimeUnit;

/**
 * Mining Module - Handles mining operations and calculations
 */
@ReactModule(name = MiningModule.NAME)
public class MiningModule extends ReactContextBaseJavaModule {
    public static final String NAME = "FinovaMining";
    private static final String TAG = "MiningModule";
    private final ReactApplicationContext reactContext;
    private final SharedPreferences prefs;
    private final ExecutorService executor;
    private final ScheduledExecutorService scheduler;
    private boolean isMining = false;

    public MiningModule(ReactApplicationContext reactContext) {
        super(reactContext);
        this.reactContext = reactContext;
        this.prefs = reactContext.getSharedPreferences("finova_mining", Context.MODE_PRIVATE);
        this.executor = Executors.newFixedThreadPool(2);
        this.scheduler = Executors.newScheduledThreadPool(2);
    }

    @Override
    public String getName() {
        return NAME;
    }

    @ReactMethod
    public void startMining(Promise promise) {
        executor.execute(() -> {
            try {
                if (isMining) {
                    promise.resolve(createMiningResponse("Mining already active", true));
                    return;
                }

                // Start mining process
                isMining = true;
                long startTime = System.currentTimeMillis();
                
                SharedPreferences.Editor editor = prefs.edit();
                editor.putBoolean("is_mining", true);
                editor.putLong("mining_start_time", startTime);
                editor.apply();

                // Schedule periodic mining updates
                scheduler.scheduleAtFixedRate(this::processMiningUpdate, 0, 30, TimeUnit.SECONDS);

                WritableMap response = createMiningResponse("Mining started successfully", true);
                response.putLong("startTime", startTime);
                promise.resolve(response);

                // Emit mining started event
                sendMiningEvent("onMiningStarted", response);

            } catch (Exception e) {
                Log.e(TAG, "Failed to start mining", e);
                promise.reject("MINING_START_FAILED", e.getMessage());
            }
        });
    }

    @ReactMethod
    public void stopMining(Promise promise) {
        executor.execute(() -> {
            try {
                if (!isMining) {
                    promise.resolve(createMiningResponse("Mining not active", false));
                    return;
                }

                // Calculate final mining session
                long startTime = prefs.getLong("mining_start_time", 0);
                long endTime = System.currentTimeMillis();
                double sessionDuration = (endTime - startTime) / 1000.0 / 3600.0; // hours
                
                double sessionRewards = calculateSessionRewards(sessionDuration);
                updateTotalBalance(sessionRewards);

                // Stop mining
                isMining = false;
                SharedPreferences.Editor editor = prefs.edit();
                editor.putBoolean("is_mining", false);
                editor.putLong("mining_end_time", endTime);
                editor.putFloat("last_session_rewards", (float)sessionRewards);
                editor.apply();

                WritableMap response = createMiningResponse("Mining stopped successfully", false);
                response.putLong("endTime", endTime);
                response.putDouble("sessionRewards", sessionRewards);
                response.putDouble("sessionDuration", sessionDuration);
                promise.resolve(response);

                // Emit mining stopped event
                sendMiningEvent("onMiningStopped", response);

            } catch (Exception e) {
                Log.e(TAG, "Failed to stop mining", e);
                promise.reject("MINING_STOP_FAILED", e.getMessage());
            }
        });
    }

    @ReactMethod
    public void getMiningStatus(Promise promise) {
        executor.execute(() -> {
            try {
                WritableMap status = Arguments.createMap();
                status.putBoolean("isMining", isMining);
                
                if (isMining) {
                    long startTime = prefs.getLong("mining_start_time", 0);
                    long currentTime = System.currentTimeMillis();
                    double duration = (currentTime - startTime) / 1000.0 / 3600.0;
                    
                    status.putLong("startTime", startTime);
                    status.putDouble("duration", duration);
                    status.putDouble("currentRate", calculateCurrentMiningRate());
                    status.putDouble("estimatedRewards", calculateEstimatedRewards(duration));
                }

                status.putDouble("totalBalance", getTotalBalance());
                status.putDouble("todayEarnings", getTodayEarnings());
                status.putLong("totalMiningTime", getTotalMiningTime());
                
                promise.resolve(status);

            } catch (Exception e) {
                Log.e(TAG, "Failed to get mining status", e);
                promise.reject("STATUS_FETCH_FAILED", e.getMessage());
            }
        });
    }

    @ReactMethod
    public void calculateMiningRate(ReadableMap userParams, Promise promise) {
        executor.execute(() -> {
            try {
                int userLevel = userParams.hasKey("level") ? userParams.getInt("level") : 1;
                int totalUsers = userParams.hasKey("totalUsers") ? userParams.getInt("totalUsers") : 50000;
                int activeReferrals = userParams.hasKey("activeReferrals") ? userParams.getInt("activeReferrals") : 0;
                double totalHoldings = userParams.hasKey("totalHoldings") ? userParams.getDouble("totalHoldings") : 0;
                boolean kycVerified = userParams.hasKey("kycVerified") && userParams.getBoolean("kycVerified");

                // Base mining rate calculation (Pi Network inspired)
                double baseRate = getCurrentPhaseRate(totalUsers);
                double pioneerBonus = calculatePioneerBonus(totalUsers);
                double referralBonus = 1.0 + (activeReferrals * 0.1);
                double securityBonus = kycVerified ? 1.2 : 0.8;
                double regressionFactor = Math.exp(-0.001 * totalHoldings);
                double xpMultiplier = 1.0 + (userLevel * 0.02);

                double finalRate = baseRate * pioneerBonus * referralBonus * securityBonus * regressionFactor * xpMultiplier;

                WritableMap result = Arguments.createMap();
                result.putDouble("baseRate", baseRate);
                result.putDouble("pioneerBonus", pioneerBonus);
                result.putDouble("referralBonus", referralBonus);
                result.putDouble("securityBonus", securityBonus);
                result.putDouble("regressionFactor", regressionFactor);
                result.putDouble("xpMultiplier", xpMultiplier);
                result.putDouble("finalRate", finalRate);
                result.putDouble("dailyEstimate", finalRate * 24);

                promise.resolve(result);

            } catch (Exception e) {
                Log.e(TAG, "Failed to calculate mining rate", e);
                promise.reject("RATE_CALCULATION_FAILED", e.getMessage());
            }
        });
    }

    @ReactMethod
    public void getMiningHistory(int limit, Promise promise) {
        executor.execute(() -> {
            try {
                WritableArray history = Arguments.createArray();
                
                // Mock mining history - replace with actual data
                for (int i = 0; i < Math.min(limit, 30); i++) {
                    WritableMap session = Arguments.createMap();
                    long timestamp = System.currentTimeMillis() - (i * 24 * 60 * 60 * 1000L);
                    session.putLong("timestamp", timestamp);
                    session.putDouble("duration", 8.5 + (Math.random() * 4));
                    session.putDouble("rewards", 2.5 + (Math.random() * 3));
                    session.putDouble("rate", 0.3 + (Math.random() * 0.2));
                    session.putString("status", "completed");
                    history.pushMap(session);
                }

                promise.resolve(history);

            } catch (Exception e) {
                Log.e(TAG, "Failed to get mining history", e);
                promise.reject("HISTORY_FETCH_FAILED", e.getMessage());
            }
        });
    }

    @ReactMethod
    public void claimRewards(Promise promise) {
        executor.execute(() -> {
            try {
                if (!isMining) {
                    promise.reject("NOT_MINING", "Mining must be active to claim rewards");
                    return;
                }

                long startTime = prefs.getLong("mining_start_time", 0);
                double sessionDuration = (System.currentTimeMillis() - startTime) / 1000.0 / 3600.0;
                double claimableRewards = calculateSessionRewards(sessionDuration);

                if (claimableRewards < 0.001) {
                    promise.reject("INSUFFICIENT_REWARDS", "Minimum 0.001 FIN required to claim");
                    return;
                }

                // Update balance and reset session
                updateTotalBalance(claimableRewards);
                
                SharedPreferences.Editor editor = prefs.edit();
                editor.putLong("mining_start_time", System.currentTimeMillis());
                editor.putFloat("last_claim_amount", (float)claimableRewards);
                editor.apply();

                WritableMap response = createMiningResponse("Rewards claimed successfully", true);
                response.putDouble("claimedAmount", claimableRewards);
                response.putDouble("newBalance", getTotalBalance());
                promise.resolve(response);

                // Emit rewards claimed event
                sendMiningEvent("onRewardsClaimed", response);

            } catch (Exception e) {
                Log.e(TAG, "Failed to claim rewards", e);
                promise.reject("CLAIM_FAILED", e.getMessage());
            }
        });
    }

    // Helper methods
    private void processMiningUpdate() {
        if (!isMining) return;

        try {
            long startTime = prefs.getLong("mining_start_time", 0);
            double duration = (System.currentTimeMillis() - startTime) / 1000.0 / 3600.0;
            double currentRewards = calculateEstimatedRewards(duration);
            double currentRate = calculateCurrentMiningRate();

            WritableMap update = Arguments.createMap();
            update.putDouble("duration", duration);
            update.putDouble("currentRewards", currentRewards);
            update.putDouble("currentRate", currentRate);
            update.putLong("timestamp", System.currentTimeMillis());

            sendMiningEvent("onMiningUpdate", update);

        } catch (Exception e) {
            Log.e(TAG, "Mining update failed", e);
        }
    }

    private double getCurrentPhaseRate(int totalUsers) {
        if (totalUsers < 100000) return 0.1; // Phase 1
        if (totalUsers < 1000000) return 0.05; // Phase 2
        if (totalUsers < 10000000) return 0.025; // Phase 3
        return 0.01; // Phase 4
    }

    private double calculatePioneerBonus(int totalUsers) {
        return Math.max(1.0, 2.0 - (totalUsers / 1000000.0));
    }

    private double calculateCurrentMiningRate() {
        // Mock calculation - replace with actual rate calculation
        return 0.35 + (Math.random() * 0.1);
    }

    private double calculateSessionRewards(double hours) {
        double rate = calculateCurrentMiningRate();
        return rate * hours;
    }

    private double calculateEstimatedRewards(double hours) {
        return calculateSessionRewards(hours);
    }

    private void updateTotalBalance(double amount) {
        double currentBalance = getTotalBalance();
        SharedPreferences.Editor editor = prefs.edit();
        editor.putFloat("total_balance", (float)(currentBalance + amount));
        editor.apply();
    }

    private double getTotalBalance() {
        return prefs.getFloat("total_balance", 0.0f);
    }

    private double getTodayEarnings() {
        // Mock today's earnings
        return prefs.getFloat("today_earnings", 2.5f);
    }

    private long getTotalMiningTime() {
        return prefs.getLong("total_mining_time", 0);
    }

    private WritableMap createMiningResponse(String message, boolean mining) {
        WritableMap response = Arguments.createMap();
        response.putBoolean("success", true);
        response.putString("message", message);
        response.putBoolean("isMining", mining);
        response.putLong("timestamp", System.currentTimeMillis());
        return response;
    }

    private void sendMiningEvent(String eventName, WritableMap data) {
        if (reactContext.hasActiveCatalystInstance()) {
            reactContext
                .getJSModule(DeviceEventManagerModule.RCTDeviceEventEmitter.class)
                .emit(eventName, data);
        }
    }

    @Override
    public void onCatalystInstanceDestroy() {
        super.onCatalystInstanceDestroy();
        if (scheduler != null && !scheduler.isShutdown()) {
            scheduler.shutdown();
        }
        if (executor != null && !executor.isShutdown()) {
            executor.shutdown();
        }
    }
}
