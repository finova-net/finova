package com.finova.reactnative;

import android.content.Context;
import android.content.pm.PackageInfo;
import android.content.pm.PackageManager;
import android.os.Build;
import android.provider.Settings;
import android.telephony.TelephonyManager;
import android.net.wifi.WifiManager;
import android.net.wifi.WifiInfo;
import android.hardware.fingerprint.FingerprintManager;
import android.security.keystore.KeyGenParameterSpec;
import android.security.keystore.KeyProperties;
import android.util.Base64;
import androidx.biometric.BiometricManager;
import androidx.core.content.ContextCompat;
import java.security.MessageDigest;
import java.security.SecureRandom;
import java.util.HashMap;
import java.util.Map;
import java.util.Locale;
import java.util.TimeZone;
import javax.crypto.Cipher;
import javax.crypto.KeyGenerator;
import javax.crypto.SecretKey;
import javax.crypto.spec.IvParameterSpec;
import java.security.KeyStore;
import java.nio.charset.StandardCharsets;

import com.facebook.react.bridge.ReactApplicationContext;
import com.facebook.react.bridge.ReactContextBaseJavaModule;
import com.facebook.react.bridge.ReactMethod;
import com.facebook.react.bridge.Promise;
import com.facebook.react.bridge.WritableMap;
import com.facebook.react.bridge.WritableNativeMap;

public class DeviceInfoModule extends ReactContextBaseJavaModule {
    
    private static final String MODULE_NAME = "FinovaDeviceInfo";
    private static final String KEYSTORE_ALIAS = "FinovaDeviceKey";
    private static final String SHARED_PREFS_NAME = "finova_device_prefs";
    private final ReactApplicationContext reactContext;
    
    public DeviceInfoModule(ReactApplicationContext reactContext) {
        super(reactContext);
        this.reactContext = reactContext;
    }
    
    @Override
    public String getName() {
        return MODULE_NAME;
    }
    
    @ReactMethod
    public void getDeviceInfo(Promise promise) {
        try {
            WritableMap deviceInfo = new WritableNativeMap();
            
            // Basic device information
            deviceInfo.putString("deviceId", getSecureDeviceId());
            deviceInfo.putString("brand", Build.BRAND);
            deviceInfo.putString("manufacturer", Build.MANUFACTURER);
            deviceInfo.putString("model", Build.MODEL);
            deviceInfo.putString("product", Build.PRODUCT);
            deviceInfo.putString("device", Build.DEVICE);
            deviceInfo.putString("board", Build.BOARD);
            deviceInfo.putString("hardware", Build.HARDWARE);
            deviceInfo.putString("serial", Build.SERIAL);
            deviceInfo.putString("androidId", getAndroidId());
            
            // System information
            deviceInfo.putString("osVersion", Build.VERSION.RELEASE);
            deviceInfo.putInt("sdkVersion", Build.VERSION.SDK_INT);
            deviceInfo.putString("buildId", Build.ID);
            deviceInfo.putString("incremental", Build.VERSION.INCREMENTAL);
            deviceInfo.putString("codename", Build.VERSION.CODENAME);
            deviceInfo.putString("fingerprint", Build.FINGERPRINT);
            
            // Application information
            PackageInfo packageInfo = getPackageInfo();
            if (packageInfo != null) {
                deviceInfo.putString("appVersion", packageInfo.versionName);
                deviceInfo.putInt("appBuildNumber", packageInfo.versionCode);
                deviceInfo.putString("packageName", packageInfo.packageName);
                deviceInfo.putDouble("firstInstallTime", packageInfo.firstInstallTime);
                deviceInfo.putDouble("lastUpdateTime", packageInfo.lastUpdateTime);
            }
            
            // Network information
            deviceInfo.putString("networkOperator", getNetworkOperator());
            deviceInfo.putString("networkOperatorName", getNetworkOperatorName());
            deviceInfo.putString("wifiMacAddress", getWifiMacAddress());
            deviceInfo.putString("ipAddress", getIpAddress());
            
            // Security features
            deviceInfo.putBoolean("hasFingerprint", hasFingerprintSupport());
            deviceInfo.putBoolean("hasBiometric", hasBiometricSupport());
            deviceInfo.putBoolean("isRooted", isDeviceRooted());
            deviceInfo.putBoolean("isEmulator", isEmulator());
            deviceInfo.putBoolean("hasSecurityPatch", hasSecurityPatch());
            
            // Locale and timezone
            deviceInfo.putString("locale", getCurrentLocale());
            deviceInfo.putString("timezone", getTimezone());
            deviceInfo.putInt("timezoneOffset", getTimezoneOffset());
            
            // Hardware specifications
            deviceInfo.putString("cpuAbi", getCpuAbi());
            deviceInfo.putLong("totalMemory", getTotalMemory());
            deviceInfo.putLong("availableMemory", getAvailableMemory());
            deviceInfo.putLong("totalStorage", getTotalStorage());
            deviceInfo.putLong("availableStorage", getAvailableStorage());
            
            // Anti-bot detection metrics
            deviceInfo.putString("deviceFingerprint", generateDeviceFingerprint());
            deviceInfo.putDouble("bootTime", getBootTime());
            deviceInfo.putBoolean("isDeveloperModeEnabled", isDeveloperModeEnabled());
            deviceInfo.putBoolean("isAdbEnabled", isAdbEnabled());
            deviceInfo.putBoolean("isDebuggable", isDebuggable());
            
            // Finova-specific identifiers
            deviceInfo.putString("finovaDeviceId", generateFinovaDeviceId());
            deviceInfo.putString("securityToken", generateSecurityToken());
            deviceInfo.putDouble("timestamp", System.currentTimeMillis());
            
            promise.resolve(deviceInfo);
            
        } catch (Exception e) {
            promise.reject("DEVICE_INFO_ERROR", "Failed to get device info: " + e.getMessage(), e);
        }
    }
    
