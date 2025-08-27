package com.finova.reactnative;

import android.content.Context;
import android.net.ConnectivityManager;
import android.net.Network;
import android.net.NetworkCapabilities;
import android.net.NetworkRequest;
import android.os.Build;
import android.os.Handler;
import android.os.Looper;
import android.telephony.TelephonyManager;
import android.util.Log;

import androidx.annotation.NonNull;
import androidx.annotation.Nullable;

import com.facebook.react.bridge.Arguments;
import com.facebook.react.bridge.Callback;
import com.facebook.react.bridge.Promise;
import com.facebook.react.bridge.ReactApplicationContext;
import com.facebook.react.bridge.ReactContextBaseJavaModule;
import com.facebook.react.bridge.ReactMethod;
import com.facebook.react.bridge.WritableMap;
import com.facebook.react.modules.core.DeviceEventManagerModule;

import java.util.HashMap;
import java.util.Map;
import java.util.concurrent.ExecutorService;
import java.util.concurrent.Executors;

/**
 * Finova Network Status Module for React Native Android
 * Monitors network connectivity and quality for optimal mining performance
 * Integrates with Finova's anti-bot detection and fair distribution systems
 * 
 * @author Finova Network Team
 * @version 1.0.0
 * @since 2025-07-26
 */
public class NetworkStatusModule extends ReactContextBaseJavaModule {
    
    private static final String MODULE_NAME = "FinovaNetworkStatus";
    private static final String TAG = "FinovaNetworkStatus";
    
    // Event names for React Native
    private static final String EVENT_NETWORK_CHANGED = "finovaNetworkChanged";
    private static final String EVENT_CONNECTION_QUALITY = "finovaConnectionQuality";
    private static final String EVENT_MINING_NETWORK_STATUS = "finovaMiningNetworkStatus";
    
    // Network quality thresholds for Finova mining optimization
    private static final int EXCELLENT_QUALITY_THRESHOLD = 90;
    private static final int GOOD_QUALITY_THRESHOLD = 70;
    private static final int FAIR_QUALITY_THRESHOLD = 50;
    private static final int POOR_QUALITY_THRESHOLD = 30;
    
    // Mining network requirements
    private static final long MIN_BANDWIDTH_FOR_MINING = 1_000_000; // 1 Mbps in bps
    private static final int MIN_SIGNAL_STRENGTH = -85; // dBm
    private static final long MAX_LATENCY_FOR_MINING = 500; // milliseconds
    
    private final ReactApplicationContext reactContext;
    private final ConnectivityManager connectivityManager;
    private final TelephonyManager telephonyManager;
    private final ExecutorService networkExecutor;
    private final Handler mainHandler;
    
    private ConnectivityManager.NetworkCallback networkCallback;
    private boolean isMonitoring = false;
    private NetworkInfo currentNetworkInfo;
    
    // Network quality metrics for anti-bot detection
    private long lastNetworkChangeTime = 0;
    private int networkChangeCount = 0;
    private double averageLatency = 0.0;
    private long totalBandwidthTests = 0;
    
    public NetworkStatusModule(ReactApplicationContext reactContext) {
        super(reactContext);
        this.reactContext = reactContext;
        this.connectivityManager = (ConnectivityManager) reactContext.getSystemService(Context.CONNECTIVITY_SERVICE);
        this.telephonyManager = (TelephonyManager) reactContext.getSystemService(Context.TELEPHONY_SERVICE);
        this.networkExecutor = Executors.newFixedThreadPool(2);
        this.mainHandler = new Handler(Looper.getMainLooper());
        this.currentNetworkInfo = new NetworkInfo();
        
        initializeNetworkCallback();
        Log.d(TAG, "Finova NetworkStatusModule initialized");
    }
    
    @NonNull
    @Override
    public String getName() {
        return MODULE_NAME;
    }
    
