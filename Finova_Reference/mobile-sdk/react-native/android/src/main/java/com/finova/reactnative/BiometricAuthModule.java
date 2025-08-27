package com.finova.reactnative;

import android.app.Activity;
import android.content.Context;
import android.hardware.biometrics.BiometricManager;
import android.hardware.biometrics.BiometricPrompt;
import android.os.Build;
import android.os.CancellationSignal;
import android.util.Base64;
import android.util.Log;

import androidx.annotation.NonNull;
import androidx.annotation.RequiresApi;
import androidx.biometric.BiometricConstants;
import androidx.fragment.app.FragmentActivity;

import com.facebook.react.bridge.Arguments;
import com.facebook.react.bridge.Callback;
import com.facebook.react.bridge.Promise;
import com.facebook.react.bridge.ReactApplicationContext;
import com.facebook.react.bridge.ReactContextBaseJavaModule;
import com.facebook.react.bridge.ReactMethod;
import com.facebook.react.bridge.ReadableMap;
import com.facebook.react.bridge.WritableMap;
import com.facebook.react.modules.core.DeviceEventManagerModule;

import java.security.KeyPair;
import java.security.KeyPairGenerator;
import java.security.PrivateKey;
import java.security.PublicKey;
import java.security.Signature;
import java.security.spec.ECGenParameterSpec;
import java.util.concurrent.Executor;
import java.util.concurrent.Executors;

import javax.crypto.Cipher;
import javax.crypto.KeyGenerator;
import javax.crypto.SecretKey;
import javax.crypto.spec.GCMParameterSpec;

/**
 * Finova Network Biometric Authentication Module
 * Handles KYC verification, security bonuses, and anti-bot measures
 * Enterprise-grade implementation with comprehensive security features
 */
public class BiometricAuthModule extends ReactContextBaseJavaModule {
    
    private static final String MODULE_NAME = "FinovaBiometricAuth";
    private static final String TAG = "FinovaBiometric";
    private static final String KEYSTORE_ALIAS = "finova_biometric_key";
    private static final String PREFERENCE_NAME = "finova_biometric_prefs";
    
    // Error codes matching Finova Network specifications
    private static final String ERROR_NOT_AVAILABLE = "BIOMETRIC_NOT_AVAILABLE";
    private static final String ERROR_NOT_ENROLLED = "BIOMETRIC_NOT_ENROLLED";
    private static final String ERROR_USER_CANCEL = "USER_CANCEL";
    private static final String ERROR_AUTHENTICATION_FAILED = "AUTH_FAILED";
    private static final String ERROR_LOCKOUT = "BIOMETRIC_LOCKOUT";
    private static final String ERROR_SECURITY_VIOLATION = "SECURITY_VIOLATION";
    private static final String ERROR_INVALID_CONTEXT = "INVALID_CONTEXT";
    
    // Finova specific constants
    private static final String EVENT_BIOMETRIC_SUCCESS = "onBiometricAuthSuccess";
    private static final String EVENT_BIOMETRIC_ERROR = "onBiometricAuthError";
    private static final String EVENT_KYC_VERIFIED = "onKYCVerified";
    private static final String EVENT_SECURITY_BONUS_UPDATED = "onSecurityBonusUpdated";
    
    private final ReactApplicationContext reactContext;
    private final Executor executor;
    private androidx.biometric.BiometricPrompt biometricPrompt;
    private CancellationSignal cancellationSignal;
    private FinovaSecurityManager securityManager;
    private FinovaKYCManager kycManager;

    public BiometricAuthModule(ReactApplicationContext reactContext) {
        super(reactContext);
        this.reactContext = reactContext;
        this.executor = Executors.newSingleThreadExecutor();
        this.securityManager = new FinovaSecurityManager(reactContext);
        this.kycManager = new FinovaKYCManager(reactContext);
        
        Log.d(TAG, "Finova BiometricAuthModule initialized");
    }