    @ReactMethod
    public void validateDeviceIntegrity(Promise promise) {
        try {
            WritableMap integrity = new WritableNativeMap();
            
            // Security validations
            integrity.putBoolean("isSecureDevice", isSecureDevice());
            integrity.putBoolean("hasValidFingerprint", hasValidDeviceFingerprint());
            integrity.putBoolean("isGenuineApp", isGenuineApp());
            integrity.putDouble("riskScore", calculateRiskScore());
            integrity.putString("integrityLevel", getIntegrityLevel());
            
            // Behavioral analysis
            integrity.putMap("behaviorMetrics", getBehaviorMetrics());
            integrity.putMap("usagePatterns", getUsagePatterns());
            
            promise.resolve(integrity);
            
        } catch (Exception e) {
            promise.reject("INTEGRITY_CHECK_ERROR", "Failed to validate device integrity: " + e.getMessage(), e);
        }
    }
    
    @ReactMethod
    public void generateBiometricChallenge(Promise promise) {
        try {
            if (!hasBiometricSupport()) {
                promise.reject("BIOMETRIC_NOT_SUPPORTED", "Biometric authentication not supported");
                return;
            }
            
            String challenge = generateSecureChallenge();
            WritableMap result = new WritableNativeMap();
            result.putString("challenge", challenge);
            result.putDouble("expiresAt", System.currentTimeMillis() + 300000); // 5 minutes
            result.putString("type", "biometric_challenge");
            
            promise.resolve(result);
            
        } catch (Exception e) {
            promise.reject("BIOMETRIC_CHALLENGE_ERROR", "Failed to generate biometric challenge: " + e.getMessage(), e);
        }
    }
    
    private String getSecureDeviceId() {
        try {
            String androidId = Settings.Secure.getString(
                reactContext.getContentResolver(), 
                Settings.Secure.ANDROID_ID
            );
            
            if (androidId == null || "9774d56d682e549c".equals(androidId)) {
                // Fallback for devices without valid Android ID
                return generateFallbackDeviceId();
            }
            
            return hashString(androidId + Build.SERIAL + Build.FINGERPRINT);
            
        } catch (Exception e) {
            return generateFallbackDeviceId();
        }
    }
    
    private String getAndroidId() {
        try {
            return Settings.Secure.getString(
                reactContext.getContentResolver(), 
                Settings.Secure.ANDROID_ID
            );
        } catch (Exception e) {
            return "unknown";
        }
    }
    
    private PackageInfo getPackageInfo() {
        try {
            PackageManager packageManager = reactContext.getPackageManager();
            return packageManager.getPackageInfo(reactContext.getPackageName(), 0);
        } catch (Exception e) {
            return null;
        }
    }
    
    private String getNetworkOperator() {
        try {
            TelephonyManager telephonyManager = (TelephonyManager) reactContext
                .getSystemService(Context.TELEPHONY_SERVICE);
            return telephonyManager.getNetworkOperator();
        } catch (Exception e) {
            return "unknown";
        }
    }
    
    private String getNetworkOperatorName() {
        try {
            TelephonyManager telephonyManager = (TelephonyManager) reactContext
                .getSystemService(Context.TELEPHONY_SERVICE);
            return telephonyManager.getNetworkOperatorName();
        } catch (Exception e) {
            return "unknown";
        }
    }
    