    @Override
    public Map<String, Object> getConstants() {
        final Map<String, Object> constants = new HashMap<>();
        
        // Network types for Finova mining optimization
        constants.put("NETWORK_TYPE_UNKNOWN", "unknown");
        constants.put("NETWORK_TYPE_WIFI", "wifi");
        constants.put("NETWORK_TYPE_CELLULAR", "cellular");
        constants.put("NETWORK_TYPE_ETHERNET", "ethernet");
        constants.put("NETWORK_TYPE_VPN", "vpn");
        
        // Connection states
        constants.put("CONNECTION_STATE_CONNECTED", "connected");
        constants.put("CONNECTION_STATE_CONNECTING", "connecting");
        constants.put("CONNECTION_STATE_DISCONNECTED", "disconnected");
        constants.put("CONNECTION_STATE_BLOCKED", "blocked");
        
        // Quality levels for mining performance
        constants.put("QUALITY_EXCELLENT", "excellent");
        constants.put("QUALITY_GOOD", "good");
        constants.put("QUALITY_FAIR", "fair");
        constants.put("QUALITY_POOR", "poor");
        constants.put("QUALITY_UNUSABLE", "unusable");
        
        // Mining network status
        constants.put("MINING_NETWORK_OPTIMAL", "optimal");
        constants.put("MINING_NETWORK_REDUCED", "reduced");
        constants.put("MINING_NETWORK_LIMITED", "limited");
        constants.put("MINING_NETWORK_DISABLED", "disabled");
        
        return constants;
    }
    