    @Override
    @NonNull
    public String getName() {
        return MODULE_NAME;
    }

    /**
     * Check if biometric authentication is available and configured
     * Returns comprehensive status for Finova KYC requirements
     */
    @ReactMethod
    public void isBiometricAvailable(Promise promise) {
        try {
            Activity currentActivity = getCurrentActivity();
            if (currentActivity == null) {
                promise.reject(ERROR_INVALID_CONTEXT, "No current activity found");
                return;
            }

            androidx.biometric.BiometricManager biometricManager = 
                androidx.biometric.BiometricManager.from(currentActivity);
            
            WritableMap result = Arguments.createMap();
            
            switch (biometricManager.canAuthenticate(androidx.biometric.BiometricManager.Authenticators.BIOMETRIC_STRONG)) {
                case androidx.biometric.BiometricManager.BIOMETRIC_SUCCESS:
                    result.putBoolean("isAvailable", true);
                    result.putString("status", "AVAILABLE");
                    result.putBoolean("kycEligible", true);
                    result.putDouble("securityBonusMultiplier", 1.2); // 20% bonus as per whitepaper
                    break;
                    
                case androidx.biometric.BiometricManager.BIOMETRIC_ERROR_NO_HARDWARE:
                    result.putBoolean("isAvailable", false);
                    result.putString("status", "NO_HARDWARE");
                    result.putBoolean("kycEligible", false);
                    result.putDouble("securityBonusMultiplier", 0.8); // Penalty as per whitepaper
                    break;
                    
                case androidx.biometric.BiometricManager.BIOMETRIC_ERROR_HW_UNAVAILABLE:
                    result.putBoolean("isAvailable", false);
                    result.putString("status", "HW_UNAVAILABLE");
                    result.putBoolean("kycEligible", false);
                    result.putDouble("securityBonusMultiplier", 0.8);
                    break;
                    
                case androidx.biometric.BiometricManager.BIOMETRIC_ERROR_NONE_ENROLLED:
                    result.putBoolean("isAvailable", false);
                    result.putString("status", "NONE_ENROLLED");
                    result.putBoolean("kycEligible", false);
                    result.putDouble("securityBonusMultiplier", 0.8);
                    break;
                    
                default:
                    result.putBoolean("isAvailable", false);
                    result.putString("status", "UNKNOWN");
                    result.putBoolean("kycEligible", false);
                    result.putDouble("securityBonusMultiplier", 0.8);
                    break;
            }
            
            // Add device security assessment
            result.putMap("deviceSecurity", securityManager.assessDeviceSecurity());
            
            promise.resolve(result);
            
        } catch (Exception e) {
            Log.e(TAG, "Error checking biometric availability", e);
            promise.reject(ERROR_NOT_AVAILABLE, "Failed to check biometric availability: " + e.getMessage());
        }
    }