    private String getWifiMacAddress() {
        try {
            WifiManager wifiManager = (WifiManager) reactContext.getApplicationContext()
                .getSystemService(Context.WIFI_SERVICE);
            WifiInfo wifiInfo = wifiManager.getConnectionInfo();
            return wifiInfo.getMacAddress();
        } catch (Exception e) {
            return "unknown";
        }
    }
    
    private String getIpAddress() {
        try {
            WifiManager wifiManager = (WifiManager) reactContext.getApplicationContext()
                .getSystemService(Context.WIFI_SERVICE);
            WifiInfo wifiInfo = wifiManager.getConnectionInfo();
            int ipAddress = wifiInfo.getIpAddress();
            return String.format(Locale.getDefault(), "%d.%d.%d.%d",
                (ipAddress & 0xff), (ipAddress >> 8 & 0xff),
                (ipAddress >> 16 & 0xff), (ipAddress >> 24 & 0xff));
        } catch (Exception e) {
            return "0.0.0.0";
        }
    }
    
    private boolean hasFingerprintSupport() {
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.M) {
            try {
                FingerprintManager fingerprintManager = (FingerprintManager) reactContext
                    .getSystemService(Context.FINGERPRINT_SERVICE);
                return fingerprintManager != null && fingerprintManager.isHardwareDetected();
            } catch (Exception e) {
                return false;
            }
        }
        return false;
    }
    
    private boolean hasBiometricSupport() {
        try {
            BiometricManager biometricManager = BiometricManager.from(reactContext);
            int result = biometricManager.canAuthenticate(BiometricManager.Authenticators.BIOMETRIC_WEAK);
            return result == BiometricManager.BIOMETRIC_SUCCESS;
        } catch (Exception e) {
            return false;
        }
    }
    
    private boolean isDeviceRooted() {
        return checkRootMethod1() || checkRootMethod2() || checkRootMethod3();
    }
    
    private boolean checkRootMethod1() {
        String[] paths = {
            "/system/app/Superuser.apk", "/sbin/su", "/system/bin/su",
            "/system/xbin/su", "/data/local/xbin/su", "/data/local/bin/su",
            "/system/sd/xbin/su", "/system/bin/failsafe/su", "/data/local/su",
            "/su/bin/su"
        };
        
        for (String path : paths) {
            if (new java.io.File(path).exists()) return true;
        }
        return false;
    }
    
    private boolean checkRootMethod2() {
        try {
            Process process = Runtime.getRuntime().exec("which su");
            java.io.BufferedReader in = new java.io.BufferedReader(
                new java.io.InputStreamReader(process.getInputStream())
            );
            return in.readLine() != null;
        } catch (Exception e) {
            return false;
        }
    }
    
    private boolean checkRootMethod3() {
        String buildTags = Build.TAGS;
        return buildTags != null && buildTags.contains("test-keys");
    }
    
    private boolean isEmulator() {
        return (Build.FINGERPRINT.startsWith("generic") ||
                Build.FINGERPRINT.startsWith("unknown") ||
                Build.MODEL.contains("google_sdk") ||
                Build.MODEL.contains("Emulator") ||
                Build.MODEL.contains("Android SDK built for x86") ||
                Build.MANUFACTURER.contains("Genymotion") ||
                (Build.BRAND.startsWith("generic") && Build.DEVICE.startsWith("generic")) ||
                "google_sdk".equals(Build.PRODUCT));
    }
    
    private boolean hasSecurityPatch() {
        return Build.VERSION.SDK_INT >= Build.VERSION_CODES.M &&
               Build.VERSION.SECURITY_PATCH != null &&
               !Build.VERSION.SECURITY_PATCH.isEmpty();
    }
    
    private String getCurrentLocale() {
        return Locale.getDefault().toString();
    }
    
    private String getTimezone() {
        return TimeZone.getDefault().getID();
    }
    
    private int getTimezoneOffset() {
        return TimeZone.getDefault().getOffset(System.currentTimeMillis()) / 1000;
    }
    
    private String getCpuAbi() {
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.LOLLIPOP) {
            return Build.SUPPORTED_ABIS[0];
        } else {
            return Build.CPU_ABI;
        }
    }
    
    private long getTotalMemory() {
        try {
            android.app.ActivityManager actManager = (android.app.ActivityManager) reactContext
                .getSystemService(Context.ACTIVITY_SERVICE);
            android.app.ActivityManager.MemoryInfo memInfo = new android.app.ActivityManager.MemoryInfo();
            actManager.getMemoryInfo(memInfo);
            return memInfo.totalMem;
        } catch (Exception e) {
            return -1;
        }
    }
    
    private long getAvailableMemory() {
        try {
            android.app.ActivityManager actManager = (android.app.ActivityManager) reactContext
                .getSystemService(Context.ACTIVITY_SERVICE);
            android.app.ActivityManager.MemoryInfo memInfo = new android.app.ActivityManager.MemoryInfo();
            actManager.getMemoryInfo(memInfo);
            return memInfo.availMem;
        } catch (Exception e) {
            return -1;
        }
    }
    
    private long getTotalStorage() {
        try {
            android.os.StatFs stat = new android.os.StatFs(
                android.os.Environment.getDataDirectory().getPath()
            );
            return stat.getBlockSizeLong() * stat.getBlockCountLong();
        } catch (Exception e) {
            return -1;
        }
    }
    
    private long getAvailableStorage() {
        try {
            android.os.StatFs stat = new android.os.StatFs(
                android.os.Environment.getDataDirectory().getPath()
            );
            return stat.getBlockSizeLong() * stat.getAvailableBlocksLong();
        } catch (Exception e) {
            return -1;
        }
    }
    
    private String generateDeviceFingerprint() {
        try {
            StringBuilder fingerprint = new StringBuilder();
            fingerprint.append(Build.BRAND).append("|");
            fingerprint.append(Build.MODEL).append("|");
            fingerprint.append(Build.DEVICE).append("|");
            fingerprint.append(Build.PRODUCT).append("|");
            fingerprint.append(Build.HARDWARE).append("|");
            fingerprint.append(Build.MANUFACTURER).append("|");
            fingerprint.append(Build.VERSION.RELEASE).append("|");
            fingerprint.append(Build.VERSION.SDK_INT).append("|");
            fingerprint.append(getAndroidId()).append("|");
            fingerprint.append(getCurrentLocale()).append("|");
            fingerprint.append(getTimezone());
            
            return hashString(fingerprint.toString());
        } catch (Exception e) {
            return "unknown";
        }
    }
    
    private double getBootTime() {
        return System.currentTimeMillis() - android.os.SystemClock.elapsedRealtime();
    }
    
    private boolean isDeveloperModeEnabled() {
        try {
            return Settings.Global.getInt(reactContext.getContentResolver(),
                Settings.Global.DEVELOPMENT_SETTINGS_ENABLED, 0) != 0;
        } catch (Exception e) {
            return false;
        }
    }
    
    private boolean isAdbEnabled() {
        try {
            return Settings.Global.getInt(reactContext.getContentResolver(),
                Settings.Global.ADB_ENABLED, 0) != 0;
        } catch (Exception e) {
            return false;
        }
    }
    
    private boolean isDebuggable() {
        return (reactContext.getApplicationInfo().flags & 
                android.content.pm.ApplicationInfo.FLAG_DEBUGGABLE) != 0;
    }
    
    private String generateFinovaDeviceId() {
        try {
            String baseId = getSecureDeviceId();
            String finovaPrefix = "FIN_";
            String timestamp = String.valueOf(System.currentTimeMillis() / 1000);
            return finovaPrefix + hashString(baseId + timestamp + "FINOVA_SALT").substring(0, 16);
        } catch (Exception e) {
            return "FIN_UNKNOWN_" + System.currentTimeMillis();
        }
    }
    
    private String generateSecurityToken() {
        try {
            SecureRandom random = new SecureRandom();
            byte[] token = new byte[32];
            random.nextBytes(token);
            return Base64.encodeToString(token, Base64.NO_WRAP);
        } catch (Exception e) {
            return "unknown_token";
        }
    }
    
    private boolean isSecureDevice() {
        return !isDeviceRooted() && 
               !isEmulator() && 
               !isDeveloperModeEnabled() && 
               !isAdbEnabled() &&
               hasSecurityPatch();
    }
    
    private boolean hasValidDeviceFingerprint() {
        try {
            String currentFingerprint = generateDeviceFingerprint();
            String storedFingerprint = getStoredFingerprint();
            
            if (storedFingerprint == null) {
                storeFingerprint(currentFingerprint);
                return true;
            }
            
            return currentFingerprint.equals(storedFingerprint);
        } catch (Exception e) {
            return false;
        }
    }
    
    private boolean isGenuineApp() {
        try {
            PackageInfo packageInfo = getPackageInfo();
            if (packageInfo == null) return false;
            
            // Check if app is signed with expected signature
            android.content.pm.Signature[] signatures = packageInfo.signatures;
            if (signatures == null || signatures.length == 0) return false;
            
            // In production, verify against known app signature hash
            String signatureHash = hashString(signatures[0].toCharsString());
            return signatureHash.length() > 0; // Simplified check
            
        } catch (Exception e) {
            return false;
        }
    }
    
    private double calculateRiskScore() {
        double riskScore = 0.0;
        
        if (isDeviceRooted()) riskScore += 0.3;
        if (isEmulator()) riskScore += 0.4;
        if (isDeveloperModeEnabled()) riskScore += 0.1;
        if (isAdbEnabled()) riskScore += 0.1;
        if (!hasSecurityPatch()) riskScore += 0.05;
        if (!hasValidDeviceFingerprint()) riskScore += 0.2;
        
        return Math.min(1.0, riskScore);
    }
    
    private String getIntegrityLevel() {
        double riskScore = calculateRiskScore();
        
        if (riskScore < 0.1) return "HIGH";
        else if (riskScore < 0.3) return "MEDIUM";
        else if (riskScore < 0.6) return "LOW";
        else return "CRITICAL";
    }
    
    private WritableMap getBehaviorMetrics() {
        WritableMap metrics = new WritableNativeMap();
        
        // Placeholder for behavioral analysis
        metrics.putDouble("sessionDuration", getAverageSessionDuration());
        metrics.putDouble("interactionRate", getInteractionRate());
        metrics.putDouble("responseTime", getAverageResponseTime());
        metrics.putInt("dailyUsage", getDailyUsageCount());
        
        return metrics;
    }
    
    private WritableMap getUsagePatterns() {
        WritableMap patterns = new WritableNativeMap();
        
        // Placeholder for usage pattern analysis
        patterns.putString("mostActiveHour", getMostActiveHour());
        patterns.putString("usageFrequency", getUsageFrequency());
        patterns.putDouble("consistencyScore", getConsistencyScore());
        
        return patterns;
    }
    
    private String generateSecureChallenge() {
        try {
            SecureRandom random = new SecureRandom();
            byte[] challenge = new byte[32];
            random.nextBytes(challenge);
            
            long timestamp = System.currentTimeMillis();
            String challengeData = Base64.encodeToString(challenge, Base64.NO_WRAP) + 
                                 "|" + timestamp + "|" + getSecureDeviceId();
            
            return Base64.encodeToString(challengeData.getBytes(StandardCharsets.UTF_8), 
                                       Base64.NO_WRAP);
        } catch (Exception e) {
            return "challenge_error";
        }
    }
    
    private String generateFallbackDeviceId() {
        try {
            StringBuilder fallback = new StringBuilder();
            fallback.append(Build.BOARD).append("|");
            fallback.append(Build.BRAND).append("|");
            fallback.append(Build.DEVICE).append("|");
            fallback.append(Build.HARDWARE).append("|");
            fallback.append(Build.MANUFACTURER).append("|");
            fallback.append(Build.MODEL).append("|");
            fallback.append(Build.PRODUCT);
            
            return hashString(fallback.toString() + System.currentTimeMillis());
        } catch (Exception e) {
            return "fallback_id_" + System.currentTimeMillis();
        }
    }
    
    private String hashString(String input) {
        try {
            MessageDigest md = MessageDigest.getInstance("SHA-256");
            byte[] hash = md.digest(input.getBytes(StandardCharsets.UTF_8));
            StringBuilder hexString = new StringBuilder();
            
            for (byte b : hash) {
                String hex = Integer.toHexString(0xff & b);
                if (hex.length() == 1) hexString.append('0');
                hexString.append(hex);
            }
            
            return hexString.toString();
        } catch (Exception e) {
            return "hash_error";
        }
    }
    
    private String getStoredFingerprint() {
        try {
            android.content.SharedPreferences prefs = reactContext
                .getSharedPreferences(SHARED_PREFS_NAME, Context.MODE_PRIVATE);
            return prefs.getString("device_fingerprint", null);
        } catch (Exception e) {
            return null;
        }
    }
    
    private void storeFingerprint(String fingerprint) {
        try {
            android.content.SharedPreferences prefs = reactContext
                .getSharedPreferences(SHARED_PREFS_NAME, Context.MODE_PRIVATE);
            prefs.edit().putString("device_fingerprint", fingerprint).apply();
        } catch (Exception e) {
            // Ignore storage errors
        }
    }
    
    // Placeholder methods for behavioral metrics
    private double getAverageSessionDuration() { return 300.0; } // 5 minutes
    private double getInteractionRate() { return 0.75; }
    private double getAverageResponseTime() { return 250.0; } // milliseconds
    private int getDailyUsageCount() { return 12; }
    private String getMostActiveHour() { return "14:00"; }
    private String getUsageFrequency() { return "regular"; }
    private double getConsistencyScore() { return 0.85; }
}