    /**
     * Start monitoring network status for Finova mining operations
     */
    @ReactMethod
    public void startNetworkMonitoring(Promise promise) {
        try {
            if (!isMonitoring) {
                if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.N) {
                    connectivityManager.registerDefaultNetworkCallback(networkCallback);
                } else {
                    NetworkRequest.Builder builder = new NetworkRequest.Builder();
                    connectivityManager.registerNetworkCallback(builder.build(), networkCallback);
                }
                
                isMonitoring = true;
                Log.d(TAG, "Finova network monitoring started");
                
                // Initial network status check
                updateCurrentNetworkInfo();
                promise.resolve(getCurrentNetworkStatus());
            } else {
                promise.resolve(getCurrentNetworkStatus());
            }
        } catch (Exception e) {
            Log.e(TAG, "Error starting network monitoring", e);
            promise.reject("NETWORK_MONITORING_ERROR", "Failed to start network monitoring: " + e.getMessage());
        }
    }
    
    /**
     * Stop network monitoring
     */
    @ReactMethod
    public void stopNetworkMonitoring(Promise promise) {
        try {
            if (isMonitoring) {
                connectivityManager.unregisterNetworkCallback(networkCallback);
                isMonitoring = false;
                Log.d(TAG, "Finova network monitoring stopped");
            }
            promise.resolve(null);
        } catch (Exception e) {
            Log.e(TAG, "Error stopping network monitoring", e);
            promise.reject("NETWORK_MONITORING_ERROR", "Failed to stop network monitoring: " + e.getMessage());
        }
    }
    
    /**
     * Get current network status optimized for Finova mining
     */
    @ReactMethod
    public void getCurrentNetworkStatus(Promise promise) {
        networkExecutor.execute(() -> {
            try {
                updateCurrentNetworkInfo();
                WritableMap networkStatus = getCurrentNetworkStatus();
                promise.resolve(networkStatus);
            } catch (Exception e) {
                Log.e(TAG, "Error getting network status", e);
                promise.reject("NETWORK_STATUS_ERROR", "Failed to get network status: " + e.getMessage());
            }
        });
    }
    
    /**
     * Test network quality for mining optimization
     */
    @ReactMethod
    public void testNetworkQuality(Promise promise) {
        networkExecutor.execute(() -> {
            try {
                NetworkQualityResult result = performNetworkQualityTest();
                WritableMap qualityMap = Arguments.createMap();
                
                qualityMap.putDouble("latency", result.latency);
                qualityMap.putDouble("downloadSpeed", result.downloadSpeed);
                qualityMap.putDouble("uploadSpeed", result.uploadSpeed);
                qualityMap.putInt("qualityScore", result.qualityScore);
                qualityMap.putString("qualityLevel", result.qualityLevel);
                qualityMap.putString("miningStatus", result.miningStatus);
                qualityMap.putBoolean("isOptimalForMining", result.isOptimalForMining);
                qualityMap.putDouble("miningEfficiencyFactor", result.miningEfficiencyFactor);
                
                // Update average latency for anti-bot detection
                updateLatencyMetrics(result.latency);
                
                promise.resolve(qualityMap);
            } catch (Exception e) {
                Log.e(TAG, "Error testing network quality", e);
                promise.reject("NETWORK_QUALITY_ERROR", "Failed to test network quality: " + e.getMessage());
            }
        });
    }
    
    /**
     * Get network metrics for anti-bot detection
     */
    @ReactMethod
    public void getNetworkMetrics(Promise promise) {
        try {
            WritableMap metrics = Arguments.createMap();
            metrics.putInt("networkChangeCount", networkChangeCount);
            metrics.putDouble("averageLatency", averageLatency);
            metrics.putDouble("lastNetworkChangeTime", lastNetworkChangeTime);
            metrics.putDouble("totalBandwidthTests", totalBandwidthTests);
            metrics.putBoolean("hasStableConnection", hasStableConnection());
            metrics.putDouble("connectionStabilityScore", calculateConnectionStabilityScore());
            
            promise.resolve(metrics);
        } catch (Exception e) {
            Log.e(TAG, "Error getting network metrics", e);
            promise.reject("NETWORK_METRICS_ERROR", "Failed to get network metrics: " + e.getMessage());
        }
    }
    
    /**
     * Check if current network supports Finova mining
     */
    @ReactMethod
    public void isMiningNetworkReady(Promise promise) {
        networkExecutor.execute(() -> {
            try {
                boolean isReady = checkMiningNetworkReadiness();
                WritableMap result = Arguments.createMap();
                result.putBoolean("isReady", isReady);
                result.putString("reason", getMiningReadinessReason(isReady));
                result.putDouble("estimatedMiningEfficiency", calculateMiningEfficiency());
                
                promise.resolve(result);
            } catch (Exception e) {
                Log.e(TAG, "Error checking mining network readiness", e);
                promise.reject("MINING_NETWORK_ERROR", "Failed to check mining network readiness: " + e.getMessage());
            }
        });
    }
    
    private void initializeNetworkCallback() {
        networkCallback = new ConnectivityManager.NetworkCallback() {
            @Override
            public void onAvailable(@NonNull Network network) {
                Log.d(TAG, "Network available: " + network);
                handleNetworkChange("available", network);
            }
            
            @Override
            public void onLost(@NonNull Network network) {
                Log.d(TAG, "Network lost: " + network);
                handleNetworkChange("lost", network);
            }
            
            @Override
            public void onCapabilitiesChanged(@NonNull Network network, @NonNull NetworkCapabilities networkCapabilities) {
                Log.d(TAG, "Network capabilities changed: " + network);
                handleNetworkCapabilitiesChange(network, networkCapabilities);
            }
            
            @Override
            public void onBlockedStatusChanged(@NonNull Network network, boolean blocked) {
                Log.d(TAG, "Network blocked status changed: " + network + ", blocked: " + blocked);
                handleNetworkBlockedStatusChange(network, blocked);
            }
        };
    }
    
    private void handleNetworkChange(String changeType, Network network) {
        mainHandler.post(() -> {
            try {
                networkChangeCount++;
                lastNetworkChangeTime = System.currentTimeMillis();
                
                updateCurrentNetworkInfo();
                
                WritableMap eventData = Arguments.createMap();
                eventData.putString("changeType", changeType);
                eventData.putString("networkId", network.toString());
                eventData.putMap("networkInfo", getCurrentNetworkStatus());
                eventData.putDouble("timestamp", lastNetworkChangeTime);
                
                sendEvent(EVENT_NETWORK_CHANGED, eventData);
                
                // Check mining network status
                checkAndNotifyMiningNetworkStatus();
                
            } catch (Exception e) {
                Log.e(TAG, "Error handling network change", e);
            }
        });
    }
    
    private void handleNetworkCapabilitiesChange(Network network, NetworkCapabilities capabilities) {
        mainHandler.post(() -> {
            try {
                updateCurrentNetworkInfo();
                
                WritableMap eventData = Arguments.createMap();
                eventData.putString("networkId", network.toString());
                eventData.putMap("capabilities", parseNetworkCapabilities(capabilities));
                eventData.putDouble("timestamp", System.currentTimeMillis());
                
                sendEvent(EVENT_CONNECTION_QUALITY, eventData);
                
            } catch (Exception e) {
                Log.e(TAG, "Error handling capabilities change", e);
            }
        });
    }
    
    private void handleNetworkBlockedStatusChange(Network network, boolean blocked) {
        mainHandler.post(() -> {
            try {
                WritableMap eventData = Arguments.createMap();
                eventData.putString("networkId", network.toString());
                eventData.putBoolean("blocked", blocked);
                eventData.putDouble("timestamp", System.currentTimeMillis());
                
                sendEvent(EVENT_NETWORK_CHANGED, eventData);
                
                if (blocked) {
                    // Notify that mining might be affected
                    notifyMiningNetworkIssue("Network blocked - mining temporarily suspended");
                }
                
            } catch (Exception e) {
                Log.e(TAG, "Error handling blocked status change", e);
            }
        });
    }
    
    private void updateCurrentNetworkInfo() {
        try {
            Network activeNetwork = connectivityManager.getActiveNetwork();
            if (activeNetwork != null) {
                NetworkCapabilities capabilities = connectivityManager.getNetworkCapabilities(activeNetwork);
                if (capabilities != null) {
                    currentNetworkInfo.isConnected = true;
                    currentNetworkInfo.networkType = determineNetworkType(capabilities);
                    currentNetworkInfo.isMetered = !capabilities.hasCapability(NetworkCapabilities.NET_CAPABILITY_NOT_METERED);
                    currentNetworkInfo.isRoaming = !capabilities.hasCapability(NetworkCapabilities.NET_CAPABILITY_NOT_ROAMING);
                    currentNetworkInfo.linkDownstreamBandwidth = capabilities.getLinkDownstreamBandwidthKbps();
                    currentNetworkInfo.linkUpstreamBandwidth = capabilities.getLinkUpstreamBandwidthKbps();
                    currentNetworkInfo.signalStrength = getSignalStrength();
                } else {
                    currentNetworkInfo.isConnected = false;
                }
            } else {
                currentNetworkInfo.isConnected = false;
            }
        } catch (Exception e) {
            Log.e(TAG, "Error updating network info", e);
            currentNetworkInfo.isConnected = false;
        }
    }
    
    private WritableMap getCurrentNetworkStatus() {
        WritableMap status = Arguments.createMap();
        
        status.putBoolean("isConnected", currentNetworkInfo.isConnected);
        status.putString("networkType", currentNetworkInfo.networkType);
        status.putBoolean("isMetered", currentNetworkInfo.isMetered);
        status.putBoolean("isRoaming", currentNetworkInfo.isRoaming);
        status.putInt("linkDownstreamBandwidth", currentNetworkInfo.linkDownstreamBandwidth);
        status.putInt("linkUpstreamBandwidth", currentNetworkInfo.linkUpstreamBandwidth);
        status.putInt("signalStrength", currentNetworkInfo.signalStrength);
        status.putString("connectionState", currentNetworkInfo.isConnected ? "connected" : "disconnected");
        status.putDouble("timestamp", System.currentTimeMillis());
        
        // Add Finova-specific mining optimization data
        status.putBoolean("isOptimalForMining", isOptimalForMining());
        status.putDouble("miningEfficiencyFactor", calculateMiningEfficiency());
        status.putString("miningNetworkStatus", getMiningNetworkStatus());
        
        return status;
    }
    
    private NetworkQualityResult performNetworkQualityTest() {
        NetworkQualityResult result = new NetworkQualityResult();
        
        try {
            // Simulate network quality test (in real implementation, use actual network testing)
            long startTime = System.currentTimeMillis();
            
            // Test latency to Finova servers
            result.latency = testLatency();
            
            // Test download speed
            result.downloadSpeed = testDownloadSpeed();
            
            // Test upload speed  
            result.uploadSpeed = testUploadSpeed();
            
            // Calculate quality score based on all metrics
            result.qualityScore = calculateQualityScore(result.latency, result.downloadSpeed, result.uploadSpeed);
            result.qualityLevel = determineQualityLevel(result.qualityScore);
            result.miningStatus = determineMiningStatus(result.qualityScore, result.latency);
            result.isOptimalForMining = result.qualityScore >= GOOD_QUALITY_THRESHOLD && result.latency <= MAX_LATENCY_FOR_MINING;
            result.miningEfficiencyFactor = calculateMiningEfficiencyFromQuality(result.qualityScore);
            
            totalBandwidthTests++;
            
        } catch (Exception e) {
            Log.e(TAG, "Error performing network quality test", e);
            result.qualityScore = 0;
            result.qualityLevel = "unusable";
            result.miningStatus = "disabled";
        }
        
        return result;
    }
    
    private double testLatency() {
        // Simulate latency test to Finova network
        // In real implementation, ping Finova servers
        return Math.random() * 200 + 50; // 50-250ms simulated latency
    }
    
    private double testDownloadSpeed() {
        // Simulate download speed test
        // In real implementation, download test file from Finova CDN
        return currentNetworkInfo.linkDownstreamBandwidth * 1000 * 0.8; // Convert kbps to bps with efficiency factor
    }
    
    private double testUploadSpeed() {
        // Simulate upload speed test
        // In real implementation, upload test data to Finova servers
        return currentNetworkInfo.linkUpstreamBandwidth * 1000 * 0.7; // Convert kbps to bps with efficiency factor
    }
    
    private int calculateQualityScore(double latency, double downloadSpeed, double uploadSpeed) {
        int latencyScore = Math.max(0, 100 - (int)(latency / 5)); // Lower latency = higher score
        int downloadScore = Math.min(100, (int)(downloadSpeed / MIN_BANDWIDTH_FOR_MINING * 50));
        int uploadScore = Math.min(100, (int)(uploadSpeed / (MIN_BANDWIDTH_FOR_MINING * 0.5) * 30));
        
        return (latencyScore + downloadScore + uploadScore) / 3;
    }
    
    private String determineQualityLevel(int qualityScore) {
        if (qualityScore >= EXCELLENT_QUALITY_THRESHOLD) return "excellent";
        if (qualityScore >= GOOD_QUALITY_THRESHOLD) return "good";
        if (qualityScore >= FAIR_QUALITY_THRESHOLD) return "fair";
        if (qualityScore >= POOR_QUALITY_THRESHOLD) return "poor";
        return "unusable";
    }
    
    private String determineMiningStatus(int qualityScore, double latency) {
        if (qualityScore >= GOOD_QUALITY_THRESHOLD && latency <= MAX_LATENCY_FOR_MINING) {
            return "optimal";
        } else if (qualityScore >= FAIR_QUALITY_THRESHOLD && latency <= MAX_LATENCY_FOR_MINING * 1.5) {
            return "reduced";
        } else if (qualityScore >= POOR_QUALITY_THRESHOLD) {
            return "limited";
        } else {
            return "disabled";
        }
    }
    
    private double calculateMiningEfficiencyFromQuality(int qualityScore) {
        return Math.max(0.1, Math.min(1.0, qualityScore / 100.0));
    }
    
    private String determineNetworkType(NetworkCapabilities capabilities) {
        if (capabilities.hasTransport(NetworkCapabilities.TRANSPORT_WIFI)) {
            return "wifi";
        } else if (capabilities.hasTransport(NetworkCapabilities.TRANSPORT_CELLULAR)) {
            return "cellular";
        } else if (capabilities.hasTransport(NetworkCapabilities.TRANSPORT_ETHERNET)) {
            return "ethernet";
        } else if (capabilities.hasTransport(NetworkCapabilities.TRANSPORT_VPN)) {
            return "vpn";
        } else {
            return "unknown";
        }
    }
    
    private WritableMap parseNetworkCapabilities(NetworkCapabilities capabilities) {
        WritableMap caps = Arguments.createMap();
        
        caps.putBoolean("hasInternet", capabilities.hasCapability(NetworkCapabilities.NET_CAPABILITY_INTERNET));
        caps.putBoolean("isValidated", capabilities.hasCapability(NetworkCapabilities.NET_CAPABILITY_VALIDATED));
        caps.putBoolean("isNotMetered", capabilities.hasCapability(NetworkCapabilities.NET_CAPABILITY_NOT_METERED));
        caps.putBoolean("isNotRoaming", capabilities.hasCapability(NetworkCapabilities.NET_CAPABILITY_NOT_ROAMING));
        caps.putInt("linkDownstreamBandwidth", capabilities.getLinkDownstreamBandwidthKbps());
        caps.putInt("linkUpstreamBandwidth", capabilities.getLinkUpstreamBandwidthKbps());
        
        return caps;
    }
    
    private int getSignalStrength() {
        // For cellular networks, get signal strength
        // This is simplified - real implementation would use telephony APIs
        return currentNetworkInfo.networkType.equals("cellular") ? -75 : -30; // dBm
    }
    
    private boolean isOptimalForMining() {
        return currentNetworkInfo.isConnected &&
               !currentNetworkInfo.isRoaming &&
               currentNetworkInfo.linkDownstreamBandwidth >= (MIN_BANDWIDTH_FOR_MINING / 1000) &&
               currentNetworkInfo.signalStrength >= MIN_SIGNAL_STRENGTH;
    }
    
    private double calculateMiningEfficiency() {
        if (!currentNetworkInfo.isConnected) return 0.0;
        
        double efficiency = 0.5; // Base efficiency
        
        // Network type bonus
        switch (currentNetworkInfo.networkType) {
            case "wifi":
                efficiency += 0.3;
                break;
            case "ethernet":
                efficiency += 0.4;
                break;
            case "cellular":
                efficiency += currentNetworkInfo.isRoaming ? 0.1 : 0.2;
                break;
        }
        
        // Bandwidth factor
        double bandwidthFactor = Math.min(1.0, currentNetworkInfo.linkDownstreamBandwidth * 1000.0 / MIN_BANDWIDTH_FOR_MINING);
        efficiency *= bandwidthFactor;
        
        // Signal strength factor (for cellular)
        if (currentNetworkInfo.networkType.equals("cellular")) {
            double signalFactor = Math.max(0.2, 1.0 + (currentNetworkInfo.signalStrength + 85) / 50.0);
            efficiency *= signalFactor;
        }
        
        return Math.max(0.0, Math.min(1.0, efficiency));
    }
    
    private String getMiningNetworkStatus() {
        double efficiency = calculateMiningEfficiency();
        
        if (efficiency >= 0.8) return "optimal";
        if (efficiency >= 0.6) return "reduced";
        if (efficiency >= 0.3) return "limited";
        return "disabled";
    }
    
    private boolean checkMiningNetworkReadiness() {
        return isOptimalForMining() && hasStableConnection();
    }
    
    private String getMiningReadinessReason(boolean isReady) {
        if (isReady) return "Network is optimal for mining";
        
        if (!currentNetworkInfo.isConnected) return "No network connection";
        if (currentNetworkInfo.isRoaming) return "Roaming network not recommended for mining";
        if (currentNetworkInfo.linkDownstreamBandwidth < (MIN_BANDWIDTH_FOR_MINING / 1000)) return "Insufficient bandwidth for mining";
        if (currentNetworkInfo.signalStrength < MIN_SIGNAL_STRENGTH) return "Poor signal strength";
        if (!hasStableConnection()) return "Network connection is unstable";
        
        return "Network conditions not optimal for mining";
    }
    
    private boolean hasStableConnection() {
        long currentTime = System.currentTimeMillis();
        long timeSinceLastChange = currentTime - lastNetworkChangeTime;
        
        // Consider connection stable if no changes in last 30 seconds and less than 10 changes per hour
        return timeSinceLastChange > 30000 && networkChangeCount < 10;
    }
    
    private double calculateConnectionStabilityScore() {
        long currentTime = System.currentTimeMillis();
        long timeSinceLastChange = currentTime - lastNetworkChangeTime;
        
        double timeStability = Math.min(1.0, timeSinceLastChange / 300000.0); // Normalize to 5 minutes
        double changeStability = Math.max(0.0, 1.0 - (networkChangeCount / 20.0)); // Penalize frequent changes
        double latencyStability = averageLatency > 0 ? Math.max(0.0, 1.0 - (averageLatency / 1000.0)) : 0.5;
        
        return (timeStability + changeStability + latencyStability) / 3.0;
    }
    
    private void updateLatencyMetrics(double latency) {
        if (averageLatency == 0.0) {
            averageLatency = latency;
        } else {
            averageLatency = (averageLatency * 0.8) + (latency * 0.2); // Exponential moving average
        }
    }
    
    private void checkAndNotifyMiningNetworkStatus() {
        String status = getMiningNetworkStatus();
        double efficiency = calculateMiningEfficiency();
        
        WritableMap eventData = Arguments.createMap();
        eventData.putString("status", status);
        eventData.putDouble("efficiency", efficiency);
        eventData.putBoolean("isOptimal", isOptimalForMining());
        eventData.putString("recommendation", getMiningNetworkRecommendation(status));
        eventData.putDouble("timestamp", System.currentTimeMillis());
        
        sendEvent(EVENT_MINING_NETWORK_STATUS, eventData);
    }
    
    private String getMiningNetworkRecommendation(String status) {
        switch (status) {
            case "optimal":
                return "Perfect conditions for maximum mining rewards";
            case "reduced":
                return "Good mining conditions with slightly reduced efficiency";
            case "limited":
                return "Mining possible but efficiency is limited";
            case "disabled":
                return "Network conditions not suitable for mining";
            default:
                return "Check network connection for mining";
        }
    }
    
    private void notifyMiningNetworkIssue(String message) {
        WritableMap eventData = Arguments.createMap();
        eventData.putString("type", "issue");
        eventData.putString("message", message);
        eventData.putString("severity", "warning");
        eventData.putDouble("timestamp", System.currentTimeMillis());
        
        sendEvent(EVENT_MINING_NETWORK_STATUS, eventData);
    }
    
    private void sendEvent(String eventName, @Nullable WritableMap params) {
        reactContext
            .getJSModule(DeviceEventManagerModule.RCTDeviceEventEmitter.class)
            .emit(eventName, params);
    }
    
    @Override
    public void onCatalystInstanceDestroy() {
        super.onCatalystInstanceDestroy();
        
        try {
            if (isMonitoring) {
                connectivityManager.unregisterNetworkCallback(networkCallback);
            }
            
            if (networkExecutor != null && !networkExecutor.isShutdown()) {
                networkExecutor.shutdown();
            }
            
            Log.d(TAG, "Finova NetworkStatusModule destroyed");
        } catch (Exception e) {
            Log.e(TAG, "Error destroying NetworkStatusModule", e);
        }
    }
    
    // Inner classes for data structures
    private static class NetworkInfo {
        boolean isConnected = false;
        String networkType = "unknown";
        boolean isMetered = false;
        boolean isRoaming = false;
        int linkDownstreamBandwidth = 0;
        int linkUpstreamBandwidth = 0;
        int signalStrength = -100;
    }
    
    private static class NetworkQualityResult {
        double latency = 0.0;
        double downloadSpeed = 0.0;
        double uploadSpeed = 0.0;
        int qualityScore = 0;
        String qualityLevel = "unknown";
        String miningStatus = "unknown";
        boolean isOptimalForMining = false;
        double miningEfficiencyFactor = 0.0;
    }
}