    /**
     * Authenticate user with biometric for KYC verification
     * Implements Finova's Proof-of-Humanity system
     */
    @ReactMethod
    public void authenticateForKYC(ReadableMap options, Promise promise) {
        try {
            Activity currentActivity = getCurrentActivity();
            if (!(currentActivity instanceof FragmentActivity)) {
                promise.reject(ERROR_INVALID_CONTEXT, "Activity must be FragmentActivity for biometric authentication");
                return;
            }

            String title = options.hasKey("title") ? options.getString("title") : "Finova KYC Verification";
            String subtitle = options.hasKey("subtitle") ? options.getString("subtitle") : "Verify your identity to earn security bonus";
            String description = options.hasKey("description") ? options.getString("description") : "Place your finger on the sensor to complete KYC verification";
            String userId = options.hasKey("userId") ? options.getString("userId") : "";
            
            if (userId.isEmpty()) {
                promise.reject(ERROR_INVALID_CONTEXT, "User ID is required for KYC verification");
                return;
            }

            FragmentActivity fragmentActivity = (FragmentActivity) currentActivity;
            
            // Create biometric prompt info
            androidx.biometric.BiometricPrompt.PromptInfo promptInfo = 
                new androidx.biometric.BiometricPrompt.PromptInfo.Builder()
                    .setTitle(title)
                    .setSubtitle(subtitle)
                    .setDescription(description)
                    .setNegativeButtonText("Cancel")
                    .setAllowedAuthenticators(androidx.biometric.BiometricManager.Authenticators.BIOMETRIC_STRONG)
                    .build();

            // Create authentication callback with Finova-specific handling
            androidx.biometric.BiometricPrompt.AuthenticationCallback authCallback = 
                new androidx.biometric.BiometricPrompt.AuthenticationCallback() {
                    
                @Override
                public void onAuthenticationSucceeded(@NonNull androidx.biometric.BiometricPrompt.AuthenticationResult result) {
                    super.onAuthenticationSucceeded(result);
                    
                    try {
                        // Process KYC verification
                        processKYCVerification(userId, result, promise);
                        
                    } catch (Exception e) {
                        Log.e(TAG, "Error processing KYC verification", e);
                        promise.reject(ERROR_AUTHENTICATION_FAILED, "KYC processing failed: " + e.getMessage());
                    }
                }

                @Override
                public void onAuthenticationError(int errorCode, @NonNull CharSequence errString) {
                    super.onAuthenticationError(errorCode, errString);
                    
                    String error = mapBiometricError(errorCode);
                    WritableMap errorResult = Arguments.createMap();
                    errorResult.putString("error", error);
                    errorResult.putString("message", errString.toString());
                    errorResult.putInt("errorCode", errorCode);
                    
                    // Notify React Native about security event
                    sendEvent(EVENT_BIOMETRIC_ERROR, errorResult);
                    
                    promise.reject(error, errString.toString());
                }

                @Override
                public void onAuthenticationFailed() {
                    super.onAuthenticationFailed();
                    
                    // Record failed attempt for anti-bot analysis
                    securityManager.recordFailedAttempt(userId);
                    
                    WritableMap failResult = Arguments.createMap();
                    failResult.putString("status", "FAILED");
                    failResult.putString("message", "Authentication failed - please try again");
                    
                    sendEvent(EVENT_BIOMETRIC_ERROR, failResult);
                }
            };

            // Initialize biometric prompt
            biometricPrompt = new androidx.biometric.BiometricPrompt(fragmentActivity, executor, authCallback);
            
            // Start authentication
            biometricPrompt.authenticate(promptInfo);
            
        } catch (Exception e) {
            Log.e(TAG, "Error starting biometric authentication", e);
            promise.reject(ERROR_AUTHENTICATION_FAILED, "Failed to start authentication: " + e.getMessage());
        }
    }

    /**
     * Generate biometric signature for anti-bot verification
     * Creates unique signature for human verification system
     */
    @ReactMethod
    public void generateBiometricSignature(ReadableMap data, Promise promise) {
        try {
            String challenge = data.hasKey("challenge") ? data.getString("challenge") : "";
            String userId = data.hasKey("userId") ? data.getString("userId") : "";
            long timestamp = data.hasKey("timestamp") ? (long) data.getDouble("timestamp") : System.currentTimeMillis();
            
            if (challenge.isEmpty() || userId.isEmpty()) {
                promise.reject(ERROR_INVALID_CONTEXT, "Challenge and userId are required");
                return;
            }

            Activity currentActivity = getCurrentActivity();
            if (!(currentActivity instanceof FragmentActivity)) {
                promise.reject(ERROR_INVALID_CONTEXT, "FragmentActivity required for signature generation");
                return;
            }

            // Create crypto object for signature
            Signature signature = securityManager.createSignature();
            androidx.biometric.BiometricPrompt.CryptoObject cryptoObject = 
                new androidx.biometric.BiometricPrompt.CryptoObject(signature);

            androidx.biometric.BiometricPrompt.PromptInfo promptInfo = 
                new androidx.biometric.BiometricPrompt.PromptInfo.Builder()
                    .setTitle("Finova Anti-Bot Verification")
                    .setSubtitle("Prove you're human")
                    .setDescription("Generate biometric signature to verify authenticity")
                    .setNegativeButtonText("Cancel")
                    .setAllowedAuthenticators(androidx.biometric.BiometricManager.Authenticators.BIOMETRIC_STRONG)
                    .build();

            androidx.biometric.BiometricPrompt.AuthenticationCallback callback = 
                new androidx.biometric.BiometricPrompt.AuthenticationCallback() {
                    
                @Override
                public void onAuthenticationSucceeded(@NonNull androidx.biometric.BiometricPrompt.AuthenticationResult result) {
                    try {
                        // Generate signature with biometric-secured key
                        String signatureData = generateSecureSignature(challenge, userId, timestamp, result);
                        
                        WritableMap signatureResult = Arguments.createMap();
                        signatureResult.putString("signature", signatureData);
                        signatureResult.putString("userId", userId);
                        signatureResult.putString("challenge", challenge);
                        signatureResult.putDouble("timestamp", timestamp);
                        signatureResult.putDouble("humanProbability", calculateHumanProbability(userId));
                        signatureResult.putMap("deviceFingerprint", securityManager.getDeviceFingerprint());
                        
                        promise.resolve(signatureResult);
                        
                    } catch (Exception e) {
                        Log.e(TAG, "Error generating biometric signature", e);
                        promise.reject(ERROR_SECURITY_VIOLATION, "Signature generation failed: " + e.getMessage());
                    }
                }

                @Override
                public void onAuthenticationError(int errorCode, @NonNull CharSequence errString) {
                    String error = mapBiometricError(errorCode);
                    promise.reject(error, errString.toString());
                }
            };

            FragmentActivity fragmentActivity = (FragmentActivity) currentActivity;
            biometricPrompt = new androidx.biometric.BiometricPrompt(fragmentActivity, executor, callback);
            biometricPrompt.authenticate(promptInfo, cryptoObject);

        } catch (Exception e) {
            Log.e(TAG, "Error in generateBiometricSignature", e);
            promise.reject(ERROR_AUTHENTICATION_FAILED, "Failed to generate signature: " + e.getMessage());
        }
    }

    /**
     * Verify user's human probability score
     * Implements Finova's AI-powered bot detection
     */
    @ReactMethod
    public void verifyHumanProbability(ReadableMap userData, Promise promise) {
        try {
            String userId = userData.hasKey("userId") ? userData.getString("userId") : "";
            
            if (userId.isEmpty()) {
                promise.reject(ERROR_INVALID_CONTEXT, "User ID is required");
                return;
            }

            // Calculate comprehensive human probability
            double humanProbability = calculateHumanProbability(userId);
            
            WritableMap result = Arguments.createMap();
            result.putDouble("humanProbability", humanProbability);
            result.putString("riskLevel", getRiskLevel(humanProbability));
            result.putMap("behaviorAnalysis", securityManager.analyzeBehaviorPatterns(userId));
            result.putMap("deviceSecurity", securityManager.assessDeviceSecurity());
            result.putBoolean("kycRequired", humanProbability < 0.7);
            result.putDouble("miningPenalty", calculateMiningPenalty(humanProbability));
            
            // Update security bonus based on verification
            updateSecurityBonus(userId, humanProbability);
            
            promise.resolve(result);
            
        } catch (Exception e) {
            Log.e(TAG, "Error verifying human probability", e);
            promise.reject(ERROR_SECURITY_VIOLATION, "Human verification failed: " + e.getMessage());
        }
    }

    /**
     * Cancel ongoing biometric authentication
     */
    @ReactMethod
    public void cancelAuthentication() {
        try {
            if (biometricPrompt != null) {
                // Note: BiometricPrompt doesn't have direct cancel method
                // Authentication will be cancelled when prompt is dismissed
                Log.d(TAG, "Biometric authentication cancellation requested");
            }
            
            if (cancellationSignal != null && !cancellationSignal.isCanceled()) {
                cancellationSignal.cancel();
            }
            
        } catch (Exception e) {
            Log.e(TAG, "Error cancelling authentication", e);
        }
    }

    /**
     * Get current KYC status for user
     */
    @ReactMethod
    public void getKYCStatus(String userId, Promise promise) {
        try {
            WritableMap kycStatus = kycManager.getKYCStatus(userId);
            promise.resolve(kycStatus);
            
        } catch (Exception e) {
            Log.e(TAG, "Error getting KYC status", e);
            promise.reject("KYC_ERROR", "Failed to get KYC status: " + e.getMessage());
        }
    }

    // Private helper methods

    private void processKYCVerification(String userId, androidx.biometric.BiometricPrompt.AuthenticationResult result, Promise promise) {
        try {
            // Generate KYC verification data
            WritableMap kycData = Arguments.createMap();
            kycData.putString("userId", userId);
            kycData.putDouble("timestamp", System.currentTimeMillis());
            kycData.putString("verificationMethod", "BIOMETRIC_FINGERPRINT");
            kycData.putMap("biometricData", extractBiometricData(result));
            kycData.putMap("deviceSecurity", securityManager.assessDeviceSecurity());
            kycData.putDouble("humanProbability", calculateHumanProbability(userId));
            
            // Process KYC through manager
            boolean kycSuccess = kycManager.processKYCVerification(userId, kycData);
            
            if (kycSuccess) {
                // Update security bonus (1.2x multiplier as per whitepaper)
                updateSecurityBonus(userId, 1.2);
                
                WritableMap successResult = Arguments.createMap();
                successResult.putBoolean("success", true);
                successResult.putString("status", "KYC_VERIFIED");
                successResult.putDouble("securityBonusMultiplier", 1.2);
                successResult.putMap("kycData", kycData);
                
                // Notify React Native about successful KYC
                sendEvent(EVENT_KYC_VERIFIED, successResult);
                sendEvent(EVENT_SECURITY_BONUS_UPDATED, successResult);
                
                promise.resolve(successResult);
                
            } else {
                promise.reject("KYC_FAILED", "KYC verification process failed");
            }
            
        } catch (Exception e) {
            Log.e(TAG, "Error processing KYC verification", e);
            promise.reject("KYC_PROCESSING_ERROR", "KYC processing failed: " + e.getMessage());
        }
    }

    private String generateSecureSignature(String challenge, String userId, long timestamp, 
                                         androidx.biometric.BiometricPrompt.AuthenticationResult result) throws Exception {
        
        // Combine data for signature
        String data = challenge + "|" + userId + "|" + timestamp + "|" + System.currentTimeMillis();
        
        // Get signature from crypto object
        Signature signature = result.getCryptoObject().getSignature();
        signature.update(data.getBytes("UTF-8"));
        byte[] signatureBytes = signature.sign();
        
        // Encode signature
        return Base64.encodeToString(signatureBytes, Base64.NO_WRAP);
    }

    private double calculateHumanProbability(String userId) {
        // Implement comprehensive human probability calculation
        // Based on Finova whitepaper factors
        
        double biometricConsistency = securityManager.analyzeBiometricConsistency(userId);
        double behaviorPatterns = securityManager.analyzeBehaviorPatterns(userId).getDouble("humanScore");
        double socialGraphValidity = securityManager.validateSocialConnections(userId);
        double deviceAuthenticity = securityManager.checkDeviceAuthenticity();
        double interactionQuality = securityManager.measureInteractionQuality(userId);
        
        // Weight factors as per whitepaper algorithm
        double weightedScore = (biometricConsistency * 0.25) + 
                              (behaviorPatterns * 0.25) + 
                              (socialGraphValidity * 0.2) + 
                              (deviceAuthenticity * 0.15) + 
                              (interactionQuality * 0.15);
        
        // Clamp between 0.1 and 1.0
        return Math.max(0.1, Math.min(1.0, weightedScore));
    }

    private String getRiskLevel(double humanProbability) {
        if (humanProbability >= 0.9) return "VERY_LOW";
        if (humanProbability >= 0.7) return "LOW";
        if (humanProbability >= 0.5) return "MEDIUM";
        if (humanProbability >= 0.3) return "HIGH";
        return "VERY_HIGH";
    }

    private double calculateMiningPenalty(double humanProbability) {
        // Implement mining penalty based on human probability
        // Lower probability = higher penalty
        if (humanProbability >= 0.9) return 1.0;  // No penalty
        if (humanProbability >= 0.7) return 0.95; // 5% penalty
        if (humanProbability >= 0.5) return 0.8;  // 20% penalty
        if (humanProbability >= 0.3) return 0.5;  // 50% penalty
        return 0.1; // 90% penalty for very suspicious accounts
    }

    private void updateSecurityBonus(String userId, double multiplier) {
        try {
            securityManager.updateSecurityBonus(userId, multiplier);
            
            WritableMap bonusUpdate = Arguments.createMap();
            bonusUpdate.putString("userId", userId);
            bonusUpdate.putDouble("securityBonusMultiplier", multiplier);
            bonusUpdate.putDouble("timestamp", System.currentTimeMillis());
            
            sendEvent(EVENT_SECURITY_BONUS_UPDATED, bonusUpdate);
            
        } catch (Exception e) {
            Log.e(TAG, "Error updating security bonus", e);
        }
    }

    private WritableMap extractBiometricData(androidx.biometric.BiometricPrompt.AuthenticationResult result) {
        WritableMap biometricData = Arguments.createMap();
        
        try {
            biometricData.putString("authenticationType", "FINGERPRINT");
            biometricData.putDouble("timestamp", System.currentTimeMillis());
            biometricData.putBoolean("cryptoObjectUsed", result.getCryptoObject() != null);
            biometricData.putString("deviceId", securityManager.getDeviceId());
            
        } catch (Exception e) {
            Log.e(TAG, "Error extracting biometric data", e);
        }
        
        return biometricData;
    }

    private String mapBiometricError(int errorCode) {
        switch (errorCode) {
            case BiometricConstants.ERROR_USER_CANCEL:
                return ERROR_USER_CANCEL;
            case BiometricConstants.ERROR_LOCKOUT:
            case BiometricConstants.ERROR_LOCKOUT_PERMANENT:
                return ERROR_LOCKOUT;
            case BiometricConstants.ERROR_NO_BIOMETRICS:
                return ERROR_NOT_ENROLLED;
            case BiometricConstants.ERROR_HW_NOT_PRESENT:
            case BiometricConstants.ERROR_HW_UNAVAILABLE:
                return ERROR_NOT_AVAILABLE;
            default:
                return ERROR_AUTHENTICATION_FAILED;
        }
    }

    private void sendEvent(String eventName, WritableMap params) {
        try {
            reactContext
                .getJSModule(DeviceEventManagerModule.RCTDeviceEventEmitter.class)
                .emit(eventName, params);
        } catch (Exception e) {
            Log.e(TAG, "Error sending event: " + eventName, e);
        }
    }

    // Clean up resources
    @Override
    public void onCatalystInstanceDestroy() {
        super.onCatalystInstanceDestroy();
        
        if (cancellationSignal != null) {
            cancellationSignal.cancel();
        }
        
        if (executor != null && !executor.equals(Executors.newSingleThreadExecutor())) {
            // Clean up executor if needed
        }
        
        Log.d(TAG, "BiometricAuthModule destroyed");
    }
}
